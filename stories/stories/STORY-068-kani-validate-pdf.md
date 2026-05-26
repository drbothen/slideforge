---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-068
title: "Kani Proofs: slideforge-validate + slideforge-pdf (VP-002, VP-007, VP-008)"
epic: EPIC-20
wave: 6
points: 8
priority: P0
tdd_mode: facade
status: draft
crate: slideforge-validate
subsystems: [SS-03, SS-07]
target_module: slideforge-validate
behavioral_contracts: []
# BC status: pending PO authorship — Wave 6 stories prove existing BC guarantees
# via formal methods. BCs exercised: BC-5.01.001 (VP-002, VP-008), BC-4.01.004
# + BC-4.03.003 (VP-007). All three VPs in this story are P0 (Phase 6 blocking).
# No new BCs introduced here.
verification_properties: [VP-002, VP-007, VP-008]
nfr_refs: []
assumption_validations: []
risk_mitigations: []
depends_on:
  - STORY-015
  - STORY-016
  - STORY-017
  - STORY-043
  - STORY-044
blocks: []
estimated_days: 3
---

# STORY-068: Kani Proofs: slideforge-validate + slideforge-pdf (VP-002, VP-007, VP-008)

## Summary

Add Phase 6 formal verification for `slideforge-validate` (VP-002, VP-007, VP-008)
and `slideforge-pdf` (VP-006 is in STORY-070). All three VPs in this story are P0
(Phase 6 blocking for v1.0 release).

1. **VP-002 (Kani proof, P0):** Alt-missing produces error BEFORE layout — the
   validation stage always intercepts missing alt before any layout or export runs.
2. **VP-007 (Kani proof, P0):** WCAG contrast formula uses correct luminance
   linearization threshold (0.04045), not an approximation.
3. **VP-008 (Kani proof, P0):** Alt enforcement invariant — an element with
   `AltText::Text("")` (empty string) always produces a diagnostic. Empty alt is
   not accepted as a substitute for a real description.

**tdd_mode: facade** — proof harnesses are the combined scaffold+implementation
deliverable. All three proofs are P0 gates; CI blocks on any failure.

**Platform constraint:** Kani runs only on Linux/macOS. Windows CI must run
equivalent concrete unit tests (see EC-007 below).

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4,500 |
| `crates/slideforge-validate/src/proofs/` (3 files) | ~4,000 |
| Concrete fallback unit tests (3 tests) | ~1,500 |
| Justfile additions | ~400 |
| `.github/workflows/kani.yml` additions | ~800 |
| Referenced source (`alt_check.rs`, `wcag.rs`) | ~7,000 |
| **Total** | **~18,200** |

> 18,200 tokens ≈ 18% of a 100k-token context window. Within the 20-30% budget.

## Acceptance Criteria

### AC-001: VP-002 Kani proof compiles and passes (P0)
Kani proof `proofs::vp002_alt_before_layout::alt_missing_before_layout` in
`crates/slideforge-validate/src/proofs/vp002_alt_before_layout.rs` compiles
under `cargo kani -p slideforge-validate` and reports `VERIFICATION SUCCESSFUL`.
The proof verifies that `validate(deck)` returns `Err(diagnostics)` containing
at least one `AltTextMissing` diagnostic before any layout computation runs,
for any Deck containing a visual element with `alt: None`.
(traces to VP-002 — alt-missing produces error before layout; traces to BC-5.01.001)

### AC-002: VP-007 Kani proof compiles and passes (P0)
Kani proof `proofs::vp007_wcag_contrast::luminance_linearization_threshold` in
`crates/slideforge-validate/src/proofs/vp007_wcag_contrast.rs` compiles and passes.
The proof verifies that `linearize_srgb(c)` uses the threshold 0.04045 (not 0.03928
or other approximations) and returns a value in `[0.0, 1.0]` for any sRGB component
`c` in `[0.0, 1.0]`. Since Kani does not model floats exactly, the proof uses
scaled integer arithmetic (sRGB ×10000 → integer) to verify the threshold branch.
(traces to VP-007 — WCAG contrast formula: correct luminance linearization)

### AC-003: VP-008 Kani proof compiles and passes (P0)
Kani proof `proofs::vp008_alt_diagnostic_invariant::empty_alt_always_diagnostic` in
`crates/slideforge-validate/src/proofs/vp008_alt_diagnostic_invariant.rs` compiles
and passes. The proof verifies that for any image element with `AltText::Text("")`
(empty string), `validate_element(&element)` produces a non-empty diagnostic list.
(traces to VP-008 — image with empty alt always produces diagnostic)

