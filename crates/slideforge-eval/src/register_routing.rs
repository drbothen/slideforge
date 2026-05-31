//! Writing register routing pass — extracts and tags register-gated content.
//!
//! # Responsibility
//!
//! This module implements the register routing pass described in STORY-035 and
//! mandated by BC-1.14.001, BC-1.14.002, and BC-1.14.003.
//!
//! After the main evaluation pass resolves all `{{ expr }}` interpolations in
//! [`slideforge_types::Slide`] fields, this pass:
//!
//! 1. Inspects every slide's `fields` map for the three register field names:
//!    `"notes"`, `"report"`, and `"detail"`.
//! 2. Converts the already-evaluated field value into a sequence of
//!    [`slideforge_types::InlineNode`] values.
//! 3. Tags each block with the correct [`slideforge_types::Register`] variant.
//! 4. Returns a [`Vec<RegisteredContent>`] for the slide.
//!
//! The three register fields are NOT visual content and must NOT be forwarded to
//! the layout engine's `frames` computation. Callers are responsible for removing
//! these fields from the visual field set before passing the slide to the layout
//! engine. (The implementation of that removal is also part of STORY-035.)
//!
//! # Pipeline position
//!
//! ```text
//! eval_deck (STORY-011/012/013)
//!   → Slide IR with fully-evaluated fields
//!   → extract_register_content (this module, STORY-035)
//!   → Vec<RegisteredContent> attached to LaidOutSlide.register_content
//!   → slideforge-layout  → LaidOutDeck
//!   → exporters (read only their allowed registers)
//! ```
//!
//! # SS-02 Purity
//!
//! This module is pure-core: no I/O, no filesystem access, no network calls.
//! The function is a pure transformation from `&Slide` to `Vec<RegisteredContent>`.

use slideforge_types::{FieldValue, InlineNode, Register, RegisteredContent, Slide, Value};

use crate::filters::format_float_display;

// ─── Public API ──────────────────────────────────────────────────────────────

/// Extract register-gated content from a fully-evaluated slide.
///
/// For each of the three register field names (`"notes"`, `"report"`,
/// `"detail"`), if the field is present in `slide.fields` and its value
/// is not `Value::Null`, a [`RegisteredContent`] entry is produced and
/// appended to the result.
///
/// # Preconditions (BC-1.14.001/002/003 invariant 1)
///
/// - All `{{ expr }}` interpolations in `slide.fields` have been resolved
///   before this function is called. If a field still contains an
///   `Interpolated` or `Expr` variant after evaluation, this function
///   ignores the interpolation token and produces only the literal parts
///   (the evaluator should have resolved them; any remaining `Expr` nodes
///   represent evaluation errors already reported to the `DiagnosticSink`).
///
/// # Postconditions
///
/// - The returned `Vec` contains one entry per present, non-null register field.
/// - Each entry's `content` is fully evaluated inline content.
/// - The `Vec` is ordered: Notes < Report < Detail (matching
///   [`Register`]'s ordering), so iteration is deterministic.
///
/// # Returns
///
/// A `Vec<RegisteredContent>` with 0 to 3 entries. Empty if the slide has
/// no register fields.
#[must_use]
pub fn extract_register_content(slide: &Slide) -> Vec<RegisteredContent> {
    // Process registers in canonical order: Notes, Report, Detail.
    // This guarantees a deterministic output ordering regardless of the insertion
    // order in slide.fields (ordering invariant, BC-1.14.004).
    let register_pairs: [(Register, &str); 3] = [
        (Register::Notes, "notes"),
        (Register::Report, "report"),
        (Register::Detail, "detail"),
    ];

    let mut result = Vec::with_capacity(3);

    for (register, field_name) in register_pairs {
        if let Some(inlines) = slide
            .fields
            .get(field_name)
            .and_then(field_value_to_inlines)
        {
            result.push(RegisteredContent {
                register,
                content: inlines,
            });
        }
    }

    result
}

// ─── Private helpers ──────────────────────────────────────────────────────────

