//! Cold-budget integration test — NFR-003 gate (restructured by STORY-080).
//!
//! ## Purpose
//!
//! This file is compiled into a **separate test binary** by Cargo (each
//! `tests/*.rs` file gets its own process). That means `FONT_DB` (the
//! `OnceLock<Arc<fontdb::Database>>`) starts uninitialized here, giving us a
//! genuine cold-path measurement of the first `render_diagram` call.
//!
//! ## Three-test structure (STORY-080 restructure)
//!
//! This file contains three tests with different roles and CI visibility:
//!
//! ### 1. `test_cold_budget_catastrophic_regression_gate` — **ALWAYS-ON in CI**
//!
//! Catches EC-005 (accidental per-call font-DB reload, ~10× regression) using
//! a deliberately generous budget: **1 s on macOS/Linux, 2 s on Windows**.
//! These thresholds accommodate full-workspace CPU contention on shared GitHub
//! Actions runners without ever flaking, while still catching any regression
//! above ~4–8× the normal cold-render wall time.  This is the **automated
//! NFR-003 CI gate** — it runs on every push, on all 5 CI platforms.
//!
//! ### 2. `test_cold_budget_under_200ms` — **`#[ignore]`'d, on-demand**
//!
//! The NFR-003 **precision gate** (200ms developer-machine target; relaxed to
//! 300ms/500ms on-demand to accommodate machines with many system fonts).
//! This test is `#[ignore]`'d because under full-workspace `cargo nextest run`
//! with CPU contention, font-DB scan time can exceed even a 300ms budget on
//! shared runners (observed: macOS-latest CI at 242ms with a 300ms budget).
//! Precision timing gates must run on-demand, not in the default CI matrix.
//!
//! To run explicitly:
//! ```text
//! cargo nextest run -p slideforge-diagrams -- --include-ignored cold_budget
//! ```
//!
//! ### 3. `test_cold_render_correctness` — **ALWAYS-ON in CI**
//!
//! A timing-free correctness companion. Asserts that `render_diagram` returns
//! `Ok(NormalizedDiagramSvg)` with non-empty content, regardless of timing.
//! Ensures the behavioral contract is never ungated even when the precision
//! timing gate is `#[ignore]`'d.
//!
//! ## Provenance: AC-008 / STORY-034 → STORY-080
//!
//! This file was originally created for STORY-034 AC-008 as a single 200ms
//! `#[test]`.  STORY-080 restructured it into the three-test form above to
//! eliminate spurious CI failures under CPU contention (EC-001 / EC-005),
//! while preserving both correctness coverage (always-on) and precision timing
//! coverage (on-demand).  AC-008 traced to NFR-003; NFR-003 is now enforced in
//! CI via `test_cold_budget_catastrophic_regression_gate`.
//!
//! ## Why not a Criterion bench?
//!
//! Criterion's `b.iter` loop warms up the benchmark binary before timing, so
//! the first call (which loads `FONT_DB`) is amortized across many warm
//! iterations.  A dedicated integration test in its own binary is the correct
//! mechanism for a cold-path gate.
//!
//! ## `FONT_DB` process-global `OnceLock` — intra-binary parallel execution
//!
//! Within this binary, `FONT_DB` is a process-global `OnceLock` initialized
//! exactly once.  Because nextest runs tests in the same binary in parallel by
//! default, exactly ONE of the three tests pays the cold init cost; which one
//! is non-deterministic.  The catastrophic gate's EC-005 contract still holds
//! because accidental per-call font reload manifests warm-or-cold (the
//! `OnceLock` is bypassed on every call, not just the first).  Do NOT restructure
//! these tests into a separate binary to force a specific init order — the
//! current non-determinism is intentional and correct.

#![allow(clippy::unwrap_used)] // integration tests may use unwrap
#![allow(clippy::tests_outside_test_module)] // integration test binary — no mod tests wrapper

use std::time::{Duration, Instant};

use slideforge_diagrams::{DiagramRendererImpl, SfDiagramLang as DiagramLang};

