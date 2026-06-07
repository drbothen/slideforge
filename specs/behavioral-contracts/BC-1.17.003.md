---
document_type: behavioral-contract
level: L3
version: "1.2.1"
status: active
producer: product-owner
timestamp: 2026-06-05T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md, specs/architecture/adr/ADR-019-stage-2b-field-to-block-threading.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-14
capability: CAP-010
lifecycle_status: active
introduced: v1.0.0
modified: ["2026-06-06 v1.1 (architect adjudication F-087-P1-001): PC3b added; Inv 6/7 amended to cite ValueRangeValidator Stage 5 + E-VAL-011 + DI-018 accumulation; Inv-9 added; enforcement point moved from lay_out() to ValueRangeValidator.", "2026-06-06 v1.2 (architect adjudication F-087-P2-002, pass-2 adjudication 2026-06-06): PC-9 added — aggregate label and per-component row text threaded as visible ContentBlocks via compose_component_row_text; empty-components → E-VAL-011 reaffirmed (F-087-P2-001).", "2026-06-06 v1.2.1 (F-087-P3-001 follow-through): PC-6 HTML clause and PC-9 exporters clause scoped with contingency notes — HtmlExporter deferred project-wide to STORY-050/Phase-4; layout IR materialized by STORY-087."]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-1.17.003: weighted_composite Slide Type Requires title + label + components[]; Both Top-Level and Per-Component label Are Mandatory

## Description

The `weighted_composite` slide type displays a composite score built from multiple
weighted sub-components (e.g., a vendor scorecard with dimensions "Quality", "Price",
"Support" each having a weight and score). Color conveys the composite and per-component
score magnitude, making this a color-coded type. WCAG AA compliance requires a top-level
`label "..."` co-encoding the aggregate result in text, and a per-component `label`
co-encoding each component's score. Both the top-level label and every component's label
are mandatory. `LabelCheck` enforces this at compile time.

## Preconditions

1. A `slide weighted_composite:` block appears in the .sf source.
2. The slide has been parsed and evaluated by `eval_deck`.
3. The slide type keyword `"weighted_composite"` is registered in `SLIDE_TYPE_KEYWORDS`,
   the `regions.rs` region map, and `COLOR_CODED_TYPES` in `LabelCheckValidator`.
4. `build_inner` is running with `BuildOptions::default()` (`strict: true`).
5. `fields["components"]` is a list where each element has at minimum `name`, `weight`,
   `score`, and `label` sub-fields.

## Postconditions

1. `fields["title"]` is required. If absent or empty-after-trim, a compile error is
   produced.
2. `fields["label"]` (top-level, the aggregate label) is required. If absent or
   empty-after-trim, `LabelCheck` emits `E-A11-002`:
   `Missing label on color-coded element 'weighted_composite' '<title-value>' at <file>:<line>:<col>. Color alone must not convey meaning. Add label "...".`
   In strict mode: `Err(BuildError::ValidationFailed)`; no output.
3. `fields["components"]` is required. It must be a non-empty list. Each component
   in the list must have:
   - `name` (required, non-empty string): the component's display name.
   - `weight` (required, positive number): the weighting factor (e.g., `0.5` for 50%).
   - `score` (required, number in [0, 100]): the component's raw score.
   - `label` (required, non-empty string): the per-component textual label co-encoding
     the score (e.g., `label "Good (85/100)"`).
3b. Component `weight` and `score` range validation is performed by `ValueRangeValidator`
    at Stage 5 (pre-layout). Error code: E-VAL-011. Severity: `DiagnosticSeverity::Error`.
    `SlideType::lay_out()` is geometry-only and does NOT perform weight/score range validation.
4. A missing per-component `label` on ANY component emits `E-A11-002` for that
   component. The error message identifies the component by its `name` field:
   `Missing label on color-coded element 'weighted_composite.component' '<component-name>' at <file>:<line>:<col>. Add label "..." to this component.`
5. All component validation errors are accumulated before returning (DI-018). A slide
   with 3 components all missing labels produces 3 E-A11-002 diagnostics plus 1 E-A11-002
   for the missing top-level label if also absent — 4 total.
6. When all required fields are valid:
   - The slide renders the composite score visually with per-component rows.
   - The top-level `label` text is rendered visibly (accessible co-encoding of the aggregate).
   - Each component's `label` text is rendered adjacent to that component's score bar.
   - PPTX, PDF: all label texts are in the accessibility tree.
   - HTML: all label texts are in the accessibility tree.
     **Contingency:** HTML rendering of label texts is CONTINGENT on the
     HtmlExporter being registered, which is deferred project-wide (anchor:
     STORY-050 / Phase-4). The layout IR (ColorLabel → RegionRole::Body;
     per-component rows → RegionRole::Generic) is materialized correctly by
     STORY-087 and will render once HTML export exists. PPTX/PDF/DOCX
     rendering is delivered in STORY-087.
