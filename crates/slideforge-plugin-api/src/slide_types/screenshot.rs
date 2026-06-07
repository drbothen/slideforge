//! The `screenshot` slide type — a product screenshot or UI capture slide.
//!
//! A `screenshot` slide displays a product, application, or UI screenshot
//! with a title and required alt text. An optional caption provides context.
//!
//! Maps to the `"Picture with Caption"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `screenshot` slide type.
///
/// Required fields: `title`, `src`, `alt`.
/// Optional fields: `caption`, plus common optional fields.
///
/// The canonical media-source field keyword is `src` per BC-1.16.001 PC-10/EC-006.
///
/// Maps to the `"Picture with Caption"` OOXML layout.
#[derive(Debug)]
pub struct ScreenshotSlideType {
    /// Required fields for the screenshot slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the screenshot slide type.
    optional: Vec<FieldDef>,
}

impl ScreenshotSlideType {
    /// Construct a new `ScreenshotSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("caption"),
            description: Arc::from(
                "A caption displayed below the screenshot providing additional context.",
            ),
            required: false,
            default_value: None,
            expected_type: None,
        }];
        // `alt` is required on screenshot slides; exclude it from the common optional
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
                    description: Arc::from("The slide title shown above the screenshot."),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
                FieldDef {
                    name: Arc::from("src"),
                    description: Arc::from(
                        "Path or URL to the screenshot image (PNG, JPEG, GIF, WebP). \
                         Canonical keyword per BC-1.16.001 PC-10/EC-006.",
                    ),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
                FieldDef {
                    name: Arc::from("alt"),
                    description: Arc::from(
                        "Accessibility alt text describing what the screenshot shows. \
                         Required for WCAG AA compliance.",
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

impl Default for ScreenshotSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for ScreenshotSlideType {
    fn id(&self) -> &'static str {
        "screenshot"
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
