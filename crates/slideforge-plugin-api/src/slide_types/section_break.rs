//! The `section_break` slide type — a section divider slide.
//!
//! A `section_break` slide signals the start of a new major section in the
//! presentation. It has a required `title` and an optional `subtitle`.
//! It maps to the `"Section Header"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `section_break` slide type.
///
/// Required fields: `title`.
/// Optional fields: `subtitle`, plus common optional fields.
///
/// Maps to the `"Section Header"` OOXML layout.
#[derive(Debug)]
pub struct SectionBreakSlideType {
    /// Required fields for the section break slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the section break slide type.
    optional: Vec<FieldDef>,
}

impl SectionBreakSlideType {
    /// Construct a new `SectionBreakSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("subtitle"),
            description: Arc::from("An optional subtitle or brief description of the section."),
            required: false,
            default_value: None,
        }];
        optional.extend(common_optional_fields());
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from("The section title displayed prominently on the divider."),
                required: true,
                default_value: None,
            }],
            optional,
        }
    }
}

impl Default for SectionBreakSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for SectionBreakSlideType {
    fn id(&self) -> &'static str {
        "section_break"
    }

    fn required_fields(&self) -> &[FieldDef] {
        &self.required
    }

    fn optional_fields(&self) -> &[FieldDef] {
        &self.optional
    }

    fn layout_name(&self) -> &'static str {
        "Section Header"
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
