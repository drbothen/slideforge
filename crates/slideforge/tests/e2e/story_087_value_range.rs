//! STORY-087 — F-087-P1-001 Value-Range Red Gate: build()-level integration tests.
//!
//! These tests prove that value-range enforcement is REACHABLE from
//! `slideforge::build()`. They call `slideforge::build()` — NOT `lay_out()`
//! directly — so they exercise the real pipeline path through `build_inner`
//! including Stage 5 (Validator loop).
//!
//! ## Architecture rationale (from architect adjudication)
//!
//! `ProgressBarSlideType::lay_out()` and `WeightedCompositeSlideType::lay_out()`
//! both contain value-range checks, but `build_inner` calls `layout::run` which
//! calls `region_frames_for` — NOT `SlideType::lay_out()`. The checks in
//! `lay_out()` are dead code at build time (TD-VSDD-059 paper-fix).
//!
//! Option B (architect decision): add `ValueRangeValidator` to
//! `slideforge-validate`, register it in `slideforge::registry::register_bundled_plugins`
//! at Stage 5 alongside `LabelCheckValidator`. Remove the dead checks from
//! `lay_out()`.
//!
//! ## Red Gate discipline
//!
//! The `progress_bar` failing tests (value=101, value=-1) are the PRIMARY Red Gate
//! proof — they FAIL currently because no `ValueRangeValidator` is registered, so
//! `build()` returns `Ok` for out-of-range values instead of `Err(ValidationFailed)`.
//!
//! ## Traceability
//!
//! - BC-1.17.002 PC3: progress_bar value outside [0,100] → compile error E-VAL-011
//! - BC-1.17.003 inv 6: weighted_composite weight must be positive → E-VAL-011
//! - BC-1.17.003 inv 7: weighted_composite score must be in [0,100] → E-VAL-011
//! - Finding F-087-P1-001: value-range validation in lay_out() is dead code
//! - Architect adjudication: Option B — dedicated ValueRangeValidator Stage 5
//! - ADR-016 Decision 3: Validator surface registered in registry.rs

#![allow(clippy::unwrap_used)] // integration tests — panics are intentional failures
#![allow(clippy::doc_markdown)] // test comments reference identifiers like `E-VAL-011`

use crate::e2e::{BrandTmpDir, fixture_source};

// ─────────────────────────────────────────────────────────────────────────────
// progress_bar value-range build()-level tests
// These DO use DSL fixtures because `value` is a bare integer literal.
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.002 PC3 / F-087-P1-001 Red Gate:
/// `build()` with `progress_bar` slide containing `value 101` must return
/// `Err(BuildError::ValidationFailed)` with at least one diagnostic having
/// `code == "E-VAL-011"`.
///
/// ## Red Gate behavior (pre-implementation)
///
/// Currently no `ValueRangeValidator` is registered. `build()` returns `Ok`
/// (layout and export succeed; the out-of-range value is silently accepted).
/// This test asserts `Err(ValidationFailed)` → FAILS at Red Gate.
///
/// ## Post-implementation behavior
///
/// After `ValueRangeValidator` is registered at Stage 5, `build()` emits
/// `E-VAL-011` with message containing "got 101" and returns `ValidationFailed`.
/// This test passes.
///
/// Traceability: BC-1.17.002 PC3; adjudication §5.3B row 1.
#[test]
fn test_BC_1_17_002_build_progress_bar_value_101_is_validation_failed() {
    let brand = BrandTmpDir::new("s087_pb_101");
    let source = fixture_source("story-087-progress-bar-101.sf");
    let opts = brand.build_options("pptx", true); // strict=true: ValidationFailed fires

    let result = slideforge::build(&source, &opts);

    // Primary assertion: must be Err(ValidationFailed).
    // Currently FAILS at Red Gate (no validator → build returns Ok).
    assert!(
        matches!(
            result,
            Err(slideforge::error::BuildError::ValidationFailed { .. })
        ),
        "F-087-P1-001 Red Gate: progress_bar value=101 must produce \
         BuildError::ValidationFailed; currently no ValueRangeValidator is \
         registered so build() returns Ok. \
         Got: {result:?}"
    );

    // Secondary assertion: the error code must be exactly "E-VAL-011".
    // This is load-bearing — "any Err" is not sufficient (TD-VSDD-059).
    if let Err(slideforge::error::BuildError::ValidationFailed { diagnostics, count }) = result {
        assert!(
            count > 0,
            "ValidationFailed.count must be > 0 for value=101 fixture"
        );
        let has_e_val_011 = diagnostics.iter().any(|d| d.code.as_ref() == "E-VAL-011");
        assert!(
            has_e_val_011,
            "ValidationFailed.diagnostics must contain at least one 'E-VAL-011' diagnostic \
             (value-range violation). \
             Got codes: {:?}",
            diagnostics
                .iter()
                .map(|d| d.code.as_ref())
                .collect::<Vec<_>>()
        );

        // Also verify the message mentions the actual value.
        let mentions_101 = diagnostics
            .iter()
            .any(|d| d.code.as_ref() == "E-VAL-011" && d.message.contains("101"));
        assert!(
            mentions_101,
            "E-VAL-011 diagnostic message must mention the offending value 101; \
             got messages: {:?}",
            diagnostics
                .iter()
                .filter(|d| d.code.as_ref() == "E-VAL-011")
                .map(|d| d.message.as_ref())
                .collect::<Vec<_>>()
        );
    }
}

