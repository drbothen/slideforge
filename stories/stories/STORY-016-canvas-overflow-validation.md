---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-016
title: "Canvas Overflow + Zero-Slide + Strict/Warn-Only Mode"
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
  - BC-3.03.001
  - BC-3.03.002
  - BC-3.03.003
  - BC-3.03.004
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-015
blocks:
  - STORY-017
  - STORY-026
  - STORY-055
  - STORY-068
estimated_days: 2
---

# STORY-016: Canvas Overflow + Zero-Slide + Strict/Warn-Only Mode

## Summary

Extend `slideforge-validate` with three content validators: canvas overflow detection
(BC-3.03.001), the strict-mode no-output gate (BC-3.03.002), zero-slide deck detection
(BC-3.03.004), and the warn-only error-slide placeholder system (BC-3.03.003).

Canvas overflow: the validator estimates content height using font metrics from the brand
system (or fallback metrics when brand is not yet loaded) and emits E-LAY-001 when a
slide's content exceeds the allocated placeholder height. This is a WARNING (not blocking
by default); `--strict-overflow` promotes it to blocking.

Zero-slide deck: a `Deck` IR with zero slides after evaluation emits E-LAY-002 (always
blocking; `--warn-only` does not demote it).

Strict-mode gate: when any blocking validation error is present (E-A11-001, E-A11-002,
E-LAY-002, E-EVL-* errors), the build produces no output. This gate is implemented in
the CLI (STORY-055) — the validator emits errors; the CLI makes the output decision.
This story implements the validator side: emitting the right error codes at the right
severity levels.

This story also defines the `ValidationMode` struct (strict vs. warn-only) used by all
validators in this crate.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-3.03.001 | Canvas overflow produces CanvasOverflow warning with EMU estimate | All 5 postconditions + 3 invariants |
| BC-3.03.002 | Strict mode produces no output on validation error | All 5 postconditions + 3 invariants |
| BC-3.03.003 | Warn-only mode renders error-slide placeholders and continues | All 5 postconditions + 3 invariants |
| BC-3.03.004 | Zero-slide deck produces validation error | All 3 postconditions + 3 invariants |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4,500 |
| BC files (4 BCs) | ~4,000 |
| STORY-015 structure reference | ~1,500 |
| STORY-001 IR types reference | ~1,500 |
| Target source files to write | ~5,000 |
| Test files | ~3,500 |
| **Total** | **~20,000** |

Agent context budget: 200k tokens. This story is ~10% of budget — within limit.

## Acceptance Criteria

- [ ] **AC-001** — A `slide content:` with 20 bullet points where the estimated content
  height exceeds the body placeholder EMU height emits E-LAY-001:
  `CanvasOverflow: slide '<title>' field 'bullets' overflows by ~<N> EMU (~<M>pt).
  Consider reducing content or font size.` The warning includes file:line:col reference.
  (traces to BC-3.03.001 postcondition 1, postcondition 2)

- [ ] **AC-002** — In default (strict) mode without `--strict-overflow`, E-LAY-001 is a
  WARNING. Output is still written. Exit code 0 (unless other blocking errors).
  (traces to BC-3.03.001 postcondition 3, postcondition 4; BC-3.03.001 invariant 1)

- [ ] **AC-003** — With `--strict-overflow` flag: E-LAY-001 is promoted to a blocking error.
  No output written. Exit code 2.
  (traces to BC-3.03.001 postcondition 5)

- [ ] **AC-004** — `--strict-overflow` + `--warn-only` flags compose: overflow remains
  fatal (E-LAY-001 as error, exit 2); all other validation errors are demoted to warnings.
  (traces to BC-3.03.001 edge case EC-003)

- [ ] **AC-005** — Deck with a blocking validation error (e.g., E-A11-001 from STORY-015)
  in strict mode (default): zero output files written. All errors reported. Exit code 2.
  The output directory is NOT created if it did not exist.
  (traces to BC-3.03.002 postcondition 1, postcondition 2, postcondition 4)

- [ ] **AC-006** — In strict mode, the all-or-nothing invariant: if building both PPTX and
  DOCX and there is a validation error, NEITHER file is written.
  (traces to BC-3.03.002 invariant 1)

