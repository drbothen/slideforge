# STORY-089 Lessons Learned

_Captured 2026-06-07 after PR #68 merge (develop c722c28b)._

---

## LESSON-19: SIBLING-SWEEP — Exhaustive Propagation Before Re-Running Adversary

**Category:** Process gap (recurred ~5x in STORY-089 LOCAL cascade)
**ID:** LESSON-19
**Date:** 2026-06-07
**Severity:** HIGH — directly caused ~11 adversary passes where 3-CLEAN streak reset repeatedly

### Root Cause

When a fix-burst changed a canonical VALUE, TEXT, COUNT, or ANCHOR in one artifact
(error code, message template, site count, ADR/BC/PC citation, stale-state comment),
the fix was applied to the primary file but NOT propagated to all sibling artifacts
before the next adversary pass was dispatched. The adversary then found the same
stale value in a different file, raising a new finding that reset the 3-CLEAN streak
to 0/3.

Artifacts that carried the same canonical value but were NOT swept in the same burst:
- Production code (implementation + doc-comments)
- Consuming spec documents (BC, ADR, error-taxonomy, story notes)
- Test files (especially doc-comment examples and fixture strings)
- Fixtures and snapshot baselines

### Concrete STORY-089 Instances

1. E-VAL-104 error code added to error-taxonomy.md but NOT to BC-1.18.001 or the
   story notes simultaneously — adversary found the gap on the next pass.
2. Message template wording changed in validate_fields impl but NOT in the error-taxonomy
   T1/T2 description prose — adversary flagged misalignment.
3. ADR-020 Decision 8 text updated but the BC tracing comment in the source was stale —
   adversary found stale cite.
4. BC-1.17.002 scoped to range-only (ValueRangeValidator) but the story acceptance
   criteria still implied broader scope — adversary found contradiction.
5. ~3 additional instances of citation or count mismatch between spec documents.

Each instance individually took one adversary pass to find, and the re-dispatch cost
(fresh-context agent spawn) accumulated into a deep cascade despite none of the root
findings being architecturally complex.

### Rule (codified as LESSON-19 / SIBLING-SWEEP)

When a fix-burst changes a canonical VALUE, TEXT, COUNT, or ANCHOR, the burst MUST:

1. Before writing any fix, run an exhaustive grep across ALL artifact categories
   to enumerate every occurrence of the old value.
2. Fix ALL occurrences in ONE burst — production code, every consuming spec
   (BC/ADR/taxonomy/story), test files, doc-comments, and fixtures.
3. Verify with a post-fix grep that zero occurrences of the old value remain
   (in non-archival files).
4. THEN re-run the adversary.

The orchestrator must grep-map every occurrence itself and dispatch ONE coordinated
exhaustive fix, not a series of per-file partial fixes.

### Correct Pattern

```
orchestrator: grep -r "old-error-code" .factory/specs/ crates/ --include="*.md" --include="*.rs"
orchestrator: dispatch implementer with ALL N files in scope simultaneously
implementer: fix all N files in one burst, run exhaustive post-grep
implementer: confirm zero remaining occurrences
orchestrator: dispatch adversary pass N+1
```

### Anti-Pattern (what happened repeatedly)

```
implementer: fix production code only
orchestrator: dispatch adversary pass N+1
adversary: finds same value in BC file → new finding → 3-CLEAN streak resets
implementer: fix BC file only
orchestrator: dispatch adversary pass N+2
adversary: finds same value in error-taxonomy → new finding → streak resets again
... repeat ~5x
```

### Applicability

This rule applies whenever the changed artifact is:
- An error code or error message template
- A count or total (BC count, test count, story count)
- A citation anchor (ADR-NNN, BC-N.NN.NNN, PC-NNN, HS-NNN)
- A stale-state comment that references old behavior
- A type name, function signature, or trait name referenced across crates and specs

Lower-risk changes (pure impl, no cross-artifact citation) do not require exhaustive
grep before adversary dispatch, but the cost of an unnecessary grep is far lower than
the cost of a reset streak.

---

_Full cascade detail: `.factory/cycles/STORY-089/implementation/red-gate-log.md`_
