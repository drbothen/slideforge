---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-09
state_version: "1.1"
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
total_stories: 90
total_points: 556
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
wave_5_status: "10 of 22 Wave-5 stories MERGED (…STORY-072 PR#76, STORY-082 PR#77, STORY-088 PR#78). 12 stories remain. In-flight (1 active worktree): STORY-081 (Pass 17 (first against unified engine + BC-3.05.001 anchor): PPTX unification VERIFIED clean (Invariant 9 holds; FU-DUAL-RUN-GENERATOR resolved); found 2 MED in non-PPTX legs — F-P17-001 DOCX nested-link formatting drop (fixed) + F-P17-002 HTML Footnote/Math vs PC-4 (Footnote fixed; Math codified as STORY-045 deferral, BC-3.05.001 v1.4.1); streak 0/3; new HEAD d185badb; NEXT adversary Pass 18 fresh). CI infra fix PR#79 merged."
develop_sha: "15838de1"
develop_pr_count: 79
error_taxonomy_version: "v2.28"
workspace_tests: "3825 pass / 20 skip / 0 fail (STORY-081 worktree HEAD d185badb; Pass-17 fixes: DOCX nested-link formatting + HTML Footnote <span role=note> + grep-zero comment cleanup)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## ZERO-CONTEXT RESUME — START HERE

**Project:** slideforge — data-reactive branded document platform (Rust, greenfield, Phase 3 TDD)
**Repository:** https://github.com/drbothen/slideforge | **Workspace:** /Users/jmagady/Dev/slideforge

**Verify dev branch:** `git rev-parse develop` must equal `git rev-parse origin/develop`. Canonical SHA: `15838de1` (79 merged PRs, 0 open PRs).

**Factory worktree:** `.factory/` on branch `factory-artifacts`. Pushed to origin (human-authorized 2026-06-04; ongoing pushes authorized).

**Current position:** Phase 3, **Wave 5 IN PROGRESS**. STORY-088 MERGED PR #78 (15838de1, ADMIN OVERRIDE). CI fix PR #79 merged. 10 of 22 done. 12 stories remain.

---

## DURABLE RESUME — SAME MACHINE OR FRESH CLONE

Both branches are on origin (durable, machine-independent):

- `origin/factory-artifacts` — all `.factory/` state, all 17 STORY-081 adversary pass reports + orchestrator-EC004 note, ADR-023, ADR-024, BC-5.02.002 v1.5, BC-3.05.001 v1.4.1, BC-3.02.002 v1.5.1. (Run `git -C .factory log -1` for current HEAD.)
- `origin/feature/STORY-081` @ `d185badb` — 22 STORY-081 implementation commits (11 from Pass-10+11+13+14+15+16+17 fix bursts: 0b2ce53e a:highlight, 49b1fc8e pdf font double-load, f7c26fba normalize deterministic, 34c128b2 notes highlight child + body assertions + font docstring, 5556aa75 EC-004 body Link hlinkClick, eec4be32 DOCX hyperlink-format fix + combined-form assertions + e2e extension, d73bbc64 PPTX body+notes nested-link hlinkClick wrapper-descent, 5048a987 body link-in-link orphan-rel rid inherit-on-None + cross-path parity guard, c1401f33 ADR-024 inline-run generator unification, e5b1e92e dead-code cleanup, d185badb DOCX nested-link display-text formatting + HTML Footnote span-role-note + grep-zero comment cleanup); based on develop `cbebfd57`; MUST rebase onto develop `15838de1` at PR step.

**Same-machine resume:** `.factory/` and `.worktrees/STORY-081/` worktrees already exist on disk.
1. Run `vsdd-factory:factory-worktree-health`
2. Verify `git -C .worktrees/STORY-081 rev-parse HEAD` == `d185badb`
3. Continue at NEXT ACTION: adversary Pass 18 (fresh, HEAD d185badb, against BC-3.05.001 v1.4.1).

**Fresh-clone (different machine) resume — exact commands:**
```
git clone https://github.com/drbothen/slideforge.git && cd slideforge
git fetch origin factory-artifacts feature/STORY-081
git worktree add .factory factory-artifacts
git worktree add .worktrees/STORY-081 feature/STORY-081
git rev-parse develop   # must equal origin/develop == 15838de1
```
Then read `.factory/STATE.md` and continue at the NEXT ACTION below.

**Exact resume point:** Phase 3 / Wave 5 / STORY-081 adversary convergence at **streak 0/3** (Pass 17 RESET — 2 MED non-PPTX-leg findings: F-P17-001 DOCX nested-link display-text formatting drop + F-P17-002 HTML Footnote/<code> vs PC-4; both remediated at d185badb; BC-3.05.001→v1.4.1 Math deferral codified). NEXT: adversary Pass 18 (fresh context, sequential LESSON-7, against BC-3.05.001 v1.4.1) → 3/3 CONVERGED → demo-recorder per-AC → rebase onto develop `15838de1` → pr-manager 9-step.

---

## IN-FLIGHT WORKTREES — EXACT RESUME STATE

**1 active worktree. Based on cbebfd57; MUST rebase onto develop 15838de1 at PR step.**

### STORY-081 — Slide-Level Inline Markup (EPIC-18, BC-3.05.001, 13 pts)
- **Worktree:** `.worktrees/STORY-081` | **Branch:** `feature/STORY-081` | **HEAD:** `d185badb` (Pass-17 fixes — commits since c739d59c: 0b2ce53e (body a:highlight), 49b1fc8e (pdf font double-load), f7c26fba (normalize deterministic), 34c128b2 (notes a:highlight child + 3 body assertions + font docstring helper), 5556aa75 (EC-004 body Link hlinkClick implementation), eec4be32 (DOCX hyperlink-format fix + combined-form assertions + e2e extension), d73bbc64 (PPTX body+notes nested-link hlinkClick wrapper-descent), 5048a987 (body link-in-link orphan-rel rid inherit-on-None + cross-path body/notes parity guard), c1401f33 (ADR-024 unification), e5b1e92e (dead-code cleanup), d185badb (DOCX nested-link display-text formatting + HTML Footnote span-role-note + grep-zero comment cleanup); 11 commits; tests 3825 pass)
- **Adversary streak:** **0/3** — Pass 17 RESET (F-P17-001: DOCX silently dropped formatting inside link display text; F-P17-002: HTML Footnote emitted `<small>` not `<span role="note">`, Math codified as STORY-045 deferral). Both remediated at d185badb; BC-3.05.001→v1.4.1.
- **BC ANCHOR RESOLVED (human ruling 2026-06-09):** re-anchored BC-3.02.002 → BC-3.05.001 v1.4.1 (Rich Inline Formatting, CAP-024). BC-3.05.001 amended with slide-level field scope + PC-1..PC-5 per-exporter rendering matrix + HI-1..HI-5 hyperlink invariants + unified-engine Invariant 9 + v1.4.1 Math/Footnote PC-4 corrections. BC-3.02.002→v1.5.1 (cross-ref added; slide-level follow-up closed by STORY-081 via BC-3.05.001).

