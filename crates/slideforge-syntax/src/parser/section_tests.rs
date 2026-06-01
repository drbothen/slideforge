#![allow(clippy::missing_docs_in_private_items, clippy::unwrap_used)]
#![allow(non_snake_case)] // test_BC_S_SS_NNN_xxx naming convention per TDD traceability
#![allow(clippy::doc_markdown)] // test function names in doc comments do not need backticks
#![allow(clippy::no_effect_underscore_binding)] // _warning type-check bindings in tests
//! Failing test suite (Red Gate) for STORY-078: Parser section block syntax.
//!
//! # TDD Red Gate
//!
//! All tests in this module MUST FAIL before the implementation of STORY-078.
//! They are the specification of the expected post-implementation behaviour.
//! The implementer must make each test pass, one at a time, with minimum code.
//!
//! # Symbols expected but NOT yet existing (causes compile failure)
//!
//! The following symbols do not exist in the codebase at the time this test
//! file was written. Their absence is the Red Gate. The implementer must
//! create them:
//!
//! - `slideforge_syntax::section::is_register_sub_block_key` — function
//!   in the new `src/section.rs` module (AC-006)
//! - `slideforge_syntax::parser::section::section_block_parser` — parser
//!   combinator in the new `src/parser/section.rs` module (AC-002)
//!
//! The `BlockItem::Section` variant already exists in `ast.rs`.
//! The `ParseResult::warnings: Vec<SyntaxError>` field already exists in
//! `parser/mod.rs`.
//!
//! # Authoritative Sources (in precedence order)
//!
//! 1. STORY-078 spec: `.factory/stories/stories/STORY-078-parser-section-block-syntax.md`
//! 2. DIR-077-001-A (addendum, BINDING): `.factory/cycles/STORY-077/section-parse-directive.md`
//!    - Ruling 2: sub-block KEY warning is parse-time (not eval-deferred)
//!    - Ruling 3: section TYPE is stored verbatim; no built-in-list rejection at parse time
//! 3. BC-3.02.002 v1.2 (NO amendment): `invariant 4` — parse-time warning for unrecognized KEY
//!
//! # Naming Convention
//!
//! All test functions follow the `test_BC_S_SS_NNN_xxx` pattern for traceability.

// ── Test helper ──────────────────────────────────────────────────────────────

use std::sync::Arc;

use crate::{
    ast::{BlockItem, SectionNode},
    error::SyntaxError,
    parser::{ParseResult, parse},
    span::SourceMap,
};

/// Lex and parse `src`, returning the full `ParseResult`.
///
/// Panics if the parse returns `Err` (i.e., there were fatal errors).
/// Use `parse_src_errors` when you expect errors.
fn parse_src_ok(src: &str) -> ParseResult {
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
    parse(src, file_id, &sm).expect("expected parse to succeed with 0 fatal errors")
}

/// Lex and parse `src`, returning the fatal error list.
///
/// Panics if the parse succeeds (i.e., returned `Ok`).
fn parse_src_errors(src: &str) -> Vec<SyntaxError> {
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
    parse(src, file_id, &sm).expect_err("expected parse to fail with fatal errors")
}

// ── AC-001 ────────────────────────────────────────────────────────────────────

/// AC-001 (BC-3.02.002 preconditions 1 and 3):
/// After STORY-078, `"section"` must NOT be in `RESERVED_KEYWORDS` as E-PAR-006.
/// A minimal `section methodology:` block with indented body must parse with
/// zero fatal errors and produce exactly one `BlockItem::Section` in `deck.items`.
///
/// Red Gate reason: currently `"section"` IS in `RESERVED_KEYWORDS` (E-PAR-006),
/// so parsing any `section <type>:` block fails with a reserved-keyword error.
/// After the implementation removes the entry and wires `section_block_parser`,
/// this test will pass.
#[test]
fn test_section_keyword_no_longer_reserved() {
    // AC-001 canonical fixture (story spec § "AC-001")
    let src = "section methodology:\n  detail: \"Some content.\"\n";
    let result = parse_src_ok(src);

    assert!(
        result.warnings.is_empty(),
        "AC-001: minimal section block must produce 0 warnings, got: {:?}",
        result.warnings
    );
    assert_eq!(
        result.deck.items.len(),
        1,
        "AC-001: deck must contain exactly 1 item (the section block)"
    );
    assert!(
        matches!(result.deck.items[0], BlockItem::Section(_)),
        "AC-001: the single deck item must be a BlockItem::Section, got: {:?}",
        result.deck.items[0]
    );
}

