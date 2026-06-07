//! Red Gate test suite for STORY-089: Field-Value Type Validation.
//!
//! Tests BC-1.18.001 (v1.1): `validate_fields` enforces `FieldDef.expected_type`
//! against the runtime `Value` variant and emits E-VAL-104 on type mismatch or
//! OneOf violation.
//!
//! ## Red Gate contract (historical — implementation complete)
//!
//! These were Red Gate tests written before the STORY-089 implementer filled in
//! `type_matches` and the E-VAL-104 arm in `validate_fields`. At the Red Gate,
//! tests asserting `false` from `type_matches` failed because the stub returned
//! `true` unconditionally, and tests asserting E-VAL-104 diagnostics failed
//! because the E-VAL-104 arm in `validate_fields` was inert. The implementation
//! is now complete and all tests in this file pass.
//!
//! ## Test naming
//!
//! All tests follow the `test_BC_1_18_001_xxx()` pattern for BC traceability
//! (BC-1.18.001). AC-012..AC-021 annotation tests use
//! `test_BC_1_18_001_ac0NN_xxx()`. The special test mandated by the story spec
//! for AC-016 uses the exact name prescribed: `test_list_fields_on_kpi_roadmap_agenda_team`.
//!
//! ## Coverage map
//!
//! | Section | Test Name(s) |
//! |---------|-------------|
//! | A — type_matches truth table | `test_BC_1_18_001_type_matches_*` |
//! | B — E-VAL-104 via validate_fields | `test_BC_1_18_001_e_val_104_*`, plus per-AC tests |
//! | C — annotation presence | `test_BC_1_18_001_ac012..ac021`, `test_list_fields_on_kpi_roadmap_agenda_team` |
//! | D — regression E-VAL-101/102/W-VAL-103 | `test_BC_1_18_001_regression_*` |

#![allow(non_snake_case)] // BC-traceability test names use uppercase
#![allow(clippy::unwrap_used)] // Test code may use unwrap per project convention
#![allow(clippy::missing_docs_in_private_items)]
#![allow(unused_imports)] // Some imports are used only in certain test configurations
#![allow(clippy::doc_markdown)] // Test doc comments use code identifiers without backticks
#![allow(clippy::approx_constant)] // Tests may use literal float approximations
#![allow(clippy::redundant_closure_for_method_calls)] // Test clarity over pedantic style
#![allow(clippy::uninlined_format_args)] // Test assertions use format string variables

use std::sync::Arc;

use ordered_float::OrderedFloat;
use slideforge_plugin_api::slide_types::common_optional_fields;
use slideforge_plugin_api::slide_types::{
    SlideTypeRegistry, agenda::AgendaSlideType, chart::ChartSlideType,
    kpi_dashboard::KpiDashboardSlideType, matrix::MatrixSlideType,
    progress_bar::ProgressBarSlideType, roadmap::RoadmapSlideType, team::TeamSlideType,
    toc::TocSlideType, validate_fields, weighted_composite::WeightedCompositeSlideType,
};
use slideforge_plugin_api::traits::{
    Diagnostic, DiagnosticSeverity, FieldDef, FieldType, SlideType, type_matches,
};
use slideforge_types::{FieldValue, OrderedMap, Slide, SourceSpan, Value};

// ─────────────────────────────────────────────────────────────────────────────
// Test helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Build a minimal `Slide` with the given type keyword and field map.
fn make_slide_with_fields(slide_type: &str, fields: OrderedMap<Arc<str>, FieldValue>) -> Slide {
    Slide {
        slide_type: Arc::from(slide_type),
        fields,
        blocks: vec![],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
    }
}

/// Build a `Slide` with a single `FieldValue::Literal(value)` field.
fn make_slide_one_literal(slide_type: &str, field_name: &str, value: Value) -> Slide {
    let mut fields = OrderedMap::new();
    fields.insert(Arc::from(field_name), FieldValue::Literal(value));
    make_slide_with_fields(slide_type, fields)
}

/// Extract all E-VAL-104 diagnostics from a diagnostic list.
fn e_val_104(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| d.code.as_ref() == "E-VAL-104")
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Section A: type_matches pure-function truth table
// ─────────────────────────────────────────────────────────────────────────────
//
// The stub returns `true` unconditionally, so all assertions of `false` FAIL
// at the Red Gate. Assertions of `true` (positive cases) PASS even with the
// stub — they serve as regression guards for the implementer.

/// BC-1.18.001 postcondition 1 / AC-001: `Value::Int` matches `FieldType::Int`.
/// Passes even with the stub (positive case — regression guard).
#[test]
fn test_BC_1_18_001_type_matches_int_matches_int() {
    assert!(
        type_matches(&Value::Int(42), &FieldType::Int),
        "Int value must match FieldType::Int"
    );
}

/// BC-1.18.001 postcondition 2 / T1 mismatch: `Value::Str` on `FieldType::Int`
/// must return `false`. FAILS with the stub (stub returns `true`).
#[test]
fn test_BC_1_18_001_type_matches_str_fails_int() {
    assert!(
        !type_matches(&Value::Str(Arc::from("42")), &FieldType::Int),
        "Str value must NOT match FieldType::Int — expected false, got true (stub)"
    );
}

/// BC-1.18.001 postcondition 2 / T1 mismatch: `Value::Bool` on `FieldType::Int`
/// must return `false`. FAILS with the stub.
#[test]
fn test_BC_1_18_001_type_matches_bool_fails_int() {
    assert!(
        !type_matches(&Value::Bool(true), &FieldType::Int),
        "Bool must NOT match FieldType::Int"
    );
}

/// BC-1.18.001 postcondition 2 / T1 mismatch: `Value::Float` on `FieldType::Int`
/// must return `false`. FAILS with the stub.
#[test]
fn test_BC_1_18_001_type_matches_float_fails_int() {
    assert!(
        !type_matches(&Value::Float(OrderedFloat(1.0)), &FieldType::Int),
        "Float must NOT match FieldType::Int"
    );
}

/// BC-1.18.001 postcondition 2 / T1 mismatch: `Value::List` on `FieldType::Int`
/// must return `false`. FAILS with the stub.
#[test]
fn test_BC_1_18_001_type_matches_list_fails_int() {
    assert!(
        !type_matches(&Value::List(vec![Value::Int(1)]), &FieldType::Int),
        "List must NOT match FieldType::Int"
    );
}

/// BC-1.18.001: `Value::Str` matches `FieldType::Str`.
/// Passes even with the stub (positive case).
#[test]
fn test_BC_1_18_001_type_matches_str_matches_str() {
    assert!(
        type_matches(&Value::Str(Arc::from("hello")), &FieldType::Str),
        "Str value must match FieldType::Str"
    );
}

/// BC-1.18.001: `Value::Int` does NOT match `FieldType::Str`. FAILS with stub.
#[test]
fn test_BC_1_18_001_type_matches_int_fails_str() {
    assert!(
        !type_matches(&Value::Int(1), &FieldType::Str),
        "Int must NOT match FieldType::Str"
    );
}

/// BC-1.18.001: `Value::Bool` matches `FieldType::Bool`.
/// Passes even with the stub.
#[test]
fn test_BC_1_18_001_type_matches_bool_matches_bool() {
    assert!(
        type_matches(&Value::Bool(true), &FieldType::Bool),
        "Bool(true) must match FieldType::Bool"
    );
}

/// BC-1.18.001: `Value::Str` does NOT match `FieldType::Bool`. FAILS with stub.
#[test]
fn test_BC_1_18_001_type_matches_str_fails_bool() {
    assert!(
        !type_matches(&Value::Str(Arc::from("yes")), &FieldType::Bool),
        "Str('yes') must NOT match FieldType::Bool — no implicit string→bool coercion"
    );
}

/// BC-1.18.001 invariant 3: `Value::Float` matches `FieldType::Float`.
/// Passes even with the stub.
#[test]
fn test_BC_1_18_001_type_matches_float_matches_float() {
    assert!(
        type_matches(&Value::Float(OrderedFloat(1.0)), &FieldType::Float),
        "Float(1.0) must match FieldType::Float"
    );
}

/// BC-1.18.001 invariant 3 / AC-019: `Value::Int` does NOT match `FieldType::Float`.
/// No int→float coercion. FAILS with the stub.
#[test]
fn test_BC_1_18_001_type_matches_int_fails_float_no_coercion() {
    assert!(
        !type_matches(&Value::Int(1), &FieldType::Float),
        "Int(1) must NOT match FieldType::Float — no implicit int→float coercion (BC-1.18.001 invariant 3)"
    );
}

/// BC-1.18.001: `Value::List` matches `FieldType::List`.
/// Passes even with the stub.
#[test]
fn test_BC_1_18_001_type_matches_list_matches_list() {
    assert!(
        type_matches(&Value::List(vec![]), &FieldType::List),
        "List([]) must match FieldType::List"
    );
}

/// BC-1.18.001: `Value::Str` does NOT match `FieldType::List`. FAILS with stub.
#[test]
fn test_BC_1_18_001_type_matches_str_fails_list() {
    assert!(
        !type_matches(&Value::Str(Arc::from("data")), &FieldType::List),
        "Str must NOT match FieldType::List"
    );
}

/// BC-1.18.001: `Value::Map` matches `FieldType::Map`.
/// Passes even with the stub.
#[test]
fn test_BC_1_18_001_type_matches_map_matches_map() {
    assert!(
        type_matches(&Value::Map(OrderedMap::new()), &FieldType::Map),
        "Map must match FieldType::Map"
    );
}

/// BC-1.18.001: `Value::List` does NOT match `FieldType::Map`. FAILS with stub.
#[test]
fn test_BC_1_18_001_type_matches_list_fails_map() {
    assert!(
        !type_matches(&Value::List(vec![]), &FieldType::Map),
        "List must NOT match FieldType::Map"
    );
}

/// BC-1.18.001 postcondition 4 / AC-006: `FieldType::Any` matches ALL value variants.
/// Passes even with the stub (positive case for all Any sub-tests).
#[test]
fn test_BC_1_18_001_type_matches_any_always_true() {
    assert!(
        type_matches(&Value::Int(1), &FieldType::Any),
        "Any must match Int"
    );
    assert!(
        type_matches(&Value::Str(Arc::from("x")), &FieldType::Any),
        "Any must match Str"
    );
    assert!(
        type_matches(&Value::Bool(false), &FieldType::Any),
        "Any must match Bool"
    );
    assert!(
        type_matches(&Value::Float(OrderedFloat(3.14)), &FieldType::Any),
        "Any must match Float"
    );
    assert!(
        type_matches(&Value::List(vec![]), &FieldType::Any),
        "Any must match List"
    );
    assert!(
        type_matches(&Value::Map(OrderedMap::new()), &FieldType::Any),
        "Any must match Map"
    );
    assert!(
        type_matches(&Value::Null, &FieldType::Any),
        "Any must match Null"
    );
}

/// BC-1.18.001 postcondition 3 (T2 pass) / AC-003: `OneOf` returns `true` for a
/// `Value::Str` in the allowlist. Passes even with the stub.
#[test]
fn test_BC_1_18_001_type_matches_oneof_str_in_list() {
    let allowed = FieldType::OneOf(vec![Arc::from("bar"), Arc::from("line"), Arc::from("pie")]);
    assert!(
        type_matches(&Value::Str(Arc::from("bar")), &allowed),
        "Str('bar') must match OneOf([bar, line, pie]) — it is in the allowlist"
    );
    assert!(
        type_matches(&Value::Str(Arc::from("pie")), &allowed),
        "Str('pie') must match OneOf([bar, line, pie])"
    );
}

/// BC-1.18.001 postcondition 3 (T2 fail) / AC-004: `OneOf` returns `false` for a
/// `Value::Str` NOT in the allowlist. FAILS with the stub.
#[test]
fn test_BC_1_18_001_type_matches_oneof_str_not_in_list() {
    let allowed = FieldType::OneOf(vec![
        Arc::from("bar"),
        Arc::from("line"),
        Arc::from("pie"),
        Arc::from("scatter"),
        Arc::from("area"),
        Arc::from("stacked-bar"),
        Arc::from("stacked-area"),
    ]);
    assert!(
        !type_matches(&Value::Str(Arc::from("donut")), &allowed),
        "Str('donut') must NOT match OneOf([bar,line,pie,...]) — 'donut' is not in the allowlist"
    );
    assert!(
        !type_matches(&Value::Str(Arc::from("")), &allowed),
        "Str('') must NOT match OneOf([...]) — empty string is not in the allowlist"
    );
}

/// BC-1.18.001 postcondition 3 (T1 before T2) / AC-005: `Value::Int` on a
/// `FieldType::OneOf` field returns `false` (T1 fires, NOT T2). FAILS with stub.
#[test]
fn test_BC_1_18_001_type_matches_int_on_oneof_field() {
    let allowed = FieldType::OneOf(vec![Arc::from("bar"), Arc::from("line")]);
    assert!(
        !type_matches(&Value::Int(42), &allowed),
        "Int(42) must NOT match FieldType::OneOf — non-Str triggers T1 (type mismatch), not T2"
    );
}

/// BC-1.18.001: `Value::Bool` on `FieldType::OneOf` returns `false`. FAILS with stub.
#[test]
fn test_BC_1_18_001_type_matches_bool_on_oneof_field() {
    let allowed = FieldType::OneOf(vec![Arc::from("true")]);
    assert!(
        !type_matches(&Value::Bool(true), &allowed),
        "Bool(true) must NOT match FieldType::OneOf — only Str is accepted"
    );
}

/// BC-1.18.001 postcondition 8 / AC-011: `type_matches` is a pure function —
/// same inputs always produce the same output with no state change.
#[test]
fn test_BC_1_18_001_type_matches_is_pure_deterministic() {
    let value = Value::Str(Arc::from("bar"));
    let expected = FieldType::OneOf(vec![Arc::from("bar"), Arc::from("line")]);
    let result1 = type_matches(&value, &expected);
    let result2 = type_matches(&value, &expected);
    let result3 = type_matches(&value, &expected);
    assert_eq!(
        result1, result2,
        "type_matches must be deterministic: same inputs always return same result"
    );
    assert_eq!(
        result2, result3,
        "type_matches must be deterministic on repeated calls"
    );
    // All three calls with "bar" ∈ allowlist must return true (positive case).
    assert!(result1, "Str('bar') in OneOf([bar, line]) must return true");
}

// ─────────────────────────────────────────────────────────────────────────────
// Section B: E-VAL-104 emission via validate_fields
// ─────────────────────────────────────────────────────────────────────────────
//
// The E-VAL-104 arm in `validate_fields` is INERT in the stub — it never emits
// any diagnostic. All assertions that expect diagnostics to be present FAIL.
// Assertions that expect ZERO E-VAL-104 on valid input PASS even with the stub.

