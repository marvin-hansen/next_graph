use next_graph::{CsmGraph, DynamicGraph, Freezable, GraphMut, GraphView};

// Freezable Trait Tests
#[test]
fn test_freeze_empty_graph() {
    let dynamic_graph = DynamicGraph::<String, u32>::new();
    let csm_graph: CsmGraph<String, u32> = dynamic_graph.freeze();

    assert_eq!(csm_graph.number_nodes(), 0);
    assert_eq!(csm_graph.number_edges(), 0);
    assert!(!csm_graph.contains_root_node());
    assert!(csm_graph.is_frozen());
}

#[test]
fn test_freeze_graph_with_nodes_no_edges() {
    let mut dynamic_graph = DynamicGraph::<String, u32>::new();
    dynamic_graph.add_node("A".to_string());
    dynamic_graph.add_node("B".to_string());
    dynamic_graph.add_root_node("C".to_string());

    let csm_graph: CsmGraph<String, u32> = dynamic_graph.freeze();

    assert_eq!(csm_graph.number_nodes(), 3);
    assert_eq!(csm_graph.number_edges(), 0);
    assert!(csm_graph.contains_node(0));
    assert!(csm_graph.contains_node(1));
    assert!(csm_graph.contains_node(2));
    assert_eq!(csm_graph.get_node(0), Some(&"A".to_string()));
    assert_eq!(csm_graph.get_node(1), Some(&"B".to_string()));
    assert_eq!(csm_graph.get_node(2), Some(&"C".to_string()));
    assert!(csm_graph.contains_root_node());
    assert_eq!(csm_graph.get_root_node(), Some(&"C".to_string()));
    assert_eq!(csm_graph.get_root_index(), Some(2));
}

#[test]
fn test_freeze_graph_with_nodes_and_edges() {
    let mut dynamic_graph = DynamicGraph::<String, u32>::new();
    let n0 = dynamic_graph.add_node("A".to_string());
    let n1 = dynamic_graph.add_node("B".to_string());
    let n2 = dynamic_graph.add_node("C".to_string());
    let n3 = dynamic_graph.add_node("D".to_string());

    dynamic_graph.add_edge(n0, n1, 10).unwrap();
    dynamic_graph.add_edge(n0, n2, 20).unwrap();
    dynamic_graph.add_edge(n1, n2, 30).unwrap();
    dynamic_graph.add_edge(n2, n3, 40).unwrap();
    dynamic_graph.add_edge(n0, n1, 15).unwrap(); // Parallel edge

    let csm_graph: CsmGraph<String, u32> = dynamic_graph.freeze();

    assert_eq!(csm_graph.number_nodes(), 4);
    assert_eq!(csm_graph.number_edges(), 5);

    assert!(csm_graph.contains_edge(n0, n1));
    assert!(csm_graph.contains_edge(n0, n2));
    assert!(csm_graph.contains_edge(n1, n2));
    assert!(csm_graph.contains_edge(n2, n3));

    // Check parallel edge
    let edges_from_n0 = csm_graph.get_edges(n0).unwrap();
    assert_eq!(edges_from_n0.len(), 3); // (n0,n1,10), (n0,n2,20), (n0,n1,15)
    let mut targets_from_n0: Vec<usize> = edges_from_n0.iter().map(|(t, _)| *t).collect();
    targets_from_n0.sort_unstable();
    assert_eq!(targets_from_n0, vec![n1, n1, n2]);

    // Check specific edge weights (requires iterating, as `contains_edge` doesn't check weight)
    let has_edge_0_1_10 = csm_graph
        .get_edges(n0)
        .unwrap()
        .iter()
        .any(|&(t, w)| t == n1 && w == &10);
    let has_edge_0_1_15 = csm_graph
        .get_edges(n0)
        .unwrap()
        .iter()
        .any(|&(t, w)| t == n1 && w == &15);
    assert!(has_edge_0_1_10);
    assert!(has_edge_0_1_15);
}

