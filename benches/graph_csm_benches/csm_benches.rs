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
fn create_chain_graph_dyn(num_nodes: usize) -> DynamicGraph<(), ()> {
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

/// Helper to create a graph with a cycle.
/// This is a chain with a back-edge from the last node to the first.
fn create_cyclic_graph_dyn(num_nodes: usize) -> DynamicGraph<(), ()> {
    let mut graph = create_chain_graph_dyn(num_nodes);
    if num_nodes > 1 {
        graph.add_edge(num_nodes - 1, 0, ()).unwrap();
    }
    graph
}

/// Helper for large, more complex graphs
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

fn bench_csm_graph(c: &mut Criterion) {
    // --- Group 1: Small-Scale Lookups (Cache-Hot) ---
    let mut lookup_group = c.benchmark_group("CsmGraph Lookups (Small Scale)");
    let sparse_graph = create_hub_graph(10);
    lookup_group.bench_function("contains_edge (linear scan)", |b| {
        b.iter(|| {
            black_box(sparse_graph.contains_edge(0, 5));
            black_box(sparse_graph.contains_edge(0, 11));
        })
    });
    let dense_graph = create_hub_graph(100);
    lookup_group.bench_function("contains_edge (binary search)", |b| {
        b.iter(|| {
            black_box(dense_graph.contains_edge(0, 50));
            black_box(dense_graph.contains_edge(0, 101));
        })
    });
    lookup_group.finish();

    // --- Group 2: Small-Scale Algorithms (Cache-Hot) ---
    let mut algo_group = c.benchmark_group("CsmGraph Sequential Algorithms (Small Scale)");
    let chain_graph = create_chain_graph_dyn(1_000).freeze();
    let cyclic_graph = create_cyclic_graph_dyn(1_000).freeze();

    algo_group.bench_function("shortest_path (1k nodes)", |b| {
        b.iter(|| black_box(chain_graph.shortest_path(0, 999)))
    });
    algo_group.bench_function("topological_sort (DAG, 1k nodes)", |b| {
        b.iter(|| black_box(chain_graph.topological_sort()))
    });
    algo_group.bench_function("find_cycle (has cycle, 1k nodes)", |b| {
        b.iter(|| black_box(cyclic_graph.find_cycle()))
    });
    algo_group.finish();

    // --- Group 3: Large-Scale Algorithms (Cache-Stressed) ---
    // This is the benchmark that will show the real impact of the SoA refactor.
    // 1M nodes and 5M edges will not fit in the CPU cache.
    let mut large_algo_group = c.benchmark_group("CsmGraph Sequential Algorithms (Large Scale)");
    let large_graph = create_general_graph_dyn(1_000_000, 5).freeze();

    large_algo_group.bench_function("shortest_path (1M nodes, 5M edges)", |b| {
        b.iter(|| black_box(large_graph.shortest_path(0, 999_999)))
    });

    large_algo_group.bench_function("topological_sort (DAG, 1M nodes, 5M edges)", |b| {
        b.iter(|| black_box(large_graph.topological_sort()))
    });
    large_algo_group.finish();

    // --- Group 4: Graph Lifecycle Performance ---
    let mut construction_group = c.benchmark_group("CsmGraph Lifecycle");
    construction_group.bench_function("freeze (1k nodes, 999 edges)", |b| {
        b.iter_with_setup(
            || create_chain_graph_dyn(1_000),
            |graph_to_freeze| {
                black_box(graph_to_freeze.freeze());
            },
        );
    });
    construction_group.bench_function("unfreeze (1k nodes, 999 edges)", |b| {
        b.iter_with_setup(
            || create_chain_graph_dyn(1_000).freeze(),
            |frozen_graph| {
                black_box(frozen_graph.unfreeze());
            },
        );
    });

    // This will properly measure the performance of the freeze/unfreeze operations
    // on a graph that does not fit in the CPU cache.
    construction_group.bench_function("freeze (1M nodes, 5M edges)", |b| {
        b.iter_with_setup(
            || create_general_graph_dyn(1_000_000, 5),
            |graph_to_freeze| {
                black_box(graph_to_freeze.freeze());
            },
        );
    });
    construction_group.bench_function("unfreeze (1M nodes, 5M edges)", |b| {
        b.iter_with_setup(
            || create_general_graph_dyn(1_000_000, 5).freeze(),
            |frozen_graph| {
                black_box(frozen_graph.unfreeze());
            },
        );
    });
    construction_group.finish();
}

criterion_group!(csm_benches, bench_csm_graph);
