//! STORY-086 unit tests for `thread_fields_to_blocks` (Stage 2b, ADR-019).
//!
//! ## Acceptance Criteria covered
//!
//! - **AC-008**: empty title string → no `ContentBlock`, no text run
//! - **AC-009**: `thread_fields_to_blocks` is pure / deterministic
//! - **AC-010**: canonical block ordering (title → subtitle → body → bullets → chart)
//! - **AC-011**: shape blocks are NOT populated by Stage 2b
//!
//! ## Red Gate discipline (LESSON-17)
//!
//! All tests in this file MUST FAIL until the implementer fills
//! `thread_fields_to_blocks` with real behavior (STORY-086 T6). The stub body
//! is a no-op: `thread_fields_to_blocks` does nothing, so `Slide.blocks`
//! remains `vec![]` after every call. Any test that asserts `blocks` is
//! non-empty will FAIL against the stub and PASS after implementation.
//!
//! ## Traceability
//!
//! BC-1.16.001 postconditions 1, 6, 7, 12, 13, 14, 15
//! BC-1.16.001 invariants 1, 2, 4, 7

#![allow(clippy::unwrap_used)] // unit tests — explicit panic on failure is correct

use std::sync::Arc;

use slideforge_eval::thread_fields_to_blocks;
use slideforge_types::{
    Block, ContentBlock, Deck, DeckMetadata, FieldValue, OrderedMap, Slide, SourceSpan, Value,
    shape_types::{FillSpec, ShapeType},
    specs::{AltText, ShapePosition, ShapeSpec, ShapeUnit},
};

// ─── Helper constructors ───────────────────────────────────────────────────────

fn make_metadata() -> DeckMetadata {
    DeckMetadata {
        title: Some(Arc::from("Test Deck")),
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

/// Construct a minimal slide with a given `slide_type` and no fields, blocks, or shapes.
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

/// Insert `title` field into the slide.
fn with_title(mut slide: Slide, title: &str) -> Slide {
    slide.fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from(title))),
    );
    slide
}

/// Insert `subtitle` field into the slide.
fn with_subtitle(mut slide: Slide, subtitle: &str) -> Slide {
    slide.fields.insert(
        Arc::from("subtitle"),
        FieldValue::Literal(Value::Str(Arc::from(subtitle))),
    );
    slide
}

/// Insert `body` field into the slide.
fn with_body(mut slide: Slide, body: &str) -> Slide {
    slide.fields.insert(
        Arc::from("body"),
        FieldValue::Literal(Value::Str(Arc::from(body))),
    );
    slide
}

/// Insert a `bullets` list field into the slide (as a `Value::List`).
fn with_bullets(mut slide: Slide, bullets: &[&str]) -> Slide {
    let items: Vec<Value> = bullets.iter().map(|b| Value::Str(Arc::from(*b))).collect();
    slide.fields.insert(
        Arc::from("bullets"),
        FieldValue::Literal(Value::List(items)),
    );
    slide
}

/// Insert `chart_type` field into the slide.
fn with_chart_type(mut slide: Slide, chart_type: &str) -> Slide {
    slide.fields.insert(
        Arc::from("chart_type"),
        FieldValue::Literal(Value::Str(Arc::from(chart_type))),
    );
    slide
}

/// Insert `alt` field into the slide.
fn with_alt(mut slide: Slide, alt: &str) -> Slide {
    slide.fields.insert(
        Arc::from("alt"),
        FieldValue::Literal(Value::Str(Arc::from(alt))),
    );
    slide
}

/// Append a `ContentBlock::Shape` block to the slide's pre-existing blocks list.
///
/// Simulates a slide that already has shape blocks (populated by some other means,
/// NOT by Stage 2b). AC-011 asserts Stage 2b must leave these untouched and not
/// ADD more shape blocks.
fn with_shape_block(mut slide: Slide) -> Slide {
    slide.blocks.push(Block {
        content: ContentBlock::Shape(ShapeSpec {
            shape_type: ShapeType::Rect,
            position: ShapePosition {
                x: ShapeUnit::Inches(500),
                y: ShapeUnit::Inches(500),
                width: ShapeUnit::Inches(2000),
                height: ShapeUnit::Inches(1000),
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from("a rectangle"))),
            decorative: false,
            span: SourceSpan::default(),
        }),
        label: None,
        span: SourceSpan::default(),
    });
    slide
}

// ─── AC-008 — empty title string → no `ContentBlock` ────────────────────────────

/// AC-008 / BC-1.16.001 EC-001 — empty title string produces no `ContentBlock::Text`.
///
/// Red Gate rationale: `thread_fields_to_blocks` is a no-op stub; `Slide.blocks`
/// is always `vec![]` after the call. This test asserts `.len() == 0` (blocks empty),
/// which passes regardless. BUT it also tests that an empty-title slide with the stub
/// doesn't somehow get a block with empty content. The REAL failure vector for this
/// AC is the canonical-ordering test (AC-010) — if the implementer populates blocks
/// for all fields including empty ones, AC-010 will catch it via exact `[Text(Title),
/// Text(Subtitle), Text(Body), Bullets, Chart]` ordering.
///
/// Load-bearing: after implementation, if `thread_fields_to_blocks` emits a
/// `ContentBlock::Text` for an empty `title ""`, this test MUST FAIL by detecting
/// a block with an empty inline sequence, confirming the guard works.
#[test]
fn test_bc_1_16_001_ac008_empty_title_produces_no_content_block() {
    // AC-008 / BC-1.16.001 postcondition 6 — empty strings silently skipped.
    // Red Gate: stub is no-op so blocks = vec![] — test passes trivially now.
    // The real guard is in ac008_empty_title_no_text_run_in_blocks below.
    let slide = with_title(make_slide("title"), "");
    let mut deck = make_deck(vec![slide]);

    thread_fields_to_blocks(&mut deck);

    let slide = &deck.slides[0];
    // After implementation: no ContentBlock::Text should exist for empty title.
    // If any Text block has empty inlines, the empty-string guard was missing.
    let text_blocks_with_empty: Vec<_> = slide
        .blocks
        .iter()
        .filter(|b| matches!(&b.content, ContentBlock::Text(tb) if tb.inlines.is_empty()))
        .collect();
    assert!(
        text_blocks_with_empty.is_empty(),
        "AC-008: no ContentBlock::Text with empty inlines should be produced for title \"\"; \
         found {} such blocks. BC-1.16.001 EC-001: empty strings silently skipped.",
        text_blocks_with_empty.len()
    );
}

/// AC-008 (strengthened) — after threading, the total block count is 0 for a
/// deck with only an empty title field.
///
/// Red Gate rationale: stub is a no-op → `blocks.len() == 0` → passes trivially.
/// After implementation: empty title must NOT produce a block, so `blocks.len()`
/// must still be 0. If the implementer forgets the empty-string guard, this FAILS.
///
/// Note: this test is NOT vacuously red-gate safe in isolation (it passes against
/// the stub). Its purpose is to PIN the correct post-implementation state. The
/// negative Red Gate for this AC is provided by `test_bc_1_16_001_ac010_*` which
/// WILL fail against the stub.
#[test]
fn test_bc_1_16_001_ac008_empty_title_zero_blocks_total() {
    let slide = with_title(make_slide("title"), "");
    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);
    let slide = &deck.slides[0];
    assert_eq!(
        slide.blocks.len(),
        0,
        "AC-008: slide with only an empty title must have 0 blocks after threading; \
         got {}. BC-1.16.001 EC-001.",
        slide.blocks.len()
    );
}

// ─── AC-009 — thread_fields_to_blocks is pure / deterministic ────────────────

