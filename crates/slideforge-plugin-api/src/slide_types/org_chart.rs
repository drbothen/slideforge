//! The `org-chart` slide type — an organizational chart slide.
//!
//! An `org-chart` slide displays a hierarchical organization chart.
//! Nodes can be provided as a structured list or as Mermaid diagram syntax
//! via the body.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

/// The built-in `org-chart` slide type.
///
/// Required fields: `title`.
/// Optional fields: `nodes`, `notes`, `footer`, `logo`, `tags`.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct OrgChartSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl OrgChartSlideType {
    /// Construct a new `OrgChartSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from("The slide title (e.g., \"Organizational Structure\")."),
                required: true,
                default_value: None,
            }],
            optional: vec![
                FieldDef {
                    name: Arc::from("nodes"),
                    description: Arc::from(
                        "Hierarchical node definitions for the org chart. Each node may \
                         include name, title, image, and parent reference. Can be provided \
                         as a Mermaid diagram in the body instead.",
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

impl Default for OrgChartSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for OrgChartSlideType {
    fn id(&self) -> &'static str {
        "org-chart"
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
