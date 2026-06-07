---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-087
title: "Color-Coded Slide Types: status, progress_bar, weighted_composite (Registration + LabelCheck WCAG Enforcement)"
epic: EPIC-01
wave: 4
points: 13
priority: P1
tdd_mode: strict
status: in-progress
spec_version: "1.3"
last_updated: "2026-06-06"
changelog:
  - version: "1.3"
    date: "2026-06-06"
    note: "F-087-P1-001 architect adjudication: value-range validation moves from SlideType::lay_out() to dedicated ValueRangeValidator (Stage 5, slideforge-validate/src/value_range.rs), wired after LabelCheckValidator in slideforge::registry.rs; lay_out() in progress_bar.rs and weighted_composite.rs is now geometry-only; ACs re-targeted to BuildError::ValidationFailed with E-VAL-011; File Structure Requirements updated; Tasks T4.2/T4.3 updated."
  - version: "1.2"
    date: "2026-06-06"
    note: "Wave 5→4 pull-in (human-authorized) to close F-G3-HIGH-003 within Wave 4."
target_module: slideforge-plugin-api, slideforge-syntax, slideforge-layout, slideforge-validate
subsystems: [SS-14, SS-01, SS-05, SS-03]
behavioral_contracts: [BC-1.17.001, BC-1.17.002, BC-1.17.003]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023]
closes_findings: [F-G3-HIGH-003]
depends_on:
  - STORY-086
  - STORY-003
  - STORY-017
blocks: []
estimated_days: 5
---

# STORY-087: Color-Coded Slide Types — status, progress_bar, weighted_composite

## Uncertainty Resolution (2026-06-06)

This spec was corrected against the real codebase on `develop` @ 030dec6c. Full resolution:
`.factory/specs/wave4-expanded-scope-uncertainty-resolution.md` (D3, D4).

Key corrections applied:
- **Real `SlideType` trait methods** (from `crates/slideforge-plugin-api/src/traits/slide_type.rs`):
  `id()`, `required_fields()`, `optional_fields()`, `layout_name()`, `lay_out()`.
  There is NO `keyword()`, `region_spec()`, or `validate_fields()` method on the trait.
- **Custom value-range validation** (progress_bar [0,100]; weighted_composite weight>0/score[0,100])
  is performed by `ValueRangeValidator` at Stage 5 (pre-layout) in `slideforge-validate`, NOT in
  `lay_out()`. `lay_out()` in `progress_bar.rs` and `weighted_composite.rs` is geometry-only
  (F-087-P1-001: value-range in lay_out() is dead code — lay_out() is never called by build_inner).
- **Region registration function** is `region_frames_for(keyword, w, h) -> Option<Vec<Frame>>`
  in `crates/slideforge-layout/src/regions.rs`, NOT `slide_type_regions()` / `RegionSpec`.
- **`severity_cards` keyword gap:** present in `COLOR_CODED_TYPES` and `regions.rs` but
  ABSENT from `SLIDE_TYPE_KEYWORDS` in `keywords.rs`. STORY-087 must add it.
- **Registration path:** each type is registered via `r.register(Box::new(TypeImpl::new()))`
  in `crates/slideforge-plugin-api/src/slide_types/registry.rs::Default::default()`.
- **Template pattern:** `crates/slideforge-plugin-api/src/slide_types/stat_callout.rs`.

## Subsystem Anchor Justifications

- SS-14 (Plugin API, `slideforge-plugin-api`) owns the `SlideType` trait implementations
  for `status`, `progress_bar`, and `weighted_composite`. Per ADR-006 (plugin-first
  architecture) and the established pattern in STORY-003 (31 SlideType implementations)
  and STORY-083/084/085 (EPIC-21 plugin surface completeness), all slide type
  implementations live in `slideforge-plugin-api/src/slide_types/`. These three types
  are the remaining color-coded types not yet registered in the type set.

- SS-01 (DSL Parser, `slideforge-syntax`) owns the keyword registration in `keywords.rs`
  (the `SLIDE_TYPE_KEYWORDS` PHF set). Without keyword registration, parsing `slide status:`
  produces `E-PAR-NNN` (unknown slide type keyword). The keyword set is a PHF perfect hash
  built at compile time; new keywords are added to the `phf_set!` macro in `keywords.rs`.

- SS-05 (Layout Engine, `slideforge-layout`) owns the region map in `regions.rs`. Each
  slide type keyword requires a match arm in `region_frames_for(slide_type_keyword, w, h)
  -> Option<Vec<Frame>>` returning a static frame skeleton. Without this entry,
  `layout::run` returns `None` from `region_frames_for` and emits
  `LayoutError::UnknownSlideType`. The three new types each require frame skeletons
  defining their static geometry (color indicator frame + label/title frame + optional
  bar background frame). Value-proportional geometry (e.g., progress_bar fill width) is
  NOT computed here — that goes in `SlideType::lay_out()` which has field access.

- SS-03 (Validation, `slideforge-validate`) owns `LabelCheckValidator`. The finding
  F-G3-HIGH-003 confirmed that `COLOR_CODED_TYPES` in `LabelCheckValidator` already
  references `"status"`, `"progress_bar"`, and `"weighted_composite"` — but these types
  are not registered, so `LayoutError::UnknownSlideType` fires before the validator runs.
  This story completes the registration so LabelCheck becomes functional for these three
  types. No change to the LabelCheck logic is needed — only to type registration and
  the keyword/regions infrastructure.

## Dependency Anchor Justifications

- Depends on STORY-086 (Stage 2b field-to-block threading): `status`, `progress_bar`,
  and `weighted_composite` are content-bearing slide types. Their `title` and `label`
  fields must thread into `Slide.blocks` as `ContentBlock::Text` entries so that layout
  and exporters can render them. Without Stage 2b, these slide types would produce
  content-empty output — the same defect that motivated STORY-086. STORY-087 must not
  merge before STORY-086 is live.

- Depends on STORY-003 (31 SlideType Implementations): establishes the `SlideType` trait
  implementation pattern, the existing PHF keyword set, and the `regions.rs` region map
  structure that this story extends. The three new types follow exactly the same pattern.

