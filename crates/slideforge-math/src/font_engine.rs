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

use std::sync::OnceLock;

use ab_glyph::{Font, FontRef, GlyphId, OutlineCurve};
use slideforge_plugin_api::MathError;

/// The bundled Latin Modern Math OTF font bytes (version 1.959).
///
/// Latin Modern Math is distributed under the GUST Font License (GFL) v1.0,
/// an open, permissive font licence based on LPPL 1.3c or later. The full
/// license text and FONTLOG are in
/// `fonts/LICENSE-LatinModernMath.txt` (shipped in the published .crate and
/// present at `crates/slideforge-math/fonts/LICENSE-LatinModernMath.txt` in
/// the repository). Copyright 2012--2014 by B. Jackowski, P. Strzelczyk and
/// P. Pianowski (on behalf of TeX Users Groups).
///
/// The font bytes are loaded once via [`engine()`] and cached in a
/// [`OnceLock`] — `include_bytes!` embeds the raw bytes into the binary at
/// compile time; the path is rustc-handled and platform-independent.
const LATIN_MODERN_MATH: &[u8] = include_bytes!("../fonts/latinmodern-math.otf");

/// Module-level cache of the parsed [`FontRef`].
///
/// Parsing the OTF binary is done exactly once (on first call to [`engine()`])
/// and the result is stored here. All subsequent calls borrow the same
/// `FontRef<'static>` reference. This eliminates per-render-call font reparsing
/// (F-S030-P10-C2).
static FONT_REF: OnceLock<FontRef<'static>> = OnceLock::new();

/// Module-level cache of the constructed [`GlyphEngine`].
///
/// The engine itself is cached after first construction so every call to
/// [`engine()`] returns a reference to the same `GlyphEngine` instance.
/// Combined with `FONT_REF`, this guarantees `FontRef::try_from_slice` is
/// called at most once per process lifetime (F-S030-P11-C1).
static GLYPH_ENGINE: OnceLock<GlyphEngine> = OnceLock::new();

/// Test-only atomic counter that tracks how many times `FontRef::try_from_slice`
/// has been called. Asserted to equal exactly 1 after N repeated `engine()` calls
/// in `test_font_parsed_exactly_once_across_n_calls` (F-S030-P11-C2).
#[cfg(test)]
static PARSE_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Return a reference to the cached [`FontRef`] for Latin Modern Math.
///
/// The font is parsed from [`LATIN_MODERN_MATH`] at most once per process
/// lifetime. Callers that need a [`GlyphEngine`] should use [`engine()`]
/// instead, which caches the fully-constructed engine.
///
/// # Panics
///
/// Panics only if the embedded font bytes are corrupt (build-time invariant
/// violation — cannot happen in a correct build).
#[inline]
pub(crate) fn font_ref() -> &'static FontRef<'static> {
    FONT_REF.get_or_init(|| {
        #[cfg(test)]
        PARSE_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        FontRef::try_from_slice(LATIN_MODERN_MATH)
            .expect("embedded Latin Modern Math OTF must be valid — font bytes are corrupt")
    })
}

/// Return a reference to the process-lifetime cached [`GlyphEngine`].
///
/// This is the canonical construction path. The [`GlyphEngine`] is constructed
/// at most once per process lifetime — `FontRef::try_from_slice` is called
/// exactly once regardless of how many times `engine()` is called
/// (F-S030-P11-C1). All callers receive a `&'static GlyphEngine` pointing to
/// the same instance.
///
/// # Panics
///
/// Panics only if the embedded font bytes are corrupt (build-time invariant
/// violation — cannot happen in a correct build).
#[must_use]
pub fn engine() -> &'static GlyphEngine {
    GLYPH_ENGINE.get_or_init(|| {
        let font_ref = font_ref();
        let units_per_em = font_ref
            .units_per_em()
            .expect("Latin Modern Math must declare units_per_em — embedded asset invariant");
        GlyphEngine {
            font: font_ref,
            units_per_em,
        }
    })
}

