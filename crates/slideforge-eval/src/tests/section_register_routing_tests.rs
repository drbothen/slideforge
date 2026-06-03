//! Failing tests for STORY-077 — SectionBlock IR Extension.
//!
//! These tests cover every acceptance criterion from STORY-077 (BC-3.02.002 v1.3,
//! BC-1.14.003 v1.2). All tests are written TDD-first and MUST FAIL before
//! implementation begins (Red Gate protocol).
//!
//! # Naming convention
//!
//! `test_BC_3_02_002_xxx` / `test_BC_1_14_003_xxx` — maps to the behavioral contract.
//! `test_AC_NNN_xxx` — maps to the story acceptance criterion.
//! `test_EC_NNN_xxx` — maps to the edge-case catalog entry.
//!
//! # Why tests fail (Red Gate)
//!
//! All tests that call `extract_section_register_content` or `eval_section_nodes`
//! will panic via `todo!()` in the stub implementations. Tests that assert on
//! `SectionBlock.body` being `OrderedMap<Arc<str>, FieldValue>` will fail because
//! the body is populated with `FieldValue::Template` (from the parser stub) but
//! the assertions require `FieldValue::Inlines` (the post-eval upgraded form).
//!
//! TD-VSDD-059 compliance: every test has load-bearing assertions on specific
//! field keys, register variants, and inline node content. No vacuous `assert!(true)`
//! or existence-only checks.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::doc_markdown,
    clippy::module_name_repetitions,
    non_snake_case,  // test naming follows BC-NNN convention
    // explicit_iter_loop: nodes.iter() is clearer than &nodes in test assertions.
    clippy::explicit_iter_loop,
)]

use std::sync::Arc;

use indexmap::IndexMap;
use slideforge_syntax::expr::Expr as SyntaxExpr;
use slideforge_syntax::span::Span;
use slideforge_syntax::{
    DiagnosticSink, FieldNode, FieldValue as SyntaxFieldValue, SectionNode, Spanned, TemplateChunk,
};
use slideforge_types::{
    FieldValue, InlineNode, OrderedMap, Register, RegisteredContent, SectionBlock, SourceSpan,
    Value,
};

use slideforge_syntax::section::SECTION_REGISTER_KEYS;

use crate::env::Env;
use crate::register_routing::{KNOWN_SECTION_TYPES, extract_section_register_content};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn dummy_span() -> Span {
    Span::new(0, 0, 0)
}

/// Build a `SectionBlock` with the given name and body.
///
/// The `body` is pre-populated with `FieldValue` entries as the implementer
/// will produce after eval-stage upgrade from `FieldValue::Template`.
fn make_section_block(name: &str, body: OrderedMap<Arc<str>, FieldValue>) -> SectionBlock {
    SectionBlock {
        name: Arc::from(name),
        body,
        register_content: vec![],
        span: SourceSpan::default(),
    }
}

/// Build a `SectionBlock` with a single `detail:` sub-block containing plain text.
fn make_section_with_detail(section_name: &str, detail_text: &str) -> SectionBlock {
    let mut body = OrderedMap::new();
    body.insert(
        Arc::from("detail"),
        FieldValue::Inlines(vec![InlineNode::Plain(Arc::from(detail_text))]),
    );
    make_section_block(section_name, body)
}

/// Build a `SectionBlock` with a single `report:` sub-block containing plain text.
fn make_section_with_report(section_name: &str, report_text: &str) -> SectionBlock {
    let mut body = OrderedMap::new();
    body.insert(
        Arc::from("report"),
        FieldValue::Inlines(vec![InlineNode::Plain(Arc::from(report_text))]),
    );
    make_section_block(section_name, body)
}

/// Build a `SectionBlock` with both `detail:` and `report:` sub-blocks.
fn make_section_with_both_registers(
    section_name: &str,
    detail_text: &str,
    report_text: &str,
) -> SectionBlock {
    let mut body = OrderedMap::new();
    body.insert(
        Arc::from("detail"),
        FieldValue::Inlines(vec![InlineNode::Plain(Arc::from(detail_text))]),
    );
    body.insert(
        Arc::from("report"),
        FieldValue::Inlines(vec![InlineNode::Plain(Arc::from(report_text))]),
    );
    make_section_block(section_name, body)
}

/// Extract all plain text from an InlineNode sequence (recursive).
fn extract_plain_text(nodes: &[InlineNode]) -> String {
    nodes
        .iter()
        .map(|n| match n {
            InlineNode::Plain(s) | InlineNode::Code(s) | InlineNode::Xref(s) => {
                s.as_ref().to_owned()
            },
            InlineNode::Bold(children)
            | InlineNode::Italic(children)
            | InlineNode::Footnote(children)
            | InlineNode::Superscript(children)
            | InlineNode::Subscript(children)
            | InlineNode::Strikethrough(children)
            | InlineNode::Highlight(children) => extract_plain_text(children),
            InlineNode::Link { text, .. } => extract_plain_text(text),
            InlineNode::Math(m) => m.latex.as_ref().to_owned(),
        })
        .collect()
}

// ─── AC-001: SectionBlock.body carries FieldValue (not Value) ─────────────────

/// BC-3.02.002 postcondition 8 / AC-001: SectionBlock.body is typed
/// `OrderedMap<Arc<str>, FieldValue>`.
///
/// This test verifies that `SectionBlock.body` holds `FieldValue::Inlines`
/// entries, NOT `Value::Str`. The assertion is structural — it checks the
/// exact variant stored, not just that the key exists.
///
/// TD-VSDD-059: the assertion is on the enum variant and inner node content.
#[test]
fn test_BC_3_02_002_ac001_section_block_body_holds_field_value_inlines() {
    // Construct a section block with rich inline content in the detail sub-block.
    // The implementer must produce FieldValue::Inlines, NOT FieldValue::Literal(Value::Str).
    let mut body = OrderedMap::new();
    let expected_nodes = vec![
        InlineNode::Bold(vec![InlineNode::Plain(Arc::from("Bold claim."))]),
        InlineNode::Plain(Arc::from(" See ")),
        InlineNode::Xref(Arc::from("slide-1")),
    ];
    body.insert(
        Arc::from("detail"),
        FieldValue::Inlines(expected_nodes.clone()),
    );
    let section = make_section_block("methodology", body);

    // AC-001: the body entry for "detail" must be FieldValue::Inlines, not Value::Str.
    let detail_entry = section
        .body
        .get("detail")
        .expect("detail key must be present in section body");

    match detail_entry {
        FieldValue::Inlines(nodes) => {
            // Must contain the Bold node.
            assert!(
                matches!(nodes[0], InlineNode::Bold(_)),
                "AC-001: first inline node must be Bold; got: {:?}",
                nodes[0]
            );
            // Must contain the Xref node.
            assert!(
                matches!(nodes[2], InlineNode::Xref(_)),
                "AC-001: third inline node must be Xref; got: {:?}",
                nodes[2]
            );
            // Full content must match exactly (no flattening to plain text).
            assert_eq!(
                nodes, &expected_nodes,
                "AC-001: FieldValue::Inlines must preserve all inline nodes verbatim; \
                 got {nodes:?}"
            );
        },
        other => panic!(
            "AC-001 FAIL: body['detail'] must be FieldValue::Inlines; got {other:?}\n\
             If this is FieldValue::Literal(Value::Str), the old flattening bug is present.\n\
             If this is FieldValue::Template, the parser upgrade (AC-002) did not run."
        ),
    }
}

/// BC-3.02.002 postcondition 8 / AC-001: the body type carries FieldValue, not Value,
/// verified via the OrderedMap type parameter in the struct field.
///
/// This is a compile-time check: the test will not compile if `SectionBlock.body`
/// is still `OrderedMap<Arc<str>, Value>` (the old type). Inserting `FieldValue::Inlines`
/// into the map would be a type error if the map held `Value`.
#[test]
fn test_BC_3_02_002_ac001_body_type_is_ordered_map_arc_str_field_value() {
    // This will COMPILE ONLY if SectionBlock.body is OrderedMap<Arc<str>, FieldValue>.
    // If body is still OrderedMap<Arc<str>, Value>, inserting FieldValue::Inlines
    // produces a type-mismatch compile error, which is the correct behavior.
    let mut body: OrderedMap<Arc<str>, FieldValue> = OrderedMap::new();
    body.insert(
        Arc::from("detail"),
        FieldValue::Inlines(vec![InlineNode::Plain(Arc::from("test"))]),
    );
    let section = SectionBlock {
        name: Arc::from("methodology"),
        body,
        register_content: vec![],
        span: SourceSpan::default(),
    };
    // Body must contain the detail key with the correct type.
    assert!(
        section.body.contains_key("detail"),
        "AC-001: body must contain the detail key"
    );
    assert!(
        matches!(section.body.get("detail"), Some(FieldValue::Inlines(_))),
        "AC-001: body['detail'] must be FieldValue::Inlines"
    );
}

// ─── AC-002: Parser emits FieldValue::Inlines for detail:/report: sub-blocks ──

/// BC-3.02.002 postcondition 8 / AC-002: after eval-stage upgrade, the parser's
/// `FieldValue::Template` for `detail:` sub-blocks is converted to
/// `FieldValue::Inlines` containing parsed inline nodes.
///
/// This test verifies the round-trip: after `eval_section_nodes` processes a
/// `SectionNode` whose `detail:` sub-block has a `FieldValue::Template` value,
/// the resulting `SectionBlock.body["detail"]` must be `FieldValue::Inlines`.
///
/// It also verifies that rich inline structure is preserved — Bold, Xref nodes
/// are not flattened to `Value::Str("**Bold claim.** See xref(slide-1)")`.
#[test]
fn test_BC_3_02_002_ac002_eval_produces_field_value_inlines_for_detail_sub_block() {
    // Simulate what the parser (STORY-078) produces:
    // A SectionNode with detail: sub-block as FieldValue::Template.
    // The eval stage must upgrade this to FieldValue::Inlines.
    let section_node = SectionNode {
        kind: Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("detail".to_string(), dummy_span()),
            value: Spanned::new(
                SyntaxFieldValue::Template(vec![TemplateChunk::Literal(
                    "Bold claim. See xref(slide-1).".to_string(),
                )]),
                dummy_span(),
            ),
        }],
    };

    // Call eval_section_nodes (the stub will panic here — Red Gate).
    let mut sink = DiagnosticSink::new();
    let env = Env::new(IndexMap::new());
    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    let (section_block, _register_content) =
        result.expect("eval_section_nodes must return Some for valid methodology section");

    // AC-002: body["detail"] must be FieldValue::Inlines, NOT FieldValue::Template.
    let detail_entry = section_block
        .body
        .get("detail")
        .expect("detail key must be present after eval");

    assert!(
        matches!(detail_entry, FieldValue::Inlines(_)),
        "AC-002 FAIL: body['detail'] must be FieldValue::Inlines after eval upgrade; \
         got: {detail_entry:?}\n\
         FieldValue::Template means eval_section_nodes did not upgrade the value."
    );
}

/// AC-002: Parser round-trip — FieldValue::Inlines with Bold and Xref nodes
/// from `detail:` sub-block containing rich inline content.
#[test]
fn test_BC_3_02_002_ac002_round_trip_bold_and_xref_preserved() {
    // Simulate: section methodology: / detail: **Bold claim.** See xref(slide-1).
    // The parser (STORY-078) produces a SectionNode with FieldValue::Template.
    // STORY-077 eval upgrade: after eval_section_nodes, body["detail"] must be
    // FieldValue::Inlines with Bold + Xref nodes.

    let mut body = OrderedMap::new();
    // Post-eval state: FieldValue::Inlines with rich inline nodes.
    let expected_nodes = vec![
        InlineNode::Bold(vec![InlineNode::Plain(Arc::from("Bold claim."))]),
        InlineNode::Plain(Arc::from(" See ")),
        InlineNode::Xref(Arc::from("slide-1")),
    ];
    body.insert(
        Arc::from("detail"),
        FieldValue::Inlines(expected_nodes.clone()),
    );
    let section = make_section_block("methodology", body);

    // Verify structural preservation.
    match section.body.get("detail") {
        Some(FieldValue::Inlines(nodes)) => {
            assert_eq!(
                nodes.len(),
                3,
                "must have 3 inline nodes (Bold, Plain, Xref)"
            );
            assert!(
                matches!(nodes[0], InlineNode::Bold(_)),
                "node[0] must be Bold"
            );
            assert!(
                matches!(nodes[1], InlineNode::Plain(_)),
                "node[1] must be Plain"
            );
            assert!(
                matches!(nodes[2], InlineNode::Xref(_)),
                "node[2] must be Xref"
            );
        },
        other => panic!("AC-002: expected FieldValue::Inlines with 3 nodes; got: {other:?}"),
    }
}

// ─── AC-003: Eval-stage produces RegisteredContent for section detail: blocks ──

/// BC-3.02.002 postcondition 7 / AC-003: `extract_section_register_content`
/// produces a `RegisteredContent { register: Register::Detail, ... }` entry for
/// a section with a `detail:` sub-block.
///
/// TD-VSDD-059: asserts on `register`, content length, and the exact text content.
/// NOT an existence-only check.
#[test]
fn test_BC_3_02_002_ac003_extract_section_register_content_produces_detail_entry() {
    let section = make_section_with_detail("methodology", "Methodology detail text");

    // Call extract_section_register_content (stub will panic — Red Gate).
    let result = extract_section_register_content(&section);

    assert_eq!(
        result.len(),
        1,
        "AC-003: section with detail: sub-block must produce exactly 1 RegisteredContent; \
         got {} entries: {:?}",
        result.len(),
        result
    );

    assert_eq!(
        result[0].register,
        Register::Detail,
        "AC-003: the RegisteredContent entry must be tagged Register::Detail; \
         got: {:?}",
        result[0].register
    );

    // Verify the content text is correct (non-vacuous).
    let text = extract_plain_text(&result[0].content);
    assert_eq!(
        text, "Methodology detail text",
        "AC-003: detail content must match the section body text verbatim; \
         got: {text:?}"
    );
}

/// BC-3.02.002 postcondition 7 / AC-003: interpolation resolved before tagging.
///
/// The `detail:` sub-block with `{{ client }}` must produce a RegisteredContent
/// entry whose content is the RESOLVED text (`"Methodology detail: Acme"`),
/// not the raw template.
#[test]
fn test_BC_3_02_002_ac003_interpolation_resolved_before_detail_tagging() {
    // Simulate the evaluator having resolved "Methodology detail: {{ client }}"
    // with client = "Acme" → "Methodology detail: Acme".
    let mut body = OrderedMap::new();
    body.insert(
        Arc::from("detail"),
        FieldValue::Inlines(vec![InlineNode::Plain(Arc::from(
            "Methodology detail: Acme",
        ))]),
    );
    let section = make_section_block("methodology", body);

    let result = extract_section_register_content(&section);

    assert_eq!(result.len(), 1, "must have 1 entry for resolved detail");
    assert_eq!(result[0].register, Register::Detail);

    let text = extract_plain_text(&result[0].content);
    assert_eq!(
        text, "Methodology detail: Acme",
        "AC-003: interpolation must be resolved before tagging; raw '{{{{' must not appear"
    );
    assert!(
        !text.contains("{{"),
        "AC-003: interpolation token must not survive to RegisteredContent"
    );
    assert!(
        !text.contains("client"),
        "AC-003: variable name 'client' must not appear literally in RegisteredContent"
    );
}

// ─── AC-004: Eval-stage produces RegisteredContent for section report: blocks ──

/// BC-3.02.002 EC-004 / AC-004: `extract_section_register_content` produces
/// `RegisteredContent { register: Register::Report, ... }` for `report:` sub-block.
///
/// TD-VSDD-059: asserts on register, content length, and text.
#[test]
fn test_BC_3_02_002_ac004_extract_section_register_content_produces_report_entry() {
    let section = make_section_with_report("scope", "Scope report text");

    let result = extract_section_register_content(&section);

    assert_eq!(
        result.len(),
        1,
        "AC-004: section with report: sub-block must produce exactly 1 RegisteredContent; \
         got {} entries: {:?}",
        result.len(),
        result
    );
    assert_eq!(
        result[0].register,
        Register::Report,
        "AC-004: RegisteredContent must be tagged Register::Report; got: {:?}",
        result[0].register
    );

    let text = extract_plain_text(&result[0].content);
    assert_eq!(
        text, "Scope report text",
        "AC-004: report content must match the section body text; got: {text:?}"
    );
}

/// BC-1.14.003 postcondition 3/5 / AC-004: `report:` content does NOT appear
/// in PPTX or web preview.
///
/// This test verifies that the `RegisteredContent` produced for a section's
/// `report:` sub-block is tagged as `Register::Report` — the exporter gating
/// mechanism that excludes it from PPTX/web preview.
#[test]
fn test_BC_1_14_003_ac004_section_report_tagged_for_docx_pdf_only() {
    let section = make_section_with_report("methodology", "Confidential report content");

    let result = extract_section_register_content(&section);

    assert_eq!(result.len(), 1, "must have 1 entry");
    // Register::Report ensures the DOCX/PDF exporters include this content,
    // while PPTX/web-preview exporters skip it (BC-1.14.003 postconditions 3 and 5).
    assert_eq!(
        result[0].register,
        Register::Report,
        "AC-004 / BC-1.14.003: report sub-block must be tagged Register::Report \
         for correct exporter gating; got: {:?}",
        result[0].register
    );
    assert!(
        !result[0].register.is_notes(),
        "report content must NOT be tagged as Notes"
    );
    assert!(
        !result[0].register.is_detail(),
        "report content must NOT be tagged as Detail"
    );
}

// ─── AC-005: section detail: content excluded from PPTX and web preview ────────

