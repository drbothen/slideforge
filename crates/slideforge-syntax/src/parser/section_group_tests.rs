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

use crate::span::SourceMap;
use crate::{BlockItem, parse};

// ─── AC-001: section "Name": parses with correct SectionGroupNode ─────────────

/// AC-001 — `section "Background":` parses into a `SectionGroupNode` with
/// `name == "Background"`. Traces to BC-4.01.003 precondition 5.
#[test]
fn test_BC_4_01_003_ac001_section_group_node_parses_with_name() {
    let src = r#"slideforge_version "0.1.0"
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
    let src = r#"slideforge_version "0.1.0"
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

/// AC-010 — `section "":` (empty quoted name) produces `E-PAR-023` error.
///
/// Traces to BC-4.01.003 postcondition 7 + invariant 5.
#[test]
fn test_BC_4_01_003_ac010_empty_section_name_rejected_e_par_023() {
    let src = r#"slideforge_version "0.1.0"
section "":
  slide title:
    title "Should Be Rejected"
"#;
    let mut source_map = SourceMap::default();
    let file_id = source_map.add_file(std::sync::Arc::from("test.sf"), std::sync::Arc::from(src));
    let result = parse(src, file_id, &source_map);

    match result {
        Err(errors) => {
            // Fatal errors must include E-PAR-023.
            let has_e_par_023 = errors.iter().any(|e| format!("{e}").contains("E-PAR-023"));
            assert!(
                has_e_par_023,
                "parse error for empty section name must include E-PAR-023; got: {errors:?}"
            );
        },
        Ok(parse_result) => {
            // If parse "succeeded", the warnings must include E-PAR-023
            // (accumulated non-fatal error).
            let has_e_par_023 = parse_result
                .warnings
                .iter()
                .any(|w| format!("{w}").contains("E-PAR-023"));
            // Either fatal error or warning is acceptable — as long as E-PAR-023 appears.
            // The key invariant is that no empty-named SectionGroupNode appears in AST.
            let empty_group = parse_result.deck.items.iter().any(|item| {
                if let BlockItem::SectionGroup(s) = item {
                    s.value().name.value().is_empty()
                } else {
                    false
                }
            });
            assert!(
                !empty_group,
                "SectionGroupNode with empty name must NOT appear in AST (BC-4.01.003 inv5)"
            );
            if !has_e_par_023 {
                panic!(
                    "E-PAR-023 must appear in errors or warnings for empty section name; \
                     warnings: {:?}",
                    parse_result.warnings
                );
            }
        },
    }
}

/// AC-010 — `section "":` produces no `SectionGroupNode` in the AST for the rejected block.
///
/// Traces to BC-4.01.003 postcondition 7 + invariant 5.
#[test]
fn test_BC_4_01_003_ac010_empty_section_name_no_section_group_node() {
    let src = r#"slideforge_version "0.1.0"
section "":
  slide title:
    title "Should Be Rejected"
"#;
    let mut source_map = SourceMap::default();
    let file_id = source_map.add_file(std::sync::Arc::from("test.sf"), std::sync::Arc::from(src));

    let deck_items = match parse(src, file_id, &source_map) {
        Ok(result) => result.deck.items,
        Err(_) => {
            // Fatal parse error is acceptable (E-PAR-023 is fatal per spec).
            return;
        },
    };

    let empty_group = deck_items.iter().find(|item| {
        if let BlockItem::SectionGroup(s) = item {
            s.value().name.value().is_empty()
        } else {
            false
        }
    });

    assert!(
        empty_group.is_none(),
        "SectionGroupNode with empty name must NOT appear in AST (BC-4.01.003 PC7)"
    );
}

// ─── AC-011: duplicate section names → W-PAR-002 ─────────────────────────────

/// AC-011 — Two `section "Background":` blocks produce `W-PAR-002` warning.
/// Build exits with code 0 (cosmetic warning). Both nodes are in AST.
///
/// Traces to BC-4.01.003 postcondition 8 + invariant 5.
#[test]
fn test_BC_4_01_003_ac011_duplicate_section_name_warning_w_par_002() {
    let src = r#"slideforge_version "0.1.0"
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
    let src = r#"slideforge_version "0.1.0"
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
