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
//! | `test_BC_4_02_001_exporter_trait_id_and_extension` | precondition 3 | AC-001 |
//! | `test_BC_4_02_001_determinism_same_deck_twice` | invariant 4 | Test Strategy |
//! | `test_BC_4_02_001_two_slides_two_headings` | postcondition 3 | AC-003 |
//! | `test_BC_4_02_001_ec001_only_notes_slide_no_notes_in_body` | EC-001 | AC-005 |
//! | `test_BC_4_02_001_ec002_multi_paragraph_report` | EC-002 | AC-003 |
//! | `test_BC_4_02_001_ec004_detail_and_report_same_slide` | EC-004 | AC-007 |
//! | `test_BC_4_02_001_zip_assembler_deterministic_entry_order` | invariant 4 | Task 2 |
//! | `test_BC_4_02_001_zip_assembler_epoch_timestamps` | invariant 4 | Task 2 |
//! | `test_BC_4_02_001_styles_xml_contains_required_styles` | postcondition 2 | Task 4 |

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(non_snake_case)]
#![allow(clippy::doc_markdown)]

use std::io::Read as IoRead;
use std::sync::Arc;

use slideforge_eval::BleedChecker;
use slideforge_layout::types::{
    BoundingBox, Emu, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize,
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

/// BC-4.02.001 precondition 3 / AC-001:
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

    // Bold run property.
    assert!(
        doc_xml.contains("<w:b/>") || doc_xml.contains("<w:b/>"),
        "Bold InlineNode must produce <w:b/> run property (AC-008); got:\n{doc_xml}"
    );

    // Italic run property.
    assert!(
        doc_xml.contains("<w:i/>") || doc_xml.contains("<w:i/>"),
        "Italic InlineNode must produce <w:i/> run property (AC-008); got:\n{doc_xml}"
    );

    assert!(
        doc_xml.contains("Bold"),
        "Bold text \"Bold\" must appear in document.xml (AC-008)"
    );
    assert!(
        doc_xml.contains("italic"),
        "Italic text \"italic\" must appear in document.xml (AC-008)"
    );
    assert!(
        doc_xml.contains(" and "),
        "Plain text \" and \" must appear in document.xml (AC-008)"
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
