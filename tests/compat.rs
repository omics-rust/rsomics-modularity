//! Value-exact compatibility with networkx 3.6.1 `nx.community.modularity`
//! (`weight=None`). Golden Q values were generated once with NetworkX 3.6.1 and
//! are frozen here as full-precision f64 constants; the fixtures under
//! `tests/golden/` are the exact edge lists and partitions they were computed
//! from. No Python or subprocess runs at test time.

use rsomics_modularity::modularity_from_edge_list;

/// Bit-level tolerance: value-exact ports must land within ~1 ULP.
const EPS: f64 = 1e-12;

fn parse_parts(text: &str) -> Vec<(String, String)> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let mut it = l.split_whitespace();
            (it.next().unwrap().to_owned(), it.next().unwrap().to_owned())
        })
        .collect()
}

fn q(edges: &str, parts: &str, resolution: f64) -> f64 {
    modularity_from_edge_list(edges, &parse_parts(parts), resolution)
        .unwrap()
        .modularity
}

macro_rules! case {
    ($name:ident, $edges:literal, $parts:literal, $res:expr, $golden:expr) => {
        #[test]
        fn $name() {
            let got = q(
                include_str!(concat!("golden/", $edges)),
                include_str!(concat!("golden/", $parts)),
                $res,
            );
            assert!(
                (got - $golden).abs() <= EPS,
                "{}: got {:.17} expected {:.17} (Δ={:e})",
                stringify!($name),
                got,
                $golden,
                got - $golden
            );
        }
    };
}

case!(
    two_triangles,
    "two_triangles.txt",
    "two_triangles.part",
    1.0,
    0.5
);
case!(path6, "path6.txt", "path6.part", 1.0, 0.30000000000000004);
case!(k5, "k5.txt", "k5.part", 1.0, 0.0);
case!(
    karate_2comm,
    "karate.txt",
    "karate_2comm.part",
    1.0,
    0.3582347140039448
);
case!(
    karate_2comm_res2,
    "karate.txt",
    "karate_2comm.part",
    2.0,
    -0.14250493096646943
);
case!(
    karate_4comm,
    "karate.txt",
    "karate_4comm.part",
    1.0,
    0.3806706114398422
);
case!(gnm_a, "gnm_a.txt", "gnm_a.part", 1.0, 0.3328858024691358);
case!(gnm_b, "gnm_b.txt", "gnm_b.part", 1.0, 0.296784375);

// Self-loops: networkx counts a self-loop as +2 degree and one within-community
// edge. `selfloop_bridge` = triangle {a,b,c} bridged to d with a self-loop on d;
// `selfloop_isolated` = triangle plus a node whose only edge is a self-loop.
case!(
    selfloop_bridge,
    "selfloop_bridge.txt",
    "selfloop_bridge.part",
    1.0,
    0.22
);
case!(
    selfloop_bridge_res2,
    "selfloop_bridge.txt",
    "selfloop_bridge.part",
    2.0,
    -0.36
);
case!(
    selfloop_isolated,
    "selfloop_isolated.txt",
    "selfloop_isolated.part",
    1.0,
    0.375
);
case!(
    selfloop_isolated_res2,
    "selfloop_isolated.txt",
    "selfloop_isolated.part",
    2.0,
    -0.25
);

#[test]
fn not_a_partition_missing_node_bails() {
    let edges = "a b\nb c\nc a\n";
    // node `c` is unassigned
    let parts = [
        ("a".to_owned(), "0".to_owned()),
        ("b".to_owned(), "0".to_owned()),
    ];
    let err = modularity_from_edge_list(edges, &parts, 1.0).unwrap_err();
    assert!(
        err.to_string().contains("not a partition"),
        "expected NotAPartition-style error, got: {err}"
    );
}

#[test]
fn not_a_partition_double_assigned_bails() {
    let edges = "a b\nb c\nc a\n";
    let parts = [
        ("a".to_owned(), "0".to_owned()),
        ("b".to_owned(), "0".to_owned()),
        ("c".to_owned(), "0".to_owned()),
        ("a".to_owned(), "1".to_owned()),
    ];
    let err = modularity_from_edge_list(edges, &parts, 1.0).unwrap_err();
    assert!(err.to_string().contains("not a partition"), "got: {err}");
}

#[test]
fn stray_community_node_bails() {
    let edges = "a b\nb c\nc a\n";
    let parts = [
        ("a".to_owned(), "0".to_owned()),
        ("b".to_owned(), "0".to_owned()),
        ("c".to_owned(), "0".to_owned()),
        ("z".to_owned(), "1".to_owned()),
    ];
    let err = modularity_from_edge_list(edges, &parts, 1.0).unwrap_err();
    assert!(err.to_string().contains("not a partition"), "got: {err}");
}
