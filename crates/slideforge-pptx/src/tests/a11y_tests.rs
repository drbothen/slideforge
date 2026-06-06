//! Tests for STORY-039: PPTX Accessibility Metadata.
//!
//! Covers BC-4.01.004 (alt text embedding) and BC-5.01.005 (lang propagation
//! to PPTX Core Properties). All tests pass in Green state — implementation is
//! complete. Tests parse REAL serialized PPTX output — no mock strings.
//!
//! STORY-039 scope expansion (human-authorized 2026-06-03):
//! AC-005 uses REAL FrameContent::Chart/Diagram frames (not Image proxies — F-039-C2 fix).
//! EC-006/EC-007 test the Decorative-sentinel path (ChartSpec.alt = None / DiagramSpec.alt = None).
//!
//! ## Traceability
//!
//! | Test function | AC | BC clause | What is verified |
//! |---|---|---|---|
//! | `test_BC_4_01_004_ac001_non_decorative_image_has_non_empty_descr` | AC-001 | postcondition 1 | `build_image_picture` sets descr from AltTextEmbedder |
//! | `test_BC_4_01_004_ac002_decorative_image_has_empty_descr_attribute_present` | AC-002 | postcondition 2 | `build_image_picture` sets descr="" for Decorative |
//! | `test_BC_4_01_004_ac003_300_char_alt_not_truncated` | AC-003 | invariant 1 | 300-char alt text embedded without truncation |
//! | `test_BC_4_01_004_ac004_special_chars_xml_escaped_well_formed` | AC-004 | EC-001 | XML special chars escaped; slide XML is well-formed |
//! | `test_BC_4_01_004_ac005_chart_frame_alt_on_enclosing_shape` | AC-005 | EC-005 | Chart frame alt threaded to enclosing shape cNvPr |
//! | `test_BC_4_01_004_ac005_diagram_frame_alt_on_enclosing_shape` | AC-005 | EC-005 | Diagram frame alt threaded to enclosing shape cNvPr |
//! | `test_BC_5_01_005_ac006_dc_language_exact_bcp47_en_us` | AC-006 | postcondition 1 | dc:language = "en-US" exact BCP-47 |
//! | `test_BC_5_01_005_ac006_dc_language_exact_bcp47_zh_hant_tw` | AC-006 | postcondition 1 | dc:language = "zh-Hant-TW" exact BCP-47 |
//! | `test_BC_5_01_005_ac007_no_lang_defaults_to_en` | AC-007 | EC-003 | No lang declaration defaults to "en" |
//! | `test_BC_4_01_004_ec001_all_decorative_slide` | EC-001 | postcondition 2 | All decorative frames produce descr="" |
//! | `test_BC_4_01_004_ec002_zh_tw_lang_bcp47_embedded` | EC-002 | EC-003 | zh-Hant-TW lang embedded unchanged |
//! | `test_BC_4_01_004_ec005_300_char_alt_exact_length` | EC-005 | invariant 1 | 300-char descr exact length verified |
//! | `test_BC_4_01_004_ec006_chart_none_alt_maps_to_decorative_descr_empty` | EC-006 | EC-006 | Chart AltText::Decorative produces descr="" |
//! | `test_BC_4_01_004_ec007_diagram_none_alt_maps_to_decorative_descr_empty` | EC-007 | EC-007 | Diagram AltText::Decorative produces descr="" |

#![allow(non_snake_case)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use std::io::Read as _;
use std::sync::Arc;

use quick_xml::Reader;
use quick_xml::events::Event;
use slideforge_layout::{BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize};
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::{
    Brand, BrandFonts, BrandPalette, Deck, Emu, NormalizedDiagramSvg, SourceSpan,
};
use zip::ZipArchive;

use crate::PptxExporter;

// ─── Fixture builders ────────────────────────────────────────────────────────

fn make_metadata_with_lang(lang: &str) -> slideforge_types::deck::DeckMetadata {
    use slideforge_types::deck::DeckMetadata;
    DeckMetadata {
        title: Some(Arc::from("A11y Test Deck")),
        slideforge_version: Arc::from("0.1.0"),
        lang: Some(Arc::from(lang)),
        author: None,
        section_order: None,
    }
}

fn make_metadata_no_lang() -> slideforge_types::deck::DeckMetadata {
    use slideforge_types::deck::DeckMetadata;
    DeckMetadata {
        title: Some(Arc::from("A11y Test Deck")),
        slideforge_version: Arc::from("0.1.0"),
        lang: None,
        author: None,
        section_order: None,
    }
}

fn make_deck_with_lang(lang: &str) -> Deck {
    use slideforge_types::ordered_map::OrderedMap;
    use slideforge_types::slide::Slide;
    Deck {
        slides: vec![Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }],
        vars: OrderedMap::new(),
        metadata: make_metadata_with_lang(lang),
        registers: OrderedMap::new(),
        section_blocks: vec![],
    }
}