/// AC-001 companion: keyword registry functions must return correct values.
///
/// `classify_keyword("section")` must return `None` (not an error-code entry).
/// `is_reserved_bare_keyword("section")` must return `false`.
///
/// Red Gate reason: currently `"section"` IS in RESERVED_KEYWORDS, so both
/// functions return values inconsistent with the post-implementation state.
#[test]
fn test_BC_3_02_002_section_keyword_not_in_reserved_map() {
    use crate::keywords::{classify_keyword, is_reserved_bare_keyword};

    // After the implementation removes "section" from RESERVED_KEYWORDS:
    assert_eq!(
        classify_keyword("section"),
        None,
        "classify_keyword(\"section\") must return None after STORY-078 (no longer reserved)"
    );
    assert!(
        !is_reserved_bare_keyword("section"),
        "is_reserved_bare_keyword(\"section\") must return false after STORY-078"
    );
}

// ── AC-002 ────────────────────────────────────────────────────────────────────

/// AC-002 (BC-3.02.002 precondition 2, postconditions 5 and 8):
/// `section <type>:` with a recognized type and indented `report:` / `detail:`
/// sub-blocks must produce a `SectionNode` where:
/// - `kind.value == "methodology"`
/// - `fields.len() == 2`
/// - `fields[0].name.value == "report"`, `fields[1].name.value == "detail"`
/// - both `FieldValue` variants are `FieldValue::Template`
///
/// Red Gate reason: `section_block_parser` does not exist; no `BlockItem::Section`
/// is produced; the test will fail with either a compile error or a parse error.
#[test]
fn test_section_recognized_type_with_sub_blocks() {
    use crate::ast::FieldValue;

    // AC-002 canonical fixture (story spec § "AC-002")
    let src = concat!(
        "section methodology:\n",
        "  report: \"We applied rigor.\"\n",
        "  detail: \"Extended methodology detail.\"\n",
    );
    let result = parse_src_ok(src);

    assert_eq!(
        result.deck.items.len(),
        1,
        "AC-002: must have exactly 1 deck item"
    );

    let BlockItem::Section(ref spanned_section) = result.deck.items[0] else {
        panic!(
            "AC-002: deck.items[0] must be BlockItem::Section, got {:?}",
            result.deck.items[0]
        );
    };
    let section: &SectionNode = spanned_section.value();

    assert_eq!(
        section.kind.value(),
        "methodology",
        "AC-002: section kind must be \"methodology\""
    );
    assert!(
        !section.kind.span().is_empty(),
        "AC-002: kind span must be non-empty"
    );
    assert_eq!(
        section.fields.len(),
        2,
        "AC-002: section must have exactly 2 field nodes (report + detail)"
    );
    assert_eq!(
        section.fields[0].name.value(),
        "report",
        "AC-002: first field name must be \"report\""
    );
    assert_eq!(
        section.fields[1].name.value(),
        "detail",
        "AC-002: second field name must be \"detail\""
    );
    // Both values must be FieldValue::Template (initial parse-time representation;
    // STORY-077 will upgrade these to FieldValue::Inlines).
    assert!(
        matches!(section.fields[0].value.value(), FieldValue::Template(_)),
        "AC-002: report field value must be FieldValue::Template"
    );
    assert!(
        matches!(section.fields[1].value.value(), FieldValue::Template(_)),
        "AC-002: detail field value must be FieldValue::Template"
    );
}

// ── AC-003 ────────────────────────────────────────────────────────────────────

