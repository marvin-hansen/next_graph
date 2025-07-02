use next_graph::{DynamicGraph, GraphView};

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
    let graph = DynamicGraph::<String, u32>::with_capacity(10);
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
