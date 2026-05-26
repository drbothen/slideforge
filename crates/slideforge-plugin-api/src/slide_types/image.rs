//! The `image` slide type — a full-bleed or featured image slide.
//!
//! An `image` slide displays a visual asset with a title and required alt text.
//! The `alt` field is mandatory for WCAG AA accessibility compliance —
//! omitting it is a compile error.
//!
//! Maps to the `"Picture with Caption"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

/// The built-in `image` slide type.
///
/// Required fields: `title`, `image`, `alt`.
/// Optional fields: `caption`, `notes`, `footer`, `logo`, `decorative`, `tags`.
///
/// Maps to the `"Picture with Caption"` OOXML layout.
#[derive(Debug)]
pub struct ImageSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl ImageSlideType {
    /// Construct a new `ImageSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
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
            optional: vec![
                FieldDef {
                    name: Arc::from("caption"),
                    description: Arc::from(
                        "A caption displayed below the image (visible to all viewers).",
                    ),
                    required: false,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("decorative"),
                    description: Arc::from(
                        "When true, marks the image as decorative (empty alt text in output, \
                         PDF Artifact tag). Overrides the `alt` field.",
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
