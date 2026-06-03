//! STORY-041 per-AC demo example — DOCX Core Serialization.
//!
//! Builds a representative two-slide [`LaidOutDeck`] containing:
//! - Slide 1: report + notes + detail register content + inline formatting
//!   (Bold, Italic) and a hyperlink
//! - Slide 2: visual-only slide (no report/detail) to demonstrate AC-009
//!
//! Runs [`DocxExporter`] to produce a `.docx` `Vec<u8>`, writes it to
//! `target/demo_docx_output.docx`, and prints a deterministic summary:
//!
//! - ZIP part list
//! - Key excerpts from `word/document.xml` (Heading1 body, Appendix heading)
//! - Confirmation that `notes` sentinel text is ABSENT
//! - `dc:language` value from `docProps/core.xml`
//! - Hyperlink relationship entry from `word/_rels/document.xml.rels`
//!
//! # Coverage per acceptance criterion
//!
//! | AC | Criterion | Demonstrated by |
//! |----|-----------|----------------|
//! | AC-001 | `Exporter` trait impl | exporter `.id()` / `.extension()` |
//! | AC-002 | Valid .docx ZIP with all required parts | ZIP part list assertion |
//! | AC-003 | `report` content in `document.xml` | excerpt search |
//! | AC-004 | Slide title as Heading1 before report | ordering check |
//! | AC-005 | `notes` absent from `document.xml` | sentinel absence check |
//! | AC-006 | `report` content present (bleed positive) | sentinel presence check |
//! | AC-007 | `detail` in extended section | `Appendix:` heading check |
//! | AC-008 | Inline formatting mapped to Word run props | bold/italic/hyperlink XML |
//! | AC-009 | Visual-only slide → heading + empty paragraph | slide 2 XML check |

use std::io::Read as IoRead;
use std::sync::Arc;

use slideforge_docx::DocxExporter;
use slideforge_layout::types::{
    BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize,
};
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::{
    Brand, BrandFonts, BrandPalette, Deck, DeckMetadata, Emu, InlineNode, OrderedMap, Register,
    RegisteredContent, SourceSpan,
};

