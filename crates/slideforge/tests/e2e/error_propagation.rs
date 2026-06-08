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

// ── BC-1.15.002 TV-13.1: cross-stage eval + validator error accumulation ─────

/// BC-1.15.002 TV-13.1: eval Error (non-fatal, deck returned Some) AND validator
/// Error (missing alt) both reported in a single strict-mode build run.
///
/// This is the load-bearing end-to-end test for F-P2-MED-001: cross-stage error
/// accumulation broken. Before the fix, `compile_inner` early-returned `EvalFailed`
/// as soon as eval had non-fatal Error diagnostics — BEFORE the validator loop ran.
/// Decks with both an eval Error and a validator Error reported only the eval error.
///
/// After the fix: eval non-fatal errors do NOT cause an early return. Validators
/// run, all diagnostics are merged, and the strict gate fires once with all errors.
///
/// Fixture: `test-eval-and-validator-errors.sf`
///   - 2x E-EVL-001 (undefined variables in title fields) — non-fatal eval errors
///   - 1x E-A11-001 (missing alt on chart block) — validator error
///
/// BC-1.15.002 PC1: all 3 errors appear in the merged result.
/// BC-1.15.002 invariant 3: accumulation applies to evaluator + validators collectively.
/// BC-1.15.002 canonical test vector TV-13.1.
#[test]
fn test_bc_1_15_002_cross_stage_eval_and_validator_errors_both_reported() {
    let brand = BrandTmpDir::new("bc_1_15_002_tv13");
    let source = fixture_source("test-eval-and-validator-errors.sf");
    let opts = brand.build_options("pptx", true); // strict=true

    let result = slideforge::build(&source, &opts);

    // Must be an Err (strict mode, errors present).
    assert!(
        result.is_err(),
        "BC-1.15.002 TV-13.1: strict build with eval Error + validator Error must \
         return Err; got: {result:?}"
    );

    // The error must carry BOTH the eval diagnostics (E-EVL-001) and the
    // validator diagnostics (E-A11-001). Inspect the merged error variant.
    let err = result.unwrap_err();
    match &err {
        slideforge::error::BuildError::MultistageFailed {
            eval_diagnostics,
            validator_diagnostics,
            ..
        } => {
            // PC1: eval diagnostics must contain E-EVL-001.
            let has_evl = eval_diagnostics.iter().any(|d| {
                d.code()
                    .is_some_and(|c| c.to_string().contains("E-EVL-001"))
            });
            assert!(
                has_evl,
                "BC-1.15.002 TV-13.1 PC1: MultistageFailed.eval_diagnostics must contain \
                 E-EVL-001; got codes: {:?}",
                eval_diagnostics
                    .iter()
                    .map(|d| d.code().map(|c| c.to_string()).unwrap_or_default())
                    .collect::<Vec<_>>()
            );

            // PC1: validator diagnostics must contain E-A11-001.
            let has_a11 = validator_diagnostics
                .iter()
                .any(|d| d.code.as_ref() == "E-A11-001");
            assert!(
                has_a11,
                "BC-1.15.002 TV-13.1 PC1: MultistageFailed.validator_diagnostics must \
                 contain E-A11-001; got codes: {:?}",
                validator_diagnostics
                    .iter()
                    .map(|d| d.code.as_ref())
                    .collect::<Vec<_>>()
            );

            // Invariant: at least 2 E-EVL-001 entries (2 undefined vars in fixture).
            let evl_count = eval_diagnostics
                .iter()
                .filter(|d| {
                    d.code()
                        .is_some_and(|c| c.to_string().contains("E-EVL-001"))
                })
                .count();
            assert!(
                evl_count >= 2,
                "BC-1.15.002 TV-13.1: eval_diagnostics must contain at least 2 E-EVL-001 \
                 entries (2 undefined variables in fixture); got {evl_count}"
            );
        },
        other => {
            panic!(
                "BC-1.15.002 TV-13.1: expected BuildError::MultistageFailed for a deck \
                 with both eval and validator errors; got: {other:?}"
            );
        },
    }
}

/// BC-1.15.002 TV-13.1 warn-only variant: strict=false must NOT return an error
/// even when the deck has both eval and validator errors.
///
/// In warn-only mode, all diagnostics are demoted to warnings and the build
/// continues to produce output.
#[test]
fn test_bc_1_15_002_cross_stage_warn_only_succeeds_despite_errors() {
    let brand = BrandTmpDir::new("bc_1_15_002_warnonly");
    let source = fixture_source("test-eval-and-validator-errors.sf");
    let opts = brand.build_options("pptx", false); // strict=false (warn-only)

    let result = slideforge::build(&source, &opts);

    assert!(
        result.is_ok(),
        "BC-1.15.002 TV-13.1 warn-only: build with strict=false must succeed \
         (eval + validator errors demoted to warnings); got: {result:?}"
    );
}

// ── EC-002: @include graceful failure ────────────────────────────────────────

/// EC-002: a source with an `@include` directive (unimplemented or pointing to
/// a missing file) must not panic — it must fail with a structured error.
///
/// Current behavior: `@include` is not yet implemented in `slideforge-syntax`.
/// The directive is currently treated as unknown syntax; the pipeline returns some
/// form of parse or eval failure without panicking. When `@include` is fully
/// implemented (tracked separately from STORY-050), the assertion should be tightened
/// to `Err(BuildError::ParseFailed { .. })` with code `E-INC-001` for missing files.
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
/// Current behavior: `@data` is not yet implemented in `slideforge-eval`.
/// The directive currently passes through without binding data; the pipeline
/// completes without panicking. When `@data` is fully implemented, the assertion
/// should be tightened to `Err(BuildError::EvalFailed { .. })` with a
/// `E-DATA-001` code for missing data files.
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
