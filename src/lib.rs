//! Newman-Girvan modularity Q of a graph partition — value-exact port of
//! `networkx.algorithms.community.quality.modularity` (undirected, unweighted,
//! `weight=None`).
//!
//! For the reduced form networkx actually evaluates (Clauset-Newman-Moore 2004),
//! the modularity is
//!
//! ```text
//! Q = Σ_c [ L_c / m  −  γ · out_degree_sum · in_degree_sum · norm ]
//! ```
//!
//! For an undirected graph `out_degree_sum == in_degree_sum` is the sum of node
//! degrees in community `c`, `deg_sum = Σ_v deg(v)`, `m = deg_sum / 2`,
//! `norm = 1 / deg_sum²`, and `L_c` is the intra-community edge weight. A
//! self-loop follows networkx semantics: it adds 2 to its node's degree and
//! counts as one within-community edge in `L_c`. The per-community terms are
//! summed left-to-right in the order the communities are given — matching
//! networkx's `sum(map(...))` — so the floating-point result is bit-identical.

use std::collections::HashMap;

use rsomics_common::RsomicsError;
use serde::Serialize;

/// Undirected simple graph over interned integer node ids. Neighbour lists are
/// dedup'd so parallel edges collapse, and a self-loop is recorded once per node
/// via `self_loop`, matching a bona-fide `nx.Graph` built from edge-list input.
pub struct Graph {
    node_to_idx: HashMap<String, usize>,
    idx_to_node: Vec<String>,
    adj: Vec<Vec<usize>>,
    self_loop: Vec<bool>,
}

impl Graph {
    fn intern(&mut self, name: &str) -> usize {
        if let Some(&idx) = self.node_to_idx.get(name) {
            return idx;
        }
        let idx = self.idx_to_node.len();
        self.node_to_idx.insert(name.to_owned(), idx);
        self.idx_to_node.push(name.to_owned());
        self.adj.push(Vec::new());
        self.self_loop.push(false);
        idx
    }

    #[must_use]
    pub fn n(&self) -> usize {
        self.idx_to_node.len()
    }

    /// networkx degree: a self-loop contributes 2.
    fn degree(&self, v: usize) -> usize {
        self.adj[v].len() + if self.self_loop[v] { 2 } else { 0 }
    }
}

/// Parse a whitespace-delimited `u v` edge list. `#` comments and blank lines
/// are skipped, parallel edges deduplicated, and a repeated self-loop collapses
/// to a single one — as in a simple `nx.Graph`.
#[must_use]
pub fn parse_edge_list(input: &str) -> Graph {
    let mut g = Graph {
        node_to_idx: HashMap::new(),
        idx_to_node: Vec::new(),
        adj: Vec::new(),
        self_loop: Vec::new(),
    };

    for line in input.lines() {
        // nx.parse_edgelist strips a '#' comment anywhere in the line before tokenising.
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let (Some(u), Some(v)) = (parts.next(), parts.next()) else {
            continue;
        };
        let ui = g.intern(u);
        let vi = g.intern(v);
        if ui == vi {
            g.self_loop[ui] = true;
            continue;
        }
        if !g.adj[ui].contains(&vi) {
            g.adj[ui].push(vi);
            g.adj[vi].push(ui);
        }
    }
    g
}

/// The modularity payload; serialised as `{"modularity": Q}` under `--json`.
#[derive(Debug, Clone, Serialize)]
pub struct Modularity {
    pub modularity: f64,
}

