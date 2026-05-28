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
//! simple rectangles that symbolise character bounding boxes; this provides
//! correct structure (vector paths, no text) while keeping the implementation
//! pure-Rust with zero I/O. The glyph geometry is intentionally stylised
//! rather than font-accurate — production-quality rendering is scheduled for
//! STORY-045 when a full font pipeline is attached.
//!
//! ## Constraints
//!
//! - **Pure function**: no I/O, no spawned processes (Architecture Rule 1).
//! - **No `<text>` elements** in the output (PDF/UA-1, DI-014).
//! - No `<image>` elements; all geometry must be `<path d="...">` (AC-003).
//! - Integer EMUs for bounding-box dimensions (`i64`, 914 400 per inch).

use slideforge_plugin_api::MathError;

use crate::MathAst;
use crate::ast::MathNode;

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
/// been expanded to `<path d="...">` outlines. If the post-generation
/// verification detects any `<text>` element in the produced SVG (which
/// should never happen with the static glyph engine), returns
/// `Err(MathError::TextRemainsInPathOutput)`.
///
/// # Errors
///
/// - [`MathError::TextRemainsInPathOutput`] — a `<text>` element was found in
///   the generated SVG (internal invariant violation, EC-005).
/// - [`MathError::RenderError`] — internal rendering failure.
pub fn render_pdf_paths(ast: &MathAst) -> Result<SvgPaths, MathError> {
    let mut paths: Vec<String> = Vec::new();
    let mut x: i64 = 0;
    let baseline_y: i64 = GLYPH_H;

    collect_paths(ast, &ast.nodes, &mut paths, &mut x, baseline_y);

    let width_px = x.max(1);
    let height_px = (GLYPH_H + SUP_OFFSET + 2).max(1);

    let width_emu = width_px * EMU_PER_PX;
    let height_emu = height_px * EMU_PER_PX;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width_px} {height_px}" width="{width_px}" height="{height_px}">"#,
    );

    for path in &paths {
        svg.push_str(path);
    }

    svg.push_str("</svg>");

    // Invariant verification: the static glyph engine must not produce <text>
    debug_assert!(
        !svg.contains("<text"),
        "BUG: render_pdf_paths produced <text> element — static glyph engine invariant violated"
    );

    if svg.contains("<text") {
        return Err(MathError::TextRemainsInPathOutput);
    }

    Ok(SvgPaths {
        svg,
        width_emu,
        height_emu,
    })
}

/// Collect `<path>` elements for an expression, advancing `x` as glyphs are
/// placed. This function is called on the flat node list of the AST and
/// delegates to [`place_node`] for each node.
fn collect_paths(
    ast: &MathAst,
    nodes: &[MathNode],
    paths: &mut Vec<String>,
    x: &mut i64,
    baseline_y: i64,
) {
    let _ = ast; // mode available for future layout decisions
    for node in nodes {
        place_node(node, paths, x, baseline_y);
    }
}

