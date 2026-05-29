---
document_type: verification-property
vp_id: VP-048
title: "Shape: from_inches / from_em with i64::MAX input returns Err(LayoutError::ArithmeticOverflow) — not silent saturation"
module: slideforge-layout
tool: Kani
phase: P6
priority: P1
status: draft
kani_amenable: true
bc_trace: [BC-3.04.001]
anchored_to_bc: BC-3.04.001
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-048: Shape EMU Conversion Overflow Returns Hard Error (Not Silent Saturation)

## Property Statement

`ShapeUnit::from_inches` and `ShapeUnit::from_em` MUST use `checked_mul` for the EMU
conversion. For any input where `milliinches * 914_400` (or `milliems * brand_font_size_emu`)
would overflow `i64`, the function MUST return
`Err(LayoutError::ArithmeticOverflow { source_slide_index, span })`, which the accumulation
site wraps in `LayoutError::Multiple`. Silent saturation via `saturating_mul` is FORBIDDEN.

Formally, for the Inches case:
- If `milliinches.checked_mul(914_400).is_none()` → the function returns `Err(ArithmeticOverflow { .. })`
- For `ShapeUnit::Inches(i64::MAX)`: `i64::MAX * 914_400` overflows, so the function MUST return `Err`

The same holds for `from_em`: if `milliems.checked_mul(brand_font_size_emu)` overflows, the function MUST return `Err`.

The caller accumulates the error via `LayoutError::multiple(errors)` before propagating to the user as E-LAY-006.

## Motivation

BC-3.04.001 Invariant 8, EC-014, EC-015. If `saturating_mul` were used instead of `checked_mul`,
an extreme input value like `i64::MAX` would silently produce a large-but-wrong EMU coordinate
that passes through to PPTX/PDF/HTML exporters without any error signal. The resulting output
would have garbage shape positions that are visually wrong and impossible to diagnose. The
production-grade principle (CLAUDE.md Rule 1) requires hard errors for unrepresentable inputs,
never silent fallback.

CLAUDE.md: "no silent fallback on invalid input"; "Rule 4: no rationalized deferrals".

## Feasibility Assessment

Kani-amenable: `from_inches` and `from_em` are pure functions on `i64` inputs performing
integer multiplication with `checked_mul`. Kani can exhaustively verify over symbolic `i64`
inputs with overflow assumptions:

1. For `i64::MAX` input, the function returns `Err`.
2. For any input `n` where `n * 914_400` does NOT overflow, the function returns `Ok`.
3. No use of `saturating_mul` or `wrapping_mul` anywhere in the path.

`kani_amenable: true` — pure integer arithmetic with `checked_mul`; no I/O or pipeline state.

## Proof Harness Skeleton

```rust
// crates/slideforge-layout/src/proofs/shape_overflow.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    /// BC-3.04.001 EC-014: i64::MAX inches overflows — must return Err, not saturate
    #[kani::proof]
    fn inches_max_returns_arithmetic_overflow() {
        let result = from_inches(i64::MAX, 0, SourceSpan::default());
        kani::assert!(matches!(result, Err(LayoutError::ArithmeticOverflow { .. })));
    }

    /// BC-3.04.001 EC-015: i64::MAX em overflows — must return Err, not saturate
    #[kani::proof]
    fn em_max_returns_arithmetic_overflow() {
        // brand_font_size_emu default is 457_200 (36pt)
        let result = from_em(i64::MAX, 457_200, 0, SourceSpan::default());
        kani::assert!(matches!(result, Err(LayoutError::ArithmeticOverflow { .. })));
    }

    /// Any inches value in [0, 100_000] (up to 100in) must succeed without overflow
    #[kani::proof]
    fn inches_within_slide_bounds_succeeds() {
        let milliinches: i64 = kani::any();
        kani::assume(milliinches >= 0);
        kani::assume(milliinches <= 100_000); // 100in upper bound
        // max: 100_000 * 914_400 = 91_440_000_000 << i64::MAX (9.2e18)
        let result = from_inches(milliinches, 0, SourceSpan::default());
        kani::assert!(result.is_ok());
    }

    /// Overflow check: i64::MAX * 914_400 overflows i64 — verify checked_mul returns None
    #[kani::proof]
    fn checked_mul_detects_overflow() {
        let overflow_result = i64::MAX.checked_mul(914_400);
        kani::assert!(overflow_result.is_none());
    }
}
```

## Test Coverage (before Phase 6)

```rust
// crates/slideforge-layout/src/tests/shape_overflow.rs
#[test]
fn test_inches_max_returns_arithmetic_overflow() {
    // BC-3.04.001 EC-014 canonical test vector
    let result = from_inches(i64::MAX, 0, SourceSpan::dummy());
    assert!(
        matches!(result, Err(LayoutError::ArithmeticOverflow { source_slide_index: 0, .. })),
        "i64::MAX inches must return ArithmeticOverflow, not panic or saturate"
    );
}

#[test]
fn test_em_max_returns_arithmetic_overflow() {
    // BC-3.04.001 EC-015 canonical test vector
    let result = from_em(i64::MAX, 457_200, 0, SourceSpan::dummy());
    assert!(
        matches!(result, Err(LayoutError::ArithmeticOverflow { .. })),
        "i64::MAX em must return ArithmeticOverflow"
    );
}

#[test]
fn test_overflow_accumulates_in_multiple() {
    // layout_shapes accumulates the overflow error via LayoutError::multiple
    let shapes = vec![
        ShapeSpec {
            position: ShapePosition {
                x: ShapeUnit::Inches(i64::MAX),
                ..default_position()
            },
            ..default_shape()
        },
    ];
    let result = layout_shapes(0, &shapes, &BrandConfig::default());
    assert!(matches!(
        result,
        Err(LayoutError::Multiple { ref inner }) if inner.iter().any(|e| matches!(e, LayoutError::ArithmeticOverflow { .. }))
    ));
}

#[test]
fn test_valid_inches_succeeds() {
    // Sanity: 1in does NOT overflow
    let result = from_inches(1_000, 0, SourceSpan::dummy()); // 1in = Inches(1000)
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 914_400);
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `from_inches(i64::MAX, slide=0, span)` | `Err(LayoutError::ArithmeticOverflow { source_slide_index: 0, .. })` | overflow (EC-014) |
| `from_em(i64::MAX, 457_200, slide=0, span)` | `Err(LayoutError::ArithmeticOverflow { source_slide_index: 0, .. })` | overflow (EC-015) |
| Slide with `ShapeUnit::Inches(i64::MAX)` in position | `Err(LayoutError::Multiple { inner: [ArithmeticOverflow { .. }] })` | accumulation |
| `from_inches(1_000, slide=0, span)` | `Ok(914_400)` | happy-path |
| `from_inches(500, slide=0, span)` | `Ok(457_200)` | canonical (BC) |
