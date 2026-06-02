---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-02
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
total_stories: 81
total_points: 491
total_waves: 6
total_epics: 21
dtu_required: false
dtu_assessment: 2026-05-24
dtu_clones_built: n/a
dtu_services: []
wave_1_gate: "PASS 2026-05-27 — 3 gate passes, 11 findings fixed"
wave_2_gate: "PASS 2026-05-27 — 11 gate passes, 19 findings fixed, 3/3 clean (passes 9-10-11)"
wave_3_gate: "PASSED 2026-05-31 — PR #38 (7d266ad7); adversary pass 8 strict-CLEAN; holdout must-pass 5/5"
wave_4_batch_a_complete: 9
wave_4_batch_a_total: 10
wave_4_started: 2026-05-31
wave_4_total_stories: 18
wave_4_total_points: 114
wave_5_total_points: 109
develop_sha: "e5d818e7"
develop_pr_count: 48
workspace_tests: "~2700+ (48 PRs merged; STORY-045 PDF/UA-1+veraPDF; STORY-075 footer detection; STORY-073 +TextRun tests; STORY-076 brand tests; STORY-078 parser tests)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## What Is This Project?

slideforge is a DATA-REACTIVE BRANDED DOCUMENT PLATFORM. Generates branded .pptx, .docx, .pdf, .html, and web preview from a single .sf DSL file with data binding, iteration, conditionals, and a plugin-first architecture.

**Repository:** https://github.com/drbothen/slideforge
**Workspace:** /Users/jmagady/Dev/slideforge
**Factory worktree:** .factory/ on branch `factory-artifacts`

---

## NEXT ACTIONS (Cold-Start Priority Order)

A new session with zero prior context should proceed in this order:

### 1. STORY-077 — SectionBlock IR Extension + Inline-Markup Parser (PRIORITY: P0)

**State:** Worktree `.worktrees/STORY-077` on branch `feature/S-077` (HEAD 1dc97f36). Section IR + register routing + inline-markup parser (TemplateChunk + `chunks_to_inline_nodes`) ALL IMPLEMENTED. Adversary cascade: pass-1 (F-077-P5-001 Expr::Call unreachable fixed), pass-2 (F-077-P2-001 genuine parse→eval test; F-077-P2-002 delimiter spans; underscore regression tests), pass-3 (CLEAN 1/3), pass-4 in progress. E-EVL-012/013 added (collision fix from E-EVL taxonomy); E-PAR-019/020 added (collision fix from E-PAR taxonomy). error-taxonomy v2.10. Streak **0/3** (pass-3 was CLEAN but pass-4+ cycle ongoing; see below).

**Scope:** 13 pts, P0, EPIC-18, crates: slideforge-types + slideforge-syntax + slideforge-eval. BCs: BC-3.02.002 v1.5, BC-1.14.003 v1.3. `SectionBlock.body: OrderedMap<Arc<str>, FieldValue>` + `register_content` field + inline-markup TemplateChunk parser + `chunks_to_inline_nodes`. ~32 layout call-site adjustments. Blocks STORY-041/042.

**Non-blocking deferred finding:** OBS-077-P4-A — duplicate `SectionBlock` type name across `slideforge-types` + `slideforge-plugin-api`; STORY-041/042 traceability concern. See Drift Items.

**Next step:** Adversary pass 4 → 3/3 strict-CLEAN → demo-recorder → pr-manager → human merge.

### 2. STORY-081 — Slide-Level Inline Markup (PRIORITY: P0, Wave 5)

**State:** DRAFT. New story created 2026-06-02. Depends on STORY-077 (inline-markup parser must land first) + STORY-041/042/043/044/046. BLOCKS v1.0 release.

**Scope:** 13 pts, P0, EPIC-18, Wave 5. BCs: BC-3.02.002 v1.5 (PC8). eval + layout + all-exporter structural formatting for slide-level inline markup. Closes the temporary inconsistency where section-level inline markup lands in STORY-077 but slide bodies remain plain strings.

