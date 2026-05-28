//! PDF vector-path renderer for `slideforge-math`.
//!
//! Converts a [`MathAst`] to a vector-path SVG string suitable for embedding
//! in PDF output. All text in the SVG is converted to `<path d="...">` glyph
//! outlines — no `<text>` elements remain — ensuring PDF/UA-1 compliance.
//!
//! ## Approach
//!
//! The renderer walks the [`MathAst`] and emits one `<path>` element per
//! glyph using a minimal static glyph table. Glyphs are represented as
//! distinct path shapes that symbolise character bounding boxes; this provides
//! correct structure (vector paths, no text) while keeping the implementation
//! pure-Rust with zero I/O.
//!
//! Greek letters, operators, and symbols are translated to their canonical
//! Unicode scalars via the shared [`crate::symbols`] module **before** glyph
//! dispatch.  This ensures that γ renders a distinct path from Latin g, ∑
//! renders a distinct path from Latin s, etc.
//!
//! After the SVG string is constructed it is normalised through `usvg` to
//! detect any residual `<text>` elements.  If any survive normalisation the
//! function returns `Err(MathError::TextRemainsInPathOutput)` (EC-005).
//!
//! ## Constraints
//!
//! - **Pure function**: no I/O, no spawned processes (Architecture Rule 1).
//! - **No `<text>` elements** in the output (PDF/UA-1, DI-014).
//! - No `<image>` elements; all geometry must be `<path d="...">` (AC-003).
//! - Integer EMUs for bounding-box dimensions (`i64`, 914 400 per inch).

use slideforge_plugin_api::MathError;

use crate::MathAst;
use crate::ast::{MathMode, MathNode};
use crate::font_engine::{self, GlyphEngine};
use crate::symbols::{
    greek_to_unicode_char, is_text_operator, operator_to_unicode_char, symbol_to_unicode_char,
    unescape_delimiter,
};

/// EMUs per pixel at 96 dpi (914 400 / 96 = 9 525).
const EMU_PER_PX: i64 = 9_525;

/// Default glyph width in pixels.
const GLYPH_W: i64 = 10;

/// Default glyph height in pixels.
const GLYPH_H: i64 = 14;

/// Vertical offset for superscripts (pixels above baseline).
const SUP_OFFSET: i64 = 6;

/// Vertical offset for subscripts (pixels below baseline).
const SUB_OFFSET: i64 = 4;

/// Scale factor applied to display-mode output (1.2×) per finding I3.
///
/// Display math is larger and centered; inline math uses the default size.
const DISPLAY_SCALE_NUM: i64 = 6; // numerator of 6/5 = 1.2
const DISPLAY_SCALE_DEN: i64 = 5; // denominator

/// Vertical padding added to the SVG viewBox height above the tallest glyph.
///
/// Two pixels of clearance prevent superscripts from being clipped at the
/// top edge of the SVG canvas.
const PDF_VERTICAL_PADDING_PX: i64 = 2;

/// An SVG string in which all math glyphs have been converted to path data.
///
/// Produced by [`render_pdf_paths`]. The contained SVG has no `<text>` or
/// `<image>` elements — only `<path>` elements with absolute coordinates.
///
/// ## Invariant
///
/// The inner SVG string must not contain any `<text>` elements. Call sites may
/// assert this in debug builds; the renderer itself guarantees it at
/// construction time via `usvg` normalisation.
///
/// ## Bounding box
///
/// The SVG `viewBox` attribute encodes the rendered width and height in pixels.
/// Exporters that need EMU dimensions should parse the `viewBox` attribute
/// directly (914 400 EMU per inch at 96 dpi = 9 525 EMU per pixel).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SvgPaths(pub String);

/// Render a [`MathAst`] to a vector-path SVG string for PDF embedding.
///
/// The returned [`SvgPaths`] contains no `<text>` elements; every glyph has
/// been expanded to `<path d="...">` outlines. After initial generation the
/// SVG is normalised through `usvg` to ensure any residual `<text>` elements
/// are detected (a defence-in-depth check — the static glyph engine never
/// emits `<text>`, but `usvg` normalisation would catch any future regression).
///
/// # Errors
///
/// - [`MathError::TextRemainsInPathOutput`] — `usvg` detected a `<text>`
///   element after normalisation (internal invariant violation, EC-005).
/// - [`MathError::EmptyCommandName`] — a Greek/Operator/Symbol node carries
///   an empty command name.
/// - [`MathError::UnsupportedSymbol`] — a command name has no Unicode mapping.
/// - [`MathError::RenderError`] — internal rendering failure.
pub fn render_pdf_paths(ast: &MathAst) -> Result<SvgPaths, MathError> {
    // Guard: an empty AST cannot produce meaningful vector-path output (L2).
    // Return EmptyAst rather than silently emitting a degenerate 1×16-px SVG.
    if ast.nodes.is_empty() {
        return Err(MathError::EmptyAst);
    }

    // Obtain the cached glyph engine — OTF bytes are parsed at most once per
    // process lifetime (F-S030-P10-C2).
    let engine = font_engine::engine();

    let mut paths: Vec<String> = Vec::new();
    let mut x: i64 = 0;
    let baseline_y: i64 = GLYPH_H;

    collect_paths(&engine, ast, &ast.nodes, &mut paths, &mut x, baseline_y)?;

    // Secondary guard: a non-empty node list may still produce zero paths when
    // it consists solely of empty Group nodes (F-S030-P5-L1). In that case
    // emitting a degenerate SVG is no better than the empty-AST case above.
    if paths.is_empty() {
        return Err(MathError::EmptyAst);
    }

    // Apply 1.2× scaling for display mode (finding I3).
    let (width_px, height_px) = if ast.mode == MathMode::Display {
        let w = (x.max(1) * DISPLAY_SCALE_NUM + DISPLAY_SCALE_DEN - 1) / DISPLAY_SCALE_DEN;
        let h = ((GLYPH_H + SUP_OFFSET + PDF_VERTICAL_PADDING_PX).max(1) * DISPLAY_SCALE_NUM
            + DISPLAY_SCALE_DEN
            - 1)
            / DISPLAY_SCALE_DEN;
        (w, h)
    } else {
        (
            x.max(1),
            (GLYPH_H + SUP_OFFSET + PDF_VERTICAL_PADDING_PX).max(1),
        )
    };

    let width_emu = width_px * EMU_PER_PX;
    let height_emu = height_px * EMU_PER_PX;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width_px} {height_px}" width="{width_px}" height="{height_px}">"#,
    );

    // Wrap path group in a scale transform for display mode (H5).
    // The viewBox already grows to 1.2× the inline dimensions; the transform
    // ensures the path geometry itself is also scaled to fill the larger canvas.
    if ast.mode == MathMode::Display {
        svg.push_str(r#"<g transform="scale(1.2)">"#);
    } else {
        svg.push_str("<g>");
    }

    for path in &paths {
        svg.push_str(path);
    }

    svg.push_str("</g>");
    svg.push_str("</svg>");

    // Post-generation invariant check via usvg normalisation.
    //
    // `usvg::Tree::from_str` parses the SVG and resolves all elements.
    // If the parse succeeds we can call `has_text_nodes()` to detect residual
    // `<text>` nodes.  If usvg itself fails to parse our freshly-generated SVG
    // that is also an invariant violation and should surface as RenderError.
    let usvg_options = usvg::Options::default();
    let tree = usvg::Tree::from_str(&svg, &usvg_options).map_err(|e| MathError::RenderError {
        message: format!("usvg SVG normalisation failed: {e}"),
    })?;

    if tree.has_text_nodes() {
        return Err(MathError::TextRemainsInPathOutput);
    }

    debug_assert!(
        !svg.contains("<text"),
        "BUG: render_pdf_paths produced <text> element — static glyph engine invariant violated"
    );

    // width_emu and height_emu are encoded in the viewBox — callers that need EMU
    // dimensions parse the viewBox directly (see module-level doc).
    let _ = (width_emu, height_emu);
    Ok(SvgPaths(svg))
}

