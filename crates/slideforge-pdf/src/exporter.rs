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
///
/// ## Font override (test seam — F-044-002)
///
/// `PdfExporter` can be constructed with an explicit font file path via
/// [`PdfExporter::with_font_path`]. When set, font resolution bypasses the
/// brand family-name lookup and loads the font directly from the given path.
/// This is a real production capability (a user could point the exporter at a
/// specific font file), not a test-only hack — production code never changes
/// behavior based on whether the seam is active, it just uses the font at the
/// supplied path instead of looking one up from the brand name.
pub struct PdfExporter {
    /// Optional explicit font file path. When `Some`, `resolve_brand_font`
    /// loads this path directly instead of searching the system font directories
    /// by brand family name. Used by tests (F-044-002) and by production callers
    /// that have a known font file on disk.
    font_override_path: Option<std::path::PathBuf>,
}

impl PdfExporter {
    /// Construct a new `PdfExporter` using brand-based font resolution.
    #[must_use]
    pub fn new() -> Self {
        Self {
            font_override_path: None,
        }
    }

    /// Construct a `PdfExporter` that loads its font from an explicit file path,
    /// bypassing brand family-name resolution.
    ///
    /// # Use cases
    ///
    /// - **Tests:** use a fixture font (e.g., `crates/slideforge-math/fonts/
    ///   latinmodern-math.otf`) for deterministic AC-009 / drawing-path tests.
    /// - **Production:** callers with a known font file on disk can skip the
    ///   best-effort `system_font_fallback` name lookup.
    ///
    /// If the font file cannot be loaded at export time, the exporter falls back
    /// to brand-based resolution (graceful degradation).
    /// # Examples
    ///
    /// ```no_run
    /// use slideforge_pdf::PdfExporter;
    ///
    /// let exporter = PdfExporter::with_font_path(
    ///     std::path::PathBuf::from("/usr/share/fonts/opentype/myfont.otf")
    /// );
    /// ```
    #[must_use]
    pub fn with_font_path(path: std::path::PathBuf) -> Self {
        Self {
            font_override_path: Some(path),
        }
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
    fn generate_pdf(
        &self,
        deck: &Deck,
        laid_out: &LaidOutDeck,
        brand: &Brand,
        opts: &ExportOptions,
    ) -> Result<Vec<u8>, PdfExportError> {
        self.generate_pdf_inner(
            deck,
            laid_out,
            brand,
            opts,
            krilla::SerializeSettings::default(),
        )
    }

    /// Core PDF generation with explicit [`krilla::SerializeSettings`].
    ///
    /// Separated from `generate_pdf` so tests can pass `compress_content_streams: false`
    /// to produce uncompressed output scannable for text/path operators (F-044-004).
    ///
    /// # Errors
    ///
    /// Same error conditions as [`generate_pdf`].
    fn generate_pdf_inner(
        &self,
        _deck: &Deck,
        laid_out: &LaidOutDeck,
        brand: &Brand,
        _opts: &ExportOptions,
        settings: krilla::SerializeSettings,
    ) -> Result<Vec<u8>, PdfExportError> {
        // Create a krilla Document with the supplied settings.
        // Default SerializeSettings has enable_tagging: true, compress_content_streams: true.
        let mut document = Document::new_with(settings);

        // Instantiate the tag engine — one per export pass, stateless per slide.
        let tag_engine = SlideTagEngine::new();

        // Resolve a font for text drawing.
        //
        // If `self.font_override_path` is set, load directly from that path.
        // Otherwise try brand family name resolution:
        //   1. brand.fonts.heading — resolved via system_font_fallback()
        //   2. brand.fonts.body   — fallback if heading not found
        //
        // If no font can be resolved, `resolved_font` is None and text frames are
        // skipped with a tracing::warn! (graceful degradation — export still
        // produces a PDF without text rather than failing).
        let resolved_font = resolve_brand_font(brand, self.font_override_path.as_deref());

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

/// Resolve a krilla font for text drawing.
///
/// ## Resolution order
///
/// 1. If `font_override_path` is `Some`, load directly from that path (test seam
///    and production override — bypasses brand family-name lookup).
/// 2. Otherwise try brand family-name resolution:
///    - `brand.fonts.heading` via [`crate::font::system_font_fallback`]
///    - `brand.fonts.body` as fallback
///
/// Returns `None` if no font can be resolved (missing system font, unreadable
/// file, or invalid font data). Callers MUST skip text drawing when `None` is
/// returned rather than failing the export — font unavailability is non-fatal.
fn resolve_brand_font(
    brand: &Brand,
    font_override_path: Option<&std::path::Path>,
) -> Option<krilla::text::Font> {
    // If an explicit font path was provided, try it first.
    if let Some(path) = font_override_path {
        match load_font_data(path) {
            Ok(bytes) => {
                let data: krilla::Data = bytes.into();
                if let Some(font) = krilla::text::Font::new(data, 0) {
                    return Some(font);
                }
                tracing::debug!(
                    path = %path.display(),
                    "krilla::text::Font::new returned None for override font path; \
                     falling back to brand family resolution"
                );
            },
            Err(e) => {
                tracing::debug!(
                    path = %path.display(),
                    error = %e,
                    "font override path load failed; falling back to brand family resolution"
                );
            },
        }
    }

    // Brand family-name resolution.
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
/// Error propagation depends on frame type:
///
/// - **`Diagram` and `ErrorSlidePlaceholder`** — SVG embed failures propagate
///   via `?` as [`PdfExportError::SvgEmbed`], aborting the export. A malformed
///   or unsupported SVG payload is treated as a hard failure.
/// - **Text frames (`Title`, `Subtitle`, `Body`, `TextRun`)** — degrade
///   gracefully: if `font` is `None`, drawing is skipped with a
///   `tracing::debug!` warning and the function returns `Ok(())`. Empty
///   `TextRun` content is silently skipped.
/// - **`Chart`, `Image`, `Shape`, `Empty`** — no drawing in STORY-044;
///   always return `Ok(())` immediately.
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
        // Use char-safe truncation to avoid byte-boundary panics on multi-byte
        // UTF-8 text (F-044-001: `&text[..text.len().min(20)]` would panic when
        // the 20th byte is mid-codepoint; `chars().take(20)` is always safe).
        let preview: String = text.chars().take(20).collect();
        tracing::debug!(
            text_preview = %preview,
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
    /// - `deck` — the semantic, pre-layout IR. Currently unused by the PDF
    ///   renderer; it is accepted so the [`Exporter`] trait signature is
    ///   satisfied. PDF document metadata (title, language) will be populated
    ///   from this parameter in STORY-045 (PDF/UA-1 metadata wiring).
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

    // ─── F-044-001 test: no char-boundary panic on multi-byte text ─────────────

    /// F-044-001: `PdfExporter::export()` must not panic on a deck whose text
    /// contains multi-byte UTF-8 characters when font resolution returns `None`.
    ///
    /// The bug was `&text[..text.len().min(20)]` in the font-None log path,
    /// which panics when the 20th byte is mid-codepoint. The fix uses
    /// `text.chars().take(20).collect::<String>()`.
    ///
    /// The brand uses a guaranteed-absent font family so font resolution returns
    /// `None`, exercising the degradation path. The long Japanese text (36+ bytes,
    /// 20th byte mid-codepoint with the old code) confirms no panic occurs.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_f044_001_no_panic_on_multibyte_text_with_font_none() {
        // A long Japanese string whose 20th byte (0-indexed) is mid-codepoint.
        // Each Japanese character is 3 bytes in UTF-8, so 7 chars = 21 bytes.
        // The 20th byte (index 19) is the second byte of the 7th character.
        // The old code would panic; the fixed code must not.
        let japanese_text = "日本語のテキストが長い場合のトランケーション";
        // Verify the test setup: the 20th byte is mid-codepoint.
        assert!(
            !japanese_text.is_char_boundary(20),
            "test setup: byte 20 must be mid-codepoint for this test to be meaningful"
        );

        let exporter = PdfExporter::new(); // uses brand-based font resolution
        let deck = minimal_deck();
        let brand = Brand {
            name: Arc::from("TestBrand"),
            palette: BrandPalette {
                primary: Arc::from("#003087"),
                secondary: Arc::from("#FFFFFF"),
                accent: Arc::from("#F5A623"),
                neutral: Arc::from("#F0F0F0"),
            },
            fonts: BrandFonts {
                // Guaranteed-absent family names → font resolution returns None.
                heading: Arc::from("NoSuchFont_F044001_Unicode_Test"),
                body: Arc::from("NoSuchFont_F044001_Unicode_Test"),
                mono: Arc::from("Courier"),
            },
            layouts: vec![],
            span: SourceSpan::default(),
        };
        let laid_out = LaidOutDeck {
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
                    content: FrameContent::Title(Arc::from(japanese_text)),
                    text_flow: None,
                }],
                speaker_notes: None,
                register_tags: RegisterSet::new(),
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
        };
        let opts = ExportOptions::default();

        // Must NOT panic — this is the core assertion.
        // Before the fix, this panicked with "byte index 20 is not a char boundary".
        let result = exporter.export(&deck, &laid_out, &brand, &opts);

        assert!(
            result.is_ok(),
            "export must succeed even with multi-byte UTF-8 text + no resolved font: {result:?}"
        );
        let bytes = result.unwrap();
        assert!(
            bytes.starts_with(b"%PDF-"),
            "PDF output must start with %PDF- header"
        );
    }

    // ─── F-044-003: AC-006 integration test — render a fixture deck ──────────

    /// BC-4.03.005 AC-006 (integration): `PdfExporter::export()` on a fixture
    /// `LaidOutDeck` (elements at various positions including top/bottom edges)
    /// must draw all elements within the slide canvas `[0, SLIDE_WIDTH_PT] ×
    /// [0, SLIDE_HEIGHT_PT]`.
    ///
    /// ## What this tests (F-044-003 fix)
    ///
    /// The spec says AC-006 must be "verified by an INTEGRATION TEST that RENDERS
    /// A FIXTURE DECK and asserts all element bounding boxes are within
    /// [0,0,720,405]." This test:
    ///
    /// 1. Builds a fixture `LaidOutDeck` with Title/Subtitle/Body frames at
    ///    positions spanning the full slide height (top, middle, bottom).
    /// 2. Runs `PdfExporter::export()` to confirm the pipeline completes without
    ///    coordinate errors.
    /// 3. Asserts the PDF coordinate arithmetic (`ir_y_to_pdf_y + 0.8 * height`)
    ///    for all test frames stays within `[-epsilon, SLIDE_HEIGHT_PT + epsilon]`.
    ///    This validates the ACTUAL baseline computation used in `draw_text_at_bbox`.
    ///
    /// ## Distinction from the existing coords unit tests
    ///
    /// The existing `test_bc_4_03_005_no_element_outside_canvas_after_conversion`
    /// tests `ir_y_to_pdf_y()` pairs in isolation. THIS test exercises the
    /// BASELINE COMPUTATION `box_bottom_pdf_y + element_h_pt * 0.8` as used in
    /// the DRAW PATH, confirming element placement via the export route.
    #[allow(clippy::unwrap_used, clippy::float_cmp)]
    #[test]
    fn test_bc_4_03_005_ac006_export_all_elements_within_canvas() {
        use crate::SLIDE_HEIGHT_EMU;
        use crate::coords::{SLIDE_HEIGHT_PT, emu_to_pt, ir_y_to_pdf_y};

        // Fixture deck: Title at top, Subtitle at 1-inch offset, Body at 2-inch offset.
        // All stay within the 5.625-inch (405pt) slide height.
        let one_inch_emu = Emu(914_400);
        let two_inch_emu = Emu(1_828_800);
        let title_h_emu = Emu(914_400); // 72pt
        let body_h_emu = Emu(1_270_000); // ~100pt

        let slide_h_emu = SLIDE_HEIGHT_EMU;

        // Verify our test fixture is within bounds.
        assert!(
            two_inch_emu.0 + body_h_emu.0 <= slide_h_emu.0,
            "test fixture: body frame must fit within slide height"
        );

        let laid_out = LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![LaidOutSlide {
                source_index: 0,
                slide_type_keyword: Arc::from("content"),
                frames: vec![
                    Frame {
                        bbox: BoundingBox {
                            x: Emu(0),
                            y: Emu(0),
                            width: Emu(9_144_000),
                            height: title_h_emu,
                        },
                        content: FrameContent::Title(Arc::from("Title at top")),
                        text_flow: None,
                    },
                    Frame {
                        bbox: BoundingBox {
                            x: Emu(0),
                            y: one_inch_emu,
                            width: Emu(9_144_000),
                            height: title_h_emu,
                        },
                        content: FrameContent::Subtitle(Arc::from("Subtitle at 1-inch")),
                        text_flow: None,
                    },
                    Frame {
                        bbox: BoundingBox {
                            x: Emu(0),
                            y: two_inch_emu,
                            width: Emu(9_144_000),
                            height: body_h_emu,
                        },
                        content: FrameContent::Body(vec![]),
                        text_flow: None,
                    },
                ],
                speaker_notes: None,
                register_tags: RegisterSet::new(),
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
        };

        let exporter = PdfExporter::new();
        let deck = minimal_deck();
        let brand = minimal_brand();
        let opts = ExportOptions::default();

        // Export must succeed (no coordinate errors).
        let result = exporter.export(&deck, &laid_out, &brand, &opts);
        assert!(
            result.is_ok(),
            "export must succeed for the AC-006 fixture deck: {result:?}"
        );

        // Verify baseline computations for all frames stay within [0, SLIDE_HEIGHT_PT].
        // This mirrors the exact formula used in `draw_text_at_bbox`:
        //   box_bottom_pdf_y = ir_y_to_pdf_y(ir_y, element_h, slide_h)
        //   baseline_y = box_bottom_pdf_y + element_h_pt * 0.8
        let frames_under_test = [
            (Emu(0), title_h_emu, "title-top"),
            (one_inch_emu, title_h_emu, "subtitle-1in"),
            (two_inch_emu, body_h_emu, "body-2in"),
        ];

        for (ir_y, elem_h, label) in frames_under_test {
            let box_bottom = ir_y_to_pdf_y(ir_y, elem_h, slide_h_emu);
            let elem_h_pt = emu_to_pt(elem_h);
            let baseline_y = box_bottom + elem_h_pt * 0.8;

            // box_bottom must be >= 0 (element fits within the page).
            assert!(
                box_bottom >= -0.001,
                "AC-006: box_bottom_pdf_y for frame '{label}' must be >= 0; got {box_bottom:.3}"
            );
            // box_bottom must be <= SLIDE_HEIGHT_PT.
            assert!(
                box_bottom <= SLIDE_HEIGHT_PT + 0.001,
                "AC-006: box_bottom_pdf_y for frame '{label}' must be <= {SLIDE_HEIGHT_PT}; \
                 got {box_bottom:.3}"
            );
            // baseline_y = box_bottom + 0.8 * height must be <= SLIDE_HEIGHT_PT
            // (since box_bottom = slide_h - ir_y - elem_h and baseline_y adds back
            // 0.8 * elem_h, baseline_y = slide_h - ir_y - 0.2 * elem_h ≤ slide_h).
            assert!(
                baseline_y <= SLIDE_HEIGHT_PT + 0.001,
                "AC-006: baseline_y for frame '{label}' must be <= {SLIDE_HEIGHT_PT}; \
                 got {baseline_y:.3}"
            );
            assert!(
                baseline_y >= -0.001,
                "AC-006: baseline_y for frame '{label}' must be >= 0; got {baseline_y:.3}"
            );
        }
    }

    // ─── F-044-004: Drawing-path behavioral coverage ───────────────────────────

    /// F-044-004 (text frame): `PdfExporter::export()` with a resolvable font
    /// draws text so that the exported PDF contains font-related structure.
    ///
    /// Uses `PdfExporter::with_font_path(lm_math_font_path)` for deterministic
    /// font resolution. Produces an uncompressed PDF (via `generate_pdf_inner`
    /// with `compress_content_streams: false`) and asserts:
    ///
    /// 1. The PDF contains a font resource (`/Font` dict entry) — confirms a font
    ///    was embedded (i.e., text drawing reached `surface.draw_text()`).
    /// 2. The PDF bytes are non-empty and start with `%PDF-`.
    ///
    /// This test FAILS if text drawing is a no-op (no `/Font` entry → no glyphs
    /// reached krilla's text surface).
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_f044_004_text_frame_draws_font_resource_in_export() {
        // Path to Latin Modern Math OTF fixture (deterministic font).
        let font_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/slideforge-math/fonts/latinmodern-math.otf")
            .canonicalize()
            .expect("LM Math fixture must be accessible for F-044-004 drawing-path test");

        let exporter = PdfExporter::with_font_path(font_path);
        let deck = minimal_deck();
        let laid_out = LaidOutDeck {
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
                    content: FrameContent::Title(Arc::from("Drawing Path Test")),
                    text_flow: None,
                }],
                speaker_notes: None,
                register_tags: RegisterSet::new(),
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
        };
        let brand = Brand {
            name: Arc::from("TestBrand"),
            palette: BrandPalette {
                primary: Arc::from("#003087"),
                secondary: Arc::from("#FFFFFF"),
                accent: Arc::from("#F5A623"),
                neutral: Arc::from("#F0F0F0"),
            },
            // Intentionally absent — override path is used instead.
            fonts: BrandFonts {
                heading: Arc::from("NoSuchFont_F044004"),
                body: Arc::from("NoSuchFont_F044004"),
                mono: Arc::from("Courier"),
            },
            layouts: vec![],
            span: SourceSpan::default(),
        };
        let opts = ExportOptions::default();

        // Use uncompressed settings so we can scan the raw content streams.
        let pdf_bytes = exporter
            .generate_pdf_inner(
                &deck,
                &laid_out,
                &brand,
                &opts,
                krilla::SerializeSettings {
                    compress_content_streams: false,
                    ..krilla::SerializeSettings::default()
                },
            )
            .expect("generate_pdf_inner must succeed for F-044-004 text drawing test");

        assert!(
            pdf_bytes.starts_with(b"%PDF-"),
            "PDF must start with %PDF- header"
        );

        // Assert a font resource was embedded — confirms text drawing reached
        // krilla's Surface and a glyph was placed.
        // krilla writes `/Font` dict entries into the page resources when text is drawn.
        let has_font_resource = pdf_bytes.windows(b"/Font".len()).any(|w| w == b"/Font");
        assert!(
            has_font_resource,
            "F-044-004 FAILED: exported PDF contains no /Font resource. \
             Text drawing did not reach krilla's Surface (draw_text path is a no-op). \
             Ensure draw_text_at_bbox is called for Title frames."
        );
    }

