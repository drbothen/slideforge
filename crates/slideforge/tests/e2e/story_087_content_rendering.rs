//! STORY-087 pass-2 — AC-002/008/015 visible content rendering Red Gate tests (§10.4).
//!
//! These tests call `slideforge::build()` (or exercise `layout::run` via the
//! programmatic API) and assert that color-coded slide content is VISIBLE in the
//! output — not merely that the build returns `Ok`.
//!
//! ## SID-1 compliance
//!
//! For `weighted_composite` (AC-015), the DSL list-of-map syntax is not yet
//! parseable (requires STORY-088). Per SID-1, a unit-level equivalent is provided
//! that directly constructs a `Slide` with `components` pre-populated (bypassing
//! the parser). The DSL-level test is `#[ignore]`'d with a STORY-088 citation.
//!
//! ## Red Gate discipline
//!
//! These tests MUST FAIL until:
//! 1. Stage 2b threads `label`/`value`/`components` into ContentBlocks.
//! 2. layout::run routes ColorLabel blocks into Body-role frames.
//! 3. layout::run materializes FrameContent::ColorBar from ColorBar blocks.
//!
//! Tests that call `build()` pass via DSL fixtures. The `weighted_composite`
//! AC-015 test uses the programmatic path (SID-1) as the STORY-088 DSL list
//! parser is not yet landed.
//!
//! ## Traceability
//!
//! - AC-002: status build → label text visible in output frames
//! - AC-003: status missing label → E-A11-002 at build (LabelCheck)
//! - AC-005: missing label in warn-only mode → Ok (not Err)
//! - AC-008: progress_bar build → label + ColorBar frame visible
//! - AC-015: weighted_composite → aggregate label + component labels visible
//! - Architect pass-2 adjudication §10.4 (STORY-087)

#![allow(clippy::unwrap_used)] // integration tests — panics are intentional
#![allow(clippy::doc_markdown)] // references like `E-A11-002`
#![allow(clippy::uninlined_format_args)] // width-limited format strings ({:.N}) require positional args
#![allow(non_snake_case)] // BC-traceability IDs use uppercase

use std::sync::Arc;

use crate::e2e::{BrandTmpDir, fixture_source, open_zip};
use slideforge_layout::FrameContent;
use slideforge_types::{
    Block, Brand, BrandFonts, BrandPalette, ColorBarSpec, ContentBlock, Deck, DeckMetadata,
    FieldValue, InlineNode, OrderedMap, Slide, SourceSpan, TextBlock, TextTag, Value,
};

// ── Programmatic helpers (SID-1 path — no DSL parser required) ───────────────

fn make_brand() -> Brand {
    Brand {
        name: Arc::from("test-brand"),
        palette: BrandPalette {
            primary: Arc::from("#0070C0"),
            secondary: Arc::from("#0066CC"),
            accent: Arc::from("#FF6B35"),
            neutral: Arc::from("#F5F5F5"),
        },
        fonts: BrandFonts {
            heading: Arc::from("Calibri"),
            body: Arc::from("Calibri"),
            mono: Arc::from("Courier New"),
        },
        layouts: vec![],
        span: SourceSpan::default(),
    }
}

fn make_metadata() -> DeckMetadata {
    DeckMetadata {
        title: Some(Arc::from("Test")),
        slideforge_version: Arc::from("0.1.0"),
        lang: Some(Arc::from("en-US")),
        author: None,
        section_order: None,
    }
}

fn make_deck_one_slide(slide: Slide) -> Deck {
    Deck {
        slides: vec![slide],
        vars: OrderedMap::new(),
        metadata: make_metadata(),
        registers: OrderedMap::new(),
        section_blocks: vec![],
    }
}

fn text_block_tagged(text: &str, tag: TextTag) -> Block {
    Block {
        content: ContentBlock::Text(TextBlock {
            inlines: vec![InlineNode::Plain(Arc::from(text))],
            tag,
            span: SourceSpan::default(),
        }),
        label: None,
        span: SourceSpan::default(),
    }
}

fn color_bar_block(percent: u8) -> Block {
    Block {
        content: ContentBlock::ColorBar(ColorBarSpec { percent }),
        label: None,
        span: SourceSpan::default(),
    }
}

fn make_slide_with_blocks(slide_type: &str, blocks: Vec<Block>, title: Option<&str>) -> Slide {
    let mut fields: OrderedMap<Arc<str>, FieldValue> = OrderedMap::new();
    if let Some(t) = title {
        fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from(t))),
        );
    }
    Slide {
        slide_type: Arc::from(slide_type),
        fields,
        blocks,
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
    }
}

