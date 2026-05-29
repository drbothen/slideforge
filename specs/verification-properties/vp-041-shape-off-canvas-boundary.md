---
document_type: verification-property
vp_id: VP-041
title: "Shape: x + width == page_width is NOT off-canvas; x + width == page_width + 1 EMU IS off-canvas"
module: slideforge-layout
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: true
bc_trace: [BC-3.04.001]
anchored_to_bc: BC-3.04.001
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-041: Shape Off-Canvas Boundary (Inclusive Boundary Semantics)

## Property Statement

For a shape with position `x_emu + width_emu == PAGE_WIDTH_EMU` (inclusive boundary),
the layout stage MUST NOT produce `LayoutWarning::OffCanvas`. For a shape with
`x_emu + width_emu == PAGE_WIDTH_EMU + 1` (one EMU beyond the page edge), the layout
stage MUST produce `LayoutWarning::OffCanvas { slide_index, shape_type, x_emu, y_emu }`.

Formally:
- `x + width <= PAGE_WIDTH_EMU → !off_canvas(x, width)`
- `x + width > PAGE_WIDTH_EMU → off_canvas(x, width)`

The same semantics apply to Y-axis: `y + height <= PAGE_HEIGHT_EMU` is on-canvas.

## Motivation

BC-3.04.001 Invariant 6 and EC-002, EC-003. The inclusive boundary means a shape
exactly touching the page edge is valid — this is the common case for full-width
backgrounds and banners. A strict `>` check ensures the boundary condition is not
accidentally treated as off-canvas, which would produce spurious warnings on every
full-width shape.

## Feasibility Assessment

Kani-amenable: the off-canvas check is a pure comparison function:
`x_emu + width_emu > PAGE_WIDTH_EMU`. Kani can formally verify the boundary
conditions with symbolic `i64` inputs.

`kani_amenable: true` — pure comparison on integer EMU values.

## Proof Harness Skeleton

```rust
// crates/slideforge-layout/src/proofs/off_canvas.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    const PAGE_WIDTH_EMU: i64 = 9_144_000; // 10in

    #[kani::proof]
    fn inclusive_boundary_is_not_off_canvas() {
        let x: i64 = kani::any();
        let w: i64 = kani::any();
        kani::assume(x >= 0 && w >= 0);
        kani::assume(x + w == PAGE_WIDTH_EMU); // exact boundary
        let result = is_off_canvas_x(x, w, PAGE_WIDTH_EMU);
        kani::assert!(!result);
    }

    #[kani::proof]
    fn one_emu_beyond_boundary_is_off_canvas() {
        let x: i64 = kani::any();
        let w: i64 = kani::any();
        kani::assume(x >= 0 && w >= 0);
        kani::assume(x + w == PAGE_WIDTH_EMU + 1); // one EMU beyond
        let result = is_off_canvas_x(x, w, PAGE_WIDTH_EMU);
        kani::assert!(result);
    }

    #[kani::proof]
    fn strictly_within_is_not_off_canvas() {
        let x: i64 = kani::any();
        let w: i64 = kani::any();
        kani::assume(x >= 0 && w >= 0);
        kani::assume(x + w < PAGE_WIDTH_EMU);
        let result = is_off_canvas_x(x, w, PAGE_WIDTH_EMU);
        kani::assert!(!result);
    }
}
```

## Test Coverage (before Phase 6)

```rust
// crates/slideforge-layout/src/tests/off_canvas.rs
const PAGE_W: i64 = 9_144_000; // 10in

#[test]
fn test_exact_boundary_is_on_canvas() {
    // x=8in, width=2in, page=10in: 8+2==10 → on-canvas (EC-002 canonical vector)
    let x = 8 * 914_400; // 8in in EMU
    let w = 2 * 914_400; // 2in in EMU
    assert!(!is_off_canvas_x(x, w, PAGE_W));
}

#[test]
fn test_one_emu_beyond_is_off_canvas() {
    let x = 8 * 914_400;
    let w = 2 * 914_400 + 1; // 1 EMU beyond 10in
    assert!(is_off_canvas_x(x, w, PAGE_W));
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `x=8in, width=2in` on 10in page (`x+w == PAGE_W`) | No `OffCanvas` warning | boundary (EC-002) |
| `x=8in, width=2in + 1EMU` on 10in page | `LayoutWarning::OffCanvas` | boundary (EC-003) |
| `x=-1in, width=2in` (negative x) | `LayoutWarning::OffCanvas` | edge-case (EC-004) |
