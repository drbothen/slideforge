---
document_type: behavioral-contract
level: L3
version: "1.3"
status: active
producer: product-owner
timestamp: 2026-06-07T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md, planning/d-fielddef-type-validation-proposal.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-14
capability: CAP-022
lifecycle_status: active
introduced: v1.0.0
modified:
  - version: "1.1"
    date: 2026-06-07
    by: product-owner
    reason: "Field-name corrections per architect adjudication / ADR-020: roadmap.milestones → roadmap.phases; dropped matrix.rows (cells is polymorphic, expected_type: None) and toc.items (TOC entries auto-generated, no list field) from Priority-1 annotated list. Authoritative total: 8 annotated field sites across 8 slide types."
  - version: "1.2"
    date: 2026-06-07
    by: product-owner
    reason: "STORY-089 adversary findings M1 + M2. M1: removed trailing ' (at <file>:<line>:<col>)' from all E-VAL-104 message templates in PC-2, PC-3, EC-001, EC-003, and canonical test vectors — aligning with E-VAL-101/102 convention where the source span is carried structurally in Diagnostic.span (rendered by miette) rather than embedded literally in the message string. M2: added Invariant 8 recording that chart.data is optional at field-schema level (validate_fields does not emit E-VAL-101 for absent chart.data); data is required at render time by ChartRenderer, not at schema-validation time. Cross-reference STORY-089."
  - version: "1.3"
    date: 2026-06-07
    by: product-owner
    reason: "STORY-089 adversary LOW fixes. (1) Corrected annotated-site count from 9 to 8 in all three locations (frontmatter v1.1 reason, PC-9 body, Traceability Slide Types Affected table): per-component `weight: Float` on weighted_composite is validated via nested Map, not a top-level FieldDef (T3 deferred, ADR-020 Decision 6). PC-9 body carries the first-occurrence clarifying parenthetical. (2) No other content changes."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-1.18.001: validate_fields Enforces Field-Value Type Against FieldDef.expected_type; Emits E-VAL-104 on Type Mismatch or OneOf Violation

## Description

`validate_fields` (in `slideforge-plugin-api/src/slide_types/registry.rs`) extends its
schema-driven validation to check whether each present field's runtime `Value` variant
matches the `FieldType` declared on the corresponding `FieldDef.expected_type`. If a
field's value has the wrong `Value` variant for its declared `FieldType`, E-VAL-104 is
emitted (broken, exit 2 in strict mode). If a field's value is `Value::Str` but the
string is not in the declared `FieldType::OneOf` allowlist, E-VAL-104 is emitted with a
distinct message branch describing the disallowed value and the permitted set.

Fields annotated with `FieldType::Any` or with `expected_type: None` are unconditionally
skipped — no type check is performed for polymorphic or unannotated fields.

## Preconditions

1. `eval_deck` has completed successfully; all `Slide.fields` entries are
   `FieldValue::Literal(Value::*)` or `FieldValue::Inlines(Vec<InlineNode>)`.
   No `FieldValue::Template` remains.
2. The `SlideType` implementation for the slide's `slide_type` keyword has been
   registered in the bundled plugin registry via `SlideTypeRegistry::register`.
3. At least one `FieldDef` in `required_fields()` or `optional_fields()` has
   `expected_type: Some(FieldType::T)` where `T` is NOT `Any`.
4. `validate_fields` is called at Stage 5 (pre-layout, post-eval) as part of the
   standard `build_inner` validation pass.
5. `build_inner` is running in strict mode (`BuildOptions::strict == true`).

## Postconditions

1. **Happy path — correct type.** When a field is present as
   `FieldValue::Literal(Value::V)` and `type_matches(V, expected)` returns `true`,
   no E-VAL-104 is emitted for that field. The diagnostic list for type checking is empty
   for that field.

2. **T1 — Type-mismatch.** When a field is present as `FieldValue::Literal(Value::V)`,
   `expected_type` is `Some(FieldType::T)` where `T` is not `Any`, and
   `type_matches(Value::V, T)` returns `false`, `validate_fields` pushes exactly one
   `Diagnostic` with:
   - `severity: DiagnosticSeverity::Error`
   - `code: Arc::from("E-VAL-104")`
   - `message`: `Field '<name>' on <type> slide has wrong type: expected <expected_display>, got <actual_type>. See the DSL reference for valid field types.`
   - `span`: the slide's `source_span` (rendered by miette as `file:line:col` — not embedded in the message string, consistent with E-VAL-101/102)