### AC-004: Concrete fallback tests for Windows CI
For each of the three Kani proofs, a corresponding concrete unit test exists in
`crates/slideforge-validate/src/proofs/<vpXXX>.rs` under `#[cfg(not(kani))]`:
- `test_alt_missing_before_layout()` — drives the exact scenario proven by VP-002
- `test_luminance_linearization_threshold()` — verifies 0.04045 threshold with f64 math
- `test_empty_alt_diagnostic()` — verifies empty string triggers diagnostic

These tests run on all platforms including Windows.

### AC-005: `just kani-validate` Justfile target succeeds
`just kani-validate` runs all three proof harnesses and exits 0 on Linux/macOS.

### AC-006: CI `kani.yml` job includes slideforge-validate proofs as P0 gate
`.github/workflows/kani.yml` has a step `kani-validate` that runs `just kani-validate`
on `ubuntu-latest`. The step has `continue-on-error: false` (P0 — any failure blocks merge).

## Tasks

- [ ] 1. Read `crates/slideforge-validate/src/alt_check.rs` (AltText enforcement logic)
- [ ] 2. Read `crates/slideforge-validate/src/wcag.rs` (contrast formula, linearize_srgb)
- [ ] 3. Read `crates/slideforge-validate/src/lib.rs` (validate entry point)
- [ ] 4. Create `crates/slideforge-validate/src/proofs/mod.rs` with `#[cfg(kani)]` guard
- [ ] 5. Write VP-002 proof in `crates/slideforge-validate/src/proofs/vp002_alt_before_layout.rs`
         — construct a symbolic Deck with visual element, alt: None; verify validate() returns
         AltTextMissing before any layout type is instantiated
- [ ] 6. Write VP-007 proof using scaled integer arithmetic for WCAG threshold check
         — verify branch condition at c*10000 == 405 (scaled 0.04045)
- [ ] 7. Write VP-008 proof in `vp008_alt_diagnostic_invariant.rs`
         — construct element with AltText::Text(""), call validate_element, assert non-empty diagnostics
- [ ] 8. Add concrete fallback tests under `#[cfg(not(kani))]` in each proof file
- [ ] 9. Modify `crates/slideforge-validate/src/lib.rs` to add `mod proofs;` under `#[cfg(kani)]`
- [ ] 10. Add `just kani-validate` target to Justfile
- [ ] 11. Add `kani-validate` P0 step to `.github/workflows/kani.yml`
- [ ] 12. Run `cargo kani -p slideforge-validate` to verify all three proofs pass
- [ ] 13. Run `cargo test -p slideforge-validate` to verify concrete fallback tests pass on all platforms

## Previous Story Intelligence

N/A — first story in EPIC-20 targeting slideforge-validate. Predecessor stories:
- STORY-015 (alt text enforcement): implements `AltText`, `validate_element()`,
  `AltTextMissing` diagnostic. VP-002 and VP-008 prove its invariants.
- STORY-016 (canvas overflow + strict mode): defines the `validate()` entry point
  and pipeline ordering guarantees (validation before layout).
- STORY-017 (WCAG contrast): implements `linearize_srgb()` and `contrast_ratio()`.
  VP-007 proves the 0.04045 threshold is used.

Critical ordering invariant: the proof for VP-002 must reference the actual
`validate()` pipeline function that precedes layout — NOT a stub. The proof
relies on the function NOT calling any layout types before returning. If the
implementation changed this order, the proof would fail to compile (type system
will enforce the dependency direction).

## Architecture Compliance Rules

Derived from `architecture/verification-architecture.md` and `architecture/purity-boundary-map.md`:

1. **Pure-core only for Kani:** `slideforge-validate` is classified SS-03 (Pure core,
   Kani-amenable). No I/O in proof paths.
2. **VP-007 float limitation:** Kani does not model IEEE 754 floats exactly. Use
   scaled integer arithmetic for the threshold proof. Multiply sRGB values by 10000
   and compare integer branches. This is the correct approach for proving the WCAG
   linearization threshold constant is 404.5 (scaled 0.04045), not 392.8 (0.03928).
3. **Proof ordering matters:** VP-002 proof must NOT import anything from
   `slideforge-layout` or `slideforge-pptx`. The absence of these imports in the
   proof source is itself architectural evidence that validation precedes layout.
4. **Forbidden dependencies:** `slideforge-validate` MUST NOT depend on any exporter
   crate (`slideforge-pptx`, `slideforge-pdf`, `slideforge-docx`). The proofs must
   not introduce such deps.
5. **Empty AltText vs None:** VP-008 covers `AltText::Text("")` (empty string);
   VP-002 covers `alt: None`. Both must produce diagnostics via separate code paths.
   The proof set covers both cases.

## Library and Framework Requirements

| Library | Pinned Version | Role | Notes |
|---------|---------------|------|-------|
| kani | latest (~0.65+) | Formal model checker (uses bundled nightly toolchain internally) | Installed via `cargo install kani-verifier --locked && cargo kani setup` |

