//! Parse error types for the slideforge DSL.
//!
//! All variants implement both `thiserror::Error` and `miette::Diagnostic`
//! with a non-`None` `SourceCode` and non-empty `help` text, as required by
//! ADR-012 and the architecture compliance rules for STORY-006.
//!
//! # Error Accumulation
//!
//! The parser accumulates all errors before returning. Callers receive a
//! `Vec<SyntaxError>` — never a single error. The [`crate::parse`] function
//! only returns `Err(errors)` when the vec is non-empty.

use miette::{Diagnostic, NamedSource, SourceSpan};

// ─── SyntaxError ─────────────────────────────────────────────────────────────

/// A parse-time error produced by the slideforge DSL parser.
///
/// Each variant carries enough information to produce a `miette`-rendered
/// diagnostic with a source snippet, a human-readable label, and a hint.
///
/// # Diagnostic codes
///
/// | Variant | Code |
/// |---------|------|
/// | `IndentError` | `E-PAR-001` |
/// | `UnexpectedToken` | `E-PAR-002` |
/// | `UnexpectedEof` | `E-PAR-003` |
#[derive(Debug, Clone, thiserror::Error, Diagnostic)]
pub enum SyntaxError {
    /// Indentation is inconsistent — neither a valid push nor a valid pop.
    ///
    /// Code: `E-PAR-001`
    ///
    /// Emitted when a line's leading-space count does not match any level
    /// on the parser's indent stack. The `expected` field is the innermost
    /// valid level; `found` is what the source actually contained.
    #[error(
        "Unexpected indentation at {file}:{line}:{col}. Expected {expected} spaces, found {found}."
    )]
    #[diagnostic(
        code("E-PAR-001"),
        help(
            "Use exactly {expected} spaces of indentation here. slideforge does not allow tab characters."
        )
    )]
    IndentError {
        /// Source file path (for the error message prefix).
        file: String,
        /// One-based line number.
        line: u32,
        /// One-based column number.
        col: u32,
        /// The indentation level that was expected.
        expected: usize,
        /// The indentation level that was found.
        found: usize,
        /// Source code context for miette rendering.
        #[source_code]
        src: NamedSource<String>,
        /// Source span highlighting the unexpected indentation.
        #[label("unexpected indentation here")]
        at: SourceSpan,
    },

    /// An unexpected token was encountered.
    ///
    /// Code: `E-PAR-002`
    ///
    /// Emitted when the parser encounters a token that does not fit the
    /// grammar at that position.
    #[error("Unexpected token at {file}:{line}:{col}: {message}")]
    #[diagnostic(
        code("E-PAR-002"),
        help("Check the slideforge DSL syntax. See the language reference for valid constructs.")
    )]
    UnexpectedToken {
        /// Source file path.
        file: String,
        /// One-based line number.
        line: u32,
        /// One-based column number.
        col: u32,
        /// Human-readable description of what was unexpected.
        message: String,
        /// Source code context for miette rendering.
        #[source_code]
        src: NamedSource<String>,
        /// Source span highlighting the unexpected token.
        #[label("unexpected token here")]
        at: SourceSpan,
    },

    /// Unexpected end of file.
    ///
    /// Code: `E-PAR-003`
    ///
    /// Emitted when the token stream ends while the parser is still expecting
    /// more tokens to complete a production rule.
    #[error("Unexpected end of file in {file}: {message}")]
    #[diagnostic(
        code("E-PAR-003"),
        help("The file ended unexpectedly. Ensure all blocks are properly closed.")
    )]
    UnexpectedEof {
        /// Source file path.
        file: String,
        /// Human-readable description of what was expected.
        message: String,
        /// Source code context for miette rendering.
        #[source_code]
        src: NamedSource<String>,
        /// Span at which the EOF was detected (points to the last byte).
        #[label("file ended here")]
        at: SourceSpan,
    },
}

// ─── Constructors ─────────────────────────────────────────────────────────────

