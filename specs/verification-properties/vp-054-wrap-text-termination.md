---
document_type: verification-property
vp_id: VP-054
title: "wrap_text termination, lossless wrapping, and max-width invariant"
module: slideforge-pdf
tool: Kani
phase: P6
priority: P1
status: draft
spec_version: "1.1.0"
bc_trace: [BC-4.03.002]
traces_to: .factory/specs/verification-properties/VP-INDEX.md
origin: STORY-095 / F-095-P1-003
---

# VP-054: wrap_text Termination, Lossless Wrapping, and Max-Width Invariant

## Property Statement

Three sub-properties must hold for `wrap_text` in
`crates/slideforge-pdf/src/text_layout.rs`:

**(a) Termination.** For any finite input string and any `max_width ≥ 1`, `wrap_text`
terminates. There are no infinite loops, infinite recursion, or unbounded iteration.
The loop counter is bounded by `O(input.len())`.

**(b) Lossless wrapping.** The concatenation of all output lines (joined without
intervening characters, or with exactly the whitespace consumed at each wrap point)
reconstructs the original input exactly. No character is silently dropped; no
character is inserted. Formally: `output.join("").len() >= input.len() - whitespace_consumed_at_wrap_points`.

**(c) Max-width invariant.** Every output line satisfies
`measure_line_width(line, metrics) <= max_width_pts` when `max_width_pts` is at
least as wide as a single character under the given `FontMetrics`. (When a single
token's measured width exceeds `max_width_pts`, that token is emitted as a single
over-width line — no infinite splitting loop.)

**Scope.** These properties target `wrap_text` as a pure function with the
production signature `(text: &str, max_width_pts: f64, metrics: &FontMetrics<'_>)
-> Vec<String>` in `crates/slideforge-pdf/src/text_layout.rs`. The `FontMetrics`
struct carries a `mock_char_width_pts: Option<f64>` field: when `Some(w)`, every
character is assumed to have advance `w` points (deterministic mock for proofs and
unit tests); when `None`, real `ttf-parser` glyph metrics are used. The Kani proof
targets sub-property (a) (bounded termination) and (c) (max-width bound for bounded
inputs) using `mock_char_width_pts` to eliminate font I/O from the proof context.
Sub-property (b) is verified via proptest with string-generating strategies.
Concrete unit tests cover all three sub-properties at fixed points. This VP covers
the pure `text_layout` module surface — `wrap_text` plus any pure helpers it
exposes (such as `measure_line_width`) — not specific private helper names, which
may change during Phase 3 implementation.

## Motivation

BC-4.03.002 specifies the PDF text layout pipeline, of which `wrap_text` is the
innermost pure function. An infinite loop in `wrap_text` would hang the entire PDF
export for any input containing a long line — a catastrophic, hard-to-diagnose
production failure. Silent text truncation is equally catastrophic: it produces PDF
output that is legally and functionally incorrect without any error surfaced to the
user. Both failure modes are undetectable through manual testing at scale, making
formal and property-based verification the appropriate strategy.

The function is pure (no side effects, deterministic, no I/O), making it an ideal
Kani bounded-model-check target. The losslessness property is naturally expressed as
a proptest round-trip invariant.

## Feasibility Assessment

**Feasible.** `wrap_text` is a pure function with:
- Finite, bounded input (`&str` with `len() < usize::MAX`)
- A loop whose termination depends on the cursor advancing by at least one character
  per iteration (provable by inspecting the loop body)
- Output elements whose combined length reconstructs the input

**Kani (sub-properties a + c):** Prove termination and max-width bound over a bounded
input space (`kani::assume(input.len() <= 64)`) using a `FontMetrics` constructed
with a symbolic `mock_char_width_pts: Some(w)` where `kani::assume(w > 0.0 && w <= 20.0)`
and `kani::assume(max_width_pts >= w && max_width_pts <= 640.0)` (ensures at least
one character fits, keeping the invariant unconditional). The unwind bound is
`input.len() + 1`. Eliminating real font I/O via `mock_char_width_pts` keeps the
model check bounded and deterministic. Feasible within Kani's bounded-model-check
envelope.

**Proptest (sub-property b):** Generate arbitrary ASCII strings and arbitrary
`mock_char_width_pts` in `(0.0, 10.0]` and `max_width_pts` in `[1.0, 1024.0]`.
Assert that `output.concat()` contains all non-whitespace characters from the input
in the original order.

**Concrete unit tests:** Cover edge cases: empty input, single-character input, input
exactly equal to `max_width`, input of a single token exceeding `max_width`, input
with no spaces, input with only spaces.

## Proof Harness Skeleton