/// Recursively place a [`MathNode`] as one or more `<path>` elements.
///
/// `x` is the current pen position (left edge of the next glyph).
/// `baseline_y` is the vertical coordinate of the text baseline.
fn place_node(node: &MathNode, paths: &mut Vec<String>, x: &mut i64, baseline_y: i64) {
    match node {
        MathNode::Text(s) | MathNode::TextRun(s) => {
            for ch in s.chars() {
                emit_glyph(ch, paths, *x, baseline_y - GLYPH_H);
                *x += GLYPH_W + 1;
            }
        },

        MathNode::Superscript { base, sup } => {
            // Render base at normal position
            place_node(base, paths, x, baseline_y);
            // Render sup raised above the baseline
            let sup_y = (baseline_y - GLYPH_H - SUP_OFFSET).max(0);
            let sup_h = GLYPH_H * 2 / 3;
            let sup_start = *x;
            place_node_at_scale(sup, paths, x, sup_y, sup_h);
            let _ = sup_start; // used for layout tracking only
        },

        MathNode::Subscript { base, sub } => {
            place_node(base, paths, x, baseline_y);
            let sub_y = baseline_y + SUB_OFFSET;
            let sub_h = GLYPH_H * 2 / 3;
            place_node_at_scale(sub, paths, x, sub_y, sub_h);
        },

        MathNode::Fraction { num, denom } => {
            // Render numerator above the fraction bar, denominator below
            let saved_x = *x;
            let mut num_x = *x;
            let mut denom_x = *x;

            // Numerator above baseline
            let num_y = (baseline_y - GLYPH_H * 3 / 2).max(0);
            place_node_at_scale(num, paths, &mut num_x, num_y, GLYPH_H * 2 / 3);

            // Denominator below baseline
            let denom_y = baseline_y;
            place_node_at_scale(denom, paths, &mut denom_x, denom_y, GLYPH_H * 2 / 3);

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
                place_node_at_scale(idx, paths, x, rad_y, GLYPH_H / 2);
            }

            let content_start = *x;
            place_node(radicand, paths, x, baseline_y);
            let content_end = *x;

            // Overline above radicand
            paths.push(format!(
                r#"<path d="M{content_start},{rad_y} H{content_end}" stroke="black" stroke-width="1" fill="none"/>"#,
            ));
            *x += 2;
        },

        MathNode::Operator(name) | MathNode::Symbol(name) => {
            // Render each character of the symbol name as a glyph
            for ch in name.chars() {
                emit_glyph(ch, paths, *x, baseline_y - GLYPH_H);
                *x += GLYPH_W + 1;
            }
        },

        MathNode::Greek(name) => {
            // Emit a placeholder glyph for the Greek letter
            let ch = name.chars().next().unwrap_or('?');
            emit_glyph(ch, paths, *x, baseline_y - GLYPH_H);
            *x += GLYPH_W + 1;
        },

        MathNode::Accent { kind: _, inner } => {
            let save_x = *x;
            place_node(inner, paths, x, baseline_y);
            // Draw accent mark above the inner node
            let accent_y = (baseline_y - GLYPH_H - 3).max(0);
            paths.push(format!(
                r#"<path d="M{save_x},{accent_y} H{}" stroke="black" stroke-width="1" fill="none"/>"#,
                *x
            ));
        },

        MathNode::Group(nodes) => {
            for n in nodes {
                place_node(n, paths, x, baseline_y);
            }
        },

        MathNode::Delimiter { left, right, inner } => {
            // Left delimiter
            for ch in left.chars() {
                emit_glyph(ch, paths, *x, baseline_y - GLYPH_H);
                *x += GLYPH_W + 1;
            }
            for n in inner {
                place_node(n, paths, x, baseline_y);
            }
            // Right delimiter
            for ch in right.chars() {
                emit_glyph(ch, paths, *x, baseline_y - GLYPH_H);
                *x += GLYPH_W + 1;
            }
        },

        MathNode::Align(rows) => {
            let mut row_y = baseline_y;
            for row in rows {
                let mut row_x = *x;
                for n in row {
                    place_node(n, paths, &mut row_x, row_y);
                }
                *x = (*x).max(row_x);
                row_y += GLYPH_H + 4;
            }
        },

        MathNode::Cases(cases) => {
            let mut row_y = baseline_y;
            for (cond, result) in cases {
                let mut row_x = *x;
                for n in result {
                    place_node(n, paths, &mut row_x, row_y);
                }
                row_x += 8;
                for n in cond {
                    place_node(n, paths, &mut row_x, row_y);
                }
                *x = (*x).max(row_x);
                row_y += GLYPH_H + 4;
            }
        },

        MathNode::Space => {
            *x += GLYPH_W / 2;
        },
    }
}

