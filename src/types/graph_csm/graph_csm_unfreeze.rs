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
impl<N: Clone + Sync + Send, W: Clone + Sync + Send> Unfreezable<N, W> for CsmGraph<N, W> {
    /// Consumes the static graph to create a dynamic, mutable representation.
    ///
    /// This is an O(V + E) operation that re-builds the flexible adjacency list
    /// structure from the compact CSR format, allowing the graph to re-enter
    /// an evolutionary phase.
    fn unfreeze(self) -> DynamicGraph<N, W> {
        let num_nodes = self.nodes.len();

        // --- Step 1: Deconstruct the CsmGraph ---
        // We take ownership of the internal parts of the CsmGraph.
        let CsmGraph {
            nodes: csm_nodes,
            forward_edges: (offsets, adjacencies),
            backward_edges: _, // The backward edges are not needed to reconstruct the forward graph.
            root_index,
        } = self;

        // --- Step 2: Reconstruct the Dynamic Node List ---
        // The DynamicGraph uses `Vec<Option<N>>` to handle "tombstoning".
        // We convert every node `N` into `Some(N)`.
        let dynamic_nodes: Vec<Option<N>> = csm_nodes.into_iter().map(Some).collect();

        // --- Step 3: Reconstruct the Adjacency List (`Vec<Vec<...>>`) ---
        // This is the core of the unfreeze operation. We iterate through each node's
        // slice in the CSR `adjacencies` vector and use it to build the
        // corresponding inner `Vec` in our new adjacency list.
        let mut dynamic_edges = Vec::with_capacity(num_nodes);
        for i in 0..num_nodes {
            let start = offsets[i];
            let end = offsets[i + 1];

            // Get the slice of edges for node `i` from the CSR data.
            let edge_slice = &adjacencies[start..end];

            // Create a new Vec for this node's edges by cloning the data from the slice.
            dynamic_edges.push(edge_slice.to_vec());
        }

        // --- Step 4: Construct the new DynamicGraph ---
        DynamicGraph::construct(dynamic_nodes, dynamic_edges, root_index)
    }
}
