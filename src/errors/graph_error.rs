use std::fmt;

/// A lightweight, copyable, stack-allocated error type for the next_graph library
/// that provides context about the failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphError {
    /// An operation was attempted on a node index that does not exist or has been removed.
    NodeNotFound(usize),

    /// An edge could not be created, typically between two nodes.
    EdgeCreationError { source: usize, target: usize },

    /// An operation was attempted on an edge that does not exist.
    EdgeNotFoundError { source: usize, target: usize },

    /// The operation could not be completed because the graph contains a cycle.
    GraphContainsCycle,
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NodeNotFound(index) => {
                write!(
                    f,
                    "Node with index {} not found; it may be out of bounds or have been removed.",
                    index
                )
            }
            Self::EdgeCreationError { source, target } => {
                write!(
                    f,
                    "Edge from {} to {} could not be created; a node may not exist or the edge already exists.",
                    source, target
                )
            }
            Self::EdgeNotFoundError { source, target } => {
                write!(f, "Edge from {} to {} not found.", source, target)
            }
            Self::GraphContainsCycle => {
                write!(f, "Operation failed because the graph contains a cycle.")
            }
        }
    }
}

// This makes GraphError a fully-fledged error type compatible with the Rust ecosystem.
impl std::error::Error for GraphError {}
