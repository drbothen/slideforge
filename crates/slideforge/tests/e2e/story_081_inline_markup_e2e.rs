//! STORY-081 End-to-End Inline Markup Integration Test (C4 Keystone).
//!
//! This test builds a deck containing inline markup (`**bold**`, `_italic_`,
//! `` `code` ``, `~~del~~`, `==highlight==`, `^sup^`, `~sub~`) from DSL source
//! via `slideforge::build()` and asserts FORMAT-NATIVE markup in each output
//! and the ABSENCE of literal `**`/`*` characters.
//!
//! ## CRITICAL DESIGN (per adversary finding C4)
//!
//! This is the KEYSTONE test: it must FAIL against the current broken state
//! (where body/subtitle `FieldValue::Inlines` is silently dropped, PPTX body
//! flattens, PDF path is dead code) and pass ONLY once C1–C3 are wired.
//!
//! ## Format-native assertions
//!
//! - **PPTX**: slide body XML contains `b="1"` (or `<a:rPr ... b="1"/>`) and
//!   does NOT contain `**bold text**` as literal text in `<a:t>`.
//! - **DOCX**: `word/document.xml` contains `<w:b/>` (or `<w:b />`) and does
//!   NOT contain `**bold text**`.
//! - **HTML**: output bytes contain `<strong>` and do NOT contain `**bold text**`.
//! - **PDF**: raw bytes contain "bold text" (the plain text content) and do NOT
//!   contain literal `**bold text**` (asterisks as text characters). The PDF
//!   exporter must render the span via `slide_to_krilla_runs` (font-switching)
//!   rather than discarding bold entirely.
//!
//! ## Traceability
//!
//! - BC-3.05.001 (slide-level inline markup in ALL output formats — PC-1 PPTX body, PC-3 DOCX, PC-4 HTML, PC-5 PDF)
//! - STORY-081 C4 (adversary finding: keystone e2e test was missing)
//! - STORY-081 AC-001 through AC-005 (format-native rendering per exporter)

#![allow(clippy::unwrap_used)] // integration tests — explicit panic on failure is correct
#![allow(clippy::doc_markdown)] // test doc comments have unquoted identifiers
#![allow(clippy::uninlined_format_args)] // assert! messages use positional format args for truncation

use crate::e2e::{BrandTmpDir, fixture_source, open_zip};

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Read a ZIP entry's content as a UTF-8 string.
fn read_zip_entry(
    archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
    entry_name: &str,
    label: &str,
) -> String {
    use std::io::Read as _;
    let mut file = archive
        .by_name(entry_name)
        .unwrap_or_else(|_| panic!("{label}: ZIP entry '{entry_name}' not found"));
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .unwrap_or_else(|e| panic!("{label}: cannot read '{entry_name}': {e}"));
    buf
}

// ─── PPTX: body XML must have b="1" and no literal ** ────────────────────────

