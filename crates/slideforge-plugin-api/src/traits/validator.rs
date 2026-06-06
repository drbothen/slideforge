//! [`Validator`] trait — checks a [`Deck`] for content and accessibility issues.
//!
//! A `Validator` plugin receives the semantic [`Deck`] IR before export and
//! returns a (possibly empty) list of [`Diagnostic`]s. Validators check for:
//! overflow (text too long for slide), WCAG AA contrast violations, missing
//! `alt` text on visual elements, and format-specific constraints. Built-in
//! validators ship with slideforge; external validators can be registered for
//! domain-specific checks.

use std::sync::Arc;

use slideforge_layout::LaidOutDeck;
use slideforge_types::{Deck, SourceSpan};

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

/// A plugin that checks a [`Deck`] for content and accessibility issues.
///
/// Validators are run before every export. They produce a list of
/// [`Diagnostic`]s. An empty list means the deck is clean. In strict mode,
/// any `Error`-severity diagnostic causes export to fail.
///
/// Register implementations with [`crate::PluginRegistry::register_validator`].
///
/// ## Two-pass validation (ADR-018)
///
/// The validator is called twice per `build()` invocation:
///
/// 1. **Stage 5 (pre-layout):** [`Validator::validate`] receives the semantic
///    [`Deck`] IR produced by `slideforge-eval`. Use this pass for structural
///    checks: zero-slide count, missing `lang` attribute, overflow estimates
///    from field values.
///
/// 2. **Stage 6b (post-layout):** [`Validator::validate_post_layout`] receives
///    the geometric [`LaidOutDeck`] IR produced by `slideforge-layout`. Use this
///    pass for checks that require `FrameContent` data — particularly alt-text
///    presence on visual elements (`Chart`, `Image`, `Diagram` frames) that are
///    only populated after layout.
///
/// The default implementation of `validate_post_layout` is a no-op. Validators
/// that only need semantic-IR data do not need to override it.
///
/// Diagnostics from both passes are accumulated into a single list and evaluated
/// by the strict-mode gate exactly once (after Stage 6b completes).
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
    /// Called at **Stage 5 (pre-layout)** on the semantic [`Deck`] IR.
    /// Returning an empty `Vec` means the deck is clean for this validator.
    fn validate(&self, deck: &Deck, opts: &ValidatorOptions) -> Vec<Diagnostic>;

    /// Validate `laid_out` after the layout pass and return all diagnostics found.
    ///
    /// Called at **Stage 6b (post-layout)** on the geometric [`LaidOutDeck`] IR.
    /// This method is invoked AFTER [`Validator::validate`] and AFTER
    /// `layout::run` has produced a [`LaidOutDeck`]. It provides access to
    /// `FrameContent` variants (`Chart`, `Image`, `Diagram`) that are only
    /// present in the geometric IR, not in the semantic [`Deck`] passed to
    /// [`Validator::validate`].
    ///
    /// ## Contract
    ///
    /// - Called once per `build()` invocation, after `layout::run` succeeds.
    /// - Returns diagnostics with the same [`Diagnostic`] type as [`Validator::validate`].
    /// - An empty `Vec` means the laid-out deck is clean for this validator.
    /// - In strict mode, any `Error`-severity diagnostic returned from this method
    ///   causes `BuildError::ValidationFailed` (identical treatment to `validate()`).
    /// - This method is called on every registered `Validator`, regardless of
    ///   whether that validator also overrides `validate()`.
    ///
    /// ## Default implementation
    ///
    /// The default implementation is a no-op (returns an empty `Vec`).
    /// Validators that only need semantic-IR data (e.g., `ZeroSlideValidator`,
    /// `LangValidator`) do not need to override this method.
    ///
    /// Validators that check `ContentBlock`-level attributes — particularly
    /// accessibility validators checking alt-text presence on visual elements —
    /// MUST override this method. This is the only pass where `FrameContent::Chart`,
    /// `FrameContent::Image`, and `FrameContent::Diagram` are available.
    ///
    /// ## Additive-defaulted extension (ADR-018 / BC-5.02.001 invariant 2)
    ///
    /// This is a defaulted method. Existing `Validator` implementations compile
    /// unchanged and inherit the default no-op. No existing implementor is
    /// required to add code.
    fn validate_post_layout(
        &self,
        laid_out: &LaidOutDeck,
        opts: &ValidatorOptions,
    ) -> Vec<Diagnostic> {
        let _ = (laid_out, opts); // default no-op — see ADR-018 Decision 2
        vec![]
    }
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
}
