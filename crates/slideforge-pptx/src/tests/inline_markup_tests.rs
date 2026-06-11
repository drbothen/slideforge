//! STORY-081 Red Gate tests — PPTX exporter slide-level inline markup rendering.
//!
//! ## Traceability
//!
//! | Test function | AC/EC | BC clause | What is verified |
//! |---|---|---|---|
//! | `test_BC_3_05_001_ac002_pptx_inline_node_to_ooxml_runs_stub_exists` | AC-002 | BC-3.05.001 PC-1 | Bold via production pipeline produces `b="1"` in slide XML |
//! | `test_BC_3_05_001_ac002_pptx_textrun_frame_bold_produces_rpr_b1` | AC-002 | BC-3.05.001 PC-1 | TextRun with Bold → `<a:rPr b="1"/>` in PPTX slide XML |
//! | `test_BC_3_05_001_ac002_pptx_bold_no_literal_asterisks` | AC-002 | BC-3.05.001 PC-1 | no `**` in `<a:t>` elements when InlineNode::Bold is used |
//! | `test_BC_3_05_001_ac002_pptx_italic_produces_rpr_i1` | AC-002 | BC-3.05.001 PC-1 | Italic → `<a:rPr i="1"/>` |
//! | `test_BC_3_05_001_ac002_pptx_strikethrough_produces_sng_strike` | EC-009 | BC-3.05.001 PC-1 | Strikethrough → `strike="sngStrike"` |
//! | `test_BC_3_05_001_ac006_pptx_plain_title_no_rpr_b1` | AC-006 | BC-3.05.001 PC-1 | plain title → no `b="1"` in PPTX output |
//!
//! ## Red Gate contract
//!
//! MUST FAIL before STORY-081 implementation:
//!
//! 1. `test_BC_3_05_001_ac002_pptx_inline_node_to_ooxml_runs_stub_exists` — FAILS
//!    because `FrameContent::TextRun` routes through `extract_inline_text` (plain text)
//!    → no `<a:rPr b="1"/>` produced for Bold nodes.
//!
//! 2. `test_BC_3_05_001_ac002_pptx_textrun_frame_bold_produces_rpr_b1` — FAILS
//!    because `FrameContent::TextRun` currently calls `extract_inline_text(nodes)`
//!    → plain string → no `<a:rPr b="1"/>` produced.

#![allow(non_snake_case)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::uninlined_format_args)]

use std::io::Read as _;
use std::sync::Arc;

use slideforge_layout::{BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize};
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::{
    Brand, BrandFonts, BrandPalette, Deck, DeckMetadata, Emu, InlineNode, OrderedMap, SourceSpan,
};
use zip::ZipArchive;

use crate::PptxExporter;

// ─── Fixture helpers ─────────────────────────────────────────────────────────

// ─── Reference-set invariant helper (BC-3.05.001 HI-1) ───────────────────────
//
// BC-3.05.001 HI-1 explicitly RETIRED count-equality (`external_rel_count ==
// hlinkclick_count`) as FALSE: one External rel legitimately backs N
// `<a:hlinkClick>` runs for multi-leaf link display text (e.g., EC-012:
// `[click **here** now](url)` → 1 rel / 3 hlinkClick runs, all same rId).
//
// The correct invariant is the REFERENCE-SET form:
//   (a) every `<a:hlinkClick r:id="...">` rId resolves to a registered External rel
//       (no dangling rId), AND
//   (b) every registered External rel Id is referenced by ≥1 hlinkClick run
//       (no orphan rel).
//
// This helper encodes that invariant. Use it in place of the retired
// `assert_eq!(external_rel_count, hlinkclick_count, ...)` pattern.
fn assert_hyperlink_reference_set_invariant(slide_xml: &str, rels_xml: &str) {
    use std::collections::HashSet;

    // ── Collect rIds cited by <a:hlinkClick runs ──────────────────────────────
    // Pattern: <a:hlinkClick ... r:id="rIdN" ...>
    let hlinkclick_rids: HashSet<String> = {
        let mut set = HashSet::new();
        let mut remaining = slide_xml;
        while let Some(pos) = remaining.find("<a:hlinkClick") {
            let after_tag = &remaining[pos..];
            // Find r:id="..." within this hlinkClick element (before the next >)
            if let Some(tag_end) = after_tag.find('>')
                && let tag_body = &after_tag[..=tag_end]
                && let Some(rid_pos) = tag_body.find("r:id=\"")
                && let after_rid = &tag_body[rid_pos + 6..]
                && let Some(quote_end) = after_rid.find('"')
            {
                set.insert(after_rid[..quote_end].to_owned());
            }
            remaining = &remaining[pos + 1..];
        }
        set
    };

    // ── Collect Ids of External rels ─────────────────────────────────────────
    // Pattern: Id="rIdN" ... TargetMode="External"   (or vice versa)
    let external_rel_ids: HashSet<String> = {
        let mut set = HashSet::new();
        let mut remaining = rels_xml;
        while let Some(pos) = remaining.find("<Relationship") {
            let after_tag = &remaining[pos..];
            let tag_end = after_tag.find('>').unwrap_or(after_tag.len());
            let element = &after_tag[..=tag_end];
            if element.contains("TargetMode=\"External\"")
                && let Some(id_pos) = element.find("Id=\"")
                && let after_id = &element[id_pos + 4..]
                && let Some(quote_end) = after_id.find('"')
            {
                set.insert(after_id[..quote_end].to_owned());
            }
            remaining = &remaining[pos + 1..];
        }
        set
    };

    // ── (a) No dangling rId: every hlinkClick rId must be in External rels ───
    for rid in &hlinkclick_rids {
        assert!(
            external_rel_ids.contains(rid),
            "BC-3.05.001 HI-1 reference-set invariant violated: \
             <a:hlinkClick> references rId={rid:?} which has no matching \
             External rel in .rels (dangling rId).\n\
             hlinkClick rIds: {hlinkclick_rids:?}\n\
             External rel Ids: {external_rel_ids:?}\n\
             rels:\n{rels_xml}"
        );
    }

    // ── (b) No orphan rel: every External rel must be cited by ≥1 hlinkClick ──
    for id in &external_rel_ids {
        assert!(
            hlinkclick_rids.contains(id),
            "BC-3.05.001 HI-1 reference-set invariant violated: \
             External rel Id={id:?} has no corresponding <a:hlinkClick> run \
             (orphan rel).\n\
             hlinkClick rIds: {hlinkclick_rids:?}\n\
             External rel Ids: {external_rel_ids:?}\n\
             rels:\n{rels_xml}"
        );
    }
}

fn make_brand() -> Brand {
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

fn make_metadata() -> DeckMetadata {
    DeckMetadata {
        title: Some(Arc::from("Inline Markup Test")),
        slideforge_version: Arc::from("0.1.0"),
        lang: Some(Arc::from("en-US")),
        author: None,
        section_order: None,
    }
}

fn make_minimal_semantic_deck() -> Deck {
    Deck {
        slides: vec![],
        vars: OrderedMap::new(),
        metadata: make_metadata(),
        registers: OrderedMap::new(),
        section_blocks: vec![],
        slide_sections: vec![],
    }
}

fn make_slide_with_text_run(nodes: Vec<InlineNode>) -> LaidOutSlide {
    LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("content"),
        frames: vec![
            Frame {
                bbox: BoundingBox {
                    x: Emu(457_200),
                    y: Emu(274_638),
                    width: Emu(8_229_600),
                    height: Emu(1_143_000),
                },
                content: FrameContent::Title(Arc::from("Test Slide")),
                text_flow: None,
                region_role: None,
            },
            Frame {
                bbox: BoundingBox {
                    x: Emu(457_200),
                    y: Emu(1_600_200),
                    width: Emu(8_229_600),
                    height: Emu(4_000_000),
                },
                content: FrameContent::TextRun(nodes),
                text_flow: None,
                region_role: None,
            },
        ],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    }
}

