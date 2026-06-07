---
document_type: adr
adr_id: ADR-020
title: FieldDef type-annotation extension — FieldType enum + expected_type field for schema-driven field-value validation
status: accepted
date: 2026-06-07
subsystems_affected:
  - SS-14
  - SS-03
supersedes: null
superseded_by: null
related_adrs:
  - ADR-006
  - ADR-016
traces_to:
  - .factory/specs/behavioral-contracts/BC-1.18.001.md
  - .factory/specs/domain-spec/capabilities.md#CAP-022
  - .factory/planning/d-fielddef-type-validation-proposal.md
  - .factory/specs/prd-supplements/error-taxonomy.md
---

# ADR-020: FieldDef Type-Annotation Extension — FieldType Enum + expected_type Field for Schema-Driven Field-Value Validation

## Context

`FieldDef` in `slideforge-plugin-api/src/traits/slide_type.rs` carries four fields:
`name: Arc<str>`, `description: Arc<str>`, `required: bool`, and
`default_value: Option<Value>`. It carries no type information. The `validate_fields`
function in `src/slide_types/registry.rs` enforces presence (E-VAL-101), non-emptiness
(E-VAL-102), and no-unknown-keys (W-VAL-103), but it does NOT check whether a field's
runtime `Value` variant is appropriate for that field. A `progress_bar` slide receiving
`value: "seventy-five"` (a `Value::Str`) silently passes field validation, then reaches
`lay_out()` where it either panics (`.unwrap()` violation) or produces garbage layout.

A separate `ValueRangeValidator` (STORY-087) provides ad-hoc type awareness for the
three color-coded slide types, but it is imperative per-type code in `slideforge-validate`,
not schema-driven. It does not generalize to the remaining 31 slide types.

The `LayoutError::FieldTypeMismatch` variant in `slide_type.rs` shows that the architects
already anticipated type errors as a failure mode in `lay_out()`. That variant is the
defensive late-stage fallback; moving type checking to Stage 5 (pre-layout, schema-driven)
is the correct architectural resolution. BC-1.18.001 formalizes the behavioral contract;
E-VAL-104 is registered in error-taxonomy.md v2.20 (commit def8bb74). This ADR records
the design decisions that make the implementation possible. The scoping proposal at
`.factory/planning/d-fielddef-type-validation-proposal.md` (commit 5d60a39b) provides
the full analysis from which these decisions derive.

ADR-006 established the plugin-first architecture; the `SlideType` trait and `FieldDef`
are the schema surface of that architecture. ADR-016 confirmed that all bundled `SlideType`
implementations live in `slideforge-plugin-api/src/slide_types/`, which is exactly where
the new `FieldType` enum and `FieldDef.expected_type` field reside. ADR-019 established
Stage 2b (field-to-block threading); type validation at Stage 5 is downstream of and
independent from that stage — both coexist in the same `build_inner` pass sequence.

## Decisions

### Decision 1: FieldType enum in slideforge-plugin-api

We define a `FieldType` enum in `slideforge-plugin-api/src/traits/slide_type.rs`,
co-located with `FieldDef`, deriving `Debug`, `Clone`, `PartialEq`, `Eq`, and `Hash`:

```rust
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

Variants align 1:1 with `slideforge-types::Value` discriminants (Str, Int, Float, Bool,
List, Map) plus `Any` (opt-out) and `OneOf` (T2 enum validation). `FieldType` is a
SCHEMA-level constraint; `Value` is a RUNTIME-level representation. They are separate
types at different abstraction levels in different crates — `FieldType` is NOT added to
`slideforge-types`. `FieldType::Float` is included because the `weighted_composite`
`weight` field is `Value::Float`; omitting it would make the enum incomplete against
the actual runtime type system.

### Decision 2: FieldDef gains expected_type: Option\<FieldType\>

`FieldDef` gains one new optional field:

```rust
pub expected_type: Option<FieldType>,
```

`None` is semantically identical to `Some(FieldType::Any)`: the validator skips type
checking for that field. This is the backward-compatible default for all existing
`FieldDef` construction sites.

### Decision 3: #[non_exhaustive] on FieldDef + stable constructor API

`FieldDef` is marked `#[non_exhaustive]`. External plugin authors MUST NOT use struct
literal syntax to construct `FieldDef`; the stable construction surface is:

- `FieldDef::new(name, description, required, default_value) -> Self` — returns a
  `FieldDef` with `expected_type: None`.
- `FieldDef::with_type(name, description, required, default_value, expected_type) -> Self`
  — returns a `FieldDef` with the specified `FieldType`.

This is the go-forward policy for all plugin-api schema structs: new fields that would
break external struct literal construction are paired with `#[non_exhaustive]` and a
constructor API. Internal construction sites within `slideforge-plugin-api` may use
struct literals; they must include `expected_type: None` or `Some(..)` explicitly.

All existing internal `FieldDef { ... }` construction sites across all 34 slide type
files and `src/slide_types/mod.rs` are updated mechanically to include `expected_type`.
The compiler catches every missed site — no site is silently compatible via default.

`impl Default for FieldDef` is NOT provided. `FieldDef` has no meaningful "empty"
default — `name` and `description` being empty strings is not a valid FieldDef state.
The named constructors are the correct API.

### Decision 4: No SlideType trait signature change

The `SlideType` trait methods `required_fields() -> &[FieldDef]` and
`optional_fields() -> &[FieldDef]` are unchanged. Adding `expected_type` to `FieldDef`
is a struct-field addition, not a trait method change. All 34 bundled `SlideType`
implementations implement the same trait interface.

### Decision 5: type_matches pure helper function

A free function `type_matches(value: &Value, expected: &FieldType) -> bool` lives in
`slideforge-plugin-api/src/traits/slide_type.rs` adjacent to `FieldType`. It is a
deterministic, side-effect-free, pure function: it performs no I/O, reads no globals,
and is fully determined by its two arguments. This makes it Kani-amenable for formal
verification in Phase 6. Kani can exhaustively verify the `type_matches` truth table
over the finite `FieldType` variant space within a bounded model.

The `validate_fields` function in `registry.rs` calls `type_matches` for every field
whose `expected_type` is `Some(T)` where `T != FieldType::Any`. E-VAL-104 is emitted
when `type_matches` returns `false`. Error accumulation per DI-018 is preserved — all
E-VAL-104 diagnostics for a slide are collected before returning.

### Decision 6: Validation scope tiers and T3 deferral

Two validation tiers are in scope:

- **T1 (type-mismatch):** `Value` variant does not match `FieldType` discriminant.
  Covered by `type_matches` returning `false` for the non-`OneOf` variants.
- **T2 (OneOf violation):** `Value::Str(s)` where `s` is not in
  `FieldType::OneOf(variants)`. Included because `OneOf` is a variant in the enum; a
  variant that is silently ignored is a design error, not a scope reduction.

Tier 3 (format-specific constraints: BCP-47 language tags, hex color format, URL
format) is DEFERRED to a future story. T3 requires format-specific validators with no
natural home in `FieldType` without significantly complicating the enum. The
`FieldType` enum is forward-compatible with T3: a future `FieldType::Format(FormatKind)`
variant forces compile-time callsite updates via exhaustive match, preventing silent
ignore of the new variant. The T3 deferral is anchored to a future Wave-TBD story;
it is NOT a "ship and polish later" deferral — the mechanism is fully in place, only
the format-specific variant definitions are absent.

### Decision 7: Backward compatibility via pre-v1.0 SemVer

Applying `#[non_exhaustive]` to `FieldDef` is a SemVer-breaking change for external
consumers that construct `FieldDef` via struct literal syntax. This is acceptable:
slideforge is pre-v1.0 (all crates are at `0.x`), no plugins are published externally,
and SemVer allows breaking changes in `0.x` minor version bumps. For post-v1.0 plugin
authors, the `FieldDef::new()`/`FieldDef::with_type()` constructor API is the stable
surface, and `#[non_exhaustive]` prevents the struct-literal anti-pattern from entering
external plugin code before v1.0 locks the API.

## Rationale

**Why FieldType in plugin-api, not slideforge-types?** `slideforge-types` is the
runtime IR crate — it defines `Value`, `Deck`, `LaidOutDeck`, `Brand`, and related
types. `FieldType` is a schema constraint, not a runtime value. Adding `FieldType` to
`slideforge-types` would make the IR crate aware of plugin-api schema concerns, which
is the wrong dependency direction. `FieldDef` already lives in `slideforge-plugin-api`
alongside the `SlideType` trait; `FieldType` belongs there by colocation with its
consumer (BC-1.18.001 Architecture Anchors; proposal §1.1).

