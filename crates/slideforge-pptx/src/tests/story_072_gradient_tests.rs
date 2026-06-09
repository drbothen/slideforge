//! STORY-072: shape: Gradient Fills — PPTX exporter tests (Red Gate).
//!
//! Tests for AC-004 (PPTX native gradient emission via ooxmlsdk typed builders).
//!
//! ## Red Gate status
//!
//! ALL tests in this file MUST FAIL before the STORY-072 implementation begins.
//! The failure mode is a `todo!()` panic from stub functions in the PPTX serializer.
//!
//! ## Traceability
//!
//! | Test function | AC | BC-3.04.001 clause |
//! |---|---|---|
//! | `test_BC_3_04_001_ac004_pptx_shape_gradient_frame_emits_grad_fill_element` | AC-004 | postcondition 5 (all output formats) |
//! | `test_BC_3_04_001_ac004_pptx_gradient_stop_positions_are_0_and_100000` | AC-004 | postcondition 5 + OOXML §DrawingML |
//! | `test_BC_3_04_001_ac004_pptx_gradient_direction_is_top_to_bottom` | AC-004 | postcondition 5 |
//! | `test_BC_3_04_001_ac004_pptx_gradient_stop_colors_match_from_to` | AC-004 | postcondition 5 |
//! | `test_BC_3_04_001_ac004_pptx_gradient_stop_from_precedes_to_in_xml` | AC-004 | stop ordering (LOW-001) |
//! | `test_BC_3_04_001_ac004_pptx_gradient_shape_alt_text_on_cnvpr` | AC-004 + AC-005 | BC-3.04.001 invariant 1 |
//! | `test_BC_3_04_001_ac004_pptx_gradient_decorative_emits_empty_descr` | AC-005 | BC-4.01.004 postcondition 2 |
//! | `test_BC_3_04_001_ec005_pptx_same_from_to_colors_valid` | EC-005 | STORY-072 EC-005 |
//! | `test_BC_3_04_001_ac004_pptx_full_slide_with_gradient_shape_builds` | AC-004 | end-to-end |

#![allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    non_snake_case
)]

use std::sync::Arc;

use slideforge_layout::{
    BoundingBox, FillSpec, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize, ShapeFrame,
    ShapeType,
};
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::{AltText, Brand, BrandFonts, BrandPalette, Deck, Emu, Rgb};

use crate::PptxExporter;

// ─── Fixture builders ────────────────────────────────────────────────────────

/// Build a minimal `Deck` (required by `Exporter::export` signature).
fn make_deck_one_slide() -> Deck {
    use slideforge_types::deck::DeckMetadata;
    use slideforge_types::ordered_map::OrderedMap;
    use slideforge_types::slide::Slide;

    Deck {
        slides: vec![Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: slideforge_types::SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }],
        vars: OrderedMap::new(),
        metadata: DeckMetadata {
            title: Some(Arc::from("Gradient Test Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        },
        registers: OrderedMap::new(),
        section_blocks: vec![],
    }
}

/// Build a minimal `Brand`.
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
        span: slideforge_types::SourceSpan::default(),
    }
}

/// Build a `ShapeFrame` with `FillSpec::Gradient { from, to }`.
fn gradient_shape_frame(from: Rgb, to: Rgb, alt: AltText) -> ShapeFrame {
    ShapeFrame {
        shape_type: ShapeType::Rect,
        fill: FillSpec::Gradient { from, to },
        text: None,
        alt,
    }
}

/// Gradient bounding box: 1in × 1in at (1in, 1in).
fn gradient_bbox() -> BoundingBox {
    BoundingBox {
        x: Emu(914_400),
        y: Emu(914_400),
        width: Emu(914_400),
        height: Emu(914_400),
    }
}

/// Build a single-slide `LaidOutDeck` containing one gradient `ShapeFrame`.
fn gradient_slide_deck(from: Rgb, to: Rgb, alt: AltText) -> LaidOutDeck {
    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("title"),
            frames: vec![Frame {
                bbox: gradient_bbox(),
                content: FrameContent::Shape(gradient_shape_frame(from, to, alt)),
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        }],
        sections: vec![],
        warnings: vec![],
    }
}

