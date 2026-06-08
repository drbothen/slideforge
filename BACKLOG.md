---
purpose: Durable task list — zero-context session resume source
updated: 2026-06-07
---

# Slideforge Factory — Durable Backlog

**PURPOSE:** This file is the durable source-of-truth task list. On session start,
the orchestrator rebuilds in-session tasks (one TaskCreate per OPEN item) from this
file. State-manager must mirror updates to this file alongside STATE.md at every
milestone (merge, gate pass, follow-up resolution).

## Zero-Context Resume Steps

1. Run `vsdd-factory:factory-worktree-health` — verify .factory/ worktree is clean on factory-artifacts.
2. Verify `git rev-parse develop` == `git rev-parse origin/develop`. Canonical SHA in STATE.md frontmatter (`develop_sha`).
3. Read `.factory/STATE.md` — current phase, position, standing rules, open blockers.
4. Read this BACKLOG.md — create one in-session Task per OPEN item in the Active Work table below.
5. Confirm workspace tests green: `cargo nextest run --workspace --no-fail-fast` (expect 3393+ pass, 0 fail, 18 skip).
6. Await human go-ahead before picking the next story — do NOT auto-start delivery.

---

## Active Work

| id | item | status | priority | source / anchor |
|----|------|--------|----------|----------------|
| WAVE5-UNCERTAINTY-PASS | Wave-5 remove-uncertainty pass COMPLETE (Stages 1-2 done; NFR-002 sweep done). Stage 1 (2026-06-07): ADR-021/022 authored, NFR-002 deferred (nfr-catalog v1.3, prd v1.1, BC-3.06.002 v1.3), export-arch v1.2, dep-graph v1.1. Stage 2 (2026-06-07): 17 stories realigned to registry-verified pins; NFR-002 <50ms incremental gate deferred to v1.x in system-overview.md, cicd-setup.md, ux-spec (FLOW-003/SCR-002/SCR-008/UX-INDEX), epics.md, wave-schedule.md. STORY-082 spec reconciled separately (prior commit 7df72268). Wave-5 delivery may now begin. | complete | P0 | vsdd-factory:remove-uncertainty output; WAVE5-UNCERTAINTY-S2 decisions log |
| WAVE5-DELIVERY | Deliver 21 remaining Wave-5 stories (per sprint-state.yaml + dependency-graph.md). Order: STORY-082 (EPIC-08, P0), STORY-081 (EPIC-18, P0), STORY-088 (EPIC-02, P1) first (P0/P1 independents); then chains: EPIC-14: 046→047→048; EPIC-15: 055→056→057/058/059 (056 also needs 047); EPIC-16: 060→061+062+063; EPIC-17: 064→065; P2 independents: STORY-072, STORY-074, STORY-079, STORY-080. STORY-089 is the 1 merged story (PR #68, c722c28b). | in_progress — 1 of 22 done (STORY-089) | P0 | STORY-INDEX.md, sprint-state.yaml, dependency-graph.md, wave-schedule.md |

---

## Open Follow-ups (deferred, anchored)

| id | item | status | severity | anchor |
|----|------|--------|----------|--------|
| FU-SEC-001-HARDENING | ImagePathValidator residual string-layer bypass vectors (percent-encoding, whitespace, unicode dots) + OS path canonicalization as primary defense. | pending | low (defense-in-depth) | Anchored to future image-loading story; SEC-001-HARDENING label in STATE.md. |
| FU-SEC-001-DIAG-HARDENING | Unbounded user-authored strings embedded verbatim in diagnostic messages (E-VAL-104 T2 + W-VAL-103 pattern); truncate to ~512 chars to bound memory amplification. | pending | low | Anchored to future validator/diagnostic-hardening story. |
| FU-STORY003-CLEANUP | Stale todo!() doc-comments in STORY-003-era #[cfg(test)] modules (e.g. registry.rs ~449-1001) assert validate_fields/suggest are todo!() — now false since STORY-089; sweep STORY-003-era test modules. | pending | low (OBS, out of STORY-089 scope) | Maintenance sweep or quick docs PR; not a story blocker. |
| FU-SEC-002-DEMO | Demo example binary uses panic on fs ops — ACCEPTED (demo-only, no production impact); no action unless promoted to tutorial. | wontfix / accepted | n/a | security-reviewer PR #68 assessment. |

---

## Phase Backlog

| id | item | status | notes |
|----|------|--------|-------|
| PHASE-4-7 | Holdout eval (Phase 4) → Adversarial refinement (Phase 5) → Formal hardening (Phase 6: Kani + cargo-fuzz + cargo-mutants) → 7-dim convergence (Phase 7). | pending | Required before v1.0. Wave 6 = EPIC-20 Phase-6 stories (STORY-066–071). Gated on Wave 5 gate PASS. |

---

## Recently Completed (audit trail)

- **STORY-082 spec reconciliation** (2026-06-07) — BC-4.01.003 v1.3 (EC-010/EC-011, PC 7/8, Inv 5); error-taxonomy v2.24 (E-PAR-023/W-PAR-002); export-architecture v1.1 (p14 ext sectionLst, raw quick-xml injection, sha2+quick-xml prod deps); STORY-082 body realigned (AC-010/AC-011). STORY-082 now ready for delivery.
- **Wave-4 follow-up PR #65** (0d0113a2) — SEC-002 CLOSED: split a11y arm, AltText::Unspecified → tracing::warn!.
- **Wave-4 follow-up PR #66** (23f09c62) — SEC-001 (E-VAL-012, error-taxonomy v2.19, BC-1.16.001 EC-012) + diag-span CLOSED. +15 tests.
- **Wave-4 follow-up PR #67** (df207b84) — OBS-P6-001 (status slide RegionRole::Title frame) + OBS-P6-002 (canonical ColorBar percent:u8 in FrameContent) CLOSED. +5 tests.
- **STORY-089 PR #68** (c722c28b) — FieldType enum + type_matches() + E-VAL-104 (T1/T2) + FieldSchemaValidator wired into Stage-5. 8 Priority-1 annotations. Latent validate_fields dead-letter closed (ADR-020 Decision 8). error-taxonomy v2.23. BC-1.18.001 v1.5. 3-CLEAN (~11-pass cascade). Wave 5 slot 1 COMPLETE.