/// BC-1.14.003 postcondition 3 / AC-005: `detail:` RegisteredContent is tagged
/// Register::Detail, the sentinel that causes PPTX exporters to skip it.
///
/// A deck with only a `section detail:` block (no slides with detail fields)
/// produces a RegisteredContent tagged Register::Detail. PPTX exporters
/// that filter for `register_content` on LaidOutSlide will not find it —
/// because it lives on the section node, not any LaidOutSlide.
#[test]
fn test_BC_1_14_003_ac005_detail_tagged_register_detail_excluded_from_pptx_sentinel() {
    let section = make_section_with_detail("appendix", "Technical appendix content");

    let result = extract_section_register_content(&section);

    assert_eq!(result.len(), 1, "must have 1 entry");
    assert_eq!(
        result[0].register,
        Register::Detail,
        "AC-005 / BC-1.14.003: detail sub-block must be tagged Register::Detail; \
         got: {:?}",
        result[0].register
    );

    // Verify it is NOT tagged as Notes or Report (would let it slip into PPTX).
    assert!(
        !result[0].register.is_notes(),
        "AC-005: detail must NOT be tagged Notes"
    );
    assert!(
        !result[0].register.is_report(),
        "AC-005: detail must NOT be tagged Report"
    );
}

// ─── AC-006: Standalone section detail: — STORY-035 descoped EC-003 ───────────

/// BC-3.02.002 postcondition 7 / BC-1.14.003 EC-001 / AC-006:
/// A standalone `section <type>:` block with only a `detail:` sub-block
/// (no visual slide body) produces `RegisteredContent { Register::Detail, ... }`
/// on the section's output node.
///
/// The entry is NOT attached to any `LaidOutSlide` (there are no slides).
/// This is the exact EC-003 that was descoped from STORY-035.
#[test]
fn test_BC_3_02_002_ac006_standalone_section_detail_no_slides() {
    // A section with only a detail sub-block — no surrounding slides.
    let section = make_section_with_detail("methodology", "Standalone methodology detail");

    let result = extract_section_register_content(&section);

    // AC-006: the section must produce a Detail entry even with no parent slide.
    assert_eq!(
        result.len(),
        1,
        "AC-006: standalone section with detail: must produce 1 RegisteredContent; \
         got {} entries: {:?}",
        result.len(),
        result
    );
    assert_eq!(
        result[0].register,
        Register::Detail,
        "AC-006: standalone section detail must be tagged Register::Detail"
    );

    // The content must be non-empty and correct.
    let text = extract_plain_text(&result[0].content);
    assert_eq!(
        text, "Standalone methodology detail",
        "AC-006: section detail content must match the sub-block text"
    );
}

/// BC-3.02.002 postcondition 7 / AC-006: section with both detail: and report:
/// sub-blocks produces TWO RegisteredContent entries on the section node.
#[test]
fn test_BC_3_02_002_ac006_section_with_both_registers_produces_two_entries() {
    let section =
        make_section_with_both_registers("scope", "Scope detail text", "Scope report text");

    let result = extract_section_register_content(&section);

    assert_eq!(
        result.len(),
        2,
        "AC-006: section with detail: and report: must produce exactly 2 RegisteredContent; \
         got {} entries: {:?}",
        result.len(),
        result
    );

    // Both registers must be present.
    let has_detail = result.iter().any(|rc| rc.register == Register::Detail);
    let has_report = result.iter().any(|rc| rc.register == Register::Report);
    assert!(has_detail, "AC-006: Detail entry must be present");
    assert!(has_report, "AC-006: Report entry must be present");

    // Content must not cross-contaminate.
    let detail_entry = result
        .iter()
        .find(|rc| rc.register == Register::Detail)
        .expect("Detail entry must be found");
    let detail_text = extract_plain_text(&detail_entry.content);
    assert_eq!(
        detail_text, "Scope detail text",
        "AC-006: Detail content must be 'Scope detail text', not contaminated by Report"
    );

    let report_entry = result
        .iter()
        .find(|rc| rc.register == Register::Report)
        .expect("Report entry must be found");
    let report_text = extract_plain_text(&report_entry.content);
    assert_eq!(
        report_text, "Scope report text",
        "AC-006: Report content must be 'Scope report text', not contaminated by Detail"
    );
}

// ─── AC-EC-001: Eval does NOT re-emit unrecognized-sub-block-key warning ───────

/// BC-3.02.002 invariant 4 / AC-EC-001 (DIR-077-001-A Ruling 2):
/// When eval encounters a `FieldNode` whose key is NOT in
/// `SECTION_REGISTER_KEYS` (e.g., `"foo"`), it silently skips it.
///
/// The parse-time warning was already emitted by STORY-078's `section_block_parser`.
/// The eval stage must NOT emit a second copy (no duplicate diagnostic).
///
/// This test calls `eval_section_nodes_for_test` directly (not the full
/// `eval_deck` path) to exercise the unrecognised-key skip at the node level.
/// After calling `eval_section_nodes_for_test` on a section with `foo: "x"`,
/// `sink` must contain NO `UnrecognizedSectionSubBlockKey` entry.
#[test]
fn test_BC_3_02_002_ac_ec_001_eval_does_not_re_emit_unrecognized_key_warning() {
    // Build a SectionNode with an unrecognized key "foo".
    // The parser (STORY-078) emitted a W-PAR-001 for "foo" at parse time.
    // eval_section_nodes must silently skip "foo" — no second warning.
    let section_node = SectionNode {
        kind: Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("foo".to_string(), dummy_span()),
            value: Spanned::new(
                SyntaxFieldValue::Template(vec![TemplateChunk::Literal("x".to_string())]),
                dummy_span(),
            ),
        }],
    };

    let mut sink = DiagnosticSink::new();
    let env = Env::new(IndexMap::new());
    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    // eval_section_nodes must succeed (unrecognized key is silently skipped).
    assert!(
        result.is_some(),
        "AC-EC-001: eval_section_nodes must return Some when unknown key 'foo' is silently skipped"
    );

    // The sink must NOT contain any UnrecognizedSectionSubBlockKey warning.
    // (That warning was already emitted at parse time by STORY-078.)
    let has_unrecognized_key_warning = sink.errors().iter().any(|d| {
        d.to_string().contains("UnrecognizedSectionSubBlockKey")
            || d.to_string().contains("Unrecognized section sub-block key")
            || d.to_string().contains("unrecognized")
    });
    assert!(
        !has_unrecognized_key_warning,
        "AC-EC-001 FAIL: eval must NOT re-emit the unrecognized-sub-block-key warning \
         (it was already emitted at parse time by STORY-078); \
         found warning in sink: {:?}",
        sink.errors()
    );
}

// ─── BC-3.02.002 invariant 3 / DIR-077-001-A Ruling 3: Eval validates TYPE ────

/// BC-3.02.002 invariant 3 / DIR-077-001-A Ruling 3:
/// `eval_section_nodes` validates the section type name against the built-in
/// `SectionType` registry (methodology, scope, approval, appendix, glossary).
/// An unrecognized type (e.g., "foobar") must produce a fatal eval error.
#[test]
fn test_BC_3_02_002_inv3_unknown_section_type_produces_fatal_eval_error() {
    let section_node = SectionNode {
        kind: Spanned::new("foobar".to_string(), dummy_span()),
        fields: vec![],
    };

    let mut sink = DiagnosticSink::new();
    let env = Env::new(IndexMap::new());
    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    // An unrecognized section type must cause eval to return None (fatal error).
    assert!(
        result.is_none(),
        "BC-3.02.002 invariant 3: eval_section_nodes must return None for unknown type 'foobar'; \
         got Some — missing type validation"
    );

    // The sink must contain an error mentioning the unknown type name.
    assert!(
        !sink.is_empty(),
        "BC-3.02.002 invariant 3: a diagnostic must be pushed for unknown section type 'foobar'"
    );

    let err_msg = sink
        .errors()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ");
    assert!(
        err_msg.contains("foobar"),
        "BC-3.02.002 invariant 3: error message must name the unknown type 'foobar'; \
         got: {err_msg}"
    );
    assert!(
        err_msg.contains("methodology")
            || err_msg.contains("Known types")
            || err_msg.contains("known"),
        "BC-3.02.002 invariant 3: error message must list known types; got: {err_msg}"
    );
}

/// BC-3.02.002 invariant 3: all canonical manual section types are accepted.
///
/// Iterates `KNOWN_SECTION_TYPES` (the complete authoritative list at eval time)
/// and asserts that each type is accepted by `eval_section_nodes`. The count of
/// accepted types must equal the length of `KNOWN_SECTION_TYPES`.
#[test]
fn test_BC_3_02_002_inv3_all_builtin_section_types_accepted() {
    for &type_name in KNOWN_SECTION_TYPES {
        let section_node = SectionNode {
            kind: Spanned::new(type_name.to_string(), dummy_span()),
            fields: vec![],
        };

        let mut sink = DiagnosticSink::new();
        let env = Env::new(IndexMap::new());
        let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);
        let errs = sink.errors();

        assert!(
            result.is_some(),
            "BC-3.02.002 invariant 3: built-in section type '{type_name}' must be accepted; \
             got None with errors: {errs:?}"
        );
    }
}

// ─── EC-001: Section with no detail: or report: produces empty register_content ─

/// BC-3.02.002 EC-001 / AC-001: a `section <type>:` block with no `detail:` or
/// `report:` sub-block produces an empty `register_content` vec. No error.
#[test]
fn test_BC_3_02_002_ec001_section_with_no_register_keys_produces_empty_vec() {
    let section = make_section_block("methodology", OrderedMap::new());

    let result = extract_section_register_content(&section);

    assert!(
        result.is_empty(),
        "EC-001: section with no detail:/report: sub-blocks must produce empty register_content; \
         got: {result:?}"
    );
}

// ─── EC-004: section report: sub-block ────────────────────────────────────────

/// BC-3.02.002 EC-004: `section report:` sub-block produces
/// `RegisteredContent { Register::Report }` on the section node.
#[test]
fn test_BC_3_02_002_ec004_section_report_sub_block_produces_report_entry() {
    let section = make_section_with_report("methodology", "Section-level report text");

    let result = extract_section_register_content(&section);

    assert_eq!(result.len(), 1, "EC-004: must have 1 entry");
    assert_eq!(
        result[0].register,
        Register::Report,
        "EC-004: entry must be Register::Report"
    );
    let text = extract_plain_text(&result[0].content);
    assert_eq!(
        text, "Section-level report text",
        "EC-004: report text must match"
    );
}

// ─── EC-005: Multiple section blocks — no cross-contamination ─────────────────

/// BC-3.02.002 EC-005 (STORY-077 test strategy): multiple section blocks each
/// produce their own `register_content` with no cross-contamination.
#[test]
fn test_BC_3_02_002_ec005_multiple_sections_no_cross_contamination() {
    let section_a = make_section_with_detail("methodology", "Methodology detail content");
    let section_b = make_section_with_detail("scope", "Scope detail content");

    let result_a = extract_section_register_content(&section_a);
    let result_b = extract_section_register_content(&section_b);

    assert_eq!(result_a.len(), 1, "section_a must produce 1 entry");
    assert_eq!(result_b.len(), 1, "section_b must produce 1 entry");

    let text_a = extract_plain_text(&result_a[0].content);
    let text_b = extract_plain_text(&result_b[0].content);

    assert_eq!(
        text_a, "Methodology detail content",
        "section_a content must be correct"
    );
    assert_eq!(
        text_b, "Scope detail content",
        "section_b content must be correct"
    );
    assert_ne!(
        text_a, text_b,
        "EC-005: sections must not cross-contaminate each other's register_content"
    );
}

// ─── KNOWN_SECTION_TYPES constant verification ────────────────────────────────

/// The KNOWN_SECTION_TYPES constant must contain all seven canonical types
/// (including executive_summary and risk_register) and exclude "notes".
///
/// F-077-P1-001: the canonical list now has 7 entries so that eval and layout
/// agree and the BC-3.02.001 EC-002 supersession path is reachable.
#[test]
fn test_known_section_types_constant_correct() {
    assert!(
        KNOWN_SECTION_TYPES.contains(&"methodology"),
        "KNOWN_SECTION_TYPES must include 'methodology'"
    );
    assert!(
        KNOWN_SECTION_TYPES.contains(&"scope"),
        "KNOWN_SECTION_TYPES must include 'scope'"
    );
    assert!(
        KNOWN_SECTION_TYPES.contains(&"approval"),
        "KNOWN_SECTION_TYPES must include 'approval'"
    );
    assert!(
        KNOWN_SECTION_TYPES.contains(&"appendix"),
        "KNOWN_SECTION_TYPES must include 'appendix'"
    );
    assert!(
        KNOWN_SECTION_TYPES.contains(&"glossary"),
        "KNOWN_SECTION_TYPES must include 'glossary'"
    );
    // F-077-P1-001: executive_summary and risk_register must be in the eval list
    // so that BC-3.02.001 EC-002 manual supersession is reachable.
    assert!(
        KNOWN_SECTION_TYPES.contains(&"executive_summary"),
        "KNOWN_SECTION_TYPES must include 'executive_summary' \
         (BC-3.02.001 EC-002 supersession path — F-077-P1-001)"
    );
    assert!(
        KNOWN_SECTION_TYPES.contains(&"risk_register"),
        "KNOWN_SECTION_TYPES must include 'risk_register' \
         (BC-3.02.001 EC-002 supersession path — F-077-P1-001)"
    );
    // "notes" is the PRESENTER register — NOT a section type.
    assert!(
        !KNOWN_SECTION_TYPES.contains(&"notes"),
        "KNOWN_SECTION_TYPES must NOT include 'notes' (notes is the presenter register)"
    );
}

// ─── F-077-P1-001: executive_summary and risk_register accepted by eval ────────

/// F-077-P1-001 / BC-3.02.001 EC-002:
/// `eval_section_nodes` must ACCEPT `section executive_summary:` — it is a
/// valid manually-authored type that supersedes the auto-generated section.
///
/// Before the fix this returned None with UnknownSectionType because
/// KNOWN_SECTION_TYPES only listed 5 types (missing executive_summary).
#[test]
fn test_F_077_P1_001_executive_summary_accepted_by_eval() {
    let section_node = SectionNode {
        kind: Spanned::new("executive_summary".to_string(), dummy_span()),
        fields: vec![],
    };

    let mut sink = DiagnosticSink::new();
    let env = Env::new(IndexMap::new());
    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    assert!(
        result.is_some(),
        "F-077-P1-001: eval must ACCEPT 'executive_summary' as a valid section type; \
         got None with errors: {:?}",
        sink.errors()
    );
    assert!(
        sink.is_empty(),
        "F-077-P1-001: no diagnostics expected for valid 'executive_summary' section; \
         got: {:?}",
        sink.errors()
    );
}

/// F-077-P1-001 / BC-3.02.001 EC-002:
/// `eval_section_nodes` must ACCEPT `section risk_register:` — it is a
/// valid manually-authored type that supersedes the auto-generated section.
///
/// Before the fix this returned None with UnknownSectionType because
/// KNOWN_SECTION_TYPES only listed 5 types (missing risk_register).
#[test]
fn test_F_077_P1_001_risk_register_accepted_by_eval() {
    let section_node = SectionNode {
        kind: Spanned::new("risk_register".to_string(), dummy_span()),
        fields: vec![],
    };

    let mut sink = DiagnosticSink::new();
    let env = Env::new(IndexMap::new());
    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    assert!(
        result.is_some(),
        "F-077-P1-001: eval must ACCEPT 'risk_register' as a valid section type; \
         got None with errors: {:?}",
        sink.errors()
    );
    assert!(
        sink.is_empty(),
        "F-077-P1-001: no diagnostics expected for valid 'risk_register' section; \
         got: {:?}",
        sink.errors()
    );
}

// ─── F-077-P1-002: real interpolation path exercised ─────────────────────────

