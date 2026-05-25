---
document_type: domain-spec-section
level: L2
section: "assumptions"
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs:
  - .factory/specs/product-brief.md
  - .factory/planning/domain-research.md
  - .factory/planning/dsl-competitor-analysis.md
  - .factory/planning/q3-decision-final.md
input-hash: "[pending]"
traces_to: L2-INDEX.md
---

# Assumptions

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.
> Every ASM has Status: unvalidated until confirmed by research or implementation.

---

| ASM-ID | Assumption | Confidence | Impact-if-Wrong | Validation Method | Status | Traced To |
|--------|-----------|-----------|----------------|------------------|--------|-----------|
| ASM-001 | Engineering teams writing structured slide specs will accept an indentation-significant DSL syntax as their primary authoring interface rather than a GUI | Medium | HIGH — entire product premise fails if target users reject CLI/DSL authoring | User research or beta with 5+ target teams; analyze adoption rate vs. Python reference tool | unvalidated | CAP-001, product-brief.md §1 — Holdout candidate: yes |
| ASM-002 | ooxmlsdk v0.6 can produce PPTX files that render correctly across PowerPoint, Keynote, Google Slides, and LibreOffice with production-grade fidelity | Medium | HIGH — PPTX export is the primary v1.0 differentiator; incorrect output blocks all adoption | Implement a reference deck and open it in all four renderers as a spike (S-OOXML); inspect element ordering and placeholder inheritance | unvalidated | CAP-015, DI-013 — Holdout candidate: yes |
| ASM-003 | Spike S14 will identify a viable single-binary Mermaid rendering approach (mermaid-rs or WASM) that meets quality bar | Low | HIGH — diagram slide type ships v1.0 but requires a rendering engine; if S14 fails, diagram type must be deferred or the single-binary constraint relaxed | Execute Spike S14 in Phase 1; benchmark both mermaid-rs and bundled WASM options | unvalidated | CAP-014 — Holdout candidate: yes |
| ASM-004 | The plotters crate (pure Rust SVG charts) produces chart output visually acceptable for branded enterprise presentations | Medium | MEDIUM — chart slide type ships v1.0; inadequate chart quality would require vendor change | Render sample charts covering bar, line, pie, scatter with brand colors; review against standard presentation quality bar | unvalidated | CAP-013 |
| ASM-005 | pulldown-latex v0.7.1 covers the LaTeX subset used by target users (basic equations, fractions, summation, matrix notation) without gaps that require a more complete engine | Medium | MEDIUM — math rendering quality affects usability for technical content; gaps require fallback | Test the top 20 LaTeX expressions from engineering slide decks against pulldown-latex; document any gaps and whether KaTeX fallback covers them | unvalidated | CAP-012 |
| ASM-006 | MathML-to-OMML conversion via Microsoft's mml2omml.xsl + libxslt FFI is acceptable for v1.0 despite introducing a C dependency | Medium | MEDIUM — introduces build complexity and potential portability issues on MUSL targets; wrong if libxslt bundling is too complex | Prototype the FFI on Linux x86_64 and macOS arm64; measure binary size overhead; verify MUSL cross-compile works | unvalidated | CAP-012 |
| ASM-007 | Compiling from HTML to PDF via Chrome headless (--print-to-pdf) produces tagged PDF that can be made PDF/UA-1 compliant via post-processing | Low | HIGH — PDF/UA-1 is in the quality bar; if Chrome headless cannot produce tagged PDF, the PDF exporter architecture must change | Research Chrome --print-to-pdf tagged PDF support; prototype and run veraPDF against output | unvalidated | CAP-017, DI-014 — Holdout candidate: yes |
| ASM-008 | Users who currently use python-pptx will migrate to slideforge if the DSL is easier to write than Python dicts and produces equivalent visual output | Medium | MEDIUM — if migration friction is too high, python-pptx users stay; reduces TAM | Beta test with 3+ python-pptx users; compare effort for a reference 25-slide deck | unvalidated | product-brief.md §2 |
| ASM-009 | The DOCX Level 1 output (report register → body paragraphs, auto-generated sections) is sufficient for v1.0 without needing full flowing document layout | High | LOW — DOCX Level 2 is planned for v1.x; Level 1 covers the incident-report use case from the seed reference | Validate DOCX Level 1 output satisfies the 1898 & Co. incident report format; get feedback from at least one user | unvalidated | CAP-016 |
| ASM-010 | Git-based package distribution (no hosted registry) is sufficient for v1.0 package sharing needs | Medium | LOW — if users need a package registry, the git-based approach still works but discovery is harder | Survey v1.0 users on package-sharing workflows; plan registry for v1.x if demand exists | unvalidated | CAP-025 |
| ASM-011 | The 500ms cold build time target for a 25-slide deck is achievable without comemo incremental compilation | Medium | MEDIUM — if 500ms is not achievable in straight-through build, incremental compilation moves from v1.x to v1.0 | Implement the core pipeline for 5 slide types and benchmark; extrapolate to 31 types and 25 slides | unvalidated | R-007 |
| ASM-012 | The Rust chumsky 0.10 parser combinator supports the slideforge DSL grammar without requiring custom error recovery beyond what chumsky provides | High | LOW — chumsky is designed for DSL authoring and error recovery; low risk that the grammar would exceed its capabilities | Prototype the DSL grammar for 3 slide types and test error recovery behavior | unvalidated | CAP-001 |
| ASM-013 | WCAG AA compliance for the web preview SVG canvas is achievable via @axe-core/playwright without requiring a full ARIA role tree | Medium | MEDIUM — SVG accessibility is complex; incorrect ARIA implementation could require rework | Run axe-core against a prototype SVG canvas; document required ARIA attributes per element type | unvalidated | CAP-017, CAP-020 |
| ASM-014 | Font metric divergence (slide text overflow on non-Windows platforms) is manageable with a 10% overflow margin warning strategy rather than requiring per-platform font metric databases | Medium | MEDIUM — if font metrics diverge by more than 10% across platforms, overflow warnings become unreliable and layout becomes non-deterministic | Measure actual font metrics for Liberation Sans vs. Aptos on Windows, macOS, and Linux; document divergence magnitude | unvalidated | DEC-013, domain-research.md §5.2 |
