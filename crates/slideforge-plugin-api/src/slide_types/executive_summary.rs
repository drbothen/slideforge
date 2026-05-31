//! The `executive_summary` slide type — a high-level executive overview slide.
//!
//! An `executive_summary` slide distills the most critical information for
//! executive stakeholders. It has a required `title` and `summary`, with an
//! optional bullet list for supporting points.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `executive_summary` slide type.
///
/// Required fields: `title`, `summary`.
/// Optional fields: `bullets`, plus common optional fields (`report`, `detail`, etc.).
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct ExecutiveSummarySlideType {
    /// Required fields for the executive summary slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the executive summary slide type.
    optional: Vec<FieldDef>,
}

impl ExecutiveSummarySlideType {
    /// Construct a new `ExecutiveSummarySlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![FieldDef {
            name: Arc::from("bullets"),
            description: Arc::from("Supporting bullet points that elaborate on the summary."),
            required: false,
            default_value: None,
        }];
        optional.extend(common_optional_fields());
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from("The slide title (e.g., \"Executive Summary\")."),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("summary"),
                    description: Arc::from("The one-paragraph executive summary (3-5 sentences)."),
                    required: true,
                    default_value: None,
                },
            ],
            optional,
        }
    }
}

impl Default for ExecutiveSummarySlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for ExecutiveSummarySlideType {
    fn id(&self) -> &'static str {
        "executive_summary"
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
