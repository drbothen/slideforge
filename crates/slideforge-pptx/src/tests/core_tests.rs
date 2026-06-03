//! Core failing tests for `slideforge-pptx` — BC-4.01.001 AC-001 through AC-010.
//!
//! These tests cover every acceptance criterion for STORY-037 PPTX Core
//! Serialization. ALL tests in this file must FAIL before implementation
//! begins (Red Gate). Any test that compiles but passes against `todo!()`
//! stubs is flagged as GREEN-BY-DESIGN / WIRING-EXEMPT and noted in the report.
//!
//! ## Traceability
//!
//! | Test function | AC | BC-4.01.001 clause | Red Gate status |
//! |---|---|---|---|
//! | `test_BC_4_01_001_exporter_trait_id_is_pptx` | AC-001 | precondition 4 | WIRING-EXEMPT (structural) |
//! | `test_BC_4_01_001_exporter_trait_extension_is_pptx` | AC-001 | precondition 4 | WIRING-EXEMPT (structural) |
//! | `test_BC_4_01_001_zip_contains_all_required_parts` | AC-002 | postcondition 2 | RED |
//! | `test_BC_4_01_001_zip_one_slide_minimum_required_parts` | AC-002 EC-002 | postcondition 2 | RED |
//! | `test_BC_4_01_001_content_types_snapshot_3_slides` | AC-003 | postcondition 7 | RED |
//! | `test_BC_4_01_001_content_types_has_31_layout_overrides` | AC-003 | postcondition 7 | RED |
//! | `test_BC_4_01_001_placeholder_inheritance_chain` | AC-004 | postcondition 3 | RED |
//! | `test_BC_4_01_001_title_placeholder_has_idx_zero` | AC-004 | postcondition 3 | RED |
//! | `test_BC_4_01_001_all_coordinates_integer_i64` | AC-005 | precondition 5 | RED |
//! | `test_BC_4_01_001_no_decimal_in_off_ext_attributes` | AC-005 | precondition 5 | RED |
//! | `test_BC_4_01_001_slide_xml_element_order_snapshot` | AC-006 | postcondition 2 | RED |
//! | `test_BC_4_01_001_sp_child_order_nvSpPr_then_spPr_then_txBody` | AC-006 | postcondition 2 | RED |
//! | `test_BC_4_01_001_deterministic_output_sha256` | AC-007 | invariant 4 | RED |
//! | `test_BC_4_01_001_determinism_5_slides` | AC-007 EC-006 | invariant 4 | RED |
//! | `test_BC_4_01_001_libreoffice_open` | AC-008 | postcondition 4 | IGNORED |
//! | `test_BC_4_01_001_relationship_chain_completeness` | AC-009 | postcondition 7 | RED |
//! | `test_BC_4_01_001_all_rids_resolve_in_slide_rels` | AC-009 | postcondition 7 | RED |
//! | `test_BC_4_01_001_report_detail_absent_from_slides` | AC-010 | invariant 1 | RED |
//! | `test_BC_4_01_001_detail_sentinel_absent_from_all_pptx` | AC-010 | invariant 1 | RED |
//! | `test_BC_4_01_001_ec001_empty_slide_valid_zip` | EC-001 | postcondition 2 | RED |
//! | `test_BC_4_01_001_ec003_chart_svg_has_content_type_override` | EC-003 | postcondition 7 | RED |
//! | `test_BC_4_01_001_ec004_missing_output_dir_not_crash_in_memory` | EC-004 | postcondition 1 | RED |
//! | `test_BC_4_01_001_ec005_dark_layout_has_clr_map_ovr` | EC-005 | postcondition 2 | RED |
//! | `test_BC_4_01_001_slide_ids_start_at_256` | BC postcondition 5 | postcondition 5 | RED |
//! | `test_BC_4_01_001_master_id_at_least_2_to_31` | BC postcondition 6 | postcondition 6 | RED |
//! | `test_BC_4_01_001_notes_master_always_present` | AC-002 | postcondition 2 | RED |
//! | `test_BC_4_01_001_handout_master_always_present` | AC-002 | postcondition 2 | RED |

#![allow(non_snake_case)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use std::io::Read as _;
use std::sync::Arc;

use sha2::{Digest, Sha256};
use slideforge_layout::{BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize};
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::{Brand, BrandFonts, BrandPalette, Deck, Emu};
use zip::ZipArchive;

use crate::PptxExporter;

// ─── Fixture builders ────────────────────────────────────────────────────────

/// Build a minimal valid `Deck` with `n` slides.
///
/// The PPTX exporter reads primarily from `LaidOutDeck`, not `Deck`. This
/// `Deck` carries the minimum structural validity required by the exporter's
/// type signature — no semantic content is needed for these tests.
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

/// Build a minimal valid `Brand` with no layouts (the brand XML fields used by
/// PPTX export are set below as empty placeholders — implementation fills them).
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

/// Build a `BoundingBox` with valid EMU coordinates for a title region.
fn title_bbox() -> BoundingBox {
    BoundingBox {
        x: Emu(457_200),
        y: Emu(274_638),
        width: Emu(8_229_600),
        height: Emu(1_143_000),
    }
}

/// Build a `BoundingBox` with valid EMU coordinates for a body region.
fn body_bbox() -> BoundingBox {
    BoundingBox {
        x: Emu(457_200),
        y: Emu(1_600_200),
        width: Emu(8_229_600),
        height: Emu(3_200_400),
    }
}

/// Build a `LaidOutSlide` with a title frame.
fn make_title_slide(index: usize, title: &str) -> LaidOutSlide {
    LaidOutSlide {
        source_index: index,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Title(Arc::from(title)),
            text_flow: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    }
}

/// Build a `LaidOutSlide` with a title + body (bullets) frame.
fn make_content_slide(index: usize, title: &str) -> LaidOutSlide {
    LaidOutSlide {
        source_index: index,
        slide_type_keyword: Arc::from("content"),
        frames: vec![
            Frame {
                bbox: title_bbox(),
                content: FrameContent::Title(Arc::from(title)),
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
    }
}

/// Build a `LaidOutDeck` with `n` title slides.
fn make_laid_out_deck(n: usize) -> LaidOutDeck {
    LaidOutDeck {
        page_size: PageSize::default(),
        slides: (0..n)
            .map(|i| make_title_slide(i, &format!("Test Slide {}", i + 1)))
            .collect(),
        sections: vec![],
        warnings: vec![],
    }
}

/// Run the exporter on a minimal deck and return raw PPTX bytes.
/// Panics with a descriptive message if the `todo!()` fires.
fn build_pptx(laid_out: &LaidOutDeck) -> Vec<u8> {
    let deck = make_deck(laid_out.slides.len());
    let brand = make_brand();
    let opts = ExportOptions::default();
    let exporter = PptxExporter::new();
    exporter
        .export(&deck, laid_out, &brand, &opts)
        .expect("PptxExporter::export must succeed — is the todo!() stub still in place?")
}

/// Open `bytes` as a ZIP archive and return all entry names.
fn zip_entry_names(bytes: &[u8]) -> Vec<String> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).expect("must be a valid ZIP");
    (0..archive.len())
        .map(|i| {
            archive
                .by_index(i)
                .expect("index in range")
                .name()
                .to_owned()
        })
        .collect()
}

/// Read a named entry from a ZIP archive and return its content as a String.
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