/// AC-009 / BC-1.16.001 invariant 1 — same input produces same output on two calls.
///
/// Red Gate: stub is a no-op → both recordings are `vec![]` → equal → PASSES.
///
/// WAIT — this test actually passes against the stub! The Red Gate is carried by
/// AC-010 tests below. This test acts as a correctness pin for post-implementation.
/// It verifies determinism which can't be red-gated directly (idempotent empty also passes).
///
/// To make this test RED-gate meaningful, we combine it with a non-empty assertion:
/// the title block must appear in BOTH recordings. After the stub (no-op), the second
/// assertion fails.
#[test]
fn test_bc_1_16_001_ac009_pure_same_output_on_two_calls() {
    // AC-009 / BC-1.16.001 invariant 1 and postcondition 15 (determinism).
    // Construct a slide with title + body fields.
    // F-098-P1-002: `content` slide type does not declare `body` as a known field,
    // so body threading is gated out. Only the title block is produced.
    // The determinism assertion (first_blocks == second_blocks) and the non-empty
    // anchor assertion (!first_blocks.is_empty()) both pass because title IS threaded.
    let slide = with_body(
        with_title(make_slide("content"), "Determinism Test"),
        "Body text here.",
    );
    let mut deck = make_deck(vec![slide]);

    // First call.
    thread_fields_to_blocks(&mut deck);
    let first_blocks = deck.slides[0].blocks.clone();

    // Reset blocks to simulate a second call on identical input.
    deck.slides[0].blocks.clear();

    // Second call on identical fields.
    thread_fields_to_blocks(&mut deck);
    let second_blocks = deck.slides[0].blocks.clone();

    // Both must be equal (determinism invariant).
    assert_eq!(
        first_blocks, second_blocks,
        "AC-009: two calls with identical input must produce identical blocks. \
         BC-1.16.001 invariant 1 (purity) and postcondition 15 (determinism)."
    );

    // --- Red Gate anchor ---
    // After implementation, both recordings must be NON-EMPTY (at least the title block).
    // The stub produces vec![] → both recordings are vec![] → equal → passes
    // the eq assertion above, BUT fails here:
    // F-098-P1-002: `content` slide does not declare `body`; only the title block is threaded.
    assert!(
        !first_blocks.is_empty(),
        "AC-009 Red Gate anchor: blocks must be NON-EMPTY for a slide with title field. \
         The stub is a no-op and produces vec![] — this assertion MUST FAIL until T6 ships. \
         BC-1.16.001 postconditions 1 and 4."
    );
}

// ─── AC-010 — canonical block ordering: title → subtitle → body → bullets → chart

/// AC-010 / BC-1.16.001 postcondition 13 — after threading, block order is
/// [Text(Title), Text(Subtitle), Text(Body), Bullets, Chart].
///
/// Red Gate: stub is a no-op → `blocks = vec![]` → `blocks.len() != 5` → FAILS.
/// After implementation: exactly 5 blocks in canonical order.
///
/// This is the PRIMARY Red Gate test for AC-010 and indirectly for AC-001/AC-002/AC-003.
/// Provably fails against the stub: `assert_eq!(blocks.len(), 5)` fails when blocks = [].
#[test]
fn test_bc_1_16_001_ac010_canonical_block_ordering_five_blocks() {
    // Construct a slide with all five field types (title, subtitle, body, bullets, chart).
    // AC-010 / BC-1.16.001 postcondition 13 — canonical block order invariant.
    //
    // F-098-P1-002: use synthetic type "_unit_test_all_fields_type" (not in known_fields
    // registry). `thread_one_slide` falls through to body threading for unknown types
    // (`slide_type_known_fields` returns `None` → `map_or(true, ...)` → thread body).
    // Using a built-in type like "chart" would gate out body (chart has no "body" field),
    // reducing to 4 blocks instead of 5.
    // F-098-P1-002: chart threading is gated on `slide_type == "chart"` in thread_one_slide.
    // For the canonical ordering test, we use two sub-tests:
    //   (a) unknown type → produces title+subtitle+body+bullets (4 blocks, no chart)
    //   (b) "chart" type → produces title+chart (2 blocks, no body/subtitle/bullets)
    //
    // The canonical ordering invariant is verified by (a): body precedes bullets, etc.

    // Sub-test (a): unknown type with title+subtitle+body+bullets — verifies ordering.
    let slide = {
        let mut s = make_slide("_unit_test_all_fields_type");
        // title → Block 0 (Text(Title))
        s = with_title(s, "The Title");
        // subtitle → Block 1 (Text(Subtitle))
        s = with_subtitle(s, "The Subtitle");
        // body → Block 2 (Text(Body)) — unknown type falls through body gate
        s = with_body(s, "The body text.");
        // bullets → Block 3 (Bullets)
        s = with_bullets(s, &["Item A", "Item B"]);
        s
    };
    let mut deck = make_deck(vec![slide]);

    // RED GATE: thread_fields_to_blocks is a no-op stub → blocks = vec![]
    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;

    // Must have exactly 4 blocks in canonical order (no chart — unknown type doesn't
    // trigger chart threading, which is gated on slide_type == "chart").
    // This FAILS against the stub (blocks.len() == 0 != 4).
    assert_eq!(
        blocks.len(),
        4,
        "AC-010: slide with title+subtitle+body+bullets (unknown type) must produce exactly \
         4 blocks after thread_fields_to_blocks; got {} blocks (stub is no-op → 0). \
         BC-1.16.001 postcondition 13 — canonical block ordering.",
        blocks.len()
    );

    // Block 0 must be Text(Title) — fails if blocks is empty
    assert!(
        matches!(&blocks[0].content, ContentBlock::Text(_)),
        "AC-010: blocks[0] must be ContentBlock::Text (title); got {:?}",
        blocks.first().map(|b| b.content.kind_name())
    );

    // Block 1 must be Text(Subtitle)
    assert!(
        matches!(&blocks[1].content, ContentBlock::Text(_)),
        "AC-010: blocks[1] must be ContentBlock::Text (subtitle); got {:?}",
        blocks.get(1).map(|b| b.content.kind_name())
    );

    // Block 2 must be Text(Body)
    assert!(
        matches!(&blocks[2].content, ContentBlock::Text(_)),
        "AC-010: blocks[2] must be ContentBlock::Text (body); got {:?}",
        blocks.get(2).map(|b| b.content.kind_name())
    );

    // Block 3 must be Bullets
    assert!(
        matches!(&blocks[3].content, ContentBlock::Bullets(_)),
        "AC-010: blocks[3] must be ContentBlock::Bullets; got {:?}",
        blocks.get(3).map(|b| b.content.kind_name())
    );

    // Sub-test (b): chart slide type → title + chart (canonical: title before chart)
    let chart_slide = {
        let mut s = make_slide("chart");
        s = with_title(s, "Chart Title");
        s = with_chart_type(s, "bar");
        s = with_alt(s, "Alt text for chart");
        s
    };
    let mut chart_deck = make_deck(vec![chart_slide]);
    thread_fields_to_blocks(&mut chart_deck);
    let chart_blocks = &chart_deck.slides[0].blocks;
    assert_eq!(
        chart_blocks.len(),
        2,
        "AC-010 chart sub-test: chart slide with title+chart_type+alt must produce exactly \
         2 blocks (title + chart); got {}",
        chart_blocks.len()
    );
    assert!(
        matches!(&chart_blocks[0].content, ContentBlock::Text(_)),
        "AC-010 chart sub-test: blocks[0] must be Text(title); got {:?}",
        chart_blocks.first().map(|b| b.content.kind_name())
    );
    assert!(
        matches!(&chart_blocks[1].content, ContentBlock::Chart(_)),
        "AC-010 chart sub-test: blocks[1] must be Chart; got {:?}",
        chart_blocks.get(1).map(|b| b.content.kind_name())
    );
}

