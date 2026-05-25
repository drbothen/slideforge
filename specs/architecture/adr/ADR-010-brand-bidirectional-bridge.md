---
document_type: adr
adr_id: ADR-010
title: Brand bidirectional bridge
status: accepted
date: 2026-05-24
spike_input: S5-brand-synthesis-layout-taxonomy.md
traces_to: ARCH-INDEX.md
supersedes: ~
---

# ADR-010: Brand Bidirectional Bridge

## Context

Corporate users have existing `.pptx` brand templates. New users want to define brand from
a `brand.toml` spec. Both flows must produce the same internal `Brand` struct that drives
layout and PPTX/DOCX export. S5 spike confirmed both directions work and established the
definitive layout taxonomy (31 layouts).

## Decision

`slideforge-brand` implements the `BrandProvider` trait with bidirectional support:
- **Synthesis direction:** `brand.toml` → `BrandTemplate` → `.pptx` / `.docx` template bytes
- **Extraction direction:** existing `.pptx` bytes → `BrandConfig` (TOML)

## Consequences

**31-layout taxonomy (locked by S5):**
- 11 standard OOXML layouts (required for cross-renderer compatibility)
- 20 custom slideforge layouts (CL-01 through CL-20, `SF ` prefix for grouping)
- Total: 31 layouts per brand template

**Single slide master architecture (DI-016):**
All slides in a deck share one slide master. `brand_overlay:` swaps placeholder content
(logo, footer text) but does not switch masters. Multi-master deferred to v2.
Rationale: multi-master degrades in Google Slides (Q4 decision).

**Dark layout invariant (Feasibility Note 4):**
CL-01 (SF Section Divider) and CL-11 (SF End Slide) use `clrMapOvr`. For LibreOffice
compatibility, BOTH `clrMapOvr` AND explicit white run colors are written on all
placeholder text. This is a binding implementation constraint (see brand-architecture.md).

**ECMA-376 element ordering:** The synthesis function enforces strict child ordering in
theme1.xml (`clrScheme → fontScheme → fmtScheme`), slideMaster1.xml, and all layout files.
Violated ordering causes PowerPoint to silently reset to default theme.

**Extraction completeness:** All 12 OOXML theme color slots are extracted (DI-015).
Missing slots produce `BrandError::MissingColorSlot`. Logo extraction parses
`slideMaster1.xml.rels` media relationships.

**S5 prototype validated:** 27 KB `.pptx` with 16 layouts generated, extraction round-trip
confirmed. Production implementation extends to all 31 layouts following identical patterns.
