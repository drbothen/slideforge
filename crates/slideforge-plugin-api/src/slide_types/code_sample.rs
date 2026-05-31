//! The `code_sample` slide type — a code listing slide.
//!
//! A `code_sample` slide displays source code with syntax highlighting.
//! The `code` field is required; `language` specifies the syntax highlighting
//! language (defaults to plain text if omitted).
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `code_sample` slide type.
///
/// Required fields: `title`, `code`.
/// Optional fields: `language`, plus common optional fields.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct CodeSampleSlideType {
    /// Required fields for the code sample slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the code sample slide type.
    optional: Vec<FieldDef>,
}

impl CodeSampleSlideType {
    /// Construct a new `CodeSampleSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("language"),
            description: Arc::from(
                "Programming language for syntax highlighting (e.g., \"rust\", \
                 \"python\", \"typescript\"). Defaults to plain text.",
            ),
            required: false,
            default_value: None,
        }];
        optional.extend(common_optional_fields());
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from("The slide title shown above the code block."),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("code"),
                    description: Arc::from(
                        "The source code to display. Provided as a fenced code block \
                         in the slide body.",
                    ),
                    required: true,
                    default_value: None,
                },
            ],
            optional,
        }
    }
}

impl Default for CodeSampleSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for CodeSampleSlideType {
    fn id(&self) -> &'static str {
        "code_sample"
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
