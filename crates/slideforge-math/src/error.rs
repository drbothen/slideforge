//! Crate-internal math error types.
//!
//! [`MathDiagnostic`] carries a single parser/interpolation/rendering diagnostic.
//! [`MathRendererError`] is the crate-level error enum used throughout
//! `slideforge-math` internals. It converts to the public
//! [`slideforge_plugin_api::MathError`] at the plugin boundary.

use std::sync::Arc;

use slideforge_types::SourceSpan;
use thiserror::Error;

/// A single diagnostic produced by the math parser or interpolator.
///
/// Diagnostics accumulate across a parsing pass — the parser never stops on
/// the first error (BC-1.10.002: error accumulation invariant).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathDiagnostic {
    /// The underlying error.
    pub error: MathRendererError,
    /// The source location of the diagnostic.
    pub span: SourceSpan,
}

impl MathDiagnostic {
    /// Construct a new [`MathDiagnostic`].
    #[must_use]
    pub fn new(error: MathRendererError, span: SourceSpan) -> Self {
        MathDiagnostic { error, span }
    }
}

/// Crate-internal error enum for the `slideforge-math` pipeline.
///
/// This type is richer than the public [`slideforge_plugin_api::MathError`]
/// and carries the extra context needed for user-facing diagnostics (spans,
/// hints, variable names). It is converted to the public type at the
/// plugin-trait boundary in `lib.rs`.
///
/// `#[non_exhaustive]` ensures that adding variants in minor releases does not
/// break downstream crates that match on this enum (`SemVer` hygiene, quality-bar rule).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MathRendererError {
    /// A LaTeX command is not in the supported command set for slideforge.
    ///
    /// Carries a correction hint for user-facing display and a stable machine-
    /// readable error code (`E-EXP-006`) for programmatic handling.
    #[error("[{error_code}] unsupported LaTeX command `\\{command}`: {hint}")]
    UnsupportedCommand {
        /// The unsupported command name (without leading backslash).
        command: Arc<str>,
        /// Source location for the command token.
        span: SourceSpan,
        /// A human-readable correction hint surfaced to the user.
        hint: Arc<str>,
        /// Stable error code for this class of error: always `"E-EXP-006"`.
        error_code: &'static str,
    },

    /// A `@{var}` interpolation reference names a variable not in scope.
    #[error("[{error_code}] undefined variable `{var_name}` in math expression")]
    UndefinedVariable {
        /// The variable name referenced in the `@{{var_name}}` expression.
        var_name: Arc<str>,
        /// Source location of the `@{{...}}` token.
        span: SourceSpan,
        /// Stable error code for this class of error: always `"E-EVL-002"`.
        error_code: &'static str,
    },

    /// A general LaTeX parse error with a free-form description.
    #[error("math parse error: {message}")]
    ParseError {
        /// Human-readable description of the parse failure.
        message: Arc<str>,
        /// Source location of the offending token.
        span: SourceSpan,
    },

    /// The requested operation is not yet implemented in this version.
    ///
    /// Used in stubs only — every `NotYetImplemented` path must have a
    /// failing test driving its implementation.
    #[error("not yet implemented")]
    NotYetImplemented,

    /// A Greek, operator, or symbol command name was empty.
    ///
    /// An empty command name has no defined Unicode mapping and cannot
    /// produce a glyph.
    #[error("math command name is empty — cannot resolve Unicode glyph")]
    EmptyCommandName,

    /// A command name was not found in the Unicode symbol tables.
    ///
    /// The PDF path renderer only supports commands that have a canonical
    /// Unicode mapping.
    #[error("unsupported math symbol: \\{name}")]
    UnsupportedSymbol {
        /// The command name that has no Unicode mapping.
        name: Arc<str>,
    },
}