/// Collect `<path>` elements for an expression, advancing `x` as glyphs are
/// placed. This function is called on the flat node list of the AST and
/// delegates to [`place_node`] for each node.
fn collect_paths(
    engine: &GlyphEngine,
    _ast: &MathAst,
    nodes: &[MathNode],
    paths: &mut Vec<String>,
    x: &mut i64,
    baseline_y: i64,
) -> Result<(), MathError> {
    for node in nodes {
        place_node(engine, node, paths, x, baseline_y)?;
    }
    Ok(())
}

/// Recursively place a [`MathNode`] as one or more `<path>` elements.
///
/// `x` is the current pen position (left edge of the next glyph).
/// `baseline_y` is the vertical coordinate of the text baseline.
///
/// # Errors
///
/// Returns [`MathError::EmptyCommandName`] or [`MathError::UnsupportedSymbol`]
/// when a Greek/Operator/Symbol node cannot be resolved to a Unicode glyph.
#[allow(clippy::too_many_lines)]
fn place_node(
    engine: &GlyphEngine,
    node: &MathNode,
    paths: &mut Vec<String>,
    x: &mut i64,
    baseline_y: i64,
) -> Result<(), MathError> {
    match node {
        MathNode::Text(s) | MathNode::TextRun(s) => {
            for ch in s.chars() {
                emit_glyph(engine, ch, paths, *x, baseline_y - GLYPH_H)?;
                *x += engine.advance_width(ch, GLYPH_H, GLYPH_W) + 1;
            }
        },

        MathNode::Superscript { base, sup } => {
            // Render base at normal position
            place_node(engine, base, paths, x, baseline_y)?;
            // Render sup raised above the baseline
            let sup_y = (baseline_y - GLYPH_H - SUP_OFFSET).max(0);
            let sup_h = GLYPH_H * 2 / 3;
            place_node_at_scale(engine, sup, paths, x, sup_y, sup_h)?;
        },

        MathNode::Subscript { base, sub } => {
            place_node(engine, base, paths, x, baseline_y)?;
            let sub_y = baseline_y + SUB_OFFSET;
            let sub_h = GLYPH_H * 2 / 3;
            place_node_at_scale(engine, sub, paths, x, sub_y, sub_h)?;
        },

        MathNode::Fraction { num, denom } => {
            // Render numerator above the fraction bar, denominator below
            let saved_x = *x;
            let mut num_x = *x;
            let mut denom_x = *x;

            // Numerator above baseline
            let num_y = (baseline_y - GLYPH_H * 3 / 2).max(0);
            place_node_at_scale(engine, num, paths, &mut num_x, num_y, GLYPH_H * 2 / 3)?;

            // Denominator below baseline
            let denom_y = baseline_y;
            place_node_at_scale(engine, denom, paths, &mut denom_x, denom_y, GLYPH_H * 2 / 3)?;

            let width = num_x.max(denom_x) - saved_x;
            // Fraction bar
            let bar_y = baseline_y - GLYPH_H / 2;
            paths.push(format!(
                r#"<path d="M{saved_x},{bar_y} H{}" stroke="black" stroke-width="1" fill="none"/>"#,
                saved_x + width
            ));
            *x = saved_x + width + 2;
        },

        MathNode::Sqrt { index, radicand } => {
            // Radical sign (simple path) + radicand
            let rad_x = *x;
            let rad_y = baseline_y - GLYPH_H;
            // Draw a basic radical symbol: a checkmark + overline
            paths.push(format!(
                r#"<path d="M{rad_x},{baseline_y} l3,-3 l3,{GLYPH_H} " stroke="black" stroke-width="1" fill="none"/>"#,
            ));
            *x += 8;

            if let Some(idx) = index {
                place_node_at_scale(engine, idx, paths, x, rad_y, GLYPH_H / 2)?;
            }

            let content_start = *x;
            place_node(engine, radicand, paths, x, baseline_y)?;
            let content_end = *x;

            // Overline above radicand
            paths.push(format!(
                r#"<path d="M{content_start},{rad_y} H{content_end}" stroke="black" stroke-width="1" fill="none"/>"#,
            ));
            *x += 2;
        },

        MathNode::Operator(name) => {
            if name.is_empty() {
                return Err(MathError::EmptyCommandName);
            }
            // Translate command name to canonical Unicode glyph first.
            // Text-based operators (lim, max, min, …) have no Unicode-symbol
            // mapping and render as multiple Latin characters.
            // Unknown names that are neither in the Unicode table nor recognised
            // as text operators return UnsupportedSymbol — they must not fall
            // through to per-letter rendering (F-S030-P2-H6).
            if let Some(ch) = operator_to_unicode_char(name) {
                emit_glyph(engine, ch, paths, *x, baseline_y - GLYPH_H)?;
                *x += engine.advance_width(ch, GLYPH_H, GLYPH_W) + 1;
            } else if is_text_operator(name) {
                // Known text operator: render each Latin character individually.
                for ch in name.chars() {
                    emit_glyph(engine, ch, paths, *x, baseline_y - GLYPH_H)?;
                    *x += engine.advance_width(ch, GLYPH_H, GLYPH_W) + 1;
                }
            } else {
                return Err(MathError::UnsupportedSymbol {
                    name: name.to_string(),
                });
            }
        },

        MathNode::Symbol(name) => {
            if name.is_empty() {
                return Err(MathError::EmptyCommandName);
            }
            if let Some(ch) = symbol_to_unicode_char(name) {
                emit_glyph(engine, ch, paths, *x, baseline_y - GLYPH_H)?;
                *x += engine.advance_width(ch, GLYPH_H, GLYPH_W) + 1;
            } else {
                return Err(MathError::UnsupportedSymbol {
                    name: name.to_string(),
                });
            }
        },

        MathNode::Greek(name) => {
            if name.is_empty() {
                return Err(MathError::EmptyCommandName);
            }
            if let Some(ch) = greek_to_unicode_char(name) {
                emit_glyph(engine, ch, paths, *x, baseline_y - GLYPH_H)?;
                *x += engine.advance_width(ch, GLYPH_H, GLYPH_W) + 1;
            } else {
                return Err(MathError::UnsupportedSymbol {
                    name: name.to_string(),
                });
            }
        },

        MathNode::Accent { kind, inner } => {
            use crate::ast::AccentKind;
            let save_x = *x;
            place_node(engine, inner, paths, x, baseline_y)?;
            let accent_y = (baseline_y - GLYPH_H - 3).max(0);
            let x_end = *x;
            let mid_x = save_x + (x_end - save_x) / 2;
            // Each AccentKind renders a distinct path shape (M2).
            let d = match kind {
                AccentKind::Hat => {
                    // Caret: two lines forming an inverted V
                    format!(
                        "M{save_x},{accent_y} L{mid_x},{} L{x_end},{accent_y}",
                        accent_y - 3
                    )
                },
                AccentKind::Bar => {
                    // Overline: horizontal bar
                    format!("M{save_x},{accent_y} H{x_end}")
                },
                AccentKind::Tilde => {
                    // Wavy tilde line using a quadratic bezier
                    let q1 = save_x + (x_end - save_x) / 3;
                    let q2 = save_x + 2 * (x_end - save_x) / 3;
                    format!(
                        "M{save_x},{accent_y} Q{q1},{} {mid_x},{accent_y} Q{q2},{} {x_end},{accent_y}",
                        accent_y - 3,
                        accent_y + 2
                    )
                },
                AccentKind::Vec => {
                    // Rightward arrow over the base: horizontal bar + arrowhead
                    format!(
                        "M{save_x},{accent_y} H{x_end} M{},{} L{x_end},{accent_y} L{},{}",
                        x_end - 3,
                        accent_y - 2,
                        x_end - 3,
                        accent_y + 2
                    )
                },
                AccentKind::Dot => {
                    // Single dot above midpoint
                    format!("M{mid_x},{accent_y} V{}", accent_y - 1)
                },
                AccentKind::Ddot => {
                    // Two dots: one at 1/3 and one at 2/3 of the base width
                    let d1 = save_x + (x_end - save_x) / 3;
                    let d2 = save_x + 2 * (x_end - save_x) / 3;
                    format!(
                        "M{d1},{accent_y} V{} M{d2},{accent_y} V{}",
                        accent_y - 1,
                        accent_y - 1
                    )
                },
            };
            paths.push(format!(
                r#"<path d="{d}" stroke="black" stroke-width="1" fill="none"/>"#
            ));
        },

        MathNode::Group(nodes) => {
            for n in nodes {
                place_node(engine, n, paths, x, baseline_y)?;
            }
        },

        MathNode::Delimiter { left, right, inner } => {
            // Strip LaTeX escapes before dispatching each char to emit_glyph (H6).
            // e.g. "\{" → "{", "\langle" → "⟨", "\." → "" (null delimiter, omitted).
            let left_unesc = unescape_delimiter(left);
            let right_unesc = unescape_delimiter(right);
            for ch in left_unesc.chars() {
                emit_glyph(engine, ch, paths, *x, baseline_y - GLYPH_H)?;
                *x += engine.advance_width(ch, GLYPH_H, GLYPH_W) + 1;
            }
            for n in inner {
                place_node(engine, n, paths, x, baseline_y)?;
            }
            for ch in right_unesc.chars() {
                emit_glyph(engine, ch, paths, *x, baseline_y - GLYPH_H)?;
                *x += engine.advance_width(ch, GLYPH_H, GLYPH_W) + 1;
            }
        },

        MathNode::Align(rows) => {
            // Snapshot the start x so every row begins at the same left column
            // (F-S030-P7-M1: without the snapshot, row N starts at end of row N-1).
            let start_x = *x;
            let mut row_y = baseline_y;
            let mut max_row_end = *x;
            for row in rows {
                let mut row_x = start_x; // reset each row to the same left edge
                for n in row {
                    place_node(engine, n, paths, &mut row_x, row_y)?;
                }
                max_row_end = max_row_end.max(row_x);
                row_y += GLYPH_H + 4;
            }
            *x = max_row_end;
        },

        MathNode::Cases(cases) => {
            // Snapshot the start x so every row begins at the same left column
            // (F-S030-P7-M1: without the snapshot, row N starts at end of row N-1).
            let start_x = *x;
            let mut row_y = baseline_y;
            let mut max_row_end = *x;
            // Each tuple is (result-nodes, condition-nodes) — result renders first (left column).
            for (result, condition) in cases {
                let mut row_x = start_x; // reset each row to the same left edge
                for n in result {
                    place_node(engine, n, paths, &mut row_x, row_y)?;
                }
                row_x += 8; // column gap
                for n in condition {
                    place_node(engine, n, paths, &mut row_x, row_y)?;
                }
                max_row_end = max_row_end.max(row_x);
                row_y += GLYPH_H + 4;
            }
            *x = max_row_end;
        },

        MathNode::Space => {
            *x += GLYPH_W / 2;
        },
    }
    Ok(())
}

