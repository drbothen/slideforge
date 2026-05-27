//! The `comparison` slide type — a side-by-side option comparison slide.
//!
//! A `comparison` slide presents two options (A and B) for evaluation, with
//! an optional `criteria` field listing the evaluation dimensions.
//!
//! Maps to the `"Two Content"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `comparison` slide type.
///
/// Required fields: `title`, `option_a`, `option_b`.
/// Optional fields: `criteria`, plus common optional fields (`report`, `detail`, etc.).
///
/// Maps to the `"Two Content"` OOXML layout.
#[derive(Debug)]
pub struct ComparisonSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl ComparisonSlideType {
    /// Construct a new `ComparisonSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("criteria"),
            description: Arc::from("Evaluation criteria dimensions used to compare the options."),
            required: false,
            default_value: None,
        }];
        optional.extend(common_optional_fields());
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from("The slide title (e.g., \"Option Comparison\")."),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("option_a"),
                    description: Arc::from(
                        "The heading and content for the first option (left column).",
                    ),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("option_b"),
                    description: Arc::from(
                        "The heading and content for the second option (right column).",
                    ),
                    required: true,
                    default_value: None,
                },
            ],
            optional,
        }
    }
}

impl Default for ComparisonSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for ComparisonSlideType {
    fn id(&self) -> &'static str {
        "comparison"
    }

    fn required_fields(&self) -> &[FieldDef] {
        &self.required
    }

    fn optional_fields(&self) -> &[FieldDef] {
        &self.optional
    }

    fn layout_name(&self) -> &'static str {
        "Two Content"
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
