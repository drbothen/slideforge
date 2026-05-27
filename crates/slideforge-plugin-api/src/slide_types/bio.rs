//! The `bio` slide type — a single-person biography slide.
//!
//! A `bio` slide introduces an individual with their name, title, and a brief
//! biography. An optional photo and alt text are supported.
//!
//! Maps to the `"Two Content"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `bio` slide type.
///
/// Required fields: `name`, `title`, `bio`.
/// Optional fields: `image`, plus common optional fields
/// (`notes`, `alt`, `decorative`, `tags`, `footer`, `logo`, etc.).
///
/// Maps to the `"Two Content"` OOXML layout.
#[derive(Debug)]
pub struct BioSlideType {
    /// Required fields for the bio slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the bio slide type.
    optional: Vec<FieldDef>,
}

impl BioSlideType {
    /// Construct a new `BioSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("image"),
            description: Arc::from("Path or URL to the person's photo."),
            required: false,
            default_value: None,
        }];
        optional.extend(common_optional_fields());
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
            optional,
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
