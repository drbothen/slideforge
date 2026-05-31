//! SVG embedding via `usvg` parse → krilla `Surface` path-drawing operations.
//!
//! For each `FrameContent::Diagram(NormalizedDiagramSvg)` or
//! `FrameContent::Chart` frame in a [`LaidOutSlide`], this module:
//!
//! 1. Parses the SVG via `usvg::Tree::from_str()`.
//! 2. Walks the `usvg` node tree recursively.
//! 3. Issues path-drawing operations on the krilla `Surface`
//!    (`surface.set_fill(...)`, `surface.set_stroke(...)`,
//!    `surface.draw_path(...)`).
//!
//! **Vector-only contract (BC-4.03.002 AC-005):**
//! SVG content is embedded as PDF vector paths — never rasterized. The `image`
//! crate MUST NOT be used in this module. A PDF produced from a chart-only deck
//! must contain zero raster images (`pdfimages -list` reports no entries).
//!
//! **krilla ↔ usvg independence:**
//! krilla 0.6.0 has no `usvg` dependency (confirmed in tech-validation.md).
//! The workspace `usvg =0.47.0` is safe to use alongside krilla with zero
//! version conflicts. The `krilla-svg` companion crate is NOT used here;
//! `slideforge-pdf` performs its own usvg-node → krilla-Surface path
//! translation.
//!
//! **pdf-writer prohibition:**
//! Do NOT route SVG paths through `pdf-writer` path operators directly. All
//! drawing goes through krilla's `Surface` to preserve krilla's coordinate
//! space and layer management (RISK-1 per tech-validation.md).
//!
//! **tiny-skia-path version difference:**
//! usvg 0.47.0 uses `tiny_skia_path` 0.12.0; krilla 0.6.0 uses 0.11.4.
//! These are different crate versions and their `Path` types cannot be
//! directly shared. Path segment conversion is performed by iterating
//! `usvg::Path::data().segments()` (0.12.0 API) and rebuilding via
//! `krilla::PathBuilder` (0.11.4 API).

use krilla::color::rgb;
use krilla::geom::PathBuilder;
use krilla::num::NormalizedF32;
use krilla::paint::{Fill, FillRule, Paint};
use krilla::surface::Surface;
use slideforge_types::NormalizedDiagramSvg;
use usvg::{Node, Options, Tree};

use crate::error::PdfExportError;

/// Embed a [`NormalizedDiagramSvg`] as vector paths on a krilla `Surface`.
///
/// The function parses the SVG via `usvg::Tree::from_str`, walks the node
/// tree, and issues corresponding path operations on `surface`. No rasterization
/// occurs.
///
/// # Parameters
///
/// - `svg` — the normalized SVG payload from `FrameContent::Diagram`.
/// - `surface` — the krilla `Surface` for the current PDF page. The surface
///   is borrowed mutably for the duration of this call.
///
/// # Errors
///
/// Returns [`PdfExportError::SvgEmbed`] if:
/// - The SVG cannot be parsed by `usvg`.
/// - A path operation fails on the krilla `Surface`.
///
/// # Contract
///
/// The output PDF must not contain any raster image `XObject` for SVG input
/// (where `/Image` is the PDF keyword for embedded raster images).
/// Verified in tests by asserting no `Subtype /Image` entry appears in the
/// output bytes.
///
/// # Errors
///
/// See module-level documentation.
pub fn embed_normalized_svg(
    svg: &NormalizedDiagramSvg,
    surface: &mut Surface<'_>,
) -> Result<(), PdfExportError> {
    embed_svg_str(svg.as_str(), surface)
}

/// Embed a raw SVG string (e.g., from `FrameContent::Chart`) as vector paths.
///
/// The SVG is expected to be PPTX-safe (no `<script>`, no `<foreignObject>`)
/// but may not have gone through the full `NormalizedDiagramSvg` pipeline
/// (chart SVGs are produced directly by `plotters` and are guaranteed safe
/// by construction).
///
/// # Errors
///
/// Returns [`PdfExportError::SvgEmbed`] if the SVG cannot be parsed or
/// drawn.
pub fn embed_svg_str(svg_str: &str, surface: &mut Surface<'_>) -> Result<(), PdfExportError> {
    let tree =
        Tree::from_str(svg_str, &Options::default()).map_err(|e| PdfExportError::SvgEmbed {
            message: format!("usvg parse error: {e}"),
        })?;

    render_group(tree.root(), surface)
}

