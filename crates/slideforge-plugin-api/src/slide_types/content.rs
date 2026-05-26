//! The `content` slide type — a general-purpose body slide.
//!
//! A `content` slide has a required `title` and a flexible body area that
//! can hold bullets, paragraphs, or mixed inline content. This is the most
//! common slide type in a presentation.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide, Value};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

/// The built-in `content` slide type.
///
/// Required fields: `title`.
/// Optional fields: `layout`, `background`, `footer`, `notes`.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct ContentSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl ContentSlideType {
    /// Construct a new `ContentSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from("The slide title shown in the title area."),
                required: true,
                default_value: None,
            }],
            optional: vec![
                FieldDef {
                    name: Arc::from("layout"),
                    description: Arc::from(
                        "Content layout variant: one-col (default), two-col, three-col.",
                    ),
                    required: false,
                    default_value: Some(Value::Str(Arc::from("one-col"))),
                },
                FieldDef {
                    name: Arc::from("background"),
                    description: Arc::from("Override background color for this slide."),
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
                    name: Arc::from("notes"),
                    description: Arc::from("Presenter notes for this slide."),
                    required: false,
                    default_value: None,
                },
            ],
        }
    }
}

impl Default for ContentSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for ContentSlideType {
    fn id(&self) -> &str {
        "content"
    }

    fn required_fields(&self) -> &[FieldDef] {
        &self.required
    }

    fn optional_fields(&self) -> &[FieldDef] {
        &self.optional
    }

    fn layout_name(&self) -> &str {
        "Title and Content"
    }

    fn lay_out(
        &self,
        _slide: &Slide,
        _brand: &Brand,
        _canvas: Canvas,
    ) -> Result<LaidOutSlide, LayoutError> {
        // Stub: returns an empty LaidOutSlide. Full implementation in Phase 3.
        Ok(LaidOutSlide {
            width: SLIDE_WIDTH,
            height: SLIDE_HEIGHT,
            elements: vec![],
            slide_index: 0,
        })
    }
}
