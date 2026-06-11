//! AC-INT-1 integration test — STORY-073: `ContentBlock::Bullets` → `FrameContent::TextRun`
//!
//! This integration test file exercises the full `layout::run()` pipeline end-to-end
//! with decks containing `ContentBlock::Bullets` blocks.
//!
//! ## Acceptance criteria covered
//!
//! - **AC-INT-1**: `layout::run()` end-to-end with bullets produces correct
//!   `LaidOutDeck` output.
//! - **AC-001**: One `FrameContent::TextRun` frame per `BulletItem` in source order.
//! - **AC-002**: Unknown xref target in bullet content → `LayoutWarning::XrefTargetNotFound`.
//! - **AC-003**: Depth-65 bullet inline tree → `LayoutError::InlineDepthExceeded`.
//! - **EC-001**: Empty bullet list → zero frames, no error.
//! - **EC-002**: Bullet item with empty inline sequence → one frame, no error.
//! - **EC-003**: Nested bullet items → frame per item (not flattened).
//! - **EC-004**: Xref inside nested bold inside bullet → warning still accumulated.
//! - **EC-005**: Multiple depth violations → hard error (not bail-on-first internally).
//!
//! ## Red Gate
//!
//! At Red Gate all tests below FAIL because:
//! - `layout::run` does not iterate `ContentBlock::Bullets` to produce frames.
//! - No `FrameContent::TextRun` frames are produced for bullet items.
//! - Xref validation does not scan bullet content.
//! - Depth errors are not raised for bullet inline trees.
//!
//! ## BC traceability
//!
//! BC-3.05.001 v1.3.4 — All 12 inline format types render to correct output per format.

#![allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used,
    // doc_markdown: test doc-comments contain bare EMU numeric examples (e.g. x=457_200)
    // and math expressions that clippy pedantic would require backtick-quoting; this is
    // intentional in test documentation and does not affect runtime behavior.
    clippy::doc_markdown
)]

use std::sync::Arc;

use slideforge_layout::inline::MAX_INLINE_DEPTH;
use slideforge_layout::{FrameContent, LaidOutDeck, LayoutError, LayoutWarning, run};
use slideforge_types::{
    Block, Brand, BrandFonts, BrandPalette, BulletItem, ContentBlock, Deck, DeckMetadata,
    FieldValue, InlineNode, OrderedMap, Slide, SourceSpan, Value,
};

// ─────────────────────────────────────────────────────────────────────────────
// Test helpers (mirroring the pattern from lib.rs::tests)
// ─────────────────────────────────────────────────────────────────────────────

fn make_metadata() -> DeckMetadata {
    DeckMetadata {
        title: Some(Arc::from("Integration Test Deck")),
        slideforge_version: Arc::from("0.1.0"),
        lang: Some(Arc::from("en-US")),
        author: None,
        section_order: None,
    }
}

fn make_deck(slides: Vec<Slide>) -> Deck {
    Deck {
        slides,
        vars: OrderedMap::new(),
        metadata: make_metadata(),
        registers: OrderedMap::new(),
        section_blocks: vec![],
        slide_sections: vec![],
    }
}

#[allow(dead_code)]
fn make_slide(slide_type: &str) -> Slide {
    Slide {
        slide_type: Arc::from(slide_type),
        fields: OrderedMap::new(),
        blocks: vec![],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
        field_spans: OrderedMap::new(),
    }
}

fn make_slide_with_title(slide_type: &str, title: &str) -> Slide {
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from(title))),
    );
    Slide {
        slide_type: Arc::from(slide_type),
        fields,
        blocks: vec![],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
        field_spans: OrderedMap::new(),
    }
}

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

/// Build a `BulletItem` with the given inlines and no children.
fn flat_bullet(inlines: Vec<InlineNode>) -> BulletItem {
    BulletItem {
        inlines,
        children: vec![],
        span: SourceSpan::default(),
    }
}

/// Build a slide with one `ContentBlock::Bullets` block containing `items`.
fn bullets_slide(slide_type: &str, items: Vec<BulletItem>) -> Slide {
    let block = Block {
        content: ContentBlock::Bullets(items),
        label: None,
        span: SourceSpan::default(),
    };
    Slide {
        slide_type: Arc::from(slide_type),
        fields: OrderedMap::new(),
        blocks: vec![block],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
        field_spans: OrderedMap::new(),
    }
}

/// Return `true` if `err` is `BulletsOnContentlessSlideType` or a `Multiple` whose
/// inner vec contains at least one `BulletsOnContentlessSlideType` entry.
///
/// Used by the E-LAY-008 family of tests to verify the error invariant without
/// constraining whether the error is bare or wrapped in `Multiple` (the accumulation
/// channel introduced by error-taxonomy v2.30 §234 / DI-018).
fn is_or_contains_contentless(err: &LayoutError) -> bool {
    match err {
        LayoutError::BulletsOnContentlessSlideType { .. } => true,
        LayoutError::Multiple { inner } => inner
            .iter()
            .any(|e| matches!(e, LayoutError::BulletsOnContentlessSlideType { .. })),
        _ => false,
    }
}

/// Extract the first `BulletsOnContentlessSlideType` from `err`, whether `err` is
/// that variant directly or a `Multiple` wrapping it.
///
/// Returns `None` if `err` contains no `BulletsOnContentlessSlideType` entries.
fn extract_first_contentless(err: &LayoutError) -> Option<&LayoutError> {
    match err {
        LayoutError::BulletsOnContentlessSlideType { .. } => Some(err),
        LayoutError::Multiple { inner } => inner
            .iter()
            .find(|e| matches!(e, LayoutError::BulletsOnContentlessSlideType { .. })),
        _ => None,
    }
}

/// Count `FrameContent::TextRun` frames in a `LaidOutDeck` slide at index `slide_idx`.
fn text_run_count(deck: &LaidOutDeck, slide_idx: usize) -> usize {
    deck.slides[slide_idx]
        .frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::TextRun(_)))
        .count()
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-INT-1 — layout::run end-to-end with ContentBlock::Bullets
// ─────────────────────────────────────────────────────────────────────────────

/// AC-INT-1 / AC-001 — `layout::run` on a deck with `ContentBlock::Bullets`
/// produces one `FrameContent::TextRun` frame per `BulletItem`, in source order.
///
/// The deck contains:
/// - Slide 0: title slide (no bullets) — establishes a known xref target.
/// - Slide 1: content slide with 3 bullet items (Plain, Bold, Xref-known).
///
/// Expected: slide 1 has 3 `TextRun` frames for bullets; no warnings; slide count
/// preserved.
///
/// At Red Gate: 0 `TextRun` frames for bullet items → assertion fails.
#[test]
fn test_bc_3_05_001_story073_ac_int1_bullets_end_to_end() {
    let title_slide = make_slide_with_title("title", "introduction");
    let bullet_items = vec![
        flat_bullet(vec![InlineNode::Plain(Arc::from("plain text item"))]),
        flat_bullet(vec![InlineNode::Bold(vec![InlineNode::Plain(Arc::from(
            "bold item",
        ))])]),
        flat_bullet(vec![InlineNode::Xref(Arc::from("introduction"))]),
    ];
    let content_slide = bullets_slide("content", bullet_items);
    let deck = make_deck(vec![title_slide, content_slide]);
    let brand = make_brand();

    let result =
        run(&deck, &brand).expect("layout::run must succeed for deck with ContentBlock::Bullets");

    // AC-INT-1 assertion 1: slide count preserved.
    assert_eq!(result.slides.len(), 2, "slide count must be preserved");

    // AC-INT-1 assertion 2: 3 TextRun frames on slide 1 (the bullets slide).
    assert_eq!(
        text_run_count(&result, 1),
        3,
        "slide 1 must have 3 FrameContent::TextRun frames for 3 bullet items; \
         got {} TextRun frames (total frames on slide 1: {})",
        text_run_count(&result, 1),
        result.slides[1].frames.len()
    );

    // AC-INT-1 assertion 3: zero warnings (all known xref, depth ≤ 64).
    assert!(
        result.warnings.is_empty(),
        "well-formed bullet deck must produce zero warnings; got: {:?}",
        result.warnings
    );
}

/// AC-INT-1 / AC-001 — `TextRun` frames carry the full `Vec<InlineNode>` sequence
/// verbatim — no inline processing occurs at layout time (BC-3.05.001 invariant 6).
///
/// Verifies that the inlines for each bullet item appear unchanged in the corresponding
/// `TextRun` frame.
///
/// At Red Gate: no `TextRun` frames → no inline content to check → assertion fails.
#[test]
fn test_bc_3_05_001_story073_ac_int1_text_run_carries_inlines_verbatim() {
    let item1 = flat_bullet(vec![
        InlineNode::Plain(Arc::from("verbatim ")),
        InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold"))]),
    ]);
    let item2 = flat_bullet(vec![InlineNode::Plain(Arc::from("second item"))]);
    // F-094-P1-002: use "content" (has Body slot; "title" returns
    // Err(LayoutError::BulletsOnContentlessSlideType) / E-LAY-008).
    let slide = bullets_slide("content", vec![item1.clone(), item2.clone()]);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result =
        run(&deck, &brand).expect("layout::run must succeed for verbatim inline content test");

    let text_run_frames: Vec<Vec<InlineNode>> = result.slides[0]
        .frames
        .iter()
        .filter_map(|f| match &f.content {
            FrameContent::TextRun(nodes) => Some(nodes.clone()),
            _ => None,
        })
        .collect();

    // Both item inline sequences must appear verbatim in a TextRun frame.
    assert!(
        text_run_frames.iter().any(|nodes| nodes == &item1.inlines),
        "item1 inlines must appear verbatim in a TextRun frame; \
         got frames: {text_run_frames:?}",
    );
    assert!(
        text_run_frames.iter().any(|nodes| nodes == &item2.inlines),
        "item2 inlines must appear verbatim in a TextRun frame; \
         got frames: {text_run_frames:?}",
    );
}

