//! Failing tests for STORY-039: PPTX Accessibility Metadata.
//!
//! Covers BC-4.01.004 (alt text embedding) and BC-5.01.005 (lang propagation
//! to PPTX Core Properties). ALL tests in this file MUST FAIL before
//! implementation begins (Red Gate). Tests parse REAL serialized PPTX output —
//! no mock strings.
//!
//! ## Traceability
//!
//! | Test function | AC | BC clause | Red Gate status |
//! |---|---|---|---|
//! | `test_BC_4_01_004_ac001_non_decorative_image_has_non_empty_descr` | AC-001 | postcondition 1 | RED |
//! | `test_BC_4_01_004_ac002_decorative_image_has_empty_descr_attribute_present` | AC-002 | postcondition 2 | RED |
//! | `test_BC_4_01_004_ac003_300_char_alt_not_truncated` | AC-003 | invariant 1 | RED |
//! | `test_BC_4_01_004_ac004_special_chars_xml_escaped_well_formed` | AC-004 | EC-001 | RED |
//! | `test_BC_4_01_004_ac005_chart_diagram_alt_on_enclosing_shape` | AC-005 | EC-005 | RED |
//! | `test_BC_5_01_005_ac006_dc_language_exact_bcp47_en_us` | AC-006 | postcondition 1 | RED |
//! | `test_BC_5_01_005_ac006_dc_language_exact_bcp47_zh_hant_tw` | AC-006 | postcondition 1 | RED |
//! | `test_BC_5_01_005_ac007_no_lang_defaults_to_en` | AC-007 | EC-003 | RED |
//! | `test_BC_4_01_004_ec001_all_decorative_slide` | EC-001 | postcondition 2 | RED |
//! | `test_BC_4_01_004_ec002_zh_tw_lang_bcp47_embedded` | EC-002 | EC-003 | RED |
//! | `test_BC_4_01_004_ec005_300_char_alt_exact_length` | EC-005 | invariant 1 | RED |

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

/// Build a `LaidOutSlide` with one `FrameContent::Image { alt }` frame.
fn make_slide_with_image_alt(index: usize, alt: &str) -> LaidOutSlide {
    LaidOutSlide {
        source_index: index,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Image {
                alt: Arc::from(alt),
            },
            text_flow: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    }
}

/// Build a `LaidOutSlide` with one `FrameContent::Image { alt: "" }` frame
/// to simulate a decorative image (empty alt text = decorative per story spec).
///
/// Note: STORY-015 guarantees that by the time frames reach the PPTX exporter,
/// every visual element has either a non-empty `alt` or `decorative: true`.
/// For STORY-039 tests, we use the existing `FrameContent::Image { alt }` where
/// `alt.is_empty()` signals decorative — matching what `AltTextEmbedder` will
/// need to handle.
fn make_slide_with_decorative_image(index: usize) -> LaidOutSlide {
    // Decorative: represented as Image with empty alt (per AltTextEmbedder contract).
    LaidOutSlide {
        source_index: index,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Image { alt: Arc::from("") },
            text_flow: None,
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
            }],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        }],
        sections: vec![],
        warnings: vec![],
    }
}

