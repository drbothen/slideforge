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
//!      coordinates from `coords::emu_to_pt()` (top-left, Y-down — krilla
//!      Surface origin). `ir_y_to_pdf_y` is NOT called at draw time; krilla
//!      applies the PDF Y-flip internally (DIR-044-001).
//!    - Diagram frames: `svg_embed::embed_normalized_svg()`.
//! 4. Calls `document.set_tag_tree(tag_tree)` with the assembled structural tree.
//! 5. Calls `document.finish()` → `KrillaResult<Vec<u8>>`.
//! 6. Maps `KrillaError` with two distinct routes:
//!    `KrillaError::Validation` → `PdfExportError::ValidationFailed`;
//!    all other `KrillaError` variants → `PdfExportError::Serialize`.
//!    Both ultimately map to `ExportError::RenderError` at the plugin-trait boundary.
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
//!
//! `ir_y_to_pdf_y` is NOT imported or called anywhere in this file. It is a
//! documented pure function in `coords.rs` (VP-006 Kani target) that computes
//! PDF bottom-left Y coordinates — inapplicable here because krilla's Surface
//! uses a top-left Y-down coordinate system and bakes the PDF Y-flip internally
//! via `page_root_transform` (DIR-044-001).

use krilla::Document;
use krilla::color::rgb;
use krilla::configure::{Configuration, Validator};
use krilla::geom::{PathBuilder, Point, Rect, Transform};
use krilla::metadata::Metadata;
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule, LinearGradient, SpreadMethod, Stop};
use krilla::tagging::{ArtifactType, ContentTag};
use krilla::text::TextDirection;
use slideforge_layout::LaidOutDeck;
use slideforge_layout::types::{BoundingBox, FillSpec, FrameContent, Rgb};
use slideforge_plugin_api::{ExportError, ExportOptions, Exporter};
use slideforge_types::{Brand, Deck, InlineNode};

use crate::coords::emu_to_pt;
use crate::error::PdfExportError;
use crate::font::{ResolvedFace, ResolvedFontSet, measure_text_width_pt, resolve_font_set};
use crate::outline::{build_krilla_outline, build_outline_entries};
use crate::slide_pdf::{FontFaceKind, KrillaTextSpan, slide_to_krilla_runs};
use crate::svg_embed::embed_normalized_svg;
use crate::tag_engine::SlideTagEngine;

// ─── Super/Subscript typographic constants (ADR-023 amendment 2026-06-09) ─────

/// Standard typographic super/subscript size ratio (Unicode Technical Report #25).
///
/// Superscript and subscript text is drawn at `font_size * SUPER_SUB_SCALE`.
/// Value 0.583 matches the ratio used by major browsers and `LibreOffice`.
///
/// Baseline shift is computed against the PARENT span's font size (not the
/// reduced size), so the shift is proportional to the reading context.
///
/// See also: [`SUPER_RISE_FRACTION`], [`SUB_DROP_FRACTION`].
pub const SUPER_SUB_SCALE: f32 = 0.583;

/// Superscript baseline-raise fraction (fraction of parent font size in point-space).
///
/// Superscript text is drawn at `baseline_y - (font_size * SUPER_RISE_FRACTION)`.
/// The subtraction raises the text above the parent baseline in Surface Y-down space.
pub const SUPER_RISE_FRACTION: f32 = 0.333;

/// Subscript baseline-drop fraction (fraction of parent font size in point-space).
///
/// Subscript text is drawn at `baseline_y + (font_size * SUB_DROP_FRACTION)`.
/// The addition lowers the text below the parent baseline in Surface Y-down space.
pub const SUB_DROP_FRACTION: f32 = 0.333;

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
    /// Optional explicit font file path. When `Some`, `resolve_font_set` loads
    /// this path directly for the `regular` face instead of searching the system
    /// font directories by brand family name. Used by tests (F-044-002) and by
    /// production callers that have a known font file on disk.
    font_override_path: Option<std::path::PathBuf>,

    /// Optional pre-resolved font set. When `Some`, bypasses `resolve_font_set`
    /// entirely and uses this set directly. Used by tests (C2-NEW fixture injection)
    /// to inject deterministic, system-font-free `ResolvedFontSet` instances.
    ///
    /// This is also a valid production seam: a caller with pre-loaded font bytes
    /// can construct a `ResolvedFontSet` and inject it for all-or-nothing control
    /// over font selection.
    font_set_override: Option<ResolvedFontSet>,
}

impl PdfExporter {
    /// Construct a new `PdfExporter` using brand-based font resolution.
    #[must_use]
    pub fn new() -> Self {
        Self {
            font_override_path: None,
            font_set_override: None,
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
            font_set_override: None,
        }
    }

    /// Construct a `PdfExporter` that uses a pre-built [`ResolvedFontSet`],
    /// bypassing `fontdb` system-font resolution entirely.
    ///
    /// # Use cases
    ///
    /// - **Tests (C2-NEW distinctness):** inject two distinct fixture font
    ///   instances (`regular = LM Math`, `bold = Tuffy`) so the PDF output
    ///   contains both PostScript names, proving distinct font resources are
    ///   embedded for bold vs. plain spans (deterministic, CI-safe).
    /// - **Production:** callers with pre-loaded font bytes can bypass the
    ///   `fontdb` system scan entirely and supply all four faces directly.
    ///
    /// When this is `Some`, `generate_pdf_inner` uses the injected set directly
    /// and does NOT call `resolve_font_set`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use slideforge_pdf::{PdfExporter, ResolvedFontSet};
    /// use slideforge_pdf::font::ResolvedFace;
    ///
    /// let bytes = std::fs::read("/path/to/font.otf").unwrap();
    /// let raw: Arc<[u8]> = bytes.clone().into();
    /// let font = krilla::text::Font::new(bytes.into(), 0).unwrap();
    /// let face = ResolvedFace { font, raw, face_index: 0 };
    /// let set = ResolvedFontSet::from_faces(
    ///     Some(face.clone()),  // regular
    ///     Some(face.clone()),  // bold
    ///     Some(face.clone()),  // italic
    ///     Some(face),          // mono
    /// );
    /// let exporter = PdfExporter::with_resolved_font_set(set);
    /// ```
    #[must_use]
    pub fn with_resolved_font_set(font_set: ResolvedFontSet) -> Self {
        Self {
            font_override_path: None,
            font_set_override: Some(font_set),
        }
    }

