//! `PdfExporter` — implements the [`Exporter`] plugin trait for PDF output.
//!
//! ## REAL trait signature (reconciled from codebase inspection)
//!
//! The [`Exporter`] trait in `slideforge-plugin-api` has a DIFFERENT signature
//! than what the story spec described. The REAL signature is:
//!
//! ```text
//! fn export(
//!     &self,
//!     deck: &Deck,
//!     laid_out: &LaidOutDeck,
//!     brand: &Brand,
//!     opts: &ExportOptions,
//! ) -> Result<Vec<u8>, ExportError>;
//! ```
//!
//! Divergences from the spec's claimed signature:
//! - Returns `Result<Vec<u8>, ExportError>` — NOT `Result<(), ExportError>`.
//! - Takes `deck: &Deck` (semantic IR) AND `laid_out: &LaidOutDeck` (geometric IR).
//! - Takes `opts: &ExportOptions` — NOT `output: &mut dyn Write`.
//! - `ExportError` comes from `slideforge_plugin_api` — NOT `PdfExportError`.
//!   The exporter maps `PdfExportError` → `ExportError::RenderError` at the
//!   trait boundary.
//!
//! ## Architecture (BC-4.03.002 + STORY-044)
//!
//! `PdfExporter` is an effectful shell (ARCH-INDEX SS-07). It:
//! 1. Creates a `krilla::Document::new()`.
//! 2. Iterates over `laid_out.slides`, calling `SlideTagEngine::tag_slide`
//!    for each slide to build its structural tag sub-tree.
//! 3. Draws slide content via krilla's Surface API:
//!    - Text frames (Title/Subtitle/Body/TextRun): `surface.draw_text()` at
//!      coordinates from `coords::emu_to_pt()` / `coords::ir_y_to_pdf_y()`.
//!    - Diagram frames: `svg_embed::embed_normalized_svg()`.
//! 4. Calls `document.set_tag_tree(tag_tree)` with the assembled structural tree.
//! 5. Calls `document.finish()` → `KrillaResult<Vec<u8>>`.
//! 6. Maps `KrillaError` → `PdfExportError::Serialize` → `ExportError::RenderError`.
//! 7. Returns the PDF bytes.
//!
//! No subprocess is spawned. No FFI to C libraries. Pure Rust.
//!
//! ## EMU coordinate policy (BC-4.03.005 / Architecture Compliance Rule 2)
//!
//! ALL EMU-to-point conversions go through `coords::emu_to_pt()`. No inline
//! `emu / 12700` arithmetic is permitted anywhere in this file. This invariant
//! enables the Kani proof for `emu_to_pt` (VP-006, Phase 6) to cover all
//! conversion sites.

use krilla::Document;
use krilla::geom::Point;
use krilla::page::PageSettings;
use krilla::text::TextDirection;
use slideforge_layout::LaidOutDeck;
use slideforge_layout::types::{BoundingBox, FrameContent};
use slideforge_plugin_api::{ExportError, ExportOptions, Exporter};
use slideforge_types::{Brand, Deck, Emu};

use crate::coords::{emu_to_pt, ir_y_to_pdf_y};
use crate::error::PdfExportError;
use crate::font::load_font_data;
use crate::svg_embed::embed_normalized_svg;
use crate::tag_engine::SlideTagEngine;

/// PDF exporter implementing the [`Exporter`] plugin trait.
///
/// Produces a tagged PDF from a [`LaidOutDeck`] using `krilla 0.6.0` as the
/// primary PDF engine and [`SlideTagEngine`] for the PDF/UA-1 structure tree.
///
/// ## Thread safety
///
/// `PdfExporter` is `Send + Sync` (no interior mutability, no thread-local
/// state). Multiple concurrent export operations on independent decks are safe.
pub struct PdfExporter;

