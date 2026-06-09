# STORY-088 Demo Evidence Report

**Story:** STORY-088 — Bullets list-literal field-value DSL syntax (`bullets: ["A","B","C"]`)
**Crates:** `slideforge-syntax`, `slideforge-eval`, `slideforge`
**Branch:** `feature/STORY-088`
**Recorded:** 2026-06-09
**Recording tool:** VHS (terminal capture of `cargo nextest` tests)

---

## Artifacts

| File | Format | Covered ACs |
|------|--------|-------------|
| `AC-001-004-list-literal-parse.gif` | GIF (PR embed) | AC-001, AC-002, AC-003, AC-004 |
| `AC-001-004-list-literal-parse.webm` | WEBM (archival) | AC-001, AC-002, AC-003, AC-004 |
| `AC-001-004-list-literal-parse.tape` | VHS script | AC-001, AC-002, AC-003, AC-004 |
| `AC-005-006-error-paths-regression.gif` | GIF (PR embed) | AC-005 (integer, boolean, nested), AC-006 |
| `AC-005-006-error-paths-regression.webm` | WEBM (archival) | AC-005, AC-006 |
| `AC-005-006-error-paths-regression.tape` | VHS script | AC-005, AC-006 |
| `AC-007-e2e-pptx-runs.gif` | GIF (PR embed) | AC-007 |
| `AC-007-e2e-pptx-runs.webm` | WEBM (archival) | AC-007 |
| `AC-007-e2e-pptx-runs.tape` | VHS script | AC-007 |
| `AC-012-013-set-rule-variant-vars.gif` | GIF (PR embed) | AC-012 (positive + error), AC-013 (positive + error + nested) |
| `AC-012-013-set-rule-variant-vars.webm` | WEBM (archival) | AC-012, AC-013 |
| `AC-012-013-set-rule-variant-vars.tape` | VHS script | AC-012, AC-013 |

---

## Coverage Map

### AC-001 — `bullets: ["A","B","C"]` parses to FieldValue::List(Vec<FieldValue>)

**Demonstrated in recording:** `AC-001-004-list-literal-parse` — Section 1
**Evidence:** `cargo nextest run -p slideforge-syntax -E 'test(ac001_bullets_field_list_literal)'` runs:
- `test_bc_1_01_002_ac001_bullets_field_list_literal_parses_to_field_value_list` — asserts `SlideNode.fields["bullets"]` is `FieldValue::List(vec![Template(...), Template(...), Template(...)])` and eval produces `Value::List([Str("Item A"), Str("Item B"), Str("Item C")])`

**Result:** PASS

---

### AC-002 — Empty list-literal `bullets: []` parses to FieldValue::List([])

**Demonstrated in recording:** `AC-001-004-list-literal-parse` — Section 2
**Evidence:** `test_bc_1_01_002_ac002_empty_list_literal_parses_to_field_value_list_empty` asserts `FieldValue::List(vec![])` for `bullets: []`. No error emitted.

**Result:** PASS

---

### AC-003 — Single-item list `bullets: ["Only"]` parses correctly

**Demonstrated in recording:** `AC-001-004-list-literal-parse` — Section 3
**Evidence:** `test_bc_1_01_002_ac003_single_item_list_parses_as_list_not_bare_string` asserts the single-element is `FieldValue::List(vec![Template(...)])`, not a bare `FieldValue::Template`.

**Result:** PASS

---

### AC-004 — List-literal parses inside standard slide block indentation

**Demonstrated in recording:** `AC-001-004-list-literal-parse` — Section 4
**Evidence:** `test_bc_1_01_002_ac004_list_literal_in_full_slide_block_with_multiple_fields` parses a slide with `title`, `bullets: [...]`, and `body` fields; asserts all four fields parse correctly without off-by-one indentation errors or partial-parse failures.

**Result:** PASS

---

### AC-005 — Non-string list items produce E-PAR diagnostic, NOT a panic

**Demonstrated in recording:** `AC-005-006-error-paths-regression` — Sections 1–4
**Evidence:**

- `test_bc_1_01_002_ac005_non_string_list_items_produce_epar_not_panic` — `bullets: [42, true]` produces an `E-PAR` diagnostic without panicking; error accumulation continues
- `test_bc_1_01_002_med001a_integer_item_produces_e_par_024_with_type_integer` — error message contains `"got integer"` for integer list elements
- `test_bc_1_01_002_med001b_boolean_item_produces_e_par_024_with_type_boolean` — error message contains `"got boolean"` for boolean list elements
- `test_bc_1_01_002_med_p3_001_deck_nested_list_item_is_not_silently_accepted` — `bullets: [["x"]]` produces `E-PAR-024` with "got nested list" description

The same `E-PAR-024` code and `"got <type>"` message format appears in ALL value-position contexts (field-value, set-rule, variant-vars) — demonstrated by AC-012 and AC-013 recordings.

**Result:** PASS (all error paths)

---

### AC-006 — @var binding form continues to work (no regression)

**Demonstrated in recording:** `AC-005-006-error-paths-regression` — Sections 5–6
**Evidence:**

