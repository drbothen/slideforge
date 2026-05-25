---
document_type: adr
adr_id: ADR-014
title: Mermaid via mermaid-rs-renderer
status: accepted
date: 2026-05-24
spike_input: S14-mermaid-diagram-engine.md
traces_to: ARCH-INDEX.md
supersedes: ~
---

# ADR-014: Mermaid Rendering via mermaid-rs-renderer

## Context

The `DiagramRenderer` plugin surface must produce PPTX-safe SVG from Mermaid source.
The single-binary constraint is the dominant filter (Q1, R3). S14 spike evaluated six
candidates; only Option A (mermaid-rs-renderer) satisfies all v1.0 constraints.

## Decision

Use `mermaid-rs-renderer = "=0.2.2"` as the `MermaidRenderer` plugin implementation.
Post-process all SVG output via `usvg = "=0.47.0"` before PPTX embedding.

## Consequences

**Performance:**
- Cold font DB scan: ~124ms (amortized — once per process).
- Warm render: < 3ms per diagram. For 5 diagrams in a 25-slide deck: ~124ms + 4×3ms = ~136ms.
- Combined with parse (89µs) + PPTX export: well within NFR-001 (< 500ms).

**PPTX-safe SVG:**
- Output: pure `<text>`, `<rect>`, `<path>` elements. No `<foreignObject>`, no `<script>`.
- `viewBox`, `width`, and `height` present on all outputs. 8/8 diagram types confirmed.
- usvg normalization step: flattens CSS classes, resolves `<use>` refs, ensures absolute dimensions.

**Coverage:** 23 diagram types covering all v1.0 use cases. Core types (flowchart, sequence,
class, state) confirmed PPTX-safe. selkie-rs reports 85% structural parity with mermaid.js
reference — the 15% gap is aesthetic (label positioning), not structural.

**`#![forbid(unsafe_code)]`:** `slideforge-diagrams` enforces this lint on its own code.
Transitive `unsafe` in `fontdb`/`ttf-parser` (font file mmap) is in separately audited,
widely-used crates — considered sound and acceptable.

**Escape hatch:** `selkie-rs = "=0.3.0"` (the underlying engine) is available as a
direct dependency if `mermaid-rs-renderer` proves limiting. Switching is a dep swap,
not an architecture redesign.

**Rejected alternatives:**
- mermaid-cli (mmdc): Node.js + 200MB Chromium; `<foreignObject>` in SVG output (PPTX-unsafe).
- Headless browser (Playwright): same blocking issues as mmdc.
- Kroki (HTTP): offline-hostile; requires network or Docker.
- Mermaid WASM: no existing implementation; 6-12 weeks R&D with uncertain outcome.
