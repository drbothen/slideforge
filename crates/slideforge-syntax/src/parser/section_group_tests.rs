//! Tests for STORY-082: `section "Name":` slide-grouping block parser.
//!
//! Covers AC-001, AC-002, AC-010, AC-011 from STORY-082 at the parser level.
//!
//! ## TDD Red Gate
//!
//! These tests MUST FAIL before implementation. They drive:
//! - `SectionGroupNode` AST type in `ast.rs`
//! - `BlockItem::SectionGroup` variant in `ast.rs`
//! - `section_group_parser` in `parser/section_group.rs`
//! - E-PAR-023 error for empty section group name
//! - W-PAR-002 warning for duplicate section group names
//!
//! ## Traceability
//!
//! | Test | AC | BC clause |
//! |------|----|----|
//! | `test_BC_4_01_003_ac001_section_group_node_parses_with_name` | AC-001 | BC-4.01.003 PC5 |
//! | `test_BC_4_01_003_ac002_section_group_distinct_from_section_block` | AC-002 | BC-4.01.003 PC5 |
//! | `test_BC_4_01_003_ac010_empty_section_name_rejected_e_par_023` | AC-010 | BC-4.01.003 PC7+inv5 |
//! | `test_BC_4_01_003_ac010_empty_section_name_no_section_group_node` | AC-010 | BC-4.01.003 PC7+inv5 |
//! | `test_BC_4_01_003_ac011_duplicate_section_name_warning_w_par_002` | AC-011 | BC-4.01.003 PC8+inv5 |
//! | `test_BC_4_01_003_ac011_duplicate_section_name_both_sections_present` | AC-011 | BC-4.01.003 PC8 |

#![allow(non_snake_case)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::manual_assert)]
#![allow(clippy::missing_docs_in_private_items)]

use crate::lexer::lex;
use crate::parser::deck::deck_parser;
use crate::span::SourceMap;
use crate::token::Token;
use crate::{BlockItem, parse};
use chumsky::input::Input as _;
use chumsky::{Parser, prelude::SimpleSpan};

// ─── AC-001: section "Name": parses with correct SectionGroupNode ─────────────

/// AC-001 — `section "Background":` parses into a `SectionGroupNode` with
/// `name == "Background"`. Traces to BC-4.01.003 precondition 5.
#[test]
fn test_BC_4_01_003_ac001_section_group_node_parses_with_name() {
    let src = r#"slideforge_version "1"
section "Background":
  slide title:
    title "Background Slide"
"#;
    let mut source_map = SourceMap::default();
    let file_id = source_map.add_file(std::sync::Arc::from("test.sf"), std::sync::Arc::from(src));
    let result = parse(src, file_id, &source_map);

    // Parse must succeed with no fatal errors.
    let parse_result = result.expect("section \"Background\": must parse without fatal errors");

    // Find the SectionGroup block item.
    let section_group = parse_result
        .deck
        .items
        .iter()
        .find_map(|item| {
            if let BlockItem::SectionGroup(spanned) = item {
                Some(spanned.value())
            } else {
                None
            }
        })
        .expect("deck must contain a BlockItem::SectionGroup for section \"Background\":");

    assert_eq!(
        section_group.name.value().as_ref(),
        "Background",
        "SectionGroupNode.name must be \"Background\" (AC-001 / BC-4.01.003 PC5)"
    );
}

// ─── AC-002: disambiguation — quoted-string vs bare-ident ────────────────────

