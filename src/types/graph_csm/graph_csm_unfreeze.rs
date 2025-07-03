use crate::{CsmGraph, DynamicGraph, Unfreezable};

/// This implementation provides the logic for the "unfreeze" part of the graph's
/// evolutionary lifecycle. It allows a static, high-performance `CsmGraph` to be
/// converted back into a flexible `DynamicGraph`, readying it for a new phase
/// of mutations and evolution.
///
/// # Infallibility and Design
///
/// A key architectural feature of this method is that it is **infallible**—it does
/// not return a `Result`. This is a deliberate design choice based on the
/// following guarantees:
///
/// 1.  **Guaranteed Input Integrity:** The `unfreeze` method consumes `self`, taking
///     ownership of a `CsmGraph`. Within this library's ecosystem, a `CsmGraph`
///     can only be created by the `.freeze()` operation, which guarantees that its
///     internal state (the CSR structures) is always perfectly consistent and valid.
///
/// 2.  **Deterministic Transformation:** The operation is a deterministic data
///     transformation, not a failable action. It deconstructs the highly structured
///     CSR format into the less constrained `Vec<Vec<...>>` adjacency list. Every
///     valid `CsmGraph` has exactly one corresponding `DynamicGraph` representation.
///
/// There are no logical branches in the `unfreeze` process that could result in a
/// user-handleable error. An out-of-bounds access would indicate a critical bug
/// in the `.freeze()` method's construction logic, which would be a `panic`-worthy
/// programmer error, not a runtime failure.
///
/// This infallible signature provides a strong guarantee to the user: transitioning
/// from an analysis state back to an evolutionary state is always a safe and
/// predictable operation.
impl<N, W> Unfreezable<N, W> for CsmGraph<N, W>
where
    N: Clone + Sync + Send,
    W: Clone + Sync + Send + Default,
{
    /// Consumes the `CsmGraph` and converts it back into a mutable `DynamicGraph`.
    ///
    /// This operation reconstructs the adjacency list representation from the CSR
    /// format, allowing the graph to be modified again. It requires `N` and `W`
    /// to be `Clone` because the underlying data is copied into the new,
    /// mutation-friendly structure.
    fn unfreeze(self) -> DynamicGraph<N, W> {
        let num_nodes = self.nodes.len();

        // 1. Convert the compact `Vec<N>` into `Vec<Option<N>>` for the dynamic graph.
        //    This is a simple map operation that wraps each node payload in `Some`.
        let dynamic_nodes: Vec<Option<N>> = self.nodes.into_iter().map(Some).collect();

        // 2. Reconstruct the adjacency list `Vec<Vec<(usize, W)>>` from the CSR format.
        let dynamic_edges: Vec<Vec<(usize, W)>> = (0..num_nodes)
            .map(|i| {
                // Get the slice of neighbors for node `i` using the offsets.
                let start = self.forward_edges.offsets[i];
                let end = self.forward_edges.offsets[i + 1];

                // Get parallel slices for the targets and weights from the SoA structure.
                let targets_slice = &self.forward_edges.targets[start..end];
                let weights_slice = &self.forward_edges.weights[start..end];

                // Use `zip` to combine the parallel slices back into tuples,
                // reconstructing the `Vec<(usize, W)>` for node `i`.
                targets_slice
                    .iter()
                    .zip(weights_slice.iter())
                    .map(|(&target, weight)| (target, weight.clone()))
                    .collect()
            })
            .collect();

        // 3. Construct the new DynamicGraph from its reassembled parts.
        DynamicGraph::construct(dynamic_nodes, dynamic_edges, self.root_index)
    }
}
