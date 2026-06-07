//! The `chart` slide type — a data visualization chart slide.
//!
//! A `chart` slide renders a chart (bar, line, pie, scatter, etc.) from
//! a data source using the `ChartRenderer` plugin. The `chart_type` field is
//! required. The `data` field is **optional at field-schema level**: it is
//! required at render time by the `ChartRenderer` plugin, but a chart slide
//! may be declared without `data` when using a `@data` reference resolved at
//! runtime or when rendering a layout preview. The `alt` field is optional per
//! spec (it is part of common optional fields); `decorative` can suppress it
//! when the chart is decorative only.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, FieldType, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `chart` slide type.
///
/// Required fields: `title`, `chart_type`.
/// Optional fields: `data`, `report`, `detail`, plus common optional fields.
///
/// Note: `data` is required at render time by the `ChartRenderer` plugin, but
/// is declared optional at field-schema level to allow layout previews and
/// `@data` runtime references without triggering E-VAL-101. Authors should
/// always provide `data` for charts that will be exported.
///
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
        let mut optional = vec![
            // Polymorphic: data may be a List or a Str data-source reference.
            // Optional at field-schema level (STORY-089 architect decision):
            //   - `expected_type: None` to avoid false E-VAL-104 (BC-1.18.001 invariant 5).
            //   - Required at render time by the ChartRenderer plugin (not enforced here).
            //   - Declared optional to allow layout previews and @data runtime references.
            // TODO(future story): Route A via Arc<PluginRegistry> would let ChartRenderer
            // verify data presence at a later pipeline stage without field-schema false positives.
            FieldDef {
                name: Arc::from("data"),
                description: Arc::from(
                    "Data source for the chart. May be an inline data block or a \
                     `@data` reference to an external file. Required at render time \
                     by the ChartRenderer plugin; optional at field-schema level to \
                     support layout previews and runtime @data references.",
                ),
                required: false,
                default_value: None,
                expected_type: None,
            },
        ];
        optional.extend(common_optional_fields());
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
            ],
            optional,
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
