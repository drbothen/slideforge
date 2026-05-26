//! The `kpi-dashboard` slide type — a key performance indicator dashboard slide.
//!
//! A `kpi-dashboard` slide displays multiple KPI metrics in a grid layout,
//! each with a label, value, and trend indicator.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

/// The built-in `kpi-dashboard` slide type.
///
/// Required fields: `title`.
/// Optional fields: `kpis`, `notes`, `footer`, `logo`, `tags`.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct KpiDashboardSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl KpiDashboardSlideType {
    /// Construct a new `KpiDashboardSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from(
                    "The slide title (e.g., \"KPI Dashboard\" or \"Q1 Metrics\").",
                ),
                required: true,
                default_value: None,
            }],
            optional: vec![
                FieldDef {
                    name: Arc::from("kpis"),
                    description: Arc::from(
                        "List of KPI metric definitions. Each entry may include label, \
                         value, target, trend, and unit. Can be provided as body blocks instead.",
                    ),
                    required: false,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("notes"),
                    description: Arc::from("Presenter notes for this slide."),
                    required: false,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("footer"),
                    description: Arc::from("Override footer text for this slide."),
                    required: false,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("logo"),
                    description: Arc::from("Override the brand logo for this slide."),
                    required: false,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("tags"),
                    description: Arc::from("User-defined tags for filtering and grouping."),
                    required: false,
                    default_value: None,
                },
            ],
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
        "kpi-dashboard"
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
        _slide: &Slide,
        _brand: &Brand,
        _canvas: Canvas,
    ) -> Result<LaidOutSlide, LayoutError> {
        // Stub: returns an empty LaidOutSlide. Full geometric layout in Phase 3.
        Ok(LaidOutSlide {
            width: SLIDE_WIDTH,
            height: SLIDE_HEIGHT,
            elements: vec![],
            slide_index: 0,
        })
    }
}