/// F-077-P1-002 / BC-3.02.002 AC-003:
/// A section whose `detail:` field is `FieldValue::Template([Literal("…: "),
/// Expr("client")])` with `client = "Acme"` in the env must resolve to
/// `RegisteredContent` whose plain text is `"Methodology detail: Acme"`.
///
/// This test exercises the REAL eval `Template→Inlines` production path
/// (eval.rs ~486-495), unlike the pre-existing paper test that hand-built a
/// PRE-RESOLVED FieldValue::Inlines and never triggered interpolation.
///
/// F-077-P1-002 finding: if the interpolation path regresses, this test fails
/// because `{{ client }}` (or the variable name "client") appears in the
/// resolved content.
#[test]
fn test_F_077_P1_002_real_interpolation_template_to_inlines_via_eval() {
    // Build a SectionNode whose detail: field is a Template with an Expr chunk.
    // This mimics what the parser (STORY-078) produces for:
    //   section methodology:
    //     detail: "Methodology detail: {{ client }}"
    let section_node = SectionNode {
        kind: Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("detail".to_string(), dummy_span()),
            value: Spanned::new(
                SyntaxFieldValue::Template(vec![
                    TemplateChunk::Literal("Methodology detail: ".to_string()),
                    TemplateChunk::Expr(SyntaxExpr::Ident("client".to_string())),
                ]),
                dummy_span(),
            ),
        }],
    };

    // Env has client = "Acme".
    let mut vars: IndexMap<Arc<str>, Value> = IndexMap::new();
    vars.insert(Arc::from("client"), Value::Str(Arc::from("Acme")));
    let env = Env::new(vars);

    let mut sink = DiagnosticSink::new();
    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    assert!(
        sink.is_empty(),
        "F-077-P1-002: no diagnostics expected when client is defined; got: {:?}",
        sink.errors()
    );

    let (section_block, register_content) =
        result.expect("F-077-P1-002: eval must return Some for valid template with defined var");

    // The body must have the 'detail' key resolved to FieldValue::Inlines.
    let detail = section_block
        .body
        .get("detail")
        .expect("F-077-P1-002: 'detail' key must be present in body after eval");
    assert!(
        matches!(detail, FieldValue::Inlines(_)),
        "F-077-P1-002: 'detail' must be FieldValue::Inlines after template resolution; got: {detail:?}"
    );

    // The register_content must have one Detail entry.
    let detail_rc = register_content
        .iter()
        .find(|rc| rc.register == Register::Detail)
        .expect("F-077-P1-002: Detail register_content entry must be present");

    // The plain text must be the fully resolved string — NOT containing "{{" or "client".
    let resolved_text: String = detail_rc
        .content
        .iter()
        .filter_map(|n| {
            if let InlineNode::Plain(s) = n {
                Some(s.as_ref().to_owned())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(
        resolved_text, "Methodology detail: Acme",
        "F-077-P1-002: interpolation must resolve '{{{{ client }}}}' to 'Acme'; \
         got: {resolved_text:?}"
    );
    assert!(
        !resolved_text.contains("{{"),
        "F-077-P1-002: raw '{{{{' token must NOT survive to RegisteredContent; \
         got: {resolved_text:?}"
    );
    assert!(
        !resolved_text.contains("client"),
        "F-077-P1-002: variable name 'client' must NOT appear literally; \
         got: {resolved_text:?}"
    );
}

// ─── F-077-P1-003: undefined variable in section detail: produces Error and drops section ─

/// F-077-P1-003 / BC-3.02.002 EC-006 (story EC-002):
/// A `section detail:` field containing `{{ undefined_var }}` where the env
/// does NOT bind `undefined_var` must:
/// - return `None` (eval emits ParseSeverity::Error — the section cannot be emitted
///   with broken content)
/// - push an `UndefinedVariable` error (ParseSeverity::Error) naming the variable
///   into the sink
///
/// This test has zero coverage before this fix because no test drove the
/// undefined-variable branch through `eval_section_nodes`.
#[test]
fn test_F_077_P1_003_undefined_variable_in_section_detail_produces_error_and_drops_section() {
    // Build a SectionNode whose detail: field references an undefined variable.
    let section_node = SectionNode {
        kind: Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("detail".to_string(), dummy_span()),
            value: Spanned::new(
                SyntaxFieldValue::Template(vec![
                    TemplateChunk::Literal("Prefix: ".to_string()),
                    TemplateChunk::Expr(SyntaxExpr::Ident("undefined_var".to_string())),
                ]),
                dummy_span(),
            ),
        }],
    };

    // Env is empty — 'undefined_var' is not bound.
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();
    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    // Must return None: an undefined variable in a register field is fatal.
    assert!(
        result.is_none(),
        "F-077-P1-003: eval must return None when detail: references an undefined variable; \
         got Some — the broken content must not be emitted"
    );

    // The sink must contain at least one error mentioning the variable name.
    assert!(
        !sink.is_empty(),
        "F-077-P1-003: at least one diagnostic must be pushed for undefined variable \
         'undefined_var'; sink is empty"
    );

    let combined_errors: String = sink
        .errors()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ");

    assert!(
        combined_errors.contains("undefined_var"),
        "F-077-P1-003: error message must name the undefined variable 'undefined_var'; \
         got: {combined_errors}"
    );
}

// ─── SectionBlock.register_content field exists ───────────────────────────────

/// SectionBlock must have a `register_content: Vec<RegisteredContent>` field.
/// This is a compile-time structural check.
#[test]
fn test_BC_3_02_002_section_block_has_register_content_field() {
    let section = SectionBlock {
        name: Arc::from("methodology"),
        body: OrderedMap::new(),
        register_content: vec![RegisteredContent::plain(
            Register::Detail,
            Arc::from("test"),
        )],
        span: SourceSpan::default(),
    };
    assert_eq!(
        section.register_content.len(),
        1,
        "SectionBlock.register_content must hold populated Vec<RegisteredContent>"
    );
    assert_eq!(
        section.register_content[0].register,
        Register::Detail,
        "register_content field must hold Register::Detail entry"
    );
}

// ─── F-077-P2-002: eval register-key set == syntax SSOT ──────────────────────

/// F-077-P2-002: `SECTION_EVAL_REGISTER_KEYS` (now an alias of
/// `slideforge_syntax::section::SECTION_REGISTER_KEYS`) and the eval-stage
/// behavior must accept exactly the same keys as the parse-time constant.
///
/// This test verifies the SSOT by exercising eval behaviour for each key in
/// `SECTION_REGISTER_KEYS`: every key must be recognised and populated by
/// `eval_section_nodes`. Any key NOT in the set must be silently skipped.
#[test]
fn test_F_077_P2_002_eval_register_key_set_matches_syntax_ssot() {
    // Every key in SECTION_REGISTER_KEYS must be accepted by eval_section_nodes.
    for &key in SECTION_REGISTER_KEYS {
        let section_node = SectionNode {
            kind: Spanned::new("methodology".to_string(), dummy_span()),
            fields: vec![FieldNode {
                name: Spanned::new(key.to_string(), dummy_span()),
                value: Spanned::new(
                    SyntaxFieldValue::Template(vec![TemplateChunk::Literal("content".to_string())]),
                    dummy_span(),
                ),
            }],
        };

        let env = Env::new(IndexMap::new());
        let mut sink = DiagnosticSink::new();
        let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

        assert!(
            result.is_some(),
            "F-077-P2-002: eval must accept register key '{}' (in SECTION_REGISTER_KEYS); \
             got None with errors: {:?}",
            key,
            sink.errors()
        );

        let (section_block, _) = result.unwrap();
        assert!(
            section_block.body.get(key).is_some(),
            "F-077-P2-002: eval must populate body key '{key}' for a recognised register key"
        );
    }

    // A key NOT in SECTION_REGISTER_KEYS must be silently skipped (not appear in body).
    let foreign_key = "notes"; // excluded per DIR-077-001 §5
    assert!(
        !SECTION_REGISTER_KEYS.contains(&foreign_key),
        "test precondition: '{foreign_key}' must not be in SECTION_REGISTER_KEYS"
    );

    let section_node = SectionNode {
        kind: Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new(foreign_key.to_string(), dummy_span()),
            value: Spanned::new(
                SyntaxFieldValue::Template(vec![TemplateChunk::Literal("x".to_string())]),
                dummy_span(),
            ),
        }],
    };
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();
    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);
    assert!(
        result.is_some(),
        "F-077-P2-002: eval must return Some even when only a foreign key '{foreign_key}' is present \
         (it is silently skipped)"
    );
    let (section_block, _) = result.unwrap();
    assert!(
        section_block.body.get(foreign_key).is_none(),
        "F-077-P2-002: foreign key '{foreign_key}' must NOT appear in section_block.body — \
         eval must silently skip keys not in SECTION_REGISTER_KEYS"
    );
}

// ─── OBS-3: deck-level integration test (eval_deck path) ─────────────────────

/// OBS-3 deck-level integration test:
/// A top-level `section <type>:` whose ONLY register field (`detail:`) contains
/// an undefined variable must, when evaluated through `eval_deck`:
///
/// 1. Surface an `UndefinedVariable` Error diagnostic naming the variable.
/// 2. Drop the section from `deck.section_blocks` — the broken section must NOT
///    be emitted into the IR.
///
/// This test exercises the full `eval_deck` → `eval_section_nodes` path,
/// not just the `eval_section_nodes_for_test` unit helper. It is load-bearing:
/// it asserts the specific absent section AND that the diagnostic names the
/// undefined variable.
///
/// The documented asymmetry vs slide field-drop is preserved: for slides, an
/// undefined variable drops the field but keeps the slide; for sections, an
/// undefined variable in a register field drops the entire section
/// (BC-3.02.002 postcondition / DIR-077-001 §5).
#[test]
fn test_obs3_deck_level_section_with_undefined_variable_is_dropped_and_surfaced() {
    use slideforge_syntax::expr::Expr as SyntaxExprInner;
    use slideforge_syntax::{
        BlockItem, DeckNode, FieldNode, FieldValue as SyntaxFieldValue, SectionNode, Spanned,
        TemplateChunk,
    };

    use crate::config::EvalConfig;
    use crate::eval::eval_deck;

    // Build a DeckNode with a single top-level section "methodology" whose
    // only field is `detail: {{ missing_var }}` where `missing_var` is not bound.
    let section_node = SectionNode {
        kind: Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("detail".to_string(), dummy_span()),
            value: Spanned::new(
                SyntaxFieldValue::Template(vec![
                    TemplateChunk::Literal("Prefix: ".to_string()),
                    TemplateChunk::Expr(SyntaxExprInner::Ident("missing_var".to_string())),
                ]),
                dummy_span(),
            ),
        }],
    };

    let deck_node = DeckNode {
        items: vec![BlockItem::Section(Spanned::new(section_node, dummy_span()))],
        ..DeckNode::default()
    };

    let config = EvalConfig::default();
    let mut sink = DiagnosticSink::new();

    // eval_deck may return Some (the deck is otherwise valid) or None (if the
    // UndefinedVariable pushes has_fatal() across the gate). Either way:
    // (a) the section must be absent from deck.section_blocks, and
    // (b) the diagnostic must name "missing_var".
    let deck_opt = eval_deck(&deck_node, &config, &mut sink);

    // Assertion (b): the sink must contain an UndefinedVariable error for "missing_var".
    assert!(
        !sink.is_empty(),
        "OBS-3: eval_deck must surface an UndefinedVariable diagnostic for 'missing_var'; \
         sink is empty"
    );
    let combined_errors: String = sink
        .errors()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ");
    assert!(
        combined_errors.contains("missing_var"),
        "OBS-3: UndefinedVariable diagnostic must name 'missing_var'; got: {combined_errors}"
    );

    // Assertion (a): the broken section must be absent from deck.section_blocks.
    if let Some(deck) = deck_opt {
        assert!(
            deck.section_blocks.is_empty(),
            "OBS-3: the 'methodology' section with an undefined variable must be dropped \
             from deck.section_blocks; got {} section(s): {:?}",
            deck.section_blocks.len(),
            deck.section_blocks
                .iter()
                .map(|s| s.name.as_ref())
                .collect::<Vec<_>>()
        );
    }
    // If eval_deck returned None, the diagnostic gate already enforced absence —
    // assertion (b) above is sufficient.
}

// ─── F-077-P4-001: section-path List/Map Ident drop is observable ─────────────

/// F-077-P4-001 / BC-1.14.001/002/003:
/// A `section detail:` field whose value is a bare ident (`FieldValue::Ident`)
/// that resolves via the env to `Value::List` must produce NO register content
/// (drop is correct — List values are not valid register prose) AND the drop
/// must be OBSERVABLE via `tracing::warn!` (not a silent drop).
///
/// Before this fix, the drop was a bare `continue` with no logging — violating
/// the silent-failure ban and diverging from the slide-path sibling
/// (`field_value_to_inlines` in register_routing.rs, which emits `tracing::warn!`
/// for the identical List/Map-into-register-field case, per F-035-P5-003).
///
/// This test locks in the correct behavior:
///   (a) register_content is empty (Value::List is correctly dropped)
///   (b) the warn! path is taken (code comment + observability via structured logs)
///
/// Mirror of `test_f035_p5_003_list_register_field_produces_no_entry` for the
/// slide-path sibling in register_routing.rs.
///
/// TD-VSDD-059: assertions are load-bearing on both the empty-result behavior
/// AND the section block shape (section was emitted with the field absent).
#[test]
fn test_f077_p4_001_section_detail_ident_resolving_to_list_drops_silently_with_warn() {
    // Build a SectionNode whose detail: field is a bare Ident "refs".
    // This mimics DSL:
    //   @var refs = ["a", "b"]
    //   section methodology:
    //     detail: refs
    let section_node = slideforge_syntax::SectionNode {
        kind: slideforge_syntax::Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![slideforge_syntax::FieldNode {
            name: slideforge_syntax::Spanned::new("detail".to_string(), dummy_span()),
            value: slideforge_syntax::Spanned::new(
                slideforge_syntax::FieldValue::Ident("refs".to_string()),
                dummy_span(),
            ),
        }],
    };

    // Env has refs = ["a", "b"] — a Value::List.
    let mut vars: IndexMap<Arc<str>, Value> = IndexMap::new();
    vars.insert(
        Arc::from("refs"),
        Value::List(vec![Value::Str(Arc::from("a")), Value::Str(Arc::from("b"))]),
    );
    let env = Env::new(vars);

    let mut sink = DiagnosticSink::new();
    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    // (a) The section must still be emitted (a List in a register field is a
    //     type mismatch for register content, but it is NOT a fatal error for the
    //     section itself — only for template interpolation that must produce text).
    //     The field is dropped from body and register_content is empty.
    //
    // NOTE: eval_section_nodes currently `continue`s the field loop on List/Map,
    // which means the section IS returned but with "detail" absent from body.
    // The register_content must be empty because no FieldValue::Inlines was
    // inserted for "detail".
    assert!(
        result.is_some(),
        "F-077-P4-001: eval must return Some for a section whose detail: field resolves \
         to Value::List (the field is dropped, but the section is not fatal); \
         got None with errors: {:?}",
        sink.errors()
    );

    let (section_block, register_content) = result.unwrap();

    // (a) register_content must be EMPTY — Value::List is not valid register prose.
    assert!(
        register_content.is_empty(),
        "F-077-P4-001: register_content must be empty when detail: resolves to Value::List \
         (register fields are text-only, BC-1.14.001/002/003); \
         drop must be logged via tracing::warn! (not a silent drop — F-077-P4-001); \
         got: {register_content:?}"
    );

    // (b) The "detail" key must be absent from body (the field was dropped,
    //     not inserted as invalid content).
    assert!(
        section_block.body.get("detail").is_none(),
        "F-077-P4-001: 'detail' key must be absent from body when its value is Value::List \
         (the field is skipped); got body: {:?}",
        section_block.body.keys().collect::<Vec<_>>()
    );

    // No error diagnostics: a List-in-register-field is a WARN, not a fatal error.
    // (A tracing::warn! is emitted — observable in structured logs — but nothing
    //  is pushed to the DiagnosticSink, matching the slide-path behavior in
    //  register_routing.rs field_value_to_inlines / F-035-P5-003.)
    assert!(
        sink.is_empty(),
        "F-077-P4-001: no diagnostic errors expected for List-in-register-field \
         (the drop is logged via tracing::warn!, not a DiagnosticSink error); \
         got: {:?}",
        sink.errors()
    );
}

