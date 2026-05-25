---
document_type: domain-spec-section
level: L2
section: "risks"
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
  - .factory/specs/risks.md
input-hash: "[pending]"
traces_to: L2-INDEX.md
---

# Risk Register

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.
> All risks Status: open. Category: security | performance | reliability | business.

---

| R-ID | Risk | Likelihood | Impact | Category | Status | Mitigation | Traced To |
|------|------|-----------|--------|---------|--------|-----------|-----------|
| R-001 | ooxmlsdk v0.6 has correctness gaps in placeholder inheritance or element ordering that cause broken PPTX output in one or more renderers | Medium | HIGH | reliability | open | Execute OOXML correctness spike in Phase 1; validate against all 4 renderers before writing story stubs; pin ooxmlsdk version with `=` in Cargo.toml; document known gaps in ADR-007. NFR candidate: yes — multi-renderer parity gate is in CLAUDE.md quality bar | ASM-002, CAP-015, DI-013 |
| R-002 | Single-binary Mermaid rendering (Spike S14) fails: mermaid-rs is too immature and WASM embedding is too large | Medium | HIGH | reliability | open | S14 spike must complete in Phase 1 before diagram slide type is committed to a wave. If S14 fails: either defer diagram type to v1.x or make Node.js an optional dependency with a clear user-facing install note. Architect owns the decision. | ASM-003, CAP-014 |
| R-003 | HTML-to-PDF pipeline via Chrome headless cannot produce PDF/UA-1 compliant tagged PDF | Medium | HIGH | reliability | open | Research Chrome headless tagged PDF support as part of ASM-007 validation. Mitigation if Chrome fails: switch PDF exporter to pdf-rs or lopdf with direct tagged PDF generation. NFR candidate: yes — veraPDF in CI quality bar. Security focus: no | ASM-007, CAP-017, DI-014 |
| R-004 | Font metric divergence across platforms causes text overflow in PPTX slides rendered on Linux or macOS using different fonts than the brand specifies | Medium | MEDIUM | reliability | open | Use Liberation/Noto as fallback font chain with documented panose classification; implement 10% overflow margin warning; measure divergence per ASM-014; document in brand.toml guidelines. NFR candidate: yes — 500ms perf gate requires layout that avoids re-measurement. | ASM-014, DEC-013 |
| R-005 | The 500ms cold build target for a 25-slide deck is not achievable with straight-through compilation for 31 slide types | Medium | MEDIUM | performance | open | Benchmark after 5 slide types implemented (Phase 3); assess parallelism opportunities in layout and export stages. If target is breached: comemo incremental compilation moves from v1.x to v1.0 scope. NFR candidate: yes — 500ms is a CLAUDE.md quality gate | ASM-011, CAP-027 |
| R-006 | HTTP data source fetching at compile time introduces network dependency that breaks air-gapped or offline builds | Low | MEDIUM | reliability | open | HTTP data source documented as "optional, build-time only." Provide `--offline` flag that skips HTTP sources and uses last cached data. Warn when HTTP source is declared but --offline is set. NFR candidate: no | CAP-003 |
| R-007 | libxslt C dependency for MathML→OMML conversion breaks the single-binary goal on MUSL Linux targets | Medium | MEDIUM | reliability | open | Prototype libxslt bundling on MUSL before committing (ASM-006). Alternative: defer OMML to v1.x and use SVG fallback for PPTX math in v1.0. Security focus: no | ASM-006, CAP-012 |
| R-008 | Competitor landscape shift: Typst adds PPTX export or a major vendor adopts a Rust-native branded slide tool, reducing slideforge's differentiation window | Low | HIGH | business | open | Monitor Typst roadmap quarterly; slideforge's brand bridge, compile-time accessibility, and data-reactive DSL are not replicable features in Typst's architecture. Execute v1.0 within 12 months to establish market presence. | differentiators.md |
| R-009 | Target users (engineering teams, incident response) have less adoption momentum than anticipated; slideforge remains a niche tool | Medium | HIGH | business | open | Focus v1.0 on the incident-response and executive-reporting use cases that the seed reference already serves; seed the community with the 1898 & Co. use case; build CLI and VS Code extension in parallel for discoverability. Holdout eval ASM-001 must show mean satisfaction ≥ 0.85 before v1.0 release. | product-brief.md §1, ASM-001 |
| R-010 | DOCX output quality is too low to compete with tools like Pandoc for users who primarily need document output | Medium | MEDIUM | business | open | DOCX Level 1 in v1.0 is positioned as "structured branded report" not "flowing prose document." Marketing should set correct expectations. DOCX Level 2 (proper flowing layout) in v1.x. Validate DOCX output against 1898 report format (ASM-009). | CAP-016, ASM-009 |
| R-011 | Security: HTTP data source plugin fetches attacker-controlled URLs, potentially enabling SSRF if slideforge is run in a server-side context | Low | HIGH | security | open | Security focus: yes. Mitigation: Document that `@data` HTTP sources are ONLY for build-time compilation, never for server-side user-supplied URLs. Implement URL allowlist option (`[data.allowed_domains]` in slideforge.toml). cargo audit + cargo deny in CI catches supply chain issues. Run semgrep on HTTP data source plugin code. NFR candidate: yes — security-reviewer on every PR | CAP-003 |
| R-012 | Security: Package installation from arbitrary git URLs could install malicious .sf files or plugin WASM modules | Low | HIGH | security | open | Security focus: yes. Mitigation: Packages install .sf content only (not executable code) in v1.0 (no WASM plugins yet). sf.lock checksum verification (sha256) prevents tampering after install. Document supply chain risk. SBOM generation on release. cargo audit + cargo deny in CI. NFR candidate: yes | CAP-025 |
| R-013 | DSL syntax conflicts: the chosen syntax for inline math ($...$), variable interpolation ({{ }}), and indentation create ambiguous edge cases that confuse users | Medium | MEDIUM | reliability | open | The 14 syntax conflicts analyzed in q1-decision-final.md §9 are resolved and documented. Regression test the conflict resolution cases. Every conflict has a specific error message with resolution hint. | CAP-012, DEC-009 |
| R-014 | Snapshot test maintenance burden: 31 slide types × 4 renderers requires a large fixture set that becomes expensive to maintain | Medium | MEDIUM | reliability | open | Use insta for snapshot tests with deterministic fixture generation from the reference deck. Visual diff (LibreOffice headless screenshots) is a separate CI job that only runs on PPTX-touching changes. Architect must design snapshot strategy in Phase 1. NFR candidate: no | CAP-010, CAP-015 |
| R-015 | ooxmlsdk or another critical dependency has a license change or is abandoned post-v1.0 | Low | HIGH | business | open | All production crate deps pinned with `=` in Cargo.toml; Cargo.lock committed; SBOM generated on release. Monitor crate health quarterly. If ooxmlsdk is abandoned: maintain a fork; OOXML generation is a bounded problem with a known schema. NFR candidate: no | CAP-015 |
| R-016 | Cross-compilation to Windows x86_64 via MSVC toolchain produces different PPTX output than macOS or Linux builds | Low | MEDIUM | reliability | open | Run CI matrix across all 4 platforms (macOS arm64/x86_64, Linux x86_64, Windows x86_64). Use integer EMU and avoid any platform-dependent float operations in the pipeline. Known source of cross-platform divergence: font path resolution. | CAP-015, DI-010 |