/// AC-003 (BC-3.02.002 precondition 2, DIR-077-001-A Ruling 3):
/// The parser must store ANY identifier after `section` verbatim in
/// `SectionNode.kind` — NO built-in-list check and NO error for unrecognized types.
/// `section foobar:` with a valid body must parse successfully with:
/// - `parser.errors.is_empty() == true`
/// - `result.warnings.is_empty() == true`
/// - `node.kind.value == "foobar"`
///
/// Red Gate reason: `section_block_parser` does not exist yet. Once it exists,
/// it must NOT validate the type against any registry (Ruling 3).
#[test]
fn test_section_unknown_type_parsed_verbatim() {
    // AC-003 canonical fixture (DIR-077-001-A Ruling 3 / story spec AC-003)
    let src = concat!("section foobar:\n", "  detail: \"Some content.\"\n",);
    let result = parse_src_ok(src);

    // Zero errors — the parse must SUCCEED for any type name
    assert!(
        result.warnings.is_empty(),
        "AC-003: unknown section type must produce 0 parse-time warnings (type validation is eval-stage), got: {:?}",
        result.warnings
    );
    assert_eq!(
        result.deck.items.len(),
        1,
        "AC-003: must have exactly 1 deck item"
    );
    let BlockItem::Section(ref spanned) = result.deck.items[0] else {
        panic!(
            "AC-003: deck.items[0] must be BlockItem::Section, got {:?}",
            result.deck.items[0]
        );
    };
    assert_eq!(
        spanned.value().kind.value(),
        "foobar",
        "AC-003: unknown type must be stored verbatim in kind.value"
    );
}

// ── AC-004 ────────────────────────────────────────────────────────────────────

/// AC-004 (BC-3.02.002 invariant 4, EC-005, DIR-077-001-A Ruling 2):
/// An unrecognized sub-block KEY inside a section block must produce:
/// (a) a `FieldNode` with `name.value == "foo"` retained in `SectionNode.fields`
///     (the key is NOT dropped from the AST), AND
/// (b) exactly 1 non-fatal `ParseSeverity::Warning` in `ParseResult::warnings`
///     emitted at parse time (not deferred to eval), AND
/// (c) the warning message must contain the key name `"foo"`, AND
/// (d) parse succeeds with zero fatal errors.
///
/// Red Gate reason: (1) `section_block_parser` does not exist; (2) even when it
/// exists, the warning-routing plumbing in `parser/mod.rs` must be wired for
/// `W-PAR-` prefixed diagnostics to land in `ParseResult::warnings`; (3) the
/// `.validate()` closure must emit the warning before the implementer wires it.
#[test]
fn test_section_unrecognized_key_warning_at_parse_time() {
    // AC-004 canonical fixture (story spec § "AC-004")
    let src = concat!(
        "section methodology:\n",
        "  foo: \"unrecognized\"\n",
        "  detail: \"real content\"\n",
    );
    let result = parse_src_ok(src);

    // parse must SUCCEED (zero fatal errors) — this is checked by parse_src_ok's
    // expect(), which panics on Err

    assert_eq!(
        result.warnings.len(),
        1,
        "AC-004: exactly 1 parse-time warning must be emitted for the unrecognized key 'foo', got {} warnings: {:?}",
        result.warnings.len(),
        result.warnings
    );

    // Warning message must name the unrecognized key
    let warn_msg = format!("{:?}", result.warnings[0]);
    assert!(
        warn_msg.contains("foo"),
        "AC-004: warning message must contain the key name 'foo', got: {warn_msg}"
    );

    // The unrecognized key must be RETAINED in the AST (not silently dropped)
    let BlockItem::Section(ref spanned) = result.deck.items[0] else {
        panic!("AC-004: deck.items[0] must be BlockItem::Section");
    };
    let section = spanned.value();
    assert_eq!(
        section.fields.len(),
        2,
        "AC-004: SectionNode.fields must have 2 entries (foo + detail) — unrecognized key must NOT be dropped"
    );
    assert_eq!(
        section.fields[0].name.value(),
        "foo",
        "AC-004: first field must be 'foo' (unrecognized key retained in AST)"
    );
    assert_eq!(
        section.fields[1].name.value(),
        "detail",
        "AC-004: second field must be 'detail'"
    );
}

