use next_graph::{DynamicGraph, GraphMut, GraphView};

#[test]
fn test_new_dynamic_graph() {
    let graph = DynamicGraph::<String, u32>::new();
    assert_eq!(graph.number_nodes(), 0);
    assert_eq!(graph.number_edges(), 0);
    assert!(!graph.contains_root_node());
    assert!(!graph.is_frozen());
}

#[test]
fn test_with_capacity_dynamic_graph() {
    let graph = DynamicGraph::<String, u32>::with_capacity(10, None);
    assert_eq!(graph.number_nodes(), 0);
    assert_eq!(graph.number_edges(), 0);
    assert!(!graph.contains_root_node());
    assert!(!graph.is_frozen());

    // Crate with capacity of 10 nodes and each node with capacity of 4 edges per node
    let graph = DynamicGraph::<String, u32>::with_capacity(10, Some(4));
    assert_eq!(graph.number_nodes(), 0);
    assert_eq!(graph.number_edges(), 0);
    assert!(!graph.contains_root_node());
    assert!(!graph.is_frozen());
}

#[test]
fn test_default_dynamic_graph() {
    let graph = DynamicGraph::<String, u32>::default();
    assert_eq!(graph.number_nodes(), 0);
    assert_eq!(graph.number_edges(), 0);
    assert!(!graph.contains_root_node());
    assert!(!graph.is_frozen());
}

#[test]
fn test_default_root_index() {
    let graph = DynamicGraph::<String, u32>::default();
    assert_eq!(graph.number_nodes(), 0);
    assert_eq!(graph.number_edges(), 0);
    assert!(graph.root_index().is_none());
    assert!(!graph.contains_root_node());
    assert!(!graph.is_frozen());
}

#[test]
fn test_from_parts_and_to_parts_round_trip() {
    // 1. Define the component parts for a graph.
    //    Node at index 2 is "tombstoned".
    let original_nodes: Vec<Option<String>> = vec![
        Some("A".to_string()),
        Some("B".to_string()),
        None,
        Some("D".to_string()),
    ];
    let original_edges: Vec<Vec<(usize, i32)>> = vec![
        vec![(1, 10)], // A -> B
        vec![(3, 20)], // B -> D
        vec![],        // Tombstoned node has no edges
        vec![(0, 30)], // D -> A
    ];
    let original_root = Some(0);

    // 2. Construct the graph from these parts.
    let graph = DynamicGraph::from_parts(
        original_nodes.clone(),
        original_edges.clone(),
        original_root,
    );

    // 3. Verify the state of the constructed graph using the public API.
    assert_eq!(graph.number_nodes(), 3); // 4 total slots, 3 active nodes
    assert_eq!(graph.number_edges(), 3);
    assert_eq!(graph.get_root_index(), Some(0));
    assert!(graph.contains_edge(0, 1));
    assert!(!graph.contains_node(2)); // Index 2 is a tombstone

    // 4. Deconstruct the graph back into its parts.
    let (nodes_after, edges_after, root_after) = graph.to_parts();

    // 5. Verify that the deconstructed parts match the original input.
    assert_eq!(original_nodes, nodes_after);
    assert_eq!(original_edges, edges_after);
    assert_eq!(original_root, root_after);
}

#[test]
fn test_parts_with_empty_graph() {
    let original_nodes: Vec<Option<()>> = vec![];
    let original_edges: Vec<Vec<(usize, ())>> = vec![];
    let original_root = None;

    // Create and immediately deconstruct an empty graph.
    let graph = DynamicGraph::from_parts(
        original_nodes.clone(),
        original_edges.clone(),
        original_root,
    );
    let (nodes_after, edges_after, root_after) = graph.to_parts();

    // Verify the parts are still empty.
    assert!(nodes_after.is_empty());
    assert!(edges_after.is_empty());
    assert_eq!(root_after, None);
}

#[test]
#[should_panic(expected = "The number of node payloads must equal the number of adjacency lists.")]
fn test_from_parts_panics_on_mismatched_lengths() {
    // Create input data where the number of nodes does not match the number of edge lists.
    let nodes: Vec<Option<()>> = vec![Some(()), Some(())]; // len = 2
    let edges: Vec<Vec<(usize, ())>> = vec![vec![]]; // len = 1
    let root_index = None;

    // This call is expected to panic due to the assertion inside `from_parts`.
    // The `#[should_panic]` attribute makes the test pass if the panic occurs.
    DynamicGraph::from_parts(nodes, edges, root_index);
}

#[test]
fn test_to_parts_on_iteratively_built_graph() {
    // This test ensures `to_parts` works correctly on a graph
    // not built with `from_parts`.
    let mut graph = DynamicGraph::<&str, u32>::new();
    let n0 = graph.add_node("A");
    let n1 = graph.add_node("B");
    graph.add_edge(n0, n1, 100).unwrap();
    graph.remove_node(n0).unwrap(); // Tombstone node 0

    let (nodes, edges, root) = graph.to_parts();

    // Expected state after mutations
    let expected_nodes = vec![None, Some("B")];
    // Edges from node 0 were cleared upon removal
    let expected_edges: Vec<Vec<(usize, u32)>> = vec![vec![], vec![]];
    let expected_root = None;

    assert_eq!(nodes, expected_nodes);
    assert_eq!(edges, expected_edges);
    assert_eq!(root, expected_root);
}
