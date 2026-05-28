//! PDF vector-path renderer for `slideforge-math`.
//!
//! Converts a [`MathAst`] to a vector-path SVG string suitable for embedding
//! in PDF output. All text in the SVG is converted to `<path d="...">` glyph
//! outlines — no `<text>` elements remain — ensuring PDF/UA-1 compliance via
//! the `usvg` normalisation pass.
//!
//! ## Approach (STORY-030 implementation target)
//!
//! 1. Render [`MathAst`] to an intermediate SVG string via a font-glyph outline
//!    engine (`ab_glyph` or `ttf-parser`).
//! 2. Normalise the intermediate SVG with `usvg` to remove any residual `<text>`
//!    elements.
//! 3. If any `<text>` element survives normalisation, return
//!    [`MathError::TextRemainsInPathOutput`].
//!
//! ## Constraints
//!
//! - **Pure function**: no I/O, no spawned processes (Architecture Rule 1).
//! - **No `<text>` elements** in the output (PDF/UA-1, DI-014).
//! - No `<image>` elements; all geometry must be `<path d="...">` (AC-003).

use slideforge_plugin_api::MathError;

use crate::MathAst;

/// An SVG string in which all math glyphs have been converted to path data.
///
/// Produced by [`render_pdf_paths`]. The contained SVG has no `<text>` or
/// `<image>` elements — only `<path>` elements with absolute coordinates.
///
/// The struct also carries the bounding-box dimensions in EMUs so that PDF
/// and PPTX exporters can position the math object without re-parsing the SVG.
///
/// ## Invariant
///
/// `svg` must not contain any `<text>` elements. Call sites may assert this
/// in debug builds; the renderer itself guarantees it at construction time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SvgPaths {
    /// The complete SVG string with all glyphs rendered as `<path>` data.
    pub svg: String,
    /// Bounding-box width in EMUs (914 400 per inch).
    pub width_emu: i64,
    /// Bounding-box height in EMUs (914 400 per inch).
    pub height_emu: i64,
}

/// Render a [`MathAst`] to a vector-path SVG string for PDF embedding.
///
/// The returned [`SvgPaths`] contains no `<text>` elements; every glyph has
/// been expanded to `<path d="...">` outlines. If the normalisation step
/// cannot remove all `<text>` elements (e.g., the font engine does not cover
/// a glyph), `Err(MathError::TextRemainsInPathOutput)` is returned.
///
/// # Errors
///
/// - [`MathError::TextRemainsInPathOutput`] — one or more `<text>` elements
///   survived SVG normalisation (EC-005).
/// - [`MathError::RenderError`] — internal rendering failure (e.g., missing
///   glyph outline data for a required character).
#[allow(unused_variables)] // parameter used in implementation — stub only
pub fn render_pdf_paths(ast: &MathAst) -> Result<SvgPaths, MathError> {
    todo!("STORY-030: implement PDF path renderer — see AC-003 and EC-005")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::ast::{MathMode, MathNode};

    // Helper: build a minimal MathAst inline expression.
    fn inline_ast(nodes: Vec<MathNode>) -> MathAst {
        MathAst::new(MathMode::Inline, nodes)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-1.10.003 — AC-003: PDF path output — vector SVG, no text characters
    // ─────────────────────────────────────────────────────────────────────────

    /// `render_pdf_paths` succeeds for a simple expression and returns `SvgPaths`.
    ///
    /// RED GATE: must fail with `todo!()` until implemented.
    #[test]
    #[should_panic(expected = "STORY-030")]
    fn test_bc_1_10_003_pdf_paths_returns_svg_paths() {
        let ast = inline_ast(vec![MathNode::Text(Arc::from("x"))]);
        let _ = render_pdf_paths(&ast).unwrap();
    }

    /// `render_pdf_paths` output SVG contains no `<text>` elements.
    ///
    /// RED GATE: must fail with `todo!()` until implemented.
    #[test]
    #[should_panic(expected = "STORY-030")]
    fn test_bc_1_10_003_pdf_paths_no_text_elements_in_svg() {
        let ast = inline_ast(vec![MathNode::Text(Arc::from("E"))]);
        let paths = render_pdf_paths(&ast).unwrap();
        assert!(
            !paths.svg.contains("<text"),
            "SVG must not contain <text> elements; got: {}",
            &paths.svg[..paths.svg.len().min(200)]
        );
    }

    /// `render_pdf_paths` output SVG contains no `<image>` elements.
    ///
    /// RED GATE: must fail with `todo!()` until implemented.
    #[test]
    #[should_panic(expected = "STORY-030")]
    fn test_bc_1_10_003_pdf_paths_no_image_elements_in_svg() {
        let ast = inline_ast(vec![MathNode::Text(Arc::from("E"))]);
        let paths = render_pdf_paths(&ast).unwrap();
        assert!(
            !paths.svg.contains("<image"),
            "SVG must not contain <image> elements; got: {}",
            &paths.svg[..paths.svg.len().min(200)]
        );
    }

    /// `render_pdf_paths` returns positive EMU dimensions.
    ///
    /// RED GATE: must fail with `todo!()` until implemented.
    #[test]
    #[should_panic(expected = "STORY-030")]
    fn test_bc_1_10_003_pdf_paths_positive_emu_dimensions() {
        let ast = inline_ast(vec![MathNode::Text(Arc::from("x"))]);
        let paths = render_pdf_paths(&ast).unwrap();
        assert!(paths.width_emu > 0, "width_emu must be positive");
        assert!(paths.height_emu > 0, "height_emu must be positive");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-1.10.003 — EC-005: usvg fails to normalise → TextRemainsInPathOutput
    // ─────────────────────────────────────────────────────────────────────────

    /// `SvgPaths` struct fields are accessible and the type is `Clone + PartialEq`.
    ///
    /// This is a compile-time property test (no `todo!()` involved).
    #[test]
    fn test_bc_1_10_003_svg_paths_struct_fields_accessible() {
        let paths = SvgPaths {
            svg: "<svg/>".to_owned(),
            width_emu: 914_400,
            height_emu: 457_200,
        };
        let cloned = paths.clone();
        assert_eq!(paths, cloned);
        assert_eq!(cloned.width_emu, 914_400);
        assert_eq!(cloned.height_emu, 457_200);
    }
}