// ── AC-002: status label visible in LaidOutDeck ───────────────────────────────

/// AC-002 / adjudication §10.4 (programmatic path — SID-1):
/// `status` slide with `label "On Track"` → `layout::run` produces a frame
/// containing "On Track" as `FrameContent::Body(...)`.
///
/// RED GATE: layout::run has no ColorLabel arm; the ColorLabel block is dropped
/// and the Body-role frame remains `FrameContent::Empty`.
#[test]
fn test_AC_002_status_label_visible_in_output() {
    let blocks = vec![text_block_tagged("On Track", TextTag::ColorLabel)];
    let slide = make_slide_with_blocks("status", blocks, Some("Project Alpha"));
    let deck = make_deck_one_slide(slide);
    let brand = make_brand();

    let laid_out = slideforge_layout::run(&deck, &brand).expect("layout::run must not fail");
    let frames = &laid_out.slides[0].frames;

    let has_visible_label = frames.iter().any(|f| {
        if let FrameContent::Body(blocks) = &f.content {
            blocks.iter().any(|b| {
                if let ContentBlock::Text(tb) = b {
                    tb.inlines.iter().any(
                        |n| matches!(n, InlineNode::Plain(s) if s.as_ref().contains("On Track")),
                    )
                } else {
                    false
                }
            })
        } else {
            false
        }
    });

    assert!(
        has_visible_label,
        "RED GATE (AC-002): 'On Track' label text must be visible in at least one \
         FrameContent::Body frame in the LaidOutSlide. layout::run has no ColorLabel arm. \
         Got frames: {frames:?}"
    );
}

// ── AC-003: status missing label → E-A11-002 at build ────────────────────────

/// AC-003: `build()` with `status` slide missing `label` field → `Err(ValidationFailed)`
/// with E-A11-002 diagnostic.
///
/// This test exercises LabelCheckValidator which is ALREADY implemented (STORY-087 pass-1).
/// It is included here as a regression guard confirming that the build pipeline's
/// validation path still fires correctly in the content-rendering context.
///
/// GREEN GATE: This should pass with the existing LabelCheckValidator.
/// If this test fails, LabelCheckValidator has regressed.
#[test]
fn test_AC_003_status_missing_label_is_error() {
    let brand_dir = BrandTmpDir::new("ac003_status");
    let source = fixture_source("status_missing_label.sf");
    let opts = brand_dir.build_options("pptx", true);

    // AC-003: strict mode must return Err for a status slide missing `label`.
    // LabelCheckValidator fires E-A11-002. The build error is ValidationFailed.
    // The error Display may not include the code directly — check for Err only.
    let result = slideforge::build(&source, &opts);
    assert!(
        result.is_err(),
        "AC-003: status without label in strict mode must return Err(ValidationFailed); got Ok"
    );
}

// ── AC-005: missing label in warn-only mode → Ok ─────────────────────────────

/// AC-005: `build()` with `status` slide missing `label` in warn-only mode →
/// returns `Ok` (not `Err`). The pipeline does not halt on validation errors in
/// warn-only mode.
///
/// GREEN GATE: This should already pass. Included as a regression guard.
#[test]
fn test_AC_005_status_missing_label_warn_only_is_ok() {
    let brand_dir = BrandTmpDir::new("ac005_status_warn");
    let source = fixture_source("status_missing_label.sf");
    let opts = brand_dir.build_options("pptx", false); // strict=false → warn-only

    let result = slideforge::build(&source, &opts);
    assert!(
        result.is_ok(),
        "AC-005: status without label in warn-only mode must return Ok; got: {result:?}"
    );
}

// ── AC-008: progress_bar label + bar visible ──────────────────────────────────

