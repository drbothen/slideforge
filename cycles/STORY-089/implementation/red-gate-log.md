# Red Gate Log — STORY-089

**Date:** 2026-06-07
**Test Writer:** vsdd-factory:test-writer (Claude Sonnet 4.6)
**Story:** STORY-089 — Field-Value Type Validation: FieldSchemaValidator Pipeline Wiring

---

## Red Gate Summary

| Test | File | Status | Reason |
|------|------|--------|--------|
| `test_BC_1_18_001_ac009_strict_progress_bar_str_value_returns_e_val_104` | `crates/slideforge/tests/e2e/story_089_field_schema.rs` | **RED** | E-VAL-104 absent; only E-VAL-011 present (FieldSchemaValidator not wired) |
| `test_BC_1_18_001_ac010_warn_only_progress_bar_str_value_returns_ok` | `crates/slideforge/tests/e2e/story_089_field_schema.rs` | GUARD (passes trivially) | No FieldSchemaValidator → Ok for wrong reason; guard documents post-wiring contract |
| `test_BC_1_18_001_ac009_positive_control_valid_int_value_strict_ok` | `crates/slideforge/tests/e2e/story_089_field_schema.rs` | GUARD (passes) | Guards false-positive after wiring |
| `test_BC_1_18_001_ac009_regression_chart_no_data_strict_ok` | `crates/slideforge/tests/e2e/story_089_field_schema.rs` | GUARD (passes trivially) | Documents architect's chart.data → optional decision |
| AC-018 compile_fail doctest on `FieldDef` | `crates/slideforge-plugin-api/src/traits/slide_type.rs` (line 166) | PASSES | `#[non_exhaustive]` already applied; artifact documents correct behavior |

---

## Red Gate Evidence (AC-009)

Command run:
```
cargo nextest run -p slideforge -E 'test(story_089_field_schema)'
```

Output:
```
TRY 2 FAIL  slideforge::e2e_tests
  e2e_story_089_field_schema::test_BC_1_18_001_ac009_strict_progress_bar_str_value_returns_e_val_104

thread panicked at:
AC-009 RED GATE: ValidationFailed.diagnostics must contain at least one E-VAL-104
diagnostic (T1 type-mismatch: expected integer, got string — from FieldSchemaValidator
calling validate_fields).
Pre-wiring: only E-VAL-011 present (ValueRangeValidator) — FieldSchemaValidator is not
registered, validate_fields() is dead code.
Post-wiring: both E-VAL-011 and E-VAL-104 must appear.
Got diagnostic codes: ["E-VAL-011"]

Summary: 4 tests run: 3 passed, 1 failed, 103 skipped
```

---

## Test File: `crates/slideforge/tests/e2e/story_089_field_schema.rs`

Added to: `crates/slideforge/tests/e2e_tests.rs` via `#[path = "e2e/story_089_field_schema.rs"] mod e2e_story_089_field_schema;`

### Test inventory:

1. **`test_BC_1_18_001_ac009_strict_progress_bar_str_value_returns_e_val_104`** — PRIMARY RED GATE
   - Fixture: `story-089-progress-bar-str-value.sf` (progress_bar with `value "fifty"` — Str on Int field)
   - Build options: format=pptx, strict=true
   - Asserts: `Err(ValidationFailed)` AND diagnostics contain E-VAL-104
   - Status: **FAILS** — only E-VAL-011 present, E-VAL-104 absent (FieldSchemaValidator not registered)

2. **`test_BC_1_18_001_ac010_warn_only_progress_bar_str_value_returns_ok`** — GUARD
   - Same fixture, strict=false
   - Asserts: `Ok(BuildOutput)` with non-empty bytes and extension=="pptx"
   - Status: PASSES (trivially pre-wiring: no validator → Ok; correctly post-wiring: strict gate not applied)
   - Note: `BuildOutput` has no `diagnostics` field; warn-only E-VAL-104 diagnostics are NOT surfaced to callers.

3. **`test_BC_1_18_001_ac009_positive_control_valid_int_value_strict_ok`** — FALSE-POSITIVE GUARD
   - Fixture: `story-087-progress-bar-75.sf` (valid `value 75`)
   - Build options: format=pptx, strict=true
   - Asserts: `Ok(BuildOutput)`
   - Status: PASSES (guards against FieldSchemaValidator emitting false E-VAL-104 for valid Int)

