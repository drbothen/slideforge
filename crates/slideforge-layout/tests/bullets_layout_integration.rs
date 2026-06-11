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
    clippy::expect_used
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
    // F-094-P1-002: use "content" (has Body slot; "title" returns Err(InvalidBoundingBox)).
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
    // F-094-P1-002: use "content" (has Body slot; "title" returns Err(InvalidBoundingBox)
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
    // F-094-P1-002: use "content" (has Body slot; "title" returns Err(InvalidBoundingBox)
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
    // F-094-P1-002: use "content" (has Body slot; "title" returns Err(InvalidBoundingBox)
    // even for empty-inlines items because the bbox check fires before inlines are read).
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
    // F-094-P1-002: use "content" (has Body slot; "title" returns Err(InvalidBoundingBox)).
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
    // F-094-P1-002: use "content" (has Body slot; "title" returns Err(InvalidBoundingBox)).
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
    // F-094-P1-002: use "content" (has Body slot; "title" returns Err(InvalidBoundingBox)
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
    // F-094-P1-002: use "content" (has Body slot; "title" returns Err(InvalidBoundingBox)
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
/// `Err(LayoutError::InvalidBoundingBox)` rather than silently placing frames
/// at (0,0).
///
/// DEFECT: current code hits Phase 3 (no Empty slot) and returns `None` →
/// uses the clamped `(x=0, y=0, width=page_width, height=914_400)` fallback,
/// which passes `is_valid` because x=0 is `>= 0`. This is a silent wrong
/// positioning that violates the "no silent fallback to ANY position not a
/// defined region" rule.
///
/// This test MUST FAIL against the unfixed code (run succeeds instead of erring).
/// It will pass only after the Phase-3 fallback is replaced with
/// `Err(LayoutError::InvalidBoundingBox)`.
///
/// Load-bearing: `.is_err()` fails when `run` returns `Ok(...)` with (0,0) frames.
#[test]
fn test_f094_p1_002_bullets_on_title_slide_returns_invalid_bbox_error() {
    let items = vec![flat_bullet(vec![InlineNode::Plain(Arc::from("item"))])];
    let slide = bullets_slide("title", items);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand);

    assert!(
        result.is_err(),
        "F-094-P1-002 Red Gate: bullets on a 'title' slide (no Body slot) must return \
         Err(LayoutError::InvalidBoundingBox); layout::run returned Ok(..)"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, LayoutError::InvalidBoundingBox { .. }),
        "F-094-P1-002 Red Gate: error must be LayoutError::InvalidBoundingBox; \
         got: {err:?}"
    );
}

/// F-094-P1-002 / BC-3.06.003 — closing slide type (no Body slot).
#[test]
fn test_f094_p1_002_bullets_on_closing_slide_returns_invalid_bbox_error() {
    let items = vec![flat_bullet(vec![InlineNode::Plain(Arc::from("item"))])];
    let slide = bullets_slide("closing", items);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand);

    assert!(
        result.is_err(),
        "F-094-P1-002 Red Gate: bullets on a 'closing' slide (no Body slot) must return \
         Err(LayoutError::InvalidBoundingBox); layout::run returned Ok(..)"
    );
    assert!(
        matches!(result.unwrap_err(), LayoutError::InvalidBoundingBox { .. }),
        "F-094-P1-002: error must be LayoutError::InvalidBoundingBox"
    );
}

/// F-094-P1-002 / BC-3.06.003 — `section_break` slide type (no Body slot).
#[test]
fn test_f094_p1_002_bullets_on_section_break_returns_invalid_bbox_error() {
    let items = vec![flat_bullet(vec![InlineNode::Plain(Arc::from("item"))])];
    let slide = bullets_slide("section_break", items);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand);

    assert!(
        result.is_err(),
        "F-094-P1-002 Red Gate: bullets on 'section_break' (no Body slot) must error"
    );
    assert!(
        matches!(result.unwrap_err(), LayoutError::InvalidBoundingBox { .. }),
        "F-094-P1-002: error must be LayoutError::InvalidBoundingBox"
    );
}

/// F-094-P1-002 / BC-3.06.003 — blank slide type (zero regions, no Body slot).
#[test]
fn test_f094_p1_002_bullets_on_blank_slide_returns_invalid_bbox_error() {
    let items = vec![flat_bullet(vec![InlineNode::Plain(Arc::from("item"))])];
    let slide = bullets_slide("blank", items);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand);

    assert!(
        result.is_err(),
        "F-094-P1-002 Red Gate: bullets on 'blank' (no slots at all) must error"
    );
    assert!(
        matches!(result.unwrap_err(), LayoutError::InvalidBoundingBox { .. }),
        "F-094-P1-002: error must be LayoutError::InvalidBoundingBox"
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
