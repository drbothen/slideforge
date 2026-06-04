---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-017
title: "Color-Coded Label + WCAG Contrast Enforcement"
epic: EPIC-04
wave: 2
points: 5
priority: P0
tdd_mode: strict
status: draft
crate: slideforge-validate
subsystems: [SS-03]
target_module: slideforge-validate
behavioral_contracts:
  - BC-5.01.003
  - BC-5.01.004
  - BC-5.01.005
verification_properties: [VP-007]
nfr_refs: [NFR-012, NFR-013, NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-016
blocks:
  - STORY-039
  - STORY-043
  - STORY-045
  - STORY-046
  - STORY-055
  - STORY-068
estimated_days: 2
---

# STORY-017: Color-Coded Label + WCAG Contrast Enforcement

## Summary

Extend `slideforge-validate` with two accessibility validators: color-coded label
enforcement (BC-5.01.003) and deck lang declaration handling (BC-5.01.004 + BC-5.01.005).
The validators also include the WCAG AA contrast ratio check for color-coded elements.

**Color-coded label enforcement (BC-5.01.003):** Slide types that use color to convey
meaning (`severity_cards`, `status`, `progress_bar`, `weighted_composite`) MUST declare
a `label "..."` field. Missing or blank `label` produces E-A11-002 (blocking in strict
mode). This is independent of the `alt` requirement — color-coded elements need both
`alt` (for the visual element) AND `label` (for the color meaning).

**WCAG contrast check:** For `severity_cards` and `status` elements, the declared
foreground/background color pair is checked against WCAG AA contrast ratios:
- Normal text: ≥ 4.5:1
- Large text (18pt+ or 14pt+ bold): ≥ 3:1

**Lang declaration (BC-5.01.004 + BC-5.01.005):** Missing `lang "..."` in deck
metadata produces E-A11-003 as a cosmetic warning (exit 0). The default "en" is
applied. The lang value is read by all exporters from `deck.metadata.lang` (`DeckMetadata.lang` in the semantic Deck IR) via the `Exporter` trait's `deck: &Deck` parameter.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-5.01.003 | Missing label on color-coded element is compile error | All 4 postconditions + 3 invariants |
| BC-5.01.004 | Missing deck lang declaration produces lint warning; default is "en" | All 4 postconditions + 3 invariants |
| BC-5.01.005 | lang declaration propagates to PPTX Core Properties, PDF /Lang, HTML lang attr | All 5 postconditions + 3 invariants |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4,500 |
| BC files (3 BCs) | ~3,000 |
| STORY-015/016 reference | ~1,500 |
| STORY-001 IR types reference | ~1,000 |
| Target source files to write | ~5,000 |
| Test files | ~3,500 |
| **Total** | **~18,500** |

Agent context budget: 200k tokens. This story is ~9% of budget — within limit.

## Acceptance Criteria

- [ ] **AC-001** — `severity_cards:` block with no `label` field produces E-A11-002:
  `Missing label on color-coded element 'severity_cards' at <file>:<line>:<col>.
  Color alone must not convey meaning. Add label "...".` Build exits 2 in strict mode.
  (traces to BC-5.01.003 postcondition 1, postcondition 2)

- [ ] **AC-002** — `status:`, `progress_bar:`, and `weighted_composite:` elements each
  require `label "..."`. Missing label on any of these produces E-A11-002.
  (traces to BC-5.01.003 postcondition 1)

- [ ] **AC-003** — `label ""` (empty string) on a color-coded element produces E-A11-002
  (same rule as alt text — empty string is not a valid label).
  (traces to BC-5.01.003 invariant 2)

- [ ] **AC-004** — `decorative: true` does NOT exempt color-coded elements from the label
  requirement. A `severity_cards:` element with `decorative: true` but no `label` still
  produces E-A11-002.
  (traces to BC-5.01.003 invariant 1; edge case EC-005)

- [ ] **AC-005** — 3 `status:` elements on one slide, 2 missing `label`: 2× E-A11-002
  accumulated.
  (traces to BC-5.01.003 postcondition 3)

- [ ] **AC-006** — In `--warn-only` mode, E-A11-002 is demoted to a warning. Build
  continues; error-slide placeholder inserted for affected slide; exit 0.
  (traces to BC-5.01.003 postcondition 4)

- [ ] **AC-007** — `severity_cards:` with foreground `#FFFFFF` and background `#FF0000`
  (red): contrast ratio computed as W3C WCAG 2.1 relative luminance formula. If contrast
  < 4.5:1 for normal text (< 3:1 for large text), E-A11-004 warning emitted. This is
  a WARNING (not blocking) — hint to author but does not block build.
  (traces to BC-5.01.003 — WCAG 1.4.1 enforcement)

- [ ] **AC-008** — WCAG contrast ratio algorithm: relative luminance L = 0.2126R + 0.7152G
  + 0.0722B (after sRGB gamma expansion). Contrast ratio = (L_lighter + 0.05) / (L_darker + 0.05).
  Colors declared as hex `#RRGGBB` or as brand palette references. Brand palette colors
  use the color values from `Brand.palette` (brand must be loaded before contrast check).
  (traces to BC-5.01.003 invariant — WCAG 1.4.1 requires color independence)

- [ ] **AC-009** — Deck with no `lang "..."` declaration produces E-A11-003:
  `Missing lang declaration in deck metadata. Defaulting to "en". Screen readers may
  mispronounce non-English content. Add lang "en-US" (or appropriate BCP-47 tag).`
  Exit code 0 (cosmetic severity). Output produced.
  (traces to BC-5.01.004 postcondition 1, postcondition 2, postcondition 3)

- [ ] **AC-010** — E-A11-003 does NOT count as a blocking error. Even in strict mode, a
  deck with ONLY E-A11-003 still produces output (exit 0).
  (traces to BC-5.01.004 postcondition 4; BC-5.01.004 invariant 1)

- [ ] **AC-011** — `lang ""` (empty string `lang` field) produces E-A11-003 and defaults
  to `"en"`.
  (traces to BC-5.01.004 invariant 3)

- [ ] **AC-012** — The resolved lang value (`"en"` if absent, declared value otherwise)
  is stored in `DeckMetadata.lang` as `Some(Arc::from("en"))` before passing to layout
  and exporters. Exporters read `DeckMetadata.lang` for PPTX `dc:language`, PDF `/Lang`,
  HTML `lang` attribute. The validator populates the default; exporters trust it.
  (traces to BC-5.01.005 postcondition 1, postcondition 2, postcondition 3, BC-5.01.005 invariant 2)

- [ ] **AC-013** — `lang "zh-Hant-TW"` propagates unchanged to all output formats. No
  truncation, normalization, or case change.
  (traces to BC-5.01.005 invariant 1)

- [ ] **AC-014** — `cargo test -p slideforge-validate` passes. All tests from STORY-015,
  STORY-016, and this story pass.

- [ ] **AC-015** — `#![forbid(unsafe_code)]`, zero `.unwrap()`, `clippy::pedantic` clean,
  all public items documented.
  (traces to NFR-021, NFR-022, NFR-023, NFR-024)

## Previous Story Intelligence

STORY-016 established:
- `ValidationMode` + `ValidationConfig` structs.
- `error_slide_placeholder()` function for warn-only mode.
- `ValidationError` enum with E-LAY-001/002 variants.

This story adds two more validators to the same crate. Key reuse points:
- `is_blank()` from `src/utils.rs` (STORY-015) — reuse for label empty-string check.
- `ValidationConfig` from STORY-016 — `WcagContrastValidator::validate()` takes the
  same config type.
- `DiagnosticSink` usage pattern established in STORY-015 — same pattern here.

WCAG contrast ratio computation is a pure function with no dependencies. Implement it
in `src/wcag.rs` as a standalone module. The formula is from W3C WCAG 2.1 §1.4.3.

## Architecture Compliance Rules

1. **Pure Core (SS-03):** WCAG contrast calculation is pure arithmetic. No I/O.
2. **Color-coded element type list:** The list of slide types requiring `label "..."` is
   maintained in a `const` array in `src/label_check.rs`. New slide types that use color
   semantically MUST be added to this array. The const is: `const COLOR_CODED_TYPES: &[&str]`
   `= &["severity_cards", "status", "progress_bar", "weighted_composite"];`
3. **Lang default is injected by validator:** The `LangValidator` sets `metadata.lang`
   to `Some(Arc::from("en"))` when absent, before the layout engine runs. This is the
   single write point for the default. Exporters MUST NOT hardcode any lang value.
4. **WCAG contrast is a WARNING (not blocking):** E-A11-004 is severity Warning. Only
   E-A11-001 and E-A11-002 are blocking validation errors. Contrast checks are advisory.
5. **Forbidden dependencies:** Same as STORY-015/016 — no effectful crates.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `slideforge-types` | workspace | `Deck`, `Slide`, `ContentBlock`, `DeckMetadata`, `SourceSpan` |
| `slideforge-plugin-api` | workspace | `Validator` trait |
| `thiserror` | `=2.0.18` | Extends `ValidationError` |
| `miette` | workspace (`"7"` with `fancy` feature — resolves to latest 7.x, currently 7.6.0) | Diagnostic impls |

Dev dependencies: `insta` (compatible)

## File Structure Requirements

Files to extend (from STORY-016 baseline):

```
crates/slideforge-validate/
├── src/
│   └── error.rs    # EXTEND: add MissingLabel (E-A11-002), LowContrast (E-A11-004),
│                   #         MissingLang (E-A11-003) variants
```

Files to create (new in this story):

```
crates/slideforge-validate/
├── src/
│   ├── label_check.rs      # LabelCheckValidator + COLOR_CODED_TYPES const
│   ├── lang_validator.rs   # LangValidator: sets default lang; emits E-A11-003
│   └── wcag.rs             # WCAG contrast ratio computation (pure functions)
```

## Tasks

1. **Extend `src/error.rs`** — add `MissingLabel`, `LowContrast` (E-A11-004, severity Warning), `MissingLang` (E-A11-003, severity Cosmetic) variants. (20 min)
2. **Write `src/wcag.rs`** — pure WCAG contrast functions:
   - `fn srgb_component_to_linear(c: f64) -> f64` (gamma expansion: `c/12.92` if c ≤ 0.04045 else `((c + 0.055) / 1.055).powf(2.4)`)
   - `fn relative_luminance(r: u8, g: u8, b: u8) -> f64` (L = 0.2126R + 0.7152G + 0.0722B after gamma)
   - `fn contrast_ratio(l1: f64, l2: f64) -> f64` ((lighter + 0.05) / (darker + 0.05))
   - `fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)>` (parse `#RRGGBB`)
   - `fn wcag_aa_passes(fg: (u8, u8, u8), bg: (u8, u8, u8), large_text: bool) -> bool` (threshold: 4.5 normal, 3.0 large)
   (45 min)
3. **Write `src/label_check.rs`** — `LabelCheckValidator` implementing `Validator` trait:
   - Walk slides for `ContentBlock` variants matching `COLOR_CODED_TYPES`.
   - For each: check `label` field — None or blank → push E-A11-002.
   - Also run `WcagContrastValidator::check_element()` on the same element if color
     fields are declared. Push E-A11-004 (warning) if contrast fails.
   (45 min)
4. **Write `src/lang_validator.rs`** — `LangValidator` implementing `Validator` trait:
   - Check `deck.metadata.lang`.
   - If `None` or blank: push E-A11-003 (cosmetic); set `metadata.lang = Some(Arc::from("en"))`.
   - If present and valid BCP-47 (non-empty, non-blank): no action.
   (20 min)
5. **Write unit and integration tests**. See Test Strategy. (50 min)
6. **Run `cargo test -p slideforge-validate`** and confirm pass. (10 min)

## Test Strategy

### Unit tests (`#[cfg(test)] mod tests` in `wcag.rs`)

- `test_srgb_to_linear_low()`: `srgb_component_to_linear(0.0) == 0.0`.
- `test_srgb_to_linear_mid()`: `srgb_component_to_linear(0.5)` ≈ 0.2140 (within 1e-4).
- `test_srgb_to_linear_max()`: `srgb_component_to_linear(1.0) == 1.0`.
- `test_luminance_white()`: `relative_luminance(255, 255, 255)` ≈ 1.0.
- `test_luminance_black()`: `relative_luminance(0, 0, 0)` == 0.0.
- `test_contrast_black_white()`: `contrast_ratio(1.0, 0.0)` == 21.0 (maximum contrast).
- `test_contrast_ratio_4_5()`: construct fg/bg that produces exactly 4.5:1; `wcag_aa_passes` returns true.
- `test_contrast_ratio_4_4()`: contrast 4.4:1; `wcag_aa_passes` returns false (normal text).
- `test_contrast_large_text_3_0()`: contrast 3.0:1; `wcag_aa_passes(large_text: true)` returns true.
- `test_parse_hex_valid()`: `parse_hex_color("#FF0000")` == `Some((255, 0, 0))`.
- `test_parse_hex_invalid()`: `parse_hex_color("red")` == `None`.

### Unit tests (`#[cfg(test)] mod tests` in `label_check.rs`)

- `test_severity_cards_missing_label()`: severity_cards without label → E-A11-002.
- `test_status_missing_label()`: status without label → E-A11-002.
- `test_progress_bar_missing_label()`: progress_bar without label → E-A11-002.
- `test_weighted_composite_missing_label()`: weighted_composite without label → E-A11-002.
- `test_label_present_no_error()`: severity_cards with `label "HIGH: action required"` → 0 errors.
- `test_label_empty_string()`: `label ""` → E-A11-002.
- `test_label_whitespace()`: `label "  "` → E-A11-002.
- `test_decorative_does_not_exempt_label()`: severity_cards with `decorative: true` but no label → E-A11-002.
- `test_multiple_missing_labels()`: 3 status elements, 2 missing → 2× E-A11-002.
- `test_low_contrast_warning()`: severity_cards with fg/bg producing 3.0:1 → E-A11-004 warning (not blocking).
- `test_adequate_contrast_no_warning()`: fg/bg at 4.5:1 → 0 E-A11-004.

### Unit tests (`#[cfg(test)] mod tests` in `lang_validator.rs`)

- `test_missing_lang_produces_warning()`: deck with no `lang` → E-A11-003; metadata.lang = Some("en").
- `test_lang_present_no_warning()`: `lang "en-US"` → 0 E-A11-003; metadata.lang = Some("en-US").
- `test_lang_empty_string()`: `lang ""` → E-A11-003; defaults to "en".
- `test_lang_not_blocking()`: deck with only E-A11-003 → severity Cosmetic; output produced.
- `test_lang_zh_hant_tw_unchanged()`: `lang "zh-Hant-TW"` → metadata.lang = Some("zh-Hant-TW") (unchanged).

### Integration test

- `test_full_validation_pipeline()`: deck with all three issues (missing alt, missing label, missing lang) → 3 errors of correct codes; correct exit codes.

## Dependencies

**Depends on:** STORY-016 (ValidationMode, ValidationConfig, ValidationError enum extension point, utils::is_blank).

**Blocks:**
- STORY-039 (PPTX accessibility metadata — reads `DeckMetadata.lang` and label values from IR)
- STORY-043 (PDF export — reads lang for `/Lang` entry)
- STORY-045 (PDF/UA-1 CI gate — lang is required for veraPDF compliance)
- STORY-046 (HTML export — reads `DeckMetadata.lang` for `<html lang="...">`)
- STORY-055 (CLI build command — uses full validation pipeline including lang default injection)
- STORY-068 (Kani proofs for validate — includes proofs VP-007 WCAG formula, VP-008 alt invariant)

### Dependency Anchor Justifications

- SS-03 owns this story's scope because SS-03 is the Validation subsystem.
- STORY-017 depends on STORY-016 because it extends the `slideforge-validate` crate and
  reuses `ValidationConfig` and `ValidationError` established in STORY-016.
- STORY-017 blocks STORY-039/043/046 because those exporters read `DeckMetadata.lang`
  which is guaranteed to be non-None after this story's `LangValidator` runs.

## Implementation Notes

### WCAG 2.1 Contrast Ratio Algorithm (W3C Reference)

The algorithm is specified in WCAG 2.1 §1.4.3 (Contrast Minimum) and the associated
technique G18. The implementation in `src/wcag.rs` must match this specification exactly.

Reference values for test verification:
- Black (#000000) vs White (#FFFFFF): contrast ratio = 21.0:1
- Navy (#003366) vs White (#FFFFFF): contrast ratio ≈ 9.0:1 (varies with exact hex)
- Red (#FF0000) vs White (#FFFFFF): contrast ratio ≈ 4.0:1 (fails AA for normal text)
- Dark red (#CC0000) vs White (#FFFFFF): contrast ratio ≈ 5.9:1 (passes AA)

The sRGB gamma expansion formula:
- If `c / 255.0 <= 0.04045`: `linear = (c / 255.0) / 12.92`
- Else: `linear = ((c / 255.0 + 0.055) / 1.055).powf(2.4)`

Relative luminance: `L = 0.2126 * R_linear + 0.7152 * G_linear + 0.0722 * B_linear`

Contrast ratio: `(max(L1, L2) + 0.05) / (min(L1, L2) + 0.05)`

### Color-Coded Slide Types Register

```rust
/// Slide types that use color to convey meaning and therefore require
/// a `label "..."` field (WCAG 1.4.1: Use of Color).
/// When adding a new slide type that uses color semantically, add it here.
pub const COLOR_CODED_TYPES: &[&str] = &[
    "severity_cards",
    "status",
    "progress_bar",
    "weighted_composite",
];
```

### Lang Default Injection

The `LangValidator` modifies `DeckMetadata.lang` in-place to inject the default `"en"`
when absent. This is the ONLY place where the default is injected. After `LangValidator`
runs, all downstream code can treat `metadata.lang` as `Some("...")` — never `None`.

Implementation note: `DeckMetadata` is in `slideforge-types`. To allow the validator
to write to it, the `validate()` method must take `&mut Deck`. Update the `Validator`
trait signature if needed, or use an interior mutability pattern with `RefCell` (prefer
`&mut` parameter for clarity).

If the `Validator` trait is `validate(&self, deck: &Deck, sink: &mut DiagnosticSink)`,
the lang default injection cannot happen in the trait method. In that case, implement
it as a separate `fn inject_lang_default(deck: &mut Deck, sink: &mut DiagnosticSink)`
called before the trait dispatch loop. The `LangValidator` struct does the error
emission; the injection is a separate pre-processing step.

### Contrast Check: When Brand Colors Are Not Yet Loaded

In Wave 2, the brand system is not yet built (STORY-022/023 are in Wave 3). For Wave 2,
the contrast check only applies when colors are declared as hex strings in the `.sf`
source (e.g., `color "#FF0000"`). When colors are brand palette references
(`color brand.accent1`), skip the contrast check in this story — it requires the
brand to be loaded. The test coverage for brand-reference contrast is deferred to
STORY-039 (PPTX export integration).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `severity_cards` with `label ""` (empty) | E-A11-002: empty label invalid |
| EC-002 | `status` with `label "  "` (whitespace) | E-A11-002: whitespace-only invalid |
| EC-003 | `progress_bar` with both valid label and adequate contrast | No errors |
| EC-004 | Multiple `severity_cards` on slide, one missing label | E-A11-002 per missing label; count = violations |
| EC-005 | `severity_cards` with `decorative: true` but no label | E-A11-002 (decorative does NOT exempt from label) |
| EC-006 | Deck with `lang "en-US"` | No E-A11-003; lang propagates unchanged |
| EC-007 | Deck with `lang ""` | E-A11-003; defaults to "en" |
| EC-008 | Contrast check on brand palette color | Skip contrast check in Wave 2 (brand not loaded); deferred to STORY-039 |
| EC-009 | Low contrast + missing label on same element | Both E-A11-002 (blocking) and E-A11-004 (warning) emitted |
