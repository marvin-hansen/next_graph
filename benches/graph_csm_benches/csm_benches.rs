use criterion::{Criterion, criterion_group};
use next_graph::{DynamicGraph, Freezable, GraphAlgorithms, GraphMut, GraphView, Unfreezable};
use std::hint::black_box;

/// Helper to create a graph with a "hub" node connected to many "spoke" nodes.
/// This is ideal for testing `contains_edge` with a variable number of neighbors.
fn create_hub_graph(num_spokes: usize) -> DynamicGraph<(), ()> {
    let mut graph = DynamicGraph::new();
    let hub = graph.add_node(());
    // Add spoke nodes and create edges from the hub to each spoke.
    for _ in 0..num_spokes {
        let spoke = graph.add_node(());
        graph.add_edge(hub, spoke, ()).unwrap();
    }
    graph
}

/// Helper to create a long chain of nodes.
/// This is a simple Directed Acyclic Graph (DAG) useful for testing traversal algorithms.
fn create_chain_graph(num_nodes: usize) -> DynamicGraph<(), ()> {
    let mut graph = DynamicGraph::new();
    if num_nodes == 0 {
        return graph;
    }
    let mut prev_node = graph.add_node(());
    for _ in 1..num_nodes {
        let current_node = graph.add_node(());
        graph.add_edge(prev_node, current_node, ()).unwrap();
        prev_node = current_node;
    }
    graph
}

fn bench_csm_graph(c: &mut Criterion) {
    // --- Group 1: Edge Lookup Performance ---
    let mut lookup_group = c.benchmark_group("CsmGraph Lookups");

    // Benchmark `contains_edge` with a small number of neighbors to trigger linear scan.
    let sparse_graph = create_hub_graph(10).freeze();
    lookup_group.bench_function("contains_edge (linear scan)", |b| {
        b.iter(|| {
            // Check for an edge that exists and one that doesn't.
            black_box(sparse_graph.contains_edge(0, 5));
            black_box(sparse_graph.contains_edge(0, 11)); // non-existent
        })
    });

    // Benchmark `contains_edge` with many neighbors to trigger binary search.
    // The threshold is 64, so we use 100 neighbors.
    let dense_graph = create_hub_graph(100).freeze();
    lookup_group.bench_function("contains_edge (binary search)", |b| {
        b.iter(|| {
            // Check for an edge that exists and one that doesn't.
            black_box(dense_graph.contains_edge(0, 50));
            black_box(dense_graph.contains_edge(0, 101)); // non-existent
        })
    });
    lookup_group.finish();

    // --- Group 2: Graph Algorithm Performance ---
    let mut algo_group = c.benchmark_group("CsmGraph Algorithms");
    let chain_graph = create_chain_graph(1_000).freeze();

    algo_group.bench_function("topological_sort (1k nodes)", |b| {
        b.iter(|| {
            black_box(chain_graph.topological_sort());
        })
    });

    algo_group.bench_function("shortest_path (1k nodes)", |b| {
        b.iter(|| {
            black_box(chain_graph.shortest_path(0, 999));
        })
    });
    algo_group.finish();

    // --- Group 3: Graph Lifecycle Performance ---
    let mut construction_group = c.benchmark_group("CsmGraph Lifecycle");

    // Benchmark the `freeze` operation using iter_with_setup.
    construction_group.bench_function("freeze (1k nodes, 999 edges)", |b| {
        b.iter_with_setup(
            || {
                // SETUP: This code runs before each measurement but is not timed.
                // It creates the DynamicGraph we want to freeze.
                create_chain_graph(1_000)
            },
            |graph_to_freeze| {
                // BENCH: This is the code that gets measured.
                // It consumes the graph created in the setup block.
                black_box(graph_to_freeze.freeze());
            },
        );
    });

    // Benchmark the `unfreeze` operation.
    // We use `iter_with_setup` to ensure the setup cost of creating the CsmGraph
    // is not included in the measurement.
    construction_group.bench_function("unfreeze (1k nodes, 999 edges)", |b| {
        b.iter_with_setup(
            || {
                // SETUP: This code runs before each measurement but is not timed.
                // It creates the CsmGraph we want to unfreeze.
                create_chain_graph(1_000).freeze()
            },
            |frozen_graph| {
                // BENCH: This is the code that gets measured.
                // It consumes the graph created in the setup block.
                black_box(frozen_graph.unfreeze());
            },
        );
    });
    construction_group.finish();
}

criterion_group!(csm_benches, bench_csm_graph);
