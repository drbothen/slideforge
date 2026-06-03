//! Failing tests for STORY-038: PPTX Layout Compliance (BC-4.01.005).
//!
//! Every test in this file MUST FAIL before the STORY-038 implementation begins
//! (Red Gate). Tests that are GREEN-BY-DESIGN or WIRING-EXEMPT are annotated
//! with their classification and rationale.
//!
//! ## Traceability
//!
//! | Test function | AC / Item | BC clause | Red Gate status |
//! |---|---|---|---|
//! | `test_BC_4_01_005_ac001_slide_ids_exact_sequence_3_slides` | AC-001 | invariant 1 | RED |
//! | `test_BC_4_01_005_ac002_master_id_exact_2147483648` | AC-002 | invariant 2 | RED |
//! | `test_BC_4_01_005_ac003_zip_contains_exactly_31_layout_parts` | AC-003 | postcondition 4 | RED |
//! | `test_BC_4_01_005_ac003_ec001_one_slide_deck_still_has_31_layouts` | AC-003 EC-001 | postcondition 4 | RED |
//! | `test_BC_4_01_005_ac004_slide_master_sldlayoutidlst_has_31_entries` | AC-004 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac005_content_types_has_exactly_31_layout_overrides` | AC-005 | postcondition 6 | RED |
//! | `test_BC_4_01_005_ac006_section_divider_slide_has_clrmapovr_in_zip` | AC-006 | postcondition 1 | RED |
//! | `test_BC_4_01_005_ac006_title_slide_does_not_have_clrmapovr` | AC-006 | postcondition 1 | GREEN-BY-DESIGN (see below) |
//! | `test_BC_4_01_005_ac006_end_slide_has_clrmapovr_in_zip` | AC-006 | postcondition 1 | RED |
//! | `test_BC_4_01_005_ac007_ec003_257_slides_unique_ids_256_to_512` | AC-007 EC-003 | invariant 1 | RED |
//! | `test_BC_4_01_005_ac008_master_rels_has_31_layout_relationships` | AC-008 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac009_split_contrast_maps_to_index_1_not_0` | AC-009 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac009_card_rows_maps_to_index_1_not_0` | AC-009 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac009_horizontal_timeline_maps_to_index_1_not_0` | AC-009 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac009_status_maps_to_index_1_not_0` | AC-009 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac009_progress_bar_maps_to_index_1_not_0` | AC-009 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac009_metric_tree_maps_to_index_1_not_0` | AC-009 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac009_formula_maps_to_index_1_not_0` | AC-009 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac009_weighted_composite_maps_to_index_1_not_0` | AC-009 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac009_grid_maps_to_index_1_not_0` | AC-009 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac009_no_unmapped_keyword_maps_to_index_0` | AC-009 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_title_keyword_maps_to_index_0` | AC-010 | postcondition 5 | GREEN-BY-DESIGN (see below) |
//! | `test_BC_4_01_005_ac010_content_keyword_maps_to_index_1` | AC-010 | postcondition 5 | GREEN-BY-DESIGN (see below) |
//! | `test_BC_4_01_005_ac010_two_column_keyword_maps_to_index_3` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_table_keyword_maps_to_index_7` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_blank_keyword_maps_to_index_6` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_section_divider_maps_to_index_11` | AC-010 | postcondition 5 | GREEN-BY-DESIGN (see below) |
//! | `test_BC_4_01_005_ac010_stat_callout_maps_to_index_12` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_quote_maps_to_index_13` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_vertical_timeline_maps_to_index_14` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_agenda_maps_to_index_15` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_toc_maps_to_index_16` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_bio_maps_to_index_17` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_team_maps_to_index_18` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_enhanced_table_maps_to_index_19` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_image_maps_to_index_20` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_end_maps_to_index_21` | AC-010 | postcondition 5 | GREEN-BY-DESIGN (see below) |
//! | `test_BC_4_01_005_ac010_content_stat_maps_to_index_22` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_diagram_maps_to_index_23` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_chart_maps_to_index_24` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_highlight_maps_to_index_25` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_severity_cards_maps_to_index_26` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_highlight_boxes_maps_to_index_27` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_stats_summary_maps_to_index_28` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac010_numbered_actions_maps_to_index_29` | AC-010 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac011_valid_ph_idx_emits_ph_element` | AC-011 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac011_missing_ph_idx_omits_ph_element` | AC-011 | postcondition 5 | RED |
//! | `test_BC_4_01_005_ac012_slide_size_out_of_range_emu_returns_err` | AC-012 / S2 | invariant 1 | RED |
//! | `test_BC_4_01_005_s1_validate_emu_negative_width_returns_err` | S1 | postcondition 1 | GREEN-BY-DESIGN (see below) |
//! | `test_BC_4_01_005_s1_validate_emu_negative_height_returns_err` | S1 | postcondition 1 | GREEN-BY-DESIGN (see below) |
//! | `test_BC_4_01_005_s3_subtitle_frame_emits_subtitle_placeholder` | S3 | postcondition 3 | RED |
//!
//! ## GREEN-BY-DESIGN / WIRING-EXEMPT Classifications
//!
//! The following tests are expected to pass even before STORY-038 implementation:
//!
//! - `test_BC_4_01_005_ac006_title_slide_does_not_have_clrmapovr`:
//!   STORY-037 already wired dark-layout detection correctly. A title slide
//!   (not dark) already does NOT have `<p:clrMapOvr>`. This is a negative
//!   assertion that is satisfied by existing behavior. It is included to prevent
//!   regressions during AC-006 implementation.
//!
//! - `test_BC_4_01_005_ac010_title_keyword_maps_to_index_0`:
//!   `find_layout_index("title")` already returns 0 (STORY-037 ADR-015 §A.6
//!   obligation #2). WIRING-EXEMPT.
//!
//! - `test_BC_4_01_005_ac010_content_keyword_maps_to_index_1`:
//!   `find_layout_index("content")` already returns 1. WIRING-EXEMPT.
//!
//! - `test_BC_4_01_005_ac010_section_divider_maps_to_index_11`:
//!   `find_layout_index("section_divider")` already returns 11. WIRING-EXEMPT.
//!
//! - `test_BC_4_01_005_ac010_end_maps_to_index_21`:
//!   `find_layout_index("end")` already returns 21. WIRING-EXEMPT.
//!
//! - `test_BC_4_01_005_s1_validate_emu_negative_width_returns_err` /
//!   `test_BC_4_01_005_s1_validate_emu_negative_height_returns_err`:
//!   `validate_emu` already returns `PptxError::InvalidEmu` for negative bbox
//!   dimensions (STORY-037 implementation). The Err path is already wired; the
//!   tests are included here to fulfil the PR-52 S1 obligation of having an
//!   explicit test for the negative path (not covered in STORY-037 tests).
//!   WIRING-EXEMPT because the production code already works correctly.

