mod errors;
mod extensions;
mod traits;
pub mod types;
pub mod utils_test;

// errors
pub use errors::graph_error::GraphError;
// extensions - optional. Must be enabled as feature in Cargo.toml
#[cfg(feature = "parallel")]
pub use extensions::graph_algo_ext::ParallelGraphAlgorithmsExt;
// traits
pub use traits::graph_algo::GraphAlgorithms;
pub use traits::graph_freeze::Freezable;
pub use traits::graph_mut::GraphMut;
pub use traits::graph_traversal::GraphTraversal;
pub use traits::graph_unfreeze::Unfreezable;
pub use traits::graph_view::GraphView;
// types
pub use types::graph_csm::CsmGraph;
pub use types::graph_dynamic::DynamicGraph;