- Depends on STORY-017 (Color-Coded Label + WCAG Contrast Enforcement): STORY-017
  delivered `LabelCheckValidator` and the `COLOR_CODED_TYPES` constant. This story
  activates LabelCheck for `status`, `progress_bar`, and `weighted_composite` by
  ensuring they are registered slide types (so the pipeline reaches the label validator).
  The LabelCheck logic itself is correct; no modification needed.

- Blocks nothing: these are new standalone slide types. No existing Wave 5 stories depend
  on them. They are Wave 5 P1 scope — gated on STORY-086 (which must pass Wave 4 Gate 3
  re-check first).

## Summary

Wave 4 Gate finding F-G3-HIGH-003 identified that `LabelCheckValidator.COLOR_CODED_TYPES`
references `"status"`, `"progress_bar"`, and `"weighted_composite"` — but these three
slide type keywords are not registered in `SLIDE_TYPE_KEYWORDS`, the `regions.rs` region
map, or the `slideforge-plugin-api` slide type set. As a result, the pipeline emits
`LayoutError::UnknownSlideType` before the label validator runs, making LabelCheck dead
for these types end-to-end.

The human authorized `status`, `progress_bar`, and `weighted_composite` as v1.0 in-scope
types on 2026-06-05 (recorded in ADR-019 `human_gate_resolved` field). This story
implements all three types using the same plugin-first pattern as the 31 existing types.
The implementation template is `crates/slideforge-plugin-api/src/slide_types/stat_callout.rs`.

Each new type requires four registration steps:
1. `SlideType` trait implementation in `slideforge-plugin-api/src/slide_types/<type>.rs`
   with methods: `id()`, `required_fields()`, `optional_fields()`, `layout_name()`, `lay_out()`.
2. Keyword registration: add to `SLIDE_TYPE_KEYWORDS` PHF set in `keywords.rs`.
3. Region skeleton: add match arm in `region_frames_for()` in `regions.rs`.
4. Registry wiring: call `r.register(Box::new(<Type>::new()))` in `registry.rs::Default::default()`.

This story ALSO adds `"severity_cards"` to `SLIDE_TYPE_KEYWORDS` — an existing gap where
`severity_cards` appears in `COLOR_CODED_TYPES` and `regions.rs` but is absent from the PHF
keyword set (D4 gap). The AC-024 consistency assertion will fail without this fix.

**Combined story justification:** The three types share the same implementation pattern.
Splitting into three stories would require three identical scaffolding steps. The 13-point
estimate reflects non-trivial scope: `weighted_composite` adds per-component label iteration
to LabelCheck, `progress_bar` and `weighted_composite` value-range validation (0–100;
weight>0/score[0,100]) is enforced by the new `ValueRangeValidator` at Stage 5 via
`E-VAL-011` (F-087-P1-001: moved from lay_out() which is never called by build_inner),
and all three require region frame skeletons and geometry-only `lay_out()` implementations.

## Narrative

As a slideforge user building status dashboards, project trackers, and vendor scorecards,
I want to use `slide status:`, `slide progress_bar:`, and `slide weighted_composite:` in
my `.sf` files,
so that my color-coded status indicators, progress bars, and composite scoring slides are
rendered in PPTX, PDF, and DOCX with accessible textual labels co-encoding the
color-conveyed meaning (per WCAG AA), and missing labels produce compile errors before
any inaccessible output is produced.

## Behavioral Contracts

| BC | Title | Version | Role in This Story |
|----|-------|---------|-------------------|
| BC-1.17.001 | status Slide Type Requires title + label | v1.0 | Primary: defines the `status` type registration, required fields, E-A11-002 error on missing label, LabelCheck enforcement pre-layout on Slide.fields |
| BC-1.17.002 | progress_bar Slide Type Requires title + label + value(0–100) | v1.0 | Primary: defines the `progress_bar` type registration, required fields including value range [0,100] validation, E-A11-002 for missing label |
| BC-1.17.003 | weighted_composite Slide Type Requires title + label + components[]; both top-level and per-component labels mandatory | v1.0 | Primary: defines the `weighted_composite` type registration, component iteration for per-component label enforcement, total error accumulation (DI-018) |

## Acceptance Criteria

### status Slide Type

#### AC-001 — status slide type is a registered keyword and parses without error
A `.sf` source containing `slide status: title "Project Alpha" label "On Track"` is
parsed without error. The parser produces a `SlideNode` with `slide_type = "status"`.
The keyword `"status"` is present in `SLIDE_TYPE_KEYWORDS` (the PHF set in `keywords.rs`).
No `E-PAR-NNN` (unknown keyword) is emitted.
(traces to BC-1.17.001 precondition 3 — keyword registered in SLIDE_TYPE_KEYWORDS;
BC-1.17.001 invariant 5 — keyword registration required before pipeline runs)

#### AC-002 — status slide with title + label builds successfully under strict=true
`slideforge::build(FIXTURE_STATUS_VALID, BuildOptions { strict: true, format: "pptx", .. })`
returns `Ok(BuildOutput)`. The fixture contains `slide status: title "Project Alpha" label "On Track"`.
The PPTX output contains a slide with the label text "On Track" accessible to screen readers
(present in an OOXML element, not hidden via CSS-equivalent styling in OOXML).
No E-A11-002 is emitted.
(traces to BC-1.17.001 postcondition 2 — when both present, valid;
BC-1.17.001 postcondition 3 — label text visible in PPTX)

#### AC-003 — status slide missing label → E-A11-002 in strict mode
A fixture with `slide status: title "Project Beta"` (no label field) built with
`strict: true` returns `Err(BuildError::ValidationFailed)` where the diagnostics list
contains at least one `Diagnostic { code: "E-A11-002", .. }` whose message includes
"status" and "Project Beta". No output bytes are produced.
(traces to BC-1.17.001 postcondition 2 — E-A11-002 for missing label;
BC-1.17.001 EC-001)

#### AC-004 — status slide empty label → E-A11-002
A fixture with `slide status: title "Project Gamma" label ""` (empty string label)
returns `Err(BuildError::ValidationFailed)` with code "E-A11-002". Empty label is not
a valid co-encoding.
(traces to BC-1.17.001 EC-004 — empty label equivalent to absent;
BC-1.17.001 invariant 1 — label mandatory, no opt-out)

