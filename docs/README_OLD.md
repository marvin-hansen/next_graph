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

Add `next_graph` to your `Cargo.toml` as git dependency:

```toml
[dependencies]
next_graph = { git = "https://github.com/marvin-hansen/next_graph.git" , branch = "main" }
```

Note, there is no official release on crates.io so you have to use git. See
the [official documentation for details](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html).

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

## 🚀 Performance

`next_graph` is designed from the ground up for performance and efficiency, allowing it to handle
large-scale graphs with millions of nodes and edges with ease. Its core data structure, `CsmGraph`, is an immutable,
cache-friendly representation that enables lightning-fast traversals and analytics.

The performance benchmarks below were run on an Apple M3 Max with 16 cores.

### Benchmark Results

| Operation       | Scale | Graph Configuration                          |  Mean Time  | Throughput (Est.)        |
|:----------------|:------|:---------------------------------------------|:-----------:|:-------------------------|
| **Edge Lookup** | Tiny  | `contains_edge` (Linear Scan, degree < 64)   | **~7.7 ns** | ~130 Million lookups/sec |
|                 | Tiny  | `contains_edge` (Binary Search, degree > 64) | **~8.2 ns** | ~122 Million lookups/sec |
| **Algorithms**  | Small | `shortest_path` (1k nodes)                   | **~5.3 µs** | ~188,000 paths/sec       |
|                 | Small | `topological_sort` (1k nodes, DAG)           | **~5.2 µs** | ~192,000 sorts/sec       |
|                 | Small | `find_cycle` (1k nodes, has cycle)           | **~7.1 µs** | ~140,000 checks/sec      |
|                 | Large | `shortest_path` (1M nodes, 5M edges)         | **~482 µs** | ~2,000 paths/sec         |
|                 | Large | `topological_sort` (1M nodes, 5M edges)      | **~2.9 ms** | ~345 sorts/sec           |
| **Lifecycle**   | Small | `freeze` (1k nodes, 999 edges)               | **~42 µs**  | ~23,800 freezes/sec      |
|                 | Small | `unfreeze` (1k nodes, 999 edges)             | **~12 µs**  | ~81,600 unfreezes/sec    |
|                 | Large | `freeze` (1M nodes, 5M edges)                | **~75 ms**  | ~13 freezes/sec          |
|                 | Large | `unfreeze` (1M nodes, 5M edges)              | **~24 ms**  | ~41 unfreezes/sec        |

*(Note: Time units are nanoseconds (ns), microseconds (µs), and milliseconds (ms). Throughput is an approximate
calculation based on the mean time.)*

### Performance Design

The design of `next_graph`'s static analysis structure, `CsmGraph`, is based on the principles for high-performance
sparse graph representation detailed in the paper "NWHy: A Framework for Hypergraph Analytics" (Liu et al.).
Specifically, `next_graph` adopts the paper's foundational model of using two mutually-indexed Compressed Sparse Row (
CSR) structures to enable efficient, `O(degree)` bidirectional traversal—one for forward (outbound) edges and one for
the transposed graph for backward (inbound) edges.

However, `next_graph` introduces three significant architectural enhancements over this baseline to provide optimal
performance and to support the specific requirements of dynamically evolving systems.

1. **Struct of Arrays (SoA) Memory Layout:** The internal CSR adjacency structures are implemented using a Struct of
   Arrays layout. Instead of a single `Vec<(target, weight)>`, `next_graph` uses two parallel vectors: `Vec<target>` and
   `Vec<weight>`. This memory layout improves data locality for topology-only algorithms (e.g., reachability, cycle
   detection). By iterating exclusively over the `targets` vector, these algorithms avoid loading unused edge weight
   data into the CPU cache, which minimizes memory bandwidth usage and reduces cache pollution.

2. **Adaptive Edge Containment Checks:** The `contains_edge` method employs a hybrid algorithm that adapts to the data's
   shape at runtime. It performs an `O(1)` degree check on the source node and selects the optimal search strategy: a
   cache-friendly linear scan for low-degree nodes (where the number of neighbors is less than a compile-time threshold,
   e.g., 64) and a logarithmically faster binary search for high-degree nodes. This ensures the best possible lookup
   performance across varied graph structures.

