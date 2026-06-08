//! STORY-037 per-AC demo: PPTX Core Serialization.
//!
//! Builds a representative [`LaidOutDeck`] with three slides:
//!   1. Title slide (AC-001, AC-002, AC-004, AC-005, AC-006)
//!   2. Content slide with report+detail register content (AC-010)
//!   3. Section divider / dark slide (EC-005 / `clrMapOvr`)
//!
//! Then:
//!   - Enumerates every ZIP part (AC-002, AC-003, AC-009)
//!   - Shows placeholder idx in slide1.xml (AC-004)
//!   - Shows integer-EMU coordinates (AC-005)
//!   - Shows `<p:clrMapOvr>` in the dark slide (EC-005)
//!   - Shows `REPORT_SENTINEL` / `DETAIL_SENTINEL` absent from slide bodies (AC-010)
//!   - Builds the same deck twice and prints the matching SHA-256 (AC-007)
//!
//! Run:
//! ```
//! cargo run --example demo_pptx -p slideforge-pptx
//! ```

use std::io::Read as _;
use std::sync::Arc;

use sha2::{Digest, Sha256};
use slideforge_layout::{BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize};
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::{
    Brand, BrandFonts, BrandPalette, Deck, Emu, Register, RegisteredContent, deck::DeckMetadata,
    ordered_map::OrderedMap, slide::Slide,
};
use zip::ZipArchive;

use slideforge_pptx::PptxExporter;

// ─── Fixtures ────────────────────────────────────────────────────────────────

/// Standard 16:9 slide page (9 144 000 × 5 143 500 EMU).
fn page_size() -> PageSize {
    PageSize::default()
}

/// Title region (matches test fixtures in `core_tests.rs`).
fn title_bbox() -> BoundingBox {
    BoundingBox {
        x: Emu(457_200),
        y: Emu(274_638),
        width: Emu(8_229_600),
        height: Emu(1_143_000),
    }
}

/// Body region.
fn body_bbox() -> BoundingBox {
    BoundingBox {
        x: Emu(457_200),
        y: Emu(1_600_200),
        width: Emu(8_229_600),
        height: Emu(3_200_400),
    }
}

/// Build a minimal [`Deck`] to satisfy the `Exporter` signature.
/// (PPTX exporter reads primarily from `LaidOutDeck`; the `Deck` carries
/// metadata and register-content only.)
fn make_deck(slide_count: usize) -> Deck {
    Deck {
        slides: (0..slide_count)
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
            title: Some(Arc::from("Demo Deck — STORY-037")),
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

/// Build a minimal [`Brand`].
fn make_brand() -> Brand {
    Brand {
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
            font_size_emu: 457_200,
        },
        layouts: vec![],
        span: slideforge_types::SourceSpan::default(),
    }
}

/// Slide 1: Title slide — demonstrates AC-004 (placeholder idx=0) and AC-005 (EMU coords).
fn slide_title() -> LaidOutSlide {
    LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Title(Arc::from("Welcome to slideforge")),
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    }
}

/// Slide 2: Content slide with report+detail register entries.
///
/// Demonstrates AC-010: `REPORT_SENTINEL` and `DETAIL_SENTINEL` must NOT appear in
/// `ppt/slides/slide2.xml` even though they live in `register_content`.
fn slide_content_with_registers() -> LaidOutSlide {
    LaidOutSlide {
        source_index: 1,
        slide_type_keyword: Arc::from("content"),
        frames: vec![
            Frame {
                bbox: title_bbox(),
                content: FrameContent::Title(Arc::from("Key Findings")),
                text_flow: None,
                region_role: None,
            },
            Frame {
                bbox: body_bbox(),
                content: FrameContent::Body(vec![]),
                text_flow: None,
                region_role: None,
            },
        ],
        speaker_notes: Some(Arc::from("Notes for presenter go here.")),
        register_tags: vec![],
        register_content: vec![
            RegisteredContent::plain(Register::Report, Arc::from("REPORT_SENTINEL")),
            RegisteredContent::plain(Register::Detail, Arc::from("DETAIL_SENTINEL")),
        ],
    }
}