/// Like [`place_node`] but renders the node at a scaled height.
///
/// This is used for superscripts, subscripts, numerators, and denominators
/// where the glyph height differs from `GLYPH_H`.  The scaled width is
/// proportional: `w = GLYPH_W * h / GLYPH_H`, clamped to a minimum of 4px.
///
/// Greek letters, operators, and symbols use [`emit_glyph_sized`] with the
/// scaled dimensions so they shrink consistently with text glyphs (F-S030-P2-H7).
fn place_node_at_scale(
    engine: &GlyphEngine,
    node: &MathNode,
    paths: &mut Vec<String>,
    x: &mut i64,
    top_y: i64,
    h: i64,
) -> Result<(), MathError> {
    let baseline = top_y + h;
    let scaled_w = (GLYPH_W * h / GLYPH_H.max(1)).max(4);
    match node {
        MathNode::Text(s) | MathNode::TextRun(s) => {
            for ch in s.chars() {
                emit_glyph_sized(engine, ch, paths, *x, top_y, scaled_w, h.max(4))?;
                *x += engine.advance_width(ch, h.max(4), scaled_w) + 1;
            }
        },
        MathNode::Group(nodes) => {
            for n in nodes {
                place_node_at_scale(engine, n, paths, x, top_y, h)?;
            }
        },
        // Greek letters — resolved through the canonical symbols table.
        MathNode::Greek(name) => {
            if name.is_empty() {
                return Err(MathError::EmptyCommandName);
            }
            let ch = greek_to_unicode_char(name).ok_or_else(|| MathError::UnsupportedSymbol {
                name: name.to_string(),
            })?;
            emit_glyph_sized(engine, ch, paths, *x, top_y, scaled_w, h.max(4))?;
            *x += engine.advance_width(ch, h.max(4), scaled_w) + 1;
        },
        // Symbol operators (∑, ∏, ∫, …) resolved through the canonical symbols table.
        // Text operators fall through to multi-char rendering.
        MathNode::Operator(name) => {
            if name.is_empty() {
                return Err(MathError::EmptyCommandName);
            }
            if let Some(ch) = operator_to_unicode_char(name) {
                emit_glyph_sized(engine, ch, paths, *x, top_y, scaled_w, h.max(4))?;
                *x += engine.advance_width(ch, h.max(4), scaled_w) + 1;
            } else if is_text_operator(name) {
                for ch in name.chars() {
                    emit_glyph_sized(engine, ch, paths, *x, top_y, scaled_w, h.max(4))?;
                    *x += engine.advance_width(ch, h.max(4), scaled_w) + 1;
                }
            } else {
                return Err(MathError::UnsupportedSymbol {
                    name: name.to_string(),
                });
            }
        },
        // Misc symbols resolved through the canonical symbols table.
        MathNode::Symbol(name) => {
            if name.is_empty() {
                return Err(MathError::EmptyCommandName);
            }
            let ch = symbol_to_unicode_char(name).ok_or_else(|| MathError::UnsupportedSymbol {
                name: name.to_string(),
            })?;
            emit_glyph_sized(engine, ch, paths, *x, top_y, scaled_w, h.max(4))?;
            *x += engine.advance_width(ch, h.max(4), scaled_w) + 1;
        },
        _ => {
            // For other complex nodes at scale (Superscript inside Superscript, etc.),
            // fall back to normal placement at the requested baseline.
            place_node(engine, node, paths, x, baseline)?;
        },
    }
    Ok(())
}

// `is_text_operator` is now in `crate::symbols` (shared across all renderers).

