---
document_type: architecture-section
section: system-overview
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
traces_to: ARCH-INDEX.md
---

# System Overview

## Pipeline

slideforge compiles a single `.sf` source file through six sequential stages:

```
.sf source
    │
    ▼ [SS-01] slideforge-syntax
  Typed AST (chumsky 0.10 + hand-written indentation lexer)
    │
    ▼ [SS-02] slideforge-eval  ← @data sources (SS-10 slideforge-data)
  Deck IR (semantic, pre-layout)
    │
    ▼ [SS-04] slideforge-brand  ← brand.toml or .pptx template
  Deck + Brand
    │
    ▼ [SS-03] slideforge-validate
  Validated Deck (all compile-time checks pass; brand palette available for contrast checks)
    │
    ▼ [SS-05] slideforge-layout (31 SlideType plugins from SS-15)
  LaidOutDeck IR (geometric, post-layout)
    │
    ├──▶ [SS-06] slideforge-pptx  → output.pptx
    ├──▶ [SS-07] slideforge-pdf   → output.pdf  (Phase 4)
    ├──▶ [SS-08] slideforge-docx  → output.docx
    └──▶ [SS-09] slideforge-preview → live web preview (axum)
```

Each stage boundary is a stable typed contract: the Typed AST, Deck IR, and
LaidOutDeck IR are the three integration points between subsystems.

## Bounded Contexts

The four domain bounded contexts map to the six pipeline stages:

| Bounded Context | Pipeline Stages | Stable Contract |
|----------------|----------------|----------------|
| Authoring | Parse + Evaluate + Brand + Validate | Deck IR (validated, with Brand) |
| Branding | Brand loading (stage 3, before Validate) | Brand struct |
| Layout | Layout | LaidOutDeck IR |
| Export | Export | Output bytes per format |

The Cross-Cutting context (accessibility, plugin system, packages, workspace)
provides infrastructure consumed by every stage.

## Watch Mode

In `slideforge watch` / `slideforge serve`, the pipeline re-runs on change:
- File watcher (`notify` crate) monitors `.sf` and included files
- HTTP data sources poll on configurable intervals
- Changed files trigger re-evaluation from the earliest changed stage
- LaidOutDeck delta is pushed to connected browsers via WebSocket

The incremental rebuild budget (NFR-002: < 50ms) is satisfied by:
- Parse-only time for 25 slides: 71µs (S4 benchmark)
- Total NFR-001 (< 500ms cold) leaves ~429ms for eval + layout + export

## Single-Binary Architecture

slideforge ships as a single statically-linked Rust binary. No runtime
dependencies are required:
- No Node.js (mermaid-rs-renderer is pure Rust; S14 spike confirmed)
- No Chromium (rejected in S2 for PDF, S14 for diagrams)
- No Java (veraPDF is CI-only, not embedded in the binary)
- The web preview's axum server is embedded; `slideforge serve` spawns it in-process

This constraint is enforced by the cargo deny policy (dependency-graph.md).
