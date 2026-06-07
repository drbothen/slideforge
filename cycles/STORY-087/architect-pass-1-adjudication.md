---
document_type: architect-adjudication
story: STORY-087
finding: F-087-P1-001
severity: CRITICAL
decision: Option B — Dedicated Pipeline-Wired Validator (ValueRangeValidator)
status: DECIDED
author: architect
timestamp: 2026-06-06
version: "1.0"
---

# Architect Adjudication — STORY-087 F-087-P1-001
## Value-Range Validation Is Dead Code (TD-VSDD-059 Paper-Fix Pattern)

---

## 1. Problem Statement (Confirmed)

The adversary finding is confirmed accurate. `ProgressBarSlideType::lay_out()` and
`WeightedCompositeSlideType::lay_out()` both contain value-range validation logic
(`value` in [0,100] for `progress_bar`; `weight > 0` and `score` in [0,100] for
`weighted_composite`). However, `slideforge::build()` never calls `lay_out()` on
any registered `SlideType`.

The call chain in `build_inner` is:
```
build_inner → layout::run (crates/slideforge-layout/src/layout.rs:119)
  → region_frames_for(keyword_str, ...) (line 153)
  → [no SlideType::lay_out() call anywhere in the chain]
```

`layout::run` calls `region_frames_for` directly (which is a static keyword→frame
dispatch in `regions.rs`), bypassing the `SlideType` plugin entirely. The value-range
checks in `lay_out()` are therefore unreachable at build time. A deck authored with
`value 101` produces no error. The unit tests only pass because they call `lay_out()`
directly — a textbook TD-VSDD-059 paper-fix.

---

## 2. Option Analysis

### Option A — Route registered SlideTypes through `lay_out()` in `layout::run`

**What it means:** Modify `layout::run` to look up the `SlideType` registry for each
slide, call `slide_type.lay_out(slide, brand, canvas)`, and use the returned
`LaidOutSlide` instead of `region_frames_for`.

**Problems:**

1. **Blast radius is ALL 34 registered slide types.** All 34 types currently rely on
   `region_frames_for` as the frame geometry source. Their `lay_out()` implementations
   are stubs that return minimal frame skeletons (3–7 frames), not the full geometry
   produced by the region-map pipeline (which also runs `thread_media_alt_into_frames`,
   `fill_region_slot_or_append`, `push_bullet_frames`, `layout_shapes`, and
   `run_inline_validation`). Wiring `lay_out()` as the primary path would require
   migrating all 34 types to produce a complete `LaidOutSlide` including the entire
   post-region-map pipeline — a weeks-long refactor.

2. **`lay_out()` signature is semantically incompatible with the pipeline.** It returns
   a self-contained `LaidOutSlide` but lacks the `source_index` that `layout::run`
   assigns, the deck-level `deck_warnings` sink, and the `sections` collection that
   `collect_sections` produces. Merging these would require either changing the trait
   signature (breaking all plugin authors) or duplicating the post-processing passes.

3. **ADR-005 two-IR model violation.** `layout::run` is declared as the sole
   semantic→geometric boundary. Routing layout through per-type `lay_out()` calls
   dissolves that boundary into 34 per-type entry points.

4. **Kani/formal-verification impact.** `layout::run` is a declared pure-function
   candidate (VP-011). Introducing dynamic dispatch via a plugin trait call inside
   `layout::run` adds a non-provable side surface; the Kani proof would lose its
   coverage of the core frame-allocation invariants.

**Verdict: REJECTED.** Option A is architecturally unsound and has prohibitive blast
radius.

### Option B — Dedicated Pipeline-Wired Validator (Stage 5, Validator trait)

**What it means:** Add a new `ValueRangeValidator` (in `slideforge-validate`) that
implements the `Validator` trait, reads `Slide.fields` at Stage 5 (pre-layout, same
stage as `LabelCheckValidator`), and emits a structured `Diagnostic` with the
canonical error code for out-of-range values. Register it in `slideforge::registry.rs`
alongside `LabelCheckValidator`.

**Why this is correct:**

