---
document_type: adr
adr_id: ADR-011
title: WCAG AA toolchain
status: accepted
date: 2026-05-24
spike_input: S3-wcag-tooling-choice.md
traces_to: ARCH-INDEX.md
supersedes: ~
---

# ADR-011: WCAG AA Toolchain

## Context

All four output surfaces must meet WCAG AA (NFR-012 through NFR-015). S3 spike evaluated
accessibility tooling for each surface. DSL-level alt enforcement (DI-001) must be compile-time.

## Decision

Per-surface toolchain:

| Surface | CI Gate (every PR) | Release Gate (manual) |
|---------|-------------------|----------------------|
| Web preview | `@axe-core/playwright` — zero serious/critical | VoiceOver + NVDA walkthrough |
| HTML export | `@axe-core/playwright` — zero serious/critical | Same screen-reader walkthrough |
| PDF export | `veraPDF -f ua1` (Docker sidecar) — exit 0 required | PAC 2024 on Windows VM |
| PPTX export | Custom OOXML linter (Rust, in `slideforge-pptx`) | PowerPoint Accessibility Checker |
| Brand colors | `slideforge-validate` contrast checker — compile-time | N/A |
| DSL `alt` | `slideforge-syntax` chumsky validator — compile-time | N/A |

## Consequences

**axe-core/playwright (web/HTML):**
- WCAG tags: `wcag2a`, `wcag2aa`, `wcag21a`, `wcag21aa`, `wcag22a`, `wcag22aa`
- Node.js lives only in `crates/slideforge-preview/tests/package.json` — not in production binary
- pa11y rejected: functional duplicate with SVG ARIA false positives (HTML_CodeSniffer mode)

**veraPDF (PDF):**
- Docker sidecar: `verapdf/rest:1.26.0` in GitHub Actions `services:` block
- On macOS dev machines: veraPDF not run locally — trust CI
- PAC 2024: Windows-only desktop tool, manual per-release gate

**Custom OOXML linter (PPTX):**
- Rules A1-A4 prototyped in S3: slide title placeholder, shape alt text, table header row, language
- Runs at emit time inside `slideforge-pptx` — compile-time enforcement, not post-hoc check
- Production: `quick-xml` element walker (not string matching) with full rule set

**Contrast checker (brand):**
- Pure function in `slideforge-validate::BrandValidator::check_theme_pairs()`
- WCAG 2.x luminance linearization uses 0.04045 threshold (not 0.03928 from older docs)
- Kani proof candidate (VP-007)

**SVG constraint (ADR-008 linkage):**
Web preview MUST use SVG rendering, not `<canvas>`. This is an S3-derived architecture
constraint — a bare canvas is opaque to axe-core.
