//! The `closing` slide type — a closing or call-to-action slide.
//!
//! A `closing` slide ends the presentation with a title, an optional call to
//! action, and contact information. It maps to the `"Title Slide"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

/// The built-in `closing` slide type.
///
/// Required fields: `title`.
/// Optional fields: `call_to_action`, `contact`, `notes`, `footer`, `logo`, `tags`.
///
/// Maps to the `"Title Slide"` OOXML layout.
#[derive(Debug)]
pub struct ClosingSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl ClosingSlideType {
    /// Construct a new `ClosingSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from(
                    "The closing slide title (e.g., \"Thank You\", \"Questions?\", \
                     \"Let's Connect\").",
                ),
                required: true,
                default_value: None,
            }],
            optional: vec![
                FieldDef {
                    name: Arc::from("call_to_action"),
                    description: Arc::from("A prominent call to action displayed below the title."),
                    required: false,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("contact"),
                    description: Arc::from(
                        "Contact information (email, website, social handles) for follow-up.",
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

impl Default for ClosingSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for ClosingSlideType {
    fn id(&self) -> &'static str {
        "closing"
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
        // Stub: returns an empty LaidOutSlide. Full geometric layout in Phase 3.
        Ok(LaidOutSlide {
            width: SLIDE_WIDTH,
            height: SLIDE_HEIGHT,
            elements: vec![],
            slide_index: 0,
        })
    }
}
