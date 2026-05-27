//! The `title` slide type — the opening title slide.
//!
//! A `title` slide has a prominent centered title and an optional subtitle.
//! It maps to the `"Title Slide"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `title` slide type.
///
/// Required fields: `title`.
/// Optional fields: `subtitle`, `author`, `date`, plus common optional fields.
///
/// Maps to the `"Title Slide"` OOXML layout.
#[derive(Debug)]
pub struct TitleSlideType {
    /// Required fields for the title slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the title slide type.
    optional: Vec<FieldDef>,
}

impl TitleSlideType {
    /// Construct a new `TitleSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![
            FieldDef {
                name: Arc::from("subtitle"),
                description: Arc::from("A subtitle or deck description below the title."),
                required: false,
                default_value: None,
            },
            FieldDef {
                name: Arc::from("author"),
                description: Arc::from("The presenter or author name."),
                required: false,
                default_value: None,
            },
            FieldDef {
                name: Arc::from("date"),
                description: Arc::from("The presentation date."),
                required: false,
                default_value: None,
            },
        ];
        optional.extend(common_optional_fields());
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from("The main title displayed prominently on the slide."),
                required: true,
                default_value: None,
            }],
            optional,
        }
    }
}

impl Default for TitleSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for TitleSlideType {
    fn id(&self) -> &'static str {
        "title"
    }

    fn required_fields(&self) -> &[FieldDef] {
        &self.required
    }

    fn optional_fields(&self) -> &[FieldDef] {
        &self.optional
    }

    fn layout_name(&self) -> &'static str {
        "Title Slide"
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
