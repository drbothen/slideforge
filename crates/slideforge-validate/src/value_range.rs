//! Numeric field value-range enforcement validator (F-087-P1-001).
//!
//! [`ValueRangeValidator`] checks every slide whose `slide_type` is one of the
//! numeric-range types for valid field values. Value-range violations produce
//! [`E_VAL_011`] diagnostics with [`DiagnosticSeverity::Error`] severity, which
//! causes [`BuildError::ValidationFailed`](slideforge_plugin_api::ValidatorOptions)
//! in strict mode.
//!
//! ## Enforcement model
//!
//! Per architect adjudication F-087-P1-001 (Option B), value-range validation is
//! performed at **Stage 5** (pre-layout, same as [`LabelCheckValidator`](crate::LabelCheckValidator))
//! by reading `Slide.fields` directly. `SlideType::lay_out()` is geometry-only and
//! does NOT perform value-range validation.
//!
//! ## Error codes
//!
//! | Code | Severity | Meaning |
//! |------|----------|---------|
//! | `E-VAL-011` | Error | Numeric field out of required range or wrong type |
//!
//! ## Traceability
//!
//! - BC-1.17.002 PC3: `progress_bar` `value` outside [0, 100] → E-VAL-011
//! - BC-1.17.003 inv 6: `weighted_composite` component `weight` ≤ 0 → E-VAL-011
//! - BC-1.17.003 inv 7: `weighted_composite` component `score` outside [0, 100] → E-VAL-011
//! - DI-018: all component errors accumulated before returning (no bail-on-first)
//! - ADR-016 Decision 3: Validator surface registered in `slideforge::registry`

use std::sync::Arc;

use slideforge_plugin_api::{Diagnostic, DiagnosticSeverity, Validator, ValidatorOptions};
use slideforge_types::{Deck, FieldValue, SourceSpan, Value};

/// Error code for a numeric field that is absent, wrong type, or out of range.
///
/// Traces to BC-1.17.002 PC3 and BC-1.17.003 inv 6/7.
/// Allocated by architect adjudication F-087-P1-001 (2026-06-06).
pub const E_VAL_011: &str = "E-VAL-011";

/// Validates numeric field ranges for slide types that have strict numeric contracts.
///
/// ## Slide types checked
///
/// - `progress_bar` — `Slide.fields["value"]` must be `Value::Int(n)` with
///   `0 ≤ n ≤ 100`. Absent or wrong-type fields also produce E-VAL-011.
/// - `weighted_composite` — for each `Value::Map` in `Slide.fields["components"]`:
///   - `weight` must be a positive `Value::Float` or positive `Value::Int`.
///   - `score` must be `Value::Int(n)` with `0 ≤ n ≤ 100`.
///   - All component errors are accumulated (DI-018 — no bail-on-first).
///
/// Non-color-coded slide types are not checked.
///
/// Register with [`slideforge_plugin_api::PluginRegistry::register_validator`].
pub struct ValueRangeValidator;

impl Validator for ValueRangeValidator {
    fn id(&self) -> &'static str {
        "value-range"
    }

    fn validate(&self, deck: &Deck, _opts: &ValidatorOptions) -> Vec<Diagnostic> {
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        for slide in &deck.slides {
            match slide.slide_type.as_ref() {
                "progress_bar" => {
                    validate_progress_bar_value(
                        &slide.fields,
                        &slide.source_span,
                        &mut diagnostics,
                    );
                },
                "weighted_composite" => {
                    validate_weighted_composite_components(
                        &slide.fields,
                        &slide.source_span,
                        &mut diagnostics,
                    );
                },
                // Other slide types: no numeric range validation.
                _ => {},
            }
        }

        diagnostics
    }
}

/// Validate the `value` field on a `progress_bar` slide.
///
/// BC-1.17.002 PC3: `value` must be `Value::Int(n)` with `0 ≤ n ≤ 100`.
/// Absent or wrong-type → E-VAL-011 (type error message).
/// Out-of-range → E-VAL-011 (range error message with actual value).
fn validate_progress_bar_value(
    fields: &slideforge_types::OrderedMap<Arc<str>, FieldValue>,
    span: &SourceSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match fields.get("value") {
        Some(FieldValue::Literal(Value::Int(n))) => {
            if !(0..=100).contains(n) {
                diagnostics.push(make_range_error(
                    &format!("progress_bar value must be between 0 and 100; got {n}."),
                    span,
                ));
            }
            // In-range: no diagnostic.
        },
        Some(FieldValue::Literal(other)) => {
            // Wrong type (not Int).
            let actual_type = value_type_name(other);
            diagnostics.push(make_range_error(
                &format!("progress_bar value must be an integer; got {actual_type} or absent."),
                span,
            ));
        },
        _ => {
            // Absent (None) or non-Literal FieldValue.
            diagnostics.push(make_range_error(
                "progress_bar value must be an integer; got absent or absent.",
                span,
            ));
        },
    }
}

