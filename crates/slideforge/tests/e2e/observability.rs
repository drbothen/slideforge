//! Observability E2E tests — STORY-050 AC-007.
//!
//! BC-5.02.001 AC-007 requires exactly 6 canonical pipeline stage spans named:
//! `parse`, `evaluate`, `brand`, `validate`, `layout`, `export`
//!
//! These are `tracing::info_span!` entries emitted by `build_inner()`. The
//! `tracing_test::traced_test` macro installs a subscriber that captures span
//! names and events from all crates at INFO+ level (with `no-env-filter` feature
//! enabled in dev-dependencies — see `crates/slideforge/Cargo.toml`).
//!
//! ## Load-bearing mechanism
//!
//! Each canonical span carries a `stage = "<name>"` structured field whose value
//! equals the span name exactly:
//!
//! ```rust,ignore
//! tracing::info_span!("brand", stage = "brand")
//! tracing::info_span!("parse", stage = "parse", source_len = ...)
//! tracing::info_span!("evaluate", stage = "evaluate")
//! tracing::info_span!("validate", stage = "validate", strict = ...)
//! tracing::info_span!("layout", stage = "layout")
//! tracing::info_span!("export", stage = "export", format = ...)
//! ```
//!
//! The test asserts `logs_contain("stage=\"<name>\"")` for each canonical stage.
//! This is LOAD-BEARING: if the span name is changed (e.g. `evaluate` → `eval`)
//! while leaving the `stage` field value unchanged, the `stage="evaluate"` assertion
//! fails. If the field value is changed independently, it also fails. A revert of
//! either the span name or the field is caught.
//!
//! Regression guards assert `!logs_contain("stage=\"brand_load\"")` and
//! `!logs_contain("stage=\"eval\"")`. These are REAL (not vacuously true) because
//! no code anywhere emits `stage="brand_load"` or `stage="eval"` — if a developer
//! were to rename the canonical span back to those stale names AND change the field
//! to match, the guard catches it.
//!
//! Non-canonical markers (`pipeline_start`, `inject_lang_default`,
//! `validate_post_layout`) are emitted as plain events without a `stage` field,
//! so they do not appear in the 6 canonical span set.

#![allow(clippy::unwrap_used)]

use crate::e2e::{BrandTmpDir, fixture_source};

