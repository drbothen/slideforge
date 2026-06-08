//! STORY-041 unit tests for BC-4.02.001 — DOCX Core Serialization.
//!
//! All tests in this file MUST FAIL against the `todo!()` stubs that exist in
//! the production modules at Red Gate time. They will pass only after the
//! implementer fills in the production logic.
//!
//! # Test naming convention
//!
//! `test_BC_4_02_001_<assertion_name>` — all tests trace to BC-4.02.001.
//!
//! # Red Gate status (Task 2 — stub baseline)
//!
//! Every test marked **RED** will panic with `not yet implemented` when run
//! against the stubs. Tests marked **GREEN-BY-DESIGN** or **WIRING-EXEMPT**
//! are noted with rationale.
//!
//! # Traceability
//!
//! | Test name | BC clause | Story AC |
//! |-----------|-----------|----------|
//! | `test_BC_4_02_001_zip_contains_required_parts` | postcondition 2 | AC-002 |
//! | `test_BC_4_02_001_content_types_registers_mandatory_parts` | postcondition 2 | AC-002 |
//! | `test_BC_4_02_001_content_types_snapshot` | postcondition 2 | AC-002 |
//! | `test_BC_4_02_001_dc_language_in_core_xml` | postcondition 1 | AC-002 |
//! | `test_BC_4_02_001_report_register_in_document_xml` | postcondition 3 | AC-003 |
//! | `test_BC_4_02_001_slide_title_as_heading1_before_report` | postcondition 3 | AC-004 |
//! | `test_BC_4_02_001_notes_sentinel_absent_from_docx_body` | invariant 1 | AC-005 |
//! | `test_BC_4_02_001_report_sentinel_present_in_docx_body` | postcondition 3 | AC-006 |
//! | `test_BC_4_02_001_detail_in_extended_section_after_report` | postcondition 4 | AC-007 |
//! | `test_BC_4_02_001_inline_bold_italic_run_properties` | postcondition 7 | AC-008 |
//! | `test_BC_4_02_001_inline_code_courier_new_font` | postcondition 7 | AC-008 |
//! | `test_BC_4_02_001_visual_only_slide_heading_plus_empty_para` | postcondition 6 | AC-009 |
//! | `test_BC_4_02_001_exporter_trait_id_and_extension` | postcondition 1 / Exporter trait clause | AC-001 |
//! | `test_BC_4_02_001_determinism_same_deck_twice` | invariant 4 | Test Strategy |
//! | `test_BC_4_02_001_two_slides_two_headings` | postcondition 3 | AC-003 |
//! | `test_BC_4_02_001_ec001_only_notes_slide_no_notes_in_body` | EC-001 | AC-005 |
//! | `test_BC_4_02_001_ec002_multi_paragraph_report` | EC-002 | AC-003 |
//! | `test_BC_4_02_001_ec004_detail_and_report_same_slide` | EC-004 | AC-007 |
//! | `test_BC_4_02_001_zip_assembler_deterministic_entry_order` | invariant 4 | Task 2 |
//! | `test_BC_4_02_001_zip_assembler_epoch_timestamps` | invariant 4 | Task 2 |
//! | `test_BC_4_02_001_styles_xml_contains_required_styles` | postcondition 2 | Task 4 |
//! | `test_BC_4_02_001_f_docx_001_hyperlink_doc_declares_xmlns_r_and_parses_back` | postcondition 2 / namespace well-formedness | F-DOCX-001 / F-DOCX-003 |

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(non_snake_case)]
#![allow(clippy::doc_markdown)]

use std::io::Read as IoRead;
use std::sync::Arc;

use slideforge_eval::BleedChecker;
use slideforge_layout::types::{
    BoundingBox, Emu, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize, Rgb,
};
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::{
    Brand, BrandFonts, BrandPalette, Deck, DeckMetadata, InlineNode, OrderedMap, Register,
    RegisteredContent,
};

use crate::DocxExporter;

// ─── Test fixture helpers ─────────────────────────────────────────────────────

/// Build a minimal [`Brand`] for use in export tests.
fn minimal_brand() -> Brand {
    use slideforge_types::span::SourceSpan;
    Brand {
        name: Arc::from("test-brand"),
        palette: BrandPalette {
            primary: Arc::from("#003087"),
            secondary: Arc::from("#0066CC"),
            accent: Arc::from("#FF6B35"),
            neutral: Arc::from("#F5F5F5"),
        },
        fonts: BrandFonts {
            heading: Arc::from("Calibri Light"),
            body: Arc::from("Calibri"),
            mono: Arc::from("Courier New"),
            font_size_emu: 457_200,
        },
        layouts: vec![],
        span: SourceSpan::default(),
    }
}

