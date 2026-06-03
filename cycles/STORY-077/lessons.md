# STORY-077 Lessons — Cycle Closing

Captured 2026-06-03 at post-merge burst (PR #49, c8913cad).
Cascade: 26 adversary passes (including 26 full-cascade rounds), 3/3 strict-CLEAN at passes 24-25-26.
Final CI: 16/16 green. Security-reviewer: CLEAN (0 crit/0 important). PR-reviewer: APPROVE.

---

## Security Findings

### SEC-002: Link URL Has No Scheme Validation [LOW — DEFERRED with concrete dependency]

`[text](url)` link URL is stored verbatim in `InlineNode::Link.url` with no allowlist filter.
Unvalidated `javascript:` and `data:` schemes can be stored in the IR.

**No exploit surface today:** no HTML exporter ships in v1.0 scope yet. The IR is
not rendered to any browser-facing output until STORY-046 (Static HTML Exporter).

**Required action before STORY-046 ships:**
Enforce a scheme allowlist at the parse boundary in `slideforge-syntax`. Allowlist:
`http`, `https`, `mailto`. Reject all other schemes with a new structured error code
(E-PAR-022 or next available). This converts an unvalidated-input sink into a hard
compile error before any user content reaches an HTML render context.

**Concrete dependency:** STORY-046 (HTML exporter). This deferral is justified because
the fix requires a new E-PAR code and parser boundary change — which should be allocated
alongside the story that creates the actual exploit surface, not as a standalone patch.

**Disposition:** [deferred] — attached to STORY-046 as a pre-condition. Recorded in
STATE.md Follow-Ups table. This is NOT a silent drop; it is a codified dependency deferral.

---

## Pending-Intent Enhancements

### OBS-077-P24-A: Underscore (`_`) Italic Flanking Guard Is Open-Only [LOW — pending-intent]

The current italic flanking guard implements DIR-077-002 §1: "NO right-flanking delimiter
run for `_` (no italic closing at word-boundary when preceded by alphanumeric)." This
is SPEC-CONFORMANT behavior.

However, the asymmetry means `_apply file_path here_` closes italic early at the
word-internal underscore in `file_path`, producing `_apply file` + `path here_` split
across italic/non-italic. Users may encounter this with identifier-containing text.

**Not a defect.** DIR-077-002 §1 mandates NO flanking — this is intentional to prevent
false italic triggers in prose. Right-flanking symmetry (matching the `*` rule, which
DOES apply a flanking guard) is a UX enhancement that changes the spec.

**Candidate for:** STORY-081 (slide-level inline markup) or a dedicated PO decision to
amend DIR-077-002 §1 to add right-flanking guard parity with `*`.

**Disposition:** [pending-intent] — recorded for PO + STORY-081 scope decision.

---

### OBS-077-P25-A: Parse→Eval Round-Trip Test Consolidation [LOW — optional]

AC-002 (section register routing) is covered across two test layers:
1. Unit tests in `slideforge-syntax` (parse-side: section blocks produce correct IR)
2. Unit tests in `slideforge-eval` with hand-constructed `SectionBlock` nodes (eval-side)

A single integration test that flows a real `section <type>: ...` source string through
parse → eval → `RegisteredContent` assertion would consolidate the seam verification and
make the AC-002 contract visible at one touch point.

**Not a gap.** The two-layer coverage is sound; the seam is covered. This is a readability
and maintenance enhancement.

**Disposition:** [optional follow-up] — low priority. Can be added in any future test
housekeeping sweep. Not attached to a story.

---

## Recurring Taxonomy Debt — Codified Follow-Up

### E-EVL-007..011 Unregistered + E-PAR-012 Retired-Code Reuse [LOW — follow-up story]

This issue surfaced in **3+ passes** of the STORY-077 cascade (passes 6, 13, 21+), which
per cycle-closing checklist triggers mandatory codification — not merely a note.

**The debt:**
- `E-EVL-007` through `E-EVL-011` are allocated in `slideforge-eval` error handling code
  but do not appear in `.factory/specs/prd-supplements/error-taxonomy.md`. Any tooling
  or documentation that scans the taxonomy for completeness will undercount the eval
  error space by 5 codes.
- `E-PAR-012` reuses a code slot that was previously retired (the slot was vacated when
  an earlier error was refactored). The taxonomy file should mark the slot as retired
  and allocate a fresh code.

**Required action:** A taxonomy reconciliation sweep that:
1. Audits all `E-EVL-*` codes allocated in `slideforge-eval` source against the taxonomy file.
2. Registers all missing codes with canonical names, descriptions, and severity.
3. Marks `E-PAR-012` (and any other retired slots) explicitly as `[RETIRED: <reason>]`.
4. Allocates a new code for the `E-PAR-012` use case if it is still active.

**Codification:** [codified] — this will be tracked as a follow-up story (taxonomy
reconciliation story, to be created in the next story decomposition pass). Priority: P1,
Wave 5 or Wave 6. Must complete before Phase 6 (formal hardening), because Kani proofs
and error-coverage mutation testing will require the taxonomy to be exhaustive.

**Disposition:** [follow-up story — to be created]. Not a blocker for Wave 4 Batch B.
Recorded in STATE.md Follow-Ups table with concrete dependency (Phase 6 / formal hardening).

---

## Process Lessons

### LESSON-P077-A: Fresh-Context Pass Caught a HIGH That "Clean" Pass Missed [validated]

Pass 16 returned strict-CLEAN (0 findings). Pass 17, dispatched with fresh context (no
carry-over from pass 16), found F-077-P17-001 (HIGH): slide-level inline-markup chunks
were silently dropped — text data loss that pass 16 missed despite examining the same code.

**Why pass 16 missed it:** The adversary in pass 16 carried context from the pass-14/15
fix-burst (E-PAR-019/020/021 fatal routing). The cognitive attention was on the error-path
changes; the happy-path data-flow (slide-level evaluation path) was not freshly interrogated.

**Implication:** The strict 3-CLEAN protocol (which requires three CONSECUTIVE clean passes,
not one) and the fresh-context dispatch rule (each pass gets a clean context) are VALIDATED
by this event. The protocol is correctly calibrated.

**Process change:** None. The existing protocol (BC-5.39.001 strict 3-CLEAN + fresh-context
dispatch) already covers this case correctly. This lesson confirms the protocol — do not
weaken it.

**Disposition:** [codified — validates existing protocol, no change needed]

### LESSON-P077-B: Sibling Enumeration Sweep Must Be Generalized [process refinement]

Pass 22 performed a sibling sweep scoped to the section-type list (the enumeration that
triggered the pass-22 finding). Pass 23 found stale enumerations in a different but related
enumeration — the list of `FieldValue` variant arms in a match block that was structurally
parallel to the section-type list but not included in the pass-22 sweep.

**Root cause:** The sweep was scoped too narrowly: "fix the enumeration that was flagged"
rather than "fix all enumerations touched by this story."

**Corrective rule:** When executing a sibling-site sweep for an enumeration change
(TD-VSDD-060), the sweep scope must be: ALL match arms, variant lists, and enum definitions
in the SAME CRATE(S) that enumerate over types introduced or modified in this story — not
just the specific enumeration that the current finding flagged.

**Disposition:** [codified — refinement to TD-VSDD-060 sweep discipline]

---

## Follow-Up Cycle Lessons

### LESSON-FU-A: Scanner Changes Require Upfront Invariant Analysis [process — scanner discipline]

Touching the inline-markup scanner after 3/3 strict-CLEAN convergence triggered a 5-pass sub-cascade (P1 through P5) before re-achieving 3/3 strict-CLEAN. Each pass exposed a deeper invariant in the scanner's closer-prediction / absolute-offset machinery:

- Pass 1 (F-FU-P1-001): closer-prediction desync after offset threading change.
- Pass 2 (F-FU-P2-001): EOF unclosed-delimiter detection missed the new absolute-offset invariant.
- Pass 3 (F-FU-P3-001): stale rustdoc referencing the pre-threading parameter semantics.

Root cause: the fix touched one invariant (bilateral flanking) but the offset-threading change had downstream invariant consequences in closer-prediction agreement, EOF detection, and documentation — none of which were enumerated before implementation began.

**Corrective discipline for all future scanner changes:**

Before implementing any change to the inline-markup scanner (`slideforge-syntax`), explicitly enumerate:
1. **Closer-prediction agreement**: will this change alter which closing delimiter is predicted? Re-verify all opener/closer pairing logic.
2. **Absolute-offset threading**: does this change touch any byte offset passed through the scanner? Re-verify EOF and unclosed-delimiter paths.
3. **EOF unclosed detection**: does the scanner's unclosed-at-EOF path depend on any value this change modifies?
4. **Rustdoc / comment currency**: are any doc comments on modified functions or parameters still accurate?

This is not a new protocol — it is a specific application of TD-VSDD-060 (sibling-site sweep) to the scanner's internal invariant graph. The strict 3-CLEAN + fresh-context cascade correctly surfaced each layer; the lesson is to front-load this analysis and fix all layers in one burst rather than discovering them pass-by-pass.

**Disposition:** [codified — scanner-specific invariant analysis discipline, no process rule change needed beyond this lesson]

---

## Cycle Summary (including follow-up cycle)

| Metric | Value |
|--------|-------|
| Total adversary passes (main cascade) | 26 |
| Total adversary passes (follow-up cascade) | 7 |
| Total findings | ~45+ (across all passes; see burst-log.md) |
| Streak passes (main) | 24, 25, 26 (3/3 strict-CLEAN) |
| Streak passes (follow-up) | passes 3–5 / PR-6 / post-CI-7 (3/3 strict-CLEAN re-achieved) |
| PR (main) | #49 — merged to develop as c8913cad 2026-06-03 |
| PR (follow-up) | #50 — merged to develop as f2573bb1 2026-06-03 |
| CI gate | 16/16 green (both PRs) |
| Security-reviewer | CLEAN (0 crit, 0 important, both PRs) |
| PR-reviewer | APPROVE (0 blocking, both PRs) |
| BCs satisfied | BC-3.02.002 v1.5 (AC-001..006), BC-1.14.003 v1.3 (AC-EC-001) |
| Error taxonomy | v2.13 — E-PAR-022 + E-EVL-007–011 registered; E-PAR-012 retired |
| Wave 4 Batch A | 10/10 COMPLETE (STORY-077 was the final story; follow-up PRs are post-batch cleanup) |
