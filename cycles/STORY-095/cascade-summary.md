# STORY-095 — LOCAL Adversary Cascade Summary

**Story:** REND-002 + REND-008-pdf fix — PDF line-wrap + /Figure + bold subset (EPIC-13, BC-4.03.001 + BC-4.03.002, VP-054 (NEW v1.2.1), 8 pts)
**PR:** #86 squash-merged → develop `6c27fcf3` (86 merged PRs) 2026-06-11
**Cascade:** 16 passes, 25 findings closed, CONVERGED 3/3 strict-CLEAN (passes 14-15-16)
**Strict-CLEAN enforcement precedent:** Orchestrator enforced strict criterion at pass 11 — OBS-severity entries count as findings under BC-5.39.001 (streak reset to 0/3).

---

## Pass Log

### Pass 1 — 9 findings (3C/3H/2L/1O)
- F-095-P1-001 CRIT: wrap engine not wired to inline paths (PDF draw_text_run) — both draw paths required
- F-095-P1-002 CRIT: alt paper-fix — ColorBar /Figure+/Alt sourced from placeholder text, not ColorLabel-derived label
- F-095-P1-003 CRIT: VP-054 mis-anchored to VP-006 — VP-054 allocated; story spec repointed VP-006→VP-054
- F-095-P1-004 HIGH: char-wrap space injection — frag0 boundary space added when continuation fragment starts a word
- F-095-P1-005 HIGH: no frame-bottom clamp — overflow guard + `tracing::warn` required
- F-095-P1-006 HIGH: unanchored deferral comment (process-gap — no story ID cited); fixed in-scope per lesson 2 proactive sweep
- F-095-P1-007 LOW: veraPDF fixture vacuous — fixture must exercise /Figure tag end-to-end
- F-095-P1-008 LOW: test placement — snapshot test in red_gate.rs moved to dedicated spec test
- F-095-P1-009 OBS: non-ASCII alt untested
Fix-burst committed `a404b9f4`

### Pass 2 — 3 findings
- F-095-P2-001 MED: inline char-wrap gap — `char_split_word` path not covered by wrap wiring
- F-095-P2-002 MED: VP-054 signature mismatch — 3-arg wrap_text signature not reflected in VP
- F-095-P2-003 LOW: docstring stale
Fix-burst committed `def91e66`

### Pass 3 — 0 findings
- CLEAN (strict): yes. CLEAN (PR-merge): yes.
- Streak: 1/3

### Pass 4 — 1 finding (MED) — streak reset
- F-095-P4-001 MED: measure≠draw fragment spacing — `SpanWord.is_continuation` required to unify measure and draw paths
Fix-burst committed `57ddf666`

### Pass 5 — 2 findings (MED) — streak reset
- F-095-P5-001 MED: mixed-line asymmetry — measure path and draw path used different space-injection logic
- F-095-P5-002 MED: untested draw guard — `space_before_word` shared helper factored out; proxy test added
Fix-burst committed `8c7956c4`

### Pass 6 — 0 findings
- CLEAN (strict): yes. CLEAN (PR-merge): yes.
- Streak: 1/3

### Pass 7 — 1 finding (HIGH) — streak reset
- F-095-P7-001 HIGH: veraPDF gate never exercised /Figure end-to-end — fixture extended with /Figure element; `pdf-ua1-verapdf` CI job wired up
Fix-burst committed `959e5d3c`

### Pass 8 — 0 findings
- CLEAN (strict): yes. CLEAN (PR-merge): yes.
- Streak: 1/3

### Pass 9 — 1 finding (MED) — streak reset
- F-095-P9-001 MED: plain-path frag0 space-drop — plain text (non-inline) path dropped boundary space before first word of continuation fragment
Fix-burst committed `db89a374`

### Pass 10 — 1 finding (MED) — streak reset
- F-095-P10-001 MED: only-spaces spec/code drift — wrap_text behavior on only-whitespace input undocumented; test row added to VP-054 v1.2.1
Fix-burst committed `4674ae69`

### Pass 11 — 1 finding (OBS) — streak reset
**Strict enforcement precedent:** Orchestrator ruled OBS counts as finding under BC-5.39.001; streak reset to 0/3.
- F-095-P11-001 OBS: doc allocation claim — `wrap_text` doc comment claimed "allocates one Vec per line" but actual implementation allocates per-fragment; comment corrected
Fix-burst committed `4d7003a7`

### Pass 12 — 1 finding (MED) — streak reset
- F-095-P12-001 MED: stale ColorBar tagging doc→class-kill 5 sites — doc comments at 5 sites claimed ColorBar was tagged as `/Artifact`; corrected to `/Figure` with `/Alt` throughout
Fix-burst committed `73a753ee`

### Pass 13 — 2 findings
- F-095-P13-001 MED: stale present-tense RED-gate prose in `tests/story_095_red_gate.rs` at 13 location ranges — rewording to historical/past-tense framing required (test bodies functionally correct)
- F-095-P13-002 LOW: `exporter.rs:932-934` doc says `== 0` but guard at line 942 is `<= 0` — doc corrected

### Pass 14 — 0 findings
- CLEAN (strict): yes. CLEAN (PR-merge): yes.
- Streak: 1/3

### Pass 15 — 0 findings
- CLEAN (strict): yes. CLEAN (PR-merge): yes.
- Streak: 2/3

### Pass 16 — 0 findings
- CLEAN (strict): yes. CLEAN (PR-merge): yes.
- Streak: 3/3 — **CONVERGED**

---

## Post-Convergence Reviews

- **security-reviewer:** CLEAN (0 crit / 0 important; 2 suggestion-tier: SEC-001 alt-length uncapped, SEC-002 wrap-precondition undocumented — logged as FU-095-ALT-LENGTH-CAP and FU-095-WRAP-PRECONDITION-DOC)
- **pr-reviewer:** APPROVE (5 non-blocking nits; nit 5 description inaccuracy fixed pre-merge)
- **CI:** green — fast tier ~6 min; `pdf-ua1-verapdf` CI job PASSED with new /Figure fixture

---

## Deliverables

- Pure word-wrap engine `text_layout.rs`: word-boundary split + char-fallback + frag0 boundary space; `SpanWord.is_continuation`; `space_before_word` shared helper (measure==draw by construction)
- Wrap wired into BOTH PDF draw paths (inline `draw_text_run` + plain `draw_packed_line`)
- Frame-bottom clamp + `tracing::warn` overflow guard
- ColorBar `/Figure`+`/Alt` end-to-end from ColorLabel-derived label (layout.rs threading; decorative stays `/Artifact`)
- Bold subset preserved across wrap
- veraPDF fixtures extended; `pdf-ua1-verapdf` CI job PASSED on PR
- `FontMetrics` `mock_char_width_pts` + `mock_space_width_pts`
- VP-054 v1.2.1: `wrap_text` termination/lossless/max-width; BC-4.03.002
- Workspace tests: 4180 pass / 20 skip / 0 fail
