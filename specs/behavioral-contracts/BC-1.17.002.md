---
document_type: behavioral-contract
level: L3
version: "1.0"
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
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-1.17.002: progress_bar Slide Type Requires title + label + value(0–100); label Co-Encodes Numeric Progress Accessibly

## Description

The `progress_bar` slide type displays a visual progress bar where the filled portion
conveys completion state numerically. Because color/proportion alone cannot convey
meaning to screen readers or color-blind users, `label "..."` is mandatory and must
co-encode the numeric progress in text form (e.g., `label "42% complete"`). The `value`
field (0–100 integer) drives the visual fill proportion. `LabelCheck` enforces the
label requirement at compile time; a missing label is a fatal error (E-A11-002) in
strict mode.

## Preconditions

1. A `slide progress_bar:` block appears in the .sf source.
2. The slide has been parsed and evaluated by `eval_deck`.
3. The slide type keyword `"progress_bar"` is registered in `SLIDE_TYPE_KEYWORDS`,
   the `regions.rs` region map, and `COLOR_CODED_TYPES` in `LabelCheckValidator`.
4. `build_inner` is running with `BuildOptions::default()` (`strict: true`).

## Postconditions

1. `fields["title"]` is required. If absent or empty-after-trim, a compile error
   (required-field error) is produced before the label check.
2. `fields["label"]` is required. If absent or empty-after-trim, `LabelCheck` emits
   `E-A11-002` with message:
   `Missing label on color-coded element 'progress_bar' '<title-value>' at <file>:<line>:<col>. Color alone must not convey meaning. Add label "...".`
   In strict mode: `Err(BuildError::ValidationFailed)`; no output.
3. `fields["value"]` is required and must be an integer in the range [0, 100] (inclusive).
   - If absent: compile error (required-field error).
   - If present but outside [0, 100]: compile error with message:
     `progress_bar value must be between 0 and 100; got <value>.`
4. When `title`, `label`, and `value` are all valid:
   - The slide renders a visual progress bar filled to `value`% of its total width.
   - The `label` text is rendered visibly, providing the accessible text co-encoding
     of the numeric progress.
   - PPTX: label text in accessible OOXML element.
   - PDF: label text in PDF structure tree.
   - HTML: label text in DOM.
5. `LabelCheck.validate()` (Stage 5) reads `Slide.fields["label"]` directly.
   LabelCheck does NOT require Stage 2b to have run.
6. The `value` field validation is performed either by the `SlideType` trait implementation
   (at type-registration time) or by a dedicated pre-layout validator. The exact
   enforcement point is the SlideType trait's `validate_fields(&Slide.fields)` method.

## Invariants

1. **`label` is mandatory — no opt-out.** Unlike visual media (where `decorative: true`
   is an opt-out), color-coded progress bars require a label. (DI-002)
2. **`title` is mandatory.** Identifies what progress is being tracked.
3. **`value` is mandatory and bounded.** Must be in `[0, 100]` inclusive. The integer
   constraint means `value: 50.5` (float) is a type error; only integer-valued expressions
   are accepted.