3. **T2 — OneOf violation.** When a field is present as
   `FieldValue::Literal(Value::Str(s))`, `expected_type` is
   `Some(FieldType::OneOf(variants))`, and `s` is NOT in `variants`, `validate_fields`
   pushes exactly one `Diagnostic` with:
   - `severity: DiagnosticSeverity::Error`
   - `code: Arc::from("E-VAL-104")`
   - `message`: `Field '<name>' on <type> slide has disallowed value "<s>": allowed values are [<v1>, <v2>, ...].`
   - `span`: the slide's `source_span` (rendered by miette as `file:line:col` — not embedded in the message string, consistent with E-VAL-101/102)
   (Note: `FieldType::OneOf` is reached only when `Value::Str` is the variant. If the
   value is `Value::Int` on a `OneOf`-typed field, the T1 type-mismatch message fires
   first — `OneOf` expects `Str`; an `Int` is a type mismatch, not an allowed-values
   violation.)

4. **FieldType::Any / unannotated skip.** For any field where `expected_type` is `None`
   or `Some(FieldType::Any)`, `type_matches` is never called and no E-VAL-104 is emitted
   for that field, regardless of the field's runtime value.

5. **FieldValue::Inlines skipped.** Fields present as `FieldValue::Inlines(_)` are not
   type-checked by this arm — `FieldValue::Inlines` is treated as present-but-
   untypeable-at-this-stage; no E-VAL-104 is emitted. The existing E-VAL-101 (required
   absent) and E-VAL-102 (required empty) checks operate on their own arms and are
   unaffected.

6. **Error accumulation (DI-018).** All E-VAL-104 diagnostics for a slide are accumulated
   before returning; `validate_fields` does NOT halt on the first type-mismatch. A slide
   with three mistyped fields produces three E-VAL-104 diagnostics.

7. **Strict-mode exit.** In strict mode, at least one E-VAL-104 in the accumulated
   diagnostic list causes `build_inner` to return `Err(BuildError::ValidationFailed)`,
   which renders no output (exit 2). In `--warn-only` mode, E-VAL-104 is reported as a
   warning; output is produced with an error-slide placeholder for the affected slide.

8. **`type_matches` is pure.** The helper function `type_matches(value: &Value, expected:
   &FieldType) -> bool` has no side effects, does not perform I/O, and is fully
   determined by its inputs. It is Kani-amenable.

9. **Priority-1 annotated fields validated.** The following fields across the listed
   slide types are annotated (Priority-1 — realistic type-mismatch risk) and therefore
   produce E-VAL-104 signals in practice (8 annotated FieldDef sites across 8 slide types;
   per-component `weight: Float` on weighted_composite is validated via nested Map, not a
   top-level FieldDef — T3 deferred, ADR-020 Decision 6):
   - `progress_bar.value` → `FieldType::Int`
   - `chart.chart_type` → `FieldType::OneOf(["bar", "line", "pie", "scatter", "area", "stacked-bar", "stacked-area"])`
   - `decorative` (all slides via `common_optional_fields`) → `FieldType::Bool`
   - `weighted_composite.components` → `FieldType::List`
   - `kpi_dashboard.kpis` → `FieldType::List`
   - `roadmap.phases` → `FieldType::List`
   - `agenda.items` → `FieldType::List`
   - `team.members` → `FieldType::List`

   Note: `matrix.cells` is polymorphic (structure depends on configured dimensions) →
   `expected_type: None`, NO Priority-1 annotation; only the mechanical `None` sweep
   applies. `toc` has no list field (TOC entries are auto-generated from section
   structure, not stored in a user-authored list field) → no Priority-1 annotation for
   `toc`. Both receive only the mechanical `expected_type: None` sweep covering all
   unannotated FieldDef sites. (Correction per architect adjudication / ADR-020.)

## Invariants

1. **E-VAL-104 only fires for annotated fields.** A `FieldDef` with
   `expected_type: None` is behaviorally identical to `expected_type: Some(FieldType::Any)`.
   Neither ever produces E-VAL-104. The annotation sweep is a prerequisite; an unannotated
   field silently passes type checking.

2. **E-VAL-104 does not replace E-VAL-101 or E-VAL-102.** Type checking runs on
   PRESENT field values. Absent fields are still caught by E-VAL-101. Empty-string
   required fields are still caught by E-VAL-102. A required field that is both absent
   AND mistyped cannot be detected as mistyped — absence is the dominant diagnostic.