1. **Matches the exact pattern already proven in production.** `LabelCheckValidator`
   reads `Slide.fields["label"]` at Stage 5, emits `E-A11-002`, and is wired through
   `build_inner`'s validator loop. It is fully reachable from `slideforge::build()`.
   `ValueRangeValidator` would use the identical architecture.

2. **Surgical — zero blast radius on the 34 existing slide types.** The existing types
   are not touched. `region_frames_for` continues to serve all 34 types unchanged.
   Only the two STORY-087 types are affected by the new validator behavior.

3. **"Compile error" semantics are correctly satisfied.** BC-1.17.002 PC3 and
   BC-1.17.003 inv 6/7 state that out-of-range values are "a compile error." In the
   slideforge pipeline, a `Diagnostic` with `DiagnosticSeverity::Error` emitted at
   Stage 5 and propagated through the `build_inner` strict gate (`BuildError::
   ValidationFailed`) IS the product's "compile error" mechanism. This is the same
   mechanism that makes missing labels a compile error. The terminology "compile error"
   = a `Diagnostic::Error` that causes `ValidationFailed` in strict mode. A dedicated
   validator satisfies this semantics fully.

4. **ADR-006 plugin-first compliance.** A new `Validator` plugin is exactly the right
   plugin surface for content-rules enforcement. Adding a new `Validator` implementation
   for value-range rules is the prescribed extension path.

5. **Keeps `lay_out()` pure and testable for future wiring.** The `lay_out()` stub
   implementations in `progress_bar.rs` and `weighted_composite.rs` can stay, but
   the value-range validation code MUST be removed from `lay_out()` to eliminate the
   paper-fix pattern (the code must live where it is reachable). Clean separation:
   `lay_out()` = geometry; `Validator` = content rules.

6. **Stage 2b independence.** `Slide.fields` is populated by `eval_deck` (Stage 2a);
   value-range validation reads `slide.fields` directly. No dependency on Stage 2b
   (field-to-block threading). Same pre-layout guarantee as `LabelCheckValidator`.

**Verdict: SELECTED.**

### Option C — Hybrid (skipped)

The hybrid reduces to Option B: the correct hybrid is "keep `lay_out()` for geometry
stubs + add `Validator` for content rules." No additional mechanism is needed.

---

## 3. Decision

**Decision: Option B.**

Add a `ValueRangeValidator` to `slideforge-validate`, register it in
`slideforge::registry.rs`, and remove the duplicate value-range checks from
`SlideType::lay_out()` in `progress_bar.rs` and `weighted_composite.rs`.

---

## 4. Canonical Error Variant / Code

### Error Code

The canonical error code for value-range violations is **`E-VAL-011`** (newly
allocated). This is in the `E-VAL` namespace, which is the existing namespace used
by `validate_fields` in `registry.rs` (E-VAL-101, E-VAL-102, W-VAL-103 are informal
codes in the registry, not yet in the taxonomy). Since the error taxonomy does not
yet contain a `Validation Errors (E-VAL)` section, this adjudication allocates
`E-VAL-011` to cover numeric field out-of-range violations.

Rationale for E-VAL over E-A11: value-range is a data-contract violation, not an
accessibility violation. E-A11-002 is specifically the missing-label accessibility
error. Using E-VAL keeps the namespace semantics clean.

Message format:
```
progress_bar value must be between 0 and 100; got <N>.
```
(This matches BC-1.17.002 PC3 verbatim.)

For `weighted_composite`:
```
weighted_composite components[<idx>].weight must be positive; got <N>.
weighted_composite components[<idx>].score must be between 0 and 100; got <N>.
```

Severity: `DiagnosticSeverity::Error` (broken; strict-mode fatal).

### BC Contract Language Reconciliation

BC-1.17.002 PC3 says "compile error" and later in PC3 says: "The exact enforcement
point is the SlideType trait's `validate_fields(&Slide.fields)` method."

BC-1.17.003 invariants 6/7 say "a compile error with source span."

The phrase "SlideType trait's `validate_fields` method" was written in the BC before
the pipeline integration gap was discovered. It is NOW INCORRECT — there is no
`validate_fields` method on the `SlideType` trait. The CORRECT enforcement point is
the `Validator` plugin surface at Stage 5. This adjudication amends the BCs accordingly
(see Section 6).