/// Run the exporter and extract `slide1.xml` content.
fn build_pptx_slide1_xml(laid_out: &LaidOutDeck) -> String {
    use std::io::Read as _;
    use zip::ZipArchive;

    let deck = make_deck_one_slide();
    let brand = make_brand();
    let opts = ExportOptions::default();
    let exporter = PptxExporter::new();
    let bytes = exporter
        .export(&deck, laid_out, &brand, &opts)
        .expect("PptxExporter::export must succeed for gradient slide");

    let cursor = std::io::Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).expect("must be a valid ZIP");
    let mut entry = archive
        .by_name("ppt/slides/slide1.xml")
        .expect("slide1.xml must be present");
    let mut buf = String::new();
    entry.read_to_string(&mut buf).expect("must be valid UTF-8");
    buf
}

// ─── Tests ───────────────────────────────────────────────────────────────────

/// AC-004 (STORY-072) — A `FrameContent::Shape` with `FillSpec::Gradient` produces
/// a `<p:sp>` element containing `<a:gradFill>` in the PPTX slide XML.
///
/// RED GATE: The current serializer skips `FrameContent::Shape` frames entirely
/// (outputs nothing). After implementation, the XML must contain `<a:gradFill>`.
#[test]
fn test_BC_3_04_001_ac004_pptx_shape_gradient_frame_emits_grad_fill_element() {
    let from = Rgb { r: 255, g: 0, b: 0 };
    let to = Rgb { r: 0, g: 0, b: 255 };
    let laid_out = gradient_slide_deck(
        from,
        to,
        AltText::Provided(Arc::from("Gradient background")),
    );
    let xml = build_pptx_slide1_xml(&laid_out);

    // RED GATE: currently the Shape frame is skipped — no <a:gradFill> in output.
    // After implementation, this assertion must pass.
    assert!(
        xml.contains("gradFill"),
        "PPTX slide1.xml must contain 'gradFill' for a FillSpec::Gradient shape; \
         got XML (first 500 chars): {}",
        &xml[..xml.len().min(500)]
    );
}

/// AC-004 (STORY-072) — PPTX gradient stops are at `pos="0"` and `pos="100000"`.
///
/// ECMA-376 requires integer stop positions: 0 = 0%, 100000 = 100%.
/// No float positions are permitted (DI-010 / CLAUDE.md "no f64 in IR").
///
/// RED GATE: fails because `<a:gradFill>` is not yet emitted.
#[test]
fn test_BC_3_04_001_ac004_pptx_gradient_stop_positions_are_0_and_100000() {
    let laid_out = gradient_slide_deck(
        Rgb { r: 255, g: 0, b: 0 },
        Rgb { r: 0, g: 0, b: 255 },
        AltText::Provided(Arc::from("Gradient background")),
    );
    let xml = build_pptx_slide1_xml(&laid_out);

    // OOXML integer stop positions.
    assert!(
        xml.contains("pos=\"0\""),
        "PPTX gradient XML must contain stop pos=\"0\"; got XML snippet: {}",
        &xml[..xml.len().min(500)]
    );
    assert!(
        xml.contains("pos=\"100000\""),
        "PPTX gradient XML must contain stop pos=\"100000\"; got XML snippet: {}",
        &xml[..xml.len().min(500)]
    );
}

/// AC-004 (STORY-072) — PPTX gradient direction is top-to-bottom:
/// `<a:lin ang="5400000"/>`.
///
/// In OOXML, the linear angle is in 1/60000 degrees. 5400000 = 90°, which is
/// top-to-bottom (vertical linear, per STORY-072 v1.0 fixed direction).
///
/// RED GATE: fails because `<a:lin>` is not yet emitted.
#[test]
fn test_BC_3_04_001_ac004_pptx_gradient_direction_is_top_to_bottom() {
    let laid_out = gradient_slide_deck(
        Rgb { r: 255, g: 0, b: 0 },
        Rgb { r: 0, g: 0, b: 255 },
        AltText::Provided(Arc::from("Gradient background")),
    );
    let xml = build_pptx_slide1_xml(&laid_out);

    assert!(
        xml.contains("5400000"),
        "PPTX gradient XML must contain ang=\"5400000\" (top-to-bottom); \
         got XML snippet: {}",
        &xml[..xml.len().min(500)]
    );
}

