//! STORY-081 Red Gate tests — eval-stage slide-level inline markup conversion.
//!
//! ## Traceability
//!
//! | Test function | AC/EC | BC clause | What is verified |
//! |---|---|---|---|
//! | `test_BC_3_05_001_ac001_bullet_bold_produces_field_value_inlines` | AC-001 | BC-3.05.001 precondition 5 | bold bullet → FieldValue::Inlines, not FieldValue::Str |
//! | `test_BC_3_05_001_ac001_bullet_italic_produces_field_value_inlines` | AC-001 | BC-3.05.001 precondition 5 | italic bullet → FieldValue::Inlines |
//! | `test_BC_3_05_001_ac001_body_inline_markup_produces_field_value_inlines` | AC-001 | BC-3.05.001 precondition 5 | body field with markup → FieldValue::Inlines |
//! | `test_BC_3_05_001_ac001_rejects_literal_asterisks_in_bullet` | AC-001 | BC-3.05.001 precondition 5 | bullet must NOT produce FieldValue::Str("**bold**") |
//! | `test_BC_3_05_001_ac001_plain_bullet_stays_str` | AC-001 | BC-3.05.001 precondition 5 / Invariant 10 | plain-text bullet → FieldValue::Str (EC-006) |
//! | `test_BC_3_05_001_ac001_all_8_inline_forms_in_body` | AC-001 | BC-3.05.001 precondition 5 | all 8 inline forms → FieldValue::Inlines with correct variants |
//! | `test_BC_3_05_001_ac006_title_with_bold_emits_warning` | AC-006 | BC-3.05.001 Slide-Level Title Constraint / EC-011 | title with markup → EvalError::InlineMarkupInTitle (E-EVL-015) in sink |
//! | `test_BC_3_05_001_ac006_title_with_bold_strips_to_plain_str` | AC-006 | BC-3.05.001 Slide-Level Title Constraint / EC-011 | title with markup → FieldValue::Str (plain) for PPTX |
//! | `test_BC_3_05_001_ac006_plain_title_no_warning` | AC-006 | BC-3.05.001 Slide-Level Title Constraint / EC-011 | plain title → no warning emitted |
//! | `test_BC_3_05_001_ec003_nested_bold_italic_in_bullet` | EC-003 | BC-3.05.001 precondition 5 | Bold([Italic(...)]) nesting preserved |
//! | `test_BC_3_05_001_ec007_var_resolves_to_asterisks_stays_plain` | EC-007 | BC-3.05.001 Invariant 10 / EC-015 | `{{ var }}` resolving to "**bold**" → Plain("**bold**"), not re-parsed |
//! | `test_BC_3_05_001_ec010_empty_bullets_list` | EC-010 | BC-3.05.001 precondition 5 | empty bullets list → no FieldValue::Inlines produced |
//! | `test_BC_3_05_001_ac001_caption_inline_markup_produces_inlines` | AC-001 | BC-3.05.001 precondition 5 | caption field with markup → FieldValue::Inlines |
//! | `test_BC_3_05_001_ac001_description_inline_markup_produces_inlines` | AC-001 | BC-3.05.001 precondition 5 | description field with markup → FieldValue::Inlines |
//! | `test_BC_3_05_001_ac001_subtitle_inline_markup_produces_inlines` | AC-001 | BC-3.05.001 precondition 5 | subtitle field with markup → FieldValue::Inlines |
//!
//! ## Red Gate contract
//!
//! ALL tests in this file MUST FAIL before STORY-081 implementation begins.
//! The implementation target is `eval_slide_node` in
//! `crates/slideforge-eval/src/for_eval.rs`: the `FieldValue::Template(chunks)` arm
//! must detect inline-markup-carrying fields and call `chunks_to_inline_nodes`
//! instead of `flatten_chunks_to_string`.

#![allow(non_snake_case)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::similar_names)]

use std::collections::HashMap;
use std::sync::Arc;

use slideforge_syntax::span::Span;
use slideforge_syntax::{DiagnosticSink, FieldNode, FieldValue, SlideNode, Spanned, TemplateChunk};
use slideforge_types::{FieldValue as TypedFieldValue, InlineNode};

use crate::env::Env;
use crate::for_eval::eval_slide_node;

// ─── Fixture helpers ──────────────────────────────────────────────────────────

/// Build a dummy span for test AST nodes.
fn dummy_span() -> Span {
    Span::new(0, 0, 0)
}

/// Build a minimal `SlideNode` with a single named field whose value is a
/// `FieldValue::Template(chunks)`.
fn make_slide_with_template_field(
    slide_type: &str,
    field_name: &str,
    chunks: Vec<TemplateChunk>,
) -> SlideNode {
    SlideNode {
        kind: Spanned::new(slide_type.to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new(field_name.to_string(), dummy_span()),
            value: Spanned::new(FieldValue::Template(chunks), dummy_span()),
        }],
        inline_items: vec![],
        tags: vec![],
    }
}

/// Build a `DiagnosticSink` and empty `Env` for tests.
fn make_env_and_sink() -> (Env, DiagnosticSink) {
    (Env::new(indexmap::IndexMap::new()), DiagnosticSink::new())
}

/// Extract the `FieldValue` for a field from the result of `eval_slide_node`.
fn get_field(result: &slideforge_types::Slide, field_name: &str) -> Option<TypedFieldValue> {
    result.fields.get(field_name).cloned()
}

// ─── AC-001: Bullet with bold inline markup → FieldValue::Inlines ─────────────