/// AC-002 — A file with both `section methodology:` (bare-ident) and
/// `section "Background":` (quoted-string) produces two DISTINCT AST node types.
///
/// Traces to BC-4.01.003 precondition 5 (no grammar collision).
#[test]
fn test_BC_4_01_003_ac002_section_group_distinct_from_section_block() {
    let src = r#"slideforge_version "1"
section methodology:
  report: "This is the methodology."
section "Background":
  slide title:
    title "Background"
"#;
    let mut source_map = SourceMap::default();
    let file_id = source_map.add_file(std::sync::Arc::from("test.sf"), std::sync::Arc::from(src));
    let result = parse(src, file_id, &source_map);

    let parse_result = result.expect("mixed section forms must parse without fatal errors");
    let items = &parse_result.deck.items;

    let section_block_count = items
        .iter()
        .filter(|i| matches!(i, BlockItem::Section(_)))
        .count();
    let section_group_count = items
        .iter()
        .filter(|i| matches!(i, BlockItem::SectionGroup(_)))
        .count();

    assert_eq!(
        section_block_count, 1,
        "must have exactly 1 SectionBlock (bare-ident, STORY-078); got {section_block_count}"
    );
    assert_eq!(
        section_group_count, 1,
        "must have exactly 1 SectionGroup (quoted-string, STORY-082); got {section_group_count}"
    );

    // The SectionBlock must be the bare-ident form (kind = "methodology").
    let section_block = items
        .iter()
        .find_map(|i| {
            if let BlockItem::Section(s) = i {
                Some(s.value())
            } else {
                None
            }
        })
        .expect("SectionBlock must be present");
    assert_eq!(section_block.kind.value(), "methodology");

    // The SectionGroup must have name = "Background".
    let section_group = items
        .iter()
        .find_map(|i| {
            if let BlockItem::SectionGroup(s) = i {
                Some(s.value())
            } else {
                None
            }
        })
        .expect("SectionGroup must be present");
    assert_eq!(
        section_group.name.value().as_ref(),
        "Background",
        "SectionGroupNode.name must be \"Background\""
    );
}

// ─── AC-010: empty section name → E-PAR-023 ──────────────────────────────────

/// AC-010 — `section "":` (empty quoted name) produces `E-PAR-023` fatal error.
///
/// Traces to BC-4.01.003 postcondition 7 + invariant 5 + error-taxonomy.md §24.
///
/// Per error-taxonomy.md §24: "Parse Errors (E-PAR) — Always fatal. Build halts
/// with accumulated errors. No output produced."  `--warn-only` does NOT demote
/// parse errors.  Only `Err` is spec-compliant; `Ok` is a regression.
#[test]
fn test_BC_4_01_003_ac010_empty_section_name_rejected_e_par_023() {
    let src = r#"slideforge_version "1"
section "":
  slide title:
    title "Should Be Rejected"
"#;
    let mut source_map = SourceMap::default();
    let file_id = source_map.add_file(std::sync::Arc::from("test.sf"), std::sync::Arc::from(src));

    // E-PAR-023 is always fatal (error-taxonomy.md §24); parse() MUST return Err.
    let errors = parse(src, file_id, &source_map).expect_err(
        "parse() MUST return Err for section \"\": — E-PAR-023 is always fatal \
         (error-taxonomy.md §24: Parse Errors are always fatal; --warn-only does \
         NOT demote parse errors)",
    );

    let has_e_par_023 = errors.iter().any(|e| format!("{e}").contains("E-PAR-023"));
    assert!(
        has_e_par_023,
        "parse error for empty section name must include E-PAR-023; got: {errors:?}"
    );
}

