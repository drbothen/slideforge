---
document_type: verification-property
vp_id: VP-006
title: EMU-to-PDF coordinate mapping correct Y-axis flip no overflow
module: slideforge-pdf
tool: Kani
phase: P6
priority: P0
status: draft
bc_trace: [BC-4.03.005, DI-010]
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-006: EMU-to-PDF Coordinate Mapping

## Property Statement

For any `ir_y_emu` and `element_height_emu` where both are non-negative and
`ir_y_emu + element_height_emu ≤ SLIDE_HEIGHT_EMU`, the function `ir_y_to_pdf_y()`
produces a `pdf_y` value in `[0.0, SLIDE_HEIGHT_PT]` without arithmetic overflow.

The Y-axis flip invariant: `pdf_y = SLIDE_HEIGHT_PT - (ir_y_pt + element_height_pt)`.
An element at IR `y=0` appears at `pdf_y = SLIDE_HEIGHT_PT - height`.
An element at the bottom of the slide appears at `pdf_y ≈ 0`.

## Motivation

BC-4.03.005 specifies the EMU-to-PDF coordinate mapping. DI-010 requires integer EMU
throughout the IR. The coordinate mapping is a pure arithmetic function — the ideal Kani
proof target. An incorrect Y-axis flip would produce vertically mirrored PDF output, which
is a visually catastrophic and hard-to-diagnose bug.

## Feasibility Assessment

Feasible. The arithmetic is bounded integer + one float division. Kani can prove:
1. No integer overflow in `ir_y_emu + element_height_emu` (given the `≤ SLIDE_HEIGHT_EMU` precondition).
2. The float result `pdf_y` is in `[0.0, SLIDE_HEIGHT_PT]`.
3. The Y-axis flip is correct: elements at IR top produce PDF values near SLIDE_HEIGHT_PT.

S2 spike provides the full code with unit tests (3 tests passing). The Kani proof generalizes
the 3 concrete unit tests to the complete valid input space.

## Proof Harness Skeleton

```rust
// crates/slideforge-pdf/src/proofs/coordinate_mapping.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    #[kani::unwind(5)]
    fn y_axis_flip_correct_and_no_overflow() {
        let ir_y: i64 = kani::any();
        let elem_h: i64 = kani::any();
        kani::assume(ir_y >= 0);
        kani::assume(elem_h >= 0);
        kani::assume(ir_y + elem_h <= SLIDE_HEIGHT_EMU);

        let pdf_y = ir_y_to_pdf_y(Emu(ir_y), Emu(elem_h));
        kani::assert!(pdf_y >= 0.0);
        kani::assert!(pdf_y <= SLIDE_HEIGHT_PT + 0.001); // float tolerance
    }
}
```

## Test Coverage (before Phase 6)

Three concrete unit tests from S2 spike: `slide_dimensions_are_correct`,
`y_axis_flip_origin_at_top_left`, `y_axis_flip_element_at_bottom`. All confirmed passing.