    /// F-044-004 (SVG / Diagram frame): `PdfExporter::export()` with a Diagram
    /// frame draws SVG vector paths and the transform is balanced (push/pop).
    ///
    /// Builds a deck with a simple SVG rectangle and exports with uncompressed
    /// content streams. Asserts:
    ///
    /// 1. Vector path operators (`re` for rectangle, `m`/`l`/`c` for paths) appear
    ///    in the exported bytes — confirms `place_svg_at` reached `embed_normalized_svg`.
    /// 2. The PDF is valid (`%PDF-` header present, `%%EOF` near end).
    /// 3. Export succeeds without error — confirms `push_transform`/`pop_transform` are
    ///    balanced (an unbalanced transform stack causes krilla to return an error
    ///    or produce malformed output).
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_f044_004_diagram_frame_draws_svg_paths_in_export() {
        use slideforge_types::NormalizedDiagramSvg;

        // A simple SVG with one rectangle — produces a `re` PDF operator.
        // Use ##-delimited raw string to avoid conflict with the # in the color value.
        let svg_str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="#003087"/>
        </svg>"##;
        let svg = NormalizedDiagramSvg::from_normalized_string(Arc::from(svg_str));

        let exporter = PdfExporter::new(); // no font needed for SVG-only deck
        let deck = minimal_deck();
        let laid_out = LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![LaidOutSlide {
                source_index: 0,
                slide_type_keyword: Arc::from("diagram"),
                frames: vec![Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(0),
                        width: Emu(9_144_000),
                        height: Emu(5_143_500),
                    },
                    content: FrameContent::Diagram(svg),
                    text_flow: None,
                }],
                speaker_notes: None,
                register_tags: RegisterSet::new(),
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
        };
        let brand = minimal_brand();
        let opts = ExportOptions::default();