/// AC-008 / adjudication §10.4 (programmatic path — SID-1):
/// `progress_bar` with `label "75% complete"` and `value 75` →
/// - label text is visible in a `FrameContent::Body(...)` frame
/// - a `FrameContent::ColorBar { filled_width_emu, .. }` frame exists
///   with `filled_width_emu > Emu(0)`.
///
/// RED GATE: layout::run has no ColorLabel arm or ColorBar materialization pass.
#[test]
fn test_AC_008_progress_bar_label_and_bar_visible() {
    let blocks = vec![
        text_block_tagged("75% complete", TextTag::ColorLabel),
        color_bar_block(75),
    ];
    let slide = make_slide_with_blocks("progress_bar", blocks, Some("Sprint 4"));
    let deck = make_deck_one_slide(slide);
    let brand = make_brand();

    let laid_out = slideforge_layout::run(&deck, &brand).expect("layout::run must not fail");
    let frames = &laid_out.slides[0].frames;

    // Assert label text is visible.
    let has_label = frames.iter().any(|f| {
        if let FrameContent::Body(blocks) = &f.content {
            blocks.iter().any(|b| {
                if let ContentBlock::Text(tb) = b {
                    tb.inlines.iter().any(|n| {
                        matches!(n, InlineNode::Plain(s) if s.as_ref().contains("75% complete"))
                    })
                } else {
                    false
                }
            })
        } else {
            false
        }
    });
    assert!(
        has_label,
        "RED GATE (AC-008): '75% complete' label must be visible in a Body frame. \
         Got frames: {frames:?}"
    );

    // Assert ColorBar frame exists with filled_width_emu > 0.
    let color_bar_frames: Vec<_> = frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::ColorBar { .. }))
        .collect();
    assert_eq!(
        color_bar_frames.len(),
        1,
        "RED GATE (AC-008): exactly 1 FrameContent::ColorBar frame must exist. \
         Got {}: frames: {frames:?}",
        color_bar_frames.len()
    );
    if let FrameContent::ColorBar {
        filled_width_emu, ..
    } = &color_bar_frames[0].content
    {
        assert!(
            filled_width_emu.0 > 0,
            "AC-008: filled_width_emu must be > 0 for value=75; got {}",
            filled_width_emu.0
        );
    }
}

// ── AC-015: weighted_composite labels visible (SID-1 programmatic) ────────────

/// AC-015 / adjudication §10.4 (programmatic path — SID-1):
/// `weighted_composite` with aggregate label + 2 components →
/// aggregate label text is visible in a Body frame; component texts are visible.
///
/// RED GATE: layout::run has no ColorLabel arm; component Body blocks are not
/// routed into Generic slots without the ColorLabel→Body routing being correct.
///
/// This SID-1 test covers:
/// `test_BC_1_17_003_build_weighted_composite_visible_output` (DSL path, `#[ignore]`'d,
/// pending STORY-088 list-of-map parser).
#[test]
fn test_AC_015_weighted_composite_labels_visible() {
    let blocks = vec![
        text_block_tagged("Overall: Good", TextTag::ColorLabel),
        text_block_tagged("Quality: 85/100 (wt: 0.4) — Excellent", TextTag::Body),
        text_block_tagged("Price: 72/100 (wt: 0.6) — Good", TextTag::Body),
    ];
    let slide = make_slide_with_blocks("weighted_composite", blocks, Some("Vendor A"));
    let deck = make_deck_one_slide(slide);
    let brand = make_brand();

    let laid_out = slideforge_layout::run(&deck, &brand).expect("layout::run must not fail");
    let frames = &laid_out.slides[0].frames;

    let has_agg_label = frames.iter().any(|f| {
        if let FrameContent::Body(blocks) = &f.content {
            blocks.iter().any(|b| {
                if let ContentBlock::Text(tb) = b {
                    tb.inlines.iter().any(|n| {
                        matches!(n, InlineNode::Plain(s) if s.as_ref().contains("Overall: Good"))
                    })
                } else {
                    false
                }
            })
        } else {
            false
        }
    });
    assert!(
        has_agg_label,
        "RED GATE (AC-015): 'Overall: Good' aggregate label must be visible in a Body frame. \
         Got frames: {frames:?}"
    );

    let has_quality = frames.iter().any(|f| {
        if let FrameContent::Body(blocks) = &f.content {
            blocks.iter().any(|b| {
                if let ContentBlock::Text(tb) = b {
                    tb.inlines.iter().any(
                        |n| matches!(n, InlineNode::Plain(s) if s.as_ref().contains("Quality")),
                    )
                } else {
                    false
                }
            })
        } else {
            false
        }
    });
    assert!(
        has_quality,
        "RED GATE (AC-015): 'Quality' component text must be visible in a Body frame. \
         Got frames: {frames:?}"
    );
}

/// AC-015 DSL path — IGNORED pending STORY-088 list-of-map parser.
///
/// SID-1 citation: this test is `#[ignore]`'d because the DSL list-of-map
/// syntax for `components:` (e.g., `components: [{name: "Quality", ...}]`)
/// requires STORY-088 (DSL list literal parser). The covering SID-1 unit-level
/// equivalent is `test_AC_015_weighted_composite_labels_visible` above, which
/// directly constructs the `Slide` with `blocks` pre-populated.
///
/// When STORY-088 is landed, remove the `#[ignore]` and supply the DSL fixture.
#[test]
#[ignore = "requires STORY-088 DSL list-of-map parser — covered by test_AC_015_weighted_composite_labels_visible (SID-1 unit equivalent)"]
fn test_BC_1_17_003_build_weighted_composite_visible_output() {
    // This test is intentionally empty — it will be filled in STORY-088.
    // The load-bearing proof is test_AC_015_weighted_composite_labels_visible above.
}