- `test_bc_1_01_002_ac006_vars_block_list_assignment_parses_to_fieldvalue_list` — `@var items = ["A","B","C"]` parses to `FieldValue::List` in the vars-block context
- `test_bc_1_01_002_ac006_regression_bullets_ident_reference_still_parses` — `bullets: items` (ident reference) still parses without error after the list-literal arm was added

**Result:** PASS (no regression)

---

### AC-007 — E2E: `bullets: ["A","B","C"]` produces ≥3 text runs in PPTX output

**Demonstrated in recording:** `AC-007-e2e-pptx-runs`
**Evidence:** `test_bc_1_01_002_ac007_direct_list_literal_bullets_produces_ge3_text_runs_pptx` in `crates/slideforge/tests/e2e/story_088_bullets_list_literal.rs`:
- Reads fixture `story-088-bullets-direct-literal.sf` (`bullets: ["Item A", "Item B", "Item C"]`, no `@var`)
- Calls `slideforge::build()` → asserts `Ok`
- Opens PPTX ZIP, reads `ppt/slides/slide1.xml`
- Asserts `<a:r>` count ≥ 3 AND literal text "Item A", "Item B", "Item C" present in slide XML

The recording also shows the fixture file contents before running the E2E test, making the DSL → PPTX chain visible.

**Result:** PASS (E2E closure of STORY-086 AC-007 parser-gap note)

---

### AC-012 — Set-rule list-literal default: `set <type>: bullets ["A","B"]` parses to FieldValue::List

**Demonstrated in recording:** `AC-012-013-set-rule-variant-vars` — Sections 1–3

**Positive case evidence:**
- `test_bc_1_01_002_ac012_set_rule_list_literal_parses_to_list_value` — `set content: bullets ["Step 1", "Step 2", "Step 3"]` parses to `FieldValue::List([Template(...), ...])`; the default flows through the set-rule default-merge mechanism

**Error case evidence:**
- `test_bc_1_01_002_ac012_set_rule_non_string_list_items_produce_e_par_024` — `set content: bullets [42, true]` produces `E-PAR-024`; error accumulation continues
- `test_bc_1_01_002_p5_med001_set_rule_integer_item_has_got_type_in_message` — `"got integer"` appears in the set-rule position error message, confirming the same E-PAR-024 format as the field-value position

**Result:** PASS (positive and error paths)

---

### AC-013 — Variant vars list-literal override: `vars: items: ["A","B"]` inside a variant block parses to FieldValue::List

**Demonstrated in recording:** `AC-012-013-set-rule-variant-vars` — Sections 4–6

**Positive case evidence:**
- `test_bc_1_01_002_ac013_variant_vars_list_literal_parses_to_fieldvalue_list` — `variant short:` / `vars:` / `items: ["Quick win", "Low effort"]` parses to `FieldValue::List([Template(...), Template(...)])`

**Error case evidence:**
- `test_bc_1_01_002_ac013_variant_vars_non_string_list_items_produce_e_par_024` — `items: [42, true]` in variant vars position produces `E-PAR-024`
- `test_bc_1_01_002_ac013_variant_vars_nested_list_is_rejected` — `items: [["x"]]` produces `E-PAR-024` nested list rejection in the variant vars position

**Result:** PASS (positive, error, and nested-list-rejection paths)

---

## Clippy / Fmt Gate Confirmation

All CI gates ran clean before recording (3/3 strict-CLEAN adversary convergence confirmed on feature/STORY-088):

| Gate | Command | Result |
|------|---------|--------|
| Format | `cargo fmt --all -- --check` | CLEAN |
| Clippy (canonical gate) | `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items` | CLEAN |
| slideforge-syntax full suite | `cargo nextest run -p slideforge-syntax` | CLEAN (460/460 PASS, 1 skipped) |
| AC-007 E2E | `cargo nextest run -p slideforge -E 'test(story_088)'` | CLEAN (1/1 PASS) |

---

## Error Path Coverage Summary

The E-PAR-024 `"got <type>"` error message is demonstrated consistently across all three value-position surfaces:

| Position | Input | Error | Demonstrated in |
|----------|-------|-------|-----------------|
| Field-value (`bullets: [42]`) | integer element | E-PAR-024 "got integer" | AC-005-006 recording, section 2 |
| Field-value (`bullets: [true]`) | boolean element | E-PAR-024 "got boolean" | AC-005-006 recording, section 3 |
| Field-value (`bullets: [["x"]]`) | nested list | E-PAR-024 "got nested list" | AC-005-006 recording, section 4 |
| Set-rule (`set content: bullets [42]`) | integer element | E-PAR-024 "got integer" | AC-012-013 recording, section 3 |
| Set-rule (`set content: bullets [42, true]`) | both types | E-PAR-024 | AC-012-013 recording, section 2 |
| Variant-vars (`items: [42, true]`) | non-string elements | E-PAR-024 | AC-012-013 recording, section 5 |
| Variant-vars (`items: [["x"]]`) | nested list | E-PAR-024 | AC-012-013 recording, section 6 |