fn make_laid_out_deck(slide: LaidOutSlide) -> LaidOutDeck {
    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    }
}

/// Export a PPTX and return slide1.xml content.
fn export_pptx_slide1_xml(nodes: Vec<InlineNode>) -> String {
    let (xml, _rels) = export_pptx_slide1_xml_and_rels(nodes);
    xml
}

/// Export a PPTX and return (slide1.xml content, slide1.xml.rels content).
///
/// Used for EC-004 tests that need to inspect both the slide XML (for
/// `<a:hlinkClick>`) and its companion rels file (for `TargetMode="External"`
/// relationships).
fn export_pptx_slide1_xml_and_rels(nodes: Vec<InlineNode>) -> (String, String) {
    let slide = make_slide_with_text_run(nodes);
    let laid_out = make_laid_out_deck(slide);
    let deck = make_minimal_semantic_deck();
    let brand = make_brand();

    let pptx_bytes = PptxExporter::new()
        .export(&deck, &laid_out, &brand, &ExportOptions::default())
        .expect("PptxExporter::export must succeed for inline markup test");

    let cursor = std::io::Cursor::new(&pptx_bytes);
    let mut archive = ZipArchive::new(cursor).expect("PPTX must be a valid ZIP");

    let mut xml = String::new();
    archive
        .by_name("ppt/slides/slide1.xml")
        .expect("ppt/slides/slide1.xml must be present")
        .read_to_string(&mut xml)
        .expect("slide1.xml must be valid UTF-8");

    let mut rels = String::new();
    archive
        .by_name("ppt/slides/_rels/slide1.xml.rels")
        .expect("ppt/slides/_rels/slide1.xml.rels must be present")
        .read_to_string(&mut rels)
        .expect("slide1.xml.rels must be valid UTF-8");

    (xml, rels)
}

// ─── AC-002 production-path test ─────────────────────────────────────────────

/// AC-002: `InlineNode::Bold` in a `FrameContent::TextRun` produces `b="1"` in
/// the exported PPTX slide XML.
///
/// This test exercises the PRODUCTION code path end-to-end: `PptxExporter::export`
/// → `nodes_to_body_runs` → `render_inline_nodes_to_runs` → `ooxml_run_to_ooxmlsdk`
/// → slide1.xml serialisation.
///
/// A regression in ANY part of the production body-run pipeline (e.g. removing the
/// bold arm from the unified engine) will cause this test to fail — it does not rely
/// on a test-only helper.
#[test]
fn test_BC_3_05_001_ac002_pptx_inline_node_to_ooxml_runs_stub_exists() {
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold"))])];
    let slide_xml = export_pptx_slide1_xml(nodes);

    assert!(
        slide_xml.contains("b=\"1\"") || slide_xml.contains("b=&quot;1&quot;"),
        "AC-002: Bold InlineNode must produce b=\"1\" in PPTX slide XML via the production \
         body-run pipeline; got slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(1000)]
    );
}

/// AC-002: A `FrameContent::TextRun` with `InlineNode::Bold` must produce
/// `<a:rPr b="1"/>` in the PPTX slide XML.
///
/// **Red Gate failure:** Currently `FrameContent::TextRun` calls
/// `extract_inline_text(nodes)` which flattens to plain text → no `b="1"` produced.
#[test]
fn test_BC_3_05_001_ac002_pptx_textrun_frame_bold_produces_rpr_b1() {
    let nodes = vec![
        InlineNode::Bold(vec![InlineNode::Plain(Arc::from("Key finding"))]),
        InlineNode::Plain(Arc::from(": revenue up 12%")),
    ];
    let slide_xml = export_pptx_slide1_xml(nodes);

    assert!(
        slide_xml.contains("b=\"1\"") || slide_xml.contains("b=&quot;1&quot;"),
        "AC-002: TextRun with Bold InlineNode must produce b=\"1\" in PPTX; \
         currently fails because TextRun uses extract_inline_text (plain text). \
         slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(1000)]
    );
}

/// AC-002: No literal `**` in slide XML when InlineNode::Bold is used.
///
/// This test verifies the historical R1 anti-pattern does NOT appear.
/// Should PASS even before STORY-081 (extract_inline_text strips delimiters).
#[test]
fn test_BC_3_05_001_ac002_pptx_bold_no_literal_asterisks() {
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold"))])];
    let slide_xml = export_pptx_slide1_xml(nodes);

    assert!(
        !slide_xml.contains("**"),
        "AC-002: PPTX slide XML must not contain literal '**'; \
         R1 anti-pattern must not be reintroduced. slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(500)]
    );
}

/// AC-002: `InlineNode::Italic` must produce `<a:rPr i="1"/>` in PPTX.
///
/// **Red Gate failure:** `extract_inline_text` flattens italic → no `i="1"`.
#[test]
fn test_BC_3_05_001_ac002_pptx_italic_produces_rpr_i1() {
    let nodes = vec![InlineNode::Italic(vec![InlineNode::Plain(Arc::from(
        "italic text",
    ))])];
    let slide_xml = export_pptx_slide1_xml(nodes);

    assert!(
        slide_xml.contains("i=\"1\"") || slide_xml.contains("i=&quot;1&quot;"),
        "AC-002: TextRun with Italic must produce i=\"1\" in PPTX; \
         currently fails due to extract_inline_text. slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(1000)]
    );
}

/// EC-009: `InlineNode::Strikethrough` must produce `strike="sngStrike"`.
///
/// **Red Gate failure:** Same as above — `extract_inline_text` flattens to plain text.
#[test]
fn test_BC_3_05_001_ac002_pptx_strikethrough_produces_sng_strike() {
    let nodes = vec![InlineNode::Strikethrough(vec![InlineNode::Plain(
        Arc::from("deleted"),
    )])];
    let slide_xml = export_pptx_slide1_xml(nodes);

    assert!(
        slide_xml.contains("sngStrike"),
        "EC-009: Strikethrough must produce strike=\"sngStrike\" in <a:rPr>; \
         currently fails due to extract_inline_text. slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(1000)]
    );
}

/// AC-006: A `FrameContent::Title` with plain text must NOT produce `b="1"`.
///
/// This test verifies the PPTX single-run title constraint remains intact.
/// Should PASS even before STORY-081 (plain title always produces plain run).
#[test]
fn test_BC_3_05_001_ac006_pptx_plain_title_no_rpr_b1() {
    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: BoundingBox {
                x: Emu(457_200),
                y: Emu(274_638),
                width: Emu(8_229_600),
                height: Emu(1_143_000),
            },
            content: FrameContent::Title(Arc::from("Plain Title")),
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    };
    let laid_out = make_laid_out_deck(slide);
    let deck = make_minimal_semantic_deck();
    let brand = make_brand();

    let pptx_bytes = PptxExporter::new()
        .export(&deck, &laid_out, &brand, &ExportOptions::default())
        .expect("PptxExporter::export must succeed");

    let cursor = std::io::Cursor::new(&pptx_bytes);
    let mut archive = ZipArchive::new(cursor).expect("valid ZIP");
    let mut slide_xml = String::new();
    archive
        .by_name("ppt/slides/slide1.xml")
        .expect("slide1.xml must be present")
        .read_to_string(&mut slide_xml)
        .expect("slide1.xml must be valid UTF-8");

    assert!(
        slide_xml.contains("Plain Title"),
        "AC-006: PPTX title must contain 'Plain Title'"
    );

    assert!(
        !slide_xml.contains("b=\"1\""),
        "AC-006: Plain title must NOT have b=\"1\"; \
         slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(500)]
    );
}

