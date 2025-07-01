mod graph_algo;
mod graph_dfs_utils;
mod graph_unfreeze;
mod graph_view;

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

impl<N, W> CsmGraph<N, W> {
    // In `impl<N, W> CsmGraph<N, W>`

    /// Creates a new, empty `CsmGraph`.
    ///
    /// The graph will have zero nodes and zero edges.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),

            // The forward CSR is empty. The offsets vector must contain a single `0`
            // to correctly represent the `V + 1` length rule, where V=0.
            forward_edges: (vec![0], Vec::new()),

            // The backward CSR is also empty.
            backward_edges: (vec![0], Vec::new()),

            root_index: None,
        }
    }

    /// Creates a new, empty `DynamicGraph` with pre-allocated capacity.
    ///
    /// This is the most efficient way to start building a graph when the approximate
    /// final size is known, as it can reduce or eliminate memory reallocations
    /// during the `add_node` and `add_edge` process.
    ///
    /// # Arguments
    /// * `num_nodes`: The number of nodes to pre-allocate space for.
    pub fn with_capacity(num_nodes: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(num_nodes),
            // The forward CSR is empty. The offsets vector must contain a single `0`
            // to correctly represent the `V + 1` length rule, where V=0.
            forward_edges: (vec![0], Vec::new()),

            // The backward CSR is also empty.
            backward_edges: (vec![0], Vec::new()),

            root_index: None,
        }
    }
}

impl<N, W> Default for CsmGraph<N, W> {
    fn default() -> Self {
        Self::new()
    }
}