fn make_deck_no_lang() -> Deck {
    use slideforge_types::ordered_map::OrderedMap;
    use slideforge_types::slide::Slide;
    Deck {
        slides: vec![Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }],
        vars: OrderedMap::new(),
        metadata: make_metadata_no_lang(),
        registers: OrderedMap::new(),
        section_blocks: vec![],
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
        },
        layouts: vec![],
        span: SourceSpan::default(),
    }
}

fn title_bbox() -> BoundingBox {
    BoundingBox {
        x: Emu(457_200),
        y: Emu(274_638),
        width: Emu(8_229_600),
        height: Emu(1_143_000),
    }
}

/// Build a `LaidOutSlide` with one `FrameContent::Image { alt: AltText::Provided }` frame.
fn make_slide_with_image_alt(index: usize, alt: &str) -> LaidOutSlide {
    use slideforge_types::AltText;
    LaidOutSlide {
        source_index: index,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Image {
                alt: AltText::Provided(Arc::from(alt)),
            },
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    }
}

/// Build a `LaidOutSlide` with one `FrameContent::Image { alt: AltText::Decorative }` frame
/// to simulate a decorative image.
///
/// STORY-039: AltText::Decorative is the explicit opt-out marker (not empty Arc<str>).
/// The PPTX exporter emits `descr=""` for decorative frames (attribute present, empty value).
fn make_slide_with_decorative_image(index: usize) -> LaidOutSlide {
    use slideforge_types::AltText;
    LaidOutSlide {
        source_index: index,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Image {
                alt: AltText::Decorative,
            },
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    }
}

/// Build a minimal `LaidOutDeck` with one slide containing an image with alt text.
fn make_laid_out_deck_with_image_alt(alt: &str) -> LaidOutDeck {
    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![make_slide_with_image_alt(0, alt)],
        sections: vec![],
        warnings: vec![],
    }
}

/// Build a minimal `LaidOutDeck` with one slide containing a decorative image.
fn make_laid_out_deck_with_decorative_image() -> LaidOutDeck {
    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![make_slide_with_decorative_image(0)],
        sections: vec![],
        warnings: vec![],
    }
}

