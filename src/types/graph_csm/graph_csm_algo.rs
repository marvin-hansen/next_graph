use crate::{CsmGraph, GraphAlgorithms, GraphTraversal, GraphView};
use std::collections::VecDeque;
use std::slice;

/// Private enum representing the state of a node during a DFS traversal.
/// It is not part of the public API.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NodeState {
    /// The node has not yet been visited.
    Unvisited,
    /// The node is currently in the recursion stack (being visited).
    VisitingInProgress,
    /// The node and all its descendants have been fully visited and are known to be cycle-free.
    Visited,
}

impl<N, W> GraphAlgorithms<N, W> for CsmGraph<N, W>
where
    N: Sync + Send,
    W: Sync + Send + Default,
{
    /// Finds a cycle in the graph using an iterative Depth-First Search (DFS).
    ///
    /// This method traverses the graph to detect a back edge, which indicates a cycle.
    /// The iterative approach using an explicit stack makes it robust against stack
    /// overflows, even for very deep or large graphs. The search is comprehensive and
    /// will find a cycle in any of the graph's disconnected components if one exists.
    ///
    /// # Returns
    ///
    /// - `Some(Vec<usize>)`: If a cycle is found, this returns a vector of node indices
    ///   that form the cycle. The path explicitly starts and ends with the same node
    ///   to represent the closed loop (e.g., `[1, 2, 0, 1]`). For a self-loop on
    ///   node `n`, it returns `vec![n, n]`.
    /// - `None`: If the graph is a Directed Acyclic Graph (DAG) and contains no cycles.
    ///
    /// # Complexity
    ///
    /// - **Time Complexity:** O(V + E), where V is the number of nodes and E is the
    ///   number of edges, as each node and edge is visited exactly once.
    /// - **Space Complexity:** O(V) for storing node states, predecessors, and the DFS stack.
    ///
    /// # Example
    ///
    ///
    /// Checks if the graph contains any directed cycles.
    ///
    /// This implementation leverages the iterative `topological_sort` method,
    /// making it a highly robust way to detect cycles in graphs of any size
    /// without risking a stack overflow.
    ///
    /// use next_graph::{DynamicGraph, Freezable, GraphAlgorithms, GraphMut};
    ///
    /// // --- Graph with a cycle ---
    /// let mut dynamic_graph = DynamicGraph::new();
    /// let n0 = dynamic_graph.add_node("A");
    /// let n1 = dynamic_graph.add_node("B");
    /// let n2 = dynamic_graph.add_node("C");
    ///
    /// dynamic_graph.add_edge(n0, n1, ()).unwrap();
    /// dynamic_graph.add_edge(n1, n2, ()).unwrap();
    /// dynamic_graph.add_edge(n2, n0, ()).unwrap(); // Cycle: 0 -> 1 -> 2 -> 0
    ///
    /// let graph = dynamic_graph.freeze();
    /// let cycle = graph.find_cycle();
    ///
    /// assert!(cycle.is_some());
    /// let path = cycle.unwrap();
    /// // The path explicitly shows the closed loop, e.g., [0, 1, 2, 0].
    /// assert_eq!(path.len(), 4);
    /// assert_eq!(path.first(), path.last());
    /// assert!(path.contains(&n0));
    /// assert!(path.contains(&n1));
    /// assert!(path.contains(&n2));
    ///
    /// // --- Graph with no cycle (DAG) ---
    /// let mut dag = DynamicGraph::new();
    /// let n0 = dag.add_node("A");
    /// let n1 = dag.add_node("B");
    /// dag.add_edge(n0, n1, ()).unwrap();
    ///
    /// let graph_no_cycle = dag.freeze();
    /// assert_eq!(graph_no_cycle.find_cycle(), None);
    /// ```
    fn find_cycle(&self) -> Option<Vec<usize>> {
        let num_nodes = self.number_nodes();
        if num_nodes == 0 {
            return None;
        }

        let mut states = vec![NodeState::Unvisited; num_nodes];
        let mut predecessors = vec![None; num_nodes];

        for i in 0..num_nodes {
            if states[i] == NodeState::Unvisited {
                // We use the concrete `slice::Iter` type to avoid Box and dynamic dispatch.
                let mut stack: Vec<(usize, slice::Iter<'_, usize>)> = Vec::new();

                // By accessing the internal fields directly, we bypass the opaque `impl Trait`
                // returned by the `outbound_edges` trait method.
                let start = self.forward_edges.offsets[i];
                let end = self.forward_edges.offsets[i + 1];
                let neighbors = self.forward_edges.targets[start..end].iter();

                states[i] = NodeState::VisitingInProgress;
                stack.push((i, neighbors));

                while let Some((u_ref, neighbors)) = stack.last_mut() {
                    let u = *u_ref;

                    if let Some(&v) = neighbors.next() {
                        if states[v] == NodeState::VisitingInProgress {
                            // --- Cycle Found: Reconstruct the Path ---
                            let mut path = vec![u];
                            let mut current = u;

                            while let Some(predecessor) = predecessors[current] {
                                path.push(predecessor);
                                if predecessor == v {
                                    break;
                                }
                                current = predecessor;
                            }
                            path.reverse();
                            path.push(v); // Make the cycle explicit: [v, ..., u, v]
                            return Some(path);
                        }

                        if states[v] == NodeState::Unvisited {
                            predecessors[v] = Some(u);
                            states[v] = NodeState::VisitingInProgress;

                            // --- FIX: Manually create the concrete iterator here as well ---
                            let v_start = self.forward_edges.offsets[v];
                            let v_end = self.forward_edges.offsets[v + 1];
                            let v_neighbors = self.forward_edges.targets[v_start..v_end].iter();
                            stack.push((v, v_neighbors));
                        }
                    } else {
                        // If the neighbor iterator is exhausted, we are done with this node.
                        states[u] = NodeState::Visited;
                        stack.pop();
                    }
                }
            }
        }

        None // No cycles found after checking all nodes.
    }

    fn has_cycle(&self) -> bool {
        self.topological_sort().is_none()
    }

    /// Computes a topological sort of the graph using the iterative Kahn's algorithm.
    ///
    /// This method is robust against stack overflows, making it suitable for graphs
    /// of any size. It works by repeatedly finding nodes with no incoming edges.
    ///
    /// # Returns
    /// - `Some(Vec<usize>)` if the graph is a Directed Acyclic Graph (DAG). The
    ///   vector contains the nodes in a valid topological order.
    /// - `None` if the graph contains a cycle, as a topological sort is not possible.
    fn topological_sort(&self) -> Option<Vec<usize>> {
        let num_nodes = self.number_nodes();
        if num_nodes == 0 {
            return Some(Vec::new());
        }

        // --- Kahn's Algorithm Implementation ---

        // 1. Compute in-degrees for all nodes. This is an O(V+E) operation.
        let mut in_degrees = vec![0; num_nodes];
        // This loop structure is generally more cache-friendly than iterating
        // inbound_edges for each node individually.
        for i in 0..num_nodes {
            if let Ok(neighbors) = self.outbound_edges(i) {
                for neighbor in neighbors {
                    in_degrees[neighbor] += 1;
                }
            }
        }

        // 2. Initialize a queue with all nodes that have an in-degree of 0.
        let mut queue = VecDeque::with_capacity(num_nodes);
        for i in 0..num_nodes {
            if in_degrees[i] == 0 {
                queue.push_back(i);
            }
        }

        // 3. Process the queue.
        let mut sorted_list = Vec::with_capacity(num_nodes);
        while let Some(u) = queue.pop_front() {
            sorted_list.push(u);

            // For each neighbor of the dequeued node, decrement its in-degree.
            if let Ok(neighbors) = self.outbound_edges(u) {
                for v in neighbors {
                    in_degrees[v] -= 1;
                    // If a neighbor's in-degree becomes 0, it's ready to be processed.
                    if in_degrees[v] == 0 {
                        queue.push_back(v);
                    }
                }
            }
        }

        // 4. Validate the result. If the sorted list includes all nodes, the sort
        // was successful. Otherwise, a cycle was present and prevented some nodes
        // from being processed.
        if sorted_list.len() == num_nodes {
            Some(sorted_list)
        } else {
            None // Cycle detected
        }
    }

    /// Checks if a path exists from a start to a stop index.
    fn is_reachable(&self, start_index: usize, stop_index: usize) -> bool {
        self.shortest_path_len(start_index, stop_index).is_some()
    }

    /// Returns the length of the shortest path (in number of nodes) from a start to a stop index.
    fn shortest_path_len(&self, start_index: usize, stop_index: usize) -> Option<usize> {
        if !self.contains_node(start_index) || !self.contains_node(stop_index) {
            return None;
        }
        if start_index == stop_index {
            return Some(1);
        }

        let mut queue = VecDeque::new();
        let mut visited = vec![false; self.number_nodes()];

        queue.push_back((start_index, 1)); // (node, path_length)
        visited[start_index] = true;

        while let Some((current_node, current_len)) = queue.pop_front() {
            // The unwrap() is safe here because we've already confirmed the node exists.
            for neighbor in self.outbound_edges(current_node).unwrap() {
                if neighbor == stop_index {
                    return Some(current_len + 1);
                }
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back((neighbor, current_len + 1));
                }
            }
        }
        None
    }

    /// Finds the complete shortest path from a start to a stop index.
    fn shortest_path(&self, start_index: usize, stop_index: usize) -> Option<Vec<usize>> {
        if !self.contains_node(start_index) || !self.contains_node(stop_index) {
            return None;
        }
        if start_index == stop_index {
            return Some(vec![start_index]);
        }

        let mut queue = VecDeque::new();
        let mut predecessors = vec![None; self.number_nodes()];
        let mut visited = vec![false; self.number_nodes()];

        queue.push_back(start_index);
        visited[start_index] = true;

        let mut found = false;
        'bfs_loop: while let Some(current_node) = queue.pop_front() {
            // The unwrap() is safe here because we've already confirmed the node exists.
            for neighbor in self.outbound_edges(current_node).unwrap() {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    predecessors[neighbor] = Some(current_node);
                    queue.push_back(neighbor);

                    if neighbor == stop_index {
                        found = true;
                        break 'bfs_loop;
                    }
                }
            }
        }

        if !found {
            return None;
        }

        // Reconstruct path by walking backwards from the stop index.
        let mut path = Vec::new();
        let mut current = Some(stop_index);
        while let Some(curr_index) = current {
            path.push(curr_index);
            current = predecessors[curr_index];
        }
        path.reverse();
        Some(path)
    }
}
