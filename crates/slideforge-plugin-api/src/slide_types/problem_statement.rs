//! The `problem_statement` slide type — a problem definition slide.
//!
//! A `problem_statement` slide articulates the core problem being addressed.
//! It has required `title` and `problem` fields, with an optional `impact`
//! field to describe the consequences of the problem.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `problem_statement` slide type.
///
/// Required fields: `title`, `problem`.
/// Optional fields: `impact`, plus common optional fields (`report`, `detail`, etc.).
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct ProblemStatementSlideType {
    /// Required fields for the problem statement slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the problem statement slide type.
    optional: Vec<FieldDef>,
}

impl ProblemStatementSlideType {
    /// Construct a new `ProblemStatementSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("impact"),
            description: Arc::from("The business or human impact of the problem if left unsolved."),
            required: false,
            default_value: None,
        }];
        optional.extend(common_optional_fields());
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from("The slide title (e.g., \"The Problem\")."),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("problem"),
                    description: Arc::from(
                        "A clear, concise statement of the problem being addressed.",
                    ),
                    required: true,
                    default_value: None,
                },
            ],
            optional,
        }
    }
}

impl Default for ProblemStatementSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for ProblemStatementSlideType {
    fn id(&self) -> &'static str {
        "problem_statement"
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
