//! Criterion benchmark: cold diagram render time (font DB initialization).
//!
//! ## Purpose (AC-003 / NFR-003)
//!
//! Measures the total time for the FIRST call to `DiagramRendererImpl::render_diagram`
//! in a fresh process. The font database is initialized on the first call.
//!
//! ## Gate
//!
//! The benchmark must pass < 200ms on the CI Linux x86_64 runner.
//! Empirical from Spike S14: 124ms on M-series Mac.
//!
//! ## NOTE: This benchmark requires implementation to be complete.
//! During the Red Gate phase, this benchmark file exists as a stub.
//! The Criterion harness will not compile until `todo!()` stubs are removed.

fn main() {
    // Benchmark stub — Criterion harness requires implementation.
    // The implementer must add:
    //   criterion_group!(benches, cold_render_benchmark);
    //   criterion_main!(benches);
    // And implement the benchmark function after render_diagram is implemented.
    eprintln!("cold_render benchmark: stub — not yet implemented (Red Gate phase)");
}
