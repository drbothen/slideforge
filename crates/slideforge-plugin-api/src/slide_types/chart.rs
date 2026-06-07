//! The `chart` slide type — a data visualization chart slide.
//!
//! A `chart` slide renders a chart (bar, line, pie, scatter, etc.) from
//! a data source using the `ChartRenderer` plugin. The `chart_type` and
//! `data` fields are required. The `alt` field is optional per spec (it is
//! part of common optional fields); `decorative` can suppress it when the
//! chart is decorative only.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, FieldType, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `chart` slide type.
///
/// Required fields: `title`, `chart_type`, `data`.
/// Optional fields: `report`, `detail`, plus common optional fields.
/// Note: `alt` is included via common optional fields. For WCAG AA compliance,
/// authors should provide `alt` on every non-decorative chart.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct ChartSlideType {
    /// Required fields for the chart slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the chart slide type.
    optional: Vec<FieldDef>,
}

impl ChartSlideType {
    /// Construct a new `ChartSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from("The slide title shown above the chart."),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
                // Priority-1 annotation: chart_type must be one of the 7 allowed values.
                // AC-013 (BC-1.18.001 postcondition 9):
                //   `chart_type: "donut"` emits E-VAL-104 T2 (disallowed value).
                //   `chart_type: 42` emits E-VAL-104 T1 (expected string, got integer).
                FieldDef {
                    name: Arc::from("chart_type"),
                    description: Arc::from(
                        "The chart type to render: bar, line, pie, scatter, area, \
                         stacked-bar, stacked-area.",
                    ),
                    required: true,
                    default_value: None,
                    expected_type: Some(FieldType::OneOf(vec![
                        Arc::from("bar"),
                        Arc::from("line"),
                        Arc::from("pie"),
                        Arc::from("scatter"),
                        Arc::from("area"),
                        Arc::from("stacked-bar"),
                        Arc::from("stacked-area"),
                    ])),
                },
                // Polymorphic: data may be a List or a Str data-source reference.
                // AC-021 (BC-1.18.001 invariant 5): must be None to avoid false E-VAL-104.
                FieldDef {
                    name: Arc::from("data"),
                    description: Arc::from(
                        "Data source for the chart. May be an inline data block or a \
                         `@data` reference to an external file.",
                    ),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
            ],
            optional: common_optional_fields(),
        }
    }
}

impl Default for ChartSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for ChartSlideType {
    fn id(&self) -> &'static str {
        "chart"
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
