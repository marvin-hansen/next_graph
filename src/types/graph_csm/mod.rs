pub struct CsmGraph<N, W> {
    // Node payloads, indexed directly by `usize`.
    nodes: Vec<N>,

    // CSR structure for forward traversal (successors).
    forward_edges: (Vec<usize>, Vec<(usize, W)>),

    // CSR structure for backward traversal (predecessors).
    backward_edges: (Vec<usize>, Vec<(usize, W)>),

    // Index of the designated root node.
    root_index: Option<usize>,
}
