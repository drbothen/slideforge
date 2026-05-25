---
document_type: domain-spec-section
level: L2
section: "differentiators"
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs:
  - .factory/planning/dsl-competitor-analysis.md
  - .factory/planning/domain-research.md
  - .factory/planning/q1-decision-final.md
  - .factory/planning/q3-decision-final.md
  - .factory/planning/q4-q15-decisions.md
input-hash: "[pending]"
traces_to: L2-INDEX.md
---

# Competitive Differentiators

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.
> Source: q1-decision-final.md §0, domain-research.md §3, dsl-competitor-analysis.md.

---

## Differentiator Table

| Differentiator | Description | Supporting Capabilities | Competitor Gap |
|----------------|-------------|------------------------|---------------|
| D-001: Live Data Reactivity | `@data` binds external sources at compile time; watch mode polls and auto-refreshes; enables nightly CI-driven branded reports without manual authoring | CAP-003, CAP-004, CAP-027 | No competitor offers compile-time data binding with watch mode refresh in a PPTX-producing tool. Typst has native data loading but produces PDF-only. Quarto requires R/Python runtime. |
| D-002: Opinionated Shared Slide Vocabulary | 31 named slide types with enforced semantics; every user means the same thing by `slide severity_cards:`. The shared vocabulary is the portability layer. | CAP-010, CAP-009 | Slidev/Marp have no typed slide vocabulary — every layout is ad-hoc CSS. Typst's slide frameworks (touying/polylux) have types but not branded enterprise templates. |
| D-003: Production-Grade Multi-Format from One Source | One .sf file → PPTX + DOCX + PDF + HTML + web preview. Writing registers route content to the right format. Document sections auto-generate from slide data. | CAP-015, CAP-016, CAP-017, CAP-029 | Slidev: HTML only. Marp: HTML + limited PPTX. Typst: PDF only. Quarto: PPTX + PDF + HTML but requires R/Python and cannot produce branded OOXML from a single DSL. |
| D-004: OOXML Brand Template Bridge | Load any corporate .pptx or .docx template; synthesize brand from brand.toml; extract brand from existing template. Bidirectional. | CAP-018, CAP-019 | All Markdown-based tools (Marp, Slidev, Quarto) use CSS themes with no OOXML brand awareness. python-pptx supports templates but has no synthesis or extraction. |
| D-005: Compile-Time Accessibility Enforcement | `alt "..."` missing = compile error. `label "..."` missing on color-coded elements = compile error. WCAG AA validated in CI. No deferred or runtime accessibility. | CAP-020, CAP-022 | No competitor enforces accessibility at compile time. Marp has known a11y failures (Ctrl+ zoom). Slidev has no BiDi support. python-pptx has no accessibility validation. |
| D-006: Single Binary, Zero Runtime Dependencies | One binary installs everywhere. No Node.js, no Python, no npm. Comparable only to Marp CLI in the competitor field, but Marp has no PPTX output. | CAP-001 through CAP-030 (deployment) | Slidev requires Node + Vite. Quarto requires R/Python. python-pptx requires Python. Only Marp CLI is also a single binary, but without PPTX and brand support. |
| D-007: Plugin-First Extensibility with Dog-Fooding Guarantee | All 10 extensibility surfaces are traits; bundled plugins use the same API as future external plugins. The trait API is proven by the bundled implementation. | CAP-021 | No competitor offers a trait-based plugin system with dog-fooding guarantees. python-pptx is not extensible. Slidev's component system fragments the ecosystem. |

---

## Competitor Gap Summary

| Capability | Slidev | Marp | Typst | Quarto | python-pptx | slideforge |
|-----------|--------|------|-------|--------|-------------|------------|
| PPTX output | No | Limited | No | Yes | Yes | **Yes (production-grade)** |
| Data-reactive DSL | Vue components | No | Yes (PDF) | Yes (R/Python) | Python code | **Yes (DSL-native, all formats)** |
| Brand template system | CSS themes | CSS themes | Internal templates | Themes | Manual | **Full OOXML brand bridge** |
| Single binary | No (Node) | Yes | Yes | No | No | **Yes** |
| Compile-time a11y | No | No | No | No | No | **Yes** |
| Multi-format output | HTML only | HTML + PPTX | PDF only | PPTX + PDF + HTML | PPTX only | **5 formats** |

Source: domain-research.md §3 (competitive landscape update, May 2026).

---

## Differentiation Sustainability Analysis

| Differentiator | Moat Strength | Replication Timeline by Best Competitor |
|----------------|--------------|----------------------------------------|
| D-001: Live Data Reactivity + PPTX | HIGH — Typst would need to add PPTX export first | 18-24 months for Typst (PDF-only today) |
| D-002: Opinionated Type Vocabulary | HIGH — requires domain expertise + library of templates | Already fragmented in Slidev; hard to retrofit |
| D-003: Multi-Format from One Source | MEDIUM — Quarto is close but requires R/Python | 6-12 months for Quarto to close OOXML gap |
| D-004: OOXML Brand Bridge | HIGH — requires deep OOXML knowledge | 12+ months for any new entrant |
| D-005: Compile-Time Accessibility | MEDIUM — technically feasible for others | 3-6 months for a focused effort |
| D-006: Single Binary | LOW — achievable for others | Already done by Marp CLI for HTML output |
| D-007: Plugin-First Dog-Fooding | LOW — architectural pattern, not unique | 6-12 months for similar architecture |
