//! STORY-098 exit-code end-to-end tests — F-098-P1-004 + PO adjudication F-098-ADJ-BODY-CONTENT.
//!
//! ## Acceptance Criteria covered
//!
//! - **AC-001** (content-drop strict): `body` on a slide type that does NOT declare it
//!   (e.g., `chart`) in strict mode → `Err(BuildError::ValidationFailed)` with W-VAL-103
//!   promoted to Error (`CONTENT_DROP_KEY`, BC-3.03.002 Invariant 4).
//! - **AC-001 warn-only**: same slide type in warn-only mode → `Ok`, W-VAL-103 non-blocking.
//! - **AC-002** (body on content render-success): `body` on `content` slide in strict mode
//!   → `Ok` (exit 0); body prose rendered in `FrameContent::Body` frame in `LaidOutDeck`;
//!   NO W-VAL-103 emitted. `body` is a declared known field on `content`.
//!   (PO adjudication F-098-ADJ-BODY-CONTENT; BC-3.03.002 v1.3 EC-007 reversed.)
//! - **AC-001 regression guard** (`test_body_on_quote_strict_exits_2`): `body` on `quote`
//!   slide (which does NOT declare `body`) in strict mode → `Err(ValidationFailed)` + W-VAL-103.
//!   Guards the `known_fields()` authority boundary from future drift.
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
//! F-098-P1-004 (adversary pass-1 finding); BC-3.03.002 v1.3 Invariant 4
//! (content-drop keys → Error in strict mode; `body` on `content` is VALID per EC-007 reversal);
//! BC-1.11.002 v1.2 (E-LAY-003 for missing/empty chart data);
//! STORY-098 AC-001, AC-002, AC-004, AC-005; PO adjudication F-098-ADJ-BODY-CONTENT.

#![allow(clippy::unwrap_used)] // test assertions — panics are intentional

use crate::e2e::BrandTmpDir;

// ── AC-001: body on non-supporting slide — strict mode → exit 2 ──────────────

/// AC-001 / BC-3.03.002 v1.3 Invariant 4: `body` on a slide type that does NOT
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
        "  body \"This body field is not a declared known_field on chart slides.\"\n",
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

