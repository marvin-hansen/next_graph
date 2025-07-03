#![cfg(feature = "parallel")]
// This entire module becomes available only  if the parallel feature is enabled.
use crate::{CsmGraph, GraphTraversal, GraphView};
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// A trait that provides parallel versions of graph algorithms.
///
/// This trait is intended for implementation on the static `CsmGraph` and leverages
/// Rayon for data parallelism. It is only available when the "parallel" feature
/// is enabled.
///
/// The methods require the node payload `N` and edge weight `W` to be thread-safe.
pub trait ParallelGraphAlgorithmsExt<N, W>: GraphView<N, W>
where
    N: Send + Sync,
    W: Send + Sync,
{
    // --- Parallel Structural Validation ---

    /// Computes a topological sort of the graph in parallel using a BFS-like approach (Kahn's Algorithm).
    ///
    /// This can be significantly faster than the sequential version for graphs with high
    /// levels of parallelism (i.e., many nodes with an in-degree of 0 at each step).
    ///
    /// # Returns
    /// `Some(Vec<usize>)` containing the node indices in a valid linear ordering if the
    /// graph is a DAG. Returns `None` if the graph contains a cycle.
    fn topological_sort_par(&self) -> Option<Vec<usize>>;

    // --- Parallel Pathfinding and Reachability Algorithms ---

    /// Checks if a path of any length exists from a start to a stop index, using a parallel BFS.
    ///
    /// This algorithm can be significantly faster on large, wide graphs with many paths.
    fn is_reachable_par(&self, start_index: usize, stop_index: usize) -> bool;

    /// Returns the length of the shortest path from a start to a stop index, using a parallel BFS.
    fn shortest_path_len_par(&self, start_index: usize, stop_index: usize) -> Option<usize>;

    /// Finds the complete shortest path from a start to a stop index, using a parallel BFS.
    fn shortest_path_par(&self, start_index: usize, stop_index: usize) -> Option<Vec<usize>>;
}

