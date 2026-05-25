---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-020
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

# BC-5.01.001: Missing alt on Visual Element Is Compile Error with Element Location

## Description

Every image, chart, and diagram declared in a .sf file must have a non-empty `alt "..."`
field, OR must be explicitly marked `decorative: true`. If either condition is absent,
the build produces E-A11-001 with the element type, identifier, and source location.
This is a compile-time contract — accessibility is never deferred to runtime or
post-processing. The error is fatal in strict mode.

## Preconditions

1. A visual element (image, chart slide, diagram slide, or shape with visual content) is declared in the source.
2. The element does NOT have a non-empty `alt "..."` field.
3. The element does NOT have `decorative: true`.

## Postconditions

1. E-A11-001 is emitted: `Missing alt text on <element-type> '<identifier>' at <file>:<line>:<col>. Add alt "..." or mark decorative: true`.
2. Build exits with code 2 (validation error) in strict mode.
3. No output is produced in strict mode.
4. In `--warn-only` mode: warning emitted, build continues, output produced.

## Invariants

1. Every visual element in the output has EITHER non-empty alt text OR is marked as a PDF Artifact / empty-alt in all output formats. (DI-001)
2. The alt text requirement is checked BEFORE layout and export — it is part of the validation stage.
3. `decorative: true` produces empty alt attributes in all output formats (empty string in PPTX `<p:ph altText="">`, empty `/Alt` in PDF Artifact, `alt=""` in HTML).
4. No output format can contain a visual element with neither alt text nor decorative marker.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `alt ""` (empty string) | Treated as missing alt text. E-A11-001 emitted. An empty string is not a valid alt text. |
| EC-002 | `alt " "` (whitespace only) | Treated as missing alt text. Whitespace-only alt text is rejected. |
| EC-003 | All images on a slide have `decorative: true` (DEC-008) | No E-A11-001. All images produce empty alt in output. No accessibility error. |
| EC-004 | Chart slide with no `alt` field | E-A11-001 for the chart slide. Chart alt text should describe the data shown. |
| EC-005 | Diagram slide with no `alt` field | E-A11-001 for the diagram slide. |
| EC-006 | Image inside a shape: block with no alt | E-A11-001 for the shape's visual content. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `image: "photo.png"` with no alt | E-A11-001 at line:col of image declaration | error (TV-7.1) |
| `image: "photo.png"` with `alt "Team photo from Q3 offsite"` | No error; alt text preserved in all outputs | happy-path |
| `image: "background.png"` with `decorative: true` | No error; empty alt in all outputs | edge-case (TV-7.3) |
| `image: "photo.png"` with `alt ""` | E-A11-001 (empty string is not valid alt) | error |
| `slide diagram:` with no `alt` field | E-A11-001 on the diagram element | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Every image in output has non-empty alt XOR is Artifact/empty | integration test: parse output PPTX/PDF/HTML, verify alt presence |
| VP-TBD | Error count = number of visual elements missing alt (or decorative) | unit test with N-image fixture |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 ("Accessibility Validation") per capabilities.md §CAP-020 |
| Capability Anchor Justification | CAP-020 ("Accessibility Validation") per capabilities.md §CAP-020 — "alt '...' on all visual elements (images, charts, diagrams) ... compile error if absent" is the exact language of CAP-020 |
| L2 Domain Invariants | DI-001 (alt text required on all visual elements) |
| Architecture Module | slideforge-eval or slideforge-validate crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.01.002 — composes with (decorative: true opt-out is the complement of this BC)
- BC-5.01.003 — related to (same pattern for color-coded elements)
- BC-3.03.002 — depends on (this validation error triggers the no-output gate)
- BC-4.03.001 — depends on (PDF/UA-1 requires alt text; this BC ensures alt is present at PDF export time)

## Architecture Anchors

- `architecture/cross-cutting.md#accessibility-validation` — compile-time accessibility enforcement

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
