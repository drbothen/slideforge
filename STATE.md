---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-05
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
wave_4_merged: 19
wave_4_started: 2026-05-31
wave_4_total_stories: 21
wave_4_total_points: 129
wave_5_total_points: 114
develop_sha: "e704e700"
develop_pr_count: 59
error_taxonomy_version: "v2.13"
workspace_tests: "~3214 (59 merged PRs; STORY-049 rounds 2-4 fixed — build() end-to-end Ok path confirmed; 3214/3215 pass, 1 pre-existing cold_budget flake)"
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

Phase 3, **Wave 4 — 19/21 merged. STORY-049 IN PROGRESS (passes 5-8 complete; 0/3 strict-CLEAN; passes 9-11 pending).**

- `develop` = `e704e700` (59 merged PRs; origin/develop confirmed). **Open PRs: 0. Active worktrees: .worktrees/STORY-049.**
- Workspace: ~3214/3215 pass (1 pre-existing cold_budget flake). 0 CI failures.
- STORY-049 code HEAD: 70622461. Pass 5 strict-CLEAN. Pass 6: 3 doc defects (DSL doctest syntax, inverted brand-provider comment, missing validate stage in rustdoc) — FIXED commit 1c8af9a2. Pass 7 strict-CLEAN. Pass 8: F-PASS8-001 (panic=unwind safety perimeter had no CI enforcement despite Architecture Compliance Rule 4 being stated) — FIXED by adding scripts/check-panic-profile.sh + ci.yml check-panic-profile job wired into all-checks-pass (commit 70622461, pass verified + fail-when-violated verified). Streak: 0/3 strict-CLEAN (behavioral dims converged; remaining finds peripheral). Passes 9-11 pending.
- STORY-085 MERGED PR #59 (e704e700, 2026-06-05). STORY-084 MERGED PR #58 (801f351b). STORY-083 MERGED PR #57 (5aaa27d2).

**Batch C COMPLETE (3/3):** STORY-083 + STORY-084 + STORY-085 ALL MERGED. Remaining Wave 4: STORY-049 (in-progress) → STORY-050.
**Wave 4 gate** runs only after ALL 21 Wave 4 stories merge. **STORY-082** (slide-grouping sections) → Wave 5.

---

## NEXT ACTIONS (fresh orchestrator — execute in order)

**STORY-049 IN PROGRESS. Passes 5-8 complete. Passes 9-11 pending (need 3/3 strict-CLEAN before demo-recorder).**

1. **STORY-049 adversary pass 9** — Dispatch adversary (absolute cwd: `/Users/jmagady/Dev/slideforge/.worktrees/STORY-049`) for pass 9. Code HEAD: 70622461. Pass 8 finding F-PASS8-001 FIXED (scripts/check-panic-profile.sh + ci.yml check-panic-profile job). Behavioral dimensions converged at passes 5+7; remaining findings have been peripheral (docs, CI enforcement). Streak: 0/3 strict-CLEAN. MUST reach 3/3 before demo-recorder.
2. **STORY-049 demo-recorder** — after 3/3 strict-CLEAN, record demo evidence per AC-001 through AC-008.
3. **STORY-049 pr-manager** — full 9-step PR cycle after demo evidence.
4. **STORY-050** — End-to-End Integration Test Suite (8 pts, P0) — gated on STORY-049 merge.
5. **Wave 4 gate** after all 21 stories merged (19/21 done; remaining: STORY-049, STORY-050).
6. **STORY-082** — PPTX Slide-Grouping Sections (Wave 5, 5 pts, P0, BC-4.01.003 Half B). After Wave 4 gate passes.

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
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3 GATE PASSED. Wave 4: 19/21 merged. Batch C COMPLETE (083+084+085 all merged). Remaining: STORY-049 (UNGATED) → STORY-050 → Wave 4 gate. | Per-story delivery |
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

**Batch C — ALL MERGED (3/3):**
- STORY-083: Plugin Registry Builder — MERGED PR #57 (5aaa27d2, 2026-06-05) — 6-pass cascade, 3/3 strict-CLEAN
- STORY-084: Bundled SectionType Implementations — MERGED PR #58 (801f351b, 2026-06-05) — 7-pass cascade, 3/3 strict-CLEAN
- STORY-085: Bundled DefaultInlineFormat + PPTX dog-fooding — MERGED PR #59 (e704e700, 2026-06-05) — 9-pass cascade, 3/3 strict-CLEAN (passes 7-8-9). 20/20 CI green; security APPROVE/CLEAN; pr-reviewer APPROVE.
- STORY-049: Plugin Registry Assembly — root crate pipeline driver (5 pts; IN PROGRESS — passes 5-8 complete; passes 9-11 pending; 0/3 strict-CLEAN). Active worktree: .worktrees/STORY-049. Code HEAD: 70622461.
- STORY-050: E2E Integration Test Suite (gates on STORY-049)
**STORY-082** (slide-grouping sections) moved to Wave 5 (human-authorized split from STORY-040)

