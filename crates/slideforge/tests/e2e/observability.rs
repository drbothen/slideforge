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
//! Non-canonical markers (`pipeline_start`, `inject_lang_default`,
//! `validate_post_layout`) are emitted as plain events, not spans, so they do
//! not appear in the 6 canonical span set.

#![allow(clippy::unwrap_used)]

use crate::e2e::{BrandTmpDir, fixture_source};

/// AC-007: all 6 canonical pipeline stage span names are present in the captured
/// tracing output when `build()` succeeds.
///
/// Asserts:
/// - Each of the 6 canonical names (`parse`, `evaluate`, `brand`, `validate`,
///   `layout`, `export`) appears in the captured output.
/// - The stale names `brand_load` and `eval` do NOT appear (regression guard).
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

    // Assert each of the 6 canonical pipeline stage spans fired.
    //
    // Each span emits a `tracing::info!("pipeline stage: <name>")` event inside
    // it. `tracing_test` captures these as log lines. The span name appears in
    // the log line context path (e.g., `test_name:parse{...}: slideforge: pipeline stage: parse`).
    // We assert on the unique "pipeline stage: <name>" message text, which is
    // unambiguous and tied directly to the span.
    for canonical in &["parse", "evaluate", "brand", "validate", "layout", "export"] {
        let marker = format!("pipeline stage: {canonical}");
        assert!(
            logs_contain(&marker),
            "AC-007: canonical span '{canonical}' must emit 'pipeline stage: {canonical}' in \
             tracing output; if this fails, the span was removed or the test subscriber is not \
             capturing INFO events from the 'slideforge' crate"
        );
    }

    // Regression guard: stale event text from before the AC-007 rename fix.
    // If the production code reverts to the old names, these assertions will fail.
    assert!(
        !logs_contain("stage=\"brand_load\""),
        "AC-007: stale structured field stage=\"brand_load\" must NOT appear — \
         the canonical span name is 'brand'"
    );
    assert!(
        !logs_contain("stage=\"eval\""),
        "AC-007: stale structured field stage=\"eval\" must NOT appear — \
         the canonical span name is 'evaluate'"
    );
}

/// AC-007 (abort trace): when parse fails, span markers for later stages must NOT appear.
///
/// `brand` span fires before parse (brand is loaded first).
/// `evaluate`, `validate`, `layout`, `export` must NOT appear (pipeline aborted at parse).
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

    // Post-failure stage markers must NOT appear.
    // Each canonical span emits "pipeline stage: <name>" inside it.
    // If parse fails early-return, these markers must not be in the captured output.
    assert!(
        !logs_contain("pipeline stage: evaluate"),
        "AC-007: 'evaluate' stage marker must NOT fire when parse fails"
    );
    assert!(
        !logs_contain("pipeline stage: validate"),
        "AC-007: 'validate' stage marker must NOT fire when parse fails"
    );
    assert!(
        !logs_contain("pipeline stage: layout"),
        "AC-007: 'layout' stage marker must NOT fire when parse fails"
    );
    assert!(
        !logs_contain("pipeline stage: export"),
        "AC-007: 'export' stage marker must NOT fire when parse fails"
    );
}
