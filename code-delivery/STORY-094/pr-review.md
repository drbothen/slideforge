# PR #85 — Final Re-Review (LESSON-9) — APPROVE

**PR:** drbothen/slideforge #85 — STORY-094 REND-001/003
**Branch:** feature/STORY-094 → develop
**Re-review delta:** c8f96258 (prior APPROVE head) → a5384c92 (current head)
**Verdict:** ✅ **APPROVE**

---

## Scope of This Re-Review

This is the FINAL re-review following the prior APPROVE at head `c8f96258`. The prior
review carried 3 non-blocking body nits (F1 ripple-sites row, F2 field_spans type, F3
test-count) plus the SEC-001 process note. Two commits have landed since:

- `ed4a9289` — SEC-001 fix (CWE-116 ANSI filename injection)
- `a5384c92` — fmt-only follow-up (CI fmt check had failed on ed4a9289)

The delta touches exactly ONE file: `crates/slideforge/src/lib.rs` (+207 / −4).

---

## 1. Delta Code Review (line-by-line)

### ed4a9289 — SEC-001 fix
- **`sanitize_source_name(name: &str) -> String`** (new, `pub(crate)`, line 756):
  Replaces every `char::is_control()` character with U+FFFD. Fast-path returns an
  unmodified owned copy when no control chars are present (no allocation churn beyond
  the unavoidable `to_owned`). Covers C0 (incl. ESC `\x1b`, BEL, CR, TAB, NUL), DEL,
  and C1 ranges via `char::is_control`. Correct and complete for the CWE-116 surface.
- **Chokepoint wiring in `compile_inner`** (line ~844): `options.source_name` is now
  run through `sanitize_source_name` before `source_map.add_file`. This is the single
  registration point feeding miette's `GraphicalReportHandler`, so sanitizing here
  covers all downstream diagnostic emission. `None` still falls back to `<source>`.
  No behavioral change for the happy path (normal filenames pass through unchanged).
- **`slide_type` safety doc**: doc comment cites
  `slideforge_syntax::lexer::Lexer::scan_ident` constraining slide-type keywords to
  `[a-zA-Z0-9_-]`, so that field can never carry control chars — no sanitization
  needed. Reasonable argument; consistent with the lexer-first DSL design.
- **2 tests** (in `#[cfg(test)] mod tests`, lines 3587 + 3651): one unit test asserting
  the sanitization contract (ESC/BEL/CR replaced, normal Unicode + ASCII unchanged,
  extension preserved), one integration test confirming `compile_inner` does not panic
  on a hostile source name. `.unwrap()/.expect()` usages are confined to these test
  functions — permitted under the CLAUDE.md zero-unwrap rule (tests are exempt).

### a5384c92 — fmt-only
- Pure rustfmt reflow of the `map_or_else` call introduced in ed4a9289. Zero semantic
  change. Confirmed by diff: only whitespace/line-wrapping differs.

**No `unwrap()/expect()` in production scope. No `println!`. No `f64` coordinate use.
No silent fallback. Nothing in the delta touches the forbidden-pattern table.**

---

## 2. PR Body Verification

| Item | Status |
|------|--------|
| F1 (ripple-sites row) | Corrected in body — Per-Crate Summary now lists pdf/docx/plugin-api/html ripple sites with TD-VSDD-060 sibling-site sweep note. ✓ |
| F2 (field_spans type) | Corrected — body now shows `OrderedMap<Arc<str>, SourceSpan>` (deterministic ordering for comemo Hash) consistently in Architecture Decisions and Per-Crate table. ✓ |
| F3 (test-count) | Corrected — Test Evidence reads 4142 pass / 20 skip / 0 fail, matching the badge and the post-SEC-001 suite. ✓ |
| SEC-001 section | Accurate — marked RESOLVED in `ed4a9289`, CWE-116, LOW severity, sanitizer chokepoint + U+FFFD + slide_type lexer-constraint argument all described and matching the code. ✓ |

The body now matches reality on every point flagged in the prior review.

---

## 3. Consistency With Prior APPROVE Rationale

The prior APPROVE rested on: REND-001/003 fixes correct, 5 ACs + E-LAY-008 tested,
20-pass adversarial convergence, CI green, only LOW/non-blocking findings remaining.

Nothing in the c8f96258→a5384c92 delta contradicts that rationale:
- The delta is additive (a security hardening function + tests) and a fmt fix.
- It does not modify the REND-001 layout flow, the REND-003 nvGrpSpPr serialization,
  the E-LAY-008 path, or any AC-covered code.
- The one prior open item — SEC-001 as a process note — is now RESOLVED with a
  load-bearing test (not a paper fix; TD-VSDD-059 satisfied: the unit test exercises
  the real `sanitize_source_name` contract).

## CI Status
`fmt`, `clippy`, `doctest`, `supply-chain`, `pdf-ua1-*`, `check-panic-profile`,
`check-pdf-deps` all PASS on the current head. `test (linux-x86_64)` pending (re-running
on new head); remaining jobs tier-skipped per STORY-091 tiered-CI design. No failures.

---

## Findings

**Blocking:** None.
**Non-blocking:** None new. The prior F1/F2/F3 body nits are all resolved.

Known deferrals (F-094-P1-006 CT_NotesBody raw-string, SPEC-DRIFT-31vs35) remain
human-acknowledgement items, unchanged by this delta and out of scope for code review.

---

## Verdict: ✅ APPROVE

The SEC-001 fix is correct, minimal, well-documented, and test-backed; the fmt follow-up
is cosmetic; the PR body now matches reality on all previously-flagged points. Nothing
in the delta regresses the prior APPROVE basis.
