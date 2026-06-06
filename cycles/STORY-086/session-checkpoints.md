---
document_type: session-checkpoints
cycle: STORY-086
producer: state-manager
---

# Session Checkpoints — STORY-086

Archived checkpoints (superseded by newer ones). Latest checkpoint lives in STATE.md.

---

## Checkpoint archived 2026-06-06 (superseded by pass-8 checkpoint)

| Field | Value |
|-------|-------|
| **Date** | 2026-06-06 |
| **Position** | Wave 4: 21/21 merged. Wave 4 gate FAILED. STORY-086 delivery in progress: adversary pass 6 COMPLETE (F-086-P6-MED-001 untrimmed content found + REMEDIATED; ADR-019 v1.3; HEAD a055345e; exit gate CLEAN; 3300 pass, 14 skipped). Adversary LOCAL pass 7 DISPATCHED. Streak 0/3. Stories 88 / 545 pts. BLK-002 OPEN. |
| **develop SHA** | `030dec6c` (61 merged PRs; origin/develop confirmed) |
| **Active worktrees** | `.worktrees/STORY-086` (feature/STORY-086, HEAD a055345e) |
| **Open PRs** | 0 |
| **Workspace crates** | 17 |
| **Workspace tests** | 3300/3300 pass, 14 skipped (e2e AC-007 bullets intentionally ignored pending STORY-088; 1 pre-existing cold_budget flake tracked under STORY-080) |
| **DURABLE ARTIFACTS** | (1) `.factory/cycles/STORY-086/adversarial-reviews/adversary-STORY-086-pass-1.md`; (2) `adversary-STORY-086-pass-2.md`; (3) `adversary-STORY-086-pass-3.md` (HIGH-001 geometry tag-drive; MED-001 stale docs); (4) `adversary-STORY-086-pass-4.md`; (5) `adversary-STORY-086-pass-5.md` (CRIT-001 decorative-first + D5 @var gap + MED-001/002 + OBS-1); (6) `adversary-STORY-086-pass-6.md` (MED-001 untrimmed content; ADR-019 §3.1/§3.3 spec-vs-spec contradiction); (7) `.factory/cycles/STORY-086/architect-pass-1-adjudication.md` D1-D5; (8) `.factory/cycles/STORY-086/architect-pass-6-adjudication.md` (trim canonical); (9) `.factory/specs/wave4-expanded-scope-uncertainty-resolution.md`; (10) `ADR-019` v1.3; (11) `BC-1.16.001` v1.1; (12) `BC-3.04.001` v1.6; (13) error-taxonomy v2.17; (14) STORY-086 v1.4 + STORY-088 v1.2 (8 pts). |
| **RESUME INSTRUCTION** | Adversary LOCAL pass 7 is IN PROGRESS (streak 0/3). Await result; if CLEAN streak advances to 1/3. Target: 3 consecutive strict-CLEAN passes (BC-5.39.001). After convergence: demo-recorder → pr-manager 9-step → security-reviewer + pr-reviewer (independent) → merge (STANDING MERGE AUTH). After STORY-086 merges: re-run Gate 3 (adversary) + Gate 5 (holdout) on patched develop. Only after re-gate passes: advance to Wave 5. BLK-002 OPEN until STORY-086 merges + gates re-pass. |
