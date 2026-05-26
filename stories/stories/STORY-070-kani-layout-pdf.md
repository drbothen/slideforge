---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-070
title: "Kani Proofs: slideforge-pdf (VP-006)"
epic: EPIC-20
wave: 6
points: 5
priority: P0
tdd_mode: facade
status: draft
crate: slideforge-pdf
subsystems: [SS-07]
target_module: slideforge-pdf
behavioral_contracts: []
# BC status: pending PO authorship — Wave 6 formal-verification stories prove existing
# behavioral guarantees. BC exercised through VP-006: BC-4.03.005 (EMU-to-PDF coordinate
# mapping). VP-006 is P0 (Phase 6 blocking). No new BCs introduced here.
verification_properties: [VP-006]
nfr_refs: []
assumption_validations: []
risk_mitigations: []
depends_on:
  - STORY-043
  - STORY-044
blocks: []
estimated_days: 2
---

# STORY-070: Kani Proofs: slideforge-pdf (VP-006)

## Summary

Add Phase 6 formal verification for `slideforge-pdf`, specifically the EMU-to-PDF
coordinate mapping pure function.

**VP-006 (Kani proof, P0):** The `emu_to_pdf_y(ir_y, element_height, page_height)`
function implements the correct Y-axis flip (PDF origin is bottom-left; slideforge
origin is top-left) and never overflows `i64` for any OOXML-valid coordinate input.

The Y-axis flip formula is:
```
pdf_y = page_height_emu - ir_y - element_height
```

The Kani proof formally verifies:
1. `pdf_y` is always ≥ 0 (no negative PDF coordinates)
2. `pdf_y + element_height` ≤ `page_height_emu` (no coordinates beyond page top)
3. No integer overflow occurs in any intermediate computation

**tdd_mode: facade** — the Kani harness is the combined scaffold+implementation.
VP-006 is P0 — CI blocks on failure.

**Platform constraint:** Kani runs only on Linux/macOS. A concrete fallback unit test
runs on Windows.

The proof was partially sketched in the verification-architecture.md file (see
implementation notes). STORY-070 delivers the COMPLETE proof plus CI integration.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3,000 |
| `crates/slideforge-pdf/src/proofs/vp006_coordinate_mapping.rs` | ~2,000 |
| Concrete fallback test | ~500 |
| Justfile additions | ~300 |
| `.github/workflows/kani.yml` addition | ~600 |
| Referenced source (`coordinate_mapping.rs`) | ~3,000 |
| **Total** | **~9,400** |

> 9,400 tokens ≈ 9% of a 100k-token context window. Well within budget.

## Acceptance Criteria

### AC-001: VP-006 Kani proof compiles and passes (P0)
Kani proof `proofs::vp006_coordinate_mapping::y_axis_flip_no_overflow` in
`crates/slideforge-pdf/src/proofs/vp006_coordinate_mapping.rs` compiles under
`cargo kani -p slideforge-pdf` and reports `VERIFICATION SUCCESSFUL`.
The proof verifies that for all OOXML-valid inputs (ir_y, element_height,
page_height all within `0..=SLIDE_HEIGHT_EMU`), the Y-axis flip formula
produces a non-negative result that does not exceed page_height and involves
no intermediate i64 overflow.
(traces to VP-006 — EMU-to-PDF coordinate mapping: correct Y-axis flip, no overflow;
traces to BC-4.03.005 postcondition)

### AC-002: Proof covers the key boundary cases via kani::assume
The proof uses `kani::assume` to constrain inputs to the OOXML coordinate domain:
- `ir_y >= 0 && ir_y <= SLIDE_HEIGHT_EMU`
- `element_height >= 0 && element_height <= SLIDE_HEIGHT_EMU`
- `ir_y + element_height <= page_height` (element fits within page)
- `page_height > 0 && page_height <= SLIDE_HEIGHT_EMU`

These constraints match real OOXML document limits without over-constraining the proof.

### AC-003: Proof verifies the three postconditions
The proof asserts all three postconditions after calling `emu_to_pdf_y()`:
1. `pdf_y >= 0` — no negative PDF coordinates
2. `pdf_y + element_height <= page_height` — no coordinates above page top
3. `pdf_y == page_height - ir_y - element_height` — formula correctness

