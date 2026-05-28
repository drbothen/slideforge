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
/// A `Value` is considered empty (triggering E-LAY-003) when it is:
/// - [`Value::List`] with zero elements (top-level only — not checked recursively)
/// - [`Value::Map`] with zero entries (empty mapping has no chart rows)
///
/// `Value::Null`, `Value::Str`, `Value::Int`, `Value::Bool` are NOT flagged
/// as empty by this guard; those type mismatches are caught by the eval-stage
/// type checker (E-EVL-006) before the chart renderer is reached.
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
pub fn data_is_empty(data: &Value) -> bool {
    match data {
        Value::List(items) => items.is_empty(),
        Value::Map(m) => m.is_empty(),
        _ => false,
    }
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
///   The expression is passed through as a function argument for callers that need it,
///   but is NOT inlined into the message body (miette renders the span pointer separately).
/// * `span` — Source location of the `data` binding line in the `.sf` file.
///
/// # Returns
///
/// A [`Diagnostic`] with:
/// - `code`: `E-LAY-003`
/// - `severity`: [`DiagnosticSeverity::Error`]
/// - `message`: `"Chart data is empty for slide '<slide_title>'. Rendering error-slide placeholder."`
/// - `hint`: `"Ensure the data source contains at least one row."`
/// - `span`: the provided `span` (miette renders the expression location as a source pointer)
#[must_use]
pub fn build_empty_data_diagnostic(
    slide_title: &str,
    _expression: &str,
    span: SourceSpan,
) -> Diagnostic {
    Diagnostic {
        severity: slideforge_plugin_api::DiagnosticSeverity::Error,
        code: std::sync::Arc::from(E_LAY_003),
        // HIGH-004: canonical message format — expression NOT inlined; miette renders the
        // span source pointer which points at the data binding expression directly.
        message: std::sync::Arc::from(format!(
            "Chart data is empty for slide '{slide_title}'. Rendering error-slide placeholder."
        )),
        span,
        hint: Some(std::sync::Arc::from(
            "Ensure the data source contains at least one row.",
        )),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use slideforge_plugin_api::DiagnosticSeverity;
    use slideforge_types::{SourceSpan, Value};

    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // data_is_empty — BC-1.11.002 precondition helper (MED-004: narrower anchor)
    //
    // MED-004: section anchor updated from "AC-002 / postcondition 1 / invariant 2"
    // to "precondition helper". `data_is_empty` is a PRECONDITION guard — it does
    // not verify postconditions; it enables them by preventing the renderer call
    // when data is absent. Postcondition-level assertions are in the lib.rs tests.
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-1.11.002 precondition helper — `Value::List([])` is recognized as empty.
    #[test]
    fn test_bc_1_11_002_data_is_empty_for_empty_list() {
        assert!(
            data_is_empty(&Value::List(vec![])),
            "Value::List([]) must be recognized as empty data"
        );
    }

    /// BC-1.11.002 EC-003 — Single-element list (exactly 1 row) is not empty.
    ///
    /// The spec EC-003 states: data with exactly 1 row → non-empty, chart renders normally.
    ///
    /// Red Gate: fails until `data_is_empty` is implemented.
    #[test]
    fn test_bc_1_11_002_data_is_not_empty_for_single_element_list() {
        let row = Value::Str(Arc::from("row1"));
        assert!(
            !data_is_empty(&Value::List(vec![row])),
            "Value::List with 1 element must NOT be empty (EC-003: exactly 1 row renders normally)"
        );
    }

    /// BC-1.11.002 — Multi-element list is not empty.
    ///
    /// Red Gate: fails until `data_is_empty` is implemented.
    #[test]
    fn test_bc_1_11_002_data_is_not_empty_for_multi_element_list() {
        let items = vec![Value::Int(10), Value::Int(20), Value::Int(30)];
        assert!(
            !data_is_empty(&Value::List(items)),
            "Value::List with 3 elements must NOT be recognized as empty"
        );
    }

    /// BC-1.11.002 — Non-list values (`Value::Int`, `Value::Null`, `Value::Str("")`)
    /// are NOT flagged as empty by the chart guard.
    ///
    /// The empty-data guard only checks `Value::List`. Non-list bindings are
    /// caught earlier by the eval-stage type checker (E-EVL-006). The chart guard
    /// must not intercept them.
    ///
    /// Red Gate: fails until `data_is_empty` is implemented.
    #[test]
    fn test_bc_1_11_002_data_is_not_empty_for_non_list_value() {
        assert!(
            !data_is_empty(&Value::Int(0)),
            "Value::Int(0) must not be flagged as empty by the chart guard"
        );
        assert!(
            !data_is_empty(&Value::Null),
            "Value::Null must not be flagged as empty by the chart guard"
        );
        assert!(
            !data_is_empty(&Value::Str(Arc::from(""))),
            "Value::Str(\"\") must not be flagged as empty by the chart guard"
        );
        assert!(
            !data_is_empty(&Value::Bool(false)),
            "Value::Bool(false) must not be flagged as empty by the chart guard"
        );
    }

    /// BC-1.11.002 — Outer non-empty list containing an inner empty list is NOT empty.
    ///
    /// Only top-level `Value::List([])` triggers the empty-data check. A list
    /// with one element (even if that element is itself an empty list) is
    /// considered non-empty at this stage — the outer list has 1 element.
    ///
    /// MED-001: renamed from `test_bc_1_11_002_data_is_empty_for_nested_empty_list`
    /// (the old name was misleading — it tests the not-empty case for an outer list).
    #[test]
    fn test_bc_1_11_002_data_is_not_empty_for_outer_list_with_inner_empty_list() {
        // Outer list has one element (inner empty list) → outer is NOT empty.
        let nested = Value::List(vec![Value::List(vec![])]);
        assert!(
            !data_is_empty(&nested),
            "Value::List([Value::List([])]) — outer list has 1 element so must NOT be empty; \
             only a top-level empty List triggers E-LAY-003"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HIGH-003 — data_is_empty extended cases: Value::Map
    // ─────────────────────────────────────────────────────────────────────────

    /// HIGH-003 — `Value::Map({})` (empty map) is recognized as empty.
    ///
    /// An empty map binding has no chart rows. The guard must catch this at
    /// the same stage as an empty list.
    #[test]
    fn test_bc_1_11_002_data_is_empty_for_empty_map() {
        use slideforge_types::OrderedMap;
        let empty_map: OrderedMap<Arc<str>, Value> = OrderedMap::new();
        assert!(
            data_is_empty(&Value::Map(empty_map)),
            "Value::Map({{}}) must be recognized as empty data by the chart guard"
        );
    }

    /// HIGH-003 — Non-empty `Value::Map` is NOT empty.
    ///
    /// A map with at least one entry has chart rows; guard must pass it through.
    #[test]
    fn test_bc_1_11_002_data_is_not_empty_for_nonempty_map() {
        use slideforge_types::OrderedMap;
        let mut map: OrderedMap<Arc<str>, Value> = OrderedMap::new();
        map.insert(Arc::from("revenue"), Value::Int(100));
        assert!(
            !data_is_empty(&Value::Map(map)),
            "Value::Map with 1 entry must NOT be recognized as empty"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // build_empty_data_diagnostic — BC-1.11.002 AC-001 / postcondition 1
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-1.11.002 AC-001 — E-LAY-003 is emitted with the correct error code.
    ///
    /// Red Gate: fails until `build_empty_data_diagnostic` is implemented.
    #[test]
    fn test_bc_1_11_002_diagnostic_uses_e_lay_003_code() {
        let span = SourceSpan::default();
        let diag = build_empty_data_diagnostic("My Slide", "{{ kpis.monthly }}", span);
        assert_eq!(
            diag.code.as_ref(),
            E_LAY_003,
            "diagnostic code must be E-LAY-003"
        );
    }

    /// BC-1.11.002 AC-001 / AC-003 — Diagnostic is `Error` severity by default.
    ///
    /// `build_empty_data_diagnostic` always returns `Error` severity because the
    /// pipeline dispatcher (not this function) downgrades to `Warning` for
    /// warn-only mode. AC-003 strict mode causes exit code 2 by the `DiagnosticSink`
    /// seeing an `Error`-severity diagnostic.
    ///
    /// MED-002: renamed from `test_bc_1_11_002_diagnostic_severity_depends_on_mode`
    /// (old name implied mode-awareness in the function, but the function is stateless —
    /// it always returns Error and the dispatcher handles downgrade).
    #[test]
    fn test_bc_1_11_002_diagnostic_is_error_severity_by_default() {
        // Strict mode (default): the diagnostic returned must be Error severity.
        // The caller (pipeline dispatcher) is responsible for warn-only downgrade.
        let span = SourceSpan::default();
        let diag = build_empty_data_diagnostic("KPI Dashboard", "{{ kpis.monthly }}", span);
        assert_eq!(
            diag.severity,
            DiagnosticSeverity::Error,
            "E-LAY-003 must be Error severity so that strict mode can accumulate it as a build failure"
        );
    }

    /// BC-1.11.002 AC-001 — Message contains the slide title.
    ///
    /// Per AC-001 the message format is:
    /// `"Chart data is empty for slide '<slide_title>'. Rendering error-slide placeholder."`
    ///
    /// Red Gate: fails until `build_empty_data_diagnostic` is implemented.
    #[test]
    fn test_bc_1_11_002_diagnostic_message_contains_slide_title() {
        let span = SourceSpan::default();
        let diag = build_empty_data_diagnostic("Q3 Performance", "{{ revenue }}", span);
        assert!(
            diag.message.contains("Q3 Performance"),
            "diagnostic message must contain the slide title 'Q3 Performance'; got: {}",
            diag.message
        );
    }

    /// BC-1.11.002 AC-001 / HIGH-004 — Diagnostic uses canonical message format.
    ///
    /// The canonical message is:
    /// `"Chart data is empty for slide '<title>'. Rendering error-slide placeholder."`
    ///
    /// The expression is NOT inlined in the message body (miette renders the span
    /// source pointer separately). This test verifies the canonical format.
    ///
    /// Updated by HIGH-004 adversarial finding (three sources of truth reconciled).
    #[test]
    fn test_bc_1_11_002_diagnostic_message_contains_data_expression() {
        let span = SourceSpan::default();
        let expr = "{{ kpis.monthly }}";
        let diag = build_empty_data_diagnostic("Revenue Overview", expr, span);
        // HIGH-004: canonical message does NOT inline the expression — verify canonical form.
        assert!(
            diag.message.contains("Revenue Overview"),
            "diagnostic message must contain the slide title; got: {}",
            diag.message
        );
        assert!(
            diag.message.contains("Rendering error-slide placeholder"),
            "diagnostic message must contain canonical suffix; got: {}",
            diag.message
        );
        // Verify expression is NOT in message body (it belongs in the span/field).
        assert!(
            !diag.message.contains("expression:"),
            "canonical message must NOT inline expression: prefix; got: {}",
            diag.message
        );
    }

    /// BC-1.11.002 AC-001 — Hint text is present and contains remediation guidance.
    ///
    /// Per AC-001, the hint is: `"Ensure the data source contains at least one row."`
    ///
    /// Red Gate: fails until `build_empty_data_diagnostic` is implemented.
    #[test]
    fn test_bc_1_11_002_diagnostic_hint_includes_remediation() {
        let span = SourceSpan::default();
        let diag = build_empty_data_diagnostic("My Slide", "{{ data }}", span);
        let hint = diag.hint.as_ref().expect(
            "E-LAY-003 diagnostic must include a correction hint (hint field must be Some)",
        );
        assert!(
            hint.contains("data source") || hint.contains("at least one row"),
            "hint must suggest remediation for empty data; got: {hint}"
        );
    }

    /// BC-1.11.002 AC-001 — Span from the call site is preserved in the diagnostic.
    ///
    /// Red Gate: fails until `build_empty_data_diagnostic` is implemented.
    #[test]
    fn test_bc_1_11_002_diagnostic_preserves_span() {
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
