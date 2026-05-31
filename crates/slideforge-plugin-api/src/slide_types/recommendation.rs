//! The `recommendation` slide type — a proposal or recommendation slide.
//!
//! A `recommendation` slide presents a specific course of action with its
//! rationale and risk profile. It has required `title` and `recommendation`
//! fields, with optional `rationale` and `risk` fields.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `recommendation` slide type.
///
/// Required fields: `title`, `recommendation`.
/// Optional fields: `rationale`, `risk`, plus common optional fields (`report`, `detail`, etc.).
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct RecommendationSlideType {
    /// Required fields for the recommendation slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the recommendation slide type.
    optional: Vec<FieldDef>,
}

impl RecommendationSlideType {
    /// Construct a new `RecommendationSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![
            FieldDef {
                name: Arc::from("rationale"),
                description: Arc::from("The reasoning or evidence supporting the recommendation."),
                required: false,
                default_value: None,
            },
            FieldDef {
                name: Arc::from("risk"),
                description: Arc::from(
                    "Key risks associated with the recommendation and mitigations.",
                ),
                required: false,
                default_value: None,
            },
        ];
        optional.extend(common_optional_fields());
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from("The slide title (e.g., \"Our Recommendation\")."),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("recommendation"),
                    description: Arc::from("The specific action or decision being recommended."),
                    required: true,
                    default_value: None,
                },
            ],
            optional,
        }
    }
}

impl Default for RecommendationSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for RecommendationSlideType {
    fn id(&self) -> &'static str {
        "recommendation"
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
