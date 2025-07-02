use crate::{CsmGraph, GraphError, GraphTraversal, GraphView};

impl<N: Sync + Send, W: Sync + Send> GraphTraversal<N, W> for CsmGraph<N, W> {
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
}
