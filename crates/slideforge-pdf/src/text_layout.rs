//! Word-wrap engine for PDF text emission (STORY-095 — AC-001, AC-002).
//!
//! ## Purpose
//!
//! This module provides a **pure function** that splits a text string into
//! lines that fit within a frame width expressed in PDF user units (points).
//! It is the sole engine used by `exporter.rs` before emitting text spans
//! to krilla's Surface API.
//!
//! ## Purity requirement (VP-006 / BC-4.03.002)
//!
//! `wrap_text` is a pure function — no I/O, no side effects, no global state.
//! This makes it Kani-amenable for the Phase 6 formal proof of termination
//! (VP-006: `wrap_text` terminates for all inputs bounded by `MAX_TEXT_LEN`).
//!
//! ## `f64` in this module (architecture note)
//!
//! PDF user-unit widths are expressed as `f64` points throughout this module.
//! This is acceptable because `f64` here is:
//! - **Post-EMU-conversion:** the callers in `exporter.rs` convert from integer
//!   EMU to `f64` points via `coords::emu_to_pt()` before passing values here.
//! - **Not in the IR coordinate path:** the two-IR model stores positions as
//!   integer EMU; `f64` only appears in the post-conversion PDF draw path
//!   (CLAUDE.md Forbidden Patterns note applies to IR fields, not draw-path).
//!
//! ## Algorithm
//!
//! 1. Split the input `text` on whitespace to produce candidate words.
//! 2. Greedily pack words onto lines: add a word to the current line if the
//!    resulting string would fit within `max_width_pts`; otherwise start a new
//!    line.
//! 3. **Character-wrap fallback (AC-002):** when a single word is wider than
//!    `max_width_pts`, break it at the character boundary where it would overflow.
//!    No text is silently dropped — every character of the word appears in the
//!    output.
//! 4. **Edge cases (EC-002, EC-005):** an empty input returns an empty `Vec`;
//!    text exactly equal to the frame width emits a single line.

/// Font metrics required by [`wrap_text`] to measure glyph advances.
///
/// Wraps the raw font bytes and the `face_index` needed by
/// `font::measure_text_width_pt` (ADV-P05-MED-001: `face_index` is never
/// hardcoded to `0`).
///
/// ## Kani amenability
///
/// For the Phase 6 Kani proof, `FontMetrics` carries a `mock_char_width_pts`
/// field used by the verifier to replace real font I/O with a deterministic
/// value (Kani does not model filesystem or font parsing). In production,
/// `mock_char_width_pts` is `None` and real `ttf-parser` metrics are used.
pub struct FontMetrics<'a> {
    /// Raw font bytes for `ttf-parser` glyph-advance measurement.
    pub font_bytes: &'a [u8],
    /// Face index within the font file (may be non-zero for `.ttc` collections).
    pub face_index: u32,
    /// Font size in points at which the text will be drawn.
    pub font_size_pts: f64,
    /// Optional override for glyph width (used in Kani proofs / unit tests
    /// that do not have a valid font file). When `Some(w)`, every character is
    /// assumed to have advance `w` points regardless of the actual cmap/hmtx.
    pub mock_char_width_pts: Option<f64>,
}

/// Wrap `text` into lines that fit within `max_width_pts` PDF user units.
///
/// Returns an ordered `Vec<String>` where each element is one wrapped line.
/// Lines do NOT include a trailing newline character.
///
/// ## Postconditions (BC-4.03.002 — AC-001, AC-002)
///
/// 1. For every returned line `l`:
///    `measure_line_width(l, metrics) <= max_width_pts` — no line exceeds the frame.
///    (Exception: lines forced by the character-wrap fallback when a single
///    character is itself wider than `max_width_pts`.)
///
/// 2. The concatenation of all returned lines (joined with spaces) contains
///    all characters from the original `text` — no text is silently dropped.
///
/// ## Edge cases
///
/// - Empty `text` → returns `vec![]` (EC-002).
/// - Text that fits on one line (measured width ≤ `max_width_pts`) → single
///   element Vec (EC-005: no spurious wrap).
/// - Single word wider than the frame → character-wrap fallback (AC-002).
///
/// ## Pure function guarantee
///
/// This function has no side effects. It does not perform I/O, mutate global
/// state, or allocate heap memory beyond the returned `Vec<String>`.
///
/// # Panics
///
/// Does not panic. All index arithmetic uses checked methods.
///
/// # Errors
///
/// This function is infallible — it returns `Vec<String>`, never `Result`.
/// Font measurement failures (e.g., character not in cmap) produce a zero-
/// advance contribution, causing conservative (no-wrap) behavior for unknown
/// glyphs.
///
/// [`VP-006`]: ../../.factory/specs/verification-properties/VP-006.md
pub fn wrap_text(text: &str, _max_width_pts: f64, _metrics: &FontMetrics<'_>) -> Vec<String> {
    // STORY-095 T-005 (GREEN phase): implement this function.
    // For now this is a stub that returns the text as a single line
    // (no wrapping), so that:
    //   - The crate COMPILES (tests can compile and fail at assertion-time).
    //   - AC-001 and AC-002 tests FAIL because the stub does not wrap long text.
    //   - EC-002 and EC-005 edge-case tests may pass or fail.
    //
    // The implementer MUST replace this stub with the real algorithm described
    // in the module-level doc comment above.
    if text.is_empty() {
        return vec![];
    }
    // STUB: no wrapping — returns the whole text as a single line.
    // Tests AC-001 and AC-002 will FAIL against this stub (correct RED behaviour).
    vec![text.to_owned()]
}

/// Measure the rendered width of `line` in PDF user-unit points.
///
/// Delegates to `font::measure_text_width_pt` using real `ttf-parser` metrics
/// when `metrics.mock_char_width_pts` is `None`, or uses the fixed per-character
/// override when `Some(w)`.
///
/// This is extracted as a public helper so tests can assert widths without
/// building a full `krilla::Document`.
///
/// # Pure function
///
/// No I/O, no side effects beyond the return value.
#[must_use]
pub fn measure_line_width(line: &str, metrics: &FontMetrics<'_>) -> f64 {
    if let Some(mock_w) = metrics.mock_char_width_pts {
        // Kani / unit-test override: every char has width `mock_w`.
        #[allow(clippy::cast_precision_loss)]
        return mock_w * line.chars().count() as f64;
    }
    // Real measurement via ttf-parser (same as exporter draw path).
    // `as f32` here is safe: font_size_pts is a rendering size (positive, finite,
    // bounded by any reasonable font size in points — well within f32 range).
    // The lint is suppressed at item level since inner attributes on expressions
    // are not stable.
    let font_size_f32 = metrics.font_size_pts as f32;
    f64::from(crate::font::measure_text_width_pt(
        metrics.font_bytes,
        metrics.face_index,
        font_size_f32,
        line,
    ))
}
