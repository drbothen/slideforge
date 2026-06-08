//! Integration tests for the `slideforge-html` static HTML exporter.
//!
//! ## CI-gated tests (AC-007 / AC-009)
//!
//! The `html_wcag_fixture_*` tests are `#[ignore]`'d because they require
//! `@axe-core/playwright` and a headless browser installed in the CI environment.
//! They generate fixture HTML files that the `.github/workflows/html-wcag.yml`
//! CI job feeds to axe-core for WCAG AA validation.
//!
//! Blocking CI dependency: `@axe-core/playwright` + Playwright headless browser.
//! These tests will be un-ignored by the CI workflow, not by Rust unit test runs.
//!
//! Per SID-1 discipline: the Rust-verifiable behavior (HTML structure, role/alt
//! attributes, lang, no canvas) is covered by unit tests in `exporter.rs` and
//! `render.rs`. These integration tests exist ONLY for the axe-core CI pipeline.
//!
//! Story: STORY-046 (AC-007 / AC-009)

#![allow(non_snake_case)]

use std::sync::Arc;

use slideforge_html::HtmlExporter;
use slideforge_layout::{BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize};
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::{
    AltText, Brand, BrandFonts, BrandPalette, Deck, DeckMetadata, Emu, NormalizedDiagramSvg,
    OrderedMap, SourceSpan,
};

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures
// ─────────────────────────────────────────────────────────────────────────────

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
            heading: Arc::from("Calibri"),
            body: Arc::from("Calibri"),
            mono: Arc::from("Courier New"),
            font_size_emu: 457_200,
        },
        layouts: vec![],
        span: SourceSpan::default(),
    }
}

