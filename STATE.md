---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-1-spec-crystallization-pending-preflight
status: READY_FOR_PHASE_1
last_updated: 2026-05-23
---

# Slideforge Factory State

## Pipeline Mode
**Greenfield** — new Rust + DSL PowerPoint generator project. Scaffolding pre-applied (Cargo workspace + 7 crate stubs already committed on main @ e44aacd).

## Workspace
- Repo: `/Users/jmagady/Dev/slideforge`
- Factory worktree: `.factory/` on branch `factory-artifacts`
- Brief: `.factory/specs/product-brief.md` (canonical) — sourced from `.factory/seed/PROJECT-SEED.md`
- Reference materials: `.factory/seed/reference/` (Python implementation — IMMUTABLE source-of-truth for visual behavior)

## Phase Status

| Phase | Status | Gate |
|-------|--------|------|
| Pre-Pipeline (toolchain preflight) | PENDING | — |
| Market Intelligence | COMPLETE — GO | Human reviews GO/CAUTION/STOP |
| Phase 0 (Codebase Ingestion) | N/A — scaffolding is empty stubs, brief is canonical | — |
| Phase 1 (Spec Crystallization) | READY — pending human GO approval on market intel | Awaiting human confirmation of GO verdict |
| Phase 2 (Story Decomposition) | PENDING | — |
| Phase 3 (TDD Implementation) | PENDING | — |
| Phase 4 (Holdout Eval) | PENDING | — |
| Phase 5 (Adversarial) | PENDING | — |
| Phase 6 (Hardening) | PENDING | — |
| Phase 7 (Convergence) | PENDING | — |

## Open Questions (BLOCKING Phase 1)
RESOLVED 2026-05-23 — see Decisions Log below for the 7 answers.

## Open Architecture Questions (for Phase 1 architect)

The following ADRs must be authored and decided during Phase 1 spec crystallization:

- **ADR-001 (proposed):** Brand synthesis approach — full-from-scratch OOXML generation; document layout taxonomy that maps 23 slide types to slideLayoutN.xml files; placeholder positioning model; theme XML generation strategy
- **ADR-002 (proposed):** Multi-renderer snapshot test matrix — PowerPoint + Keynote + Google Slides + LibreOffice headless rendering parity guarantees
- **ADR-003 (proposed):** PDF backend choice (Typst-as-backend mirroring office2pdf, OR direct PDF via printpdf/lopdf, OR HTML→PDF via headless browser)
- **ADR-004 (proposed):** @include resolution semantics — relative vs. absolute paths, cycle detection, source-span propagation across files
- **ADR-005 (proposed):** Web-preview architecture — embedded server (axum?), websocket protocol, canvas renderer choice, hot-reload semantics
- **ADR-006 (proposed):** IR shape — does the same IR feed PPTX + PDF + HTML + canvas, or do we have format-specific lowering passes?
- **ADR-007 (proposed):** MSRV policy and ooxmlsdk integration — covers the 1.85→1.88 bump and version-pinning policy. HIGH severity per preflight. Must be decided before any crate implementing ooxmlsdk is started.
- **ADR-008 (proposed):** Web-preview canvas renderer architecture — is the canvas renderer the HTML exporter's output piped to a browser canvas, a separate IR-to-canvas renderer, or an SVG-intermediate approach? Distinct from ADR-005. Determines whether slideforge-html must be dual-use (static export + live preview).
- **ADR-009 (proposed):** Error recovery semantics — what does the parser do when it sees a malformed slide? Partial AST? Error continuation? Recovery hints? What IR is produced on partial parse, and is layout/export attempted?
- **ADR-010 (proposed):** IR stability and exporter plugin contract — is the IR a stable internal contract that 3rd-party exporters can target? Is there a first-class plugin interface? Is `slideforge-ffi` the intended external exporter surface?
- **ADR-011 (proposed):** WCAG AA tooling choice — pa11y vs. playwright+axe vs. lighthouse CLI. Must be decided before web-preview story is decomposed; affects CI design and HTML exporter output requirements.

## Quality Bar (Non-Negotiable)

Declared 2026-05-23. v1.0 release is gated on ALL rows below — no "ship and polish later" tolerated.

| Dimension | Day-1 Gate |
|-----------|-----------|
| Spec convergence | 3 clean adversarial passes on PRD + architecture before Phase 2 starts |
| Tests | Every public API has unit tests; snapshot tests per slide type; integration tests for CLI; fuzz harness in CI |
| Implementation | `#![forbid(unsafe_code)]` (except FFI if added); zero `.unwrap()` outside tests; `clippy::pedantic` clean; `#![warn(missing_docs)]` enforced on public APIs |
| Verification | Kani proofs for pure-core functions in `slideforge-syntax` and `slideforge-eval`; `cargo-fuzz` harness; `cargo-mutants` mutation testing in CI with documented score budget |
| Visual parity | Snapshot tests against rendered XML; CI renders sample decks in headless LibreOffice + screenshots; visual diff against fixtures |
| Performance | < 500ms cold build for 25-slide deck enforced in CI as a benchmark gate (criterion + bench regression check); incremental rebuild < 50ms |
| Documentation | rustdoc on every public item; published to docs.rs on release; user-facing DSL reference book; every ADR signed off |
| Security | `cargo audit` + `cargo deny` in CI; signed release artifacts; SBOM generation per release; semgrep or CodeQL scan per PR; security-reviewer agent on every PR |
| Supply chain | All production-crate deps pinned with `=`; `Cargo.lock` committed; `rust-toolchain.toml` pinned; reproducible builds verified |
| Multi-platform | macOS arm64+x86_64, Linux x86_64+arm64, Windows x86_64 binaries from v1.0; cross-platform CI matrix |
| Multi-renderer parity | Synthesized .pptx must render correctly in PowerPoint (Office), Keynote, Google Slides, LibreOffice — verified via automated rendering + visual diff in CI |
| Observability | `tracing` instrumentation throughout the pipeline; structured logs; opentelemetry-compatible export hooks |
| Accessibility | Web preview (Q7) audited against WCAG AA via accessibility-auditor on every PR touching the preview |
| Convergence gate | Full 7-dimension convergence check (spec/tests/impl/verify/visual/perf/docs) before release |
| Holdout eval | Mean satisfaction >= 0.85, must-pass >= 0.6 (factory default — non-negotiable for v1.0) |

