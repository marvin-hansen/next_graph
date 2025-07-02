use next_graph::{DynamicGraph, GraphError, GraphMut, GraphView};

#[test]
fn test_add_node() {
    let mut graph = DynamicGraph::<String, u32>::new();
    let node_idx = graph.add_node("A".to_string());
    assert_eq!(node_idx, 0);
    assert_eq!(graph.number_nodes(), 1);
    assert!(graph.contains_node(0));
    assert_eq!(graph.get_node(0), Some(&"A".to_string()));

    let node_idx_1 = graph.add_node("B".to_string());
    assert_eq!(node_idx_1, 1);
    assert_eq!(graph.number_nodes(), 2);
    assert!(graph.contains_node(1));
    assert_eq!(graph.get_node(1), Some(&"B".to_string()));
}

#[test]
fn test_update_node() {
    let mut graph = DynamicGraph::<String, u32>::new();
    let node_idx = graph.add_node("A".to_string());
    assert_eq!(graph.get_node(node_idx), Some(&"A".to_string()));

    // Update existing node
    assert!(graph.update_node(node_idx, "Z".to_string()).is_ok());
    assert_eq!(graph.get_node(node_idx), Some(&"Z".to_string()));

    // Update non-existent node (out of bounds)
    assert_eq!(
        graph.update_node(99, "X".to_string()),
        Err(GraphError::NodeNotFound(99))
    );

    // Update tombstoned node
    let node_to_remove = graph.add_node("TO_REMOVE".to_string());
    graph.remove_node(node_to_remove).unwrap();
    assert_eq!(
        graph.update_node(node_to_remove, "X".to_string()),
        Err(GraphError::NodeNotFound(node_to_remove))
    );
}

#[test]
fn test_remove_node() {
    let mut graph = DynamicGraph::<String, u32>::new();
    let n0 = graph.add_node("A".to_string());
    let n1 = graph.add_node("B".to_string());
    let n2 = graph.add_node("C".to_string());
    graph.add_edge(n0, n1, 10).unwrap();
    graph.add_root_node("ROOT".to_string());

    assert_eq!(graph.number_nodes(), 4);
    assert!(graph.contains_node(n0));
    assert!(graph.contains_node(n1));
    assert!(graph.contains_node(n2));
    assert!(graph.contains_root_node());

    // Remove an existing node
    assert!(graph.remove_node(n1).is_ok());
    assert!(!graph.contains_node(n1));
    assert_eq!(graph.number_nodes(), 3); // Only counts non-tombstoned nodes

    // Try to remove an already removed node
    assert_eq!(graph.remove_node(n1), Err(GraphError::NodeNotFound(n1)));

    // Try to remove a non-existent node
    assert_eq!(graph.remove_node(99), Err(GraphError::NodeNotFound(99)));

    // Check that root node is cleared if removed
    let root_idx = graph.get_root_index().unwrap();
    assert!(graph.remove_node(root_idx).is_ok());
    assert!(!graph.contains_root_node());
}

#[test]
fn test_add_edge() {
    let mut graph = DynamicGraph::<String, u32>::new();
    let node_a = graph.add_node("A".to_string());
    let node_b = graph.add_node("B".to_string());
    let node_c = graph.add_node("C".to_string());

    // Add a valid edge
    assert!(graph.add_edge(node_a, node_b, 10).is_ok());
    assert_eq!(graph.number_edges(), 1);
    assert!(graph.contains_edge(node_a, node_b));

    // Add another valid edge
    assert!(graph.add_edge(node_a, node_c, 20).is_ok());
    assert_eq!(graph.number_edges(), 2);
    assert!(graph.contains_edge(node_a, node_c));

    // Add parallel edge
    assert!(graph.add_edge(node_a, node_b, 15).is_ok());
    assert_eq!(graph.number_edges(), 3);
    assert!(graph.contains_edge(node_a, node_b)); // Still true

    // Add edge with non-existent source
    assert_eq!(
        graph.add_edge(99, node_b, 5),
        Err(GraphError::EdgeCreationError {
            source: 99,
            target: node_b
        })
    );

    // Add edge with non-existent target
    assert_eq!(
        graph.add_edge(node_a, 99, 5),
        Err(GraphError::EdgeCreationError {
            source: node_a,
            target: 99
        })
    );

    // Add edge to/from a removed node
    let removed_node = graph.add_node("REMOVED".to_string());
    graph.remove_node(removed_node).unwrap();
    assert_eq!(
        graph.add_edge(node_a, removed_node, 5),
        Err(GraphError::EdgeCreationError {
            source: node_a,
            target: removed_node
        })
    );
    assert_eq!(
        graph.add_edge(removed_node, node_a, 5),
        Err(GraphError::EdgeCreationError {
            source: removed_node,
            target: node_a
        })
    );
}

#[test]
fn test_remove_edge() {
    let mut graph = DynamicGraph::<String, u32>::new();
    let node_a = graph.add_node("A".to_string());
    let node_b = graph.add_node("B".to_string());
    let node_c = graph.add_node("C".to_string());

    graph.add_edge(node_a, node_b, 10).unwrap();
    graph.add_edge(node_a, node_c, 20).unwrap();
    graph.add_edge(node_a, node_b, 15).unwrap(); // Parallel edge

    assert_eq!(graph.number_edges(), 3);
    assert!(graph.contains_edge(node_a, node_b));

    // Remove an existing edge
    assert!(graph.remove_edge(node_a, node_c).is_ok());
    assert_eq!(graph.number_edges(), 2);
    assert!(!graph.contains_edge(node_a, node_c));

    // Remove one of the parallel edges
    assert!(graph.remove_edge(node_a, node_b).is_ok());
    assert_eq!(graph.number_edges(), 1);
    assert!(graph.contains_edge(node_a, node_b)); // One parallel edge still exists

    // Try to remove a non-existent edge
    assert_eq!(
        graph.remove_edge(node_a, node_c),
        Err(GraphError::EdgeNotFoundError {
            source: node_a,
            target: node_c
        })
    );

    // Try to remove edge from non-existent source node
    assert_eq!(
        graph.remove_edge(99, node_b),
        Err(GraphError::NodeNotFound(99))
    );
}

#[test]
fn test_add_root_node() {
    let mut graph = DynamicGraph::<String, u32>::new();
    let _node_a = graph.add_node("A".to_string());
    let root_idx = graph.add_root_node("ROOT".to_string());

    assert_eq!(root_idx, 1); // 0 was "A", 1 is "ROOT"
    assert!(graph.contains_root_node());
    assert_eq!(graph.get_root_node(), Some(&"ROOT".to_string()));
    assert_eq!(graph.get_root_index(), Some(root_idx));

    // Overwrite existing root
    let new_root_idx = graph.add_root_node("NEW_ROOT".to_string());
    assert_eq!(new_root_idx, 2);
    assert!(graph.contains_root_node());
    assert_eq!(graph.get_root_node(), Some(&"NEW_ROOT".to_string()));
    assert_eq!(graph.get_root_index(), Some(new_root_idx));
}

#[test]
fn test_clear() {
    let mut graph = DynamicGraph::<String, u32>::new();
    graph.add_node("A".to_string());
    graph.add_node("B".to_string());
    graph.add_edge(0, 1, 10).unwrap();
    graph.add_root_node("ROOT".to_string());

    assert_eq!(graph.number_nodes(), 3);
    assert_eq!(graph.number_edges(), 1);
    assert!(graph.contains_root_node());

    graph.clear();

    assert_eq!(graph.number_nodes(), 0);
    assert_eq!(graph.number_edges(), 0);
    assert!(!graph.contains_root_node());
}
