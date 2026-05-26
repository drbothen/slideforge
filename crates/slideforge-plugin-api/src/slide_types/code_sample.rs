//! The `code-sample` slide type — a code listing slide.
//!
//! A `code-sample` slide displays source code with syntax highlighting.
//! The `code` field is required; `language` specifies the syntax highlighting
//! language (defaults to plain text if omitted).
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

/// The built-in `code-sample` slide type.
///
/// Required fields: `title`, `code`.
/// Optional fields: `language`, `notes`, `footer`, `logo`, `tags`.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct CodeSampleSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl CodeSampleSlideType {
    /// Construct a new `CodeSampleSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
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
            optional: vec![
                FieldDef {
                    name: Arc::from("language"),
                    description: Arc::from(
                        "Programming language for syntax highlighting (e.g., \"rust\", \
                         \"python\", \"typescript\"). Defaults to plain text.",
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

impl Default for CodeSampleSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for CodeSampleSlideType {
    fn id(&self) -> &'static str {
        "code-sample"
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
