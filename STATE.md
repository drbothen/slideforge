---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-06
phase_1_approved: 2026-05-25
phase_2_approved: 2026-05-25
phase_1_convergence: "17 passes, 69 findings, 3/3 clean (passes 15-16-17)"
phase_2_convergence: "22 passes, 96+ findings, 3/3 clean (passes 20-21-22)"
prd_bcs: 116
prd_hs: 15
prd_vps: 15
prd_supplements: 4
spikes_resolved: 7
spikes_total: 7
total_stories: 88
total_points: 542
total_waves: 6
total_epics: 21
dtu_required: false
dtu_assessment: 2026-05-24
dtu_clones_built: n/a
dtu_services: []
wave_1_gate: "PASS 2026-05-27 — 3 gate passes, 11 findings fixed"
wave_2_gate: "PASS 2026-05-27 — 11 gate passes, 19 findings fixed, 3/3 clean (passes 9-10-11)"
wave_3_gate: "PASSED 2026-05-31 — PR #38 (7d266ad7); adversary pass 8 strict-CLEAN; holdout must-pass 5/5"
wave_4_batch_a_complete: 10
wave_4_batch_a_total: 10
wave_4_merged: 21
story_050_status: DONE
wave_4_started: 2026-05-31
wave_4_total_stories: 21
wave_4_total_points: 150
wave_5_total_points: 132
develop_sha: "030dec6c"
develop_pr_count: 61
error_taxonomy_version: "v2.16"
workspace_tests: "~3290 (61 merged PRs; STORY-086 TextTag + Stage-2b + bullets path; 3290/3290 pass, 1 pre-existing cold_budget flake tracked under STORY-080)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## What Is This Project?

slideforge is a DATA-REACTIVE BRANDED DOCUMENT PLATFORM. Generates branded .pptx, .docx, .pdf, .html, and web preview from a single .sf DSL file with data binding, iteration, conditionals, and a plugin-first architecture.

**Repository:** https://github.com/drbothen/slideforge
**Workspace:** /Users/jmagady/Dev/slideforge
**Factory worktree:** .factory/ on branch `factory-artifacts`

---

## CURRENT POSITION

Phase 3, **Wave 4 — 21/21 MERGED. Wave 4 gate FAILED. STORY-086 delivery IN PROGRESS (Red Gate DONE; implementer GREEN PASS DONE; adversary pass 2 pending; streak 0/3). BLK-002 OPEN.**

- `develop` = `030dec6c` (61 merged PRs; origin/develop confirmed). **Open PRs: 0.**
- Active worktrees: `.worktrees/STORY-086` (feature/STORY-086, HEAD 065303b2, 6 micro-commits 27a6c3d2..065303b2).
- Workspace: 3290/3290 pass (1 pre-existing cold_budget flake tracked under STORY-080). Canonical exit gate CLEAN: fmt + pedantic clippy + RUSTDOCFLAGS doc + nextest.
- Wave 4 gate ran 2026-06-06. Gate 1 PASS. Gate 2 SKIP (no DTU). Gate 3 FAIL (adversary: 1 CRITICAL + 3 HIGH). Gate 5 FAIL (holdout: mean 0.56 / min_critical 0.30 — both below threshold). Consistency audit FAIL (4 blockers, 8 warnings — swept).

**Wave 4 gate FAILED.** Root cause: eval emits no slide-body ContentBlocks (for_eval.rs:342) — content-EMPTY output across all exporters + a11y strict-gate unsatisfiable even with correct alt.
**REMEDIATION IN PROGRESS.** Red Gate committed (6c87c9ef, feature/STORY-086). Implementer GREEN PASS COMPLETE (HEAD 065303b2, 6 micro-commits). All 6 RED→GREEN. Issue 1 (alt=None media blocks): Stage 2b emits Chart/Image/Diagram even when alt=None (AltText::Unspecified); pre-layout AltTextValidator::validate() restricted to Shape only (ADR-018 v1.2 Decision-3). Issue 2 (TextTag routing): Stage 2b tags TextBlocks; layout.rs routes ContentBlock::Text(tag) → FrameContent::Title/Subtitle/Body/TextRun (AC-023); DOCX document_body.rs added Subtitle→Heading2 + Body→Normal arms (positional fallback removed, AC-003 via TAG path); PDF tag_engine.rs added /ActualText on Text-in-Body frames; PPTX slide_serializer.rs required no change; no TextTag inspection placed in exporters (ADR-005 boundary held). Issue 3 (AC-007 bullets): Value::List unit test + E2E fixture green. BLK-002 remains OPEN until STORY-086 merges + Wave 4 gates re-pass.

---

## NEXT ACTIONS (zero-context orchestrator: execute in order)

**STATUS: Wave 4 gate FAILED (2026-06-06). STORY-086 Red Gate DONE + Implementer GREEN PASS DONE. Adversary LOCAL cascade resuming from pass 2, streak 0/3.**

### Step 1 — DONE: Remediation scoped + de-risked

ADR-019 (Stage 2b post-eval field-to-block threading pass) accepted (v1.1 — D1 TextTag→FrameContent routing corrected to layout-side). STORY-086 (slide-field-to-block threading, Wave 4 remediation, 21 pts, P0) created — closes BLK-002/F-G3-CRIT-001/F-G3-HIGH-001/002. STORY-087 (color-coded slide types, Wave 5, closes F-G3-HIGH-003) created. STORY-088 (bullets list-literal DSL, Wave 5, 5 pts) created. BCs BC-1.16.001 + BC-1.17.001/002/003 created; BC-5.02.001 v1.6; BC-5.01.001 v1.3; error-taxonomy v2.16; BC-INDEX 116 BCs. Stories corrected to real codebase: STORY-086 v1.2, STORY-087 v1.1, STORY-088 v1.1. BCs 4.01.001/4.02.001/1.16.001 prose clarified (no version bump).

### Step 2 — IN PROGRESS: Deliver STORY-086

Red Gate COMPLETE (commit 6c87c9ef on feature/STORY-086): test-writer added TextTag scaffolding (enum {Title,Subtitle,Body,Untagged} in slideforge-types/src/block.rs, non-optional tag: TextTag on TextBlock, 18-site Untagged sweep across 8 files) + 6 intentional behavioral failures.

Implementer GREEN PASS COMPLETE (HEAD now 065303b2, 6 micro-commits 27a6c3d2..065303b2). All 6 RED→GREEN. Full canonical exit gate CLEAN (fmt + pedantic clippy + RUSTDOCFLAGS doc + nextest; 3290 tests pass; 1 pre-existing cold_budget flake tolerated under STORY-080). Three issues resolved in-scope:
- Issue 1: Stage 2b emits Chart/Image/Diagram blocks when alt=None (AltText::Unspecified); AltTextValidator::validate() restricted to ContentBlock::Shape only (ADR-018 v1.2 Decision-3); out-of-scope pre-layout Chart/Image/Diagram tests migrated to Shape; single-fire AC-005 guaranteed.
- Issue 2 (TextTag routing): Stage 2b sets semantic tags; layout.rs routes ContentBlock::Text(tag) → FrameContent::Title/Subtitle/Body/TextRun (tag-driven, AC-023). DOCX document_body.rs needed new Subtitle→Heading2 + Body→Normal FrameContent arms (positional fallback removed); AC-003 now passes via TAG path not positional. PDF tag_engine.rs added /ActualText on Text-in-Body frames. PPTX slide_serializer.rs: no change. No TextTag inspection placed in exporters (ADR-005 boundary held). RECON CORRECTION to spec v1.2 "zero exporter change" noted (DOCX + PDF DID need arms).
- Issue 3: AC-007 @var-binding bullets path green (Value::List unit test + E2E).

NEXT SUB-STEP: RESUME adversary LOCAL cascade from pass 2, streak 0/3, target 3 strict-CLEAN. Then demo → PR → security-reviewer + pr-reviewer → merge. Then RE-RUN Wave 4 gates (Gate 3 + Gate 5). BLK-002 stays OPEN.

