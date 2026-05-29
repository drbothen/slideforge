---
document_type: verification-property
vp_id: VP-037
title: "Shape: position in PPTX XML matches EMU conversion of declared user-unit values"
module: slideforge-layout
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: true
bc_trace: [BC-3.04.001, DI-010]
anchored_to_bc: BC-3.04.001
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-037: Shape Position in PPTX XML Matches EMU Conversion

## Property Statement

For any `ShapeSpec.position` field expressed in supported units, the layout stage
MUST convert to integer EMU using the exact constants defined in BC-3.04.001
Postcondition 2 via integer arithmetic (no f64). The conversion functions are pure:

| User Unit | Formula |
|-----------|---------|
| `ShapeUnit::Inches(milliinches)` | `milliinches * 914_400 / 1_000` |
| `ShapeUnit::Em(milliems)` | `milliems * brand_font_size_emu / 1_000` |
| `ShapeUnit::Cm(millicm)` | `millicm * 360_000 / 1_000` |
| `ShapeUnit::Mm(millimm)` | `millimm * 36_000 / 1_000` |
| `ShapeUnit::Pt(millipt)` | `millipt * 12_700 / 1_000` |
| `ShapeUnit::Px(millipx)` | `millipx * 9_525 / 1_000` |

Canonical test vector: `0.5in` → `ShapeUnit::Inches(500)` → `Emu(500 * 914_400 / 1_000) = Emu(457_200)`.

## Motivation

BC-3.04.001 Postcondition 2 and DI-010 (integer EMU for all coordinates). Incorrect
EMU conversion produces visually wrong shape positions in PPTX output — shapes appear
at wrong locations. The conversion is a pure arithmetic function (no I/O), making it
ideal for both unit testing and Kani formal verification.

## Feasibility Assessment

Kani-amenable: the conversion functions `inches_to_emu`, `em_to_emu`, `cm_to_emu`,
etc. are pure functions on `i64` inputs using integer arithmetic. Kani can verify:
1. No overflow for any representable milliinches value within slide bounds.
2. The canonical test vector `Inches(500) → Emu(457_200)` is exact.

`kani_amenable: true` — pure integer arithmetic conversion.

## Proof Harness Skeleton

```rust
// crates/slideforge-layout/src/proofs/shape_emu.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn inches_to_emu_canonical_vector() {
        // 0.5in = Inches(500) → Emu(457_200)
        let result = shape_unit_to_emu(ShapeUnit::Inches(500), 0);
        kani::assert!(result == 457_200);
    }

    #[kani::proof]
    fn inches_to_emu_no_overflow() {
        // Slide width is 10in = 9_144_000 EMU; milliinches fits in i64
        let milliinches: i64 = kani::any();
        kani::assume(milliinches >= 0);
        kani::assume(milliinches <= 100_000); // up to 100in, reasonable slide bound
        // Multiplication: milliinches * 914_400 must not overflow i64
        // max: 100_000 * 914_400 = 91_440_000_000 < i64::MAX (9.2e18)
        let _ = shape_unit_to_emu(ShapeUnit::Inches(milliinches), 0);
        // No panic = no overflow (Kani verifies)
    }

    #[kani::proof]
    fn cm_to_emu_canonical() {
        // 1cm = Cm(1000) → Emu(360_000)
        let result = shape_unit_to_emu(ShapeUnit::Cm(1000), 0);
        kani::assert!(result == 360_000);
    }
}
```

## Test Coverage (before Phase 6)

```rust
// crates/slideforge-layout/src/tests/shape_emu.rs
#[test]
fn test_inches_to_emu() {
    // BC-3.04.001 Postcondition 2 canonical vector
    assert_eq!(shape_unit_to_emu(ShapeUnit::Inches(500), 0), 457_200);
    assert_eq!(shape_unit_to_emu(ShapeUnit::Inches(1000), 0), 914_400);
}

#[test]
fn test_cm_to_emu() {
    assert_eq!(shape_unit_to_emu(ShapeUnit::Cm(1000), 0), 360_000);
}

#[test]
fn test_mm_to_emu() {
    assert_eq!(shape_unit_to_emu(ShapeUnit::Mm(1000), 0), 36_000);
}

#[test]
fn test_pt_to_emu() {
    assert_eq!(shape_unit_to_emu(ShapeUnit::Pt(1000), 0), 12_700);
}

#[test]
fn test_px_to_emu() {
    assert_eq!(shape_unit_to_emu(ShapeUnit::Px(1000), 0), 9_525);
}
```

## Test Vectors

| Input | Expected EMU | Category |
|-------|-------------|----------|
| `ShapeUnit::Inches(500)` (0.5in) | `457_200` | canonical (BC) |
| `ShapeUnit::Inches(1000)` (1.0in) | `914_400` | happy-path |
| `ShapeUnit::Cm(1000)` (1cm) | `360_000` | canonical (BC) |
| `ShapeUnit::Mm(1000)` (1mm) | `36_000` | canonical (BC) |
| `ShapeUnit::Pt(1000)` (1pt) | `12_700` | canonical (BC) |
| `ShapeUnit::Px(1000)` (1px) | `9_525` | canonical (BC) |
