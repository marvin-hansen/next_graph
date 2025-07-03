use next_graph::utils_test::gen_utils::create_csm_graph;
use next_graph::{DynamicGraph, Freezable, GraphAlgorithms, GraphMut};

#[test]
fn test_find_cycle_no_cycle() {
    let graph = create_csm_graph();
    assert_eq!(graph.find_cycle(), None);
}

#[test]
fn test_find_cycle_with_cycle() {
    let mut dynamic_graph = DynamicGraph::new();
    let n0 = dynamic_graph.add_node("A".to_string());
    let n1 = dynamic_graph.add_node("B".to_string());
    let n2 = dynamic_graph.add_node("C".to_string());

    dynamic_graph.add_edge(n0, n1, 10).unwrap();
    dynamic_graph.add_edge(n1, n2, 20).unwrap();
    dynamic_graph.add_edge(n2, n0, 30).unwrap(); // Cycle: 0 -> 1 -> 2 -> 0

    let graph = dynamic_graph.freeze();
    let cycle = graph.find_cycle();
    assert!(cycle.is_some());
    let path = cycle.unwrap();
    assert_eq!(path.len(), 4);
    assert!(path.contains(&0));
    assert!(path.contains(&1));
    assert!(path.contains(&2));
}

#[test]
fn test_has_cycle() {
    let graph_no_cycle = create_csm_graph();
    assert!(!graph_no_cycle.has_cycle());

    let mut dynamic_graph = DynamicGraph::new();
    let n0 = dynamic_graph.add_node("A".to_string());
    let n1 = dynamic_graph.add_node("B".to_string());
    dynamic_graph.add_edge(n0, n1, 10).unwrap();
    dynamic_graph.add_edge(n1, n0, 20).unwrap(); // Cycle
    let graph_with_cycle = dynamic_graph.freeze();
    assert!(graph_with_cycle.has_cycle());
}

#[test]
fn test_topological_sort_no_cycle() {
    let graph = create_csm_graph();
    let sort = graph.topological_sort();
    assert!(sort.is_some());
    let sorted_nodes = sort.unwrap();
    assert_eq!(sorted_nodes.len(), 5);
    // A -> B, A -> C, B -> D, C -> D, D -> E
    // Expected order: A, B, C, D, E (or A, C, B, D, E)
    // Verify that dependencies are met
    let pos_a = sorted_nodes.iter().position(|&n| n == 0).unwrap();
    let pos_b = sorted_nodes.iter().position(|&n| n == 1).unwrap();
    let pos_c = sorted_nodes.iter().position(|&n| n == 2).unwrap();
    let pos_d = sorted_nodes.iter().position(|&n| n == 3).unwrap();
    let pos_e = sorted_nodes.iter().position(|&n| n == 4).unwrap();

    assert!(pos_a < pos_b);
    assert!(pos_a < pos_c);
    assert!(pos_b < pos_d);
    assert!(pos_c < pos_d);
    assert!(pos_d < pos_e);
}

#[test]
fn test_topological_sort_with_cycle() {
    let mut dynamic_graph = DynamicGraph::new();
    let n0 = dynamic_graph.add_node("A".to_string());
    let n1 = dynamic_graph.add_node("B".to_string());
    dynamic_graph.add_edge(n0, n1, 10).unwrap();
    dynamic_graph.add_edge(n1, n0, 20).unwrap(); // Cycle
    let graph = dynamic_graph.freeze();
    assert_eq!(graph.topological_sort(), None);
}

#[test]
fn test_is_reachable() {
    let graph = create_csm_graph();

    assert!(graph.is_reachable(0, 4)); // A -> E
    assert!(graph.is_reachable(0, 3)); // A -> D
    assert!(graph.is_reachable(1, 4)); // B -> E
    assert!(!graph.is_reachable(4, 0)); // E -> A (no path)
    assert!(!graph.is_reachable(0, 99)); // Non-existent target
    assert!(!graph.is_reachable(99, 0)); // Non-existent source
}

#[test]
fn test_shortest_path_len() {
    let graph = create_csm_graph();

    assert_eq!(graph.shortest_path_len(0, 0), Some(1)); // Self loop
    assert_eq!(graph.shortest_path_len(0, 1), Some(2)); // A -> B
    assert_eq!(graph.shortest_path_len(0, 3), Some(3)); // A -> B -> D or A -> C -> D
    assert_eq!(graph.shortest_path_len(0, 4), Some(4)); // A -> B -> D -> E or A -> C -> D -> E
    assert_eq!(graph.shortest_path_len(4, 0), None); // No path
    assert_eq!(graph.shortest_path_len(0, 99), None); // Non-existent target
    assert_eq!(graph.shortest_path_len(99, 0), None); // Non-existent source
}