/// AC-INT-1 / AC-002 — `layout::run` produces `LayoutWarning::XrefTargetNotFound`
/// for an unknown xref target inside a bullet item.
///
/// The deck contains a single title slide with one bullet containing
/// `InlineNode::Xref("missing-slide")`. "missing-slide" is not a slide title.
///
/// At Red Gate: bullet not processed → no `TextRun` frame → xref not validated →
/// no warning in LaidOutDeck.warnings → assertion fails.
#[test]
fn test_bc_3_05_001_story073_ac_int1_unknown_xref_in_bullet_warns() {
    // F-094-P1-002: use "content" (has Body slot; "title" returns
    // Err(LayoutError::BulletsOnContentlessSlideType) / E-LAY-008
    // before inline validation can fire, masking the XrefTargetNotFound warning).
    let slide = bullets_slide(
        "content",
        vec![flat_bullet(vec![InlineNode::Xref(Arc::from(
            "missing-slide",
        ))])],
    );
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result =
        run(&deck, &brand).expect("unknown xref in bullet must produce warning, not error");

    let xref_warnings: Vec<_> = result
        .warnings
        .iter()
        .filter(|w| matches!(w, LayoutWarning::XrefTargetNotFound { .. }))
        .collect();

    assert_eq!(
        xref_warnings.len(),
        1,
        "one unknown xref in bullet must produce exactly 1 XrefTargetNotFound warning; \
         got: {xref_warnings:?}"
    );
    assert!(
        matches!(
            xref_warnings[0],
            LayoutWarning::XrefTargetNotFound { target, source_slide_index: 0 }
            if target.as_ref() == "missing-slide"
        ),
        "warning must carry target='missing-slide' and source_slide_index=0; \
         got: {:?}",
        xref_warnings[0]
    );
}