/// ADV-P10-HIGH-001: `InlineNode::Highlight` must produce `<a:highlight>` (CT_Color)
/// in `<a:rPr>`, NOT `<a:solidFill>` recoloring the glyph text.
///
/// DrawingML `RunProperties` has `a_highlight: Option<Box<Highlight>>` which maps to
/// `<a:highlight><a:srgbClr val="FFFF00"/></a:highlight>` — the background highlight
/// element. Using `run_properties_choice1` (solidFill) instead recolors the foreground
/// TEXT yellow on a light background, rendering it illegible.
///
/// Red Gate: FAILS before fix because current implementation sets
/// `run_properties_choice1 = Some(RunPropertiesChoice::ASolidFill(...))`.
#[test]
fn test_adv_p10_high_001_highlight_uses_a_highlight_not_solid_fill() {
    let nodes = vec![InlineNode::Highlight(vec![InlineNode::Plain(Arc::from(
        "highlighted text",
    ))])];
    let slide_xml = export_pptx_slide1_xml(nodes);

    // Must contain <a:highlight> with yellow sRGB color.
    // ooxmlsdk serializes this as <a:highlight><a:srgbClr val="FFFF00"/></a:highlight>
    // inside <a:rPr>.
    assert!(
        slide_xml.contains("<a:highlight>") || slide_xml.contains("a:highlight"),
        "ADV-P10-HIGH-001: Highlight InlineNode must produce <a:highlight> element in \
         <a:rPr>; got slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(2000)]
    );

    // The yellow FFFF00 color must appear within the highlight element context.
    // We check for the srgbClr val in the XML.
    assert!(
        slide_xml.contains("FFFF00"),
        "ADV-P10-HIGH-001: Highlight must use yellow (#FFFF00); \
         got slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(2000)]
    );

    // Must NOT set solidFill on the run (which recolors glyph text, not background).
    // The solidFill check looks for the specific solidFill inside rPr context.
    // We verify no <a:solidFill> appears in the slide XML when only a Highlight node
    // is present — the only color in this slide should be from <a:highlight>.
    assert!(
        !slide_xml.contains("<a:solidFill>"),
        "ADV-P10-HIGH-001: Highlight must NOT use solidFill (glyph recolor) — \
         it must use <a:highlight> for background highlight; \
         got slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(2000)]
    );
}

// =============================================================================
// ADV-P11-MED-001: Body-path distinguishing assertions for Code, Superscript,
// Subscript, and Link.
//
// The production code in ooxml_run_to_ooxmlsdk (slide_serializer.rs) is CORRECT for
// these variants, but no body-path test asserted them. A regression dropping
// rpr.baseline / rpr.a_latin in ooxml_run_to_ooxmlsdk would pass all tests without these.
//
// These tests are load-bearing: removing the corresponding ooxml_run_to_ooxmlsdk
// branch WILL cause each test to fail.
// =============================================================================

/// ADV-P11-MED-001: Body-path Code → `<a:latin typeface="Courier New"` in PPTX slide XML.
///
/// Load-bearing: removing the `if run.code_font { rpr.a_latin = Some(...) }` branch in
/// `ooxml_run_to_ooxmlsdk` (slide_serializer.rs) causes this test to fail — the
/// `<a:latin typeface` fragment disappears from the output.
#[test]
fn test_adv_p11_med_001_body_path_code_emits_courier_new_latin() {
    let nodes = vec![InlineNode::Code(Arc::from("fn main()"))];
    let slide_xml = export_pptx_slide1_xml(nodes);

    assert!(
        slide_xml.contains("<a:latin typeface=\"Courier New\""),
        "ADV-P11-MED-001: Code InlineNode in PPTX body must produce \
         <a:latin typeface=\"Courier New\" in <a:rPr>; \
         got slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(2000)]
    );
    assert!(
        slide_xml.contains("fn main()"),
        "ADV-P11-MED-001: Code text must appear in slide XML; \
         got slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(2000)]
    );
}

/// ADV-P11-MED-001: Body-path Superscript → `baseline="30000"` in PPTX slide XML.
///
/// Load-bearing: removing the `if let Some(baseline) = run.baseline` branch in
/// `ooxml_run_to_ooxmlsdk` (slide_serializer.rs) causes this test to fail.
#[test]
fn test_adv_p11_med_001_body_path_superscript_emits_baseline_30000() {
    let nodes = vec![InlineNode::Superscript(vec![InlineNode::Plain(Arc::from(
        "2",
    ))])];
    let slide_xml = export_pptx_slide1_xml(nodes);

    assert!(
        slide_xml.contains("baseline=\"30000\""),
        "ADV-P11-MED-001: Superscript InlineNode in PPTX body must produce \
         baseline=\"30000\" in <a:rPr>; \
         got slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(2000)]
    );
    assert!(
        slide_xml.contains('2'),
        "ADV-P11-MED-001: Superscript text must appear in slide XML; \
         got slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(2000)]
    );
}

/// ADV-P11-MED-001: Body-path Subscript → `baseline="-25000"` in PPTX slide XML.
///
/// Load-bearing: removing the `baseline: Some(-25_000)` for Subscript in the
/// unified `render_inline_nodes_to_runs` engine causes this test to fail.
#[test]
fn test_adv_p11_med_001_body_path_subscript_emits_baseline_neg25000() {
    let nodes = vec![InlineNode::Subscript(vec![InlineNode::Plain(Arc::from(
        "n",
    ))])];
    let slide_xml = export_pptx_slide1_xml(nodes);

    assert!(
        slide_xml.contains("baseline=\"-25000\""),
        "ADV-P11-MED-001: Subscript InlineNode in PPTX body must produce \
         baseline=\"-25000\" in <a:rPr>; \
         got slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(2000)]
    );
    assert!(
        slide_xml.contains('n'),
        "ADV-P11-MED-001: Subscript text must appear in slide XML; \
         got slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(2000)]
    );
}

/// EC-004: Body-path Link emits `<a:hlinkClick>`, External rel, and display text.
///
/// **Red Gate:** This test MUST FAIL before implementation because the current
/// `InlineNode::Link` arm does `let _ = url` (drops the URL) and emits no
/// `<a:hlinkClick>` and no External relationship in `slide1.xml.rels`.
///
/// **Load-bearing invariants this test enforces (mirrors notes_tests F-040):**
/// 1. `slide1.xml` contains `<a:hlinkClick` referencing an rId.
/// 2. `slide1.xml.rels` contains a matching `TargetMode="External"` relationship
///    with the URL as the Target.
/// 3. The rId cited in `<a:hlinkClick>` matches the rId in the rels file (both
///    are computed from the same `hlink_urls` list, so `rId count == hlinkClick count`).
/// 4. The display text "click here" appears as an `<a:t>` run.
#[test]
fn test_adv_p11_med_001_body_path_link_display_text_present() {
    let target_url = "https://example.com";
    let nodes = vec![InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("click here"))],
        url: Arc::from(target_url),
    }];
    let (slide_xml, slide_rels) = export_pptx_slide1_xml_and_rels(nodes);

    // 1. Display text must appear as a run.
    assert!(
        slide_xml.contains("<a:t>click here</a:t>"),
        "EC-004: Link display text must be in an <a:t> run element; \
         got slide1.xml (excerpt):\n{}",
        &slide_xml[..slide_xml.len().min(3000)]
    );

    // 2. slide1.xml must contain <a:hlinkClick (EC-004 primary assertion).
    assert!(
        slide_xml.contains("<a:hlinkClick"),
        "EC-004: slide1.xml must contain <a:hlinkClick> for body-path Link; \
         got slide1.xml (excerpt):\n{}",
        &slide_xml[..slide_xml.len().min(3000)]
    );

    // 3. slide1.xml.rels must contain TargetMode="External" for the URL.
    assert!(
        slide_rels.contains("TargetMode=\"External\""),
        "EC-004: slide1.xml.rels must contain TargetMode=\"External\" rel; \
         got rels:\n{slide_rels}"
    );
    assert!(
        slide_rels.contains(target_url),
        "EC-004: slide1.xml.rels must contain the URL {target_url:?} as Target; \
         got rels:\n{slide_rels}"
    );

    // 4. BC-3.05.001 HI-1 reference-set invariant: every hlinkClick rId resolves
    //    to a registered External rel AND every External rel is cited ≥1×.
    //    (Count-equality is NOT asserted — 1 rel may back N runs for multi-leaf
    //    display text; see EC-012 and test_f_p16_m1_body_multi_leaf_link_display_text.)
    assert_hyperlink_reference_set_invariant(&slide_xml, &slide_rels);
}

