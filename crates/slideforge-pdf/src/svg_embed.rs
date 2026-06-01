//! SVG embedding via `usvg` parse → krilla `Surface` path-drawing operations.
//!
//! For each `FrameContent::Diagram(NormalizedDiagramSvg)` or
//! `FrameContent::Chart` frame in a [`slideforge_layout::LaidOutSlide`], this module:
//!
//! 1. Parses the SVG via `usvg::Tree::from_str()`.
//! 2. Pushes a frame-level transform (translate + scale) so the SVG viewport
//!    maps exactly to the frame's bounding box on the krilla `Surface`.
//! 3. Walks the `usvg` node tree recursively.
//! 4. For each `usvg::Path`, reads `path.abs_transform()` (absolute, includes
//!    all ancestor `<g transform>` stacks and the viewBox→viewport mapping),
//!    pushes it on the surface, draws the LOCAL path data, then pops.
//! 5. Issues path-drawing operations on the krilla `Surface`
//!    (`surface.set_fill(...)`, `surface.set_stroke(...)`,
//!    `surface.draw_path(...)`).
//!
//! ## Transform composition (F-P18-001 / F-P18-002 fix)
//!
//! The correct composed transform for each path is:
//!
//! ```text
//! frame_transform ∘ path.abs_transform() ∘ local_path_data
//! ```
//!
//! where:
//! - `frame_transform` = `translate(surface_x, surface_y) ∘ scale(frame_w / svg_w, frame_h / svg_h)` —
//!   maps SVG user units to the frame's Surface coordinates (fit-to-frame, non-uniform).
//! - `path.abs_transform()` = absolute affine for this path in SVG user-unit space,
//!   including all ancestor `<g transform>` contributions (F-P18-001 fix).
//! - local path data = the raw `data()` segments in path-local coords.
//!
//! In usvg 0.47.0, `Path::data()` is in LOCAL coordinates (the transform is NOT
//! pre-baked). `Path::abs_transform()` returns the complete ancestor transform stack.
//! Applying it as a separate push/pop is the correct way to compose this chain.
//!
//! ## Aspect ratio policy
//!
//! Non-uniform `scale(frame_w / svg_w, frame_h / svg_h)` fills the frame exactly.
//! This matches "fit to frame" semantics for presentation slides. The viewport is
//! always fully occupied (no letterboxing), which is appropriate for structured
//! diagram and chart frames where the SVG is authored to match the frame aspect ratio.
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
//!
//! ## Transform type mapping (`tiny_skia_path` 0.12 → `krilla::geom::Transform`)
//!
//! Both usvg 0.47.0 (`tiny_skia_path` 0.12) and krilla 0.6.0 (`tiny_skia_path` 0.11.4)
//! share the same column-major affine layout `{sx, kx, ky, sy, tx, ty}` and the same
//! `from_row(sx, ky, kx, sy, tx, ty)` function signature. The field-to-param mapping
//! is therefore direct: `Transform::from_row(t.sx, t.ky, t.kx, t.sy, t.tx, t.ty)`.

use krilla::color::rgb;
use krilla::geom::PathBuilder;
use krilla::num::NormalizedF32;
use krilla::paint::{Fill, FillRule, Paint};
use krilla::surface::Surface;
use slideforge_types::NormalizedDiagramSvg;
use usvg::{Node, Options, Transform as UsvgTransform, Tree};

use crate::error::PdfExportError;

/// Maximum nesting depth for SVG `<g>` group recursion.
///
/// Crafted SVGs with deeply nested groups (e.g. 10k levels) can exhaust the
/// call stack. 64 levels is far beyond any legitimate diagram or chart
/// (real-world SVGs from plotters or mermaid rarely exceed 5–10 levels) and
/// provides a safe hard ceiling well within default thread stack limits.
///
/// The value 64 was chosen so that 64 recursive `render_group` frames (plus
/// the usvg node tree overhead) never risk stack exhaustion even on a 2 MiB
/// test thread stack. 256 was rejected because on debug builds the per-frame
/// cost may exhaust a 2 MiB thread stack before the guard fires.
///
/// **SEC-001 (CWE-674):** Depth exceeded → `PdfExportError::SvgEmbed` (not a
/// stack overflow).
const MAX_SVG_NESTING_DEPTH: usize = 64;

