use next_graph::{CsmGraph, DynamicGraph, Freezable, GraphMut, GraphView, Unfreezable};

#[test]
fn test_unfreeze_empty_graph() {
    let csm_graph = CsmGraph::<String, u32>::new();
    let dynamic_graph: DynamicGraph<String, u32> = csm_graph.unfreeze();

    assert_eq!(dynamic_graph.number_nodes(), 0);
    assert_eq!(dynamic_graph.number_edges(), 0);
    assert!(!dynamic_graph.contains_root_node());
    assert!(!dynamic_graph.is_frozen());
}

#[test]
fn test_unfreeze_graph_with_nodes_no_edges() {
    let mut dynamic_graph_orig = DynamicGraph::new();
    dynamic_graph_orig.add_node("A".to_string());
    dynamic_graph_orig.add_node("B".to_string());
    dynamic_graph_orig.add_root_node("C".to_string());

    let csm_graph = dynamic_graph_orig.freeze();
    let dynamic_graph: DynamicGraph<String, u32> = csm_graph.unfreeze();

    assert_eq!(dynamic_graph.number_nodes(), 3);
    assert_eq!(dynamic_graph.number_edges(), 0);
    assert!(dynamic_graph.contains_node(0));
    assert!(dynamic_graph.contains_node(1));
    assert!(dynamic_graph.contains_node(2));
    assert_eq!(dynamic_graph.get_node(0), Some(&"A".to_string()));
    assert_eq!(dynamic_graph.get_node(1), Some(&"B".to_string()));
    assert_eq!(dynamic_graph.get_node(2), Some(&"C".to_string()));
    assert!(dynamic_graph.contains_root_node());
    assert_eq!(dynamic_graph.get_root_node(), Some(&"C".to_string()));
    assert_eq!(dynamic_graph.get_root_index(), Some(2));
}

#[test]
fn test_unfreeze_graph_with_nodes_and_edges() {
    let mut dynamic_graph_orig = DynamicGraph::new();
    let n0 = dynamic_graph_orig.add_node("A".to_string());
    let n1 = dynamic_graph_orig.add_node("B".to_string());
    let n2 = dynamic_graph_orig.add_node("C".to_string());
    let n3 = dynamic_graph_orig.add_node("D".to_string());

    dynamic_graph_orig.add_edge(n0, n1, 10).unwrap();
    dynamic_graph_orig.add_edge(n0, n2, 20).unwrap();
    dynamic_graph_orig.add_edge(n1, n3, 30).unwrap();
    dynamic_graph_orig.add_edge(n2, n3, 40).unwrap();
    dynamic_graph_orig.add_edge(n0, n1, 15).unwrap(); // Parallel edge

    let csm_graph = dynamic_graph_orig.freeze();
    let dynamic_graph: DynamicGraph<String, u32> = csm_graph.unfreeze();

    assert_eq!(dynamic_graph.number_nodes(), 4);
    assert_eq!(dynamic_graph.number_edges(), 5);

    assert!(dynamic_graph.contains_edge(n0, n1));
    assert!(dynamic_graph.contains_edge(n0, n2));
    assert!(dynamic_graph.contains_edge(n1, n3));
    assert!(dynamic_graph.contains_edge(n2, n3));

    // Check parallel edge
    let edges_from_n0 = dynamic_graph.get_edges(n0).unwrap();
    assert_eq!(edges_from_n0.len(), 3); // (n0,n1,10), (n0,n2,20), (n0,n1,15)
    let mut targets_from_n0: Vec<usize> = edges_from_n0.iter().map(|(t, _)| *t).collect();
    targets_from_n0.sort_unstable();
    assert_eq!(targets_from_n0, vec![n1, n1, n2]);

    // Check specific edge weights (requires iterating, as `contains_edge` doesn't check weight)
    let has_edge_0_1_10 = dynamic_graph
        .get_edges(n0)
        .unwrap()
        .iter()
        .any(|&(t, w)| t == n1 && w == &10);
    let has_edge_0_1_15 = dynamic_graph
        .get_edges(n0)
        .unwrap()
        .iter()
        .any(|&(t, w)| t == n1 && w == &15);
    assert!(has_edge_0_1_10);
    assert!(has_edge_0_1_15);
}