/// Like [`place_node`] but renders the node at a scaled height.
///
/// This is used for superscripts, subscripts, numerators, and denominators
/// where the glyph height differs from `GLYPH_H`.
fn place_node_at_scale(node: &MathNode, paths: &mut Vec<String>, x: &mut i64, top_y: i64, h: i64) {
    let baseline = top_y + h;
    match node {
        MathNode::Text(s) | MathNode::TextRun(s) => {
            for ch in s.chars() {
                emit_glyph_sized(ch, paths, *x, top_y, (GLYPH_W * 2 / 3).max(4), h.max(4));
                *x += (GLYPH_W * 2 / 3 + 1).max(5);
            }
        },
        MathNode::Group(nodes) => {
            for n in nodes {
                place_node_at_scale(n, paths, x, top_y, h);
            }
        },
        _ => {
            // For complex nodes at scale, fall back to normal placement at the
            // requested baseline
            place_node(node, paths, x, baseline);
        },
    }
}

/// Emit a single glyph as a `<path>` element at position `(x, y)`.
///
/// The path is a glyph outline composed of rectangles and strokes that are
/// unique to each character. This ensures the output is actual path data, not
/// uniform rectangles, meeting AC-003's "path data" requirement.
fn emit_glyph(ch: char, paths: &mut Vec<String>, x: i64, y: i64) {
    emit_glyph_sized(ch, paths, x, y, GLYPH_W, GLYPH_H);
}

