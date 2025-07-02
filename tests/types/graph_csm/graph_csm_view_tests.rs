use next_graph::{CsmGraph, DynamicGraph, Freezable, GraphView};

#[cfg(test)]
mod csm_graph_view_tests {
    use next_graph::GraphMut;
    use super::*;

    // Helper function to create a CsmGraph from a DynamicGraph
    fn create_csm_graph() -> CsmGraph<String, u32> {
        let mut dynamic_graph = DynamicGraph::new();
        let n0 = dynamic_graph.add_node("A".to_string());
        let n1 = dynamic_graph.add_node("B".to_string());
        let n2 = dynamic_graph.add_node("C".to_string());
        dynamic_graph.add_edge(n0, n1, 10).unwrap();
        dynamic_graph.add_edge(n0, n2, 20).unwrap();
        dynamic_graph.add_root_node("ROOT".to_string());
        dynamic_graph.freeze()
    }

    #[test]
    fn test_is_frozen() {
        let graph = create_csm_graph();
        assert!(graph.is_frozen());
    }

    #[test]
    fn test_contains_node() {
        let graph = create_csm_graph();
        assert!(graph.contains_node(0));
        assert!(graph.contains_node(1));
        assert!(graph.contains_node(2));
        assert!(graph.contains_node(3)); // Root node
        assert!(!graph.contains_node(99));
    }

    #[test]
    fn test_get_node() {
        let graph = create_csm_graph();
        assert_eq!(graph.get_node(0), Some(&"A".to_string()));
        assert_eq!(graph.get_node(3), Some(&"ROOT".to_string()));
        assert_eq!(graph.get_node(99), None);
    }

    #[test]
    fn test_number_nodes() {
        let graph = create_csm_graph();
        assert_eq!(graph.number_nodes(), 4);
    }

    #[test]
    fn test_contains_edge() {
        let graph = create_csm_graph();
        assert!(graph.contains_edge(0, 1));
        assert!(graph.contains_edge(0, 2));
        assert!(!graph.contains_edge(1, 0)); // Directed
        assert!(!graph.contains_edge(0, 99));
        assert!(!graph.contains_edge(99, 0));
    }

    #[test]
    fn test_number_edges() {
        let graph = create_csm_graph();
        assert_eq!(graph.number_edges(), 2);
    }

    #[test]
    fn test_contains_root_node() {
        let graph = create_csm_graph();
        assert!(graph.contains_root_node());

        let empty_graph = CsmGraph::<String, u32>::new();
        assert!(!empty_graph.contains_root_node());
    }

    #[test]
    fn test_get_root_node() {
        let graph = create_csm_graph();
        assert_eq!(graph.get_root_node(), Some(&"ROOT".to_string()));

        let empty_graph = CsmGraph::<String, u32>::new();
        assert_eq!(empty_graph.get_root_node(), None);
    }

    #[test]
    fn test_get_root_index() {
        let graph = create_csm_graph();
        assert_eq!(graph.get_root_index(), Some(3));

        let empty_graph = CsmGraph::<String, u32>::new();
        assert_eq!(empty_graph.get_root_index(), None);
    }

    #[test]
    fn test_get_edges() {
        let graph = create_csm_graph();

        let edges_n0 = graph.get_edges(0).unwrap();
        assert_eq!(edges_n0.len(), 2);
        assert!(edges_n0.iter().any(|&(t, w)| t == 1 && w == &10));
        assert!(edges_n0.iter().any(|&(t, w)| t == 2 && w == &20));

        let edges_n1 = graph.get_edges(1).unwrap();
        assert!(edges_n1.is_empty());

        assert_eq!(graph.get_edges(99), None);
    }
}