/// AC-001: A slide bullet containing `**Key finding**: revenue up 12%` must
/// produce `FieldValue::Inlines` containing `InlineNode::Bold` — NOT
/// `FieldValue::Str("**Key finding**: revenue up 12%")`.
///
/// Red Gate: currently fails because `eval_slide_node` uses `flatten_chunks_to_string`
/// for all Template fields, producing `FieldValue::Str` regardless of markup.
#[test]
fn test_BC_3_05_001_ac001_bullet_bold_produces_field_value_inlines() {
    let bold_chunk = TemplateChunk::Bold(vec![TemplateChunk::Literal("Key finding".to_string())]);
    let chunks = vec![
        bold_chunk,
        TemplateChunk::Literal(": revenue up 12%".to_string()),
    ];

    let slide_node = make_slide_with_template_field("content", "bullets", chunks);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed for a valid slide");

    let field_value =
        get_field(&result, "bullets").expect("bullets field must be present in evaluated slide");

    // The field MUST be Inlines — not Str("**Key finding**...").
    assert!(
        matches!(&field_value, TypedFieldValue::Inlines(_)),
        "AC-001: bullets field with bold markup must produce FieldValue::Inlines; \
         got: {field_value:?}"
    );

    // The Inlines content must start with InlineNode::Bold.
    if let TypedFieldValue::Inlines(nodes) = &field_value {
        assert!(
            !nodes.is_empty(),
            "AC-001: Inlines must be non-empty for a bullet with markup"
        );
        assert!(
            matches!(nodes[0], InlineNode::Bold(_)),
            "AC-001: first InlineNode in bullet must be Bold; got: {:?}",
            nodes[0]
        );
    }
}

/// AC-001: An italic bullet must produce `FieldValue::Inlines` with `InlineNode::Italic`.
#[test]
fn test_BC_3_05_001_ac001_bullet_italic_produces_field_value_inlines() {
    let italic_chunk = TemplateChunk::Italic(vec![TemplateChunk::Literal("appendix".to_string())]);
    let chunks = vec![
        TemplateChunk::Literal("See ".to_string()),
        italic_chunk,
        TemplateChunk::Literal(" for details".to_string()),
    ];

    let slide_node = make_slide_with_template_field("content", "bullets", chunks);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed");

    let field_value = get_field(&result, "bullets").expect("bullets field must be present");

    assert!(
        matches!(&field_value, TypedFieldValue::Inlines(_)),
        "AC-001: italic bullet must produce FieldValue::Inlines; got: {field_value:?}"
    );

    if let TypedFieldValue::Inlines(nodes) = &field_value {
        // Expect: Plain("See "), Italic([Plain("appendix")]), Plain(" for details")
        assert_eq!(
            nodes.len(),
            3,
            "Expected 3 inline nodes: Plain, Italic, Plain"
        );
        assert!(
            matches!(&nodes[1], InlineNode::Italic(_)),
            "Second node must be Italic; got: {:?}",
            nodes[1]
        );
    }
}

/// AC-001 rejection test: a bullet with bold markup must NOT produce a literal
/// `FieldValue::Str("**Key finding**: revenue up 12%")` (literal asterisks).
///
/// This test verifies the specific anti-pattern that STORY-081 closes (R1 finding).
#[test]
fn test_BC_3_05_001_ac001_rejects_literal_asterisks_in_bullet() {
    let chunks = vec![
        TemplateChunk::Bold(vec![TemplateChunk::Literal("Key finding".to_string())]),
        TemplateChunk::Literal(": revenue up 12%".to_string()),
    ];

    let slide_node = make_slide_with_template_field("content", "bullets", chunks);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed");

    let field_value = get_field(&result, "bullets").expect("bullets field must be present");

    // Explicitly reject FieldValue::Str containing literal asterisks.
    if let TypedFieldValue::Literal(slideforge_types::Value::Str(s)) = &field_value {
        assert!(
            !s.contains("**"),
            "AC-001: bullet field must NOT contain literal '**' characters; \
             got FieldValue::Str({s:?}) — this is the R1 anti-pattern that STORY-081 closes"
        );
    }
    // If it's Inlines, that's also the correct form — the assertion above protects
    // against the specific failure mode (literal asterisks in Str).
}

/// EC-006: A plain-text bullet (no inline markup) may stay as `FieldValue::Str`
/// or become `FieldValue::Inlines([Plain(...)])`. Both are valid per the spec.
/// This test verifies that plain-text bullets are NOT broken by the STORY-081 change.
#[test]
fn test_BC_3_05_001_ac001_plain_bullet_stays_str() {
    let chunks = vec![TemplateChunk::Literal(
        "Revenue grew 12% year-over-year".to_string(),
    )];

    let slide_node = make_slide_with_template_field("content", "bullets", chunks);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed");

    let field_value = get_field(&result, "bullets").expect("bullets field must be present");

    // Both FieldValue::Str and FieldValue::Inlines([Plain(...)]) are valid.
    // The important invariant: the literal text must NOT contain literal asterisks.
    match &field_value {
        TypedFieldValue::Literal(slideforge_types::Value::Str(s)) => {
            assert!(
                !s.contains("**"),
                "plain bullet must not contain literal '**'; got: {s:?}"
            );
        },
        TypedFieldValue::Inlines(nodes) => {
            assert!(
                !nodes.is_empty(),
                "Inlines for plain bullet must be non-empty"
            );
        },
        other => panic!("plain bullet must be Str or Inlines; got: {other:?}"),
    }
}

// ─── AC-001: Body / caption / description / subtitle fields ───────────────────