/// Build a minimal `LaidOutDeck` with a single title-only slide (no visual frames).
fn make_laid_out_deck_with_lang_only() -> LaidOutDeck {
    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("title"),
            frames: vec![Frame {
                bbox: title_bbox(),
                content: FrameContent::Title(Arc::from("Lang Test Slide")),
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

/// Build a `LaidOutDeck` with one slide containing a REAL `FrameContent::Chart` frame (AC-005).
///
/// STORY-039: AC-005 requires REAL Chart/Diagram frames — NOT Image proxies (F-039-C2).
/// The alt text is `AltText::Provided(...)` as threaded from `ChartSpec.alt`.
///
/// `slide_serializer.rs` routes `Chart { alt }` frames through `AltTextEmbedder`
/// and emits `descr` on the enclosing `<p:grpSp>`. The accompanying test guards
/// against regressions that would skip Chart frames or omit the `descr` attribute.
fn make_laid_out_deck_with_chart_frame(alt: &str) -> LaidOutDeck {
    use slideforge_types::AltText;
    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("chart"),
            frames: vec![Frame {
                bbox: title_bbox(),
                // REAL FrameContent::Chart with Provided alt text.
                // This is the CORRECT fixture for AC-005 (not an Image proxy — F-039-C2).
                content: FrameContent::Chart {
                    alt: AltText::Provided(Arc::from(alt)),
                },
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

/// Build a `LaidOutDeck` with one slide containing a REAL `FrameContent::Diagram` frame (AC-005).
///
/// STORY-039: AC-005 requires REAL Diagram frames — NOT Image proxies (F-039-C2).
/// The alt text is `AltText::Provided(...)` as threaded from `DiagramSpec.alt`.
fn make_laid_out_deck_with_diagram_frame(alt: &str) -> LaidOutDeck {
    use slideforge_types::AltText;
    // Minimal valid SVG for the frame payload.
    let svg = Arc::from(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300" aria-label="diagram" role="img"><title>diagram</title></svg>"#,
    );
    let normalized = NormalizedDiagramSvg::from_normalized_string(svg);
    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("diagram"),
            frames: vec![Frame {
                bbox: title_bbox(),
                // REAL FrameContent::Diagram with Provided alt text.
                // Alt text threaded from DiagramSpec.alt (STORY-039 IR threading).
                content: FrameContent::Diagram {
                    svg: normalized,
                    alt: AltText::Provided(Arc::from(alt)),
                },
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

/// Build a `LaidOutDeck` with one slide containing a `FrameContent::Chart`
/// where `ChartSpec.alt = None` was mapped to `AltText::Decorative` (EC-006).
///
/// The test asserts that when `alt = AltText::Decorative`, the PPTX emits `descr=""`
/// (not absent). This tests the Decorative-distinction code path in `AltTextEmbedder`
/// and guards against regressions that would omit the `descr` attribute entirely.
fn make_laid_out_deck_with_chart_none_alt() -> LaidOutDeck {
    use slideforge_types::AltText;
    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("chart"),
            frames: vec![Frame {
                bbox: title_bbox(),
                // AltText::Decorative = what layout::run produces when ChartSpec.alt = None
                // (safe sentinel + tracing::warn! per EC-006).
                content: FrameContent::Chart {
                    alt: AltText::Decorative,
                },
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

/// Build a `LaidOutDeck` with one slide containing a `FrameContent::Diagram`
/// where `DiagramSpec.alt = None` was mapped to `AltText::Decorative` (EC-007).
fn make_laid_out_deck_with_diagram_none_alt() -> LaidOutDeck {
    use slideforge_types::AltText;
    let svg = Arc::from(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"><title>empty</title></svg>"#,
    );
    let normalized = NormalizedDiagramSvg::from_normalized_string(svg);
    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("diagram"),
            frames: vec![Frame {
                bbox: title_bbox(),
                // AltText::Decorative = what layout::run produces when DiagramSpec.alt = None
                // (safe sentinel + tracing::warn! per EC-007).
                content: FrameContent::Diagram {
                    svg: normalized,
                    alt: AltText::Decorative,
                },
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

// ─── ZIP helpers ──────────────────────────────────────────────────────────────

/// Read a named entry from a PPTX ZIP as a UTF-8 string.
fn zip_read_entry(pptx_bytes: &[u8], path: &str) -> String {
    let cursor = std::io::Cursor::new(pptx_bytes);
    let mut archive = ZipArchive::new(cursor).expect("must be a valid ZIP");
    let mut entry = archive
        .by_name(path)
        .unwrap_or_else(|_| panic!("entry '{path}' must exist in the PPTX ZIP"));
    let mut buf = String::new();
    entry
        .read_to_string(&mut buf)
        .expect("entry must be valid UTF-8");
    buf
}

/// Check that the given XML string is well-formed (parseable by quick-xml).
///
/// Returns `Ok(())` if the XML is valid, `Err(msg)` if it cannot be parsed.
fn assert_xml_well_formed(xml: &str) -> Result<(), String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => return Ok(()),
            Ok(_) => {},
            Err(e) => return Err(format!("XML parse error: {e}")),
        }
        buf.clear();
    }
}

/// Extract the value of `attr_name` from a `<tag attr_name="VALUE">` element
/// in the given XML string. Returns `None` if the attribute is not found.
///
/// This is a simple scan, not a full XML parse; it works for extracting
/// well-known attributes from OOXML element serializations.
fn find_attr_value_in_xml(xml: &str, tag: &str, attr_name: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e) | Event::Empty(e)) => {
                // Check if this element's local name matches `tag`.
                let local_name = e.local_name();
                if local_name.as_ref() == tag.as_bytes() {
                    // Scan attributes for `attr_name`.
                    for attr in e.attributes().flatten() {
                        let key = attr.key.local_name();
                        if key.as_ref() == attr_name.as_bytes() {
                            return Some(
                                String::from_utf8(attr.value.into_owned())
                                    .expect("attribute value must be UTF-8"),
                            );
                        }
                    }
                }
            },
            Ok(Event::Eof) => return None,
            _ => {},
        }
        buf.clear();
    }
}

/// Extract the text content of the first element matching `tag` in `xml`.
///
/// Used to read `<dc:language>VALUE</dc:language>` from core.xml.
fn find_element_text(xml: &str, tag: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_target = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let local_name = e.local_name();
                if local_name.as_ref() == tag.as_bytes() {
                    in_target = true;
                }
            },
            Ok(Event::Text(t)) if in_target => {
                return Some(
                    t.unescape()
                        .expect("text must be valid XML escaped")
                        .into_owned(),
                );
            },
            Ok(Event::End(_)) => {
                in_target = false;
            },
            Ok(Event::Eof) => return None,
            _ => {},
        }
        buf.clear();
    }
}

/// Collect ALL `descr` attribute values from `<p:cNvPr>` (or `cNvPr`) elements in `xml`.
///
/// Each `<p:cNvPr descr="...">` element contributes one entry. Returns empty
/// vec if no `cNvPr` elements are found.
fn collect_cnvpr_descr_values(xml: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut descr_values: Vec<String> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e) | Event::Empty(e)) => {
                let local_name = e.local_name();
                // Match "cNvPr" (common non-visual drawing properties)
                // regardless of namespace prefix (p:cNvPr, a:cNvPr, etc.).
                if local_name.as_ref() == b"cNvPr" {
                    for attr in e.attributes().flatten() {
                        let key = attr.key.local_name();
                        if key.as_ref() == b"descr" {
                            descr_values.push(
                                String::from_utf8(attr.value.into_owned())
                                    .expect("descr value must be UTF-8"),
                            );
                        }
                    }
                }
            },
            Ok(Event::Eof) => break,
            _ => {},
        }
        buf.clear();
    }
    descr_values
}

/// Collect all `name` attribute values from `<*:cNvPr name="...">` elements in `xml`.
///
/// Mirrors `collect_cnvpr_descr_values` but reads the `name` attribute instead.
/// Used to assert that chart shapes carry `name="Chart N"` rather than `name="Image N"`
/// (F-039-P-LOW-001: Selection Pane / accessibility-tree label correctness).
fn collect_cnvpr_name_values(xml: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut name_values: Vec<String> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e) | Event::Empty(e)) => {
                let local_name = e.local_name();
                // Match "cNvPr" regardless of namespace prefix.
                if local_name.as_ref() == b"cNvPr" {
                    for attr in e.attributes().flatten() {
                        let key = attr.key.local_name();
                        if key.as_ref() == b"name" {
                            name_values.push(
                                String::from_utf8(attr.value.into_owned())
                                    .expect("name value must be UTF-8"),
                            );
                        }
                    }
                }
            },
            Ok(Event::Eof) => break,
            _ => {},
        }
        buf.clear();
    }
    name_values
}

