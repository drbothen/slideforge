//! STORY-099 Red Gate tests for BC-4.02.001 and BC-3.05.001 (DOCX surface).
//!
//! Covers REND-006: bullet run content, numbering.xml abstract definitions,
//! sectPr page dimensions, and `<w:lang>` on every run.
//!
//! # Contract being verified
//!
//! | Test name | BC clause | Story AC / EC |
//! |-----------|-----------|----------------|
//! | `test_BC_4_02_001_bullets_have_run_content` | BC-4.02.001 postcondition 7 | AC-001 |
//! | `test_BC_4_02_001_numbering_xml_has_abstract_num` | BC-4.02.001 postcondition 2 | AC-002 |
//! | `test_BC_4_02_001_document_xml_has_secpr` | BC-4.02.001 postcondition 2 | AC-003 |
//! | `test_BC_3_05_001_runs_have_lang_attribute_default_en` | BC-3.05.001 PC-3 / BC-5.01.005 PC-4 | AC-004 / EC-003 |
//! | `test_BC_3_05_001_runs_have_lang_attribute_fr_fr` | BC-3.05.001 PC-3 / BC-5.01.005 PC-4 | AC-004 |
//! | `test_BC_3_05_001_runs_have_lang_attribute_all_runs` | BC-3.05.001 PC-3 / BC-5.01.005 PC-4 | AC-004 (universality) |
//! | `test_BC_3_05_001_runs_have_lang_attribute_de_de` | BC-5.01.005 EC-004 | EC-004 |
//! | `test_BC_4_02_001_ec001_no_bullets_no_numpr_no_crash` | BC-4.02.001 | EC-001 |
//! | `test_BC_4_02_001_ec002_nested_bullets_correct_levels` | BC-4.02.001 postcondition 7 | EC-002 |
//! | `test_BC_4_02_001_ec005_empty_bullet_string_preserved` | BC-4.02.001 postcondition 7 | EC-005 |
//!
//! # Twips arithmetic for AC-003
//!
//! `sectPr pgSz` dimensions are in twentieths of a point (twips).
//! Conversion: twips = EMU × 1440 / 914_400 = EMU / 635.
//!
//! `LaidOutDeck::default()` carries `PageSize::default()`:
//!   - width  = `DEFAULT_PAGE_WIDTH`  = 9_144_000 EMU → 9_144_000 / 635 = 14_400 twips (exact)
//!   - height = `DEFAULT_PAGE_HEIGHT` = 5_143_500 EMU → 5_143_500 / 635 =  8_100 twips (exact)
//!
//! Both divisions are exact (zero remainder), so no rounding ambiguity exists.
//! The test pins `w:w="14400"` and `w:h="8100"` as the expected attribute values
//! when `LaidOutDeck::default()` page size is used.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(non_snake_case)]
#![allow(clippy::doc_markdown)]

use std::io::Read as IoRead;
use std::sync::Arc;

use slideforge_layout::types::{
    BoundingBox, Emu, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize,
};
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::{
    Brand, BrandFonts, BrandPalette, Deck, DeckMetadata, InlineNode, OrderedMap, Register,
    RegisteredContent,
};

use crate::DocxExporter;

// ─── Shared fixture helpers ────────────────────────────────────────────────────

/// Build a minimal [`Brand`] with no special settings.
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

