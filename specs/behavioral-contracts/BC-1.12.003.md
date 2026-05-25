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
capability: CAP-014
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

# BC-1.12.003: SVG Normalized via usvg Before PPTX Embedding (no foreignObject, absolute dims)

## Description

Before embedding a diagram SVG into any output format, it is passed through the `usvg`
normalization pipeline. usvg resolves `<use>` references, inlines CSS class-based styles
into presentation attributes, replaces unsupported SVG elements, and ensures all dimensions
are absolute pixel values (no percentage `%` units). This normalization step is required
because PPTX embedding is sensitive to SVG constructs that are legal in browsers but
cause rendering failures in Office applications.

## Preconditions

1. A valid SVG string has been produced by mermaid-rs-renderer (no E-EXP-008).
2. The `usvg` crate is compiled into the slideforge binary (no runtime dependency).

## Postconditions

1. The normalized SVG contains no `<foreignObject>` elements.
2. The normalized SVG contains no `<script>` elements.
3. The normalized SVG contains no CSS `@keyframes` or CSS class-based styles (all inlined to presentation attributes).
4. The normalized SVG root element has `width` and `height` attributes as absolute pixel values (no `%`, no `em`, no `rem`).
5. All `<use href="...">` references are resolved and inlined.
6. The normalized SVG is the form used for all format embeddings (PPTX, HTML, PDF).
7. If usvg fails to normalize the SVG (corrupted input), E-EXP-004 is emitted.

## Invariants

1. usvg normalization is applied to EVERY diagram SVG before format embedding — no bypass path.
2. Normalization is applied after mermaid-rs-renderer produces the SVG and before any format-specific exporter receives it.
3. The normalized SVG faithfully represents the same visual diagram as the input SVG.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SVG contains `<use href="#symbol">` cross-references | Resolved and inlined by usvg; no dangling references in output |
| EC-002 | SVG uses `width="100%"` (percentage dimensions) | Normalized to absolute px computed from viewBox + container |
| EC-003 | SVG contains `<foreignObject>` (rare but technically valid Mermaid output) | foreignObject removed or replaced by usvg normalization |
| EC-004 | usvg normalization fails (malformed SVG from renderer) | E-EXP-004 emitted; exit 3; error names diagram slide title |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| flowchart SVG with CSS class styles (`.nodeLabel { color: red }`) | Normalized: style inlined as presentation attribute; no `<style>` element | happy-path |
| SVG with percentage width | Normalized: width/height in absolute px | happy-path |
| Malformed SVG (from mocked renderer) | E-EXP-004; exit 3 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Normalized SVG has no foreignObject, no script, no % dimensions | unit test: parse normalized SVG, check for forbidden elements/attributes |
| VP-TBD | usvg failure produces E-EXP-004 (not a panic) | unit test with malformed SVG input |
| VP-TBD | All <use> references resolved (no dangling href) | unit test: parse normalized SVG, verify no <use> elements remain |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-014 ("Diagram Rendering (Mermaid)") per capabilities.md §CAP-014 |
| Capability Anchor Justification | CAP-014 ("Diagram Rendering (Mermaid)") per capabilities.md §CAP-014 — "The output SVG is PPTX-safe by default: no foreignObject, no script, no percentage dimensions" is the normalization requirement |
| L2 Domain Invariants | DI-013 (PPTX output must pass multi-renderer fidelity check) |
| Architecture Module | slideforge-diagrams crate — usvg normalization step (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.12.001 — composes with (normalization is a step in BC-1.12.001's rendering pipeline)
- BC-4.01.001 — depends on (PPTX exporter receives normalized SVG from this BC)

## Architecture Anchors

- `architecture/system-overview.md#diagram-renderer` — usvg normalization pipeline
- `architecture/system-overview.md#spike-s14` — mermaid-rs-renderer + usvg integration

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