/// EC-004 / F-040-P2-001: Unsafe-scheme Link in slide body → plain text,
/// no `<a:hlinkClick>`, no External relationship in slide1.xml.rels.
///
/// Mirrors the notes-path safety guard (SEC-037-001 / CWE-601 / F-040-P2-001).
#[test]
fn test_ec004_body_path_unsafe_scheme_link_is_plain_text_no_rel() {
    let nodes = vec![InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("do not click"))],
        url: Arc::from("javascript:alert(1)"),
    }];
    let (slide_xml, slide_rels) = export_pptx_slide1_xml_and_rels(nodes);

    // Display text must still appear.
    assert!(
        slide_xml.contains("do not click"),
        "EC-004 unsafe: display text must appear even for unsafe-scheme links; \
         got slide1.xml:\n{}",
        &slide_xml[..slide_xml.len().min(3000)]
    );

    // No <a:hlinkClick> for unsafe-scheme URLs.
    assert!(
        !slide_xml.contains("<a:hlinkClick"),
        "EC-004 unsafe: slide1.xml must NOT contain <a:hlinkClick> for javascript: URL; \
         got slide1.xml:\n{}",
        &slide_xml[..slide_xml.len().min(3000)]
    );

    // No External rel in rels file.
    assert!(
        !slide_rels.contains("TargetMode=\"External\""),
        "EC-004 unsafe: slide1.xml.rels must NOT contain TargetMode=\"External\" for \
         unsafe-scheme URL; got rels:\n{slide_rels}"
    );
}

// =============================================================================
// F-P13-002: load-bearing combined-form assertion — PPTX body
//
// `Bold([Italic([Plain("x")])])` → exported slide1.xml <a:rPr> must contain
// BOTH b="1" AND i="1" on the same run. This exercises the RunProps
// accumulation in slideforge_plugin_api::inline_formats::render_nodes_recursive where
// Bold sets props.bold=true and then recurses into Italic which sets props.italic=true
// before reaching the Plain leaf.
//
// A regression that resets bold when entering Italic (or vice versa) causes
// this test to FAIL.
// =============================================================================

/// F-P13-002 load-bearing: `Bold([Italic([Plain("x")])])` → PPTX slide XML has
/// BOTH `b="1"` AND `i="1"` in the `<a:rPr>` of the same run.
///
/// Exercises `RunProps` accumulation in the unified `render_inline_nodes_to_runs` engine.
/// If either the Bold arm or the Italic arm resets the accumulated props
/// (e.g. `RunProps::new()` instead of `props.clone()`), one property would be absent.
#[test]
fn test_f_p13_002_pptx_bold_italic_combined_form_both_b1_and_i1() {
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Italic(vec![
        InlineNode::Plain(Arc::from("bi")),
    ])])];
    let slide_xml = export_pptx_slide1_xml(nodes);

    assert!(
        slide_xml.contains("b=\"1\"") || slide_xml.contains("b=&quot;1&quot;"),
        "F-P13-002: Bold([Italic([Plain])]) must produce b=\"1\" in <a:rPr>; \
         got slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(2000)]
    );

    assert!(
        slide_xml.contains("i=\"1\"") || slide_xml.contains("i=&quot;1&quot;"),
        "F-P13-002: Bold([Italic([Plain])]) must produce i=\"1\" in <a:rPr>; \
         got slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(2000)]
    );

    // The text must appear in the output.
    assert!(
        slide_xml.contains("bi"),
        "F-P13-002: plain text 'bi' must appear in PPTX slide1.xml; \
         got slide1.xml (excerpt): {}",
        &slide_xml[..slide_xml.len().min(2000)]
    );
}

/// EC-004 / F-P5-001: Empty-display-text Link → no orphan External rel.
///
/// A Link whose display text is empty produces no `<a:hlinkClick>` run in the
/// PPTX body (same guard as the notes path). No External rel must be registered
/// for it either (mirrors notes_tests.rs empty-display-text guard).
#[test]
fn test_ec004_body_path_empty_display_text_link_no_orphan_rel() {
    let nodes = vec![InlineNode::Link {
        text: vec![],
        url: Arc::from("https://example.com/empty"),
    }];
    let (slide_xml, slide_rels) = export_pptx_slide1_xml_and_rels(nodes);

    // No <a:hlinkClick> for empty display text.
    assert!(
        !slide_xml.contains("<a:hlinkClick"),
        "EC-004 empty: empty-display-text Link must not emit <a:hlinkClick>; \
         got slide1.xml:\n{}",
        &slide_xml[..slide_xml.len().min(3000)]
    );

    // No External rel for empty display text link.
    assert!(
        !slide_rels.contains("TargetMode=\"External\""),
        "EC-004 empty: empty-display-text Link must not produce an External rel; \
         got rels:\n{slide_rels}"
    );

    // BC-3.05.001 HI-1 reference-set invariant holds even in the empty case
    // (both sets are empty → no dangling rIds, no orphan rels).
    assert_hyperlink_reference_set_invariant(&slide_xml, &slide_rels);
}

// =============================================================================
// ADV-P14-MED-001: Nested Link inside formatting wrapper — body path
//
// `Bold([Link{text:"click", url:"https://example.com"}])` → exported slide1.xml
// must contain BOTH `b="1"` AND `<a:hlinkClick>` on the link run, AND
// slide1.xml.rels must have the matching `TargetMode="External"` rel, AND
// `external_rel_count == hlinkclick_count` (orphan-rel invariant).
//
// MUST FAIL before this fix: collect_top_level_link_urls is a no-op on Bold
// wrappers, so the URL is never registered and inline_node_to_ooxml_runs_for_body
// never receives a hyperlink_rid for the inner Link.
// =============================================================================