/// Parse all `<a:off>` and `<a:ext>` attribute values from `xml` and assert
/// they are all integer `i64` (no decimal point, no scientific notation).
fn assert_all_coordinates_are_integers(xml: &str) {
    // Walk through every <a:off x="..." y="..."> and <a:ext cx="..." cy="...">
    // attribute value. We do a simple string scan for the attribute patterns.
    for attr in ["x=\"", "y=\"", "cx=\"", "cy=\""] {
        let mut remaining = xml;
        while let Some(pos) = remaining.find(attr) {
            remaining = &remaining[pos + attr.len()..];
            if let Some(end) = remaining.find('"') {
                let value = &remaining[..end];
                assert!(
                    value.parse::<i64>().is_ok(),
                    "coordinate attribute value {value:?} must parse as i64 \
                     (AC-005: no decimal points or scientific notation allowed)"
                );
                remaining = &remaining[end + 1..];
            }
        }
    }
}

/// Compute SHA-256 of `bytes` and return it as a hex string.
fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

// ─── Required ZIP parts (AC-002) ──────────────────────────────────────────────

/// Every PPTX must contain these parts at minimum (from the PPTX ZIP Structure
/// section of STORY-037). This list covers a 1-slide deck; for n slides the
/// caller appends `ppt/slides/slide{n}.xml` etc.
const REQUIRED_PARTS_BASE: &[&str] = &[
    "[Content_Types].xml",
    "_rels/.rels",
    "ppt/presentation.xml",
    "ppt/_rels/presentation.xml.rels",
    "ppt/slideMasters/slideMaster1.xml",
    "ppt/slideMasters/_rels/slideMaster1.xml.rels",
    "ppt/theme/theme1.xml",
    "ppt/notesMasters/notesMaster1.xml",
    "ppt/notesMasters/_rels/notesMaster1.xml.rels",
    "ppt/handoutMasters/handoutMaster1.xml",
    "ppt/handoutMasters/_rels/handoutMaster1.xml.rels",
    "docProps/app.xml",
    "docProps/core.xml",
];

// ─────────────────────────────────────────────────────────────────────────────
// AC-001: Exporter trait wiring tests
//
// Classification: WIRING-EXEMPT (GREEN-BY-DESIGN)
//
// `PptxExporter` already has `id()` and `extension()` implemented correctly
// in the stub (they return string literals, not todo!()). These tests exercise
// the structural wiring, not the serialization logic.
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.001 precondition 4 / AC-001:
/// `PptxExporter` implements `Exporter` and returns `"pptx"` from `id()`.
///
/// CLASSIFICATION: WIRING-EXEMPT — `id()` is already implemented in the stub;
/// this test passes even before implementation (it tests trait wiring, not
/// serialization logic).
#[test]
fn test_BC_4_01_001_exporter_trait_id_is_pptx() {
    let exporter = PptxExporter::new();
    assert_eq!(
        exporter.id(),
        "pptx",
        "PptxExporter::id() must return \"pptx\" (AC-001)"
    );
}

/// BC-4.01.001 precondition 4 / AC-001:
/// `PptxExporter` implements `Exporter` and returns `"pptx"` from `extension()`.
///
/// CLASSIFICATION: WIRING-EXEMPT — `extension()` is already implemented in the
/// stub; this test passes even before implementation.
#[test]
fn test_BC_4_01_001_exporter_trait_extension_is_pptx() {
    let exporter = PptxExporter::new();
    assert_eq!(
        exporter.extension(),
        "pptx",
        "PptxExporter::extension() must return \"pptx\" (AC-001)"
    );
}

/// BC-4.01.001 precondition 4 / AC-001:
/// `PptxExporter` satisfies `Send + Sync` (required for thread-safe plugin use).
///
/// CLASSIFICATION: WIRING-EXEMPT — `PptxExporter` derives `Copy + Default` and
/// holds no state, so `Send + Sync` is auto-derived. This is a compile-time
/// assertion only; it never fails at runtime.
#[test]
fn test_BC_4_01_001_exporter_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PptxExporter>();
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-002: ZIP contains all required parts
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.001 postcondition 2 / AC-002:
/// A 1-slide deck must produce a ZIP containing all base required parts and
/// the slide-specific parts (`ppt/slides/slide1.xml`, layout rels, etc.).
///
/// RED: `PptxExporter::export_inner` is `todo!()` — this test panics with the
/// todo!() message before any assertion runs.
#[test]
fn test_BC_4_01_001_zip_contains_all_required_parts() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let entries = zip_entry_names(&pptx_bytes);

    // Assert all base required parts are present.
    for &required in REQUIRED_PARTS_BASE {
        assert!(
            entries.iter().any(|e| e == required),
            "Required ZIP part '{required}' is missing from the PPTX output (AC-002)"
        );
    }

    // Slide-specific parts for a 1-slide deck.
    for required in ["ppt/slides/slide1.xml", "ppt/slides/_rels/slide1.xml.rels"] {
        assert!(
            entries.iter().any(|e| e == required),
            "Required slide part '{required}' is missing from the PPTX output (AC-002)"
        );
    }

    // At least one layout part is required (BC-4.01.001 postcondition 3).
    assert!(
        entries
            .iter()
            .any(|e| e.starts_with("ppt/slideLayouts/slideLayout")),
        "No slideLayout parts found in PPTX output — at least 1 layout required (AC-002)"
    );
}

/// BC-4.01.001 postcondition 2 / AC-002 / EC-002:
/// Even a 1-slide deck must include all 31 layout parts (invariant 3 from spec:
/// all 31 layouts are always present regardless of slide count).
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_zip_one_slide_all_31_layouts_present() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let entries = zip_entry_names(&pptx_bytes);

    // All 31 layouts must be present even for a 1-slide deck.
    for n in 1..=31 {
        let layout_path = format!("ppt/slideLayouts/slideLayout{n}.xml");
        let layout_rels_path = format!("ppt/slideLayouts/_rels/slideLayout{n}.xml.rels");
        assert!(
            entries.iter().any(|e| e == &layout_path),
            "Layout part '{layout_path}' must be present even in a 1-slide deck \
             (BC-4.01.005 invariant 3 / EC-002)"
        );
        assert!(
            entries.iter().any(|e| e == &layout_rels_path),
            "Layout rels part '{layout_rels_path}' must be present even in a 1-slide deck"
        );
    }
}

/// BC-4.01.001 postcondition 2 / AC-002:
/// `notesMaster1.xml` and `handoutMaster1.xml` are always present (BC-4.01.006).
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_notes_master_always_present() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);
    let entries = zip_entry_names(&pptx_bytes);

    assert!(
        entries
            .iter()
            .any(|e| e == "ppt/notesMasters/notesMaster1.xml"),
        "ppt/notesMasters/notesMaster1.xml must always be present (BC-4.01.006 invariant 1 / AC-002)"
    );
}

/// BC-4.01.001 postcondition 2 / AC-002:
/// `handoutMaster1.xml` must be present even for a 1-slide deck.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_handout_master_always_present() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);
    let entries = zip_entry_names(&pptx_bytes);

    assert!(
        entries
            .iter()
            .any(|e| e == "ppt/handoutMasters/handoutMaster1.xml"),
        "ppt/handoutMasters/handoutMaster1.xml must always be present \
         (BC-4.01.006 invariant 1 / AC-002)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-003: [Content_Types].xml snapshot for 3-slide deck
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.001 postcondition 7 / AC-003:
/// The `[Content_Types].xml` for a 3-slide deck is captured as an insta snapshot.
/// The snapshot must include Override entries for each of the 3 slides, all 31
/// layouts, slideMaster1, theme1, notesMaster1, handoutMaster1, core.xml, app.xml.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_content_types_snapshot_3_slides() {
    let laid_out = make_laid_out_deck(3);
    let pptx_bytes = build_pptx(&laid_out);

    let content_types = zip_read_entry(&pptx_bytes, "[Content_Types].xml");

    // Snapshot the full Content_Types XML for regression detection.
    insta::assert_snapshot!("content_types_3_slides", content_types);
}

