//! STORY-087 pass-2 — Stage 2b threading Red Gate tests.
//!
//! Exercises `thread_fields_to_blocks` for color-coded slide types:
//! `status`, `progress_bar`, `weighted_composite`, and `stat_callout`.
//!
//! ## Red Gate discipline
//!
//! ALL tests in this file MUST FAIL until the implementer extends
//! `thread_fields_to_blocks` (STORY-087 TDD green pass). The current Stage 2b
//! only threads `title`, `subtitle`, `body`, `bullets`, and media fields.
//! It does NOT thread `label`, `value`, or `components` — so assertions for
//! `ColorLabel`, `ColorBar`, and per-component `Body` blocks will fail.
//!
//! ## Traceability
//!
//! - BC-1.17.001 PC-8: `label` → `ContentBlock::Text(TextTag::ColorLabel)` for `status`
//! - BC-1.17.002 PC-9: `label` → `ColorLabel`, `value` → `ContentBlock::ColorBar`
//! - BC-1.17.003 PC-9: `label` → `ColorLabel`, `components` → `ContentBlock::Text(Body)` × N
//! - Architect pass-2 adjudication §10.1 (STORY-087)

#![allow(clippy::unwrap_used)] // unit tests — panics are intentional
#![allow(non_snake_case)] // BC-traceability IDs use uppercase
#![allow(clippy::doc_markdown)] // test module doc has unbackticked slide-type names

use std::sync::Arc;

use slideforge_eval::thread_fields_to_blocks;
use slideforge_types::{
    Block, ContentBlock, Deck, DeckMetadata, FieldValue, OrderedMap, Slide, SourceSpan, TextTag,
    Value,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_metadata() -> DeckMetadata {
    DeckMetadata {
        title: Some(Arc::from("Test")),
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

fn str_field(s: &str) -> FieldValue {
    FieldValue::Literal(Value::Str(Arc::from(s)))
}

fn int_field(n: i64) -> FieldValue {
    FieldValue::Literal(Value::Int(n))
}

/// Count blocks with `ContentBlock::Text(TextBlock { tag: T, .. })` and matching text.
fn color_label_blocks_with_text<'a>(blocks: &'a [Block], text: &str) -> Vec<&'a Block> {
    blocks
        .iter()
        .filter(|b| {
            if let ContentBlock::Text(tb) = &b.content {
                tb.tag == TextTag::ColorLabel && {
                    let joined: String = tb
                        .inlines
                        .iter()
                        .filter_map(|n| {
                            if let slideforge_types::InlineNode::Plain(s) = n {
                                Some(s.as_ref())
                            } else {
                                None
                            }
                        })
                        .collect();
                    joined.contains(text)
                }
            } else {
                false
            }
        })
        .collect()
}

fn color_bar_blocks(blocks: &[Block]) -> Vec<&Block> {
    blocks
        .iter()
        .filter(|b| matches!(b.content, ContentBlock::ColorBar(_)))
        .collect()
}

fn body_blocks_with_text<'a>(blocks: &'a [Block], text: &str) -> Vec<&'a Block> {
    blocks
        .iter()
        .filter(|b| {
            if let ContentBlock::Text(tb) = &b.content {
                tb.tag == TextTag::Body && {
                    let joined: String = tb
                        .inlines
                        .iter()
                        .filter_map(|n| {
                            if let slideforge_types::InlineNode::Plain(s) = n {
                                Some(s.as_ref())
                            } else {
                                None
                            }
                        })
                        .collect();
                    joined.contains(text)
                }
            } else {
                false
            }
        })
        .collect()
}

fn body_blocks(blocks: &[Block]) -> Vec<&Block> {
    blocks
        .iter()
        .filter(|b| {
            if let ContentBlock::Text(tb) = &b.content {
                tb.tag == TextTag::Body
            } else {
                false
            }
        })
        .collect()
}

// ── §10.1 Threading tests — status ───────────────────────────────────────────