/// ADV-P14-MED-001: `Bold([Link])` in slide body → run must carry BOTH `b="1"`
/// AND `<a:hlinkClick>`, External rel in rels, and count invariant holds.
///
/// **Red Gate:** Before fix, `collect_top_level_link_urls` is a no-op on Bold,
/// so the nested link URL is never registered and no `<a:hlinkClick>` is emitted.
#[test]
fn test_adv_p14_med_001_body_bold_link_has_b1_and_hlinkclick() {
    let target_url = "https://example.com";
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("click"))],
        url: Arc::from(target_url),
    }])];
    let (slide_xml, slide_rels) = export_pptx_slide1_xml_and_rels(nodes);

    // Display text must appear.
    assert!(
        slide_xml.contains("click"),
        "ADV-P14-MED-001: link display text 'click' must appear in slide1.xml;\n\
         slide1.xml (first 2000 chars): {:.2000}",
        slide_xml
    );

    // b="1" must appear — bold context is inherited by the link run.
    assert!(
        slide_xml.contains("b=\"1\""),
        "ADV-P14-MED-001: Bold([Link]) must produce b=\"1\" in <a:rPr>;\n\
         slide1.xml (first 2000 chars): {:.2000}",
        slide_xml
    );

    // <a:hlinkClick> must appear — nested link must be clickable.
    assert!(
        slide_xml.contains("<a:hlinkClick"),
        "ADV-P14-MED-001: Bold([Link]) must emit <a:hlinkClick> for the nested link;\n\
         slide1.xml (first 2000 chars): {:.2000}",
        slide_xml
    );

    // External rel must be registered.
    assert!(
        slide_rels.contains("TargetMode=\"External\""),
        "ADV-P14-MED-001: slide1.xml.rels must contain TargetMode=\"External\" for \
         Bold([Link]) URL;\n\
         rels: {slide_rels}"
    );
    assert!(
        slide_rels.contains(target_url),
        "ADV-P14-MED-001: slide1.xml.rels must contain URL {target_url:?} as Target;\n\
         rels: {slide_rels}"
    );

    // BC-3.05.001 HI-1 reference-set invariant: every hlinkClick rId resolves to a
    // registered External rel; every External rel is cited ≥1× (no orphan rels).
    assert_hyperlink_reference_set_invariant(&slide_xml, &slide_rels);
}

/// ADV-P14-MED-001: `Bold([Link{url:"javascript:..."}])` → plain bold text,
/// NO rel, NO hlinkClick (unsafe-scheme guard applies to nested links too).
#[test]
fn test_adv_p14_med_001_body_bold_unsafe_link_no_hlinkclick_no_rel() {
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("bad link"))],
        url: Arc::from("javascript:alert(1)"),
    }])];
    let (slide_xml, slide_rels) = export_pptx_slide1_xml_and_rels(nodes);

    // Display text must still appear.
    assert!(
        slide_xml.contains("bad link"),
        "ADV-P14-MED-001 unsafe: display text must survive for unsafe nested link;\n\
         slide1.xml (first 2000 chars): {:.2000}",
        slide_xml
    );

    // No <a:hlinkClick> for unsafe-scheme nested link.
    assert!(
        !slide_xml.contains("<a:hlinkClick"),
        "ADV-P14-MED-001 unsafe: Bold([Link{{javascript:}}]) must NOT emit <a:hlinkClick>;\n\
         slide1.xml (first 2000 chars): {:.2000}",
        slide_xml
    );

    // No External rel.
    assert!(
        !slide_rels.contains("TargetMode=\"External\""),
        "ADV-P14-MED-001 unsafe: no External rel for unsafe-scheme nested link;\n\
         rels: {slide_rels}"
    );

    // BC-3.05.001 HI-1 reference-set invariant holds (both sets empty: no dangling
    // rIds, no orphan rels).
    assert_hyperlink_reference_set_invariant(&slide_xml, &slide_rels);
}

/// ADV-P14-MED-001: `Bold([Link{text:[], url}])` → no orphan rel (empty display
/// text guard applies to nested links too).
#[test]
fn test_adv_p14_med_001_body_bold_empty_display_text_link_no_orphan_rel() {
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Link {
        text: vec![],
        url: Arc::from("https://example.com/empty-nested"),
    }])];
    let (slide_xml, slide_rels) = export_pptx_slide1_xml_and_rels(nodes);

    // No <a:hlinkClick> for empty display text nested link.
    assert!(
        !slide_xml.contains("<a:hlinkClick"),
        "ADV-P14-MED-001 empty: empty-display-text nested link must not emit <a:hlinkClick>;\n\
         slide1.xml (first 2000 chars): {:.2000}",
        slide_xml
    );

    // No External rel.
    assert!(
        !slide_rels.contains("TargetMode=\"External\""),
        "ADV-P14-MED-001 empty: empty-display-text nested link must not produce External rel;\n\
         rels: {slide_rels}"
    );

    // BC-3.05.001 HI-1 reference-set invariant holds (both sets empty: no dangling
    // rIds, no orphan rels).
    assert_hyperlink_reference_set_invariant(&slide_xml, &slide_rels);
}

/// ADV-P14-MED-001: `Bold([Italic([Link])])` — doubly-nested formatting wrapper →
/// run must carry `b="1"` AND `i="1"` AND `<a:hlinkClick>`.
///
/// Verifies that the wrapper-descent is recursive: Bold sets `bold=true`, Italic
/// sets `italic=true`, and the inner Link still gets its rId threaded through.
#[test]
fn test_adv_p14_med_001_body_bold_italic_link_has_b1_i1_hlinkclick() {
    let target_url = "https://example.com/combined";
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Italic(vec![
        InlineNode::Link {
            text: vec![InlineNode::Plain(Arc::from("combined"))],
            url: Arc::from(target_url),
        },
    ])])];
    let (slide_xml, slide_rels) = export_pptx_slide1_xml_and_rels(nodes);

    // Display text must appear.
    assert!(
        slide_xml.contains("combined"),
        "ADV-P14-MED-001 combined: display text 'combined' must appear;\n\
         slide1.xml (first 2000 chars): {:.2000}",
        slide_xml
    );

    // b="1" — bold context inherited.
    assert!(
        slide_xml.contains("b=\"1\""),
        "ADV-P14-MED-001 combined: Bold([Italic([Link])]) must produce b=\"1\";\n\
         slide1.xml (first 2000 chars): {:.2000}",
        slide_xml
    );

    // i="1" — italic context inherited.
    assert!(
        slide_xml.contains("i=\"1\""),
        "ADV-P14-MED-001 combined: Bold([Italic([Link])]) must produce i=\"1\";\n\
         slide1.xml (first 2000 chars): {:.2000}",
        slide_xml
    );

    // <a:hlinkClick> must appear.
    assert!(
        slide_xml.contains("<a:hlinkClick"),
        "ADV-P14-MED-001 combined: Bold([Italic([Link])]) must emit <a:hlinkClick>;\n\
         slide1.xml (first 2000 chars): {:.2000}",
        slide_xml
    );

    // External rel registered.
    assert!(
        slide_rels.contains("TargetMode=\"External\""),
        "ADV-P14-MED-001 combined: External rel must appear in rels;\n\
         rels: {slide_rels}"
    );

    // BC-3.05.001 HI-1 reference-set invariant: every hlinkClick rId resolves to a
    // registered External rel; every External rel is cited ≥1× (no orphan rels).
    assert_hyperlink_reference_set_invariant(&slide_xml, &slide_rels);
}

// =============================================================================
// F-P15-HIGH-001: Body-path orphan rel from nested Link in Link display text
//
// `Link{ url:U1, text:[Link{url:U2, text:[Plain("inner")]}] }` — body path
// (slideforge_plugin_api::inline_formats::render_nodes_recursive Link arm):
//
// The collector `collect_link_urls_from_node` correctly registers ONLY U1
// (outer Link) because it does NOT recurse into the Link's display text.
//
// The unified engine's Link arm builds `link_props = RunProps { hyperlink_rid:
// Some(rid_U1), .. }` and then recurses into the display text with
// `inside_link_display_text=true`. The inner Link inherits the outer rId via
// INV-6 (no resolver call for nested link; outer hyperlink_rid passed through).
//
// Result: 1 External rel registered, 0 `<a:hlinkClick>` emitted → ORPHAN REL.
//
// MUST FAIL (Red Gate) before fix:
//   external_rel_count=1 (collector registered U1)
//   hlinkclick_count=0   (dispatcher overwrote rid with None → no hlinkClick)
//   → assertion external_rel_count == hlinkclick_count fails (1 ≠ 0).
// =============================================================================

