//! Criterion benchmark: warm diagram render + normalize time (font DB cached).
//!
//! ## Purpose (AC-008 / NFR-004)
//!
//! Measures the per-call time for the full diagram pipeline (`render_mermaid` +
//! `usvg_normalize`) after `mermaid-rs-renderer` has already initialized its
//! internal font database (second and subsequent calls).
//!
//! ## Gate (NFR-004)
//!
//! The combined render + normalize pipeline must complete in < 10ms on the CI
//! Linux `x86_64` runner. The usvg normalization step is expected to add < 1ms
//! overhead (empirical from Spike S14 context). If normalization overhead
//! exceeds 10% of the warm budget (1ms), flag for architect review (AC-008).
//!
//! Empirical from Spike S14: < 3ms typical for flowcharts (render only).

// NOTE: Per AC-008, a < 1ms micro-benchmark isolating normalize-only
// overhead is deferred to STORY-037 (PPTX exporter). Current benchmarks
// measure combined render+normalize.

use criterion::{Criterion, criterion_group, criterion_main};
use slideforge_diagrams::{DiagramRendererImpl, types::DiagramLang};

/// Benchmark function: warm render+normalize — per-call time after font DB cached.
///
/// Exercises the full diagram pipeline: `render_mermaid` + `usvg_normalize`
/// (BC-1.12.003 invariant 1: normalization is mandatory after every render).
fn warm_render_benchmark(c: &mut Criterion) {
    let source = "graph TD\n  A-->B";

    // Warm up: call the full pipeline once to force font DB initialization.
    // The warmup call will panic (todo!()) until STORY-034 is implemented.
    // When implemented, this will successfully warm the font DB.
    let _ = DiagramRendererImpl::render_diagram(source, DiagramLang::Mermaid, "warmup");

    let mut group = c.benchmark_group("warm_render_normalize");

    group.bench_function("flowchart", |b| {
        b.iter(|| {
            DiagramRendererImpl::render_diagram(
                std::hint::black_box(source),
                DiagramLang::Mermaid,
                "bench",
            )
            .expect("benchmark: flowchart render+normalize must succeed")
        });
    });

    group.bench_function("sequence_diagram", |b| {
        let seq_source = "sequenceDiagram\n  Alice->>Bob: Hello";
        b.iter(|| {
            DiagramRendererImpl::render_diagram(
                std::hint::black_box(seq_source),
                DiagramLang::Mermaid,
                "bench",
            )
            .expect("benchmark: sequenceDiagram render+normalize must succeed")
        });
    });

    group.finish();
}

criterion_group!(benches, warm_render_benchmark);
criterion_main!(benches);