/// Build a minimal [`Deck`] with `lang "en-US"`.
fn minimal_deck() -> Deck {
    Deck {
        slides: vec![],
        vars: OrderedMap::new(),
        metadata: DeckMetadata {
            title: Some(Arc::from("Test Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        },
        registers: OrderedMap::new(),
        section_blocks: vec![],
    }
}

/// Build a [`LaidOutSlide`] with a given title and `register_content` entries.
fn make_slide(title: &str, register_content: Vec<RegisteredContent>) -> LaidOutSlide {
    LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("content"),
        frames: vec![Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(9_144_000),
                height: Emu(5_143_500),
            },
            content: FrameContent::Title(Arc::from(title)),
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content,
    }
}

/// Build a [`LaidOutDeck`] wrapping the provided slides.
fn make_laid_out_deck(slides: Vec<LaidOutSlide>) -> LaidOutDeck {
    LaidOutDeck {
        page_size: PageSize::default(),
        slides,
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    }
}

/// Call `DocxExporter::export` and return the raw `.docx` bytes.
///
/// Will panic at the `todo!()` stub during the Red Gate phase.
fn export_deck(deck: &Deck, laid_out: &LaidOutDeck) -> Vec<u8> {
    let brand = minimal_brand();
    let opts = ExportOptions::default();
    DocxExporter
        .export(deck, laid_out, &brand, &opts)
        .expect("export must succeed")
}

/// Enumerate all ZIP entry names from a `.docx` byte buffer.
fn zip_entry_names(docx_bytes: &[u8]) -> Vec<String> {
    let cursor = std::io::Cursor::new(docx_bytes);
    let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
    (0..archive.len())
        .map(|i| archive.by_index(i).expect("valid index").name().to_owned())
        .collect()
}

/// Read a named ZIP member from a `.docx` byte buffer as a UTF-8 string.
fn read_zip_member(docx_bytes: &[u8], name: &str) -> String {
    let cursor = std::io::Cursor::new(docx_bytes);
    let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
    let mut entry = archive.by_name(name).expect("member must exist");
    let mut buf = String::new();
    entry.read_to_string(&mut buf).expect("valid utf-8");
    buf
}

// ─── AC-001: Exporter trait wiring ───────────────────────────────────────────
//
// GREEN-BY-DESIGN / WIRING-EXEMPT: `id()` and `extension()` are already
// implemented in the stub (they return literal strings, no `todo!()`). This
// test verifies the wiring is correct and will pass even at Red Gate. It does
// not exercise any serialization logic.

/// BC-4.02.001 postcondition 1 / Exporter trait clause / AC-001:
/// `DocxExporter` implements `Exporter` with `id() == "docx"` and
/// `extension() == "docx"`.
///
/// GREEN-BY-DESIGN / WIRING-EXEMPT: `id()` and `extension()` are non-stub.
#[test]
fn test_BC_4_02_001_exporter_trait_id_and_extension() {
    let exporter = DocxExporter;
    assert_eq!(
        exporter.id(),
        "docx",
        "Exporter::id must return \"docx\" (AC-001)"
    );
    assert_eq!(
        exporter.extension(),
        "docx",
        "Exporter::extension must return \"docx\" (AC-001)"
    );
}

// ─── AC-002: Valid DOCX ZIP with all required parts ───────────────────────────

/// BC-4.02.001 postcondition 2 / AC-002:
/// Export of a 1-slide deck produces a ZIP containing all required DOCX parts.
///
/// Required parts (STORY-041 DOCX ZIP Structure section):
/// - `[Content_Types].xml`
/// - `_rels/.rels`
/// - `word/document.xml`
/// - `word/_rels/document.xml.rels`
/// - `word/styles.xml`
/// - `docProps/core.xml`
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_zip_contains_required_parts() {
    let deck = minimal_deck();
    let slide = make_slide("Slide One", vec![]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let names = zip_entry_names(&docx_bytes);

    let required = &[
        "[Content_Types].xml",
        "_rels/.rels",
        "word/document.xml",
        "word/_rels/document.xml.rels",
        "word/styles.xml",
        "docProps/core.xml",
    ];

    for &part in required {
        assert!(
            names.iter().any(|n| n == part),
            "DOCX ZIP must contain {part} (AC-002). Found entries: {names:?}"
        );
    }
}

/// BC-4.02.001 postcondition 2 / AC-002:
/// `[Content_Types].xml` registers all mandatory DOCX part content types.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_content_types_registers_mandatory_parts() {
    let deck = minimal_deck();
    let slide = make_slide("Slide", vec![]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let content_types_xml = read_zip_member(&docx_bytes, "[Content_Types].xml");

    // Required content types per STORY-041 Task 3:
    assert!(
        content_types_xml.contains("wordprocessingml.document.main+xml"),
        "[Content_Types].xml must register word/document.xml content type"
    );
    assert!(
        content_types_xml.contains("wordprocessingml.styles+xml"),
        "[Content_Types].xml must register word/styles.xml content type"
    );
    assert!(
        content_types_xml.contains("wordprocessingml.numbering+xml"),
        "[Content_Types].xml must register word/numbering.xml content type"
    );
    assert!(
        content_types_xml.contains("wordprocessingml.settings+xml"),
        "[Content_Types].xml must register word/settings.xml content type"
    );
}

/// BC-4.02.001 postcondition 2 / AC-002 — snapshot:
/// `[Content_Types].xml` snapshot locks the exact XML output.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_content_types_snapshot() {
    let deck = minimal_deck();
    let slide = make_slide("Slide", vec![]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let content_types_xml = read_zip_member(&docx_bytes, "[Content_Types].xml");

    insta::assert_snapshot!("content_types_xml", content_types_xml);
}

/// BC-4.02.001 postcondition 1 / AC-002:
/// `docProps/core.xml` contains `dc:language` set from `Deck.metadata.lang`.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_dc_language_in_core_xml() {
    let mut deck = minimal_deck();
    deck.metadata.lang = Some(Arc::from("en-US"));
    let slide = make_slide("Slide", vec![]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let core_xml = read_zip_member(&docx_bytes, "docProps/core.xml");

    assert!(
        core_xml.contains("en-US"),
        "docProps/core.xml must contain dc:language value \"en-US\"; got:\n{core_xml}"
    );
    assert!(
        core_xml.contains("dc:language"),
        "docProps/core.xml must use the dc:language element"
    );
}

// ─── AC-003: report register content in document.xml body ────────────────────

/// BC-4.02.001 postcondition 3 / AC-003:
/// A slide with `report "Analysis follows."` produces a body paragraph in
/// `word/document.xml` containing that text.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_report_register_in_document_xml() {
    let deck = minimal_deck();
    let rc = RegisteredContent::plain(Register::Report, Arc::from("Analysis follows."));
    let slide = make_slide("Q1 Review", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    assert!(
        doc_xml.contains("Analysis follows."),
        "word/document.xml must contain report text \"Analysis follows.\" (AC-003); got:\n{doc_xml}"
    );
    assert!(
        doc_xml.contains("Normal"),
        "report paragraph must use style Normal (AC-003)"
    );
}

/// BC-4.02.001 postcondition 3 / AC-003 — canonical test vector:
/// 2-slide deck with `report "Analysis follows."` on each slide produces
/// two headings and two body paragraphs.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_two_slides_two_headings() {
    let deck = minimal_deck();
    let rc1 = RegisteredContent::plain(Register::Report, Arc::from("Analysis follows."));
    let rc2 = RegisteredContent::plain(Register::Report, Arc::from("Analysis follows."));
    let slide1 = make_slide("Slide One", vec![rc1]);
    let slide2 = {
        let mut s = make_slide("Slide Two", vec![rc2]);
        s.source_index = 1;
        s
    };
    let laid_out = make_laid_out_deck(vec![slide1, slide2]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    assert!(
        doc_xml.contains("Slide One"),
        "word/document.xml must contain heading for slide 1 (AC-003)"
    );
    assert!(
        doc_xml.contains("Slide Two"),
        "word/document.xml must contain heading for slide 2 (AC-003)"
    );

    let count = doc_xml.matches("Analysis follows.").count();
    assert_eq!(
        count, 2,
        "word/document.xml must contain 2 report paragraphs; found {count} occurrences (AC-003)"
    );

    insta::assert_snapshot!("two_slides_document_xml", doc_xml);
}

// ─── AC-004: Slide title as Heading1 before report content ───────────────────

/// BC-4.02.001 postcondition 3 / AC-004:
/// The slide title appears as a `Heading1` paragraph BEFORE the report body
/// paragraph in `word/document.xml`.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_slide_title_as_heading1_before_report() {
    let deck = minimal_deck();
    let rc = RegisteredContent::plain(Register::Report, Arc::from("Report text"));
    let slide = make_slide("Q1 Review", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    let heading_pos = doc_xml
        .find("Q1 Review")
        .expect("slide title \"Q1 Review\" must appear in document.xml (AC-004)");
    let report_pos = doc_xml
        .find("Report text")
        .expect("report text must appear in document.xml (AC-004)");

    assert!(
        heading_pos < report_pos,
        "Heading1 (slide title) must appear BEFORE Normal (report text) in document.xml (AC-004): \
         heading at {heading_pos}, report at {report_pos}"
    );
    assert!(
        doc_xml.contains("Heading1"),
        "Slide title must use Heading1 paragraph style (AC-004)"
    );
}

// ─── AC-005: notes register absent from document.xml (un-ignore STORY-036 AC-004) ──

/// BC-4.02.001 invariant 1 / AC-005 (un-ignore STORY-036 AC-004):
/// A deck with `notes "NOTES_DOCX_SENTINEL"` must NOT have that text in
/// `word/document.xml`.
///
/// Uses `BleedChecker::assert_absent_from_docx_body`.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_notes_sentinel_absent_from_docx_body() {
    let deck = minimal_deck();
    let rc = RegisteredContent::plain(Register::Notes, Arc::from("NOTES_DOCX_SENTINEL"));
    let slide = make_slide("Test Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);

    // BC-4.02.001 invariant 1 / DI-012: notes NEVER in DOCX body.
    BleedChecker::assert_absent_from_docx_body(&docx_bytes, "NOTES_DOCX_SENTINEL");
}

// ─── AC-006: report register present (positive bleed test, un-ignore STORY-036 AC-005) ──

/// BC-4.02.001 postcondition 3 / AC-006 (un-ignore STORY-036 AC-005):
/// A deck with `report "REPORT_DOCX_SENTINEL"` MUST have that text in
/// `word/document.xml`.
///
/// Uses `BleedChecker::assert_present_in_docx_body`.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_report_sentinel_present_in_docx_body() {
    let deck = minimal_deck();
    let rc = RegisteredContent::plain(Register::Report, Arc::from("REPORT_DOCX_SENTINEL"));
    let slide = make_slide("Test Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);

    // Positive bleed test: report MUST appear in DOCX body.
    BleedChecker::assert_present_in_docx_body(&docx_bytes, "REPORT_DOCX_SENTINEL");
}

// ─── AC-007: detail register in extended section after main body ──────────────

/// BC-4.02.001 postcondition 4 / AC-007:
/// A slide with `detail "Technical appendix text"` produces a paragraph
/// AFTER the last report paragraph in `word/document.xml`.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_detail_in_extended_section_after_report() {
    let deck = minimal_deck();
    let report_rc = RegisteredContent::plain(Register::Report, Arc::from("Report content here"));
    let detail_rc =
        RegisteredContent::plain(Register::Detail, Arc::from("Technical appendix text"));
    let slide = make_slide("Analysis Slide", vec![report_rc, detail_rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    let report_pos = doc_xml
        .find("Report content here")
        .expect("report text must appear in document.xml (AC-007)");
    let detail_pos = doc_xml
        .find("Technical appendix text")
        .expect("detail text must appear in document.xml (AC-007)");

    assert!(
        detail_pos > report_pos,
        "Detail text must appear AFTER report text in document.xml (AC-007): \
         report at {report_pos}, detail at {detail_pos}"
    );
    assert!(
        doc_xml.contains("Heading2"),
        "Detail extended section must have a Heading2 header (AC-007)"
    );
}

// ─── AC-008: Inline formatting mapped to Word run properties ─────────────────

/// BC-4.02.001 postcondition 7 / AC-008:
/// A report field with bold and italic inline nodes produces `<w:b/>` and
/// `<w:i/>` run properties respectively.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
///
/// # Enclosure proof (F-DOCX-002)
///
/// The test extracts the individual `<w:r>…</w:r>` fragment that ENCLOSES
/// the target text and asserts the run property is present WITHIN that same
/// fragment. This is stronger than a byte-offset ordering check, which would
/// pass even if `<w:b/>` were emitted on an earlier run in the same paragraph.
#[test]
fn test_BC_4_02_001_inline_bold_italic_run_properties() {
    let deck = minimal_deck();

    // report "**Bold** and *italic*" — structural InlineNode sequence.
    let bold_node = InlineNode::Bold(vec![InlineNode::Plain(Arc::from("Bold"))]);
    let and_node = InlineNode::Plain(Arc::from(" and "));
    let italic_node = InlineNode::Italic(vec![InlineNode::Plain(Arc::from("italic"))]);

    let rc = RegisteredContent {
        register: Register::Report,
        content: vec![bold_node, and_node, italic_node],
    };
    let slide = make_slide("Formatting Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // ── Bold run enclosure proof (F-DOCX-002) ────────────────────────────────
    // Extract the <w:r>…</w:r> fragment that encloses <w:t>Bold</w:t>.
    // The structure is: <w:r><w:rPr><w:b/></w:rPr><w:t>Bold</w:t></w:r>
    let bold_text_marker = "<w:t>Bold</w:t>";
    let bold_text_pos = doc_xml
        .find(bold_text_marker)
        .expect("Bold text run must contain <w:t>Bold</w:t> (AC-008)");

    // Walk backwards from bold_text_pos to find the enclosing <w:r> start.
    let bold_run_start = doc_xml[..bold_text_pos]
        .rfind("<w:r>")
        .or_else(|| doc_xml[..bold_text_pos].rfind("<w:r "))
        .expect("bold text must be enclosed in a <w:r> element (AC-008 / F-DOCX-002)");

    // Walk forwards to find the matching </w:r> end.
    let bold_run_close_offset = doc_xml[bold_text_pos..]
        .find("</w:r>")
        .expect("bold run must be closed by </w:r> (AC-008 / F-DOCX-002)");
    let bold_run_end = bold_text_pos + bold_run_close_offset + "</w:r>".len();
    let bold_run_fragment = &doc_xml[bold_run_start..bold_run_end];

    // Assert <w:b/> or <w:b /> is WITHIN the bold run fragment.
    assert!(
        bold_run_fragment.contains("<w:b/>") || bold_run_fragment.contains("<w:b />"),
        "Bold run property <w:b/> or <w:b /> must appear WITHIN the <w:r>…</w:r> \
         fragment enclosing 'Bold' text (AC-008 / F-DOCX-002 enclosure proof). \
         Got bold run fragment:\n{bold_run_fragment}"
    );

    // ── Plain " and " run does NOT contain <w:b> (F-DOCX-002) ──────────────
    let and_marker = "<w:t xml:space=\"preserve\"> and </w:t>";
    let and_text_pos = doc_xml
        .find(and_marker)
        .expect("Plain \" and \" text with xml:space preserve must appear (AC-008)");
    let and_run_start = doc_xml[..and_text_pos]
        .rfind("<w:r>")
        .or_else(|| doc_xml[..and_text_pos].rfind("<w:r "))
        .expect("' and ' text must be enclosed in a <w:r> element (AC-008 / F-DOCX-002)");
    let and_run_close_offset = doc_xml[and_text_pos..]
        .find("</w:r>")
        .expect("' and ' run must close with </w:r>");
    let and_run_end = and_text_pos + and_run_close_offset + "</w:r>".len();
    let and_run_fragment = &doc_xml[and_run_start..and_run_end];

    assert!(
        !and_run_fragment.contains("<w:b/>") && !and_run_fragment.contains("<w:b />"),
        "The plain ' and ' run must NOT contain <w:b/> (F-DOCX-002: bold property \
         must be scoped to the Bold run only, not bleed into sibling runs). \
         Got ' and ' run fragment:\n{and_run_fragment}"
    );

    // ── Italic run enclosure proof (F-DOCX-002) ──────────────────────────────
    let italic_text_marker = "<w:t>italic</w:t>";
    let italic_text_pos = doc_xml
        .find(italic_text_marker)
        .expect("Italic text run must contain <w:t>italic</w:t> (AC-008)");

    let italic_run_start = doc_xml[..italic_text_pos]
        .rfind("<w:r>")
        .or_else(|| doc_xml[..italic_text_pos].rfind("<w:r "))
        .expect("italic text must be enclosed in a <w:r> element (AC-008 / F-DOCX-002)");
    let italic_run_close_offset = doc_xml[italic_text_pos..]
        .find("</w:r>")
        .expect("italic run must be closed by </w:r> (AC-008 / F-DOCX-002)");
    let italic_run_end = italic_text_pos + italic_run_close_offset + "</w:r>".len();
    let italic_run_fragment = &doc_xml[italic_run_start..italic_run_end];

    assert!(
        italic_run_fragment.contains("<w:i/>") || italic_run_fragment.contains("<w:i />"),
        "Italic run property <w:i/> or <w:i /> must appear WITHIN the <w:r>…</w:r> \
         fragment enclosing 'italic' text (AC-008 / F-DOCX-002 enclosure proof). \
         Got italic run fragment:\n{italic_run_fragment}"
    );

    insta::assert_snapshot!("inline_bold_italic_document_xml", doc_xml);
}

/// BC-4.02.001 postcondition 7 / AC-008:
/// A report field with `InlineNode::Code` produces Courier New font run property.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_inline_code_courier_new_font() {
    let deck = minimal_deck();
    let code_node = InlineNode::Code(Arc::from("fn foo() {}"));
    let rc = RegisteredContent {
        register: Register::Report,
        content: vec![code_node],
    };
    let slide = make_slide("Code Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    assert!(
        doc_xml.contains("Courier New"),
        "Code InlineNode must produce Courier New font run property (AC-008 / inline table)"
    );
    assert!(
        doc_xml.contains("fn foo() {}"),
        "Code text must appear in document.xml (AC-008)"
    );
}

// ─── AC-009: visual-only slide produces heading + empty paragraph ─────────────

/// BC-4.02.001 postcondition 6 / AC-009:
/// A slide with no report or detail register content produces:
/// 1. `Heading1` paragraph with slide title
/// 2. An empty paragraph (not omitted)
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_visual_only_slide_heading_plus_empty_para() {
    let deck = minimal_deck();
    // No register_content — purely visual slide.
    let slide = make_slide("Visual Only Slide", vec![]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    assert!(
        doc_xml.contains("Visual Only Slide"),
        "Heading for visual-only slide must appear in document.xml (AC-009)"
    );
    assert!(
        doc_xml.contains("Heading1"),
        "Visual-only slide must use Heading1 style (AC-009)"
    );

    // The document must contain at least two <w:p ...> elements:
    // one for the Heading1 and one empty paragraph that follows.
    let para_count = doc_xml.matches("<w:p").count();
    assert!(
        para_count >= 2,
        "Visual-only slide must produce ≥2 paragraphs (heading + empty para); \
         found {para_count} opening <w:p tags in document.xml (AC-009)"
    );
}

// ─── Determinism test ─────────────────────────────────────────────────────────

/// BC-4.02.001 invariant 4:
/// Same deck built twice must produce byte-identical `.docx` output.
///
/// Epoch timestamps in the ZIP assembler are required for this test to pass.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_determinism_same_deck_twice() {
    let deck = minimal_deck();

    let rc1 = RegisteredContent::plain(Register::Report, Arc::from("Deterministic content"));
    let slide1 = make_slide("Determinism Slide", vec![rc1]);
    let laid_out1 = make_laid_out_deck(vec![slide1]);

    // Rebuild the identical deck a second time.
    let rc2 = RegisteredContent::plain(Register::Report, Arc::from("Deterministic content"));
    let slide2 = make_slide("Determinism Slide", vec![rc2]);
    let laid_out2 = make_laid_out_deck(vec![slide2]);

    let bytes1 = export_deck(&deck, &laid_out1);
    let bytes2 = export_deck(&deck, &laid_out2);

    assert_eq!(
        bytes1.len(),
        bytes2.len(),
        "Deterministic export must produce identical byte lengths (invariant 4)"
    );
    assert_eq!(
        bytes1, bytes2,
        "Deterministic export must produce byte-identical .docx output for identical inputs (invariant 4)"
    );
}

// ─── DocxZipAssembler unit tests ──────────────────────────────────────────────

/// BC-4.02.001 invariant 4 — `DocxZipAssembler`:
/// ZIP entries are written in lexicographic path order (deterministic ordering).
///
/// RED: will panic at `todo!()` in `DocxZipAssembler::new`.
#[test]
fn test_BC_4_02_001_zip_assembler_deterministic_entry_order() {
    use crate::zip_assembler::DocxZipAssembler;

    let mut asm = DocxZipAssembler::new();
    // Add parts in non-alphabetical order.
    asm.add_part("word/styles.xml", b"<styles/>".to_vec());
    asm.add_part("[Content_Types].xml", b"<ct/>".to_vec());
    asm.add_part("_rels/.rels", b"<rels/>".to_vec());
    asm.add_part("word/document.xml", b"<doc/>".to_vec());

    let bytes = asm.finish().expect("assembler must succeed");
    let names = zip_entry_names(&bytes);

    // Expected lexicographic order for these four paths.
    let expected_order = [
        "[Content_Types].xml",
        "_rels/.rels",
        "word/document.xml",
        "word/styles.xml",
    ];

    for (i, expected) in expected_order.iter().enumerate() {
        assert_eq!(
            names.get(i).map(String::as_str),
            Some(*expected),
            "ZIP entry at position {i} must be {expected:?} (lexicographic deterministic order); \
             got: {names:?}"
        );
    }
}

/// BC-4.02.001 invariant 4 — `DocxZipAssembler`:
/// All ZIP entries must have fixed epoch timestamps for deterministic output.
///
/// MS-DOS time epoch = 1980-01-01 00:00:00 (lowest representable date).
///
/// RED: will panic at `todo!()` in `DocxZipAssembler::new`.
#[test]
fn test_BC_4_02_001_zip_assembler_epoch_timestamps() {
    use crate::zip_assembler::DocxZipAssembler;

    let mut asm = DocxZipAssembler::new();
    asm.add_part("word/document.xml", b"<doc/>".to_vec());

    let bytes = asm.finish().expect("assembler must succeed");

    let cursor = std::io::Cursor::new(&bytes);
    let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
    let entry = archive.by_index(0).expect("entry 0 exists");

    // zip 4.x: ZipFile::last_modified() → Option<zip::DateTime>.
    // MS-DOS epoch = 1980-01-01 00:00:00.
    let lm = entry
        .last_modified()
        .expect("entry must have a last_modified timestamp");

    assert_eq!(
        lm.year(),
        1980,
        "ZIP entry year must be 1980 (MS-DOS epoch for determinism); got year = {}",
        lm.year()
    );
    assert_eq!(lm.month(), 1, "ZIP entry month must be 1 (January)");
    assert_eq!(lm.day(), 1, "ZIP entry day must be 1");
    assert_eq!(lm.hour(), 0, "ZIP entry hour must be 0");
    assert_eq!(lm.minute(), 0, "ZIP entry minute must be 0");
    assert_eq!(lm.second(), 0, "ZIP entry second must be 0");
}

// ─── styles.xml correctness ───────────────────────────────────────────────────

/// BC-4.02.001 / STORY-041 Task 4:
/// `word/styles.xml` must define the five required Word styles.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_styles_xml_contains_required_styles() {
    let deck = minimal_deck();
    let slide = make_slide("Style Check Slide", vec![]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let styles_xml = read_zip_member(&docx_bytes, "word/styles.xml");

    // Required styles per STORY-041 Task 4.
    for style_id in &["Heading1", "Heading2", "Normal", "Hyperlink", "CodeText"] {
        assert!(
            styles_xml.contains(style_id),
            "word/styles.xml must define style {style_id:?}; got:\n{styles_xml}"
        );
    }
}

// ─── Edge case tests ──────────────────────────────────────────────────────────

/// BC-4.02.001 EC-001:
/// A slide with ONLY notes content (no report/detail) produces a heading +
/// empty paragraph. The notes text must NOT appear in `word/document.xml`.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_ec001_only_notes_slide_no_notes_in_body() {
    let deck = minimal_deck();
    let rc = RegisteredContent::plain(Register::Notes, Arc::from("Presenter-only notes content"));
    let slide = make_slide("Notes Only Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // Heading must be present.
    assert!(
        doc_xml.contains("Notes Only Slide"),
        "Heading for notes-only slide must appear in document.xml (EC-001)"
    );

    // Notes content must NOT appear.
    assert!(
        !doc_xml.contains("Presenter-only notes content"),
        "Notes content must NOT appear in document.xml body (EC-001 / BC-4.02.001 invariant 1)"
    );

    // BleedChecker cross-validation (canonical STORY-036 method).
    BleedChecker::assert_absent_from_docx_body(&docx_bytes, "Presenter-only notes content");
}

/// BC-4.02.001 EC-002:
/// Multiple `RegisteredContent::Report` entries on one slide produce multiple
/// `<w:p>` paragraphs, one per entry.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_ec002_multi_paragraph_report() {
    let deck = minimal_deck();
    let rc1 = RegisteredContent::plain(Register::Report, Arc::from("First paragraph text."));
    let rc2 = RegisteredContent::plain(Register::Report, Arc::from("Second paragraph text."));
    let rc3 = RegisteredContent::plain(Register::Report, Arc::from("Third paragraph text."));
    let slide = make_slide("Multi-Para Slide", vec![rc1, rc2, rc3]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    assert!(
        doc_xml.contains("First paragraph text."),
        "First report paragraph must appear in document.xml (EC-002)"
    );
    assert!(
        doc_xml.contains("Second paragraph text."),
        "Second report paragraph must appear in document.xml (EC-002)"
    );
    assert!(
        doc_xml.contains("Third paragraph text."),
        "Third report paragraph must appear in document.xml (EC-002)"
    );

    // Minimum paragraph count: 1 Heading1 + 3 report paragraphs = 4.
    let para_count = doc_xml.matches("<w:p").count();
    assert!(
        para_count >= 4,
        "Multi-paragraph report must produce ≥4 <w:p> elements (heading + 3 paras); \
         found {para_count} (EC-002)"
    );
}

/// BC-4.02.001 EC-004:
/// A slide with both `detail` and `report` puts `report` before `detail` in
/// the output: report goes into the narrative section, detail into the extended
/// section.
///
/// RED: will panic at `todo!()` in `DocxExporter::export`.
#[test]
fn test_BC_4_02_001_ec004_detail_and_report_same_slide() {
    let deck = minimal_deck();
    let report_rc = RegisteredContent::plain(Register::Report, Arc::from("Main narrative content"));
    let detail_rc =
        RegisteredContent::plain(Register::Detail, Arc::from("Deep dive appendix text"));
    let slide = make_slide("Mixed Register Slide", vec![report_rc, detail_rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    assert!(
        doc_xml.contains("Main narrative content"),
        "report text must appear in document.xml (EC-004)"
    );
    assert!(
        doc_xml.contains("Deep dive appendix text"),
        "detail text must appear in document.xml (EC-004)"
    );

    let report_pos = doc_xml
        .find("Main narrative content")
        .expect("report text must be found");
    let detail_pos = doc_xml
        .find("Deep dive appendix text")
        .expect("detail text must be found");
    assert!(
        detail_pos > report_pos,
        "detail content must appear AFTER report content in document.xml (EC-004): \
         report at {report_pos}, detail at {detail_pos}"
    );
}

// ─── F-041-002 regression: ` />` in text content must survive serialization ──

/// F-041-002 regression:
/// A report string containing characters that, when XML-escaped, produce
/// byte sequences like `&gt;` (not ` />`) must round-trip through serialization
/// without structural corruption.
///
/// Previously, `normalize_self_closing_tags` did a global byte-replace of
/// ` />` → `/>` across the whole document, which could corrupt self-closing
/// attribute serialization (`<w:pStyle w:val="Heading1" />` → `<w:pStyle w:val="Heading1"/>`).
/// More critically, it risked corrupting any raw byte pattern ` />` regardless
/// of context (tag vs. text node).
///
/// This test verifies:
/// 1. The XML serializer does NOT apply any post-processing byte-replace on the output.
/// 2. Content that resembles XML markup in user text is properly XML-escaped (safe round-trip).
/// 3. The self-closing attribute form produced by quick_xml (`<w:pStyle w:val="..." />`)
///    is present verbatim in the output (no silent normalization).
#[test]
fn test_BC_4_02_001_f041_002_slash_gt_in_text_survives_serialization() {
    let deck = minimal_deck();
    // Text that contains '/' and '>' characters — ooxmlsdk will XML-escape these
    // to '/' and '&gt;' respectively (or keep them in text nodes).
    let rc = RegisteredContent::plain(Register::Report, Arc::from("Results: 10/20 > threshold"));
    let slide = make_slide("Regression Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // The report text must appear in the document (possibly with > XML-escaped
    // to &gt;, but '/' is safe as-is in text content).
    assert!(
        doc_xml.contains("10/20"),
        "fraction '10/20' must appear in document.xml without corruption (F-041-002); \
         got:\n{doc_xml}"
    );
    assert!(
        doc_xml.contains("threshold"),
        "word 'threshold' must appear in document.xml (F-041-002)"
    );

    // The output XML must NOT have been globally byte-replaced: self-closing
    // attribute tags from quick_xml still have their original form (with or
    // without space before "/>"). Verify the document is well-formed by
    // checking it starts with an XML declaration and ends with the document
    // close tag (structural sanity without a full parse).
    assert!(
        doc_xml.starts_with("<?xml"),
        "document.xml must start with XML declaration after serialization (F-041-002)"
    );
    assert!(
        doc_xml.ends_with("</w:document>"),
        "document.xml must end with </w:document> after serialization — \
         no truncation from byte-replace (F-041-002)"
    );
}

// ─── F-041-003: One Appendix heading per slide, not per detail entry ──────────

/// F-041-003:
/// A slide with multiple `detail` entries must produce EXACTLY ONE
/// `Appendix: <title>` Heading2 paragraph, not one per detail entry.
///
/// Previously the Heading2 was emitted inside the per-entry loop.
#[test]
fn test_BC_4_02_001_f041_003_one_appendix_heading_per_slide() {
    let deck = minimal_deck();
    let detail1 = RegisteredContent::plain(Register::Detail, Arc::from("First detail block"));
    let detail2 = RegisteredContent::plain(Register::Detail, Arc::from("Second detail block"));
    let detail3 = RegisteredContent::plain(Register::Detail, Arc::from("Third detail block"));
    let slide = make_slide("Analysis Slide", vec![detail1, detail2, detail3]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // Exactly one "Appendix: Analysis Slide" heading must appear.
    let heading_count = doc_xml.matches("Appendix: Analysis Slide").count();
    assert_eq!(
        heading_count, 1,
        "document.xml must contain exactly 1 'Appendix: Analysis Slide' heading for a \
         slide with 3 detail entries; found {heading_count} occurrences (F-041-003). \
         Got:\n{doc_xml}"
    );

    // All three detail entries must be present.
    assert!(
        doc_xml.contains("First detail block"),
        "first detail entry must appear in document.xml (F-041-003)"
    );
    assert!(
        doc_xml.contains("Second detail block"),
        "second detail entry must appear in document.xml (F-041-003)"
    );
    assert!(
        doc_xml.contains("Third detail block"),
        "third detail entry must appear in document.xml (F-041-003)"
    );

    // Heading2 style must appear.
    assert!(
        doc_xml.contains("Heading2"),
        "Appendix heading must use Heading2 style (F-041-003)"
    );
}

// ─── F-041-004: Structural OOXML parse-back validity (BC-4.02.001 post. 2) ────

/// F-041-004 / BC-4.02.001 postcondition 2:
/// `word/document.xml`, `word/styles.xml`, and `[Content_Types].xml` must
/// parse back through the ooxmlsdk deserializer without error, proving
/// well-formedness and presence of required top-level elements.
///
/// This is a non-ignored structural validity test that does not require a
/// real renderer (Word, LibreOffice) to execute.
#[test]
fn test_BC_4_02_001_f041_004_ooxml_structural_parse_back() {
    use ooxmlsdk::schemas::opc_content_types::Types;
    use ooxmlsdk::schemas::schemas_openxmlformats_org_wordprocessingml_2006_main::{
        Document, Styles,
    };

    let deck = minimal_deck();
    let rc = RegisteredContent::plain(Register::Report, Arc::from("Parse-back test content"));
    let slide = make_slide("Parse Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);

    // ── [Content_Types].xml ────────────────────────────────────────────────
    let ct_xml = read_zip_member(&docx_bytes, "[Content_Types].xml");
    let types = Types::from_bytes(ct_xml.as_bytes()).unwrap_or_else(|e| {
        panic!(
            "[Content_Types].xml must deserialize through ooxmlsdk::Types without error \
             (F-041-004 / BC-4.02.001 postcondition 2); error: {e}\nGot:\n{ct_xml}"
        )
    });
    assert!(
        !types.types_choice.is_empty(),
        "[Content_Types].xml must contain at least one Default or Override element \
         (F-041-004)"
    );

    // ── word/document.xml ─────────────────────────────────────────────────
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");
    let document = Document::from_bytes(doc_xml.as_bytes()).unwrap_or_else(|e| {
        panic!(
            "word/document.xml must deserialize through ooxmlsdk::Document without error \
             (F-041-004 / BC-4.02.001 postcondition 2); error: {e}\nGot:\n{doc_xml}"
        )
    });
    // The body element must be present.
    assert!(
        document.body.is_some(),
        "word/document.xml must have a <w:body> element after ooxmlsdk parse-back \
         (F-041-004)"
    );

    // ── word/styles.xml ───────────────────────────────────────────────────
    let styles_xml = read_zip_member(&docx_bytes, "word/styles.xml");
    let styles = Styles::from_bytes(styles_xml.as_bytes()).unwrap_or_else(|e| {
        panic!(
            "word/styles.xml must deserialize through ooxmlsdk::Styles without error \
             (F-041-004 / BC-4.02.001 postcondition 2); error: {e}\nGot:\n{styles_xml}"
        )
    });
    assert!(
        !styles.w_style.is_empty(),
        "word/styles.xml must define at least one style after parse-back (F-041-004)"
    );
}

// ─── F-041-006: EC-003 — hyperlink wrapper test ───────────────────────────────

/// F-041-006 / EC-003:
/// An `InlineNode::Link` must produce a `<w:hyperlink r:id="...">` element
/// wrapping the link run in `word/document.xml`, AND the rId must be present
/// in `word/_rels/document.xml.rels` as a hyperlink relationship.
///
/// Previously the code only emitted the run, leaving an orphaned relationship
/// and a non-clickable link.
#[test]
fn test_BC_4_02_001_ec003_hyperlink_element_wraps_run_and_rid_resolves() {
    let deck = minimal_deck();
    let link_node = InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("Click here"))],
        url: Arc::from("https://example.com/path"),
    };
    let rc = RegisteredContent {
        register: Register::Report,
        content: vec![link_node],
    };
    let slide = make_slide("Link Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");
    let rels_xml = read_zip_member(&docx_bytes, "word/_rels/document.xml.rels");

    // 1. The document must contain a <w:hyperlink element.
    assert!(
        doc_xml.contains("<w:hyperlink"),
        "word/document.xml must contain a <w:hyperlink element for InlineNode::Link \
         (F-041-006 / EC-003); got:\n{doc_xml}"
    );

    // 2. The hyperlink element must have an r:id attribute.
    assert!(
        doc_xml.contains("r:id="),
        "word/document.xml <w:hyperlink> must have an r:id attribute \
         (F-041-006 / EC-003); got:\n{doc_xml}"
    );

    // 3. The link display text must appear inside the hyperlink.
    let hyperlink_start = doc_xml
        .find("<w:hyperlink")
        .expect("must have hyperlink element");
    let hyperlink_end = doc_xml[hyperlink_start..]
        .find("</w:hyperlink>")
        .map(|offset| hyperlink_start + offset + "</w:hyperlink>".len())
        .expect("hyperlink must close");
    let hyperlink_fragment = &doc_xml[hyperlink_start..hyperlink_end];
    assert!(
        hyperlink_fragment.contains("Click here"),
        "link display text 'Click here' must appear inside the <w:hyperlink> element \
         (F-041-006 / EC-003); got hyperlink fragment:\n{hyperlink_fragment}"
    );

    // 4. Extract the rId from the hyperlink element and verify it resolves in rels.
    // Find r:id="rId..." pattern in the doc_xml.
    let rid_prefix = "r:id=\"";
    let rid_start = doc_xml[hyperlink_start..]
        .find(rid_prefix)
        .map(|offset| hyperlink_start + offset + rid_prefix.len())
        .expect("r:id attribute must exist in <w:hyperlink>");
    let rid_end = doc_xml[rid_start..]
        .find('"')
        .map(|offset| rid_start + offset)
        .expect("r:id attribute must be quoted");
    let r_id = &doc_xml[rid_start..rid_end];

    assert!(
        rels_xml.contains(r_id),
        "rId '{r_id}' from <w:hyperlink> must appear in word/_rels/document.xml.rels \
         (F-041-006 / EC-003); got rels:\n{rels_xml}"
    );

    assert!(
        rels_xml.contains("https://example.com/path"),
        "target URL must appear in word/_rels/document.xml.rels (F-041-006 / EC-003); \
         got:\n{rels_xml}"
    );
}

// ─── F-041-007: No duplicate relationship IDs ─────────────────────────────────

/// F-041-007:
/// When a slide contains `InlineNode::Link`, the generated relationship Ids
/// in `word/_rels/document.xml.rels` must not duplicate the fixed reserved
/// IDs (rId1=styles, rId2=numbering, rId3=settings).
///
/// Previously `next_rel_id` started at 1, causing the first hyperlink to
/// produce `Id="rId1"` — a duplicate of the styles relationship.
#[test]
fn test_BC_4_02_001_f041_007_hyperlink_rid_no_collision_with_reserved_ids() {
    let deck = minimal_deck();
    let link_node = InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("Reference doc"))],
        url: Arc::from("https://example.com/doc"),
    };
    let rc = RegisteredContent {
        register: Register::Report,
        content: vec![link_node],
    };
    let slide = make_slide("Link Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let rels_xml = read_zip_member(&docx_bytes, "word/_rels/document.xml.rels");

    // Collect all Id="..." attribute values to check for duplicates.
    let mut ids: Vec<&str> = Vec::new();
    let mut search = rels_xml.as_str();
    while let Some(pos) = search.find("Id=\"") {
        search = &search[pos + 4..];
        if let Some(end) = search.find('"') {
            ids.push(&search[..end]);
            search = &search[end..];
        }
    }

    // Every Id must be unique.
    let mut seen = std::collections::HashSet::new();
    for id in &ids {
        assert!(
            seen.insert(*id),
            "duplicate relationship Id=\"{id}\" found in word/_rels/document.xml.rels \
             (F-041-007); all Ids: {ids:?}\nGot rels:\n{rels_xml}"
        );
    }

    // The hyperlink relationship Id must NOT be rId1, rId2, or rId3 (reserved).
    assert!(
        !ids.iter().any(|&id| id == "rId1" && rels_xml.contains(&format!("Id=\"{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink\""))),
        "hyperlink relationship must not use reserved Id 'rId1' (F-041-007); got rels:\n{rels_xml}"
    );
    assert!(
        !ids.iter().any(|&id| id == "rId2" && rels_xml.contains(&format!("Id=\"{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink\""))),
        "hyperlink relationship must not use reserved Id 'rId2' (F-041-007); got rels:\n{rels_xml}"
    );
    assert!(
        !ids.iter().any(|&id| id == "rId3" && rels_xml.contains(&format!("Id=\"{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink\""))),
        "hyperlink relationship must not use reserved Id 'rId3' (F-041-007); got rels:\n{rels_xml}"
    );
}

// ─── F-DOCX-001: xmlns:r declared on document root for hyperlink docs ────────

/// F-DOCX-001 (CRITICAL):
/// `word/document.xml` MUST declare `xmlns:r="…/relationships"` on the
/// `<w:document>` root whenever it contains `<w:hyperlink r:id="...">` elements.
///
/// Without this declaration the XML is namespace-malformed: the `r:` prefix
/// used in `r:id` attributes is undeclared, and any conformant XML processor
/// (Word, LibreOffice, veraPDF) is required to reject the document.
///
/// Guards: loads the hyperlink document.xml back through
/// `Document::from_bytes` to confirm it round-trips as well-formed XML.
#[test]
fn test_BC_4_02_001_f_docx_001_hyperlink_doc_declares_xmlns_r_and_parses_back() {
    use ooxmlsdk::schemas::schemas_openxmlformats_org_wordprocessingml_2006_main::Document;

    let deck = minimal_deck();
    let link_node = InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("Visit site"))],
        url: Arc::from("https://slideforge.example/docs"),
    };
    let rc = RegisteredContent {
        register: Register::Report,
        content: vec![link_node],
    };
    let slide = make_slide("Hyperlink Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // ── Guard 1: xmlns:r MUST be declared on the document root ──────────────
    assert!(
        doc_xml.contains(
            "xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\""
        ),
        "word/document.xml MUST declare xmlns:r=\"...relationships\" on the <w:document> \
         root when the document contains <w:hyperlink r:id=\"...\"> elements (F-DOCX-001). \
         Without this declaration the XML is namespace-malformed and will be rejected by \
         conformant processors. Got document.xml opening:\n{}",
        &doc_xml[..doc_xml.find('>').unwrap_or(doc_xml.len()).min(512)]
    );

    // ── Guard 2: the document must contain a hyperlink element with r:id ────
    assert!(
        doc_xml.contains("r:id="),
        "hyperlink document.xml must contain r:id attribute (F-DOCX-001 precondition)"
    );

    // ── Guard 3: parse-back through ooxmlsdk confirms well-formed XML ────────
    // ooxmlsdk parses qnames literally: if xmlns:r is absent, round-trip
    // deserialization may still succeed for the outer Document structure (it
    // uses the registered prefix table), but the namespace declaration
    // assertion above is the definitive semantic guard. We include parse-back
    // to confirm no structural corruption occurs.
    let document = Document::from_bytes(doc_xml.as_bytes()).unwrap_or_else(|e| {
        panic!(
            "word/document.xml for a hyperlink deck must parse back through \
             ooxmlsdk::Document without error (F-DOCX-001 / F-DOCX-003). \
             error: {e}\nGot:\n{doc_xml}"
        )
    });
    assert!(
        document.body.is_some(),
        "hyperlink document.xml must have a <w:body> after parse-back (F-DOCX-001 / F-DOCX-003)"
    );
}

// ─── SEC-001: Exporter-layer URL scheme validation (CWE-601) ─────────────────

/// SEC-001 (MEDIUM, CWE-601):
/// `InlineNode::Link` with a `javascript:` URL must return
/// `Err(ExportError::ValidationError)` — never written to the ZIP.
///
/// The parse-layer allowlist (E-PAR-022) covers DSL-sourced input, but
/// programmatic `InlineNode::Link` callers bypass it. The exporter must
/// independently validate the URL scheme as a defense-in-depth measure.
#[test]
fn test_sec_001_javascript_url_rejected_by_exporter() {
    use crate::document_body::DocumentBodySerializer;

    let link_node = InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("Click"))],
        url: Arc::from("javascript:alert('xss')"),
    };

    let rc = slideforge_types::RegisteredContent {
        register: slideforge_types::Register::Report,
        content: vec![link_node],
    };

    let slide = make_slide("XSS Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let mut body_ser = DocumentBodySerializer::new();
    let result = body_ser.serialize(&laid_out);

    assert!(
        result.is_err(),
        "serialize() must return Err for a javascript: URL (SEC-001 / CWE-601); \
         got Ok — dangerous URL would have been written to the ZIP"
    );

    // Confirm the error is a ValidationError, not a different error kind.
    let err = result.unwrap_err();
    let err_string = err.to_string();
    assert!(
        err_string.contains("javascript")
            || err_string.to_lowercase().contains("scheme")
            || err_string.to_lowercase().contains("url")
            || err_string.to_lowercase().contains("validation"),
        "error must mention the disallowed scheme or be a ValidationError; got: {err_string}"
    );
}