- [ ] **AC-007** — In `--warn-only` mode with E-A11-001: warning emitted; error-slide
  placeholder inserted at the affected slide's position; all non-erroring slides rendered
  normally; output written; exit 0.
  (traces to BC-3.03.003 postcondition 1, postcondition 2, postcondition 4, postcondition 5)

- [ ] **AC-008** — Error-slide placeholder preserves slide count and order: a broken slide
  at position N is replaced by an error-slide at position N, not skipped.
  (traces to BC-3.03.003 invariant 1)

- [ ] **AC-009** — Parse errors (E-PAR-*) remain fatal in `--warn-only` mode. `--warn-only`
  does not demote parse errors to warnings.
  (traces to BC-3.03.003 invariant 3)

- [ ] **AC-010** — `deck.sf` with only `vars:` and no slides emits E-LAY-002:
  `Zero-slide deck: no slide blocks found in 'deck.sf'. A deck must contain at least one slide.`
  No output. Exit code 2.
  (traces to BC-3.03.004 postcondition 1, postcondition 2, postcondition 3)

- [ ] **AC-011** — `deck.sf` with one slide where `@if false` wraps the only slide: after
  evaluation, Deck IR has 0 slides → E-LAY-002 emitted. Exit code 2.
  (traces to BC-3.03.004 invariant 2)

- [ ] **AC-012** — E-LAY-002 is NOT demoted by `--warn-only`. A zero-slide deck always
  produces no output and exit 2.
  (traces to BC-3.03.004 invariant 3)

- [ ] **AC-013** — Multiple slides all overflow: all E-LAY-001 warnings accumulated; all
  output produced; warnings listed at end. (DEC-013)
  (traces to BC-3.03.001 edge case EC-001)

- [ ] **AC-014** — `cargo test -p slideforge-validate` passes with all tests from
  STORY-015 and new tests from this story.

- [ ] **AC-015** — `#![forbid(unsafe_code)]`, zero `.unwrap()`, `clippy::pedantic` clean,
  all public items documented.
  (traces to NFR-021, NFR-022, NFR-023, NFR-024)

## Previous Story Intelligence

STORY-015 created the `slideforge-validate` crate with:
- `ValidationError` enum (extend with `CanvasOverflow`, `ZeroSlide` variants in this story)
- `DiagnosticSink` integration
- `AltTextValidator` implementing `Validator` trait

This story extends the same crate structure. Key design constraint: the validator
itself does not make the strict/warn-only decision — it emits errors with severity
levels; the CLI (STORY-055) reads the severity map and decides whether to write output.
The `ValidationMode` struct in this story encodes the current mode and is passed to
validators so they can set error severity correctly.

## Architecture Compliance Rules

1. **Pure Core (SS-03):** All validators in `slideforge-validate` are pure in-memory
   functions. Canvas overflow estimation uses font metrics passed in via `ValidationConfig`,
   not filesystem access.
2. **Validator pipeline position:** Validators run on `Deck` IR (after eval, before layout).
   Canvas overflow is an ESTIMATE based on character counts and default font metrics —
   it is NOT the authoritative layout engine computation. The authoritative overflow
   detection is in the layout engine (SS-05 / STORY-026). This story's overflow validator
   is a fast compile-time heuristic.
3. **Error-slide placeholder:** The placeholder `Slide` object is constructed by the
   validator and inserted into the `Deck` IR when in warn-only mode. It uses a special
   `slide_type: Arc::from("error-placeholder")` that exporters recognize.
4. **Font metrics for overflow estimation:** Use a `FontMetrics` struct with hardcoded
   fallback values (11pt body, 2 lines × line-height = 1.3 × font-size in EMU). The
   brand system (STORY-022/023) will provide real font metrics when available. Default
   fallback: body text line height = `Emu::from_points(11.0 * 1.3)`.
5. **Forbidden dependencies:** Same as STORY-015 — `slideforge-validate` MUST NOT
   depend on effectful crates.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `slideforge-types` | workspace | `Deck`, `Slide`, `ContentBlock`, `Emu`, `SourceSpan` |
