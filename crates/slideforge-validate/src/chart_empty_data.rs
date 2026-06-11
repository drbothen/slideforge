//! Chart empty-data validator — intercepts `E-LAY-003` before `ChartRenderer` is called.
//!
//! [`ChartEmptyDataValidator`] is a Stage-5 (pre-layout) [`Validator`] that detects
//! chart slides whose data binding has evaluated to an empty collection and emits
//! `E-LAY-003` before the `ChartRenderer` plugin is invoked.
//!
//! ## Architecture (BC-1.11.002 invariant 2)
//!
//! BC-1.11.002 invariant 2: "The ChartRenderer plugin is never called with empty
//! data — the validator intercepts before plugin invocation." This validator is the
//! interception point. It inspects each `chart` slide's `data` field value. If the
//! value is a `Value::List([])` or `Value::Map({})` (empty collection), it emits
//! `E-LAY-003` and the pipeline gate (in strict mode) aborts before export.
//!
//! ## STORY-098: Stub — not yet implemented
//!
//! This struct is a compilable stub. `validate()` currently returns an empty `Vec`
//! (no diagnostics). The tests in the test module will FAIL assertions because they
//! expect `E-LAY-003` to be emitted. This is the Red Gate for AC-004 and AC-005.
//!
//! The implementer must:
//!   1. Implement `validate()` to inspect `chart` slides for empty `data` field values.
//!   2. Emit `E-LAY-003` with `DiagnosticSeverity::Error` when `data` is empty.
//!   3. Add `pub use chart_empty_data::ChartEmptyDataValidator;` to `lib.rs`.
//!   4. Register with the pipeline (wire into `build_inner`).
//!
//! ## Traceability
//!
//! - BC-1.11.002 (chart empty data → error-slide placeholder)
//! - BC-3.03.002 v1.2 (strict mode exits non-zero on validation error)
//! - STORY-098 AC-004, AC-005

use slideforge_plugin_api::{Diagnostic, Validator, ValidatorOptions};
use slideforge_types::Deck;

/// Stage-5 validator that intercepts empty chart data before `ChartRenderer` is called.
///
/// Emits `E-LAY-003` with `Error` severity for any `chart` slide whose evaluated
/// `data` field is an empty collection (`Value::List([])` or `Value::Map({})`).
///
/// ## STORY-098: Stub — not yet implemented
///
/// This implementation is a compilable stub that returns no diagnostics. Tests
/// asserting E-LAY-003 emission will FAIL at assertion time (Red Gate for AC-004/005).
pub struct ChartEmptyDataValidator;

