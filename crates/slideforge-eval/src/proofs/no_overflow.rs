//! Kani proof stub: no silent integer arithmetic overflow (VP-005).
//!
//! # Invariant
//!
//! All integer arithmetic operations (`+`, `-`, `*`, `/`, `%`) either:
//! a) Produce a correct `Value::Int` result, or
//! b) Push `EvalError::TypeMismatch` (overflow) to the diagnostic sink.
//!
//! No `i64::wrapping_*` or `i64::saturating_*` operations are used —
//! overflow is always detected and surfaced as a diagnostic.
//!
//! # Note
//!
//! This file is compiled only when `#[cfg(kani)]` is active (Kani proof runner
//! mode). The proof bodies are stubs with `todo!()` — full proofs are
//! implemented in STORY-067.

/// VP-005: Verify that integer addition never silently overflows.
///
/// The proof will use `kani::any::<i64>()` pairs and verify that `checked_add`
/// is always used (panicking inputs are impossible; overflow is reported as a
/// diagnostic error).
///
/// Full proof implementation deferred to STORY-067.
#[cfg(kani)]
#[kani::proof]
fn verify_no_int_add_overflow() {
    todo!("Implement in STORY-067")
}

/// VP-005: Verify that integer multiplication never silently overflows.
///
/// Full proof implementation deferred to STORY-067.
#[cfg(kani)]
#[kani::proof]
fn verify_no_int_mul_overflow() {
    todo!("Implement in STORY-067")
}
