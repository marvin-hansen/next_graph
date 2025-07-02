use next_graph::{DynamicGraph, GraphMut, GraphView};

#[test]
fn test_contains_node() {
    let mut graph = DynamicGraph::<String, u32>::new();
    let node_idx = graph.add_node("A".to_string());
    assert!(graph.contains_node(node_idx));
    assert!(!graph.contains_node(99)); // Non-existent index

    graph.remove_node(node_idx).unwrap();
    assert!(!graph.contains_node(node_idx)); // Removed node
}

#[test]
fn test_get_node() {
    let mut graph = DynamicGraph::<String, u32>::new();
    let node_idx = graph.add_node("A".to_string());
    assert_eq!(graph.get_node(node_idx), Some(&"A".to_string()));
    assert_eq!(graph.get_node(99), None); // Non-existent index

    graph.remove_node(node_idx).unwrap();
    assert_eq!(graph.get_node(node_idx), None); // Removed node
}

#[test]
fn test_number_nodes() {
    let mut graph = DynamicGraph::<String, u32>::new();
    assert_eq!(graph.number_nodes(), 0);
    graph.add_node("A".to_string());
    assert_eq!(graph.number_nodes(), 1);
    let n1 = graph.add_node("B".to_string());
    assert_eq!(graph.number_nodes(), 2);
    graph.remove_node(n1).unwrap();
    assert_eq!(graph.number_nodes(), 1); // Removed node should not be counted
}

#[test]
fn test_contains_edge() {
    let mut graph = DynamicGraph::<String, u32>::new();
    let node_a = graph.add_node("A".to_string());
    let node_b = graph.add_node("B".to_string());
    let node_c = graph.add_node("C".to_string());

    graph.add_edge(node_a, node_b, 10).unwrap();

    assert!(graph.contains_edge(node_a, node_b));
    assert!(!graph.contains_edge(node_a, node_c)); // Edge not present
    assert!(!graph.contains_edge(node_b, node_a)); // Directed edge
    assert!(!graph.contains_edge(99, node_b)); // Non-existent source
    assert!(!graph.contains_edge(node_a, 99)); // Non-existent target

    graph.remove_node(node_b).unwrap();
    assert!(!graph.contains_edge(node_a, node_b)); // Edge to removed node
}

#[test]
fn test_number_edges() {
    let mut graph = DynamicGraph::<String, u32>::new();
    assert_eq!(graph.number_edges(), 0);
    let node_a = graph.add_node("A".to_string());
    let node_b = graph.add_node("B".to_string());
    let node_c = graph.add_node("C".to_string());

    graph.add_edge(node_a, node_b, 10).unwrap();
    assert_eq!(graph.number_edges(), 1);
    graph.add_edge(node_a, node_c, 20).unwrap();
    assert_eq!(graph.number_edges(), 2);
    graph.add_edge(node_a, node_b, 15).unwrap(); // Parallel edge
    assert_eq!(graph.number_edges(), 3);

    graph.remove_node(node_b).unwrap();
    // number_edges is O(V) and counts all edges, even to/from tombstoned nodes
    // until freeze. So it should still be 3.
    assert_eq!(graph.number_edges(), 3);
}

#[test]
fn test_contains_root_node() {
    let mut graph = DynamicGraph::<String, u32>::new();
    assert!(!graph.contains_root_node());

    let root_idx = graph.add_root_node("ROOT".to_string());
    assert!(graph.contains_root_node());

    graph.remove_node(root_idx).unwrap();
    assert!(!graph.contains_root_node());
}

#[test]
fn test_get_root_node() {
    let mut graph = DynamicGraph::<String, u32>::new();
    assert_eq!(graph.get_root_node(), None);

    let root_idx = graph.add_root_node("ROOT".to_string());
    assert_eq!(graph.get_root_node(), Some(&"ROOT".to_string()));

    graph.remove_node(root_idx).unwrap();
    assert_eq!(graph.get_root_node(), None);
}

#[test]
fn test_get_root_index() {
    let mut graph = DynamicGraph::<String, u32>::new();
    assert_eq!(graph.get_root_index(), None);

    let root_idx = graph.add_root_node("ROOT".to_string());
    assert_eq!(graph.get_root_index(), Some(root_idx));

    graph.remove_node(root_idx).unwrap();
    assert_eq!(graph.get_root_index(), None);
}
