//! The `toc` slide type — a table of contents slide.
//!
//! A `toc` slide shows the major sections of the presentation, typically
//! auto-generated from `section_break` slides. Maps to the
//! `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `toc` slide type.
///
/// Required fields: `title`.
/// Optional fields: common optional fields.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct TocSlideType {
    /// Required fields for the TOC slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the TOC slide type.
    optional: Vec<FieldDef>,
}

impl TocSlideType {
    /// Construct a new `TocSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from(
                    "The slide title (e.g., \"Table of Contents\" or \"Overview\").",
                ),
                required: true,
                default_value: None,
            }],
            optional: common_optional_fields(),
        }
    }
}

impl Default for TocSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for TocSlideType {
    fn id(&self) -> &'static str {
        "toc"
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
        })
    }
}