### AC-004: Concrete fallback test covers Windows CI
A concrete unit test `test_y_axis_flip_correctness` under `#[cfg(not(kani))]` in
the same file drives the proof scenario with concrete values including:
- Middle of page: ir_y = page_height / 2
- Top of page: ir_y = 0
- Bottom of page: ir_y = page_height - element_height
- Boundary: ir_y + element_height = page_height (exactly fills page)

### AC-005: `just kani-pdf` Justfile target succeeds
`just kani-pdf` runs the VP-006 harness and exits 0 on Linux/macOS with Kani installed.
The target is documented with a platform constraint comment.

### AC-006: CI `kani.yml` includes slideforge-pdf proof as P0 gate
`.github/workflows/kani.yml` has a step `kani-pdf` with `continue-on-error: false`
(default) running `just kani-pdf` on `ubuntu-latest`.

## Tasks

- [ ] 1. Read `crates/slideforge-pdf/src/coordinate_mapping.rs` (emu_to_pdf_y function)
- [ ] 2. Read `crates/slideforge-pdf/src/lib.rs` (SLIDE_HEIGHT_EMU constant definition)
- [ ] 3. Verify `emu_to_pdf_y` is a pure function with no side effects (required for Kani)
         — if it is not pure, refactor minimally under `#[cfg(kani)]` wrapper
- [ ] 4. Create `crates/slideforge-pdf/src/proofs/mod.rs` with `#[cfg(kani)]` guard
- [ ] 5. Write VP-006 proof in `crates/slideforge-pdf/src/proofs/vp006_coordinate_mapping.rs`
         — use `kani::any::<i64>()` for all three inputs; apply `kani::assume` bounds;
         call `emu_to_pdf_y`; assert three postconditions
- [ ] 6. Add concrete fallback unit test under `#[cfg(not(kani))]` section
- [ ] 7. Modify `crates/slideforge-pdf/src/lib.rs` to add `mod proofs;` under `#[cfg(kani)]`
- [ ] 8. Add `just kani-pdf` target to Justfile
- [ ] 9. Add `kani-pdf` P0 step to `.github/workflows/kani.yml`
- [ ] 10. Run `cargo kani -p slideforge-pdf --harness proofs::vp006_coordinate_mapping::proofs::y_axis_flip_no_overflow`
- [ ] 11. Run `cargo test -p slideforge-pdf` to verify concrete fallback test passes

## Previous Story Intelligence

N/A — first story in EPIC-20 targeting slideforge-pdf. Predecessor stories:
- STORY-043 (PDF core: pdf-writer + krilla + SlideTagEngine): establishes the
  `slideforge-pdf` crate structure and EMU constants.
- STORY-044 (PDF: EMU-to-PDF coordinate mapping + Y-axis flip): implements the
  `emu_to_pdf_y` pure function that VP-006 proves. A Kani skeleton was added in
  STORY-044 (noted in the wave schedule). STORY-070 delivers the COMPLETE proof.

The verification-architecture.md file contains a proof skeleton for VP-006 (see
`y_axis_flip_no_overflow` in that document). Use it as the starting point and complete
it with the correct function call signature, constant names, and postcondition assertions
matching the actual STORY-044 implementation.

Key implementation note from STORY-044: the function signature is
`emu_to_pdf_y(ir_y: Emu, element_height: Emu, page_height: Emu) -> PdfUnit`
where `Emu` wraps `i64` and `PdfUnit` wraps `f64` or `i64` (check STORY-044 for
the actual return type). If `PdfUnit` uses `f64`, the proof must use Kani's float
handling — see note in implementation notes below.

## Architecture Compliance Rules

Derived from `architecture/verification-architecture.md` and `architecture/purity-boundary-map.md`:

1. **Pure function requirement:** Kani proofs only work on pure functions. `emu_to_pdf_y`
   must have no side effects, no I/O, no heap allocation in the proof path. If the
   production function uses `PdfUnit(f64)`, see the float handling note below.
2. **Float handling:** Kani 0.55+ has limited f64 support. If `PdfUnit` is `f64`,
   use an integer proxy: `emu_to_pdf_y_int(ir_y: i64, elem_h: i64, page_h: i64) -> i64`
   that computes in integer EMU space. The proof verifies the integer formula; a
   separate concrete test verifies the float conversion. This mirrors the VP-007
   approach from STORY-068.