/// AC-004 (STORY-072) — PPTX gradient stop colors match the `from` and `to` `Rgb`.
///
/// `from` (#FF0000, red) must appear at pos=0; `to` (#0000FF, blue) at pos=100000.
///
/// RED GATE: fails because gradient fill is not emitted.
#[test]
fn test_BC_3_04_001_ac004_pptx_gradient_stop_colors_match_from_to() {
    let from = Rgb { r: 255, g: 0, b: 0 }; // #FF0000
    let to = Rgb { r: 0, g: 0, b: 255 }; // #0000FF
    let laid_out = gradient_slide_deck(
        from,
        to,
        AltText::Provided(Arc::from("Red-to-blue gradient")),
    );
    let xml = build_pptx_slide1_xml(&laid_out);

    // Both hex values must appear in the XML.
    assert!(
        xml.contains("FF0000") || xml.contains("ff0000"),
        "PPTX gradient XML must contain 'FF0000' (from color) in a gradient stop; \
         got XML snippet: {}",
        &xml[..xml.len().min(500)]
    );
    assert!(
        xml.contains("0000FF") || xml.contains("0000ff"),
        "PPTX gradient XML must contain '0000FF' (to color) in a gradient stop; \
         got XML snippet: {}",
        &xml[..xml.len().min(500)]
    );
}

/// AC-004 (STORY-072) — PPTX gradient stops are emitted in order: `from` color at
/// `pos="0"` PRECEDES `to` color at `pos="100000"` in the serialized XML.
///
/// A swapped emission (from@100000, to@0) would still pass the independent existence
/// checks in `test_BC_3_04_001_ac004_pptx_gradient_stop_colors_match_from_to`, but
/// would produce a visually reversed gradient. This test pins the ordering explicitly
/// by comparing substring offsets (LOW-001 adversary Pass-1 finding closure).
///
/// Contract: `from` = #FF0000 (red), `to` = #0000FF (blue).
/// Expected XML order: `...pos="0"...FF0000...` before `...pos="100000"...0000FF...`
#[test]
fn test_BC_3_04_001_ac004_pptx_gradient_stop_from_precedes_to_in_xml() {
    let from = Rgb { r: 255, g: 0, b: 0 }; // #FF0000
    let to = Rgb { r: 0, g: 0, b: 255 }; // #0000FF
    let laid_out = gradient_slide_deck(
        from,
        to,
        AltText::Provided(Arc::from("Red-to-blue gradient for ordering test")),
    );
    let xml = build_pptx_slide1_xml(&laid_out);

    // Locate the `pos="0"` stop block containing the from-color (FF0000).
    // Locate the `pos="100000"` stop block containing the to-color (0000FF).
    // The from-stop must appear BEFORE the to-stop in the serialized XML.
    let from_hex = if xml.contains("FF0000") {
        "FF0000"
    } else {
        "ff0000"
    };
    let to_hex = if xml.contains("0000FF") {
        "0000FF"
    } else {
        "0000ff"
    };

    let from_pos = xml.find(from_hex).unwrap_or(usize::MAX);
    let to_pos = xml.find(to_hex).unwrap_or(usize::MAX);

    assert_ne!(
        from_pos,
        usize::MAX,
        "from-color '{}' must appear in PPTX gradient XML; got (first 500): {}",
        from_hex,
        &xml[..xml.len().min(500)]
    );
    assert_ne!(
        to_pos,
        usize::MAX,
        "to-color '{}' must appear in PPTX gradient XML; got (first 500): {}",
        to_hex,
        &xml[..xml.len().min(500)]
    );
    assert!(
        from_pos < to_pos,
        "from-color (pos=\"0\") stop must precede to-color (pos=\"100000\") stop in XML; \
         found from-color '{}' at byte {} AFTER to-color '{}' at byte {}. \
         XML (first 600): {}",
        from_hex,
        from_pos,
        to_hex,
        to_pos,
        &xml[..xml.len().min(600)]
    );
}

