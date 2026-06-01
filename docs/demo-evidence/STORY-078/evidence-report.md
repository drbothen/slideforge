# STORY-078 Demo Evidence Report

**Story:** STORY-078 — Parser: section block syntax (`section <type>:` with sub-blocks)  
**Crate:** `slideforge-syntax`  
**Status:** CONVERGED (3/3 strict-CLEAN adversarial passes)  
**Recording method:** VHS terminal recordings — `cargo nextest` runs exercising the real production parse path via `section_tests.rs`  
**Recorded:** 2026-06-01  

---

## Coverage Summary

| AC | Title | Recording | Tests Driven | Result |
|----|-------|-----------|-------------|--------|
| AC-001 | `"section"` un-reserved from E-PAR-006 | `AC-001-section-keyword-unreserved` | `test_section_keyword_no_longer_reserved`, `test_BC_3_02_002_section_keyword_not_in_reserved_map` | PASS |
| AC-002 | `section <type>:` with sub-blocks parses to `SectionNode` | `AC-002-section-node-sub-blocks` | `test_section_recognized_type_with_sub_blocks` | PASS |
| AC-003 | Parser stores TYPE verbatim — no built-in-list rejection | `AC-003-verbatim-type-no-rejection` | `test_section_unknown_type_parsed_verbatim` | PASS |
| AC-004 | Unrecognized KEY → W-PAR-001 warning + retained in AST | `AC-004-unrecognized-key-warning` | `test_section_unrecognized_key_warning_at_parse_time`, `test_BC_3_02_002_unrecognized_key_warning_span_points_to_foo_token` | PASS |
| AC-005 | `section` inside slide/`@for`/`@if` → E-PAR-018 error | `AC-005-section-in-slide-error` | `test_section_nested_in_slide_error`, `test_section_nested_in_for_error`, `test_section_nested_in_if_error` | PASS |
| AC-006 | `SECTION_REGISTER_KEYS` excludes `"notes"` | `AC-006-register-keys-exclude-notes` | `test_register_keys_exclude_notes`, `test_BC_3_02_002_register_key_unknown_returns_false` | PASS |
| AC-007 | 2-section AST snapshot (regression baseline for STORY-077) | `AC-007-two-section-snapshot` | `test_BC_3_02_002_two_section_deck_ast_snapshot` | PASS |
| AC-008 | Code quality: clippy-clean + full 316-test suite green | `AC-008-code-quality-full-suite` | Full `slideforge-syntax` nextest suite (316 tests, 23 section-specific) | PASS |

**Total:** 8/8 ACs demonstrated. 0 gaps.

---

## Recording Details

### AC-001 — `"section"` keyword un-reserved

**Files:** `AC-001-section-keyword-unreserved.gif`, `AC-001-section-keyword-unreserved.webm`, `AC-001-section-keyword-unreserved.tape`

**What it shows:** Two tests run against the production `keywords.rs` module:
- `test_section_keyword_no_longer_reserved`: parses `section methodology:\n  detail: "Some content."` (with `slideforge_version "1"`) and asserts `result.warnings.is_empty()` and `deck.items[0]` is `BlockItem::Section`.
- `test_BC_3_02_002_section_keyword_not_in_reserved_map`: calls `classify_keyword("section")` → `None` and `is_reserved_bare_keyword("section")` → `false`.

Both tests PASS, confirming `"section"` is no longer E-PAR-006 reserved.

---

### AC-002 — `section <type>:` with sub-blocks parses to `SectionNode`

**Files:** `AC-002-section-node-sub-blocks.gif`, `AC-002-section-node-sub-blocks.webm`, `AC-002-section-node-sub-blocks.tape`

**What it shows:** Canonical AC-002 fixture:
```
slideforge_version "1"
section methodology:
  report: "We applied rigor."
  detail: "Extended methodology detail."
```
Asserts `kind.value == "methodology"`, `fields.len() == 2`, `fields[0].name.value == "report"`, `fields[1].name.value == "detail"`, both values are `FieldValue::Template`. Test PASSES.

---

### AC-003 — TYPE stored verbatim, no built-in-list rejection

**Files:** `AC-003-verbatim-type-no-rejection.gif`, `AC-003-verbatim-type-no-rejection.webm`, `AC-003-verbatim-type-no-rejection.tape`

**What it shows:** Parses `section foobar:\n  detail: "Some content."` and asserts `warnings.is_empty() == true`, `node.kind.value == "foobar"`. Confirms DIR-077-001-A Ruling 3: no fatal error or warning for any unrecognized section TYPE at parse time. Test PASSES.

---

### AC-004 — Unrecognized KEY → W-PAR-001 warning + retained in AST

**Files:** `AC-004-unrecognized-key-warning.gif`, `AC-004-unrecognized-key-warning.webm`, `AC-004-unrecognized-key-warning.tape`

