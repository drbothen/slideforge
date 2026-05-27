//! The `content` slide type — a general-purpose body slide.
//!
//! A `content` slide has a required `title` and a flexible body area that
//! can hold bullets, paragraphs, or mixed inline content. This is the most
//! common slide type in a presentation.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `content` slide type.
///
/// Required fields: `title`.
/// Optional fields: `bullets`, `takeaway`, plus common optional fields
/// (`notes`, `report`, `detail`, `tags`, `alt`, `lang`, `decorative`, `footer`, `logo`).
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct ContentSlideType {
    /// Required fields for the content slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the content slide type.
    optional: Vec<FieldDef>,
}

impl ContentSlideType {
    /// Construct a new `ContentSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![
            FieldDef {
                name: Arc::from("bullets"),
                description: Arc::from(
                    "Bullet list items for the slide body. Can be provided as body blocks instead.",
                ),
                required: false,
                default_value: None,
            },
            FieldDef {
                name: Arc::from("takeaway"),
                description: Arc::from(
                    "The key takeaway or call-to-action displayed at the bottom of the slide.",
                ),
                required: false,
                default_value: None,
            },
        ];
        optional.extend(common_optional_fields());
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from("The slide title shown in the title area."),
                required: true,
                default_value: None,
            }],
            optional,
        }
    }
}

impl Default for ContentSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for ContentSlideType {
    fn id(&self) -> &'static str {
        "content"
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
        // Stub: returns an empty LaidOutSlide. Full implementation in Phase 3.
        Ok(LaidOutSlide {
            source_index: 0,
            slide_type_keyword: std::sync::Arc::clone(&slide.slide_type),
            frames: vec![],
            speaker_notes: None,
            register_tags: vec![],
        })
    }
}
