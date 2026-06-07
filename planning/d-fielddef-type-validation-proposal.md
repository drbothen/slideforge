---
document_type: scoping-proposal
status: draft
producer: architect
timestamp: 2026-06-07T00:00:00
scope: Wave-4 follow-up (d) — FieldDef type validation
decision_required: deliver-now-standalone vs fold-into-Wave-5
traces_to:
  - .factory/stories/STORY-INDEX.md
  - .factory/specs/prd-supplements/error-taxonomy.md
  - crates/slideforge-plugin-api/src/traits/slide_type.rs
  - crates/slideforge-plugin-api/src/slide_types/registry.rs
---

# Decomposition + Scoping Proposal: FieldDef Type Validation (Wave-4 follow-up d)

## Summary of the Gap

`validate_fields` in `crates/slideforge-plugin-api/src/slide_types/registry.rs` emits
three diagnostic codes: E-VAL-101 (required field absent), E-VAL-102 (required field
empty), W-VAL-103 (unknown field). It does NOT check whether a field value has the
correct type for that field because `FieldDef` carries no type information — only
`name`, `description`, `required: bool`, and `default_value: Option<Value>`.

A separate `ValueRangeValidator` (added in STORY-087) patches over this for the three
color-coded slide types (`progress_bar`, `status`, `weighted_composite`), but it is
type-specific imperative code in `slideforge-validate`, not schema-driven type checking.

The human has authorized extending `FieldDef` with type annotations to make field-value
type mismatches a first-class, schema-driven compile error.

---

## Section 1: FieldDef Extension Design

### 1.1 Where FieldType Lives

`FieldType` belongs in `slideforge-plugin-api`, NOT in `slideforge-types`. Rationale:

- `FieldDef` is already in `slideforge-plugin-api/src/traits/slide_type.rs`. FieldType
  annotates FieldDef; they must co-locate or FieldDef would depend on slideforge-types
  for FieldType (that dependency already exists — `FieldDef.default_value: Option<Value>`
  already imports from `slideforge-types`).
- `slideforge-types` defines the VALUE runtime type (`Value`, `TypeKind`). `FieldType`
  is a SCHEMA type — a constraint on what Values are acceptable. These are different
  levels of abstraction. Mixing them would make `slideforge-types` aware of plugin-api
  schema concerns — wrong dependency direction.
- Downstream: the `SlideType` trait, `validate_fields`, and all 34 built-in slide type
  implementations are in `slideforge-plugin-api`. `FieldType` is consumed only there.

### 1.2 The FieldType Enum

```rust
/// The expected value type for a field, used in schema-driven validation.
///
/// Consistent with the `Value` type system (slideforge-types) and the
/// "no implicit coercion" rule (R3 finding): `NO` stays string, `1.10`
/// stays string. `FieldType::Str` matches only `Value::Str`; it does not
/// accept `Value::Int(1)` where a string is declared.
///
/// `FieldType::Any` opts out of type checking entirely — the field can
/// hold any `Value` variant. This is the correct annotation for fields
/// with polymorphic content (e.g., `data`, `bullets`, `components`
/// whose parsed representation may vary across DSL evolution stages).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldType {
    /// Any `Value` variant is acceptable. Used for polymorphic fields.
    Any,
    /// Field must be `Value::Str`.
    Str,
    /// Field must be `Value::Int`.
    Int,
    /// Field must be `Value::Float`.
    Float,
    /// Field must be `Value::Bool`.
    Bool,
    /// Field must be `Value::List`.
    List,
    /// Field must be `Value::Map`.
    Map,
    /// Field must be one of the listed string literals (T2 tier — enum validation).
    /// The validator checks `Value::Str(s)` where `s` is in `variants`.
    OneOf(Vec<Arc<str>>),
}
```

Note: `FieldType::Float` is included because `weighted_composite` components use
`Value::Float` for `weight`. Omitting it would make the enum incomplete against the
actual `Value` type system.

### 1.3 FieldDef Extension

The extension to `FieldDef` is purely additive — one new optional field:

```rust
pub struct FieldDef {
    pub name: Arc<str>,
    pub description: Arc<str>,
    pub required: bool,
    pub default_value: Option<Value>,
    /// Expected type for field-value type checking.
    ///
    /// `None` means no type constraint (equivalent to `Some(FieldType::Any)`).
    /// Introduced as `Option<FieldType>` rather than `FieldType` so that
    /// existing FieldDef construction sites that do not set `expected_type`
    /// remain valid without a struct-update syntax change.
    pub expected_type: Option<FieldType>,
}
```

`None` (no type annotation) is identical in behavior to `Some(FieldType::Any)` — the
validator skips the type check. This is the backward-compatible default for all 34 slide
types until their `required_fields()`/`optional_fields()` are updated with annotations.

### 1.4 SlideType Trait Impact

The `SlideType` trait signature does NOT change. `required_fields()` and
`optional_fields()` already return `&[FieldDef]`. Adding a field to `FieldDef` (with
`None` default semantics) is not a trait signature change — it is a struct field
addition. Callers that construct `FieldDef` using named-field syntax must add
`expected_type: None` to every construction site, OR we add `#[non_exhaustive]` +
provide a builder/constructor. Using `#[non_exhaustive]` on `FieldDef` is the cleaner
approach: it means external plugins compiled against the old API can add the field
without a hard break, and we add `FieldDef::new(...)` or `FieldDef { ..Default::default() }`
as the construction path.

Decision: add `impl Default for FieldDef` returning `FieldDef { name: Arc::from(""),
description: Arc::from(""), required: false, default_value: None, expected_type: None }`.
All existing construction sites use struct literal syntax with ALL fields named — they
must be updated to include `expected_type: None`. This is mechanical but not zero-cost
(counted in Section 4).

### 1.5 validate_fields Extension

The type-check arm is added to `validate_fields` immediately after the empty-string
arm (E-VAL-102 check) for required fields, and as a new loop for both required and
optional fields with non-None `expected_type`:

```rust
// After E-VAL-102 check for required fields:
Some(FieldValue::Literal(v)) => {
    if let Some(expected) = &field_def.expected_type {
        if !type_matches(v, expected) {
            diags.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: Arc::from("E-VAL-104"),
                message: Arc::from(format!(
                    "Field '{name}' on {type_id} slide has wrong type: \
                     expected {expected_display}, got {actual}.",
                    name = field_def.name,
                    expected_display = expected.display_name(),
                    actual = v.type_name(),
                )),
                ...
            });
        }
    }
}
```

A pure function `type_matches(value: &Value, expected: &FieldType) -> bool` lives in
`slideforge-plugin-api` alongside `FieldType`. It is a deterministic pure function with
no side effects — Kani-provable.

---

## Section 2: Validation Scope Tiers

### Tier 1 — Type-Mismatch (T1): RECOMMENDED IN SCOPE

**What:** `Value` is present but has the wrong variant for the field's declared
`FieldType`. Example: `value: "fifty"` on a `progress_bar` slide where `value` is
declared `FieldType::Int`.

**Error code:** E-VAL-104 (proposed — see Section 3).

**Validation surface:** `validate_fields` in `slideforge-plugin-api`. This is Stage 5
(pre-layout), same as E-VAL-101/102/W-VAL-103. No new validator registration needed —
`validate_fields` is called by the existing pipeline on every slide.

**T1 is required for v1.0** because without it, type-declared fields can be silently
passed wrong-typed values into `lay_out()`, which receives `&Slide` and must defensively
handle every variant. The existing `ValueRangeValidator` already does ad-hoc type
checking for 2 of 34 slide types; T1 makes it schema-driven and covers all 34.

Additionally: `LayoutError::FieldTypeMismatch` already exists in the codebase (defined
in `slide_type.rs`). Its presence means the architects already anticipated type mismatch
as a failure mode in `lay_out()`. Moving the check to schema-driven compile time (T1) is
the correct resolution — not leaving it to each `lay_out()` implementation.

### Tier 2 — Enum/Allowed-Values (T2): RECOMMENDED IN SCOPE

**What:** `Value::Str` is the correct variant but not in the allowed set. Example:
`chart_type: "donut"` where `chart_type` is `FieldType::OneOf(["bar", "line", "pie",
"scatter", "area", "stacked-bar", "stacked-area"])`.

**T2 is in scope** because `FieldType::OneOf` is defined as part of T1's enum, and
implementing T1 without T2 means defining an enum variant that is silently ignored.
`FieldType::OneOf` matching logic is a single `variants.iter().any(|v| v == s)` call.
The marginal cost of T2 given T1 is zero at the framework level; it is only annotation
cost at the slide-type level.

**Which fields benefit from T2:** Only `chart_type` has a known closed enum of allowed
values in v1.0 spec. All other string fields are open-ended text content. `lang` (BCP-47)
is a T3 concern. `chart_type`'s allowed values are: `bar`, `line`, `pie`, `scatter`,
`area`, `stacked-bar`, `stacked-area` (7 values, per the chart.rs description field).

### Tier 3 — Format-Specific (T3): DEFERRED to Wave-TBD

**What:** Field value has correct type but fails a format constraint: BCP-47 language
tags (`lang`), hex color names, URL format, path format.

**T3 is deferred** because:
1. Language tag validation requires a BCP-47 parser or allowlist — non-trivial.
2. Color validation: E-PAR-015 already catches invalid hex colors at parse time; the
   validation boundary is handled.
3. Path format: E-VAL-012 (ImagePathValidator) already handles path traversal
   containment — a separate concern.
4. T3 adds a dependency on format-specific validators that have no natural home in
   `FieldType` without making `FieldType` very complex.

**Anchor for T3 deferral:** a future story in Wave-TBD to be created when the format
validator patterns are established. The `expected_type` field on `FieldDef` is forward-
compatible: a `FieldType::Format(FormatKind)` variant can be added later without
breaking existing `match expected_type` callsites (the match is exhaustive — adding a
variant requires updating match arms, but this is a safe and visible compile-time break
that prevents silent ignore of new cases).

---

## Section 3: Error-Code Plan

### Pre-Registration Collision Check

Codes currently allocated in the E-VAL namespace (from error-taxonomy.md v2.19 and
grep of crates/):

- E-VAL-001: appears in test code (unrelated namespace — confirmed this is actually in a
  different subsystem, a false-positive from the grep; no E-VAL-001 appears in
  error-taxonomy.md production code)
- E-VAL-003: confirmed only in test-helper code, not emitted in production paths
- E-VAL-011: `ValueRangeValidator` (numeric range)
- E-VAL-012: `ImagePathValidator` (path traversal)
- E-VAL-101, E-VAL-102: informal codes in `validate_fields` (required absent/empty) —
  NOT yet formally registered in error-taxonomy.md per the note in taxonomy v2.18
- W-VAL-103: informal code in `validate_fields` (unknown field)
- W-VAL-002: in `slideforge-plugin-api/src/traits/validator.rs` (validator warning)

**Codes to formally register in this story:**

| Proposed Code | Severity | Exit (strict) | Message Format | New? |
|---|---|---|---|---|
| E-VAL-101 | broken | 2 | `Required field '<name>' missing on <type> slide. Required fields for '<type>': [<list>].` | Formal registration only — code already in use |
| E-VAL-102 | broken | 2 | `Required field '<name>' is empty on <type> slide. Required fields for '<type>': [<list>].` | Formal registration only — code already in use |
| W-VAL-103 | cosmetic | 0 | `Unknown field '<key>' for slide type '<type>'. Known fields: [<list>].` | Formal registration only — code already in use |
| E-VAL-104 | broken | 2 | `Field '<name>' on <type> slide has wrong type: expected <expected>, got <actual>. See DSL reference for valid types.` | NEW |

The taxonomy registration for E-VAL-101/102/W-VAL-103 is debt acknowledged in taxonomy
v2.18 ("will be formally registered in a future spec burst"). This story resolves that
debt as part of the same burst — no extra work.

### E-VAL-104 Collision Check

From the taxonomy, codes E-VAL-013 through E-VAL-103 are unallocated (the namespace
jumps from E-VAL-012 to E-VAL-101). E-VAL-104 is confirmed free. The choice of E-VAL-104
(rather than E-VAL-013) follows the existing informal grouping convention: 1xx codes are
the `validate_fields` schema family (101 absent, 102 empty, 103 unknown, 104 type-mismatch).
This grouping is coherent and should be formally established.