3. **Formal Evolutionary Lifecycle:** The most significant architectural addition is a formal two-state model for graph
   evolution. `next_graph` defines two distinct representations: a mutable `DynamicGraph` optimized for efficient `O(1)`
   node and edge additions, and the immutable `CsmGraph` optimized for analysis. The library provides high-performance
   `O(V + E)` `.freeze()` and `.unfreeze()` operations to transition between these states. This two-state model directly
   supports systems that require dynamic structural evolution, such as those modeling emergent causality, by providing a
   controlled mechanism to separate the mutation phase from the immutable analysis phase.

While the NWHypergraph paper provides an excellent blueprint for a high-performance static graph engine, these
modifications extend that foundation into a more flexible, cache-aware, and dynamically adaptable framework
purpose-built for the lifecycle of evolving graph systems.

### Estimated Performance and Memory at Scale

Ideal Conditions: Time estimates assume linear scalability and do not account for potential system bottlenecks like
memory bandwidth saturation at extreme scales. These numbers represent a best-case scenario based on the initial
benchmarks.

1. **Complexity:** Time and space complexity are assumed to be **O(V + E)**.
2. **Density:** The graph is assumed to have a constant density of **5 edges per node** (`E = 5 * V`).
3. **Data Types:** Memory is calculated assuming `u64` for node payloads (8 bytes) and `u64` for edge weights (8 bytes).
   `usize` is assumed to be 8 bytes (64-bit system). The `CsrAdjacency` uses the optimized Struct of Arrays (SoA)
   layout.
4. **Ideal Conditions:** Time estimates assume linear scalability and do not account for potential system bottlenecks
   like memory bandwidth saturation at extreme scales. These numbers represent a best-case scenario based on the initial
   benchmarks.

| Graph Size (Nodes) | Edges (Est.) | Memory Footprint (Est.) | Freeze Time (Est.) | Shortest Path (Est.) | Topo Sort / Find Cycle (Est.) |
|:-------------------|:-------------|:------------------------|:-------------------|:---------------------|:------------------------------|
| **1 Million**      | 5 Million    | **~184 MB**             | **75 ms**          | **0.48 ms**          | **2.9 ms**                    |
| **10 Million**     | 50 Million   | **~1.8 GB**             | **750 ms**         | **4.8 ms**           | **29 ms**                     |
| **50 Million**     | 250 Million  | **~9.2 GB**             | **3.75 seconds**   | **24 ms**            | **145 ms**                    |
| **100 Million**    | 500 Million  | **~18.4 GB**            | **7.5 seconds**    | **48 ms**            | **290 ms**                    |
| **500 Million**    | 2.5 Billion  | **~92 GB**              | **37.5 seconds**   | **241 ms**           | **1.45 seconds**              |
| **1 Billion**      | 5 Billion    | **~184 GB**             | **~75 seconds**    | **~482 ms**          | **~2.9 seconds**              |

---

These are estimated values interpolated from the previous benchmark results meant as an indicator for
how much hardware each class of graph size would require. For precise measurements, please design and
run realistic benchmarks that approximate your specific workload.

For any graph that can fit within a typical server's RAM, nearly every analytical query
is sub-second. This is the sweet spot for interactive, near-real-time data exploration on a massive scale.

The table shows that the primary limiting factor is memory. A graph with 500 million nodes is
computationally trivial to analyze (a topological_sort takes less than 1.5 seconds),  
but it requires a machine with at least 92 GB of available RAM.

**Billion node graphs:** With a freeze time of just over a minute and a memory footprint under
200 GB, this class of problem is solvable on a single high-memory workstation available today.
For example, a commercially available M3 Ultra Mac Studio can be configured with up to 512GB unified memory
that haas 819GB/s memory bandwidth and thus process complex graphs up to 2 billion nodes and 5 billion edges
with single digit second processing time. As a matter of fact, a single data center server (for example, Dell PowerEdge)
with 3 TB of memory can process a graph with a staggering 16 billion nodes and up to 80 billion edges and still complete
the shortest path algorithm within 10 seconds or less.

## Misc.

Parallel graph algorithms have been experimented with using Rayon, but none yielded any performance improvements.
Therefore, the parallel implementation has been feature gated and moved inside an optional type extension.
It is not entirely clear why the parallelism on graphs with one million nodes yielded no improvement,
but at this stage it is assumed to be an implementation problem because parallel graph
algorithms are notoriously hard to implement correctly. Contributions and PR's with improvements of
the parallel code are welcome. The general recommendation is to stick with the existing algorithms as these are the most
optimized.

## 📜 Licence

This project is licensed under the [MIT license](LICENSE).

## 💻 Author

* [Marvin Hansen](https://github.com/marvin-hansen).
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC
