---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-01
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
wave_3_gate: "PASSED 2026-05-31 — PR #38 (7d266ad7); adversary pass 8 strict-CLEAN; holdout must-pass 5/5"
wave_4_batch_a_complete: 7
wave_4_batch_a_total: 10
wave_4_started: 2026-05-31
wave_4_total_stories: 18
wave_4_total_points: 109
develop_sha: "47856465"
develop_pr_count: 46
workspace_tests: "~2700+ (46 PRs merged; STORY-073 +TextRun tests; STORY-076 brand tests; STORY-078 parser tests)"
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

### 1. STORY-045 — PDF/UA-1 Tagging + veraPDF (PRIORITY: P0, long pole)

**State:** Worktree `/Users/jmagady/Dev/slideforge/.worktrees/STORY-045` on branch `feature/S-045` (HEAD 9d098dd3). Adversary cascade at **0/3 strict-CLEAN** (pass 1 complete + pass 2 complete — F-P2-001 FIXED, F-P2-002 ADJUDICATED out-of-scope). Tests pass. Pass 3 NEXT.

**Scope:** Story v1.1, 8 pts, BC-4.03.001 v1.3. ACs: AC-001..AC-009 (original) + AC-010 (/Outlines bookmarks), AC-011 (Hn /Title text), AC-012 (Validator::UA1 both export paths), AC-013 (veraPDF isCompliant CI gate).

**Pass-1 fixes already applied (do NOT re-do):** F-001 CI runs correct test `ac013_verapdf_full_compliance`; F-002 CI flag `--run-ignored all`; F-003 `PdfExportError::ValidationFailed` + `KrillaError::Validation` match; F-004 `src/outline.rs` with `build_outline_entries` (count/order/destination tests); F-005 `source_index`/`page_idx` invariant; F-006 deterministic font via `with_font_path`; F-007 `/Lang`-absent fatal under UA1; F-008 structural outline assertions; OBS-012 Subtitle H2 own text.

**Mandatory forward-obligations** (MUST land in STORY-045, not split out): (a) `decorative_frame_indices` → emit `ContentTag::Artifact(ArtifactType::Other)` for decorative Image/Shape frames in SAME change as decorative drawing; (b) per-block/per-line baselines — **DONE** (F-P2-001 fixed: per-block/bullet baseline stacking with BODY_LINE_LEADING=1.2, worktree HEAD 9d098dd3); (c) text color from brand/theme palette — **RE-COUPLED to future bg-fill story (orchestrator ruling 2026-06-01) — NOT a STORY-045 blocker** (see Decisions Log entry 2026-06-01 and STORY-045 Forward-Obligations section).

**Next step:** Dispatch LOCAL adversary pass 3 against `/Users/jmagady/Dev/slideforge/.worktrees/STORY-045` (ABSOLUTE path — see LESSON-1). Continue cascade to 3/3 strict-CLEAN, then demo-recorder per-AC, push, pr-manager 9-step PR → orchestrator dispatches independent security-reviewer + pr-reviewer → human merge approval.

---

### 2. STORY-075 — Brand Loader: Footer Detection from .pptx (PRIORITY: P1)

**State:** Worktree `/Users/jmagady/Dev/slideforge/.worktrees/STORY-075` on branch `feature/S-075` (HEAD d3c9787c). **CONVERGED** (3/3 strict-CLEAN at passes 3/4/5). **PR #47 OPEN** (base develop, rebased onto 47856465). Demo evidence committed. Full pedantic gate green. pr-reviewer APPROVE (0 blocking). Security pass 1 found SEC-001 (IMPORTANT zip-bomb) + 3 SUGGESTIONs — ALL FIXED (MAX_XML_ENTRY_BYTES take() guard + test, doc comments, RAII temp-file). Security RE-REVIEW in progress.

**Next step:** Await security RE-REVIEW CLEAN confirmation → human merge approval (LESSON-6).

---

### 3. STORY-077 — SectionBlock IR Extension (PRIORITY: P0, now UNBLOCKED)

