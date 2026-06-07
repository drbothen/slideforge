---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-089
title: "FieldDef Type Annotation + validate_fields E-VAL-104 Enforcement (Schema-Driven Field-Value Type Validation)"
epic: EPIC-01
wave: 5
points: 8
priority: P0
tdd_mode: strict
status: draft
spec_version: "1.2"
last_updated: "2026-06-07"
changelog:
  - version: "1.0"
    date: "2026-06-07"
    note: "Initial story creation. Human-authorized Wave-4 follow-up (d), folded into Wave 5 slot 1. Implements ADR-020: FieldType enum + FieldDef.expected_type + type_matches + E-VAL-104 in validate_fields. BC-1.18.001 is the governing contract. Error taxonomy v2.20 formally registers E-VAL-101/102/W-VAL-103 and allocates E-VAL-104."
  - version: "1.1"
    date: "2026-06-07"
    note: "Field-name corrections per ADR-020/BC-1.18.001 v1.1 architect adjudication: roadmap list field is 'phases' (not 'milestones'); matrix.cells is polymorphic → expected_type: None (no Priority-1 annotation, drop matrix.rows); toc has no list field (entries auto-generated) → no annotation, mechanical sweep only. Priority-1 authoritative mapping (9 sites / 8 types): progress_bar.value→Int, chart.chart_type→OneOf(7), decorative→Bool, weighted_composite.components→List, kpi_dashboard.kpis→List, roadmap.phases→List, agenda.items→List, team.members→List. Polymorphic/None: matrix.cells, chart.data. toc: no annotation."
  - version: "1.2"
    date: "2026-06-07"
    note: "Subsystem anchor correction (adversary pass observation): add SS-03 (slideforge-validate) to subsystems. Pipeline wiring (human-authorized, adversary HIGH-1) added a new FieldSchemaValidator in slideforge-validate/src/field_schema.rs (Validator surface #5) and registered it in the slideforge bundled-plugins registry. This is what makes validate_fields live at build, satisfying AC-009/AC-010 and BC-1.18.001 PC-7. Prior claim that no changes to slideforge-validate were required was inaccurate."
target_module: slideforge-plugin-api
subsystems: [SS-14, SS-03]
behavioral_contracts: [BC-1.18.001]
verification_properties: []
nfr_refs: [NFR-022, NFR-024]
closes_findings: []
assumption_validations: []
risk_mitigations: []
depends_on: []
blocks: []
estimated_days: 5
history_note: >
  Created 2026-06-07 as Wave-4 follow-up (d), human-authorized per the
  d-fielddef-type-validation-proposal.md (commit 5d60a39b) and BC-1.18.001
  (commit def8bb74). Folded into Wave 5 as slot 1 (independent of all other
  Wave 5 stories). ADR-020 (commit 32fddb9a) governs the design decisions:
  FieldType enum in slide_type.rs, FieldDef.expected_type: Option<FieldType>,
  #[non_exhaustive] on FieldDef, FieldDef::new()/with_type() constructors,
  type_matches() pure function, E-VAL-104 arm in validate_fields, T1 + T2 in
  scope, T3 deferred. Points: 8 (per proposal §5 Option A estimate: ~5 days).
  Priority P0: v1.0 quality gate — type-mismatched field values must NOT reach
  lay_out() silently (LayoutError::FieldTypeMismatch is the existing defensive
  variant; Stage 5 type checking is the correct pre-layout resolution).
  Minimal overlap with STORY-088: both touch slideforge-plugin-api but in
  disjoint areas — 089 modifies slide_types/traits schema (slide_type.rs,
  registry.rs, slide type files), 088 modifies parser/eval (deck.rs, field_to_block.rs).
  No dependency on STORY-088.
---

# STORY-089: FieldDef Type Annotation + validate_fields E-VAL-104 Enforcement

## Subsystem Anchor Justifications

- **SS-14 (Plugin API, `slideforge-plugin-api`)** owns the core schema surface of this story.
  `FieldType` is a schema constraint on `FieldDef`, which is the schema surface of the `SlideType`
  trait. Both `FieldDef` and the `SlideType` trait live in
  `slideforge-plugin-api/src/traits/slide_type.rs`. The `validate_fields` function (which gains
  the E-VAL-104 arm) lives in `slideforge-plugin-api/src/slide_types/registry.rs`. All 34+
  bundled `SlideType` implementations live in `slideforge-plugin-api/src/slide_types/`.
  Per ARCH-INDEX Subsystem Registry, SS-14 = Plugin API: all `SlideType` trait implementations,
  `FieldDef`, `validate_fields`, and the plugin registry.

- **SS-03 (Validation, `slideforge-validate`)** is also in scope. Pipeline wiring (human-authorized
  per adversary HIGH-1) required adding a new `FieldSchemaValidator` in
  `slideforge-validate/src/field_schema.rs` (Validator extensibility surface #5) and registering
  it in the `slideforge` bundled-plugins registry. This is what makes `validate_fields` execute
  at Stage 5 of the build pipeline, satisfying AC-009/AC-010 and BC-1.18.001 postcondition 7
  (PC-7). Without this wiring, `validate_fields` is dead code and E-VAL-104 never fires at
  build time. The earlier claim that no changes to `slideforge-validate` were required was
  inaccurate; this correction was identified as an adversary pass observation and authorized
  as a justified scope expansion.

## Dependency Anchor Justifications

- **`depends_on: []` (no dependencies):** This story depends only on STORY-003 (31 SlideType
  implementations — merged Wave 1) and STORY-086 (Stage 2b threading — merged Wave 4, established
  `FieldValue::Literal` as the correct read path for validators). Both are merged. No Wave 5
  stories are prerequisites. STORY-089 is slot-1 in Wave 5 precisely because it has zero
  Wave 5 dependencies.

- **`blocks: []` (no downstream blocks):** No Wave 5 story has a declared dependency on STORY-089.
  The E-VAL-104 diagnostic path is a new validation arm — it does not change any existing API
  or IR type that downstream stories depend on. `FieldDef` gains a new field, but downstream
  stories that construct `FieldDef` use `FieldDef::new()` (the stable constructor); the
  `#[non_exhaustive]` attribute prevents compile-time regression in callers outside the crate.
  CLI stories (STORY-055 etc.) consume the pipeline end-to-end but do not import `FieldDef`
  directly — they invoke `slideforge::build()`.

## Summary

`validate_fields` in `crates/slideforge-plugin-api/src/slide_types/registry.rs` currently
enforces three diagnostic codes: E-VAL-101 (required field absent), E-VAL-102 (required field
empty), W-VAL-103 (unknown field). It does NOT check whether a field's runtime `Value` variant
is appropriate for that field's declared type — because `FieldDef` carries no type information.

This means a `progress_bar` slide receiving `value: "seventy-five"` (a `Value::Str` where
`Value::Int` is required) silently passes field validation and reaches `lay_out()`, where it
either panics (`.unwrap()` violation) or produces garbage layout. The existing
`LayoutError::FieldTypeMismatch` variant in `slide_type.rs` shows that the architects already
anticipated this failure mode in `lay_out()`. Moving type checking to Stage 5 (pre-layout,
schema-driven) is the correct architectural resolution. E-VAL-104 is formally registered in
error-taxonomy.md v2.20 (commit def8bb74).

This story implements ADR-020 in full:
1. Adds `FieldType` enum to `slide_type.rs` (7 variants: Any, Str, Int, Float, Bool, List, Map, OneOf)
2. Adds `expected_type: Option<FieldType>` field to `FieldDef`
3. Marks `FieldDef` `#[non_exhaustive]` and adds `FieldDef::new()` + `FieldDef::with_type()` constructors
4. Adds `type_matches(value: &Value, expected: &FieldType) -> bool` pure function to `slide_type.rs`
5. Adds E-VAL-104 arm to `validate_fields` in `registry.rs` (T1 type-mismatch + T2 OneOf violation)
6. Sweeps all `FieldDef { ... }` struct literal construction sites across ~80-120 sites in 34+ slide type files, adding `expected_type: None` or `Some(..)` as appropriate
7. Annotates Priority-1 fields (10 sites across 9 slide types) that produce real E-VAL-104 signals in practice
8. Formally registers E-VAL-101, E-VAL-102, W-VAL-103 in error-taxonomy.md (existing codes, formal registration only) alongside new E-VAL-104

## Narrative

As a slideforge user authoring presentation decks, I want the build system to detect
when I assign the wrong value type to a field (e.g., `value: "75%"` on a `progress_bar`
slide instead of `value: 75`), so that I receive a clear E-VAL-104 compile error with
the field name, expected type, and actual type before any broken output is produced —
rather than getting garbage layout output or a runtime panic.

## Behavioral Contracts

| BC | Title | Version | Role in This Story |
|----|-------|---------|-------------------|
| BC-1.18.001 | validate_fields Enforces Field-Value Type Against FieldDef.expected_type; Emits E-VAL-104 on Type Mismatch or OneOf Violation | v1.0 | Primary: defines E-VAL-104 (T1 + T2), type_matches semantics, postconditions 1-9, invariants 1-7, edge cases EC-001..EC-013, and all canonical test vectors for this story |

## Acceptance Criteria

### AC-001 — Happy path: correct field type produces no E-VAL-104
A `progress_bar` slide constructed with
`fields["value"] = FieldValue::Literal(Value::Int(42))` and the corresponding `FieldDef`
annotated with `expected_type: Some(FieldType::Int)` produces no E-VAL-104 diagnostic.
`validate_fields` returns without appending to the E-VAL-104 accumulator for this field.
(traces to BC-1.18.001 postcondition 1 — happy path, correct type, no E-VAL-104)

### AC-002 — T1 type-mismatch: wrong Value variant emits E-VAL-104
A `progress_bar` slide constructed with
`fields["value"] = FieldValue::Literal(Value::Str(Arc::from("42")))` and
`expected_type: Some(FieldType::Int)` causes `validate_fields` to push exactly one
`Diagnostic` with:
- `severity: DiagnosticSeverity::Error`
- `code: Arc::from("E-VAL-104")`
- `message` containing: `"Field 'value' on progress_bar slide has wrong type: expected integer, got string."` and the source span `(at <file>:<line>:<col>)`
(traces to BC-1.18.001 postcondition 2 — T1 type-mismatch, E-VAL-104 emitted;
BC-1.18.001 EC-001 — `value: "75%"` case; BC-1.18.001 canonical test vector row 2)

### AC-003 — T2 OneOf: valid OneOf value produces no E-VAL-104
A `chart` slide constructed with
`fields["chart_type"] = FieldValue::Literal(Value::Str(Arc::from("bar")))` and
`expected_type: Some(FieldType::OneOf(vec![Arc::from("bar"), Arc::from("line"), ...]))` (full allowlist)
produces no E-VAL-104 diagnostic. `type_matches(Value::Str("bar"), FieldType::OneOf([...]))`
returns `true` when `"bar"` is in the allowlist.
(traces to BC-1.18.001 postcondition 1 — happy path for OneOf;
BC-1.18.001 EC-012 — `chart_type: "bar"` passes;
BC-1.18.001 canonical test vector row 3)

### AC-004 — T2 OneOf: disallowed string value emits E-VAL-104 with message branch B
A `chart` slide constructed with
`fields["chart_type"] = FieldValue::Literal(Value::Str(Arc::from("donut")))` and
`expected_type: Some(FieldType::OneOf([...full allowlist...]))` causes `validate_fields` to
push exactly one `Diagnostic` with:
- `code: Arc::from("E-VAL-104")`
- `message` containing: `Field 'chart_type' on chart slide has disallowed value "donut": allowed values are [bar, line, pie, scatter, area, stacked-bar, stacked-area].`
(traces to BC-1.18.001 postcondition 3 — T2 OneOf violation, message branch B;
BC-1.18.001 EC-003 — `chart_type: "donut"` case;
BC-1.18.001 canonical test vector row 4)

### AC-005 — T1 fires before T2 when non-Str value on OneOf-typed field
A `chart` slide constructed with
`fields["chart_type"] = FieldValue::Literal(Value::Int(42))` and
`expected_type: Some(FieldType::OneOf([...]))` causes `validate_fields` to push one
E-VAL-104 with the T1 type-mismatch message (`"expected string, got integer"`) — NOT
the T2 disallowed-value message. The OneOf allowlist check is never reached for non-Str values.
(traces to BC-1.18.001 postcondition 3 — note: OneOf reached only when Value::Str;
BC-1.18.001 EC-002 — `chart_type: 42` triggers T1 before T2;
BC-1.18.001 canonical test vector row 5)

### AC-006 — FieldType::Any / unannotated skip: no E-VAL-104 for unannotated fields
A field with `expected_type: None` holding any `Value` variant (including a semantically
mismatched one, e.g., `Value::Int(99)` on a text field) produces no E-VAL-104.
`type_matches` is never called for `expected_type: None` fields.
(traces to BC-1.18.001 postcondition 4 — Any/unannotated skip;
BC-1.18.001 invariant 1 — None is behaviorally identical to Some(FieldType::Any);
BC-1.18.001 EC-009 — unannotated field skip;
BC-1.18.001 canonical test vector row 7)

### AC-007 — FieldValue::Inlines fields are not type-checked
A field with `expected_type: Some(FieldType::Str)` holding `FieldValue::Inlines(_)` produces
no E-VAL-104. Inlines fields are unconditionally skipped by the type-check arm.
(traces to BC-1.18.001 postcondition 5 — FieldValue::Inlines skipped;
BC-1.18.001 EC-010 — Inlines on Str-typed field, no false positive)

### AC-008 — Error accumulation: N mistyped fields → N E-VAL-104 diagnostics
A `weighted_composite` slide with two simultaneously mistyped annotated fields
(e.g., `components: Value::Str("...")` with `FieldType::List` AND `weight: Value::Int(1)`
with `FieldType::Float` on a sub-component) causes `validate_fields` to return exactly
2 E-VAL-104 diagnostics. Validation does NOT halt on the first mismatch.
(traces to BC-1.18.001 postcondition 6 — error accumulation per DI-018;
BC-1.18.001 invariant 7 — MUST iterate all fields, collect ALL diagnostics;
BC-1.18.001 EC-011 — multi-field simultaneous mismatch;
BC-1.18.001 canonical test vector row 9)

### AC-009 — Strict mode: E-VAL-104 causes BuildError::ValidationFailed, no output
When `slideforge::build()` is called in strict mode (`BuildOptions::strict == true`) with a
fixture containing a mistyped annotated field (e.g., `slide progress_bar: title "T" label "L" value "oops"`),
the build returns `Err(BuildError::ValidationFailed)` and produces zero output bytes.
At least one E-VAL-104 is present in the diagnostics list.
(traces to BC-1.18.001 postcondition 7 — strict-mode exit 2;
BC-1.18.001 EC-001 — `value: "75%"` triggering E-VAL-104 in strict mode)

### AC-010 — Warn-only mode: E-VAL-104 reported as warning, output produced with error-slide
When `slideforge::build()` is called with `BuildOptions::strict == false` (warn-only) with
the same mistyped field fixture as AC-009, the build returns `Ok(BuildOutput)`. The diagnostics
list contains at least one diagnostic for the affected slide. An error-slide placeholder is
rendered for the affected slide; output is produced.
(traces to BC-1.18.001 postcondition 7 — warn-only behavior;
BC-1.18.001 EC-013 — `--warn-only` mode with E-VAL-104)

### AC-011 — type_matches is a pure function: no side effects, deterministic
A unit test verifies that `type_matches(v, expected)` returns the same result for the same
inputs on repeated calls with no intervening state change. The function has no side effects,
does not mutate globals, and does not perform I/O. It is testable in isolation without any
mutable state setup.
(traces to BC-1.18.001 postcondition 8 — type_matches is pure; Kani-amenable;
BC-1.18.001 canonical test vectors — all rows are deterministic pure-function calls)

### AC-012 — Priority-1 annotated fields: progress_bar.value enforces FieldType::Int
`ProgressBarSlideType::required_fields()` returns a `FieldDef` for `"value"` with
`expected_type: Some(FieldType::Int)`. A unit test confirms this annotation is present
and correct (not `None`, not `Some(FieldType::Str)`).
(traces to BC-1.18.001 postcondition 9 — Priority-1 annotation: progress_bar.value → Int)

### AC-013 — Priority-1 annotated fields: chart.chart_type enforces FieldType::OneOf
`ChartSlideType::required_fields()` or `ChartSlideType::optional_fields()` returns a
`FieldDef` for `"chart_type"` with
`expected_type: Some(FieldType::OneOf(vec!["bar", "line", "pie", "scatter", "area", "stacked-bar", "stacked-area"]))`.
A unit test confirms all 7 allowlist values are present in the OneOf variant list.
(traces to BC-1.18.001 postcondition 9 — Priority-1 annotation: chart.chart_type → OneOf(7 values))

### AC-014 — Priority-1 annotated fields: decorative in common_optional_fields enforces FieldType::Bool
`common_optional_fields()` in `slide_types/mod.rs` returns a `FieldDef` for `"decorative"`
with `expected_type: Some(FieldType::Bool)`. A unit test confirms the annotation. A separate
test confirms that a `decorative: true` (Value::Bool) slide passes type check with no
E-VAL-104, while `decorative: "yes"` (Value::Str) emits E-VAL-104 T1.
(traces to BC-1.18.001 postcondition 9 — Priority-1 annotation: decorative common optional → Bool;
BC-1.18.001 invariant 4 — `decorative: true` in DSL parses to Value::Bool(true), not Value::Str;
BC-1.18.001 EC-004/005 — `decorative: "yes"` fails, `decorative: true` passes)

### AC-015 — Priority-1 annotated fields: weighted_composite.components enforces FieldType::List
`WeightedCompositeSlideType::required_fields()` returns a `FieldDef` for `"components"` with
`expected_type: Some(FieldType::List)`. A test confirms `components: Value::Str("...")` emits
E-VAL-104 T1 (`"expected list, got string"`).
(traces to BC-1.18.001 postcondition 9 — Priority-1 annotation: weighted_composite.components → List;
BC-1.18.001 EC-006 — `components: "see attached"` case)

### AC-016 — Priority-1 annotated fields: kpi_dashboard.kpis, roadmap.phases, weighted_composite.components, agenda.items, team.members all enforce FieldType::List
Unit tests for each slide type confirm the respective list field has `expected_type: Some(FieldType::List)`
and that passing a `Value::Str` on that field emits E-VAL-104 T1.
Slide types: `kpi_dashboard` (`kpis`), `roadmap` (`phases`), `agenda` (`items`), `team` (`members`).
Note: `weighted_composite.components` is also a List-annotated Priority-1 field (covered by AC-015).
Note: `matrix.cells` is polymorphic → `expected_type: None` (no Priority-1 annotation); `matrix` has
no `rows` field to annotate.
Note: `toc` has no list field (entries are auto-generated, not user-supplied) → no Priority-1 annotation;
`toc.rs` receives only mechanical `expected_type: None` sweep.
(traces to BC-1.18.001 postcondition 9 — Priority-1 annotation: kpi_dashboard.kpis, roadmap.phases,
agenda.items, team.members → List; weighted_composite.components → List per AC-015;
BC-1.18.001 EC-006 pattern — Str on List-typed field emits T1)

### AC-017 — FieldDef::new() constructor: returns FieldDef with expected_type: None
`FieldDef::new(name, description, required, default_value)` returns a valid `FieldDef` with
`expected_type: None`. All existing internal struct literal construction sites in
`slideforge-plugin-api/src/slide_types/` are updated to include `expected_type: None` (or
`Some(..)` for annotated fields). The workspace compiles with zero errors after the sweep.
(traces to BC-1.18.001 precondition 2 — FieldDef construction must compile across all 34+ slide types;
ADR-020 Decision 3 — #[non_exhaustive] + constructor API)

### AC-018 — FieldDef is #[non_exhaustive]: external struct literal construction fails to compile
`FieldDef` has the `#[non_exhaustive]` attribute. A doc-test or compile-time assertion
(in a `#[cfg(test)] mod tests` block using `trybuild` or a `compile_fail` doctest) verifies
that external struct literal construction of `FieldDef { name: ..., ... }` fails to compile
outside the crate.
(traces to ADR-020 Decision 3 — #[non_exhaustive] enforces constructor API for external callers;
BC-1.18.001 precondition 2 — SlideType implementation registered via stable constructor API)

### AC-019 — Invariant: FieldType::Float strictly requires Value::Float; Value::Int on Float-typed field emits E-VAL-104
A `weighted_composite` component with `weight: 1` (Value::Int) on a `FieldType::Float`-annotated
field emits E-VAL-104 T1 (`"expected float, got integer"`). Authors must use `weight: 1.0`.
This is intentional — no implicit int→float coercion (per CLAUDE.md "no implicit type coercion"
rule and Q1 decisions).
(traces to BC-1.18.001 invariant 3 — FieldType::Float matches only Value::Float;
BC-1.18.001 EC-008 — `weight: 1` (Int) fails FieldType::Float)

### AC-020 — Invariant: E-VAL-104 does not replace E-VAL-101 or E-VAL-102; absent required fields stay as E-VAL-101
A slide with a required field that is absent still produces E-VAL-101 (absent) — NOT E-VAL-104
(type mismatch). Type checking runs only on PRESENT field values. Absent fields are caught by
the E-VAL-101 arm before the type-check arm is reached.
(traces to BC-1.18.001 invariant 2 — E-VAL-104 does not replace E-VAL-101 or E-VAL-102;
BC-1.18.001 precondition 1 — fields are FieldValue::Literal or Inlines; absence is a separate concern)

### AC-021 — Polymorphic fields use expected_type: None; no false E-VAL-104 on data-source-bound fields
Fields with runtime value diversity (e.g., `data` on `chart` slides — may be `Value::List` or
`Value::Str` data-source reference) have `expected_type: None`. A test confirms that
`Value::Str` on a `None`-annotated field produces no E-VAL-104 regardless of the value.
(traces to BC-1.18.001 invariant 5 — polymorphic fields must use expected_type: None;
BC-1.18.001 EC-009 — unannotated field skip;
ADR-020 Decision 2 — None ≡ Any semantics)

## Architecture Mapping

| Component | Crate | File | Change Type | Pure/Effectful |
|-----------|-------|------|-------------|----------------|
| `FieldType` enum (7 variants + #[non_exhaustive]) | `slideforge-plugin-api` | `src/traits/slide_type.rs` | NEW enum | Pure |
| `FieldDef.expected_type: Option<FieldType>` field | `slideforge-plugin-api` | `src/traits/slide_type.rs` | MODIFY struct (additive) | Pure |
| `#[non_exhaustive]` on `FieldDef` | `slideforge-plugin-api` | `src/traits/slide_type.rs` | MODIFY struct attribute | Pure |
| `FieldDef::new()` constructor | `slideforge-plugin-api` | `src/traits/slide_type.rs` | NEW `impl FieldDef` | Pure |
| `FieldDef::with_type()` constructor | `slideforge-plugin-api` | `src/traits/slide_type.rs` | NEW `impl FieldDef` | Pure |
| `type_matches(value: &Value, expected: &FieldType) -> bool` | `slideforge-plugin-api` | `src/traits/slide_type.rs` | NEW free function | Pure |
| E-VAL-104 arm in `validate_fields` | `slideforge-plugin-api` | `src/slide_types/registry.rs` | MODIFY function (additive arm) | Pure |
| `common_optional_fields()` — `decorative: FieldType::Bool` | `slideforge-plugin-api` | `src/slide_types/mod.rs` | MODIFY: add `expected_type: Some(FieldType::Bool)` to `decorative` FieldDef; `expected_type: None` to remaining 8 common optional fields | Pure |
| `progress_bar.rs` — `value: FieldType::Int` annotation | `slideforge-plugin-api` | `src/slide_types/progress_bar.rs` | MODIFY: annotate `value` FieldDef with `expected_type: Some(FieldType::Int)`; add `expected_type: None` to all others | Pure |
| `chart.rs` — `chart_type: FieldType::OneOf([...])` annotation | `slideforge-plugin-api` | `src/slide_types/chart.rs` | MODIFY: annotate `chart_type` FieldDef with `expected_type: Some(FieldType::OneOf([...]))`; add `expected_type: None` to all others | Pure |
| `weighted_composite.rs` — `components: FieldType::List`, `weight: FieldType::Float` | `slideforge-plugin-api` | `src/slide_types/weighted_composite.rs` | MODIFY: annotate `components` + per-component `weight` FieldDef; `expected_type: None` to remaining | Pure |
| `kpi_dashboard.rs` — `kpis: FieldType::List` | `slideforge-plugin-api` | `src/slide_types/kpi_dashboard.rs` | MODIFY: annotate `kpis`; `expected_type: None` to remaining | Pure |
| `roadmap.rs` — `phases: FieldType::List` | `slideforge-plugin-api` | `src/slide_types/roadmap.rs` | MODIFY: annotate `phases`; `expected_type: None` to remaining | Pure |
| `matrix.rs` — mechanical sweep only; `cells: expected_type: None` (polymorphic) | `slideforge-plugin-api` | `src/slide_types/matrix.rs` | MODIFY: `cells` → `expected_type: None` (polymorphic); `expected_type: None` to all remaining FieldDef sites; NO Priority-1 annotation (no `rows` field exists) | Pure |
| `agenda.rs` — `items: FieldType::List` | `slideforge-plugin-api` | `src/slide_types/agenda.rs` | MODIFY: annotate `items`; `expected_type: None` to remaining | Pure |
| `toc.rs` — mechanical sweep only; no list field annotation | `slideforge-plugin-api` | `src/slide_types/toc.rs` | MODIFY: mechanical sweep only — add `expected_type: None` to existing FieldDef sites (title required + commons); toc entries are auto-generated, not user-supplied; NO `items` annotation | Pure |
| `team.rs` — `members: FieldType::List` | `slideforge-plugin-api` | `src/slide_types/team.rs` | MODIFY: annotate `members`; `expected_type: None` to remaining | Pure |
| All remaining 23+ slide type files | `slideforge-plugin-api` | `src/slide_types/*.rs` | MODIFY: add `expected_type: None` to every FieldDef construction site (mechanical; compiler-directed — will not compile until ALL sites are updated) | Pure |
| Error taxonomy formal registration | `.factory/specs/prd-supplements/error-taxonomy.md` | — | MODIFY: formally register E-VAL-101, E-VAL-102, W-VAL-103 (existing informal codes); allocate E-VAL-104 (new) | Spec artifact |

**Architecture Sections Referenced:**
- `architecture/module-decomposition.md` — SS-14 scope, `slideforge-plugin-api` crate structure
- `architecture/dependency-graph.md` — confirms `slideforge-plugin-api` has no downstream
  compile-time dependency issue (other crates import from it; we are adding to its public API
  in a backward-compatible way via `#[non_exhaustive]`)

**Forbidden Dependencies:**
- `slideforge-plugin-api/src/traits/slide_type.rs` and `src/slide_types/` MUST NOT import
  `slideforge-pptx`, `slideforge-pdf`, `slideforge-docx`, `slideforge-html`, or
  `slideforge-validate`. These are plugin-api crate internals; they have no rendering or
  validator dependencies. Adding a dependency on any of these crates MUST cause the build
  to fail via `Cargo.toml` `[dev-dependencies]` vs `[dependencies]` discipline.
- `FieldType` MUST NOT be added to `slideforge-types` (wrong dependency direction — SS-15
  is the runtime IR crate, not the schema-constraint crate). Per ADR-020 Decision 1 rationale.

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec (v1.0) | ~4,800 |
| BC-1.18.001 full text | ~3,200 |
| ADR-020 full text | ~3,500 |
| d-fielddef-type-validation-proposal.md (§1-§6) | ~4,000 |
| `slideforge-plugin-api/src/traits/slide_type.rs` (current — FieldDef, SlideType trait, LayoutError) | ~2,500 |
| `slideforge-plugin-api/src/slide_types/registry.rs` (validate_fields function — current) | ~2,000 |
| `slideforge-plugin-api/src/slide_types/mod.rs` (common_optional_fields, mod re-exports) | ~1,500 |
| Priority-1 slide type files (progress_bar.rs, chart.rs, weighted_composite.rs, kpi_dashboard.rs, roadmap.rs, matrix.rs, agenda.rs, toc.rs, team.rs) — 9 files | ~4,500 |
| Representative sample of remaining 23 slide type files (pattern reference — 3 files) | ~1,500 |
| `slideforge-types` Value enum (for type_matches truth table) | ~1,000 |
| Unit test files (new — type_matches truth table + E-VAL-104 per annotated field + accumulation + strict/warn tests) | ~5,000 |
| Tool outputs (compiler messages confirming #[non_exhaustive] sites, test results) | ~2,500 |
| BC files (1 BC) | ~3,200 |
| **TOTAL ESTIMATED** | **~38,700 tokens** |

~39,000 tokens is ~19.5% of a 200k context window — within the 20-30% per-story budget ceiling.
If context pressure is high: load slide type files on-demand one at a time during the annotation
sweep (each ~400-600 tokens); do NOT preload all 34 files upfront (~5,000 additional tokens).

## Previous Story Intelligence

This story follows STORY-087 (color-coded slide types) and STORY-003 (31 SlideType implementations).

**From STORY-087 (closed Wave 4):**
- The real `validate_fields` is a FREE FUNCTION in `registry.rs`, NOT a trait method on `SlideType`.
  The trait methods are: `id()`, `required_fields()`, `optional_fields()`, `layout_name()`, `lay_out()`.
  There is no `validate_fields()` method on the `SlideType` trait — do not add one.
- `validate_fields` is called by `build_inner` at Stage 5 (pre-layout). It reads `Slide.fields`
  (which are `FieldValue::Literal(Value::*)` or `FieldValue::Inlines(...)` after eval completes).
- `FieldValue::Inlines` is present-but-untypeable at Stage 5 — skip it in the type-check arm.
  Only `FieldValue::Literal(v)` triggers the type-check arm.
- Template for `SlideType` implementations: `crates/slideforge-plugin-api/src/slide_types/stat_callout.rs`.
  Follow exactly this pattern when reading existing construction sites.
- `ValueRangeValidator` in `slideforge-validate/src/value_range.rs` is ORTHOGONAL — it checks
  numeric ranges (progress_bar value ∈ [0,100]) at Stage 5 via a separate validator. Do NOT
  conflate type checking (this story) with range checking (STORY-087). E-VAL-104 (type) and
  E-VAL-011 (range) are different diagnostic codes on different validators.

**From STORY-003 (Wave 1):**
- All `FieldDef { ... }` construction sites use struct literal syntax with ALL fields named.
  Adding `expected_type: Option<FieldType>` requires updating EVERY construction site to include
  the new field. The Rust compiler will catch every missed site — do not guess, let the compiler
  direct the sweep.
- Common optional fields (decorative, notes, brand_overlay, etc.) are in `common_optional_fields()`
  in `slide_types/mod.rs`. Updating this function with `expected_type` values propagates to ALL
  34 slide types automatically for the common fields. Individual slide types only need annotation
  for their per-type `required_fields()` and any type-specific `optional_fields()` that are NOT
  in `common_optional_fields()`.
- All slide types are in `crates/slideforge-plugin-api/src/slide_types/` (per ADR-016). The
  correct template is `stat_callout.rs`.

**From LESSON-16 (STORY-083):**
- All new public items require rustdoc. `FieldType` enum and its variants need doc comments.
  `type_matches` needs a doc comment. `FieldDef::new()` and `FieldDef::with_type()` constructors
  need doc comments. The `missing_docs` lint fires on new public items without docs.

**From LESSON-13 (STORY-049):**
- The Red Gate discipline: ALL non-trivial function bodies start as `todo!()`. For `type_matches`,
  the stub is `todo!("implement type_matches")`. For the E-VAL-104 arm in `validate_fields`, the
  stub is the new match arm returning immediately (failing the E-VAL-104 tests). The implementer
  MUST write failing tests FIRST, then implement to make them pass — no swapping order.
- The mechanical `expected_type: None` sweep across 80-120 construction sites is scaffolding work,
  NOT implementation — it makes the workspace compile (Red Gate step). The Priority-1 annotations
  and `type_matches` logic are the implementation step (Green Gate).

**From STORY-086 (Stage 2b threading, Wave 4):**
- `FieldValue::Literal(Value::*)` is the canonical representation after eval. The type-check arm
  should pattern-match on `FieldValue::Literal(ref v)` and call `type_matches(v, expected)`.
- `FieldValue::Inlines(_)` is the other variant — skip it in the type-check arm with no E-VAL-104.
- The diagnostic struct: `Diagnostic { severity, code, message, span }`. Use `slide.source_span`
  for the span (as established by E-VAL-101/102 in the existing `validate_fields`). Match the
  exact message format from BC-1.18.001 postconditions 2 and 3.

## Architecture Compliance Rules

1. **ADR-006 (plugin-first):** `FieldType` and `type_matches` are in `slideforge-plugin-api`
   because that is where `FieldDef` and the `SlideType` trait live. Moving them to `slideforge-types`
   would be ADR-020 Rejected Alternative A — wrong dependency direction.

2. **ADR-020 Decision 5 (`type_matches` is pure):** `type_matches(value: &Value, expected: &FieldType) -> bool`
   MUST have no side effects, no global reads, no I/O. It is deterministic and fully determined
   by its two arguments. This makes it Kani-provable in Wave 6 (Phase 6 formal hardening).
   Any implementation that mutates state, logs, or reads a global is a contract violation.

3. **ADR-020 Decision 3 (`#[non_exhaustive]` on FieldDef):** `FieldDef` MUST be marked
   `#[non_exhaustive]`. External callers MUST use `FieldDef::new()` or `FieldDef::with_type()`.
   Internal code in `slideforge-plugin-api` may use struct literals — but only AFTER updating
   ALL existing construction sites to include `expected_type`.

4. **DI-018 (error accumulation):** `validate_fields` MUST iterate ALL fields and collect ALL
   E-VAL-104 diagnostics before returning. Halting on the first mismatch is a BC-1.18.001
   invariant 7 violation. Use the same accumulation pattern as E-VAL-101/102 in the existing
   `validate_fields` body.

5. **No implicit coercion (CLAUDE.md rule, Q1 decisions):** `FieldType::Float` matches only
   `Value::Float`. `Value::Int(1)` on a `FieldType::Float` field is a type mismatch → E-VAL-104.
   Authors must write `weight: 1.0`, not `weight: 1`. There is no int→float coercion path.
   Do not add one. This is intentional and documented in BC-1.18.001 invariant 3.

6. **`#![forbid(unsafe_code)]`** — No `unsafe` blocks anywhere in this story's changes.

7. **Zero `.unwrap()` in non-test code.** `validate_fields` already uses `?` and explicit
   match arms. Maintain that discipline in the new E-VAL-104 arm. `type_matches` uses pure
   match expressions — no `unwrap()` needed.

8. **`Hash + Eq + Clone + Debug + PartialEq` on `FieldType`:** Per CLAUDE.md and ADR-020
   Decision 1 (comemo compatibility). `OneOf(Vec<Arc<str>>)` must derive all of these.
   Verify that `Vec<Arc<str>>` supports `Hash + Eq` (it does — `Arc<str>` implements both).

9. **`FieldType` must be `#[non_exhaustive]`:** Per ADR-020 Alternative C discussion: `FieldType`
   itself is `#[non_exhaustive]` to allow future variants (e.g., `Format(FormatKind)` for T3).
   Exhaustive match on `FieldType` in `type_matches` is required within the crate — `#[non_exhaustive]`
   only affects external match arms. Internal match must handle all current variants.

10. **`LayoutError::FieldTypeMismatch` remains:** Do NOT remove this variant from `slide_type.rs`.
    It is a defensive internal invariant for unannotated/polymorphic fields that reach `lay_out()`.
    E-VAL-104 at Stage 5 is the new primary path; `LayoutError::FieldTypeMismatch` is the
    last-resort fallback. BC-1.18.001 invariant 6 explicitly governs this.

## Library and Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-types` | workspace path | `Value` enum (runtime type representation — FieldType is the schema-side mirror) |
| `slideforge-plugin-api` | workspace path (self) | `FieldDef`, `SlideType` trait, `validate_fields` — all changes are within this crate |
| `tracing` | 0.1 (workspace) | `tracing::debug!` in `validate_fields` for diagnostic context (existing pattern) |
| `arc-str` | via `std::sync::Arc<str>` | `Arc::from("E-VAL-104")` for error code; `Arc<str>` in `OneOf(Vec<Arc<str>>)` variant |

No new external dependencies required. All dependencies are already in the workspace.
`FieldType::OneOf(Vec<Arc<str>>)` uses only stdlib types — no `phf` or `hashbrown` dependency.

## File Structure Requirements

Files to MODIFY (implementation):
```
crates/slideforge-plugin-api/src/traits/slide_type.rs
    [Add FieldType enum (#[non_exhaustive], Debug+Clone+PartialEq+Eq+Hash derives)]
    [Add expected_type: Option<FieldType> field to FieldDef struct]
    [Add #[non_exhaustive] attribute to FieldDef]
    [Add FieldDef::new(name, description, required, default_value) -> Self constructor]
    [Add FieldDef::with_type(name, description, required, default_value, expected_type) -> Self constructor]
    [Add type_matches(value: &Value, expected: &FieldType) -> bool free function (pure)]
    [Keep LayoutError::FieldTypeMismatch variant — do NOT remove]

crates/slideforge-plugin-api/src/slide_types/registry.rs
    [Add E-VAL-104 arm to validate_fields: after E-VAL-102 check, for FieldValue::Literal(v)
     where field_def.expected_type is Some(T) and type_matches(v, T) returns false,
     push Diagnostic with code "E-VAL-104" and correct message format (T1 or T2 branch)]

crates/slideforge-plugin-api/src/slide_types/mod.rs
    [Update common_optional_fields(): add expected_type: Some(FieldType::Bool) to "decorative";
     add expected_type: None to the remaining 8 common optional fields]

--- Priority-1 annotation sites (10 FieldDef sites across 9 slide types) ---
crates/slideforge-plugin-api/src/slide_types/progress_bar.rs
    [Annotate "value" FieldDef: expected_type: Some(FieldType::Int)]
    [Add expected_type: None to all other FieldDef construction sites in this file]

crates/slideforge-plugin-api/src/slide_types/chart.rs
    [Annotate "chart_type" FieldDef: expected_type: Some(FieldType::OneOf(vec![
        Arc::from("bar"), Arc::from("line"), Arc::from("pie"), Arc::from("scatter"),
        Arc::from("area"), Arc::from("stacked-bar"), Arc::from("stacked-area")]))]
    [Add expected_type: None to all other FieldDef construction sites in this file]

crates/slideforge-plugin-api/src/slide_types/weighted_composite.rs
    [Annotate "components" FieldDef: expected_type: Some(FieldType::List)]
    [Annotate per-component "weight" FieldDef: expected_type: Some(FieldType::Float)]
    [Add expected_type: None to remaining FieldDef construction sites (e.g., polymorphic fields like "data")]

crates/slideforge-plugin-api/src/slide_types/kpi_dashboard.rs
    [Annotate "kpis" FieldDef: expected_type: Some(FieldType::List)]
    [Add expected_type: None to remaining FieldDef construction sites]

crates/slideforge-plugin-api/src/slide_types/roadmap.rs
    [Annotate "phases" FieldDef: expected_type: Some(FieldType::List)]
    [NOTE: field is named "phases", NOT "milestones" — per ADR-020/BC-1.18.001 v1.1]
    [Add expected_type: None to remaining FieldDef construction sites]

crates/slideforge-plugin-api/src/slide_types/matrix.rs
    [Mechanical sweep only — NO Priority-1 annotation]
    ["cells" field is polymorphic → expected_type: None (no specific type annotation)]
    [No "rows" field exists to annotate]
    [Add expected_type: None to all remaining FieldDef construction sites]

crates/slideforge-plugin-api/src/slide_types/agenda.rs
    [Annotate "items" FieldDef: expected_type: Some(FieldType::List)]
    [Add expected_type: None to remaining FieldDef construction sites]

crates/slideforge-plugin-api/src/slide_types/toc.rs
    [Mechanical sweep only — NO Priority-1 annotation]
    [toc entries are auto-generated, not user-supplied; no "items" field to annotate]
    [Add expected_type: None to all existing FieldDef construction sites (title required + commons)]

crates/slideforge-plugin-api/src/slide_types/team.rs
    [Annotate "members" FieldDef: expected_type: Some(FieldType::List)]
    [Add expected_type: None to remaining FieldDef construction sites]

--- Mechanical sweep: remaining 23+ slide type files (expected_type: None to all FieldDef sites) ---
crates/slideforge-plugin-api/src/slide_types/title.rs
crates/slideforge-plugin-api/src/slide_types/content.rs
crates/slideforge-plugin-api/src/slide_types/two_column.rs
crates/slideforge-plugin-api/src/slide_types/comparison.rs
crates/slideforge-plugin-api/src/slide_types/blank.rs
crates/slideforge-plugin-api/src/slide_types/image.rs
crates/slideforge-plugin-api/src/slide_types/quote.rs
crates/slideforge-plugin-api/src/slide_types/stat_callout.rs
crates/slideforge-plugin-api/src/slide_types/kpi_callout.rs
crates/slideforge-plugin-api/src/slide_types/timeline.rs
crates/slideforge-plugin-api/src/slide_types/process_flow.rs
crates/slideforge-plugin-api/src/slide_types/org_chart.rs
crates/slideforge-plugin-api/src/slide_types/risk_register.rs
crates/slideforge-plugin-api/src/slide_types/survey_results.rs
crates/slideforge-plugin-api/src/slide_types/side_by_side.rs
crates/slideforge-plugin-api/src/slide_types/calendar.rs
crates/slideforge-plugin-api/src/slide_types/grid.rs
crates/slideforge-plugin-api/src/slide_types/bio.rs
crates/slideforge-plugin-api/src/slide_types/announcement.rs
crates/slideforge-plugin-api/src/slide_types/closing.rs
crates/slideforge-plugin-api/src/slide_types/cover.rs
crates/slideforge-plugin-api/src/slide_types/divider.rs
crates/slideforge-plugin-api/src/slide_types/status.rs      [from STORY-087]
    [+ any additional slide type files confirmed to exist on develop]
    [Compiler directs: add expected_type: None to every FieldDef { ... } literal]

--- Spec artifact ---
.factory/specs/prd-supplements/error-taxonomy.md
    [Formally register E-VAL-101 (required absent), E-VAL-102 (required empty),
     W-VAL-103 (unknown field) — already in production use, formally registered in this burst]
    [Allocate E-VAL-104 (new): type-mismatch / OneOf violation]
```

Files to CREATE:
```
crates/slideforge-plugin-api/tests/field_type_validation.rs
    [Integration tests for AC-001..AC-021:
     - type_matches truth table (all FieldType variants × all Value variants)
     - E-VAL-104 T1 for each Priority-1 annotated field (progress_bar.value, decorative, etc.)
     - E-VAL-104 T2 for chart_type OneOf violation
     - T1 before T2 ordering for non-Str on OneOf-typed field
     - error accumulation: 2 mistyped fields → 2 E-VAL-104 diagnostics
     - FieldValue::Inlines skipped (no false positive)
     - expected_type: None skipped (no false positive)
     - strict mode: E-VAL-104 → BuildError::ValidationFailed
     - warn-only mode: E-VAL-104 → Ok with error-slide placeholder
     - FieldDef::new() returns expected_type: None
     - FieldDef::with_type() returns expected_type: Some(provided)]
```

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `progress_bar` slide with `value: "75%"` (Value::Str instead of Int) | E-VAL-104 T1: `"Field 'value' on progress_bar slide has wrong type: expected integer, got string."` E-VAL-011 from ValueRangeValidator does NOT fire (type error caught first). |
| EC-002 | `chart` slide with `chart_type: 42` (Value::Int on OneOf-typed field) | E-VAL-104 T1 (type-mismatch, not T2/OneOf): `"Field 'chart_type' on chart slide has wrong type: expected string, got integer."` OneOf allowlist check never reached. |
| EC-003 | `chart` slide with `chart_type: "donut"` (Value::Str, not in OneOf allowlist) | E-VAL-104 T2: `"Field 'chart_type' on chart slide has disallowed value "donut": allowed values are [bar, line, pie, scatter, area, stacked-bar, stacked-area]."` |
| EC-004 | `decorative: "yes"` (Value::Str on Bool-typed field) | E-VAL-104 T1: `"Field 'decorative' on <type> slide has wrong type: expected boolean, got string."` |
| EC-005 | `decorative: true` (Value::Bool on Bool-typed field) | No E-VAL-104. `type_matches(Bool(true), FieldType::Bool)` returns `true`. No false positive. |
| EC-006 | `weighted_composite` with `components: "see attached"` (Value::Str on List-typed field) | E-VAL-104 T1: `"Field 'components' on weighted_composite slide has wrong type: expected list, got string."` |
| EC-007 | `weighted_composite` with `components: []` (empty list) | No E-VAL-104 — `Value::List([])` matches `FieldType::List`. E-VAL-011 fires separately (ValueRangeValidator, STORY-087 scope). |
| EC-008 | Per-component `weight: 1` (Value::Int on Float-typed field) | E-VAL-104 T1: `"Field 'weight' on weighted_composite slide has wrong type: expected float, got integer."` Authors must use `weight: 1.0`. |
| EC-009 | Field with `expected_type: None` and any value | No E-VAL-104. Unannotated fields are skipped unconditionally. |
| EC-010 | `FieldValue::Inlines(nodes)` for a field annotated with `FieldType::Str` | No E-VAL-104 — Inlines fields are not type-checked by this arm. |
| EC-011 | Slide with two mistyped fields simultaneously | Two E-VAL-104 diagnostics accumulated and returned (DI-018). Build fails with all diagnostics reported. |
| EC-012 | `chart_type: "bar"` (in OneOf allowlist) | No E-VAL-104. `FieldType::OneOf` passes for any `Value::Str` in the variant list. |
| EC-013 | `--warn-only` mode with E-VAL-104 | E-VAL-104 reported as warning; error-slide placeholder rendered; exit 0. |
| EC-014 | Polymorphic `data` field on chart slide with `Value::Str` (data-source reference) | No E-VAL-104 — `data` field has `expected_type: None`. No false positive for data-source reference strings. |

## Tasks

### T1 — Add FieldType enum and FieldDef.expected_type to slide_type.rs

**Files:** `crates/slideforge-plugin-api/src/traits/slide_type.rs`

1. Add `#[non_exhaustive]` attribute to `FieldDef` struct definition.
2. Add `pub expected_type: Option<FieldType>` field to `FieldDef` after `default_value`.
3. Define `FieldType` enum above or adjacent to `FieldDef`:
   ```rust
   #[non_exhaustive]
   #[derive(Debug, Clone, PartialEq, Eq, Hash)]
   pub enum FieldType {
       Any,
       Str,
       Int,
       Float,
       Bool,
       List,
       Map,
       OneOf(Vec<Arc<str>>),
   }
   ```
4. Add `impl FieldDef` block with `::new()` and `::with_type()` constructors.
5. Add `pub fn type_matches(value: &Value, expected: &FieldType) -> bool` free function.
   All function body must start as `todo!("implement type_matches")` in the Red Gate step.

**Red Gate deliverable:** Workspace compiles with all new items stubbed as `todo!()`.

### T2 — Update all FieldDef construction sites across 34+ slide type files (mechanical sweep)

**Files:** `src/slide_types/mod.rs` + all 34+ `src/slide_types/*.rs` files

1. Run the workspace with the new `expected_type` field — the compiler will emit one error per
   missed struct literal site. Fix each site by adding `expected_type: None` (or `Some(..)` for
   annotated fields). This is purely mechanical — the compiler directs every fix.
2. Priority order: `mod.rs` common_optional_fields first (propagates to all types), then
   Priority-1 annotation sites (9 files with meaningful annotations), then mechanical sweep
   (remaining 23+ files with `expected_type: None` only).
3. Polymorphic fields (e.g., `data` on chart, `cells` on matrix): use `expected_type: None`.
   Do NOT annotate with a specific type — false E-VAL-104 positives would result.

**Red Gate deliverable:** All construction sites updated; workspace compiles without error.
All tests pass (no behavior change yet — type_matches is still `todo!()`).

### T3 — Write Red Gate tests for E-VAL-104 behavior (failing tests before implementation)

**Files:** `crates/slideforge-plugin-api/tests/field_type_validation.rs` (NEW)

Write unit tests that fail (Red Gate) because `type_matches` is still `todo!()`:
- `test_type_matches_int_matches_int()` — `type_matches(Value::Int(1), FieldType::Int)` → true
- `test_type_matches_str_fails_int()` — `type_matches(Value::Str("x"), FieldType::Int)` → false
- `test_type_matches_any_always_true()` — `type_matches(any_value, FieldType::Any)` → true
- `test_type_matches_oneof_str_in_list()` — in-list str → true
- `test_type_matches_oneof_str_not_in_list()` — not-in-list str → false (T2)
- `test_type_matches_int_on_oneof_field()` — Int on OneOf → false (T1, not T2)
- `test_validate_fields_t1_emits_e_val_104()` — wrong type on annotated field → E-VAL-104
- `test_validate_fields_t2_emits_e_val_104_oneof()` — disallowed str on OneOf → E-VAL-104 message branch B
- `test_validate_fields_accumulates_two_errors()` — two mistyped fields → two E-VAL-104 diagnostics
- `test_validate_fields_none_annotation_skips()` — `expected_type: None` → no E-VAL-104
- `test_validate_fields_inlines_skips()` — `FieldValue::Inlines` → no E-VAL-104
- `test_progress_bar_value_annotation()` — FieldDef for `value` has `expected_type: Some(FieldType::Int)`
- `test_chart_type_annotation_oneof_7_values()` — `chart_type` FieldDef has OneOf with 7 values
- `test_decorative_annotation_bool()` — `decorative` in common_optional_fields has `expected_type: Some(FieldType::Bool)`
- (Additional tests per AC-001..AC-021 as needed)

**Red Gate deliverable:** Tests are written and failing (due to `todo!()` panics or assertion failures).

### T4 — Implement type_matches and E-VAL-104 arm in validate_fields

**Files:** `src/traits/slide_type.rs` (type_matches body), `src/slide_types/registry.rs` (E-VAL-104 arm)

1. Implement `type_matches(value: &Value, expected: &FieldType) -> bool`:
   - `Any` → `true` always
   - `Str` → matches `Value::Str`
   - `Int` → matches `Value::Int`
   - `Float` → matches `Value::Float`
   - `Bool` → matches `Value::Bool`
   - `List` → matches `Value::List`
   - `Map` → matches `Value::Map`
   - `OneOf(variants)` → matches `Value::Str(s)` where `variants.iter().any(|v| v.as_ref() == s)`;
     for non-Str values, returns `false` (T1 fires, not T2).
2. Add E-VAL-104 arm to `validate_fields` in `registry.rs`:
   - For each field with `expected_type: Some(T)` where T is not `Any`:
     - Match `FieldValue::Literal(v)`: call `type_matches(v, T)`
       - If false AND T is `OneOf` AND v is `Value::Str(s)`: message branch B (disallowed value)
       - If false otherwise: message branch A (type mismatch)
       - Both push `Diagnostic { severity: Error, code: "E-VAL-104", message, span: slide.source_span }`
     - Match `FieldValue::Inlines(_)`: skip (no E-VAL-104)
     - Match `None` (field absent): skip (E-VAL-101 arm handles this separately)
   - Accumulate ALL E-VAL-104 diagnostics before returning (DI-018).
3. Implement `FieldDef::new()` and `FieldDef::with_type()` constructor bodies.

**Green Gate deliverable:** All T3 tests pass. Workspace test suite passes (`cargo nextest run -p slideforge-plugin-api`).

### T5 — Apply Priority-1 annotations to 9 slide types

**Files:** progress_bar.rs, chart.rs, weighted_composite.rs, kpi_dashboard.rs, roadmap.rs, matrix.rs, agenda.rs, toc.rs, team.rs

Update the `FieldDef` for each Priority-1 field from `expected_type: None` to the correct annotation.
Authoritative Priority-1 mapping (9 sites / 8 types):
- `progress_bar.value` → `Some(FieldType::Int)`
- `chart.chart_type` → `Some(FieldType::OneOf([7 values]))`
- `weighted_composite.components` → `Some(FieldType::List)`
- `weighted_composite` per-component `weight` → `Some(FieldType::Float)` (if modeled as a FieldDef)
- `kpi_dashboard.kpis` → `Some(FieldType::List)`
- `roadmap.phases` → `Some(FieldType::List)` — NOTE: field is named `phases`, NOT `milestones`
- `agenda.items` → `Some(FieldType::List)`
- `team.members` → `Some(FieldType::List)`
- `decorative` in `common_optional_fields()` → `Some(FieldType::Bool)` (done in T2)

Polymorphic fields (remain `None`):
- `matrix.cells` → `expected_type: None` (polymorphic — no Priority-1 annotation; `matrix` has no `rows` field)
- `chart.data` → `expected_type: None` (polymorphic)

toc.rs — mechanical sweep ONLY:
- `toc` has no user-supplied list field (entries are auto-generated); no `items` annotation
- Apply `expected_type: None` to all existing FieldDef construction sites in `toc.rs` (title required + commons)

After applying annotations, run `cargo nextest run -p slideforge-plugin-api` to confirm
Priority-1 tests in T3 now pass.

### T6 — Formally register E-VAL-101/102/W-VAL-103 and allocate E-VAL-104 in error-taxonomy.md

**File:** `.factory/specs/prd-supplements/error-taxonomy.md`

Add formal entries for:
- E-VAL-101: `Required field '<name>' missing on <type> slide.` (broken, exit 2) — formal registration only, code already in production use
- E-VAL-102: `Required field '<name>' is empty on <type> slide.` (broken, exit 2) — formal registration only
- W-VAL-103: `Unknown field '<key>' for slide type '<type>'.` (cosmetic, exit 0) — formal registration only
- E-VAL-104: `Field '<name>' on <type> slide has wrong type: expected <expected>, got <actual>. See DSL reference for valid field types.` (broken, exit 2) — NEW

This resolves the long-standing taxonomy debt acknowledged in earlier spec bursts.

### T7 — Full workspace gate check and clippy cleanup

```bash
cargo nextest run --workspace --no-fail-fast
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo fmt --all -- --check
```

Resolve any clippy findings. Ensure all new public items have rustdoc. Confirm `#![forbid(unsafe_code)]`
still passes with zero unsafe blocks introduced.

## Verification Plan

| Test ID | Test Name | Property Verified | AC |
|---------|-----------|-------------------|----|
| T3-01 | `test_type_matches_int_matches_int` | `type_matches(Int, FieldType::Int)` returns true | AC-001, AC-011 |
| T3-02 | `test_type_matches_str_fails_int` | `type_matches(Str, FieldType::Int)` returns false | AC-002, AC-011 |
| T3-03 | `test_type_matches_any_always_true` | Any FieldType::Any → true for all Value variants | AC-006, AC-011 |
| T3-04 | `test_type_matches_oneof_str_in_list` | In-list str → true (T2 pass) | AC-003, AC-011 |
| T3-05 | `test_type_matches_oneof_str_not_in_list` | Not-in-list str → false (T2 fail) | AC-004, AC-011 |
| T3-06 | `test_type_matches_int_on_oneof_field` | Int on OneOf → false (T1) | AC-005, AC-011 |
| T3-07 | `test_validate_fields_t1_emits_e_val_104` | T1 mismatch produces E-VAL-104 with message branch A | AC-002 |
| T3-08 | `test_validate_fields_t2_emits_e_val_104_oneof` | T2 violation produces E-VAL-104 with message branch B | AC-004 |
| T3-09 | `test_validate_fields_accumulates_two_errors` | 2 mistyped fields → exactly 2 E-VAL-104 diagnostics | AC-008 |
| T3-10 | `test_validate_fields_none_annotation_skips` | expected_type: None → no E-VAL-104 | AC-006 |
| T3-11 | `test_validate_fields_inlines_skips` | FieldValue::Inlines → no E-VAL-104 | AC-007 |
| T3-12 | `test_progress_bar_value_annotation` | progress_bar.value FieldDef has expected_type: Some(FieldType::Int) | AC-012 |
| T3-13 | `test_chart_type_annotation_oneof_7_values` | chart_type FieldDef has OneOf([7 values]) | AC-013 |
| T3-14 | `test_decorative_annotation_bool` | decorative common optional has expected_type: Some(FieldType::Bool) | AC-014 |
| T3-15 | `test_decorative_str_emits_e_val_104` | `decorative: "yes"` (Str) emits E-VAL-104 T1 | AC-014 |
| T3-16 | `test_decorative_bool_passes` | `decorative: true` (Bool) passes type check | AC-014 |
| T3-17 | `test_weighted_composite_components_list_annotation` | components FieldDef has expected_type: Some(FieldType::List) | AC-015 |
| T3-18 | `test_weight_float_annotation` | per-component weight FieldDef has expected_type: Some(FieldType::Float) | AC-019 |
| T3-19 | `test_weight_int_emits_e_val_104` | `weight: 1` (Int) emits E-VAL-104 T1 (no int→float coercion) | AC-019 |
| T3-20 | `test_list_fields_on_kpi_roadmap_agenda_team` | kpi_dashboard.kpis, roadmap.phases, agenda.items, team.members have expected_type: Some(FieldType::List); roadmap asserts optional_fields() has FieldDef name=="phases" with expected_type==Some(FieldType::List) (NOT milestones); matrix and toc excluded from List-assertion scope (polymorphic/auto-generated) | AC-016 |
| T3-21 | `test_e_val_104_code_exact_string` | Emitted code is exactly `"E-VAL-104"`, not a variant | AC-002, AC-004 |
| T3-22 | `test_fieldddef_new_returns_none_annotation` | FieldDef::new() returns expected_type: None | AC-017 |
| T3-23 | `test_e_val_101_still_fires_for_absent_required_field` | Absent required field → E-VAL-101, not E-VAL-104 | AC-020 |
| T3-24 | `test_polymorphic_data_field_none` | `data` field on chart slide has expected_type: None | AC-021 |
