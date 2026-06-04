# PR #55 (STORY-039) — Security Fix-Burst Re-Review

**Verdict: APPROVE** (re-confirmed after security fix-burst `2a46c683`)
**CLEAN (PR-merge): yes** — zero CRIT/HIGH/MED findings.

This re-review is scoped to commit `2a46c683` (SEC-039-001 + SEC-039-002), layered
on the prior APPROVE of the full STORY-039 diff. Files re-reviewed: `error.rs`,
`lib.rs`, `slide_serializer.rs`, `tests/core_tests.rs`.

## What I verified

### 1. `build_doc_props` signature change cleanly threaded
- `build_doc_props` now returns `Result<(), PptxError>`.
- Single call site (`lib.rs:146`) threads the error with `?` — verified by
  `grep -rn build_doc_props crates/` returns exactly one call + one definition.
- No swallowed errors: validation fires via `validate_lang_for_xml(lang_raw)?`
  BEFORE `xml_escape`, so a malformed-XML output is never produced. The final
  `Ok(())` is correctly placed after both parts are pushed.
- Error type is `PptxError` via `thiserror` `#[error(...)]`, consistent with the
  rest of the enum.

### 2. New `InvalidLanguageTag` variant + validator documented and correct
- `PptxError::InvalidLanguageTag { lang, reason }` has a full doc comment AND each
  struct field is individually documented — satisfies the `#![warn(missing_docs)]`
  gate. Confirmed independently: `RUSTDOCFLAGS="-D warnings" cargo doc -p
  slideforge-pptx --no-deps` builds clean.
- `validate_lang_for_xml` is documented (XML 1.0 §2.2 legal-char set cited) and
  the illegal ranges are correct: `0x0000..=0x0008 | 0x000B | 0x000C |
  0x000E..=0x001F | 0xFFFE | 0xFFFF`. Note it intentionally permits #x9 (tab),
  #xA (LF), #xD (CR) and U+FFFD — matching the XML 1.0 legal set. Lossless on
  valid tags (no transformation; original string flows to `xml_escape` unchanged).

### 3. Serializer `tracing::error` fallback is convention-clean
- All three `unwrap_or("")` occurrences (Diagram/Image/Chart arms) replaced with
  `if let Some(s) { s } else { tracing::error!(...); "" }`.
- No `.unwrap()`/`.expect()`/`println!`/`panic!`/`unsafe` in the changed prod code
  (grep-verified). `tracing::error!` with structured fields `slide_index` +
  `frame_idx` is the correct library-crate logging convention.
- Borrow correctness: `frame_idx` is bound by the enclosing `for` loop and
  `slide_index` is a fn parameter — both in scope at every tracing site. `s`
  borrows from `alt_decision_str` which lives the whole iteration; valid.
- Behavior on the Some path is byte-identical to the previous code. Only the
  (contract-unreachable) None path changed from silent to observable. No new test
  for the unreachable branch is acceptable here — it is genuinely unconstructable
  through the public API given the AltTextEmbedder coupling.

### 4. New SEC-039-001 tests are load-bearing, not tautological
- `cargo nextest run -p slideforge-pptx -E 'test(sec039)'` → 3/3 pass.
- **Mutation kill confirmed**: neutering `validate_lang_for_xml` to `return Ok(())`
  causes BOTH negative tests (NUL `test_...nul_control_char...`, SOH
  `test_...soh_control_char...`) to FAIL, while the losslessness test still passes.
  This proves the negatives assert real error behavior and the positive asserts a
  no-false-positive property — none are tautological.
- Negative tests assert `result.is_err()` (not panic, not Ok-malformed) on control
  chars. Positive test asserts valid `en-US` and 4-part `zh-Hant-TW` export Ok AND
  appear unchanged in `docProps/core.xml` (lossless pass-through, BC-5.01.005 inv 1).
- Existing lang test `test_f037_006_core_xml_escapes_language_value` is UNWEAKENED:
  it uses `en-US<&"test>` (XML-special but legal-1.0 chars), a disjoint concern from
  control-char rejection. It still passes; the new validator does not interfere.

### 5. No regression to the rest of the diff; conventions hold
- `#![forbid(unsafe_code)]` still present in `lib.rs`.
- Integer-only codepoint comparison (`ch as u32`) — no `f64`.
- `cargo clippy -p slideforge-pptx --all-targets --all-features -- -D warnings
  -W clippy::pedantic` → clean.
- Full pptx suite: `cargo nextest run -p slideforge-pptx` → 132/132 pass (1 skipped).

## Findings

None blocking. None at CRIT/HIGH/MED. No nits worth raising — the fix-burst is
tight, well-documented, and the tests are independently verified load-bearing.

**PR REVIEW VERDICT: APPROVE**
