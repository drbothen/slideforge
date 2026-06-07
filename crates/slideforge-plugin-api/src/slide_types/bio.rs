//! The `bio` slide type — a single-person biography slide.
//!
//! A `bio` slide introduces an individual with their name, title, and a brief
//! biography. An optional photo and alt text are supported.
//!
//! Maps to the `"Two Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `bio` slide type.
///
/// Required fields: `name`, `title`, `bio`.
/// Optional fields: `src`, plus common optional fields
/// (`notes`, `alt`, `decorative`, `tags`, `footer`, `logo`, etc.).
///
/// The canonical media-source field keyword is `src` per BC-1.16.001 PC-10/EC-006.
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
        // Canonical media-source keyword is `src` per BC-1.16.001 PC-10/EC-006;
        // field_to_block.rs reads "src" to construct ContentBlock::Image.
        let mut optional = vec![FieldDef {
            name: Arc::from("src"),
            description: Arc::from(
                "Path or URL to the person's photo. \
                 Canonical keyword per BC-1.16.001 PC-10/EC-006.",
            ),
            required: false,
            default_value: None,
            expected_type: None,
        }];
        optional.extend(common_optional_fields());
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("name"),
                    description: Arc::from("The person's full name."),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from("The person's job title or role."),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
                FieldDef {
                    name: Arc::from("bio"),
                    description: Arc::from(
                        "A short biography of the person (2-4 sentences recommended).",
                    ),
                    required: true,
                    default_value: None,
                    expected_type: None,
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
