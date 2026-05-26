//! The `chart` slide type — a data visualization chart slide.
//!
//! A `chart` slide renders a chart (bar, line, pie, scatter, etc.) from
//! a data source using the `ChartRenderer` plugin. The `chart_type` and
//! `data` fields are required, as is `alt` for WCAG AA compliance.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

/// The built-in `chart` slide type.
///
/// Required fields: `title`, `chart_type`, `data`, `alt`.
/// Optional fields: `notes`, `footer`, `logo`, `decorative`, `tags`.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct ChartSlideType {
    required: Vec<FieldDef>,
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
                },
                FieldDef {
                    name: Arc::from("chart_type"),
                    description: Arc::from(
                        "The chart type to render: bar, line, pie, scatter, area, \
                         stacked-bar, stacked-area.",
                    ),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("data"),
                    description: Arc::from(
                        "Data source for the chart. May be an inline data block or a \
                         `@data` reference to an external file.",
                    ),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("alt"),
                    description: Arc::from(
                        "Accessibility alt text describing the chart and its key insight. \
                         Required for WCAG AA compliance.",
                    ),
                    required: true,
                    default_value: None,
                },
            ],
            optional: vec![
                FieldDef {
                    name: Arc::from("decorative"),
                    description: Arc::from(
                        "When true, marks the chart as decorative (empty alt text in output). \
                         Overrides the `alt` field.",
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
