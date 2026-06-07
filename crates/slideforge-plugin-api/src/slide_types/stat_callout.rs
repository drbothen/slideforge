//! The `stat_callout` slide type — a two-statistic highlight slide.
//!
//! A `stat_callout` slide displays two key statistics side-by-side, each with
//! a numeric value and a descriptive label. This is a high-impact slide type
//! frequently used in executive summaries.
//!
//! The DSL keyword is `stat_callout` (with an underscore).

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `stat_callout` slide type.
///
/// Required fields: `stat_1`, `label_1`, `stat_2`, `label_2`.
/// Optional fields: `stat_3`, `label_3`, plus common optional fields.
///
/// Maps to the `"Two Content"` OOXML layout.
#[derive(Debug)]
pub struct StatCalloutSlideType {
    /// Required fields for the stat callout slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the stat callout slide type.
    optional: Vec<FieldDef>,
}

impl StatCalloutSlideType {
    /// Construct a new `StatCalloutSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let mut optional = vec![
            FieldDef {
                name: Arc::from("stat_3"),
                description: Arc::from(
                    "An optional third statistic value (e.g., \"$4.2B\", \"93%\", \"12x\").",
                ),
                required: false,
                default_value: None,
                expected_type: None,
            },
            FieldDef {
                name: Arc::from("label_3"),
                description: Arc::from(
                    "A short label describing what stat_3 measures (≤ 6 words).",
                ),
                required: false,
                default_value: None,
                expected_type: None,
            },
        ];
        optional.extend(common_optional_fields());
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("stat_1"),
                    description: Arc::from(
                        "The first statistic value (e.g., \"$4.2B\", \"93%\", \"12x\").",
                    ),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
                FieldDef {
                    name: Arc::from("label_1"),
                    description: Arc::from(
                        "A short label describing what stat_1 measures (≤ 6 words).",
                    ),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
                FieldDef {
                    name: Arc::from("stat_2"),
                    description: Arc::from(
                        "The second statistic value (e.g., \"$4.2B\", \"93%\", \"12x\").",
                    ),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
                FieldDef {
                    name: Arc::from("label_2"),
                    description: Arc::from(
                        "A short label describing what stat_2 measures (≤ 6 words).",
                    ),
                    required: true,
                    default_value: None,
                    expected_type: None,
                },
            ],
            optional,
        }
    }
}

impl Default for StatCalloutSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for StatCalloutSlideType {
    fn id(&self) -> &'static str {
        "stat_callout"
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
        // Stub: returns an empty LaidOutSlide. Full implementation in Phase 3.
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