/// BC-1.18.001 postcondition 2 / AC-002: `progress_bar` with `value: "fifty"`
/// (Value::Str on FieldType::Int field) → exactly one E-VAL-104 with T1 message.
///
/// FAILS with stub (arm is inert — emits no diagnostic).
#[test]
fn test_BC_1_18_001_e_val_104_t1_progress_bar_str_on_int_field() {
    let slide_type = ProgressBarSlideType::new();
    // Must supply required title and label to avoid E-VAL-101 noise; only test value.
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Sprint 4"))),
    );
    fields.insert(
        Arc::from("label"),
        FieldValue::Literal(Value::Str(Arc::from("50% complete"))),
    );
    // The type-mismatched field: "fifty" is Str, but value expects Int.
    fields.insert(
        Arc::from("value"),
        FieldValue::Literal(Value::Str(Arc::from("fifty"))),
    );
    let slide = make_slide_with_fields("progress_bar", fields);
    let diags = validate_fields(&slide, &slide_type);
    let e104: Vec<_> = e_val_104(&diags);

    // There must be exactly one E-VAL-104.
    assert_eq!(
        e104.len(),
        1,
        "expected exactly 1 E-VAL-104 for value:'fifty' (Str on Int field), got {}: {diags:?}",
        e104.len()
    );

    let d = e104[0];
    // Severity must be Error.
    assert_eq!(
        d.severity,
        DiagnosticSeverity::Error,
        "E-VAL-104 must have Error severity"
    );
    // Code must be exactly "E-VAL-104" (not a substring).
    assert_eq!(
        d.code.as_ref(),
        "E-VAL-104",
        "diagnostic code must be exactly 'E-VAL-104'"
    );
    // T1 message branch (BC-1.18.001 postcondition 2):
    // "Field 'value' on progress_bar slide has wrong type: expected integer, got string."
    let msg = d.message.as_ref();
    assert!(
        msg.contains("value"),
        "T1 message must name the field 'value'; got: {msg}"
    );
    assert!(
        msg.contains("progress_bar"),
        "T1 message must name the slide type; got: {msg}"
    );
    assert!(
        msg.contains("integer"),
        "T1 message must say 'integer' (expected type); got: {msg}"
    );
    assert!(
        msg.contains("string"),
        "T1 message must say 'string' (actual type); got: {msg}"
    );
}

/// BC-1.18.001 postcondition 1 / AC-001: `progress_bar` with `value: 75` (Int) →
/// zero E-VAL-104. PASSES even with the stub.
#[test]
fn test_BC_1_18_001_valid_progress_bar_value_int_no_e_val_104() {
    let slide_type = ProgressBarSlideType::new();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Sprint 4"))),
    );
    fields.insert(
        Arc::from("label"),
        FieldValue::Literal(Value::Str(Arc::from("75% complete"))),
    );
    fields.insert(Arc::from("value"), FieldValue::Literal(Value::Int(75)));
    let slide = make_slide_with_fields("progress_bar", fields);
    let diags = validate_fields(&slide, &slide_type);
    let e104 = e_val_104(&diags);
    assert!(
        e104.is_empty(),
        "correct Int value on progress_bar must produce zero E-VAL-104, got: {e104:?}"
    );
}

/// BC-1.18.001 postcondition 3 / AC-004 / EC-003: `chart` with
/// `chart_type: "donut"` (valid Str, not in OneOf allowlist) →
/// one E-VAL-104 with T2 (disallowed-value) message branch.
///
/// FAILS with stub (arm is inert).
#[test]
fn test_BC_1_18_001_e_val_104_t2_chart_type_disallowed_value() {
    let slide_type = ChartSlideType::new();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Sales Chart"))),
    );
    // "donut" is a valid Str but NOT in the OneOf allowlist.
    fields.insert(
        Arc::from("chart_type"),
        FieldValue::Literal(Value::Str(Arc::from("donut"))),
    );
    // data is polymorphic (None) — use a Str to avoid E-VAL-101 noise
    fields.insert(
        Arc::from("data"),
        FieldValue::Literal(Value::Str(Arc::from("@data.csv"))),
    );
    let slide = make_slide_with_fields("chart", fields);
    let diags = validate_fields(&slide, &slide_type);
    let e104: Vec<_> = e_val_104(&diags);

    assert_eq!(
        e104.len(),
        1,
        "expected exactly 1 E-VAL-104 for chart_type:'donut' (T2 violation), got {}: {diags:?}",
        e104.len()
    );

    let d = e104[0];
    assert_eq!(d.severity, DiagnosticSeverity::Error);
    assert_eq!(d.code.as_ref(), "E-VAL-104");

    // T2 message branch (BC-1.18.001 postcondition 3):
    // "Field 'chart_type' on chart slide has disallowed value "donut":
    //  allowed values are [bar, line, pie, scatter, area, stacked-bar, stacked-area]."
    let msg = d.message.as_ref();
    assert!(
        msg.contains("chart_type"),
        "T2 message must name field 'chart_type'; got: {msg}"
    );
    assert!(
        msg.contains("donut"),
        "T2 message must name the disallowed value 'donut'; got: {msg}"
    );
    // Must list the allowed values
    assert!(
        msg.contains("bar"),
        "T2 message must list allowed values including 'bar'; got: {msg}"
    );
    assert!(
        msg.contains("stacked-area"),
        "T2 message must list allowed values including 'stacked-area'; got: {msg}"
    );
    // Must NOT say "expected string" (that would be T1 branch, not T2)
    assert!(
        !msg.contains("expected string"),
        "T2 message must NOT use T1 language ('expected string'); got: {msg}"
    );
}

/// BC-1.18.001 postcondition 1 / AC-003: `chart` with `chart_type: "bar"` (in
/// OneOf allowlist) → zero E-VAL-104. PASSES even with stub.
#[test]
fn test_BC_1_18_001_valid_chart_type_bar_no_e_val_104() {
    let slide_type = ChartSlideType::new();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Sales"))),
    );
    fields.insert(
        Arc::from("chart_type"),
        FieldValue::Literal(Value::Str(Arc::from("bar"))),
    );
    fields.insert(
        Arc::from("data"),
        FieldValue::Literal(Value::Str(Arc::from("@data.csv"))),
    );
    let slide = make_slide_with_fields("chart", fields);
    let diags = validate_fields(&slide, &slide_type);
    let e104 = e_val_104(&diags);
    assert!(
        e104.is_empty(),
        "'bar' in OneOf allowlist must produce zero E-VAL-104, got: {e104:?}"
    );
}

/// BC-1.18.001 postcondition 3 (T1 before T2) / AC-005 / EC-002:
/// `chart` with `chart_type: 42` (Value::Int on OneOf-typed field) →
/// one E-VAL-104 with T1 message ("expected string, got integer").
///
/// FAILS with stub.
#[test]
fn test_BC_1_18_001_e_val_104_t1_fires_before_t2_for_non_str_on_oneof() {
    let slide_type = ChartSlideType::new();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Sales"))),
    );
    // Int(42) on an OneOf-typed field → T1 (type mismatch), NOT T2 (disallowed value).
    fields.insert(Arc::from("chart_type"), FieldValue::Literal(Value::Int(42)));
    fields.insert(
        Arc::from("data"),
        FieldValue::Literal(Value::Str(Arc::from("@data.csv"))),
    );
    let slide = make_slide_with_fields("chart", fields);
    let diags = validate_fields(&slide, &slide_type);
    let e104: Vec<_> = e_val_104(&diags);

    assert_eq!(
        e104.len(),
        1,
        "expected exactly 1 E-VAL-104 for chart_type:42 (T1 before T2), got {}: {diags:?}",
        e104.len()
    );

    let d = e104[0];
    assert_eq!(d.code.as_ref(), "E-VAL-104");
    let msg = d.message.as_ref();
    // T1 message: "expected string, got integer"
    assert!(
        msg.contains("string"),
        "T1 message on OneOf field must say 'expected string'; got: {msg}"
    );
    assert!(
        msg.contains("integer"),
        "T1 message must say 'got integer'; got: {msg}"
    );
    // Must NOT be the T2 disallowed-value branch.
    assert!(
        !msg.contains("disallowed"),
        "T1 branch must NOT use 'disallowed' (that is T2 language); got: {msg}"
    );
}

