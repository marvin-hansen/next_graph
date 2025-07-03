use criterion::{Criterion, criterion_group};
use next_graph::{
    CsmGraph, DynamicGraph, Freezable, GraphAlgorithms, GraphMut, GraphView, Unfreezable,
};
use std::hint::black_box;

/// Helper to create a graph with a "hub" node connected to many "spoke" nodes.
/// This is ideal for testing `contains_edge` with a variable number of neighbors.
fn create_hub_graph(num_spokes: usize) -> CsmGraph<(), ()> {
    let mut graph = DynamicGraph::new();
    let hub = graph.add_node(());
    // Add spoke nodes and create edges from the hub to each spoke.
    for _ in 0..num_spokes {
        let spoke = graph.add_node(());
        graph.add_edge(hub, spoke, ()).unwrap();
    }
    graph.freeze()
}

/// Helper to create a long chain of nodes.
/// This is a simple Directed Acyclic Graph (DAG) useful for testing traversal algorithms.
fn create_chain_graph(num_nodes: usize) -> CsmGraph<(), ()> {
    let mut graph = DynamicGraph::new();
    if num_nodes == 0 {
        return graph.freeze();
    }
    let mut prev_node = graph.add_node(());
    for _ in 1..num_nodes {
        let current_node = graph.add_node(());
        graph.add_edge(prev_node, current_node, ()).unwrap();
        prev_node = current_node;
    }
    graph.freeze()
}

/// Helper to create a graph with a cycle.
/// This is a chain with a back-edge from the last node to the first.
fn create_cyclic_graph(num_nodes: usize) -> CsmGraph<(), ()> {
    let mut graph = DynamicGraph::new();

    let mut prev_node = graph.add_node(());
    for _ in 1..num_nodes {
        let current_node = graph.add_node(());
        graph.add_edge(prev_node, current_node, ()).unwrap();
        prev_node = current_node;
    }

    if num_nodes > 1 {
        // Create a cycle from the last node back to the root.
        graph.add_edge(num_nodes - 1, 0, ()).unwrap();
    }

    graph.freeze()
}

fn bench_csm_graph(c: &mut Criterion) {
    // --- Group 1: Edge Lookup Performance ---
    let mut lookup_group = c.benchmark_group("CsmGraph Lookups");

    // Benchmark `contains_edge` with a small number of neighbors to trigger linear scan.
    let sparse_graph = create_hub_graph(10);
    lookup_group.bench_function("contains_edge (linear scan)", |b| {
        b.iter(|| {
            // Check for an edge that exists and one that doesn't.
            black_box(sparse_graph.contains_edge(0, 5));
            black_box(sparse_graph.contains_edge(0, 11)); // non-existent
        })
    });

    // Benchmark `contains_edge` with many neighbors to trigger binary search.
    // The threshold is 64, so we use 100 neighbors.
    let dense_graph = create_hub_graph(100);
    lookup_group.bench_function("contains_edge (binary search)", |b| {
        b.iter(|| {
            // Check for an edge that exists and one that doesn't.
            black_box(dense_graph.contains_edge(0, 50));
            black_box(dense_graph.contains_edge(0, 101)); // non-existent
        })
    });
    lookup_group.finish();

    // --- Group 2: Sequential Graph Algorithm Performance ---
    let mut algo_group = c.benchmark_group("CsmGraph Sequential Algorithms");
    let chain_graph = create_chain_graph(1_000);
    let cyclic_graph = create_cyclic_graph(1_000);

    // --- Pathfinding & Reachability ---
    algo_group.bench_function("shortest_path (1k nodes)", |b| {
        b.iter(|| black_box(chain_graph.shortest_path(0, 999)))
    });

    algo_group.bench_function("shortest_path_len (1k nodes)", |b| {
        b.iter(|| black_box(chain_graph.shortest_path_len(0, 999)))
    });

    algo_group.bench_function("is_reachable (1k nodes)", |b| {
        b.iter(|| black_box(chain_graph.is_reachable(0, 999)))
    });

    // --- Structural Validation ---
    algo_group.bench_function("topological_sort (DAG, 1k nodes)", |b| {
        b.iter(|| black_box(chain_graph.topological_sort()))
    });

    algo_group.bench_function("find_cycle (has cycle, 1k nodes)", |b| {
        b.iter(|| black_box(cyclic_graph.find_cycle()))
    });

    algo_group.bench_function("has_cycle (has cycle, 1k nodes)", |b| {
        b.iter(|| black_box(cyclic_graph.has_cycle()))
    });

    algo_group.bench_function("find_cycle (no cycle, 1k nodes)", |b| {
        b.iter(|| black_box(chain_graph.find_cycle()))
    });

    algo_group.bench_function("has_cycle (no cycle, 1k nodes)", |b| {
        b.iter(|| black_box(chain_graph.has_cycle()))
    });
    algo_group.finish();

    // --- Group 3: Graph Lifecycle Performance ---
    let mut construction_group = c.benchmark_group("CsmGraph Lifecycle");

    // Benchmark the `freeze` operation using iter_with_setup.
    construction_group.bench_function("freeze (1k nodes, 999 edges)", |b| {
        b.iter_with_setup(
            || {
                // SETUP: This code runs before each measurement but is not timed.
                create_chain_graph(1_000)
            },
            |graph_to_freeze| {
                // BENCH: This is the code that gets measured.
                black_box(graph_to_freeze);
            },
        );
    });

    // Benchmark the `unfreeze` operation.
    construction_group.bench_function("unfreeze (1k nodes, 999 edges)", |b| {
        b.iter_with_setup(
            || {
                // SETUP: This code runs before each measurement but is not timed.
                create_chain_graph(1_000)
            },
            |frozen_graph| {
                // BENCH: This is the code that gets measured.
                black_box(frozen_graph.unfreeze());
            },
        );
    });
    construction_group.finish();
}

criterion_group!(csm_benches, bench_csm_graph);