/// Run `PptxExporter::export` with the given deck + laid_out and return the bytes.
///
/// Panics with a descriptive message if the exporter returns an error.
fn build_pptx(deck: &Deck, laid_out: &LaidOutDeck) -> Vec<u8> {
    let brand = make_brand();
    let opts = ExportOptions::default();
    let exporter = PptxExporter::new();
    exporter
        .export(deck, laid_out, &brand, &opts)
        .expect("PptxExporter::export must succeed for a11y tests")
}

// ─── AC-001: Non-decorative image → non-empty descr ──────────────────────────

/// Exercises VP-TBD: All non-decorative picture shapes have non-empty descr.
///
/// BC-4.01.004 postcondition 1:
/// Every non-decorative image, chart, or diagram shape in the PPTX has a
/// non-empty `descr` attribute on `<p:cNvPr>` containing the DSL `alt "..."`
/// value.
///
/// Test vector (BC-4.01.004 canonical):
///   Input: Image with `alt "Revenue chart Q1 2026"`
///   Expected: `<p:cNvPr descr="Revenue chart Q1 2026">`
#[test]
fn test_BC_4_01_004_ac001_non_decorative_image_has_non_empty_descr() {
    let alt_text = "Revenue chart Q1 2026";
    let deck = make_deck_with_lang("en-US");
    let laid_out = make_laid_out_deck_with_image_alt(alt_text);

    let pptx_bytes = build_pptx(&deck, &laid_out);

    // Parse slide1.xml from the ZIP.
    let slide_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    // Collect all cNvPr descr values in the slide.
    let descr_values = collect_cnvpr_descr_values(&slide_xml);

    // AC-001: At least one non-empty descr matching the alt text must be present.
    assert!(
        descr_values.iter().any(|d| d == alt_text),
        "slide1.xml must contain a <p:cNvPr descr=\"{alt_text}\"> element \
         (BC-4.01.004 postcondition 1). Found descr values: {descr_values:?}"
    );

    // AC-001: No non-decorative shape may have an empty descr.
    let non_empty_descrs: Vec<&String> = descr_values.iter().filter(|d| !d.is_empty()).collect();
    assert!(
        !non_empty_descrs.is_empty(),
        "slide1.xml must have at least one non-empty descr attribute \
         (BC-4.01.004 postcondition 1); found only: {descr_values:?}"
    );
}

// ─── AC-002: Decorative image → descr="" (attribute present, empty) ───────────

/// BC-4.01.004 postcondition 2:
/// Every decorative element has `descr=""` on `<p:cNvPr>`. The attribute MUST
/// be present with an empty value — not absent.
///
/// Test vector (BC-4.01.004 canonical):
///   Input: Image with `decorative: true`
///   Expected: `<p:cNvPr descr="">`
#[test]
fn test_BC_4_01_004_ac002_decorative_image_has_empty_descr_attribute_present() {
    let deck = make_deck_with_lang("en-US");
    let laid_out = make_laid_out_deck_with_decorative_image();

    let pptx_bytes = build_pptx(&deck, &laid_out);
    let slide_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    let descr_values = collect_cnvpr_descr_values(&slide_xml);

    // AC-002: The decorative frame's cNvPr must have descr="" (empty string value).
    // The attribute MUST BE PRESENT (not absent) — that is why we check for the
    // empty string in the collected descr_values rather than for absence.
    assert!(
        descr_values.iter().any(String::is_empty),
        "slide1.xml must contain a <p:cNvPr descr=\"\"> element (empty value, attribute present) \
         for the decorative frame (BC-4.01.004 postcondition 2). \
         Found descr values: {descr_values:?}"
    );
}

// ─── AC-003: 300-char alt text not truncated ──────────────────────────────────

/// BC-4.01.004 invariant 1: Alt text values are NEVER truncated.
///
/// An alt text value of 300 characters must be embedded in full.
/// The `descr` attribute value length must equal the input length.
#[test]
fn test_BC_4_01_004_ac003_300_char_alt_not_truncated() {
    let alt_300 = "A".repeat(300);
    let deck = make_deck_with_lang("en-US");
    let laid_out = make_laid_out_deck_with_image_alt(&alt_300);

    let pptx_bytes = build_pptx(&deck, &laid_out);
    let slide_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    let descr_values = collect_cnvpr_descr_values(&slide_xml);

    assert!(
        descr_values.iter().any(|d| d == &alt_300),
        "slide1.xml must contain the full 300-character alt text in a descr attribute \
         (BC-4.01.004 invariant 1 — never truncated). \
         Found descr values (first 50 chars each): {:?}",
        descr_values
            .iter()
            .map(|d| &d[..d.len().min(50)])
            .collect::<Vec<_>>()
    );

    let found_len = descr_values.iter().filter(|d| d.len() == 300).count();
    assert_eq!(
        found_len, 1,
        "exactly one descr must carry the full 300-character string; found {found_len}"
    );
}