/// AC-010 — `section "":` produces no `SectionGroupNode` in the AST for the rejected block.
///
/// Traces to BC-4.01.003 postcondition 7 + invariant 5.
///
/// # Load-bearing design
///
/// E-PAR-023 is always fatal (`parse()` returns `Err`), so inspecting the `parse()`
/// result directly would never yield an `Ok` to unwrap — the assertion about the
/// recovered AST would be permanently unreachable (paper-fix / TD-VSDD-059).
///
/// Instead this test exercises the sentinel-discard logic (`deck.rs:615-652`) by
/// calling `deck_parser` directly on the raw token stream.  `deck_parser` is the
/// combinator that chumsky drives *inside* `parse()`, and it is exactly where the
/// sentinel is discarded.  The `into_output_errors()` call returns the partially-
/// recovered `DeckNode` alongside the chumsky `Rich` errors, letting us assert BOTH:
///   1. The chumsky error list contains E-PAR-023 (the rule fired).
///   2. No `BlockItem::SectionGroup` with an empty name survived in the recovered
///      AST after the sentinel-discard branch ran (BC-4.01.003 PC-7 / inv-5).
///
/// Regression proof: if a future change removed the `if name.is_empty() { continue; }`
/// guard from `deck.rs`, assertion 2 would fire because the empty-named node would
/// survive.  If a future change removed the E-PAR-023 emitter from
/// `section_group.rs`, assertion 1 would fire.
#[test]
fn test_BC_4_01_003_ac010_empty_section_name_no_section_group_node() {
    let src = r#"slideforge_version "1"
section "":
  slide title:
    title "Should Be Rejected"
"#;
    let mut source_map = SourceMap::default();
    let file_id = source_map.add_file(std::sync::Arc::from("test.sf"), std::sync::Arc::from(src));

    // ── Assertion 1 (via parse()): E-PAR-023 is fatal — parse() returns Err. ──────
    // This proves the error-taxonomy rule: no output is produced for an empty-named
    // section.  If parse() ever returned Ok, this assertion would fire.
    let parse_errors = parse(src, file_id, &source_map).expect_err(
        "parse() MUST return Err for section \"\": — E-PAR-023 is always fatal \
         (BC-4.01.003 PC-7; error-taxonomy.md §24)",
    );
    let has_e_par_023 = parse_errors
        .iter()
        .any(|e| format!("{e}").contains("E-PAR-023"));
    assert!(
        has_e_par_023,
        "parse error for empty section name must include E-PAR-023; got: {parse_errors:?}"
    );

    // ── Assertion 2 (via deck_parser): sentinel-discard — no empty-named node survives. ──
    // Call the chumsky combinator directly (the same parser that parse() drives
    // internally) to obtain the partially-recovered DeckNode alongside the raw errors.
    // The sentinel-discard branch in deck.rs:626-628 runs inside deck_parser, so
    // even if chumsky's error-recovery emits a temporary empty-named SectionGroupNode
    // as a sentinel, it is stripped before the DeckNode is returned.
    let (tokens, _lex_errs) = lex(src, std::sync::Arc::from("test.sf"));
    let spanned: Vec<(Token, SimpleSpan)> = tokens
        .into_iter()
        .map(|(t, s)| (t, SimpleSpan::from(s)))
        .collect();
    let eoi = SimpleSpan::from(src.len()..src.len());
    let input = spanned
        .as_slice()
        .map(eoi, |(t, s): &(Token, SimpleSpan)| (t, s));

    let (deck_opt, _raw_errors) = deck_parser(file_id).parse(input).into_output_errors();

    assert!(
        deck_opt.is_some(),
        "deck_parser must produce a recovered DeckNode for `section \"\":` (sentinel emitted \
         then discarded at deck.rs:628); None here would silently skip the sentinel-discard \
         assertion (LESSON-14 / TD-VSDD-059)"
    );
    let recovered_deck = deck_opt.expect("checked is_some above");
    let empty_group = recovered_deck.items.iter().find(|item| {
        if let BlockItem::SectionGroup(s) = item {
            s.value().name.value().is_empty()
        } else {
            false
        }
    });
    assert!(
        empty_group.is_none(),
        "sentinel-discard (deck.rs:626-628) must remove the empty-named \
         SectionGroupNode from the recovered AST (BC-4.01.003 PC-7 / inv-5)"
    );
}

// ─── AC-011: duplicate section names → W-PAR-002 ─────────────────────────────

/// AC-011 — Two `section "Background":` blocks produce `W-PAR-002` warning.
/// Build exits with code 0 (cosmetic warning). Both nodes are in AST.
///
/// Traces to BC-4.01.003 postcondition 8 + invariant 5.
#[test]
fn test_BC_4_01_003_ac011_duplicate_section_name_warning_w_par_002() {
    let src = r#"slideforge_version "1"
section "Background":
  slide title:
    title "First Background"
section "Background":
  slide title:
    title "Second Background"
"#;
    let mut source_map = SourceMap::default();
    let file_id = source_map.add_file(std::sync::Arc::from("test.sf"), std::sync::Arc::from(src));
    let result = parse(src, file_id, &source_map);

    // Parse must SUCCEED (duplicate name is a WARNING, not a fatal error).
    let parse_result = result.expect(
        "duplicate section name must NOT produce a fatal parse error — \
         W-PAR-002 is cosmetic (BC-4.01.003 PC8)",
    );

    // Warnings must include W-PAR-002.
    let has_w_par_002 = parse_result
        .warnings
        .iter()
        .any(|w| format!("{w}").contains("W-PAR-002"));
    assert!(
        has_w_par_002,
        "duplicate section name must produce W-PAR-002 warning; \
         got warnings: {:?}",
        parse_result.warnings
    );
}