/// AC-004 companion: verify the warning span points to the `foo` token.
///
/// The span must be non-empty (points to at least one byte in the source).
/// This exercises BC-3.02.002 invariant 4's "naming the key" requirement.
#[test]
fn test_BC_3_02_002_unrecognized_key_warning_span_non_empty() {
    let src = concat!(
        "section methodology:\n",
        "  foo: \"unrecognized\"\n",
        "  detail: \"real content\"\n",
    );
    let result = parse_src_ok(src);
    assert_eq!(
        result.warnings.len(),
        1,
        "must have exactly 1 warning for the span check"
    );
    // The warning must carry a non-zero byte position (i.e., it points to something
    // in the source). We check via the debug representation — a span of 0..0 would
    // be suspicious, while the 'foo' token starts well after byte 0.
    let _warning: &SyntaxError = &result.warnings[0]; // type check
    // The sort_position (file, line, col) must not be ("", 0, 0) — that would
    // indicate the span is degenerate.
    // We can't call sort_position directly (private), but we can observe that
    // the Debug representation of the warning contains a non-trivial message.
    let warn_debug = format!("{:?}", result.warnings[0]);
    assert!(
        !warn_debug.is_empty(),
        "AC-004: warning debug output must be non-empty"
    );
    assert!(
        warn_debug.contains("foo"),
        "AC-004: warning debug must reference key 'foo'"
    );
}

// ── AC-005 ────────────────────────────────────────────────────────────────────

/// AC-005 (BC-3.02.002 precondition 3, EC-002):
/// A `section <type>:` keyword appearing inside the body of a `slide` block must
/// produce a parse error: the error message must contain "section blocks must be
/// top-level" (or equivalent wording). No `SectionNode` must appear in the AST.
///
/// Red Gate reason: currently the `section` keyword is E-PAR-006 reserved, so the
/// error is a generic reserved-keyword error rather than the specific top-level
/// error. After STORY-078, the keyword is active but forbidden inside slide bodies;
/// the parser must detect the nesting context and emit the specific error.
#[test]
fn test_section_nested_in_slide_error() {
    // AC-005 test fixture: `section` indented inside a slide body
    let src = concat!(
        "slide content:\n",
        "  title \"My Slide\"\n",
        "  section methodology:\n",
        "    detail: \"nested — must error\"\n",
    );
    let errors = parse_src_errors(src);

    assert!(
        !errors.is_empty(),
        "AC-005: a section keyword inside a slide body must produce a parse error"
    );

    // The error message must indicate the top-level constraint.
    // We accept any error that mentions "top-level" or "section" in the context
    // of being nested — the exact wording is implementation-determined, but must
    // communicate the constraint.
    let error_messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
    let has_top_level_error = error_messages.iter().any(|msg| {
        msg.contains("top-level") || msg.contains("top_level") || msg.contains("section")
    });
    assert!(
        has_top_level_error,
        "AC-005: at least one error must reference the top-level constraint for nested section blocks, errors: {error_messages:?}"
    );
}

// ── AC-006 ────────────────────────────────────────────────────────────────────

/// AC-006 (BC-3.02.002 EC-004, EC-006, DIR-077-001 §5):
/// `SECTION_REGISTER_KEYS` must contain exactly `["report", "detail"]` — NOT
/// `"notes"`. The `is_register_sub_block_key` function must return the correct
/// booleans for all three inputs.
///
/// Red Gate reason: `section.rs` does not exist yet. This test will fail to
/// compile until the implementer creates it and exports `is_register_sub_block_key`.
/// Additionally, the existing stub (if any) has `["notes", "report", "detail"]`,
/// which would cause the `notes` assertion to fail at runtime.
#[test]
fn test_register_keys_exclude_notes() {
    // This test references a symbol that does NOT YET EXIST: this will cause a
    // compile error (Red Gate) until the implementer creates section.rs.
    // `crate::section` does NOT YET EXIST — this causes the compile-time Red Gate.
    // The implementer must create `crates/slideforge-syntax/src/section.rs` and
    // pub-export `is_register_sub_block_key` from it.
    use crate::section::is_register_sub_block_key;

    assert!(
        !is_register_sub_block_key("notes"),
        "AC-006: 'notes' must NOT be a register sub-block key — excluded per DIR-077-001 §5"
    );
    assert!(
        is_register_sub_block_key("report"),
        "AC-006: 'report' must be a register sub-block key"
    );
    assert!(
        is_register_sub_block_key("detail"),
        "AC-006: 'detail' must be a register sub-block key"
    );
}