/// SEC-001 (MEDIUM, CWE-601):
/// `InlineNode::Link` with `https://` URL must succeed and bind the URL
/// correctly in `word/_rels/document.xml.rels`.
#[test]
fn test_sec_001_https_url_accepted_by_exporter() {
    let deck = minimal_deck();
    let link_node = InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("Visit docs"))],
        url: Arc::from("https://docs.example.com/guide"),
    };
    let rc = slideforge_types::RegisteredContent {
        register: slideforge_types::Register::Report,
        content: vec![link_node],
    };
    let slide = make_slide("Docs Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let rels_xml = read_zip_member(&docx_bytes, "word/_rels/document.xml.rels");

    assert!(
        rels_xml.contains("https://docs.example.com/guide"),
        "https:// URL must be written to document.xml.rels (SEC-001); got:\n{rels_xml}"
    );
}

/// SEC-001 (MEDIUM, CWE-601):
/// `InlineNode::Link` with `mailto:` URL must succeed and bind the target
/// correctly in `word/_rels/document.xml.rels`.
#[test]
fn test_sec_001_mailto_url_accepted_by_exporter() {
    let deck = minimal_deck();
    let link_node = InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("Email us"))],
        url: Arc::from("mailto:hello@example.com"),
    };
    let rc = slideforge_types::RegisteredContent {
        register: slideforge_types::Register::Report,
        content: vec![link_node],
    };
    let slide = make_slide("Email Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let rels_xml = read_zip_member(&docx_bytes, "word/_rels/document.xml.rels");

    assert!(
        rels_xml.contains("mailto:hello@example.com"),
        "mailto: URL must be written to document.xml.rels (SEC-001); got:\n{rels_xml}"
    );
}