/// AC-INT-1 / AC-003 — `layout::run` returns `Err(InlineDepthExceeded)` for a
/// bullet item with a 65-deep nested `Bold` tree (BC-3.05.001 invariant 4).
///
/// Canonical test vector from BC-3.05.001:
///   65-deep `Bold(Bold(...))` → `LayoutError::InlineDepthExceeded { depth: 65 }`.
///
/// At Red Gate: bullet not validated → returns Ok → assertion fails.
#[test]
fn test_bc_3_05_001_story073_ac_int1_depth_65_bullet_is_hard_error() {
    // Build the canonical 65-deep Bold tree.
    let mut node = InlineNode::Plain(Arc::from("leaf"));
    for _ in 0..=MAX_INLINE_DEPTH {
        node = InlineNode::Bold(vec![node]);
    }
    // F-094-P1-002: use "content" (has Body slot; "title" returns
    // Err(LayoutError::BulletsOnContentlessSlideType) / E-LAY-008
    // before inline validation runs, masking the InlineDepthExceeded error).
    let slide = bullets_slide("content", vec![flat_bullet(vec![node])]);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand);

    assert!(
        result.is_err(),
        "depth-65 bullet inline tree must return Err(InlineDepthExceeded); got Ok"
    );
    match result.unwrap_err() {
        LayoutError::InlineDepthExceeded {
            source_slide_index,
            depth,
            max,
        } => {
            assert_eq!(source_slide_index, 0, "source_slide_index must be 0");
            assert_eq!(
                depth, 65,
                "depth must be 65 (BC literal — first rejected level)"
            );
            assert_eq!(
                max, MAX_INLINE_DEPTH,
                "max must equal MAX_INLINE_DEPTH (64)"
            );
        },
        other => panic!("expected LayoutError::InlineDepthExceeded, got: {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-001 — Empty bullet list
// ─────────────────────────────────────────────────────────────────────────────

/// EC-001 — `layout::run` with `ContentBlock::Bullets(vec![])` produces zero
/// `TextRun` frames from bullets, no error, and no warning.
///
/// At Red Gate: trivially passes (bullets are skipped entirely). This becomes a
/// regression guard after implementation — confirming the implementation doesn't
/// crash or emit spurious warnings on an empty list.
#[test]
fn test_bc_3_05_001_story073_ec001_empty_bullets_no_frames_no_error_no_warning() {
    let slide = bullets_slide("title", vec![]);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand).expect("ContentBlock::Bullets(vec![]) must not error");

    assert_eq!(
        text_run_count(&result, 0),
        0,
        "empty bullet list must produce 0 TextRun frames"
    );
    assert!(
        result.warnings.is_empty(),
        "empty bullet list must produce no warnings; got: {:?}",
        result.warnings
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-002 — Bullet item with empty inline sequence
// ─────────────────────────────────────────────────────────────────────────────

/// EC-002 — A `BulletItem` with `inlines: vec![]` produces one `TextRun` frame
/// with an empty inline sequence. No depth error; no warning.
///
/// At Red Gate: 0 `TextRun` frames produced → assertion fails.
#[test]
fn test_bc_3_05_001_story073_ec002_empty_bullet_item_inlines_one_frame_no_error() {
    let empty_item = flat_bullet(vec![]);
    // F-094-P1-002: use "content" (has Body slot; "title" returns
    // Err(LayoutError::BulletsOnContentlessSlideType) / E-LAY-008
    // even for empty-inlines items because the contentless check fires before inlines are read).
    let slide = bullets_slide("content", vec![empty_item]);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand).expect("BulletItem with empty inlines must not error");

    assert_eq!(
        text_run_count(&result, 0),
        1,
        "one BulletItem with empty inlines must produce 1 TextRun frame; got {}",
        text_run_count(&result, 0)
    );
    assert!(
        result.warnings.is_empty(),
        "empty-inlines bullet must produce no warnings; got: {:?}",
        result.warnings
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-003 — Nested bullet items (children)
// ─────────────────────────────────────────────────────────────────────────────

/// EC-003 — Nested bullet item: `BulletItem.children` are NOT flattened.
/// Each item (parent and child) produces its own `FrameContent::TextRun` frame.
///
/// At Red Gate: 0 `TextRun` frames → assertion fails.
#[test]
fn test_bc_3_05_001_story073_ec003_nested_bullet_produces_frame_per_item() {
    let child = flat_bullet(vec![InlineNode::Plain(Arc::from("child bullet"))]);
    let parent = BulletItem {
        inlines: vec![InlineNode::Plain(Arc::from("parent bullet"))],
        children: vec![child],
        span: SourceSpan::default(),
    };
    // F-094-P1-002: use "content" (has Body slot; "title" returns
    // Err(LayoutError::BulletsOnContentlessSlideType) / E-LAY-008).
    let slide = bullets_slide("content", vec![parent]);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand).expect("nested bullet items must not error");

    // Parent item + 1 child item = 2 TextRun frames.
    assert_eq!(
        text_run_count(&result, 0),
        2,
        "nested bullet (1 parent + 1 child) must produce 2 TextRun frames; got {}",
        text_run_count(&result, 0)
    );

    let frame_inlines: Vec<Vec<InlineNode>> = result.slides[0]
        .frames
        .iter()
        .filter_map(|f| match &f.content {
            FrameContent::TextRun(nodes) => Some(nodes.clone()),
            _ => None,
        })
        .collect();

    let parent_inlines = vec![InlineNode::Plain(Arc::from("parent bullet"))];
    let child_inlines = vec![InlineNode::Plain(Arc::from("child bullet"))];

    assert!(
        frame_inlines.iter().any(|nodes| nodes == &parent_inlines),
        "parent bullet inlines must appear in a TextRun frame; frames: {frame_inlines:?}",
    );
    assert!(
        frame_inlines.iter().any(|nodes| nodes == &child_inlines),
        "child bullet inlines must appear in a TextRun frame (not dropped/flattened); \
         frames: {frame_inlines:?}",
    );
}

/// OBS-3 load-bearing — EC-003 positional frame order pinned at integration level.
///
/// The existing `test_..._ec003_nested_bullet_produces_frame_per_item` verifies
/// membership via `.any()` and a count check, but does NOT assert that the parent
/// frame precedes the child frame by absolute position.  This companion test
/// closes that gap at the integration level (the unit-level counterpart is
/// `test_..._exact_frame_order_three_levels` in lib.rs).
///
/// Deck: single slide, one parent bullet with one child bullet and one grandchild
/// bullet.  After `layout::run`, the three `TextRun` frames in `LaidOutSlide.frames`
/// must appear in source order:
///
///   `parent_frame_idx` < `child_frame_idx` < `grandchild_frame_idx`
///
/// Regression property: a bug that emits children-before-parents, or flattens/
/// reorders the tree, will produce frames in the wrong relative position and fail
/// at least one of the two `<` assertions below.
///
/// Uses `positional indexing` — no `.any()`.
#[test]
fn test_bc_3_05_001_story073_ec003_integration_frame_order_parent_before_child() {
    let grandchild = BulletItem {
        inlines: vec![InlineNode::Plain(Arc::from("INT_EC003_GRANDCHILD"))],
        children: vec![],
        span: SourceSpan::default(),
    };
    let child = BulletItem {
        inlines: vec![InlineNode::Plain(Arc::from("INT_EC003_CHILD"))],
        children: vec![grandchild],
        span: SourceSpan::default(),
    };
    let parent = BulletItem {
        inlines: vec![InlineNode::Plain(Arc::from("INT_EC003_PARENT"))],
        children: vec![child],
        span: SourceSpan::default(),
    };
    // F-094-P1-002: use "content" (has Body slot; "title" returns
    // Err(LayoutError::BulletsOnContentlessSlideType) / E-LAY-008).
    let slide = bullets_slide("content", vec![parent]);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand)
        .expect("three-level nested bullet must not error (all depths ≤ MAX_BULLET_DEPTH)");

    let slide_out = &result.slides[0];

    // Collect (frame_vector_index, inlines) for every TextRun frame, preserving order.
    let text_run_positions: Vec<(usize, &Vec<InlineNode>)> = slide_out
        .frames
        .iter()
        .enumerate()
        .filter_map(|(idx, f)| match &f.content {
            FrameContent::TextRun(nodes) => Some((idx, nodes)),
            _ => None,
        })
        .collect();

    // Exactly 3 TextRun frames: parent, child, grandchild.
    assert_eq!(
        text_run_positions.len(),
        3,
        "EC-003 integration: three-level nested bullet must produce exactly 3 TextRun \
         frames; got {} (total frames: {})",
        text_run_positions.len(),
        slide_out.frames.len()
    );

    let parent_frame_idx = text_run_positions[0].0;
    let child_frame_idx = text_run_positions[1].0;
    let grandchild_frame_idx = text_run_positions[2].0;

    let parent_nodes = text_run_positions[0].1;
    let child_nodes = text_run_positions[1].1;
    let grandchild_nodes = text_run_positions[2].1;

    // OBS-3 load-bearing ORDER assertions — cannot be satisfied by `.any()`.
    assert!(
        parent_frame_idx < child_frame_idx,
        "EC-003 integration: parent frame (index {parent_frame_idx}) must appear BEFORE \
         child frame (index {child_frame_idx}) in LaidOutSlide.frames — \
         a regression that emits children-before-parents would fail here"
    );
    assert!(
        child_frame_idx < grandchild_frame_idx,
        "EC-003 integration: child frame (index {child_frame_idx}) must appear BEFORE \
         grandchild frame (index {grandchild_frame_idx}) in LaidOutSlide.frames"
    );

    // Content correctness: each positional slot carries its item's inlines verbatim.
    let expected_parent = vec![InlineNode::Plain(Arc::from("INT_EC003_PARENT"))];
    let expected_child = vec![InlineNode::Plain(Arc::from("INT_EC003_CHILD"))];
    let expected_grandchild = vec![InlineNode::Plain(Arc::from("INT_EC003_GRANDCHILD"))];

    assert_eq!(
        parent_nodes, &expected_parent,
        "EC-003 integration: frame at index {parent_frame_idx} must carry parent inlines; \
         got: {parent_nodes:?}"
    );
    assert_eq!(
        child_nodes, &expected_child,
        "EC-003 integration: frame at index {child_frame_idx} must carry child inlines; \
         got: {child_nodes:?}"
    );
    assert_eq!(
        grandchild_nodes, &expected_grandchild,
        "EC-003 integration: frame at index {grandchild_frame_idx} must carry grandchild \
         inlines; got: {grandchild_nodes:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-004 — Xref inside nested container inside bullet
// ─────────────────────────────────────────────────────────────────────────────

/// EC-004 — `run_inline_validation` traverses into container nodes inside bullet
/// inline content. An `Xref` nested inside `Bold(Italic(...))` inside a bullet must
/// still trigger `XrefTargetNotFound`.
///
/// At Red Gate: bullet not processed → no frame → xref not validated → no warning →
/// assertion fails.
#[test]
fn test_bc_3_05_001_story073_ec004_deep_nested_xref_in_bullet_layout_run() {
    let unknown = Arc::from("__ec004_deep_xref__");
    // F-094-P1-002: use "content" (has Body slot; "title" returns
    // Err(LayoutError::BulletsOnContentlessSlideType) / E-LAY-008
    // before inline validation runs, masking the XrefTargetNotFound warning).
    let slide = bullets_slide(
        "content",
        vec![flat_bullet(vec![InlineNode::Bold(vec![
            InlineNode::Italic(vec![InlineNode::Xref(Arc::clone(&unknown))]),
        ])])],
    );
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result =
        run(&deck, &brand).expect("deep nested xref in bullet must produce warning, not error");

    assert!(
        result.warnings.iter().any(|w| matches!(
            w,
            LayoutWarning::XrefTargetNotFound { target, .. }
            if target.as_ref() == "__ec004_deep_xref__"
        )),
        "XrefTargetNotFound must be produced for xref inside Bold(Italic(Xref)) in bullet; \
         got: {:?}",
        result.warnings
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-005 — Multiple bullets on same slide with depth violations
// ─────────────────────────────────────────────────────────────────────────────

/// EC-005 — Multiple bullets on the same slide, both at depth 65 (hard error).
///
/// `layout::run` must return `Err(InlineDepthExceeded)` when ANY bullet item has
/// a depth-65 inline tree. The "accumulate all errors" (DI-018) property applies
/// to warnings (soft) not to depth errors (hard). The first hard error terminates
/// processing and returns Err.
///
/// At Red Gate: bullets not validated → returns Ok → assertion fails.
#[test]
fn test_bc_3_05_001_story073_ec005_multiple_depth_exceeded_bullets_returns_error() {
    // Build two 65-deep Bold trees.
    let make_deep_node = || {
        let mut node = InlineNode::Plain(Arc::from("leaf"));
        for _ in 0..=MAX_INLINE_DEPTH {
            node = InlineNode::Bold(vec![node]);
        }
        node
    };

    let items = vec![
        flat_bullet(vec![make_deep_node()]),
        flat_bullet(vec![make_deep_node()]),
    ];
    // F-094-P1-002: use "content" (has Body slot; "title" returns
    // Err(LayoutError::BulletsOnContentlessSlideType) / E-LAY-008
    // before inline validation runs, masking the InlineDepthExceeded error).
    let slide = bullets_slide("content", items);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand);

    assert!(
        result.is_err(),
        "two depth-65 bullet items must cause Err(InlineDepthExceeded); got Ok"
    );
    assert!(
        matches!(result.unwrap_err(), LayoutError::InlineDepthExceeded { .. }),
        "error must be InlineDepthExceeded"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// VP-047 — All 12 inline variants in bullet content survive layout unchanged
// ─────────────────────────────────────────────────────────────────────────────

/// VP-047 (bullets extension) — All 12 `InlineNode` variants in a bullet item's
/// `inlines` sequence survive the layout pass verbatim in the `FrameContent::TextRun`
/// frame (BC-3.05.001 invariant 6 / VP-047).
///
/// This is the bullet-specific extension of the VP-047 test (which covers `TextBlock`
/// content). The layout stage must NOT process, transform, or drop any variant.
///
/// At Red Gate: no `TextRun` frame is produced for bullet items → the inline content
/// cannot be inspected → assertion fails because the frame doesn't exist.
#[test]
fn test_bc_3_05_001_story073_vp047_all_12_inline_variants_in_bullet_survive_layout() {
    use slideforge_types::MathNode;

    // Construct one BulletItem whose inlines contain all 12 InlineNode variants.
    // The Xref target "introduction" is provided as a known slide title so no
    // XrefTargetNotFound warning is emitted.
    let all_12_inlines = vec![
        InlineNode::Plain(Arc::from("plain")),
        InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold"))]),
        InlineNode::Italic(vec![InlineNode::Plain(Arc::from("italic"))]),
        InlineNode::Code(Arc::from("code()")),
        InlineNode::Link {
            text: vec![InlineNode::Plain(Arc::from("link text"))],
            url: Arc::from("https://example.com"),
        },
        InlineNode::Math(MathNode {
            latex: Arc::from("x^2"),
            display: false,
            span: SourceSpan::default(),
        }),
        InlineNode::Footnote(vec![InlineNode::Plain(Arc::from("footnote"))]),
        InlineNode::Xref(Arc::from("introduction")), // known title
        InlineNode::Superscript(vec![InlineNode::Plain(Arc::from("sup"))]),
        InlineNode::Subscript(vec![InlineNode::Plain(Arc::from("sub"))]),
        InlineNode::Strikethrough(vec![InlineNode::Plain(Arc::from("strike"))]),
        InlineNode::Highlight(vec![InlineNode::Plain(Arc::from("highlight"))]),
    ];
    assert_eq!(
        all_12_inlines.len(),
        12,
        "test setup: must have exactly 12 inline variants"
    );

    let title_slide = make_slide_with_title("title", "introduction");
    let bullet_item = flat_bullet(all_12_inlines.clone());
    let content_slide = bullets_slide("content", vec![bullet_item]);
    let deck = make_deck(vec![title_slide, content_slide]);
    let brand = make_brand();

    let result = run(&deck, &brand)
        .expect("layout::run must succeed for bullet with all 12 InlineNode variants");

    // Zero warnings (known xref, depth ≤ 64).
    assert!(
        result.warnings.is_empty(),
        "all-12-variants bullet must produce zero warnings; got: {:?}",
        result.warnings
    );

    // Find the TextRun frame carrying all 12 variants.
    let bullet_frame = result.slides[1]
        .frames
        .iter()
        .find_map(|f| match &f.content {
            FrameContent::TextRun(nodes) if nodes.len() == 12 => Some(nodes.clone()),
            _ => None,
        });

    let bullet_frame = bullet_frame.expect(
        "must find a FrameContent::TextRun frame carrying exactly 12 InlineNode variants \
         (one per BulletItem.inlines for the all-12-variants bullet)",
    );

    assert_eq!(
        bullet_frame, all_12_inlines,
        "VP-047: all 12 InlineNode variants must survive the layout pass unchanged in the \
         FrameContent::TextRun frame; got: {bullet_frame:?}",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-INT-1 — Mixed slide deck (Bullets + other ContentBlocks)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-INT-1 step 5 — `layout::run` on a deck that mixes `ContentBlock::Bullets`
/// with other content blocks produces correct `LaidOutDeck` output.
///
/// A slide with both `ContentBlock::Text` and `ContentBlock::Bullets` blocks must:
/// - Produce a `TextRun` frame for the Text block.
/// - Produce one `TextRun` frame per `BulletItem`.
/// - Not produce errors for well-formed input.
///
/// At Red Gate: Bullets block produces no frames → `TextRun` count is 1 (from Text only),
/// not 4 (1 from Text + 3 from Bullets) → assertion fails.
#[test]
fn test_bc_3_05_001_story073_ac_int1_mixed_text_and_bullets_blocks() {
    use slideforge_types::TextBlock;

    let text_block = Block {
        content: ContentBlock::Text(TextBlock {
            inlines: vec![InlineNode::Plain(Arc::from("intro paragraph"))],
            tag: slideforge_types::TextTag::Untagged,
            span: SourceSpan::default(),
        }),
        label: None,
        span: SourceSpan::default(),
    };
    let bullet_block = Block {
        content: ContentBlock::Bullets(vec![
            flat_bullet(vec![InlineNode::Plain(Arc::from("bullet 1"))]),
            flat_bullet(vec![InlineNode::Plain(Arc::from("bullet 2"))]),
            flat_bullet(vec![InlineNode::Plain(Arc::from("bullet 3"))]),
        ]),
        label: None,
        span: SourceSpan::default(),
    };
    let slide = Slide {
        slide_type: Arc::from("content"),
        fields: OrderedMap::new(),
        blocks: vec![text_block, bullet_block],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
        field_spans: OrderedMap::new(),
    };
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result =
        run(&deck, &brand).expect("layout::run must succeed for mixed Text + Bullets blocks");

    // Expect 4 TextRun frames: 1 from Text block + 3 from Bullets block.
    assert_eq!(
        text_run_count(&result, 0),
        4,
        "mixed Text + 3-item Bullets must produce 4 TextRun frames (1 + 3); \
         got {} TextRun frames (total frames: {})",
        text_run_count(&result, 0),
        result.slides[0].frames.len()
    );

    // No errors, no warnings (all content is well-formed).
    assert!(
        result.warnings.is_empty(),
        "well-formed mixed deck must produce zero warnings; got: {:?}",
        result.warnings
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// EMU correctness — bullet TextRun bbox satisfies is_valid
// ─────────────────────────────────────────────────────────────────────────────

/// AC-INT-1 / BC-3.06.003 — All bounding boxes for bullet `TextRun` frames must
/// satisfy `BoundingBox::is_valid` (non-negative, non-zero, within page bounds).
///
/// Uses integer EMUs (`914_400` per inch) — no f64 (project convention DI-010 / ADR-013).
///
/// At Red Gate: no `TextRun` frames for bullets → loop iterates nothing → trivially
/// passes. Becomes load-bearing after implementation.
#[test]
fn test_bc_3_05_001_story073_bullet_text_run_bboxes_are_valid() {
    // This test uses a "content" slide (has Body slot) so bullets get correct bbox.
    let items = vec![
        flat_bullet(vec![InlineNode::Plain(Arc::from("first"))]),
        flat_bullet(vec![InlineNode::Plain(Arc::from("second"))]),
    ];
    let slide = bullets_slide("content", items);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand).expect("layout::run must succeed for valid bullet items");

    let page_w = result.page_size.width;
    let page_h = result.page_size.height;

    for (slide_idx, laid_out_slide) in result.slides.iter().enumerate() {
        for (frame_idx, frame) in laid_out_slide.frames.iter().enumerate() {
            if matches!(frame.content, FrameContent::TextRun(_)) {
                assert!(
                    frame.bbox.is_valid(page_w, page_h),
                    "BC-3.06.003: bullet TextRun frame bbox must be valid; \
                     slide {slide_idx} frame {frame_idx}: {:?}",
                    frame.bbox
                );
                // Additional EMU precision checks — no f64, use integer EMU arithmetic.
                // width and height must be positive (non-zero Emu values).
                assert!(
                    frame.bbox.width.0 > 0,
                    "bullet TextRun frame width must be > 0 EMU; got {:?}",
                    frame.bbox.width
                );
                assert!(
                    frame.bbox.height.0 > 0,
                    "bullet TextRun frame height must be > 0 EMU; got {:?}",
                    frame.bbox.height
                );
                // BC-3.06.003: position must be non-zero (from region map, not fallback).
                // Bullets on a content slide must be at the body region position, not (0,0).
                assert!(
                    frame.bbox.x.0 > 0,
                    "BC-3.06.003 F-094-P1-002: bullet bbox.x must be > 0 EMU (region-map \
                     position expected, not (0,0) fallback); got x={:?}",
                    frame.bbox.x
                );
                assert!(
                    frame.bbox.y.0 > 0,
                    "BC-3.06.003 F-094-P1-002: bullet bbox.y must be > 0 EMU (region-map \
                     position expected, not (0,0) fallback); got y={:?}",
                    frame.bbox.y
                );
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// F-094-P1-002: bullets on slide types with no Body slot must error
// ─────────────────────────────────────────────────────────────────────────────

/// F-094-P1-002 (MED) / BC-3.06.003 postcondition 2:
/// When bullets appear on a slide type that has no Body or Generic Empty slot
/// (`title`, `closing`, `section_break`, `blank`), `layout::run()` must return
/// `Err(LayoutError::BulletsOnContentlessSlideType)` (E-LAY-008) rather than
/// silently placing frames at (0,0).
///
/// Current behavior (post-fix): the layout engine detects that the slide type
/// has no Body region and returns `Err(LayoutError::BulletsOnContentlessSlideType)`
/// carrying the slide type name, source slide index, and correction hint
/// (use `bullets_only` slide type instead).
///
/// Load-bearing: `.is_err()` fails when `run` returns `Ok(...)` with (0,0) frames;
/// the variant match pins the specific E-LAY-008 error kind.
#[test]
fn test_f094_p1_002_bullets_on_title_slide_returns_contentless_slide_error() {
    let items = vec![flat_bullet(vec![InlineNode::Plain(Arc::from("item"))])];
    let slide = bullets_slide("title", items);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand);

    assert!(
        result.is_err(),
        "F-094-P1-002 / F-094-P2-003: bullets on a 'title' slide (no Body region) must return \
         Err(LayoutError::BulletsOnContentlessSlideType); layout::run returned Ok(..)"
    );
    let err = result.unwrap_err();
    // E-LAY-008 accumulation (error-taxonomy v2.30 §234): a single-slide deck returns
    // Multiple { inner: [BulletsOnContentlessSlideType { .. }] } — the Multiple wrapper
    // is the uniform accumulation channel per the DI-018 / LayoutError::multiple() contract.
    assert!(
        is_or_contains_contentless(&err),
        "F-094-P2-003 Red Gate: error must be or contain LayoutError::BulletsOnContentlessSlideType \
         (E-LAY-008); got: {err:?}"
    );
}

/// F-094-P1-002 / BC-3.06.003 — closing slide type (no Body slot).
#[test]
fn test_f094_p1_002_bullets_on_closing_slide_returns_contentless_slide_error() {
    let items = vec![flat_bullet(vec![InlineNode::Plain(Arc::from("item"))])];
    let slide = bullets_slide("closing", items);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand);

    assert!(
        result.is_err(),
        "F-094-P1-002 / F-094-P2-003: bullets on a 'closing' slide (no Body region) must return \
         Err; layout::run returned Ok(..)"
    );
    assert!(
        is_or_contains_contentless(&result.unwrap_err()),
        "F-094-P2-003: error must be or contain LayoutError::BulletsOnContentlessSlideType (E-LAY-008)"
    );
}

/// F-094-P1-002 / BC-3.06.003 — `section_break` slide type (no Body slot).
#[test]
fn test_f094_p1_002_bullets_on_section_break_returns_contentless_slide_error() {
    let items = vec![flat_bullet(vec![InlineNode::Plain(Arc::from("item"))])];
    let slide = bullets_slide("section_break", items);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand);

    assert!(
        result.is_err(),
        "F-094-P1-002 / F-094-P2-003: bullets on 'section_break' (no Body region) must error"
    );
    assert!(
        is_or_contains_contentless(&result.unwrap_err()),
        "F-094-P2-003: error must be or contain LayoutError::BulletsOnContentlessSlideType (E-LAY-008)"
    );
}

/// F-094-P1-002 / BC-3.06.003 — blank slide type (zero regions, no Body slot).
#[test]
fn test_f094_p1_002_bullets_on_blank_slide_returns_contentless_slide_error() {
    let items = vec![flat_bullet(vec![InlineNode::Plain(Arc::from("item"))])];
    let slide = bullets_slide("blank", items);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand);

    assert!(
        result.is_err(),
        "F-094-P1-002 / F-094-P2-003: bullets on 'blank' (no slots at all) must error"
    );
    assert!(
        is_or_contains_contentless(&result.unwrap_err()),
        "F-094-P2-003: error must be or contain LayoutError::BulletsOnContentlessSlideType (E-LAY-008)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-094-P1-003: body text + bullets coexistence — bullets must get body region bbox
// ─────────────────────────────────────────────────────────────────────────────

/// F-094-P1-003 (MED) / BC-3.06.003 postcondition 1:
/// When a `content` slide has BOTH a Body text block AND a Bullets block, bullets
/// must get the correct (non-zero, region-map) bbox — NOT the (0,0) fallback.
///
/// DEFECT: `ContentBlock::Body` fills the Body-role Empty slot via
/// `fill_region_slot_or_append`, replacing the Empty frame with `FrameContent::Body`.
/// When `ContentBlock::Bullets` then searches for an Empty Body-role slot, it finds
/// none (already replaced), falls through to Generic/None search (also none on
/// `content` slides), and hits Phase-3 → (0,0) bbox.
///
/// This test MUST FAIL against the unfixed code (bullets get x=0, y=0).
/// It will pass only after the Bullets lookup also finds already-filled Body slots.
///
/// Load-bearing: asserts `bbox.x.0 > 0` on all `TextRun` frames (from bullets),
/// which fails when x=0.
#[test]
fn test_f094_p1_003_body_text_plus_bullets_bullets_get_body_region_bbox() {
    use slideforge_types::{Block, TextBlock, TextTag};

    // Build a content slide with:
    //   1. Body text block (fills the Body-role Empty slot)
    //   2. Bullets block (must inherit the same body region bbox)
    let body_block = Block {
        content: ContentBlock::Text(TextBlock {
            tag: TextTag::Body,
            inlines: vec![InlineNode::Plain(Arc::from("body text"))],
            span: SourceSpan::default(),
        }),
        label: None,
        span: SourceSpan::default(),
    };
    let bullet_items = vec![flat_bullet(vec![InlineNode::Plain(Arc::from(
        "bullet item",
    ))])];
    let bullet_block = Block {
        content: ContentBlock::Bullets(bullet_items),
        label: None,
        span: SourceSpan::default(),
    };

    let slide = Slide {
        slide_type: Arc::from("content"),
        fields: OrderedMap::new(),
        blocks: vec![body_block, bullet_block],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
        field_spans: OrderedMap::new(),
    };

    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand).expect(
        "F-094-P1-003 Red Gate: layout::run must succeed for content slide with body+bullets",
    );

    // Find TextRun frames (from bullets).
    let text_run_frames: Vec<_> = result.slides[0]
        .frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::TextRun(_)))
        .collect();

    assert!(
        !text_run_frames.is_empty(),
        "F-094-P1-003: content slide with bullets must produce at least one TextRun frame"
    );

    for frame in text_run_frames {
        assert!(
            frame.bbox.x.0 > 0,
            "F-094-P1-003 Red Gate: bullet TextRun bbox.x must be > 0 (body region position); \
             got x={} — indicates (0,0) fallback still in effect when Body+Bullets coexist",
            frame.bbox.x.0
        );
        assert!(
            frame.bbox.y.0 > 0,
            "F-094-P1-003 Red Gate: bullet TextRun bbox.y must be > 0 (body region position); \
             got y={}",
            frame.bbox.y.0
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// F-094-P2-001 (CRITICAL): N bullets must yield N distinct, strictly-increasing
// bbox.y values — not N overlapping frames at the same y.
// ─────────────────────────────────────────────────────────────────────────────

/// F-094-P2-001 (CRITICAL) / BC-3.06.003 postcondition 1:
/// Three bullets on a `content` slide must produce three `TextRun` frames whose
/// `bbox.y` values are STRICTLY INCREASING and NON-OVERLAPPING.
///
/// DEFECT: `push_bullet_frames_inner` currently assigns the same `body_bbox`
/// to every bullet item — N bullets produce N frames all at identical y.
/// REND-001 intent is un-stacked, legible bullets (not N identical positions).
///
/// RED GATE: this test must fail until per-bullet vertical flow is implemented.
/// Each bullet must advance a cursor: y[i+1] >= y[i] + height[i].
///
/// FU-DIAGNOSTIC-FIELD-PINNING: all y values are pinned (not just `y[0] != y[1]`).
#[test]
fn test_f094_p2_001_bullets_have_distinct_increasing_y_values() {
    // Three-item bullet list on a `content` slide.
    let items = vec![
        flat_bullet(vec![InlineNode::Plain(Arc::from("Bullet One"))]),
        flat_bullet(vec![InlineNode::Plain(Arc::from("Bullet Two"))]),
        flat_bullet(vec![InlineNode::Plain(Arc::from("Bullet Three"))]),
    ];
    let slide = bullets_slide("content", items);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand)
        .expect("F-094-P2-001: layout::run must succeed for content slide with 3 bullets");

    let text_run_frames: Vec<_> = result.slides[0]
        .frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::TextRun(_)))
        .collect();

    assert_eq!(
        text_run_frames.len(),
        3,
        "F-094-P2-001: 3-item bullet list must produce exactly 3 TextRun frames; \
         got {} frames (total slide frames: {})",
        text_run_frames.len(),
        result.slides[0].frames.len()
    );

    // F-094-P2-001 load-bearing: y values must be strictly increasing and non-overlapping.
    // Defective code: all bullets get the SAME body_bbox → all have identical y.
    for i in 0..text_run_frames.len() - 1 {
        let curr = &text_run_frames[i];
        let next = &text_run_frames[i + 1];
        // Non-overlapping: next frame must start at or after current frame's bottom edge.
        let curr_bottom = curr.bbox.y.0.saturating_add(curr.bbox.height.0);
        assert!(
            next.bbox.y.0 >= curr_bottom,
            "F-094-P2-001 Red Gate: TextRun frame {i} and frame {} overlap — \
             frame {i} y={} height={} (bottom={curr_bottom}), frame {} y={} \
             (expected y >= {curr_bottom}). Defect: all bullets share the same bbox.",
            i + 1,
            curr.bbox.y.0,
            curr.bbox.height.0,
            i + 1,
            next.bbox.y.0
        );
        // Strictly increasing: consecutive y values must differ (no zero-height stacking).
        assert!(
            next.bbox.y.0 > curr.bbox.y.0,
            "F-094-P2-001 Red Gate: TextRun frame {} y ({}) must be strictly greater than \
             frame {i} y ({}) — all bullets must have distinct y positions",
            i + 1,
            next.bbox.y.0,
            curr.bbox.y.0
        );
    }

    // All frames must remain within page bounds.
    let page_h = result.page_size.height;
    for (idx, frame) in text_run_frames.iter().enumerate() {
        let bottom = frame.bbox.y.0.saturating_add(frame.bbox.height.0);
        assert!(
            bottom <= page_h.0,
            "F-094-P2-001: TextRun frame {idx} extends below page_height \
             (y={} + height={} = {bottom} > page_h={})",
            frame.bbox.y.0,
            frame.bbox.height.0,
            page_h.0
        );
    }
}

/// F-094-P2-001 — Depth-indented child bullet must have x > parent bullet x.
///
/// A bullet with a child (one nesting level) must produce:
///   - Parent frame at some `x_parent`
///   - Child frame at `x_child` > `x_parent` (horizontal indentation)
///
/// DEFECT: all frames currently get the same `body_bbox.x` — no indentation.
/// RED GATE: this test must fail until depth-x-offset is implemented.
#[test]
fn test_f094_p2_001_child_bullet_x_greater_than_parent_x() {
    use slideforge_types::BulletItem;

    let child = BulletItem {
        inlines: vec![InlineNode::Plain(Arc::from("child item"))],
        children: vec![],
        span: SourceSpan::default(),
    };
    let parent = BulletItem {
        inlines: vec![InlineNode::Plain(Arc::from("parent item"))],
        children: vec![child],
        span: SourceSpan::default(),
    };
    let block = slideforge_types::Block {
        content: ContentBlock::Bullets(vec![parent]),
        label: None,
        span: SourceSpan::default(),
    };
    let slide = Slide {
        slide_type: Arc::from("content"),
        fields: OrderedMap::new(),
        blocks: vec![block],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
        field_spans: OrderedMap::new(),
    };
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand)
        .expect("F-094-P2-001: layout::run must succeed for content slide with nested bullets");

    let text_run_frames: Vec<_> = result.slides[0]
        .frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::TextRun(_)))
        .collect();

    assert_eq!(
        text_run_frames.len(),
        2,
        "F-094-P2-001: parent+child bullet must produce exactly 2 TextRun frames; \
         got {} (total frames: {})",
        text_run_frames.len(),
        result.slides[0].frames.len()
    );

    let parent_x = text_run_frames[0].bbox.x;
    let child_x = text_run_frames[1].bbox.x;
    assert!(
        child_x > parent_x,
        "F-094-P2-001 Red Gate: child bullet x ({child_x:?}) must be strictly greater than \
         parent bullet x ({parent_x:?}) — depth indentation not applied. \
         Defect: all bullets share the same body_bbox.x."
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-094-P2-002 (HIGH): body text + bullets coexistence — no overlapping frames.
// Body frame bbox must be shrunk to its content extent; bullets flow below.
// ─────────────────────────────────────────────────────────────────────────────

/// F-094-P2-002 (HIGH) / BC-3.06.003 postcondition 1:
/// A `content` slide with BOTH a Body text block AND a Bullets block must produce
/// non-overlapping frames: the Body frame and every bullet `TextRun` frame must have
/// pairwise non-overlapping, orderly-stacked bboxes.
///
/// DEFECT: Phase 1b borrows the filled Body frame's `bbox` wholesale → Body and
/// every bullet frame have the SAME identical `bbox` (same `y`, same `height`).
///
/// RED GATE: this test must fail until the flow-cursor mechanism is applied.
/// Body frame covers `[y_body, y_body+h_body)`; bullets start at `y_body+h_body` or later.
///
/// FU-DIAGNOSTIC-FIELD-PINNING: all frame `bbox` values pinned.
#[test]
fn test_f094_p2_002_body_and_bullets_frames_non_overlapping() {
    use slideforge_types::{Block, TextBlock, TextTag};

    let body_block = Block {
        content: ContentBlock::Text(TextBlock {
            tag: TextTag::Body,
            inlines: vec![InlineNode::Plain(Arc::from("Body paragraph content"))],
            span: SourceSpan::default(),
        }),
        label: None,
        span: SourceSpan::default(),
    };
    let bullet_items = vec![
        flat_bullet(vec![InlineNode::Plain(Arc::from("Bullet A"))]),
        flat_bullet(vec![InlineNode::Plain(Arc::from("Bullet B"))]),
    ];
    let bullet_block = slideforge_types::Block {
        content: ContentBlock::Bullets(bullet_items),
        label: None,
        span: SourceSpan::default(),
    };
    let slide = Slide {
        slide_type: Arc::from("content"),
        fields: OrderedMap::new(),
        blocks: vec![body_block, bullet_block],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
        field_spans: OrderedMap::new(),
    };
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand)
        .expect("F-094-P2-002: layout::run must succeed for content slide with body+bullets");

    let slide_out = &result.slides[0];

    // Collect the Body frame and TextRun frames.
    let body_frames: Vec<_> = slide_out
        .frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::Body(_)))
        .collect();
    let text_run_frames: Vec<_> = slide_out
        .frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::TextRun(_)))
        .collect();

    assert!(
        !body_frames.is_empty(),
        "F-094-P2-002: content slide with Body block must produce a Body frame"
    );
    assert_eq!(
        text_run_frames.len(),
        2,
        "F-094-P2-002: 2-item bullet list must produce exactly 2 TextRun frames; \
         got {} (total frames: {})",
        text_run_frames.len(),
        slide_out.frames.len()
    );

    // All frames (Body + TextRuns) must be pairwise non-overlapping.
    // Overlap criterion: two frames [y1, y1+h1) and [y2, y2+h2) overlap iff
    //   y1 < y2+h2 AND y2 < y1+h1 (intervals on the y-axis, ignoring x for this check).
    let all_frames: Vec<_> = body_frames.iter().chain(text_run_frames.iter()).collect();

    for i in 0..all_frames.len() {
        for j in i + 1..all_frames.len() {
            let a = &all_frames[i];
            let b = &all_frames[j];
            let a_top = a.bbox.y.0;
            let a_bot = a.bbox.y.0.saturating_add(a.bbox.height.0);
            let b_top = b.bbox.y.0;
            let b_bot = b.bbox.y.0.saturating_add(b.bbox.height.0);
            let overlaps = a_top < b_bot && b_top < a_bot;
            assert!(
                !overlaps,
                "F-094-P2-002 Red Gate: frame {i} [y={a_top}, y+h={a_bot}) overlaps frame {j} \
                 [y={b_top}, y+h={b_bot}) — Body and bullet frames must be non-overlapping. \
                 Defect: Phase 1b borrows body_bbox unchanged for all bullet frames.",
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// F-094-P2-003 (MED): E-LAY-008 error variant + message fields
// ─────────────────────────────────────────────────────────────────────────────

/// F-094-P2-003 (MED) / E-LAY-008 / error taxonomy v2.30:
/// When bullets appear on a contentless slide type (no Body or Generic Empty region),
/// `layout::run` must return `Err(LayoutError::BulletsOnContentlessSlideType { .. })`
/// whose Display message contains:
///   - `"[E-LAY-008]"` code prefix
///   - the slide type name (e.g. `"title"`)
///   - a correction hint mentioning `"bullets_only"`
///
/// RED GATE: currently returns `Err(InvalidBoundingBox)` — must fail until the
/// Phase-3 fallback is replaced with `BulletsOnContentlessSlideType`.
///
/// FU-DIAGNOSTIC-FIELD-PINNING: pins the code prefix, slide type, and hint
/// distinguishing field VALUES (not just `is_err()`).
#[test]
fn test_f094_p2_003_bullets_on_contentless_returns_e_lay_008_variant_and_message() {
    let items = vec![flat_bullet(vec![InlineNode::Plain(Arc::from("item"))])];
    let slide = bullets_slide("title", items);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand);

    assert!(
        result.is_err(),
        "F-094-P2-003: bullets on 'title' (no body region) must return Err; got Ok"
    );
    let err = result.unwrap_err();

    // E-LAY-008 accumulation (error-taxonomy v2.30 §234): extract the first
    // BulletsOnContentlessSlideType from the Multiple wrapper.
    let contentless = extract_first_contentless(&err).unwrap_or_else(|| {
        panic!(
            "F-094-P2-003 Red Gate: expected error to be or contain \
             LayoutError::BulletsOnContentlessSlideType (E-LAY-008); got: {err:?}"
        )
    });

    // Must be the E-LAY-008 variant (not InvalidBoundingBox).
    let (variant_slide_type, variant_slide_index) = match contentless {
        LayoutError::BulletsOnContentlessSlideType {
            slide_type,
            source_slide_index,
            ..
        } => (slide_type.clone(), *source_slide_index),
        other => panic!(
            "F-094-P2-003 Red Gate: expected LayoutError::BulletsOnContentlessSlideType, \
             got: {other:?}"
        ),
    };
    assert_eq!(
        variant_slide_type.as_ref(),
        "title",
        "F-094-P2-003: BulletsOnContentlessSlideType.slide_type must be 'title'; got '{variant_slide_type}'"
    );
    assert_eq!(
        variant_slide_index, 0,
        "F-094-P2-003: source_slide_index must be 0; got {variant_slide_index}"
    );

    // E-LAY-008 message must contain code prefix, slide type, and hint.
    let msg = contentless.to_string();
    assert!(
        msg.contains("[E-LAY-008]"),
        "F-094-P2-003: error message must contain '[E-LAY-008]' code prefix; got: {msg}"
    );
    assert!(
        msg.contains("title"),
        "F-094-P2-003: error message must contain the slide type 'title'; got: {msg}"
    );
    assert!(
        msg.contains("bullets_only"),
        "F-094-P2-003: error message must contain 'bullets_only' in the hint; got: {msg}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-094-P3-001 — E-LAY-008 span is NOT fabricated (real field span)
// ─────────────────────────────────────────────────────────────────────────────

/// F-094-P3-001 (a): `BulletsOnContentlessSlideType` must carry a non-default
/// (non-unknown) `SourceSpan` when the slide was constructed with a real
/// `field_spans` entry for "bullets".
///
/// Strengthen of `test_f094_p2_003`: in addition to checking the variant and
/// message, assert `!span.is_unknown()` (FU-DIAGNOSTIC-FIELD-PINNING).
///
/// RED: currently the `Block` constructed by `field_to_block.rs` uses
/// `SourceSpan::default()` → `span.is_unknown() == true`. Fails until
/// `field_spans` is threaded from eval through to layout.
#[test]
fn test_f094_p3_001_bullets_on_contentless_span_is_non_default() {
    use slideforge_types::SourceSpan;

    let items = vec![flat_bullet(vec![InlineNode::Plain(Arc::from("item"))])];
    // Build a slide where the "bullets" field has a non-default span.
    // We do this by constructing a Slide with a field_spans entry for "bullets"
    // that has a real byte offset.
    let mut slide = bullets_slide("title", items);
    // Inject a non-default span for the "bullets" key so that thread_fields_to_blocks
    // (or layout) can surface it. After the fix, block.span should not be
    // is_unknown() when field_spans carries a real span.
    slide.field_spans.insert(
        Arc::from("bullets"),
        SourceSpan::new(Arc::from("test.sf"), 3, 1, 42),
    );
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand);
    assert!(
        result.is_err(),
        "F-094-P3-001: bullets on 'title' must return Err; got Ok"
    );
    let err = result.unwrap_err();

    // E-LAY-008 accumulation (error-taxonomy v2.30 §234): extract the first
    // BulletsOnContentlessSlideType from the Multiple wrapper, then check its span.
    let contentless = extract_first_contentless(&err).unwrap_or_else(|| {
        panic!("F-094-P3-001: expected error to be or contain BulletsOnContentlessSlideType, got: {err:?}")
    });
    match contentless {
        LayoutError::BulletsOnContentlessSlideType { span, .. } => {
            assert!(
                !span.is_unknown(),
                "F-094-P3-001: BulletsOnContentlessSlideType.span must not be unknown (is_unknown=true); \
                 expected a real field span after field_spans threading. Got span: {span:?}"
            );
        },
        other => panic!("F-094-P3-001: expected BulletsOnContentlessSlideType, got: {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// F-094-P3-002 — Geometric bullet-overflow detection (not count-based)
// ─────────────────────────────────────────────────────────────────────────────

// F-094-P3-002 test is in crates/slideforge-validate/src/canvas_overflow.rs
// (validate_post_layout test requires slideforge-layout dep — already in validate crate).

// ─────────────────────────────────────────────────────────────────────────────
// F-094-P6-001 — E-LAY-008 accumulates across slides (error-taxonomy v2.30 §234)
// ─────────────────────────────────────────────────────────────────────────────

/// F-094-P6-001 (MED) / error-taxonomy v2.30 §234:
/// When a deck contains TWO slides with `bullets:` on a contentless slide type,
/// `layout::run` must return `Err(LayoutError::Multiple { inner })` containing
/// BOTH `BulletsOnContentlessSlideType` instances — not just the first.
///
/// Deck shape:
///   - Slide 0: `title` + bullets (E-LAY-008, `source_slide_index=0`, `slide_type="title"`)
///   - Slide 1: `content` + bullets (valid — must NOT produce an error)
///   - Slide 2: `closing` + bullets (E-LAY-008, `source_slide_index=2`, `slide_type="closing"`)
///
/// Expected:
///   - `run` returns `Err(LayoutError::Multiple { inner })` where `inner` has exactly 2
///     `BulletsOnContentlessSlideType` entries.
///   - First entry: `slide_type="title"`, `source_slide_index=0`.
///   - Second entry: `slide_type="closing"`, `source_slide_index=2`.
///   - A single strict-mode build pass surfaces both (no bail-on-first).
///
/// RED: current code hard-`return Err(BulletsOnContentlessSlideType { .. })` at the
/// first contentless slide — the second instance is never reported.
#[test]
fn test_f094_p6_001_e_lay_008_accumulates_across_two_slides() {
    let items = || vec![flat_bullet(vec![InlineNode::Plain(Arc::from("item"))])];

    // Slide 0: title + bullets → E-LAY-008
    let slide0 = bullets_slide("title", items());
    // Slide 1: content + bullets → valid (must NOT be in the error set)
    let slide1 = bullets_slide("content", items());
    // Slide 2: closing + bullets → E-LAY-008
    let slide2 = bullets_slide("closing", items());

    let deck = make_deck(vec![slide0, slide1, slide2]);
    let brand = make_brand();

    let result = run(&deck, &brand);

    assert!(
        result.is_err(),
        "F-094-P6-001: deck with two contentless-bullets slides must return Err"
    );
    let err = result.unwrap_err();

    // Must be Multiple containing exactly 2 BulletsOnContentlessSlideType entries.
    let inner = match err {
        LayoutError::Multiple { ref inner } => inner.clone(),
        LayoutError::BulletsOnContentlessSlideType {
            source_slide_index,
            ref slide_type,
            ..
        } => {
            panic!(
                "F-094-P6-001 RED: got single BulletsOnContentlessSlideType (slide_index={source_slide_index}, \
                 slide_type='{slide_type}') — expected Multiple with 2 entries. \
                 The layout engine bailed on the first E-LAY-008 and never reported slide 2."
            );
        },
        other => panic!("F-094-P6-001: expected Multiple, got: {other:?}"),
    };

    assert_eq!(
        inner.len(),
        2,
        "F-094-P6-001: Multiple must contain exactly 2 BulletsOnContentlessSlideType entries \
         (one per contentless slide); got {}: {inner:?}",
        inner.len()
    );

    // Pin first entry: title slide (index 0).
    match &inner[0] {
        LayoutError::BulletsOnContentlessSlideType {
            slide_type,
            source_slide_index,
            ..
        } => {
            assert_eq!(
                slide_type.as_ref(),
                "title",
                "F-094-P6-001: first error must be for slide_type='title'; got '{slide_type}'"
            );
            assert_eq!(
                *source_slide_index, 0,
                "F-094-P6-001: first error source_slide_index must be 0; got {source_slide_index}"
            );
        },
        other => {
            panic!("F-094-P6-001: inner[0] must be BulletsOnContentlessSlideType, got: {other:?}")
        },
    }

    // Pin second entry: closing slide (index 2).
    match &inner[1] {
        LayoutError::BulletsOnContentlessSlideType {
            slide_type,
            source_slide_index,
            ..
        } => {
            assert_eq!(
                slide_type.as_ref(),
                "closing",
                "F-094-P6-001: second error must be for slide_type='closing'; got '{slide_type}'"
            );
            assert_eq!(
                *source_slide_index, 2,
                "F-094-P6-001: second error source_slide_index must be 2; got {source_slide_index}"
            );
        },
        other => {
            panic!("F-094-P6-001: inner[1] must be BulletsOnContentlessSlideType, got: {other:?}")
        },
    }
}

/// F-094-P6-001 (b) — mixed deck: one E-LAY-008 + valid slides.
///
/// A deck with ONE contentless-bullets slide and other valid slides must:
///   - Return Err (strict mode fails).
///   - Report the single instance (not silently drop it).
///   - NOT emit any output (since it's an error, run returns Err).
///
/// This test verifies that the fix didn't inadvertently suppress single-instance
/// E-LAY-008 errors.
#[test]
fn test_f094_p6_001_mixed_deck_single_e_lay_008_still_reported() {
    let items = || vec![flat_bullet(vec![InlineNode::Plain(Arc::from("item"))])];

    // Slide 0: content + bullets → valid
    let slide0 = bullets_slide("content", items());
    // Slide 1: title + bullets → E-LAY-008
    let slide1 = bullets_slide("title", items());

    let deck = make_deck(vec![slide0, slide1]);
    let brand = make_brand();

    let result = run(&deck, &brand);

    assert!(
        result.is_err(),
        "F-094-P6-001(b): mixed deck with one contentless-bullets slide must return Err"
    );
    let err = result.unwrap_err();

    // Must report the E-LAY-008 error (either directly or wrapped in Multiple).
    let is_e_lay_008 = match &err {
        LayoutError::BulletsOnContentlessSlideType { .. } => true,
        LayoutError::Multiple { inner } => inner
            .iter()
            .any(|e| matches!(e, LayoutError::BulletsOnContentlessSlideType { .. })),
        _ => false,
    };
    assert!(
        is_e_lay_008,
        "F-094-P6-001(b): error must be or contain BulletsOnContentlessSlideType; got: {err:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-094-P15-001 — Bullet width must be clamped to the owning body region right
// edge, not the page right edge.
// ─────────────────────────────────────────────────────────────────────────────

/// F-094-P15-001 (HIGH) — Two-col slide: every bullet frame must be contained
/// within the left-column body region.
///
/// two_col left-column body bbox (from regions.rs):
///   x=457_200, y=1_188_720, w=3_886_200, h=3_657_600
///   → region_right_edge = 457_200 + 3_886_200 = 4_343_400 EMU
///
/// DEFECT (pre-fix): `bullet_width = page_width − bullet_x = 9_144_000 − bullet_x`.
/// At depth-0 on two_col: `9_144_000 − 457_200 = 8_686_800` far exceeds the
/// column right edge 4_343_400, overlapping the right column by ~4.4 in.
///
/// CORRECT: `right_edge = body_bbox.x + body_bbox.width = 4_343_400`
///          `bullet_width = (right_edge − bullet_x).max(1)`
///          Depth-0: `4_343_400 − 457_200 = 3_886_200`
///
/// RED GATE: with page-relative width the right-containment assertion fails
/// (`bbox.x + bbox.width > body_right_edge`).
#[test]
fn test_f094_p15_001_two_col_bullet_width_contained_in_body_region() {
    // two_col left-body region constants (from regions.rs — must match exactly).
    const BODY_X: i64 = 457_200;
    const BODY_W: i64 = 3_886_200;
    const BODY_RIGHT_EDGE: i64 = BODY_X + BODY_W; // = 4_343_400

    let items = vec![
        flat_bullet(vec![InlineNode::Plain(Arc::from("col-left item one"))]),
        flat_bullet(vec![InlineNode::Plain(Arc::from("col-left item two"))]),
    ];
    let slide = bullets_slide("two_col", items);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand)
        .expect("F-094-P15-001: layout::run must succeed for two_col slide with bullets");

    let text_run_frames: Vec<_> = result.slides[0]
        .frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::TextRun(_)))
        .collect();

    assert_eq!(
        text_run_frames.len(),
        2,
        "F-094-P15-001: two_col slide with 2 bullet items must produce 2 TextRun frames; \
         got {} (total frames: {})",
        text_run_frames.len(),
        result.slides[0].frames.len()
    );

    for (idx, frame) in text_run_frames.iter().enumerate() {
        let bbox_x = frame.bbox.x.0;
        let bbox_w = frame.bbox.width.0;
        let right_edge = bbox_x.saturating_add(bbox_w);

        // Left containment: bullet must start at or after body_bbox.x.
        assert!(
            bbox_x >= BODY_X,
            "F-094-P15-001: frame {idx} bbox.x ({bbox_x}) must be >= body_bbox.x ({BODY_X}); \
             bullet starts before the owning body region"
        );
        // Right containment: bullet right edge must not exceed body region right edge.
        assert!(
            right_edge <= BODY_RIGHT_EDGE,
            "F-094-P15-001 RED GATE: frame {idx} right edge ({right_edge}) exceeds \
             body_right_edge ({BODY_RIGHT_EDGE}). \
             Defect: bullet_width = page_width − bullet_x ({}) instead of \
             body_right_edge − bullet_x ({}). \
             This overlaps the right column by {} EMU.",
            result.page_size.width.0 - bbox_x,
            BODY_RIGHT_EDGE - bbox_x,
            right_edge - BODY_RIGHT_EDGE
        );
    }
}

/// F-094-P15-001 (HIGH) — content slide: depth-1 bullet right edge must not
/// exceed the body region right edge (8_686_800 EMU).
///
/// content body bbox: x=457_200, w=8_229_600 → right_edge=8_686_800.
/// Page width: 9_144_000 EMU (0.5 in past body right edge).
///
/// At depth-0 the page-relative formula coincidentally gives the correct answer
/// for the content slide (both equal 8_686_800). The bug surfaces at depth≥1.
///
/// DEFECT (pre-fix): depth-1 bullet_x = 457_200 + 457_200 = 914_400;
///   page-relative width = 9_144_000 − 914_400 = 8_229_600;
///   body-relative width = 8_686_800 − 914_400 = 7_772_400.
///   Overshoot = 457_200 EMU (exactly 0.5 in past body right edge).
///
/// RED GATE: right-containment assertion fails for depth-1 on content.
#[test]
fn test_f094_p15_001_content_depth1_bullet_width_contained_in_body_region() {
    // content body bbox (from regions.rs).
    const BODY_X: i64 = 457_200;
    const BODY_W: i64 = 8_229_600;
    const BODY_RIGHT_EDGE: i64 = BODY_X + BODY_W; // = 8_686_800

    // A parent+child bullet: parent at depth-0, child at depth-1.
    let child = slideforge_types::BulletItem {
        inlines: vec![InlineNode::Plain(Arc::from("depth-1 child"))],
        children: vec![],
        span: slideforge_types::SourceSpan::default(),
    };
    let parent = slideforge_types::BulletItem {
        inlines: vec![InlineNode::Plain(Arc::from("depth-0 parent"))],
        children: vec![child],
        span: slideforge_types::SourceSpan::default(),
    };
    let slide = bullets_slide("content", vec![parent]);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand)
        .expect("F-094-P15-001: layout::run must succeed for content slide with depth-1 bullets");

    let text_run_frames: Vec<_> = result.slides[0]
        .frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::TextRun(_)))
        .collect();

    assert_eq!(
        text_run_frames.len(),
        2,
        "F-094-P15-001: parent+child bullet must produce 2 TextRun frames; \
         got {} (total: {})",
        text_run_frames.len(),
        result.slides[0].frames.len()
    );

    for (idx, frame) in text_run_frames.iter().enumerate() {
        let bbox_x = frame.bbox.x.0;
        let bbox_w = frame.bbox.width.0;
        let right_edge = bbox_x.saturating_add(bbox_w);
        assert!(
            right_edge <= BODY_RIGHT_EDGE,
            "F-094-P15-001 RED GATE (content depth-1): frame {idx} right edge ({right_edge}) \
             exceeds body_right_edge ({BODY_RIGHT_EDGE}). \
             page-relative overshoot = {} EMU (0.5 in at depth-1, more at deeper depths).",
            right_edge - BODY_RIGHT_EDGE
        );
    }
}

/// F-094-P15-001 (HIGH) — deep-indent boundary: near-MAX_BULLET_DEPTH on a narrow
/// two_col column must still produce BC-3.06.003-valid frames (width >= 1, x < body_right_edge).
///
/// At very high depth, bullet_x may approach or exceed body_right_edge.
/// The `.max(1)` width clamp and the BulletDepthExceeded guard (MAX_BULLET_DEPTH=64)
/// must produce a valid minimal frame rather than panic or zero-width.
///
/// two_col column width = 3_886_200 EMU (≈ 4.25 in).
/// BULLET_DEPTH_INDENT_EMU = 457_200 (0.5 in).
/// Depths where bullet_x < body_right_edge: max depth = floor(3_886_200 / 457_200) = 8.
/// Depth-8: bullet_x = 457_200 + 8*457_200 = 457_200 + 3_657_600 = 4_114_800; still < 4_343_400.
/// Depth-9: bullet_x = 457_200 + 9*457_200 = 457_200 + 4_114_800 = 4_572_000 > 4_343_400 → clamped.
///
/// This test exercises depth-8 (last depth inside the column) and verifies:
///   - width > 0 (not clamped to 0 by overflow)
///   - right_edge <= body_right_edge OR width == 1 (clamped-to-minimum case)
#[test]
fn test_f094_p15_001_two_col_deep_indent_boundary_produces_valid_frame() {
    // two_col left-body constants.
    const BODY_X: i64 = 457_200;
    const BODY_W: i64 = 3_886_200;
    const BODY_RIGHT_EDGE: i64 = BODY_X + BODY_W; // 4_343_400

    // Build a bullet nested 8 levels deep to approach the column boundary.
    let mut item = slideforge_types::BulletItem {
        inlines: vec![InlineNode::Plain(Arc::from("depth-8 leaf"))],
        children: vec![],
        span: slideforge_types::SourceSpan::default(),
    };
    // Wrap 8 times: depth-7 → … → depth-0 (top-level).
    for depth in (0..8u32).rev() {
        item = slideforge_types::BulletItem {
            inlines: vec![InlineNode::Plain(Arc::from(
                format!("depth-{depth}").as_str(),
            ))],
            children: vec![item],
            span: slideforge_types::SourceSpan::default(),
        };
    }

    let slide = bullets_slide("two_col", vec![item]);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand).expect(
        "F-094-P15-001 deep-indent: layout::run must succeed for two_col with 8-level nesting",
    );

    let text_run_frames: Vec<_> = result.slides[0]
        .frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::TextRun(_)))
        .collect();

    // 9 frames: depth-0 through depth-8.
    assert_eq!(
        text_run_frames.len(),
        9,
        "F-094-P15-001 deep-indent: 8-level nested bullet must produce 9 TextRun frames"
    );

    let page_w = result.page_size.width;
    let page_h = result.page_size.height;

    for (idx, frame) in text_run_frames.iter().enumerate() {
        // All frames must be BC-3.06.003 valid.
        assert!(
            frame.bbox.is_valid(page_w, page_h),
            "F-094-P15-001 deep-indent: frame {idx} must be BC-3.06.003 valid; got: {:?}",
            frame.bbox
        );
        assert!(
            frame.bbox.width.0 >= 1,
            "F-094-P15-001 deep-indent: frame {idx} width must be >= 1 EMU; got {}",
            frame.bbox.width.0
        );
        // Right containment: bullet right edge <= body_right_edge OR width == 1 (clamped minimum).
        let right_edge = frame.bbox.x.0.saturating_add(frame.bbox.width.0);
        assert!(
            right_edge <= BODY_RIGHT_EDGE || frame.bbox.width.0 == 1,
            "F-094-P15-001 deep-indent: frame {idx} right_edge ({right_edge}) exceeds \
             body_right_edge ({BODY_RIGHT_EDGE}) and width ({}) != 1 (clamped minimum). \
             Bullet must be contained within the owning body region.",
            frame.bbox.width.0
        );
    }
}
