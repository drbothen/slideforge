---
document_type: lessons-learned
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-06-06T00:00:00
cycle: STORY-086
inputs: [STATE.md]
traces_to: STATE.md
---

# Lessons Learned — STORY-086

Durable lessons from the STORY-086 sub-cycle (Wave 4 remediation — content threading + TextTag).
Numbered continuously for cycle-close codification.

---

## Process-Level

### Drift Items (Process Gaps)

**PG-TD060-SCOPE** (Discovered: Pass 4, 2026-06-06)
TD-VSDD-060 sibling-sweep scope during STORY-086 did not include narrative/doc/test-name sites
describing an old contract (only match-arm compile sites). A follow-up improvement story or
justified deferral should codify whether the sibling-sweep scope should extend to doc-comments,
test names, and story spec narrative references.
_Codification action: follow-up improvement story or justified deferral anchored at STORY-086 sub-cycle close._

**PG-WORKTREE-TYPES** (Discovered: Pass 7, 2026-06-06)
When an agent corrects a spec against code for an IN-FLIGHT worktree story, it MUST read
type/code definitions from the worktree (`.worktrees/STORY-NNN/`), NOT the main checkout
(develop, which is without the unmerged story's additions). The PO read develop's types for
STORY-086 pass-7 remediation and wrongly STRIPPED TextTag / TextBlock.tag / AltText::Unspecified
from BC-1.16.001 (commit 608ec7b0) — those types exist only in the STORY-086 worktree.
Reverted by recovery commit f2261592 (BC-1.16.001 v1.3 with legitimate PC-7/PC-10 fixes intact).

This is the SECOND process-gap this cycle (first: PG-TD060-SCOPE from pass-4). Both reinforce
LESSON-1/LESSON-16 (worktree-absolute paths).

Codification rule: Orchestrator dispatches to spec-correction agents MUST explicitly pin the
worktree type path for in-flight stories (e.g., `--cwd /Users/jmagady/Dev/slideforge/.worktrees/STORY-086`
or an explicit instruction "read types from `.worktrees/STORY-086/crates/`"). The agent must
not infer the source path from environment defaults.

_Discovered: Adversary Pass 7, 2026-06-06._
_Both process-gaps noted for cycle-close codification._
_Codification action: follow-up improvement story or justified deferral anchored at STORY-086 sub-cycle close._

---

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| PG-WORKTREE-TYPES | Spec-correction agents MUST read types from worktree path, not main checkout, for in-flight stories | Orchestrator dispatch instructions | proposed |
| PG-TD060-SCOPE | TD-VSDD-060 sibling-sweep scope definition (include doc-comments + test names) | Implementer/adversary TD-VSDD-060 protocol | proposed |
