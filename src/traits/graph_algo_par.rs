

#![cfg(feature = "parallel")] // This entire module only exists if the feature is enabled.
use crate::GraphView;
use rayon::prelude::*;

/// An extension trait for running graph algorithms in parallel using Rayon.
///
/// This trait is only available when the "parallel" feature is enabled. It provides
/// methods that leverage multiple cores to accelerate computations on large graphs.
///
/// To use these methods, you must enable the `parallel` feature in your `Cargo.toml`
/// and bring this trait into scope: `use next_graph::ParallelGraphAlgorithms;`
pub trait GraphAlgorithmsParallel<N, W>: GraphView<N, W> + Sync
where
    N: Sync,
    W: Sync,
{
    fn par_find_node<'a, P>(&'a self, predicate: P) -> Option<(usize, &'a N)>
    where
        P: Fn(&(usize, &'a N)) -> bool + Sync + Send;
}