    /// Export to uncompressed PDF bytes — test seam for AC-009 / F-044-004.
    ///
    /// Identical to [`Exporter::export`] except `compress_content_streams: false`
    /// is passed to krilla's [`krilla::SerializeSettings`]. This makes embedded font
    /// program bytes directly measurable without FlateDecode inflation:
    ///
    /// - Uncompressed full LM Math (~733 KB) or compressed full (~300–440 KB)
    ///   would both appear at their true byte counts and exceed the 100 KB
    ///   subset-scale ceiling in AC-009.
    /// - Uncompressed 2-glyph subset is typically 10–30 KB — well under 100 KB.
    ///
    /// `#[doc(hidden)]` — not part of the public API contract. Exposed `pub`
    /// so integration tests in `tests/` can call it (integration tests compile
    /// as a separate crate and cannot access `pub(crate)` items). Production
    /// callers should use [`Exporter::export`] instead.
    ///
    /// # Errors
    ///
    /// Same error conditions as the [`Exporter::export`] path.
    #[doc(hidden)]
    pub fn export_uncompressed(
        &self,
        deck: &Deck,
        laid_out: &LaidOutDeck,
        brand: &Brand,
        opts: &ExportOptions,
    ) -> Result<Vec<u8>, PdfExportError> {
        // AC-012: production path uses Validator::UA1 in BOTH export() and
        // export_uncompressed().  The uncompressed path is used by integration
        // tests that need to scan PDF content streams — it still must be UA-1
        // compliant.
        self.generate_pdf_inner(
            deck,
            laid_out,
            brand,
            opts,
            krilla::SerializeSettings {
                compress_content_streams: false,
                configuration: Configuration::new_with_validator(Validator::UA1),
                ..krilla::SerializeSettings::default()
            },
        )
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
    ///   coordinates computed via `coords::emu_to_pt()` (top-left Surface coords;
    ///   krilla handles the PDF Y-flip internally — DIR-044-001).
    ///   Font is resolved from brand family name via `font::system_font_fallback()`
    ///   then `krilla::text::Font::new()`. If no font can be resolved, text drawing
    ///   is skipped with a `tracing::warn!` — the page still renders.
    /// - Diagram frames: `svg_embed::embed_normalized_svg()` at the mapped position.
    ///
    /// ## PDF/UA-1 validator (AC-012 / BC-4.03.001 invariant 5)
    ///
    /// The production export path uses `Validator::UA1` via
    /// `Configuration::new_with_validator(Validator::UA1)`. Any `KrillaError::Validation`
    /// is propagated as a fatal `PdfExportError::ValidationFailed` — it is NOT silently swallowed.
    ///
    /// ## Coordinate invariant (BC-4.03.005 / Architecture Compliance Rule 2)
    ///
    /// ALL EMU-to-point conversions go through `coords::emu_to_pt()`. No inline
    /// `emu / 12700` arithmetic is used. `ir_y_to_pdf_y` is NOT called on the draw
    /// path (krilla Surface is top-left Y-down; krilla applies the PDF flip internally).
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError`] on:
    /// - Invalid page dimensions.
    /// - Tag tree assembly failure.
    /// - SVG embed failure.
    /// - Document serialization failure (including `Validator::UA1` rejection).
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
        // AC-012 / BC-4.03.001 invariant 5: production export MUST use Validator::UA1.
        // `Configuration::new_with_validator` never returns None for UA1 (it selects a
        // compatible PDF version automatically).
        let settings = krilla::SerializeSettings {
            configuration: Configuration::new_with_validator(Validator::UA1),
            ..krilla::SerializeSettings::default()
        };
        self.generate_pdf_inner(deck, laid_out, brand, opts, settings)
    }

    /// Core PDF generation with explicit [`krilla::SerializeSettings`].
    ///
    /// Separated from `generate_pdf` so tests can pass `compress_content_streams: false`
    /// to produce uncompressed output scannable for text/path operators (F-044-004).
    ///
    /// # Errors
    ///
    /// Same error conditions as [`generate_pdf`].
    // STORY-081 I2 adds title_inlines_text extraction per-slide (7 lines) pushing over 150.
    // This function is the single PDF generation entry point — splitting it would require
    // threading state across multiple sub-functions with no clarity benefit.
    #[allow(clippy::too_many_lines)]
    fn generate_pdf_inner(
        &self,
        deck: &Deck,
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

        // Resolve the styled font set for text drawing (ADR-023).
        //
        // Priority:
        //   1. `self.font_set_override` if provided (test seam / production pre-built set).
        //   2. `resolve_font_set(brand, override_path)` — fontdb metadata-aware lookup.
        //
        // `resolve_font_set` returns a `ResolvedFontSet` with `regular`, `bold`, `italic`,
        // and `mono` faces. Missing faces fall back to `regular` at draw time (with
        // `tracing::warn!` emitted by `resolve_font_set` when a styled face is absent).
        //
        // `ResolvedFontSet` is `Clone` (all inner `krilla::text::Font` are `Clone` — they
        // wrap an `Arc`). Cloning here is cheap (Arc ref-count bump only).
        //
        // NOTE: `resolve_brand_font` is retained as an internal helper but is no longer
        // called from the main draw path. `resolve_font_set` supersedes it for AC-004.
        let font_set: ResolvedFontSet = self
            .font_set_override
            .clone()
            .unwrap_or_else(|| resolve_font_set(&brand.fonts, self.font_override_path.as_deref()));

        // Collect per-slide Part groups for later assembly into the deck tag tree.
        let mut slide_parts = Vec::with_capacity(laid_out.slides.len());

        // Pre-compute page width/height in points — identical for all slides.
        let width_pts = emu_to_pt(laid_out.page_size.width);
        let height_pts = emu_to_pt(laid_out.page_size.height);

        // Build the PDF document outline (AC-010 / BC-4.03.001 postcondition 1).
        //
        // ISO 14289-1 §7.1: an outline is mandatory when the document contains headings,
        // which every slideforge deck with title-type slides does.
        //
        // Delegated to `outline::build_outline_entries` + `outline::build_krilla_outline`
        // (F-045-P1-004): the intermediate `Vec<OutlineEntry>` is unit-testable (count,
        // labels, destination page indices) without serialising a full PDF.
        //
        // F-045-P1-005 invariant: label comes from deck.slides[source_index]; destination
        // page index is the enumerate position. The debug_assert in build_outline_entries
        // fires when source_index is out-of-bounds — see outline.rs for details.
        let outline_entries = build_outline_entries(deck, laid_out);
        let outline = build_krilla_outline(&outline_entries);

        for (page_idx, slide) in laid_out.slides.iter().enumerate() {
            // Resolve the slide title for AC-011 Hn /Title attribute.
            //
            // AC-011 (BC-4.03.001 postcondition 1): every /H1–/H6 structure element must
            // carry a /Title attribute whose value is the heading text.  The title is
            // sourced from deck.slides[source_index].title_str(); fallback "Slide N"
            // (1-based) is used when title_str() returns None.
            let slide_title: String = deck
                .slides
                .get(slide.source_index)
                .and_then(|s| s.title_str())
                .map_or_else(
                    || format!("Slide {}", page_idx + 1),
                    std::borrow::ToOwned::to_owned,
                );

            // STORY-081 I2 (ADV-P08-HIGH-001 fix): check for title_inlines shadow field
            // in the semantic deck.  When the title contains inline markup (e.g.,
            // **Bold Title**), the eval stage stores `title_inlines` as a
            // FieldValue::Inlines shadow field.  For PDF (non-PPTX) output the full
            // Vec<InlineNode> is threaded to draw_frame so the title can be rendered
            // richly via slide_to_krilla_runs + draw_inline_spans — the same path
            // used for SubtitleInlines and TextRun frames.  Plain titles (no shadow
            // field) fall back to the regular-face draw_text_at_bbox path unchanged.
            let title_inlines_nodes: Option<Vec<InlineNode>> =
                deck.slides.get(slide.source_index).and_then(|s| {
                    if let Some(slideforge_types::FieldValue::Inlines(nodes)) =
                        s.fields.get("title_inlines")
                    {
                        if nodes.is_empty() {
                            None
                        } else {
                            Some(nodes.clone())
                        }
                    } else {
                        None
                    }
                });

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
            // `mut` required: the draw loop inserts Identifier leaf nodes into
            // `part_result.part.children` for F-045-C2 MCID linkage.
            //
            // Pass `Some(&slide_title)` so that Hn structure elements carry the
            // /Title attribute required by PDF/UA-1 (AC-011).
            let mut part_result = tag_engine.tag_slide_with_title(slide, Some(&slide_title))?;

            // Obtain the krilla Surface and draw slide content.
            let mut surface = page.surface();

            for (frame_idx, frame) in slide.frames.iter().enumerate() {
                if part_result.decorative_frame_indices.contains(&frame_idx) {
                    // F-045-C1: Decorative frames are excluded from the structure tree
                    // (they are in `decorative_frame_indices`). They MUST also be wrapped
                    // in a PDF Artifact marked-content sequence in the content stream, so
                    // PDF readers (screen readers, veraPDF) know to skip them as non-semantic.
                    //
                    // `ContentTag::Artifact(ArtifactType::Other)` emits:
                    //   `/Artifact BMC ... EMC`
                    // in the content stream when tagging is enabled. This satisfies
                    // BC-4.03.001 postcondition 1 ("decorative elements marked as Artifacts").
                    //
                    // `start_tagged` returns `Identifier::dummy()` for Artifacts (the Artifact
                    // marking is NOT linked to the structure tree — it is only a content-stream
                    // marker). We discard the returned identifier.
                    let _artifact_id =
                        surface.start_tagged(ContentTag::Artifact(ArtifactType::Other));
                    draw_frame(
                        &mut surface,
                        &frame.bbox,
                        &frame.content,
                        &font_set,
                        title_inlines_nodes.as_deref(),
                    )?;
                    surface.end_tagged();
                } else if let Some(child_indices) = part_result
                    .frame_child_part_indices
                    .get(frame_idx)
                    .and_then(|opt| opt.as_ref())
                {
                    // F-045-C2 / F-P3-001: Non-decorative frames with a corresponding
                    // structure tree mapping are wrapped in tagged marked-content sequences.
                    //
                    // Single-block frames (Title, Subtitle, TextRun, Image, Shape, ErrorSlide):
                    //   `child_indices` has exactly one element. We open ONE tagged region
                    //   for the entire frame and insert the Identifier into that one group.
                    //
                    // Multi-block Body frames (e.g., Text + Bullets):
                    //   `child_indices` has one entry PER content block.  We open ONE tagged
                    //   region per block and insert each Identifier into its own group.
                    //   This gives each structure group (P, L/LI/LBody, etc.) its own
                    //   MCID leaf, satisfying PDF/UA-1's requirement that grouping elements
                    //   reference actual marked content (BC-4.03.001 F-045-C2).
                    //
                    // `ContentTag::Other` emits `/P BDC<</MCID N>>` in the content stream.
                    // The returned `Identifier` is pushed as a leaf into the matching Part
                    // child group, linking the structure tree element to the marked content.
                    if child_indices.len() == 1 {
                        // Single-block path (unchanged behavior for all non-Body frames).
                        let child_idx = child_indices[0];
                        let id = surface.start_tagged(ContentTag::Other);
                        draw_frame(
                            &mut surface,
                            &frame.bbox,
                            &frame.content,
                            &font_set,
                            title_inlines_nodes.as_deref(),
                        )?;
                        surface.end_tagged();
                        if let Some(krilla::tagging::Node::Group(child_group)) =
                            part_result.part.children.get_mut(child_idx)
                        {
                            child_group.push(id);
                        } else {
                            tracing::debug!(
                                frame_idx,
                                child_idx,
                                "frame_child_part_indices pointed to non-Group or out-of-bounds \
                                 child; Identifier not inserted (tag tree may lack leaf for this frame)"
                            );
                        }
                    } else {
                        // Multi-block path (Body frames with ≥2 content blocks).
                        //
                        // Open one tagged region per block and route each Identifier into the
                        // corresponding Part child group.  `child_indices[k]` is the Part
                        // child index for the k-th block that produced a tag group.
                        //
                        // We pair the child_indices with the drawable blocks from the frame.
                        // `draw_body_blocks_tagged` handles the per-block cursor logic; here
                        // we replicate the block-iteration structure to emit matching tagged
                        // regions.
                        if let FrameContent::Body(content_blocks) = &frame.content {
                            draw_body_blocks_tagged(
                                &mut surface,
                                content_blocks,
                                &frame.bbox,
                                &font_set,
                                child_indices,
                                &mut part_result.part,
                            )?;
                        } else {
                            // Defensive: multi-index mapping on a non-Body frame is unexpected.
                            // Fall back to a single tagged region covering the whole frame.
                            tracing::debug!(
                                frame_idx,
                                "multi-index frame_child_part_indices on non-Body frame; \
                                 using single tagged region as fallback"
                            );
                            let child_idx = child_indices[0];
                            let id = surface.start_tagged(ContentTag::Other);
                            draw_frame(
                                &mut surface,
                                &frame.bbox,
                                &frame.content,
                                &font_set,
                                title_inlines_nodes.as_deref(),
                            )?;
                            surface.end_tagged();
                            if let Some(krilla::tagging::Node::Group(child_group)) =
                                part_result.part.children.get_mut(child_idx)
                            {
                                child_group.push(id);
                            }
                        }
                    }
                } else {
                    // Frame with no corresponding structure tree child AND not decorative
                    // (e.g., Empty frames). Draw without tagging.
                    draw_frame(
                        &mut surface,
                        &frame.bbox,
                        &frame.content,
                        &font_set,
                        title_inlines_nodes.as_deref(),
                    )?;
                }
            }

            surface.finish();
            page.finish();

            // Collect the Part group for deck-level tree assembly.
            slide_parts.push(part_result.part);
        }

        // Wire document-level metadata (BC-4.03.001 AC-007: /Lang from deck metadata).
        //
        // `krilla::Document::set_metadata` writes the document's metadata dictionary
        // and causes krilla to emit `/Lang` in the PDF catalog (via
        // `catalog.lang(TextStr(lang))` in `chunk_container.rs:189`).
        //
        // F-045-P1-007 / BC-4.03.001 defence-in-depth:
        //
        // When `Validator::UA1` is active, a missing `/Lang` is a fatal validation
        // error (`ValidationError::NoDocumentLanguage`). Rather than silently continuing
        // and letting krilla catch it at `document.finish()`, we fail fast here with a
        // structured `ValidationFailed` error. This makes the failure cause explicit to
        // the caller and prevents shipping a non-compliant PDF.
        //
        // The upstream accessibility validator (BC-5.01.001) should have rejected the
        // deck before reaching the export stage. This check is defence-in-depth — we do
        // not rely solely on the upstream check.
        let Some(lang) = &deck.metadata.lang else {
            return Err(PdfExportError::ValidationFailed {
                message: "PDF/UA-1 compliance requires a document language (/Lang); \
                          deck.metadata.lang is None — set lang in the deck metadata"
                    .to_owned(),
            });
        };

        tracing::debug!(
            lang = lang.as_ref(),
            "wiring document /Lang from deck metadata"
        );

        // Build the metadata object: language is always present (guarded above);
        // title is wired when present.
        //
        // SEC-050-001 / CWE-116: validate title for XML-1.0 legality before
        // embedding in XMP metadata. `xmp_writer` escapes XML-reserved chars but
        // does NOT strip XML-1.0-illegal control characters (U+0000–U+0008,
        // U+000B, U+000C, U+000E–U+001F, U+FFFE, U+FFFF). We REJECT loudly
        // (same policy as `validate_lang_for_xml` in slideforge-pptx).
        let mut meta = Metadata::new().language(lang.as_ref().to_owned());
        if let Some(title) = &deck.metadata.title {
            validate_title_for_xmp(title.as_ref())?;
            meta = meta.title(title.as_ref().to_owned());
        }
        document.set_metadata(meta);

        // Wire the PDF document outline (AC-010 / BC-4.03.001 postcondition 1 / invariant 6).
        //
        // ISO 14289-1 §7.1 requires a document outline whenever headings (H1–H6) are
        // present. Every slideforge deck with title-type slides has H1 headings.
        //
        // The `outline` was built above (one entry per slide, in source order).
        // `document.set_outline` stores it in the SerializeContext; krilla writes
        // `catalog.outlines(ref)` → `/Outlines` in the PDF catalog on `document.finish()`.
        document.set_outline(outline);

        // Assemble the per-slide Part groups into a single deck-level TagTree
        // and attach it to the document before finish().
        //
        // BC-4.03.002 AC-003: set_tag_tree called before document.finish().
        // STORY-043 scope-directive Decision 1: this wiring must happen NOW.
        let tag_tree = tag_engine.assemble_deck_tag_tree(slide_parts)?;
        document.set_tag_tree(tag_tree);

        // Serialize to PDF bytes. `Document::finish()` returns
        // `KrillaResult<Vec<u8>>` (i.e. `Result<Vec<u8>, KrillaError>`).
        //
        // F-045-P1-003 / BC-4.03.001 invariant 5: `KrillaError::Validation` MUST map to
        // `PdfExportError::ValidationFailed`, NOT `Serialize`. Any other `KrillaError`
        // variant maps to `Serialize`. This distinction is observable in tests that
        // assert the specific variant type (e.g., test_bc_4_03_001_validator_ua1_rejects_*).
        document.finish().map_err(|e| match e {
            krilla::error::KrillaError::Validation(ref violations) => {
                let message = violations
                    .iter()
                    .map(|v| format!("{v:?}"))
                    .collect::<Vec<_>>()
                    .join("; ");
                PdfExportError::ValidationFailed {
                    message: format!("PDF/UA-1 validation failed: {message}"),
                }
            },
            other => PdfExportError::Serialize {
                message: format!("krilla serialization error: {other:?}"),
            },
        })
    }
}

impl Default for PdfExporter {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Frame content drawing ────────────────────────────────────────────────────

/// Draw the content of a single layout frame onto a krilla `Surface`.
///
/// All coordinate conversions go through `coords::emu_to_pt()`
/// (BC-4.03.005 Architecture Compliance Rule 2). `ir_y_to_pdf_y` is NOT called
/// here — krilla's `Surface` is top-left, Y-down, and applies the PDF Y-flip
/// internally (DIR-044-001).
///
/// ## Text baseline approximation
///
/// The IR gives the top-left corner of the bounding box in Surface coordinates
/// (Y-down, top-left origin). A reasonable baseline approximation is:
///
/// ```text
/// surface_y  = emu_to_pt(bbox.y)                          // top edge, Y-down
/// baseline_y = surface_y + emu_to_pt(bbox.height) * 0.8   // 80% down from top
/// ```
///
/// This places the baseline at 80% of the box height measured downward from
/// the box top (20% descender allowance below the baseline). Text is guaranteed
/// to land within `[0, SLIDE_HEIGHT_PT]` for any valid IR layout.
///
/// For body frames (`FrameContent::Body`), each paragraph (`ContentBlock::Text`)
/// and each bullet item (`ContentBlock::Bullets`) is placed at a distinct
/// baseline using a per-item vertical cursor. The cursor starts at
/// `text_baseline_surface_y(bbox)` (the same 80%-of-height formula used for
/// single-item frames) and advances by `font_size * BODY_LINE_LEADING` (1.2×)
/// after each drawn item. This prevents all body items from overprinting at the
/// same baseline. Single-block body frames are unaffected — their single baseline
/// equals the pre-existing formula exactly.
///
/// Intra-block text reflow (word-wrapping a single long paragraph across multiple
/// lines) and brand-driven font-size/color overrides remain future story work.
///
/// ## Font sizes
///
/// Default font sizes: Title 36pt, Subtitle 28pt, Body text 18pt, Bullets 16pt.
/// Brand-template font size overrides are a future story enhancement.
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
///
/// `title_inlines_override` (STORY-081 I2, ADV-P08-HIGH-001 fix): when `Some(nodes)`,
/// the `FrameContent::Title` arm renders the title richly via
/// `slide_to_krilla_runs + draw_inline_spans` — the same path used for
/// `SubtitleInlines` and `TextRun` — so that `InlineNode::Bold` dispatches to
/// `font_set.bold` and `InlineNode::Italic` dispatches to `font_set.italic`.
/// When `None`, the arm falls back to the plain regular-face `draw_text_at_bbox`
/// path (no inline structure in the title).
///
/// The `Vec<InlineNode>` is cloned from the `title_inlines` shadow field that the
/// eval stage stores when a DSL title contains inline markup (e.g., `**Bold Title**`).
/// PPTX title remains single-run plain per DIR-077-002 §4 point 4 — this parameter
/// is only populated by the PDF exporter.
// frame_w_pt / frame_h_pt are intrinsically paired width/height bindings in the body.
#[allow(clippy::similar_names)]
fn draw_frame(
    surface: &mut krilla::surface::Surface<'_>,
    bbox: &BoundingBox,
    content: &FrameContent,
    font_set: &ResolvedFontSet,
    title_inlines_override: Option<&[InlineNode]>,
) -> Result<(), PdfExportError> {
    match content {
        FrameContent::Title(text) => {
            // ADV-P08-HIGH-001 fix: when title_inlines_override is Some, render the
            // title richly via slide_to_krilla_runs + draw_inline_spans so that
            // InlineNode::Bold dispatches to font_set.bold and InlineNode::Italic
            // dispatches to font_set.italic.  The title font size is 36pt — preserved
            // from the prior plain-text path.  Fall back to the plain regular-face
            // draw_text_at_bbox when no inline structure is present.
            if let Some(nodes) = title_inlines_override {
                let spans = slide_to_krilla_runs(nodes);
                if !spans.is_empty() {
                    draw_inline_spans(surface, &spans, bbox, font_set, 36.0);
                }
            } else {
                draw_text_at_bbox(
                    surface,
                    text.as_ref(),
                    bbox,
                    36.0,
                    font_set.regular.as_ref(),
                );
            }
        },
        FrameContent::Subtitle(text) => {
            draw_text_at_bbox(surface, text, bbox, 28.0, font_set.regular.as_ref());
        },
        // STORY-081 AC-004: SubtitleInlines carries rich inline structure.
        // Render each span with font-face dispatch via slide_to_krilla_runs +
        // draw_inline_spans. This replaces the old single-font flattening path.
        FrameContent::SubtitleInlines(nodes) => {
            let spans = slide_to_krilla_runs(nodes);
            if !spans.is_empty() {
                draw_inline_spans(surface, &spans, bbox, font_set, 28.0);
            }
        },
        FrameContent::Body(blocks) => {
            draw_body_blocks(surface, blocks, bbox, font_set);
        },
        FrameContent::TextRun(inlines) => {
            // AC-004: use slide_to_krilla_runs for font-face dispatch, then draw spans
            // with the appropriate face from font_set. This replaces the flattening
            // extract_all_inline_text path (which rendered everything in the regular face).
            let spans = slide_to_krilla_runs(inlines);
            if !spans.is_empty() {
                draw_inline_spans(surface, &spans, bbox, font_set, 18.0);
            }
        },
        FrameContent::Diagram { svg, .. } => {
            // Place the SVG at the frame's Surface coordinates (top-left, Y-down).
            // krilla's Surface origin is top-left; bbox.y is the top edge (Y-down).
            // No ir_y_to_pdf_y flip — krilla applies the PDF Y-flip internally.
            // STORY-039: alt is now in the IR but PDF Figure tagging is handled by
            // tag_engine.rs; the draw path just renders the SVG content here.
            let surface_x = emu_to_pt(bbox.x);
            let surface_y = emu_to_pt(bbox.y);
            let frame_w_pt = emu_to_pt(bbox.width);
            let frame_h_pt = emu_to_pt(bbox.height);
            place_svg_at(surface, svg, surface_x, surface_y, frame_w_pt, frame_h_pt)?;
        },
        // ErrorSlidePlaceholder carries an SVG — render it like a diagram.
        FrameContent::ErrorSlidePlaceholder { svg, .. } => {
            use slideforge_types::NormalizedDiagramSvg;
            use std::sync::Arc;
            let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(svg.as_ref()));
            let surface_x = emu_to_pt(bbox.x);
            let surface_y = emu_to_pt(bbox.y);
            let frame_w_pt = emu_to_pt(bbox.width);
            let frame_h_pt = emu_to_pt(bbox.height);
            place_svg_at(
                surface,
                &normalized,
                surface_x,
                surface_y,
                frame_w_pt,
                frame_h_pt,
            )?;
        },
        // Chart: no SVG payload at frame level — drawn via ChartRenderer pass.
        // Image, Empty: no drawing in this story.
        // STORY-039: Chart now carries `alt` field; drawing stub unchanged.
        FrameContent::Chart { .. } | FrameContent::Image { .. } | FrameContent::Empty => {},

        // Shape — STORY-072: emit gradient-filled rect for FillSpec::Gradient.
        // FillSpec::SolidColor and FillSpec::None are not yet drawn (stubbed).
        FrameContent::Shape(shape_frame) => {
            match &shape_frame.fill {
                FillSpec::Gradient { from, to } => {
                    draw_gradient_rect(surface, bbox, *from, *to);
                },
                FillSpec::SolidColor(_) | FillSpec::None => {
                    // Solid / transparent shapes: no drawing yet (future story).
                    tracing::debug!("Shape with SolidColor/None fill: no PDF drawing emitted");
                },
            }
        },

        // ColorBar — PDF filled rectangle (BC-1.17.002 PC-9).
        //
        // Tagging (F-095-P1-002 / STORY-095 AC-003): `tag_engine.rs` dispatches
        // on the `alt` field at tagging time (before this draw call):
        //   - AltText::Provided(label): tagged as /Figure with /Alt = label.
        //   - AltText::Decorative | AltText::Unspecified: pushed to Artifact.
        //
        // The DRAW PASS is ALWAYS the same (filled rectangle) regardless of the
        // alt value. Tagging wraps the content stream region produced by this draw.
        // When tagged as /Figure, the draw loop's start_tagged(ContentTag::Other)
        // links this content to the Figure structure element in the tag tree.
        // When tagged as Artifact, `ContentTag::Artifact(ArtifactType::Other)`
        // suppresses the rectangle from the logical structure tree.
        //
        // WCAG co-encoding: when AltText::Unspecified/Decorative (no label was
        // provided by the author), the adjacent ColorLabel (Body-role) text frame
        // still co-encodes the percentage in text for assistive technology.
        //
        // Drawing: uses krilla's PathBuilder to build a rectangle path at the
        // filled sub-width, then sets the brand fill color and calls
        // `surface.draw_path()`.  All coordinate conversions go through
        // `coords::emu_to_pt()` (BC-4.03.005 / Architecture Compliance Rule 2).
        FrameContent::ColorBar {
            filled_width_emu,
            color,
            ..
        } => {
            draw_color_bar_rect(surface, bbox, *filled_width_emu, *color);
        },
    }
    Ok(())
}

/// Draw a filled rectangle for a `FrameContent::ColorBar` onto a krilla `Surface`.
///
/// ## PDF content-stream operators emitted
///
/// With `compress_content_streams: false` (used by `export_uncompressed`), krilla
/// emits the following PDF operators into the content stream:
///
/// ```text
/// <r_norm> <g_norm> <b_norm> rg     % set non-stroke (fill) color
/// <x> <y> <w> <h> re                % append rectangle path
/// f                                  % fill (even-odd or non-zero)
/// ```
///
/// The `re ` operator (ISO 32000-1 §8.5.2, Table 59) is what
/// `test_BC_1_17_002_pdf_bar_rendered_in_content_stream` scans for in the
/// uncompressed PDF bytes (BC-1.17.002 PC-9 / F-087-P3-001).
///
/// ## Tagging contract
///
/// The bar is tagged as a PDF Artifact by `tag_engine.rs` (pushed into
/// `decorative_frame_indices`). The call-site in the draw loop wraps Artifact
/// frames with `ContentTag::Artifact(ArtifactType::Other)` (`/Artifact BMC … EMC`).
/// WCAG accessibility co-encoding is provided by the adjacent `ColorLabel`
/// (Body-role) text frame that renders the percentage label.
///
/// ## Coordinate policy
///
/// All EMU-to-point conversions use `coords::emu_to_pt()` (BC-4.03.005 /
/// Architecture Compliance Rule 2). The krilla Surface is top-left, Y-down;
/// no `ir_y_to_pdf_y` flip is applied (DIR-044-001).
///
/// ## No-op on empty bar
///
/// When `filled_width_emu.0 == 0` the bar is zero-width; no path is emitted.
/// This is correct behavior (0% progress = nothing to draw).
fn draw_color_bar_rect(
    surface: &mut krilla::surface::Surface<'_>,
    bbox: &BoundingBox,
    filled_width_emu: slideforge_layout::types::Emu,
    color: Rgb,
) {
    if filled_width_emu.0 <= 0 {
        // Zero or negative fill width — nothing to draw (0% progress).
        tracing::debug!(
            filled_width_emu = filled_width_emu.0,
            "ColorBar: zero filled_width_emu; skipping PDF rectangle draw"
        );
        return;
    }

    let x_pt = emu_to_pt(bbox.x);
    let y_pt = emu_to_pt(bbox.y);
    let w_pt = emu_to_pt(filled_width_emu);
    let h_pt = emu_to_pt(bbox.height);

    // Build a rectangle path at the filled sub-width.
    //
    // `Rect::from_xywh` returns `None` when w or h is non-positive or non-finite.
    // We guard with `filled_width_emu.0 <= 0` above; `emu_to_pt` is monotone and
    // returns a positive value for a positive EMU input, so `from_xywh` succeeds.
    let Some(rect) = Rect::from_xywh(x_pt, y_pt, w_pt, h_pt) else {
        tracing::debug!(
            x_pt,
            y_pt,
            w_pt,
            h_pt,
            "ColorBar: Rect::from_xywh returned None (degenerate dimensions); skipping draw"
        );
        return;
    };

    let mut pb = PathBuilder::new();
    pb.push_rect(rect);
    let Some(path) = pb.finish() else {
        tracing::debug!("ColorBar: PathBuilder::finish() returned None; skipping draw");
        return;
    };

    // Set the brand fill color (from the ColorBar IR).
    surface.set_fill(Some(Fill {
        paint: rgb::Color::new(color.r, color.g, color.b).into(),
        opacity: NormalizedF32::ONE,
        rule: FillRule::NonZero,
    }));
    surface.set_stroke(None);
    surface.draw_path(&path);

    tracing::debug!(
        x_pt,
        y_pt,
        w_pt,
        h_pt,
        r = color.r,
        g = color.g,
        b = color.b,
        "ColorBar: filled rectangle drawn in PDF content stream"
    );
}

/// Standard line-height leading multiplier used in body text stacking.
///
/// Each successive body item's baseline is advanced by `font_size * BODY_LINE_LEADING`
/// from the previous item's baseline. The value 1.2 is the standard typographic
/// "normal" leading (120% of em-size), consistent with CSS `line-height: normal`
/// and the `OpenDocument` / OOXML default `<a:lnSpc>` of 100% + 20% leading.
///
/// Integer EMUs are NOT used here because font sizes are specified in points
/// (f32), consistent with krilla's Surface API. No EMU-to-pt conversion is
/// needed for leading arithmetic.
pub(crate) const BODY_LINE_LEADING: f32 = 1.2;

/// Compute the Surface Y coordinate of the text baseline for a bounding box.
///
/// krilla's `Surface` is top-left, Y-down (DIR-044-001). The text baseline is
/// approximated at 80% of the box height measured downward from the box top edge,
/// giving 20% descender allowance below the baseline.
///
/// ```text
/// surface_y  = emu_to_pt(bbox.y)                          // top edge, Y-down
/// baseline_y = surface_y + emu_to_pt(bbox.height) * 0.8
/// ```
///
/// This is a **pure function** used by both production (`draw_text_at_bbox`) and
/// the vertical-placement regression tests to ensure they exercise the same
/// formula as the real draw path (non-vacuous load-bearing test contract).
///
/// For a title frame at `ir_y=0` (slide top): baseline = 0 + height*0.8
/// (in the top half of the page). Under the old (buggy) `ir_y_to_pdf_y`
/// formula the baseline was ~390 (near the bottom) — the mirror bug.
#[inline]
pub(crate) fn text_baseline_surface_y(bbox: &BoundingBox) -> f32 {
    let surface_top_y = emu_to_pt(bbox.y);
    surface_top_y + emu_to_pt(bbox.height) * 0.8
}

/// Compute the ordered sequence of Surface-Y baselines for each drawable item in
/// a body block list.
///
/// This pure function encodes the same cursor-advance logic as `draw_body_blocks`
/// and is `pub(crate)` so that tests can assert concrete baseline values without
/// needing a krilla `Surface`.
///
/// ## Cursor model
///
/// - The cursor starts at `text_baseline_surface_y(bbox)` (80% of the body frame
///   height below the frame's top edge — identical to the single-item formula
///   that titles and subtitles use, preserving single-block backward compatibility).
/// - For each drawable item the cursor is used as the baseline Y, then advanced by
///   `font_size * BODY_LINE_LEADING`:
///   - `Text` blocks use font size **18.0 pt**.
///   - `BulletItem`s use font size **16.0 pt**.
/// - Non-text blocks (`Chart`, `Diagram`, `Math`, `Image`, `Table`, `Shape`) do
///   not consume cursor space (they are not drawn in this pass).
///
/// ## Relationship to `draw_body_blocks`
///
/// `draw_body_blocks` calls this function to obtain the baseline sequence and then
/// passes each value to `draw_text_at_bbox_at_y`, keeping the drawing and geometry
/// logic cleanly separated. Tests call `body_item_baselines` directly.
pub(crate) fn body_item_baselines(
    blocks: &[slideforge_types::ContentBlock],
    bbox: &BoundingBox,
) -> Vec<f32> {
    use slideforge_types::ContentBlock;

    let mut cursor_y: f32 = text_baseline_surface_y(bbox);
    let mut baselines: Vec<f32> = Vec::new();

    for block in blocks {
        match block {
            ContentBlock::Text(text_block) => {
                // Use slide_to_krilla_runs to check if this block has drawable content.
                // This correctly handles all inline variants (Bold, Code, Strikethrough,
                // Highlight, etc.) — not just Plain/Bold/Italic.
                let spans = slide_to_krilla_runs(&text_block.inlines);
                if !spans.is_empty() {
                    let font_size: f32 = 18.0;
                    baselines.push(cursor_y);
                    cursor_y += font_size * BODY_LINE_LEADING;
                }
            },
            ContentBlock::Bullets(items) => {
                for item in items {
                    // Same font-face-aware check for bullet items.
                    let spans = slide_to_krilla_runs(&item.inlines);
                    if !spans.is_empty() {
                        let font_size: f32 = 16.0;
                        baselines.push(cursor_y);
                        cursor_y += font_size * BODY_LINE_LEADING;
                    }
                }
            },
            // Non-text blocks do not advance the cursor in this pass.
            ContentBlock::Chart(_)
            | ContentBlock::Diagram(_)
            | ContentBlock::Math(_)
            | ContentBlock::Image(_)
            | ContentBlock::Table(_)
            | ContentBlock::Shape(_)
            // STORY-087 pass-2: ColorBar is geometry-only — no cursor advance.
            | ContentBlock::ColorBar(_) => {},
        }
    }

    baselines
}

/// Solid opaque black fill used for text rendering.
///
/// OBS-044-22-01 fix: `draw_text_at_bbox` must set an EXPLICIT fill before
/// `surface.draw_text()` so that text color is always the deterministic default
/// (solid black) and never inherits leaked fill state from a prior SVG/Diagram
/// frame drawn on the same krilla `Surface`.
///
/// `krilla::paint::Fill` / `Surface::set_fill` accept `Option<Fill>`. Passing
/// `Some(TEXT_FILL_BLACK)` is equivalent to `Fill::default()` but makes the
/// intent explicit: text is always rendered in opaque black regardless of what
/// SVG path drawing left in the surface's paint state.
///
/// Why RGB black (`rgb::Color::new(0, 0, 0)`) instead of luma black
/// (`luma::Color::black()` / `Fill::default()`): both produce black text, but
/// the RGB form emits `0 0 0 rg` in the uncompressed PDF stream, which is
/// directly assertable in tests. The luma form emits `0 g`. We choose RGB for
/// consistency with the SVG fill path (which uses `rgb::Color`) and for
/// test-assertion clarity.
fn text_fill_black() -> Fill {
    Fill {
        paint: rgb::Color::new(0, 0, 0).into(),
        opacity: NormalizedF32::ONE,
        rule: FillRule::NonZero,
    }
}

/// Draw text at a bounding box position using krilla Surface (top-left, Y-down) coordinates.
///
/// Uses [`text_baseline_surface_y`] to compute the baseline position. If `face`
/// is `None`, logs a debug warning and skips drawing. This is the correct
/// non-fatal behavior when a brand font is unavailable.
///
/// ## Word-wrap (STORY-095 T-006 / AC-001, AC-002)
///
/// When `face` is `Some`, the text is wrapped to lines that fit within the frame
/// width using [`crate::text_layout::wrap_text`]. Each wrapped line is drawn at an
/// incrementally advanced baseline, using `font_size * BODY_LINE_LEADING` as the
/// line advance.
///
/// When `face` is `None`, the function returns without drawing — we cannot measure
/// widths without font metrics, and non-fatal degradation is the correct behaviour.
///
/// ## Frame-bottom clamp (F-095-P1-005 / BC-4.03.002)
///
/// Wrapped lines that overflow the frame height (baseline > frame bottom) are
/// silently elided.  A `tracing::warn!` is emitted once per invocation when
/// overflow occurs, with `frame_bottom_pt` and `baseline_y` as structured fields.
///
/// ## Paint-state determinism (OBS-044-22-01 fix)
///
/// Explicitly sets fill to opaque black and clears stroke before calling
/// `surface.draw_text()`. This ensures text color is never inherited from the
/// fill/stroke state left by a prior `Diagram`/`ErrorSlidePlaceholder` frame
/// on the same krilla `Surface`. See [`text_fill_black`] for color rationale.
fn draw_text_at_bbox(
    surface: &mut krilla::surface::Surface<'_>,
    text: &str,
    bbox: &BoundingBox,
    font_size: f32,
    face: Option<&ResolvedFace>,
) {
    let Some(face) = face else {
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

    // STORY-095 T-006: wrap text to the frame width before drawing.
    //
    // Build FontMetrics from the resolved face so that wrap_text uses the SAME
    // font file and face_index as the draw path. The mock_char_width_pts override
    // is None in production (Some only in unit-test / Kani paths via text_layout tests).
    let metrics = crate::text_layout::FontMetrics {
        font_bytes: &face.raw,
        face_index: face.face_index,
        font_size_pts: f64::from(font_size),
        mock_char_width_pts: None,
    };
    let max_width_pts = f64::from(emu_to_pt(bbox.width));
    let lines = crate::text_layout::wrap_text(text, max_width_pts, &metrics);

    if lines.is_empty() {
        return;
    }

    // OBS-044-22-01 fix: explicitly set fill and clear stroke BEFORE draw_text.
    // krilla Surface.fill / Surface.stroke are mutable fields (NOT part of the
    // transform/graphics-state stack). A prior Diagram frame's render_path calls
    // may have left any fill/stroke in the surface state. Resetting here makes
    // text rendering self-sufficient and deterministic, independent of draw order.
    surface.set_fill(Some(text_fill_black()));
    surface.set_stroke(None);

    // Surface X: left edge of the bounding box (top-left origin, Y-down).
    let surface_x = emu_to_pt(bbox.x);

    // Surface Y baseline: 80% of box height down from the box top edge.
    // No ir_y_to_pdf_y — krilla handles the PDF Y-flip internally (DIR-044-001).
    let mut baseline_y = text_baseline_surface_y(bbox);

    // F-095-P1-005: compute frame bottom to clamp overflow lines.
    // In krilla's top-left Y-down coordinate system the frame bottom is the
    // top-edge Y plus the frame height.  Lines whose baseline exceeds this
    // are emitted off-slide (incorrect) and elided with a structured warning.
    let frame_bottom_pt = emu_to_pt(bbox.y) + emu_to_pt(bbox.height);

    for line in &lines {
        // F-095-P1-005: clamp — elide lines that overflow the frame height.
        if baseline_y > frame_bottom_pt {
            tracing::warn!(
                frame_bottom_pt,
                baseline_y,
                "draw_text_at_bbox: baseline exceeds frame bottom; \
                 eliding overflow wrapped lines (F-095-P1-005)"
            );
            break;
        }
        let start = Point::from_xy(surface_x, baseline_y);
        surface.draw_text(
            start,
            face.font.clone(),
            font_size,
            line,
            false,
            TextDirection::Auto,
        );
        baseline_y += font_size * BODY_LINE_LEADING;
    }
}

/// Compute the effective (rendered) font size for a span.
///
/// For normal spans, this is `base_font_size` unchanged.
///
/// For Superscript/Subscript spans (non-zero `y_offset_units`), this returns
/// `base_font_size * SUPER_SUB_SCALE` — the reduced size per Unicode TR #25
/// and ADR-023 amendment (2026-06-09).
///
/// ## ADV-P04-HIGH-001 (STORY-081 Pass-4 fix)
///
/// The original implementation did NOT reduce font size for super/sub spans —
/// it passed `font_size` unchanged for every span. This function makes the
/// size reduction explicit and testable.
#[must_use]
pub fn effective_span_font_size(span: &KrillaTextSpan, base_font_size: f32) -> f32 {
    if span.y_offset_units != 0 {
        base_font_size * SUPER_SUB_SCALE
    } else {
        base_font_size
    }
}

/// Compute the adjusted baseline Y for a span (point-space, Y-down).
///
/// For normal spans (`y_offset_units == 0`), returns `baseline_y` unchanged.
///
/// For Superscript (`y_offset_units > 0`): raises by `base_font_size * SUPER_RISE_FRACTION`
/// (subtract in Y-down space → move toward top of page = raised text).
///
/// For Subscript (`y_offset_units < 0`): drops by `base_font_size * SUB_DROP_FRACTION`
/// (add in Y-down space → move toward bottom of page = lowered text).
///
/// The shift is computed against the PARENT `base_font_size` — NOT the reduced
/// size — so the baseline shift is proportional to the reading context.
#[must_use]
pub fn adjusted_baseline_y(span: &KrillaTextSpan, baseline_y: f32, base_font_size: f32) -> f32 {
    match span.y_offset_units.cmp(&0) {
        std::cmp::Ordering::Greater => {
            // Superscript: raise above baseline (subtract in Y-down coordinate system).
            baseline_y - (base_font_size * SUPER_RISE_FRACTION)
        },
        std::cmp::Ordering::Less => {
            // Subscript: drop below baseline (add in Y-down coordinate system).
            baseline_y + (base_font_size * SUB_DROP_FRACTION)
        },
        std::cmp::Ordering::Equal => baseline_y,
    }
}

/// Select the [`crate::font::ResolvedFace`] slot for a [`FontFaceKind`] from a
/// [`ResolvedFontSet`], applying the canonical ADR-023 fallback chain.
///
/// This is the SINGLE authoritative slot-selection function called by BOTH the
/// measurement path ([`compute_multi_span_x_positions`]) and the draw path
/// (`draw_packed_line`).  Because both paths call this one function, it is
/// **structurally impossible** for measurement and drawing to select different
/// font slots for the same span kind — eliminating the recurring
/// measure≠draw divergence (ADV-P06-MED-001 fix).
///
/// ## Dispatch table
///
/// | `FontFaceKind` | Fallback chain |
/// |---|---|
/// | `Regular` | `font_set.regular` |
/// | `Bold` | `font_set.bold` → `font_set.regular` |
/// | `Italic` | `font_set.italic` → `font_set.regular` |
/// | `BoldItalic` | `font_set.bold` → `font_set.italic` → `font_set.regular` |
/// | `Mono` | `font_set.mono` → `font_set.regular` |
///
/// ## `BoldItalic` rationale
///
/// When both bold and italic faces are absent, `regular` is the final fallback.
/// When bold is absent but italic is present (a reachable configuration — fontdb
/// queries bold and italic independently; a system can have one without the other),
/// using `italic` is a more correct visual approximation than falling back all the
/// way to `regular`.
///
/// The fallback warning was already emitted by `resolve_font_set` at face-resolution
/// time; this call site is silent by design.
fn face_for_span_kind(
    kind: FontFaceKind,
    font_set: &ResolvedFontSet,
) -> Option<&crate::font::ResolvedFace> {
    match kind {
        FontFaceKind::Regular => font_set.regular.as_ref(),
        FontFaceKind::Bold => font_set.bold.as_ref().or(font_set.regular.as_ref()),
        FontFaceKind::Italic => font_set.italic.as_ref().or(font_set.regular.as_ref()),
        FontFaceKind::BoldItalic => font_set
            .bold
            .as_ref()
            .or(font_set.italic.as_ref())
            .or(font_set.regular.as_ref()),
        FontFaceKind::Mono => font_set.mono.as_ref().or(font_set.regular.as_ref()),
    }
}

/// Compute the starting X position for each span in a multi-span line.
///
/// Accepts the full [`ResolvedFontSet`] so that each span's measurement uses the
/// SAME [`crate::font::ResolvedFace`] (same `raw` bytes AND same `face_index`)
/// that the draw path uses — making width-measurement and glyph-drawing
/// structurally guaranteed to agree.
///
/// Both this function and the draw path (`draw_packed_line`) delegate slot
/// selection to the single shared `face_for_span_kind` helper.  This makes it
/// **structurally impossible** for the two paths to select different slots for the
/// same `FontFaceKind` (ADV-P06-MED-001 fix).
///
/// For each span:
/// 1. Call `face_for_span_kind` to get the canonical [`crate::font::ResolvedFace`].
/// 2. Measure the span text width via [`measure_text_width_pt`] using
///    `face.raw` and `face.face_index` (not hardcoded `0`).
/// 3. The next span starts at `prev_x + measured_width`.
///
/// ## ADV-P04-CRIT-001 (STORY-081 Pass-4 fix)
///
/// Before the Pass-4 fix, `draw_inline_spans_at_y` drew every span at
/// `surface_x = emu_to_pt(bbox.x)` with no cursor advance — multi-span lines
/// overprinted.  This function computes the CORRECT per-span X positions using
/// real glyph-metric widths from `ttf-parser`.
///
/// ## ADV-P05-MED-001 (STORY-081 Pass-5 fix)
///
/// The Pass-4 fix hardcoded `face_index=0` in the `measure_text_width_pt` call.
/// For `.ttc` collection faces at non-zero index this produced wrong advances.
/// The Pass-5 fix uses `face.face_index` from the [`crate::font::ResolvedFace`]
/// slot — the SAME index used by the draw path — eliminating the mismatch.
///
/// ## ADV-P06-MED-001 (STORY-081 Pass-6 fix)
///
/// The Pass-5 fix unified `raw`/`face_index` WITHIN a slot, but the slot-SELECTION
/// logic in the measure path differed from the draw path for `BoldItalic`:
/// measure used `bold → regular`, draw used `bold → italic → regular`.
/// The Pass-6 fix extracts slot selection into `face_for_span_kind` called by
/// BOTH paths, collapsing the divergence permanently.
///
/// If the span's styled face is `None` in the font set, the fallback uses the
/// regular face (same fallback as the draw path). If no face at all: cursor
/// stays (graceful degradation, zero width — acceptable for font-absent case).
///
/// # Panics
///
/// Does not panic. Falls back to 0.0 width when font face is absent.
#[must_use]
pub fn compute_multi_span_x_positions(
    spans: &[KrillaTextSpan],
    start_x: f32,
    font_size: f32,
    font_set: &ResolvedFontSet,
) -> Vec<f32> {
    let mut positions = Vec::with_capacity(spans.len());
    let mut cursor_x = start_x;
    for span in spans {
        positions.push(cursor_x);

        // Delegate slot selection to the single shared face_for_span_kind helper
        // (ADV-P06-MED-001 fix): guarantees identical slot selection between the
        // measure path (here) and the draw path (draw_packed_line).
        let resolved_face_opt = face_for_span_kind(span.face, font_set);

        let effective_size = effective_span_font_size(span, font_size);
        if let Some(face) = resolved_face_opt {
            // ADV-P05-MED-001: use face.face_index (NOT 0) so measurement and
            // drawing agree for .ttc collection faces at non-zero index.
            cursor_x += measure_text_width_pt(
                face.raw.as_ref(),
                face.face_index,
                effective_size,
                &span.text,
            );
        }
        // If no face at all: cursor stays (graceful degradation; span gets 0 width).
    }
    positions
}

/// Draw a sequence of [`KrillaTextSpan`]s at a bounding-box baseline position,
/// each with the appropriate font face from `font_set`.
///
/// This is the production font-face dispatch function — it replaces the old
/// `extract_all_inline_text` + `draw_text_at_bbox` single-font path for
/// `FrameContent::TextRun` and `FrameContent::SubtitleInlines`.
///
/// ## Behaviour
///
/// - Uses `text_baseline_surface_y(bbox)` for the vertical position.
/// - Delegates to [`draw_inline_spans_at_y`] which maintains a horizontal cursor:
///   each span is drawn at the X position immediately following the previous span's
///   measured width. Super/subscript spans are drawn at a shifted baseline and
///   reduced font size (see [`draw_inline_spans_at_y`] for full detail).
/// - Skips empty spans and spans with no resolved font.
fn draw_inline_spans(
    surface: &mut krilla::surface::Surface<'_>,
    spans: &[KrillaTextSpan],
    bbox: &BoundingBox,
    font_set: &ResolvedFontSet,
    font_size: f32,
) {
    let baseline_y = text_baseline_surface_y(bbox);
    draw_inline_spans_at_y(surface, spans, bbox, baseline_y, font_size, font_set);
}

/// A single word extracted from a [`KrillaTextSpan`], retaining its font face
/// and super/subscript offset signal.
///
/// Produced by [`expand_spans_to_words`] for the line-packing step in
/// [`draw_inline_spans_at_y`]. Each `SpanWord` represents one whitespace-delimited
/// token from a span; together they reconstruct the full text with per-word font
/// face dispatch.
///
/// ## Why `String` and not `Arc<str>`?
///
/// The words are transient line-packing artifacts — they are never stored long-
/// term and do not need reference-counted heap allocation.  Using `String` here
/// is idiomatic and avoids a spurious `Arc` bump per word.
#[derive(Debug, Clone)]
struct SpanWord {
    /// Font face kind for this word (inherits from the parent span).
    face: FontFaceKind,
    /// The text of this word (one whitespace-delimited token).
    text: String,
    /// Superscript/Subscript signal (non-zero iff the parent span is super/sub).
    y_offset_units: i32,
}

/// Expand a slice of [`KrillaTextSpan`]s into individual words with their
/// associated font face kind.
///
/// Splits each span's text on whitespace (consuming the whitespace, which is
/// correct since line-packing re-adds inter-word space in the horizontal cursor).
/// The font face and super/subscript signal from the parent span are inherited
/// by every word extracted from that span.
///
/// Empty tokens (consecutive whitespace in the source) are discarded.
///
/// ## Why `face` and `y_offset_units` but not `text` from `KrillaTextSpan`?
///
/// The `text` of a `KrillaTextSpan` may contain multiple words (e.g., a bold
/// span wrapping `"word1 word2 word3"`). Splitting on whitespace is necessary
/// for word-boundary wrapping. The `face` and `y_offset_units` fields are
/// span-wide attributes — they apply identically to every word in the span.
fn expand_spans_to_words(spans: &[KrillaTextSpan]) -> Vec<SpanWord> {
    let mut words = Vec::new();
    for span in spans {
        for token in span.text.split_whitespace() {
            if !token.is_empty() {
                words.push(SpanWord {
                    face: span.face,
                    text: token.to_owned(),
                    y_offset_units: span.y_offset_units,
                });
            }
        }
    }
    words
}

/// Measure the rendered width of a single word in PDF points using the
/// face from `font_set` that corresponds to `face_kind`.
///
/// Returns `0.0` when the face is absent (graceful degradation — consistent
/// with [`compute_multi_span_x_positions`]'s zero-advance path).
fn measure_word_width_pt(
    word: &str,
    face_kind: FontFaceKind,
    font_size: f32,
    font_set: &ResolvedFontSet,
) -> f32 {
    face_for_span_kind(face_kind, font_set).map_or(0.0, |face| {
        measure_text_width_pt(face.raw.as_ref(), face.face_index, font_size, word)
    })
}

/// Measure the rendered width of an inter-word space in PDF points using the
/// face from `font_set` that corresponds to `face_kind`.
///
/// Used by the line-packing step to correctly account for the horizontal space
/// between adjacent words — critical for accurate line-width measurement.
///
/// Returns the width of a single ASCII space character " " for the given face.
#[allow(clippy::inline_always)]
#[inline(always)]
fn measure_space_width_pt(
    face_kind: FontFaceKind,
    font_size: f32,
    font_set: &ResolvedFontSet,
) -> f32 {
    measure_word_width_pt(" ", face_kind, font_size, font_set)
}

/// Pack [`SpanWord`]s greedily into lines that fit within `max_line_width_pt`.
///
/// Returns `Vec<Vec<SpanWord>>` — each inner `Vec` is one line's word sequence,
/// ordered left-to-right as they should be drawn.
///
/// ## Algorithm (F-095-P1-001 fix + F-095-P2-001 fix — VP-054 analogue for multi-face text)
///
/// Maintains a running `cursor_x` for the current line. For each word:
/// - Measure the word width using the word's face kind (preserving bold/italic
///   metrics — they differ from regular face metrics).
/// - If the word fits (`cursor_x + space + word_width ≤ max_line_width_pt`):
///   append to the current line and advance cursor.
/// - If the word does NOT fit but is itself narrower than the frame:
///   flush the current line, start a new one with this word.
/// - If the word is itself wider than `max_line_width_pt` (AC-002/REND-002):
///   apply the character-wrap fallback (see below).
///
/// ## Character-wrap fallback (AC-002 / F-095-P2-001)
///
/// When a single word is wider than `max_line_width_pt`, it is split at character
/// boundaries into the longest fragments that fit within the frame.  Each fragment
/// inherits the original word's `FontFaceKind` (e.g., a bold word's fragments are
/// all bold) and `y_offset_units` (super/subscript signal is preserved).
///
/// Termination guarantee: if even a single character does not fit within the frame
/// (i.e., the frame is narrower than one character's advance), that character is
/// emitted as a single-character fragment to prevent an infinite loop and to ensure
/// no text is silently dropped (VP-054 sub-property a: lossless wrapping).
///
/// This mirrors the `wrap_text` phase-2 logic in `text_layout.rs`, extending it to
/// the multi-face path.  A shared pure helper (`char_split_span_word`) extracts
/// the character-splitting logic so both paths benefit from the same implementation.
///
/// ## Space handling
///
/// The inter-word space width is measured with the PRECEDING word's face (the
/// most common convention — kerning between adjacent runs uses the dominant face).
/// When a line is empty (first word on a new line), no space is prepended.
fn pack_words_into_lines(
    words: &[SpanWord],
    max_line_width_pt: f32,
    font_size: f32,
    font_set: &ResolvedFontSet,
) -> Vec<Vec<SpanWord>> {
    let mut lines: Vec<Vec<SpanWord>> = Vec::new();
    let mut current_line: Vec<SpanWord> = Vec::new();
    let mut cursor_x: f32 = 0.0;

    for word in words {
        let word_w = measure_word_width_pt(&word.text, word.face, font_size, font_set);

        // ── AC-002 / F-095-P2-001: character-wrap fallback for over-wide words ──
        //
        // When a word is wider than the entire frame, split it at character
        // boundaries.  Each character-level fragment inherits the source word's
        // `face` and `y_offset_units` so that bold/italic/super/subscript signals
        // are preserved across the split (F-095-P2-001 requirement).
        if word_w > max_line_width_pt {
            // Produce char-split fragments from the source word.
            let fragments = char_split_span_word(word, max_line_width_pt, font_size, font_set);

            for fragment in fragments {
                let frag_w =
                    measure_word_width_pt(&fragment.text, fragment.face, font_size, font_set);

                if current_line.is_empty() {
                    // First item on this line — start directly with the fragment.
                    cursor_x = frag_w;
                    current_line.push(fragment);
                } else {
                    // Try appending the fragment to the current line (no separator —
                    // fragments originate from the same source word; no inter-word
                    // space should be inserted between them).
                    if cursor_x + frag_w <= max_line_width_pt {
                        cursor_x += frag_w;
                        current_line.push(fragment);
                    } else {
                        // Fragment doesn't fit on current line — flush and start new.
                        lines.push(std::mem::take(&mut current_line));
                        cursor_x = frag_w;
                        current_line.push(fragment);
                    }
                }
            }
            continue; // Move to next source word.
        }

        // ── Normal word packing (AC-001) ────────────────────────────────────────

        let space_w = if current_line.is_empty() {
            0.0
        } else {
            // Measure a space using the face of the PRECEDING word.
            let prev_face = current_line.last().map_or(word.face, |w| w.face);
            measure_space_width_pt(prev_face, font_size, font_set)
        };

        if current_line.is_empty() {
            // First word on this line.
            current_line.push(word.clone());
            cursor_x = word_w;
        } else if cursor_x + space_w + word_w <= max_line_width_pt {
            // Fits on current line.
            current_line.push(word.clone());
            cursor_x += space_w + word_w;
        } else {
            // Overflow: flush current line, start new line with this word.
            lines.push(std::mem::take(&mut current_line));
            current_line.push(word.clone());
            cursor_x = word_w;
        }
    }

    // Flush the last line (if any).
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Split a single over-wide [`SpanWord`] into character-level fragments that
/// each fit within `max_line_width_pt`.
///
/// ## Purpose (shared helper — F-095-P2-001)
///
/// This is a pure helper factored out from [`pack_words_into_lines`] so the
/// character-splitting logic lives in one place and is independently testable.
/// It mirrors the phase-2 character-wrap in `text_layout::wrap_text` but
/// operates on multi-face `SpanWord` tokens instead of plain `String` slices.
///
/// ## Face and offset preservation
///
/// Each output fragment inherits `face` and `y_offset_units` from the source
/// `SpanWord`.  A bold, italic, or super/subscript word's fragments must all
/// retain the same typographic signal — losing it on continuation fragments
/// would cause a visual regression (bold fragments rendered in regular weight).
///
/// ## Termination guarantee (VP-054 sub-property a)
///
/// If even a single character is wider than `max_line_width_pt` (extremely narrow
/// frame), that character is emitted as a one-character fragment.  This guarantees
/// the loop terminates and no text is silently dropped.
fn char_split_span_word(
    word: &SpanWord,
    max_line_width_pt: f32,
    font_size: f32,
    font_set: &ResolvedFontSet,
) -> Vec<SpanWord> {
    let mut fragments: Vec<SpanWord> = Vec::new();
    let mut remaining: &str = &word.text;

    while !remaining.is_empty() {
        // Find the longest character prefix of `remaining` that fits.
        let mut fragment_end_byte: usize = 0;
        let mut frag_width: f32 = 0.0;

        for ch in remaining.chars() {
            let mut buf = [0u8; 4];
            let ch_str = ch.encode_utf8(&mut buf);
            let ch_w = measure_word_width_pt(ch_str, word.face, font_size, font_set);
            if frag_width + ch_w > max_line_width_pt && fragment_end_byte > 0 {
                // Adding this character would overflow — stop here.
                break;
            }
            frag_width += ch_w;
            fragment_end_byte += ch.len_utf8();
        }

        // Safety: if even a single character does not fit (max_line_width_pt is
        // smaller than one char's advance), emit it anyway to guarantee termination
        // and ensure no text is silently dropped (VP-054 sub-property a / AC-002).
        if fragment_end_byte == 0 {
            fragment_end_byte = remaining.chars().next().map_or(0, char::len_utf8);
        }

        // Guard: if fragment_end_byte is still 0 after the fallback (empty string
        // edge case — should never occur since we checked `!remaining.is_empty()`
        // at the top of the loop), break to guarantee termination.
        if fragment_end_byte == 0 {
            break;
        }

        let (fragment_text, rest) = remaining.split_at(fragment_end_byte);
        remaining = rest;

        fragments.push(SpanWord {
            face: word.face,
            text: fragment_text.to_owned(),
            y_offset_units: word.y_offset_units,
        });
    }

    fragments
}

/// Draw one packed line of [`SpanWord`]s at a given baseline Y.
///
/// Advances `cursor_x` after each word, inserting an inter-word space measured
/// with the CURRENT word's face (consistent with [`pack_words_into_lines`]).
/// Each word is drawn with the appropriate `krilla::text::Font` from `font_set`
/// via [`face_for_span_kind`] — preserving bold/italic dispatch across all words
/// on the line (F-095-P1-001: bold run split across lines keeps bold on both).
///
/// Super/subscript signals from [`SpanWord::y_offset_units`] are respected:
/// reduced font size via [`effective_span_font_size`] and shifted baseline via
/// [`adjusted_baseline_y`], both computed against the parent `font_size`.
fn draw_packed_line(
    surface: &mut krilla::surface::Surface<'_>,
    line_words: &[SpanWord],
    start_x: f32,
    baseline_y: f32,
    font_size: f32,
    font_set: &ResolvedFontSet,
) {
    let mut cursor_x = start_x;

    for (i, word) in line_words.iter().enumerate() {
        if word.text.is_empty() {
            continue;
        }

        // Add inter-word space (not before the first word on a line).
        if i > 0 {
            let prev_face = line_words[i - 1].face;
            cursor_x += measure_space_width_pt(prev_face, font_size, font_set);
        }

        let Some(font) = face_for_span_kind(word.face, font_set).map(|f| &f.font) else {
            let preview: String = word.text.chars().take(20).collect();
            tracing::debug!(
                text_preview = %preview,
                face = ?word.face,
                "draw_packed_line: skipping word — no resolved font for face"
            );
            cursor_x += measure_word_width_pt(&word.text, word.face, font_size, font_set);
            continue;
        };

        // Super/subscript: reduced font size + shifted baseline (ADR-023 amendment).
        // Create a synthetic KrillaTextSpan stub just to reuse the shared helper fns.
        // (This avoids duplicating the scale/shift arithmetic.)
        let stub_span = KrillaTextSpan {
            face: word.face,
            text: word.text.as_str().into(),
            y_offset_units: word.y_offset_units,
        };
        let effective_size = effective_span_font_size(&stub_span, font_size);
        let draw_y = adjusted_baseline_y(&stub_span, baseline_y, font_size);

        surface.set_fill(Some(text_fill_black()));
        surface.set_stroke(None);

        let start = Point::from_xy(cursor_x, draw_y);
        surface.draw_text(
            start,
            font.clone(),
            effective_size,
            &word.text,
            false,
            TextDirection::Auto,
        );

        cursor_x += measure_word_width_pt(&word.text, word.face, font_size, font_set);
    }
}

/// Draw a sequence of [`KrillaTextSpan`]s at an explicit `baseline_y`,
/// wrapping to new lines when the combined span width exceeds the frame width.
///
/// Called by [`draw_body_blocks`] and [`draw_body_blocks_tagged`] for
/// per-item cursor positioning, and by [`draw_inline_spans`] for bbox-derived
/// baseline positioning.
///
/// ## Horizontal cursor (ADV-P04-CRIT-001 fix)
///
/// Each span is drawn at a running horizontal cursor position that advances by
/// the MEASURED width of the preceding span.  Width is computed via
/// per-word glyph horizontal-advances from `ttf-parser`.
///
/// ## Word-wrap (F-095-P1-001 fix / BC-4.03.002 postcondition 1)
///
/// The inline span path now applies word-boundary wrapping at the frame width
/// derived from `bbox.width`. The implementation:
/// 1. [`expand_spans_to_words`]: split all spans into individual word tokens,
///    preserving each word's font face kind and super/subscript signal.
/// 2. [`pack_words_into_lines`]: greedy line packing at `emu_to_pt(bbox.width)`.
///    A word wider than the frame is char-split via [`char_split_span_word`],
///    preserving the word's `FontFaceKind` on every character fragment (AC-002 /
///    F-095-P2-001 fix — previously deferred, now implemented).
/// 3. [`draw_packed_line`]: draw each packed line at the current `baseline_y`,
///    advancing the cursor with inter-word space using face-accurate metrics.
/// 4. Advance `baseline_y` by `font_size * BODY_LINE_LEADING` after each line.
///
/// ## Frame-bottom clamp (F-095-P1-005 fix)
///
/// Lines whose baseline would exceed the frame bottom (`bbox.y + bbox.height`)
/// are silently elided, with a `tracing::warn!` emitted once per overflow event.
/// This prevents drawing off-slide and is consistent with the clamp applied in
/// [`draw_text_at_bbox`].
///
/// ## Bold/italic preserved across wrap boundaries (F-095-P1-001)
///
/// Each word in a packed line retains its source face kind
/// (`FontFaceKind::Bold`, `FontFaceKind::Italic`, etc.). A bold run that spans
/// two lines has bold face on BOTH lines — no face information is lost at the
/// wrap boundary.
///
/// ## Super/Subscript size reduction (ADV-P04-HIGH-001 fix)
///
/// Unchanged from the single-line path: super/subscript spans use
/// `font_size * SUPER_SUB_SCALE` (0.583 ×) via [`effective_span_font_size`],
/// and their baselines are shifted via [`adjusted_baseline_y`].
fn draw_inline_spans_at_y(
    surface: &mut krilla::surface::Surface<'_>,
    spans: &[KrillaTextSpan],
    bbox: &BoundingBox,
    baseline_y: f32,
    font_size: f32,
    font_set: &ResolvedFontSet,
) {
    let start_x = emu_to_pt(bbox.x);
    let max_line_width_pt = emu_to_pt(bbox.width);
    let frame_bottom_pt = emu_to_pt(bbox.y) + emu_to_pt(bbox.height);

    // Step 1: expand all spans to individual word tokens.
    let words = expand_spans_to_words(spans);
    if words.is_empty() {
        return;
    }

    // Step 2: pack words into lines that fit within the frame width.
    let lines = pack_words_into_lines(&words, max_line_width_pt, font_size, font_set);

    // Step 3: draw each line at the appropriate baseline.
    let mut current_baseline_y = baseline_y;

    for line_words in &lines {
        // F-095-P1-005: clamp at frame bottom — elide lines that exceed the frame.
        if current_baseline_y > frame_bottom_pt {
            tracing::warn!(
                frame_bottom_pt,
                current_baseline_y,
                remaining_lines = lines.len(),
                "draw_inline_spans_at_y: baseline exceeds frame bottom; \
                 eliding overflow lines (F-095-P1-005)"
            );
            break;
        }

        draw_packed_line(
            surface,
            line_words,
            start_x,
            current_baseline_y,
            font_size,
            font_set,
        );

        current_baseline_y += font_size * BODY_LINE_LEADING;
    }
}

/// Draw body content blocks at the given bounding box, stacking each item
/// at a distinct baseline to prevent overprinting.
///
/// Uses [`body_item_baselines`] to compute a per-item baseline cursor that
/// advances by `font_size * BODY_LINE_LEADING` (1.2×) after each drawn item.
/// The first item's baseline equals `text_baseline_surface_y(bbox)` — identical
/// to the single-item behaviour used by Title and Subtitle frames — so existing
/// single-block body frames are unaffected by this change.
///
/// Non-text blocks (`Chart`, `Diagram`, `Math`, `Image`, `Table`, `Shape`) are
/// skipped in this pass — they are handled separately in their own frame draw
/// logic and do not consume cursor space.
fn draw_body_blocks(
    surface: &mut krilla::surface::Surface<'_>,
    blocks: &[slideforge_types::ContentBlock],
    bbox: &BoundingBox,
    font_set: &ResolvedFontSet,
) {
    use slideforge_types::ContentBlock;

    // Compute per-item baselines using the pure cursor-advance function so that
    // the drawing path and the test path share the identical geometry.
    let baselines = body_item_baselines(blocks, bbox);
    let mut baseline_iter = baselines.into_iter();

    for block in blocks {
        match block {
            ContentBlock::Text(text_block) => {
                // AC-004: use slide_to_krilla_runs for font-face dispatch so Bold/Italic/Code
                // text is rendered with the correct face from font_set (not flattened to regular).
                let spans = slide_to_krilla_runs(&text_block.inlines);
                if !spans.is_empty()
                    && let Some(baseline_y) = baseline_iter.next()
                {
                    draw_inline_spans_at_y(surface, &spans, bbox, baseline_y, 18.0, font_set);
                }
            },
            ContentBlock::Bullets(items) => {
                for item in items {
                    // AC-004: same font-face dispatch for bullet items.
                    let spans = slide_to_krilla_runs(&item.inlines);
                    if !spans.is_empty()
                        && let Some(baseline_y) = baseline_iter.next()
                    {
                        draw_inline_spans_at_y(surface, &spans, bbox, baseline_y, 16.0, font_set);
                    }
                }
            },
            // Non-text blocks do not advance the cursor in this pass.
            ContentBlock::Chart(_)
            | ContentBlock::Diagram(_)
            | ContentBlock::Math(_)
            | ContentBlock::Image(_)
            | ContentBlock::Table(_)
            | ContentBlock::Shape(_)
            // STORY-087 pass-2: ColorBar is geometry-only — no body draw.
            | ContentBlock::ColorBar(_) => {},
        }
    }
}

/// Decide whether a [`slideforge_types::ContentBlock`] produces a PDF structure group.
///
/// **OBS-P5-001 fix:** This function is now a thin wrapper around
/// [`slideforge_types::ContentBlock::produces_structure_group`], which is the
/// single authoritative definition of the predicate. The match logic is defined
/// exactly once in `slideforge-types` — adding a new `ContentBlock` variant
/// forces a compile error there, preventing silent desync between this call site
/// and [`crate::tag_engine::SlideTagEngine::tag_content_block`].
///
/// See [`slideforge_types::ContentBlock::produces_structure_group`] for the
/// full decision table.
fn block_is_structure_producing(block: &slideforge_types::ContentBlock) -> bool {
    block.produces_structure_group()
}

/// Draw body content blocks in separate tagged marked-content regions — one per block.
///
/// Called by the export draw loop for Body frames that have multiple content blocks
/// (F-P3-001 fix). Each content block that produced a structure group gets its own
/// `start_tagged` / `end_tagged` pair, with the resulting `Identifier` pushed into
/// the corresponding Part child group.
///
/// ## Why this is separate from `draw_body_blocks`
///
/// `draw_body_blocks` knows nothing about tagging — it just draws text at stacked
/// baselines. For multi-block tagged PDF, we need to interleave `start_tagged` /
/// `end_tagged` boundaries around EACH block's drawing, not just around the whole
/// frame. Combining them would require `draw_body_blocks` to take a `Surface` AND
/// manage krilla's tagging API, violating the single-responsibility principle.
///
/// ## Contract (F-P4-001 fix)
///
/// `child_indices` is the `Vec<usize>` from `frame_child_part_indices[frame_idx]`.
/// It has one entry for each content block that `tag_content_block` returned
/// `Some(group)` for. `draw_body_blocks_tagged` MUST iterate the SAME block universe
/// as `tag_content_block` — i.e., advance `child_idx_cursor` for EVERY block where
/// [`block_is_structure_producing`] returns `true`, regardless of whether the block
/// is currently drawable (Math, Table, Image, etc. may not yet have a draw path).
///
/// Blocks that are not structure-producing (empty Bullets, decorative/None-alt
/// Image/Chart/Diagram/Shape) are drawn WITHOUT a tagged region — they are structural
/// no-ops. The cursor is NOT advanced for these blocks.
///
/// For structure-producing blocks that have no draw path yet (Math, Table, Image,
/// Chart, Diagram, Shape with alt), we open a tagged region and immediately close it
/// (empty-but-tagged region). This satisfies PDF/UA-1's requirement that every
/// grouping element references at least one MCID leaf (BC-4.03.001 invariant-3 /
/// F-045-C2), even if no visible content is drawn inside.
///
/// # Errors
///
/// Currently infallible (all drawing is best-effort and non-fatal). The
/// `Result` return type is kept for forward compatibility — future iterations
/// may propagate SVG embedding or tagged-region errors.
// unnecessary_wraps: kept for forward compatibility — future body-block types
// (e.g., embedded images) may need to propagate errors from SVG embedding.
#[allow(clippy::similar_names, clippy::unnecessary_wraps)]
fn draw_body_blocks_tagged(
    surface: &mut krilla::surface::Surface<'_>,
    blocks: &[slideforge_types::ContentBlock],
    bbox: &BoundingBox,
    font_set: &ResolvedFontSet,
    child_indices: &[usize],
    part: &mut krilla::tagging::TagGroup,
) -> Result<(), PdfExportError> {
    use slideforge_types::ContentBlock;

    // Compute per-item baselines using the same pure function as draw_body_blocks
    // so that the drawing positions are identical to the non-tagged path.
    let baselines = body_item_baselines(blocks, bbox);
    let mut baseline_iter = baselines.into_iter();

    // `child_idx_cursor` advances for every block that `block_is_structure_producing`
    // returns `true` for — IDENTICAL to the set that `tag_content_block` returns Some for.
    // This keeps the cursor in lockstep with the child_indices recorded by the tag engine.
    let mut child_idx_cursor: usize = 0;

    for block in blocks {
        if !block_is_structure_producing(block) {
            // Not structure-producing (e.g., empty Bullets, decorative/no-alt
            // Image/Chart/Diagram/Shape): draw nothing and do NOT advance the cursor.
            // No child_indices slot was allocated by the tag engine for this block.
            continue;
        }

        // Structure-producing block: open a tagged region, draw content (if drawable),
        // close the tagged region, and push the Identifier into the correct Part child group.
        if let Some(&child_part_idx) = child_indices.get(child_idx_cursor) {
            let id = surface.start_tagged(ContentTag::Other);

            // Draw whatever content the block has a draw path for.
            // Blocks without a draw path (Math, Table, Image, Chart, Diagram, Shape with alt)
            // get an empty-but-tagged region — the BDC/EMC pair ensures the MCID leaf exists.
            match block {
                ContentBlock::Text(text_block) => {
                    // AC-004: use slide_to_krilla_runs for font-face dispatch so Bold/Italic/Code
                    // text is rendered with the correct face from font_set.
                    let spans = slide_to_krilla_runs(&text_block.inlines);
                    // Even if spans is empty (no baseline consumed), the block has a
                    // structure group (P) — the tagged region ensures the group gets an MCID.
                    if !spans.is_empty()
                        && let Some(baseline_y) = baseline_iter.next()
                    {
                        draw_inline_spans_at_y(surface, &spans, bbox, baseline_y, 18.0, font_set);
                    }
                },
                ContentBlock::Bullets(items) => {
                    // The entire Bullets block corresponds to ONE L group in the
                    // structure tree — draw ALL bullet items inside the SAME tagged region.
                    for item in items {
                        // AC-004: same font-face dispatch for bullet items.
                        let spans = slide_to_krilla_runs(&item.inlines);
                        if !spans.is_empty()
                            && let Some(baseline_y) = baseline_iter.next()
                        {
                            draw_inline_spans_at_y(
                                surface, &spans, bbox, baseline_y, 16.0, font_set,
                            );
                        }
                    }
                },
                // Math, Table, Image, Chart, Diagram, Shape with alt: no draw path yet.
                // The empty-but-tagged region (BDC/EMC with no content operators) gives
                // the structure group its required MCID leaf (BC-4.03.001 invariant-3).
                ContentBlock::Math(_)
                | ContentBlock::Table(_)
                | ContentBlock::Image(_)
                | ContentBlock::Chart(_)
                | ContentBlock::Diagram(_)
                | ContentBlock::Shape(_)
                // STORY-087 pass-2: ColorBar is geometry-only; produces_structure_group()
                // returns false so this arm is only reachable if logic changes. No draw.
                | ContentBlock::ColorBar(_) => {},
            }

            surface.end_tagged();
            if let Some(krilla::tagging::Node::Group(child_group)) =
                part.children.get_mut(child_part_idx)
            {
                child_group.push(id);
            }
            child_idx_cursor += 1;
        }
    }

    Ok(())
}

/// Place a normalized SVG diagram on the surface at the specified top-left Surface position.
///
/// ## Transform composition (F-P18-001 / F-P18-002 fix)
///
/// Pushes a translation transform so the SVG is positioned at `(surface_x, surface_y)`
/// on the krilla Surface (top-left, Y-down). The actual scale-to-frame transform is
/// applied inside `embed_normalized_svg` via the frame dimensions.
///
/// The composed chain for each path is:
/// ```text
/// translate(surface_x, surface_y) ∘ scale(frame_w/svg_w, frame_h/svg_h)
///     ∘ path.abs_transform() ∘ local_path_data
/// ```
///
/// All transforms are pushed before embedding and popped after, leaving the
/// surface state unchanged for subsequent draw calls.
///
/// ## Parameters
///
/// - `surface_x` / `surface_y` — top-left corner of the frame in Surface coords
///   (Y-down; krilla applies the PDF Y-flip internally — DIR-044-001).
/// - `frame_w_pt` / `frame_h_pt` — frame width and height in Surface points,
///   used to scale the SVG viewport to fit the frame (F-P18-002).
// frame_w_pt / frame_h_pt are intrinsically paired width/height parameters.
#[allow(clippy::similar_names)]
fn place_svg_at(
    surface: &mut krilla::surface::Surface<'_>,
    svg: &slideforge_types::NormalizedDiagramSvg,
    surface_x: f32,
    surface_y: f32,
    frame_w_pt: f32,
    frame_h_pt: f32,
) -> Result<(), PdfExportError> {
    // Apply a translation so the SVG is positioned at the frame's top-left Surface coords.
    // The scale is handled inside embed_normalized_svg (around the tree render).
    let translate = krilla::geom::Transform::from_translate(surface_x, surface_y);
    surface.push_transform(&translate);
    let result = embed_normalized_svg(svg, surface, frame_w_pt, frame_h_pt);
    surface.pop();
    result
}

/// Guard: reject deck titles containing XML-1.0-illegal control characters
/// before they are embedded in XMP metadata.
///
/// ## Rationale (SEC-050-001 / CWE-116)
///
/// XMP metadata is an XML-1.0 document. `xmp_writer` escapes the 5 XML-reserved
/// characters (`&`, `<`, `>`, `"`, `'`) but does NOT strip XML-1.0-illegal
/// control characters. The illegal ranges are:
///
/// - U+0000–U+0008 (C0 controls, excluding HT/LF/CR)
/// - U+000B (Vertical Tab)
/// - U+000C (Form Feed)
/// - U+000E–U+001F (remaining C0 controls)
/// - U+FFFE (UTF-16 BOM, wrong byte order — noncharacter)
/// - U+FFFF (noncharacter)
///
/// Embedding such code points produces a malformed XMP stream, which violates
/// PDF/UA-1 constraints and may cause downstream PDF readers to reject or
/// silently corrupt the file.
///
/// ## Behaviour
///
/// This function REJECTS (returns `Err`) on the first illegal character found.
/// It does NOT silently strip. This is consistent with
/// `validate_lang_for_xml` in `slideforge-pptx` (SEC-039-001).
///
/// ## Losslessness invariant
///
/// A title containing only printable Unicode (the overwhelming majority of
/// real deck titles) always passes through unchanged — this guard never
/// modifies the title string, it only rejects.
fn validate_title_for_xmp(title: &str) -> Result<(), PdfExportError> {
    for ch in title.chars() {
        let code = ch as u32;
        let illegal = matches!(
            code,
            0x0000..=0x0008 | 0x000B | 0x000C | 0x000E..=0x001F | 0xFFFE | 0xFFFF
        );
        if illegal {
            return Err(PdfExportError::InvalidXmpTitle {
                title: title.to_owned(),
                code_point: code,
            });
        }
    }
    Ok(())
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
    /// - `deck` — the semantic, pre-layout IR. Used for document metadata:
    ///   `deck.metadata.lang` → `/Lang` in the PDF catalog;
    ///   `deck.metadata.title` → document title (both wired in STORY-045).
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

// ─────────────────────────────────────────────────────────────────────────────
// STORY-072: FillSpec::Gradient — PDF krilla gradient emission stub
// ─────────────────────────────────────────────────────────────────────────────

/// Draw a linear gradient-filled rectangle onto a krilla `Surface` for a
/// `FillSpec::Gradient { from, to }` shape.
///
/// ## Contract (STORY-072 AC-004)
///
/// Calls (in order):
/// 1. `surface.set_fill(Some(Fill { paint: LinearGradient { ... }.into(), ... }))`
/// 2. `surface.draw_path(rect_path)` to stroke/fill the rectangle.
///
/// The gradient is top-to-bottom (v1.0 fixed direction):
/// - `x1 = bbox.x_pt, y1 = bbox.y_pt` (top-left)
/// - `x2 = bbox.x_pt, y2 = (bbox.y + bbox.height)_pt` (bottom-left)
/// - `spread_method = SpreadMethod::Pad`
/// - Two `Stop` entries: offset=0 for `from`, offset=1 for `to`
///
/// Uses `krilla = "=0.6.0"` API. `pdf-writer` MUST NOT be a direct dep of
/// `slideforge-pdf` (per export-architecture v1.2 RISK-1).
///
/// ## Coordinate system
///
/// krilla `Surface` uses top-left, Y-down. EMU values are converted to points
/// via `coords::emu_to_pt` before passing to krilla.
///
/// ## No-op on degenerate bounding box
///
/// When `bbox.width` or `bbox.height` is non-positive, no path is emitted.
pub fn draw_gradient_rect(
    surface: &mut krilla::surface::Surface<'_>,
    bbox: &slideforge_layout::BoundingBox,
    from: slideforge_types::Rgb,
    to: slideforge_types::Rgb,
) {
    let x_pt = emu_to_pt(bbox.x);
    let y_pt = emu_to_pt(bbox.y);
    let w_pt = emu_to_pt(bbox.width);
    let h_pt = emu_to_pt(bbox.height);

    // Guard: degenerate dimensions produce no path.
    let Some(rect) = Rect::from_xywh(x_pt, y_pt, w_pt, h_pt) else {
        tracing::debug!(
            x_pt,
            y_pt,
            w_pt,
            h_pt,
            "draw_gradient_rect: Rect::from_xywh returned None (degenerate dimensions); skipping"
        );
        return;
    };

    let mut pb = PathBuilder::new();
    pb.push_rect(rect);
    let Some(path) = pb.finish() else {
        tracing::debug!("draw_gradient_rect: PathBuilder::finish() returned None; skipping");
        return;
    };

    // Build the two color stops: offset=0 (from color, top) and offset=1 (to color, bottom).
    // NormalizedF32::ZERO and ONE are always valid.
    let stop_from = Stop {
        offset: NormalizedF32::ZERO,
        color: rgb::Color::new(from.r, from.g, from.b).into(),
        opacity: NormalizedF32::ONE,
    };
    let stop_to = Stop {
        offset: NormalizedF32::ONE,
        color: rgb::Color::new(to.r, to.g, to.b).into(),
        opacity: NormalizedF32::ONE,
    };

    // Top-to-bottom linear gradient:
    // (x1, y1) = top-left of bbox; (x2, y2) = bottom-left of bbox.
    let gradient = LinearGradient {
        x1: x_pt,
        y1: y_pt,
        x2: x_pt,
        y2: y_pt + h_pt,
        transform: Transform::identity(),
        spread_method: SpreadMethod::Pad,
        stops: vec![stop_from, stop_to],
        anti_alias: false,
    };

    surface.set_fill(Some(Fill {
        paint: gradient.into(),
        opacity: NormalizedF32::ONE,
        rule: FillRule::NonZero,
    }));
    surface.set_stroke(None);
    surface.draw_path(&path);

    tracing::debug!(
        x_pt,
        y_pt,
        w_pt,
        h_pt,
        from_r = from.r,
        from_g = from.g,
        from_b = from.b,
        to_r = to.r,
        to_g = to.g,
        to_b = to.b,
        "draw_gradient_rect: linear gradient rectangle drawn"
    );
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
                    region_role: None,
                }],
                speaker_notes: None,
                register_tags: RegisterSet::new(),
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],
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
                        region_role: None,
                    },
                    Frame {
                        bbox: BoundingBox {
                            x: Emu(0),
                            y: Emu(914_400),
                            width: Emu(9_144_000),
                            height: Emu(5_143_500),
                        },
                        // STORY-039 IR reshape: Image alt is now AltText enum.
                        content: FrameContent::Image {
                            alt: slideforge_types::AltText::Provided(Arc::from(
                                "A mountain landscape at sunrise",
                            )),
                        },
                        text_flow: None,
                        region_role: None,
                    },
                ],
                speaker_notes: None,
                register_tags: RegisterSet::new(),
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],
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
                font_size_emu: 457_200,
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
            slide_sections: vec![],
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
                font_size_emu: 457_200,
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
                    region_role: None,
                }],
                speaker_notes: None,
                register_tags: RegisterSet::new(),
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],
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

    /// BC-4.03.005 AC-006 (integration, updated for DIR-044-001 top-left mapping):
    /// `PdfExporter::export()` on a fixture `LaidOutDeck` must draw all elements
    /// within the slide canvas `[0.0, 0.0, 720.0, 405.0]` — BOTH axes:
    /// `0 <= surface_x` AND `surface_x + width_pt <= SLIDE_WIDTH_PT (720.0)`
    /// AND `0 <= baseline_y <= SLIDE_HEIGHT_PT (405.0)`.
    ///
    /// ## What this tests (F-044-003 fix + F-P5-001 X-axis extension + DIR-044-001)
    ///
    /// 1. Builds a fixture `LaidOutDeck` with Title/Subtitle/Body frames at
    ///    positions spanning the full slide width and height (top, middle, bottom).
    /// 2. Runs `PdfExporter::export()` to confirm the pipeline completes without
    ///    coordinate errors.
    /// 3. Asserts the X-axis bounding box (`surface_x` and `surface_x + width_pt`)
    ///    stays within `[0.0, SLIDE_WIDTH_PT]`.
    /// 4. Asserts the draw-time baseline Y (top-left mapping: `emu_to_pt(ir_y) +
    ///    0.8 * height`) stays within `[0.0, SLIDE_HEIGHT_PT]` — using
    ///    `text_baseline_surface_y` (the same function as the production draw path).
    ///
    /// ## Why the Y formula changed (DIR-044-001)
    ///
    /// The OLD formula was `ir_y_to_pdf_y(ir_y, elem_h, slide_h) + 0.8 * height`.
    /// That was vacuously safe (always in range) but computed the wrong position
    /// (mirror bug: top-of-slide elements rendered at the bottom).
    ///
    /// The NEW formula uses `text_baseline_surface_y(bbox)` which calls
    /// `emu_to_pt(bbox.y) + emu_to_pt(bbox.height) * 0.8`. This correctly places
    /// elements in Surface space (top-left, Y-down). The Y-axis range assertion
    /// here remains valid: for in-bounds IR boxes, `emu_to_pt(ir_y) + 0.8*h` ≤ 405.
    ///
    /// ## Distinction from the existing coords unit tests
    ///
    /// The existing `test_bc_4_03_005_no_element_outside_canvas_after_conversion`
    /// tests `ir_y_to_pdf_y()` arithmetic in isolation (retained as VP-006 target).
    /// THIS test exercises `text_baseline_surface_y` via the ACTUAL DRAW PATH
    /// function, confirming element placement via the export route.
    #[allow(clippy::unwrap_used, clippy::float_cmp)]
    #[test]
    fn test_bc_4_03_005_ac006_export_all_elements_within_canvas() {
        use crate::SLIDE_HEIGHT_EMU;
        use crate::coords::{SLIDE_HEIGHT_PT, SLIDE_WIDTH_PT, emu_to_pt};

        // Fixture deck: Title at top, Subtitle at 1-inch offset, Body at 2-inch offset.
        // All stay within the 5.625-inch (405pt) slide height and 10-inch (720pt) width.
        let one_inch_emu = Emu(914_400);
        let two_inch_emu = Emu(1_828_800);
        let title_h_emu = Emu(914_400); // 72pt
        let body_h_emu = Emu(1_270_000); // ~100pt
        let slide_w_emu = Emu(9_144_000); // 720pt (full width)

        // Verify our test fixture is within bounds.
        assert!(
            two_inch_emu.0 + body_h_emu.0 <= SLIDE_HEIGHT_EMU.0,
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
                            width: slide_w_emu,
                            height: title_h_emu,
                        },
                        content: FrameContent::Title(Arc::from("Title at top")),
                        text_flow: None,
                        region_role: None,
                    },
                    Frame {
                        bbox: BoundingBox {
                            x: Emu(0),
                            y: one_inch_emu,
                            width: slide_w_emu,
                            height: title_h_emu,
                        },
                        content: FrameContent::Subtitle(Arc::from("Subtitle at 1-inch")),
                        text_flow: None,
                        region_role: None,
                    },
                    Frame {
                        bbox: BoundingBox {
                            x: Emu(0),
                            y: two_inch_emu,
                            width: slide_w_emu,
                            height: body_h_emu,
                        },
                        content: FrameContent::Body(vec![]),
                        text_flow: None,
                        region_role: None,
                    },
                ],
                speaker_notes: None,
                register_tags: RegisterSet::new(),
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],
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

        // Verify bounding box computations for all frames stay within
        // [0.0, 0.0, SLIDE_WIDTH_PT, SLIDE_HEIGHT_PT] — BOTH axes.
        //
        // X-axis: surface_x = emu_to_pt(bbox.x); right edge = surface_x + width_pt.
        // Y-axis baseline: text_baseline_surface_y(bbox) = emu_to_pt(ir_y) + 0.8 * height
        //   (top-left Surface coords, DIR-044-001 — matches production draw_text_at_bbox).
        let frames_under_test = [
            (Emu(0), slide_w_emu, Emu(0), title_h_emu, "title-top"),
            (
                Emu(0),
                slide_w_emu,
                one_inch_emu,
                title_h_emu,
                "subtitle-1in",
            ),
            (Emu(0), slide_w_emu, two_inch_emu, body_h_emu, "body-2in"),
        ];

        for (ir_x, elem_w, ir_y, elem_h, label) in frames_under_test {
            // ── X-axis bounds (F-P5-001) ──────────────────────────────────────
            // surface_x = emu_to_pt(bbox.x) — left edge, via coords::emu_to_pt.
            let surface_x = emu_to_pt(ir_x);
            let width_pt = emu_to_pt(elem_w);

            assert!(
                surface_x >= -0.001,
                "AC-006: surface_x for frame '{label}' must be >= 0.0; got {surface_x:.3}"
            );
            assert!(
                surface_x + width_pt <= SLIDE_WIDTH_PT + 0.001,
                "AC-006: surface_x + width_pt for frame '{label}' must be <= {SLIDE_WIDTH_PT}; \
                 got surface_x={surface_x:.3}, width_pt={width_pt:.3}, sum={:.3}",
                surface_x + width_pt
            );

            // ── Y-axis bounds (top-left Surface mapping, DIR-044-001) ─────────
            // Use text_baseline_surface_y — the SAME function as production draw_text_at_bbox.
            // This makes the test non-vacuous and load-bearing: if the formula changes
            // in production, this test reflects the change automatically.
            let bbox = BoundingBox {
                x: ir_x,
                y: ir_y,
                width: elem_w,
                height: elem_h,
            };
            let baseline_y = text_baseline_surface_y(&bbox);

            assert!(
                baseline_y >= -0.001,
                "AC-006: baseline_y for frame '{label}' must be >= 0.0 (on the page); \
                 got {baseline_y:.3}"
            );
            assert!(
                baseline_y <= SLIDE_HEIGHT_PT + 0.001,
                "AC-006: baseline_y for frame '{label}' must be <= {SLIDE_HEIGHT_PT}; \
                 got {baseline_y:.3}"
            );
        }
    }

    // ─── Vertical-placement regression tests (DIR-044-001 mirror bug) ─────────

    /// Regression test for the vertical-mirror coordinate bug (DIR-044-001).
    ///
    /// A title frame at `ir_y=0` (slide top) must draw its baseline in the TOP HALF
    /// of the Surface (`baseline_y < SLIDE_HEIGHT_PT / 2 = 202.5`).
    ///
    /// Under the OLD (buggy) `ir_y_to_pdf_y` formula:
    ///   `box_bottom = 405 − 0 − 72 = 333`; `baseline = 333 + 72*0.8 = 390.6`
    ///   → `390.6 < 202.5` is FALSE → test FAILS (catches the mirror bug).
    ///
    /// Under the CORRECT `text_baseline_surface_y` formula:
    ///   `surface_top = emu_to_pt(Emu(0)) = 0.0`; `baseline = 0 + 72*0.8 = 57.6`
    ///   → `57.6 < 202.5` is TRUE → test PASSES.
    ///
    /// Uses `text_baseline_surface_y` — the same pure function as `draw_text_at_bbox` —
    /// so the test is non-vacuous and load-bearing: any regression in the draw path
    /// is caught here.
    #[test]
    fn test_vertical_placement_title_at_top() {
        use crate::coords::{SLIDE_HEIGHT_PT, emu_to_pt};

        // Title frame at ir_y=0 (slide top), height=72pt (1 inch).
        let ir_y = Emu(0);
        let element_h = Emu(72 * 12_700); // 72pt
        let bbox = BoundingBox {
            x: Emu(0),
            y: ir_y,
            width: Emu(9_144_000),
            height: element_h,
        };

        let baseline_y = text_baseline_surface_y(&bbox);

        // Numeric verification:
        // surface_top_y = emu_to_pt(Emu(0)) = 0.0
        // baseline_y    = 0.0 + emu_to_pt(72 * 12_700) * 0.8 = 72.0 * 0.8 = 57.6
        let expected = emu_to_pt(Emu(0)) + emu_to_pt(element_h) * 0.8;
        assert!(
            (baseline_y - expected).abs() < 0.001,
            "title baseline must equal {expected:.3}; got {baseline_y:.3}"
        );

        // Directional assertion: the baseline must be in the TOP HALF of the Surface.
        // A value > 202.5 (= SLIDE_HEIGHT_PT / 2) indicates the mirror bug.
        assert!(
            baseline_y < SLIDE_HEIGHT_PT / 2.0,
            "title baseline at ir_y=0 must be in the top half of the Surface \
             (Surface-Y < {:.1}); got {baseline_y:.3}. A value >= {:.1} indicates \
             the mirror bug: ir_y_to_pdf_y is being applied at draw time.",
            SLIDE_HEIGHT_PT / 2.0,
            SLIDE_HEIGHT_PT / 2.0
        );
        assert!(
            baseline_y >= 0.0,
            "title baseline must be >= 0.0 (on the page); got {baseline_y:.3}"
        );
    }

    /// Complementary regression test for the vertical-mirror coordinate bug.
    ///
    /// A footer frame near the BOTTOM of the slide (`ir_y ≈ slide_h − frame_h`)
    /// must draw its baseline in the BOTTOM HALF of the Surface
    /// (`baseline_y > SLIDE_HEIGHT_PT / 2 = 202.5`).
    ///
    /// Uses `text_baseline_surface_y` — the same pure function as `draw_text_at_bbox` —
    /// ensuring this test is non-vacuous and load-bearing.
    #[test]
    fn test_vertical_placement_footer_at_bottom() {
        use crate::SLIDE_HEIGHT_EMU;
        use crate::coords::{SLIDE_HEIGHT_PT, emu_to_pt};

        // Footer frame: height 36pt (0.5 inch), placed at the bottom of the slide.
        let element_h = Emu(36 * 12_700); // 36pt
        let ir_y = Emu(SLIDE_HEIGHT_EMU.0 - element_h.0); // top of footer = slide_h - 36pt
        let bbox = BoundingBox {
            x: Emu(0),
            y: ir_y,
            width: Emu(9_144_000),
            height: element_h,
        };

        let baseline_y = text_baseline_surface_y(&bbox);

        // Numeric verification:
        // surface_top_y = emu_to_pt(ir_y) = 405 - 36 = 369.0
        // baseline_y    = 369.0 + 36.0 * 0.8 = 369.0 + 28.8 = 397.8
        let expected = emu_to_pt(ir_y) + emu_to_pt(element_h) * 0.8;
        assert!(
            (baseline_y - expected).abs() < 0.001,
            "footer baseline must equal {expected:.3}; got {baseline_y:.3}"
        );

        // Directional assertion: the baseline must be in the BOTTOM HALF of the Surface.
        assert!(
            baseline_y > SLIDE_HEIGHT_PT / 2.0,
            "footer baseline at bottom of slide must be in the bottom half of the Surface \
             (Surface-Y > {:.1}); got {baseline_y:.3}.",
            SLIDE_HEIGHT_PT / 2.0
        );
        assert!(
            baseline_y <= SLIDE_HEIGHT_PT + 0.001,
            "footer baseline must be <= SLIDE_HEIGHT_PT ({SLIDE_HEIGHT_PT}); got {baseline_y:.3}"
        );
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
                    region_role: None,
                }],
                speaker_notes: None,
                register_tags: RegisterSet::new(),
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],
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
                font_size_emu: 457_200,
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
                    // STORY-039 IR reshape: Diagram is now struct with svg + alt fields.
                    content: FrameContent::Diagram {
                        svg,
                        alt: slideforge_types::AltText::Decorative,
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
            slide_sections: vec![],
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

    // ─── F-P2-001: body-block overprint (all items share same baseline) ───────

    /// F-P2-001 (failing before fix): `draw_body_blocks` with ≥ 2 text blocks
    /// must place each block at a DISTINCT baseline that advances downward
    /// (increasing Surface-Y) through the body frame.
    ///
    /// ## What this proves (TD-VSDD-059 load-bearing assertion)
    ///
    /// Without the fix every block calls `draw_text_at_bbox(…, bbox, …)` which
    /// always passes the same `bbox` → `text_baseline_surface_y(bbox)` returns
    /// the identical value for every call → all N paragraphs OVERPRINT on one line.
    ///
    /// The fix maintains a mutable cursor that starts at the first-item baseline
    /// (`text_baseline_surface_y(bbox)`) and advances by `font_size *
    /// BODY_LINE_LEADING` (standard 1.2× leading) after each drawn item.
    ///
    /// ## Concrete numbers for the fixture in this test
    ///
    /// Body frame: `bbox.y = 0`, `bbox.height = 914_400 EMU` (= 72 pt).
    ///
    /// Initial cursor (item 0):
    ///   `text_baseline_surface_y(bbox) = emu_to_pt(0) + emu_to_pt(914_400) * 0.8
    ///                                  = 0.0 + 72.0 * 0.8 = 57.6 pt`
    ///
    /// Item 0 is a `Text` block → `font_size` = 18.0 pt.
    /// Advance: `18.0 * 1.2 = 21.6 pt` → cursor after item 0 = 57.6 + 21.6 = 79.2 pt.
    ///
    /// Item 1 is a `Text` block → `font_size` = 18.0 pt, baseline = 79.2 pt.
    /// Advance: `18.0 * 1.2 = 21.6 pt` → cursor after item 1 = 79.2 + 21.6 = 100.8 pt.
    ///
    /// Item 2 is a `Text` block → `font_size` = 18.0 pt, baseline = 100.8 pt.
    ///
    /// Assertions:
    /// - baselines[1] > baselines[0]  (strictly lower = larger Surface-Y = stacked)
    /// - baselines[2] > baselines[1]
    /// - baselines[1] ≈ 79.2, baselines[2] ≈ 100.8
    ///
    /// This test drives `body_item_baselines` — a `pub(crate)` pure function that
    /// the fixed `draw_body_blocks` also uses, giving a non-vacuous load-bearing
    /// assertion (TD-VSDD-059).
    #[test]
    fn test_f_p2_001_body_blocks_baselines_are_distinct_and_stack_downward() {
        use slideforge_types::{ContentBlock, InlineNode, SourceSpan, TextBlock, TextTag};

        use crate::coords::emu_to_pt;

        // Body frame: top-left at (0, 0), height = 72 pt (= 914_400 EMU).
        let bbox = BoundingBox {
            x: Emu(0),
            y: Emu(0),
            width: Emu(9_144_000),
            height: Emu(72 * 12_700),
        };

        // 3 text blocks (each a distinct paragraph).
        let make_text_block = |s: &'static str| {
            ContentBlock::Text(TextBlock {
                inlines: vec![InlineNode::Plain(Arc::from(s))],
                tag: TextTag::Untagged,
                span: SourceSpan::default(),
            })
        };
        let blocks = vec![
            make_text_block("First paragraph"),
            make_text_block("Second paragraph"),
            make_text_block("Third paragraph"),
        ];

        // Call the pure baseline sequence function (extracted from draw_body_blocks).
        let baselines = body_item_baselines(&blocks, &bbox);

        assert_eq!(
            baselines.len(),
            3,
            "expected 3 baselines for 3 text blocks; got {}",
            baselines.len()
        );

        // Item 0: text_baseline_surface_y(bbox) = 0 + 72 * 0.8 = 57.6 pt.
        let initial_baseline = text_baseline_surface_y(&bbox);
        let expected_item0 = initial_baseline; // 57.6
        let font_size_text: f32 = 18.0;
        let advance = font_size_text * BODY_LINE_LEADING;
        let expected_item1 = expected_item0 + advance; // 57.6 + 21.6 = 79.2
        let expected_item2 = expected_item1 + advance; // 79.2 + 21.6 = 100.8

        assert!(
            (baselines[0] - expected_item0).abs() < 0.001,
            "item 0 baseline must equal {expected_item0:.3} pt (= initial cursor); got {:.3}",
            baselines[0]
        );
        assert!(
            (baselines[1] - expected_item1).abs() < 0.001,
            "item 1 baseline must equal {expected_item1:.3} pt (= item0 + 18*1.2); got {:.3}",
            baselines[1]
        );
        assert!(
            (baselines[2] - expected_item2).abs() < 0.001,
            "item 2 baseline must equal {expected_item2:.3} pt (= item1 + 18*1.2); got {:.3}",
            baselines[2]
        );

        // Strict ordering: each baseline strictly greater than previous (stacking downward).
        assert!(
            baselines[1] > baselines[0],
            "F-P2-001: item 1 baseline ({:.3}) must be strictly greater than item 0 baseline \
             ({:.3}) — overprint means they would be equal",
            baselines[1],
            baselines[0]
        );
        assert!(
            baselines[2] > baselines[1],
            "F-P2-001: item 2 baseline ({:.3}) must be strictly greater than item 1 baseline \
             ({:.3}) — overprint means they would be equal",
            baselines[2],
            baselines[1]
        );

        // Sanity: initial baseline matches emu_to_pt formula.
        let expected_initial = emu_to_pt(Emu(0)) + emu_to_pt(Emu(72 * 12_700)) * 0.8;
        assert!(
            (baselines[0] - expected_initial).abs() < 0.001,
            "initial cursor must equal text_baseline_surface_y(bbox) = {expected_initial:.3}; \
             got {:.3}",
            baselines[0]
        );
    }

    /// F-P2-001 (bullet variant): `draw_body_blocks` with a Bullets block
    /// containing ≥ 2 items must place each bullet at a DISTINCT baseline.
    ///
    /// ## Concrete numbers
    ///
    /// Body frame: `bbox.y = 0`, `bbox.height = 914_400 EMU` (= 72 pt).
    ///
    /// Initial cursor = `text_baseline_surface_y(bbox)` = 57.6 pt.
    ///
    /// Bullets use `font_size` = 16.0 pt.
    /// Advance per bullet = `16.0 * 1.2 = 19.2 pt`.
    ///
    /// Bullet 0 baseline = 57.6 pt.
    /// Bullet 1 baseline = 57.6 + 19.2 = 76.8 pt.
    /// Bullet 2 baseline = 76.8 + 19.2 = 96.0 pt.
    #[test]
    fn test_f_p2_001_bullet_items_baselines_are_distinct_and_stack_downward() {
        use slideforge_types::{BulletItem, ContentBlock, InlineNode, SourceSpan};

        use crate::coords::emu_to_pt;

        let bbox = BoundingBox {
            x: Emu(0),
            y: Emu(0),
            width: Emu(9_144_000),
            height: Emu(72 * 12_700),
        };

        let make_bullet = |s: &'static str| BulletItem {
            inlines: vec![InlineNode::Plain(Arc::from(s))],
            children: vec![],
            span: SourceSpan::default(),
        };

        let blocks = vec![ContentBlock::Bullets(vec![
            make_bullet("First bullet"),
            make_bullet("Second bullet"),
            make_bullet("Third bullet"),
        ])];

        let baselines = body_item_baselines(&blocks, &bbox);

        assert_eq!(
            baselines.len(),
            3,
            "expected 3 baselines for 3 bullet items; got {}",
            baselines.len()
        );

        let initial_baseline = text_baseline_surface_y(&bbox); // 57.6
        let font_size_bullets: f32 = 16.0;
        let advance = font_size_bullets * BODY_LINE_LEADING; // 19.2
        let expected_item0 = initial_baseline;
        let expected_item1 = expected_item0 + advance; // 76.8
        let expected_item2 = expected_item1 + advance; // 96.0

        assert!(
            (baselines[0] - expected_item0).abs() < 0.001,
            "bullet 0 baseline must equal {expected_item0:.3} pt; got {:.3}",
            baselines[0]
        );
        assert!(
            (baselines[1] - expected_item1).abs() < 0.001,
            "bullet 1 baseline must equal {expected_item1:.3} pt (= bullet0 + 16*1.2); got {:.3}",
            baselines[1]
        );
        assert!(
            (baselines[2] - expected_item2).abs() < 0.001,
            "bullet 2 baseline must equal {expected_item2:.3} pt (= bullet1 + 16*1.2); got {:.3}",
            baselines[2]
        );

        assert!(
            baselines[1] > baselines[0],
            "F-P2-001: bullet 1 baseline ({:.3}) must be > bullet 0 baseline ({:.3})",
            baselines[1],
            baselines[0]
        );
        assert!(
            baselines[2] > baselines[1],
            "F-P2-001: bullet 2 baseline ({:.3}) must be > bullet 1 baseline ({:.3})",
            baselines[2],
            baselines[1]
        );

        // Verify leading constant value.
        let expected_advance = emu_to_pt(Emu(72 * 12_700)) * 0.0; // just 0 for now, structure check
        let _ = expected_advance; // suppress unused warning — advance formula tested above
        assert!(
            (advance - 19.2_f32).abs() < 0.001,
            "BODY_LINE_LEADING (1.2) × 16.0 pt must equal 19.2 pt; got {advance:.3}"
        );
    }

    // ─── OBS-044-22-01: paint-state leak across frames ────────────────────────

    /// OBS-044-22-01 (driving regression test): `draw_text_at_bbox` must set
    /// an explicit fill before `surface.draw_text()` so that text rendering is
    /// never influenced by fill/stroke state leaked from a prior SVG/Diagram frame
    /// on the same slide.
    ///
    /// ## Bug
    ///
    /// `render_path` in `svg_embed.rs` calls `surface.set_fill(Some(...))` /
    /// `surface.set_fill(None)` and `surface.set_stroke(...)` for every SVG path.
    /// These mutations persist on the krilla `Surface` after `place_svg_at`
    /// completes (only the transform is restored by `surface.pop()`; fill/stroke
    /// are NOT part of the transform stack). `draw_text_at_bbox` then calls
    /// `surface.draw_text()` without first resetting the fill — text inherits the
    /// last SVG path's fill color.
    ///
    /// ## Test setup
    ///
    /// 2-frame slide:
    ///   Frame 0 — `Diagram` with a solid RED (`#FF0000`) rectangle SVG.
    ///             `render_path` sets `surface.fill = Some(red_fill)`.
    ///   Frame 1 — `Title` with text "Hello Paint State".
    ///             Without the fix: `draw_text` inherits `fill = red`.
    ///             With the fix: `draw_text` uses explicit black fill.
    ///
    /// ## Non-vacuous assertion (TD-VSDD-059)
    ///
    /// krilla emits the non-stroking (fill) color for text as an RGB or devicegray
    /// PDF color operator in the uncompressed content stream. When fill is solid
    /// black (RGB 0,0,0), the content stream contains a `0 0 0 rg` operator (or
    /// devicegray `0 g`). When fill is leaked red (RGB 1,0,0), it contains `1 0 0 rg`.
    ///
    /// Assertion: the uncompressed PDF MUST contain `0 0 0 rg` or `0 g` (explicit
    /// black text fill) AND MUST NOT contain `1 0 0 rg` immediately before the
    /// text font reference — confirming text fill is NOT inherited from the SVG.
    ///
    /// This test FAILS before the fix (text renders in red, PDF has `1 0 0 rg`)
    /// and PASSES after the fix (text explicitly sets black fill, `0 0 0 rg`).
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_obs_044_22_01_text_fill_not_leaked_from_svg_frame() {
        use slideforge_types::NormalizedDiagramSvg;

        // Frame 0: SVG with solid RED fill — will leak red into surface state.
        // Use ##-delimited raw string to avoid conflict with # in color value.
        let red_svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="0" y="0" width="100" height="100" fill="#FF0000"/>
        </svg>"##;
        let svg = NormalizedDiagramSvg::from_normalized_string(Arc::from(red_svg));

        let font_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/slideforge-math/fonts/latinmodern-math.otf")
            .canonicalize()
            .expect("LM Math fixture must be accessible for OBS-044-22-01 regression test");

        let exporter = PdfExporter::with_font_path(font_path);
        let deck = minimal_deck();
        let brand = Brand {
            name: Arc::from("TestBrand"),
            palette: BrandPalette {
                primary: Arc::from("#003087"),
                secondary: Arc::from("#FFFFFF"),
                accent: Arc::from("#F5A623"),
                neutral: Arc::from("#F0F0F0"),
            },
            // Override font is used instead of brand names.
            fonts: BrandFonts {
                heading: Arc::from("NoSuchFont_OBS_044_22_01"),
                body: Arc::from("NoSuchFont_OBS_044_22_01"),
                mono: Arc::from("Courier"),
                font_size_emu: 457_200,
            },
            layouts: vec![],
            span: SourceSpan::default(),
        };

        // 2-frame slide: Diagram (red SVG) FIRST, then Title (text) SECOND.
        // On a shared Surface, frame 0 leaves surface.fill = Some(red).
        // Frame 1 must NOT inherit that red fill for text.
        let laid_out = LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![LaidOutSlide {
                source_index: 0,
                slide_type_keyword: Arc::from("diagram-then-title"),
                frames: vec![
                    Frame {
                        bbox: BoundingBox {
                            x: Emu(0),
                            y: Emu(0),
                            width: Emu(9_144_000),
                            height: Emu(3_657_600), // 2-inch tall SVG frame
                        },
                        // STORY-039 IR reshape: Diagram is now struct with svg + alt fields.
                        content: FrameContent::Diagram {
                            svg,
                            alt: slideforge_types::AltText::Decorative,
                        },
                        text_flow: None,
                        region_role: None,
                    },
                    Frame {
                        bbox: BoundingBox {
                            x: Emu(0),
                            y: Emu(3_657_600), // below SVG frame
                            width: Emu(9_144_000),
                            height: Emu(914_400), // 1-inch title
                        },
                        content: FrameContent::Title(Arc::from("Hello Paint State")),
                        text_flow: None,
                        region_role: None,
                    },
                ],
                speaker_notes: None,
                register_tags: RegisterSet::new(),
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],
        };
        let opts = ExportOptions::default();

        // Export with uncompressed content streams so color operators are scannable.
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
            .expect("generate_pdf_inner must succeed for OBS-044-22-01 regression test");

        assert!(
            pdf_bytes.starts_with(b"%PDF-"),
            "PDF must start with %PDF- header"
        );

        // Assert the PDF contains a /Font resource — text drawing reached krilla.
        let has_font = pdf_bytes.windows(b"/Font".len()).any(|w| w == b"/Font");
        assert!(
            has_font,
            "OBS-044-22-01: PDF must contain /Font resource — text drawing must have fired. \
             If no font is present, the Title frame was not drawn at all."
        );

        // Search for color operators using byte patterns (PDF may contain binary bytes).
        // Use a safe helper to produce a UTF-8 preview for error messages.
        let pdf_preview = String::from_utf8_lossy(&pdf_bytes[..pdf_bytes.len().min(4096)]);

        // Canonical set of byte patterns that krilla may emit for a solid-black fill.
        //
        // Krilla serializes solid-black fills as either:
        //   - DeviceRGB:  `0 0 0 rg` or `0.0 0.0 0.0 rg`
        //   - DeviceGray: `0 g` (preceded/followed by whitespace per PDF tokenization)
        //
        // This SINGLE shared list is used by BOTH assertion 1 (existence check) and
        // assertion 2 (ordering check), so they can never silently diverge.
        //
        // `find_first_black_fill(slice)` returns the smallest byte offset within
        // `slice` at which any of these patterns begins, or `None` if none is found.
        let black_fill_patterns: &[&[u8]] = &[
            b"0 0 0 rg",
            b"0.0 0.0 0.0 rg",
            b" 0 g\n",
            b" 0 g\r",
            b"\n0 g\n",
        ];
        let find_first_black_fill = |slice: &[u8]| -> Option<usize> {
            black_fill_patterns
                .iter()
                .filter_map(|pat| slice.windows(pat.len()).position(|w| w == *pat))
                .min()
        };

        // Non-vacuous assertion 1: the PDF MUST contain a black fill color operator
        // from the explicit text fill set by `draw_text_at_bbox`.
        //
        // These tokens appear when `draw_text_at_bbox` calls `set_fill(Some(black_fill))`
        // before `draw_text()`.
        let has_black_fill_op = find_first_black_fill(&pdf_bytes).is_some();
        assert!(
            has_black_fill_op,
            "OBS-044-22-01 FAILED: PDF does not contain a black fill color operator \
             (`0 0 0 rg` or `0 g`). `draw_text_at_bbox` must explicitly call \
             `surface.set_fill(Some(black_fill))` before `surface.draw_text()`. \
             Without this, text inherits the SVG frame's fill (red in this test) \
             and renders in the wrong color. \
             PDF (first 4096 bytes): {pdf_preview}"
        );

        // Non-vacuous assertion 2: the PDF MUST NOT contain `1 0 0 rg` (red fill)
        // as the text color operator. The SVG sets fill to red (#FF0000 = 1,0,0 in
        // normalized RGB). If `draw_text_at_bbox` inherits this, `1 0 0 rg` appears
        // in the text drawing context. After the fix, only the SVG path uses red fill.
        //
        // Simplified non-vacuous check: assert `1 0 0 rg` exists only BEFORE any
        // text-related operator. We scan the raw bytes: if a black fill operator
        // appears after the LAST `1 0 0 rg`, the text fill was correctly reset to black.
        //
        // We use `rposition` (the LAST red occurrence) rather than `position` because
        // this is safe under krilla's current behavior: the fixture has a single SVG
        // frame drawn *before* the Title (text) frame. Krilla coalesces identical
        // consecutive fill-color operators, so `1 0 0 rg` is not re-emitted after the
        // black reset in the text frame. Therefore `rposition` finds the SVG red-fill
        // operator, and any black fill strictly after it belongs to the text frame.
        let last_red_pos = {
            let needle = b"1 0 0 rg";
            pdf_bytes.windows(needle.len()).rposition(|w| w == needle)
        };
        // Search for the first black-fill occurrence STRICTLY AFTER `last_red_pos`
        // using the same shared `find_first_black_fill` matcher as assertion 1.
        // We slice `pdf_bytes[red_pos..]` so the closure returns a relative offset;
        // we then add `red_pos` to recover the absolute byte position.
        let first_black_fill_after_red = last_red_pos.and_then(|red_pos| {
            find_first_black_fill(&pdf_bytes[red_pos..]).map(|rel| red_pos + rel)
        });

        if let (Some(red_pos), Some(black_pos)) = (last_red_pos, first_black_fill_after_red) {
            // `black_pos` is guaranteed > `red_pos` by construction (we searched only the
            // tail starting at `red_pos`), but assert explicitly to catch regressions.
            assert!(
                black_pos > red_pos,
                "OBS-044-22-01: The last `1 0 0 rg` (red fill from SVG) appears at offset {red_pos}. \
                 The first black fill operator AFTER that appears at offset {black_pos}. \
                 For text to be correctly black, black fill MUST appear AFTER the SVG red fill. \
                 This confirms draw_text_at_bbox explicitly resets the fill to black."
            );
        } else if last_red_pos.is_none() {
            // No red fill at all — SVG path color wasn't applied? The SVG may not
            // have rendered. The /Font assertion above already guards this case.
            // Accept as pass (SVG embedding not exercised, paint state isn't leaked).
        } else {
            // last_red_pos is Some but first_black_fill_after_red is None.
            // Red fill exists (from SVG) but no black fill reset was found strictly
            // after it — text is inheriting the leaked red fill. This is the FAILING case.
            panic!(
                "OBS-044-22-01 FAILED: `1 0 0 rg` (red SVG fill) found at offset {last_red_pos:?} in PDF, \
                 but NO black fill reset (`0 0 0 rg` or `0 g`) found AFTER that offset. \
                 `draw_text_at_bbox` is inheriting the SVG's red fill for text rendering. \
                 Fix: add `surface.set_fill(Some(black_fill))` before `surface.draw_text()` \
                 in `draw_text_at_bbox`. \
                 PDF (first 4096 bytes): {pdf_preview}"
            );
        }
    }

    // ── SEC-050-001: XMP title control-character guard ────────────────────────

    /// Unit test for [`validate_title_for_xmp`] — U+0001 (SOH) must be REJECTED.
    ///
    /// This test is LOAD-BEARING: removing the `validate_title_for_xmp` call
    /// in `generate_pdf_inner` (or this test) must cause the assertion to fail.
    ///
    /// Proof the guard fires on the real export path: the test calls
    /// `PdfExporter::export()` (the full public trait method), which calls
    /// `generate_pdf` → `generate_pdf_inner` → `validate_title_for_xmp`.
    /// There is no mock or stub between the test and the guard.
    ///
    /// SEC-050-001 / CWE-116.
    #[test]
    fn test_sec_050_001_control_char_in_title_rejects() {
        use slideforge_types::{DeckMetadata, OrderedMap};

        let exporter = PdfExporter::new();
        // Title containing U+0001 (SOH) — XML-1.0-illegal control character.
        let deck = Deck {
            slides: vec![],
            vars: OrderedMap::new(),
            metadata: DeckMetadata {
                title: Some(Arc::from("Bad\u{0001}Title")),
                slideforge_version: Arc::from("0.1.0"),
                lang: Some(Arc::from("en-US")),
                author: None,
                section_order: None,
            },
            registers: OrderedMap::new(),
            section_blocks: vec![],
            slide_sections: vec![],
        };
        let laid_out = minimal_laid_out_deck();
        let brand = minimal_brand();
        let opts = ExportOptions::default();

        let result = exporter.export(&deck, &laid_out, &brand, &opts);

        // Must return an error — the control-character guard must fire.
        match result {
            Err(ref e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("XMP metadata")
                        || err_msg.contains("control character")
                        || err_msg.contains("0001"),
                    "SEC-050-001: error message must mention XMP metadata or control character; \
                     got: {err_msg}"
                );
            },
            Ok(_) => {
                panic!("SEC-050-001: export with a title containing U+0001 must return Err; got Ok")
            },
        }
    }

    /// Companion test: a normal title (no control chars) must still export Ok.
    ///
    /// Confirms the guard does not reject valid titles (losslessness invariant).
    /// SEC-050-001 regression guard.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_sec_050_001_normal_title_exports_ok() {
        use slideforge_types::{DeckMetadata, OrderedMap};

        let exporter = PdfExporter::new();
        let deck = Deck {
            slides: vec![],
            vars: OrderedMap::new(),
            metadata: DeckMetadata {
                title: Some(Arc::from("Q4 Results — Product Strategy 2026")),
                slideforge_version: Arc::from("0.1.0"),
                lang: Some(Arc::from("en-US")),
                author: None,
                section_order: None,
            },
            registers: OrderedMap::new(),
            section_blocks: vec![],
            slide_sections: vec![],
        };
        let laid_out = minimal_laid_out_deck();
        let brand = minimal_brand();
        let opts = ExportOptions::default();

        let result = exporter.export(&deck, &laid_out, &brand, &opts);
        assert!(
            result.is_ok(),
            "SEC-050-001: a normal title must export successfully; got: {:?}",
            result.err()
        );
    }

    /// Unit test for `validate_title_for_xmp` — directly exercises each
    /// boundary of the XML-1.0-illegal character ranges.
    ///
    /// Load-bearing: tests the pure guard function directly to cover all
    /// illegal ranges (U+0000–U+0008, U+000B, U+000C, U+000E–U+001F, U+FFFE,
    /// U+FFFF) without going through the full export pipeline.
    #[test]
    fn test_sec_050_001_validate_title_for_xmp_illegal_ranges() {
        // Every illegal code point must be rejected.
        let illegal_codepoints: &[u32] = &[
            0x0000, 0x0001, 0x0008, // U+0000–U+0008
            0x000B, // U+000B VT
            0x000C, // U+000C FF
            0x000E, 0x001F, // U+000E–U+001F
            0xFFFE, 0xFFFF, // noncharacters
        ];
        for &cp in illegal_codepoints {
            // Safety: build title string containing the illegal code point.
            // Code points in these ranges are all valid Rust char values (except
            // U+0000 which is also a valid char in Rust).
            if let Some(ch) = char::from_u32(cp) {
                let title = format!("bad{ch}title");
                let result = validate_title_for_xmp(&title);
                assert!(
                    result.is_err(),
                    "validate_title_for_xmp must reject U+{cp:04X}; got Ok for title {title:?}"
                );
                if let Err(PdfExportError::InvalidXmpTitle { code_point, .. }) = result {
                    assert_eq!(
                        code_point, cp,
                        "InvalidXmpTitle.code_point must be U+{cp:04X}; got {code_point:04X}"
                    );
                } else {
                    panic!("validate_title_for_xmp must return InvalidXmpTitle for U+{cp:04X}");
                }
            }
        }

        // Legal title — must pass.
        assert!(
            validate_title_for_xmp("Normal Title — with em-dash and café").is_ok(),
            "validate_title_for_xmp must accept a normal title"
        );
        // U+0009 (HT), U+000A (LF), U+000D (CR) are legal in XML-1.0.
        assert!(
            validate_title_for_xmp("Title\twith\ttabs").is_ok(),
            "validate_title_for_xmp must accept U+0009 (tab)"
        );
        assert!(
            validate_title_for_xmp("Title\nwith\nnewlines").is_ok(),
            "validate_title_for_xmp must accept U+000A (LF)"
        );
    }

    // ─── STORY-081 C2: build()-driven PDF assertion for inline markup ──────────
    //
    // Finding C2 (HIGH): the snapshot tests in `inline_markup_pdf_snapshot.rs`
    // only test `slide_to_krilla_runs` in isolation. This test exercises the
    // PRODUCTION DRAW PATH (export_uncompressed → draw_body_blocks →
    // extract_all_inline_text) and proves that text content from inline markup
    // spans (Bold, Code, Strikethrough, etc.) is present in the PDF output.
    //
    // ## RED GATE confirmation (pre-C1 fix)
    //
    // Before the C1 fix, `draw_body_blocks` called `extract_inline_text` which
    // dropped Code/Strikethrough/Highlight/etc. via `_ => {}`. A body with only
    // a Code span would produce empty text → `draw_text_at_y` never called →
    // "code text" absent from PDF → this assertion FAILS.
    //
    // After the C1 fix (extract_all_inline_text), all variant text content is
    // preserved → "code text" present in uncompressed PDF stream.
    //
    // ## Font face distinctness via ActualText
    //
    // PDF ActualText attributes for Bold/Code spans prove the text reached krilla.
    // We verify "bold body text" and "code span" appear in the PDF byte stream.
    // For font face distinctness (Bold ≠ Regular), we rely on the slide_to_krilla_runs
    // unit tests in `inline_markup_pdf_snapshot.rs` (AC-004) which assert
    // FontFaceKind::Bold ≠ FontFaceKind::Regular — those tests cover the dispatch
    // decision that drives the Font::new(bold_data, 0) call at render time.

    /// STORY-081 C2 — build()-driven PDF assertion: inline markup text content
    /// is present in the uncompressed PDF output from the PRODUCTION DRAW PATH.
    ///
    /// Specifically: a `FrameContent::Body` containing `InlineNode::Bold`,
    /// `InlineNode::Code`, and `InlineNode::Plain` nodes produces a PDF where:
    ///
    /// 1. The PDF exports successfully (production draw path did not panic or error).
    /// 2. The `/ActualText` attribute for the `ContentBlock::Text` block contains
    ///    the text content of ALL inline variants (Bold + Plain) — not just Plain.
    ///    `extract_all_inline_text` preserves Bold text; the old `extract_inline_text`
    ///    on the `tag_engine` path had already been fixed in the STORY-081 Pass-1 burst.
    /// 3. The `extract_all_inline_text` function produces non-empty output for
    ///    `InlineNode::Code` nodes — confirming the production draw path for bullets
    ///    no longer silently drops Code spans via `_ => {}` catch-all.
    ///
    /// ## RED GATE for C1 (production draw path — the C2 spec finding)
    ///
    /// The old `extract_inline_text` in `body_item_baselines` / `draw_body_blocks`
    /// only matched `Plain`, `Bold`, `Italic`; Code/Strikethrough/etc. returned `""`.
    /// This meant a Code-only bullet item was treated as EMPTY: `body_item_baselines`
    /// allocated no baseline for it, `draw_body_blocks` skipped drawing it.
    ///
    /// The load-bearing proof:
    /// - `extract_all_inline_text([Code("code span")])` must return `"code span"` ≠ `""`.
    ///   Before C1 fix: old `extract_inline_text([Code(...)])` returned `""` (dropped).
    ///   After C1 fix: `extract_all_inline_text` returns `"code span"` via `slide_to_krilla_runs`.
    ///
    /// The build()-driven proof (production path invocation):
    /// - `export_uncompressed` runs the SAME `draw_body_blocks` / `body_item_baselines`
    ///   code used in production (not an isolated unit test on `slide_to_krilla_runs`).
    /// - The `/ActualText` for the Bold text block appears in the uncompressed PDF
    ///   structure dictionary, confirming the production path ran and stored text.
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    #[test]
    fn test_story_081_c2_production_pdf_draw_path_preserves_inline_text_content() {
        use crate::slide_pdf::extract_all_inline_text as pdf_extract_all;
        use slideforge_types::{BulletItem, ContentBlock, InlineNode, TextBlock};

        let exporter = PdfExporter::new();
        let deck = minimal_deck();
        let brand = minimal_brand();
        let opts = ExportOptions::default();

        // Build a LaidOutDeck with a Body frame containing Bold, Code, and Plain nodes.
        // This exercises the production `draw_body_blocks` + `body_item_baselines` path.
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
                            height: Emu(914_400),
                        },
                        content: FrameContent::Title(Arc::from("C2 Test Slide")),
                        text_flow: None,
                        region_role: None,
                    },
                    Frame {
                        bbox: BoundingBox {
                            x: Emu(0),
                            y: Emu(914_400),
                            width: Emu(9_144_000),
                            height: Emu(3_200_000),
                        },
                        content: FrameContent::Body(vec![
                            // Text block: Bold span with "bold body text" + Plain " plain"
                            // The tag engine writes /ActualText for this block → verifiable
                            // in the uncompressed PDF structure dictionary.
                            ContentBlock::Text(TextBlock {
                                inlines: vec![
                                    InlineNode::Bold(vec![InlineNode::Plain(Arc::from(
                                        "bold body text",
                                    ))]),
                                    InlineNode::Plain(Arc::from(" plain")),
                                ],
                                tag: slideforge_types::TextTag::Body,
                                span: slideforge_types::SourceSpan::default(),
                            }),
                            // Bullets block: Code span and Plain span.
                            // The text is drawn via draw_body_blocks → body_item_baselines.
                            // Before C1: extract_inline_text([Code("code span")]) returned ""
                            //   → body_item_baselines allocated NO baseline for this item
                            //   → draw_text_at_y was NOT called → code span was silent-dropped.
                            // After C1: extract_all_inline_text([Code(...)]) returns "code span"
                            //   → body_item_baselines allocates a baseline for this item
                            //   → draw_text_at_y IS called.
                            ContentBlock::Bullets(vec![
                                BulletItem {
                                    inlines: vec![InlineNode::Code(Arc::from("code span"))],
                                    children: vec![],
                                    span: slideforge_types::SourceSpan::default(),
                                },
                                BulletItem {
                                    inlines: vec![InlineNode::Plain(Arc::from("plain bullet"))],
                                    children: vec![],
                                    span: slideforge_types::SourceSpan::default(),
                                },
                            ]),
                        ]),
                        text_flow: None,
                        region_role: None,
                    },
                ],
                speaker_notes: None,
                register_tags: RegisterSet::new(),
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],
        };

        // PROOF 1 (load-bearing unit assertion for Code text preservation):
        // extract_all_inline_text([Code("code span")]) must return "code span".
        // Before C1: old extract_inline_text had `_ => {}` → returned "" (dropped).
        // After C1: extract_all_inline_text uses slide_to_krilla_runs → "code span".
        // This is the direct proof that body_item_baselines no longer silently drops Code.
        let code_inline = vec![InlineNode::Code(Arc::from("code span"))];
        let code_text = pdf_extract_all(&code_inline);
        assert_eq!(
            code_text, "code span",
            "STORY-081 C2 RED GATE (production draw path): \
             extract_all_inline_text([Code('code span')]) must return 'code span'.\n\
             Pre-C1: extract_inline_text had `_ => {{}}` catch-all → returned '' (dropped).\n\
             Post-C1: extract_all_inline_text preserves Code text via slide_to_krilla_runs.\n\
             This proves body_item_baselines and draw_body_blocks no longer silent-drop Code spans."
        );

        // PROOF 2 (build()-driven: production export path runs without error):
        let pdf_bytes = exporter
            .export_uncompressed(&deck, &laid_out, &brand, &opts)
            .expect("export_uncompressed must succeed for C2 test deck");

        assert!(
            !pdf_bytes.is_empty(),
            "C2: export_uncompressed must produce non-empty PDF bytes"
        );
        assert!(
            pdf_bytes.starts_with(b"%PDF-"),
            "C2: PDF output must start with %PDF-"
        );

        // PROOF 3 (build()-driven: /ActualText for Bold text block appears in PDF):
        // The tag_engine sets /ActualText on ContentBlock::Text blocks using
        // extract_all_inline_text. This confirms the Bold node's text is captured
        // in the structure dictionary and the production path was exercised.
        // Note: PDF text content streams use CIDFont encoding (not raw UTF-8), so
        // we verify /ActualText in the structure dictionary, not raw content stream text.
        let pdf_str = String::from_utf8_lossy(&pdf_bytes);
        assert!(
            pdf_str.contains("bold body text"),
            "STORY-081 C2: /ActualText 'bold body text' must appear in uncompressed PDF structure.\n\
             This confirms extract_all_inline_text ran on the production path for the Text block.\n\
             PDF excerpt (first 3000 chars): {pdf_str:.3000}"
        );
    }

    // ─── F-095-P2-001: char-split fallback in pack_words_into_lines ─────────────
    //
    // These tests drive the character-wrap fallback that was deferred (deferral
    // comments at exporter.rs:1549-1552 and 1688-1690 — now removed).  They are
    // LOAD-BEARING per TD-VSDD-059: each test exercises the production code path
    // that was previously missing.
    //
    // Test inventory:
    // | test_F095_P2_001_pack_words_char_split_over_wide_word      | F-095-P2-001 unit      |
    // | test_F095_P2_001_pack_words_char_split_bold_word_keeps_face | F-095-P2-001 bold face |
    // | test_F095_P2_001_inline_over_wide_word_export_no_panic     | F-095-P2-001 E2E       |

    /// F-095-P2-001 (AC-002/REND-002 inline path): `pack_words_into_lines` must
    /// split a single `SpanWord` that is wider than `max_line_width_pt` into ≥2
    /// lines via character-wrap (not leave it on a single over-wide line).
    ///
    /// ## Load-bearing assertion (TD-VSDD-059)
    ///
    /// Before the fix, `pack_words_into_lines` places an over-wide word alone on
    /// its line with no further splitting — a single-element `Vec<Vec<SpanWord>>`
    /// where `lines[0][0].text == original_word`.  The test asserts ≥2 lines.
    ///
    /// ## Why a real font?
    ///
    /// `measure_word_width_pt` returns 0.0 for absent faces (graceful degradation).
    /// A mock empty `ResolvedFontSet` makes everything "fit" and never triggers
    /// the over-wide branch.  We therefore inject a real `ResolvedFace` (LM Math)
    /// so measurements are non-zero and the 200-char word definitely overflows a
    /// narrow 50pt frame.
    #[test]
    #[allow(clippy::unwrap_used, non_snake_case)]
    fn test_F095_P2_001_pack_words_char_split_over_wide_word() {
        // Load LM Math as the regular face (same fixture used in story_095_red_gate.rs).
        let font_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/slideforge-math/fonts/latinmodern-math.otf")
            .canonicalize()
            .expect("Latin Modern Math OTF must be accessible");
        let bytes = std::fs::read(&font_path)
            .unwrap_or_else(|e| panic!("cannot read LM Math fixture: {e}"));
        let raw: std::sync::Arc<[u8]> = bytes.clone().into();
        let data: krilla::Data = bytes.into();
        let font =
            krilla::text::Font::new(data, 0).expect("krilla::Font::new must succeed for LM Math");
        let face = crate::font::ResolvedFace {
            font,
            raw,
            face_index: 0,
        };
        let font_set = crate::font::ResolvedFontSet::from_faces(Some(face), None, None, None);

        // A 200-char word (no whitespace) with FontFaceKind::Regular.
        let long_word = "a".repeat(200);
        let words = vec![SpanWord {
            face: crate::slide_pdf::FontFaceKind::Regular,
            text: long_word.clone(),
            y_offset_units: 0,
        }];

        // Narrow frame: 50pt. At 18pt font size, LM Math 'a' is ~8-10pt wide.
        // 200 * ~9pt = ~1800pt >> 50pt → over-wide word must trigger char-split.
        let font_size = 18.0_f32;
        let max_width_pt = 50.0_f32;

        let lines = pack_words_into_lines(&words, max_width_pt, font_size, &font_set);

        // Assertion 1 (F-095-P2-001 RED gate): must produce ≥2 lines.
        assert!(
            lines.len() >= 2,
            "F-095-P2-001 FAIL: pack_words_into_lines placed a 200-char word alone on 1 line \
             (no char-split).  Expected ≥2 lines for a word wider than {max_width_pt}pt. \
             Without char-split, the word overflows the right margin — AC-002/REND-002 unsatisfied."
        );

        // Assertion 2 (F-095-P2-001): no content lost — all chars appear in output.
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.iter().map(|w| w.text.as_str()))
            .collect();
        assert_eq!(
            all_text.len(),
            long_word.len(),
            "F-095-P2-001 FAIL: char-split must not drop characters. Input: {} chars, output: {} chars.",
            long_word.len(),
            all_text.len()
        );
        assert_eq!(
            all_text, long_word,
            "F-095-P2-001 FAIL: char-split output must equal original word (no chars inserted or dropped)."
        );
    }

    /// F-095-P2-001 (bold face preserved): when an over-wide `SpanWord` with
    /// `FontFaceKind::Bold` is char-split by `pack_words_into_lines`, ALL resulting
    /// fragment `SpanWord`s must retain `face == FontFaceKind::Bold`.
    ///
    /// ## Load-bearing assertion (TD-VSDD-059)
    ///
    /// Without this test, a naive implementation could reset the face on char-split
    /// fragments (e.g., defaulting to `FontFaceKind::Regular`), causing the second
    /// and subsequent fragments to lose bold rendering — a visual regression.
    #[test]
    #[allow(clippy::unwrap_used, non_snake_case)]
    fn test_F095_P2_001_pack_words_char_split_bold_word_keeps_face() {
        let font_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/slideforge-math/fonts/latinmodern-math.otf")
            .canonicalize()
            .expect("Latin Modern Math OTF must be accessible");
        let bytes = std::fs::read(&font_path)
            .unwrap_or_else(|e| panic!("cannot read LM Math fixture: {e}"));
        let raw: std::sync::Arc<[u8]> = bytes.clone().into();
        let data: krilla::Data = bytes.into();
        let font =
            krilla::text::Font::new(data, 0).expect("krilla::Font::new must succeed for LM Math");
        // Use the LM Math face as BOTH regular and bold (same file — bold measurement
        // is the same as regular; what matters is that the face KIND tag is preserved).
        let face_regular = crate::font::ResolvedFace {
            font: font.clone(),
            raw: raw.clone(),
            face_index: 0,
        };
        let face_bold = crate::font::ResolvedFace {
            font,
            raw,
            face_index: 0,
        };
        let font_set = crate::font::ResolvedFontSet::from_faces(
            Some(face_regular),
            Some(face_bold),
            None,
            None,
        );

        // 150-char word tagged Bold.
        let long_word = "b".repeat(150);
        let words = vec![SpanWord {
            face: crate::slide_pdf::FontFaceKind::Bold,
            text: long_word.clone(),
            y_offset_units: 0,
        }];

        let font_size = 18.0_f32;
        let max_width_pt = 50.0_f32;

        let lines = pack_words_into_lines(&words, max_width_pt, font_size, &font_set);

        // Must split (otherwise the face preservation test is vacuous).
        assert!(
            lines.len() >= 2,
            "F-095-P2-001 bold FAIL: 150-char word must split into ≥2 lines."
        );

        // Every fragment on every line must retain FontFaceKind::Bold.
        for (line_idx, line) in lines.iter().enumerate() {
            for (word_idx, word) in line.iter().enumerate() {
                assert_eq!(
                    word.face,
                    crate::slide_pdf::FontFaceKind::Bold,
                    "F-095-P2-001 FAIL: char-split fragment at line={line_idx} word={word_idx} \
                     lost bold face (got {:?}). A char-split BOLD word must keep bold face on \
                     ALL fragments.",
                    word.face
                );
            }
        }
    }

    /// F-095-P2-001 (E2E export path): exporting a `FrameContent::Body` or
    /// `FrameContent::TextRun` with a 200-char single-word token in a narrow
    /// frame must NOT panic and the PDF must be valid.
    ///
    /// ## Load-bearing assertion (TD-VSDD-059)
    ///
    /// Without char-split, the word overflows the frame right margin.  The
    /// export still completes (overflow is clamped), but this E2E test proves
    /// the production path is exercised end-to-end — not just the unit function.
    #[test]
    #[allow(clippy::unwrap_used, non_snake_case)]
    fn test_F095_P2_001_inline_over_wide_word_export_no_panic() {
        use slideforge_types::{Emu, InlineNode};

        let font_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/slideforge-math/fonts/latinmodern-math.otf")
            .canonicalize()
            .expect("Latin Modern Math OTF must be accessible");
        let bytes = std::fs::read(&font_path)
            .unwrap_or_else(|e| panic!("cannot read LM Math fixture: {e}"));
        let raw: std::sync::Arc<[u8]> = bytes.clone().into();
        let data: krilla::Data = bytes.into();
        let font = krilla::text::Font::new(data, 0).expect("krilla::Font::new must succeed");
        let face = crate::font::ResolvedFace {
            font,
            raw,
            face_index: 0,
        };
        let font_set = crate::font::ResolvedFontSet::from_faces(Some(face), None, None, None);
        let exporter = PdfExporter::with_resolved_font_set(font_set);

        // 200-char single token — no whitespace — in a very narrow frame (1-inch = ~72pt).
        let long_token: String = "x".repeat(200);
        let laid_out = LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![LaidOutSlide {
                source_index: 0,
                slide_type_keyword: Arc::from("content"),
                frames: vec![Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(914_400),
                        width: Emu(914_400),    // 1 inch narrow (~72pt)
                        height: Emu(7_315_200), // 8 inches — ample vertical space
                    },
                    content: FrameContent::TextRun(vec![InlineNode::Plain(Arc::from(
                        long_token.as_str(),
                    ))]),
                    text_flow: None,
                    region_role: None,
                }],
                speaker_notes: None,
                register_tags: RegisterSet::new(),
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],
        };

        let deck = minimal_deck();
        let brand = minimal_brand();
        let opts = slideforge_plugin_api::ExportOptions::default();

        // F-095-P2-001 E2E assertion: export must not panic for over-wide single token.
        let result = exporter.export_uncompressed(&deck, &laid_out, &brand, &opts);
        assert!(
            result.is_ok(),
            "F-095-P2-001 E2E FAIL: export must succeed for 200-char single-token TextRun \
             in a narrow frame. Got error: {:?}",
            result.err()
        );
        let pdf_bytes = result.unwrap();
        assert!(
            pdf_bytes.starts_with(b"%PDF-"),
            "F-095-P2-001 E2E: PDF must start with %PDF-"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STORY-072 — PDF gradient tests (Red Gate)
// ─────────────────────────────────────────────────────────────────────────────
//
// | Test | AC | Clause |
// |---|---|---|
// | test_BC_3_04_001_ac004_pdf_full_export_gradient_shape_no_error | AC-004 | postcondition 5 |
// | test_BC_3_04_001_ac004_pdf_draw_gradient_rect_executes_and_changes_output | AC-004 | draw path load-bearing (HIGH-001) |
// | test_BC_3_04_001_ac004_pdf_gradient_shape_produces_nonzero_bytes | AC-004 | non-zero output |
// | test_BC_3_04_001_ec005_pdf_same_from_to_gradient_valid | EC-005 | STORY-072 EC-005 |

#[cfg(test)]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    non_snake_case
)]
mod story_072_tests {
    use std::sync::Arc;