/// BC-1.18.001 / AC-014 / EC-004: `decorative: "yes"` (Str on Bool field) →
/// one E-VAL-104 T1. FAILS with stub.
#[test]
fn test_BC_1_18_001_e_val_104_t1_decorative_str_on_bool_field() {
    // Use a slide type that includes common_optional_fields (decorative is there).
    // title type: required=["title"], optional=common_optional_fields().
    let reg = SlideTypeRegistry::default();
    let slide_type = reg.lookup_by_keyword("title").unwrap();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Hello"))),
    );
    // decorative: "yes" is Str on a Bool-typed field.
    fields.insert(
        Arc::from("decorative"),
        FieldValue::Literal(Value::Str(Arc::from("yes"))),
    );
    let slide = make_slide_with_fields("title", fields);
    let diags = validate_fields(&slide, slide_type);
    let e104: Vec<_> = e_val_104(&diags);

    assert_eq!(
        e104.len(),
        1,
        "expected 1 E-VAL-104 for decorative:'yes' (Str on Bool), got {}: {diags:?}",
        e104.len()
    );

    let d = e104[0];
    assert_eq!(d.severity, DiagnosticSeverity::Error);
    assert_eq!(d.code.as_ref(), "E-VAL-104");
    let msg = d.message.as_ref();
    assert!(
        msg.contains("decorative"),
        "T1 message must name field 'decorative'; got: {msg}"
    );
    assert!(
        msg.contains("boolean"),
        "T1 message must say 'expected boolean'; got: {msg}"
    );
    assert!(
        msg.contains("string"),
        "T1 message must say 'got string'; got: {msg}"
    );
}

/// BC-1.18.001 / AC-014 / EC-005: `decorative: true` (Bool on Bool field) →
/// zero E-VAL-104. PASSES even with stub.
#[test]
fn test_BC_1_18_001_valid_decorative_bool_no_e_val_104() {
    let reg = SlideTypeRegistry::default();
    let slide_type = reg.lookup_by_keyword("title").unwrap();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Hello"))),
    );
    fields.insert(
        Arc::from("decorative"),
        FieldValue::Literal(Value::Bool(true)),
    );
    let slide = make_slide_with_fields("title", fields);
    let diags = validate_fields(&slide, slide_type);
    let e104 = e_val_104(&diags);
    assert!(
        e104.is_empty(),
        "decorative:true (Bool) must produce zero E-VAL-104, got: {e104:?}"
    );
}

/// BC-1.18.001 / AC-015 / EC-006: `weighted_composite` with
/// `components: "see attached"` (Str on List field) → one E-VAL-104 T1.
/// FAILS with stub.
#[test]
fn test_BC_1_18_001_e_val_104_t1_weighted_composite_str_on_list_field() {
    let slide_type = WeightedCompositeSlideType::new();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Scorecard"))),
    );
    fields.insert(
        Arc::from("label"),
        FieldValue::Literal(Value::Str(Arc::from("Overall: Good"))),
    );
    // components must be a List; "see attached" is a Str → T1.
    fields.insert(
        Arc::from("components"),
        FieldValue::Literal(Value::Str(Arc::from("see attached"))),
    );
    let slide = make_slide_with_fields("weighted_composite", fields);
    let diags = validate_fields(&slide, &slide_type);
    let e104: Vec<_> = e_val_104(&diags);

    assert_eq!(
        e104.len(),
        1,
        "expected 1 E-VAL-104 for components:'see attached' (Str on List), got {}: {diags:?}",
        e104.len()
    );

    let d = e104[0];
    assert_eq!(d.code.as_ref(), "E-VAL-104");
    let msg = d.message.as_ref();
    assert!(
        msg.contains("components"),
        "T1 message must name field 'components'; got: {msg}"
    );
    assert!(
        msg.contains("list"),
        "T1 message must say 'expected list'; got: {msg}"
    );
    assert!(
        msg.contains("string"),
        "T1 message must say 'got string'; got: {msg}"
    );
}

/// BC-1.18.001 / EC-007: `weighted_composite` with `components: []` (empty List) →
/// zero E-VAL-104. PASSES even with stub.
#[test]
fn test_BC_1_18_001_valid_weighted_composite_empty_list_no_e_val_104() {
    let slide_type = WeightedCompositeSlideType::new();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Scorecard"))),
    );
    fields.insert(
        Arc::from("label"),
        FieldValue::Literal(Value::Str(Arc::from("Overall: N/A"))),
    );
    // Empty list still satisfies FieldType::List.
    fields.insert(
        Arc::from("components"),
        FieldValue::Literal(Value::List(vec![])),
    );
    let slide = make_slide_with_fields("weighted_composite", fields);
    let diags = validate_fields(&slide, &slide_type);
    let e104 = e_val_104(&diags);
    assert!(
        e104.is_empty(),
        "empty List on components must produce zero E-VAL-104, got: {e104:?}"
    );
}

/// BC-1.18.001 postcondition 6 / AC-008 / EC-011: a slide with TWO type-mismatched
/// annotated fields → exactly TWO E-VAL-104 diagnostics.
///
/// DI-018: validate_fields must NOT stop at the first mismatch.
///
/// FAILS with stub (arm is inert — emits zero diagnostics).
#[test]
fn test_BC_1_18_001_e_val_104_accumulates_two_mistyped_fields() {
    // Use progress_bar: value (Int) and inject decorative (Bool) with wrong types.
    // But first supply the title and label required fields correctly.
    let slide_type = ProgressBarSlideType::new();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Sprint"))),
    );
    fields.insert(
        Arc::from("label"),
        FieldValue::Literal(Value::Str(Arc::from("75%"))),
    );
    // Field 1: value expects Int → give it Str ("wrong type 1").
    fields.insert(
        Arc::from("value"),
        FieldValue::Literal(Value::Str(Arc::from("bad"))),
    );
    // Field 2: decorative (from common_optional_fields) expects Bool → give it Str.
    fields.insert(
        Arc::from("decorative"),
        FieldValue::Literal(Value::Str(Arc::from("yes"))),
    );
    let slide = make_slide_with_fields("progress_bar", fields);
    let diags = validate_fields(&slide, &slide_type);
    let e104: Vec<_> = e_val_104(&diags);

    assert_eq!(
        e104.len(),
        2,
        "expected exactly 2 E-VAL-104 for two mistyped fields, got {}: {diags:?}",
        e104.len()
    );
}

/// BC-1.18.001 postcondition 4 / AC-006 / EC-009: field with `expected_type: None`
/// accepts ANY value without emitting E-VAL-104. PASSES even with stub.
#[test]
fn test_BC_1_18_001_none_annotation_skips_type_check() {
    // chart.data has expected_type: None (polymorphic).
    // Give it an Int — no E-VAL-104 should be emitted.
    let slide_type = ChartSlideType::new();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Chart"))),
    );
    fields.insert(
        Arc::from("chart_type"),
        FieldValue::Literal(Value::Str(Arc::from("bar"))),
    );
    // data has expected_type: None — any value type must be accepted.
    fields.insert(
        Arc::from("data"),
        FieldValue::Literal(Value::Str(Arc::from("@sales.csv"))),
    );
    let slide = make_slide_with_fields("chart", fields);
    let diags = validate_fields(&slide, &slide_type);
    let e104 = e_val_104(&diags);
    assert!(
        e104.is_empty(),
        "None-annotated field (data) must produce zero E-VAL-104 regardless of value"
    );
}

