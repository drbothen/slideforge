---
story: STORY-078
phase: red-gate
date: 2026-06-01
agent: test-writer
status: RED_GATE_CONFIRMED
---

# Red Gate Log — STORY-078: Parser section block syntax

## Result: RED (compile failures + expected runtime failures)

All tests in the STORY-078 test suite fail before implementation. The Red Gate
is confirmed by two independent categories of failure:

---

## Category 1: Compile-time failures (primary Red Gate)

Two `E0432` errors prevent the test binary from being compiled at all.
These reference `crate::section::is_register_sub_block_key`, a symbol that
does not yet exist (requires the implementer to create `src/section.rs`).

```
error[E0432]: unresolved import `crate::section`
   --> crates/slideforge-syntax/src/parser/section_tests.rs:400:16
    |
400 |     use crate::section::is_register_sub_block_key;
    |                ^^^^^^^ could not find `section` in the crate root

error[E0432]: unresolved import `crate::section`
   --> crates/slideforge-syntax/src/parser/section_tests.rs:423:16
    |
423 |     use crate::section::is_register_sub_block_key;
    |                ^^^^^^^ could not find `section` in the crate root

error: could not compile `slideforge-syntax` (lib test) due to 2 previous errors
error: command `...cargo test --no-run --message-format json-render-diagnostics --package slideforge-syntax` exited with code 101
```

## Category 2: Expected runtime failures (once compile issues are resolved)

Once the implementer creates `section.rs`, the following tests will fail at
runtime because the production code does not yet implement the required behavior:

| Test | Failure Reason |
|------|---------------|
| `test_section_keyword_no_longer_reserved` | `"section"` is still in `RESERVED_KEYWORDS`; parsing fails with E-PAR-006 reserved-keyword error instead of producing `BlockItem::Section` |
| `test_BC_3_02_002_section_keyword_not_in_reserved_map` | `classify_keyword("section")` returns `Some(("E-PAR-006", ...))` instead of `None` |
| `test_section_recognized_type_with_sub_blocks` | No `section_block_parser` → no `BlockItem::Section` in output |
| `test_section_unknown_type_parsed_verbatim` | No `section_block_parser` → parse fails |
| `test_section_unrecognized_key_warning_at_parse_time` | No `section_block_parser` → no warnings in `ParseResult::warnings` |
| `test_BC_3_02_002_unrecognized_key_warning_span_non_empty` | Same — no warnings |
| `test_section_nested_in_slide_error` | Currently produces E-PAR-006 (reserved keyword), not the specific "section blocks must be top-level" error |
| `test_section_spans_present` | No `SectionNode` produced → panic at index |
| `test_section_error_accumulation` | No `section_block_parser` → recovery path not exercised |
| `test_BC_3_02_002_two_section_deck_ast_snapshot` | No `section_block_parser` → snapshot can't be taken |

---

## Files Written

| File | Purpose |
|------|---------|
| `crates/slideforge-syntax/src/parser/section_tests.rs` | Test suite — all failing tests |
| `crates/slideforge-syntax/src/parser/mod.rs` | Added `#[cfg(test)] mod section_tests;` |

---

## Tests Written (by AC)

| Test Name | AC/BC Clause | Failure Type |
|-----------|-------------|--------------|
| `test_section_keyword_no_longer_reserved` | AC-001 | Runtime — E-PAR-006 still fires |
| `test_BC_3_02_002_section_keyword_not_in_reserved_map` | AC-001 | Runtime — `classify_keyword` returns Some |
| `test_section_recognized_type_with_sub_blocks` | AC-002 | Runtime — no parser |
| `test_section_unknown_type_parsed_verbatim` | AC-003 (DIR-077-001-A Ruling 3) | Runtime — no parser |
| `test_section_unrecognized_key_warning_at_parse_time` | AC-004 (DIR-077-001-A Ruling 2) | Runtime — no warnings |
| `test_BC_3_02_002_unrecognized_key_warning_span_non_empty` | AC-004 | Runtime — no warnings |
| `test_section_nested_in_slide_error` | AC-005 | Runtime — wrong error type |
| `test_register_keys_exclude_notes` | AC-006 | Compile — `crate::section` missing |
| `test_BC_3_02_002_register_key_unknown_returns_false` | AC-006 companion | Compile — `crate::section` missing |
| `test_reserved_name_collision` | EC-005/EC-006 | Runtime — no parser |
| `test_section_spans_present` | span propagation | Runtime — no SectionNode |
| `test_section_error_accumulation` | error accumulation / Q23 | Runtime — no parser |
| `test_BC_3_02_002_two_section_deck_ast_snapshot` | AC-007 | Runtime — no parser |

Total: 13 tests across all STORY-078 ACs and edge cases.

---

## No implementation code was written

Confirmed: zero production implementation was added by the test-writer.
The `todo!()` stubs in (the not-yet-existing) `section.rs` remain untouched.
No `section_block_parser` combinator was created.
No changes were made to `keywords.rs`, `deck.rs`, or `ast.rs`.
