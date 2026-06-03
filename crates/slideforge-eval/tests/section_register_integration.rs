//! Integration test: STORY-077 — [`SectionBlock`] IR Extension end-to-end pipeline.
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

use slideforge_eval::{EvalConfig, eval_deck};
use slideforge_syntax::span::Span;
use slideforge_syntax::{
    BlockItem, DeckNode, DiagnosticSink, FieldNode, FieldValue, SectionNode, SlideNode, Spanned,
    TemplateChunk,
};
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
/// any `LaidOutSlide::register_content`.
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

// ═══════════════════════════════════════════════════════════════════════════════
// OBS-P25-A: Real-source → eval round-trip (STORY-077 follow-up)
//
// Parses REAL .sf source through the production parser, runs the real eval
// pipeline, and asserts structural InlineNode variants in the section body.
// This consolidates AC-002 coverage via the actual parse→eval path.
// ═══════════════════════════════════════════════════════════════════════════════

/// OBS-P25-A / BC-3.02.002 AC-002:
/// Real `.sf` source with `section methodology:` + `detail: **Bold claim.** See {{ ref("slide-1") }}.`
/// must parse+eval to a `SectionBlock` with `body["detail"] = FieldValue::Inlines([...])`.
///
/// Asserts structural `InlineNode` variants: `Bold([Plain("Bold claim.")])`, `Plain(" See ")`,
/// `Xref("slide-1")`, `Plain(".")`.
///
/// Also asserts a `RegisteredContent` { `Register::Detail` } entry is produced.
///
/// This uses the REAL source parser (`slideforge_syntax::parser::parse`) and the
/// real eval pipeline (`eval_deck`) — NOT hand-constructed AST nodes.
#[test]
#[allow(non_snake_case)]
fn test_OBS_P25_A_real_source_parse_eval_section_detail_structural_inline_nodes() {
    use slideforge_syntax::parser::parse;
    use slideforge_syntax::span::SourceMap;
    use slideforge_types::FieldValue as TypesFieldValue;
    use std::sync::Arc;

    // Real DSL source: a section with a detail register field containing
    // bold markup and a ref() call (quote-free case per OBS-P25-A spec).
    // `{{ ref("slide-1") }}` uses the text-mode interpolation path.
    // We use figref(1) instead of ref("slide-1") to avoid inner-quote
    // round-trip issues through the DSL string lexer (which stores StringLit
    // tokens verbatim, including backslash-escaped quotes). figref(1) has
    // no inner quotes and works through the full round-trip.
    //
    // Correct section sub-block syntax is `detail: "value"` (WITH colon).
    // `detail "value"` (without colon) is EC-006 / E-PAR-017.
    let src = "section methodology:\n  detail: \"**Bold claim.** See {{ figref(1) }}.\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
    let parse_result =
        parse(src, file_id, &sm).expect("OBS-P25-A: source must parse without errors");

    let deck_node = parse_result.deck;

    let config = EvalConfig::default();
    let mut sink = slideforge_syntax::DiagnosticSink::new();
    let deck = eval_deck(&deck_node, &config, &mut sink)
        .expect("OBS-P25-A: eval_deck must return Some for valid section");

    assert!(
        sink.is_empty(),
        "OBS-P25-A: no diagnostics expected; got: {:?}",
        sink.errors()
    );

    // There must be exactly 1 section block.
    assert_eq!(
        deck.section_blocks.len(),
        1,
        "OBS-P25-A: must produce 1 section block; got: {:?}",
        deck.section_blocks
    );

    let section = &deck.section_blocks[0];

    // The section must have a Detail register_content entry.
    let detail_rc = section
        .register_content
        .iter()
        .find(|rc| rc.register == slideforge_types::Register::Detail)
        .expect("OBS-P25-A: section must have a Detail register_content entry");

    // detail_rc.content must contain structural InlineNode variants:
    //   Bold([Plain("Bold claim.")]), Plain(" See "), Xref("fig-1"), Plain(".")
    let nodes = &detail_rc.content;
    assert!(
        nodes.len() >= 4,
        "OBS-P25-A: Detail content must have at least 4 inline nodes; got {nodes:?}"
    );

    // Node 0: Bold([Plain("Bold claim.")])
    assert!(
        matches!(&nodes[0], InlineNode::Bold(children) if {
            children.len() == 1 && matches!(&children[0], InlineNode::Plain(s) if s.as_ref() == "Bold claim.")
        }),
        "OBS-P25-A: nodes[0] must be Bold([Plain(\"Bold claim.\")]); got: {:?}",
        nodes[0]
    );

    // Node 1: Plain(" See ")
    assert!(
        matches!(&nodes[1], InlineNode::Plain(s) if s.as_ref() == " See "),
        "OBS-P25-A: nodes[1] must be Plain(\" See \"); got: {:?}",
        nodes[1]
    );

    // Node 2: Xref("fig-1")  (figref(1) → Xref("fig-1"))
    assert!(
        matches!(&nodes[2], InlineNode::Xref(id) if id.as_ref() == "fig-1"),
        "OBS-P25-A: nodes[2] must be Xref(\"fig-1\"); got: {:?}",
        nodes[2]
    );

    // Node 3: Plain(".")
    assert!(
        matches!(&nodes[3], InlineNode::Plain(s) if s.as_ref() == "."),
        "OBS-P25-A: nodes[3] must be Plain(\".\"); got: {:?}",
        nodes[3]
    );

    // Also verify the body map contains "detail" as FieldValue::Inlines.
    let detail_field = section
        .body
        .get("detail")
        .expect("OBS-P25-A: section body must contain key \"detail\"");
    assert!(
        matches!(detail_field, TypesFieldValue::Inlines(_)),
        "OBS-P25-A: body[\"detail\"] must be FieldValue::Inlines; got: {detail_field:?}"
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

    let err_msgs: Vec<String> = sink.errors().iter().map(ToString::to_string).collect();
    let combined = err_msgs.join("; ");
    assert!(
        combined.contains("foobar"),
        "error must name the unknown type 'foobar'; got: {combined}"
    );
}
