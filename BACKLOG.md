---
purpose: Durable task list — zero-context session resume source
updated: 2026-06-08
---

# Slideforge Factory — Durable Backlog

**PURPOSE:** This file is the durable source-of-truth task list. On session start,
the orchestrator rebuilds in-session tasks (one TaskCreate per OPEN item) from this
file. State-manager must mirror updates to this file alongside STATE.md at every
milestone (merge, gate pass, follow-up resolution).

## Zero-Context Resume Steps

1. Run `vsdd-factory:factory-worktree-health` — verify .factory/ worktree is clean on factory-artifacts.
2. Verify `git rev-parse develop` == `git rev-parse origin/develop`. Canonical SHA in STATE.md frontmatter (`develop_sha`). Current: `cbebfd57` (71 merged PRs).
3. Read `.factory/STATE.md` — current phase, position, standing rules, open blockers.
4. Read this BACKLOG.md — create one in-session Task per OPEN item in the Active Work table below.
5. Confirm workspace tests green: `cargo nextest run --workspace --no-fail-fast` (expect ~3690+ pass, 0 fail, 18 skip; slideforge-cli (build command + unified compile/export_format API) added in PR #71; known flaky: cold_budget tracked STORY-080).
6. Await human go-ahead before picking the next story — do NOT auto-start delivery.

---

## Active Work

| id | item | status | priority | source / anchor |
|----|------|--------|----------|----------------|
| WAVE5-UNCERTAINTY-PASS | Wave-5 remove-uncertainty pass COMPLETE (Stages 1-2 done; NFR-002 sweep done). Stage 1 (2026-06-07): ADR-021/022 authored, NFR-002 deferred (nfr-catalog v1.3, prd v1.1, BC-3.06.002 v1.3), export-arch v1.2, dep-graph v1.1. Stage 2 (2026-06-07): 17 stories realigned to registry-verified pins; NFR-002 <50ms incremental gate deferred to v1.x in system-overview.md, cicd-setup.md, ux-spec (FLOW-003/SCR-002/SCR-008/UX-INDEX), epics.md, wave-schedule.md. STORY-082 spec reconciled separately (prior commit 7df72268). Wave-5 delivery may now begin. | complete | P0 | vsdd-factory:remove-uncertainty output; WAVE5-UNCERTAINTY-S2 decisions log |
| WAVE5-DEP-PREP | Wave-5 dependency prep PR #69 (3e3a978f). [workspace.dependencies] centralized + ADR-022 major-version migrations (toml 1.1.2, sha2 0.11.0, criterion 0.8.2, notify 8.2.0, indexmap 2.14) + INERT Wave-5 catalog entries. ADR-022 migration tasks DONE. | complete | P0 | PR #69, develop 3e3a978f |
| WAVE5-DELIVERY | **ACTIVE DIRECTIVE (human 2026-06-08): launch full-width fan-out of the APPROVED BATCH now — STORY-082, 088, 072, 074, 079, 080, 047, 081 (8 parallel). HELD: STORY-057/058/064 (slideforge-cli same-crate conflict, deliver after batch), STORY-056 (←047 not yet merged), STORY-060/061 (FU-SEC-001-GIT2-OPENSSL). Apply LESSON-21 exit-gate to every story.** Deliver remaining Wave-5 stories. Full-width fan-out (up to 8 parallel) AUTHORIZED 2026-06-07; batch selected 2026-06-08. **3 of 22 done:** STORY-089 MERGED PR #68 (c722c28b); STORY-046 MERGED PR #70 (fa85d1136); STORY-055 MERGED PR #71 (cbebfd57). Dep-prep MERGED PR #69 (3e3a978f). **19 stories / 98 pts remain.** **Both unlock keys (046+055) merged → L1 UNLOCKED.** **Dependency levels (topological):** **APPROVED BATCH (L0 + L1-now-unlocked — launch in parallel):** STORY-082 (EPIC-08/E08, P0, 5pt), STORY-088 (EPIC-02/E02, P1, 8pt), STORY-072 (EPIC-07/E07, P2, 3pt), STORY-074 (EPIC-07/E07, P2, 3pt — NOTE: same EPIC-07 as 072; verify crate before full parallel), STORY-079 (EPIC-12/E12, P2, 3pt), STORY-080 (EPIC-19/E19, P2, 3pt — also fixes cold_budget timing flake), STORY-047 (P1, 8pt)←046, STORY-081 (P0, 13pt)←046. **HELD AFTER THIS BATCH (same-crate slideforge-cli conflict):** STORY-057 (P0,5pt)←055; STORY-058 (P0,5pt)←055; STORY-064 (P1,5pt)←055. **HELD (←047 unmerged):** STORY-056 (P0,8pt)←047+055. **HELD (FU-SEC-001-GIT2-OPENSSL must resolve first):** STORY-060 (P1,8pt)←055; STORY-061 (P1,5pt)←060. **L2 remaining (6 stories):** STORY-048 (P1,8pt)←047; STORY-056 (P0,8pt)←047+055; STORY-061 (P1,5pt)←060; STORY-062 (P1,3pt)←060; STORY-063 (P1,3pt)←060; STORY-065 (P1,5pt)←064. **L3 (1 story):** STORY-059 (P0,5pt)←056. **Critical path (3 deep from current):** 047→056→059. **CAUTION:** resolve FU-SEC-001-GIT2-OPENSSL BEFORE launching STORY-060/061. | in_progress — 3 of 22 done (STORY-089 PR #68, STORY-046 PR #70, STORY-055 PR #71); dep-prep done | P0 | STORY-INDEX.md, sprint-state.yaml, dependency-graph.md, wave-schedule.md |

---

## Open Follow-ups (deferred, anchored)

| id | item | status | severity | anchor |
|----|------|--------|----------|--------|
| FU-SEC-001-GIT2-OPENSSL | Inert `git2 = {features=["https"]}` catalog entry in [workspace.dependencies] will pull `openssl-sys` (banned by deny.toml) when first consumed. Inert now (git2 absent from Cargo.lock). MUST resolve before STORY-060/061: either drop `https` feature and route package HTTPS via reqwest+rustls (already in catalog), OR add `vendored-openssl` + a scoped deny.toml exception. Architecture decision for package-manager story. | pending | MEDIUM | Anchored to STORY-060; discovered during PR #69 review (pr-reviewer NIT escalated). |
| FU-SEC-001-HARDENING | ImagePathValidator residual string-layer bypass vectors (percent-encoding, whitespace, unicode dots) + OS path canonicalization as primary defense. | pending | low (defense-in-depth) | Anchored to future image-loading story; SEC-001-HARDENING label in STATE.md. |
| FU-SEC-001-DIAG-HARDENING | Unbounded user-authored strings embedded verbatim in diagnostic messages (E-VAL-104 T2 + W-VAL-103 pattern); truncate to ~512 chars to bound memory amplification. | pending | low | Anchored to future validator/diagnostic-hardening story. |
| FU-046-SEC-001-DATA-SCHEME | STORY-046 LOW: SVG href attributes may emit `data:` scheme URIs — scope + tighten allowlist in SVG generation path. | pending | low | Anchored to STORY-047/048; identified by security-reviewer on PR #70. |
| FU-046-SEC-002-NPM-PIN | STORY-046 LOW: axe-core / npm audit tooling not version-pinned in CI gate script — pin to specific version for reproducible security auditing. | pending | low | Anchored to STORY-047/048; identified by security-reviewer on PR #70. |
| FU-046-USVG-UNUSED | pr-reviewer suggestion on PR #70: `usvg` declared in Cargo.toml but unused in slideforge-html final implementation — consider removing or confirm reserved for future SVG optimization pass. | pending | low (OBS) | Anchored to STORY-047 or trivial doc/cleanup sweep. |
| FU-046-LIBRS-DOC | pr-reviewer suggestion on PR #70: stale lib.rs module-level doc-comment in slideforge-html (references pre-P4 architecture) — update to reflect P4 Composite Rendering Model. | pending | low (OBS) | Trivial doc sweep; can bundle with STORY-047 or standalone doc PR. |
| FU-STORY003-CLEANUP | Stale todo!() doc-comments in STORY-003-era #[cfg(test)] modules (e.g. registry.rs ~449-1001) assert validate_fields/suggest are todo!() — now false since STORY-089; sweep STORY-003-era test modules. | pending | low (OBS, out of STORY-089 scope) | Maintenance sweep or quick docs PR; not a story blocker. |
| FU-SEC-002-DEMO | Demo example binary uses panic on fs ops — ACCEPTED (demo-only, no production impact); no action unless promoted to tutorial. | wontfix / accepted | n/a | security-reviewer PR #68 assessment. |
| FU-055-SEC-001-TMPFILE | STORY-055 LOW: predictable tmp-file name / symlink race in shared output directory (CWE-377/CWE-59). When build output is written to a shared dir, a predictable temp filename allows a symlink attack. | pending | low | Anchor Phase 6 hardening; identified by security-reviewer on PR #71. |
| FU-055-SEC-002-OTEL-ENDPOINT | STORY-055 LOW (otel-feature-only): `--otel-endpoint` CLI flag value passed unvalidated as a collector URL (CWE-918 SSRF-adjacent). No network request without `otel` feature; low severity. | pending | low | Anchor Phase 6 / future otel hardening; identified by security-reviewer on PR #71. |
| FU-055-OFFLINE-FLAG | STORY-055: `--offline` global flag declared in CLI arg struct but not threaded into the pipeline — no behavioral effect until data-source fetching exists. Not a STORY-055 defect (data-sources are future work). | pending | low | Anchored to STORY-021 (data-source integration); discovered STORY-055 cascade. |
| FU-055-VARIANT-NOTE | STORY-055: variant selection works end-to-end via `eval_deck_with_variant` (C-1 was a fabricated deferral caught by adversary). No action required. Traceability note only. | closed (traceability) | n/a | STORY-055 cascade; C-1 resolved. |
| FU-055-JSON-HASFATAL-DOC | STORY-055: story spec L442-443 illustrative `--json` snippet shows `has_fatal:true` at exit 2, which differs from code's `has_fatal = exit1 \|\| exit3` definition. Non-binding illustrative prose — does NOT change behavior. Optional trivial story-doc fix. | pending | low (prose-only) | STORY-055 story spec; non-blocking; optional doc PR. |

---

## Phase Backlog

| id | item | status | notes |
|----|------|--------|-------|
| PHASE-4-7 | Holdout eval (Phase 4) → Adversarial refinement (Phase 5) → Formal hardening (Phase 6: Kani + cargo-fuzz + cargo-mutants) → 7-dim convergence (Phase 7). | pending | Required before v1.0. Wave 6 = EPIC-20 Phase-6 stories (STORY-066–071). Gated on Wave 5 gate PASS. |

---

## Recently Completed (audit trail)

- **STORY-055 PR #71** (cbebfd57, 2026-06-08) — `slideforge build` CLI command + miette diagnostics. Unified compile pipeline: compile_core single source; build_inner = compile_core + export_format; new public API compile()/export_format()/CompileOptions/CompiledDeck; BuildError::MultistageFailed for cross-stage eval+validator error accumulation. 7-pass LOCAL 3-CLEAN + 4 post-convergence CI-fix cycles (B1 cross-crate intra-doc link; B2 otel TracingGuard Tokio runtime gated on `otel` feature (ADR-021); B3 snapshots-job shared-process tracing test isolation — init_tracing already-set→Ok(noop); B4 Windows export-error test injection — portable blocker-file technique). Security CLEAN (FU-055-SEC-001-TMPFILE + FU-055-SEC-002-OTEL-ENDPOINT → Phase 6). pr-reviewer APPROVE. adversary 3-CLEAN (passes 5-6-7 of 7). CI all-green incl windows+macos+snapshots+docs+bench. STORY-057/058/060/064 UNLOCKED (←055). STORY-056 UNLOCKED (←047+055). LESSON-21 + LESSON-22 codified.
- **STORY-046 PR #70** (fa85d1136, 2026-06-08) — Static HTML exporter (slideforge-html crate). P4 Composite Rendering Model. ADR-008 P4 amendment; BC-4.03.003 v1.4 (4-step heading chain + synthetic visually-hidden h1); HtmlExporter registered in root registry (human-authorized). Fixed CI axe-core module-resolution bug. 23-pass LOCAL cascade; 3/3 strict-CLEAN (passes 21-22-23). slideforge-html crate ~132 new tests. Security CLEAN (2 LOW follow-ups FU-046-SEC-001-DATA-SCHEME + FU-046-SEC-002-NPM-PIN anchored STORY-047/048). pr-reviewer APPROVE. CI all-green. STORY-047 + STORY-081 UNLOCKED.
- **Wave-5 dep-prep PR #69** (3e3a978f, 2026-06-08) — [workspace.dependencies] centralized + ADR-022 migrations done (toml 1.1.2, sha2 0.11.0, criterion 0.8.2, notify 8.2.0, indexmap 2.14) + INERT Wave-5 catalog entries. Security CLEAN; CI green. ADR-022 migration tasks DONE. FU-SEC-001-GIT2-OPENSSL identified (MEDIUM, anchored STORY-060).
- **Wave-5 uncertainty pass COMPLETE** (2026-06-08) — all 21 stories spec-accurate. 3 factory-artifacts commits: 7df72268+277fc481+b353613a. ADR-021/022 authored; NFR-002 deferred; dep-graph v1.1; 17 stories realigned. Full-width fan-out authorized.
- **STORY-082 spec reconciliation** (2026-06-07) — BC-4.01.003 v1.3 (EC-010/EC-011, PC 7/8, Inv 5); error-taxonomy v2.24 (E-PAR-023/W-PAR-002); export-architecture v1.1 (p14 ext sectionLst, raw quick-xml injection, sha2+quick-xml prod deps); STORY-082 body realigned (AC-010/AC-011). STORY-082 now ready for delivery.
- **Wave-4 follow-up PR #65** (0d0113a2) — SEC-002 CLOSED: split a11y arm, AltText::Unspecified → tracing::warn!.
- **Wave-4 follow-up PR #66** (23f09c62) — SEC-001 (E-VAL-012, error-taxonomy v2.19, BC-1.16.001 EC-012) + diag-span CLOSED. +15 tests.
- **Wave-4 follow-up PR #67** (df207b84) — OBS-P6-001 (status slide RegionRole::Title frame) + OBS-P6-002 (canonical ColorBar percent:u8 in FrameContent) CLOSED. +5 tests.
- **STORY-089 PR #68** (c722c28b) — FieldType enum + type_matches() + E-VAL-104 (T1/T2) + FieldSchemaValidator wired into Stage-5. 8 Priority-1 annotations. Latent validate_fields dead-letter closed (ADR-020 Decision 8). error-taxonomy v2.23. BC-1.18.001 v1.5. 3-CLEAN (~11-pass cascade). Wave 5 slot 1 COMPLETE.
