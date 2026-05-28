---
story: STORY-014
title: "Type System: No Implicit Coercion + ${{ seq }} Disambiguation"
phase: red-gate
date: 2026-05-27
agent: vsdd-factory:test-writer
---

# Red Gate Log — STORY-014

## Test Suite

**File:** `crates/slideforge-eval/tests/type_system_tests.rs`
**Tests:** 22 integration tests
**Kani stubs:** `crates/slideforge-eval/src/proofs/no_coercion.rs`, `no_overflow.rs`

## Red Gate Results

```
cargo test -p slideforge-eval --test type_system_tests --no-fail-fast

running 22 tests
...
test result: FAILED. 18 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out
```

## Failing Tests (Red Gate Verified)

| Test Name | BC | Failure Reason |
|-----------|-----|----------------|
| `test_bc_1_02_003_compare_string_to_bool_produces_type_error` | BC-1.02.003 §5 | `eval_binop` for `BinOpKind::Eq` does not check type compatibility between `Str` and `Bool`. Returns `Bool(false)` instead of `None` + E-EVL-003. |
| `test_bc_1_02_003_string_in_condition_type_error` | BC-1.02.003 inv-2 | `BlockItem::If` evaluation is a no-op stub in `eval_block_items`. No E-EVL-003 is produced for `@if "true":`. |
| `test_bc_1_02_003_int_in_condition_type_error` | BC-1.02.003 inv-2 | Same as above: `@if 0:` produces no error (stub). |
| `test_bc_1_02_003_arith_string_rate_hint_mentions_float_filter` | BC-1.02.003 §4, EC-002 | E-EVL-003 error for `Str * Int` exists but has no hint mentioning `| float` conversion path. |

## Passing Tests (18 — Regression Guards and Documentation)

The 18 passing tests document CORRECT behaviors that exist in STORY-011/012:

- `test_bc_1_02_003_invariant_str_*` — `Expr::Str` never coerces to Int/Bool (already correct)
- `test_bc_1_02_003_no_flag_string_stays_no` — string "NO" preserved through `eval_deck`
- `test_bc_1_02_003_version_string_stays_unchanged` — "1.10" not normalized to "1.1"
- `test_bc_1_02_003_yes_string_stays_yes` — "yes" not coerced to true
- `test_bc_1_02_003_zero_string_stays_string` — "0" not coerced to 0
- `test_bc_1_02_003_arith_on_string_produces_type_error` — Str + Int → E-EVL-003
- `test_bc_1_02_003_explicit_int_conversion` — `| int` filter works
- `test_bc_1_02_003_explicit_float_conversion` — `| float` filter works
- `test_bc_1_02_003_bool_filter_does_not_exist` — `| bool` → E-EVL-004 (regression guard)
- `test_bc_1_02_003_bool_filter_help_text_excludes_bool` — regression guard
- `test_bc_1_02_003_int_string_concat_type_error` — Str + Int → E-EVL-003
- `test_bc_1_02_004_dollar_interpolation_basic` — "$" + currency filter concatenation
- `test_bc_1_02_004_dollar_interpolation_count` — "$" + count + " items"
- `test_bc_1_02_004_dollar_space_not_interpolation` — "$ 100" (space is load-bearing)
- `test_bc_1_02_004_dollar_dollar_brace_is_math_mode_not_interpolation` — MathDisplay passthrough
- `test_bc_1_02_004_dollar_interp_with_suffix` — "$1500 USD"

These 18 tests pass immediately because STORY-011 already implemented these behaviors.
They are kept as REGRESSION GUARDS per BC-5.39.001 — adding `| bool` in a future story
would cause `test_bc_1_02_003_bool_filter_does_not_exist` to fail (correct catch).

## What the Implementer Must Do

1. **Add `Eq`/`Ne` type checking to `eval_binop`:** `BinOpKind::Eq` and `BinOpKind::Ne`
   currently apply structural equality for ALL type combinations. Per BC-1.02.003
   postcondition 5, comparing `Str` to `Bool` (or `Int` to `Bool`, etc.) must produce
   E-EVL-003. Only same-type or numerically-compatible comparisons are allowed without error.

2. **Implement `@if` condition type checking in `eval_block_items`:** The `BlockItem::If`
   arm currently silently skips the block. The implementer must evaluate the condition
   expression, verify it is `Value::Bool`, and push E-EVL-003 if it is any other type.

3. **Add `| float` hint to arithmetic TypeMismatch on string operands:** When
   `eval_arithmetic` fires E-EVL-003 for a string operand, the error message or help
   text must mention `| float` (or `| int`) as the explicit conversion path.

## Kani Stubs

Kani proof stub modules compile clean under `#[cfg(kani)]`:

```
crates/slideforge-eval/src/proofs/mod.rs        — declares no_coercion, no_overflow
crates/slideforge-eval/src/proofs/no_coercion.rs  — VP-004 stubs (todo!)
crates/slideforge-eval/src/proofs/no_overflow.rs  — VP-005 stubs (todo!)
```

AC-011 is satisfied: `cargo check -p slideforge-eval` (without `--cfg kani`) produces
zero compile errors. Full proofs deferred to STORY-067.

## Existing Test Regression Check

```
cargo test -p slideforge-eval --lib --no-fail-fast

test result: ok. 124 passed; 0 failed; 0 ignored; 0 measured
```

Zero regressions in existing 124 unit tests.
