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

# BC-1.12.001: slide diagram: Renders Mermaid Source to PPTX-Safe SVG via mermaid-rs-renderer

## Description

The `slide diagram:` slide type accepts a Mermaid diagram source block and renders it
to SVG using the `mermaid-rs-renderer` pure Rust crate (no Node.js, no Chrome). The
output SVG is PPTX-safe by default: no `<foreignObject>`, no `<script>`, no percentage
dimensions. The SVG is then normalized via usvg before embedding in any output format.
This supports all 23 Mermaid diagram types covered by mermaid-rs-renderer v0.2.2.

## Preconditions

1. A `slide diagram:` block exists in the source with a valid `source:` field containing Mermaid markup.
2. The `alt "..."` field is present (required by BC-5.01.001).
3. The `lang:` field is set to "mermaid" (the only supported language in v1.0).
4. The mermaid-rs-renderer crate is compiled into the slideforge binary (no runtime dependency).

## Postconditions

1. A valid SVG string is produced by mermaid-rs-renderer.
2. The SVG contains no `<foreignObject>` elements.
3. The SVG contains no `<script>` elements.
4. The SVG contains no CSS `@keyframes` animations.
5. The SVG root element has `viewBox`, `width`, and `height` attributes with absolute (px) values.
6. The SVG is normalized via usvg: CSS class-based styles are inlined, `<use>` references resolved, all dimensions absolute.
7. `<svg aria-label="<alt text>">` and `<title><alt text></title>` are injected into the SVG before embedding.
8. Warm render time (font DB cached after first call): < 10ms per diagram.
9. Cold render time (first diagram in process, font DB scan): < 200ms.

## Invariants

1. No Node.js process is spawned. (D-006: single binary)
2. No HTTP request is made during rendering. (offline-compatible)
3. All 23 Mermaid diagram types supported by mermaid-rs-renderer v0.2.2 are available.
4. The rendering path: Mermaid source → mermaid-rs-renderer → usvg normalization → SvgData.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Flowchart with 4 nodes (simplest real use case) | SVG produced; no foreignObject; warm render < 3ms |
| EC-002 | ER diagram with 10 entities | SVG produced; warm render < 5ms (empirical from S14) |
| EC-003 | Gantt chart with 5 tasks | SVG produced; warm render < 1ms (empirical from S14) |
| EC-004 | PPTX embedding: SVG embedded as /ppt/media/diagramN.svg | SVG present in ZIP; relationship correctly referenced from slide XML |
| EC-005 | HTML output: SVG inlined as `<img>` or inline SVG | aria-label preserved; alt text accessible via ARIA |
| EC-006 | PDF output: SVG converted to paths via usvg+resvg | Vector paths embedded; no rasterization required for simple diagrams |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `flowchart LR\n  A[Client] --> B[API]` with alt "Architecture" | PPTX-safe SVG; no foreignObject; aria-label="Architecture"; exit 0 | happy-path (TV-12.1) |
| `sequenceDiagram\n  Alice->>Bob: Hello` with alt | Valid SVG; warm render < 3ms | happy-path |
| `gantt\n  section S1\n  Task :a1, 2026-01-01, 30d` with alt | Valid SVG; < 1ms warm | happy-path |
| Valid source, no alt field | E-A11-001 (blocked upstream by BC-5.01.001) | error (blocked upstream) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Rendered SVG contains no foreignObject element (PPTX safety) | unit test: parse SVG, check for forbidden elements |
| VP-TBD | Warm render time < 10ms for typical diagrams | Criterion benchmark |
| VP-TBD | Cold render time < 200ms | integration test (measure first call in fresh process) |
| VP-TBD | usvg normalization produces absolute dimensions (no % units) | unit test: parse normalized SVG, verify width/height are px values |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-014 ("Diagram Rendering (Mermaid)") per capabilities.md §CAP-014 |
| Capability Anchor Justification | CAP-014 ("Diagram Rendering (Mermaid)") per capabilities.md §CAP-014 — "single-binary constraint resolved by Spike S14 (mermaid-rs-renderer)" is the precise decision recorded in CAP-014 |
| L2 Domain Invariants | (none directly — D-006 single binary is a differentiator) |
| Architecture Module | slideforge-diagrams crate — MermaidRenderer (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.12.002 — related to (this BC is the happy path; BC-1.12.002 is the error path)
- BC-1.12.003 — composes with (usvg normalization is a step in this BC's rendering pipeline)
- BC-5.01.001 — depends on (alt text is required before diagram reaches this stage)

## Architecture Anchors

- `architecture/system-overview.md#diagram-renderer` — MermaidRenderer + DiagramRenderer trait
- `architecture/system-overview.md#spike-s14` — mermaid-rs-renderer selection rationale

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
