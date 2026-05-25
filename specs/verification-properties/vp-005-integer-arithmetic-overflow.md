---
document_type: verification-property
vp_id: VP-005
title: Integer arithmetic in {{ expr }} no overflow
module: slideforge-eval
tool: Kani
phase: P6
priority: P1
status: draft
bc_trace: [BC-1.02.001, DI-004]
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-005: Integer Arithmetic in {{ expr }} No Overflow

## Property Statement

For any integer arithmetic expression `{{ a + b }}`, `{{ a - b }}`, `{{ a * b }}`,
or `{{ a / b }}` where `a` and `b` are `Value::Int(i64)`, the evaluator MUST NOT
produce a result via Rust's wrapping or overflowing arithmetic — it MUST detect
overflow and return `EvalError::ArithmeticOverflow` rather than silently producing
an incorrect result.

Formally: `∀ a: i64, b: i64 where checked_op(a, b) = None`,
`eval_binop(Op::Add, Value::Int(a), Value::Int(b)) = Err(EvalError::ArithmeticOverflow)`.

## Motivation

BC-1.02.001 specifies that expression evaluation must produce correct arithmetic
results. DI-004 ("Values in the DSL are never silently coerced") extends to arithmetic
correctness: silent overflow is a form of silent value corruption. A presentation
template that sums financial figures or counts must not silently wrap values. This is
especially critical for data-driven slides where `@data` sources provide unbounded
integers from user-supplied CSV/JSON files.

## Feasibility Assessment

Feasible. The evaluator uses `Value::Int(i64)` for integer values. Rust's
`i64::checked_add`, `i64::checked_sub`, `i64::checked_mul`, and `i64::checked_div`
return `None` on overflow — the evaluator must use these instead of bare `+`, `-`,
`*`, `/` operators. Kani can exhaustively verify for symbolically-chosen `i64` pairs
that every overflow case produces `Err(EvalError::ArithmeticOverflow)` and every
non-overflow case produces `Ok(Value::Int(_))`.

The division-by-zero case (denominator = 0) is a separate error variant
`EvalError::DivisionByZero` and is naturally covered by the same harness.

## Proof Harness Skeleton

```rust
// crates/slideforge-eval/src/proofs/arithmetic_overflow.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn add_overflow_is_detected() {
        let a: i64 = kani::any();
        let b: i64 = kani::any();
        let result = eval_binop(BinOp::Add, Value::Int(a), Value::Int(b));
        match a.checked_add(b) {
            Some(expected) => {
                kani::assert!(matches!(result, Ok(Value::Int(v)) if v == expected));
            }
            None => {
                kani::assert!(matches!(result, Err(EvalError::ArithmeticOverflow)));
            }
        }
    }

    #[kani::proof]
    fn sub_overflow_is_detected() {
        let a: i64 = kani::any();
        let b: i64 = kani::any();
        let result = eval_binop(BinOp::Sub, Value::Int(a), Value::Int(b));
        match a.checked_sub(b) {
            Some(expected) => {
                kani::assert!(matches!(result, Ok(Value::Int(v)) if v == expected));
            }
            None => {
                kani::assert!(matches!(result, Err(EvalError::ArithmeticOverflow)));
            }
        }
    }

    #[kani::proof]
    fn mul_overflow_is_detected() {
        let a: i64 = kani::any();
        let b: i64 = kani::any();
        let result = eval_binop(BinOp::Mul, Value::Int(a), Value::Int(b));
        match a.checked_mul(b) {
            Some(expected) => {
                kani::assert!(matches!(result, Ok(Value::Int(v)) if v == expected));
            }
            None => {
                kani::assert!(matches!(result, Err(EvalError::ArithmeticOverflow)));
            }
        }
    }

    #[kani::proof]
    fn div_by_zero_is_detected() {
        let a: i64 = kani::any();
        let result = eval_binop(BinOp::Div, Value::Int(a), Value::Int(0));
        kani::assert!(matches!(result, Err(EvalError::DivisionByZero)));
    }

    #[kani::proof]
    fn div_overflow_i64_min_neg1_is_detected() {
        // i64::MIN / -1 overflows because the result exceeds i64::MAX
        let result = eval_binop(BinOp::Div, Value::Int(i64::MIN), Value::Int(-1));
        kani::assert!(matches!(result, Err(EvalError::ArithmeticOverflow)));
    }
}
```

## Test Coverage (before Phase 6)

Concrete unit tests in `slideforge-eval/src/tests/arithmetic.rs`:
- `test_add_overflow_returns_error` — tests `i64::MAX + 1`
- `test_sub_overflow_returns_error` — tests `i64::MIN - 1`
- `test_mul_overflow_returns_error` — tests `i64::MAX * 2`
- `test_div_by_zero_returns_error` — tests `1 / 0`
- `test_div_i64_min_neg1_returns_error` — tests the edge case `i64::MIN / -1`