#### AC-005 — status slide missing label + strict=false → Ok, warning emitted
A fixture with `slide status: title "Project Delta"` (no label) built with
`strict: false` returns `Ok(BuildOutput)`. A `tracing::warn!` is emitted containing
"E-A11-002" or "missing label". Output is produced but is non-conformant WCAG AA.
(traces to BC-1.17.001 EC-006 — warn-only mode)

#### AC-006 — LabelCheck for status reads Slide.fields, not Slide.blocks (Stage 2b independence)
A unit test calls `LabelCheckValidator::validate()` on a `Deck` where the `status` slide
has `Slide.fields["label"] = None` but `Slide.blocks = vec![]` (pre-Stage-2b state).
The validator still fires E-A11-002. LabelCheck must NOT depend on `Slide.blocks` being
populated by Stage 2b.
(traces to BC-1.17.001 invariant 3 — LabelCheck operates on Slide.fields;
BC-1.17.001 postcondition 4 — Stage 5 pre-layout pass on Slide.fields directly)

### progress_bar Slide Type

#### AC-007 — progress_bar slide type is registered and parses
A `.sf` source containing `slide progress_bar: title "Completion" label "75% done" value 75`
parses without error. The keyword `"progress_bar"` is present in `SLIDE_TYPE_KEYWORDS`.
(traces to BC-1.17.002 precondition 3 — keyword registered;
BC-1.17.002 invariant 6 — keyword must be registered before pipeline runs)

#### AC-008 — progress_bar with valid title + label + value(75) builds successfully
`slideforge::build(FIXTURE_PROGRESS_VALID, BuildOptions { strict: true, format: "pptx", .. })`
returns `Ok(BuildOutput)`. The fixture contains `slide progress_bar: title "Sprint 4" label "75% complete" value 75`.
The PPTX output contains the label text "75% complete" and a visual bar element. No
E-A11-002 is emitted.
(traces to BC-1.17.002 postcondition 4 — valid fields → Ok;
BC-1.17.002 canonical test vector — happy path)

#### AC-009 — progress_bar missing label → E-A11-002
A fixture with `slide progress_bar: title "Sprint 4" value 75` (no label) returns
`Err(BuildError::ValidationFailed)` with code "E-A11-002" identifying "progress_bar"
and "Sprint 4" in the message.
(traces to BC-1.17.002 postcondition 2 — E-A11-002 for missing label;
BC-1.17.002 EC-001)

#### AC-010 — progress_bar value=0 is valid (boundary)
A fixture with `slide progress_bar: title "Start" label "0% done" value 0` returns
`Ok(BuildOutput)`. The value 0 is within the valid [0, 100] range.
(traces to BC-1.17.002 EC-005 — value 0 is valid boundary;
BC-1.17.002 invariant 3 — value bounded 0-100 inclusive)

#### AC-011 — progress_bar value=100 is valid (boundary)
A fixture with `slide progress_bar: title "Done" label "100% complete" value 100`
returns `Ok(BuildOutput)`. The value 100 is within the valid range.
(traces to BC-1.17.002 EC-006 — value 100 is valid boundary)

#### AC-012 — progress_bar value=101 → BuildError::ValidationFailed with E-VAL-011
A call to `slideforge::build()` with a fixture containing `slide progress_bar: title "Over" label "Done" value 101`
returns `Err(BuildError::ValidationFailed)` where the diagnostics list contains at least one
`Diagnostic { code: "E-VAL-011", .. }` with message `"progress_bar value must be between 0 and 100; got 101."`.
The error is emitted by `ValueRangeValidator` at Stage 5 (pre-layout). No output bytes are produced.
**NOTE (F-087-P1-001):** Enforcement is NOT in `lay_out()` — it is in the pipeline-wired `ValueRangeValidator`.
(traces to BC-1.17.002 postcondition 3 — value outside [0,100] is compile error, enforced by ValueRangeValidator at Stage 5;
BC-1.17.002 EC-003)

#### AC-013 — progress_bar value=-1 → BuildError::ValidationFailed with E-VAL-011
A call to `slideforge::build()` with a fixture containing `slide progress_bar: title "Negative" label "Done" value -1`
returns `Err(BuildError::ValidationFailed)` where the diagnostics list contains at least one
`Diagnostic { code: "E-VAL-011", .. }` with message `"progress_bar value must be between 0 and 100; got -1."`.
The error is emitted by `ValueRangeValidator` at Stage 5.
**NOTE (F-087-P1-001):** Enforcement is NOT in `lay_out()` — it is in the pipeline-wired `ValueRangeValidator`.
(traces to BC-1.17.002 EC-004 — negative value is out of range, enforced by ValueRangeValidator at Stage 5)

### weighted_composite Slide Type

#### AC-014 — weighted_composite slide type is registered and parses
A `.sf` source containing a valid `slide weighted_composite:` block parses without error.
The keyword `"weighted_composite"` is present in `SLIDE_TYPE_KEYWORDS`.
(traces to BC-1.17.003 precondition 3 — keyword registered;
BC-1.17.003 invariant 2 — title mandatory)

#### AC-015 — weighted_composite with valid fields builds successfully
A fixture with:
```
slide weighted_composite:
  title "Vendor A"
  label "Overall: Good (78/100)"
  components:
    - name "Quality" weight 0.4 score 85 label "Excellent"
    - name "Price" weight 0.6 score 72 label "Acceptable"
```
returns `Ok(BuildOutput)`. The PPTX output contains the aggregate label "Overall: Good (78/100)"
and per-component labels "Excellent" and "Acceptable" in accessible OOXML elements.
(traces to BC-1.17.003 postcondition 6 — valid fields → visual output with all labels;
BC-1.17.003 canonical test vector — happy path)

#### AC-016 — weighted_composite missing top-level label → E-A11-002
The same fixture as AC-015 but with `label` (top-level) removed returns
`Err(BuildError::ValidationFailed)` with one E-A11-002 for the missing aggregate label.
(traces to BC-1.17.003 postcondition 2 — top-level label required;
BC-1.17.003 EC-001)