/// BC-1.17.002 PC3 / F-087-P1-001:
/// `build()` with `progress_bar` slide containing `value -1` (below zero) must return
/// `Err(BuildError::ValidationFailed)` with at least one `"E-VAL-011"` diagnostic.
///
/// ## Red Gate behavior (pre-implementation)
///
/// Currently no `ValueRangeValidator` is registered. `build()` returns `Ok`.
/// This test asserts `Err(ValidationFailed)` → FAILS at Red Gate.
///
/// Traceability: BC-1.17.002 PC3; adjudication §5.3B row 2.
#[test]
fn test_BC_1_17_002_build_progress_bar_value_neg1_is_validation_failed() {
    let brand = BrandTmpDir::new("s087_pb_neg1");
    let source = fixture_source("story-087-progress-bar-neg1.sf");
    let opts = brand.build_options("pptx", true);

    let result = slideforge::build(&source, &opts);

    assert!(
        matches!(
            result,
            Err(slideforge::error::BuildError::ValidationFailed { .. })
        ),
        "F-087-P1-001 Red Gate: progress_bar value=-1 must produce \
         BuildError::ValidationFailed; currently no ValueRangeValidator is \
         registered so build() returns Ok. \
         Got: {result:?}"
    );

    if let Err(slideforge::error::BuildError::ValidationFailed { diagnostics, .. }) = result {
        let has_e_val_011 = diagnostics.iter().any(|d| d.code.as_ref() == "E-VAL-011");
        assert!(
            has_e_val_011,
            "ValidationFailed.diagnostics must contain 'E-VAL-011' for value=-1; \
             got codes: {:?}",
            diagnostics
                .iter()
                .map(|d| d.code.as_ref())
                .collect::<Vec<_>>()
        );
        let mentions_neg1 = diagnostics
            .iter()
            .any(|d| d.code.as_ref() == "E-VAL-011" && d.message.contains("-1"));
        assert!(
            mentions_neg1,
            "E-VAL-011 message must mention the offending value -1; \
             got messages: {:?}",
            diagnostics
                .iter()
                .filter(|d| d.code.as_ref() == "E-VAL-011")
                .map(|d| d.message.as_ref())
                .collect::<Vec<_>>()
        );
    }
}

