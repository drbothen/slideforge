---
story: STORY-099
title: "REND-006 — lessons learned"
date: 2026-06-12
---

# STORY-099 Lessons Learned

## L-a: Test-writer timeless-prose discipline — recurrence of STORY-096 L-a [process-gap]

**Category:** test-writer discipline
**Severity:** Process gap (recurrence risk: HIGH — STORY-096 L-a pattern repeated)
**Tag:** [process-gap]
**Follow-up vehicle:** FU-099-PROCESS-GAP-TESTWRITER-PROSE (see STATE.md Open Follow-Ups)

**What happened:** Despite the explicit timeless-prose instruction added after STORY-096,
the test-writer dispatch for STORY-099 again produced doc-comments and assertion messages
written in historical present-tense voice: "currently emits empty `<w:p/>`", "RED: this
will pass after implementation", "previously the section serializer dropped lang". These
comments were stale the moment the TDD green pass fixed the behavior.

The recurrence was caught in the stale-prose sweep (Step 2.3 of the post-Red-Gate
runbook) before the LOCAL cascade began — the sweep step is now validated as an
effective early catch. No cascade pass was consumed cleaning up stale prose.

**Root cause:** The timeless-prose constraint is present in the dispatch preamble text
but is not structurally enforced in the test-writer agent prompt itself. Agents operating
under context pressure tend to revert to descriptive historical voice for doc-comments
when the spec text uses past-tense framing ("was empty", "previously missing").

**Mitigation required:** The test-writer agent prompt (in the plugin definition, not only
the dispatch preamble) must encode the timeless-prose rule as a first-class constraint
alongside SID-1 (no-ignored-test rationalization) and LESSON-17 (no `#[should_panic]`
placeholders). Until the prompt is hardened, the Step 2.3 stale-prose sweep must remain
in the per-story-delivery runbook.

**Deferral note:** Hardening the test-writer agent prompt itself is a factory-pipeline
process change (not a codebase change). Deferred to the next agent-prompt-update cycle.
No current story vehicle. Follow-up logged as FU-099-PROCESS-GAP-TESTWRITER-PROSE, which
is a sibling to FU-096-PROCESS-GAP-OOXML-PLACEMENT in the same process-gap category.

---

## L-b: Security parity sweeps must span sibling exporters in the same cycle [structural fix now applied]

**Category:** security review discipline
**Severity:** HIGH (same CWE-116 gap appeared one story later in a sibling exporter)

**What happened:** SEC-096-001 (STORY-096) was a CWE-116 vulnerability in the PPTX
exporter: language tags written into XML attributes without validation, allowing
XML-special characters to produce malformed XML. The fix for STORY-096 added
`validate_lang_for_xml` to the PPTX code path.

SEC-099-001 (STORY-099) was the identical vulnerability in the DOCX exporter: the same
language tag written into `<w:lang w:val="..."/>` without validation. Found one story
later by the security reviewer.

**Root cause:** The SEC-096-001 fix was applied to the PPTX exporter only. The security
review for STORY-096 noted the fix but did not include a sweep of sibling exporters
(DOCX, HTML, PDF) for the same pattern. There is no checklist step in the security review
dispatch that requires "sweep sibling exporters when a security guard is added to one."

**Structural fix (now applied):** The fix for STORY-099 moved both `DEFAULT_DECK_LANG`
and `validate_xml_lang` (the canonical validator) into `slideforge-types`. PPTX and DOCX
now both delegate to the shared implementation. HTML and PDF exporters do not use
`<w:lang>` attributes, so the specific XML-attribute vector is resolved. Future exporters
that write language tags into XML attributes must import and call `validate_xml_lang`
from `slideforge-types`.

**Mitigation for future stories:** When a security guard is added to any exporter, the
security review dispatch must include an explicit step: "Sweep all sibling exporters for
the same pattern. If the guard is applicable to siblings, apply it in the same cycle."
This is now a standing rule for security review dispatches on this project.

---

## L-c: Spec mis-anchors (AC → wrong BC) are a 2-story pattern — reinforces FU-096-PROCESS-GAP-OOXML-PLACEMENT scope

**Category:** spec-authoring process gap
**Severity:** MED (caught mid-cascade; no production impact)
**Tag:** [process-gap]

**What happened:** F-099-002 (pass 2, MED): AC-004 in the STORY-099 spec was anchored to
BC-3.05.001 (inline-format universality — the inline markup BC from STORY-081). The
correct anchor was BC-5.01.005 PC-4 (language propagation to DOCX run-level `<w:lang>`).
BC-3.05.001 describes an orthogonal contract (bold/italic/code inline marks); it does not
govern `<w:lang>` attribute emission.

The misanchor caused:
- 4 test names to reference the wrong BC ID.
- The story traceability table to cite BC-3.05.001 as the AC-004 source.
- The dependency-graph to record an incorrect depends_on entry (STORY-081 instead of
  STORY-073 + STORY-085).

All were caught at pass 2 and fixed by story-writer dispatch (commits 83f97b94 + 7bf5d623
+ factory-artifacts sweep).

**Pattern:** This is the second consecutive story with a spec mis-anchor caught
mid-cascade:
- STORY-096 F-096-A001: spec defect — wrong element in wrong XML container.
- STORY-099 F-099-002: wrong BC ID in AC traceability.

Both were caught by the adversary, not by a gate at story-authoring time.

**Root cause:** BC-anchor verification is not part of the story-writer exit gate. The
story-writer produces traceability rows by matching AC text to BC postcondition language,
but there is no structured cross-check step that verifies each AC → BC link resolves to
a real, matching postcondition.

**Reinforcement of FU-096-PROCESS-GAP-OOXML-PLACEMENT scope:** The OOXML-placement
follow-up (logged after STORY-096) already targeted story-writer and product-owner
dispatch templates. This lesson extends that scope: the template update must include a
BC-anchor verification step, not only OOXML element placement validation. The follow-up
scope is now: "at story-authoring time, verify (a) any OOXML element name cited in an AC
against the typed schema; (b) every AC → BC link resolves to a real, matching
postcondition in the cited BC."

**Deferral:** Same vehicle as FU-096-PROCESS-GAP-OOXML-PLACEMENT — pending next
spec-authoring cycle. No current story vehicle.