#### AC-017 — weighted_composite component missing label → E-A11-002 for that component
A fixture where component 2 ("Price") has no `label` field returns
`Err(BuildError::ValidationFailed)` with E-A11-002 identifying "weighted_composite.component"
"Price" in the message.
(traces to BC-1.17.003 postcondition 4 — per-component label required;
BC-1.17.003 EC-002)

#### AC-018 — weighted_composite accumulates 3 E-A11-002 when top + 2 components missing labels
A fixture with top-level label absent and both components missing labels returns
`Err(BuildError::ValidationFailed)` with exactly 3 E-A11-002 diagnostics: one for the
top-level missing label, one per component. Error accumulation does not bail on first.
(traces to BC-1.17.003 postcondition 5 — all errors accumulated before returning;
BC-1.17.003 invariant 5 — error accumulation is total across all components;
BC-1.17.003 EC-003)

#### AC-019 — weighted_composite empty components list → BuildError::ValidationFailed with E-VAL-011
A call to `slideforge::build()` with a fixture containing `components: []` (empty list) returns
`Err(BuildError::ValidationFailed)` where the diagnostics list contains at least one
`Diagnostic { code: "E-VAL-011", .. }` with message indicating "weighted_composite requires at least one component".
The error is emitted by `ValueRangeValidator` at Stage 5.
**NOTE (F-087-P1-001):** Enforcement is NOT in `lay_out()` — it is in the pipeline-wired `ValueRangeValidator`.
(traces to BC-1.17.003 postcondition 3 — components list required and non-empty, enforced by ValueRangeValidator at Stage 5;
BC-1.17.003 EC-004)

#### AC-020 — weighted_composite component score=101 → BuildError::ValidationFailed with E-VAL-011
A call to `slideforge::build()` with a fixture containing a component with `score 101` returns
`Err(BuildError::ValidationFailed)` where the diagnostics list contains at least one
`Diagnostic { code: "E-VAL-011", .. }` with message `"weighted_composite components[<idx>].score must be between 0 and 100; got 101."`.
The error is emitted by `ValueRangeValidator` at Stage 5.
**NOTE (F-087-P1-001):** Enforcement is NOT in `lay_out()` — it is in the pipeline-wired `ValueRangeValidator`.
(traces to BC-1.17.003 EC-007 — score out of range, enforced by ValueRangeValidator at Stage 5;
BC-1.17.003 invariant 7 — score bounded 0-100)

#### AC-021 — weighted_composite component weight=0 → BuildError::ValidationFailed with E-VAL-011
A call to `slideforge::build()` with a fixture containing a component with `weight 0` returns
`Err(BuildError::ValidationFailed)` where the diagnostics list contains at least one
`Diagnostic { code: "E-VAL-011", .. }` with message `"weighted_composite components[<idx>].weight must be positive; got 0."`.
The error is emitted by `ValueRangeValidator` at Stage 5.
**NOTE (F-087-P1-001):** Enforcement is NOT in `lay_out()` — it is in the pipeline-wired `ValueRangeValidator`.
(traces to BC-1.17.003 EC-006 — zero weight, enforced by ValueRangeValidator at Stage 5;
BC-1.17.003 invariant 6 — weight must be positive)

#### AC-022 — LabelCheck for weighted_composite iterates component sub-fields without Stage 2b
A unit test calls `LabelCheckValidator::validate()` on a `Deck` with a `weighted_composite`
slide where `Slide.fields["components"]` is a resolved list of 2 components, one missing
its `label` sub-field. `Slide.blocks = vec![]` (pre-Stage-2b). The validator fires E-A11-002
for the specific component. LabelCheck does NOT require Stage 2b to have run.
(traces to BC-1.17.003 postcondition 7 — Stage 5 pre-layout reads Slide.fields;
BC-1.17.003 invariant 8 — LabelCheck is a Stage 5 pre-layout check)

### All Three Types — Registration Completeness

#### AC-023 — LayoutError::UnknownSlideType is NOT emitted for any of the three types
Unit tests construct `LaidOutDeck` runs for `status`, `progress_bar`, and
`weighted_composite` slides with all required fields present. None of the three builds
returns `LayoutError::UnknownSlideType`. This is the direct closure of F-G3-HIGH-003.
(traces to BC-1.17.001 invariant 5, BC-1.17.002 invariant 6, BC-1.17.003 precondition 3 —
keyword must be registered before the pipeline runs; F-G3-HIGH-003 finding description)

#### AC-024 — All COLOR_CODED_TYPES entries are present in SLIDE_TYPE_KEYWORDS (consistency invariant)
A unit test asserts that `LabelCheckValidator::COLOR_CODED_TYPES` (the `&[&str]` constant)
contains `"status"`, `"progress_bar"`, `"weighted_composite"`, AND `"severity_cards"`.
The unit test also asserts that ALL entries in `COLOR_CODED_TYPES` are present in
`SLIDE_TYPE_KEYWORDS` — no type in the label-check guard is missing from the parser's
keyword set. Before this story, `"severity_cards"` is in `COLOR_CODED_TYPES` but NOT in
`SLIDE_TYPE_KEYWORDS` (D4 gap). After this story, all four types appear in both sets.
This eliminates the dead-guard scenario that constituted F-G3-HIGH-003.
(traces to BC-1.17.001 postcondition 5 — type in COLOR_CODED_TYPES AND SLIDE_TYPE_KEYWORDS;
BC-1.17.002 postcondition 3 same; BC-1.17.003 precondition 3 same;
D4 gap resolution — severity_cards added to SLIDE_TYPE_KEYWORDS)

## Architecture Mapping

