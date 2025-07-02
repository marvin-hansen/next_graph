use crate::{CsmGraph, GraphTraversal};

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

/// This private `impl` block contains internal helper logic for the `CsmGraph`.
impl<N: Sync + Send, W: Sync + Send> CsmGraph<N, W> {
    pub(crate) fn dfs_visit_for_cycle(
        &self,
        u: usize,
        states: &mut [NodeState],
        predecessors: &mut [Option<usize>],
    ) -> Option<Vec<usize>> {
        states[u] = NodeState::VisitingInProgress; // Mark as "in progress"

        //
        if let Ok(neighbors) = self.outbound_edges(u) {
            for v in neighbors {
                // If we encounter a node that is currently in the "Visiting" state,
                // we have found a back edge, which means we have a cycle.
                if states[v] == NodeState::VisitingInProgress {
                    // Handle self-loops explicitly
                    if u == v {
                        return Some(vec![u, u]);
                    }
                    let mut cycle = vec![u];
                    let mut current = u;
                    while let Some(pred) = predecessors[current] {
                        if pred == v {
                            cycle.push(v);
                            cycle.reverse();
                            return Some(cycle);
                        }
                        cycle.push(pred);
                        current = pred;
                    }
                }

                // If the neighbor is unvisited, explore it.
                if states[v] == NodeState::Unvisited {
                    predecessors[v] = Some(u);
                    if let Some(path) = self.dfs_visit_for_cycle(v, states, predecessors) {
                        return Some(path); // Propagate the found cycle up
                    }
                }
                // If states[v] is `Visited`, we do nothing. That branch is known to be safe.
            }
        }

        // We have explored all paths from `u` and found no cycles. Mark it as fully visited.
        states[u] = NodeState::Visited;
        None
    }

    // Add this new helper function inside your private `impl<N, W> CsmGraph<N, W>` block
    pub(crate) fn dfs_visit_for_sort(
        &self,
        u: usize,
        states: &mut [NodeState],
        sorted_list: &mut Vec<usize>,
    ) -> Result<(), ()> {
        // Returns Ok(()) on success, Err(()) if a cycle is found
        states[u] = NodeState::VisitingInProgress;

        if let Ok(neighbors) = self.outbound_edges(u) {
            for v in neighbors {
                if states[v] == NodeState::VisitingInProgress {
                    // Cycle detected! Abort immediately.
                    return Err(());
                }

                if states[v] == NodeState::Unvisited {
                    // If a cycle is found in a deeper path, propagate the error up.
                    if self.dfs_visit_for_sort(v, states, sorted_list).is_err() {
                        return Err(());
                    }
                }
            }
        }

        // This is the crucial step for topological sort.
        // After visiting all of u's children, u is "finished." We add it to our list.
        states[u] = NodeState::Visited;
        sorted_list.push(u);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DynamicGraph, Freezable, GraphMut, GraphView};

    #[test]
    fn test_dfs_visit_for_cycle_visited_neighbor() {
        let mut dynamic_graph = DynamicGraph::new();
        let n0 = dynamic_graph.add_node("A".to_string());
        let n1 = dynamic_graph.add_node("B".to_string());
        let n2 = dynamic_graph.add_node("C".to_string());
        dynamic_graph.add_edge(n0, n1, 1).unwrap();
        dynamic_graph.add_edge(n1, n2, 1).unwrap();
        let graph = dynamic_graph.freeze();

        // Manually set n2 as visited to test the `if states[v] is Visited` branch
        // This is a white-box test, directly manipulating internal state for coverage.
        let mut states = vec![NodeState::Unvisited; graph.number_nodes()];
        let mut predecessors = vec![None; graph.number_nodes()];
        states[n2] = NodeState::Visited;

        // Start DFS from n0. n1 will be visited, then n2 will be encountered as Visited.
        let cycle = graph.dfs_visit_for_cycle(n0, &mut states, &mut predecessors);
        assert_eq!(cycle, None);
    }

    #[test]
    fn test_dfs_visit_for_sort_cycle_detection() {
        let mut dynamic_graph = DynamicGraph::new();
        let n0 = dynamic_graph.add_node("A".to_string());
        let n1 = dynamic_graph.add_node("B".to_string());
        dynamic_graph.add_edge(n0, n1, 1).unwrap();
        dynamic_graph.add_edge(n1, n0, 1).unwrap(); // Cycle
        let graph = dynamic_graph.freeze();

        let mut states = vec![NodeState::Unvisited; graph.number_nodes()];
        let mut sorted_list = Vec::new();

        // Starting DFS from n0 should detect the cycle
        let result = graph.dfs_visit_for_sort(n0, &mut states, &mut sorted_list);
        assert!(result.is_err());
    }