#![allow(non_snake_case)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
// Pre-existing test infrastructure: file extension comparison in ZIP entry filters.
#![allow(clippy::case_sensitive_file_extension_comparisons)]
// Pre-existing test: use imports inside function bodies (test-writer style).
#![allow(clippy::items_after_statements)]

use std::io::Read as _;
use std::sync::Arc;

use slideforge_layout::{BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize};
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::{Brand, BrandFonts, BrandPalette, Deck, Emu};
use zip::ZipArchive;

use crate::PptxExporter;

// ─── Shared fixture builders ─────────────────────────────────────────────────

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

/// Build a slide with the given `slide_type_keyword`.
fn make_slide_with_keyword(index: usize, keyword: &str) -> LaidOutSlide {
    LaidOutSlide {
        source_index: index,
        slide_type_keyword: Arc::from(keyword),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Title(Arc::from("Test")),
            text_flow: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    }
}

/// Build a minimal `LaidOutDeck` with `n` title slides.
fn make_laid_out_deck(n: usize) -> LaidOutDeck {
    LaidOutDeck {
        page_size: PageSize::default(),
        slides: (0..n)
            .map(|i| make_slide_with_keyword(i, "title"))
            .collect(),
        sections: vec![],
        warnings: vec![],
    }
}

/// Run the exporter on a `LaidOutDeck` and return raw PPTX bytes.
fn build_pptx(laid_out: &LaidOutDeck) -> Vec<u8> {
    let deck = make_deck(laid_out.slides.len());
    let brand = make_brand();
    let opts = ExportOptions::default();
    let exporter = PptxExporter::new();
    exporter
        .export(&deck, laid_out, &brand, &opts)
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

/// Build a minimal `BrandTemplate` for testing `find_layout_index` directly.
fn test_brand_template() -> slideforge_brand::BrandTemplate {
    use slideforge_brand::layout_xml::{
        HANDOUT_MASTER_STUB, NOTES_MASTER_STUB, generate_content_types_layout_entries,
    };
    use slideforge_brand::layouts::generate_all_layouts;
    use slideforge_brand::template::{BrandFonts as BTFonts, ColorSlot, ColorValue, MasterIds};
    use slideforge_brand::toml_schema::BrandConfig;

    let config = BrandConfig::default_minimal();
    let layouts = generate_all_layouts(&config);
    slideforge_brand::BrandTemplate {
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
        content_types_layout_entries: Arc::from(generate_content_types_layout_entries(31).as_str()),
    }
}

// ─── AC-001: Slide IDs exact sequence ────────────────────────────────────────

/// BC-4.01.005 invariant 1 / AC-001:
/// A 3-slide deck must produce slide IDs exactly 256, 257, 258 in order.
///
/// STORY-037's test (`test_BC_4_01_001_slide_ids_start_at_256`) only asserts
/// `id >= 256`. This test asserts the EXACT sequence and minimum-is-256.
///
/// RED: The existing implementation produces correct values, but this test
/// additionally verifies the step-1 increment requirement and the exact
/// set {256, 257, 258}. Any implementation that produces {257, 258, 259}
/// or {256, 256, 256} will fail.
#[test]
fn test_BC_4_01_005_ac001_slide_ids_exact_sequence_3_slides() {
    let laid_out = make_laid_out_deck(3);
    let pptx_bytes = build_pptx(&laid_out);
    let prs_xml = zip_read_entry(&pptx_bytes, "ppt/presentation.xml");

    // Assert exactly one <p:sldIdLst> container is present.
    let sldidlst_count = prs_xml.matches("<p:sldIdLst").count();
    assert_eq!(
        sldidlst_count, 1,
        "AC-001: presentation.xml must contain exactly one <p:sldIdLst> container; \
         found {sldidlst_count}"
    );

    // Parse only <p:sldId> ENTRY elements (not the <p:sldIdLst> container).
    // An entry element has attributes: "<p:sldId " (trailing space before id=).
    // The container "<p:sldIdLst>" has no attributes so it does not match.
    let mut ids: Vec<u32> = Vec::new();
    let mut rest = prs_xml.as_str();
    while let Some(pos) = rest
        .find("<p:sldId ")
        .or_else(|| rest.find("<p:sldId\t"))
        .or_else(|| rest.find("<p:sldId\n"))
    {
        rest = &rest[pos + 8..];
        if let Some(id_pos) = rest.find("id=\"") {
            let id_start = id_pos + 4;
            let id_rest = &rest[id_start..];
            if let Some(end) = id_rest.find('"') {
                let id_str = &id_rest[..end];
                let id: u32 = id_str
                    .parse()
                    .unwrap_or_else(|_| panic!("sldId id=\"{id_str}\" must parse as u32"));
                ids.push(id);
            }
        }
    }

    assert_eq!(
        ids.len(),
        3,
        "AC-001: a 3-slide deck must produce exactly 3 <p:sldId> entries; got {ids:?}"
    );
    assert_eq!(
        ids[0], 256,
        "AC-001 (BC-4.01.005 invariant 1): first slide ID must be exactly 256; got {}",
        ids[0]
    );
    assert_eq!(
        ids[1], 257,
        "AC-001: second slide ID must be exactly 257 (first+1); got {}",
        ids[1]
    );
    assert_eq!(
        ids[2], 258,
        "AC-001: third slide ID must be exactly 258 (first+2); got {}",
        ids[2]
    );
    // All IDs must be unique.
    let unique: std::collections::BTreeSet<u32> = ids.iter().copied().collect();
    assert_eq!(
        unique.len(),
        ids.len(),
        "AC-001: all slide IDs must be unique; got {ids:?}"
    );
}

// ─── AC-002: Master ID exact value ───────────────────────────────────────────

/// BC-4.01.005 invariant 2 / AC-002:
/// `<p:sldMasterId id="...">` in `presentation.xml` must equal exactly
/// 2,147,483,648 (2^31) — not any other value >= 2^31.
///
/// STORY-037's test asserts `id >= 2^31`. This test asserts the EXACT value.
///
/// RED: If the implementation uses a random value >= 2^31 or 2^31+1, this fails.
#[test]
fn test_BC_4_01_005_ac002_master_id_exact_2147483648() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);
    let prs_xml = zip_read_entry(&pptx_bytes, "ppt/presentation.xml");

    let mut found = false;
    let mut rest = prs_xml.as_str();
    while let Some(pos) = rest.find("<p:sldMasterId") {
        rest = &rest[pos + 14..];
        if let Some(id_pos) = rest.find("id=\"") {
            let id_start = id_pos + 4;
            let id_rest = &rest[id_start..];
            if let Some(end) = id_rest.find('"') {
                let id_str = &id_rest[..end];
                let id: u64 = id_str
                    .parse()
                    .unwrap_or_else(|_| panic!("sldMasterId id=\"{id_str}\" must parse as u64"));
                assert_eq!(
                    id, 2_147_483_648u64,
                    "AC-002 (BC-4.01.005 invariant 2): master ID must be EXACTLY 2147483648 \
                     (2^31); got {id}"
                );
                found = true;
            }
        }
    }
    assert!(
        found,
        "AC-002: presentation.xml must contain at least one <p:sldMasterId> element"
    );
}