impl Validator for ChartEmptyDataValidator {
    fn id(&self) -> &'static str {
        "chart-empty-data"
    }

    /// Validate all `chart` slides for empty data binding.
    ///
    /// ## STORY-098: Stub — not yet implemented
    ///
    /// Currently returns `vec![]` (no diagnostics). This causes test assertions for
    /// E-LAY-003 to fail — which is the Red Gate for AC-004 and AC-005.
    ///
    /// The implementer must replace this stub with a production implementation that:
    /// 1. Iterates over `deck.slides`.
    /// 2. For each slide with `slide_type == "chart"`, reads the `data` field value.
    /// 3. If the value is `FieldValue::Literal(Value::List([]))` or
    ///    `FieldValue::Literal(Value::Map({}))`, emits `E-LAY-003` with Error severity.
    /// 4. Uses `slideforge_charts::validation::build_empty_data_diagnostic` to construct
    ///    the canonical `E-LAY-003` diagnostic (canonical message format, hint text, span).
    fn validate(&self, _deck: &Deck, _opts: &ValidatorOptions) -> Vec<Diagnostic> {
        // STORY-098 STUB: returns no diagnostics — Red Gate for AC-004/AC-005.
        // Tests asserting E-LAY-003 will FAIL at assertion time until implemented.
        vec![]
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)] // BC-traceability IDs use uppercase: test_BC_S_SS_NNN_xxx
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use slideforge_plugin_api::{Diagnostic, DiagnosticSeverity, Validator, ValidatorOptions};
    use slideforge_types::{Deck, DeckMetadata, FieldValue, OrderedMap, Slide, SourceSpan, Value};

    use super::ChartEmptyDataValidator;

    // ── Test helpers ──────────────────────────────────────────────────────────

    fn make_metadata() -> DeckMetadata {
        DeckMetadata {
            title: Some(Arc::from("Test Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        }
    }

    fn make_deck(slides: Vec<Slide>) -> Deck {
        Deck {
            slides,
            vars: OrderedMap::new(),
            metadata: make_metadata(),
            registers: OrderedMap::new(),
            section_blocks: vec![],
            slide_sections: vec![],
        }
    }

    fn make_chart_slide_with_data(title: &str, data: Value) -> Slide {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from(title))),
        );
        fields.insert(
            Arc::from("chart_type"),
            FieldValue::Literal(Value::Str(Arc::from("bar"))),
        );
        fields.insert(Arc::from("data"), FieldValue::Literal(data));
        Slide {
            slide_type: Arc::from("chart"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
            field_spans: OrderedMap::new(),
        }
    }

    fn make_chart_slide_empty_data(title: &str) -> Slide {
        make_chart_slide_with_data(title, Value::List(vec![]))
    }

    fn make_chart_slide_nonempty_data(title: &str) -> Slide {
        // EC-003: exactly 1 data row → non-empty → chart renders normally.
        let row = Value::Str(Arc::from("Q1: 100"));
        make_chart_slide_with_data(title, Value::List(vec![row]))
    }

    fn default_opts() -> ValidatorOptions {
        ValidatorOptions::default()
    }

    // ── AC-004: chart with empty data → E-LAY-003, Error severity (strict mode) ─

    /// BC-1.11.002 postcondition 2 / AC-004 (T-005 RED):
    ///
    /// A `chart` slide whose `data` field evaluates to `Value::List([])` (empty collection)
    /// must emit `E-LAY-003` with `DiagnosticSeverity::Error` severity.
    ///
    /// In strict mode, Error severity causes the pipeline gate to exit 2, no output.
    ///
    /// RED: `ChartEmptyDataValidator.validate()` is a stub returning `vec![]`. This
    /// test FAILS at the first assertion ("must emit E-LAY-003; got diags: []").
    ///
    /// FU-DIAGNOSTIC-FIELD-PINNING: assert message text, code, severity, hint presence.
    #[test]
    fn test_BC_1_11_002_chart_empty_data_strict_exits_2() {
        let slide = make_chart_slide_empty_data("Revenue Chart");
        let deck = make_deck(vec![slide]);
        let diags = ChartEmptyDataValidator.validate(&deck, &default_opts());

        // Must emit exactly one E-LAY-003 diagnostic.
        // RED: stub returns vec![] — this assertion FAILS.
        let e_lay_003: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == "E-LAY-003")
            .collect();
        assert!(
            !e_lay_003.is_empty(),
            "BC-1.11.002 AC-004: chart with empty data must emit E-LAY-003; \
             STUB currently returns vec![] — RED GATE: this assertion fails before implementation. \
             got diags: {diags:?}"
        );
        assert_eq!(
            e_lay_003.len(),
            1,
            "BC-1.11.002 AC-004: exactly 1 E-LAY-003 for a single empty-data chart; \
             got {} — all diags: {diags:?}",
            e_lay_003.len()
        );

        let diag = e_lay_003[0];

        // FU-DIAGNOSTIC-FIELD-PINNING: severity must be Error (pipeline gate uses this to exit 2).
        assert_eq!(
            diag.severity,
            DiagnosticSeverity::Error,
            "BC-1.11.002 postcondition 2: E-LAY-003 must be Error severity so strict mode \
             can gate on it and exit 2; got: {:?}",
            diag.severity
        );

        // FU-DIAGNOSTIC-FIELD-PINNING: canonical message format (BC-1.11.002 postcondition 1).
        // "Chart data is empty for slide '<title>'. Rendering error-slide placeholder."
        assert!(
            diag.message.contains("Revenue Chart"),
            "BC-1.11.002 postcondition 5: E-LAY-003 message must name the slide title; \
             got: {}",
            diag.message
        );
        assert!(
            diag.message.contains("Rendering error-slide placeholder"),
            "BC-1.11.002 postcondition 1: E-LAY-003 message must contain canonical suffix; \
             got: {}",
            diag.message
        );

        // FU-DIAGNOSTIC-FIELD-PINNING: code pinned to "E-LAY-003".
        assert_eq!(
            diag.code.as_ref(),
            "E-LAY-003",
            "BC-1.11.002 postcondition 1: diagnostic code must be exactly 'E-LAY-003'; \
             got: {}",
            diag.code
        );

        // Hint must be present with remediation guidance.
        assert!(
            diag.hint.is_some(),
            "BC-1.11.002 postcondition 5: E-LAY-003 must carry a remediation hint; \
             got hint: {:?}",
            diag.hint
        );
    }

    /// BC-1.11.002 postcondition 3 / AC-005 (T-006 RED):
    ///
    /// In warn-only mode, E-LAY-003 is still emitted by the validator (always fires
    /// regardless of mode). The pipeline gate (not the validator) decides whether to
    /// block on Error severity. This test verifies the diagnostic IS emitted.
    ///
    /// In the pipeline: warn-only → gate skipped → exporter renders `ErrorSlidePlaceholder`.
    ///
    /// RED: stub returns vec![] — assertion "must emit E-LAY-003" FAILS.
    #[test]
    fn test_BC_1_11_002_chart_empty_data_warn_only_placeholder() {
        let slide = make_chart_slide_empty_data("Revenue Chart");
        let deck = make_deck(vec![slide]);

        // The validator is mode-agnostic. It always emits E-LAY-003 Error severity.
        // The pipeline gate in warn-only skips the Error check; the exporter renders placeholder.
        let diags = ChartEmptyDataValidator.validate(&deck, &default_opts());

        // RED: stub returns vec![] — this assertion FAILS.
        let e_lay_003: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code.as_ref() == "E-LAY-003")
            .collect();
        assert!(
            !e_lay_003.is_empty(),
            "BC-1.11.002 AC-005: E-LAY-003 must be emitted for empty-data chart \
             (validator fires regardless of warn-only mode; gate handles demotion); \
             STUB currently returns vec![] — RED GATE. got diags: {diags:?}"
        );
        // Validator always emits Error — the pipeline gate converts to warn in warn-only.
        assert_eq!(
            e_lay_003[0].severity,
            DiagnosticSeverity::Error,
            "BC-1.11.002 AC-005: ChartEmptyDataValidator always emits Error severity; \
             warn-only demotion is the pipeline gate's responsibility; got: {:?}",
            e_lay_003[0].severity
        );
    }

    /// BC-1.11.002 EC-003 guard (non-empty chart data → no E-LAY-003):
    ///
    /// A chart slide with non-empty data must NOT produce E-LAY-003.
    /// Guard that may PASS at Red Gate (stub returns vec![] — no false positives).
    /// Must CONTINUE PASSING after implementation.
    #[test]
    fn test_BC_1_11_002_nonempty_chart_data_no_e_lay_003() {
        let slide = make_chart_slide_nonempty_data("Revenue Chart");
        let deck = make_deck(vec![slide]);
        let diags = ChartEmptyDataValidator.validate(&deck, &default_opts());

        let e_lay_003: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == "E-LAY-003")
            .collect();
        assert!(
            e_lay_003.is_empty(),
            "EC-003 guard: chart with non-empty data must NOT produce E-LAY-003; \
             got: {e_lay_003:?}"
        );
    }

    /// BC-1.11.002 EC-004 (multiple chart slides, one empty, one non-empty):
    ///
    /// Only the empty-data chart emits E-LAY-003 (DI-018: accumulation, all slides checked).
    ///
    /// RED: stub returns vec![] — assertion "only empty chart emits E-LAY-003" FAILS
    /// on the count == 1 check (got 0).
    #[test]
    fn test_BC_1_11_002_only_empty_data_chart_emits_e_lay_003() {
        let empty_chart = make_chart_slide_empty_data("Empty Chart Slide");
        let nonempty_chart = make_chart_slide_nonempty_data("Normal Chart Slide");
        let deck = make_deck(vec![empty_chart, nonempty_chart]);
        let diags = ChartEmptyDataValidator.validate(&deck, &default_opts());

        // RED: stub returns vec![] — count == 0 != 1 → FAILS.
        let e_lay_003: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == "E-LAY-003")
            .collect();
        assert_eq!(
            e_lay_003.len(),
            1,
            "BC-1.11.002 EC-004: only the empty-data chart slide emits E-LAY-003; \
             STUB returns vec![] — RED GATE. got {} — all diags: {diags:?}",
            e_lay_003.len()
        );

        // The diagnostic must name the empty slide title.
        assert!(
            e_lay_003[0].message.contains("Empty Chart Slide"),
            "E-LAY-003 must identify the empty-data slide by title; \
             got: {}",
            e_lay_003[0].message
        );
    }

    /// BC-1.11.002 invariant 2 — ChartRenderer NEVER called with empty data.
    ///
    /// The validator fires at Stage 5 (pre-layout). It emits E-LAY-003 with Error severity
    /// for empty data. In strict mode, the pipeline gate aborts BEFORE the export stage
    /// where ChartRenderer would be invoked.
    ///
    /// This test verifies the validator's side of invariant 2:
    ///   - Validator emits E-LAY-003 Error severity for empty data.
    ///   - Pipeline gate (strict mode) sees Error → exits 2 → ChartRenderer never called.
    ///
    /// Full end-to-end invariant 2 proof is in Phase 6 cargo-fuzz (BC-1.11.002 VP-TBD).
    ///
    /// RED: stub returns vec![] — assertion FAILS.
    #[test]
    fn test_BC_1_11_002_invariant_2_validator_prevents_renderer_call() {
        let slide = make_chart_slide_empty_data("KPI Dashboard");
        let deck = make_deck(vec![slide]);
        let diags = ChartEmptyDataValidator.validate(&deck, &default_opts());

        // RED: stub returns vec![] — assertion FAILS.
        let e_lay_003: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == "E-LAY-003")
            .collect();
        assert!(
            !e_lay_003.is_empty(),
            "BC-1.11.002 invariant 2: validator must emit E-LAY-003 for empty-data chart \
             to prevent ChartRenderer call via pipeline gate; \
             STUB currently returns vec![] — RED GATE. got diags: {diags:?}"
        );
        assert_eq!(
            e_lay_003[0].severity,
            DiagnosticSeverity::Error,
            "BC-1.11.002 invariant 2: E-LAY-003 must be Error severity so the strict-mode \
             gate aborts before ChartRenderer is invoked; got: {:?}",
            e_lay_003[0].severity
        );
    }
}