impl PdfExporter {
    /// Construct a new `PdfExporter`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Core PDF generation logic — called from [`Exporter::export`].
    ///
    /// Returns raw PDF bytes on success. `&self` is included for future use
    /// when `PdfExporter` carries font caches or configuration.
    ///
    /// ## Drawing pass (STORY-044)
    ///
    /// For each slide, this function draws:
    /// - Text frames (Title/Subtitle/Body/TextRun): `surface.draw_text()` at
    ///   coordinates computed via `coords::emu_to_pt()` and `coords::ir_y_to_pdf_y()`.
    ///   Font is resolved from brand family name via `font::system_font_fallback()`
    ///   then `krilla::text::Font::new()`. If no font can be resolved, text drawing
    ///   is skipped with a `tracing::warn!` — the page still renders.
    /// - Diagram frames: `svg_embed::embed_normalized_svg()` at the mapped position.
    ///
    /// ## Coordinate invariant (BC-4.03.005 / Architecture Compliance Rule 2)
    ///
    /// ALL EMU-to-point conversions go through `coords::emu_to_pt()` and
    /// `coords::ir_y_to_pdf_y()`. No inline `emu / 12700` arithmetic is used.
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError`] on:
    /// - Invalid page dimensions.
    /// - Tag tree assembly failure.
    /// - SVG embed failure.
    /// - Document serialization failure.
    ///
    /// Note: Font resolution failures are non-fatal — text is skipped with a
    /// structured warning, and the export continues. This ensures a partial PDF
    /// (without text) is returned rather than an export failure when a brand font
    /// is unavailable in the current environment.
    #[allow(clippy::unused_self)]
    fn generate_pdf(
        &self,
        _deck: &Deck,
        laid_out: &LaidOutDeck,
        brand: &Brand,
        _opts: &ExportOptions,
    ) -> Result<Vec<u8>, PdfExportError> {
        // Create a krilla Document with default settings.
        // Default SerializeSettings has enable_tagging: true.
        let mut document = Document::new();

        // Instantiate the tag engine — one per export pass, stateless per slide.
        let tag_engine = SlideTagEngine::new();

        // Resolve a font for text drawing. Source order:
        //   1. brand.fonts.heading — resolved via system_font_fallback()
        //   2. brand.fonts.body   — fallback if heading not found
        //
        // If neither resolves, `resolved_font` is None and text frames are
        // skipped with a tracing::warn! (graceful degradation — export still
        // produces a PDF without text rather than failing).
        let resolved_font = resolve_brand_font(brand);

        // Collect per-slide Part groups for later assembly into the deck tag tree.
        let mut slide_parts = Vec::with_capacity(laid_out.slides.len());

        // Slide height in EMU for ir_y_to_pdf_y — taken from the deck's page size.
        let slide_h_emu = laid_out.page_size.height;

        for slide in &laid_out.slides {
            // Convert page dimensions via coords:: — Architecture Compliance Rule 2.
            let width_pts = emu_to_pt(laid_out.page_size.width);
            let height_pts = emu_to_pt(laid_out.page_size.height);

            let page_settings = PageSettings::from_wh(width_pts, height_pts).ok_or_else(|| {
                PdfExportError::Serialize {
                    message: format!(
                        "invalid page size: {width_pts}pt x {height_pts}pt (slide {})",
                        slide.source_index
                    ),
                }
            })?;

            let mut page = document.start_page_with(page_settings);

            // Build the structural tag sub-tree for this slide.
            let part_result = tag_engine.tag_slide(slide)?;

            // Obtain the krilla Surface and draw slide content.
            let mut surface = page.surface();

            for frame in &slide.frames {
                draw_frame(
                    &mut surface,
                    &frame.bbox,
                    &frame.content,
                    slide_h_emu,
                    resolved_font.as_ref(),
                )?;
            }

            surface.finish();
            page.finish();

            // Collect the Part group for deck-level tree assembly.
            slide_parts.push(part_result.part);
        }

        // Assemble the per-slide Part groups into a single deck-level TagTree
        // and attach it to the document before finish().
        //
        // BC-4.03.002 AC-003: set_tag_tree called before document.finish().
        // STORY-043 scope-directive Decision 1: this wiring must happen NOW.
        let tag_tree = tag_engine.assemble_deck_tag_tree(slide_parts)?;
        document.set_tag_tree(tag_tree);

        // Serialize to PDF bytes. `Document::finish()` returns
        // `KrillaResult<Vec<u8>>` (i.e. `Result<Vec<u8>, KrillaError>`).
        document.finish().map_err(|e| PdfExportError::Serialize {
            message: format!("krilla serialization error: {e:?}"),
        })
    }
}

impl Default for PdfExporter {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Font resolution ──────────────────────────────────────────────────────────

/// Resolve a krilla font for text drawing from the brand font configuration.
///
/// Tries `brand.fonts.heading` first, then `brand.fonts.body`. For each, calls
/// [`crate::font::system_font_fallback`] to find a font file by family name,
/// then `load_font_data` to read the bytes, then `krilla::text::Font::new`.
///
/// Returns `None` if no font can be resolved (missing system font or unreadable
/// file). Callers MUST skip text drawing when `None` is returned rather than
/// failing the export — font unavailability is non-fatal.
fn resolve_brand_font(brand: &Brand) -> Option<krilla::text::Font> {
    let candidates = [brand.fonts.heading.as_ref(), brand.fonts.body.as_ref()];
    for family in &candidates {
        if let Some(font) = try_resolve_font(family) {
            return Some(font);
        }
    }
    tracing::warn!(
        heading = %brand.fonts.heading,
        body = %brand.fonts.body,
        "brand font families not found on this system — text drawing will be skipped; \
         PDF will contain structural content but no visible text"
    );
    None
}

/// Attempt to resolve a single font family name to a `krilla::text::Font`.
///
/// Returns `None` on any failure (family not found, file unreadable, invalid
/// font data). All failures are logged at `tracing::debug!` level for
/// diagnostics without exposing internal paths to callers (SEC-005).
fn try_resolve_font(family: &str) -> Option<krilla::text::Font> {
    use crate::font::system_font_fallback;

    let path = system_font_fallback(family)?;
    let bytes = load_font_data(&path)
        .map_err(|e| {
            tracing::debug!(family, path = %path.display(), error = %e, "font file load failed");
        })
        .ok()?;
    let data: krilla::Data = bytes.into();
    let font = krilla::text::Font::new(data, 0);
    if font.is_none() {
        tracing::debug!(
            family,
            "krilla::text::Font::new returned None for font file"
        );
    }
    font
}

// ─── Frame content drawing ────────────────────────────────────────────────────

/// Draw the content of a single layout frame onto a krilla `Surface`.
///
/// All coordinate conversions go through `coords::emu_to_pt()` and
/// `coords::ir_y_to_pdf_y()` (BC-4.03.005 Architecture Compliance Rule 2).
///
/// ## Text baseline approximation
///
/// PDF text coordinates are specified at the **baseline** of the first line of
/// text. The IR gives the top-left corner of the bounding box. A reasonable
/// baseline approximation for a single-line draw is:
///
/// ```text
/// baseline_y ≈ ir_y_to_pdf_y(ir_y, element_h, slide_h) + element_h_pt * 0.8
/// ```
///
/// This places the baseline at approximately 80% of the box height from the
/// PDF bottom of the box (i.e., 20% descender allowance below the text). Text
/// is guaranteed to land within `[0, SLIDE_HEIGHT_PT]` for any valid IR layout.
///
/// Precise multi-line typography is deferred to STORY-045 (text flow engine).
///
/// ## Font sizes
///
/// Default font sizes: Title 36pt, Subtitle 28pt, Body/other 18pt.
/// These defaults are overridden when brand template font sizes are available
/// (STORY-045 scope).
///
/// # Errors
///
/// Returns [`PdfExportError::SvgEmbed`] for diagram frame failures. All other
/// frame types silently skip on failure (font unavailable, empty content, etc.).
fn draw_frame(
    surface: &mut krilla::surface::Surface<'_>,
    bbox: &BoundingBox,
    content: &FrameContent,
    slide_h_emu: Emu,
    font: Option<&krilla::text::Font>,
) -> Result<(), PdfExportError> {
    match content {
        FrameContent::Title(text) => {
            draw_text_at_bbox(surface, text, bbox, slide_h_emu, 36.0, font);
        },
        FrameContent::Subtitle(text) => {
            draw_text_at_bbox(surface, text, bbox, slide_h_emu, 28.0, font);
        },
        FrameContent::Body(blocks) => {
            draw_body_blocks(surface, blocks, bbox, slide_h_emu, font);
        },
        FrameContent::TextRun(inlines) => {
            let text = extract_inline_text(inlines);
            if !text.is_empty() {
                draw_text_at_bbox(surface, &text, bbox, slide_h_emu, 18.0, font);
            }
        },
        FrameContent::Diagram(svg) => {
            // Place the SVG at the frame's PDF coordinates.
            // SVG content is drawn at the PDF-mapped position using a translate
            // transform so paths land within the frame's bounding box.
            let pdf_x = emu_to_pt(bbox.x);
            let pdf_y = ir_y_to_pdf_y(bbox.y, bbox.height, slide_h_emu);
            place_svg_at(surface, svg, pdf_x, pdf_y)?;
        },
        // ErrorSlidePlaceholder carries an SVG — render it like a diagram.
        FrameContent::ErrorSlidePlaceholder { svg, .. } => {
            use slideforge_types::NormalizedDiagramSvg;
            use std::sync::Arc;
            let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(svg.as_ref()));
            let pdf_x = emu_to_pt(bbox.x);
            let pdf_y = ir_y_to_pdf_y(bbox.y, bbox.height, slide_h_emu);
            place_svg_at(surface, &normalized, pdf_x, pdf_y)?;
        },
        // Chart: no SVG payload at frame level — drawn via ChartRenderer pass.
        // Image, Shape, Empty: no drawing in this story.
        FrameContent::Chart
        | FrameContent::Image { .. }
        | FrameContent::Shape(_)
        | FrameContent::Empty => {},
    }
    Ok(())
}

