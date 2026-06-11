---
document_type: verification-property
vp_id: VP-054
title: "wrap_text termination, lossless wrapping, and max-width invariant"
module: slideforge-pdf
tool: Kani
phase: P6
priority: P1
status: draft
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

**(c) Max-width invariant.** Every output line satisfies `line.len() <= max_width`
when `max_width >= 1`. (When a single token exceeds `max_width`, that token is
emitted as a single over-width line — no infinite splitting loop.)

**Scope.** These properties target `wrap_text` as a pure function taking
`(input: &str, max_width: usize)` and returning `Vec<String>`. The Kani proof
focuses on sub-property (a) (bounded termination) and (c) (max-width bound for
bounded inputs). Sub-property (b) is verified via proptest with string-generating
strategies. Concrete unit tests cover all three sub-properties at fixed points.

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
input space (`kani::assume(input.len() <= 64)`, `kani::assume(max_width >= 1 && max_width <= 32)`).
The unwind bound is `input.len() + 1`. Feasible within Kani's bounded-model-check envelope.

**Proptest (sub-property b):** Generate arbitrary ASCII strings and arbitrary `max_width`
values in `[1, 1024]`. Assert that `output.concat()` contains all non-whitespace
characters from the input in the original order.

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
    /// Sub-property (c): no output line exceeds max_width (except single-token overflow).
    #[kani::proof]
    #[kani::unwind(65)]  // input.len() <= 64 + 1 iteration overhead
    fn wrap_text_terminates_and_respects_max_width() {
        // Bounded input: Kani cannot handle unbounded &str; use fixed-length array proxy.
        let len: usize = kani::any();
        kani::assume(len > 0 && len <= 64);

        let max_width: usize = kani::any();
        kani::assume(max_width >= 1 && max_width <= 32);

        // Build a synthetic ASCII string of `len` bytes (space + printable ASCII).
        // Kani symbolic bytes — no actual &str needed if wrap_text accepts &[u8] proxy;
        // if wrap_text takes &str, use a fixed-size stub or the char-array helper.
        let input: Vec<u8> = (0..len).map(|_| {
            let b: u8 = kani::any();
            kani::assume(b == b' ' || (b >= b'a' && b <= b'z'));
            b
        }).collect();
        let input_str = std::str::from_utf8(&input).expect("valid ASCII");

        let lines = wrap_text(input_str, max_width);

        // Termination is implicit: reaching this assertion proves the loop ended.
        // Max-width invariant: every line whose source token fits must be <= max_width.
        for line in &lines {
            // A line may exceed max_width ONLY if it contains no space (single token).
            let has_space = line.contains(' ');
            if has_space {
                kani::assert!(line.len() <= max_width);
            }
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
        max_width in 1usize..=128,
    ) {
        let lines = wrap_text(&input, max_width);
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
        input in "[a-z]{1,16}( [a-z]{1,16}){1,15}",
        max_width in 1usize..=32,
    ) {
        let lines = wrap_text(&input, max_width);
        for line in &lines {
            prop_assert!(
                line.len() <= max_width || !line.contains(' '),
                "line {:?} exceeds max_width {} and contains spaces", line, max_width
            );
        }
    }
}
```

## Test Coverage (before Phase 6)

Concrete unit tests to be written in STORY-095 (`#[cfg(test)] mod tests` in
`crates/slideforge-pdf/src/text_layout.rs`):

| Test | Input | max_width | Expected behavior |
|------|-------|-----------|-------------------|
| empty input | `""` | 10 | `vec![""]` or `vec![]` — no panic |
| single word fits | `"hello"` | 10 | `vec!["hello"]` |
| single word exact | `"hello"` | 5 | `vec!["hello"]` |
| single word over-width | `"hello"` | 3 | `vec!["hello"]` (no split, no hang) |
| two words fit | `"hi yo"` | 10 | `vec!["hi yo"]` |
| two words wrap | `"hi yo"` | 4 | `vec!["hi", "yo"]` |
| only spaces | `"   "` | 5 | no panic, terminates |
| long line no spaces | 100 × `'a'` | 10 | single element, no hang |

## Verification Layers

| Layer | Tool | Phase | Platform |
|-------|------|-------|----------|
| Bounded termination + max-width | Kani (proof) | P6 | Linux / macOS |
| Losslessness round-trip | proptest | P3 | all |
| Max-width with multi-word inputs | proptest | P3 | all |
| Fixed-point edge cases | unit tests | P3 | all |