**Why Hash + Eq + Clone?** These three derives are mandatory for all types in the
slideforge IR type system (CLAUDE.md Conventions: "All IR types implement Hash + Eq + Clone
from day 1"). `FieldType` annotates `FieldDef`, which is returned from `SlideType` trait
methods and lives in the plugin registry. Hash + Eq are required for comemo-compatible
memoization in the future; they are also required for Kani bounded model checking of
`type_matches`. Deriving these from day one prevents a future forced migration of all
match callsites.

**Why Option\<FieldType\> not FieldType?** `Option<FieldType>` with `None` as the
skip-sentinel is the additive choice: all existing 80-120 `FieldDef` construction sites
can be mechanically updated to `expected_type: None` without changing behavior. A
non-optional `FieldType` would require every slide type to declare `FieldType::Any`
explicitly — equivalent in behavior but more verbose with no benefit. The `None` ≡
`Any` identity is a documented invariant (BC-1.18.001 Invariant 1).

**Why #[non_exhaustive] not a builder pattern?** A full builder (`.name()`, `.required()`,
`.with_type()`) adds substantial boilerplate for a struct with five fields. Two named
constructors (`::new` and `::with_type`) cover 100% of actual usage patterns: you either
know the type at construction time or you do not. The `#[non_exhaustive]` attribute
forces external callers to use the constructors without requiring a builder chain. This
is the lighter-weight, idiomatic Rust approach (see `std::fs::OpenOptions` as a precedent
for the builder pattern; `FieldDef` is simpler and does not need that level of
configurability).

**Why no Default impl?** A `FieldDef` with empty `name` and `description` is not a
valid schema entry. Providing `Default` would allow `FieldDef { required: true, ..Default::default() }`
with an empty name — a construction path that should not exist. Named constructors
make the required arguments explicit.

**Why Float in the enum?** The `weighted_composite` slide type requires `weight` fields
to be `Value::Float` (e.g., `weight: 0.4`). Omitting `FieldType::Float` would mean the
enum is incomplete against the `Value` type system — a gap that would produce a false
`FieldType::Any` annotation on `weight`, silently allowing `weight: "heavy"` to pass
type checking. The production-grade default prohibits silent type-system gaps.

**Why keep LayoutError::FieldTypeMismatch?** It remains as a defensive internal
invariant for unannotated/polymorphic fields that reach `lay_out()`. E-VAL-104 at
Stage 5 catches annotated fields before layout; the `LayoutError` variant guards the
unannotated paths. Removing it would weaken the defense-in-depth posture.

## Consequences

### Positive

- Schema-driven type validation covers all 34 slide types uniformly once `expected_type`
  annotations are populated — no per-type imperative type-checking code required.
- `type_matches` is a pure function with no side effects, making it Kani-provable in
  Phase 6. Formal verification of the type-dispatch truth table is achievable within
  bounded model checking (finite `FieldType` variant space).
- `FieldType::OneOf` subsumes the closed-enum validation for `chart_type` (7 allowed
  values) without requiring a separate error code or validator — T2 is included in T1's
  infrastructure at near-zero marginal cost.
- `#[non_exhaustive]` on `FieldDef` enforces the constructor API going forward. Future
  fields added to `FieldDef` will not silently compile in external plugin code — the
  compile-time break is the intended signal that the constructor must be updated.
- `FieldType::Format(FormatKind)` can be added in a future story for T3 format
  validation with zero risk of silent regression — exhaustive match arms enforce
  callsite updates.
- Priority-1 annotations (10 field sites across 9 slide types — `progress_bar.value`,
  `chart.chart_type`, `decorative` common optional, `weighted_composite.components`,
  `kpi_dashboard.kpis`, `roadmap.milestones`, `matrix.rows`, `agenda.items`,
  `toc.items`, `team.members`) produce real E-VAL-104 signals in practice, closing
  the most realistic author type-error paths.

### Negative / Trade-offs

- All 80-120 `FieldDef { ... }` struct literal construction sites in
  `slideforge-plugin-api/src/slide_types/` must be updated to include
  `expected_type: None` (or `Some(..)` for annotated fields). This is mechanical
  compiler-directed work — the compiler catches every missed site — but it touches
  approximately 36 files. Context budget for STORY-089 must account for this sweep.
- Applying `#[non_exhaustive]` is a SemVer-breaking change for hypothetical external
  consumers. Mitigated by the pre-v1.0 status and the absence of published plugins.
  The constructor API is the go-forward stable surface.