---

## 5. Implementation Directive

### 5.1 Files to Create

**`crates/slideforge-validate/src/value_range.rs`** — NEW FILE

Implement `ValueRangeValidator`:

```rust
// BC-1.17.002 PC3 + BC-1.17.003 inv 6/7 enforcement
// Registered as Surface-5 Validator in slideforge::registry.
//
// Error code: E-VAL-011
// Stage: 5 (pre-layout, reads Slide.fields directly)
// Stage-2b independence: guaranteed — reads FieldValue::Literal only.
```

Logic:

- For every slide in `deck.slides`:
  - If `slide.slide_type == "progress_bar"`:
    - Read `slide.fields["value"]` as `FieldValue::Literal(Value::Int(n))`.
    - If absent or wrong type: emit `E-VAL-011` with message
      `"progress_bar value must be an integer; got <actual_type> or absent."`,
      span = `slide.source_span`.
    - If present but `n < 0 || n > 100`: emit `E-VAL-011` with message
      `"progress_bar value must be between 0 and 100; got <n>."`,
      span = `slide.source_span`.
  - If `slide.slide_type == "weighted_composite"`:
    - Read `slide.fields["components"]` as `FieldValue::Literal(Value::List(list))`.
    - For each `Value::Map(comp)` at index `idx`:
      - Check `comp["weight"]`: if absent, or if `Value::Float(f) where f <= 0.0`,
        or if `Value::Int(n) where n <= 0`: emit `E-VAL-011` with message
        `"weighted_composite components[<idx>].weight must be positive; got <actual>."`.
      - Check `comp["score"]`: if absent or not `Value::Int`, emit `E-VAL-011` with
        message `"weighted_composite components[<idx>].score must be an integer; ..."`.
        If `Value::Int(n) where !(0..=100).contains(&n)`: emit `E-VAL-011` with
        message `"weighted_composite components[<idx>].score must be between 0 and 100;
        got <n>."`.
    - ALL errors are accumulated — do NOT bail on first (DI-018). Same collect-all
      pattern as `LabelCheckValidator`'s per-component iteration.

Validator ID: `"value-range"`.

**Error accumulation invariant**: the validator MUST iterate all components and
collect all `E-VAL-011` diagnostics before returning — same as `LabelCheckValidator`'s
`weighted_composite` component loop.

### 5.2 Files to Modify

**`crates/slideforge-validate/src/lib.rs`**
- Add `pub mod value_range;`
- Re-export `ValueRangeValidator` from the crate root (add to `pub use` block).

**`crates/slideforge/src/registry.rs`**
- Import `ValueRangeValidator` from `slideforge_validate`.
- Register: `builder.register_validator(Box::new(ValueRangeValidator));`
  after `LabelCheckValidator` registration (line 113 area).

**`crates/slideforge-plugin-api/src/slide_types/progress_bar.rs`**
- REMOVE the value-range validation code from `lay_out()` (lines 141–161, the
  `value_int` extraction and `!(0..=100).contains(&value_int)` check).
- REMOVE the label check from `lay_out()` (lines 128–137, the `label_ok` block).
- Keep: the three-frame geometry production (the `frames` vec and `Ok(LaidOutSlide{...})`).
- The `lay_out()` method is now geometry-only. All content validation moves to
  the `Validator` surface.
- Update the module docstring to remove the claim "Value range validation is performed
  in `lay_out()`" and replace with "Value range validation is performed by
  `ValueRangeValidator` at Stage 5 (pre-layout). `lay_out()` is geometry-only."

**`crates/slideforge-plugin-api/src/slide_types/weighted_composite.rs`**
- REMOVE the label check from `lay_out()` (lines 138–147, the `label_ok` block and
  its `if !label_ok` early return).
- REMOVE the `components` non-empty check from `lay_out()` (lines 149–165).
- REMOVE the per-component weight/score validation loop from `lay_out()` (lines 172–253).
- Keep: the static 7-frame skeleton geometry production (the `frames` vec and
  `Ok(LaidOutSlide{...})`).