/// BC-1.17.001 PC-8 / adjudication §10.1:
/// `status` slide with `label "On Track"` → `ContentBlock::Text(TextTag::ColorLabel)`
/// with text "On Track" in `slide.blocks`.
///
/// RED GATE: Stage 2b does NOT thread `label` for `status` yet.
#[test]
fn test_status_label_threads_as_color_label() {
    let mut slide = make_slide("status");
    slide
        .fields
        .insert(Arc::from("label"), str_field("On Track"));
    let mut deck = make_deck(vec![slide]);

    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;
    let found = color_label_blocks_with_text(blocks, "On Track");
    assert_eq!(
        found.len(),
        1,
        "RED GATE: `status` with `label 'On Track'` must produce exactly 1 \
         ContentBlock::Text(TextTag::ColorLabel) with text 'On Track'. \
         Current Stage 2b does not thread `label` for status. \
         Implement thread_fields_to_blocks extension §4.2 (STORY-087). \
         Got blocks: {blocks:?}"
    );
}

/// BC-1.17.001 PC-8 / adjudication §10.1:
/// `status` slide WITHOUT `label` field → NO `ColorLabel` block emitted.
///
/// RED GATE: This test PASSES vacuously today (no label → no ColorLabel).
/// It is here to pin the no-label-no-block invariant.
#[test]
fn test_status_no_label_no_block() {
    let slide = make_slide("status");
    // No label field
    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);
    let blocks = &deck.slides[0].blocks;
    let found = color_label_blocks_with_text(blocks, "");
    assert_eq!(
        found.len(),
        0,
        "status without `label` must produce 0 ColorLabel blocks; got {found:?}"
    );
}

// ── §10.1 Threading tests — progress_bar ─────────────────────────────────────

/// BC-1.17.002 PC-9 / adjudication §10.1:
/// `progress_bar` with `label "75% done"` → `ContentBlock::Text(TextTag::ColorLabel)`.
///
/// RED GATE: Stage 2b does NOT thread `label` for `progress_bar` yet.
#[test]
fn test_progress_bar_label_threads_as_color_label() {
    let mut slide = make_slide("progress_bar");
    slide
        .fields
        .insert(Arc::from("label"), str_field("75% done"));
    slide.fields.insert(Arc::from("value"), int_field(75));
    let mut deck = make_deck(vec![slide]);

    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;
    let found = color_label_blocks_with_text(blocks, "75% done");
    assert_eq!(
        found.len(),
        1,
        "RED GATE: `progress_bar` with `label '75% done'` must produce 1 \
         ContentBlock::Text(TextTag::ColorLabel). Stage 2b does not thread \
         `label` for progress_bar yet. Got blocks: {blocks:?}"
    );
}

/// BC-1.17.002 PC-9 / adjudication §10.1:
/// `progress_bar` with `value 75` → `ContentBlock::ColorBar(ColorBarSpec { percent: 75 })`.
///
/// RED GATE: Stage 2b does NOT thread `value` as ColorBar yet.
#[test]
fn test_progress_bar_value_threads_as_color_bar() {
    let mut slide = make_slide("progress_bar");
    slide.fields.insert(Arc::from("value"), int_field(75));
    let mut deck = make_deck(vec![slide]);

    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;
    let bar_blocks = color_bar_blocks(blocks);
    assert_eq!(
        bar_blocks.len(),
        1,
        "RED GATE: `progress_bar` with `value 75` must produce exactly 1 \
         ContentBlock::ColorBar. Stage 2b does not thread `value` as ColorBar yet. \
         Got blocks: {blocks:?}"
    );
    if let ContentBlock::ColorBar(spec) = &bar_blocks[0].content {
        assert_eq!(
            spec.percent, 75,
            "ColorBarSpec.percent must be 75; got {}",
            spec.percent
        );
    }
}

