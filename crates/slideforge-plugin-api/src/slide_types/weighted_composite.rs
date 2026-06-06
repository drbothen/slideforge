//! The `weighted_composite` slide type — a multi-component composite score.
//!
//! A `weighted_composite` slide displays a composite score built from multiple
//! weighted sub-components (e.g., a vendor scorecard with dimensions "Quality",
//! "Price", "Support", each with a weight and score). Color conveys the
//! composite and per-component score magnitude, making this a color-coded type.
//!
//! WCAG AA compliance requires:
//! - A top-level `label` co-encoding the aggregate result in text.
//! - A per-component `label` co-encoding each component's score.
//!
//! Both top-level and component labels are mandatory. Missing labels accumulate
//! as E-A11-002 diagnostics (DI-018 error accumulation).
//!
//! DSL keyword: `weighted_composite`
//!
//! Required fields: `title`, `label` (aggregate), `components` (list)
//!
//! Each component in `components` requires: `name`, `weight`, `score`, `label`.
//!
//! # References
//!
//! - BC-1.17.003 — `weighted_composite` Slide Type Requires title + label + components\[\]
//! - STORY-087 — Color-Coded Slide Types (Wave 4)

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `weighted_composite` slide type.
///
/// Displays a composite score from multiple weighted sub-components. Both the
/// top-level `label` and each component's `label` are mandatory per WCAG 1.4.1
/// (Use of Color). Missing labels produce E-A11-002 diagnostics; all are
/// accumulated before returning (DI-018).
///
/// Required fields: `title`, `label` (aggregate), `components` (non-empty list).
/// Each component requires: `name`, `weight` (positive), `score` (0–100), `label`.
/// Optional fields: common optional fields (notes, report, detail, tags, etc.).
///
/// # Validation in `lay_out()`
///
/// - `components` must be a non-empty list → `Err(LayoutError::MissingRequiredField)`.
/// - Each component `weight` must be positive → `Err(LayoutError::FieldTypeMismatch)`.
/// - Each component `score` must be in \[0, 100\] → `Err(LayoutError::FieldTypeMismatch)`.
/// - ALL component errors are accumulated before returning (not bail-on-first).
///
/// Maps to the `"Blank"` OOXML layout (custom geometry produced by `lay_out`).
#[derive(Debug)]
pub struct WeightedCompositeSlideType {
    /// Required fields for the `weighted_composite` slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the `weighted_composite` slide type.
    optional: Vec<FieldDef>,
}

impl WeightedCompositeSlideType {
    /// Construct a new `WeightedCompositeSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let optional = common_optional_fields();
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from(
                        "The title naming what is being evaluated \
                         (e.g., \"Vendor A Scorecard\"). Required; non-empty.",
                    ),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("label"),
                    description: Arc::from(
                        "The accessible text co-encoding for the aggregate composite score \
                         (e.g., \"Overall: Good (78/100)\"). Mandatory per WCAG 1.4.1 \
                         (Use of Color). Empty or absent → E-A11-002.",
                    ),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("components"),
                    description: Arc::from(
                        "A non-empty list of scored sub-components. Each component must have: \
                         `name` (string), `weight` (positive number), `score` (0–100), \
                         and `label` (non-empty string co-encoding the score). \
                         Empty list → compile error. Missing component label → E-A11-002.",
                    ),
                    required: true,
                    default_value: None,
                },
            ],
            optional,
        }
    }
}

impl Default for WeightedCompositeSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for WeightedCompositeSlideType {
    fn id(&self) -> &'static str {
        "weighted_composite"
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
        todo!(
            "STORY-087 AC014/AC015/AC020/AC021: WeightedCompositeSlideType::lay_out not yet \
             implemented — must validate components non-empty, weight>0, score∈[0,100], \
             accumulate all errors (DI-018)"
        )
    }
}