3. **SLIDE_HEIGHT_EMU constant:** The standard OOXML slide height is 6858000 EMU
   (9144000 EMU for landscape). Use `SLIDE_HEIGHT_EMU = 6858000i64` as the proof bound.
   Verify this constant matches the production code in STORY-044.
4. **Forbidden dependencies:** `slideforge-pdf` MUST NOT depend on `slideforge-syntax`,
   `slideforge-eval`, or `slideforge-validate` in its proofs. The coordinate mapping
   is a pure arithmetic function; it needs no parser or evaluator types.
5. **`#[cfg(kani)]` required:** Proof module must not compile in normal builds.

## Library and Framework Requirements

| Library | Pinned Version | Role | Notes |
|---------|---------------|------|-------|
| kani | latest (~0.65+) | Formal model checker (uses bundled nightly toolchain internally) | Installed via `cargo install kani-verifier --locked && cargo kani setup` |

No new crate deps required. The proof uses only `kani::any()`, `kani::assume()`,
`kani::assert()`, and the production `emu_to_pdf_y` function from STORY-044.

## File Structure Requirements

Files to CREATE:
- `crates/slideforge-pdf/src/proofs/mod.rs` — module root with `#[cfg(kani)]` guard
- `crates/slideforge-pdf/src/proofs/vp006_coordinate_mapping.rs` — VP-006 Kani proof + concrete fallback

Files to MODIFY:
- `crates/slideforge-pdf/src/lib.rs` — add `mod proofs;` under `#[cfg(kani)]`
- `Justfile` — add `kani-pdf` target
- `.github/workflows/kani.yml` — add `kani-pdf` P0 step

Files to NOT touch:
- `crates/slideforge-pdf/src/coordinate_mapping.rs` — proof calls it, not modifies it
- Any other production source in slideforge-pdf

## Implementation Notes

### VP-006 Proof (from architecture sketch, completed)

```rust
// crates/slideforge-pdf/src/proofs/vp006_coordinate_mapping.rs
#[cfg(kani)]
mod proofs {
    use crate::coordinate_mapping::{emu_to_pdf_y_int, SLIDE_HEIGHT_EMU};

    #[kani::proof]
    #[kani::unwind(10)]
    fn y_axis_flip_no_overflow() {
        let ir_y: i64 = kani::any();
        let elem_h: i64 = kani::any();
        let page_h: i64 = kani::any();

        // Constrain to valid OOXML slide coordinate range
        kani::assume(ir_y >= 0 && ir_y <= SLIDE_HEIGHT_EMU);
        kani::assume(elem_h >= 0 && elem_h <= SLIDE_HEIGHT_EMU);
        kani::assume(page_h > 0 && page_h <= SLIDE_HEIGHT_EMU);
        kani::assume(ir_y + elem_h <= page_h);

        // Call the production Y-axis flip function (integer version for Kani)
        let pdf_y = emu_to_pdf_y_int(ir_y, elem_h, page_h);

        // Postcondition 1: no negative PDF coordinates
        kani::assert(pdf_y >= 0, "Y coordinate must be non-negative in PDF space");

        // Postcondition 2: element must fit within page bounds
        kani::assert(
            pdf_y + elem_h <= page_h,
            "Element must not extend beyond page top in PDF space"
        );

        // Postcondition 3: formula correctness (Y-axis flip)
        kani::assert(
            pdf_y == page_h - ir_y - elem_h,
            "PDF Y must equal page_height - ir_y - element_height"
        );
    }
}

// Concrete fallback test for Windows CI and general regression
#[cfg(not(kani))]
#[cfg(test)]
mod fallback_tests {
    use crate::coordinate_mapping::{emu_to_pdf_y_int, SLIDE_HEIGHT_EMU};

    /// Standard OOXML slide: 9144000 EMU wide × 6858000 EMU tall (16:9)
    const PAGE_H: i64 = SLIDE_HEIGHT_EMU; // 6858000

    #[test]
    fn test_y_axis_flip_correctness() {
        // Top of page (ir_y = 0) → PDF bottom (pdf_y = page_height - elem_h)
        let elem_h = 914400i64; // 1 inch
        assert_eq!(emu_to_pdf_y_int(0, elem_h, PAGE_H), PAGE_H - elem_h);

        // Middle of page
        let ir_y = PAGE_H / 2;
        let expected = PAGE_H - ir_y - elem_h;
        assert_eq!(emu_to_pdf_y_int(ir_y, elem_h, PAGE_H), expected);
        assert!(expected >= 0);

        // Bottom of page: element fills from bottom to one elem_h above
        let ir_y_bottom = PAGE_H - elem_h;
        assert_eq!(emu_to_pdf_y_int(ir_y_bottom, elem_h, PAGE_H), 0);
    }

    #[test]
    fn test_no_overflow_near_boundaries() {
        let max_coord = SLIDE_HEIGHT_EMU;
        let elem_h = 1i64;
        let result = emu_to_pdf_y_int(max_coord - elem_h, elem_h, max_coord);
        assert_eq!(result, 0);
        assert!(result >= 0);
    }
}
```