/// AC-001: A `body` field with bold inline markup must produce `FieldValue::Inlines`.
#[test]
fn test_BC_3_05_001_ac001_body_inline_markup_produces_field_value_inlines() {
    let chunks = vec![
        TemplateChunk::Bold(vec![TemplateChunk::Literal("Note".to_string())]),
        TemplateChunk::Literal(": this is important".to_string()),
    ];

    let slide_node = make_slide_with_template_field("content", "body", chunks);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed");

    let field_value = get_field(&result, "body").expect("body field must be present");

    assert!(
        matches!(&field_value, TypedFieldValue::Inlines(_)),
        "AC-001: body field with bold markup must produce FieldValue::Inlines; \
         got: {field_value:?}"
    );
}

/// AC-001: A `caption` field with inline markup must produce `FieldValue::Inlines`.
#[test]
fn test_BC_3_05_001_ac001_caption_inline_markup_produces_inlines() {
    let chunks = vec![
        TemplateChunk::Italic(vec![TemplateChunk::Literal("Figure 1".to_string())]),
        TemplateChunk::Literal(": revenue chart".to_string()),
    ];

    let slide_node = make_slide_with_template_field("content", "caption", chunks);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed");

    let field_value = get_field(&result, "caption").expect("caption field must be present");

    assert!(
        matches!(&field_value, TypedFieldValue::Inlines(_)),
        "AC-001: caption field with italic markup must produce FieldValue::Inlines; \
         got: {field_value:?}"
    );
}

/// AC-001: A `description` field with inline markup must produce `FieldValue::Inlines`.
#[test]
fn test_BC_3_05_001_ac001_description_inline_markup_produces_inlines() {
    let chunks = vec![
        TemplateChunk::Bold(vec![TemplateChunk::Literal("Description".to_string())]),
        TemplateChunk::Literal(": key metric".to_string()),
    ];

    let slide_node = make_slide_with_template_field("content", "description", chunks);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed");

    let field_value = get_field(&result, "description").expect("description field must be present");

    assert!(
        matches!(&field_value, TypedFieldValue::Inlines(_)),
        "AC-001: description field with bold markup must produce FieldValue::Inlines; \
         got: {field_value:?}"
    );
}

/// AC-001: A `subtitle` field with inline markup must produce `FieldValue::Inlines`.
#[test]
fn test_BC_3_05_001_ac001_subtitle_inline_markup_produces_inlines() {
    let chunks = vec![TemplateChunk::Italic(vec![TemplateChunk::Literal(
        "subtitle text".to_string(),
    )])];

    let slide_node = make_slide_with_template_field("title", "subtitle", chunks);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed");

    let field_value = get_field(&result, "subtitle").expect("subtitle field must be present");

    assert!(
        matches!(&field_value, TypedFieldValue::Inlines(_)),
        "AC-001: subtitle field with italic markup must produce FieldValue::Inlines; \
         got: {field_value:?}"
    );
}

// ─── AC-001: All 8 inline forms in body ──────────────────────────────────────

/// AC-001 completeness: A body field with all 8 inline markup forms must
/// produce `FieldValue::Inlines` containing all 8 `InlineNode` variants:
/// Bold, Italic, Code, Link, Superscript, Subscript, Strikethrough, Highlight.
#[test]
fn test_BC_3_05_001_ac001_all_8_inline_forms_in_body() {
    let chunks = vec![
        TemplateChunk::Bold(vec![TemplateChunk::Literal("bold".to_string())]),
        TemplateChunk::Literal(" ".to_string()),
        TemplateChunk::Italic(vec![TemplateChunk::Literal("italic".to_string())]),
        TemplateChunk::Literal(" ".to_string()),
        TemplateChunk::Code("code".to_string()),
        TemplateChunk::Literal(" ".to_string()),
        TemplateChunk::Link {
            text: vec![TemplateChunk::Literal("link".to_string())],
            url: "https://example.com".to_string(),
        },
        TemplateChunk::Literal(" ".to_string()),
        TemplateChunk::Superscript(vec![TemplateChunk::Literal("sup".to_string())]),
        TemplateChunk::Literal(" ".to_string()),
        TemplateChunk::Subscript(vec![TemplateChunk::Literal("sub".to_string())]),
        TemplateChunk::Literal(" ".to_string()),
        TemplateChunk::Strikethrough(vec![TemplateChunk::Literal("del".to_string())]),
        TemplateChunk::Literal(" ".to_string()),
        TemplateChunk::Highlight(vec![TemplateChunk::Literal("highlight".to_string())]),
    ];

    let slide_node = make_slide_with_template_field("content", "body", chunks);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed");

    let field_value = get_field(&result, "body").expect("body field must be present");

    assert!(
        matches!(&field_value, TypedFieldValue::Inlines(_)),
        "AC-001: body with all 8 forms must produce FieldValue::Inlines; got: {field_value:?}"
    );

    if let TypedFieldValue::Inlines(nodes) = &field_value {
        // Verify all 8 structural inline variants are present.
        let has_bold = nodes.iter().any(|n| matches!(n, InlineNode::Bold(_)));
        let has_italic = nodes.iter().any(|n| matches!(n, InlineNode::Italic(_)));
        let has_code = nodes.iter().any(|n| matches!(n, InlineNode::Code(_)));
        let has_link = nodes.iter().any(|n| matches!(n, InlineNode::Link { .. }));
        let has_sup = nodes
            .iter()
            .any(|n| matches!(n, InlineNode::Superscript(_)));
        let has_sub = nodes.iter().any(|n| matches!(n, InlineNode::Subscript(_)));
        let has_del = nodes
            .iter()
            .any(|n| matches!(n, InlineNode::Strikethrough(_)));
        let has_highlight = nodes.iter().any(|n| matches!(n, InlineNode::Highlight(_)));

        assert!(has_bold, "InlineNode::Bold must be present");
        assert!(has_italic, "InlineNode::Italic must be present");
        assert!(has_code, "InlineNode::Code must be present");
        assert!(has_link, "InlineNode::Link must be present");
        assert!(has_sup, "InlineNode::Superscript must be present");
        assert!(has_sub, "InlineNode::Subscript must be present");
        assert!(has_del, "InlineNode::Strikethrough must be present");
        assert!(has_highlight, "InlineNode::Highlight must be present");
    }
}