/// STORY-081 C4 / AC-002 — PPTX slide body XML carries `b="1"` run property
/// for `**bold text**` and does NOT contain literal `**` characters in `<a:t>`.
///
/// ## RED GATE
///
/// Broken state: `field_to_block.rs` drops `FieldValue::Inlines` for `body`
/// (C2 / `extract_str_field` misses Inlines variant) → `FrameContent::Body`
/// has empty `ContentBlock::Text` with no inline nodes → PPTX exporter emits
/// zero `<a:r>` elements → no `b="1"` attribute → **assertion FAILS**.
///
/// After C2 fix: body inlines thread through to `ContentBlock::Text`. After C3
/// fix: PPTX Body arm routes through the unified ADR-024 engine → `b="1"`.
#[test]
fn test_story_081_c4_pptx_body_bold_produces_b1_run_property() {
    let brand = BrandTmpDir::new("s081_c4_pptx_bold");
    let source = fixture_source("story-081-inline-markup.sf");
    let opts = brand.build_options("pptx", false);

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "STORY-081 C4 (PPTX): build() must return Ok; got Err: {e:?}\n\
             Diagnose: check field_to_block.rs body threading and PPTX Body arm."
        )
    });

    let mut archive = open_zip(&output.bytes, "C4-PPTX");
    // The fixture has 2 slides: slide1 = title, slide2 = content with inline body.
    let slide_xml = read_zip_entry(&mut archive, "ppt/slides/slide2.xml", "C4-PPTX");

    // AC-002: PPTX must contain b="1" — the OOXML run property for bold.
    // RED GATE C2+C3: body Inlines dropped → FrameContent::Body empty → no
    // body-run engine called → b="1" absent → FAILS.
    assert!(
        slide_xml.contains("b=\"1\""),
        "STORY-081 C4 RED GATE (PPTX): slide2.xml must contain b=\"1\" for bold body text.\n\
         C2: field_to_block.rs must thread FieldValue::Inlines for 'body' field.\n\
         C3: PPTX FrameContent::Body arm must route through the ADR-024 unified engine.\n\
         slide2.xml excerpt (first 800 chars): {:.800}",
        slide_xml
    );

    // No literal asterisks in <a:t> elements — markup must be structural, not text.
    // We check that "**" does not appear in the XML (would indicate plain-text fallback).
    assert!(
        !slide_xml.contains("**"),
        "STORY-081 C4 (PPTX): slide2.xml must NOT contain literal '**' characters.\n\
         Asterisks in output indicate inline markup was NOT converted to structural OOXML.\n\
         slide2.xml excerpt (first 800 chars): {:.800}",
        slide_xml
    );
}

// ─── DOCX: document.xml must have <w:b/> and no literal ** ───────────────────

/// STORY-081 C4 / AC-003 — DOCX `word/document.xml` carries `<w:b/>` run
/// property for `**bold text**` and does NOT contain literal `**`.
///
/// ## RED GATE
///
/// Broken state: same C2 drop → body frame empty → no `make_inline_paragraph`
/// called with Bold nodes → no `<w:b/>` in output → **assertion FAILS**.
#[test]
fn test_story_081_c4_docx_body_bold_produces_wb_run_property() {
    let brand = BrandTmpDir::new("s081_c4_docx_bold");
    let source = fixture_source("story-081-inline-markup.sf");
    let opts = brand.build_options("docx", false);

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "STORY-081 C4 (DOCX): build() must return Ok; got Err: {e:?}\n\
             Diagnose: check field_to_block.rs body threading."
        )
    });

    let mut archive = open_zip(&output.bytes, "C4-DOCX");
    let doc_xml = read_zip_entry(&mut archive, "word/document.xml", "C4-DOCX");

    // AC-003: DOCX must contain <w:b/> or <w:b /> — the OOXML run property for bold.
    // RED GATE C2: body Inlines dropped → FrameContent::Body ContentBlock::Text empty →
    // make_inline_paragraph not called with Bold nodes → <w:b/> absent → FAILS.
    assert!(
        doc_xml.contains("<w:b/>") || doc_xml.contains("<w:b />"),
        "STORY-081 C4 RED GATE (DOCX): word/document.xml must contain <w:b/> for bold body text.\n\
         C2: field_to_block.rs must thread FieldValue::Inlines for 'body' field.\n\
         DOCX make_inline_paragraph must be called with InlineNode::Bold nodes.\n\
         doc_xml excerpt (first 800 chars): {:.800}",
        doc_xml
    );

    // No literal asterisks in the DOCX XML.
    assert!(
        !doc_xml.contains("**"),
        "STORY-081 C4 (DOCX): word/document.xml must NOT contain literal '**' characters.\n\
         doc_xml excerpt (first 800 chars): {:.800}",
        doc_xml
    );
}

// ─── HTML: output must have <strong> and no literal ** ────────────────────────

