# rsomics-modularity

Newman-Girvan modularity `Q` of a graph partition — a value-exact Rust port of
`networkx.community.modularity` for the undirected, unweighted case.

Given an undirected graph and a partition of its nodes into communities, the
modularity measures how much more densely connected the communities are than a
random graph with the same degree sequence. It is the objective that Louvain,
CNM and Leiden maximise.

## Install

```sh
cargo install rsomics-modularity
```

## Usage

```sh
# edge list on stdin, partition in a file
rsomics-modularity --communities parts.txt < graph.edges

# from a file, with a resolution parameter
rsomics-modularity graph.edges --communities parts.txt --resolution 2.0

# machine-readable
rsomics-modularity graph.edges --communities parts.txt --json
# {"modularity": 0.3582347140039448}
```

**Graph** — an undirected edge list, one `u v` per line. `#` comments and blank
lines are ignored; node names are arbitrary whitespace-free strings. Parallel
edges are deduplicated and self-loops dropped, giving the simple `nx.Graph` a
bona-fide edge list yields.

**`--communities FILE`** (required) — `node community_label` per line. Every
node in the graph must be assigned to exactly one community; a missing,
doubly-assigned or stray node is a hard error, matching networkx's
`NotAPartition`. The community iteration order — which fixes the left-to-right
order of the modularity sum, and hence the exact f64 result — is the
first-appearance order of the community labels in this file.

**`--resolution R`** (default `1.0`) — the resolution parameter γ. Below 1
favours larger communities, above 1 smaller ones.

## Formula

For the reduced form networkx actually evaluates (Clauset-Newman-Moore),

```
Q = Σ_c [ L_c / m  −  γ · k_c² · norm ]
```

where the sum runs over communities `c` in the order given, `L_c` is the number
of intra-community edges of `c` (each counted once), `k_c` is the sum of node
degrees in `c`, `deg_sum = Σ_v deg(v)`, `m = deg_sum / 2`, `norm = 1 / deg_sum²`
and `γ` is the resolution parameter. The arithmetic and summation order mirror
networkx term-for-term, so `Q` is bit-identical (≤ 1 ULP).

The win over networkx is algorithmic: labels are interned to `0..n` integer ids
once, then `L_c` and `k_c` are accumulated in a single O(m) pass over the edges —
no per-community subgraph is built.

## Origin

This crate is an independent Rust reimplementation of
`networkx.community.modularity` based on:

- The NetworkX 3.6.1 source of
  `networkx.algorithms.community.quality.modularity`
  (BSD-3-Clause — reading and citing is permitted).
- The published method: A. Clauset, M. E. J. Newman and C. Moore, "Finding
  community structure in very large networks", *Phys. Rev. E* 70, 066111 (2004).
  doi:10.1103/PhysRevE.70.066111. See also M. E. J. Newman and M. Girvan,
  "Finding and evaluating community structure in networks", *Phys. Rev. E* 69,
  026113 (2004). doi:10.1103/PhysRevE.69.026113.
- Black-box behaviour testing against NetworkX 3.6.1; golden `Q` values were
  generated with NetworkX 3.6.1 and frozen as full-precision f64 constants.

License: MIT OR Apache-2.0.
Upstream credit: NetworkX <https://networkx.org> (BSD-3-Clause).