// ─── AC-006: Title with inline markup → warning + plain string ────────────────

/// AC-006: A `title` field with bold markup must emit an
/// `EvalError::InlineMarkupInTitle (E-EVL-015)` diagnostic into the diagnostic sink.
///
/// The diagnostic is emitted at `ParseSeverity::Error` severity (F-P25-HIGH-001):
/// this causes the strict gate in `slideforge::compile_inner` to fire and return
/// `BuildError::EvalFailed` — the build fails in strict mode (default), satisfying
/// BC-3.05.001 EC-011.
///
/// In warn-only mode (strict=false) the gate does not fire and output is produced
/// with the title stripped to plain text.
///
/// **Strengthened (F-P27-MED-001):** also asserts that the rendered diagnostic message
/// shows DIFFERENT `slide_title` and `stripped_text` substitutions — i.e. the
/// "Inline markup in title field: '...'" part differs from the "Stripped to: '...'"
/// part, confirming that `slide_title` carries the markup form (e.g. `**Bold Title**`)
/// and `stripped_text` carries the plain form (e.g. `Bold Title`).
#[test]
fn test_BC_3_05_001_ac006_title_with_bold_emits_warning() {
    let chunks = vec![TemplateChunk::Bold(vec![TemplateChunk::Literal(
        "Bold Title".to_string(),
    )])];

    let slide_node = make_slide_with_template_field("title", "title", chunks);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let _result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must not return None for a title-with-markup slide");

    // The diagnostic sink must contain EvalError::InlineMarkupInTitle (E-EVL-015).
    let has_title_warning = sink_has_inline_markup_title_warning(&sink);
    assert!(
        has_title_warning,
        "AC-006: eval_slide_node must emit EvalError::InlineMarkupInTitle (E-EVL-015) for a \
         title with bold markup; no such diagnostic found in the sink (len: {})",
        sink.len()
    );

    // F-P27-MED-001: the rendered error message must show DIFFERENT before/after values.
    // The `#[error(...)]` template is:
    //   "Inline markup in title field: '{slide_title}' — PPTX requires plain-text titles.
    //    Stripped to: '{stripped_text}'"
    // With the bug both substitutions are identical; with the fix slide_title contains
    // markup delimiters (e.g. "**Bold Title**") and stripped_text is plain ("Bold Title").
    let msg = get_inline_markup_title_diagnostic_message(&sink)
        .expect("E-EVL-015 diagnostic must be present in sink");
    assert!(
        !msg.contains("Stripped to: 'Bold Title'") || msg.contains("**"),
        "F-P27-MED-001: rendered diagnostic must show markup in the title portion; \
         got: {msg:?}"
    );
    // Stronger: slide_title portion contains '**' (markup delimiter)
    // The message format is: "Inline markup in title field: '...' — PPTX ..."
    // Extract the part before " — PPTX" to verify it has markup.
    assert!(
        msg.contains("**"),
        "F-P27-MED-001: the E-EVL-015 diagnostic message must contain '**' (the markup \
         form of the title); got: {msg:?}"
    );
}

/// AC-006: A `title` field with bold markup must produce a plain-text
/// `FieldValue::Str("Bold Title")` (no markup delimiters, no asterisks).
///
/// For PPTX output, the PPTX exporter needs plain text from the title.
/// For non-PPTX outputs (DOCX/PDF/HTML), the inline structure is preserved
/// in a separate field (deferred to implementer; test verifies PPTX path).
#[test]
fn test_BC_3_05_001_ac006_title_with_bold_strips_to_plain_str() {
    let chunks = vec![TemplateChunk::Bold(vec![TemplateChunk::Literal(
        "Bold Title".to_string(),
    )])];

    let slide_node = make_slide_with_template_field("title", "title", chunks);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must not return None");

    let field_value = get_field(&result, "title").expect("title field must be present");

    // The PPTX-path title field must be plain string "Bold Title" (no asterisks).
    match &field_value {
        TypedFieldValue::Literal(slideforge_types::Value::Str(s)) => {
            assert_eq!(
                s.as_ref(),
                "Bold Title",
                "AC-006: title with bold markup must strip to plain 'Bold Title'; got: {s:?}"
            );
            assert!(
                !s.contains("**"),
                "AC-006: title must NOT contain literal '**' asterisks; got: {s:?}"
            );
        },
        other => {
            panic!("AC-006: title with markup must produce FieldValue::Str (plain); got: {other:?}")
        },
    }
}

/// AC-006: A plain-text title must NOT emit any `InlineMarkupInTitle` warning.
#[test]
fn test_BC_3_05_001_ac006_plain_title_no_warning() {
    let chunks = vec![TemplateChunk::Literal("Plain Title".to_string())];

    let slide_node = make_slide_with_template_field("title", "title", chunks);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let _result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed for plain title");

    let has_warning = sink_has_inline_markup_title_warning(&sink);
    assert!(
        !has_warning,
        "AC-006: plain title must NOT emit InlineMarkupInTitle warning"
    );
}

// ─── EC-003: Nested bold-italic in bullet ────────────────────────────────────