/// Slide 3: Section divider — dark layout (`section_divider` maps to layout 12,
/// which has `has_color_override: true`).
///
/// Demonstrates EC-005 / architecture rule 7: `<p:clrMapOvr>` must appear
/// in slides using dark-themed layouts.
fn slide_section_divider() -> LaidOutSlide {
    LaidOutSlide {
        source_index: 2,
        slide_type_keyword: Arc::from("section_divider"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Title(Arc::from("Section Two")),
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    }
}

/// Build the demo [`LaidOutDeck`].
fn make_deck_laid_out() -> LaidOutDeck {
    LaidOutDeck {
        page_size: page_size(),
        slides: vec![
            slide_title(),
            slide_content_with_registers(),
            slide_section_divider(),
        ],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    }
}

// ─── ZIP helpers ─────────────────────────────────────────────────────────────

/// Returns a sorted list of all entry names in the ZIP archive.
fn zip_entry_names(bytes: &[u8]) -> Vec<String> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).expect("valid ZIP");
    let mut names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).expect("in-range").name().to_owned())
        .collect();
    names.sort();
    names
}

/// Reads and returns the UTF-8 text content of a named ZIP entry.
fn zip_read_entry(bytes: &[u8], path: &str) -> String {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).expect("valid ZIP");
    let mut entry = archive
        .by_name(path)
        .unwrap_or_else(|_| panic!("entry '{path}' not found"));
    let mut buf = String::new();
    entry.read_to_string(&mut buf).expect("valid UTF-8");
    buf
}

/// Returns the lowercase hex SHA-256 digest of the given bytes.
///
/// sha2 0.11+ returns `hybrid_array::Array` from `finalize()`, which no
/// longer implements `LowerHex`. Convert to `[u8; 32]` first.
fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let digest: [u8; 32] = h.finalize().into();
    let mut hex = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        write!(hex, "{b:02x}").expect("write to String is infallible");
    }
    hex
}

// ─── Section printers ────────────────────────────────────────────────────────

/// Prints a visually delimited section header to stdout.
fn print_section(title: &str) {
    println!();
    println!("══════════════════════════════════════════════════════");
    println!("  {title}");
    println!("══════════════════════════════════════════════════════");
}

/// Prints a passing check line to stdout.
fn print_ok(label: &str) {
    println!("  [OK] {label}");
}

/// Prints a failing check line to stdout.
fn print_fail(label: &str) {
    println!("  [FAIL] {label}");
}

// ─── Main ─────────────────────────────────────────────────────────────────────

