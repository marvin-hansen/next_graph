// This entire file is only compiled when the "parallel" feature is enabled.
#![cfg(feature = "parallel")]

use criterion::{Criterion, criterion_group};
use next_graph::{DynamicGraph, Freezable, GraphMut, ParallelGraphAlgorithmsExt};
use std::hint::black_box;

/// Helper to create a large, general-purpose graph for stressing algorithms.
/// This is identical to the helper in the sequential benchmarks for a fair comparison.
fn create_general_graph_dyn(num_nodes: usize, edges_per_node: usize) -> DynamicGraph<(), ()> {
    let mut graph = DynamicGraph::with_capacity(num_nodes, Some(edges_per_node));
    for _ in 0..num_nodes {
        graph.add_node(());
    }

    if num_nodes > 1 {
        for i in 0..num_nodes {
            for j in 1..=edges_per_node {
                let target = (i + j * 37) % num_nodes; // Use a prime to spread edges
                graph.add_edge(i, target, ()).unwrap();
            }
        }
    }
    graph
}

/// Defines the benchmark suite for parallel algorithms on `CsmGraph`.
pub fn bench_csm_graph_par(c: &mut Criterion) {
    // --- Group: Large-Scale Parallel Algorithms (Cache-Stressed) ---
    // This group directly mirrors the "CsmGraph Sequential Algorithms (Large Scale)"
    // group, using the same graph data but calling the parallel methods.
    let mut large_par_algo_group = c.benchmark_group("CsmGraph Parallel Algorithms (Large Scale)");

    // Use a large graph that will not fit in the CPU cache to properly
    // test performance on memory-bound, multi-core workloads.
    let large_graph = create_general_graph_dyn(1_000_000, 5).freeze();

    large_par_algo_group.bench_function("topological_sort_par (DAG, 1M nodes, 5M edges)", |b| {
        b.iter(|| black_box(large_graph.topological_sort_par()))
    });

    large_par_algo_group.bench_function("shortest_path_par (1M nodes, 5M edges)", |b| {
        b.iter(|| black_box(large_graph.shortest_path_par(0, 999_999)))
    });

    large_par_algo_group.finish();
}

// Register this benchmark suite with Criterion.
criterion_group!(csm_par_benches, bench_csm_graph_par);