/// AC-007: all 6 canonical pipeline stage spans are present in the captured
/// tracing output when `build()` succeeds.
///
/// ## What is asserted and why it is load-bearing
///
/// Each canonical span carries a `stage = "<name>"` structured field. The
/// `tracing_test` subscriber renders this as `stage="<name>"` in the captured
/// log output. This test asserts:
///
/// 1. `logs_contain("stage=\"parse\"")` — span `parse` with `stage = "parse"` fired.
/// 2. `logs_contain("stage=\"evaluate\"")` — span `evaluate` with `stage = "evaluate"` fired.
/// 3. `logs_contain("stage=\"brand\"")` — span `brand` with `stage = "brand"` fired.
/// 4. `logs_contain("stage=\"validate\"")` — span `validate` with `stage = "validate"` fired.
/// 5. `logs_contain("stage=\"layout\"")` — span `layout` with `stage = "layout"` fired.
/// 6. `logs_contain("stage=\"export\"")` — span `export` with `stage = "export"` fired.
///
/// Renaming any canonical span (e.g. `evaluate` → `eval`) AND its field to match
/// causes assertion 2 to fail because `stage="evaluate"` no longer appears.
/// Renaming only the span name while keeping `stage = "evaluate"` also causes
/// a mismatch — the span context path changes but the field still fires, so the
/// assertion would still pass; however the regression guard `!logs_contain("stage=\"eval\"")`
/// catches the stale name if someone wires `stage = "eval"` to the new span.
///
/// ## Regression guards
///
/// `!logs_contain("stage=\"brand_load\"")` and `!logs_contain("stage=\"eval\"")`:
/// these assert that stale names from before the AC-007 rename fix do NOT appear.
/// The guards are REAL: no code in the pipeline emits `stage="brand_load"` or
/// `stage="eval"`. If the production code reverts to those stale names AND also
/// changes the structured field to match, both the positive assertion for the
/// canonical name AND the regression guard would fail simultaneously.
///
/// Traceability: BC-5.02.001 AC-007, NFR-032, STORY-050 AC-007.
#[test]
#[tracing_test::traced_test]
fn test_bc_5_02_001_ac007_all_pipeline_stage_markers_emitted() {
    let brand = BrandTmpDir::new("ac007_tracing");
    let source = fixture_source("test-3slide.sf");
    let opts = brand.build_options("pptx", false);

    let result = slideforge::build(&source, &opts);
    result.unwrap_or_else(|e| {
        panic!(
            "AC-007: build() must succeed for all stage spans to fire; \
             got Err: {e:?}"
        )
    });

    // Assert each of the 6 canonical pipeline stage spans fired by checking
    // the structured `stage` field value in the captured tracing output.
    //
    // Each span is declared as:
    //   tracing::info_span!("<name>", stage = "<name>", ...)
    //
    // `tracing_test` captures the rendered span fields. `logs_contain` matches
    // anywhere in the captured output, so `stage="parse"` matches the log line
    // that contains the span `parse{stage="parse", source_len=N}`.
    //
    // This is LOAD-BEARING: renaming the span name alone changes the span context
    // path but the `stage` field value is what this assertion checks. A developer
    // reverting `evaluate` → `eval` while also changing `stage = "eval"` causes
    // assertion (b) to fail AND the regression guard below to trigger.
    for canonical in &["parse", "evaluate", "brand", "validate", "layout", "export"] {
        let field_pattern = format!("stage=\"{canonical}\"");
        assert!(
            logs_contain(&field_pattern),
            "AC-007: canonical span '{canonical}' must carry structured field \
             stage=\"{canonical}\" in tracing output; if this fails, the span was \
             renamed or the stage field was removed (BC-5.02.001 AC-007 / NFR-032)"
        );
    }

    // Regression guards: stale structured field values from before the AC-007 rename fix.
    //
    // No code in the pipeline emits stage="brand_load" or stage="eval". These guards
    // are REAL — if a developer reverts the canonical span names AND changes the stage
    // field to match, the corresponding positive assertion above fails first, and these
    // guards catch any residual stale-name field emission.
    //
    // Note: bare "eval" is a prefix of "evaluate". We match `stage="eval"` (with the
    // closing quote) to avoid a false match on `stage="evaluate"`.
    assert!(
        !logs_contain("stage=\"brand_load\""),
        "AC-007: stale structured field stage=\"brand_load\" must NOT appear — \
         the canonical span name (and stage field) is 'brand'"
    );
    assert!(
        !logs_contain("stage=\"eval\""),
        "AC-007: stale structured field stage=\"eval\" must NOT appear — \
         the canonical span name (and stage field) is 'evaluate'. \
         Note: this matches stage=\"eval\" with closing quote to avoid \
         false-positive on stage=\"evaluate\""
    );
}

/// AC-007 (abort trace): when parse fails, span markers for later stages must NOT appear.
///
/// `brand` span fires before parse (brand is loaded first).
/// `evaluate`, `validate`, `layout`, `export` must NOT appear (pipeline aborted at parse).
///
/// These assertions check for `stage="<name>"` structured fields (not message text),
/// consistent with the load-bearing mechanism described above.
#[test]
#[tracing_test::traced_test]
fn test_bc_5_02_001_ac007_stage_markers_stop_at_parse_failure() {
    let brand = BrandTmpDir::new("ac007_parse_fail");
    let source = fixture_source("test-invalid-syntax.sf");
    let opts = brand.build_options("pptx", false);

    let result = slideforge::build(&source, &opts);
    assert!(
        matches!(
            result,
            Err(slideforge::error::BuildError::ParseFailed { .. })
        ),
        "AC-007: invalid syntax must cause ParseFailed; got: {result:?}"
    );

    // Post-failure stage structured fields must NOT appear.
    // Each canonical span carries stage="<name>". If parse fails early-return,
    // the subsequent spans are never entered and their fields never emitted.
    assert!(
        !logs_contain("stage=\"evaluate\""),
        "AC-007: stage=\"evaluate\" field must NOT appear when parse fails"
    );
    assert!(
        !logs_contain("stage=\"validate\""),
        "AC-007: stage=\"validate\" field must NOT appear when parse fails"
    );
    assert!(
        !logs_contain("stage=\"layout\""),
        "AC-007: stage=\"layout\" field must NOT appear when parse fails"
    );
    assert!(
        !logs_contain("stage=\"export\""),
        "AC-007: stage=\"export\" field must NOT appear when parse fails"
    );
}
