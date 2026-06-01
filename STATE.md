---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-05-31
phase_1_approved: 2026-05-25
phase_2_approved: 2026-05-25
phase_1_convergence: "17 passes, 69 findings, 3/3 clean (passes 15-16-17)"
phase_2_convergence: "22 passes, 96+ findings, 3/3 clean (passes 20-21-22)"
prd_bcs: 109
prd_hs: 15
prd_vps: 15
prd_supplements: 4
spikes_resolved: 7
spikes_total: 7
total_stories: 77
total_points: 462
total_waves: 6
total_epics: 21
dtu_required: false
dtu_assessment: 2026-05-24
dtu_clones_built: n/a
dtu_services: []
wave_1_gate: "PASS 2026-05-27 — 3 gate passes, 11 findings fixed"
wave_2_gate: "PASS 2026-05-27 — 11 gate passes, 19 findings fixed, 3/3 clean (passes 9-10-11)"
wave_3_gate: "PASSED 2026-05-31 — PR #38 (7d266ad7); #[non_exhaustive] hardening + slideforge-brand [workspace.dependencies] + conventions.md v1.3; adversary pass 8 strict-CLEAN; holdout must-pass 5/5"
wave_4_batch_a_complete: 3
wave_4_batch_a_total: 9
wave_4_started: 2026-05-31
wave_4_total_stories: 17
wave_4_total_points: 104
wave_4_batch_a: "STORY-035→036, STORY-043→044→045, STORY-073, STORY-075, STORY-076, STORY-077 (parallel)"
wave_4_batch_b: "STORY-037→038→039→040, STORY-041→042 (parallel, after Batch A)"
wave_4_batch_c: "STORY-049→050 (after Batch B)"
wave_4_new_p0: "STORY-077 (8pts SectionBlock IR extension) — architect-directed spin-out from STORY-035 F-002 descope; blocks STORY-041/042"
story035_status: "MERGED — PR #39, squash commit 0e7d9fde (2026-05-31). 10-pass LOCAL adversary cascade, 3/3 strict-CLEAN at passes 8/9/10. Option D single-source eval-stage routing. AC-005 descoped → STORY-077."
story036_status: "MERGED — PR #40, squash commit 094f8dca (2026-05-31). 9-pass LOCAL adversary cascade, 3/3 strict-CLEAN at passes 7/8/9. BleedChecker test utility (test-utils-gated). Exporter ACs deferred to STORY-037/041/046. 6 fix-bursts (XML entity decoder hardening)."
story043_status: "MERGED — PR #41, squash commit 331d456c (2026-05-31). 14-pass LOCAL adversary cascade, 3/3 strict-CLEAN at passes 12/13/14. New slideforge-pdf crate (krilla 0.6.0 pure-Rust PDF; moved from [workspace] exclude → members). PdfExporter + SlideTagEngine + svg_embed + font + check-pdf-deps CI job. indexmap 2.9→2.10. 2 MED + 3 LOW security findings fixed. Workspace now 16 crates."
story044_status: "IN PROGRESS — worktree .worktrees/STORY-044, branch feature/S-044, HEAD a39e068c (PUSHED to origin). 57/57 slideforge-pdf tests PASS; 2394 workspace tests pass (1 flaky timing-only in slideforge-diagrams, passes in isolation). clippy(pedantic+unwrap_used)+fmt+doc ALL clean. SVG transform fix JUST LANDED (a39e068c) — adversary re-verification required (3-CLEAN streak RESET)."
develop_sha: "331d456c"
develop_pr_count: 41
workspace_tests: "2394 (STORY-044 worktree full-suite; 1 timing-flaky skipped in isolation; 57/57 slideforge-pdf green)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## What Is This Project?

slideforge is a DATA-REACTIVE BRANDED DOCUMENT PLATFORM. Generates branded .pptx, .docx, .pdf, .html, and web preview from a single .sf DSL file with data binding, iteration, conditionals, and a plugin-first architecture.

**Repository:** https://github.com/drbothen/slideforge
**Workspace:** /Users/jmagady/Dev/slideforge
**Factory worktree:** .factory/ on branch `factory-artifacts`

## Session Resume Brief (2026-05-31 — STORY-044 MID-FLIGHT)