| `slideforge-plugin-api` | workspace | `Validator` trait |
| `thiserror` | `=2.0.18` | Extends `ValidationError` from STORY-015 |
| `miette` | workspace (`"7"` with `fancy` feature — resolves to latest 7.x, currently 7.6.0) | Diagnostic impls |

Dev dependencies: `insta` (compatible)

## File Structure Requirements

Files to extend (from STORY-015 baseline):

```
crates/slideforge-validate/
├── src/
│   ├── error.rs          # EXTEND: add CanvasOverflow and ZeroSlide variants
│   └── alt_text.rs       # READ-ONLY: established in STORY-015
```

Files to create (new in this story):

```
crates/slideforge-validate/
├── src/
│   ├── canvas_overflow.rs  # CanvasOverflowValidator: Validator impl
│   ├── zero_slide.rs       # ZeroSlideValidator: Validator impl
│   ├── mode.rs             # ValidationMode enum (Strict, WarnOnly) + ValidationConfig
│   └── error_slide.rs      # error_slide_placeholder(error: &ValidationError) -> Slide
```

## Tasks

1. **Extend `src/error.rs`** — add `CanvasOverflow { slide_title, field, overflow_emu: Emu, span }` (E-LAY-001, severity Warning) and `ZeroSlide { file, span }` (E-LAY-002, severity Error). (25 min)
2. **Write `src/mode.rs`** — `ValidationMode` enum with `Strict` and `WarnOnly` variants; `ValidationConfig { mode: ValidationMode, strict_overflow: bool, large_deck_warn_threshold: usize }`. (20 min)
3. **Write `src/error_slide.rs`** — `pub fn error_slide_placeholder(error: &ValidationError, position: u32) -> Slide`. Creates a `Slide` with `slide_type: Arc::from("error-placeholder")`, `title` field = `"Error: {error_code}"`, and `body` field = the formatted error message. (30 min)
4. **Write `src/canvas_overflow.rs`** — `CanvasOverflowValidator`:
   - `validate(&self, deck: &Deck, config: &ValidationConfig, sink: &mut DiagnosticSink)`.
   - Walk each slide; for each `ContentBlock::Bullets(items)`: estimate height as
     `items.len() * line_height_emu` where `line_height_emu = Emu::from_points(11.0 * 1.3)`.
   - Standard body placeholder height: `Emu(4_343_400)` (4.74 inches, standard body area).
   - If estimated height > placeholder height: emit E-LAY-001 with overflow amount.
   - If `config.strict_overflow`: set error severity to Error (blocking); else Warning.
   (60 min)
5. **Write `src/zero_slide.rs`** — `ZeroSlideValidator`:
   - If `deck.slides.is_empty()`: emit E-LAY-002 (always Error severity, never demoted).
   (20 min)
6. **Write unit and integration tests**. See Test Strategy. (50 min)
7. **Run `cargo test -p slideforge-validate`** and confirm all tests pass. (10 min)

## Test Strategy

### Unit tests (`#[cfg(test)] mod tests` in `canvas_overflow.rs`)

- `test_no_overflow_no_warning()`: slide with 5 bullets (fits) → 0 E-LAY-001.
- `test_overflow_20_bullets()`: slide with 20 bullets (overflows estimate) → 1 E-LAY-001 with positive overflow_emu.
- `test_overflow_is_warning_not_error()`: overflow without `--strict-overflow` → severity = Warning; output still produced.
- `test_strict_overflow_promotes_to_error()`: overflow with `strict_overflow: true` → severity = Error; no output.
- `test_strict_overflow_plus_warn_only()`: `strict_overflow: true` + `warn_only: true` → overflow remains Error; exit 2.
- `test_overflow_multiple_slides()`: 3 slides all overflow → 3 E-LAY-001 warnings.
- `test_overflow_excluded_slide_not_checked()`: `@if`-excluded slide → not in Deck IR → not checked.

### Unit tests (`#[cfg(test)] mod tests` in `zero_slide.rs`)

- `test_zero_slide_deck()`: empty `deck.slides` → 1 E-LAY-002.
- `test_single_slide_deck_no_error()`: deck with 1 slide → 0 E-LAY-002.
- `test_zero_slide_after_all_if_excluded()`: Deck IR has 0 slides (all @if-excluded in eval stage) → E-LAY-002.
- `test_zero_slide_warn_only_still_blocks()`: `warn_only: true` + zero slides → E-LAY-002 still Error severity.

