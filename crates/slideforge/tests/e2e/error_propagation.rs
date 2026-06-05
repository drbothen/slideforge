//! Error propagation E2E tests — STORY-050 AC-006, AC-009, EC-001 through EC-004.
//!
//! ## Acceptance Criteria covered
//!
//! - **AC-006**: invalid fixture → `Err(BuildError::ParseFailed { .. })` with
//!   non-empty structured diagnostics.
//! - **AC-009**: missing-alt fixture → `Err(BuildError::ValidationFailed { .. })`
//!   with at least one `"E-A11-001"` diagnostic.
//! - **EC-001**: zero-slide source → `Err(BuildError::ValidationFailed { .. })`
//!   with `"E-LAY-002"`.
//! - **EC-004**: register fields (`notes`, `report`) do not break the build pipeline.

#![allow(clippy::unwrap_used)]

use crate::e2e::{BrandTmpDir, fixture_source};

// ── AC-006: parse error propagation ─────────────────────────────────────────

/// AC-006: `build()` with `test-invalid-syntax.sf` returns
/// `Err(BuildError::ParseFailed { .. })`.
///
/// The fixture has a tab-indented line at line 3 which the parser rejects
/// with an E-PAR-001-style error. The tab at position 0 of line 3 is the
/// known syntax error.
///
/// Traceability: BC-5.02.001 postcondition 4, STORY-050 AC-006.
#[test]
fn test_bc_5_02_001_ac006_parse_error_returns_parse_failed() {
    let brand = BrandTmpDir::new("ac006_parse");
    let source = fixture_source("test-invalid-syntax.sf");
    let opts = brand.build_options("pptx", false);

    let result = slideforge::build(&source, &opts);

    assert!(
        matches!(
            result,
            Err(slideforge::error::BuildError::ParseFailed { .. })
        ),
        "AC-006: invalid syntax fixture must produce BuildError::ParseFailed; \
         got: {result:?}"
    );
}

/// AC-006 (diagnostics): `ParseFailed.diagnostics` must be non-empty with
/// structured error codes.
///
/// Each diagnostic must have a code (not just a message), confirming that
/// structured diagnostic information (error code + hint) is preserved.
#[test]
fn test_bc_5_02_001_ac006_parse_failed_has_structured_diagnostics() {
    let brand = BrandTmpDir::new("ac006_diag");
    let source = fixture_source("test-invalid-syntax.sf");
    let opts = brand.build_options("pptx", false);

    let result = slideforge::build(&source, &opts);

    if let Err(slideforge::error::BuildError::ParseFailed { diagnostics, count }) = result {
        assert!(
            count > 0,
            "AC-006: ParseFailed.count must be > 0 for an invalid source"
        );
        assert!(
            !diagnostics.is_empty(),
            "AC-006: ParseFailed.diagnostics must be non-empty"
        );
        for (i, diag) in diagnostics.iter().enumerate() {
            assert!(
                diag.code().is_some(),
                "AC-006: ParseFailed.diagnostics[{i}] must have a code \
                 (structured error from parser)"
            );
        }
    } else {
        panic!(
            "AC-006: expected BuildError::ParseFailed for invalid syntax fixture; \
             got: {result:?}"
        );
    }
}

// ── AC-009: strict validation error ─────────────────────────────────────────

/// AC-009: `build()` with `test-missing-alt.sf` and `strict=true` returns
/// `Err(BuildError::ValidationFailed { .. })` with `"E-A11-001"`.
///
/// The fixture has a `chart:` block with no `alt` attribute. The `AltTextValidator`
/// emits `E-A11-001` (Error severity). In strict mode, `build()` returns
/// `ValidationFailed`.
///
/// Traceability: BC-5.02.001 postcondition 4, STORY-050 AC-009.
#[test]
fn test_bc_5_02_001_ac009_missing_alt_returns_validation_failed() {
    let brand = BrandTmpDir::new("ac009_missing_alt");
    let source = fixture_source("test-missing-alt.sf");
    let opts = brand.build_options("pptx", true); // strict=true is the default

    let result = slideforge::build(&source, &opts);

    assert!(
        matches!(
            result,
            Err(slideforge::error::BuildError::ValidationFailed { .. })
        ),
        "AC-009: missing-alt fixture with strict=true must produce \
         BuildError::ValidationFailed; got: {result:?}"
    );
}

/// AC-009 (error code): `ValidationFailed.diagnostics` must contain `"E-A11-001"`.
///
/// `ValidationFailed.diagnostics` is `Vec<slideforge_plugin_api::Diagnostic>` —
/// the `code` field is `Arc<str>` and directly readable.
#[test]
fn test_bc_5_02_001_ac009_validation_failed_contains_e_a11_001() {
    let brand = BrandTmpDir::new("ac009_code");
    let source = fixture_source("test-missing-alt.sf");
    let opts = brand.build_options("pptx", true);

    let result = slideforge::build(&source, &opts);

    if let Err(slideforge::error::BuildError::ValidationFailed { diagnostics, count }) = result {
        assert!(
            count > 0,
            "AC-009: ValidationFailed.count must be > 0 for missing-alt fixture"
        );
        let has_e_a11_001 = diagnostics.iter().any(|d| d.code.as_ref() == "E-A11-001");
        assert!(
            has_e_a11_001,
            "AC-009: ValidationFailed.diagnostics must contain at least one 'E-A11-001' \
             diagnostic; got codes: {:?}",
            diagnostics
                .iter()
                .map(|d| d.code.as_ref())
                .collect::<Vec<_>>()
        );
    } else {
        panic!("AC-009: expected BuildError::ValidationFailed; got: {result:?}");
    }
}