#[test]
fn test_unfreeze_graph_with_tombstoned_nodes_from_original() {
    let mut dynamic_graph_orig = DynamicGraph::new();
    let n0 = dynamic_graph_orig.add_node("A".to_string());
    let n1 = dynamic_graph_orig.add_node("B".to_string());
    let n2 = dynamic_graph_orig.add_node("C".to_string());
    let n3 = dynamic_graph_orig.add_node("D".to_string());

    dynamic_graph_orig.add_edge(n0, n1, 10).unwrap();
    dynamic_graph_orig.add_edge(n0, n2, 20).unwrap();
    dynamic_graph_orig.add_edge(n2, n3, 30).unwrap();
    dynamic_graph_orig.add_edge(n1, n3, 40).unwrap();

    dynamic_graph_orig.add_root_node("ROOT".to_string());
    let root_idx = dynamic_graph_orig.get_root_index().unwrap();

    dynamic_graph_orig.remove_node(n1).unwrap();
    dynamic_graph_orig
        .update_node(root_idx, "NEW_ROOT".to_string())
        .unwrap();

    let csm_graph = dynamic_graph_orig.freeze();
    let dynamic_graph: DynamicGraph<String, u32> = csm_graph.unfreeze();

    assert_eq!(dynamic_graph.number_nodes(), 4);
    assert_eq!(dynamic_graph.number_edges(), 2);

    // Check remapping:
    // Old: n0 (idx 0), n1 (idx 1, tombstoned), n2 (idx 2), n3 (idx 3), root (idx 4)
    // New: n0 (idx 0), n2 (idx 1), n3 (idx 2), root (idx 3)
    assert!(dynamic_graph.contains_node(0));
    assert!(dynamic_graph.contains_node(1));
    assert!(dynamic_graph.contains_node(2));
    assert!(dynamic_graph.contains_node(3));
    assert_eq!(dynamic_graph.get_node(0), Some(&"A".to_string()));
    assert_eq!(dynamic_graph.get_node(1), Some(&"C".to_string()));
    assert_eq!(dynamic_graph.get_node(2), Some(&"D".to_string()));
    assert_eq!(dynamic_graph.get_node(3), Some(&"NEW_ROOT".to_string()));

    assert!(!dynamic_graph.contains_node(4)); // Old root index is now out of bounds

    // Check edges after remapping
    assert!(!dynamic_graph.contains_edge(0, 0)); // Old (n0, n1) -> (new n0, new n0) - should be gone
    assert!(dynamic_graph.contains_edge(0, 1)); // Old (n0, n2) -> (new n0, new n1)
    assert!(dynamic_graph.contains_edge(1, 2)); // Old (n2, n3) -> (new n1, new n2)
    assert!(!dynamic_graph.contains_edge(0, 2)); // Old (n1, n3) -> (new n0, new n2) - should be gone

    // Check root node remapping
    assert!(dynamic_graph.contains_root_node());
    assert_eq!(dynamic_graph.get_root_node(), Some(&"NEW_ROOT".to_string()));
    assert_eq!(dynamic_graph.get_root_index(), Some(3)); // Old root (idx 4) is now new root (idx 3)
}

#[test]
fn test_unfreeze_single_node_no_edges() {
    let mut dynamic_graph_orig = DynamicGraph::new();
    dynamic_graph_orig.add_node("Single".to_string());

    let csm_graph = dynamic_graph_orig.freeze();
    let dynamic_graph: DynamicGraph<String, u32> = csm_graph.unfreeze();

    assert_eq!(dynamic_graph.number_nodes(), 1);
    assert_eq!(dynamic_graph.number_edges(), 0);
    assert!(dynamic_graph.contains_node(0));
    assert_eq!(dynamic_graph.get_node(0), Some(&"Single".to_string()));
    assert!(!dynamic_graph.contains_root_node());
}

