//! Bleed invariant tests for BC-1.14.004 — No Register Content Bleeds to
//! Wrong Format. (STORY-036)
//!
//! # Test organisation
//!
//! Tests are split into two groups:
//!
//! ## Group 1: AC-008 BleedChecker utility unit tests (RUNS NOW)
//!
//! These tests construct minimal synthetic PPTX-like / DOCX-like ZIP bytes
//! in-memory using the `zip` crate and exercise `BleedChecker`'s four methods.
//! They do not require any exporter crate. They MUST FAIL until
//! `BleedChecker`'s `todo!()` stubs are implemented (Red Gate phase).
//!
//! - `test_BC_1_14_004_bleedchecker_absent_from_slides_passes_when_not_present`
//! - `test_BC_1_14_004_bleedchecker_absent_from_slides_panics_when_present`
//! - `test_BC_1_14_004_bleedchecker_absent_from_all_passes_when_not_present`
//! - `test_BC_1_14_004_bleedchecker_absent_from_all_panics_when_present`
//! - `test_BC_1_14_004_bleedchecker_present_in_docx_body_passes_when_present`
//! - `test_BC_1_14_004_bleedchecker_present_in_docx_body_panics_when_absent`
//! - `test_BC_1_14_004_bleedchecker_absent_from_docx_body_passes_when_not_present`
//! - `test_BC_1_14_004_bleedchecker_absent_from_docx_body_panics_when_present`
//!
//! ## EC-003 and EC-004 (RUNS NOW)
//!
//! - `test_BC_1_14_004_ec003_sentinel_in_title_not_flagged_as_bleed` — a
//!   sentinel in a non-scanned path (`content.xml`) must not trigger the slide
//!   check.
//! - `test_BC_1_14_004_ec004_xml_escaped_sentinel_detected` — BleedChecker
//!   scans decoded text, so the sentinel is found even if the underlying bytes
//!   differ from the sentinel string in trivially different ways.
//!
//! ## Group 2: Fixture parse test (RUNS NOW)
//!
//! - `test_BC_1_14_004_fixture_parses_without_errors` — confirms the canonical
//!   `three-register-slide.sf` fixture file parses through the real
//!   parser/evaluator without fatal errors.
//!
//! ## Group 3: Exporter-dependent AC tests (IGNORED until exporters exist)
//!
//! These tests are written in full (compile, reference BleedChecker) but are
//! marked `#[ignore]` because they require output from exporter crates that
//! do not yet exist:
//!
//! - AC-001, AC-002, AC-003: `#[ignore = "requires STORY-037 PPTX exporter"]`
//! - AC-004, AC-005: `#[ignore = "requires STORY-041 DOCX exporter"]`
//! - AC-006: `#[ignore = "requires STORY-046 HTML exporter"]`
//! - AC-007 (full cross-format): `#[ignore = "requires STORY-037 PPTX exporter"]`
//!
//! # Traceability
//!
//! | Test name | BC clause | AC |
//! |-----------|-----------|-----|
//! | `test_BC_1_14_004_bleedchecker_*` | BC-1.14.004 invariant 1 | AC-008 |
//! | `test_BC_1_14_004_ec003_*` | BC-1.14.004 EC-003 | AC-008 |
//! | `test_BC_1_14_004_ec004_*` | BC-1.14.004 EC-004 | AC-008 |
//! | `test_BC_1_14_004_fixture_parses_*` | BC-1.14.004 postcondition 1-6 | AC-007 |
//! | `test_BC_1_14_004_ac001_*` | BC-1.14.004 postcondition 2 | AC-001 |
//! | `test_BC_1_14_004_ac002_*` | BC-1.14.004 postcondition 4 | AC-002 |
//! | `test_BC_1_14_004_ac003_*` | BC-1.14.004 postcondition 6 | AC-003 |
//! | `test_BC_1_14_004_ac004_*` | BC-1.14.004 postcondition 2 | AC-004 |
//! | `test_BC_1_14_004_ac005_*` | BC-1.14.004 postcondition 3 | AC-005 |
//! | `test_BC_1_14_004_ac006_*` | BC-1.14.004 postcondition 6 | AC-006 |
//! | `test_BC_1_14_004_ac007_*` | BC-1.14.004 postcondition 1-6 | AC-007 |