- **Pass 17 findings (all REMEDIATED):**
  - F-P17-001[MED] — DOCX Link arm used collect_plain_text on display children → `[**here**](url)` dropped bold; PPTX+HTML preserved it; DOCX lone divergent surface. BC-3.05.001 PC-3 violation. REMEDIATED d185badb: DOCX Link arm recurses via build_hyperlink_display_runs → structured WR runs with `<w:rPr>` in `<w:hyperlink>`.
  - F-P17-002[MED] — HTML Footnote → `<small>` (PC-4 says `<span role="note">`); Math → `<code class="math">` but PC-4 v1.4.0 said `<math>` unconditionally. Footnote REMEDIATED d185badb (impl fix). Math REMEDIATED as spec reconciliation: BC-3.05.001 v1.4.1 codifies `<code class="math">` as v1.0 behavior; full MathML deferred to STORY-045.
  - OBS-P17-A[OBS] — stale dispatch-site comments + dead exemption branch in grep-zero guard. REMEDIATED d185badb (comments updated; dead branch removed; guard re-verified).
  - OBS-P17-B[OBS] — EC-011 strict-mode trace out-of-scope (BC-1.15.003). Non-finding.
- **ADR-024 implementation summary (HEAD e5b1e92e, confirmed clean at Pass 17):** New unified engine `render_inline_nodes_to_runs(nodes, hlink_resolver)` + `OoxmlRun` + `serialize_ooxml_run` in `slideforge-plugin-api/src/inline_formats/ooxml_runs.rs`. Body path via `ooxml_run_to_ooxmlsdk`; notes path via `serialize_ooxml_run`. Old body generator cluster (~312 lines) DELETED. AC-005 grep-zero audit clean. FU-PPTX-DUAL-RUN-GENERATOR CONFIRMED RESOLVED.
- **Pre-PR blockers:** FU-DIAGRAMS-COLD-BUDGET-TIMING-GATE RESOLVED. FU-CI-ARM64-TEST-FAILURE RESOLVED. Confirm arm64 CI green on STORY-081 PR.
- **fontdb pin:** =0.23.0 direct pin retained; cargo deny PASS.
- **Workspace tests at HEAD d185badb:** 3825 pass / 20 skip / 0 fail.
- **NEXT ACTION:** adversary Pass 18 fresh against BC-3.05.001 v1.4.1 (sequential LESSON-7). Need 3 consecutive strict-CLEAN to converge 3/3.

---

## WAVE 5 DELIVERY SUMMARY

**10 of 22 done (develop 15838de1, 79 PRs):**

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
- **CI-FIX MERGED** PR #79: ci.yml test-matrix `timeout-minutes` 30→75 + `cache-on-failure: "true"` (Swatinem/rust-cache). Roots out linux-arm64 cold-build-timeout self-perpetuating loop (cancelled jobs never saved cache). ci-workflow-analyzer caught initial no-op (`save-always` invalid for rust-cache) → corrected to `cache-on-failure`.

**12 stories remain. 1 active worktree (see IN-FLIGHT section above).**

**HELD (next batch after in-flight merges):**
- STORY-057/058/064 (slideforge-cli same-crate conflict — deliver AFTER this batch, serialized)
- STORY-056 (←047 now MERGED — UNBLOCKED; may start after in-flight batch)
- STORY-048 (←047 now MERGED — UNBLOCKED)
- STORY-060/061 (FU-SEC-001-GIT2-OPENSSL must be resolved FIRST)

**OPEN FOLLOW-UPS:**
- **FU-VP-043-NOTES-PATH** (architect/formal-verifier, Phase-6, non-blocking): VP-043 ("all 12 inline variants produce distinct non-empty XML in PPTX") now implicitly spans body AND notes via the unified ADR-024 engine (BC-3.05.001 v1.4.0 amendment). Assess whether a separate notes-path proof/snapshot variant is needed; update VP-INDEX + verification-coverage-matrix under the vp_index source-of-truth policy. Source: PO BC-3.05.001 v1.4.0 amendment.
- **FU-CI-ARM64-TEST-FAILURE** (RESOLVED — commits 49b1fc8e + f7c26fba): Root cause confirmed — two wall-clock timing gates (`cold_budget.rs` + `normalize_under_budget` warm-path test) were flaking on the slow emulated arm64 runner. Both converted to deterministic `font_db_load_count == 1` assertions; underlying double-load defect in `slideforge-pdf/src/font.rs` fixed. Confirm fully closed by verifying arm64 CI green on STORY-081 PR.
- **FU-DIAGRAMS-COLD-BUDGET-TIMING-GATE** (RESOLVED — commits 49b1fc8e + f7c26fba): `crates/slideforge-diagrams/tests/cold_budget.rs` flaky wall-clock gate eliminated. Root cause: real double `load_system_fonts` call per `resolve_font_set` in `slideforge-pdf/src/font.rs` (called in both `resolve_font_set` AND `resolve_regular_face`). Fixed to load once and share `&fontdb::Database`. Both timing gates converted to deterministic `font_db_load_count == 1`; perf budget remains gated by criterion benches.
- **FU-NOTES-HIGHLIGHT-ATTR** (RESOLVED — fixed IN STORY-081 commit 34c128b2; NOT deferred to STORY-040): notes-path `emit_run` now emits `<a:highlight><a:srgbClr val="FFFF00"/></a:highlight>` child element; `highlight="yellow"` attribute form eliminated; Red-Gate test confirms child present + attribute absent.
- **FU-PPTX-DUAL-RUN-GENERATOR** (RESOLVED by ADR-024 — 2026-06-09; CONFIRMED RESOLVED by Pass 17 Axis-B verification): Root cause of 3 STORY-081 findings (ADV-P11-HIGH-001, F-P15-HIGH-001, F-P16-M1). The two generators are now ONE engine (`render_inline_nodes_to_runs` in `slideforge-plugin-api`); divergence impossible by construction. Cycle-closing checklist S-7.02 process-gap satisfied — resolved, not deferred. No follow-up story required; architectural unification is complete.
- **FU-STORY-045-HTML-MATHML** (non-blocking; STORY-045 is the tracked vehicle): HTML Math full MathML `<math>` rendering deferred to STORY-045. BC-3.05.001 PC-4 v1.4.1 codifies the v1.0 `<code class="math">{escaped LaTeX}</code>` fallback as the CURRENT behavior; MathML is an explicit non-silent deferral. Source: F-P17-002 Math half (spec reconciliation, not impl defect). No action required in STORY-081 scope.
- **F-085-P6-001 SUPERSEDED** [parity ruling, Pass 14]: F-085-P6-001 "top-level-only hlinkClick" is SUPERSEDED for nested links by the Pass-14 cross-format parity ruling. Body + notes now emit `<a:hlinkClick>` for wrapped links (`Bold([Link])`, `Italic([Link])`, etc.); orphan-rel invariant preserved. The 5 F-085-P6-001 tests in STORY-085 notes behavior were updated at d73bbc64 to expect 1-rel/1-click for wrapped links. F-085-P6-001 docstring carve-out removed. FU-PPTX-DUAL-RUN-GENERATOR RESOLVED (ADR-024).
- **FU-088-BC10102-ANCHOR** (spec-steward; non-blocking): pre-existing BC-1.01.002 H1 title/anchor mismatch.
- **FU-EXIT-GATE-DISTINGUISHING-OUTPUT** (process-improvement; source OBS-1 Pass-3 + OBS-P04-001 Pass-4 + OBS-P05-001 Pass-5 + ADV-P06-MED-001 Pass-6 + PROCESS-NOTE Pass-8 + ADV-P11-HIGH-001 Pass-11 + orchestrator EC-004 catch Pass-11 validation + F-P13-002 Pass-13): Anti-pattern recurred 9th time. LESSON codification: "MED distinguishing-assertion findings must close with assertions that FAIL when the form's unique output is removed; presence-of-display-text is vacuous; COMBINED-form cells (bold+italic, bold+link, nested) must have per-exporter assertions". Exit-gate rule must verify (1) non-test caller consumes a dispatch fn's DISTINGUISHING output; (2) multi-span-same-line positional assertion; (3) when drawable font present but raw bytes absent, non-silent diagnostic; (4) BOTH measure path AND draw path select slots via THE SAME helper; (5) multi-exporter "rich rendering" closures MUST be verified PER EXPORTER; (6) COMBINED-form compound nodes must have load-bearing assertions on EVERY exporter surface. Target: lessons-codification / Standing Process Rules.
---