    #[test]
    fn test_dfs_visit_for_cycle_deep_cycle() {
        let mut dynamic_graph = DynamicGraph::<String, u32>::new();
        let n0 = dynamic_graph.add_node("A".to_string());
        let n1 = dynamic_graph.add_node("B".to_string());
        let n2 = dynamic_graph.add_node("C".to_string());
        let n3 = dynamic_graph.add_node("D".to_string());

        dynamic_graph.add_edge(n0, n1, 1).unwrap();
        dynamic_graph.add_edge(n1, n2, 1).unwrap();
        dynamic_graph.add_edge(n2, n3, 1).unwrap();
        dynamic_graph.add_edge(n3, n1, 1).unwrap(); // Cycle: n1 -> n2 -> n3 -> n1

        let graph = dynamic_graph.freeze();

        let mut states = vec![NodeState::Unvisited; graph.number_nodes()];
        let mut predecessors = vec![None; graph.number_nodes()];

        let cycle = graph.dfs_visit_for_cycle(n0, &mut states, &mut predecessors);
        assert!(cycle.is_some());
        let path = cycle.unwrap();
        assert_eq!(path.len(), 3); // n1, n2, n3
        assert!(path.contains(&n1));
        assert!(path.contains(&n2));
        assert!(path.contains(&n3));
    }

    #[test]
    fn test_dfs_visit_for_sort_deep_cycle() {
        let mut dynamic_graph = DynamicGraph::<String, u32>::new();
        let n0 = dynamic_graph.add_node("A".to_string());
        let n1 = dynamic_graph.add_node("B".to_string());
        let n2 = dynamic_graph.add_node("C".to_string());
        let n3 = dynamic_graph.add_node("D".to_string());

        dynamic_graph.add_edge(n0, n1, 1).unwrap();
        dynamic_graph.add_edge(n1, n2, 1).unwrap();
        dynamic_graph.add_edge(n2, n3, 1).unwrap();
        dynamic_graph.add_edge(n3, n1, 1).unwrap(); // Cycle: n1 -> n2 -> n3 -> n1

        let graph = dynamic_graph.freeze();

        let mut states = vec![NodeState::Unvisited; graph.number_nodes()];
        let mut sorted_list = Vec::new();

        // Starting DFS from n0 should eventually hit the cycle and return Err
        let result = graph.dfs_visit_for_sort(n0, &mut states, &mut sorted_list);
        assert!(result.is_err());
    }

    #[test]
    fn test_dfs_visit_for_cycle_no_outgoing_edges() {
        let mut dynamic_graph = DynamicGraph::<String, u32>::new();
        let n1 = dynamic_graph.add_node("B".to_string());
        let graph = dynamic_graph.freeze();

        let mut states = vec![NodeState::Unvisited; graph.number_nodes()];
        let mut predecessors = vec![None; graph.number_nodes()];

        // Call dfs_visit_for_cycle on a node with no outgoing edges
        let cycle = graph.dfs_visit_for_cycle(n1, &mut states, &mut predecessors);
        assert_eq!(cycle, None);
        assert_eq!(states[n1], NodeState::Visited);
    }

    #[test]
    fn test_dfs_visit_for_sort_no_outgoing_edges() {
        let mut dynamic_graph = DynamicGraph::<String, u32>::new();
        let n1 = dynamic_graph.add_node("B".to_string());
        let graph = dynamic_graph.freeze();

        let mut states = vec![NodeState::Unvisited; graph.number_nodes()];
        let mut sorted_list = Vec::new();

        // Call dfs_visit_for_sort on a node with no outgoing edges
        let result = graph.dfs_visit_for_sort(n1, &mut states, &mut sorted_list);
        assert!(result.is_ok());
        assert_eq!(states[n1], NodeState::Visited);
        assert!(sorted_list.contains(&n1));
    }

    #[test]
    fn test_dfs_visit_for_sort_visited_neighbor() {
        let mut dynamic_graph = DynamicGraph::<String, u32>::new();
        let n0 = dynamic_graph.add_node("A".to_string());
        let n1 = dynamic_graph.add_node("B".to_string());
        let n2 = dynamic_graph.add_node("C".to_string());
        dynamic_graph.add_edge(n0, n1, 1).unwrap();
        dynamic_graph.add_edge(n1, n2, 1).unwrap();
        let graph = dynamic_graph.freeze();

        let mut states = vec![NodeState::Unvisited; graph.number_nodes()];
        let mut sorted_list = Vec::new();

        // Manually set n2 as visited before starting DFS from n0
        states[n2] = NodeState::Visited;

        // Start DFS from n0. n1 will be visited, then n2 will be encountered as Visited.
        let result = graph.dfs_visit_for_sort(n0, &mut states, &mut sorted_list);
        assert!(result.is_ok());
        // Ensure n0 and n1 are in the sorted list, and n2 is not re-added.
        assert!(sorted_list.contains(&n0));
        assert!(sorted_list.contains(&n1));
        assert!(!sorted_list.contains(&n2)); // n2 was already visited, so it shouldn't be added by this DFS call
    }
}
