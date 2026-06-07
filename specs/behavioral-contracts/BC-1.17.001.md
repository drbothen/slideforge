---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified: ["2026-06-06 v1.2 (architect adjudication F-087-P2-002, pass-2 adjudication 2026-06-06): PC-8 added — label threading mechanism via Stage 2b (TextTag::ColorLabel → RegionRole::Body → visible ContentBlock)."]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-1.17.001: status Slide Type Requires title + label; label Is Mandatory WCAG Color Co-Encoding

## Description

The `status` slide type displays a single-item status indicator where a color conveys
state (e.g., red = at risk, green = on track, amber = attention). Because WCAG AA
prohibits color as the sole conveyor of meaning, `label "..."` is mandatory on every
`status` slide and is the co-encoding mechanism that makes the color-coded status
accessible. The `LabelCheck` validator enforces this at compile time: a `status` slide
missing `label` is a fatal error (E-A11-002) in strict mode.

## Preconditions

1. A `slide status:` block appears in the .sf source.
2. The slide has been parsed into a `SlideNode` and evaluated by `eval_deck`.
3. The slide type keyword `"status"` is registered in `SLIDE_TYPE_KEYWORDS` and in
   the `regions.rs` region map.
4. `build_inner` is running with `BuildOptions::default()` (`strict: true`).

## Postconditions

1. `fields["title"]` is required. If absent or empty-after-trim, a compile error is
   produced (E-SLT-001 or equivalent type-required-field error) before the threading pass.
2. `fields["label"]` is required. If absent or empty-after-trim, the `LabelCheck`
   validator emits `E-A11-002` with the message format:
   `Missing label on color-coded element 'status' '<title-value>' at <file>:<line>:<col>. Color alone must not convey meaning. Add label "...".`
   In strict mode: `Err(BuildError::ValidationFailed)` is returned; no output is produced.
3. When both `title` and `label` are present:
   - The slide is rendered with the status color indicator AND the textual label visible
     in the same frame.
   - PPTX: the label text is present in an OOXML element accessible to screen readers.
   - PDF: the label text is included in the PDF structure tree.
   - HTML: the label text is present in the DOM (not hidden via CSS).
4. The `LabelCheck.validate()` pre-layout pass (Stage 5) operates on `Slide.fields`
   directly (not on `Slide.blocks`), so it is functional for `status` slides regardless
   of whether Stage 2b has run.
5. The slide type is registered in `COLOR_CODED_TYPES` within `LabelCheckValidator`
   and in `SLIDE_TYPE_KEYWORDS` in `keywords.rs` and the region map in `regions.rs`.
8. The `label` field is threaded into `Slide.blocks` as
   `ContentBlock::Text(TextTag::ColorLabel)` by `thread_fields_to_blocks` (Stage 2b,
   ADR-019 Decision 3). At layout time, `fill_region_slot_or_append` routes
   `TextTag::ColorLabel → RegionRole::Body`, placing the label text into the Body-role
   frame as `FrameContent::Body(...)`. Exporters render this frame as visible text
   accessible to screen readers. (Mechanism: architect adjudication F-087-P2-002.)

## Invariants

1. **`label` is mandatory, not optional.** There is no `decorative: true` opt-out for
   the label requirement on color-coded slide types. Color co-encoding cannot be
   "opted out of" — the slide type's semantic purpose depends on the label being
   present. (DI-002)
2. **`title` is mandatory.** A `status` slide without a title is a compile error. The
   title identifies what is being statused.
3. **LabelCheck operates on `Slide.fields`, not `Slide.blocks`.** Unlike AltTextValidator
   (which requires post-layout `LaidOutDeck` frames), LabelCheck reads `Slide.fields["label"]`
   directly at Stage 5 (pre-layout). This is correct and intentional — `label` is a
   semantic field that resolves during eval, not a content block.
4. **E-A11-002 is the error code for missing label.** The error is `broken`, exit 2 in
   strict mode, warning in `--warn-only` mode. The error message format is defined in
   the error taxonomy.
5. **The slide type keyword `"status"` must be registered before the pipeline runs.**
   If `"status"` is not in `SLIDE_TYPE_KEYWORDS`, the pipeline emits
   `LayoutError::UnknownSlideType` before the label validator runs. Both registration
   AND label enforcement are required for the type to be functional.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `slide status:` with `title "At Risk"` but no `label` field | `E-A11-002: Missing label on color-coded element 'status' 'At Risk' at <file>:<line>:<col>.` Strict mode: `Err(BuildError::ValidationFailed)`. No output. |