4. **Label should co-encode the numeric progress.** This is a content recommendation,
   not a compile-time enforced format constraint. Authors may write `label "In progress"` —
   the validator does not require the label to contain a percentage. However, WCAG
   authoring guidance (and slideforge's user docs) strongly recommend including the
   numeric value in the label for full accessibility (e.g., `label "47% complete"`).
5. **E-A11-002 for missing label.** Same error code and severity as BC-1.17.001.
6. **The keyword `"progress_bar"` must be registered before the pipeline runs.**
   An unregistered keyword causes `LayoutError::UnknownSlideType` before label validation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `slide progress_bar:` with `title "Sprint 4"` and `value 65` but no `label` | E-A11-002 emitted. Strict mode: `Err(BuildError::ValidationFailed)`. No output. |
| EC-002 | `slide progress_bar:` with `title "Sprint 4"`, `label "65% complete"`, `value 65` | Valid. Progress bar renders at 65% fill; label "65% complete" visible in output. |
| EC-003 | `fields["value"] = 101` | Compile error: value out of range [0, 100]. |
| EC-004 | `fields["value"] = -1` | Compile error: value out of range [0, 100]. |
| EC-005 | `fields["value"] = 0` | Valid. Progress bar renders at 0% fill (empty bar). |
| EC-006 | `fields["value"] = 100` | Valid. Progress bar renders at 100% fill (full bar). |
| EC-007 | `fields["label"] = ""` (empty string) | E-A11-002. Empty label is equivalent to absent. |
| EC-008 | `slide progress_bar:` built with `strict: false`, no label | Warning emitted. Output produced. Non-conforming WCAG AA. |
| EC-009 | `fields["value"]` is a `{{ expr }}` that evaluates to 75 | Valid. `eval_deck` resolves to `FieldValue::Literal(Value::Int(75))`; passes value check. |
| EC-010 | `fields["value"]` is a `{{ expr }}` that evaluates to 150 | Compile error: value 150 is out of range [0, 100]. Error cites source span of the expression. |
| EC-011 | Label present but title absent | Required-field error for title emitted first; E-A11-002 NOT emitted (label is present). Both errors accumulated if validation continues past title check. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slide progress_bar: title "Completion" label "75% done" value 75` | `Ok(BuildOutput)` — progress bar at 75%; label "75% done" visible | happy-path |
| `slide progress_bar: title "Completion" value 75` (no label) | `Err(BuildError::ValidationFailed)` with `code == "E-A11-002"` | error (missing label) |
| `slide progress_bar: title "Completion" label "Done" value 0` | `Ok(BuildOutput)` — 0% progress bar | boundary (zero value) |
| `slide progress_bar: title "Completion" label "Done" value 100` | `Ok(BuildOutput)` — 100% filled bar | boundary (max value) |
| `slide progress_bar: title "Completion" label "Done" value 101` | Compile error: value out of range | error (over-range) |
| `slide progress_bar: title "Completion" label "Done" value -5` | Compile error: value out of range | error (under-range) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `progress_bar` slide with absent `label` always produces E-A11-002 in strict mode | unit test |
| VP-TBD | `fields["value"]` outside [0, 100] always produces a compile error | unit test |
| VP-TBD | `progress_bar` slide with valid title + label + value in [0,100] produces `Ok(BuildOutput)` | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 |
| Capability Anchor Justification | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 — `progress_bar` is one of the 31 built-in slide types; it enforces its own required fields (`title`, `label`, `value`) and layout rules via the `SlideType` plugin trait, exactly as CAP-010 describes. |
| L2 Domain Invariants | DI-002 (label required on color-coded elements), DI-018 (error accumulation) |
| Architecture Module | slideforge-plugin-api crate (SS-14) — `progress_bar` SlideType implementation; slideforge-validate crate (SS-03) — LabelCheckValidator |
| Stories | (filled by story-writer — STORY-C: `status-progress-bar-weighted-composite-slide-types`) |

## Related BCs

- BC-1.17.001 — sibling: `status` slide type (same label-mandatory pattern; simpler fields)
- BC-1.17.003 — sibling: `weighted_composite` slide type (same label pattern plus per-component labels)
- BC-5.01.003 — depends on (missing-label compile error that this BC instantiates for `progress_bar`)
- BC-3.01.001 — depends on (SlideType trait enforces required fields)

## Architecture Anchors

- `specs/architecture/ARCH-INDEX.md` SS-14 (Plugin API — SlideType implementations)
- `specs/architecture/ARCH-INDEX.md` SS-03 (Validation — LabelCheckValidator)
- `specs/architecture/adr/ADR-019-stage-2b-field-to-block-threading.md` — LabelCheck operating pre-layout

## Story Anchor

(filled by story-writer — STORY-C)

## VP Anchors

(filled after VP creation)
