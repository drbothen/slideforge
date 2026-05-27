//! The `diagram` slide type — a diagram or flowchart slide.
//!
//! A `diagram` slide renders a diagram from Mermaid syntax (or another
//! `DiagramRenderer` plugin) and places it on the slide. The `alt` field
//! is optional per spec and is included via common optional fields; authors
//! should provide it for WCAG AA compliance on non-decorative diagrams.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `diagram` slide type.
///
/// Required fields: `title`, `diagram`.
/// Optional fields: `report`, `detail`, plus common optional fields.
/// Note: `alt` is included via common optional fields.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct DiagramSlideType {
    /// Required fields for the diagram slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the diagram slide type.
    optional: Vec<FieldDef>,
}

impl DiagramSlideType {
    /// Construct a new `DiagramSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from("The slide title shown above the diagram."),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("diagram"),
                    description: Arc::from(
                        "The diagram source (Mermaid syntax or other supported format). \
                         Provided as a fenced code block in the slide body.",
                    ),
                    required: true,
                    default_value: None,
                },
            ],
            optional: common_optional_fields(),
        }
    }
}

impl Default for DiagramSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for DiagramSlideType {
    fn id(&self) -> &'static str {
        "diagram"
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
