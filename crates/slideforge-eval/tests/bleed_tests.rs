//! Bleed invariant tests for BC-1.14.004 — No Register Content Bleeds to
//! Wrong Format. (STORY-036)
//!
//! # Test organisation
//!
//! Tests are split into two groups:
//!
//! ## Group 1: AC-008 BleedChecker utility unit tests (RUNS NOW — PASSING)
//!
//! These tests construct minimal synthetic PPTX-like / DOCX-like ZIP bytes
//! in-memory using the `zip` crate and exercise `BleedChecker`'s four methods.
//! They do not require any exporter crate. `BleedChecker` is fully implemented
//! (the `todo!()` stubs from the Red Gate phase are gone); all tests in this
//! group pass. The decoder uses a per-token tolerant strategy (`decode_xml_entities`)
//! that decodes predefined entities and numeric refs correctly while mapping unknown
//! or malformed entities to U+FFFD — never returning Err for the whole member.
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
//! - `test_BC_1_14_004_ec003_sentinel_in_nonscanned_path_not_flagged_as_bleed` —
//!   a sentinel in a non-scanned path (`ppt/theme/theme1.xml`) must not trigger
//!   the slide check.
//! - `test_BC_1_14_004_ec003_sentinel_in_notes_slide_not_flagged_as_bleed` —
//!   a sentinel in `ppt/notesSlides/notesSlide1.xml` must not trigger the slide
//!   check (the spec's canonical EC-003 scenario).
//! - `test_BC_1_14_004_ec004_xml_escaped_content_detected` — BleedChecker
//!   entity-decodes scanned text, so `R&D roadmap` is detected even when the
//!   member contains `R&amp;D roadmap` (genuine XML-escaping coverage).
//! - `test_BC_1_14_004_ec004_lt_gt_escaped_content_detected` — same for
//!   `<item>` serialised as `&lt;item&gt;`.
//! - `test_BC_1_14_004_ec004_unescaped_sentinel_absent_passes` — an absence
//!   check for an unescaped sentinel that genuinely does not appear in the
//!   decoded text must pass without panicking.
//!
//! ## F-003 zero-member guard tests (RUNS NOW)
//!
//! - `test_BC_1_14_004_absent_from_slides_panics_on_slideless_pptx` — a PPTX
//!   with no slide bodies makes the slide absence check panic (fail-closed).
//! - `test_BC_1_14_004_absent_from_all_panics_on_empty_archive` — an empty
//!   ZIP makes the all-members absence check panic (fail-closed).
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
//! | `test_BC_1_14_004_absent_from_slides_panics_on_slideless_pptx` | BC-1.14.004 EC-004 | AC-008 |
//! | `test_BC_1_14_004_absent_from_all_panics_on_empty_archive` | BC-1.14.004 EC-004 | AC-008 |
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

/// Build a ZIP with only non-slide members (no `ppt/slides/slideN.xml`).
///
/// Used for F-003 testing: `assert_absent_from_pptx_slides` must fail-closed
/// when given a PPTX with no slide bodies.
fn make_pptx_zip_no_slides() -> Vec<u8> {
    let buf = Vec::new();
    let cursor = std::io::Cursor::new(buf);
    let mut zip = zip::ZipWriter::new(cursor);
    let opts =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Only a theme file — no slide bodies.
    zip.start_file("ppt/theme/theme1.xml", opts).unwrap();
    zip.write_all(b"<a:theme/>").unwrap();

    let cursor = zip.finish().unwrap();
    cursor.into_inner()
}

