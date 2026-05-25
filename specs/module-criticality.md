---
document_type: module-criticality
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
phase: 1b
lifecycle: mutable through Phase 5; frozen after Phase 5 gate
traces_to: .factory/specs/architecture/ARCH-INDEX.md
---

# Module Criticality — slideforge v1.0

> Classifies every crate by kill-rate tier for mutation testing budget (cargo-mutants).
> CRITICAL ≥ 95% kill rate | HIGH ≥ 90% | MEDIUM ≥ 80% | LOW ≥ 70%

---

## CRITICAL (≥ 95% kill rate required)

These crates implement invariants that, if violated, produce silently corrupt output
or broken accessibility. Formal verification (Kani) applies to their pure-core functions.

| Crate | Justification |
|-------|--------------|
| slideforge-syntax | DSL parsing is the entry point for every user interaction. Parse errors with wrong spans or tab-detection failures corrupt the entire pipeline. VPs VP-001, VP-003, VP-009, VP-014. |
| slideforge-eval | Type system and variable evaluation. Implicit coercion (DI-004) or scope bugs (DI-006) produce silently wrong slide content. VPs VP-004, VP-005, VP-010, VP-015. |
| slideforge-validate | Owns ALL compile-time validation — alt text, color contrast, canvas overflow. A bug here silently ships inaccessible documents. VPs VP-002, VP-007, VP-008. |

---

## HIGH (≥ 90% kill rate required)

Crates where bugs produce incorrect but detectable output (wrong PPTX structure,
wrong coordinates, wrong color values). Visual regression and snapshot tests catch
most failures, but mutation testing ensures coverage is not superficial.

| Crate | Justification |
|-------|--------------|
| slideforge-layout | Deck → LaidOutDeck conversion. Wrong coordinates produce layout bugs visible in every output format. VP-011. |
| slideforge-pptx | PPTX serialization. Incorrect placeholder inheritance, missing Content_Types entries, or wrong color values produce corrupt files or visual regressions. VP-013. |
| slideforge-brand | 31-layout synthesis. Dark layout invariant (clrMapOvr + explicit white) — incorrect synthesis produces wrong colors in LibreOffice. VP-012. |
| slideforge-types | IR type definitions. Incorrect Hash/Eq/Clone impls corrupt incremental computation and Kani proofs. |
| slideforge-plugin-api | Trait contracts. A subtle trait API change breaks all plugin implementations silently. |

---

## MEDIUM (≥ 80% kill rate required)

Crates where bugs are largely caught by snapshot tests or external validation tools,
but mutation testing ensures the logic is genuinely exercised.

| Crate | Justification |
|-------|--------------|
| slideforge-charts | SVG chart generation. Visual regression CI catches most output bugs. Mutation testing ensures algorithmic correctness of data-to-SVG mapping. |
| slideforge-diagrams | Diagram rendering and SVG normalization. Unit tests cover the usvg pipeline; mutation testing catches edge cases in the normalization logic. |
| slideforge-math | LaTeX transformation. Output is verified by downstream format-specific tests; mutation testing ensures formula correctness. |
| slideforge-pdf | PDF coordinate mapping (VP-006 — CRITICAL subset). Structure tree construction verified by veraPDF gate. Overall crate is MEDIUM because veraPDF provides the authoritative correctness signal. |
| slideforge-docx | DOCX serialization. Snapshot tests cover correctness; mutation testing ensures register (notes/report/detail) routing is correct. |
| slideforge-html | HTML exporter; peer to slideforge-docx. Snapshot tests cover correctness; axe-core/playwright provides the authoritative WCAG correctness signal. |

---

## LOW (≥ 70% kill rate required)

Crates that are primarily infrastructure and I/O orchestration. Correctness is
largely verified by integration tests and CI pipeline behavior.

| Crate | Justification |
|-------|--------------|
| slideforge-data | DataSource plugins. External data fetching. Integration tests cover the I/O paths; the logic is relatively thin (fetch → return Value). |
| slideforge-preview | axum server + WebSocket. Correctness verified by axe-core/playwright E2E tests. The server logic is straightforward. |
| slideforge-package | sf.lock management and git-based package install. Integration tests cover the package install/verify cycle. |
| slideforge-config | Config cascade parsing. Unit tests cover precedence rules; the logic is straightforward TOML deserialization. |
| slideforge | Plugin registry assembly. Correctness is verified by the full integration test suite — all bundled plugins exercised. |
| slideforge-cli | CLI orchestration. Integration tests cover the command matrix. Mutation testing for CLI argument parsing has low ROI. |
