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
/// | `ReservedKeyword` | `E-PAR-006` |
/// | `VarNameCollision` | `E-PAR-008` |
/// | `RawKeyword` | `E-PAR-009` |
/// | `VersionError` | `E-PAR-010` |
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

    /// A reserved keyword was used as an identifier.
    ///
    /// Code: `E-PAR-006`
    ///
    /// Emitted when the parser encounters an identifier that is reserved for a
    /// future language feature (e.g. `@fn`, `@mixin`, `@while`). The keyword
    /// exists in the grammar but is not yet implemented.
    #[error("Reserved keyword '{keyword}' used at {file}:{line}:{col}: {message}")]
    #[diagnostic(
        code("E-PAR-006"),
        help(
            "'{keyword}' is reserved for a future slideforge feature. \
             Choose a different name or wait for the feature to be implemented."
        )
    )]
    ReservedKeyword {
        /// Source file path.
        file: String,
        /// One-based line number.
        line: u32,
        /// One-based column number.
        col: u32,
        /// The reserved keyword that was encountered.
        keyword: String,
        /// Human-readable description of why the keyword is reserved.
        message: String,
        /// Source code context for miette rendering.
        #[source_code]
        src: NamedSource<String>,
        /// Source span highlighting the reserved keyword.
        #[label("reserved keyword here")]
        at: SourceSpan,
    },

    /// A variable name collides with a slide-type keyword.
    ///
    /// Code: `E-PAR-008`
    ///
    /// Emitted when a `vars:` entry uses a name that exactly matches one of the
    /// 31 built-in slide type keywords (e.g. `title`, `content`, `chart`).
    /// This would cause ambiguity during evaluation.
    ///
    /// Note: the `raw` keyword produces `VarNameCollision` (E-PAR-008) when
    /// used as a variable name in a `vars:` block — the variable name
    /// collision check fires before the raw-keyword check (AC-011).
    #[error(
        "Variable name '{name}' at {file}:{line}:{col} collides with a slide type keyword: {message}"
    )]
    #[diagnostic(
        code("E-PAR-008"),
        help(
            "'{name}' is a built-in slide type. Use a different variable name, \
             e.g. '{name}_data' or 'my_{name}'."
        )
    )]
    VarNameCollision {
        /// Source file path.
        file: String,
        /// One-based line number.
        line: u32,
        /// One-based column number.
        col: u32,
        /// The variable name that caused the collision.
        name: String,
        /// Human-readable description of the collision.
        message: String,
        /// Source code context for miette rendering.
        #[source_code]
        src: NamedSource<String>,
        /// Source span highlighting the colliding name.
        #[label("collides with slide type keyword")]
        at: SourceSpan,
    },

    /// The `raw` keyword or a `raw*` variant was used.
    ///
    /// Code: `E-PAR-009`
    ///
    /// Emitted when `raw`, `raw pptx:`, `raw html:`, or any `raw*` construct
    /// appears at a position where slideforge expects a field name or block
    /// keyword.  Raw escape hatches are IR-internal only; user `.sf` files must
    /// use the shape DSL instead.
    #[error("'raw' keyword rejected at {file}:{line}:{col}: {message}")]
    #[diagnostic(
        code("E-PAR-009"),
        help(
            "The 'raw' escape hatch is not available in user .sf files. \
             Use a 'shape:' block to embed custom shapes."
        )
    )]
    RawKeyword {
        /// Source file path.
        file: String,
        /// One-based line number.
        line: u32,
        /// One-based column number.
        col: u32,
        /// Human-readable description of why `raw` was rejected.
        message: String,
        /// Source code context for miette rendering.
        #[source_code]
        src: NamedSource<String>,
        /// Source span highlighting the `raw` token.
        #[label("'raw' is not allowed here")]
        at: SourceSpan,
    },

    /// A version declaration error.
    ///
    /// Code: `E-PAR-010`
    ///
    /// Emitted in two cases:
    ///
    /// 1. **Missing version**: the `.sf` file does not start with
    ///    `slideforge_version "N"`. The error carries a hint to add the
    ///    declaration.
    /// 2. **Forward-incompatible version**: the declared major version is
    ///    greater than 1 (the only version this build understands). This error
    ///    is **always fatal** — it is never demoted to a warning even if
    ///    `--warn-only` is passed. Parsing halts immediately.
    #[error("Version error at {file}: {message}")]
    #[diagnostic(
        code("E-PAR-010"),
        help(
            "Add 'slideforge_version \"1\"' as the first line of your .sf file. \
             If you see this on a file with a version declaration, the declared \
             version is not compatible with this build of slideforge."
        )
    )]
    VersionError {
        /// Source file path.
        file: String,
        /// Human-readable description of the version problem.
        message: String,
        /// Whether this error is fatal regardless of `--warn-only`.
        ///
        /// Forward-incompatible versions (`>= 2`) are always fatal.
        /// Missing version is also fatal by default.
        is_fatal: bool,
        /// Source code context for miette rendering.
        #[source_code]
        src: NamedSource<String>,
        /// Source span at which the problem was detected.
        #[label("version problem here")]
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

    /// Construct a `ReservedKeyword` error (E-PAR-006).
    #[must_use]
    pub fn reserved_keyword(
        file: String,
        line: u32,
        col: u32,
        keyword: String,
        message: String,
        source_text: String,
        byte_offset: usize,
        token_len: usize,
    ) -> Self {
        let span_len = token_len.max(1);
        let src = NamedSource::new(file.as_str(), source_text);
        Self::ReservedKeyword {
            file,
            line,
            col,
            keyword,
            message,
            src,
            at: SourceSpan::from((byte_offset, span_len)),
        }
    }

    /// Construct a `VarNameCollision` error (E-PAR-008).
    #[must_use]
    pub fn var_name_collision(
        file: String,
        line: u32,
        col: u32,
        name: String,
        message: String,
        source_text: String,
        byte_offset: usize,
        token_len: usize,
    ) -> Self {
        let span_len = token_len.max(1);
        let src = NamedSource::new(file.as_str(), source_text);
        Self::VarNameCollision {
            file,
            line,
            col,
            name,
            message,
            src,
            at: SourceSpan::from((byte_offset, span_len)),
        }
    }

    /// Construct a `RawKeyword` error (E-PAR-009).
    #[must_use]
    pub fn raw_keyword(
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
        Self::RawKeyword {
            file,
            line,
            col,
            message,
            src,
            at: SourceSpan::from((byte_offset, span_len)),
        }
    }

    /// Construct a `VersionError` (E-PAR-010).
    #[must_use]
    pub fn version_error(
        file: String,
        message: String,
        pub_fatal: bool,
        source_text: String,
        byte_offset: usize,
    ) -> Self {
        let src_len = source_text.len();
        let span_len = src_len.saturating_sub(byte_offset).clamp(1, 20);
        let src = NamedSource::new(file.as_str(), source_text);
        Self::VersionError {
            file,
            message,
            is_fatal: pub_fatal,
            src,
            at: SourceSpan::from((byte_offset, span_len)),
        }
    }

    /// Returns `true` if this error is fatal regardless of `--warn-only`.
    ///
    /// Currently, only [`SyntaxError::VersionError`] with `is_fatal == true`
    /// is always-fatal.  All other variants are fatal by default but could be
    /// demoted by `--warn-only` in a future implementation.
    #[must_use]
    pub fn is_always_fatal(&self) -> bool {
        matches!(self, Self::VersionError { is_fatal: true, .. })
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

    // ── STORY-009: new error variant constructors and codes ───────────────────

    #[test]
    fn test_bc_1_09_006_reserved_keyword_error_code_e_par_006() {
        use miette::Diagnostic;
        let e = SyntaxError::reserved_keyword(
            "test.sf".to_string(),
            2,
            1,
            "@fn".to_string(),
            "user-defined functions are planned for v2".to_string(),
            "@fn compute(x):\n".to_string(),
            0,
            3,
        );
        let code = e.code().expect("ReservedKeyword must have diagnostic code");
        let code_str = code.to_string();
        assert!(
            code_str.contains("E-PAR-006"),
            "code must be E-PAR-006; got: {code_str}"
        );
        let msg = e.to_string();
        assert!(
            msg.contains("@fn"),
            "message must include keyword; got: {msg}"
        );
    }

    #[test]
    fn test_bc_1_09_006_reserved_keyword_derives_debug_clone() {
        let e = SyntaxError::reserved_keyword(
            "t.sf".to_string(),
            1,
            1,
            "@fn".to_string(),
            "test".to_string(),
            "@fn\n".to_string(),
            0,
            3,
        );
        let e2 = e.clone();
        let _ = format!("{e:?}");
        let _ = format!("{e2:?}");
    }

    #[test]
    fn test_bc_1_09_008_var_name_collision_error_code_e_par_008() {
        use miette::Diagnostic;
        let e = SyntaxError::var_name_collision(
            "test.sf".to_string(),
            3,
            3,
            "chart".to_string(),
            "chart is a built-in slide type".to_string(),
            "vars:\n  chart: \"data\"\n".to_string(),
            7,
            5,
        );
        let code = e
            .code()
            .expect("VarNameCollision must have diagnostic code");
        let code_str = code.to_string();
        assert!(
            code_str.contains("E-PAR-008"),
            "code must be E-PAR-008; got: {code_str}"
        );
        let msg = e.to_string();
        assert!(
            msg.contains("chart"),
            "message must include var name; got: {msg}"
        );
    }

    #[test]
    fn test_bc_1_09_009_raw_keyword_error_code_e_par_009() {
        use miette::Diagnostic;
        let e = SyntaxError::raw_keyword(
            "test.sf".to_string(),
            2,
            3,
            "raw escape hatch not allowed in user files".to_string(),
            "slide content:\n  raw pptx: \"<a:sp/>\"\n".to_string(),
            18,
            3,
        );
        let code = e.code().expect("RawKeyword must have diagnostic code");
        let code_str = code.to_string();
        assert!(
            code_str.contains("E-PAR-009"),
            "code must be E-PAR-009; got: {code_str}"
        );
    }

    #[test]
    fn test_bc_1_09_010_version_error_code_e_par_010() {
        use miette::Diagnostic;
        let e = SyntaxError::version_error(
            "test.sf".to_string(),
            "forward-incompatible version 2".to_string(),
            true,
            "slideforge_version \"2\"\n".to_string(),
            0,
        );
        let code = e.code().expect("VersionError must have diagnostic code");
        let code_str = code.to_string();
        assert!(
            code_str.contains("E-PAR-010"),
            "code must be E-PAR-010; got: {code_str}"
        );
    }

    #[test]
    fn test_bc_1_09_010_version_error_is_always_fatal_when_incompatible() {
        // AC-005: forward-incompatible versions are always fatal.
        let e = SyntaxError::version_error(
            "t.sf".to_string(),
            "v2 incompatible".to_string(),
            true,
            "slideforge_version \"2\"\n".to_string(),
            0,
        );
        assert!(
            e.is_always_fatal(),
            "VersionError(is_fatal=true) must report is_always_fatal()=true"
        );
    }

    #[test]
    fn test_bc_1_09_010_version_error_non_fatal_not_always_fatal() {
        // A missing-version warning (if ever demoted) must not be marked always-fatal.
        let e = SyntaxError::version_error(
            "t.sf".to_string(),
            "missing version (warning)".to_string(),
            false, // not fatal
            "slide title:\n  title \"T\"\n".to_string(),
            0,
        );
        assert!(
            !e.is_always_fatal(),
            "VersionError(is_fatal=false) must NOT report is_always_fatal()=true"
        );
    }

    #[test]
    fn test_bc_1_09_006_reserved_keyword_has_non_empty_help() {
        use miette::Diagnostic;
        let e = SyntaxError::reserved_keyword(
            "t.sf".to_string(),
            1,
            1,
            "@fn".to_string(),
            "test".to_string(),
            "@fn\n".to_string(),
            0,
            3,
        );
        let help = e.help();
        assert!(help.is_some(), "ReservedKeyword must have help text");
        let help_str = help.map(|h| h.to_string()).unwrap_or_default();
        assert!(!help_str.is_empty(), "help must not be empty");
    }

    #[test]
    fn test_bc_1_09_008_var_name_collision_has_non_empty_help() {
        use miette::Diagnostic;
        let e = SyntaxError::var_name_collision(
            "t.sf".to_string(),
            1,
            1,
            "chart".to_string(),
            "collision".to_string(),
            "vars:\n  chart: \"x\"\n".to_string(),
            0,
            5,
        );
        let help = e.help();
        assert!(help.is_some(), "VarNameCollision must have help text");
    }

    #[test]
    fn test_bc_1_09_009_raw_keyword_has_non_empty_help() {
        use miette::Diagnostic;
        let e = SyntaxError::raw_keyword(
            "t.sf".to_string(),
            1,
            1,
            "raw not allowed".to_string(),
            "raw pptx: \"\"\n".to_string(),
            0,
            3,
        );
        let help = e.help();
        assert!(help.is_some(), "RawKeyword must have help text");
    }
}
