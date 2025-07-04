use next_graph::{CsmGraph, GraphView};

#[test]
fn test_new_csm_graph() {
    let graph = CsmGraph::<String, u32>::new();
    assert_eq!(graph.number_nodes(), 0);
    assert_eq!(graph.number_edges(), 0);
    assert!(!graph.contains_root_node());
    assert!(graph.is_frozen());
}

#[test]
fn test_with_capacity_csm_graph() {
    let graph = CsmGraph::<String, u32>::with_capacity(10);
    assert_eq!(graph.number_nodes(), 0);
    assert_eq!(graph.number_edges(), 0);
    assert!(!graph.contains_root_node());
    assert!(graph.is_frozen());
}

#[test]
fn test_default_csm_graph() {
    let graph = CsmGraph::<String, u32>::default();
    assert_eq!(graph.number_nodes(), 0);
    assert_eq!(graph.number_edges(), 0);
    assert!(!graph.contains_root_node());
    assert!(graph.is_frozen());
}
