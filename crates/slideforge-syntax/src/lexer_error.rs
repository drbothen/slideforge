//! Lexer error types for the slideforge DSL.
//!
//! All lexer errors carry exact `file:line:col` location information so that
//! downstream consumers (the CLI, the language server) can render precise
//! source pointers via `miette`.
//!
//! The lexer uses **error accumulation** (DI-018): it never fails fast on the
//! first error. Instead it pushes every error it encounters into a
//! `Vec<LexError>` and continues scanning. The parser (STORY-006) decides
//! what to do with the accumulated errors.

use std::sync::Arc;

/// An error produced by the slideforge lexer.
///
/// Every variant carries the source file name and the exact byte position
/// (line + column, 1-indexed) at which the problem was detected.
///
/// # Error Accumulation
///
/// The `lex` function never returns `Err`. Instead it returns a
/// `(Vec<Spanned<Token>>, Vec<LexError>)` tuple. When `errors` is non-empty
/// the token stream may still be partially populated — the lexer continues
/// scanning after every error to collect as many diagnostics as possible in
/// a single pass.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LexError {
    /// A tab character (`\t`) was found in the leading whitespace of a line.
    ///
    /// slideforge requires spaces for indentation. Tabs are rejected at the
    /// lexer level, before the chumsky parser layer runs, so that detection
    /// is deterministic and amenable to formal verification (VP-001).
    ///
    /// # Example message
    ///
    /// ```text
    /// Tab character at deck.sf:3:1. slideforge requires spaces for indentation.
    /// ```
    #[error("Tab character at {file}:{line}:{col}. slideforge requires spaces for indentation.")]
    TabIndentation {
        /// Source file path (or `"<unknown>"`).
        file: Arc<str>,
        /// One-based line number.
        line: u32,
        /// One-based column number.
        col: u32,
        /// Byte offset from the start of the source string.
        byte_offset: usize,
    },

    /// An indentation level on the current line does not match any level
    /// that is on the indent stack (neither a push nor a valid pop).
    ///
    /// This happens when a line has, for example, 3 spaces of indentation
    /// after a block that used 2 spaces, and no enclosing block used 3.
    #[error(
        "Inconsistent indentation at {file}:{line}:{col}: \
         expected {expected} spaces but got {got}."
    )]
    IndentationInconsistency {
        /// Source file path.
        file: Arc<str>,
        /// One-based line number.
        line: u32,
        /// One-based column number of the first non-space character.
        col: u32,
        /// The indentation level that was expected (top of the indent stack).
        expected: usize,
        /// The actual indentation level encountered.
        got: usize,
    },

    /// A string literal was opened with `"` but no closing `"` was found
    /// before the end of the current line or the end of the file.
    #[error("Unterminated string literal at {file}:{line}:{col}.")]
    UnterminatedString {
        /// Source file path.
        file: Arc<str>,
        /// One-based line number where the string opened.
        line: u32,
        /// One-based column number of the opening `"`.
        col: u32,
    },

    /// A math block (`$...$` or `$$...$$`) was opened but no matching
    /// closing delimiter was found before the end of the file.
    #[error("Unterminated math block at {file}:{line}:{col}.")]
    UnterminatedMath {
        /// Source file path.
        file: Arc<str>,
        /// One-based line number where the math delimiter appeared.
        line: u32,
        /// One-based column number of the opening delimiter.
        col: u32,
    },

    /// A byte that is not valid in any slideforge DSL context was encountered.
    ///
    /// The lexer records the character and its position, then skips it and
    /// continues scanning.
    #[error("Invalid character {ch:?} at {file}:{line}:{col}.")]
    InvalidCharacter {
        /// Source file path.
        file: Arc<str>,
        /// One-based line number.
        line: u32,
        /// One-based column number.
        col: u32,
        /// The offending character.
        ch: char,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc_1_01_003_tab_indentation_error_message() {
        let e = LexError::TabIndentation {
            file: Arc::from("deck.sf"),
            line: 3,
            col: 1,
            byte_offset: 42,
        };
        let msg = e.to_string();
        assert!(msg.contains("deck.sf:3:1"), "message must include file:line:col");
        assert!(
            msg.contains("spaces for indentation"),
            "message must mention spaces requirement"
        );
    }

    #[test]
    fn test_bc_1_01_003_tab_indentation_carries_byte_offset() {
        let e = LexError::TabIndentation {
            file: Arc::from("x.sf"),
            line: 1,
            col: 1,
            byte_offset: 99,
        };
        if let LexError::TabIndentation { byte_offset, .. } = e {
            assert_eq!(byte_offset, 99);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn test_bc_1_01_003_indentation_inconsistency_error_message() {
        let e = LexError::IndentationInconsistency {
            file: Arc::from("deck.sf"),
            line: 5,
            col: 4,
            expected: 2,
            got: 3,
        };
        let msg = e.to_string();
        assert!(msg.contains("deck.sf:5:4"));
        assert!(msg.contains("expected 2"));
        assert!(msg.contains("got 3"));
    }

    #[test]
    fn test_bc_1_01_003_unterminated_string_error_message() {
        let e = LexError::UnterminatedString {
            file: Arc::from("a.sf"),
            line: 2,
            col: 7,
        };
        let msg = e.to_string();
        assert!(msg.contains("a.sf:2:7"));
        assert!(msg.contains("Unterminated string"));
    }

    #[test]
    fn test_bc_1_01_003_unterminated_math_error_message() {
        let e = LexError::UnterminatedMath {
            file: Arc::from("b.sf"),
            line: 10,
            col: 3,
        };
        let msg = e.to_string();
        assert!(msg.contains("b.sf:10:3"));
        assert!(msg.contains("Unterminated math"));
    }

    #[test]
    fn test_bc_1_01_003_invalid_character_error_message() {
        let e = LexError::InvalidCharacter {
            file: Arc::from("c.sf"),
            line: 1,
            col: 5,
            ch: '\u{0000}',
        };
        let msg = e.to_string();
        assert!(msg.contains("c.sf:1:5"));
        assert!(msg.contains("Invalid character"));
    }

    #[test]
    fn test_bc_1_01_003_lex_error_derives_debug_clone_eq() {
        let e = LexError::TabIndentation {
            file: Arc::from("f.sf"),
            line: 1,
            col: 1,
            byte_offset: 0,
        };
        let e2 = e.clone();
        assert_eq!(e, e2);
        let _ = format!("{e:?}");
    }
}
