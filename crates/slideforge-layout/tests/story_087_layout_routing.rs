//! STORY-087 pass-2 — layout routing Red Gate tests (§10.2).
//!
//! These tests verify that `layout::run` correctly routes color-coded slide
//! content blocks into the pre-allocated region frames and materializes
//! `FrameContent::ColorBar` from `ContentBlock::ColorBar(ColorBarSpec)`.
//!
//! ## Red Gate discipline
//!
//! ALL tests in this file MUST FAIL until the implementer:
//! 1. Extends Stage 2b (thread_fields_to_blocks) to produce ColorLabel + ColorBar blocks.
//! 2. Adds the ColorLabel arm to layout::run's TextTag routing.
//! 3. Adds the ColorBar materialization pass in layout::run.
//!
//! The tests directly construct `Slide.blocks` (bypassing Stage 2b) per SID-1
//! discipline: layout routing tests should not depend on Stage 2b being implemented
//! — they verify the layout pipeline in isolation.
//!
//! ## Traceability
//!
//! - BC-1.17.001 PC-8: status label → Body-role frame visible
//! - BC-1.17.002 PC-9: progress_bar label → Body-role frame; value → ColorBar frame
//! - BC-1.17.003 PC-9: weighted_composite label + components → frames
//! - Architect pass-2 adjudication §10.2 (STORY-087)

#![allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::doc_markdown
)]

use std::sync::Arc;

use slideforge_layout::{FrameContent, run};
use slideforge_types::{
    Block, Brand, BrandFonts, BrandPalette, ColorBarSpec, ContentBlock, Deck, DeckMetadata, Emu,
    FieldValue, InlineNode, OrderedMap, Slide, SourceSpan, TextBlock, TextTag, Value,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_brand() -> Brand {
    Brand {
        name: Arc::from("test-brand"),
        palette: BrandPalette {
            primary: Arc::from("#003087"),
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

/// Build a minimal slide with the given type keyword and a pre-populated `blocks` list.
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

// ── §10.2 Layout routing — status ────────────────────────────────────────────

/// BC-1.17.001 PC-8 / adjudication §10.2:
/// `layout::run` with `status` slide having a `ColorLabel` block in `blocks` →
/// the Body-role frame becomes `FrameContent::Body(...)` containing "On Track".
/// No `FrameContent::Empty` remains in the Body slot.
///
/// RED GATE: layout::run does not yet handle TextTag::ColorLabel — it has no
/// arm for it in the ContentBlock::Text match, so the ColorLabel block is
/// currently dropped (falls through to `_ => {}`).
#[test]
fn test_status_label_fills_body_slot() {
    let blocks = vec![text_block_tagged("On Track", TextTag::ColorLabel)];
    let slide = make_slide_with_blocks("status", blocks, Some("Project Alpha"));
    let deck = make_deck_one_slide(slide);
    let brand = make_brand();

    let laid_out = run(&deck, &brand).expect("layout::run must not fail");
    let frames = &laid_out.slides[0].frames;

    // Exactly one Body-role frame should carry FrameContent::Body with "On Track".
    let body_with_label: Vec<_> = frames
        .iter()
        .filter(|f| {
            if let FrameContent::Body(blocks) = &f.content {
                blocks.iter().any(|b| {
                    if let ContentBlock::Text(tb) = b {
                        tb.inlines.iter().any(|n| {
                            matches!(n, InlineNode::Plain(s) if s.as_ref().contains("On Track"))
                        })
                    } else {
                        false
                    }
                })
            } else {
                false
            }
        })
        .collect();

    assert_eq!(
        body_with_label.len(),
        1,
        "RED GATE: status ColorLabel block must fill the Body-role slot as \
         FrameContent::Body containing 'On Track'. layout::run has no ColorLabel arm yet. \
         Got frames: {frames:?}"
    );
}

// ── §10.2 Layout routing — progress_bar label ────────────────────────────────

/// BC-1.17.002 PC-9 / adjudication §10.2:
/// `progress_bar` with `ColorLabel` block → Body-role frame (frame 2) becomes
/// `FrameContent::Body(...)` containing the label text.
///
/// RED GATE: layout::run has no ColorLabel arm.
#[test]
fn test_progress_bar_label_fills_body_slot() {
    let blocks = vec![
        text_block_tagged("75% complete", TextTag::ColorLabel),
        color_bar_block(75),
    ];
    let slide = make_slide_with_blocks("progress_bar", blocks, Some("Sprint 4"));
    let deck = make_deck_one_slide(slide);
    let brand = make_brand();

    let laid_out = run(&deck, &brand).expect("layout::run must not fail");
    let frames = &laid_out.slides[0].frames;

    let body_with_label: Vec<_> = frames
        .iter()
        .filter(|f| {
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
        })
        .collect();

    assert_eq!(
        body_with_label.len(),
        1,
        "RED GATE: progress_bar ColorLabel block must fill the Body-role slot as \
         FrameContent::Body containing '75% complete'. Got frames: {frames:?}"
    );
}

// ── §10.2 Layout routing — progress_bar ColorBar materialization ─────────────

/// BC-1.17.002 PC-9 / adjudication §10.2:
/// `progress_bar` with `ContentBlock::ColorBar { percent: 75 }` in blocks →
/// one Generic-role frame becomes `FrameContent::ColorBar { filled_width_emu, total_width_emu, .. }`
/// where `filled_width_emu == total_width_emu * 75 / 100` (integer EMU, exact).
///
/// RED GATE: layout::run has no ColorBar materialization pass yet.
#[test]
fn test_progress_bar_color_bar_width_proportional() {
    let blocks = vec![color_bar_block(75)];
    let slide = make_slide_with_blocks("progress_bar", blocks, Some("Sprint 4"));
    let deck = make_deck_one_slide(slide);
    let brand = make_brand();

    let laid_out = run(&deck, &brand).expect("layout::run must not fail");
    let frames = &laid_out.slides[0].frames;

    let color_bar_frames: Vec<_> = frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::ColorBar { .. }))
        .collect();

    assert_eq!(
        color_bar_frames.len(),
        1,
        "RED GATE: progress_bar with ColorBar block must produce exactly 1 \
         FrameContent::ColorBar frame. layout::run has no ColorBar materialization pass yet. \
         Got frames: {frames:?}"
    );

    if let FrameContent::ColorBar {
        filled_width_emu,
        total_width_emu,
        ..
    } = &color_bar_frames[0].content
    {
        let expected_filled = Emu(total_width_emu.0 * 75 / 100);
        assert_eq!(
            *filled_width_emu, expected_filled,
            "filled_width_emu must be total_width_emu * 75 / 100 = {}; got {}",
            expected_filled.0, filled_width_emu.0
        );
        assert!(
            total_width_emu.0 > 0,
            "total_width_emu must be positive (bar-background frame width)"
        );
    }
}

/// BC-1.17.002 boundary / adjudication §10.2:
/// `progress_bar` with `value 0` → `filled_width_emu == Emu(0)`.
///
/// RED GATE: layout::run has no ColorBar materialization pass yet.
#[test]
fn test_progress_bar_color_bar_value_0() {
    let blocks = vec![color_bar_block(0)];
    let slide = make_slide_with_blocks("progress_bar", blocks, Some("Sprint 4"));
    let deck = make_deck_one_slide(slide);
    let brand = make_brand();

    let laid_out = run(&deck, &brand).expect("layout::run must not fail");
    let frames = &laid_out.slides[0].frames;

    let color_bar_frames: Vec<_> = frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::ColorBar { .. }))
        .collect();

    assert_eq!(
        color_bar_frames.len(),
        1,
        "RED GATE: progress_bar value=0 must produce 1 FrameContent::ColorBar; got {frames:?}"
    );
    if let FrameContent::ColorBar {
        filled_width_emu, ..
    } = &color_bar_frames[0].content
    {
        assert_eq!(
            *filled_width_emu,
            Emu(0),
            "value=0 → filled_width_emu must be Emu(0); got {}",
            filled_width_emu.0
        );
    }
}