### Ground Truth

- **develop:** `331d456c` (41 merged PRs). Wave 4 Batch A: 3/9 MERGED (STORY-035, STORY-036, STORY-043).
- **STORY-044 worktree:** `/Users/jmagady/Dev/slideforge/.worktrees/STORY-044`
- **Branch:** `feature/S-044` — HEAD `a39e068c` — PUSHED to `origin/feature/S-044`.
- **Test status:** 57/57 slideforge-pdf GREEN. 2394 workspace tests pass (1 timing-flaky in slideforge-diagrams cold_budget — passes when run in isolation with `cargo nextest run -p slideforge-diagrams`; not a code regression).
- **Toolchain clean:** clippy(pedantic + unwrap_used) + fmt + `RUSTDOCFLAGS=-D warnings cargo doc --workspace` all clean.

### STORY-044 Status (PDF: EMU-to-PDF Coordinate Mapping + Y-Axis Flip + Content Drawing)

**MAJOR BUG FOUND AND FIXED (mid-stream):** A vertical-mirror coordinate bug was discovered. krilla 0.6.0's `Surface` uses a **top-left, Y-down** coordinate system and applies the PDF Y-flip internally via `page_root_transform` (`Transform::from_row(1,0,0,-1,0,h)` in page.rs:262-263). Draw-time placement therefore uses `emu_to_pt(ir_y)` directly — NOT `ir_y_to_pdf_y`. The wrong "exporter applies the flip" model had propagated to 5 spec artifacts; all corrected:

| Artifact | Change |
|----------|--------|
| BC-4.03.005 | v1.1 → v1.2 (coordinate model corrected; `ir_y_to_pdf_y` retained as pure VP-006 Kani target only) |
| STORY-044 spec | draw-time formulas corrected; SVG transform section updated |
| export-architecture.md | PDF section: krilla Surface model documented |
| ir-design.md | Coordinate system note for PDF exporters |
| architecture-feasibility-report.md | PDF backend coordinate model section |
| vp-006-emu-pdf-coordinate.md | AC-001/002 Kani vectors remain valid (pure function arithmetic); draw-time note added |

Architect directive: `.factory/cycles/STORY-044/coord-model-directive.md` (binding, DIR-044-001).

`ir_y_to_pdf_y` is RETAINED as a pure function in `coords.rs` — it is a VP-006 Kani proof target. It is NOT called on the draw path. AC-001/002 test vectors remain valid.

**SVG/Diagram fidelity fix JUST LANDED** (commit a39e068c, human-authorized scope expansion): `svg_embed` now applies usvg `abs_transform` per path + `place_svg_at` does fit-to-frame scaling (translate∘scale∘abs_transform). Non-vacuous geometry tests extract coords from uncompressed content stream. Closes:
- F-P18-001: transforms dropped → mermaid diagrams collapsed to origin
- F-P18-002: no scaling → oversize/clip
- F-P18-003 / F-P16-001: param naming

**AC-009 font subsetting** (re-scoped from STORY-043): driven through real `export()` with uncompressed measurement + <100KB subset bound (observed ~2.1KB). COMPLETE.

### Adversary Cascade History

18 passes run (detail in `.factory/cycles/STORY-044/cascade-log.md` if created). Cascade was converging on the pre-SVG-fix code (pass 17 was strict-CLEAN). The SVG scope-expansion fix (a39e068c) is NEW code — **3-CLEAN streak RESET to 0/3**. The SVG path (svg_embed.rs `render_path`/`abs_transform`, exporter.rs `place_svg_at` scaling) has NOT been adversarially verified yet.

### Top Next Actions (ordered — for fresh session)

1. **RESUME STORY-044 adversary cascade.** Focus on the NEW SVG transform+scaling path (commit a39e068c): `svg_embed.rs::render_path`, `svg_embed.rs::abs_transform` application, `exporter.rs::place_svg_at` scaling (translate∘scale∘abs_transform). Worktree `.worktrees/STORY-044`, HEAD `a39e068c`. Drive to BC-5.39.001 3-CLEAN (3 consecutive strict-CLEAN). Convergence is NEAR — text/coords/AC-009 already converged; only new SVG code needs adversarial verification. Use local adversary, minimum 3-clean.

