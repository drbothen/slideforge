---
document_type: verification-property
vp_id: VP-007
title: WCAG contrast formula correct luminance linearization
module: slideforge-validate
tool: Kani
phase: P6
priority: P0
status: draft
bc_trace: [BC-4.01.004, BC-4.03.003]
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-007: WCAG Contrast Formula Correct Luminance Linearization

## Property Statement

For any RGB color `(r, g, b)` where each channel is in `[0, 255]`, the
`relative_luminance(r, g, b)` function:
1. Returns a value in `[0.0, 1.0]`
2. Uses the 0.04045 linearization threshold (not the older 0.03928)
3. Black `(0,0,0)` returns `0.0`; white `(255,255,255)` returns `1.0`

And for any two colors with luminance L1 ≥ L2:
`contrast_ratio(L1, L2) = (L1 + 0.05) / (L2 + 0.05)` is in `[1.0, 21.0]`.

## Motivation

BC-4.01.004 requires all brand color values to pass WCAG AA contrast ratio checks;
BC-4.03.003 requires all body text to meet 4.5:1 contrast ratio. Both contracts depend
on the correctness of the contrast ratio formula (WCAG 1.4.3 Contrast — not 1.4.1 Use
of Color). S3 spike identified this as a Kani candidate. The WCAG 2.x spec uses 0.04045
(not 0.03928 from some older references). Using the wrong threshold changes contrast
ratios by < 0.5% in practice but must be correct per spec. The formula is a pure math
function with documented invariants.

## Feasibility Assessment

Feasible. The luminance function is bounded floating-point arithmetic over `[0, 255]`
input range. Kani can prove the output range invariant and the boundary values. The
linearization threshold comparison is a simple float comparison. The S3 spike prototype
already has 6/6 unit tests passing for representative color pairs.

## Proof Harness Skeleton

```rust
// crates/slideforge-validate/src/proofs/contrast.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn luminance_in_unit_interval() {
        let r: u8 = kani::any();
        let g: u8 = kani::any();
        let b: u8 = kani::any();
        let lum = relative_luminance(r, g, b);
        kani::assert!(lum >= 0.0);
        kani::assert!(lum <= 1.0 + 1e-6); // float tolerance
    }

    #[kani::proof]
    fn black_is_zero_white_is_one() {
        kani::assert!((relative_luminance(0, 0, 0) - 0.0).abs() < 1e-6);
        kani::assert!((relative_luminance(255, 255, 255) - 1.0).abs() < 1e-6);
    }

    #[kani::proof]
    fn contrast_ratio_in_range() {
        let r1: u8 = kani::any(); let g1: u8 = kani::any(); let b1: u8 = kani::any();
        let r2: u8 = kani::any(); let g2: u8 = kani::any(); let b2: u8 = kani::any();
        let ratio = contrast_ratio(r1, g1, b1, r2, g2, b2);
        kani::assert!(ratio >= 1.0 - 1e-6);
        kani::assert!(ratio <= 21.0 + 1e-6);
    }
}
```

## Test Coverage (before Phase 6)

S3 spike: `src/contrast.rs`, 6/6 unit tests passing. Colors tested: dark navy on white
(11.50), accent orange on white (2.80 — FAIL), white on severity red (4.83 — WARN).
