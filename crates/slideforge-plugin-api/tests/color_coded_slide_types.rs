//! Red Gate integration tests for STORY-087: Color-Coded Slide Types.
//!
//! Covers BC-1.17.001 (status), BC-1.17.002 (`progress_bar`), and
//! BC-1.17.003 (`weighted_composite`) `lay_out()` and registration behaviors.
//!
//! ALL tests marked "FAILS at Red Gate" will panic with `todo!()` until the
//! `lay_out()` implementations are filled in by the implementer.
//!
//! `LabelCheck` tests (AC-006, AC-018, AC-022, F-G3-HIGH-003, NFR-021/022/023)
//! live in `crates/slideforge-validate/src/label_check.rs` (correct crate per
//! project convention: validator tests live near the validator impl).
//!
//! Keyword consistency tests (AC-024) live in
//! `crates/slideforge-syntax/src/keywords.rs`.
//!
//! # AC coverage (this file)
//!
//! | AC  | Test(s) |
//! |-----|---------|
//! | AC-001 | test_BC_1_17_001_ac001_status_keyword_registered |
//! | AC-002 | test_BC_1_17_001_ac002_status_lay_out_ok_with_valid_fields |
//! |        | test_BC_1_17_001_ac002_status_lay_out_geometry_two_frames |
//! | AC-003 | test_BC_1_17_001_ac003_status_lay_out_missing_label_errors |
//! | AC-004 | test_BC_1_17_001_ac004_status_lay_out_empty_label_errors |
//! | AC-007 | test_BC_1_17_002_ac007_progress_bar_keyword_registered |
//! | AC-008 | test_BC_1_17_002_ac008_progress_bar_lay_out_ok_with_valid_fields |
//! |        | test_BC_1_17_002_ac008_progress_bar_lay_out_geometry_three_frames |
//! | AC-009 | test_BC_1_17_002_ac009_progress_bar_missing_label_errors |
//! | AC-010 | test_BC_1_17_002_ac010_progress_bar_value_0_valid |
//! | AC-011 | test_BC_1_17_002_ac011_progress_bar_value_100_valid |
//! | AC-012 | test_BC_1_17_002_ac012_progress_bar_value_101_errors |
//! | AC-013 | test_BC_1_17_002_ac013_progress_bar_value_minus1_errors |
//! | AC-014 | test_BC_1_17_003_ac014_weighted_composite_keyword_registered |
//! | AC-015 | test_BC_1_17_003_ac015_weighted_composite_lay_out_ok_with_valid_fields |
//! |        | test_BC_1_17_003_ac015_weighted_composite_lay_out_geometry_min_frames |
//! | AC-016 | test_BC_1_17_003_ac016_weighted_composite_missing_top_label_errors |
//! | AC-017 | test_BC_1_17_003_ac017_weighted_composite_missing_component_label_errors |
//! | AC-019 | test_BC_1_17_003_ac019_weighted_composite_empty_components_errors |
//! | AC-020 | test_BC_1_17_003_ac020_weighted_composite_score_101_errors |
//! | AC-021 | test_BC_1_17_003_ac021_weighted_composite_weight_zero_errors |
//! | AC-023 | test_BC_1_17_001_ac023_all_three_types_registered_not_unknown |
//! |        | test_BC_1_17_001_registration_count_34_after_story_087 |

#![allow(non_snake_case)] // BC-traceability IDs use uppercase: test_BC_S_SS_NNN_xxx
#![allow(clippy::unwrap_used)] // Test code may use unwrap per project convention

use std::sync::Arc;

use ordered_float::OrderedFloat;
use slideforge_plugin_api::slide_types::{
    SlideTypeRegistry, progress_bar::ProgressBarSlideType, status::StatusSlideType,
    weighted_composite::WeightedCompositeSlideType,
};
use slideforge_plugin_api::traits::{Canvas, LayoutError, SlideType};
use slideforge_types::{
    Brand, BrandFonts, BrandPalette, FieldValue, OrderedMap, Slide, SourceSpan, Value,
};

