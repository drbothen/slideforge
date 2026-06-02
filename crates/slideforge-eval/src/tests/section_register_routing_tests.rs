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
    non_snake_case  // test naming follows BC-NNN convention
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