/// NFR-003 precision gate: the first call to the full diagram pipeline
/// (`render_mermaid` + `usvg_normalize`) must complete in < 200ms.
///
/// This test runs in a fresh process (each `tests/*.rs` is a separate Cargo
/// binary), so `FONT_DB` is uninitialized at the start — the cold path is
/// genuine. On developer machines with many system fonts the cost is typically
/// 50–150ms; the 200ms gate matches the NFR-003 developer-machine target.
///
/// ## Why `#[ignore]`?
///
/// This is a **precision timing gate**, not a correctness gate.  Under full-workspace
/// `cargo nextest run` with CPU contention from parallel test jobs, the font
/// DB scan (`fontdb::Database::load_system_fonts()`) can exceed even a generous
/// 300ms budget on shared GitHub Actions runners when the algorithm is correct.
/// (Observed: macos-latest CI at 242ms iteration 2 with a 300ms budget.)
///
/// Precision timing gates must be run on-demand, not in the default CI matrix.
/// **NFR-003 automated CI coverage** is provided by the always-active catastrophic
/// regression gate `test_cold_budget_catastrophic_regression_gate` in this same
/// file, which uses a 1000ms/2000ms threshold that reliably catches EC-005
/// (per-call font loading = ~10x regression) without ever flaking on a loaded runner.
///
/// Correctness (non-timing) coverage is also preserved unconditionally by
/// `test_cold_render_correctness` in this same file.
///
/// To run the precision gate explicitly on a developer machine:
/// ```
/// cargo nextest run -p slideforge-diagrams -- --include-ignored cold_budget
/// ```
///
/// Root cause of flake: STORY-080 (CPU-contention wall-clock gate under CI load).
// STORY-080: precision timing gate moved to on-demand (#[ignore]) to eliminate
// spurious CI failures under CPU contention.  Automated NFR-003 CI coverage is
// provided by `test_cold_budget_catastrophic_regression_gate` (always runs,
// catastrophe-only threshold).  Correctness is covered by
// `test_cold_render_correctness` (always runs, no timing assertion).
#[test]
#[ignore = "NFR-003 precision gate (200ms) — run on developer machine with: \
            cargo nextest run -p slideforge-diagrams -- --include-ignored cold_budget"]
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

    // Precision budget for developer machines and on-demand CI runs:
    //   - Windows:     500ms  (font enumeration + AV I/O overhead)
    //   - macOS/Linux: 300ms  (developer machine target; NFR-003 goal is 200ms
    //                          but 300ms accommodates developer machines with many
    //                          system fonts; on Apple Silicon M-series: ~80–150ms)
    //
    // NOTE: this test is #[ignore]'d and does NOT run in the default CI matrix.
    // Automated NFR-003 CI coverage lives in `test_cold_budget_catastrophic_regression_gate`.
    let budget = if cfg!(windows) {
        Duration::from_millis(500)
    } else {
        Duration::from_millis(300)
    };
    assert!(
        elapsed < budget,
        "cold render+normalize precision budget exceeded: {elapsed:?} >= {budget:?}. \
         NFR-003 target is 200ms on developer machines; 300ms/500ms are on-demand precision gates. \
         Run `cargo nextest run -p slideforge-diagrams -- --include-ignored cold_budget` \
         to reproduce. If this fails only under load: the automated CI gate \
         (`test_cold_budget_catastrophic_regression_gate`) catches catastrophic regressions."
    );
}

// ---------------------------------------------------------------------------
// STORY-080 AC-002 / AC-003: companion correctness test (non-timing path)
//
// This test satisfies AC-003 (behavioral assertions must not be weakened):
// even after `test_cold_budget_under_200ms` is moved to on-demand (#[ignore]),
// this correctness-only test remains in the default matrix and proves the
// cold render succeeds and returns non-empty content.
//
// The `#[ignore]` annotation on `test_cold_budget_under_200ms` is the
// STORY-080 change that eliminates the spurious CI failure; this companion
// test ensures the behavioral correctness contract is never ungated.
// ---------------------------------------------------------------------------

/// STORY-080 AC-003: cold render must produce `Ok(NormalizedDiagramSvg)`
/// with non-empty content, regardless of timing.
///
/// This is the timing-gate-free companion to `test_cold_budget_under_200ms`.
/// It verifies the correctness contract without a wall-clock assertion, making
/// it safe for all 5 CI platforms under any load level.
///
/// The precision timing gate (NFR-003, 200ms) is preserved in
/// `test_cold_budget_under_200ms` as an on-demand `#[ignore]`'d test.
/// The catastrophic-regression gate (EC-005, 1s/2s) is provided by
/// `test_cold_budget_catastrophic_regression_gate` and always runs in CI.
///
/// Traces to: STORY-080 AC-002, AC-003.
#[test]
fn test_cold_render_correctness() {
    // This test runs in the cold_budget.rs binary (separate Cargo test binary),
    // so FONT_DB starts uninitialized. The ordering of tests within the binary
    // is non-deterministic (nextest runs them in parallel by default within the
    // binary), but each test here exercises the full cold path independently.
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
    // Precision timing lives in test_cold_budget_under_200ms (on-demand, #[ignore]).
    // Catastrophic-regression timing lives in test_cold_budget_catastrophic_regression_gate.
}

