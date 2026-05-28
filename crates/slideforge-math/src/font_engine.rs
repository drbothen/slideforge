//! Font glyph outline engine for `slideforge-math`.
//!
//! Uses [`ab_glyph`] to rasterise real glyph outlines from the bundled
//! **Latin Modern Math** OpenType font (GUST Font License). Replaces the
//! previous hand-coded synthetic-rectangle glyph table.
//!
//! ## Coordinate system
//!
//! [`ab_glyph`] returns font-space coordinates with y increasing upward and the
//! baseline at y = 0. SVG uses y increasing downward. The [`GlyphEngine`]
//! converts automatically: given a target cell `(left, top, width, height)` in
//! SVG pixel coordinates the engine places the baseline at `top + height` and
//! negates the y component from the font.
//!
//! ## Scaling
//!
//! All output coordinates are integers (rounded to the nearest pixel) to
//! preserve compatibility with the EMU-first integer arithmetic used throughout
//! slideforge's IR layer.
//!
//! ## Fallback
//!
//! Characters without a glyph in Latin Modern Math fall back to a rectangular
//! bounding-box marker path so the output is always non-empty.

#![allow(clippy::cast_precision_loss)] // i64 cell coords ≤ a few hundred pixels; f32 is exact
#![allow(clippy::cast_possible_truncation)] // advance.round() → i64: clamped to ≥1, no overflow
#![allow(clippy::many_single_char_names)] // (x, y, w, h) are canonical names for glyph geometry

use ab_glyph::{Font, FontRef, GlyphId, OutlineCurve};
use slideforge_plugin_api::MathError;

/// The bundled Latin Modern Math OTF font bytes.
///
/// Latin Modern Math is distributed under the GUST Font License (GFL), an
/// open, permissive font licence based on LPPL 1.3c. The font is embedded as
/// raw bytes — it is *not* a Rust crate dependency and therefore is not subject
/// to `cargo deny` licence policy (which covers only crate dependencies).
const LATIN_MODERN_MATH: &[u8] = include_bytes!("../fonts/latinmodern-math.otf");

/// Glyph outline engine backed by the bundled Latin Modern Math font.
///
/// `GlyphEngine` is intended to be constructed once and reused across calls
/// to [`render_pdf_paths`][crate::pdf_paths::render_pdf_paths]. It is `!Send`
/// (due to `FontRef` borrowing `LATIN_MODERN_MATH`'s static bytes) but is
/// safe to pass to multi-threaded code via `Arc<GlyphEngine>` if needed.
pub struct GlyphEngine {
    /// Reference to the embedded font, borrowing the static byte slice.
    font: FontRef<'static>,
    /// Units per EM for the font (typically 1000 for LM Math).
    units_per_em: f32,
}

impl GlyphEngine {
    /// Construct a new [`GlyphEngine`] from the embedded Latin Modern Math font.
    ///
    /// # Panics
    ///
    /// Panics only if the embedded font bytes are corrupt (build-time invariant
    /// violation — cannot happen in a correct build).
    #[must_use]
    pub fn new() -> Self {
        let font = FontRef::try_from_slice(LATIN_MODERN_MATH)
            .expect("embedded Latin Modern Math OTF must be valid — font bytes are corrupt");
        let units_per_em = font.units_per_em().unwrap_or(1000.0);
        Self { font, units_per_em }
    }

