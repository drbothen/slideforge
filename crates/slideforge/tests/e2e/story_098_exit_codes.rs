//! STORY-098 exit-code end-to-end tests — F-098-P1-004.
//!
//! ## Acceptance Criteria covered
//!
//! - **AC-001** (body strict): `body` on `content` slide in strict mode →
//!   `Err(BuildError::ValidationFailed)` with W-VAL-103 promoted to Error.
//! - **AC-002** (body warn-only): `body` on `content` slide in warn-only mode →
//!   `Ok`, W-VAL-103 as non-blocking Warning, body NOT threaded into output.
//! - **AC-004** (empty-chart strict): chart slide with no `data:` field in strict
//!   mode → `Err(BuildError::ValidationFailed)` with E-LAY-003.
//! - **AC-005** (empty-chart warn-only): chart slide with no `data:` field in
//!   warn-only mode → `Ok` (E-LAY-003 non-blocking); build produces output.
//!
//! ## Pattern
//!
//! These tests follow the STORY-089 / STORY-094 E-LAY-008 pattern:
//! - Inline DSL source (no fixture file needed for simple cases)
//! - `BrandTmpDir::new(label)` for brand setup
//! - `build_options("pptx", strict)` for strict/warn-only toggle
//! - Assert `matches!(result, Ok(_))` or `matches!(result, Err(ValidationFailed { .. }))`
//! - Verify diagnostic codes in the failure payload
//!
//! ## Traceability
//!
//! F-098-P1-004 (adversary pass-1 finding); BC-3.03.002 v1.2 Invariant 4
//! (content-drop keys → Error in strict mode); BC-1.11.002 v1.2 (E-LAY-003
//! for missing/empty chart data); STORY-098 AC-001, AC-002, AC-004, AC-005.

#![allow(clippy::unwrap_used)] // test assertions — panics are intentional

use crate::e2e::BrandTmpDir;

// ── AC-001: body on non-supporting slide — strict mode → exit 2 ──────────────

/// AC-001 / BC-3.03.002 v1.2 Invariant 4: `body` on a slide type that does NOT
/// declare it must return `Err(BuildError::ValidationFailed)` in strict mode.
///
/// `body` is a `CONTENT_DROP_KEY` — when it appears on a slide type that does not
/// declare it, `FieldSchemaValidator` emits W-VAL-103 promoted to Error severity.
/// In strict mode, any Error diagnostic → `Err(ValidationFailed)`.
///
/// Uses `chart` slide type: `chart` has `chart_type`, `data`, `alt`, common fields —
/// but NOT `body`. Chart slides also need `data` to avoid E-LAY-003, so this test
/// provides `data` to isolate the body-on-unsupporting-type signal.
///
/// Note: `content` slides now DECLARE `body` as an optional field (STORY-098
/// F-098-P1-002 resolution), so `body` on `content` is valid.
///
/// Traceability: F-098-P1-004; BC-3.03.002 Invariant 4; STORY-098 AC-001.
#[test]
fn test_f098_p1_004_ac001_body_on_unsupporting_type_strict_exits_2() {
    let brand = BrandTmpDir::new("s098_body_chart_strict");

    // `chart` slide with `body` field: body is an unknown field for chart type.
    // data is provided to avoid E-LAY-003 co-firing (isolate W-VAL-103 signal).
    // alt is provided to avoid E-A11-001 co-firing.
    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "\n",
        "slide chart:\n",
        "  title \"Revenue Chart\"\n",
        "  chart_type \"bar\"\n",
        "  data [\"Q1: 100\", \"Q2: 120\"]\n",
        "  alt \"Bar chart\"\n",
        "  body \"This body field is schema-invalid on chart slides.\"\n",
    );
    let opts = brand.build_options("pptx", true); // strict=true

    let result = slideforge::build(source, &opts);

    assert!(
        matches!(
            result,
            Err(slideforge::error::BuildError::ValidationFailed { .. })
        ),
        "AC-001 / F-098-P1-004: `body` on `chart` slide in strict mode must return \
         Err(ValidationFailed) — W-VAL-103 is promoted to Error for content-drop keys \
         (BC-3.03.002 Invariant 4). Got: {result:?}"
    );

    let Err(slideforge::error::BuildError::ValidationFailed {
        ref diagnostics, ..
    }) = result
    else {
        unreachable!("matched Err above")
    };

    // W-VAL-103 must be present with Error severity.
    let w_val_103: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_ref() == "W-VAL-103" && d.message.contains("'body'"))
        .collect();
    assert!(
        !w_val_103.is_empty(),
        "AC-001: ValidationFailed must carry W-VAL-103 for 'body' on 'chart' slide. \
         Got codes: {:?}",
        diagnostics
            .iter()
            .map(|d| d.code.as_ref())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        w_val_103[0].severity,
        slideforge_plugin_api::DiagnosticSeverity::Error,
        "AC-001: W-VAL-103 for content-drop key 'body' must be Error severity \
         (BC-3.03.002 Invariant 4). Got: {:?}",
        w_val_103[0].severity
    );
}