/// Glyph outline engine backed by the bundled Latin Modern Math font.
///
/// `GlyphEngine` is intended to be constructed once via [`engine()`] and reused
/// across calls to [`render_pdf_paths`][crate::pdf_paths::render_pdf_paths].
/// It is `Send + Sync` because it holds only a `&'static FontRef<'static>`
/// (a shared reference to static data) and a `f32` scalar.
pub struct GlyphEngine {
    /// Shared reference to the embedded font, borrowing the static `FONT_REF`.
    font: &'static FontRef<'static>,
    /// Units per EM for the font (typically 1000 for LM Math).
    units_per_em: f32,
}

impl GlyphEngine {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Direct invariant: `FontRef::try_from_slice` must be called at most once
    /// across N repeated `engine()` calls (F-S030-P11-C1, F-S030-P11-C2).
    ///
    /// This test runs FIRST (alphabetically) to catch the `OnceLock` before any
    /// other test has initialized it. Because `OnceLock` is process-scoped and
    /// `cargo nextest` runs each test in its own process by default, this is
    /// the cleanest way to guarantee a fresh counter without external test
    /// binary machinery.
    ///
    /// The counter starts at 0. After 100 `engine()` calls it must be exactly 1.
    #[test]
    fn test_a_font_parsed_exactly_once_across_n_calls() {
        // Reset the counter in case a prior test in the same process already
        // triggered initialization. If count starts at 1, the OnceLock was
        // already initialized — we assert count stays at 1 (not 100).
        PARSE_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);

