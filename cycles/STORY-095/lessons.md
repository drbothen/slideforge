# STORY-095 — Lessons Learned

Captured: 2026-06-11. Source: 16-pass LOCAL adversary cascade + post-convergence reviews.

---

## STRICT-CLEAN-OBS-PRECEDENT [process / BC-5.39.001 enforcement]

**Context:** Orchestrator enforced BC-5.39.001 strict criterion at pass 11 when the adversary issued a single OBS-severity finding (doc comment claiming "allocates one Vec per line" while implementation allocated per-fragment). Streak was reset from 1/3 to 0/3.

**Rule confirmed:** OBS-severity entries count as findings under BC-5.39.001 strict criterion. CLEAN (strict) requires ZERO findings of ANY severity — OBS included. Streak advancement requires strict-CLEAN, not PR-merge-CLEAN.

**Note:** PR-merge-CLEAN threshold (zero CRIT/HIGH/MED) is a separate concept used only at the PR-merge gate, not for streak progression.

**Standing:** This ruling is now precedent for all future LOCAL adversary cascades on this project. Orchestrator must enforce at every cascade.

---

## STALE-DOC-CLASS [process / defect pattern]

**Context:** 5 of the last 6 findings in this cascade (passes 10-13) were doc-prose drift:
- P10-001: `wrap_text` only-whitespace behavior undocumented in VP-054
- P11-001: doc comment claimed wrong allocation strategy
- P12-001: doc comments at 5 sites claimed ColorBar was `/Artifact` instead of `/Figure`
- P13-001: stale present-tense RED-gate prose in test file (13 location ranges)
- P13-002: doc says `== 0` but guard is `<= 0`

**Root cause:** After GREEN phase (tests passing, implementation correct), doc comments and test prose were not swept for stale references before cascade start.

**Process-improvement candidate:** Run a proactive stale-doc sweep BEFORE dispatching the first adversary pass, after GREEN phase completes:
```bash
grep -rn "current code\|currently\|stub returns\|always\|== 0\|TODO\|FIXME" \
  crates/<target-crate>/src/ crates/<target-crate>/tests/ \
  | grep -v "^Binary"
```
This class of finding is cheap to prevent and expensive to discover late (each one resets the streak).

**Note:** F-095-P1-006 (unanchored deferral comment, process-gap) was fixed in-scope — the class is covered by this lesson's proactive sweep.

---

## MEASURE-DRAW-PARITY [implementation pattern]

**Context:** Passes 4 and 5 both found measure≠draw divergence in the word-wrap engine. The measure path (which computes line widths) and the draw path (which renders glyphs) used different space-injection logic, causing visual spacing to differ from measured spacing.

**Solution:** Factor shared logic into a single helper (`space_before_word`) called by BOTH paths. "Measure==draw by construction" is the correct architecture — if they share the helper, they cannot diverge.

**Standing pattern for all future wrap/layout engines:** Any engine that has both a measure pass and a draw pass MUST share the boundary/spacing logic. Separate implementations of the same rule are a defect waiting to happen.
