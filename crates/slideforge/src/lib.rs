//! # slideforge
//!
//! Compile a structured DSL into branded `PowerPoint` presentations.
//!
//! This crate is the main entry point. It re-exports the public surface of
//! the workspace's component crates ([`slideforge_syntax`],
//! [`slideforge_eval`], [`slideforge_layout`], [`slideforge_pptx`],
//! [`slideforge_validate`]) and provides high-level convenience APIs.
//!
//! ## Status
//!
//! This crate is in initial scaffolding (Phase 0). The full implementation
//! follows the specification in `seed/PROJECT-SEED.md` at the repository
//! root. See that document and `seed/DSL-GRAMMAR.ebnf` before implementing.

#![warn(missing_docs)]

// Re-exports will be added as component crates are implemented (Phase 1+).
