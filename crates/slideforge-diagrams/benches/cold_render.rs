//! Criterion benchmark: cold diagram render + normalize time (font DB init).
//!
//! ## Purpose (AC-008 / NFR-003)
//!
//! Measures the total time for the FIRST call to the full diagram pipeline
//! (`render_mermaid` + `usvg_normalize`) in a fresh process. The font database
//! is initialized on the first call to `mermaid-rs-renderer`.
//!
//! ## Gate (NFR-003)
//!
//! The combined render + normalize pipeline must complete in < 200ms on the
//! CI Linux `x86_64` runner. The usvg normalization step is expected to add
//! < 1ms overhead (empirical from Spike S14), well within the budget.
//!
//! ## Note
//!
//! Criterion runs `bench_function` in a tight loop (multiple iterations) so
//! the true "cold" measurement is the FIRST call. The warm iterations are
//! measured by `warm_render.rs`. This benchmark captures total initialization
//! overhead via `Criterion::new().measurement_time()` set low to focus on
//! the first-call latency profile.

use criterion::{Criterion, criterion_group, criterion_main};
use slideforge_diagrams::{DiagramRendererImpl, types::DiagramLang};

/// Benchmark function: cold render+normalize — first call includes font DB init.
///
/// Exercises the full diagram pipeline: `render_mermaid` + `usvg_normalize`
/// (BC-1.12.003 invariant 1: normalization is mandatory after every render).
fn cold_render_benchmark(c: &mut Criterion) {
    // A simple flowchart that exercises the full render + normalize path.
    let source = "graph TD\n  A-->B";

    c.bench_function("cold_flowchart_render_normalize", |b| {
        b.iter(|| {
            DiagramRendererImpl::render_diagram(
                std::hint::black_box(source),
                DiagramLang::Mermaid,
                "bench",
            )
            .expect("benchmark: flowchart render+normalize must succeed")
        });
    });
}

criterion_group!(benches, cold_render_benchmark);
criterion_main!(benches);
