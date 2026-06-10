---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-10
state_version: "1.2"
phase_1_approved: 2026-05-25
phase_2_approved: 2026-05-25
phase_1_convergence: "17 passes, 69 findings, 3/3 clean (passes 15-16-17)"
phase_2_convergence: "22 passes, 96+ findings, 3/3 clean (passes 20-21-22)"
prd_bcs: 117
prd_hs: 15
prd_vps: 15
prd_supplements: 4
spikes_resolved: 7
spikes_total: 7
total_stories: 93
total_points: 571
total_waves: 6
total_epics: 21
dtu_required: false
dtu_assessment: 2026-05-24
dtu_clones_built: n/a
dtu_services: []
wave_1_gate: "PASS 2026-05-27 — 3 gate passes, 11 findings fixed"
wave_2_gate: "PASS 2026-05-27 — 11 gate passes, 19 findings fixed, 3/3 clean (passes 9-10-11)"
wave_3_gate: "PASSED 2026-05-31 — PR #38 (7d266ad7); adversary pass 8 strict-CLEAN; holdout must-pass 5/5"
wave_4_gate: "PASSED 2026-06-07 — Gate 1 PASS; Gate 2 SKIP (no DTU); Gate 3 PASS (all 4 original findings closed; NEW-INT-001 image-alt RESOLVED PR #64); Gate 5 PASS (mean 1.00, min_critical 1.00; trajectory 0.56->0.86->1.00). BLK-002 CLOSED. develop 02d484cf (64 merged PRs)."
wave_4_merged: 23
wave_5_dep_prep: "MERGED PR #69 (3e3a978f) — [workspace.dependencies] centralized + ADR-022 major-version migrations: toml 1.1.2, sha2 0.11.0, criterion 0.8.2, notify 8.2.0, indexmap 2.14. INERT Wave-5 catalog entries added. Security CLEAN; CI green."
wave_5_status: "10 of 25 Wave-5 stories MERGED (STORY-072/082/088 PR#76/77/78 most recent). 15 remain (12 feature + 3 CI initiative STORY-091/092/093 NEXT). In-flight: STORY-081 PR #80 OPEN (MERGEABLE, HEAD 5c665cd0). PR-level adversary CONVERGED (P31-P34); security CLEAN; pr-reviewer APPROVE. PR #80 bench FAILED (infra — cold-build timeout; PR #81 fixes). PR #81 OPEN (ci/bench-timeout-cache-on-failure, MERGEABLE): bench PASSED; snapshots + linux-arm64 FAILED on infra flakes (re-runnable). NEXT: merge PR #81 first, then PR #80, then CI initiative. CONFIRMED root cause: 9.77 GB / 23 caches / 97.7% of ~10 GB limit."
develop_sha: "15838de1"
develop_pr_count: 79
open_prs: 2
error_taxonomy_version: "v2.28"
workspace_tests: "4083 pass / 20 skip / 0 fail (STORY-081 worktree HEAD 5c665cd0; post list-bullets×088 fix; includes STORY-072/082/088 tests)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## ZERO-CONTEXT RESUME — START HERE

**Project:** slideforge — data-reactive branded document platform (Rust, greenfield, Phase 3 TDD)
**Repository:** https://github.com/drbothen/slideforge (public) | **Default branch:** `main` | **Dev branch:** `develop`
**Workspace:** /Users/jmagady/Dev/slideforge