#[test]
fn test_shortest_path() {
    let graph = create_csm_graph();

    assert_eq!(graph.shortest_path(0, 0), Some(vec![0])); // Self loop
    assert_eq!(graph.shortest_path(0, 1), Some(vec![0, 1])); // A -> B
    assert_eq!(graph.shortest_path(0, 4), Some(vec![0, 1, 3, 4])); // A -> B -> D -> E (one possible shortest path)
    assert_eq!(graph.shortest_path(4, 0), None); // No path
    assert_eq!(graph.shortest_path(0, 99), None); // Non-existent target
    assert_eq!(graph.shortest_path(99, 0), None); // Non-existent source
}

#[test]
fn test_find_cycle_disconnected_graph() {
    let mut dynamic_graph = DynamicGraph::new();
    dynamic_graph.add_node("A".to_string());
    dynamic_graph.add_node("B".to_string());
    dynamic_graph.add_node("C".to_string());
    dynamic_graph.add_edge(0, 1, 1).unwrap();
    dynamic_graph.add_edge(1, 0, 1).unwrap(); // Cycle in one component
    dynamic_graph.add_node("D".to_string()); // Disconnected node

    let graph = dynamic_graph.freeze();
    assert!(graph.find_cycle().is_some());
}

#[test]
fn test_topological_sort_disconnected_graph() {
    let mut dynamic_graph = DynamicGraph::new();
    dynamic_graph.add_node("A".to_string());
    dynamic_graph.add_node("B".to_string());
    dynamic_graph.add_edge(0, 1, 1).unwrap();
    dynamic_graph.add_node("C".to_string()); // Disconnected node

    let graph = dynamic_graph.freeze();
    let sort = graph.topological_sort();
    assert!(sort.is_some());
    let sorted_nodes = sort.unwrap();
    assert_eq!(sorted_nodes.len(), 3);
    // The exact order of disconnected components can vary, but dependencies must be met.
    let pos_a = sorted_nodes.iter().position(|&n| n == 0).unwrap();
    let pos_b = sorted_nodes.iter().position(|&n| n == 1).unwrap();
    assert!(pos_a < pos_b);
}

#[test]
fn test_shortest_path_len_no_path() {
    let mut dynamic_graph = DynamicGraph::new();
    dynamic_graph.add_node("A".to_string());
    dynamic_graph.add_node("B".to_string());
    dynamic_graph.add_node("C".to_string());
    dynamic_graph.add_edge(0, 1, 1).unwrap();
    // No path from 0 to 2
    let graph = dynamic_graph.freeze();
    assert_eq!(graph.shortest_path_len(0, 2), None);
}

#[test]
fn test_shortest_path_no_path() {
    let mut dynamic_graph = DynamicGraph::new();
    dynamic_graph.add_node("A".to_string());
    dynamic_graph.add_node("B".to_string());
    dynamic_graph.add_node("C".to_string());
    dynamic_graph.add_edge(0, 1, 1).unwrap();
    // No path from 0 to 2
    let graph = dynamic_graph.freeze();
    assert_eq!(graph.shortest_path(0, 2), None);
}

#[test]
fn test_find_cycle_empty_graph() {
    let dynamic_graph = DynamicGraph::<String, u32>::new();
    let csm_graph = dynamic_graph.freeze();
    assert_eq!(csm_graph.find_cycle(), None);
}

#[test]
fn test_find_cycle_single_node_no_edges() {
    let mut dynamic_graph = DynamicGraph::<String, u32>::new();
    dynamic_graph.add_node("A".to_string());
    let csm_graph = dynamic_graph.freeze();
    assert_eq!(csm_graph.find_cycle(), None);
}

#[test]
fn test_topological_sort_empty_graph() {
    let dynamic_graph = DynamicGraph::<String, u32>::new();
    let csm_graph = dynamic_graph.freeze();
    assert_eq!(csm_graph.topological_sort(), Some(vec![]));
}

#[test]
fn test_topological_sort_single_node_no_edges() {
    let mut dynamic_graph = DynamicGraph::<String, u32>::new();
    dynamic_graph.add_node("A".to_string());
    let csm_graph = dynamic_graph.freeze();
    assert_eq!(csm_graph.topological_sort(), Some(vec![0]));
}

#[test]
fn test_find_cycle_self_loop() {
    let mut dynamic_graph = DynamicGraph::new();
    let n0 = dynamic_graph.add_node("A".to_string());
    dynamic_graph.add_edge(n0, n0, 10).unwrap(); // Self-loop
    let graph = dynamic_graph.freeze();
    let cycle = graph.find_cycle();
    assert!(cycle.is_some());
    assert_eq!(cycle.unwrap(), vec![0, 0]);
}

