use crate::types::graph_csm::graph_dfs_utils::NodeState;
use crate::{CsmGraph, GraphAlgorithms, GraphError, GraphView};
use std::collections::VecDeque;

impl<N, W> GraphAlgorithms<N, W> for CsmGraph<N, W> {
    /// Returns a non-allocating iterator over the direct successors (outgoing edges) of node `a`.
    fn outbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError> {
        if !self.contains_node(a) {
            return Err(GraphError::NodeNotFound(a));
        }

        let (offsets, adjacencies) = &self.forward_edges;
        let start = offsets[a];
        let end = offsets[a + 1];

        // This map adapter over a slice iterator is the concrete type the compiler needs.
        let iter = adjacencies[start..end]
            .iter()
            .map(|(target, _weight)| *target);
        Ok(iter)
    }

    /// Returns a non-allocating iterator over the direct predecessors (incoming edges) of node `a`.
    fn inbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError> {
        if !self.contains_node(a) {
            return Err(GraphError::NodeNotFound(a));
        }

        let (offsets, adjacencies) = &self.backward_edges;
        let start = offsets[a];
        let end = offsets[a + 1];

        let iter = adjacencies[start..end]
            .iter()
            .map(|(source, _weight)| *source);
        Ok(iter)
    }

    /// Finds a single cycle in the graph and returns the path of nodes that form it.
    fn find_cycle(&self) -> Option<Vec<usize>> {
        let num_nodes = self.number_nodes();
        if num_nodes == 0 {
            return None;
        }

        // The tracking vector now uses the self-documenting NodeState enum.
        let mut states = vec![NodeState::Unvisited; num_nodes];
        let mut predecessors = vec![None; num_nodes];

        for i in 0..num_nodes {
            if states[i] == NodeState::Unvisited {
                if let Some(cycle) = self.dfs_visit_for_cycle(i, &mut states, &mut predecessors) {
                    return Some(cycle);
                }
            }
        }
        None
    }

    fn has_cycle(&self) -> bool {
        self.find_cycle().is_some()
    }

    fn topological_sort(&self) -> Option<Vec<usize>> {
        let num_nodes = self.number_nodes();
        if num_nodes == 0 {
            return Some(Vec::new());
        }

        // The tracking vector uses your NodeState enum.
        let mut states = vec![NodeState::Unvisited; num_nodes];
        // The list will store the reverse topological sort.
        let mut sorted_list = Vec::with_capacity(num_nodes);

        // Iterate through all nodes to handle disconnected graphs.
        for i in 0..num_nodes {
            if states[i] == NodeState::Unvisited {
                // Call dfs_visit_for_sort. If it returns an error, a cycle was found,
                // so a topological sort is impossible.
                if self
                    .dfs_visit_for_sort(i, &mut states, &mut sorted_list)
                    .is_err()
                {
                    return None; // Cycle detected, return None as per the trait contract.
                }
            }
        }

        // The DFS produces nodes in "reverse finishing order," which is a reverse
        // topological sort. We must reverse the list to get the correct order.
        sorted_list.reverse();

        Some(sorted_list)
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
        while let Some(current_node) = queue.pop_front() {
            if current_node == stop_index {
                found = true;
                break;
            }
            for neighbor in self.outbound_edges(current_node).unwrap() {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    predecessors[neighbor] = Some(current_node);
                    queue.push_back(neighbor);
                }
            }
        }

        if !found {
            return None;
        }

        // Reconstruct path by walking backwards from the stop index.
        let mut path = Vec::new();
        let mut current = stop_index;
        while let Some(pred_index) = predecessors[current] {
            path.push(current);
            current = pred_index;
        }
        path.push(start_index);
        path.reverse();
        Some(path)
    }
}