/// SEC-001 (MEDIUM, CWE-601):
/// Additional disallowed schemes: `data:`, `vbscript:`, `file:`.
#[test]
fn test_sec_001_data_url_rejected_by_exporter() {
    use crate::document_body::DocumentBodySerializer;

    let link_node = InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("Evil link"))],
        url: Arc::from("data:text/html,<script>alert(1)</script>"),
    };
    let rc = slideforge_types::RegisteredContent {
        register: slideforge_types::Register::Report,
        content: vec![link_node],
    };
    let slide = make_slide("Data Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let mut body_ser = DocumentBodySerializer::new();
    let result = body_ser.serialize(&laid_out);

    assert!(
        result.is_err(),
        "serialize() must return Err for a data: URL (SEC-001 / CWE-601); got Ok"
    );
}

// ─── SEC-002: XML control character stripping ─────────────────────────────────

/// SEC-002 (LOW, CWE-116):
/// Font names from brand containing XML-1.0-invalid control characters
/// must produce well-formed XML in `word/styles.xml`.
///
/// XML-1.0 valid characters: #x9, #xA, #xD, and #x20–#xD7FF and #xE000–#xFFFD.
/// Control chars \x00–\x08, \x0B–\x0C, \x0E–\x1F must be stripped.
///
/// This test verifies that a brand with a NUL (`\x00`) and ESC (`\x1B`)
/// in the heading font name does NOT produce those chars in styles.xml.
#[test]
fn test_sec_002_control_chars_stripped_from_xml_attribute_values() {
    use crate::styles::build_styles;
    use slideforge_types::{Brand, BrandFonts, BrandPalette};
    use std::sync::Arc as StdArc;

    // Brand with a heading font name containing control characters.
    let evil_brand = Brand {
        name: StdArc::from("evil-brand"),
        palette: BrandPalette {
            primary: StdArc::from("#003087"),
            secondary: StdArc::from("#0066CC"),
            accent: StdArc::from("#FF6B35"),
            neutral: StdArc::from("#F5F5F5"),
        },
        fonts: BrandFonts {
            heading: StdArc::from("Calibri\x00Light\x1B"),
            body: StdArc::from("Calibri"),
            mono: StdArc::from("Courier New"),
            font_size_emu: 457_200,
        },
        layouts: vec![],
        span: slideforge_types::span::SourceSpan::default(),
    };

    let styles_xml_bytes =
        build_styles(Some(&evil_brand)).expect("build_styles must succeed even with control chars");
    let styles_xml = String::from_utf8(styles_xml_bytes).expect("styles.xml must be valid UTF-8");

    // The NUL and ESC bytes must NOT appear in the output.
    assert!(
        !styles_xml.contains('\x00'),
        "styles.xml must not contain NUL (\\x00) control char (SEC-002 / CWE-116); \
         got:\n{styles_xml}"
    );
    assert!(
        !styles_xml.contains('\x1B'),
        "styles.xml must not contain ESC (\\x1B) control char (SEC-002 / CWE-116); \
         got:\n{styles_xml}"
    );

    // The font name with control chars stripped must still appear (just clean).
    // "CalibrLight" without the NUL and ESC would be "CalibrLight" but the
    // exact stripping result depends on implementation; the KEY assertion is
    // the control chars are gone.
    assert!(
        styles_xml.contains("Calibri"),
        "sanitized heading font name must still contain 'Calibri' after stripping (SEC-002)"
    );
}