From `grep -rE "E-VAL-104" crates/`: zero matches. Confirmed free.

### Message Format for E-VAL-104

```
Field 'chart_type' on chart slide has wrong type: expected string, got integer. See DSL reference for valid chart_type values.
```

For T2 (OneOf violation), the message distinguishes type-match from value-match:

```
Field 'chart_type' on chart slide has disallowed value "donut": allowed values are [bar, line, pie, scatter, area, stacked-bar, stacked-area].
```

This is produced by the same E-VAL-104 code with a different message branch. No
separate code for T2 — the distinction is in the message text, consistent with the
pattern established by E-BRD-001 (three BrandError variants, one code) and E-EXP-003
(multiple PdfExportError variants, one code).

---

## Section 4: Impact Analysis

### 4a. Backward Compatibility of FieldDef Extension

Adding `expected_type: Option<FieldType>` to `FieldDef` is ADDITIVE but not source-
compatible with struct literal construction sites (Rust requires all fields in a struct
literal). There are two paths:

**Path A (chosen):** Add `#[non_exhaustive]` to `FieldDef` and add
`impl Default for FieldDef`. All existing construction sites use `FieldDef { name: ...,
description: ..., required: ..., default_value: ... }` — they must be updated to include
`expected_type: None`. This is mechanical but the compiler catches every missed site.

Count of `FieldDef { ` construction sites (from grep of slide type files + common_optional_fields):

- `common_optional_fields()` in `slide_types/mod.rs`: 9 sites
- 34 slide type files, each with 1-5 `FieldDef` constructions:
  - `blank.rs`: 0 required fields (no FieldDef construction for required), ~9 optional
    via `common_optional_fields()` (not inline construction — uses the function)
  - Most slide types: 1-5 inline FieldDef constructions for required fields + inline
    optional fields; the common_optional_fields() fields are not re-constructed inline
  - Estimated: approximately 80-120 individual `FieldDef { ... }` struct literals across
    all 34 slide type files + `mod.rs` common fields

This is a mechanical multi-site update, not a design change. The compiler will catch
every missed site.

**Path B (rejected):** Use `FieldDef::builder()` to avoid struct literal updates. This
adds more code without benefit — the struct literal approach with `expected_type: None`
is clearer.

### 4b. How Many Slide Types Need Annotation to Make the Feature Meaningful

The feature is meaningful as infrastructure even before all annotations are added — once
E-VAL-104 is emitted for ANY slide type's annotated fields, the contract is delivered.
However, to produce user value at v1.0 quality, annotations must cover fields where type
errors are plausible in practice.

**Fields where type mismatch is a realistic author mistake:**

| Field | Slide Types | Realistic Mistake | FieldType |
|---|---|---|---|
| `value` | `progress_bar` | `value: "75%"` (string instead of int) | `Int` |
| `chart_type` | `chart` | `chart_type: 42` (int instead of string) | `OneOf([...])` |
| `decorative` | all (common optional) | `decorative: "yes"` (string instead of bool) | `Bool` |
| `components` | `weighted_composite` | `components: "..."` (string instead of list) | `List` |
| `kpis` | `kpi_dashboard` | `kpis: "..."` (string instead of list) | `List` |
| `milestones` | `roadmap` | similar | `List` |
| `rows` | `matrix` | similar | `List` |
| `items` | `agenda`, `toc` | similar | `List` |
| `members` | `team` | similar | `List` |

All remaining string fields (`title`, `subtitle`, `label`, `quote`, `attribution`, etc.)
are `FieldType::Str` — but since all DSL field values parsed without explicit type syntax
arrive as `Value::Str` by default (per parser behavior), annotating them as `FieldType::Str`
does NOT generate E-VAL-104 in practice. The annotation is correct but produces no new
diagnostics. It is documentation value, not validation value. Therefore:

**Annotation priority for meaningful validation:**
- Priority-1 (produces real E-VAL-104 signals): `value` (progress_bar Int), `decorative`
  (Bool, common optional), `components`/`kpis`/`milestones`/`rows`/`items`/`members`
  (List types), `chart_type` (OneOf).