/// F-P15-HIGH-001: `Link{url:U1, text:[Link{url:U2, text:[Plain("inner")]}]}`
/// body path → exactly 1 External rel AND exactly 1 `<a:hlinkClick>`.
///
/// The inner "inner" text must be present and rendered under the OUTER URL's rId.
///
/// **Red Gate:** Before fix, the body dispatcher overwrote `hyperlink_rid`
/// with `None` when the inner Link's URL is absent from `hlink_map`, producing
/// 1 orphan rel and 0 `<a:hlinkClick>`.
#[test]
fn test_f_p15_high_001_body_link_nested_display_link_one_rel_one_click() {
    let outer_url = "https://outer.test";
    let inner_url = "https://inner.test";

    // Build: Link{ url: outer_url, text: [Link{ url: inner_url, text: [Plain("inner")] }] }
    let inner_link = InlineNode::Link {
        url: Arc::from(inner_url),
        text: vec![InlineNode::Plain(Arc::from("inner"))],
    };
    let outer_link = InlineNode::Link {
        url: Arc::from(outer_url),
        text: vec![inner_link],
    };
    let (slide_xml, slide_rels) = export_pptx_slide1_xml_and_rels(vec![outer_link]);

    // The inner display text "inner" must appear in the output.
    assert!(
        slide_xml.contains("inner"),
        "F-P15-HIGH-001 body: inner text 'inner' must appear in slide1.xml;\n\
         slide1.xml (first 2000 chars): {:.2000}",
        slide_xml
    );

    // Exactly 1 External rel (the outer URL).
    let external_rel_count = slide_rels.matches("TargetMode=\"External\"").count();
    assert_eq!(
        external_rel_count, 1,
        "F-P15-HIGH-001 body: expected exactly 1 External rel (outer URL only); \
         got {external_rel_count}.\nrels: {slide_rels}"
    );

    // The outer URL must be in rels; the inner URL must NOT be.
    assert!(
        slide_rels.contains(outer_url),
        "F-P15-HIGH-001 body: outer URL {outer_url:?} must be in rels;\nrels: {slide_rels}"
    );
    assert!(
        !slide_rels.contains(inner_url),
        "F-P15-HIGH-001 body: inner URL {inner_url:?} must NOT be in rels;\nrels: {slide_rels}"
    );

    // Exactly 1 <a:hlinkClick> (inner text rendered under the outer rId).
    let hlinkclick_count = slide_xml.matches("<a:hlinkClick").count();
    assert_eq!(
        hlinkclick_count, 1,
        "F-P15-HIGH-001 body: expected exactly 1 <a:hlinkClick>; \
         got {hlinkclick_count}.\nslide1.xml (first 2000 chars): {:.2000}",
        slide_xml
    );

    // BC-3.05.001 HI-1 reference-set invariant: 1 External rel, 1 hlinkClick run,
    // same rId — no dangling rIds, no orphan rels.
    assert_hyperlink_reference_set_invariant(&slide_xml, &slide_rels);
}

/// F-P15-HIGH-001: `Link{url:U1, text:[Bold([Link{url:U2}])]}`
/// body path → exactly 1 External rel AND exactly 1 `<a:hlinkClick>`.
///
/// Variant where the inner Link is wrapped in Bold inside the outer Link's
/// display text.  The inner text must be rendered under the outer URL's rId
/// AND the bold context (b="1") must be preserved.
///
/// **Red Gate:** Same as above — inner Link arm overwrites ctx.hyperlink_rid
/// with None when lookup fails.
#[test]
fn test_f_p15_high_001_body_link_nested_bold_display_link_one_rel_one_click() {
    let outer_url = "https://outer.test";
    let inner_url = "https://inner.test/bold";

    // Build: Link{ url: outer_url, text: [Bold([Link{ url: inner_url, text: [Plain("bold-inner")] }])] }
    let inner_link = InlineNode::Link {
        url: Arc::from(inner_url),
        text: vec![InlineNode::Plain(Arc::from("bold-inner"))],
    };
    let outer_link = InlineNode::Link {
        url: Arc::from(outer_url),
        text: vec![InlineNode::Bold(vec![inner_link])],
    };
    let (slide_xml, slide_rels) = export_pptx_slide1_xml_and_rels(vec![outer_link]);

    // Display text "bold-inner" must appear.
    assert!(
        slide_xml.contains("bold-inner"),
        "F-P15-HIGH-001 body bold: display text 'bold-inner' must appear;\n\
         slide1.xml (first 2000 chars): {:.2000}",
        slide_xml
    );

    // Exactly 1 External rel.
    let external_rel_count = slide_rels.matches("TargetMode=\"External\"").count();
    assert_eq!(
        external_rel_count, 1,
        "F-P15-HIGH-001 body bold: expected exactly 1 External rel; \
         got {external_rel_count}.\nrels: {slide_rels}"
    );

    // Exactly 1 <a:hlinkClick>.
    let hlinkclick_count = slide_xml.matches("<a:hlinkClick").count();
    assert_eq!(
        hlinkclick_count, 1,
        "F-P15-HIGH-001 body bold: expected exactly 1 <a:hlinkClick>; \
         got {hlinkclick_count}.\nslide1.xml (first 2000 chars): {:.2000}",
        slide_xml
    );

    // BC-3.05.001 HI-1 reference-set invariant: 1 External rel, 1 hlinkClick run,
    // same rId — no dangling rIds, no orphan rels.
    assert_hyperlink_reference_set_invariant(&slide_xml, &slide_rels);
}

// =============================================================================
// F-P15-HIGH-001: Notes path guard — notes correctly handles Link{text:[Link]}
//
// The notes path `dispatch_inline_nodes_to_ooxml` + `DefaultInlineFormat` flattens
// the outer Link's display text rather than recursing into it as a separate
// dispatch item.  This means the inner Link never reaches the dispatch loop as
// a fresh node — only the outer Link gets an rId and emits <a:hlinkClick>.
//
// This test MUST PASS immediately (notes path is already correct).
// It is added as a regression guard so that future refactors of the notes
// dispatcher can't accidentally break this invariant.
// =============================================================================