/// Draw text at a bounding box position using PDF coordinate mapping.
///
/// If `font` is `None`, logs a debug warning and skips drawing. This is the
/// correct non-fatal behavior when a brand font is unavailable.
fn draw_text_at_bbox(
    surface: &mut krilla::surface::Surface<'_>,
    text: &str,
    bbox: &BoundingBox,
    slide_h_emu: Emu,
    font_size: f32,
    font: Option<&krilla::text::Font>,
) {
    let Some(font) = font else {
        tracing::debug!(
            text_preview = &text[..text.len().min(20)],
            "skipping text draw: no resolved font"
        );
        return;
    };
    if text.is_empty() {
        return;
    }

    // PDF X: left edge of the bounding box.
    let pdf_x = emu_to_pt(bbox.x);

    // PDF Y baseline: bottom of the bounding box in PDF coords + 80% of height
    // as the baseline approximation (20% descender allowance).
    let box_bottom_pdf_y = ir_y_to_pdf_y(bbox.y, bbox.height, slide_h_emu);
    let element_h_pt = emu_to_pt(bbox.height);
    let baseline_y = box_bottom_pdf_y + element_h_pt * 0.8;

    let start = Point::from_xy(pdf_x, baseline_y);
    surface.draw_text(
        start,
        font.clone(),
        font_size,
        text,
        false,
        TextDirection::Auto,
    );
}