### Implications
1. **No "ship it, polish later" PRs.** Every merge to default branch goes through full per-story-delivery flow with adversarial review, security review, and demo evidence.
2. **Phase 6 formal hardening is non-optional** for v1.0 — Kani + fuzz + mutation testing must all green-light.
3. **CI/CD matrix is built in Phase 1**, before any feature stories start. dx-engineer + devops-engineer expand `.github/workflows/ci.yml` into a full matrix (clippy + fmt + test + bench + audit + deny + mutants + fuzz smoke + cross-platform build + LibreOffice render-test + accessibility) as part of phase-1-cicd-setup.
4. **Timeline expectation:** v1.0 takes real engineering time. The factory executes rigorously, not fast.

## Decisions Log
- 2026-05-23 — Workspace resolved to `/Users/jmagady/Dev/slideforge`
- 2026-05-23 — Mode: greenfield (scaffolding pre-applied counts as Phase 0 stub)
- 2026-05-23 — `factory-artifacts` orphan branch + worktree initialized (commit 562ccab)
- 2026-05-23 — Seed bundle relocated from `main:seed/` to `factory-artifacts:.factory/seed/`
- 2026-05-23 — Canonical brief established at `.factory/specs/product-brief.md`
- 2026-05-23 — **Q1 — Project name:** `slideforge` (confirmed; matches scaffolding crate names crates/slideforge*; no rename required)
- 2026-05-23 — **Q2 — DSL syntax style:** Indentation-significant (YAML/Python-like). chumsky semantic-indentation parser. NOT brace-delimited.
- 2026-05-23 — **Q3 — Multi-file project support:** `@include "path.sf"` directives supported. Affects parser (source-span tracking across files), eval (resolution + cycle detection), and project config (`slideforge.toml`).
- 2026-05-23 — **Q4 — Brand template format:** BIDIRECTIONAL BRIDGE in v1.0. Both `.pptx` and `.toml` accepted as input. Plus a new CLI command `slideforge extract-brand <template.pptx> -o brand.toml` that scans an existing `.pptx` and emits a deterministic `.toml` manifest. Plus FULL SYNTHESIS in v1.0 — given only a `.toml` (no base `.pptx`), slideforge generates a complete valid `.pptx` brand scaffold from scratch (theme XML, slide master, all 11 slide layouts, notes master, handout master, relationships, content types, embedded logo media). This is a major scope expansion vs. the seed's "load template" approach.
- 2026-05-23 — **Q5 — PDF/HTML exporters scope:** All three exporters (PPTX + PDF + HTML) ship in v1.0. NOT deferred to v1.x. PDF backend choice (Typst-as-backend vs. direct printpdf/lopdf) is an OPEN ADR for the architect (ADR-003).
- 2026-05-23 — **Q6 — Python binding API style:** DSL-only. Python integration means Python builds `.sf` strings and shells out to the `slideforge` CLI binary; NO pyo3 dict-based API in v1.0. pyo3 deferred indefinitely unless explicit user demand surfaces.
- 2026-05-23 — **Q7 — Live preview architecture:** Typst-style web preview. `slideforge watch` runs an embedded web server with websocket reload and a canvas-based renderer. Whether the canvas renderer shares code with the HTML exporter or stands alone is OPEN — see ADR-008 (architect to decide in Phase 1). NOT just-rebuild-pptx. NOT browser-tab-HTML-only. NOT defer.
- 2026-05-23 — **SCOPE EXPANSION NOTE:** The 7 answers represent a ~2× scope expansion vs. seed Section 6 Phase 2-4 estimates. Specifically: full brand synthesis (Q4) is roughly equivalent in size to "all 23 slide types"; PDF+HTML+web-preview in v1.0 (Q5, Q7) adds another major chunk. The seed's phased plan needs to be re-scoped by the product-owner during Phase 1 PRD work — do NOT just transcribe seed §6 verbatim into the PRD.
- 2026-05-23 — Production-grade-from-day-1 declared. All VSDD Phase 6 formal hardening gates are non-negotiable for v1.0. CI/CD matrix built in Phase 1 before feature stories begin. v1.0 must meet full 7-dimension convergence.

## Market Intel Gate
```yaml
market_intel:
  recommendation: GO
  confidence: medium
  assessed_at: 2026-05-23
  assessor: business-analyst
  artifact: .factory/planning/market-intel-2026-05-23.md
  commit: bcabaa8
  human_decision: pending
  notes: ""
```

## Drift Items
_(None yet)_

## Next Action
Pre-Phase-1 planning burst complete (2026-05-23). Brief sharded into 9 companion files. Visual parity contract locked. ADR list expanded to ADR-001..ADR-011. Spikes register created (S1..S13). Q7 "likely" wording removed; ADR-008 opened. Human approval of market intel GO verdict still required. After approval: enter phase-1-spec-crystallization with CI/CD matrix expansion as the first sub-step. Phase 1 architect must close S1 (ooxmlsdk coverage), S5 (brand synthesis layout taxonomy), S6 (multi-renderer parity baseline), and ADR-007 (MSRV policy) before implementation stories begin.
