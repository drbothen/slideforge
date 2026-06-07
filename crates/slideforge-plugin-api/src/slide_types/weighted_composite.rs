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
//! # Validation
//!
//! - Label validation (top-level and per-component) is performed by
//!   `LabelCheckValidator` at Stage 5 (pre-layout).
//! - Value-range validation (`weight > 0`, `score ∈ [0, 100]`) is performed by
//!   `ValueRangeValidator` at Stage 5 (pre-layout), registered in
//!   `slideforge::registry::register_bundled_plugins`.
//! - `lay_out()` is geometry-only and does NOT perform content validation.
//!
//! See architect adjudication F-087-P1-001 for the full rationale.
//!
//! # References
//!
//! - BC-1.17.003 — `weighted_composite` Slide Type Requires title + label + components\[\]
//! - STORY-087 — Color-Coded Slide Types (Wave 4)
//! - Architect adjudication F-087-P1-001: Option B — `ValueRangeValidator` Stage 5

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, FieldType, LayoutError, SlideType};

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
/// # Geometry-only `lay_out()`
///
/// `lay_out()` produces the static 7-frame skeleton. All content validation
/// (label presence, weight/score ranges, non-empty components) is performed by
/// `LabelCheckValidator` and `ValueRangeValidator` at Stage 5 (pre-layout).
/// See architect adjudication F-087-P1-001.
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
                    expected_type: None,
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
                    expected_type: None,
                },
                // Priority-1 annotation: components must be a list of component maps.
                // AC-015 (BC-1.18.001 postcondition 9):
                //   `components: "see attached"` emits E-VAL-104 T1 ("expected list, got string").
                // Note: per-component `weight` is inside the Map values — it is not a top-level
                // FieldDef. The Float annotation on weight would require a future nested-field
                // validation feature (T3+ scope). The top-level `components` field is annotated
                // as List to catch the most common type error at this level.
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
                    expected_type: Some(FieldType::List),
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
        slide: &Slide,
        _brand: &Brand,
        _canvas: Canvas,
    ) -> Result<LaidOutSlide, LayoutError> {
        use slideforge_layout::types::{BoundingBox, Frame, FrameContent, RegionRole};
        use slideforge_types::Emu;

        // Geometry-only: all content validation (label presence, weight/score range,
        // non-empty components) is performed at Stage 5 (pre-layout) by
        // LabelCheckValidator and ValueRangeValidator. This method produces the
        // static 7-frame skeleton only.
        // See architect adjudication F-087-P1-001 for the full rationale.

        // Produce the static 7-frame skeleton (title + agg-label + 5 component slots).
        let row_base_y = Emu(1_371_600);
        let row_height = Emu(594_360);
        let row_gap = Emu(640_080); // row_height + spacing
        let mut frames = vec![
            Frame {
                bbox: BoundingBox {
                    x: Emu(457_200),
                    y: Emu(274_320),
                    width: Emu(8_229_600),
                    height: Emu(502_920),
                },
                content: FrameContent::Empty,
                text_flow: None,
                region_role: Some(RegionRole::Title),
            },
            Frame {
                bbox: BoundingBox {
                    x: Emu(457_200),
                    y: Emu(822_960),
                    width: Emu(8_229_600),
                    height: Emu(411_480),
                },
                content: FrameContent::Empty,
                text_flow: None,
                region_role: Some(RegionRole::Body),
            },
        ];
        // 5 component row slots.
        for i in 0..5 {
            frames.push(Frame {
                bbox: BoundingBox {
                    x: Emu(457_200),
                    y: Emu(row_base_y.0 + i64::from(i) * row_gap.0),
                    width: Emu(8_229_600),
                    height: row_height,
                },
                content: FrameContent::Empty,
                text_flow: None,
                region_role: Some(RegionRole::Generic),
            });
        }

        Ok(LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::clone(&slide.slide_type),
            frames,
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        })
    }
}