/// EC-003: A bullet with `**_bold italic_**` must produce
/// `InlineNode::Bold([InlineNode::Italic([InlineNode::Plain("bold italic")])])`.
#[test]
fn test_BC_3_05_001_ec003_nested_bold_italic_in_bullet() {
    let nested = TemplateChunk::Bold(vec![TemplateChunk::Italic(vec![TemplateChunk::Literal(
        "bold italic".to_string(),
    )])]);

    let slide_node = make_slide_with_template_field("content", "bullets", vec![nested]);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed");

    let field_value = get_field(&result, "bullets").expect("bullets field must be present");

    assert!(
        matches!(&field_value, TypedFieldValue::Inlines(_)),
        "EC-003: nested bold-italic bullet must produce FieldValue::Inlines; \
         got: {field_value:?}"
    );

    if let TypedFieldValue::Inlines(nodes) = &field_value {
        assert_eq!(nodes.len(), 1, "Expected exactly 1 top-level node (Bold)");
        match &nodes[0] {
            InlineNode::Bold(bold_children) => {
                assert_eq!(
                    bold_children.len(),
                    1,
                    "Bold must have exactly 1 child (Italic)"
                );
                match &bold_children[0] {
                    InlineNode::Italic(italic_children) => {
                        assert_eq!(italic_children.len(), 1, "Italic must have 1 child (Plain)");
                        assert!(
                            matches!(&italic_children[0], InlineNode::Plain(s) if s.as_ref() == "bold italic"),
                            "EC-003: innermost node must be Plain('bold italic'); \
                             got: {:?}",
                            italic_children[0]
                        );
                    },
                    other => panic!("EC-003: Bold child must be Italic; got: {other:?}"),
                }
            },
            other => panic!("EC-003: top-level node must be Bold; got: {other:?}"),
        }
    }
}

// ─── EC-007: Variable resolving to "**bold**" stays Plain ────────────────────

/// EC-007: A `{{ var }}` expression that resolves to the string `"**bold**"` must
/// produce `InlineNode::Plain(Arc::from("**bold**"))` — NOT further parsed.
///
/// This is the security invariant: resolved values are not re-parsed for markup.
#[test]
fn test_BC_3_05_001_ec007_var_resolves_to_asterisks_stays_plain() {
    use slideforge_syntax::Expr;

    // Template: {{ var_with_asterisks }} where var evaluates to "**bold**".
    let chunks = vec![TemplateChunk::Expr(Expr::Ident(
        "var_with_asterisks".to_string(),
    ))];

    let slide_node = make_slide_with_template_field("content", "bullets", chunks);

    // Set up env with var_with_asterisks = "**bold**"
    let mut deck_vars = indexmap::IndexMap::new();
    deck_vars.insert(
        Arc::from("var_with_asterisks"),
        slideforge_types::Value::Str(Arc::from("**bold**")),
    );
    let env = Env::new(deck_vars);

    let mut sink = DiagnosticSink::new();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed");

    let field_value = get_field(&result, "bullets").expect("bullets field must be present");

    match &field_value {
        TypedFieldValue::Inlines(nodes) => {
            // If Inlines, the resolved value must be a single Plain("**bold**")
            // — not Bold([Plain("bold")]).
            assert_eq!(
                nodes.len(),
                1,
                "EC-007: resolved var must produce exactly 1 InlineNode"
            );
            assert!(
                matches!(&nodes[0], InlineNode::Plain(s) if s.as_ref() == "**bold**"),
                "EC-007: resolved var containing '**' must be Plain, not Bold; got: {:?}",
                nodes[0]
            );
        },
        TypedFieldValue::Literal(slideforge_types::Value::Str(s)) => {
            // If Str, it must contain the literal asterisks (not be parsed).
            assert_eq!(
                s.as_ref(),
                "**bold**",
                "EC-007: resolved var must produce Str('**bold**'); got: {s:?}"
            );
        },
        other => panic!("EC-007: unexpected field value variant: {other:?}"),
    }
}

// ─── EC-010: Empty bullets list ───────────────────────────────────────────────

/// EC-010: `bullets: []` (empty chunk list) must not produce any error.
/// Either no field is set or an empty Inlines is valid.
#[test]
fn test_BC_3_05_001_ec010_empty_bullets_list() {
    let slide_node = make_slide_with_template_field("content", "bullets", vec![]);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None);

    // Must not panic or return None (unless there is a fatal error for other reasons).
    // An empty bullets list is valid — either produces nothing or Inlines([]).
    assert!(
        result.is_some() || sink.has_fatal(),
        "EC-010: empty bullets list must not crash eval_slide_node"
    );

    // Must not have produced any unexpected fatal errors.
    assert!(
        !sink.has_fatal(),
        "EC-010: empty bullets list must not produce any fatal errors; \
         got {} diagnostics",
        sink.len()
    );
}

// ─── Helper: detect InlineMarkupInTitle warning ───────────────────────────────

/// Check the diagnostic sink for an `EvalError::InlineMarkupInTitle` (E-EVL-015) entry.
///
/// Used by AC-006 tests. Returns `true` if such a diagnostic is present.
///
/// STORY-081 / F-P25-HIGH-001: detects the `E-EVL-015` code in the sink diagnostics.
/// The eval stage pushes `EvalError::InlineMarkupInTitle` at `ParseSeverity::Error`
/// (promoted from Warning by F-P25-HIGH-001) when a title field contains inline markup.
/// In strict mode (default), this causes the strict gate in `slideforge::compile_inner`
/// to fire — the build fails with `BuildError::EvalFailed`.
pub(crate) fn sink_has_inline_markup_title_warning(sink: &DiagnosticSink) -> bool {
    // Detect by error code "E-EVL-015" (EvalError::InlineMarkupInTitle).
    // DiagnosticSink stores Box<dyn miette::Diagnostic> — inspect each entry's
    // code via the miette::Diagnostic::code() method.
    sink.errors().iter().any(|diag| {
        diag.code()
            .is_some_and(|c| c.to_string().contains("E-EVL-015"))
    })
}