//
// --- High-Performance Parallel Implementation for CsmGraph ---
//
impl<N, W> ParallelGraphAlgorithmsExt<N, W> for CsmGraph<N, W>
where
    N: Send + Sync,
    W: Send + Sync + Default,
{
    /// Computes a topological sort of the graph in parallel using Kahn's algorithm.
    ///
    /// This method leverages data parallelism to process nodes, which can be significantly
    /// faster than the sequential version for graphs with a high degree of parallelism
    /// (i.e., graphs that are "wide" rather than "deep").
    ///
    /// The algorithm first performs a fast sequential pass to calculate the in-degrees of all
    /// nodes. It then identifies an initial "frontier" of nodes with zero in-degrees. In each
    /// subsequent step, it processes the current frontier in parallel, atomically decrementing
    /// the in-degrees of neighbor nodes and collecting those that reach an in-degree of zero
    /// into the next frontier.
    ///
    /// # Returns
    ///
    /// - `Some(Vec<usize>)`: If the graph is a Directed Acyclic Graph (DAG), this returns
    ///   a vector of node indices in a valid topological order. The output is made
    ///   deterministic by sorting nodes at each frontier level.
    /// - `None`: If the graph contains a cycle, as a topological sort is not possible.
    ///
    /// # Complexity
    ///
    /// - **Time Complexity:** O(V + E), where V is the number of nodes and E is the number of edges.
    ///   The wall-clock time can be substantially lower on multi-core systems for graphs
    ///   with sufficient parallelism.
    /// - **Space Complexity:** O(V) for storing in-degrees, the frontier, and the result list.
    ///
    fn topological_sort_par(&self) -> Option<Vec<usize>> {
        let num_nodes = self.number_nodes();
        if num_nodes == 0 {
            return Some(Vec::new());
        }

        // 1. Compute in-degrees. A sequential pass is extremely fast and cache-friendly.
        let mut in_degrees_val = vec![0; num_nodes];
        for i in 0..num_nodes {
            // The unwrap is safe because we are iterating within the bounds of existing nodes.
            for neighbor in self.outbound_edges(i).unwrap() {
                in_degrees_val[neighbor] += 1;
            }
        }

        // Convert to atomic integers for safe concurrent modification.
        let in_degrees: Vec<AtomicUsize> =
            in_degrees_val.into_iter().map(AtomicUsize::new).collect();

        // 2. Find the initial frontier of nodes with zero in-degrees in parallel.
        let mut frontier: Vec<usize> = (0..num_nodes)
            .into_par_iter()
            .filter(|&i| in_degrees[i].load(Ordering::Relaxed) == 0)
            .collect();

        let mut sorted_list = Vec::with_capacity(num_nodes);

        // 3. Process frontiers in parallel until no nodes are left.
        while !frontier.is_empty() {
            frontier.sort_unstable(); // For deterministic output
            sorted_list.extend(&frontier);

            let next_frontier: Vec<usize> = frontier
                .par_iter()
                .map(|&u| {
                    // Each parallel task produces a Vec of its "ready" neighbors
                    self.outbound_edges(u)
                        .unwrap()
                        .filter_map(|v| {
                            // Atomically decrement the in-degree. `fetch_sub` returns the *previous* value.
                            if in_degrees[v].fetch_sub(1, Ordering::Relaxed) == 1 {
                                Some(v)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .flatten() // Use Rayon's parallel flatten combinator
                .collect();

            frontier = next_frontier;
        }

        // 4. Validate the result.
        if sorted_list.len() == num_nodes {
            Some(sorted_list)
        } else {
            None // A cycle was detected.
        }
    }

    /// Checks if a path exists from a start to a stop index using a parallel BFS.
    ///
    /// This method is a convenience wrapper around [`shortest_path_len_par`]. It is
    /// most effective on large, "wide" graphs where the parallel search can explore
    /// many paths concurrently.
    ///
    /// # Arguments
    ///
    /// * `start_index`: The index of the node where the path should start.
    /// * `stop_index`: The index of the node where the path should end.
    ///
    /// # Returns
    ///
    /// - `true` if a path exists from `start_index` to `stop_index`.
    /// - `false` if no such path exists or if the indices are invalid.
    ///
    fn is_reachable_par(&self, start_index: usize, stop_index: usize) -> bool {
        self.shortest_path_len_par(start_index, stop_index)
            .is_some()
    }

    /// Finds the length of the shortest path from a start to a stop index using a parallel BFS.
    ///
    /// This method implements a parallel Breadth-First Search (BFS) to find the shortest
    /// path length in terms of the number of edges. At each step of the BFS, it expands
    /// the entire frontier of nodes in parallel. This can provide a significant speedup
    /// for graphs where the number of nodes at each distance from the source is large.
    ///
    /// Thread-safe visitation is managed using a `Vec<AtomicBool>`.
    ///
    /// # Arguments
    ///
    /// * `start_index`: The index of the node where the path should start.
    /// * `stop_index`: The index of the node where the path should end.
    ///
    /// # Returns
    ///
    /// - `Some(usize)`: The length of the shortest path, including the start and end nodes
    ///   (i.e., number of nodes in the path). A path from a node to itself has length 1.
    /// - `None`: If no path exists or if the indices are invalid.
    fn shortest_path_len_par(&self, start_index: usize, stop_index: usize) -> Option<usize> {
        if !self.contains_node(start_index) || !self.contains_node(stop_index) {
            return None;
        }
        if start_index == stop_index {
            return Some(1);
        }

        let num_nodes = self.number_nodes();
        // Use atomic booleans to safely track visited nodes across threads.
        let visited: Vec<AtomicBool> = (0..num_nodes).map(|_| AtomicBool::new(false)).collect();
        visited[start_index].store(true, Ordering::Relaxed);

        let mut frontier = vec![start_index];
        let mut distance = 1;

        while !frontier.is_empty() {
            distance += 1;

            // In parallel, expand the entire frontier.
            let next_frontier: Vec<usize> = frontier
                .par_iter()
                .map(|&u| {
                    // The unwrap is safe because we are dealing with valid node indices.
                    self.outbound_edges(u)
                        .unwrap()
                        .filter_map(|v| {
                            // Atomically "claim" the node. `swap` returns the previous value.
                            // If it was `false`, this thread is the first to visit it.
                            if !visited[v].swap(true, Ordering::Relaxed) {
                                Some(v)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect();

            // If the target is in the new frontier, we've found the shortest path length.
            if next_frontier.par_iter().any(|&v| v == stop_index) {
                return Some(distance);
            }

            if next_frontier.is_empty() {
                return None; // No path found.
            }
            frontier = next_frontier;
        }
        None
    }

    /// Finds the complete shortest path from a start to a stop index using a parallel BFS.
    ///
    /// This method implements a parallel Breadth-First Search (BFS) that tracks predecessors
    /// to reconstruct the path. It is optimized to stop all parallel tasks as soon as the
    /// target node is found by any thread, preventing unnecessary work.
    ///
    /// Thread-safe predecessor tracking is managed using a `Vec<AtomicUsize>` and atomic
    /// `compare_exchange` operations to "claim" nodes. An `AtomicBool` is used to signal
    /// when the target is found.
    ///
    /// # Arguments
    ///
    /// * `start_index`: The index of the node where the path should start.
    /// * `stop_index`: The index of the node where the path should end.
    ///
    /// # Returns
    ///
    /// - `Some(Vec<usize>)`: A vector of node indices representing the shortest path from
    ///   the start to the stop node.
    /// - `None`: If no path exists or if the indices are invalid.
    fn shortest_path_par(&self, start_index: usize, stop_index: usize) -> Option<Vec<usize>> {
        let num_nodes = self.number_nodes();
        if !self.contains_node(start_index) || !self.contains_node(stop_index) {
            return None;
        }
        if start_index == stop_index {
            return Some(vec![start_index]);
        }

        const UNVISITED: usize = usize::MAX;
        // Use an atomic vector to safely store the predecessor of each node.
        let predecessors: Vec<AtomicUsize> = (0..num_nodes)
            .map(|_| AtomicUsize::new(UNVISITED))
            .collect();

        // Mark the start node as visited by setting its predecessor to itself.
        predecessors[start_index].store(start_index, Ordering::Relaxed);

        let mut frontier = vec![start_index];
        let target_found = AtomicBool::new(false);

        'bfs_loop: while !frontier.is_empty() && !target_found.load(Ordering::Relaxed) {
            let next_frontier: Vec<usize> = frontier
                .par_iter()
                .map(|&u| {
                    // Early exit for this task if another thread has already found the target.
                    if target_found.load(Ordering::Relaxed) {
                        return Vec::new();
                    }

                    self.outbound_edges(u)
                        .unwrap()
                        .filter_map(|v| {
                            // Atomically "claim" the node by setting its predecessor.
                            // `compare_exchange` only succeeds if the current value is `UNVISITED`.
                            if predecessors[v]
                                .compare_exchange(
                                    UNVISITED,
                                    u,
                                    Ordering::Relaxed,
                                    Ordering::Relaxed,
                                )
                                .is_ok()
                            {
                                // If we found the target, set the flag to stop other threads.
                                if v == stop_index {
                                    target_found.store(true, Ordering::Relaxed);
                                }
                                Some(v)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect();

            if target_found.load(Ordering::Relaxed) {
                break 'bfs_loop;
            }

            if next_frontier.is_empty() {
                return None; // No path found.
            }
            frontier = next_frontier;
        }

        // If the target was found, reconstruct the path.
        if target_found.load(Ordering::Relaxed) {
            let mut path = Vec::new();
            let mut current = stop_index;
            // Walk backwards from the target to the source.
            while current != start_index {
                path.push(current);
                let pred = predecessors[current].load(Ordering::Relaxed);
                // This check protects against race conditions, though it's unlikely to fail.
                if pred == UNVISITED {
                    return None;
                }
                current = pred;
            }
            path.push(start_index);
            path.reverse();
            Some(path)
        } else {
            None
        }
    }
}