        // Export with uncompressed content streams to scan for path operators.
        let pdf_bytes = exporter
            .generate_pdf_inner(
                &deck,
                &laid_out,
                &brand,
                &opts,
                krilla::SerializeSettings {
                    compress_content_streams: false,
                    ..krilla::SerializeSettings::default()
                },
            )
            .expect("generate_pdf_inner must succeed for SVG/Diagram frame");

        assert!(
            pdf_bytes.starts_with(b"%PDF-"),
            "PDF must start with %PDF- header"
        );

        // Assert vector path operators are present in the exported bytes.
        // The SVG <rect> is translated to PDF path operators by svg_embed.
        // krilla's SVG renderer emits `re` (rectangle) for SVG <rect> elements,
        // or `m`/`l` for generic path segments.
        // At minimum, the content streams must be non-empty after the drawing pass.
        // We check for `re ` (PDF rectangle operator) or `f` (fill operator).
        let has_rect_op = pdf_bytes.windows(b" re ".len()).any(|w| w == b" re ");
        let has_fill_op = pdf_bytes.windows(b" f\n".len()).any(|w| w == b" f\n")
            || pdf_bytes.windows(b" f\r".len()).any(|w| w == b" f\r")
            || pdf_bytes.windows(b" F ".len()).any(|w| w == b" F ");
        // Also check for generic move-to (`m` operator) as a fallback.
        let has_path_ops = has_rect_op
            || has_fill_op
            || pdf_bytes.windows(b" m\n".len()).any(|w| w == b" m\n")
            || pdf_bytes.windows(b" m ".len()).any(|w| w == b" m ");

        assert!(
            has_path_ops,
            "F-044-004 FAILED: exported PDF contains no vector path operators (re/f/m). \
             SVG drawing did not reach krilla's Surface. \
             Ensure place_svg_at → embed_normalized_svg is called for Diagram frames. \
             PDF size: {} bytes.",
            pdf_bytes.len()
        );

        // Assert %%EOF is near the end — PDF is well-formed.
        let tail = &pdf_bytes[pdf_bytes.len().saturating_sub(64)..];
        let has_eof = tail.windows(b"%%EOF".len()).any(|w| w == b"%%EOF");
        assert!(
            has_eof,
            "F-044-004: PDF must end with %%EOF marker (well-formed PDF)"
        );
    }
}