// ─── Test helpers ────────────────────────────────────────────────────────────

fn stub_brand() -> Brand {
    Brand {
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
    }
}

fn stub_canvas() -> Canvas {
    Canvas::default()
}

/// Build a `Slide` with the given `slide_type` and string field pairs.
fn make_slide_str(slide_type: &str, fields: Vec<(&str, &str)>) -> Slide {
    let mut field_map = OrderedMap::new();
    for (k, v) in fields {
        field_map.insert(Arc::from(k), FieldValue::Literal(Value::Str(Arc::from(v))));
    }
    Slide {
        slide_type: Arc::from(slide_type),
        fields: field_map,
        blocks: vec![],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
    }
}

/// Build a `Slide` with a mix of string and integer fields.
fn make_slide_mixed(
    slide_type: &str,
    str_fields: Vec<(&str, &str)>,
    int_fields: Vec<(&str, i64)>,
) -> Slide {
    let mut field_map = OrderedMap::new();
    for (k, v) in str_fields {
        field_map.insert(Arc::from(k), FieldValue::Literal(Value::Str(Arc::from(v))));
    }
    for (k, v) in int_fields {
        field_map.insert(Arc::from(k), FieldValue::Literal(Value::Int(v)));
    }
    Slide {
        slide_type: Arc::from(slide_type),
        fields: field_map,
        blocks: vec![],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
    }
}

/// Build a component map with required fields.
/// `weight` is a float > 0; `score` is an integer [0, 100]; `label` is optional.
fn make_component(
    name: &str,
    weight: f64,
    score: i64,
    label: Option<&str>,
) -> OrderedMap<Arc<str>, Value> {
    let mut m = OrderedMap::new();
    m.insert(Arc::from("name"), Value::Str(Arc::from(name)));
    m.insert(Arc::from("weight"), Value::Float(OrderedFloat(weight)));
    m.insert(Arc::from("score"), Value::Int(score));
    if let Some(lbl) = label {
        m.insert(Arc::from("label"), Value::Str(Arc::from(lbl)));
    }
    m
}