/// BC-1.17.002 PC4 / F-087-P1-001 regression guard:
/// `build()` with `progress_bar` slide containing `value 50` (valid, mid-range)
/// must return `Ok(BuildOutput)` — no E-VAL-011 emitted.
///
/// ## Red Gate behavior
///
/// This test is a POSITIVE (regression guard) test. It currently passes because
/// build() returns Ok for any progress_bar value. After implementation, it still
/// passes because value=50 is valid. The test guards against false positives in
/// the `ValueRangeValidator` implementation.
///
/// Traceability: BC-1.17.002 PC4; adjudication §5.3B row 3.
#[test]
fn test_BC_1_17_002_build_progress_bar_value_50_is_ok() {
    let brand = BrandTmpDir::new("s087_pb_50");
    let source = fixture_source("story-087-progress-bar-50.sf");
    let opts = brand.build_options("pptx", true); // strict=true: any error would surface

    let result = slideforge::build(&source, &opts);

    assert!(
        result.is_ok(),
        "progress_bar with value=50 (valid) must return Ok; \
         if ValidationFailed is returned, the ValueRangeValidator has a false-positive bug. \
         Got: {result:?}"
    );
}

/// BC-1.17.002 boundary: `value=0` (lower boundary) must not emit E-VAL-011.
///
/// Traceability: BC-1.17.002 PC3 boundary; adjudication §5.3A row 2.
#[test]
fn test_BC_1_17_002_build_progress_bar_value_0_boundary_is_ok() {
    let brand = BrandTmpDir::new("s087_pb_0");
    // Inline DSL: value=0 is the lower boundary of [0, 100].
    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "slide progress_bar:\n",
        "  title \"Start\"\n",
        "  label \"Not started\"\n",
        "  value 0\n",
    );
    let opts = brand.build_options("pptx", true);
    let result = slideforge::build(source, &opts);
    assert!(
        result.is_ok(),
        "progress_bar with value=0 (lower boundary) must return Ok; got: {result:?}"
    );
}

/// BC-1.17.002 boundary: `value=100` (upper boundary) must not emit E-VAL-011.
///
/// Traceability: BC-1.17.002 PC3 boundary; adjudication §5.3A row 3.
#[test]
fn test_BC_1_17_002_build_progress_bar_value_100_boundary_is_ok() {
    let brand = BrandTmpDir::new("s087_pb_100");
    // Inline DSL: value=100 is the upper boundary of [0, 100].
    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "slide progress_bar:\n",
        "  title \"Done\"\n",
        "  label \"Complete\"\n",
        "  value 100\n",
    );
    let opts = brand.build_options("pptx", true);
    let result = slideforge::build(source, &opts);
    assert!(
        result.is_ok(),
        "progress_bar with value=100 (upper boundary) must return Ok; got: {result:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// weighted_composite value-range build()-level tests
//
// DSL BLOCKER: The current slideforge DSL parser does not support list-of-map
// literals (STORY-088 — DSL list-literal parser). The `components` field of
// `weighted_composite` must be a `Value::List(Vec<Value::Map(...)>)`, which
// requires syntax like:
//
//   @var comps = [{name: "A", weight: 0.4, score: 101, label: "bad"}]
//
// This syntax is not yet supported. Therefore the build()-level tests for
// `weighted_composite` score/weight violations are marked `#[ignore]` with
// this blocker annotation.
//
// The PRIMARY load-bearing proof gate for the wiring fix is the `progress_bar`
// tests above (adjudication §8: "at least 2 of the 5 build()-level integration
// tests...specifically `test_build_progress_bar_value_101_is_validation_failed`
// and `test_build_weighted_composite_score_101_is_validation_failed`"). Since
// the latter requires DSL blocker resolution, the `progress_bar` pair is the
// minimal proof gate for STORY-087 delivery. The `weighted_composite` tests
// will be un-ignored when STORY-088 ships.
//
// For unit-level proof of weighted_composite score/weight range enforcement,
// see crates/slideforge-validate/src/value_range.rs (unit tests that call
// `ValueRangeValidator.validate(&deck, &opts)` directly with programmatic
// Deck construction — these do NOT require DSL list-literal support).
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.003 inv 7 / F-087-P1-001:
/// `build()` with `weighted_composite` slide containing a component with
/// `score: 101` must return `Err(BuildError::ValidationFailed)` with `E-VAL-011`.
///
/// ## IGNORED: DSL blocker STORY-088
///
/// The `components` field must be `Value::List(Vec<Value::Map(...)>)`. The DSL
/// parser does not yet support list-of-map literals (STORY-088). Until DSL
/// list-literal support ships, this test cannot express the fixture via
/// `slideforge::build()`.
///
/// The unit-level equivalent is:
///   `test_BC_1_17_003_score_101_error` in
///   `crates/slideforge-validate/src/value_range.rs`.
///
/// Un-ignore when STORY-088 (DSL list-literal parser) ships.
///
/// Traceability: BC-1.17.003 inv 7; adjudication §5.3B row 4.
#[test]
#[ignore = "DSL blocker: list-of-map literals require STORY-088 (DSL list-literal parser)"]
fn test_BC_1_17_003_build_weighted_composite_score_101_is_validation_failed() {
    let brand = BrandTmpDir::new("s087_wc_score101");
    // TODO(STORY-088): replace with a fixture file once list-of-map DSL syntax
    // is supported. The fixture must produce a deck with a `weighted_composite`
    // slide containing a component map with `score: 101`.
    //
    // Expected DSL (not yet parseable):
    //   @var comps = [{name: "Quality", weight: 0.4, score: 101, label: "Over"}]
    //   slide weighted_composite:
    //     title "Vendor A"
    //     label "Overall: Bad"
    //     components: comps
    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "slide weighted_composite:\n",
        "  title \"Vendor A Scorecard\"\n",
        "  label \"Overall: Bad\"\n",
        // components field omitted — DSL cannot express list-of-map yet
        // When STORY-088 ships, add: components with score 101
    );
    let opts = brand.build_options("pptx", true);

    let result = slideforge::build(source, &opts);

    // With proper list-of-map DSL support, this assertion should hold:
    assert!(
        matches!(
            result,
            Err(slideforge::error::BuildError::ValidationFailed { .. })
        ),
        "weighted_composite with component score=101 must produce ValidationFailed; \
         got: {result:?}"
    );
    if let Err(slideforge::error::BuildError::ValidationFailed { diagnostics, .. }) = result {
        let has_e_val_011 = diagnostics.iter().any(|d| d.code.as_ref() == "E-VAL-011");
        assert!(
            has_e_val_011,
            "ValidationFailed must contain E-VAL-011 for score=101; \
             got codes: {:?}",
            diagnostics
                .iter()
                .map(|d| d.code.as_ref())
                .collect::<Vec<_>>()
        );
    }
}