| Component | Crate | File | Change Type | Pure/Effectful |
|-----------|-------|------|-------------|---------------|
| `StatusSlideType` | `slideforge-plugin-api` | `src/slide_types/status.rs` (NEW) | New SlideType impl | Pure |
| `ProgressBarSlideType` | `slideforge-plugin-api` | `src/slide_types/progress_bar.rs` (NEW) | New SlideType impl | Pure |
| `WeightedCompositeSlideType` | `slideforge-plugin-api` | `src/slide_types/weighted_composite.rs` (NEW) | New SlideType impl | Pure |
| `slide_types/mod.rs` export | `slideforge-plugin-api` | `src/slide_types/mod.rs` | Add 3 pub mod + re-exports | Pure |
| PHF keyword registration | `slideforge-syntax` | `src/keywords.rs` | Add 3 keywords to `SLIDE_TYPE_KEYWORDS` phf_set! | Pure |
| Region frame skeletons | `slideforge-layout` | `src/regions.rs` | Add 4 match arms to `region_frames_for()`: `"status"`, `"progress_bar"`, `"weighted_composite"`, and `"severity_cards"` (existing keyword gap, D4) | Pure |
| `lay_out()` implementations | `slideforge-plugin-api` | `src/slide_types/status.rs`, `progress_bar.rs`, `weighted_composite.rs` | Geometry-only frame production. Value-range validation is NOT here (F-087-P1-001: moved to ValueRangeValidator). `lay_out()` returns frame skeleton; does NOT return FieldTypeMismatch for value-range violations. | Pure |
| `ValueRangeValidator` | `slideforge-validate` | `src/value_range.rs` (NEW) | Stage 5 pre-layout validator: checks progress_bar value∈[0,100] and weighted_composite weight>0/score∈[0,100]; emits E-VAL-011; accumulates all errors (DI-018). Registered in `slideforge::registry.rs` after LabelCheckValidator. | Pure |
| LabelCheck `COLOR_CODED_TYPES` verification | `slideforge-validate` | `src/label_check.rs` | Verify consistency (no code change expected if guard is already present; add assertions) | Pure |
| Plugin registry registration | `slideforge-plugin-api` | `src/slide_types/registry.rs` | Add `r.register(Box::new(StatusSlideType::new()))` etc. in `Default::default()` | Effectful (registry assembly) |

**Forbidden Dependencies:**
- `slideforge-plugin-api::slide_types::status/progress_bar/weighted_composite` MUST NOT
  import `slideforge-pptx`, `slideforge-pdf`, `slideforge-docx`, or `slideforge-html`.
  Slide type implementations are pure-core: they define semantics, required fields, and
  validation rules, but do not render output format bytes. Rendering is exporter-side.
- New slide type modules MUST implement the `SlideType` trait via the public API only.
  No direct imports of slideforge-eval or slideforge-layout internals (ADR-006 dog-food rule).

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~5,000 |
| BC-1.17.001 full text | ~2,500 |
| BC-1.17.002 full text | ~2,500 |
| BC-1.17.003 full text | ~2,500 |
| wave4-content-threading-assessment §4 (F-G3-HIGH-003 analysis) | ~1,000 |
| STORY-003 (31 SlideType impl pattern) excerpts | ~2,000 |
| STORY-017 (LabelCheckValidator) excerpts | ~1,500 |
| `slideforge-plugin-api/src/slide_types/` existing files (pattern reference) | ~2,000 |
| `slideforge-syntax/src/keywords.rs` (PHF set, current) | ~1,000 |
| `slideforge-layout/src/regions.rs` (region map structure) | ~2,000 |
| `slideforge-validate/src/label_check.rs` (COLOR_CODED_TYPES + validate logic) | ~1,500 |
| `slideforge-validate/src/value_range.rs` (NEW — ValueRangeValidator) | ~1,000 |
| `slideforge/src/registry.rs` (validator registration site) | ~500 |
| Unit test files (new) | ~4,500 |
| E2E fixture files (new) | ~2,000 |
| Tool outputs (compiler messages, test results) | ~3,000 |
| **TOTAL ESTIMATED** | **~35,000 tokens** |

35,000 tokens is ~17.5% of a 200k context window — well within the 20-30% per-story budget.

## Previous Story Intelligence

This story follows STORY-086 (Stage 2b threading) and STORY-003 (31 SlideType implementations).

From STORY-003 and STORY-083/084/085 cascades:
- The real `SlideType` trait surface (from `crates/slideforge-plugin-api/src/traits/slide_type.rs`):
  `id() -> &'static str`, `required_fields() -> &[FieldDef]`, `optional_fields() -> &[FieldDef]`,
  `layout_name() -> &'static str`, `lay_out(&self, slide, brand, canvas) -> Result<LaidOutSlide, LayoutError>`.
  There is NO `keyword()`, NO `region_spec()`, NO `validate_fields()` trait method.
  Template: `crates/slideforge-plugin-api/src/slide_types/stat_callout.rs`.
- Custom value-range validation (progress_bar [0,100]; weighted_composite weight>0/score[0,100])
  goes in `ValueRangeValidator` (Stage 5, `slideforge-validate/src/value_range.rs`), NOT in `lay_out()`.
  **F-087-P1-001 finding:** `lay_out()` is never called by `build_inner` — it is only called directly
  in unit tests, making any validation there dead code (TD-VSDD-059 paper-fix pattern).
  `lay_out()` in `progress_bar.rs` and `weighted_composite.rs` is geometry-only.
  The `validate_fields` FREE FUNCTION in `registry.rs` checks required/optional field
  presence only — not value ranges.
- PHF keyword registration uses the `phf_set!` macro in `keywords.rs`. Forgetting to
  add the keyword here causes the parser to emit E-PAR-NNN (unknown slide type) even
  when the SlideType impl exists. This is the root cause of F-G3-HIGH-003. Also add
  `"severity_cards"` which has the same gap.
- The region function is `region_frames_for(keyword, w, h) -> Option<Vec<Frame>>` in
  `regions.rs`. New match arms return `Some(vec![...])` with static `FrameContent::Empty`
  frames (the geometry skeleton). An arm missing here causes `LayoutError::UnknownSlideType`.
  Value-proportional geometry (e.g., progress_bar fill width proportional to `value` field)
  is computed in `lay_out()`, NOT in `region_frames_for()` (which has no field access).
- Registration: `r.register(Box::new(TypeImpl::new()))` in `registry.rs::Default::default()`.
- LabelCheck reads `Slide.fields["label"]` directly (Stage 5 pre-layout). It does NOT
  read `Slide.blocks`. Do not add any Stage 2b dependency to LabelCheck — it must work
  even when blocks are empty.

