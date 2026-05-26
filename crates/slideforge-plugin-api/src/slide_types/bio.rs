//! The `bio` slide type — a single-person biography slide.
//!
//! A `bio` slide introduces an individual with their name, title, and a brief
//! biography. An optional photo and alt text are supported.
//!
//! Maps to the `"Two Content"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

/// The built-in `bio` slide type.
///
/// Required fields: `name`, `title`, `bio`.
/// Optional fields: `image`, `alt`, `notes`, `footer`, `logo`, `decorative`, `tags`.
///
/// Maps to the `"Two Content"` OOXML layout.
#[derive(Debug)]
pub struct BioSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl BioSlideType {
    /// Construct a new `BioSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("name"),
                    description: Arc::from("The person's full name."),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from("The person's job title or role."),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("bio"),
                    description: Arc::from(
                        "A short biography of the person (2-4 sentences recommended).",
                    ),
                    required: true,
                    default_value: None,
                },
            ],
            optional: vec![
                FieldDef {
                    name: Arc::from("image"),
                    description: Arc::from("Path or URL to the person's photo."),
                    required: false,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("alt"),
                    description: Arc::from(
                        "Accessibility alt text for the photo. Required when `image` is set \
                         and `decorative` is false.",
                    ),
                    required: false,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("decorative"),
                    description: Arc::from(
                        "When true, marks the photo as decorative (empty alt text in output).",
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

impl Default for BioSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for BioSlideType {
    fn id(&self) -> &'static str {
        "bio"
    }

    fn required_fields(&self) -> &[FieldDef] {
        &self.required
    }

    fn optional_fields(&self) -> &[FieldDef] {
        &self.optional
    }

    fn layout_name(&self) -> &'static str {
        "Two Content"
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