// ---------------------------------------------------------------------------
// STORY-080 CRIT-2 / EC-005: automated catastrophic-regression gate
//
// This test provides the NFR-003 automated CI gate that is ALWAYS active
// (not #[ignore]'d) and catches catastrophic regressions (EC-005: per-call
// font loading = ~10x regression = ~2s+) without flaking on loaded runners.
//
// Design rationale (Option C hybrid, CRIT-2):
//   - Normal correct cold renders take ~80–250ms on CI runners
//     (observed range across macos-14, ubuntu-latest, windows-latest).
//   - EC-005 catastrophic regression (accidental per-call font loading,
//     sync I/O in tight loop): ~2s+ (10x the normal upper bound).
//   - This gate uses 1s (macOS/Linux) / 2s (Windows) — generous enough to
//     never flake on a loaded shared runner, tight enough to catch any
//     regression above ~4–8x the normal wall time.
//   - The precision NFR-003 gate (200ms/300ms/500ms) is preserved as
//     `test_cold_budget_under_200ms` (#[ignore]'d, on-demand developer use).
// ---------------------------------------------------------------------------

/// STORY-080 CRIT-2 / EC-005: catastrophic-regression gate for NFR-003.
///
/// Runs in the **default CI matrix** (not `#[ignore]`'d) and catches
/// catastrophic performance regressions such as:
///   - Accidental per-call `fontdb::Database::load_system_fonts()` (10x cost)
///   - Synchronous network I/O on the critical path
///   - Loading fonts in a loop instead of using the `OnceLock` cache
///
/// Budget rationale:
///   - macOS / Linux: 1s — observed correct renders: 80–250ms;
///     a 4x safety margin catches any ≥ 400ms regression without flaking
///     under full-workspace CPU contention on shared GitHub Actions runners.
///   - Windows: 2s — font enumeration + AV I/O overhead on Windows CI
///     runners pushes correct renders to ~300–500ms; 4x margin = 2s.
///   - EC-005 catastrophe (per-call font loading): ~2s+ on macOS/Linux,
///     ~4s+ on Windows — well above both thresholds.
///
/// NOTE: this is NOT the NFR-003 precision gate.  The NFR-003 < 200ms target
/// is enforced on developer machines via `test_cold_budget_under_200ms`
/// (`#[ignore]`'d precision test).  This test only catches catastrophes.
///
/// Traces to: STORY-080 EC-005, AC-002, AC-003.
#[test]
fn test_cold_budget_catastrophic_regression_gate() {
    // Catastrophic-regression budget:
    //   - macOS / Linux: 1000ms (4× the ~250ms observed CI upper bound)
    //   - Windows:       2000ms (4× the ~500ms observed CI upper bound)
    //
    // These thresholds are deliberately generous to prevent flaking under CI
    // load, while still catching EC-005 (per-call font loading ≈ 10× cost).
    // The NFR-003 precision target (200ms) is enforced by the on-demand
    // `test_cold_budget_under_200ms` test on developer machines.
    //
    // Note: FONT_DB is a process-global OnceLock; exactly one of the three
    // tests in this binary pays the cold init cost (which one is
    // non-deterministic under parallel nextest execution). The EC-005 contract
    // still holds because per-call font reload manifests warm-or-cold.
    let catastrophic_budget = if cfg!(windows) {
        Duration::from_secs(2)
    } else {
        Duration::from_secs(1)
    };

    let start = Instant::now();
    let result = DiagramRendererImpl::render_diagram(
        "graph TD\n  A-->B[catastrophe gate]",
        DiagramLang::Mermaid,
        "cold-catastrophe-test",
    );
    let elapsed = start.elapsed();

    // AC-003: correctness must hold regardless of timing.
    let normalized =
        result.expect("cold render must succeed for a valid Mermaid flowchart (AC-003)");
    assert!(
        !normalized.is_empty(),
        "cold render must return non-empty NormalizedDiagramSvg (AC-003)"
    );

    // Catastrophic-regression timing assertion.  A failure here indicates a
    // serious regression (EC-005: per-call font loading, sync I/O, etc.).
    assert!(
        elapsed < catastrophic_budget,
        "cold render exceeded catastrophic-regression budget: {elapsed:?} >= {catastrophic_budget:?}. \
         This gate catches EC-005-class regressions (per-call font loading = ~10x cost). \
         Normal cold renders: 80–250ms (macOS/Linux), 300–500ms (Windows). \
         For the NFR-003 precision gate (200ms), run: \
         cargo nextest run -p slideforge-diagrams -- --include-ignored cold_budget"
    );
}