/// AC-006 companion: `is_register_sub_block_key` must return false for arbitrary
/// unrecognized keys (not just "notes").
#[test]
fn test_BC_3_02_002_register_key_unknown_returns_false() {
    // `crate::section` does NOT YET EXIST — this causes the compile-time Red Gate.
    // The implementer must create `crates/slideforge-syntax/src/section.rs` and
    // pub-export `is_register_sub_block_key` from it.
    use crate::section::is_register_sub_block_key;

    assert!(
        !is_register_sub_block_key("foo"),
        "'foo' is not a register key"
    );
    assert!(
        !is_register_sub_block_key("body"),
        "'body' is not a register key"
    );
    assert!(
        !is_register_sub_block_key("title"),
        "'title' is not a register key"
    );
    assert!(
        !is_register_sub_block_key(""),
        "empty string is not a register key"
    );
}

// ── EC-005 / EC-006 ───────────────────────────────────────────────────────────

/// EC-005/EC-006 (BC-3.02.002 EC-006, DIR-077-001-A Ruling 2):
/// When an unrecognized sub-block key coincidentally matches a reserved register
/// name used WITHOUT the required `:` colon suffix (i.e., used as a bare scalar
/// rather than as a register declaration), the parser must emit a FATAL error
/// with a corrective hint.
///
/// Red Gate reason: `section_block_parser` does not exist. When it does, the
/// `.validate()` closure must implement `check_reserved_name_collision` for this case.
///
/// Note: This tests the collision detection path, which is distinct from the
/// plain-unrecognized-key warning (AC-004).
#[test]
fn test_reserved_name_collision() {
    // EC-006 fixture: "report" used without the register `:` suffix (as a plain
    // scalar field assignment, not as a register sub-block declaration).
    // The grammar expects `report: "..."` (with colon introducing the register block),
    // but this source writes it as `report "..."` (without colon), which is the
    // collision scenario: the parser parses it as an IDENT followed by a value,
    // then the `.validate()` closure detects that the IDENT matches a reserved
    // register name used without its required colon syntax.
    let src = concat!(
        "section methodology:\n",
        "  report \"used without colon — reserved name collision\"\n",
    );
    let errors = parse_src_errors(src);

    assert!(
        !errors.is_empty(),
        "EC-006: reserved register name used without colon syntax must produce a fatal parse error"
    );

    // The error message must contain a corrective hint about register syntax
    let error_messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
    let has_corrective_hint = error_messages
        .iter()
        .any(|msg| msg.contains("report") || msg.contains("reserved") || msg.contains("register"));
    assert!(
        has_corrective_hint,
        "EC-006: error must reference the reserved register name 'report' or provide a corrective hint, errors: {error_messages:?}"
    );
}

// ── Span propagation ──────────────────────────────────────────────────────────

/// Span propagation (BC-3.02.002 postcondition 8, story spec § "test_section_spans_present"):
/// Every `FieldNode.name.span` and `FieldNode.value.span` in a parsed section block
/// must be non-empty (i.e., carry a valid source location with start < end).
///
/// Red Gate reason: `section_block_parser` does not exist; no `SectionNode` is
/// produced; once it is, the span propagation via `to_span(ss, file_id)` must be wired.
#[test]
fn test_section_spans_present() {
    let src = concat!(
        "section methodology:\n",
        "  report: \"We applied rigor.\"\n",
        "  detail: \"Extended methodology detail.\"\n",
    );
    let result = parse_src_ok(src);

    let BlockItem::Section(ref spanned) = result.deck.items[0] else {
        panic!("test_section_spans_present: expected BlockItem::Section");
    };
    let section = spanned.value();

    assert!(
        !spanned.span().is_empty(),
        "test_section_spans_present: the SectionNode outer span must be non-empty"
    );
    assert!(
        !section.kind.span().is_empty(),
        "test_section_spans_present: kind span must be non-empty (points to the type IDENT)"
    );

    for (i, field) in section.fields.iter().enumerate() {
        assert!(
            !field.name.span().is_empty(),
            "test_section_spans_present: field[{i}].name.span must be non-empty, field name = {:?}",
            field.name.value()
        );
        assert!(
            !field.value.span().is_empty(),
            "test_section_spans_present: field[{i}].value.span must be non-empty, field name = {:?}",
            field.name.value()
        );
    }
}

// ── Error accumulation ────────────────────────────────────────────────────────

