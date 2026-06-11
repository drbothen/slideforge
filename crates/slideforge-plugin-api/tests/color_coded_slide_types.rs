//! Geometry and registration tests for STORY-087: Color-Coded Slide Types.
//!
//! Covers BC-1.17.001 (status), BC-1.17.002 (`progress_bar`), and
//! BC-1.17.003 (`weighted_composite`) `lay_out()` geometry and registration.
//!
//! ## F-087-P1-001 adjudication change (2026-06-06)
//!
//! Per the architect adjudication for finding F-087-P1-001, `lay_out()` is now
//! GEOMETRY-ONLY. All value-range validation (value [0,100], weight>0, score [0,100])
//! has moved to `ValueRangeValidator` (Stage 5, pre-layout). Label validation has
//! moved to `LabelCheckValidator` (Stage 5). As a result:
//!
//! - `lay_out()` ALWAYS returns `Ok` for any field values (including out-of-range).
//! - The `lay_out()`-direct error tests (AC-003, AC-004, AC-009, AC-012, AC-013,
//!   AC-016, AC-017, AC-019, AC-020, AC-021) are removed from this file.
//! - Value-range enforcement is tested at `build()` level in:
//!   `crates/slideforge/tests/e2e/story_087_value_range.rs`
//! - Label enforcement is tested in:
//!   `crates/slideforge-validate/src/label_check.rs`
//!
//! ## Remaining test coverage (this file)
//!
//! | AC  | Test(s) |
//! |-----|---------|
//! | AC-001 | test_BC_1_17_001_ac001_status_keyword_registered |
//! | AC-002 | test_BC_1_17_001_ac002_status_lay_out_ok_with_valid_fields |
//! |        | test_BC_1_17_001_ac002_status_lay_out_geometry_two_frames |
//! | AC-003g | test_BC_1_17_001_ac003g_status_lay_out_geometry_only_missing_label_ok |
//! | AC-004g | test_BC_1_17_001_ac004g_status_lay_out_geometry_only_empty_label_ok |
//! | AC-007 | test_BC_1_17_002_ac007_progress_bar_keyword_registered |
//! | AC-008 | test_BC_1_17_002_ac008_progress_bar_lay_out_ok_with_valid_fields |
//! |        | test_BC_1_17_002_ac008_progress_bar_lay_out_geometry_three_frames |
//! | AC-009g | test_BC_1_17_002_ac009g_progress_bar_lay_out_geometry_only_missing_label_ok |
//! | AC-010 | test_BC_1_17_002_ac010_progress_bar_value_0_valid |
//! | AC-011 | test_BC_1_17_002_ac011_progress_bar_value_100_valid |
//! | AC-012g | test_BC_1_17_002_ac012g_progress_bar_lay_out_geometry_only_value_101_ok |
//! | AC-013g | test_BC_1_17_002_ac013g_progress_bar_lay_out_geometry_only_value_neg1_ok |
//! | AC-014 | test_BC_1_17_003_ac014_weighted_composite_keyword_registered |
//! | AC-015 | test_BC_1_17_003_ac015_weighted_composite_lay_out_ok_with_valid_fields |
//! |        | test_BC_1_17_003_ac015_weighted_composite_lay_out_geometry_min_frames |
//! | AC-016g | test_BC_1_17_003_ac016g_weighted_composite_lay_out_geometry_only_missing_label_ok |
//! | AC-017g | test_BC_1_17_003_ac017g_weighted_composite_lay_out_geometry_only_missing_component_label_ok |
//! | AC-019g | test_BC_1_17_003_ac019g_weighted_composite_lay_out_geometry_only_empty_components_ok |
//! | AC-020g | test_BC_1_17_003_ac020g_weighted_composite_lay_out_geometry_only_score_101_ok |
//! | AC-021g | test_BC_1_17_003_ac021g_weighted_composite_lay_out_geometry_only_weight_zero_ok |
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
use slideforge_plugin_api::traits::{Canvas, SlideType};
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
            font_size_emu: 457_200,
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
        field_spans: OrderedMap::new(),
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
        field_spans: OrderedMap::new(),
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
        field_spans: OrderedMap::new(),
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
// AC-003g / F-087-P1-001 adjudication: lay_out() is geometry-only
// status lay_out() with missing label must still return Ok (geometry-only)
// ─────────────────────────────────────────────────────────────────────────────

