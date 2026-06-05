---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-04
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
total_stories: 85
total_points: 511
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
wave_4_merged: 16
wave_4_started: 2026-05-31
wave_4_total_stories: 21
wave_4_total_points: 129
wave_5_total_points: 114
develop_sha: "869fb401"
develop_pr_count: 56
error_taxonomy_version: "v2.13"
workspace_tests: "~3013 (56 merged PRs)"
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

Phase 3, **Wave 4 — 16/21 merged. Batch B pptx chain (037→038→039→040) COMPLETE. Batch B docx chain (041→042) DONE.**

- `develop` = `869fb401` (56 merged PRs; origin/develop = local develop). **Open PRs: 0. Active worktrees: none.**
- Workspace builds clean. ~3013 tests pass, 0 failures.
- STORY-040 MERGED PR #56 (869fb401, 2026-06-04). Speaker notes (`.rels` + NotesSlidePart + `<p:notes>`), notesMaster1.xml, handoutMaster1.xml. 7-pass adversary cascade; strict-CLEAN on passes 5/6/7. SafeUrl guard (link_safety.rs) shipped — CWE-601 defense-in-depth for notes hyperlinks. Re-scoped 5→3 pts; slide-grouping sections split to STORY-082 (Wave 5, human-authorized).
- STORY-039 MERGED PR #55 (a4f29e5a). STORY-037 PR #52 (2ebf184f). STORY-038 PR #54 (c031805c). STORY-041 PR #51 (a3b47303). STORY-042 PR #53 (56f3f57d).

**Batch B COMPLETE.** **LESSON-13 reconciliation completed 2026-06-04** — STORY-049 was found NOT implementation-ready (plugin registry RegistryBuilder + surface ownership not defined; SectionType/InlineFormat bundled impls missing). Three prerequisite stories added (STORY-083/084/085, ADR-016 authored). Wave 4 now has 21 stories (was 18). **Batch C (next):** STORY-083/084/085 (parallel-eligible) → STORY-049 → STORY-050.
**Wave 4 gate** runs only after ALL 21 Wave 4 stories merge. **STORY-082** (slide-grouping sections) → Wave 5.

---

## NEXT ACTIONS (fresh orchestrator — execute in order)

**Batch C — LESSON-13 reconciliation applied. Three prerequisites added before STORY-049.**

1. **STORY-083** — Plugin Registry Builder + Surface Enforcement (slideforge-plugin-api, 3 pts; dep STORY-002 only; parallel-safe). Full per-story delivery flow. Defines `RegistryBuilder`, `RegistryError::MissingSurface`, `surface_count()`, `surface_names()`.
2. **STORY-084** — Bundled SectionType Implementations (slideforge-plugin-api, 3 pts; deps STORY-002/042/077). Can run **parallel** to STORY-083 (both dep-free from Batch B COMPLETE).
3. **STORY-085** — Bundled DefaultInlineFormat + PPTX OOXML dog-fooding refactor (slideforge-plugin-api + slideforge-pptx, 8 pts; deps STORY-002/028/037/038). Can run **parallel** to STORY-083.
4. **STORY-049** — Plugin Registry Assembly (root crate, pipeline driver, exposes `build()`; 5 pts) — **GATED on STORY-083 + STORY-084 + STORY-085 ALL merged**.
5. **STORY-050** — End-to-End Integration Test Suite — gated on STORY-049.
6. **Wave 4 gate** after all 21 stories merged (16/21 done; remaining: STORY-083, STORY-084, STORY-085, STORY-049, STORY-050).
7. **STORY-082** — PPTX Slide-Grouping Sections (Wave 5, 5 pts, P0, BC-4.01.003 Half B). After Wave 4 gate passes.

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
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (109 BCs, 15 HS, 4 supplements) + arch (14 ADRs, 15 VPs, 20 crates) + UX spec. 17 passes, 69 findings, 3/3 clean. |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 85 stories, 21 epics, 6 waves, 511 pts (LESSON-13 reconciliation: +4 stories/+14 pts added 2026-06-04). 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3 GATE PASSED. Wave 4: 16/21 merged. Batch B COMPLETE (037→040, 041→042). Batch C: STORY-083/084/085→049→050. | Per-story delivery |
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

**Batch C (next):** STORY-083 + STORY-084 + STORY-085 (parallel-eligible) → STORY-049 → STORY-050
- STORY-083: Plugin Registry Builder (3 pts, slideforge-plugin-api; dep STORY-002 only)
- STORY-084: Bundled SectionType Implementations (3 pts; deps STORY-002/042/077)
- STORY-085: Bundled DefaultInlineFormat + PPTX dog-fooding (8 pts; deps STORY-002/028/037/038)
- STORY-049: Plugin Registry Assembly — root crate pipeline driver (5 pts; gates on 083+084+085)
- STORY-050: E2E Integration Test Suite (gates on STORY-049)
**STORY-082** (slide-grouping sections) moved to Wave 5 (human-authorized split from STORY-040)

---

## Session Resume Checkpoint

**CLEAN CHECKPOINT — safe to clear context and resume in a fresh session. No in-flight worktree/PR. A fresh orchestrator resumes by reading this STATE.md and starting at STORY-083 (Batch C first step — see NEXT ACTIONS).**

| Field | Value |
|-------|-------|
| **Date** | 2026-06-04 |
| **Position** | Wave 4: 16/21 merged. Batch B COMPLETE. LESSON-13 reconciliation applied — STORY-083/084/085 added as Batch C prerequisites. STORY-049 gated on all three. NEXT: start STORY-083 (parallel-eligible with 084/085). |
| **develop SHA** | `869fb401` (56 merged PRs; origin/develop = local develop) |
| **Active worktrees** | none |
| **Open PRs** | 0 |
| **Workspace crates** | 16 |
| **Spec deltas this session** | STORY-040 re-scoped 5→3 pts (slide-grouping split to STORY-082, human-authorized). STORY-082 → Wave 5 (5 pts, P0, BC-4.01.003 Half B). BC-4.01.003 v1.2 (Half A notes+masters; Half B deferred). BC-5.01.005 v1.2 (deck.metadata.lang SoT). LESSON-13 reconciliation 2026-06-04: ADR-016 authored; BC-5.02.001→v1.3 (invariant 3 + postcondition 2 counts); BC-5.02.002→v1.3 (PC-5/EC-004 OOXML dog-fooding); plugin-architecture.md/ARCH-INDEX.md/crate-architecture.md corrected (owner-crate + root-crate definitions); STORY-083/084/085 added (14 pts new); STORY-049 amended (5 pts, gates on 083/084/085). Total stories 81→85; total points 497→511; Wave 4 stories 18→21; Wave 4 points 115→129. |
| **factory-artifacts** | PUSHED to remote (origin/factory-artifacts) — human-authorized 2026-06-04. Upstream tracking set. Fresh sessions: clone repo + `git worktree add .factory factory-artifacts`. |

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

---

## Open Follow-Ups (non-blocking)

| Item | Severity | Target |
|------|----------|--------|
| SEC-042-001 (CWE-400): docx section serializers no upper bound on items count | LOW | STORY-049 / layout hardening |
| SEC-001 (CWE-494, veraPDF): Docker `verapdf/cli:latest` not digest-pinned | MED | Before v1.0 / Phase 6 |
| SEC-003 (CWE-189): `emu_to_pt` i64→f32 precision loss | LOW | Phase 6 Kani |
| SEC-004: CI tee predictable temp path (self-hosted only) | LOW | Phase 6 |
| SEC-037-001: pptx InlineNode::Link (slide body) must adopt `is_safe_link_scheme` from link_safety.rs (SafeUrl guard now EXISTS in slideforge-pptx — wire slide-body Link path). CWE-601. | LOW | pptx link-rendering story / Phase 5 |
| SafeUrl shared newtype (pptx/pdf/html will hit same docx SEC-001 gap) | cross-exporter | Hardening story before Phase 6 |
| cold_budget flaky perf test (test_cold_budget_under_200ms, slideforge-diagrams): macOS CI jitter | LOW | STORY-080 |
| OBS-FU-HTML-REDIR: HTML exporter URL open-redirect (CWE-601) | security | STORY-046 |
| E-PAR-021 cosmetic message format (nesting_depth_exceeded_msg raw byte offset) | minor | Message-cleanup follow-up |
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