// ─── AC-004: Special characters XML-escaped ───────────────────────────────────

/// BC-4.01.004 EC-001: Alt text with XML special chars must be embedded correctly.
///
/// An alt text value of `Revenue & Cost "Q1" <2026> 'fin'` must be embedded
/// such that the produced slide XML is well-formed (parseable without error).
/// `ooxmlsdk` handles escaping automatically; this test verifies the pipeline
/// does not break on special characters.
///
/// Test vector (BC-4.01.004 canonical):
///   Input: `alt "Revenue & Cost"` (ampersand)
///   Expected: well-formed XML with `descr="Revenue &amp; Cost"`
#[test]
fn test_BC_4_01_004_ac004_special_chars_xml_escaped_well_formed() {
    let alt_with_specials = r#"Revenue & Cost "Q1" <2026> 'fin'"#;
    let deck = make_deck_with_lang("en-US");
    let laid_out = make_laid_out_deck_with_image_alt(alt_with_specials);

    let pptx_bytes = build_pptx(&deck, &laid_out);
    let slide_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    // Primary assertion: the slide XML must be well-formed (parses without error).
    assert_xml_well_formed(&slide_xml).unwrap_or_else(|msg| {
        panic!(
            "slide1.xml is NOT well-formed XML after embedding alt text with special chars \
             (BC-4.01.004 EC-001). Error: {msg}\nSlide XML (first 500 chars):\n{}",
            &slide_xml[..slide_xml.len().min(500)]
        )
    });

    // Secondary assertion: a cNvPr element with a non-empty descr must be present.
    let descr_values = collect_cnvpr_descr_values(&slide_xml);
    assert!(
        descr_values.iter().any(|d| !d.is_empty()),
        "slide1.xml must contain at least one non-empty descr attribute after embedding \
         special-char alt text. Found: {descr_values:?}"
    );
}

// ─── AC-005: Chart/diagram alt on enclosing shape (REAL frames, not Image proxies) ────

/// BC-4.01.004 EC-005: A chart frame's alt text goes on the enclosing shape's `<p:cNvPr>`.
///
/// This test uses a REAL `FrameContent::Chart { alt: AltText::Provided(...) }` frame
/// (not an Image proxy — F-039-C2 fix). The alt text is threaded from `ChartSpec.alt`
/// via the IR.
///
/// `build_shape_tree` in `slide_serializer.rs` routes `FrameContent::Chart { alt }` through
/// `AltTextEmbedder` and emits a `<p:grpSp>` or `<p:pic>` shape with the `descr` attribute
/// set from the alt decision. This test guards against regressions that would cause Chart
/// frames to be skipped or the `descr` attribute to be omitted.
#[test]
fn test_BC_4_01_004_ac005_chart_frame_alt_on_enclosing_shape() {
    let alt_text = "Bar chart: Q1 revenue by region";
    let deck = make_deck_with_lang("en-US");
    // REAL FrameContent::Chart { alt: AltText::Provided } — NOT an Image proxy.
    let laid_out = make_laid_out_deck_with_chart_frame(alt_text);

    let pptx_bytes = build_pptx(&deck, &laid_out);
    let slide_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    // The cNvPr on the enclosing group shape (p:grpSp or p:pic) must carry the alt text.
    // Regression guard: `slide_serializer.rs` routes Chart { alt } through AltTextEmbedder
    // and emits descr on the enclosing shape. Any regression that skips Chart frames
    // will be caught here.
    let descr_values = collect_cnvpr_descr_values(&slide_xml);

    assert!(
        descr_values.iter().any(|d| d == alt_text),
        "slide1.xml must have descr=\"{alt_text}\" on the enclosing <p:grpSp>/<p:pic> \
         shape for the FrameContent::Chart frame (BC-4.01.004 EC-005). \
         Found descr values: {descr_values:?}"
    );

    // F-039-P-LOW-001: The cNvPr `name` attribute for a chart shape must start with
    // "Chart", not "Image". PowerPoint Selection Pane and OOXML accessibility trees
    // display this label — "Image N" is misleading for chart shapes.
    // The `descr` attribute must be unaffected (asserted above).
    let name_values = collect_cnvpr_name_values(&slide_xml);
    assert!(
        name_values.iter().any(|n| n.starts_with("Chart")),
        "slide1.xml must have name=\"Chart N\" (starts with \"Chart\") on the enclosing \
         <p:cNvPr> for the FrameContent::Chart frame (F-039-P-LOW-001). \
         Got name values: {name_values:?}"
    );
}