### Integration tests (`tests/`)

- `test_strict_mode_no_output_on_alt_error()`: build with E-A11-001; strict mode → output dir not created.
- `test_warn_only_output_with_error_slide()`: build with E-A11-001; warn-only → output contains error-slide at position N.
- `test_parse_error_fatal_in_warn_only()`: parse error + warn-only → no output (parse errors always fatal).

## Dependencies

**Depends on:** STORY-015 (slideforge-validate crate structure, ValidationError, AltTextValidator).

**Blocks:**
- STORY-017 (WCAG contrast validator — extends same crate)
- STORY-026 (layout engine — receives Deck IR after validation; validation mode gate is before layout)
- STORY-055 (CLI build command — reads ValidationMode from this story's config)
- STORY-068 (Kani proofs for validate — includes proofs on zero-slide invariant VP-002)

### Dependency Anchor Justifications

- SS-03 owns this story's scope because SS-03 is the Validation subsystem.
- STORY-016 depends on STORY-015 because this story extends the `slideforge-validate`
  crate and the `ValidationError` enum established in STORY-015.
- STORY-016 blocks STORY-017 because STORY-017 adds WCAG validators to the same crate,
  and the `ValidationMode` struct from this story is a shared dependency.
- STORY-016 blocks STORY-026 because the layout engine (STORY-026) runs AFTER
  validation; it consumes a Deck IR that has passed (or been demoted in warn-only mode).

## Implementation Notes

### Canvas Overflow Estimation Algorithm

This is a HEURISTIC estimator, not an exact layout computation. The heuristic:

```
line_height = Emu::from_points(11.0 * 1.3)  // 11pt body × 1.3 line-height = 14.3pt

For slide with N bullet items:
  estimated_height = N * line_height
  body_placeholder_height = Emu(4_343_400)  // 4.74 inches standard body
  if estimated_height > body_placeholder_height:
      overflow_emu = estimated_height - body_placeholder_height
      emit E-LAY-001 with overflow_emu
```

The actual layout engine (STORY-026) will do precise text flow computation. The
heuristic is intentionally conservative to catch obvious overflows early.

### Error-Slide Placeholder Design

The error-slide placeholder is a `Slide` with:
- `slide_type: Arc::from("__error_placeholder__")` (internal type, not user-visible)
- `fields["title"] = FieldValue::Literal(Value::Str("⚠ Slide Error".into()))`
- `fields["error_code"] = FieldValue::Literal(Value::Str(error.code().into()))`
- `fields["error_message"] = FieldValue::Literal(Value::Str(error.to_string().into()))`
- `fields["source_location"] = FieldValue::Literal(Value::Str(span_to_string.into()))`

Exporters (PPTX, PDF, HTML) recognize `__error_placeholder__` and render a visually
prominent error card (red background, white text with error details).

### ValidationMode Design Note

The `ValidationMode` is determined by the CLI flags and passed into the validation
pipeline. Validators themselves do not access `std::env` or parse CLI args. The mode
is always explicit:

```rust
let config = ValidationConfig {
    mode: if warn_only { ValidationMode::WarnOnly } else { ValidationMode::Strict },
    strict_overflow: cli_flags.strict_overflow,
    large_deck_warn_threshold: 500,
};
```

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Overflow in watch mode | Warning shown in CLI; web preview overlay shows warning; rendering continues |
| EC-002 | Overflow on `@if`-excluded slide | Excluded slide not in Deck IR → not checked → no E-LAY-001 |
| EC-003 | `--strict-overflow` + `--warn-only` | Overflow remains fatal (exit 2); other errors are warnings; no E-CFG-003 |
| EC-004 | Zero-slide + other errors | All errors accumulated; E-LAY-002 is always Error; exit 2 |
| EC-005 | All slides are error-slide placeholders in warn-only mode | All N slides are placeholders; exit 0 |
| EC-006 | Empty output dir before strict-mode error | Output dir NOT created (not even an empty directory) |
| EC-007 | Both PPTX and DOCX requested; one validation error | Neither file written (all-or-nothing, BC-3.03.002 invariant 1) |