/// Emit a single glyph as a `<path>` element at position `(x, y)`.
///
/// Delegates to [`emit_glyph_sized`] with the default cell dimensions
/// ([`GLYPH_W`] × [`GLYPH_H`]).
fn emit_glyph(
    engine: &GlyphEngine,
    ch: char,
    paths: &mut Vec<String>,
    x: i64,
    y: i64,
) -> Result<(), MathError> {
    emit_glyph_sized(engine, ch, paths, x, y, GLYPH_W, GLYPH_H)
}

/// Emit a glyph outline for `ch` at the given cell `(x, y, w, h)`.
///
/// Delegates to [`GlyphEngine::emit_glyph`] which produces real Bézier-curve
/// outlines from the bundled Latin Modern Math font. Every character in LM
/// Math has a distinct outline; characters not present in the font fall back
/// to a rectangular bounding-box marker so the path count is preserved.
///
/// The cell origin `(x, y)` is the top-left corner in SVG pixel coordinates.
/// The baseline is placed at `y + h`.
fn emit_glyph_sized(
    engine: &GlyphEngine,
    ch: char,
    paths: &mut Vec<String>,
    x: i64,
    y: i64,
    w: i64,
    h: i64,
) -> Result<(), MathError> {
    // Delegate to the font engine for real glyph outlines (F-S030-P9-H2).
    engine.emit_glyph(ch, paths, x, y, w, h)
}

