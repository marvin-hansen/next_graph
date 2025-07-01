pub struct DynamicGraph<N, W> {
    // Node payloads, indexed directly by `usize`.
    // The use of `Option` allows for efficient O(1) node removal ("tombstoning")
    // without invalidating other node indices.
    nodes: Vec<Option<N>>,

    // Adjacency list: A vector where each index corresponds to a source node,
    // and the value is a list of its outgoing edges.
    edges: Vec<Vec<(usize, W)>>,

    // Index of the designated root node.
    root_index: Option<usize>,
}

impl<N, W> DynamicGraph<N, W> {
    pub fn construct(
        nodes: Vec<Option<N>>,
        edges: Vec<Vec<(usize, W)>>,
        root_index: Option<usize>,
    ) -> Self {
        Self {
            nodes,
            edges,
            root_index,
        }
    }
}
