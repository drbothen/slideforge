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

# BC-5.01.002: decorative: true Opts Out of Alt Requirement; Emits Empty Alt in All Formats

## Description

An image, shape, or visual element marked `decorative: true` in the .sf source is
exempt from the alt text requirement of BC-5.01.001. In all output formats, decorative
elements produce the appropriate empty/artifact markers: `descr=""` in PPTX, PDF Artifact
tag (excluded from structure tree), `alt="" role="presentation"` in HTML. This covers
DEC-008 (all elements on a slide may be legitimately decorative).

## Preconditions

1. A visual element is declared with `decorative: true` in the .sf source.
2. The element does NOT have an `alt "..."` field (if both are present, decorative wins).
3. The build is in any mode (strict or warn-only).

## Postconditions

1. No E-A11-001 is emitted for this element.
2. In PPTX output: the element's `<p:cNvPr>` has `descr=""`.
3. In PDF output: the element is marked as a PDF Artifact and excluded from the
   `/StructTree`.
4. In HTML output: the element has `alt=""` and `role="presentation"`.
5. The build succeeds (decorative: true is valid in all build modes).

## Invariants

1. `decorative: true` is the ONLY mechanism to opt out of alt text. Silence (omitting
   the alt field) is NOT treated as decorative — it produces E-A11-001. (DI-001)
2. The decorative marker propagates to ALL output formats from the same IR record.
3. If both `alt "..."` and `decorative: true` are specified, the element is treated as
   decorative and a lint warning W-A11-001 is emitted ("alt text ignored for decorative
   element").

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All visual elements on a slide are `decorative: true` (DEC-008) | No E-A11-001; all elements produce empty alt in all formats; slide is valid |
| EC-002 | `decorative: true` with `alt "Some text"` also set | Decorative wins; W-A11-001 lint warning; alt text NOT embedded; element treated as artifact |
| EC-003 | Decorative shape in a chart slide | Chart group shape treated as Artifact in PDF; `descr=""` in PPTX |
| EC-004 | decorative: true in --warn-only mode | Behaves identically to strict mode — decorative is valid in both modes |
| EC-005 | decorative: false (explicit false) | Treated as missing decorative marker; alt is still required |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `image: "bg.png"` with `decorative: true` | No E-A11-001; PPTX `descr=""`; HTML `alt="" role="presentation"` | happy-path (TV-7.3) |
| Slide with 5 images all `decorative: true` | No errors; all 5 have empty alt in all formats | edge-case (DEC-008) |
| `image: "photo.png"` with `alt "Team"` and `decorative: true` | W-A11-001 lint warning; element treated as decorative; alt "Team" not in output | edge-case |
| `image: "bg.png"` with no alt and no decorative | E-A11-001 (covered by BC-5.01.001) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | decorative elements have empty alt in PPTX, Artifact in PDF, alt="" in HTML | integration test: build fixture; parse all three output formats |
| VP-TBD | No E-A11-001 for decorative elements | unit test: parse .sf with all-decorative slide; assert zero A11 errors |
| VP-TBD | W-A11-001 emitted when both alt and decorative: true are set | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 ("Accessibility Validation") per capabilities.md §CAP-020 |
| Capability Anchor Justification | CAP-020 ("Accessibility Validation") per capabilities.md §CAP-020 — "decorative: true opts out" is part of the alt text enforcement contract in CAP-020 |
| L2 Domain Invariants | DI-001 (alt text required OR decorative: true — this BC specifies the decorative path) |
| Architecture Module | slideforge-validate crate (SS-03) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.01.001 — composes with (this BC is the decorative complement; together they cover DI-001 completely)
- BC-4.01.004 — depends on (PPTX empty alt embedding spec for decorative elements)
- BC-4.03.001 — depends on (PDF Artifact marking for decorative elements)
- BC-4.03.003 — depends on (HTML alt="" + role="presentation" for decorative elements)

## Architecture Anchors

- `architecture/plugin-architecture.md` — decorative element handling

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
