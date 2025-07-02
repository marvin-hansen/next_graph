# next_graph

A high-performance, dual-state graph library for Rust, designed for an evolutionary lifecycle of mutation and analysis.

## Overview

`next_graph` is built around a unique concept: the **evolutionary lifecycle of a graph**. Many applications require a
graph to be built and modified (an "evolutionary" phase) and then heavily analyzed (an "analysis" phase). This library
provides two distinct graph representations tailored for each phase:

1. **`DynamicGraph`**: A flexible, **mutable** graph representation. It's designed for easy and efficient addition and
   removal of nodes and edges. This is your primary tool for building and evolving the graph's structure.

2. **`CsmGraph` (Compressed Sparse Matrix Graph)**: A high-performance, **immutable** graph representation. It uses a *
   *Compressed Sparse Row (CSR)** format, which provides extremely fast traversals and lookups. This is the ideal state
   for running complex algorithms after the graph structure has stabilized.

The power of `next_graph` lies in the seamless transition between these two states using a `freeze` and `unfreeze`
mechanism.

- **`.freeze()`**: Consumes a `DynamicGraph` and converts it into a highly optimized `CsmGraph`.
- **`.unfreeze()`**: Consumes a `CsmGraph` and converts it back into a `DynamicGraph`, ready for another round of
  mutations.

This dual-state design allows you to get the best of both worlds: a flexible, easy-to-use API for building your graph
and a blazingly fast, cache-friendly structure for analyzing it.

## Key Features

- **Dual-State System**: Switch between mutable `DynamicGraph` and read-optimized `CsmGraph`.
- **High-Performance Algorithms**: `CsmGraph` provides fast implementations for shortest path, topological sort, cycle
  detection, and more.
- **Ergonomic API**: A clean, trait-based API (`GraphView`, `GraphMut`, `GraphAlgorithms`) provides a consistent
  interface.
- **Stable Indices**: Node indices are stable throughout the graph's lifetime, even after node removals (which are
  handled via "tombstoning").
- **Performance-Aware Design**: `DynamicGraph` can be initialized with capacity hints to avoid reallocations, and
  `CsmGraph` uses an adaptive edge lookup that switches between linear scan and binary search for optimal performance.

## Installation

Add `next_graph` to your `Cargo.toml`:

## Usage

```rust
use next_graph::{DynamicGraph, Freezable, GraphAlgorithms, GraphMut, GraphView, Unfreezable};

fn main() {
// 1. Create a mutable `DynamicGraph` to build our initial structure.
//    We can provide capacity hints for performance.
let mut graph = DynamicGraph::with_capacity(5, Some(3));
println!("Phase 1: Building the graph...");

    // Add nodes and get their stable indices.
    let san_francisco = graph.add_node("San Francisco");
    let seattle = graph.add_node("Seattle");
    let chicago = graph.add_node("Chicago");
    let new_york = graph.add_node("New York");

    // Add weighted edges representing flight distances.
    graph.add_edge(san_francisco, seattle, 807).unwrap();
    graph.add_edge(seattle, chicago, 2062).unwrap();
    graph.add_edge(chicago, new_york, 790).unwrap();
    graph.add_edge(san_francisco, chicago, 2132).unwrap(); // A direct flight

    println!("- Graph has {} nodes and {} edges.", graph.number_nodes(), graph.number_edges());

    // 2. Freeze the graph for high-performance analysis.
    //    `.freeze()` consumes the DynamicGraph and returns an immutable CsmGraph.
    println!("\nPhase 2: Freezing for analysis...");
    let frozen_graph = graph.freeze();

    // `CsmGraph` uses a compact format (CSR) for fast queries.
    assert!(frozen_graph.is_frozen());

    // Edge lookups are extremely fast.
    assert!(frozen_graph.contains_edge(seattle, chicago));
    assert!(!frozen_graph.contains_edge(new_york, seattle));

    // Run complex algorithms.
    let path = frozen_graph.shortest_path(seattle, new_york).unwrap();
    let path_nodes: Vec<_> = path.iter().map(|&i| frozen_graph.get_node(i).unwrap()).collect();
    println!("- Shortest path from Seattle to New York: {:?}", path_nodes);
    assert_eq!(path_nodes, vec![&"Seattle", &"Chicago", &"New York"]);

    // 3. Unfreeze the graph to re-enter a mutable, evolutionary state.
    //    Note: Node and edge payloads must be `Clone` to unfreeze.
    println!("\nPhase 3: Unfreezing for further mutation...");
    let mut graph = frozen_graph.unfreeze();

    // Add a new city and connect it.
    let denver = graph.add_node("Denver");
    graph.add_edge(san_francisco, denver, 1267).unwrap();
    graph.add_edge(denver, chicago, 1003).unwrap();

    println!("- Graph now has {} nodes.", graph.number_nodes());

    // 4. Re-freeze for final analysis.
    println!("\nPhase 4: Re-freezing with new data...");
    let final_graph = graph.freeze();

    // The shortest path from SF to Chicago is now shorter through Denver.
    let new_path = final_graph.shortest_path(san_francisco, chicago).unwrap();
    let new_path_nodes: Vec<_> = new_path.iter().map(|&i| final_graph.get_node(i).unwrap()).collect();
    println!("- New shortest path from San Francisco to Chicago: {:?}", new_path_nodes);
    assert_eq!(new_path_nodes, vec![&"San Francisco", &"Denver", &"Chicago"]);
}
```

## 📜 Licence

This project is licensed under the [MIT license](LICENSE).

## 💻 Author

* [Marvin Hansen](https://github.com/marvin-hansen).
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC
