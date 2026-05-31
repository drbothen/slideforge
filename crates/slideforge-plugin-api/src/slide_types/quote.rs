//! The `quote` slide type — a pull-quote or testimonial slide.
//!
//! A `quote` slide displays a prominent quotation with its attribution.
//! Both `quote` and `attribution` are required.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `quote` slide type.
///
/// Required fields: `quote`, `attribution`.
/// Optional fields: common optional fields.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct QuoteSlideType {
    /// Required fields for the quote slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the quote slide type.
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
            optional: common_optional_fields(),
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