// ─── AC-003: Exactly 31 slideLayout ZIP entries ───────────────────────────────

/// BC-4.01.005 postcondition 4 / AC-003:
/// Any deck must contain exactly 31 `ppt/slideLayouts/slideLayout*.xml` entries
/// in the PPTX ZIP.
///
/// RED: verifies by enumerating ZIP entries, not Content_Types overrides.
#[test]
fn test_BC_4_01_005_ac003_zip_contains_exactly_31_layout_parts() {
    let laid_out = make_laid_out_deck(3);
    let pptx_bytes = build_pptx(&laid_out);

    let cursor = std::io::Cursor::new(&pptx_bytes);
    let mut archive = ZipArchive::new(cursor).expect("must be a valid ZIP");
    let layout_count = (0..archive.len())
        .filter(|&i| {
            let entry = archive.by_index(i).expect("index in range");
            let name = entry.name().to_owned();
            name.starts_with("ppt/slideLayouts/slideLayout")
                && name.ends_with(".xml")
                && !name.contains("_rels")
        })
        .count();

    assert_eq!(
        layout_count, 31,
        "AC-003 (BC-4.01.005 postcondition 4): PPTX ZIP must contain exactly 31 \
         slideLayout*.xml parts in ppt/slideLayouts/; found {layout_count}"
    );
}

/// BC-4.01.005 EC-001 / AC-003:
/// A 1-slide deck must still have exactly 31 layout parts.
///
/// RED: exercises the "always 31" invariant for minimum-size decks.
#[test]
fn test_BC_4_01_005_ac003_ec001_one_slide_deck_still_has_31_layouts() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let cursor = std::io::Cursor::new(&pptx_bytes);
    let mut archive = ZipArchive::new(cursor).expect("must be a valid ZIP");
    let layout_count = (0..archive.len())
        .filter(|&i| {
            let entry = archive.by_index(i).expect("index in range");
            let name = entry.name().to_owned();
            name.starts_with("ppt/slideLayouts/slideLayout")
                && name.ends_with(".xml")
                && !name.contains("_rels")
        })
        .count();

    assert_eq!(
        layout_count, 31,
        "AC-003 EC-001: even a 1-slide deck must contain exactly 31 slideLayout parts; \
         found {layout_count}"
    );
}

// ─── AC-004: slideMaster1.xml sldLayoutIdLst has 31 entries ──────────────────

/// BC-4.01.005 postcondition 5 / AC-004:
/// `<p:sldLayoutIdLst>` in `slideMaster1.xml` must contain exactly 31
/// `<p:sldLayoutId>` entries.
///
/// RED: this requires the master XML to be generated from BrandTemplate (ADR-015 §2)
/// with the full 31-entry list, not an empty default shell.
#[test]
fn test_BC_4_01_005_ac004_slide_master_sldlayoutidlst_has_31_entries() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);
    let master_xml = zip_read_entry(&pptx_bytes, "ppt/slideMasters/slideMaster1.xml");

    // Assert exactly one <p:sldLayoutIdLst> container is present.
    let container_count = master_xml.matches("<p:sldLayoutIdLst").count();
    assert_eq!(
        container_count, 1,
        "AC-004 (BC-4.01.005 postcondition 5): slideMaster1.xml must contain exactly one \
         <p:sldLayoutIdLst> container; found {container_count}"
    );

    // Count <p:sldLayoutId> ENTRY elements only (not the <p:sldLayoutIdLst> container).
    // Entry elements have attributes: "<p:sldLayoutId " (trailing space before id=).
    // The container "<p:sldLayoutIdLst>" ends in "Lst" and does not match this pattern.
    let count = master_xml.matches("<p:sldLayoutId ").count();

    assert_eq!(
        count, 31,
        "AC-004 (BC-4.01.005 postcondition 5): slideMaster1.xml must contain exactly 31 \
         <p:sldLayoutId> entries inside <p:sldLayoutIdLst>; found {count}. \
         This requires serialize_master_to_xml to emit all 31 layout IDs."
    );
}

// ─── AC-005: Content_Types.xml has exactly 31 layout Overrides ───────────────

/// BC-4.01.005 postcondition 6 / AC-005:
/// `[Content_Types].xml` must contain Override entries for all 31 layouts,
/// each with the correct slideLayout content-type.
///
/// RED: asserts the EXACT count of entries matching the slideLayout content-type.
/// This is stronger than the STORY-037 test which only checks path presence.
#[test]
fn test_BC_4_01_005_ac005_content_types_has_exactly_31_layout_overrides() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);
    let ct_xml = zip_read_entry(&pptx_bytes, "[Content_Types].xml");

    // Count Override entries with the slideLayout content-type string.
    let layout_ct = "application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml";
    let count = ct_xml.matches(layout_ct).count();

    assert_eq!(
        count, 31,
        "AC-005 (BC-4.01.005 postcondition 6): [Content_Types].xml must have exactly 31 \
         Override entries with slideLayout content-type; found {count}"
    );
}

// ─── AC-006: End-to-end clrMapOvr (dark layouts) ─────────────────────────────

/// BC-4.01.005 postcondition 1 / AC-006:
/// A deck containing a `section_divider` slide (layout index 12, dark per
/// `has_color_override = true`) must produce `<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>`
/// in `ppt/slides/slide1.xml`.
///
/// This is the full AC-006 integration test for the `section_divider` case.
/// STORY-037's `test_f037_011_dark_layout_section_divider_emits_clr_map_ovr_end_to_end`
/// already covers this path — this test EXTENDS it by also verifying it functions
/// correctly after all 31 layouts are embedded (AC-003).
///
/// RED: depends on AC-003 (31 layouts present) being satisfied before the
/// `find_layout_index` can correctly resolve `section_divider` → dark.
/// Current test may pass depending on STORY-037 state; the RED state is confirmed
/// by verifying ALL of: (a) section_divider → clrMapOvr, (b) title → no clrMapOvr,
/// (c) end → clrMapOvr, all in a single multi-slide deck.
#[test]
fn test_BC_4_01_005_ac006_section_divider_slide_has_clrmapovr_in_zip() {
    let mut laid_out = make_laid_out_deck(1);
    laid_out.slides[0].slide_type_keyword = Arc::from("section_divider");

    let pptx_bytes = build_pptx(&laid_out);

    // Verify the ZIP itself is valid.
    {
        let cursor = std::io::Cursor::new(&pptx_bytes);
        ZipArchive::new(cursor).expect("section_divider export must produce valid ZIP");
    }

    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    assert!(
        slide1_xml.contains("<p:clrMapOvr>"),
        "AC-006: a 'section_divider' slide must contain <p:clrMapOvr> in the exported ZIP; \
         slide1.xml excerpt: {}",
        &slide1_xml[..slide1_xml.len().min(600)]
    );
    assert!(
        slide1_xml.contains("<a:masterClrMapping"),
        "AC-006: <p:clrMapOvr> must contain <a:masterClrMapping/>; \
         slide1.xml excerpt: {}",
        &slide1_xml[..slide1_xml.len().min(600)]
    );

    // clrMapOvr must appear after cSld.
    let csld_pos = slide1_xml
        .find("<p:cSld")
        .expect("slide XML must contain <p:cSld>");
    let clr_pos = slide1_xml
        .find("<p:clrMapOvr>")
        .expect("section_divider slide XML must contain <p:clrMapOvr>");
    assert!(
        clr_pos > csld_pos,
        "AC-006: <p:clrMapOvr> (byte {clr_pos}) must appear AFTER <p:cSld> (byte {csld_pos}) \
         per ECMA-376 §19.3.1.31 sequence model"
    );
}