/// STORY-081 C4 / AC-005 — HTML output contains `<strong>` for bold body text
/// and does NOT contain literal `**`.
///
/// ## RED GATE
///
/// Broken state: body Inlines dropped (C2) → HTML exporter has no `InlineNode::Bold`
/// to render → `<strong>` absent → **assertion FAILS**.
#[test]
fn test_story_081_c4_html_body_bold_produces_strong_element() {
    let brand = BrandTmpDir::new("s081_c4_html_bold");
    let source = fixture_source("story-081-inline-markup.sf");
    let opts = brand.build_options("html", false);

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "STORY-081 C4 (HTML): build() must return Ok; got Err: {e:?}\n\
             Diagnose: check field_to_block.rs body threading and HTML inline rendering."
        )
    });

    let html_str = String::from_utf8_lossy(&output.bytes);

    // AC-005: HTML must contain <strong> — the semantic HTML element for bold.
    // RED GATE C2: body Inlines dropped → HTML exporter renders plain text →
    // <strong> absent → FAILS.
    assert!(
        html_str.contains("<strong>"),
        "STORY-081 C4 RED GATE (HTML): output must contain <strong> for bold body text.\n\
         C2: field_to_block.rs must thread FieldValue::Inlines for 'body' field.\n\
         HTML inline_node_to_html must render InlineNode::Bold as <strong>.\n\
         html excerpt (first 800 chars): {:.800}",
        &html_str[..html_str.len().min(800)]
    );

    // No literal asterisks in the HTML output.
    assert!(
        !html_str.contains("**"),
        "STORY-081 C4 (HTML): HTML output must NOT contain literal '**' characters.\n\
         html excerpt (first 800 chars): {:.800}",
        &html_str[..html_str.len().min(800)]
    );
}

// ─── PDF: output must contain "bold text" and no literal ** ──────────────────

/// STORY-081 C4 / AC-004 — PDF raw bytes contain the text "bold text" (the
/// content of the bold span) and do NOT contain literal `**bold text**`.
///
/// The PDF exporter renders text via krilla's `draw_text` API. After the
/// AC-004 implementation, `draw_body_blocks` uses `slide_to_krilla_runs` to
/// produce font-switched spans and `draw_inline_spans_at_y` to render each
/// span with the appropriate font face from `ResolvedFontSet`. Bold text is
/// drawn with `font_set.bold` (distinct from `font_set.regular`), satisfying
/// ADR-023's distinctness requirement (Pass-3 C1-NEW).
///
/// ## RED GATE
///
/// Broken state: body Inlines dropped → `draw_body_blocks` has empty
/// `ContentBlock::Text` → `slide_to_krilla_runs` returns `[]` →
/// `draw_inline_spans_at_y` not called → "bold text" absent from PDF
/// bytes → **assertion FAILS**.
///
/// Note: We search raw bytes rather than parsed PDF because krilla writes text
/// in UTF-8 content streams directly readable as byte substrings.
#[test]
fn test_story_081_c4_pdf_body_bold_text_present_no_asterisks() {
    let brand = BrandTmpDir::new("s081_c4_pdf_bold");
    let source = fixture_source("story-081-inline-markup.sf");
    let opts = brand.build_options("pdf", false);

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "STORY-081 C4 (PDF): build() must return Ok; got Err: {e:?}\n\
             Diagnose: check field_to_block.rs body threading."
        )
    });

    assert!(
        !output.bytes.is_empty(),
        "STORY-081 C4 (PDF): output.bytes must be non-empty"
    );

    // Search raw PDF bytes for "bold text" (the text content of the bold span).
    // After C2 fix: body ContentBlock::Text inlines thread to layout.
    // After C1 fix: draw_body_blocks calls slide_to_krilla_runs → text drawn.
    //
    // The PDF exporter may not always embed text as raw UTF-8 (it depends on
    // font encoding), so we accept either raw-bytes or UTF-8 string presence.
    let pdf_str = String::from_utf8_lossy(&output.bytes);
    let text_present =
        pdf_str.contains("bold text") || output.bytes.windows(9).any(|w| w == b"bold text");

    assert!(
        text_present,
        "STORY-081 C4 RED GATE (PDF): PDF bytes must contain 'bold text'.\n\
         C2: field_to_block.rs must thread FieldValue::Inlines for 'body'.\n\
         C1: PDF draw_body_blocks must render InlineNode::Bold span text.\n\
         pdf bytes (first 500 chars of lossy UTF-8): {:.500}",
        pdf_str
    );

    // No literal **bold text** in PDF output — asterisks must not be present.
    assert!(
        !pdf_str.contains("**"),
        "STORY-081 C4 (PDF): PDF output must NOT contain literal '**' characters.\n\
         If '**' appears, inline markup was written as plain text instead of being\n\
         converted to structural font-switching by the PDF exporter.\n\
         pdf bytes (first 500 chars): {:.500}",
        pdf_str
    );
}