/// AC-011 — Both `section "Background":` entries appear in the AST.
///
/// Traces to BC-4.01.003 postcondition 8.
#[test]
fn test_BC_4_01_003_ac011_duplicate_section_name_both_sections_present() {
    let src = r#"slideforge_version "1"
section "Background":
  slide title:
    title "First Background"
section "Background":
  slide title:
    title "Second Background"
"#;
    let mut source_map = SourceMap::default();
    let file_id = source_map.add_file(std::sync::Arc::from("test.sf"), std::sync::Arc::from(src));
    let result = parse(src, file_id, &source_map);

    let parse_result =
        result.expect("duplicate section name must parse successfully (warning only)");

    let group_count = parse_result
        .deck
        .items
        .iter()
        .filter(|item| matches!(item, BlockItem::SectionGroup(_)))
        .count();
    assert_eq!(
        group_count, 2,
        "both duplicate SectionGroupNode entries must appear in AST; got {group_count} \
         (BC-4.01.003 PC8)"
    );
}

// ─── CRIT-3: section group parses real slide children (not empty) ─────────────

/// CRIT-3 — A `section "Name":` with N slides must produce a `SectionGroupNode`
/// with `slides.len() == N` containing the correct slide types.
///
/// This test closes the CRIT-3 finding: the parser was previously consuming
/// the section body and DISCARDING the tokens. This test asserts the real
/// slide children are populated in `SectionGroupNode.slides`.
///
/// Traces to BC-4.01.003 (parser must populate slide children).
#[test]
fn test_BC_4_01_003_crit3_section_group_slides_parsed() {
    let src = r#"slideforge_version "1"
section "Background":
  slide title:
    title "First Slide"
  slide content:
    title "Second Slide"
  slide bullets:
    title "Third Slide"
"#;
    let mut source_map = SourceMap::default();
    let file_id = source_map.add_file(std::sync::Arc::from("test.sf"), std::sync::Arc::from(src));
    let result = parse(src, file_id, &source_map);

    let parse_result = result.expect("section with 3 slides must parse without fatal errors");

    let section_group = parse_result
        .deck
        .items
        .iter()
        .find_map(|item| {
            if let BlockItem::SectionGroup(spanned) = item {
                Some(spanned.value())
            } else {
                None
            }
        })
        .expect("deck must contain a BlockItem::SectionGroup");

    assert_eq!(
        section_group.slides.len(),
        3,
        "SectionGroupNode.slides must contain 3 children; got {} \
         (CRIT-3: parser was previously discarding section body tokens)",
        section_group.slides.len()
    );

    // Verify that children are Slide block items with correct types.
    let types: Vec<&str> = section_group
        .slides
        .iter()
        .filter_map(|item| {
            if let BlockItem::Slide(spanned) = item {
                Some(spanned.value().kind.value().as_str())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(
        types,
        vec!["title", "content", "bullets"],
        "section slide children must be in source order with correct types; got {types:?}"
    );
}

/// CRIT-3 supplement — `section "Empty":\n` with no indented body fails to parse.
///
/// # Load-bearing design
///
/// The grammar for a section group is:
/// ```text
/// section_group ::= "section" STRING ":" NEWLINE INDENT slide_block* DEDENT
/// ```
/// `INDENT` is a mandatory token.  A bare `section "Empty":\n` (no subsequent
/// indented block) produces no `Indent` token, so the combinator fails.  This
/// means `parse()` MUST return `Err` for this input.
///
/// The old `if let Ok(...) { ... }` form was vacuous: the `Ok` arm was never
/// taken so the assertion inside it was never executed (TD-VSDD-059 / OBS-P14-1).
///
/// The fix asserts the parse FAILS (unconditional) — a regression where the
/// grammar silently accepted an empty section body would be caught here.
/// The CRIT-3 *affirmative* assertion (non-empty body → populated `slides` vec)
/// is already covered by `test_BC_4_01_003_crit3_section_group_slides_parsed`.
#[test]
fn test_BC_4_01_003_crit3_empty_section_body() {
    // `section "Empty":\n` has no subsequent INDENT token — the grammar requires
    // `INDENT slide_block* DEDENT`, so this MUST fail to parse.
    let src = "slideforge_version \"1\"\nsection \"Empty\":\n";
    let mut source_map = SourceMap::default();
    let file_id = source_map.add_file(std::sync::Arc::from("test.sf"), std::sync::Arc::from(src));

    let result = parse(src, file_id, &source_map);

    // Unconditional assertion: empty section body must NOT parse successfully.
    // If the grammar were relaxed to allow INDENT-less bodies this test would
    // catch the regression immediately.
    assert!(
        result.is_err(),
        "section \"Empty\": with no indented body must produce a parse error \
         (grammar requires INDENT … DEDENT); got Ok: {result:?}"
    );
}