3. **`FieldType::Float` matches `Value::Float` only.** The `weighted_composite`
   per-component `weight` field is typed `Float`. A `Value::Int` weight (e.g., `weight:
   1`) will fail `FieldType::Float` type check and produce E-VAL-104. Authors must use
   explicit float literals (e.g., `weight: 1.0`). This is intentional — no implicit int→
   float coercion (per CLAUDE.md "no implicit type coercion" rule and Q1 decisions).

4. **`decorative: true` parses to `Value::Bool(true)`.** The DSL parser emits explicit
   `true`/`false` keywords as `Value::Bool`, not `Value::Str("true")`. Therefore
   `decorative: true` with `FieldType::Bool` annotation passes type check with no false
   positive. YAML-style coercion (`NO` → bool) is the forbidden pattern; bare DSL `true`
   is a bool literal. See R2 risk analysis in the scoping proposal.

5. **Polymorphic fields use `expected_type: None`.** Fields with runtime value diversity
   across data-source binding scenarios — such as `data` on chart slides (may be `List`
   or a data-source reference `Str`) — MUST be annotated `None` (no constraint). Forcing
   `FieldType::List` on polymorphic fields would cause false E-VAL-104 positives.
   The annotation sweep must apply judgment per field, not mechanical coverage.

6. **`LayoutError::FieldTypeMismatch` is superseded at Stage 5.** The existing
   `LayoutError::FieldTypeMismatch` variant (defined in `slide_type.rs`) was the
   original late-stage guard for type errors in `lay_out()`. With E-VAL-104 enforcement
   at Stage 5 (pre-layout), any annotated-field type mismatch is caught before layout
   runs. The layout variant remains as a defensive internal invariant for
   unannotated/polymorphic fields only — it is not the primary type-enforcement path.

7. **Error accumulation per DI-018.** `validate_fields` MUST iterate all fields and
   collect ALL E-VAL-104 diagnostics before returning. Halting on the first mismatch is
   a violation of the error-accumulation invariant.

8. **`chart.data` is optional at field-schema level; required at render time.** The
   `data` field on `chart` slides is annotated `expected_type: None` (polymorphic —
   may be `Value::List` for inline data or `Value::Str` for a data-source reference).
   Crucially, `data` is also declared **optional** in `optional_fields()`, not in
   `required_fields()`. This means `validate_fields` does NOT emit E-VAL-101 when a
   chart slide has no `data` field. The absence is deliberately permitted at schema-
   validation time (Stage 5, pre-layout). The `ChartRenderer` plugin is responsible for
   enforcing data presence at render time (export stage) — a chart with no data
   produces a render-time error (E-EXP-005 or equivalent), not a Stage-5 validation
   error. This separation allows chart slides to be data-bound at runtime (e.g., via
   `@data` in a watch-mode context) without requiring inline `data:` at parse time.
   The reclassification from required to optional was applied in STORY-089 wiring and
   is tested by the chart-without-data test vector in the STORY-089 test suite. See
   also Invariant 5 (polymorphic fields use `expected_type: None`) and Postcondition 4
   (`FieldType::Any / unannotated skip`).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `progress_bar` slide with `value: "75%"` (Value::Str instead of Int) | E-VAL-104 T1 emitted: `Field 'value' on progress_bar slide has wrong type: expected integer, got string. See the DSL reference for valid field types.` (source span rendered by miette as `file:line:col`). E-VAL-011 from ValueRangeValidator does NOT fire (it reads Int value; the Str is caught first by E-VAL-104). |