### Step 3 — Re-run failed Wave 4 gates

After STORY-086 merges: re-run Gate 3 (adversary) + Gate 5 (holdout) on the patched develop. Gate 1 + Gate 2 carry over from first run.

### Step 4 — Advance to Wave 5 only after re-gate passes

Only after all Wave 4 gates pass: begin Wave 5 with STORY-087, STORY-082, STORY-081, etc. STORY-082 — PPTX Slide-Grouping Sections (Wave 5, 5 pts, P0, BC-4.01.003 Half B, BC-1.14.003). Split from STORY-040 (human-authorized 2026-06-04). EPIC-08. BLK-002 remains OPEN until STORY-086 merges + gates re-pass.

---

## STANDING AUTHORIZATIONS

**STANDING MERGE AUTH (granted 2026-06-02):** Orchestrator MAY squash-merge any PR that is CI-green + security-reviewer CLEAN + pr-reviewer APPROVE, without re-asking the human. GitHub blocks author self-approve, so orchestrator runs `gh pr merge --squash` directly once those gates pass. Revocable by human.

**factory-artifacts PUSHED to remote (origin/factory-artifacts) — human-authorized 2026-06-04 for cross-machine durability; ongoing pushes of factory-artifacts to remote are now authorized.** Upstream tracking set (`-u origin factory-artifacts`). Fresh sessions on any machine may clone + `git worktree add .factory factory-artifacts` to restore all artifacts.

---

## Phase Progress

| Phase | Status | Key Output |
|-------|--------|-----------|
| Pre-pipeline | DONE | Toolchain + LLM + MCP preflight (2026-05-23) |
| Market intelligence | DONE 2026-05-23 | GO with medium confidence |
| Planning (25 DSL decisions) | DONE 2026-05-24 | q1–q25 docs + 14 research threads + 7/7 spikes resolved |
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (112 BCs, 15 HS, 4 supplements) + arch (14 ADRs (+4 added Phase 3: ADR-015..018), 15 VPs, 20 crates) + UX spec. 17 passes, 69 findings, 3/3 clean. |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 88 stories, 21 epics, 6 waves, 542 pts (LESSON-13 reconciliation: +4 stories/+14 pts added 2026-06-04; STORY-086/087 added 2026-06-06 +26 pts; STORY-086 scope 13→21 pts + STORY-088 added 2026-06-06 +5 pts net). 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3 GATE PASSED. Wave 4: 21/21 merged. Wave 4 gate RAN 2026-06-06 — FAILED (Gate 3: 1 CRIT + 3 HIGH; Gate 5: holdout 0.56/0.30 below threshold). Remediation in progress. | Per-story delivery |
| Phases 4–7 | NOT STARTED | Holdout / Adversarial / Formal Hardening / Convergence |

## Wave 4 Story Status

**Batch A — ALL MERGED (10/10):** PRs #39–#49 (develop c8913cad). STORY-035, -036, -043, -044, -073, -075, -076, -078, -045, -077. STORY-077 follow-ups PR #50 (f2573bb1, 2026-06-03).

**Batch B — ALL MERGED (6/6):**
- STORY-041 MERGED PR #51 (a3b47303, 2026-06-03)
- STORY-042 MERGED PR #53 (56f3f57d, 2026-06-03) — docx chain COMPLETE
- STORY-037 MERGED PR #52 (2ebf184f, 2026-06-03)
- STORY-038 MERGED PR #54 (c031805c, 2026-06-03)
- STORY-039 MERGED PR #55 (a4f29e5a, 2026-06-04)
- STORY-040 MERGED PR #56 (869fb401, 2026-06-04) — pptx chain 037→038→039→040 COMPLETE

**Batch C — ALL MERGED (5/5):**
- STORY-083: Plugin Registry Builder — MERGED PR #57 (5aaa27d2, 2026-06-05) — 6-pass cascade, 3/3 strict-CLEAN
- STORY-084: Bundled SectionType Implementations — MERGED PR #58 (801f351b, 2026-06-05) — 7-pass cascade, 3/3 strict-CLEAN
- STORY-085: Bundled DefaultInlineFormat + PPTX dog-fooding — MERGED PR #59 (e704e700, 2026-06-05) — 9-pass cascade, 3/3 strict-CLEAN (passes 7-8-9). 20/20 CI green; security APPROVE/CLEAN; pr-reviewer APPROVE.
- STORY-049: Plugin Registry Assembly — MERGED PR #60 (e6f7832d, 2026-06-05) — 12-pass cascade, 3/3 strict-CLEAN (passes 10-11-12). 21/21 CI green; security APPROVE/CLEAN; pr-reviewer APPROVE.
- STORY-050: E2E Integration Suite + PDF/a11y/observability pipeline fixes — MERGED PR #61 (030dec6c, 2026-06-06). All CI green; security APPROVE/CLEAN (SEC-050-001 fixed in-scope); pr-reviewer APPROVE/CLEAN. LOCAL cascade 5 passes, 3/3 strict-CLEAN (passes 3-4-5). DRIFT-CRITICAL-1 (PDF title) + DRIFT-CRITICAL-2 (alt-text bypass) RESOLVED. ADR-018 v1.1, BC-5.02.001 v1.5, BC-5.01.001 v1.2, error-taxonomy v2.15.

**Wave 4 = 21/21 COMPLETE.**
**STORY-082** (slide-grouping sections) moved to Wave 5 (human-authorized split from STORY-040)

---

## Session Resume Checkpoint

**STORY-086 Red Gate DONE + Implementer GREEN PASS DONE. Streak 0/3. NEXT: adversary LOCAL pass 2.**

| Field | Value |
|-------|-------|
| **Date** | 2026-06-06 |
| **Position** | Wave 4: 21/21 merged. Wave 4 gate FAILED. STORY-086 delivery in progress: Red Gate DONE (6c87c9ef) + implementer GREEN PASS DONE (HEAD 065303b2, 6 micro-commits). All 6 RED→GREEN. Stories 88 / 542 pts. BLK-002 OPEN. |
| **develop SHA** | `030dec6c` (61 merged PRs; origin/develop confirmed) |
| **Active worktrees** | `.worktrees/STORY-086` (feature/STORY-086, HEAD 065303b2) |
| **Open PRs** | 0 |
| **Workspace crates** | 17 |
| **Workspace tests** | 3290/3290 pass (1 pre-existing cold_budget flake tracked under STORY-080) |
| **factory-artifacts** | PUSHED to remote (origin/factory-artifacts) — human-authorized 2026-06-04. Upstream tracking set. Fresh sessions: clone repo + `git worktree add .factory factory-artifacts`. |
| **DURABLE ARTIFACTS** | (1) `.factory/cycles/STORY-086/adversarial-reviews/adversary-STORY-086-pass-1.md` — full pass-1 findings; (2) `.factory/cycles/STORY-086/architect-pass-1-adjudication.md` — D1-D5 Issues 1/2/3 resolution; (3) `.factory/specs/wave4-expanded-scope-uncertainty-resolution.md` — authoritative D1-D5 resolution + real codebase mapping; (4) `.factory/specs/architecture/adr/ADR-019-stage-2b-field-to-block-threading.md` v1.1 — TextTag→FrameContent routing corrected to layout-side; (5) STORY-086 v1.2 + STORY-087 v1.1 + STORY-088 v1.1 — corrected to real codebase; (6) ADR-018 v1.2 — AltTextValidator::validate() restricted to Shape only (Decision-3). |
| **RESUME INSTRUCTION** | STORY-086 implementer GREEN PASS is DONE (HEAD 065303b2). RESUME adversary LOCAL cascade from pass 2, streak 0/3. Dispatch adversary to `.worktrees/STORY-086` with absolute path (LESSON-1). Target: 3 strict-CLEAN passes (BC-5.39.001). After convergence: demo-recorder → pr-manager 9-step → security-reviewer + pr-reviewer (independent) → merge (STANDING MERGE AUTH). After STORY-086 merges: re-run Gate 3 (adversary) + Gate 5 (holdout) on patched develop. Only after re-gate passes: advance to Wave 5. BLK-002 OPEN until STORY-086 merges + gates re-pass. |

