//! The `timeline` slide type — a project or historical timeline slide.
//!
//! A `timeline` slide displays events or milestones in chronological order.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `timeline` slide type.
///
/// Required fields: `title`.
/// Optional fields: `milestones`, plus common optional fields (`report`, `detail`, etc.).
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct TimelineSlideType {
    /// Required fields for the timeline slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the timeline slide type.
    optional: Vec<FieldDef>,
}

impl TimelineSlideType {
    /// Construct a new `TimelineSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("milestones"),
            description: Arc::from(
                "List of timeline milestones. Each entry may include date, label, \
                 and status. Can be provided as body blocks instead.",
            ),
            required: false,
            default_value: None,
        }];
        optional.extend(common_optional_fields());
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from("The slide title (e.g., \"Project Timeline\")."),
                required: true,
                default_value: None,
            }],
            optional,
        }
    }
}

impl Default for TimelineSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for TimelineSlideType {
    fn id(&self) -> &'static str {
        "timeline"
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