/// Return the rendered `Display` message of the first `E-EVL-015` diagnostic in
/// the sink, or `None` if no such diagnostic is present.
///
/// Used by F-P27-MED-001 tests to assert distinguishing `slide_title` vs
/// `stripped_text` values in the rendered error message.
pub(crate) fn get_inline_markup_title_diagnostic_message(sink: &DiagnosticSink) -> Option<String> {
    sink.errors().iter().find_map(|diag| {
        if diag
            .code()
            .is_some_and(|c| c.to_string().contains("E-EVL-015"))
        {
            Some(diag.to_string())
        } else {
            None
        }
    })
}

// ─── F-P27-MED-001 Red Gate: slide_title must carry markup form ───────────────

/// F-P27-MED-001 (Pass-27 finding): The `EvalError::InlineMarkupInTitle` diagnostic's
/// `slide_title` field must carry the markup form (e.g. `**Bold Title**`) while
/// `stripped_text` carries the plain form (`Bold Title`). These fields must differ.
///
/// BC-3.05.001 EC-011 and the field's own doc comment:
///   `slide_title: "The slide title text as it appeared (may include partial markup)."`
///
/// Red Gate: this test FAILS against the current code where both fields are set to
/// the same stripped value. After the fix it must pass.
#[test]
fn test_P27_MED_001_title_markup_diagnostic_has_distinguishing_fields() {
    // Input: title with bold markup: **Bold Title**
    let chunks = vec![TemplateChunk::Bold(vec![TemplateChunk::Literal(
        "Bold Title".to_string(),
    )])];

    let slide_node = make_slide_with_template_field("title", "title", chunks);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let _result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must not return None for a title-with-markup slide");

    // Must have emitted E-EVL-015.
    assert!(
        sink_has_inline_markup_title_warning(&sink),
        "P27-MED-001: E-EVL-015 diagnostic must be present in sink"
    );

    // Extract the rendered message to inspect both substituted fields.
    let msg = get_inline_markup_title_diagnostic_message(&sink)
        .expect("E-EVL-015 diagnostic must be present");

    // The message format (from error.rs #[error(...)]) is:
    //   "Inline markup in title field: '{slide_title}' — PPTX requires plain-text titles.
    //    Stripped to: '{stripped_text}'"
    //
    // REQUIREMENT 1: `slide_title` must contain markup delimiters (e.g. "**").
    assert!(
        msg.contains("**"),
        "P27-MED-001: E-EVL-015 message must contain '**' (the markup form of the title); \
         got: {msg:?}"
    );

    // REQUIREMENT 2: `stripped_text` must be plain "Bold Title" (no asterisks).
    // The "Stripped to: 'Bold Title'" portion must appear without asterisks.
    assert!(
        msg.contains("Stripped to: 'Bold Title'"),
        "P27-MED-001: E-EVL-015 message must contain \"Stripped to: 'Bold Title'\"; \
         got: {msg:?}"
    );

    // REQUIREMENT 3: The two substitutions must differ (slide_title ≠ stripped_text).
    // With the bug: "... '**Bold Title**' ... Stripped to: '**Bold Title**'"  -- NO wait
    // With the bug: "... 'Bold Title' ... Stripped to: 'Bold Title'" (both stripped).
    // With the fix: "... '**Bold Title**' ... Stripped to: 'Bold Title'" (different).
    //
    // We can verify this by checking that the portion before " — PPTX" contains "**"
    // AND the "Stripped to:" portion does NOT contain "**".
    let has_markup_in_title_part = msg
        .split(" \u{2014} PPTX")  // split on " — PPTX" (em dash)
        .next()
        .is_some_and(|prefix| prefix.contains("**"));
    let has_markup_in_stripped_part = msg
        .find("Stripped to: '")
        .is_some_and(|idx| msg[idx..].contains("**"));

    assert!(
        has_markup_in_title_part,
        "P27-MED-001: slide_title portion (before ' — PPTX') must contain '**'; got: {msg:?}"
    );
    assert!(
        !has_markup_in_stripped_part,
        "P27-MED-001: stripped_text portion (after 'Stripped to: ') must NOT contain '**'; \
         got: {msg:?}"
    );
}

/// F-P27-MED-001: Also test italic title — the `slide_title` field must carry `_Italic Title_`
/// while `stripped_text` is `Italic Title`.
#[test]
fn test_P27_MED_001_italic_title_diagnostic_has_distinguishing_fields() {
    let chunks = vec![TemplateChunk::Italic(vec![TemplateChunk::Literal(
        "Italic Title".to_string(),
    )])];

    let slide_node = make_slide_with_template_field("title", "title", chunks);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let _result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed");

    assert!(
        sink_has_inline_markup_title_warning(&sink),
        "P27-MED-001: E-EVL-015 must be emitted for italic title"
    );

    let msg = get_inline_markup_title_diagnostic_message(&sink).expect("E-EVL-015 must be present");

    // Italic delimiter is '_' — the markup form must contain '_'.
    assert!(
        msg.contains('_'),
        "P27-MED-001: E-EVL-015 message for italic title must contain '_' (markup delimiter); \
         got: {msg:?}"
    );
    // Stripped form must be "Italic Title" (no underscores in stripped part).
    assert!(
        msg.contains("Stripped to: 'Italic Title'"),
        "P27-MED-001: stripped_text must be 'Italic Title'; got: {msg:?}"
    );
}