/// AC-002 / BC-3.03.002 v1.3 Route A: `body` on a slide type that does NOT
/// declare it MUST return `Ok(_)` in warn-only mode — W-VAL-103 is non-blocking.
/// Body must NOT be threaded into the output slide blocks.
///
/// Note: `content` slides now declare `body` as an optional field, so this test
/// uses `chart` slides (which do NOT support `body`).
///
/// Traceability: F-098-P1-004; BC-3.03.002 Route A; STORY-098 AC-002.
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
        "  body \"This body field is not a declared known_field on chart slides.\"\n",
    );
    let opts = brand.build_options("pptx", false); // strict=false → warn-only

    let result = slideforge::build(source, &opts);

    assert!(
        result.is_ok(),
        "AC-002 / F-098-P1-004: `body` on `chart` slide in warn-only mode must return \
         Ok — W-VAL-103 is non-blocking (BC-3.03.002 Route A). Got: {result:?}"
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

// ── AC-002 v1.2: body on content slide — strict mode render success ───────────

/// AC-002 v1.2 / PO adjudication F-098-ADJ-BODY-CONTENT / BC-3.03.002 v1.3 EC-007:
/// `body` on a `content` slide in strict mode MUST return `Ok` (exit 0), NO W-VAL-103
/// emitted, and the `LaidOutDeck` MUST contain a `FrameContent::Body` frame carrying
/// the prose string "Prose text".
///
/// ## Contract (BC-3.03.002 v1.3 EC-007 — REVERSED from v1.2)
///
/// `body` is a declared `known_field` on the `content` slide type (content.rs optional
/// fields, STORY-098 F-098-P1-002). `known_fields()` is the authority (BC-3.03.002
/// v1.3 Invariant 4): because `body` IS in `content`'s `known_fields`, W-VAL-103 is
/// NOT emitted and the build SUCCEEDS in strict mode.
///
/// The threaded body text "Prose text" must be visible in a `FrameContent::Body` frame
/// in the `LaidOutDeck`. This is the load-bearing assertion (TD-VSDD-059): mere
/// exit-0 is insufficient — the test asserts the DISTINGUISHING content (the prose
/// string) in the output IR, not just frame presence.
///
/// Strict Ok implies no W-VAL-103 was accumulated (any Error-severity diagnostic
/// would cause `Err(ValidationFailed)` in strict mode).
///
/// ## Traceability
///
/// BC-3.03.002 v1.3 EC-007 (reversed); PO adjudication F-098-ADJ-BODY-CONTENT;
/// BC-4.01.001 v1.2 PC-11 (`TextTag::Body` → `FrameContent::Body` for content slides);
/// STORY-098 AC-002 v1.2; BC-3.03.002 v1.3 Invariant 4 (`known_fields()` authority).
#[test]
fn test_body_on_content_renders_and_exits_0() {
    use std::sync::Arc;

    use slideforge::CompileOptions;
    use slideforge_layout::FrameContent;
    use slideforge_plugin_api::BrandSource;
    use slideforge_types::block::ContentBlock;
    use slideforge_types::inline::InlineNode;

    let brand = BrandTmpDir::new("s098_body_content_strict");

    // `content` slide with `title` and `body` — both schema-valid.
    // `body` is a declared optional field on `content` (content.rs, STORY-098).
    // In strict mode, this must NOT trigger W-VAL-103 and must build successfully.
    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "\n",
        "slide content:\n",
        "  title \"My Content Slide\"\n",
        "  body \"Prose text\"\n",
    );

    let compile_opts = CompileOptions {
        brand_source: Some(BrandSource::TomlFile(Arc::from(
            brand.brand_toml_path.to_string_lossy().as_ref(),
        ))),
        strict: true, // strict mode — W-VAL-103 would cause Err(ValidationFailed)
        active_variant: None,
        source_name: Some(Arc::from("<test:ac002>")),
    };

    let result = slideforge::compile(source, &compile_opts);

    // Exit-0 semantics: strict Ok means no W-VAL-103 or other Error diagnostics.
    assert!(
        result.is_ok(),
        "AC-002 v1.2 / BC-3.03.002 v1.3 EC-007: `body` on `content` slide in strict mode \
         MUST return Ok — body is a declared known_field on content type; W-VAL-103 must \
         NOT be emitted (PO adjudication F-098-ADJ-BODY-CONTENT). \
         Got Err: {}",
        match &result {
            Ok(_) => String::new(),
            Err(e) => format!("{e}"),
        }
    );

    let compiled = result.unwrap();
    let laid_out = &compiled.laid_out;

    // DISTINGUISHING load-bearing assertion (TD-VSDD-059): the prose string "Prose text"
    // must appear in a FrameContent::Body frame in the LaidOutDeck.
    // This is the AC-002 v1.2 requirement: "assert the DISTINGUISHING content — the prose
    // string — not mere frame presence."
    //
    // BC-4.01.001 v1.2 PC-11: body field on content slide → TextTag::Body →
    // ContentBlock::Text → FrameContent::Body in LaidOutSlide.
    let has_prose_text = laid_out.slides.iter().any(|slide| {
        slide.frames.iter().any(|f| {
            if let FrameContent::Body(blocks) = &f.content {
                blocks.iter().any(|b| {
                    if let ContentBlock::Text(tb) = b {
                        tb.inlines.iter().any(
                            |n| matches!(n, InlineNode::Plain(s) if s.as_ref().contains("Prose text")),
                        )
                    } else {
                        false
                    }
                })
            } else {
                false
            }
        })
    });

    assert!(
        has_prose_text,
        "AC-002 v1.2 LOAD-BEARING (TD-VSDD-059): `body \"Prose text\"` on `content` slide \
         MUST appear as 'Prose text' in a FrameContent::Body frame in the LaidOutDeck \
         (BC-4.01.001 v1.2 PC-11: TextTag::Body → FrameContent::Body). \
         Got slides: {:?}",
        laid_out
            .slides
            .iter()
            .map(|s| s
                .frames
                .iter()
                .map(|f| format!("{:?}", f.content))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );
}

// ── AC-001 regression guard: body on quote (non-declaring type) → exit 2 ─────

/// AC-001 regression guard / BC-3.03.002 v1.3 Invariant 4 — `known_fields()` authority:
/// `body` on `quote` slide type in strict mode MUST return `Err(ValidationFailed)` with
/// W-VAL-103 at Error severity.
///
/// ## Contract
///
/// `quote` does NOT declare `body` in `known_fields()`. Per BC-3.03.002 v1.3 Invariant 4,
/// the content-drop keys `{"shape", "body"}` trigger W-VAL-103 promoted to broken/Error
/// in strict mode when the slide type does NOT include the key in its `known_fields()`.
///
/// This test guards the `known_fields()` authority boundary: if `known_fields("quote")`
/// were incorrectly updated to include `"body"`, this test would fail and catch the drift.
///
/// ## Why quote, not chart
///
/// `chart` is already covered by the AC-001 tests above. `quote` is a second non-body-
/// declaring type, chosen because it is a text-heavy slide (unlikely to accidentally
/// acquire `body` via `common_optional_fields`). Using a second type guards against a
/// regression where the content-drop logic uses a hardcoded type list instead of
/// `known_fields()` as the authority.
///
/// The W-VAL-103 message format must be UNCHANGED per BC-3.03.002 v1.3 Route A:
/// `"Unknown field 'body' for slide type 'quote'..."` (same format, different type name).
///
/// ## Traceability
///
/// BC-3.03.002 v1.3 Invariant 4 (`known_fields()` authority); BC-3.03.002 v1.3 EC-007
/// reversed (body on content valid; body on non-declaring types still triggers exit 2);
/// STORY-098 AC-003 regression guard.
#[test]
fn test_body_on_quote_strict_exits_2() {
    let brand = BrandTmpDir::new("s098_body_quote_strict");

    // `quote` slide with required fields + `body` which is NOT a known_field on quote.
    // quote requires `quote` and `attribution`; body is not in common_optional_fields.
    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "\n",
        "slide quote:\n",
        "  quote \"The best way to predict the future is to invent it.\"\n",
        "  attribution \"Alan Kay\"\n",
        "  body \"This body field is not declared in known_fields for quote slides.\"\n",
    );
    let opts = brand.build_options("pptx", true); // strict=true

    let result = slideforge::build(source, &opts);

    assert!(
        matches!(
            result,
            Err(slideforge::error::BuildError::ValidationFailed { .. })
        ),
        "AC-001 regression guard: `body` on `quote` slide (non-declaring type) in strict \
         mode MUST return Err(ValidationFailed) — W-VAL-103 is promoted to Error for \
         content-drop key 'body' on slide types whose known_fields() does NOT include \
         'body' (BC-3.03.002 v1.3 Invariant 4). Got: {result:?}"
    );

    let Err(slideforge::error::BuildError::ValidationFailed {
        ref diagnostics, ..
    }) = result
    else {
        unreachable!("matched Err above")
    };

    // W-VAL-103 must be present with Error severity (message format UNCHANGED — Route A).
    let w_val_103: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_ref() == "W-VAL-103" && d.message.contains("'body'"))
        .collect();
    assert!(
        !w_val_103.is_empty(),
        "AC-001 regression guard: ValidationFailed must carry W-VAL-103 for 'body' on \
         'quote' slide. known_fields('quote') does not include 'body'. \
         Got codes: {:?}",
        diagnostics
            .iter()
            .map(|d| d.code.as_ref())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        w_val_103[0].severity,
        slideforge_plugin_api::DiagnosticSeverity::Error,
        "AC-001 regression guard: W-VAL-103 for content-drop key 'body' on 'quote' slide \
         must be Error severity (BC-3.03.002 v1.3 Invariant 4). Got: {:?}",
        w_val_103[0].severity
    );
    // Message format check — Route A: W-VAL-103 message format UNCHANGED.
    assert!(
        w_val_103[0].message.contains("quote"),
        "AC-001 regression guard: W-VAL-103 message must name the slide type 'quote'; \
         got: {}",
        w_val_103[0].message
    );
}