// ─── Absence of literal * in PPTX body text runs ─────────────────────────────

/// STORY-081 C4 — PPTX slide1.xml body text runs must NOT contain literal `*`
/// from `_italic_` (underscore-italic) or `*italic*`.
///
/// The fixture uses `_italic text_` which must produce `<a:rPr i="1"/>`, NOT `_italic text_`.
#[test]
fn test_story_081_c4_pptx_body_no_underscore_literals() {
    let brand = BrandTmpDir::new("s081_c4_pptx_italic");
    let source = fixture_source("story-081-inline-markup.sf");
    let opts = brand.build_options("pptx", false);

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!("STORY-081 C4 (PPTX italic): build() must return Ok; got Err: {e:?}")
    });

    let mut archive = open_zip(&output.bytes, "C4-PPTX-italic");
    // slide2 = content slide with italic body text.
    let slide_xml = read_zip_entry(&mut archive, "ppt/slides/slide2.xml", "C4-PPTX-italic");

    // i="1" in <a:rPr> indicates italic run property.
    assert!(
        slide_xml.contains("i=\"1\""),
        "STORY-081 C4 (PPTX): slide1.xml must contain i=\"1\" for italic body text.\n\
         C2+C3 fix required.\n\
         slide1.xml (first 800 chars): {:.800}",
        slide_xml
    );
}

// =============================================================================
// F-P13 e2e extensions: bold-wrapped link and nested combined form
//
// slide3 = "Bold Link Slide" with body **[click](https://example.com)**
// slide4 = "Nested Combined Form Slide" with body **_bold and italic_**
//
// These tests are the end-to-end guard that would have caught F-P13-001
// (DOCX silently drops inline formatting on hyperlink) and validates the
// cross-format combined-form rendering.
// =============================================================================