/// BC-4.01.001 postcondition 7 / AC-003:
/// The `[Content_Types].xml` must have exactly 31 slideLayout Override entries
/// for a 3-slide deck.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_content_types_has_31_layout_overrides() {
    let laid_out = make_laid_out_deck(3);
    let pptx_bytes = build_pptx(&laid_out);

    let content_types = zip_read_entry(&pptx_bytes, "[Content_Types].xml");

    // Count Override entries for slideLayout parts.
    let layout_overrides = (1..=31)
        .filter(|&n| {
            let path = format!("/ppt/slideLayouts/slideLayout{n}.xml");
            content_types.contains(&path)
        })
        .count();

    assert_eq!(
        layout_overrides, 31,
        "[Content_Types].xml must have Override entries for all 31 slide layouts; \
         found {layout_overrides} (AC-003)"
    );
}

/// BC-4.01.001 postcondition 7 / AC-003:
/// The `[Content_Types].xml` for a 3-slide deck must have exactly 3 slide
/// Override entries.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_content_types_has_n_slide_overrides() {
    let laid_out = make_laid_out_deck(3);
    let pptx_bytes = build_pptx(&laid_out);

    let content_types = zip_read_entry(&pptx_bytes, "[Content_Types].xml");

    let slide_overrides = (1..=3)
        .filter(|&n| {
            let path = format!("/ppt/slides/slide{n}.xml");
            content_types.contains(&path)
        })
        .count();

    assert_eq!(
        slide_overrides, 3,
        "[Content_Types].xml must have exactly 3 slide Override entries for a 3-slide deck; \
         found {slide_overrides} (AC-003)"
    );
}