- Priority-2 (correctness annotation, no new signals): all `title`, `label`, `subtitle`,
  `quote`, etc. (Str type — already guaranteed by parser default).

**Recommendation:** annotate Priority-1 fields in this story (8-12 FieldDef sites across
5-8 slide types). Annotate Priority-2 (remaining string fields) in the same story as
mechanical completion — but do NOT block the story on exhaustive P2 coverage. The
contract is: any `FieldType` that is non-`Str` gets annotated in scope. String fields
get `None` or explicit `Str` — either is correct behavior.

### 4c. Files and Crates Touched

| Crate | Files | Nature of Change |
|---|---|---|
| `slideforge-plugin-api` | `src/traits/slide_type.rs` | Add `FieldType` enum + `FieldDef.expected_type` field + `type_matches()` |
| `slideforge-plugin-api` | `src/slide_types/registry.rs` | Add E-VAL-104 arm to `validate_fields` |
| `slideforge-plugin-api` | `src/slide_types/mod.rs` | Update `common_optional_fields()` — add `expected_type: None` (or `Some(Bool)` for `decorative`) to 9 FieldDef sites |
| `slideforge-plugin-api` | `src/slide_types/progress_bar.rs` | Annotate `value: FieldType::Int` |
| `slideforge-plugin-api` | `src/slide_types/chart.rs` | Annotate `chart_type: FieldType::OneOf([...])` |
| `slideforge-plugin-api` | `src/slide_types/weighted_composite.rs` | Annotate `components: FieldType::List` |
| `slideforge-plugin-api` | `src/slide_types/kpi_dashboard.rs`, `roadmap.rs`, `matrix.rs`, `agenda.rs`, `toc.rs`, `team.rs`, `survey_results.rs`, `risk_register.rs`, `process_flow.rs`, `timeline.rs`, `org_chart.rs` | Add `expected_type: None` to all FieldDef sites + annotate List fields |
| `slideforge-plugin-api` | All remaining 23 slide type files | Add `expected_type: None` to all FieldDef sites (mechanical) |
| `slideforge-validate` | `src/value_range.rs` | No change — E-VAL-011 stays; ValueRangeValidator is orthogonal (range checks, not type checks) |
| `slideforge-plugin-api` | `src/traits/slide_type.rs` tests | Add E-VAL-104 tests |
| Error taxonomy | `.factory/specs/prd-supplements/error-taxonomy.md` | Register E-VAL-101, E-VAL-102, W-VAL-103 (formal), allocate E-VAL-104 (new) |

**File count touched:** approximately 36 files (34 slide type files + registry.rs + mod.rs + slide_type.rs).

**The number that might look alarming is misleading.** 28 of the 34 slide type file
changes are ONE-LINE additions of `expected_type: None` per FieldDef construction site.
They are mechanical compiler-directed edits, not design work. The substantive work is in
6 files: `slide_type.rs` (FieldType enum), `registry.rs` (E-VAL-104 logic), and 4-6
slide types with Priority-1 annotations.

### 4d. Breaking Change Detection

`FieldDef` is a `pub` struct used by external plugin authors (the `SlideType` trait is
the extension surface for plugins per the product brief). Adding a new field changes the
construction API: external plugin code that constructs `FieldDef { name: ..., ... }`
will fail to compile unless `#[non_exhaustive]` is applied.

**With `#[non_exhaustive]` on `FieldDef`:** Existing struct literal constructions OUTSIDE
the crate are NOT valid (non_exhaustive prohibits external struct literals). However,
`FieldDef` is currently constructible with struct literal syntax from outside the crate
(no non_exhaustive today). Applying `#[non_exhaustive]` IS a breaking change for any
external plugin that constructs `FieldDef`. This is a SemVer-breaking change.

**Resolution:** slideforge v1.0 is pre-1.0 (crates at 0.x). SemVer allows breaking
changes in 0.x minor bumps. The factory is building toward v1.0; no published API exists
yet. This is not a blocker.

For post-v1.0: add `FieldDef::new(name, description, required, default_value) -> Self`
as the stable construction API (no `expected_type` parameter — uses `None`), and
`FieldDef::with_type(name, description, required, default_value, expected_type) -> Self`
as the typed variant. Internal code uses struct literals. External plugins use
constructors. This migration is a one-time effort properly scoped in this story.