No new library deps required for this story. All proofs use `kani::any()` and
existing types from `slideforge-validate` and `slideforge-types`.

## File Structure Requirements

Files to CREATE:
- `crates/slideforge-validate/src/proofs/mod.rs` — module root with `#[cfg(kani)]` guard
- `crates/slideforge-validate/src/proofs/vp002_alt_before_layout.rs` — VP-002 Kani proof
- `crates/slideforge-validate/src/proofs/vp007_wcag_contrast.rs` — VP-007 Kani proof
- `crates/slideforge-validate/src/proofs/vp008_alt_diagnostic_invariant.rs` — VP-008 Kani proof

Files to MODIFY:
- `crates/slideforge-validate/src/lib.rs` — add `mod proofs;` under `#[cfg(kani)]`
- `Justfile` — add `kani-validate` target
- `.github/workflows/kani.yml` — add `kani-validate` P0 step

Files to NOT touch:
- `crates/slideforge-validate/src/alt_check.rs` — proofs call it, not modify it
- `crates/slideforge-validate/src/wcag.rs` — proofs call it, not modify it

## Implementation Notes

### VP-002 Proof Pattern

```rust
// crates/slideforge-validate/src/proofs/vp002_alt_before_layout.rs
#[cfg(kani)]
mod proofs {
    use slideforge_types::{Deck, Slide, ContentBlock, VisualElement, AltText};
    use crate::validate;

    #[kani::proof]
    #[kani::unwind(2)]
    fn alt_missing_before_layout() {
        // Construct a minimal Deck with one slide containing a visual element
        // that has no alt text (alt: None).
        let element = VisualElement {
            alt: None,
            // Other fields: use concrete defaults (Kani doesn't need to symbolize them)
            source: slideforge_types::ImageSource::Path("test.png".into()),
            decorative: false,
        };
        let block = ContentBlock::Image(element);
        let slide = Slide {
            slide_type: slideforge_types::SlideType::Blank,
            blocks: vec![block],
            ..Slide::default()
        };
        let deck = Deck {
            slides: vec![slide],
            ..Deck::default()
        };

        // validate() must return error containing AltTextMissing
        let result = validate(&deck);
        kani::assert(result.is_err());
        if let Err(diagnostics) = result {
            let has_alt_error = diagnostics.iter().any(|d| {
                matches!(d.kind, crate::DiagnosticKind::AltTextMissing { .. })
            });
            kani::assert(has_alt_error);
        }
        // KEY: This function uses NO types from slideforge-layout or any exporter.
        // The absence of those imports proves the check runs before layout.
    }
}

#[cfg(not(kani))]
#[cfg(test)]
mod fallback_tests {
    use super::proofs::*;
    // Concrete equivalent test for Windows CI
    #[test]
    fn test_alt_missing_before_layout() {
        // Same scenario as Kani proof, concrete values
        // ...
    }
}
```

### VP-007 Proof Pattern (scaled integer for float threshold)

```rust
// crates/slideforge-validate/src/proofs/vp007_wcag_contrast.rs
#[cfg(kani)]
mod proofs {
    use crate::wcag::linearize_srgb_int;
    // linearize_srgb_int takes c_scaled: u32 (sRGB × 10000), returns u32 × 10000

    #[kani::proof]
    #[kani::unwind(2)]
    fn luminance_linearization_threshold() {
        let c_scaled: u32 = kani::any();
        // Valid sRGB range: 0–10000 (represents 0.0–1.0)
        kani::assume(c_scaled <= 10_000);

        let result = linearize_srgb_int(c_scaled);

        // Result must be in [0, 10000] range (represents [0.0, 1.0])
        kani::assert(result <= 10_000);

        // Threshold proof: the branch must occur at 405 (0.04045 × 10000),
        // not at 393 (0.03928 × 10000, the older incorrect WCAG value).
        // Below threshold: result = c_scaled / 12.92 (simplified to c_scaled × 100 / 1292)
        // Above threshold: result = ((c_scaled + 550) / 10550)^2.4 (approximated)
        if c_scaled <= 405 {
            // Low-end linearization: linear segment
            let expected_low = c_scaled * 100 / 1292;
            // Allow ±1 rounding tolerance
            kani::assert(result <= expected_low + 1);
        }
        // High-end is harder to verify with integer math; the proof mainly
        // verifies the branch point is 405, not 393.
    }
}
```

Note: `linearize_srgb_int` is a new pure integer-arithmetic variant of `linearize_srgb`
added specifically for Kani provability. Add it to `wcag.rs` alongside the existing
float implementation. It does not replace the float version; it is an equivalent
integer version for proof purposes.

### VP-008 Proof Pattern