/// BC-1.17.002 boundary / adjudication §10.2:
/// `progress_bar` with `value 100` → `filled_width_emu == total_width_emu`.
///
/// RED GATE: layout::run has no ColorBar materialization pass yet.
#[test]
fn test_progress_bar_color_bar_value_100() {
    let blocks = vec![color_bar_block(100)];
    let slide = make_slide_with_blocks("progress_bar", blocks, Some("Sprint 4"));
    let deck = make_deck_one_slide(slide);
    let brand = make_brand();

    let laid_out = run(&deck, &brand).expect("layout::run must not fail");
    let frames = &laid_out.slides[0].frames;

    let color_bar_frames: Vec<_> = frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::ColorBar { .. }))
        .collect();

    assert_eq!(
        color_bar_frames.len(),
        1,
        "RED GATE: progress_bar value=100 must produce 1 FrameContent::ColorBar; got {frames:?}"
    );
    if let FrameContent::ColorBar {
        filled_width_emu,
        total_width_emu,
        ..
    } = &color_bar_frames[0].content
    {
        assert_eq!(
            *filled_width_emu, *total_width_emu,
            "value=100 → filled_width_emu must equal total_width_emu ({}); got filled={}",
            total_width_emu.0, filled_width_emu.0
        );
    }
}

// ── §10.2 Layout routing — weighted_composite ────────────────────────────────