- `FieldType::Float` strictly requires `Value::Float` for float-typed fields. Authors
  writing `weight: 1` (an integer literal) will receive E-VAL-104 and must correct to
  `weight: 1.0`. This is intentional per the no-implicit-coercion rule, but it is a
  usability friction point documented in BC-1.18.001 Invariant 3.
- Polymorphic fields (`data` on chart slides, `cells` on matrix, etc.) MUST use
  `expected_type: None` — the annotation sweep requires per-field judgment, not
  mechanical coverage. Annotating a polymorphic field with the wrong `FieldType`
  produces false E-VAL-104 positives. This judgment call is the primary complexity
  risk in STORY-089.

### Status as of 2026-06-07

Decision accepted and pending implementation. BC-1.18.001 is active (committed
def8bb74). E-VAL-104 is registered in error-taxonomy.md v2.20. STORY-089 is the
implementing story (Wave 5, slot 1). No production code changes have been made yet —
this is a spec-first ADR preceding implementation.

## Alternatives Considered

- **Option A — Add FieldType to slideforge-types, not plugin-api.** Rejected: wrong
  dependency direction. `slideforge-types` is the runtime IR crate (leaf crate with
  zero workspace dependencies). Making it aware of schema-constraint types (`FieldType`)
  from the plugin-api domain violates the IR crate's scope contract (SS-15 per ADR-016).

- **Option B — Keep type checking imperative per slide type (status quo).** Rejected:
  the existing `ValueRangeValidator` in `slideforge-validate` is already evidence that
  ad-hoc per-type type checking produces duplication and gaps. Generalizing to 34 slide
  types with imperative code would require 34 separate type-check implementations and
  a centralized dispatch that is functionally equivalent to `FieldType` but harder to
  verify. The schema-driven approach is superior in every measurable dimension.

- **Option C — FieldType as a non-exhaustive enum from day one (no `#[non_exhaustive]` on FieldDef).** Rejected: `#[non_exhaustive]` on `FieldType` itself (not `FieldDef`) is
  correct and will be applied, but that does not address the struct-literal construction
  problem for `FieldDef`. The two `#[non_exhaustive]` usages serve different purposes:
  `FieldType` is non-exhaustive to allow future variants; `FieldDef` is non-exhaustive
  to enforce the constructor API.

- **Option D — Builder pattern for FieldDef (FieldDefBuilder).** Rejected: five-field
  structs do not warrant a full builder. Two named constructors (`::new`, `::with_type`)
  cover all usage patterns with far less boilerplate. The builder pattern introduces an
  additional type, additional import, and additional cognitive overhead for no benefit
  over named constructors.

- **Option E — Include T3 format validation in scope.** Rejected for STORY-089 scope.
  BCP-47 validation requires an allowlist or parser; URL format validation requires
  regex or a URL crate dependency; hex color validation is already handled by E-PAR-015
  at parse time. T3 adds dependencies and complexity that have no natural home in
  `FieldType` without a significant enum extension. The `Format(FormatKind)` slot in
  the future enum is preserved; T3 is deferred to Wave-TBD.

## Source / Origin

- **Scoping proposal:** `.factory/planning/d-fielddef-type-validation-proposal.md`
  (commit 5d60a39b) — full design analysis, risk assessment, effort estimate, and
  scoping recommendation. All decisions in this ADR derive from the proposal.
- **Behavioral contract:** `.factory/specs/behavioral-contracts/BC-1.18.001.md`
  (committed def8bb74) — postconditions, invariants, edge cases, and test vectors for
  `validate_fields` E-VAL-104 enforcement. BC-1.18.001 traces to CAP-022
  (Compile-Time Content Validation) and DI-018 (error accumulation).
- **Error taxonomy:** `.factory/specs/prd-supplements/error-taxonomy.md` v2.20
  (committed def8bb74) — E-VAL-104 allocated; E-VAL-101, E-VAL-102, W-VAL-103 formally
  registered in the same burst.
- **Capability anchor:** `domain-spec/capabilities.md` CAP-022 — "Compile-Time Content
  Validation" is the L2 capability that FieldDef type annotations extend.
- **Related ADRs:** ADR-006 (plugin-first architecture — `FieldDef` is the schema
  surface of the `SlideType` plugin trait); ADR-016 (bundled `SlideType` impls in
  `slideforge-plugin-api/src/slide_types/` — the annotation sweep is in that directory).
- **Stories:** STORY-089 (Wave 5 slot 1, implementing story).