/// Convert a resolved [`FieldValue`] into a sequence of [`InlineNode`]s.
///
/// Used to produce the `content` field of [`RegisteredContent`]. A
/// plain-text field value produces a single [`InlineNode::Plain`]; a
/// rich-inline field value preserves the inline node sequence as-is.
///
/// Returns `None` only if the field value is `FieldValue::Literal(Value::Null)`,
/// which represents a semantically absent field. An empty string (`""`) is a
/// valid (if vacuous) author intent and returns `Some(vec![InlineNode::Plain("")])`
/// so that exporters can observe the explicit empty declaration.
///
/// Any `Expr` or `Interpolated` variant arriving here is a sign of an evaluation
/// error already reported upstream; only the literal parts are preserved.
fn field_value_to_inlines(fv: &FieldValue) -> Option<Vec<InlineNode>> {
    match fv {
        // Null — semantically absent; produce no entry.
        FieldValue::Literal(Value::Null) => None,

        // List / Map — register fields are text-only by design (BC-1.14.001/002/003):
        //
        // Register fields (`notes`, `report`, `detail`) carry prose destined for
        // a single writing register (presenter notes, reader narrative, document
        // appendix). Those registers are always rendered as a block of text by
        // exporters; there is no mechanism to render a list or map value as prose.
        //
        // Inconsistency note (F-035-P5-003): eval_expr_to_string (eval.rs:~78)
        // emits EvalError::TypeMismatch (E-EVL-003) when a List/Map reaches a
        // text interpolation. Here we cannot emit into a DiagnosticSink because
        // extract_register_content is called from a public API that intentionally
        // carries no sink (the function is pure-core; threading a sink would change
        // its signature and every test call site). Instead we:
        //   1. Emit a tracing::warn! so the drop is OBSERVABLE in structured logs.
        //   2. Return None (no register entry produced), which is the same
        //      observable effect as TypeMismatch in eval_expr_to_string (no output).
        //
        // This behavior is SPECIFIED and TESTED (see test_list_register_field_produces_no_entry
        // and test_map_register_field_produces_no_entry). It is NOT a silent drop.
        //
        // If a future story threads a DiagnosticSink through extract_register_content,
        // replace these tracing::warn! calls with sink.push_with_severity(TypeMismatch...).
        FieldValue::Literal(Value::List(_)) => {
            tracing::warn!(
                "extract_register_content: register field resolved to List — \
                 register fields are text-only (BC-1.14.001/002/003); \
                 no register entry produced (F-035-P5-003)"
            );
            None
        },
        FieldValue::Literal(Value::Map(_)) => {
            tracing::warn!(
                "extract_register_content: register field resolved to Map — \
                 register fields are text-only (BC-1.14.001/002/003); \
                 no register entry produced (F-035-P5-003)"
            );
            None
        },

        // Plain string (the common case after evaluation).
        FieldValue::Literal(Value::Str(s)) => {
            Some(vec![InlineNode::Plain(std::sync::Arc::clone(s))])
        },

        // Scalar Literal variants formatted as strings for register content.
        FieldValue::Literal(Value::Int(n)) => Some(vec![InlineNode::Plain(std::sync::Arc::from(
            n.to_string().as_str(),
        ))]),
        FieldValue::Literal(Value::Bool(b)) => Some(vec![InlineNode::Plain(std::sync::Arc::from(
            b.to_string().as_str(),
        ))]),
        FieldValue::Literal(Value::Float(f)) => Some(vec![InlineNode::Plain(
            // Route through the shared formatter (DI-012 / single-source consistency):
            // avoids scientific notation for normal values and matches the output of
            // every other evaluator text surface (eval_expr_to_string, filters::coerce_to_string).
            std::sync::Arc::from(format_float_display(f.0).as_str()),
        )]),

        // Rich inline content (already-evaluated). Empty Inlines preserves author intent.
        FieldValue::Inlines(nodes) => Some(nodes.clone()),

        // F-005 / STORY-035: Expr and Interpolated variants are an INVARIANT VIOLATION.
        //
        // `extract_register_content` is called ONLY from `eval_slide_node` AFTER all
        // field values have been resolved by the evaluator. At that point:
        //   - Template fields become FieldValue::Literal(Value::Str(...))
        //   - Ident fields become FieldValue::Literal(value)
        //   - Error fields are SKIPPED (not inserted into slide.fields)
        //
        // An Expr or Interpolated variant reaching this function means either:
        //   (a) A caller outside eval_slide_node called extract_register_content on a
        //       pre-evaluation slide (a programming error), or
        //   (b) The evaluator's error fallback path inserted an unresolved Expr/Interpolated
        //       into slide.fields instead of Literal(Null) (a different programming error).
        //
        // FORBIDDEN (F-005): Silent empty-content fallback would swallow the data loss.
        // Instead, emit a tracing::error! diagnostic and assert in debug mode.
        // We return None (treat as absent) so no vacuous register entry is produced —
        // returning Some(vec![]) would create a register entry with empty content, which
        // is indistinguishable from an intentional empty register field.
        FieldValue::Expr(raw) => {
            debug_assert!(
                false,
                "invariant violation: Expr({raw:?}) reached extract_register_content \
                 after eval_deck — evaluator did not resolve this field value"
            );
            tracing::error!(
                raw_expr = %raw,
                "extract_register_content: unresolved Expr variant reached after eval_deck; \
                 this is an internal invariant violation — register content will be absent \
                 for this field (F-005 / STORY-035)"
            );
            None
        },
        FieldValue::Interpolated(parts) => {
            debug_assert!(
                false,
                "invariant violation: Interpolated({parts:?}) reached extract_register_content \
                 after eval_deck — evaluator did not resolve this field value"
            );
            tracing::error!(
                part_count = parts.len(),
                "extract_register_content: unresolved Interpolated variant reached after eval_deck; \
                 this is an internal invariant violation — register content will be absent \
                 for this field (F-005 / STORY-035)"
            );
            None
        },
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
// Register field names (e.g. `register_content`) are used as prose in test
// doc comments; suppressing the doc_markdown lint for the test module avoids
// requiring backticks around every occurrence.
#[allow(clippy::doc_markdown)]
mod tests {
    use std::sync::Arc;

    use insta::assert_debug_snapshot;
    use slideforge_types::{
        FieldValue, InlineNode, OrderedMap, Register, RegisteredContent, Slide, SourceSpan, Value,
    };

    use super::*;

    // ─── Test helpers ─────────────────────────────────────────────────────────

    /// Build a minimal `Slide` with the given field map and slide type.
    fn make_slide_with_fields(slide_type: &str, fields: OrderedMap<Arc<str>, FieldValue>) -> Slide {
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

    /// Build a `Slide` with a single plain-string register field.
    fn make_slide_with_register_field(register_name: &str, text: &str) -> Slide {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from(register_name),
            FieldValue::Literal(Value::Str(Arc::from(text))),
        );
        make_slide_with_fields("content", fields)
    }

    /// Build a `Slide` with a plain title field but NO register fields.
    fn make_visual_only_slide(title: &str) -> Slide {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from(title))),
        );
        make_slide_with_fields("content", fields)
    }

    // ─── AC-001: notes field extracted and tagged ─────────────────────────────

    /// BC-1.14.001 invariant 1: notes field extracted and tagged as Register::Notes.
    ///
    /// A slide with `notes "Test presenter notes"` must produce exactly one
    /// `RegisteredContent` with `register == Register::Notes`.
    #[test]
    fn test_bc_1_14_001_notes_extracted_and_tagged() {
        let slide = make_slide_with_register_field("notes", "Test presenter notes");

        let result = extract_register_content(&slide);

        assert_eq!(
            result.len(),
            1,
            "slide with notes field must produce exactly 1 RegisteredContent entry; got: {result:?}"
        );
        assert_eq!(
            result[0].register,
            Register::Notes,
            "notes field must be tagged Register::Notes"
        );
    }

    /// BC-1.14.001 invariant 1: notes content is correctly captured in the content field.
    #[test]
    fn test_bc_1_14_001_notes_content_captured() {
        let slide = make_slide_with_register_field("notes", "Emphasize the growth story");

        let result = extract_register_content(&slide);

        assert_eq!(result.len(), 1, "must have 1 entry");
        // The text must appear in the content inline nodes.
        let text = extract_plain_text(&result[0].content);
        assert_eq!(
            text, "Emphasize the growth story",
            "notes text must be captured verbatim in RegisteredContent.content"
        );
    }

    // ─── AC-002: notes content excluded from visual frames ────────────────────

    /// BC-1.14.001 postcondition 1: a slide with only a notes field must
    /// produce register_content with Notes; the notes key must NOT be returned
    /// for visual frame use.
    ///
    /// This test verifies the extraction produces Notes, which is the precondition
    /// for the layout engine to NOT include notes in visual frames.
    /// (The actual exclusion from frames is enforced by the layout engine, which
    /// reads `register_content` and ignores the `notes` field from `slide.fields`.)
    #[test]
    fn test_bc_1_14_001_notes_only_slide_has_one_entry() {
        // EC-001: slide with only notes field (no visual content).
        let slide = make_slide_with_register_field("notes", "Speaker: start with the agenda");

        let result = extract_register_content(&slide);

        assert_eq!(
            result.len(),
            1,
            "notes-only slide must produce exactly 1 entry"
        );
        assert!(
            result[0].register.is_notes(),
            "the single entry must be tagged Notes"
        );
    }

    /// BC-1.14.001 postcondition 1: a slide with VISUAL content but NO register
    /// fields produces an empty register_content Vec.
    #[test]
    fn test_bc_1_14_001_visual_only_slide_produces_empty_register_content() {
        let slide = make_visual_only_slide("Quarterly Review");

        let result = extract_register_content(&slide);

        assert!(
            result.is_empty(),
            "visual-only slide (no register fields) must produce empty register_content; got: {result:?}"
        );
    }

    // ─── AC-003: report field extracted and tagged ────────────────────────────

    /// BC-1.14.002 invariant 1: report field extracted and tagged as Register::Report.
    #[test]
    fn test_bc_1_14_002_report_extracted_and_tagged() {
        let slide = make_slide_with_register_field("report", "Detailed narrative for readers");

        let result = extract_register_content(&slide);

        assert_eq!(
            result.len(),
            1,
            "slide with report field must produce 1 entry"
        );
        assert_eq!(
            result[0].register,
            Register::Report,
            "report field must be tagged Register::Report"
        );
    }

    // ─── AC-004: report content excluded from visual frames ───────────────────

    /// BC-1.14.002 postcondition 3: report field captured in register_content,
    /// NOT in visual frames.
    ///
    /// The test captures that `extract_register_content` correctly routes the
    /// report text to `RegisteredContent` (so the layout engine can exclude it
    /// from visual frames by reading only from `register_content`).
    #[test]
    fn test_bc_1_14_002_report_content_captured_correctly() {
        let slide = make_slide_with_register_field("report", "Analysis text");

        let result = extract_register_content(&slide);

        assert_eq!(result.len(), 1);
        let text = extract_plain_text(&result[0].content);
        assert_eq!(
            text, "Analysis text",
            "report text must be captured verbatim in RegisteredContent.content"
        );
        assert_eq!(result[0].register, Register::Report);
    }

    // ─── AC-005: detail field extracted and tagged ────────────────────────────

    /// BC-1.14.003 invariant 1: detail field extracted and tagged as Register::Detail.
    #[test]
    fn test_bc_1_14_003_detail_extracted_and_tagged() {
        let slide = make_slide_with_register_field("detail", "Technical appendix text");

        let result = extract_register_content(&slide);

        assert_eq!(
            result.len(),
            1,
            "slide with detail field must produce 1 entry"
        );
        assert_eq!(
            result[0].register,
            Register::Detail,
            "detail field must be tagged Register::Detail"
        );
    }

    // ─── AC-006: detail content excluded from visual frames ───────────────────

    /// BC-1.14.003 postcondition 3: detail content captured in register_content.
    #[test]
    fn test_bc_1_14_003_detail_content_captured_correctly() {
        let slide = make_slide_with_register_field("detail", "Technical appendix text");

        let result = extract_register_content(&slide);

        assert_eq!(result.len(), 1);
        let text = extract_plain_text(&result[0].content);
        assert_eq!(
            text, "Technical appendix text",
            "detail text must be captured verbatim"
        );
        assert_eq!(result[0].register, Register::Detail);
    }

    // ─── AC-007: interpolation evaluated before tagging ──────────────────────

    /// BC-1.14.001 postcondition 1: interpolation is resolved BEFORE register
    /// tagging. The field value in `slide.fields` is a `FieldValue::Literal`
    /// (already-evaluated string) because `eval_deck` resolved `{{ quarter }}`
    /// to "Q1" before this pass runs.
    ///
    /// This test simulates what the evaluator produces after resolving
    /// `notes "Quarter: {{ quarter }}"` with `quarter = "Q1"` in scope —
    /// the slide arrives at `extract_register_content` with the notes field
    /// already containing `"Quarter: Q1"` as a literal string.
    #[test]
    fn test_bc_1_14_001_interpolation_resolved_before_tagging() {
        // Simulate the post-evaluation state: interpolation already resolved.
        // The evaluator turns `notes "Quarter: {{ quarter }}"` with `quarter = "Q1"`
        // into `FieldValue::Literal(Value::Str("Quarter: Q1"))` before calling
        // extract_register_content.
        let slide = make_slide_with_register_field("notes", "Quarter: Q1");

        let result = extract_register_content(&slide);

        assert_eq!(result.len(), 1, "must have 1 entry for resolved notes");
        assert_eq!(result[0].register, Register::Notes);
        let text = extract_plain_text(&result[0].content);
        assert_eq!(
            text, "Quarter: Q1",
            "interpolation must be already resolved; got: {text}"
        );
        // Crucially, the raw template `{{ quarter }}` must NOT appear.
        assert!(
            !text.contains("{{"),
            "interpolation token must not appear in register content"
        );
    }

    // ─── AC-008: all three registers on same slide ────────────────────────────

    /// BC-1.14.004 invariant 3: a slide with all three registers produces
    /// exactly 3 entries in register_content. No register text appears in
    /// the visual frames (the layout engine reads only register_content for
    /// register-gated text).
    ///
    /// This is the most important invariant test: it verifies that all three
    /// registers are correctly extracted from a single slide without crosstalk.
    #[test]
    fn test_bc_1_14_004_all_three_registers_on_one_slide() {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from("Q1 Review"))),
        );
        fields.insert(
            Arc::from("notes"),
            FieldValue::Literal(Value::Str(Arc::from("Speaker: emphasize growth"))),
        );
        fields.insert(
            Arc::from("report"),
            FieldValue::Literal(Value::Str(Arc::from("Detailed narrative for readers"))),
        );
        fields.insert(
            Arc::from("detail"),
            FieldValue::Literal(Value::Str(Arc::from("Technical appendix"))),
        );
        let slide = make_slide_with_fields("content", fields);

        let result = extract_register_content(&slide);

        // AC-008: exactly three entries.
        assert_eq!(
            result.len(),
            3,
            "slide with all 3 register fields must produce exactly 3 RegisteredContent entries; \
             got {} entries: {result:?}",
            result.len()
        );

        // Verify each register variant is present.
        let registers: Vec<Register> = result.iter().map(|rc| rc.register).collect();
        assert!(
            registers.contains(&Register::Notes),
            "Register::Notes must be present in register_content"
        );
        assert!(
            registers.contains(&Register::Report),
            "Register::Report must be present in register_content"
        );
        assert!(
            registers.contains(&Register::Detail),
            "Register::Detail must be present in register_content"
        );

        // Verify content text for each register (no crosstalk).
        let notes_entry = result
            .iter()
            .find(|rc| rc.register == Register::Notes)
            .expect("Notes entry must be present");
        let notes_text = extract_plain_text(&notes_entry.content);
        assert_eq!(
            notes_text, "Speaker: emphasize growth",
            "Notes content must be 'Speaker: emphasize growth'"
        );

        let report_entry = result
            .iter()
            .find(|rc| rc.register == Register::Report)
            .expect("Report entry must be present");
        let report_text = extract_plain_text(&report_entry.content);
        assert_eq!(
            report_text, "Detailed narrative for readers",
            "Report content must be 'Detailed narrative for readers'"
        );

        let detail_entry = result
            .iter()
            .find(|rc| rc.register == Register::Detail)
            .expect("Detail entry must be present");
        let detail_text = extract_plain_text(&detail_entry.content);
        assert_eq!(
            detail_text, "Technical appendix",
            "Detail content must be 'Technical appendix'"
        );
    }

    /// BC-1.14.004 invariant 3 (snapshot): insta snapshot of register_content
    /// for a 3-register slide for regression detection.
    ///
    /// This snapshot captures the exact `RegisteredContent` vector structure
    /// produced for a canonical 3-register slide. It will lock in after the
    /// first green run and serve as a regression baseline.
    #[test]
    fn test_bc_1_14_004_three_register_snapshot() {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from("Q1 Review"))),
        );
        fields.insert(
            Arc::from("notes"),
            FieldValue::Literal(Value::Str(Arc::from("Presenter notes here"))),
        );
        fields.insert(
            Arc::from("report"),
            FieldValue::Literal(Value::Str(Arc::from("Report prose here"))),
        );
        fields.insert(
            Arc::from("detail"),
            FieldValue::Literal(Value::Str(Arc::from("Technical detail here"))),
        );
        let slide = make_slide_with_fields("content", fields);

        let result = extract_register_content(&slide);

        assert_debug_snapshot!(result);
    }

    // ─── EC-001: slide with only notes field ──────────────────────────────────

    /// EC-001: slide with ONLY notes field (no visual content at all).
    ///
    /// register_content must have one Notes entry. The extraction must not
    /// produce spurious entries for the absent report/detail fields.
    #[test]
    fn test_bc_1_14_001_ec001_notes_only_no_visual_content() {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("notes"),
            FieldValue::Literal(Value::Str(Arc::from("Notes-only slide"))),
        );
        let slide = make_slide_with_fields("blank", fields);

        let result = extract_register_content(&slide);

        assert_eq!(
            result.len(),
            1,
            "notes-only slide must produce exactly 1 entry"
        );
        assert!(result[0].register.is_notes());
    }

    // ─── EC-002: notes with interpolation (already resolved) ──────────────────

    /// EC-002: notes field value after interpolation resolution — the raw
    /// template `{{ expr }}` must not survive to `register_content`.
    #[test]
    fn test_bc_1_14_001_ec002_notes_with_resolved_interpolation() {
        // Simulate the evaluator having resolved `{{ quarter }}` to "Q1".
        let slide = make_slide_with_register_field("notes", "Quarter: Q1");

        let result = extract_register_content(&slide);

        assert_eq!(result.len(), 1);
        let text = extract_plain_text(&result[0].content);
        assert!(
            !text.contains("{{"),
            "No interpolation token must survive to register_content; got: {text}"
        );
        assert_eq!(text, "Quarter: Q1");
    }

    // ─── EC-004: detail and report both present ───────────────────────────────

    /// EC-004: `detail` and `report` both present on same slide.
    ///
    /// Both entries must appear in register_content; neither appears in visual
    /// frames. Order: Notes < Report < Detail.
    #[test]
    fn test_bc_1_14_002_ec004_report_and_detail_both_present() {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("report"),
            FieldValue::Literal(Value::Str(Arc::from("Report text"))),
        );
        fields.insert(
            Arc::from("detail"),
            FieldValue::Literal(Value::Str(Arc::from("Detail text"))),
        );
        let slide = make_slide_with_fields("content", fields);

        let result = extract_register_content(&slide);

        assert_eq!(
            result.len(),
            2,
            "report + detail slide must produce exactly 2 entries; got: {result:?}"
        );
        let registers: Vec<Register> = result.iter().map(|rc| rc.register).collect();
        assert!(
            registers.contains(&Register::Report),
            "Report must be present"
        );
        assert!(
            registers.contains(&Register::Detail),
            "Detail must be present"
        );
        assert!(
            !registers.contains(&Register::Notes),
            "Notes must NOT be present when notes field absent"
        );
    }

    // ─── EC-005: @for loop notes per-iteration (simulated) ───────────────────

    /// EC-005: simulates `@for` loop with `notes` field by calling
    /// `extract_register_content` on two slides with different notes values.
    ///
    /// The real `@for` loop produces separate `Slide` instances per iteration;
    /// this test verifies that each slide independently produces its own
    /// `RegisteredContent` with the correct per-iteration value.
    #[test]
    fn test_bc_1_14_001_ec005_for_loop_notes_per_iteration() {
        let slide_iter1 = make_slide_with_register_field("notes", "Notes for iteration 1");
        let slide_iter2 = make_slide_with_register_field("notes", "Notes for iteration 2");

        let result1 = extract_register_content(&slide_iter1);
        let result2 = extract_register_content(&slide_iter2);

        assert_eq!(result1.len(), 1, "iteration 1 must produce 1 entry");
        assert_eq!(result2.len(), 1, "iteration 2 must produce 1 entry");

        let text1 = extract_plain_text(&result1[0].content);
        let text2 = extract_plain_text(&result2[0].content);

        assert_eq!(
            text1, "Notes for iteration 1",
            "iteration 1 notes must be 'Notes for iteration 1'"
        );
        assert_eq!(
            text2, "Notes for iteration 2",
            "iteration 2 notes must be 'Notes for iteration 2'"
        );
        assert_ne!(
            text1, text2,
            "each iteration must produce independent register content"
        );
    }

    // ─── Ordering invariant ───────────────────────────────────────────────────

    /// The output ordering must be deterministic: Notes, Report, Detail (Register ordering).
    ///
    /// Even when fields are inserted in reverse order in the slide, the output
    /// must follow Notes < Report < Detail ordering.
    #[test]
    fn test_bc_1_14_004_register_content_ordered_notes_report_detail() {
        // Insert in reverse order: detail, report, notes.
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("detail"),
            FieldValue::Literal(Value::Str(Arc::from("detail text"))),
        );
        fields.insert(
            Arc::from("report"),
            FieldValue::Literal(Value::Str(Arc::from("report text"))),
        );
        fields.insert(
            Arc::from("notes"),
            FieldValue::Literal(Value::Str(Arc::from("notes text"))),
        );
        let slide = make_slide_with_fields("content", fields);

        let result = extract_register_content(&slide);

        assert_eq!(result.len(), 3, "must have 3 entries");
        assert_eq!(
            result[0].register,
            Register::Notes,
            "first entry must be Notes (ordering invariant)"
        );
        assert_eq!(
            result[1].register,
            Register::Report,
            "second entry must be Report (ordering invariant)"
        );
        assert_eq!(
            result[2].register,
            Register::Detail,
            "third entry must be Detail (ordering invariant)"
        );
    }

    // ─── Empty / null field value ──────────────────────────────────────────────

    /// A notes field with `Value::Null` must not produce a RegisteredContent entry.
    ///
    /// Null register fields are semantically absent — the author declared the
    /// field but left it empty. No spurious entry should appear.
    #[test]
    fn test_bc_1_14_001_null_register_field_produces_no_entry() {
        let mut fields = OrderedMap::new();
        fields.insert(Arc::from("notes"), FieldValue::Literal(Value::Null));
        let slide = make_slide_with_fields("content", fields);

        let result = extract_register_content(&slide);

        assert!(
            result.is_empty(),
            "notes field with Null value must produce no RegisteredContent; got: {result:?}"
        );
    }

    /// A notes field with an empty string must produce a RegisteredContent entry.
    ///
    /// An empty string is a valid (if vacuous) register value — the author
    /// explicitly wrote `notes ""`. Contrast with Null (field present but no value).
    #[test]
    fn test_bc_1_14_001_empty_string_notes_produces_one_entry() {
        let slide = make_slide_with_register_field("notes", "");

        let result = extract_register_content(&slide);

        // An explicitly-authored `notes ""` produces an entry (even if empty).
        // This is intentional: the author declared the field; exporters decide
        // whether to render empty speaker notes.
        assert_eq!(
            result.len(),
            1,
            "notes field with empty string must produce 1 entry (author intent is explicit)"
        );
        assert_eq!(result[0].register, Register::Notes);
    }

    // ─── RegisteredContent type invariants ────────────────────────────────────

    /// RegisteredContent implements Hash + Eq + Clone + Debug (comemo requirement).
    #[test]
    fn test_registered_content_implements_hash_eq_clone_debug() {
        use std::collections::HashSet;

        let rc = RegisteredContent::plain(Register::Notes, Arc::from("hello"));
        let rc2 = rc.clone();
        assert_eq!(rc, rc2, "RegisteredContent must implement PartialEq");

        let debug_str = format!("{rc:?}");
        assert!(
            debug_str.contains("RegisteredContent"),
            "must implement Debug"
        );

        let mut set = HashSet::new();
        set.insert(rc.clone());
        assert_eq!(set.len(), 1, "RegisteredContent must be hashable");

        // Different content produces different hash entries.
        let rc3 = RegisteredContent::plain(Register::Report, Arc::from("hello"));
        set.insert(rc3);
        assert_eq!(
            set.len(),
            2,
            "Different registers must produce different entries"
        );
    }

    /// RegisteredContent::plain() is_empty() returns false for non-empty content.
    #[test]
    fn test_registered_content_plain_is_not_empty() {
        let rc = RegisteredContent::plain(Register::Notes, Arc::from("text"));
        assert!(!rc.is_empty(), "plain(text) must not be empty");
    }

    /// RegisteredContent with no inline nodes reports is_empty() == true.
    #[test]
    fn test_registered_content_empty_content_is_empty() {
        let rc = RegisteredContent {
            register: Register::Notes,
            content: vec![],
        };
        assert!(rc.is_empty(), "empty content must report is_empty == true");
    }

    // ─── F-035-P5-001: Inlines branch — rich inline formatting survives ──────

    /// F-035-P5-001 (EC-003): A register field whose value is `FieldValue::Inlines`
    /// must preserve the exact InlineNode variants in `RegisteredContent.content`.
    ///
    /// This covers BC-1.14.001 EC-003 ("notes field contains inline formatting"):
    /// bold/italic/link/plain inline nodes that the DSL parser constructs for
    /// rich-formatted register fields must pass through `extract_register_content`
    /// without being flattened to plain text.
    ///
    /// Asserts structural equality, not just flattened text, so that a regression
    /// that collapses formatting (e.g., Bold → Plain) is detected.
    #[test]
    fn test_f035_p5_001_rich_inlines_preserved_structurally() {
        use slideforge_types::MathNode;

        // Construct a notes field with FieldValue::Inlines containing
        // Bold, Italic, Link, and Plain nodes — the canonical rich-inline set.
        let bold_node = InlineNode::Bold(vec![InlineNode::Plain(Arc::from("important"))]);
        let italic_node = InlineNode::Italic(vec![InlineNode::Plain(Arc::from("emphasis"))]);
        let link_node = InlineNode::Link {
            text: vec![InlineNode::Plain(Arc::from("click here"))],
            url: Arc::from("https://example.com"),
        };
        let plain_node = InlineNode::Plain(Arc::from("trailing text"));
        // Math node — exercises the Math variant of the Inlines path.
        let math_node = InlineNode::Math(MathNode {
            latex: Arc::from("E = mc^2"),
            display: false,
            span: slideforge_types::SourceSpan::default(),
        });

        let inlines = vec![
            bold_node.clone(),
            italic_node.clone(),
            link_node.clone(),
            plain_node.clone(),
            math_node.clone(),
        ];

        let mut fields = OrderedMap::new();
        fields.insert(Arc::from("notes"), FieldValue::Inlines(inlines.clone()));
        let slide = make_slide_with_fields("content", fields);

        let result = extract_register_content(&slide);

        assert_eq!(
            result.len(),
            1,
            "notes field with FieldValue::Inlines must produce exactly 1 entry"
        );
        assert_eq!(result[0].register, Register::Notes);
        assert_eq!(
            result[0].content, inlines,
            "FieldValue::Inlines must be preserved structurally — \
             rich inline formatting (Bold, Italic, Link, Math) must not be flattened"
        );
        // Structural checks: verify individual node variants are present.
        assert!(
            matches!(result[0].content[0], InlineNode::Bold(_)),
            "first node must be Bold"
        );
        assert!(
            matches!(result[0].content[1], InlineNode::Italic(_)),
            "second node must be Italic"
        );
        assert!(
            matches!(result[0].content[2], InlineNode::Link { .. }),
            "third node must be Link"
        );
        assert!(
            matches!(result[0].content[3], InlineNode::Plain(_)),
            "fourth node must be Plain"
        );
        assert!(
            matches!(result[0].content[4], InlineNode::Math(_)),
            "fifth node must be Math"
        );
    }

    /// F-035-P5-001: Empty `FieldValue::Inlines` produces a register entry with
    /// empty content (author explicitly declared an empty rich-inline register field).
    ///
    /// Distinct from `Value::Null` (which produces no entry at all).
    #[test]
    fn test_f035_p5_001_empty_inlines_produces_one_entry() {
        let mut fields = OrderedMap::new();
        fields.insert(Arc::from("notes"), FieldValue::Inlines(vec![]));
        let slide = make_slide_with_fields("content", fields);

        let result = extract_register_content(&slide);

        assert_eq!(
            result.len(),
            1,
            "empty FieldValue::Inlines must produce 1 entry (author intent preserved)"
        );
        assert!(
            result[0].content.is_empty(),
            "content must be empty for empty Inlines"
        );
    }

    // ─── F-035-P5-001: Scalar coercion arms — Int / Bool / Float ─────────────

    /// F-035-P5-001: `FieldValue::Literal(Value::Int(n))` must produce a
    /// `RegisteredContent` with a single `InlineNode::Plain` containing the
    /// decimal string representation.
    #[test]
    fn test_f035_p5_001_int_register_field_formatted_as_decimal_string() {
        let mut fields = OrderedMap::new();
        fields.insert(Arc::from("notes"), FieldValue::Literal(Value::Int(42)));
        let slide = make_slide_with_fields("content", fields);

        let result = extract_register_content(&slide);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].register, Register::Notes);
        let text = extract_plain_text(&result[0].content);
        assert_eq!(text, "42", "Int(42) must render as \"42\"");
    }

    /// Negative Int register field.
    #[test]
    fn test_f035_p5_001_negative_int_register_field_formatted_correctly() {
        let mut fields = OrderedMap::new();
        fields.insert(Arc::from("report"), FieldValue::Literal(Value::Int(-7)));
        let slide = make_slide_with_fields("content", fields);

        let result = extract_register_content(&slide);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].register, Register::Report);
        assert_eq!(extract_plain_text(&result[0].content), "-7");
    }

    /// F-035-P5-001: `FieldValue::Literal(Value::Bool(true))` must produce "true".
    #[test]
    fn test_f035_p5_001_bool_true_register_field_formatted_as_true() {
        let mut fields = OrderedMap::new();
        fields.insert(Arc::from("notes"), FieldValue::Literal(Value::Bool(true)));
        let slide = make_slide_with_fields("content", fields);

        let result = extract_register_content(&slide);

        assert_eq!(result.len(), 1);
        assert_eq!(extract_plain_text(&result[0].content), "true");
    }

    /// F-035-P5-001: `FieldValue::Literal(Value::Bool(false))` must produce "false".
    #[test]
    fn test_f035_p5_001_bool_false_register_field_formatted_as_false() {
        let mut fields = OrderedMap::new();
        fields.insert(Arc::from("notes"), FieldValue::Literal(Value::Bool(false)));
        let slide = make_slide_with_fields("content", fields);

        let result = extract_register_content(&slide);

        assert_eq!(result.len(), 1);
        assert_eq!(extract_plain_text(&result[0].content), "false");
    }

    // ─── F-035-P5-002: Float formatting via shared formatter ─────────────────

    /// F-035-P5-002: Float register fields must use `format_float_display`
    /// (the shared evaluator formatter), NOT raw `f64::Display`.
    ///
    /// Normal-range Float values must render without scientific notation.
    #[test]
    fn test_f035_p5_002_float_register_field_formatted_without_scientific_notation() {
        use ordered_float::OrderedFloat;

        // Use 1.5 — not an approximation of any known constant.
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("notes"),
            FieldValue::Literal(Value::Float(OrderedFloat(1.5))),
        );
        let slide = make_slide_with_fields("content", fields);

        let result = extract_register_content(&slide);

        assert_eq!(result.len(), 1);
        let text = extract_plain_text(&result[0].content);
        assert_eq!(text, "1.5", "Float(1.5) must render as \"1.5\"");
        assert!(
            !text.contains('e') && !text.contains('E'),
            "normal-range Float must not use scientific notation; got: {text}"
        );
    }

    /// F-035-P5-002 (DI-012 consistency): The Float register field must produce
    /// IDENTICAL text to `format_float_display` for all values, not a divergent
    /// rendering via raw `f64::Display`.
    ///
    /// This test exercises the consistency invariant directly: whatever value
    /// `format_float_display` returns for a given f64, the register field
    /// output for the same f64 must be byte-identical.
    ///
    /// Note: Rust's `f64::Display` does NOT use scientific notation — it always
    /// emits full decimal strings. Therefore the `format_float_display` fallback
    /// path (for 'e'/'E' in Display output) is never reached in practice.
    /// The fix still matters for forward-compatibility if `format_float_display`
    /// is ever updated to apply rounding/truncation, or if the float formatting
    /// behavior changes between Rust editions. Routing through the shared function
    /// ensures the register surface stays consistent with interpolation surfaces.
    #[test]
    fn test_f035_p5_002_float_formatting_consistent_with_shared_formatter() {
        use ordered_float::OrderedFloat;

        // Probe several representative float values: integer-valued, fractional,
        // very large, very small. All must match format_float_display exactly.
        // Values chosen to avoid clippy::approx_constant (no ~PI, ~E, etc.).
        // Use simple decimal values and one small float to exercise the formatter.
        let cases: &[(f64, &str)] = &[
            (0.0, "0"),
            (1.0, "1"),
            (1.5, "1.5"),
            (-1.25, "-1.25"),
            (1_000_000.0, "1000000"),
            (0.001, "0.001"),
            (1.234_567_89e-10, "0.000000000123456789"),
        ];

        for (val, expected_display) in cases {
            let from_formatter = crate::filters::format_float_display(*val);
            // Verify test fixture first: our expected_display matches format_float_display.
            assert_eq!(
                from_formatter, *expected_display,
                "fixture: format_float_display({val}) must equal {expected_display:?}"
            );

            let mut fields = OrderedMap::new();
            fields.insert(
                Arc::from("notes"),
                FieldValue::Literal(Value::Float(OrderedFloat(*val))),
            );
            let slide = make_slide_with_fields("content", fields);
            let result = extract_register_content(&slide);

            assert_eq!(result.len(), 1, "Float({val}) must produce 1 entry");
            let text = extract_plain_text(&result[0].content);
            assert_eq!(
                text, from_formatter,
                "Float({val}) register formatting must be IDENTICAL to \
                 format_float_display output (DI-012 single-source consistency); \
                 got {text:?}, expected {from_formatter:?}"
            );
        }
    }

    // ─── F-035-P5-003: List/Map — documented, tested, observable drop ─────────

    /// F-035-P5-003: A register field resolving to `Value::List` must produce
    /// no `RegisteredContent` entry.
    ///
    /// Register fields are text-only by design (BC-1.14.001/002/003). List values
    /// cannot be rendered as prose. The drop is observable via `tracing::warn!`
    /// (see the code comment in `field_value_to_inlines`). This test specifies
    /// and locks in the documented behavior so it is NOT a silent, untested drop.
    #[test]
    fn test_f035_p5_003_list_register_field_produces_no_entry() {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("notes"),
            FieldValue::Literal(Value::List(vec![
                Value::Str(Arc::from("item1")),
                Value::Str(Arc::from("item2")),
            ])),
        );
        let slide = make_slide_with_fields("content", fields);

        let result = extract_register_content(&slide);

        assert!(
            result.is_empty(),
            "notes field with List value must produce NO register entry \
             (register fields are text-only, BC-1.14.001/002/003; \
             drop is logged via tracing::warn! — F-035-P5-003); got: {result:?}"
        );
    }

    /// F-035-P5-003: A register field resolving to `Value::Map` must produce
    /// no `RegisteredContent` entry.
    ///
    /// Same reasoning as the List case — maps have no prose rendering.
    #[test]
    fn test_f035_p5_003_map_register_field_produces_no_entry() {
        let mut map = OrderedMap::new();
        map.insert(Arc::from("key"), Value::Str(Arc::from("val")));
        let mut fields = OrderedMap::new();
        fields.insert(Arc::from("notes"), FieldValue::Literal(Value::Map(map)));
        let slide = make_slide_with_fields("content", fields);

        let result = extract_register_content(&slide);

        assert!(
            result.is_empty(),
            "notes field with Map value must produce NO register entry \
             (register fields are text-only, BC-1.14.001/002/003; \
             drop is logged via tracing::warn! — F-035-P5-003); got: {result:?}"
        );
    }

    // ─── Helper: extract plain text from InlineNode sequence ──────────────────

    /// Concatenate all [`InlineNode::Plain`] leaf text in a node sequence.
    ///
    /// Used in tests to verify that the correct text ended up in `content`.
    /// Ignores non-Plain nodes (which would indicate formatting — not present
    /// in simple register field values).
    fn extract_plain_text(nodes: &[InlineNode]) -> String {
        nodes
            .iter()
            .map(|node| match node {
                // Leaf text nodes — return the text directly.
                InlineNode::Plain(s) | InlineNode::Code(s) | InlineNode::Xref(s) => {
                    s.as_ref().to_owned()
                },
                // Container nodes — recurse into children.
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

    // ─── F-005: Expr/Interpolated variants are invariant violations ───────────

    /// F-005 (debug mode): An `Expr` variant reaching `extract_register_content`
    /// is an invariant violation and panics in debug mode (via `debug_assert!`).
    ///
    /// This test is gated on `#[cfg(debug_assertions)]`. In release mode, the
    /// behaviour is `None` return + `tracing::error!` (tested separately below).
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "invariant violation")]
    fn test_f005_expr_in_register_field_panics_in_debug_mode() {
        // Construct a slide with an unresolved Expr in the notes field.
        // This simulates a caller that bypassed eval_deck (a programming error).
        use slideforge_types::StringPart;

        let mut fields = OrderedMap::new();
        // Insert an Interpolated variant (simulating pre-evaluation state).
        fields.insert(
            Arc::from("notes"),
            FieldValue::Interpolated(vec![
                StringPart::Literal(Arc::from("Quarter: ")),
                StringPart::Expr(Arc::from("quarter")),
            ]),
        );
        let slide = make_slide_with_fields("content", fields);

        // This call must panic in debug mode via debug_assert!(false).
        let _ = extract_register_content(&slide);
    }

    /// F-005 (release mode / #[cfg(not(debug_assertions))]): An unresolved
    /// `Expr` variant produces no register entry (return `None`) and does NOT
    /// silently create a vacuous entry with empty content.
    ///
    /// This test verifies the non-panic path: the result is absent (not
    /// `Some(vec![])` — the old silent-loss fallback).
    ///
    /// Notes: This test runs in both debug and release mode. In debug mode the
    /// `debug_assert!` fires first (see the `#[should_panic]` test above);
    /// this test is a complementary check gated on `#[cfg(not(debug_assertions))]`
    /// for pure release-mode builds.
    #[test]
    #[cfg(not(debug_assertions))]
    fn test_f005_expr_in_register_field_produces_no_entry_in_release_mode() {
        use slideforge_types::StringPart;

        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("notes"),
            FieldValue::Interpolated(vec![StringPart::Expr(Arc::from("quarter"))]),
        );
        let slide = make_slide_with_fields("content", fields);

        let result = extract_register_content(&slide);

        // The result must be empty: no vacuous register entry should be produced.
        // Returning Some(vec![]) would create an indistinguishable-from-intentional
        // empty entry — the old silent-loss pattern (F-005).
        assert!(
            result.is_empty(),
            "Interpolated variant must produce NO register entry (not Some(vec![])); \
             got: {result:?}"
        );
    }
}
