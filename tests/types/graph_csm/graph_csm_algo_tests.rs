use next_graph::{CsmGraph, DynamicGraph, Freezable, GraphAlgorithms, GraphError};

#[cfg(test)]
mod csm_graph_algo_tests {
    use super::*;

    // Helper function to create a CsmGraph from a DynamicGraph
    fn create_csm_graph() -> CsmGraph<String, u32> {
        let mut dynamic_graph = DynamicGraph::new();
        let n0 = dynamic_graph.add_node("A".to_string());
        let n1 = dynamic_graph.add_node("B".to_string());
        let n2 = dynamic_graph.add_node("C".to_string());
        let n3 = dynamic_graph.add_node("D".to_string());
        let n4 = dynamic_graph.add_node("E".to_string());

        dynamic_graph.add_edge(n0, n1, 10).unwrap();
        dynamic_graph.add_edge(n0, n2, 20).unwrap();
        dynamic_graph.add_edge(n1, n3, 30).unwrap();
        dynamic_graph.add_edge(n2, n3, 40).unwrap();
        dynamic_graph.add_edge(n3, n4, 50).unwrap();

        dynamic_graph.freeze()
    }

    #[test]
    fn test_outbound_edges() {
        let graph = create_csm_graph();

        // Test existing node with outbound edges
        let edges_n0: Vec<usize> = graph.outbound_edges(0).unwrap().collect();
        assert_eq!(edges_n0.len(), 2);
        assert!(edges_n0.contains(&1));
        assert!(edges_n0.contains(&2));

        // Test existing node with no outbound edges
        let edges_n4: Vec<usize> = graph.outbound_edges(4).unwrap().collect();
        assert!(edges_n4.is_empty());

        // Test non-existent node
        assert!(matches!(graph.outbound_edges(99), Err(GraphError::NodeNotFound(99))));
    }

    #[test]
    fn test_inbound_edges() {
        let graph = create_csm_graph();

        // Test existing node with inbound edges
        let edges_n3: Vec<usize> = graph.inbound_edges(3).unwrap().collect();
        assert_eq!(edges_n3.len(), 2);
        assert!(edges_n3.contains(&1));
        assert!(edges_n3.contains(&2));

        // Test existing node with no inbound edges
        let edges_n0: Vec<usize> = graph.inbound_edges(0).unwrap().collect();
        assert!(edges_n0.is_empty());

        // Test non-existent node
        assert!(matches!(graph.inbound_edges(99), Err(GraphError::NodeNotFound(99))));
    }

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
        assert_eq!(path.len(), 3);
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
}