### `emu_to_pdf_y_int` helper (add to coordinate_mapping.rs)

If the production `emu_to_pdf_y` returns `f64` (as `PdfUnit`), add an integer-domain
proxy alongside it (not instead of it):

```rust
// crates/slideforge-pdf/src/coordinate_mapping.rs
// SLIDE_HEIGHT_EMU for a standard 16:9 slide (9.14" × 6.86")
pub const SLIDE_HEIGHT_EMU: i64 = 6_858_000;

// Integer-domain proxy for formal verification (Kani + concrete tests)
// Computes Y-axis flip entirely in EMU integer space.
pub fn emu_to_pdf_y_int(ir_y: i64, element_height: i64, page_height: i64) -> i64 {
    page_height - ir_y - element_height
}
```

The production `emu_to_pdf_y` function may call `emu_to_pdf_y_int` and then convert
to PDF user units (divide by 12700 for 1/72-inch units), or it may do the conversion
independently. The integer proxy is a thin addition for provability; it does not
replace the production function.

### Justfile Targets

```
kani-pdf:
    # Platform: Linux/macOS only. P0 gate — must pass for v1.0 release.
    cargo kani -p slideforge-pdf --harness proofs::vp006_coordinate_mapping::proofs::y_axis_flip_no_overflow
```

### CI Configuration

```yaml
# .github/workflows/kani.yml
- name: kani-pdf (P0 gate)
  if: runner.os != 'Windows'
  # continue-on-error: false (P0)
  run: |
    cargo install kani-verifier --locked
    just kani-pdf
```

## Dependencies

### Dependency Justification

- STORY-070 depends on STORY-043 because the `slideforge-pdf` crate structure,
  `SLIDE_HEIGHT_EMU` constant, and `PdfUnit` type are established there. Without
  the crate, there is no proof target.
- STORY-070 depends on STORY-044 because `emu_to_pdf_y` and the Y-axis flip formula
  are implemented in that story. The proof verifies the behavior of STORY-044's code.
- STORY-070 does not block any other story.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ir_y = 0 (top of slide) — VP-006 | pdf_y = page_h - elem_h; element sits at PDF top minus height |
| EC-002 | ir_y = page_h - elem_h (bottom of slide) — VP-006 | pdf_y = 0; element sits at PDF bottom |
| EC-003 | ir_y = page_h / 2 (middle) — VP-006 | pdf_y = page_h/2 - elem_h; symmetric flip |
| EC-004 | elem_h = 0 (zero-height element) — VP-006 | pdf_y = page_h - ir_y; no crash |
| EC-005 | page_h = 1 (minimum positive) — VP-006 | proof: constrained via kani::assume(ir_y + elem_h <= 1); passes |
| EC-006 | page_h = SLIDE_HEIGHT_EMU (maximum standard) — VP-006 | proof: all valid positions within range |
| EC-007 | Windows CI (Kani unavailable) | Concrete fallback tests run; two boundary tests cover the proof scenarios |

## Test Strategy

- **VP-006:** Kani bounded model checking. VERIFICATION SUCCESSFUL is the only pass
  criterion. P0 — blocks v1.0 release if failing.
- **Concrete fallback:** Two unit tests cover the boundary conditions (top, bottom, middle
  of page, zero-height element). Run on all 5 CI platforms.
- **Regression:** Once the proof passes, it is a permanent P0 CI gate. Any change to
  `emu_to_pdf_y_int` that breaks the proof is a release blocker.

---

*Subsystem anchor justification: SS-07 (PDF Export) owns this story's scope because
slideforge-pdf is the sole crate implementing PDF coordinate mapping — the target
function of VP-006, per ARCH-INDEX Subsystem Registry.*
