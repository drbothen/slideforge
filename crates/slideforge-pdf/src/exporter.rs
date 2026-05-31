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
//! ## Architecture (BC-4.03.002)
//!
//! `PdfExporter` is an effectful shell (ARCH-INDEX SS-07). It:
//! 1. Creates a `krilla::Document::new()`.
//! 2. Iterates over `laid_out.slides`, calling into `SlideTagEngine` and
//!    `svg_embed` for each slide.
//! 3. Calls `document.set_tag_tree(tag_tree)` with the assembled structural tree.
//! 4. Calls `document.finish()` → `KrillaResult<Vec<u8>>`.
//! 5. Maps `KrillaError` → `PdfExportError::Serialize` → `ExportError::RenderError`.
//! 6. Returns the PDF bytes.
//!
//! No subprocess is spawned. No FFI to C libraries. Pure Rust.
//!
//! ## EMU canonicalization (F-005)
//!
//! All EMU-to-points conversions use [`slideforge_types::Emu::to_points`] which
//! calls the canonical `EMU_PER_POINT = 12_700` constant defined in
//! `slideforge-types`. The private `EMU_PER_POINT` constant previously
//! duplicated here has been removed.

use krilla::Document;
use krilla::page::PageSettings;
use slideforge_layout::LaidOutDeck;
use slideforge_plugin_api::{ExportError, ExportOptions, Exporter};
use slideforge_types::{Brand, Deck};

use crate::error::PdfExportError;
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
    /// when `PdfExporter` carries font caches or configuration (STORY-044+).
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError`] on failure. The [`Exporter::export`]
    /// implementation maps this to [`ExportError::RenderError`].
    #[allow(clippy::unused_self)]
    fn generate_pdf(
        &self,
        _deck: &Deck,
        laid_out: &LaidOutDeck,
        _brand: &Brand,
        _opts: &ExportOptions,
    ) -> Result<Vec<u8>, PdfExportError> {
        // Create a krilla Document with default settings.
        // Default SerializeSettings has enable_tagging: true.
        let mut document = Document::new();

        // Instantiate the tag engine — one per export pass, stateless per slide.
        let tag_engine = SlideTagEngine::new();

        // Collect per-slide Part groups for later assembly into the deck tag tree.
        let mut slide_parts = Vec::with_capacity(laid_out.slides.len());

        for slide in &laid_out.slides {
            // Convert page dimensions from EMU to PDF points using the canonical
            // Emu::to_points() from slideforge-types (EMU_PER_POINT = 12_700).
            // TD-VSDD-060: no duplicate EMU_PER_POINT constant in this crate.
            // f64→f32 truncation is intentional: PDF point precision at typical
            // slide sizes (720×540 pt) loses < 0.01 pt — below rendering tolerance.
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            let width_pts = laid_out.page_size.width.to_points() as f32;
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            let height_pts = laid_out.page_size.height.to_points() as f32;

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
            // tag_slide returns a PartResult containing the Part TagGroup and
            // the list of decorative frame indices.
            let part_result = tag_engine.tag_slide(slide)?;

            // `surface` is obtained for drawing operations.
            // Content drawing (text, SVG paths) is added in STORY-044/STORY-045.
            // The page is currently blank except for the tag structure tree.
            //
            // Decorative frames (part_result.decorative_frame_indices) will be
            // marked with ContentTag::Artifact(ArtifactType::Other) during the
            // drawing pass in STORY-044.
            let surface = page.surface();
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
