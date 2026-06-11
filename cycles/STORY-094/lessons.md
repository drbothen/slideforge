# STORY-094 — Lessons Learned

Captured: 2026-06-11. Source: 20-pass LOCAL adversary cascade + post-convergence reviews.

---

## LESSON-23-CANDIDATE [process-gap][codified-candidate] REVIEWER-PATH-DISCIPLINE

**Context:** Adversary pass 4 was VOIDED. The reviewer's `Read` calls resolved relative paths against the MAIN repo while `Grep` hits landed in the worktree, producing a false "work missing + files mutating" halt report. The pass had to be re-run from scratch.

**Root cause:** No mandatory absolute-path preamble in adversary/reviewer dispatch prompts. Relative-path resolution depends on the agent's `cwd` at launch; worktree agents may have a different `cwd` than expected.

**Mitigation (current, manual):** PATH DISCIPLINE preamble required in every adversary dispatch: "all Read/Grep calls MUST use absolute worktree paths." Orchestrator must independently verify ground-truth before acting on any halt report.

**Codification path:** Add to adversary and reviewer agent prompts:
1. Absolute-path mandate: all file-tool calls use absolute paths derived from the explicit `--cwd` argument.
2. Self-sanity-check: at start of each pass, run `git log -1 --format='%H'` in the worktree and verify it matches the expected HEAD SHA before making any findings claims.

**Impact:** 1 voided pass out of 20 = ~5% cascade overhead. With a 30+ pass cascade this becomes meaningful.

---

## PAPER-CLOSURE-LADDER [process-gap]

**Context:** The E-LAY-008 diagnostic span went through three rounds before an honest closure:
1. Default span (`deck.sf:1:1`) — passed its own unit test (test used hardcoded fixture path)
2. `<byte:N>:0:0` column-0 fallback — passed its own unit test (test didn't assert span values)
3. Real SourceMap-resolved `actual-file.sf:L:C` span — required end-to-end integration test

Each layer passed the test written to verify it. The adversary caught layers 1 and 2 as paper closures.

**Mitigation (FU-DIAGNOSTIC-FIELD-PINNING reinforcement):** Closure tests for diagnostic spans MUST:
- Assert the RENDERED message text, not just error code presence
- Assert the file/line/column span values with pinned expected values from a real fixture
- Verify the end-to-end path from source file → SourceMap → diagnostic output

Do NOT accept "error variant emitted" as a passing criterion for diagnostic correctness.

---

## STREAK-2/3-FINDS [observation]

**Context:** Both cascade streak-resets occurred at 2/3:
- Pass 15 (streak was 2/3): HIGH finding — two_col bullet width page-relative vs region-relative. Real layout correctness bug.
- Post-convergence (after 3/3 achieved): LOW SEC-001 finding — `sanitize_source_name` missing.

Neither was a false positive. Both required genuine implementation fixes.

**Observation:** The final-pass skepticism instructions in adversary dispatches are load-bearing. The common assumption that "if it passed the first two, pass three is ceremonial" is wrong for this codebase.

**Standing rule:** Keep final-pass skepticism instructions at full intensity. Do NOT soften adversary prompts for passes 2/3 or post-convergence reviews.