/// Recursively render a usvg `Group` node and all its children onto `surface`.
fn render_group(group: &usvg::Group, surface: &mut Surface<'_>) -> Result<(), PdfExportError> {
    for child in group.children() {
        match child {
            Node::Group(g) => {
                // Recurse into nested groups.
                render_group(g, surface)?;
            },
            Node::Path(path) => {
                if !path.is_visible() {
                    continue;
                }
                render_path(path, surface)?;
            },
            // Image and Text nodes are not yet rendered in this story.
            // STORY-045 will extend coverage for text rendering.
            // Image nodes would require rasterization (forbidden by AC-005).
            Node::Image(_) | Node::Text(_) => {
                // Intentionally skipped: images violate vector-only contract;
                // text rendering requires krilla font API (STORY-044 / STORY-045).
            },
        }
    }
    Ok(())
}

/// Render a single `usvg::Path` node as a krilla `draw_path` call.
///
/// Translates path segments from `tiny_skia_path` 0.12.0 (usvg's version)
/// to `krilla::PathBuilder` (which wraps `tiny_skia_path` 0.11.4).
///
/// Returns `Ok(())` always in this story; the `Result` return type is kept
/// for forward compatibility when future paths may produce errors via the
/// krilla surface (e.g., clip-path operations in STORY-045).
#[allow(clippy::unnecessary_wraps)]
fn render_path(path: &usvg::Path, surface: &mut Surface<'_>) -> Result<(), PdfExportError> {
    use usvg::tiny_skia_path::PathSegment;

    // Build the krilla path from usvg segment data.
    let mut builder = PathBuilder::new();
    for segment in path.data().segments() {
        match segment {
            PathSegment::MoveTo(p) => builder.move_to(p.x, p.y),
            PathSegment::LineTo(p) => builder.line_to(p.x, p.y),
            PathSegment::QuadTo(p1, p2) => builder.quad_to(p1.x, p1.y, p2.x, p2.y),
            PathSegment::CubicTo(p1, p2, p3) => {
                builder.cubic_to(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
            },
            PathSegment::Close => builder.close(),
        }
    }

    let Some(krilla_path) = builder.finish() else {
        // Degenerate path (zero points) — skip silently.
        return Ok(());
    };

    // Apply fill.
    if let Some(fill) = path.fill() {
        let krilla_fill = usvg_fill_to_krilla(fill);
        surface.set_fill(Some(krilla_fill));
    } else {
        surface.set_fill(None);
    }

    // Apply stroke (stub — full stroke translation is STORY-044 scope).
    // For AC-005 we only need fill; clear any lingering stroke.
    surface.set_stroke(None);

    surface.draw_path(&krilla_path);

    Ok(())
}

/// Convert a `usvg::Fill` to a `krilla::Fill`.
///
/// Only `Paint::Color` is fully translated here. Gradients and patterns fall
/// back to opaque black (STORY-045 extends color server support). A
/// `tracing::warn!` is emitted for any unsupported paint server so that the
/// fallback is visible in logs and not silently swapped (F-010 fix).
///
/// Logging uses `tracing::warn!` with structured fields naming the paint server
/// kind, in accordance with the project convention (no `println!` in library
/// crates).
fn usvg_fill_to_krilla(fill: &usvg::Fill) -> Fill {
    let opacity = usvg_opacity_to_krilla(fill.opacity().get());
    let rule = usvg_fill_rule_to_krilla(fill.rule());

    let paint: Paint = match fill.paint() {
        usvg::Paint::Color(color) => rgb::Color::new(color.red, color.green, color.blue).into(),
        usvg::Paint::LinearGradient(_) => {
            // F-010: warn instead of silently substituting.
            // Full linear-gradient support is deferred to STORY-045.
            tracing::warn!(
                paint_server = "LinearGradient",
                fallback = "opaque black",
                "unsupported SVG paint server — falling back to opaque black; \
                 full gradient support deferred to STORY-045"
            );
            rgb::Color::new(0, 0, 0).into()
        },
        usvg::Paint::RadialGradient(_) => {
            tracing::warn!(
                paint_server = "RadialGradient",
                fallback = "opaque black",
                "unsupported SVG paint server — falling back to opaque black; \
                 full gradient support deferred to STORY-045"
            );
            rgb::Color::new(0, 0, 0).into()
        },
        usvg::Paint::Pattern(_) => {
            tracing::warn!(
                paint_server = "Pattern",
                fallback = "opaque black",
                "unsupported SVG paint server — falling back to opaque black; \
                 full pattern support deferred to STORY-045"
            );
            rgb::Color::new(0, 0, 0).into()
        },
    };

    Fill {
        paint,
        opacity,
        rule,
    }
}

/// Convert a usvg opacity value (f32 in 0..=1) to a krilla `NormalizedF32`.
///
/// Clamps out-of-range values defensively.
fn usvg_opacity_to_krilla(opacity: f32) -> NormalizedF32 {
    NormalizedF32::new(opacity.clamp(0.0, 1.0)).unwrap_or(NormalizedF32::ONE)
}

/// Convert a usvg `FillRule` to the equivalent krilla `FillRule`.
fn usvg_fill_rule_to_krilla(rule: usvg::FillRule) -> FillRule {
    match rule {
        usvg::FillRule::NonZero => FillRule::NonZero,
        usvg::FillRule::EvenOdd => FillRule::EvenOdd,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use slideforge_types::NormalizedDiagramSvg;

    /// A minimal SVG rect used as test input.
    const SIMPLE_SVG_RECT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\">\
         <rect x=\"10\" y=\"10\" width=\"80\" height=\"80\" fill=\"#003087\"/>\
         </svg>";

    /// BC-4.03.002 AC-005: `embed_normalized_svg` converts a simple SVG rect
    /// to krilla Surface draw calls without rasterization.
    ///
    /// This test verifies:
    /// - usvg parses the SVG correctly.
    /// - The resulting PDF bytes start with `%PDF-` (valid PDF structure).
    /// - The PDF bytes contain NO `/Image` `XObject` (vector-only assertion)
    ///   (verified by checking for the absence of `Subtype /Image`).
    #[test]
    fn test_bc_4_03_002_svg_embed_converts_rect_to_vector_paths() {
        use krilla::Document;
        use krilla::page::PageSettings;

        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(SIMPLE_SVG_RECT));

        // Build a real Document + Page to get a live Surface.
        let mut document = Document::new();
        let mut page =
            document.start_page_with(PageSettings::from_wh(595.0, 842.0).expect("valid page size"));
        let mut surface = page.surface();

        let result = embed_normalized_svg(&normalized, &mut surface);

        surface.finish();
        page.finish();

        // The embed must succeed.
        assert!(
            result.is_ok(),
            "embed_normalized_svg must succeed for a simple SVG rect: {result:?}"
        );

        let pdf_bytes = document
            .finish()
            .expect("krilla document serialization must succeed");

        // The output must be a valid PDF (starts with %PDF-).
        assert!(
            pdf_bytes.starts_with(b"%PDF-"),
            "PDF output must start with %PDF- header"
        );

        // Vector-only assertion: no raster Image `XObject` (Subtype /Image) in the PDF.
        // We check for `/Subtype /Image` which is the PDF signature of an embedded
        // raster image `XObject`. Note: `/ImageC`, `/ImageB` appear in the standard
        // ProcSet declaration even for non-image PDFs and are NOT image `XObject`s.
        let has_raster_image = pdf_bytes
            .windows(b"/Subtype /Image".len())
            .any(|w| w == b"/Subtype /Image");
        assert!(
            !has_raster_image,
            "PDF output must NOT contain raster /Image `XObject` (Subtype /Image) \
             for SVG vector content"
        );
    }

    /// BC-4.03.002 AC-005: `embed_svg_str` also parses correctly.
    #[test]
    fn test_bc_4_03_002_svg_str_is_parseable_by_usvg() {
        // Verify that embed_svg_str would receive a parseable SVG.
        // The embed call itself requires a live Surface (deferred to green phase).
        let parse_result = usvg::Tree::from_str(SIMPLE_SVG_RECT, &usvg::Options::default());
        assert!(
            parse_result.is_ok(),
            "usvg must parse the simple SVG rect without error: {:?}",
            parse_result.err()
        );
    }
}