#[test]
fn test_freeze_graph_with_tombstoned_nodes() {
    let mut dynamic_graph = DynamicGraph::<String, u32>::new();
    let n0 = dynamic_graph.add_node("A".to_string()); // Will be remapped to 0
    let n1 = dynamic_graph.add_node("B".to_string()); // Will be tombstoned
    let n2 = dynamic_graph.add_node("C".to_string()); // Will be remapped to 1
    let n3 = dynamic_graph.add_node("D".to_string()); // Will be remapped to 2

    dynamic_graph.add_edge(n0, n1, 10).unwrap(); // Edge to tombstoned node
    dynamic_graph.add_edge(n0, n2, 20).unwrap();
    dynamic_graph.add_edge(n2, n3, 30).unwrap();
    dynamic_graph.add_edge(n1, n3, 40).unwrap(); // Edge from tombstoned node

    dynamic_graph.add_root_node("ROOT".to_string()); // Add a root node
    let root_idx = dynamic_graph.get_root_index().unwrap();

    dynamic_graph.remove_node(n1).unwrap(); // Tombstone n1 using the public API
    dynamic_graph
        .update_node(root_idx, "NEW_ROOT".to_string())
        .unwrap(); // Update root node

    let csm_graph: CsmGraph<String, u32> = dynamic_graph.freeze();

    assert_eq!(csm_graph.number_nodes(), 4); // n1 should be gone, but root node is new
    assert_eq!(csm_graph.number_edges(), 2); // Edges involving n1 should be gone

    // Check remapping:
    // Old: n0 (idx 0), n1 (idx 1, tombstoned), n2 (idx 2), n3 (idx 3), root (idx 4)
    // New: n0 (idx 0), n2 (idx 1), n3 (idx 2), root (idx 3)
    assert!(csm_graph.contains_node(0));
    assert!(csm_graph.contains_node(1));
    assert!(csm_graph.contains_node(2));
    assert!(csm_graph.contains_node(3));
    assert_eq!(csm_graph.get_node(0), Some(&"A".to_string()));
    assert_eq!(csm_graph.get_node(1), Some(&"C".to_string()));
    assert_eq!(csm_graph.get_node(2), Some(&"D".to_string()));
    assert_eq!(csm_graph.get_node(3), Some(&"NEW_ROOT".to_string()));

    assert!(!csm_graph.contains_node(4)); // Old root index is now out of bounds

    // Check edges after remapping
    assert!(!csm_graph.contains_edge(0, 0)); // Old (n0, n1) -> (new n0, new n0) - should be gone
    assert!(csm_graph.contains_edge(0, 1)); // Old (n0, n2) -> (new n0, new n1)
    assert!(csm_graph.contains_edge(1, 2)); // Old (n2, n3) -> (new n1, new n2)
    assert!(!csm_graph.contains_edge(0, 2)); // Old (n1, n3) -> (new n0, new n2) - should be gone

    // Check root node remapping
    assert!(csm_graph.contains_root_node());
    assert_eq!(csm_graph.get_root_node(), Some(&"NEW_ROOT".to_string()));
    assert_eq!(csm_graph.get_root_index(), Some(3)); // Old root (idx 4) is now new root (idx 3)
}

#[test]
fn test_freeze_graph_with_tombstoned_root_node() {
    let mut dynamic_graph = DynamicGraph::<String, u32>::new();
    let _n0 = dynamic_graph.add_node("A".to_string());
    let _n1 = dynamic_graph.add_node("B".to_string());
    dynamic_graph.add_root_node("ROOT".to_string());
    let root_idx = dynamic_graph.get_root_index().unwrap();

    dynamic_graph.remove_node(root_idx).unwrap(); // Tombstone the root node

    let csm_graph: CsmGraph<String, u32> = dynamic_graph.freeze();

    assert_eq!(csm_graph.number_nodes(), 2);
    assert_eq!(csm_graph.get_node(0), Some(&"A".to_string()));
    assert_eq!(csm_graph.get_node(1), Some(&"B".to_string()));
    assert!(!csm_graph.contains_root_node()); // Root should be gone
    assert_eq!(csm_graph.get_root_index(), None);
}

#[test]
fn test_freeze_graph_with_all_nodes_tombstoned() {
    let mut dynamic_graph = DynamicGraph::<String, u32>::new();
    let n0 = dynamic_graph.add_node("A".to_string());
    let n1 = dynamic_graph.add_node("B".to_string());
    dynamic_graph.add_edge(n0, n1, 10).unwrap();

    dynamic_graph.remove_node(n0).unwrap();
    dynamic_graph.remove_node(n1).unwrap();

    let csm_graph: CsmGraph<String, u32> = dynamic_graph.freeze();

    assert_eq!(csm_graph.number_nodes(), 0);
    assert_eq!(csm_graph.number_edges(), 0);
    assert!(!csm_graph.contains_root_node());
}
