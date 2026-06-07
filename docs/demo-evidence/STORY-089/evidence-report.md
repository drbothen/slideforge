# STORY-089 Demo Evidence Report

**Story:** STORY-089 — Field-Value Type Validation (BC-1.18.001)
**Branch:** feature/STORY-089
**Recorded:** 2026-06-07

---

## Recording Vehicle

**Type:** CLI library demo (VHS terminal recording of an example binary)

**Example binary:** `crates/slideforge/examples/story_089_field_validation.rs`
**Run command:** `cargo run --example story_089_field_validation -p slideforge -q`

STORY-089 is a library/validation feature — the CLI is a later Wave-5 story. The demo
vehicle is a self-contained example binary that calls `slideforge::build()` on inline
DSL sources and prints the outcome + diagnostic details for each acceptance criterion.

---

## Coverage Map

| Recording | Acceptance Criteria | Path |
|-----------|---------------------|------|
| `AC-009-010-field-validation-e-val-104.gif` / `.webm` | AC-009, AC-010, + regression guard | All 4 scenes |

---

## Scene Descriptions

### Scene 1 — AC-009 error path: T1 Str-on-Int type-mismatch → E-VAL-104

**DSL:** `progress_bar` slide with `value "fifty"` (a string literal where the field
`expected_type` is `FieldType::Int`).

**Build mode:** `strict=true`

**Expected (post-wiring):** `Err(BuildError::ValidationFailed)` carrying at least one
diagnostic with code `E-VAL-104` and a message containing "expected integer, got string".

**Demonstrated:**
- `build()` returns `Err(ValidationFailed)` with 1 error
- E-VAL-104 diagnostic is present
- Diagnostic message: `Field 'value' on progress_bar slide has wrong type: expected integer, got string.`
- Message contains "integer" (expected type) — PASS
- Message contains "string" (actual type) — PASS
- Severity is `Error` — PASS

**Traceability:** BC-1.18.001 postcondition 2 (T1); STORY-089 AC-009; ADR-020 Decision 8.

---

### Scene 2 — AC-009 error path: T2 OneOf violation → E-VAL-104

**DSL:** `chart` slide with `chart_type "donut"` (a value not in the OneOf allowlist
`[bar, line, pie, scatter, area, stacked-bar, stacked-area]`).

**Build mode:** `strict=true`

**Expected:** `Err(BuildError::ValidationFailed)` carrying E-VAL-104 with a message
identifying the disallowed value and listing the allowed values.

**Demonstrated:**
- `build()` returns `Err(ValidationFailed)` with 1 error
- E-VAL-104 diagnostic is present
- Diagnostic message: `Field 'chart_type' on chart slide has disallowed value "donut": allowed values are [bar, line, pie, scatter, area, stacked-bar, stacked-area].`
- Message contains disallowed-value language — PASS
- Message references allowed values (e.g. "bar") — PASS

**Traceability:** BC-1.18.001 postcondition 3 (T2); STORY-089 AC-009 OneOf path.

---

### Scene 3 — AC-009 positive control + AC-010 guard: valid deck → Ok, no E-VAL-104

**DSL:** Two slides — `progress_bar` with `value 75` (valid Int) and `chart` with
`chart_type "bar"` (valid OneOf value).

**Build mode:** `strict=true`

**Expected:** `Ok(BuildOutput)` — no E-VAL-104 for correctly-typed fields.

**Demonstrated:**
- `build()` returns `Ok` for correctly-typed deck — PASS
- `BuildOutput.bytes` non-empty (35,688 bytes) — PASS
- `BuildOutput.extension = "pptx"` — PASS
- No E-VAL-104 fired — `FieldSchemaValidator` correctly accepts valid types

**Traceability:** BC-1.18.001 postcondition 1; STORY-089 AC-009 positive control; AC-010 guard.

---

### Scene 4 — Regression guard: chart WITHOUT data field → Ok in strict mode

**DSL:** `chart` slide with `title`, `chart_type "bar"`, `alt` — but NO `data` field.

**Build mode:** `strict=true`

**Expected:** `Ok(BuildOutput)` — `chart.data` is optional per architect decision
(chart slides may reference `@data` at runtime; requiring it at build time would
spuriously reject valid runtime-data-bound slides).

**Demonstrated:**
- `build()` returns `Ok` — chart.data absence does not fire E-VAL-101 — PASS
- `BuildOutput` non-empty (34,825 bytes) — PASS

**Traceability:** STORY-089 architect decision (chart.data → optional);
BC-1.18.001 invariant 2 (absent optional field emits no E-VAL-104/101).

---

## Artifacts

| File | Type | Size | Description |
|------|------|------|-------------|
| `AC-009-010-field-validation-e-val-104.gif` | GIF animation | 551 KB | PR-embeddable demo recording |
| `AC-009-010-field-validation-e-val-104.webm` | WebM video | 353 KB | Archival recording |
| `AC-009-010-field-validation-e-val-104.tape` | VHS script | 1.3 KB | Reproducible tape source |

---

## Gate Results (LESSON-15 canonical gate)

All three gate checks passed before recording:

| Gate | Command | Result |
|------|---------|--------|
| fmt | `cargo fmt --all -- --check` | PASS |
| clippy pedantic | `cargo clippy -p slideforge --example story_089_field_validation -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items` | PASS |
| build | `cargo build --workspace` | PASS |

Example binary run output (warm, ~1.4 s):
- Scene 1: all PASS (E-VAL-104 for Str-on-Int)
- Scene 2: all PASS (E-VAL-104 for OneOf violation)
- Scene 3: all PASS (Ok for valid types)
- Scene 4: all PASS (Ok for absent optional field)

---

## Traceability Summary

| Criterion | Demonstrated | Evidence |
|-----------|-------------|---------|
| E-VAL-104 fires for `value "fifty"` on progress_bar (T1 Str-on-Int) | Yes — Scene 1 | Err(ValidationFailed) + E-VAL-104 message "expected integer, got string" |
| E-VAL-104 fires for `chart_type "donut"` (T2 OneOf violation) | Yes — Scene 2 | Err(ValidationFailed) + E-VAL-104 message with allowed-values list |
| Valid deck (progress_bar value 75, chart chart_type "bar") builds Ok | Yes — Scene 3 | Ok(BuildOutput) 35,688 bytes |
| chart WITHOUT data builds Ok in strict mode (optional field) | Yes — Scene 4 | Ok(BuildOutput) 34,825 bytes |
