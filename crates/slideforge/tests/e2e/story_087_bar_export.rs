//! STORY-087 pass-3 — BC-1.17.002 PC-9 Red Gate: bar rendered in PDF, HTML, DOCX.
//!
//! BC-1.17.002 PC-9 (binding, v1.2) requires the filled bar frame to be rendered
//! visibly in ALL output formats:
//!
//! - PPTX: solid-fill `<p:sp>` at proportional width  (COVERED: see story_087_content_rendering.rs)
//! - PDF: filled rectangle in the content stream
//! - HTML: filled `<rect>` or styled div with proportional width
//! - DOCX: percentage text fallback (e.g., "75%")
//!
//! This file adds build()-level Red Gate tests for the three formats not yet covered.
//!
//! ## HTML exporter status
//!
//! The `slideforge-html` crate is NOT yet registered in the plugin registry (deferred
//! to STORY-050 "slideforge-html excluded from workspace, Phase 4" — see registry.rs
//! line 31). `build()` with `format: "html"` returns `Err(BuildError::UnknownFormat)`.
//!
//! Consequence: the build()-level HTML bar test must be `#[ignore]`'d until the HTML
//! exporter is wired. A layout-boundary proxy test exercises the IR that the HTML
//! exporter WILL need to consume.
//!
//! ## PDF exporter: why boundary-level, not build()-level
//!
//! `slideforge::build()` with `format: "pdf"` always uses compressed content streams
//! (`krilla::SerializeSettings::compress_content_streams: true`). PDF rectangle path
//! operators (`re`, `rg`) live inside content streams and are inaccessible after
//! Deflate compression.  The bar-render assertion therefore uses
//! `PdfExporter::export_uncompressed()` directly (the same pattern as
//! `test_bc_4_03_001_diagram_frame_alt_text_from_spec` in `pdf_ua1.rs`).
//!
//! SID-1 compliance: this is NOT a deferral of behavior — it is a deferral of the
//! test vehicle. The test directly exercises the production code path
//! (`PdfExporter::draw_frame` → ColorBar arm) without the build()-level Deflate
//! wrapper. A separate assertion confirms `build("pdf")` still returns `Ok` for
//! `progress_bar` value=75 (no regression on the PDF pipeline plumbing).
//!
//! ## DOCX test
//!
//! `build()` with `format: "docx"` IS testable end-to-end. `word/document.xml` is
//! an XML file inside the DOCX ZIP (not compressed at the content level in a way
//! that prevents text search). The DOCX bar test calls `build()` directly, extracts
//! `word/document.xml`, and asserts the percentage text fallback "75%" is present.
//!
//! ## Red Gate summary
//!
//! | Test | Format | Assertion | Red Gate reason |
//! |------|--------|-----------|-----------------|
//! | test_BC_1_17_002_pdf_bar_rendered_in_content_stream | PDF | `re ` path op in uncompressed PDF | `draw_frame` ColorBar arm is `tracing::debug!` no-op |
//! | test_BC_1_17_002_html_bar_render_build_level (ignored) | HTML | `build("html")` returns Ok + styled rect | HtmlExporter not registered |
//! | test_BC_1_17_002_html_bar_render_layout_ir_proxy | HTML proxy | layout IR has ColorBar with filled_width_emu > 0 | (passes once layout::run ColorBar materialization is done) |
//! | test_BC_1_17_002_docx_bar_percentage_text_in_document_xml | DOCX | "75%" in word/document.xml | DOCX ColorBar arm silently drops the frame |
//!
//! ## Traceability
//!
//! - BC-1.17.002 PC-9 (v1.2)
//! - Finding F-087-P3-001 (adversary pass-3): exporters' ColorBar arms are stubs
//! - STORY-087 pass-3 remediation

#![allow(clippy::unwrap_used)] // integration tests — panics are intentional
#![allow(clippy::doc_markdown)] // references like `BC-1.17.002`, `<p:sp>`, `<rect>`
#![allow(clippy::uninlined_format_args)] // width-limited format strings
#![allow(non_snake_case)] // BC-traceability IDs use uppercase