## CURRENT POSITION

Phase 3, **Wave 5 IN PROGRESS** (develop `15838de1`, 79 merged PRs). 10 of 22 done. 12 stories remain. 90 stories / 556 pts total.

- Active worktrees: 1 — STORY-081 in `.worktrees/STORY-081` on `feature/STORY-081` HEAD `d185badb`. Pass 17 RESET (F-P17-001 DOCX nested-link formatting + F-P17-002 HTML Footnote/Math PC-4; both remediated; BC-3.05.001→v1.4.1); streak 0/3. PPTX unification (ADR-024) VERIFIED clean at Pass 17 — FU-PPTX-DUAL-RUN-GENERATOR CONFIRMED RESOLVED. Adversary Pass 18 NEXT (fresh, HEAD d185badb, against BC-3.05.001 v1.4.1). Open PRs: 0.
- Workspace: 3825 pass / 20 skip / 0 fail (STORY-081 worktree at HEAD d185badb; cargo deny PASS; cold_budget PERMANENTLY FIXED; FU-DIAGRAMS-COLD-BUDGET-TIMING-GATE RESOLVED; FU-CI-ARM64-TEST-FAILURE RESOLVED — confirm on STORY-081 PR arm64 CI).
- Uncertainty pass: COMPLETE. ADR-022 dep-centralization: DONE. ADR-008 P4 amendment: DONE. ADR-021 async runtime: DONE.

---

## NEXT ACTIONS