| EC-002 | `chart` slide with `chart_type: 42` (Value::Int on OneOf-typed field) | E-VAL-104 T1 (type-mismatch, not T2/OneOf): `Field 'chart_type' on chart slide has wrong type: expected string, got integer. ...` OneOf allowlist check is never reached because the value is not Str. |
| EC-003 | `chart` slide with `chart_type: "donut"` (Value::Str, not in OneOf allowlist) | E-VAL-104 T2 emitted: `Field 'chart_type' on chart slide has disallowed value "donut": allowed values are [bar, line, pie, scatter, area, stacked-bar, stacked-area].` (source span rendered by miette as `file:line:col`). |
| EC-004 | `decorative: "yes"` (Value::Str on Bool-typed field) | E-VAL-104 T1: `Field 'decorative' on <type> slide has wrong type: expected boolean, got string. ...` |
| EC-005 | `decorative: true` (Value::Bool on Bool-typed field) | No E-VAL-104. type_matches(Bool(true), FieldType::Bool) returns true. No false positive. |
| EC-006 | `weighted_composite` slide with `components: "see attached"` (Value::Str on List-typed field) | E-VAL-104 T1: `Field 'components' on weighted_composite slide has wrong type: expected list, got string. ...` |
| EC-007 | `weighted_composite` slide with `components: []` (empty list) | No E-VAL-104 — `Value::List([])` matches `FieldType::List`. E-VAL-011 fires separately for the empty-components structural violation (ValueRangeValidator). |
| EC-008 | `weighted_composite` per-component `weight: 1` (Value::Int on Float-typed field) | E-VAL-104 T1: `Field 'weight' on weighted_composite slide has wrong type: expected float, got integer. ...` Authors must use `weight: 1.0`. |
| EC-009 | Field with `expected_type: None` and a semantically wrong value | No E-VAL-104. Unannotated fields are skipped unconditionally. |
| EC-010 | `FieldValue::Inlines(nodes)` for a field annotated with `FieldType::Str` | No E-VAL-104 — Inlines fields are not type-checked by this arm. No false positive. |
| EC-011 | Slide with two mistyped fields simultaneously | Two E-VAL-104 diagnostics accumulated and returned. Build fails with all diagnostics reported (DI-018). |
| EC-012 | `chart_type: "bar"` (in OneOf allowlist) | No E-VAL-104. `FieldType::OneOf` passes for any `Value::Str` in the variant list. |
| EC-013 | `--warn-only` mode with E-VAL-104 | E-VAL-104 reported as warning (severity downgraded in warn-only); error-slide placeholder rendered for the affected slide; exit 0. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `progress_bar` slide: `fields["value"] = FieldValue::Literal(Value::Int(42))` with `FieldDef { name: "value", expected_type: Some(FieldType::Int), ... }` | No E-VAL-104 diagnostic. `validate_fields` returns empty E-VAL-104 list for this field. | happy-path (Int matches Int) |
| `progress_bar` slide: `fields["value"] = FieldValue::Literal(Value::Str("42"))` with `FieldType::Int` | One E-VAL-104 with message `Field 'value' on progress_bar slide has wrong type: expected integer, got string. See the DSL reference for valid field types.` (source span in `Diagnostic.span`, rendered by miette) | error-path (T1 type-mismatch) |
| `chart` slide: `fields["chart_type"] = FieldValue::Literal(Value::Str("bar"))` with `FieldType::OneOf(["bar","line","pie","scatter","area","stacked-bar","stacked-area"])` | No E-VAL-104. | happy-path (OneOf — value in allowlist) |
| `chart` slide: `fields["chart_type"] = FieldValue::Literal(Value::Str("donut"))` with `FieldType::OneOf([...])` | One E-VAL-104 with message `Field 'chart_type' on chart slide has disallowed value "donut": allowed values are [bar, line, pie, scatter, area, stacked-bar, stacked-area].` (source span in `Diagnostic.span`, rendered by miette) | error-path (T2 OneOf violation) |
| `chart` slide: `fields["chart_type"] = FieldValue::Literal(Value::Int(42))` with `FieldType::OneOf([...])` | One E-VAL-104 T1 (type-mismatch, not T2): `Field 'chart_type' on chart slide has wrong type: expected string, got integer. ...` | error-path (Int on OneOf-typed field triggers T1 before T2) |
| Any slide: `fields["decorative"] = FieldValue::Literal(Value::Bool(true))` with `FieldType::Bool` | No E-VAL-104. | happy-path (Bool matches Bool, no false positive) |
| Any slide: `fields["decorative"] = FieldValue::Literal(Value::Str("yes"))` with `FieldType::Bool` | One E-VAL-104 T1: `Field 'decorative' on <type> slide has wrong type: expected boolean, got string. ...` | error-path (T1 — YAML-style coercible string) |
| Any slide: field with `expected_type: None` and `Value::Int(99)` | No E-VAL-104 (unannotated field). | edge-case (unannotated skip) |
| `weighted_composite` slide with two mistyped fields: `components: Str("...")` and `weight: Int(1)` (per-component) | Two E-VAL-104 diagnostics, both accumulated, both returned. Build exits 2 with both diagnostics reported. | error-path (multi-field accumulation) |
| `weighted_composite` slide: `fields["components"] = FieldValue::Inlines([...])` with `FieldType::List` | No E-VAL-104 — Inlines is not type-checked. | edge-case (Inlines skip) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `type_matches(v, FieldType::Any)` returns `true` for any `Value` variant | unit test + Kani (exhaustive over Value variants) |
| VP-TBD | `type_matches(Value::Bool(_), FieldType::Bool)` returns `true`; `type_matches(Value::Str(_), FieldType::Bool)` returns `false` | unit test |
| VP-TBD | For a slide with N mistyped annotated fields, `validate_fields` returns exactly N E-VAL-104 diagnostics (accumulation completeness) | unit test (parameterized) |
| VP-TBD | `type_matches` is a pure function: no side effects, deterministic, zero I/O — provable by Kani bounded model checking | Kani proof |
| VP-TBD | E-VAL-104 code string is exactly `"E-VAL-104"` in all emitted diagnostics — no variant produces a different code | unit test (string equality) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 ("Compile-Time Content Validation") per capabilities.md §CAP-022 |
| Capability Anchor Justification | CAP-022 ("Compile-Time Content Validation") per capabilities.md §CAP-022 — this BC defines the schema-driven type validation arm of the compile-time content validation pipeline. `validate_fields` enforcing `FieldDef.expected_type` is a compile-time check (Stage 5, pre-layout) that prevents type-mismatched field values from reaching `lay_out()`, producing user-visible E-VAL-104 diagnostics in strict mode. This is exactly what CAP-022 defines: "validate ... weight normalization. Strict mode (default build): validation errors produce no output." |
| L2 Domain Invariants | DI-018 (error accumulation — `validate_fields` accumulates all E-VAL-104 before returning; no fail-on-first), DI-004 (type safety — the "no implicit coercion" rule makes `Value::Int` on a `FieldType::Str` field a hard error, not a silent coercion) |
| Architecture Module | slideforge-plugin-api crate (SS-14) — `src/traits/slide_type.rs` (FieldType enum, FieldDef.expected_type), `src/slide_types/registry.rs` (validate_fields E-VAL-104 arm, type_matches helper) |
| Architecture Decision | Proposal: `.factory/planning/d-fielddef-type-validation-proposal.md` (commit 5d60a39b). No ADR required (additive field on existing struct, no architectural trade-off requiring formal record). |
| Stories | STORY-089 (to be filed by story-writer — field-value type validation Wave 5 slot 1) |
| Slide Types Affected | **Priority-1 annotated (8 annotated FieldDef sites, 8 slide types):** progress_bar, chart, weighted_composite, kpi_dashboard, roadmap, agenda, team (+ `decorative` on all slides via common_optional_fields). **Mechanical only (`expected_type: None` sweep):** matrix (cells → None; polymorphic), toc (no list field; entries auto-generated). All 34 slide types receive the mechanical unannotated-FieldDef None sweep. |

