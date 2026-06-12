//! STORY-089 — Field-Schema Validator Pipeline Wiring: build()-level Red Gate tests.
//!
//! ## What STORY-089 added (before this test file)
//!
//! STORY-089 added:
//! - `FieldType` / `type_matches` / `FieldDef.expected_type` (in `slideforge-plugin-api`)
//! - The E-VAL-104 arm inside `validate_fields` (in `slideforge-plugin-api/slide_types/registry.rs`)
//! - Priority-1 `expected_type` annotations on `progress_bar.value` (Int), `chart.chart_type`
//!   (OneOf), `weighted_composite.components` (List), `decorative` (Bool), and others.
//!
//! ## What the Red Gate exposed (historical wiring gap — now resolved)
//!
//! At the Red Gate, `validate_fields` was a **dead letter** at build time. `build_inner`
//! called `validator.validate(&deck, &validator_opts)` for every registered `Validator`
//! plugin (Stage 5), but no registered validator called `validate_fields`. The
//! `FieldSchemaValidator` had not yet been created or registered.
//!
//! Pre-wiring consequence: `build()` on a `progress_bar` slide with `value "fifty"`
//! (Str instead of Int) returned `Ok` in both strict and warn-only mode.
//!
//! ## Current wired state (post-STORY-089 implementation)
//!
//! `FieldSchemaValidator` is registered in `slideforge::registry::register_bundled_plugins`
//! at Stage 5 (alongside `ValueRangeValidator`, `LabelCheckValidator`, etc.). It:
//! - Iterates every `Slide` in the `Deck`.
//! - Looks up the `SlideType` for each slide in the `SlideTypeRegistry`.
//! - Calls `validate_fields(slide, slide_type)` for each slide.
//! - Accumulates all returned `Diagnostic` values (DI-018: no bail-on-first).
//! - Returns the accumulated `Vec<Diagnostic>`.
//!
//! Result: `build()` on a `progress_bar` slide with `value "fifty"` now returns
//! `Err(ValidationFailed)` in strict mode with at least one E-VAL-104 diagnostic.
//!
//! ## Build() API shape (strict vs. warn-only)
//!
//! ```text
//! BuildOptions { strict: true  } → Err(ValidationFailed { diagnostics, count })
//! BuildOptions { strict: false } → Ok(BuildOutput { bytes, extension })
//! ```
//!
//! Warn-only diagnostics: `BuildOutput` has NO `diagnostics` field. The warn-only path
//! emits E-VAL-104 diagnostics as `tracing::warn!` events in `build_inner` (Stage 5
//! log loop) and then continues the pipeline. The build does NOT surface warn-only
//! diagnostics to callers via `BuildOutput` — only strict mode carries them in
//! `ValidationFailed.diagnostics`. See `slideforge/src/lib.rs build_inner` for the
//! Stage 5 gate logic.
//!
//! ## AC-018 compile-fail artifact location
//!
//! The AC-018 `#[non_exhaustive]` compile-fail doctest lives on `FieldDef` in
//! `slideforge-plugin-api/src/traits/slide_type.rs` (see the rustdoc on `FieldDef`).
//! It is run by `cargo test -p slideforge-plugin-api --doc`.
//!
//! ## Test naming
//!
//! All test names follow the BC-traceability pattern `test_BC_1_18_001_xxx()`.
//! STORY-089 traces to BC-1.18.001 (Field-Value Type Validation).
//!
//! ## Traceability
//!
//! - BC-1.18.001 postcondition 2 (T1): Str on Int field → E-VAL-104
//! - BC-1.18.001 postcondition 7: E-VAL-104 reachable from build() strict-mode exit (AC-009/AC-010)
//! - ADR-020 Decision 8: FieldSchemaValidator as Stage-5 Validator plugin
//! - ADR-016 Decision 3: Validator surface registered in registry.rs

#![allow(clippy::unwrap_used)] // integration tests — panics on assertion failure are correct
#![allow(clippy::doc_markdown)] // test doc comments reference identifiers like E-VAL-104
#![allow(non_snake_case)] // BC-traceability IDs use uppercase: test_BC_S_SS_NNN_xxx
#![allow(clippy::uninlined_format_args)] // test assertion messages use named format args

use crate::e2e::{BrandTmpDir, fixture_source};