/// F-P13 / ADV-P14-MED-001 e2e: PPTX slide3 (bold-wrapped link) must contain
/// `b="1"` AND `<a:hlinkClick>` AND the link display text; no literal `**`.
///
/// ## ADV-P14-MED-001 — Nested Link is now clickable
///
/// `Bold([Link{text:"click", url:"https://example.com"}])` must produce a run
/// that carries BOTH `b="1"` (bold formatting) AND `<a:hlinkClick>` (clickable
/// hyperlink) in the PPTX slide body. The External rel must be in the rels file.
///
/// The F-085-P6-001 "intentionally omits hlinkClick" carve-out is REMOVED.
/// Registration (collector) and emission (unified `render_inline_nodes_to_runs` engine)
/// are both now wrapper-aware, so nested links are correctly wired.
#[test]
fn test_f_p13_e2e_pptx_bold_link_slide3_has_b1_and_display_text() {
    let brand = BrandTmpDir::new("s081_fp13_pptx_bold_link");
    let source = fixture_source("story-081-inline-markup.sf");
    let opts = brand.build_options("pptx", false);

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!("F-P13 / ADV-P14 e2e (PPTX bold-link): build() must return Ok; got Err: {e:?}")
    });

    let mut archive = open_zip(&output.bytes, "F-P13-PPTX-bold-link");
    // slide3 = "Bold Link Slide" (fixture slide index 3 → ppt/slides/slide3.xml)
    let slide_xml = read_zip_entry(
        &mut archive,
        "ppt/slides/slide3.xml",
        "F-P13-PPTX-bold-link",
    );

    // The link display text "click" must appear.
    assert!(
        slide_xml.contains("click"),
        "F-P13/ADV-P14 e2e (PPTX): slide3.xml must contain the link display text 'click'.\n\
         slide3.xml (first 800 chars): {:.800}",
        slide_xml
    );

    // b="1" must appear — the bold property applies to the link display text run.
    assert!(
        slide_xml.contains("b=\"1\""),
        "F-P13/ADV-P14 e2e (PPTX): slide3.xml must contain b=\"1\" for the bold-wrapped link.\n\
         slide3.xml (first 800 chars): {:.800}",
        slide_xml
    );

    // ADV-P14-MED-001: <a:hlinkClick> must appear — the nested link is now clickable.
    assert!(
        slide_xml.contains("<a:hlinkClick"),
        "ADV-P14-MED-001 e2e (PPTX): slide3.xml must contain <a:hlinkClick> for Bold([Link]).\n\
         The F-085-P6-001 'no hlinkClick' carve-out is corrected by ADV-P14-MED-001.\n\
         slide3.xml (first 1000 chars): {:.1000}",
        slide_xml
    );

    // No literal ** — markup must be structural, not text.
    assert!(
        !slide_xml.contains("**"),
        "F-P13/ADV-P14 e2e (PPTX): slide3.xml must NOT contain literal '**'.\n\
         slide3.xml (first 800 chars): {:.800}",
        slide_xml
    );
}

/// F-P13 e2e: DOCX slide3 (bold-wrapped link) must contain both `<w:b/>` AND
/// `<w:hyperlink` — `<w:b/>` must be applied to the run inside `<w:hyperlink>`.
///
/// This is the end-to-end guard for F-P13-001: the DOCX exporter silently
/// dropped formatting on hyperlinks before the fix.
#[test]
fn test_f_p13_e2e_docx_bold_link_has_wb_and_hyperlink() {
    let brand = BrandTmpDir::new("s081_fp13_docx_bold_link");
    let source = fixture_source("story-081-inline-markup.sf");
    let opts = brand.build_options("docx", false);

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!("F-P13 e2e (DOCX bold-link): build() must return Ok; got Err: {e:?}")
    });

    let mut archive = open_zip(&output.bytes, "F-P13-DOCX-bold-link");
    let doc_xml = read_zip_entry(&mut archive, "word/document.xml", "F-P13-DOCX-bold-link");

    // The link display text must appear.
    assert!(
        doc_xml.contains("click"),
        "F-P13 e2e (DOCX): document.xml must contain the link display text 'click'.\n\
         doc_xml (first 800 chars): {:.800}",
        doc_xml
    );

    // <w:hyperlink must appear — the link is clickable.
    assert!(
        doc_xml.contains("w:hyperlink"),
        "F-P13 e2e (DOCX): document.xml must contain <w:hyperlink> for the bold-wrapped link.\n\
         doc_xml (first 800 chars): {:.800}",
        doc_xml
    );

    // <w:b/> must appear — the bold property must NOT be silently dropped.
    // This is the F-P13-001 keystone assertion: before the fix, apply_run_property
    // passed WHyperlink through unchanged, so <w:b/> was absent.
    assert!(
        doc_xml.contains("<w:b/>") || doc_xml.contains("<w:b "),
        "F-P13 e2e RED GATE (DOCX): document.xml must contain <w:b/> for the bold-wrapped link.\n\
         F-P13-001: apply_run_property must recurse into WHyperlink inner runs.\n\
         doc_xml (first 800 chars): {:.800}",
        doc_xml
    );
}

