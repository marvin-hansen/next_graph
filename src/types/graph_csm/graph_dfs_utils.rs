use crate::{CsmGraph, GraphAlgorithms};

/// Private enum representing the state of a node during a DFS traversal.
/// It is not part of the public API.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum NodeState {
    /// The node has not yet been visited.
    Unvisited,
    /// The node is currently in the recursion stack (being visited).
    VisitingInProgress,
    /// The node and all its descendants have been fully visited and are known to be cycle-free.
    Visited,
}

/// This private `impl` block contains internal helper logic for the `CsmGraph`.
impl<N, W> CsmGraph<N, W> {
    pub(crate) fn dfs_visit_for_cycle(
        &self,
        u: usize,
        states: &mut [NodeState],
        predecessors: &mut [Option<usize>],
    ) -> Option<Vec<usize>> {
        states[u] = NodeState::VisitingInProgress; // Mark as "in progress"

        if let Ok(neighbors) = self.outbound_edges(u) {
            for v in neighbors {
                // If we encounter a node that is currently in the "Visiting" state,
                // we have found a back edge, which means we have a cycle.
                if states[v] == NodeState::VisitingInProgress {
                    // CYCLE DETECTED!
                    let mut cycle = vec![v, u];
                    let mut current = u;
                    while let Some(pred) = predecessors[current] {
                        cycle.push(pred);
                        if pred == v {
                            cycle.reverse();
                            return Some(cycle);
                        }
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
