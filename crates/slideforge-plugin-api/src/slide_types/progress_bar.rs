//! The `progress_bar` slide type — a visual numeric progress indicator.
//!
//! A `progress_bar` slide displays a filled progress bar where the filled
//! proportion conveys completion state. Because color/proportion alone cannot
//! convey meaning to screen readers, the `label` field is mandatory and must
//! co-encode the numeric progress in text form (e.g., `label "75% complete"`).
//! The `value` field (integer 0–100) drives the visual fill proportion.
//!
//! DSL keyword: `progress_bar`
//!
//! Required fields: `title`, `label`, `value`
//!
//! # References
//!
//! - BC-1.17.002 — `progress_bar` Slide Type Requires title + label + value(0–100)
//! - STORY-087 — Color-Coded Slide Types (Wave 4)

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `progress_bar` slide type.
///
/// Displays a visual progress bar filled to `value`% of the bar's total width.
/// The `label` field is mandatory (WCAG 1.4.1: Use of Color) and must co-encode
/// the numeric progress in accessible text form.
///
/// Required fields: `title`, `label`, `value` (integer in \[0, 100\]).
/// Optional fields: common optional fields (notes, report, detail, tags, etc.).
///
/// # Value range validation
///
/// `lay_out()` validates that `value` is an integer in the range \[0, 100\]
/// inclusive. Values outside this range return
/// `Err(LayoutError::FieldTypeMismatch)` with a message indicating the
/// expected range and actual value. This is performed in `lay_out()` because
/// the trait has no separate validation method.
///
/// Maps to the `"Blank"` OOXML layout (custom geometry produced by `lay_out`).
#[derive(Debug)]
pub struct ProgressBarSlideType {
    /// Required fields for the `progress_bar` slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the `progress_bar` slide type.
    optional: Vec<FieldDef>,
}

impl ProgressBarSlideType {
    /// Construct a new `ProgressBarSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let optional = common_optional_fields();
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from(
                        "The title identifying what progress is being tracked \
                         (e.g., \"Sprint 4 Completion\"). Required; non-empty.",
                    ),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("label"),
                    description: Arc::from(
                        "The accessible text co-encoding for the color-coded progress \
                         (e.g., \"75% complete\"). Mandatory per WCAG 1.4.1 \
                         (Use of Color). Empty or absent → E-A11-002.",
                    ),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("value"),
                    description: Arc::from(
                        "The numeric completion percentage as an integer in the range \
                         [0, 100] inclusive. Drives the visual fill proportion of the bar. \
                         Values outside [0, 100] are a compile error.",
                    ),
                    required: true,
                    default_value: None,
                },
            ],
            optional,
        }
    }
}

impl Default for ProgressBarSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for ProgressBarSlideType {
    fn id(&self) -> &'static str {
        "progress_bar"
    }

    fn required_fields(&self) -> &[FieldDef] {
        &self.required
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
        todo!("STORY-087 AC007/AC008/AC012/AC013: ProgressBarSlideType::lay_out not yet implemented — must validate value ∈ [0,100]")
    }
}