/// BC-4.01.004 EC-005 (Diagram variant): A diagram frame's alt text goes on the
/// enclosing shape's `<p:cNvPr>`.
///
/// This test uses a REAL `FrameContent::Diagram { svg, alt: AltText::Provided(...) }` frame
/// (not an Image proxy — F-039-C2 fix). The alt text is threaded from `DiagramSpec.alt`.
///
/// `slide_serializer.rs` calls `build_picture` for Diagram frames and passes `alt` through
/// to `<p:cNvPr descr=...>`. This test guards against regressions that would cause
/// `build_picture` to revert to setting `description: None` and omit the `descr` attribute.
#[test]
fn test_BC_4_01_004_ac005_diagram_frame_alt_on_enclosing_shape() {
    let alt_text = "Flowchart: Q1 deployment pipeline steps";
    let deck = make_deck_with_lang("en-US");
    // REAL FrameContent::Diagram { svg, alt: AltText::Provided } — NOT an Image proxy.
    let laid_out = make_laid_out_deck_with_diagram_frame(alt_text);

    let pptx_bytes = build_pptx(&deck, &laid_out);
    let slide_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    // The cNvPr on the enclosing <p:pic> for the diagram must carry the alt text.
    // Regression guard: `build_picture` passes `alt` to the cNvPr description field.
    // Any regression that reverts description back to None will be caught here.
    let descr_values = collect_cnvpr_descr_values(&slide_xml);

    assert!(
        descr_values.iter().any(|d| d == alt_text),
        "slide1.xml must have descr=\"{alt_text}\" on the enclosing <p:pic> shape \
         for the FrameContent::Diagram frame (BC-4.01.004 EC-005). \
         Found descr values: {descr_values:?}"
    );
}

// ─── AC-006: dc:language exact BCP-47 (en-US) ────────────────────────────────

/// Exercises VP-TBD: PPTX dc:language matches deck lang declaration exactly.
///
/// BC-5.01.005 postcondition 1:
/// PPTX `docProps/core.xml` contains `<dc:language>en-US</dc:language>` when
/// the deck has `lang "en-US"`.
///
/// Test vector (BC-5.01.005 canonical):
///   Input: `lang "en-US"` → PPTX export
///   Expected: `<dc:language>en-US</dc:language>` in core.xml
#[test]
fn test_BC_5_01_005_ac006_dc_language_exact_bcp47_en_us() {
    let deck = make_deck_with_lang("en-US");
    let laid_out = make_laid_out_deck_with_lang_only();

    let pptx_bytes = build_pptx(&deck, &laid_out);
    let core_xml = zip_read_entry(&pptx_bytes, "docProps/core.xml");

    // BC-5.01.005 postcondition 1: exact BCP-47 tag, no transformation.
    let lang_value = find_element_text(&core_xml, "language").unwrap_or_else(|| {
        panic!(
            "docProps/core.xml must contain a <dc:language> element \
             (BC-5.01.005 postcondition 1). core.xml content:\n{core_xml}"
        )
    });

    assert_eq!(
        lang_value, "en-US",
        "dc:language in core.xml must be exactly \"en-US\" — no normalization \
         or case change (BC-5.01.005 invariant 1). Got: {lang_value:?}"
    );
}

/// BC-5.01.005 postcondition 1 + invariant 1:
/// A deck with `lang "zh-Hant-TW"` (4-part BCP-47) must produce
/// `<dc:language>zh-Hant-TW</dc:language>` — unchanged, no case folding.
///
/// Test vector (BC-5.01.005 EC-002):
///   Input: `lang "zh-Hant-TW"` (4-part BCP-47)
///   Expected: all formats embed "zh-Hant-TW" unchanged
#[test]
fn test_BC_5_01_005_ac006_dc_language_exact_bcp47_zh_hant_tw() {
    let deck = make_deck_with_lang("zh-Hant-TW");
    let laid_out = make_laid_out_deck_with_lang_only();

    let pptx_bytes = build_pptx(&deck, &laid_out);
    let core_xml = zip_read_entry(&pptx_bytes, "docProps/core.xml");

    let lang_value = find_element_text(&core_xml, "language").unwrap_or_else(|| {
        panic!(
            "docProps/core.xml must contain a <dc:language> element \
             (BC-5.01.005 postcondition 1). core.xml:\n{core_xml}"
        )
    });

    assert_eq!(
        lang_value, "zh-Hant-TW",
        "dc:language must be exactly \"zh-Hant-TW\" — lossless, no case change, no truncation \
         (BC-5.01.005 invariant 1). Got: {lang_value:?}"
    );
}

// ─── AC-007: No lang → default "en" ──────────────────────────────────────────