**State:** Worktree `.worktrees/STORY-077` on branch `feature/S-077` (fresh from develop). Red Gate DONE (16 RED tests). Implementer GREEN (commit fd539c4e). Adversary pass 1 found 1 CRIT + 2 HIGH + 2 OBS — fix-burst IN PROGRESS (implementer + story-writer). **0/3 strict-CLEAN streak.**

**CRIT:** F-077-P1-001 — eval/layout section-type list divergence rejecting `executive_summary`/`risk_register`. **HIGH:** paper-tested interpolation; untested EC-002 undefined-var handling. **OBS:** EC-006 spec contradiction; missing layout regression test. Fix-burst in progress.

**Scope:** 8 pts, P0, EPIC-02, crates: slideforge-types + slideforge-syntax + slideforge-eval. BCs: BC-3.02.002 v1.3, BC-1.14.003 v1.4. `SectionBlock.body: OrderedMap<Arc<str>, FieldValue>` + `register_content` field. ~32 layout call-site adjustments. Blocks STORY-041/042.

**Next step:** Complete fix-burst → adversary pass 2 → continue cascade to 3/3 strict-CLEAN → demo-recorder → pr-manager → human merge. **DO NOT touch STORY-077 index/citations mid-fix.**

---

## Session Resume Checkpoint

| Field | Value |
|-------|-------|
| **Date** | 2026-06-01 |
| **Position** | Phase 3, Wave 4 Batch A — 7/10 complete. STORY-045 adversary cascade pass-3 next (0/3 streak; F-P2-001 fixed, F-P2-002 adjudicated out-of-scope). STORY-075 PR #47 OPEN awaiting security re-review CLEAN + human merge. STORY-077 fix-burst in progress (pass-1 findings). |
| **develop SHA** | 47856465 (46 merged PRs) |
| **origin/develop** | authoritative — run `git fetch` before starting; local `develop` ref may be stale |
| **Active worktrees** | `.worktrees/STORY-045` (feature/S-045 @ 9d098dd3), `.worktrees/STORY-075` (feature/S-075 @ d3c9787c), `.worktrees/STORY-077` (feature/S-077) |
| **Open PRs** | 1 — #47 (STORY-075, awaiting security re-review + human merge) |
| **Workspace crates** | 16 |
| **STORY-073** | MERGED — PR #45, squash 47856465 (2026-06-01). slideforge-layout. TextRun per BulletItem (recursive, parent-before-children). MAX_BULLET_DEPTH=64 + LayoutError::BulletDepthExceeded (E-LAY-007). 7-pass adversary, 3/3 strict-CLEAN (P5/6/7). Security CLEAN + pr-reviewer APPROVE. |
| **STORY-076** | MERGED — PR #44, squash eb78be51 (2026-06-01). slideforge-brand. ColorSlot.is_derived; Option B verbatim base hex; brand.toml derived-comment. 7-pass adversary, 3/3 strict-CLEAN (P4/6/7). Security re-review CLEAN (2 MED fixed: CWE-789 MAX_TRANSFORMS_PER_SLOT=8, CWE-117 val sanitization, commit cbd7fefa). pr-reviewer APPROVE. |
| **STORY-078** | MERGED — PR #46, squash 5ab4cef4 (2026-06-01). slideforge-syntax. Un-reserved `section` keyword; section_block_parser → SectionNode; W-PAR-001 parse-time warning; E-PAR-017/E-PAR-018. 10-pass adversary, 3/3 strict-CLEAN (P7/8/9). Security CLEAN + pr-reviewer APPROVE. UNBLOCKS STORY-077. |
| **STORY-045** | IN PROGRESS — worktree .worktrees/STORY-045 (feature/S-045 @ 9d098dd3). Pass 1 + pass 2 done (F-P2-001 FIXED, F-P2-002 adjudicated out-of-scope per orchestrator ruling). Pass-3 NEXT. 0/3 strict-CLEAN streak. Story v1.1 (8 pts). |
| **STORY-075** | PR #47 OPEN — worktree .worktrees/STORY-075 (feature/S-075 @ d3c9787c). 3/3 CLEAN. Rebased + demo committed + pedantic gate green. pr-reviewer APPROVE. Security re-review in progress. Awaiting CLEAN + human merge. |
| **STORY-077** | FIX-BURST IN PROGRESS — worktree .worktrees/STORY-077 (feature/S-077). Red Gate done, impl green (fd539c4e). Pass-1 found 1 CRIT + 2 HIGH + 2 OBS; fix-burst ongoing. 0/3 streak. DO NOT touch index/citations mid-fix. |
| **BC deltas** | BC-2.01.001 v1.4, BC-3.02.002 v1.3 (SS-01), BC-3.05.001 v1.3.5 (SS-05), BC-4.03.001 v1.3, error-taxonomy v2.8 (E-PAR-017, E-PAR-018, W-PAR-001, E-LAY-007) — all committed cfc28426 (factory-artifacts, local-only). |
| **factory-artifacts** | Local only. Push requires explicit human authorization per CLAUDE.md. |

