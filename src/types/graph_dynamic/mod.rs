mod graph_freeze;
mod graph_mut;
mod graph_view;

pub struct DynamicGraph<N, W> {
    // Optional pre-allocated capacity for each node's edge list.
    // This is a performance optimization set by `with_capacity`.
    // By default, no edge capacity is pre-allocated.
    num_edges_per_node: Option<usize>,
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
            num_edges_per_node: None,
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
    /// * `num_nodes`: The number of nodes to pre-allocate space for.
    /// * `num_edges_per_node`: An optional hint for the average number of outgoing
    ///   edges per node. Providing this pre-allocates memory for edge lists,
    ///   making `add_edge` calls more performant and predictable by avoiding
    ///   reallocations. If `None`, no capacity is pre-allocated for edges.
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
    /// use next_graph::{DynamicGraph, GraphView}; // Replace with your crate name
    ///
    /// // Pre-allocate for a graph with ~1000 nodes and ~5 edges per node.
    /// let graph = DynamicGraph::<(), ()>::with_capacity(1_000, Some(5));
    ///
    /// // The graph is still empty, but its internal buffers are ready.
    /// assert_eq!(graph.number_nodes(), 0);
    /// ```
    pub fn with_capacity(num_nodes: usize, num_edges_per_node: Option<usize>) -> Self {
        Self {
            num_edges_per_node,
            nodes: Vec::with_capacity(num_nodes),
            edges: Vec::with_capacity(num_nodes),
            root_index: None,
        }
    }
}

impl<N, W> DynamicGraph<N, W> {
    /// Creates a `DynamicGraph` directly from its constituent parts.
    ///
    /// This is the most performant way to construct a graph from an existing, validated
    /// dataset, as it bypasses the per-call overhead of methods like `add_edge`.
    ///
    /// # Preconditions
    /// The caller is responsible for ensuring the integrity of the data. Specifically:
    /// - The length of the `edges` vector must be exactly equal to the length of the `nodes` vector.
    /// - Every `usize` target index within the `edges` lists must be a valid index into the `nodes` vector
    ///   (i.e., less than `nodes.len()`).
    ///
    /// # Panics
    /// This method will panic in debug builds if `nodes.len() != edges.len()`. It may
    /// cause out-of-bounds panics later if the edge index precondition is violated by the caller.
    ///
    /// # Examples
    ///
    /// use next_graph::{DynamicGraph, GraphView};
    ///
    /// // Node payloads: Node 1 is "tombstoned" (removed).
    /// let nodes = vec![Some("A"), None, Some("C")];
    ///
    /// // Adjacency lists:
    /// // Node 0 ("A") -> Node 2 ("C") with weight 10.
    /// // Node 1 (None) has no edges.
    /// // Node 2 ("C") -> Node 0 ("A") with weight 5.
    /// let edges = vec![
    ///     vec![(2, 10)],
    ///     vec![],
    ///     vec![(0, 5)],
    /// ];
    ///
    /// // The root node is at index 0.
    /// let root_index = Some(0);
    ///
    /// let graph = DynamicGraph::from_parts(nodes, edges, root_index);
    ///
    /// assert_eq!(graph.number_nodes(), 2); // Only counts non-tombstoned nodes.
    /// assert_eq!(graph.number_edges(), 2);
    /// assert!(graph.contains_edge(0, 2));
    /// ```
    ///
    pub fn from_parts(
        nodes: Vec<Option<N>>,
        edges: Vec<Vec<(usize, W)>>,
        root_index: Option<usize>,
    ) -> Self {
        // A non-negotiable sanity check. This prevents gross structural mismatches.
        assert_eq!(
            nodes.len(),
            edges.len(),
            "The number of node payloads must equal the number of adjacency lists."
        );

        Self {
            nodes,
            edges,
            root_index,
            // When building from parts, we assume the user has already handled capacity.
            num_edges_per_node: None,
        }
    }

    /// Consumes the graph and returns its raw component parts.
    ///
    /// This method deconstructs the graph into a tuple containing its internal node
    /// vector, adjacency list, and root index. This is an O(1) operation as it
    /// simply moves ownership of the internal data.
    ///
    /// It is the inverse of [`from_parts`], making it useful for serialization or for
    /// moving the graph's data to another system.
    ///
    /// # Returns
    ///
    /// A tuple `(nodes, edges, root_index)` where:
    /// - `nodes`: A `Vec<Option<N>>` of node payloads.
    /// - `edges`: A `Vec<Vec<(usize, W)>>` representing the adjacency list.
    /// - `root_index`: An `Option<usize>` for the root node.
    ///
    pub fn to_parts(self) -> (Vec<Option<N>>, Vec<Vec<(usize, W)>>, Option<usize>) {
        (self.nodes, self.edges, self.root_index)
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
            num_edges_per_node: None,
            nodes,
            edges,
            root_index,
        }
    }
}
