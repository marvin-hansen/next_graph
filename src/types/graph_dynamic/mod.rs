mod graph_freeze;
mod graph_mut;
mod graph_view;

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
    pub fn root_index(&self) -> Option<usize> {
        self.root_index
    }
}

impl<N, W> DynamicGraph<N, W> {
    /// Creates a new, empty `DynamicGraph`.
    ///
    /// The graph is initialized with no nodes, no edges, and no capacity. This is
    /// ideal for building a graph when the final size is unknown.
    ///
    /// # Examples
    ///
    /// ```
    /// use next_graph::DynamicGraph; // Replace with your crate name
    ///
    /// let graph = DynamicGraph::<String, u32>::new();
    /// // assert_eq!(graph.number_nodes(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            root_index: None,
        }
    }

    /// Creates a new, empty `DynamicGraph` with pre-allocated capacity.
    ///
    /// This is the most efficient way to start building a large graph when the
    /// approximate final size is known, as it can reduce or eliminate costly
    /// memory reallocations during the `add_node` process.
    ///
    /// # Arguments
    ///
    /// * `num_nodes`: The number of nodes to pre-allocate space for. This will reserve
    ///   capacity in both the node list and the primary adjacency list.
    ///
    /// # Note on Capacity
    ///
    /// This method pre-allocates the main vector for nodes and the outer vector for
    /// the adjacency list. It does not pre-allocate the inner vectors for each
    /// node's specific edge list, as their individual sizes are not known upfront.
    ///
    /// # Examples
    ///
    /// ```
    /// use next_graph::DynamicGraph; // Replace with your crate name
    ///
    /// // Pre-allocate for a graph with around 1,000 nodes.
    /// let graph = DynamicGraph::<(), ()>::with_capacity(1_000);
    /// // The graph is still empty, but its internal buffers are ready.
    /// // assert_eq!(graph.number_nodes(), 0);
    /// ```
    pub fn with_capacity(num_nodes: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(num_nodes),
            edges: Vec::with_capacity(num_nodes),
            root_index: None,
        }
    }
}

impl<N, W> Default for DynamicGraph<N, W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N, W> DynamicGraph<N, W> {
    // Internal helper for unfreeze
    pub(crate) fn construct(
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