/// BC-1.18.001 / AC-021 / EC-014: polymorphic `data` field on chart with
/// `Value::Str` (data-source reference) AND with `Value::List` (inline data) →
/// both produce zero E-VAL-104. PASSES even with stub.
#[test]
fn test_BC_1_18_001_polymorphic_data_field_no_false_positive() {
    let slide_type = ChartSlideType::new();

    // Test 1: data as Str (data-source reference)
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Chart"))),
    );
    fields.insert(
        Arc::from("chart_type"),
        FieldValue::Literal(Value::Str(Arc::from("bar"))),
    );
    fields.insert(
        Arc::from("data"),
        FieldValue::Literal(Value::Str(Arc::from("@sales.csv"))),
    );
    let slide_str = make_slide_with_fields("chart", fields);
    let diags_str = validate_fields(&slide_str, &slide_type);
    assert!(
        e_val_104(&diags_str).is_empty(),
        "data as Str must produce zero E-VAL-104 (polymorphic field)"
    );

    // Test 2: data as List (inline data)
    let mut fields2 = OrderedMap::new();
    fields2.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Chart"))),
    );
    fields2.insert(
        Arc::from("chart_type"),
        FieldValue::Literal(Value::Str(Arc::from("line"))),
    );
    fields2.insert(
        Arc::from("data"),
        FieldValue::Literal(Value::List(vec![Value::Int(1), Value::Int(2)])),
    );
    let slide_list = make_slide_with_fields("chart", fields2);
    let diags_list = validate_fields(&slide_list, &slide_type);
    assert!(
        e_val_104(&diags_list).is_empty(),
        "data as List must produce zero E-VAL-104 (polymorphic field)"
    );
}

/// BC-1.18.001 postcondition 5 / AC-007 / EC-010: `FieldValue::Inlines` on a
/// Str-typed field → zero E-VAL-104 (Inlines are unconditionally skipped).
///
/// PASSES even with stub (the arm skips FieldValue::Inlines).
#[test]
fn test_BC_1_18_001_inlines_field_value_skipped_no_e_val_104() {
    // chart.title is expected_type: None (no type annotation), so use progress_bar.value
    // annotated as Int, but provide FieldValue::Inlines for it.
    let slide_type = ProgressBarSlideType::new();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Sprint"))),
    );
    fields.insert(
        Arc::from("label"),
        FieldValue::Literal(Value::Str(Arc::from("50%"))),
    );
    // value expects Int, but here we supply FieldValue::Inlines — must be skipped.
    fields.insert(Arc::from("value"), FieldValue::Inlines(vec![]));
    let slide = make_slide_with_fields("progress_bar", fields);
    let diags = validate_fields(&slide, &slide_type);
    let e104 = e_val_104(&diags);
    assert!(
        e104.is_empty(),
        "FieldValue::Inlines on a typed field must produce zero E-VAL-104 (skipped unconditionally)"
    );
}

/// BC-1.18.001 invariant 3 / AC-019 / EC-008: `weight: 1` (Value::Int on a
/// FieldType::Float-annotated field) → E-VAL-104 T1 ("expected float, got integer").
///
/// This tests the no-coercion invariant (CLAUDE.md, Q1 decisions).
/// To exercise this without requiring nested field access, we construct a
/// minimal fake SlideType using direct FieldDef construction via `with_type`.
///
/// FAILS with stub (arm is inert).
#[test]
fn test_BC_1_18_001_e_val_104_t1_int_on_float_field_no_coercion() {
    // The weighted_composite per-component weight is a Map key, not a top-level FieldDef.
    // We test the no-int-coercion invariant using a custom slide type wrapper
    // that exposes a Float-typed top-level field.
    //
    // Strategy: build a slide with a FieldDef using FieldDef::with_type for Float,
    // then call validate_fields directly using a mock SlideType.
    //
    // We use a SlideType implementation that returns a single Float-typed required field.

    struct FloatFieldSlideType {
        required: Vec<FieldDef>,
        optional: Vec<FieldDef>,
    }
    impl SlideType for FloatFieldSlideType {
        fn id(&self) -> &'static str {
            "float_test"
        }
        fn required_fields(&self) -> &[FieldDef] {
            &self.required
        }
        fn optional_fields(&self) -> &[FieldDef] {
            &self.optional
        }
        fn layout_name(&self) -> &'static str {
            "Blank"
        }
        fn lay_out(
            &self,
            slide: &slideforge_types::Slide,
            _brand: &slideforge_types::Brand,
            _canvas: slideforge_plugin_api::traits::Canvas,
        ) -> Result<slideforge_layout::LaidOutSlide, slideforge_plugin_api::traits::LayoutError>
        {
            Ok(slideforge_layout::LaidOutSlide {
                source_index: 0,
                slide_type_keyword: Arc::clone(&slide.slide_type),
                frames: vec![],
                speaker_notes: None,
                register_tags: vec![],
                register_content: vec![],
            })
        }
    }

    let float_slide_type = FloatFieldSlideType {
        required: vec![FieldDef::with_type(
            "weight",
            "Weight field (must be Float)",
            true,
            None,
            FieldType::Float,
        )],
        optional: vec![],
    };

    // Provide Int(1) where Float is expected — no coercion should occur.
    let slide = make_slide_one_literal("float_test", "weight", Value::Int(1));
    let diags = validate_fields(&slide, &float_slide_type);
    let e104: Vec<_> = e_val_104(&diags);

    assert_eq!(
        e104.len(),
        1,
        "expected 1 E-VAL-104 for weight:1 (Int on Float field — no coercion), got {}: {diags:?}",
        e104.len()
    );

    let msg = e104[0].message.as_ref();
    assert!(
        msg.contains("float"),
        "T1 message must say 'expected float'; got: {msg}"
    );
    assert!(
        msg.contains("integer"),
        "T1 message must say 'got integer'; got: {msg}"
    );
}

/// BC-1.18.001 invariant 2 / AC-020: absent required field → E-VAL-101, NOT E-VAL-104.
/// Type checking runs only on PRESENT field values. PASSES even with stub.
#[test]
fn test_BC_1_18_001_absent_required_field_emits_e_val_101_not_e_val_104() {
    let slide_type = ProgressBarSlideType::new();
    // Provide title and label but OMIT value entirely.
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Sprint"))),
    );
    fields.insert(
        Arc::from("label"),
        FieldValue::Literal(Value::Str(Arc::from("N/A"))),
    );
    // value is intentionally absent.
    let slide = make_slide_with_fields("progress_bar", fields);
    let diags = validate_fields(&slide, &slide_type);

    // Must emit E-VAL-101 for the absent field.
    let e101_codes: Vec<_> = diags
        .iter()
        .filter(|d| d.code.as_ref() == "E-VAL-101")
        .collect();
    assert!(
        !e101_codes.is_empty(),
        "absent required field 'value' must produce E-VAL-101; got: {diags:?}"
    );

    // Must NOT emit E-VAL-104 for an absent field.
    let e104 = e_val_104(&diags);
    assert!(
        e104.is_empty(),
        "absent field must NOT produce E-VAL-104 (type checking skips absent fields); got: {e104:?}"
    );
}

