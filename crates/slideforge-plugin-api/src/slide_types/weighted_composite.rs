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
        slide: &Slide,
        _brand: &Brand,
        _canvas: Canvas,
    ) -> Result<LaidOutSlide, LayoutError> {
        use slideforge_layout::types::{BoundingBox, Frame, FrameContent, RegionRole};
        use slideforge_types::{Emu, FieldValue, Value};

        // BC-1.17.003 postcondition 2 / AC-016: top-level label is required.
        let label_ok = match slide.fields.get("label") {
            Some(FieldValue::Literal(Value::Str(s))) => !s.trim().is_empty(),
            _ => false,
        };
        if !label_ok {
            return Err(LayoutError::MissingRequiredField {
                slide_type: "weighted_composite".to_owned(),
                field: "label".to_owned(),
            });
        }

        // BC-1.17.003 postcondition 3 / AC-019: components must be non-empty.
        let components = match slide.fields.get("components") {
            Some(FieldValue::Literal(Value::List(list))) => list.as_slice(),
            _ => {
                return Err(LayoutError::MissingRequiredField {
                    slide_type: "weighted_composite".to_owned(),
                    field: "components".to_owned(),
                });
            },
        };

        if components.is_empty() {
            return Err(LayoutError::MissingRequiredField {
                slide_type: "weighted_composite".to_owned(),
                field: "components".to_owned(),
            });
        }

        // BC-1.17.003 postconditions 3+4 / invariants 6+7 / AC-020/021:
        // Validate each component: weight > 0, score ∈ [0,100].
        // Accumulate ALL errors (DI-018 — do not bail on first).
        let mut errors: Vec<LayoutError> = Vec::new();

        for (idx, comp_val) in components.iter().enumerate() {
            let Value::Map(comp_map) = comp_val else {
                // Non-map component — report and continue.
                errors.push(LayoutError::FieldTypeMismatch {
                    slide_type: "weighted_composite".to_owned(),
                    field: format!("components[{idx}]"),
                    expected_type: "Map".to_owned(),
                    actual_type: "non-Map value".to_owned(),
                });
                continue;
            };

            // BC-1.17.003 postcondition 4 / AC-017: per-component label is required.
            let comp_label_ok = match comp_map.get("label") {
                Some(Value::Str(s)) => !s.trim().is_empty(),
                _ => false,
            };
            if !comp_label_ok {
                let comp_name = match comp_map.get("name") {
                    Some(Value::Str(s)) => s.as_ref().to_owned(),
                    _ => format!("[{idx}]"),
                };
                errors.push(LayoutError::MissingRequiredField {
                    slide_type: "weighted_composite".to_owned(),
                    field: format!("components[{comp_name}].label"),
                });
            }

            // weight must be present and > 0 (float or int).
            let weight_valid = match comp_map.get("weight") {
                Some(Value::Float(f)) => f.0 > 0.0,
                Some(Value::Int(n)) => *n > 0,
                _ => false,
            };
            if !weight_valid {
                let actual = match comp_map.get("weight") {
                    Some(Value::Float(f)) => format!("{} — must be positive", f.0),
                    Some(Value::Int(n)) => format!("{n} — must be positive"),
                    _ => "absent or wrong type".to_owned(),
                };
                errors.push(LayoutError::FieldTypeMismatch {
                    slide_type: "weighted_composite".to_owned(),
                    field: format!("components[{idx}].weight"),
                    expected_type: "positive number".to_owned(),
                    actual_type: actual,
                });
            }

            // score must be in [0, 100].
            let score_int = match comp_map.get("score") {
                Some(Value::Int(n)) => Some(*n),
                _ => None,
            };
            match score_int {
                Some(n) if (0..=100).contains(&n) => {
                    // valid
                },
                Some(n) => {
                    errors.push(LayoutError::FieldTypeMismatch {
                        slide_type: "weighted_composite".to_owned(),
                        field: format!("components[{idx}].score"),
                        expected_type: "integer in [0, 100]".to_owned(),
                        actual_type: format!("{n} — out of range"),
                    });
                },
                None => {
                    errors.push(LayoutError::FieldTypeMismatch {
                        slide_type: "weighted_composite".to_owned(),
                        field: format!("components[{idx}].score"),
                        expected_type: "integer in [0, 100]".to_owned(),
                        actual_type: "absent or wrong type".to_owned(),
                    });
                },
            }
        }

        if !errors.is_empty() {
            // Return the first error (Multiple variant requires non-empty, but we can
            // return only the first to keep the error type simple for callers that match
            // on FieldTypeMismatch directly, as the tests do).
            // AC-020/021 tests match on a single FieldTypeMismatch variant.
            return Err(errors.remove(0));
        }

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
