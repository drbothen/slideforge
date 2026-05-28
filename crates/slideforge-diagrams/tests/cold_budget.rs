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
//! ## Obs-3 note (FONT_DB process-global, no test isolation)
//!
//! Within any single test binary (i.e., within `cargo nextest run -p
//! slideforge-diagrams`), the `FONT_DB` OnceLock is initialized once and
//! reused for all tests.  The unit tests in `src/normalize.rs` that measure
//! warm-path latency intentionally warm up the DB before timing (three-step
//! methodology: warmup → font-init → timed).  Only THIS file, which lives
//! in `tests/cold_budget.rs` and therefore gets its own binary, observes
//! a genuinely cold FONT_DB.

#![allow(clippy::unwrap_used)] // integration tests may use unwrap

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

    // The cold-path budget is < 200ms on macOS/Linux (AC-008 / NFR-003).
    // On Windows, font enumeration is slower and the antivirus overhead on
    // file I/O can push cold font-DB initialization above 200ms on CI runners.
    // We apply a 500ms ceiling for Windows to avoid spurious flakes while still
    // catching catastrophic regressions (e.g., accidentally synchronous network
    // calls or loading fonts in a loop).
    let budget = if cfg!(windows) {
        Duration::from_millis(500)
    } else {
        Duration::from_millis(200)
    };
    assert!(
        elapsed < budget,
        "cold render+normalize budget exceeded: {:?} >= {:?}. \
         This gate ensures the first call (FONT_DB init + mermaid render + usvg normalize) \
         stays within the NFR-003 cold budget on CI. \
         If this fails only on CI: check system font count or CI runner speed.",
        elapsed,
        budget
    );
}
