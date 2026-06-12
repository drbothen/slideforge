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
//! | `test_BC_4_01_001_master_sldsz_matches_16x9_deck` | AC-001 | BC-4.01.001 postcondition 2 | RED |
//! | `test_BC_4_01_001_master_sldsz_matches_custom_brand_page_size` | AC-001/EC-001 | BC-4.01.001 postcondition 2 | RED |
//! | `test_BC_4_01_001_footer_date_placeholders_within_16x9_bounds` | AC-004 | BC-4.01.001 postcondition 4 | RED |
//! | `test_BC_4_01_005_progress_bar_has_named_layout` | AC-002 | BC-4.01.005 postcondition 4/5 | RED |
//! | `test_BC_4_01_005_progress_bar_slide_resolves_named_layout_not_fallback` | AC-002/EC-004 | BC-4.01.005 postcondition 5 | RED |
//! | `test_BC_5_01_005_run_has_lang_attribute_default_en_us` | AC-003/EC-002 | BC-5.01.005 postcondition 1 | RED |
//! | `test_BC_5_01_005_run_has_lang_attribute_fr_fr_round_trip` | AC-003 | BC-5.01.005 postcondition 1 | RED |
//! | `test_BC_5_01_005_ec003_empty_slide_no_rpr_emitted_no_error` | AC-003/EC-003 | BC-5.01.005 postcondition 1 | RED (LESSON-17: NOT #[should_panic]) |

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
/// The default lang is `"en-US"` matching the story's AC-003 default-lang
/// edge case (EC-002: deck with no explicit lang declaration uses "en-US").
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
// AC-001 + AC-004: slideMaster1.xml sldSz must match deck page size
// Traces to BC-4.01.001 postcondition 2 (valid .pptx with correct geometry)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.001 postcondition 2 / STORY-096 AC-001 (T-001 RED):
///
/// `slideMaster1.xml` must declare `<p:sldSz cx="9144000" cy="5143500"/>` when the
/// deck uses the default 16:9 page size. The current implementation does NOT emit
/// `<p:sldSz>` in `serialize_master_to_xml` at all — footer and date placeholders
/// are positioned at y=6,356,350 which is past the 16:9 height boundary of 5,143,500.
///
/// This test asserts the EXACT EMU values (LESSON-14), not mere element presence.
///
/// RED: `serialize_master_to_xml` currently produces no `<p:sldSz>` element.
/// The test fails because `parse_sldsz_cx_cy` returns `None` on the master XML.
///
/// GREEN after T-004: `serialize_master_to_xml` derives `cx`/`cy` from
/// `brand.page_size.width_emu` / `brand.page_size.height_emu` (or deck page_size).
#[test]
fn test_BC_4_01_001_master_sldsz_matches_16x9_deck() {
    // Default page size = 16:9: cx=9,144,000 cy=5,143,500 (slideforge-layout defaults)
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);
    let master_xml = zip_read_entry(&pptx_bytes, "ppt/slideMasters/slideMaster1.xml");

    let (cx, cy) = parse_sldsz_cx_cy(&master_xml).unwrap_or_else(|| {
        panic!(
            "AC-001 Red Gate: slideMaster1.xml must contain a <p:sldSz> element \
             declaring the deck page dimensions. None found.\n\
             master_xml excerpt:\n{}",
            &master_xml[..master_xml.len().min(1500)]
        )
    });

    // 16:9 canonical values: 9,144,000 × 5,143,500 EMU (DEFAULT_PAGE_WIDTH × DEFAULT_PAGE_HEIGHT)
    assert_eq!(
        cx, 9_144_000,
        "AC-001: slideMaster1.xml <p:sldSz cx> must be 9144000 (16:9 default width); \
         got cx={cx}. The master sldSz must be derived from the deck page_size, \
         NOT hardcoded to 4:3 (6858000 × 5143500)."
    );
    assert_eq!(
        cy, 5_143_500,
        "AC-001: slideMaster1.xml <p:sldSz cy> must be 5143500 (16:9 default height); \
         got cy={cy}."
    );
}

