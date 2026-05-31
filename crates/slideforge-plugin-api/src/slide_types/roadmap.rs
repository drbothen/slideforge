//! The `roadmap` slide type — a product or project roadmap slide.
//!
//! A `roadmap` slide displays planned phases, features, or milestones over
//! time. Phases can be provided as structured data or as body blocks.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `roadmap` slide type.
///
/// Required fields: `title`.
/// Optional fields: `phases`, plus common optional fields (`report`, `detail`, etc.).
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct RoadmapSlideType {
    /// Required fields for the roadmap slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the roadmap slide type.
    optional: Vec<FieldDef>,
}

impl RoadmapSlideType {
    /// Construct a new `RoadmapSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("phases"),
            description: Arc::from(
                "Roadmap phase definitions. Each phase may include label, \
                 timeframe, and items. Can be provided as body blocks instead.",
            ),
            required: false,
            default_value: None,
        }];
        optional.extend(common_optional_fields());
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from(
                    "The slide title (e.g., \"Product Roadmap\" or \"Q1-Q4 Roadmap\").",
                ),
                required: true,
                default_value: None,
            }],
            optional,
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