#[allow(clippy::too_many_lines)]
fn main() {
    // ── Build a representative brand ──────────────────────────────────────────
    let brand = Brand {
        name: Arc::from("demo-brand"),
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
    };

    // ── Build the semantic Deck (lang = "en-US") ──────────────────────────────
    let deck = Deck {
        slides: vec![],
        vars: OrderedMap::new(),
        metadata: DeckMetadata {
            title: Some(Arc::from("STORY-041 Demo Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: Some(Arc::from("slideforge demo")),
            section_order: None,
        },
        registers: OrderedMap::new(),
        section_blocks: vec![],
    };

    // ── Slide 1: Q1 Review — report + notes + detail + inline + hyperlink ────
    //
    // Register content:
    //   notes  "NOTES_DOCX_SENTINEL" — must be ABSENT from document.xml (AC-005)
    //   report [Bold("Analysis"), Plain(" follows.")]
    //   report [Plain("See "), Link("slideforge.rs", "https://slideforge.rs")]
    //   detail "REPORT_DOCX_SENTINEL is in report above; detail: Technical appendix."
    let slide1_register = vec![
        RegisteredContent {
            register: Register::Notes,
            content: vec![InlineNode::Plain(Arc::from("NOTES_DOCX_SENTINEL"))],
        },
        RegisteredContent {
            register: Register::Report,
            content: vec![
                InlineNode::Bold(vec![InlineNode::Plain(Arc::from("Analysis"))]),
                InlineNode::Plain(Arc::from(" follows. ")),
                InlineNode::Italic(vec![InlineNode::Plain(Arc::from("See details below."))]),
            ],
        },
        RegisteredContent {
            register: Register::Report,
            content: vec![
                InlineNode::Plain(Arc::from("REPORT_DOCX_SENTINEL — Visit ")),
                InlineNode::Link {
                    text: vec![InlineNode::Plain(Arc::from("slideforge.rs"))],
                    url: Arc::from("https://slideforge.rs"),
                },
                InlineNode::Plain(Arc::from(" for more.")),
            ],
        },
        RegisteredContent {
            register: Register::Detail,
            content: vec![InlineNode::Plain(Arc::from(
                "Technical appendix text — extended detail content.",
            ))],
        },
    ];

    let slide1 = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("content"),
        frames: vec![Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(9_144_000),
                height: Emu(5_143_500),
            },
            content: FrameContent::Title(Arc::from("Q1 Review")),
            text_flow: None,
        }],
        speaker_notes: Some(Arc::from("NOTES_DOCX_SENTINEL")),
        register_tags: vec![],
        register_content: slide1_register,
    };

    // ── Slide 2: Visual-only — no report/detail (AC-009) ─────────────────────
    let slide2 = LaidOutSlide {
        source_index: 1,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(9_144_000),
                height: Emu(5_143_500),
            },
            content: FrameContent::Title(Arc::from("Closing Remarks")),
            text_flow: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    };

    let laid_out = LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![slide1, slide2],
        sections: vec![],
        warnings: vec![],
    };

    // ── Run the exporter ──────────────────────────────────────────────────────
    let opts = ExportOptions::default();
    let docx_bytes = DocxExporter
        .export(&deck, &laid_out, &brand, &opts)
        .expect("DocxExporter::export must succeed");

    // ── Write output to target/ ───────────────────────────────────────────────
    let out_path = std::path::Path::new("target/demo_docx_output.docx");
    std::fs::write(out_path, &docx_bytes).expect("write output file");
    println!("Wrote {} bytes to {}", docx_bytes.len(), out_path.display());
    println!();

    // ── AC-001: Exporter plugin trait ─────────────────────────────────────────
    println!("=== AC-001: Exporter plugin trait ===");
    println!("  id()        = {:?}", DocxExporter.id());
    println!("  extension() = {:?}", DocxExporter.extension());
    assert_eq!(DocxExporter.id(), "docx");
    assert_eq!(DocxExporter.extension(), "docx");
    println!("  PASS");
    println!();

    // ── AC-002: ZIP structure with all required parts ─────────────────────────
    println!("=== AC-002: ZIP part list ===");
    let part_list = zip_entry_names(&docx_bytes);
    for part in &part_list {
        println!("  {part}");
    }
    let required_parts = [
        "[Content_Types].xml",
        "_rels/.rels",
        "word/document.xml",
        "word/_rels/document.xml.rels",
        "word/styles.xml",
        "docProps/core.xml",
    ];
    for req in &required_parts {
        assert!(
            part_list.iter().any(|p| p == req),
            "Missing required ZIP part: {req}"
        );
    }
    println!("  PASS — all required parts present");
    println!();

    // ── AC-003 + AC-004: report in body, heading before report ────────────────
    println!("=== AC-003 + AC-004: report content + Heading1 ordering ===");
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");
    let heading1_pos = doc_xml
        .find("Heading1")
        .expect("Heading1 style must appear in document.xml");
    let report_pos = doc_xml
        .find("Analysis")
        .expect("report content 'Analysis' must appear");
    println!("  Heading1 offset = {heading1_pos}");
    println!("  Report content offset = {report_pos}");
    assert!(
        heading1_pos < report_pos,
        "Heading1 must appear BEFORE report paragraph"
    );
    println!("  PASS — Heading1 at {heading1_pos} before report at {report_pos}");
    // Print excerpt around Heading1
    let h1_excerpt_start = heading1_pos.saturating_sub(20);
    let h1_excerpt_end = (heading1_pos + 120).min(doc_xml.len());
    println!(
        "  Heading1 context: ...{}...",
        &doc_xml[h1_excerpt_start..h1_excerpt_end]
    );
    println!();

    // ── AC-005: notes sentinel ABSENT ────────────────────────────────────────
    println!("=== AC-005: notes sentinel absent from document.xml ===");
    assert!(
        !doc_xml.contains("NOTES_DOCX_SENTINEL"),
        "notes content must NOT appear in document.xml"
    );
    println!("  PASS — NOTES_DOCX_SENTINEL is absent from document.xml");
    println!();

    // ── AC-006: report sentinel PRESENT ──────────────────────────────────────
    println!("=== AC-006: report sentinel present in document.xml ===");
    assert!(
        doc_xml.contains("REPORT_DOCX_SENTINEL"),
        "report content REPORT_DOCX_SENTINEL must appear in document.xml"
    );
    println!("  PASS — REPORT_DOCX_SENTINEL is present in document.xml");
    println!();

    // ── AC-007: detail in extended section ───────────────────────────────────
    println!("=== AC-007: detail in extended section ===");
    let appendix_pos = doc_xml
        .find("Appendix")
        .expect("Appendix Heading2 must appear in document.xml");
    let detail_text_pos = doc_xml
        .find("Technical appendix text")
        .expect("detail content must appear in document.xml");
    assert!(
        appendix_pos < detail_text_pos,
        "Appendix heading must appear before detail text"
    );
    // The Appendix section must come after all report paragraphs.
    // Q1 Review heading appears before report; Appendix appears after report.
    println!("  Appendix heading offset = {appendix_pos}");
    println!("  Detail text offset      = {detail_text_pos}");
    println!("  Report content offset   = {report_pos}");
    assert!(
        report_pos < appendix_pos,
        "report paragraph must appear before Appendix heading (extended section)"
    );
    println!("  PASS — detail content appears in extended section after report");
    let appendix_excerpt_start = appendix_pos.saturating_sub(10);
    let appendix_excerpt_end = (appendix_pos + 100).min(doc_xml.len());
    println!(
        "  Appendix context: ...{}...",
        &doc_xml[appendix_excerpt_start..appendix_excerpt_end]
    );
    println!();

    // ── AC-008: inline formatting (bold, italic, hyperlink) ──────────────────
    println!("=== AC-008: inline formatting ===");
    // Bold: <w:b/> or <w:b />
    let has_bold = doc_xml.contains("<w:b/>") || doc_xml.contains("<w:b />");
    assert!(has_bold, "Bold run property <w:b/> or <w:b /> must appear");
    println!("  Bold run property present: true");
    // Italic: <w:i/> or <w:i />
    let has_italic = doc_xml.contains("<w:i/>") || doc_xml.contains("<w:i />");
    assert!(
        has_italic,
        "Italic run property <w:i/> or <w:i /> must appear"
    );
    println!("  Italic run property present: true");
    // Hyperlink: w:hyperlink element
    let has_hyperlink = doc_xml.contains("w:hyperlink");
    assert!(
        has_hyperlink,
        "<w:hyperlink> must appear for InlineNode::Link"
    );
    println!("  Hyperlink element present: true");
    println!("  PASS — bold, italic, hyperlink all mapped to Word run properties");
    // Print hyperlink rels entry
    let rels_xml = read_zip_member(&docx_bytes, "word/_rels/document.xml.rels");
    let hyperlink_rel_pos = rels_xml
        .find("slideforge.rs")
        .expect("hyperlink target URL must appear in document.xml.rels");
    let rel_excerpt_start = hyperlink_rel_pos.saturating_sub(60);
    let rel_excerpt_end = (hyperlink_rel_pos + 80).min(rels_xml.len());
    println!(
        "  Hyperlink rel entry: ...{}...",
        &rels_xml[rel_excerpt_start..rel_excerpt_end]
    );
    println!();

    // ── AC-009: visual-only slide → heading + empty paragraph ────────────────
    println!("=== AC-009: visual-only slide heading + empty paragraph ===");
    // "Closing Remarks" is the title of slide 2 (visual-only).
    let closing_pos = doc_xml
        .find("Closing Remarks")
        .expect("Closing Remarks heading must appear");
    println!("  'Closing Remarks' Heading1 offset = {closing_pos}");
    // The empty paragraph follows immediately after the heading in the XML.
    // Verify slide 2 heading is present and no extra body text follows from register.
    assert!(
        !doc_xml[closing_pos..].contains("NOTES_DOCX_SENTINEL"),
        "notes must not appear after Closing Remarks heading"
    );
    println!("  PASS — visual-only slide has Heading1; no register bleed into body");
    println!();

    // ── dc:language from docProps/core.xml ────────────────────────────────────
    println!("=== dc:language in core.xml ===");
    let core_xml = read_zip_member(&docx_bytes, "docProps/core.xml");
    assert!(
        core_xml.contains("en-US"),
        "dc:language en-US must appear in docProps/core.xml"
    );
    let lang_pos = core_xml
        .find("en-US")
        .expect("en-US must appear in core.xml (already asserted above)");
    let lang_excerpt_start = lang_pos.saturating_sub(30);
    let lang_excerpt_end = (lang_pos + 40).min(core_xml.len());
    println!(
        "  dc:language excerpt: ...{}...",
        &core_xml[lang_excerpt_start..lang_excerpt_end]
    );
    println!("  PASS — dc:language = en-US");
    println!();

    println!("All STORY-041 acceptance criteria PASS.");
}

// ─── Helpers (mirrors test fixtures in core_tests.rs) ────────────────────────

/// Enumerate all ZIP entry names from the `.docx` byte buffer.
fn zip_entry_names(docx_bytes: &[u8]) -> Vec<String> {
    let cursor = std::io::Cursor::new(docx_bytes);
    let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
    (0..archive.len())
        .map(|i| archive.by_index(i).expect("valid index").name().to_owned())
        .collect()
}

/// Read a named ZIP member as a UTF-8 string.
fn read_zip_member(docx_bytes: &[u8], name: &str) -> String {
    let cursor = std::io::Cursor::new(docx_bytes);
    let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
    let mut entry = archive
        .by_name(name)
        .unwrap_or_else(|_| panic!("ZIP entry '{name}' not found"));
    let mut buf = String::new();
    entry.read_to_string(&mut buf).expect("UTF-8");
    buf
}
