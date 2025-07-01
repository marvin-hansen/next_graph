use crate::{GraphError, GraphView};

pub trait GraphMut<N, W>: GraphView<N, W> {
    // Node Mutation
    fn add_node(&mut self, node: N) -> usize;
    fn update_node(&mut self, index: usize, node: N) -> Result<(), GraphError>;

    // Edge Mutation
    fn add_edge(&mut self, a: usize, b: usize, weight: W) -> Result<(), GraphError>;
    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), GraphError>;

    // Root Node Mutation
    fn add_root_node(&mut self, node: N) -> usize;

    // Graph-wide Mutation
    fn clear(&mut self);
}
