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
#![allow(non_snake_case)] // BC-traceability IDs use uppercase

use std::sync::Arc;

use crate::e2e::{BrandTmpDir, fixture_source};
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