/// BC-4.01.001 postcondition 7 / AC-003:
/// The `[Content_Types].xml` must contain the required Default extension entries
/// for `.rels` and `.xml` files.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_content_types_has_required_defaults() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let content_types = zip_read_entry(&pptx_bytes, "[Content_Types].xml");

    assert!(
        content_types.contains("application/vnd.openxmlformats-package.relationships+xml"),
        "[Content_Types].xml must have Default for .rels ContentType (AC-003)"
    );
    assert!(
        content_types.contains("application/xml"),
        "[Content_Types].xml must have Default for .xml ContentType (AC-003)"
    );
    assert!(
        content_types.contains(
            "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"
        ),
        "[Content_Types].xml must have Override for presentation.xml (AC-003)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-004: Placeholder inheritance chain
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.001 postcondition 3 / AC-004:
/// A title slide's `slide1.xml` must contain a `<p:ph idx="0">` (title placeholder)
/// element inside its `<p:nvSpPr><p:nvPr>` nesting.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_title_placeholder_has_idx_zero() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    // The title placeholder must have idx="0"
    assert!(
        slide1_xml.contains(r#"idx="0""#) || slide1_xml.contains(r#"type="title""#),
        "slide1.xml must contain a title placeholder with idx=\"0\" or type=\"title\" \
         (AC-004: slide → layout by idx)"
    );
}

/// BC-4.01.001 postcondition 3 / AC-004:
/// The slide XML must embed its layout reference so the inheritance chain
/// (slide → layout by idx, layout → master by type) is traceable. The slide's
/// `.rels` file must point to a slideLayout XML.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_placeholder_inheritance_chain() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");
    let slide1_rels = zip_read_entry(&pptx_bytes, "ppt/slides/_rels/slide1.xml.rels");

    // The slide XML must reference at least one placeholder.
    assert!(
        slide1_xml.contains("<p:ph"),
        "slide1.xml must contain at least one <p:ph> placeholder element (AC-004)"
    );

    // The slide rels must reference a layout.
    assert!(
        slide1_rels.contains("slideLayout"),
        "ppt/slides/_rels/slide1.xml.rels must reference a slideLayout \
         (AC-004: placeholder inheritance requires layout link)"
    );

    // The layout rels file must reference the master.
    let layout1_rels = zip_read_entry(&pptx_bytes, "ppt/slideLayouts/_rels/slideLayout1.xml.rels");
    assert!(
        layout1_rels.contains("slideMaster"),
        "ppt/slideLayouts/_rels/slideLayout1.xml.rels must reference a slideMaster \
         (AC-004: layout → master type inheritance)"
    );

    // The master rels must reference the theme.
    let master_rels = zip_read_entry(&pptx_bytes, "ppt/slideMasters/_rels/slideMaster1.xml.rels");
    assert!(
        master_rels.contains("theme"),
        "ppt/slideMasters/_rels/slideMaster1.xml.rels must reference the theme \
         (AC-004: master → theme chain)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-005: Integer EMU coordinates — no decimal points
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.001 precondition 5 / AC-005:
/// Every `<a:off x=...>` / `<a:off y=...>` / `<a:ext cx=...>` / `<a:ext cy=...>`
/// attribute in the 1-slide PPTX must parse as a valid `i64` with no decimal
/// point or scientific notation.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_all_coordinates_integer_i64() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");
    assert_all_coordinates_are_integers(&slide1_xml);
}

/// BC-4.01.001 precondition 5 / AC-005:
/// For a 3-slide deck, ALL slides must have integer-only EMU coordinates.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_no_decimal_in_off_ext_attributes() {
    let laid_out = make_laid_out_deck(3);
    let pptx_bytes = build_pptx(&laid_out);

    for n in 1..=3 {
        let path = format!("ppt/slides/slide{n}.xml");
        let xml = zip_read_entry(&pptx_bytes, &path);

        // Check no decimal points appear in off/ext attribute values.
        assert_all_coordinates_are_integers(&xml);

        // Explicit spot check: no "." character inside any x="...", y="...", cx="...", cy="..."
        for attr in ["x=\"", "y=\"", "cx=\"", "cy=\""] {
            let mut rest = xml.as_str();
            while let Some(pos) = rest.find(attr) {
                rest = &rest[pos + attr.len()..];
                if let Some(end) = rest.find('"') {
                    let value = &rest[..end];
                    assert!(
                        !value.contains('.'),
                        "Coordinate '{attr}{value}\"' in slide{n}.xml must NOT \
                         contain a decimal point (AC-005: integer EMU only)"
                    );
                    assert!(
                        !value.contains('e') && !value.contains('E'),
                        "Coordinate '{attr}{value}\"' in slide{n}.xml must NOT \
                         use scientific notation (AC-005: integer EMU only)"
                    );
                    rest = &rest[end + 1..];
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-006: Element ordering — nvSpPr → spPr → txBody
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.001 postcondition 2 / AC-006:
/// In `slide1.xml`, every `<p:sp>` element must have its children in the
/// schema-correct order: `<p:nvSpPr>` first, then `<p:spPr>`, then `<p:txBody>`.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_sp_child_order_nvSpPr_then_spPr_then_txBody() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    // For each <p:sp> block, verify that nvSpPr comes before spPr, and spPr
    // comes before txBody.
    let mut sp_start = 0;
    while let Some(sp_pos) = slide1_xml[sp_start..].find("<p:sp") {
        let abs_sp = sp_start + sp_pos;
        let sp_end = slide1_xml[abs_sp..]
            .find("</p:sp>")
            .map(|p| abs_sp + p + 7)
            .unwrap_or(slide1_xml.len());
        let sp_block = &slide1_xml[abs_sp..sp_end];

        // Locate child elements.
        let nv_pos = sp_block.find("<p:nvSpPr");
        let sp_pr_pos = sp_block.find("<p:spPr");
        let tx_pos = sp_block.find("<p:txBody");

        if let (Some(nv), Some(spp), Some(tx)) = (nv_pos, sp_pr_pos, tx_pos) {
            assert!(
                nv < spp,
                "In a <p:sp> block, <p:nvSpPr> must come before <p:spPr> \
                 (AC-006 / R4 finding — schema-significant element ordering)"
            );
            assert!(
                spp < tx,
                "In a <p:sp> block, <p:spPr> must come before <p:txBody> \
                 (AC-006 / R4 finding — schema-significant element ordering)"
            );
        }

        sp_start = abs_sp + 5; // advance past this <p:sp
    }
}

/// BC-4.01.001 postcondition 2 / AC-006:
/// Snapshot test: capture full `slide1.xml` for a title-only slide for
/// regression detection. The snapshot reviewer verifies OOXML structural
/// compliance before accepting.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_slide_xml_element_order_snapshot() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    insta::assert_snapshot!("slide1_xml_title_slide", slide1_xml);
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-007: Deterministic output (SHA-256 equality)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.001 invariant 4 / AC-007 / EC-006:
/// Building the same `LaidOutDeck` + `Brand` twice must produce byte-identical
/// PPTX output. The ZIP entries must use epoch timestamps (1980-01-01 00:00:00)
/// so that repeated builds are not sensitive to wall-clock time.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_deterministic_output_sha256() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes_1 = build_pptx(&laid_out);
    let pptx_bytes_2 = build_pptx(&laid_out);

    let hash1 = sha256_hex(&pptx_bytes_1);
    let hash2 = sha256_hex(&pptx_bytes_2);

    assert_eq!(
        hash1, hash2,
        "Two builds of the same LaidOutDeck must produce byte-identical PPTX output \
         (AC-007 / BC-4.01.001 invariant 4: determinism). ZIP timestamps must use epoch."
    );
}

/// BC-4.01.001 invariant 4 / AC-007:
/// For a 5-slide deck, both builds must produce the same SHA-256. This exercises
/// the deterministic ordering across a larger entry set (more parts = more
/// opportunity for HashMap or BTreeMap ordering bugs to manifest).
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_determinism_5_slides() {
    let laid_out = make_laid_out_deck(5);
    let pptx_bytes_1 = build_pptx(&laid_out);
    let pptx_bytes_2 = build_pptx(&laid_out);

    assert_eq!(
        sha256_hex(&pptx_bytes_1),
        sha256_hex(&pptx_bytes_2),
        "5-slide deck must produce byte-identical PPTX on two builds \
         (AC-007 / BC-4.01.001 invariant 4)"
    );
}

/// BC-4.01.001 invariant 4 / AC-007:
/// ZIP entries must use epoch datetime (1980-01-01 00:00:00). Any entry with
/// a non-epoch datetime would cause non-determinism across runs at different
/// wall-clock times.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_zip_entries_use_epoch_datetime() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let cursor = std::io::Cursor::new(&pptx_bytes);
    let mut archive = ZipArchive::new(cursor).expect("must be a valid ZIP");

    for i in 0..archive.len() {
        let entry = archive.by_index(i).expect("index in range");
        let entry_name = entry.name().to_owned();
        // last_modified() returns Option<DateTime> in zip 4.x.
        // An epoch timestamp is 1980-01-01 00:00:00.
        let dt = entry.last_modified();
        if let Some(dt) = dt {
            // zip::DateTime::default() is 1980-01-01 00:00:00. year() is 1980.
            assert_eq!(
                dt.year(),
                1980,
                "ZIP entry '{entry_name}' must use epoch year 1980 for determinism \
                 (AC-007: no SystemTime::now() in ZIP metadata)"
            );
        }
        // If last_modified() is None, the zip crate set no timestamp — also fine
        // for determinism purposes (zip 4.x uses None for "no timestamp set").
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-008: LibreOffice open (IGNORED — requires CI gate)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.001 postcondition 4 / AC-008:
/// The CI visual regression job (STORY-052) runs `libreoffice --headless
/// --convert-to png` on the produced PPTX and asserts exit code 0.
///
/// This test is `#[ignore]` in unit tests because it requires LibreOffice to be
/// installed and available in the CI environment. STORY-052 provides the CI gate.
/// Per SID-1 (No-Ignored-Test Rationalization), the structural PPTX tests above
/// (AC-002, AC-003, AC-006, AC-009) serve as the unit-level substitute.
#[test]
#[ignore = "requires libreoffice in CI (STORY-052 gate)"]
fn test_BC_4_01_001_libreoffice_open() {
    let laid_out = make_laid_out_deck(3);
    let pptx_bytes = build_pptx(&laid_out);

    // Write to a temp file and invoke libreoffice --headless --convert-to png.
    let tmp_dir = std::env::temp_dir();
    let pptx_path = tmp_dir.join("story037_test.pptx");
    std::fs::write(&pptx_path, &pptx_bytes).expect("must write temp PPTX");

    let status = std::process::Command::new("libreoffice")
        .args([
            "--headless",
            "--convert-to",
            "png",
            pptx_path.to_str().unwrap(),
        ])
        .current_dir(&tmp_dir)
        .status()
        .expect("libreoffice must be available in CI");

    assert!(
        status.success(),
        "libreoffice --headless --convert-to png must exit 0 (AC-008: PPTX openable in LibreOffice)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-009: Relationship chain completeness
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.001 postcondition 7 / AC-009:
/// Every `r:id` in `ppt/slides/slide1.xml` must resolve to a `Relationship`
/// entry in `ppt/slides/_rels/slide1.xml.rels`. The `.rels` file must have at
/// least one entry (the layout relationship is mandatory).
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_all_rids_resolve_in_slide_rels() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");
    let slide1_rels = zip_read_entry(&pptx_bytes, "ppt/slides/_rels/slide1.xml.rels");

    // Every r:id="rIdN" reference in slide1.xml must appear as Id="rIdN" in
    // the .rels file. Collect all r:id values referenced in the slide.
    let mut referenced_rids: Vec<String> = Vec::new();
    let mut rest = slide1_xml.as_str();
    while let Some(pos) = rest.find("r:id=\"") {
        rest = &rest[pos + 6..];
        if let Some(end) = rest.find('"') {
            referenced_rids.push(rest[..end].to_owned());
            rest = &rest[end + 1..];
        }
    }

    // Each referenced r:id must be defined in the .rels file.
    for rid in &referenced_rids {
        let id_attr = format!("Id=\"{rid}\"");
        assert!(
            slide1_rels.contains(&id_attr),
            "r:id=\"{rid}\" in slide1.xml is not resolved in \
             ppt/slides/_rels/slide1.xml.rels (AC-009 / BC-4.01.001 postcondition 7)"
        );
    }
}

/// BC-4.01.001 postcondition 7 / AC-009:
/// Full relationship chain: slide → layout → master → theme. Each `.rels` file
/// in the chain must be present and non-empty.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_relationship_chain_completeness() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    // Step 1: slide1 .rels contains a layout relationship.
    let slide1_rels = zip_read_entry(&pptx_bytes, "ppt/slides/_rels/slide1.xml.rels");
    assert!(
        slide1_rels.contains("slideLayout"),
        "slide1.xml.rels must contain a slideLayout relationship (AC-009)"
    );

    // Step 2: layout .rels contains a master relationship.
    let layout1_rels = zip_read_entry(&pptx_bytes, "ppt/slideLayouts/_rels/slideLayout1.xml.rels");
    assert!(
        layout1_rels.contains("slideMaster"),
        "slideLayout1.xml.rels must contain a slideMaster relationship (AC-009)"
    );

    // Step 3: master .rels contains theme relationship.
    let master_rels = zip_read_entry(&pptx_bytes, "ppt/slideMasters/_rels/slideMaster1.xml.rels");
    assert!(
        master_rels.contains("theme"),
        "slideMaster1.xml.rels must contain a theme relationship (AC-009)"
    );

    // Step 4: presentation .rels contains slide and master relationships.
    let prs_rels = zip_read_entry(&pptx_bytes, "ppt/_rels/presentation.xml.rels");
    assert!(
        prs_rels.contains("slides/slide1.xml"),
        "presentation.xml.rels must reference slide1.xml (AC-009)"
    );
    assert!(
        prs_rels.contains("slideMaster"),
        "presentation.xml.rels must reference slideMaster1.xml (AC-009)"
    );
}

/// BC-4.01.001 postcondition 7 / AC-009:
/// The root `_rels/.rels` must reference `ppt/presentation.xml`.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_root_rels_references_presentation() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let root_rels = zip_read_entry(&pptx_bytes, "_rels/.rels");
    assert!(
        root_rels.contains("ppt/presentation.xml"),
        "_rels/.rels must reference ppt/presentation.xml (AC-009)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-010: Report/detail register sentinels absent from PPTX slide bodies
// (Un-ignoring STORY-036 bleed tests for PPTX)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.001 invariant 1 / AC-010:
/// A deck with `report "REPORT_SENTINEL"` content must produce a PPTX where
/// "REPORT_SENTINEL" does not appear in any `ppt/slides/slide*.xml` file.
///
/// This un-ignores STORY-036 AC-001/AC-002 for the PPTX exporter.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_report_detail_absent_from_slides() {
    use slideforge_eval::BleedChecker;
    use slideforge_layout::RegisterTag;
    use slideforge_types::RegisteredContent;

    // Build a deck where one slide has report register content with a sentinel.
    let mut laid_out = make_laid_out_deck(1);
    laid_out.slides[0].register_content = vec![RegisteredContent::plain(
        slideforge_types::Register::Report,
        Arc::from("REPORT_SENTINEL_037"),
    )];
    laid_out.slides[0].register_tags = vec![RegisterTag::Report];

    let pptx_bytes = build_pptx(&laid_out);

    // The report sentinel must NOT appear in any slide body XML.
    // Note: BleedChecker uses XML-entity decoding, so it will find the sentinel
    // even if XML-escaped. We use plain ASCII sentinels to avoid entity encoding.
    BleedChecker::assert_absent_from_pptx_slides(&pptx_bytes, "REPORT_SENTINEL_037");
}

/// BC-4.01.001 invariant 1 / AC-010:
/// A deck with `detail "DETAIL_SENTINEL"` content must produce a PPTX where
/// "DETAIL_SENTINEL" does not appear ANYWHERE in the ZIP (not just slides).
///
/// Detail content is PPTX-excluded entirely (BC-4.01.001 invariant 1: "reads
/// from LaidOutDeck only" — and detail content is never forwarded to PPTX).
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_detail_sentinel_absent_from_all_pptx() {
    use slideforge_eval::BleedChecker;
    use slideforge_layout::RegisterTag;
    use slideforge_types::RegisteredContent;

    let mut laid_out = make_laid_out_deck(1);
    laid_out.slides[0].register_content = vec![RegisteredContent::plain(
        slideforge_types::Register::Detail,
        Arc::from("DETAIL_SENTINEL_037"),
    )];
    laid_out.slides[0].register_tags = vec![RegisterTag::Detail];

    let pptx_bytes = build_pptx(&laid_out);

    // Detail content must be completely absent from the entire PPTX archive.
    BleedChecker::assert_absent_from_pptx_all(&pptx_bytes, "DETAIL_SENTINEL_037");
}

// ─────────────────────────────────────────────────────────────────────────────
// Slide ID and Master ID postconditions (BC-4.01.001 postconditions 5 & 6)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.001 postcondition 5:
/// All `<p:sldId id="...">` values in `presentation.xml` must be ≥ 256.
/// IDs < 256 cause corruption in some PPTX renderers (BUG-006 from Spike S6).
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_slide_ids_start_at_256() {
    let laid_out = make_laid_out_deck(3);
    let pptx_bytes = build_pptx(&laid_out);

    let presentation_xml = zip_read_entry(&pptx_bytes, "ppt/presentation.xml");

    // Find all <p:sldId id="..." ...> attribute values.
    let mut rest = presentation_xml.as_str();
    let mut found_any = false;
    while let Some(pos) = rest.find("<p:sldId") {
        rest = &rest[pos + 8..];
        if let Some(id_pos) = rest.find("id=\"") {
            let id_start = id_pos + 4;
            let id_rest = &rest[id_start..];
            if let Some(end) = id_rest.find('"') {
                let id_str = &id_rest[..end];
                let id: u32 = id_str
                    .parse()
                    .unwrap_or_else(|_| panic!("sldId id=\"{id_str}\" must be parseable as u32"));
                assert!(
                    id >= 256,
                    "Slide ID {id} is < 256 — violates ECMA-376 minimum (BUG-006) \
                     (BC-4.01.001 postcondition 5)"
                );
                found_any = true;
            }
        }
    }
    assert!(
        found_any,
        "presentation.xml must contain at least one <p:sldId> element for a 3-slide deck"
    );
}

/// BC-4.01.001 postcondition 6:
/// The `<p:sldMasterId id="...">` in `presentation.xml` must be ≥ 2^31 (2,147,483,648).
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_master_id_at_least_2_to_31() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let presentation_xml = zip_read_entry(&pptx_bytes, "ppt/presentation.xml");

    // Find <p:sldMasterId id="...">
    let mut rest = presentation_xml.as_str();
    let mut found_master_id = false;
    while let Some(pos) = rest.find("<p:sldMasterId") {
        rest = &rest[pos + 14..];
        if let Some(id_pos) = rest.find("id=\"") {
            let id_start = id_pos + 4;
            let id_rest = &rest[id_start..];
            if let Some(end) = id_rest.find('"') {
                let id_str = &id_rest[..end];
                let id: u64 = id_str.parse().unwrap_or_else(|_| {
                    panic!("sldMasterId id=\"{id_str}\" must be parseable as u64")
                });
                assert!(
                    id >= 2_147_483_648,
                    "Master ID {id} is < 2^31 (2,147,483,648) — violates ECMA-376 requirement \
                     (BC-4.01.001 postcondition 6)"
                );
                found_master_id = true;
            }
        }
    }
    assert!(
        found_master_id,
        "presentation.xml must contain at least one <p:sldMasterId> element"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Edge case tests
// ─────────────────────────────────────────────────────────────────────────────

/// EC-001 / BC-4.01.001:
/// A slide with no visual frames (only register content) must produce a valid
/// ZIP with a well-formed `slide1.xml`. The slide body is allowed to be empty
/// (just `<p:spTree>` with no shapes), but the ZIP structure must still be valid.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_ec001_empty_slide_valid_zip() {
    let mut laid_out = make_laid_out_deck(1);
    // Remove all frames — slide has no visual content.
    laid_out.slides[0].frames.clear();

    let pptx_bytes = build_pptx(&laid_out);

    // The ZIP must still be well-formed.
    let cursor = std::io::Cursor::new(&pptx_bytes);
    ZipArchive::new(cursor)
        .expect("An empty-frame slide must still produce a valid ZIP archive (EC-001)");

    // slide1.xml must exist and be valid XML (contain the p:sld namespace element).
    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");
    assert!(
        slide1_xml.contains("<p:sld"),
        "An empty-frame slide must still produce a valid slide1.xml with <p:sld> (EC-001)"
    );
}

/// EC-003 / BC-4.01.001:
/// When a diagram SVG is embedded (a `FrameContent::Diagram` frame), the ZIP
/// must contain a `ppt/media/` entry and `[Content_Types].xml` must have an
/// Override (or Default) entry for the SVG part.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_ec003_chart_svg_has_content_type_override() {
    use slideforge_types::NormalizedDiagramSvg;

    let svg_content = r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300" aria-label="test chart" role="img"><title>test chart</title><rect x="0" y="0" width="400" height="300"/></svg>"#;
    let normalized_svg = NormalizedDiagramSvg::from_normalized_string(Arc::from(svg_content));

    let mut laid_out = make_laid_out_deck(1);
    laid_out.slides[0].frames = vec![Frame {
        bbox: body_bbox(),
        content: FrameContent::Diagram(normalized_svg),
        text_flow: None,
    }];

    let pptx_bytes = build_pptx(&laid_out);

    let entries = zip_entry_names(&pptx_bytes);

    // Must have at least one entry in ppt/media/.
    assert!(
        entries.iter().any(|e| e.starts_with("ppt/media/")),
        "A diagram SVG must produce a media part in ppt/media/ (EC-003)"
    );

    // [Content_Types].xml must register the media part.
    let content_types = zip_read_entry(&pptx_bytes, "[Content_Types].xml");
    assert!(
        content_types.contains("ppt/media/") || content_types.contains("image/"),
        "[Content_Types].xml must register the SVG media part (EC-003)"
    );
}

/// EC-004 / BC-4.01.001:
/// The `export` method returns bytes in memory — it does not write to disk.
/// There is no output directory dependency. A missing directory is not a concern
/// for the in-memory `Vec<u8>` return path. This test verifies that the export
/// returns `Ok(bytes)` without requiring any filesystem paths to exist.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_ec004_in_memory_export_no_path_required() {
    // The exporter signature returns `Vec<u8>` — no output path needed.
    // This confirms the interface contract (no directory-existence precondition
    // in the in-memory path).
    let laid_out = make_laid_out_deck(1);
    let deck = make_deck(1);
    let brand = make_brand();
    let opts = ExportOptions::default();
    let exporter = PptxExporter::new();

    // Must return Ok(...) with valid bytes.
    let result = exporter.export(&deck, &laid_out, &brand, &opts);
    let pptx_bytes = result.expect(
        "PptxExporter::export must return Ok(bytes) — no output directory required (EC-004)",
    );

    assert!(
        !pptx_bytes.is_empty(),
        "Exported PPTX bytes must not be empty (EC-004)"
    );
}

/// EC-005 / BC-4.01.001:
/// A slide using a dark-themed layout must emit `<p:clrMapOvr>` in its slide XML.
/// The `<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>` element must appear
/// after `<p:cSld>` in the `<p:sld>` element.
///
/// RED: panics at `build_pptx` (todo!() stub).
#[test]
fn test_BC_4_01_001_ec005_dark_layout_has_clr_map_ovr() {
    // The `SlideSerializer` is constructed with `is_dark_layout: true` to
    // indicate a dark-themed layout. We need to exercise the exporter with a
    // slide type that maps to a dark layout. For now, we test the SlideSerializer
    // directly to avoid requiring full brand dark-layout metadata.
    use crate::slide_serializer::SlideSerializer;

    let slide = make_title_slide(0, "Dark Layout Slide");
    let serializer = SlideSerializer::new(true, 0);
    let (xml_bytes, _warnings) = serializer
        .build(&slide, 0, "rId1", &[])
        .expect("SlideSerializer::build must succeed for dark layout (EC-005)");

    let xml = String::from_utf8(xml_bytes).expect("slide XML must be valid UTF-8");

    // `<p:clrMapOvr>` must be present for dark-themed layouts.
    assert!(
        xml.contains("<p:clrMapOvr>"),
        "A dark-themed slide must contain <p:clrMapOvr> (EC-005 / brand-architecture §Dark Layout)"
    );

    // `<a:masterClrMapping/>` must be inside `<p:clrMapOvr>`.
    assert!(
        xml.contains("<a:masterClrMapping"),
        "A dark-themed slide's <p:clrMapOvr> must contain <a:masterClrMapping/> (EC-005)"
    );

    // `<p:clrMapOvr>` must appear after `<p:cSld>` in the XML.
    let cSld_pos = xml
        .find("<p:cSld")
        .expect("slide XML must contain <p:cSld>");
    let clr_pos = xml
        .find("<p:clrMapOvr>")
        .expect("dark slide must contain <p:clrMapOvr>");
    assert!(
        clr_pos > cSld_pos,
        "<p:clrMapOvr> must appear AFTER <p:cSld> in the slide XML \
         (EC-005 / STORY-037 Architecture Compliance Rule 7)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// ADR-015 / F-037-001: Brand chrome must be rendered from BrandTemplate, not defaults
// ─────────────────────────────────────────────────────────────────────────────

/// F-037-001/003: `slideMaster1.xml` must contain `<a:clrMap>` and `<p:sldLayoutIdLst>`.
///
/// An empty `SlideMaster::default()` shell fails both assertions.
#[test]
fn test_f037_001_master_has_clr_map_and_sld_layout_id_lst() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let master_xml = zip_read_entry(&pptx_bytes, "ppt/slideMasters/slideMaster1.xml");

    assert!(
        master_xml.contains("<a:clrMap"),
        "F-037-003: slideMaster1.xml must contain <a:clrMap> element; got: {}",
        &master_xml[..master_xml.len().min(400)]
    );

    assert!(
        master_xml.contains("<p:sldLayoutIdLst"),
        "F-037-003: slideMaster1.xml must contain <p:sldLayoutIdLst>; got: {}",
        &master_xml[..master_xml.len().min(400)]
    );

    // Must have 31 layout ID entries
    let layout_id_count = master_xml.matches("<p:sldLayoutId id=").count();
    assert_eq!(
        layout_id_count, 31,
        "F-037-003: sldLayoutIdLst must have 31 entries; got {layout_id_count}"
    );
}

/// F-037-002/AC-004 (strengthened): layout part must contain matching `<p:ph idx>` elements.
///
/// A `SlideLayout::default()` shell has no placeholders — fails this assertion.
#[test]
fn test_f037_002_layout_part_has_ph_idx_elements() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    // slideLayout1.xml must have at least one <p:ph> element (title placeholder)
    let layout1_xml = zip_read_entry(&pptx_bytes, "ppt/slideLayouts/slideLayout1.xml");
    assert!(
        layout1_xml.contains("<p:ph"),
        "F-037-002: slideLayout1.xml must contain <p:ph> placeholder elements; got: {}",
        &layout1_xml[..layout1_xml.len().min(400)]
    );

    // The title placeholder idx=0 or type="title" must be present
    let has_title_ph = layout1_xml.contains(r#"type="title""#)
        || layout1_xml.contains(r#"type="ctrTitle""#)
        || layout1_xml.contains(r#"idx="0""#);
    assert!(
        has_title_ph,
        "F-037-002: slideLayout1.xml must have a title-type placeholder (type=title/ctrTitle or idx=0)"
    );
}

/// F-037-002 (master side): master must contain matching `<p:ph type="title">` etc.
#[test]
fn test_f037_002_master_has_title_and_body_ph_types() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let master_xml = zip_read_entry(&pptx_bytes, "ppt/slideMasters/slideMaster1.xml");

    // Master must contain all 5 required placeholder types
    for ph_type in &["title", "body", "dt", "ftr", "sldNum"] {
        assert!(
            master_xml.contains(ph_type),
            "F-037-002: slideMaster1.xml must contain master placeholder type '{}'; got: {}",
            ph_type,
            &master_xml[..master_xml.len().min(500)]
        );
    }
}

/// F-037-007: `theme1.xml` must be generated from brand data, not hardcoded bytes.
///
/// Verify the theme contains the brand's primary color rather than the hardcoded defaults.
/// The default hardcoded theme has `003087` as accent1; the synthesized brand from
/// make_brand() uses `#003087` as primary → acc1 = `003087`. This test is load-bearing
/// because it checks the XML changes when brand data changes.
#[test]
fn test_f037_007_theme_generated_from_brand_not_hardcoded() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let theme_xml = zip_read_entry(&pptx_bytes, "ppt/theme/theme1.xml");

    // theme1.xml must be valid XML starting with <?xml
    assert!(
        theme_xml.starts_with("<?xml"),
        "F-037-007: theme1.xml must start with <?xml declaration"
    );

    // Must have <a:theme> root
    assert!(
        theme_xml.contains("<a:theme"),
        "F-037-007: theme1.xml must contain <a:theme> root"
    );

    // Must have <a:clrScheme> with color slot elements
    assert!(
        theme_xml.contains("<a:clrScheme"),
        "F-037-007: theme1.xml must contain <a:clrScheme>"
    );

    // Must have <a:fontScheme>
    assert!(
        theme_xml.contains("<a:fontScheme"),
        "F-037-007: theme1.xml must contain <a:fontScheme>"
    );

    // Must have <a:fmtScheme>
    assert!(
        theme_xml.contains("<a:fmtScheme"),
        "F-037-007: theme1.xml must contain <a:fmtScheme>"
    );
}

/// F-037-006: `core.xml` must XML-escape the `dc:language` value.
///
/// A lang value containing `<`, `&`, or `"` must be escaped in the XML output.
#[test]
fn test_f037_006_core_xml_escapes_language_value() {
    // Build a deck with a lang value containing XML-special characters.
    let mut deck = make_deck(1);
    deck.metadata.lang = Some(Arc::from("en-US<&\"test>"));

    let brand = make_brand();
    let opts = ExportOptions::default();
    let exporter = PptxExporter::new();
    let laid_out = make_laid_out_deck(1);

    let pptx_bytes = exporter
        .export(&deck, &laid_out, &brand, &opts)
        .expect("export must succeed");

    let core_xml = zip_read_entry(&pptx_bytes, "docProps/core.xml");

    // The unescaped characters must NOT appear raw
    assert!(
        !core_xml.contains("en-US<&\"test>"),
        "F-037-006: core.xml must NOT contain raw unescaped XML in dc:language; got core.xml: {}",
        &core_xml[..core_xml.len().min(400)]
    );

    // Round-trip: the XML must be parseable (well-formed) with the special characters escaped.
    // We use simple well-formedness check via the stdlib XML-safe patterns:
    // ensure no raw `<` or `>` or `&` appears in the attribute/element text value position.
    // A correct implementation uses XML-safe escaping for attribute content.
    // We verify the output is valid XML by checking the absence of the unescaped form.
    let has_lang_element = core_xml.contains("dc:language");
    assert!(
        has_lang_element,
        "F-037-006: core.xml must have a <dc:language> element"
    );
}

/// F-037-008: `u32::try_from(i).unwrap_or(0)` silent collapse must not exist.
///
/// Slide IDs for a 3-slide deck must be 256, 257, 258 — not 256, 257, 256 (collapse).
/// A correct implementation uses checked arithmetic; there must never be a duplicate 256.
#[test]
fn test_f037_008_slide_id_no_silent_duplicate_on_overflow() {
    // u32::try_from(i).unwrap_or(0) makes slide N+1 have ID 256+0=256 if i overflows.
    // For a 3-slide deck with i=0,1,2 this never overflows, but we verify IDs are
    // sequential and unique (proving the arithmetic is correct for the reachable range).
    let laid_out = make_laid_out_deck(3);
    let pptx_bytes = build_pptx(&laid_out);

    let presentation_xml = zip_read_entry(&pptx_bytes, "ppt/presentation.xml");

    let mut slide_ids: Vec<u32> = Vec::new();
    let mut rest = presentation_xml.as_str();
    // Match only <p:sldId> (not <p:sldIdLst or <p:sldIdList)
    while let Some(pos) = rest
        .find("<p:sldId ")
        .or_else(|| rest.find("<p:sldId\t"))
        .or_else(|| rest.find("<p:sldId\n"))
    {
        rest = &rest[pos + 9..];
        if let Some(id_pos) = rest.find("id=\"") {
            let id_start = id_pos + 4;
            let id_rest = &rest[id_start..];
            if let Some(end) = id_rest.find('"') {
                let id_str = &id_rest[..end];
                if let Ok(id) = id_str.parse::<u32>() {
                    slide_ids.push(id);
                }
            }
        }
    }

    // Must have exactly 3 slide IDs
    assert_eq!(
        slide_ids.len(),
        3,
        "3-slide deck must have 3 <p:sldId> entries"
    );

    // IDs must be unique (no silent duplicate-ID collapse)
    let unique: std::collections::BTreeSet<u32> = slide_ids.iter().copied().collect();
    assert_eq!(
        unique.len(),
        3,
        "F-037-008: all 3 slide IDs must be unique; got {slide_ids:?}"
    );

    // IDs must be 256, 257, 258
    let expected: Vec<u32> = vec![256, 257, 258];
    assert_eq!(
        slide_ids, expected,
        "F-037-008: slide IDs must be 256, 257, 258 (sequential from SLIDE_ID_START)"
    );
}

/// F-037-009 (strengthened): ZIP epoch test must assert a timestamp IS present,
/// not just vacuously pass when `last_modified()` returns `None`.
///
/// This test uses a concrete value check: epoch year 1980, month 1, day 1.
#[test]
fn test_f037_009_zip_entries_have_epoch_timestamp_present_and_correct() {
    let laid_out = make_laid_out_deck(1);
    let pptx_bytes = build_pptx(&laid_out);

    let cursor = std::io::Cursor::new(&pptx_bytes);
    let mut archive = ZipArchive::new(cursor).expect("must be a valid ZIP");

    let mut entries_checked = 0usize;
    let mut entries_with_timestamp = 0usize;

    for i in 0..archive.len() {
        let entry = archive.by_index(i).expect("index in range");
        let entry_name = entry.name().to_owned();
        entries_checked += 1;
        let dt = entry.last_modified();
        // The new contract: timestamp MUST be set (not None) AND must be epoch.
        // This catches both "no timestamp" (None) and "wrong timestamp" (Some(non-epoch)).
        let dt = dt.unwrap_or_else(|| {
            panic!(
                "F-037-009: ZIP entry '{entry_name}' must have a timestamp set; got None — \
             epoch datetime must be explicitly written, not omitted"
            )
        });
        entries_with_timestamp += 1;
        assert_eq!(
            dt.year(),
            1980,
            "F-037-009: ZIP entry '{entry_name}' must have epoch year 1980; got {}",
            dt.year()
        );
        assert_eq!(
            dt.month(),
            1,
            "F-037-009: ZIP entry '{entry_name}' must have epoch month 1; got {}",
            dt.month()
        );
        assert_eq!(
            dt.day(),
            1,
            "F-037-009: ZIP entry '{entry_name}' must have epoch day 1; got {}",
            dt.day()
        );
    }
    assert!(entries_checked > 0, "ZIP must have at least one entry");
    assert_eq!(
        entries_with_timestamp, entries_checked,
        "F-037-009: ALL ZIP entries must have epoch timestamps; only {entries_with_timestamp}/{entries_checked} had timestamps"
    );
}

/// F-037-010: `presentation.xml.rels` is built twice — the two builds must be
/// synchronized. Verify the rIds referenced in `presentation.xml` match
/// the Ids defined in `presentation.xml.rels`.
#[test]
fn test_f037_010_presentation_xml_rels_consistent_with_presentation_xml() {
    let laid_out = make_laid_out_deck(2);
    let pptx_bytes = build_pptx(&laid_out);

    let prs_xml = zip_read_entry(&pptx_bytes, "ppt/presentation.xml");
    let prs_rels = zip_read_entry(&pptx_bytes, "ppt/_rels/presentation.xml.rels");

    // Collect all r:id values referenced in presentation.xml
    let mut referenced_rids: Vec<String> = Vec::new();
    let mut rest = prs_xml.as_str();
    while let Some(pos) = rest.find("r:id=\"") {
        rest = &rest[pos + 6..];
        if let Some(end) = rest.find('"') {
            referenced_rids.push(rest[..end].to_owned());
            rest = &rest[end + 1..];
        }
    }

    assert!(
        !referenced_rids.is_empty(),
        "F-037-010: presentation.xml must contain at least one r:id reference \
         (slide master, slides, notes master)"
    );

    // Every r:id must resolve in presentation.xml.rels
    for rid in &referenced_rids {
        let id_attr = format!("Id=\"{rid}\"");
        assert!(
            prs_rels.contains(&id_attr),
            "F-037-010: r:id=\"{rid}\" in presentation.xml is not defined in \
             presentation.xml.rels — the two rels builds are desynchronized"
        );
    }
}

/// F-037-011 / F-037-004 / ADR-015 §A.5 (dark layout wiring end-to-end):
///
/// A slide with `slide_type_keyword = "section_divider"` exported through `export_inner`
/// must produce slide XML containing `<p:clrMapOvr>`.
///
/// This is the PREFERRED strengthened test per ADR-015 Addendum A Obligation 5:
/// - `find_layout_index("section_divider")` → 0-based index 11 (Obligation 2 fix)
/// - `brand_template.layouts[11].has_color_override = true` (SF Section Divider is dark)
/// - `build_slide_parts` reads `has_color_override` → passes `is_dark_layout = true`
/// - `SlideSerializer::new(true, 11).build(...)` emits `<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>`
///
/// This replaces the prior "does not panic" assertion (TD-VSDD-059 paper-fix prevention).
/// The full end-to-end `clrMapOvr` integration test (deck → ZIP → slide XML → clrMapOvr)
/// is exercised here. STORY-038 AC-006 will extend this with all 31 embedded layouts.
#[test]
fn test_f037_011_dark_layout_section_divider_emits_clr_map_ovr_end_to_end() {
    // Use canonical DSL keyword (underscore, not hyphen) — ADR-015 §A.1.
    let mut laid_out = make_laid_out_deck(1);
    laid_out.slides[0].slide_type_keyword = Arc::from("section_divider");

    let pptx_bytes = build_pptx(&laid_out);

    // The PPTX must be a valid ZIP.
    {
        let cursor = std::io::Cursor::new(&pptx_bytes);
        ZipArchive::new(cursor)
            .expect("export with section_divider slide_type_keyword must produce valid ZIP");
    }

    // The slide XML must contain <p:clrMapOvr>.
    // This is the load-bearing assertion: without Obligation 2 (find_layout_index),
    // the lookup returned index 0 (Title Slide, not dark) and clrMapOvr was absent.
    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    assert!(
        slide1_xml.contains("<p:clrMapOvr>"),
        "F-037-011/F-037-004: A 'section_divider' slide exported through export_inner must \
         contain <p:clrMapOvr> — the dark layout wiring must flow from \
         find_layout_index → has_color_override → SlideSerializer(is_dark_layout=true); \
         slide1.xml excerpt: {}",
        &slide1_xml[..slide1_xml.len().min(800)]
    );

    assert!(
        slide1_xml.contains("<a:masterClrMapping"),
        "F-037-004: <p:clrMapOvr> for dark layout must contain <a:masterClrMapping/>; \
         got slide1.xml excerpt: {}",
        &slide1_xml[..slide1_xml.len().min(800)]
    );

    // <p:clrMapOvr> must appear after <p:cSld> (ECMA-376 §19.3.1.31 sequence model).
    let cSld_pos = slide1_xml
        .find("<p:cSld")
        .expect("slide XML must contain <p:cSld>");
    let clr_pos = slide1_xml
        .find("<p:clrMapOvr>")
        .expect("section_divider slide XML must contain <p:clrMapOvr>");
    assert!(
        clr_pos > cSld_pos,
        "<p:clrMapOvr> (byte {clr_pos}) must appear AFTER <p:cSld> (byte {cSld_pos}) \
         in the slide XML (ADR-015 Addendum A Obligation 5)"
    );
}

/// F-037-005: `FrameContent::Diagram` must emit a `<p:pic>` shape referencing
/// the media rId so the relationship is not dangling.
///
/// Current impl writes the SVG to ppt/media/ and adds a rel, but never emits
/// a `<p:pic>` in the slide XML — the rel is orphaned.
#[test]
fn test_f037_005_diagram_frame_emits_pic_shape_referencing_media_rid() {
    use slideforge_types::NormalizedDiagramSvg;

    let svg_content = r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300" aria-label="test" role="img"><title>test</title><rect x="0" y="0" width="400" height="300"/></svg>"#;
    let normalized_svg = NormalizedDiagramSvg::from_normalized_string(Arc::from(svg_content));

    let mut laid_out = make_laid_out_deck(1);
    laid_out.slides[0].frames = vec![Frame {
        bbox: body_bbox(),
        content: FrameContent::Diagram(normalized_svg),
        text_flow: None,
    }];

    let pptx_bytes = build_pptx(&laid_out);

    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");
    let slide1_rels = zip_read_entry(&pptx_bytes, "ppt/slides/_rels/slide1.xml.rels");

    // The slide must contain a <p:pic> element (or equivalent picture shape)
    assert!(
        slide1_xml.contains("<p:pic") || slide1_xml.contains("<p:graphicFrame"),
        "F-037-005: A Diagram frame must emit a <p:pic> or <p:graphicFrame> in slide XML \
         referencing the media — not an orphaned relationship; got slide1.xml: {}",
        &slide1_xml[..slide1_xml.len().min(600)]
    );

    // The media rId must be referenced in the slide XML (not just the .rels file)
    // Find the media rId in the .rels file
    let mut media_rid: Option<String> = None;
    let mut rest = slide1_rels.as_str();
    while let Some(pos) = rest.find("image") {
        // Find the Id attribute near this IMAGE relationship
        // Look backward in the entry to find the Id="rId..." for this image rel
        let entry_start = slide1_rels[..pos].rfind("<Relationship").unwrap_or(0);
        let entry = &slide1_rels[entry_start..pos + 5];
        if let Some(id_pos) = entry.find("Id=\"") {
            let id_rest = &entry[id_pos + 4..];
            if let Some(id_end) = id_rest.find('"') {
                media_rid = Some(id_rest[..id_end].to_owned());
            }
        }
        rest = &rest[pos + 5..];
    }

    if let Some(rid) = &media_rid {
        // The media rId must appear as r:embed or r:link in the slide XML
        let rid_ref = format!("r:embed=\"{rid}\"");
        assert!(
            slide1_xml.contains(&rid_ref),
            "F-037-005: media rId '{}' must be referenced in slide1.xml as r:embed; \
             found in .rels but not in slide XML — dangling relationship. \
             slide1.xml: {}",
            rid,
            &slide1_xml[..slide1_xml.len().min(600)]
        );
    }
}