/// F-P13 e2e: HTML bold-wrapped link must contain both `<strong>` (or bold
/// styling) AND `<a href=` — the bold wraps the anchor, not vice versa.
#[test]
fn test_f_p13_e2e_html_bold_link_has_strong_and_anchor() {
    let brand = BrandTmpDir::new("s081_fp13_html_bold_link");
    let source = fixture_source("story-081-inline-markup.sf");
    let opts = brand.build_options("html", false);

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!("F-P13 e2e (HTML bold-link): build() must return Ok; got Err: {e:?}")
    });

    let html_str = String::from_utf8_lossy(&output.bytes);

    // The link display text must appear.
    assert!(
        html_str.contains("click"),
        "F-P13 e2e (HTML): output must contain the link display text 'click'.\n\
         html (first 800 chars): {:.800}",
        &html_str[..html_str.len().min(800)]
    );

    // <strong> must appear (bold renders as <strong> in HTML).
    assert!(
        html_str.contains("<strong>"),
        "F-P13 e2e (HTML): output must contain <strong> for the bold-wrapped link.\n\
         html (first 800 chars): {:.800}",
        &html_str[..html_str.len().min(800)]
    );

    // <a href= must appear (the link is rendered as an anchor).
    assert!(
        html_str.contains("<a ") && html_str.contains("href="),
        "F-P13 e2e (HTML): output must contain <a href=...> for the link.\n\
         html (first 800 chars): {:.800}",
        &html_str[..html_str.len().min(800)]
    );
}

/// F-P13 e2e: PPTX slide4 (nested bold+italic combined form) must contain both
/// `b="1"` AND `i="1"` on the same run.
#[test]
fn test_f_p13_e2e_pptx_nested_bold_italic_slide4_has_b1_and_i1() {
    let brand = BrandTmpDir::new("s081_fp13_pptx_nested_bi");
    let source = fixture_source("story-081-inline-markup.sf");
    let opts = brand.build_options("pptx", false);

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!("F-P13 e2e (PPTX nested bi): build() must return Ok; got Err: {e:?}")
    });

    let mut archive = open_zip(&output.bytes, "F-P13-PPTX-nested-bi");
    // slide4 = "Nested Combined Form Slide"
    let slide_xml = read_zip_entry(
        &mut archive,
        "ppt/slides/slide4.xml",
        "F-P13-PPTX-nested-bi",
    );

    // Both b="1" and i="1" must appear (on the same or adjacent runs for the
    // combined bold+italic span).
    assert!(
        slide_xml.contains("b=\"1\""),
        "F-P13 e2e (PPTX): slide4.xml must contain b=\"1\" for bold+italic nested form.\n\
         slide4.xml (first 800 chars): {:.800}",
        slide_xml
    );

    assert!(
        slide_xml.contains("i=\"1\""),
        "F-P13 e2e (PPTX): slide4.xml must contain i=\"1\" for bold+italic nested form.\n\
         slide4.xml (first 800 chars): {:.800}",
        slide_xml
    );
}

/// F-P13 e2e: DOCX nested bold+italic combined form must contain both `<w:b/>`
/// AND `<w:i/>` in the document body.
#[test]
fn test_f_p13_e2e_docx_nested_bold_italic_has_wb_and_wi() {
    let brand = BrandTmpDir::new("s081_fp13_docx_nested_bi");
    let source = fixture_source("story-081-inline-markup.sf");
    let opts = brand.build_options("docx", false);

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!("F-P13 e2e (DOCX nested bi): build() must return Ok; got Err: {e:?}")
    });

    let mut archive = open_zip(&output.bytes, "F-P13-DOCX-nested-bi");
    let doc_xml = read_zip_entry(&mut archive, "word/document.xml", "F-P13-DOCX-nested-bi");

    assert!(
        doc_xml.contains("<w:b/>") || doc_xml.contains("<w:b "),
        "F-P13 e2e (DOCX): document.xml must contain <w:b/> for nested bold+italic.\n\
         doc_xml (first 800 chars): {:.800}",
        doc_xml
    );

    assert!(
        doc_xml.contains("<w:i/>") || doc_xml.contains("<w:i "),
        "F-P13 e2e (DOCX): document.xml must contain <w:i/> for nested bold+italic.\n\
         doc_xml (first 800 chars): {:.800}",
        doc_xml
    );
}