// ─────────────────────────────────────────────────────────────────────────────
// AC-009: strict mode — E-VAL-104 from FieldSchemaValidator
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.18.001 postcondition 7 / AC-009:
/// `build()` in strict mode on a `progress_bar` with `value "fifty"` (Str instead
/// of Int) MUST return `Err(BuildError::ValidationFailed)` AND the failure MUST
/// carry at least one diagnostic with `code == "E-VAL-104"`.
///
/// ## Historical Red Gate state (resolved by STORY-089 wiring)
///
/// At the Red Gate, `validate_fields` in `slideforge-plugin-api/slide_types/registry.rs`
/// correctly produced an E-VAL-104 diagnostic for `value "fifty"` on a `progress_bar`
/// slide, but `validate_fields` was NEVER called from `build_inner()`. No registered
/// `Validator` plugin invoked it.
///
/// ## Pre-wiring behavior (historical)
///
/// At the Red Gate, `ValueRangeValidator` (which was registered) detected
/// `Value::Str("fifty")` as a wrong-type on the `value` field and emitted `E-VAL-011`
/// (not E-VAL-104). This caused `Err(ValidationFailed)` in strict mode — but with the
/// wrong code. The test was RED because it required E-VAL-104 (field-schema type mismatch
/// from `FieldSchemaValidator`) and got only E-VAL-011 (range validator's wrong-type arm).
///
/// Pre-wiring evidence:
/// Run: `cargo nextest run -p slideforge -E 'test(test_BC_1_18_001_ac009_strict)'`
/// Pre-wiring result: `FAIL` at the E-VAL-104 assertion:
///   `Got diagnostic codes: ["E-VAL-011"]`
///
/// ## Current post-wiring behavior
///
/// With `FieldSchemaValidator` registered at Stage 5:
/// - `FieldSchemaValidator::validate(&deck)` calls `validate_fields` for each slide.
/// - `validate_fields` detects `Value::Str("fifty")` on `FieldType::Int` field.
/// - Returns one E-VAL-104 (T1: "expected integer, got string").
/// - `ValueRangeValidator` is range-only (BC-1.17.002 v1.3) and does NOT emit E-VAL-011
///   for wrong-type — type checking is delegated to `FieldSchemaValidator`.
/// - Only E-VAL-104 is present. The E-VAL-104 assertion PASSES.
///
/// ## Fixture: story-089-progress-bar-str-value.sf
///
/// ```text
/// slideforge_version "1"
/// lang "en-US"
/// slide progress_bar:
///   title "Sprint 4 Progress"
///   label "fifty percent complete"
///   value "fifty"
/// ```
///
/// Traceability: BC-1.18.001 postcondition 7; ADR-020 Decision 8; STORY-089 AC-009.
#[test]
fn test_BC_1_18_001_ac009_strict_progress_bar_str_value_returns_e_val_104() {
    let brand = BrandTmpDir::new("s089_ac009_strict");
    let source = fixture_source("story-089-progress-bar-str-value.sf");
    let opts = brand.build_options("pptx", true); // strict=true: ValidationFailed must fire

    let result = slideforge::build(&source, &opts);

    // Must be Err — any strict violation causes ValidationFailed.
    // Post-wiring: FieldSchemaValidator produces E-VAL-104 for Str on Int field.
    // ValueRangeValidator (BC-1.17.002 v1.3 / BC-1.18.001 EC-001) does NOT emit
    // E-VAL-011 for wrong-type — range-only validator; type checking delegated to
    // FieldSchemaValidator. So E-VAL-104 is the ONLY diagnostic code that causes
    // strict mode to return Err(ValidationFailed) here.
    assert!(
        matches!(
            result,
            Err(slideforge::error::BuildError::ValidationFailed { .. })
        ),
        "AC-009: build() in strict mode on progress_bar with value=\"fifty\" must \
         return Err(BuildError::ValidationFailed). \
         \nGot: {result:?}"
    );

    // Load-bearing assertion: the ValidationFailed MUST carry E-VAL-104.
    //
    // TD-VSDD-059 discipline: asserting Err alone is insufficient — any validator
    // can cause ValidationFailed. E-VAL-104 specifically proves that FieldSchemaValidator
    // (which calls validate_fields) is registered and wired at Stage 5.
    //
    // Post-wiring: FieldSchemaValidator runs → E-VAL-104 for Str on Int field.
    // ValueRangeValidator does NOT emit E-VAL-011 for wrong-type (BC-1.17.002 v1.3).
    // → Only E-VAL-104 is present. This assertion PASSES.
    let Err(slideforge::error::BuildError::ValidationFailed {
        ref diagnostics, ..
    }) = result
    else {
        unreachable!("matched Err above")
    };

    let e_val_104_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_ref() == "E-VAL-104")
        .collect();

    assert!(
        !e_val_104_diags.is_empty(),
        "AC-009: ValidationFailed.diagnostics must contain at least one E-VAL-104 \
         diagnostic (T1 type-mismatch: expected integer, got string — from FieldSchemaValidator \
         calling validate_fields). \
         \nGot diagnostic codes: {:?}",
        diagnostics
            .iter()
            .map(|d| d.code.as_ref())
            .collect::<Vec<_>>()
    );

    // Boundary assertion: E-VAL-011 must be ABSENT for wrong-type value.
    // BC-1.17.002 v1.3 / BC-1.18.001 EC-001: ValueRangeValidator is range-only;
    // it must NOT emit E-VAL-011 for a Str on an Int field.
    let e_val_011_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_ref() == "E-VAL-011")
        .collect();
    assert!(
        e_val_011_diags.is_empty(),
        "AC-009 boundary: E-VAL-011 must be ABSENT for wrong-type progress_bar value \
         (BC-1.17.002 v1.3 / BC-1.18.001 EC-001 — ValueRangeValidator is range-only; \
         type checking delegated to FieldSchemaValidator). \
         \nGot diagnostic codes: {:?}",
        diagnostics
            .iter()
            .map(|d| d.code.as_ref())
            .collect::<Vec<_>>()
    );

    // E-VAL-104 message must contain "integer" (expected) and "string" (actual).
    // BC-1.18.001 postcondition 2 T1 message format:
    //   "Field 'value' on progress_bar slide has wrong type: expected integer, got string."
    let first = e_val_104_diags[0];
    let msg = first.message.as_ref();
    assert!(
        msg.contains("integer"),
        "AC-009: E-VAL-104 T1 message must say 'integer' (expected type); got: {msg}"
    );
    assert!(
        msg.contains("string"),
        "AC-009: E-VAL-104 T1 message must say 'string' (actual type); got: {msg}"
    );

    // Severity must be Error.
    assert_eq!(
        first.severity,
        slideforge_plugin_api::DiagnosticSeverity::Error,
        "AC-009: E-VAL-104 must have Error severity (not Warning)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-010: warn-only mode — no ValidationFailed on type-mismatch fixture
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.18.001 postcondition 7 / AC-010 — guard test:
/// `build()` in warn-only mode (`strict=false`) on the same `progress_bar` fixture
/// with `value "fifty"` MUST return `Ok(BuildOutput)` — the build does NOT hard-fail.
///
/// ## AC-010 special analysis — guard test, not a Red Gate test
///
/// At the Red Gate (pre-wiring), no `FieldSchemaValidator` was registered. `build()`
/// returned `Ok` for this fixture regardless of strict mode (both strict and warn-only
/// succeeded — the wiring was absent so the type mismatch went undetected). The `Ok`
/// assertion PASSED trivially for the wrong reason.
///
/// Post-wiring (current): `FieldSchemaValidator` fires and produces E-VAL-104. Because
/// `strict=false`, `build_inner` does NOT apply the strict gate. The build continues
/// past validation to layout and export. Result: `Ok(BuildOutput)`. This assertion
/// still PASSES, now for the correct reason.
///
/// This test is therefore a GUARD test — not a Red Gate test. Its purpose is to
/// prevent regression: after wiring, warn-only mode must NOT return `ValidationFailed`
/// for type-mismatch findings.
///
/// ## Warn-only diagnostics: NOT surfaced in BuildOutput
///
/// `BuildOutput` has only `bytes: Vec<u8>` and `extension: String`. There is no
/// `diagnostics` field. Warn-only E-VAL-104 diagnostics are emitted as `tracing::warn!`
/// events in `build_inner` (Stage 5 log loop) and are NOT returned in `BuildOutput`.
/// This test therefore cannot assert the presence of E-VAL-104 in the `Ok` result —
/// only that the build did NOT return `ValidationFailed`.
///
/// ## Implementer note
///
/// If warn-only diagnostic surfacing is added to `BuildOutput` in a future story,
/// this test should be updated to assert E-VAL-104 presence in the output.
///
/// Traceability: BC-1.18.001 postcondition 7; ADR-020 Decision 8; STORY-089 AC-010.
#[test]
fn test_BC_1_18_001_ac010_warn_only_progress_bar_str_value_returns_ok() {
    let brand = BrandTmpDir::new("s089_ac010_warn");
    let source = fixture_source("story-089-progress-bar-str-value.sf");
    // strict=false: warn-only mode. Build must NOT return ValidationFailed.
    let opts = brand.build_options("pptx", false);

    let result = slideforge::build(&source, &opts);

    // Guard assertion: warn-only mode must return Ok regardless of validation diagnostics.
    //
    // Pre-wiring: PASSES trivially (no validator → Ok).
    // Post-wiring: PASSES correctly (E-VAL-104 emitted as warn; strict gate not applied).
    //
    // This test is a GUARD — it documents the post-wiring contract for warn-only mode.
    assert!(
        result.is_ok(),
        "AC-010: build() in warn-only mode (strict=false) on progress_bar with \
         value=\"fifty\" must return Ok — type-mismatch diagnostics are demoted to \
         warnings and the build proceeds. \
         \nPre-wiring: Ok for wrong reason (no validator). \
         Post-wiring: Ok for correct reason (strict gate not applied). \
         \nGot: {result:?}"
    );

    // Confirm output has expected structure.
    // This guards against regressions where the warn-only path accidentally panics
    // or returns incomplete output.
    let output = result.unwrap();
    assert!(
        !output.bytes.is_empty(),
        "AC-010: BuildOutput.bytes must be non-empty in warn-only mode for progress_bar fixture"
    );
    assert_eq!(
        output.extension, "pptx",
        "AC-010: BuildOutput.extension must be 'pptx' for pptx format"
    );

    // NOTE: BuildOutput.diagnostics does NOT exist. Warn-only E-VAL-104 diagnostics
    // are emitted only as tracing::warn! events — not surfaced to callers.
    // When a future story adds diagnostic surfacing to BuildOutput, add an assertion here:
    //
    //   let e104_present = output.diagnostics.iter().any(|d| d.code == "E-VAL-104");
    //   assert!(e104_present, "AC-010: warn-only output must carry E-VAL-104 in diagnostics");
    //
    // Until then, this test only asserts the build does not fail. The diagnostic
    // assertion is pending warn-only output API (not deferred — simply not yet specced).
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-009 positive control: valid Int value → strict build Ok, no E-VAL-104
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.18.001 postcondition 1 / AC-009 positive control:
/// `build()` in strict mode on a `progress_bar` with `value 75` (valid Int) MUST
/// return `Ok` — no E-VAL-104 diagnostic for a correctly-typed field.
///
/// This is a FALSE-POSITIVE GUARD. Post-wiring, if `FieldSchemaValidator` incorrectly
/// emits E-VAL-104 for a valid Int field, this test FAILS and catches the regression.
///
/// Uses the existing STORY-087 fixture `story-087-progress-bar-75.sf` which has
/// `value 75` (a valid integer in range).
///
/// Pre-wiring: PASSES trivially (no validator → Ok).
/// Post-wiring: PASSES correctly (no E-VAL-104 for valid Int).
///
/// Traceability: BC-1.18.001 postcondition 1; STORY-089 AC-009 positive control.
#[test]
fn test_BC_1_18_001_ac009_positive_control_valid_int_value_strict_ok() {
    let brand = BrandTmpDir::new("s089_ac009_pos");
    let source = fixture_source("story-087-progress-bar-75.sf");
    let opts = brand.build_options("pptx", true); // strict=true

    let result = slideforge::build(&source, &opts);

    // Must return Ok — valid Int value must not trigger E-VAL-104.
    assert!(
        result.is_ok(),
        "AC-009 positive control: build() with progress_bar value=75 (valid Int) must \
         return Ok in strict mode. FieldSchemaValidator must NOT emit E-VAL-104 for \
         correct types. \
         \nGot: {result:?}"
    );

    // Additional guard: ensure no E-VAL-104 was injected even if build returned Ok
    // via some future mechanism where Ok carries warnings.
    // (Currently BuildOutput has no diagnostics field, so this is structural only.)
    let output = result.unwrap();
    assert!(
        !output.bytes.is_empty(),
        "AC-009 positive control: BuildOutput.bytes must be non-empty for valid fixture"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Field-schema behavior: chart without `data` — E-VAL-101 is absent (optional field)
// BC-1.11.002 v1.2 behavior: E-LAY-003 is emitted instead (missing data source)
// ─────────────────────────────────────────────────────────────────────────────

/// STORY-089 architect decision guard + STORY-098 BC-1.11.002 v1.2 contract:
///
/// `build()` in strict mode on a `chart` slide WITHOUT the `data` field MUST return
/// `Err(ValidationFailed)` carrying `E-LAY-003` (NOT `E-VAL-101`).
///
/// ## Two separate decisions at play
///
/// 1. **STORY-089 FieldSchemaValidator decision**: `chart.data` is OPTIONAL at
///    field-schema level — `validate_fields` must NOT emit `E-VAL-101` for absent
///    `data`. This allows `@data` runtime references without parse-time false positives.
///
/// 2. **STORY-098 BC-1.11.002 v1.2 / F-098-P1-007**: `ChartEmptyDataValidator`
///    catches missing `data` at Stage 5 and emits `E-LAY-003` (broken/exit-2 in
///    strict mode). Missing `data` == empty `data` per PO adjudication BINDING.
///
/// ## Contract (post-STORY-098)
///
/// - `E-VAL-101` is ABSENT (chart.data is optional at field-schema level — STORY-089).
/// - `E-LAY-003` IS PRESENT (ChartEmptyDataValidator catches it — STORY-098).
/// - Result: `Err(ValidationFailed)` with E-LAY-003 (NOT E-VAL-101).
///
/// ## Fixture: inline DSL source (no file needed)
///
/// Chart slide with `title`, `chart_type`, `alt` but no `data` field.
/// `lang` satisfies `LangValidator`; `alt` satisfies `AltTextValidator`.
///
/// Traceability: STORY-089 architect decision (chart.data → optional at field-schema);
///               BC-1.11.002 v1.2 EC-005 (missing data → E-LAY-003, STORY-098).
#[test]
fn test_BC_1_18_001_ac009_regression_chart_no_data_emits_e_lay_003_not_e_val_101() {
    let brand = BrandTmpDir::new("s089_chart_no_data");

    // Chart slide without `data` field.
    // - title + chart_type: required fields provided.
    // - data: ABSENT — triggers E-LAY-003 (ChartEmptyDataValidator, STORY-098).
    // - alt: provided to satisfy AltTextValidator (chart frame alt text post-layout).
    // - lang: provided to satisfy LangValidator.
    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "\n",
        "slide chart:\n",
        "  title \"Q3 Revenue\"\n",
        "  chart_type \"bar\"\n",
        "  alt \"Bar chart showing Q3 revenue by region\"\n",
    );
    let opts = brand.build_options("pptx", true); // strict=true

    let result = slideforge::build(source, &opts);

    // Must be Err(ValidationFailed) — E-LAY-003 blocks in strict mode.
    // (Previously: returned Ok because ChartEmptyDataValidator was absent.)
    assert!(
        matches!(
            result,
            Err(slideforge::error::BuildError::ValidationFailed { .. })
        ),
        "STORY-098 BC-1.11.002 v1.2: build() on chart with no data: field must return \
         Err(ValidationFailed) — ChartEmptyDataValidator emits E-LAY-003 (broken/exit-2). \
         \nGot: {result:?}"
    );

    let Err(slideforge::error::BuildError::ValidationFailed {
        ref diagnostics, ..
    }) = result
    else {
        unreachable!("matched Err above")
    };

    // E-LAY-003 must be present (ChartEmptyDataValidator — STORY-098 F-098-P1-007).
    let e_lay_003: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_ref() == "E-LAY-003")
        .collect();
    assert!(
        !e_lay_003.is_empty(),
        "STORY-098 BC-1.11.002 v1.2: ValidationFailed must carry E-LAY-003 for \
         chart with no data: field. Got codes: {:?}",
        diagnostics
            .iter()
            .map(|d| d.code.as_ref())
            .collect::<Vec<_>>()
    );

    // E-VAL-101 must be ABSENT — chart.data is OPTIONAL at field-schema level
    // (STORY-089 architect decision). ChartEmptyDataValidator handles the runtime check.
    let e_val_101: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_ref() == "E-VAL-101")
        .collect();
    assert!(
        e_val_101.is_empty(),
        "STORY-089 regression guard: E-VAL-101 must be ABSENT for chart with no data: \
         (chart.data is optional at field-schema level per STORY-089 architect decision). \
         Got: {:?}",
        e_val_101
    );
}
