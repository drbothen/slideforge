//! The `risk_register` slide type — a structured risk register slide.
//!
//! A `risk_register` slide displays a table of identified risks with their
//! likelihood, impact, and mitigation status.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `risk_register` slide type.
///
/// Required fields: `title`.
/// Optional fields: `risks`, plus common optional fields (`report`, `detail`, etc.).
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct RiskRegisterSlideType {
    /// Required fields for the risk register slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the risk register slide type.
    optional: Vec<FieldDef>,
}

impl RiskRegisterSlideType {
    /// Construct a new `RiskRegisterSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("risks"),
            description: Arc::from(
                "List of risk entries. Each entry may include risk, likelihood, \
                 impact, owner, and mitigation.",
            ),
            required: false,
            default_value: None,
        }];
        optional.extend(common_optional_fields());
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from("The slide title (e.g., \"Risk Register\")."),
                required: true,
                default_value: None,
            }],
            optional,
        }
    }
}

impl Default for RiskRegisterSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for RiskRegisterSlideType {
    fn id(&self) -> &'static str {
        "risk_register"
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
