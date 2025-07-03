use crate::types::graph_csm::CsrAdjacency;
use crate::{CsmGraph, DynamicGraph, Freezable};

// This implementation requires that the edge weight `W` is both `Clone` (to be
// duplicated for the forward and backward graphs) and `Default` (to allow for
// efficient, safe allocation of the final adjacency vectors).
impl<N, W> Freezable<N, W> for DynamicGraph<N, W>
where
    N: Clone,
    W: Clone + Default,
{
    /// Consumes the dynamic graph to create a static, high-performance `CsmGraph`.
    ///
    /// This is a computationally intensive "stop the world" operation with O(V + E)
    /// complexity. It performs several key optimizations:
    ///   1. Removes "tombstoned" (deleted) nodes and re-indexes the graph.
    ///   2. Builds both forward and backward CSR structures in a single pass to save memory.
    ///   3. Sorts all adjacency lists for fast O(log degree) edge checking in the `CsmGraph`.
    ///
    /// # Infallibility and Design
    ///
    /// This method is **infallible** and does not return a `Result`. This is a
    /// deliberate architectural choice based on the following guarantees:
    ///
    /// 1.  **Guaranteed Input Integrity:** The `freeze` method consumes `self`, taking
    ///     ownership of a `DynamicGraph`. The internal structure of `DynamicGraph`,
    ///     while flexible, is always in a valid, well-defined state (e.g., its
    ///     `nodes` and `edges` vectors are guaranteed to be of equal length).
    ///
    /// 2.  **Deterministic Transformation:** The entire operation is a deterministic
    ///     data transformation. It converts one valid data structure into another.
    ///     Every step—compacting nodes, remapping indices, counting degrees, and
    ///     placing edges—is based on the state of the input graph. There are no
    ///     external inputs or logical conditions that can cause a recoverable runtime
    ///     failure.
    ///
    /// An error such as an out-of-bounds index during this process would indicate a
    /// critical bug in the `freeze` implementation itself. Such a programmer error
    /// should rightly cause a `panic`, as it represents an invalid program state,
    /// not a runtime error that a user could handle. This infallible signature
    /// provides a strong guarantee: transitioning from an evolutionary state to an
    /// analysis state is always a safe and predictable operation.
    fn freeze(self) -> CsmGraph<N, W> {
        // --- First Pass: Counting and Compacting ---

        let old_nodes = self.nodes;
        let old_edges = self.edges;
        let old_root_index = self.root_index;

        let mut compacted_nodes = Vec::with_capacity(old_nodes.len());
        let mut remapping_table = vec![0; old_nodes.len()];
        // **IMPROVEMENT**: Explicitly track which nodes were removed.
        let mut is_tombstoned = vec![false; old_nodes.len()];
        let mut new_root_index = None;
        let mut total_edges = 0;

        // Compact the node list, create the remapping table, and track tombstones.
        for (old_index, node_opt) in old_nodes.into_iter().enumerate() {
            if let Some(node) = node_opt {
                let new_index = compacted_nodes.len();
                remapping_table[old_index] = new_index;
                compacted_nodes.push(node);

                if old_root_index == Some(old_index) {
                    new_root_index = Some(new_index);
                }
            } else {
                // Mark this index as tombstoned for later checks.
                is_tombstoned[old_index] = true;
            }
        }
        compacted_nodes.shrink_to_fit();
        let num_new_nodes = compacted_nodes.len();

        if num_new_nodes == 0 {
            return CsmGraph::new();
        }

        // Count degrees for the new, compacted graph.
        let mut out_degrees = vec![0; num_new_nodes];
        let mut in_degrees = vec![0; num_new_nodes];

        for (old_source_idx, edge_list) in old_edges.iter().enumerate() {
            // **IMPROVEMENT**: Use the clear, unambiguous tombstone check.
            if !is_tombstoned[old_source_idx] {
                let new_source_idx = remapping_table[old_source_idx];
                for (old_target_idx, _) in edge_list {
                    // Also ensure the target wasn't tombstoned.
                    if !is_tombstoned[*old_target_idx] {
                        let new_target_idx = remapping_table[*old_target_idx];
                        out_degrees[new_source_idx] += 1;
                        in_degrees[new_target_idx] += 1;
                        total_edges += 1;
                    }
                }
            }
        }

        // --- Offset Calculation (Cumulative Sum) ---
        let fwd_offsets = calculate_offsets(&out_degrees);
        let back_offsets = calculate_offsets(&in_degrees);

        // --- Second Pass: Placement ---
        let mut fwd_targets = vec![0; total_edges];
        let mut fwd_weights = vec![W::default(); total_edges];
        let mut back_targets = vec![0; total_edges];
        let mut back_weights = vec![W::default(); total_edges];

        let mut fwd_offsets_copy = fwd_offsets.clone();
        let mut back_offsets_copy = back_offsets.clone();

        // Place each edge into its correct position in the CSR arrays.
        // We can now safely use `into_iter` to avoid cloning weights unnecessarily.
        for (old_source_idx, edge_list) in old_edges.into_iter().enumerate() {
            // **IMPROVEMENT**: Use the same robust check here.
            if !is_tombstoned[old_source_idx] {
                let new_source_idx = remapping_table[old_source_idx];
                for (old_target_idx, weight) in edge_list {
                    if !is_tombstoned[old_target_idx] {
                        let new_target_idx = remapping_table[old_target_idx];

                        // Forward placement
                        let fwd_write_head = fwd_offsets_copy[new_source_idx];
                        fwd_targets[fwd_write_head] = new_target_idx;
                        fwd_weights[fwd_write_head] = weight.clone(); // Clone for backward edge
                        fwd_offsets_copy[new_source_idx] += 1;

                        // Backward placement
                        let back_write_head = back_offsets_copy[new_target_idx];
                        back_targets[back_write_head] = new_source_idx;
                        back_weights[back_write_head] = weight; // Move the original weight
                        back_offsets_copy[new_target_idx] += 1;
                    }
                }
            }
        }

        // Sort the adjacency lists for each node to enable binary search lookups.
        sort_adjacencies(&fwd_offsets, &mut fwd_targets, &mut fwd_weights);
        sort_adjacencies(&back_offsets, &mut back_targets, &mut back_weights);

        // --- Final Construction ---
        let forward_edges = CsrAdjacency {
            offsets: fwd_offsets,
            targets: fwd_targets,
            weights: fwd_weights,
        };
        let backward_edges = CsrAdjacency {
            offsets: back_offsets,
            targets: back_targets,
            weights: back_weights,
        };

        CsmGraph::construct(
            compacted_nodes,
            forward_edges,
            backward_edges,
            new_root_index,
        )
    }
}

/// Helper function to calculate CSR offsets from a degree count vector.
fn calculate_offsets(degrees: &[usize]) -> Vec<usize> {
    let mut offsets = Vec::with_capacity(degrees.len() + 1);
    let mut total = 0;
    offsets.push(total);
    for &degree in degrees {
        total += degree;
        offsets.push(total);
    }
    offsets
}

/// Helper function to sort the adjacency lists within the CSR arrays.
fn sort_adjacencies<W>(offsets: &[usize], targets: &mut [usize], weights: &mut [W])
where
    W: Clone,
{
    for i in 0..offsets.len() - 1 {
        let start = offsets[i];
        let end = offsets[i + 1];
        if start < end {
            // Create a temporary Vec of tuples to sort by target index.
            let mut slice_to_sort: Vec<_> = targets[start..end]
                .iter()
                .zip(weights[start..end].iter())
                .map(|(&t, w)| (t, w.clone()))
                .collect();

            // Sort unstably by target index, which is faster.
            slice_to_sort.sort_unstable_by_key(|(target, _)| *target);

            // Write the sorted data back to the main arrays.
            for (j, (target, weight)) in slice_to_sort.into_iter().enumerate() {
                targets[start + j] = target;
                weights[start + j] = weight;
            }
        }
    }
}
