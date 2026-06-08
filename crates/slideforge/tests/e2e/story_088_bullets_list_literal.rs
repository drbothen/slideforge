//! STORY-088 End-to-End Tests: bullets list-literal DSL syntax.
//!
//! ## Acceptance Criteria covered
//!
//! - **AC-007**: `bullets: ["Item A", "Item B", "Item C"]` (direct list-literal,
//!   no `@var`) produces a PPTX with ≥3 `<a:r>` text runs containing the bullet
//!   strings. This is the E2E closure of the STORY-086 AC-007 parser-gap note.
//!
//! ## Module structure
//!
//! This module contains:
//! 1. `test_bc_1_01_002_ac007_direct_list_literal_bullets_produces_ge3_text_runs_pptx`
//!    — the STORY-088 primary E2E test (direct `bullets: [...]` form).
//!
//! The STORY-086 AC-007 test (`test_bc_1_16_001_ac007_bullets_slide_produces_ge3_text_runs_in_pptx`)
//! in `story_086_content_threading.rs` exercises the `@var items = [...]` form.
//! That test is currently `#[ignore]`'d pending this story and will be un-ignored
//! by the implementer when STORY-088 ships.
//!
//! ## Red Gate discipline
//!
//! This test MUST FAIL until:
//! 1. `FieldValue::List` parser arm is added to `deck.rs::value_parser()`.
//! 2. `FieldValue::List` eval arm is added to `eval_field_value_to_value`.
//! 3. Stage 2b `thread_fields_to_blocks` routes `Value::List` → `ContentBlock::Bullets`.
//!
//! Without (1), the `bullets: [...]` field produces a parse error and `build()` fails.
//!
//! ## Architecture compliance (STORY-050)
//!
//! All tests call `slideforge::build()` via the public API only.
//! No imports from `slideforge_pptx`, `slideforge_docx`, `slideforge_pdf`, etc.
//!
//! ## Traceability
//!
//! BC-1.01.002 (parse entry point);
//! BC-1.16.001 PC-7 (Value::List → ContentBlock::Bullets);
//! LESSON-13 positive content vector (bullet text assertions);
//! LESSON-14 (assert content, not mere structural presence).

#![allow(clippy::unwrap_used)] // integration tests — explicit panic on failure is correct
#![allow(clippy::doc_markdown)] // test doc comments have unquoted identifiers

use crate::e2e::{BrandTmpDir, open_zip};

// ─── Fixture helper ────────────────────────────────────────────────────────────

/// Read the story-088 direct-literal bullets fixture.
///
/// The fixture uses the canonical direct `bullets: [...]` form (no `@var`),
/// which is the primary new syntax delivered by STORY-088.
fn story_088_direct_literal_source() -> String {
    let path = format!(
        "{}/tests/fixtures/story-088-bullets-direct-literal.sf",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("STORY-088: cannot read fixture 'story-088-bullets-direct-literal.sf': {e}")
    })
}

/// Read ZIP entry as UTF-8 string.
fn read_zip_entry(
    archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
    entry: &str,
    label: &str,
) -> String {
    use std::io::Read as _;
    let mut file = archive
        .by_name(entry)
        .unwrap_or_else(|e| panic!("STORY-088 {label}: ZIP entry '{entry}' not found: {e}"));
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .unwrap_or_else(|e| panic!("STORY-088 {label}: cannot read '{entry}': {e}"));
    buf
}

// ─── AC-007 (STORY-088 primary): direct list-literal bullets → ≥3 PPTX runs ──

/// BC-1.01.002 AC-007 — `bullets ["Item A", "Item B", "Item C"]` (direct
/// list-literal field-value, no `@var`) produces a PPTX where the content slide
/// contains ≥3 `<a:r>` text runs with the corresponding bullet strings.
///
/// ## Red Gate path
///
/// **Step 1 (parser gate):** The fixture source contains `bullets: ["Item A", ...]`.
/// Without the `[...]` arm in `value_parser()`, parsing produces a parse error
/// and `build()` returns `Err(BuildError::ParseFailed)` — the `unwrap_or_else`
/// in this test panics before the PPTX assertions are reached.
///
/// **Step 2 (eval gate):** Once the parser accepts the list literal and produces
/// `FieldValue::List([...])`, the evaluator must have a `FieldValue::List` arm
/// to produce `Value::List([...])`. Without the eval arm (stub: returns `None`),
/// the field is dropped → `Slide.blocks` has no `ContentBlock::Bullets` →
/// `<a:r>` count is 0 → assertion FAILS.
///
/// **Step 3 (threading gate):** `thread_fields_to_blocks` must route
/// `Value::List(items)` → `ContentBlock::Bullets(items)` (already implemented
/// in STORY-086; verified by the Stage-2b unit test). This gate is already green.
///
/// ## Content assertions (LESSON-13 / LESSON-14)
///
/// The test asserts VISIBLE CONTENT — the bullet text strings "Item A", "Item B",
/// "Item C" must appear in the PPTX slide XML — not just structural validity.
///
/// Traces: BC-1.01.002 (parse entry point);
///         BC-1.16.001 PC-7 (Value::List → ContentBlock::Bullets);
///         LESSON-13 positive content vector; LESSON-14 content assertion.
#[test]
fn test_bc_1_01_002_ac007_direct_list_literal_bullets_produces_ge3_text_runs_pptx() {
    let brand = BrandTmpDir::new("s088_ac007_direct_literal");
    let source = story_088_direct_literal_source();
    let opts = brand.build_options("pptx", false);

    // RED GATE STEP 1: Without the parser list-literal arm, build() returns Err
    // (parse error on the `[` token in the bullets field) → panic here.
    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "AC-007 RED GATE: build() with direct list-literal bullets must return Ok; \
             got Err: {e:?}. \
             Check: value_parser() in deck.rs must have a [...]  list-literal arm. \
             Without it, 'bullets: [...]' produces a parse error before eval runs."
        )
    });

    let mut archive = open_zip(&output.bytes, "AC-007");
    let slide_xml = read_zip_entry(&mut archive, "ppt/slides/slide1.xml", "AC-007");

    // Count <a:r> run elements — each bullet item produces one TextRun frame
    // which serializes as one <a:r><a:t>item text</a:t></a:r>.
    let a_r_count = slide_xml.matches("<a:r>").count();

    // RED GATE STEP 2/3: Must have at least 3 runs (one per bullet: "Item A", "Item B", "Item C").
    assert!(
        a_r_count >= 3,
        "AC-007 RED GATE: PPTX slide1.xml must contain ≥3 <a:r> text runs for \
         3 direct list-literal bullet items. Got {a_r_count} runs. \
         Check: FieldValue::List eval arm and thread_fields_to_blocks routing. \
         BC-1.01.002 + BC-1.16.001 PC-7."
    );

    // LESSON-14: assert the CONTENT (text strings), not just the run count.
    assert!(
        slide_xml.contains("Item A"),
        "AC-007: PPTX slide1.xml must contain bullet text 'Item A'; \
         got {a_r_count} runs"
    );
    assert!(
        slide_xml.contains("Item B"),
        "AC-007: PPTX slide1.xml must contain bullet text 'Item B'"
    );
    assert!(
        slide_xml.contains("Item C"),
        "AC-007: PPTX slide1.xml must contain bullet text 'Item C'"
    );
}
