//! `slideforge-syntax` — lexer and parser for the slideforge DSL.
//!
//! This crate converts raw `.sf` source text into a typed token stream and a
//! full abstract syntax tree (AST). The pipeline stage is:
//!
//! ```text
//! .sf source text
//!   → [slideforge-syntax::lex]   → Vec<Spanned<Token>>
//!   → [slideforge-syntax::parse] → DeckNode
//!   → [slideforge-eval]          → Deck
//! ```
//!
//! # Architecture Constraints (SS-01)
//!
//! `slideforge-syntax` is **Pure Core** — it performs no I/O, no filesystem
//! access, and no network calls. The caller (typically `slideforge-cli`) is
//! responsible for reading the `.sf` file and passing the in-memory string to
//! [`lex`] or [`parse`].
//!
//! # Error Accumulation (DI-018)
//!
//! Both the lexer and the parser accumulate **all** errors in a single pass.
//! [`lex`] returns `(tokens, lex_errors)`. [`parse`] converts all lex errors
//! and parse errors into [`SyntaxError`]s and returns them as a single
//! `Vec<SyntaxError>`.
//!
//! # Source Spans
//!
//! All AST nodes carry a [`Span`] identifying their position in the source
//! file by byte offset. The [`SourceMap`] associates file IDs with file paths
//! and text for use in diagnostic rendering.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod ast;
pub mod error;
pub mod expr;
pub mod include;
pub mod keywords;
pub mod known_fields;
pub mod lexer;
pub mod lexer_error;
pub mod parser;
pub mod render;
pub mod sink;
pub mod span;
pub mod template;
pub mod token;

// Re-export the public API surface at the crate root for ergonomic use.

pub use ast::{
    AliasNode, BlockItem, DeckNode, FieldNode, FieldValue, ForNode, IfNode, SectionNode, SetRule,
    SetRuleValue, ShapeNode, SlideNode, TemplateValue, VariantNode, VariantsBlock, VarsBlock,
};
pub use error::{ParseSeverity, SyntaxError};
pub use expr::{BinOpKind, Expr, UnaryOpKind};
pub use include::{resolve_includes, vars_scope_from_deck};
pub use keywords::{
    classify_keyword, is_directive_keyword, is_slide_type_keyword, is_structural_keyword,
};
pub use known_fields::known_fields;
pub use lexer::lex;
pub use lexer_error::LexError;
pub use parser::{ParseResult, parse, parse_checked};
pub use render::DiagnosticRenderer;
pub use sink::{BoxDiagnostic, DiagnosticSink};
pub use span::{SourceFile, SourceMap, Span, Spanned};
pub use template::TemplateChunk;
pub use token::{LexerMode, Token};

// Note: `token::Spanned` (type alias for `(T, Range<usize>)`) is NOT re-exported
// here to avoid name collision with `span::Spanned` (the struct).
// Callers that need the lexer's `Spanned<T>` type should use `token::Spanned`
// or `crate::token::Spanned` directly.