/// F-087-P1-001 adjudication: `StatusSlideType::lay_out()` is geometry-only after
/// the refactor. It must return `Ok` even when the `label` field is absent.
///
/// Label enforcement has moved to `LabelCheckValidator` at Stage 5.
/// `lay_out()` only produces the frame geometry skeleton.
///
/// This test replaces AC-003 (which tested `Err(MissingRequiredField)` from
/// `lay_out()` — no longer correct after geometry-only refactor).
#[test]
fn test_BC_1_17_001_ac003g_status_lay_out_geometry_only_missing_label_ok() {
    let st = StatusSlideType::new();
    let slide = make_slide_str("status", vec![("title", "Project Beta")]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_ok(),
        "StatusSlideType::lay_out() (geometry-only) must return Ok even with \
         missing label; label enforcement is Stage 5 (LabelCheckValidator). \
         Got: {result:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-004g / F-087-P1-001 adjudication: lay_out() is geometry-only
// status lay_out() with empty label must still return Ok (geometry-only)
// ─────────────────────────────────────────────────────────────────────────────

/// F-087-P1-001 adjudication: empty label `""` no longer causes `lay_out()` to
/// return Err — `lay_out()` is geometry-only. Label enforcement (including blank
/// label rejection) is Stage 5 (`LabelCheckValidator`).
///
/// This test replaces AC-004.
#[test]
fn test_BC_1_17_001_ac004g_status_lay_out_geometry_only_empty_label_ok() {
    let st = StatusSlideType::new();
    let slide = make_slide_str("status", vec![("title", "Project Gamma"), ("label", "")]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_ok(),
        "StatusSlideType::lay_out() (geometry-only) must return Ok even with \
         empty label; label enforcement is Stage 5 (LabelCheckValidator). \
         Got: {result:?}"
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
// AC-009g / F-087-P1-001 adjudication: lay_out() is geometry-only
// progress_bar lay_out() with missing label must return Ok (geometry-only)
// ─────────────────────────────────────────────────────────────────────────────

/// F-087-P1-001 adjudication: `ProgressBarSlideType::lay_out()` is geometry-only
/// after the refactor. Missing `label` no longer causes Err from `lay_out()`.
/// Label enforcement is Stage 5 (`LabelCheckValidator`).
///
/// This test replaces AC-009.
#[test]
fn test_BC_1_17_002_ac009g_progress_bar_lay_out_geometry_only_missing_label_ok() {
    let st = ProgressBarSlideType::new();
    let slide = make_slide_mixed(
        "progress_bar",
        vec![("title", "Sprint 4")],
        vec![("value", 75)],
    );
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_ok(),
        "ProgressBarSlideType::lay_out() (geometry-only) must return Ok even with \
         missing label; label enforcement is Stage 5 (LabelCheckValidator). \
         Got: {result:?}"
    );
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
// AC-012g / F-087-P1-001 adjudication: lay_out() is geometry-only
// progress_bar lay_out() with value=101 must return Ok (geometry-only)
// Value-range enforcement is now at Stage 5 (ValueRangeValidator).
// ─────────────────────────────────────────────────────────────────────────────

/// F-087-P1-001 adjudication: `ProgressBarSlideType::lay_out()` is geometry-only.
/// `value=101` no longer causes `lay_out()` to return Err.
///
/// Value-range enforcement (value ∈ [0,100]) has moved to `ValueRangeValidator`
/// at Stage 5 (pre-layout). The build()-level test is:
///   `test_BC_1_17_002_build_progress_bar_value_101_is_validation_failed`
///   in `crates/slideforge/tests/e2e/story_087_value_range.rs`.
///
/// This test replaces AC-012.
#[test]
fn test_BC_1_17_002_ac012g_progress_bar_lay_out_geometry_only_value_101_ok() {
    let st = ProgressBarSlideType::new();
    let slide = make_slide_mixed(
        "progress_bar",
        vec![("title", "Over"), ("label", "Done")],
        vec![("value", 101)],
    );
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_ok(),
        "ProgressBarSlideType::lay_out() (geometry-only) must return Ok even for \
         value=101; value-range enforcement is Stage 5 (ValueRangeValidator). \
         Got: {result:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-013g / F-087-P1-001 adjudication: lay_out() is geometry-only
// progress_bar lay_out() with value=-1 must return Ok (geometry-only)
// ─────────────────────────────────────────────────────────────────────────────

/// F-087-P1-001 adjudication: `value=-1` no longer causes `lay_out()` to return Err.
/// This test replaces AC-013.
#[test]
fn test_BC_1_17_002_ac013g_progress_bar_lay_out_geometry_only_value_neg1_ok() {
    let st = ProgressBarSlideType::new();
    let slide = make_slide_mixed(
        "progress_bar",
        vec![("title", "Negative"), ("label", "Done")],
        vec![("value", -1)],
    );
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_ok(),
        "ProgressBarSlideType::lay_out() (geometry-only) must return Ok even for \
         value=-1; value-range enforcement is Stage 5 (ValueRangeValidator). \
         Got: {result:?}"
    );
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
// AC-016g / F-087-P1-001 adjudication: lay_out() is geometry-only
// weighted_composite lay_out() with missing top-level label must return Ok
// ─────────────────────────────────────────────────────────────────────────────

/// F-087-P1-001 adjudication: `WeightedCompositeSlideType::lay_out()` is
/// geometry-only. Missing top-level label no longer causes Err from `lay_out()`.
/// Label enforcement is Stage 5 (`LabelCheckValidator`).
///
/// This test replaces AC-016.
#[test]
fn test_BC_1_17_003_ac016g_weighted_composite_lay_out_geometry_only_missing_label_ok() {
    let st = WeightedCompositeSlideType::new();
    let comp1 = make_component("Quality", 0.4, 85, Some("Excellent"));
    let slide = make_weighted_composite_slide(None, vec![comp1]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_ok(),
        "WeightedCompositeSlideType::lay_out() (geometry-only) must return Ok even \
         with missing top-level label; label enforcement is Stage 5 (LabelCheckValidator). \
         Got: {result:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-017g / F-087-P1-001 adjudication: lay_out() is geometry-only
// weighted_composite lay_out() with missing component label must return Ok
// ─────────────────────────────────────────────────────────────────────────────

/// F-087-P1-001 adjudication: missing component label no longer causes `lay_out()`
/// to return Err. Component label enforcement is Stage 5 (`LabelCheckValidator`
/// component iteration loop).
///
/// This test replaces AC-017.
#[test]
fn test_BC_1_17_003_ac017g_weighted_composite_lay_out_geometry_only_missing_component_label_ok() {
    let st = WeightedCompositeSlideType::new();
    let comp_ok = make_component("Quality", 0.4, 85, Some("Excellent"));
    let comp_no_label = make_component("Price", 0.6, 72, None);
    let slide = make_weighted_composite_slide(Some("Overall: Good"), vec![comp_ok, comp_no_label]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_ok(),
        "WeightedCompositeSlideType::lay_out() (geometry-only) must return Ok even \
         when a component is missing its label; label enforcement is Stage 5 \
         (LabelCheckValidator). Got: {result:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-019g / F-087-P1-001 adjudication: lay_out() is geometry-only
// weighted_composite lay_out() with empty components must return Ok
// ─────────────────────────────────────────────────────────────────────────────

/// F-087-P1-001 adjudication: empty components list no longer causes `lay_out()`
/// to return Err. `lay_out()` is geometry-only — it produces the 7-frame skeleton
/// regardless of field values.
///
/// Components validation (non-empty, weight/score range) has moved to
/// `ValueRangeValidator` at Stage 5.
///
/// This test replaces AC-019.
#[test]
fn test_BC_1_17_003_ac019g_weighted_composite_lay_out_geometry_only_empty_components_ok() {
    let st = WeightedCompositeSlideType::new();
    let slide = make_weighted_composite_slide(Some("Overall: N/A"), vec![]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_ok(),
        "WeightedCompositeSlideType::lay_out() (geometry-only) must return Ok even \
         for empty components list; components validation is Stage 5 \
         (ValueRangeValidator). Got: {result:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-020g / F-087-P1-001 adjudication: lay_out() is geometry-only
// weighted_composite lay_out() with score=101 must return Ok (geometry-only)
// Value-range enforcement is now at Stage 5 (ValueRangeValidator).
// ─────────────────────────────────────────────────────────────────────────────

/// F-087-P1-001 adjudication: `WeightedCompositeSlideType::lay_out()` is
/// geometry-only. Component `score=101` no longer causes `lay_out()` to return Err.
///
/// Value-range enforcement (score ∈ [0,100]) has moved to `ValueRangeValidator`
/// at Stage 5. The build()-level test is:
///   `test_BC_1_17_003_build_weighted_composite_score_101_is_validation_failed`
///   in `crates/slideforge/tests/e2e/story_087_value_range.rs` (currently
///   `#[ignore]`'d pending STORY-088 DSL list-literal support).
///
/// The unit-level test is:
///   `test_BC_1_17_003_score_101_error` in
///   `crates/slideforge-validate/src/value_range.rs`.
///
/// This test replaces AC-020.
#[test]
fn test_BC_1_17_003_ac020g_weighted_composite_lay_out_geometry_only_score_101_ok() {
    let st = WeightedCompositeSlideType::new();
    let comp_bad = make_component("Quality", 0.4, 101, Some("Impossible"));
    let slide = make_weighted_composite_slide(Some("Overall"), vec![comp_bad]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_ok(),
        "WeightedCompositeSlideType::lay_out() (geometry-only) must return Ok even \
         for component score=101; value-range enforcement is Stage 5 \
         (ValueRangeValidator). Got: {result:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-021g / F-087-P1-001 adjudication: lay_out() is geometry-only
// weighted_composite lay_out() with weight=0 must return Ok (geometry-only)
// ─────────────────────────────────────────────────────────────────────────────

/// F-087-P1-001 adjudication: Component `weight=0` no longer causes `lay_out()`
/// to return Err. Weight enforcement is Stage 5 (`ValueRangeValidator`).
///
/// This test replaces AC-021.
#[test]
fn test_BC_1_17_003_ac021g_weighted_composite_lay_out_geometry_only_weight_zero_ok() {
    let st = WeightedCompositeSlideType::new();
    let comp_zero = make_component("Price", 0.0, 72, Some("Acceptable"));
    let slide = make_weighted_composite_slide(Some("Overall"), vec![comp_zero]);
    let result = st.lay_out(&slide, &stub_brand(), stub_canvas());
    assert!(
        result.is_ok(),
        "WeightedCompositeSlideType::lay_out() (geometry-only) must return Ok even \
         for component weight=0; weight enforcement is Stage 5 (ValueRangeValidator). \
         Got: {result:?}"
    );
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