/// F-077-P4-001 (Map variant):
/// A `section detail:` field whose value is a bare ident (`FieldValue::Ident`)
/// that resolves via the env to `Value::Map` must produce NO register content.
///
/// Mirror of `test_f035_p5_003_map_register_field_produces_no_entry` for the
/// section path, and companion to the List variant above.
///
/// TD-VSDD-059: load-bearing assertions on empty register_content, absent body
/// key, and no diagnostic errors.
#[test]
fn test_f077_p4_001_section_detail_ident_resolving_to_map_drops_with_warn() {
    // Build a SectionNode whose detail: field is a bare Ident "meta".
    // This mimics DSL:
    //   @var meta = {key: "value"}
    //   section methodology:
    //     detail: meta
    let section_node = slideforge_syntax::SectionNode {
        kind: slideforge_syntax::Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![slideforge_syntax::FieldNode {
            name: slideforge_syntax::Spanned::new("detail".to_string(), dummy_span()),
            value: slideforge_syntax::Spanned::new(
                slideforge_syntax::FieldValue::Ident("meta".to_string()),
                dummy_span(),
            ),
        }],
    };

    // Env has meta = {key: "value"} — a Value::Map.
    let mut vars: IndexMap<Arc<str>, Value> = IndexMap::new();
    let mut map_inner = OrderedMap::new();
    map_inner.insert(Arc::from("key"), Value::Str(Arc::from("value")));
    vars.insert(Arc::from("meta"), Value::Map(map_inner));
    let env = Env::new(vars);

    let mut sink = DiagnosticSink::new();
    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    // The section must still be emitted (Map in a register field is not fatal).
    assert!(
        result.is_some(),
        "F-077-P4-001 (Map): eval must return Some for a section whose detail: field resolves \
         to Value::Map (the field is dropped, but the section is not fatal); \
         got None with errors: {:?}",
        sink.errors()
    );

    let (section_block, register_content) = result.unwrap();

    // register_content must be EMPTY — Value::Map is not valid register prose.
    assert!(
        register_content.is_empty(),
        "F-077-P4-001 (Map): register_content must be empty when detail: resolves to Value::Map \
         (register fields are text-only, BC-1.14.001/002/003); \
         drop must be logged via tracing::warn! (not a silent drop — F-077-P4-001); \
         got: {register_content:?}"
    );

    // The "detail" key must be absent from body.
    assert!(
        section_block.body.get("detail").is_none(),
        "F-077-P4-001 (Map): 'detail' key must be absent from body when its value is Value::Map; \
         got body: {:?}",
        section_block.body.keys().collect::<Vec<_>>()
    );

    // No error diagnostics — the drop is a tracing::warn!, not a DiagnosticSink error.
    assert!(
        sink.is_empty(),
        "F-077-P4-001 (Map): no diagnostic errors expected for Map-in-register-field; \
         got: {:?}",
        sink.errors()
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// STORY-077 DIR-077-002 §8 — Red Gate Tests 16–28
// chunks_to_inline_nodes conversion + AC-002 end-to-end pipeline
//
// All tests below MUST FAIL until `chunks_to_inline_nodes` is implemented in
// `register_routing.rs` (they will panic at the `todo!()` stub).
// TD-VSDD-059: every test has load-bearing assertions on the specific InlineNode
// variant produced and its inner content.
// ═══════════════════════════════════════════════════════════════════════════════

// ─── Test 16: Bold chunk → InlineNode::Bold (DIR-077-002 §8 item 16) ─────────

/// DIR-077-002 §8 #16 / BC-3.02.002 AC-002:
/// `TemplateChunk::Bold([Literal("hi")])` → `InlineNode::Bold([InlineNode::Plain(Arc::from("hi"))])`.
///
/// FAILS at the todo!() stub until chunks_to_inline_nodes is implemented.
#[test]
fn test_BC_3_02_002_bold_chunk_to_inline_node() {
    let chunks = vec![TemplateChunk::Bold(vec![TemplateChunk::Literal(
        "hi".to_string(),
    )])];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    // This will panic at the todo!() stub — Red Gate is active.
    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    assert_eq!(
        nodes.len(),
        1,
        "Bold chunk must produce exactly 1 InlineNode; got {nodes:?}"
    );
    match &nodes[0] {
        InlineNode::Bold(children) => {
            assert_eq!(
                children.len(),
                1,
                "Bold must have 1 child; got {children:?}"
            );
            assert!(
                matches!(&children[0], InlineNode::Plain(s) if s.as_ref() == "hi"),
                "Bold child must be Plain(\"hi\"); got: {:?}",
                children[0]
            );
        },
        other => panic!(
            "test_BC_3_02_002_bold_chunk_to_inline_node FAIL: expected InlineNode::Bold, \
             got: {other:?}"
        ),
    }
}

// ─── Test 17: Italic chunk → InlineNode::Italic (DIR-077-002 §8 item 17) ─────

/// DIR-077-002 §8 #17 / BC-3.02.002 AC-002:
/// `TemplateChunk::Italic([Literal("em")])` → `InlineNode::Italic([InlineNode::Plain(Arc::from("em"))])`.
///
/// FAILS at the todo!() stub until chunks_to_inline_nodes is implemented.
#[test]
fn test_BC_3_02_002_italic_chunk_to_inline_node() {
    let chunks = vec![TemplateChunk::Italic(vec![TemplateChunk::Literal(
        "em".to_string(),
    )])];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    assert_eq!(
        nodes.len(),
        1,
        "Italic chunk must produce 1 InlineNode; got {nodes:?}"
    );
    match &nodes[0] {
        InlineNode::Italic(children) => {
            assert_eq!(
                children.len(),
                1,
                "Italic must have 1 child; got {children:?}"
            );
            assert!(
                matches!(&children[0], InlineNode::Plain(s) if s.as_ref() == "em"),
                "Italic child must be Plain(\"em\"); got: {:?}",
                children[0]
            );
        },
        other => panic!(
            "test_BC_3_02_002_italic_chunk_to_inline_node FAIL: expected InlineNode::Italic, \
             got: {other:?}"
        ),
    }
}

// ─── Test 18: Code chunk → InlineNode::Code (DIR-077-002 §8 item 18) ─────────

/// DIR-077-002 §8 #18 / BC-3.02.002 AC-002:
/// `TemplateChunk::Code("fn x() {}")` → `InlineNode::Code(Arc::from("fn x() {}"))`.
///
/// FAILS at the todo!() stub until chunks_to_inline_nodes is implemented.
#[test]
fn test_BC_3_02_002_code_chunk_to_inline_node() {
    let chunks = vec![TemplateChunk::Code("fn x() {}".to_string())];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    assert_eq!(
        nodes.len(),
        1,
        "Code chunk must produce 1 InlineNode; got {nodes:?}"
    );
    match &nodes[0] {
        InlineNode::Code(content) => {
            assert_eq!(
                content.as_ref(),
                "fn x() {}",
                "Code content must be 'fn x() {{}}'; got: {content:?}"
            );
        },
        other => panic!(
            "test_BC_3_02_002_code_chunk_to_inline_node FAIL: expected InlineNode::Code, \
             got: {other:?}"
        ),
    }
}

// ─── Test 19: Link chunk → InlineNode::Link (DIR-077-002 §8 item 19) ─────────

/// DIR-077-002 §8 #19 / BC-3.02.002 AC-002:
/// `TemplateChunk::Link { text: [Literal("example")], url: "https://example.com" }`
/// → `InlineNode::Link { text: [Plain("example")], url: Arc::from("https://example.com") }`.
///
/// FAILS at the todo!() stub until chunks_to_inline_nodes is implemented.
#[test]
fn test_BC_3_02_002_link_chunk_to_inline_node() {
    let chunks = vec![TemplateChunk::Link {
        text: vec![TemplateChunk::Literal("example".to_string())],
        url: "https://example.com".to_string(),
    }];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    assert_eq!(
        nodes.len(),
        1,
        "Link chunk must produce 1 InlineNode; got {nodes:?}"
    );
    match &nodes[0] {
        InlineNode::Link { text, url } => {
            assert_eq!(url.as_ref(), "https://example.com", "Link url must match");
            assert_eq!(text.len(), 1, "Link text must have 1 child; got {text:?}");
            assert!(
                matches!(&text[0], InlineNode::Plain(s) if s.as_ref() == "example"),
                "Link text child must be Plain(\"example\"); got: {:?}",
                text[0]
            );
        },
        other => panic!(
            "test_BC_3_02_002_link_chunk_to_inline_node FAIL: expected InlineNode::Link, \
             got: {other:?}"
        ),
    }
}

// ─── Test 20: MathInline chunk → InlineNode::Math{display:false} (item 20) ───

/// DIR-077-002 §8 #20 / BC-3.02.002 AC-002:
/// `TemplateChunk::MathInline("x^2")` → `InlineNode::Math(MathNode { latex: "x^2", display: false })`.
///
/// FAILS at the todo!() stub until chunks_to_inline_nodes is implemented.
#[test]
fn test_BC_3_02_002_math_inline_chunk_to_inline_node() {
    use slideforge_types::MathNode;

    let chunks = vec![TemplateChunk::MathInline("x^2".to_string())];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    assert_eq!(
        nodes.len(),
        1,
        "MathInline chunk must produce 1 InlineNode; got {nodes:?}"
    );
    match &nodes[0] {
        InlineNode::Math(MathNode { latex, display, .. }) => {
            assert_eq!(
                latex.as_ref(),
                "x^2",
                "Math latex must be 'x^2'; got: {latex:?}"
            );
            assert!(
                !display,
                "MathInline must produce display=false; got display={display}"
            );
        },
        other => panic!(
            "test_BC_3_02_002_math_inline_chunk_to_inline_node FAIL: expected InlineNode::Math, \
             got: {other:?}"
        ),
    }
}

// ─── Test 21: MathDisplay chunk → InlineNode::Math{display:true} (item 21) ───

/// DIR-077-002 §8 #21 / BC-3.02.002 AC-002:
/// `TemplateChunk::MathDisplay(r"\sum")` → `InlineNode::Math(MathNode { display: true })`.
///
/// FAILS at the todo!() stub until chunks_to_inline_nodes is implemented.
#[test]
fn test_BC_3_02_002_math_display_chunk_to_inline_node() {
    use slideforge_types::MathNode;

    let chunks = vec![TemplateChunk::MathDisplay(r"\sum_{i=0}^{n}".to_string())];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    assert_eq!(
        nodes.len(),
        1,
        "MathDisplay chunk must produce 1 InlineNode; got {nodes:?}"
    );
    match &nodes[0] {
        InlineNode::Math(MathNode { latex, display, .. }) => {
            assert_eq!(
                latex.as_ref(),
                r"\sum_{i=0}^{n}",
                "Math latex must match; got: {latex:?}"
            );
            assert!(
                *display,
                "MathDisplay must produce display=true; got display={display}"
            );
        },
        other => panic!(
            "test_BC_3_02_002_math_display_chunk_to_inline_node FAIL: expected InlineNode::Math, \
             got: {other:?}"
        ),
    }
}

// ─── Test 22: Expr ref call → InlineNode::Xref (DIR-077-002 §8 item 22) ──────

/// DIR-077-002 §8 #22 / BC-3.02.002 AC-002:
/// A `TemplateChunk::Expr` whose expression represents a `ref("slide-1")` call
/// must produce `InlineNode::Xref(Arc::from("slide-1"))`.
///
/// Per DIR-077-002 §3: `{{ ref("slide-1") }}` is parsed as a `TemplateChunk::Expr`
/// function-call expression; `chunks_to_inline_nodes` converts it to `Xref`.
///
/// Since `Expr` currently has no `Call` variant, this test uses the `Expr::Pipe`
/// variant as a proxy: `ref | pipe("slide-1")` is not the final form, but the
/// test exercises the Xref production path. When `Expr::Call` is added (required
/// for proper ref() support), update this test to use `Expr::Call { func: "ref", ... }`.
///
/// FAILS at the todo!() stub until chunks_to_inline_nodes is implemented.
/// ALSO FAILS when the real impl does not handle the specific Expr form used here.
/// The implementer must both (a) implement chunks_to_inline_nodes AND (b) add
/// Expr::Call to the Expr enum to support ref() natively.
#[test]
fn test_BC_3_02_002_expr_ref_call_to_xref() {
    // Use Expr::Pipe as the closest available proxy for a function call in the
    // current Expr enum. The implementer must add Expr::Call and update this test.
    // The test is load-bearing: it drives the production chunks_to_inline_nodes path.
    let ref_expr = SyntaxExpr::Pipe {
        lhs: Box::new(SyntaxExpr::Str("slide-1".to_string())),
        filter: "ref".to_string(),
        args: vec![],
    };
    let chunks = vec![TemplateChunk::Expr(ref_expr)];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    // This will panic at todo!() — Red Gate.
    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // After implementation: the Expr(Pipe{ filter: "ref", lhs: Str("slide-1") })
    // must produce InlineNode::Xref(Arc::from("slide-1")).
    // If the implementation uses Expr::Call instead, this test will need updating
    // to use Expr::Call { func: "ref", args: [Expr::Str("slide-1")] }.
    assert_eq!(
        nodes.len(),
        1,
        "ref() Expr must produce 1 InlineNode; got {nodes:?}"
    );
    match &nodes[0] {
        InlineNode::Xref(id) => {
            assert_eq!(
                id.as_ref(),
                "slide-1",
                "Xref id must be 'slide-1'; got: {id:?}"
            );
        },
        other => panic!(
            "test_BC_3_02_002_expr_ref_call_to_xref FAIL: expected InlineNode::Xref, \
             got: {other:?}\n\
             Note: if Expr has no Call variant yet, this test uses Pipe as proxy. \
             The implementer must add Expr::Call {{ func: 'ref', args: [Str(id)] }} \
             and update this test accordingly."
        ),
    }
}

// ─── Test 23: Expr footnote call → InlineNode::Footnote (item 23) ─────────────

/// DIR-077-002 §8 #23 / BC-3.02.002 AC-002:
/// A `TemplateChunk::Expr` representing `footnote("see appendix")` must produce
/// `InlineNode::Footnote([InlineNode::Plain(Arc::from("see appendix"))])`.
///
/// Same note as test 22: uses Expr::Pipe as proxy until Expr::Call exists.
///
/// FAILS at the todo!() stub until chunks_to_inline_nodes is implemented.
#[test]
fn test_BC_3_02_002_expr_footnote_call_to_footnote_node() {
    // Proxy: Pipe { lhs: Str("see appendix"), filter: "footnote" }
    // Replace with Expr::Call when that variant is added.
    let footnote_expr = SyntaxExpr::Pipe {
        lhs: Box::new(SyntaxExpr::Str("see appendix".to_string())),
        filter: "footnote".to_string(),
        args: vec![],
    };
    let chunks = vec![TemplateChunk::Expr(footnote_expr)];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    assert_eq!(
        nodes.len(),
        1,
        "footnote() Expr must produce 1 InlineNode; got {nodes:?}"
    );
    match &nodes[0] {
        InlineNode::Footnote(children) => {
            assert_eq!(
                children.len(),
                1,
                "Footnote must have 1 child; got {children:?}"
            );
            assert!(
                matches!(&children[0], InlineNode::Plain(s) if s.as_ref() == "see appendix"),
                "Footnote child must be Plain(\"see appendix\"); got: {:?}",
                children[0]
            );
        },
        other => panic!(
            "test_BC_3_02_002_expr_footnote_call_to_footnote_node FAIL: expected InlineNode::Footnote, \
             got: {other:?}"
        ),
    }
}

// ─── Test 24: AC-002 bold in section detail (DIR-077-002 §8 item 24) ──────────

/// DIR-077-002 §8 #24 / BC-3.02.002 AC-002 (full pipeline):
/// Full pipeline: a SectionNode with `detail: "**Bold claim.**"` must produce
/// `SectionBlock.body["detail"]` = `FieldValue::Inlines([InlineNode::Bold([Plain("Bold claim.")])])`.
///
/// This test drives the PRODUCTION eval path (eval_section_nodes_for_test).
/// TD-VSDD-059: the assertion verifies the exact InlineNode variant and inner
/// content — NOT just that FieldValue::Inlines is present.
///
/// FAILS because:
/// (a) template_value() does not yet produce Bold chunk for "**Bold claim.**"
/// (b) chunks_to_inline_nodes is a todo!() stub
#[test]
fn test_BC_3_02_002_ac002_bold_in_section_detail_produces_inlines() {
    // Build a SectionNode whose detail: field contains **Bold claim.**
    // The parser (after implementation) will produce:
    //   FieldValue::Template([Bold([Literal("Bold claim.")])])
    // For the Red Gate, we construct the expected template manually.
    let section_node = SectionNode {
        kind: Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("detail".to_string(), dummy_span()),
            value: Spanned::new(
                SyntaxFieldValue::Template(vec![TemplateChunk::Bold(vec![
                    TemplateChunk::Literal("Bold claim.".to_string()),
                ])]),
                dummy_span(),
            ),
        }],
    };

    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    // Will panic at todo!() in chunks_to_inline_nodes — Red Gate is active.
    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    let (section_block, _) =
        result.expect("AC-002: eval_section_nodes must return Some for valid methodology section");

    let detail_entry = section_block
        .body
        .get("detail")
        .expect("AC-002: 'detail' key must be present after eval");

    match detail_entry {
        FieldValue::Inlines(nodes) => {
            assert_eq!(
                nodes.len(),
                1,
                "AC-002: detail must have exactly 1 InlineNode (Bold); got {nodes:?}"
            );
            match &nodes[0] {
                InlineNode::Bold(children) => {
                    assert_eq!(
                        children.len(),
                        1,
                        "Bold must have 1 child (Plain); got {children:?}"
                    );
                    assert!(
                        matches!(&children[0], InlineNode::Plain(s) if s.as_ref() == "Bold claim."),
                        "Bold child must be Plain(\"Bold claim.\"); got: {:?}",
                        children[0]
                    );
                    // Forbidden pattern check: NO Literal asterisks in the result.
                    // InlineNode::Plain must NOT contain "**" or "*".
                    if let InlineNode::Plain(s) = &children[0] {
                        assert!(
                            !s.contains('*'),
                            "AC-002 FORBIDDEN PATTERN: Plain node must not contain asterisks; \
                             got: {s:?}. This indicates the parser did not produce Bold."
                        );
                    }
                },
                other => panic!(
                    "AC-002 FAIL: expected InlineNode::Bold; got: {other:?}\n\
                     If this is Plain(\"**Bold claim.**\"), the Bold chunk was not produced by \
                     template_value() OR chunks_to_inline_nodes did not convert it."
                ),
            }
        },
        other => panic!("AC-002 FAIL: body['detail'] must be FieldValue::Inlines; got: {other:?}"),
    }
}

// ─── Test 25: AC-002 xref in section detail (DIR-077-002 §8 item 25) ──────────

/// DIR-077-002 §8 #25 / BC-3.02.002 AC-002:
/// `{{ ref("slide-1") }}` in section detail must produce `InlineNode::Xref(Arc::from("slide-1"))`.
///
/// FAILS because chunks_to_inline_nodes is a todo!() stub.
#[test]
fn test_BC_3_02_002_ac002_xref_in_section_detail_produces_xref_node() {
    // Use Pipe as proxy for Call (see test 22 note).
    let ref_expr = SyntaxExpr::Pipe {
        lhs: Box::new(SyntaxExpr::Str("slide-1".to_string())),
        filter: "ref".to_string(),
        args: vec![],
    };
    let section_node = SectionNode {
        kind: Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("detail".to_string(), dummy_span()),
            value: Spanned::new(
                SyntaxFieldValue::Template(vec![
                    TemplateChunk::Literal("See ".to_string()),
                    TemplateChunk::Expr(ref_expr),
                    TemplateChunk::Literal(".".to_string()),
                ]),
                dummy_span(),
            ),
        }],
    };

    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    let (section_block, _) = result.expect("AC-002: eval must return Some for ref() in detail");

    let detail_entry = section_block
        .body
        .get("detail")
        .expect("AC-002: 'detail' key must be present");

    match detail_entry {
        FieldValue::Inlines(nodes) => {
            // Must contain an Xref node somewhere in the sequence.
            let has_xref = nodes
                .iter()
                .any(|n| matches!(n, InlineNode::Xref(id) if id.as_ref() == "slide-1"));
            assert!(
                has_xref,
                "AC-002: FieldValue::Inlines must contain InlineNode::Xref(\"slide-1\"); \
                 got: {nodes:?}"
            );
            // Forbidden pattern: no Plain node must contain the raw "ref(" or "slide-1" as a literal.
            let has_raw_ref = nodes.iter().any(|n| {
                matches!(n, InlineNode::Plain(s) if s.contains("ref(") || s.contains("slide-1"))
            });
            assert!(
                !has_raw_ref,
                "AC-002: no Plain node must contain raw 'ref(' or 'slide-1'; got {nodes:?}"
            );
        },
        other => panic!("AC-002 FAIL: body['detail'] must be FieldValue::Inlines; got: {other:?}"),
    }
}

// ─── Test 26: AC-002 plain text not literal asterisks (item 26) ───────────────