## Related BCs

- BC-1.17.002 — sibling (progress_bar required-field contract; this BC adds type enforcement for `value: Int` on the same slide type)
- BC-1.17.003 — sibling (weighted_composite required-field contract; this BC adds type enforcement for `components: List` and per-component `weight: Float`)
- BC-1.15.001 — depends on (diagnostic reporting with source spans; E-VAL-104 carries `slide.source_span` in `Diagnostic.span` — rendered by miette as `file:line:col`, not embedded in the message string)
- BC-1.15.002 — depends on (error accumulation invariant; E-VAL-104 participates in the same accumulation pool as E-VAL-101/102/W-VAL-103)
- BC-3.03.002 — composes with (strict mode produces no output on validation error; E-VAL-104 participates in this gate — at least one E-VAL-104 causes exit 2 and no output)
- BC-3.03.003 — composes with (warn-only mode renders error-slide placeholders; E-VAL-104 is downgraded to warning in warn-only)

## Architecture Anchors

- `crates/slideforge-plugin-api/src/traits/slide_type.rs` — FieldType enum, FieldDef struct extension
- `crates/slideforge-plugin-api/src/slide_types/registry.rs` — validate_fields E-VAL-104 arm + type_matches helper
- `specs/architecture/ARCH-INDEX.md` SS-14 (Plugin API — SlideType implementations and validate_fields)
- `.factory/planning/d-fielddef-type-validation-proposal.md` — scoping proposal with FieldType design, annotation priority, risk analysis

## Story Anchor

STORY-089 (to be filled by story-writer)

## VP Anchors

(filled after VP creation)
