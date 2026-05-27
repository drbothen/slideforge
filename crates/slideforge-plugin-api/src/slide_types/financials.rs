//! The `financials` slide type — a financial data table slide.
//!
//! A `financials` slide presents financial metrics, P&L, budget, or forecast
//! data in a structured tabular format.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `financials` slide type.
///
/// Required fields: `title`.
/// Optional fields: `rows`, plus common optional fields (`report`, `detail`, etc.).
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct FinancialsSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl FinancialsSlideType {
    /// Construct a new `FinancialsSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("rows"),
            description: Arc::from(
                "Table row definitions for financial data. Can be provided as \
                 body table blocks instead.",
            ),
            required: false,
            default_value: None,
        }];
        optional.extend(common_optional_fields());
        Self {
            required: vec![FieldDef {
                name: Arc::from("title"),
                description: Arc::from(
                    "The slide title (e.g., \"Financial Summary\" or \"FY2025 P&L\").",
                ),
                required: true,
                default_value: None,
            }],
            optional,
        }
    }
}

impl Default for FinancialsSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for FinancialsSlideType {
    fn id(&self) -> &'static str {
        "financials"
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
