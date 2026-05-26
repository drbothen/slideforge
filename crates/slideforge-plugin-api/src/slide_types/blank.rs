//! The `blank` slide type — an empty canvas slide.
//!
//! A `blank` slide has no required fields. It provides an empty canvas
//! for completely custom layouts using the `shape:` DSL block. It maps to
//! the `"Blank"` PPTX layout.

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `blank` slide type.
///
/// Required fields: none.
/// Optional fields: common optional fields (`notes`, `tags`, `alt`, `lang`,
/// `decorative`, `footer`, `logo`, `report`, `detail`).
///
/// Maps to the `"Blank"` OOXML layout.
#[derive(Debug)]
pub struct BlankSlideType {
    optional: Vec<FieldDef>,
}

impl BlankSlideType {
    /// Construct a new `BlankSlideType`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            optional: common_optional_fields(),
        }
    }
}

impl Default for BlankSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for BlankSlideType {
    fn id(&self) -> &'static str {
        "blank"
    }

    fn required_fields(&self) -> &[FieldDef] {
        &[]
    }

    fn optional_fields(&self) -> &[FieldDef] {
        &self.optional
    }

    fn layout_name(&self) -> &'static str {
        "Blank"
    }

    fn lay_out(
        &self,
        _slide: &Slide,
        _brand: &Brand,
        _canvas: Canvas,
    ) -> Result<LaidOutSlide, LayoutError> {
        // Stub: returns an empty LaidOutSlide. Full implementation in Phase 3.
        Ok(LaidOutSlide {
            width: SLIDE_WIDTH,
            height: SLIDE_HEIGHT,
            elements: vec![],
            slide_index: 0,
        })
    }
}