/// Draw body content blocks at the given bounding box.
///
/// Iterates text-bearing blocks (Text, Bullets) and draws their inline text.
/// Non-text blocks (Chart, Diagram, Math, Image, Table, Shape) are skipped in
/// this pass — they are handled separately in their own frame draw logic.
fn draw_body_blocks(
    surface: &mut krilla::surface::Surface<'_>,
    blocks: &[slideforge_types::ContentBlock],
    bbox: &BoundingBox,
    slide_h_emu: Emu,
    font: Option<&krilla::text::Font>,
) {
    use slideforge_types::ContentBlock;

    for block in blocks {
        match block {
            ContentBlock::Text(text_block) => {
                let text = extract_inline_text(&text_block.inlines);
                if !text.is_empty() {
                    draw_text_at_bbox(surface, &text, bbox, slide_h_emu, 18.0, font);
                }
            },
            ContentBlock::Bullets(items) => {
                for item in items {
                    let text = extract_inline_text(&item.inlines);
                    if !text.is_empty() {
                        draw_text_at_bbox(surface, &text, bbox, slide_h_emu, 16.0, font);
                    }
                }
            },
            // Other block types are not drawn in this pass.
            ContentBlock::Chart(_)
            | ContentBlock::Diagram(_)
            | ContentBlock::Math(_)
            | ContentBlock::Image(_)
            | ContentBlock::Table(_)
            | ContentBlock::Shape(_) => {},
        }
    }
}

