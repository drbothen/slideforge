//! `slideforge-eval` — expression evaluator and variable environment for the
//! slideforge DSL.
//!
//! This crate implements Phase 3 STORY-011 (Expression Evaluator Core),
//! STORY-012 (Variable Scoping + `@for` Evaluation), and STORY-013
//! (`@if/@elif/@else` Evaluation + `@include` Cycle Detection). It evaluates
//! [`slideforge_syntax::Expr`] AST nodes in a scoped variable [`Env`],
//! evaluates `@for` and `@if/@elif/@else` blocks into sequences of
//! [`slideforge_types::Slide`]s, and aggregates a complete parsed
//! [`slideforge_syntax::DeckNode`] into a semantic [`slideforge_types::Deck`]
//! IR. All errors are accumulated into a [`slideforge_syntax::DiagnosticSink`]
//! without short-circuiting.
//!
//! # Pipeline position
//!
//! ```text
//! .sf source
//!   → slideforge-syntax::lex / parse → DeckNode (AST)
//!   → slideforge-eval::eval_deck     → Deck     (semantic IR)
//!   → slideforge-layout              → LaidOutDeck
//!   → exporters (pptx / pdf / html)
//! ```
//!
//! # Architecture Constraints (SS-02)
//!
//! `slideforge-eval` is **Pure Core** — it performs no I/O, no filesystem
//! access, and no network calls. All side effects are delegated to the caller
//! (typically `slideforge-cli` or `slideforge-layout`).
//!
//! # Error accumulation
//!
//! All public functions accept a `&mut DiagnosticSink` and push errors into it
//! rather than returning `Err`. This follows the same accumulation discipline
//! as `slideforge-syntax`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod config;
pub mod env;
pub mod error;
pub mod eval;
pub mod expr;
pub mod filters;
pub mod for_eval;
pub mod if_eval;
pub mod include_cycle;
pub mod register_routing;

#[cfg(test)]
pub(crate) mod tests;

#[cfg(kani)]
pub mod proofs;

// ─── test-utils feature: BleedChecker ────────────────────────────────────────
//
// The `test-utils` feature gates the `bleed_check` module so that `BleedChecker`
// is available to exporter crate test suites (slideforge-pptx, slideforge-docx,
// etc.) without making `zip` a hard production dependency.
//
// Why `#[cfg(feature = "test-utils")]` here rather than `#[cfg(test)]`:
// - `#[cfg(test)]` items are ONLY compiled in the crate's own test build.
//   They are NOT visible to external crates, even via dev-dependencies.
// - `#[cfg(feature = "test-utils")]` is compiled whenever the feature is active,
//   including when another crate enables it in [dev-dependencies]. This is the
//   only way to share test-only utilities across crate boundaries.
//
// Usage in exporter crates:
//
//   [dev-dependencies]
//   slideforge-eval = { ..., features = ["test-utils"] }
//
//   In test code:
//   use slideforge_eval::BleedChecker;
#[cfg(feature = "test-utils")]
pub mod bleed_check;

#[cfg(feature = "test-utils")]
pub use bleed_check::BleedChecker;

// ─── Public API re-exports ───────────────────────────────────────────────────

pub use config::EvalConfig;
pub use env::Env;
pub use error::EvalError;
pub use eval::{
    eval_deck, eval_deck_with_cycle_check, eval_deck_with_variant, eval_expr_to_string,
};
pub use expr::eval_expr;
pub use filters::{AVAILABLE_FILTERS, apply_filter};
pub use for_eval::{eval_block_items, eval_for_block, eval_slide_node};
pub use if_eval::eval_if_chain;
pub use include_cycle::{IncludeGraph, check_include_cycles};
pub use register_routing::extract_register_content;