/// Assign every graph node to a community id from `(node, label)` pairs.
///
/// Community iteration order is the first-appearance order of the labels — the
/// order the reduced modularity sum walks over communities, kept deterministic
/// so the fold order (and thus the f64 result) matches a fixed networkx
/// `communities` list. networkx requires the communities to *partition* the
/// nodes: every node assigned exactly once and no stray labels. A missing or
/// doubly-assigned node raises `NotAPartition`; we mirror that as a loud error.
fn community_ids(
    g: &Graph,
    assignments: &[(String, String)],
    n_communities: &mut usize,
) -> Result<Vec<usize>, RsomicsError> {
    let n = g.n();
    let mut comm_of = vec![usize::MAX; n];
    let mut label_to_id: HashMap<&str, usize> = HashMap::new();
    let mut next_id = 0usize;

    for (node, label) in assignments {
        let Some(&idx) = g.node_to_idx.get(node.as_str()) else {
            return Err(RsomicsError::InvalidInput(format!(
                "community node {node:?} is not present in the graph — not a partition"
            )));
        };
        if comm_of[idx] != usize::MAX {
            return Err(RsomicsError::InvalidInput(format!(
                "node {node:?} assigned to more than one community — not a partition"
            )));
        }
        let id = *label_to_id.entry(label.as_str()).or_insert_with(|| {
            let id = next_id;
            next_id += 1;
            id
        });
        comm_of[idx] = id;
    }

    if let Some(unassigned) = comm_of.iter().position(|&c| c == usize::MAX) {
        let name = &g.idx_to_node[unassigned];
        return Err(RsomicsError::InvalidInput(format!(
            "node {name:?} is not assigned to any community — not a partition"
        )));
    }

    *n_communities = next_id;
    Ok(comm_of)
}

/// Newman-Girvan modularity of `g` under the community assignment, replicating
/// networkx's reduced-formula arithmetic term-for-term.
fn modularity_q(g: &Graph, comm_of: &[usize], n_communities: usize, resolution: f64) -> f64 {
    let n = g.n();

    let mut deg_sum_int: u128 = 0;
    for v in 0..n {
        deg_sum_int += g.degree(v) as u128;
    }
    let deg_sum = deg_sum_int as f64;
    let m = deg_sum / 2.0;
    let norm = 1.0 / ((deg_sum_int * deg_sum_int) as f64);

    let mut l_c = vec![0u64; n_communities];
    let mut degree_sum = vec![0u128; n_communities];
    for v in 0..n {
        degree_sum[comm_of[v]] += g.degree(v) as u128;
    }
    for v in 0..n {
        let cv = comm_of[v];
        for &w in &g.adj[v] {
            if comm_of[w] == cv && v < w {
                l_c[cv] += 1;
            }
        }
        if g.self_loop[v] {
            l_c[cv] += 1;
        }
    }

    (0..n_communities)
        .map(|c| {
            let ds = degree_sum[c] as f64;
            l_c[c] as f64 / m - resolution * ds * ds * norm
        })
        .sum()
}

/// Parse an edge list, assign communities and compute Q in one call.
///
/// # Errors
/// Returns `InvalidInput` if the assignments do not partition the graph's nodes.
pub fn modularity_from_edge_list(
    input: &str,
    assignments: &[(String, String)],
    resolution: f64,
) -> Result<Modularity, RsomicsError> {
    let g = parse_edge_list(input);
    let mut n_communities = 0usize;
    let comm_of = community_ids(&g, assignments, &mut n_communities)?;
    let q = modularity_q(&g, &comm_of, n_communities, resolution);
    Ok(Modularity { modularity: q })
}

#[cfg(test)]
mod tests {
    use super::modularity_from_edge_list;

    #[test]
    fn inline_hash_comment_matches_clean_graph() {
        let assignments = [
            ("0".to_owned(), "a".to_owned()),
            ("1".to_owned(), "a".to_owned()),
            ("2".to_owned(), "b".to_owned()),
            ("3".to_owned(), "b".to_owned()),
        ];
        let commented = "0 1\n1 2#c\n2 3\n0 #x\n";
        let clean = "0 1\n1 2\n2 3\n";
        let q_commented = modularity_from_edge_list(commented, &assignments, 1.0).unwrap();
        let q_clean = modularity_from_edge_list(clean, &assignments, 1.0).unwrap();
        assert_eq!(q_commented.modularity, q_clean.modularity);
    }
}