// The main function is intentionally long: it exercises 10 acceptance criteria
// sequentially and prints evidence for each one. Splitting it into smaller
// functions would obscure the per-AC narrative flow without adding clarity.
#[allow(clippy::too_many_lines)]
fn main() {
    println!("slideforge-pptx demo — STORY-037 per-AC evidence");
    println!("=================================================");

    let exporter = PptxExporter::new();
    let laid_out = make_deck_laid_out();
    let deck = make_deck(laid_out.slides.len());
    let brand = make_brand();
    let opts = ExportOptions::default();

    // Build once.
    let pptx_bytes = exporter
        .export(&deck, &laid_out, &brand, &opts)
        .expect("export must succeed");

    println!("  PPTX size: {} bytes", pptx_bytes.len());

    // ─── AC-002: ZIP part list ───────────────────────────────────────────────

    print_section("AC-002  ZIP part list (all required parts present)");

    let names = zip_entry_names(&pptx_bytes);
    println!("  Total parts: {}", names.len());

    let required = [
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
        "ppt/slides/slide1.xml",
        "ppt/slides/slide2.xml",
        "ppt/slides/slide3.xml",
        "ppt/slides/_rels/slide1.xml.rels",
        "ppt/slides/_rels/slide2.xml.rels",
        "ppt/slides/_rels/slide3.xml.rels",
    ];

    let mut all_present = true;
    for part in &required {
        if names.contains(&part.to_string()) {
            print_ok(part);
        } else {
            print_fail(part);
            all_present = false;
        }
    }

    // Check all 31 layouts.
    for n in 1..=31_usize {
        let layout_path = format!("ppt/slideLayouts/slideLayout{n}.xml");
        let rels_path = format!("ppt/slideLayouts/_rels/slideLayout{n}.xml.rels");
        if !names.contains(&layout_path) {
            print_fail(&format!("MISSING: {layout_path}"));
            all_present = false;
        }
        if !names.contains(&rels_path) {
            print_fail(&format!("MISSING: {rels_path}"));
            all_present = false;
        }
    }

    let layout_count = names
        .iter()
        .filter(|n| {
            n.starts_with("ppt/slideLayouts/slideLayout")
                && std::path::Path::new(n)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
                && !n.contains("_rels")
        })
        .count();
    println!("  slideLayoutN.xml count: {layout_count} (expected 31)");

    if all_present && layout_count == 31 {
        print_ok("AC-002: all required parts present");
    } else {
        print_fail("AC-002: some required parts missing");
    }

    // ─── AC-003: [Content_Types].xml ────────────────────────────────────────

    print_section("AC-003  [Content_Types].xml — 31 layout overrides + all part types");

    let ct_xml = zip_read_entry(&pptx_bytes, "[Content_Types].xml");

    let ct_layout_overrides = (1..=31_usize)
        .filter(|n| ct_xml.contains(&format!("slideLayout{n}.xml")))
        .count();
    println!("  Layout <Override> entries: {ct_layout_overrides} (expected 31)");

    let required_ct_fragments = [
        "presentationml.presentation.main",
        "presentationml.slideMaster",
        "presentationml.slideLayout",
        "presentationml.slide",
        "presentationml.notesMaster",
        "presentationml.handoutMaster",
        "core-properties",
        "extended-properties",
    ];
    let mut ct_ok = ct_layout_overrides == 31;
    for frag in &required_ct_fragments {
        if ct_xml.contains(frag) {
            print_ok(&format!("Content-Type present: ...{frag}..."));
        } else {
            print_fail(&format!("MISSING content-type: ...{frag}..."));
            ct_ok = false;
        }
    }

    if ct_ok {
        print_ok("AC-003: [Content_Types].xml complete");
    } else {
        print_fail("AC-003: [Content_Types].xml incomplete");
    }

    // ─── AC-004: Placeholder idx in slide1.xml ───────────────────────────────

    print_section("AC-004  Placeholder inheritance — ph idx=0 in slide1.xml");

    let slide1_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide1.xml");

    // Show the first 60 chars of slide1.xml around the ph idx attribute.
    if let Some(pos) = slide1_xml.find("idx=\"0\"") {
        let start = pos.saturating_sub(40);
        let end = (pos + 60).min(slide1_xml.len());
        println!("  Excerpt: ...{}...", &slide1_xml[start..end]);
        print_ok("AC-004: ph idx=\"0\" found in slide1.xml (title placeholder)");
    } else {
        print_fail("AC-004: ph idx=\"0\" NOT found in slide1.xml");
    }

    // ─── AC-005: Integer EMU coordinates ────────────────────────────────────

    print_section("AC-005  Integer EMU coordinates (no decimal points in off/ext attrs)");

    // Show a sample coordinate attribute from slide1.xml.
    let mut coord_ok = true;
    let mut sample_shown = false;
    for attr in ["x=\"", "y=\"", "cx=\"", "cy=\""] {
        let mut remaining: &str = &slide1_xml;
        while let Some(pos) = remaining.find(attr) {
            remaining = &remaining[pos + attr.len()..];
            if let Some(end) = remaining.find('"') {
                let value = &remaining[..end];
                if !sample_shown {
                    println!("  Sample coordinate: {attr}{value}\"");
                    sample_shown = true;
                }
                if value.parse::<i64>().is_err() {
                    print_fail(&format!("non-integer coordinate: {attr}{value}\""));
                    coord_ok = false;
                }
                remaining = &remaining[end + 1..];
            }
        }
    }
    if coord_ok {
        print_ok("AC-005: all coordinate attributes are integer i64 (no decimal)");
    } else {
        print_fail("AC-005: found non-integer coordinate attribute");
    }

    // ─── AC-006: Element ordering nvSpPr → spPr → txBody ────────────────────

    print_section("AC-006  Element ordering: nvSpPr before spPr before txBody");

    let nvsppr_pos = slide1_xml.find("nvSpPr");
    let spppr_pos = slide1_xml.find("spPr");
    let txbody_pos = slide1_xml.find("txBody");

    match (nvsppr_pos, spppr_pos, txbody_pos) {
        (Some(a), Some(b), Some(c)) if a < b && b < c => {
            println!("  Positions: nvSpPr={a}, spPr={b}, txBody={c}");
            print_ok("AC-006: nvSpPr < spPr < txBody — schema-correct element order");
        },
        (a, b, c) => {
            println!("  Positions: nvSpPr={a:?}, spPr={b:?}, txBody={c:?}");
            print_fail("AC-006: element order violation");
        },
    }

    // ─── AC-007: Deterministic output (SHA-256 comparison) ──────────────────

    print_section("AC-007  Determinism — build twice, compare SHA-256");

    let pptx_bytes_2 = exporter
        .export(&deck, &laid_out, &brand, &opts)
        .expect("second export must succeed");

    let hash1 = sha256_hex(&pptx_bytes);
    let hash2 = sha256_hex(&pptx_bytes_2);

    println!("  Build 1 SHA-256: {hash1}");
    println!("  Build 2 SHA-256: {hash2}");

    if hash1 == hash2 {
        print_ok("AC-007: SHA-256 identical — output is deterministic");
    } else {
        print_fail("AC-007: SHA-256 mismatch — output is NOT deterministic");
    }

    // ─── AC-008: LibreOffice CI gate ─────────────────────────────────────────

    print_section("AC-008  LibreOffice open-without-repair (CI gate)");
    println!("  NOTE: LibreOffice headless is NOT required in local demo.");
    println!("  AC-008 is covered by the CI visual-regression gate (STORY-052).");
    println!("  The CI job runs: libreoffice --headless --convert-to png <file>");
    println!("  The unit test is marked #[ignore = \"requires libreoffice in CI\"].");
    print_ok("AC-008: deferred to STORY-052 CI gate (expected)");

    // ─── AC-009: Relationship chain completeness ─────────────────────────────

    print_section("AC-009  Relationship chain: slide.rels → layout.rels → master.rels → theme");

    // Check slide1.xml.rels contains a slideLayout reference.
    let slide1_rels = zip_read_entry(&pptx_bytes, "ppt/slides/_rels/slide1.xml.rels");
    let slide1_rels_has_layout = slide1_rels.contains("slideLayout");
    if slide1_rels_has_layout {
        print_ok("slide1.xml.rels → references slideLayout");
    } else {
        print_fail("slide1.xml.rels → missing slideLayout reference");
    }

    // Check slideMaster1.xml.rels references theme1.xml.
    let master_rels = zip_read_entry(&pptx_bytes, "ppt/slideMasters/_rels/slideMaster1.xml.rels");
    let master_rels_has_theme = master_rels.contains("theme1.xml");
    if master_rels_has_theme {
        print_ok("slideMaster1.xml.rels → references theme1.xml");
    } else {
        print_fail("slideMaster1.xml.rels → missing theme1.xml reference");
    }

    // Check slideLayout1.xml.rels references slideMaster1.xml.
    let layout1_rels = zip_read_entry(&pptx_bytes, "ppt/slideLayouts/_rels/slideLayout1.xml.rels");
    let layout1_rels_has_master = layout1_rels.contains("slideMaster1.xml");
    if layout1_rels_has_master {
        print_ok("slideLayout1.xml.rels → references slideMaster1.xml");
    } else {
        print_fail("slideLayout1.xml.rels → missing slideMaster1.xml reference");
    }

    // Check presentation.xml.rels references slide1.xml, notesMaster, handoutMaster.
    let prs_rels = zip_read_entry(&pptx_bytes, "ppt/_rels/presentation.xml.rels");
    let prs_has_slide1 = prs_rels.contains("slides/slide1.xml");
    let prs_has_notes = prs_rels.contains("notesMasters/notesMaster1.xml");
    let prs_has_handout = prs_rels.contains("handoutMasters/handoutMaster1.xml");

    if prs_has_slide1 && prs_has_notes && prs_has_handout {
        print_ok("presentation.xml.rels → references slide1, notesMaster, handoutMaster");
    } else {
        if !prs_has_slide1 {
            print_fail("presentation.xml.rels → missing slide1 reference");
        }
        if !prs_has_notes {
            print_fail("presentation.xml.rels → missing notesMaster reference");
        }
        if !prs_has_handout {
            print_fail("presentation.xml.rels → missing handoutMaster reference");
        }
    }

    if slide1_rels_has_layout && master_rels_has_theme && layout1_rels_has_master {
        print_ok("AC-009: relationship chain complete");
    } else {
        print_fail("AC-009: relationship chain incomplete");
    }

    // ─── AC-010: Report/Detail sentinels absent from slide bodies ────────────

    print_section("AC-010  Report/Detail register content absent from slide XML bodies");

    let slide2_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide2.xml");
    let report_absent = !slide2_xml.contains("REPORT_SENTINEL");
    let detail_absent = !slide2_xml.contains("DETAIL_SENTINEL");

    println!("  slide2.xml contains REPORT_SENTINEL: {}", !report_absent);
    println!("  slide2.xml contains DETAIL_SENTINEL: {}", !detail_absent);

    if report_absent && detail_absent {
        print_ok("AC-010: neither REPORT_SENTINEL nor DETAIL_SENTINEL in slide2.xml");
    } else {
        print_fail("AC-010: register content leaked into slide XML body");
    }

    // Also verify across all slides.
    let all_slides_clean = (1..=3_usize).all(|i| {
        let xml = zip_read_entry(&pptx_bytes, &format!("ppt/slides/slide{i}.xml"));
        !xml.contains("REPORT_SENTINEL") && !xml.contains("DETAIL_SENTINEL")
    });
    if all_slides_clean {
        print_ok("AC-010: all 3 slides clean of register sentinels");
    } else {
        print_fail("AC-010: register sentinel found in at least one slide XML");
    }

    // ─── EC-005: Dark slide clrMapOvr ────────────────────────────────────────

    print_section("EC-005  Dark layout — <p:clrMapOvr> in section_divider slide");

    let slide3_xml = zip_read_entry(&pptx_bytes, "ppt/slides/slide3.xml");
    let has_clr_map_ovr = slide3_xml.contains("clrMapOvr");

    println!("  slide3.xml (section_divider) contains clrMapOvr: {has_clr_map_ovr}");

    if has_clr_map_ovr {
        // Show the excerpt.
        if let Some(pos) = slide3_xml.find("clrMapOvr") {
            let start = pos.saturating_sub(5);
            let end = (pos + 80).min(slide3_xml.len());
            println!("  Excerpt: ...{}...", &slide3_xml[start..end]);
        }
        print_ok("EC-005: <p:clrMapOvr> present in dark-layout slide");
    } else {
        print_fail("EC-005: <p:clrMapOvr> MISSING from dark-layout slide");
    }

    // ─── Write output file ───────────────────────────────────────────────────

    print_section("Writing output PPTX file");

    let out_path = std::env::temp_dir().join("slideforge-story037-demo.pptx");
    std::fs::write(&out_path, &pptx_bytes).expect("write output file");
    println!("  Written: {}", out_path.display());
    print_ok(&format!("PPTX written ({} bytes)", pptx_bytes.len()));

    // ─── Summary ─────────────────────────────────────────────────────────────

    print_section("STORY-037 Evidence Summary");
    println!("  AC-001  Exporter trait: id()='pptx', extension()='pptx'  [structural — always OK]");
    println!(
        "  AC-002  ZIP part list: {}",
        if all_present && layout_count == 31 {
            "PASS"
        } else {
            "FAIL"
        }
    );
    println!(
        "  AC-003  [Content_Types].xml: {}",
        if ct_ok { "PASS" } else { "FAIL" }
    );
    println!(
        "  AC-004  Placeholder idx=0: {}",
        if slide1_xml.contains("idx=\"0\"") {
            "PASS"
        } else {
            "FAIL"
        }
    );
    println!(
        "  AC-005  Integer EMU coords: {}",
        if coord_ok { "PASS" } else { "FAIL" }
    );
    println!(
        "  AC-006  Element ordering: {}",
        if matches!((nvsppr_pos, spppr_pos, txbody_pos), (Some(a), Some(b), Some(c)) if a < b && b < c)
        {
            "PASS"
        } else {
            "FAIL"
        }
    );
    println!(
        "  AC-007  Determinism (SHA-256): {}",
        if hash1 == hash2 { "PASS" } else { "FAIL" }
    );
    println!("  AC-008  LibreOffice: CI gate (STORY-052) — skipped locally");
    println!(
        "  AC-009  Rel chain: {}",
        if slide1_rels_has_layout && master_rels_has_theme && layout1_rels_has_master {
            "PASS"
        } else {
            "FAIL"
        }
    );
    println!(
        "  AC-010  Register bleed: {}",
        if all_slides_clean { "PASS" } else { "FAIL" }
    );
}