---

## Standing Process Rules

| ID | Rule |
|----|------|
| LESSON-1 | Adversary dispatches MUST pass the ABSOLUTE worktree path (`--cwd /Users/jmagady/Dev/slideforge/.worktrees/STORY-NNN`). |
| LESSON-2 | Canonical clippy gate: `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items`. Bare `-D warnings` misses pedantic lints. Also run `RUSTDOCFLAGS="-D warnings" cargo doc` for doc-link gates. |
| LESSON-3 | Re-run full pre-push gate (fmt + pedantic clippy + nextest) after EVERY commit to a feature branch before PR. |
| LESSON-5 | pr-manager CANNOT spawn sub-agents. Orchestrator dispatches security-reviewer + pr-reviewer independently. |
| LESSON-6 | RELAXED by STANDING MERGE AUTH: orchestrator may merge when CI-green + security-reviewer CLEAN + pr-reviewer APPROVE. |
| LESSON-7 | Per-story adversary convergence passes run SEQUENTIALLY (one at a time). Never parallelize passes of one story — parallel final passes can disagree, wasting a pass and creating adjudication ambiguity. Only parallelize across DIFFERENT stories. |
| LESSON-8 | Every pre-push gate AND implementer exit-gate MUST run the FULL canonical clippy (LESSON-2) + `cargo fmt --all -- --check`. Bare `-D warnings` misses pedantic + fmt and ships CI-RED PRs (happened twice on STORY-038). |
| LESSON-9 | When PR-level security/pr-reviewer findings are fixed AFTER per-story adversary convergence, the code diff changes — RE-RUN security-reviewer + pr-reviewer + wait for CI before merge. Do not assume prior convergence still holds. |
| LESSON-10 | For strict 3-CLEAN on compliance/extension stories: do a PROACTIVE exhaustive doc-vs-code + sibling-site (warn/error-propagation) consistency audit BEFORE final convergence passes, to avoid per-finding ping-pong (STORY-038 took 16 passes). |
| LESSON-11 | Stub/test-writer for EXTENSION stories often produces a high GREEN-BY-DESIGN ratio (prior-merged behavior is legitimate PRE-EXISTING-BEHAVIOR). But stricter new tests can expose REAL bugs in already-merged code — treat such finds as in-scope fixes for the compliance story. |
| LESSON-12 | Agents working in feature worktrees MUST NEVER commit to the `develop` branch. Twice this session a STORY-038 fix-burst commit accidentally landed on LOCAL develop (28636fae, 9730e6a3) — both unpushed and discarded via hard-reset to origin/develop. Mitigations: (a) worktree dispatches operate ONLY within `.worktrees/STORY-NNN` and never `git switch`/checkout develop in the main worktree; (b) after EVERY PR squash-merge, orchestrator verifies `git rev-parse develop == git rev-parse origin/develop` and resets local develop to origin if drifted, BEFORE creating the next story's worktree so new branches fork from the correct base; (c) fresh-session factory-worktree-health startup check MUST include this develop==origin/develop assertion. |
| LESSON-13 | UPSTREAM-DATA VERIFICATION (highest value). Before implementing a story that CONSUMES IR data produced by an upstream pipeline stage (e.g. layout::run threading, register content, sections), VERIFY the data is ACTUALLY threaded through the real pipeline (semantic Deck → eval → layout::run → LaidOutDeck), not merely present as a type. Instruct test-writer to confirm upstream-data availability FIRST and drive the REAL end-to-end path (construct semantic input, run the pipeline, assert) so any threading gap surfaces in the Red Gate, not adversary review. When the gap is large/architectural, get an architect assessment + human scope decision (expand vs split) BEFORE implementation. Evidence: STORY-039 (chart/diagram alt not threaded through layout::run — caught late, required cross-crate IR refactor); STORY-040 (slide-grouping sections data did not exist in IR at all — required split to STORY-082). |
| LESSON-14 | PRESENCE-VS-CONTENT TESTS. Acceptance tests that assert an artifact merely EXISTS (e.g. a ZIP part is present) can pass while the artifact is empty/orphaned/schema-invalid. Strengthen ACs/tests to assert CONTENT and VALIDITY: parse the real output and assert required child elements/attributes, relationship wiring (e.g. slide→notesSlide back-rel), and schema correctness — not just presence. When emitting hand-built OOXML, audit element-by-element against ECMA-376 CT content models (the canonical valid form usually exists elsewhere in the codebase — reuse/compare it). Evidence: STORY-040 presence-only tests passed while notesMaster was an empty stub, notes were orphaned (no slide→notesSlide rel), and grpSpPr was schema-invalid; adversary caught all three. |
| LESSON-15 | DEMO-EXAMPLE CANONICAL CLIPPY. demo-recorder example binaries (`crates/*/examples/*.rs`) are built by `--all-targets` and MUST pass the FULL canonical clippy including `-W clippy::missing_docs_in_private_items` (LESSON-2) — example private items need doc comments. The demo-recorder's own clippy check has twice omitted that flag → CI-red (STORY-039 too_many_lines; STORY-040 missing_docs). Orchestrator: after demo recording, run the FULL canonical clippy (with `-W missing_docs_in_private_items`) on the example before push, OR dispatch demo-recorder with the explicit full flag set. |
| LESSON-16 | PRE-PUSH GATE MUST MIRROR EXACT CI INVOCATIONS. Local per-story gates that run a narrower clippy (`cargo clippy -p <crate> --all-targets -- -D warnings`) and skip the docs gate will miss CI-catching defects. STORY-083 PR #57 first CI run FAILED on: (a) pedantic clippy escalated to deny caught `doc_markdown` + `unnecessary_literal_bound` + `uninlined_format_args` in the demo example binary; (b) `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` caught a broken intra-doc link `[Err(RegistryError::MissingSurface)]`. Fix commit bc52da1a was required before CI went green. **Remediation:** before push, orchestrator/implementer MUST run the EXACT canonical CI commands: `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items` AND `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` — especially when a story adds an example binary or new intra-doc links. Also note (process-gap): adversary pass-3 first attempt produced a false "non-deterministic file" report because it read the main-checkout path instead of the worktree path; re-dispatch with explicit worktree-absolute path discipline resolved it. Going-forward adversary dispatches in worktree stories MUST pin worktree-absolute paths (re-enforces LESSON-1). |
| LESSON-17 | INVERTED RED GATE ANTI-PATTERN (`#[should_panic]` PLACEHOLDER). A `#[should_panic]` test wrapping a `todo!()` call PASSES against stub code (the `todo!()` panic satisfies `should_panic`) and then BREAKS on correct implementation. This is the opposite of TDD Red Gate: tests must FAIL on stubs and PASS on implementation. Evidence: STORY-084 test-writer initially submitted `#[should_panic]` placeholders that gave false GREEN against `todo!()` stubs. Corrected before implementer dispatch by converting to behavioral `assert_eq!`/`assert!` tests that properly FAIL on stubs. **Rule:** test-writer MUST NEVER write `#[should_panic]` as a placeholder for behavioral correctness tests. `#[should_panic]` is valid only for testing deliberate panic-on-invalid-input paths — and those tests must also verify they FAIL on the stub. |

---

## Blocking Issues