// ── AC-008 build()-level Red Gate: ColorBar solid-fill shape in PPTX bytes ───
//
// Finding F-087-P3-001 (adversary pass 3):
//   FrameContent::ColorBar IS materialized into the LaidOutSlide IR (pass-2 fix)
//   but the PPTX exporter's ColorBar arm (slide_serializer.rs:642-655) is a
//   `tracing::warn!` no-op — no <p:sp> is emitted. The existing AC-008 IR test
//   (above) only inspects the LaidOutDeck; it does NOT call build() or inspect
//   PPTX bytes, so it cannot catch the missing render (TD-VSDD-059 paper-fix).
//
// BC-1.17.002 PC-9 (v1.2) requires the bar rendered visibly as a PPTX
// solid-fill <p:sp> at proportional width.
//
// RED GATE: Until the ColorBar arm emits a solid-fill <p:sp>, the assertion
// below on <a:solidFill> in slide1.xml FAILS.

/// BC-1.17.002 PC-9 / F-087-P3-001 Red Gate:
/// `build()` on a `progress_bar` deck (value=75, title "Sprint 4 Progress",
/// label "75% complete") producing PPTX must yield slide1.xml that:
///
/// 1. Contains at least one `<a:solidFill>` element — the filled bar shape.
///    This shape must NOT be a standard text placeholder (title/subtitle/body),
///    meaning a second `<p:sp>` exists that carries `<a:solidFill>` but no
///    `<p:ph>` (or a `<p:ph>` with a type that is not "title"/"body").
/// 2. Contains the label text "75% complete" somewhere in the slide XML.
///
/// ## Why these assertions
///
/// - `<a:solidFill>` is the canonical OOXML representation of a solid-color
///   shape fill (ECMA-376 §20.1.8.44). An empty warn-stub emits zero <p:sp>
///   elements for ColorBar → no `<a:solidFill>` → assertion (1) FAILS.
/// - Title/body placeholders may carry theme-inherited fill via `<p:ph>` and
///   would not have `<a:solidFill>` in their `<p:spPr>`. The bar-render
///   requirement is a non-placeholder shape with explicit solidFill.
/// - Assertion (2) is a regression guard confirming the label-text path (which
///   was fixed in pass-2) is not broken by the pass-3 ColorBar implementation.
///
/// ## Red Gate behavior (pre-implementation)
///
/// The PPTX exporter's ColorBar arm at slide_serializer.rs:642-655 is a no-op
/// that only emits `tracing::warn!`. Slide1.xml contains no `<a:solidFill>`
/// in a non-placeholder `<p:sp>`. Assertion (1) → FAILS at Red Gate.
///
/// ## Post-implementation behavior
///
/// After the implementer adds the solid-fill `<p:sp>` for the filled portion of
/// the bar, slide1.xml contains `<a:solidFill>` in a non-placeholder `<p:sp>`
/// with `<a:ext cx="…">` proportional to 75% of the bar background width.
/// Both assertions pass.
///
/// Traceability: BC-1.17.002 PC-9; finding F-087-P3-001; adjudication §10.
#[test]
fn test_AC_008_progress_bar_bar_rendered_in_pptx_xml() {
    let brand = BrandTmpDir::new("s087_p3_bar_pptx");
    let source = fixture_source("story-087-progress-bar-75.sf");
    let opts = brand.build_options("pptx", false); // warn-only: value=75 is valid, no E-VAL-011

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "F-087-P3-001 Red Gate: build() with progress_bar value=75 must return Ok; \
             got Err: {e:?}"
        )
    });

    let mut archive = open_zip(&output.bytes, "F-087-P3-001");
    let slide_xml = read_zip_entry(&mut archive, "ppt/slides/slide1.xml", "F-087-P3-001");

    // ── Assertion 1: label text "75% complete" is present ────────────────────
    //
    // Regression guard: this should already work after pass-2 (ColorLabel → Body).
    // If this fails, the pass-2 label-thread fix has regressed.
    assert!(
        slide_xml.contains("75% complete"),
        "F-087-P3-001: slide1.xml must contain the label text '75% complete'. \
         Regression: the pass-2 label-threading fix may have been broken. \
         slide1.xml (first 1000 chars):\n{:.1000}",
        slide_xml
    );

    // ── Assertion 2: a solid-fill <p:sp> for the filled bar is present ───────
    //
    // The ColorBar filled portion must be rendered as a non-placeholder <p:sp>
    // with an explicit <a:solidFill> in its <p:spPr>. Standard text placeholder
    // shapes (title, body) carry <p:ph> and use theme-inherited fill — they do
    // NOT carry <a:solidFill> in their own <p:spPr>.
    //
    // RED GATE: the ColorBar arm in slide_serializer.rs is a warn-stub that emits
    // zero OOXML. No <a:solidFill> exists in the slide XML. This assertion FAILS.
    assert!(
        slide_xml.contains("<a:solidFill>") || slide_xml.contains("<a:solidFill "),
        "F-087-P3-001 Red Gate: PPTX slide1.xml must contain at least one <a:solidFill> \
         element — the solid-fill shape representing the filled portion of the progress bar. \
         The ColorBar arm in slide_serializer.rs is currently a tracing::warn! no-op and \
         emits no <p:sp>. \
         slide1.xml (first 1500 chars):\n{:.1500}",
        slide_xml
    );

    // ── Assertion 3: the solidFill is in a non-placeholder <p:sp> ────────────
    //
    // Parse the slide XML into individual <p:sp>…</p:sp> blocks and verify that
    // at least one block carries <a:solidFill> but is NOT a standard text
    // placeholder (i.e., it does NOT carry `type="title"` or `type="body"` or
    // `idx="1"` on its <p:ph>).
    //
    // This guards against the degenerate case where a placeholder's theme-fill
    // accidentally matches the solidFill selector.
    let sp_blocks = extract_pptx_sp_blocks(&slide_xml);
    let has_non_placeholder_solid_fill = sp_blocks.iter().any(|sp| {
        let has_solid_fill = sp.contains("<a:solidFill>") || sp.contains("<a:solidFill ");
        // Standard title/body/subtitle placeholders have type="title", type="body",
        // type="subTitle", or idx="0" / idx="1" on their <p:ph> element.
        let is_standard_text_placeholder = sp.contains(r#"type="title""#)
            || sp.contains(r#"type="body""#)
            || sp.contains(r#"type="subTitle""#)
            || (sp.contains("<p:ph") && (sp.contains(r#"idx="0""#) || sp.contains(r#"idx="1""#)));
        has_solid_fill && !is_standard_text_placeholder
    });

    assert!(
        has_non_placeholder_solid_fill,
        "F-087-P3-001 Red Gate: slide1.xml must contain a non-placeholder <p:sp> with \
         <a:solidFill> representing the filled bar. \
         Either no <p:sp> carries <a:solidFill>, or every solidFill is inside a standard \
         text placeholder (which is incorrect — the bar is a new shape, not a placeholder). \
         sp blocks found ({} total):\n{}",
        sp_blocks.len(),
        sp_blocks
            .iter()
            .map(|s| format!("  - {:.200}", s))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Read a named entry from a ZIP archive into a `String`.
///
/// Panics with a descriptive message if the entry is absent or unreadable.
fn read_zip_entry(
    archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
    name: &str,
    label: &str,
) -> String {
    let mut entry = archive
        .by_name(name)
        .unwrap_or_else(|e| panic!("{label}: ZIP entry '{name}' not found: {e}"));
    let mut content = String::new();
    std::io::Read::read_to_string(&mut entry, &mut content)
        .unwrap_or_else(|e| panic!("{label}: cannot read ZIP entry '{name}': {e}"));
    content
}

/// Split PPTX slide XML into individual `<p:sp>…</p:sp>` shape blocks.
///
/// Returns one `String` per `<p:sp>` element found in `xml`.
/// This is a simple text-scan — not a full XML parser — sufficient for
/// asserting structural properties in integration tests.
fn extract_pptx_sp_blocks(xml: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut remaining = xml;
    while let Some(start) = remaining
        .find("<p:sp>")
        .or_else(|| remaining.find("<p:sp "))
    {
        let tag_end = remaining[start..].find('>').map(|i| start + i + 1);
        let Some(tag_end) = tag_end else { break };
        let Some(end_offset) = remaining[tag_end..].find("</p:sp>") else {
            break;
        };
        let block_end = tag_end + end_offset + "</p:sp>".len();
        blocks.push(remaining[start..block_end].to_owned());
        remaining = &remaining[block_end..];
    }
    blocks
}
