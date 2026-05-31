//! The `matrix` slide type — a 2×2 or N×M matrix analysis slide.
//!
//! A `matrix` slide arranges content in a grid format for frameworks such as
//! the Boston Consulting Group growth-share matrix or an impact/effort matrix.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `matrix` slide type.
///
/// Required fields: `title`.
/// Optional fields: `cells`, plus common optional fields (`report`, `detail`, etc.).
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct MatrixSlideType {
    /// Required fields for the matrix slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the matrix slide type.
    optional: Vec<FieldDef>,
}

impl MatrixSlideType {
    /// Construct a new `MatrixSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("cells"),
            description: Arc::from(
                "Grid cell content definitions. Structure depends on the matrix \
                 dimensions. Can be provided as body blocks instead.",
            ),
            required: false,
            default_value: None,
        }];
        optional.extend(common_optional_fields());
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from("The slide title (e.g., \"Impact / Effort Matrix\")."),
                required: true,
                default_value: None,
            }],
            optional,
        }
    }
}

impl Default for MatrixSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for MatrixSlideType {
    fn id(&self) -> &'static str {
        "matrix"
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
