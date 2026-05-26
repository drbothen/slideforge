//! The `stat-callout` slide type — a two-statistic highlight slide.
//!
//! A `stat-callout` slide displays two key statistics side-by-side, each with
//! a numeric value and a descriptive label. This is a high-impact slide type
//! frequently used in executive summaries.
//!
//! The DSL keyword is `stat-callout` (with a hyphen).

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

/// The built-in `stat-callout` slide type.
///
/// Required fields: `stat_1`, `label_1`, `stat_2`, `label_2`.
/// Optional fields: `title`, `context`.
///
/// Maps to the `"Two Content"` OOXML layout.
#[derive(Debug)]
pub struct StatCalloutSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl StatCalloutSlideType {
    /// Construct a new `StatCalloutSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("stat_1"),
                    description: Arc::from(
                        "The first statistic value (e.g., \"$4.2B\", \"93%\", \"12x\").",
                    ),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("label_1"),
                    description: Arc::from(
                        "A short label describing what stat_1 measures (≤ 6 words).",
                    ),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("stat_2"),
                    description: Arc::from(
                        "The second statistic value (e.g., \"$4.2B\", \"93%\", \"12x\").",
                    ),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("label_2"),
                    description: Arc::from(
                        "A short label describing what stat_2 measures (≤ 6 words).",
                    ),
                    required: true,
                    default_value: None,
                },
            ],
            optional: vec![
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from("Optional section title above the statistics."),
                    required: false,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("context"),
                    description: Arc::from("A short contextual note below the statistics."),
                    required: false,
                    default_value: None,
                },
            ],
        }
    }
}

impl Default for StatCalloutSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for StatCalloutSlideType {
    fn id(&self) -> &str {
        "stat-callout"
    }

    fn required_fields(&self) -> &[FieldDef] {
        &self.required
    }

    fn optional_fields(&self) -> &[FieldDef] {
        &self.optional
    }

    fn layout_name(&self) -> &str {
        "Two Content"
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