/// Validate `weight` and `score` on each component of a `weighted_composite` slide.
///
/// BC-1.17.003 inv 6: `weight` must be positive (Float > 0.0 or Int > 0).
/// BC-1.17.003 inv 7: `score` must be `Int(n)` with `0 ≤ n ≤ 100`.
/// DI-018: all errors accumulated — no bail-on-first.
fn validate_weighted_composite_components(
    fields: &slideforge_types::OrderedMap<Arc<str>, FieldValue>,
    span: &SourceSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let components = match fields.get("components") {
        Some(FieldValue::Literal(Value::List(list))) if list.is_empty() => {
            // BC-1.17.003 PC-3 / architect pass-2 adjudication §7 (F-087-P2-001):
            // An empty list is a valid DSL value that violates the "non-empty components"
            // postcondition. Emit E-VAL-011 and return.
            diagnostics.push(make_range_error(
                "weighted_composite requires at least one component; got empty list.",
                span,
            ));
            return;
        },
        Some(FieldValue::Literal(Value::List(list))) => list.as_slice(),
        _ => {
            // Absent or wrong type — not a range error; no E-VAL-011.
            return;
        },
    };

    for (idx, comp_val) in components.iter().enumerate() {
        let Value::Map(comp_map) = comp_val else {
            // Non-map component — skip range checks for this entry.
            continue;
        };

        // Check weight: must be present and > 0.
        match comp_map.get("weight") {
            Some(Value::Float(f)) if f.0 > 0.0 => {
                // Valid.
            },
            Some(Value::Int(n)) if *n > 0 => {
                // Valid.
            },
            Some(Value::Float(f)) => {
                // Float ≤ 0.
                diagnostics.push(make_range_error(
                    &format!(
                        "weighted_composite components[{idx}].weight must be positive; \
                         got {}.",
                        f.0
                    ),
                    span,
                ));
            },
            Some(Value::Int(n)) => {
                // Int ≤ 0.
                diagnostics.push(make_range_error(
                    &format!(
                        "weighted_composite components[{idx}].weight must be positive; \
                         got {n}."
                    ),
                    span,
                ));
            },
            _ => {
                // Absent or wrong type.
                diagnostics.push(make_range_error(
                    &format!(
                        "weighted_composite components[{idx}].weight must be positive; \
                         got absent or wrong type."
                    ),
                    span,
                ));
            },
        }

        // Check score: must be Int in [0, 100].
        match comp_map.get("score") {
            Some(Value::Int(n)) if (0..=100).contains(n) => {
                // Valid.
            },
            Some(Value::Int(n)) => {
                // Out of range.
                diagnostics.push(make_range_error(
                    &format!(
                        "weighted_composite components[{idx}].score must be between 0 and 100; \
                         got {n}."
                    ),
                    span,
                ));
            },
            Some(other) => {
                // Wrong type.
                let actual_type = value_type_name(other);
                diagnostics.push(make_range_error(
                    &format!(
                        "weighted_composite components[{idx}].score must be an integer; \
                         got {actual_type}."
                    ),
                    span,
                ));
            },
            None => {
                // Absent.
                diagnostics.push(make_range_error(
                    &format!(
                        "weighted_composite components[{idx}].score must be an integer; \
                         got absent."
                    ),
                    span,
                ));
            },
        }
    }
}

/// Construct an E-VAL-011 error diagnostic.
fn make_range_error(message: &str, span: &SourceSpan) -> Diagnostic {
    Diagnostic {
        severity: DiagnosticSeverity::Error,
        code: Arc::from(E_VAL_011),
        message: Arc::from(message),
        span: span.clone(),
        hint: None,
    }
}

/// Return a human-readable type name for a [`Value`] variant.
fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Int(_) => "Int",
        Value::Float(_) => "Float",
        Value::Str(_) => "Str",
        Value::Bool(_) => "Bool",
        Value::List(_) => "List",
        Value::Map(_) => "Map",
        Value::Null => "Null",
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)] // BC-traceability IDs use uppercase
#[allow(clippy::unwrap_used)] // test code
mod tests {
    use std::sync::Arc;

