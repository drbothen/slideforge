//! Red Gate failing tests for STORY-096: REND-007 — PPTX Master 16:9 Geometry +
//! progress_bar Layout + Run lang.
//!
//! All tests in this file MUST FAIL before the STORY-096 implementation begins.
//! Per BC-5.39.001 / VSDD TDD discipline, every test here drives a behaviorally
//! meaningful assertion (LESSON-14: content/value tests, not mere presence tests).
//!
//! ## Traceability
//!
//! | Test function | AC/EC | BC clause | Red Gate status |
//! |---|---|---|---|
//! | `test_BC_4_01_001_master_has_no_sldsz_presentation_carries_16x9` | AC-001 | BC-4.01.001 postcondition 2 | RED |
//! | `test_BC_4_01_001_master_has_no_sldsz_presentation_carries_custom_page_size` | AC-001/EC-001 | BC-4.01.001 postcondition 2 | RED |
//! | `test_BC_4_01_001_footer_date_placeholders_within_16x9_bounds` | AC-004 | BC-4.01.001 postcondition 4 | RED |
//! | `test_BC_4_01_005_progress_bar_has_named_layout` | AC-002 | BC-4.01.005 postcondition 4/5 | RED |
//! | `test_BC_4_01_005_progress_bar_slide_resolves_named_layout_not_fallback` | AC-002/EC-004 | BC-4.01.005 postcondition 5 | RED |
//! | `test_BC_5_01_005_run_has_lang_attribute_default_en_us` | AC-003/EC-002 | BC-5.01.005 postcondition 1 | RED |
//! | `test_BC_5_01_005_run_has_lang_attribute_fr_fr_round_trip` | AC-003 | BC-5.01.005 postcondition 1 | RED |
//! | `test_BC_5_01_005_ec003_empty_slide_no_rpr_emitted_no_error` | AC-003/EC-003 | BC-5.01.005 postcondition 1 | RED (LESSON-17: NOT #[should_panic]) |
//! | `test_BC_5_01_005_f096_002_no_lang_defaults_to_en_cross_surface` | F-096-002 | BC-5.01.005 v1.3 | RED → GREEN |

#![allow(non_snake_case)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use std::io::Read as _;
use std::sync::Arc;

use slideforge_layout::{BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize};
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::{Brand, BrandFonts, BrandPalette, Deck, Emu, InlineNode};
use zip::ZipArchive;

use crate::PptxExporter;

// ─── Fixture builders ─────────────────────────────────────────────────────────

/// Build a minimal valid `Deck` with `n` slides.
///
/// Sets `lang` to an explicit `"en-US"` declaration (not the no-lang default).
/// The no-lang default is `"en"` (BC-5.01.005 v1.3 / story EC-002), applied by
/// `export_inner` via `DEFAULT_DECK_LANG`; that path is exercised by
/// `make_deck_no_lang` + `test_BC_5_01_005_f096_002_no_lang_defaults_to_en_cross_surface`.
fn make_deck(n: usize) -> Deck {
    use slideforge_types::deck::DeckMetadata;
    use slideforge_types::ordered_map::OrderedMap;
    use slideforge_types::slide::Slide;

    Deck {
        slides: (0..n)
            .map(|_| Slide {
                slide_type: Arc::from("title"),
                fields: OrderedMap::new(),
                blocks: vec![],
                register: None,
                tags: vec![],
                source_span: slideforge_types::SourceSpan::default(),
                overlay: None,
                register_content: vec![],
                field_spans: OrderedMap::new(),
            })
            .collect(),
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
        slide_sections: vec![],
    }
}

/// Build a minimal valid `Deck` with `lang: None` (no declaration in the DSL source).
///
/// Used by F-096-002 tests to verify BC-5.01.005 v1.3: both `<a:rPr lang>` and
/// `<dc:language>` must default to `"en"` when no lang is declared.
fn make_deck_no_lang(n: usize) -> Deck {
    let mut deck = make_deck(n);
    deck.metadata.lang = None;
    deck
}

/// Build a minimal valid `Deck` with an explicit `lang` value.
fn make_deck_with_lang(n: usize, lang: &str) -> Deck {
    let mut deck = make_deck(n);
    deck.metadata.lang = Some(Arc::from(lang));
    deck
}

/// Build a minimal valid `Brand` with default (empty) layouts.
///
/// Page dimensions are NOT set here — they are added by `brand_with_page_size`
/// for tests that exercise the page_size path.
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

fn title_bbox() -> BoundingBox {
    BoundingBox {
        x: Emu(457_200),
        y: Emu(274_638),
        width: Emu(8_229_600),
        height: Emu(1_143_000),
    }
}

fn body_bbox() -> BoundingBox {
    BoundingBox {
        x: Emu(457_200),
        y: Emu(1_600_200),
        width: Emu(8_229_600),
        height: Emu(3_200_400),
    }
}

/// Build a `LaidOutSlide` for the given keyword and optional body text.
fn make_slide_with_keyword(index: usize, keyword: &str) -> LaidOutSlide {
    LaidOutSlide {
        source_index: index,
        slide_type_keyword: Arc::from(keyword),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Title(Arc::from("Test Slide")),
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    }
}

/// Build a `LaidOutSlide` with a title AND a body text frame containing plain text.
///
/// AC-003 tests require a slide with actual text runs to check `<a:rPr lang="...">`.
fn make_slide_with_body_text(index: usize) -> LaidOutSlide {
    use slideforge_types::block::{ContentBlock, TextBlock, TextTag};

    LaidOutSlide {
        source_index: index,
        slide_type_keyword: Arc::from("content"),
        frames: vec![
            Frame {
                bbox: title_bbox(),
                content: FrameContent::Title(Arc::from("Title Text")),
                text_flow: None,
                region_role: None,
            },
            Frame {
                bbox: body_bbox(),
                content: FrameContent::Body(vec![ContentBlock::Text(TextBlock {
                    inlines: vec![InlineNode::Plain(Arc::from("Hello world"))],
                    tag: TextTag::Body,
                    span: slideforge_types::SourceSpan::default(),
                })]),
                text_flow: None,
                region_role: None,
            },
        ],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    }
}