/// Build a ZIP with zero members.
///
/// Used for F-003 testing: `assert_absent_from_pptx_all` must fail-closed
/// when given an empty archive.
fn make_empty_zip() -> Vec<u8> {
    let buf = Vec::new();
    let cursor = std::io::Cursor::new(buf);
    let zip = zip::ZipWriter::new(cursor);
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
#[should_panic(expected = "BleedChecker: register-content bleed detected in PPTX slide body")]
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
#[should_panic(expected = "BleedChecker: register-content bleed detected in PPTX archive")]
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
#[should_panic(expected = "BleedChecker: expected register content NOT found in DOCX body")]
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
#[should_panic(expected = "BleedChecker: register-content bleed detected in DOCX body")]
fn test_BC_1_14_004_bleedchecker_absent_from_docx_body_panics_when_present() {
    let docx = make_docx_zip("NOTES_DOCX_SENTINEL leaked into document body");
    BleedChecker::assert_absent_from_docx_body(&docx, "NOTES_DOCX_SENTINEL");
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-003: Sentinel in a non-scanned path must NOT be flagged as bleed
//
// BC-1.14.004 EC-003: `BleedChecker` checks specifically scoped ZIP paths.
// A sentinel in a path that `assert_absent_from_pptx_slides` does NOT scan
// must not trigger a panic.
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.14.004 EC-003 / AC-008:
/// A sentinel in a non-slides path (e.g., `ppt/theme/theme1.xml`) does NOT
/// cause `assert_absent_from_pptx_slides` to panic.
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

/// BC-1.14.004 EC-003 / AC-008 — notesSlides scenario:
/// A sentinel in `ppt/notesSlides/notesSlide1.xml` does NOT cause
/// `assert_absent_from_pptx_slides` to panic.
///
/// This is the spec's canonical EC-003 scenario: a sentinel legitimately
/// present in notesSlides must not be detected as a slide-body bleed.
/// The slide body itself is clean; only the notes slide carries the sentinel.
///
/// `assert_absent_from_pptx_slides` only scans `ppt/slides/slide*.xml`,
/// so the notes slide is not in scope.
#[test]
fn test_BC_1_14_004_ec003_sentinel_in_notes_slide_not_flagged_as_bleed() {
    // Build a PPTX where the notes slide holds the sentinel but the slide body
    // is clean. make_pptx_zip places the second argument in notesSlide1.xml.
    let pptx = make_pptx_zip("clean slide body", "EC003_SENTINEL_IN_NOTES");
    // assert_absent_from_pptx_slides must NOT panic because it only scans
    // ppt/slides/slide*.xml; notesSlides is out of scope.
    BleedChecker::assert_absent_from_pptx_slides(&pptx, "EC003_SENTINEL_IN_NOTES");
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-004: BleedChecker entity-decodes XML text before searching
//
// BC-1.14.004 EC-004: BleedChecker must find sentinels even when the exporter
// has XML-escaped them. These tests are GENUINE (non-tautological): they embed
// only the ESCAPED form in the ZIP member and assert that the UNESCAPED sentinel
// is detected. With the old implementation (no entity decoding) these tests
// would fail (false negative). After the F-001 fix they must pass.
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.14.004 EC-004 / AC-008 — ampersand escaping:
/// BleedChecker detects `R&D roadmap` even when the slide member contains
/// `R&amp;D roadmap` (the XML-escaped form a conforming exporter would produce).
///
/// With the OLD implementation (no entity decoding) this test demonstrates a
/// FALSE NEGATIVE: `assert_absent_from_pptx_all` would NOT panic even though
/// the content `R&D roadmap` is present (escaped as `R&amp;D roadmap`), silently
/// missing a real P0 bleed.
///
/// After the F-001 fix (entity decoding via quick_xml::escape::unescape),
/// the decoded text is `R&D roadmap` and the sentinel IS found → panic.
#[test]
fn test_BC_1_14_004_ec004_xml_escaped_content_detected() {
    // Embed ONLY the XML-escaped form in the slide body.
    // A conforming PPTX exporter writing "R&D roadmap" would produce this.
    let pptx = make_pptx_zip(
        r"<a:t>R&amp;D roadmap</a:t>",
        "notes content without sentinel",
    );

    // The sentinel is the HUMAN-READABLE unescaped string.
    // assert_absent_from_pptx_all must PANIC because the decoded slide body
    // contains "R&D roadmap". If entity decoding is absent, this wrongly passes.
    let result = std::panic::catch_unwind(|| {
        BleedChecker::assert_absent_from_pptx_all(&pptx, "R&D roadmap");
    });
    assert!(
        result.is_err(),
        "BleedChecker must detect 'R&D roadmap' even when escaped as 'R&amp;D roadmap' \
         (EC-004: entity decoding prevents false negatives)"
    );

    // Also verify the slide-scoped check detects it.
    let result2 = std::panic::catch_unwind(|| {
        BleedChecker::assert_absent_from_pptx_slides(&pptx, "R&D roadmap");
    });
    assert!(
        result2.is_err(),
        "assert_absent_from_pptx_slides must also detect 'R&D roadmap' via entity decoding"
    );
}

/// BC-1.14.004 EC-004 / AC-008 — less-than / greater-than escaping:
/// BleedChecker detects `<item>` even when the member contains `&lt;item&gt;`
/// (the XML-escaped form). Covers `&lt;` and `&gt;` entity references.
#[test]
fn test_BC_1_14_004_ec004_lt_gt_escaped_content_detected() {
    // Embed only the escaped form.
    let pptx = make_pptx_zip(r"<a:t>&lt;item&gt;</a:t>", "notes content");

    // The sentinel is the unescaped form; must be detected after decoding.
    let result = std::panic::catch_unwind(|| {
        BleedChecker::assert_absent_from_pptx_slides(&pptx, "<item>");
    });
    assert!(
        result.is_err(),
        "BleedChecker must detect '<item>' even when escaped as '&lt;item&gt;' \
         (EC-004: &lt;/&gt; entity decoding)"
    );
}

/// BC-1.14.004 EC-004 / AC-008 — positive direction (no false positives):
/// An absence check for a sentinel that genuinely does NOT appear in the
/// decoded text must pass without panicking.
///
/// This guards against the entity-decoding implementation accidentally
/// matching the wrong sentinel.
#[test]
fn test_BC_1_14_004_ec004_unescaped_sentinel_absent_passes() {
    // Slide body contains `R&amp;D roadmap` (= decoded `R&D roadmap`).
    // We check for a DIFFERENT sentinel — must not panic.
    let pptx = make_pptx_zip(r"<a:t>R&amp;D roadmap</a:t>", "notes content");
    // "DETAIL_SENTINEL" is not in the decoded text — absence check must pass.
    BleedChecker::assert_absent_from_pptx_slides(&pptx, "DETAIL_SENTINEL");
}

/// BC-1.14.004 EC-004 / AC-008 — tolerant per-entity decoding (F-036-002):
/// An unknown named entity (`&copy;`) adjacent to a correctly-escaped sentinel
/// (`R&amp;D`) in the same `<a:t>` element must NOT prevent the sentinel from
/// being detected.
///
/// This is the root bug in the previous all-or-nothing implementation: if any
/// entity in the member was unrecognised, `quick_xml::escape::unescape` returned
/// `Err` for the WHOLE member, which caused the code to fall back to raw text.
/// Raw text for `R&amp;D` does NOT contain `R&D`, so the bleed was silently missed.
///
/// The fix uses a per-token tolerant decoder (`decode_xml_entities`) where unknown
/// named entities decode to U+FFFD (the Unicode replacement character) rather than
/// the empty string (not `Some("")` — using the empty string would cause false-positive
/// joins like `A&copy;B` → `AB`). After the fix:
/// - Decoded text: `&copy;` → `\u{FFFD}`, `R&amp;D` → `R&D` → sentinel found.
/// - Only the decoded text is searched (the raw-search branch was removed, F-P4-001).
///
/// This test MUST FAIL against the old all-or-nothing fallback implementation
/// and MUST PASS after the per-token tolerant-decode fix.
#[test]
fn test_BC_1_14_004_ec004_unknown_entity_does_not_mask_adjacent_sentinel() {
    // Build a PPTX slide member containing BOTH an unknown entity (`&copy;`) and
    // the XML-escaped form of the sentinel (`R&amp;D`) in the same text run.
    let pptx = make_pptx_zip(
        r"<a:t>&copy; R&amp;D roadmap &amp; analysis</a:t>",
        "notes without sentinel",
    );

    // The sentinel is the human-readable unescaped string.
    // assert_absent_from_pptx_all must PANIC: decoded slide body contains "R&D roadmap".
    // An all-or-nothing decoder would fall back to raw text (no `R&D` there) → false green.
    let result = std::panic::catch_unwind(|| {
        BleedChecker::assert_absent_from_pptx_all(&pptx, "R&D roadmap");
    });
    assert!(
        result.is_err(),
        "BleedChecker must detect 'R&D roadmap' even when an unknown entity (&copy;) \
         appears in the same member — tolerant per-entity decoding required (F-036-002)"
    );

    // Also verify the slide-scoped check detects it.
    let result2 = std::panic::catch_unwind(|| {
        BleedChecker::assert_absent_from_pptx_slides(&pptx, "R&D roadmap");
    });
    assert!(
        result2.is_err(),
        "assert_absent_from_pptx_slides must also detect 'R&D roadmap' via tolerant \
         per-entity decoding when an unrelated unknown entity is present (F-036-002)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-PASS3-001: Unknown-entity placeholder tests — U+FFFD, not empty string
//
// These three tests verify that unknown XML entities (e.g. `&copy;`) are
// decoded to the Unicode replacement character U+FFFD rather than the empty
// string. Using the empty string causes two categories of bug:
//   (a) False positive: `A&copy;B` → `AB` triggers `assert_absent` panic
//       even though `AB` is not semantically present.
//   (b) False green: `R&copy;D` → `RD` passes `assert_present(.., "RD")`
//       when `RD` was never actually written — hiding a routing failure.
//
// With U+FFFD: `A&copy;B` → `A\u{FFFD}B` (not `AB`); `R&copy;D` →
// `R\u{FFFD}D` (not `RD`). These tests MUST FAIL against the `.or(Some(""))`
// implementation and MUST PASS after the U+FFFD placeholder fix.
// ─────────────────────────────────────────────────────────────────────────────

/// F-PASS3-001 (a) — no false positive:
/// `assert_absent_from_pptx_slides(.., "AB")` must NOT panic when the slide
/// member contains `A&copy;B` (unknown entity between A and B).
///
/// With `.or(Some(""))`, `&copy;` decodes to `""` → `AB` is formed → panic
/// (spurious RED). With U+FFFD, the decoded text is `A\u{FFFD}B` which does
/// NOT contain `AB` → no panic (correct).
///
/// This test FAILS against the empty-string fallback and PASSES after the fix.
#[test]
fn test_BC_1_14_004_unknown_entity_placeholder_no_false_positive_absent_check() {
    // Slide member contains A followed by &copy; followed by B.
    // The human-readable sentinel to check absence of is "AB".
    let pptx = make_pptx_zip(r"<a:t>A&copy;B</a:t>", "notes without sentinel");
    // Must NOT panic — "AB" is not semantically present; the unknown entity
    // separates A and B and must decode to a non-joining placeholder (U+FFFD),
    // not the empty string.
    BleedChecker::assert_absent_from_pptx_slides(&pptx, "AB");
}

/// F-PASS3-001 (b) — no false green:
/// `assert_present_in_docx_body(.., "RD")` must PANIC when `word/document.xml`
/// contains only `R&copy;D` (unknown entity between R and D, no literal `RD`).
///
/// With `.or(Some(""))`, `&copy;` decodes to `""` → decoded text is `RD` →
/// the check PASSES (false green: routing failure is silently hidden). With
/// U+FFFD, the decoded text is `R\u{FFFD}D` which does NOT contain `RD` → the
/// check correctly PANICS (routing failure detected).
///
/// This test FAILS against the empty-string fallback (the check incorrectly
/// passes) and PASSES after the fix (the check correctly panics).
#[test]
#[should_panic(expected = "BleedChecker: expected register content NOT found in DOCX body")]
fn test_BC_1_14_004_unknown_entity_placeholder_no_false_green_present_check() {
    // word/document.xml contains ONLY R followed by &copy; followed by D.
    // There is no literal "RD" substring anywhere in the raw or decoded text
    // (after the U+FFFD fix). We check that "RD" is present — it must NOT be
    // found because R and D are separated by the unknown entity placeholder.
    let docx = make_docx_zip(r"R&copy;D and nothing else matching");
    BleedChecker::assert_present_in_docx_body(&docx, "RD");
}

/// F-PASS3-001 (c) — no false negative (regression guard):
/// The existing `&copy; R&amp;D roadmap` → detect `R&D roadmap` test must
/// still work after the U+FFFD fix. The placeholder must not break detection
/// of legitimately escaped predefined entities.
///
/// `&copy;` → `\u{FFFD}`, `R&amp;D` → `R&D`. The decoded text contains
/// `R&D roadmap` as a substring → sentinel IS found → assert_absent panics.
#[test]
fn test_BC_1_14_004_unknown_entity_placeholder_no_false_negative_regression() {
    // Slide member: &copy; (unknown) followed by R&amp;D roadmap (escaped sentinel).
    let pptx = make_pptx_zip(
        r"<a:t>&copy; R&amp;D roadmap &amp; analysis</a:t>",
        "notes without sentinel",
    );
    // assert_absent_from_pptx_all must PANIC: decoded text contains "R&D roadmap".
    let result = std::panic::catch_unwind(|| {
        BleedChecker::assert_absent_from_pptx_all(&pptx, "R&D roadmap");
    });
    assert!(
        result.is_err(),
        "Regression: BleedChecker must still detect 'R&D roadmap' after U+FFFD fix — \
         the placeholder must not break detection of predefined entity decoding (F-PASS3-001 c)"
    );
    // Also verify the slide-scoped check detects it.
    let result2 = std::panic::catch_unwind(|| {
        BleedChecker::assert_absent_from_pptx_slides(&pptx, "R&D roadmap");
    });
    assert!(
        result2.is_err(),
        "Regression: assert_absent_from_pptx_slides must also detect 'R&D roadmap' \
         via predefined entity decoding (U+FFFD fix must not break this — F-PASS3-001 c)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-003: Zero-member guard tests — absence checks must fail-closed
//
// assert_absent_from_pptx_slides and assert_absent_from_pptx_all must NOT
// silently return "absent" when they scan zero members. A valid PPTX has ≥1
// slide body; an empty archive is not a valid PPTX.
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.14.004 / AC-008 — F-003 guard:
/// `assert_absent_from_pptx_slides` must PANIC when given a PPTX-shaped ZIP
/// that contains no `ppt/slides/slide*.xml` members.
///
/// A PPTX with zero slide bodies is either malformed or indicates an exporter
/// structural regression. An absence check on zero members would be vacuously
/// true — silently hiding the defect. The checker must fail-closed.
#[test]
#[should_panic(expected = "BleedChecker: no ppt/slides/slide*.xml members found")]
fn test_BC_1_14_004_absent_from_slides_panics_on_slideless_pptx() {
    let pptx = make_pptx_zip_no_slides();
    // Must panic: no slide bodies found, fail-closed.
    BleedChecker::assert_absent_from_pptx_slides(&pptx, "ANY_SENTINEL");
}

/// BC-1.14.004 / AC-008 — F-003 guard:
/// `assert_absent_from_pptx_all` must PANIC when given an empty ZIP archive
/// (zero members).
///
/// An empty archive means nothing was scanned, so the absence check would be
/// vacuously true. The checker must fail-closed to expose this condition.
#[test]
#[should_panic(expected = "BleedChecker: PPTX archive contains zero members")]
fn test_BC_1_14_004_absent_from_all_panics_on_empty_archive() {
    let empty = make_empty_zip();
    // Must panic: empty archive, fail-closed.
    BleedChecker::assert_absent_from_pptx_all(&empty, "ANY_SENTINEL");
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

    // Fail if there were any FATAL-severity errors (warnings are acceptable
    // for things like a missing brand file, but the fixture must not have
    // fatal parse errors). We gate on has_fatal() rather than errors.is_empty()
    // because DiagnosticSink::errors() returns ALL diagnostics regardless of
    // severity — asserting it is empty would spuriously fail if the parser
    // ever emits a benign WARNING for this fixture (F-PASS3-002).
    assert!(
        !sink.has_fatal(),
        "three-register-slide.sf must produce zero fatal parse errors; \
         {} diagnostic(s) in sink (use has_fatal() to gate on severity, \
         not errors().is_empty() which rejects warnings)",
        sink.errors().len()
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

// ─────────────────────────────────────────────────────────────────────────────
// F-P4-005: Regression pin tests — fail against old dual-search / whole-member
// fallback, pass after per-token tolerant decoder + decoded-only search.
//
// These three tests pin the exact bugs identified in the Phase 4 adversarial
// pass (F-P4-001 and F-P4-003). They are written as first-class tests
// (not #[ignore]'d) so any future regression immediately breaks CI.
//
// Test 1 — Escape-machinery collision (F-P4-001):
//   With the OLD dual-search: member `R&amp;D` (no literal "amp" text), sentinel
//   "amp". The raw bytes contain the literal substring "amp" (inside "&amp;D").
//   `|| raw.contains("amp")` → true → absence check spuriously panics (false
//   positive). With decoded-only search: decoded text is `R&D`, which does NOT
//   contain "amp" → absence check correctly passes. Also verify presence check
//   for "amp" correctly reports absent (no panic on assert_absent_from_pptx_all).
//
// Test 2 — Escape-machinery collision for assert_absent_from_pptx_all
//   (same scenario, covering the all-members scan path).
//
// Test 3 — Malformed-numeric resilience (F-P4-003):
//   Member contains BOTH a malformed numeric ref `&#xZZ;` AND an escaped
//   sentinel `R&amp;D`. With the OLD whole-member fallback: unescape_with returns
//   Err on the malformed ref → decoded = raw_utf8 → decoded text still contains
//   "&amp;" → sentinel "R&D" NOT found in decoded → false GREEN (the real
//   escaped bleed is silently missed). With per-token decoder: `&#xZZ;` →
//   U+FFFD (one bad token, rest decoded normally) → `R&amp;D` → `R&D` → sentinel
//   IS found → absence check correctly panics.
// ─────────────────────────────────────────────────────────────────────────────

/// F-P4-005 / F-P4-001 — Escape-machinery collision, slides scan:
/// `assert_absent_from_pptx_slides("amp")` must NOT panic when the slide member
/// contains `R&amp;D` but no literal text "amp".
///
/// With the OLD dual-search (`|| raw.contains(sentinel)`): the raw bytes contain
/// the substring "amp" embedded in "&amp;D" → spurious panic (false positive).
/// With decoded-only search: decoded text is `R&D`, which does not contain "amp"
/// → absence check correctly passes (no panic).
///
/// This test FAILS against the old dual-search implementation (because the raw
/// bytes of "&amp;D" do contain "amp") and PASSES after the fix.
#[test]
fn test_BC_1_14_004_f_p4_005_escape_machinery_collision_slides_no_false_positive() {
    // Slide body contains ONLY R&amp;D — no literal text "amp" anywhere.
    let pptx = make_pptx_zip(r"<a:t>R&amp;D project</a:t>", "notes without any amp");
    // Sentinel "amp" must NOT be detected — it only exists in XML machinery
    // (&amp;), not in the decoded human-readable text.
    // Decoded: "R&D project" — does not contain "amp".
    // OLD dual-search raw bytes DO contain "amp" inside "&amp;" → false panic.
    BleedChecker::assert_absent_from_pptx_slides(&pptx, "amp");
}

/// F-P4-005 / F-P4-001 — Escape-machinery collision, all-members scan:
/// `assert_absent_from_pptx_all("amp")` must NOT panic when the only occurrence
/// of "amp" in the archive is inside XML escape machinery (`&amp;`).
///
/// Same bug as above but covering the `assert_absent_from_pptx_all` code path.
/// OLD dual-search: raw bytes of `&amp;D` contain "amp" → spurious panic.
/// Fixed decoded-only: decoded `R&D` does not contain "amp" → passes.
#[test]
fn test_BC_1_14_004_f_p4_005_escape_machinery_collision_all_no_false_positive() {
    // Both slide body and notes contain &amp; but no literal "amp" text.
    let pptx = make_pptx_zip(
        r"<a:t>R&amp;D project</a:t>",
        r"notes with R&amp;D reference",
    );
    // "amp" is only in XML machinery, not in decoded text.
    BleedChecker::assert_absent_from_pptx_all(&pptx, "amp");
}

/// F-P4-005 / F-P4-003 — Malformed numeric ref + escaped sentinel:
/// When a ZIP member contains BOTH a malformed numeric char ref (`&#xZZ;`) AND
/// an XML-escaped sentinel (`R&amp;D`), `assert_absent_from_pptx_all` MUST
/// PANIC — the real escaped bleed must be detected despite the malformed token.
///
/// With the OLD whole-member fallback: `unescape_with` returns `Err` because
/// `&#xZZ;` is malformed → `decoded` is set to `raw_utf8` (still escaped) →
/// decoded text contains `&amp;D` not `&D` → sentinel `R&D` not found → FALSE
/// GREEN (bleed silently missed).
///
/// With the per-token tolerant decoder: `&#xZZ;` → U+FFFD (one bad token,
/// processing continues) → `R&amp;D` → `R&D` → sentinel IS found → absence
/// check correctly panics (bleed detected).
///
/// This test FAILS against the old whole-member-fallback impl (the absence
/// check incorrectly passes — false green) and PASSES after the per-token fix
/// (the absence check correctly panics — real bleed detected).
#[test]
fn test_BC_1_14_004_f_p4_005_malformed_numeric_ref_does_not_mask_escaped_sentinel() {
    // Build a PPTX slide member containing:
    //   - &#xZZ; (malformed numeric char ref — not valid hex)
    //   - R&amp;D roadmap (correctly escaped sentinel)
    let pptx = make_pptx_zip(
        r"<a:t>&#xZZ; R&amp;D roadmap</a:t>",
        "notes without sentinel",
    );

    // assert_absent_from_pptx_all must PANIC: the decoded text must contain
    // "R&D roadmap" even though an unrelated malformed ref appears nearby.
    // OLD whole-member fallback: entire member falls back to raw bytes → false green.
    // Per-token decoder: &#xZZ; → U+FFFD, R&amp;D → R&D → bleed detected → panic.
    let result = std::panic::catch_unwind(|| {
        BleedChecker::assert_absent_from_pptx_all(&pptx, "R&D roadmap");
    });
    assert!(
        result.is_err(),
        "BleedChecker must detect 'R&D roadmap' even when a malformed numeric char ref \
         (&#xZZ;) appears in the same member — per-token tolerant decoding required (F-P4-003)"
    );

    // Also verify the slide-scoped check detects it.
    let result2 = std::panic::catch_unwind(|| {
        BleedChecker::assert_absent_from_pptx_slides(&pptx, "R&D roadmap");
    });
    assert!(
        result2.is_err(),
        "assert_absent_from_pptx_slides must also detect 'R&D roadmap' via per-token \
         decoding when a malformed numeric ref appears in the same member (F-P4-003)"
    );
}