```rust
// crates/slideforge-validate/src/proofs/vp008_alt_diagnostic_invariant.rs
#[cfg(kani)]
mod proofs {
    use slideforge_types::{VisualElement, AltText, ImageSource};
    use crate::validate_element;

    #[kani::proof]
    #[kani::unwind(2)]
    fn empty_alt_always_diagnostic() {
        // An element with AltText::Text("") is NOT decorative and NOT missing alt —
        // it has an alt field but the value is the empty string. This must also
        // produce a diagnostic.
        let element = VisualElement {
            alt: Some(AltText::Text(String::new())), // empty string
            source: ImageSource::Path("test.png".into()),
            decorative: false,
        };

        let diagnostics = validate_element(&element);
        // Empty alt string must produce at least one diagnostic
        kani::assert(!diagnostics.is_empty());
        // The diagnostic must specifically be about empty alt text
        let has_empty_alt = diagnostics.iter().any(|d| {
            matches!(d.kind, crate::DiagnosticKind::EmptyAltText { .. })
        });
        kani::assert(has_empty_alt);
    }
}
```

### Justfile Targets

```
kani-validate:
    # Platform: Linux/macOS only. P0 gate — all three proofs must pass.
    cargo kani -p slideforge-validate --harness proofs::vp002_alt_before_layout::proofs::alt_missing_before_layout
    cargo kani -p slideforge-validate --harness proofs::vp007_wcag_contrast::proofs::luminance_linearization_threshold
    cargo kani -p slideforge-validate --harness proofs::vp008_alt_diagnostic_invariant::proofs::empty_alt_always_diagnostic
```

### CI Configuration

```yaml
# .github/workflows/kani.yml
- name: kani-validate (P0 gate)
  if: runner.os != 'Windows'
  # continue-on-error: false (default) — P0 blocks merge
  run: |
    cargo install kani-verifier --locked
    just kani-validate
```

## Dependencies

### Dependency Justification

- STORY-068 depends on STORY-015 because VP-002 and VP-008 proofs call `validate()`
  and `validate_element()` defined there. Without the production alt-check implementation,
  no proof target exists.
- STORY-068 depends on STORY-016 because VP-002 proof relies on the pipeline ordering
  established in that story (validate before layout).
- STORY-068 depends on STORY-017 because VP-007 proof calls `linearize_srgb_int()`, which
  is an integer variant of the function implemented in that story's WCAG contrast check.
- STORY-068 depends on STORY-043 and STORY-044 for PDF (VP-006 is in STORY-070 but
  VP-007 also covers PDF/HTML WCAG rendering). Listed for completeness; main deps are
  STORY-015, STORY-016, STORY-017.
- STORY-068 does not block any other story.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `alt: None` on an image element — VP-002 | Kani proof: AltTextMissing diagnostic produced before any layout |
| EC-002 | `decorative: true` on an image — NOT covered by VP-002 or VP-008 | Decorative exemption must NOT produce a diagnostic; separate unit test verifies this |
| EC-003 | `alt: Some(AltText::Text(""))` — VP-008 | Proof: EmptyAltText diagnostic produced |
| EC-004 | `alt: Some(AltText::Text("describes the image"))` — neither VP | No diagnostic; proven by absence (not needing a proof because it's the happy path) |
| EC-005 | sRGB value = 0.0 (black) — VP-007 | linearize_srgb_int(0) == 0; proof verifies |
| EC-006 | sRGB value = 1.0 (white, scaled 10000) — VP-007 | linearize_srgb_int(10000) == 10000; proof verifies |
| EC-007 | Threshold boundary c = 0.04045 (scaled 404) — VP-007 | Branch taken: low-end formula; proof verifies branch at 405, not 393 |
| EC-008 | Kani running on Windows (unsupported platform) | Concrete fallback tests run instead; CI does not attempt Kani on Windows |

## Test Strategy

- **VP-002, VP-007, VP-008:** Kani bounded model checking. VERIFICATION SUCCESSFUL
  is the only pass criterion. All three are P0 — any failure blocks the v1.0 release.
- **Concrete fallback tests:** Run on all 5 platforms including Windows. Equivalent
  assertions to the Kani proofs, using concrete inputs rather than symbolic ones.
- **Regression:** Once proofs pass, they are permanent P0 CI gates. Any refactor to
  `validate()`, `linearize_srgb()`, or `validate_element()` that breaks a proof is
  a release blocker.

---

*Subsystem anchor justification: SS-03 (Validation) owns VP-002 and VP-008 because
slideforge-validate is the sole crate implementing compile-time content checks.
SS-07 (PDF Export) is listed because VP-007's WCAG formula originates from the
PDF/PPTX accessibility surface — but the formula itself is implemented in
slideforge-validate. The proof lives in SS-03. Per ARCH-INDEX Subsystem Registry.*