### 4e. Snapshot and Test Impact

- Existing tests in `registry.rs` construct slides with `FieldValue::Literal(Value::Str(...))`.
  These will not be affected by E-VAL-104 until slide types are annotated.
- After annotation, existing tests that construct slides with the wrong type for an
  annotated field will receive additional diagnostics. None of the current tests pass a
  wrong-typed value for a field with a known-type annotation (the current tests use
  valid values). Zero test regressions expected.
- New Red Gate tests required: ~8-12 tests in `registry.rs` (one per E-VAL-104 scenario
  per annotated field type).
- Snapshot tests: no snapshot test in `slideforge-plugin-api` covers `validate_fields`
  output format. No snapshot test regressions.

---

## Section 5: Effort Estimate

### Story Point Reference Frame

- STORY-086 (field-to-block threading): 21 pts — complex threading pipeline, multiple
  crates, ADR required, non-trivial architectural decision about pipeline stage.
- STORY-087 (color-coded slide types): 13 pts — 3 new slide types + new validator +
  label-check extension. Affected ~8 files substantively.
- STORY-003 (31 slide type implementations): 8 pts — template work across 31 files.

### This Work

**Option A: Single story (recommended)**

| Component | Effort |
|---|---|
| FieldType enum design + `#[non_exhaustive]` on FieldDef + Default impl | ~1 day |
| `type_matches()` + E-VAL-104 in `validate_fields` | ~0.5 day |
| Red Gate tests for E-VAL-104 (8-12 tests) | ~0.5 day |
| Priority-1 annotations (6-8 slide types, 10-15 FieldDef sites) | ~1 day |
| Mechanical `expected_type: None` addition across remaining 28 slide types | ~0.5 day |
| FieldDef constructor API + update all construction sites | ~0.5 day |
| Error taxonomy registration (E-VAL-101/102/W-VAL-103 formal + E-VAL-104 new) | ~0.5 day |
| BC authoring + ADR + story | ~0.5 day |
| **Total** | **~5 days → 8 story points** |

**Option B: Split into two stories**

- STORY-A: FieldDef infrastructure (FieldType enum, `expected_type` field, E-VAL-104 in
  `validate_fields`, tests, taxonomy registration). Points: 5. Delivers the framework.
- STORY-B: Per-slide-type annotation sweep (Priority-1 + Priority-2 for all 34 types).
  Points: 5. Delivers the user value.

Split rationale: the infrastructure story has a clean merge boundary — `expected_type:
None` everywhere, E-VAL-104 logic exists but never fires. The annotation story then
populates the `expected_type` fields to make E-VAL-104 fire.

This split is reasonable but not necessary — the work is sequential and coherent within
a single story. The main risk of a single story is context exhaustion on the mechanical
annotation sweep. Given the factory's per-story worktree model, this is manageable.

**Recommended: single story, 8 points.** If mid-story context becomes a concern, the
implementer can deliver Option A (infrastructure only) as a partial PR and create STORY-B
for annotations — but this should be the exception, not the plan.

---

## Section 6: Risks

### R1: `#[non_exhaustive]` on FieldDef changes external plugin construction API

**Severity:** Medium. **Likelihood:** Low (no published plugins yet; v1.0 pre-release).
**Mitigation:** Add `FieldDef::new()` and `FieldDef::with_type()` constructors in the
same story. Document in rustdoc that struct literal construction is internal.

### R2: Annotating `decorative: FieldType::Bool` in common_optional_fields() triggers E-VAL-104 on any slide using `decorative: "true"` (string instead of bool)

**Severity:** High (real DSL authors would hit this if the DSL parser emits
`Value::Str("true")` for the `decorative:` field). **Likelihood:** Depends on parser
behavior for boolean fields. Need to confirm: does the parser emit `Value::Bool(true)` or
`Value::Str("true")` for `decorative: true` in DSL?