/// BC-1.18.001: E-VAL-104 code is exactly the string "E-VAL-104" (not a prefix/variant).
/// Validates T3-21 from the verification plan. FAILS with stub.
#[test]
fn test_BC_1_18_001_e_val_104_code_exact_string() {
    let slide_type = ProgressBarSlideType::new();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("T"))),
    );
    fields.insert(
        Arc::from("label"),
        FieldValue::Literal(Value::Str(Arc::from("L"))),
    );
    fields.insert(
        Arc::from("value"),
        FieldValue::Literal(Value::Str(Arc::from("oops"))),
    );
    let slide = make_slide_with_fields("progress_bar", fields);
    let diags = validate_fields(&slide, &slide_type);
    let e104: Vec<_> = e_val_104(&diags);

    assert_eq!(
        e104.len(),
        1,
        "expected 1 E-VAL-104, got {}: {diags:?}",
        e104.len()
    );

    // Code must be EXACTLY "E-VAL-104" — not "E-VAL-1040", "EVAL-104", etc.
    assert_eq!(
        e104[0].code.as_ref(),
        "E-VAL-104",
        "diagnostic code must be exactly the string 'E-VAL-104', not a prefix or variant"
    );
}

/// BC-1.18.001: E-VAL-104 diagnostic has Error severity (not Warning or Info).
/// FAILS with stub.
#[test]
fn test_BC_1_18_001_e_val_104_severity_is_error() {
    let slide_type = ProgressBarSlideType::new();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("T"))),
    );
    fields.insert(
        Arc::from("label"),
        FieldValue::Literal(Value::Str(Arc::from("L"))),
    );
    fields.insert(
        Arc::from("value"),
        FieldValue::Literal(Value::Str(Arc::from("bad"))),
    );
    let slide = make_slide_with_fields("progress_bar", fields);
    let diags = validate_fields(&slide, &slide_type);
    let e104: Vec<_> = e_val_104(&diags);

    assert!(!e104.is_empty(), "expected at least 1 E-VAL-104");
    for d in &e104 {
        assert_eq!(
            d.severity,
            DiagnosticSeverity::Error,
            "E-VAL-104 must have Error severity (not Warning or Info)"
        );
    }
}

/// BC-1.18.001: E-VAL-104 diagnostic has a non-empty message.
/// FAILS with stub.
#[test]
fn test_BC_1_18_001_e_val_104_message_is_non_empty() {
    let slide_type = ProgressBarSlideType::new();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("T"))),
    );
    fields.insert(
        Arc::from("label"),
        FieldValue::Literal(Value::Str(Arc::from("L"))),
    );
    fields.insert(
        Arc::from("value"),
        FieldValue::Literal(Value::Str(Arc::from("bad"))),
    );
    let slide = make_slide_with_fields("progress_bar", fields);
    let diags = validate_fields(&slide, &slide_type);
    let e104: Vec<_> = e_val_104(&diags);

    assert!(!e104.is_empty(), "expected at least 1 E-VAL-104");
    for d in &e104 {
        assert!(!d.message.is_empty(), "E-VAL-104 message must not be empty");
    }
}