---

## Decisions Log

| Date | ID | Decision |
|------|-----|---------|
| 2026-06-04 | LESSON-13-STORY-049 | STORY-049 LESSON-13 reconciliation (human-authorized 2026-06-04). STORY-049 found NOT implementation-ready: RegistryBuilder/surface enforcement absent from BC-5.02.001; SectionType (7 impls) + InlineFormat (12 impls) bundled ownership undefined; root crate vs owner-crate confusion in arch docs. Three decisions: (A) code-conforms-to-spec — RegistryBuilder + RegistryError::MissingSurface + surface_count/surface_names → STORY-083 (slideforge-plugin-api, 3 pts); (B) SectionType (7) + InlineFormat (12) bundled impls owned by slideforge-plugin-api → STORY-084 (3 pts) + STORY-085 (8 pts); (C) root crate = pipeline driver exposing build(), NOT owner-crate. Artifacts: ADR-016 authored; BC-5.02.001→v1.3 (invariant 3 + postcondition 2 counts corrected); BC-5.02.002→v1.3 (PC-5/EC-004 OOXML dog-fooding); plugin-architecture.md/ARCH-INDEX.md/crate-architecture.md corrected; reconciliation assessment at .factory/planning/story-049-reconciliation-assessment.md. Wave 4: 18→21 stories, 115→129 pts. Total: 81→85 stories, 497→511 pts. |
| 2026-06-04 | STORY-040 | STORY-040 MERGED PR #56 (869fb401) — Batch B pptx chain complete (037→038→039→040). Slide-grouping split to STORY-082 (human-authorized). SafeUrl guard (link_safety.rs is_safe_link_scheme, CWE-601) shipped. SEC-040-001 (URL safety + XML escaping for notes hyperlinks) verified via test — ooxmlsdk escapes correctly, no prod change required. |
| 2026-06-04 | SEC-039 | SEC-039-001 (CWE-116, MED) + SEC-039-002 (CWE-754, LOW) FIXED IN-SCOPE during STORY-039 PR review — validate_lang_for_xml rejects XML-1.0-illegal control chars in dc:language; loud tracing::error fallback for unexpected AltText variants. Neither deferred. |
| 2026-06-04 | BC-5.01.005-v1.2 | Invariant 2 amended: lang SoT corrected to `deck.metadata.lang` (`DeckMetadata.lang`) — human-authorized (SoT rule 7), architect-recommended Option 2. `LaidOutDeck` carries no lang field and will not gain one; all exporters read lang via `Exporter` trait `deck: &Deck` param. No code change — impl was already correct. 5 stale story-spec refs corrected (STORY-017, STORY-039 ×3, STORY-041); 1 residual test doc-comment (a11y_tests.rs:831) → STORY-040 drive-by. |

---

## Quality Bar (Non-Negotiable Gates)

Production-grade from day 1. Full table in CLAUDE.md. `#![forbid(unsafe_code)]`; zero `.unwrap()` outside tests; `clippy::pedantic`; `#![warn(missing_docs)]`; Kani+fuzz+mutants (Phase 6); WCAG AA; PDF/UA-1; < 500ms cold build; signed releases; SBOM; cross-platform macOS+Linux+Windows.