From the DSL spec and Q1 decisions: `NO` stays `"NO"` (string); `1.10` stays `"1.10"`
(string). But `true` and `false` as bare keywords — per the DSL grammar — are parsed as
`Value::Bool`. This is consistent with the "no implicit coercion" rule: coercion is
banned, but EXPLICIT bool literals in the DSL (`true`/`false`) parse to `Value::Bool`.
The YAML-style coercion of `"NO"` → bool is the forbidden pattern, not the parsing of
explicit `true`/`false` keywords.

**Conclusion:** `decorative: true` in DSL → `Value::Bool(true)` → passes `FieldType::Bool`.
`decorative: "yes"` in DSL → `Value::Str("yes")` → fails `FieldType::Bool` → E-VAL-104.
This is correct behavior. No risk.

### R3: Some slide types have fields like `data`, `kpis`, `components` that may receive `Value::List` OR `Value::Map` depending on data source

**Severity:** Medium. **Likelihood:** Medium. `data` on `chart` slides may be either a
list of series objects (List) or a data-source reference (Str? or Map?). If `data` is
polymorphic, annotating it `FieldType::List` would cause false E-VAL-104.

**Mitigation:** Use `expected_type: None` (no constraint = `FieldType::Any`) for
genuinely polymorphic fields (`data`, `rows` on matrix, `cells`). Only annotate fields
with a clearly fixed type. This is a judgment call per field — requires the implementer
to check the evaluator's handling of each field during annotation sweep.

### R4: Wave 5 sequencing conflict

**Severity:** Low. Wave 5 has STORY-088 (bullets list-literal DSL syntax) as a dependency
of several stories. Adding a new story that touches `slideforge-plugin-api` widely could
create a merge conflict surface with Wave 5 stories. However: `slideforge-plugin-api` is
also touched by STORY-088 (indirectly, via eval consuming `Value::List`). The FieldDef
changes are in `slide_types/` and `traits/` — the merge surface is limited.

---

## Section 7: Recommendation

### RECOMMENDATION: Fold into Wave 5, early slot

**Rationale:**

1. **Sizing:** At 8 story points, this is a medium story. Wave 5 currently has 21
   stories at 122 points (5.8 pts average). Adding 1 story of 8 pts raises Wave 5 to
   22 stories / 130 points — a 6.5% point increase. This is within the wave's capacity
   if the human agrees.

2. **Dependency fit:** This story depends on STORY-003 (31 slide type implementations)
   and STORY-086 (field-to-block threading, which established that `FieldValue::Literal`
   is the correct way to read field values in validators). Both are merged. It has no
   dependency on STORY-088 (bullets DSL) or any other pending Wave 5 story. It can be
   slot-1 in Wave 5 without blocking anything.

3. **Not blocking a current PR or gate:** Wave 4 is in its final merges (STORY-087
   merged; STORY-088 is pending). There is no in-flight blocker that requires this
   feature NOW. The gap (type mismatch producing no diagnostic) is a known quality hole,
   not a correctness regression or a gate failure.

4. **Against deliver-now-standalone:** The Wave 4 gate (Gate 5) is nearly closed.
   Inserting a new story into an already-gating wave adds re-gate overhead. The story
   touches 36 files — re-running the full gate suite on a modified version of
   `slideforge-plugin-api` is real cost. Better to batch this with Wave 5 which already
   has several `slideforge-plugin-api` touches.

5. **E-VAL-101/102/W-VAL-103 registration debt:** The taxonomy formally acknowledges
   these codes as unregistered ("future spec burst"). That debt can be closed in the same
   Wave 5 spec burst that allocates E-VAL-104. Closing it in a Wave-4 standalone story
   would be premature — it would re-open the gate for a doc-only taxonomy change.

**If the human prefers deliver-now-standalone:** The work is self-contained and does not
break any existing passing tests. The gate overhead is manageable. The argument would be
that closing the type-validation gap before Wave 5 improves the quality baseline that
Wave 5 CLI stories (which invoke the compiler end-to-end) rely on. This is a valid
position.

**Position summary:**
- Fold into Wave 5 (early slot, before STORY-088): LOW RISK, correct fit, no regression risk.
- Deliver now standalone: ACCEPTABLE, adds gate overhead, justified if Wave 5 start is distant.
- Do not deliver (defer beyond Wave 5): NOT RECOMMENDED. This is a v1.0 quality gap —
  the production-grade default prohibits deferring it past v1.0.