**Next step:** After STORY-077 merges, test-writer → implementer → adversary cascade → demo-recorder → pr-manager → human merge.

---

## Session Resume Checkpoint

| Field | Value |
|-------|-------|
| **Date** | 2026-06-02 |
| **Position** | Phase 3, Wave 4 Batch A — 9/10 complete. STORY-045 MERGED PR #48 (e5d818e7). STORY-077 cascade rounds 1-3 complete (E-EVL-012/013, E-PAR-019/020; error-taxonomy v2.10); pass 4 NEXT; streak 0/3. STORY-081 draft (Wave 5). Open PRs: 0. |
| **develop SHA** | e5d818e7 (48 merged PRs) |
| **origin/develop** | authoritative — run `git fetch` before starting; local `develop` ref may be stale |
| **Active worktrees** | `.worktrees/STORY-077` (feature/S-077 @ 1dc97f36) |
| **Open PRs** | 0 |
| **Workspace crates** | 16 |
| **STORY-045** | MERGED — PR #48 squash-merged 2026-06-02 at develop e5d818e7. Worktree cleaned up. |
| **STORY-077** | IN PROGRESS — worktree .worktrees/STORY-077 (feature/S-077 @ 1dc97f36). Story v1.5 (13 pts). Inline-markup parser implemented. Cascade pass-1/2/3 done; pass 4 NEXT. Streak 0/3. E-EVL-012/013, E-PAR-019/020 added; error-taxonomy v2.10. |
| **STORY-081** | DRAFT — Wave 5, EPIC-18, 13 pts, P0. Slide-Level Inline Markup. Depends on STORY-077. Not started. |
| **BC deltas** | BC-2.01.001 v1.4, BC-3.02.002 v1.5 (PC8 inline-markup + recognized-type list 5→7), BC-3.05.001 v1.3.5, BC-1.14.003 v1.3 (SS-02), BC-4.03.001 v1.3, error-taxonomy v2.10 — all local-only on factory-artifacts. |
| **factory-artifacts** | Local only. Push requires explicit human authorization per CLAUDE.md. |

---

## Current Status

Phase 3 IN PROGRESS. Wave 1/2/3 COMPLETE (gates PASSED). **Wave 4 Batch A: 9/10 complete.** STORY-035/036/043/044/045/073/075/076/078 MERGED. STORY-077 cascade pass-1/2/3 done (pass 4 NEXT; streak 0/3; inline-markup implemented; error-taxonomy v2.10). STORY-081 draft (Wave 5, 13 pts).

develop: `e5d818e7` (48 merged PRs, 0 failures). 81 stories / 491 pts. Workspace: 16 crates. Open PRs: 0.

## Phase Progress

| Phase | Status | Key Output |
|-------|--------|-----------|
| Pre-pipeline | DONE | Toolchain + LLM + MCP preflight (2026-05-23) |
| Market intelligence | DONE 2026-05-23 | GO with medium confidence |
| Planning (25 DSL decisions) | DONE 2026-05-24 | q1–q25 decision docs + 14 research threads + 7/7 spikes resolved |
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (109 BCs, 15 HS, 4 supplements) + architecture (14 ADRs, 15 VPs, 20 crates) + UX spec. 17 passes, 69 findings, 3/3 clean. |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 81 stories (77 original + 4 added 2026-06-01/02), 21 epics, 6 waves, 491 pts. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3 GATE PASSED. **Wave 4 Batch A 9/10 (STORY-035+036+043+044+045+073+075+076+078 MERGED). STORY-077 cascade pass-1/2/3 done (pass 4 NEXT; streak 0/3; inline-markup implemented). STORY-081 DRAFT Wave 5.** | Per-story delivery |
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
| STORY-076 | srgbClr Transform | MERGED | #44 | eb78be51 |
| STORY-075 | Footer Detection | MERGED | #47 | 953fb5b2 |
| STORY-078 | Parser: section block syntax (P0) | MERGED | #46 | 5ab4cef4 |
| STORY-073 | Bullets Layout | MERGED | #45 | 47856465 |
| STORY-045 | PDF/UA-1 + veraPDF (P0) | MERGED | #48 | e5d818e7 |
| STORY-077 | SectionBlock IR Extension + Inline-Markup Parser (P0) | IN PROGRESS — cascade pass-1/2/3 done; pass 4 NEXT; streak 0/3; inline-markup implemented; E-EVL-012/013 + E-PAR-019/020 | — | 1dc97f36 (wt) |
| STORY-081 | Slide-Level Inline Markup (P0, Wave 5) | DRAFT — depends on STORY-077; not started | — | — |

