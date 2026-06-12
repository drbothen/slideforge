---
story: STORY-096
title: "REND-007 — lessons learned"
date: 2026-06-12
---

# STORY-096 Lessons Learned

## L-a: Red Gate doc-comments must be authored TIMELESS from the start

**Category:** test-writer discipline
**Severity:** Process gap (recurrence risk: HIGH — affected 2 of 9 passes)

**What happened:** Pass 2 findings F-096-P2-001 and F-096-P2-002 were triggered by
doc-comments written during the Red Gate phase that used historical narration:
"previously hardcoded to en-US", "RED: currently drops lang attribute". After the
TDD green pass fixed the behavior, these comments were no longer true — but they
survived a dedicated cleanup commit and were still present when pass 2 began.

The pass-4 findings F-096-A005, F-096-A006, F-096-A007 repeated the same class:
comments written with "was / previously / currently" voice that became stale as soon as
the code changed.

**Root cause:** The test-writer and implementer dispatches did not include an explicit
constraint requiring TIMELESS prose in doc-comments and test assertions. "Currently X"
and "previously Y" are statements about the author's present moment, not about the
code's invariants.

**Mitigation:** Fold this rule into ALL test-writer dispatches:
> "Every doc-comment, assertion message, and inline comment must be authored in
> TIMELESS declarative voice. Phrases like 'currently', 'previously', 'was hardcoded',
> 'RED: this will fail', and 'after this fix' are FORBIDDEN outside of version changelog
> sections. Write what the code DOES, not what it DID or IS BEING CHANGED TO DO."

Include this in the per-story-delivery runbook template alongside SID-1 and LESSON-17.

---

## L-b: Module traceability tables must be reconciled in EVERY fix-burst that adds or renames tests

**Category:** fix-burst discipline
**Severity:** Process gap (recurrence risk: HIGH — caused pass-6 HIGH finding)

**What happened:** F-096-P6-001 was a HIGH finding triggered by a mismatch between
the module-level traceability table in `master_serializer.rs` (which listed 9 functions)
and the test matrix (which had 11 rows after the pass-4/pass-5 fix-burst added new
tests and renamed existing ones). Both the count and at least one function name were
stale.

This is a recurrence of a class seen in STORY-094 and STORY-095. In all three cases,
a fix-burst added or renamed tests without propagating the change to the traceability
table in the same commit.

**Root cause:** Implementer fix-burst discipline does not include a mandatory sweep of
traceability tables in the same crate. The table is authored once (often by the
test-writer) and is not automatically kept in sync.

**Mitigation:** Add to the fix-burst checklist for ALL implementer dispatches:
> "Before committing any fix-burst that adds, removes, or renames a test function:
> (1) grep for the old function name in all comments and tables in the same crate;
> (2) update every traceability table row that references the changed function;
> (3) verify the table row count equals the actual function count via `grep -c fn test_`."

This check must be part of the exit gate self-audit, not deferred to the adversary.

---

## L-c: OOXML-element-placement claims in specs/stories must be validated at authoring time [process-gap]

**Category:** spec-authoring process gap
**Severity:** CRITICAL (schema-invalid element survived Phase 1/2 convergence)
**Tag:** [process-gap]

**What happened:** The original story spec (v1.0) AC-001 stated that `<p:sldSz>` should
be "derived from `brand.page_size`" in the slide master — implying it was a VALID child
of `CT_SlideMaster`. It is not. `<p:sldSz>` is a valid child of `CT_Presentation` only
(ECMA-376 §19.3.1.42). Emitting it in `slideMaster1.xml` produces a schema-invalid
document that triggers a PowerPoint repair dialog (violating BC-4.01.001 PC-2 and PC-4).

The schema-invalid element survived:
- Phase 1 spec crystallization (3/3 clean adversary passes)
- Phase 2 story decomposition (3/3 clean adversary passes)
- story-writer authoring
- The TDD Red Gate phase (test was written to assert presence, not absence)
- Passes 1–3 of the LOCAL cascade (finding not detected until pass 4)

The adversary caught it at pass 4 via a cross-reference between the story AC text and
the ECMA-376 typed schema — not via code inspection.

**Root cause:** There is no authoring-time gate that validates OOXML element placement
claims in spec/story text against the ECMA-376 schema or the `ooxmlsdk` typed schema.
The story-writer agent does not have a schema lookup step for OOXML element names cited
in acceptance criteria.

**Deferral note:** The fix requires changes to the story-writer and product-owner
dispatch templates to include an OOXML-element-validation step at story-authoring time.
This is a process change to the factory pipeline, not a code change. It is being tracked
as a follow-up to the NEXT spec-authoring cycle (story-writer/PO dispatch templates).
No current story vehicle exists for this process fix.

**STATE.md follow-up:** Logged as open drift item pending next spec-authoring cycle.
Until the process fix is in place, the mitigation is: spec-reviewer and adversary passes
at Phase 1/2 MUST include an explicit OOXML element placement check when any AC text
names a specific XML element.