/// Extract a flat plain-text string from a sequence of [`InlineNode`]s.
///
/// Traverses `Bold` and `Italic` nodes recursively to collect all
/// [`InlineNode::Plain`] leaf text. Other inline variants (Code, Xref, etc.)
/// are skipped in this story — they contribute to the text stream in STORY-045.
fn extract_inline_text(inlines: &[slideforge_types::InlineNode]) -> String {
    use slideforge_types::InlineNode;

    let mut out = String::new();
    for node in inlines {
        match node {
            InlineNode::Plain(s) => out.push_str(s),
            InlineNode::Bold(children) | InlineNode::Italic(children) => {
                out.push_str(&extract_inline_text(children));
            },
            // Other variants (Code, Xref, Math, etc.) deferred to STORY-045.
            _ => {},
        }
    }
    out
}

/// Place a normalized SVG diagram on the surface at the specified PDF position.
///
/// Applies a translation transform so the SVG paths land at `(pdf_x, pdf_y)`.
/// The transform is pushed before embedding and popped after, leaving the
/// surface state unchanged for subsequent draw calls.
fn place_svg_at(
    surface: &mut krilla::surface::Surface<'_>,
    svg: &slideforge_types::NormalizedDiagramSvg,
    pdf_x: f32,
    pdf_y: f32,
) -> Result<(), PdfExportError> {
    // Apply a translation so the SVG is positioned at the frame's PDF coords.
    let transform = krilla::geom::Transform::from_translate(pdf_x, pdf_y);
    surface.push_transform(&transform);
    let result = embed_normalized_svg(svg, surface);
    surface.pop();
    result
}

impl Exporter for PdfExporter {
    // Trait requires `&str`; return type is tied to `&self` per trait contract
    // even though we return `'static` literals. Suppressing the unnecessary_literal_bound
    // lint because the trait signature (not ours to change) imposes the `&self` lifetime.
    #[allow(clippy::unnecessary_literal_bound)]
    fn id(&self) -> &str {
        "pdf"
    }

    #[allow(clippy::unnecessary_literal_bound)]
    fn extension(&self) -> &str {
        "pdf"
    }

