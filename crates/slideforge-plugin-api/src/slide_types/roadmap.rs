//! The `roadmap` slide type — a product or project roadmap slide.
//!
//! A `roadmap` slide displays planned phases, features, or milestones over
//! time. Phases can be provided as structured data or as body blocks.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

/// The built-in `roadmap` slide type.
///
/// Required fields: `title`.
/// Optional fields: `phases`, `notes`, `footer`, `logo`, `tags`.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct RoadmapSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl RoadmapSlideType {
    /// Construct a new `RoadmapSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from(
                    "The slide title (e.g., \"Product Roadmap\" or \"Q1-Q4 Roadmap\").",
                ),
                required: true,
                default_value: None,
            }],
            optional: vec![
                FieldDef {
                    name: Arc::from("phases"),
                    description: Arc::from(
                        "Roadmap phase definitions. Each phase may include label, \
                         timeframe, and items. Can be provided as body blocks instead.",
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

impl Default for RoadmapSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for RoadmapSlideType {
    fn id(&self) -> &'static str {
        "roadmap"
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