/// BC-5.01.005 EC-003:
/// A deck with no `lang` declaration has `deck.metadata.lang = None`, which the
/// evaluator resolves to "en" (set by the caller of `PptxExporter::export` per
/// BC-5.01.004). The PPTX must embed `<dc:language>en</dc:language>`.
///
/// Test vector (BC-5.01.005 canonical):
///   Input: No lang declared → any export
///   Expected: All formats embed "en" (default)
#[test]
fn test_BC_5_01_005_ac007_no_lang_defaults_to_en() {
    let deck = make_deck_no_lang();
    let laid_out = make_laid_out_deck_with_lang_only();

    let pptx_bytes = build_pptx(&deck, &laid_out);
    let core_xml = zip_read_entry(&pptx_bytes, "docProps/core.xml");

    let lang_value = find_element_text(&core_xml, "language").unwrap_or_else(|| {
        panic!(
            "docProps/core.xml must contain a <dc:language> element even when deck has no lang \
             declaration (BC-5.01.005 EC-003). core.xml:\n{core_xml}"
        )
    });

    assert_eq!(
        lang_value, "en",
        "dc:language must default to \"en\" (not \"en-US\") when no lang is declared \
         (BC-5.01.005 EC-003 / BC-5.01.004 default). Got: {lang_value:?}"
    );
}

// ─── EC-001: All images on slide decorative ───────────────────────────────────

/// BC-4.01.004 EC-004 (story mapping EC-001):
/// When ALL images on a slide are decorative, every `<p:cNvPr>` for visual
/// frames must have `descr=""`. No violations (non-empty descr on a decorative
/// frame) should appear.
#[test]
fn test_BC_4_01_004_ec001_all_decorative_slide() {
    let deck = make_deck_with_lang("en-US");
    let laid_out = make_laid_out_deck_with_decorative_image();

    let pptx_bytes = build_pptx(&deck, &laid_out);
    let slide_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    let descr_values = collect_cnvpr_descr_values(&slide_xml);

    // EC-001: all visual frame descr values must be empty (decorative).
    // Non-empty descr on a decorative frame is a violation.
    for descr in &descr_values {
        assert!(
            descr.is_empty(),
            "All frames are decorative — descr must be empty for each cNvPr. \
             Found non-empty descr: {descr:?} (BC-4.01.004 postcondition 2 / EC-004)"
        );
    }
    // Must have at least one cNvPr with descr="" present.
    assert!(
        !descr_values.is_empty(),
        "At least one cNvPr with descr=\"\" must be present for the decorative frame. \
         Got: {descr_values:?}"
    );
}

// ─── EC-002: zh-Hant-TW lang (BCP-47 4-part) ─────────────────────────────────

/// BC-4.01.004 EC-003 (story mapping EC-002):
/// `lang "zh-Hant-TW"` is embedded unchanged in dc:language.
/// This is verified by `test_BC_5_01_005_ac006_dc_language_exact_bcp47_zh_hant_tw`
/// above. This test exercises it from the AC angle (checking the slide XML is
/// also well-formed when the lang tag is non-ASCII BCP-47).
#[test]
fn test_BC_4_01_004_ec002_zh_tw_lang_bcp47_embedded() {
    let deck = make_deck_with_lang("zh-Hant-TW");
    let laid_out = make_laid_out_deck_with_lang_only();

    let pptx_bytes = build_pptx(&deck, &laid_out);

    // Verify core.xml is well-formed when lang is "zh-Hant-TW".
    let core_xml = zip_read_entry(&pptx_bytes, "docProps/core.xml");
    assert_xml_well_formed(&core_xml).unwrap_or_else(|msg| {
        panic!(
            "docProps/core.xml must be well-formed XML even for zh-Hant-TW lang. \
             Error: {msg}\ncore.xml:\n{core_xml}"
        )
    });

    let lang_value = find_element_text(&core_xml, "language")
        .unwrap_or_else(|| panic!("dc:language element missing in core.xml for zh-Hant-TW test"));
    assert_eq!(
        lang_value, "zh-Hant-TW",
        "dc:language must be 'zh-Hant-TW' unchanged (BC-5.01.005 EC-002)"
    );
}

// ─── EC-005: 300-character alt text exact length ──────────────────────────────

/// BC-4.01.004 EC-005 (story mapping: AC-003 / invariant 1):
/// A 300-character alt text is embedded verbatim — descr attribute length
/// equals input length exactly. OOXML schema permits long `descr` strings.
///
/// This test is distinct from AC-003: it checks the EXACT character count
/// (not just contains the string) and also verifies the slide XML remains
/// well-formed after embedding.
#[test]
fn test_BC_4_01_004_ec005_300_char_alt_exact_length() {
    // 300-char sentinel with varied characters to avoid naive truncation detection.
    let alt_300: String = (0_u32..300)
        .map(|i| {
            // Mix of safe ASCII chars: letters, digits, spaces.
            match i % 10 {
                0..=3 => char::from_u32(u32::from(b'A') + i % 26).unwrap(),
                4..=6 => char::from_u32(u32::from(b'a') + i % 26).unwrap(),
                7..=8 => char::from_u32(u32::from(b'0') + i % 10).unwrap(),
                _ => ' ',
            }
        })
        .collect();
    assert_eq!(alt_300.len(), 300, "test fixture must be exactly 300 chars");

    let deck = make_deck_with_lang("en-US");
    let laid_out = make_laid_out_deck_with_image_alt(&alt_300);

    let pptx_bytes = build_pptx(&deck, &laid_out);
    let slide_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    // Slide XML must remain well-formed after embedding 300-char alt.
    assert_xml_well_formed(&slide_xml).unwrap_or_else(|msg| {
        panic!(
            "slide1.xml must be well-formed XML after embedding 300-char alt text. \
             Error: {msg}"
        )
    });

    // The descr attribute must carry the full 300-char string (not truncated).
    let descr_values = collect_cnvpr_descr_values(&slide_xml);
    let matching: Vec<&String> = descr_values.iter().filter(|d| d == &&alt_300).collect();
    assert_eq!(
        matching.len(),
        1,
        "exactly one cNvPr descr must carry the full 300-char string \
         (BC-4.01.004 invariant 1). Found descr lengths: {:?}",
        descr_values.iter().map(String::len).collect::<Vec<_>>()
    );
}