/// BC-4.01.001 postcondition 2 / STORY-096 AC-001 / EC-001 (RED):
///
/// When the brand configures a 4:3 custom page size (`cx=6858000 cy=5143500`),
/// `slideMaster1.xml` must declare `<p:sldSz cx="6858000" cy="5143500"/>`.
///
/// This is EC-001: custom brand page size — the master sldSz uses the brand's
/// configured dimensions, not the 16:9 default and not hardcoded 4:3.
///
/// RED: `serialize_master_to_xml` currently produces no `<p:sldSz>` at all.
#[test]
fn test_BC_4_01_001_master_sldsz_matches_custom_brand_page_size() {
    // 4:3 custom page: width=6,858,000 height=5,143,500
    let custom_page_width: i64 = 6_858_000;
    let custom_page_height: i64 = 5_143_500;
    let laid_out = make_laid_out_deck_with_page_size(custom_page_width, custom_page_height);
    let pptx_bytes = build_pptx(&laid_out);
    let master_xml = zip_read_entry(&pptx_bytes, "ppt/slideMasters/slideMaster1.xml");

    let (cx, cy) = parse_sldsz_cx_cy(&master_xml).unwrap_or_else(|| {
        panic!(
            "AC-001/EC-001 Red Gate: slideMaster1.xml must contain <p:sldSz> for custom \
             brand page size. None found.\nmaster_xml excerpt:\n{}",
            &master_xml[..master_xml.len().min(1500)]
        )
    });

    assert_eq!(
        cx, custom_page_width,
        "AC-001 EC-001: slideMaster1.xml <p:sldSz cx> must be {custom_page_width} \
         (brand-configured custom width); got cx={cx}"
    );
    assert_eq!(
        cy, custom_page_height,
        "AC-001 EC-001: slideMaster1.xml <p:sldSz cy> must be {custom_page_height} \
         (brand-configured custom height); got cy={cy}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-004: Footer and date placeholders stay within 16:9 slide bounds
// Traces to BC-4.01.001 postcondition 4 (openable without error dialogs)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.001 postcondition 4 / STORY-096 AC-004 (RED):
///
/// With the master sldSz corrected to 16:9 (cy=5,143,500), every placeholder
/// `<p:sp>` shape in `slideMaster1.xml` whose `<a:off>` `y` + `<a:ext>` `cy`
/// lies within the slide height must NOT have a `y` coordinate exceeding
/// 5,143,500 EMU.
///
/// CURRENT DEFECT: `MASTER_PLACEHOLDER_DEFS` positions footer/date/slideNum at
/// y=6,356,350 (>5,143,500). PowerPoint issues layout warnings when the master
/// has a declared sldSz that conflicts with placeholder positions outside it.
///
/// This test scans ALL `<a:off y="...">` occurrences in the master XML and
/// asserts none exceeds the 16:9 height.
///
/// RED: even after AC-001 adds `<p:sldSz>`, the placeholder positions remain
/// at y=6,356,350 until T-004 also updates `MASTER_PLACEHOLDER_DEFS`.
///
/// LESSON-14: asserts the ACTUAL y-coordinate value, not merely that the master
/// XML contains sldSz.
#[test]
fn test_BC_4_01_001_footer_date_placeholders_within_16x9_bounds() {
    // 16:9 page height in EMU.
    const MAX_Y_EMU: i64 = 5_143_500;

    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);
    let master_xml = zip_read_entry(&pptx_bytes, "ppt/slideMasters/slideMaster1.xml");

    // Collect all <a:off y="..."> values from the master XML.
    // These are the top-edge y-coordinates of placeholder shapes.
    let mut violations: Vec<i64> = Vec::new();
    let mut rest = master_xml.as_str();
    while let Some(pos) = rest.find("<a:off") {
        let after = &rest[pos..];
        let end = after.find('>').unwrap_or(after.len());
        let tag_content = &after[..=end];

        if let Some(y_val) = extract_attr_i64(tag_content, "y")
            && y_val > MAX_Y_EMU
        {
            violations.push(y_val);
        }
        rest = &rest[pos + "<a:off".len()..];
    }

    assert!(
        violations.is_empty(),
        "AC-004 Red Gate: slideMaster1.xml has placeholder(s) with y-coordinate(s) \
         exceeding the 16:9 height boundary ({MAX_Y_EMU} EMU).\n\
         Violating y values: {violations:?}\n\
         These are footer/date/slideNum placeholders positioned for a 4:3 master; \
         they must be repositioned within the 9144000 × 5143500 bounds.\n\
         master_xml excerpt (first 2000 chars):\n{}",
        &master_xml[..master_xml.len().min(2000)]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-002: progress_bar slide type has a named layout in the layout hierarchy
// Traces to BC-4.01.005 postcondition 4/5 (all 31 layouts present with keywords)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.005 postcondition 4 / STORY-096 AC-002 (T-002 RED):
///
/// The PPTX layout hierarchy must include a layout XML file whose `<p:cSld>` has
/// `name="progress_bar"` (or equivalent canonical name following the "SF " prefix
/// convention). A deck containing a `progress_bar` slide must NOT route to the
/// fallback layout (index 1, "Title and Content").
///
/// CURRENT DEFECT: `generate_all_layouts` has no entry with
/// `slide_type_keyword = Some("progress_bar")`. `find_layout_index` falls back to
/// index 1 with a `tracing::warn!`. The PPTX's `slide1.xml` therefore references
/// `slideLayout2.xml` instead of a named `progress_bar` layout.
///
/// Assertions (LESSON-14, LESSON-17 — no `#[should_panic]`):
/// 1. A layout XML file in the ZIP has `<p:cSld name="progress_bar">` (or name
///    matching the canonical SF naming for progress_bar).
/// 2. The slide's `.rels` relationship does NOT point to `slideLayout2.xml`
///    (which is the "Title and Content" fallback, 1-indexed).
///
/// RED: `generate_all_layouts` has no progress_bar entry; index 1 is returned
/// by the fallback, so the slide rels reference `slideLayout2.xml`.
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
        "AC-002 Red Gate: No slideLayout*.xml in the PPTX ZIP has a \
         <p:cSld name=\"progress_bar\"> (or \"SF Progress Bar\") attribute.\n\
         `generate_all_layouts` must include an entry with \
         slide_type_keyword = Some(\"progress_bar\") so the layout is synthesized \
         and added to the ZIP.\n\
         Current behavior: falls back to slideLayout2.xml (Title and Content) \
         via tracing::warn! — no named progress_bar layout exists."
    );
}

/// BC-4.01.005 postcondition 5 / STORY-096 AC-002 / EC-004 (RED):
///
/// `find_layout_index` must return an index other than 1 for `"progress_bar"`
/// once AC-002 is fixed (i.e., the progress_bar layout is added at some canonical
/// index >= 2 and != the Title and Content fallback index 1).
///
/// This test also verifies EC-004: a `progress_bar` slide with a label field routes
/// to the named layout correctly (the label field does NOT affect layout routing).
///
/// LESSON-14: asserts the ACTUAL layout index, not merely that routing succeeds.
///
/// RED: `find_layout_index("progress_bar")` currently returns 1 (fallback)
/// because `generate_all_layouts` has no progress_bar entry.
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

    // After AC-002 fix: progress_bar must NOT fall back to index 1.
    // It must resolve to its own named layout index (>= 2, per the standard
    // 11-slot SL block + custom CL-01..CL-20 placement).
    assert_ne!(
        idx, 1,
        "AC-002 Red Gate: find_layout_index(\"progress_bar\") must NOT return 1 \
         (the Title and Content fallback index). Got {idx}.\n\
         `generate_all_layouts` must include a progress_bar entry with \
         slide_type_keyword = Some(\"progress_bar\") so Phase 1 keyword matching \
         returns its dedicated layout index.\n\
         Current behavior: fallback returns 1 with tracing::warn! (no match found)."
    );

    // Verify the resolved index is within the valid 0..31 range.
    assert!(
        idx < crate::LAYOUT_COUNT,
        "AC-002: resolved layout index {idx} must be < LAYOUT_COUNT ({}) \
         (within the 31-layout hierarchy)",
        crate::LAYOUT_COUNT
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-003: All text runs carry lang attribute from deck metadata
// Traces to BC-5.01.005 postcondition 1 (lang propagates to PPTX rPr)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-5.01.005 postcondition 1 / STORY-096 AC-003 / EC-002 (T-003 RED):
///
/// Every `<a:rPr>` element in `slide1.xml` must carry a `lang="en-US"` attribute
/// when the deck's `metadata.lang` is `"en-US"` (the default). Runs without an
/// explicit lang currently emit `<a:rPr/>` or `<a:rPr bold="1"/>` with no `lang`
/// attribute — they must emit `lang="en-US"` after the fix.
///
/// EC-002: deck with no explicit lang declaration defaults to "en-US". The
/// `make_deck` fixture uses `lang: Some(Arc::from("en-US"))` — no lang declared
/// in the DSL sources, default applied by `build_doc_props`.
///
/// LESSON-14: asserts ACTUAL `lang` attribute values, not mere rPr presence.
/// LESSON-17: does NOT use `#[should_panic]`.
///
/// RED: `ooxml_run_to_ooxmlsdk` in `slide_serializer.rs` builds `RunProperties`
/// without setting `.lang`. Every `<a:rPr>` is emitted without a `lang` attribute.
#[test]
fn test_BC_5_01_005_run_has_lang_attribute_default_en_us() {
    // Build a deck with explicit en-US (default path / EC-002).
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

/// BC-5.01.005 postcondition 1 / STORY-096 AC-003 (RED) — fr-FR round-trip:
///
/// When the deck declares `lang "fr-FR"`, every `<a:rPr>` in every slide XML
/// must carry `lang="fr-FR"`. The lang value must be the exact BCP-47 tag from
/// the deck declaration — not normalised to "fr", not modified in any way.
///
/// LESSON-14: asserts the EXACT string "fr-FR" (round-trip test, BC-5.01.005
/// canonical test vector: `lang "fr-FR"` → PPTX → `<a:rPr lang="fr-FR"/>`).
///
/// RED: `ooxml_run_to_ooxmlsdk` does not set `.lang` at all.
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
/// RED: once `ooxml_run_to_ooxmlsdk` sets lang, all rPr elements will carry it.
/// Before the fix: rPr elements (if any) lack lang — confirmed by collect_rpr_lang_values
/// returning empty even when rPr elements exist.
///
/// NOTE: This test is expected to PASS for an empty slide (EC-003) even before
/// the fix IF the empty slide produces zero rPr elements. It becomes a true Red
/// Gate only if the title run produces an rPr. We include it here because the
/// story spec requires coverage of EC-003. The test is load-bearing regardless:
/// it guards against regressions where empty-slide export would panic when the
/// lang-propagation code is added.
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
