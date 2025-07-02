use next_graph::utils_test::gen_utils::create_csm_graph;
use next_graph::{GraphError, GraphTraversal};

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
    assert!(matches!(
        graph.outbound_edges(99),
        Err(GraphError::NodeNotFound(99))
    ));
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
    assert!(matches!(
        graph.inbound_edges(99),
        Err(GraphError::NodeNotFound(99))
    ));
}