7. `LabelCheck.validate()` (Stage 5, pre-layout) reads `Slide.fields["label"]` and
   iterates `Slide.fields["components"]` (the resolved list) to check each component's
   `label` sub-field. This does NOT require Stage 2b.
9. The top-level `label` field is threaded into `Slide.blocks` as
   `ContentBlock::Text(TextTag::ColorLabel)` by `thread_fields_to_blocks` (Stage 2b,
   ADR-019 Decision 3). At layout time, `fill_region_slot_or_append` routes
   `TextTag::ColorLabel → RegionRole::Body`, placing the aggregate label text into the
   Body-role frame as `FrameContent::Body(...)`, rendered visibly by exporters (PPTX/PDF/DOCX
   delivered in STORY-087; HTML CONTINGENT on HtmlExporter — deferred, anchor: STORY-050/Phase-4).
   Each component in `components[]` is threaded as `ContentBlock::Text(TextTag::Body)`
   with composed text `'<name>: <score>/100 (wt: <weight>) — <label>'` (produced by the
   pure helper `compose_component_row_text`). Each composed block claims one of the five
   pre-allocated `RegionRole::Generic` row frames in registration order via
   `fill_region_slot_or_append`'s generic-fallback path. A maximum of 5 component rows
   are rendered; components beyond index 4 produce no additional frames (the 5 Generic
   slots are exhausted). (Mechanism: architect adjudication F-087-P2-002.)
   An empty `components: []` list (postcondition 3 / invariant 4) causes
   `ValueRangeValidator` to emit E-VAL-011 with message
   `"weighted_composite requires at least one component; got empty list."` — confirmed
   per architect adjudication F-087-P2-001. `Err(BuildError::ValidationFailed)` is
   returned; no output is produced.

## Invariants

1. **Top-level `label` is mandatory — no opt-out.** (DI-002)
2. **Every component's `label` is mandatory.** The accessibility obligation applies to
   each individually color-coded component, not just the aggregate. (DI-002)
3. **`title` is mandatory.** Names what is being evaluated.
4. **`components` is mandatory and non-empty.** An empty `components: []` is a compile
   error (a weighted_composite with no components has no semantic content).
5. **Error accumulation is total across all components.** Missing labels on N components
   produce N E-A11-002 diagnostics. The validator does not bail on the first missing
   component label. (DI-018)
6. **`weight` must be positive.** A zero or negative weight is a compile error. Weights
   do not need to sum to 1.0 or 100 — the rendering normalizes them automatically.
   Enforced by `ValueRangeValidator` at Stage 5. Error code: E-VAL-011.
7. **`score` must be in [0, 100] inclusive.** Out-of-range values are a compile error
   with source span. Enforced by `ValueRangeValidator` at Stage 5. Error code: E-VAL-011.
   Error accumulation: all components' errors are collected before returning (DI-018).
8. **LabelCheck is a Stage 5 (pre-layout) check.** It reads `Slide.fields` directly.
   It does not require or interact with Stage 2b.
9. **`ValueRangeValidator` is registered at Stage 5. No weight/score range validation
   occurs in `SlideType::lay_out()`.** The `lay_out()` method in `weighted_composite.rs`
   is geometry-only after architect adjudication F-087-P1-001. Tests calling `lay_out()`
   directly do NOT exercise the value-range enforcement.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Top-level `label` absent, all component labels present | E-A11-002 for the top-level missing label. Strict: `ValidationFailed`. |
