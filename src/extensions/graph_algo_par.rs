#![cfg(feature = "parallel")]

// This entire module only exists if the feature is enabled.
use crate::{CsmGraph, GraphTraversal, GraphView};
use rayon::prelude::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// A trait that provides parallel versions of graph algorithms.
///
/// This trait is intended for implementation on the static `CsmGraph` and leverages
/// Rayon for data parallelism. It is only available when the "parallel" feature
/// is enabled.
///
/// The methods require the node payload `N` and edge weight `W` to be thread-safe.
pub trait ParallelGraphAlgorithms<N, W>: GraphView<N, W>
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
// --- Default implementation for CsmGraph --- ---
//

impl<N, W> ParallelGraphAlgorithms<N, W> for CsmGraph<N, W>
where
    N: Send + Sync,
    W: Send + Sync,
{
    /// Computes a topological sort in parallel using a BFS-like approach (Kahn's Algorithm).
    fn topological_sort_par(&self) -> Option<Vec<usize>> {
        let num_nodes = self.number_nodes();
        if num_nodes == 0 {
            return Some(Vec::new());
        }

        // --- Step 1: Parallel In-Degree Calculation ---
        // Use an array of atomics for thread-safe counting.
        let in_degrees: Vec<AtomicUsize> = (0..num_nodes)
            .into_par_iter()
            .map(|i| AtomicUsize::new(self.inbound_edges(i).unwrap().count()))
            .collect();

        // --- Step 2: Initialize the First Frontier ---
        let mut frontier: Vec<usize> = (0..num_nodes)
            .into_par_iter()
            .filter(|&i| in_degrees[i].load(Ordering::Relaxed) == 0)
            .collect();

        let mut sorted_list = Vec::with_capacity(num_nodes);

        // --- Step 3: Parallel Level-by-Level Processing ---
        while !frontier.is_empty() {
            // Add the completed frontier to our sorted list.
            sorted_list.extend(&frontier);

            // In parallel, find all neighbors of the current frontier, decrement their
            // in-degrees, and collect the ones that drop to zero.
            let next_frontier: Vec<usize> = frontier
                .par_iter()
                .flat_map(|&u| {
                    self.outbound_edges(u)
                        .unwrap()
                        .filter_map(|v| {
                            // `fetch_sub` returns the *previous* value. If it was 1, the new value is 0.
                            if in_degrees[v].fetch_sub(1, Ordering::Relaxed) == 1 {
                                Some(v)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>() // Collect neighbors for this node
                })
                .collect(); // Collect all new frontier nodes from all threads

            frontier = next_frontier;
        }

        // --- Step 4: Validation ---
        if sorted_list.len() == num_nodes {
            Some(sorted_list)
        } else {
            None // A cycle was detected, as not all nodes were processed.
        }
    }

    /// Checks if a path exists from a start to a stop index, using a parallel BFS.
    fn is_reachable_par(&self, start_index: usize, stop_index: usize) -> bool {
        self.shortest_path_len_par(start_index, stop_index)
            .is_some()
    }

    /// Returns the length of the shortest path, using a parallel BFS.
    fn shortest_path_len_par(&self, start_index: usize, stop_index: usize) -> Option<usize> {
        if !self.contains_node(start_index) || !self.contains_node(stop_index) {
            return None;
        }
        if start_index == stop_index {
            return Some(1);
        }

        let num_nodes = self.number_nodes();
        // Use atomic booleans for a thread-safe visited set.
        let visited: Vec<AtomicBool> = (0..num_nodes).map(|_| AtomicBool::new(false)).collect();
        visited[start_index].store(true, Ordering::Relaxed);

        let mut frontier = vec![start_index];
        let mut current_len = 1;

        while !frontier.is_empty() {
            // Check if any thread in the *previous* iteration found the target.
            if self.is_reachable_par(stop_index, stop_index) { /* A bit of a hack to check a flag*/
            }

            let next_frontier: Vec<usize> = frontier
                .par_iter()
                .flat_map(|&u| {
                    self.outbound_edges(u)
                        .unwrap()
                        .filter_map(|v| {
                            // `compare_exchange` is an atomic "check and set". It succeeds only
                            // if the current value is `false`, guaranteeing only one thread
                            // ever "discovers" a node.
                            if visited[v]
                                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                                .is_ok()
                            {
                                Some(v)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect();

            current_len += 1;

            // If any of the newly discovered nodes is the stop index, we're done.
            if next_frontier.par_iter().any(|&v| v == stop_index) {
                return Some(current_len);
            }

            frontier = next_frontier;
        }

        None
    }

    /// Finds the complete shortest path from a start to a stop index, using a parallel BFS.
    fn shortest_path_par(&self, start_index: usize, stop_index: usize) -> Option<Vec<usize>> {
        if !self.contains_node(start_index) || !self.contains_node(stop_index) {
            return None;
        }
        if start_index == stop_index {
            return Some(vec![start_index]);
        }

        let num_nodes = self.number_nodes();
        // Use a Mutex-wrapped Vec for the predecessor map to allow safe concurrent writes.
        let predecessors: Vec<Mutex<Option<usize>>> =
            (0..num_nodes).map(|_| Mutex::new(None)).collect();
        let visited: Vec<AtomicBool> = (0..num_nodes).map(|_| AtomicBool::new(false)).collect();
        visited[start_index].store(true, Ordering::Relaxed);

        let mut frontier = vec![start_index];
        let found = AtomicBool::new(false);

        while !frontier.is_empty() && !found.load(Ordering::Relaxed) {
            frontier = frontier
                .par_iter()
                .flat_map(|&u| {
                    self.outbound_edges(u)
                        .unwrap()
                        .filter_map(|v| {
                            if visited[v]
                                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                                .is_ok()
                            {
                                // Lock and write the predecessor for this newly found node.
                                *predecessors[v].lock().unwrap() = Some(u);
                                if v == stop_index {
                                    found.store(true, Ordering::Relaxed);
                                }
                                Some(v)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect();
        }

        if !found.load(Ordering::Relaxed) {
            return None;
        }

        // Path reconstruction is sequential as it follows a single chain.
        let mut path = Vec::new();
        let mut current = Some(stop_index);
        while let Some(curr_idx) = current {
            path.push(curr_idx);
            // Lock to read the predecessor value.
            current = *predecessors[curr_idx].lock().unwrap();
        }
        path.reverse();
        Some(path)
    }
}