// ── AC-002: body on non-supporting slide — warn-only mode → exit 0 ───────────

/// AC-002 / BC-3.03.002 v1.2 Route B: `body` on a slide type that does NOT
/// declare it MUST return `Ok(_)` in warn-only mode — W-VAL-103 is non-blocking.
/// Body must NOT be threaded into the output slide blocks.
///
/// Note: `content` slides now declare `body` as an optional field, so this test
/// uses `chart` slides (which do NOT support `body`).
///
/// Traceability: F-098-P1-004; BC-3.03.002 Route B; STORY-098 AC-002.
#[test]
fn test_f098_p1_004_ac002_body_on_unsupporting_type_warn_only_exits_0() {
    let brand = BrandTmpDir::new("s098_body_chart_warn");

    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "\n",
        "slide chart:\n",
        "  title \"Revenue Chart\"\n",
        "  chart_type \"bar\"\n",
        "  data [\"Q1: 100\", \"Q2: 120\"]\n",
        "  alt \"Bar chart\"\n",
        "  body \"This body field is schema-invalid on chart slides.\"\n",
    );
    let opts = brand.build_options("pptx", false); // strict=false → warn-only

    let result = slideforge::build(source, &opts);

    assert!(
        result.is_ok(),
        "AC-002 / F-098-P1-004: `body` on `chart` slide in warn-only mode must return \
         Ok — W-VAL-103 is non-blocking (BC-3.03.002 Route B). Got: {result:?}"
    );
}

// ── AC-004: empty chart — strict mode → exit 2 ───────────────────────────────

/// AC-004 / BC-1.11.002 v1.2 EC-005: chart slide with no `data:` field in strict
/// mode MUST return `Err(BuildError::ValidationFailed)` with E-LAY-003.
///
/// `ChartEmptyDataValidator` catches missing data at Stage 5.
/// E-LAY-003 is `broken`/exit-2 in strict mode (error-taxonomy v2.31).
///
/// Traceability: F-098-P1-004; BC-1.11.002 v1.2 EC-005; STORY-098 AC-004;
///               error-taxonomy v2.31 E-LAY-003.
#[test]
fn test_f098_p1_004_ac004_chart_no_data_strict_exits_2() {
    let brand = BrandTmpDir::new("s098_chart_no_data_strict");

    // Chart slide with no `data:` field — triggers E-LAY-003.
    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "\n",
        "slide chart:\n",
        "  title \"Revenue\"\n",
        "  chart_type \"bar\"\n",
        "  alt \"Bar chart for revenue\"\n",
    );
    let opts = brand.build_options("pptx", true); // strict=true

    let result = slideforge::build(source, &opts);

    assert!(
        matches!(
            result,
            Err(slideforge::error::BuildError::ValidationFailed { .. })
        ),
        "AC-004 / F-098-P1-004: chart with no data: field in strict mode must return \
         Err(ValidationFailed) — E-LAY-003 is broken/exit-2 per error-taxonomy v2.31. \
         Got: {result:?}"
    );

    let Err(slideforge::error::BuildError::ValidationFailed {
        ref diagnostics, ..
    }) = result
    else {
        unreachable!("matched Err above")
    };

    let e_lay_003: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_ref() == "E-LAY-003")
        .collect();
    assert!(
        !e_lay_003.is_empty(),
        "AC-004: ValidationFailed must carry E-LAY-003 for chart with no data: field. \
         Got codes: {:?}",
        diagnostics
            .iter()
            .map(|d| d.code.as_ref())
            .collect::<Vec<_>>()
    );
    assert!(
        e_lay_003[0]
            .message
            .contains("[E-LAY-003] Chart data is empty"),
        "AC-004: E-LAY-003 message must have [E-LAY-003] self-prefix per error-taxonomy \
         v2.31. Got: {}",
        e_lay_003[0].message
    );
}

// ── AC-005: empty chart — warn-only mode → exit 0 ────────────────────────────

/// AC-005 / BC-1.11.002 v1.2 PC-007: chart slide with no `data:` field in warn-only
/// mode MUST return `Ok(_)` — E-LAY-003 is non-blocking (exit-0) in warn-only.
///
/// Traceability: F-098-P1-004; BC-1.11.002 v1.2 PC-007; STORY-098 AC-005;
///               error-taxonomy v2.31 E-LAY-003 (--warn-only → exit 0).
#[test]
fn test_f098_p1_004_ac005_chart_no_data_warn_only_exits_0() {
    let brand = BrandTmpDir::new("s098_chart_no_data_warn");

    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "\n",
        "slide chart:\n",
        "  title \"Revenue\"\n",
        "  chart_type \"bar\"\n",
        "  alt \"Bar chart for revenue\"\n",
    );
    let opts = brand.build_options("pptx", false); // strict=false → warn-only

    let result = slideforge::build(source, &opts);

    assert!(
        result.is_ok(),
        "AC-005 / F-098-P1-004: chart with no data: field in warn-only mode must return \
         Ok — E-LAY-003 is non-blocking (exit-0) per error-taxonomy v2.31. \
         Got: {result:?}"
    );
}