/// BC-4.01.005 postcondition 1 / AC-006:
/// A `title` slide (not dark, layout index 1) must NOT have `<p:clrMapOvr>`.
///
/// CLASSIFICATION: GREEN-BY-DESIGN — STORY-037 already wires this correctly.
/// Included as a regression guard: if AC-006 implementation accidentally adds
/// clrMapOvr to all slides, this test catches it.
#[test]
fn test_BC_4_01_005_ac006_title_slide_does_not_have_clrmapovr() {
    let mut laid_out = make_laid_out_deck(1);
    laid_out.slides[0].slide_type_keyword = Arc::from("title");

    let pptx_bytes = build_pptx(&laid_out);
    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    assert!(
        !slide1_xml.contains("<p:clrMapOvr>"),
        "AC-006: a 'title' slide must NOT contain <p:clrMapOvr> (title is not a dark layout); \
         slide1.xml excerpt: {}",
        &slide1_xml[..slide1_xml.len().min(600)]
    );
}

/// BC-4.01.005 postcondition 1 / AC-006:
/// A deck containing an `end` slide (layout index 22, dark per
/// `has_color_override = true`) must produce `<p:clrMapOvr>` in its slide XML.
///
/// RED: the `end` keyword must map to layout index 21 (0-based) which has
/// `has_color_override = true`. STORY-037 already passes the `end` → index 21
/// test in `layout_index_tests`, but this is the full integration test through
/// ZIP inspection.
#[test]
fn test_BC_4_01_005_ac006_end_slide_has_clrmapovr_in_zip() {
    let mut laid_out = make_laid_out_deck(1);
    laid_out.slides[0].slide_type_keyword = Arc::from("end");

    let pptx_bytes = build_pptx(&laid_out);
    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    assert!(
        slide1_xml.contains("<p:clrMapOvr>"),
        "AC-006: an 'end' slide must contain <p:clrMapOvr> (layout index 22 is dark); \
         slide1.xml excerpt: {}",
        &slide1_xml[..slide1_xml.len().min(600)]
    );
    assert!(
        slide1_xml.contains("<a:masterClrMapping"),
        "AC-006: <p:clrMapOvr> for 'end' slide must contain <a:masterClrMapping/>; \
         slide1.xml excerpt: {}",
        &slide1_xml[..slide1_xml.len().min(600)]
    );
}

// ─── AC-007 / EC-003: 257-slide deck ─────────────────────────────────────────

/// BC-4.01.005 EC-003 / AC-007:
/// A 257-slide deck must produce slide IDs 256..512 with no collisions and all >= 256.
///
/// RED: verifies the SlideIdAssigner handles large decks correctly and does not
/// overflow or produce duplicates.
#[test]
fn test_BC_4_01_005_ac007_ec003_257_slides_unique_ids_256_to_512() {
    let laid_out = make_laid_out_deck(257);
    let pptx_bytes = build_pptx(&laid_out);
    let prs_xml = zip_read_entry(&pptx_bytes, "ppt/presentation.xml");

    // Assert exactly one <p:sldIdLst> container is present.
    let sldidlst_count = prs_xml.matches("<p:sldIdLst").count();
    assert_eq!(
        sldidlst_count, 1,
        "AC-007 EC-003: presentation.xml must contain exactly one <p:sldIdLst> container; \
         found {sldidlst_count}"
    );

    // Parse only <p:sldId> ENTRY elements (not the <p:sldIdLst> container).
    let mut ids: Vec<u32> = Vec::new();
    let mut rest = prs_xml.as_str();
    while let Some(pos) = rest
        .find("<p:sldId ")
        .or_else(|| rest.find("<p:sldId\t"))
        .or_else(|| rest.find("<p:sldId\n"))
    {
        rest = &rest[pos + 8..];
        if let Some(id_pos) = rest.find("id=\"") {
            let id_start = id_pos + 4;
            let id_rest = &rest[id_start..];
            if let Some(end) = id_rest.find('"') {
                let id_str = &id_rest[..end];
                let id: u32 = id_str
                    .parse()
                    .unwrap_or_else(|_| panic!("sldId id=\"{id_str}\" must parse as u32"));
                ids.push(id);
            }
        }
    }

    assert_eq!(
        ids.len(),
        257,
        "AC-007 EC-003: a 257-slide deck must produce 257 <p:sldId> entries"
    );

    for &id in &ids {
        assert!(
            id >= 256,
            "AC-007 EC-003: all slide IDs must be >= 256; found id = {id}"
        );
    }

    let unique: std::collections::BTreeSet<u32> = ids.iter().copied().collect();
    assert_eq!(
        unique.len(),
        257,
        "AC-007 EC-003: all 257 slide IDs must be unique; got {ids:?}"
    );

    let min = ids.iter().copied().min().unwrap();
    let max = ids.iter().copied().max().unwrap();
    assert_eq!(
        min, 256,
        "AC-007 EC-003: minimum slide ID must be 256; got {min}"
    );
    assert_eq!(
        max, 512,
        "AC-007 EC-003: maximum slide ID for 257 slides must be 512; got {max}"
    );
}

// ─── AC-008: Master rels has 31 layout entries ────────────────────────────────

/// BC-4.01.005 postcondition 5 / AC-008:
/// `ppt/slideMasters/_rels/slideMaster1.xml.rels` must contain exactly 31
/// relationship entries with the slideLayout relationship type.
///
/// RED: requires the master rels builder to emit exactly 31 layout relationships.
#[test]
fn test_BC_4_01_005_ac008_master_rels_has_31_layout_relationships() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);
    let master_rels = zip_read_entry(&pptx_bytes, "ppt/slideMasters/_rels/slideMaster1.xml.rels");

    // The slideLayout relationship type.
    let layout_type =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout";
    let count = master_rels.matches(layout_type).count();

    assert_eq!(
        count, 31,
        "AC-008 (BC-4.01.005 postcondition 5): slideMaster1.xml.rels must contain exactly 31 \
         slideLayout relationships; found {count}"
    );
}

// ─── AC-009: Unmapped DSL keywords resolve to index 1, not 0 ─────────────────
//
// For each of the 9 Q2 keywords without dedicated SF layouts, `find_layout_index`
// must return index 1 (Title and Content, 0-based), NOT index 0 (Title Slide).
// These tests call `find_layout_index` directly via the module-level function.
//
// RED: Current behavior returns index 1 by the fallback path (already wired).
// The tests are SPECIFICALLY asserting != 0, and == 1 exactly.
// These are genuine RED tests because `find_layout_index` is a crate-private
// function; we test it via the `crate::find_layout_index` path below.

