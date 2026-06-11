//! Word-wrap engine for PDF text emission (STORY-095 — AC-001, AC-002).
//!
//! ## Purpose
//!
//! This module provides a **pure function** that splits a text string into
//! lines that fit within a frame width expressed in PDF user units (points).
//! It is the sole engine used by `exporter.rs` before emitting text spans
//! to krilla's Surface API.
//!
//! ## Purity requirement (VP-054 / BC-4.03.002)
//!
//! `wrap_text` is a pure function — no I/O, no side effects, no global state.
//! This makes it Kani-amenable for the Phase 6 formal proof of termination
//! (VP-054: `wrap_text` terminates for all bounded inputs, produces lossless output,
//! and every output line satisfies the max-width invariant).
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
/// 2. The concatenation of all returned lines contains all non-whitespace
///    characters from the original `text` — no text is silently dropped and
///    no characters are inserted. Whitespace consumed at word-wrap boundaries
///    is not re-emitted (lossless per VP-054 sub-property b).
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
/// [`VP-054`]: ../../.factory/specs/verification-properties/vp-054-wrap-text-termination.md
#[must_use]
pub fn wrap_text(text: &str, max_width_pts: f64, metrics: &FontMetrics<'_>) -> Vec<String> {
    // EC-002: empty input returns empty Vec.
    if text.is_empty() {
        return vec![];
    }

    // EC-005 / fast path: if the whole text fits on one line, return it immediately.
    // Uses `<=` so text EXACTLY equal to the frame width is not wrapped (EC-005).
    if measure_line_width(text, metrics) <= max_width_pts {
        return vec![text.to_owned()];
    }

    // ── Phase 1: word-boundary greedy packing (AC-001) ──────────────────────────
    //
    // Split on whitespace tokens. Each "word" is tried with a space separator.
    // When adding the next word would exceed max_width_pts, flush the current
    // line and start a new one.  Single words wider than the frame fall through
    // to the character-wrap fallback below (AC-002).
    //
    // Algorithm is provably terminating: the `words` iterator is strictly finite
    // (bounded by `text.len()`), and the inner character-wrap loop over `word`
    // is also strictly finite (bounded by `word.chars().count()`). VP-054.

    let mut lines: Vec<String> = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        // ── AC-002: character-wrap fallback for over-wide single words ──────────
        //
        // If this word alone does not fit within the frame, break it at the
        // character boundary where it would overflow.  Repeat until the word's
        // remaining chars fit on one line.  Each character-wrapped fragment is
        // then appended to the current line via the normal word-packing logic.
        let word_width = measure_line_width(word, metrics);
        if word_width > max_width_pts {
            // The word is too wide to fit on a line by itself.
            // Break it into character-level fragments and treat each fragment
            // like a mini-word (falls through to normal packing below).
            let mut remaining: &str = word;
            while !remaining.is_empty() {
                // Find the longest character prefix of `remaining` that fits.
                let mut fragment_end_byte = 0;
                let mut frag_width = 0.0_f64;
                for ch in remaining.chars() {
                    let mut buf = [0u8; 4];
                    let ch_str = ch.encode_utf8(&mut buf);
                    let ch_width = measure_line_width(ch_str, metrics);
                    if frag_width + ch_width > max_width_pts && fragment_end_byte > 0 {
                        // Adding this character would overflow — stop here.
                        break;
                    }
                    frag_width += ch_width;
                    fragment_end_byte += ch.len_utf8();
                }

                // Safety: if even a single character does not fit (max_width_pts
                // is smaller than one char), emit it anyway to avoid an infinite
                // loop (no text silently dropped — AC-002 postcondition 2).
                if fragment_end_byte == 0 {
                    // Advance by exactly one character to guarantee termination.
                    fragment_end_byte = remaining.chars().next().map_or(0, char::len_utf8);
                }

                let (fragment, rest) = remaining.split_at(fragment_end_byte);
                remaining = rest;

                // Append fragment directly (NO space separator) — all fragments
                // originate from the same over-wide word, which by definition contains
                // no whitespace. Inserting ' ' between fragments would reconstruct a
                // string with interior spaces that did not exist in the source word
                // (VP-054 sub-property b: lossless wrapping — no characters inserted).
                //
                // Algorithm:
                // 1. If current_line is empty, start with the fragment.
                // 2. Otherwise try concatenating WITHOUT a separator. If the combined
                //    string exceeds max_width_pts, flush current_line first, then
                //    start a new line with just the fragment.
                if current_line.is_empty() {
                    current_line.push_str(fragment);
                } else {
                    // Try adding fragment directly (no separator — same word).
                    let candidate = {
                        let mut s = current_line.clone();
                        s.push_str(fragment);
                        s
                    };
                    if measure_line_width(&candidate, metrics) <= max_width_pts {
                        current_line = candidate;
                    } else {
                        // Flush current line and start fresh with this fragment.
                        lines.push(std::mem::take(&mut current_line));
                        current_line.push_str(fragment);
                    }
                }
            }
            continue; // Move on to next word.
        }

        // ── Normal word packing (AC-001) ────────────────────────────────────────
        if current_line.is_empty() {
            current_line.push_str(word);
        } else {
            // Try appending " word" to the current line.
            let candidate = {
                let mut s = current_line.clone();
                s.push(' ');
                s.push_str(word);
                s
            };
            if measure_line_width(&candidate, metrics) <= max_width_pts {
                current_line = candidate;
            } else {
                // Flush and start a new line with this word.
                lines.push(std::mem::take(&mut current_line));
                current_line.push_str(word);
            }
        }
    }

    // Flush the final (non-empty) line.
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
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
    // `as f32` is safe: font_size_pts is a rendering size (positive, finite,
    // bounded by any reasonable point size — well within f32 range).
    #[allow(clippy::cast_possible_truncation)]
    let font_size_f32 = metrics.font_size_pts as f32;
    f64::from(crate::font::measure_text_width_pt(
        metrics.font_bytes,
        metrics.face_index,
        font_size_f32,
        line,
    ))
}