| ID | Description | Opened | Status |
|----|-------------|--------|--------|
| BLK-001 | Alt-text enforcement non-functional end-to-end (Gap 2). | 2026-06-05 | RESOLVED — STORY-050 ADR-018 post-layout validation pass. |
| BLK-002 | Wave 4 gate FAILED — content-rendering gap (eval emits no slide-body ContentBlocks, for_eval.rs:342) + a11y strict-gate unsatisfiable (F-G3-CRIT-001). All exporters produce content-EMPTY output; holdout mean 0.56 / min_critical 0.30 below thresholds. Wave 4 does NOT advance to Wave 5 until remediated. | 2026-06-06 | OPEN — STORY-086 created (remediation scoped). Closes on STORY-086 merge + Wave 4 re-gate pass. |

---

## Open Follow-Ups (non-blocking)

| Item | Severity | Target |
|------|----------|--------|
| **VP-INDEX propagation: BC-5.01.001 v1.3 / BC-5.02.001 v1.6 updated VP language for the AltTextValidator post-layout path — VP-INDEX may require update to reflect new VP wording. Verify and propagate during STORY-086 delivery (vp_index_is_vp_catalog_source_of_truth policy).** | LOW (hygiene) | STORY-086 delivery |
| **OBS-1 (STORY-050 pass-1, expanded at Wave 4 gate): CanvasOverflowValidator + AltText pre-layout + LabelCheck are ALL functionally inert end-to-end — reads `Slide.blocks` which is always empty post-eval (for_eval.rs:342). Gate G3-HIGH-002: 3/5 validators dead (AltTextValidator pre-layout, CanvasOverflowValidator, LabelCheck). Gate G3-HIGH-003: LabelCheck COLOR_CODED_TYPES matches no registered slide types → WCAG 1.4.1 dead. These are BLOCKED on the content-threading gap fix (BLK-002). Target: resolve via Wave 4 remediation (Steps 1-2) + validator-hardening story before Phase 6.** | HIGH (a11y + WCAG correctness gap) | Wave 4 remediation (BLK-002) |
| **F-G3-CRIT-001 (Wave 4 gate adversary): Default strict build of ANY chart/diagram/image/screenshot/bio deck unconditionally fails E-A11-001 even with correct alt — because eval sets slide.blocks=vec![] (for_eval.rs:342) so thread_media_alt_into_frames never runs; decorative:true won't parse (E-PAR-002) so no remedy exists. Root cause: content-threading gap. Remediation: architect design (Step 1) + implementer fix (Step 2). IN PROGRESS.** | CRITICAL | Wave 4 remediation Steps 1-2 |
| **F-G3-HIGH-002 (Wave 4 gate adversary): AltText pre-layout validator + LabelCheck are inert (3/5 validators dead) — same root cause as F-G3-CRIT-001 content-threading gap. Fix via Steps 1-2.** | HIGH | Wave 4 remediation Steps 1-2 |
| **F-G3-HIGH-003 (Wave 4 gate adversary): LabelCheck COLOR_CODED_TYPES matches no registered slide types → WCAG 1.4.1 enforcement is dead letter. Fix via Steps 1-2.** | HIGH | Wave 4 remediation Steps 1-2 |
| **E-PAR-002 (Wave 4 gate adversary): decorative:true won't parse — missing parser rule. Fix in-scope with Steps 1-2.** | HIGH | Wave 4 remediation Steps 1-2 |
| **F-G3-HIGH-004 (Wave 4 gate adversary): for_eval.rs:336-341 comment misleading — implies blocks are populated but they are not. Fix in-scope.** | MED (misleading comment) | Wave 4 remediation fix-burst |
| **NOTE — Wave 4 story-spec frontmatter stale:** 19/21 Wave 4 story spec files have stale `status:` frontmatter fields (e.g. `draft`, `in-progress`, `ready` instead of `merged`). Canonical status = sprint-state.yaml + STORY-INDEX.md (both now corrected). Story-spec frontmatter is a hygiene gap, not a blocker. Hygiene follow-up anchored to a spec-steward pass before Phase 4 begins. | LOW (hygiene) | Spec-steward pass before Phase 4 |
| **EC-003/EC-002 (STORY-050 pass-4/5): story spec Edge Cases reference future `BuildError::DataFailed` + `E-DAT-001` (and `@include`/`@data` behavior) that are unimplemented upstream; E2E tests assert no-panic only. Sanctioned cross-story deferral — Wave 4 gate / future @data+@include stories must tighten EC-002/EC-003 to assert the real error variants once those features land.** | integration | wave-gate / @data story |
| ~~**[DRIFT-CRITICAL-1] PDF export non-functional since STORY-044.**~~ **RESOLVED via STORY-050 (030dec6c, 2026-06-06).** eval.rs title derivation fixed; PDF/UA-1 passes. Wave 4 gate must verify closure end-to-end. | RESOLVED | Wave 4 gate verification |
| ~~**[DRIFT-CRITICAL-2] Alt-text enforcement non-functional end-to-end.**~~ **RESOLVED via STORY-050 (030dec6c, 2026-06-06).** Post-layout validation pass (ADR-018) implemented; AltTextValidator now fires end-to-end. Wave 4 gate must verify closure. | RESOLVED | Wave 4 gate verification |
| ~~OBS-E (STORY-049 pass-4): STORY-050 E2E must include multi-slide deck with inline formatting + data binding routed through build() to close BC-5.02.002 EC-004 end-to-end.~~ **NOTE: Wave 4 gate holdout (Gate 5) found exporters produce content-EMPTY output (for_eval.rs:342 gap) — BC-5.02.002 EC-004 end-to-end coverage is effectively blocked until BLK-002 content-threading is fixed. OBS-E remains open pending BLK-002 remediation.** | content-threading blocked | Wave 4 remediation (BLK-002) |
| SEC-042-001 (CWE-400): docx section serializers no upper bound on items count | LOW | STORY-049 / layout hardening |
| SEC-001 (CWE-494, veraPDF): Docker `verapdf/cli:latest` not digest-pinned | MED | Before v1.0 / Phase 6 |
| SEC-003 (CWE-189): `emu_to_pt` i64→f32 precision loss | LOW | Phase 6 Kani |
| SEC-004: CI tee predictable temp path (self-hosted only) | LOW | Phase 6 |
| SEC-037-001: pptx InlineNode::Link (slide body) must adopt `is_safe_link_scheme` from link_safety.rs (SafeUrl guard now EXISTS in slideforge-pptx — wire slide-body Link path). CWE-601. | LOW | pptx link-rendering story / Phase 5 |
| SafeUrl shared newtype (pptx/pdf/html will hit same docx SEC-001 gap) | cross-exporter | Hardening story before Phase 6 |
| cold_budget flaky perf test (`slideforge-diagrams::cold_budget_under_200ms`): pre-existing intermittent CI-red risk (macOS timing jitter); not introduced by STORY-085 | LOW | STORY-080 (maintenance follow-up) |
| OBS-FU-HTML-REDIR (OBS-1 from STORY-085): HTML exporter Link/Xref URL scheme allowlist (CWE-601) — ANCHORED to STORY-046 AC-010 (added 2026-06-05). Required security acceptance criterion before STORY-046 ships. | security | STORY-046 AC-010 |
| E-PAR-021 cosmetic message format (nesting_depth_exceeded_msg raw byte offset) | minor | Message-cleanup follow-up |
| OBS-2 from STORY-085 (process-gap): AC-005 dog-fooding audit test uses filename/comment-prefix exemptions instead of #[cfg(test)] semantics — robustness nit with no current trigger; could false-positive if a production file gains an inline cfg(test) block with `<a:r` literals. Low priority. | process | self-improvement epic |
| OBS-FU-P1-A: DSL inner-quote round-trip | minor | Lexer follow-up |
| OBS-FU-P1-C: Link URL `)` truncation (Wikipedia disambiguation links) | minor | Link-parser follow-up |
| BC-1.14.001/002 still `subsystem: SS-TBD` | LOW | Next spec-hygiene pass |
| OBS-P6-001: PDF exporter ignores `opts.strict`/warnings | LOW | Post-STORY-045 wave-gate |
| DEF-041-P5-001: EC-002 paragraph-break preservation depends on STORY-035 evaluator (deferred) | integration | Wave-gate |
| STORY-081: slide-level inline markup (P0 for v1.0, Wave 5, EPIC-18, depends STORY-077) | P0 | Wave 5 |
| ~~BC-5.01.005 invariant 2 LaidOutDeck.lang~~ **RESOLVED 2026-06-04** — BC amended to v1.2 (human-authorized, rule 7); `deck.metadata.lang` is now the spec-canonical SoT; 5 stale story-spec refs corrected (STORY-017, STORY-039 ×3, STORY-041). No code change — impl was already correct. | CLOSED | — |
| SEC-040-002 (CWE-674/400): unbounded recursion in inline-node tree traversal — notes_slide.rs serialize/collect/extract + crate-wide pattern (e.g. lib.rs inline_nodes_to_plain_text). Needs bounded-traversal helper (depth cap). | LOW | Hardening story / Phase 6 |
| Control-char (U+0000–U+001F) XML-1.0 sanitization for notes + slide-body text (only lang path has validate_lang_for_xml). Crate-wide text-sanitization policy needed. | LOW | Phase 5 adversarial refinement |
| PR #56 description has mislabeled file paths (notes.rs/masters.rs/inline.rs vs actual notes_slide.rs/notes_master.rs) — cosmetic, merged PR; no action required. | trivial | — |
| Empty-string lang `lang ""` (Some("")) normalization to "en" — owned by slideforge-validate (SS-03), not exporter. | LOW | wave-gate / validator story |
| Diagram frame with no media rId emits no `<p:pic>` → its alt descr is not emitted (pre-existing STORY-037 media structure; latent a11y gap). | LOW | pptx diagram-media story / Phase 6 a11y audit |
| [process-gap] TDD Red-Gate citation discipline: per-test Red-Gate rationale must name a symbol PROVABLY on the production code path the test asserts (STORY-039 pass-1 F-039-PG1 — cited `AltTextEmbedder::embed` was not on the asserted path). | process | self-improvement epic (test-writer/implementer Red-Gate gate) |
| [process-gap OBS-4] vsdd-factory:adversary tool profile (Read/Grep/Glob) cannot execute cargo gates. LOCAL-pass dispatch instructions MUST NOT ask adversary to "run gates" — gate-execution-of-record is implementer exit-gate + orchestrator spot-verification. Follow-up: either grant adversary `exec` for LOCAL passes OR codify orchestrator-runs-gates as standing protocol. (surfaced STORY-085 pass 3) | process | self-improvement epic / tool-profile policy |