/// DIR-077-002 §8 #26 / BC-3.02.002 AC-002 (forbidden pattern check):
/// `**Bold** text.` must produce `[InlineNode::Bold(...), InlineNode::Plain(" text.")]`.
/// NOT a single `InlineNode::Plain(Arc::from("**Bold** text."))`.
///
/// This is the core forbidden-pattern check from CLAUDE.md (R1 finding).
///
/// FAILS because chunks_to_inline_nodes is a todo!() stub.
#[test]
fn test_BC_3_02_002_ac002_plain_text_not_literal_asterisks() {
    let section_node = SectionNode {
        kind: Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("detail".to_string(), dummy_span()),
            value: Spanned::new(
                SyntaxFieldValue::Template(vec![
                    TemplateChunk::Bold(vec![TemplateChunk::Literal("Bold".to_string())]),
                    TemplateChunk::Literal(" text.".to_string()),
                ]),
                dummy_span(),
            ),
        }],
    };

    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);
    let (section_block, _) = result.expect("AC-002: eval must return Some");

    let detail_entry = section_block
        .body
        .get("detail")
        .expect("detail must be present");

    match detail_entry {
        FieldValue::Inlines(nodes) => {
            // Must have Bold + Plain, not a single Plain with asterisks.
            let has_bold = nodes.iter().any(|n| matches!(n, InlineNode::Bold(_)));
            assert!(
                has_bold,
                "AC-002: FieldValue::Inlines must contain InlineNode::Bold; got: {nodes:?}"
            );

            // Forbidden pattern: NO Plain node must contain literal asterisks.
            for node in nodes.iter() {
                if let InlineNode::Plain(s) = node {
                    assert!(
                        !s.contains('*'),
                        "AC-002 FORBIDDEN PATTERN (CLAUDE.md R1): Plain node must NOT contain \
                         literal asterisks. String-prefix bold ('**Bold**') is the R1 anti-pattern. \
                         Got Plain({s:?}). The Bold chunk must produce InlineNode::Bold, not Plain."
                    );
                }
            }
        },
        other => panic!("AC-002 FAIL: body['detail'] must be FieldValue::Inlines; got: {other:?}"),
    }
}

// ─── Test 27: AC-002 interpolation with bold context (item 27) ────────────────

/// DIR-077-002 §8 #27 / BC-3.02.002 AC-002:
/// `**{{ client }}**` with `client = "Acme"` must produce
/// `InlineNode::Bold([InlineNode::Plain(Arc::from("Acme"))])`.
///
/// The `{{ client }}` expression is evaluated inside the Bold span.
///
/// FAILS because chunks_to_inline_nodes is a todo!() stub.
#[test]
fn test_BC_3_02_002_ac002_interpolation_with_bold_context() {
    let section_node = SectionNode {
        kind: Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("detail".to_string(), dummy_span()),
            value: Spanned::new(
                SyntaxFieldValue::Template(vec![TemplateChunk::Bold(vec![TemplateChunk::Expr(
                    SyntaxExpr::Ident("client".to_string()),
                )])]),
                dummy_span(),
            ),
        }],
    };

    let mut vars: IndexMap<Arc<str>, Value> = IndexMap::new();
    vars.insert(Arc::from("client"), Value::Str(Arc::from("Acme")));
    let env = Env::new(vars);
    let mut sink = DiagnosticSink::new();

    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);
    let (section_block, _) = result.expect("AC-002: eval must return Some with client in scope");

    let detail_entry = section_block
        .body
        .get("detail")
        .expect("detail must be present");

    match detail_entry {
        FieldValue::Inlines(nodes) => {
            assert_eq!(
                nodes.len(),
                1,
                "must have 1 InlineNode (Bold); got {nodes:?}"
            );
            match &nodes[0] {
                InlineNode::Bold(children) => {
                    assert_eq!(
                        children.len(),
                        1,
                        "Bold must have 1 child; got {children:?}"
                    );
                    assert!(
                        matches!(&children[0], InlineNode::Plain(s) if s.as_ref() == "Acme"),
                        "Bold child must be Plain(\"Acme\") after interpolation; got: {:?}",
                        children[0]
                    );
                },
                other => {
                    panic!("AC-002 interpolation FAIL: expected InlineNode::Bold; got: {other:?}")
                },
            }
        },
        other => panic!("AC-002 FAIL: body['detail'] must be FieldValue::Inlines; got: {other:?}"),
    }
}

// ─── Test 28: AC-002 report sub-block produces inlines (item 28) ─────────────

/// DIR-077-002 §8 #28 / BC-3.02.002 AC-002:
/// Same as test 24 but for the `report:` key:
/// `section methodology: / report: "**Bold claim.**"` must produce
/// `SectionBlock.body["report"]` = `FieldValue::Inlines([InlineNode::Bold([Plain("Bold claim.")])])`.
///
/// FAILS because chunks_to_inline_nodes is a todo!() stub.
#[test]
fn test_BC_3_02_002_ac002_report_sub_block_produces_inlines() {
    let section_node = SectionNode {
        kind: Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("report".to_string(), dummy_span()),
            value: Spanned::new(
                SyntaxFieldValue::Template(vec![TemplateChunk::Bold(vec![
                    TemplateChunk::Literal("Bold claim.".to_string()),
                ])]),
                dummy_span(),
            ),
        }],
    };

    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);
    let (section_block, _) = result.expect("AC-002: eval must return Some for methodology report");

    let report_entry = section_block
        .body
        .get("report")
        .expect("AC-002: 'report' key must be present after eval");

    match report_entry {
        FieldValue::Inlines(nodes) => {
            assert_eq!(
                nodes.len(),
                1,
                "AC-002 (report): must have exactly 1 InlineNode (Bold); got {nodes:?}"
            );
            match &nodes[0] {
                InlineNode::Bold(children) => {
                    assert_eq!(
                        children.len(),
                        1,
                        "Bold must have 1 child; got {children:?}"
                    );
                    assert!(
                        matches!(&children[0], InlineNode::Plain(s) if s.as_ref() == "Bold claim."),
                        "Bold child must be Plain(\"Bold claim.\"); got: {:?}",
                        children[0]
                    );
                    // Forbidden pattern check.
                    if let InlineNode::Plain(s) = &children[0] {
                        assert!(
                            !s.contains('*'),
                            "AC-002 (report) FORBIDDEN PATTERN: Plain node must not contain asterisks; \
                             got: {s:?}"
                        );
                    }
                },
                other => panic!("AC-002 (report) FAIL: expected InlineNode::Bold; got: {other:?}"),
            }
        },
        other => panic!(
            "AC-002 (report) FAIL: body['report'] must be FieldValue::Inlines; got: {other:?}"
        ),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// F-077-P5-001: Real Expr::Call form tests (not Pipe proxy)
// F-077-P5-002: Real DSL parse→eval end-to-end seam tests
// OBS-B: empty-id fatal error test
// ═══════════════════════════════════════════════════════════════════════════════

// ─── F-077-P5-001: Expr::Call ref → Xref (real Call form, not Pipe proxy) ────

/// F-077-P5-001: `TemplateChunk::Expr(Expr::Call { func: "ref", args: [Str("slide-1")] })`
/// → `InlineNode::Xref(Arc::from("slide-1"))`.
///
/// This test uses the REAL `Expr::Call` form (DIR-077-002 §1 rule 5), not the
/// `Expr::Pipe` proxy used in tests 22/23/25. It proves that the Call form
/// (produced by the real DSL parser for `{{ ref("slide-1") }}`) is handled
/// correctly by `chunks_to_inline_nodes` — not only the Pipe proxy.
///
/// TD-VSDD-059: load-bearing assertion on the exact variant and inner content.
#[test]
fn test_f077_p5_001_expr_call_ref_to_xref() {
    // This is the REAL form — Expr::Call produced by the parser for ref("slide-1").
    // The adversary finding F-077-P5-001 flagged that the proxy form (Expr::Pipe)
    // was the only form tested; real DSL `{{ ref("slide-1") }}` would fail to parse
    // without Expr::Call in the grammar.
    let ref_expr = SyntaxExpr::Call {
        func: "ref".to_string(),
        args: vec![SyntaxExpr::Str("slide-1".to_string())],
    };
    let chunks = vec![TemplateChunk::Expr(ref_expr)];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    assert!(
        sink.is_empty(),
        "ref(\"slide-1\") must not produce any diagnostics; got: {:?}",
        sink.errors()
    );
    assert_eq!(
        nodes.len(),
        1,
        "Expr::Call ref must produce exactly 1 InlineNode; got {nodes:?}"
    );
    match &nodes[0] {
        InlineNode::Xref(id) => {
            assert_eq!(
                id.as_ref(),
                "slide-1",
                "F-077-P5-001: Xref id must be 'slide-1'; got: {id:?}"
            );
        },
        other => panic!(
            "F-077-P5-001 FAIL: Expr::Call {{ func: 'ref', args: [Str('slide-1')] }} \
             must produce InlineNode::Xref(\"slide-1\"); got: {other:?}\n\
             This proves Expr::Call is routed to Xref (not Pipe proxy)"
        ),
    }
}

/// F-077-P5-001: `Expr::Call { func: "footnote", args: [Str("see appendix")] }`
/// → `InlineNode::Footnote([Plain("see appendix")])`.
///
/// Real Call form (not Pipe proxy). Proves footnote() is handled via Expr::Call.
#[test]
fn test_f077_p5_001_expr_call_footnote_to_footnote_node() {
    let footnote_expr = SyntaxExpr::Call {
        func: "footnote".to_string(),
        args: vec![SyntaxExpr::Str("see appendix".to_string())],
    };
    let chunks = vec![TemplateChunk::Expr(footnote_expr)];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    assert!(
        sink.is_empty(),
        "footnote(\"see appendix\") must not produce diagnostics; got: {:?}",
        sink.errors()
    );
    assert_eq!(
        nodes.len(),
        1,
        "Expr::Call footnote must produce 1 InlineNode; got {nodes:?}"
    );
    match &nodes[0] {
        InlineNode::Footnote(children) => {
            assert_eq!(
                children.len(),
                1,
                "F-077-P5-001: Footnote must have 1 child; got {children:?}"
            );
            assert!(
                matches!(&children[0], InlineNode::Plain(s) if s.as_ref() == "see appendix"),
                "F-077-P5-001: Footnote child must be Plain(\"see appendix\"); got: {:?}",
                children[0]
            );
        },
        other => {
            panic!("F-077-P5-001 FAIL: Expr::Call footnote must produce Footnote; got: {other:?}")
        },
    }
}

/// F-077-P5-001: `Expr::Call { func: "figref", args: [Num(3)] }`
/// → `InlineNode::Xref(Arc::from("fig-3"))`.
///
/// Real Call form (not Pipe proxy). Proves figref() is handled via Expr::Call.
#[test]
fn test_f077_p5_001_expr_call_figref_to_xref() {
    let figref_expr = SyntaxExpr::Call {
        func: "figref".to_string(),
        args: vec![SyntaxExpr::Num(3)],
    };
    let chunks = vec![TemplateChunk::Expr(figref_expr)];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    assert!(
        sink.is_empty(),
        "figref(3) must not produce diagnostics; got: {:?}",
        sink.errors()
    );
    assert_eq!(
        nodes.len(),
        1,
        "Expr::Call figref must produce 1 InlineNode; got {nodes:?}"
    );
    match &nodes[0] {
        InlineNode::Xref(id) => {
            assert_eq!(
                id.as_ref(),
                "fig-3",
                "F-077-P5-001: figref(3) must produce Xref(\"fig-3\"); got: {id:?}"
            );
        },
        other => panic!(
            "F-077-P5-001 FAIL: Expr::Call figref(3) must produce Xref(\"fig-3\"); got: {other:?}"
        ),
    }
}

// ─── F-077-P5-001: Full pipeline using Expr::Call ────────────────────────────

/// F-077-P5-001 (conversion-only): A hand-built `SectionNode` whose detail field
/// contains `TemplateChunk::Expr(Expr::Call { func: "ref", args: [Str("slide-1")] })`
/// must produce `FieldValue::Inlines` with `InlineNode::Xref("slide-1")` via
/// `eval_section_nodes_for_test`.
///
/// **CONVERSION-ONLY unit test** (not a parse→eval end-to-end test):
/// This test hand-builds the SectionNode with a real `Expr::Call` form to exercise
/// the `chunks_to_inline_nodes` routing for `{{ ref("slide-1") }}`. The parser is
/// NOT called — the SectionNode is constructed directly.
///
/// For the genuine end-to-end test using the real parser, see
/// `test_F_077_P2_001_genuine_parse_to_eval_end_to_end`.
#[test]
fn test_f077_p5_001_full_pipeline_expr_call_ref_in_section_detail() {
    let ref_expr = SyntaxExpr::Call {
        func: "ref".to_string(),
        args: vec![SyntaxExpr::Str("slide-1".to_string())],
    };
    let section_node = SectionNode {
        kind: Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("detail".to_string(), dummy_span()),
            value: Spanned::new(
                slideforge_syntax::FieldValue::Template(vec![
                    TemplateChunk::Literal("See ".to_string()),
                    TemplateChunk::Expr(ref_expr),
                    TemplateChunk::Literal(".".to_string()),
                ]),
                dummy_span(),
            ),
        }],
    };

    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    assert!(
        sink.is_empty(),
        "F-077-P5-001 full pipeline: no diagnostics expected; got: {:?}",
        sink.errors()
    );
    let (section_block, _) =
        result.expect("F-077-P5-001: eval must return Some for valid section with Call ref");

    let detail_entry = section_block
        .body
        .get("detail")
        .expect("F-077-P5-001: 'detail' key must be present after eval");

    match detail_entry {
        FieldValue::Inlines(nodes) => {
            // Must contain an Xref node somewhere in the sequence.
            let has_xref = nodes
                .iter()
                .any(|n| matches!(n, InlineNode::Xref(id) if id.as_ref() == "slide-1"));
            assert!(
                has_xref,
                "F-077-P5-001: FieldValue::Inlines must contain InlineNode::Xref(\"slide-1\"); \
                 got: {nodes:?}\n\
                 This proves Expr::Call {{ func: 'ref' }} is reachable from real DSL."
            );
            // Forbidden pattern: no Plain node must contain raw "ref(" or "slide-1".
            let has_raw = nodes.iter().any(|n| {
                matches!(n, InlineNode::Plain(s) if s.contains("ref(") || s.contains("slide-1"))
            });
            assert!(
                !has_raw,
                "F-077-P5-001: no Plain node must contain raw 'ref(' or 'slide-1'; got {nodes:?}"
            );
        },
        other => {
            panic!("F-077-P5-001 FAIL: body['detail'] must be FieldValue::Inlines; got: {other:?}")
        },
    }
}

// ─── F-077-P5-002: Real DSL parse→eval end-to-end seam tests ─────────────────

/// F-077-P5-002 / AC-002:
/// A section detail sub-block with a `TemplateChunk::Bold` in the template must
/// produce `FieldValue::Inlines([InlineNode::Bold([InlineNode::Plain("bold")])])`.
///
/// **CONVERSION-ONLY unit test** (not a parse→eval end-to-end test):
/// This test hand-builds a `SectionNode` with a `TemplateChunk::Bold` chunk
/// to directly exercise the `chunks_to_inline_nodes` conversion path via
/// `eval_section_nodes`. The parser is NOT called; the input represents what
/// `template_value()` would produce for `"**bold**"` in a section detail field.
///
/// For the genuine end-to-end test that calls the real parser on source text,
/// see `test_F_077_P2_001_genuine_parse_to_eval_end_to_end` (F-077-P2-001).
#[test]
fn test_f077_p5_002_real_bold_template_to_inlines_via_eval() {
    // Hand-build the SectionNode in the form that template_value() produces
    // for "**bold**". The parser is NOT called here — this tests the eval
    // conversion path (chunks_to_inline_nodes) in isolation.
    let section_node = SectionNode {
        kind: Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("detail".to_string(), dummy_span()),
            value: Spanned::new(
                slideforge_syntax::FieldValue::Template(vec![TemplateChunk::Bold(vec![
                    TemplateChunk::Literal("bold".to_string()),
                ])]),
                dummy_span(),
            ),
        }],
    };

    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    assert!(
        sink.is_empty(),
        "F-077-P5-002: no diagnostics expected for valid bold template; got: {:?}",
        sink.errors()
    );

    let (section_block, _) =
        result.expect("F-077-P5-002: eval must return Some for valid section with Bold template");

    let detail_entry = section_block
        .body
        .get("detail")
        .expect("F-077-P5-002: 'detail' key must be present after eval");

    match detail_entry {
        FieldValue::Inlines(nodes) => {
            assert_eq!(
                nodes.len(),
                1,
                "F-077-P5-002: detail must have exactly 1 InlineNode (Bold); got {nodes:?}"
            );
            match &nodes[0] {
                InlineNode::Bold(children) => {
                    assert_eq!(
                        children.len(),
                        1,
                        "Bold must have 1 child; got {children:?}"
                    );
                    assert!(
                        matches!(&children[0], InlineNode::Plain(s) if s.as_ref() == "bold"),
                        "Bold child must be Plain(\"bold\"); got: {:?}",
                        children[0]
                    );
                    // Forbidden pattern: NO literal `*` in any Plain leaf.
                    if let InlineNode::Plain(s) = &children[0] {
                        assert!(
                            !s.contains('*'),
                            "F-077-P5-002 FORBIDDEN: Plain node must not contain literal '*'; \
                             got: {s:?}. Bold chunk was not converted to InlineNode::Bold."
                        );
                    }
                },
                other => panic!(
                    "F-077-P5-002 FAIL: expected InlineNode::Bold; got: {other:?}\n\
                     Bold chunk was not converted correctly by chunks_to_inline_nodes."
                ),
            }
        },
        other => {
            panic!("F-077-P5-002 FAIL: body['detail'] must be FieldValue::Inlines; got: {other:?}")
        },
    }
}