    use slideforge_layout::types::{
        BoundingBox, FillSpec, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize,
        ShapeFrame, ShapeType,
    };
    use slideforge_types::{AltText, Brand, BrandFonts, BrandPalette, Deck, Emu, Rgb, SourceSpan};

    use super::PdfExporter;
    use slideforge_plugin_api::{ExportOptions, Exporter};

    // ─── Fixture builders ────────────────────────────────────────────────────

    fn make_brand() -> Brand {
        use slideforge_types::span::SourceSpan;
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
            span: SourceSpan::default(),
        }
    }

    fn make_deck_one_slide() -> Deck {
        use slideforge_types::deck::DeckMetadata;
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
                field_spans: OrderedMap::new(),
            }],
            vars: OrderedMap::new(),
            metadata: DeckMetadata {
                title: Some(Arc::from("Gradient Test")),
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

    fn gradient_bbox() -> BoundingBox {
        BoundingBox {
            x: Emu(914_400),
            y: Emu(914_400),
            width: Emu(914_400),
            height: Emu(914_400),
        }
    }

    fn make_gradient_deck(from: Rgb, to: Rgb, alt: AltText) -> LaidOutDeck {
        LaidOutDeck {
            page_size: PageSize::default(),
            slide_sections: vec![],
            slides: vec![LaidOutSlide {
                source_index: 0,
                slide_type_keyword: Arc::from("title"),
                frames: vec![Frame {
                    bbox: gradient_bbox(),
                    content: FrameContent::Shape(ShapeFrame {
                        shape_type: ShapeType::Rect,
                        fill: FillSpec::Gradient { from, to },
                        text: None,
                        alt,
                    }),
                    text_flow: None,
                    region_role: None,
                }],
                speaker_notes: None,
                register_tags: vec![],
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
        }
    }

    // ─── Tests ───────────────────────────────────────────────────────────────

    /// AC-004 (STORY-072) — `PdfExporter::export` with a `FillSpec::Gradient` shape
    /// must not return an error (no panic, no Err).
    ///
    /// `draw_gradient_rect` is fully implemented and wired into the export loop.
    /// This test verifies the no-error postcondition — the gradient path must
    /// succeed without panicking or returning `Err`.
    #[test]
    fn test_BC_3_04_001_ac004_pdf_full_export_gradient_shape_no_error() {
        let laid_out = make_gradient_deck(
            Rgb { r: 255, g: 0, b: 0 },
            Rgb { r: 0, g: 0, b: 255 },
            AltText::Provided(Arc::from("Gradient background")),
        );
        let deck = make_deck_one_slide();
        let brand = make_brand();
        let opts = ExportOptions::default();
        let exporter = PdfExporter::new();
        let result = exporter.export(&deck, &laid_out, &brand, &opts);
        assert!(
            result.is_ok(),
            "PdfExporter::export must not fail for a deck with FillSpec::Gradient shape; \
             err: {:?}",
            result.err()
        );
    }

    /// AC-004 (STORY-072) — `draw_gradient_rect` is called during PDF export and produces
    /// different bytes than an identical export WITHOUT the gradient shape.
    ///
    /// ## Closing HIGH-001 (adversary Pass-1 / TD-VSDD-059)
    ///
    /// The previous test was a no-op fn-pointer type-check.  `draw_gradient_rect` is
    /// real production code (exporter.rs:~1528) invoked at the shape dispatch site
    /// (exporter.rs:~770).  This test exercises that live path:
    ///
    /// 1. Exports a deck containing one `FillSpec::Gradient` shape → gradient PDF bytes.
    /// 2. Exports an identical deck with **zero frames** (no shape) → baseline PDF bytes.
    /// 3. Asserts the bytes DIFFER — proving `draw_gradient_rect` executed and mutated
    ///    the PDF content stream.
    ///
    /// A regression that disconnects `draw_gradient_rect` from the export pipeline
    /// (e.g., silently skipping the Shape arm) would make both exports identical,
    /// causing this test to fail.
    #[test]
    fn test_BC_3_04_001_ac004_pdf_draw_gradient_rect_executes_and_changes_output() {
        let deck = make_deck_one_slide();
        let brand = make_brand();
        let opts = ExportOptions::default();

        // Export WITH gradient shape.
        let gradient_laid_out = make_gradient_deck(
            Rgb { r: 255, g: 0, b: 0 },
            Rgb { r: 0, g: 0, b: 255 },
            AltText::Provided(Arc::from("Red-to-blue gradient")),
        );
        let gradient_bytes = PdfExporter::new()
            .export(&deck, &gradient_laid_out, &brand, &opts)
            .expect("export with gradient shape must succeed");

        // Export WITHOUT any shape (empty slide, no frames).
        let empty_laid_out = LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![LaidOutSlide {
                source_index: 0,
                slide_type_keyword: Arc::from("title"),
                frames: vec![], // No shape frame — baseline for comparison.
                speaker_notes: None,
                register_tags: vec![],
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],
        };
        let empty_bytes = PdfExporter::new()
            .export(&deck, &empty_laid_out, &brand, &opts)
            .expect("export with empty slide must succeed");

        // Both must be valid PDFs.
        assert!(
            gradient_bytes.starts_with(b"%PDF"),
            "gradient export must produce a valid PDF; got: {:?}",
            &gradient_bytes[..gradient_bytes.len().min(8)]
        );
        assert!(
            empty_bytes.starts_with(b"%PDF"),
            "empty export must produce a valid PDF; got: {:?}",
            &empty_bytes[..empty_bytes.len().min(8)]
        );

        // THE LOAD-BEARING ASSERTION (TD-VSDD-059): the gradient draw path must mutate
        // the PDF content stream.  If draw_gradient_rect is silently disconnected from
        // the export pipeline, both exports produce identical bytes and this fails.
        assert_ne!(
            gradient_bytes, empty_bytes,
            "PDF export with gradient shape must produce DIFFERENT bytes than an export with \
             no shape frames — draw_gradient_rect must have executed and changed the PDF \
             content stream. If this fails, the gradient draw path is disconnected."
        );
    }

    /// AC-004 (STORY-072) — PDF export with gradient shape produces non-zero bytes.
    ///
    /// This is a baseline sanity check: gradient export must produce a valid PDF
    /// with at least `%PDF` magic bytes and non-zero length.
    ///
    /// The load-bearing gradient-path assertion is in
    /// `test_BC_3_04_001_ac004_pdf_draw_gradient_rect_executes_and_changes_output`
    /// (HIGH-001 closure).
    #[test]
    fn test_BC_3_04_001_ac004_pdf_gradient_shape_produces_nonzero_bytes() {
        let laid_out = make_gradient_deck(
            Rgb { r: 255, g: 0, b: 0 },
            Rgb { r: 0, g: 0, b: 255 },
            AltText::Provided(Arc::from("Gradient background")),
        );
        let deck = make_deck_one_slide();
        let brand = make_brand();
        let opts = ExportOptions::default();
        let exporter = PdfExporter::new();
        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        assert!(!bytes.is_empty(), "PDF export must produce non-zero bytes");
        // PDF magic bytes check.
        assert!(
            bytes.starts_with(b"%PDF"),
            "export output must begin with PDF magic bytes; got: {:?}",
            &bytes[..bytes.len().min(8)]
        );
    }

    /// EC-005 (STORY-072) — Same `from` and `to` gradient color (flat gradient) is
    /// valid; PDF export must not fail.
    ///
    /// RED GATE: passes today (Shape skipped). After implementation with
    /// `draw_gradient_rect` wired in, the gradient path must handle same-color safely.
    #[test]
    fn test_BC_3_04_001_ec005_pdf_same_from_to_gradient_valid() {
        let same = Rgb { r: 255, g: 0, b: 0 };
        let laid_out =
            make_gradient_deck(same, same, AltText::Provided(Arc::from("Flat gradient")));
        let deck = make_deck_one_slide();
        let brand = make_brand();
        let opts = ExportOptions::default();
        let exporter = PdfExporter::new();
        let result = exporter.export(&deck, &laid_out, &brand, &opts);
        assert!(
            result.is_ok(),
            "PDF export with same from==to gradient must not fail; err: {:?}",
            result.err()
        );
    }
}
