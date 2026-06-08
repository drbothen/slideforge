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
            font_size_emu: 457_200,
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

/// BC-1.17.003 PC-9 / F-087-P7-001:
/// `weighted_composite` with 6 component Body blocks MUST cap at 5 rendered rows.
/// The 6th component exceeds the 5 pre-allocated Generic-role slots; it must be
/// silently dropped — no additional/stray frame is appended.
///
/// ## RED GATE
///
/// The current `field_to_block.rs` threading iterates ALL components with no
/// `.take(5)` cap (lines ~336-345). When 6 Body blocks arrive at `layout::run`,
/// the 6th exhausts all 5 Generic-role Empty slots and falls through to
/// `fill_region_slot_or_append` Phase 3 (layout.rs ~870-898), which APPENDS a
/// stray full-page-bbox frame with:
///   - bbox: x=Emu(0), y=Emu(0), width=page_width, height=Emu(914_400)
///   - region_role: None
///
/// This test fails until the cap (`.take(5)`) is applied at threading
/// (`field_to_block.rs`) or at layout-slot fill (layout.rs Generic fallback).
///
/// ## Assertions (both load-bearing, not "doesn't panic")
///
/// 1. Exactly 5 `FrameContent::Body` frames with `region_role == Generic` exist
///    (the 6th component row must NOT appear).
/// 2. No frame has `bbox.x == Emu(0) && bbox.y == Emu(0) && region_role == None`
///    (i.e., no Phase-3 stray full-page-bbox frame was appended).
///
/// Traceability: BC-1.17.003 PC-9, F-087-P7-001.
#[test]
#[allow(non_snake_case)]
fn test_BC_1_17_003_pc9_six_components_caps_at_5_no_stray_frame() {
    // Build 1 ColorLabel (aggregate) + 6 component Body blocks — one more than
    // the 5 Generic-role slots pre-allocated for weighted_composite.
    let mut blocks = vec![text_block_tagged("Overall: Good", TextTag::ColorLabel)];
    for i in 1..=6 {
        blocks.push(text_block_tagged(
            &format!("Component {i}: 80/100 (wt: 0.17) — Good"),
            TextTag::Body,
        ));
    }
    let slide = make_slide_with_blocks("weighted_composite", blocks, Some("Vendor A"));
    let deck = make_deck_one_slide(slide);
    let brand = make_brand();

    let laid_out = run(&deck, &brand).expect("layout::run must not fail");
    let frames = &laid_out.slides[0].frames;

    // Assertion 1: exactly 5 Generic-role Body frames (not 6).
    // The ColorLabel fills the Body-role slot; each component Body fills a Generic slot.
    // If a 6th component slips through, the Generic-slot count would still be 5 but
    // an extra Phase-3 frame (region_role: None) would be appended — caught by
    // Assertion 2. We verify the Generic Body count is exactly 5.
    let generic_body_frames: Vec<_> = frames
        .iter()
        .filter(|f| {
            matches!(f.content, FrameContent::Body(_))
                && f.region_role == Some(slideforge_layout::RegionRole::Generic)
        })
        .collect();

    assert_eq!(
        generic_body_frames.len(),
        5,
        "RED GATE (BC-1.17.003 PC-9 / F-087-P7-001): \
         weighted_composite with 6 component Body blocks must produce EXACTLY 5 \
         Generic-role Body frames (5-slot cap), not {}. \
         All frames: {frames:?}",
        generic_body_frames.len()
    );

    // Assertion 2: no stray Phase-3 full-page-bbox frame appended.
    // Phase-3 fallback frames have region_role == None and bbox at (0,0).
    // Any such frame indicates the 6th component overflowed the slot list.
    let stray_frames: Vec<_> = frames
        .iter()
        .filter(|f| {
            f.region_role.is_none()
                && f.bbox.x == slideforge_types::Emu(0)
                && f.bbox.y == slideforge_types::Emu(0)
                && matches!(f.content, FrameContent::Body(_))
        })
        .collect();

    assert_eq!(
        stray_frames.len(),
        0,
        "RED GATE (BC-1.17.003 PC-9 / F-087-P7-001): \
         a stray Phase-3 full-page-bbox frame was appended for the 6th component \
         (region_role=None, bbox origin at (0,0)). The threading cap (.take(5)) \
         is missing. Stray frame(s): {stray_frames:?}. All frames: {frames:?}"
    );
}

