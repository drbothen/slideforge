//! The `video` slide type — an embedded video slide.
//!
//! A `video` slide embeds a video asset (local file or URL) into the
//! presentation. The `alt` field is required for WCAG AA compliance —
//! it describes the video content for screen readers and non-playing exports.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `video` slide type.
///
/// Required fields: `title`, `video_url`, `alt`.
/// Optional fields: common optional fields.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct VideoSlideType {
    /// Required fields for the video slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the video slide type.
    optional: Vec<FieldDef>,
}

impl VideoSlideType {
    /// Construct a new `VideoSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from("The slide title shown above the video player."),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
                FieldDef {
                    name: Arc::from("video_url"),
                    description: Arc::from(
                        "Path or URL to the video file (MP4, MOV) or streaming URL \
                         (YouTube, Vimeo, etc.).",
                    ),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
                FieldDef {
                    name: Arc::from("alt"),
                    description: Arc::from(
                        "Accessibility description of the video content. Used by screen \
                         readers and in non-interactive export formats (PDF, PPTX thumbnail). \
                         Required for WCAG AA compliance.",
                    ),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
            ],
            // `alt` is required on video slides; exclude it from the common optional
            // fields to avoid duplicating it in required ∪ optional (F-P2-003).
            optional: common_optional_fields()
                .into_iter()
                .filter(|f| f.name.as_ref() != "alt")
                .collect(),
        }
    }
}

impl Default for VideoSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for VideoSlideType {
    fn id(&self) -> &'static str {
        "video"
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