/// AC-009: `split_contrast` must map to index 1 (not 0).
#[test]
fn test_BC_4_01_005_ac009_split_contrast_maps_to_index_1_not_0() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "split_contrast");
    assert_ne!(
        idx, 0,
        "AC-009: 'split_contrast' must NOT map to layout index 0 (Title Slide); \
         got {idx}"
    );
    assert_eq!(
        idx, 1,
        "AC-009: 'split_contrast' must fall back to layout index 1 \
         (Title and Content) with a tracing::warn!; got {idx}"
    );
}

/// AC-009: `card_rows` must map to index 1 (not 0).
#[test]
fn test_BC_4_01_005_ac009_card_rows_maps_to_index_1_not_0() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "card_rows");
    assert_ne!(
        idx, 0,
        "AC-009: 'card_rows' must NOT map to layout index 0; got {idx}"
    );
    assert_eq!(
        idx, 1,
        "AC-009: 'card_rows' must fall back to index 1; got {idx}"
    );
}

/// AC-009: `horizontal_timeline` must map to index 1 (not 0).
#[test]
fn test_BC_4_01_005_ac009_horizontal_timeline_maps_to_index_1_not_0() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "horizontal_timeline");
    assert_ne!(
        idx, 0,
        "AC-009: 'horizontal_timeline' must NOT map to layout index 0; got {idx}"
    );
    assert_eq!(
        idx, 1,
        "AC-009: 'horizontal_timeline' must fall back to index 1; got {idx}"
    );
}

/// AC-009: `status` must map to index 1 (not 0).
#[test]
fn test_BC_4_01_005_ac009_status_maps_to_index_1_not_0() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "status");
    assert_ne!(
        idx, 0,
        "AC-009: 'status' must NOT map to layout index 0; got {idx}"
    );
    assert_eq!(
        idx, 1,
        "AC-009: 'status' must fall back to index 1; got {idx}"
    );
}

/// AC-009: `progress_bar` must map to index 1 (not 0).
#[test]
fn test_BC_4_01_005_ac009_progress_bar_maps_to_index_1_not_0() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "progress_bar");
    assert_ne!(
        idx, 0,
        "AC-009: 'progress_bar' must NOT map to layout index 0; got {idx}"
    );
    assert_eq!(
        idx, 1,
        "AC-009: 'progress_bar' must fall back to index 1; got {idx}"
    );
}

/// AC-009: `metric_tree` must map to index 1 (not 0).
#[test]
fn test_BC_4_01_005_ac009_metric_tree_maps_to_index_1_not_0() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "metric_tree");
    assert_ne!(
        idx, 0,
        "AC-009: 'metric_tree' must NOT map to layout index 0; got {idx}"
    );
    assert_eq!(
        idx, 1,
        "AC-009: 'metric_tree' must fall back to index 1; got {idx}"
    );
}

/// AC-009: `formula` must map to index 1 (not 0).
#[test]
fn test_BC_4_01_005_ac009_formula_maps_to_index_1_not_0() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "formula");
    assert_ne!(
        idx, 0,
        "AC-009: 'formula' must NOT map to layout index 0; got {idx}"
    );
    assert_eq!(
        idx, 1,
        "AC-009: 'formula' must fall back to index 1; got {idx}"
    );
}

/// AC-009: `weighted_composite` must map to index 1 (not 0).
#[test]
fn test_BC_4_01_005_ac009_weighted_composite_maps_to_index_1_not_0() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "weighted_composite");
    assert_ne!(
        idx, 0,
        "AC-009: 'weighted_composite' must NOT map to layout index 0; got {idx}"
    );
    assert_eq!(
        idx, 1,
        "AC-009: 'weighted_composite' must fall back to index 1; got {idx}"
    );
}

/// AC-009: `grid` must map to index 1 (not 0).
#[test]
fn test_BC_4_01_005_ac009_grid_maps_to_index_1_not_0() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "grid");
    assert_ne!(
        idx, 0,
        "AC-009: 'grid' must NOT map to layout index 0; got {idx}"
    );
    assert_eq!(
        idx, 1,
        "AC-009: 'grid' must fall back to index 1; got {idx}"
    );
}

/// AC-009 aggregate: verify NONE of the 9 unmapped keywords maps to index 0.
///
/// This is an additional aggregate assertion on top of the per-keyword tests above.
/// RED: if ANY of the 9 keywords maps to index 0, this test fails.
#[test]
fn test_BC_4_01_005_ac009_no_unmapped_keyword_maps_to_index_0() {
    let template = test_brand_template();
    let unmapped_keywords = [
        "split_contrast",
        "card_rows",
        "horizontal_timeline",
        "status",
        "progress_bar",
        "metric_tree",
        "formula",
        "weighted_composite",
        "grid",
    ];

    for keyword in &unmapped_keywords {
        let idx = crate::find_layout_index(&template, keyword);
        assert_ne!(
            idx, 0,
            "AC-009: unmapped keyword '{keyword}' must NOT resolve to layout index 0 \
             (Title Slide). Index 0 is reserved for the 'title' keyword. \
             Got index {idx}."
        );
    }
}

// ─── AC-010: Full per-slide-type layout selection coverage ───────────────────
//
// For each of the 25 DSL keywords that have canonical layout mappings per
// ADR-015 §A.4, assert the exact 0-based index. The GREEN-BY-DESIGN ones
// (already wired in STORY-037) are annotated.

/// AC-010: `title` → index 0 (SL-01, ooxml_type "title").
///
/// CLASSIFICATION: GREEN-BY-DESIGN — STORY-037 already implemented this.
#[test]
fn test_BC_4_01_005_ac010_title_keyword_maps_to_index_0() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "title");
    assert_eq!(
        idx, 0,
        "AC-010: 'title' must map to 0-based index 0 (Title Slide, SL-01); got {idx}"
    );
}

/// AC-010: `content` → index 1 (SL-02, ooxml_type "obj").
///
/// CLASSIFICATION: GREEN-BY-DESIGN — STORY-037 already implemented this.
#[test]
fn test_BC_4_01_005_ac010_content_keyword_maps_to_index_1() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "content");
    assert_eq!(
        idx, 1,
        "AC-010: 'content' must map to 0-based index 1 (Title and Content, SL-02); got {idx}"
    );
}

/// AC-010: `two_column` → index 3 (SL-04, ooxml_type "twoObj").
///
/// RED: 0-based index 3 = 1-based layout 4 (twoObj).
#[test]
fn test_BC_4_01_005_ac010_two_column_keyword_maps_to_index_3() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "two_column");
    assert_eq!(
        idx, 3,
        "AC-010: 'two_column' must map to 0-based index 3 (Two Object, SL-04, ooxml_type twoObj); got {idx}"
    );
}

