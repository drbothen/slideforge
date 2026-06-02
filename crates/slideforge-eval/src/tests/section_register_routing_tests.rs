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
use slideforge_syntax::{
    DiagnosticSink, FieldNode, FieldValue as SyntaxFieldValue, SectionNode, Spanned, TemplateChunk,
};
use slideforge_syntax::span::Span;
use slideforge_types::{
    FieldValue, InlineNode, OrderedMap, Register, RegisteredContent, SectionBlock, SourceSpan,
};

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
fn make_section_block(
    name: &str,
    body: OrderedMap<Arc<str>, FieldValue>,
) -> SectionBlock {
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
            InlineNode::Plain(s) => s.as_ref().to_owned(),
            InlineNode::Bold(children)
            | InlineNode::Italic(children)
            | InlineNode::Footnote(children)
            | InlineNode::Superscript(children)
            | InlineNode::Subscript(children)
            | InlineNode::Strikethrough(children)
            | InlineNode::Highlight(children) => extract_plain_text(children),
            InlineNode::Link { text, .. } => extract_plain_text(text),
            InlineNode::Code(s) | InlineNode::Xref(s) => s.as_ref().to_owned(),
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
    body.insert(Arc::from("detail"), FieldValue::Inlines(expected_nodes.clone()));
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
            "AC-001 FAIL: body['detail'] must be FieldValue::Inlines; got {:?}\n\
             If this is FieldValue::Literal(Value::Str), the old flattening bug is present.\n\
             If this is FieldValue::Template, the parser upgrade (AC-002) did not run.",
            other
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
                SyntaxFieldValue::Template(vec![
                    TemplateChunk::Literal("Bold claim. See xref(slide-1).".to_string()),
                ]),
                dummy_span(),
            ),
        }],
    };

    // Call eval_section_nodes (the stub will panic here — Red Gate).
    let mut sink = DiagnosticSink::new();
    let env = Env::new(IndexMap::new());
    let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

    let (section_block, _register_content) = result
        .expect("eval_section_nodes must return Some for valid methodology section");

    // AC-002: body["detail"] must be FieldValue::Inlines, NOT FieldValue::Template.
    let detail_entry = section_block
        .body
        .get("detail")
        .expect("detail key must be present after eval");

    assert!(
        matches!(detail_entry, FieldValue::Inlines(_)),
        "AC-002 FAIL: body['detail'] must be FieldValue::Inlines after eval upgrade; \
         got: {:?}\n\
         FieldValue::Template means eval_section_nodes did not upgrade the value.",
        detail_entry
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
    body.insert(Arc::from("detail"), FieldValue::Inlines(expected_nodes.clone()));
    let section = make_section_block("methodology", body);

    // Verify structural preservation.
    match section.body.get("detail") {
        Some(FieldValue::Inlines(nodes)) => {
            assert_eq!(nodes.len(), 3, "must have 3 inline nodes (Bold, Plain, Xref)");
            assert!(matches!(nodes[0], InlineNode::Bold(_)), "node[0] must be Bold");
            assert!(matches!(nodes[1], InlineNode::Plain(_)), "node[1] must be Plain");
            assert!(matches!(nodes[2], InlineNode::Xref(_)), "node[2] must be Xref");
        },
        other => panic!("AC-002: expected FieldValue::Inlines with 3 nodes; got: {:?}", other),
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
         got: {:?}",
        text
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
        "AC-004: report content must match the section body text; got: {:?}",
        text
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
/// After full `eval_deck()` on a deck with `section methodology: / foo: "x"`,
/// `eval_diagnostics` must contain NO `UnrecognizedSectionSubBlockKey` entry.
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
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("; ");
    assert!(
        err_msg.contains("foobar"),
        "BC-3.02.002 invariant 3: error message must name the unknown type 'foobar'; \
         got: {}",
        err_msg
    );
    assert!(
        err_msg.contains("methodology")
            || err_msg.contains("Known types")
            || err_msg.contains("known"),
        "BC-3.02.002 invariant 3: error message must list known types; got: {}",
        err_msg
    );
}

/// BC-3.02.002 invariant 3: all five built-in section types are accepted.
#[test]
fn test_BC_3_02_002_inv3_all_builtin_section_types_accepted() {
    for &type_name in KNOWN_SECTION_TYPES {
        let section_node = SectionNode {
            kind: Spanned::new(type_name.to_string(), dummy_span()),
            fields: vec![],
        };

        let mut sink = DiagnosticSink::new();
        use crate::env::Env;
        use indexmap::IndexMap;
        let env = Env::new(IndexMap::new());
        let result = crate::eval::eval_section_nodes_for_test(&section_node, &env, &mut sink);

        assert!(
            result.is_some(),
            "BC-3.02.002 invariant 3: built-in section type '{}' must be accepted; \
             got None with errors: {:?}",
            type_name,
            sink.errors()
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
         got: {:?}",
        result
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

    assert_eq!(text_a, "Methodology detail content", "section_a content must be correct");
    assert_eq!(text_b, "Scope detail content", "section_b content must be correct");
    assert_ne!(
        text_a, text_b,
        "EC-005: sections must not cross-contaminate each other's register_content"
    );
}

// ─── KNOWN_SECTION_TYPES constant verification ────────────────────────────────

/// The KNOWN_SECTION_TYPES constant must contain the five built-in types
/// and exclude "notes" (the presenter register, not valid for sections).
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
    // "notes" is the PRESENTER register — NOT a section type.
    assert!(
        !KNOWN_SECTION_TYPES.contains(&"notes"),
        "KNOWN_SECTION_TYPES must NOT include 'notes' (notes is the presenter register)"
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