4. **`test_BC_1_18_001_ac009_regression_chart_no_data_strict_ok`** — REGRESSION INTENT
   - Inline source: chart slide without `data` field (architect decision: chart.data → optional)
   - Build options: format=pptx, strict=true
   - Asserts: `Ok(BuildOutput)` — no E-VAL-101 for absent optional data field
   - Status: PASSES (trivially pre-wiring; documents post-wiring contract)

---

## Fixture: `crates/slideforge/tests/fixtures/story-089-progress-bar-str-value.sf`

```
slideforge_version "1"
lang "en-US"

slide progress_bar:
  title "Sprint 4 Progress"
  label "fifty percent complete"
  value "fifty"
```

Canonical test vector per BC-1.18.001 postcondition 2 (T1 type-mismatch):
`value "fifty"` is `Value::Str` where `FieldType::Int` is expected → E-VAL-104.

---

## AC-018: Compile-fail doctest artifact

**Location:** `crates/slideforge-plugin-api/src/traits/slide_type.rs`, on the `FieldDef` struct (line ~166)

**Status:** PASSES (AC-018 requires artifact existence; `#[non_exhaustive]` is already applied)

Command run:
```
cargo test -p slideforge-plugin-api --doc
```

Output:
```
test slideforge-plugin-api/src/traits/slide_type.rs - traits::slide_type::FieldDef (line 166) - compile fail ... ok
test slideforge-plugin-api/src/traits/slide_type.rs - traits::slide_type::FieldDef (line 196) ... ok
test result: ok. 1 passed; ... finished
```

The `compile_fail` doctest at line 166 verifies that struct literal construction of `FieldDef` fails to compile (simulating external-crate perspective). The passing doctest at line 196 demonstrates correct usage via `FieldDef::new()` and `FieldDef::with_type()`.

---

## Build() API shape discovered by test writing

```
BuildOptions { format: "pptx", strict: true }  → Result<BuildOutput, BuildError>
BuildOptions { format: "pptx", strict: false } → Result<BuildOutput, BuildError>

BuildOutput { bytes: Vec<u8>, extension: String }
  -- NO diagnostics field. Warn-only diagnostics NOT surfaced to callers.

BuildError::ValidationFailed { diagnostics: Vec<Diagnostic>, count: usize }
  -- diagnostics carries ALL diagnostics (Error + Warning per HIGH-2).
```

Warn-only (strict=false): E-VAL-104 diagnostics emitted as `tracing::warn!` events in `build_inner` Stage 5 log loop. `build_inner` does NOT return `Err` and does NOT populate `BuildOutput` with diagnostics. The caller has no way to inspect them via the current API.

---

## Implementer handoff

To make `test_BC_1_18_001_ac009_strict_progress_bar_str_value_returns_e_val_104` pass:

1. Create `slideforge_validate::FieldSchemaValidator` that implements `Validator`:
   - In `validate(&self, deck: &Deck, _opts: &ValidatorOptions) -> Vec<Diagnostic>`:
   - Get the `SlideTypeRegistry` (via `SLIDE_TYPE_REGISTRY` or a local `SlideTypeRegistry::default()`).
   - For each `slide` in `deck.slides`:
     - `let kw = slide.slide_type.as_ref();`
     - `if let Some(st) = registry.lookup_by_keyword(kw) { diags.extend(validate_fields(slide, st)); }`
   - Return accumulated diagnostics.

2. Register in `slideforge::registry::register_bundled_plugins`:
   - Add `builder.register_validator(Box::new(FieldSchemaValidator));`

3. Reclassify `chart.data` as optional in `ChartSlideType::new()` before or during the wiring to preserve the regression intent test (chart without data → Ok in strict mode).

The test fails specifically at the E-VAL-104 assertion with:
`Got diagnostic codes: ["E-VAL-011"]`

After wiring, both codes must appear:
`Got diagnostic codes: ["E-VAL-011", "E-VAL-104"]`
(or `["E-VAL-104", "E-VAL-011"]` depending on validator registration order)

---

## Traceability

- BC-1.18.001 postcondition 10 (AC-009): E-VAL-104 reachable from `build()`
- BC-1.18.001 postcondition 11 (AC-010): warn-only → Ok
- BC-1.18.001 postcondition 2 (T1): Str on Int field → E-VAL-104
- ADR-020 Decision 4: FieldSchemaValidator as Stage-5 Validator plugin
- ADR-016 Decision 3: Validator surface registered in registry.rs
- TD-VSDD-059: load-bearing assertion (E-VAL-104 code, not just Err)
