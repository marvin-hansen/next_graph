use next_graph::{CsmGraph, DynamicGraph, Freezable, GraphView};

#[cfg(test)]
mod csm_graph_view_tests {
    use super::*;
    use next_graph::GraphMut;

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
        assert!(!graph.contains_edge(11, 12)); // Directed
        assert!(!graph.contains_edge(0, 99));
        assert!(!graph.contains_edge(99, 0));
    }

    #[test]
    fn test_contains_edge_with_binary_search() {
        // SETUP: Create a graph that will trigger the binary search path.
        // The threshold is 64, so we need a node with more than 64 outgoing edges.
        // We will create a source node (index 0) with 100 outgoing edges.

        // This test assumes a mutable CsrGraph that can be frozen into a CsmGraph.
        // Replace `CsrGraph` with your actual mutable graph builder if it's different.
        let mut builder = DynamicGraph::new();

        // Add the source node and 200 other nodes to serve as potential targets.
        for _ in 0..=200 {
            builder.add_node(()); // Node payload is irrelevant for this test.
        }

        let source_node = 0;
        // Add 100 edges from the source node. To make the test robust, we add them
        // to non-contiguous targets (e.g., all odd-numbered nodes from 1 to 199).
        // The freeze() operation is expected to sort these, which is required for binary search.
        for i in 0..100 {
            let target_node = (i * 2) + 1; // Creates edges to 1, 3, 5, ..., 199
            builder
                .add_edge(source_node, target_node, ())
                .expect("Failed to add edge"); // Edge weight is irrelevant.
        }

        // Freeze the graph to get the high-performance, immutable CsmGraph.
        let graph = builder.freeze();

        // EXECUTE & ASSERT
        // Now that the source node has 100 neighbors, calls to `contains_edge`
        // for that node will use the binary search implementation.

        // 1. Test for an edge that exists in the middle of the sorted list.
        let existing_target = 101; // 0 -> 101 should exist.
        assert!(
            graph.contains_edge(source_node, existing_target),
            "Binary search should find an existing edge."
        );

        // 2. Test for an edge that does NOT exist but is within the range of neighbors.
        let non_existing_target = 100; // 0 -> 100 should NOT exist.
        assert!(
            !graph.contains_edge(source_node, non_existing_target),
            "Binary search should NOT find a non-existing edge."
        );

        // 3. Test for the very first edge in the sorted neighbor list.
        let first_target = 1;
        assert!(
            graph.contains_edge(source_node, first_target),
            "Binary search should find the first edge in the list."
        );

        // 4. Test for the very last edge in the sorted neighbor list.
        let last_target = 199;
        assert!(
            graph.contains_edge(source_node, last_target),
            "Binary search should find the last edge in the list."
        );
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

    #[test]
    fn test_get_edges_err() {
        let dynamic_graph = DynamicGraph::<String, u32>::new();
        let graph: CsmGraph<String, u32> = dynamic_graph.freeze();

        let res = graph.get_edges(0);
        assert!(res.is_none());

        assert_eq!(graph.get_edges(99), None);
    }
}