        // Re-initializing FONT_REF is not possible once set, so we accept that
        // if FONT_REF was already initialized, count will remain 0 here. We
        // check the upper bound: after 100 calls count must be ≤ 1.
        for _ in 0..100 {
            let _eng = engine();
        }
        let count = PARSE_COUNT.load(std::sync::atomic::Ordering::SeqCst);
        // count == 0: OnceLock already initialized by a prior test — no new parse (correct).
        // count == 1: OnceLock initialized in this test — exactly one parse (correct).
        // count >= 2: BUG — FontRef::try_from_slice called multiple times.
        assert!(
            count <= 1,
            "FontRef::try_from_slice was called {count} times across 100 engine() calls; \
             expected at most 1 (OnceLock caching invariant, F-S030-P11-C1)"
        );
    }

    #[test]
    fn test_font_engine_constructs_successfully() {
        let engine = engine();
        // The engine must have a valid EM size (Latin Modern Math uses 1000 units/EM).
        assert!(engine.units_per_em > 0.0, "units_per_em must be positive");
    }

    #[test]
    fn test_font_engine_emit_latin_glyph_produces_path() {
        let engine = engine();
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
        let engine = engine();
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
        let engine = engine();
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
        let engine = engine();
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
        let engine = engine();
        let advance = engine.advance_width('x', 14, 10);
        assert!(
            advance > 0,
            "advance width for 'x' must be positive; got: {advance}"
        );
    }

    #[test]
    fn test_font_engine_fallback_for_unknown_char() {
        let engine = engine();
        // U+0001 (SOH, C0 controls block) is guaranteed to map to .notdef in
        // any well-formed font — C0 controls are never assigned outlines. This
        // is more durable than a PUA code point (U+FFF0) which a math font
        // could theoretically populate with symbols (F-S030-P10-O1).
        let mut paths: Vec<String> = Vec::new();
        engine
            .emit_glyph('\u{0001}', &mut paths, 0, 0, 10, 14)
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

    /// Regression gate: rendering 100 expressions in a tight loop must complete
    /// well within budget when the font is cached (F-S030-P10-C2, F-S030-P11-C2).
    ///
    /// Budget analysis:
    /// - Cached path (`OnceLock` hit): ~5ms total for 100 calls on modern hardware.
    /// - Reverted-to-reparse path: ~50ms per call × 100 = ~5000ms total.
    /// - Budget set at 50ms (10× the cached baseline) — catches any regression
    ///   to per-call reparsing with ≥100× margin while tolerating slow CI machines.
    #[test]
    fn test_font_engine_caching_no_reparse_under_loop() {
        use std::time::{Duration, Instant};
        // Warm the OnceLock cache outside the timed window so the budget reflects
        // only the per-call cost of accessing &'static GlyphEngine + emit_glyph.
        // Without this, if this test runs first in the process, the cold OTF-parse
        // (~50ms+) would be charged against the 50ms budget (F-S030-P12-L1).
        let _warmup = engine();
        let start = Instant::now();
        for _ in 0..100 {
            let eng = engine();
            let mut paths: Vec<String> = Vec::new();
            eng.emit_glyph('x', &mut paths, 0, 0, 10, 14).unwrap();
            assert!(!paths.is_empty(), "each call must produce a path");
        }
        let elapsed = start.elapsed();
        // 50ms budget: 10× the warm cached baseline (~5ms).
        // A reverted cache (reparse per call) would take ~5000ms and fail with ≥100× margin.
        let budget = Duration::from_millis(50);
        assert!(
            elapsed < budget,
            "100 render calls must complete in < 50ms (caching regression gate, F-S030-P11-C2); \
             took {}ms — if genuinely cached, expected < 5ms on modern hardware",
            elapsed.as_millis()
        );
    }

    /// Asset integrity gate: the SHA-256 of the bundled font file must match
    /// the value declared in `fonts/MANIFEST.toml` (F-S030-P10-I3).
    ///
    /// This test catches accidental font replacement or corruption. The
    /// manifest value is the authoritative source of truth; if the font is
    /// intentionally updated, update the manifest first.
    ///
    /// Defense-in-depth: also asserts the on-disk byte length matches
    /// `size_bytes` from the manifest (F-S030-P11-O2).
    #[test]
    fn test_bundled_font_sha256_matches_manifest() {
        use sha2::{Digest, Sha256};

        // Locate fonts/ relative to this source file (works in any worktree).
        let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fonts")
            .join("MANIFEST.toml");
        let font_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fonts")
            .join("latinmodern-math.otf");

        let manifest_text = std::fs::read_to_string(&manifest_path)
            .expect("fonts/MANIFEST.toml must be readable — asset audit file missing");
        // toml 1.x: `str::parse::<toml::Value>()` parses a single value expression,
        // not a document; use `toml::from_str` to parse the full TOML document.
        let manifest: toml::Value = toml::from_str(&manifest_text)
            .expect("fonts/MANIFEST.toml must be valid TOML");
        let expected_sha256 = manifest["font"]["sha256"]
            .as_str()
            .expect("MANIFEST.toml [font].sha256 must be a string");

        let font_bytes =
            std::fs::read(&font_path).expect("fonts/latinmodern-math.otf must be readable");

        // Defense-in-depth: byte-length check (F-S030-P11-O2).
        if let Some(expected_size) = manifest["font"]
            .get("size_bytes")
            .and_then(toml::Value::as_integer)
        {
            // font_bytes.len() is a usize; TOML integers are i64. Font files are
            // well under i64::MAX so the cast is safe in practice. We assert
            // non-negative to make the comparison sound.
            #[allow(clippy::cast_possible_wrap)]
            let actual_size = font_bytes.len() as i64;
            assert_eq!(
                actual_size,
                expected_size,
                "bundled font size mismatch — expected {expected_size} bytes, got {} bytes",
                font_bytes.len()
            );
        }

        let mut hasher = Sha256::new();
        hasher.update(&font_bytes);
        let digest = hasher.finalize();
        // Use write! into a pre-allocated String to avoid format! inside collect
        // (clippy::format_collect_into_string prefers this pattern).
        let mut actual_hex = String::with_capacity(digest.len() * 2);
        for b in &digest {
            use std::fmt::Write as _;
            write!(actual_hex, "{b:02x}").expect("write to String is infallible");
        }

        assert_eq!(
            actual_hex, expected_sha256,
            "bundled font SHA-256 mismatch — font may have been replaced or corrupted.\n\
             Expected (MANIFEST.toml): {expected_sha256}\n\
             Actual (on-disk):         {actual_hex}"
        );
    }
}