/// Build a `weighted_composite` slide with optional top-level label and components.
fn make_weighted_composite_slide(
    top_label: Option<&str>,
    components: Vec<OrderedMap<Arc<str>, Value>>,
) -> Slide {
    let mut field_map = OrderedMap::new();
    field_map.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Vendor A Scorecard"))),
    );
    if let Some(lbl) = top_label {
        field_map.insert(
            Arc::from("label"),
            FieldValue::Literal(Value::Str(Arc::from(lbl))),
        );
    }
    let component_list: Vec<Value> = components.into_iter().map(Value::Map).collect();
    field_map.insert(
        Arc::from("components"),
        FieldValue::Literal(Value::List(component_list)),
    );
    Slide {
        slide_type: Arc::from("weighted_composite"),
        fields: field_map,
        blocks: vec![],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-001 / BC-1.17.001 precondition 3 / invariant 5
// status keyword registration and schema
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.001 AC-001: `"status"` is registered in `SlideTypeRegistry::default()`.
/// Verifies `id()`, `required_fields()` (title, label — exactly 2), and `layout_name()`.
#[test]
fn test_BC_1_17_001_ac001_status_keyword_registered() {
    let reg = SlideTypeRegistry::default();
    let st = reg
        .lookup_by_keyword("status")
        .expect("'status' must be registered in SlideTypeRegistry::default()");

    assert_eq!(
        st.id(),
        "status",
        "StatusSlideType::id() must return \"status\""
    );

    let req: Vec<&str> = st
        .required_fields()
        .iter()
        .map(|f| f.name.as_ref())
        .collect();
    assert!(
        req.contains(&"title"),
        "status required_fields must include 'title'; got: {req:?}"
    );
    assert!(
        req.contains(&"label"),
        "status required_fields must include 'label' (WCAG 1.4.1); got: {req:?}"
    );
    assert_eq!(
        req.len(),
        2,
        "status must have exactly 2 required fields (title, label); got {req:?}"
    );
    assert!(
        !st.layout_name().is_empty(),
        "layout_name() must return a non-empty string"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-002 / BC-1.17.001 postcondition 2+3
// status lay_out() happy path
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.001 AC-002: `StatusSlideType::lay_out()` with valid title + label
/// returns `Ok(LaidOutSlide)`.
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_001_ac002_status_lay_out_ok_with_valid_fields() {
    let st = StatusSlideType::new();
    let slide = make_slide_str(
        "status",
        vec![("title", "Project Alpha"), ("label", "On Track")],
    );
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_ok(),
        "StatusSlideType::lay_out() with valid fields must return Ok; got: {result:?}"
    );
}

/// BC-1.17.001 AC-002 geometry: `lay_out()` produces at least 2 frames —
/// one color indicator (Generic role) and one body frame (Body role).
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_001_ac002_status_lay_out_geometry_two_frames() {
    use slideforge_layout::types::RegionRole;

    let st = StatusSlideType::new();
    let slide = make_slide_str(
        "status",
        vec![("title", "Project Alpha"), ("label", "On Track")],
    );
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas()).unwrap();

    assert!(
        result.frames.len() >= 2,
        "StatusSlideType lay_out() must produce >= 2 frames (indicator + body); got {}",
        result.frames.len()
    );
    let has_body = result
        .frames
        .iter()
        .any(|f| f.region_role == Some(RegionRole::Body));
    assert!(
        has_body,
        "must have at least one Body-role frame for the label"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-003 / BC-1.17.001 postcondition 2 / EC-001
// status lay_out() missing label → Err(MissingRequiredField)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.001 AC-003: `StatusSlideType::lay_out()` with missing `label`
/// returns `Err(LayoutError::MissingRequiredField { field: "label" })`.
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_001_ac003_status_lay_out_missing_label_errors() {
    let st = StatusSlideType::new();
    let slide = make_slide_str("status", vec![("title", "Project Beta")]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());

    assert!(
        result.is_err(),
        "StatusSlideType::lay_out() with missing label must return Err; got Ok"
    );
    match result.unwrap_err() {
        LayoutError::MissingRequiredField { slide_type, field } => {
            assert_eq!(slide_type, "status", "slide_type in error must be 'status'");
            assert_eq!(field, "label", "missing field must be 'label'");
        },
        other => panic!(
            "Expected LayoutError::MissingRequiredField {{ field: 'label' }}, got: {other:?}"
        ),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-004 / BC-1.17.001 EC-004
// status lay_out() empty label → Err
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.001 AC-004: Empty label `""` is equivalent to absent — must return Err.
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_001_ac004_status_lay_out_empty_label_errors() {
    let st = StatusSlideType::new();
    let slide = make_slide_str("status", vec![("title", "Project Gamma"), ("label", "")]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_err(),
        "StatusSlideType::lay_out() with empty label must return Err; got Ok"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-007 / BC-1.17.002 precondition 3 / invariant 6
// progress_bar keyword registration
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.002 AC-007: `"progress_bar"` is registered.
/// Verifies `id()`, `required_fields()` (title, label, value — exactly 3).
#[test]
fn test_BC_1_17_002_ac007_progress_bar_keyword_registered() {
    let reg = SlideTypeRegistry::default();
    let st = reg
        .lookup_by_keyword("progress_bar")
        .expect("'progress_bar' must be registered in SlideTypeRegistry::default()");

    assert_eq!(st.id(), "progress_bar");

    let req: Vec<&str> = st
        .required_fields()
        .iter()
        .map(|f| f.name.as_ref())
        .collect();
    assert!(req.contains(&"title"), "must include 'title'; got: {req:?}");
    assert!(req.contains(&"label"), "must include 'label'; got: {req:?}");
    assert!(req.contains(&"value"), "must include 'value'; got: {req:?}");
    assert_eq!(
        req.len(),
        3,
        "must have exactly 3 required fields; got {req:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-008 / BC-1.17.002 postcondition 4
// progress_bar lay_out() happy path (value=75)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.002 AC-008: `ProgressBarSlideType::lay_out()` with title + label + value=75
/// returns `Ok`.
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_002_ac008_progress_bar_lay_out_ok_with_valid_fields() {
    let st = ProgressBarSlideType::new();
    let slide = make_slide_mixed(
        "progress_bar",
        vec![("title", "Sprint 4"), ("label", "75% complete")],
        vec![("value", 75)],
    );
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_ok(),
        "ProgressBarSlideType::lay_out() with valid fields must return Ok; got: {result:?}"
    );
}

/// BC-1.17.002 AC-008 geometry: `lay_out()` produces at least 3 frames —
/// title (Title) + bar background (Generic) + label text (Body).
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_002_ac008_progress_bar_lay_out_geometry_three_frames() {
    use slideforge_layout::types::RegionRole;

    let st = ProgressBarSlideType::new();
    let slide = make_slide_mixed(
        "progress_bar",
        vec![("title", "Completion"), ("label", "75% done")],
        vec![("value", 75)],
    );
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas()).unwrap();

    assert!(
        result.frames.len() >= 3,
        "ProgressBarSlideType lay_out() must produce >= 3 frames; got {}",
        result.frames.len()
    );
    let has_title = result
        .frames
        .iter()
        .any(|f| f.region_role == Some(RegionRole::Title));
    assert!(has_title, "must have Title-role frame");

    let has_body = result
        .frames
        .iter()
        .any(|f| f.region_role == Some(RegionRole::Body));
    assert!(has_body, "must have Body-role frame (label)");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-009 / BC-1.17.002 postcondition 2 / EC-001
// progress_bar missing label → Err
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.002 AC-009: Missing label field → `Err(MissingRequiredField { field: "label" })`.
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_002_ac009_progress_bar_missing_label_errors() {
    let st = ProgressBarSlideType::new();
    let slide = make_slide_mixed(
        "progress_bar",
        vec![("title", "Sprint 4")],
        vec![("value", 75)],
    );
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());

    assert!(result.is_err(), "missing label must return Err");
    match result.unwrap_err() {
        LayoutError::MissingRequiredField { slide_type, field } => {
            assert_eq!(slide_type, "progress_bar");
            assert_eq!(field, "label");
        },
        other => panic!("Expected MissingRequiredField{{label}}, got: {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-010 / BC-1.17.002 EC-005 — value=0 is valid boundary
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.002 AC-010: `value=0` is within [0,100] → `Ok`.
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_002_ac010_progress_bar_value_0_valid() {
    let st = ProgressBarSlideType::new();
    let slide = make_slide_mixed(
        "progress_bar",
        vec![("title", "Start"), ("label", "0% done")],
        vec![("value", 0)],
    );
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_ok(),
        "value=0 must return Ok (valid boundary); got: {result:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-011 / BC-1.17.002 EC-006 — value=100 is valid boundary
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.002 AC-011: `value=100` is within [0,100] → `Ok`.
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_002_ac011_progress_bar_value_100_valid() {
    let st = ProgressBarSlideType::new();
    let slide = make_slide_mixed(
        "progress_bar",
        vec![("title", "Done"), ("label", "100% complete")],
        vec![("value", 100)],
    );
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_ok(),
        "value=100 must return Ok (valid boundary); got: {result:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-012 / BC-1.17.002 postcondition 3 / EC-003
// progress_bar value=101 → Err(FieldTypeMismatch { field: "value" })
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.002 AC-012: `value=101` → `Err(LayoutError::FieldTypeMismatch)`.
/// `expected_type` must describe [0,100]; `actual_type` must mention 101.
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_002_ac012_progress_bar_value_101_errors() {
    let st = ProgressBarSlideType::new();
    let slide = make_slide_mixed(
        "progress_bar",
        vec![("title", "Over"), ("label", "Done")],
        vec![("value", 101)],
    );
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());

    assert!(result.is_err(), "value=101 must return Err; got Ok");
    match result.unwrap_err() {
        LayoutError::FieldTypeMismatch {
            slide_type,
            field,
            expected_type,
            actual_type,
        } => {
            assert_eq!(slide_type, "progress_bar");
            assert_eq!(field, "value");
            assert!(
                expected_type.contains('0') && expected_type.contains("100"),
                "expected_type must describe [0,100]; got: {expected_type}"
            );
            assert!(
                actual_type.contains("101"),
                "actual_type must mention 101; got: {actual_type}"
            );
        },
        other => panic!("Expected FieldTypeMismatch for value=101, got: {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-013 / BC-1.17.002 EC-004
// progress_bar value=-1 → Err(FieldTypeMismatch)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.002 AC-013: `value=-1` → `Err(LayoutError::FieldTypeMismatch)`.
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_002_ac013_progress_bar_value_minus1_errors() {
    let st = ProgressBarSlideType::new();
    let slide = make_slide_mixed(
        "progress_bar",
        vec![("title", "Negative"), ("label", "Done")],
        vec![("value", -1)],
    );
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());

    assert!(result.is_err(), "value=-1 must return Err; got Ok");
    match result.unwrap_err() {
        LayoutError::FieldTypeMismatch {
            field, actual_type, ..
        } => {
            assert_eq!(field, "value");
            assert!(
                actual_type.contains("-1"),
                "actual_type must mention -1; got: {actual_type}"
            );
        },
        other => panic!("Expected FieldTypeMismatch for value=-1, got: {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-014 / BC-1.17.003 precondition 3 / invariant 2
// weighted_composite keyword registration
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.003 AC-014: `"weighted_composite"` is registered.
/// Verifies `id()`, `required_fields()` (title, label, components — exactly 3).
#[test]
fn test_BC_1_17_003_ac014_weighted_composite_keyword_registered() {
    let reg = SlideTypeRegistry::default();
    let st = reg
        .lookup_by_keyword("weighted_composite")
        .expect("'weighted_composite' must be registered in SlideTypeRegistry::default()");

    assert_eq!(st.id(), "weighted_composite");

    let req: Vec<&str> = st
        .required_fields()
        .iter()
        .map(|f| f.name.as_ref())
        .collect();
    assert!(req.contains(&"title"), "must include 'title'");
    assert!(req.contains(&"label"), "must include 'label'");
    assert!(req.contains(&"components"), "must include 'components'");
    assert_eq!(
        req.len(),
        3,
        "must have exactly 3 required fields; got {req:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-015 / BC-1.17.003 postcondition 6
// weighted_composite lay_out() happy path
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.003 AC-015: `WeightedCompositeSlideType::lay_out()` with all valid
/// fields returns `Ok`.
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_003_ac015_weighted_composite_lay_out_ok_with_valid_fields() {
    let st = WeightedCompositeSlideType::new();
    let comp1 = make_component("Quality", 0.4, 85, Some("Excellent"));
    let comp2 = make_component("Price", 0.6, 72, Some("Acceptable"));
    let slide = make_weighted_composite_slide(Some("Overall: Good (78/100)"), vec![comp1, comp2]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_ok(),
        "WeightedCompositeSlideType::lay_out() with valid fields must return Ok; got: {result:?}"
    );
}

/// BC-1.17.003 AC-015 geometry: `lay_out()` produces at least 7 frames —
/// title + aggregate label + 5 component row slots.
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_003_ac015_weighted_composite_lay_out_geometry_min_frames() {
    use slideforge_layout::types::RegionRole;

    let st = WeightedCompositeSlideType::new();
    let comp1 = make_component("Quality", 0.4, 85, Some("Excellent"));
    let comp2 = make_component("Price", 0.6, 72, Some("Acceptable"));
    let slide = make_weighted_composite_slide(Some("Overall: Good"), vec![comp1, comp2]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas()).unwrap();

    // Static skeleton: title + agg-label + 5 component slots = 7 frames minimum
    assert!(
        result.frames.len() >= 7,
        "weighted_composite lay_out() must produce >= 7 frames; got {}",
        result.frames.len()
    );
    let has_title = result
        .frames
        .iter()
        .any(|f| f.region_role == Some(RegionRole::Title));
    assert!(has_title, "must have Title-role frame");
    let has_body = result
        .frames
        .iter()
        .any(|f| f.region_role == Some(RegionRole::Body));
    assert!(has_body, "must have Body-role frame (aggregate label)");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-016 / BC-1.17.003 postcondition 2 / EC-001
// weighted_composite lay_out() missing top-level label → Err
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.003 AC-016: Missing top-level label → `Err(MissingRequiredField { field: "label" })`.
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_003_ac016_weighted_composite_missing_top_label_errors() {
    let st = WeightedCompositeSlideType::new();
    let comp1 = make_component("Quality", 0.4, 85, Some("Excellent"));
    let slide = make_weighted_composite_slide(None, vec![comp1]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());

    assert!(result.is_err(), "missing top-level label must return Err");
    match result.unwrap_err() {
        LayoutError::MissingRequiredField { slide_type, field } => {
            assert_eq!(slide_type, "weighted_composite");
            assert_eq!(field, "label");
        },
        other => panic!("Expected MissingRequiredField{{label}}, got: {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-017 / BC-1.17.003 postcondition 4 / EC-002
// weighted_composite lay_out() missing component label → Err
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.003 AC-017: Component missing `label` sub-field returns Err.
/// Error must identify `"weighted_composite"` and reference "label" or "component".
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_003_ac017_weighted_composite_missing_component_label_errors() {
    let st = WeightedCompositeSlideType::new();
    let comp_ok = make_component("Quality", 0.4, 85, Some("Excellent"));
    let comp_no_label = make_component("Price", 0.6, 72, None);
    let slide = make_weighted_composite_slide(Some("Overall: Good"), vec![comp_ok, comp_no_label]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());

    assert!(result.is_err(), "component missing label must return Err");
    let err = result.unwrap_err();
    let (err_slide_type, err_field) = match &err {
        LayoutError::MissingRequiredField { slide_type, field }
        | LayoutError::FieldTypeMismatch {
            slide_type, field, ..
        } => (slide_type.as_str(), field.as_str()),
        other => panic!("Expected error for missing component label, got: {other:?}"),
    };
    assert_eq!(err_slide_type, "weighted_composite");
    assert!(
        err_field.contains("label") || err_field.contains("component"),
        "field must reference label/component; got: {err_field}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-019 / BC-1.17.003 postcondition 3 / EC-004
// weighted_composite lay_out() empty components list → Err
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.003 AC-019: `components: []` (empty list) must return Err.
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_003_ac019_weighted_composite_empty_components_errors() {
    let st = WeightedCompositeSlideType::new();
    let slide = make_weighted_composite_slide(Some("Overall: N/A"), vec![]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(result.is_err(), "empty components list must return Err");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-020 / BC-1.17.003 invariant 7 / EC-007
// weighted_composite component score=101 → Err(FieldTypeMismatch)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.003 AC-020: Component `score=101` → `Err(LayoutError::FieldTypeMismatch)`.
/// `expected_type` must describe [0,100]; `actual_type` must mention 101.
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_003_ac020_weighted_composite_score_101_errors() {
    let st = WeightedCompositeSlideType::new();
    let comp_bad = make_component("Quality", 0.4, 101, Some("Impossible"));
    let slide = make_weighted_composite_slide(Some("Overall"), vec![comp_bad]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());

    assert!(result.is_err(), "score=101 must return Err");
    match result.unwrap_err() {
        LayoutError::FieldTypeMismatch {
            slide_type,
            field,
            expected_type,
            actual_type,
        } => {
            assert_eq!(slide_type, "weighted_composite");
            assert!(
                field.contains("score") || field.contains("component"),
                "field must identify score/component; got: {field}"
            );
            assert!(
                expected_type.contains('0') && expected_type.contains("100"),
                "expected_type must describe [0,100]; got: {expected_type}"
            );
            assert!(
                actual_type.contains("101"),
                "actual_type must mention 101; got: {actual_type}"
            );
        },
        other => panic!("Expected FieldTypeMismatch for score=101, got: {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-021 / BC-1.17.003 invariant 6 / EC-006
// weighted_composite component weight=0 → Err(FieldTypeMismatch)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-1.17.003 AC-021: Component `weight=0` (non-positive) → `Err(FieldTypeMismatch)`.
///
/// FAILS at Red Gate: `lay_out()` is `todo!()` → panic.
#[test]
fn test_BC_1_17_003_ac021_weighted_composite_weight_zero_errors() {
    let st = WeightedCompositeSlideType::new();
    let comp_zero = make_component("Price", 0.0, 72, Some("Acceptable"));
    let slide = make_weighted_composite_slide(Some("Overall"), vec![comp_zero]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());

    assert!(result.is_err(), "weight=0 must return Err");
    match result.unwrap_err() {
        LayoutError::FieldTypeMismatch {
            slide_type, field, ..
        } => {
            assert_eq!(slide_type, "weighted_composite");
            assert!(
                field.contains("weight") || field.contains("component"),
                "field must identify weight/component; got: {field}"
            );
        },
        other => panic!("Expected FieldTypeMismatch for weight=0, got: {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-023 / F-G3-HIGH-003
// All three types registered — no UnknownSlideType path
// ─────────────────────────────────────────────────────────────────────────────

/// AC-023 / F-G3-HIGH-003: All three types are registered in `SlideTypeRegistry`.
/// `lookup_by_keyword` returns `Some` — the `UnknownSlideType` error path is closed.
///
/// The structural registration check does NOT call `lay_out()`. This half of AC-023
/// passes at Red Gate (stubs are wired). The `lay_out()`-calling half is covered by
/// the happy-path tests above (which fail at Red Gate with `todo!()` panic).
#[test]
fn test_BC_1_17_001_ac023_all_three_types_registered_not_unknown() {
    let reg = SlideTypeRegistry::default();

    let st_status = reg.lookup_by_keyword("status");
    let st_pb = reg.lookup_by_keyword("progress_bar");
    let st_wc = reg.lookup_by_keyword("weighted_composite");

    assert!(
        st_status.is_some(),
        "'status' must be registered — absence would cause UnknownSlideType"
    );
    assert!(
        st_pb.is_some(),
        "'progress_bar' must be registered — absence would cause UnknownSlideType"
    );
    assert!(
        st_wc.is_some(),
        "'weighted_composite' must be registered — absence would cause UnknownSlideType"
    );

    // Verify lay_out() does NOT return an UnknownSlideType-like InternalError.
    // At Red Gate the todo!() panic is acceptable; after implementation Ok is expected.
    // We verify via the direct happy-path tests above. This assertion is structural.
    assert_eq!(st_status.unwrap().id(), "status");
    assert_eq!(st_pb.unwrap().id(), "progress_bar");
    assert_eq!(st_wc.unwrap().id(), "weighted_composite");
}

// ─────────────────────────────────────────────────────────────────────────────
// Registration count
// ─────────────────────────────────────────────────────────────────────────────

/// After STORY-087: `SlideTypeRegistry::default()` must register exactly 34 types
/// (31 original + status + `progress_bar` + `weighted_composite`).
/// `severity_cards` is a keyword but is NOT registered as a `SlideType` impl.
#[test]
fn test_BC_1_17_001_registration_count_34_after_story_087() {
    let reg = SlideTypeRegistry::default();
    let count = reg.all_keywords().len();
    assert_eq!(
        count, 34,
        "SlideTypeRegistry::default must register exactly 34 slide types after STORY-087; \
         got {count}"
    );
}