fn make_deck(lang: &str) -> Deck {
    Deck {
        slides: vec![],
        vars: OrderedMap::new(),
        metadata: DeckMetadata {
            title: Some(Arc::from("Fixture Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from(lang)),
            author: None,
            section_order: None,
        },
        registers: OrderedMap::new(),
        section_blocks: vec![],
    }
}

fn make_three_slide_laid_out_deck() -> LaidOutDeck {
    let bbox = BoundingBox {
        x: Emu(0),
        y: Emu(0),
        width: Emu(9_144_000),
        height: Emu(5_143_500),
    };

    // Slide 1: title slide with Title + Subtitle (the common cover shape — CRIT-1 regression guard).
    // Title at H1 level → Subtitle must emit <h2> (not <h3>) to avoid h1→h3 skip.
    // Including this shape in the axe-core fixture ensures the CI WCAG gate exercises it.
    let title_bbox = BoundingBox {
        x: Emu(0),
        y: Emu(0),
        width: Emu(9_144_000),
        height: Emu(1_800_000),
    };
    let subtitle_bbox = BoundingBox {
        x: Emu(0),
        y: Emu(1_800_000),
        width: Emu(9_144_000),
        height: Emu(3_343_500),
    };
    let slide1 = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![
            Frame {
                bbox: title_bbox,
                content: FrameContent::Title(Arc::from("Fixture Presentation")),
                text_flow: None,
                region_role: None,
            },
            Frame {
                bbox: subtitle_bbox,
                content: FrameContent::Subtitle(Arc::from(
                    "Exploring accessible document generation",
                )),
                text_flow: None,
                region_role: None,
            },
        ],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    };

    // Slide 2: non-decorative image with alt text
    let slide2 = LaidOutSlide {
        source_index: 1,
        slide_type_keyword: Arc::from("content"),
        frames: vec![Frame {
            bbox,
            content: FrameContent::Image {
                alt: AltText::Provided(Arc::from(
                    "Bar chart: Q1 $1M, Q2 $1.3M, Q3 $1.2M, Q4 $1.8M",
                )),
            },
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    };

    // Slide 3: chart SVG with role="img" and <title>
    let svg_str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"><rect x="0" y="0" width="400" height="300" fill="#003087"/></svg>"##;
    let normalized_svg = NormalizedDiagramSvg::from_normalized_string(Arc::from(svg_str));
    let slide3 = LaidOutSlide {
        source_index: 2,
        slide_type_keyword: Arc::from("content"),
        frames: vec![Frame {
            bbox,
            content: FrameContent::Diagram {
                svg: normalized_svg,
                alt: AltText::Provided(Arc::from("Revenue by quarter")),
            },
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    };

    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![slide1, slide2, slide3],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-007: WCAG AA validation via axe-core (CI-gated, #[ignore])
// ─────────────────────────────────────────────────────────────────────────────

/// AC-007 — Zero WCAG AA violations via `@axe-core/playwright` in CI.
///
/// This test is `#[ignore]`'d because it requires `@axe-core/playwright` and a
/// headless browser installed in the CI environment (`.github/workflows/html-wcag.yml`).
/// The CI workflow un-ignores this test and feeds the generated HTML file to
/// axe-core for WCAG AA validation.
///
/// Blocking CI dependency: `@axe-core/playwright` + Playwright headless browser.
/// Story: STORY-046 (AC-007 / BC-4.03.003 postcondition 2 / invariant 4)
///
/// Per SID-1 discipline: the Rust-verifiable structural invariants (lang, no
/// canvas, role/alt attributes) are covered by unit tests in `exporter.rs` and
/// `render.rs`. This test exists solely to emit the fixture HTML for axe-core.
#[test]
#[ignore = "requires @axe-core/playwright + Playwright headless browser (CI-gated)"]
fn test_BC_4_03_003_ac_007_wcag_aa_fixture_generation_for_axe_core() {
    let exporter = HtmlExporter::new();
    let deck = make_deck("en-US");
    let laid_out = make_three_slide_laid_out_deck();
    let brand = make_brand();
    let opts = ExportOptions::default();

    let bytes = exporter
        .export(&deck, &laid_out, &brand, &opts)
        .expect("export must succeed for WCAG fixture");
    let html = String::from_utf8(bytes).expect("output must be valid UTF-8");

    // Write the fixture HTML to a known path for the CI axe-core job to pick up.
    let fixture_path = std::env::temp_dir().join("slideforge-html-wcag-fixture.html");
    std::fs::write(&fixture_path, &html)
        .unwrap_or_else(|e| panic!("failed to write fixture HTML to {fixture_path:?}: {e}"));

    // Structural pre-assertions (belt-and-suspenders before axe-core runs):
    assert!(
        html.contains(r#"lang="en-US""#),
        "fixture must contain lang=\"en-US\""
    );
    assert!(
        html.trim_start().starts_with("<!DOCTYPE html>"),
        "fixture must start with <!DOCTYPE html>"
    );

    let doc = scraper::Html::parse_document(&html);
    let canvas_sel = scraper::Selector::parse("canvas").expect("valid selector");
    assert_eq!(
        doc.select(&canvas_sel).count(),
        0,
        "fixture must contain zero <canvas> elements"
    );
}

/// CRIT-1 heading-order fixture validation (Rust-verifiable, NOT ignored).
///
/// Verifies that the axe-core fixture HTML (generated by `make_three_slide_laid_out_deck`)
/// has correct heading order: exactly one h1 document-wide, subtitle as h2 on the
/// title slide (not h3), and no heading-level skip anywhere in the document.
///
/// This test runs on every `cargo test` (no `#[ignore]`) — it is the Rust-level
/// gate ensuring the fixture itself is correct before axe-core sees it.
///
/// Story: STORY-046 (CRIT-1 / BC-4.03.003 invariant 4 / AC-007 / ADR-008)
#[test]
fn test_CRIT1_fixture_heading_order_no_level_skip() {
    let exporter = HtmlExporter::new();
    let deck = make_deck("en-US");
    let laid_out = make_three_slide_laid_out_deck();
    let brand = make_brand();
    let opts = ExportOptions::default();

    let bytes = exporter
        .export(&deck, &laid_out, &brand, &opts)
        .expect("export must succeed");
    let html = String::from_utf8(bytes).expect("valid UTF-8");

    let doc = scraper::Html::parse_document(&html);

    // Must have exactly one <h1> document-wide.
    let h1_sel = scraper::Selector::parse("h1").expect("valid selector");
    assert_eq!(
        doc.select(&h1_sel).count(),
        1,
        "CRIT-1 fixture: exactly one h1 must exist in the full document; got html of length {}",
        html.len()
    );

    // Must have exactly one <h2> (the Subtitle on slide 1, level = h1+1 = h2).
    let h2_sel = scraper::Selector::parse("h2").expect("valid selector");
    assert_eq!(
        doc.select(&h2_sel).count(),
        1,
        "CRIT-1 fixture: exactly one h2 must exist (Subtitle on title slide); \
         if this is 0, the CRIT-1 fix regressed and Subtitle became h3 again"
    );

    // Must have NO <h3> in the fixture document (no content slides with subtitle in this fixture).
    let h3_sel = scraper::Selector::parse("h3").expect("valid selector");
    assert_eq!(
        doc.select(&h3_sel).count(),
        0,
        "CRIT-1 fixture: no h3 should appear in this fixture (would indicate h1→h3 skip)"
    );

    // h1 must appear before h2 in document order (title precedes subtitle).
    let h1_pos = html.find("<h1").expect("<h1> must be present");
    let h2_pos = html.find("<h2").expect("<h2> must be present");
    assert!(
        h1_pos < h2_pos,
        "CRIT-1 fixture: h1 must appear before h2 in document order"
    );
}

/// AC-009 — Body text contrast ratio assertion (CI-gated, #[ignore]).
///
/// This test is `#[ignore]`'d because contrast ratio validation via
/// `axe-core`'s `color-contrast` rule requires a headless browser.
///
/// Blocking CI dependency: `@axe-core/playwright` (color-contrast rule).
/// Story: STORY-046 (AC-009 / BC-4.03.003 postcondition 8)
///
/// Per SID-1: the compile-time color contrast gate (BC-5.01.003) blocks
/// low-contrast brand colors before they reach the HTML exporter. The axe-core
/// `color-contrast` rule in CI confirms the structural guarantee at runtime.
#[test]
#[ignore = "requires @axe-core/playwright color-contrast rule (CI-gated); structural coverage in unit tests"]
fn test_BC_4_03_003_ac_009_color_contrast_axe_core_ci_gate() {
    // This test generates the same fixture as AC-007. The CI workflow runs
    // axe-core with `--rules color-contrast` on the fixture HTML.
    // No assertions here — the axe-core exit code is the gate.
    let exporter = HtmlExporter::new();
    let deck = make_deck("en-US");
    let laid_out = make_three_slide_laid_out_deck();
    let brand = make_brand();
    let opts = ExportOptions::default();

    let bytes = exporter
        .export(&deck, &laid_out, &brand, &opts)
        .expect("export must succeed for contrast fixture");
    let html = String::from_utf8(bytes).expect("valid UTF-8");
    let fixture_path = std::env::temp_dir().join("slideforge-html-contrast-fixture.html");
    std::fs::write(&fixture_path, html)
        .unwrap_or_else(|e| panic!("failed to write contrast fixture: {e}"));
}