/// Build a minimal `LaidOutDeck` with `n` title slides.
fn make_laid_out_deck(n: usize) -> LaidOutDeck {
    LaidOutDeck {
        page_size: PageSize::default(), // 16:9 = 9144000 × 5143500
        slides: (0..n)
            .map(|i| make_slide_with_keyword(i, "title"))
            .collect(),
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    }
}

/// Build a `LaidOutDeck` with a custom page size — for EC-001 (custom brand page).
fn make_laid_out_deck_with_page_size(width: i64, height: i64) -> LaidOutDeck {
    LaidOutDeck {
        page_size: PageSize {
            width: Emu(width),
            height: Emu(height),
        },
        slides: vec![make_slide_with_keyword(0, "title")],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    }
}

/// Run the exporter on a `LaidOutDeck` with the default deck and brand.
fn build_pptx(laid_out: &LaidOutDeck) -> Vec<u8> {
    let deck = make_deck(laid_out.slides.len());
    let brand = make_brand();
    let opts = ExportOptions::default();
    let exporter = PptxExporter::new();
    exporter
        .export(&deck, laid_out, &brand, &opts)
        .expect("PptxExporter::export must succeed")
}

/// Run the exporter with an explicit `Deck` (for lang tests).
fn build_pptx_with_deck(deck: &Deck, laid_out: &LaidOutDeck) -> Vec<u8> {
    let brand = make_brand();
    let opts = ExportOptions::default();
    let exporter = PptxExporter::new();
    exporter
        .export(deck, laid_out, &brand, &opts)
        .expect("PptxExporter::export must succeed")
}

/// Read a named entry from a ZIP archive as a UTF-8 string.
fn zip_read_entry(bytes: &[u8], path: &str) -> String {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).expect("must be a valid ZIP");
    let mut entry = archive
        .by_name(path)
        .unwrap_or_else(|_| panic!("entry '{path}' must exist in the ZIP"));
    let mut buf = String::new();
    entry.read_to_string(&mut buf).expect("must be valid UTF-8");
    buf
}

/// Parse the `cx` and `cy` attribute values from a `<p:sldSz ...>` element in `xml`.
///
/// Returns `(cx, cy)` as `i64` values, or `None` if the element is absent.
///
/// This is the load-bearing helper for AC-001 and AC-004 tests:
/// it extracts EXACT values, not just verifying element presence (LESSON-14).
fn parse_sldsz_cx_cy(xml: &str) -> Option<(i64, i64)> {
    let pos = xml.find("<p:sldSz")?;
    let after = &xml[pos..];
    let end = after.find('>')?;
    let attrs_str = &after[..=end];

    let cx = extract_attr_i64(attrs_str, "cx")?;
    let cy = extract_attr_i64(attrs_str, "cy")?;
    Some((cx, cy))
}

/// Extract a named attribute value as `i64` from an XML attribute string.
///
/// Handles both `attr="value"` and `attr='value'` quoting.
fn extract_attr_i64(attrs: &str, attr_name: &str) -> Option<i64> {
    // Try double-quoted form: attr="value"
    let search_dq = format!("{attr_name}=\"");
    if let Some(pos) = attrs.find(&search_dq) {
        let rest = &attrs[pos + search_dq.len()..];
        if let Some(end) = rest.find('"') {
            return rest[..end].parse().ok();
        }
    }
    // Try single-quoted form: attr='value'
    let single_quoted_prefix = format!("{attr_name}='");
    if let Some(pos) = attrs.find(&single_quoted_prefix) {
        let rest = &attrs[pos + single_quoted_prefix.len()..];
        if let Some(end) = rest.find('\'') {
            return rest[..end].parse().ok();
        }
    }
    None
}

