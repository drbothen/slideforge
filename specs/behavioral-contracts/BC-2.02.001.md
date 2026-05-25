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
capability: CAP-019
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

# BC-2.02.001: brand_overlay: Overrides Logo/Footer/Confidentiality on Individual Slide

## Description

A `brand_overlay:` block on an individual slide or section overrides specific brand
chrome elements (logo, footer_text, confidentiality_label) for that slide without
switching the slide master. The override affects only the placeholder content values
injected into the slide's XML, not the layout structure or theme. This is the per-slide
customization mechanism for multi-client or multi-audience decks.

## Preconditions

1. The slide or section containing `brand_overlay:` has a valid `slide` type declaration.
2. The `brand_overlay:` block declares at least one of: `logo`, `footer_text`,
   or `confidentiality`.
3. A brand is loaded or synthesized (the overlay amends, not replaces, the brand).

## Postconditions

1. In the PPTX output, the affected slide's logo placeholder contains the override logo
   image bytes (if `logo:` declared in `brand_overlay:`).
2. The affected slide's footer placeholder text is replaced by the `footer_text:` value
   (if declared).
3. A confidentiality label shape is added or updated on the affected slide (if
   `confidentiality:` declared).
4. All other slides in the deck use the base brand's logo, footer, and confidentiality
   settings.
5. The slide layout and master assignment are unchanged for the overlay slide. (DI-016)

## Invariants

1. `brand_overlay:` NEVER changes the slide master assignment. Single-master
   architecture is inviolable in v1.0. (DI-016)
2. Overlay is applied at export time; the `Deck` IR retains the overlay as metadata
   on the slide node.
3. An overlay with an empty `footer_text: ""` clears the footer placeholder content
   (not the placeholder structure).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | brand_overlay: logo path does not exist | E-BRD-001 (broken, exit 4): brand overlay logo file not found at '<path>' |
| EC-002 | brand_overlay: applied to a `slide end:` type | Overlay applied; end slides use CL-11 (SF End Slide) but still accept logo/footer overrides |
| EC-003 | brand_overlay: in a @for loop — different logo per iteration | Overlay per slide is applied individually; each loop iteration gets its own overlay values |
| EC-004 | brand_overlay: declared with no fields (empty block) | Parse warning: empty brand_overlay block has no effect; build continues |
| EC-005 | Multiple brand_overlay: blocks on same slide | E-PAR-002 (syntax error): duplicate brand_overlay block on same slide |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Slide with `brand_overlay: footer_text "CONFIDENTIAL"` | That slide's footer placeholder text = "CONFIDENTIAL"; all others use brand footer | happy-path |
| Slide with `brand_overlay: logo "client-logo.png"` | That slide's logo placeholder contains client-logo.png bytes; master logo unchanged | happy-path |
| brand_overlay: logo with nonexistent path | E-BRD-001; exit 4 | error |
| brand_overlay: with no fields | Parse warning; build continues; no overlay effect | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Overlay applies only to targeted slide; adjacent slides unaffected | integration test (multi-slide deck; inspect per-slide XML) |
| VP-TBD | Slide master rId is identical on all slides regardless of brand_overlay | unit test (check rId on overlay slide vs. non-overlay slide) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-019 ("Per-Slide Brand Overlay") per capabilities.md §CAP-019 |
| Capability Anchor Justification | CAP-019 ("Per-Slide Brand Overlay") per capabilities.md §CAP-019 — overriding logo/footer/confidentiality on individual slides without switching masters is the exact definition of CAP-019 |
| L2 Domain Invariants | DI-016 (single master architecture in v1.0) |
| Architecture Module | slideforge-pptx crate — per-slide overlay application (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-2.02.002 — composes with (this BC specifies what can be overridden; BC-2.02.002 specifies what cannot)
- BC-4.01.001 — depends on (PPTX serializer applies the overlay during export)

## Architecture Anchors

- `architecture/branding-subsystem.md#overlay` — per-slide brand overlay application

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
