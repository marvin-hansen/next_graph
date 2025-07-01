use crate::{CsmGraph, DynamicGraph, Freezable};

// This implementation requires that the edge weight `W` is both `Clone` (to be
// duplicated for the forward and backward graphs) and `Default` (to allow for
// efficient, safe allocation of the final adjacency vectors).
impl<N: Clone, W: Clone + Default> Freezable<N, W> for DynamicGraph<N, W> {
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
        // --- Phase 1: Compaction and Index Remapping ---
        // This crucial step removes any `None` nodes (tombstones) and creates a mapping
        // from the old, potentially sparse indices to the new, compact indices.
        let mut new_nodes = Vec::with_capacity(self.nodes.len());
        let mut remapping_table = vec![None; self.nodes.len()]; // old_index -> Option<new_index>
        for (old_index, node_opt) in self.nodes.into_iter().enumerate() {
            if let Some(node_payload) = node_opt {
                let new_index = new_nodes.len();
                new_nodes.push(node_payload);
                remapping_table[old_index] = Some(new_index);
            }
        }

        let num_new_nodes = new_nodes.len();
        if num_new_nodes == 0 {
            // Handle the edge case of an empty or all-tombstoned graph.
            return CsmGraph::construct(
                Vec::new(),
                (vec![0], Vec::new()),
                (vec![0], Vec::new()),
                None,
            );
        }

        // --- Phase 2: Edge Remapping and Degree Counting ---
        // We iterate through all old edges once. In this single pass, we remap their
        // indices and count the in-degrees and out-degrees for the new, compact nodes.
        let mut remapped_edges = Vec::new();
        let mut out_degrees = vec![0; num_new_nodes];
        let mut in_degrees = vec![0; num_new_nodes];

        for (old_source_idx, edge_list) in self.edges.iter().enumerate() {
            // Only process edges from non-tombstoned source nodes.
            if let Some(new_source_idx) = remapping_table[old_source_idx] {
                for (old_target_idx, weight) in edge_list {
                    // Only process edges to non-tombstoned target nodes.
                    if let Some(new_target_idx) = remapping_table[*old_target_idx] {
                        remapped_edges.push((new_source_idx, new_target_idx, weight.clone()));
                        out_degrees[new_source_idx] += 1;
                        in_degrees[new_target_idx] += 1;
                    }
                }
            }
        }
        let total_edges = remapped_edges.len();

        // --- Phase 3: Calculate CSR Offsets ---
        // Use the degree counts to calculate the final `offsets` vectors for both CSRs
        // via a cumulative sum (prefix sum).
        let mut fwd_offsets = vec![0; num_new_nodes + 1];
        let mut back_offsets = vec![0; num_new_nodes + 1];
        for i in 0..num_new_nodes {
            fwd_offsets[i + 1] = fwd_offsets[i] + out_degrees[i];
            back_offsets[i + 1] = back_offsets[i] + in_degrees[i];
        }

        // --- Phase 4: Final Edge Placement (No intermediate allocations) ---
        // This pass places each edge into its final sorted position in both the
        // forward and backward adjacency vectors using the offsets as write-heads.
        let mut fwd_adj = vec![Default::default(); total_edges];
        let mut back_adj = vec![Default::default(); total_edges];

        // Use copies of offsets as counters for the next available slot for each node.
        let mut fwd_write_heads = fwd_offsets.clone();
        let mut back_write_heads = back_offsets.clone();

        for (source, target, weight) in remapped_edges {
            // Place in forward adjacency list
            let fwd_pos = fwd_write_heads[source];
            fwd_adj[fwd_pos] = (target, weight.clone());
            fwd_write_heads[source] += 1;

            // Place in backward adjacency list
            let back_pos = back_write_heads[target];
            back_adj[back_pos] = (source, weight);
            back_write_heads[target] += 1;
        }

        // --- Step 5: Sort Adjacency Lists ---
        // For each node, sort its slice of neighbors for fast binary search later.
        for i in 0..num_new_nodes {
            let fwd_slice = &mut fwd_adj[fwd_offsets[i]..fwd_offsets[i + 1]];
            fwd_slice.sort_unstable_by_key(|(target, _)| *target);

            let back_slice = &mut back_adj[back_offsets[i]..back_offsets[i + 1]];
            back_slice.sort_unstable_by_key(|(source, _)| *source);
        }

        // --- Step 6: Final Assembly ---
        // Remap the root index and construct the final, optimized CsmGraph.
        let new_root_index = self.root_index.and_then(|old_idx| remapping_table[old_idx]);

        // --- Step 7: Return the Final CSM Graph ---
        CsmGraph::construct(
            new_nodes,
            (fwd_offsets, fwd_adj),
            (back_offsets, back_adj),
            new_root_index,
        )
    }
}
