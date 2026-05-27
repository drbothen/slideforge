//! Kani proof stub: no implicit type coercion (VP-004, BC-1.02.003).
//!
//! # Invariant DI-004
//!
//! No `Value::Str` is ever silently converted to `Value::Int`, `Value::Float`,
//! or `Value::Bool` by any evaluator path. Explicit conversion is ONLY via
//! the `| int`, `| float`, and `| string` filter functions. There is no
//! `| bool` filter.
//!
//! # Note
//!
//! This file is compiled only when `#[cfg(kani)]` is active (Kani proof runner
//! mode). The proof bodies are stubs with `todo!()` — full proofs are
//! implemented in STORY-067.

/// VP-004: Verify that `eval_expr` never silently coerces a string literal
/// to a numeric type.
///
/// The proof will enumerate all possible `Expr::Str` literals (within a
/// bounded string set) and verify that any arithmetic operation on them
/// produces `EvalError::TypeMismatch` (E-EVL-003), never a numeric result.
///
/// Full proof implementation deferred to STORY-067.
#[cfg(kani)]
#[kani::proof]
fn verify_no_string_to_int_coercion() {
    todo!("Implement in STORY-067")
}

/// VP-004: Verify that `eval_expr` never silently coerces a string to a bool
/// in a conditional context.
///
/// The proof will verify that `!` applied to `Value::Str` always produces
/// `EvalError::TypeMismatch`, never `Value::Bool`.
///
/// Full proof implementation deferred to STORY-067.
#[cfg(kani)]
#[kani::proof]
fn verify_no_string_to_bool_coercion() {
    todo!("Implement in STORY-067")
}