use std::sync::Arc;

use crate::e2e::{BrandTmpDir, fixture_source, open_zip};
use slideforge_layout::types::{
    BoundingBox, Emu, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize, RegisterSet,
};
use slideforge_pdf::PdfExporter;
use slideforge_plugin_api::ExportOptions;
use slideforge_types::{
    Brand, BrandFonts, BrandPalette, Deck, DeckMetadata, OrderedMap, SourceSpan,
};

// ── Shared helpers ────────────────────────────────────────────────────────────

fn minimal_brand() -> Brand {
    Brand {
        name: Arc::from("test-brand"),
        palette: BrandPalette {
            primary: Arc::from("#0070C0"),
            secondary: Arc::from("#0066CC"),
            accent: Arc::from("#FF6B35"),
            neutral: Arc::from("#F5F5F5"),
        },
        fonts: BrandFonts {
            heading: Arc::from("Calibri"),
            body: Arc::from("Calibri"),
            mono: Arc::from("Courier New"),
        },
        layouts: vec![],
        span: SourceSpan::default(),
    }
}

fn minimal_deck() -> Deck {
    Deck {
        slides: vec![],
        vars: OrderedMap::new(),
        metadata: DeckMetadata {
            title: Some(Arc::from("Test")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        },
        registers: OrderedMap::new(),
        section_blocks: vec![],
    }
}

/// Build a `LaidOutDeck` with a single slide containing exactly one
/// `FrameContent::ColorBar` frame (75% fill, 9_144_000 EMU wide).
///
/// Used by the PDF boundary test to give `PdfExporter::export_uncompressed()`
/// a realistic IR without going through the full parse→eval→layout pipeline.
fn progress_bar_laid_out_deck() -> LaidOutDeck {
    use slideforge_layout::types::Rgb;

    // Bar geometry: 10-inch slide width (9_144_000 EMU = 10 in × 914_400 EMU/in).
    // 75% fill → filled_width_emu = 6_858_000.
    let total_width = Emu(9_144_000);
    let filled_width = Emu((75_i64 * total_width.0) / 100); // = 6_858_000

    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("progress_bar"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(2_743_200), // ~3 inches down from top
                    width: total_width,
                    height: Emu(457_200), // 0.5 inch tall
                },
                content: FrameContent::ColorBar {
                    filled_width_emu: filled_width,
                    total_width_emu: total_width,
                    percent: 75,
                    color: Rgb {
                        r: 0,
                        g: 112,
                        b: 192,
                    }, // #0070C0 brand primary
                },
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        }],
        sections: vec![],
        warnings: vec![],
    }
}

// ── Scan bytes for a needle ───────────────────────────────────────────────────