## Decisions Log (milestones)

- 2026-05-23 — Workspace resolved, mode: greenfield. Market intelligence: GO.
- 2026-05-24 — ALL 25 DSL DESIGN QUESTIONS COMPLETE. ALL 7/7 SPIKES RESOLVED.
- 2026-05-25 — PHASE 1 CONVERGED (3/3 clean) + APPROVED. PHASE 2 CONVERGED (3/3 clean) + APPROVED. PHASE 3 STARTED.
- 2026-05-26 — Wave 1: STORIES 001–010 + 051/052/053/054 MERGED (14 stories).
- 2026-05-27 — WAVE 1 GATE PASSED. WAVE 2: 7 stories MERGED. WAVE 2 GATE PASSED.
- 2026-05-27/28 — WAVE 3 BATCH 1: 6 new crates (data, brand, layout, math, charts, diagrams), 494 new tests, PRs #21–#26.
- 2026-05-28/30 — WAVE 3 BATCH 2: 13 stories MERGED (PRs #27–#34).
- 2026-05-30 — WAVE 3 BATCH 3: STORY-021/024/025 MERGED (PRs #35–#37).
- 2026-05-31 — WAVE 3 GATE PASSED — 2584 tests GREEN; holdout must-pass 5/5; adversary 3-CLEAN (P6/7/8); fix-PR #38 (7d266ad7).
- 2026-05-31 — WAVE 4 STARTED. STORY-035/036/043/044 MERGED (PRs #39–#42). STORY-077 created (P0, 8pts, blocks 041/042). Total: 80 stories/473 pts. 16 crates.
- 2026-06-01 — DI-2/3/5 RESOLVED; DI-4→STORY-079; DI-6→STORY-080; DI-1 justified deferral. PR #43 (a4ddae8d) doctest CI job (DI-5 closed).
- 2026-06-01 — STORY-077 BLOCKED: parser gap (STORY-027 decomp gap). STORY-078 created (5pts, P0, EPIC-02). DIR-077-001 + DIR-077-001-A issued.
- 2026-06-01 — STORY-076 MERGED — PR #44, squash eb78be51. slideforge-brand. ColorSlot.is_derived; Option B verbatim hex; 7-pass adversary, 3/3 strict-CLEAN (P4/6/7). Security re-review CLEAN (2 MED fixed: CWE-789 + CWE-117, commit cbd7fefa). Wave 4 Batch A: 5/10.
- 2026-06-01 — STORY-078 MERGED — PR #46, squash 5ab4cef4. slideforge-syntax. Un-reserved `section`; section_block_parser → SectionNode; W-PAR-001; E-PAR-017/018. 10-pass adversary, 3/3 strict-CLEAN (P7/8/9). UNBLOCKS STORY-077. Wave 4 Batch A: 6/10.
- 2026-06-01 — STORY-073 MERGED — PR #45, squash 47856465 (develop HEAD). slideforge-layout. TextRun per BulletItem (recursive). MAX_BULLET_DEPTH=64 + E-LAY-007. 7-pass adversary, 3/3 strict-CLEAN (P5/6/7). Security CLEAN + pr-reviewer APPROVE. Wave 4 Batch A: 7/10.
- 2026-06-01 — Spec corrections committed to factory-artifacts (cfc28426, local-only): BC-2.01.001 v1.4, BC-3.02.002 v1.3, BC-3.05.001 v1.3.5, BC-4.03.001 v1.3, error-taxonomy v2.8 (E-PAR-017/018, W-PAR-001, E-LAY-007).
- 2026-06-01 — STORY-045 scope expanded by human: story v1.1 (8 pts), BC-4.03.001 v1.3 (AC-010/011/012/013 added). Adversary pass 1 found 3 CRIT+3 HIGH+2 MED — all fixed. Pass-2 next (0/3 streak).
- 2026-06-01 — ORCHESTRATOR ADJUDICATION (STORY-045 forward-obligations #2 and #3). Pass-2 surfaced process-gap: both obligations claimed mandatory but STORY-045 spec is silent on them (grep-confirmed). RULING: Obligation #2 (per-block/per-line baselines) CONFIRMED IN-SCOPE and FIXED — genuine correctness defect (all body text overprinting at shared baseline), not merely a forward-obligation. F-P2-001 closed; BODY_LINE_LEADING=1.2 per-block stacking; docstring corrected; TDD tests added; worktree advanced 0e01c254→9d098dd3. Obligation #3 (text color from brand/theme palette) + F-P2-002 (hardcoded text_fill_black) RE-COUPLED to the future bg-fill story and removed as STORY-045 blocker. Rationale: obligation #3's own text conditions it on the same change as bg fills; STORY-045 draws no background fills (FrameContent::Shape/Image/Empty are no-ops), so the contrast defect cannot occur; black text on no-fill background is PDF/UA-1-compliant; brand-driven text color is outside STORY-045 spec ACs. This is faithful to the obligation's condition, not an MVP deferral. BINDING: future adversary passes on STORY-045 must not re-flag F-P2-002.
- 2026-06-01 — STORY-075 CONVERGED (3/3 strict-CLEAN, passes 3/4/5). Demo evidence recorded. Parked — STORY-076 dependency now resolved. Rebase + PR NEXT.
- 2026-06-01 — STORY-075 PR #47 OPEN. Rebased onto develop 47856465 (d3c9787c). Demo evidence committed. Full pedantic gate green. pr-reviewer APPROVE (0 blocking). Security pass 1: SEC-001 IMPORTANT (zip-bomb) + 3 SUGGESTIONs — ALL FIXED (MAX_XML_ENTRY_BYTES take() guard + test, doc comments, RAII temp-file). Security re-review in progress. Awaiting CLEAN + human merge approval.
- 2026-06-01 — STORY-077 fresh delivery from develop. Red Gate done (16 RED). Impl green (fd539c4e). Adversary pass 1: 1 CRIT (F-077-P1-001 eval/layout section-type list divergence rejecting executive_summary/risk_register), 2 HIGH (paper-tested interpolation; untested EC-002 undefined-var), 2 OBS (EC-006 spec contradiction; missing layout regression test). Fix-burst in progress. 0/3 streak.
- 2026-06-01 — Session lessons LESSON-1..LESSON-6 codified (see Lessons section below).
- 2026-06-02 — STORY-075 MERGED — PR #47, squash 953fb5b2. slideforge-brand. Footer detection from .pptx. 5-pass adversary, 3/3 strict-CLEAN (P3/4/5). Security CLEAN (SEC-001 zip-bomb fixed: MAX_XML_ENTRY_BYTES take() guard + test). pr-reviewer APPROVE. Worktree .worktrees/STORY-075 cleaned up. Wave 4 Batch A: 8/10. develop: 953fb5b2 (47 PRs).
- 2026-06-02 — STORY-077 pass-3 CLEAN (strict: yes, PR-merge: yes). Streak 1/3. Pass-1 fix-burst (CRIT F-077-P1-001 section-type SSOT) + pass-2 fix-burst (register-key SSOT alias, BC corrections, OBS test gaps) both complete. BC-3.02.002 bumped v1.3→v1.4 (recognized-type list 5→7: +executive_summary, +risk_register). BC-1.14.003 bumped v1.2→v1.3 (subsystem SS-TBD→SS-02). STORY-077 bumped v1.3→v1.4 (EC-006 re-scoped top-level-only directive). All committed this burst. Pass 4 NEXT.
- 2026-06-02 — STORY-045 adversary pass 4 found F-P4-001 [MEDIUM]: draw_body_blocks_tagged mis-routes MCIDs for mixed Body blocks (sibling-function divergence from tag_content_block; latent per STORY-027 v1 layout scope, but production-grade fix in scope per canonical principle). Pass-3 fixes (per-block MCID, EC-008 test, deterministic font) all verified clean by pass 4. Streak reset 0/3. Fix in progress; worktree at 561fecd7 (will advance). Pass 5 NEXT.
- 2026-06-02 — STORY-045 adversary cascade CONVERGED — 3/3 strict-CLEAN (passes 6/7/8). All 8-pass cascade fixes applied (F-001..F-008, F-P2-001, F-P3-001, F-P4-001, OBS-P5-001). Worktree HEAD advanced to 2f2ef856. NEXT: demo-recorder per-AC → rebase onto develop → push → pr-manager 9-step PR cycle.
- 2026-06-02 — STORY-077 scope expansion (human decision 2026-06-02). DIR-077-002 issued: inline-markup parser (TemplateChunk variants + parser + chunks_to_inline_nodes in slideforge-eval) added to STORY-077 scope. Story v1.4→v1.5, 8→13 pts, est_days 3→5. BC-3.02.002 bumped v1.4→v1.5 (PC8 inline-markup clarification). Pass-5 finding F-077-P5-001 now properly resolved by expansion scope; streak reset 0/3. Worktree HEAD c7c1ae6f (section IR + register routing complete).
- 2026-06-02 — STORY-081 created (Slide-Level Inline Markup). EPIC-18, Wave 5, P0, 13 pts, status draft. Depends on STORY-077 + STORY-041/042/043/044/046. BC-3.02.002 v1.5 (PC8). BLOCKS v1.0 release. Closes temporary inconsistency where section-level inline markup lands in STORY-077 but slide bodies remain plain strings.
- 2026-06-02 — STORY-045 MERGED — PR #48 squash-merged at develop e5d818e7. Worktree cleaned up. Wave 4 Batch A: 9/10. develop: e5d818e7 (48 PRs). Open PRs: 0.
- 2026-06-02 — STORY-077 inline-markup cascade rounds: pass-1 (F-077-P5-001 Expr::Call unreachable fixed), pass-2 (F-077-P2-001 genuine parse→eval test; F-077-P2-002 delimiter spans; underscore regression tests), pass-3 CLEAN (1/3 streak). E-EVL diagnostic-code collision resolved → E-EVL-012 (FigrefInvalidArg) + E-EVL-013 (InlineXrefEmptyId). E-PAR collision resolved → E-PAR-019 (unclosed inline markup) + E-PAR-020 (empty inline markup span). error-taxonomy bumped v2.8→v2.10. HEAD 1dc97f36. Streak reset 0/3 by pass-4 (in progress). Pass 4 NEXT.

## Lessons / Process Gaps (codified 2026-06-01)

| ID | Lesson | Action |
|----|--------|--------|
| LESSON-1 | Adversary dispatches MUST use ABSOLUTE worktree paths. Read-only adversary agents have no Bash and don't reliably honor `cd` — 2 passes this session reviewed develop baseline instead of feature worktree. | Standing rule: orchestrator always passes `--cwd /Users/jmagady/Dev/slideforge/.worktrees/STORY-NNN` in adversary dispatch. |
| LESSON-2 | Canonical local clippy gate is the FULL pedantic form: `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items`. Bare `-- -D warnings` misses doc_markdown/uninlined_format_args; caused PR #45 to fail CI twice. | All agents verifying clippy MUST use the full form. CLAUDE.md already states the full form — treat deviations as defects. |
| LESSON-3 | After test-writer adds tests in a follow-up commit, the FULL pre-push gate (fmt + pedantic clippy + nextest) MUST be re-run before PR. STORY-073 OBS-2 test commits bypassed it → PR #45 red. | Per-story delivery checklist: pre-push gate is mandatory after EVERY commit to a feature branch. |
| LESSON-4 | Story-body BC version-LABEL pins drift when a BC bumps mid-cycle for an unrelated clause. Treat pure label drift as NON-BLOCKING when BC anchor + cited clause content resolve unchanged. | Do not reopen converged stories for label-only BC version drift. Consider removing version pins from story-body BC tables entirely in Phase 3+. |
| LESSON-5 | pr-manager (sub-agent) CANNOT dispatch sub-agents. The ORCHESTRATOR must dispatch the mandated independent security-reviewer + pr-reviewer for every PR. pr-manager does only an inline baseline review. | Orchestrator must explicitly dispatch security-reviewer + pr-reviewer after pr-manager creates the PR. |
| LESSON-6 | GitHub blocks the PR author account (drbothen) from self-approving. Human must click Approve. Auto-mode classifier requires explicit human merge authorization. | All PRs require explicit human merge approval — do not assume auto-merge. |
| DEFERRED | STORY-078/045 diagnostic messages embed user identifier tokens. Add `sanitize_ident_for_display` helper + lexer invariant assert. | Phase 6 hardening (LOW severity). Not a story blocker. Lexer-mitigated, not exploitable. |

## STORY-045 Forward-Obligations

These obligations MUST be addressed in STORY-045 and cannot be split:

- **From STORY-036:** contiguous-run sentinels for register content (BleedChecker ACs `#[ignore]`'d pending exporters; register sentinels MUST be contiguous XML runs).
- **From STORY-043:** real frame-level Diagram/Chart alt text wired through SlideTagEngine (STORY-043 stub uses placeholder; STORY-045 must wire real alt text from IR frames).
- **From STORY-044 (F-P19 cascade):**
  1. `decorative_frame_indices` → emit `ContentTag::Artifact(ArtifactType::Other)` for decorative Image/Shape — MUST land in SAME change as decorative drawing.
  2. `draw_body_blocks` single shared baseline → per-block/per-line baselines — **DONE** (F-P2-001 fixed 2026-06-01: per-block/bullet baseline stacking with BODY_LINE_LEADING=1.2; docstring corrected; TDD tests added; worktree HEAD 9d098dd3).
  3. `text_fill_black()` → resolve text color from brand/theme palette — **RE-COUPLED to future bg-fill story; NOT a STORY-045 blocker** (orchestrator ruling 2026-06-01). STORY-045 draws no background fills, so black-on-dark contrast defect cannot occur; PDF/UA-1-compliant. Obligation #3's own text conditions it on the same change as bg fills. Track as coupled obligation on whichever future story first adds slide/region background fills. Future adversary passes on STORY-045 must not re-flag this.

## Key Spec References

| Document | Scope |
|----------|-------|
| .factory/specs/behavioral-contracts/ | 109 BCs organized by section |
| .factory/specs/architecture/ARCH-INDEX.md | 12 architecture sections, 14 ADRs |
| .factory/specs/prd.md | PRD + 109 BCs + 15 holdout scenarios |
| .factory/stories/wave-schedule.md | 6 waves, batching, dependency order |
| .factory/stories/STORY-INDEX.md | 81 stories with status |
| .factory/stories/sprint-state.yaml | Current sprint/wave state |
| .factory/planning/q1-decision-final.md | Computation, formats, registers, charts, math, brand, roadmap |
| .factory/planning/q3-decision-final.md | Plugin-first architecture, 10 surfaces, trait signatures |
| .factory/cycles/STORY-044/coord-model-directive.md | Binding DIR-044-001: krilla coordinate model correction |
| .factory/cycles/STORY-077/section-parse-directive.md | Binding DIR-077-001 + DIR-077-001-A: section-block parser ownership + chumsky parse-time-warning |
| .factory/cycles/STORY-077/chumsky-parse-warning-research.md | chumsky 0.10.1 parse-time diagnostics verdict |

## Quality Bar (Non-Negotiable)

Production-grade from day 1. Key enforced gates:
- `#![forbid(unsafe_code)]` on all crates; zero `.unwrap()` outside tests; `clippy::pedantic` clean; `#![warn(missing_docs)]` on public APIs
- Kani proofs for pure-core functions (Phase 6); `cargo-fuzz` harness; `cargo-mutants` with documented kill-rate budget
- WCAG AA: web preview + HTML; PDF/UA-1 on PDF; OOXML accessibility linter on PPTX
- < 500ms cold build, < 50ms incremental — CI benchmark gate
- Signed releases + SBOM; `cargo audit` + `cargo deny` in CI; semgrep/CodeQL per PR
- Cross-platform: macOS arm64+x86_64, Linux x86_64+arm64, Windows x86_64
- Full quality bar table: CLAUDE.md

## Drift Items

| Date | Item | Severity | Notes |
|------|------|----------|-------|
| 2026-05-28 | LOCAL adversary 3-CLEAN on STORY-034 ran macOS-only, missed Linux Trebuchet MS substitution failure. | LOW | DI-1: justified deferral (no action). Surface to user for Linux-container adversary codification decision. |
| 2026-06-01 | BC-1.14.001, BC-1.14.002 still carry `subsystem: SS-TBD` (correct value SS-02 per STORY-035). BC-1.14.003 RESOLVED (bumped v1.2→v1.3, SS-TBD→SS-02, committed 2026-06-02). | LOW | BC-1.14.001/002 fold into next spec-hygiene pass. Not a story blocker. |
| 2026-06-02 | OBS-P6-001 (STORY-045 pass 6): PDF exporter ignores `opts.strict` and `laid_out.warnings` — cross-exporter contract concern. Exporter-specific opt interpretation is a wave-gate integration question, not a single-story concern. | LOW | Route to wave-gate post-STORY-045 merge. Non-blocking. |
| 2026-06-02 | OBS-077-P4-A (STORY-077 pass 4): Duplicate `SectionBlock` type name across `slideforge-types` + `slideforge-plugin-api` — STORY-041/042 traceability concern. Name collision could cause confusion but does not block STORY-077 functionality. | LOW | Route to STORY-041/042 (plugin-api consumers). Non-blocking for STORY-077. |
| 2026-06-02 | E-EVL-007..E-EVL-011 implemented in slideforge-eval/src/error.rs but NOT registered in error-taxonomy.md (pre-existing taxonomy debt, predates STORY-077). | LOW | Follow-up: taxonomy-completeness backfill task. Not a STORY-077 blocker. |
| 2026-06-02 | SEC-001 (STORY-045 PR #48): veraPDF Docker image `verapdf/cli:latest` in .github/workflows/pdf-ua1.yml not digest-pinned (CWE-494). CI-only risk. | MEDIUM | Fix before v1.0 / Phase 6. Merged in #48 — track for Phase 6 hardening sweep. |
| 2026-06-02 | SEC-002 (STORY-045 PR #48): emu_to_pt i64→f32 precision loss (low impact; Phase 6 Kani bound). SEC-003: CI tee temp-file uses predictable path (self-hosted runner only, LOW). | LOW | Noted. Phase 6 formal hardening. |

## Process Wins (apply to future stories)

- Pre-implementation tech-validation (research-agent) + architect coordinate-model directive for new-dependency stories catches library-vs-spec coordinate bugs before implementation.
- Test-utils feature-gating pattern (STORY-036 BleedChecker) avoids test-only deps leaking into production builds.
- SVG paint-state isolation pattern (STORY-044): explicitly set fill + clear stroke before every text draw.
- Security re-review after targeted MED fix (STORY-076): fix-then-re-review pattern confirms no new surface introduced.
