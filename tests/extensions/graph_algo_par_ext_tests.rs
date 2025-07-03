#![cfg(feature = "parallel")]
// This entire module is only compiled and run when the "parallel" feature is enabled.
// i.e. use cargo t --features parallel

use next_graph::{DynamicGraph, Freezable, GraphMut, ParallelGraphAlgorithmsExt};

// --- Test Helper Functions ---

/// Creates a standard Directed Acyclic Graph (DAG) for testing.
///
/// Structure:
/// 0 -> 1 -> 3
/// |         ^
/// v         |
/// 2 --------+
fn create_test_dag() -> DynamicGraph<(), ()> {
    let mut dag = DynamicGraph::new();
    let n0 = dag.add_node(()); // 0
    let n1 = dag.add_node(()); // 1
    let n2 = dag.add_node(()); // 2
    let n3 = dag.add_node(()); // 3
    dag.add_edge(n0, n1, ()).unwrap();
    dag.add_edge(n0, n2, ()).unwrap();
    dag.add_edge(n1, n3, ()).unwrap();
    dag.add_edge(n2, n3, ()).unwrap();
    dag
}

/// Creates a graph with a cycle for testing.
/// It's the test DAG with a back-edge from 3 to 0.
fn create_cyclic_graph() -> DynamicGraph<(), ()> {
    let mut graph = create_test_dag();
    graph.add_edge(3, 0, ()).unwrap(); // Cycle: 3 -> 0
    graph
}

/// Creates a disconnected graph for testing.
/// Component 1: 0 -> 1. Component 2: 2 (isolated).
fn create_disconnected_graph() -> DynamicGraph<(), ()> {
    let mut graph = DynamicGraph::new();
    let n0 = graph.add_node(());
    let n1 = graph.add_node(());
    graph.add_node(()); // Node 2, isolated
    graph.add_edge(n0, n1, ()).unwrap();
    graph
}

// --- topological_sort_par Tests ---

#[test]
fn test_topo_par_on_dag() {
    let graph = create_test_dag().freeze();
    let sorted = graph.topological_sort_par();
    assert!(sorted.is_some(), "Topological sort should exist for a DAG");
    let sorted_nodes = sorted.unwrap();
    assert_eq!(sorted_nodes.len(), 4);

    // The implementation is deterministic due to sorting frontiers.
    // Level 0: {0}
    // Level 1: {1, 2} -> sorted to [1, 2]
    // Level 2: {3}
    assert_eq!(sorted_nodes, vec![0, 1, 2, 3]);
}

#[test]
fn test_topo_par_on_cyclic_graph() {
    let graph = create_cyclic_graph().freeze();
    assert!(
        graph.topological_sort_par().is_none(),
        "Topological sort should not exist for a cyclic graph"
    );
}

#[test]
fn test_topo_par_on_disconnected_graph() {
    let graph = create_disconnected_graph().freeze();
    let sorted = graph.topological_sort_par();
    assert!(sorted.is_some());
    let sorted_nodes = sorted.unwrap();
    assert_eq!(sorted_nodes.len(), 3);
    // The implementation is deterministic.
    // Level 0: {0, 2} -> sorted to [0, 2]
    // Level 1: {1}
    assert_eq!(sorted_nodes, vec![0, 2, 1]);
}

#[test]
fn test_topo_par_on_empty_graph() {
    let graph = DynamicGraph::<(), ()>::new().freeze();
    assert_eq!(
        graph.topological_sort_par(),
        Some(vec![]),
        "Topological sort of an empty graph is an empty vec"
    );
}

#[test]
fn test_topo_par_on_single_node_graph() {
    let mut graph = DynamicGraph::<(), ()>::new();
    graph.add_node(());
    let frozen = graph.freeze();
    assert_eq!(
        frozen.topological_sort_par(),
        Some(vec![0]),
        "Topological sort of a single node is the node itself"
    );
}

// --- Pathfinding and Reachability Tests ---

#[test]
fn test_is_reachable_par() {
    let graph = create_test_dag().freeze();
    assert!(graph.is_reachable_par(0, 3));
    assert!(graph.is_reachable_par(0, 1));
    assert!(!graph.is_reachable_par(3, 0));
    assert!(!graph.is_reachable_par(1, 2));
    // To self
    assert!(graph.is_reachable_par(1, 1));
    // Invalid nodes
    assert!(!graph.is_reachable_par(0, 99));
    assert!(!graph.is_reachable_par(99, 0));
}

#[test]
fn test_shortest_path_len_par() {
    let graph = create_test_dag().freeze();
    assert_eq!(graph.shortest_path_len_par(0, 3), Some(3)); // 0->1->3 or 0->2->3
    assert_eq!(graph.shortest_path_len_par(0, 1), Some(2));
    assert_eq!(graph.shortest_path_len_par(3, 0), None);
    // To self
    assert_eq!(graph.shortest_path_len_par(2, 2), Some(1));
    // Invalid nodes
    assert_eq!(graph.shortest_path_len_par(0, 99), None);
    assert_eq!(graph.shortest_path_len_par(99, 0), None);
}

#[test]
fn test_shortest_path_par() {
    let graph = create_test_dag().freeze();

    // Test a valid path
    let path = graph.shortest_path_par(0, 3);
    assert!(path.is_some());
    let p = path.unwrap();
    // Either path is valid and has length 3.
    assert_eq!(p.len(), 3);
    assert!(p == vec![0, 1, 3] || p == vec![0, 2, 3]);

    // Test a non-existent path
    assert_eq!(graph.shortest_path_par(3, 0), None);

    // Test path to self
    assert_eq!(graph.shortest_path_par(1, 1), Some(vec![1]));

    // Test invalid nodes
    assert_eq!(graph.shortest_path_par(0, 99), None);
    assert_eq!(graph.shortest_path_par(99, 0), None);
}

#[test]
fn test_pathfinding_on_disconnected_graph() {
    let graph = create_disconnected_graph().freeze();
    // Path within a component
    assert_eq!(graph.shortest_path_par(0, 1), Some(vec![0, 1]));
    // Path to a disconnected node
    assert_eq!(graph.shortest_path_par(0, 2), None);
    assert!(!graph.is_reachable_par(1, 2));
}

#[test]
fn test_pathfinding_on_empty_graph() {
    let graph = DynamicGraph::<(), ()>::new().freeze();
    assert_eq!(graph.shortest_path_par(0, 0), None);
    assert_eq!(graph.shortest_path_len_par(0, 0), None);
    assert!(!graph.is_reachable_par(0, 0));
}
