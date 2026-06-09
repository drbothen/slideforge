# STORY-082 Lessons Learned

_Captured 2026-06-09 after PR #77 merge (develop c60cca36). PPTX slide-grouping sections — 18-pass LOCAL cascade, 3/3 strict-CLEAN (passes 16-17-18)._

---

## OBS-P14-2 [process-gap]: Vacuous-test sweep must cross crate boundaries

**Category:** Process gap (caught at Pass 14; sibling survived to Pass 14)
**ID:** OBS-P14-2
**Date:** 2026-06-09
**Severity:** MEDIUM — caused streak reset at Pass 14; required additional fix burst

### Root Cause

STORY-082 acceptance criterion AC-010 (E-PAR-023 fatal parse error on empty
section name) had a vacuous test in the eval layer: the test body used an
unguarded `Err(_) => return` / `if let Ok(val) = ...` pattern that allowed
the assertion to be silently skipped on error paths. The implementer fixed
this vacuous pattern during the Pass-4 → Pass-14 convergence cascade.

However, an identical vacuous-assertion pattern existed in the SYNTAX layer
for the same AC (a test that would pass trivially even if the production
path returned the wrong error or no error at all). The syntax-layer sibling
survived undetected through Passes 5-13 and was only caught at Pass 14.

### Why the sibling survived

The fix at Pass 4 (eval-layer vacuous test) was scoped to the crate where
the finding was raised. No cross-crate sweep was performed at the time of
the fix. The adversary at subsequent passes focused on the corrected crate
and did not re-examine sibling crates for the same pattern until Pass 14.

This mirrors the LESSON-19 / SIBLING-SWEEP pattern, applied to test-rigor
defects rather than canonical-value propagation defects.

### Rule (OBS-P14-2)

For any story whose ACs include **fatal-parse-error behavior** (or any AC
that requires testing an error path), a vacuous-assertion grep MUST be run
across ALL crates in the story's scope — not just the crate where the
primary fixing occurred.

**Grep patterns to use (per-story convergence checklist):**

```bash
# Unguarded Err catch-and-return (test silently passes on any Err)
grep -rn 'Err(_)\s*=>\s*\(return\|{\s*}\)' crates/

# Unguarded if-let without assertion (test silently skips on parse failure)
grep -rn 'if let Ok\|if let Some' crates/ | grep -v 'assert\|expect\|unwrap\|panic'
```

**Rule:** Add this grep to the per-story convergence checklist for every
story whose ACs include **fatal-parse-error**, **warning-emission**, or
**error-recovery** behavior. The sweep must cover the full crate scope
of the story (all crates touched by the story's implementation), not just
the primary implementation crate.

### Disposition

This process-gap lesson is recorded for application to the per-story
adversary dispatch checklist. No follow-up story is required: the sweep
is a zero-cost addition to existing dispatch protocol. The orchestrator
should incorporate this grep into the STORY-081, STORY-088, and all
future stories with error-path ACs.

_Anchor: adversary dispatch protocol; applicable to any story with error-path ACs._

---

## Follow-up Registered

**FU-082-SEC-S001-NUL-CALLSITE-TEST** (SUGGESTION — non-blocking depth-of-defense):
Add a dedicated NUL (U+0000) call-site test through `SectionListBuilder::build_ext_lst`.
Currently NUL is covered only by the `strip_control_chars` helper unit test;
U+0001 and U+000B are covered at the section-attr write path. The NUL callsite
test was identified by the pr-reviewer as a non-blocking observation; security
review was CLEAN. This is depth-of-defense polish, not a correctness gap.

_Disposition: register as a non-story follow-up. Deliver if a Wave-5 or Wave-6
story opens the slideforge-pptx section-builder module; otherwise close at
Phase 6 formal hardening if the helper is Kani-verified._

---

_Full cascade detail: 18 passes, findings: CRIT→HIGH→HIGH→HIGH→MED→MED→CLEAN×3.
Spec version: 1.0 → 1.1 → 1.2 → 1.3 (sha2 pin, E-PAR-023 exit-code, SlideSectionEntry anchor)._