// ─── EC-006: ChartSpec.alt = None → AltText::Decorative + descr="" ───────────

/// BC-4.01.004 EC-006 (amended story spec):
/// When `ChartSpec.alt = None` reaches the layout engine (upstream validator miss),
/// `layout::run` must map it to `AltText::Decorative` and emit `tracing::warn!`.
///
/// This test verifies the PPTX output side of that contract: a
/// `FrameContent::Chart { alt: AltText::Decorative }` frame must produce
/// `descr=""` on its enclosing shape (attribute PRESENT with empty value, NOT absent).
///
/// `slide_serializer.rs` routes `Chart { alt: AltText::Decorative }` through
/// `AltTextEmbedder` and emits `descr=""` (present, empty) on the enclosing
/// `<p:grpSp>/<p:pic>` cNvPr. This test guards against regressions that would
/// skip Chart frames entirely or emit the `descr` attribute as absent rather than
/// empty.
#[test]
fn test_BC_4_01_004_ec006_chart_none_alt_maps_to_decorative_descr_empty() {
    let deck = make_deck_with_lang("en-US");
    // Chart frame with AltText::Decorative (simulates ChartSpec.alt = None path).
    let laid_out = make_laid_out_deck_with_chart_none_alt();

    let pptx_bytes = build_pptx(&deck, &laid_out);
    let slide_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    // Assertion: a cNvPr with descr="" must be present (Decorative = empty descr).
    // The attribute MUST be present (not absent) — OOXML accessibility contract.
    //
    // Regression guard: slide_serializer.rs routes Chart { AltText::Decorative } through
    // AltTextEmbedder and emits descr="". Any regression that skips Chart frames or
    // omits the descr attribute will cause collect_cnvpr_descr_values to return an
    // empty vec, failing the assertion.
    let descr_values = collect_cnvpr_descr_values(&slide_xml);
    assert!(
        descr_values.iter().any(String::is_empty),
        "slide1.xml must contain <p:cNvPr descr=\"\"> for a Chart frame with \
         AltText::Decorative (EC-006 — ChartSpec.alt = None safe-sentinel path). \
         The descr attribute MUST be present with empty value. \
         Found descr values: {descr_values:?}"
    );
}

// ─── EC-007: DiagramSpec.alt = None → AltText::Decorative + descr="" ─────────

/// BC-4.01.004 EC-007 (amended story spec):
/// When `DiagramSpec.alt = None` reaches the layout engine (upstream validator miss),
/// `layout::run` must map it to `AltText::Decorative` and emit `tracing::warn!`.
///
/// This test verifies the PPTX output side: a
/// `FrameContent::Diagram { svg, alt: AltText::Decorative }` frame must produce
/// `descr=""` on its enclosing `<p:pic>` shape.
///
/// `build_picture` in `slide_serializer.rs` passes `AltText::Decorative` through
/// to the `description` field as `Some("".to_owned())` so that the `descr` attribute
/// is present (even if empty) for decorative frames. This test guards against
/// regressions that would revert `description` to `None` and omit the `descr` attribute.
#[test]
fn test_BC_4_01_004_ec007_diagram_none_alt_maps_to_decorative_descr_empty() {
    let deck = make_deck_with_lang("en-US");
    // Diagram frame with AltText::Decorative (simulates DiagramSpec.alt = None path).
    let laid_out = make_laid_out_deck_with_diagram_none_alt();

    let pptx_bytes = build_pptx(&deck, &laid_out);
    let slide_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    // Assertion: a cNvPr with descr="" must be present (Decorative = empty descr attribute).
    //
    // Regression guard: `build_picture` sets description: Some("".to_owned()) for Decorative
    // frames so that `descr=""` is emitted. `collect_cnvpr_descr_values` only collects
    // cNvPr elements that have a descr attribute; any regression reverting to description: None
    // (which emits no descr attribute) will cause this assertion to fail.
    let descr_values = collect_cnvpr_descr_values(&slide_xml);
    assert!(
        descr_values.iter().any(String::is_empty),
        "slide1.xml must contain <p:cNvPr descr=\"\"> for a Diagram frame with \
         AltText::Decorative (EC-007 — DiagramSpec.alt = None safe-sentinel path). \
         The descr attribute MUST be present with empty value (not absent). \
         Found descr values: {descr_values:?}"
    );
}
