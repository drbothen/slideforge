---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-01
story_078_created: 2026-06-01
story_079_created: 2026-06-01
story_080_created: 2026-06-01
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
total_stories: 80
total_points: 473
total_waves: 6
total_epics: 21
dtu_required: false
dtu_assessment: 2026-05-24
dtu_clones_built: n/a
dtu_services: []
wave_1_gate: "PASS 2026-05-27 — 3 gate passes, 11 findings fixed"
wave_2_gate: "PASS 2026-05-27 — 11 gate passes, 19 findings fixed, 3/3 clean (passes 9-10-11)"
wave_3_gate: "PASSED 2026-05-31 — PR #38 (7d266ad7); #[non_exhaustive] hardening + slideforge-brand [workspace.dependencies] + conventions.md v1.3; adversary pass 8 strict-CLEAN; holdout must-pass 5/5"
wave_4_batch_a_complete: 4
wave_4_batch_a_total: 10
wave_4_started: 2026-05-31
wave_4_total_stories: 18
wave_4_total_points: 109
wave_4_batch_a: "STORY-035→036, STORY-043→044→045, STORY-073, STORY-075, STORY-076, STORY-078→STORY-077 (parallel; STORY-077 blocked on STORY-078)"
wave_4_batch_b: "STORY-037→038→039→040, STORY-041→042 (parallel, after Batch A)"
wave_4_batch_c: "STORY-049→050 (after Batch B)"
wave_4_new_p0: "STORY-077 (8pts SectionBlock IR extension) — architect-directed spin-out from STORY-035 F-002 descope; blocks STORY-041/042; BLOCKED on STORY-078 (parser gap). STORY-078 (5pts Parser section-block syntax) — new Wave 4 Batch A story resolving STORY-027 decomposition gap per DIR-077-001 2026-06-01."
story035_status: "MERGED — PR #39, squash commit 0e7d9fde (2026-05-31). 10-pass LOCAL adversary cascade, 3/3 strict-CLEAN at passes 8/9/10. Option D single-source eval-stage routing. AC-005 descoped → STORY-077."
story036_status: "MERGED — PR #40, squash commit 094f8dca (2026-05-31). 9-pass LOCAL adversary cascade, 3/3 strict-CLEAN at passes 7/8/9. BleedChecker test utility (test-utils-gated). Exporter ACs deferred to STORY-037/041/046. 6 fix-bursts (XML entity decoder hardening)."
story043_status: "MERGED — PR #41, squash commit 331d456c (2026-05-31). 14-pass LOCAL adversary cascade, 3/3 strict-CLEAN at passes 12/13/14. New slideforge-pdf crate (krilla 0.6.0 pure-Rust PDF; moved from [workspace] exclude → members). PdfExporter + SlideTagEngine + svg_embed + font + check-pdf-deps CI job. indexmap 2.9→2.10. 2 MED + 3 LOW security findings fixed. Workspace now 16 crates."
story044_status: "MERGED — PR #42, squash 94f74402 (2026-05-31). Post-SVG-fix re-verification: 27-pass LOCAL adversary cascade total, 3/3 strict-CLEAN at passes 25/26/27 (BC-5.39.001). CI 18/18 green (1 flaky Windows slideforge-data test, passed on re-run)."
develop_sha: "a4ddae8d"
develop_pr_count: 43
workspace_tests: "2443 (CI Windows) + 66 doctests (ubuntu, PR #43); 62/62 slideforge-pdf green"
workspace_test_failures: 0
---

# Slideforge — Factory State

## What Is This Project?

slideforge is a DATA-REACTIVE BRANDED DOCUMENT PLATFORM. Generates branded .pptx, .docx, .pdf, .html, and web preview from a single .sf DSL file with data binding, iteration, conditionals, and a plugin-first architecture.

**Repository:** https://github.com/drbothen/slideforge
**Workspace:** /Users/jmagady/Dev/slideforge
**Factory worktree:** .factory/ on branch `factory-artifacts`

## Session Resume Checkpoint