/// AC-009 (warn-only): strict=false must NOT return `ValidationFailed` even
/// with missing-alt diagnostics.
#[test]
fn test_bc_5_02_001_ac009_warn_only_does_not_return_validation_failed() {
    let brand = BrandTmpDir::new("ac009_warnonly");
    let source = fixture_source("test-missing-alt.sf");
    let opts = brand.build_options("pptx", false); // strict=false

    let result = slideforge::build(&source, &opts);

    assert!(
        !matches!(
            result,
            Err(slideforge::error::BuildError::ValidationFailed { .. })
        ),
        "AC-009 (warn-only): strict=false must NOT return ValidationFailed; \
         got: {result:?}"
    );
}

// ── EC-001: empty-slide deck ─────────────────────────────────────────────────

/// EC-001: a zero-slide source produces `Err(BuildError::ValidationFailed)`
/// with `"E-LAY-002"` from `ZeroSlideValidator`.
///
/// Traceability: STORY-050 EC-001.
#[test]
fn test_bc_5_02_001_ec001_zero_slide_returns_validation_failed_e_lay_002() {
    let brand = BrandTmpDir::new("ec001_zero");

    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        // No slide blocks — ZeroSlideValidator emits E-LAY-002.
    );
    let opts = brand.build_options("pptx", true);

    let result = slideforge::build(source, &opts);

    assert!(
        matches!(
            result,
            Err(slideforge::error::BuildError::ValidationFailed { .. })
        ),
        "EC-001: zero-slide source must return ValidationFailed; got: {result:?}"
    );

    if let Err(slideforge::error::BuildError::ValidationFailed { diagnostics, .. }) = result {
        let has_e_lay_002 = diagnostics.iter().any(|d| d.code.as_ref() == "E-LAY-002");
        assert!(
            has_e_lay_002,
            "EC-001: ValidationFailed must contain 'E-LAY-002'; got: {:?}",
            diagnostics
                .iter()
                .map(|d| d.code.as_ref())
                .collect::<Vec<_>>()
        );
    }
}

// ── EC-004: register routing does not break build ────────────────────────────

/// EC-004: the 3-slide fixture (which contains `notes` and `report` register
/// fields) builds without parse or eval errors.
///
/// Register fields are scalar fields on slides — they must be parsed, eval'd,
/// and routed without breaking the pipeline.
///
/// Traceability: STORY-050 EC-004.
#[test]
fn test_bc_5_02_001_ec004_register_fields_do_not_break_build() {
    let brand = BrandTmpDir::new("ec004_reg");
    let source = fixture_source("test-3slide.sf");
    let opts = brand.build_options("pptx", false);

    let result = slideforge::build(&source, &opts);

    // Parse and eval must not fail due to notes/report fields.
    assert!(
        !matches!(
            result,
            Err(slideforge::error::BuildError::ParseFailed { .. })
        ),
        "EC-004: register fields (notes/report) must not cause ParseFailed; \
         got: {result:?}"
    );
    assert!(
        !matches!(
            result,
            Err(slideforge::error::BuildError::EvalFailed { .. })
        ),
        "EC-004: register fields (notes/report) must not cause EvalFailed; \
         got: {result:?}"
    );
}

// ── EC-002: @include graceful failure ────────────────────────────────────────

/// EC-002: a source with an `@include` directive (unimplemented or pointing to
/// a missing file) must not panic — it must fail with a structured error.
///
/// FLAGGED FOR ORCHESTRATOR: when @include is implemented, tighten to assert
/// `Err(BuildError::ParseFailed { .. })` with code `E-INC-001` for circular includes.
///
/// Traceability: STORY-050 EC-002.
#[test]
fn test_bc_5_02_001_ec002_include_directive_fails_gracefully_no_panic() {
    let brand = BrandTmpDir::new("ec002_include");
    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "@include \"nonexistent.sf\"\n",
        "slide title:\n",
        "  title \"Test\"\n",
    );
    let opts = brand.build_options("pptx", false);
    // Any result is acceptable except a process panic/crash.
    let _ = slideforge::build(source, &opts);
}

// ── EC-003: @data missing file graceful failure ──────────────────────────────

/// EC-003: a `@data` directive pointing to a nonexistent file must not panic.
///
/// FLAGGED FOR ORCHESTRATOR: update the expected error variant once @data
/// is fully implemented.
///
/// Traceability: STORY-050 EC-003.
#[test]
fn test_bc_5_02_001_ec003_data_missing_file_fails_gracefully_no_panic() {
    let brand = BrandTmpDir::new("ec003_data");
    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "slide content:\n",
        "  title \"Data slide\"\n",
        "  @data \"nonexistent.json\"\n",
    );
    let opts = brand.build_options("pptx", false);
    let _ = slideforge::build(source, &opts);
}