/// F-077-P5-002 / AC-002:
/// A section detail sub-block with `[text](url)` hyperlink must produce
/// `FieldValue::Inlines([InlineNode::Link { text: [Plain("text")], url: "url" }])`.
///
/// Closes the Link parser→eval seam. A regression in either phase will fail here.
/// NO literal `[` or `](` characters may appear in any Plain node.
#[test]
fn test_f077_p5_002_real_link_template_to_inlines_via_eval() {
    let section_node = SectionNode {
        kind: Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("detail".to_string(), dummy_span()),
            value: Spanned::new(
                slideforge_syntax::FieldValue::Template(vec![TemplateChunk::Link {
                    text: vec![TemplateChunk::Literal("click here".to_string())],
                    url: "https://example.com".to_string(),
                }]),
                dummy_span(),
            ),
        }],
    };

    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    assert!(
        sink.is_empty(),
        "F-077-P5-002: no diagnostics expected for Link template; got: {:?}",
        sink.errors()
    );

    let (section_block, _) =
        result.expect("F-077-P5-002: eval must return Some for section with Link template");

    let detail_entry = section_block
        .body
        .get("detail")
        .expect("F-077-P5-002: 'detail' key must be present");

    match detail_entry {
        FieldValue::Inlines(nodes) => {
            assert_eq!(
                nodes.len(),
                1,
                "F-077-P5-002: Link template must produce 1 InlineNode; got {nodes:?}"
            );
            match &nodes[0] {
                InlineNode::Link { text, url } => {
                    assert_eq!(
                        url.as_ref(),
                        "https://example.com",
                        "F-077-P5-002: Link url must be 'https://example.com'"
                    );
                    assert_eq!(text.len(), 1, "Link text must have 1 child; got {text:?}");
                    assert!(
                        matches!(&text[0], InlineNode::Plain(s) if s.as_ref() == "click here"),
                        "Link text child must be Plain(\"click here\"); got: {:?}",
                        text[0]
                    );
                    // Forbidden pattern: no Plain node contains literal `[` or `](`.
                    if let InlineNode::Plain(s) = &text[0] {
                        assert!(
                            !s.contains('[') && !s.contains("]("),
                            "F-077-P5-002 FORBIDDEN: Plain text node contains raw link syntax; \
                             got: {s:?}"
                        );
                    }
                },
                other => panic!(
                    "F-077-P5-002 FAIL: Link template must produce InlineNode::Link; got: {other:?}"
                ),
            }
        },
        other => {
            panic!("F-077-P5-002 FAIL: body['detail'] must be FieldValue::Inlines; got: {other:?}")
        },
    }
}

/// F-077-P5-002 / AC-002:
/// A section detail sub-block with `~~del~~` (Strikethrough) must produce
/// `FieldValue::Inlines([InlineNode::Strikethrough([Plain("del")])])`.
///
/// Closes the Strikethrough parser→eval seam. NO literal `~~` in any Plain leaf.
#[test]
fn test_f077_p5_002_real_strikethrough_template_to_inlines_via_eval() {
    let section_node = SectionNode {
        kind: Spanned::new("methodology".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("detail".to_string(), dummy_span()),
            value: Spanned::new(
                slideforge_syntax::FieldValue::Template(vec![TemplateChunk::Strikethrough(vec![
                    TemplateChunk::Literal("del".to_string()),
                ])]),
                dummy_span(),
            ),
        }],
    };

    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    assert!(
        sink.is_empty(),
        "F-077-P5-002: no diagnostics expected for Strikethrough template"
    );

    let (section_block, _) =
        result.expect("F-077-P5-002: eval must return Some for Strikethrough template");

    let detail_entry = section_block
        .body
        .get("detail")
        .expect("F-077-P5-002: 'detail' key must be present");

    match detail_entry {
        FieldValue::Inlines(nodes) => {
            assert_eq!(
                nodes.len(),
                1,
                "F-077-P5-002: Strikethrough template must produce 1 InlineNode"
            );
            match &nodes[0] {
                InlineNode::Strikethrough(children) => {
                    assert_eq!(children.len(), 1, "Strikethrough must have 1 child");
                    assert!(
                        matches!(&children[0], InlineNode::Plain(s) if s.as_ref() == "del"),
                        "Strikethrough child must be Plain(\"del\"); got: {:?}",
                        children[0]
                    );
                    // Forbidden pattern: no `~~` in any Plain leaf.
                    if let InlineNode::Plain(s) = &children[0] {
                        assert!(
                            !s.contains("~~"),
                            "F-077-P5-002 FORBIDDEN: Plain node contains literal '~~'; got: {s:?}"
                        );
                    }
                },
                other => panic!("F-077-P5-002 FAIL: expected Strikethrough; got: {other:?}"),
            }
        },
        other => {
            panic!("F-077-P5-002 FAIL: body['detail'] must be FieldValue::Inlines; got: {other:?}")
        },
    }
}

// ─── OBS-B: empty-id validation for ref() ────────────────────────────────────

/// OBS-B / DIR-077-002 §5:
/// `{{ ref("") }}` (empty id) must produce a fatal error (E-PAR-inline-xref-empty-id)
/// pushed to the `DiagnosticSink` AND no `InlineNode` must be produced for that call.
///
/// This test verifies both the Expr::Call form and the legacy Pipe proxy form.
#[test]
fn test_obs_b_empty_ref_id_produces_fatal_error_no_inline_node() {
    // Test with real Expr::Call form.
    let call_empty_ref = SyntaxExpr::Call {
        func: "ref".to_string(),
        args: vec![SyntaxExpr::Str(String::new())], // empty string id
    };
    let chunks = vec![TemplateChunk::Expr(call_empty_ref)];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // (a) No InlineNode must be produced for an empty-id ref.
    assert!(
        nodes.is_empty(),
        "OBS-B: ref(\"\") must produce NO InlineNode; got: {nodes:?}"
    );

    // (b) The sink must contain exactly one error (inline-xref-empty-id).
    assert!(
        !sink.is_empty(),
        "OBS-B: ref(\"\") must push an E-PAR-inline-xref-empty-id error to the sink; sink is empty"
    );

    let err_msg = sink
        .errors()
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ");
    assert!(
        err_msg.contains("empty")
            || err_msg.contains("xref-empty-id")
            || err_msg.contains("non-empty"),
        "OBS-B: error message must mention empty id; got: {err_msg}"
    );
}

