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

// These submodule declarations do NOT need `#[cfg(kani)]` — the entire
// `proofs` module is already gated with `#[cfg(kani)]` in `lib.rs`.
// Individual proof functions inside each submodule also carry `#[cfg(kani)]`
// on the function body. Triple-gating is redundant and was flagged in
// adversary pass 1 (FINDING-006, STORY-014).
pub mod no_coercion;

pub mod no_overflow;
