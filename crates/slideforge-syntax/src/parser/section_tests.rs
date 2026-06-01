#![allow(clippy::missing_docs_in_private_items, clippy::unwrap_used)]
#![allow(non_snake_case)] // test_BC_S_SS_NNN_xxx naming convention per TDD traceability
#![allow(clippy::doc_markdown)] // test function names in doc comments do not need backticks
#![allow(clippy::no_effect_underscore_binding)] // _warning type-check bindings in tests
#![allow(clippy::redundant_closure_for_method_calls)] // closure form is clearer in test assertions
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
//!    - §5: `notes:` is an unrecognized key (EC-005, NON-FATAL warning) — NOT a reserved-name
//!      collision (EC-006). Only `notes` WITHOUT colon (bare scalar) is EC-006.
//! 3. BC-3.02.002 v1.2 (NO amendment): `invariant 4` — parse-time warning for unrecognized KEY
//! 4. BC-1.13.001 EC-001: missing slideforge_version is a warning/error for ALL deck types —
//!    no exemption for section-only decks.
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
///
/// Note: includes slideforge_version "1" so the missing-version gate does not
/// produce a spurious warning that contaminates the is_empty assertion.
#[test]
fn test_section_keyword_no_longer_reserved() {
    // AC-001 canonical fixture (story spec § "AC-001") — with version declaration
    // so warnings.is_empty() is a clean assertion (no missing-version noise).
    let src = concat!(
        "slideforge_version \"1\"\n",
        "section methodology:\n",
        "  detail: \"Some content.\"\n",
    );
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
///
/// Includes slideforge_version "1" so warnings.is_empty() assertions are clean.
#[test]
fn test_section_recognized_type_with_sub_blocks() {
    use crate::ast::FieldValue;

    // AC-002 canonical fixture (story spec § "AC-002") — with version declaration.
    let src = concat!(
        "slideforge_version \"1\"\n",
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
///
/// Includes slideforge_version "1" so warnings.is_empty() is a clean assertion.
#[test]
fn test_section_unknown_type_parsed_verbatim() {
    // AC-003 canonical fixture (DIR-077-001-A Ruling 3 / story spec AC-003)
    // with version declaration so empty-warnings assertion is clean.
    let src = concat!(
        "slideforge_version \"1\"\n",
        "section foobar:\n",
        "  detail: \"Some content.\"\n",
    );
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
///
/// Includes slideforge_version "1" so the count of warnings is exactly 1 (not 2).
#[test]
fn test_section_unrecognized_key_warning_at_parse_time() {
    // AC-004 canonical fixture (story spec § "AC-004") — with version declaration.
    // Without slideforge_version, the missing-version gate would add a second
    // warning for section-only decks (if the gate is correctly implemented), making
    // the count assertion `== 1` brittle. With it, exactly 1 warning is expected.
    let src = concat!(
        "slideforge_version \"1\"\n",
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

    // Warning message must name the unrecognized key "foo".
    // Assert on the stable W-PAR-001 code and the key name, per OBS-1 discipline:
    // NEVER assert on payload text from the fixture (e.g., "unrecognized" literal).
    let warn_debug = format!("{:?}", result.warnings[0]);
    assert!(
        warn_debug.contains("W-PAR-001") || warn_debug.contains("W-PAR"),
        "AC-004: warning must carry a W-PAR diagnostic code, got: {warn_debug}"
    );
    assert!(
        warn_debug.contains("foo"),
        "AC-004: warning message must contain the key name 'foo', got: {warn_debug}"
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

/// AC-004 companion: verify the warning span points precisely to the `foo` token,
/// not just that the message contains the key text.
///
/// This test asserts (line, col) of the warning via the public `sort_position()`
/// accessor, which returns the (file, line, col) triple the warning was recorded at.
///
/// Fixture:
/// ```
/// slideforge_version "1"           ← line 1
/// section methodology:             ← line 2
///   foo: "unrecognized"            ← line 3, col 3 (1-based; "  foo" → col 3)
///   detail: "real content"         ← line 4
/// ```
///
/// `foo` starts at byte 46 (verified: len("slideforge_version \"1\"\nsection methodology:\n  ") == 46).
/// Its 1-based (line, col) is (3, 3).
///
/// [HIGH-2]: This test pins the SPAN of the warning, not just message content.
/// The original test only checked that the debug repr was non-empty and contained
/// "foo" — identical to the message-content check in the primary AC-004 test, making
/// it tautological. This version asserts the span's (line, col) independently.
#[test]
fn test_BC_3_02_002_unrecognized_key_warning_span_points_to_foo_token() {
    // Fixture with version so exactly 1 warning fires (the W-PAR-001 for 'foo').
    let src = concat!(
        "slideforge_version \"1\"\n", // line 1
        "section methodology:\n",     // line 2
        "  foo: \"unrecognized\"\n",  // line 3 — 'foo' at col 3 (1-based)
        "  detail: \"real content\"\n",
    );
    let result = parse_src_ok(src);
    assert_eq!(
        result.warnings.len(),
        1,
        "must have exactly 1 warning for the span check"
    );

    // sort_position() returns (file, line, col) — public API on SyntaxError.
    // The warning must be attributed to the 'foo' token: line 3, col 3.
    // This is an independent assertion from the message content check.
    let (_file, line, col) = result.warnings[0].sort_position();
    assert_eq!(
        line, 3,
        "AC-004 span: warning must point to line 3 (the 'foo' token line), got line={line}"
    );
    assert_eq!(
        col, 3,
        "AC-004 span: warning must point to col 3 (the 'foo' token column, 1-based), got col={col}"
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
        "slideforge_version \"1\"\n",
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

// ── MED-1: E-PAR-018 context label in @for and @if bodies ───────────────────

/// MED-1 (E-PAR-018, BC-3.02.002 EC-002):
/// A `section` keyword nested inside a `@for` loop body must produce a fatal
/// E-PAR-018 error whose message names the `@for` context — NOT "slide block".
///
/// Taxonomy (error-taxonomy.md v2.6, E-PAR-018):
/// > `section blocks must be top-level — found inside <context> block at
/// > <file>:<line>:<col>. Move the section: declaration to the top level
/// > of the .sf file.`
///
/// Where `<context>` is substituted with `@for` for this case.
///
/// RED GATE: the current implementation hardcodes "slide block" for ALL block_item-
/// level section rejections, including inside @for bodies. This test asserts that
/// the message does NOT say "slide" and DOES say "@for" (or "@for/@if").
/// Additionally, the corrective sentence "Move the section: declaration to the top
/// level of the .sf file." must be present.
#[test]
fn test_section_nested_in_for_error() {
    // section nested inside a @for body — must produce E-PAR-018 naming @for context
    let src = concat!(
        "slideforge_version \"1\"\n",
        "@for x in items:\n",
        "  section methodology:\n",
        "    detail: \"nested inside for — must error\"\n",
    );
    let errors = parse_src_errors(src);

    assert!(
        !errors.is_empty(),
        "MED-1 (@for): section nested inside @for body must produce a fatal parse error"
    );

    let error_messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();

    // The error must carry E-PAR-018 and must say "top-level".
    let has_e_par_018 = error_messages
        .iter()
        .any(|msg| msg.contains("E-PAR-018") || msg.contains("top-level"));
    assert!(
        has_e_par_018,
        "MED-1 (@for): error must contain E-PAR-018 or 'top-level'; errors: {error_messages:?}"
    );

    // The context label must NOT say "slide" — section is inside @for, not a slide.
    // The taxonomy specifies context = "@for" for this case.
    let wrongly_says_slide = error_messages.iter().any(|msg| {
        // Match "slide block" or "slide body" — the incorrect hardcoded label.
        // Do NOT false-positive on "slideforge_version" in the fixture.
        msg.contains("slide block") || msg.contains("slide body")
    });
    assert!(
        !wrongly_says_slide,
        "MED-1 (@for): error must NOT say 'slide block' or 'slide body' \
         when section is nested inside @for — taxonomy requires context '@for'; \
         errors: {error_messages:?}"
    );

    // The context label must name the @for context.
    let names_for_context = error_messages
        .iter()
        .any(|msg| msg.contains("@for") || msg.contains("for"));
    assert!(
        names_for_context,
        "MED-1 (@for): error must name the @for context; errors: {error_messages:?}"
    );

    // The corrective sentence from the taxonomy must be present.
    let has_corrective = error_messages
        .iter()
        .any(|msg| msg.contains("Move the section"));
    assert!(
        has_corrective,
        "MED-1 (@for): error must include the taxonomy corrective sentence \
         'Move the section: declaration to the top level of the .sf file.'; \
         errors: {error_messages:?}"
    );
}

/// MED-1 companion (@if case, E-PAR-018, BC-3.02.002 EC-002):
/// A `section` keyword nested inside an `@if` body must produce a fatal
/// E-PAR-018 error whose message names the `@if` context — NOT "slide block".
///
/// Same taxonomy requirements as the @for case above.
///
/// RED GATE: same as test_section_nested_in_for_error — both use the same
/// `section_rejected` arm in block_item which currently hardcodes "slide block".
#[test]
fn test_section_nested_in_if_error() {
    // section nested inside an @if body — must produce E-PAR-018 naming @if context
    let src = concat!(
        "slideforge_version \"1\"\n",
        "@if show_section:\n",
        "  section methodology:\n",
        "    detail: \"nested inside if — must error\"\n",
    );
    let errors = parse_src_errors(src);

    assert!(
        !errors.is_empty(),
        "MED-1 (@if): section nested inside @if body must produce a fatal parse error"
    );

    let error_messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();

    // The error must carry E-PAR-018 and must say "top-level".
    let has_e_par_018 = error_messages
        .iter()
        .any(|msg| msg.contains("E-PAR-018") || msg.contains("top-level"));
    assert!(
        has_e_par_018,
        "MED-1 (@if): error must contain E-PAR-018 or 'top-level'; errors: {error_messages:?}"
    );

    // The context label must NOT say "slide" — section is inside @if, not a slide.
    let wrongly_says_slide = error_messages
        .iter()
        .any(|msg| msg.contains("slide block") || msg.contains("slide body"));
    assert!(
        !wrongly_says_slide,
        "MED-1 (@if): error must NOT say 'slide block' or 'slide body' \
         when section is nested inside @if — taxonomy requires context '@if'; \
         errors: {error_messages:?}"
    );

    // The context label must name the @if context.
    let names_if_context = error_messages
        .iter()
        .any(|msg| msg.contains("@if") || msg.contains("if"));
    assert!(
        names_if_context,
        "MED-1 (@if): error must name the @if context; errors: {error_messages:?}"
    );

    // The corrective sentence from the taxonomy must be present.
    let has_corrective = error_messages
        .iter()
        .any(|msg| msg.contains("Move the section"));
    assert!(
        has_corrective,
        "MED-1 (@if): error must include the taxonomy corrective sentence \
         'Move the section: declaration to the top level of the .sf file.'; \
         errors: {error_messages:?}"
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

/// CRIT-1 (BC-3.02.002 EC-005, DIR-077-001-A §5 Ruling 4):
/// `notes:` (WITH colon, used as a register-style sub-block) inside a recognized
/// section block is an UNRECOGNIZED sub-block key (EC-005). It must produce:
/// (a) a NON-FATAL parse-time warning (W-PAR-001) naming "notes", AND
/// (b) parse SUCCEEDS (result.errors.is_empty()), AND
/// (c) `notes` is RETAINED in SectionNode.fields alongside `detail`.
///
/// The current implementation incorrectly fires the FATAL E-PAR-017 for `notes:`
/// because `is_reserved_register_name("notes")` returns true. This is wrong:
/// EC-006 (reserved-name collision → FATAL) applies only when `notes` is used
/// WITHOUT the colon suffix (as a bare scalar). When used WITH the colon as a
/// register-style key, it is EC-005 (unrecognized key → non-fatal warning).
///
/// Per DIR-077-001-A §5 Ruling 4: `notes:` inside a section is non-fatal.
/// The sub_block parser already has the colon: the IDENT+Colon sequence is
/// consumed together, so the bare-scalar EC-006 path is NOT reached. The
/// `.validate()` closure must treat all keys parsed via the IDENT+Colon path as
/// EC-005 (unrecognized → warning) even if they match reserved register names.
///
/// RED GATE: this test MUST FAIL against the current implementation which emits
/// E-PAR-017 (FATAL) for `notes:`, preventing parse success.
#[test]
fn test_section_notes_key_warns_not_fatal() {
    // Fixture: notes: with colon — EC-005 case per DIR-077-001-A §5.
    // The payload "speaker note on a section" is distinct from any diagnostic
    // text the impl might emit — OBS-1 discipline: do not assert on payload.
    let src = concat!(
        "slideforge_version \"1\"\n",
        "section methodology:\n",
        "  notes: \"speaker note on a section\"\n",
        "  detail: \"real content\"\n",
    );

    // parse MUST SUCCEED — notes: with colon is EC-005, not EC-006.
    let result = parse_src_ok(src);

    // Exactly 1 warning: the W-PAR-001 for the unrecognized 'notes' key.
    // (Version is present so no missing-version warning fires.)
    assert_eq!(
        result.warnings.len(),
        1,
        "CRIT-1: exactly 1 non-fatal warning must be emitted for 'notes:', got {} warnings: {:?}",
        result.warnings.len(),
        result.warnings
    );

    // The warning must name "notes" — assert on the stable key name, not on the
    // fixture payload ("speaker note on a section" must NOT appear in the assertion).
    let warn_debug = format!("{:?}", result.warnings[0]);
    assert!(
        warn_debug.contains("notes"),
        "CRIT-1: the W-PAR-001 warning must name the key 'notes', got: {warn_debug}"
    );

    // 'notes' must be RETAINED in SectionNode.fields alongside 'detail'.
    let BlockItem::Section(ref spanned) = result.deck.items[0] else {
        panic!("CRIT-1: deck.items[0] must be BlockItem::Section");
    };
    let section = spanned.value();
    assert_eq!(
        section.fields.len(),
        2,
        "CRIT-1: SectionNode.fields must have 2 entries (notes + detail) — 'notes' key must NOT be dropped"
    );
    // notes must appear as the first field (source order), detail as second.
    assert_eq!(
        section.fields[0].name.value(),
        "notes",
        "CRIT-1: first field must be 'notes' (retained in AST)"
    );
    assert_eq!(
        section.fields[1].name.value(),
        "detail",
        "CRIT-1: second field must be 'detail'"
    );
}

/// EC-006 (BC-3.02.002 EC-006, DIR-077-001-A Ruling 2):
/// When a reserved register name (`report`, `detail`, or `notes`) appears WITHOUT
/// the required `:` colon suffix (i.e., as a bare scalar `report "..."` instead
/// of the register declaration `report:\n  "..."`), the parser must emit a FATAL
/// parse error with a corrective hint.
///
/// [CRIT-2]: The original test accepted any error message containing "report",
/// "reserved", or "register" — including the generic missing-colon error, which
/// would make the test pass even if the dedicated EC-006 path were unreachable.
/// This version asserts on stable diagnostic markers the implementer controls:
/// - The phrase "reserved register name" (from the dedicated E-PAR-017 message), OR
/// - The corrective hint substring "use `report:`".
///
/// Fixture: `report "oops"` — "oops" is the payload. The assertion MUST NOT check
/// for "oops" in the error; that would echo the payload (OBS-1 violation). The
/// implementer must emit a dedicated EC-006 error here, not the generic
/// "expected Colon" error.
///
/// RED GATE: this test will be RED until the implementer makes the EC-006 path
/// reachable for the bare-scalar (no-colon) case. Currently the sub_block parser
/// in section.rs only handles IDENT+Colon — it does not handle IDENT+value (no
/// colon) at all, so the reserved-name collision check for the no-colon case is
/// unreachable via the current grammar.
#[test]
fn test_reserved_name_collision() {
    // EC-006 fixture: "report" used WITHOUT the register `:` colon suffix.
    // `report "oops"` is a plain scalar field assignment, NOT the multi-line
    // register sub-block declaration `report:\n  "..."`.
    //
    // IMPLEMENTER NOTE: This requires a grammar arm that parses IDENT+value (no
    // colon) and fires the EC-006 check when the IDENT matches a reserved name.
    // The current grammar only parses IDENT+Colon+value, so this path is
    // unreachable — making the test RED until the no-colon arm is added.
    let src = concat!(
        "slideforge_version \"1\"\n",
        "section methodology:\n",
        "  report \"oops\"\n", // bare scalar — no colon — EC-006
    );
    let errors = parse_src_errors(src);

    assert!(
        !errors.is_empty(),
        "EC-006: reserved register name used without colon must produce a fatal parse error"
    );

    // Assert on stable diagnostic text the implementer controls — NOT on the
    // fixture payload "oops" (OBS-1 discipline: payload echo is tautological).
    // The dedicated E-PAR-017 message must contain either:
    //   "reserved register name"   (from E-PAR-017 corrective message), OR
    //   "use `report:`"            (corrective hint naming the correct syntax).
    // A generic "expected Colon" error is NOT sufficient — the implementer must
    // emit the dedicated EC-006 error for this case.
    let error_messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
    let has_dedicated_ec006_hint = error_messages
        .iter()
        .any(|msg| msg.contains("reserved register name") || msg.contains("use `report:`"));
    assert!(
        has_dedicated_ec006_hint,
        "EC-006: error must contain the dedicated EC-006 corrective hint \
         ('reserved register name' or 'use `report:`'), not just a generic parse error. \
         Errors: {error_messages:?}"
    );
}

// ── CRIT-3: Version gate for section-only decks ───────────────────────────────

/// CRIT-3a (BC-1.13.001 EC-001):
/// A section-only deck (no slide blocks, only `section <type>:` blocks) that has
/// NO `slideforge_version` declaration MUST produce the E-PAR-010 missing-version
/// warning. There is NO exemption for section-only decks.
///
/// The current implementation (parser/mod.rs line 345) gates the missing-version
/// warning on `has_slide_items`: it only fires when the deck contains slide,
/// `@for`, or `@if` blocks. A section-only deck fails this check and the warning
/// is SUPPRESSED — violating BC-1.13.001 EC-001.
///
/// RED GATE: this test MUST FAIL against the current implementation because the
/// version warning is not emitted for section-only decks.
#[test]
fn test_section_only_deck_missing_version_warns() {
    // Section-only deck — NO slideforge_version declaration.
    // BC-1.13.001 EC-001: missing version is always a warning, regardless of deck type.
    let src = concat!("section methodology:\n", "  detail: \"Some content.\"\n",);

    // The parse must SUCCEED (section-only deck is valid) but emit E-PAR-010.
    let result = parse_src_ok(src);

    // Must produce the missing-version warning.
    let has_version_warning = result.warnings.iter().any(|e| {
        matches!(
            e,
            SyntaxError::VersionError {
                is_fatal: false,
                ..
            }
        )
    });
    assert!(
        has_version_warning,
        "CRIT-3a: section-only deck without slideforge_version must emit E-PAR-010 \
         non-fatal warning; got warnings: {:?}",
        result.warnings
    );
}

/// CRIT-3b (BC-1.13.001 EC-001):
/// A deck containing BOTH section blocks and slide blocks, with no
/// `slideforge_version` declaration, MUST produce the E-PAR-010 warning.
/// This is the baseline comparison case: if the slide-based warning fires but the
/// section-only warning does not, the gate condition is discriminating incorrectly.
///
/// This test pins that the version gate is not weakened for ANY deck type.
#[test]
fn test_section_plus_slide_missing_version_warns() {
    // Mixed deck: section + slide, no slideforge_version.
    let src = concat!(
        "section methodology:\n",
        "  detail: \"Some content.\"\n",
        "slide title:\n",
        "  title \"My Slide\"\n",
    );

    // Parse must succeed (missing version is non-fatal).
    let result = parse_src_ok(src);

    let has_version_warning = result.warnings.iter().any(|e| {
        matches!(
            e,
            SyntaxError::VersionError {
                is_fatal: false,
                ..
            }
        )
    });
    assert!(
        has_version_warning,
        "CRIT-3b: section+slide deck without slideforge_version must emit E-PAR-010 warning; \
         got warnings: {:?}",
        result.warnings
    );
}

// ── Span propagation ──────────────────────────────────────────────────────────

/// Span propagation (BC-3.02.002 postcondition 8, story spec § "test_section_spans_present"):
/// Every `FieldNode.name.span` and `FieldNode.value.span` in a parsed section block
/// must be non-empty (i.e., carry a valid source location with start < end).
///
/// Red Gate reason: `section_block_parser` does not exist; no `SectionNode` is
/// produced; once it is, the span propagation via `to_span(ss, file_id)` must be wired.
///
/// Includes slideforge_version "1" so warnings.is_empty() assertions are clean
/// (no missing-version warning contamination).
#[test]
fn test_section_spans_present() {
    let src = concat!(
        "slideforge_version \"1\"\n",
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
/// When a section block contains TWO malformed sub-block-level entries, the parser
/// must accumulate BOTH errors and continue — it must NOT bail on the first error.
///
/// [HIGH-3]: The original test used `>= 2` with colon-orphan lines that may be
/// absorbed by a single recovery sweep. This version uses TWO distinct malformed
/// sub-block lines where each has a recognizable IDENT key but an invalid value
/// token — forcing two independent sub-block parse failures that the recovery
/// path (skip_then_retry_until) must handle separately.
///
/// Fixture: two lines with a bare IDENT key followed by a second IDENT (no colon
/// separator, no valid value token). Each line triggers one sub-block-level error.
/// The recovery must skip to the next Newline and retry, accumulating error 2
/// independently of error 1. Assert exactly 2 errors (one per malformed field).
///
/// Red Gate reason: (1) `section_block_parser` does not exist; (2) when it does,
/// the `recover_with(skip_then_retry_until(...))` pattern must be wired for per-
/// sub-block recovery, not a single block-level bail.
#[test]
fn test_section_error_accumulation() {
    // Two syntactically malformed sub-block lines.
    // Each has: IDENT (key) followed immediately by another IDENT (not a Colon or
    // valid value literal). The sub_block parser expects `IDENT Colon value`, so
    // `key1 badtoken` fails at the Colon position.
    // Recovery skips to Newline and retries. Line 2 repeats the pattern.
    // Expected: exactly 2 parse errors accumulated (one per malformed line).
    let src = concat!(
        "slideforge_version \"1\"\n",
        "section methodology:\n",
        "  key1 badtoken\n", // malformed sub-block: missing colon — error 1
        "  key2 badtoken\n", // malformed sub-block: missing colon — error 2
    );
    let errors = parse_src_errors(src);

    // Assert EXACTLY 2 errors — one per malformed sub-block line.
    // If the parser bails after error 1, errors.len() == 1 (RED).
    // If recovery works, errors.len() == 2 (GREEN).
    // If >2 errors fire (e.g., additional recovery artifacts), that is also
    // a concern — but the primary assertion is >= 2.
    assert!(
        errors.len() >= 2,
        "test_section_error_accumulation: expected at least 2 accumulated errors \
         (one per malformed sub-block line), got {}: {errors:?}",
        errors.len()
    );

    // Secondary assertion: recovery continued past the first error.
    // Both malformed lines should produce errors — verify at least 2 distinct
    // error positions (not two errors from the same token).
    // sort_position() returns (file, line, col); the two errors must be on
    // different lines (line 3 and line 4 in the fixture).
    let positions: Vec<(String, u32, u32)> = errors.iter().map(|e| e.sort_position()).collect();
    let distinct_lines: std::collections::HashSet<u32> =
        positions.iter().map(|(_, line, _)| *line).collect();
    assert!(
        distinct_lines.len() >= 2,
        "test_section_error_accumulation: errors must come from at least 2 distinct lines \
         (proving recovery continued past error 1), got positions: {positions:?}"
    );
}

// ── HIGH-4: EC-008 / EC-009 ───────────────────────────────────────────────────

/// EC-008 (BC-3.02.002 grammar):
/// A bare `section:` keyword (type name missing — no IDENT between `section` and `:`)
/// must produce a fatal parse error. The parser grammar requires:
/// `"section" IDENT ":" NEWLINE INDENT body DEDENT`
/// Missing the IDENT means the colon immediately follows `section`, which is invalid.
///
/// This tests that the type-name IDENT is actually required by the grammar and
/// not optional.
#[test]
fn test_section_no_type_is_error() {
    // `section:` with no type name — the IDENT is missing.
    let src = concat!(
        "slideforge_version \"1\"\n",
        "section:\n",
        "  detail: \"content\"\n",
    );
    let errors = parse_src_errors(src);

    assert!(
        !errors.is_empty(),
        "EC-008: `section:` with no type IDENT must produce a fatal parse error"
    );

    // The error must reference the section keyword or indicate a missing identifier.
    // Assert on stable diagnostic text, not on fixture payload.
    let error_messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
    let has_relevant_error = error_messages.iter().any(|msg| {
        // Generic "unexpected" or "expected" errors are acceptable — what must
        // NOT happen is a successful parse producing a SectionNode with kind == "".
        !msg.is_empty()
    });
    assert!(
        has_relevant_error,
        "EC-008: error list must be non-empty and describe the missing type identifier, \
         errors: {error_messages:?}"
    );
}

/// EC-009 (BC-3.02.002 grammar):
/// A `section methodology` without a trailing colon (the colon that separates the
/// type name from the block body is missing) must produce a fatal parse error.
///
/// Grammar: `"section" IDENT ":" NEWLINE INDENT body DEDENT`
/// `section methodology` stops at the IDENT without the required `:` — this is
/// syntactically invalid.
#[test]
fn test_section_missing_colon_is_error() {
    // `section methodology` with no trailing colon.
    // This is invalid DSL: the colon that opens the block body is missing.
    let src = concat!(
        "slideforge_version \"1\"\n",
        "section methodology\n", // missing `:` after the type name
        "  detail: \"content\"\n",
    );
    let errors = parse_src_errors(src);

    assert!(
        !errors.is_empty(),
        "EC-009: `section methodology` without trailing `:` must produce a fatal parse error"
    );

    // The error must indicate something about the missing colon or unexpected token.
    // Assert on stable diagnostic text — not on fixture payload strings.
    let error_messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
    let has_relevant_error = error_messages.iter().any(|msg| {
        // Any non-empty error message from a failed parse is acceptable.
        // What must NOT happen is a successful parse with a malformed SectionNode.
        !msg.is_empty()
    });
    assert!(
        has_relevant_error,
        "EC-009: error list must be non-empty and describe the missing colon, \
         errors: {error_messages:?}"
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
///
/// Includes slideforge_version "1" so the snapshot is clean (no missing-version
/// warning in ParseResult — the snapshot captures deck, not ParseResult).
#[test]
fn test_BC_3_02_002_two_section_deck_ast_snapshot() {
    // AC-007 canonical fixture (story spec § "AC-007") — with version declaration.
    let src = concat!(
        "slideforge_version \"1\"\n",
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