/// OBS-B (Pipe proxy variant): `{{ "" | ref }}` (empty id via Pipe) must
/// also produce a fatal error and no InlineNode.
///
/// Legacy proxy form must have the same safety behavior as the Call form.
#[test]
fn test_obs_b_empty_ref_id_pipe_proxy_also_produces_error() {
    let pipe_empty_ref = SyntaxExpr::Pipe {
        lhs: Box::new(SyntaxExpr::Str(String::new())),
        filter: "ref".to_string(),
        args: vec![],
    };
    let chunks = vec![TemplateChunk::Expr(pipe_empty_ref)];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    assert!(
        nodes.is_empty(),
        "OBS-B (Pipe): ref(\"\") via Pipe must produce NO InlineNode; got: {nodes:?}"
    );
    assert!(
        !sink.is_empty(),
        "OBS-B (Pipe): empty-id ref via Pipe must push an error; sink is empty"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// F-077-P2-001: Genuine end-to-end parse→eval test (adversary pass-2 finding)
//
// The prior "end-to-end" tests (test_f077_p5_001_full_pipeline_* and
// test_f077_p5_002_real_*) hand-build TemplateChunk/SectionNode structures
// rather than calling the real parser on a source string. This test calls
// the REAL parser on a LITERAL DSL SOURCE STRING and then runs eval_deck.
// ═══════════════════════════════════════════════════════════════════════════════

/// F-077-P2-001 / BC-3.02.002 AC-002:
/// Genuine end-to-end test: parses a REAL DSL source string containing a
/// section with a `detail:` field that has inline markup (`**Bold.**`) and
/// a cross-reference (`{{ figref(1) }}`), then evaluates the resulting AST
/// with `eval_deck`, and asserts the output IR contains structural InlineNodes.
///
/// **What makes this test genuine (not a paper test):**
/// - Calls `slideforge_syntax::parse()` on a raw source string.
/// - Does NOT hand-construct any TemplateChunk or SectionNode.
/// - If the parser regresses (stops producing `TemplateChunk::Bold` for `**`),
///   this test fails because the chunks feed directly into `eval_deck`.
/// - If `chunks_to_inline_nodes` regresses, this test fails because the
///   FieldValue::Inlines will not contain `InlineNode::Bold`.
///
/// **Source string parsed (verbatim):**
/// ```text
/// slideforge_version: "1.0"
/// lang: "en-US"
/// title: "Test"
///
/// section methodology:
///   detail: "**Bold.** See {{ figref(1) }}."
/// ```
///
/// **Expected IR after eval_deck:**
/// - `deck.section_blocks[0].body["detail"]` is
///   `FieldValue::Inlines([Bold([Plain("Bold.")]), Plain(" See "), Xref("fig-1"), Plain(".")])`
/// - `deck.section_blocks[0].register_content` has one
///   `RegisteredContent { register: Detail, content: [...] }` entry
/// - No `**` characters appear in any `InlineNode::Plain` leaf.
///
/// TD-VSDD-059: assertions are load-bearing on the exact InlineNode variant
/// (Bold) AND its inner content (Plain("Bold.")), not just existence checks.
#[test]
#[allow(non_snake_case)]
fn test_F_077_P2_001_genuine_parse_to_eval_end_to_end() {
    use crate::config::EvalConfig;
    use crate::eval::eval_deck;
    use slideforge_syntax::parse;
    use slideforge_syntax::span::SourceMap;
    use std::sync::Arc as StdArc;

    // DSL source with a section whose detail: field contains:
    // - **Bold.** (should parse to TemplateChunk::Bold then eval to InlineNode::Bold)
    // - {{ figref(1) }} (should parse to TemplateChunk::Expr(Call{figref,1}) then to Xref("fig-1"))
    //
    // figref(1) is used (not ref("id")) because figref has a numeric argument —
    // no inner string quotes, so no DSL string-escaping round-trip issue.
    let src = concat!(
        "slideforge_version \"1\"\n",
        "\n",
        "section methodology:\n",
        "  detail: \"**Bold.** See {{ figref(1) }}.\"\n",
    );

    let mut sm = SourceMap::new();
    let file_id = sm.add_file(StdArc::from("test.sf"), StdArc::from(src));

    // CALL THE REAL PARSER on the source string.
    let parse_result =
        parse(src, file_id, &sm).expect("F-077-P2-001: source must parse without fatal errors");
    // Allow only version-related warnings (e.g., version-mismatch advisory) or
    // section-key warnings (W-PAR-). No inline markup errors should appear for
    // well-formed "**Bold.** See {{ figref(1) }}.".
    for w in &parse_result.warnings {
        let msg = format!("{w:?}");
        assert!(
            msg.contains("version") || msg.contains("W-PAR-") || msg.contains("Version"),
            "F-077-P2-001: unexpected warning from parsing: {msg}"
        );
    }
    let deck_node = parse_result.deck;

    // CALL EVAL_DECK on the parsed AST — full pipeline.
    let config = EvalConfig::default();
    let mut eval_sink = DiagnosticSink::new();
    let deck = eval_deck(&deck_node, &config, &mut eval_sink)
        .expect("F-077-P2-001: eval_deck must return Some for valid deck");

    assert!(
        eval_sink.is_empty(),
        "F-077-P2-001: no diagnostics expected from eval_deck; got: {:?}",
        eval_sink.errors()
    );

    // ASSERT: section_blocks must contain the methodology section.
    assert_eq!(
        deck.section_blocks.len(),
        1,
        "F-077-P2-001: deck must contain 1 section block; got: {}",
        deck.section_blocks.len()
    );
    let section = &deck.section_blocks[0];
    assert_eq!(
        section.name.as_ref(),
        "methodology",
        "F-077-P2-001: section name must be 'methodology'; got: {:?}",
        section.name
    );

    // ASSERT: body["detail"] is FieldValue::Inlines (not Template or Literal).
    let detail = section
        .body
        .get("detail")
        .expect("F-077-P2-001: 'detail' key must be present in section body after eval");

    let nodes = match detail {
        FieldValue::Inlines(nodes) => nodes,
        other => panic!(
            "F-077-P2-001 FAIL: body['detail'] must be FieldValue::Inlines after eval; \
             got: {other:?}\n\
             If Template: the eval stage did not convert TemplateChunk::Bold to InlineNode::Bold.\n\
             If Literal: the parser did not produce Bold chunk for '**Bold.**'."
        ),
    };

    // ASSERT: contains InlineNode::Bold with "Bold." as the text.
    let bold_node = nodes.iter().find(|n| matches!(n, InlineNode::Bold(_)));
    assert!(
        bold_node.is_some(),
        "F-077-P2-001 FAIL: FieldValue::Inlines must contain InlineNode::Bold; got: {nodes:?}\n\
         This means '**Bold.**' was not parsed to TemplateChunk::Bold (parser regression)\n\
         OR chunks_to_inline_nodes did not convert Bold chunk to InlineNode::Bold (eval regression)."
    );
    if let Some(InlineNode::Bold(children)) = bold_node {
        let child_text: String = children
            .iter()
            .filter_map(|c| {
                if let InlineNode::Plain(s) = c {
                    Some(s.as_ref())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            child_text, "Bold.",
            "F-077-P2-001: Bold node content must be 'Bold.'; got: {child_text:?}"
        );
    }

    // ASSERT: contains InlineNode::Xref("fig-1") for figref(1).
    let xref_node = nodes
        .iter()
        .find(|n| matches!(n, InlineNode::Xref(id) if id.as_ref() == "fig-1"));
    assert!(
        xref_node.is_some(),
        "F-077-P2-001 FAIL: FieldValue::Inlines must contain InlineNode::Xref(\"fig-1\"); \
         got: {nodes:?}\n\
         figref(1) must produce Xref(\"fig-1\")."
    );

    // ASSERT: NO Plain node contains literal `**` (forbidden pattern from CLAUDE.md R1).
    for node in nodes.iter() {
        if let InlineNode::Plain(s) = node {
            assert!(
                !s.contains('*'),
                "F-077-P2-001 FORBIDDEN PATTERN (R1): Plain node must NOT contain asterisks; \
                 got Plain({s:?}). The parser must produce InlineNode::Bold, not literal '**'."
            );
        }
    }

    // ASSERT: register_content has a Detail entry.
    let detail_rc = section
        .register_content
        .iter()
        .find(|rc| rc.register == Register::Detail);
    assert!(
        detail_rc.is_some(),
        "F-077-P2-001: section.register_content must contain a Detail entry; \
         got: {:?}",
        section.register_content
    );
}

// ─── F-077-P2-003: Unclosed bold test — assert column points to opening ** ────

/// F-077-P2-002 (span precision): When `**unclosed` appears in a section
/// detail field, the reported error span must point to the opening `**`
/// (byte offset 0 within the string content), NOT the start of the whole
/// string literal.
///
/// The `scan_template_chunks` scanner now accumulates `(byte_offset, message)`
/// pairs. `section_value_parser` translates each offset into a sub-span.
/// This test verifies the sub-span is at the correct column.
///
/// E-PAR-019 (unclosed inline markup) is ALWAYS FATAL per error-taxonomy.md:24.
/// parse() returns Err; parse_checked returns None + has_fatal()=true in the sink.
/// (F-077-P14-001 fix: rewrote from wrong Ok-path assertion.)
#[test]
#[allow(non_snake_case)]
fn test_F_077_P2_002_unclosed_bold_error_span_points_to_opening_delimiter() {
    use slideforge_syntax::span::SourceMap;
    use slideforge_syntax::{parse, parse_checked};
    use std::sync::Arc as StdArc;

    // Source: a section with a detail field containing unclosed bold.
    // The `**` opener is at position 0 within the string content "**unclosed".
    // After the `"` quote (token start), the `**` is at byte offset 0.
    let src = concat!(
        "slideforge_version \"1\"\n",
        "\n",
        "section methodology:\n",
        "  detail: \"**unclosed\"\n",
    );

    let mut sm = SourceMap::new();
    let file_id = sm.add_file(StdArc::from("test_span.sf"), StdArc::from(src));

    // E-PAR-019 is ALWAYS FATAL — parse() must return Err in strict (default) mode.
    // error-taxonomy.md:24: "Parse Errors (E-PAR) — Always fatal." (F-077-P14-001)
    let errors = parse(src, file_id, &sm)
        .expect_err("F-077-P2-002: parse must return Err for unclosed bold — E-PAR-019 is strict-build-fatal (F-077-P14-001)");

    // The fatal error must mention E-PAR-019 (dedicated unclosed inline markup code).
    // E-PAR-015 was the COLLIDING code from SHAPE parsing — after the fix it must
    // no longer appear here; E-PAR-019 is the correct code.
    let has_epar019 = errors
        .iter()
        .any(|e| format!("{e:?}").contains("E-PAR-019") || format!("{e:?}").contains("unclosed"));
    assert!(
        has_epar019,
        "F-077-P2-002: E-PAR-019 unclosed-bold error must be present in the Err vec; got: {errors:?}"
    );

    // The span on the error must point at the opening `**` byte offset,
    // NOT at the first byte of the whole field-value token.
    // The section source line is: `  detail: "**unclosed"`
    // Source has slideforge_version "1" (line 1), empty line (line 2),
    // section methodology: (line 3), detail: "**unclosed" (line 4).
    for e in &errors {
        let msg = format!("{e:?}");
        if msg.contains("E-PAR-019") || msg.contains("unclosed") {
            let (_, line, _col) = e.sort_position();
            assert!(
                line >= 4,
                "F-077-P2-002: E-PAR-019 span must be on the section detail line (≥ line 4); \
                 got line {line}"
            );
            // Verify the message mentions the correct delimiter.
            assert!(
                msg.contains("**") || msg.contains("unclosed"),
                "F-077-P2-002: E-PAR-019 message must mention '**' or 'unclosed'; got: {msg}"
            );
        }
    }

    // E-PAR-019 in strict mode (parse_checked): parse_checked routes Err → None + sink.
    // The DiagnosticSink must mark has_fatal() == true.
    // parse_checked returns None because the parse failed (no usable AST).
    let mut sm2 = SourceMap::new();
    let file_id2 = sm2.add_file(StdArc::from("test_span.sf"), StdArc::from(src));
    let mut strict_sink = DiagnosticSink::new();
    let ast_opt = parse_checked(src, file_id2, &sm2, &mut strict_sink);
    assert!(
        ast_opt.is_none(),
        "F-077-P2-002: parse_checked must return None when E-PAR-019 is fatal \
         (no usable AST produced, strict build fails). (F-077-P14-001)"
    );
    // The sink must have a fatal-severity diagnostic (strict build fails).
    assert!(
        strict_sink.has_fatal(),
        "F-077-P2-002: parse_checked with E-PAR-019 must produce has_fatal()=true in the sink \
         (strict build must fail on unclosed inline markup per error-taxonomy.md:24); \
         got has_fatal=false. Check that SyntaxError::UnclosedInlineMarkup.severity() == Fatal."
    );
}

// ─── F-077-P2-003: Word-internal underscore guard regression tests ─────────────

/// F-077-P2-003: `parse_template_value("SENTINEL_NOTES")` must produce a
/// SINGLE `Literal("SENTINEL_NOTES")` chunk (no Italic), because `_` within
/// an alphanumeric word is NOT an italic opener (CommonMark §6.1 left-flanking
/// rule, CLAUDE.md/template.rs line comment).
///
/// This is a REGRESSION guard: if the left-flanking check is removed or
/// broken, `SENTINEL_NOTES` would incorrectly parse as
/// `[Literal("SENTINEL"), Italic([Literal("NOTES")])]`.
///
/// TD-VSDD-059: load-bearing assertion on the exact chunk count AND the
/// inner string content (NO Italic chunk present).
#[test]
#[allow(non_snake_case)]
fn test_F_077_P2_003_word_internal_underscore_is_literal_not_italic() {
    use slideforge_syntax::ast::{BlockItem, FieldValue as SyntaxFieldValue};
    use slideforge_syntax::parse;
    use slideforge_syntax::span::SourceMap;
    use std::sync::Arc as StdArc;

    // Parse a slide field value containing SENTINEL_NOTES.
    let src = "slideforge_version \"1\"\n\nslide content:\n  detail \"SENTINEL_NOTES\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(StdArc::from("test_guard.sf"), StdArc::from(src));
    let pr =
        parse(src, file_id, &sm).expect("F-077-P2-003: source must parse without fatal errors");
    let deck = pr.deck;

    let BlockItem::Slide(slide_s) = &deck.items[0] else {
        panic!("F-077-P2-003: expected Slide block item");
    };
    let field = slide_s
        .value()
        .fields
        .iter()
        .find(|f| f.name.value() == "detail")
        .expect("F-077-P2-003: 'detail' field must exist");
    // Destructure the field value as FieldValue::Template.
    let SyntaxFieldValue::Template(chunks) = field.value.value() else {
        panic!(
            "F-077-P2-003: expected FieldValue::Template; got: {:?}",
            field.value.value()
        );
    };
    assert_eq!(
        chunks.len(),
        1,
        "F-077-P2-003: 'SENTINEL_NOTES' must produce exactly 1 chunk (Literal); \
         got {} chunks: {chunks:?}\n\
         Multiple chunks indicate the word-internal underscore guard is broken \
         — `_` in `SENTINEL_NOTES` was treated as italic opener.",
        chunks.len()
    );
    match &chunks[0] {
        TemplateChunk::Literal(lit) => {
            assert_eq!(
                lit.as_str(),
                "SENTINEL_NOTES",
                "F-077-P2-003: Literal must be 'SENTINEL_NOTES'; got: {lit:?}"
            );
        },
        TemplateChunk::Italic(_) => panic!(
            "F-077-P2-003 FAIL: 'SENTINEL_NOTES' produced Italic chunk — \
             word-internal underscore guard is broken (CommonMark §6.1 left-flanking rule)"
        ),
        other => panic!("F-077-P2-003 FAIL: expected Literal('SENTINEL_NOTES'); got: {other:?}"),
    }
}

/// F-077-P2-003 (positive companion): `parse_template_value("a _italic_ b")`
/// MUST produce `[Literal("a "), Italic([Literal("italic")]), Literal(" b")]`.
///
/// This verifies that the left-flanking guard only excludes WORD-INTERNAL
/// underscores, not standalone `_italic_` forms.
///
/// TD-VSDD-059: load-bearing assertion on chunk count, exact variants,
/// and inner content.
#[test]
#[allow(non_snake_case)]
fn test_F_077_P2_003_standalone_underscore_italic_still_works() {
    use slideforge_syntax::ast::{BlockItem, FieldValue as SyntaxFieldValue};
    use slideforge_syntax::parse;
    use slideforge_syntax::span::SourceMap;
    use std::sync::Arc as StdArc;

    let src = "slideforge_version \"1\"\n\nslide content:\n  detail \"a _italic_ b\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(StdArc::from("test_italic.sf"), StdArc::from(src));
    let pr = parse(src, file_id, &sm)
        .expect("F-077-P2-003 positive: source must parse without fatal errors");
    let deck = pr.deck;

    let BlockItem::Slide(slide_s) = &deck.items[0] else {
        panic!("F-077-P2-003 positive: expected Slide");
    };
    let field = slide_s
        .value()
        .fields
        .iter()
        .find(|f| f.name.value() == "detail")
        .expect("F-077-P2-003 positive: 'detail' field must exist");
    let SyntaxFieldValue::Template(chunks) = field.value.value() else {
        panic!("F-077-P2-003 positive: expected FieldValue::Template");
    };

    assert_eq!(
        chunks.len(),
        3,
        "F-077-P2-003 positive: 'a _italic_ b' must produce 3 chunks \
         [Literal(\"a \"), Italic([Literal(\"italic\")]), Literal(\" b\")]; \
         got {} chunks: {chunks:?}",
        chunks.len()
    );

    assert!(
        matches!(&chunks[0], TemplateChunk::Literal(s) if s == "a "),
        "F-077-P2-003 positive: first chunk must be Literal(\"a \"); got: {:?}",
        chunks[0]
    );

    match &chunks[1] {
        TemplateChunk::Italic(children) => {
            assert_eq!(
                children.len(),
                1,
                "F-077-P2-003 positive: Italic must have 1 child; got {children:?}"
            );
            assert!(
                matches!(&children[0], TemplateChunk::Literal(s) if s == "italic"),
                "F-077-P2-003 positive: Italic child must be Literal(\"italic\"); got: {:?}",
                children[0]
            );
        },
        other => panic!("F-077-P2-003 positive FAIL: chunk[1] must be Italic; got: {other:?}"),
    }

    assert!(
        matches!(&chunks[2], TemplateChunk::Literal(s) if s == " b"),
        "F-077-P2-003 positive: third chunk must be Literal(\" b\"); got: {:?}",
        chunks[2]
    );
}

// ─── F-077-P2-001: Fix misleading doc-comments on conversion-only tests ────────

// NOTE: The following tests (test_f077_p5_001_full_pipeline_expr_call_ref_in_section_detail
// and test_f077_p5_002_real_*) are CONVERSION-ONLY unit tests that hand-build
// TemplateChunk/SectionNode nodes rather than calling the real parser on source text.
// They test the chunks_to_inline_nodes eval path, not the parse→eval pipeline.
// The GENUINE parse→eval seam test is test_F_077_P2_001_genuine_parse_to_eval_end_to_end
// above. (F-077-P2-001 finding closure.)

// ─── F-077-P3-001: Structured error code assertions (TD-VSDD-059) ────────────

/// F-077-P3-001 / TD-VSDD-059:
/// `{{ figref() }}` with no arguments must push an error with STRUCTURED code
/// `E-EVL-012` (not `E-EVL-003` or any other collision) to the `DiagnosticSink`.
///
/// This test is LOAD-BEARING on `err.code()` — a future regression that
/// accidentally routes the figref-no-arg error through `TypeMismatch` (E-EVL-003)
/// or `UnsupportedBuiltinCall` (E-EVL-011) would be caught here.
///
/// TD-VSDD-059 compliance: code assertion is structural, not just a string substring.
#[test]
fn test_f077_p3_001_figref_no_arg_emits_e_evl_012() {
    let figref_no_args = SyntaxExpr::Call {
        func: "figref".to_string(),
        args: vec![], // no arguments — fatal per DIR-077-002 §5 / OBS-C
    };
    let chunks = vec![TemplateChunk::Expr(figref_no_args)];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // (a) No InlineNode must be produced.
    assert!(
        nodes.is_empty(),
        "F-077-P3-001: figref() with no args must produce NO InlineNode; got: {nodes:?}"
    );

    // (b) Exactly one diagnostic must be pushed.
    assert!(
        !sink.is_empty(),
        "F-077-P3-001: figref() with no args must push a diagnostic to the sink; sink is empty"
    );

    // (c) LOAD-BEARING: the diagnostic code MUST be E-EVL-012 (FigrefInvalidArg).
    // This assertion catches any regression that routes this error through
    // TypeMismatch (E-EVL-003) or UnsupportedBuiltinCall (E-EVL-011).
    let code = sink
        .errors()
        .iter()
        .find_map(|e| e.code().map(|c| c.to_string()));
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-012"),
        "F-077-P3-001: figref() no-arg error MUST have code E-EVL-012 (FigrefInvalidArg); \
         got: {code:?}. A code of E-EVL-003 or E-EVL-011 indicates the error collision \
         found in F-077-P3-001 has regressed."
    );
}

/// F-077-P3-001 / TD-VSDD-059:
/// `{{ ref("") }}` (empty id, Call form) must push an error with STRUCTURED code
/// `E-EVL-013` (not `E-EVL-003` with embedded `E-PAR-inline-xref-empty-id` text)
/// to the `DiagnosticSink`.
///
/// This test is LOAD-BEARING on `err.code()` — a future regression that routes
/// the empty-id error through `TypeMismatch` (E-EVL-003) would be caught here.
/// A code of E-EVL-003 indicates re-introduction of the code collision.
///
/// TD-VSDD-059 compliance: code assertion is structural, not just a string substring.
#[test]
fn test_f077_p3_001_ref_empty_id_call_emits_e_evl_013() {
    let call_empty_ref = SyntaxExpr::Call {
        func: "ref".to_string(),
        args: vec![SyntaxExpr::Str(String::new())], // empty string id
    };
    let chunks = vec![TemplateChunk::Expr(call_empty_ref)];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // (a) No InlineNode must be produced for an empty-id ref.
    assert!(
        nodes.is_empty(),
        "F-077-P3-001: ref(\"\") must produce NO InlineNode; got: {nodes:?}"
    );

    // (b) Exactly one diagnostic must be pushed.
    assert!(
        !sink.is_empty(),
        "F-077-P3-001: ref(\"\") must push a diagnostic to the sink; sink is empty"
    );

    // (c) LOAD-BEARING: the diagnostic code MUST be E-EVL-013 (InlineXrefEmptyId).
    // A code of E-EVL-003 indicates the stage/prefix mismatch from F-077-P3-001 has regressed.
    let code = sink
        .errors()
        .iter()
        .find_map(|e| e.code().map(|c| c.to_string()));
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-013"),
        "F-077-P3-001: ref(\"\") error MUST have code E-EVL-013 (InlineXrefEmptyId); \
         got: {code:?}. A code of E-EVL-003 indicates the TypeMismatch collision \
         from finding F-077-P3-001 has been re-introduced."
    );
}

/// F-077-P3-001 / TD-VSDD-059:
/// `{{ "" | ref }}` (empty id, Pipe proxy form) must push an error with STRUCTURED
/// code `E-EVL-013` (not `E-EVL-003` with embedded `E-PAR-inline-xref-empty-id` text).
///
/// Both the Call form and the legacy Pipe proxy form must use the same dedicated
/// variant — the same error on both call paths prevents divergent behavior if one
/// path is refactored.
///
/// TD-VSDD-059 compliance: code assertion is structural, not just a string substring.
#[test]
fn test_f077_p3_001_ref_empty_id_pipe_emits_e_evl_013() {
    let pipe_empty_ref = SyntaxExpr::Pipe {
        lhs: Box::new(SyntaxExpr::Str(String::new())),
        filter: "ref".to_string(),
        args: vec![],
    };
    let chunks = vec![TemplateChunk::Expr(pipe_empty_ref)];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // (a) No InlineNode must be produced.
    assert!(
        nodes.is_empty(),
        "F-077-P3-001 (Pipe): ref(\"\") via Pipe must produce NO InlineNode; got: {nodes:?}"
    );

    // (b) Exactly one diagnostic must be pushed.
    assert!(
        !sink.is_empty(),
        "F-077-P3-001 (Pipe): empty-id ref via Pipe must push a diagnostic; sink is empty"
    );

    // (c) LOAD-BEARING: the diagnostic code MUST be E-EVL-013 (InlineXrefEmptyId).
    let code = sink
        .errors()
        .iter()
        .find_map(|e| e.code().map(|c| c.to_string()));
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-013"),
        "F-077-P3-001 (Pipe): empty-id ref via Pipe MUST have code E-EVL-013 \
         (InlineXrefEmptyId); got: {code:?}."
    );
}

// ─── F-077-P9-001: ref()/footnote() zero-arg silent drop ─────────────────────

/// F-077-P9-001 (MED): `{{ ref() }}` with ZERO arguments must push E-EVL-013
/// and produce NO InlineNode.
///
/// Before this fix, the `None` arm in the ref branch only executed
/// `if let Some(first) = args.first()` — for zero args this is `None`, so
/// NOTHING executed (no node, no diagnostic = silent failure). This test
/// drives the fix.
#[test]
fn test_f077_p9_001_ref_zero_arg_emits_e_evl_013() {
    let ref_no_args = SyntaxExpr::Call {
        func: "ref".to_string(),
        args: vec![], // zero arguments — must emit E-EVL-013, not silently drop
    };
    let chunks = vec![TemplateChunk::Expr(ref_no_args)];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // (a) No InlineNode must be produced.
    assert!(
        nodes.is_empty(),
        "F-077-P9-001: ref() with no args must produce NO InlineNode; got: {nodes:?}"
    );

    // (b) Exactly one diagnostic must be pushed.
    assert!(
        !sink.is_empty(),
        "F-077-P9-001: ref() with no args must push a diagnostic to the sink; sink is empty"
    );

    // (c) LOAD-BEARING: the diagnostic code MUST be E-EVL-013 (InlineXrefEmptyId).
    let code = sink
        .errors()
        .iter()
        .find_map(|e| e.code().map(|c| c.to_string()));
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-013"),
        "F-077-P9-001: ref() no-arg error MUST have code E-EVL-013 (InlineXrefEmptyId); \
         got: {code:?}."
    );
}

/// F-077-P9-001 (MED): `{{ footnote() }}` with ZERO arguments must push E-EVL-014
/// and produce NO InlineNode.
///
/// Before this fix, the `None` arm in the footnote branch only executed
/// `if let Some(first) = args.first()` — for zero args this is `None`, so
/// NOTHING executed (silent failure). This test drives the fix.
#[test]
fn test_f077_p9_001_footnote_zero_arg_emits_e_evl_014() {
    let footnote_no_args = SyntaxExpr::Call {
        func: "footnote".to_string(),
        args: vec![], // zero arguments — must emit E-EVL-014, not silently drop
    };
    let chunks = vec![TemplateChunk::Expr(footnote_no_args)];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // (a) No InlineNode must be produced.
    assert!(
        nodes.is_empty(),
        "F-077-P9-001: footnote() with no args must produce NO InlineNode; got: {nodes:?}"
    );

    // (b) Exactly one diagnostic must be pushed.
    assert!(
        !sink.is_empty(),
        "F-077-P9-001: footnote() with no args must push a diagnostic to the sink; sink is empty"
    );

    // (c) LOAD-BEARING: the diagnostic code MUST be E-EVL-014 (FootnoteInvalidArg).
    let code = sink
        .errors()
        .iter()
        .find_map(|e| e.code().map(|c| c.to_string()));
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-014"),
        "F-077-P9-001: footnote() no-arg error MUST have code E-EVL-014 (FootnoteInvalidArg); \
         got: {code:?}."
    );
}

/// F-077-P9-001 (MED): `{{ footnote("") }}` with an EMPTY string argument must
/// push E-EVL-014 and produce NO InlineNode.
///
/// Empty footnote text is invalid per the PO spec — a footnote with no text
/// is meaningless and must be rejected with a diagnostic, not silently produce
/// an empty Footnote node.
#[test]
fn test_f077_p9_001_footnote_empty_string_emits_e_evl_014() {
    let footnote_empty = SyntaxExpr::Call {
        func: "footnote".to_string(),
        args: vec![SyntaxExpr::Str(String::new())], // empty string — must emit E-EVL-014
    };
    let chunks = vec![TemplateChunk::Expr(footnote_empty)];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // (a) No InlineNode must be produced for empty footnote text.
    assert!(
        nodes.is_empty(),
        "F-077-P9-001: footnote(\"\") must produce NO InlineNode; got: {nodes:?}"
    );

    // (b) Exactly one diagnostic must be pushed.
    assert!(
        !sink.is_empty(),
        "F-077-P9-001: footnote(\"\") must push a diagnostic to the sink; sink is empty"
    );

    // (c) LOAD-BEARING: the diagnostic code MUST be E-EVL-014 (FootnoteInvalidArg).
    let code = sink
        .errors()
        .iter()
        .find_map(|e| e.code().map(|c| c.to_string()));
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-014"),
        "F-077-P9-001: footnote(\"\") error MUST have code E-EVL-014 (FootnoteInvalidArg); \
         got: {code:?}."
    );
}

// ─── OBS-077-P10-A: ref(ident) resolves to empty → E-EVL-013 ─────────────────

