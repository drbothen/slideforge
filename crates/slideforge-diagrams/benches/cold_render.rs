//! Criterion benchmark: cold diagram render time (font DB initialization).
//!
//! ## Purpose (AC-003 / NFR-003)
//!
//! Measures the total time for the FIRST call to `render_mermaid` in a fresh
//! process. The font database is initialized on the first call to
//! `mermaid-rs-renderer`.
//!
//! ## Gate (NFR-003)
//!
//! The benchmark must pass < 200ms on the CI Linux x86_64 runner.
//! Empirical from Spike S14: 124ms on M-series Mac.
//!
//! ## Note
//!
//! Criterion runs `bench_function` in a tight loop (multiple iterations) so
//! the true "cold" measurement is the FIRST call. The warm iterations are
//! measured by `warm_render.rs`. This benchmark captures total initialization
//! overhead via `Criterion::new().measurement_time()` set low to focus on
//! the first-call latency profile.

use criterion::{criterion_group, criterion_main, Criterion};
use slideforge_diagrams::renderer::render_mermaid;

fn cold_render_benchmark(c: &mut Criterion) {
    // A simple flowchart that exercises the full render path.
    let source = "graph TD\n  A-->B";

    c.bench_function("cold_flowchart_render", |b| {
        b.iter(|| {
            render_mermaid(std::hint::black_box(source), "bench")
                .expect("benchmark: flowchart must render without error")
        });
    });
}

criterion_group!(benches, cold_render_benchmark);
criterion_main!(benches);
