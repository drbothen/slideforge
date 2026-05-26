//! The `quote` slide type — a pull-quote or testimonial slide.
//!
//! A `quote` slide displays a prominent quotation with its attribution.
//! Both `quote` and `attribution` are required.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

/// The built-in `quote` slide type.
///
/// Required fields: `quote`, `attribution`.
/// Optional fields: `notes`, `footer`, `logo`, `tags`.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct QuoteSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl QuoteSlideType {
    /// Construct a new `QuoteSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("quote"),
                    description: Arc::from(
                        "The quotation text, displayed prominently on the slide.",
                    ),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("attribution"),
                    description: Arc::from(
                        "The person or source being quoted (name, title, organization).",
                    ),
                    required: true,
                    default_value: None,
                },
            ],
            optional: vec![
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

impl Default for QuoteSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for QuoteSlideType {
    fn id(&self) -> &'static str {
        "quote"
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