**RESUME PROCEDURE (zero context):**
1. Run `vsdd-factory:factory-worktree-health`
2. Verify `git rev-parse develop` == origin/develop == `15838de1`
3. Confirm workspace tests green (`cargo nextest run --workspace --no-fail-fast` — expect ~3916+ pass, ~20 skip; cold_budget PERMANENTLY FIXED; NOTE: linux-arm64 CI has 1 unresolved nextest failure — FU-CI-ARM64-TEST-FAILURE; capture test name on next PR run)
4. Read BACKLOG.md WAVE5-DELIVERY for in-flight status
5. For each in-flight story, check `git -C .worktrees/STORY-<NNN> log --oneline -5` to confirm HEAD matches the table above
6. **Continue in priority order:** STORY-081 — Pass 17 RESET (F-P17-001 DOCX nested-link formatting + F-P17-002 HTML Footnote/Math PC-4; both remediated; BC-3.05.001→v1.4.1); HEAD `d185badb`; streak **0/3**. NEXT: adversary Pass 18 (fresh context, sequential LESSON-7; HEAD d185badb; against BC-3.05.001 v1.4.1). Need 3 consecutive strict-CLEAN to converge 3/3. Per-story adversary passes SERIAL (LESSON-7 + rate-limit).

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
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 89 stories, 21 epics, 6 waves, 553 pts. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3/4 GATE PASSED. Wave 5: **10/22 done** (STORY-089 PR#68, STORY-046 PR#70, STORY-055 PR#71, STORY-079 PR#72, STORY-080 PR#73, STORY-074 PR#74, STORY-047 PR#75, STORY-072 PR#76, STORY-082 PR#77, STORY-088 PR#78). 1 in-flight worktree. 12 stories remain. | Per-story delivery |
| Phases 4-7 | NOT STARTED | Holdout / Adversarial / Formal Hardening / Convergence |

---

## Session Resume Checkpoint

**Wave 5 IN PROGRESS. develop 15838de1 (79 merged PRs). STORY-081 Pass 17 RESET (2 MED non-PPTX-leg: DOCX nested-link formatting + HTML Footnote PC-4; both remediated; BC-3.05.001→v1.4.1; PPTX unification VERIFIED clean) — streak 0/3. 1 worktree active. 12 stories remain. NEXT: adversary Pass 18 against BC-3.05.001 v1.4.1.**

| Field | Value |
|-------|-------|
| **Date** | 2026-06-09 |
| **develop SHA** | `15838de1` (79 merged PRs; origin/develop confirmed; 0 open PRs) |
| **Merged this session** | STORY-088 PR#78 (ADMIN OVERRIDE), CI-fix PR#79; STORY-072/082/088 also merged this session |
| **Active worktrees** | 1 — STORY-081 in `.worktrees/STORY-081` on `feature/STORY-081` HEAD `d185badb`. |
| **STORY-081 state** | HEAD `d185badb`; adversary **0/3** (RESET by Pass 17 — F-P17-001 DOCX nested-link formatting + F-P17-002 HTML Footnote/Math PC-4; both remediated d185badb; BC-3.05.001→v1.4.1; PPTX unification VERIFIED clean; FU-PPTX-DUAL-RUN-GENERATOR CONFIRMED RESOLVED). Fix commits since c739d59c: 0b2ce53e, 49b1fc8e, f7c26fba, 34c128b2, 5556aa75, eec4be32, d73bbc64, 5048a987, c1401f33, e5b1e92e, d185badb (11 commits). BC ANCHOR RESOLVED (human ruling 2026-06-09): BC-3.02.002 → BC-3.05.001 v1.4.1. |
| **Workspace tests** | 3825 pass / 20 skip / 0 fail (HEAD d185badb; cargo deny PASS; FU-DIAGRAMS-COLD-BUDGET-TIMING-GATE RESOLVED; FU-CI-ARM64-TEST-FAILURE RESOLVED — confirm arm64 on STORY-081 PR) |
| **factory-artifacts** | Pushed to origin. Fresh sessions: clone + `git worktree add .factory factory-artifacts`. |
| **RESUME INSTRUCTION** | STORY-081 Pass 17 RESET; streak 0/3; HEAD d185badb. ADR-024 unification complete + VERIFIED clean (Pass 17 Axis-B; FU-PPTX-DUAL-RUN-GENERATOR CONFIRMED RESOLVED). BC-3.05.001 v1.4.1 (PC-4 Math → `<code class="math">` + STORY-045 deferral; Footnote → `<span role="note">`). NEXT: adversary Pass 18 (fresh context, sequential LESSON-7, against BC-3.05.001 v1.4.1). Need 3 consecutive strict-CLEAN for 3/3 convergence. After 3/3: demo-recorder → rebase onto 15838de1 → pr-manager 9-step → STANDING MERGE AUTH → squash-merge → state-manager post-merge burst → worktree cleanup. Rate-limiting: ONE adversary/review pass at a time. Confirm arm64 CI green on STORY-081 PR (FU-CI-ARM64-TEST-FAILURE). HELD next batch: STORY-056/048 (unblocked ←047), STORY-057/058/064 (serialize cli), STORY-060/061 (GIT2-OPENSSL first). |

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

_Entries before STORY-050-MERGE archived to `.factory/cycles/wave-4-gate/decisions-archive.md`._

| Date | ID | Decision |
|------|-----|---------|
| 2026-06-09 | STORY-081-PASS17 | Pass 17 NOT CLEAN — first pass against unified engine + BC-3.05.001 v1.4.0. Axis-B (PPTX unification) VERIFIED clean: Invariant 9 grep-zero holds; FU-PPTX-DUAL-RUN-GENERATOR CONFIRMED RESOLVED. Two MED findings in non-PPTX legs: F-P17-001[MED] DOCX Link arm used collect_plain_text on display children → `[**here**](url)` dropped bold (BC-3.05.001 PC-3 violation; DOCX lone divergent surface); REMEDIATED d185badb (DOCX Link arm recurses via build_hyperlink_display_runs → structured WR runs with `<w:rPr>` in `<w:hyperlink>`). F-P17-002[MED] HTML Footnote → `<small>` (PC-4 says `<span role="note">`); Math → `<code class="math">` but PC-4 v1.4.0 said `<math>` unconditionally; Footnote REMEDIATED d185badb (impl fix → `<span role="note">`); Math resolved as spec reconciliation: BC-3.05.001→v1.4.1 codifies `<code class="math">` as v1.0 behavior + defers full MathML to STORY-045 (FU-STORY-045-HTML-MATHML). OBS-P17-A stale grep-zero comments REMEDIATED d185badb. Streak RESET 0/3. New HEAD d185badb; tests 3825 pass. NEXT: adversary Pass 18 (fresh, d185badb, BC-3.05.001 v1.4.1). |
| 2026-06-09 | STORY-081-REANCHOR | Human ruled anchor BC-3.02.002 (section blocks, wrong) → BC-3.05.001 (inline formatting, correct). Rationale: BC-3.02.002 PC8 scope-boundary explicitly excludes slide-level field values and deferred slide-level inline markup to a follow-up story; BC-3.05.001 already governs "all 12 inline format types render per format" including slide text fields. BC-3.05.001 amended to v1.4.0: slide-level field scope (title/subtitle/body/bullets/caption/description) + AC-006 dual-title shadow invariant; per-exporter rendering matrix (5 surfaces: PPTX body, PPTX notes, DOCX, HTML, PDF) with unified-engine detail per ADR-024; corrected PPTX Highlight postcondition (child-element form); corrected hyperlink reference-set invariant; rId namespace isolation; nested/wrapped link behavior; safe-URL-scheme + empty-display-text guards; Math PPTX behavior; EC-007; ADR-024/017 + BC-5.02.002 cross-references; STORY-081 added to Stories traceability. BC-3.02.002 amended to v1.5.1: cross-ref to BC-3.05.001 added; slide-level follow-up now closed by STORY-081 via BC-3.05.001. STORY-081 spec frontmatter + all 6 AC traces re-pointed to BC-3.05.001 clauses; spec_version 1.3→1.4. STORY-INDEX + dependency-graph BC traceability updated. |
| 2026-06-09 | STORY-081-PASS16 | Pass 16 NOT CLEAN — M1[MED]: PPTX notes path silently dropped formatting inside link display text (`[click **here**](url)` → plain in notes; body+HTML preserved bold); M2[MED]: cross-path parity test asserted only body/notes parity, not absolute rel/click contract; L1[LOW]: intent question resolved by human (DEFECT — preserve formatting). Human AUTHORIZED: (1) unify the two PPTX inline-run generators now in STORY-081 → ADR-024; (2) notes link-text formatting loss is a DEFECT (preserve). ADR-024 implemented (c1401f33 unification + e5b1e92e dead-code cleanup): new unified engine `render_inline_nodes_to_runs` + `OoxmlRun` + `serialize_ooxml_run` in `slideforge-plugin-api`; old body generator cluster deleted; AC-005 grep-zero clean; tests 3820 pass. FU-PPTX-DUAL-RUN-GENERATOR RESOLVED. BC-5.02.002→v1.5. Streak RESET 0/3. New HEAD e5b1e92e. Orchestrator surfaced separate BC-3.02.002 ANCHORING issue (STORY-081 slide-level inline markup anchored to section-block BC whose PC8 excludes slide-level; v1.5 deferred slide-level) — pending human adjudication before cascade resumes. |
| 2026-06-09 | STORY-081-PASS15 | Pass 15 NOT CLEAN — F-P15-HIGH-001[HIGH]: body Link dispatcher arm built `RunContext{ hyperlink_rid: None }` for inner Link whose url was absent from the rel map, OVERWRITING the inherited outer rid (Some(rId_U1)) → inner display-text renders plain, no `<a:hlinkClick>` emitted → 1 rel / 0 clicks → ORPHAN External rel; `external_rel_count == hlinkclick_count` invariant violated. Notes path correct (flattens display text; can't diverge on this axis). Root: dual-generator design asymmetry (OBS-P15-001; third FU-PPTX-DUAL-RUN-GENERATOR finding). REMEDIATED commit 5048a987: `inherited_rid = rid.or_else(|| ctx.hyperlink_rid.clone()); RunContext{ hyperlink_rid: inherited_rid, .. }` — registered rid wins; outer rid inherited only when inner lookup None. Red-Gate: body link-in-link (1/1), body link-in-bold-link (1/1). Cross-path equivalence guard `test_obs_p15_001_body_notes_cross_path_rel_click_parity` (8-shape battery) added — makes future dual-generator divergence visible immediately. Streak RESET 0/3. New HEAD 5048a987; tests 3791 pass. NEXT: adversary Pass 16 fresh. |
| 2026-06-09 | STORY-081-PASS14 | Pass 14 NOT CLEAN — ADV-P14-MED-001 ADJUDICATED cross-format parity DEFECT (not spec-sanctioned). PPTX body/notes silently dropped nested-link URL for `Bold([Link])` → bold but non-clickable; DOCX (post-F-P13-001) + HTML rendered bold+clickable. Grounds: EC-004/EC-003 no top-level carve-out; F-085-P6-001 was unexpired orphan-rel expedient never parity-ratified; orphan-rel invariant preservable via wrapper-descent collector; silent drop while Math path warns (internal inconsistency). REMEDIATED commit d73bbc64: body+notes collectors descend through Bold/Italic/Strike/Super/Sub/Highlight/Footnote wrappers; dispatcher threads hyperlink_rid; inner Link arm emits `<a:hlinkClick>`; external_rel_count==hlinkclick_count maintained. Red-Gate: body bold-link, body bold-italic-link, notes bold-link, nested unsafe-scheme, nested empty-text. e2e slide-3 updated. F-085-P6-001 docstring carve-out removed; SUPERSEDED for nested links. Streak RESET 0/3. New HEAD d73bbc64; tests 3787 pass. NEXT: adversary Pass 15 fresh. |
| 2026-06-09 | STORY-081-PASS13 | Pass 13 NOT CLEAN — fresh re-derivation at HEAD 5556aa75, COMBINED/COMPOUND-form axis. F-P13-001[MED]: DOCX `apply_run_property` `other => other` arm silently passed `WHyperlink` unchanged — `<w:b/>` never applied for `Bold([Link])`; PPTX/HTML/PDF all preserved bold-on-link; DOCX lone divergent exporter. REMEDIATED commit eec4be32 (WHyperlink arm iterates inner WR runs; FnOnce→Fn; Red-Gate tests bold+italic wrapping link). F-P13-002[MED]: every combined cell (bold+italic, bold+link, nested) untested on 4 STORY-081 exporter surfaces — gate gap hid F-P13-001. REMEDIATED commit eec4be32 (combined-form load-bearing assertions per exporter; e2e fixture extended with slides 3+4, 6 cross-exporter parity assertions). Streak RESET 0/3. New HEAD eec4be32; tests 3782 pass. |
| 2026-06-09 | STORY-081-PASS11 | Pass 11 NOT CLEAN — fresh re-derivation at HEAD f7c26fba. ADV-P11-HIGH-001: notes `emit_run` emitted `highlight="yellow"` ATTRIBUTE on `<a:rPr>` (schema-invalid; TD-VSDD-060 sibling-sweep failure of Pass-10 body fix); REMEDIATED commit 34c128b2 (child element + Red-Gate test). ADV-P11-MED-001: body path missing distinguishing assertions for Code/Super/Sub/Link; REMEDIATED commits 34c128b2 + 5556aa75. ADV-P11-LOW-001: font.rs docstring overstated load invariant; REMEDIATED commit 34c128b2. ORCHESTRATOR VALIDATION CATCH: during post-fix MED-001 validation, body Link `make_run` had `let _ = url` (URL dropped), `add_external_hyperlink` notes-only; NO `<a:hlinkClick>` ever emitted for body links — EC-004 violation masked by vacuous `test_adv_p11_med_001_body_path_link_display_text_present` (green while feature absent; TD-VSDD-059). Implemented EC-004 body-link hlinkClick (commit 5556aa75): hlink_map pre-computed in `build_slide_parts`, safe-scheme guard, threaded to `make_run`; 3 load-bearing tests. FU-NOTES-HIGHLIGHT-ATTR RESOLVED (fixed IN STORY-081, not deferred). FU-PPTX-DUAL-RUN-GENERATOR registered. Streak RESET 0/3. New HEAD 5556aa75; tests 3770 pass. NEXT: adversary Pass 12 (fresh context). |
| 2026-06-09 | STORY-081-PASS10 | Pass 10 NOT CLEAN — fresh PPTX Highlight seam examination found ADV-P10-HIGH-001 (HIGH): body `InlineNode::Highlight` emitted `<a:solidFill>` (glyph recolor → yellow text, illegible) instead of `<a:highlight>` child element (background highlight). False premise in 4 code comments (ooxmlsdk =0.6.1 does expose `a_highlight` on RunProperties). No prior pass exercised this seam; zero distinguishing tests. DOCX uses `<w:highlight>`, HTML uses `<mark>` — only PPTX diverged (TD-VSDD-060). Streak RESET 0/3. Fix burst: 0b2ce53e (a:highlight correct; load-bearing test asserts presence + solidFill absent), 49b1fc8e (pdf font double-load defect fixed; cold_budget deterministic), f7c26fba (normalize_under_budget deterministic). FU-DIAGRAMS-COLD-BUDGET-TIMING-GATE + FU-CI-ARM64-TEST-FAILURE both RESOLVED. New HEAD f7c26fba; tests 3764 pass. NEXT: adversary Pass 11 (fresh context). |
| 2026-06-09 | STORY-081-PASS9 | Pass 9 CLEAN (strict + PR-merge); streak 1/3. ADV-P08-HIGH-001 CONFIRMED closed (load-bearing): `draw_frame` now takes `title_inlines_override: Option<&[InlineNode]>`; `FrameContent::Title` arm renders richly via `slide_to_krilla_runs+draw_inline_spans(..,36.0)` when inlines present; all 4 call-sites thread `title_inlines_nodes.as_deref()`; load-bearing test `test_BC_3_02_002_adv_p08_high001_pdf_title_bold_uses_distinct_font_resource` asserts both Tuffy(bold)+LatinModernMath-Regular. Fresh re-derivation of every seam (eval/layout/PPTX/DOCX/HTML/PDF × body/title/subtitle/caption/description × all-8-forms+Math+nested) found NO new gap. Both body-span axis (P2-P6) and title-leg axis (P8) structurally eliminated + re-confirmed sound. OBS-P09-001 (cold_budget/double load_system_fonts) out-of-scope, non-blocking for streak but pre-PR blocker intact. 2 more consecutive strict-CLEAN passes required to converge 3/3. HEAD c739d59c; tests 3762 pass. |
| 2026-06-09 | STORY-081-PASS8 | Pass 8 NOT CLEAN — fresh title-seam re-derivation found PDF title-bold gap (AC-006(3) PDF leg half-real since fix-burst-2; HTML real, PDF flattened to plain String). Streak RESET 0/3. ADV-P08-HIGH-001 OPEN: `FrameContent::Title` arm uses `extract_all_inline_text` (discards markup) → `font_set.regular` unconditional; DOCX + HTML both render bold title; PDF does not. `SubtitleInlines` capability (`slide_to_krilla_runs + draw_inline_spans`) exists but title not wired to it. Body-span axis (P2-P7) re-confirmed sound. Process lesson: per-exporter rich-rendering closures need a load-bearing test EACH exporter (not by analogy). FU-EXIT-GATE-DISTINGUISHING-OUTPUT extended to cover multi-exporter per-exporter verification. |
| 2026-06-09 | STORY-081-PASS7 | Pass 7 CLEAN (strict + PR-merge); streak 1/3. First strictly-clean pass. `face_for_span_kind` shared selector (exporter.rs:1219-1234) structurally eliminated the 5-pass measure≠draw recurrence; called by BOTH measure and draw paths; BoldItalic fallback unified `bold→italic→regular`. ADV-P06-MED-001 CONFIRMED load-bearing (test_adv_p06_med001_bolditalic_measure_uses_same_slot_as_draw_when_bold_absent). No new findings. 2 more consecutive strict-CLEAN passes required to converge 3/3. HEAD f1bd9f99; tests 3761 pass. |
| 2026-06-09 | STORY-081-PASS6 | Pass 6 NOT CLEAN (1 MED + 2 OBS; zero CRIT/HIGH 2nd consecutive pass). Pass-5 ResolvedFace fixes CONFIRMED load-bearing: `ResolvedFace{font,raw,face_index}` type invariant; `face_index` threaded through `compute_multi_span_x_positions` → `measure_text_width_pt`; load-bearing regression test confirmed. New findings: ADV-P06-MED-001 — BoldItalic span: measure path (`exporter.rs:1243`) uses `bold.or(regular)` while draw path (`exporter.rs:1294`) uses `bold.or(italic).or(regular)`; when `bold=None` & `italic=Some`, measure selects REGULAR, draw selects ITALIC → cursor mis-spacing; reachable via EC-003 nested bold-italic. OBS-P06-001 — cold_budget adjudicated OUT of STORY-081 scope (pre-existing diagrams timing-gate flake, no causal link to STORY-081 font loading). OBS-P06-002 — double `load_system_fonts()` per PDF export (correct-but-slow, ADR-023-deferred); wall-clock timing gates are structural false-green/false-fail vectors; lessons-codification: use deterministic call-count assertions. Severity decaying to MED-only 2nd pass. Shared `face_for_span_kind` helper to end measure≠draw recurrence. cold_budget must be resolved before STORY-081 PR (FU-DIAGRAMS-COLD-BUDGET-TIMING-GATE). Streak 0/3. |
| 2026-06-09 | STORY-081-PASS5 | Pass 5 NOT CLEAN (1 MED + 1 OBS; zero CRIT/HIGH). Pass-4 fixes CONFIRMED load-bearing: CRIT-001 cursor advance (real cmap→hmtx), HIGH-001 size-reduction (0.583) + docstrings — all verified. Pass-4 fix burst HEAD recorded: 46ed9eb9 (was uncheckpointed). New findings: ADV-P05-MED-001 — `compute_multi_span_x_positions` (exporter.rs:1239) hardcodes `face_index=0` in `measure_text_width_pt`; fontdb resolves non-zero face_index for .ttc collections → drawn face ≠ measured face → mis-spaced multi-span lines (strictly-increasing test passes but spacing wrong). OBS-P05-001 — `with_resolved_font_set` seam allows `Some(font)+None raw` → silent zero-advance overprint; codify: `tracing::warn` or `ResolvedFace { font, raw, face_index }` type invariant. Trajectory decaying to MED-only. Next: implementer ResolvedFace refactor → Pass 6. Streak 0/3. |
| 2026-06-09 | STORY-081-PASS4 | Pass 4 NOT CLEAN; C1-NEW + C2-NEW CONFIRMED load-bearing (not paper-fixes). New findings: ADV-P04-CRIT-001 — `draw_inline_spans_at_y` resets to bbox.x per span with no horizontal cursor advance; multi-span lines overprint (regression from C1-NEW fix); "for now" forbidden-rationalization comment at exporter.rs:1172-1175. ADV-P04-HIGH-001 — `slide_pdf.rs:69-70/78-83/154-155` docstrings still describe draw_glyphs mechanism; super/sub size reduction (SUPER_SUB_SCALE) absent from implementation. OBS-P04-001 — PDF exit-gate meta-pattern (pass-2 text-drop → pass-3 face-drop → pass-4 position-collapse); extends FU-EXIT-GATE-DISTINGUISHING-OUTPUT with positional assertion requirement. Streak 0/3. |
| 2026-06-09 | STORY-081-ADR023-AMEND | Architect ruling — krilla 0.6.0 draw_glyphs unusable (naive_shape pub(crate); no public shaping API); original ADR y_offset=units_per_em/3 dimensionally wrong (normalized field; upem/3≈682 for 2048-upem TrueType = nonsensical). Blessed mechanism: two `surface.draw_text()` calls with `font_size * SUPER_SUB_SCALE` (0.583) and `±(font_size * SUPER_RISE_FRACTION/SUB_DROP_FRACTION)` (0.333) baseline shift in point-space at PARENT font size. No HarfBuzz/skrifa required. Observable contract unchanged (raised+smaller super, lowered+smaller sub). ADR-023 dispatch table amended; size-reduction is now an explicit requirement. Story spec corrected to v1.3 (commit 52a841e8). BC unchanged. |
| 2026-06-09 | STORY-081-ADR023-FIXBURST | STORY-081 ADR-023 implementer fix burst COMPLETE. HEAD 626ae472b4a6f9f0f44096fbc032e35d675b59dd (was 885d8302). Pass-3 findings ALL CLOSED: C1-NEW — ResolvedFontSet + resolve_font_set() via fontdb =0.23.0 OS/2-metadata query in slideforge-pdf/src/font.rs; font_for_span() dispatch + draw_inline_spans_at_y() consume KrillaTextSpan .face + .y_offset_units in body/bullets/subtitle/title production paths; super/sub via per-glyph y_offset. C2-NEW — build()-driven test_BC_3_02_002_ac004_pdf_bold_span_uses_distinct_font_resource injects LatinModernMath-Regular + Tuffy fixture fonts, asserts both distinct embedded font resources; fixture crates/slideforge-pdf/tests/fixtures/Tuffy.ttf added. H1-NEW — docstrings corrected; all "future enhancement"/"for now" MVP comments removed. fontdb =0.23.0 promoted transitive→direct pin; cargo deny PASS. Non-silent fallback: tracing::warn! in resolve_font_set when styled face unavailable. LESSON-21 exit gate ALL GREEN (fmt/clippy pedantic+unwrap_used/nextest 3755/shared-process cargo test/rustdoc -D warnings/cargo deny). TWO watch items for Pass 4: (1) units_per_em=1000 hardcoded — potential offset-halving on TrueType 2048-upem fonts; (2) cold_budget PERMANENTLY FIXED by STORY-080 — any recurrence = regression. Adversary streak 0/3; NEXT: adversary Pass 4 fresh-context re-review. |
| 2026-06-09 | ADR-023-APPROVED | Human approved ADR-023 (PDF styled font-face resolution via fontdb metadata-aware ResolvedFontSet, Option C, fontdb =0.23.0 promoted from transitive to direct pin, confined to slideforge-pdf, no BC change). Story spec corrected to v1.2 (commit f55cb58d). |
| 2026-06-09 | STORY-081-PASS3 | Pass 3 NOT CLEAN (2 CRIT + 1 HIGH + 1 OBS). C1-NEW: font-face dead-wiring deeper layer — `extract_all_inline_text` discards FontFaceKind+y_offset_units, zero production readers. C2-NEW: PDF tests vacuous for AC-004 font-face claim. H1-NEW: misleading docstrings. OBS-1: exit-gate gap (non-test caller must consume distinguishing output, not just exist). Architectural asymmetry (SubtitleInlines vs title shadow-field) adjudicated ACCEPTABLE. Streak 0/3. |
| 2026-06-09 | STORY-081-FIX-BURST | STORY-081 implementer fix burst COMPLETE. HEAD 885d83027295946f97bcca71f1ece620fc6dd533 (was c11d6468). All 5 Pass-2 findings CLOSED: C1 PDF body/bullet dead-wiring (extract_all_inline_text routed through live draw path; dead extract_inline_text removed); C2 build()-driven PDF assertion with ActualText proof; C3 FrameContent::SubtitleInlines(Vec<InlineNode>) new variant wired through layout + all 4 exporters; HTML/PDF rich title AC-006(3) via existing title_inlines shadow-field (matching sound DOCX path; FrameContent::Title NOT widened ~118 sites); I1 PPTX InlineNode::Math tracing::warn! EC-008 pattern. LESSON-21 exit gate ALL GREEN (fmt/clippy pedantic+unwrap_used/nextest 3754 pass 0 fail/cargo test shared-process/rustdoc -D warnings). Adversary streak: 0/3 (fix burst does not advance). NEXT: adversary Pass 3 (fresh-context). Arch asymmetry (SubtitleInlines variant exists, no TitleInlines; title uses shadow field) flagged for Pass-3 adjudication. PDF per-span font switching for subtitle/title not implemented (consistent with PDF body inline limitation; candidate follow-up). |
| 2026-06-09 | STORY-081-AC006-3-DECISION | Human adjudicated AC-006(3): fix HTML/PDF rich title in same burst as C3 (widen FrameContent Title/Subtitle inline seam once). NOT deferred. Per adversary recommendation + production-grade default. |
| 2026-06-09 | CI-ARM64-TIMEOUT-FIX | PR #79 merged → develop. ci.yml test-matrix timeout 30→75 min + cache-on-failure:true (Swatinem/rust-cache). Roots out the linux-arm64 cold-build-timeout loop (cancelled jobs never saved cache → perpetual cold builds after foundational-crate changes). ci-workflow-analyzer caught initial no-op (save-always invalid for rust-cache) → corrected to cache-on-failure. |
| 2026-06-09 | STORY-088-MERGE | PR #78 squash-merged (ADMIN OVERRIDE) → develop `15838de1`. Bullets list-literal DSL across all 4 FieldValue value positions via shared list_literal combinator; E-PAR-024; non-recursive O(1) nested-depth tracker. CONVERGED 3/3 (passes 8-9-10), 10-pass cascade. Human-approved scope expansion (AC-012 set-rule list default, AC-013 variant vars list override). Security CLEAN; pr-reviewer APPROVE; 19/20 CI green. dsl-spec v1.1, error-taxonomy v2.28. |
| 2026-06-09 | STORY-088-ARM64-DEFER | PR #78 `test (linux-arm64)` had a genuine nextest FAILURE (other 3 platforms + local 3916-test suite PASS; exact test UNKNOWN — runner hung in finalization, logs unretrievable). Human directed admin-override merge + defer investigation to next PR. STORY-088 tests run <0.04s on all platforms; suspected pre-existing perf/timing flake on slow emulated arm64 runner, not a STORY-088 defect. Tracked: FU-CI-ARM64-TEST-FAILURE. |
| 2026-06-09 | STORY-088-SCOPE-EXPANSION | Adversary Pass-4 surfaced 2 MED intent-pending findings (list-literals produced cryptic errors in set-rule defaults + variant vars overrides). Human ADJUDICATED: extend full support. story v1.2→v1.3 (AC-012 set-rule list default, AC-013 variant vars list override; same Value::List semantics + E-PAR-024 element rules, no new merge semantics). Implemented at worktree HEAD 72787dcb (SetRuleValue::List variant, variant_value list arm, eval; 13 TDD tests; OBS-088-P4-002 stale #[ignore] doc-comment fixed; exit gate green 3740 pass). FU-088-BC10102-ANCHOR registered (pre-existing BC-1.01.002 title/anchor mismatch, spec-steward, non-blocking). STORY-088 streak reset 0/3; Pass 5 next against story v1.3. |
| 2026-06-09 | STORY-082-MERGE | PR #77 squash-merged → develop `c60cca36` (77 merged PRs). PPTX slide-grouping sections (`<p14:sectionLst>`): full pipeline DSL→parser→eval(single-pass per-instance-id membership)→layout passthrough→`SectionListBuilder::inject`. SlideSectionEntry in slideforge-types (re-exported via layout). Deterministic RFC4122 v5 GUIDs. E-PAR-023 (empty name, fatal exit 1), W-PAR-002 (duplicate, warning exit 0, both emitted). CONVERGED 3/3 strict-CLEAN (passes 16-17-18) after 18-pass LOCAL cascade. Findings caught+fixed: adjacent-duplicate section merge (HIGH, per-instance-id fix), SEC-100/CWE-116 section-name sanitization (HIGH), vacuous AC-010 tests (TD-VSDD-059), E-PAR-023 exit 2→1 spec (BC v1.4, taxonomy v2.28), sha2 pin =0.11.0 spec. Security CLEAN (1 LOW pre-existing, 1 SUGGESTION); pr-reviewer APPROVE (2 non-blocking OBS); CI 25/25 green. Follow-ups: FU-082-SEC-S001-NUL-CALLSITE-TEST (NUL-byte call-site test via build_ext_lst). |
| 2026-06-09 | STORY-082-OBS-P15-1 | Adversary Pass-15 OBS-P15-1 (LOW, spec-location anchor): STORY-082 spec showed `SlideSectionEntry` defined in `slideforge-layout`; actually defined in `slideforge-types/src/deck.rs`, re-exported via `slideforge-layout/src/lib.rs`. Corrected across 6 spec locations (Subsystem Anchor, Scope Overview §2, Tasks, File Structure table, Forbidden Dependencies, Test Strategy); story spec_version 1.2→1.3. Worktree code unchanged (separately, test fix F-P15-MED-1 landed at HEAD e40a410f). |
| 2026-06-08 | STORY-082-F-P8-MED1 | Adversary Pass-8 F-P8-MED-1 (MEDIUM, spec-text defect — code correct): STORY-082 spec cited stale `sha2 =0.10.9` in 4 places vs canonical workspace pin `=0.11.0` (shared w/ slideforge-math). Fixed all 4 + reframed to `workspace = true`; full version-pin sweep (quick-xml/ooxmlsdk/chumsky/zip all match). story spec_version 1.1→1.2. Worktree code unchanged (67530b17). Streak reset 0/3 by Pass 8; Pass 9 next against HEAD 67530b17. |
| 2026-06-08 | STORY-082-IMP1-SPECFIX | Adversary Pass-5 IMP-1 (HIGH, spec defect — code was correct): E-PAR-023 (empty section name) is a parse error → exit 1 per BC-1.15.003 three-tier model, but story AC-010/EC-010, BC-4.01.003 PC7/EC-010, and error-taxonomy E-PAR-023 row wrongly said exit 2. Corrected all 3 artifacts: BC-4.01.003 v1.3→v1.4, error-taxonomy v2.27→v2.28 (+OBS-1 message-prefix), STORY-082 spec_version 1.1. Worktree code UNCHANGED (HEAD 67530b17). STORY-082 streak reset to 0/3; next adversary Pass 6 against corrected spec. |
| 2026-06-08 | STORY-072-MERGE | PR #76 squash-merged → develop `2667987e` (76 merged PRs). Shape gradient fills: `FillSpec::Gradient { from, to }` added to slideforge-types; E-PAR-016 path removed; wired parser→IR→layout passthrough→4 exporters (PPTX `<a:gradFill>`, PDF krilla LinearGradient, HTML SVG `<linearGradient>`+`url()`, DOCX solid fallback+warn). LOCAL 3/3 strict-CLEAN (passes 4-5-6). Security CLEAN (2 informational suggestions); pr-reviewer APPROVE (3 non-blocking nits); CI 25/25 green. Rebased onto develop resolving STORY-074 `font_size_emu` interaction (LESSON-19 sibling sweep: gradient_integration.rs import + 3 BrandFonts test literals). KNOWN DEFERRAL intact: ShapeNode→ShapeSpec decode + end-to-end DSL path NOT wired (FU-SHAPE-PIPELINE-WIRING wave-gate). Follow-ups registered: FU-072-SEC002-FILLATTR-INVARIANT (add doc-comment invariant on render.rs fill_attr for pre-escaped values), FU-072-PDF-PUBCRATE (tighten draw_gradient_rect pub→pub(crate)), FU-072-EPAR016-DOC-STALE (E-PAR-016 description stale post-STORY-072 — cleanup when convenient). SEC-001 → links to existing FU-SHAPE-PIPELINE-WIRING (eval decode of gradient string); no duplicate created. |
| 2026-06-08 | STORY-047-MERGE | PR #75 squash-merged → develop `95f23df3` (75 merged PRs). Web Preview Server + CSP nonce security hardening: axum WS endpoint; CSP nonce per-response; BC-4.03.004. LOCAL 3/3 strict-CLEAN (passes 4-6). Security CLEAN; pr-reviewer APPROVE; CI green (disk-space infra flake on snapshots job cleared on re-run — NOT a code defect; FU-047-DEPS-AWSLC registered). Follow-ups: FU-047-SEC005-PATH (DiagnosticMessage.file path normalization), FU-047-DF1-CLIENT-RECONCILE, FU-047-DF2-SCR007-CHROME, FU-047-ADR008-AXUM, FU-047-DEADFN (dead pub ws_upgrade_handler), FU-047-DEPS-AWSLC. STORY-056 + STORY-048 now UNBLOCKED (←047 merged). |
| 2026-06-08 | STORY-074-MERGE | PR #74 squash-merged → develop `3f7f99ed` (74 merged PRs). Brand-aware em sizing: `font_size_emu` (i64) on `Brand` drives em→EMU shape resolution; `DEFAULT_EM_IN_EMU` removed from compiled lib (confined to `#[cfg(test)]`); backward-compatible default 457_200. LOCAL 3/3 strict-CLEAN (passes 11-13; 13 total — code converged since pass 3). Security CLEAN (3 LOW deferrals), pr-reviewer APPROVE, CI green. Follow-ups: FU-074-SEC003-PALETTE-FALLBACK, FU-047-ADR008-AXUM. |
| 2026-06-08 | STORY-080-MERGE | PR #73 squash-merged → develop `e08f2f80` (73 merged PRs). Deflake cross-platform tests. cold_budget PERMANENTLY FIXED. LOCAL 3/3 (passes 5-6-7). Security CLEAN, pr-reviewer APPROVE, CI green. |
| 2026-06-08 | STORY-079-MERGE | PR #72 squash-merged → develop `2f6d5da4` (72 merged PRs). slideforge-diagrams SVG DoS hardening. BC-1.12.003 v1.3. error-taxonomy v2.26. LOCAL 3/3 (passes 3-4-5). Security CLEAN, pr-reviewer APPROVE, CI green. |
| 2026-06-08 | WAVE5-FULLWIDTH-LAUNCH | Human confirmed "full 8 parallel now". 8 per-story worktrees created off develop cbebfd57. All 8 stories Stage-1 launched in parallel. Active worktrees: STORY-082/088/072/074/079/080/047/081. Merge model: parallel development, SERIAL merge with rebase + re-gate. |
| 2026-06-08 | STORY-055-MERGE | PR #71 squash-merged → develop cbebfd57. `slideforge build` CLI + miette diagnostics. Unified compile pipeline. LESSON-21 + LESSON-22 codified. CI all-green. |
| 2026-06-08 | STORY-046-MERGE | PR #70 squash-merged → develop fa85d113. Static HTML exporter. P4 Composite Rendering Model. ADR-008 P4 amendment. BC-4.03.003 v1.4. 23-pass cascade. CI all-green. |

---

## Quality Bar (Non-Negotiable Gates)

Production-grade from day 1. Full table in CLAUDE.md. `#![forbid(unsafe_code)]`; zero `.unwrap()` outside tests; `clippy::pedantic`; `#![warn(missing_docs)]`; Kani+fuzz+mutants (Phase 6); WCAG AA; PDF/UA-1; < 500ms cold build; signed releases; SBOM; cross-platform macOS+Linux+Windows.