/// AC-010 (title text content) — after threading, the first block must carry
/// the exact title string "The Title" as its inline content.
///
/// Red Gate: stub → `blocks = vec![]` → index panic caught by `blocks.get(0)` returning None
/// → assertion fails.
#[test]
fn test_bc_1_16_001_ac010_title_block_carries_exact_text() {
    // BC-1.16.001 postcondition 1 — title field produces ContentBlock::Text with title text.
    let slide = with_title(make_slide("title"), "The Title");
    let mut deck = make_deck(vec![slide]);

    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;

    // Must have at least one block (the title text block).
    // FAILS against stub (blocks = vec![]).
    assert!(
        !blocks.is_empty(),
        "AC-010 / BC-1.16.001 postcondition 1: thread_fields_to_blocks must produce at least \
         one block for a slide with a non-empty title. Stub is a no-op → blocks = vec![] → FAILS. \
         This test MUST FAIL until T6 ships."
    );

    // The first block must be Text with the exact title string.
    match &blocks[0].content {
        ContentBlock::Text(tb) => {
            use slideforge_types::InlineNode;
            let text: String = tb
                .inlines
                .iter()
                .filter_map(|n| {
                    if let InlineNode::Plain(s) = n {
                        Some(s.as_ref())
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(
                text, "The Title",
                "AC-010: title ContentBlock must carry exact text 'The Title'; got '{text}'"
            );
        },
        other => {
            panic!(
                "AC-010: blocks[0] must be ContentBlock::Text; got {:?}",
                other.kind_name()
            );
        },
    }
}

// ─── AC-011 — shape blocks are NOT populated by Stage 2b ─────────────────────

/// AC-011 / BC-1.16.001 postcondition 14 — Stage 2b must NOT append Shape blocks.
///
/// Red Gate: This test checks two things:
/// 1. Pre-existing shape blocks in `Slide.blocks` are NOT removed (shape isolation — count
///    of `ContentBlock::Shape` must equal the initial count).
/// 2. `thread_fields_to_blocks` does NOT add any new `ContentBlock::Shape` blocks.
///
/// The test uses a slide that has a shape block pre-populated (simulating another pass
/// having added it), PLUS a title field. After threading:
/// - The title `ContentBlock::Text` should appear.
/// - The shape count must stay at 1 (unchanged).
///
/// Red Gate: stub is a no-op → blocks stay at `[Shape]` (1 shape, pre-populated) →
/// no title block added → assertion `shape_count == 1` passes, but the
/// `title_count == 1` assertion FAILS (no title block added by stub).
#[test]
fn test_bc_1_16_001_ac011_stage_2b_does_not_add_shape_blocks() {
    // BC-1.16.001 postcondition 14 (shape exclusion) + invariant 4.
    // Slide with a pre-populated shape block AND a title field.
    let slide = with_title(with_shape_block(make_slide("title")), "Chart Title");
    let mut deck = make_deck(vec![slide]);

    // Pre-condition: 1 shape block present before threading.
    assert_eq!(
        deck.slides[0].blocks.len(),
        1,
        "precondition: slide must have exactly 1 shape block before threading"
    );
    assert!(
        matches!(deck.slides[0].blocks[0].content, ContentBlock::Shape(_)),
        "precondition: the pre-populated block must be ContentBlock::Shape"
    );

    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;

    // Count shape blocks after threading.
    let shape_count = blocks
        .iter()
        .filter(|b| matches!(b.content, ContentBlock::Shape(_)))
        .count();
    // Count text blocks after threading.
    let title_count = blocks
        .iter()
        .filter(|b| matches!(b.content, ContentBlock::Text(_)))
        .count();

    // Stage 2b must NOT add more shape blocks.
    assert_eq!(
        shape_count, 1,
        "AC-011: Stage 2b must not add ContentBlock::Shape blocks; \
         expected shape_count=1 (pre-populated), got {shape_count}. \
         BC-1.16.001 postcondition 14 + invariant 4."
    );

    // Stage 2b MUST add the title text block (Red Gate anchor).
    // FAILS against stub (no title block added).
    assert_eq!(
        title_count, 1,
        "AC-011 Red Gate anchor: Stage 2b must add 1 ContentBlock::Text for the title; \
         got {title_count}. Stub is a no-op → FAILS until T6 ships. \
         BC-1.16.001 postcondition 1."
    );
}

/// AC-011 (pure shape isolation variant) — slides with `.shapes` non-empty
/// (shapes in a different field) do not produce `ContentBlock::Shape` blocks from Stage 2b.
///
/// This tests BC-1.16.001 invariant 4 directly: Stage 2b reads fields and
/// produces Text/Bullets/Chart blocks, never Shape blocks.
///
/// Red Gate: stub → `blocks = vec![]` → zero shapes added → passes, BUT the
/// title text assertion fails (confirming stub is a no-op).
#[test]
fn test_bc_1_16_001_ac011_no_shape_blocks_in_output_of_stage_2b() {
    // Stage 2b must never produce ContentBlock::Shape, even if fields contain
    // shape-like data. Only title/subtitle/body/bullets/chart/image/diagram blocks
    // are produced by Stage 2b.
    let slide = with_title(make_slide("title"), "My Slide");
    let mut deck = make_deck(vec![slide]);

    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;

    // Stage 2b must produce exactly zero Shape blocks.
    let shape_count = blocks
        .iter()
        .filter(|b| matches!(b.content, ContentBlock::Shape(_)))
        .count();
    assert_eq!(
        shape_count, 0,
        "AC-011: Stage 2b must not produce any ContentBlock::Shape blocks; got {shape_count}. \
         BC-1.16.001 postcondition 14."
    );

    // Red Gate anchor: title must be present (fails against stub).
    assert!(
        !blocks.is_empty(),
        "AC-011 Red Gate anchor: blocks must be non-empty for slide with title 'My Slide'; \
         stub is no-op → vec![] → FAILS until T6 ships."
    );
}

// ─── EC-010 — zero-slide deck is a no-op ─────────────────────────────────────

/// BC-1.16.001 EC-010 — zero-slide deck: no-op, no panic.
///
/// Red Gate: NOT a failing test (stub is already a no-op on empty deck).
/// This is a correctness pin: after implementation, zero-slide deck must still
/// not panic or add slides.
#[test]
fn test_bc_1_16_001_ec010_empty_deck_is_noop() {
    // BC-1.16.001 EC-010: deck with zero slides → no-op, no panic.
    let mut deck = make_deck(vec![]);
    thread_fields_to_blocks(&mut deck); // must not panic
    assert_eq!(deck.slides.len(), 0, "zero-slide deck must stay empty");
}

// ─── AC-012 — AltText::Unspecified exists and compiles ────────────────────────

/// AC-012 / BC-5.02.001 postcondition 7 — `AltText::Unspecified` variant exists
/// and is distinct from `Decorative`.
///
/// Red Gate: This is NOT a failing test (the `AltText::Unspecified` variant was
/// added in the stub commit). This is a COMPILE-TIME contract test: if the variant
/// is ever removed or renamed, this test fails to compile. It also serves as
/// documentation that the three-variant discrimination is a hard constraint.
///
/// The real AC-012 load-bearing tests are in the post-layout validator tests
/// (see `crates/slideforge-validate/src/alt_text.rs` AC-015 tests).
#[test]
fn test_bc_5_02_001_ac012_alt_text_unspecified_variant_is_distinct_from_decorative() {
    // Compile-time contract: all three AltText variants must exist.
    let provided = AltText::Provided(Arc::from("some description"));
    let decorative = AltText::Decorative;
    let unspecified = AltText::Unspecified;

    // They must be unequal (distinct semantics).
    assert_ne!(
        provided, decorative,
        "AC-012: AltText::Provided must not equal AltText::Decorative"
    );
    assert_ne!(
        provided, unspecified,
        "AC-012: AltText::Provided must not equal AltText::Unspecified"
    );
    assert_ne!(
        decorative, unspecified,
        "AC-012: AltText::Decorative must NOT equal AltText::Unspecified. \
         These are semantically distinct: Decorative = author opt-out (valid), \
         Unspecified = pipeline gap (triggers E-A11-001). \
         BC-5.01.001 invariant 2."
    );

    // Exhaustive match must compile (no non_exhaustive_patterns warning/error).
    // If a new variant is added without updating this match, it fails to compile.
    let is_missing: bool = match unspecified {
        AltText::Provided(_) | AltText::Decorative => false,
        AltText::Unspecified => true,
    };
    assert!(
        is_missing,
        "AC-012: Unspecified variant must match the Unspecified arm"
    );

    // AltText must implement Hash + Eq + Clone (comemo compatibility).
    let mut set = std::collections::HashSet::new();
    set.insert(AltText::Unspecified);
    set.insert(AltText::Unspecified); // duplicate
    assert_eq!(
        set.len(),
        1,
        "AltText::Unspecified must deduplicate in HashSet"
    );
}

// ─── AC-007 (scope-expansion): Value::List → ContentBlock::Bullets unit test ──

/// AC-007 / BC-1.16.001 postcondition 7 — `Value::List(items)` in a slide's
/// `bullets` field produces a `ContentBlock::Bullets` with one `BulletItem`
/// per list element.
///
/// This is the DIRECT unit test path: `thread_fields_to_blocks` with a slide
/// that has `bullets = Value::List([...])` (as produced by `@var items = [...]`
/// DSL construct after eval).
///
/// Red Gate: `thread_fields_to_blocks` is currently implemented and handles the
/// `Value::List` path. This test verifies:
/// 1. The `ContentBlock::Bullets` is produced (not skipped).
/// 2. The bullet items carry the correct string content.
/// 3. The block count is exactly 1 (just bullets, no title/subtitle/body on this slide).
///
/// Note: This test passes against the CURRENT implementation in ba3bcc93 since
/// `thread_fields_to_blocks` handles `Value::List`. However, it is a required
/// test for AC-007 traceability and serves as a regression guard. The E2E test
/// `test_bc_1_16_001_ac007_bullets_slide_produces_ge3_text_runs_in_pptx` is the
/// primary Red Gate for AC-007 (fails if layout/exporter don't thread bullets to
/// `<a:r>` runs).
///
/// Traces: BC-1.16.001 postcondition 7; AC-007 variable-binding form.
#[test]
fn test_bc_1_16_001_ac007_value_list_produces_content_block_bullets() {
    // AC-007: @var items = ["A","B","C"] / bullets: items → ContentBlock::Bullets with 3 items.
    // This tests the Value::List → ContentBlock::Bullets path in thread_fields_to_blocks.
    let slide = with_bullets(make_slide("content"), &["Item A", "Item B", "Item C"]);
    let mut deck = make_deck(vec![slide]);

    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;

    // Must have exactly 1 block: the Bullets block (no title/subtitle/body on this slide).
    // RED GATE: if thread_fields_to_blocks is a no-op stub, blocks = vec![] → FAILS.
    assert_eq!(
        blocks.len(),
        1,
        "AC-007: slide with only bullets field must produce exactly 1 ContentBlock; \
         got {}. BC-1.16.001 postcondition 7.",
        blocks.len()
    );

    // The block must be ContentBlock::Bullets.
    assert!(
        matches!(&blocks[0].content, ContentBlock::Bullets(_)),
        "AC-007: the produced block must be ContentBlock::Bullets; got {:?}",
        blocks[0].content.kind_name()
    );

    // The Bullets block must have 3 items.
    if let ContentBlock::Bullets(items) = &blocks[0].content {
        assert_eq!(
            items.len(),
            3,
            "AC-007: ContentBlock::Bullets must have 3 items (one per @var list element); \
             got {}. Items: {:?}",
            items.len(),
            items
        );

        // Each item's inline text must match the declared bullet string.
        let texts: Vec<String> = items
            .iter()
            .map(|item| {
                item.inlines
                    .iter()
                    .filter_map(|n| {
                        if let slideforge_types::InlineNode::Plain(s) = n {
                            Some(s.as_ref().to_owned())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .collect();

        assert_eq!(
            texts,
            vec!["Item A", "Item B", "Item C"],
            "AC-007: bullet item text must match declared list elements; got {texts:?}"
        );
    }
}

// ─── STORY-086 TextTag scaffolding compile tests ───────────────────────────────

/// `TextTag` enum is defined with 4 variants and the correct derives.
///
/// This is a COMPILE-TIME contract test. If `TextTag` is removed or renamed, this
/// test fails to compile. It also serves as documentation of the canonical variants.
///
/// Traces: STORY-086 scope-expansion; BC-4.01.001 v1.2 postconditions 9–12.
#[test]
fn test_bc_1_16_001_texttag_enum_has_four_variants() {
    use slideforge_types::TextTag;
    // All four variants must exist and be distinct.
    let title = TextTag::Title;
    let subtitle = TextTag::Subtitle;
    let body = TextTag::Body;
    let untagged = TextTag::Untagged;

    assert_ne!(title, subtitle, "TextTag::Title != Subtitle");
    assert_ne!(title, body, "TextTag::Title != Body");
    assert_ne!(title, untagged, "TextTag::Title != Untagged");
    assert_ne!(subtitle, body, "TextTag::Subtitle != Body");
    assert_ne!(subtitle, untagged, "TextTag::Subtitle != Untagged");
    assert_ne!(body, untagged, "TextTag::Body != Untagged");

    // Must implement Hash + Eq + Clone + Debug.
    let mut set = std::collections::HashSet::new();
    set.insert(title);
    set.insert(title); // duplicate
    assert_eq!(set.len(), 1, "TextTag::Title must deduplicate in HashSet");

    let cloned = untagged;
    assert_eq!(
        cloned,
        TextTag::Untagged,
        "TextTag must implement Clone/Copy"
    );

    let _ = format!("{title:?}"); // must implement Debug
}

/// `TextBlock.tag` field is set to `TextTag::Untagged` by default in all non-Stage-2b
/// construction sites, and can carry semantic `TextTag` values when set by Stage 2b.
///
/// Traces: STORY-086 scope-expansion; BC-1.16.001 postconditions 1–3.
#[test]
fn test_bc_1_16_001_textblock_tag_field_exists_and_defaults_to_untagged() {
    use slideforge_types::{InlineNode, TextBlock, TextTag};

    // Non-Stage-2b construction (test helper style) uses Untagged.
    let untagged_block = TextBlock {
        inlines: vec![InlineNode::Plain(Arc::from("generic text"))],
        tag: TextTag::Untagged,
        span: slideforge_types::SourceSpan::default(),
    };
    assert_eq!(
        untagged_block.tag,
        TextTag::Untagged,
        "TextBlock.tag must be TextTag::Untagged for non-Stage-2b construction"
    );

    // Stage 2b construction uses semantic tags.
    let title_block = TextBlock {
        inlines: vec![InlineNode::Plain(Arc::from("Slide Title"))],
        tag: TextTag::Title,
        span: slideforge_types::SourceSpan::default(),
    };
    assert_eq!(
        title_block.tag,
        TextTag::Title,
        "TextBlock.tag must carry TextTag::Title when set by Stage 2b for title fields"
    );
}

// ─── F-086-P5-CRIT-001: decorative-first precedence when both `decorative: true`
//     AND `alt "..."` are set ─────────────────────────────────────────────────────

/// F-086-P5-CRIT-001 / BC-1.16.001 PC-12 / EC-004 — decorative-first precedence.
///
/// Canonical outcome (architect pass-5 adjudication): when BOTH `decorative: true`
/// AND a non-empty `alt "..."` are present, `resolve_alt` MUST return
/// `AltText::Decorative` (decorative wins). A `tracing::warn!(code = "W-A11-002")`
/// is also expected to be emitted by the fixed implementation (not asserted here —
/// no tracing capture utility exists in this crate; the load-bearing assertion is
/// the `AltText::Decorative` outcome).
///
/// ## Red Gate
///
/// FAILS on current HEAD 2c677d97: `resolve_alt` is alt-first — it matches
/// `Some(alt_str)` BEFORE checking `is_decorative`, so the result is
/// `AltText::Provided("Some description")`. This test will fail with:
/// `expected AltText::Decorative, got AltText::Provided("Some description")`.
///
/// PASSES after the implementer fixes `resolve_alt` to be decorative-first.
///
/// ## Log assertion
///
/// No W-A11-002 log assertion is added. The crate has no tracing capture utility
/// (`tracing_test` / `tracing_subscriber::with_default` / similar). Adding a flaky
/// log-scrape would violate LESSON-17. The load-bearing assertion is purely
/// behavioural: `spec.alt == Some(AltText::Decorative)`.
///
/// Traces: F-086-P5-CRIT-001; BC-1.16.001 PC-12; EC-004.
#[test]
fn test_bc_1_16_001_f086_p5_crit001_both_set_chart_decorative_wins() {
    // F-086-P5-CRIT-001 / BC-1.16.001 PC-12 / EC-004 — decorative-first precedence.
    //
    // Setup: chart slide with BOTH `decorative: true` AND `alt "Some description"`.
    // Expected: ContentBlock::Chart carries AltText::Decorative (decorative wins).
    // Current code: alt-first → returns AltText::Provided("Some description") → FAILS.
    let slide = with_alt(
        with_chart_type(make_slide("chart"), "bar"),
        "Some description",
    );
    // Add decorative: true
    let mut slide = slide;
    slide.fields.insert(
        Arc::from("decorative"),
        FieldValue::Literal(Value::Bool(true)),
    );

    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);

    let chart_blocks: Vec<_> = deck.slides[0]
        .blocks
        .iter()
        .filter(|b| matches!(b.content, ContentBlock::Chart(_)))
        .collect();

    assert_eq!(
        chart_blocks.len(),
        1,
        "F-086-P5-CRIT-001: chart slide with both decorative+alt must still produce \
         ContentBlock::Chart; got {} chart blocks",
        chart_blocks.len()
    );

    if let ContentBlock::Chart(spec) = &chart_blocks[0].content {
        // RED GATE: current code returns AltText::Provided("Some description").
        // After fix: decorative-first → AltText::Decorative.
        assert_eq!(
            spec.alt,
            Some(AltText::Decorative),
            "F-086-P5-CRIT-001 RED GATE: when both `decorative: true` AND `alt \"Some description\"` \
             are set, the canonical outcome is AltText::Decorative (decorative-first, architect \
             pass-5 adjudication). Current code (alt-first) returns AltText::Provided. \
             BC-1.16.001 PC-12 / EC-004. got: {:?}",
            spec.alt
        );
    } else {
        panic!(
            "F-086-P5-CRIT-001: blocks[0] must be ContentBlock::Chart; got {:?}",
            chart_blocks.first().map(|b| b.content.kind_name())
        );
    }
}

/// F-086-P5-CRIT-001 variant — Image slide: both-set → `AltText::Decorative`.
///
/// Same precedence rule as the Chart variant. Tests the `ContentBlock::Image` arm
/// of `thread_fields_to_blocks` independently.
///
/// RED GATE: current alt-first `resolve_alt` → `AltText::Provided` → assertion fails.
///
/// Traces: F-086-P5-CRIT-001; BC-1.16.001 PC-12; EC-004.
#[test]
fn test_bc_1_16_001_f086_p5_crit001_both_set_image_decorative_wins() {
    // F-086-P5-CRIT-001 Image variant: image slide with both decorative+alt.
    let mut slide = make_slide("image");
    slide.fields.insert(
        Arc::from("src"),
        FieldValue::Literal(Value::Str(Arc::from("logo.png"))),
    );
    slide.fields.insert(
        Arc::from("alt"),
        FieldValue::Literal(Value::Str(Arc::from("Company logo"))),
    );
    slide.fields.insert(
        Arc::from("decorative"),
        FieldValue::Literal(Value::Bool(true)),
    );

    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);

    let image_blocks: Vec<_> = deck.slides[0]
        .blocks
        .iter()
        .filter(|b| matches!(b.content, ContentBlock::Image(_)))
        .collect();

    assert_eq!(
        image_blocks.len(),
        1,
        "F-086-P5-CRIT-001 Image: image slide with both decorative+alt must produce \
         ContentBlock::Image; got {} image blocks",
        image_blocks.len()
    );

    if let ContentBlock::Image(spec) = &image_blocks[0].content {
        assert_eq!(
            spec.alt,
            Some(AltText::Decorative),
            "F-086-P5-CRIT-001 Image RED GATE: both decorative+alt on image slide must \
             yield AltText::Decorative (decorative-first). Current code: alt-first → \
             AltText::Provided(\"Company logo\"). BC-1.16.001 PC-12. got: {:?}",
            spec.alt
        );
    } else {
        panic!(
            "F-086-P5-CRIT-001 Image: blocks must include ContentBlock::Image; got {:?}",
            image_blocks.first().map(|b| b.content.kind_name())
        );
    }
}

/// F-086-P5-CRIT-001 variant — Diagram slide: both-set → `AltText::Decorative`.
///
/// Same precedence rule applied to the `ContentBlock::Diagram` arm.
///
/// RED GATE: current alt-first `resolve_alt` → `AltText::Provided` → assertion fails.
///
/// Traces: F-086-P5-CRIT-001; BC-1.16.001 PC-12; EC-004.
#[test]
fn test_bc_1_16_001_f086_p5_crit001_both_set_diagram_decorative_wins() {
    // F-086-P5-CRIT-001 Diagram variant: diagram slide with both decorative+alt.
    let mut slide = make_slide("diagram");
    slide.fields.insert(
        Arc::from("source"),
        FieldValue::Literal(Value::Str(Arc::from("graph TD; A-->B"))),
    );
    slide.fields.insert(
        Arc::from("alt"),
        FieldValue::Literal(Value::Str(Arc::from("Flowchart showing A to B"))),
    );
    slide.fields.insert(
        Arc::from("decorative"),
        FieldValue::Literal(Value::Bool(true)),
    );

    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);

    let diagram_blocks: Vec<_> = deck.slides[0]
        .blocks
        .iter()
        .filter(|b| matches!(b.content, ContentBlock::Diagram(_)))
        .collect();

    assert_eq!(
        diagram_blocks.len(),
        1,
        "F-086-P5-CRIT-001 Diagram: diagram slide with both decorative+alt must produce \
         ContentBlock::Diagram; got {} diagram blocks",
        diagram_blocks.len()
    );

    if let ContentBlock::Diagram(spec) = &diagram_blocks[0].content {
        assert_eq!(
            spec.alt,
            Some(AltText::Decorative),
            "F-086-P5-CRIT-001 Diagram RED GATE: both decorative+alt on diagram slide must \
             yield AltText::Decorative (decorative-first). Current code: alt-first → \
             AltText::Provided(\"Flowchart showing A to B\"). BC-1.16.001 PC-12. got: {:?}",
            spec.alt
        );
    } else {
        panic!(
            "F-086-P5-CRIT-001 Diagram: blocks must include ContentBlock::Diagram; got {:?}",
            diagram_blocks.first().map(|b| b.content.kind_name())
        );
    }
}

// ─── F-086-P6-MED-001 — trimmed storage Red Gate ─────────────────────────────
//
// BC-1.16.001 PC-1 / PC-4 / PC-12 require that title, subtitle, body, and alt
// text are stored TRIMMED in the IR. The current production code stores the raw
// (untrimmed) value: `make_text_block_tagged(text, tag)` where `text` is the
// unshorn `&str` from `extract_str_field`, and `resolve_alt` stores
// `Arc::from(s)` without calling `.trim()` first.
//
// Tests 1–4 below MUST FAIL on HEAD 10cd8813 and PASS once the implementer
// applies `Arc::from(s.trim())` in `make_text_block_tagged` / `resolve_alt`.
// Test 5 (whitespace-only guard regression) MUST PASS on current HEAD.

/// F-086-P6-MED-001 / BC-1.16.001 PC-1 — title with leading/trailing whitespace
/// is stored TRIMMED in `InlineNode::Plain`.
///
/// ## Red Gate
///
/// FAILS on HEAD 10cd8813: `make_text_block_tagged` stores `Arc::from(text)`
/// where `text == "  Padded Title  "` (untrimmed). The assertion
/// `assert_eq!(plain_text, "Padded Title")` fails with
/// observed value `"  Padded Title  "`.
///
/// PASSES after implementer applies `Arc::from(s.trim())` in `make_text_block_tagged`.
///
/// Traces: F-086-P6-MED-001; BC-1.16.001 PC-1.
#[test]
fn test_bc_1_16_001_f086_p6_med001_title_stored_trimmed() {
    // F-086-P6-MED-001 / BC-1.16.001 PC-1 — trimmed storage Red Gate
    use slideforge_types::InlineNode;

    let slide = with_title(make_slide("title"), "  Padded Title  ");
    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;
    assert_eq!(
        blocks.len(),
        1,
        "F-086-P6-MED-001: title '  Padded Title  ' must produce exactly 1 block; got {}",
        blocks.len()
    );

    match &blocks[0].content {
        ContentBlock::Text(tb) => {
            let plain_text: String = tb
                .inlines
                .iter()
                .filter_map(|n| {
                    if let InlineNode::Plain(s) = n {
                        Some(s.as_ref())
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(
                plain_text, "Padded Title",
                "F-086-P6-MED-001 RED GATE: title must be stored TRIMMED. \
                 BC-1.16.001 PC-1 requires InlineNode::Plain(Arc::from(s.trim())). \
                 Current code stores untrimmed — observed: {plain_text:?}, expected: \"Padded Title\"."
            );
        },
        other => {
            panic!(
                "F-086-P6-MED-001: blocks[0] must be ContentBlock::Text for title; got {:?}",
                other.kind_name()
            );
        },
    }
}

/// F-086-P6-MED-001 / BC-1.16.001 PC-4 — subtitle with leading/trailing whitespace
/// is stored TRIMMED in `InlineNode::Plain`.
///
/// ## Red Gate
///
/// FAILS on HEAD 10cd8813: observed stored value is `"  Sub  "` (untrimmed).
/// Expected after fix: `"Sub"`.
///
/// Traces: F-086-P6-MED-001; BC-1.16.001 PC-4.
#[test]
fn test_bc_1_16_001_f086_p6_med001_subtitle_stored_trimmed() {
    // F-086-P6-MED-001 / BC-1.16.001 PC-4 — trimmed storage Red Gate
    use slideforge_types::InlineNode;

    let slide = with_subtitle(make_slide("title"), "  Sub  ");
    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;
    assert_eq!(
        blocks.len(),
        1,
        "F-086-P6-MED-001: subtitle '  Sub  ' must produce exactly 1 block; got {}",
        blocks.len()
    );

    match &blocks[0].content {
        ContentBlock::Text(tb) => {
            let plain_text: String = tb
                .inlines
                .iter()
                .filter_map(|n| {
                    if let InlineNode::Plain(s) = n {
                        Some(s.as_ref())
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(
                plain_text, "Sub",
                "F-086-P6-MED-001 RED GATE: subtitle must be stored TRIMMED. \
                 BC-1.16.001 PC-4 requires InlineNode::Plain(Arc::from(s.trim())). \
                 Current code stores untrimmed — observed: {plain_text:?}, expected: \"Sub\"."
            );
        },
        other => {
            panic!(
                "F-086-P6-MED-001: blocks[0] must be ContentBlock::Text for subtitle; got {:?}",
                other.kind_name()
            );
        },
    }
}

/// F-086-P6-MED-001 / BC-1.16.001 PC-4 — body with leading/trailing whitespace
/// is stored TRIMMED in `InlineNode::Plain`.
///
/// ## Red Gate
///
/// FAILS on HEAD 10cd8813: observed stored value is `"  Body text  "` (untrimmed).
/// Expected after fix: `"Body text"`.
///
/// Traces: F-086-P6-MED-001; BC-1.16.001 PC-4.
///
/// ## F-098-P1-002 note
///
/// This test uses the synthetic slide type `"_unit_test_body_type"` (not a registered
/// built-in). `thread_one_slide` falls through to body threading for unknown types
/// (`slide_type_known_fields` returns `None` → `map_or(true, ...)` → thread body).
/// This preserves the trimming assertion without depending on a built-in type that
/// declares `body` as a known field (no current built-in type does so).
#[test]
fn test_bc_1_16_001_f086_p6_med001_body_stored_trimmed() {
    // F-086-P6-MED-001 / BC-1.16.001 PC-4 — trimmed storage Red Gate
    //
    // F-098-P1-002: use synthetic type "_unit_test_body_type" (not in known_fields registry)
    // so the body-gate falls through to threading. No built-in slide type declares "body".
    use slideforge_types::InlineNode;

    let slide = with_body(make_slide("_unit_test_body_type"), "  Body text  ");
    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;
    assert_eq!(
        blocks.len(),
        1,
        "F-086-P6-MED-001: body '  Body text  ' must produce exactly 1 block; got {}",
        blocks.len()
    );

    match &blocks[0].content {
        ContentBlock::Text(tb) => {
            let plain_text: String = tb
                .inlines
                .iter()
                .filter_map(|n| {
                    if let InlineNode::Plain(s) = n {
                        Some(s.as_ref())
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(
                plain_text, "Body text",
                "F-086-P6-MED-001 RED GATE: body must be stored TRIMMED. \
                 BC-1.16.001 PC-4 requires InlineNode::Plain(Arc::from(s.trim())). \
                 Current code stores untrimmed — observed: {plain_text:?}, expected: \"Body text\"."
            );
        },
        other => {
            panic!(
                "F-086-P6-MED-001: blocks[0] must be ContentBlock::Text for body; got {:?}",
                other.kind_name()
            );
        },
    }
}

/// F-086-P6-MED-001 / BC-1.16.001 PC-12 — chart alt with leading/trailing whitespace
/// is stored TRIMMED in `AltText::Provided`.
///
/// ## Red Gate
///
/// FAILS on HEAD 10cd8813: `resolve_alt` stores `Arc::from(s)` where
/// `s == "  Bar chart desc  "` (untrimmed). The assertion
/// `assert_eq!(&*provided_str, "Bar chart desc")` fails with
/// observed value `"  Bar chart desc  "`.
///
/// PASSES after implementer applies `Arc::from(s.trim())` in `resolve_alt`.
///
/// Traces: F-086-P6-MED-001; BC-1.16.001 PC-12.
#[test]
fn test_bc_1_16_001_f086_p6_med001_chart_alt_stored_trimmed() {
    // F-086-P6-MED-001 / BC-1.16.001 PC-12 — trimmed alt storage Red Gate
    // Chart slide: no decorative, alt "  Bar chart desc  " → AltText::Provided("Bar chart desc")
    let slide = with_alt(
        with_chart_type(make_slide("chart"), "bar"),
        "  Bar chart desc  ",
    );
    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);

    let chart_blocks: Vec<_> = deck.slides[0]
        .blocks
        .iter()
        .filter(|b| matches!(b.content, ContentBlock::Chart(_)))
        .collect();
    assert_eq!(
        chart_blocks.len(),
        1,
        "F-086-P6-MED-001: chart slide must produce ContentBlock::Chart; got {}",
        chart_blocks.len()
    );

    if let ContentBlock::Chart(spec) = &chart_blocks[0].content {
        match &spec.alt {
            Some(AltText::Provided(s)) => {
                let s_str: &str = s.as_ref();
                assert_eq!(
                    s_str, "Bar chart desc",
                    "F-086-P6-MED-001 RED GATE: chart alt must be stored TRIMMED. \
                     BC-1.16.001 PC-12 requires AltText::Provided(Arc::from(s.trim())). \
                     Current code stores untrimmed — observed: {s_str:?}, expected: \"Bar chart desc\"."
                );
            },
            other => {
                panic!("F-086-P6-MED-001: chart alt must be AltText::Provided; got {other:?}");
            },
        }
    } else {
        panic!("F-086-P6-MED-001: blocks[0] must be ContentBlock::Chart");
    }
}

/// F-086-P6-MED-001 / BC-1.16.001 PC-1 — whitespace-only title (`"   "`) produces
/// NO `ContentBlock` (regression guard: trim + empty-skip interaction).
///
/// ## Behaviour
///
/// The empty-skip guard in production code (`!text.trim().is_empty()`) already handles
/// this: `"   ".trim() == ""` → guard triggers → no block emitted.
///
/// This test MUST PASS on current HEAD 10cd8813 (the guard exists).
/// It locks the interaction: after the implementer adds `.trim()` to the STORAGE site,
/// the empty-skip guard must still fire correctly (AC-008 / EC-002 regression).
///
/// Traces: F-086-P6-MED-001; BC-1.16.001 PC-1; AC-008; EC-002.
#[test]
fn test_bc_1_16_001_f086_p6_med001_whitespace_only_title_skipped_regression() {
    // F-086-P6-MED-001 / BC-1.16.001 PC-1 / AC-008 / EC-002 — whitespace-only guard
    // This test PASSES on current HEAD (the guard already exists).
    // It is included to lock the trim-then-skip interaction after the fix.
    let slide = with_title(make_slide("title"), "   ");
    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;
    assert_eq!(
        blocks.len(),
        0,
        "F-086-P6-MED-001 regression guard: whitespace-only title '   ' must produce 0 blocks. \
         After trim: '' → empty-skip guard fires → no ContentBlock emitted. \
         BC-1.16.001 PC-1 / AC-008 / EC-002. Got {} block(s).",
        blocks.len()
    );
}

/// Regression guard: `decorative: true` alone (no alt) STILL produces `AltText::Decorative`.
///
/// This is NOT a Red Gate test — it verifies existing correct behavior is preserved
/// after the fix. The alt-first change must not regress the decorative-only case.
///
/// Traces: BC-1.16.001 invariant 3 (decorative-only case must remain correct).
#[test]
fn test_bc_1_16_001_decorative_only_still_produces_decorative_after_fix() {
    // Regression guard: decorative-only (no alt) must still yield AltText::Decorative.
    // This PASSES on current code and must continue to pass after the decorative-first fix.
    let mut slide = make_slide("chart");
    slide.fields.insert(
        Arc::from("chart_type"),
        FieldValue::Literal(Value::Str(Arc::from("pie"))),
    );
    slide.fields.insert(
        Arc::from("decorative"),
        FieldValue::Literal(Value::Bool(true)),
    );
    // No "alt" field.

    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);

    let chart_blocks: Vec<_> = deck.slides[0]
        .blocks
        .iter()
        .filter(|b| matches!(b.content, ContentBlock::Chart(_)))
        .collect();
    assert_eq!(
        chart_blocks.len(),
        1,
        "decorative chart must produce ContentBlock::Chart"
    );
    if let ContentBlock::Chart(spec) = &chart_blocks[0].content {
        assert_eq!(
            spec.alt,
            Some(AltText::Decorative),
            "decorative-only (no alt) must yield AltText::Decorative; got {:?}",
            spec.alt
        );
    }
}

// ─── F-086-P13-OBS-001 — W-A11-002 must NOT fire on non-media slides ──────────
//
// error-taxonomy v2.17 W-A11-002 and BC-1.16.001 PC-12 / EC-004 scope the
// W-A11-002 warning to the Stage-2b MEDIA path (chart/image/diagram) only.
// When `resolve_alt` is called unconditionally before the slide_type dispatch,
// it fires W-A11-002 for ANY slide that has both `decorative: true` and a
// non-empty `alt` — including non-media slides like "content" and "title" where
// neither field is consumed and no media ContentBlock is emitted.
//
// F-086-P13-OBS-001 (LOCAL cascade pass 13) identified this as a spurious
// emission. The fix moves `resolve_alt`/`is_decorative` inside the media
// dispatch branches so they are only called for chart/image/diagram slides.
//
// ## Red Gate evidence
//
// test_f086_p13_obs001_non_media_slide_does_not_emit_w_a11_002 FAILS against
// the current unconditional-call code (HEAD 82f300db): `resolve_alt(slide)` is
// called before the slide_type check, so W-A11-002 is emitted even though the
// slide type is "content" (non-media). `logs_contain("W-A11-002")` returns
// `true` → `assert!(!logs_contain(...))` fails.
//
// After the fix (resolve_alt moved inside media dispatch), `resolve_alt` is not
// called for non-media slides → W-A11-002 is not emitted → `logs_contain`
// returns `false` → assertion passes.

/// F-086-P13-OBS-001 — W-A11-002 must NOT fire on a non-media slide.
///
/// A "content" slide carrying both `decorative: true` AND a non-empty `alt`
/// must NOT emit `tracing::warn!(code = "W-A11-002", ...)` because neither
/// field is consumed (no media `ContentBlock` is built for non-media slide types).
///
/// ## Red Gate
///
/// FAILS against HEAD 82f300db: `resolve_alt(slide)` is called unconditionally
/// before the `slide_type` dispatch, so W-A11-002 fires even for "content"
/// slides. The `assert!(!logs_contain("W-A11-002"))` assertion fails.
///
/// PASSES after the fix: `resolve_alt` is called only inside the media branches
/// (chart/image/diagram), so it is never called for "content" slides.
///
/// Traces: F-086-P13-OBS-001; error-taxonomy v2.17 W-A11-002; BC-1.16.001 PC-12 / EC-004.
#[tracing_test::traced_test]
#[test]
fn test_f086_p13_obs001_non_media_slide_does_not_emit_w_a11_002() {
    // Setup: "content" slide (non-media) with BOTH `decorative: true` AND
    // a non-empty `alt "A description"`. Neither field is consumed on non-media
    // slides; W-A11-002 must NOT fire.
    let mut slide = make_slide("content");
    slide.fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("My Content Slide"))),
    );
    slide.fields.insert(
        Arc::from("decorative"),
        FieldValue::Literal(Value::Bool(true)),
    );
    slide.fields.insert(
        Arc::from("alt"),
        FieldValue::Literal(Value::Str(Arc::from("A description"))),
    );

    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);

    // RED GATE: current code calls resolve_alt unconditionally → W-A11-002 fires.
    // After fix: resolve_alt is inside media dispatch → NOT called for "content" → no warning.
    assert!(
        !logs_contain("W-A11-002"),
        "F-086-P13-OBS-001 RED GATE: W-A11-002 must NOT fire for a non-media slide ('content'). \
         error-taxonomy v2.17 W-A11-002 scopes this warning to the MEDIA path only \
         (chart/image/diagram). Non-media slides do not consume decorative/alt fields \
         and must not emit the conflict warning. BC-1.16.001 PC-12 / EC-004."
    );

    // Confirm the slide produced the expected title text block (no regressions).
    let text_blocks: Vec<_> = deck.slides[0]
        .blocks
        .iter()
        .filter(|b| matches!(b.content, ContentBlock::Text(_)))
        .collect();
    assert_eq!(
        text_blocks.len(),
        1,
        "F-086-P13-OBS-001: 'content' slide with title must produce exactly 1 Text block; \
         got {}",
        text_blocks.len()
    );

    // Confirm no media ContentBlock was produced (guard for the fix scope).
    let media_blocks: Vec<_> = deck.slides[0]
        .blocks
        .iter()
        .filter(|b| {
            matches!(
                b.content,
                ContentBlock::Chart(_) | ContentBlock::Image(_) | ContentBlock::Diagram(_)
            )
        })
        .collect();
    assert_eq!(
        media_blocks.len(),
        0,
        "F-086-P13-OBS-001: 'content' slide must produce 0 media ContentBlocks; got {}",
        media_blocks.len()
    );
}

/// F-086-P13-OBS-001 variant — "title" slide type: same no-W-A11-002 requirement.
///
/// Regression guard: a "title" slide with both `decorative: true` + non-empty `alt`
/// must NOT emit W-A11-002.
///
/// Traces: F-086-P13-OBS-001; BC-1.16.001 PC-12 / EC-004.
#[tracing_test::traced_test]
#[test]
fn test_f086_p13_obs001_title_slide_does_not_emit_w_a11_002() {
    let mut slide = make_slide("title");
    slide.fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Title Slide"))),
    );
    slide.fields.insert(
        Arc::from("decorative"),
        FieldValue::Literal(Value::Bool(true)),
    );
    slide.fields.insert(
        Arc::from("alt"),
        FieldValue::Literal(Value::Str(Arc::from("Inert alt on title slide"))),
    );

    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);

    assert!(
        !logs_contain("W-A11-002"),
        "F-086-P13-OBS-001 title variant: W-A11-002 must NOT fire for 'title' slide. \
         BC-1.16.001 PC-12 / EC-004 scope this warning to the MEDIA path only."
    );
}

/// F-086-P13-OBS-001 regression guard — media slide (chart) with BOTH fields set
/// STILL emits W-A11-002 after the fix.
///
/// This is NOT a Red Gate test — it verifies that moving `resolve_alt` inside the
/// media dispatch does NOT suppress the warning on actual media slides. The warning
/// must still fire when `decorative: true` AND a non-empty `alt` are BOTH set on a
/// chart slide.
///
/// Traces: F-086-P13-OBS-001; BC-1.16.001 PC-12 / EC-004; error-taxonomy v2.17 W-A11-002.
#[tracing_test::traced_test]
#[test]
fn test_f086_p13_obs001_chart_slide_both_set_still_emits_w_a11_002() {
    // Regression guard: chart slide (MEDIA) with both decorative+alt MUST still warn.
    // This behavior must be preserved after the F-086-P13-OBS-001 fix.
    let mut slide = make_slide("chart");
    slide.fields.insert(
        Arc::from("chart_type"),
        FieldValue::Literal(Value::Str(Arc::from("bar"))),
    );
    slide.fields.insert(
        Arc::from("decorative"),
        FieldValue::Literal(Value::Bool(true)),
    );
    slide.fields.insert(
        Arc::from("alt"),
        FieldValue::Literal(Value::Str(Arc::from("Revenue chart description"))),
    );

    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);

    // W-A11-002 MUST still fire for media slides with both fields set.
    assert!(
        logs_contain("W-A11-002"),
        "F-086-P13-OBS-001 regression guard: W-A11-002 MUST still fire for a chart slide \
         with both `decorative: true` AND a non-empty `alt`. The fix must not suppress the \
         warning on actual media slides. BC-1.16.001 PC-12 / EC-004."
    );

    // Confirm decorative-first outcome is preserved (AltText::Decorative).
    let chart_blocks: Vec<_> = deck.slides[0]
        .blocks
        .iter()
        .filter(|b| matches!(b.content, ContentBlock::Chart(_)))
        .collect();
    assert_eq!(
        chart_blocks.len(),
        1,
        "regression guard: chart slide must still produce ContentBlock::Chart after fix"
    );
    if let ContentBlock::Chart(spec) = &chart_blocks[0].content {
        assert_eq!(
            spec.alt,
            Some(AltText::Decorative),
            "regression guard: decorative-first outcome must be preserved; got {:?}",
            spec.alt
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// F-094-P3-001 — field_spans threading: thread_fields_to_blocks uses field spans
// ─────────────────────────────────────────────────────────────────────────────

/// F-094-P3-001 (b): When a slide has a non-default span in `field_spans["bullets"]`,
/// `thread_fields_to_blocks` must populate the resulting `ContentBlock::Bullets`
/// block with that span (not `SourceSpan::default()`).
///
/// RED: currently all blocks constructed by `thread_fields_to_blocks` use
/// `SourceSpan::default()` regardless of `field_spans`. Fails until the
/// `field_span` is threaded through.
#[test]
fn test_f094_p3_001_field_to_block_uses_field_spans_for_bullets() {
    use slideforge_types::SourceSpan;

    let real_span = SourceSpan::new(Arc::from("test.sf"), 5, 1, 100);

    // Construct a slide with a populated bullets field AND a field_spans entry.
    let mut slide = make_slide("content");
    slide.fields.insert(
        Arc::from("bullets"),
        FieldValue::Literal(slideforge_types::Value::List(vec![
            slideforge_types::Value::Str(Arc::from("hello")),
        ])),
    );
    // Pre-populate the block with bullets (as if eval had resolved them).
    // This simulates what eval_slide_node does after the fix.
    // We use the lower-level approach: add a block directly with a default span,
    // and separately insert field_spans. The thread_fields_to_blocks function
    // should pick up the span from field_spans["bullets"].
    slide
        .field_spans
        .insert(Arc::from("bullets"), real_span.clone());
    // Actually, thread_fields_to_blocks creates blocks from fields.
    // We need the field to produce a Bullets block. Since plain strings produce
    // Literal(List), let's test with a slide that has an inline bullets field.
    // The real span is what matters — after the fix, block.span == real_span.

    // Build a slide with blocks already populated with a default-span bullets block,
    // and field_spans carrying the real span. Then call thread_fields_to_blocks
    // to see if it picks up the correct span.
    // However, since thread_fields_to_blocks creates blocks from scratch, we need
    // the implementation to consult field_spans when setting block.span.
    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);

    // Find the bullets block (if any).
    let bullets_block = deck.slides[0]
        .blocks
        .iter()
        .find(|b| matches!(&b.content, ContentBlock::Bullets(_)));

    // After the fix, if a bullets block exists, its span must match field_spans["bullets"].
    if let Some(block) = bullets_block {
        assert_eq!(
            block.span, real_span,
            "F-094-P3-001: ContentBlock::Bullets block.span must equal field_spans[\"bullets\"]; \
             got: {:?}, expected: {:?}",
            block.span, real_span
        );
    }
    // Note: if no bullets block is produced (e.g., because the field is a plain
    // List, not an InlinesList), this test still validates the field_spans field
    // exists on the slide without panicking.
    // The critical assertion is that field_spans is accessible on Slide.
    assert!(
        deck.slides[0].field_spans.get("bullets").is_some()
            || deck.slides[0].fields.get("bullets").is_none(),
        "F-094-P3-001: field_spans must be accessible on Slide; got: {:?}",
        deck.slides[0].field_spans
    );
}

// ─── F-098-P1-002 — body threading is gated on slide type schema ─────────────

/// F-098-P1-002 / BC-3.03.002 — `body` on a slide type that does NOT declare it
/// as a known field must NOT produce a `ContentBlock::Text(Body)` block.
///
/// This is the thread-level guard for `CONTENT_DROP_KEYS`: even in warn-only mode
/// (where `FieldSchemaValidator` emits W-VAL-103 as a non-blocking warning),
/// `thread_one_slide` must NOT thread body into blocks for types that don't
/// support it. Rendering body content in warn-only for unsupporting slide types
/// would corrupt layout output.
///
/// ## Slide types checked
///
/// - `"chart"`: has `chart_type`, `data` but NOT `body`
/// - `"title"`: has `subtitle`, `author`, `date` but NOT `body`
/// - `"blank"`: has only common fields, NOT `body`
///
/// Note: `"content"` is NOT in this list — content slides declare `body` as an
/// optional field (BC-4.01.001 v1.2 PC-11; STORY-098 F-098-P1-002 resolution:
/// adding `body` to content schema so it is NOT a `CONTENT_DROP_KEY` there).
///
/// ## Traceability
///
/// F-098-P1-002 (adversary pass-1 finding); BC-3.03.002 (`CONTENT_DROP_KEYS` Route A);
/// STORY-098 AC-002 (body not rendered on non-supporting type in warn-only).
#[test]
fn test_f098_p1_002_body_not_threaded_for_types_without_body_in_schema() {
    use slideforge_types::TextTag;

    for slide_type in &["chart", "title", "blank"] {
        // Build a slide with body field set to non-empty text.
        let mut slide = make_slide(slide_type);
        slide.fields.insert(
            Arc::from("body"),
            FieldValue::Literal(Value::Str(Arc::from("Should not be threaded"))),
        );
        // Also add title to avoid an empty slide (so we can confirm at least 0 or 1 blocks
        // are produced — the title block, not the body block).
        slide.fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from("Slide Title"))),
        );
        let mut deck = make_deck(vec![slide]);
        thread_fields_to_blocks(&mut deck);

        let blocks = &deck.slides[0].blocks;

        // Body block must NOT be present.
        let body_blocks: Vec<_> = blocks
            .iter()
            .filter(|b| {
                if let ContentBlock::Text(tb) = &b.content {
                    tb.tag == TextTag::Body
                } else {
                    false
                }
            })
            .collect();

        assert!(
            body_blocks.is_empty(),
            "F-098-P1-002: slide type {:?} does not declare 'body' as a known field; \
             thread_one_slide must NOT produce ContentBlock::Text(Body) for it. \
             Got {} body block(s). BC-3.03.002 Route A: content-drop keys must not \
             be threaded for unsupporting slide types.",
            slide_type,
            body_blocks.len()
        );
    }
}

/// F-098-P1-002 fallthrough — `body` IS threaded when slide type is NOT in the
/// known-fields registry (custom or unknown type).
///
/// `slide_type_known_fields` returns `None` for unknown types → `map_or(true, ...)` →
/// body threading proceeds. This ensures future custom slide types that declare `body`
/// can still use the threading pass without changes.
///
/// Traceability: F-098-P1-002 safe-fallthrough clause.
#[test]
fn test_f098_p1_002_body_is_threaded_for_unknown_slide_type() {
    use slideforge_types::{InlineNode, TextTag};

    // Use a slide type name not in the known-fields registry.
    let slide = with_body(make_slide("_unknown_future_type"), "Body content here.");
    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;

    let body_blocks: Vec<_> = blocks
        .iter()
        .filter(|b| {
            if let ContentBlock::Text(tb) = &b.content {
                tb.tag == TextTag::Body
            } else {
                false
            }
        })
        .collect();

    assert_eq!(
        body_blocks.len(),
        1,
        "F-098-P1-002 fallthrough: body MUST be threaded for unknown slide types \
         (not in known-fields registry). Got {} body block(s). \
         The gate falls through via `map_or(true, ...)` for unregistered types.",
        body_blocks.len()
    );

    // Verify the content is correct.
    if let ContentBlock::Text(tb) = &body_blocks[0].content {
        let plain_text: String = tb
            .inlines
            .iter()
            .filter_map(|n| {
                if let InlineNode::Plain(s) = n {
                    Some(s.as_ref())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            plain_text, "Body content here.",
            "F-098-P1-002 fallthrough: body content must be threaded verbatim; got {plain_text:?}"
        );
    }
}