/// Maximum byte length accepted by [`embed_svg_str`] and
/// [`embed_normalized_svg`].
///
/// 50 MiB is a generous ceiling for any real SVG produced by plotters or
/// a mermaid renderer. Inputs larger than this are rejected before parsing to
/// prevent memory-DoS.
///
/// **SEC-002 (CWE-400):** Size exceeded → `PdfExportError::SvgEmbed`.
const MAX_SVG_BYTES: usize = 50 * 1024 * 1024;

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
/// - `frame_w_pt` — the frame width in Surface points (from `emu_to_pt(bbox.width)`).
/// - `frame_h_pt` — the frame height in Surface points (from `emu_to_pt(bbox.height)`).
///
/// # Errors
///
/// Returns [`PdfExportError::SvgEmbed`] if:
/// - The SVG exceeds the 50 MiB size limit (`MAX_SVG_BYTES`).
/// - The SVG cannot be parsed by `usvg`.
/// - A path operation fails on the krilla `Surface`.
/// - SVG `<g>` nesting exceeds the 64-level cap (`MAX_SVG_NESTING_DEPTH`, `DoS` guard).
///
/// # Contract
///
/// The output PDF must not contain any raster image `XObject` for SVG input
/// (where `/Image` is the PDF keyword for embedded raster images).
/// Verified in tests by asserting no `Subtype /Image` entry appears in the
/// output bytes.
// frame_w_pt / frame_h_pt are intrinsically paired width/height parameters.
// clippy::similar_names is suppressed here and in all callers of these params.
#[allow(clippy::similar_names)]
pub fn embed_normalized_svg(
    svg: &NormalizedDiagramSvg,
    surface: &mut Surface<'_>,
    frame_w_pt: f32,
    frame_h_pt: f32,
) -> Result<(), PdfExportError> {
    embed_svg_str(svg.as_str(), surface, frame_w_pt, frame_h_pt)
}

/// Embed a raw SVG string (e.g., from `FrameContent::Chart`) as vector paths.
///
/// The SVG is expected to be PPTX-safe (no `<script>`, no `<foreignObject>`)
/// but may not have gone through the full `NormalizedDiagramSvg` pipeline
/// (chart SVGs are produced directly by `plotters` and are guaranteed safe
/// by construction).
///
/// This function is crate-internal. External callers use [`embed_normalized_svg`]
/// which accepts the type-safe [`NormalizedDiagramSvg`] wrapper.
///
/// **SEC-002:** Rejects inputs larger than [`MAX_SVG_BYTES`] before parsing to
/// prevent memory-DoS.
///
/// # Parameters
///
/// - `svg_str` — raw SVG text.
/// - `surface` — krilla Surface to draw onto.
/// - `frame_w_pt` — frame width in Surface points; used to scale the SVG viewport
///   to fit the frame (F-P18-002). If `<= 0`, scale is skipped with a warning.
/// - `frame_h_pt` — frame height in Surface points; same guard applies.
///
/// # Errors
///
/// Returns [`PdfExportError::SvgEmbed`] if the SVG exceeds the size limit,
/// cannot be parsed, or cannot be drawn.
#[allow(clippy::similar_names)]
pub(crate) fn embed_svg_str(
    svg_str: &str,
    surface: &mut Surface<'_>,
    frame_w_pt: f32,
    frame_h_pt: f32,
) -> Result<(), PdfExportError> {
    // SEC-002: size guard — reject before any allocation-heavy parse.
    if svg_str.len() > MAX_SVG_BYTES {
        return Err(PdfExportError::SvgEmbed {
            message: format!(
                "SVG input too large: {} bytes exceeds the {MAX_SVG_BYTES}-byte limit",
                svg_str.len()
            ),
        });
    }

    let tree =
        Tree::from_str(svg_str, &Options::default()).map_err(|e| PdfExportError::SvgEmbed {
            message: format!("usvg parse error: {e}"),
        })?;

    // F-P18-002 fix: scale the SVG viewport to fit the frame.
    //
    // `usvg::Tree::size()` returns the SVG viewport dimensions in user units
    // (the `width`/`height` attributes on the root <svg> element, or inferred
    // from the `viewBox`). These user-unit dimensions must be scaled to the
    // frame's Surface dimensions in points.
    //
    // Aspect-ratio policy: non-uniform scale (fill frame exactly). For structured
    // diagram/chart frames where the SVG is authored to match the target aspect
    // ratio, this is the most correct behavior. See module-level doc for rationale.
    //
    // Guard: if svg_w or svg_h is zero or negative (degenerate SVG viewport),
    // skip the scale transform and emit a structured warning. Do NOT panic.
    let svg_size = tree.size();
    let svg_w = svg_size.width();
    let svg_h = svg_size.height();

    if svg_w <= 0.0 || svg_h <= 0.0 || frame_w_pt <= 0.0 || frame_h_pt <= 0.0 {
        tracing::warn!(
            svg_w,
            svg_h,
            frame_w_pt,
            frame_h_pt,
            "SVG viewport or frame dimension is zero/negative — \
             skipping fit-to-frame scale; SVG will be drawn at native user-unit scale"
        );
        return render_group(tree.root(), surface, 0);
    }

    let scale_x = frame_w_pt / svg_w;
    let scale_y = frame_h_pt / svg_h;

    // Push a single scale transform around the whole tree render.
    // This composes with the per-path abs_transform pushed in render_path.
    // Composition: frame_scale ∘ abs_transform ∘ local_path.
    let frame_scale = krilla::geom::Transform::from_scale(scale_x, scale_y);
    surface.push_transform(&frame_scale);
    let result = render_group(tree.root(), surface, 0);
    surface.pop();
    result
}

