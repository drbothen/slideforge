//! Criterion benchmark: warm diagram render time (font DB already cached).
//!
//! ## Purpose (AC-004 / NFR-004)
//!
//! Measures the per-call render time after `mermaid-rs-renderer` has already
//! initialized its internal font database (second and subsequent calls).
//!
//! ## Gate (NFR-004)
//!
//! The benchmark must pass < 10ms on the CI Linux x86_64 runner.
//! Empirical from Spike S14: < 3ms typical for flowcharts.

use criterion::{criterion_group, criterion_main, Criterion};
use slideforge_diagrams::renderer::render_mermaid;

fn warm_render_benchmark(c: &mut Criterion) {
    // Warm up: call once to force font DB initialization before measuring.
    let source = "graph TD\n  A-->B";
    let _ = render_mermaid(source, "warmup");

    let mut group = c.benchmark_group("warm_render");

    group.bench_function("flowchart", |b| {
        b.iter(|| {
            render_mermaid(std::hint::black_box(source), "bench")
                .expect("benchmark: flowchart must render without error")
        });
    });

    group.bench_function("sequence_diagram", |b| {
        let seq_source = "sequenceDiagram\n  Alice->>Bob: Hello";
        b.iter(|| {
            render_mermaid(std::hint::black_box(seq_source), "bench")
                .expect("benchmark: sequenceDiagram must render without error")
        });
    });

    group.finish();
}

criterion_group!(benches, warm_render_benchmark);
criterion_main!(benches);
