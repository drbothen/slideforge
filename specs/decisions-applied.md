---
title: Slideforge Decisions Applied
source: "extracted from PROJECT-SEED.md 2026-05-23"
parent: product-brief.md
version: 1.0
created: 2026-05-23
status: BINDING
---

# Decisions Applied (2026-05-23)

> Source: §11.A and §11.A.1 from PROJECT-SEED.md. Verbatim extraction with one
> targeted amendment to Q7 (noted below). All decisions are BINDING for v1.0.
> The agent factory MUST treat these as authoritative over the seed §6 phase plan.

---

## §11.A Decisions Applied (2026-05-23)

The 7 Open Questions in §11 have been resolved by the human reviewer on 2026-05-23. The agent factory should treat these decisions as binding for v1.0.

### Q1 — Project name
`slideforge` (confirmed; matches scaffolding crate names crates/slideforge*; no rename required)

### Q2 — DSL syntax style
Indentation-significant (YAML/Python-like). chumsky semantic-indentation parser. NOT brace-delimited.

### Q3 — Multi-file project support
`@include "path.sf"` directives supported. Affects parser (source-span tracking across files), eval (resolution + cycle detection), and project config (`slideforge.toml`).

### Q4 — Brand template format
BIDIRECTIONAL BRIDGE in v1.0. Both `.pptx` and `.toml` accepted as input. Plus a new CLI command `slideforge extract-brand <template.pptx> -o brand.toml` that scans an existing `.pptx` and emits a deterministic `.toml` manifest. Plus FULL SYNTHESIS in v1.0 — given only a `.toml` (no base `.pptx`), slideforge generates a complete valid `.pptx` brand scaffold from scratch (theme XML, slide master, all 11 slide layouts, notes master, handout master, relationships, content types, embedded logo media). This is a major scope expansion vs. the seed's "load template" approach.

### Q5 — PDF/HTML exporters scope
All three exporters (PPTX + PDF + HTML) ship in v1.0. NOT deferred to v1.x. PDF backend choice (Typst-as-backend vs. direct printpdf/lopdf) is an OPEN ADR for the architect.

### Q6 — Python binding API style
DSL-only. Python integration means Python builds `.sf` strings and shells out to the `slideforge` CLI binary; NO pyo3 dict-based API in v1.0. Decision: pyo3 deferred indefinitely unless explicit user demand surfaces.

### Q7 — Live preview architecture

**AMENDED 2026-05-23 — "likely" wording removed.**

`slideforge watch` runs an embedded web server with websocket reload and a canvas-based renderer. **Whether the canvas renderer shares code with the HTML exporter or stands alone is OPEN — see ADR-008 (architect to decide in Phase 1).**

NOT just-rebuild-pptx. NOT browser-tab-HTML-only. NOT defer.

> Original seed wording (preserved for audit trail): "embedded web server with websocket reload and a canvas-based renderer (likely reusing the HTML exporter from Q5 as the underlying renderer)" — the word "likely" indicated an open design question, not a closed decision. ADR-008 formalizes this as an explicit open question for the Phase 1 architect.

---

### ADR Candidates for Phase 1 architect

The following architectural decisions MUST be addressed during Phase 1 spec crystallization. Each warrants a formal ADR:

- **ADR-001 (proposed):** Brand synthesis approach — full-from-scratch OOXML generation; document layout taxonomy that maps 23 slide types to slideLayoutN.xml files; placeholder positioning model; theme XML generation strategy
- **ADR-002 (proposed):** Multi-renderer snapshot test matrix — PowerPoint + Keynote + Google Slides + LibreOffice headless rendering parity guarantees
- **ADR-003 (proposed):** PDF backend choice (Typst-as-backend mirroring office2pdf, OR direct PDF via printpdf/lopdf, OR HTML→PDF via headless browser)
- **ADR-004 (proposed):** @include resolution semantics — relative vs. absolute paths, cycle detection, source-span propagation across files
- **ADR-005 (proposed):** Web-preview architecture — embedded server (axum?), websocket protocol, canvas renderer choice, hot-reload semantics
- **ADR-006 (proposed):** IR shape — does the same IR feed PPTX + PDF + HTML + canvas, or do we have format-specific lowering passes?
- **ADR-007 (proposed):** MSRV policy and ooxmlsdk integration — covers the 1.85→1.88 bump and version-pinning policy. HIGH severity per preflight. Must be decided before any crate implementing ooxmlsdk is started.
- **ADR-008 (proposed):** Web-preview canvas renderer architecture — is the canvas renderer the HTML exporter's output piped to a browser canvas, a separate IR-to-canvas renderer, or an SVG-intermediate approach? This is distinct from ADR-005 (server architecture) and determines whether the HTML exporter crate must be designed for dual-use (static export + live preview).
- **ADR-009 (proposed):** Error recovery semantics — what does the parser do when it sees a malformed slide? Partial AST? Error continuation? Recovery hints? What IR is produced on partial parse, and is layout/export attempted?
- **ADR-010 (proposed):** IR stability and exporter plugin contract — is the IR a stable internal contract that 3rd-party exporters can target? Is there a first-class plugin interface? Is `slideforge-ffi` the intended external exporter surface?
- **ADR-011 (proposed):** WCAG AA tooling choice — pa11y vs. playwright+axe vs. lighthouse CLI. Must be decided before web-preview story is decomposed, as it affects CI design and HTML exporter output requirements.

---

### Scope expansion note

The 7 answers represent a ~2x scope expansion vs. seed Section 6 Phase 2-4 estimates. Specifically: full brand synthesis (Q4) is roughly equivalent in size to "all 23 slide types"; PDF + HTML + web-preview in v1.0 (Q5, Q7) adds another major chunk. The seed's phased plan MUST be re-scoped by the product-owner during Phase 1 PRD work. Do NOT transcribe seed §6 verbatim into the PRD.

---

## §11.A.1 Quality Bar (declared 2026-05-23)

**Production-grade from day 1** is now a binding constraint for v1.0. All gates enumerated in `STATE.md > Quality Bar` apply. Key downstream consequences for spec/architecture work:

- Architect MUST design with formal verification in mind from the start (pure-core boundaries, Kani-amenable function signatures).
- Architect MUST include security and supply-chain ADRs in the Phase 1 deliverable set.
- Architect MUST define the multi-renderer parity test matrix as part of the architecture doc.
- Product-owner MUST include performance budgets (< 500ms cold, < 50ms incremental) as NFRs with measurable gates.
- Product-owner MUST include accessibility (WCAG AA on web preview) as a binding NFR.
- Story-writer MUST decompose CI/CD matrix expansion as Phase 1 stories (NOT deferred to Phase 4 polish).
- Per-story-delivery includes adversarial review + security review + demo evidence for EVERY story without exception.

---

### §11.A.2 Q1 Decision — Locked (2026-05-24)

**Original question:** "Does the DSL have computation?"

**Final answer:** Q1 expanded during discussion into the foundational product scope decision. The complete decision is documented in `.factory/planning/q1-decision-final.md` (13 sections, version roadmap through v3).

**Key decisions:**
- Computation: Data-reactive declarative (rungs 1-9). User-defined functions in v2.
- Output formats: PPTX + DOCX + PDF + HTML + web preview (5 formats in v1.0)
- Writing registers: `notes` (presenter) + `report` (reader) + `detail` (document-only) — three separate fields, linguistically distinct
- Document section types: ~15 (10 auto-generated from slide data + 5 document-only manual sections)
- Charts: SVG via plotters crate. Native OOXML ChartML in v2.
- Math: $...$ / $$...$$ with mode-based parsing. {{ }} disabled in math; @{var} enabled. LaTeX → OMML via pulldown-latex + mml2omml.xsl.
- Brand bridge: bidirectional for BOTH .pptx AND .docx. Unified brand.toml.
- LaTeX/Beamer: v2 output format (slideforge-latex exporter crate)
- Crate layout: 13 crates (added slideforge-docx, slideforge-data, slideforge-brand, slideforge-preview)

**Canonical reference:** `.factory/planning/q1-decision-final.md`
