//! The `section-break` slide type — a section divider slide.
//!
//! A `section-break` slide signals the start of a new major section in the
//! presentation. It has a required `title` and an optional `subtitle`.
//! It maps to the `"Section Header"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

/// The built-in `section-break` slide type.
///
/// Required fields: `title`.
/// Optional fields: `subtitle`, `notes`, `footer`, `logo`.
///
/// Maps to the `"Section Header"` OOXML layout.
#[derive(Debug)]
pub struct SectionBreakSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl SectionBreakSlideType {
    /// Construct a new `SectionBreakSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from("The section title displayed prominently on the divider."),
                required: true,
                default_value: None,
            }],
            optional: vec![
                FieldDef {
                    name: Arc::from("subtitle"),
                    description: Arc::from(
                        "An optional subtitle or brief description of the section.",
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

impl Default for SectionBreakSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for SectionBreakSlideType {
    fn id(&self) -> &'static str {
        "section-break"
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