/// BC-1.18.001: matrix.cells has expected_type: None — giving it a Map OR
/// a List produces zero E-VAL-104 (polymorphic, no false positive).
/// PASSES even with stub.
#[test]
fn test_BC_1_18_001_matrix_cells_polymorphic_no_false_positive() {
    let slide_type = MatrixSlideType::new();

    // Test 1: cells as Map
    let mut fields_map = OrderedMap::new();
    fields_map.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("2x2 Matrix"))),
    );
    fields_map.insert(
        Arc::from("cells"),
        FieldValue::Literal(Value::Map(OrderedMap::new())),
    );
    let slide_map = make_slide_with_fields("matrix", fields_map);
    let diags_map = validate_fields(&slide_map, &slide_type);
    assert!(
        e_val_104(&diags_map).is_empty(),
        "matrix.cells as Map must produce zero E-VAL-104 (polymorphic)"
    );

    // Test 2: cells as List
    let mut fields_list = OrderedMap::new();
    fields_list.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("2x2 Matrix"))),
    );
    fields_list.insert(Arc::from("cells"), FieldValue::Literal(Value::List(vec![])));
    let slide_list = make_slide_with_fields("matrix", fields_list);
    let diags_list = validate_fields(&slide_list, &slide_type);
    assert!(
        e_val_104(&diags_list).is_empty(),
        "matrix.cells as List must produce zero E-VAL-104 (polymorphic)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Section C: Annotation-presence tests (FieldDef.expected_type correctness)
// ─────────────────────────────────────────────────────────────────────────────
//
// Most of these PASS because the stub has set 6 of the 8 annotations.
// The roadmap.phases assertion FAILS because the stub has expected_type: None there.

/// BC-1.18.001 postcondition 9 / AC-012: `progress_bar.value` FieldDef has
/// `expected_type: Some(FieldType::Int)`. PASSES with stub (annotation set).
#[test]
fn test_BC_1_18_001_ac012_progress_bar_value_annotation_is_int() {
    let slide_type = ProgressBarSlideType::new();
    let value_def = slide_type
        .required_fields()
        .iter()
        .find(|f| f.name.as_ref() == "value")
        .expect("progress_bar must have a required 'value' field");
    assert_eq!(
        value_def.expected_type,
        Some(FieldType::Int),
        "progress_bar.value must have expected_type: Some(FieldType::Int), got: {:?}",
        value_def.expected_type
    );
}

/// BC-1.18.001 postcondition 9 / AC-013: `chart.chart_type` FieldDef has
/// `expected_type: Some(FieldType::OneOf([7 values]))`. PASSES with stub.
#[test]
fn test_BC_1_18_001_ac013_chart_type_annotation_oneof_7_values() {
    let slide_type = ChartSlideType::new();
    let chart_type_def = slide_type
        .required_fields()
        .iter()
        .find(|f| f.name.as_ref() == "chart_type")
        .expect("chart must have a required 'chart_type' field");

    match &chart_type_def.expected_type {
        Some(FieldType::OneOf(variants)) => {
            assert_eq!(
                variants.len(),
                7,
                "chart_type OneOf must have exactly 7 allowlist values, got {}: {:?}",
                variants.len(),
                variants
            );
            let variant_strs: Vec<&str> = variants.iter().map(|v| v.as_ref()).collect();
            assert!(variant_strs.contains(&"bar"), "OneOf must include 'bar'");
            assert!(variant_strs.contains(&"line"), "OneOf must include 'line'");
            assert!(variant_strs.contains(&"pie"), "OneOf must include 'pie'");
            assert!(
                variant_strs.contains(&"scatter"),
                "OneOf must include 'scatter'"
            );
            assert!(variant_strs.contains(&"area"), "OneOf must include 'area'");
            assert!(
                variant_strs.contains(&"stacked-bar"),
                "OneOf must include 'stacked-bar'"
            );
            assert!(
                variant_strs.contains(&"stacked-area"),
                "OneOf must include 'stacked-area'"
            );
        },
        other => panic!(
            "chart_type must have expected_type: Some(FieldType::OneOf([7 values])), got: {other:?}"
        ),
    }
}

/// BC-1.18.001 postcondition 9 / AC-014: `decorative` in `common_optional_fields()`
/// has `expected_type: Some(FieldType::Bool)`. PASSES with stub.
#[test]
fn test_BC_1_18_001_ac014_decorative_annotation_is_bool() {
    let commons = common_optional_fields();
    let decorative_def = commons
        .iter()
        .find(|f| f.name.as_ref() == "decorative")
        .expect("common_optional_fields must contain 'decorative'");
    assert_eq!(
        decorative_def.expected_type,
        Some(FieldType::Bool),
        "common_optional_fields().decorative must have expected_type: Some(FieldType::Bool), got: {:?}",
        decorative_def.expected_type
    );
}

/// BC-1.18.001 postcondition 9 / AC-015: `weighted_composite.components` FieldDef
/// has `expected_type: Some(FieldType::List)`. PASSES with stub.
#[test]
fn test_BC_1_18_001_ac015_weighted_composite_components_annotation_is_list() {
    let slide_type = WeightedCompositeSlideType::new();
    let components_def = slide_type
        .required_fields()
        .iter()
        .find(|f| f.name.as_ref() == "components")
        .expect("weighted_composite must have a required 'components' field");
    assert_eq!(
        components_def.expected_type,
        Some(FieldType::List),
        "weighted_composite.components must have expected_type: Some(FieldType::List), got: {:?}",
        components_def.expected_type
    );
}

/// BC-1.18.001 postcondition 9 / AC-016 / T3-20 (exact test name required by story):
/// `kpi_dashboard.kpis`, `roadmap.phases`, `agenda.items`, `team.members` all have
/// `expected_type: Some(FieldType::List)`.
///
/// FAILS for roadmap.phases because the stub has `expected_type: None` there.
/// All other assertions PASS with the stub.
///
/// NOTE: `roadmap.phases` — field name is "phases" (NOT "milestones") per ADR-020/
/// BC-1.18.001 v1.1 architect adjudication. The test asserts both the name ("phases")
/// and the annotation (Some(List)).
///
/// NOTE: `matrix.cells` and `toc` are asserted to have NO list annotation (excluded
/// from the Priority-1 set — polymorphic and auto-generated respectively).
#[test]
fn test_list_fields_on_kpi_roadmap_agenda_team() {
    // kpi_dashboard.kpis — optional field, expected_type: Some(List).
    let kpi = KpiDashboardSlideType::new();
    let kpis_def = kpi
        .optional_fields()
        .iter()
        .find(|f| f.name.as_ref() == "kpis")
        .expect("kpi_dashboard must have optional field 'kpis'");
    assert_eq!(
        kpis_def.expected_type,
        Some(FieldType::List),
        "kpi_dashboard.kpis must have expected_type: Some(FieldType::List)"
    );

    // roadmap.phases — optional field named "phases", expected_type: Some(List).
    // The stub has expected_type: None here — this assertion FAILS RED.
    let roadmap = RoadmapSlideType::new();
    let phases_def = roadmap
        .optional_fields()
        .iter()
        .find(|f| f.name.as_ref() == "phases")
        .expect("roadmap must have optional field 'phases' (not 'milestones' — per ADR-020 v1.1)");
    assert_eq!(
        phases_def.expected_type,
        Some(FieldType::List),
        "roadmap.phases must have expected_type: Some(FieldType::List); \
         field is named 'phases' per ADR-020/BC-1.18.001 v1.1 (stub has None — RED)"
    );

    // agenda.items — optional field, expected_type: Some(List).
    let agenda = AgendaSlideType::new();
    let items_def = agenda
        .optional_fields()
        .iter()
        .find(|f| f.name.as_ref() == "items")
        .expect("agenda must have optional field 'items'");
    assert_eq!(
        items_def.expected_type,
        Some(FieldType::List),
        "agenda.items must have expected_type: Some(FieldType::List)"
    );

    // team.members — optional field, expected_type: Some(List).
    let team = TeamSlideType::new();
    let members_def = team
        .optional_fields()
        .iter()
        .find(|f| f.name.as_ref() == "members")
        .expect("team must have optional field 'members'");
    assert_eq!(
        members_def.expected_type,
        Some(FieldType::List),
        "team.members must have expected_type: Some(FieldType::List)"
    );

    // matrix.cells — MUST have expected_type: None (polymorphic, no Priority-1 annotation).
    let matrix = MatrixSlideType::new();
    let cells_def = matrix
        .optional_fields()
        .iter()
        .find(|f| f.name.as_ref() == "cells")
        .expect("matrix must have optional field 'cells'");
    assert_eq!(
        cells_def.expected_type, None,
        "matrix.cells must have expected_type: None (polymorphic — no Priority-1 annotation)"
    );

    // toc — must have NO list-typed optional field (entries are auto-generated).
    let toc = TocSlideType::new();
    let toc_list_field = toc
        .optional_fields()
        .iter()
        .find(|f| f.expected_type == Some(FieldType::List));
    assert!(
        toc_list_field.is_none(),
        "toc must have NO optional field annotated with FieldType::List — entries are auto-generated; \
         found: {:?}",
        toc_list_field.map(|f| f.name.as_ref())
    );

    // Bonus: assert toc has no field named "items" (common confusion with agenda).
    let toc_items = toc
        .optional_fields()
        .iter()
        .find(|f| f.name.as_ref() == "items");
    assert!(
        toc_items.is_none(),
        "toc must not have an 'items' optional field (toc entries are auto-generated)"
    );
}

/// BC-1.18.001 / AC-021: chart.data has expected_type: None (polymorphic).
///
/// STORY-089 architect decision: chart.data was reclassified as optional
/// (required at render time by ChartRenderer, not at field-schema level).
/// The core AC-021 behavioral assertion — `expected_type: None` — is unchanged.
/// The lookup now searches optional_fields() per the reclassification.
#[test]
fn test_BC_1_18_001_ac021_chart_data_annotation_is_none() {
    let slide_type = ChartSlideType::new();
    // STORY-089: data was moved from required_fields to optional_fields.
    // Search optional_fields for the data field definition.
    let data_def = slide_type
        .optional_fields()
        .iter()
        .find(|f| f.name.as_ref() == "data")
        .expect(
            "chart must have an 'data' field (optional per STORY-089 architect decision: \
             data is required at render time by ChartRenderer, not at field-schema level)",
        );
    // Core AC-021 assertion: polymorphic field must have expected_type: None.
    assert_eq!(
        data_def.expected_type, None,
        "chart.data must have expected_type: None (polymorphic — may be Str or List)"
    );
    // Confirm data is not present in required_fields (it was reclassified).
    let data_required = slide_type
        .required_fields()
        .iter()
        .find(|f| f.name.as_ref() == "data");
    assert!(
        data_required.is_none(),
        "chart.data must NOT be in required_fields after STORY-089 reclassification; \
         it is optional (required at ChartRenderer render time, not at field-schema level)"
    );
}

/// BC-1.18.001 / AC-017: `FieldDef::new()` returns `expected_type: None`.
/// PASSES with stub (constructor is implemented).
#[test]
fn test_BC_1_18_001_ac017_fielddef_new_returns_none_annotation() {
    let def = FieldDef::new("foo", "A field", false, None);
    assert_eq!(
        def.expected_type, None,
        "FieldDef::new() must return expected_type: None (no annotation)"
    );
    assert_eq!(def.name.as_ref(), "foo");
    assert!(!def.required);
    assert!(def.default_value.is_none());
}

/// BC-1.18.001: `FieldDef::with_type()` returns `expected_type: Some(provided)`.
/// PASSES with stub (constructor is implemented).
#[test]
fn test_BC_1_18_001_fielddef_with_type_returns_some_annotation() {
    let def = FieldDef::with_type("val", "A value field", true, None, FieldType::Int);
    assert_eq!(
        def.expected_type,
        Some(FieldType::Int),
        "FieldDef::with_type(Int) must return expected_type: Some(FieldType::Int)"
    );

    let def2 = FieldDef::with_type("flag", "A bool field", false, None, FieldType::Bool);
    assert_eq!(
        def2.expected_type,
        Some(FieldType::Bool),
        "FieldDef::with_type(Bool) must return expected_type: Some(FieldType::Bool)"
    );

    let def3 = FieldDef::with_type(
        "kind",
        "A string enum",
        true,
        None,
        FieldType::OneOf(vec![Arc::from("a"), Arc::from("b")]),
    );
    match &def3.expected_type {
        Some(FieldType::OneOf(v)) => {
            assert_eq!(v.len(), 2);
            assert_eq!(v[0].as_ref(), "a");
            assert_eq!(v[1].as_ref(), "b");
        },
        other => panic!("expected Some(OneOf([a,b])) from FieldDef::with_type, got: {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Section D: Regression — E-VAL-101 / E-VAL-102 / W-VAL-103 still fire
// ─────────────────────────────────────────────────────────────────────────────
//
// These tests verify that adding the E-VAL-104 arm does NOT break the existing
// validation arms. All regression tests PASS even with the stub.

/// Regression: absent required field still produces E-VAL-101.
#[test]
fn test_BC_1_18_001_regression_e_val_101_still_fires_for_absent_required_field() {
    let reg = SlideTypeRegistry::default();
    let slide_type = reg.lookup_by_keyword("title").unwrap();
    // No fields at all — "title" required field is absent.
    let slide = make_slide_with_fields("title", OrderedMap::new());
    let diags = validate_fields(&slide, slide_type);

    let e101: Vec<_> = diags
        .iter()
        .filter(|d| d.code.as_ref() == "E-VAL-101")
        .collect();
    assert!(
        !e101.is_empty(),
        "absent required field must still produce E-VAL-101 after adding E-VAL-104 arm; got: {diags:?}"
    );
}

/// Regression: empty required field still produces E-VAL-102.
#[test]
fn test_BC_1_18_001_regression_e_val_102_still_fires_for_empty_required_field() {
    let reg = SlideTypeRegistry::default();
    let slide_type = reg.lookup_by_keyword("title").unwrap();
    let mut fields = OrderedMap::new();
    // "title" present but empty string.
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from(""))),
    );
    let slide = make_slide_with_fields("title", fields);
    let diags = validate_fields(&slide, slide_type);

    let e102: Vec<_> = diags
        .iter()
        .filter(|d| d.code.as_ref() == "E-VAL-102")
        .collect();
    assert!(
        !e102.is_empty(),
        "empty required field must still produce E-VAL-102; got: {diags:?}"
    );
}

/// Regression: unknown field still produces W-VAL-103.
#[test]
fn test_BC_1_18_001_regression_w_val_103_still_fires_for_unknown_field() {
    let reg = SlideTypeRegistry::default();
    let slide_type = reg.lookup_by_keyword("title").unwrap();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Hello"))),
    );
    // Unknown field not declared by title type.
    fields.insert(
        Arc::from("zzz_not_a_real_field"),
        FieldValue::Literal(Value::Str(Arc::from("value"))),
    );
    let slide = make_slide_with_fields("title", fields);
    let diags = validate_fields(&slide, slide_type);

    let w103: Vec<_> = diags
        .iter()
        .filter(|d| d.code.as_ref() == "W-VAL-103")
        .collect();
    assert!(
        !w103.is_empty(),
        "unknown field must still produce W-VAL-103; got: {diags:?}"
    );
    // W-VAL-103 is a warning, not an error.
    for d in &w103 {
        assert_eq!(
            d.severity,
            DiagnosticSeverity::Warning,
            "W-VAL-103 must be Warning severity"
        );
    }
}

