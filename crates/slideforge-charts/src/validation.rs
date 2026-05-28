//! Empty-data guard for the chart rendering pipeline (STORY-032).
//!
//! This module provides the precondition check that must run **before**
//! [`crate::ChartRendererImpl::dispatch_and_process`] is called. If chart data
//! is empty, the guard emits `E-LAY-003` and the caller decides whether to
//! abort (strict mode) or produce an [`FrameContent::ErrorSlidePlaceholder`]
//! (warn-only mode).
//!
//! ## Architecture contract (BC-1.11.002 invariant 2)
//!
//! `ChartRendererImpl::dispatch_and_process` is **never** called with empty data.
//! The empty-data check in this module runs at the eval-layer integration point,
//! before the renderer is invoked.
//!
//! ## Error code
//!
//! | Code | Severity | Meaning |
//! |------|----------|---------|
//! | `E-LAY-003` | Error | Chart data is empty; `ChartRenderer::render` would receive an empty series list |

use slideforge_plugin_api::Diagnostic;
use slideforge_types::{SourceSpan, Value};

/// The error code emitted when chart data is empty.
///
/// This code is in the `E-LAY-*` class because the empty-data condition is
/// detected at the layout / eval-pipeline stage, before the chart renderer
/// plugin is invoked.
pub const E_LAY_003: &str = "E-LAY-003";

/// Return `true` if `data` represents an empty collection.
///
/// A `Value` is considered empty when it is:
/// - [`Value::List`] with zero elements, or
/// - [`Value::List`] whose elements are all empty (not checked here — only
///   top-level emptiness is detected at this stage).
///
/// Any non-`List` value (e.g., `Value::Null`, `Value::Str`, `Value::Map`) is
/// considered non-empty from the chart guard's perspective; those cases are
/// caught earlier by the eval-stage type checker (E-EVL-006).
///
/// # Examples
///
/// ```rust,ignore
/// use slideforge_charts::validation::data_is_empty;
/// use slideforge_types::Value;
///
/// assert!(data_is_empty(&Value::List(vec![])));
/// assert!(!data_is_empty(&Value::List(vec![Value::Int(1)])));
/// ```
#[must_use]
pub fn data_is_empty(_data: &Value) -> bool {
    todo!()
}

