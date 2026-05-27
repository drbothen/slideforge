//! The `agenda` slide type — a meeting or presentation agenda slide.
//!
//! An `agenda` slide lists the topics or sections to be covered. It has a
//! required `title` and an optional `items` list.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `agenda` slide type.
///
/// Required fields: `title`.
/// Optional fields: `items`, plus common optional fields.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct AgendaSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl AgendaSlideType {
    /// Construct a new `AgendaSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("items"),
            description: Arc::from(
                "List of agenda items. Can be provided as body bullets instead.",
            ),
            required: false,
            default_value: None,
        }];
        optional.extend(common_optional_fields());
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from("The slide title (e.g., \"Today's Agenda\" or \"Agenda\")."),
                required: true,
                default_value: None,
            }],
            optional,
        }
    }
}

impl Default for AgendaSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for AgendaSlideType {
    fn id(&self) -> &'static str {
        "agenda"
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
