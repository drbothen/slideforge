# Red Gate Log — STORY-013

**Story:** STORY-013: @if/@elif/@else Evaluation + @include Cycle Detection
**Crate:** `slideforge-eval`
**Date:** 2026-05-26
**Agent:** test-writer

## Summary

Red Gate verified. All 18 new tests fail with `todo!()` panics before
implementation begins. 2 structural tests (error type construction and format)
pass as expected — they test the `EvalError::IncludeCycle` variant that was
added to `error.rs`, not the unimplemented detection function.

## Test Files Created

| File | Tests | Status |
|------|-------|--------|
| `crates/slideforge-eval/src/if_eval.rs` | 13 | ALL FAIL (todo!) |
| `crates/slideforge-eval/src/include_cycle.rs` | 7 | 5 FAIL (todo!) + 2 PASS (structural) |
| `crates/slideforge-eval/src/error.rs` | Extended | +2 new pass (error type tests) |

## Test Run Results

```
test result: FAILED. 126 passed; 18 failed; 0 ignored; 0 measured
```

All 18 new behavioral tests fail with `todo!()` panics at the stub call sites:
- `if_eval.rs:88` — `eval_if_chain` stub
- `include_cycle.rs:110` — `check_include_cycles` stub

## Failing Tests (18 total)

### if_eval.rs (13 tests, all failing)

| Test | BC Clause | Fail Reason |
|------|-----------|-------------|
| `test_BC_1_05_001_if_true_renders_block` | BC-1.05.001 postcondition 1 / AC-001 | todo!() |
| `test_BC_1_05_001_if_false_no_output` | BC-1.05.001 / AC-002 | todo!() |
| `test_BC_1_05_001_if_elif_else_first_truthy` | BC-1.05.001 postcondition 2 / AC-002 | todo!() |
| `test_BC_1_05_001_if_elif_else_second_truthy` | BC-1.05.001 postcondition 3 / AC-002 | todo!() |
| `test_BC_1_05_001_if_else_fallback` | BC-1.05.001 postcondition / EC-002 | todo!() |
| `test_BC_1_05_002_if_string_condition_type_error` | BC-1.05.002 postcondition 2-3 / AC-005 | todo!() |
| `test_BC_1_05_002_if_int_condition_type_error` | BC-1.05.002 postcondition 2 / AC-005 | todo!() |
| `test_BC_1_05_002_if_bool_condition_valid` | BC-1.05.002 postcondition 1 / AC-006 | todo!() |
| `test_BC_1_05_001_if_lazy_evaluation` | BC-1.05.001 invariant 2 / AC-004 | todo!() |
| `test_BC_1_05_002_if_pipe_in_condition` | BC-1.05.002 postcondition 1 / AC-007 | todo!() |
| `test_BC_1_05_001_if_undefined_condition_var` | BC-1.05.001 EC-005 / AC-008 | todo!() |
| `test_BC_1_05_001_if_slide_scope_atomic_exclusion` | BC-1.05.001 EC-001 | todo!() |
| `test_BC_1_05_001_all_branches_false_no_else` | BC-1.05.001 EC-002 | todo!() |

### include_cycle.rs (5 behavioral tests, all failing)

| Test | BC Clause | Fail Reason |
|------|-----------|-------------|
| `test_BC_1_06_002_no_cycle_single_file` | BC-1.06.002 invariant | todo!() |
| `test_BC_1_06_002_direct_cycle` | BC-1.06.002 postcondition 1-2 / AC-009 | todo!() |
| `test_BC_1_06_002_self_include` | BC-1.06.002 postcondition 1 / AC-010 | todo!() |
| `test_BC_1_06_002_diamond_include_no_cycle` | BC-1.06.002 invariant 3 / AC-011 | todo!() |
| `test_BC_1_06_002_deep_chain_no_cycle` | BC-1.06.002 VP / AC-012 | todo!() |

## Passing Tests (structural — correct pre-implementation)

| Test | Reason passes pre-implementation |
|------|----------------------------------|
| `test_BC_1_06_002_include_cycle_error_constructible` | Tests the error TYPE (added to error.rs), not the detection function |
| `test_BC_1_06_002_cycle_error_message_format` | Tests the error message FORMAT (thiserror derive), not the detection function |

These two tests verify the `EvalError::IncludeCycle` variant compiles correctly
and produces the right message format per AC-009. They are structural tests that
don't call `check_include_cycles` — passing pre-implementation is correct behavior.

## BC Coverage

| BC | Postconditions | Invariants | Edge Cases | Covered |
|----|---------------|------------|------------|---------|
| BC-1.05.001 | 3/3 | 2/2 | EC-001, EC-002, EC-005 | YES |
| BC-1.05.002 | 3/3 | 0 (N/A) | EC-004 (pipe) | YES |
| BC-1.06.002 | 2/2 | 1/1 | VP (depth) | YES |

## Stubs Created

### `crates/slideforge-eval/src/if_eval.rs`

- `eval_if_chain(env, if_node, set_rule_defaults, config, sink) -> Vec<Slide>` — stub with `todo!()`
- `eval_bool_condition(env, condition, span, sink) -> Option<bool>` — stub with `todo!()`

### `crates/slideforge-eval/src/include_cycle.rs`

- `IncludeGraph` type alias (`HashMap<Arc<str>, Vec<Arc<str>>>`)
- `check_include_cycles(root, graph, sink) -> bool` — stub with `todo!()`

### `crates/slideforge-eval/src/error.rs` (extended)

- Added `EvalError::IncludeCycle { cycle_path: Vec<Arc<str>>, span: SourceSpan }` with code `E-PAR-004`
- Added `format_cycle_path(cycle_path: &[Arc<str>]) -> String` helper

## Implementer Instructions

Make each test pass, one at a time, with minimum code:

1. Start with `eval_bool_condition` in `if_eval.rs` (drives type-check tests)
2. Then `eval_if_chain` dispatch (drives the main @if chain tests)
3. Then `check_include_cycles` DFS in `include_cycle.rs`
4. Wire `eval_if_chain` into `eval_block_items` in `for_eval.rs` (replaces the `TODO(STORY-013)` stub)
5. Add the include cycle pre-pass call in `eval_deck_with_variant` in `eval.rs`

**Critical implementation requirement for test_BC_1_05_001_if_lazy_evaluation:**
The `@elif` condition `undefined_var == "x"` must NEVER be evaluated when the
`@if` condition is `true`. Use early return after finding the first truthy branch.
