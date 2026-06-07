//! The `two_col` slide type — a two-column comparison slide.
//!
//! A `two_col` slide presents content in two side-by-side columns, each with
//! its own heading and body. It maps to the `"Two Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `two_col` slide type.
///
/// Required fields: `title`, `left`, `right`.
/// Optional fields: `report`, `detail`, plus common optional fields.
///
/// Maps to the `"Two Content"` OOXML layout.
#[derive(Debug)]
pub struct TwoColSlideType {
    /// Required fields for the two-column slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the two-column slide type.
    optional: Vec<FieldDef>,
}

impl TwoColSlideType {
    /// Construct a new `TwoColSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from("The slide title shown in the title area."),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
                FieldDef {
                    name: Arc::from("left"),
                    description: Arc::from("Content for the left column."),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
                FieldDef {
                    name: Arc::from("right"),
                    description: Arc::from("Content for the right column."),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
            ],
            optional: common_optional_fields(),
        }
    }
}

impl Default for TwoColSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for TwoColSlideType {
    fn id(&self) -> &'static str {
        "two_col"
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