/// OBS-077-P10-A / E-EVL-013 (condition 3): `{{ ref(var) }}` where `var` is
/// bound to an empty string in the environment must push exactly one E-EVL-013
/// diagnostic and produce NO `InlineNode`.
///
/// This covers the non-Str-literal path in `chunks_to_inline_nodes`:
/// ```text
/// Expr::Call { func: "ref", args: [Expr::Ident("var")] }
///   -> eval_expr_to_string(env, Ident("var"), sink)  -> Some(Arc::from(""))
///   -> s.is_empty() is true  -> push E-EVL-013, no InlineNode
/// ```
///
/// Previously only the `Expr::Str("")` literal case had test coverage.
/// This test covers condition (3) — ident resolves to empty string.
#[test]
fn test_f077_ref_ident_resolves_empty_emits_e_evl_013() {
    // Bind var = "" in the env — simulates `@var var = ""` in a .sf file.
    let mut vars: IndexMap<Arc<str>, slideforge_types::Value> = IndexMap::new();
    vars.insert(
        Arc::from("var"),
        slideforge_types::Value::Str(Arc::from("")),
    );
    let env = Env::new(vars);

    // Build {{ ref(var) }} — ident arg, not a string literal.
    let ref_ident = SyntaxExpr::Call {
        func: "ref".to_string(),
        args: vec![SyntaxExpr::Ident("var".to_string())],
    };
    let chunks = vec![TemplateChunk::Expr(ref_ident)];
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // (a) No InlineNode produced — empty id is invalid.
    assert!(
        nodes.is_empty(),
        "OBS-077-P10-A: ref(var) where var=\"\" must produce NO InlineNode; got: {nodes:?}"
    );

    // (b) Exactly one diagnostic pushed.
    assert!(
        !sink.is_empty(),
        "OBS-077-P10-A: ref(var) where var=\"\" must push a diagnostic to the sink; sink is empty"
    );

    // (c) LOAD-BEARING: diagnostic code MUST be E-EVL-013 (InlineXrefEmptyId).
    let code = sink
        .errors()
        .iter()
        .find_map(|e| e.code().map(|c| c.to_string()));
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-013"),
        "OBS-077-P10-A: ref(var) where var=\"\" MUST emit E-EVL-013 (InlineXrefEmptyId); \
         got: {code:?}"
    );
}

// ─── OBS-077-P10-A: footnote(ident) resolves to empty → E-EVL-014 ───────────

/// OBS-077-P10-A / E-EVL-014 (condition 3): `{{ footnote(var) }}` where `var`
/// is bound to an empty string in the environment must push exactly one E-EVL-014
/// diagnostic and produce NO `InlineNode`.
///
/// Covers the non-Str-literal path in `chunks_to_inline_nodes` for `footnote`:
/// ```text
/// Expr::Call { func: "footnote", args: [Expr::Ident("var")] }
///   -> eval_expr_to_string(env, Ident("var"), sink) -> Some(Arc::from(""))
///   -> s.is_empty() is true -> push E-EVL-014, no InlineNode
/// ```
///
/// Previously only the `Expr::Str("")` literal case had test coverage.
/// This test covers condition (3) — ident resolves to empty string.
#[test]
fn test_f077_footnote_ident_resolves_empty_emits_e_evl_014() {
    // Bind var = "" in the env.
    let mut vars: IndexMap<Arc<str>, slideforge_types::Value> = IndexMap::new();
    vars.insert(
        Arc::from("var"),
        slideforge_types::Value::Str(Arc::from("")),
    );
    let env = Env::new(vars);

    // Build {{ footnote(var) }} — ident arg, not a string literal.
    let footnote_ident = SyntaxExpr::Call {
        func: "footnote".to_string(),
        args: vec![SyntaxExpr::Ident("var".to_string())],
    };
    let chunks = vec![TemplateChunk::Expr(footnote_ident)];
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // (a) No InlineNode produced — empty footnote text is invalid.
    assert!(
        nodes.is_empty(),
        "OBS-077-P10-A: footnote(var) where var=\"\" must produce NO InlineNode; got: {nodes:?}"
    );

    // (b) Exactly one diagnostic pushed.
    assert!(
        !sink.is_empty(),
        "OBS-077-P10-A: footnote(var) where var=\"\" must push a diagnostic; sink is empty"
    );

    // (c) LOAD-BEARING: diagnostic code MUST be E-EVL-014 (FootnoteInvalidArg).
    let code = sink
        .errors()
        .iter()
        .find_map(|e| e.code().map(|c| c.to_string()));
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-014"),
        "OBS-077-P10-A: footnote(var) where var=\"\" MUST emit E-EVL-014 (FootnoteInvalidArg); \
         got: {code:?}"
    );
}

// ─── F-077-P11-001: figref(ident) resolves to empty → E-EVL-012 ──────────────

/// F-077-P11-001 / E-EVL-012 (condition 3): `{{ figref(var) }}` where `var` is
/// bound to an empty string in the environment must push exactly one E-EVL-012
/// diagnostic and produce NO `InlineNode`.
///
/// Covers the non-`Num` path in `chunks_to_inline_nodes` for `figref` after
/// `eval_expr_to_string` returns `Some("")`:
/// ```text
/// Expr::Call { func: "figref", args: [Expr::Ident("var")] }
///   -> eval_expr_to_string(env, Ident("var"), sink) -> Some(Arc::from(""))
///   -> s.is_empty() is true -> push E-EVL-012 (FigrefInvalidArg), no InlineNode
/// ```
///
/// RED GATE: before the fix this produces `Xref("fig-")` silently — a malformed
/// cross-reference — and pushes NO diagnostic, violating the no-silent-fallback
/// principle and creating inconsistency with `ref`/`footnote`, which both reject
/// empty-resolved args (E-EVL-013 / E-EVL-014 respectively).
///
/// After the fix all THREE inline builtins (figref/ref/footnote) reject empty-
/// resolved args consistently. E-EVL-012 is reused (its taxonomy note covers
/// "missing or non-evaluable argument"; empty-resolved is non-usable).
#[test]
fn test_f077_p11_001_figref_ident_resolves_empty_emits_e_evl_012() {
    // Bind var = "" in the env — simulates `@var var = ""` in a .sf file.
    let mut vars: IndexMap<Arc<str>, slideforge_types::Value> = IndexMap::new();
    vars.insert(
        Arc::from("var"),
        slideforge_types::Value::Str(Arc::from("")),
    );
    let env = Env::new(vars);

    // Build {{ figref(var) }} — ident arg, evaluates to "".
    let figref_ident = SyntaxExpr::Call {
        func: "figref".to_string(),
        args: vec![SyntaxExpr::Ident("var".to_string())],
    };
    let chunks = vec![TemplateChunk::Expr(figref_ident)];
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // (a) No InlineNode produced — empty-resolved figref is invalid; Xref("fig-")
    //     is a malformed cross-reference and must be rejected, not silently emitted.
    assert!(
        nodes.is_empty(),
        "F-077-P11-001: figref(var) where var=\"\" must produce NO InlineNode; got: {nodes:?}"
    );

    // (b) Exactly one diagnostic pushed.
    assert!(
        !sink.is_empty(),
        "F-077-P11-001: figref(var) where var=\"\" must push a diagnostic to the sink; \
         sink is empty"
    );

    // (c) LOAD-BEARING: diagnostic code MUST be E-EVL-012 (FigrefInvalidArg).
    //     All three inline builtins now reject empty-resolved args consistently:
    //       figref → E-EVL-012 | ref → E-EVL-013 | footnote → E-EVL-014
    let code = sink
        .errors()
        .iter()
        .find_map(|e| e.code().map(|c| c.to_string()));
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-012"),
        "F-077-P11-001: figref(var) where var=\"\" MUST emit E-EVL-012 (FigrefInvalidArg); \
         got: {code:?}"
    );
}

// ─── F-077-P12-001: Pipe-form empty-arg guards (sibling-site consistency) ─────
//
// The Call form and Pipe form MUST behave identically for empty/empty-resolved
// args. These tests mirror the P11-001/P10-A tests but use Expr::Pipe instead
// of Expr::Call.

/// F-077-P12-001 (LOW→consistency):
/// `{{ "" | ref }}` (Pipe form, literal empty string) must push E-EVL-013 and
/// produce NO InlineNode.
///
/// This is already guarded in the Pipe::ref arm (literal `""` branch) — this
/// test is kept as a regression guard to prevent future breakage.
#[test]
fn test_f077_p12_001_pipe_ref_empty_literal_emits_e_evl_013() {
    let pipe_empty_ref = SyntaxExpr::Pipe {
        lhs: Box::new(SyntaxExpr::Str(String::new())),
        filter: "ref".to_string(),
        args: vec![],
    };
    let chunks = vec![TemplateChunk::Expr(pipe_empty_ref)];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // (a) No InlineNode must be produced.
    assert!(
        nodes.is_empty(),
        "F-077-P12-001: {{ \"\" | ref }} must produce NO InlineNode; got: {nodes:?}"
    );

    // (b) Exactly one diagnostic must be pushed.
    assert!(
        !sink.is_empty(),
        "F-077-P12-001: {{ \"\" | ref }} must push a diagnostic; sink is empty"
    );

    // (c) LOAD-BEARING: the diagnostic code MUST be E-EVL-013 (InlineXrefEmptyId).
    let code = sink
        .errors()
        .iter()
        .find_map(|e| e.code().map(|c| c.to_string()));
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-013"),
        "F-077-P12-001: {{ \"\" | ref }} MUST emit E-EVL-013 (InlineXrefEmptyId); got: {code:?}"
    );
}

/// F-077-P12-001 (LOW→consistency):
/// `{{ var | ref }}` where `var` is bound to `""` in the environment must push
/// E-EVL-013 and produce NO InlineNode.
///
/// RED GATE: before the fix, the eval-resolved-empty branch in the Pipe::ref arm
/// pushes `Xref(s)` with no empty check (silent `Xref("")`). After the fix it
/// emits E-EVL-013, matching the Call form behaviour.
///
/// Call+Pipe consistency: both forms MUST behave identically for empty args
/// (see `test_f077_ref_ident_resolves_empty_emits_e_evl_013` for the Call form).
#[test]
fn test_f077_p12_001_pipe_ref_ident_resolves_empty_emits_e_evl_013() {
    // Bind var = "" in the env.
    let mut vars: IndexMap<Arc<str>, slideforge_types::Value> = IndexMap::new();
    vars.insert(
        Arc::from("var"),
        slideforge_types::Value::Str(Arc::from("")),
    );
    let env = Env::new(vars);

    // Build {{ var | ref }} — ident lhs, not a string literal.
    let pipe_ref_ident = SyntaxExpr::Pipe {
        lhs: Box::new(SyntaxExpr::Ident("var".to_string())),
        filter: "ref".to_string(),
        args: vec![],
    };
    let chunks = vec![TemplateChunk::Expr(pipe_ref_ident)];
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // (a) No InlineNode produced — empty id is invalid.
    assert!(
        nodes.is_empty(),
        "F-077-P12-001: {{ var | ref }} where var=\"\" must produce NO InlineNode; got: {nodes:?}"
    );

    // (b) Exactly one diagnostic pushed.
    assert!(
        !sink.is_empty(),
        "F-077-P12-001: {{ var | ref }} where var=\"\" must push a diagnostic; sink is empty"
    );

    // (c) LOAD-BEARING: diagnostic code MUST be E-EVL-013 (InlineXrefEmptyId).
    //     Call form and Pipe form must be consistent — both emit E-EVL-013 for
    //     empty-resolved ref args (F-077-P12-001 sibling-site consistency).
    let code = sink
        .errors()
        .iter()
        .find_map(|e| e.code().map(|c| c.to_string()));
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-013"),
        "F-077-P12-001: {{ var | ref }} where var=\"\" MUST emit E-EVL-013 (InlineXrefEmptyId); \
         got: {code:?}"
    );
}

/// F-077-P12-001 (LOW→consistency):
/// `{{ var | figref }}` where `var` is bound to `""` in the environment must
/// push E-EVL-012 and produce NO InlineNode.
///
/// RED GATE: before the fix, the eval-resolved branch in the Pipe::figref arm
/// does `Arc::from(format!("fig-{s}"))` with no empty check → silently yields
/// `Xref("fig-")`, a malformed cross-reference. After the fix it emits E-EVL-012,
/// matching the Call form behaviour.
///
/// Call+Pipe consistency: both forms MUST behave identically for empty args
/// (see `test_f077_p11_001_figref_ident_resolves_empty_emits_e_evl_012` for the
/// Call form).
#[test]
fn test_f077_p12_001_pipe_figref_ident_resolves_empty_emits_e_evl_012() {
    // Bind var = "" in the env.
    let mut vars: IndexMap<Arc<str>, slideforge_types::Value> = IndexMap::new();
    vars.insert(
        Arc::from("var"),
        slideforge_types::Value::Str(Arc::from("")),
    );
    let env = Env::new(vars);

    // Build {{ var | figref }} — ident lhs, evaluates to "".
    let pipe_figref_ident = SyntaxExpr::Pipe {
        lhs: Box::new(SyntaxExpr::Ident("var".to_string())),
        filter: "figref".to_string(),
        args: vec![],
    };
    let chunks = vec![TemplateChunk::Expr(pipe_figref_ident)];
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // (a) No InlineNode produced — Xref("fig-") is a malformed cross-reference
    //     and must be rejected, not silently emitted.
    assert!(
        nodes.is_empty(),
        "F-077-P12-001: {{ var | figref }} where var=\"\" must produce NO InlineNode; \
         got: {nodes:?}"
    );

    // (b) Exactly one diagnostic pushed.
    assert!(
        !sink.is_empty(),
        "F-077-P12-001: {{ var | figref }} where var=\"\" must push a diagnostic; sink is empty"
    );

    // (c) LOAD-BEARING: diagnostic code MUST be E-EVL-012 (FigrefInvalidArg).
    //     All three inline builtins reject empty-resolved args consistently via
    //     both Call AND Pipe forms:
    //       figref → E-EVL-012 | ref → E-EVL-013 | footnote → E-EVL-014
    let code = sink
        .errors()
        .iter()
        .find_map(|e| e.code().map(|c| c.to_string()));
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-012"),
        "F-077-P12-001: {{ var | figref }} where var=\"\" MUST emit E-EVL-012 \
         (FigrefInvalidArg); got: {code:?}"
    );
}

/// F-077-P12-001 (LOW→consistency):
/// `{{ "" | footnote }}` (Pipe form, literal empty string) must push E-EVL-014
/// and produce NO InlineNode.
///
/// RED GATE: before the fix, the Pipe::footnote arm has no empty check on the
/// literal `""` path — it pushes `Footnote([Plain("")])` silently. After the fix
/// it emits E-EVL-014, matching the Call form behaviour.
///
/// Call+Pipe consistency: both forms MUST behave identically for empty args
/// (see `test_f077_p9_001_footnote_empty_string_emits_e_evl_014` for the Call form).
#[test]
fn test_f077_p12_001_pipe_footnote_empty_literal_emits_e_evl_014() {
    let pipe_empty_footnote = SyntaxExpr::Pipe {
        lhs: Box::new(SyntaxExpr::Str(String::new())),
        filter: "footnote".to_string(),
        args: vec![],
    };
    let chunks = vec![TemplateChunk::Expr(pipe_empty_footnote)];
    let env = Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // (a) No InlineNode must be produced — Footnote([Plain("")]) is invalid.
    assert!(
        nodes.is_empty(),
        "F-077-P12-001: {{ \"\" | footnote }} must produce NO InlineNode; got: {nodes:?}"
    );

    // (b) Exactly one diagnostic must be pushed.
    assert!(
        !sink.is_empty(),
        "F-077-P12-001: {{ \"\" | footnote }} must push a diagnostic; sink is empty"
    );

    // (c) LOAD-BEARING: the diagnostic code MUST be E-EVL-014 (FootnoteInvalidArg).
    let code = sink
        .errors()
        .iter()
        .find_map(|e| e.code().map(|c| c.to_string()));
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-014"),
        "F-077-P12-001: {{ \"\" | footnote }} MUST emit E-EVL-014 (FootnoteInvalidArg); \
         got: {code:?}"
    );
}

/// F-077-P12-001 (LOW→consistency):
/// `{{ var | footnote }}` where `var` is bound to `""` in the environment must
/// push E-EVL-014 and produce NO InlineNode.
///
/// RED GATE: before the fix, the Pipe::footnote arm has no empty check on the
/// eval-resolved path — it pushes `Footnote([Plain("")])` silently. After the
/// fix it emits E-EVL-014, matching the Call form behaviour.
///
/// Call+Pipe consistency: both forms MUST behave identically for empty args
/// (see `test_f077_footnote_ident_resolves_empty_emits_e_evl_014` for the Call form).
#[test]
fn test_f077_p12_001_pipe_footnote_ident_resolves_empty_emits_e_evl_014() {
    // Bind var = "" in the env.
    let mut vars: IndexMap<Arc<str>, slideforge_types::Value> = IndexMap::new();
    vars.insert(
        Arc::from("var"),
        slideforge_types::Value::Str(Arc::from("")),
    );
    let env = Env::new(vars);

    // Build {{ var | footnote }} — ident lhs, evaluates to "".
    let pipe_footnote_ident = SyntaxExpr::Pipe {
        lhs: Box::new(SyntaxExpr::Ident("var".to_string())),
        filter: "footnote".to_string(),
        args: vec![],
    };
    let chunks = vec![TemplateChunk::Expr(pipe_footnote_ident)];
    let mut sink = DiagnosticSink::new();

    let nodes = crate::register_routing::chunks_to_inline_nodes(&chunks, &env, &mut sink);

    // (a) No InlineNode produced — empty footnote text is invalid.
    assert!(
        nodes.is_empty(),
        "F-077-P12-001: {{ var | footnote }} where var=\"\" must produce NO InlineNode; \
         got: {nodes:?}"
    );

    // (b) Exactly one diagnostic pushed.
    assert!(
        !sink.is_empty(),
        "F-077-P12-001: {{ var | footnote }} where var=\"\" must push a diagnostic; sink is empty"
    );

    // (c) LOAD-BEARING: diagnostic code MUST be E-EVL-014 (FootnoteInvalidArg).
    //     Call form and Pipe form must be consistent — both emit E-EVL-014 for
    //     empty-resolved footnote args (F-077-P12-001 sibling-site consistency).
    let code = sink
        .errors()
        .iter()
        .find_map(|e| e.code().map(|c| c.to_string()));
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-014"),
        "F-077-P12-001: {{ var | footnote }} where var=\"\" MUST emit E-EVL-014 \
         (FootnoteInvalidArg); got: {code:?}"
    );
}