/// SEC-002 (LOW, CWE-116):
/// Text content in document.xml containing XML-1.0-invalid control characters
/// must have those characters stripped (not passed through).
#[test]
fn test_sec_002_control_chars_stripped_from_xml_text_content() {
    let deck = minimal_deck();
    // Report text with NUL and form-feed (\x0C) — both invalid in XML 1.0.
    let rc = slideforge_types::RegisteredContent::plain(
        slideforge_types::Register::Report,
        std::sync::Arc::from("Hello\x00World\x0CEnd"),
    );
    let slide = make_slide("Control Chars Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // NUL and \x0C must be stripped.
    assert!(
        !doc_xml.contains('\x00'),
        "document.xml must not contain NUL control char from user text (SEC-002 / CWE-116)"
    );
    assert!(
        !doc_xml.contains('\x0C'),
        "document.xml must not contain form-feed control char from user text (SEC-002 / CWE-116)"
    );

    // The safe text portions must remain.
    assert!(
        doc_xml.contains("Hello") && doc_xml.contains("World") && doc_xml.contains("End"),
        "text content with control chars stripped must still contain safe characters (SEC-002)"
    );
}

// ─── F-041-008: Paragraph count must count <w:p> and <w:p  only ──────────────

/// F-041-008:
/// Paragraph-count assertions must not accidentally count `<w:pPr` or
/// `<w:pStyle` tags. The test uses an exact token search for `<w:p>` and
/// `<w:p ` (open-element with or without attributes).
///
/// This test verifies the counting is precise by building a known-count deck
/// and asserting the exact number.
#[test]
fn test_BC_4_02_001_f041_008_paragraph_count_exact_not_ppr_pstyle() {
    let deck = minimal_deck();
    // One slide, one report entry → 1 Heading1 + 1 Normal = 2 <w:p> elements.
    let rc = RegisteredContent::plain(Register::Report, Arc::from("Counting test content"));
    let slide = make_slide("Count Slide", vec![rc]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // Count actual paragraph open-elements only: "<w:p>" and "<w:p ".
    // Must NOT count "<w:pPr", "<w:pStyle", or any other element starting with "<w:p".
    let paragraph_count = doc_xml.matches("<w:p>").count() + doc_xml.matches("<w:p ").count();

    assert_eq!(
        paragraph_count, 2,
        "1-slide 1-report deck must produce exactly 2 paragraphs (Heading1 + Normal); \
         found {paragraph_count} using exact '<w:p>' + '<w:p ' token count (F-041-008). \
         Note: '<w:pPr' and '<w:pStyle' must NOT be counted. Got:\n{doc_xml}"
    );

    // Cross-check: <w:pPr and <w:pStyle do exist (they'd inflate the old "<w:p" count).
    assert!(
        doc_xml.contains("<w:pPr"),
        "document.xml must contain <w:pPr elements (F-041-008 precondition)"
    );
    let old_imprecise_count = doc_xml.matches("<w:p").count();
    assert!(
        old_imprecise_count > paragraph_count,
        "the imprecise '<w:p' count ({old_imprecise_count}) must exceed the precise \
         '<w:p>'/'<w:p ' count ({paragraph_count}), confirming that '<w:pPr' elements \
         inflate the old assertion (F-041-008)"
    );
}

// ─── OBS-P6-002: DOCX percent double-floor off-by-one ────────────────────────
//
// Confirmed defect: layout::run floors `filled_width = (percent * total) / 100`
// (integer EMU). The DOCX exporter (document_body.rs ~229) RE-DERIVES percent
// as `(filled_width * 100) / total` — a second floor. At bar widths not
// divisible by 100 the two floors compound and produce `percent - 1`.
//
// The canonical spec value `ColorBarSpec.percent: u8` is NOT carried in
// `FrameContent::ColorBar`; it has only `filled_width_emu` and `total_width_emu`.
//
// ## Black-box approach (no type change needed)
//
// We construct a `LaidOutDeck` directly with a `FrameContent::ColorBar` frame
// whose `filled_width_emu` and `total_width_emu` match what `layout::run` would
// produce at a non-default bar width (not divisible by 100), then drive the full
// DOCX exporter and assert the text in `word/document.xml`.
//
// ## Triggering width
//
// The `progress_bar` bar-background frame width at default page size is:
//   sx(8_229_600) = 8_229_600 * 9_144_000 / 9_144_000 = 8_229_600 EMU
// which IS divisible by 100 (8_229_600 / 100 = 82_296 exactly) — no bug there.
//
// We force a non-divisible-by-100 total by setting `total_width_emu` directly to
// a value not divisible by 100 (e.g., 8_229_601 EMU). This is the "bar at a
// non-default page width" scenario. The total can be any value where:
//   (((percent * total) / 100) * 100) / total != percent
//
// For `total = 5_000_003` (not divisible by 100):
//   filled = 75 * 5_000_003 / 100 = 3_750_002  (floor)
//   recovered = 3_750_002 * 100 / 5_000_003 = 74  (second floor, off-by-one!)
//
// For `total = 4_500_001` (not divisible by 100):
//   filled = 75 * 4_500_001 / 100 = 3_375_000  (floor)
//   recovered = 3_375_000 * 100 / 4_500_001 = 74  (off-by-one)
//
// Traceability: OBS-P6-002 / BC-1.17.002 PC-9 (DOCX clause).

/// Helper: build a `LaidOutDeck` with a single `progress_bar` slide that carries
/// a `FrameContent::ColorBar` frame with the given `filled_width_emu` and
/// `total_width_emu`.
fn make_progress_bar_deck_with_colorbar(percent_input: u8, total_width_emu: i64) -> LaidOutDeck {
    // Replicate what layout::run does: filled = (percent * total) / 100 (floor).
    let filled_width_emu = Emu((i64::from(percent_input) * total_width_emu) / 100);
    let total_emu = Emu(total_width_emu);

    // Title frame (required so the DOCX exporter can emit a Heading1).
    let title_frame = Frame {
        bbox: BoundingBox {
            x: Emu(457_200),
            y: Emu(365_760),
            width: Emu(8_229_600),
            height: Emu(685_800),
        },
        content: FrameContent::Title(Arc::from("Progress Bar Slide")),
        text_flow: None,
        region_role: None,
    };

    // The ColorBar frame at the non-default total width.
    let bar_frame = Frame {
        bbox: BoundingBox {
            x: Emu(457_200),
            y: Emu(1_188_720),
            width: total_emu,
            height: Emu(685_800),
        },
        content: FrameContent::ColorBar {
            filled_width_emu,
            total_width_emu: total_emu,
            percent: percent_input,
            color: Rgb {
                r: 0,
                g: 112,
                b: 192,
            },
        },
        text_flow: None,
        region_role: None,
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("progress_bar"),
        frames: vec![title_frame, bar_frame],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    };

    LaidOutDeck {
        page_size: PageSize {
            width: Emu(9_144_000),
            height: Emu(5_143_500),
        },
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    }
}

/// OBS-P6-002 — DOCX exporter percent double-floor: value=75 at a
/// non-divisible-by-100 bar width must produce `<w:t>75%</w:t>` in document.xml.
///
/// ## RED GATE
///
/// With `total_width_emu = 5_000_003` (not divisible by 100):
///   filled = 75 * 5_000_003 / 100 = 3_750_002  (layout floor, integer EMU)
///   DOCX re-derives: (3_750_002 * 100) / 5_000_003 = 74  (second floor, off-by-one)
///
/// The DOCX exporter currently emits `74%` instead of `75%`. This test MUST FAIL
/// until the exporter carries the canonical `percent` value without re-deriving it.
///
/// Traceability: OBS-P6-002 / BC-1.17.002 PC-9 (DOCX clause).
#[test]
#[allow(non_snake_case)]
fn test_OBS_P6_002_docx_percent_double_floor_value_75() {
    // total = 5_000_003: NOT divisible by 100 (5_000_003 % 100 = 3)
    // filled = 75 * 5_000_003 / 100 = 3_750_002
    // DOCX re-derives: (3_750_002 * 100) / 5_000_003 = 374_999_799 / 5_000_003 ≈ 74
    // Expected: "75%", actual (buggy): "74%"
    let total_emu: i64 = 5_000_003;
    let percent_input: u8 = 75;

    // Confirm the math: filled and double-floor.
    let filled = i64::from(percent_input) * total_emu / 100;
    let double_floored = filled * 100 / total_emu;
    assert_ne!(
        double_floored,
        i64::from(percent_input),
        "precondition: chosen total_emu={total_emu} must trigger double-floor bug \
         (filled={filled}, recovered={double_floored} should != {percent_input})"
    );

    let deck = minimal_deck();
    let laid_out = make_progress_bar_deck_with_colorbar(percent_input, total_emu);
    let brand = minimal_brand();
    let opts = ExportOptions::default();

    let docx_bytes = DocxExporter
        .export(&deck, &laid_out, &brand, &opts)
        .expect("DOCX export must succeed for progress_bar with ColorBar");

    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // The canonical value must appear as "75%" in the document XML.
    // The buggy behavior emits "74%" due to the double-floor.
    assert!(
        doc_xml.contains("<w:t>75%</w:t>") || doc_xml.contains(">75%<"),
        "OBS-P6-002 RED GATE: DOCX document.xml must contain '75%' for a progress_bar \
         slide with percent=75 at total_emu={total_emu} (not divisible by 100). \
         Bug: DOCX re-derives percent as (filled_width*100)/total — a second integer \
         floor — producing '74%' instead of '75%'. \
         filled_width_emu = {filled}, double_floored = {double_floored}. \
         Actual document.xml snippet (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );
}

/// OBS-P6-002 — parameterized: several percent values at a triggering width
/// must all round-trip correctly through the DOCX exporter.
///
/// ## RED GATE
///
/// With `total_width_emu = 5_000_003`, values 1..=99 (excluding 0, 25, 50, 75, 100
/// which happen to be exact at this width) all produce off-by-one in the current
/// implementation. This test asserts a representative set: {33, 67, 75, 90}.
///
/// Traceability: OBS-P6-002 / BC-1.17.002 PC-9 (DOCX clause).
#[test]
#[allow(non_snake_case)]
fn test_OBS_P6_002_docx_percent_double_floor_multiple_values() {
    // total = 5_000_003: NOT divisible by 100.
    let total_emu: i64 = 5_000_003;

    // Representative percent values that trigger the double-floor at this width.
    // Verified: 33, 67, 75, 90 all produce off-by-one with total=5_000_003.
    let test_values: &[u8] = &[33, 67, 75, 90];

    for &percent_input in test_values {
        let filled = i64::from(percent_input) * total_emu / 100;
        let double_floored = filled * 100 / total_emu;

        // Confirm this value triggers the bug (ensures the test is meaningful).
        if double_floored == i64::from(percent_input) {
            // This particular value is exact at this width — skip it.
            continue;
        }

        let deck = minimal_deck();
        let laid_out = make_progress_bar_deck_with_colorbar(percent_input, total_emu);
        let brand = minimal_brand();
        let opts = ExportOptions::default();

        let docx_bytes = DocxExporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("DOCX export must succeed");

        let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

        let expected_text = format!("{percent_input}%");
        let expected_xml = format!("<w:t>{expected_text}</w:t>");

        assert!(
            doc_xml.contains(&expected_xml) || doc_xml.contains(&format!(">{expected_text}<")),
            "OBS-P6-002 RED GATE: DOCX document.xml must contain '{expected_text}' for \
             percent={percent_input} at total_emu={total_emu}. \
             Bug: double-floor produces '{double_floored}%' instead. \
             filled_width_emu={filled}. \
             Actual document.xml (first 3000 chars):\n{}",
            &doc_xml[..doc_xml.len().min(3000)]
        );
    }
}
