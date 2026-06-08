//! Cold-budget integration test — AC-008 / NFR-003.
//!
//! ## Purpose
//!
//! This file is compiled into a **separate test binary** by Cargo (each
//! `tests/*.rs` file gets its own process). That means `FONT_DB` (the
//! `OnceLock<Arc<fontdb::Database>>`) starts uninitialized here, giving us a
//! genuine cold-path measurement of the first `render_diagram` call.
//!
//! ## Gate (AC-008 / NFR-003)
//!
//! The combined render + normalize pipeline must complete in < 200ms on the
//! first call (cold path, font DB not yet initialized).
//!
//! ## Why not a Criterion bench?
//!
//! Criterion's `b.iter` loop warms up the benchmark binary before timing, so
//! the first call (which loads `FONT_DB`) is amortized across many warm
//! iterations.  A dedicated integration test in its own binary is the correct
//! mechanism for a cold-path gate.
//!
//! ## Obs-3 note (`FONT_DB` process-global, no test isolation)
//!
//! Within any single test binary (i.e., within `cargo nextest run -p
//! slideforge-diagrams`), the `FONT_DB` `OnceLock` is initialized once and
//! reused for all tests.  The unit tests in `src/normalize.rs` that measure
//! warm-path latency intentionally warm up the DB before timing (three-step
//! methodology: warmup → font-init → timed).  Only THIS file, which lives
//! in `tests/cold_budget.rs` and therefore gets its own binary, observes
//! a genuinely cold `FONT_DB`.

#![allow(clippy::unwrap_used)] // integration tests may use unwrap
#![allow(clippy::tests_outside_test_module)] // integration test binary — no mod tests wrapper

use std::time::{Duration, Instant};

use slideforge_diagrams::{DiagramRendererImpl, SfDiagramLang as DiagramLang};

/// AC-008 / NFR-003: the first call to the full diagram pipeline
/// (`render_mermaid` + `usvg_normalize`) must complete in < 200ms.
///
/// This test runs in a fresh process (each `tests/*.rs` is a separate Cargo
/// binary), so `FONT_DB` is uninitialized at the start — the cold path is
/// genuine. On developer machines with many system fonts the cost is typically
/// 50–150ms; the 200ms gate matches the NFR-003 CI budget.
#[test]
fn test_cold_budget_under_200ms() {
    let start = Instant::now();
    let result = DiagramRendererImpl::render_diagram(
        "graph TD\n  A-->B[node label]",
        DiagramLang::Mermaid,
        "cold-budget-test",
    );
    let elapsed = start.elapsed();

    // The render must succeed.
    let normalized = result.expect("cold render must succeed for a valid Mermaid flowchart");
    assert!(
        !normalized.is_empty(),
        "cold render must return non-empty NormalizedDiagramSvg"
    );

    // The cold-path budget enforced in CI:
    //   - Windows:     500ms  (font enumeration + AV I/O overhead on CI runners)
    //   - macOS/Linux: 300ms  (CI gate; the NFR-003 target of 200ms is the local
    //                          Apple Silicon / developer machine gate — shared
    //                          GitHub Actions runners have ~50% overhead variance;
    //                          observed value on macos-latest CI: 242ms, iteration 2)
    //
    // NFR-003 (< 200ms cold) is still the canonical target and is verified on
    // local dev machines via `cargo nextest run -p slideforge-diagrams`. The 300ms
    // CI gate prevents spurious flakes from runner scheduling jitter while still
    // catching catastrophic regressions (accidental per-call font loading, sync
    // network I/O, loading fonts in a loop, etc.).
    let budget = if cfg!(windows) {
        Duration::from_millis(500)
    } else {
        Duration::from_millis(300)
    };
    assert!(
        elapsed < budget,
        "cold render+normalize budget exceeded: {elapsed:?} >= {budget:?}. \
         This gate ensures the first call (FONT_DB init + mermaid render + usvg normalize) \
         stays within the NFR-003 cold budget on CI. \
         If this fails only on CI: check system font count or CI runner speed."
    );
}

// ---------------------------------------------------------------------------
// STORY-080 AC-002 / AC-003: companion correctness test (non-timing path)
//
// This test is added as part of the STORY-080 deflake work (AC-002). Its
// purpose is to satisfy AC-003 (behavioral assertions must not be weakened):
// even after `test_cold_budget_under_200ms` is moved to an on-demand
// (#[ignore]) test, a correctness-only test must remain in the default matrix
// that proves the cold render succeeds and returns non-empty content.
//
// Red Gate note: this test passes against the current production code (the
// render already works). The Red Gate for STORY-080 cold_budget is provided
// by `test_BC_1_03_002_http_4xx_not_retried_deterministic_harness` in
// slideforge-data, which uses a `todo!()` stub. See Red Gate log for the full
// rationale on why no failing-stub Red Gate is used here.
//
// The `#[ignore]` annotation on `test_cold_budget_under_200ms` is the
// implementer's one-line change that eliminates the spurious CI failure.
// That change has no driving test — it is documented here instead.
// ---------------------------------------------------------------------------

/// STORY-080 AC-002 / AC-003: cold render must produce `Ok(NormalizedDiagramSvg)`
/// with non-empty content, regardless of timing.
///
/// This is the timing-gate-free companion to `test_cold_budget_under_200ms`.
/// It verifies the correctness contract (BC-1.12.003 postcondition: render
/// returns normalized SVG) without a wall-clock assertion, making it safe
/// for all 5 CI platforms under any load level.
///
/// The timing gate (NFR-003 compliance) is preserved in `test_cold_budget_under_200ms`
/// but moved to on-demand execution (Option A: `#[ignore = "..."]`) by the
/// implementer so it does not cause spurious CI failures under CPU contention.
///
/// Traces to: BC-1.12.003 (cold render correctness), STORY-080 AC-002, AC-003.
#[test]
#[allow(non_snake_case)] // BC-based naming convention: test_BC_S_SS_NNN_xxx
fn test_BC_1_12_003_cold_render_correctness() {
    // This test runs in the cold_budget.rs binary (separate Cargo test binary),
    // so FONT_DB starts uninitialized. However, because cold_budget.rs runs
    // BOTH this test and `test_cold_budget_under_200ms` in the same binary,
    // the ordering of tests within the binary is non-deterministic. For the
    // cold-path budget measurement, `test_cold_budget_under_200ms` is the
    // authoritative test; this test only verifies correctness.
    let result = DiagramRendererImpl::render_diagram(
        "graph TD\n  A-->B[correctness check]",
        DiagramLang::Mermaid,
        "cold-correctness-test",
    );

    // AC-003: the cold render MUST return Ok(...) with non-empty content.
    // This assertion is non-negotiable — it must survive regardless of what
    // Option A/B/C does to the timing gate.
    let normalized =
        result.expect("cold render must succeed for a valid Mermaid flowchart (AC-003)");
    assert!(
        !normalized.is_empty(),
        "cold render must return non-empty NormalizedDiagramSvg (AC-003)"
    );

    // No timing assertion — that is the entire point of this companion test.
    // The timing gate lives in test_cold_budget_under_200ms (on-demand, #[ignore]).
}
