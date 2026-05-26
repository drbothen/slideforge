//! `slideforge-syntax` — lexer and parser for the slideforge DSL.
//!
//! This crate converts raw `.sf` source text into a typed token stream and,
//! in later stories (STORY-006 through STORY-009), into a full abstract syntax
//! tree (AST). The pipeline stage is:
//!
//! ```text
//! .sf source text
//!   → [slideforge-syntax::lex]   → Vec<Spanned<Token>>
//!   → [slideforge-syntax::parse] → Ast  (STORY-006+)
//!   → [slideforge-eval]          → Deck
//! ```
//!
//! # Current Status (STORY-005)
//!
//! The lexer layer is fully implemented. The parser layer (`parse`, `ast`)
//! will be added in STORY-006 through STORY-009.
//!
//! # Architecture Constraints (SS-01)
//!
//! `slideforge-syntax` is **Pure Core** — it performs no I/O, no filesystem
//! access, and no network calls. The caller (typically `slideforge-cli`) is
//! responsible for reading the `.sf` file and passing the in-memory string to
//! [`lex`].
//!
//! # Error Accumulation (DI-018)
//!
//! The [`lex`] function accumulates **all** lexer errors in a single pass and
//! returns them alongside the (possibly partial) token stream. It never panics
//! or fails fast on the first error.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod lexer;
pub mod lexer_error;
pub mod token;

// Re-export the public API surface at the crate root for ergonomic use.

pub use lexer::lex;
pub use lexer_error::LexError;
pub use token::{LexerMode, Spanned, Token};