| Field | Value |
|-------|-------|
| **Date** | 2026-06-01 |
| **Position** | Phase 3, Wave 4 Batch A — 4/10 complete. STORY-044 MERGED. STORY-077 BLOCKED on STORY-078. Next: deliver STORY-078 (parser, 5pts, P0, unblocks STORY-077), then continue Batch A (STORY-045, 073, 075, 076), then STORY-077 rework. STORY-079/080 in Wave 5. |
| **develop SHA** | a4ddae8d (43 merged PRs) — PR #43 DI-5 doctest CI job |
| **Active worktrees** | 0 |
| **Open PRs** | 0 |
| **Workspace crates** | 16 |
| **Workspace tests** | 2443 (CI Windows) + 66 doctests (ubuntu, PR #43); 62/62 slideforge-pdf |
| **STORY-035** | MERGED — PR #39, 0e7d9fde. 10-pass adversary, 3/3 strict-CLEAN (P8/9/10). Option D routing. AC-005 → STORY-077. |
| **STORY-036** | MERGED — PR #40, 094f8dca. 9-pass adversary, 3/3 strict-CLEAN (P7/8/9). BleedChecker test-utils feature-gated. Exporter ACs → STORY-037/041/046. |
| **STORY-043** | MERGED — PR #41, 331d456c. 14-pass adversary, 3/3 strict-CLEAN (P12/13/14). slideforge-pdf crate (krilla 0.6.0, pure-Rust). 2 MED + 3 LOW security fixed. check-pdf-deps CI job. |
| **STORY-044** | MERGED — PR #42, 94f74402. 27-pass adversary, 3/3 strict-CLEAN (P25/26/27). SVG stroke rendering + paint-state-leak fix. CI 18/18 green. |
| **STORY-077** | BLOCKED on STORY-078 (parser cannot parse section <type>: blocks; DIR-077-001 2026-06-01). branch feature/S-077 retained, no active worktree. |
| **STORY-078** | NEXT TO DELIVER — Parser section-block syntax (5pts, P0, EPIC-02, Wave 4 Batch A). Delivers section_block_parser combinator + un-reserves 'section' keyword. Unblocks STORY-077. |
| **STORY-045 after 078** | PDF/UA-1 + veraPDF. Forward-obligations: (1) decorative frame Artifact tags + drawing in same change; (2) per-block/per-line baselines; (3) text color from brand/theme palette in same change as bg fills. Plus S036 register sentinels + S043 frame-level Diagram/Chart alt text. |
| **BC deltas in effect** | BC-4.03.005 v1.2 (coord model); BC-2.01.001 v1.2 (srgbClr); BC-2.01.003 v1.9 (EC-003); error-taxonomy v2.3. DI-2/DI-3 hygiene: BC-1.14.004 v1.2 (SS-02), BC-4.03.001 v1.2 (SS-07), BC-4.03.002 v1.2 (SS-07). |
| **factory-artifacts** | Local only. Push requires explicit human authorization per CLAUDE.md. |
| **Archived history** | Prior checkpoints → .factory/cycles/STORY-044/session-checkpoints.md |

## STORY-078 Kickoff Brief (cold-start)

**Goal:** Deliver STORY-078 (Parser: section block syntax, 5pts, P0, Wave 4 Batch A, EPIC-02, crate slideforge-syntax/SS-01, traces BC-3.02.002). Unblocks STORY-077.

**Authoritative artifacts (read these first):**
- Spec: `.factory/stories/stories/STORY-078-parser-section-block-syntax.md`
- Binding directive: `.factory/cycles/STORY-077/section-parse-directive.md` (DIR-077-001 + Addendum DIR-077-001-A — addendum is CURRENT ruling)
- Research: `.factory/cycles/STORY-077/chumsky-parse-warning-research.md`
- BC: `.factory/specs/behavioral-contracts/BC-3.02.002.md` (v1.2 — NO amendment; invariant 4 "parse time" is CORRECT)

**Must-honor technical decisions (DIR-077-001-A):**
1. Un-reserve `section` — `crates/slideforge-syntax/src/keywords.rs:74` maps `"section"` → E-PAR-006. Make it a recognized block introducer. `SectionNode` (ast.rs:303-314) is a placeholder — make it real (`kind: Spanned<String>`, `fields: Vec<FieldNode>`).
2. Reuse existing `template_value()` inline combinator for `detail:`/`report:` sub-block values — SINGLE inline parser. Preserve Bold/Xref/`{{ }}`/math as structured inline nodes.
3. Section TYPE stored VERBATIM in `SectionNode.kind`; parser does NOT validate against a type list and emits NO error for any type name. TYPE validation → eval stage (STORY-077, not STORY-078).
4. Unrecognized sub-block KEY → parse-time WARNING via chumsky 0.10.1 `validate()` + `Emitter::emit(Rich::custom(span,msg))` routed through `push_with_severity(ParseSeverity::Warning)` → `ParseResult::warnings`. Parse SUCCEEDS. Reserved-name collision (EC-006) → parse-time FATAL with hint.
5. `REGISTER_SUB_BLOCK_KEYS = ["report","detail"]` — `notes` is INVALID on sections (triggers unrecognized-key warning).

**Delivery flow:** worktree from origin/develop (a4ddae8d): `git worktree add .worktrees/STORY-078 -b feature/S-078 origin/develop` → test-writer Red Gate → implementer TDD → LOCAL adversary BC-5.39.001 3 consecutive strict-CLEAN → demo-recorder per-AC → push → pr-manager 9-step PR → squash-merge → worktree cleanup → post-merge state burst.

**STORY-077 disposition:** BLOCKED on 078. Old branch `feature/S-077` (2bf17d25) has defective eval work (flatten-to-Str, vacuous tests) — DISCARD and redo after 078 merges. IR type change (`SectionBlock.body: OrderedMap<Arc<str>, FieldValue>` + `register_content` field) and ~32 layout call-site adjustments are reusable references only.

## Current Status

Phase 3 IN PROGRESS. Wave 1 COMPLETE (gate PASSED). Wave 2 COMPLETE (gate PASSED). Wave 3 COMPLETE (22/22 stories, gate PASSED 2026-05-31). **Wave 4 STARTED — 18 stories / 109 pts. Batch A 4/10 complete (STORY-035, 036, 043, 044 MERGED). STORY-077 BLOCKED on STORY-078 (new, parser prerequisite). 0 active worktrees. 0 open PRs.**

develop: `a4ddae8d` (43 merged PRs, 2443 tests + 66 doctests, 0 failures). 80 stories / 473 pts total. Workspace: 16 crates.

## Phase Progress

| Phase | Status | Key Output |
|-------|--------|-----------|
| Pre-pipeline | DONE | Toolchain + LLM + MCP preflight (2026-05-23) |
| Market intelligence | DONE 2026-05-23 | GO with medium confidence |
| Planning (25 DSL decisions) | DONE 2026-05-24 | q1–q25 decision docs + 14 research threads + 7/7 spikes resolved |
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (109 BCs, 15 HS, 4 supplements) + architecture (14 ADRs, 15 VPs, 20 crates) + UX spec. 17 passes, 69 findings, 3/3 clean. |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 80 stories (77 original + 3 added 2026-06-01), 21 epics, 6 waves, 473 pts. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Wave 1: GATE PASSED. Wave 2: GATE PASSED. Wave 3: GATE PASSED 2026-05-31. **Wave 4 STARTED — Batch A 4/10 (STORY-035+036+043+044 MERGED; STORY-077 blocked on STORY-078). Next: STORY-078.** | Per-story delivery |
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
| STORY-044 | PDF Layout Integration | MERGED | #42 | 94f74402 |
| STORY-078 | Parser: section block syntax (P0) | NEXT — unblocks STORY-077 | — | — |
| STORY-045 | PDF Export Pipeline | NOT STARTED (after 044) | — | — |
| STORY-073 | Bullets Layout | NOT STARTED | — | — |
| STORY-075 | Footer Detection | NOT STARTED | — | — |
| STORY-076 | srgbClr Transform | NOT STARTED | — | — |
| STORY-077 | SectionBlock IR Extension (P0) | BLOCKED on STORY-078 | — | — |

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
- 2026-05-31 — STORY-044 MERGED (PR #42, 94f74402) — PDF coordinate mapping + Y-axis + SVG/diagram fidelity. 27-pass adversary cascade, 3/3 strict-CLEAN (P25/26/27). Real SVG stroke rendering (usvg→krilla full translation) + paint-state-leak fix (explicit fill/clear stroke before draw_text). 3 STORY-045 forward-obligations recorded; 1 Windows flaky-test drift item recorded. Wave 4 Batch A: 4/9 complete.
- 2026-06-01 — STORY-077 spec-readiness closed — BC-3.02.002 v1.2 + BC-1.14.003 v1.2 (section-level detail/report register routing + inline-structure preservation + unrecognized-sub-block-key warning clauses); STORY-077 status→ready; entering per-story delivery.
- 2026-06-01 — STORY-077 BLOCKED: prerequisite parser gap discovered — STORY-027 decomposition gap (parser cannot parse `section <type>:` blocks; STORY-008 reserved keyword not lifted). DIR-077-001 issued (architect directive: parse-stage parsing belongs in STORY-078 new story). DIR-077-001-A addendum: chumsky 0.10.1 research confirmed parse-time sub-block-KEY warning is correct via `validate()`/`emitter.emit()` + `ParseSeverity::Warning`; no BC-3.02.002 amendment needed. TYPE validation → eval stage (STORY-077). KEY warning + reserved-collision check → parse stage (STORY-078). STORY-077 status→blocked-on-078; branch feature/S-077 retained, worktree parked.
- 2026-06-01 — STORY-078 CREATED — Parser: section block syntax (5pts, P0, EPIC-02, Wave 4 Batch A). Blocks STORY-077. Added to Wave 4 Batch A. Project total: 80 stories / 473 pts.
- 2026-06-01 — DI-5 RESOLVED: PR #43 (a4ddae8d) added doctest CI job (66 doctests executed on ubuntu, `cargo test --doc --workspace --all-features`, wired into all-checks-pass aggregator; closes CI doctest-execution gap). Deferred-tasks sweep complete: DI-2/3/5 fixed, DI-4→STORY-079, DI-6→STORY-080, DI-1 deferred.
- 2026-06-01 — Deferred-tasks sweep: DI-2 RESOLVED (BC-1.14.004 → v1.2: subsystem SS-02 filled; story/arch/VP anchors resolved); DI-3 RESOLVED (BC-4.03.001 + BC-4.03.002 → v1.2: subsystem SS-07 filled; story/arch/VP anchors resolved). DI-4 → STORY-079 created (slideforge-diagrams SVG DoS hardening, 3pts, P2, Wave 5). DI-6 → STORY-080 created (test de-flake, 3pts, P2, Wave 5). DI-5 RESOLVED (see PR #43 row). DI-1 remains justified deferral (no action). New low-severity drift row added: BC-1.14.001/002/003 carry subsystem SS-TBD (correct value SS-02 per STORY-035) — fold into next spec-hygiene pass.

## STORY-045 Forward-Obligations

These obligations MUST be addressed in STORY-045 and cannot be split from each other:

- **From STORY-036:** contiguous-run sentinels for register content (BleedChecker ACs 001-007 `#[ignore]`'d pending exporters; register sentinels MUST be contiguous XML runs; add to STORY-045 implementation notes).
- **From STORY-043:** real frame-level Diagram/Chart alt text wired through SlideTagEngine (STORY-043 stub uses placeholder; STORY-045 must wire real alt text from IR frames).
- **From STORY-044 (F-P19 cascade adjudication):**
  1. `decorative_frame_indices` computed in tag_engine but NOT consumed in draw pass. Must emit `ContentTag::Artifact(ArtifactType::Other)` for decorative Image/Shape frames — MUST land in SAME change as decorative drawing, else untagged marked content emitted.
  2. `draw_body_blocks` uses single shared baseline for all blocks/bullets (multi-block overdraw). Must implement per-block/per-line baselines (real typography).
  3. `text_fill_black()` hardcodes RGB black. Must resolve text color from brand/theme palette in SAME change as brand/theme background fills — else dark-theme slides get black-on-dark contrast defect.

## Key Spec References

| Document | Scope |
|----------|-------|
| .factory/specs/behavioral-contracts/ | 109 BCs organized by section |
| .factory/specs/architecture/ARCH-INDEX.md | 12 architecture sections, 14 ADRs |
| .factory/specs/prd.md | PRD + 109 BCs + 15 holdout scenarios |
| .factory/stories/wave-schedule.md | 6 waves, batching, dependency order |
| .factory/stories/STORY-INDEX.md | 80 stories with status |
| .factory/stories/sprint-state.yaml | Current sprint/wave state |
| .factory/planning/q1-decision-final.md | Computation, formats, registers, charts, math, brand, roadmap |
| .factory/planning/q3-decision-final.md | Plugin-first architecture, 10 surfaces, trait signatures |
| .factory/cycles/STORY-044/coord-model-directive.md | Binding DIR-044-001: krilla coordinate model correction |
| .factory/cycles/STORY-077/section-parse-directive.md | Binding DIR-077-001 + addendum DIR-077-001-A: section-block parser ownership + chumsky parse-time-warning correction |
| .factory/cycles/STORY-077/chumsky-parse-warning-research.md | Research: chumsky 0.10.1 parse-time diagnostics verdict (confirms parse-time sub-block-KEY warning is correct) |

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
| 2026-05-28 | LOCAL adversary 3-CLEAN on STORY-034 ran macOS-only, missed Linux Trebuchet MS substitution failure (BC-1.12.001 violation) | LOW | Required 4 CI iterations. Process-gap: should LOCAL adversary spawn a Linux-container test pass for font/text/SVG/rendering stories? Surface to user for codification decision. DI-1: justified deferral (no action 2026-06-01). |
| 2026-05-31 | BC-1.14.004 frontmatter had `subsystem: SS-TBD` + unfilled template placeholders (Architecture Module, Stories, Story Anchor, VP Anchors). | LOW | **RESOLVED 2026-06-01** — BC-1.14.004 → v1.2: subsystem SS-02 filled; arch/story/VP anchors resolved. DI-2 closed. |
| 2026-05-31 | BC-4.03.001 + BC-4.03.002 had unfilled `subsystem: SS-TBD` + Stories/Story-Anchor/VP-Anchor placeholders (subsystem SS-07 is the correct answer for both PDF BCs). | LOW | **RESOLVED 2026-06-01** — BC-4.03.001 + BC-4.03.002 → v1.2: subsystem SS-07 filled; arch/story/VP anchors resolved. DI-3 closed. |
| 2026-05-31 | slideforge-diagrams `src/normalize.rs` usvg_normalize parses externally-sourced mermaid SVG via `usvg::Tree::from_str` WITHOUT a size/depth guard — DoS hardening gap (CWE-400), analogous to STORY-043 SEC-002. | LOW | **→ STORY-079 created 2026-06-01** (3pts, P2, Wave 5, EPIC-12). DI-4 closed (converted to story). |
| 2026-05-31 | CI has NO doctest job: `test` job uses `cargo nextest` (skips doctests); no `cargo test --doc` step. coords.rs runnable doctests checked by `cargo doc` but NOT executed in CI. | LOW | **RESOLVED 2026-06-01 (PR #43, a4ddae8d — doctest CI job added; 66 doctests now executed in CI, ubuntu runner, wired into all-checks-pass aggregator).** DI-5 closed. |
| 2026-05-31 | slideforge-data `http::tests::test_bc_1_03_002_http_4xx_not_retried` is FLAKY on windows-x86_64 — connection-count race in Windows test harness. Cold-budget timing test flaky under CI load. | LOW-MED | **→ STORY-080 created 2026-06-01** (3pts, P2, Wave 5, EPIC-19). DI-6 closed (converted to story). |
| 2026-06-01 | BC-1.14.001, BC-1.14.002, BC-1.14.003 still carry `subsystem: SS-TBD` (correct value SS-02 per STORY-035 subsystems field). Architect-flagged during DI-2 fix. | LOW | Fold into next spec-hygiene pass alongside any future SS-TBD sweeps. Not a story blocker. |

## Process Wins (apply to future stories)

- Pre-implementation tech-validation (research-agent) + architect coordinate-model directive for new-dependency stories catches library-vs-spec coordinate bugs AND wrong crate versions before implementation.
- Test-utils feature-gating pattern (STORY-036 BleedChecker) avoids test-only deps leaking into production builds.
- SVG paint-state isolation pattern (STORY-044): explicitly set fill + clear stroke before every text draw to prevent frame-to-frame bleed of paint state.