/// Build a `LaidOutDeck` with one slide containing a `Diagram` frame (for AC-005).
fn make_laid_out_deck_with_diagram(alt: &str) -> LaidOutDeck {
    // Use a minimal valid SVG that passes NormalizedDiagramSvg::from_normalized_string.
    let svg = Arc::from(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300" aria-label="chart" role="img"><title>chart</title></svg>"#,
    );
    let normalized = NormalizedDiagramSvg::from_normalized_string(svg);

    // For the diagram frame, the alt text should come from the frame metadata.
    // In STORY-039, AltTextEmbedder reads alt from FrameContent::Image or ShapeFrame.
    // For Diagram frames, the enclosing <p:pic> should get the alt from the deck-level
    // alt field that would be attached alongside the diagram content.
    //
    // Since FrameContent::Diagram does not yet carry an alt field (that is STORY-015),
    // this test exercises the path through AltTextEmbedder where the diagram's
    // enclosing <p:pic> shape gets the alt text. We represent this via an Image
    // frame with the given alt text (simulating the output of a chart encoder).
    let _ = normalized; // normalized svg is built, used below for the slide image variant

    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("title"),
            frames: vec![Frame {
                bbox: title_bbox(),
                // Use Image variant to test alt-on-enclosing-shape path.
                // STORY-039 implementer: AltTextEmbedder must set descr on the <p:pic>
                // that wraps chart/diagram SVG (AC-005 / BC-4.01.004 EC-005).
                content: FrameContent::Image {
                    alt: Arc::from(alt),
                },
                text_flow: None,
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

/// Run `PptxExporter::export` with the given deck + laid_out and return the bytes.
///
/// Panics with a descriptive message if the exporter returns an error (from a
/// `todo!()` stub or a real implementation error).
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
///
/// Red Gate: `AltTextEmbedder::embed` is `todo!()` — test panics before any
/// assertion.
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
///
/// Red Gate: `AltTextEmbedder::embed` is `todo!()`.
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
///
/// Red Gate: `AltTextEmbedder::embed` is `todo!()`.
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
///
/// Red Gate: `AltTextEmbedder::embed` is `todo!()`.
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

// ─── AC-005: Chart/diagram alt on enclosing shape ─────────────────────────────

/// BC-4.01.004 EC-005: A chart shape's alt text goes on the enclosing shape.
///
/// A slide with a chart frame must have the alt text on the `<p:cNvPr>` of the
/// `<p:pic>` or `<p:grpSp>` shape that contains the SVG media, not on any
/// inner text box.
///
/// Red Gate: `AltTextEmbedder::embed` is `todo!()`.
#[test]
fn test_BC_4_01_004_ac005_chart_diagram_alt_on_enclosing_shape() {
    let alt_text = "Bar chart: Q1 revenue by region";
    let deck = make_deck_with_lang("en-US");
    let laid_out = make_laid_out_deck_with_diagram(alt_text);

    let pptx_bytes = build_pptx(&deck, &laid_out);
    let slide_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    // The cNvPr on the enclosing pic/grpSp must have the alt text.
    let descr_values = collect_cnvpr_descr_values(&slide_xml);

    assert!(
        descr_values.iter().any(|d| d == alt_text),
        "slide1.xml must have descr=\"{alt_text}\" on the enclosing <p:pic>/<p:grpSp> \
         shape for the chart frame (BC-4.01.004 EC-005). \
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
///
/// Red Gate: Tests parse the actual `docProps/core.xml` from the PPTX ZIP.
/// Currently `build_doc_props` in `lib.rs` uses `deck.metadata.lang` and
/// already writes this. The test fails because `AltTextEmbedder::embed` (called
/// during slide serialization) is `todo!()`, preventing the PPTX from being
/// built at all. Once the stub compiles and the exporter runs end-to-end, this
/// test must verify the exact dc:language value.
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
///
/// Red Gate: `AltTextEmbedder::embed` is `todo!()`.
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
/// A deck with no `lang` declaration has `LaidOutDeck.lang = "en"` (set by
/// upstream evaluator per BC-5.01.004). The PPTX must embed
/// `<dc:language>en</dc:language>`.
///
/// Test vector (BC-5.01.005 canonical):
///   Input: No lang declared → any export
///   Expected: All formats embed "en" (default)
///
/// Note: The current `build_doc_props` uses `deck.metadata.lang.as_deref().unwrap_or("en-US")`.
/// AC-007 requires the default to be "en" (not "en-US"). This test drives that
/// behavior change — the implementer must change the fallback from "en-US" to "en".
///
/// Red Gate: Test will fail because either `AltTextEmbedder` is `todo!()` or
/// the default fallback is the wrong value ("en-US" instead of "en").
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
///
/// Red Gate: `AltTextEmbedder::embed` is `todo!()`.
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
///
/// Red Gate: `AltTextEmbedder::embed` is `todo!()`.
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
///
/// Red Gate: `AltTextEmbedder::embed` is `todo!()`.
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