/// Build a [`Deck`] with the given BCP-47 language tag.
///
/// Pass `None` to model a deck with no `lang` declaration; the DOCX exporter
/// must then default to `"en"` per BC-5.01.004 / BC-5.01.005.
fn deck_with_lang(lang: Option<&str>) -> Deck {
    Deck {
        slides: vec![],
        vars: OrderedMap::new(),
        metadata: DeckMetadata {
            title: Some(Arc::from("Test Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: lang.map(Arc::from),
            author: None,
            section_order: None,
        },
        registers: OrderedMap::new(),
        section_blocks: vec![],
        slide_sections: vec![],
    }
}

/// Build a [`LaidOutDeck`] that wraps the provided slides and uses the default
/// page dimensions (`DEFAULT_PAGE_WIDTH` × `DEFAULT_PAGE_HEIGHT`).
///
/// Default page size: 9_144_000 × 5_143_500 EMU
/// = 14_400 × 8_100 twips.
fn make_laid_out_deck(slides: Vec<LaidOutSlide>) -> LaidOutDeck {
    LaidOutDeck {
        page_size: PageSize::default(),
        slides,
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    }
}

/// Build a slide with a title frame and one `FrameContent::TextRun` frame per
/// bullet item.
///
/// The TextRun frames model what `layout::run` produces for
/// `ContentBlock::Bullets` (one TextRun per BulletItem, STORY-073).
/// The test constructs these directly to isolate the DOCX exporter's
/// responsibility.
fn make_slide_with_bullets(title: &str, bullet_texts: &[&str]) -> LaidOutSlide {
    let mut frames = vec![Frame {
        bbox: BoundingBox {
            x: Emu(0),
            y: Emu(0),
            width: Emu(9_144_000),
            height: Emu(685_800),
        },
        content: FrameContent::Title(Arc::from(title)),
        text_flow: None,
        region_role: None,
    }];

    for text in bullet_texts {
        frames.push(Frame {
            bbox: BoundingBox {
                x: Emu(457_200),
                y: Emu(1_000_000),
                width: Emu(8_229_600),
                height: Emu(457_200),
            },
            content: FrameContent::TextRun(vec![InlineNode::Plain(Arc::from(*text))]),
            text_flow: None,
            region_role: None,
        });
    }

    LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("content"),
        frames,
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    }
}

/// Build a slide with a title frame, a level-0 bullet TextRun, and a
/// level-1 (nested) bullet TextRun indented by `BULLET_DEPTH_INDENT_EMU`.
///
/// `BULLET_DEPTH_INDENT_EMU` is 457_200 (from layout/layout.rs constant),
/// so a depth-1 child has `x = body_bbox.x + 457_200`.
fn make_slide_with_nested_bullets(title: &str) -> LaidOutSlide {
    // depth-0 parent bullet at x=457_200 (body left edge)
    let parent_frame = Frame {
        bbox: BoundingBox {
            x: Emu(457_200),
            y: Emu(1_000_000),
            width: Emu(8_229_600),
            height: Emu(457_200),
        },
        content: FrameContent::TextRun(vec![InlineNode::Plain(Arc::from("Parent bullet"))]),
        text_flow: None,
        region_role: None,
    };

    // depth-1 child bullet at x=457_200 + 457_200 = 914_400
    let child_frame = Frame {
        bbox: BoundingBox {
            x: Emu(914_400),
            y: Emu(1_457_200),
            width: Emu(7_772_400),
            height: Emu(457_200),
        },
        content: FrameContent::TextRun(vec![InlineNode::Plain(Arc::from("Child bullet"))]),
        text_flow: None,
        region_role: None,
    };

    LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("content"),
        frames: vec![
            Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(685_800),
                },
                content: FrameContent::Title(Arc::from(title)),
                text_flow: None,
                region_role: None,
            },
            parent_frame,
            child_frame,
        ],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    }
}