// ── OBS-P6-001 — status slide Title frame invariants ─────────────────────────
//
// The `status` region map (regions.rs "status" arm) allocates only:
//   Frame 0: RegionRole::Generic  — the 0.75in (685,800 EMU) color indicator strip
//   Frame 1: RegionRole::Body     — the label/title body
//
// There is NO RegionRole::Title frame. `fill_region_slot_or_append` for a
// TextTag::Title block therefore falls through to Phase 2 (Generic fallback) and
// routes the title text into Frame 0 — the narrow color indicator strip — instead
// of a real title slot.
//
// These three tests assert the CORRECT post-fix behavior (invariants):
//   1. A FrameContent::Title frame exists AND its width > 685_800 EMU.
//   2. The narrow Generic (≤685_800 EMU wide) frame does NOT carry FrameContent::Title.
//   3. The title text in the FrameContent::Title frame matches the input.
//
// ALL THREE MUST FAIL now (title currently goes into the 685_800 EMU strip).
// Traceability: OBS-P6-001 / BC-1.17.001.

/// OBS-P6-001 assertion 1: the frame carrying FrameContent::Title on a `status`
/// slide must be wider than the color indicator strip (> 685_800 EMU).
///
/// RED GATE: currently the title is routed into the 0.75in (685_800 EMU) strip
/// (Frame 0, RegionRole::Generic) via fill_region_slot_or_append Phase-2 fallback,
/// because the status region map has no RegionRole::Title frame.
#[test]
#[allow(non_snake_case)]
fn test_OBS_P6_001_status_title_frame_width_exceeds_color_strip() {
    // Build a status slide with a title block and a ColorLabel block.
    let blocks = vec![
        text_block_tagged("Status Title Text", TextTag::Title),
        text_block_tagged("On Track", TextTag::ColorLabel),
    ];
    let slide = make_slide_with_blocks("status", blocks, Some("Status Title Text"));
    let deck = make_deck_one_slide(slide);
    let brand = make_brand();

    let laid_out = run(&deck, &brand).expect("layout::run must not fail for status slide");
    let frames = &laid_out.slides[0].frames;

    // The color indicator strip width in EMU at the default page size:
    //   sx(685_800) = 685_800 * 9_144_000 / 9_144_000 = 685_800 EMU (0.75in)
    // A real title slot must be substantially wider than this.
    let color_strip_width_emu: i64 = 685_800;

    // Find the FrameContent::Title frame.
    let title_frames: Vec<_> = frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::Title(_)))
        .collect();

    assert_eq!(
        title_frames.len(),
        1,
        "OBS-P6-001 RED GATE: status slide with TextTag::Title block must produce \
         exactly 1 FrameContent::Title frame. Got {} Title frames. \
         All frames: {frames:?}",
        title_frames.len()
    );

    let title_frame = title_frames[0];
    assert!(
        title_frame.bbox.width.0 > color_strip_width_emu,
        "OBS-P6-001 RED GATE: the FrameContent::Title frame on a status slide must be \
         WIDER than the color indicator strip ({color_strip_width_emu} EMU). \
         Got width = {} EMU. \
         Currently the title is routed into the 0.75in Generic strip (Frame 0) via \
         fill_region_slot_or_append Phase-2 fallback because no RegionRole::Title \
         frame exists in the status region map. \
         All frames: {frames:?}",
        title_frame.bbox.width.0
    );
}

