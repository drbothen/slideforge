---
story_id: STORY-034
phase: red-gate
timestamp: 2026-05-27
agent: test-writer
---

# Red Gate Log — STORY-034

## Summary

All 14 behavioral tests written for BC-1.12.003 fail with `todo!()` panic.
Red Gate is SATISFIED.

## Test run output

```
test result: FAILED. 70 passed; 19 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

19 failures total:
- 14 new `normalize::tests::test_bc_1_12_003_*` tests — fail with `todo!()` from `usvg_normalize` stub
- 5 STORY-033 `render_diagram` integration tests — fail because `render_diagram` now calls
  `usvg_normalize` (stub), which is the correct cascade behavior

70 tests pass (all tests that do not exercise `usvg_normalize`).

## New failing tests (14)

| Test | BC Clause | Fail Reason |
|------|-----------|-------------|
| `test_bc_1_12_003_normalize_simple_svg` | AC-001 postcondition | `todo!()` panic in `usvg_normalize` |
| `test_bc_1_12_003_normalize_returns_normalized_type` | AC-001 return type | `todo!()` panic |
| `test_bc_1_12_003_normalize_propagates_source_id` | AC-001 / error path | `todo!()` panic → can't get error |
| `test_bc_1_12_003_normalize_strips_foreignobject` | AC-002, EC-003 | `todo!()` panic |
| `test_bc_1_12_003_normalize_strips_script` | AC-003 | `todo!()` panic |
| `test_bc_1_12_003_normalize_strips_keyframes_css` | AC-004, EC-005 | `todo!()` panic |
| `test_bc_1_12_003_normalize_resolves_percentage_dimensions` | AC-005, EC-002 | `todo!()` panic |
| `test_bc_1_12_003_normalize_resolves_use_references` | AC-006, EC-001 | `todo!()` panic |
| `test_bc_1_12_003_normalize_preserves_viewbox` | AC-005 companion | `todo!()` panic |
| `test_bc_1_12_003_normalize_invalid_svg_returns_error` | AC-007 / E-EXP-004 | `todo!()` panic → no Err returned |
| `test_bc_1_12_003_normalize_empty_input_returns_error` | AC-001 edge case | `todo!()` panic → no Err returned |
| `test_bc_1_12_003_render_diagram_returns_normalized` | AC-001 / invariant 1 | `todo!()` panic via `render_diagram` |
| `test_bc_1_12_003_render_diagram_output_has_no_foreignobject` | BC-1.12.003 invariant 1 + AC-002 | `todo!()` panic via `render_diagram` |
| `test_bc_1_12_003_normalize_under_budget` | AC-008 / NFR-003 | `todo!()` panic — cannot measure timing |

## Stub test (passes — by design)

`test_bc_1_12_003_usvg_normalize_stub_is_callable` — PASSES.
This test uses `catch_unwind` to confirm the stub panics; it is the Red Gate sentinel.
The implementer will replace the `is_err()` assertion with correct-behavior assertions.

## Verdict

RED GATE SATISFIED. All 14 behavioral tests fail for the right reason (the
`usvg_normalize` stub panics with `todo!()`). No test passes vacuously.

Hand-off to Implementer: make each test pass by implementing `usvg_normalize`
in `crates/slideforge-diagrams/src/normalize.rs`.