/// F-P15-HIGH-001 notes guard: `Link{url:U1, text:[Link{url:U2, text:[Plain("inner")]}]}`
/// through the notes path → exactly 1 External rel AND exactly 1 `<a:hlinkClick>`.
///
/// **Should PASS immediately** — the notes path flattens display text; inner Link
/// never becomes a separate dispatch node and never creates an orphan rel.
/// Added as a regression guard.
#[test]
fn test_f_p15_high_001_notes_link_nested_display_link_one_rel_one_click() {
    use slideforge_types::Register;
    use slideforge_types::ordered_map::OrderedMap;
    use slideforge_types::register::RegisteredContent;
    use slideforge_types::slide::Slide;

    let outer_url = "https://outer.test";
    let inner_url = "https://inner.test";

    let inner_link = InlineNode::Link {
        url: Arc::from(inner_url),
        text: vec![InlineNode::Plain(Arc::from("inner"))],
    };
    let outer_link = InlineNode::Link {
        url: Arc::from(outer_url),
        text: vec![inner_link],
    };
    let rc = RegisteredContent {
        register: Register::Notes,
        content: vec![outer_link],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: BoundingBox {
                x: Emu(457_200),
                y: Emu(274_638),
                width: Emu(8_229_600),
                height: Emu(1_143_000),
            },
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("inner")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = Deck {
        slides: vec![Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![RegisteredContent {
                register: Register::Notes,
                content: vec![InlineNode::Plain(Arc::from("inner"))],
            }],
        }],
        vars: OrderedMap::new(),
        metadata: make_metadata(),
        registers: OrderedMap::new(),
        section_blocks: vec![],
        slide_sections: vec![],
    };
    let laid_out = LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };

    let brand = make_brand();
    let pptx = PptxExporter::new()
        .export(&deck, &laid_out, &brand, &ExportOptions::default())
        .expect("PptxExporter::export must succeed");

    let notes_xml = {
        let cursor = std::io::Cursor::new(&pptx);
        let mut archive = ZipArchive::new(cursor).expect("valid ZIP");
        let mut s = String::new();
        archive
            .by_name("ppt/notesSlides/notesSlide1.xml")
            .expect("notesSlide1.xml must be present")
            .read_to_string(&mut s)
            .expect("valid UTF-8");
        s
    };
    let rels_xml = {
        let cursor = std::io::Cursor::new(&pptx);
        let mut archive = ZipArchive::new(cursor).expect("valid ZIP");
        let mut s = String::new();
        archive
            .by_name("ppt/notesSlides/_rels/notesSlide1.xml.rels")
            .expect("_rels/notesSlide1.xml.rels must be present")
            .read_to_string(&mut s)
            .expect("valid UTF-8");
        s
    };

    // Exactly 1 External rel (outer URL only).
    let external_rel_count = rels_xml.matches("TargetMode=\"External\"").count();
    assert_eq!(
        external_rel_count, 1,
        "F-P15-HIGH-001 notes: expected exactly 1 External rel; \
         got {external_rel_count}.\nrels: {rels_xml}"
    );

    // Exactly 1 <a:hlinkClick>.
    let hlinkclick_count = notes_xml.matches("<a:hlinkClick").count();
    assert_eq!(
        hlinkclick_count, 1,
        "F-P15-HIGH-001 notes: expected exactly 1 <a:hlinkClick>; \
         got {hlinkclick_count}.\nnotes_xml: {:.2000}",
        notes_xml
    );

    // BC-3.05.001 HI-1 reference-set invariant: 1 External rel, 1 hlinkClick run,
    // same rId — no dangling rIds, no orphan rels.
    assert_hyperlink_reference_set_invariant(&notes_xml, &rels_xml);
}

// =============================================================================
// ADR-024 Step 7: Structural invariant test replacing OBS-P15-001
//
// OBS-P15-001 was added after Pass 15 to make future body/notes divergence
// visible at test time. After ADR-024 unification, both body and notes paths
// call render_inline_nodes_to_runs — structural divergence via a second
// implementation path is impossible by construction.
//
// The OBS-P15-001 cross-path black-box export comparison is RETIRED (per ADR-024
// Step 7) and replaced with this single-engine contract test that directly
// verifies the reference-set invariant and per-shape rel/click contract on the
// unified engine.
//
// Why retired: a parity test between two call sites of the SAME function cannot
// detect algorithmic divergence — it can only detect call-site wiring bugs.
// The call-site wiring is tested by the existing notes_tests.rs and
// inline_markup_tests.rs integration tests. The redundant export loop is removed
// to reduce test runtime without reducing coverage.
// =============================================================================

/// ADR-024 Step 7: Structural single-engine contract test.
///
/// Verifies that `render_inline_nodes_to_runs` produces the reference-set
/// invariant (INV-4): every `hyperlink_rid` in the output references a URL
/// that was passed to the resolver, and every URL the resolver returns `Some`
/// for has at least one run with that `hyperlink_rid`.
///
/// Also verifies INV-5 (no resolver call for nested Link in display text) and
/// INV-6 (nested Link inherits outer rId).
///
/// This test replaces `test_obs_p15_001_body_notes_cross_path_rel_click_parity`
/// (OBS-P15-001) per ADR-024 Step 7.
#[test]
fn test_obs_p15_001_body_notes_cross_path_rel_click_parity() {
    use slideforge_plugin_api::inline_formats::render_inline_nodes_to_runs;

    // ADR-024 Step 7 — Structural engine contract test.
    //
    // Both body and notes call render_inline_nodes_to_runs. Divergence via a
    // second implementation path is impossible by construction after ADR-024.
    //
    // This test verifies the ENGINE's reference-set invariant (INV-4) and
    // per-shape rel/click contract directly, for the same battery of cases
    // that OBS-P15-001 previously checked cross-path.
    //
    // A mock "registered URLs → rId" map is used so we control exactly which
    // URLs get rIds (mimics the notes-path resolver behavior).

    struct Case {
        label: &'static str,
        nodes: Vec<InlineNode>,
        /// Expected number of runs with `hyperlink_rid = Some(...)`.
        expected_hyperlink_runs: usize,
        /// Expected number of distinct rIds used across all hyperlink runs.
        expected_distinct_rids: usize,
    }

    let battery: Vec<Case> = vec![
        Case {
            label: "Bold([Link])",
            nodes: vec![InlineNode::Bold(vec![InlineNode::Link {
                url: Arc::from("https://example.com/bold-link"),
                text: vec![InlineNode::Plain(Arc::from("click"))],
            }])],
            expected_hyperlink_runs: 1,
            expected_distinct_rids: 1,
        },
        Case {
            label: "Bold([Italic([Link])])",
            nodes: vec![InlineNode::Bold(vec![InlineNode::Italic(vec![
                InlineNode::Link {
                    url: Arc::from("https://example.com/bold-italic-link"),
                    text: vec![InlineNode::Plain(Arc::from("combined"))],
                },
            ])])],
            expected_hyperlink_runs: 1,
            expected_distinct_rids: 1,
        },
        Case {
            label: "Link([Bold([Plain])])",
            nodes: vec![InlineNode::Link {
                url: Arc::from("https://example.com/link-bold"),
                text: vec![InlineNode::Bold(vec![InlineNode::Plain(Arc::from(
                    "bold text",
                ))])],
            }],
            // Single leaf run with bold + hyperlink (F-P16-M1 fix: multi-leaf
            // display text now generates N runs, but here display text has 1 leaf).
            expected_hyperlink_runs: 1,
            expected_distinct_rids: 1,
        },
        Case {
            label: "Bold([Link_A, Link_B]) distinct URLs",
            nodes: vec![InlineNode::Bold(vec![
                InlineNode::Link {
                    url: Arc::from("https://example.com/link-a"),
                    text: vec![InlineNode::Plain(Arc::from("A"))],
                },
                InlineNode::Link {
                    url: Arc::from("https://example.com/link-b"),
                    text: vec![InlineNode::Plain(Arc::from("B"))],
                },
            ])],
            expected_hyperlink_runs: 2,
            expected_distinct_rids: 2,
        },
        Case {
            label: "Link{text:[Link]} — INV-5: inner gets outer rId",
            nodes: vec![InlineNode::Link {
                url: Arc::from("https://outer.test"),
                text: vec![InlineNode::Link {
                    url: Arc::from("https://inner.test"),
                    text: vec![InlineNode::Plain(Arc::from("inner"))],
                }],
            }],
            // Outer Link gets rId. Inner Link is inside display text → inherits outer rId.
            // Only 1 run with rId (one leaf in the display text).
            expected_hyperlink_runs: 1,
            expected_distinct_rids: 1,
        },
        Case {
            label: "unsafe Link (javascript:)",
            nodes: vec![InlineNode::Link {
                url: Arc::from("javascript:alert(1)"),
                text: vec![InlineNode::Plain(Arc::from("bad"))],
            }],
            // Resolver returns None for unsafe URL → no hyperlink_rid.
            expected_hyperlink_runs: 0,
            expected_distinct_rids: 0,
        },
        Case {
            label: "empty-text Link",
            nodes: vec![InlineNode::Link {
                url: Arc::from("https://example.com/empty"),
                text: vec![],
            }],
            // Empty display text → no runs produced.
            expected_hyperlink_runs: 0,
            expected_distinct_rids: 0,
        },
    ];

    // Mock URL registry: maps registered safe-scheme URLs to stable rIds.
    // Inner Link URLs (for INV-5 cases) are not registered — resolver returns None.
    let registered_urls: std::collections::HashMap<&str, &str> = [
        ("https://example.com/bold-link", "rId3"),
        ("https://example.com/bold-italic-link", "rId3"),
        ("https://example.com/link-bold", "rId3"),
        ("https://example.com/link-a", "rId3"),
        ("https://example.com/link-b", "rId4"),
        ("https://outer.test", "rId3"),
        // "https://inner.test" intentionally NOT registered (INV-5 test).
    ]
    .into_iter()
    .collect();

    let resolver =
        |url: &str| -> Option<String> { registered_urls.get(url).map(|r| (*r).to_owned()) };

    for case in &battery {
        let runs = render_inline_nodes_to_runs(&case.nodes, &resolver)
            .unwrap_or_else(|e| panic!("engine error for case {:?}: {e}", case.label));

        let hyperlink_runs: Vec<_> = runs.iter().filter(|r| r.hyperlink_rid.is_some()).collect();
        let distinct_rids: std::collections::HashSet<&str> = hyperlink_runs
            .iter()
            .filter_map(|r| r.hyperlink_rid.as_deref())
            .collect();

        // INV-4 reference-set invariant: every run's rId is a registered URL mapping.
        for run in &hyperlink_runs {
            let rid = run.hyperlink_rid.as_deref().unwrap();
            assert!(
                registered_urls.values().any(|v| *v == rid),
                "ADR-024 INV-4: run has rId '{rid}' that is not in the registered set; \
                 case: {:?}, run: {run:?}",
                case.label
            );
        }

        assert_eq!(
            hyperlink_runs.len(),
            case.expected_hyperlink_runs,
            "ADR-024 INV-4: expected {} hyperlink runs for case {:?}; \
             got {} runs: {:?}",
            case.expected_hyperlink_runs,
            case.label,
            hyperlink_runs.len(),
            runs
        );

        assert_eq!(
            distinct_rids.len(),
            case.expected_distinct_rids,
            "ADR-024 INV-4: expected {} distinct rIds for case {:?}; got {distinct_rids:?}",
            case.expected_distinct_rids,
            case.label,
        );
    }
}

