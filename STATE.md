---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-03
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
wave_4_batch_a_complete: 10
wave_4_batch_a_total: 10
wave_4_started: 2026-05-31
wave_4_total_stories: 18
wave_4_total_points: 114
wave_5_total_points: 109
develop_sha: "2ebf184f"
develop_pr_count: 52
error_taxonomy_version: "v2.13"
workspace_tests: "~2700+ (50 PRs merged)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## What Is This Project?

slideforge is a DATA-REACTIVE BRANDED DOCUMENT PLATFORM. Generates branded .pptx, .docx, .pdf, .html, and web preview from a single .sf DSL file with data binding, iteration, conditionals, and a plugin-first architecture.

**Repository:** https://github.com/drbothen/slideforge
**Workspace:** /Users/jmagady/Dev/slideforge
**Factory worktree:** .factory/ on branch `factory-artifacts`

---

## POSITION

Phase 3, **Wave 4 Batch B — chain HEADS COMPLETE: STORY-037 MERGED (PR #52, 2ebf184f) + STORY-041 MERGED (PR #51, a3b47303).**
- develop = `2ebf184f` (52 merged PRs). Open PRs: 0. Active worktrees: none.
- STORY-037: converged 3/3 strict-CLEAN (passes 6/7/8), merged PR #52 (2ebf184f). STORY-038→039→040 now unblocked.
- STORY-041: converged 3/3 strict-CLEAN (passes 3/4/5), merged PR #51 (a3b47303). STORY-042 now unblocked.
- **Batch B remaining:** STORY-038→039→040 (pptx chain, unblocked by STORY-037) + STORY-042 (docx, unblocked by STORY-041). All can dispatch in parallel (STORY-038 and STORY-042 independent; each chain is sequential internally).
- **Wave 4 merged total:** 12/18 (Batch A 10/10 + STORY-041 + STORY-037).

---

## STANDING AUTHORIZATIONS

**STANDING MERGE AUTH (granted by human 2026-06-02):** The orchestrator MAY auto-merge any PR that is fully CI-green + security-reviewer CLEAN + pr-reviewer APPROVE, WITHOUT asking the human each time (revocable by human). This RELAXES LESSON-6 for converged PRs. LESSON-6's "GitHub blocks author self-approve" means the orchestrator does `gh pr merge --squash` itself once those gates pass.

---

## Phase Progress

| Phase | Status | Key Output |
|-------|--------|-----------|
| Pre-pipeline | DONE | Toolchain + LLM + MCP preflight (2026-05-23) |
| Market intelligence | DONE 2026-05-23 | GO with medium confidence |
| Planning (25 DSL decisions) | DONE 2026-05-24 | q1–q25 docs + 14 research threads + 7/7 spikes resolved |
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (109 BCs, 15 HS, 4 supplements) + arch (14 ADRs, 15 VPs, 20 crates) + UX spec. 17 passes, 69 findings, 3/3 clean. |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 81 stories, 21 epics, 6 waves, 491 pts. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3 GATE PASSED. Wave 4: Batch A 10/10 COMPLETE + STORY-041 MERGED (PR #51 a3b47303) + STORY-037 MERGED (PR #52 2ebf184f). 12/18 Wave 4 merged. Batch B remaining: STORY-038→039→040 (pptx chain) + STORY-042 (docx). | Per-story delivery |
| Phases 4–7 | NOT STARTED | Holdout / Adversarial / Formal Hardening / Convergence |

## Wave 4 Batch A Status (10/10 COMPLETE) + Batch B In Progress

**Batch A ALL MERGED** (PRs #39–#49, develop c8913cad): STORY-035, STORY-036, STORY-043, STORY-044, STORY-073, STORY-075, STORY-076, STORY-078, STORY-045, **STORY-077**.
STORY-077 follow-ups MERGED as PR #50 (f2573bb1, 2026-06-03).

**Batch B:**
- STORY-041 MERGED PR #51 (a3b47303, 2026-06-03). STORY-042 now unblocked.
- STORY-037 MERGED PR #52 (2ebf184f, 2026-06-03). STORY-038→039→040 now unblocked. ADR-015 + Addendum A on factory-artifacts. PR-52 follow-ups recorded in Follow-Ups table (SEC-037-001, S1–S4).

Batch C (after Batch B): STORY-049→050.

---

## Session Resume Checkpoint

| Field | Value |
|-------|-------|
| **Date** | 2026-06-03 |
| **Position** | Wave 4 Batch B — chain heads complete. STORY-037 MERGED PR #52 (2ebf184f). STORY-041 MERGED PR #51 (a3b47303). Wave 4: 12/18 merged. No open PRs, no active worktrees. Next: dispatch STORY-038→039→040 (pptx chain) + STORY-042 (docx) in parallel. |
| **develop SHA** | `2ebf184f` (52 merged PRs) |
| **Active worktrees** | none |
| **Open PRs** | 0 |
| **Workspace crates** | 16 |
| **BC deltas** | ADR-015 + Addendum A this session; BC-3.02.002 v1.5, BC-1.14.003 v1.3, error-taxonomy v2.13 — all on factory-artifacts |
| **factory-artifacts** | Local only — push requires explicit human authorization |

---

## Follow-Ups / Drift (non-blocking)

| Item | Severity | Status | Notes |
|------|----------|--------|-------|
| STORY-081: slide-level inline markup | P0 blocker for v1.0 | draft | Wave 5, 13 pts, EPIC-18. Depends on STORY-077 (now merged). Not started. |
| SEC-001: veraPDF Docker `verapdf/cli:latest` not digest-pinned (CWE-494) | MED | open | CI-only; merged in #48; fix before v1.0 / Phase 6 |
| SEC-002: link URL allowlist | LOW | **RESOLVED** | Fixed at parse boundary via E-PAR-022 (http/https/mailto allowlist) in PR #50 / f2573bb1. Closed earlier than STORY-046 deferral. STORY-046 retains the open-redirect (CWE-601) concern for the HTML exporter layer — see OBS-FU-HTML-REDIR below. |
| SEC-003 (formerly SEC-002): `emu_to_pt` i64→f32 precision loss | LOW | open | Phase 6 Kani |
| SEC-004 (formerly SEC-003): CI tee predictable temp path (self-hosted only) | LOW | open | Phase 6 |
| E-EVL-007..011 unregistered + E-PAR-012 retired-code reuse | LOW | **RESOLVED** | Registered/reconciled in error-taxonomy v2.13 (factory-artifacts 7f2e52d7, PR-independent). Taxonomy now exhaustive for these codes. |
| OBS-077-P24-A: `_` italic bilateral flanking | LOW | **RESOLVED** | Bilateral flanking shipped in PR #50 / f2573bb1 (DIR-077-002 §1 amended). |
| OBS-077-P25-A: parse→eval round-trip test | LOW | **RESOLVED** | Round-trip test added in PR #50 / f2573bb1. |
| E-PAR-021 message format cosmetic | minor | tracked | `nesting_depth_exceeded_msg` embeds raw byte offset + double-prints prefix. Pre-existing; affects only the rare E-PAR-021 path. Target: message-cleanup follow-up (no story dependency). |
| OBS-FU-HTML-REDIR: HTML exporter URL open-redirect (CWE-601) | security | deferred to STORY-046 | Parse-time allowlist validates outer scheme only. A query-embedded redirect (`https://trusted/redir?to=javascript:evil`) is NOT caught at parse (correct). HTML exporter must handle `href` safety when rendering `InlineNode::Link`. Attach to STORY-046. |
| OBS-FU-P1-A: DSL inner-quote round-trip | minor | tracked | `Token::StringLit` retains backslash-escapes verbatim — no unescape pass. `detail: "ref(\"x\")"` cannot round-trip nested escaped quotes. Pre-existing lexer follow-up. |
| OBS-FU-P1-C: Link URL `)` truncation | minor | tracked | Link URLs truncated at first `)` — Wikipedia disambiguation links mis-parsed. Pre-existing link-parser follow-up. |
| BC-1.14.001/002 still `subsystem: SS-TBD` | LOW | open | Fold into next spec-hygiene pass |
| OBS-P6-001: PDF exporter ignores `opts.strict`/warnings | LOW | open | Wave-gate concern post-STORY-045 |
| SEC-003 (docx): docx exporter URL-scheme test-doc over-claims vbscript:/file: coverage; add explicit rejection tests | LOW | open | Target: next docx touch / STORY-042 |
| SafeUrl newtype: pptx/pdf/html exporters will hit the same SEC-001 gap as docx; consider shared slideforge-types SafeUrl across all exporters | cross-exporter | tracked | Dedicated hardening story before Phase 6 |
| cold_budget flaky perf test (test_cold_budget_under_200ms, slideforge-diagrams): intermittent on macOS CI (332ms vs 300ms budget under runner jitter); passes on re-run | LOW | tracked | Stabilize budget/runner or mark perf-tolerant; target STORY-080 |
| STORY-038 spec stale ref: line ~223 references `Brand.layout_xmls` (stale field name, same issue ADR-015 corrected for STORY-037); story-writer must fix before STORY-038 impl | spec-hygiene | open | Fix in story spec before dispatching STORY-038 |
| DEF-041-P5-001: EC-002 paragraph-break preservation depends on upstream STORY-035 evaluator splitting — exporter correct given input contract | integration | deferred to wave-gate | STORY-041 adversary pass 5 deferred finding |
| SEC-037-001: pptx InlineNode::Link rendering needs ALLOWED_LINK_SCHEMES forward-guard + SafeUrl (CWE-601 defense-in-depth; mirrors docx SEC-001) | LOW | open | PR #52 reviewer suggestion. Target: pptx link-rendering story / shared SafeUrl hardening. |
| S1 (pptx): `validate_emu` Err path (slide_serializer.rs:275) untested — add negative-width unit test | LOW | open | PR #52 reviewer suggestion. Target: STORY-038. |
| S2 (pptx): silent i32 slide-size clamp (presentation.rs:159-160) at odds with no-silent-fallback rule — propagate error instead | LOW | open | PR #52 reviewer suggestion. Target: STORY-038. |
| S3 (pptx): Subtitle frames map to PlaceholderValues::Title (idx 0) not a subTitle placeholder — confirm/resolve | LOW | open | PR #52 reviewer suggestion. Target: STORY-038. |
| S4 (pptx): `build_notes_handout_masters` swallows rels-build failure with empty bytes (lib.rs:632,650) — propagate error instead | LOW | open | PR #52 reviewer suggestion. Target: STORY-040 (notes/handout master story). |

---

## Standing Process Rules (reminders for fresh session)

| ID | Rule |
|----|------|
| LESSON-1 | Adversary dispatches MUST pass the ABSOLUTE worktree path (`--cwd /Users/jmagady/Dev/slideforge/.worktrees/STORY-NNN`). |
| LESSON-2 | Canonical clippy gate: `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items`. Bare `-D warnings` misses pedantic lints. |
| LESSON-3 | Re-run full pre-push gate (fmt + pedantic clippy + nextest) after EVERY commit to a feature branch before PR. |
| LESSON-5 | pr-manager CANNOT spawn sub-agents. Orchestrator dispatches security-reviewer + pr-reviewer independently. |
| LESSON-6 | RELAXED by STANDING MERGE AUTH: orchestrator may merge when CI-green + security-reviewer CLEAN + pr-reviewer APPROVE. |

---

## Quality Bar (Non-Negotiable Gates)

Production-grade from day 1. Full table in CLAUDE.md. `#![forbid(unsafe_code)]`; zero `.unwrap()` outside tests; `clippy::pedantic`; `#![warn(missing_docs)]`; Kani+fuzz+mutants (Phase 6); WCAG AA; PDF/UA-1; < 500ms cold build; signed releases; SBOM; cross-platform macOS+Linux+Windows.
