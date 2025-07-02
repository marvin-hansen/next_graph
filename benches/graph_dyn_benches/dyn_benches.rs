// In /benches/graph_dyn_benches/dyn_benches.rs

use criterion::{Criterion, criterion_group};
use next_graph::{DynamicGraph, GraphMut, GraphView};
use std::hint::black_box;

/// Helper to create a graph with a "hub" node connected to many "spoke" nodes.
/// This is ideal for testing `contains_edge` with a variable number of neighbors.
fn create_hub_graph(num_spokes: usize) -> DynamicGraph<(), ()> {
    let mut graph = DynamicGraph::with_capacity(num_spokes, None);
    let hub = graph.add_node(());
    for _ in 0..num_spokes {
        let spoke = graph.add_node(());
        graph.add_edge(hub, spoke, ()).unwrap();
    }
    graph
}

/// Helper to create a graph with a specific number of nodes and edges.
fn create_general_graph(num_nodes: usize, edges_per_node: usize) -> DynamicGraph<(), ()> {
    let mut graph = DynamicGraph::with_capacity(edges_per_node, None);
    for _ in 0..num_nodes {
        graph.add_node(());
    }

    // Add some edges in a predictable way
    if num_nodes > 1 {
        for i in 0..num_nodes {
            for j in 1..=edges_per_node {
                let target = (i + j) % num_nodes;
                graph.add_edge(i, target, ()).unwrap();
            }
        }
    }
    graph
}

pub fn bench_dyn_graph(c: &mut Criterion) {
    // --- Group 1: Node Operations ---
    let mut node_ops_group = c.benchmark_group("DynamicGraph Node Ops");

    node_ops_group.bench_function("add_node (to 1k graph)", |b| {
        b.iter_with_setup(
            || create_general_graph(1_000, 0),
            |mut graph| {
                black_box(graph.add_node(()));
            },
        );
    });

    node_ops_group.bench_function("remove_node (from 1k graph)", |b| {
        b.iter_with_setup(
            || create_general_graph(1_000, 5),
            |mut graph| {
                // Remove a node from the middle to measure tombstoning cost
                black_box(graph.remove_node(500)).unwrap();
            },
        );
    });
    node_ops_group.finish();

    // --- Group 2: Edge Operations ---
    let mut edge_ops_group = c.benchmark_group("DynamicGraph Edge Ops");

    edge_ops_group.bench_function("add_edge (to 1k graph)", |b| {
        b.iter_with_setup(
            || create_general_graph(1_000, 5),
            |mut graph| {
                // Add an edge between two arbitrary nodes
                black_box(graph.add_edge(100, 900, ())).unwrap();
            },
        );
    });

    edge_ops_group.bench_function("remove_edge (from 1k graph)", |b| {
        b.iter_with_setup(
            || {
                let mut graph = create_general_graph(1_000, 0);
                graph.add_edge(100, 900, ()).unwrap();
                graph
            },
            |mut graph| {
                // This measures the cost of finding and removing the edge
                black_box(graph.remove_edge(100, 900)).unwrap();
            },
        );
    });
    edge_ops_group.finish();

    // --- Group 3: Lookup Performance ---
    let mut lookup_group = c.benchmark_group("DynamicGraph Lookups");

    // Benchmark `contains_edge` with many neighbors to highlight linear scan cost.
    // This provides a direct comparison to the CsmGraph's binary search.
    let dense_graph = create_hub_graph(100);
    lookup_group.bench_function("contains_edge (linear scan, 100 neighbors)", |b| {
        b.iter(|| {
            // Check for an edge that exists and one that doesn't.
            black_box(dense_graph.contains_edge(0, 50));
            black_box(dense_graph.contains_edge(0, 101)); // non-existent
        })
    });
    lookup_group.finish();
}

criterion_group!(dyn_benches, bench_dyn_graph);