---

## Current Status

Phase 3 IN PROGRESS. Wave 1/2/3 COMPLETE (gates PASSED). **Wave 4 Batch A: 7/10 complete.** STORY-035/036/043/044/073/076/078 MERGED. STORY-045 adversary cascade pass-3 next (0/3). STORY-075 PR #47 OPEN awaiting security re-review + human merge. STORY-077 fix-burst in progress (0/3).

develop: `47856465` (46 merged PRs, 0 failures). 80 stories / 473 pts. Workspace: 16 crates.

## Phase Progress

| Phase | Status | Key Output |
|-------|--------|-----------|
| Pre-pipeline | DONE | Toolchain + LLM + MCP preflight (2026-05-23) |
| Market intelligence | DONE 2026-05-23 | GO with medium confidence |
| Planning (25 DSL decisions) | DONE 2026-05-24 | q1–q25 decision docs + 14 research threads + 7/7 spikes resolved |
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (109 BCs, 15 HS, 4 supplements) + architecture (14 ADRs, 15 VPs, 20 crates) + UX spec. 17 passes, 69 findings, 3/3 clean. |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 80 stories (77 original + 3 added 2026-06-01), 21 epics, 6 waves, 473 pts. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3 GATE PASSED. **Wave 4 Batch A 7/10 (STORY-035+036+043+044+073+076+078 MERGED). STORY-045 pass-3 next (0/3). STORY-075 PR #47 OPEN. STORY-077 fix-burst (0/3).** | Per-story delivery |
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
| STORY-075 | Footer Detection | PR #47 OPEN — security re-review in progress | #47 | d3c9787c (wt) |
| STORY-078 | Parser: section block syntax (P0) | MERGED | #46 | 5ab4cef4 |
| STORY-073 | Bullets Layout | MERGED | #45 | 47856465 |
| STORY-045 | PDF/UA-1 + veraPDF (P0) | IN PROGRESS — pass-3 next (0/3 streak; F-P2-001 fixed, F-P2-002 adjudicated) | — | 9d098dd3 (wt) |
| STORY-077 | SectionBlock IR Extension (P0) | IN PROGRESS — fix-burst (pass-1 findings); 0/3 streak | — | fd539c4e (wt) |

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
| .factory/stories/STORY-INDEX.md | 80 stories with status |
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
| 2026-06-01 | BC-1.14.001, BC-1.14.002, BC-1.14.003 still carry `subsystem: SS-TBD` (correct value SS-02 per STORY-035). | LOW | Fold into next spec-hygiene pass. Not a story blocker. |

## Process Wins (apply to future stories)

- Pre-implementation tech-validation (research-agent) + architect coordinate-model directive for new-dependency stories catches library-vs-spec coordinate bugs before implementation.
- Test-utils feature-gating pattern (STORY-036 BleedChecker) avoids test-only deps leaking into production builds.
- SVG paint-state isolation pattern (STORY-044): explicitly set fill + clear stroke before every text draw.
- Security re-review after targeted MED fix (STORY-076): fix-then-re-review pattern confirms no new surface introduced.
