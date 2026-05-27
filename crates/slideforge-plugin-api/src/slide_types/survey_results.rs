//! The `survey_results` slide type — a survey or poll results slide.
//!
//! A `survey_results` slide displays findings from a survey, poll, or
//! customer research study. Results can be provided as a structured list,
//! chart data, or body blocks.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `survey_results` slide type.
///
/// Required fields: `title`.
/// Optional fields: `results`, plus common optional fields (`report`, `detail`, etc.).
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct SurveyResultsSlideType {
    /// Required fields for the survey results slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the survey results slide type.
    optional: Vec<FieldDef>,
}

impl SurveyResultsSlideType {
    /// Construct a new `SurveyResultsSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("results"),
            description: Arc::from(
                "Structured survey results data. Can be provided as body blocks instead.",
            ),
            required: false,
            default_value: None,
        }];
        optional.extend(common_optional_fields());
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from(
                    "The slide title (e.g., \"Survey Results\" or \"What Customers Said\").",
                ),
                required: true,
                default_value: None,
            }],
            optional,
        }
    }
}

impl Default for SurveyResultsSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for SurveyResultsSlideType {
    fn id(&self) -> &'static str {
        "survey_results"
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