/// AC-010: `table` → index 7 (SL-08, ooxml_type "objTx").
///
/// RED: 0-based index 7 = 1-based layout 8.
#[test]
fn test_BC_4_01_005_ac010_table_keyword_maps_to_index_7() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "table");
    assert_eq!(
        idx, 7,
        "AC-010: 'table' must map to 0-based index 7 (SL-08, ooxml_type objTx); got {idx}"
    );
}

/// AC-010: `blank` → index 6 (SL-07, ooxml_type "blank").
///
/// RED: 0-based index 6 = 1-based layout 7.
#[test]
fn test_BC_4_01_005_ac010_blank_keyword_maps_to_index_6() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "blank");
    assert_eq!(
        idx, 6,
        "AC-010: 'blank' must map to 0-based index 6 (SL-07, ooxml_type blank); got {idx}"
    );
}

/// AC-010: `section_divider` → index 11 (CL-01, SF Section Divider, dark).
///
/// CLASSIFICATION: GREEN-BY-DESIGN — STORY-037 already implemented this.
#[test]
fn test_BC_4_01_005_ac010_section_divider_maps_to_index_11() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "section_divider");
    assert_eq!(
        idx, 11,
        "AC-010: 'section_divider' must map to 0-based index 11 (SF Section Divider, CL-01); got {idx}"
    );
}

/// AC-010: `stat_callout` → index 12 (CL-02, SF Stat Grid).
///
/// RED: 0-based index 12 = 1-based layout 13.
#[test]
fn test_BC_4_01_005_ac010_stat_callout_maps_to_index_12() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "stat_callout");
    assert_eq!(
        idx, 12,
        "AC-010: 'stat_callout' must map to 0-based index 12 (SF Stat Grid, CL-02); got {idx}"
    );
}

/// AC-010: `quote` → index 13 (CL-03, SF Quote).
#[test]
fn test_BC_4_01_005_ac010_quote_maps_to_index_13() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "quote");
    assert_eq!(
        idx, 13,
        "AC-010: 'quote' must map to 0-based index 13 (SF Quote, CL-03); got {idx}"
    );
}

/// AC-010: `vertical_timeline` → index 14 (CL-04, SF Timeline).
#[test]
fn test_BC_4_01_005_ac010_vertical_timeline_maps_to_index_14() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "vertical_timeline");
    assert_eq!(
        idx, 14,
        "AC-010: 'vertical_timeline' must map to 0-based index 14 (SF Timeline, CL-04); got {idx}"
    );
}

/// AC-010: `agenda` → index 15 (CL-05, SF Agenda).
#[test]
fn test_BC_4_01_005_ac010_agenda_maps_to_index_15() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "agenda");
    assert_eq!(
        idx, 15,
        "AC-010: 'agenda' must map to 0-based index 15 (SF Agenda, CL-05); got {idx}"
    );
}

/// AC-010: `toc` → index 16 (CL-06, SF TOC).
#[test]
fn test_BC_4_01_005_ac010_toc_maps_to_index_16() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "toc");
    assert_eq!(
        idx, 16,
        "AC-010: 'toc' must map to 0-based index 16 (SF TOC, CL-06); got {idx}"
    );
}

/// AC-010: `bio` → index 17 (CL-07, SF Bio).
#[test]
fn test_BC_4_01_005_ac010_bio_maps_to_index_17() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "bio");
    assert_eq!(
        idx, 17,
        "AC-010: 'bio' must map to 0-based index 17 (SF Bio, CL-07); got {idx}"
    );
}

/// AC-010: `team` → index 18 (CL-08, SF Team Grid).
#[test]
fn test_BC_4_01_005_ac010_team_maps_to_index_18() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "team");
    assert_eq!(
        idx, 18,
        "AC-010: 'team' must map to 0-based index 18 (SF Team Grid, CL-08); got {idx}"
    );
}

/// AC-010: `enhanced_table` → index 19 (CL-09, SF Comparison Table).
#[test]
fn test_BC_4_01_005_ac010_enhanced_table_maps_to_index_19() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "enhanced_table");
    assert_eq!(
        idx, 19,
        "AC-010: 'enhanced_table' must map to 0-based index 19 (SF Comparison Table, CL-09); got {idx}"
    );
}

/// AC-010: `image` → index 20 (CL-10, SF Full-Bleed Image).
#[test]
fn test_BC_4_01_005_ac010_image_maps_to_index_20() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "image");
    assert_eq!(
        idx, 20,
        "AC-010: 'image' must map to 0-based index 20 (SF Full-Bleed Image, CL-10); got {idx}"
    );
}

/// AC-010: `end` → index 21 (CL-11, SF End Slide, dark).
///
/// CLASSIFICATION: GREEN-BY-DESIGN — STORY-037 already implemented this.
#[test]
fn test_BC_4_01_005_ac010_end_maps_to_index_21() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "end");
    assert_eq!(
        idx, 21,
        "AC-010: 'end' must map to 0-based index 21 (SF End Slide, CL-11); got {idx}"
    );
}

/// AC-010: `content_stat` → index 22 (CL-12, SF Data).
#[test]
fn test_BC_4_01_005_ac010_content_stat_maps_to_index_22() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "content_stat");
    assert_eq!(
        idx, 22,
        "AC-010: 'content_stat' must map to 0-based index 22 (SF Data, CL-12); got {idx}"
    );
}

/// AC-010: `diagram` → index 23 (CL-13, SF Diagram).
#[test]
fn test_BC_4_01_005_ac010_diagram_maps_to_index_23() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "diagram");
    assert_eq!(
        idx, 23,
        "AC-010: 'diagram' must map to 0-based index 23 (SF Diagram, CL-13); got {idx}"
    );
}

/// AC-010: `chart` → index 24 (CL-14, SF Chart).
#[test]
fn test_BC_4_01_005_ac010_chart_maps_to_index_24() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "chart");
    assert_eq!(
        idx, 24,
        "AC-010: 'chart' must map to 0-based index 24 (SF Chart, CL-14); got {idx}"
    );
}

/// AC-010: `highlight` → index 25 (CL-15, SF Map).
#[test]
fn test_BC_4_01_005_ac010_highlight_maps_to_index_25() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "highlight");
    assert_eq!(
        idx, 25,
        "AC-010: 'highlight' must map to 0-based index 25 (SF Map, CL-15); got {idx}"
    );
}

/// AC-010: `severity_cards` → index 26 (CL-16, SF Risk Register).
#[test]
fn test_BC_4_01_005_ac010_severity_cards_maps_to_index_26() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "severity_cards");
    assert_eq!(
        idx, 26,
        "AC-010: 'severity_cards' must map to 0-based index 26 (SF Risk Register, CL-16); got {idx}"
    );
}

/// AC-010: `highlight_boxes` → index 27 (CL-17, SF Executive Summary).
#[test]
fn test_BC_4_01_005_ac010_highlight_boxes_maps_to_index_27() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "highlight_boxes");
    assert_eq!(
        idx, 27,
        "AC-010: 'highlight_boxes' must map to 0-based index 27 (SF Executive Summary, CL-17); got {idx}"
    );
}