From STORY-017 (WCAG contrast + label check):
- E-A11-002 is the canonical error code for missing color-coded label. Use it exactly.
  Do not introduce a new error code for these types.
- The LabelCheck `COLOR_CODED_TYPES` constant already contains `"status"`, `"progress_bar"`,
  `"weighted_composite"`. Do not modify this constant — just ensure the types are registered.
- The `weighted_composite` type requires iterating the `components` list from
  `Slide.fields["components"]` (resolved by eval as a `Value::List` of `Value::Map`
  entries). The LabelCheck must read the `label` sub-field of each component map entry.
  This iteration logic may need to be added to `LabelCheckValidator::validate()`.

From LESSON-16 (STORY-083):
- All new public items require rustdoc. The `SlideType` trait implementations must have
  module-level and type-level doc comments. The `missing_docs` lint fires on new public types.

From LESSON-13 (STORY-049):
- Positive content vectors are mandatory. AC-002, AC-008, and AC-015 each assert that
  the label text is VISIBLE in the output (present in OOXML / PDF structure), not just
  that the build returned Ok. Do not reduce these to structure-only checks.

## Architecture Compliance Rules

1. **ADR-006 (plugin-first):** All three slide types MUST be implemented as `SlideType`
   trait implementations in `slideforge-plugin-api`. Implementing them directly in
   `slideforge-syntax`, `slideforge-layout`, or any other crate is a contract violation.
   Every bundled plugin dog-foods the public trait API.

2. **ADR-019 Decision 5.3 / BC-1.17.001 invariant 3 / BC-1.17.002 postcondition 5 /
   BC-1.17.003 invariant 8:** LabelCheck for all three types operates at Stage 5
   (pre-layout) on `Slide.fields`, NOT on `Slide.blocks`. The label field is a semantic
   field that resolves during eval. Implementing LabelCheck to depend on `Slide.blocks`
   populated by Stage 2b is an architectural violation.

3a. **F-087-P1-001 / ADR-006 / TD-VSDD-059:** Value-range validation for `progress_bar`
    and `weighted_composite` MUST live in `ValueRangeValidator` (Stage 5,
    `slideforge-validate/src/value_range.rs`), NOT in `SlideType::lay_out()`.
    `build_inner` does not call `lay_out()` — validation in `lay_out()` is unreachable
    dead code and constitutes a TD-VSDD-059 paper-fix pattern. Any `lay_out()` method
    that returns `Err(LayoutError::FieldTypeMismatch)` for value-range reasons is
    WRONG and must not be written. `lay_out()` is geometry-only.

3. **DI-018 (error accumulation):** For `weighted_composite`, missing labels on N components
   must produce N separate E-A11-002 diagnostics. The validator MUST NOT bail on the first
   missing component label. Use the accumulation pattern established in `slideforge-validate`
   (collect all diagnostics into a `Vec<Diagnostic>`, then return the full list).

4. **BC-1.17.001/002/003 invariant — no decorative opt-out for label:** The `decorative: true`
   field is an opt-out for visual media alt text (BC-5.01.001). It is NOT an opt-out for the
   WCAG color co-encoding `label` requirement on color-coded types. The `SlideType`
   implementation MUST NOT accept `decorative: true` as a substitute for `label "..."` on
   status, progress_bar, or weighted_composite slides.

5. **`#![forbid(unsafe_code)]`** — No `unsafe` blocks in new SlideType implementations
   or in the LabelCheck `weighted_composite` component iteration extension.

6. **Zero `.unwrap()` in non-test code.** Field access via `fields.get("label")` returns
   `Option`; handle `None` as missing label, not as a panic.

7. **`Hash + Eq + Clone` on all struct types.** If new structs are introduced (e.g., a
   `ComponentSpec` for `weighted_composite`), they must derive these traits for
   comemo compatibility.

## Library and Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `phf` | 0.11 (workspace) | PHF perfect hash for `SLIDE_TYPE_KEYWORDS` extension |
| `tracing` | 0.1 (workspace) | `tracing::warn!` for optional fields (e.g., missing value field in progress_bar before validation fires) |
| `slideforge-types` | workspace path | `ContentBlock`, `Value`, `FieldValue`, `Diagnostic` types |
| `slideforge-plugin-api` | workspace path | `SlideType` trait — these impls live IN this crate |

No new external dependencies required. All dependencies are already in the workspace.

## File Structure Requirements

Files to CREATE:
```
crates/slideforge-plugin-api/src/slide_types/status.rs           [StatusSlideType impl — geometry-only lay_out()]
crates/slideforge-plugin-api/src/slide_types/progress_bar.rs     [ProgressBarSlideType impl — geometry-only lay_out();
                                                                   NO value-range validation here (F-087-P1-001)]
crates/slideforge-plugin-api/src/slide_types/weighted_composite.rs [WeightedCompositeSlideType impl — geometry-only lay_out();
                                                                     NO weight/score/non-empty validation here (F-087-P1-001)]
crates/slideforge-validate/src/value_range.rs                    [ValueRangeValidator impl — Stage 5 pre-layout validator;
                                                                   checks progress_bar value∈[0,100] and
                                                                   weighted_composite weight>0/score∈[0,100] and non-empty components;
                                                                   emits E-VAL-011; accumulates all errors (DI-018);
                                                                   validator_id = "value-range"]
crates/slideforge-plugin-api/tests/color_coded_slide_types.rs    [integration tests for AC-001..AC-024]
```

