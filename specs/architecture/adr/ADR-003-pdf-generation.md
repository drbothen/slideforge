---
document_type: adr
adr_id: ADR-003
title: PDF generation via pdf-writer + krilla
status: accepted
date: 2026-05-24
spike_input: S2-pdf-backend-evaluation.md
traces_to: ARCH-INDEX.md
supersedes: ~
---

# ADR-003: PDF Generation via pdf-writer + krilla

## Context

slideforge must produce PDF/UA-1 compliant output (`veraPDF --flavour ua1` must pass — v1.0
quality bar, NFR-012). The PDF exporter consumes `LaidOutDeck` with pre-positioned EMU
coordinates. No Rust crate ships a turnkey PDF/UA-1 API. Spike S2 evaluated 5 candidates.

## Decision

Use `pdf-writer = "=0.14"` + `krilla = "=0.6"` with a custom tagged-PDF engine
(`SlideTagEngine`) in `slideforge-pdf`. The SlideTagEngine builds the `/StructTreeRoot`
required for PDF/UA-1.

## Decoupled Draw/Tag Interface (Feasibility Note 1)

`SlideTagEngine` MUST decouple its draw layer from its tag layer:

**Draw layer (krilla):** path fills, strokes, glyph placement, image/SVG embedding. Stable.

**Tag layer — primary:** krilla v0.6.0 experimental tagged PDF hooks for `/MCID` assignment
and structure tree linking.

**Tag layer — fallback:** If krilla's tagging fails `veraPDF` after ≤ 5 story-days of
Phase 6 hardening effort, switch to direct `pdf-writer` structure tree construction while
keeping krilla for drawing. No coordinate mapping rewrite required.

**Fallback-of-fallback:** Typst template approach (generate `.typ` source from
`LaidOutDeck` via Typst's `place()` function). Requires human decision to escalate.

## Consequences

**Positive:**
- Only path to PDF/UA-1 compliance within single-binary constraint.
- Both `pdf-writer` and `krilla` are by Typst's author (Laurenz Villiger) — same
  knowledge base as Typst's gold-standard PDF/UA-1 output.
- SVG chart embedding via usvg path extraction (vector quality).

**Risks tracked:**
- krilla experimental tagged PDF layer may not pass PDF/UA-1 (R1, feasibility report).
  Mitigation: fallback trigger documented above.
- Font subsetting complexity (R2): use `subsetter` crate (Typst team, Apache-2.0).

**Rejected alternatives:**
- printpdf: no PDF/UA-1 structure tree API.
- genpdf: no structure tree; architecture mismatch with pre-positioned IR.
- lopdf: too low-level; no structural support.
- Headless Chrome: 150-250 MB binary; single-binary-hostile; PDF/UA-1 unreliable for absolute-position layouts.
- Typst-as-library: cannot accept pre-positioned IR without architecture mismatch.

## Implementation Plan (Phase 4 entry)

1. Move `crates/slideforge-pdf` to `workspace.members`.
2. Add pinned deps: `pdf-writer = "=0.14"`, `krilla = "=0.6"`, `usvg = "=0.47"`, `subsetter = "=0.2"`.
3. Implement `SlideTagEngine` as first story (pure core, fully testable).
4. Implement coordinate mapping: VP-006 Kani proof candidate.
5. Integrate krilla drawing; add veraPDF CI gate.
