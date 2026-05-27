//! The `team` slide type — a team members overview slide.
//!
//! A `team` slide introduces the people on a team, typically with photos
//! and brief bios. Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `team` slide type.
///
/// Required fields: `title`.
/// Optional fields: `members`, plus common optional fields.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct TeamSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl TeamSlideType {
    /// Construct a new `TeamSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("members"),
            description: Arc::from(
                "List of team member entries. Each entry may include name, title, \
                 image, and bio. Can be provided as body blocks instead.",
            ),
            required: false,
            default_value: None,
        }];
        optional.extend(common_optional_fields());
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from(
                    "The slide title (e.g., \"Our Team\" or \"Meet the Team\").",
                ),
                required: true,
                default_value: None,
            }],
            optional,
        }
    }
}

impl Default for TeamSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for TeamSlideType {
    fn id(&self) -> &'static str {
        "team"
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