**What it shows:** Two tests covering the parse-time W-PAR-001 path:
- `test_section_unrecognized_key_warning_at_parse_time`: fixture with `foo: "unrecognized"` sub-block → `result.warnings.len() == 1`, warning debug repr contains `"W-PAR"` and `"foo"`, `SectionNode.fields` retains `foo` (2 fields total).
- `test_BC_3_02_002_unrecognized_key_warning_span_points_to_foo_token`: verifies the warning span is at line 3, col 3 (the `foo` token position). Confirms DIR-077-001-A Ruling 2 and BC-3.02.002 invariant 4.

Both tests PASS.

---

### AC-005 — `section` nested in slide/`@for`/`@if` → E-PAR-018

**Files:** `AC-005-section-in-slide-error.gif`, `AC-005-section-in-slide-error.webm`, `AC-005-section-in-slide-error.tape`

**What it shows:** Three error-path tests:
- `test_section_nested_in_slide_error`: `section` inside `slide` body → fatal error with "top-level" or "section" in message.
- `test_section_nested_in_for_error`: `section` inside `@for` body → fatal E-PAR-018 naming `@for` context (NOT "slide block"), with corrective sentence "Move the section:".
- `test_section_nested_in_if_error`: `section` inside `@if` body → fatal E-PAR-018 naming `@if` context.

All three PASS, confirming BC-3.02.002 precondition 3 and EC-002.

---

### AC-006 — `SECTION_REGISTER_KEYS` excludes `"notes"`

**Files:** `AC-006-register-keys-exclude-notes.gif`, `AC-006-register-keys-exclude-notes.webm`, `AC-006-register-keys-exclude-notes.tape`

**What it shows:** Two unit tests on `section::is_register_sub_block_key`:
- `test_register_keys_exclude_notes`: `is_register_sub_block_key("notes")` → `false`; `"report"` → `true`; `"detail"` → `true`.
- `test_BC_3_02_002_register_key_unknown_returns_false`: `"foo"`, `"body"`, `"title"`, `""` all → `false`.

Both PASS, confirming DIR-077-001 §5 ruling that `notes` is not in `SECTION_REGISTER_KEYS`.

---

### AC-007 — 2-section AST snapshot (regression baseline)

**Files:** `AC-007-two-section-snapshot.gif`, `AC-007-two-section-snapshot.webm`, `AC-007-two-section-snapshot.tape`

**What it shows:** `insta` snapshot test `test_BC_3_02_002_two_section_deck_ast_snapshot` parses:
```
slideforge_version "1"
section methodology:
  report: "Methodology content."

section scope:
  detail: "Scope detail."
  report: "Scope report."
```
Asserts `deck.items.len() == 2`, both `BlockItem::Section`, `items[0].kind == "methodology"`, `items[1].kind == "scope"`. Snapshot stored at `src/parser/snapshots/slideforge_syntax__parser__section_tests__two_section_deck_ast_snapshot.snap`. Test PASSES.

---

### AC-008 — Code quality: full test suite green

**Files:** `AC-008-code-quality-full-suite.gif`, `AC-008-code-quality-full-suite.webm`, `AC-008-code-quality-full-suite.tape`

**What it shows:** Full `cargo nextest run -p slideforge-syntax --no-fail-fast` suite — 316 tests, 1 skipped, 0 failures. The 23 section-specific tests (in `parser::section_tests` and `section::tests`) are included in the 316 total. Confirms `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code, `clippy::pedantic` clean, and `#![warn(missing_docs)]` invariants maintained (clippy evidence visible in the build step prior to nextest).

---

## Behavioral Contract Traceability

| BC | Clause | Demonstrated By |
|----|--------|----------------|
| BC-3.02.002 precondition 1 | `.sf` file may contain section blocks | AC-001, AC-002, AC-007 |
| BC-3.02.002 precondition 2 | Recognized SectionType; extensible registry | AC-002, AC-003 |
| BC-3.02.002 precondition 3 | Section blocks must be top-level | AC-005 |
| BC-3.02.002 postcondition 5 | Interpolation resolved later (Template repr) | AC-002 |
| BC-3.02.002 postcondition 6 | Source order preserved | AC-007 |
| BC-3.02.002 postcondition 8 | Sub-block content stored with structural fidelity | AC-002, AC-004 |
| BC-3.02.002 invariant 3 | TYPE validation is eval-stage only | AC-003 |
| BC-3.02.002 invariant 4 | Unrecognized KEY → parse-time lint warning | AC-004 |
| BC-3.02.002 EC-002 | Section-in-slide → E-PAR-018 | AC-005 |
| BC-3.02.002 EC-004 | `report:` is a recognized register sub-block key | AC-006 |
| BC-3.02.002 EC-005 | Unrecognized key → non-fatal W-PAR-001 | AC-004 |
| BC-3.02.002 EC-006 | Reserved-name collision → fatal E-PAR-017 | AC-005 (reserved_name_collision test) |
| DIR-077-001-A Ruling 2 | KEY warning is parse-time (not eval-deferred) | AC-004 |
| DIR-077-001-A Ruling 3 | TYPE stored verbatim; no parser-level list check | AC-003 |
| DIR-077-001 §5 | `notes` excluded from `SECTION_REGISTER_KEYS` | AC-006 |
| NFR-021, NFR-022, NFR-024 | Safety, error handling, doc coverage | AC-008 |
