//! [`Validator`] trait — checks a [`Deck`] for content and accessibility issues.
//!
//! A `Validator` plugin receives the semantic [`Deck`] IR before export and
//! returns a (possibly empty) list of [`Diagnostic`]s. Validators check for:
//! overflow (text too long for slide), WCAG AA contrast violations, missing
//! `alt` text on visual elements, and format-specific constraints. Built-in
//! validators ship with slideforge; external validators can be registered for
//! domain-specific checks.

use std::sync::Arc;

use slideforge_types::{Deck, SourceSpan};
use thiserror::Error;

/// Severity level for a [`Diagnostic`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticSeverity {
    /// A hard error: the export will fail in strict mode.
    Error,
    /// A warning: reported but does not fail strict-mode export by itself.
    Warning,
    /// An informational note: always reported but never causes failure.
    Info,
}

impl std::fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticSeverity::Error => write!(f, "error"),
            DiagnosticSeverity::Warning => write!(f, "warning"),
            DiagnosticSeverity::Info => write!(f, "info"),
        }
    }
}

/// A diagnostic message produced by a [`Validator`].
///
/// Diagnostics are rendered in the CLI using `miette` with colored source
/// pointers. The `code` field matches the error taxonomy (e.g., `"E-VAL-003"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Diagnostic {
    /// The severity of this diagnostic.
    pub severity: DiagnosticSeverity,

    /// The error or warning code from the error taxonomy (e.g., `"E-VAL-003"`).
    pub code: Arc<str>,

    /// A human-readable message describing the issue.
    pub message: Arc<str>,

    /// The source location this diagnostic points to.
    pub span: SourceSpan,

    /// An optional correction hint shown below the diagnostic in the CLI.
    pub hint: Option<Arc<str>>,
}

/// Options passed to [`Validator::validate`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ValidatorOptions {
    /// When `true`, only `Error`-severity diagnostics are returned (warnings
    /// and infos are suppressed).
    pub errors_only: bool,

    /// When `true`, WCAG AA contrast checks are skipped (for prototyping with
    /// custom brand colors that have not been finalized).
    pub skip_contrast_check: bool,
}

/// A validator error (returned when the validator itself fails, distinct from
/// the diagnostics it produces for the deck content).
#[derive(Debug, Error)]
pub enum ValidatorError {
    /// The validator encountered an internal error and could not complete.
    #[error("validator '{id}' internal error: {message}")]
    InternalError {
        /// The validator plugin ID.
        id: String,
        /// Description of the internal error.
        message: String,
    },
}

/// A plugin that checks a [`Deck`] for content and accessibility issues.
///
/// Validators are run before every export. They produce a list of
/// [`Diagnostic`]s. An empty list means the deck is clean. In strict mode,
/// any `Error`-severity diagnostic causes export to fail.
///
/// Register implementations with [`crate::PluginRegistry::register_validator`].
///
/// ## Thread safety
///
/// All implementations must be `Send + Sync`.
pub trait Validator: Send + Sync {
    /// A unique identifier for this validator plugin (e.g., `"overflow"`,
    /// `"wcag-aa"`, `"alt-text"`).
    fn id(&self) -> &str;

    /// Validate `deck` and return all diagnostics found.
    ///
    /// Returning an empty `Vec` means the deck is clean for this validator.
    fn validate(&self, deck: &Deck, opts: &ValidatorOptions) -> Vec<Diagnostic>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync + ?Sized>() {}

    #[test]
    fn test_bc_5_02_005_validator_trait_is_send_sync() {
        assert_send_sync::<dyn Validator>();
    }

    #[test]
    fn test_bc_5_02_005_validator_options_default() {
        let opts = ValidatorOptions::default();
        assert!(!opts.errors_only);
        assert!(!opts.skip_contrast_check);
    }

    #[test]
    fn test_bc_5_02_005_diagnostic_severity_display() {
        assert_eq!(DiagnosticSeverity::Error.to_string(), "error");
        assert_eq!(DiagnosticSeverity::Warning.to_string(), "warning");
        assert_eq!(DiagnosticSeverity::Info.to_string(), "info");
    }

    #[test]
    fn test_bc_5_02_005_diagnostic_fields() {
        let diag = Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: Arc::from("E-VAL-001"),
            message: Arc::from("text overflow detected"),
            span: SourceSpan::default(),
            hint: Some(Arc::from("reduce font size or text length")),
        };
        assert_eq!(diag.code.as_ref(), "E-VAL-001");
        assert!(diag.hint.is_some());
    }

    #[test]
    fn test_bc_5_02_005_diagnostic_no_hint() {
        let diag = Diagnostic {
            severity: DiagnosticSeverity::Warning,
            code: Arc::from("W-VAL-002"),
            message: Arc::from("low contrast ratio"),
            span: SourceSpan::default(),
            hint: None,
        };
        assert!(diag.hint.is_none());
    }

    #[test]
    fn test_bc_5_02_005_validator_error_internal() {
        let err = ValidatorError::InternalError {
            id: "overflow".to_owned(),
            message: "failed to compute text metrics".to_owned(),
        };
        assert!(err.to_string().contains("internal error"));
    }
}
