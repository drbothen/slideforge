---
document_type: lessons-learned
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-06-07T00:00:00
cycle: "wave-4-gate"
inputs: [STATE.md]
input-hash: ""
traces_to: STATE.md
---

# Lessons Learned — Wave 4 Gate

Lessons captured during the Wave 4 integration gate (2026-06-06 / 2026-06-07).
Gate activities: adversarial wave-diff review (Gate 3), holdout evaluation
(Gate 5), STORY-086 + STORY-087 remediation, re-gate on develop 54b8d3b1.

## Agent-Level

1. **Working-tree drift after `gh pr merge` + `git update-ref` (LESSON-WORKTREE-SYNC)**
   — After each squash-merge via `gh pr merge --squash`, the orchestrator ran
   `git update-ref refs/heads/develop origin/develop` to advance the local branch
   pointer. This moves the branch pointer but does NOT update the working tree.
   When the Wave 4 re-gate was dispatched immediately after, the adversary and
   holdout evaluator operated on the stale working tree (STORY-087 files absent
   from disk even though `git rev-parse develop` returned 54b8d3b1). The re-gate
   reviewed a STALE main-checkout working tree and produced findings against a
   code state that no longer existed on disk.
   Fix applied: `git restore --source=HEAD --staged --worktree .` to sync the
   working tree to HEAD. Re-gate re-run on the correct tree.
   **Rule (LESSON-WORKTREE-SYNC):** After every `gh pr merge`, the main checkout
   MUST be synced with a real checkout/restore:
   `git restore --source=HEAD --staged --worktree .` OR `git checkout develop`.
   `git update-ref` alone is NOT sufficient — it moves the pointer but leaves
   the working tree at the old state. Dispatch protocol must include a
   working-tree verification step (e.g., confirm that a file unique to the latest
   merge is present on disk before dispatching reviewers).
   _Discovered: Wave 4 re-gate first attempt, 2026-06-06_

## Process-Level

2. **Spec-vs-code keyword contradiction causes gate failure (NEW-INT-001)**
   — The image slide type has a keyword name mismatch between spec and code:
   BC-1.16.001 PC-10 specifies the DSL image keyword as `src`, and Stage-2b
   threading in field_to_block.rs reads `src`. But image.rs declares the
   required field as `image`, and known_fields.rs also lists `image`. A user
   writing `image: "path.png"` gets no ContentBlock::Image produced; the
   post-layout alt-path fires E-A11-001 even with a valid alt attribute.
   Additionally, validate_fields is not wired into the main pipeline (fails
   silently). Root cause: spec and code diverged during an earlier story;
   neither side was swept for consistency.
   **Rule:** When a story introduces or modifies DSL field names, a
   crate-wide sibling-site sweep (TD-VSDD-060) MUST cover: the BC canonical
   field list, the SlideType required_fields impl, known_fields.rs, AND the
   field_to_block threading arm — all four must agree on the same keyword
   before the story ships.
   _Discovered: Wave 4 Gate 5 holdout re-run, 2026-06-07_

3. **validate_fields pipeline wiring gap**
   — `validate_fields` exists as a function but is not wired into the main
   build pipeline. During the holdout evaluation, the image-slide `image`/`src`
   contradiction failed silently: no diagnostic was emitted, and no
   ContentBlock::Image was produced. The wiring gap means field-name errors
   are invisible to users until they notice missing content in the output.
   **Rule:** Any new validation function MUST be wired end-to-end before the
   story that introduces it is considered DONE. A Red Gate test must exercise
   the production pipeline path (not just the function in isolation).
   _Discovered: Wave 4 Gate 5 holdout re-run, 2026-06-07_

## Infrastructure-Level

4. **No infrastructure-level lessons this gate cycle.**

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| 1 (Working-tree drift) | LESSON-WORKTREE-SYNC: After `gh pr merge`, always run `git restore --source=HEAD --staged --worktree .` before dispatching any reviewer or gate agent. `git update-ref` alone is insufficient. | Orchestrator merge protocol | proposed |
| 2 (Spec-vs-code keyword) | FIELD-NAME-SWEEP-001: DSL field name changes require a 4-site sweep: BC canonical list + SlideType required_fields + known_fields.rs + field_to_block threading arm | Story delivery, implementer exit-gate | proposed |
| 3 (Wiring gap) | WIRING-GATE-001: New validation functions must be wired end-to-end with a Red Gate integration test before the story ships | Test-writer, implementer exit-gate | proposed |