---

## Decisions Log

| Date | ID | Decision |
|------|-----|---------|
| 2026-06-06 | STORY-086-UNCERTAINTY-REMOVED | Before session restart, ran durability + uncertainty-removal on the expanded scope. Uncertainty-scanner found STORY-086/087/088 DIVERGED from real codebase (wrong file paths, nonexistent trait methods, mis-located work — NOT version staleness). Architect resolved D1-D5 (wave4-expanded-scope-uncertainty-resolution.md + ADR-019 v1.1): D1 TextTag→FrameContent routing is LAYOUT-side (layout.rs maps tagged ContentBlock::Text → FrameContent::Title/Subtitle/Body; exporters ALREADY route correctly via slide_serializer.rs, zero exporter change); D2 sweep ~168 AltText sites/15 files + ~18 TextBlock sites + slideforge-pdf/pptx; D3 real SlideType trait id()/lay_out()/render_hint()/validate(); D4 progress_bar geometry in lay_out(), severity_cards keyword gap; D5 FieldValue::List in parser/deck.rs+ast.rs, chumsky token-stream idiom. Stories corrected (086 v1.2, 087 v1.1, 088 v1.1). BCs 4.01/4.02/1.16 prose clarified, no version bump. No external research needed. Pinned versions + ooxmlsdk/DOCX-styles confirmed clean. Factory-artifacts committed + pushed (single atomic commit, TD-VSDD-053). |
| 2026-06-06 | STORY-086-PASS1 | Adversary LOCAL pass 1 found F-086-P1-CRIT-001 (Stage 2b skipped emitting alt=None media blocks, violating BC-1.16.001 PC-9/10/11 — to avoid E-A11-001 double-fire) + 3 MED (AC-007 bullets path untested; title not in PPTX title placeholder / DOCX Heading1 — TextTag mechanism never built). Architect adjudicated: CRIT-001 = code-conforms (emit alt=None blocks per BC + pre-layout validate() owns ONLY Shape; Chart/Image/Diagram post-layout-only → single-fire; ADR-018 v1.2 amendment). Human chose IMPLEMENT TextTag in STORY-086 (scope 13→21 pts; BC-4.01.001/4.02.001 → v1.2 specify tag routing; no BC amendment). Bullets list-literal DSL syntax gap → STORY-088 (Wave 5, 5 pts). TextTag follow-up absorbed into STORY-086. Streak reset 0/3. |
| 2026-06-06 | WAVE4-REMEDIATION-SETUP | Wave 4 gate FAILED remediation scoped + approved by human. ADR-019 (Stage 2b post-eval field-to-block threading pass) accepted; AltText::Unspecified state machine bundled. STORY-086 (Wave 4 remediation, 13 pts, P0) created — closes BLK-002/F-G3-CRIT-001/F-G3-HIGH-001/002. STORY-087 (Wave 5, color-coded slide types) created — closes F-G3-HIGH-003. BCs: BC-1.16.001 + BC-1.17.001/002/003 created; BC-5.02.001 v1.6; BC-5.01.001 v1.3; error-taxonomy v2.16; BC-INDEX 116 BCs / 78 P0; STORY-INDEX 87 stories / total 537 pts. |
| 2026-06-06 | WAVE4-GATE-FAIL | Wave 4 integration gate RAN and FAILED. Gate 1 PASS (3244/3246; 2 perf-timing flakes under local CPU contention — CI green on dedicated runners). Gate 2 SKIP (no DTU). Gate 3 (adversary) FAIL: 1 CRITICAL (F-G3-CRIT-001: default strict build of ANY chart/diagram/image/screenshot/bio deck unconditionally fails E-A11-001 even with correct alt — eval sets slide.blocks=vec![] so thread_media_alt_into_frames never runs; decorative:true won't parse so no remedy) + 3 HIGH (F-G3-HIGH-002: inert pre-layout validators broader than OBS-1 — AltText pre-layout + LabelCheck also inert = 3/5 validators dead; F-G3-HIGH-003: LabelCheck COLOR_CODED_TYPES match no registered slide types → WCAG 1.4.1 dead; F-G3-HIGH-004: misleading for_eval.rs:336-341 comment). Gate 5 (holdout) FAIL: mean 0.56 (<0.85), min_critical 0.30 (<0.60) — exporters produce content-EMPTY output: PPTX empty spTree, PDF blank pages (title only in metadata), DOCX drops slide titles. Root cause: eval emits no slide-body ContentBlocks (for_eval.rs:342). Mutation testing SKIP (0 facade stories). Consistency audit FAIL (4 blockers, 8 warnings — swept in this burst). BLK-002 opened. Wave 4 does NOT advance to Wave 5. Remediation plan: see NEXT ACTIONS Steps 1-5. |
| 2026-06-06 | STORY-050-MERGE | STORY-050 MERGED PR #61 (030dec6c, 2026-06-06T02:23:41Z). Title: "feat(slideforge): E2E integration suite + PDF/a11y/observability pipeline fixes". All CI green (21/21 aggregate + 4-platform tests, clippy, fmt, doctest, snapshots, bench, perf-smoke, visual-regression, pdf-ua1 ×2, supply-chain, audit, msrv, semgrep, panic-profile, pinning-audit, check-pdf-deps, docs). Security-reviewer APPROVE/CLEAN (SEC-050-001 CWE-116 fixed in-scope). pr-reviewer APPROVE/CLEAN. LOCAL cascade 5 passes, 3/3 strict-CLEAN (passes 3-4-5). 3 critical pipeline gaps in already-merged code FIXED in-scope: Gap 1 (PDF non-functional since STORY-044 — title derivation in eval.rs); Gap 2 (alt-text enforcement bypassed end-to-end — ADR-018 post-layout validation pass); Gap 3 (observability spans never renamed — info_span! with stage= field). ADR-018 v1.1, BC-5.02.001 v1.5, BC-5.01.001 v1.2, error-taxonomy v2.15, STORY-050 spec v1.3. DRIFT-CRITICAL-1 + DRIFT-CRITICAL-2 RESOLVED. Wave 4 = 21/21 COMPLETE. Next: vsdd-factory:wave-gate (human-approval checkpoint). |
| 2026-06-05 | STORY-050-SEC | PR #61 security review found SEC-050-001 (IMPORTANT, CWE-116: control-char injection into PDF XMP title via new Gap-1 title derivation, unguarded unlike lang path). FIXED in-scope: `validate_title_for_xmp` guard in slideforge-pdf (commit 8fb680be) + `PdfExportError::InvalidXmpTitle` error variant registered under E-EXP-003 in error-taxonomy (v2.15) + 3 load-bearing tests. Sibling sweep confirmed pptx/docx don't consume title. pr-reviewer 2 non-blocking items (post-layout locator test + enumerate cleanup) also closed in-scope (commits babe815d, 7f46c946). Re-review: security APPROVE/CLEAN, pr-reviewer APPROVE/CLEAN. PR #61 awaiting CI → merge → post-merge Wave 4 gate (STANDING MERGE AUTH). Worktree HEAD 7f46c946. |
| 2026-06-05 | STORY-050-CONV | STORY-050 LOCAL adversarial cascade CONVERGED. 5 passes total; passes 3-4-5 strict-CLEAN (3/3 per BC-5.39.001). Code HEAD `23481e1e` on `feature/STORY-050`. Cascade caught 2 real paper-fixes (pass 1: Gap-3 spans never renamed despite implementer claim; pass 2: AC-007 regression guards vacuously true). All Gap-1/2/3 fixes load-bearing. Workspace 3242/3242 (1 pre-existing cold_budget flake); all canonical gates GREEN (orchestrator-verified: clippy pedantic, fmt, doc, doctest, insta). Next: demo-recorder → pr-manager 9-step → security-reviewer + pr-reviewer (independent) → merge (STANDING MERGE AUTH). |
| 2026-06-05 | STORY-050-PASS2 | STORY-050 adversary pass 2 NOT clean (3 findings: F-050-P2-HIGH-001 + OBS-050-P2-001 + OBS-050-P2-002). HIGH-001: AC-007 regression guard was itself a paper-fix — `stage="<name>"` field never emitted by production `info_span!` calls; tests asserted vacuously. REMEDIATED: added `stage=` structured field to all 6 canonical `info_span!` calls; tests rewritten; load-bearing proof: rename `evaluate`→`eval` made AC-007 FAIL, restore→PASS (commit 5d34b5ae). OBS-001: Gap-1 title None-path zero coverage. REMEDIATED: 3 eval-crate unit tests added (Some-path, None-path no-title-slide, None-path title-slide-without-title-field), all call `eval_deck_with_variant` directly (commit 10112c18). OBS-002: `tempfile` unused in `slideforge` crate. REMEDIATED: removed from crates/slideforge/Cargo.toml; retained in slideforge-brand/data/pdf (real usage confirmed); story spec aligned to v1.3 (commit b253de47). Pre-push gate: 3242/3242 pass (commit 23481e1e). Streak: 0/3. Next: adversary pass 3 on feature/STORY-050 HEAD 23481e1e. Pass-2 report: `.factory/cycles/STORY-050/adversarial-reviews/adversary-STORY-050-pass-2.md`. |
| 2026-06-05 | STORY-050-PASS1 | STORY-050 adversary pass 1 NOT clean (6 findings: F-P1-CRIT-001/HIGH-001/HIGH-002/MED-001/OBS-2/OBS-5). All remediated: CRIT-001+HIGH-001 (Gap 3 span rename + subscriber filter, commit 79d1da2e); HIGH-002 (placeholder blocks removed, 38a1d3bb); MED-001 (tempfile drift, dbcf2bd9); OBS-5 (pre-layout early-return behavior, c4e4b26e); OBS-2 (title fallback anti-pattern — spec amendment, STORY-050 v1.2). ADR-018 → v1.1 (Decision 5a error-precedence rule). OBS-1 (CanvasOverflowValidator inert) tracked as open follow-up (pre-existing; ADR-018 Decision 4 deferral). Streak: 0/3. Next: adversary pass 2 on feature/STORY-050 HEAD c4e4b26e. Pass-1 report: `.factory/cycles/STORY-050/adversarial-reviews/adversary-STORY-050-pass-1.md`. |
| 2026-06-05 | STORY-050-GAP2-SPEC-BURST | Gap-2 spec burst LANDED (factory-artifacts). ADR-018 (post-layout validation pass) accepted. BC-5.02.001 → v1.5 (additive-defaulted `validate_post_layout` method; post-layout Stage 6b; new ECs + test vectors). BC-5.01.001 → v1.2 (alt-text enforcement moved to Stage 6b). Error taxonomy → v2.14 (E-A11-001 Stage 6b note). ARCH-INDEX.md + BC-INDEX.md updated. STORY-050 spec reconciled to spec_version 1.1 (ADR-018; real BuildError variant names; File Structure matched to disk). NEXT ACTIONS Steps 1-3 DONE — resume at Step 4 (implementer fix-burst in `.worktrees/STORY-050`). |
| 2026-06-05 | STORY-050-GAP2-AUTHORIZED | Human AUTHORIZED Gap-2 = Option A on 2026-06-05: implement a POST-LAYOUT validation pass so alt-text (and other ContentBlock-level) validators actually fire end-to-end. This is the chosen fix for the CRITICAL "alt required" accessibility guarantee being non-functional. BLK-001 → RESOLVED (Option A authorized). STORY-050 status → in-progress (resume-ready). Red Gate committed at `6bbe80fc` in `.worktrees/STORY-050` (branch `feature/STORY-050`, pushed to origin). Resume at NEXT ACTIONS Step 1 (architect Gap-2 design). |
| 2026-06-05 | STORY-050-RED-GATE | STORY-050 E2E Red Gate delivered (55 pass / 7 fail; committed at `6bbe80fc`, branch `feature/STORY-050`, pushed to origin). E2E suite surfaced CRITICAL pipeline gaps in already-merged code. Gap 1 (PDF NoDocumentTitle, in-scope, ~5-line fix in eval.rs). Gap 2 (alt-text validation bypassed end-to-end — see STORY-050-GAP2-AUTHORIZED). Gap 3 (observability events vs spans, in-scope). Gap 4 (API ergonomics, defer). Full architect gap analysis: `.factory/specs/story-050-gap-analysis.md`. |
| 2026-06-05 | STORY-049-MERGE | STORY-049 MERGED PR #60 (e6f7832d, 2026-06-05). 21/21 CI checks green (note: doctest+snapshots failure on default_registry rustdoc example fixed in commit 65ee62e6 before merge); security-reviewer APPROVE/CLEAN; pr-reviewer APPROVE. LOCAL adversary cascade CONVERGED (12 passes, 3/3 strict-CLEAN passes 10-11-12). develop SHA e6f7832d (60 merged PRs). Wave 4: 20/21 merged. Batch C COMPLETE. STORY-050 now UNGATED. |
| 2026-06-05 | STORY-049-CONV | STORY-049 LOCAL adversary cascade CONVERGED. 12 passes total; passes 10-11-12 strict-CLEAN (3/3 per BC-5.39.001). Code HEAD 402b28e6; workspace 3214/3215 pass (1 pre-existing cold_budget flake); all canonical gates GREEN (fmt, clippy pedantic, doc) — implementer/devops-verified. Findings fixed across passes 1-9: build() was non-functional (C1 inverted brand routing, C2 zero end-to-end coverage, C3 paper-fix strict test) → now works end-to-end (parse→eval→validate→layout→export, proven by load-bearing Ok(BuildOutput) test); strict defaults true (IMP-1); diagnostics preserve spans (HIGH-3); catch_unwind wired into build() + panic=unwind shipped profiles + CI guard (scripts/check-panic-profile.sh, F-PASS8-001); inject_lang_default (MED-C); extension from exporter.extension() (MED-D); doc/DSL-example fixes (pass 6); tracing stage spans (pass 9). All AC-001..AC-008 load-bearing. OBS-E (multi-slide/inline E2E through build()) anchored to STORY-050. Next: demo-recorder → pr-manager → STORY-050. |
| 2026-06-05 | STORY-049-P6-8 | STORY-049 adversary passes 6-8. Pass 5: strict-CLEAN (streak 1/3). Pass 6: 3 doc defects — DSL doctest syntax error, inverted brand-provider doc comment, missing validate stage in rustdoc example — FIXED commit 1c8af9a2; streak reset to 0/3. Pass 7: strict-CLEAN (streak 1/3). Pass 8: F-PASS8-001 (panic=unwind safety perimeter had no CI enforcement: Architecture Compliance Rule 4 stated "enforced by scripts/check-panic-profile.sh" but script did not yet exist) — FIXED by creating scripts/check-panic-profile.sh (exits 0 on compliance, non-zero on violation; both branches verified) + adding check-panic-profile CI job in .github/workflows/ci.yml wired into all-checks-pass (commit 70622461); streak reset to 0/3. Code HEAD: 70622461. Behavioral dimensions fully converged (clean passes 5+7); remaining findings peripheral (docs, CI enforcement). Passes 9-11 pending. |
| 2026-06-05 | STORY-049-R4 | STORY-049 adversary round 4 — ALL FIXED (commits bd1ce67b + d4727841). HIGH-A: strict=true happy path had no test — test added driving build() to Ok(BuildOutput) with strict enforcement. HIGH-B: BuildError::Export missing #[source] attribute breaking error-chain — added. MED-C: build_inner() did not call inject_lang_default() per slideforge-validate contract — wired. MED-D: BuildOutput.extension derived from format map key, not exporter.extension() — corrected. Workspace: 3214/3215 pass (1 pre-existing cold_budget flake). Streak: 0/3 strict-CLEAN. Pass 5 pending. |
| 2026-06-05 | STORY-049-R3 | STORY-049 adversary round 3 — ALL FIXED (commit 817ab0b5). IMP-1: strict defaulted false — contradicts spec (strict=true is default per CLAUDE.md and STORY-049 spec); corrected to default true. IMP-2: doc drift — stale docstring from round-1 era updated. Untested branches: NoBrandSource and UnknownFormat error paths now tested. panic=unwind added explicitly to [profile.dist] (was already in [profile.release] from round-1 fix but dist was missing). Workspace: ~3210 pass. Streak: 0/3 strict-CLEAN. Pass 4 pending. |
| 2026-06-05 | STORY-049-R2 | STORY-049 adversary round 2 — ALL FIXED (commit 7aa9fe67). C1: brand routing inverted — TomlFile path always returned Err due to incorrect variant check; corrected routing logic. C2: no end-to-end Ok test — paper fix: added test that actually drives build() to Ok(BuildOutput) via real .sf source + brand.toml→BrandProvider::Synthesize; test fails if build() never reaches Ok. C3: strict-validation test never called build() — test restructured to call build() and assert on BuildOutput. HIGH: layout rejected empty decks; diagnostics span-stripping. All fixed. Workspace: ~3207 pass. Streak: 0/3 strict-CLEAN. Pass 3 pending. |
| 2026-06-05 | STORY-049-R1 | STORY-049 adversary round 1 complete — ALL FIXED (commits f3d026f9 + 97faa709). C1: build() always returned Err(RegistryError::RegistryBuildFailed) due to brand-id path; now routes correctly through real registry. C2: build() had zero direct test coverage despite 3192 green tests; end-to-end tests added. C3: validate stage entirely absent from build(); now wired via registry.iter_validators() on public API; diagnostics preserved through BuildError::ValidationFailed. H1: catch_unwind in dispatch.rs was dead code (build() bypassed it); now all BrandProvider::load/Validator::validate/Exporter::export calls route through it. H2: Cargo.toml [profile.release]/[profile.dist] had panic="abort" silently disabling catch_unwind; corrected to panic="unwind". H3: gratuitous `unsafe impl Send` removed. 4 MED + 2 LOW also fixed. Spec delta: STORY-049 File Structure table updated; validate-stage/panic-profile/diagnostics/iter_validators prose added. Workspace: 3201/3202 pass (1 pre-existing cold_budget flake). Streak: 0/3 strict-CLEAN. Pass 2 pending. Validates per-story LOCAL cascade: build() could never succeed despite 3192 green tests. |
| 2026-06-05 | STORY-085 | STORY-085 MERGED PR #59 (e704e700, mergedAt 2026-06-05T08:00:18Z). 20/20 CI checks green; security-reviewer APPROVE/CLEAN; pr-reviewer APPROVE. LOCAL adversary cascade 9 passes, 3/3 strict-CLEAN (passes 7-8-9). 12 InlineNode variants × 3 formats bundled; notes_slide.rs dog-fooded through InlineFormat trait (BC-5.02.002 EC-004); display_text_is_empty shared SoT in slideforge-types. Wave 4: 19/21 merged. Batch C COMPLETE. STORY-049 now UNGATED. |
| 2026-06-05 | STORY-085-CONV | STORY-085 LOCAL adversary cascade CONVERGED. 9 passes total; passes 7-8-9 strict-CLEAN (3/3 per BC-5.39.001). All findings F-001 through F-085-P6-001 closed. Code HEAD 29903a1a; 3177 workspace tests pass; fmt + pedantic clippy + doc + grep-zero + nextest all GREEN (orchestrator-verified). Two anchored deferrals: (1) OBS-1 CWE-601 — DefaultInlineFormat intentionally omits URL scheme filtering (format-agnostic layer); anchored as STORY-046 AC-010 (required security gate before HTML exporter ships); (2) OBS-2 process-gap — AC-005 audit test filename-exemption nit, low priority, anchored to self-improvement epic. |
| 2026-06-05 | STORY-085-R5 | STORY-085 adversary round 5 — all prior findings (F-001 through I-1/OBS-1) re-verified closed. F-P5-001 (orphan-External-relationship invariant: non-empty Vec<InlineNode> that flattens to empty display text causes rId-registration/hlinkClick count mismatch) + OBS-P5-001 (drift cause: rId guard in notes_slide.rs and hlinkClick guard in default_formatter.rs used separate copies of the emptiness predicate): fixed STRUCTURALLY via shared `pub fn display_text_is_empty(nodes: &[InlineNode]) -> bool` in `slideforge-types/src/inline.rs` — single source of truth reachable by both consumers without new dependency edges. Story File Structure table updated to list `crates/slideforge-types/src/inline.rs` (Modify row + justification); Subsystem Anchor Justification prose extended with slideforge-types DAG rationale. Code HEAD 4283a17c; workspace 3172 tests pass. Convergence: still 0/3 strict-CLEAN (pass 5 had findings; pass 6 pending — do NOT mark STORY-085 done). |
| 2026-06-05 | STORY-085-R4 | STORY-085 adversary round 4 — all prior findings (F-001 through OBS-4) re-verified closed. I-1 (ADR-017 over-claimed F-006 scope as fully delivered in STORY-085): reconciled via ADR text correction — Question 3, F-006 Decision table row, and Consequence #4 updated to accurately describe the intentional two-story split (STORY-085 = registry-ready signature; STORY-049 = live registry resolution). Amendment Log added to ADR-017. OBS-1 (empty-text Link node orphan-rel): fixed in code (commit 75bd321d, guard added to render_with_context Link arm + rId-count-invariant tests). No BC or story-spec files modified. Code HEAD 75bd321d; workspace 3163 tests pass. Convergence: still 0/3 strict-CLEAN (pass 4 had findings; pass 5 pending — do NOT mark STORY-085 done). |
| 2026-06-05 | STORY-085-R3 | STORY-085 adversary round 3 — 3 MED footnote-variant findings fixed in code: (1) Markdown footnote output corrected to `[note]` per spec (was `[inline body]`); (2) mandated `tracing::debug!("Footnote marker numbering deferred")` deferral log added with load-bearing `tracing-test` assertions (TD-VSDD-059); (3) stale module/test docstrings corrected. OBS-1 baseline last-wins behavior tested and confirmed passing. Spec delta: `tracing-test = "=0.2.5"` dev-dep added to Library & Framework Requirements table in STORY-085. Code HEAD now 5a99962f; workspace 3161 tests pass. Convergence: still 0/3 strict-CLEAN (pass 3 had findings; pass 4 pending — do NOT mark STORY-085 done). Process-gap OBS-4 captured: adversary (Read/Grep/Glob profile) cannot execute cargo gates — LOCAL-pass dispatches must not instruct it to "run gates"; gate-execution-of-record is implementer exit-gate + orchestrator spot-verification (see Open Follow-Ups). |
| 2026-06-05 | STORY-085-R2 | STORY-085 adversary round 2 — 6 prior findings verified closed; new findings fixed in-scope: combined run-property accumulator (no silent drop of nested formatting in any direction, commit d9daad0a); stale docstring corrected; depth guard threaded through render_with_context link path. Spec deltas: AC-006 prose split by variant group (Plain/Bold/Italic byte-identical, 6 deliberately richer variants); VP-053 harness corrected to real MathNode fields (latex/display/span via MathNode::inline). HIGH-1 doc-gate was FALSE POSITIVE (orchestrator verified cargo doc passes; adversary conceded). Convergence: still 0/3 strict-CLEAN (pass 2 had findings; pass 3 pending — do NOT mark STORY-085 done). |
| 2026-06-05 | STORY-085-R1 | STORY-085 adversary round 1 complete — 6 findings fixed. F-001: ADR-017 Option A human-approved; BC-5.02.001 → v1.4 (Invariant 2 permits additive-defaulted methods; render_with_context authorized). F-002: AC-005 paper-fix test corrected to behavioral assertion. F-003: 12-variant byte-identity guard codified. F-004: BC-3.05.001 → v1.3.6 (HTML escape-all-interpolated, EC-009/EC-010 added); VP-053 created. F-005: nested-format depth guard. F-006: registry-ready threading anchored to STORY-049. F-008: module-boundary fitness test. BC-5.02.002 → v1.4. ADR-017 accepted. STORY-085 status: 0/3 strict-CLEAN (fixes applied, pass 2 pending). |
| 2026-06-05 | STORY-084 | STORY-084 MERGED PR #58 (801f351b) — 7 bundled SectionType impls (executive_summary + risk_register auto; methodology/scope/approval/appendix/glossary manual). BC-5.02.001 SectionType surface closed. BC-3.02.001 invariant 4 (notes-register exclusion) enforced on both auto types. CRIT-084-001 (risk_register notes-exclusion, mirrored layout/sections.rs HIGH-002): closed in scope. MED-084-002/003, OBS-084-004, LOW-084-005, OBS-084-006, LOW-084-A: all closed. 7-pass adversary cascade; strict-CLEAN passes 5/6/7. Security: APPROVE, CLEAN. Inverted-Red-Gate anti-pattern caught and corrected (`#[should_panic]` tests → behavioral assertions) before implementer dispatch — LESSON-17. `severity_cards` confirmed as contributing slide-type id via slide-types-catalog + sections.rs. PR also removed 4 stray .factory/ files from develop (hygiene). Batch C: 2/3 done at time of merge. |
| 2026-06-05 | STORY-083 | STORY-083 MERGED PR #57 (5aaa27d2) — PluginRegistryBuilder + RegistryError::MissingSurface + surface_count()/surface_names() + SURFACE_NAMES. BC-5.02.001 invariant 3 closed. 6-pass adversary cascade; strict-CLEAN passes 4/5/6. Security: APPROVE (2 LOW — CWE-400 bounded by CLI-startup; CWE-209 true negative). CI required fix commit bc52da1a (pedantic clippy: doc_markdown + unnecessary_literal_bound + uninlined_format_args in demo example; rustdoc broken intra-doc link MissingSurface). LESSON-16 added. Batch C: 1/5 done. |
| 2026-06-04 | LESSON-13-STORY-049 | STORY-049 LESSON-13 reconciliation (human-authorized 2026-06-04). STORY-049 found NOT implementation-ready: RegistryBuilder/surface enforcement absent from BC-5.02.001; SectionType (7 impls) + InlineFormat (12 impls) bundled ownership undefined; root crate vs owner-crate confusion in arch docs. Three decisions: (A) code-conforms-to-spec — RegistryBuilder + RegistryError::MissingSurface + surface_count/surface_names → STORY-083 (slideforge-plugin-api, 3 pts); (B) SectionType (7) + InlineFormat (12) bundled impls owned by slideforge-plugin-api → STORY-084 (3 pts) + STORY-085 (8 pts); (C) root crate = pipeline driver exposing build(), NOT owner-crate. Artifacts: ADR-016 authored; BC-5.02.001→v1.3 (invariant 3 + postcondition 2 counts corrected); BC-5.02.002→v1.3 (PC-5/EC-004 OOXML dog-fooding); plugin-architecture.md/ARCH-INDEX.md/crate-architecture.md corrected; reconciliation assessment at .factory/planning/story-049-reconciliation-assessment.md. Wave 4: 18→21 stories, 115→129 pts. Total: 81→85 stories, 497→511 pts. |
| 2026-06-04 | STORY-040 | STORY-040 MERGED PR #56 (869fb401) — Batch B pptx chain complete (037→038→039→040). Slide-grouping split to STORY-082 (human-authorized). SafeUrl guard (link_safety.rs is_safe_link_scheme, CWE-601) shipped. SEC-040-001 (URL safety + XML escaping for notes hyperlinks) verified via test — ooxmlsdk escapes correctly, no prod change required. |
| 2026-06-04 | SEC-039 | SEC-039-001 (CWE-116, MED) + SEC-039-002 (CWE-754, LOW) FIXED IN-SCOPE during STORY-039 PR review — validate_lang_for_xml rejects XML-1.0-illegal control chars in dc:language; loud tracing::error fallback for unexpected AltText variants. Neither deferred. |
| 2026-06-04 | BC-5.01.005-v1.2 | Invariant 2 amended: lang SoT corrected to `deck.metadata.lang` (`DeckMetadata.lang`) — human-authorized (SoT rule 7), architect-recommended Option 2. `LaidOutDeck` carries no lang field and will not gain one; all exporters read lang via `Exporter` trait `deck: &Deck` param. No code change — impl was already correct. 5 stale story-spec refs corrected (STORY-017, STORY-039 ×3, STORY-041); 1 residual test doc-comment (a11y_tests.rs:831) → STORY-040 drive-by. |

---

## Quality Bar (Non-Negotiable Gates)

Production-grade from day 1. Full table in CLAUDE.md. `#![forbid(unsafe_code)]`; zero `.unwrap()` outside tests; `clippy::pedantic`; `#![warn(missing_docs)]`; Kani+fuzz+mutants (Phase 6); WCAG AA; PDF/UA-1; < 500ms cold build; signed releases; SBOM; cross-platform macOS+Linux+Windows.
