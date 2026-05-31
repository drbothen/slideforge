//! The `image` slide type — a full-bleed or featured image slide.
//!
//! An `image` slide displays a visual asset with a title and required alt text.
//! The `alt` field is mandatory for WCAG AA accessibility compliance —
//! omitting it is a compile error.
//!
//! Maps to the `"Picture with Caption"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `image` slide type.
///
/// Required fields: `title`, `image`, `alt`.
/// Optional fields: `caption`, plus common optional fields.
///
/// Maps to the `"Picture with Caption"` OOXML layout.
#[derive(Debug)]
pub struct ImageSlideType {
    /// Required fields for the image slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the image slide type.
    optional: Vec<FieldDef>,
}

impl ImageSlideType {
    /// Construct a new `ImageSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("caption"),
            description: Arc::from("A caption displayed below the image (visible to all viewers)."),
            required: false,
            default_value: None,
        }];
        // `alt` is required on image slides; exclude it from the common optional
        // fields to avoid duplicating it in required ∪ optional (F-P2-003).
        optional.extend(
            common_optional_fields()
                .into_iter()
                .filter(|f| f.name.as_ref() != "alt"),
        );
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from("The slide title shown above the image."),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("image"),
                    description: Arc::from("Path or URL to the image asset (PNG, JPEG, SVG, GIF)."),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("alt"),
                    description: Arc::from(
                        "Accessibility alt text describing the image content. \
                         Required for WCAG AA compliance.",
                    ),
                    required: true,
                    default_value: None,
                },
            ],
            optional,
        }
    }
}

impl Default for ImageSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for ImageSlideType {
    fn id(&self) -> &'static str {
        "image"
    }

    fn required_fields(&self) -> &[FieldDef] {
        &self.required
    }

    fn optional_fields(&self) -> &[FieldDef] {
        &self.optional
    }

    fn layout_name(&self) -> &'static str {
        "Picture with Caption"
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