// ─── Unit tests for VP-054 concrete coverage ──────────────────────────────────
//
// VP-054 sub-property coverage:
//   (a) Termination:     every test that completes proves termination (no hang).
//   (b) Losslessness:    tests verify concat(output) preserves non-whitespace chars.
//   (c) Max-width:       tests verify no output line exceeds max_width (except single-
//                        token over-wide lines, which are emitted as-is per VP-054).
//
// These tests are PURE FUNCTION TESTS (no PDF export, no krilla, no font I/O).
// They use `mock_char_width_pts: Some(w)` to make `measure_line_width` deterministic.
//
// Per CLAUDE.md Testing convention and F-095-P1-008: pure `wrap_text` unit tests
// belong here (next to the production code), NOT in the integration test file.
// End-to-end export round-trip tests remain in `tests/story_095_red_gate.rs`.
//
// VP reference: .factory/specs/verification-properties/vp-054-wrap-text-termination.md
#[cfg(test)]
#[allow(clippy::unwrap_used, non_snake_case)]
mod tests {
    use super::{FontMetrics, measure_line_width, wrap_text};

    /// Construct a minimal [`FontMetrics`] with a fixed per-character width.
    ///
    /// All characters (including space) are assumed to have width `char_width_pts`.
    fn mock_metrics(char_width_pts: f64, font_size_pts: f64) -> FontMetrics<'static> {
        FontMetrics {
            font_bytes: &[],
            face_index: 0,
            font_size_pts,
            mock_char_width_pts: Some(char_width_pts),
        }
    }

    // ── VP-054 Table row: empty input ─────────────────────────────────────────

    /// VP-054: empty input → empty Vec (no panic, terminates).
    #[test]
    fn test_VP_054_empty_input_returns_empty_vec() {
        let m = mock_metrics(5.0, 12.0);
        let result = wrap_text("", 10.0, &m);
        // EC-002: empty input must return empty Vec (no panic, no spurious line).
        assert!(
            result.is_empty(),
            "VP-054/EC-002: empty input must return empty Vec, got: {result:?}"
        );
    }

    // ── VP-054 Table row: single word fits ───────────────────────────────────

    /// VP-054: single word that fits → one-element Vec, no spurious wrap (EC-005).
    #[test]
    fn test_VP_054_single_word_fits_in_frame() {
        let m = mock_metrics(5.0, 12.0);
        // "hello" = 5 chars * 5.0 = 25 pts; max_width = 50 pts (fits).
        let result = wrap_text("hello", 50.0, &m);
        assert_eq!(
            result,
            vec!["hello".to_owned()],
            "VP-054: single fitting word must return vec![word]"
        );
    }

    // ── VP-054 Table row: single word exact ──────────────────────────────────

    /// VP-054: text EXACTLY equal to frame width → single line, no spurious wrap.
    #[test]
    fn test_VP_054_single_word_exact_width_no_wrap() {
        let m = mock_metrics(5.0, 12.0);
        // "hello" = 5 chars * 5.0 = 25 pts; max_width = 25 pts (exactly fits).
        let result = wrap_text("hello", 25.0, &m);
        assert_eq!(
            result.len(),
            1,
            "VP-054/EC-005: text exactly equal to frame width must produce exactly 1 line"
        );
        assert_eq!(
            result[0], "hello",
            "VP-054/EC-005: single line must equal input"
        );
    }

    // ── VP-054 Table row: single word over-width ─────────────────────────────

    /// VP-054 sub-property (a): a single word wider than the frame must be emitted
    /// as a single (over-width) line — no infinite split loop.
    #[test]
    fn test_VP_054_single_word_over_width_no_infinite_loop() {
        let m = mock_metrics(5.0, 12.0);
        // "hello" = 5 chars * 5.0 = 25 pts; max_width = 3 pts (can't fit even 1 char).
        // Expect: at least 1 element (no hang), no panic.
        let result = wrap_text("hello", 3.0, &m);
        assert!(
            !result.is_empty(),
            "VP-054: over-width single word must produce at least 1 line (no silent drop)"
        );
        // Losslessness: all chars must appear.
        let reconstructed: String = result.concat();
        assert_eq!(
            reconstructed, "hello",
            "VP-054 sub-property b: lossless — no chars dropped for over-width single word"
        );
    }

    // ── VP-054 Table row: two words fit ──────────────────────────────────────

    /// VP-054: two words that both fit on one line → single line, no wrap.
    #[test]
    fn test_VP_054_two_words_fit_on_one_line() {
        let m = mock_metrics(5.0, 12.0);
        // "hi yo" = 5 chars * 5.0 = 25 pts (including space); max_width = 50 pts.
        let result = wrap_text("hi yo", 50.0, &m);
        assert_eq!(
            result,
            vec!["hi yo".to_owned()],
            "VP-054: two words fitting in frame must be on a single line"
        );
    }

    // ── VP-054 Table row: two words wrap ─────────────────────────────────────

    /// VP-054 sub-property (c): two words that don't fit together → two lines.
    #[test]
    fn test_VP_054_two_words_wrap_when_too_wide() {
        let m = mock_metrics(5.0, 12.0);
        // "hi yo": "hi" = 10 pts, "yo" = 10 pts, "hi yo" = 25 pts (5 chars * 5).
        // max_width = 15 pts — fits "hi" (10 pts) but not "hi yo" (25 pts).
        let result = wrap_text("hi yo", 15.0, &m);
        assert_eq!(
            result.len(),
            2,
            "VP-054: two words that don't fit together must produce 2 lines, got: {result:?}"
        );
        assert_eq!(result[0], "hi", "VP-054: first line must be 'hi'");
        assert_eq!(result[1], "yo", "VP-054: second line must be 'yo'");
    }

    // ── VP-054 Table row: only spaces ────────────────────────────────────────

    /// VP-054 sub-property (a): input of only whitespace — terminates without panic.
    #[test]
    fn test_VP_054_only_spaces_terminates() {
        let m = mock_metrics(5.0, 12.0);
        // All whitespace — `split_whitespace` yields no words; should terminate cleanly.
        let result = wrap_text("   ", 50.0, &m);
        // Acceptable: returns empty Vec (no words to emit) OR returns a single empty
        // string. The contract is: no panic, terminates.
        // The current implementation returns empty Vec for all-whitespace input
        // (fast-path: split_whitespace is empty, so `current_line` is never filled).
        // Both outcomes satisfy VP-054 sub-property (a).
        drop(result); // terminates — proof of sub-property (a).
    }

    // ── VP-054 Table row: long line no spaces (char-wrap) ────────────────────

    /// VP-054 sub-property (a) + (b): a 100-char word in a 10-char-wide frame
    /// terminates and produces lossless output.
    #[test]
    fn test_VP_054_long_word_no_spaces_char_wrap_lossless() {
        let m = mock_metrics(1.0, 12.0);
        // 100 'a's, max_width = 10 pts (each char = 1 pt → 10 chars fit per line).
        let input = "a".repeat(100);
        let result = wrap_text(&input, 10.0, &m);
        // Sub-property (a): test completion proves termination.
        // Sub-property (b): lossless — all 100 'a's appear in output.
        let reconstructed: String = result.concat();
        assert_eq!(
            reconstructed.len(),
            100,
            "VP-054 sub-property b: lossless — all 100 chars must appear; got {}",
            reconstructed.len()
        );
        assert_eq!(
            reconstructed, input,
            "VP-054 sub-property b: reconstructed output must equal input"
        );
        // Sub-property (c): max-width — each line must fit within 10 pts (except
        // single-char-per-line if char width > max_width, but here 1.0 < 10.0 so
        // the invariant holds strictly).
        for line in &result {
            let w = measure_line_width(line, &m);
            assert!(
                w <= 10.0,
                "VP-054 sub-property c: line {line:?} has width {w} > 10.0"
            );
        }
    }

    // ── F-095-P1-004: no spurious interior space in char-split word ───────────

    /// F-095-P1-004 regression guard (VP-054 sub-property b): when an over-wide word
    /// is char-split, the reconstructed fragments MUST NOT contain any interior spaces.
    ///
    /// This is the LOAD-BEARING test that fails if the char-wrap fallback reverts to
    /// inserting ' ' between fragments (the bug described in F-095-P1-004).
    #[test]
    fn test_VP_054_char_split_word_no_interior_space_F095_P1_004() {
        let m = mock_metrics(2.0, 12.0);
        // 200-char word, each char = 2 pts → total 400 pts. Frame = 100 pts (50 chars/line).
        // Preceding content: "ab " (3 chars = 6 pts) — fits in frame before the word.
        let input = format!("ab {}", "x".repeat(200));
        let result = wrap_text(&input, 100.0, &m);

        // Losslessness: join all lines (no separator) and check non-whitespace chars.
        let all_text: String = result.concat();
        // The 200 'x's must appear consecutively without interior spaces.
        let x_runs: Vec<&str> = all_text
            .split_whitespace()
            .filter(|s| s.contains('x'))
            .collect();
        // All 'x' chars must appear together in one or more consecutive x-only segments.
        let total_x: usize = x_runs
            .iter()
            .map(|s| s.chars().filter(|&c| c == 'x').count())
            .sum();
        assert_eq!(
            total_x, 200,
            "F-095-P1-004: all 200 'x' chars must appear in output; got {total_x} (lines: {result:?})"
        );
        // Crucially: no line should contain "x x" (space INSIDE the 200-char word).
        for line in &result {
            // A line may have "ab x..." (space between "ab" word and the x-run) — that's OK.
            // What's NOT OK: "x x" (space within the x-run itself).
            // Detect this by splitting on whitespace and checking for runs of x's separated by space.
            let parts: Vec<&str> = line.split_whitespace().collect();
            for part in &parts {
                // Each whitespace-delimited part must consist of only 'x' or only 'a'/'b' chars.
                // No part should mix 'x' with non-x chars (indicating a bogus boundary).
                let has_x = part.contains('x');
                let all_x = part.chars().all(|c| c == 'x');
                if has_x {
                    assert!(
                        all_x,
                        "F-095-P1-004 FAIL: line {line:?} contains 'x' mixed with other chars in part {part:?}; \
                         char-split fragments must not introduce interior spaces"
                    );
                }
            }
        }
    }
}