// ─── AC-001 list-form (STORY-081×STORY-088 integration gap) ─────────────────────
//
// When `bullets:` uses the list-literal form `bullets: ["**bold**", "_note_", "plain"]`
// (DSL-sourced `FieldValue::List` in the AST, not `FieldValue::Template`), each
// markup-bearing item MUST be converted to `Vec<InlineNode>` — NOT flattened to a
// plain `Value::Str` with literal `**` / `_` characters.
//
// Before this fix (STORY-081×STORY-088 gap): the `FieldValue::List` arm in
// `eval_slide_node` (for_eval.rs ~463) called `eval_field_value_to_value` which
// flattens markup → produces `FieldValue::Literal(Value::List([Str("**bold**"), ...]))`.
// `field_to_block.rs` then builds `BulletItem { inlines: [Plain("**bold**")] }`.
// PPTX output contains literal `**` — not `b="1"`.
//
// After this fix: the `FieldValue::List` arm checks `INLINE_CONTENT_FIELDS` and, when
// any item has inline markup, produces `FieldValue::InlinesList(Vec<Vec<InlineNode>>)`.
// `field_to_block.rs` builds each `BulletItem` from the per-item `Vec<InlineNode>`.

/// AC-001 (list-form) RED GATE: `bullets: ["**Key finding**: up 12%", "_note_", "plain item"]`
/// via `FieldValue::List` must produce `FieldValue::InlinesList` — NOT
/// `FieldValue::Literal(Value::List([Str("**Key finding**..."), ...]))`.
///
/// ## Distinguishing assertion
/// - Item 0 carries `InlineNode::Bold` (not `Plain("**Key finding**...")`).
/// - Item 2 (plain) stays `Plain("plain item")` — no regression.
/// - The result variant is `FieldValue::InlinesList`.
///
/// ## Red Gate path
/// Before fix: `eval_slide_node` List arm → `eval_field_value_to_value` → flattens
/// markup → `FieldValue::Literal(Value::List([Str("**Key finding**..."), ...]))` →
/// assertion `matches!(FieldValue::InlinesList(_))` FAILS.
///
/// Traceability: BC-3.05.001 AC-001 list-form; STORY-081×STORY-088 integration gap.
#[test]
#[allow(clippy::similar_names)]
fn test_BC_3_05_001_ac001_list_form_bullet_bold_produces_inlines_list() {
    // Simulate the AST that the parser emits for:
    //   bullets: ["**Key finding**: up 12%", "_note_", "plain item"]
    let item0 = FieldValue::List(vec![
        // Item 0: "**Key finding**: up 12%" — contains Bold markup
        FieldValue::Template(vec![
            TemplateChunk::Bold(vec![TemplateChunk::Literal("Key finding".to_string())]),
            TemplateChunk::Literal(": up 12%".to_string()),
        ]),
        // Item 1: "_note_" — contains Italic markup
        FieldValue::Template(vec![TemplateChunk::Italic(vec![TemplateChunk::Literal(
            "note".to_string(),
        )])]),
        // Item 2: "plain item" — no markup
        FieldValue::Template(vec![TemplateChunk::Literal("plain item".to_string())]),
    ]);

    let slide_node = SlideNode {
        kind: Spanned::new("content".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("bullets".to_string(), dummy_span()),
            value: Spanned::new(item0, dummy_span()),
        }],
        inline_items: vec![],
        tags: vec![],
    };
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed for list-form bullets");

    assert!(
        sink.is_empty(),
        "AC-001 list-form RED GATE: eval must produce 0 diagnostics; got: {:?}",
        sink.errors()
    );

    let field_value = get_field(&result, "bullets").expect("bullets field must be present");

    // RED GATE: before fix, this is FieldValue::Literal(Value::List([...])).
    // After fix, it must be FieldValue::InlinesList([...]).
    assert!(
        matches!(&field_value, TypedFieldValue::InlinesList(_)),
        "AC-001 list-form RED GATE: bullets with markup items must produce \
         FieldValue::InlinesList; got: {field_value:?}\n\
         Before fix: FieldValue::Literal(Value::List([Str(\"**Key finding**...\"), ...])). \
         After fix: FieldValue::InlinesList([[Bold, Plain], [Italic], [Plain]])."
    );

    if let TypedFieldValue::InlinesList(items) = &field_value {
        assert_eq!(
            items.len(),
            3,
            "AC-001 list-form: InlinesList must have 3 items (one per bullet)"
        );

        // Item 0: must start with InlineNode::Bold (not Plain("**Key finding**...")).
        assert!(
            !items[0].is_empty(),
            "AC-001 list-form: item 0 must be non-empty"
        );
        assert!(
            matches!(items[0][0], InlineNode::Bold(_)),
            "AC-001 list-form: item 0 first node must be Bold; got: {:?}\n\
             RED GATE: before fix, items[0] == [Plain(\"**Key finding**: up 12%\")] \
             (literal asterisks) — the distinguishing b=\"1\" assertion in PPTX XML \
             would fail because no Bold node reaches the OOXML engine.",
            items[0][0]
        );

        // Item 1: must be Italic.
        assert!(
            !items[1].is_empty(),
            "AC-001 list-form: item 1 must be non-empty"
        );
        assert!(
            matches!(items[1][0], InlineNode::Italic(_)),
            "AC-001 list-form: item 1 first node must be Italic; got: {:?}",
            items[1][0]
        );

        // Item 2: plain text — no markup, kept as Plain.
        assert!(
            !items[2].is_empty(),
            "AC-001 list-form: item 2 must be non-empty"
        );
        assert!(
            matches!(items[2][0], InlineNode::Plain(_)),
            "AC-001 list-form: item 2 (no markup) must be Plain; got: {:?}",
            items[2][0]
        );
        if let InlineNode::Plain(s) = &items[2][0] {
            assert_eq!(
                s.as_ref(),
                "plain item",
                "AC-001 list-form: plain item text must be preserved verbatim"
            );
        }
    }
}

