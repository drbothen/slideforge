//! The `status` slide type — a single-item color-coded status indicator.
//!
//! A `status` slide displays a single status indicator where color conveys
//! state (e.g., red = at risk, green = on track, amber = attention). Because
//! WCAG AA prohibits color as the sole conveyor of meaning, the `label` field
//! is mandatory and provides the accessible text co-encoding.
//!
//! DSL keyword: `status`
//!
//! Required fields: `title`, `label`
//!
//! # References
//!
//! - BC-1.17.001 — status Slide Type Requires title + label
//! - STORY-087 — Color-Coded Slide Types (Wave 4)

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

use super::common_optional_fields;

/// The built-in `status` slide type.
///
/// Displays a single-item status indicator where color conveys state. The
/// `label` field is mandatory (WCAG 1.4.1: Use of Color) and must co-encode
/// the status meaning in text (e.g., `label "On Track"`).
///
/// Required fields: `title`, `label`.
/// Optional fields: common optional fields (notes, report, detail, tags, etc.).
///
/// Maps to the `"Blank"` OOXML layout (custom geometry produced by `lay_out`).
#[derive(Debug)]
pub struct StatusSlideType {
    /// Required fields for the status slide type.
    required: Vec<FieldDef>,
    /// Optional fields for the status slide type.
    optional: Vec<FieldDef>,
}

impl StatusSlideType {
    /// Construct a new `StatusSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        let optional = common_optional_fields();
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from(
                        "The title identifying what is being statused \
                         (e.g., \"Project Alpha\"). Required; non-empty.",
                    ),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("label"),
                    description: Arc::from(
                        "The accessible text co-encoding for the color-coded status \
                         (e.g., \"On Track\", \"At Risk\"). Mandatory per WCAG 1.4.1 \
                         (Use of Color). Empty or absent → E-A11-002.",
                    ),
                    required: true,
                    default_value: None,
                },
            ],
            optional,
        }
    }
}

impl Default for StatusSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for StatusSlideType {
    fn id(&self) -> &'static str {
        "status"
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

        // Geometry-only: label validation is performed by LabelCheckValidator
        // at Stage 5 (pre-layout). This method produces the static two-frame
        // skeleton only.
        // See architect adjudication F-087-P1-001 for the full rationale.

        // Produce the static two-frame skeleton matching the region map:
        // Frame 0 — color indicator strip (Generic role)
        // Frame 1 — label + title body (Body role)
        let frames = vec![
            Frame {
                bbox: BoundingBox {
                    x: Emu(457_200),
                    y: Emu(365_760),
                    width: Emu(685_800),
                    height: Emu(4_114_800),
                },
                content: FrameContent::Empty,
                text_flow: None,
                region_role: Some(RegionRole::Generic),
            },
            Frame {
                bbox: BoundingBox {
                    x: Emu(1_371_600),
                    y: Emu(365_760),
                    width: Emu(7_315_200),
                    height: Emu(4_114_800),
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
