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

# BC-2.02.002: brand_overlay: Does Not Switch Slide Master (Single-Master Architecture)

## Description

In v1.0, all slides in a deck share exactly one slide master. The `brand_overlay:` block
modifies placeholder content (logo, footer text, confidentiality label) but cannot
introduce a new slide master or change the layout's master association. Any attempt to
use `brand_overlay:` to switch themes or masters is a compile error.

## Preconditions

1. A deck is being built with one or more `brand_overlay:` blocks.
2. The deck has exactly one brand (from `brand:` in deck metadata).

## Postconditions

1. The serialized .pptx has exactly one `<p:sldMaster>` element in the package.
2. All slide layout `_rels` files back-reference the same `slideMaster1.xml`.
3. Every slide's `<p:sld>` references a layout, and that layout references the single master.
4. The `brand_overlay:` fields (logo, footer_text, confidentiality) are stored as
   `<p:sp>` shape overrides at the slide level — NOT as layout or master modifications.

## Invariants

1. The single-master invariant is absolute in v1.0. (DI-016)
2. No API, flag, or DSL construct in v1.0 allows adding a second slide master.
3. The PPTX relationship graph is always: slide → layout → master (one chain per slide,
   all chains converge to the same master).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | brand_overlay: attempts to set a different brand template path | Parse error: "brand_overlay does not accept a template path. Use brand: at deck level for the master brand. Per-slide overlay supports: logo, footer_text, confidentiality." |
| EC-002 | brand_overlay: on every slide in the deck | Supported; single master still present; all overlays applied as slide-level shape overrides |
| EC-003 | Two brand: declarations in the same .sf file | E-PAR-002 (syntax error): duplicate brand declaration; only one brand allowed per deck |
| EC-004 | @include includes a .sf fragment that also has brand: | The included fragment's brand: is merged (last-wins per DI-020); single master maintained |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Deck with 10 slides, 3 have brand_overlay: | .pptx has 1 slide master; all 10 slides reference the same master via layout chain | happy-path |
| brand_overlay: on every slide | .pptx has 1 slide master; per-slide shape overrides on all slides; exit 0 | happy-path |
| brand_overlay: with template: "other.pptx" (hypothetical invalid syntax) | E-PAR: brand_overlay does not accept template path | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Serialized .pptx always has exactly one sldMaster element | integration test (count sldMaster elements in package) |
| VP-TBD | All slide rId→layout→master chains terminate at the same master | integration test (traverse relationship graph) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-019 ("Per-Slide Brand Overlay") per capabilities.md §CAP-019 |
| Capability Anchor Justification | CAP-019 ("Per-Slide Brand Overlay") per capabilities.md §CAP-019 — the constraint that brand_overlay does not switch masters ("without switching masters (single-master architecture in v1.0)") is the defining qualifier of CAP-019 |
| L2 Domain Invariants | DI-016 (single master architecture in v1.0) |
| Architecture Module | slideforge-pptx crate — PPTX relationship serializer (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-2.02.001 — composes with (BC-2.02.001 specifies what is overrideable; this BC specifies what is not)
- BC-4.01.001 — depends on (PPTX serializer enforces single-master chain)

## Architecture Anchors

- `architecture/brand-architecture.md#single-master` — DI-016 single master invariant
- `architecture/system-overview.md#pptx-relationships` — layout-to-master relationship graph

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