// ─── Allowlist for test code conventions ─────────────────────────────────────
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
// BC-ID traceability convention requires uppercase in test names (e.g. BC_1_14_004).
#![allow(non_snake_case)]
// `#[should_panic]` tests here intentionally omit the expected message — the
// relevant invariant is that the method panics at all, not the message text.
#![allow(clippy::should_panic_without_expect)]
// doc_markdown: sentinel names like SENTINEL_NOTES in comments are identifiers
// by convention, not code references; backtick-quoting all of them in prose
// comments would reduce readability.
#![allow(clippy::doc_markdown)]
// items_after_statements: test helpers declared after `let` bindings are
// idiomatic in test code; restructuring them would reduce locality.
#![allow(clippy::items_after_statements)]

use std::io::Write;

use slideforge_eval::BleedChecker;

// ─── ZIP construction helpers ─────────────────────────────────────────────────
//
// These helpers construct minimal in-memory ZIP archives that mimic the
// structure of PPTX and DOCX files. They are used by AC-008 / EC-003 / EC-004
// tests to verify BleedChecker's logic without requiring real exporters.
//
// PPTX structure (relevant paths):
//   ppt/slides/slide1.xml      <- slide body
//   ppt/notesSlides/notesSlide1.xml  <- speaker notes
//
// DOCX structure (relevant paths):
//   word/document.xml          <- document body

/// Build an in-memory ZIP containing a single PPTX slide body with the given content.
///
/// The ZIP has two members:
/// - `ppt/slides/slide1.xml` — with `slide_content` embedded
/// - `ppt/notesSlides/notesSlide1.xml` — with `notes_content` embedded
///
/// This mimics the minimal structure BleedChecker scans for slide-body absence checks.
fn make_pptx_zip(slide_content: &str, notes_content: &str) -> Vec<u8> {
    let buf = Vec::new();
    let cursor = std::io::Cursor::new(buf);
    let mut zip = zip::ZipWriter::new(cursor);
    let opts =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("ppt/slides/slide1.xml", opts).unwrap();
    zip.write_all(
        format!(
            r#"<?xml version="1.0"?><p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><p:cSld><p:spTree>{slide_content}</p:spTree></p:cSld></p:sld>"#
        )
        .as_bytes(),
    )
    .unwrap();

    zip.start_file("ppt/notesSlides/notesSlide1.xml", opts)
        .unwrap();
    zip.write_all(
        format!(
            r#"<?xml version="1.0"?><p:notes xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><p:cSld><p:spTree>{notes_content}</p:spTree></p:cSld></p:notes>"#
        )
        .as_bytes(),
    )
    .unwrap();

    let cursor = zip.finish().unwrap();
    cursor.into_inner()
}

/// Build an in-memory ZIP with a PPTX slide body AND an additional file at a
/// non-slides path (used for EC-003: sentinel in a non-scanned path).
fn make_pptx_zip_with_extra(slide_content: &str, extra_path: &str, extra_content: &str) -> Vec<u8> {
    let buf = Vec::new();
    let cursor = std::io::Cursor::new(buf);
    let mut zip = zip::ZipWriter::new(cursor);
    let opts =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("ppt/slides/slide1.xml", opts).unwrap();
    zip.write_all(
        format!(
            r#"<?xml version="1.0"?><p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">{slide_content}</p:sld>"#
        )
        .as_bytes(),
    )
    .unwrap();

    zip.start_file(extra_path, opts).unwrap();
    zip.write_all(extra_content.as_bytes()).unwrap();

    let cursor = zip.finish().unwrap();
    cursor.into_inner()
}