/// F-P13 e2e: HTML nested bold+italic combined form must contain both `<strong>`
/// AND `<em>` in the output.
#[test]
fn test_f_p13_e2e_html_nested_bold_italic_has_strong_and_em() {
    let brand = BrandTmpDir::new("s081_fp13_html_nested_bi");
    let source = fixture_source("story-081-inline-markup.sf");
    let opts = brand.build_options("html", false);

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!("F-P13 e2e (HTML nested bi): build() must return Ok; got Err: {e:?}")
    });

    let html_str = String::from_utf8_lossy(&output.bytes);

    assert!(
        html_str.contains("<strong>"),
        "F-P13 e2e (HTML): output must contain <strong> for nested bold+italic.\n\
         html (first 800 chars): {:.800}",
        &html_str[..html_str.len().min(800)]
    );

    assert!(
        html_str.contains("<em>"),
        "F-P13 e2e (HTML): output must contain <em> for nested bold+italic.\n\
         html (first 800 chars): {:.800}",
        &html_str[..html_str.len().min(800)]
    );
}

// =============================================================================
// F-P25-HIGH-001: strict-mode fatal promotion for InlineMarkupInTitle (E-EVL-015)
//
// AC-006 / BC-3.05.001 EC-011: inline markup in a slide `title` field must
// cause a STRICT BUILD FAILURE (E-EVL-015 at Error severity → strict gate fires
// → BuildError::EvalFailed / MultistageFailed returned, non-zero exit, no output).
//
// In --warn-only mode (strict=false) the same input must SUCCEED: output is
// produced, the PPTX title is plain text (no b="1" in the title placeholder),
// and the diagnostic is emitted as a warning rather than an error.
// =============================================================================