/// BC-1.17.003 PC-9 / adjudication §10.2:
/// `weighted_composite` with aggregate `ColorLabel` block → Body-role frame contains label text.
///
/// RED GATE: layout::run has no ColorLabel arm.
#[test]
fn test_weighted_composite_agg_label_fills_body_slot() {
    let blocks = vec![text_block_tagged("Overall: Good", TextTag::ColorLabel)];
    let slide = make_slide_with_blocks("weighted_composite", blocks, Some("Vendor A"));
    let deck = make_deck_one_slide(slide);
    let brand = make_brand();

    let laid_out = run(&deck, &brand).expect("layout::run must not fail");
    let frames = &laid_out.slides[0].frames;

    let body_with_label: Vec<_> = frames
        .iter()
        .filter(|f| {
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
        })
        .collect();

    assert_eq!(
        body_with_label.len(),
        1,
        "RED GATE: weighted_composite ColorLabel block must fill the Body-role slot. \
         Got frames: {frames:?}"
    );
}

/// BC-1.17.003 PC-9 / adjudication §10.2:
/// `weighted_composite` with 2 component `Body` blocks → exactly 2 Generic-role slots
/// become `FrameContent::Body(...)`; 3 remaining Generic slots remain `FrameContent::Empty`.
///
/// RED GATE: layout::run already handles Body blocks via the existing Body arm,
/// so this test verifies that each Body block lands in its own Generic slot.
/// The test fails if the ColorLabel block (which also maps to Body role) consumes
/// the Body slot that should go to the first component.
#[test]
fn test_weighted_composite_2_components_fill_2_generic_slots() {
    let blocks = vec![
        text_block_tagged("Overall: Good", TextTag::ColorLabel),
        text_block_tagged("Quality: 85/100 (wt: 0.4) — Excellent", TextTag::Body),
        text_block_tagged("Price: 72/100 (wt: 0.6) — Good", TextTag::Body),
    ];
    let slide = make_slide_with_blocks("weighted_composite", blocks, Some("Vendor A"));
    let deck = make_deck_one_slide(slide);
    let brand = make_brand();

    let laid_out = run(&deck, &brand).expect("layout::run must not fail");
    let frames = &laid_out.slides[0].frames;

    // Count Body-content frames (any FrameContent::Body, regardless of role)
    // The ColorLabel fills the Body-role slot; each component Body fills a Generic slot.
    let body_frames: Vec<_> = frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::Body(_)))
        .collect();

    // 1 ColorLabel → Body slot + 2 component Body blocks → 2 Generic slots = 3 total Body frames
    assert_eq!(
        body_frames.len(),
        3,
        "RED GATE: weighted_composite with 1 ColorLabel + 2 Body blocks must produce \
         3 FrameContent::Body frames (1 for label, 2 for components). \
         Got {} Body frames; all frames: {frames:?}",
        body_frames.len()
    );

    // The 3 remaining Generic slots (for components 3–5) must remain Empty.
    let empty_generic_frames: Vec<_> = frames
        .iter()
        .filter(|f| {
            matches!(f.content, FrameContent::Empty)
                && f.region_role == Some(slideforge_layout::RegionRole::Generic)
        })
        .collect();

    assert_eq!(
        empty_generic_frames.len(),
        3,
        "RED GATE: 3 unused Generic-role slots must remain Empty; got {}. Frames: {frames:?}",
        empty_generic_frames.len()
    );
}

/// BC-1.17.003 PC-9 / adjudication §10.2:
/// 5 component blocks → all 5 Generic-role slots filled.
///
/// RED GATE: depends on layout::run routing Body blocks into Generic slots.
#[test]
fn test_weighted_composite_5_components_fill_5_generic_slots() {
    let mut blocks = vec![text_block_tagged("Overall: Excellent", TextTag::ColorLabel)];
    for i in 1..=5 {
        blocks.push(text_block_tagged(
            &format!("Component {i}: 80/100"),
            TextTag::Body,
        ));
    }
    let slide = make_slide_with_blocks("weighted_composite", blocks, Some("Vendor A"));
    let deck = make_deck_one_slide(slide);
    let brand = make_brand();

    let laid_out = run(&deck, &brand).expect("layout::run must not fail");
    let frames = &laid_out.slides[0].frames;

    // All 5 Generic-role slots should be filled.
    let empty_generic_frames: Vec<_> = frames
        .iter()
        .filter(|f| {
            matches!(f.content, FrameContent::Empty)
                && f.region_role == Some(slideforge_layout::RegionRole::Generic)
        })
        .collect();

    assert_eq!(
        empty_generic_frames.len(),
        0,
        "RED GATE: 5 component Body blocks must fill all 5 Generic-role slots. \
         Got {} empty Generic slots remaining. Frames: {frames:?}",
        empty_generic_frames.len()
    );
}