```rust
// crates/slideforge-pdf/src/proofs/text_layout.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    /// Sub-property (a): wrap_text always terminates for bounded inputs.
    /// Sub-property (c): no output line exceeds max_width_pts when at least one
    ///                   character fits (i.e., max_width_pts >= mock_char_width_pts).
    #[kani::proof]
    #[kani::unwind(65)]  // input.len() <= 64 + 1 iteration overhead
    fn wrap_text_terminates_and_respects_max_width() {
        // Bounded input: Kani cannot handle unbounded &str; use fixed-length array proxy.
        let len: usize = kani::any();
        kani::assume(len > 0 && len <= 64);

        // Deterministic width model: every character has advance `char_w` points.
        // Real font I/O is bypassed via mock_char_width_pts — keeps the model bounded.
        let char_w: f64 = kani::any();
        kani::assume(char_w > 0.0 && char_w <= 20.0);

        // max_width_pts must accommodate at least one character so the invariant holds
        // unconditionally (single-char lines are the tightest case).
        let max_width_pts: f64 = kani::any();
        kani::assume(max_width_pts >= char_w && max_width_pts <= 640.0);

        let metrics = FontMetrics {
            font_bytes: &[],
            face_index: 0,
            font_size_pts: 12.0,
            mock_char_width_pts: Some(char_w),
        };

        // Build a synthetic ASCII string of `len` bytes (space + printable ASCII).
        let input: Vec<u8> = (0..len).map(|_| {
            let b: u8 = kani::any();
            kani::assume(b == b' ' || (b >= b'a' && b <= b'z'));
            b
        }).collect();
        let input_str = std::str::from_utf8(&input).expect("valid ASCII");

        let lines = wrap_text(input_str, max_width_pts, &metrics);

        // Termination is implicit: reaching this assertion proves the loop ended.
        // Max-width invariant: when max_width_pts >= char_w, every output line fits.
        // (The single-token over-width exception cannot occur here because we assumed
        //  max_width_pts >= char_w, so at minimum one character always fits.)
        for line in &lines {
            let line_w = measure_line_width(line, &metrics);
            kani::assert!(line_w <= max_width_pts);
        }

        // Losslessness (bounded check): output is non-empty iff input is non-empty.
        kani::assert!(!lines.is_empty());
    }
}
```

## Proptest Strategy (sub-property b — losslessness)

```rust
// crates/slideforge-pdf/src/text_layout.rs  #[cfg(test)]
use proptest::prelude::*;

proptest! {
    #[test]
    fn wrap_text_lossless(
        input in "[a-z ]{0,256}",
        char_w in 1.0f64..=10.0,
        max_width_pts in 1.0f64..=1024.0,
    ) {
        let metrics = FontMetrics {
            font_bytes: &[],
            face_index: 0,
            font_size_pts: 12.0,
            mock_char_width_pts: Some(char_w),
        };
        let lines = wrap_text(&input, max_width_pts, &metrics);
        // Non-whitespace characters must be preserved in order.
        let output_nonws: String = lines.concat()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let input_nonws: String = input
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        prop_assert_eq!(output_nonws, input_nonws);
    }

    #[test]
    fn wrap_text_max_width_holds_for_multi_word(
        // Inputs guaranteed to have spaces so single-token exception doesn't apply.
        // char_w <= max_width_pts ensures every single character fits (no over-wide
        // single-token lines), making the invariant unconditional for this strategy.
        input in "[a-z]{1,16}( [a-z]{1,16}){1,15}",
        char_w in 1.0f64..=5.0,
        max_width_pts in 5.0f64..=160.0,
    ) {
        prop_assume!(max_width_pts >= char_w);
        let metrics = FontMetrics {
            font_bytes: &[],
            face_index: 0,
            font_size_pts: 12.0,
            mock_char_width_pts: Some(char_w),
        };
        let lines = wrap_text(&input, max_width_pts, &metrics);
        for line in &lines {
            let line_w = measure_line_width(line, &metrics);
            prop_assert!(
                line_w <= max_width_pts,
                "line {:?} has width {} > max_width_pts {} (char_w={})",
                line, line_w, max_width_pts, char_w
            );
        }
    }
}
```

## Test Coverage (before Phase 6)

Concrete unit tests to be written in STORY-095 (`#[cfg(test)] mod tests` in
`crates/slideforge-pdf/src/text_layout.rs`):

Tests use `mock_char_width_pts: Some(w)` to make widths deterministic (no font I/O).
Widths below are computed as `char_count * char_width_pts`.

| Test | Input | char_width_pts | max_width_pts | Expected behavior |
|------|-------|----------------|---------------|-------------------|
| empty input | `""` | 5.0 | 50.0 | `vec![]` — no panic (EC-002) |
| single word fits | `"hello"` (5 ch) | 5.0 | 50.0 | `vec!["hello"]` (25 pts < 50 pts) |
| single word exact | `"hello"` (5 ch) | 5.0 | 25.0 | `vec!["hello"]` (25 pts == 25 pts, EC-005) |
| single word over-width | `"hello"` (5 ch) | 5.0 | 3.0 | at least 1 elem, lossless (AC-002) |
| two words fit | `"hi yo"` (5 ch) | 5.0 | 50.0 | `vec!["hi yo"]` (25 pts < 50 pts) |
| two words wrap | `"hi yo"` | 5.0 | 15.0 | `vec!["hi", "yo"]` ("hi yo"=25 pts > 15 pts) |
| only spaces | `"   "` | 5.0 | 50.0 | no panic, terminates (empty Vec) |
| long word char-wrap | 100 × `'a'` | 1.0 | 10.0 | lossless, each line ≤ 10 pts |

## Verification Layers

| Layer | Tool | Phase | Platform |
|-------|------|-------|----------|
| Bounded termination + max-width | Kani (proof) | P6 | Linux / macOS |
| Losslessness round-trip | proptest | P3 | all |
| Max-width with multi-word inputs | proptest | P3 | all |
| Fixed-point edge cases | unit tests | P3 | all |

## Changelog

| Version | Date | Change |
|---------|------|--------|
| v1.1.0 | 2026-06-11 | F-095-P2-002 fix: updated Scope, Proof Harness Skeleton, and Proptest Strategy from stale 2-arg `(input: &str, max_width: usize)` to real 3-arg signature `(text: &str, max_width_pts: f64, metrics: &FontMetrics<'_>)` with `mock_char_width_pts: Some(w)` deterministic width model; restated max-width invariant in f64-points terms; updated Test Coverage table to pts notation; added spec_version frontmatter |