impl SyntaxError {
    /// Construct an `IndentError` from its parts.
    ///
    /// `source_text` is the full source file text used for miette rendering.
    /// `byte_offset` is the byte position of the indentation error in that text.
    #[must_use]
    pub fn indent_error(
        file: String,
        line: u32,
        col: u32,
        expected: usize,
        found: usize,
        source_text: String,
        byte_offset: usize,
    ) -> Self {
        let span_len = if found > 0 { found } else { 1 };
        let src = NamedSource::new(file.as_str(), source_text);
        Self::IndentError {
            file,
            line,
            col,
            expected,
            found,
            src,
            at: SourceSpan::from((byte_offset, span_len)),
        }
    }

    /// Construct an `UnexpectedToken` error.
    #[must_use]
    pub fn unexpected_token(
        file: String,
        line: u32,
        col: u32,
        message: String,
        source_text: String,
        byte_offset: usize,
        token_len: usize,
    ) -> Self {
        let span_len = token_len.max(1);
        let src = NamedSource::new(file.as_str(), source_text);
        Self::UnexpectedToken {
            file,
            line,
            col,
            message,
            src,
            at: SourceSpan::from((byte_offset, span_len)),
        }
    }

    /// Construct an `UnexpectedEof` error.
    #[must_use]
    pub fn unexpected_eof(file: String, message: String, source_text: String) -> Self {
        let eof_offset = source_text.len().saturating_sub(1);
        let src_len = source_text.len();
        let src = NamedSource::new(file.as_str(), source_text);
        Self::UnexpectedEof {
            file,
            message,
            src,
            at: SourceSpan::from((eof_offset, src_len.saturating_sub(eof_offset).max(1))),
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc_1_01_002_indent_error_message_format() {
        let e = SyntaxError::indent_error(
            "deck.sf".to_string(),
            3,
            1,
            2,
            3,
            "slide title:\n  body:\n   bad\n".to_string(),
            20,
        );
        let msg = e.to_string();
        assert!(
            msg.contains("deck.sf:3:1"),
            "message must include file:line:col; got: {msg}"
        );
        assert!(
            msg.contains("Expected 2 spaces, found 3"),
            "message must mention expected and found; got: {msg}"
        );
    }

    #[test]
    fn test_bc_1_01_002_unexpected_token_message_format() {
        let e = SyntaxError::unexpected_token(
            "deck.sf".to_string(),
            2,
            5,
            "expected field name".to_string(),
            "slide :\n  body\n".to_string(),
            6,
            1,
        );
        let msg = e.to_string();
        assert!(msg.contains("deck.sf:2:5"), "got: {msg}");
        assert!(msg.contains("expected field name"), "got: {msg}");
    }

    #[test]
    fn test_bc_1_01_002_unexpected_eof_message_format() {
        let e = SyntaxError::unexpected_eof(
            "deck.sf".to_string(),
            "expected slide block".to_string(),
            "slide ".to_string(),
        );
        let msg = e.to_string();
        assert!(msg.contains("deck.sf"), "got: {msg}");
        assert!(msg.contains("expected slide block"), "got: {msg}");
    }

    #[test]
    fn test_bc_1_01_002_syntax_error_derives_debug_clone() {
        let e =
            SyntaxError::unexpected_eof("x.sf".to_string(), "test".to_string(), "a".to_string());
        let e2 = e.clone();
        let _ = format!("{e:?}");
        let _ = format!("{e2:?}");
    }

    #[test]
    fn test_bc_1_01_002_indent_error_code_e_par_001() {
        // Verify miette Diagnostic code is accessible
        use miette::Diagnostic;
        let e = SyntaxError::indent_error("f.sf".to_string(), 1, 1, 2, 3, "  bad\n".to_string(), 0);
        let code = e.code();
        assert!(code.is_some(), "IndentError must have a diagnostic code");
        let code_str = code.map(|c| c.to_string()).unwrap_or_default();
        assert!(code_str.contains("E-PAR-001"), "got: {code_str}");
    }

    #[test]
    fn test_bc_1_01_002_syntax_error_has_non_empty_help() {
        use miette::Diagnostic;
        let e = SyntaxError::indent_error("f.sf".to_string(), 1, 1, 2, 3, "  bad\n".to_string(), 0);
        let help = e.help();
        assert!(help.is_some(), "IndentError must have help text");
        let help_str = help.map(|h| h.to_string()).unwrap_or_default();
        assert!(!help_str.is_empty(), "help text must not be empty");
    }
}