- The `lay_out()` method is now geometry-only. All content validation moves to
  `ValueRangeValidator` (weight/score) and `LabelCheckValidator` (labels).
- Update the module docstring similarly.

**NOTE for implementer:** After removing validation from `lay_out()`, the unit tests
in `registry.rs` (`test_all_31_types_lay_out_returns_ok`) that call `lay_out()` directly
with valid fields MUST still pass — this is correct because a geometry-only `lay_out()`
always returns `Ok` for the frame skeleton regardless of field values.

### 5.3 Tests the Test-Writer Must Add

All tests in `crates/slideforge-validate/src/value_range.rs` (unit tests block):

**A. Unit tests (call `ValueRangeValidator.validate(&deck, &opts)` directly):**

| Test name | Setup | Expected |
|-----------|-------|----------|
| `test_BC_1_17_002_value_in_range_no_error` | `progress_bar` slide with `value: 50` | 0 E-VAL-011 |
| `test_BC_1_17_002_value_zero_boundary` | `progress_bar` with `value: 0` | 0 E-VAL-011 |
| `test_BC_1_17_002_value_100_boundary` | `progress_bar` with `value: 100` | 0 E-VAL-011 |
| `test_BC_1_17_002_value_101_out_of_range` | `progress_bar` with `value: 101` | 1 E-VAL-011 with `"got 101"` |
| `test_BC_1_17_002_value_neg1_out_of_range` | `progress_bar` with `value: -1` | 1 E-VAL-011 with `"got -1"` |
| `test_BC_1_17_002_value_absent` | `progress_bar` with no `value` field | 1 E-VAL-011 |
| `test_BC_1_17_002_value_wrong_type` | `progress_bar` with `value: "fifty"` (Str) | 1 E-VAL-011 |
| `test_BC_1_17_003_weight_positive_ok` | `weighted_composite` with component `weight: 0.5` | 0 E-VAL-011 for weight |
| `test_BC_1_17_003_weight_zero_error` | component `weight: 0` | 1 E-VAL-011 for weight |
| `test_BC_1_17_003_weight_negative_error` | component `weight: -0.5` | 1 E-VAL-011 for weight |
| `test_BC_1_17_003_score_in_range_ok` | component `score: 85` | 0 E-VAL-011 for score |
| `test_BC_1_17_003_score_0_boundary` | component `score: 0` | 0 E-VAL-011 |
| `test_BC_1_17_003_score_100_boundary` | component `score: 100` | 0 E-VAL-011 |
| `test_BC_1_17_003_score_101_error` | component `score: 101` | 1 E-VAL-011 with `"got 101"` |
| `test_BC_1_17_003_score_neg1_error` | component `score: -1` | 1 E-VAL-011 |
| `test_BC_1_17_003_accumulation_multiple_errors` | 2 components, both with `score: 101` | 2 E-VAL-011 (DI-018 accumulation) |
| `test_non_color_coded_slide_not_checked` | `title` slide | 0 E-VAL-011 |
| `test_empty_deck_no_diagnostics` | empty deck | 0 E-VAL-011 |

**B. build()-level integration tests** (in `crates/slideforge/tests/e2e/` or
`crates/slideforge/src/lib.rs` `#[cfg(test)]` block):

These tests call `slideforge::build()` with a DSL string and verify that
`BuildError::ValidationFailed` is returned when out-of-range values are present.
This is the load-bearing gate that proves the validator is REACHABLE from the real
pipeline.

| Test name | DSL input | Expected |
|-----------|-----------|----------|
| `test_build_progress_bar_value_101_is_validation_failed` | `slide progress_bar: title "T" label "L" value 101` | `Err(BuildError::ValidationFailed)` containing at least 1 diag with `code == "E-VAL-011"` |
| `test_build_progress_bar_value_neg1_is_validation_failed` | `slide progress_bar: title "T" label "L" value -1` | Same |
| `test_build_progress_bar_value_50_is_ok` | `slide progress_bar: title "T" label "L" value 50` | `Ok(BuildOutput)` |
| `test_build_weighted_composite_score_101_is_validation_failed` | `slide weighted_composite:` with component `score: 101` | `Err(BuildError::ValidationFailed)` containing `E-VAL-011` |
| `test_build_weighted_composite_weight_zero_is_validation_failed` | component `weight: 0` | Same |