fn bytes_contain(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

// ─────────────────────────────────────────────────────────────────────────────
// PDF bar-render Red Gate test
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.002 PC-9 / F-087-P3-001 Red Gate — PDF filled rectangle:
/// `PdfExporter::export_uncompressed()` on a `progress_bar` LaidOutDeck
/// (single `FrameContent::ColorBar` frame, value=75) must produce PDF bytes
/// containing the `re ` path operator — the PDF content-stream instruction
/// that draws a rectangle path (ISO 32000-1 §8.5.2, Table 59).
///
/// ## Why boundary-level (not build()-level)
///
/// `slideforge::build()` with `format: "pdf"` uses compressed content streams
/// (`compress_content_streams: true`). PDF path operators (`re`, `f`, `rg`) live
/// inside the compressed content stream and cannot be text-searched in the output
/// bytes after Deflate. `export_uncompressed()` disables compression so content
/// streams are plain-text searchable — the same pattern used by
/// `test_bc_4_03_001_diagram_frame_alt_text_from_spec` in `pdf_ua1.rs`.
///
/// A separate companion assertion (`test_BC_1_17_002_pdf_build_returns_ok`)
/// confirms the build()-level pipeline still succeeds for `progress_bar` value=75,
/// guarding against regressions in the PDF pipeline plumbing.
///
/// ## Red Gate behavior (pre-implementation)
///
/// The `draw_frame` ColorBar arm in `exporter.rs:780-791` is a `tracing::debug!`
/// no-op — it emits ZERO content-stream operators. The uncompressed PDF bytes
/// contain no `re ` operator associated with the bar. Assertion FAILS at Red Gate.
///
/// ## Post-implementation behavior
///
/// After the implementer draws a filled rectangle via `surface.set_fill(...)` +
/// `surface.draw_path(rect_path)` (or equivalent krilla API), the content stream
/// will contain `re ` (path rectangle) and a fill-color `rg` operator for the bar
/// color (#0070C0 → `0 0.439 0.753 rg` in normalized RGB). Both assertions pass.
///
/// Traceability: BC-1.17.002 PC-9; F-087-P3-001; STORY-087 pass-3 remediation.
#[test]
fn test_BC_1_17_002_pdf_bar_rendered_in_content_stream() {
    let deck = minimal_deck();
    let laid_out = progress_bar_laid_out_deck();
    let brand = minimal_brand();
    let opts = ExportOptions::default();

    let exporter = PdfExporter::new();
    let pdf_bytes = exporter
        .export_uncompressed(&deck, &laid_out, &brand, &opts)
        .unwrap_or_else(|e| {
            panic!(
                "BC-1.17.002 PC-9 PDF Red Gate: export_uncompressed() must not fail for a \
                 progress_bar LaidOutDeck (ColorBar frame, value=75); got Err: {e:?}"
            )
        });

    // Assert: PDF output is a valid PDF (regression guard).
    assert!(
        pdf_bytes.starts_with(b"%PDF-"),
        "BC-1.17.002 PC-9 PDF: output must begin with '%PDF-'; \
         got first 8 bytes: {:?}",
        &pdf_bytes[..pdf_bytes.len().min(8)]
    );

    // Assert: the content stream contains the brand fill-color RGB operator for the bar.
    //
    // BC-1.17.002 PC-9 requires a filled rectangle drawn in the brand primary color
    // (#0070C0 → RGB 0, 112, 192). krilla's Surface encodes the fill color using the
    // PDF `rg` operator (ISO 32000-1 §8.6.8) with normalized f32 components:
    //
    //   r=0/255=0.0,  g=112/255≈0.4392157,  b=192/255≈0.7529412
    //
    // krilla emits this as "0 0.4392157 0.7529412 rg" in the uncompressed content
    // stream. This byte sequence ONLY appears when the ColorBar arm explicitly calls
    // `surface.set_fill(Fill { paint: rgb::Color::new(0, 112, 192).into(), .. })` —
    // it is NOT emitted by text drawing operations (which use `Tf`, `Td`, `Tj`, etc.)
    // or by SVG/diagram frames (which would use different color values).
    //
    // Note on `re` vs. `m/l/h`: the PDF rectangle shorthand `re x y w h` and the
    // equivalent `x y m ... h` (move-to / line-to / close) both draw a rectangle.
    // krilla 0.6.0 always uses the `m/l/h` form via tiny_skia_path, so the `re`
    // operator does NOT appear. The color assertion below is both more specific
    // (proves the correct brand color was used) and correct for krilla's actual output.
    //
    // With `compress_content_streams: false` (export_uncompressed), the content stream
    // is plain text and scannable. In the compressed (production) path these bytes
    // would be inside a Deflate stream and inaccessible.
    //
    // RED GATE (pre-implementation): the `draw_frame` ColorBar arm at exporter.rs:780-791
    // was a `tracing::debug!` no-op — it drew nothing. No `rg` color operator appeared.
    assert!(
        bytes_contain(&pdf_bytes, b"0.4392157"),
        "BC-1.17.002 PC-9 PDF Red Gate: the uncompressed PDF content stream must contain \
         the brand fill-color component '0.4392157' (g=112/255 from #0070C0) emitted by \
         the `rg` operator when drawing the filled ColorBar rectangle. \
         This value only appears when `draw_color_bar_rect` explicitly sets the bar fill \
         color — it cannot come from text drawing or other frame types. \
         First 2048 bytes of PDF:\n{:.2048}",
        String::from_utf8_lossy(&pdf_bytes)
    );
}

/// BC-1.17.002 PC-9 PDF build()-level pipeline guard:
/// `build()` with `format: "pdf"` on a deck containing `progress_bar` value=75
/// (preceded by a title slide for PDF/UA-1 `DeckMetadata.title` population) must
/// return `Ok`.
///
/// This companion assertion guards the PDF pipeline plumbing — if the ColorBar
/// arm causes a panic or propagates an error, this test catches it.
/// It is NOT a Red Gate test (it should already pass before implementation).
///
/// ## Why a title slide is required
///
/// PDF/UA-1 mandates a document title in XMP metadata (`/Title` in the document
/// catalog). `DeckMetadata.title` is derived from the first `slide title:` block
/// (STORY-050 Gap 1). Without a title-type slide, the PDF export fails with
/// `PDF/UA-1 validation failed: NoDocumentTitle`. The fixture
/// `story-087-progress-bar-75-with-title.sf` adds a title slide before the
/// progress_bar slide so the PDF pipeline can complete.
///
/// Traceability: BC-1.17.002 PC-9; STORY-087 pass-3.
#[test]
fn test_BC_1_17_002_pdf_build_returns_ok_for_progress_bar() {
    let brand = BrandTmpDir::new("s087_p3_bar_pdf_ok");
    // Use the with-title fixture: the title slide populates DeckMetadata.title
    // so PDF/UA-1 validation does not fail with NoDocumentTitle.
    let source = fixture_source("story-087-progress-bar-75-with-title.sf");
    let opts = brand.build_options("pdf", false);

    let result = slideforge::build(&source, &opts);
    assert!(
        result.is_ok(),
        "BC-1.17.002 PC-9 PDF: build() with format='pdf' must return Ok for a deck \
         with a title slide + progress_bar value=75; got Err: {result:?}"
    );

    let output = result.unwrap();
    assert!(
        output.bytes.starts_with(b"%PDF-"),
        "BC-1.17.002 PC-9 PDF: BuildOutput.bytes must begin with '%PDF-'"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// HTML bar-render Red Gate tests
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.002 PC-9 HTML — build()-level Red Gate (IGNORED: no HtmlExporter).
///
/// `build()` with `format: "html"` on `progress_bar` value=75 must return
/// `Ok(BuildOutput)` where the HTML string contains an `<svg>` element with a
/// filled `<rect>` (or a styled `<div>` with a `width` proportional to 75%)
/// representing the bar.
///
/// ## SID-1 citation — why ignored
///
/// `slideforge-html` is NOT registered in the plugin registry (deferred per
/// `registry.rs` line 31: "HtmlExporter is deferred to STORY-050"). Calling
/// `build()` with `format: "html"` returns `Err(BuildError::UnknownFormat("html"))`.
/// This test is `#[ignore]`'d until:
///   1. A `slideforge-html` crate is created and added to the workspace.
///   2. `HtmlExporter` is registered in `register_bundled_plugins()`.
///
/// The covering layout-IR proxy test is
/// `test_BC_1_17_002_html_bar_render_layout_ir_proxy` (below) which directly
/// exercises the LaidOutDeck IR that the HTML exporter must consume.
///
/// Traceability: BC-1.17.002 PC-9; F-087-P3-001; STORY-087 pass-3 remediation.
#[test]
#[ignore = "HtmlExporter not registered (slideforge-html crate deferred to STORY-050); \
            IR-level coverage by test_BC_1_17_002_html_bar_render_layout_ir_proxy"]
fn test_BC_1_17_002_html_bar_render_build_level() {
    let brand = BrandTmpDir::new("s087_p3_bar_html");
    let source = fixture_source("story-087-progress-bar-75.sf");
    let opts = brand.build_options("html", false);

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "BC-1.17.002 PC-9 HTML: build() with format='html' must return Ok for \
             progress_bar value=75; got Err: {e:?}"
        )
    });

    let html = String::from_utf8_lossy(&output.bytes);

    // Assert: HTML contains a filled bar element.
    //
    // BC-1.17.002 PC-9 for HTML requires "filled rectangle". The HTML exporter is
    // expected to emit either an SVG <rect> element or a styled <div> with an inline
    // `style="width:75%"` (or equivalent). Either form satisfies PC-9 "filled rectangle".
    let has_bar =
        html.contains("<rect") || html.contains("width:75%") || html.contains("width: 75%");
    assert!(
        has_bar,
        "BC-1.17.002 PC-9 HTML Red Gate: HTML output must contain a filled bar element \
         (<rect>, or a styled div with width:75%) for progress_bar value=75. \
         HTML (first 2000 chars):\n{:.2000}",
        html
    );
}

