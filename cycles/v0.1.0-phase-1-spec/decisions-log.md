# Decisions Log — Phase 1 Spec Crystallization (v0.1.0-phase-1-spec)

Archived from STATE.md on 2026-05-25 at Phase 1 APPROVED / Phase 2 start.

---

## Pre-Phase-1 Decisions

- 2026-05-23 — Workspace resolved, mode: greenfield
- 2026-05-23 — factory-artifacts branch + worktree initialized
- 2026-05-23 — Seed ingested, canonical brief at specs/product-brief.md
- 2026-05-23 — 7 Open Questions answered (slideforge / indented / @include / both+extract+full-synthesis / all-3-exporters / DSL-only Python / Typst-web-preview)
- 2026-05-23 — Production-grade-from-day-1 declared
- 2026-05-23 — Market intelligence: GO
- 2026-05-23 — Toolchain preflight: PASS-WITH-NOTES
- 2026-05-23 — Brief validated, sharded, parity contract locked
- 2026-05-24 — Q1 LOCKED (data-reactive, 5 formats, 3 registers, charts, math, plugin-first, brand bridge)
- 2026-05-24 — Q2 LOCKED (31 types, aliases, reserved components)
- 2026-05-24 — Q3 LOCKED (plugin-first, 10 surfaces, dog-food, Mermaid v1.0)
- 2026-05-24 — Q4-Q15 LOCKED (template overlay, a11y, shape DSL, interpolation, variants, fragments, etc.)
- 2026-05-24 — Q16-Q25 LOCKED (packages, workspace, defaults, keywords, errors, merge)
- 2026-05-24 — ALL 25 DSL DESIGN QUESTIONS COMPLETE
- 2026-05-24 — Reconciliation pass: all docs consistent

## Phase 1 Decisions and Adversarial Pass Log

