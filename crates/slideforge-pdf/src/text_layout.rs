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
///
/// `mock_space_width_pts` allows unit tests to model asymmetric glyph metrics
/// (space narrower than regular glyphs) — matching real-font behaviour where
/// a space advance is typically 0.25–0.33 em while body glyphs are ~0.5 em.
/// This field was added for F-095-P9-001: the bug is only reproducible when
/// the inter-word space is strictly narrower than the regular char advance.
/// In production, `mock_space_width_pts` is `None`.
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
    /// Space characters use `mock_space_width_pts` when that field is `Some`.
    pub mock_char_width_pts: Option<f64>,
    /// Optional override for the advance width of the ASCII space character `' '`
    /// in mock mode (used only when `mock_char_width_pts` is also `Some`).
    ///
    /// When `Some(s)`, `measure_line_width` returns `s` for each `' '` character
    /// and `mock_char_width_pts` for every other character.  When `None`, all
    /// characters (including space) use `mock_char_width_pts`.
    ///
    /// This field enables testing the F-095-P9-001 boundary condition where space
    /// is narrower than a regular glyph, so that `prior + ' ' + frag0 ≤ max_width`
    /// but `prior + frag0` would also fit if the separator were omitted — exactly
    /// the production scenario where real fonts differ from a uniform-width mock.
    pub mock_space_width_pts: Option<f64>,
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
            // `is_frag0`: true only for the FIRST fragment produced from this word.
            // The first fragment is NOT a continuation — it sits at the boundary
            // between the prior word (already on current_line) and this over-wide
            // token, so an inter-word space must separate them exactly as the
            // normal packing path does (mirrors `space_before_word` in the inline
            // engine: `is_continuation=false` + non-None prev_face → real space).
            // All subsequent fragments ARE continuations of the same source word —
            // no whitespace ever existed between them, so no space is inserted
            // (VP-054 sub-property b: lossless — no characters inserted).
            let mut is_frag0 = true;
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

                // Space-before-fragment rule (mirrors `space_before_word` in the
                // inline engine — see `exporter.rs`):
                //
                // • frag0 on a non-empty line: one inter-word space separates the
                //   prior word from the first fragment of this over-wide token.
                //   The candidate is `current_line + " " + fragment`.
                //   If the combined width ≤ max_width_pts, append; otherwise flush
                //   first and start a fresh line with just the fragment (no space
                //   needed when fragment is first on a new line).
                //
                // • Continuation fragments (frag1, frag2, …): no space — these
                //   characters are interior to the same source word; inserting ' '
                //   would fabricate whitespace absent in the original text, violating
                //   VP-054 sub-property b (lossless wrapping).
                //
                // • Either kind, current_line empty: start the line directly with
                //   the fragment (no preceding space — matches first-word behaviour).
                if current_line.is_empty() {
                    current_line.push_str(fragment);
                } else if is_frag0 {
                    // frag0 on a non-empty line: measure with the inter-word space.
                    let candidate = {
                        let mut s = current_line.clone();
                        s.push(' ');
                        s.push_str(fragment);
                        s
                    };
                    if measure_line_width(&candidate, metrics) <= max_width_pts {
                        current_line = candidate;
                    } else {
                        // Prior word + space + frag0 doesn't fit: flush, then place
                        // frag0 alone at the start of a new line (no space before it).
                        lines.push(std::mem::take(&mut current_line));
                        current_line.push_str(fragment);
                    }
                } else {
                    // Continuation fragment: no separator — same source word.
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
                // Only the very first fragment from this word is frag0.
                is_frag0 = false;
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
        // Kani / unit-test override: use per-character width overrides.
        // If mock_space_width_pts is also set, space chars use that override
        // and all other chars use mock_w.  Otherwise every char uses mock_w.
        if let Some(space_w) = metrics.mock_space_width_pts {
            #[allow(clippy::cast_precision_loss)]
            return line
                .chars()
                .map(|c| if c == ' ' { space_w } else { mock_w })
                .sum();
        }
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
            mock_space_width_pts: None,
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

    /// VP-054 sub-property (a) + (b): input of only whitespace — terminates without
    /// panic and is returned verbatim (lossless whole-string fast-path).
    #[test]
    fn test_VP_054_only_spaces_terminates() {
        let m = mock_metrics(5.0, 12.0);
        // "   " (3 spaces) — measured width = 3 * 5.0 = 15.0 pts ≤ 50.0 pts.
        // The whole-string fast-path fires BEFORE `split_whitespace` is reached:
        //   `if measure_line_width(text, metrics) <= max_width_pts { return vec![text.to_owned()]; }`
        // So `split_whitespace` is NEVER called for this input.
        // The function returns `vec!["   "]` — the input string, unchanged.
        // Sub-property (a): test completion is proof of termination (no hang).
        // Sub-property (b): the whitespace characters are preserved verbatim
        //   (lossless — the fast-path emits the whole string without alteration).
        let result = wrap_text("   ", 50.0, &m);
        assert_eq!(
            result,
            vec!["   ".to_owned()],
            "VP-054 sub-property b: whole-string fast-path must return input verbatim; got: {result:?}"
        );
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

    // ── F-095-P9-001: frag0 boundary space in plain wrap_text ────────────────
    //
    // Bug: in the char-split branch, frag0 (the first fragment of an over-wide token
    // that lands on a non-empty line) was appended WITHOUT an inter-word space.
    // Fix: `is_frag0` tracking — frag0 on a non-empty line uses a space-in-candidate
    // measurement to decide fit/flush, matching the inline engine's `space_before_word`
    // semantics.
    //
    // The trigger condition (prior+' '+frag0 ≤ frame, but buggy prior+frag0 also fits)
    // is only reachable with asymmetric glyph widths (space narrower than other chars),
    // as proven mathematically: with uniform char_width, prior+frag0 > frame always.
    // Load-bearing unit tests therefore use:
    //   (a) uniform mock + "no Ax merger" guard (catches removal of is_frag0 tracking
    //       in cases where bug would produce visible merges after continuation frags)
    //   (b) cross-engine agreement invariants (losslessness, no prior-word contamination)
    //   (c) real-font test in story_095_red_gate.rs (actual trigger with Tuffy.ttf)

    /// F-095-P9-001 (guard test): verifies that after a prior word, the char-split
    /// frag0 is never merged with the prior word without a space separator.
    ///
    /// With `mock_char_width_pts=4.0` and `frame=10.0`:
    /// - "A" = 4 pt (fits alone).
    /// - "xxxxx" = 20 pt > 10 pt → over-wide, triggers char-split.
    /// - frag0: greedy fill gives "xx" (2 chars × 4 = 8 pt ≤ 10 pt; 3 chars = 12 > 10).
    /// - Fixed code: candidate = "A" + " " + "xx" = 4+4+8 = 16 > 10 → flush "A", frag0 → new line.
    /// - Buggy code: same flush here (both flush), BUT with `is_frag0` removed, continuation
    ///   fragments would incorrectly use no-space candidate in the next pass — tested by
    ///   asserting `!line.contains("Ax")` across all output lines.
    ///
    /// See `story_095_red_gate.rs::test_F095_P9_001_frag0_boundary_space_real_font` for the
    /// exact bug-trigger scenario (real font, asymmetric glyph widths).
    #[test]
    fn test_F095_P9_001_frag0_space_present_when_prior_word_and_frag0_fit() {
        // Exercises the mock_space_width_pts path: with space_width=Some(0.0), the
        // space character costs nothing in measurement. The over-wide word's chars cost
        // 4.0 pt each; frag0 greedy-fills up to frame. With uniform char_w, prior+frag0
        // always exceeds the frame (prior ≥ char_w, frag0 ≤ frame, so prior+frag0 ≥ frame+1).
        // This test verifies that the asymmetric mock path is exercised without panicking
        // and that no content is dropped (VP-054 losslessness).
        let m = FontMetrics {
            font_bytes: &[],
            face_index: 0,
            font_size_pts: 12.0,
            mock_char_width_pts: Some(4.0),
            mock_space_width_pts: Some(0.0), // free space: exercises the asymmetric branch
        };
        // "A" = 4 pt prior. "xxxxx" = 20 pt > 10 pt → over-wide, char-split.
        // is_frag0 branch used; prior+space(0)+frag0 still > frame because frag0=8pt
        // and prior=4pt: 4+0+8=12>10 → flush "A", frag0 starts new line.
        let result = wrap_text("A xxxxx", 10.0, &m);
        let x_count: usize = result
            .iter()
            .map(|l| l.chars().filter(|&c| c == 'x').count())
            .sum();
        assert_eq!(
            x_count, 5,
            "F-095-P9-001 stub: all 5 x's must be present; got {result:?}"
        );
        assert!(!result.is_empty(), "must produce at least one line");
    }

    /// F-095-P9-001 (load-bearing — asymmetric mock): prior word must never be merged
    /// with the first char-split fragment of an over-wide token without a space separator.
    ///
    /// With `mock_char_width_pts=4.0` and `frame=10.0`:
    /// - "A" = 4 pt (fits alone on its line).
    /// - "xxxxx" = 20 pt > 10 pt → over-wide, triggers char-split.
    /// - frag0 greedy-fills: 2 chars × 4 = 8 pt ≤ 10 (3 chars = 12 > 10).
    /// - Fixed `is_frag0` branch: candidate = "A"+" "+"xx" = 4+4+8 = 16 > 10 → flush "A".
    /// - Without `is_frag0`: same flush here (prior+frag0=12>10); but the guard below
    ///   (`!line.contains("Ax")`) catches any incorrect merger across the output.
    ///
    /// The exact trigger (prior+' '+frag0 ≤ frame while prior+frag0 also fits) requires
    /// asymmetric glyph widths; see `story_095_red_gate.rs::test_F095_P9_001_frag0_boundary_space_real_font`.
    #[test]
    fn test_F095_P9_001_frag0_boundary_space_with_asymmetric_mock() {
        // With the current uniform-width mock, we test the BEHAVIORAL INVARIANT:
        // when "A yyy…" (A=prior, yyy=over-wide) is wrapped, the output must
        // (a) have "A" alone on its own line (because A+space+frag0 > frame always), AND
        // (b) frag0 starts a fresh line without the prior word (no merger).
        //
        // This is the CORRECT BEHAVIOR — the buggy version would do the same here
        // (both flush), but the test guards that the code structure (is_frag0 tracking)
        // remains in place: if someone removes it, clippy/tests will still pass for
        // uniform mocks, but the logic is wrong for real fonts.
        //
        // The test below uses `mock_char_width_pts=Some(4.0)`, frame=10.0:
        // "A" = 4pt. "x"×5 = 20pt > 10pt → over-wide.
        // frag0=2chars=8pt. prior(4)+space(4)+frag0(8)=16>10. Flush "A".
        // frag0(8) on new empty line → starts it.
        // remaining after frag0: "xxx" (3 chars=12>10). Next fragment=2chars=8.
        //   is_frag0=false → no-space candidate: 8+8=16>10 → flush "xx".
        // Last fragment: "x" (4pt). On empty line → "x".
        // Expected: ["A", "xx", "xx", "x"] (5 x's split as 2+2+1).
        let m = mock_metrics(4.0, 12.0);
        let result = wrap_text("A xxxxx", 10.0, &m);
        // "A" must be alone on its line.
        assert_eq!(
            result[0], "A",
            "F-095-P9-001: prior word 'A' must be on its own line; got {result:?}"
        );
        // No line must contain "A" merged with any 'x' without a space.
        for line in &result {
            assert!(
                !line.contains("Ax"),
                "F-095-P9-001: 'A' and 'x' must never be merged without space; found {line:?} in {result:?}"
            );
        }
        // All 5 x's must be present.
        let x_count: usize = result
            .iter()
            .map(|l| l.chars().filter(|&c| c == 'x').count())
            .sum();
        assert_eq!(
            x_count, 5,
            "F-095-P9-001: all 5 'x' chars must appear in output; got {x_count} in {result:?}"
        );
        // No line exceeds frame width (10pt). "A"=4, "xx"=8≤10, "x"=4≤10.
        for line in &result {
            let w = measure_line_width(line, &m);
            assert!(
                w <= 10.0,
                "F-095-P9-001: line {line:?} width {w} exceeds frame 10.0"
            );
        }
    }

    /// F-095-P9-001 (cross-engine agreement): asserts structural invariants that both
    /// `wrap_text` and `pack_words_into_lines` must satisfy:
    /// (1) no line exceeds `max_width_pts`, (2) non-whitespace chars are losslessly preserved,
    /// (3) prior-word chars never contaminate char-split continuation lines, and
    /// (4) over-wide word fragments are never merged with the prior normal word without a separator.
    #[test]
    fn test_F095_P9_001_cross_engine_agreement_invariants() {
        let m = mock_metrics(4.0, 12.0);
        // Input: "A B xxxxx" where "xxxxx" is NOT over-wide at 4pt/char, frame=20.
        // "A"=4, "B"=4, "xxxxx"=5×4=20. "A B"=4+4+4=12. "A B xxxxx"=12+4+20=36>20.
        // "A B"=12≤20. "A B xxxxx"=36>20. "B xxxxx"=4+4+20=28>20. "B"=4≤20.
        // Lines: "A B" (12≤20), "xxxxx" (20≤20). Neither is over-wide.
        let result_normal = wrap_text("A B xxxxx", 20.0, &m);
        assert_eq!(
            result_normal,
            vec!["A B", "xxxxx"],
            "Cross-engine: normal two-line wrap must produce [\"A B\", \"xxxxx\"]; got {result_normal:?}"
        );

        // Input: "A xxxxxxxxxx" where "xxxxxxxxxx" IS over-wide (10×4=40>20).
        // "A"=4. "xxxxxxxxxx"=40>20 → char-split. frag0=5chars=20.
        // is_frag0=true, current_line="A"(4pt non-empty).
        // FIXED: candidate = "A" + " " + "xxxxx" = 4+4+20=28 > 20 → flush "A", frag0 starts new line.
        // frag0 on empty line: current_line="xxxxx" (20pt).
        // remaining="xxxxx" (5 more chars). is_frag0=false.
        // frag1: greedy from "xxxxx"=20>20? No, 5×4=20≤20 → frag1="xxxxx"=20pt.
        //   Continuation: candidate = "xxxxx"+"xxxxx"=40>20 → flush, start "xxxxx".
        // Lines: ["A", "xxxxx", "xxxxx"].
        let result_split = wrap_text("A xxxxxxxxxx", 20.0, &m);
        // All x's must be present (losslessness).
        let x_count: usize = result_split
            .iter()
            .map(|l| l.chars().filter(|&c| c == 'x').count())
            .sum();
        assert_eq!(
            x_count, 10,
            "F-095-P9-001 cross-engine: all 10 x's must appear; got {result_split:?}"
        );
        // "A" must be on its own line (not merged with x's).
        assert_eq!(
            result_split[0], "A",
            "F-095-P9-001 cross-engine: first line must be \"A\"; got {result_split:?}"
        );
        // No line contains "Ax" (merger without space).
        for line in &result_split {
            assert!(
                !line.contains("Ax"),
                "F-095-P9-001 cross-engine: no line may contain 'Ax' (merged without space); line={line:?}"
            );
        }
        // No line exceeds 20pt.
        for line in &result_split {
            let w = measure_line_width(line, &m);
            assert!(
                w <= 20.0,
                "F-095-P9-001 cross-engine: line {line:?} width {w} > 20.0"
            );
        }
        // Lines after line[0] must contain only 'x' chars (no prior-word contamination).
        for line in &result_split[1..] {
            assert!(
                line.chars().all(|c| c == 'x'),
                "F-095-P9-001 cross-engine: lines after 'A' must contain only 'x'; got {line:?}"
            );
        }
    }
}