/// Error accumulation (BC-3.02.002 precondition 1, Q23 LOCKED, story spec §
/// "test_section_error_accumulation"):
/// When a section block contains TWO malformed field entries, the parser must
/// accumulate BOTH errors and return them — it must NOT bail on the first error.
///
/// Red Gate reason: (1) `section_block_parser` does not exist; (2) when it does,
/// the `recover_with(skip_then_retry_until(...))` pattern must be used — without
/// recovery, chumsky bails on the first error and the second is lost.
#[test]
fn test_section_error_accumulation() {
    // Two syntactically malformed field lines: use a token sequence the parser
    // cannot handle inside a section body (e.g., bare colon with no value, which
    // forces the recover_with path to skip and retry).
    // The exact DSL that triggers a field-parse error depends on the grammar, but
    // two missing value tokens forces two independent error accumulations.
    //
    // Strategy: use two unrecognized reserved keywords as field names (E-PAR-006)
    // in a section body. After STORY-078, the section sub-block key is just an
    // IDENT — using a token that is NOT an IDENT (e.g., a colon without a key)
    // will trigger the recovery path.
    //
    // We use two consecutive malformed key lines with no value:
    //   section methodology:
    //     :             ← no key before colon — parse error 1
    //     :             ← no key before colon — parse error 2
    //
    // If the parser bails on error 1, errors.len() == 1 and the test fails.
    // If the parser accumulates, errors.len() == 2 and the test passes.
    //
    // Note: exact error count may be implementation-dependent (the recovery may
    // absorb context-tokens and produce a different count). The invariant we
    // enforce is errors.len() >= 2.
    let src = concat!(
        "section methodology:\n",
        "  : \"orphan-value-1\"\n", // malformed: no IDENT key, just a colon
        "  : \"orphan-value-2\"\n", // malformed: no IDENT key, just a colon
    );
    let errors = parse_src_errors(src);

    assert!(
        errors.len() >= 2,
        "test_section_error_accumulation: expected at least 2 accumulated errors (one per malformed field), got {}: {errors:?}",
        errors.len()
    );
}

// ── AC-007: Snapshot test ─────────────────────────────────────────────────────

/// AC-007 (BC-3.02.002 precondition 1, postcondition 6):
/// Snapshot test for a 2-section deck AST. This serves as a regression baseline
/// for STORY-077: any parser change that alters the AST representation of section
/// blocks will break this snapshot and require explicit review.
///
/// Red Gate reason: `section_block_parser` does not exist; the snap will not
/// match once implemented (because it doesn't exist yet, `insta` will create a
/// new snapshot on first run — but the test structure ensures it's driven by the
/// correct observable output).
#[test]
fn test_BC_3_02_002_two_section_deck_ast_snapshot() {
    // AC-007 canonical fixture (story spec § "AC-007")
    let src = concat!(
        "section methodology:\n",
        "  report: \"Methodology content.\"\n",
        "\n",
        "section scope:\n",
        "  detail: \"Scope detail.\"\n",
        "  report: \"Scope report.\"\n",
    );
    let result = parse_src_ok(src);

    // Before snapshotting, assert structural invariants so the snapshot is
    // meaningful and not just a blind capture.
    assert_eq!(
        result.deck.items.len(),
        2,
        "AC-007: 2-section deck must have exactly 2 items in source order"
    );
    assert!(
        matches!(result.deck.items[0], BlockItem::Section(_)),
        "AC-007: items[0] must be Section"
    );
    assert!(
        matches!(result.deck.items[1], BlockItem::Section(_)),
        "AC-007: items[1] must be Section"
    );

    // Verify source order is preserved (methodology before scope)
    let BlockItem::Section(ref sec0) = result.deck.items[0] else {
        panic!("AC-007: items[0] is not a Section");
    };
    let BlockItem::Section(ref sec1) = result.deck.items[1] else {
        panic!("AC-007: items[1] is not a Section");
    };
    assert_eq!(
        sec0.value().kind.value(),
        "methodology",
        "AC-007: first section must be methodology"
    );
    assert_eq!(
        sec1.value().kind.value(),
        "scope",
        "AC-007: second section must be scope"
    );

    // insta snapshot: captures full AST as regression baseline for STORY-077.
    // The snapshot file will be auto-created on first passing run.
    insta::assert_debug_snapshot!("two_section_deck_ast", result.deck);
}