    /// Produce PDF bytes from the slideforge IR.
    ///
    /// # Parameters
    ///
    /// - `deck` — the semantic, pre-layout IR (used for metadata: title, lang)
    /// - `laid_out` — the geometric, post-layout IR (slide frames, coordinates)
    /// - `brand` — resolved brand configuration (fonts, palette, page size)
    /// - `opts` — per-export options
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::RenderError`] for all PDF generation failures,
    /// wrapping the underlying [`PdfExportError`] message.
    fn export(
        &self,
        deck: &Deck,
        laid_out: &LaidOutDeck,
        brand: &Brand,
        opts: &ExportOptions,
    ) -> Result<Vec<u8>, ExportError> {
        self.generate_pdf(deck, laid_out, brand, opts)
            .map_err(|e| ExportError::RenderError {
                message: e.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slideforge_layout::types::{
        BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize, RegisterSet,
    };
    use slideforge_types::{Brand, BrandFonts, BrandPalette, Deck, Emu, SourceSpan};
    use std::sync::Arc;

    /// Build a minimal 1-slide `LaidOutDeck` for export tests.
    fn minimal_laid_out_deck() -> LaidOutDeck {
        LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![LaidOutSlide {
                source_index: 0,
                slide_type_keyword: Arc::from("title"),
                frames: vec![Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(0),
                        width: Emu(9_144_000),
                        height: Emu(914_400),
                    },
                    content: FrameContent::Title(Arc::from("Red Gate Test")),
                    text_flow: None,
                }],
                speaker_notes: None,
                register_tags: RegisterSet::new(),
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
        }
    }

    /// Build a 1-slide `LaidOutDeck` with a Title frame + an Image figure frame.
    ///
    /// Used by `test_bc_4_03_002_export_produces_tagged_pdf` to exercise the
    /// tag tree in a non-vacuous way: the resulting PDF must contain structural
    /// element markers for both `/Part` (the slide) and `/H1` (the title heading)
    /// and `/Figure` (the image frame with alt text).
    fn title_and_figure_laid_out_deck() -> LaidOutDeck {
        LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![LaidOutSlide {
                source_index: 0,
                slide_type_keyword: Arc::from("photo"),
                frames: vec![
                    Frame {
                        bbox: BoundingBox {
                            x: Emu(0),
                            y: Emu(0),
                            width: Emu(9_144_000),
                            height: Emu(914_400),
                        },
                        content: FrameContent::Title(Arc::from("Sunrise Over Mountains")),
                        text_flow: None,
                    },
                    Frame {
                        bbox: BoundingBox {
                            x: Emu(0),
                            y: Emu(914_400),
                            width: Emu(9_144_000),
                            height: Emu(5_143_500),
                        },
                        content: FrameContent::Image {
                            alt: Arc::from("A mountain landscape at sunrise"),
                        },
                        text_flow: None,
                    },
                ],
                speaker_notes: None,
                register_tags: RegisterSet::new(),
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
        }
    }

    /// Build a minimal `Brand` for export tests.
    fn minimal_brand() -> Brand {
        Brand {
            name: Arc::from("TestBrand"),
            palette: BrandPalette {
                primary: Arc::from("#003087"),
                secondary: Arc::from("#FFFFFF"),
                accent: Arc::from("#F5A623"),
                neutral: Arc::from("#F0F0F0"),
            },
            fonts: BrandFonts {
                heading: Arc::from("Helvetica"),
                body: Arc::from("Helvetica"),
                mono: Arc::from("Courier"),
            },
            layouts: vec![],
            span: SourceSpan::default(),
        }
    }

    /// Build a minimal semantic `Deck` for export tests.
    fn minimal_deck() -> Deck {
        use slideforge_types::{DeckMetadata, OrderedMap};
        Deck {
            slides: vec![],
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

    /// BC-4.03.002 AC-001: `PdfExporter` implements the `Exporter` plugin trait.
    ///
    /// This compile-time test verifies the impl exists and `PdfExporter` satisfies
    /// the `Exporter`, `Send`, and `Sync` bounds.
    #[test]
    fn test_bc_4_03_002_pdf_exporter_implements_exporter_trait() {
        fn assert_exporter<T: Exporter + Send + Sync>() {}
        assert_exporter::<PdfExporter>();
    }

    /// BC-4.03.002 AC-001 (behavioral): `PdfExporter::export()` on a minimal
    /// 1-slide `LaidOutDeck` produces non-empty bytes starting with `%PDF-`.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_bc_4_03_002_export_produces_pdf_bytes() {
        let exporter = PdfExporter::new();
        let deck = minimal_deck();
        let laid_out = minimal_laid_out_deck();
        let brand = minimal_brand();
        let opts = ExportOptions::default();

        let result = exporter.export(&deck, &laid_out, &brand, &opts);

        assert!(
            result.is_ok(),
            "PdfExporter::export must succeed for a minimal 1-slide deck: {:?}",
            result.err()
        );
        let bytes = result.unwrap();
        assert!(!bytes.is_empty(), "PDF output must not be empty");
        assert!(
            bytes.starts_with(b"%PDF-"),
            "PDF output must start with the %PDF- magic header; got prefix: {:?}",
            &bytes[..bytes.len().min(8)]
        );
    }

    /// BC-4.03.002 AC-001 (id/extension): `PdfExporter::id()` == `"pdf"` and
    /// `PdfExporter::extension()` == `"pdf"`.
    #[test]
    fn test_bc_4_03_002_exporter_id_and_extension() {
        let exporter = PdfExporter::new();
        assert_eq!(exporter.id(), "pdf");
        assert_eq!(exporter.extension(), "pdf");
    }

    /// BC-4.03.002 AC-003 (integration): `PdfExporter::export()` produces a
    /// tagged PDF with a non-vacuous structure tree.
    ///
    /// Uses a deck with a Title frame (→ `/H1` `StructElem`) and an Image frame
    /// with alt text (→ `/Figure` `StructElem`) to verify that:
    ///
    /// 1. `StructTreeRoot` is present (`set_tag_tree` was called).
    /// 2. `/Part` `StructElem` is present (the slide Part group was emitted).
    /// 3. `/H1` `StructElem` is present (the title frame produced a heading).
    /// 4. `/Figure` `StructElem` is present (the image frame with alt text produced
    ///    a figure element).
    /// 5. `/Document` root `StructElem` is present — krilla auto-emits this via
    ///    `TagTree::serialize()` calling `struct_elem.kind(StructRole::Document)`.
    ///    Closes adversary finding F-P4-004 non-vacuously.
    ///
    /// These are the exact bytes krilla 0.6.0 writes for the corresponding
    /// `TagKind` variants (verified against krilla source and live output).
    /// An empty-but-present tree would fail assertions 2–4 — this test is
    /// non-vacuous.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_bc_4_03_002_export_produces_tagged_pdf() {
        let exporter = PdfExporter::new();
        let deck = minimal_deck();
        // Use a title+figure deck so the tag tree assertions are non-vacuous:
        // an empty (but present) StructTreeRoot would pass assertion 1 but fail 2–4.
        let laid_out = title_and_figure_laid_out_deck();
        let brand = minimal_brand();
        let opts = ExportOptions::default();

        let bytes = exporter.export(&deck, &laid_out, &brand, &opts).unwrap();

        // 1. StructTreeRoot must be present — confirms set_tag_tree was called.
        let has_struct_tree_root = bytes
            .windows(b"StructTreeRoot".len())
            .any(|w| w == b"StructTreeRoot");
        assert!(
            has_struct_tree_root,
            "exported PDF must contain StructTreeRoot (tagged PDF marker); \
             this confirms set_tag_tree was called before document.finish()"
        );

        // 2. /Part StructElem must be present — confirms the slide Part group was emitted.
        // krilla 0.6.0 writes `/S /Part` for TagKind::Part.
        let has_part = bytes.windows(b"/Part".len()).any(|w| w == b"/Part");
        assert!(
            has_part,
            "exported PDF must contain /Part StructElem; \
             confirms the slide Part TagGroup was attached to the tag tree"
        );

        // 3. /H1 StructElem must be present — confirms the Title frame → Hn(1) mapping.
        // krilla 0.6.0 writes `/S /H1` for TagKind::Hn with level 1.
        let has_h1 = bytes.windows(b"/H1".len()).any(|w| w == b"/H1");
        assert!(
            has_h1,
            "exported PDF must contain /H1 StructElem; \
             confirms the Title frame was tagged as Hn(level=1)"
        );

        // 4. /Figure StructElem must be present — confirms the Image frame → Figure mapping.
        // krilla 0.6.0 writes `/S /Figure` for TagKind::Figure.
        let has_figure = bytes.windows(b"/Figure".len()).any(|w| w == b"/Figure");
        assert!(
            has_figure,
            "exported PDF must contain /Figure StructElem; \
             confirms the Image frame (with alt text) was tagged as Figure"
        );

        // 5. /Document root StructElem must be present — confirms krilla auto-emits
        // the PDF/UA-1 Document root structure element (F-P4-004 non-vacuous closure).
        //
        // krilla 0.6.0 `TagTree::serialize()` always calls
        // `struct_elem.kind(StructRole::Document)` on the root element before
        // `document.finish()`, writing `/S /Document` into the PDF stream.
        // `StructRole::Document` serializes to `Name(b"Document")` in pdf-writer,
        // producing the literal bytes `/Document` in the output file.
        //
        // Sources:
        //   - krilla 0.6.0 src/interchange/tagging/mod.rs:1050
        //     `struct_elem.kind(StructRole::Document);`
        //   - pdf-writer 0.14.0 src/structure.rs:877
        //     `Self::Document => Name(b"Document")`
        //
        // This assertion closes adversary finding F-P4-004 non-vacuously:
        // the spec-required /Document IS in the output, emitted automatically
        // by krilla's TagTree, not by slideforge production code.
        let has_document_root = bytes.windows(b"/Document".len()).any(|w| w == b"/Document");
        assert!(
            has_document_root,
            "exported PDF must contain /Document root StructElem \
             (krilla auto-emits StructRole::Document via TagTree::serialize); \
             this confirms the PDF/UA-1 Document-root invariant (F-P4-004)"
        );
    }

    /// BC-4.03.002 AC-008 (structural no-subprocess check): `PdfExporter::export`
    /// is a pure Rust call graph — it must not spawn subprocesses.
    ///
    /// ## Load-bearing assertion (F-006 fix)
    ///
    /// The REAL enforcement is a source-level grep in `scripts/check-pdf-deps.sh`:
    ///
    /// ```bash
    /// # Assert no std::process usage in the export path
    /// if grep -r "std::process\|Command::new\|process::Command" \
    ///     crates/slideforge-pdf/src/; then
    ///     echo "FAIL: subprocess usage found" >&2
    ///     exit 1
    /// fi
    /// ```
    ///
    /// Run `bash scripts/check-pdf-deps.sh` to execute this check in CI.
    /// That is the load-bearing assertion for AC-008 (F-006 fix, scope-directive
    /// Decision 3).
    ///
    /// ## Deferred strace/dtrace integration test
    ///
    /// Full strace/dtrace integration test is deferred to STORY-049:
    ///   test name: `test_e2e_pdf_export_no_execve_syscall`
    ///   Reason: requires full CLI binary + syscall tracer (strace/dtrace/procmon).
    ///   Per SID-1: STORY-049 explicitly named, deferred by concrete dependency
    ///   (CLI binary + platform tracing tool not available in unit test context).
    ///
    /// The `Send + Sync` assertions below are supplementary structural coverage
    /// and do NOT prove absence of subprocess spawning on their own.
    #[test]
    fn test_bc_4_03_002_no_subprocess_structural_check() {
        // Structural: PdfExporter is Send + Sync, which rules out holding OS
        // handles that would indicate subprocess state.
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PdfExporter>();

        // Behavioral: `id()` returns "pdf" — not a browser binary name.
        let exporter = PdfExporter::new();
        let id = exporter.id();
        assert_ne!(id, "chrome", "exporter ID must not be a browser name");
        assert_ne!(id, "chromium", "exporter ID must not be a browser name");
        assert_eq!(id, "pdf", "exporter ID must be 'pdf'");
    }
}