/// Collect all `lang="..."` attribute values from `<a:rPr>` elements in `xml`.
///
/// Returns a `Vec<String>` of every lang value found (empty vec if none found).
/// This is the load-bearing helper for AC-003 (LESSON-14: collect actual values,
/// not just assert element presence).
fn collect_rpr_lang_values(xml: &str) -> Vec<String> {
    let mut langs = Vec::new();
    let mut rest = xml;
    while let Some(pos) = rest.find("<a:rPr") {
        let after = &rest[pos..];
        // Find the closing `>` or `/>` of the rPr element opening tag.
        let end = after.find('>').unwrap_or(after.len());
        let tag_content = &after[..=end];

        // Extract lang="..." from this rPr tag's attributes.
        let search = r#"lang=""#;
        if let Some(lang_pos) = tag_content.find(search) {
            let lang_start = lang_pos + search.len();
            let lang_rest = &tag_content[lang_start..];
            if let Some(lang_end) = lang_rest.find('"') {
                langs.push(lang_rest[..lang_end].to_owned());
            }
        }

        // Advance past the current rPr tag.
        rest = &rest[pos + "<a:rPr".len()..];
    }
    langs
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-001: slideMaster1.xml must NOT contain <p:sldSz>; presentation.xml must.
// Traces to BC-4.01.001 postcondition 2 (valid .pptx with correct geometry)
// STORY-096 spec v1.2: ECMA-376 §19.3.1.42 — CT_SlideMaster has no sldSz field.
// sldSz belongs exclusively to CT_Presentation (presentation.xml).
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.001 postcondition 2 / STORY-096 AC-001 (spec v1.2):
///
/// Verifies that `slideMaster1.xml` contains NO `<p:sldSz>` element (schema
/// correctness: `CT_SlideMaster` does not define a `sldSz` field per
/// ECMA-376 §19.3.1.42), and that `presentation.xml` carries the correct
/// `<p:sldSz cx="9144000" cy="5143500"/>` for the default 16:9 page size.
///
/// The `page_size_emu` parameter continues to be threaded through to
/// `serialize_master_to_xml` so footer/date/slideNum placeholder geometry
/// remains derived from the deck page size (AC-004 / BC-4.01.001 postcondition 4).
/// Only the invalid `<p:sldSz>` emission is removed.
///
/// Asserts EXACT EMU values in presentation.xml (LESSON-14).
#[test]
fn test_BC_4_01_001_master_has_no_sldsz_presentation_carries_16x9() {
    // Default page size: 16:9 = cx=9,144,000 cy=5,143,500 EMU.
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let master_xml = zip_read_entry(&pptx_bytes, "ppt/slideMasters/slideMaster1.xml");
    let presentation_xml = zip_read_entry(&pptx_bytes, "ppt/presentation.xml");

    // Contract (i): slideMaster1.xml must contain NO <p:sldSz> element.
    assert!(
        !master_xml.contains("<p:sldSz"),
        "AC-001 v1.2: slideMaster1.xml must NOT contain <p:sldSz> — \
         CT_SlideMaster has no such field (ECMA-376 §19.3.1.42). \
         Found a <p:sldSz> element.\nmaster_xml excerpt:\n{}",
        &master_xml[..master_xml.len().min(1500)]
    );

    // Contract (ii): presentation.xml must carry <p:sldSz cx="9144000" cy="5143500"/>.
    let (cx, cy) = parse_sldsz_cx_cy(&presentation_xml).unwrap_or_else(|| {
        panic!(
            "AC-001 v1.2: presentation.xml must contain <p:sldSz> with 16:9 dimensions. \
             None found.\npresentation_xml excerpt:\n{}",
            &presentation_xml[..presentation_xml.len().min(1500)]
        )
    });
    assert_eq!(
        cx, 9_144_000,
        "AC-001 v1.2: presentation.xml <p:sldSz cx> must be 9144000 (16:9 default width); \
         got cx={cx}."
    );
    assert_eq!(
        cy, 5_143_500,
        "AC-001 v1.2: presentation.xml <p:sldSz cy> must be 5143500 (16:9 default height); \
         got cy={cy}."
    );
}

/// BC-4.01.001 postcondition 2 / STORY-096 AC-001 / EC-001 (spec v1.2):
///
/// Verifies that for a 4:3 custom brand page size (`cx=6858000 cy=5143500`):
/// (i) `slideMaster1.xml` still contains NO `<p:sldSz>` element.
/// (ii) `presentation.xml` carries `<p:sldSz cx="6858000" cy="5143500"/>`.
///
/// EC-001: custom brand page size — `serialize_master_to_xml` uses `page_size_emu`
/// for placeholder geometry only; the `<p:sldSz>` element is never emitted in master.
/// `presentation.xml` derives its `SlideSize` from `LaidOutDeck.page_size` directly.
#[test]
fn test_BC_4_01_001_master_has_no_sldsz_presentation_carries_custom_page_size() {
    // 4:3 custom page: width=6,858,000 height=5,143,500
    let custom_page_width: i64 = 6_858_000;
    let custom_page_height: i64 = 5_143_500;
    let laid_out = make_laid_out_deck_with_page_size(custom_page_width, custom_page_height);
    let pptx_bytes = build_pptx(&laid_out);

    let master_xml = zip_read_entry(&pptx_bytes, "ppt/slideMasters/slideMaster1.xml");
    let presentation_xml = zip_read_entry(&pptx_bytes, "ppt/presentation.xml");

    // Contract (i): slideMaster1.xml must contain NO <p:sldSz> element.
    assert!(
        !master_xml.contains("<p:sldSz"),
        "AC-001/EC-001 v1.2: slideMaster1.xml must NOT contain <p:sldSz> — \
         CT_SlideMaster has no such field (ECMA-376 §19.3.1.42). \
         Found a <p:sldSz> element.\nmaster_xml excerpt:\n{}",
        &master_xml[..master_xml.len().min(1500)]
    );

    // Contract (ii): presentation.xml must carry the custom-brand page dimensions.
    let (cx, cy) = parse_sldsz_cx_cy(&presentation_xml).unwrap_or_else(|| {
        panic!(
            "AC-001/EC-001 v1.2: presentation.xml must contain <p:sldSz> for custom \
             brand page size. None found.\npresentation_xml excerpt:\n{}",
            &presentation_xml[..presentation_xml.len().min(1500)]
        )
    });
    assert_eq!(
        cx, custom_page_width,
        "AC-001 EC-001 v1.2: presentation.xml <p:sldSz cx> must be {custom_page_width} \
         (brand-configured custom width); got cx={cx}."
    );
    assert_eq!(
        cy, custom_page_height,
        "AC-001 EC-001 v1.2: presentation.xml <p:sldSz cy> must be {custom_page_height} \
         (brand-configured custom height); got cy={cy}."
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-004: Footer and date placeholders stay within 16:9 slide bounds
// Traces to BC-4.01.001 postcondition 4 (openable without error dialogs)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.001 postcondition 4 / STORY-096 AC-004:
///
/// Verifies that every placeholder `<p:sp>` shape in `slideMaster1.xml` has
/// both its top edge (`off.y`) AND its bottom edge (`off.y + ext.cy`) within
/// the 16:9 slide height boundary of 5,143,500 EMU
/// (BC-4.01.001 postcondition 4).
///
/// `MASTER_PLACEHOLDER_DEFS` must size and position the body placeholder so
/// its bottom edge does not extend past the page height. The body placeholder
/// at `static_y=1_600_200, cy=4_525_963` overflows the 16:9 boundary
/// (bottom = 6,126,163 > 5,143,500); `cy` must be derived from the footer
/// zone top rather than using an unchecked static value.
///
/// Footer/date/slideNum placeholders use a page-height-derived `y` via
/// `serialize_master_to_xml`, so their tops stay within bounds; this test
/// verifies BOTH top-edge AND bottom-edge invariants for ALL master shapes.
///
/// LESSON-14: asserts ACTUAL coordinate arithmetic, not merely element presence.
/// F-096-003: strengthened to check `off.y + ext.cy <= page_height`
/// (bottom-edge invariant) in addition to the pre-existing top-edge check.
#[test]
fn test_BC_4_01_001_footer_date_placeholders_within_16x9_bounds() {
    // 16:9 page height in EMU.
    const PAGE_HEIGHT: i64 = 5_143_500;

    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);
    let master_xml = zip_read_entry(&pptx_bytes, "ppt/slideMasters/slideMaster1.xml");

    // Parse (off.y, ext.cy) pairs from each <a:xfrm> block in the master XML.
    // Each <a:xfrm> contains <a:off> followed by <a:ext>; we collect them as
    // pairs to check BOTH top-edge and bottom-edge invariants.
    let mut top_violations: Vec<i64> = Vec::new();
    let mut bottom_violations: Vec<(i64, i64)> = Vec::new(); // (y, cy) pairs

    let mut rest = master_xml.as_str();
    while let Some(xfrm_pos) = rest.find("<a:xfrm") {
        let after_xfrm = &rest[xfrm_pos..];
        // Find the end of the xfrm block (up to </a:xfrm>).
        let xfrm_end = after_xfrm.find("</a:xfrm>").unwrap_or(after_xfrm.len());
        let xfrm_block = &after_xfrm[..xfrm_end];

        // Extract off.y from <a:off y="..."> within this xfrm block.
        let off_y: Option<i64> = xfrm_block.find("<a:off").and_then(|p| {
            let tag_start = &xfrm_block[p..];
            let tag_end = tag_start.find('>').unwrap_or(tag_start.len());
            extract_attr_i64(&tag_start[..=tag_end], "y")
        });

        // Extract ext.cy from <a:ext cy="..."> within this xfrm block.
        let ext_cy: Option<i64> = xfrm_block.find("<a:ext").and_then(|p| {
            let tag_start = &xfrm_block[p..];
            let tag_end = tag_start.find('>').unwrap_or(tag_start.len());
            extract_attr_i64(&tag_start[..=tag_end], "cy")
        });

        if let Some(y) = off_y {
            // Top-edge: y must not exceed page height.
            if y > PAGE_HEIGHT {
                top_violations.push(y);
            }
            // Bottom-edge: y + cy must not exceed page height.
            if let Some(cy) = ext_cy.filter(|&h| y + h > PAGE_HEIGHT) {
                bottom_violations.push((y, cy));
            }
        }

        rest = &rest[xfrm_pos + "<a:xfrm".len()..];
    }

    assert!(
        top_violations.is_empty(),
        "AC-004 top-edge: slideMaster1.xml has placeholder(s) with top-edge y \
         exceeding the 16:9 height boundary ({PAGE_HEIGHT} EMU).\n\
         Violating top-edge y values: {top_violations:?}\n\
         master_xml excerpt (first 2000 chars):\n{}",
        &master_xml[..master_xml.len().min(2000)]
    );

    assert!(
        bottom_violations.is_empty(),
        "AC-004 bottom-edge / F-096-003: slideMaster1.xml has placeholder(s) whose \
         bottom edge (off.y + ext.cy) exceeds the 16:9 height boundary \
         ({PAGE_HEIGHT} EMU).\n\
         Violating (y, cy) pairs: {bottom_violations:?}\n\
         The body placeholder cy must be derived from the footer zone top so its \
         bottom edge fits within the page: cy = footer_zone_top - body_y - margin.\n\
         master_xml excerpt (first 2000 chars):\n{}",
        &master_xml[..master_xml.len().min(2000)]
    );
}

/// BC-4.01.001 postcondition 4 / STORY-096 AC-004 / F-096-004:
///
/// Pathological page height guard: when `page_height < FOOTER_MARGIN_FROM_BOTTOM`
/// (e.g., a 100,000 EMU tall slide), `footer_y` must not underflow to a negative
/// value. `serialize_master_to_xml` must clamp `footer_y` to a non-negative floor
/// (saturating_sub semantics) so no `<a:off y="...">` emits a negative offset.
///
/// This test exercises the edge case directly by calling `serialize_master_to_xml`
/// with a pathological tiny height and asserting that NO `<a:off y="...">` in the
/// output is negative.
#[test]
fn test_BC_4_01_001_footer_y_saturating_on_tiny_page() {
    use slideforge_brand::layout_xml::serialize_master_to_xml;
    use slideforge_brand::layout_xml::{
        HANDOUT_MASTER_STUB, NOTES_MASTER_STUB, generate_content_types_layout_entries,
    };
    use slideforge_brand::layouts::generate_all_layouts;
    use slideforge_brand::template::{BrandFonts as BTFonts, ColorSlot, ColorValue, MasterIds};
    use slideforge_brand::toml_schema::BrandConfig;

    let config = BrandConfig::default_minimal();
    let layouts = generate_all_layouts(&config);
    let template = slideforge_brand::BrandTemplate {
        colors: std::array::from_fn(|i| ColorSlot {
            name: Arc::from(slideforge_brand::template::COLOR_SLOT_NAMES[i]),
            value: ColorValue::Hex(Arc::from("003087")),
            is_derived: false,
        }),
        fonts: BTFonts {
            heading: Arc::from("Calibri"),
            body: Arc::from("Calibri"),
        },
        logo: None,
        footer_text: None,
        footer_flags: slideforge_brand::FooterFlags::default(),
        layout_names: vec![],
        layouts,
        notes_master_stub: NOTES_MASTER_STUB.to_vec(),
        handout_master_stub: HANDOUT_MASTER_STUB.to_vec(),
        master_ids: MasterIds::default(),
        content_types_layout_entries: Arc::from(
            generate_content_types_layout_entries(crate::LAYOUT_COUNT).as_str(),
        ),
    };

    // Pathological tiny page height — less than FOOTER_MARGIN_FROM_BOTTOM (571,500 EMU).
    // Without clamping: footer_y = 100_000 - 571_500 = -471_500 (underflow).
    let xml_bytes = serialize_master_to_xml(&template, (1_000_000, 100_000));
    let xml = std::str::from_utf8(&xml_bytes).expect("valid UTF-8");

    // Scan all <a:off y="..."> values; none must be negative.
    let mut negative_ys: Vec<i64> = Vec::new();
    let mut rest = xml;
    while let Some(pos) = rest.find("<a:off") {
        let after = &rest[pos..];
        let end = after.find('>').unwrap_or(after.len());
        let tag_content = &after[..=end];
        if let Some(y) = extract_attr_i64(tag_content, "y").filter(|&v| v < 0) {
            negative_ys.push(y);
        }
        rest = &rest[pos + "<a:off".len()..];
    }

    assert!(
        negative_ys.is_empty(),
        "F-096-004: serialize_master_to_xml with pathological tiny page height \
         (100,000 EMU < FOOTER_MARGIN_FROM_BOTTOM 571,500 EMU) must not emit \
         negative <a:off y=\"...\"> values. Got negative y values: {negative_ys:?}.\n\
         footer_zone_top must be clamped: \
         (page_height - FOOTER_MARGIN_FROM_BOTTOM).max(0)."
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-002: progress_bar slide type has a named layout in the layout hierarchy
// Traces to BC-4.01.005 postcondition 4/5 (all 31 layouts present with keywords)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.005 postcondition 4 / STORY-096 AC-002:
///
/// Verifies that the PPTX layout hierarchy includes a layout XML file whose
/// `<p:cSld>` has `name="progress_bar"` (or equivalent "SF Progress Bar"
/// canonical name), so a `progress_bar` slide does NOT route to the fallback
/// layout (index 1, "Title and Content") (BC-4.01.005 postcondition 4).
///
/// `generate_all_layouts` must include an entry with
/// `slide_type_keyword = Some("progress_bar")`. `find_layout_index` must match
/// it by keyword and return its dedicated index.
///
/// Assertions (LESSON-14, LESSON-17 — no `#[should_panic]`):
/// 1. A layout XML file in the ZIP has `<p:cSld name="progress_bar">` (or name
///    matching the canonical SF naming for progress_bar).
/// 2. The slide's `.rels` relationship does NOT point to `slideLayout2.xml`
///    (which is the "Title and Content" fallback, 1-indexed).
#[test]
fn test_BC_4_01_005_progress_bar_has_named_layout() {
    let mut laid_out = make_laid_out_deck(1);
    laid_out.slides[0].slide_type_keyword = Arc::from("progress_bar");

    let pptx_bytes = build_pptx(&laid_out);

    // Assertion 1: at least one slideLayout*.xml contains a cSld with the
    // canonical progress_bar layout name.
    // The name convention follows "SF Progress Bar" or the slide_type_keyword "progress_bar"
    // embedded in the cSld name attribute.
    let mut found_progress_bar_layout = false;
    let cursor = std::io::Cursor::new(&pptx_bytes);
    let mut archive = ZipArchive::new(cursor).expect("must be valid ZIP");
    for i in 0..archive.len() {
        let entry = archive.by_index(i).expect("index in range");
        let name = entry.name().to_owned();
        if name.starts_with("ppt/slideLayouts/slideLayout")
            && name.to_ascii_lowercase().ends_with(".xml")
            && !name.contains("_rels")
        {
            drop(entry);
            let layout_xml = zip_read_entry(&pptx_bytes, &name);
            // The canonical naming is either:
            //   <p:cSld name="progress_bar"> (exact keyword as name)
            //   <p:cSld name="SF Progress Bar"> (SF-prefix convention)
            // Either is acceptable; we check for "progress_bar" or "Progress Bar" case-insensitively.
            let lower = layout_xml.to_lowercase();
            if lower.contains(r#"name="progress_bar""#)
                || lower.contains(r#"name="sf progress bar""#)
            {
                found_progress_bar_layout = true;
                break;
            }
        }
    }

    assert!(
        found_progress_bar_layout,
        "AC-002: No slideLayout*.xml in the PPTX ZIP has a \
         <p:cSld name=\"progress_bar\"> (or \"SF Progress Bar\") attribute.\n\
         `generate_all_layouts` must include an entry with \
         slide_type_keyword = Some(\"progress_bar\") so the layout is synthesized \
         and added to the ZIP with its canonical name."
    );
}

/// BC-4.01.005 postcondition 5 / STORY-096 AC-002 / EC-004:
///
/// Verifies that `find_layout_index` returns an index other than 1 for
/// `"progress_bar"` — i.e., the progress_bar layout is resolved to its own
/// dedicated slot (>= 2, not the Title and Content fallback at index 1)
/// (BC-4.01.005 postcondition 5).
///
/// Also verifies EC-004: a `progress_bar` slide with a label field routes
/// to the named layout correctly (the label field does NOT affect layout routing).
///
/// LESSON-14: asserts the ACTUAL layout index, not merely that routing succeeds.
#[test]
fn test_BC_4_01_005_progress_bar_slide_resolves_named_layout_not_fallback() {
    use slideforge_brand::layout_xml::{
        HANDOUT_MASTER_STUB, NOTES_MASTER_STUB, generate_content_types_layout_entries,
    };
    use slideforge_brand::layouts::generate_all_layouts;
    use slideforge_brand::template::{BrandFonts as BTFonts, ColorSlot, ColorValue, MasterIds};
    use slideforge_brand::toml_schema::BrandConfig;

    let config = BrandConfig::default_minimal();
    let layouts = generate_all_layouts(&config);

    let brand_template = slideforge_brand::BrandTemplate {
        colors: std::array::from_fn(|i| ColorSlot {
            name: Arc::from(slideforge_brand::template::COLOR_SLOT_NAMES[i]),
            value: ColorValue::Hex(Arc::from("003087")),
            is_derived: false,
        }),
        fonts: BTFonts {
            heading: Arc::from("Calibri"),
            body: Arc::from("Calibri"),
        },
        logo: None,
        footer_text: None,
        footer_flags: slideforge_brand::FooterFlags::default(),
        layout_names: vec![],
        layouts,
        notes_master_stub: NOTES_MASTER_STUB.to_vec(),
        handout_master_stub: HANDOUT_MASTER_STUB.to_vec(),
        master_ids: MasterIds::default(),
        content_types_layout_entries: Arc::from(
            generate_content_types_layout_entries(crate::LAYOUT_COUNT).as_str(),
        ),
    };

    let idx = crate::find_layout_index(&brand_template, "progress_bar");

    // AC-002 / F-096-005: progress_bar must resolve to its dedicated index 30
    // (CL-20, SF Progress Bar, the 31st layout in the 0-based 0..30 range).
    // Index 30 is the exact slot defined in `generate_all_layouts` for the
    // progress_bar DSL keyword (Q2 built-in type). Asserting the exact index
    // is load-bearing: `!= 1` would pass for any non-fallback hit, masking
    // accidental slot collisions or keyword-matching regressions.
    assert_eq!(
        idx, 30,
        "AC-002 / F-096-005: find_layout_index(\"progress_bar\") must return exactly 30 \
         (CL-20, SF Progress Bar, 0-based). Got {idx}.\n\
         `generate_all_layouts` must place progress_bar at index 30 via Phase 1 \
         keyword match (slide_type_keyword = Some(\"progress_bar\"))."
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-003: All text runs carry lang attribute from deck metadata
// Traces to BC-5.01.005 postcondition 1 (lang propagates to PPTX rPr)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-5.01.005 postcondition 1 / STORY-096 AC-003:
///
/// Verifies that every `<a:rPr>` element in `slide1.xml` carries a `lang="en-US"`
/// attribute when the deck's `metadata.lang` is explicitly `"en-US"` (BC-5.01.005
/// postcondition 1). `ooxml_run_to_ooxmlsdk` must set `rpr.lang` on every
/// `RunProperties` from `deck.metadata.lang`.
///
/// This fixture exercises an EXPLICIT `lang "en-US"` declaration, not the EC-002
/// no-lang default path. The no-lang default is `"en"` (BC-5.01.005 v1.3); that
/// path is covered by `make_deck_no_lang` +
/// `test_BC_5_01_005_f096_002_no_lang_defaults_to_en_cross_surface`.
///
/// LESSON-14: asserts ACTUAL `lang` attribute values, not mere rPr presence.
/// LESSON-17: does NOT use `#[should_panic]`.
#[test]
fn test_BC_5_01_005_run_has_lang_attribute_default_en_us() {
    // Build a deck with explicit en-US (explicit declaration path, not EC-002 default).
    let mut laid_out = make_laid_out_deck(1);
    laid_out.slides[0] = make_slide_with_body_text(0);

    let deck = make_deck_with_lang(1, "en-US");
    let pptx_bytes = build_pptx_with_deck(&deck, &laid_out);
    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    // Verify the slide actually contains <a:rPr> elements before checking lang.
    assert!(
        slide1_xml.contains("<a:rPr"),
        "AC-003 prerequisite: slide1.xml must contain at least one <a:rPr> element \
         (the slide has body text runs); got slide excerpt:\n{}",
        &slide1_xml[..slide1_xml.len().min(2000)]
    );

    let lang_values = collect_rpr_lang_values(&slide1_xml);

    // At least one rPr must have a lang attribute.
    assert!(
        !lang_values.is_empty(),
        "AC-003 Red Gate: No <a:rPr> in slide1.xml has a lang=\"...\" attribute.\n\
         `ooxml_run_to_ooxmlsdk` must set `rpr.lang = Some(\"en-US\".to_owned())` \
         (or the deck's declared lang value) on every RunProperties.\n\
         slide1.xml excerpt:\n{}",
        &slide1_xml[..slide1_xml.len().min(2000)]
    );

    // Every lang value found must be exactly "en-US" (exact BCP-47 tag —
    // BC-5.01.005 invariant 1: lossless, no truncation or normalization).
    for lang in &lang_values {
        assert_eq!(
            lang.as_str(),
            "en-US",
            "AC-003 Red Gate: <a:rPr lang> value must be exactly \"en-US\" \
             (the deck's declared lang); got \"{lang}\".\n\
             BC-5.01.005 invariant 1: language propagation is lossless."
        );
    }
}

/// BC-5.01.005 postcondition 1 / STORY-096 AC-003 — fr-FR round-trip:
///
/// Verifies that when the deck declares `lang "fr-FR"`, every `<a:rPr>` in
/// slide XML carries `lang="fr-FR"` (BC-5.01.005 postcondition 1, canonical
/// test vector: `lang "fr-FR"` → PPTX → `<a:rPr lang="fr-FR"/>`).
///
/// The lang value must be the exact BCP-47 tag from the deck declaration —
/// not normalised to "fr", not modified in any way (BC-5.01.005 invariant 1:
/// lossless propagation).
///
/// LESSON-14: asserts the EXACT string "fr-FR", not mere attribute presence.
#[test]
fn test_BC_5_01_005_run_has_lang_attribute_fr_fr_round_trip() {
    let mut laid_out = make_laid_out_deck(1);
    laid_out.slides[0] = make_slide_with_body_text(0);

    // Explicit fr-FR deck declaration.
    let deck = make_deck_with_lang(1, "fr-FR");
    let pptx_bytes = build_pptx_with_deck(&deck, &laid_out);
    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    assert!(
        slide1_xml.contains("<a:rPr"),
        "AC-003 fr-FR prerequisite: slide1.xml must contain at least one <a:rPr> \
         (the slide has body text runs)"
    );

    let lang_values = collect_rpr_lang_values(&slide1_xml);

    assert!(
        !lang_values.is_empty(),
        "AC-003 Red Gate (fr-FR): No <a:rPr> in slide1.xml has a lang attribute.\n\
         `ooxml_run_to_ooxmlsdk` must propagate `deck.metadata.lang` (\"fr-FR\") \
         to `rpr.lang`.\n\
         slide1.xml excerpt:\n{}",
        &slide1_xml[..slide1_xml.len().min(2000)]
    );

    for lang in &lang_values {
        assert_eq!(
            lang.as_str(),
            "fr-FR",
            "AC-003 Red Gate (fr-FR round-trip): <a:rPr lang> must be exactly \
             \"fr-FR\" (BC-5.01.005 invariant 1: lossless propagation); \
             got \"{lang}\""
        );
    }
}

/// BC-5.01.005 postcondition 1 / STORY-096 AC-003 / F-096-001 — universality:
///
/// Verifies that EVERY `<a:rPr>` element in `slide1.xml` carries `lang="en-US"`,
/// not just body runs. Title and Subtitle frames use `build_shape` which previously
/// hardcoded `run_properties: Some(Box::default())` (RunProperties with `language:
/// None`), so the title run emitted `<a:rPr></a:rPr>` with NO lang attribute.
///
/// Universality contract (AC-003): `collect_rpr_lang_values(xml).len()` must
/// equal `xml.matches("<a:rPr").count()` — every single rPr element must carry
/// the lang tag, regardless of which frame type produced it.
///
/// This test uses a slide with BOTH a Title frame (plain string → `build_shape`)
/// AND a Body frame (inline nodes → `ooxml_run_to_ooxmlsdk`) to ensure both
/// code paths are exercised.
#[test]
fn test_BC_5_01_005_ac003_all_rpr_carry_lang_universality() {
    // make_slide_with_body_text includes both a Title frame and a Body frame.
    let mut laid_out = make_laid_out_deck(1);
    laid_out.slides[0] = make_slide_with_body_text(0);

    let deck = make_deck_with_lang(1, "en-US");
    let pptx_bytes = build_pptx_with_deck(&deck, &laid_out);
    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    let rpr_total = slide1_xml.matches("<a:rPr").count();
    let lang_values = collect_rpr_lang_values(&slide1_xml);

    assert!(
        rpr_total > 0,
        "F-096-001 prerequisite: slide1.xml must contain at least one <a:rPr> element; \
         the slide has a Title and a Body frame"
    );

    assert_eq!(
        lang_values.len(),
        rpr_total,
        "F-096-001 / AC-003 universality: EVERY <a:rPr> must carry a lang attribute. \
         Found {rpr_total} <a:rPr> elements but only {} carry lang=\"...\". \
         The title run path (build_shape) must thread lang into RunProperties, \
         not use Box::default().",
        lang_values.len()
    );

    for lang in &lang_values {
        assert_eq!(
            lang.as_str(),
            "en-US",
            "F-096-001: every <a:rPr lang> must be \"en-US\"; got \"{lang}\""
        );
    }
}

/// BC-5.01.005 postcondition 1 / STORY-096 AC-003 / EC-003 (RED):
///
/// A slide with NO text content (zero frames, or only a title frame with no body)
/// must produce a valid ZIP with no error — even if there are no `<a:rPr>` elements
/// to annotate.
///
/// EC-003: "Slide with no text content — no rPr emitted; no error."
///
/// This test verifies:
/// 1. Export succeeds (returns Ok) — no panic or error from lang propagation.
/// 2. If `<a:rPr>` elements ARE present (e.g., title run), they ALL have `lang`.
/// 3. If NO `<a:rPr>` elements are present, the test passes (empty slide is valid).
///
/// LESSON-17 compliance: this test does NOT use `#[should_panic]`. It asserts
/// correctness of the empty-content path directly.
///
/// Contract: `ooxml_run_to_ooxmlsdk` sets `lang` on every `RunProperties` it
/// builds, so all `<a:rPr>` elements carry the deck's declared lang value.
///
/// NOTE: For a truly empty slide (zero frames → zero rPr elements) this test
/// passes by definition — EC-003 is satisfied. The test is load-bearing
/// regardless: it guards against regressions where empty-slide export panics
/// or where `lang` is omitted from any `<a:rPr>` that is emitted.
#[test]
fn test_BC_5_01_005_ec003_empty_slide_no_rpr_error() {
    // EC-003: slide with no text frames at all.
    let mut laid_out = make_laid_out_deck(1);
    laid_out.slides[0].frames.clear();

    let deck = make_deck_with_lang(1, "en-US");
    // Must not panic or return Err.
    let pptx_bytes = build_pptx_with_deck(&deck, &laid_out);

    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    // Invariant: the ZIP is valid and the slide XML is well-formed.
    assert!(
        slide1_xml.contains("<p:sld"),
        "EC-003: empty-frame slide must still produce a well-formed slide1.xml"
    );

    // If any <a:rPr> elements ARE present, they must carry a lang attribute.
    // (This is the AC-003 guard for this edge case.)
    let lang_values = collect_rpr_lang_values(&slide1_xml);
    let rpr_count = slide1_xml.matches("<a:rPr").count();

    if rpr_count > 0 {
        // There are rPr elements — they must ALL have lang.
        assert_eq!(
            lang_values.len(),
            rpr_count,
            "EC-003 Red Gate: slide1.xml has {rpr_count} <a:rPr> element(s) but only \
             {} carry a lang attribute. All <a:rPr> elements must have lang=\"en-US\".",
            lang_values.len()
        );
        for lang in &lang_values {
            assert_eq!(
                lang.as_str(),
                "en-US",
                "EC-003: <a:rPr lang> on empty-slide runs must be \"en-US\"; got \"{lang}\""
            );
        }
    }
    // If rpr_count == 0 (truly empty slide), the test passes — EC-003 satisfied.
}

// ─────────────────────────────────────────────────────────────────────────────
// F-096-002: no-lang default must be "en" on ALL surfaces (BC-5.01.005 v1.3)
// ─────────────────────────────────────────────────────────────────────────────

/// Extract the text content of a `<dc:language>` element from a core.xml string.
///
/// Returns `Some(text)` if the element is found with non-empty content, else `None`.
fn find_dc_language(xml: &str) -> Option<String> {
    let open = "<dc:language>";
    let close = "</dc:language>";
    let start = xml.find(open)?;
    let content_start = start + open.len();
    let end = xml[content_start..].find(close)?;
    let text = xml[content_start..content_start + end].to_owned();
    if text.is_empty() { None } else { Some(text) }
}

/// BC-5.01.005 v1.3 / F-096-002 — cross-surface "en" default:
///
/// When `deck.metadata.lang` is `None` (no `lang` declaration in the DSL source),
/// the PPTX exporter must emit the identical string `"en"` on every surface:
///
/// 1. Every `<a:rPr lang="...">` in `slide1.xml` carries `lang="en"`.
/// 2. `docProps/core.xml` contains `<dc:language>en</dc:language>`.
/// 3. No `lang="en-US"` appears anywhere in `slide1.xml`.
///
/// BC-5.01.004 injects `"en"` into `DeckMetadata.lang` for absent declarations;
/// `export_inner` and `build_doc_props` must both derive from the same
/// `DEFAULT_DECK_LANG` constant (value `"en"`) so the two surfaces cannot diverge.
///
/// Cross-surface identity guarantee: both `export_inner` and `build_doc_props`
/// derive from the same `DEFAULT_DECK_LANG` constant, so the two surfaces are
/// structurally prevented from diverging.
#[test]
fn test_BC_5_01_005_f096_002_no_lang_defaults_to_en_cross_surface() {
    // Deck with lang = None — no declaration in DSL source.
    let deck = make_deck_no_lang(1);
    let mut laid_out = make_laid_out_deck(1);
    laid_out.slides[0] = make_slide_with_body_text(0);

    let pptx_bytes = build_pptx_with_deck(&deck, &laid_out);

    // ── Surface 1: docProps/core.xml dc:language ──────────────────────────────
    let core_xml = zip_read_entry(&pptx_bytes, "docProps/core.xml");
    let dc_lang = find_dc_language(&core_xml).unwrap_or_else(|| {
        panic!(
            "F-096-002: docProps/core.xml must contain <dc:language> even when \
             deck has no lang declaration. core.xml:\n{core_xml}"
        )
    });
    assert_eq!(
        dc_lang, "en",
        "F-096-002: dc:language must be \"en\" (not \"en-US\") when no lang is \
         declared (BC-5.01.005 v1.3 / BC-5.01.004 default). Got: {dc_lang:?}"
    );

    // ── Surface 2: slide1.xml <a:rPr lang="..."> ──────────────────────────────
    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    // Prerequisite: the slide must have rPr elements (body text is present).
    assert!(
        slide1_xml.contains("<a:rPr"),
        "F-096-002 prerequisite: slide1.xml must contain at least one <a:rPr> \
         element (slide has body text runs)"
    );

    let lang_values = collect_rpr_lang_values(&slide1_xml);
    let rpr_total = slide1_xml.matches("<a:rPr").count();

    // Every rPr must carry a lang attribute (universality — AC-003 contract).
    assert_eq!(
        lang_values.len(),
        rpr_total,
        "F-096-002: all {rpr_total} <a:rPr> elements must carry lang=\"en\"; \
         only {} carry a lang attribute",
        lang_values.len()
    );

    // Every lang must be exactly "en", not "en-US".
    for lang in &lang_values {
        assert_eq!(
            lang.as_str(),
            "en",
            "F-096-002: <a:rPr lang> must be \"en\" when no lang is declared \
             (BC-5.01.005 v1.3); got \"{lang}\""
        );
    }

    // Negative assertion: "en-US" must not appear at all in slide1.xml.
    assert!(
        !slide1_xml.contains("lang=\"en-US\""),
        "F-096-002: slide1.xml must NOT contain lang=\"en-US\" when deck lang is \
         None (default is \"en\", not \"en-US\"). slide1.xml excerpt:\n{}",
        &slide1_xml[..slide1_xml.len().min(2000)]
    );
}