/// AC-001 (list-form) plain-only regression guard: `bullets: ["A", "B"]` (no markup)
/// must NOT be converted to `FieldValue::InlinesList` — it must stay as
/// `FieldValue::Literal(Value::List([Str("A"), Str("B")]))` (STORY-088 no-regression).
///
/// This guards against over-eager conversion that would break STORY-088's plain behavior.
#[test]
fn test_BC_3_05_001_ac001_list_form_plain_bullets_stay_literal_list() {
    let list_field = FieldValue::List(vec![
        FieldValue::Template(vec![TemplateChunk::Literal("Item A".to_string())]),
        FieldValue::Template(vec![TemplateChunk::Literal("Item B".to_string())]),
    ]);

    let slide_node = SlideNode {
        kind: Spanned::new("content".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("bullets".to_string(), dummy_span()),
            value: Spanned::new(list_field, dummy_span()),
        }],
        inline_items: vec![],
        tags: vec![],
    };
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed for plain list bullets");

    assert!(
        sink.is_empty(),
        "AC-001 plain list regression: must produce 0 diagnostics; got: {:?}",
        sink.errors()
    );

    let field_value = get_field(&result, "bullets").expect("bullets field must be present");

    // Plain bullets → FieldValue::Literal(Value::List) — STORY-088 behavior preserved.
    assert!(
        matches!(
            &field_value,
            TypedFieldValue::Literal(slideforge_types::Value::List(_))
        ),
        "AC-001 plain list regression: bullets without markup must stay \
         FieldValue::Literal(Value::List); got: {field_value:?}"
    );
}

/// EC-007 (list-form): A list item that is `{{ var }}` resolving to `"**bold**"` must
/// produce `InlineNode::Plain("**bold**")` in its per-item nodes — NOT further re-parsed.
///
/// This guards the EC-007 invariant (resolved values are not re-parsed) for the list path.
#[test]
fn test_BC_3_05_001_ec007_list_form_var_resolves_to_asterisks_stays_plain() {
    use slideforge_syntax::Expr;

    // bullets: ["**bold**", {{ var_with_asterisks }}]
    // Item 0 has literal markup (Bold); item 1 resolves "**bold**" from env.
    let list_field = FieldValue::List(vec![
        FieldValue::Template(vec![TemplateChunk::Bold(vec![TemplateChunk::Literal(
            "bold".to_string(),
        )])]),
        // Item 1: {{ var_with_asterisks }} where var = "**bold**" — must stay Plain
        FieldValue::Template(vec![TemplateChunk::Expr(Expr::Ident(
            "var_with_asterisks".to_string(),
        ))]),
    ]);

    let slide_node = SlideNode {
        kind: Spanned::new("content".to_string(), dummy_span()),
        fields: vec![FieldNode {
            name: Spanned::new("bullets".to_string(), dummy_span()),
            value: Spanned::new(list_field, dummy_span()),
        }],
        inline_items: vec![],
        tags: vec![],
    };

    let mut deck_vars = indexmap::IndexMap::new();
    deck_vars.insert(
        Arc::from("var_with_asterisks"),
        slideforge_types::Value::Str(Arc::from("**bold**")),
    );
    let env = Env::new(deck_vars);
    let mut sink = DiagnosticSink::new();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed");

    let field_value = get_field(&result, "bullets").expect("bullets field must be present");

    // Item 0 has markup → result is InlinesList.
    if let TypedFieldValue::InlinesList(items) = &field_value {
        assert_eq!(items.len(), 2, "EC-007 list-form: must have 2 items");

        // Item 0: Bold("bold")
        assert!(
            matches!(items[0][0], InlineNode::Bold(_)),
            "EC-007 list-form: item 0 must be Bold; got: {:?}",
            items[0][0]
        );

        // Item 1: resolved var "**bold**" must be Plain("**bold**"), NOT Bold.
        // EC-007 invariant: resolved values are not re-parsed.
        assert!(
            !items[1].is_empty(),
            "EC-007 list-form: item 1 must be non-empty"
        );
        assert!(
            matches!(&items[1][0], InlineNode::Plain(s) if s.as_ref() == "**bold**"),
            "EC-007 list-form: item 1 resolved from var must be Plain(\"**bold**\"), not Bold; \
             got: {:?}",
            items[1][0]
        );
    } else {
        panic!(
            "EC-007 list-form: expected InlinesList (item 0 has Bold markup); got: {field_value:?}"
        );
    }
}

/// F-P27-MED-001: Test code-span title — `slide_title` must carry `` `Code Title` ``
/// while `stripped_text` is `Code Title`.
#[test]
fn test_P27_MED_001_code_title_diagnostic_has_distinguishing_fields() {
    let chunks = vec![TemplateChunk::Code("Code Title".to_string())];

    let slide_node = make_slide_with_template_field("title", "title", chunks);
    let (env, mut sink) = make_env_and_sink();
    let set_rules: HashMap<(Arc<str>, Arc<str>), slideforge_types::Value> = HashMap::new();

    let _result = eval_slide_node(&env, &slide_node, &set_rules, &mut sink, None)
        .expect("eval_slide_node must succeed");

    assert!(
        sink_has_inline_markup_title_warning(&sink),
        "P27-MED-001: E-EVL-015 must be emitted for code-span title"
    );

    let msg = get_inline_markup_title_diagnostic_message(&sink).expect("E-EVL-015 must be present");

    // Code delimiter is backtick.
    assert!(
        msg.contains('`'),
        "P27-MED-001: E-EVL-015 message for code title must contain '`' (markup delimiter); \
         got: {msg:?}"
    );
    assert!(
        msg.contains("Stripped to: 'Code Title'"),
        "P27-MED-001: stripped_text must be 'Code Title'; got: {msg:?}"
    );
}
