use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

use rsomics_modularity::modularity_from_edge_list;

const EDGES: &str = include_str!("gnm_bench.txt");
const PARTS: &str = include_str!("gnm_bench.part");

fn assignments() -> Vec<(String, String)> {
    PARTS
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| {
            let mut it = l.split_whitespace();
            (it.next().unwrap().to_owned(), it.next().unwrap().to_owned())
        })
        .collect()
}

fn bench_modularity(c: &mut Criterion) {
    let parts = assignments();
    c.bench_function("modularity_gnm_1000_8000", |b| {
        b.iter(|| {
            let q = modularity_from_edge_list(black_box(EDGES), black_box(&parts), 1.0).unwrap();
            black_box(q);
        });
    });
}

criterion_group!(benches, bench_modularity);
criterion_main!(benches);