| EC-002 | Top-level `label` present, one component missing `label` | E-A11-002 for the specific component missing its label. |
| EC-003 | Top-level `label` absent AND two components missing `label` | Three E-A11-002 diagnostics accumulated: one for top-level + one per component. |
| EC-004 | `components: []` (empty list) | Compile error: `weighted_composite` requires at least one component. Separate from label check. |
| EC-005 | Component with `name "Quality"`, `weight 0.5`, `score 85`, `label "Good"` | Valid component. |
| EC-006 | Component with `name "Price"`, `weight 0` | Compile error: weight must be positive. |
| EC-007 | Component with `score 105` | Compile error: score out of range [0, 100]. |
| EC-008 | Component with `label ""` (empty label) | E-A11-002 for that component. Empty label is not valid. |
| EC-009 | `slide weighted_composite:` built with `strict: false`, top-level label absent | Warning emitted. Output produced but non-conforming. |
| EC-010 | 5 components all missing labels + top-level label missing | 6 E-A11-002 diagnostics (5 component + 1 top-level) all accumulated and reported together in one build. |
| EC-011 | `components` provided via `@data` from JSON: `@data scores from "scores.json"` with computed `label` field | Valid if `eval_deck` resolves each component's `label` sub-field to a non-empty string. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slide weighted_composite: title "Vendor A" label "Overall: Good (78/100)" components: [{ name "Quality" weight 0.4 score 85 label "Excellent" }, { name "Price" weight 0.6 score 72 label "Acceptable" }]` | `Ok(BuildOutput)` — composite rendered with aggregate label and per-component labels visible | happy-path |
| Same as above but `label` (top-level) absent | `Err(BuildError::ValidationFailed)` with `code == "E-A11-002"` for top-level | error (missing top-level label) |
| Same structure but component 2 has no `label` | `Err(BuildError::ValidationFailed)` with one `E-A11-002` for "Price" component | error (missing component label) |
| 2 components both missing `label`, top-level also absent | `Err(BuildError::ValidationFailed)` with 3 `E-A11-002` diagnostics accumulated | multi-error |
| `components: []` | Compile error: empty components list | error (empty components) |
| Component with `score 101` | Compile error: score out of range [0, 100] | error (out-of-range score) |
| Component with `weight -0.1` | Compile error: weight must be positive | error (non-positive weight) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Missing top-level `label` always produces E-A11-002 | unit test |
| VP-TBD | Missing per-component `label` produces E-A11-002 for that component | unit test |
| VP-TBD | N components missing labels → N E-A11-002 diagnostics (accumulation) | unit test |
| VP-TBD | `score` outside [0, 100] always produces compile error | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 |
| Capability Anchor Justification | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 — `weighted_composite` is one of the 31 built-in slide types that "each enforce their own required fields and layout rules via the `SlideType` plugin trait." The mandatory top-level and per-component `label` fields are this type's required-field enforcement rule. |
| L2 Domain Invariants | DI-002 (label required on color-coded elements — both top-level and per-component), DI-018 (error accumulation — all component label errors accumulated before return) |
| Architecture Module | slideforge-plugin-api crate (SS-14) — `weighted_composite` SlideType implementation; slideforge-validate crate (SS-03) — LabelCheckValidator (extended to traverse component sub-fields) |
| Stories | (filled by story-writer — STORY-D: `status-progress-bar-weighted-composite-slide-types`) |

## Related BCs

- BC-1.17.001 — sibling: `status` slide type (same label-mandatory pattern; simplest form)
- BC-1.17.002 — sibling: `progress_bar` slide type (top-level label only; no component iteration)
- BC-5.01.003 — depends on (missing-label compile error that this BC extends to per-component scope)
- BC-3.01.001 — depends on (SlideType trait enforces required fields at type level)

## Architecture Anchors

- `specs/architecture/ARCH-INDEX.md` SS-14 (Plugin API — SlideType implementations)
- `specs/architecture/ARCH-INDEX.md` SS-03 (Validation — LabelCheckValidator, component-iteration extension)
- `specs/architecture/adr/ADR-019-stage-2b-field-to-block-threading.md` — LabelCheck pre-layout on Slide.fields; weighted_composite analysis in wave4-content-threading-assessment.md §4.2

## Story Anchor

(filled by story-writer — STORY-D)

## VP Anchors

(filled after VP creation)

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| 1.0 | 2026-06-05 | Initial creation — weighted_composite slide type label + component validation contract |
| 1.1 | 2026-06-06 | PC-3b / Inv-6/7/9 added per architect adjudication F-087-P1-001: ValueRangeValidator Stage 5 enforcement; E-VAL-011 allocated; lay_out() is geometry-only |
| 1.2 | 2026-06-06 | PC-9 added per architect adjudication F-087-P2-002 (pass-2, 2026-06-06): aggregate label threaded as TextTag::ColorLabel → Body-role frame; per-component rows threaded as TextTag::Body via compose_component_row_text → Generic-role row frames. Empty-components → E-VAL-011 ("weighted_composite requires at least one component; got empty list.") reaffirmed per F-087-P2-001. Rendering mechanism decided (Option T — Threading). |
| 1.2.1 | 2026-06-06 | PC-6 HTML clause split and scoped; PC-9 exporters clause annotated (F-087-P3-001 follow-through): HTML rendering of aggregate label and per-component row labels is CONTINGENT on HtmlExporter registration, deferred project-wide (anchor: STORY-050 / Phase-4). Layout IR (ColorLabel→Body, Body→Generic) materialized by STORY-087. PPTX/PDF/DOCX unaffected. |