---

## Session Resume Checkpoint

**IN-FLIGHT CHECKPOINT — STORY-049 passes 5-8 complete; pass 9 pending. Resume: dispatch adversary pass 9 in worktree .worktrees/STORY-049 (absolute path). Streak 0/3 strict-CLEAN.**

| Field | Value |
|-------|-------|
| **Date** | 2026-06-05 |
| **Position** | Wave 4: 19/21 merged. STORY-049 IN PROGRESS — passes 5-8 complete. Pass 5: strict-CLEAN. Pass 6: 3 doc defects (DSL doctest syntax, inverted brand-provider comment, missing validate stage in rustdoc) — FIXED commit 1c8af9a2. Pass 7: strict-CLEAN. Pass 8: F-PASS8-001 (panic=unwind safety perimeter had no CI enforcement despite Architecture Compliance Rule 4) — FIXED: added scripts/check-panic-profile.sh + ci.yml check-panic-profile job wired into all-checks-pass (commit 70622461; pass verified + fail-when-violated verified). Code HEAD: 70622461. Streak: 0/3 strict-CLEAN (behavioral dims converged passes 5+7; remaining finds peripheral). Passes 9-11 pending. |
| **develop SHA** | `e704e700` (59 merged PRs; origin/develop confirmed) |
| **Active worktrees** | `.worktrees/STORY-049` (branch: feature/STORY-049) |
| **Open PRs** | 0 |
| **Workspace crates** | 17 (slideforge root crate now populated) |
| **Spec deltas this session** | STORY-049: strict defaults true (was false, spec contradiction); NoBrandSource/UnknownFormat tested; panic=unwind explicit on [profile.dist]; inject_lang_default wired per slideforge-validate contract; BuildError::Export carries #[source]; extension() from exporter not format key. Pass 6: doc fixes (DSL doctest syntax, inverted brand-provider comment, missing validate stage in rustdoc). Pass 8: scripts/check-panic-profile.sh + ci.yml check-panic-profile job added (Architecture Compliance Rule 4 now CI-enforced). |
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
| LESSON-16 | PRE-PUSH GATE MUST MIRROR EXACT CI INVOCATIONS. Local per-story gates that run a narrower clippy (`cargo clippy -p <crate> --all-targets -- -D warnings`) and skip the docs gate will miss CI-catching defects. STORY-083 PR #57 first CI run FAILED on: (a) pedantic clippy escalated to deny caught `doc_markdown` + `unnecessary_literal_bound` + `uninlined_format_args` in the demo example binary; (b) `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` caught a broken intra-doc link `[Err(RegistryError::MissingSurface)]`. Fix commit bc52da1a was required before CI went green. **Remediation:** before push, orchestrator/implementer MUST run the EXACT canonical CI commands: `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items` AND `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` — especially when a story adds an example binary or new intra-doc links. Also note (process-gap): adversary pass-3 first attempt produced a false "non-deterministic file" report because it read the main-checkout path instead of the worktree path; re-dispatch with explicit worktree-absolute path discipline resolved it. Going-forward adversary dispatches in worktree stories MUST pin worktree-absolute paths (re-enforces LESSON-1). |
| LESSON-17 | INVERTED RED GATE ANTI-PATTERN (`#[should_panic]` PLACEHOLDER). A `#[should_panic]` test wrapping a `todo!()` call PASSES against stub code (the `todo!()` panic satisfies `should_panic`) and then BREAKS on correct implementation. This is the opposite of TDD Red Gate: tests must FAIL on stubs and PASS on implementation. Evidence: STORY-084 test-writer initially submitted `#[should_panic]` placeholders that gave false GREEN against `todo!()` stubs. Corrected before implementer dispatch by converting to behavioral `assert_eq!`/`assert!` tests that properly FAIL on stubs. **Rule:** test-writer MUST NEVER write `#[should_panic]` as a placeholder for behavioral correctness tests. `#[should_panic]` is valid only for testing deliberate panic-on-invalid-input paths — and those tests must also verify they FAIL on the stub. |

---

## Open Follow-Ups (non-blocking)

| Item | Severity | Target |
|------|----------|--------|
| OBS-E (STORY-049 pass-4): STORY-050 E2E must include multi-slide deck with inline formatting + data binding routed through build() to close BC-5.02.002 EC-004 end-to-end — build()'s own tests cover only a trivial title slide (appropriate; E2E owns deep coverage). Anchored to STORY-050 as required scope. | STORY-050 scope | STORY-050 |
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