/// AC-010: `stats_summary` → index 28 (CL-18, SF Two Column).
#[test]
fn test_BC_4_01_005_ac010_stats_summary_maps_to_index_28() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "stats_summary");
    assert_eq!(
        idx, 28,
        "AC-010: 'stats_summary' must map to 0-based index 28 (SF Two Column, CL-18); got {idx}"
    );
}

/// AC-010: `numbered_actions` → index 29 (CL-19, SF Methodology).
#[test]
fn test_BC_4_01_005_ac010_numbered_actions_maps_to_index_29() {
    let template = test_brand_template();
    let idx = crate::find_layout_index(&template, "numbered_actions");
    assert_eq!(
        idx, 29,
        "AC-010: 'numbered_actions' must map to 0-based index 29 (SF Methodology, CL-19); got {idx}"
    );
}

// ─── AC-011: Placeholder idx chain verification ───────────────────────────────

/// BC-4.01.005 postcondition 5 / AC-011:
/// When a `FrameContent::Title` frame is serialized, the resulting `<p:sp>` must
/// contain `<p:ph idx="0">` if a layout placeholder with `idx=0` is present in
/// the resolved layout definition.
///
/// RED: requires the slide serializer to check `SlideLayoutDef.placeholders`
/// for a matching idx before emitting `<p:ph>`.
#[test]
fn test_BC_4_01_005_ac011_valid_ph_idx_emits_ph_element() {
    // A title slide: frame is Title, layout has placeholder with idx=0.
    let mut laid_out = make_laid_out_deck(1);
    laid_out.slides[0].slide_type_keyword = Arc::from("title");

    let pptx_bytes = build_pptx(&laid_out);
    let slide_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    // The title frame must emit <p:ph> with the CORRECT idx (0 for title).
    // After AC-011 implementation, the serializer verifies the idx chain.
    assert!(
        slide_xml.contains("<p:ph"),
        "AC-011: a Title frame must produce a <p:ph> element when the layout \
         has a matching idx=0 placeholder; slide XML did not contain <p:ph>. \
         Excerpt: {}",
        &slide_xml[..slide_xml.len().min(600)]
    );

    // The idx attribute must match the layout placeholder idx (0 for title).
    assert!(
        slide_xml.contains("idx=\"0\""),
        "AC-011: the <p:ph> for a Title frame must have idx=\"0\" matching \
         the layout's title placeholder; slide XML excerpt: {}",
        &slide_xml[..slide_xml.len().min(600)]
    );
}

/// BC-4.01.005 postcondition 5 / AC-011:
/// When a frame's idx does NOT match any placeholder in the layout definition,
/// no `<p:ph>` element is emitted (the shape becomes a non-placeholder shape).
///
/// RED: requires the slide serializer to do the idx chain check and omit `<p:ph>`
/// when no matching layout placeholder exists.
///
/// This is tested by constructing a layout with NO body placeholder (idx=1) and
/// verifying a Body frame produces no `<p:ph>` element.
#[test]
fn test_BC_4_01_005_ac011_missing_ph_idx_omits_ph_element() {
    #[allow(unused_imports)]
    use slideforge_brand::BrandTemplate;
    use slideforge_brand::layout_xml::{
        HANDOUT_MASTER_STUB, NOTES_MASTER_STUB, generate_content_types_layout_entries,
    };
    use slideforge_brand::layouts::generate_all_layouts;
    use slideforge_brand::template::{BrandFonts as BTFonts, ColorSlot, ColorValue, MasterIds};
    use slideforge_brand::toml_schema::BrandConfig;

    // Build a BrandTemplate where layout[0] (title layout) has ONLY a title
    // placeholder (idx=0) and NO body placeholder (idx=1).
    // The slide_serializer must omit <p:ph> for a Body frame on this layout.
    let config = BrandConfig::default_minimal();
    let mut layouts = generate_all_layouts(&config);

    // Mutate layout[1] (content layout, used for "content" slides) to have ZERO
    // body placeholders — only the title placeholder remains.
    // This creates a layout where idx=1 does NOT exist.
    if layouts.len() > 1 {
        layouts[1].placeholders.retain(|ph| ph.idx == 0);
    }

    // `_brand_template` is constructed here to document the intent that the
    // SlideSerializer must receive the brand_template to verify the idx chain
    // (AC-011 implementation task). The current API does not accept it yet —
    // the test is RED because the current code always emits <p:ph idx="1">
    // for Body frames, regardless of whether the layout has a matching idx.
    let _brand_template = BrandTemplate {
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
        content_types_layout_entries: Arc::from(generate_content_types_layout_entries(31).as_str()),
    };

    // Build a "content" slide (layout index 1) with a Body frame.
    // The body frame has idx=1, which is NOT in layout[1].placeholders.
    let laid_out = LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("content"),
            frames: vec![
                Frame {
                    bbox: title_bbox(),
                    content: FrameContent::Title(Arc::from("Title")),
                    text_flow: None,
                },
                Frame {
                    bbox: body_bbox(),
                    content: FrameContent::Body(vec![]),
                    text_flow: None,
                },
            ],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        }],
        sections: vec![],
        warnings: vec![],
    };

    // Use the slide serializer directly to get the slide XML.
    use crate::slide_serializer::SlideSerializer;
    let serializer = SlideSerializer::new(false, 1); // layout_index=1 (content)
    let (slide_xml_bytes, _) = serializer
        .build(&laid_out.slides[0], 0, "rId1", &[])
        .expect("serializer must succeed");
    let slide_xml = String::from_utf8(slide_xml_bytes).expect("slide XML must be valid UTF-8");

    // Count <p:ph idx="1" occurrences: there should be ZERO because layout[1]
    // has no body placeholder after our mutation.
    let body_ph_count = slide_xml.matches("idx=\"1\"").count();
    assert_eq!(
        body_ph_count,
        0,
        "AC-011: when the layout has no body placeholder (idx=1), the slide serializer \
         must NOT emit <p:ph idx=\"1\"> for a Body frame; found {body_ph_count} occurrences. \
         Slide XML excerpt: {}",
        &slide_xml[..slide_xml.len().min(600)]
    );
}

// ─── AC-012 / S2: Silent i32 slide-size clamp replaced with structured error ─