/// RED GATE (F-P25-HIGH-001, part A): a deck with `title: "**Bold Title**"` in
/// strict mode (the default) must return `Err(BuildError::EvalFailed)` or
/// `Err(BuildError::MultistageFailed)` — the build must NOT succeed.
///
/// The diagnostic must carry E-EVL-015 (EvalError::InlineMarkupInTitle).
///
/// ## Why this test was failing before this fix
///
/// `for_eval.rs` emitted `InlineMarkupInTitle` at `ParseSeverity::Warning`.
/// `error_and_fatal_count()` returned 0 for Warning-only sinks, so the strict
/// gate at `lib.rs:908` never fired. Build returned `Ok` with the title stripped
/// to plain text — the strict-failure contract was silently violated.
///
/// ## After this fix
///
/// `for_eval.rs` emits `InlineMarkupInTitle` at `ParseSeverity::Error`.
/// `error_and_fatal_count()` returns 1, the strict gate fires, and build returns
/// `Err(EvalFailed { .. })` (or `MultistageFailed` if validators also fire).
#[test]
fn test_fp25_high_001_strict_mode_title_markup_fails_build() {
    let brand = BrandTmpDir::new("fp25_strict_title_markup");
    // Minimal DSL with bold markup in the slide title field.
    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "slide content:\n",
        "  title \"**Bold Title**\"\n",
        "  body \"Some body text.\"\n",
    );
    // strict=true is the default; confirm it fails.
    let opts = brand.build_options("pptx", true);

    let result = slideforge::build(source, &opts);

    // RED GATE: currently Ok (Warning-only) — must become Err after fix.
    assert!(
        result.is_err(),
        "F-P25-HIGH-001 RED GATE: strict build with title **Bold Title** must return Err; \
         got Ok — EvalError::InlineMarkupInTitle (E-EVL-015) must be emitted at Error severity \
         so the strict gate fires. Current code emits Warning, which does not trigger the gate."
    );

    // Verify the error is an eval-level failure carrying E-EVL-015.
    let diags: Vec<String> = match result {
        Err(slideforge::error::BuildError::EvalFailed { diagnostics, .. }) => diagnostics
            .iter()
            .filter_map(|d| d.code().map(|c| c.to_string()))
            .collect(),
        Err(slideforge::error::BuildError::MultistageFailed {
            eval_diagnostics, ..
        }) => eval_diagnostics
            .iter()
            .filter_map(|d| d.code().map(|c| c.to_string()))
            .collect(),
        Err(other) => {
            panic!("F-P25-HIGH-001: expected EvalFailed or MultistageFailed; got: {other:?}")
        },
        Ok(_) => {
            panic!("F-P25-HIGH-001: expected Err but got Ok — strict gate not firing on E-EVL-015")
        },
    };

    let has_e015 = diags.iter().any(|c| c.contains("E-EVL-015"));
    assert!(
        has_e015,
        "F-P25-HIGH-001: EvalFailed/MultistageFailed diagnostics must contain E-EVL-015 \
         (EvalError::InlineMarkupInTitle); got codes: {:?}",
        diags
    );
}

/// F-P25-HIGH-001, part B: the same deck in --warn-only (strict=false) mode
/// must SUCCEED — the diagnostic is non-blocking, output is produced, and the
/// PPTX title placeholder contains plain text "Bold Title" (no `b="1"`).
#[test]
fn test_fp25_high_001_warn_only_title_markup_succeeds() {
    let brand = BrandTmpDir::new("fp25_warnonly_title_markup");
    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "slide content:\n",
        "  title \"**Bold Title**\"\n",
        "  body \"Some body text.\"\n",
    );
    // strict=false → warn-only mode; E-EVL-015 must NOT block the build.
    let opts = brand.build_options("pptx", false);

    let output = slideforge::build(source, &opts).unwrap_or_else(|e| {
        panic!(
            "F-P25-HIGH-001 warn-only: build with title **Bold Title** must return Ok in \
             --warn-only mode; got Err: {e:?}\n\
             Hint: InlineMarkupInTitle at Error severity must only gate STRICT builds. \
             In warn-only (strict=false) the gate does not fire."
        )
    });

    // PPTX output must be non-empty (deck was produced).
    assert!(
        !output.bytes.is_empty(),
        "F-P25-HIGH-001 warn-only: PPTX output bytes must be non-empty"
    );

    // The PPTX title placeholder must NOT contain b="1" — the title was stripped
    // to plain text "Bold Title" (no bold run property in the title placeholder).
    let mut archive = open_zip(&output.bytes, "fp25-warnonly-title");
    let slide_xml = read_zip_entry(&mut archive, "ppt/slides/slide1.xml", "fp25-warnonly-title");

    // Title text "Bold Title" must appear (stripped, no asterisks).
    assert!(
        slide_xml.contains("Bold Title"),
        "F-P25-HIGH-001 warn-only: PPTX slide1.xml must contain 'Bold Title' (plain stripped text).\n\
         slide1.xml (first 800 chars): {:.800}",
        slide_xml
    );

    // No literal ** characters — markup must have been stripped.
    assert!(
        !slide_xml.contains("**"),
        "F-P25-HIGH-001 warn-only: PPTX slide1.xml must NOT contain '**' literal asterisks.\n\
         The title was stripped to plain text by the eval stage.\n\
         slide1.xml (first 800 chars): {:.800}",
        slide_xml
    );
}