Files to MODIFY:
```
crates/slideforge-plugin-api/src/slide_types/mod.rs              [add pub mod status; pub mod progress_bar; pub mod weighted_composite; re-exports]
crates/slideforge-plugin-api/src/slide_types/registry.rs         [add r.register(Box::new(StatusSlideType::new())),
                                                                   r.register(Box::new(ProgressBarSlideType::new())),
                                                                   r.register(Box::new(WeightedCompositeSlideType::new()))
                                                                   in Default::default()]
crates/slideforge-syntax/src/keywords.rs                         [add "status", "progress_bar", "weighted_composite", AND "severity_cards" (existing gap, D4)
                                                                   to phf_set! in SLIDE_TYPE_KEYWORDS;
                                                                   update test_bc_1_09_008_is_slide_type_keyword_all_31_types to include severity_cards]
crates/slideforge-layout/src/regions.rs                          [add 4 match arms to region_frames_for(): "status", "progress_bar", "weighted_composite",
                                                                   "severity_cards" (already in registry/regions but absent from PHF — verify/add)]
crates/slideforge-validate/src/lib.rs                            [add pub mod value_range; re-export ValueRangeValidator (F-087-P1-001)]
crates/slideforge-validate/src/label_check.rs                    [extend validate() to iterate weighted_composite components; add AC-022, AC-024 consistency assertion unit tests]
crates/slideforge/src/registry.rs                                [register ValueRangeValidator after LabelCheckValidator:
                                                                   builder.register_validator(Box::new(ValueRangeValidator));
                                                                   (F-087-P1-001 — this is the wiring point that makes value-range reachable from build())]
```

**CRITICAL (F-087-P1-001): lay_out() in progress_bar.rs and weighted_composite.rs is geometry-only.**
- Do NOT add value-range checks (`!(0..=100).contains(&value_int)` etc.) in `lay_out()`.
- `lay_out()` in these modules MUST NOT return `Err(LayoutError::FieldTypeMismatch)` for
  value-range violations — that was dead code because `build_inner` never calls `lay_out()`.
- All value-range enforcement lives in `ValueRangeValidator` at Stage 5.
- The `lay_out()` test in `test_all_31_types_lay_out_returns_ok` MUST still pass — a
  geometry-only `lay_out()` returns `Ok` for any field values.

**Region frame skeleton guidance for `region_frames_for()` (static geometry only — no field access):**
- `status`: two `FrameContent::Empty` frames — a color indicator frame (left/top strip EMUs)
  and a label+title text frame (main body). Follow the two-frame pattern used by `severity_cards`
  already in `regions.rs`.
- `progress_bar`: three `FrameContent::Empty` frames — title frame (top), bar background frame
  (full-width fixed height), and label text frame (below bar). The bar fill frame with
  value-proportional width is constructed in `ProgressBarSlideType::lay_out()`, NOT here.
  `region_frames_for` returns the STATIC skeleton; `lay_out()` adds the dynamic bar fill frame.
- `weighted_composite`: title frame + aggregate label frame at top; N fixed-height component
  row frames. Since N is not known at region-skeleton time, return a reasonable fixed set
  (e.g., title + label + 5 component row slots as Empty frames). `lay_out()` constructs the
  actual per-component geometry from the resolved `components` field.

All frames returned by `region_frames_for` have `FrameContent::Empty`; `lay_out()` overrides
them with actual content and may add additional frames.

## Tasks

- [ ] **T1 — Red Gate: write all failing tests first**
  - [ ] T1.1: Write E2E tests for `status` type (AC-001..AC-006) using `slideforge::build()`. All must FAIL (type not registered → UnknownSlideType).
  - [ ] T1.2: Write E2E tests for `progress_bar` type (AC-007..AC-013). All must FAIL.
  - [ ] T1.3: Write E2E tests for `weighted_composite` type (AC-014..AC-022). All must FAIL.
  - [ ] T1.4: Write consistency assertion unit tests (AC-023, AC-024). Must FAIL.
  - [ ] T1.5: Verify Red Gate density ≥0.5 before implementation starts.

- [ ] **T2 — Keyword registration (slideforge-syntax)**
  - [ ] T2.1: Add `"status"`, `"progress_bar"`, `"weighted_composite"`, AND `"severity_cards"` (D4 gap fix)
    to `SLIDE_TYPE_KEYWORDS` phf_set! in `keywords.rs`. Four additions total.
  - [ ] T2.2: Update the test `test_bc_1_09_008_is_slide_type_keyword_all_31_types` in `keywords.rs`
    (or its test file) to include `"severity_cards"` in the expected set.
  - [ ] T2.3: Run `cargo build -p slideforge-syntax` — verify PHF compiles without error.
  - [ ] T2.4: Run parser-level AC-001, AC-007, AC-014 tests — verify parse no longer emits E-PAR-NNN.

- [ ] **T3 — Region frame skeleton registration (slideforge-layout)**
  - [ ] T3.1: Add match arms for `"status"`, `"progress_bar"`, `"weighted_composite"` to `region_frames_for()`
    in `regions.rs`. Each arm returns `Some(vec![...])` with static `FrameContent::Empty` frames.
    Also verify `"severity_cards"` is already present in `region_frames_for()` (it should be per D4).
  - [ ] T3.2: Frame skeletons: status = [color_indicator Empty, label_title Empty];
    progress_bar = [title Empty, bar_bg Empty, label Empty];
    weighted_composite = [title Empty, aggregate_label Empty, row0..row4 Empty (5 fixed slots)].
    Value-proportional geometry goes in `lay_out()`, NOT here.
  - [ ] T3.3: Run `cargo build -p slideforge-layout` — verify no UnknownSlideType for the three types.
  - [ ] T3.4: Run AC-023 — must now pass.