#[test]
fn test_find_cycle_multiple_cycles() {
    let mut dynamic_graph = DynamicGraph::new();
    let n0 = dynamic_graph.add_node("A".to_string());
    let n1 = dynamic_graph.add_node("B".to_string());
    let n2 = dynamic_graph.add_node("C".to_string());
    let n3 = dynamic_graph.add_node("D".to_string());

    // Cycle 1: 0 -> 1 -> 0
    dynamic_graph.add_edge(n0, n1, 1).unwrap();
    dynamic_graph.add_edge(n1, n0, 1).unwrap();

    // Cycle 2: 2 -> 3 -> 2
    dynamic_graph.add_edge(n2, n3, 1).unwrap();
    dynamic_graph.add_edge(n3, n2, 1).unwrap();

    let graph = dynamic_graph.freeze();
    let cycle = graph.find_cycle();
    assert!(cycle.is_some());
    // The exact cycle found depends on DFS traversal order, but it should be one of them.
    let path = cycle.unwrap();
    assert_eq!(path.len(), 3);
    assert_eq!(path.first(), path.last(), "Path should be a closed loop");

    // The exact cycle found depends on DFS traversal order
    // , but it should be one of the valid cycle paths.
    assert!(
        path == vec![0, 1, 0]
            || path == vec![1, 0, 1]
            || path == vec![2, 3, 2]
            || path == vec![3, 2, 3]
    );
}

#[test]
fn test_topological_sort_complex_disconnected() {
    let mut dynamic_graph = DynamicGraph::new();
    // Component 1: 0 -> 1 -> 2
    let n0 = dynamic_graph.add_node("A".to_string());
    let n1 = dynamic_graph.add_node("B".to_string());
    let n2 = dynamic_graph.add_node("C".to_string());
    dynamic_graph.add_edge(n0, n1, 1).unwrap();
    dynamic_graph.add_edge(n1, n2, 1).unwrap();

    // Component 2: 3 -> 4
    let n3 = dynamic_graph.add_node("D".to_string());
    let n4 = dynamic_graph.add_node("E".to_string());
    dynamic_graph.add_edge(n3, n4, 1).unwrap();

    // Component 3: 5 (isolated)
    dynamic_graph.add_node("F".to_string());

    let graph = dynamic_graph.freeze();
    let sort = graph.topological_sort();
    assert!(sort.is_some());
    let sorted_nodes = sort.unwrap();
    assert_eq!(sorted_nodes.len(), 6);

    // Verify dependencies within components
    let pos_0 = sorted_nodes.iter().position(|&n| n == n0).unwrap();
    let pos_1 = sorted_nodes.iter().position(|&n| n == n1).unwrap();
    let pos_2 = sorted_nodes.iter().position(|&n| n == n2).unwrap();
    assert!(pos_0 < pos_1);
    assert!(pos_1 < pos_2);

    let pos_3 = sorted_nodes.iter().position(|&n| n == n3).unwrap();
    let pos_4 = sorted_nodes.iter().position(|&n| n == n4).unwrap();
    assert!(pos_3 < pos_4);

    // Isolated node 5 can be anywhere relative to others
}

#[test]
fn test_shortest_path_len_complex() {
    let mut dynamic_graph = DynamicGraph::new();
    let n0 = dynamic_graph.add_node("A".to_string());
    let n1 = dynamic_graph.add_node("B".to_string());
    let n2 = dynamic_graph.add_node("C".to_string());
    let n3 = dynamic_graph.add_node("D".to_string());
    let n4 = dynamic_graph.add_node("E".to_string());

    dynamic_graph.add_edge(n0, n1, 1).unwrap();
    dynamic_graph.add_edge(n0, n2, 1).unwrap();
    dynamic_graph.add_edge(n1, n3, 1).unwrap();
    dynamic_graph.add_edge(n2, n3, 1).unwrap();
    dynamic_graph.add_edge(n3, n4, 1).unwrap();

    let graph = dynamic_graph.freeze();

    assert_eq!(graph.shortest_path_len(n0, n4), Some(4)); // 0->1->3->4 or 0->2->3->4
    assert_eq!(graph.shortest_path_len(n0, n3), Some(3));
}

#[test]
fn test_shortest_path_complex() {
    let mut dynamic_graph = DynamicGraph::new();
    let n0 = dynamic_graph.add_node("A".to_string());
    let n1 = dynamic_graph.add_node("B".to_string());
    let n2 = dynamic_graph.add_node("C".to_string());
    let n3 = dynamic_graph.add_node("D".to_string());
    let n4 = dynamic_graph.add_node("E".to_string());

    dynamic_graph.add_edge(n0, n1, 1).unwrap();
    dynamic_graph.add_edge(n0, n2, 1).unwrap();
    dynamic_graph.add_edge(n1, n3, 1).unwrap();
    dynamic_graph.add_edge(n2, n3, 1).unwrap();
    dynamic_graph.add_edge(n3, n4, 1).unwrap();

    let graph = dynamic_graph.freeze();

    let path = graph.shortest_path(n0, n4).unwrap();
    assert_eq!(path.len(), 4);
    assert_eq!(path[0], n0);
    assert_eq!(path[3], n4);
    // Path could be 0->1->3->4 or 0->2->3->4. Check that it's a valid path.
    assert!((path[1] == n1 && path[2] == n3) || (path[1] == n2 && path[2] == n3));
}