/// AC-004 + AC-005 (STORY-072) — PPTX gradient shape alt text appears in `descr`
/// on `<p:cNvPr>` (BC-4.01.004 invariant 2 / BC-3.04.001 invariant 1).
///
/// RED GATE: fails because gradient shape frame is not yet serialized to `<p:sp>`.
#[test]
fn test_BC_3_04_001_ac004_pptx_gradient_shape_alt_text_on_cnvpr() {
    let alt_text = "Red to blue gradient background";
    let laid_out = gradient_slide_deck(
        Rgb { r: 255, g: 0, b: 0 },
        Rgb { r: 0, g: 0, b: 255 },
        AltText::Provided(Arc::from(alt_text)),
    );
    let xml = build_pptx_slide1_xml(&laid_out);

    assert!(
        xml.contains(alt_text),
        "PPTX slide1.xml must contain the gradient shape alt text '{alt_text}' in descr; \
         got XML snippet: {}",
        &xml[..xml.len().min(600)]
    );
}

/// AC-005 (STORY-072) — `AltText::Decorative` gradient shape emits `descr=""`
/// (empty attribute) on `<p:cNvPr>` (BC-4.01.004 postcondition 2).
///
/// RED GATE: fails because gradient shape frame is not yet serialized.
#[test]
fn test_BC_3_04_001_ac004_pptx_gradient_decorative_emits_empty_descr() {
    let laid_out = gradient_slide_deck(
        Rgb { r: 0, g: 255, b: 0 },
        Rgb { r: 0, g: 0, b: 255 },
        AltText::Decorative,
    );
    let xml = build_pptx_slide1_xml(&laid_out);

    // After implementation: descr="" must appear in the <p:cNvPr> of the gradient shape.
    assert!(
        xml.contains("descr=\"\""),
        "Decorative gradient shape must emit descr=\"\" on <p:cNvPr>; \
         got XML snippet: {}",
        &xml[..xml.len().min(500)]
    );
}

/// EC-005 (STORY-072) — Same `from` and `to` color (flat gradient) is valid;
/// the PPTX serializer must not panic and must produce a gradient fill element.
///
/// RED GATE: fails because gradient shape frame is not yet serialized.
#[test]
fn test_BC_3_04_001_ec005_pptx_same_from_to_colors_valid() {
    let same_color = Rgb { r: 255, g: 0, b: 0 };
    let laid_out = gradient_slide_deck(
        same_color,
        same_color,
        AltText::Provided(Arc::from("Flat gradient")),
    );
    // No panic expected; must produce a valid PPTX with gradFill.
    let xml = build_pptx_slide1_xml(&laid_out);
    assert!(
        xml.contains("gradFill"),
        "Flat gradient (same from==to) must still emit a gradFill element; \
         got XML snippet: {}",
        &xml[..xml.len().min(500)]
    );
}

/// AC-004 (STORY-072) — Full round-trip: a single-slide LaidOutDeck with a
/// gradient shape must produce a valid PPTX ZIP without errors.
///
/// RED GATE: the current serializer silently skips Shape frames, so the PPTX
/// is valid but contains no gradient element (test_BC..._emits_grad_fill_element
/// catches that separately). After implementation, export must succeed AND emit
/// the gradient element.
///
/// This test only checks the "no error on export" gate (the Shape frame is
/// currently silently skipped, so it already passes if the Shape skip path is
/// maintained). This test becomes load-bearing only when the gradient shape
/// is wired through. Mark as a pre-condition test for the series.
#[test]
fn test_BC_3_04_001_ac004_pptx_full_slide_with_gradient_shape_builds() {
    let laid_out = gradient_slide_deck(
        Rgb { r: 255, g: 0, b: 0 },
        Rgb { r: 0, g: 0, b: 255 },
        AltText::Provided(Arc::from("Gradient background")),
    );
    // Should not panic or return Err — the exporter handles FillSpec::Gradient.
    let deck = make_deck_one_slide();
    let brand = make_brand();
    let opts = ExportOptions::default();
    let exporter = PptxExporter::new();
    let result = exporter.export(&deck, &laid_out, &brand, &opts);
    assert!(
        result.is_ok(),
        "PptxExporter::export must not fail for a deck containing a FillSpec::Gradient shape; \
         err: {:?}",
        result.err()
    );
}