/// BC-1.17.002 PC-9 boundary / adjudication §10.1:
/// `progress_bar` with `value 0` → `ContentBlock::ColorBar(ColorBarSpec { percent: 0 })`.
///
/// RED GATE: Stage 2b does NOT thread `value` as ColorBar yet.
#[test]
fn test_progress_bar_value_zero_threads_as_color_bar() {
    let mut slide = make_slide("progress_bar");
    slide.fields.insert(Arc::from("value"), int_field(0));
    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);
    let blocks = &deck.slides[0].blocks;
    let bar_blocks = color_bar_blocks(blocks);
    assert_eq!(
        bar_blocks.len(),
        1,
        "RED GATE: progress_bar value=0 must produce 1 ColorBar block; got {blocks:?}"
    );
    if let ContentBlock::ColorBar(spec) = &bar_blocks[0].content {
        assert_eq!(
            spec.percent, 0,
            "ColorBarSpec.percent must be 0; got {}",
            spec.percent
        );
    }
}

/// BC-1.17.002 PC-9 boundary / adjudication §10.1:
/// `progress_bar` with `value 100` → `ContentBlock::ColorBar(ColorBarSpec { percent: 100 })`.
///
/// RED GATE: Stage 2b does NOT thread `value` as ColorBar yet.
#[test]
fn test_progress_bar_value_100_threads_as_color_bar() {
    let mut slide = make_slide("progress_bar");
    slide.fields.insert(Arc::from("value"), int_field(100));
    let mut deck = make_deck(vec![slide]);
    thread_fields_to_blocks(&mut deck);
    let blocks = &deck.slides[0].blocks;
    let bar_blocks = color_bar_blocks(blocks);
    assert_eq!(
        bar_blocks.len(),
        1,
        "RED GATE: progress_bar value=100 must produce 1 ColorBar block; got {blocks:?}"
    );
    if let ContentBlock::ColorBar(spec) = &bar_blocks[0].content {
        assert_eq!(
            spec.percent, 100,
            "ColorBarSpec.percent must be 100; got {}",
            spec.percent
        );
    }
}

// ── §10.1 Threading tests — weighted_composite ───────────────────────────────

/// BC-1.17.003 PC-9 / adjudication §10.1:
/// `weighted_composite` with `label "Overall: Good"` → `ContentBlock::Text(TextTag::ColorLabel)`.
///
/// RED GATE: Stage 2b does NOT thread `label` for `weighted_composite` yet.
#[test]
fn test_weighted_composite_label_threads_as_color_label() {
    let mut slide = make_slide("weighted_composite");
    slide
        .fields
        .insert(Arc::from("label"), str_field("Overall: Good"));
    // No components (empty list for simplicity — threading is independent of component count)
    slide.fields.insert(
        Arc::from("components"),
        FieldValue::Literal(Value::List(vec![])),
    );
    let mut deck = make_deck(vec![slide]);

    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;
    let found = color_label_blocks_with_text(blocks, "Overall: Good");
    assert_eq!(
        found.len(),
        1,
        "RED GATE: `weighted_composite` with `label 'Overall: Good'` must produce 1 \
         ContentBlock::Text(TextTag::ColorLabel). Stage 2b does not thread `label` \
         for weighted_composite yet. Got blocks: {blocks:?}"
    );
}

/// BC-1.17.003 PC-9 / adjudication §10.1:
/// `weighted_composite` with 2 components → exactly 2 `ContentBlock::Text(TextTag::Body)` blocks.
///
/// RED GATE: Stage 2b does NOT thread `components` yet.
#[test]
fn test_weighted_composite_2_components_produce_2_body_blocks() {
    let mut comp1: OrderedMap<Arc<str>, Value> = OrderedMap::new();
    comp1.insert(Arc::from("name"), Value::Str(Arc::from("Quality")));
    comp1.insert(
        Arc::from("weight"),
        Value::Float(ordered_float::OrderedFloat(0.4)),
    );
    comp1.insert(Arc::from("score"), Value::Int(85));
    comp1.insert(Arc::from("label"), Value::Str(Arc::from("Excellent")));

    let mut comp2: OrderedMap<Arc<str>, Value> = OrderedMap::new();
    comp2.insert(Arc::from("name"), Value::Str(Arc::from("Price")));
    comp2.insert(
        Arc::from("weight"),
        Value::Float(ordered_float::OrderedFloat(0.6)),
    );
    comp2.insert(Arc::from("score"), Value::Int(72));
    comp2.insert(Arc::from("label"), Value::Str(Arc::from("Good")));

    let mut slide = make_slide("weighted_composite");
    slide
        .fields
        .insert(Arc::from("label"), str_field("Overall: Good"));
    slide.fields.insert(
        Arc::from("components"),
        FieldValue::Literal(Value::List(vec![Value::Map(comp1), Value::Map(comp2)])),
    );
    let mut deck = make_deck(vec![slide]);

    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;
    // Count ONLY TextTag::Body blocks (not ColorLabel or other tags)
    let body_count = body_blocks(blocks).len();
    assert_eq!(
        body_count, 2,
        "RED GATE: `weighted_composite` with 2 components must produce exactly 2 \
         ContentBlock::Text(TextTag::Body) blocks (one per component). \
         Stage 2b does not thread `components` yet. \
         Got {body_count} Body blocks; all blocks: {blocks:?}"
    );
}