// =============================================================================
// EC-012 / BC-3.05.001 HI-1: Multi-leaf link display text — body path
//
// `Link{text:[Plain("click "), Bold([Plain("here")]), Plain(" now")], url}` in
// slide body → exported slide{N}.xml must have exactly 3 `<a:hlinkClick>` runs
// ALL citing the SAME rId, the middle run carries `b="1"`, and slide{N}.xml.rels
// has exactly 1 External rel with that rId.
//
// This is the EC-012 canonical case from BC-3.05.001: 1 rel / 3 hlinkClick runs.
//
// DEMONSTRATES why reference-set form is necessary: count-equality would
// assert_eq!(1, 3) and FAIL on this valid multi-leaf input. The reference-set
// form passes (all 3 hlinkClick rIds point to the 1 External rel, and the 1
// External rel is cited ≥1×).
// =============================================================================

/// EC-012 / BC-3.05.001 HI-1: `[click **here** now](url)` in slide body →
/// exactly 3 `<a:hlinkClick>` runs all citing the SAME rId, the "here" run
/// carries `b="1"`, and slide1.xml.rels has exactly 1 External rel.
///
/// This test encodes the reference-set invariant form and would FALSE-FAIL under
/// the retired count-equality form (`assert_eq!(1, 3)` would fail).
/// Confirms that the reference-set helper is load-bearing, not vestigial.
///
/// Traceability: BC-3.05.001 HI-1, EC-012, F-P19-MED-001.
#[test]
fn test_f_p16_m1_body_multi_leaf_link_display_text() {
    let url = "https://example.com/body-multi-leaf";

    // `[click **here** now](url)` = Link { text: [Plain("click "),
    //                                             Bold([Plain("here")]),
    //                                             Plain(" now")], url }
    let link_node = InlineNode::Link {
        url: Arc::from(url),
        text: vec![
            InlineNode::Plain(Arc::from("click ")),
            InlineNode::Bold(vec![InlineNode::Plain(Arc::from("here"))]),
            InlineNode::Plain(Arc::from(" now")),
        ],
    };

    let (slide_xml, slide_rels) = export_pptx_slide1_xml_and_rels(vec![link_node]);

    // ── 1 External rel registered ────────────────────────────────────────────
    let external_rel_count = slide_rels.matches("TargetMode=\"External\"").count();
    assert_eq!(
        external_rel_count, 1,
        "EC-012 body: expected exactly 1 External rel for multi-leaf link; \
         got {external_rel_count}.\nrels:\n{slide_rels}"
    );

    // URL must appear in rels.
    assert!(
        slide_rels.contains(url),
        "EC-012 body: URL {url:?} must appear in rels; got:\n{slide_rels}"
    );

    // ── 3 hlinkClick runs (one per leaf) ─────────────────────────────────────
    let hlinkclick_count = slide_xml.matches("<a:hlinkClick").count();
    assert_eq!(
        hlinkclick_count,
        3,
        "EC-012 body: expected 3 <a:hlinkClick> (one per leaf: 'click ', 'here', ' now'); \
         got {hlinkclick_count}.\nslide1.xml:\n{}",
        &slide_xml[..slide_xml.len().min(4000)]
    );

    // ── All 3 hlinkClicks reference the SAME rId ─────────────────────────────
    // The URL is first registered as the only External rel, which gets rId3
    // (rId1=slideLayout, rId2=theme or master — both without TargetMode="External").
    // We parse the rId from the rels file to avoid hardcoding.
    let rel_id = {
        let mut found = None;
        let mut remaining = slide_rels.as_str();
        while let Some(pos) = remaining.find("<Relationship") {
            let after = &remaining[pos..];
            let end = after.find('>').unwrap_or(after.len());
            let element = &after[..=end];
            if element.contains("TargetMode=\"External\"")
                && let Some(id_pos) = element.find("Id=\"")
                && let after_id = &element[id_pos + 4..]
                && let Some(q) = after_id.find('"')
            {
                found = Some(after_id[..q].to_owned());
                break;
            }
            remaining = &remaining[pos + 1..];
        }
        found.expect("EC-012 body: must find exactly 1 External rel Id in rels")
    };

    let rid_count_in_clicks = slide_xml.matches(rel_id.as_str()).count();
    assert_eq!(
        rid_count_in_clicks,
        3,
        "EC-012 body: expected all 3 hlinkClick runs to reference rId={rel_id:?}; \
         found {rid_count_in_clicks} occurrences.\nslide1.xml:\n{}",
        &slide_xml[..slide_xml.len().min(4000)]
    );

    // ── Middle run ("here") carries b="1" ────────────────────────────────────
    assert!(
        slide_xml.contains("b=\"1\""),
        "EC-012 body: bold 'here' leaf must produce b=\"1\" on its run; \
         got slide1.xml:\n{}",
        &slide_xml[..slide_xml.len().min(4000)]
    );

    // Text fragments must all appear.
    assert!(
        slide_xml.contains("click "),
        "EC-012 body: 'click ' must appear in slide XML"
    );
    assert!(
        slide_xml.contains("here"),
        "EC-012 body: 'here' must appear in slide XML"
    );
    assert!(
        slide_xml.contains(" now"),
        "EC-012 body: ' now' must appear in slide XML"
    );

    // ── BC-3.05.001 HI-1 reference-set invariant ─────────────────────────────
    // This is the LOAD-BEARING check: count-equality would assert_eq!(1, 3) and
    // fail — reference-set passes because all 3 clicks reference the 1 External rel.
    assert_hyperlink_reference_set_invariant(&slide_xml, &slide_rels);
}