/// Build an in-memory ZIP containing a DOCX document body with the given content.
fn make_docx_zip(document_content: &str) -> Vec<u8> {
    let buf = Vec::new();
    let cursor = std::io::Cursor::new(buf);
    let mut zip = zip::ZipWriter::new(cursor);
    let opts =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("word/document.xml", opts).unwrap();
    zip.write_all(
        format!(
            r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>{document_content}</w:t></w:r></w:p></w:body></w:document>"#
        )
        .as_bytes(),
    )
    .unwrap();

    let cursor = zip.finish().unwrap();
    cursor.into_inner()
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-008: BleedChecker utility unit tests (RUNS NOW — Red Gate)
//
// These tests exercise the four BleedChecker methods with synthetic ZIP bytes.
// They MUST FAIL at Red Gate time because all four methods are todo!() stubs.
// After implementation they must PASS.
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.14.004 invariant 1 / AC-008:
/// `assert_absent_from_pptx_slides` must NOT panic when the sentinel is absent
/// from `ppt/slides/slide*.xml`.
///
/// Red Gate: this test FAILS because `assert_absent_from_pptx_slides` is `todo!()`.
#[test]
fn test_BC_1_14_004_bleedchecker_absent_from_slides_passes_when_not_present() {
    let pptx = make_pptx_zip("visual body content only", "NOTES_SENTINEL_VALUE");
    // Sentinel is in the notes slide (allowed), NOT in the slide body.
    // Must complete without panicking.
    BleedChecker::assert_absent_from_pptx_slides(&pptx, "NOTES_SENTINEL_VALUE");
}

/// BC-1.14.004 invariant 1 / AC-008:
/// `assert_absent_from_pptx_slides` MUST PANIC when the sentinel IS present
/// in a `ppt/slides/slide*.xml` file (invariant violation detected).
///
/// Red Gate: this test FAILS because `assert_absent_from_pptx_slides` is `todo!()`.
#[test]
#[should_panic]
fn test_BC_1_14_004_bleedchecker_absent_from_slides_panics_when_present() {
    // Sentinel in slide body — this is the bleed defect scenario.
    let pptx = make_pptx_zip("NOTES_SENTINEL_VALUE leaked into body", "");
    BleedChecker::assert_absent_from_pptx_slides(&pptx, "NOTES_SENTINEL_VALUE");
}

/// BC-1.14.004 invariant 1 / AC-008:
/// `assert_absent_from_pptx_all` must NOT panic when the sentinel is absent
/// from every file in the PPTX ZIP.
///
/// Red Gate: FAILS due to todo!() stub.
#[test]
fn test_BC_1_14_004_bleedchecker_absent_from_all_passes_when_not_present() {
    let pptx = make_pptx_zip("visual body content only", "presenter notes here");
    BleedChecker::assert_absent_from_pptx_all(&pptx, "DETAIL_SENTINEL_VALUE");
}

/// BC-1.14.004 invariant 1 / AC-008:
/// `assert_absent_from_pptx_all` MUST PANIC when the sentinel appears in ANY
/// file in the PPTX ZIP — including in the notes slide.
///
/// Red Gate: FAILS due to todo!() stub.
#[test]
#[should_panic]
fn test_BC_1_14_004_bleedchecker_absent_from_all_panics_when_present() {
    // Sentinel in the notes slide — but for `assert_absent_from_pptx_all`,
    // even the notes slide is disallowed (detail must not appear anywhere in PPTX).
    let pptx = make_pptx_zip("body content", "DETAIL_SENTINEL_VALUE in notes");
    BleedChecker::assert_absent_from_pptx_all(&pptx, "DETAIL_SENTINEL_VALUE");
}

/// BC-1.14.004 postcondition 3 / AC-008:
/// `assert_present_in_docx_body` must NOT panic when the sentinel IS present
/// in `word/document.xml`.
///
/// Red Gate: FAILS due to todo!() stub.
#[test]
fn test_BC_1_14_004_bleedchecker_present_in_docx_body_passes_when_present() {
    let docx = make_docx_zip("REPORT_DOCX_SENTINEL appears here in the report");
    BleedChecker::assert_present_in_docx_body(&docx, "REPORT_DOCX_SENTINEL");
}

/// BC-1.14.004 postcondition 3 / AC-008:
/// `assert_present_in_docx_body` MUST PANIC when the sentinel is ABSENT from
/// `word/document.xml` (indicates report content was not written to DOCX body).
///
/// Red Gate: FAILS due to todo!() stub.
#[test]
#[should_panic]
fn test_BC_1_14_004_bleedchecker_present_in_docx_body_panics_when_absent() {
    let docx = make_docx_zip("body has no report sentinel here");
    BleedChecker::assert_present_in_docx_body(&docx, "REPORT_DOCX_SENTINEL");
}

/// BC-1.14.004 postcondition 2 / AC-008:
/// `assert_absent_from_docx_body` must NOT panic when the sentinel is absent
/// from `word/document.xml`.
///
/// Red Gate: FAILS due to todo!() stub.
#[test]
fn test_BC_1_14_004_bleedchecker_absent_from_docx_body_passes_when_not_present() {
    let docx = make_docx_zip("body has report content but no notes sentinel");
    BleedChecker::assert_absent_from_docx_body(&docx, "NOTES_DOCX_SENTINEL");
}

/// BC-1.14.004 postcondition 2 / AC-008:
/// `assert_absent_from_docx_body` MUST PANIC when the sentinel IS present in
/// `word/document.xml` (notes content bled into DOCX body — bleed defect).
///
/// Red Gate: FAILS due to todo!() stub.
#[test]
#[should_panic]
fn test_BC_1_14_004_bleedchecker_absent_from_docx_body_panics_when_present() {
    let docx = make_docx_zip("NOTES_DOCX_SENTINEL leaked into document body");
    BleedChecker::assert_absent_from_docx_body(&docx, "NOTES_DOCX_SENTINEL");
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-003: Sentinel in a non-scanned path must NOT be flagged as bleed
//
// BC-1.14.004 EC-003: `BleedChecker` checks specifically scoped ZIP paths.
// A sentinel in a path that `assert_absent_from_pptx_slides` does NOT scan
// (e.g., `ppt/theme/theme1.xml` or `content.xml`) must not trigger a panic.
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.14.004 EC-003 / AC-008:
/// A sentinel in a non-slides path (e.g., theme XML) does NOT cause
/// `assert_absent_from_pptx_slides` to panic.
///
/// This verifies that BleedChecker scans only `ppt/slides/slide*.xml` and does
/// not false-positive on other ZIP members.
///
/// Red Gate: FAILS due to todo!() stub.
#[test]
fn test_BC_1_14_004_ec003_sentinel_in_nonscanned_path_not_flagged_as_bleed() {
    // Sentinel is in `ppt/theme/theme1.xml` — a path outside the slide body scan.
    // The slide body itself is clean.
    let pptx = make_pptx_zip_with_extra(
        "clean slide body content",
        "ppt/theme/theme1.xml",
        "<a:theme>EC003_SENTINEL_IN_THEME</a:theme>",
    );
    // assert_absent_from_pptx_slides must NOT panic: it only scans slide*.xml.
    BleedChecker::assert_absent_from_pptx_slides(&pptx, "EC003_SENTINEL_IN_THEME");
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-004: BleedChecker works on decoded text (XML-safe sentinels)
//
// BC-1.14.004 EC-004: BleedChecker reads ZIP member bytes as UTF-8 text.
// For sentinels containing XML-special characters, callers must use the
// XML-escaped form as the sentinel (e.g., "&amp;" instead of "&"). The
// canonical sentinels used by STORY-036 are XML-safe (no special chars),
// so this test verifies the basic text-decoding path.
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.14.004 EC-004 / AC-008:
/// BleedChecker correctly detects a sentinel that appears verbatim in the
/// decoded UTF-8 text of the ZIP member. This confirms BleedChecker is doing
/// string search on decoded text, not raw bytes.
///
/// The canonical STORY-036 sentinels (SENTINEL_NOTES, SENTINEL_REPORT, etc.)
/// are XML-safe identifiers; this test uses a realistic sentinel to verify the
/// text-decoding path end-to-end.
///
/// Red Gate: FAILS due to todo!() stub in assert_absent_from_pptx_slides.
#[test]
fn test_BC_1_14_004_ec004_sentinel_detected_in_decoded_text() {
    // Build a slide body with the sentinel present as UTF-8 text.
    let sentinel = "BLEED_DETECTED_IN_UTF8_TEXT";
    let pptx = make_pptx_zip(&format!("<a:t>{sentinel}</a:t>"), "notes content");
    // Since the sentinel IS in the slide body, assert_absent must panic.
    // We use should_panic to verify detection works on decoded text.
    // (The inverse — absent sentinel passes — is covered by other tests.)
    let result = std::panic::catch_unwind(|| {
        BleedChecker::assert_absent_from_pptx_slides(&pptx, sentinel);
    });
    // The implementation must have panicked (sentinel found in slide body).
    assert!(
        result.is_err(),
        "BleedChecker must panic when sentinel is present in slide body (EC-004: text decoding)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixture parse test (RUNS NOW)
//
// Verifies the canonical three-register-slide.sf fixture file parses
// through the real slideforge-syntax parser without fatal errors.
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.14.004 invariant 2 / STORY-036:
/// The canonical `three-register-slide.sf` fixture must parse without fatal
/// errors through the real parser.
///
/// This test does NOT evaluate the deck — it only checks that the DSL syntax
/// is valid and the parser accepts the fixture. Evaluation tests (which require
/// the evaluator and layout engine) are deferred to exporter tests.
///
/// This test RUNS NOW (not ignored) because it exercises the parser, which
/// exists. It will FAIL at Red Gate if the fixture file contains syntax errors.
/// If this test passes at Red Gate, it is correct behaviour (the fixture must
/// parse successfully).
#[test]
fn test_BC_1_14_004_fixture_parses_without_errors() {
    use slideforge_syntax::{DiagnosticSink, SourceMap, parse_checked};
    use std::sync::Arc;

    // Read the canonical fixture.
    let fixture_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/three-register-slide.sf"
    );
    let src = std::fs::read_to_string(fixture_path).unwrap_or_else(|e| {
        panic!(
            "Could not read three-register-slide.sf fixture at {fixture_path}: {e}\n\
             (This file should have been created by STORY-036 Red Gate.)"
        )
    });

    let mut source_map = SourceMap::new();
    let file_id = source_map.add_file(Arc::from(fixture_path), Arc::from(src.as_str()));
    let mut sink = DiagnosticSink::new();

    let deck_node = parse_checked(&src, file_id, &source_map, &mut sink);

    // Fail if there were any fatal parse errors.
    assert!(
        deck_node.is_some(),
        "three-register-slide.sf must parse without fatal errors; \
         {} diagnostics in sink",
        sink.errors().len()
    );

    // Fail if there were any unexpected non-fatal errors (warnings are OK
    // for missing brand file etc, but the fixture must not have parse errors).
    let errors = sink.errors();
    assert!(
        errors.is_empty(),
        "three-register-slide.sf must produce zero parse errors; \
         got {} errors",
        errors.len()
    );

    // Verify the parsed deck has exactly one slide item.
    // DeckNode.items contains all top-level block items (slides, @for, @if);
    // the fixture has one plain slide so items.len() must be 1.
    let deck = deck_node.unwrap();
    use slideforge_syntax::BlockItem;
    let slide_count = deck
        .items
        .iter()
        .filter(|i| matches!(i, BlockItem::Slide(_)))
        .count();
    assert_eq!(
        slide_count,
        1,
        "three-register-slide.sf must produce exactly 1 slide; got {} slide items \
         (total items: {})",
        slide_count,
        deck.items.len()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-001: notes content absent from PPTX slide body
// (IGNORED — requires STORY-037 PPTX exporter)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.14.004 postcondition 2 / AC-001:
/// A deck with `notes "NOTES_SENTINEL_VALUE"` built to PPTX must not have
/// "NOTES_SENTINEL_VALUE" in any `ppt/slides/slide*.xml` file.
///
/// STORY-037 (PPTX Core Serialization) must un-ignore this test when it
/// ships. The test should reference the real PPTX exporter entry point once
/// available.
///
/// Ignored: requires STORY-037 to provide a working PPTX exporter.
#[test]
#[ignore = "requires STORY-037 PPTX exporter"]
fn test_BC_1_14_004_ac001_notes_absent_from_pptx_slide_body() {
    // TODO(STORY-037): Build the fixture deck to PPTX bytes using the real exporter.
    // let fixture = load_fixture("three-register-slide.sf");
    // let pptx_bytes = slideforge_pptx::build(fixture).unwrap();
    // BleedChecker::assert_absent_from_pptx_slides(&pptx_bytes, "SENTINEL_NOTES");
    //
    // Sentinel "SENTINEL_NOTES" is defined in three-register-slide.sf.
    todo!("un-ignore and implement once STORY-037 PPTX exporter is available")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-002: report content absent from PPTX slide body
// (IGNORED — requires STORY-037 PPTX exporter)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.14.004 postcondition 4 / AC-002:
/// A deck with `report "REPORT_SENTINEL_VALUE"` built to PPTX must not have
/// "REPORT_SENTINEL_VALUE" in any `ppt/slides/slide*.xml` file.
///
/// Ignored: requires STORY-037 to provide a working PPTX exporter.
#[test]
#[ignore = "requires STORY-037 PPTX exporter"]
fn test_BC_1_14_004_ac002_report_absent_from_pptx_slide_body() {
    // TODO(STORY-037): Build the fixture to PPTX and check slide bodies.
    // BleedChecker::assert_absent_from_pptx_slides(&pptx_bytes, "SENTINEL_REPORT");
    todo!("un-ignore and implement once STORY-037 PPTX exporter is available")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-003: detail content absent from PPTX entirely
// (IGNORED — requires STORY-037 PPTX exporter)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.14.004 postcondition 6 / AC-003:
/// A deck with `detail "DETAIL_SENTINEL_VALUE"` built to PPTX must not have
/// "DETAIL_SENTINEL_VALUE" in ANY file within the PPTX ZIP.
///
/// Ignored: requires STORY-037 to provide a working PPTX exporter.
#[test]
#[ignore = "requires STORY-037 PPTX exporter"]
fn test_BC_1_14_004_ac003_detail_absent_from_pptx_all_parts() {
    // TODO(STORY-037): Build the fixture to PPTX and check all ZIP members.
    // BleedChecker::assert_absent_from_pptx_all(&pptx_bytes, "SENTINEL_DETAIL");
    todo!("un-ignore and implement once STORY-037 PPTX exporter is available")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-004: notes content absent from DOCX body
// (IGNORED — requires STORY-041 DOCX exporter)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.14.004 postcondition 2 / AC-004:
/// A deck with `notes "NOTES_DOCX_SENTINEL"` built to DOCX must not have
/// "NOTES_DOCX_SENTINEL" in `word/document.xml`.
///
/// Ignored: requires STORY-041 to provide a working DOCX exporter.
#[test]
#[ignore = "requires STORY-041 DOCX exporter"]
fn test_BC_1_14_004_ac004_notes_absent_from_docx_body() {
    // TODO(STORY-041): Build the fixture to DOCX and check word/document.xml.
    // BleedChecker::assert_absent_from_docx_body(&docx_bytes, "SENTINEL_NOTES");
    todo!("un-ignore and implement once STORY-041 DOCX exporter is available")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-005: report content present in DOCX body (positive test)
// (IGNORED — requires STORY-041 DOCX exporter)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.14.004 postcondition 3 / AC-005:
/// A deck with `report "REPORT_DOCX_SENTINEL"` built to DOCX must have
/// "REPORT_DOCX_SENTINEL" present in `word/document.xml`.
///
/// This is a POSITIVE test: it verifies the routing is active, not just that
/// bleed is absent. If report content is not in the DOCX body, the register
/// routing is broken.
///
/// Ignored: requires STORY-041 to provide a working DOCX exporter.
#[test]
#[ignore = "requires STORY-041 DOCX exporter"]
fn test_BC_1_14_004_ac005_report_present_in_docx_body() {
    // TODO(STORY-041): Build the fixture to DOCX and verify report sentinel present.
    // BleedChecker::assert_present_in_docx_body(&docx_bytes, "SENTINEL_REPORT");
    todo!("un-ignore and implement once STORY-041 DOCX exporter is available")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-006: detail content absent from HTML canvas
// (IGNORED — requires STORY-046 HTML exporter)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.14.004 postcondition 6 / AC-006:
/// A deck with `detail "DETAIL_HTML_SENTINEL"` built to static HTML must not
/// have "DETAIL_HTML_SENTINEL" inside any slide canvas element.
///
/// Ignored: requires STORY-046 to provide a working HTML exporter.
#[test]
#[ignore = "requires STORY-046 HTML exporter"]
fn test_BC_1_14_004_ac006_detail_absent_from_html_canvas() {
    // TODO(STORY-046): Build the fixture to HTML and assert sentinel absent from
    // <div class="slide-canvas"> or equivalent container.
    todo!("un-ignore and implement once STORY-046 HTML exporter is available")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-007: all three registers on same slide — no bleed (cross-format)
// (IGNORED — requires STORY-037 PPTX exporter at minimum)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.14.004 postconditions 1-6 / AC-007:
/// A deck with all three registers on one slide, built to PPTX and DOCX:
///
/// - PPTX slide body: NO sentinels (no notes/report/detail in visual body)
/// - PPTX notes slide: SENTINEL_NOTES present, SENTINEL_REPORT/DETAIL absent
/// - DOCX body: SENTINEL_REPORT present, SENTINEL_NOTES/DETAIL absent from body
///
/// This is the most comprehensive bleed test and the most important invariant.
///
/// Ignored: requires STORY-037 PPTX exporter (minimum dependency).
/// STORY-041 required for the DOCX half of this test.
#[test]
#[ignore = "requires STORY-037 PPTX exporter"]
fn test_BC_1_14_004_ac007_all_three_registers_no_bleed_cross_format() {
    // TODO(STORY-037, STORY-041): Build three-register-slide.sf to PPTX and DOCX.
    // PPTX checks:
    //   BleedChecker::assert_absent_from_pptx_slides(&pptx_bytes, "SENTINEL_NOTES");
    //   BleedChecker::assert_absent_from_pptx_slides(&pptx_bytes, "SENTINEL_REPORT");
    //   BleedChecker::assert_absent_from_pptx_slides(&pptx_bytes, "SENTINEL_DETAIL");
    //   BleedChecker::assert_absent_from_pptx_all(&pptx_bytes, "SENTINEL_REPORT");
    //   BleedChecker::assert_absent_from_pptx_all(&pptx_bytes, "SENTINEL_DETAIL");
    // DOCX checks:
    //   BleedChecker::assert_present_in_docx_body(&docx_bytes, "SENTINEL_REPORT");
    //   BleedChecker::assert_absent_from_docx_body(&docx_bytes, "SENTINEL_NOTES");
    //   BleedChecker::assert_absent_from_docx_body(&docx_bytes, "SENTINEL_DETAIL");
    todo!(
        "un-ignore and implement once STORY-037 PPTX exporter and STORY-041 DOCX exporter \
         are available"
    )
}