**Verify dev branch:** `git rev-parse develop` must equal `git rev-parse origin/develop`. Canonical SHA: `15838de1` (79 merged PRs, 2 open PRs: #80, #81).

**Factory worktree:** `.factory/` on branch `factory-artifacts`. Pushed to origin (human-authorized 2026-06-04; ongoing pushes authorized).

**Current position:** Phase 3, **Wave 5 IN PROGRESS**. 10 of 25 done (93 stories / 571 pts). 15 stories remain (12 feature + 3 CI initiative).

**STANDING MERGE AUTH:** Orchestrator MAY squash-merge any PR that is CI-green + security-reviewer CLEAN + pr-reviewer APPROVE, without re-asking human.

---

### Workstream A — STORY-081 merge (in-flight, complete first)

**Two open PRs — merge in order:**
1. **PR #81** (`ci/bench-timeout-cache-on-failure` → develop): bench timeout 20→60 min + `cache-on-failure: "true"`. bench PASSED (23m43s). `snapshots` + `test (linux-arm64)` failed on infra flakes (re-runnable — NOT code defects; `gh run rerun <run-id> --failed`). MERGE FIRST.
2. **PR #80** (`feature/STORY-081` → develop): STORY-081 Slide-Level Inline Markup. HEAD `5c665cd0`. PR-level adversary CONVERGED (P31-P34); security CLEAN; pr-reviewer APPROVE. `bench` FAILED (infra cold-build-timeout — PR #81 fixes). After PR #81 merges → rebase → force-push → re-run CI → squash-merge.

**ROOT CAUSE of PR #81/#80 flakes CONFIRMED:** GitHub Actions cache = **9.77 GB / 23 active caches / 97.7% of ~10 GB repo limit**. `test-linux-arm64` cache LRU-evicted → arm64 cold builds (22-60 min) + resource-starvation flakes. STORY-092 is the structural fix.

**Admin-merge rule (STANDING AUTH):** If `gh run rerun --failed` keeps flaking after 2 attempts on a diagnosed-infra-only failure (confirmed: no code changes, same class as STORY-088 precedent), orchestrator MAY admin-merge past infra flakes.

---

### Workstream B — CI performance initiative (NEXT PRIORITY, after STORY-081 lands)

Human directed 2026-06-10: deliver CI stories BEFORE remaining wave-5 feature stories.

**3 stories, dependency-ordered (EPIC-19, priority NEXT):**
- **STORY-091** — Tiered CI triggers + GitHub merge queue. Gates slow legs (full matrix, bench) off per-PR path → ~6-8 min feedback. Biggest per-PR wall-clock win.
- **STORY-092** — Cache reliability + disk headroom. Fixes the CONFIRMED 9.77 GB/23-cache thrash + `No space left on device` flakes; bumps rust-cache → v2.9.1 SHA `c19371144...`; cache-on-failure on all 11 sites; disk cleanup. HIGHEST-leverage flake fix.
- **STORY-093** — arm64 build-time reduction. mold linker (arm64 only) + `[profile.ci]` line-tables-only (`--cargo-profile ci`).

**Key confirmed facts (do not re-research):**
- arm64 runner is ALREADY native `ubuntu-24.04-arm` (NOT QEMU). Arm64 slowness is cache-eviction, not emulation.
- Action SHAs confirmed 2026-06-10 via git ls-remote: rust-cache `c19371144...`, free-disk-space `54081f13...`, setup-mold `9c9c13bf...`, install-action `fd2f5e3d...`. Verify at impl time (pinned SHAs may rotate).
- Portable playbook: `.factory/playbooks/tiered-ci-merge-queue.md`.
- Research sidecar: `.factory/planning/ci-speed-research.md`.

---

## DURABLE RESUME — SAME MACHINE OR FRESH CLONE

All branches are on origin (durable, machine-independent):

- `origin/factory-artifacts` — all `.factory/` state; ADR-023/024; BC-3.05.001 v1.4.3; BC-3.02.002 v1.5.1; BC-5.02.002 v1.5; 34-pass STORY-081 adversary reports. (Run `git -C .factory log -1` for current HEAD.)
- `origin/feature/STORY-081` @ `5c665cd0` — PR #80 OPEN (→ develop, MERGEABLE). LOCAL adversarial CONVERGED 3/3 (passes 28-29-30). PR-level CONVERGED (P31-P34). Security CLEAN. pr-reviewer APPROVE. Workspace 4083 pass / 20 skip / 0 fail.
- `origin/ci/bench-timeout-cache-on-failure` — PR #81 OPEN (→ develop, MERGEABLE). bench timeout 20→60 min + cache-on-failure fix.

**Same-machine resume:** `.factory/` and `.worktrees/STORY-081/` worktrees already exist on disk.
1. Run `vsdd-factory:factory-worktree-health`
2. Verify `git -C .worktrees/STORY-081 rev-parse HEAD` == `5c665cd0`
3. Check `gh pr checks 81` — if snapshots still FAILED, rerun that job; if linux-arm64 green → merge PR #81.
4. Continue at NEXT ACTIONS below.

**Fresh-clone (different machine) resume — exact commands:**
```
git clone https://github.com/drbothen/slideforge.git && cd slideforge
git fetch origin factory-artifacts feature/STORY-081 ci/bench-timeout-cache-on-failure
git worktree add .factory factory-artifacts
git worktree add .worktrees/STORY-081 feature/STORY-081
git rev-parse develop   # must equal origin/develop == 15838de1
```
Then read `.factory/STATE.md` → NEXT ACTIONS.

**Exact resume point:** Phase 3 / Wave 5. Two in-flight PRs. STORY-081 HEAD `5c665cd0`, PR #80 OPEN (MERGEABLE). PR #81 (bench fix) OPEN (MERGEABLE) — merge first. After PR #81 merges: rebase feature/STORY-081 → force-push → re-run PR #80 CI → squash-merge → post-merge burst + worktree cleanup. Then deliver STORY-091 → STORY-092 → STORY-093 (CI initiative, priority NEXT). Then resume remaining wave-5 feature stories.

---

## IN-FLIGHT WORKTREES — EXACT RESUME STATE

**1 active worktree. 2 open PRs (merge in order: #81 first, then #80).**

### CI-FIX — PR #81 (ci/bench-timeout-cache-on-failure → develop) — MERGE FIRST
- bench timeout 20→60 min + `cache-on-failure: "true"` for all build-heavy CI jobs.
- **CI status:** bench PASSED (23m43s). snapshots FAILED (transient `No space left on device` disk flake — re-run job, NOT a code defect). linux-arm64 pending.
- **NEXT:** `gh run rerun --failed <run-id>` on PR #81 snapshots job → all-green → squash-merge PR #81.

### STORY-081 — Slide-Level Inline Markup (EPIC-18, BC-3.05.001, 13 pts) — PR #80
- **Worktree:** `.worktrees/STORY-081` | **Branch:** `feature/STORY-081` | **HEAD:** `5c665cd0`. PR #80 OPEN → develop, MERGEABLE.
- **Adversary streak:** LOCAL **3/3 CONVERGED** (passes 28-29-30 strict-CLEAN). PR-level **CONVERGED** (passes P31-P34, all CRIT/HIGH/MED resolved). 34-pass cascade complete. Workspace 4083 pass / 20 skip / 0 fail.
- **PR #80 CI status:** 24/25 jobs PASS. `bench` FAILED — infra cold-build-cancel loop (20-min timeout, cache never saved on cancel; PR #81 raises limit to 60 min + cache-on-failure). NOT a code defect. `test (linux-arm64)` PASSED on PR #80 — FU-CI-ARM64-TEST-FAILURE CONFIRMED RESOLVED.
- **PR-level findings (all RESOLVED):** Passes P31-P34 complete; all CRIT/HIGH/MED resolved. Pre-PR integration gap: list-form bullets inline markup (FieldValue::InlinesList).
- **Security:** CLEAN. **pr-reviewer:** APPROVE.
- **Non-blocking follow-ups (post-merge; do NOT block PR #80):**
  - FU-S1-FONTDB-COUNT-VISIBILITY: `slideforge-diagrams::normalize::font_db_load_count()` is `pub` but test-only — should be `#[cfg(test)] pub(crate)`.
  - FU-S3-CAPTION-FIXTURE: `crates/slideforge/tests/fixtures/story-081-caption-markup.sf` references nonexistent `test-image.png` — add comment or use `decorative: true`.
  - FU-TD1-DEAD-FONT-COUNTER: `slideforge-pdf/src/font.rs` `LOAD_SYSTEM_FONTS_COUNT` is `#[allow(dead_code)]` — remove or wire a reader.
- **NEXT ACTION (after PR #81 merges):** Rebase `feature/STORY-081` onto new develop → `git push --force-with-lease` → re-run PR #80 CI (bench now warm + 60-min budget → passes) → STANDING MERGE AUTH → squash-merge PR #80 → post-merge state burst → worktree cleanup.

---

## WAVE 5 DELIVERY SUMMARY

**10 of 25 done (develop 15838de1, 79 PRs). +3 CI initiative stories (STORY-091/092/093) added 2026-06-10.**

- **STORY-089 MERGED** PR #68 (c722c28b): field-value type validation. FieldSchemaValidator live. error-taxonomy v2.24. ADR-020.
- **STORY-046 MERGED** PR #70 (fa85d113): Static HTML exporter (slideforge-html crate). P4 Composite Rendering Model. BC-4.03.003 v1.4. STORY-047 + STORY-081 UNLOCKED.
- **STORY-055 MERGED** PR #71 (cbebfd57): `slideforge build` CLI + miette diagnostics. STORY-057/058/060/064 UNLOCKED. STORY-056 UNLOCKED (←047+055 both now merged).
- **STORY-079 MERGED** PR #72 (2f6d5da4): slideforge-diagrams SVG DoS hardening. SEC-001+SEC-002 guards. BC-1.12.003 v1.3. error-taxonomy v2.26. LOCAL 3/3 (passes 3-4-5).
- **STORY-080 MERGED** PR #73 (e08f2f80): deflake cross-platform tests. cold_budget PERMANENTLY FIXED. LOCAL 3/3 (passes 5-6-7).
- **STORY-074 MERGED** PR #74 (3f7f99ed): brand-aware em sizing. `font_size_emu` (i64). DEFAULT_EM_IN_EMU removed from lib. LOCAL 3/3 (passes 11-13 of 13).
- **STORY-047 MERGED** PR #75 (95f23df3): preview WS server + CSP nonce security hardening. BC-4.03.004. axum WS. LOCAL 3/3 (passes 4-6). STORY-056 + STORY-048 UNLOCKED.
- **STORY-072 MERGED** PR #76 (2667987e): shape gradient fills — `FillSpec::Gradient { from, to }` end-to-end (parser→IR→layout→PPTX/PDF/HTML/DOCX). BC-3.04.001 v1.8. LOCAL 3/3 strict-CLEAN (passes 4-5-6). Security CLEAN; pr-reviewer APPROVE; CI 25/25 green. Rebased over STORY-074 em-sizing interaction (font_size_emu sibling sweep).
- **STORY-082 MERGED** PR #77 (c60cca36): PPTX slide-grouping sections — full pipeline DSL→parser→eval→layout→PPTX `<p14:sectionLst>`. BC-4.01.003 v1.4 + BC-1.14.003. SlideSectionEntry in slideforge-types (re-exported via layout). Deterministic RFC4122 v5 GUIDs. E-PAR-023 exit 1, W-PAR-002 duplicate warning. CONVERGED 3/3 strict-CLEAN (passes 16-17-18) after 18-pass cascade. Findings caught+fixed: adjacent-duplicate section merge (HIGH, per-instance-id fix), SEC-100/CWE-116 section-name sanitization (HIGH), vacuous AC-010 tests (TD-VSDD-059), E-PAR-023 exit 2→1 spec (BC v1.4, taxonomy v2.28), sha2 pin =0.11.0 spec. Security CLEAN (1 LOW pre-existing, 1 SUGGESTION); pr-reviewer APPROVE (2 non-blocking OBS); CI 25/25 green. Rebased over STORY-074/072 (font_size_emu + slide_sections struct-field sibling sweep). Follow-up: FU-082-SEC-S001-NUL-CALLSITE-TEST.
- **STORY-088 MERGED** PR #78 (15838de1, ADMIN OVERRIDE): bullets list-literal DSL `bullets: ["A","B"]` across all 4 FieldValue value positions (field-value/@var/set-rule AC-012/variant-vars AC-013) via shared `list_literal_elements` combinator; E-PAR-024 (non-string element / nested list, non-recursive O(1) depth tracker); dsl-spec v1.1, error-taxonomy v2.28. CONVERGED 3/3 (passes 8-9-10) after 10-pass cascade — human-approved scope expansion + caught/fixed a self-introduced CRITICAL recursion DoS + quad-duplication→shared-combinator refactor. Security CLEAN; pr-reviewer APPROVE; 19/20 CI green; linux-arm64 nextest failure DEFERRED (FU-CI-ARM64-TEST-FAILURE) via admin merge per human. FU-088-BC10102-ANCHOR registered (pre-existing BC-1.01.002 title/anchor mismatch, spec-steward, non-blocking).
- **DEP-PREP MERGED** PR #69 (3e3a978f): [workspace.dependencies] centralized + ADR-022 migrations done.
- **CI-FIX MERGED** PR #79: ci.yml test-matrix `timeout-minutes` 30→75 + `cache-on-failure: "true"` (Swatinem/rust-cache). Roots out linux-arm64 cold-build-timeout self-perpetuating loop. ci-workflow-analyzer corrected initial no-op (`save-always` invalid for rust-cache → `cache-on-failure`).
- **CI-FIX OPEN** PR #81 (ci/bench-timeout-cache-on-failure → develop): bench `timeout-minutes` 20→60 + `cache-on-failure: "true"`. bench PASSED (23m43s); snapshots FAILED (transient disk flake, re-runnable); linux-arm64 pending. NEXT: rerun snapshots → merge.

**15 stories remain (12 feature + 3 CI initiative: STORY-091/092/093 NEXT). 1 active worktree (see IN-FLIGHT section above).**

**HELD (next batch after in-flight merges):**
- STORY-057/058/064 (slideforge-cli same-crate conflict — deliver AFTER this batch, serialized)
- STORY-056 (←047 now MERGED — UNBLOCKED; may start after in-flight batch)
- STORY-048 (←047 now MERGED — UNBLOCKED)
- STORY-060/061 (FU-SEC-001-GIT2-OPENSSL must be resolved FIRST)

**OPEN FOLLOW-UPS:**
- **FU-DIAGNOSTIC-FIELD-PINNING** [process-gap, lessons-codification]: "Diagnostic emitted" tests must assert the rendered message and/or distinguishing struct field values matching the BC canonical test vector — not just the error code. AC-006 test asserted presence-by-error-code only; let F-P27-MED-001 (slide_title carrying stripped text instead of markup form) survive 26 passes undetected. Also: SourceSpan threading for eval diagnostics is a systematic gap — EvalError::InlineMarkupInTitle uses SourceSpan::default() pending STORY-012 SourceMap. Source: OBS-P27-001.
- **FU-BC-ACCURACY-AUDIT** [process-gap, lessons-codification]: BC postconditions were authored as ideal/aspirational behavior (Footnote PPTX, Math PPTX/DOCX/PDF) that drifted from the deferred v1.0 implementation. BC authoring MUST describe ACTUAL current behavior + explicit deferral citations (story ID + code line); re-anchor/amendment bursts must audit ALL variant clauses (all 12 forms × all 5 surfaces) against code, not just the changed story's forms. EXTENDED (OBS-P25-004/005): scope must also include diagnostic-type liveness (grep every BC-named LayoutWarning/EvalError/LayoutError type against ALL construction sites — not just doc comments; a declared-but-never-constructed variant is dead code and a type-drift gap) + the title-constraint section (verify struct field counts, error codes, severity labels against live error.rs definition). EXTENDED (demo-surfaced drift 2026-06-10): scope must also require STORY-SPEC propagation — when a BC is reconciled/corrected, the anchored story spec's per-form/per-diagnostic descriptions must be swept in the SAME burst; the BC v1.4.2/v1.4.3 cascade corrected DOCX Code (rStyle→RunFonts), the title diagnostic type (LayoutWarning→EvalError), EC-008 a:highlight form, and Math v1.0 degraded behavior — but the story spec was not updated until the demo step caught it (4 categories of drift survived undetected). Source: F-P17-002 (Math) + F-P24-MED-001 (Footnote/Code/Strike/Highlight/Link/Xref) + OBS-P25-004/005 (dead LayoutWarning variant + type drift through 24 passes) + demo-surfaced story-spec drift (STORY-081 spec v1.4→v1.5). Target: Standing Process Rules / lessons.
- **FU-LINK-SCHEME-CONSISTENCY** [human/architect adjudication]: Cross-surface unsafe-scheme policy alignment. DOCX hard-errors on unsafe-scheme InlineNode::Link (fatal ExportError::ValidationError, SEC-001/CWE-601); PPTX and HTML silently degrade to plain run/span + tracing::warn!. An all-formats build fails (driven by DOCX); per-format PPTX or HTML builds individually succeed. Design question: align all surfaces to hard-error (defense-in-depth), all to degrade (author-friendliness), or keep current split (DOCX security-strict, PPTX/HTML lenient)? This is a security policy decision. Do NOT resolve in a BC fix burst. Escalate to human/architect at next relevant story or security review. Source: F-P25-LOW-001 / BC-3.05.001 EC-013 Open Questions.
- **OBS-P24-001-REBASE** [rebase-readiness, non-blocking]: Rebase cbebfd57→15838de1 will hit shared-type seams: FieldValue::Inlines (types/slide.rs), FrameContent::SubtitleInlines + TextRun→Vec<InlineNode> (layout/types.rs) vs develop's STORY-072/082/088 (gradient/sectionLst/bullets). Exhaustive non-wildcard matches fail-loud — guaranteed manual-merge surface at PR step. Flag for pr-manager rebase dispatch.
- **FU-REANCHOR-COMPLETENESS-GREP** [process-gap, lessons-codification]: Any re-anchor / stale-identifier sweep MUST run a completeness grep covering ALL FILE TYPES (not `--include=*.rs`) as a required exit gate before declaring done; verified by adversary. The completeness grep MUST include snapshot/fixture/filename checks (`find -iname`) + `insta --unreferenced=reject` in the exit gate. Source: OBS-P20-1 (Pass-19 sweep declared complete while 5+ files retained old anchor); EXTENDED by Pass-22 finding — the Pass-20 .rs-scoped completeness grep missed a tracked `.snap` file carrying the old BC_3_02_002 anchor; git-rm'd + committed 2e13fb6e; all-filetype grep + insta --unreferenced=reject confirm zero residuals. Target Standing Process Rules / lessons.
- **FU-VP-043-NOTES-PATH** (architect/formal-verifier, Phase-6, non-blocking): VP-043 ("all 12 inline variants produce distinct non-empty XML in PPTX") now implicitly spans body AND notes via the unified ADR-024 engine (BC-3.05.001 v1.4.0 amendment). Assess whether a separate notes-path proof/snapshot variant is needed; update VP-INDEX + verification-coverage-matrix under the vp_index source-of-truth policy. Source: PO BC-3.05.001 v1.4.0 amendment.
- **FU-CI-ARM64-TEST-FAILURE** (RESOLVED — commits 49b1fc8e + f7c26fba; CONFIRMED CLOSED — PR #80 `test (linux-arm64)` PASSED): Root cause was two wall-clock timing gates flaking on the slow emulated arm64 runner. Both converted to deterministic `font_db_load_count == 1` assertions; double-load defect in `slideforge-pdf/src/font.rs` fixed.
- **FU-CI-SPEED** [devops-engineer + ci-workflow-analyzer; implement as PR(s) AFTER STORY-081 merges]: Make CI faster. Observed pain: arm64 QEMU-emulated takes 22min (long pole); bench cold-build-cancel loop (now fixed by PR #81 but structurally recurring); `No space left on device` disk flakes forcing full re-runs; 20-crate workspace rebuild on every job. Implement after STORY-081 merges to avoid churning in-flight CI:
  - **Tier 1 (biggest wins):** (a) Replace QEMU-emulated `linux-arm64` with GitHub native `ubuntu-24.04-arm` hosted runners (GA 2025) — native ≈ 3-5x faster (~6-7 min vs 22 min). (b) Cache audit: EVERY build-heavy job uses `Swatinem/rust-cache` with `cache-on-failure: "true"` + sensible shared-key — the bench loop proved one missing `cache-on-failure` = perpetual cold builds. (c) Disk-space hardening: add `jlumbroso/free-disk-space` step (reclaims ~10-20 GB of preinstalled toolchains) or move to larger runner — kills the transient `No space left on device` flakes on snapshots/build-heavy jobs. (Absorbs FU-CI-RUNNER-DISK below.)
  - **Tier 2 (structural):** (d) Build-once: `cargo nextest archive` per platform → run partitions, instead of every test job rebuilding from scratch; or share `target/` via cache key. (e) sccache (GHA-cache or S3 backend) for compiled-artifact reuse across jobs. (f) Faster linker (mold on Linux, lld on Windows). (g) nextest partition/shard heavy test crates across runners.
  - **Tier 3 (tuning/policy):** (h) CI build profile tuned for compile speed (high codegen-units, `debug = "line-tables-only"`; keep `CARGO_INCREMENTAL=0`). (i) Tiered triggers: x86_64 + fast checks on every push; full multi-platform matrix + bench on merge-queue/develop/nightly/label. (j) Larger runners for bottleneck build/test jobs.
- **FU-CI-RUNNER-DISK** (absorbed into FU-CI-SPEED Tier 1c): transient `No space left on device` on snapshots and build-heavy jobs — disk-space hardening step or larger runner.
- **FU-DIAGRAMS-COLD-BUDGET-TIMING-GATE** (RESOLVED — commits 49b1fc8e + f7c26fba): `crates/slideforge-diagrams/tests/cold_budget.rs` flaky wall-clock gate eliminated. Root cause: real double `load_system_fonts` call per `resolve_font_set` in `slideforge-pdf/src/font.rs` (called in both `resolve_font_set` AND `resolve_regular_face`). Fixed to load once and share `&fontdb::Database`. Both timing gates converted to deterministic `font_db_load_count == 1`; perf budget remains gated by criterion benches.
- **FU-NOTES-HIGHLIGHT-ATTR** (RESOLVED — fixed IN STORY-081 commit 34c128b2; NOT deferred to STORY-040): notes-path `emit_run` now emits `<a:highlight><a:srgbClr val="FFFF00"/></a:highlight>` child element; `highlight="yellow"` attribute form eliminated; Red-Gate test confirms child present + attribute absent.
- **FU-PPTX-DUAL-RUN-GENERATOR** (RESOLVED by ADR-024 — 2026-06-09; CONFIRMED RESOLVED by Pass 17 Axis-B verification): Root cause of 3 STORY-081 findings (ADV-P11-HIGH-001, F-P15-HIGH-001, F-P16-M1). The two generators are now ONE engine (`render_inline_nodes_to_runs` in `slideforge-plugin-api`); divergence impossible by construction. Cycle-closing checklist S-7.02 process-gap satisfied — resolved, not deferred. No follow-up story required; architectural unification is complete.
- **FU-STORY-045-HTML-MATHML** (non-blocking; STORY-045 is the tracked vehicle): HTML Math full MathML `<math>` rendering deferred to STORY-045. BC-3.05.001 PC-4 v1.4.1 codifies the v1.0 `<code class="math">{escaped LaTeX}</code>` fallback as the CURRENT behavior; MathML is an explicit non-silent deferral. Source: F-P17-002 Math half (spec reconciliation, not impl defect). No action required in STORY-081 scope.
- **F-085-P6-001 SUPERSEDED** [parity ruling, Pass 14]: F-085-P6-001 "top-level-only hlinkClick" is SUPERSEDED for nested links by the Pass-14 cross-format parity ruling. Body + notes now emit `<a:hlinkClick>` for wrapped links (`Bold([Link])`, `Italic([Link])`, etc.); orphan-rel invariant preserved. The 5 F-085-P6-001 tests in STORY-085 notes behavior were updated at d73bbc64 to expect 1-rel/1-click for wrapped links. F-085-P6-001 docstring carve-out removed. FU-PPTX-DUAL-RUN-GENERATOR RESOLVED (ADR-024).
- **FU-088-BC10102-ANCHOR** (spec-steward; non-blocking): pre-existing BC-1.01.002 H1 title/anchor mismatch.
- **FU-EXIT-GATE-DISTINGUISHING-OUTPUT** (process-improvement; source OBS-1 Pass-3 + OBS-P04-001 Pass-4 + OBS-P05-001 Pass-5 + ADV-P06-MED-001 Pass-6 + PROCESS-NOTE Pass-8 + ADV-P11-HIGH-001 Pass-11 + orchestrator EC-004 catch Pass-11 validation + F-P13-002 Pass-13): Anti-pattern recurred 9th time. LESSON codification: "MED distinguishing-assertion findings must close with assertions that FAIL when the form's unique output is removed; presence-of-display-text is vacuous; COMBINED-form cells (bold+italic, bold+link, nested) must have per-exporter assertions". Exit-gate rule must verify (1) non-test caller consumes a dispatch fn's DISTINGUISHING output; (2) multi-span-same-line positional assertion; (3) when drawable font present but raw bytes absent, non-silent diagnostic; (4) BOTH measure path AND draw path select slots via THE SAME helper; (5) multi-exporter "rich rendering" closures MUST be verified PER EXPORTER; (6) COMBINED-form compound nodes must have load-bearing assertions on EVERY exporter surface. Target: lessons-codification / Standing Process Rules.
---

## CURRENT POSITION

Phase 3, **Wave 5 IN PROGRESS** (develop `15838de1`, 79 merged PRs). 10 of 25 done. 15 stories remain (12 feature + 3 CI initiative STORY-091/092/093). 93 stories / 571 pts total.

- Active worktrees: 1 — STORY-081 in `.worktrees/STORY-081` on `feature/STORY-081` HEAD `5c665cd0`. PR #80 OPEN (→ develop, MERGEABLE). PR-level adversary CONVERGED (P31-P34); security CLEAN; pr-reviewer APPROVE. Open PRs: 2 (PR #80 + PR #81).
- Workspace: 4083 pass / 20 skip / 0 fail (STORY-081 worktree HEAD 5c665cd0; cargo deny PASS; cold_budget PERMANENTLY FIXED; FU-CI-ARM64-TEST-FAILURE CONFIRMED RESOLVED per PR #80 linux-arm64 PASS).
- Uncertainty pass: COMPLETE. ADR-022 dep-centralization: DONE. ADR-008 P4 amendment: DONE. ADR-021 async runtime: DONE.

---

## NEXT ACTIONS

**EXACT ORDERED TASK LIST (execute in sequence; do not parallelize):**

### Phase A — STORY-081 merge (complete first)

1. **Rerun PR #81 infra flakes:** `gh run rerun --failed <PR-81-run-id>` (run id from `gh pr checks 81`). `snapshots` failed on `No space left on device` (disk infra flake); `test (linux-arm64)` failed on runner-lost-comms. Both re-runnable. PR #81 has zero code changes. Confirm all jobs green. If re-runs keep flaking after 2 attempts → admin-merge per STANDING AUTH (same precedent as STORY-088).
2. **Merge PR #81:** STANDING MERGE AUTH → squash-merge PR #81 → develop (bench timeout 20→60 min + cache-on-failure lands repo-wide; warms bench cache for subsequent PRs).
3. **Rebase STORY-081 onto new develop:** `git -C .worktrees/STORY-081 rebase origin/develop` → resolve mechanical conflicts → `git push --force-with-lease origin feature/STORY-081`.
4. **Re-run PR #80 CI:** bench now has 60-min budget + warm cache → passes. Confirm all 25 jobs green. `test (linux-arm64)` already passed on prior run (FU-CI-ARM64-TEST-FAILURE CONFIRMED RESOLVED). Run PR #80 alone (avoid concurrent-run cache thrash with other PRs).
5. **Squash-merge PR #80:** STANDING MERGE AUTH → squash-merge `feature/STORY-081` → develop.
6. **Post-merge state burst:** state-manager updates STORY-INDEX / dependency-graph / STATE.md to MERGED; records new develop SHA. LESSON-18: `git fetch && git merge --ff-only origin/develop` in any active worktrees.
7. **Worktree cleanup:** Remove `.worktrees/STORY-081`; prune `feature/STORY-081` branch.

### Phase B — CI performance initiative (NEXT PRIORITY, after STORY-081 lands)

Human directed 2026-06-10: CI stories BEFORE remaining wave-5 feature stories.

8. **Deliver STORY-092 (cache + disk) — consider delivering first within CI batch:** This is the fix for the CONFIRMED 9.77 GB/23-cache thrash root cause. Highest-leverage flake fix. Dispatch devops-engineer + ci-workflow-analyzer. Implementation: rust-cache → v2.9.1 `c19371144...`; cache-on-failure on all 11 CI sites; free-disk-space `54081f13...` step; cache key audit. Story spec: `.factory/stories/stories/STORY-092-ci-cache-reliability-disk.md`.
9. **Deliver STORY-091 (tiered triggers + merge queue):** Biggest per-PR wall-clock win (~6-8 min feedback loop). Dispatch devops-engineer + ci-workflow-analyzer. Story spec: `.factory/stories/stories/STORY-091-ci-tiered-triggers-merge-queue.md`. Playbook: `.factory/playbooks/tiered-ci-merge-queue.md`.
10. **Deliver STORY-093 (arm64 build-time):** mold linker (arm64 only, `setup-mold` `9c9c13bf...`) + `[profile.ci]` `debug = "line-tables-only"` (via `--cargo-profile ci`). Story spec: `.factory/stories/stories/STORY-093-ci-arm64-build-time.md`.

### Phase C — Resume remaining Wave-5 feature stories (after CI initiative)

11. Next feature batch: STORY-056/048 (←047 MERGED — UNBLOCKED), STORY-057/058/064 (serialize cli, same-crate conflict), STORY-060/061 (FU-SEC-001-GIT2-OPENSSL must resolve first).

**Diagnostic commands:** `gh pr checks 80` / `gh pr checks 81` / `gh run rerun --failed <run-id>`

**RESUME PROCEDURE (zero context):**
1. Run `vsdd-factory:factory-worktree-health`
2. Verify `git rev-parse develop` == origin/develop == `15838de1`
3. Verify `git -C .worktrees/STORY-081 rev-parse HEAD` == `5c665cd0`
4. Check `gh pr checks 81` — start at Phase A step 1 above.

**PER-STORY DELIVERY SEQUENCE (BC-5.39.001):**
adversary LOCAL 3-CLEAN (passes run SEQUENTIALLY) → demo-recorder per-AC → rebase onto develop `15838de1` → push → pr-manager 9-step (orchestrator dispatches security-reviewer + pr-reviewer per LESSON-5) → STANDING MERGE AUTH: CI-green + security CLEAN + pr-reviewer APPROVE → squash-merge → state-manager post-merge burst → worktree cleanup → LESSON-18 sync check.

**APPLY LESSON-21 to EVERY story exit gate** (prevents post-convergence CI-fix cycles): nextest + `cargo test --workspace --all-features` (shared-process) + `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` + cross-platform filesystem test discipline.

---

## STANDING AUTHORIZATIONS

**STANDING MERGE AUTH (granted 2026-06-02):** Orchestrator MAY squash-merge any PR that is CI-green + security-reviewer CLEAN + pr-reviewer APPROVE, without re-asking the human. Revocable by human.

**factory-artifacts PUSHED to remote (origin/factory-artifacts) — human-authorized 2026-06-04; ongoing pushes authorized.**

---

## Phase Progress

| Phase | Status | Key Output |
|-------|--------|-----------|
| Pre-pipeline | DONE | Toolchain + LLM + MCP preflight (2026-05-23) |
| Market intelligence | DONE 2026-05-23 | GO with medium confidence |
| Planning (25 DSL decisions) | DONE 2026-05-24 | q1-q25 docs + 14 research threads + 7/7 spikes resolved |
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (116 BCs, 15 HS, 4 supplements) + arch (18 ADRs, 15 VPs, 20 crates) + UX spec. 17 passes, 69 findings, 3/3 clean. |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 89 stories, 21 epics, 6 waves, 553 pts (baseline). Now 93 stories / 571 pts after CI initiative +3. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3/4 GATE PASSED. Wave 5: **10/25 done** (STORY-089 PR#68, STORY-046 PR#70, STORY-055 PR#71, STORY-079 PR#72, STORY-080 PR#73, STORY-074 PR#74, STORY-047 PR#75, STORY-072 PR#76, STORY-082 PR#77, STORY-088 PR#78). 15 remain (12 feature + 3 CI: STORY-091/092/093 NEXT). 1 in-flight worktree. | Per-story delivery |
| Phases 4-7 | NOT STARTED | Holdout / Adversarial / Formal Hardening / Convergence |

---

## Session Resume Checkpoint

**Wave 5 IN PROGRESS. develop 15838de1 (79 merged PRs, 2 open PRs). CI initiative NEXT (human-directed 2026-06-10). STANDING MERGE AUTH active.**

| Field | Value |
|-------|-------|
| **Date** | 2026-06-10 |
| **develop SHA** | `15838de1` (79 merged PRs; origin/develop confirmed; 2 open PRs: #80, #81) |
| **Active worktrees** | 1 — STORY-081 in `.worktrees/STORY-081` on `feature/STORY-081` HEAD `5c665cd0`. |
| **PR #81 state** | OPEN (ci/bench-timeout-cache-on-failure → develop, MERGEABLE). bench PASSED (23m43s). snapshots + linux-arm64 FAILED on infra flakes (re-run or admin-merge). MERGE FIRST. |
| **PR #80 / STORY-081 state** | OPEN (→ develop, MERGEABLE). HEAD `5c665cd0`. LOCAL adversarial CONVERGED 3/3 (passes 28-29-30). PR-level adversary CONVERGED (P31-P34, all CRIT/HIGH/MED resolved). Security CLEAN. pr-reviewer APPROVE. bench FAILED (infra cold-build-timeout — PR #81 fixes). linux-arm64 PASSED. Workspace 4083 pass / 20 skip / 0 fail. |
| **Workspace tests** | 4083 pass / 20 skip / 0 fail (HEAD 5c665cd0; cargo deny PASS; FU-CI-ARM64-TEST-FAILURE CONFIRMED RESOLVED) |
| **Cache situation** | CONFIRMED ROOT CAUSE: 9.77 GB / 23 active caches / 97.7% of ~10 GB limit. LRU-evicted arm64 cache → cold builds + resource-starvation flakes. STORY-092 fixes this. |
| **CI initiative** | STORY-091/092/093 (EPIC-19, NEXT) specs written, uncertainty resolved, playbook ready. Deliver after STORY-081 merges. Consider STORY-092 first (highest-leverage flake fix). |
| **factory-artifacts** | Pushed to origin. Fresh sessions: clone + `git fetch origin factory-artifacts feature/STORY-081 ci/bench-timeout-cache-on-failure` + `git worktree add .factory factory-artifacts`. |
| **RESUME INSTRUCTION** | Phase A: rerun PR #81 infra flakes → merge PR #81 → rebase STORY-081 → re-run PR #80 CI → squash-merge → post-merge burst + cleanup. Phase B: deliver STORY-091/092/093 (CI initiative). Phase C: remaining wave-5 feature stories. |

---

## Standing Process Rules

| ID | Rule |
|----|------|
| LESSON-1 | Adversary dispatches MUST pass ABSOLUTE worktree path (`--cwd /Users/jmagady/Dev/slideforge/.worktrees/STORY-NNN`). |
| LESSON-2 | Canonical clippy: `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items`. Also run `RUSTDOCFLAGS="-D warnings" cargo doc`. |
| LESSON-3 | Re-run full pre-push gate (fmt + pedantic clippy + nextest) after EVERY commit to a feature branch before PR. |
| LESSON-5 | pr-manager CANNOT spawn sub-agents. Orchestrator dispatches security-reviewer + pr-reviewer independently. |
| LESSON-6 | RELAXED by STANDING MERGE AUTH: orchestrator may merge when CI-green + security-reviewer CLEAN + pr-reviewer APPROVE. |
| LESSON-7 | Per-story adversary convergence passes run SEQUENTIALLY. Never parallelize passes of one story. |
| LESSON-8 | Every pre-push gate AND implementer exit-gate MUST run FULL canonical clippy (LESSON-2) + `cargo fmt --all -- --check`. |
| LESSON-9 | When PR-level security/pr-reviewer findings are fixed AFTER per-story adversary convergence, RE-RUN both reviewers + wait for CI before merge. |
| LESSON-10 | For strict 3-CLEAN on compliance/extension stories: proactive doc-vs-code + sibling-site consistency audit BEFORE final convergence passes. |
| LESSON-11 | EXTENSION stories often have high GREEN-BY-DESIGN ratio. Stricter new tests can expose real bugs in already-merged code — treat as in-scope fixes. |
| LESSON-12 | Agents in feature worktrees MUST NEVER commit to `develop`. After every merge, verify `develop == origin/develop` before creating next worktree. |
| LESSON-13 | UPSTREAM-DATA VERIFICATION: before implementing a story consuming IR data from upstream, verify data is actually threaded through the real pipeline. Get architect assessment if gap is large. |
| LESSON-14 | PRESENCE-VS-CONTENT TESTS: strengthen to assert CONTENT and VALIDITY (parse output, assert required child elements, schema correctness), not just existence. |
| LESSON-15 | Demo-recorder example binaries MUST pass FULL canonical clippy including `-W clippy::missing_docs_in_private_items`. |
| LESSON-16 | PRE-PUSH GATE MUST MIRROR EXACT CI INVOCATIONS (full workspace pedantic clippy + rustdoc gate), especially when adding example binaries or intra-doc links. |
| LESSON-17 | `#[should_panic]` is NEVER a placeholder for behavioral correctness tests. Only valid for deliberate panic-on-invalid-input paths; those tests must FAIL on stubs. |
| LESSON-18 (WORKTREE-SYNC) | After `gh pr merge --squash`, run `git fetch && git merge --ff-only origin/develop` OR `git restore --source=HEAD --staged --worktree .` to sync working tree. `git update-ref` alone leaves working tree STALE. Include disk-presence check before any gate agent dispatch. |
| LESSON-19 (SIBLING-SWEEP) | When a fix-burst changes a canonical VALUE, TEXT, COUNT, or ANCHOR (error code, message template, site count, ADR/BC/PC citation, stale-state comment), it MUST sweep ALL sibling artifacts in ONE burst. Partial propagation repeatedly reset STORY-089 3-CLEAN streak (~11 passes). Orchestrator must grep-map every occurrence itself and dispatch ONE coordinated exhaustive fix. |
| LESSON-20 (ADVERSARY-SPEC-PATHS) | Per-story LOCAL adversary dispatches MUST include ABSOLUTE `.factory/` spec paths (story file, traced BCs, traced ADRs, export-architecture or equivalent spec). The per-story worktree does NOT contain the `.factory/` mount. |
| LESSON-21 (LOCAL-GATE-MIRRORS-CI-MATRIX) | Exit gate MUST include: `cargo test --workspace --all-features` (shared-process, catches global-state/test-isolation bugs nextest masks) + `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` + cross-platform/portable filesystem tests (no `/nonexistent` Unix-root paths; no virgin-global-state assumptions). |
| LESSON-22 (ORCHESTRATOR-VERIFY-UPSTREAM-CLAIMS) | Before routing a deferral/"not supported" based on an implementer's claim of an upstream gap, VERIFY the claim against actual upstream code, and confirm any cited story ID EXISTS. |
| LESSON-OPS-RATE-LIMIT | RATE-LIMITING ACTIVE: backend applies sustained server-side throttle. Dispatch adversary/review passes ONE AT A TIME (serialize). Batching 3+ simultaneous agents triggers "Server is temporarily limiting requests" rejections (transient, retry). |
| LESSON-OPS-CWD | cwd DISCIPLINE: every worktree-scoped agent MUST prefix EVERY bash command with `cd <worktree> &&` (shell does NOT persist cd across calls) and use absolute paths. A violation leaked STORY-081/082 stub files into main repo (cleaned). Include this in test-writer/implementer/demo dispatch prompts. |
| LESSON-OPS-CI-DISKSPACE | STORY-047 snapshots CI failure was a DISK-SPACE infra flake (No space left on device / ld bus error), NOT a code defect — re-run on a fresh runner clears it. reqwest rustls feature pulls heavy native deps (aws-lc-sys + ring) increasing disk pressure (FU-047-DEPS-AWSLC). |
| LESSON-OPS-SPECFIX-SCOPE | Architect/product-owner/story-writer dispatches MUST be scoped to .factory/ ONLY. Forbidden from touching crates/ or committing to develop. (One violation occurred: ADR-021 OBS-2 spec-fix agent committed workspace Cargo.toml to develop — since reset.) |

---

## Blocking Issues

| ID | Description | Status |
|----|-------------|--------|
| BLK-001 | Alt-text enforcement non-functional end-to-end. | RESOLVED — STORY-050 ADR-018 post-layout validation. |
| BLK-002 | Wave 4 gate (content threading + color-coded types + image-alt fix). | CLOSED 2026-06-07 — Gate 5 mean 1.00, min_critical 1.00. |

---

## Decisions Log

_Wave-4-gate and earlier archived to `.factory/cycles/wave-4-gate/decisions-archive.md`._
_STORY-081 full 34-pass cascade (passes 1-30 LOCAL + P31-P34 PR-level) archived to `.factory/cycles/STORY-081/`._
_Wave-5 per-story pass logs archived to `.factory/cycles/wave-5-merges-archive.md`._

| Date | ID | Decision |
|------|-----|---------|
| 2026-06-10 | STORY-CI-INITIATIVE | Human directed 2026-06-10: CI performance stories BEFORE remaining wave-5 feature stories. Research-grounded plan: 3 stories (STORY-091/092/093), EPIC-19, priority NEXT, Wave 5. Confirmed facts: arm64 runner already native ubuntu-24.04-arm (NOT QEMU); cache root cause CONFIRMED (9.77 GB / 23 caches / 97.7% of ~10 GB limit); action SHAs confirmed 2026-06-10 via git ls-remote. Delivery order: 091 (tiered triggers, biggest per-PR win) → 092 (cache+disk, highest flake leverage) → 093 (mold+profile). Portable playbook: `.factory/playbooks/tiered-ci-merge-queue.md`. Research: `.factory/planning/ci-speed-research.md`. Total: 90→93 stories, 556→571 pts. |
| 2026-06-10 | STATE-REFRESH-CI | STATE.md zero-context resume refreshed: CI initiative captured (Workstreams A+B); NEXT ACTIONS 3-phase; cache root cause (9.77 GB confirmed) recorded; frontmatter updated (total_stories: 93, total_points: 571, open_prs: 2). Historical STORY-081 34-pass cascade compacted — full record in `.factory/cycles/STORY-081/`. |
| 2026-06-10 | STORY-081-PR80 | PR #80 created (rebased onto develop 15838de1); PR-level adversary P31-P34 converged; security CLEAN; pr-reviewer APPROVE. Pre-PR integration gap: list-form bullets inline markup (FieldValue::InlinesList). Non-blocking: FU-S1-FONTDB-COUNT-VISIBILITY, FU-S3-CAPTION-FIXTURE, FU-TD1-DEAD-FONT-COUNTER. HEAD 5c665cd0. |
| 2026-06-10 | STORY-081-CONVERGED | LOCAL adversarial cascade CONVERGED 3/3 strict-CLEAN (passes 28-29-30) at code HEAD f047348c / BC-3.05.001 v1.4.3. ADR-024 PPTX unification; BC re-anchor BC-3.02.002→BC-3.05.001; 12-form×5-surface reconciliation. FU-LINK-SCHEME-CONSISTENCY OPEN (architect adjudication). Full 34-pass cascade report in `.factory/cycles/STORY-081/`. |
| 2026-06-09 | CI-ARM64-TIMEOUT-FIX | PR #79 merged → develop. ci.yml test-matrix timeout 30→75 min + cache-on-failure:true (Swatinem/rust-cache). |
| 2026-06-09 | STORY-088-MERGE | PR #78 squash-merged (ADMIN OVERRIDE) → develop `15838de1`. Bullets list-literal DSL. CONVERGED 3/3 (passes 8-9-10). dsl-spec v1.1, error-taxonomy v2.28. |
| 2026-06-09 | STORY-082-MERGE | PR #77 squash-merged → develop `c60cca36`. PPTX slide-grouping sections. CONVERGED 3/3 (passes 16-17-18). |
| 2026-06-08 | WAVE5-BATCH-MERGES | PRs #69-#76 merged (STORY-089/046/055/079/080/074/047/072). 10 Wave-5 stories done. develop advanced to 15838de1 (79 merged PRs). |

---

## Quality Bar (Non-Negotiable Gates)

Production-grade from day 1. Full table in CLAUDE.md. `#![forbid(unsafe_code)]`; zero `.unwrap()` outside tests; `clippy::pedantic`; `#![warn(missing_docs)]`; Kani+fuzz+mutants (Phase 6); WCAG AA; PDF/UA-1; < 500ms cold build; signed releases; SBOM; cross-platform macOS+Linux+Windows.
