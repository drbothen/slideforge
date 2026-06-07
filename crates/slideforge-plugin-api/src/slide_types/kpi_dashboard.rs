//! The `kpi_dashboard` slide type — a key performance indicator dashboard slide.
//!
//! A `kpi_dashboard` slide displays multiple KPI metrics in a grid layout,
//! each with a label, value, and trend indicator.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, FieldType, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `kpi_dashboard` slide type.
///
/// Required fields: `title`.
/// Optional fields: `kpis`, plus common optional fields (`report`, `detail`, etc.).
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct KpiDashboardSlideType {
    /// Required fields for the KPI dashboard slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the KPI dashboard slide type.
    optional: Vec<FieldDef>,
}

impl KpiDashboardSlideType {
    /// Construct a new `KpiDashboardSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        // Priority-1 annotation: kpis must be a list.
        // AC-016 (BC-1.18.001 postcondition 9): `kpis: "see report"` emits E-VAL-104 T1.
        let mut optional = vec![FieldDef {
            name: Arc::from("kpis"),
            description: Arc::from(
                "List of KPI metric definitions. Each entry may include label, \
                 value, target, trend, and unit. Can be provided as body blocks instead.",
            ),
            required: false,
            default_value: None,
            expected_type: Some(FieldType::List),
        }];
        optional.extend(common_optional_fields());
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from(
                    "The slide title (e.g., \"KPI Dashboard\" or \"Q1 Metrics\").",
                ),
                required: true,
                default_value: None,
                expected_type: None,
            }],
            optional,
        }
    }
}

impl Default for KpiDashboardSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for KpiDashboardSlideType {
    fn id(&self) -> &'static str {
        "kpi_dashboard"
    }

    fn required_fields(&self) -> &[FieldDef] {
        &self.required
    }

    fn optional_fields(&self) -> &[FieldDef] {
        &self.optional
    }

    fn layout_name(&self) -> &'static str {
        "Title and Content"
    }

    fn lay_out(
        &self,
        slide: &Slide,
        _brand: &Brand,
        _canvas: Canvas,
    ) -> Result<LaidOutSlide, LayoutError> {
        // Stub: returns an empty LaidOutSlide. Full geometric layout in Phase 3.
        Ok(LaidOutSlide {
            source_index: 0,
            slide_type_keyword: std::sync::Arc::clone(&slide.slide_type),
            frames: vec![],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        })
    }
}