| EC-002 | `slide status:` with `title "On Track"` and `label "On Track"` | Valid. Slide renders with color indicator and visible label text. |
| EC-003 | `slide status:` with `title ""` (empty title) | Compile error: title is required and must be non-empty. E-SLT-001 or equivalent required-field error before label check runs. |
| EC-004 | `slide status:` with `label ""` (empty label) | E-A11-002. Empty label is equivalent to absent label — does not satisfy WCAG co-encoding. |
| EC-005 | `slide status:` with `label "  "` (whitespace-only label) | E-A11-002. Whitespace-only label treated as absent (no meaningful text content). |
| EC-006 | `slide status:` built with `strict: false` and no `label` | Warning emitted via `tracing::warn!`, E-A11-002 as warning. Output produced with color indicator but missing textual label (non-conforming WCAG AA output flagged in warn-only). |
| EC-007 | `slide status:` in a `@for` loop producing 3 slides, 1 missing label | All three slides are validated; only the one missing label emits E-A11-002. Error accumulation continues (DI-018). |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slide status: title "Project Alpha" label "On Track"` | `Ok(BuildOutput)` — slide renders with color indicator and label "On Track" | happy-path |
| `slide status: title "Project Beta"` (no label) | `Err(BuildError::ValidationFailed)` containing `Diagnostic { code: "E-A11-002", message: contains "status", "Project Beta" }` | error (missing label) |
| `slide status: title "Project Gamma" label ""` (empty label) | `Err(BuildError::ValidationFailed)` containing `Diagnostic { code: "E-A11-002" }` | error (empty label) |
| `slide status: title "Project Delta" label "At Risk"` built with `strict: false` | `Ok(BuildOutput)` — warning emitted; output produced | warn-only |
| `slide status:` (no title, no label) | Compile error on missing title; both required-field and E-A11-002 diagnostics accumulated | multi-error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `status` slide with absent `label` always produces E-A11-002 in strict mode | unit test |
| VP-TBD | `status` slide with non-empty `label` and `title` produces `Ok(BuildOutput)` | unit test |
| VP-TBD | LabelCheck.validate() on `status` slide reads `Slide.fields["label"]` (not `Slide.blocks`) | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 |
| Capability Anchor Justification | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 — "`status` is one of the 31 built-in slide types that each enforce their own required fields and layout rules via the SlideType plugin trait," per the exact definition in CAP-010. The `label` requirement is the `status` type's required-field enforcement. |
| L2 Domain Invariants | DI-002 (label required on color-coded elements — `status` is a color-coded type) |
| Architecture Module | slideforge-plugin-api crate (SS-14) — `status` SlideType implementation; slideforge-validate crate (SS-03) — LabelCheckValidator |
| Stories | (filled by story-writer — STORY-B: `status-progress-bar-weighted-composite-slide-types`) |

## Related BCs

- BC-1.17.002 — sibling: `progress_bar` slide type (same label-mandatory pattern, different fields)
- BC-1.17.003 — sibling: `weighted_composite` slide type (same label pattern, component-level labels also mandatory)
- BC-5.01.003 — depends on (missing-label compile error, which this BC instantiates for the `status` type)
- BC-3.01.001 — depends on (SlideType trait enforces required fields; `status` implements SlideType)

## Architecture Anchors

- `specs/architecture/ARCH-INDEX.md` SS-14 (Plugin API — SlideType implementations)
- `specs/architecture/ARCH-INDEX.md` SS-03 (Validation — LabelCheckValidator)
- `specs/architecture/adr/ADR-019-stage-2b-field-to-block-threading.md` — LabelCheck operates pre-layout on `Slide.fields` (Section 4.1 / F-G3-HIGH-003 finding)

## Story Anchor

(filled by story-writer — STORY-B)

## VP Anchors

(filled after VP creation)

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| 1.0 | 2026-06-05 | Initial creation — status slide type label-mandatory contract |
| 1.2 | 2026-06-06 | PC-8 added per architect adjudication F-087-P2-002 (pass-2, 2026-06-06): label field threaded by Stage 2b as TextTag::ColorLabel → RegionRole::Body → visible FrameContent::Body. Rendering mechanism decided (Option T — Threading). |