/// BC-1.17.003 PC-9 / adjudication §10.1:
/// Component text contains name, score, and label from `compose_component_row_text`.
///
/// RED GATE: Stage 2b does NOT thread `components` yet.
#[test]
fn test_weighted_composite_component_text_contains_name_score_label() {
    let mut comp: OrderedMap<Arc<str>, Value> = OrderedMap::new();
    comp.insert(Arc::from("name"), Value::Str(Arc::from("Quality")));
    comp.insert(
        Arc::from("weight"),
        Value::Float(ordered_float::OrderedFloat(0.4)),
    );
    comp.insert(Arc::from("score"), Value::Int(85));
    comp.insert(Arc::from("label"), Value::Str(Arc::from("Excellent")));

    let mut slide = make_slide("weighted_composite");
    slide.fields.insert(Arc::from("label"), str_field("Agg"));
    slide.fields.insert(
        Arc::from("components"),
        FieldValue::Literal(Value::List(vec![Value::Map(comp)])),
    );
    let mut deck = make_deck(vec![slide]);

    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;
    let body = body_blocks(blocks);
    assert_eq!(
        body.len(),
        1,
        "RED GATE: 1 component → 1 Body block; got {body:?}"
    );
    if let ContentBlock::Text(tb) = &body[0].content {
        let text: String = tb
            .inlines
            .iter()
            .filter_map(|n| {
                if let slideforge_types::InlineNode::Plain(s) = n {
                    Some(s.as_ref())
                } else {
                    None
                }
            })
            .collect();
        assert!(
            text.contains("Quality"),
            "Component body text must contain 'Quality'; got: {text}"
        );
        assert!(
            text.contains("85"),
            "Component body text must contain '85' (score); got: {text}"
        );
        assert!(
            text.contains("Excellent"),
            "Component body text must contain 'Excellent' (label); got: {text}"
        );
    }
}

// ── §10.1 Threading tests — stat_callout ─────────────────────────────────────

/// BC-1.17.003-adjacent / adjudication §10.1:
/// `stat_callout` with `stat_1 "$4.2B"` → `ContentBlock::Text(TextTag::Body)` with "$4.2B".
///
/// RED GATE: Stage 2b does NOT thread `stat_1` for `stat_callout` yet.
#[test]
fn test_stat_callout_stat1_threads_as_body() {
    let mut slide = make_slide("stat_callout");
    slide.fields.insert(Arc::from("stat_1"), str_field("$4.2B"));
    let mut deck = make_deck(vec![slide]);

    thread_fields_to_blocks(&mut deck);

    let blocks = &deck.slides[0].blocks;
    let found = body_blocks_with_text(blocks, "$4.2B");
    assert_eq!(
        found.len(),
        1,
        "RED GATE: `stat_callout` with `stat_1 '$4.2B'` must produce 1 \
         ContentBlock::Text(TextTag::Body) with text '$4.2B'. \
         Stage 2b does not thread stat_callout fields yet. \
         Got blocks: {blocks:?}"
    );
}