- 2026-05-24 — PHASE 1 STARTED: devops-engineer bumped MSRV to 1.88 (clean build verified)
- 2026-05-24 — Domain research completed (34KB); 4 bounded contexts confirmed; font metric divergence = #1 multi-format risk
- 2026-05-24 — Spike S2 RESOLVED: ADOPT pdf-writer + krilla (PDF/UA-1 decisive axis)
- 2026-05-24 — Spike S3 RESOLVED: axe-core/playwright + veraPDF + custom OOXML linter
- 2026-05-24 — Spike S5 RESOLVED: 31 layouts (11 standard + 20 custom); dark: clrMapOvr + solidFill fallback
- 2026-05-24 — Spike S6 RESOLVED: SSIM≥0.99 + PSNR≥35dB dual gate; visual-parity-contract.md SSIM 0.97 threshold is stale — must be updated to ≥0.99 when architecture agent runs
- 2026-05-24 — Spike S14 RESOLVED: ADOPT mermaid-rs-renderer v0.2.2 (65µs–3ms/diagram, pure SVG, 8/8 types)
- 2026-05-24 — L2 Domain Spec COMPLETE (12 files; 30 CAPs, 22 DIs, 20 DECs, 14 ASMs, 16 Rs, 18 FMs)
- 2026-05-24 — Spike S1 RESOLVED: ADOPT-WITH-WORKAROUNDS — ooxmlsdk 0.6.1; 55/57 PASS; 2 workarounds identified
- 2026-05-24 — Spike S4 RESOLVED: VIABLE-WITH-CAVEATS — hybrid hand-written lexer + chumsky 0.10 via Stream
- 2026-05-24 — PRD COMPLETE: 101 BCs (BC-1.01 through BC-5.05), 15 holdout scenarios, 4 supplements (error-taxonomy, interface-definitions, nfr-catalog, test-vectors)
- 2026-05-24 — ADVERSARIAL PASS 1 COMPLETE: 17 findings (4C, 6H, 7M) — all resolved. 95 architecture anchors fixed; 7 new BCs added (BC-1.03.006, BC-1.03.007, BC-5.03.004–006, BC-5.06.001–002); BC total → 109 (71 P0, 38 P1). Six-stage pipeline formalized; 20-crate workspace confirmed; Exporter trait timing fields + flag composition; E-BRD-005/E-CFG-007/E-CFG-008 added; E-CFG-003 retired; HS-012 + SCR-001 + ADR-007 updated; CAP-017 Chrome → pdf-writer. Streak: 0/3 — Pass 2 dispatching.
- 2026-05-24 — ALL 7/7 SPIKES RESOLVED: S1 S2 S3 S4 S5 S6 S14 — P1-03b (architecture) is unblocked
- 2026-05-24 — READY FOR ARCHITECTURE FEASIBILITY REVIEW: P1-03b + P1-05 can run in parallel
- 2026-05-24 — P1-05 ARCHITECTURE FEASIBILITY: PASS-WITH-NOTES — 5 notes identified and fully addressed; BC-1.03.005 added for mermaid-rs-renderer pinning
- 2026-05-24 — P1-03b ARCHITECTURE COMPLETE: ARCH-INDEX + 12 section files + 14 ADRs (ADR-001–ADR-014) + module-criticality.md + feasibility-report
- 2026-05-24 — P1-06 UX SPEC COMPLETE: UX-INDEX + 10 screens (SCR-001–SCR-010) + 5 flows (FLOW-001–FLOW-005); 7 VPs (VP-001–VP-008) + VP-INDEX
- 2026-05-24 — BC-1.03.005 ADDED: mermaid-rs-renderer version-pinning contract (feasibility note 5)
- 2026-05-24 — P1-07 DTU ASSESSMENT COMPLETE: DTU_REQUIRED=false — all external services (mermaid-rs-renderer, LibreOffice headless, axe-core) are pure crate deps or test tools; no clone infrastructure required
- 2026-05-24 — P1-07 GENE TRANSFUSION COMPLETE: behavioral-only — 75 behaviors classified from Python reference; 0 code ported; Rust implementation starts from scratch with spec as truth
- 2026-05-24 — P1-07 CI/CD SETUP COMPLETE: 3 workflows (ci.yml expanded, release.yml new, security.yml new); 5-platform matrix (macOS arm64+x86_64, Linux x86_64+musl, Windows x86_64); cargo-deny, nextest, criterion gate, visual-diff.py, SBOM stub — all committed to main
- 2026-05-24 — ALL PRE-ADVERSARIAL STEPS COMPLETE: STATUS → READY_FOR_ADVERSARIAL_REVIEW
- 2026-05-24 — ADVERSARIAL PASS 2 COMPLETE: 12 findings (1C, 4H, 6M, 1L) — all resolved. Phantom anchors eliminated across 80+ BC files; trait signatures reconciled in plugin-architecture.md + error-architecture.md; ARCH-INDEX updated; L2-INDEX.md, error-taxonomy, nfr-catalog, test-vectors corrected; HS-003 + HS-015 fixed. Finding trajectory: 17→12. Streak: 0/3 — Pass 3 dispatching.
- 2026-05-24 — ADVERSARIAL PASS 3 COMPLETE: 10 findings (0C, 3H, 5M, 2L) — all resolved. Exit code standardized (BC-1.07.001); HS-004/005/006/007/012/015 field names + error codes corrected; ADR-004 error code prefixes fixed; ADR-007 DSL syntax + placeholder error code updated; VP-INDEX VP-005 trace corrected; architecture-feasibility-report BC count updated; test-vectors TV-1.3 message format fixed. Finding trajectory: 17→12→10 (zero CRITICALs). Streak: 0/3 — Pass 4 dispatching.
- 2026-05-25 — ADVERSARIAL PASS 4 COMPLETE: 6 findings (0C, 1H, 5M) — all resolved. NFR table IDs reconciled (prd.md); BC-2.01.004 brand field names corrected; HS-006 related_bcs updated; E-BRD-005 retired (error-taxonomy.md + brand-architecture.md); DEC-016 field names fixed (edge-cases.md); ASM-007 invalidated (assumptions.md); R-003 mitigated (risks.md); crate-architecture.md 12→13 arithmetic corrected. Finding trajectory: 17→12→10→6 (accelerating, zero CRITICALs). Total fixed across 4 passes: 45. Streak: 0/3 — Pass 5 dispatching.
- 2026-05-25 — ADVERSARIAL PASS 5 COMPLETE: 2 findings (0C, 2H) — all resolved. BC-1.03.004 module anchor fixed; VP-002 description corrected in VP-INDEX.md + verification-architecture.md + verification-coverage-matrix.md. Finding trajectory: 17→12→10→6→2 (strong convergence, zero CRITICALs). Total fixed across 5 passes: 47. Streak: 0/3 — Pass 6 dispatching.
- 2026-05-25 — ADVERSARIAL PASS 6 COMPLETE: 2 findings (0M, 1L) — both resolved. Single root cause: VP-002 propagation gap (vp-002-alt-enforcement-parse.md 5 field updates + verification-architecture.md 1 bullet fix). Finding trajectory: 17→12→10→6→2→2. Total fixed across 6 passes: 49. Streak: 0/3 — Pass 7 dispatching.
- 2026-05-25 — ADVERSARIAL PASS 7 CLEAN: ZERO findings. CLEAN (strict): yes. CLEAN (PR-merge): yes. Finding trajectory: 17→12→10→6→2→2→0. Total fixed across 6 non-clean passes: 49. Streak: 1/3 — Pass 8 dispatching.
- 2026-05-25 — ADVERSARIAL PASS 8 COMPLETE: 2 findings (2M) — both resolved + 5 preventive sweep fixes. BC-3.03.004 frontmatter typo (`verfication_priority` → `verification_priority`); BC-3.03.004, BC-3.01.001, BC-3.01.002, BC-5.01.001–004 module corrected (`slideforge-eval` → `slideforge-validate`). Finding trajectory: 17→12→10→6→2→2→0→2. Total edits: 56. STREAK RESET: 0/3 — Pass 9 dispatching.
- 2026-05-25 — ADVERSARIAL PASS 9 COMPLETE: 1 finding (1M) — resolved. BC-4.01.004 BC-INDEX.md title corrected (`StructTreeRoot tag sequence` → `Text contrast ratio minimum (WCAG 1.4.3)`). Finding trajectory: 17→12→10→6→2→2→0→2→1. Total fixed: 52. STREAK RESET: 0/3 — Pass 10 dispatching.
- 2026-05-25 — ADVERSARIAL PASS 10 COMPLETE: 3 findings (3M) — all resolved. VP-002 moved to module-criticality.md purity table; slideforge-html classified and added to module-criticality.md, purity-boundary-map.md, and crate-architecture.md; PRD error table deferred to error-taxonomy supplement (drift-proof pattern, consistent with NFR deferral). Finding trajectory: 17→12→10→6→2→2→0→2→1→3. Total fixed: 55. STREAK RESET: 0/3 — Pass 11 dispatching.
- 2026-05-25 — ADVERSARIAL PASS 11 COMPLETE: 3 findings (1H, 2M) — all resolved. VP-014/VP-015 fuzz sections added to verification-architecture.md (all 15 VPs now confirmed); BrandValidator row removed from purity-boundary-map.md (all 20 crates confirmed); PRD error count corrected 42→53 in prd.md. Finding trajectory: 17→12→10→6→2→2→0→2→1→3→3. Total fixed: 58. STREAK RESET: 0/3 — Pass 12 dispatching.
- 2026-05-25 — ADVERSARIAL PASS 12 COMPLETE: 1 finding (1H) — resolved. VP-007 BC trace corrected (mis-anchor to contrast formula BC instead of contrast BCs); VP-007 file and VP-INDEX BC traceability column updated. Finding trajectory: 17→12→10→6→2→2→0→2→1→3→3→1. Total fixed: 59. STREAK RESET: 0/3 — Pass 13 dispatching.
- 2026-05-25 — ADVERSARIAL PASS 13 COMPLETE: 3 findings (3M) — all resolved. FM-011 rewritten for pdf-writer/krilla (Chrome reference removed from failure-modes.md); pipeline stage order corrected (Brand→Validate) in system-overview.md; HS must-pass count updated 10→11 + threshold 6→7 in HS-INDEX.md; timing field order corrected in interface-definitions.md. Finding trajectory: 17→12→10→6→2→2→0→2→1→3→3→1→3. Total fixed: 62. STREAK RESET: 0/3 — consistency-validator sweep next, then Pass 14.
- 2026-05-25 — P1-10 CONSISTENCY-VALIDATOR SWEEP COMPLETE: 6 findings (2H, 2M, 2L) — all fixed. Pipeline order corrected in 6 docs (prd.md, L2-INDEX.md, events.md, nfr-catalog.md, ARCH-INDEX.md, architecture-feasibility-report.md); L2-INDEX P0/P1 counts corrected (P0=19, P1=11 non-contiguous); VP-005 integer-arithmetic-overflow.md created; VP titles aligned with VP-INDEX for VP-004/VP-006/VP-007/VP-008; VP-002 validate→parse terminology corrected; CAP-026 init step added in capabilities.md. Total fixed: 68 (62 adversarial + 6 CV). STATUS → ADVERSARIAL_PASS_14_PENDING.
- 2026-05-25 — ADVERSARIAL PASS 14 COMPLETE: 1 finding (1H) — resolved. SS-03/SS-04 subsystem labels swapped in system-overview.md pipeline diagram. Finding trajectory: 17→12→10→6→2→2→0→2→1→3→3→1→3→1. Total fixed: 69 (63 adversarial + 6 CV). STREAK RESET: 0/3 — Pass 15 dispatching.
- 2026-05-25 — ADVERSARIAL PASS 15 CLEAN: ZERO findings. CLEAN (strict): yes. CLEAN (PR-merge): yes. Finding trajectory: 17→12→10→6→2→2→0→2→1→3→3→1→3→1→0. Total fixed: 69 (63 adversarial + 6 CV). STREAK: 1/3 — Pass 16 dispatching.
- 2026-05-25 — ADVERSARIAL PASS 16 CLEAN: ZERO findings. CLEAN (strict): yes. CLEAN (PR-merge): yes. Finding trajectory: 17→12→10→6→2→2→0→2→1→3→3→1→3→1→0→0. Total fixed: 69 (63 adversarial + 6 CV). STREAK: 2/3 — Pass 17 dispatching (FINAL PASS — if clean → CONVERGED).
- 2026-05-25 — ADVERSARIAL PASS 17 CLEAN: ZERO findings. CLEAN (strict): yes. CLEAN (PR-merge): yes. Finding trajectory: 17→12→10→6→2→2→0→2→1→3→3→1→3→1→0→0→0. Total fixed: 69 (63 adversarial + 6 CV). STREAK: 3/3 — CONVERGED per BC-5.39.001. P1-09 DONE. STATUS → PHASE_1_HUMAN_APPROVAL_PENDING.
- 2026-05-25 — PHASE 1 APPROVED by human. Phase 2 (Story Decomposition) authorized to begin.
