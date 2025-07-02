use crate::{CsmGraph, DynamicGraph, Freezable, GraphMut};

// Helper function to create a CsmGraph from a DynamicGraph
pub fn create_csm_graph() -> CsmGraph<String, u32> {
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