#[test]
fn test_unfreeze_single_node_with_self_loop() {
    let mut dynamic_graph_orig = DynamicGraph::new();
    let n0 = dynamic_graph_orig.add_node("Loop".to_string());
    dynamic_graph_orig.add_edge(n0, n0, 100).unwrap();

    let csm_graph = dynamic_graph_orig.freeze();
    let dynamic_graph: DynamicGraph<String, u32> = csm_graph.unfreeze();

    assert_eq!(dynamic_graph.number_nodes(), 1);
    assert_eq!(dynamic_graph.number_edges(), 1);
    assert!(dynamic_graph.contains_node(0));
    assert_eq!(dynamic_graph.get_node(0), Some(&"Loop".to_string()));
    assert!(dynamic_graph.contains_edge(n0, n0));
    assert_eq!(dynamic_graph.get_edges(n0).unwrap().len(), 1);
    assert_eq!(dynamic_graph.get_edges(n0).unwrap()[0], (n0, &100));
}

#[test]
fn test_unfreeze_graph_with_root_not_last_node() {
    let mut dynamic_graph_orig = DynamicGraph::new();
    dynamic_graph_orig.add_node("A".to_string());
    dynamic_graph_orig.add_root_node("ROOT".to_string()); // Root is at index 1
    dynamic_graph_orig.add_node("B".to_string());

    let csm_graph = dynamic_graph_orig.freeze();
    let dynamic_graph: DynamicGraph<String, u32> = csm_graph.unfreeze();

    assert_eq!(dynamic_graph.number_nodes(), 3);
    assert_eq!(dynamic_graph.get_node(0), Some(&"A".to_string()));
    assert_eq!(dynamic_graph.get_node(1), Some(&"ROOT".to_string()));
    assert_eq!(dynamic_graph.get_node(2), Some(&"B".to_string()));
    assert!(dynamic_graph.contains_root_node());
    assert_eq!(dynamic_graph.get_root_index(), Some(1));
    assert_eq!(dynamic_graph.get_root_node(), Some(&"ROOT".to_string()));
}

#[test]
fn test_unfreeze_graph_with_complex_edges() {
    let mut dynamic_graph_orig = DynamicGraph::new();
    let n0 = dynamic_graph_orig.add_node("Node0".to_string());
    let n1 = dynamic_graph_orig.add_node("Node1".to_string());
    let n2 = dynamic_graph_orig.add_node("Node2".to_string());
    let n3 = dynamic_graph_orig.add_node("Node3".to_string());

    dynamic_graph_orig.add_edge(n0, n1, 1).unwrap();
    dynamic_graph_orig.add_edge(n0, n2, 2).unwrap();
    dynamic_graph_orig.add_edge(n1, n2, 3).unwrap();
    dynamic_graph_orig.add_edge(n2, n3, 4).unwrap();
    dynamic_graph_orig.add_edge(n0, n1, 5).unwrap(); // Parallel edge

    let csm_graph = dynamic_graph_orig.freeze();
    let dynamic_graph: DynamicGraph<String, u32> = csm_graph.unfreeze();

    assert_eq!(dynamic_graph.number_nodes(), 4);
    assert_eq!(dynamic_graph.number_edges(), 5);

    // Verify edges from n0
    let edges_from_n0 = dynamic_graph.get_edges(n0).unwrap();
    assert_eq!(edges_from_n0.len(), 3);
    let mut expected_n0_edges = vec![(n1, &1), (n2, &2), (n1, &5)];
    expected_n0_edges.sort_unstable();
    let mut actual_n0_edges = edges_from_n0
        .iter()
        .map(|&(t, w)| (t, w))
        .collect::<Vec<_>>();
    actual_n0_edges.sort_unstable();
    assert_eq!(actual_n0_edges, expected_n0_edges);

    // Verify edges from n1
    let edges_from_n1 = dynamic_graph.get_edges(n1).unwrap();
    assert_eq!(edges_from_n1.len(), 1);
    assert_eq!(edges_from_n1[0], (n2, &3));

    // Verify edges from n2
    let edges_from_n2 = dynamic_graph.get_edges(n2).unwrap();
    assert_eq!(edges_from_n2.len(), 1);
    assert_eq!(edges_from_n2[0], (n3, &4));

    // Verify edges from n3 (no outgoing edges)
    assert!(dynamic_graph.get_edges(n3).unwrap().is_empty());
}