/// Build an `E-LAY-003` [`Diagnostic`] for an empty-data chart slide.
///
/// The diagnostic is `Error` severity — in strict mode it aborts export; in
/// warn-only mode it is downgraded by the pipeline dispatcher.
///
/// # Arguments
///
/// * `slide_title` — The title of the chart slide (for the human-readable message).
/// * `expression` — The data binding expression from the DSL (e.g., `{{ kpis.monthly }}`).
/// * `span` — Source location of the `data` binding line in the `.sf` file.
///
/// # Returns
///
/// A [`Diagnostic`] with:
/// - `code`: `E-LAY-003`
/// - `severity`: [`DiagnosticSeverity::Error`]
/// - `message`: `"Chart data is empty for slide '<slide_title>'. Rendering error-slide placeholder."`
/// - `hint`: `"Ensure the data source contains at least one row."`
/// - `span`: the provided `span`
#[must_use]
pub fn build_empty_data_diagnostic(
    _slide_title: &str,
    _expression: &str,
    _span: SourceSpan,
) -> Diagnostic {
    todo!()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use slideforge_plugin_api::DiagnosticSeverity;
    use slideforge_types::{SourceSpan, Value};

    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // data_is_empty — BC-1.11.002 postcondition 1 + invariant 2
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-1.11.002 — `Value::List([])` is recognized as empty.
    ///
    /// Red Gate: fails until `data_is_empty` is implemented.
    #[test]
    fn test_bc_1_11_002_empty_list_is_empty() {
        assert!(
            data_is_empty(&Value::List(vec![])),
            "Value::List([]) must be recognized as empty data"
        );
    }

    /// BC-1.11.002 — `Value::List` with one element is not empty.
    ///
    /// Red Gate: fails until `data_is_empty` is implemented.
    #[test]
    fn test_bc_1_11_002_nonempty_list_is_not_empty() {
        assert!(
            !data_is_empty(&Value::List(vec![Value::Int(1)])),
            "Value::List with elements must NOT be recognized as empty"
        );
    }

    /// BC-1.11.002 EC-003 — Single-element list (exactly 1 row) is not empty.
    ///
    /// Red Gate: fails until `data_is_empty` is implemented.
    #[test]
    fn test_bc_1_11_002_single_element_list_is_not_empty() {
        let row = Value::Str(Arc::from("row1"));
        assert!(
            !data_is_empty(&Value::List(vec![row])),
            "Value::List with 1 element must NOT be empty (EC-003: exactly 1 row renders normally)"
        );
    }

    /// BC-1.11.002 — Non-list Value (Str) is not empty from the guard's perspective.
    ///
    /// Red Gate: fails until `data_is_empty` is implemented.
    #[test]
    fn test_bc_1_11_002_non_list_value_is_not_empty() {
        // Non-list values are caught earlier by E-EVL-006; the chart guard
        // does not treat them as empty.
        assert!(
            !data_is_empty(&Value::Str(Arc::from("irrelevant"))),
            "Value::Str must not be flagged as empty by the chart guard"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // build_empty_data_diagnostic — BC-1.11.002 postcondition 1
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-1.11.002 AC-001 — E-LAY-003 is emitted with the correct error code.
    ///
    /// Red Gate: fails until `build_empty_data_diagnostic` is implemented.
    #[test]
    fn test_bc_1_11_002_diagnostic_has_correct_code() {
        let span = SourceSpan::default();
        let diag = build_empty_data_diagnostic("My Slide", "{{ kpis.monthly }}", span);
        assert_eq!(
            diag.code.as_ref(),
            E_LAY_003,
            "diagnostic code must be E-LAY-003"
        );
    }

    /// BC-1.11.002 AC-001 — Diagnostic is Error severity in strict mode.
    ///
    /// Red Gate: fails until `build_empty_data_diagnostic` is implemented.
    #[test]
    fn test_bc_1_11_002_diagnostic_is_error_severity() {
        let span = SourceSpan::default();
        let diag = build_empty_data_diagnostic("KPI Dashboard", "{{ kpis.monthly }}", span);
        assert_eq!(
            diag.severity,
            DiagnosticSeverity::Error,
            "E-LAY-003 must be Error severity"
        );
    }

    /// BC-1.11.002 AC-001 — Message contains slide title.
    ///
    /// Red Gate: fails until `build_empty_data_diagnostic` is implemented.
    #[test]
    fn test_bc_1_11_002_diagnostic_message_contains_slide_title() {
        let span = SourceSpan::default();
        let diag = build_empty_data_diagnostic("Revenue Overview", "{{ revenue }}", span);
        assert!(
            diag.message.contains("Revenue Overview"),
            "diagnostic message must contain the slide title; got: {}",
            diag.message
        );
    }

    /// BC-1.11.002 AC-001 — Hint text is present.
    ///
    /// Red Gate: fails until `build_empty_data_diagnostic` is implemented.
    #[test]
    fn test_bc_1_11_002_diagnostic_has_hint() {
        let span = SourceSpan::default();
        let diag = build_empty_data_diagnostic("My Slide", "{{ data }}", span);
        assert!(
            diag.hint.is_some(),
            "E-LAY-003 diagnostic must include a correction hint"
        );
    }

    /// BC-1.11.002 AC-001 — Span is preserved in the diagnostic.
    ///
    /// Red Gate: fails until `build_empty_data_diagnostic` is implemented.
    #[test]
    fn test_bc_1_11_002_diagnostic_span_preserved() {
        let span = SourceSpan {
            file: Arc::from("deck.sf"),
            line: 12,
            col: 3,
            byte_offset: 100,
        };
        let diag = build_empty_data_diagnostic("My Slide", "{{ kpis }}", span.clone());
        assert_eq!(
            diag.span, span,
            "diagnostic span must match the provided source location"
        );
    }
}