    /// Emit a real glyph outline for `ch` as one or more SVG `<path>` elements.
    ///
    /// The glyph is scaled to fit inside the cell `(x, y, w, h)` where `(x, y)`
    /// is the top-left corner, `w` is the width, and `h` is the height in SVG
    /// pixels. The baseline is placed at `y + h`.
    ///
    /// If the font has no glyph for `ch`, a rectangular fallback is emitted.
    ///
    /// # Errors
    ///
    /// Returns `Err` only if writing to the output buffer fails (currently
    /// infallible since the buffer is an in-memory `Vec<String>`).
    pub fn emit_glyph(
        &self,
        ch: char,
        paths: &mut Vec<String>,
        x: i64,
        y: i64,
        w: i64,
        h: i64,
    ) -> Result<(), MathError> {
        let glyph_id = self.font.glyph_id(ch);

        // Fallback: characters with no glyph (glyph_id == 0 means .notdef)
        // get a rectangular bounding-box marker so the path count is preserved.
        if glyph_id == GlyphId(0) {
            Self::emit_fallback_rect(paths, x, y, w, h);
            return Ok(());
        }

        let Some(outline) = self.font.outline(glyph_id) else {
            // Glyph exists but has no outline (e.g. whitespace).
            // Emit a tiny invisible rectangle to keep path count stable.
            Self::emit_fallback_rect(paths, x, y, w, h);
            return Ok(());
        };

        // Scale: map font units to pixel height h.
        // Font coordinates have baseline at y=0, ascenders positive, descenders
        // negative. We map height h to the full EM extent.
        let scale = h as f32 / self.units_per_em;
        // SVG baseline (y increases downward, baseline at bottom of cell).
        let baseline_svg = (y + h) as f32;
        let left_svg = x as f32;

        // Convert all outline curves to a single SVG path `d` string.
        // Each curve segment begins with an absolute move or continues from the
        // current pen position. We emit an `M` command before each segment for
        // maximum portability (no implicit-pen-position dependency).
        let mut d_parts: Vec<String> = Vec::with_capacity(outline.curves.len() * 2);
        let mut pen: Option<(f32, f32)> = None;

        for curve in &outline.curves {
            match curve {
                OutlineCurve::Line(p0, p1) => {
                    let (sx0, sy0) = Self::font_to_svg(p0.x, p0.y, left_svg, baseline_svg, scale);
                    let (sx1, sy1) = Self::font_to_svg(p1.x, p1.y, left_svg, baseline_svg, scale);
                    if needs_move(pen, sx0, sy0) {
                        d_parts.push(format!("M{},{}", px_round(sx0), px_round(sy0)));
                    }
                    d_parts.push(format!("L{},{}", px_round(sx1), px_round(sy1)));
                    pen = Some((sx1, sy1));
                },
                OutlineCurve::Quad(p0, p1, p2) => {
                    let (sx0, sy0) = Self::font_to_svg(p0.x, p0.y, left_svg, baseline_svg, scale);
                    let (sx1, sy1) = Self::font_to_svg(p1.x, p1.y, left_svg, baseline_svg, scale);
                    let (sx2, sy2) = Self::font_to_svg(p2.x, p2.y, left_svg, baseline_svg, scale);
                    if needs_move(pen, sx0, sy0) {
                        d_parts.push(format!("M{},{}", px_round(sx0), px_round(sy0)));
                    }
                    d_parts.push(format!(
                        "Q{},{} {},{}",
                        px_round(sx1),
                        px_round(sy1),
                        px_round(sx2),
                        px_round(sy2)
                    ));
                    pen = Some((sx2, sy2));
                },
                OutlineCurve::Cubic(p0, p1, p2, p3) => {
                    let (sx0, sy0) = Self::font_to_svg(p0.x, p0.y, left_svg, baseline_svg, scale);
                    let (sx1, sy1) = Self::font_to_svg(p1.x, p1.y, left_svg, baseline_svg, scale);
                    let (sx2, sy2) = Self::font_to_svg(p2.x, p2.y, left_svg, baseline_svg, scale);
                    let (sx3, sy3) = Self::font_to_svg(p3.x, p3.y, left_svg, baseline_svg, scale);
                    if needs_move(pen, sx0, sy0) {
                        d_parts.push(format!("M{},{}", px_round(sx0), px_round(sy0)));
                    }
                    d_parts.push(format!(
                        "C{},{} {},{} {},{}",
                        px_round(sx1),
                        px_round(sy1),
                        px_round(sx2),
                        px_round(sy2),
                        px_round(sx3),
                        px_round(sy3)
                    ));
                    pen = Some((sx3, sy3));
                },
            }
        }

        if d_parts.is_empty() {
            // Outline has no curves — whitespace or invisible glyph.
            Self::emit_fallback_rect(paths, x, y, w, h);
            return Ok(());
        }

        let d = d_parts.join(" ");
        paths.push(format!(
            r#"<path d="{d}" stroke="black" stroke-width="0.5" fill="none"/>"#
        ));
        Ok(())
    }

    /// Return the advance width of `ch` in SVG pixels at the given cell height.
    ///
    /// Falls back to `nominal_w` (the nominal cell width) for unknown glyphs.
    #[must_use]
    pub fn advance_width(&self, ch: char, h: i64, nominal_w: i64) -> i64 {
        let glyph_id = self.font.glyph_id(ch);
        if glyph_id == GlyphId(0) {
            return nominal_w;
        }
        let scale = h as f32 / self.units_per_em;
        let advance = self.font.h_advance_unscaled(glyph_id) * scale;
        // Ensure at least 1 pixel of advance to prevent zero-width glyphs.
        (advance.round() as i64).max(1)
    }

    /// Convert font-space `(fx, fy)` to SVG pixel coordinates.
    ///
    /// - `left_svg`: x offset of the glyph cell's left edge in SVG pixels
    /// - `baseline_svg`: y coordinate of the baseline in SVG pixels (y down)
    /// - `scale`: pixels per font unit
    #[inline]
    fn font_to_svg(fx: f32, fy: f32, left_svg: f32, baseline_svg: f32, scale: f32) -> (f32, f32) {
        // x: font x is relative to the glyph origin (left bearing), add cell offset
        let sx = left_svg + fx * scale;
        // y: font y increases upward, SVG y increases downward
        let sy = baseline_svg - fy * scale;
        (sx, sy)
    }

