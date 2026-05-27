//! Evaluator configuration for the slideforge DSL.
//!
//! [`EvalConfig`] is passed to top-level evaluation functions such as
//! [`crate::for_eval::eval_for_block`] and [`crate::eval::eval_deck`].
//! It controls thresholds and limits that govern evaluation behaviour
//! without changing observable correctness — only performance and
//! diagnostic verbosity are affected.

// ─── EvalConfig ──────────────────────────────────────────────────────────────

/// Configuration for the slideforge evaluator.
///
/// All fields have sensible defaults via [`Default`]. Callers that are happy
/// with the defaults should use `EvalConfig::default()`.
///
/// # Example
///
/// ```
/// use slideforge_eval::EvalConfig;
///
/// let cfg = EvalConfig::default();
/// assert_eq!(cfg.large_deck_warn_threshold, 500);
/// assert_eq!(cfg.max_total_slides, None);
/// ```
#[derive(Debug, Clone)]
pub struct EvalConfig {
    /// Emit a lint [`ParseSeverity::Warning`](slideforge_syntax::ParseSeverity)
    /// into the [`DiagnosticSink`](slideforge_syntax::DiagnosticSink) when an
    /// `@for` loop generates more slides than this threshold.
    ///
    /// Default: `500`.
    ///
    /// This is a *warning*, not an error — evaluation continues normally.
    /// Set to `usize::MAX` to disable the warning entirely.
    pub large_deck_warn_threshold: usize,

    /// Hard cap on the total number of slides that may be produced by a single
    /// [`eval_deck`](crate::eval::eval_deck) call.
    ///
    /// `None` (the default) means no cap is enforced. When `Some(n)` is set,
    /// evaluation stops adding slides once `n` slides have been accumulated and
    /// pushes an [`EvalError::TooManySlides`](crate::EvalError) error into the
    /// sink.
    ///
    /// Default: `None`.
    pub max_total_slides: Option<usize>,
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self {
            large_deck_warn_threshold: 500,
            max_total_slides: None,
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::doc_markdown)]
mod tests {
    use super::*;

    /// BC-2.12.001: default config must have `large_deck_warn_threshold` == 500.
    #[test]
    fn test_bc_2_12_001_default_large_deck_warn_threshold() {
        let cfg = EvalConfig::default();
        assert_eq!(
            cfg.large_deck_warn_threshold, 500,
            "default large_deck_warn_threshold must be 500"
        );
    }

    /// BC-2.12.001: default config must have `max_total_slides` == None.
    #[test]
    fn test_bc_2_12_001_default_max_total_slides_is_none() {
        let cfg = EvalConfig::default();
        assert!(
            cfg.max_total_slides.is_none(),
            "default max_total_slides must be None"
        );
    }

    /// BC-2.12.001: `EvalConfig` is Clone and Debug.
    #[test]
    fn test_bc_2_12_001_eval_config_clone_debug() {
        let cfg = EvalConfig {
            large_deck_warn_threshold: 100,
            max_total_slides: Some(50),
        };
        let cfg2 = cfg.clone();
        assert_eq!(cfg2.large_deck_warn_threshold, 100);
        assert_eq!(cfg2.max_total_slides, Some(50));
        let _ = format!("{cfg:?}");
    }

    /// BC-2.12.001: custom threshold values are stored correctly.
    #[test]
    fn test_bc_2_12_001_custom_threshold() {
        let cfg = EvalConfig {
            large_deck_warn_threshold: 1000,
            max_total_slides: Some(200),
        };
        assert_eq!(cfg.large_deck_warn_threshold, 1000);
        assert_eq!(cfg.max_total_slides, Some(200));
    }
}