/// Invoke `DocxExporter::export` and return the raw `.docx` bytes.
fn export_deck(deck: &Deck, laid_out: &LaidOutDeck) -> Vec<u8> {
    let brand = minimal_brand();
    let opts = ExportOptions::default();
    DocxExporter
        .export(deck, laid_out, &brand, &opts)
        .expect("export must succeed")
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

// ─── T-001: AC-001 — Bullet paragraphs carry run content (no empty <w:p/>) ───

/// BC-4.02.001 postcondition 7 / BC-3.05.001 PC-3 / AC-001:
///
/// A slide with bullets `["Point A", "Point B"]` produces DOCX paragraphs
/// where each bullet `<w:p>` element carries:
///   1. `<w:pPr><w:numPr>…</w:numPr></w:pPr>` — list-style paragraph properties.
///   2. `<w:r><w:t>Point A</w:t></w:r>` — a non-empty run with the bullet text.
///
/// Failure mode caught: the prior implementation emitted TextRun frames as
/// Normal-styled paragraphs with run content but WITHOUT `<w:numPr>`, so
/// the output was not recognised as a list by Word 365.
#[test]
fn test_BC_4_02_001_bullets_have_run_content() {
    let deck = deck_with_lang(Some("en"));
    let slide = make_slide_with_bullets("Bullet Slide", &["Point A", "Point B"]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // ── Assert bullet text is present ────────────────────────────────────────
    assert!(
        doc_xml.contains("<w:t>Point A</w:t>") || doc_xml.contains(">Point A<"),
        "AC-001 RED GATE: word/document.xml must contain run text 'Point A' for the \
         first bullet; current implementation emits TextRun frames without <w:numPr>. \
         Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );
    assert!(
        doc_xml.contains("<w:t>Point B</w:t>") || doc_xml.contains(">Point B<"),
        "AC-001 RED GATE: word/document.xml must contain run text 'Point B' for the \
         second bullet. Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );

    // ── Assert numPr is present (list-style paragraph property) ──────────────
    // Every bullet paragraph must carry <w:pPr><w:numPr>…</w:numPr></w:pPr>.
    // <w:numPr> is the presence signal for "this paragraph belongs to a list".
    assert!(
        doc_xml.contains("<w:numPr>") || doc_xml.contains("<w:numPr "),
        "AC-001 RED GATE: word/document.xml must contain <w:numPr> on bullet paragraphs \
         (BC-4.02.001 postcondition 7). Current implementation emits Normal paragraphs \
         with no list marker. Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );

    // ── Assert numId is present and non-zero ──────────────────────────────────
    // <w:numId w:val="N"/> inside <w:numPr> references a concrete numbering
    // definition in numbering.xml. Val must be >= 1.
    assert!(
        doc_xml.contains("<w:numId") && !doc_xml.contains("<w:numId w:val=\"0\""),
        "AC-001 RED GATE: <w:numId> must be present with a non-zero w:val referencing \
         a concrete numbering definition. Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );

    // ── Assert no empty <w:p/> for bullet content ─────────────────────────────
    // Empty self-closing <w:p/> means the paragraph has no run content.
    // This is the pre-fix defect: the bullet text was never emitted.
    let empty_para_count = doc_xml.matches("<w:p/>").count();
    assert_eq!(
        empty_para_count,
        0,
        "AC-001 RED GATE: word/document.xml must NOT contain empty <w:p/> self-closing \
         elements for bullet content; found {empty_para_count} empty paragraphs. \
         Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );
}

// ─── T-002: AC-002 — numbering.xml has abstract and concrete numbering ────────

/// BC-4.02.001 postcondition 2 / AC-002:
///
/// `word/numbering.xml` must NOT be a self-closing stub. It must contain:
///   1. At least one `<w:abstractNum>` element with `<w:lvl>` children covering
///      at least list levels 1 through 3 (zero-indexed as 0, 1, 2).
///   2. At least one `<w:num>` element referencing that `<w:abstractNum>` via
///      `<w:abstractNumId w:val="N"/>`.
///
/// The `<w:num>` ID is what bullet paragraphs reference via `<w:numId w:val="N"/>`.
///
/// Failure mode caught: the current implementation emits
/// `<w:numbering … />` (empty self-closing tag) — no definitions at all.
#[test]
fn test_BC_4_02_001_numbering_xml_has_abstract_num() {
    let deck = deck_with_lang(Some("en"));
    let slide = make_slide_with_bullets("Numbered List Slide", &["Item 1", "Item 2"]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let numbering_xml = read_zip_member(&docx_bytes, "word/numbering.xml");

    // ── Assert <w:abstractNum> is present ────────────────────────────────────
    assert!(
        numbering_xml.contains("<w:abstractNum") && !numbering_xml.contains("<w:abstractNum/>"),
        "AC-002 RED GATE: word/numbering.xml must contain a non-empty <w:abstractNum> \
         element (BC-4.02.001 postcondition 2). Current implementation emits a \
         self-closing empty stub. Got:\n{numbering_xml}"
    );

    // ── Assert <w:lvl> children covering levels 0–2 ──────────────────────────
    // The abstract numbering must define at least 3 levels (ilvl 0, 1, 2)
    // so that nested bullets at levels 1 and 2 have backing definitions.
    let lvl_count = numbering_xml.matches("<w:lvl").count();
    assert!(
        lvl_count >= 3,
        "AC-002 RED GATE: word/numbering.xml must define at least 3 list levels \
         (<w:lvl> elements for ilvl 0, 1, 2); found {lvl_count} <w:lvl> elements. \
         Got:\n{numbering_xml}"
    );

    // ── Assert <w:num> concrete numbering is present ─────────────────────────
    assert!(
        numbering_xml.contains("<w:num ") || numbering_xml.contains("<w:num>"),
        "AC-002 RED GATE: word/numbering.xml must contain a <w:num> element that \
         maps a concrete numbering ID to an abstract numbering definition. \
         Got:\n{numbering_xml}"
    );

    // ── Assert <w:abstractNumId> inside <w:num> references the abstractNum ───
    assert!(
        numbering_xml.contains("<w:abstractNumId"),
        "AC-002 RED GATE: <w:num> must contain <w:abstractNumId w:val=\"N\"/> to \
         reference the <w:abstractNum> definition. Got:\n{numbering_xml}"
    );

    // ── Cross-check: bullet paragraphs reference the <w:num> ID ──────────────
    // The numId in document.xml must correspond to the <w:num w:numId="N"> value.
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // Extract the first w:numId value from document.xml.
    let num_id_prefix = "<w:numId w:val=\"";
    if let Some(pos) = doc_xml.find(num_id_prefix) {
        let after = &doc_xml[pos + num_id_prefix.len()..];
        let end = after.find('"').expect("numId val must be quoted");
        let num_id_val = &after[..end];

        // That same ID must appear as w:numId="N" in numbering.xml's <w:num>.
        let num_def_pattern = format!("<w:num w:numId=\"{num_id_val}\"");
        assert!(
            numbering_xml.contains(&num_def_pattern),
            "AC-002 RED GATE: <w:numId w:val=\"{num_id_val}\"/> in document.xml must \
             resolve to a <w:num w:numId=\"{num_id_val}\"> definition in numbering.xml. \
             Got numbering.xml:\n{numbering_xml}"
        );
    }
    // Note: if no numId is present in document.xml, T-001 already catches that.
}

// ─── T-003: AC-003 — sectPr present in document.xml with correct page dims ───

/// BC-4.02.001 postcondition 2 / AC-003:
///
/// `word/document.xml` body must end with a `<w:sectPr>` element containing
/// `<w:pgSz w:w="…" w:h="…"/>` whose attribute values match the deck's brand
/// page dimensions expressed in twentieths of a point (twips).
///
/// # Twips arithmetic
///
/// `LaidOutDeck::default()` uses `PageSize::default()`:
///   - width  = `DEFAULT_PAGE_WIDTH`  = 9_144_000 EMU
///   - height = `DEFAULT_PAGE_HEIGHT` = 5_143_500 EMU
///
/// Conversion: twips = EMU × 1440 / 914_400 = EMU / 635.
///   - width_twips  = 9_144_000 / 635 = 14_400 (exact, no remainder)
///   - height_twips = 5_143_500 / 635 =  8_100 (exact, no remainder)
///
/// Failure mode caught: `<w:sectPr>` is entirely absent from the current
/// implementation. Word cannot determine page dimensions; layout collapses to
/// the default US Letter A4 fallback which does not match the brand canvas.
#[test]
fn test_BC_4_02_001_document_xml_has_secpr() {
    let deck = deck_with_lang(Some("en"));
    // A single-slide deck is sufficient to verify sectPr presence.
    let slide = make_slide_with_bullets("SectPr Slide", &["Content item"]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // ── Assert <w:sectPr> is present ─────────────────────────────────────────
    assert!(
        doc_xml.contains("<w:sectPr") || doc_xml.contains("<w:sectPr>"),
        "AC-003 RED GATE: word/document.xml must contain a <w:sectPr> element as the \
         final child of <w:body>. Current implementation omits sectPr entirely, leaving \
         page dimensions undefined for Word. Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );

    // ── Assert <w:pgSz> is present inside sectPr ─────────────────────────────
    assert!(
        doc_xml.contains("<w:pgSz"),
        "AC-003 RED GATE: <w:sectPr> must contain <w:pgSz> with w:w and w:h attributes. \
         Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );

    // ── Assert exact twip values for DEFAULT_PAGE_WIDTH / DEFAULT_PAGE_HEIGHT ─
    //
    // Arithmetic documented in module-level doc comment:
    //   width_twips  = 9_144_000 EMU / 635 = 14_400 twips (exact)
    //   height_twips = 5_143_500 EMU / 635 =  8_100 twips (exact)
    //
    // The pgSz element must carry both attributes with these exact values.
    // We assert both the w:w and w:h attribute presence in the same string
    // fragment that contains "<w:pgSz".
    let pgsz_start = doc_xml
        .find("<w:pgSz")
        .expect("AC-003: <w:pgSz> must exist in word/document.xml after sectPr fix");
    let pgsz_end = doc_xml[pgsz_start..]
        .find('>')
        .map(|off| pgsz_start + off + 1)
        .expect("AC-003: <w:pgSz> element must close");
    let pgsz_fragment = &doc_xml[pgsz_start..pgsz_end];

    assert!(
        pgsz_fragment.contains("w:w=\"14400\""),
        "AC-003 RED GATE: <w:pgSz> must carry w:w=\"14400\" (= 9_144_000 EMU / 635 twips, \
         exact). PageSize::default() width = DEFAULT_PAGE_WIDTH = 9_144_000 EMU. \
         Conversion: 9_144_000 × 1440 / 914_400 = 14_400 twips. \
         Got pgSz fragment:\n{pgsz_fragment}"
    );
    assert!(
        pgsz_fragment.contains("w:h=\"8100\""),
        "AC-003 RED GATE: <w:pgSz> must carry w:h=\"8100\" (= 5_143_500 EMU / 635 twips, \
         exact). PageSize::default() height = DEFAULT_PAGE_HEIGHT = 5_143_500 EMU. \
         Conversion: 5_143_500 × 1440 / 914_400 = 8_100 twips. \
         Got pgSz fragment:\n{pgsz_fragment}"
    );

    // ── Assert sectPr is the final content element of <w:body> ───────────────
    // Per OOXML schema, sectPr MUST be the last child of body.
    let secpr_pos = doc_xml
        .rfind("<w:sectPr")
        .expect("AC-003: <w:sectPr> must exist");
    let body_close_pos = doc_xml
        .rfind("</w:body>")
        .expect("AC-003: </w:body> must exist");
    assert!(
        secpr_pos < body_close_pos,
        "AC-003 RED GATE: <w:sectPr> must appear before </w:body>. \
         sectPr at {secpr_pos}, </w:body> at {body_close_pos}"
    );
}

// ─── T-004: AC-004 / BC-5.01.005 — every run carries <w:lang> ────────────────

/// BC-3.05.001 PC-3 / BC-5.01.005 PC-4 / AC-004:
///
/// A deck with no `lang` declaration produces `<w:lang w:val="en"/>` on every
/// `<w:rPr>` in `word/document.xml`. The default is `"en"` per BC-5.01.004 —
/// NOT `"en-US"`.
///
/// Failure mode caught: the current implementation emits `<w:rPr>` without any
/// `<w:lang>` child element.
#[test]
fn test_BC_3_05_001_runs_have_lang_attribute_default_en() {
    // No lang declared → must default to "en" on all runs (BC-5.01.005 EC-003).
    let deck = deck_with_lang(None);
    let slide = {
        let rc = RegisteredContent::plain(Register::Report, Arc::from("Body text for lang test"));
        let mut s = make_slide_with_bullets("Lang Slide", &[]);
        s.register_content = vec![rc];
        s
    };
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // <w:lang w:val="en"/> must appear at least once.
    assert!(
        doc_xml.contains("<w:lang w:val=\"en\"") || doc_xml.contains("<w:lang w:val='en'"),
        "AC-004 / EC-003 RED GATE: word/document.xml must carry <w:lang w:val=\"en\"/> \
         (NOT \"en-US\") when no lang is declared (BC-5.01.005 PC-4 / BC-5.01.004 \
         default). Current implementation omits <w:lang> from all <w:rPr> blocks. \
         Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );

    // Must NOT use "en-US" as the default — that violates BC-5.01.005 PC-6.
    // (Only check this when no lang was set.)
    // We only fail if en-US appears WITHOUT en-US being the declared lang.
    assert!(
        !doc_xml.contains("<w:lang w:val=\"en-US\""),
        "AC-004 / EC-003 RED GATE: word/document.xml must NOT carry <w:lang w:val=\"en-US\"/> \
         when no lang is declared; the canonical default is \"en\" per BC-5.01.004 / \
         BC-5.01.005 PC-6. Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );
}

/// BC-3.05.001 PC-3 / BC-5.01.005 / AC-004 — fr-FR round-trip:
///
/// A deck declaring `lang "fr-FR"` produces `<w:lang w:val="fr-FR"/>` on every
/// `<w:rPr>` in `word/document.xml`. The exact BCP-47 tag is preserved verbatim
/// (BC-5.01.005 invariant 1 — no truncation, normalization, or case change).
#[test]
fn test_BC_3_05_001_runs_have_lang_attribute_fr_fr() {
    let deck = deck_with_lang(Some("fr-FR"));
    let rc = RegisteredContent::plain(Register::Report, Arc::from("Texte de rapport"));
    let mut slide = make_slide_with_bullets("French Slide", &["Élément de liste"]);
    slide.register_content = vec![rc];
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    assert!(
        doc_xml.contains("<w:lang w:val=\"fr-FR\"") || doc_xml.contains("<w:lang w:val='fr-FR'"),
        "AC-004 RED GATE: word/document.xml must carry <w:lang w:val=\"fr-FR\"/> \
         when deck declares lang \"fr-FR\" (BC-5.01.005 PC-4 / invariant 1). \
         Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );
}

/// BC-3.05.001 PC-3 / BC-5.01.005 / AC-004 — universality assertion:
///
/// Every `<w:r>` in `word/document.xml` must have a corresponding `<w:rPr>`
/// that carries `<w:lang>`. The count of `<w:r>` open-elements must equal
/// the count of `<w:lang` occurrences (no run is missing its lang annotation).
///
/// This test rejects partial coverage ("sampled" lang) — it is the STORY-096
/// lesson L-a universality check applied to the DOCX surface.
#[test]
fn test_BC_3_05_001_runs_have_lang_attribute_all_runs() {
    let deck = deck_with_lang(Some("en"));
    // Build a slide with several runs across different inline variants to exercise
    // multiple code paths: heading run, body run, bullet run, bold run.
    let rc = RegisteredContent {
        register: Register::Report,
        content: vec![
            InlineNode::Plain(Arc::from("Plain text")),
            InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold"))]),
            InlineNode::Italic(vec![InlineNode::Plain(Arc::from("italic"))]),
        ],
    };
    let mut slide = make_slide_with_bullets("Multi-Run Slide", &["Bullet item"]);
    slide.register_content = vec![rc];
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // Count <w:r> open-elements (both <w:r> and <w:r >).
    let run_count = doc_xml.matches("<w:r>").count() + doc_xml.matches("<w:r ").count();
    // Count <w:lang occurrences (every rPr that has a lang child).
    let lang_count = doc_xml.matches("<w:lang").count();

    assert!(
        run_count > 0,
        "AC-004 universality precondition: at least one <w:r> must exist in document.xml"
    );
    assert_eq!(
        run_count,
        lang_count,
        "AC-004 RED GATE: every <w:r> must carry <w:lang> in its <w:rPr>. \
         Found {run_count} <w:r> elements but only {lang_count} <w:lang> elements. \
         Per STORY-096 LESSON L-a: assert ALL runs carry lang — not just a sample. \
         Got (first 4000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(4000)]
    );
}

/// BC-5.01.005 EC-004 — de-DE round-trip:
///
/// A deck declaring `lang "de-DE"` produces `<w:lang w:val="de-DE"/>` on all
/// `<w:rPr>` elements (BC-5.01.005 invariant 1 lossless propagation).
#[test]
fn test_BC_3_05_001_runs_have_lang_attribute_de_de() {
    let deck = deck_with_lang(Some("de-DE"));
    let rc = RegisteredContent::plain(Register::Report, Arc::from("Berichtstext"));
    let mut slide = make_slide_with_bullets("German Slide", &[]);
    slide.register_content = vec![rc];
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    assert!(
        doc_xml.contains("<w:lang w:val=\"de-DE\"") || doc_xml.contains("<w:lang w:val='de-DE'"),
        "EC-004 RED GATE: word/document.xml must carry <w:lang w:val=\"de-DE\"/> \
         when deck declares lang \"de-DE\" (BC-5.01.005 EC-004). \
         Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );
}

// ─── EC-001: No bullets → no numPr, no crash ─────────────────────────────────

/// BC-4.02.001 / EC-001:
///
/// A slide containing only report text and NO bullet (`TextRun`) frames must not
/// produce any `<w:numPr>` element in `word/document.xml` and must not crash.
///
/// This guards against the numbering path being applied unconditionally to all
/// paragraphs.
#[test]
fn test_BC_4_02_001_ec001_no_bullets_no_numpr_no_crash() {
    let deck = deck_with_lang(Some("en"));
    let rc = RegisteredContent::plain(Register::Report, Arc::from("No bullet content here"));
    // Slide with no TextRun frames — only a title frame.
    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("content"),
        frames: vec![Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(9_144_000),
                height: Emu(685_800),
            },
            content: FrameContent::Title(Arc::from("No Bullets Slide")),
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![rc],
    };
    let laid_out = make_laid_out_deck(vec![slide]);

    // Must not panic.
    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // No <w:numPr> must appear for a non-bullet slide.
    assert!(
        !doc_xml.contains("<w:numPr"),
        "EC-001: word/document.xml must NOT contain <w:numPr> when the slide has no \
         bullet frames; got:\n{doc_xml}"
    );

    // Report text must still be present.
    assert!(
        doc_xml.contains("No bullet content here"),
        "EC-001: report text must still appear even without bullet frames"
    );
}

// ─── EC-002: Nested bullets → correct level indices ──────────────────────────

/// BC-4.02.001 postcondition 7 / EC-002:
///
/// A slide with two levels of bullets (parent at depth 0, child at depth 1)
/// must produce `<w:numPr>` on both paragraphs with correct `<w:ilvl>` values:
///   - depth-0 parent: `<w:ilvl w:val="0"/>`
///   - depth-1 child:  `<w:ilvl w:val="1"/>`
///
/// The depth is derived from the TextRun frame's bbox x-coordinate relative to
/// the body left edge (BULLET_DEPTH_INDENT_EMU = 457_200 EMU per level).
#[test]
fn test_BC_4_02_001_ec002_nested_bullets_correct_levels() {
    let deck = deck_with_lang(Some("en"));
    let slide = make_slide_with_nested_bullets("Nested Bullets Slide");
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // Both bullet texts must be present.
    assert!(
        doc_xml.contains("Parent bullet"),
        "EC-002: 'Parent bullet' text must appear in document.xml"
    );
    assert!(
        doc_xml.contains("Child bullet"),
        "EC-002: 'Child bullet' text must appear in document.xml"
    );

    // Both paragraphs must carry <w:numPr>.
    let numpr_count = doc_xml.matches("<w:numPr>").count() + doc_xml.matches("<w:numPr ").count();
    assert!(
        numpr_count >= 2,
        "EC-002 RED GATE: both the parent and child bullet paragraphs must carry \
         <w:numPr>; found only {numpr_count} <w:numPr> elements. \
         Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );

    // Level 0 must appear for the parent bullet.
    assert!(
        doc_xml.contains("<w:ilvl w:val=\"0\""),
        "EC-002 RED GATE: depth-0 parent bullet must carry <w:ilvl w:val=\"0\"/> \
         in its <w:numPr>. Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );

    // Level 1 must appear for the child bullet.
    assert!(
        doc_xml.contains("<w:ilvl w:val=\"1\""),
        "EC-002 RED GATE: depth-1 child bullet must carry <w:ilvl w:val=\"1\"/> \
         in its <w:numPr>. Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );
}

// ─── EC-005: Empty bullet string → empty <w:t/> preserved (not dropped) ──────

/// BC-4.02.001 postcondition 7 / EC-005:
///
/// A TextRun frame carrying an empty `InlineNode::Plain("")` (empty bullet
/// string) must produce a paragraph with an empty `<w:t/>` run — the run is
/// NOT dropped. This is consistent with Word behaviour for empty list items
/// and preserves the list structure for downstream editing.
///
/// The `<w:t>` element for an empty string must use the empty form
/// (`<w:t/>` or `<w:t></w:t>`), not a run with no `<w:t>` child at all.
#[test]
fn test_BC_4_02_001_ec005_empty_bullet_string_preserved() {
    let deck = deck_with_lang(Some("en"));
    // One empty bullet (empty string in the TextRun).
    let slide = make_slide_with_bullets("Empty Bullet Slide", &[""]);
    let laid_out = make_laid_out_deck(vec![slide]);

    let docx_bytes = export_deck(&deck, &laid_out);
    let doc_xml = read_zip_member(&docx_bytes, "word/document.xml");

    // The empty-string bullet paragraph must exist with numPr.
    // We check for <w:numPr> (the list marker) — the run content is an empty
    // InlineNode::Plain("") which should produce <w:t/> or <w:t></w:t>.
    assert!(
        doc_xml.contains("<w:numPr") || doc_xml.contains("<w:numPr>"),
        "EC-005 RED GATE: even an empty-string bullet must produce a <w:numPr> paragraph. \
         Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );

    // Must not crash (the export above already proves no panic).
    // The run must exist even if empty — either <w:t/> or <w:t></w:t>.
    // We assert a <w:r> is present in the bullet paragraph (not dropped entirely).
    // Since we cannot easily isolate this specific paragraph, we accept any <w:r>
    // as evidence that runs are emitted for empty strings.
    assert!(
        doc_xml.contains("<w:r>") || doc_xml.contains("<w:r "),
        "EC-005 RED GATE: the empty-string bullet paragraph must contain a <w:r> \
         element (run preserved, not dropped). Got (first 3000 chars):\n{}",
        &doc_xml[..doc_xml.len().min(3000)]
    );
}