    /// Emit a rectangular fallback path for glyphs with no outline data.
    ///
    /// The rectangle matches the glyph cell bounds exactly so it is
    /// visually distinct from glyph outlines (which are typically narrower).
    fn emit_fallback_rect(paths: &mut Vec<String>, x: i64, y: i64, w: i64, h: i64) {
        let x2 = x + w;
        let y2 = y + h;
        paths.push(format!(
            r#"<path d="M{x},{y} H{x2} V{y2} H{x} Z" stroke="black" stroke-width="0.5" fill="none"/>"#
        ));
    }
}

/// Return `true` if the pen must emit an `M` command before drawing.
///
/// An `M` is needed when:
/// - the pen has no current position (`pen` is `None`), or
/// - the pen's current position is more than 0.5 px away from `(tx, ty)`.
#[inline]
fn needs_move(pen: Option<(f32, f32)>, tx: f32, ty: f32) -> bool {
    match pen {
        None => true,
        Some((px, py)) => (px - tx).abs() > 0.5 || (py - ty).abs() > 0.5,
    }
}

/// Round a floating-point pixel coordinate to the nearest integer.
///
/// Converts `f32` to an integer pixel coordinate for SVG path data. Rounding
/// keeps the output compact (no sub-pixel noise) while preserving visual
/// correctness at typical math-display sizes (14 px cell height).
#[inline]
fn px_round(v: f32) -> i64 {
    v.round() as i64
}

impl Default for GlyphEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_font_engine_constructs_successfully() {
        let engine = GlyphEngine::new();
        // The engine must have a valid EM size (Latin Modern Math uses 1000 units/EM).
        assert!(engine.units_per_em > 0.0, "units_per_em must be positive");
    }

    #[test]
    fn test_font_engine_emit_latin_glyph_produces_path() {
        let engine = GlyphEngine::new();
        let mut paths: Vec<String> = Vec::new();
        engine.emit_glyph('x', &mut paths, 0, 0, 10, 14).unwrap();
        assert!(
            !paths.is_empty(),
            "emit_glyph for 'x' must produce at least one path"
        );
        let svg_path = &paths[0];
        assert!(
            svg_path.contains("<path ") && svg_path.contains(" d=\""),
            "emitted element must be an SVG <path> with a d attribute; got: {svg_path}"
        );
    }

    #[test]
    fn test_font_engine_emit_greek_glyph_produces_path() {
        let engine = GlyphEngine::new();
        let mut paths: Vec<String> = Vec::new();
        // γ (U+03B3 — Greek small letter gamma)
        engine
            .emit_glyph('\u{03B3}', &mut paths, 0, 0, 10, 14)
            .unwrap();
        assert!(
            !paths.is_empty(),
            "emit_glyph for γ must produce at least one path"
        );
    }

    #[test]
    fn test_font_engine_glyph_contains_bezier_curves() {
        let engine = GlyphEngine::new();
        let mut paths: Vec<String> = Vec::new();
        engine.emit_glyph('E', &mut paths, 0, 0, 10, 14).unwrap();
        // A real font glyph outline for a capital letter should contain at least
        // one curve command (L, Q, or C) indicating a real outline, not just a
        // rectangle (which would only contain H/V/Z commands).
        let d = paths.join(" ");
        assert!(
            d.contains('L') || d.contains('Q') || d.contains('C'),
            "glyph outline for 'E' must contain at least one curve or line command (not just rectangle corners); got: {d}"
        );
    }

    #[test]
    fn test_font_engine_gamma_distinct_from_latin_g() {
        let engine = GlyphEngine::new();
        let mut gamma_paths: Vec<String> = Vec::new();
        let mut g_paths: Vec<String> = Vec::new();
        // γ (U+03B3) vs 'g' (U+0067)
        engine
            .emit_glyph('\u{03B3}', &mut gamma_paths, 0, 0, 10, 14)
            .unwrap();
        engine.emit_glyph('g', &mut g_paths, 0, 0, 10, 14).unwrap();
        assert_ne!(
            gamma_paths, g_paths,
            "γ (U+03B3) outline must differ from Latin 'g' outline"
        );
    }

    #[test]
    fn test_font_engine_advance_width_positive() {
        let engine = GlyphEngine::new();
        let advance = engine.advance_width('x', 14, 10);
        assert!(
            advance > 0,
            "advance width for 'x' must be positive; got: {advance}"
        );
    }

    #[test]
    fn test_font_engine_fallback_for_unknown_char() {
        let engine = GlyphEngine::new();
        // U+FFF0 is in a Private Use Area — very unlikely to be in LM Math.
        // We test that it emits *something* (the fallback rectangle) without panicking.
        let mut paths: Vec<String> = Vec::new();
        engine
            .emit_glyph('\u{FFF0}', &mut paths, 0, 0, 10, 14)
            .unwrap();
        assert!(
            !paths.is_empty(),
            "fallback must emit a rectangle path for unknown char"
        );
        assert!(
            paths[0].contains('Z'),
            "fallback rectangle path must end with Z (closed path); got: {}",
            paths[0]
        );
    }
}