2. **Demo recording** (after 3-CLEAN). Per per-story-delivery: demo-recorder, per-AC, library/test-harness modality, output to `docs/demo-evidence/STORY-044/`. Branch is already pushed.

3. **PR cycle** (after demos). pr-manager 9-step PR cycle → develop. Security review → CI → squash-merge → devops worktree cleanup (`git worktree remove .worktrees/STORY-044`). NOTE: .factory spec changes (BC v1.2 etc.) are committed in this burst — the PR itself is code-only. Ensure the post-merge state burst records develop SHA and PR count.

4. **Continue Wave 4 Batch A.** After STORY-044 merges: STORY-045 (PDF/UA-1 + veraPDF — CRITICAL forward-obligations: contiguous-run sentinels from STORY-036 + real frame-level Diagram/Chart alt from STORY-043); STORY-073 (Bullets Layout); STORY-075 (Footer Detection); STORY-076 (srgbClr Transform); STORY-077 (SectionBlock IR Extension — P0, blocks STORY-041/042).

### STORY-045 Forward-Obligations (record before context is lost)

- **From STORY-036:** contiguous-run sentinels for register content must be enforced (BleedChecker ACs 001-007 are `#[ignore]`'d pending exporters — STORY-045 implementation notes MUST address register sentinel contiguous-XML-run requirement).
- **From STORY-043:** real frame-level Diagram/Chart alt text must flow through PDF/UA-1 tagging (SlideTagEngine stub in STORY-043 uses placeholder; STORY-045 must wire real alt text from IR frames).

---

## Current Status

Phase 3 IN PROGRESS. Wave 1 COMPLETE (gate PASSED). Wave 2 COMPLETE (gate PASSED). Wave 3 COMPLETE (22/22 stories, gate PASSED 2026-05-31). **Wave 4 STARTED — 17 stories / 104 pts. Batch A 3/9 complete (STORY-035 + STORY-036 + STORY-043 MERGED). STORY-044 IN PROGRESS.**

develop: `331d456c` (41 merged PRs, 2394 tests, 0 failures). 1 active feature worktree (STORY-044). 0 open PRs. 77 stories / 462 pts total. Workspace: 16 crates.

## Phase Progress

| Phase | Status | Key Output |
|-------|--------|-----------|
| Pre-pipeline | DONE | Toolchain + LLM + MCP preflight (2026-05-23) |
| Market intelligence | DONE 2026-05-23 | GO with medium confidence |
| Planning (25 DSL decisions) | DONE 2026-05-24 | q1–q25 decision docs + 14 research threads + 7/7 spikes resolved |
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (109 BCs, 15 HS, 4 supplements) + architecture (14 ADRs, 15 VPs, 20 crates) + UX spec. 17 passes, 69 findings, 3/3 clean. |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 77 stories, 21 epics, 6 waves, 462 pts. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Wave 1: GATE PASSED. Wave 2: GATE PASSED. Wave 3: GATE PASSED 2026-05-31. **Wave 4 STARTED — Batch A 3/9 (STORY-035+036+043 MERGED). STORY-044 IN PROGRESS (SVG fix a39e068c; adversary re-verify pending).** | Per-story delivery |
| Phase 4: Holdout Evaluation | NOT STARTED | Per-wave holdout gates |
| Phase 5: Adversarial Refinement | NOT STARTED | Post-implementation cascade |
| Phase 6: Formal Hardening | NOT STARTED | Kani + fuzz + mutants + semgrep |
| Phase 7: Convergence | NOT STARTED | 7-dimension convergence assessment |

## Wave 4 Batch A Status

| Story | Title | Status | PR | Commit |
|-------|-------|--------|----|--------|
| STORY-035 | Writing Register Routing | MERGED | #39 | 0e7d9fde |
| STORY-036 | No-Bleed Invariant | MERGED | #40 | 094f8dca |
| STORY-043 | PDF Core (new crate) | MERGED | #41 | 331d456c |
| STORY-044 | PDF Layout Integration | IN PROGRESS — SVG fix landed (a39e068c); adversary re-verify pending | — | a39e068c |
| STORY-045 | PDF Export Pipeline | NOT STARTED (after 044) | — | — |
| STORY-073 | Bullets Layout | NOT STARTED | — | — |
| STORY-075 | Footer Detection | NOT STARTED | — | — |
| STORY-076 | srgbClr Transform | NOT STARTED | — | — |
| STORY-077 | SectionBlock IR Extension (P0) | NOT STARTED | — | — |

## Decisions Log (milestones)

- 2026-05-23 — Workspace resolved, mode: greenfield. Market intelligence: GO.
- 2026-05-24 — ALL 25 DSL DESIGN QUESTIONS COMPLETE. ALL 7/7 SPIKES RESOLVED. PRD COMPLETE (109 BCs). ARCHITECTURE COMPLETE (14 ADRs, 15 VPs, 20 crates). UX SPEC COMPLETE.
- 2026-05-25 — PHASE 1 CONVERGED (17 passes, 69 findings, 3/3 clean) + APPROVED. PHASE 2 CONVERGED (22 passes, 96+ findings, 3/3 clean) + APPROVED. PHASE 3 STARTED.
- 2026-05-26 — Wave 1: STORIES 001–010 + 051/052/053/054 MERGED (14 stories).
- 2026-05-27 — WAVE 1 GATE PASSED. WAVE 2: all 7 stories MERGED (PRs #14–#20). WAVE 2 GATE PASSED — 11 gate passes, 19 findings fixed, 3/3 clean.
- 2026-05-27/28 — WAVE 3 BATCH 1 COMPLETE — 6 new crates (data, brand, layout, math, charts, diagrams), 494 new tests, PRs #21–#26.
- 2026-05-28/30 — WAVE 3 BATCH 2 COMPLETE — 13 stories MERGED (PRs #27–#34, commits 7bc71f9c → 066d625f). Notable: STORY-020 29-pass adversary; STORY-028 32-pass adversary.
- 2026-05-30 — WAVE 3 BATCH 3 COMPLETE — STORY-021 (PR #35, 362c4a1f), STORY-024 (PR #36, 924cdc04), STORY-025 (PR #37, 584cbc6f) ALL MERGED.
- 2026-05-31 — WAVE 3 GATE PASSED — 2584 tests GREEN; holdout must-pass 5/5; adversary 8 passes 3-CLEAN (6/7/8); fix-PR #38 (7d266ad7). WAVE 3 COMPLETE.
- 2026-05-31 — WAVE 4 STARTED — 17 stories / 104 pts (STORY-077 added P0; STORY-073/075/076 pulled in P1; STORY-072/074 deferred Wave 5). BC-2.01.001 v1.2, BC-2.01.003 v1.9, error-taxonomy v2.3.
- 2026-05-31 — STORY-035 MERGED (PR #39, 0e7d9fde) — Writing Register Routing. 10-pass adversary cascade, 3/3 strict-CLEAN (passes 8/9/10). Option D single-source eval-stage routing. AC-005 descoped → STORY-077. Wave 4 Batch A: 1/9 complete.
- 2026-05-31 — STORY-077 CREATED — SectionBlock IR Extension (architect-directed spin-out, EPIC-18, P0, 8pts). Blocks STORY-041/042. Project total: 77 stories / 462 pts.
- 2026-05-31 — STORY-036 MERGED (PR #40, 094f8dca) — No-Bleed Invariant / BleedChecker. 9-pass adversary cascade, 3/3 strict-CLEAN (passes 7/8/9). 6 fix-bursts (XML entity decoder). Exporter ACs deferred to STORY-037/041/046. Wave 4 Batch A: 2/9 complete.
- 2026-05-31 — STORY-043 MERGED (PR #41, 331d456c) — PDF Core (slideforge-pdf crate). 14-pass adversary cascade, 3/3 strict-CLEAN (passes 12/13/14). krilla 0.6.0 pure-Rust PDF (no FFI). PdfExporter + SlideTagEngine + svg_embed + font subsetting + check-pdf-deps CI. indexmap 2.9→2.10. 2 MED + 3 LOW security fixed. Workspace 16 crates. STORY-044 unblocked. Wave 4 Batch A: 3/9 complete.
- 2026-05-31 — STORY-044 IN PROGRESS — Vertical-mirror coordinate bug found and fixed (krilla Surface is top-left Y-down; exporter MUST NOT apply ir_y_to_pdf_y at draw time — krilla applies the PDF flip internally). BC-4.03.005 bumped to v1.2. 5 spec artifacts corrected. SVG/diagram fidelity fix landed (a39e068c, human-authorized scope expansion): usvg abs_transform per-path + fit-to-frame scaling. 18 adversary passes run; SVG fix RESETS 3-CLEAN streak — re-verification pending.

## Session Resume Checkpoint

| Field | Value |
|-------|-------|
| **Date** | 2026-05-31 |
| **Position** | Phase 3, Wave 4 Batch A — 3/9 complete. STORY-044 IN PROGRESS mid-flight. SVG fix (a39e068c) just landed; adversary cascade must re-verify to 3-CLEAN before demos/PR. |
| **develop SHA** | 331d456c (41 merged PRs) |
| **STORY-044 worktree** | /Users/jmagady/Dev/slideforge/.worktrees/STORY-044 |
| **STORY-044 branch** | feature/S-044 HEAD a39e068c (pushed to origin) |
| **STORY-044 tests** | 57/57 slideforge-pdf GREEN; 2394 workspace (1 timing-flaky cold_budget, passes in isolation) |
| **STORY-044 adversary** | 18 passes; SVG fix resets streak to 0/3 — focus: svg_embed.rs render_path/abs_transform + exporter.rs place_svg_at scaling |
| **Workspace crates** | 16 (slideforge-pdf added in STORY-043) |
| **Active worktrees** | 1 (STORY-044) |
| **Open PRs** | 0 |
| **STORY-035** | MERGED — PR #39, 0e7d9fde. 10-pass adversary, 3/3 strict-CLEAN (P8/9/10). Option D routing. AC-005 → STORY-077. |
| **STORY-036** | MERGED — PR #40, 094f8dca. 9-pass adversary, 3/3 strict-CLEAN (P7/8/9). BleedChecker test-utils feature-gated. 6 fix-bursts (XML entity decoder). Exporter ACs → STORY-037/041/046. |
| **STORY-043** | MERGED — PR #41, 331d456c. 14-pass adversary, 3/3 strict-CLEAN (P12/13/14). slideforge-pdf crate (krilla 0.6.0, pure-Rust). 2 MED + 3 LOW security fixed. check-pdf-deps CI job. indexmap 2.9→2.10. |
| **BC deltas in effect** | BC-4.03.005 v1.2 (coord model corrected for krilla Surface); BC-2.01.001 v1.2 (srgbClr, Option B); BC-2.01.003 v1.9 (EC-003 widened); error-taxonomy v2.3. |
| **Cross-story caveat (S36→S37/S41)** | BleedChecker AC-001..AC-007 tests are `#[ignore]`'d pending PPTX/DOCX exporters. Register sentinels MUST be contiguous XML runs. Add to STORY-037 + STORY-041 implementation notes. |
| **Cross-story caveat (S45)** | STORY-045 must wire real frame-level Diagram/Chart alt text (SlideTagEngine stub in STORY-043). Also must enforce contiguous-run sentinels from STORY-036. |
| **factory-artifacts** | Local only. Push requires explicit human authorization per CLAUDE.md. |
| **Holdout caveat** | Full mean-satisfaction holdout (≥0.85) deferred to post-exporter waves; 9/15 scenarios blocked by missing CLI + exporters. |
| **Archived history** | Prior checkpoints → .factory/cycles/STORY-043/session-checkpoints.md |

## Per-Story Delivery Flow (reference)

1. Create worktree: `git worktree add .worktrees/STORY-NNN -b feature/S-NNN develop`
2. test-writer: stubs + failing tests (Red Gate — tests must FAIL before implementer starts)
3. implementer: TDD (make tests pass, zero `.unwrap()`, clippy::pedantic clean)
4. adversary: 3 consecutive strict-CLEAN passes (BC-5.39.001)
5. `git push origin feature/S-NNN` → PR targeting `develop` → CI (17 checks) → squash-merge → state update
6. Remove worktree: `git worktree remove .worktrees/STORY-NNN`

## Key Spec References

| Document | Scope |
|----------|-------|
| .factory/specs/behavioral-contracts/ | 109 BCs organized by section |
| .factory/specs/architecture/ARCH-INDEX.md | 12 architecture sections, 14 ADRs |
| .factory/specs/prd.md | PRD + 109 BCs + 15 holdout scenarios |
| .factory/stories/wave-schedule.md | 6 waves, batching, dependency order |
| .factory/stories/STORY-INDEX.md | 77 stories with status |
| .factory/stories/sprint-state.yaml | Current sprint/wave state |
| .factory/planning/q1-decision-final.md | Computation, formats, registers, charts, math, brand, roadmap |
| .factory/planning/q3-decision-final.md | Plugin-first architecture, 10 surfaces, trait signatures |
| .factory/cycles/STORY-044/coord-model-directive.md | Binding DIR-044-001: krilla coordinate model correction |

## Quality Bar (Non-Negotiable)

Production-grade from day 1. Key enforced gates:
- `#![forbid(unsafe_code)]` on all crates; zero `.unwrap()` outside tests; `clippy::pedantic` clean; `#![warn(missing_docs)]` on public APIs
- Kani proofs for pure-core functions (Phase 6); `cargo-fuzz` harness; `cargo-mutants` with documented kill-rate budget
- WCAG AA: web preview + HTML; PDF/UA-1 on PDF; OOXML accessibility linter on PPTX
- < 500ms cold build, < 50ms incremental (25-slide deck) — CI benchmark gate
- Signed releases + SBOM; `cargo audit` + `cargo deny` in CI; semgrep/CodeQL per PR
- Cross-platform: macOS arm64+x86_64, Linux x86_64+arm64, Windows x86_64
- Full quality bar table: CLAUDE.md

## Drift Items

| Date | Item | Severity | Notes |
|------|------|----------|-------|
| 2026-05-28 | LOCAL adversary 3-CLEAN on STORY-034 ran macOS-only, missed Linux Trebuchet MS substitution failure (BC-1.12.001 violation) | LOW | Required 4 CI iterations. Process-gap candidate: should LOCAL adversary spawn a Linux-container test pass for font/text/SVG/rendering stories? Surface to user for codification decision. |
| 2026-05-31 | BC-1.14.004 frontmatter has `subsystem: SS-TBD` + unfilled template placeholders (Architecture Module, Stories, Story Anchor, VP Anchors). May affect other BCs. | LOW | Deferred to a future spec-hygiene pass. Owner: architect / spec-steward. No story blocker — do not block Wave 4 delivery on this. |
| 2026-05-31 | BC-4.03.001 + BC-4.03.002 have unfilled `subsystem: SS-TBD` + Stories/Story-Anchor/VP-Anchor placeholders (same pattern as BC-1.14.004; subsystem SS-07 is the correct answer for both PDF BCs). | LOW | Future spec-hygiene pass. Not a story blocker. |
| 2026-05-31 | slideforge-diagrams `src/normalize.rs` usvg_normalize parses externally-sourced mermaid SVG via `usvg::Tree::from_str` WITHOUT a size/depth guard — DoS hardening gap (CWE-400), analogous to STORY-043 SEC-002 (SVG recursion cap). | LOW | Target: future slideforge-diagrams hardening story. Security-reviewer to verify when diagrams crate is next touched. |
| 2026-05-31 | CI has NO doctest job: `test` job uses `cargo nextest` (skips doctests); no `cargo test --doc` step. coords.rs added runnable doctests checked by `cargo doc` but NOT executed in CI (only local `just check` runs them). | LOW | devops-engineer follow-up to add a doctest CI step. Do not block Wave 4 delivery. |

## Process Wins (apply to future stories)

- Pre-implementation tech-validation (research-agent) + architect coordinate-model directive for new-dependency stories catches library-vs-spec coordinate bugs AND wrong crate versions (krilla API drift, zip=4.2.0 ghost in STORY-036, subsetter 0.2.3→0.2.4) BEFORE or DURING implementation. Recommend adding "validate fast-moving external crate API + version against vendored source before Red Gate" as a formal step for new-dependency stories.
- Test-utils feature-gating pattern (STORY-036 BleedChecker) avoids test-only deps leaking into production builds.