/// Emit a glyph outline at a specific size.
///
/// Each character class gets a distinct path shape:
/// - Digits: filled rectangular body with a notch to indicate the digit value
/// - Uppercase letters: stem + crossbar at varying heights
/// - Lowercase letters: shorter stem + ascender/descender hints
/// - Operators: symbolic strokes
/// - Other: bounding rectangle
#[allow(clippy::too_many_lines)]
#[allow(clippy::many_single_char_names)]
fn emit_glyph_sized(ch: char, paths: &mut Vec<String>, x: i64, y: i64, w: i64, h: i64) {
    let x1 = x;
    let y1 = y;
    let x2 = x + w;
    let y2 = y + h;
    let mid_x = x + w / 2;
    let mid_y = y + h / 2;
    let top_third = y + h / 3;
    let bot_third = y + 2 * h / 3;

    // Each character gets a distinct set of strokes so the path data is
    // semantically meaningful and distinguishable per-glyph.
    let d = match ch {
        // ── Digits ──────────────────────────────────────────────────────────
        '0' => format!(
            "M{x1},{top_third} Q{x1},{y1} {mid_x},{y1} Q{x2},{y1} {x2},{top_third} L{x2},{bot_third} Q{x2},{y2} {mid_x},{y2} Q{x1},{y2} {x1},{bot_third} Z"
        ),
        '1' => format!(
            "M{mid_x},{y1} L{mid_x},{y2} M{},{mid_y} L{mid_x},{y1}",
            x + w / 4
        ),
        '2' => format!(
            "M{x1},{top_third} Q{x1},{y1} {mid_x},{y1} Q{x2},{y1} {x2},{top_third} Q{x2},{mid_y} {x1},{y2} L{x2},{y2}"
        ),
        '3' => format!("M{x1},{y1} H{x2} L{mid_x},{mid_y} H{x2} L{x1},{y2}"),
        '4' => format!(
            "M{},{y1} L{x1},{mid_y} H{x2} M{},{y1} V{y2}",
            x + 3 * w / 4,
            x + 3 * w / 4
        ),
        '5' => format!("M{x2},{y1} H{x1} V{mid_y} H{x2} Q{x2},{y2} {x1},{y2}"),
        '6' => format!(
            "M{x2},{y1} Q{x1},{y1} {x1},{mid_y} V{bot_third} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{bot_third} Q{x2},{mid_y} {x1},{mid_y}"
        ),
        '7' => format!("M{x1},{y1} H{x2} L{x1},{y2}"),
        '8' => format!(
            "M{mid_x},{mid_y} Q{x1},{mid_y} {x1},{top_third} Q{x1},{y1} {mid_x},{y1} Q{x2},{y1} {x2},{top_third} Q{x2},{mid_y} {mid_x},{mid_y} Q{x1},{mid_y} {x1},{bot_third} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{bot_third} Q{x2},{mid_y} {mid_x},{mid_y}"
        ),
        '9' => format!(
            "M{x1},{y2} Q{x2},{y2} {x2},{mid_y} V{top_third} Q{x2},{y1} {mid_x},{y1} Q{x1},{y1} {x1},{top_third} Q{x1},{mid_y} {x2},{mid_y}"
        ),

        // ── Uppercase letters ───────────────────────────────────────────────
        'A' => format!(
            "M{x1},{y2} L{mid_x},{y1} L{x2},{y2} M{},{mid_y} H{}",
            x + w / 4,
            x + 3 * w / 4
        ),
        'B' => format!(
            "M{x1},{y1} V{y2} H{} Q{x2},{y2} {x2},{bot_third} Q{x2},{mid_y} {x1},{mid_y} H{} Q{x2},{mid_y} {x2},{top_third} Q{x2},{y1} {x1},{y1}",
            x + 3 * w / 4,
            x + 3 * w / 4
        ),
        'C' => format!(
            "M{x2},{top_third} Q{x2},{y1} {mid_x},{y1} Q{x1},{y1} {x1},{mid_y} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{bot_third}"
        ),
        'D' => {
            format!("M{x1},{y1} V{y2} H{mid_x} Q{x2},{y2} {x2},{mid_y} Q{x2},{y1} {mid_x},{y1} Z")
        },
        'E' => format!(
            "M{x2},{y1} H{x1} V{y2} H{x2} M{x1},{mid_y} H{}",
            x + 3 * w / 4
        ),
        'F' => format!("M{x2},{y1} H{x1} V{y2} M{x1},{mid_y} H{}", x + 3 * w / 4),
        'G' => format!(
            "M{x2},{top_third} Q{x2},{y1} {mid_x},{y1} Q{x1},{y1} {x1},{mid_y} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{bot_third} H{mid_x}"
        ),
        'H' => format!("M{x1},{y1} V{y2} M{x2},{y1} V{y2} M{x1},{mid_y} H{x2}"),
        'I' => format!("M{x1},{y1} H{x2} M{mid_x},{y1} V{y2} M{x1},{y2} H{x2}"),
        'J' => {
            format!("M{x2},{y1} V{bot_third} Q{x2},{y2} {mid_x},{y2} Q{x1},{y2} {x1},{bot_third}")
        },
        'K' => format!("M{x1},{y1} V{y2} M{x2},{y1} L{x1},{mid_y} L{x2},{y2}"),
        'L' => format!("M{x1},{y1} V{y2} H{x2}"),
        'M' => format!("M{x1},{y2} V{y1} L{mid_x},{mid_y} L{x2},{y1} V{y2}"),
        'N' => format!("M{x1},{y2} V{y1} L{x2},{y2} V{y1}"),
        'O' => format!(
            "M{mid_x},{y1} Q{x1},{y1} {x1},{mid_y} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{mid_y} Q{x2},{y1} {mid_x},{y1} Z"
        ),
        'P' => format!(
            "M{x1},{y2} V{y1} H{} Q{x2},{y1} {x2},{top_third} Q{x2},{mid_y} {x1},{mid_y}",
            x + 3 * w / 4
        ),
        'Q' => format!(
            "M{mid_x},{y1} Q{x1},{y1} {x1},{mid_y} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{mid_y} Q{x2},{y1} {mid_x},{y1} Z M{},{} L{x2},{y2}",
            x + 3 * w / 5,
            y + 3 * h / 5
        ),
        'R' => format!(
            "M{x1},{y2} V{y1} H{} Q{x2},{y1} {x2},{top_third} Q{x2},{mid_y} {x1},{mid_y} L{x2},{y2}",
            x + 3 * w / 4
        ),
        'S' => format!(
            "M{x2},{top_third} Q{x2},{y1} {mid_x},{y1} Q{x1},{y1} {x1},{top_third} Q{x1},{mid_y} {x2},{mid_y} Q{x2},{bot_third} {x2},{bot_third} Q{x2},{y2} {mid_x},{y2} Q{x1},{y2} {x1},{bot_third}"
        ),
        'T' => format!("M{x1},{y1} H{x2} M{mid_x},{y1} V{y2}"),
        // U and ∪ share the same arc outline at this scale
        'U' | '\u{222A}' => format!(
            "M{x1},{y1} V{bot_third} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{bot_third} V{y1}"
        ),
        'V' => format!("M{x1},{y1} L{mid_x},{y2} L{x2},{y1}"),
        'W' => format!(
            "M{x1},{y1} L{},{y2} L{mid_x},{mid_y} L{},{y2} L{x2},{y1}",
            x + w / 4,
            x + 3 * w / 4
        ),
        // X and × share the same cross outline
        'X' | '\u{00D7}' => format!("M{x1},{y1} L{x2},{y2} M{x2},{y1} L{x1},{y2}"),
        'Y' => format!("M{x1},{y1} L{mid_x},{mid_y} L{x2},{y1} M{mid_x},{mid_y} V{y2}"),
        'Z' => format!("M{x1},{y1} H{x2} L{x1},{y2} H{x2}"),

        // ── Lowercase letters ───────────────────────────────────────────────
        'a' => format!(
            "M{x2},{mid_y} Q{x2},{top_third} {mid_x},{top_third} Q{x1},{top_third} {x1},{mid_y} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{bot_third} V{y2}"
        ),
        'b' => format!("M{x1},{y1} V{y2} Q{x1},{y2} {x2},{bot_third} Q{x2},{mid_y} {x1},{mid_y}"),
        'c' => format!(
            "M{x2},{mid_y} Q{x2},{top_third} {mid_x},{top_third} Q{x1},{top_third} {x1},{mid_y} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{bot_third}"
        ),
        'd' => format!("M{x2},{y1} V{y2} Q{x2},{y2} {x1},{bot_third} Q{x1},{mid_y} {x2},{mid_y}"),
        'e' => format!(
            "M{x1},{mid_y} H{x2} Q{x2},{top_third} {mid_x},{top_third} Q{x1},{top_third} {x1},{mid_y} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{bot_third}"
        ),
        'f' => format!("M{mid_x},{y2} V{top_third} Q{mid_x},{y1} {x2},{y1} M{x1},{mid_y} H{x2}"),
        'g' => format!(
            "M{x2},{top_third} Q{x2},{top_third} {mid_x},{top_third} Q{x1},{top_third} {x1},{mid_y} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{mid_y} V{y2} Q{x2},{y2} {x1},{y2}"
        ),
        'h' => format!("M{x1},{y1} V{y2} M{x1},{mid_y} Q{x1},{top_third} {x2},{top_third} V{y2}"),
        'i' => format!(
            "M{mid_x},{mid_y} V{y2} M{mid_x},{} V{}",
            y + h / 8,
            y + h / 5
        ),
        'j' => format!(
            "M{mid_x},{mid_y} V{y2} Q{mid_x},{y2} {x1},{y2} M{mid_x},{} V{}",
            y + h / 8,
            y + h / 5
        ),
        'k' => format!("M{x1},{y1} V{y2} M{x2},{mid_y} L{x1},{mid_y} L{x2},{y2}"),
        // l and | are both a vertical stem
        'l' | '|' => format!("M{mid_x},{y1} V{y2}"),
        'm' => format!(
            "M{x1},{mid_y} V{y2} M{x1},{mid_y} Q{x1},{top_third} {mid_x},{top_third} V{y2} M{mid_x},{mid_y} Q{mid_x},{top_third} {x2},{top_third} V{y2}"
        ),
        'n' => {
            format!("M{x1},{mid_y} V{y2} M{x1},{mid_y} Q{x1},{top_third} {x2},{top_third} V{y2}")
        },
        'o' => format!(
            "M{mid_x},{top_third} Q{x1},{top_third} {x1},{mid_y} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{mid_y} Q{x2},{top_third} {mid_x},{top_third} Z"
        ),
        'p' => format!(
            "M{x1},{mid_y} V{y2} M{x1},{mid_y} Q{x1},{top_third} {x2},{top_third} Q{x2},{mid_y} {x1},{mid_y}"
        ),
        'q' => format!(
            "M{x2},{mid_y} V{y2} M{x2},{mid_y} Q{x2},{top_third} {x1},{top_third} Q{x1},{mid_y} {x2},{mid_y}"
        ),
        'r' => format!("M{x1},{mid_y} V{y2} M{x1},{mid_y} Q{x1},{top_third} {x2},{top_third}"),
        's' => format!(
            "M{x2},{mid_y} Q{x2},{top_third} {mid_x},{top_third} Q{x1},{top_third} {x1},{mid_y} Q{x1},{bot_third} {x2},{bot_third} Q{x2},{y2} {mid_x},{y2} Q{x1},{y2} {x1},{bot_third}"
        ),
        // t and + share a vertical stem + horizontal crossbar
        't' | '+' => format!("M{mid_x},{y1} V{y2} M{x1},{mid_y} H{x2}"),
        'u' => format!(
            "M{x1},{top_third} V{bot_third} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{bot_third} V{top_third}"
        ),
        'v' => format!("M{x1},{top_third} L{mid_x},{y2} L{x2},{top_third}"),
        'w' => format!(
            "M{x1},{top_third} L{},{y2} L{mid_x},{bot_third} L{},{y2} L{x2},{top_third}",
            x + w / 4,
            x + 3 * w / 4
        ),
        'x' => format!("M{x1},{top_third} L{x2},{y2} M{x2},{top_third} L{x1},{y2}"),
        'y' => format!(
            "M{x1},{top_third} L{mid_x},{bot_third} L{x2},{top_third} M{mid_x},{bot_third} L{x1},{y2}"
        ),
        'z' => format!("M{x1},{top_third} H{x2} L{x1},{y2} H{x2}"),

        // ── Common math operators / symbols ─────────────────────────────────
        // '+' is merged with 't' above (both share vertical+horizontal strokes)
        '-' => format!("M{x1},{mid_y} H{x2}"),
        '=' => format!("M{x1},{top_third} H{x2} M{x1},{bot_third} H{x2}"),
        '*' => format!(
            "M{mid_x},{y1} V{y2} M{x1},{top_third} L{x2},{bot_third} M{x2},{top_third} L{x1},{bot_third}"
        ),
        '/' => format!("M{x2},{y1} L{x1},{y2}"),
        '(' => format!("M{mid_x},{y1} Q{x1},{y1} {x1},{mid_y} Q{x1},{y2} {mid_x},{y2}"),
        ')' => format!("M{mid_x},{y1} Q{x2},{y1} {x2},{mid_y} Q{x2},{y2} {mid_x},{y2}"),
        '[' => format!("M{mid_x},{y1} H{x1} V{y2} H{mid_x}"),
        ']' => format!("M{mid_x},{y1} H{x2} V{y2} H{mid_x}"),
        '{' => format!(
            "M{mid_x},{y1} Q{x1},{y1} {x1},{top_third} Q{x1},{mid_y} {x1},{mid_y} Q{x1},{mid_y} {x1},{bot_third} Q{x1},{y2} {mid_x},{y2}"
        ),
        '}' => format!(
            "M{mid_x},{y1} Q{x2},{y1} {x2},{top_third} Q{x2},{mid_y} {x2},{mid_y} Q{x2},{mid_y} {x2},{bot_third} Q{x2},{y2} {mid_x},{y2}"
        ),
        '<' => format!("M{x2},{y1} L{x1},{mid_y} L{x2},{y2}"),
        '>' => format!("M{x1},{y1} L{x2},{mid_y} L{x1},{y2}"),
        ',' => format!("M{mid_x},{bot_third} Q{mid_x},{y2} {x1},{y2}"),
        '.' => format!("M{mid_x},{y2} V{y2}"),
        ':' => format!("M{mid_x},{top_third} V{top_third} M{mid_x},{bot_third} V{bot_third}"),
        '^' => format!("M{x1},{mid_y} L{mid_x},{y1} L{x2},{mid_y}"),
        '_' => format!("M{x1},{y2} H{x2}"),
        // '|' is merged with 'l' above (both are vertical stems)
        '!' => format!("M{mid_x},{y1} V{bot_third} M{mid_x},{y2} V{y2}"),

        // ── Unicode operators emitted by operator_symbol() / misc_symbol() ──
        '\u{2211}' => format!("M{x2},{y1} H{x1} L{x2},{mid_y} L{x1},{y2} H{x2}"), // ∑
        '\u{220F}' => format!("M{x1},{y2} V{y1} H{x2} V{y2}"),                    // ∏
        // ∫ and ⊂ share an S-curve outline at this scale
        '\u{222B}' | '\u{2282}' => {
            format!("M{x2},{y1} Q{x1},{y1} {x1},{mid_y} Q{x1},{y2} {x2},{y2}")
        }, // ∫ ⊂
        '\u{221E}' => format!(
            "M{mid_x},{mid_y} Q{x1},{y1} {x1},{mid_y} Q{x1},{y2} {mid_x},{mid_y} Q{x2},{y1} {x2},{mid_y} Q{x2},{y2} {mid_x},{mid_y}"
        ), // ∞
        '\u{00B1}' => format!("M{mid_x},{y1} V{mid_y} M{x1},{top_third} H{x2} M{x1},{y2} H{x2}"), // ±
        // '\u{00D7}' (×) is merged with 'X' in the uppercase block above
        '\u{00F7}' => format!(
            "M{x1},{mid_y} H{x2} M{mid_x},{top_third} V{top_third} M{mid_x},{bot_third} V{bot_third}"
        ), // ÷
        '\u{2264}' => format!("M{x2},{y1} L{x1},{mid_y} L{x2},{y2} M{x1},{y2} H{x2}"), // ≤
        '\u{2265}' => format!("M{x1},{y1} L{x2},{mid_y} L{x1},{y2} M{x1},{y2} H{x2}"), // ≥
        '\u{2260}' => format!(
            "M{x1},{mid_y} H{x2} M{},{top_third} L{},{bot_third} M{x1},{bot_third} H{x2}",
            x + 3 * w / 4,
            x + w / 4
        ), // ≠
        '\u{2248}' => format!(
            "M{x1},{top_third} Q{mid_x},{y1} {x2},{top_third} M{x1},{bot_third} Q{mid_x},{mid_y} {x2},{bot_third}"
        ), // ≈
        '\u{2261}' => {
            format!("M{x1},{top_third} H{x2} M{x1},{mid_y} H{x2} M{x1},{bot_third} H{x2}")
        }, // ≡
        '\u{2208}' => format!(
            "M{x2},{top_third} Q{x2},{y1} {mid_x},{y1} Q{x1},{y1} {x1},{mid_y} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{bot_third} M{x1},{mid_y} H{x2}"
        ), // ∈
        // '\u{222A}' (∪) is merged with 'U' in the uppercase block above
        '\u{2229}' => format!(
            "M{x1},{y2} V{top_third} Q{x1},{y1} {mid_x},{y1} Q{x2},{y1} {x2},{top_third} V{y2}"
        ), // ∩
        '\u{2200}' => format!("M{x1},{y1} L{mid_x},{y2} L{x2},{y1} H{x1} M{x1},{mid_y} H{x2}"), // ∀
        '\u{2203}' => format!("M{x2},{y1} H{x1} V{mid_y} H{x2} V{y2} H{x1}"),                   // ∃
        '\u{2202}' => format!(
            "M{x2},{top_third} Q{x2},{y1} {mid_x},{y1} Q{x1},{y1} {x1},{mid_y} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{mid_y} Q{x2},{top_third} {x1},{top_third}"
        ), // ∂
        '\u{2207}' => format!("M{x1},{y1} L{mid_x},{y2} L{x2},{y1} Z"),                         // ∇
        '\u{2192}' => format!(
            "M{x1},{mid_y} H{x2} M{},{top_third} L{x2},{mid_y} L{},{bot_third}",
            x + 3 * w / 4,
            x + 3 * w / 4
        ), // →
        '\u{2190}' => format!(
            "M{x2},{mid_y} H{x1} M{},{top_third} L{x1},{mid_y} L{},{bot_third}",
            x + w / 4,
            x + w / 4
        ), // ←
        '\u{21D2}' => format!(
            "M{x1},{top_third} H{} M{x1},{bot_third} H{} M{},{y1} L{x2},{mid_y} L{},{y2}",
            x + 3 * w / 4,
            x + 3 * w / 4,
            x + 3 * w / 4,
            x + 3 * w / 4
        ), // ⇒
        '\u{22C5}' => format!("M{mid_x},{mid_y} V{mid_y}"), // ⋅ (dot)
        '\u{03B1}' => format!(
            "M{x2},{top_third} Q{mid_x},{top_third} {x1},{mid_y} Q{x1},{y2} {mid_x},{y2} Q{x2},{y2} {x2},{bot_third} M{x2},{top_third} V{y2}"
        ), // α
        '\u{03B2}' => format!(
            "M{x1},{y1} V{y2} Q{x1},{y2} {x2},{bot_third} Q{x2},{mid_y} {x1},{mid_y} Q{x2},{mid_y} {x2},{top_third} Q{x2},{y1} {x1},{y1}"
        ), // β
        '\u{03C0}' => format!(
            "M{x1},{top_third} H{x2} M{},{top_third} V{y2} M{},{top_third} V{y2}",
            x + w / 4,
            x + 3 * w / 4
        ), // π

        // ── Fallback for all other characters: a labelled bounding rectangle ─
        _ => {
            // A simple rectangle so every character has a deterministic,
            // non-empty path even for uncommon glyphs.
            format!("M{x1},{y1} H{x2} V{y2} H{x1} Z")
        },
    };

    paths.push(format!(
        r#"<path d="{d}" stroke="black" stroke-width="0.5" fill="none"/>"#
    ));
}

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
        assert!(!paths.svg.is_empty(), "SvgPaths.svg must not be empty");
        // The svg field must look like an SVG document
        assert!(
            paths.svg.contains("<svg") || paths.svg.contains("<?xml"),
            "SvgPaths.svg must start with an SVG root element; got: {}",
            &paths.svg[..paths.svg.len().min(120)]
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
            !paths.svg.contains("<text"),
            "SVG must not contain <text> elements (PDF/UA-1); got: {}",
            &paths.svg[..paths.svg.len().min(400)]
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
            !paths.svg.contains("<image"),
            "SVG must not contain <image> elements (AC-003); got: {}",
            &paths.svg[..paths.svg.len().min(400)]
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
            paths.svg.contains(r#"<path d=""#) || paths.svg.contains("<path "),
            "SVG must contain <path> elements with glyph outlines; got: {}",
            &paths.svg[..paths.svg.len().min(400)]
        );
    }

    /// `render_pdf_paths` returns positive EMU dimensions.
    ///
    /// BC-1.10.003 invariant: bounding box must have positive width and height.
    ///
    /// RED GATE: fails until `render_pdf_paths` is implemented.
    #[test]
    fn test_bc_1_10_003_pdf_paths_dimensions_positive() {
        let ast = inline_ast(vec![MathNode::Text(Arc::from("x"))]);
        let paths = render_pdf_paths(&ast).expect("render_pdf_paths must succeed");
        assert!(
            paths.width_emu > 0,
            "width_emu must be positive; got: {}",
            paths.width_emu
        );
        assert!(
            paths.height_emu > 0,
            "height_emu must be positive; got: {}",
            paths.height_emu
        );
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
        assert_wellformed_xml(&paths.svg)
            .unwrap_or_else(|e| panic!("PDF path SVG is not well-formed XML: {e}"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-1.10.003 — EC-005: usvg fails → TextRemainsInPathOutput
    // ─────────────────────────────────────────────────────────────────────────

    /// [`SvgPaths`] struct fields are accessible, and the type is `Clone + PartialEq + Hash`.
    ///
    /// This is a compile-time property test — the struct definition already exists
    /// in the stub, so this test PASSES even before `render_pdf_paths` is implemented.
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
        assert!(!paths.svg.is_empty(), "SVG must not be empty");
        assert!(paths.width_emu > 0, "width_emu must be positive");
        assert!(paths.height_emu > 0, "height_emu must be positive");
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
}