- [ ] **T4 — SlideType implementations (slideforge-plugin-api)**
  Real trait methods (from `crates/slideforge-plugin-api/src/traits/slide_type.rs`):
  `id()`, `required_fields()`, `optional_fields()`, `layout_name()`, `lay_out()`.
  Template: `crates/slideforge-plugin-api/src/slide_types/stat_callout.rs`.
  - [ ] T4.1: Create `status.rs` — implement `StatusSlideType`:
    `id()` returns `"status"`, `required_fields()` returns `[FieldDef { name: "title", .. }, FieldDef { name: "label", .. }]`,
    `optional_fields()` returns `&[]`, `layout_name()` returns the PPTX layout name string.
    `lay_out()` extracts title + label from `slide.fields`; returns
    `Err(LayoutError::MissingRequiredField { field: "label", .. })` when label absent/empty.
    Value-range validation is not needed for status (label is a string). Rustdoc all public items.
  - [ ] T4.2: Create `progress_bar.rs` — implement `ProgressBarSlideType`:
    `id()` returns `"progress_bar"`, `required_fields()` includes "title", "label", "value".
    `lay_out()` is GEOMETRY-ONLY (F-087-P1-001): extract `value` from `slide.fields` to compute
    bar fill width = `bar_bg_width * value / 100` and emit as a Shape frame. Do NOT validate
    value range in `lay_out()` — range enforcement is in `ValueRangeValidator` at Stage 5.
    `lay_out()` must return `Ok(LaidOutSlide { .. })` even when `value` is absent or invalid
    (ValueRangeValidator fires before lay_out() is reached in a correct pipeline).
    Rustdoc all public items.
  - [ ] T4.3: Create `weighted_composite.rs` — implement `WeightedCompositeSlideType`:
    `id()` returns `"weighted_composite"`, `required_fields()` includes "title", "label", "components".
    `lay_out()` is GEOMETRY-ONLY (F-087-P1-001): extract `components` as `Value::List` of `Value::Map`
    entries and construct per-component row frames (static 7-frame skeleton). Do NOT validate
    weight positivity, score range, or non-empty check in `lay_out()` — all range enforcement is
    in `ValueRangeValidator` at Stage 5. Rustdoc all public items.
  - [ ] T4.4: Export all three from `slide_types/mod.rs`.
  - [ ] T4.5: Register all three in `registry.rs::Default::default()`:
    `r.register(Box::new(StatusSlideType::new()))`,
    `r.register(Box::new(ProgressBarSlideType::new()))`,
    `r.register(Box::new(WeightedCompositeSlideType::new()))`.
  - [ ] T4.6: Create `value_range.rs` in `slideforge-validate` — implement `ValueRangeValidator`
    (F-087-P1-001): Validator trait impl, validator_id = "value-range".
    For `progress_bar`: read `Slide.fields["value"]`; emit E-VAL-011 if absent, wrong type, or outside [0,100].
    For `weighted_composite`: read `Slide.fields["components"]`; emit E-VAL-011 if list absent or empty;
    for each component, check weight>0 and score∈[0,100]; accumulate ALL errors (DI-018).
    Wire `pub mod value_range;` and re-export `ValueRangeValidator` in `slideforge-validate/src/lib.rs`.
    Register in `slideforge::registry.rs` with `builder.register_validator(Box::new(ValueRangeValidator));`
    after `LabelCheckValidator` registration. Rustdoc all public items.

- [ ] **T5 — LabelCheck extension for weighted_composite component iteration**
  - [ ] T5.1: In `label_check.rs`, extend `validate()` to handle `slide_type == "weighted_composite"`: iterate `Slide.fields["components"]` (a resolved `Value::List` of `Value::Map` entries) and check each component's `"label"` sub-field. Emit E-A11-002 per component with missing label.
  - [ ] T5.2: The top-level `label` check for `weighted_composite` follows the same pattern as `status` and `progress_bar` (already handled by the generic COLOR_CODED_TYPES loop if component iteration is separate).
  - [ ] T5.3: Add consistency assertion unit test (AC-024): assert COLOR_CODED_TYPES and SLIDE_TYPE_KEYWORDS both contain all three types.
  - [ ] T5.4: Run AC-018, AC-022 — must now pass.

- [ ] **T6 — Green pass: all ACs passing**
  - [ ] T6.1: Run `cargo nextest run -p slideforge-plugin-api --no-fail-fast`.
  - [ ] T6.2: Run `cargo nextest run -p slideforge-validate --no-fail-fast`.
  - [ ] T6.3: Run `cargo nextest run -p slideforge --no-fail-fast` (E2E tests with full pipeline).
  - [ ] T6.4: Run `just check` (full workspace gate: fmt + clippy pedantic + nextest + doctests).

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-001 | BC-1.17.001 EC-001 | `slide status:` with title but no label | E-A11-002 at Stage 5 (LabelCheck, Slide.fields) |
| EC-002 | BC-1.17.001 EC-003 | `slide status:` with empty title | Required-field error for title before label check |
| EC-003 | BC-1.17.001 EC-007 | 3 status slides in @for loop, 1 missing label | All three validated; only the one missing label emits E-A11-002; error accumulation (DI-018) |
| EC-004 | BC-1.17.002 EC-009 | `progress_bar value {{ my_var }}` where my_var evaluates to 75 | Valid; eval resolves to Int(75); passes range check |
| EC-005 | BC-1.17.002 EC-010 | `progress_bar value {{ my_var }}` where my_var evaluates to 150 | Compile error: value 150 out of range [0,100]; error cites source span of expression |
| EC-006 | BC-1.17.002 EC-011 | progress_bar: label present but title absent | Required-field error for title emitted first; E-A11-002 NOT emitted (label is present) |
| EC-007 | BC-1.17.003 EC-003 | weighted_composite: top-level label absent + 2 components missing labels | 3 E-A11-002 diagnostics accumulated: 1 top-level + 2 per-component |
| EC-008 | BC-1.17.003 EC-004 | `components: []` empty list | Compile error: empty components list; distinct from label check |
| EC-009 | BC-1.17.003 EC-008 | Component `label ""` (empty label) | E-A11-002 for that component; empty label is not valid |
| EC-010 | BC-1.17.003 EC-011 | Components provided via @data from JSON | Valid if eval resolves each component's `label` sub-field to a non-empty string |
| EC-011 | wave4-assessment §4.2 | `severity_cards` was already functional in LabelCheck | Regression test: `severity_cards` must still fire E-A11-002 for missing label after this story; no regression from COLOR_CODED_TYPES changes |
| EC-012 | F-G3-HIGH-003 | Any of the 3 types built without keyword registration | After T2, this path is closed; test verifies no UnknownSlideType for the three types |

## Dependency Graph

```
STORY-003 (31 SlideType implementations — provides the implementation pattern)
STORY-017 (LabelCheckValidator — provides E-A11-002 enforcement infrastructure)
STORY-086 (Stage 2b field-to-block threading — required for content-correct output)
  └─ [STORY-087 — THIS STORY — status + progress_bar + weighted_composite] (Wave 5)
       └─ (no downstream story dependencies in v1.0)
```
