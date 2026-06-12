# STORY-098 Lessons Learned

## LESSON-IMPLEMENTER-SPEC-DEVIATION

**Category:** Process gap
**Source:** P1/P2 — implementer unilaterally reversed BC-3.03.002 EC-007 spec decision (worked out correct; ratified post-hoc by PO adjudication).

When an implementer encounters a spec decision that produces a test failure, the correct path is to route the conflict to the PO BEFORE implementing a reversal. Unilateral spec-to-code reversals violate the Standing Rule (spec wins; code is brought into alignment). Even when the reversal turns out to be correct in substance, the process violation is real: the adjudication must precede the implementation, not ratify it after the fact.

**Rule:** Spec conflicts encountered mid-implementation → pause → escalate to PO → receive adjudication → then implement. Never reverse a spec decision unilaterally.

---

## LESSON-COMMENT-BRITTLENESS

**Category:** Coding convention
**Source:** P15 — pass-14 rewrote comments with hardcoded counts + cross-crate `file:line` references; P15 then had to fix the inaccuracies introduced by P14.

Hardcoded counts (e.g., "31 slide types") and cross-crate `file:line` citations in inline comments drift silently as the codebase evolves. The adversary cannot distinguish a stale count from a correct one without a grep, and doing so adds friction to every pass.

**Convention:** Comments MUST NOT hardcode counts or cross-crate `file:line` references. Cite the module or trait by name (not line number). If a count is semantically important, derive it from code (e.g., `known_fields().len()`) rather than embedding a literal.

---

## LESSON-STALE-PROSE-DOMINANCE

**Category:** Cascade discipline
**Source:** 12 of 24 findings were stale-prose/label class across progressively-outer surfaces. The proactive post-GREEN sweep (STORY-095 lesson) was not run at cascade start.

The majority of adversary findings in a complex story are stale comments, version labels, and index annotations — not behavioral defects. Running a mechanical sweep (grep for stale version strings, outdated comments, index mismatches) at cascade START, before the first adversary pass, would have eliminated most of P6-P12 findings before the adversary saw them.

**Rule:** After exit gate goes GREEN and before dispatching the first adversary pass, run the following sweep:
1. `grep -rn "v1\.[0-9]\+" .factory/specs/behavioral-contracts/` — check for stale version labels in comments
2. `grep -rn "STORY-NNN" .factory/stories/stories/STORY-NNN-*.md` — check for stale self-references
3. `grep -rn "// [0-9]\+ " crates/` — flag hardcoded counts in comments
4. Verify BC-INDEX and STORY-INDEX title sync with story H1.

This is codified as a mandatory pre-cascade step (STORY-095 lesson reinforced here).

---

## LESSON-DE-VERSIONING-POINTER

**Category:** Anti-pattern elimination
**Source:** P12 — live version pointer in story file pointed at a specific version string; adversary (P12) found it stale.

When a spec artifact (story, BC, taxonomy) references "current version" via a literal string (e.g., `"v1.3"`), that string becomes stale the moment the artifact is updated. The fix (applied at P12) is to remove the live-pointer version label entirely and reference the artifact by path — consumers read the artifact directly and see the current version from its frontmatter.

**Rule:** Story files and cascade summaries MUST NOT contain live-pointer version strings for spec artifacts they reference. Reference by path (absolute). Version history belongs in the artifact's own changelog, not in its consumers.

---

## LESSON-REGISTRY-COHERENCE-TEST

**Category:** Architecture pattern
**Source:** Delivered as part of STORY-098 — permanent `registry↔known_fields` coherence test.

When two separate data sources must remain synchronized (e.g., plugin registry and `known_fields()` lookup table), adding a permanent cross-source assertion test eliminates the entire class of structural drift findings. The test is cheap to write, runs on every CI pass, and is self-documenting.

**Pattern:** For any two data sources that must agree structurally, write a `#[test]` that asserts their cardinality and key-set agreement. Name it with `_coherence` suffix so it is greppable. This pattern killed a whole class of findings that had previously required adversary intervention.