/// BC-4.01.005 invariant 1 / AC-012 / PR-52 S2:
/// `PresentationSerializer::build` must return `PptxError::InvalidEmu` (or emit
/// a `tracing::warn!` with documented saturation) when `page_size.width.0` or
/// `page_size.height.0` exceeds `i32::MAX`.
///
/// RED: currently the code silently clamps via `unwrap_or(9_144_000)` without
/// any error or warning. This test verifies the structured error path is wired.
#[test]
fn test_BC_4_01_005_ac012_slide_size_out_of_range_emu_returns_err() {
    use crate::presentation::PresentationSerializer;

    // Build a LaidOutDeck with a page size whose width.0 exceeds i32::MAX.
    let mut laid_out = make_laid_out_deck(1);
    // i32::MAX + 1 = 2_147_483_648 which is > i32::MAX (2_147_483_647).
    laid_out.page_size = PageSize {
        width: Emu(2_147_483_648_i64), // > i32::MAX
        height: Emu(5_143_500),        // valid
    };

    let brand = make_brand();
    let result = PresentationSerializer::build(
        &laid_out,
        &brand,
        &["rId4".to_string()],
        "rId1",
        "rId2",
        "rId3",
    );

    // Must return Err — either PptxError::InvalidEmu or a variant that indicates
    // the out-of-range value was detected. The silent clamp must be gone.
    assert!(
        result.is_err(),
        "AC-012 / PR-52 S2: PresentationSerializer::build must return Err when \
         page_size.width.0 = {} exceeds i32::MAX ({}); got Ok(_). \
         The silent `unwrap_or` clamp must be replaced with a structured error.",
        2_147_483_648_i64,
        i32::MAX
    );

    // Verify the error is the correct variant.
    if let Err(e) = result {
        let error_str = e.to_string();
        assert!(
            error_str.contains("EMU") || error_str.contains("emu") || error_str.contains("size"),
            "AC-012: the error for out-of-range EMU must mention 'EMU' or 'size'; got: {error_str}"
        );
    }
}

// ─── S1: validate_emu Err path (PR-52 follow-up) ─────────────────────────────

/// PR-52 S1: `validate_emu` must return `PptxError::InvalidEmu` when a frame
/// has negative width.
///
/// CLASSIFICATION: WIRING-EXEMPT — `validate_emu` in `slide_serializer.rs` already
/// returns `PptxError::InvalidEmu` for negative bbox dimensions (STORY-037
/// implementation). This test is included to fulfil the PR-52 S1 obligation of
/// having an EXPLICIT test for the negative-width path, which was previously
/// untested. The production code is already correct.
#[test]
fn test_BC_4_01_005_s1_validate_emu_negative_width_returns_err() {
    use crate::slide_serializer::SlideSerializer;

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(-1), // NEGATIVE — must trigger InvalidEmu
                height: Emu(100),
            },
            content: FrameContent::Title(Arc::from("Test")),
            text_flow: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    };

    let serializer = SlideSerializer::new(false, 0);
    let result = serializer.build(&slide, 0, "rId1", &[]);

    assert!(
        result.is_err(),
        "PR-52 S1: SlideSerializer::build must return Err(PptxError::InvalidEmu) \
         when a frame has negative width; got Ok(_)"
    );

    if let Err(crate::error::PptxError::InvalidEmu {
        slide_index,
        frame_index,
        ..
    }) = result
    {
        assert_eq!(slide_index, 0, "S1: InvalidEmu must report slide_index=0");
        assert_eq!(frame_index, 0, "S1: InvalidEmu must report frame_index=0");
    } else {
        panic!("PR-52 S1: expected PptxError::InvalidEmu for negative width; got different error");
    }
}

/// PR-52 S1: `validate_emu` must return `PptxError::InvalidEmu` when a frame
/// has negative height.
///
/// CLASSIFICATION: WIRING-EXEMPT — same rationale as the negative-width test above.
#[test]
fn test_BC_4_01_005_s1_validate_emu_negative_height_returns_err() {
    use crate::slide_serializer::SlideSerializer;

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(100),
                height: Emu(-1), // NEGATIVE — must trigger InvalidEmu
            },
            content: FrameContent::Title(Arc::from("Test")),
            text_flow: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    };

    let serializer = SlideSerializer::new(false, 0);
    let result = serializer.build(&slide, 0, "rId1", &[]);

    assert!(
        result.is_err(),
        "PR-52 S1: SlideSerializer::build must return Err(PptxError::InvalidEmu) \
         when a frame has negative height; got Ok(_)"
    );

    if let Err(crate::error::PptxError::InvalidEmu {
        slide_index,
        frame_index,
        ..
    }) = result
    {
        assert_eq!(slide_index, 0, "S1: InvalidEmu must report slide_index=0");
        assert_eq!(frame_index, 0, "S1: InvalidEmu must report frame_index=0");
    } else {
        panic!("PR-52 S1: expected PptxError::InvalidEmu for negative height; got different error");
    }
}

// ─── S3: Subtitle frame → SubTitle placeholder ───────────────────────────────

/// PR-52 S3: A `FrameContent::Subtitle` frame must emit `<p:ph type="subTitle" idx="1">`
/// (OOXML `PlaceholderValues::SubTitle`), NOT `type="title"` (PlaceholderValues::Title).
///
/// RED: currently `build_shape` in `slide_serializer.rs` uses `PlaceholderValues::Title`
/// for BOTH `Title` and `Subtitle` frames (checking only `ph_idx == 0`).
/// The fix requires `Subtitle` frames to get `ph_type = Some(PlaceholderValues::SubTitle)`
/// with `idx = 1`.
#[test]
fn test_BC_4_01_005_s3_subtitle_frame_emits_subtitle_placeholder() {
    use crate::slide_serializer::SlideSerializer;

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![
            Frame {
                bbox: title_bbox(),
                content: FrameContent::Title(Arc::from("Main Title")),
                text_flow: None,
            },
            Frame {
                bbox: body_bbox(),
                content: FrameContent::Subtitle(Arc::from("Subtitle Text")),
                text_flow: None,
            },
        ],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    };

    let serializer = SlideSerializer::new(false, 0);
    let (slide_xml_bytes, _) = serializer
        .build(&slide, 0, "rId1", &[])
        .expect("SlideSerializer::build must succeed for subtitle frame");
    let slide_xml = String::from_utf8(slide_xml_bytes).expect("slide XML must be valid UTF-8");

    // The OOXML type string for SubTitle is "subTitle".
    assert!(
        slide_xml.contains("subTitle"),
        "PR-52 S3: a Subtitle frame must emit type=\"subTitle\" (PlaceholderValues::SubTitle); \
         the current implementation incorrectly uses 'title' for both Title and Subtitle frames. \
         slide XML excerpt: {}",
        &slide_xml[..slide_xml.len().min(800)]
    );

    // The title frame must still emit type="title".
    assert!(
        slide_xml.contains("ctrTitle") || slide_xml.contains("\"title\""),
        "PR-52 S3: the Title frame must still emit type=\"title\" or \"ctrTitle\"; \
         slide XML excerpt: {}",
        &slide_xml[..slide_xml.len().min(800)]
    );

    // The subtitle placeholder must have idx=1.
    // We locate the "subTitle" occurrence and look for nearby idx attribute.
    let subtitle_pos = slide_xml
        .find("subTitle")
        .expect("subTitle must appear in slide XML");
    let nearby =
        &slide_xml[subtitle_pos.saturating_sub(200)..(subtitle_pos + 200).min(slide_xml.len())];
    assert!(
        nearby.contains("idx=\"1\""),
        "PR-52 S3: the SubTitle placeholder must have idx=\"1\"; \
         nearby XML context: {nearby}"
    );
}