    use ordered_float::OrderedFloat;
    use slideforge_plugin_api::{Validator, ValidatorOptions};
    use slideforge_types::{Deck, DeckMetadata, FieldValue, OrderedMap, Slide, SourceSpan, Value};

    use super::{E_VAL_011, ValueRangeValidator};

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
        }
    }

    fn default_opts() -> ValidatorOptions {
        ValidatorOptions::default()
    }

    /// Build a `progress_bar` slide with the given `value` field.
    fn make_progress_bar_slide(value_field: Option<FieldValue>) -> Slide {
        let mut fields: OrderedMap<Arc<str>, FieldValue> = OrderedMap::new();
        fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from("Sprint 4"))),
        );
        fields.insert(
            Arc::from("label"),
            FieldValue::Literal(Value::Str(Arc::from("75% complete"))),
        );
        if let Some(fv) = value_field {
            fields.insert(Arc::from("value"), fv);
        }
        Slide {
            slide_type: Arc::from("progress_bar"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }
    }

    /// Build a `weighted_composite` slide with given components.
    ///
    /// Each component is a `Value::Map` built from the provided tuples:
    /// `(name, weight_as_float, score_as_int)`.
    fn make_weighted_composite_slide(components: Vec<OrderedMap<Arc<str>, Value>>) -> Slide {
        let mut fields: OrderedMap<Arc<str>, FieldValue> = OrderedMap::new();
        fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from("Vendor A"))),
        );
        fields.insert(
            Arc::from("label"),
            FieldValue::Literal(Value::Str(Arc::from("Overall: Good"))),
        );
        let list: Vec<Value> = components.into_iter().map(Value::Map).collect();
        fields.insert(
            Arc::from("components"),
            FieldValue::Literal(Value::List(list)),
        );
        Slide {
            slide_type: Arc::from("weighted_composite"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }
    }

    /// Build a component map with float weight and int score.
    fn make_component(
        name: &str,
        weight: f64,
        score: i64,
        with_label: bool,
    ) -> OrderedMap<Arc<str>, Value> {
        let mut m: OrderedMap<Arc<str>, Value> = OrderedMap::new();
        m.insert(Arc::from("name"), Value::Str(Arc::from(name)));
        m.insert(Arc::from("weight"), Value::Float(OrderedFloat(weight)));
        m.insert(Arc::from("score"), Value::Int(score));
        if with_label {
            m.insert(Arc::from("label"), Value::Str(Arc::from("OK")));
        }
        m
    }

    // ── progress_bar — value in range ────────────────────────────────────────

    /// BC-1.17.002 PC3: `value=50` (mid-range) → 0 E-VAL-011.
    #[test]
    fn test_BC_1_17_002_value_in_range_no_error() {
        let slide = make_progress_bar_slide(Some(FieldValue::Literal(Value::Int(50))));
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011)
            .collect();
        assert!(
            errors.is_empty(),
            "progress_bar value=50 must produce no E-VAL-011; got {diags:?}"
        );
    }

    /// BC-1.17.002 PC3 boundary: `value=0` (lower boundary) → 0 E-VAL-011.
    #[test]
    fn test_BC_1_17_002_value_zero_boundary() {
        let slide = make_progress_bar_slide(Some(FieldValue::Literal(Value::Int(0))));
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011)
            .collect();
        assert!(
            errors.is_empty(),
            "progress_bar value=0 (lower boundary) must produce no E-VAL-011; got {diags:?}"
        );
    }

    /// BC-1.17.002 PC3 boundary: `value=100` (upper boundary) → 0 E-VAL-011.
    #[test]
    fn test_BC_1_17_002_value_100_boundary() {
        let slide = make_progress_bar_slide(Some(FieldValue::Literal(Value::Int(100))));
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011)
            .collect();
        assert!(
            errors.is_empty(),
            "progress_bar value=100 (upper boundary) must produce no E-VAL-011; got {diags:?}"
        );
    }

    /// BC-1.17.002 PC3: `value=101` (above max) → 1 E-VAL-011 with message containing "101".
    #[test]
    fn test_BC_1_17_002_value_101_out_of_range() {
        let slide = make_progress_bar_slide(Some(FieldValue::Literal(Value::Int(101))));
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011)
            .collect();
        assert_eq!(
            errors.len(),
            1,
            "progress_bar value=101 must produce exactly 1 E-VAL-011; got {diags:?}"
        );
        assert!(
            errors[0].message.contains("101"),
            "E-VAL-011 message must mention the offending value 101; got: {}",
            errors[0].message
        );
        assert_eq!(
            errors[0].severity,
            slideforge_plugin_api::DiagnosticSeverity::Error,
            "E-VAL-011 must have Error severity"
        );
    }

    /// BC-1.17.002 PC3: `value=-1` (below min) → 1 E-VAL-011 with message containing "-1".
    #[test]
    fn test_BC_1_17_002_value_neg1_out_of_range() {
        let slide = make_progress_bar_slide(Some(FieldValue::Literal(Value::Int(-1))));
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011)
            .collect();
        assert_eq!(
            errors.len(),
            1,
            "progress_bar value=-1 must produce exactly 1 E-VAL-011; got {diags:?}"
        );
        assert!(
            errors[0].message.contains("-1"),
            "E-VAL-011 message must mention the offending value -1; got: {}",
            errors[0].message
        );
    }

    /// BC-1.17.002 PC3: absent `value` field → 1 E-VAL-011.
    #[test]
    fn test_BC_1_17_002_value_absent() {
        let slide = make_progress_bar_slide(None);
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011)
            .collect();
        assert_eq!(
            errors.len(),
            1,
            "progress_bar with absent value must produce 1 E-VAL-011; got {diags:?}"
        );
    }

    /// BC-1.17.002 PC3: `value` is `Str` (wrong type) → 1 E-VAL-011.
    #[test]
    fn test_BC_1_17_002_value_wrong_type() {
        let slide =
            make_progress_bar_slide(Some(FieldValue::Literal(Value::Str(Arc::from("fifty")))));
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011)
            .collect();
        assert_eq!(
            errors.len(),
            1,
            "progress_bar value=Str must produce 1 E-VAL-011; got {diags:?}"
        );
    }

    // ── Non-color-coded slide not checked ─────────────────────────────────────

    /// Non-color-coded slide types must not be checked by `ValueRangeValidator`.
    #[test]
    fn test_non_color_coded_slide_not_checked() {
        let slide = Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "title slide must produce no E-VAL-011; got {diags:?}"
        );
    }

    /// Empty deck produces no diagnostics.
    #[test]
    fn test_empty_deck_no_diagnostics() {
        let deck = make_deck(vec![]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        assert!(diags.is_empty(), "empty deck must produce no diagnostics");
    }

    // ── weighted_composite — weight checks ────────────────────────────────────

    /// BC-1.17.003 inv 6: component `weight=0.5` (positive float) → 0 E-VAL-011 for weight.
    #[test]
    fn test_BC_1_17_003_weight_positive_ok() {
        let comp = make_component("Quality", 0.5, 85, true);
        let slide = make_weighted_composite_slide(vec![comp]);
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let weight_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011 && d.message.contains("weight"))
            .collect();
        assert!(
            weight_errors.is_empty(),
            "weight=0.5 must produce no weight E-VAL-011; got {diags:?}"
        );
    }

    /// BC-1.17.003 inv 6: component `weight=0` (zero) → 1 E-VAL-011 for weight.
    #[test]
    fn test_BC_1_17_003_weight_zero_error() {
        let comp = make_component("Quality", 0.0, 85, true);
        let slide = make_weighted_composite_slide(vec![comp]);
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let weight_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011 && d.message.contains("weight"))
            .collect();
        assert_eq!(
            weight_errors.len(),
            1,
            "weight=0 must produce 1 weight E-VAL-011; got {diags:?}"
        );
    }

    /// BC-1.17.003 inv 6: component `weight=-0.5` (negative) → 1 E-VAL-011 for weight.
    #[test]
    fn test_BC_1_17_003_weight_negative_error() {
        let comp = make_component("Quality", -0.5, 85, true);
        let slide = make_weighted_composite_slide(vec![comp]);
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let weight_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011 && d.message.contains("weight"))
            .collect();
        assert_eq!(
            weight_errors.len(),
            1,
            "weight=-0.5 must produce 1 weight E-VAL-011; got {diags:?}"
        );
    }

    // ── weighted_composite — score checks ─────────────────────────────────────

    /// BC-1.17.003 inv 7: component `score=85` (in range) → 0 E-VAL-011 for score.
    #[test]
    fn test_BC_1_17_003_score_in_range_ok() {
        let comp = make_component("Quality", 0.4, 85, true);
        let slide = make_weighted_composite_slide(vec![comp]);
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let score_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011 && d.message.contains("score"))
            .collect();
        assert!(
            score_errors.is_empty(),
            "score=85 must produce no score E-VAL-011; got {diags:?}"
        );
    }

    /// BC-1.17.003 inv 7 boundary: `score=0` → 0 E-VAL-011.
    #[test]
    fn test_BC_1_17_003_score_0_boundary() {
        let comp = make_component("Quality", 0.4, 0, true);
        let slide = make_weighted_composite_slide(vec![comp]);
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let score_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011 && d.message.contains("score"))
            .collect();
        assert!(
            score_errors.is_empty(),
            "score=0 (lower boundary) must produce no score E-VAL-011; got {diags:?}"
        );
    }

    /// BC-1.17.003 inv 7 boundary: `score=100` → 0 E-VAL-011.
    #[test]
    fn test_BC_1_17_003_score_100_boundary() {
        let comp = make_component("Quality", 0.4, 100, true);
        let slide = make_weighted_composite_slide(vec![comp]);
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let score_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011 && d.message.contains("score"))
            .collect();
        assert!(
            score_errors.is_empty(),
            "score=100 (upper boundary) must produce no score E-VAL-011; got {diags:?}"
        );
    }

    /// BC-1.17.003 inv 7: `score=101` (above max) → 1 E-VAL-011 with "101" in message.
    #[test]
    fn test_BC_1_17_003_score_101_error() {
        let comp = make_component("Quality", 0.4, 101, true);
        let slide = make_weighted_composite_slide(vec![comp]);
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let score_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011 && d.message.contains("score"))
            .collect();
        assert_eq!(
            score_errors.len(),
            1,
            "score=101 must produce 1 score E-VAL-011; got {diags:?}"
        );
        assert!(
            score_errors[0].message.contains("101"),
            "E-VAL-011 message must mention 101; got: {}",
            score_errors[0].message
        );
    }

    /// BC-1.17.003 inv 7: `score=-1` (below min) → 1 E-VAL-011.
    #[test]
    fn test_BC_1_17_003_score_neg1_error() {
        let comp = make_component("Quality", 0.4, -1, true);
        let slide = make_weighted_composite_slide(vec![comp]);
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let score_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011 && d.message.contains("score"))
            .collect();
        assert_eq!(
            score_errors.len(),
            1,
            "score=-1 must produce 1 score E-VAL-011; got {diags:?}"
        );
    }

    /// BC-1.17.003 / DI-018: 2 components both with `score=101` → 2 E-VAL-011 diagnostics.
    ///
    /// Proves error accumulation (no bail-on-first) per DI-018.
    ///
    /// SID-1 compliance: this unit test drives `weighted_composite` score accumulation
    /// without requiring DSL list-of-map support (STORY-088). It is the load-bearing
    /// proof for DI-018 accumulation, covering:
    ///   `test_BC_1_17_003_build_weighted_composite_score_101_is_validation_failed`
    ///   (in `crates/slideforge/tests/e2e/story_087_value_range.rs`, `#[ignore]`'d
    ///   pending STORY-088 DSL list-literal parser).
    #[test]
    fn test_BC_1_17_003_accumulation_multiple_errors() {
        let comp1 = make_component("Quality", 0.4, 101, true);
        let comp2 = make_component("Price", 0.6, 101, true);
        let slide = make_weighted_composite_slide(vec![comp1, comp2]);
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let score_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011 && d.message.contains("score"))
            .collect();
        assert_eq!(
            score_errors.len(),
            2,
            "2 components with score=101 must produce 2 score E-VAL-011 (DI-018 \
             accumulation — no bail-on-first); got {diags:?}"
        );
    }

    // ── weighted_composite — SID-1 unit tests (covers #[ignore]'d e2e tests) ──

    /// SID-1: weight=0 (Int zero) → E-VAL-011.
    ///
    /// Covers `test_BC_1_17_003_build_weighted_composite_weight_zero_is_validation_failed`
    /// (in `crates/slideforge/tests/e2e/story_087_value_range.rs`, `#[ignore]`'d
    /// pending STORY-088 DSL list-literal parser).
    #[test]
    fn test_BC_1_17_003_weight_int_zero_error() {
        let mut comp: OrderedMap<Arc<str>, Value> = OrderedMap::new();
        comp.insert(Arc::from("name"), Value::Str(Arc::from("Q")));
        comp.insert(Arc::from("weight"), Value::Int(0)); // Int zero
        comp.insert(Arc::from("score"), Value::Int(50));
        comp.insert(Arc::from("label"), Value::Str(Arc::from("Mid")));

        let slide = make_weighted_composite_slide(vec![comp]);
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let weight_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011 && d.message.contains("weight"))
            .collect();
        assert_eq!(
            weight_errors.len(),
            1,
            "Int weight=0 must produce 1 weight E-VAL-011; got {diags:?}"
        );
    }

    /// SID-1: weight absent → E-VAL-011.
    #[test]
    fn test_BC_1_17_003_weight_absent_error() {
        let mut comp: OrderedMap<Arc<str>, Value> = OrderedMap::new();
        comp.insert(Arc::from("name"), Value::Str(Arc::from("Q")));
        // weight absent
        comp.insert(Arc::from("score"), Value::Int(50));
        comp.insert(Arc::from("label"), Value::Str(Arc::from("Mid")));

        let slide = make_weighted_composite_slide(vec![comp]);
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let weight_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011 && d.message.contains("weight"))
            .collect();
        assert_eq!(
            weight_errors.len(),
            1,
            "absent weight must produce 1 weight E-VAL-011; got {diags:?}"
        );
    }

    /// SID-1: score absent → E-VAL-011.
    #[test]
    fn test_BC_1_17_003_score_absent_error() {
        let mut comp: OrderedMap<Arc<str>, Value> = OrderedMap::new();
        comp.insert(Arc::from("name"), Value::Str(Arc::from("Q")));
        comp.insert(Arc::from("weight"), Value::Float(OrderedFloat(0.4)));
        // score absent
        comp.insert(Arc::from("label"), Value::Str(Arc::from("Mid")));

        let slide = make_weighted_composite_slide(vec![comp]);
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());
        let score_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011 && d.message.contains("score"))
            .collect();
        assert_eq!(
            score_errors.len(),
            1,
            "absent score must produce 1 score E-VAL-011; got {diags:?}"
        );
    }

    /// `ValueRangeValidator` ID must be `"value-range"`.
    #[test]
    fn test_value_range_validator_id() {
        assert_eq!(ValueRangeValidator.id(), "value-range");
    }

    // ── §10.3 — empty components check ───────────────────────────────────────
    //
    // Architect pass-2 adjudication §7 (F-087-P2-001) / BC-1.17.003 postcondition 3:
    // `components: []` (empty list) → E-VAL-011 with message containing
    // "at least one component".
    //
    // RED GATE: the current `validate_weighted_composite_components` silently
    // returns when the list is empty (early-return arm without a diagnostic).
    // This test will FAIL until the implementer adds the empty-list arm
    // per architect adjudication §7.

    /// BC-1.17.003 PC-3 / F-087-P2-001 / adjudication §7:
    /// `weighted_composite` with `components: []` (empty list) → exactly 1 E-VAL-011
    /// diagnostic with message containing "at least one component".
    ///
    /// RED GATE: the current `validate_weighted_composite_components` returns
    /// early for an empty list WITHOUT emitting E-VAL-011 (the deferral comment
    /// "Empty-components validation is a separate concern" is overridden by
    /// architect adjudication §7 under the no-MVP canonical principle).
    #[test]
    fn test_BC_1_17_003_empty_components_is_error() {
        let mut fields: OrderedMap<Arc<str>, FieldValue> = OrderedMap::new();
        fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from("Vendor A"))),
        );
        fields.insert(
            Arc::from("label"),
            FieldValue::Literal(Value::Str(Arc::from("Overall: Good"))),
        );
        // Empty list — this is the case that must produce E-VAL-011.
        fields.insert(
            Arc::from("components"),
            FieldValue::Literal(Value::List(vec![])),
        );
        let slide = Slide {
            slide_type: Arc::from("weighted_composite"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let diags = ValueRangeValidator.validate(&deck, &default_opts());

        let val_011_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_VAL_011)
            .collect();

        assert_eq!(
            val_011_errors.len(),
            1,
            "RED GATE: weighted_composite with components=[] must produce exactly 1 E-VAL-011. \
             Current code silently returns for empty list (deferral comment overridden by \
             architect adjudication F-087-P2-001 §7). Implement the empty-list arm per §7. \
             Got diagnostics: {diags:?}"
        );
        assert!(
            val_011_errors[0].message.contains("at least one component"),
            "E-VAL-011 message must contain 'at least one component'; got: {}",
            val_011_errors[0].message
        );
        assert_eq!(
            val_011_errors[0].severity,
            slideforge_plugin_api::DiagnosticSeverity::Error,
            "E-VAL-011 must have Error severity (strict-mode fatal)"
        );
    }
}