/// BC-1.17.003 inv 6 / F-087-P1-001:
/// `build()` with `weighted_composite` slide containing a component with
/// `weight: 0` (non-positive) must return `Err(BuildError::ValidationFailed)`
/// with `E-VAL-011`.
///
/// ## IGNORED: DSL blocker STORY-088
///
/// Same blocker as `test_BC_1_17_003_build_weighted_composite_score_101_is_validation_failed`.
/// Un-ignore when STORY-088 ships.
///
/// Traceability: BC-1.17.003 inv 6; adjudication §5.3B row 5.
#[test]
#[ignore = "DSL blocker: list-of-map literals require STORY-088 (DSL list-literal parser)"]
fn test_BC_1_17_003_build_weighted_composite_weight_zero_is_validation_failed() {
    let brand = BrandTmpDir::new("s087_wc_wt0");
    // TODO(STORY-088): replace with fixture once DSL supports list-of-map.
    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "slide weighted_composite:\n",
        "  title \"Vendor A Scorecard\"\n",
        "  label \"Overall: Bad\"\n",
        // components field omitted — DSL cannot express list-of-map yet
    );
    let opts = brand.build_options("pptx", true);

    let result = slideforge::build(source, &opts);

    assert!(
        matches!(
            result,
            Err(slideforge::error::BuildError::ValidationFailed { .. })
        ),
        "weighted_composite with component weight=0 must produce ValidationFailed; \
         got: {result:?}"
    );
    if let Err(slideforge::error::BuildError::ValidationFailed { diagnostics, .. }) = result {
        let has_e_val_011 = diagnostics.iter().any(|d| d.code.as_ref() == "E-VAL-011");
        assert!(
            has_e_val_011,
            "ValidationFailed must contain E-VAL-011 for weight=0; \
             got codes: {:?}",
            diagnostics
                .iter()
                .map(|d| d.code.as_ref())
                .collect::<Vec<_>>()
        );
    }
}
