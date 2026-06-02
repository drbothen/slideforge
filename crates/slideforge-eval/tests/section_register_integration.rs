//! Integration test: STORY-077 — SectionBlock IR Extension end-to-end pipeline.
//!
//! Exercises the full `eval_deck()` pipeline with decks containing both slides
//! (with register fields from STORY-035) AND section blocks (with `detail:` and
//! `report:` sub-blocks from STORY-077). Asserts correct `register_content` on
//! both slide nodes and section nodes.
//!
//! These tests MUST FAIL before implementation (Red Gate): `eval_section_nodes`
//! is a `todo!()` stub so any test that exercises the section pipeline panics.
//!
//! TD-VSDD-059: all assertions are non-vacuous (specific field keys, register
//! variants, inline node content, absence checks for cross-contamination).

#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::sync::Arc;

use slideforge_eval::{EvalConfig, eval_deck};
use slideforge_syntax::{
    BlockItem, DeckNode, DiagnosticSink, FieldNode, FieldValue, SectionNode, SlideNode, Spanned,
    TemplateChunk,
};
use slideforge_syntax::span::Span;
use slideforge_types::{InlineNode, Register};

fn dummy_span() -> Span {
    Span::new(0, 0, 0)
}

fn default_config() -> EvalConfig {
    EvalConfig::default()
}

// ─── Integration: eval_deck with section nodes ─────────────────────────────────

/// BC-3.02.002 postcondition 7 / AC-003 / AC-006:
/// Full `eval_deck()` on a deck with a slide (notes/report/detail fields) AND
/// a `section methodology:` block (with `detail:` sub-block) populates:
/// - `deck.slides[0].register_content` with the slide's register entries
/// - `deck.section_blocks[0].register_content` with the section's detail entry
///
/// Also asserts no cross-contamination: section detail does NOT appear in
/// any LaidOutSlide.register_content.
#[test]
fn test_BC_3_02_002_integration_eval_deck_populates_slide_and_section_register_content() {
    // Build a slide with a notes field.
    let notes_field = FieldNode {
        name: Spanned::new("notes".to_string(), dummy_span()),
        value: Spanned::new(
            FieldValue::Template(vec![TemplateChunk::Literal(
                "Slide presenter notes".to_string(),
            )]),
            dummy_span(),
        ),
    };
    let slide_item = BlockItem::Slide(Spanned::new(
        SlideNode {
            kind: Spanned::new("content".to_string(), dummy_span()),
            tags: vec![],
            fields: vec![notes_field],
            inline_items: vec![],
        },
        dummy_span(),
    ));

    // Build a section block with a detail: sub-block.
    let detail_field = FieldNode {
        name: Spanned::new("detail".to_string(), dummy_span()),
        value: Spanned::new(
            FieldValue::Template(vec![TemplateChunk::Literal(
                "Methodology detail content".to_string(),
            )]),
            dummy_span(),
        ),
    };
    let section_item = BlockItem::Section(Spanned::new(
        SectionNode {
            kind: Spanned::new("methodology".to_string(), dummy_span()),
            fields: vec![detail_field],
        },
        dummy_span(),
    ));

    let deck_node = DeckNode {
        items: vec![slide_item, section_item],
        ..DeckNode::default()
    };

    let config = default_config();
    let mut sink = DiagnosticSink::new();

    // Call the production eval_deck — this exercises the full pipeline.
    // eval_section_nodes (stub) will panic here: RED GATE.
    let deck = eval_deck(&deck_node, &config, &mut sink)
        .expect("eval_deck must return Some for valid deck");

    assert!(
        sink.is_empty(),
        "no diagnostics expected for valid deck; got: {:?}",
        sink.errors()
    );

    // Slide register_content: must have 1 Notes entry.
    assert_eq!(deck.slides.len(), 1, "must have 1 slide");
    assert_eq!(
        deck.slides[0].register_content.len(),
        1,
        "slide must have 1 register_content entry (notes)"
    );
    assert_eq!(
        deck.slides[0].register_content[0].register,
        Register::Notes,
        "slide register_content must be tagged Notes"
    );

    // Section register_content: must have 1 Detail entry.
    assert_eq!(deck.section_blocks.len(), 1, "must have 1 section block");
    assert_eq!(
        deck.section_blocks[0].register_content.len(),
        1,
        "section must have 1 register_content entry (detail)"
    );
    assert_eq!(
        deck.section_blocks[0].register_content[0].register,
        Register::Detail,
        "section register_content must be tagged Detail"
    );

    // Cross-contamination check: section detail content must NOT appear in
    // slide register_content (it is NOT attached to any LaidOutSlide).
    let slide_has_detail = deck.slides[0]
        .register_content
        .iter()
        .any(|rc| rc.register == Register::Detail);
    assert!(
        !slide_has_detail,
        "section detail content must NOT appear in slide.register_content; \
         it must only appear on the section node"
    );

    // Content correctness.
    let section_detail_text: String = deck.section_blocks[0].register_content[0]
        .content
        .iter()
        .map(|n| {
            if let InlineNode::Plain(s) = n {
                s.as_ref().to_owned()
            } else {
                String::new()
            }
        })
        .collect();
    assert_eq!(
        section_detail_text, "Methodology detail content",
        "section detail content must match the sub-block text"
    );
}

/// BC-3.02.002 invariant 3: `eval_deck` with an unknown section type produces
/// a fatal error and returns None.
#[test]
fn test_BC_3_02_002_integration_unknown_section_type_fatal() {
    let section_item = BlockItem::Section(Spanned::new(
        SectionNode {
            kind: Spanned::new("foobar".to_string(), dummy_span()),
            fields: vec![],
        },
        dummy_span(),
    ));

    let deck_node = DeckNode {
        items: vec![section_item],
        ..DeckNode::default()
    };

    let config = default_config();
    let mut sink = DiagnosticSink::new();

    // eval_deck (stub panics here) — RED GATE.
    // When implemented: must return None (fatal) for unknown section type.
    let deck = eval_deck(&deck_node, &config, &mut sink);

    assert!(
        deck.is_none(),
        "eval_deck must return None for unknown section type 'foobar'"
    );

    let err_msgs: Vec<String> = sink.errors().iter().map(|e| e.to_string()).collect();
    let combined = err_msgs.join("; ");
    assert!(
        combined.contains("foobar"),
        "error must name the unknown type 'foobar'; got: {}",
        combined
    );
}