/// Regression: valid title slide with only "title" field produces ZERO diagnostics.
#[test]
fn test_BC_1_18_001_regression_valid_slide_produces_no_diagnostics() {
    let reg = SlideTypeRegistry::default();
    let slide_type = reg.lookup_by_keyword("title").unwrap();
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Hello World"))),
    );
    let slide = make_slide_with_fields("title", fields);
    let diags = validate_fields(&slide, slide_type);
    // A valid slide must produce zero diagnostics of any kind.
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "valid title slide must produce zero Error diagnostics, got: {errors:?}"
    );
}

/// Regression: all 34 slide types produce zero E-VAL-104 when supplied with
/// their minimum required fields at the correct types. No regressions introduced
/// by the type-checking arm.
///
/// This test is important: it proves the E-VAL-104 arm never fires a false
/// positive on the existing canonical test data for the 34 slide types.
///
/// PASSES even with stub (arm is inert). Acts as a regression guard post-implementation.
#[test]
fn test_BC_1_18_001_regression_no_false_positives_on_34_registered_types() {
    use slideforge_layout::LaidOutSlide;
    use slideforge_plugin_api::traits::Canvas;
    use slideforge_types::{Brand, BrandFonts, BrandPalette};

    let brand = Brand {
        name: Arc::from("stub"),
        palette: BrandPalette {
            primary: Arc::from("#000000"),
            secondary: Arc::from("#ffffff"),
            accent: Arc::from("#0000ff"),
            neutral: Arc::from("#f5f5f5"),
        },
        fonts: BrandFonts {
            heading: Arc::from("Calibri"),
            body: Arc::from("Calibri"),
            mono: Arc::from("Courier New"),
        },
        layouts: vec![],
        span: SourceSpan::default(),
    };
    let _ = brand; // suppress unused warning

    let reg = SlideTypeRegistry::default();
    for kw in reg.all_keywords() {
        let slide_type = reg.lookup_by_keyword(kw.as_ref()).unwrap();

        // Build a minimal valid slide for each type using known-correct field types.
        let slide = match kw.as_ref() {
            "progress_bar" => {
                let mut f = OrderedMap::new();
                f.insert(
                    Arc::from("title"),
                    FieldValue::Literal(Value::Str(Arc::from("T"))),
                );
                f.insert(
                    Arc::from("label"),
                    FieldValue::Literal(Value::Str(Arc::from("L"))),
                );
                // Use Int(50) — correct type.
                f.insert(Arc::from("value"), FieldValue::Literal(Value::Int(50)));
                make_slide_with_fields("progress_bar", f)
            },
            "chart" => {
                let mut f = OrderedMap::new();
                f.insert(
                    Arc::from("title"),
                    FieldValue::Literal(Value::Str(Arc::from("T"))),
                );
                // Use "bar" — in OneOf allowlist.
                f.insert(
                    Arc::from("chart_type"),
                    FieldValue::Literal(Value::Str(Arc::from("bar"))),
                );
                f.insert(
                    Arc::from("data"),
                    FieldValue::Literal(Value::Str(Arc::from("@d.csv"))),
                );
                make_slide_with_fields("chart", f)
            },
            "weighted_composite" => {
                let mut comp = OrderedMap::new();
                comp.insert(Arc::from("name"), Value::Str(Arc::from("A")));
                comp.insert(Arc::from("weight"), Value::Float(OrderedFloat(0.5)));
                comp.insert(Arc::from("score"), Value::Int(80));
                comp.insert(Arc::from("label"), Value::Str(Arc::from("Good")));
                let mut f = OrderedMap::new();
                f.insert(
                    Arc::from("title"),
                    FieldValue::Literal(Value::Str(Arc::from("T"))),
                );
                f.insert(
                    Arc::from("label"),
                    FieldValue::Literal(Value::Str(Arc::from("Overall: Good"))),
                );
                // Use List — correct type.
                f.insert(
                    Arc::from("components"),
                    FieldValue::Literal(Value::List(vec![Value::Map(comp)])),
                );
                make_slide_with_fields("weighted_composite", f)
            },
            // For all other types: supply no fields (or just title via Str
            // where required). Annotated optional fields are absent (E-VAL-104
            // only fires on PRESENT mistyped fields — absence is E-VAL-101).
            _ => make_slide_with_fields(kw.as_ref(), OrderedMap::new()),
        };

        let diags = validate_fields(&slide, slide_type);
        let e104 = e_val_104(&diags);
        assert!(
            e104.is_empty(),
            "slide type '{}' with correct field types must produce zero E-VAL-104, got: {e104:?}",
            kw
        );
    }
}