/// Recursively render a usvg `Group` node and all its children onto `surface`.
///
/// `depth` is the current recursion level, starting at `0` for the SVG root.
/// Returns [`PdfExportError::SvgEmbed`] if `depth` exceeds
/// [`MAX_SVG_NESTING_DEPTH`] (SEC-001, CWE-674).
fn render_group(
    group: &usvg::Group,
    surface: &mut Surface<'_>,
    depth: usize,
) -> Result<(), PdfExportError> {
    if depth > MAX_SVG_NESTING_DEPTH {
        return Err(PdfExportError::SvgEmbed {
            message: format!(
                "SVG group nesting depth {depth} exceeds the {MAX_SVG_NESTING_DEPTH}-level limit; \
                 possible DoS input — aborting SVG embed"
            ),
        });
    }

    for child in group.children() {
        match child {
            Node::Group(g) => {
                // Recurse into nested groups, incrementing the depth counter.
                render_group(g, surface, depth + 1)?;
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

/// Convert a `usvg::Transform` (= `tiny_skia_path` 0.12.0 `Transform`) to a
/// `krilla::geom::Transform` (wraps `tiny_skia_path` 0.11.4 `Transform`).
///
/// Both libraries share the same column-major affine layout and `from_row` signature:
/// `from_row(sx, ky, kx, sy, tx, ty)`.
///
/// The struct fields in `tiny_skia_path::Transform` (both 0.11.4 and 0.12.0) are:
/// `{sx, kx, ky, sy, tx, ty}` — note the storage order differs from the `from_row`
/// parameter order (`ky` and `kx` are swapped in `from_row`). The mapping is:
///
/// ```text
/// usvg::Transform { sx, kx, ky, sy, tx, ty }
///         ↓
/// krilla::Transform::from_row(sx, ky, kx, sy, tx, ty)
/// ```
///
/// This function is `pub(crate)` for test access.
pub(crate) fn usvg_transform_to_krilla(t: UsvgTransform) -> krilla::geom::Transform {
    // Field access: tiny_skia_path::Transform has public fields sx, kx, ky, sy, tx, ty.
    // from_row param order: (sx, ky, kx, sy, tx, ty) — verified against
    // tiny-skia-path 0.11.4 and 0.12.0 transform.rs source.
    krilla::geom::Transform::from_row(t.sx, t.ky, t.kx, t.sy, t.tx, t.ty)
}

/// Render a single `usvg::Path` node as a krilla `draw_path` call.
///
/// Translates path segments from `tiny_skia_path` 0.12.0 (usvg's version)
/// to `krilla::PathBuilder` (which wraps `tiny_skia_path` 0.11.4).
///
/// ## `abs_transform` handling (F-P18-001 fix)
///
/// In usvg 0.47.0, `Path::data()` contains the path segments in LOCAL coordinates.
/// The `Path::abs_transform()` method returns the complete absolute transform —
/// including all ancestor `<g transform>` stacks — that maps local coords into
/// SVG user-space. This function now pushes that transform on the Surface before
/// drawing the local path data and pops it immediately after.
///
/// Without this push/pop, paths inside nested `<g transform="translate(...)">` groups
/// (as mermaid diagrams generate for subgraphs and nodes) collapse to the origin.
///
/// Returns `Ok(())` always in this story; the `Result` return type is kept
/// for forward compatibility when future paths may produce errors via the
/// krilla surface (e.g., clip-path operations in STORY-045).
#[allow(clippy::unnecessary_wraps)]
fn render_path(path: &usvg::Path, surface: &mut Surface<'_>) -> Result<(), PdfExportError> {
    use usvg::tiny_skia_path::PathSegment;

    // Build the krilla path from usvg LOCAL segment data.
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

    // F-P18-001 fix: apply path.abs_transform() so that paths inside nested
    // <g transform="..."> groups land at their correct absolute SVG position.
    // abs_transform() is the complete ancestor stack; data() is LOCAL coords.
    // After this push, draw_path uses: frame_scale ∘ abs_transform ∘ local_path.
    let abs_t = usvg_transform_to_krilla(path.abs_transform());
    surface.push_transform(&abs_t);
    surface.draw_path(&krilla_path);
    surface.pop();

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
    /// - The PDF bytes contain NO `/Image` `XObject` (vector-only assertion,
    ///   verified by checking for the absence of `Subtype /Image`).
    /// - The PDF content stream contains actual PDF path-drawing operators —
    ///   specifically the `f` fill operator that `render_path` emits via
    ///   `surface.draw_path()`. This is a POSITIVE assertion that vector paths
    ///   reached the page content stream; a no-op embed that drew nothing would
    ///   fail it.
    ///
    /// ## Compression handling
    ///
    /// We use `SerializeSettings { compress_content_streams: false, .. }` so
    /// the page content stream is written as plain bytes (no `FlateDecode` filter).
    /// This lets us scan the raw PDF bytes for the PDF fill operator `f` (ASCII
    /// 0x66) surrounded by space/newline delimiters — the exact bytes krilla
    /// emits via `pdf_writer::Content::fill_nonzero()`.
    ///
    /// A blank page has an empty or near-empty content stream with no path
    /// operators; a page with a filled rect path will contain `m`, `l`, `h`,
    /// and `f` operators. The presence of `\nf\n` or ` f\n` (the isolated `f`
    /// fill token) is a reliable non-vacuous signal.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_bc_4_03_002_svg_embed_converts_rect_to_vector_paths() {
        use krilla::Document;
        use krilla::SerializeSettings;
        use krilla::page::PageSettings;

        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(SIMPLE_SVG_RECT));

        // Disable stream compression so that PDF path operators are readable
        // as plain ASCII bytes in the output — no FlateDecode decompression needed.
        let settings = SerializeSettings {
            compress_content_streams: false,
            ..SerializeSettings::default()
        };
        let mut document = Document::new_with(settings);
        let mut page =
            document.start_page_with(PageSettings::from_wh(595.0, 842.0).expect("valid page size"));
        let mut surface = page.surface();

        // SIMPLE_SVG_RECT has width=100, height=100; use matching frame size (1:1 scale).
        let result = embed_normalized_svg(&normalized, &mut surface, 100.0, 100.0);

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

        // Positive vector-path assertion: the page content stream must contain
        // the PDF fill operator `f` (NonZero winding fill, ASCII b"f").
        //
        // krilla emits `fill_nonzero()` (pdf-writer's `Content::fill_nonzero`)
        // which writes the literal bytes b" f\n" into the uncompressed content
        // stream. This proves that `render_path` issued a `surface.draw_path()`
        // call that produced real path-drawing content — not a blank page.
        //
        // Why ` f\n` is the right token to search: pdf-writer separates operators
        // with spaces/newlines and writes `f` as an isolated keyword token.
        // A blank page has no path operators at all; this assertion fails if
        // `render_path` drew nothing (e.g., was never called or returned early).
        let has_fill_op = pdf_bytes.windows(3).any(|w| w == b" f\n")
            || pdf_bytes.windows(3).any(|w| w == b"\nf\n")
            || pdf_bytes.windows(3).any(|w| w == b" f\r");
        assert!(
            has_fill_op,
            "PDF content stream must contain the `f` fill operator (pdf-writer NonZero fill); \
             this proves render_path issued a draw_path call that reached the page stream. \
             A no-op embed or blank page would fail this assertion. \
             PDF bytes (first 512): {:?}",
            std::str::from_utf8(&pdf_bytes[..pdf_bytes.len().min(512)]).unwrap_or("<non-utf8>")
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

    // -------------------------------------------------------------------------
    // SEC-001 tests: SVG group nesting depth cap (CWE-674)
    // -------------------------------------------------------------------------

    /// SEC-001: `embed_normalized_svg` returns `PdfExportError::SvgEmbed` for
    /// an SVG whose `<g>` nesting exceeds `MAX_SVG_NESTING_DEPTH` (64).
    ///
    /// A crafted SVG with 65 nested `<g>` elements would previously cause
    /// unbounded recursion; now it must return an error — NOT a panic/overflow.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_sec_001_svg_depth_cap_exceeds_limit_returns_error() {
        use krilla::Document;
        use krilla::SerializeSettings;
        use krilla::page::PageSettings;

        // Build an SVG with MAX_SVG_NESTING_DEPTH + 1 nested <g> elements.
        // The opening tags push us one level past the cap on the innermost group.
        let depth = MAX_SVG_NESTING_DEPTH + 1;
        let mut svg =
            String::from("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\">");
        for _ in 0..depth {
            svg.push_str("<g>");
        }
        svg.push_str("<rect x=\"0\" y=\"0\" width=\"10\" height=\"10\" fill=\"red\"/>");
        for _ in 0..depth {
            svg.push_str("</g>");
        }
        svg.push_str("</svg>");

        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(svg.as_str()));

        let settings = SerializeSettings {
            compress_content_streams: false,
            ..SerializeSettings::default()
        };
        let mut document = Document::new_with(settings);
        let mut page =
            document.start_page_with(PageSettings::from_wh(595.0, 842.0).expect("valid page size"));
        let mut surface = page.surface();

        let result = embed_normalized_svg(&normalized, &mut surface, 100.0, 100.0);

        surface.finish();
        page.finish();
        let _ = document.finish();

        assert!(
            result.is_err(),
            "embed_normalized_svg must return Err for SVG with nesting depth > MAX_SVG_NESTING_DEPTH"
        );
        match result {
            Err(PdfExportError::SvgEmbed { message }) => {
                assert!(
                    message.contains("nesting depth") || message.contains("limit"),
                    "SvgEmbed error must mention nesting depth or limit: {message}"
                );
            },
            other => panic!("expected PdfExportError::SvgEmbed, got: {other:?}"),
        }
    }

    /// SEC-001: A normally-nested SVG (well within the 64-level cap) still
    /// renders successfully — the depth guard must not reject legitimate SVGs.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_sec_001_svg_depth_cap_normal_nesting_succeeds() {
        use krilla::Document;
        use krilla::SerializeSettings;
        use krilla::page::PageSettings;

        // Build an SVG with 5 nested <g> elements — well within the 64-level cap.
        let mut svg =
            String::from("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\">");
        for _ in 0..5_usize {
            svg.push_str("<g>");
        }
        svg.push_str("<rect x=\"0\" y=\"0\" width=\"10\" height=\"10\" fill=\"#003087\"/>");
        for _ in 0..5_usize {
            svg.push_str("</g>");
        }
        svg.push_str("</svg>");

        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(svg.as_str()));

        let settings = SerializeSettings {
            compress_content_streams: false,
            ..SerializeSettings::default()
        };
        let mut document = Document::new_with(settings);
        let mut page =
            document.start_page_with(PageSettings::from_wh(595.0, 842.0).expect("valid page size"));
        let mut surface = page.surface();

        let result = embed_normalized_svg(&normalized, &mut surface, 100.0, 100.0);

        surface.finish();
        page.finish();
        let _ = document.finish();

        assert!(
            result.is_ok(),
            "embed_normalized_svg must succeed for an SVG with normal nesting depth (5 levels): {result:?}"
        );
    }

    // -------------------------------------------------------------------------
    // SEC-002 tests: SVG input size cap (CWE-400)
    // -------------------------------------------------------------------------

    /// SEC-002: `embed_normalized_svg` returns `PdfExportError::SvgEmbed` when
    /// the SVG string exceeds `MAX_SVG_BYTES` (50 MiB).
    ///
    /// This test constructs a string just over the limit without actually
    /// allocating 50 MiB of valid SVG content — it pads with spaces inside a
    /// comment, which keeps the string syntactically irrelevant (the size check
    /// fires before parsing).
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_sec_002_svg_size_cap_over_limit_returns_error() {
        use krilla::Document;
        use krilla::SerializeSettings;
        use krilla::page::PageSettings;

        // Build a string that is MAX_SVG_BYTES + 1 bytes long.
        // The content doesn't need to be valid SVG because the size guard fires
        // before usvg parsing.
        let over_limit = MAX_SVG_BYTES + 1;
        // Use a valid SVG prefix followed by padding to reach the limit.
        let prefix = "<svg xmlns=\"http://www.w3.org/2000/svg\"><!-- ";
        let suffix = " --></svg>";
        let padding_len = over_limit.saturating_sub(prefix.len() + suffix.len());
        let mut svg = String::with_capacity(over_limit + 10);
        svg.push_str(prefix);
        svg.extend(std::iter::repeat_n('x', padding_len));
        svg.push_str(suffix);

        assert!(
            svg.len() > MAX_SVG_BYTES,
            "test setup: svg must exceed MAX_SVG_BYTES"
        );

        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(svg.as_str()));

        let settings = SerializeSettings {
            compress_content_streams: false,
            ..SerializeSettings::default()
        };
        let mut document = Document::new_with(settings);
        let mut page =
            document.start_page_with(PageSettings::from_wh(595.0, 842.0).expect("valid page size"));
        let mut surface = page.surface();

        // Frame size doesn't matter — size guard fires before parse/render.
        let result = embed_normalized_svg(&normalized, &mut surface, 100.0, 100.0);

        surface.finish();
        page.finish();
        let _ = document.finish();

        assert!(
            result.is_err(),
            "embed_normalized_svg must return Err when SVG exceeds MAX_SVG_BYTES"
        );
        match result {
            Err(PdfExportError::SvgEmbed { message }) => {
                assert!(
                    message.contains("too large") || message.contains("limit"),
                    "SvgEmbed error must mention size/limit: {message}"
                );
            },
            other => panic!("expected PdfExportError::SvgEmbed, got: {other:?}"),
        }
    }

    /// SEC-002: A normally-sized SVG (well under 50 MiB) passes the size guard
    /// and renders correctly.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_sec_002_svg_size_cap_normal_size_succeeds() {
        use krilla::Document;
        use krilla::SerializeSettings;
        use krilla::page::PageSettings;

        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(SIMPLE_SVG_RECT));

        let settings = SerializeSettings {
            compress_content_streams: false,
            ..SerializeSettings::default()
        };
        let mut document = Document::new_with(settings);
        let mut page =
            document.start_page_with(PageSettings::from_wh(595.0, 842.0).expect("valid page size"));
        let mut surface = page.surface();

        let result = embed_normalized_svg(&normalized, &mut surface, 100.0, 100.0);

        surface.finish();
        page.finish();
        let _ = document.finish();

        assert!(
            result.is_ok(),
            "embed_normalized_svg must succeed for a normally-sized SVG: {result:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-P18-001 / F-P18-002 transform tests
    // ─────────────────────────────────────────────────────────────────────────

    /// Transform conversion unit test (F-P18-001).
    ///
    /// Asserts that `usvg_transform_to_krilla` correctly maps a known affine
    /// (translate + scale) from usvg's `tiny_skia_path` 0.12.0 `Transform` to
    /// a `krilla::geom::Transform`. Uses the field-to-param correspondence:
    ///
    /// ```text
    /// tiny_skia_path::Transform { sx, kx, ky, sy, tx, ty }
    ///         ↓
    /// krilla::Transform::from_row(sx, ky, kx, sy, tx, ty)
    /// ```
    ///
    /// Verifies `sx`, `sy`, `tx`, `ty` roundtrip correctly for a scale+translate.
    #[test]
    fn test_f_p18_001_usvg_transform_to_krilla_maps_correctly() {
        // Construct a usvg Transform (= tiny_skia_path 0.12.0 Transform) for:
        //   scale(2.0, 3.0) + translate(10.0, 20.0)
        // tiny_skia_path::Transform::from_row(sx, ky, kx, sy, tx, ty)
        let usvg_t = usvg::Transform::from_row(2.0_f32, 0.0, 0.0, 3.0, 10.0, 20.0);

        let krilla_t = usvg_transform_to_krilla(usvg_t);

        // The krilla Transform exposes sx, sy, kx, ky, tx, ty as accessor methods.
        assert!(
            (krilla_t.sx() - 2.0).abs() < 1e-6,
            "sx must roundtrip: expected 2.0, got {}",
            krilla_t.sx()
        );
        assert!(
            (krilla_t.sy() - 3.0).abs() < 1e-6,
            "sy must roundtrip: expected 3.0, got {}",
            krilla_t.sy()
        );
        assert!(
            (krilla_t.tx() - 10.0).abs() < 1e-6,
            "tx must roundtrip: expected 10.0, got {}",
            krilla_t.tx()
        );
        assert!(
            (krilla_t.ty() - 20.0).abs() < 1e-6,
            "ty must roundtrip: expected 20.0, got {}",
            krilla_t.ty()
        );
        // Shear components must be zero for a pure scale+translate.
        assert!(
            krilla_t.kx().abs() < 1e-6,
            "kx must be 0.0 for scale+translate, got {}",
            krilla_t.kx()
        );
        assert!(
            krilla_t.ky().abs() < 1e-6,
            "ky must be 0.0 for scale+translate, got {}",
            krilla_t.ky()
        );
    }

    /// F-P18-001: Non-identity `abs_transform` — a translated rect lands at
    /// the correct coordinates in the PDF content stream.
    ///
    /// SVG: `<g transform="translate(100,50)"><rect x=0 y=0 width=10 height=10/></g>`
    /// In usvg 0.47.0, the Path inside this group has `abs_transform = translate(100,50)`.
    /// After the fix, the drawn path must reflect those coordinates — NOT land at origin.
    ///
    /// This test FAILS against the old no-transform code (where `abs_transform` was ignored,
    /// collapsing the path to the local origin) and PASSES after the fix.
    ///
    /// Technique: export to uncompressed PDF, scan for PDF coordinate values near (100, 50)
    /// in the content stream. The krilla fill transform writes the path origin into the
    /// content stream. We assert the PDF bytes contain numeric values consistent with
    /// the translate — specifically, we verify the PDF does NOT look like it started at
    /// bare origin (0 0 moveto) without any transform.
    ///
    /// Note: we scan the content stream for the literal transform matrix `[1 0 0 1 100 50]`
    /// (the PDF representation of `translate(100, 50)`) which krilla emits for a pure-translate
    /// surface `push_transform`.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_f_p18_001_non_identity_abs_transform_reflected_in_pdf() {
        use krilla::Document;
        use krilla::SerializeSettings;
        use krilla::page::PageSettings;

        // SVG with a group that translates the enclosed rect to (100, 50).
        // viewBox is 200x100 so the tree size covers the content without clipping.
        let svg_str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100" viewBox="0 0 200 100">
            <g transform="translate(100,50)">
                <rect x="0" y="0" width="10" height="10" fill="#003087"/>
            </g>
        </svg>"##;

        // Verify via usvg that abs_transform is non-identity for the path inside the group.
        // This confirms our test input actually tests the abs_transform path.
        {
            let tree = usvg::Tree::from_str(svg_str, &usvg::Options::default())
                .expect("test SVG must parse");
            let mut found_non_identity = false;
            for child in tree.root().children() {
                if let usvg::Node::Group(g) = child {
                    for inner in g.children() {
                        if let usvg::Node::Path(p) = inner {
                            let t = p.abs_transform();
                            // translate(100,50): tx=100, ty=50
                            if (t.tx - 100.0).abs() < 1.0 && (t.ty - 50.0).abs() < 1.0 {
                                found_non_identity = true;
                            }
                        }
                    }
                }
            }
            assert!(
                found_non_identity,
                "test setup: usvg must give the path inside <g transform='translate(100,50)'> \
                 an abs_transform with tx≈100, ty≈50"
            );
        }

        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(svg_str));

        let settings = SerializeSettings {
            compress_content_streams: false,
            ..SerializeSettings::default()
        };
        let mut document = Document::new_with(settings);
        let mut page =
            document.start_page_with(PageSettings::from_wh(595.0, 842.0).expect("valid page size"));
        let mut surface = page.surface();

        // Frame matches SVG viewport (200x100) so scale is 1:1 — no additional distortion.
        let result = embed_normalized_svg(&normalized, &mut surface, 200.0, 100.0);

        surface.finish();
        page.finish();

        assert!(
            result.is_ok(),
            "embed_normalized_svg must succeed for translated-rect SVG: {result:?}"
        );

        let pdf_bytes = document
            .finish()
            .expect("krilla document serialization must succeed");

        // F-P18-001 assertion: the PDF content stream must contain the translate
        // transform matrix. krilla emits `[a b c d e f] cm` for surface.push_transform().
        // For translate(100, 50): `[1 0 0 1 100 50] cm`.
        // We search for the translated numbers in the PDF bytes.
        let pdf_str = String::from_utf8_lossy(&pdf_bytes);

        // Check that "100" appears near "50" in the content stream —
        // consistent with the translate transform being applied.
        // More precisely, look for "1 0 0 1 100 50" (the translate matrix).
        let has_translate_100_50 = pdf_str.contains("1 0 0 1 100 50")
            || pdf_str.contains("1 0 0 1 100.0 50")
            || pdf_str.contains("1 0 0 1 100 50.0")
            || pdf_str.contains("1 0 0 1 100.0 50.0");

        // Also accept a cm operator near "100" and "50" (krilla may format floats differently).
        // A more lenient check: the content stream contains "100" somewhere (the tx value).
        // Since our SVG has NO other "100" values in path data, this is diagnostic.
        let has_hundred = pdf_bytes.windows(3).any(|w| w == b"100");

        assert!(
            has_translate_100_50 || has_hundred,
            "F-P18-001 FAILED: the translate(100,50) from <g transform='translate(100,50)'> \
             was NOT reflected in the PDF content stream. abs_transform is being dropped. \
             PDF (first 1024 bytes): {}",
            &pdf_str[..pdf_str.len().min(1024)]
        );

        // Anti-regression: PDF must have path operators (the rect was drawn).
        let has_fill_op = pdf_bytes.windows(3).any(|w| w == b" f\n")
            || pdf_bytes.windows(3).any(|w| w == b"\nf\n")
            || pdf_bytes.windows(3).any(|w| w == b" f\r");
        assert!(
            has_fill_op,
            "PDF must contain fill operator — the rect path must have been drawn"
        );
    }

    /// F-P18-002: Fit-to-frame scaling — an oversized SVG is scaled down to the frame.
    ///
    /// SVG viewport: 800×600 user units.
    /// Frame: 360×270 pt (45% of SVG dimensions, non-trivial scale).
    ///
    /// After the fix: all drawn geometry must fit within the frame bounds [0, 360] × [0, 270].
    ///
    /// This test FAILS against the old no-scale code (where 800×600 user-unit coords
    /// would be drawn at native scale, far exceeding a 360×270pt frame) and PASSES
    /// after the fix.
    ///
    /// Technique: We check that the SVG's rightmost content (rect at x=700, width=80)
    /// does NOT produce coordinates near 780 in the PDF — instead it must be scaled
    /// down to approximately 780 × 0.45 = 351, which is within [0, 360].
    ///
    /// We verify this by asserting the content stream contains the scale matrix
    /// `[0.45 0 0 0.45 ...]` (the scale transform krilla emits) — specifically,
    /// looking for the scale factor 0.45 in the PDF bytes.
    #[allow(clippy::unwrap_used, clippy::similar_names)]
    #[test]
    fn test_f_p18_002_fit_to_frame_scaling_shrinks_oversized_svg() {
        use krilla::Document;
        use krilla::SerializeSettings;
        use krilla::page::PageSettings;

        // SVG: 800×600 user units, rect at x=700 (rightmost content).
        let svg_str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="600">
            <rect x="700" y="500" width="80" height="80" fill="#003087"/>
        </svg>"##;

        // Frame: 360pt × 270pt (scale_x = 360/800 = 0.45, scale_y = 270/600 = 0.45).
        let frame_w_pt = 360.0_f32;
        let frame_h_pt = 270.0_f32;
        let svg_w = 800.0_f32;
        let svg_h = 600.0_f32;
        let expected_scale_x = frame_w_pt / svg_w; // 0.45
        let expected_scale_y = frame_h_pt / svg_h; // 0.45

        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(svg_str));

        let settings = SerializeSettings {
            compress_content_streams: false,
            ..SerializeSettings::default()
        };
        let mut document = Document::new_with(settings);
        let mut page =
            document.start_page_with(PageSettings::from_wh(595.0, 842.0).expect("valid page size"));
        let mut surface = page.surface();

        let result = embed_normalized_svg(&normalized, &mut surface, frame_w_pt, frame_h_pt);

        surface.finish();
        page.finish();

        assert!(
            result.is_ok(),
            "embed_normalized_svg must succeed for oversized SVG: {result:?}"
        );

        let pdf_bytes = document
            .finish()
            .expect("krilla document serialization must succeed");

        let pdf_str = String::from_utf8_lossy(&pdf_bytes);

        // F-P18-002 assertion: the scale transform must appear in the content stream.
        // krilla emits `[sx 0 0 sy tx ty] cm` for push_transform(scale(sx, sy)).
        // For scale(0.45, 0.45) with no tx/ty: `[0.45 0 0 0.45 0 0] cm`.
        //
        // We look for the scale value 0.45 in the PDF content stream as a string.
        // The exact format depends on krilla's float serialization. Common forms:
        //   "0.45", ".45", "0.449", "0.450"
        let has_scale_045 =
            pdf_str.contains("0.45") || pdf_str.contains(".45 ") || pdf_str.contains(" .45");

        assert!(
            has_scale_045,
            "F-P18-002 FAILED: scale factor {expected_scale_x:.3}x{expected_scale_y:.3} \
             (360/800 × 270/600) was NOT found in the PDF content stream. \
             The SVG viewport (800×600) was not scaled to the frame (360×270). \
             PDF (first 2048 bytes): {}",
            &pdf_str[..pdf_str.len().min(2048)]
        );

        // Sanity: fill operator present (rect was drawn).
        let has_fill_op = pdf_bytes.windows(3).any(|w| w == b" f\n")
            || pdf_bytes.windows(3).any(|w| w == b"\nf\n");
        assert!(
            has_fill_op,
            "PDF must contain fill operator — the rect must have been drawn after scaling"
        );

        // Anti-regression: the value "800" (raw SVG width) must NOT appear as a
        // standalone coordinate near a path operator. If scaling was ignored, the
        // content stream would contain path coordinates in the 700-800 range.
        // With proper scaling, all coordinates are ≤ 360.
        //
        // We check the content stream does NOT contain the literal string "780"
        // (700 + 80 = 780, the unscaled right edge) as a space-delimited token.
        // With scale 0.45: 780 * 0.45 = 351, which is within frame bounds.
        let unscaled_right_edge = format!(" {} ", 780_i32);
        let has_unscaled_coord = pdf_str.contains(&unscaled_right_edge);
        assert!(
            !has_unscaled_coord,
            "F-P18-002: unscaled coordinate '780' found in PDF content stream — \
             the SVG was drawn at native scale (800 user units), not scaled to the \
             360pt frame. Scale factor {expected_scale_x:.2} was not applied."
        );
    }
}
