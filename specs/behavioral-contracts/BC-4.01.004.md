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
capability: CAP-015
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

# BC-4.01.004: PPTX Contains Accessibility Metadata (Alt Text, Lang, and WCAG Contrast)

## Description

The PPTX exporter must embed alt text for all non-decorative visual elements, the deck
language in Core Properties, and WCAG-compliant contrast metadata so that assistive
technology (e.g., Microsoft Accessibility Checker, NVDA) can consume the file correctly.
Decorative elements produce empty `descr=""` attributes on `<p:cNvPr>`. This contract
complements BC-5.01.001 (compile-time enforcement) by specifying the PPTX embedding
format.

## Preconditions

1. A valid `LaidOutDeck` IR exists with alt text populated on all non-decorative visual
   elements (enforced upstream by BC-5.01.001).
2. A deck language declaration is available (enforced upstream by BC-5.01.004).
3. All color values in the brand pass WCAG AA contrast ratio (4.5:1 for body text,
   3:1 for large text).

## Postconditions

1. Every non-decorative image, chart, or diagram shape in the PPTX has a non-empty
   `descr` attribute on `<p:cNvPr>` containing the DSL `alt "..."` value.
2. Every decorative element has `descr=""` on `<p:cNvPr>`.
3. The PPTX `docProps/core.xml` contains a `<dc:language>` element set to the deck's
   `lang` value.
4. The PPTX passes Microsoft Accessibility Checker "Missing Alt Text" check (zero violations
   for explicitly-set alt text).

## Invariants

1. Alt text values are never truncated. The full `alt "..."` string from the DSL is used.
2. The `<p:cNvPr>` `descr` attribute is the correct OOXML alt text location for shapes —
   NOT `<p:ph altText>` (which is for placeholder names only).
3. Language propagates to ALL output formats from the same deck source. (DI-003)
4. No visual element reaches the PPTX exporter without an alt text decision (either
   non-empty alt or decorative=true), guaranteed by BC-5.01.001.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Alt text contains quotes or special characters | Alt text value XML-escaped correctly in `descr` attribute |
| EC-002 | Alt text longer than 255 characters | Embedded in full; PPTX schema allows long descr strings |
| EC-003 | Deck lang "zh-TW" (non-ASCII BCP-47) | dc:language set to "zh-TW"; no truncation |
| EC-004 | All images on slide are decorative | All shapes have `descr=""`; Microsoft Accessibility Checker: no violations |
| EC-005 | Chart shape (SVG) with alt text | Alt embedded on the group shape containing the SVG picture |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Image with `alt "Revenue chart Q1 2026"` | PPTX `<p:cNvPr descr="Revenue chart Q1 2026">` | happy-path |
| Image with `decorative: true` | PPTX `<p:cNvPr descr="">` | edge-case |
| Deck `lang "fr-FR"` | `docProps/core.xml` has `<dc:language>fr-FR</dc:language>` | happy-path |
| Alt text with `&` character | PPTX XML: `descr="Revenue &amp; Cost"` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All non-decorative picture shapes have non-empty descr | integration test: unzip PPTX, parse slide XML, check cNvPr descr |
| VP-TBD | dc:language matches deck lang declaration exactly | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 |
| Capability Anchor Justification | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 — "accessibility metadata" is an explicit component of PPTX export in CAP-015 |
| L2 Domain Invariants | DI-001 (alt text on all visual elements), DI-003 (language propagates to all formats) |
| Architecture Module | slideforge-pptx crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-4.01.001 — composes with (accessibility metadata is part of the PPTX produced by BC-4.01.001)
- BC-5.01.001 — depends on (alt text presence is enforced compile-time; this BC specifies the embedding format)
- BC-5.01.004 — depends on (lang declaration is enforced compile-time; this BC embeds it)
- BC-4.03.001 — related to (PDF has analogous accessibility metadata in /Alt and /Lang tags)

## Architecture Anchors

- `architecture/export-subsystem.md#pptx-accessibility` — PPTX accessibility metadata embedding
- `architecture/cross-cutting.md#accessibility-validation` — compile-time enforcement (upstream)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