**CRITICAL requirement for the build()-level tests:** These tests MUST NOT call
`lay_out()` directly. They MUST call `slideforge::build()` (or `build_inner` directly
if the test harness exposes it) to prove the validation path is wired end-to-end.

---

## 6. Spec Amendments Required

### 6.1 BC-1.17.002 — version bump to 1.1

**File:** `.factory/specs/behavioral-contracts/BC-1.17.002.md`

**Change:** Amend Postcondition 3 and Postcondition 6.

PC3 (current): "If present but outside [0, 100]: compile error with message: `progress_bar
value must be between 0 and 100; got <value>.`"
PC3 (amended): Add: "This check is performed by `ValueRangeValidator` at Stage 5
(pre-layout), not by `SlideType::lay_out()`. Error code: E-VAL-011."

PC6 (current): "The `value` field validation is performed either by the `SlideType` trait
implementation (at type-registration time) or by a dedicated pre-layout validator. The exact
enforcement point is the SlideType trait's `validate_fields(&Slide.fields)` method."
PC6 (amended, REPLACE ENTIRELY): "The `value` field validation is performed by
`ValueRangeValidator` — a `Validator` plugin registered at Stage 5 (pre-layout) in
`slideforge::registry::register_bundled_plugins`. It reads `Slide.fields["value"]` directly,
before layout. Error code: E-VAL-011. Severity: `DiagnosticSeverity::Error` (strict-mode
fatal). `SlideType::lay_out()` is geometry-only and does NOT perform value-range validation."

Add to Invariants: "Invariant 7: `ValueRangeValidator` is registered at Stage 5. No
validation of `value` occurs in `SlideType::lay_out()`. Tests calling `lay_out()` directly
do NOT exercise the value-range enforcement; only `slideforge::build()` or
`build_inner()`-level tests do."

**Version bump:** `"1.0"` → `"1.1"`. Add to `modified:` array: `"2026-06-06 v1.1 (architect adjudication F-087-P1-001): enforcement point moved from lay_out() to ValueRangeValidator Stage 5; error code E-VAL-011 allocated."`.

### 6.2 BC-1.17.003 — version bump to 1.1

**File:** `.factory/specs/behavioral-contracts/BC-1.17.003.md`

**Change:** Amend Postcondition 3 (component validation) and Invariants 6/7.

PC3 (current): "A missing per-component `label` on ANY component emits `E-A11-002`..."
(PC3 is mostly about labels — leave intact.)

Add new PC3b: "Component `weight` and `score` range validation is performed by
`ValueRangeValidator` at Stage 5 (pre-layout). Error code: E-VAL-011. Severity:
`DiagnosticSeverity::Error`. `SlideType::lay_out()` is geometry-only."

Invariant 6 (current): "`weight` must be positive. A zero or negative weight is a compile
error."
Invariant 6 (amended): Add: "Enforced by `ValueRangeValidator` at Stage 5. Error code:
E-VAL-011."

Invariant 7 (current): "`score` must be in [0, 100] inclusive. Out-of-range values are a
compile error with source span."
Invariant 7 (amended): Add: "Enforced by `ValueRangeValidator` at Stage 5. Error code:
E-VAL-011. Error accumulation: all components' errors are collected before returning
(DI-018)."

Add new Invariant 9: "`ValueRangeValidator` is registered at Stage 5. No weight/score
range validation occurs in `SlideType::lay_out()`. The `lay_out()` method in
`weighted_composite.rs` is geometry-only after this adjudication. Tests calling
`lay_out()` directly do NOT exercise the value-range enforcement."

**Version bump:** `"1.0"` → `"1.1"`. Same `modified:` annotation pattern.

### 6.3 STORY-087 File Structure Requirements

**File:** The STORY-087 story spec (wherever it lives in `.factory/stories/`).

The story's File Structure Requirements section states that value-range validation
is in `lay_out()`. This must be amended to reflect the actual implementation directive:

- Remove the instruction to put value-range checks in `lay_out()`.
- Add: "Value-range validation lives in `crates/slideforge-validate/src/value_range.rs`
  (`ValueRangeValidator`), registered at Stage 5 in `slideforge::registry.rs`."
- Add: "`lay_out()` in `progress_bar.rs` and `weighted_composite.rs` is geometry-only."

### 6.4 Error Taxonomy — version bump, new E-VAL section

**File:** `.factory/specs/prd-supplements/error-taxonomy.md`

Add a new section "Validation Errors (E-VAL)" with:

| Code | Severity | Exit | Message Format | Traces To |
|------|---------|------|---------------|-----------|
| E-VAL-011 | broken | 2 | `<slide_type> <field_path> must be <constraint>; got <value>.` (See variants in BC-1.17.002 PC3, BC-1.17.003 inv 6/7) | BC-1.17.002 PC3, BC-1.17.003 inv 6/7 |

Note: E-VAL-011 is the numeric field range violation code. It is emitted by
`ValueRangeValidator` at Stage 5 (pre-layout). Strict-mode fatal (exit 2).
The codes E-VAL-101, E-VAL-102, W-VAL-103 in `registry.rs::validate_fields` are
informally used but not yet in this taxonomy; they will be formally registered in a
future spec burst.

**Version bump:** `"2.17"` → `"2.18"`.

---

## 7. Architecture Compliance Verification

| Requirement | Met? | Notes |
|-------------|------|-------|
| ADR-006 plugin-first | YES | `ValueRangeValidator` is a `Validator` plugin |
| ADR-005 two-IR model (layout::run is sole boundary) | YES | Validator runs pre-layout on semantic `Deck`; layout::run is untouched |
| ADR-016 validator surface ownership | YES | Validators are owned by `slideforge-validate`; registered in `slideforge::registry` |
| BC-1.17.002 "compile error" semantics | YES | `DiagnosticSeverity::Error` → `BuildError::ValidationFailed` in strict mode |
| BC-1.17.003 "compile error with source span" | YES | Same; `slide.source_span` carried on every diagnostic |
| DI-018 error accumulation | YES | All component errors collected before return |
| Stage-2b independence | YES | Reads `Slide.fields` directly, same as `LabelCheckValidator` |
| TD-VSDD-059 paper-fix closed | YES | Dead code in `lay_out()` is REMOVED; enforcement moves to a reachable path |
| Zero regression on 34 existing types | YES | `region_frames_for` untouched; no `lay_out()` routing change |

---

## 8. Verification — Build()-Level Tests Are the Proof Gate

The implementer self-check gate for this fix is: **at least 2 of the 5 build()-level
integration tests described in Section 5.3B must pass end-to-end** (specifically,
`test_build_progress_bar_value_101_is_validation_failed` and
`test_build_weighted_composite_score_101_is_validation_failed`). These are the
TD-VSDD-059 load-bearing tests — they exercise the real pipeline path.

Unit tests on `ValueRangeValidator.validate()` directly are REQUIRED but INSUFFICIENT
alone; they cannot prove the validator is registered and wired into `build_inner`.

---

## 9. Decision Log

| Date | Agent | Action |
|------|-------|--------|
| 2026-06-06 | architect | Finding F-087-P1-001 confirmed: value-range validation in `lay_out()` is dead code per TD-VSDD-059 |
| 2026-06-06 | architect | Option A (route via `lay_out()` in `layout::run`) rejected: prohibitive blast radius on all 34 types, ADR-005 violation, Kani-amenability regression |
| 2026-06-06 | architect | Option B (dedicated `ValueRangeValidator` at Stage 5) selected: surgical, production-proven pattern (mirrors `LabelCheckValidator`), zero regression |
| 2026-06-06 | architect | Error code E-VAL-011 allocated for numeric field range violations |
| 2026-06-06 | architect | BC-1.17.002 v1.1, BC-1.17.003 v1.1, error-taxonomy v2.18 amendments specified |
| 2026-06-06 | architect | Implementation directive written for implementer and test-writer |
