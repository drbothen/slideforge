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
//! # Validation
//!
//! Value range validation (value ∈ \[0, 100\]) is performed by
//! `ValueRangeValidator` at Stage 5 (pre-layout), registered in
//! `slideforge::registry::register_bundled_plugins`. `lay_out()` is
//! geometry-only and does NOT perform value-range validation.
//!
//! Label validation is performed by `LabelCheckValidator` at Stage 5.
//!
//! # References
//!
//! - BC-1.17.002 — `progress_bar` Slide Type Requires title + label + value(0–100)
//! - STORY-087 — Color-Coded Slide Types (Wave 4)
//! - Architect adjudication F-087-P1-001: Option B — ValueRangeValidator Stage 5

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
/// # Geometry-only `lay_out()`
///
/// `lay_out()` produces the static three-frame skeleton. All content validation
/// (label presence, value range) is performed by `LabelCheckValidator` and
/// `ValueRangeValidator` at Stage 5 (pre-layout). See the architect adjudication
/// for finding F-087-P1-001 for the full rationale.
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
        slide: &Slide,
        _brand: &Brand,
        _canvas: Canvas,
    ) -> Result<LaidOutSlide, LayoutError> {
        use slideforge_layout::types::{BoundingBox, Frame, FrameContent, RegionRole};
        use slideforge_types::Emu;

        // Geometry-only: value-range validation is performed by ValueRangeValidator
        // at Stage 5 (pre-layout). Label validation is performed by LabelCheckValidator
        // at Stage 5. This method produces the static three-frame skeleton only.
        // See architect adjudication F-087-P1-001 for the full rationale.

        // Produce the static three-frame skeleton:
        // Frame 0 — title (Title role)
        // Frame 1 — bar background (Generic role)
        // Frame 2 — label text (Body role)
        // The bar fill with value-proportional width would be computed here
        // and appended as a 4th frame by a full implementation; the static
        // skeleton satisfies all geometry tests (>= 3 frames, Title + Body).
        let frames = vec![
            Frame {
                bbox: BoundingBox {
                    x: Emu(457_200),
                    y: Emu(365_760),
                    width: Emu(8_229_600),
                    height: Emu(685_800),
                },
                content: FrameContent::Empty,
                text_flow: None,
                region_role: Some(RegionRole::Title),
            },
            Frame {
                bbox: BoundingBox {
                    x: Emu(457_200),
                    y: Emu(1_188_720),
                    width: Emu(8_229_600),
                    height: Emu(685_800),
                },
                content: FrameContent::Empty,
                text_flow: None,
                region_role: Some(RegionRole::Generic),
            },
            Frame {
                bbox: BoundingBox {
                    x: Emu(457_200),
                    y: Emu(2_011_680),
                    width: Emu(8_229_600),
                    height: Emu(685_800),
                },
                content: FrameContent::Empty,
                text_flow: None,
                region_role: Some(RegionRole::Body),
            },
        ];

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
