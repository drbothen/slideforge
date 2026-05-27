//! Kani verification proof stubs for `slideforge-eval`.
//!
//! This module is compiled only when building with Kani (the `kani` cfg flag).
//! Proof function bodies contain `todo!()` stubs that will be filled in during
//! STORY-067.
//!
//! # Verification Properties Covered
//!
//! | Module          | VP       | Description                          |
//! |----------------|---------|--------------------------------------|
//! | `no_coercion`  | VP-004  | No implicit string→numeric coercion  |
//! | `no_overflow`  | VP-005  | No silent integer arithmetic overflow |
//!
//! # Compile-only guarantee (AC-011)
//!
//! These stubs must produce zero compile errors with `cargo check -p
//! slideforge-eval` (with `--cfg kani`). Full proof correctness is deferred to
//! STORY-067.

#[cfg(kani)]
pub mod no_coercion;

#[cfg(kani)]
pub mod no_overflow;
