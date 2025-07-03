#![cfg(feature = "parallel")]
// This entire module becomes available only  if the parallel feature is enabled.
use crate::{CsmGraph, GraphTraversal, GraphView};
use rayon::prelude::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// A trait that provides parallel versions of graph algorithms.
///
/// This trait is intended for implementation on the static `CsmGraph` and leverages
/// Rayon for data parallelism. It is only available when the "parallel" feature
/// is enabled.
///
/// The methods require the node payload `N` and edge weight `W` to be thread-safe.
pub trait ParallelGraphAlgorithmsExt<N, W>: GraphView<N, W>
where
    N: Send + Sync,
    W: Send + Sync,
{
    // --- Parallel Structural Validation ---

    /// Computes a topological sort of the graph in parallel using a BFS-like approach (Kahn's Algorithm).
    ///
    /// This can be significantly faster than the sequential version for graphs with high
    /// levels of parallelism (i.e., many nodes with an in-degree of 0 at each step).
    ///
    /// # Returns
    /// `Some(Vec<usize>)` containing the node indices in a valid linear ordering if the
    /// graph is a DAG. Returns `None` if the graph contains a cycle.
    fn topological_sort_par(&self) -> Option<Vec<usize>>;

    // --- Parallel Pathfinding and Reachability Algorithms ---

    /// Checks if a path of any length exists from a start to a stop index, using a parallel BFS.
    ///
    /// This algorithm can be significantly faster on large, wide graphs with many paths.
    fn is_reachable_par(&self, start_index: usize, stop_index: usize) -> bool;

    /// Returns the length of the shortest path from a start to a stop index, using a parallel BFS.
    fn shortest_path_len_par(&self, start_index: usize, stop_index: usize) -> Option<usize>;

    /// Finds the complete shortest path from a start to a stop index, using a parallel BFS.
    fn shortest_path_par(&self, start_index: usize, stop_index: usize) -> Option<Vec<usize>>;
}

//
// --- Default implementation for CsmGraph --- ---
//