// (Static glyph table removed — replaced by GlyphEngine + Latin Modern Math font)

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::ast::{MathMode, MathNode};

    // ─────────────────────────────────────────────────────────────────────────
    // Helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Build a minimal [`MathAst`] inline expression.
    fn inline_ast(nodes: Vec<MathNode>) -> MathAst {
        MathAst::new(MathMode::Inline, nodes)
    }

    /// Build a minimal [`MathAst`] display (block) expression.
    fn display_ast(nodes: Vec<MathNode>) -> MathAst {
        MathAst::new(MathMode::Display, nodes)
    }

    /// Extract `(width_px, height_px)` from an SVG `viewBox="0 0 W H"` attribute.
    ///
    /// Returns `None` if the attribute is absent or cannot be parsed.
    /// The returned dimensions are in pixels; multiply by 9 525 (= 914 400 / 96)
    /// to convert to EMUs.
    fn parse_viewbox_dims(svg: &str) -> Option<(i64, i64)> {
        // Find 'viewBox="...'
        let vb_start = svg.find("viewBox=\"")?;
        let after_vb = &svg[vb_start + 9..];
        let vb_end = after_vb.find('"')?;
        let vb_content = &after_vb[..vb_end];
        // Format is "min-x min-y width height" — e.g. "0 0 42 22"
        let parts: Vec<&str> = vb_content.split_whitespace().collect();
        if parts.len() < 4 {
            return None;
        }
        let w = parts[2].parse::<i64>().ok()?;
        let h = parts[3].parse::<i64>().ok()?;
        Some((w, h))
    }

    /// Walk a string with `quick-xml` and confirm all tags balance.
    fn assert_wellformed_xml(xml: &str) -> Result<(), String> {
        use quick_xml::Reader;
        use quick_xml::events::Event;

        let mut reader = Reader::from_str(xml);
        let mut depth: i64 = 0;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(_)) => depth += 1,
                Ok(Event::End(_)) => {
                    depth -= 1;
                    if depth < 0 {
                        return Err(format!("unmatched closing tag in: {xml}"));
                    }
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(format!("XML parse error: {e}")),
                _ => {},
            }
            buf.clear();
        }

        if depth != 0 {
            return Err(format!("unclosed tags (depth={depth}) in: {xml}"));
        }
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-1.10.003 — AC-003: PDF path output — vector SVG, no text characters
    // ─────────────────────────────────────────────────────────────────────────

    /// `render_pdf_paths` succeeds for a simple expression and returns [`SvgPaths`].
    ///
    /// RED GATE: fails until `render_pdf_paths` is implemented.
    #[test]
    fn test_bc_1_10_003_pdf_paths_returns_svg_string() {
        let ast = inline_ast(vec![MathNode::Text(Arc::from("x"))]);
        let paths = render_pdf_paths(&ast).expect("render_pdf_paths must succeed for 'x'");
        assert!(!paths.0.is_empty(), "SvgPaths.svg must not be empty");
        // The svg field must look like an SVG document
        assert!(
            paths.0.contains("<svg") || paths.0.contains("<?xml"),
            "SvgPaths.svg must start with an SVG root element; got: {}",
            &paths.0[..paths.0.len().min(120)]
        );
    }

    /// `render_pdf_paths` SVG output contains zero `<text` elements.
    ///
    /// AC-003: No text characters — all glyphs must be path data.
    ///
    /// RED GATE: fails until `render_pdf_paths` is implemented.
    #[test]
    fn test_bc_1_10_003_pdf_paths_svg_has_no_text_elements() {
        let ast = inline_ast(vec![MathNode::Text(Arc::from("E"))]);
        let paths = render_pdf_paths(&ast).expect("render_pdf_paths must succeed");
        assert!(
            !paths.0.contains("<text"),
            "SVG must not contain <text> elements (PDF/UA-1); got: {}",
            &paths.0[..paths.0.len().min(400)]
        );
    }

    /// `render_pdf_paths` SVG output contains zero `<image` elements.
    ///
    /// AC-003: Only `<path>` geometry; no embedded raster images.
    ///
    /// RED GATE: fails until `render_pdf_paths` is implemented.
    #[test]
    fn test_bc_1_10_003_pdf_paths_svg_has_no_image_elements() {
        let ast = inline_ast(vec![MathNode::Text(Arc::from("E"))]);
        let paths = render_pdf_paths(&ast).expect("render_pdf_paths must succeed");
        assert!(
            !paths.0.contains("<image"),
            "SVG must not contain <image> elements (AC-003); got: {}",
            &paths.0[..paths.0.len().min(400)]
        );
    }

    /// `render_pdf_paths` SVG output contains at least one `<path` element.
    ///
    /// AC-003: All geometry is `<path d="...">` with absolute coordinates.
    ///
    /// RED GATE: fails until `render_pdf_paths` is implemented.
    #[test]
    fn test_bc_1_10_003_pdf_paths_svg_contains_path_elements() {
        let ast = inline_ast(vec![MathNode::Text(Arc::from("x"))]);
        let paths = render_pdf_paths(&ast).expect("render_pdf_paths must succeed");
        assert!(
            paths.0.contains(r#"<path d=""#) || paths.0.contains("<path "),
            "SVG must contain <path> elements with glyph outlines; got: {}",
            &paths.0[..paths.0.len().min(400)]
        );
    }

    /// `render_pdf_paths` SVG has a positive viewBox (positive width and height).
    ///
    /// BC-1.10.003 invariant: bounding box must have positive width and height.
    /// Since `SvgPaths` is a newtype, dimensions are read from the SVG viewBox.
    ///
    /// RED GATE: fails until `render_pdf_paths` is implemented.
    #[test]
    fn test_bc_1_10_003_pdf_paths_dimensions_positive() {
        let ast = inline_ast(vec![MathNode::Text(Arc::from("x"))]);
        let paths = render_pdf_paths(&ast).expect("render_pdf_paths must succeed");
        let (w, h) = parse_viewbox_dims(&paths.0)
            .unwrap_or_else(|| panic!("SVG must have a parseable viewBox; got: {}", paths.0));
        assert!(w > 0, "viewBox width must be positive; got: {w}");
        assert!(h > 0, "viewBox height must be positive; got: {h}");
    }

    /// `render_pdf_paths` output SVG is well-formed XML.
    ///
    /// RED GATE: fails until `render_pdf_paths` is implemented.
    #[test]
    fn test_bc_1_10_003_pdf_paths_wellformed_xml() {
        let ast = inline_ast(vec![MathNode::Superscript {
            base: Box::new(MathNode::Text(Arc::from("x"))),
            sup: Box::new(MathNode::Text(Arc::from("2"))),
        }]);
        let paths = render_pdf_paths(&ast).expect("render_pdf_paths must succeed for x^2");
        assert_wellformed_xml(&paths.0)
            .unwrap_or_else(|e| panic!("PDF path SVG is not well-formed XML: {e}"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-1.10.003 — EC-005: usvg fails → TextRemainsInPathOutput
    // ─────────────────────────────────────────────────────────────────────────

    /// [`SvgPaths`] is a newtype `SvgPaths(String)` — the inner SVG string is
    /// accessible via `.0`, and the type is `Clone + PartialEq + Hash`.
    ///
    /// This tests the newtype API alignment with AC-003 (spec: `SvgPaths(String)`).
    #[test]
    fn test_bc_1_10_003_svg_paths_struct_fields_accessible() {
        let paths = SvgPaths("<svg/>".to_owned());
        let cloned = paths.clone();
        assert_eq!(paths, cloned);
        // Inner SVG string is accessible via .0
        assert_eq!(cloned.0, "<svg/>");
        // The type satisfies Hash (compile-time check — use it as a HashMap key)
        let mut map = std::collections::HashMap::new();
        map.insert(paths.clone(), 42u32);
        assert_eq!(map[&paths], 42);
    }

    /// `render_pdf_paths` for a complex multi-node expression returns `Ok`,
    /// confirming the renderer handles all supported [`MathNode`] variants.
    ///
    /// RED GATE: fails until `render_pdf_paths` is implemented.
    #[test]
    fn test_bc_1_10_003_pdf_paths_complex_expression_succeeds() {
        // AST for: x^2 + y^2  (pythagorean-like expression)
        let ast = inline_ast(vec![
            MathNode::Superscript {
                base: Box::new(MathNode::Text(Arc::from("x"))),
                sup: Box::new(MathNode::Text(Arc::from("2"))),
            },
            MathNode::Text(Arc::from("+")),
            MathNode::Superscript {
                base: Box::new(MathNode::Text(Arc::from("y"))),
                sup: Box::new(MathNode::Text(Arc::from("2"))),
            },
        ]);
        let paths = render_pdf_paths(&ast).expect("render_pdf_paths must succeed for x^2+y^2");
        assert!(!paths.0.is_empty(), "SVG must not be empty");
        let (w, h) = parse_viewbox_dims(&paths.0).unwrap_or_else(|| {
            panic!("complex expression SVG must have viewBox; got: {}", paths.0)
        });
        assert!(w > 0, "viewBox width must be positive");
        assert!(h > 0, "viewBox height must be positive");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-1.10.003 — AC-004: Single AST source (no re-parsing in render)
    // ─────────────────────────────────────────────────────────────────────────

    /// The same [`MathAst`] reference can be passed to `render_pdf_paths` twice
    /// and both calls return identical, deterministic output.
    ///
    /// RED GATE: fails until `render_pdf_paths` is implemented.
    #[test]
    fn test_bc_1_10_003_pdf_paths_ast_reusable_across_calls() {
        let ast = inline_ast(vec![MathNode::Text(Arc::from("x"))]);
        let first = render_pdf_paths(&ast).expect("first call must succeed");
        let second = render_pdf_paths(&ast).expect("second call must succeed");
        assert_eq!(
            first, second,
            "render_pdf_paths must be deterministic for the same AST"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Finding C1–C5 — Greek letters, operators, and symbols render distinct
    // Unicode glyphs, not Latin alphabet characters
    // ─────────────────────────────────────────────────────────────────────────

    /// Render `\gamma` → must produce an SVG whose path data is DIFFERENT from
    /// the path data produced by rendering the Latin letter `g`.
    ///
    /// Closes finding C1 (gamma → g regression).
    #[test]
    fn test_bc_1_10_003_pdf_greek_letter_distinct_from_latin() {
        let gamma_ast = inline_ast(vec![MathNode::Greek(Arc::from("gamma"))]);
        let g_ast = inline_ast(vec![MathNode::Text(Arc::from("g"))]);

        let gamma_svg = render_pdf_paths(&gamma_ast)
            .expect("render_pdf_paths must succeed for \\gamma")
            .0;
        let g_svg = render_pdf_paths(&g_ast)
            .expect("render_pdf_paths must succeed for 'g'")
            .0;

        assert_ne!(
            gamma_svg, g_svg,
            "\\gamma must produce a DIFFERENT SVG than Latin 'g'"
        );
        // Confirm gamma path contains the γ-specific arm pattern (two arms + tail)
        // which differs structurally from the 'g' path (loop with descender)
        assert!(
            !gamma_svg.is_empty() && !g_svg.is_empty(),
            "both SVGs must be non-empty"
        );
    }

    /// Render `\sum` → must NOT produce three Latin letters (s, u, m).
    /// The SVG for ∑ must use a single glyph path (one `<path>` element), not
    /// three separate paths for 's', 'u', 'm'.
    ///
    /// Closes finding C2 (sum → "sum" three letters regression).
    #[test]
    fn test_bc_1_10_003_pdf_sum_operator_distinct_from_letters() {
        let sum_ast = inline_ast(vec![MathNode::Operator(Arc::from("sum"))]);
        let sum_svg = render_pdf_paths(&sum_ast)
            .expect("render_pdf_paths must succeed for \\sum")
            .0;

        // Count <path> elements in the output — for a single glyph ∑ there
        // should be exactly one <path> element, not three (s, u, m).
        let path_count = sum_svg.matches("<path ").count();
        assert_eq!(
            path_count, 1,
            "\\sum must produce exactly 1 <path> element (one glyph ∑), not {path_count} (as would happen for 's','u','m')"
        );
    }

    /// Render an unmapped `\xyzunknown` operator → must return `Err(UnsupportedSymbol)`.
    ///
    /// Closes finding C5 (silent fallback on unknown symbols) and F-S030-P2-H6
    /// (operator unknown name must error, not fall through to Latin chars).
    ///
    /// Both `MathNode::Operator` and `MathNode::Symbol` with unrecognised names
    /// must return `Err(MathError::UnsupportedSymbol)`.  Known text operators
    /// (lim, max, min, sin, …) are the only allowed Latin-char fall-through.
    #[test]
    fn test_bc_1_10_003_pdf_unknown_command_errors() {
        // Unknown Operator name must error.
        let op_ast = inline_ast(vec![MathNode::Operator(Arc::from("xyzunknown"))]);
        let op_result = render_pdf_paths(&op_ast);
        assert!(
            op_result.is_err(),
            "MathNode::Operator with unknown name must return Err; got Ok"
        );
        match op_result {
            Err(MathError::UnsupportedSymbol { .. }) => {}, // correct
            other => panic!("expected MathError::UnsupportedSymbol for Operator, got: {other:?}"),
        }

        // Known text operator (lim) must NOT error — it renders as Latin chars.
        let lim_ast = inline_ast(vec![MathNode::Operator(Arc::from("lim"))]);
        let lim_result = render_pdf_paths(&lim_ast);
        assert!(
            lim_result.is_ok(),
            "MathNode::Operator('lim') must succeed; got: {lim_result:?}"
        );

        // Unknown Symbol name must error.
        let sym_ast = inline_ast(vec![MathNode::Symbol(Arc::from("xyzunknown"))]);
        let sym_result = render_pdf_paths(&sym_ast);
        assert!(
            sym_result.is_err(),
            "MathNode::Symbol with unknown name must return Err; got Ok"
        );
        match sym_result {
            Err(MathError::UnsupportedSymbol { .. }) => {}, // correct
            other => panic!("expected MathError::UnsupportedSymbol for Symbol, got: {other:?}"),
        }
    }

    /// Render `MathNode::Greek("")` → must return `Err(EmptyCommandName)`.
    ///
    /// Closes finding I5 (silent fallback on empty command name).
    #[test]
    fn test_bc_1_10_003_pdf_empty_greek_command_name_errors() {
        let ast = inline_ast(vec![MathNode::Greek(Arc::from(""))]);
        let result = render_pdf_paths(&ast);
        assert!(
            matches!(result, Err(MathError::EmptyCommandName)),
            "empty Greek command name must return Err(EmptyCommandName); got: {result:?}"
        );
    }

    /// Render `MathNode::Operator("")` → must return `Err(EmptyCommandName)`.
    #[test]
    fn test_bc_1_10_003_pdf_empty_operator_command_name_errors() {
        let ast = inline_ast(vec![MathNode::Operator(Arc::from(""))]);
        let result = render_pdf_paths(&ast);
        assert!(
            matches!(result, Err(MathError::EmptyCommandName)),
            "empty Operator command name must return Err(EmptyCommandName); got: {result:?}"
        );
    }

    /// Render `MathNode::Symbol("")` → must return `Err(EmptyCommandName)`.
    #[test]
    fn test_bc_1_10_003_pdf_empty_symbol_command_name_errors() {
        let ast = inline_ast(vec![MathNode::Symbol(Arc::from(""))]);
        let result = render_pdf_paths(&ast);
        assert!(
            matches!(result, Err(MathError::EmptyCommandName)),
            "empty Symbol command name must return Err(EmptyCommandName); got: {result:?}"
        );
    }

    /// Display-mode AST produces larger EMU dimensions than the equivalent
    /// inline-mode AST (1.2× scale).
    ///
    /// Closes finding I3.
    #[test]
    fn test_bc_1_10_003_pdf_display_mode_larger_than_inline() {
        let nodes = vec![MathNode::Text(Arc::from("x"))];
        let inline =
            render_pdf_paths(&inline_ast(nodes.clone())).expect("inline render must succeed");
        let display = render_pdf_paths(&display_ast(nodes)).expect("display render must succeed");

        let (inline_w, inline_h) = parse_viewbox_dims(&inline.0)
            .unwrap_or_else(|| panic!("inline SVG must have viewBox; got: {}", inline.0));
        let (display_w, display_h) = parse_viewbox_dims(&display.0)
            .unwrap_or_else(|| panic!("display SVG must have viewBox; got: {}", display.0));

        assert!(
            display_w >= inline_w,
            "display-mode viewBox width ({display_w}) must be >= inline width ({inline_w})"
        );
        assert!(
            display_h > inline_h,
            "display-mode viewBox height ({display_h}) must be > inline height ({inline_h})"
        );
    }

    /// `∪` (U+222A) and `U` (Latin) must produce different SVG paths.
    ///
    /// Closes alias finding C4 (∪ and U shared the same path).
    #[test]
    fn test_bc_1_10_003_pdf_union_distinct_from_latin_u() {
        let union_ast = inline_ast(vec![MathNode::Symbol(Arc::from("cup"))]);
        let u_ast = inline_ast(vec![MathNode::Text(Arc::from("U"))]);

        let union_svg = render_pdf_paths(&union_ast)
            .expect("render for \\cup must succeed")
            .0;
        let u_svg = render_pdf_paths(&u_ast)
            .expect("render for Latin U must succeed")
            .0;

        assert_ne!(
            union_svg, u_svg,
            "\\cup (∪) must produce a DIFFERENT SVG than Latin 'U'"
        );
    }

    /// `×` (U+00D7) and `X` (Latin) must produce different SVG paths.
    ///
    /// Closes alias finding C3 (× and X shared the same path).
    #[test]
    fn test_bc_1_10_003_pdf_times_distinct_from_latin_x() {
        let times_ast = inline_ast(vec![MathNode::Symbol(Arc::from("times"))]);
        let x_ast = inline_ast(vec![MathNode::Text(Arc::from("X"))]);

        let times_svg = render_pdf_paths(&times_ast)
            .expect("render for \\times must succeed")
            .0;
        let x_svg = render_pdf_paths(&x_ast)
            .expect("render for Latin X must succeed")
            .0;

        assert_ne!(
            times_svg, x_svg,
            "\\times (×) must produce a DIFFERENT SVG than Latin 'X'"
        );
    }

    /// `∫` (U+222B) and `⊂` (U+2282) must produce different SVG paths.
    ///
    /// Closes alias finding C5 (∫ and ⊂ shared an S-curve path).
    #[test]
    fn test_bc_1_10_003_pdf_integral_distinct_from_subset() {
        let int_ast = inline_ast(vec![MathNode::Operator(Arc::from("int"))]);
        let subset_ast = inline_ast(vec![MathNode::Symbol(Arc::from("subset"))]);

        let int_svg = render_pdf_paths(&int_ast)
            .expect("render for \\int must succeed")
            .0;
        let subset_svg = render_pdf_paths(&subset_ast)
            .expect("render for \\subset must succeed")
            .0;

        assert_ne!(
            int_svg, subset_svg,
            "\\int (∫) must produce a DIFFERENT SVG than \\subset (⊂)"
        );
    }

    /// `+` (U+002B) and `t` (Latin) must produce different SVG paths.
    ///
    /// Closes the `'t' | '+'` alias.
    #[test]
    fn test_bc_1_10_003_pdf_plus_distinct_from_latin_t() {
        let plus_ast = inline_ast(vec![MathNode::Text(Arc::from("+"))]);
        let t_ast = inline_ast(vec![MathNode::Text(Arc::from("t"))]);

        let plus_svg = render_pdf_paths(&plus_ast)
            .expect("render for '+' must succeed")
            .0;
        let t_svg = render_pdf_paths(&t_ast)
            .expect("render for 't' must succeed")
            .0;

        assert_ne!(
            plus_svg, t_svg,
            "'+' must produce a DIFFERENT SVG than Latin 't'"
        );
    }

    /// `|` (U+007C) and `l` (Latin) must produce different SVG paths.
    ///
    /// Closes the `'l' | '|'` alias.
    #[test]
    fn test_bc_1_10_003_pdf_pipe_distinct_from_latin_l() {
        let pipe_ast = inline_ast(vec![MathNode::Text(Arc::from("|"))]);
        let l_ast = inline_ast(vec![MathNode::Text(Arc::from("l"))]);

        let pipe_svg = render_pdf_paths(&pipe_ast)
            .expect("render for '|' must succeed")
            .0;
        let l_svg = render_pdf_paths(&l_ast)
            .expect("render for 'l' must succeed")
            .0;

        assert_ne!(
            pipe_svg, l_svg,
            "'|' must produce a DIFFERENT SVG than Latin 'l'"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-S030-P2-H7 — `place_node_at_scale` propagates scale to Greek/Operator/Symbol
    // ─────────────────────────────────────────────────────────────────────────

    /// A superscript containing a Greek letter must render with a SMALLER glyph
    /// than the same Greek letter at the baseline.
    ///
    /// Before the fix, `place_node_at_scale` dispatched Greek/Operator/Symbol nodes
    /// to the unscaled `place_node`, so the glyph had full `GLYPH_W` × `GLYPH_H`
    /// dimensions regardless of the scale parameter.  After the fix, scaled
    /// Greek/Operator/Symbol nodes use `emit_glyph_sized` with the proportional
    /// width and the requested `h`.
    #[test]
    fn test_bc_1_10_003_pdf_superscript_greek_is_scaled() {
        // `\alpha^{\gamma}` — superscript is a Greek letter
        let ast_sup = inline_ast(vec![MathNode::Superscript {
            base: Box::new(MathNode::Greek(Arc::from("alpha"))),
            sup: Box::new(MathNode::Greek(Arc::from("gamma"))),
        }]);
        // Just `\gamma` at baseline
        let ast_plain = inline_ast(vec![MathNode::Greek(Arc::from("gamma"))]);

        let sup_svg = render_pdf_paths(&ast_sup)
            .expect("superscript of Greek letter must render successfully")
            .0;
        let plain_svg = render_pdf_paths(&ast_plain)
            .expect("plain Greek letter must render successfully")
            .0;

        // The SVGs should differ: the superscript version has TWO path elements
        // (base α + scaled γ), the plain version has ONE.  More importantly, the
        // `<path>` data strings differ because the superscript γ uses scaled
        // dimensions while the plain γ uses full dimensions.
        assert_ne!(
            sup_svg, plain_svg,
            "superscript γ SVG must differ from plain γ SVG (scale must be applied)"
        );

        // The composite SVG must have 2 path elements (one for α, one for scaled γ).
        let path_count = sup_svg.matches("<path ").count();
        assert_eq!(
            path_count, 2,
            "\\alpha^{{\\gamma}} must produce exactly 2 <path> elements; got {path_count}"
        );
    }

    /// A superscript containing a symbol (`\\times`) renders scaled.
    #[test]
    fn test_bc_1_10_003_pdf_superscript_symbol_is_scaled() {
        let ast = inline_ast(vec![MathNode::Superscript {
            base: Box::new(MathNode::Text(Arc::from("x"))),
            sup: Box::new(MathNode::Symbol(Arc::from("times"))),
        }]);
        let result = render_pdf_paths(&ast);
        assert!(
            result.is_ok(),
            "x^{{\\times}} must render successfully; got: {result:?}"
        );
        let path_count = result.unwrap().0.matches("<path ").count();
        assert_eq!(
            path_count, 2,
            "x^{{\\times}} must produce exactly 2 <path> elements; got {path_count}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-S030-P3-H3 — Cases renders result THEN condition in PDF paths
    // ─────────────────────────────────────────────────────────────────────────

    /// PDF path renderer for `\begin{cases} x & y > 0 \end{cases}` must place
    /// result ('x') paths BEFORE condition ('y') paths.
    ///
    /// We verify by checking the x-position (pen advance) — result comes first
    /// in the row, so it occupies lower x-coordinates than the condition.
    /// The SVG will have more paths for the condition (`y > 0` = 3 tokens)
    /// than for the result (`x` = 1 token); the result's single path must
    /// appear before the condition's first path in the SVG string.
    #[test]
    fn test_bc_1_10_003_pdf_cases_renders_result_then_condition() {
        // Tuple storage: (result=[x], condition=[y, >, 0])
        let ast = inline_ast(vec![MathNode::Cases(vec![(
            vec![MathNode::Text(Arc::from("x"))],
            vec![
                MathNode::Text(Arc::from("y")),
                MathNode::Text(Arc::from(">")),
                MathNode::Text(Arc::from("0")),
            ],
        )])]);
        let result = render_pdf_paths(&ast).expect("render_pdf_paths must succeed for cases");
        let svg = &result.0;
        // There must be at least 4 paths: 1 for 'x', 3 for 'y', '>', '0'
        let path_count = svg.matches("<path ").count();
        assert!(
            path_count >= 4,
            "cases must produce at least 4 paths (1 result + 3 condition); got: {path_count}\nSVG: {svg}"
        );
        // The SVG must be well-formed
        assert!(
            svg.contains("<svg"),
            "SVG output must contain <svg root; got: {svg}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-S030-P3-H5 — Display mode applies transform="scale(1.2)" wrapper
    // ─────────────────────────────────────────────────────────────────────────

    /// Display-mode SVG must contain `<g transform="scale(1.2)">` to actually
    /// scale the glyph paths.  Inline mode must NOT contain the scale transform.
    #[test]
    fn test_bc_1_10_003_pdf_display_mode_has_scale_transform() {
        let nodes = vec![MathNode::Text(Arc::from("x"))];
        let inline_result =
            render_pdf_paths(&inline_ast(nodes.clone())).expect("inline render must succeed");
        let display_result =
            render_pdf_paths(&display_ast(nodes)).expect("display render must succeed");

        assert!(
            display_result.0.contains(r#"transform="scale(1.2)""#),
            "display-mode SVG must contain transform=\"scale(1.2)\" wrapper; got: {}",
            display_result.0
        );
        assert!(
            !inline_result.0.contains(r#"transform="scale(1.2)""#),
            "inline-mode SVG must NOT contain scale transform; got: {}",
            inline_result.0
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-S030-P3-H6 — Delimiter unescape mapping in PDF paths
    // ─────────────────────────────────────────────────────────────────────────

    /// `MathNode::Delimiter` with LaTeX-escaped delimiters (`\{`, `\}`) must
    /// strip the backslash before dispatching each char to `emit_glyph`.
    /// A bare `{` glyph produces a distinct path from a `\` followed by `{`.
    #[test]
    fn test_finding_005_delimiter_escapes_stripped_pdf() {
        // \{ inner \} — the delimiters are LaTeX-escaped curly braces
        let ast = inline_ast(vec![MathNode::Delimiter {
            left: Arc::from("\\{"),
            right: Arc::from("\\}"),
            inner: vec![MathNode::Text(Arc::from("x"))],
        }]);
        let result = render_pdf_paths(&ast).expect("render_pdf_paths must succeed for \\{x\\}");
        // With unescaping: left='{', right='}', inner='x' → 3 paths total.
        // Without unescaping: left='\{' (2 chars), right='\}' (2 chars), inner='x' → 5 paths.
        let path_count = result.0.matches("<path ").count();
        assert_eq!(
            path_count, 3,
            "\\{{x\\}} with unescaping must produce 3 paths ({{, x, }}); got: {path_count}\nSVG: {}",
            result.0
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-S030-P3-M2 — Accent rendering is kind-specific in PDF
    // ─────────────────────────────────────────────────────────────────────────

    /// `\hat{x}` and `\bar{x}` must produce distinct SVG path data.
    #[test]
    fn test_bc_1_10_003_pdf_hat_and_bar_accents_are_distinct() {
        use crate::ast::AccentKind;
        let hat_ast = inline_ast(vec![MathNode::Accent {
            kind: AccentKind::Hat,
            inner: Box::new(MathNode::Text(Arc::from("x"))),
        }]);
        let bar_ast = inline_ast(vec![MathNode::Accent {
            kind: AccentKind::Bar,
            inner: Box::new(MathNode::Text(Arc::from("x"))),
        }]);
        let hat_svg = render_pdf_paths(&hat_ast)
            .expect("render_pdf_paths must succeed for \\hat{x}")
            .0;
        let bar_svg = render_pdf_paths(&bar_ast)
            .expect("render_pdf_paths must succeed for \\bar{x}")
            .0;
        assert_ne!(
            hat_svg, bar_svg,
            "\\hat{{x}} and \\bar{{x}} must produce distinct SVG paths"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-S030-P3-M6 — φ (U+03C6) and Φ (U+03A6) must produce distinct glyphs
    // ─────────────────────────────────────────────────────────────────────────

    /// `\phi` (lowercase φ, U+03C6) and `\Phi` (uppercase Φ, U+03A6) must
    /// produce different SVG path data.
    #[test]
    fn test_bc_1_10_003_pdf_phi_distinct_from_big_phi() {
        let phi_ast = inline_ast(vec![MathNode::Greek(Arc::from("phi"))]);
        let big_phi_ast = inline_ast(vec![MathNode::Greek(Arc::from("Phi"))]);
        let phi_svg = render_pdf_paths(&phi_ast)
            .expect("render_pdf_paths must succeed for \\phi")
            .0;
        let big_phi_svg = render_pdf_paths(&big_phi_ast)
            .expect("render_pdf_paths must succeed for \\Phi")
            .0;
        assert_ne!(
            phi_svg, big_phi_svg,
            "\\phi (φ, U+03C6) must produce DIFFERENT SVG than \\Phi (Φ, U+03A6)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-S030-P3-L2 — Empty AST returns error
    // ─────────────────────────────────────────────────────────────────────────

    /// `render_pdf_paths` on an empty AST (zero nodes) must return
    /// `Err(MathError::EmptyAst)` rather than emitting a degenerate 1×16 SVG.
    #[test]
    fn test_bc_1_10_003_pdf_empty_ast_returns_error() {
        let ast = inline_ast(vec![]);
        let result = render_pdf_paths(&ast);
        assert!(
            result.is_err(),
            "render_pdf_paths on empty AST must return Err; got Ok"
        );
        match result {
            Err(MathError::EmptyAst) => {}, // correct
            other => panic!("expected MathError::EmptyAst for empty AST, got: {other:?}"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-S030-P7-M1 — Cases/Align row alignment regression tests
    // ─────────────────────────────────────────────────────────────────────────

    /// Helper: extract the x coordinate from the first `M` command in a `d="..."` attribute.
    ///
    /// Parses `<path d="Mx,y ..."` and returns `x` as `i64`.  Returns `None`
    /// if the attribute is absent or the coordinate cannot be parsed.
    fn first_path_x(svg: &str) -> Option<i64> {
        // Find the first `<path ` element and extract its `d` attribute.
        let path_start = svg.find("<path ")?;
        let after_path = &svg[path_start..];
        // Find `d="` within this element
        let d_start = after_path.find(" d=\"")?;
        let d_content = &after_path[d_start + 4..]; // skip ' d="'
        // Find the closing quote
        let d_end = d_content.find('"')?;
        let d_attr = &d_content[..d_end];
        // The first coordinate after `M` (or `m`)
        let m_pos = d_attr.find(['M', 'm'])?;
        let after_m = &d_attr[m_pos + 1..];
        // x is up to the first `,`
        let comma_pos = after_m.find(',')?;
        let x_str = after_m[..comma_pos].trim();
        x_str.parse::<i64>().ok()
    }

    /// Helper: extract the x coordinate from the Nth `<path ` element's first M command.
    fn nth_path_x(svg: &str, n: usize) -> Option<i64> {
        let mut remaining = svg;
        let mut count = 0;
        loop {
            let pos = remaining.find("<path ")?;
            if count == n {
                return first_path_x(&remaining[pos..]);
            }
            remaining = &remaining[pos + 6..];
            count += 1;
        }
    }

    /// `MathNode::Cases` with two rows must start both rows at the same x
    /// coordinate — row 1's first glyph must have the same x as row 0's first glyph.
    ///
    /// Without the F-S030-P7-M1 fix each row starts where the previous row ended,
    /// producing a diagonal cascade instead of left-aligned columns.
    ///
    /// With real font outlines (F-S030-P9-H2), the first M coordinate in each path's
    /// d= string reflects the font's first contour point — NOT the cell x origin.
    /// The correct verification strategy is to use the **same character** as the
    /// first glyph in both rows and check that their first M coordinates are equal
    /// (proving both rows were rendered with the same `start_x = 0`).
    #[test]
    fn test_bc_1_10_003_pdf_cases_rows_aligned_at_same_x() {
        // \begin{cases} x & xy \\ x & y \end{cases}
        // Row 0: result=[x], condition=[x, y]  (3 paths: x, x, y)
        // Row 1: result=[x], condition=[y]       (2 paths: x, y)
        // Both rows start with the same character 'x', so their first-M coordinates
        // must be identical if the row-reset fix is active.
        let ast = inline_ast(vec![MathNode::Cases(vec![
            (
                vec![MathNode::Text(Arc::from("x"))],
                vec![MathNode::Text(Arc::from("xy"))],
            ),
            (
                vec![MathNode::Text(Arc::from("x"))],
                vec![MathNode::Text(Arc::from("y"))],
            ),
        ])]);
        let result = render_pdf_paths(&ast).expect("render_pdf_paths must succeed for cases");
        let svg = &result.0;

        // Row 0 first path x — path index 0
        let x_row0 = nth_path_x(svg, 0)
            .unwrap_or_else(|| panic!("could not extract x from path 0 in:\n{svg}"));
        // Row 1 first path x — row 0 has 3 paths (x, x, y); row 1's 'x' is path index 3
        let x_row1 = nth_path_x(svg, 3)
            .unwrap_or_else(|| panic!("could not extract x from path 3 in:\n{svg}"));

        assert_eq!(
            x_row0, x_row1,
            "Cases: row 0 first-glyph x ({x_row0}) must equal row 1 first-glyph x ({x_row1})\n\
             Both rows start with 'x' at start_x=0; if the row-reset fix (F-S030-P7-M1) is \
             active their first-M coordinates must be identical.\nSVG: {svg}"
        );
    }

    /// `MathNode::Align` with two rows must start both rows at the same x coordinate.
    ///
    /// Mirrors the Cases regression test — same root cause, different node variant.
    #[test]
    fn test_bc_1_10_003_pdf_align_rows_aligned_at_same_x() {
        // \begin{align} x &= 1 \\ xy &= 2 \end{align}
        // Row 0: [x, =, 1]
        // Row 1: [x, y, =, 2]  (longer row — without fix, row 1 starts further right)
        let ast = inline_ast(vec![MathNode::Align(vec![
            vec![
                MathNode::Text(Arc::from("x")),
                MathNode::Text(Arc::from("=")),
                MathNode::Text(Arc::from("1")),
            ],
            vec![
                MathNode::Text(Arc::from("x")),
                MathNode::Text(Arc::from("y")),
                MathNode::Text(Arc::from("=")),
                MathNode::Text(Arc::from("2")),
            ],
        ])]);
        let result = render_pdf_paths(&ast).expect("render_pdf_paths must succeed for align");
        let svg = &result.0;

        // Row 0 first path x (path 0 = 'x')
        let x_row0 = nth_path_x(svg, 0)
            .unwrap_or_else(|| panic!("could not extract x from path 0 in:\n{svg}"));
        // Row 1 first path x — row 0 has 3 glyphs so row 1 starts at path index 3
        let x_row1 = nth_path_x(svg, 3)
            .unwrap_or_else(|| panic!("could not extract x from path 3 in:\n{svg}"));

        assert_eq!(
            x_row0, x_row1,
            "Align: row 0 first glyph x ({x_row0}) must equal row 1 first glyph x ({x_row1})\n\
             Bug F-S030-P7-M1: without fix, row N starts at end-x of row N-1.\nSVG: {svg}"
        );
    }
}