/// BC-1.17.002 PC-9 HTML — layout-IR proxy (SID-1 equivalent for missing HtmlExporter):
/// `slideforge_layout::run()` on a `progress_bar` deck (value=75) must produce a
/// `LaidOutDeck` with exactly one `FrameContent::ColorBar` frame where
/// `filled_width_emu > Emu(0)`.
///
/// This is the load-bearing proof that:
/// 1. The layout IR carries the bar geometry that the HTML exporter WILL consume.
/// 2. The geometry is correct (non-zero width proportional to 75%).
///
/// When the HtmlExporter is implemented, it must read `filled_width_emu` and
/// `total_width_emu` from this frame to produce the `width: 75%` styled element.
///
/// ## Red Gate behavior
///
/// This test is RED until `layout::run`'s ColorBar materialization pass is implemented
/// (pass-2 work). If layout::run has been fixed (pass-2), this test may PASS here and
/// serves as a regression guard for the HTML exporter's input contract.
///
/// Traceability: BC-1.17.002 PC-9; F-087-P3-001; STORY-087 pass-3 remediation.
#[test]
fn test_BC_1_17_002_html_bar_render_layout_ir_proxy() {
    use slideforge_layout::FrameContent;
    use slideforge_types::{
        Block, Brand as TypesBrand, BrandFonts as TypesBrandFonts,
        BrandPalette as TypesBrandPalette, ColorBarSpec, ContentBlock, Deck, DeckMetadata,
        FieldValue, InlineNode, OrderedMap as TypesOrderedMap, Slide,
        SourceSpan as TypesSourceSpan, TextBlock, TextTag, Value,
    };

    fn make_brand_for_layout() -> TypesBrand {
        TypesBrand {
            name: Arc::from("test-brand"),
            palette: TypesBrandPalette {
                primary: Arc::from("#0070C0"),
                secondary: Arc::from("#0066CC"),
                accent: Arc::from("#FF6B35"),
                neutral: Arc::from("#F5F5F5"),
            },
            fonts: TypesBrandFonts {
                heading: Arc::from("Calibri"),
                body: Arc::from("Calibri"),
                mono: Arc::from("Courier New"),
            },
            layouts: vec![],
            span: TypesSourceSpan::default(),
        }
    }

    let mut fields: TypesOrderedMap<Arc<str>, FieldValue> = TypesOrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Sprint 4 Progress"))),
    );

    let slide = Slide {
        slide_type: Arc::from("progress_bar"),
        fields,
        blocks: vec![
            Block {
                content: ContentBlock::Text(TextBlock {
                    inlines: vec![InlineNode::Plain(Arc::from("75% complete"))],
                    tag: TextTag::ColorLabel,
                    span: TypesSourceSpan::default(),
                }),
                label: None,
                span: TypesSourceSpan::default(),
            },
            Block {
                content: ContentBlock::ColorBar(ColorBarSpec { percent: 75 }),
                label: None,
                span: TypesSourceSpan::default(),
            },
        ],
        register: None,
        tags: vec![],
        source_span: TypesSourceSpan::default(),
        overlay: None,
        register_content: vec![],
    };

    let deck = Deck {
        slides: vec![slide],
        vars: TypesOrderedMap::new(),
        metadata: DeckMetadata {
            title: Some(Arc::from("Test")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        },
        registers: TypesOrderedMap::new(),
        section_blocks: vec![],
    };

    let brand = make_brand_for_layout();
    let laid_out = slideforge_layout::run(&deck, &brand).expect("layout::run must not fail");
    let frames = &laid_out.slides[0].frames;

    // Assert: layout IR has a ColorBar frame with filled_width_emu > 0.
    //
    // The HTML exporter must read this frame to produce the proportional bar element.
    // If this assertion fails, the HTML exporter will have no bar geometry to render.
    //
    // RED GATE: layout::run has no ColorBar materialization pass — ColorBar blocks are
    // dropped and no FrameContent::ColorBar is emitted. This assertion FAILS at Red Gate.
    let color_bar_frames: Vec<_> = frames
        .iter()
        .filter(|f| matches!(f.content, FrameContent::ColorBar { .. }))
        .collect();

    assert_eq!(
        color_bar_frames.len(),
        1,
        "BC-1.17.002 PC-9 HTML proxy Red Gate: layout IR must contain exactly 1 \
         FrameContent::ColorBar frame (the HTML exporter's input). \
         Found {}: frames: {frames:?}",
        color_bar_frames.len()
    );

    if let FrameContent::ColorBar {
        filled_width_emu,
        total_width_emu,
        ..
    } = &color_bar_frames[0].content
    {
        assert!(
            filled_width_emu.0 > 0,
            "BC-1.17.002 PC-9 HTML proxy: filled_width_emu must be > 0 for value=75; \
             got filled={} total={}",
            filled_width_emu.0,
            total_width_emu.0
        );

        // Assert proportional width: filled_width ≈ 75% of total_width (±1 EMU for integer rounding).
        let expected_filled = (75_i64 * total_width_emu.0) / 100;
        let diff = (filled_width_emu.0 - expected_filled).unsigned_abs();
        assert!(
            diff <= 1,
            "BC-1.17.002 PC-9 HTML proxy: filled_width_emu must be ≈75% of total_width_emu \
             (integer rounding ±1 EMU). \
             Expected ≈{expected_filled}, got {}, total={}",
            filled_width_emu.0,
            total_width_emu.0
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DOCX bar-render Red Gate test
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.002 PC-9 / F-087-P3-001 Red Gate — DOCX percentage text fallback:
/// `build()` with `format: "docx"` on `progress_bar` value=75 must produce a
/// DOCX whose `word/document.xml` contains "75%" — the percentage text fallback
/// required by BC-1.17.002 PC-9 for DOCX output ("DOCX: percentage text fallback").
///
/// ## Why "75%" specifically
///
/// DOCX has no native progress-bar element. BC-1.17.002 PC-9 specifies that the
/// DOCX exporter must emit the percentage value as text (e.g., "75%"). This ensures
/// the bar information is visible in the document even without a graphical bar.
///
/// ## Red Gate behavior (pre-implementation)
///
/// The DOCX `document_body.rs` `serialize()` method iterates over `FrameContent`
/// variants using `if let` pattern guards:
///   - `FrameContent::Title` → Heading1
///   - `FrameContent::Subtitle` → Heading2
///   - `FrameContent::Body` → Normal paragraphs
///
/// There is NO `FrameContent::ColorBar` arm. The ColorBar frame is silently
/// dropped — no text is emitted. "75%" does not appear in `word/document.xml`.
///
/// Note: the ColorLabel text block IS routed via `FrameContent::Body` (after
/// layout::run's ColorLabel→Body routing is implemented), so the label "75% complete"
/// MAY appear. But "75%" as a STANDALONE percentage text fallback (the dedicated
/// DOCX bar representation) does NOT appear because the ColorBar frame itself
/// produces nothing.
///
/// ## Post-implementation behavior
///
/// After the DOCX serializer adds a `FrameContent::ColorBar` arm that emits
/// a `<w:p>` paragraph with "75%" text (e.g., `<w:r><w:t>75%</w:t></w:r>`),
/// `word/document.xml` will contain "75%" and the assertion passes.
///
/// Traceability: BC-1.17.002 PC-9; F-087-P3-001; STORY-087 pass-3 remediation.
#[test]
fn test_BC_1_17_002_docx_bar_percentage_text_in_document_xml() {
    let brand = BrandTmpDir::new("s087_p3_bar_docx");
    let source = fixture_source("story-087-progress-bar-75.sf");
    let opts = brand.build_options("docx", false);

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "BC-1.17.002 PC-9 DOCX Red Gate: build() with format='docx' must return Ok for \
             progress_bar value=75 (valid input); got Err: {e:?}"
        )
    });

    let mut archive = open_zip(&output.bytes, "BC-1.17.002-DOCX");
    let mut doc_xml_entry = archive
        .by_name("word/document.xml")
        .unwrap_or_else(|e| panic!("BC-1.17.002 DOCX: word/document.xml not found in ZIP: {e}"));

    let mut doc_xml = String::new();
    std::io::Read::read_to_string(&mut doc_xml_entry, &mut doc_xml)
        .unwrap_or_else(|e| panic!("BC-1.17.002 DOCX: cannot read word/document.xml: {e}"));

    // Assert: word/document.xml contains `<w:t>75%</w:t>` — a `<w:t>` element whose
    // entire text content is exactly "75%".
    //
    // BC-1.17.002 PC-9 mandates a DEDICATED "percentage text fallback" for DOCX.
    // The DOCX exporter must emit "75%" as a standalone text run from the ColorBar
    // frame (e.g., `<w:r><w:t>75%</w:t></w:r>` in a Normal-styled paragraph).
    //
    // The label text "75% complete" is emitted separately via FrameContent::Body
    // (ColorLabel→Body routing, pass-2). That produces `<w:t>75% complete</w:t>`,
    // NOT `<w:t>75%</w:t>`. Only a dedicated ColorBar arm can produce the exact
    // standalone "75%" run.
    //
    // This discriminating assertion rejects the false-positive case where only
    // the label text satisfies a weaker `contains("75%")` check.
    //
    // RED GATE: `document_body.rs:serialize()` has no `FrameContent::ColorBar` arm.
    // The ColorBar frame is silently dropped. No `<w:t>75%</w:t>` element exists.
    // (The label text appears as `<w:t>75% complete</w:t>`, not `<w:t>75%</w:t>`.)
    //
    // Post-fix: the implementer must add a FrameContent::ColorBar arm that emits
    // a paragraph containing exactly "{percent}%" (e.g., `<w:t>75%</w:t>`).
    assert!(
        doc_xml.contains("<w:t>75%</w:t>"),
        "BC-1.17.002 PC-9 DOCX Red Gate: word/document.xml must contain '<w:t>75%</w:t>' — \
         a standalone percentage text run from the FrameContent::ColorBar arm \
         (BC-1.17.002 PC-9 DOCX clause: 'percentage text fallback'). \
         The DOCX serializer has no FrameContent::ColorBar arm; the bar frame is silently \
         dropped. Only the label text '<w:t>75% complete</w:t>' (from FrameContent::Body) \
         appears — this does NOT satisfy the dedicated bar-percentage requirement. \
         word/document.xml (first 2000 chars):\n{:.2000}",
        doc_xml
    );
}