/// OBS-P6-001 assertion 2: the narrow color indicator frame (Generic role,
/// ≤ 685_800 EMU wide) must NOT contain the slide title.
///
/// RED GATE: currently fill_region_slot_or_append Phase-2 fallback routes
/// TextTag::Title into Frame 0 (the 0.75in Generic strip), so the color strip
/// carries FrameContent::Title instead of remaining FrameContent::Empty.
#[test]
#[allow(non_snake_case)]
fn test_OBS_P6_001_status_color_strip_does_not_contain_title() {
    let color_strip_width_emu: i64 = 685_800;

    let blocks = vec![
        text_block_tagged("Status Title Text", TextTag::Title),
        text_block_tagged("On Track", TextTag::ColorLabel),
    ];
    let slide = make_slide_with_blocks("status", blocks, Some("Status Title Text"));
    let deck = make_deck_one_slide(slide);
    let brand = make_brand();

    let laid_out = run(&deck, &brand).expect("layout::run must not fail for status slide");
    let frames = &laid_out.slides[0].frames;

    // Find any frame narrower than or equal to the color strip width that carries Title.
    let title_in_strip: Vec<_> = frames
        .iter()
        .filter(|f| {
            f.bbox.width.0 <= color_strip_width_emu && matches!(f.content, FrameContent::Title(_))
        })
        .collect();

    assert_eq!(
        title_in_strip.len(),
        0,
        "OBS-P6-001 RED GATE: the narrow color indicator frame \
         (width ≤ {color_strip_width_emu} EMU) must NOT carry FrameContent::Title. \
         Found {} narrow frame(s) with Title content. \
         This means the title was routed into the 0.75in strip (the Phase-2 Generic \
         fallback in fill_region_slot_or_append fired because no RegionRole::Title \
         slot exists in the status region map). \
         All frames: {frames:?}",
        title_in_strip.len()
    );

    // Additionally: the narrow Generic frame must remain FrameContent::Empty
    // (available for exporters to render as the color indicator).
    let narrow_generic_frames: Vec<_> = frames
        .iter()
        .filter(|f| {
            f.bbox.width.0 <= color_strip_width_emu
                && f.region_role == Some(slideforge_layout::RegionRole::Generic)
        })
        .collect();

    assert!(
        !narrow_generic_frames.is_empty(),
        "OBS-P6-001: status slide must still produce at least one narrow Generic frame \
         (the color indicator slot). Got frames: {frames:?}"
    );

    for narrow_frame in &narrow_generic_frames {
        assert!(
            matches!(narrow_frame.content, FrameContent::Empty),
            "OBS-P6-001 RED GATE: the narrow Generic frame (width ≤ {color_strip_width_emu} EMU) \
             must remain FrameContent::Empty (the color indicator slot must be available for \
             exporters). Got content: {:?}. All frames: {frames:?}",
            narrow_frame.content
        );
    }
}

/// OBS-P6-001 assertion 3: the FrameContent::Title frame carries the exact
/// title text from the input AND resides in a frame wider than the color strip.
///
/// RED GATE: currently the title is routed into the 0.75in (685_800 EMU) narrow
/// strip, which fails assertion 1. This test adds the text-content assertion on
/// TOP of the width check, so both the location and text are verified together.
/// The COMBINED assertion (width > strip AND text matches) is the Red Gate: it
/// fails now because the only Title frame is the narrow strip.
#[test]
#[allow(non_snake_case)]
fn test_OBS_P6_001_status_title_text_matches_input() {
    let expected_title = "My Status Slide Title";
    let color_strip_width_emu: i64 = 685_800;

    let blocks = vec![
        text_block_tagged(expected_title, TextTag::Title),
        text_block_tagged("At Risk", TextTag::ColorLabel),
    ];
    let slide = make_slide_with_blocks("status", blocks, Some(expected_title));
    let deck = make_deck_one_slide(slide);
    let brand = make_brand();

    let laid_out = run(&deck, &brand).expect("layout::run must not fail for status slide");
    let frames = &laid_out.slides[0].frames;

    // Find FrameContent::Title frames that are WIDE ENOUGH to be a real title slot
    // (wider than the color strip). This is the combined assertion: correct text
    // AND correct location. The narrow-strip Title frame does NOT satisfy this.
    let wide_title_frames: Vec<_> = frames
        .iter()
        .filter(|f| {
            matches!(f.content, FrameContent::Title(_)) && f.bbox.width.0 > color_strip_width_emu
        })
        .collect();

    assert_eq!(
        wide_title_frames.len(),
        1,
        "OBS-P6-001 RED GATE: status slide must produce exactly 1 FrameContent::Title \
         frame whose width > {color_strip_width_emu} EMU (not the narrow strip). \
         Got {} wide Title frames. \
         (Currently the only Title frame IS the narrow strip at width=685800 EMU, \
         so this assertion fails until a real Title slot is added.) \
         All frames: {frames:?}",
        wide_title_frames.len()
    );

    if let FrameContent::Title(text) = &wide_title_frames[0].content {
        assert_eq!(
            text.as_ref(),
            expected_title,
            "OBS-P6-001: the wide FrameContent::Title frame must carry the exact \
             input text '{expected_title}', got '{text}'"
        );
    }
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
