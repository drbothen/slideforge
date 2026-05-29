//! Shape layout pass — BC-3.04.001 (STORY-028).
//!
//! This module converts [`slideforge_types::ShapeSpec`] nodes from the `Deck`
//! IR into [`crate::types::Frame`]s with [`crate::types::FrameContent::Shape`]
//! content, appended after all placeholder frames in each [`crate::types::LaidOutSlide`].
//!
//! ## Responsibilities
//!
//! - Convert user-unit positions (inches, em) to integer [`slideforge_types::Emu`].
//! - Enforce that every shape has `alt` or `decorative: true`
//!   (defensive check — primary enforcement is in `slideforge-validate`).
//! - Detect off-canvas positions and emit [`crate::types::LayoutWarning::OffCanvas`].
//! - Preserve source order (first shape in source → first shape frame appended).
//!
//! ## Unit conversion (AC-001)
//!
//! | User unit | EMU conversion |
//! |-----------|---------------|
//! | `1in`     | `914_400 EMU` |
//! | `1em`     | `brand.font_size_emu` (default: `457_200` = 0.5 inch at 36pt) |
//!
//! ## Off-canvas detection (AC-003 / BC-3.04.001 EC-002)
//!
//! A shape is off-canvas if any of the following hold:
//! - `x < 0`
//! - `y < 0`
//! - `x + width > page_width`
//! - `y + height > page_height`
//!
//! Off-canvas shapes emit [`crate::types::LayoutWarning::OffCanvas`] but are
//! still produced at their declared position.

use std::sync::Arc;

use slideforge_types::Emu;

use crate::error::LayoutError;
use crate::types::{
    BoundingBox, FillSpec, Frame, LayoutWarning, PageSize, Rgb, ShapeFrame, ShapeType,
};

/// EMU per inch: 914,400 (canonical DSL unit definition, DI-010).
pub const EMU_PER_INCH: i64 = 914_400;

/// Default em-to-EMU conversion: 457,200 EMU = 0.5 inch at 36pt brand default.
///
/// Used when `brand.font_size_emu` is not overridden.
pub const DEFAULT_EM_IN_EMU: i64 = 457_200;

/// A raw shape position from the DSL, before EMU conversion.
///
/// Produced by the DSL parser and carried in the `ShapeSpec` until the layout
/// pass converts all measurements to integer EMU.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapePosition {
    /// x coordinate in raw user units.
    pub x: ShapeUnit,
    /// y coordinate in raw user units.
    pub y: ShapeUnit,
    /// width in raw user units.
    pub width: ShapeUnit,
    /// height in raw user units.
    pub height: ShapeUnit,
}

/// A measurement in a user-facing unit (inches or em).
///
/// The layout pass converts these to integer EMU via [`unit_to_emu`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShapeUnit {
    /// Measurement in inches, stored as thousandths of an inch (i.e., `0.5in`
    /// is `ShapeUnit::Inches(500)`). The inner value is `inches × 1000` as
    /// an integer to avoid `f64`.
    ///
    /// Conversion: `emu = (milliinches * 914_400) / 1_000`.
    Inches(i64),
    /// Measurement in em units, stored as thousandths of an em.
    ///
    /// Conversion: `emu = (milliem * em_in_emu) / 1_000`.
    Em(i64),
}

/// Convert a [`ShapeUnit`] measurement to integer EMU.
///
/// # Arguments
///
/// * `unit` — the measurement in user units.
/// * `em_in_emu` — the brand's em-to-EMU resolution (default [`DEFAULT_EM_IN_EMU`]).
///
/// # Returns
///
/// The measurement in integer EMU (`Emu(i64)`).
///
/// # Green-by-design self-check (BC-5.38.005)
///
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — YES for the arithmetic formula,
/// but the function has branching on the unit variant (not zero branching) and
/// calls `Emu(...)` constructor. Therefore it is NOT GREEN-BY-DESIGN and must
/// remain `todo!()`.
#[must_use]
pub fn unit_to_emu(unit: &ShapeUnit, em_in_emu: i64) -> Emu {
    todo!("BC-3.04.001 AC-001: convert ShapeUnit to integer EMU; inches use EMU_PER_INCH, em uses em_in_emu")
}

/// Convert inches (as a rational `numerator/1000`) to EMU.
///
/// For example, `from_inches(500)` converts `0.5in` → `Emu(457_200)`.
///
/// # Green-by-design self-check (BC-5.38.005)
///
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — NO for a helper that is tested
/// directly (the test would need the implementation). Kept `todo!()`.
#[must_use]
pub fn from_inches(milliinches: i64) -> Emu {
    todo!("BC-3.04.001 AC-001: from_inches — emu = (milliinches * EMU_PER_INCH) / 1_000")
}

/// Convert em units (as a rational `numerator/1000`) to EMU.
///
/// For example, `from_em(1000, DEFAULT_EM_IN_EMU)` converts `1em` → `Emu(457_200)`.
///
/// # Green-by-design self-check (BC-5.38.005)
///
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — NO. Kept `todo!()`.
#[must_use]
pub fn from_em(milliem: i64, em_in_emu: i64) -> Emu {
    todo!("BC-3.04.001 AC-001: from_em — emu = (milliem * em_in_emu) / 1_000")
}

/// Detect whether a bounding box is off the slide canvas.
///
/// A bounding box is off-canvas if `x < 0`, `y < 0`,
/// `x + width > page_width`, or `y + height > page_height`.
///
/// Returns `true` if the shape is (at least partially) off-canvas.
///
/// # Green-by-design self-check (BC-5.38.005)
///
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — NO. Has branching (`<`, `>`).
/// Kept `todo!()`.
#[must_use]
pub fn is_off_canvas(bbox: &BoundingBox, page: PageSize) -> bool {
    todo!(
        "BC-3.04.001 EC-002: is_off_canvas — check x<0, y<0, x+w>page_w, y+h>page_h"
    )
}

/// Parse a shape type keyword string into a [`ShapeType`] enum variant.
///
/// Known keywords: `"rect"`, `"ellipse"`, `"arrow"`, `"line"`, `"star"`.
/// Any other keyword maps to `ShapeType::Custom(Arc::from(keyword))`.
///
/// # Green-by-design self-check (BC-5.38.005)
///
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — NO. Has a match with multiple
/// arms over a string. Kept `todo!()`.
#[must_use]
pub fn parse_shape_type(keyword: &str) -> ShapeType {
    todo!(
        "BC-3.04.001: parse_shape_type — map keyword string to ShapeType variant"
    )
}

/// Parse a CSS-style hex color string (`#RRGGBB`) into an [`Rgb`] value.
///
/// Returns `None` if the string is not a valid 6-digit hex color.
///
/// # Green-by-design self-check (BC-5.38.005)
///
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — NO. Involves parsing and error
/// handling. Kept `todo!()`.
#[must_use]
pub fn parse_hex_color(hex: &str) -> Option<Rgb> {
    todo!(
        "BC-3.04.001: parse_hex_color — parse '#RRGGBB' into Rgb {{ r, g, b }}"
    )
}

/// The output of [`layout_shapes`].
///
/// Carries the produced shape frames and any accumulated warnings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapeLayoutOutput {
    /// Shape frames to append to the slide's `frames` list.
    pub frames: Vec<Frame>,
    /// Non-fatal warnings (off-canvas positions, etc.).
    pub warnings: Vec<LayoutWarning>,
}

/// Layout all `shape:` blocks from a semantic slide into [`Frame`]s.
///
/// This is the entry point for the shape layout pass (BC-3.04.001 postcondition
/// 4). Called once per slide by [`crate::layout::run`] after the region-map
/// frames have been produced.
///
/// # Arguments
///
/// * `shapes` — slice of `ShapeSpec` from the slide's block list.
/// * `page` — the slide's page dimensions (for off-canvas detection).
/// * `slide_index` — zero-based index of the slide (for warning/error messages).
/// * `em_in_emu` — the brand's em-to-EMU resolution.
///
/// # Errors
///
/// Returns `Err(LayoutError::MissingAlt)` if any shape has neither `alt` nor
/// `decorative: true` (EC-001 / DI-001).
///
/// # Green-by-design self-check (BC-5.38.005)
///
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — NO. Involves I/O-free but
/// multi-step logic (convert units, detect off-canvas, build frames). `todo!()`.
pub fn layout_shapes(
    shapes: &[slideforge_types::ShapeSpec],
    page: PageSize,
    slide_index: usize,
    em_in_emu: i64,
) -> Result<ShapeLayoutOutput, LayoutError> {
    todo!(
        "BC-3.04.001: layout_shapes — iterate shapes, convert positions to EMU, detect off-canvas, \
         return ShapeLayoutOutput with Frame list and warnings"
    )
}

/// Build a [`FillSpec`] from a shape spec's fill keyword.
///
/// Supported keywords in v1.0: solid color hex (`#RRGGBB`), gradient (`from:…/to:…`),
/// `"none"`. Unrecognised keywords default to `FillSpec::None`.
///
/// # Green-by-design self-check (BC-5.38.005)
///
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — NO. Has parsing + branching. `todo!()`.
#[must_use]
pub fn build_fill_spec(fill_keyword: Option<&str>) -> FillSpec {
    todo!(
        "BC-3.04.001: build_fill_spec — parse fill keyword into FillSpec variant"
    )
}

/// Build a [`ShapeFrame`] from the resolved components.
///
/// # Errors
///
/// Returns `Err(LayoutError::MissingAlt)` when `alt` is `None` and `decorative`
/// is `false` (BC-3.04.001 EC-001 / DI-001).
///
/// # Green-by-design self-check (BC-5.38.005)
///
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — NO. Branches on alt/decorative.
/// `todo!()`.
pub fn build_shape_frame(
    shape_type: ShapeType,
    fill: FillSpec,
    text: Option<Vec<slideforge_types::InlineNode>>,
    alt: Option<Arc<str>>,
    decorative: bool,
    slide_index: usize,
) -> Result<ShapeFrame, LayoutError> {
    todo!(
        "BC-3.04.001: build_shape_frame — resolve AltText, return ShapeFrame or MissingAlt error"
    )
}

#[cfg(test)]
#[allow(clippy::missing_docs_in_private_items, clippy::unwrap_used)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use crate::types::{DEFAULT_PAGE_HEIGHT, DEFAULT_PAGE_WIDTH};
    #[allow(unused_imports)]
    use slideforge_types::{AltText, ShapeSpec, SourceSpan};

    // ─────────────────────────────────────────────────────────────────────────
    // AC-001 — Shape EMU conversion from user-declared units (BC-3.04.001)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-001 — `from_inches(500)` converts 0.5in → Emu(457_200).
    ///
    /// Red Gate: `from_inches` is `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac001_half_inch_to_emu() {
        panic!("not yet implemented (Red Gate): from_inches(500) must return Emu(457_200)")
    }

    /// AC-001 — `from_inches(1000)` converts 1.0in → Emu(914_400).
    ///
    /// Red Gate: `from_inches` is `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac001_one_inch_to_emu() {
        panic!("not yet implemented (Red Gate): from_inches(1000) must return Emu(914_400)")
    }

    /// AC-001 — `from_inches(2000)` converts 2.0in → Emu(1_828_800).
    ///
    /// Red Gate: `from_inches` is `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac001_two_inches_to_emu() {
        panic!("not yet implemented (Red Gate): from_inches(2000) must return Emu(1_828_800)")
    }

    /// AC-001 — `from_em(1000, DEFAULT_EM_IN_EMU)` converts 1em → Emu(457_200).
    ///
    /// Red Gate: `from_em` is `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac001_one_em_to_emu() {
        panic!(
            "not yet implemented (Red Gate): from_em(1000, DEFAULT_EM_IN_EMU) must return Emu(457_200)"
        )
    }

    /// AC-001 — `unit_to_emu(ShapeUnit::Inches(500), DEFAULT_EM_IN_EMU)` → Emu(457_200).
    ///
    /// Red Gate: `unit_to_emu` is `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac001_unit_to_emu_inches() {
        panic!("not yet implemented (Red Gate): unit_to_emu with Inches must convert correctly")
    }

    /// AC-001 — `unit_to_emu(ShapeUnit::Em(1000), DEFAULT_EM_IN_EMU)` → Emu(457_200).
    ///
    /// Red Gate: `unit_to_emu` is `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac001_unit_to_emu_em() {
        panic!("not yet implemented (Red Gate): unit_to_emu with Em must convert correctly")
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-003 — Off-canvas detection (BC-3.04.001 EC-002)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-003 — Shape with x = -1 (negative) is detected as off-canvas.
    ///
    /// Red Gate: `is_off_canvas` is `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac003_negative_x_is_off_canvas() {
        panic!("not yet implemented (Red Gate): is_off_canvas must return true for x < 0")
    }

    /// AC-003 — Shape entirely inside page is NOT off-canvas.
    ///
    /// Red Gate: `is_off_canvas` is `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac003_valid_position_not_off_canvas() {
        panic!("not yet implemented (Red Gate): is_off_canvas must return false for valid bbox")
    }

    /// AC-003 — `layout_shapes` with negative-x shape emits OffCanvas warning AND
    /// still produces a frame.
    ///
    /// Red Gate: `layout_shapes` is `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac003_layout_shapes_emits_offcanvas_warning() {
        panic!(
            "not yet implemented (Red Gate): layout_shapes must emit LayoutWarning::OffCanvas for \
             a shape with negative x position and still produce the frame"
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-004 — Decorative shape emits AltText::Decorative (BC-3.04.001)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-004 — Shape with `decorative: true` produces `AltText::Decorative`.
    ///
    /// Red Gate: `build_shape_frame` is `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac004_decorative_shape_produces_alt_decorative() {
        panic!(
            "not yet implemented (Red Gate): build_shape_frame with decorative=true must produce \
             ShapeFrame {{ alt: AltText::Decorative, .. }}"
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EC-001 — Shape without alt or decorative returns MissingAlt error
    // ─────────────────────────────────────────────────────────────────────────

    /// EC-001 — Shape with no alt and decorative=false returns MissingAlt error.
    ///
    /// Red Gate: `build_shape_frame` is `todo!()`.
    #[test]
    fn test_bc_3_04_001_ec001_missing_alt_returns_error() {
        panic!(
            "not yet implemented (Red Gate): build_shape_frame with alt=None, decorative=false \
             must return Err(LayoutError::MissingAlt)"
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-002 — Shape Frame added to LaidOutSlide.frames (BC-3.04.001)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-002 — `layout_shapes` with one valid shape returns one frame.
    ///
    /// Red Gate: `layout_shapes` is `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac002_layout_shapes_returns_one_frame_per_shape() {
        panic!(
            "not yet implemented (Red Gate): layout_shapes with one ShapeSpec must return \
             ShapeLayoutOutput with frames.len() == 1"
        )
    }

    /// AC-002 — `layout_shapes` preserves source order (first shape → first frame appended).
    ///
    /// Red Gate: `layout_shapes` is `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac002_layout_shapes_preserves_source_order() {
        panic!(
            "not yet implemented (Red Gate): layout_shapes must preserve source order of shapes"
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ShapeType and FillSpec parsing helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// `parse_shape_type("rect")` returns `ShapeType::Rect`.
    ///
    /// Red Gate: `parse_shape_type` is `todo!()`.
    #[test]
    fn test_parse_shape_type_rect() {
        panic!("not yet implemented (Red Gate): parse_shape_type(\"rect\") must return ShapeType::Rect")
    }

    /// `parse_shape_type("ellipse")` returns `ShapeType::Ellipse`.
    ///
    /// Red Gate: `parse_shape_type` is `todo!()`.
    #[test]
    fn test_parse_shape_type_ellipse() {
        panic!(
            "not yet implemented (Red Gate): parse_shape_type(\"ellipse\") must return ShapeType::Ellipse"
        )
    }

    /// `parse_shape_type("frobnicator")` returns `ShapeType::Custom(...)`.
    ///
    /// Red Gate: `parse_shape_type` is `todo!()`.
    #[test]
    fn test_parse_shape_type_custom() {
        panic!(
            "not yet implemented (Red Gate): parse_shape_type with unknown keyword must return \
             ShapeType::Custom"
        )
    }

    /// `parse_hex_color("#003766")` returns `Some(Rgb { r: 0, g: 55, b: 102 })`.
    ///
    /// Red Gate: `parse_hex_color` is `todo!()`.
    #[test]
    fn test_parse_hex_color_valid() {
        panic!("not yet implemented (Red Gate): parse_hex_color must parse valid 6-digit hex")
    }

    /// `parse_hex_color("not-a-color")` returns `None`.
    ///
    /// Red Gate: `parse_hex_color` is `todo!()`.
    #[test]
    fn test_parse_hex_color_invalid() {
        panic!("not yet implemented (Red Gate): parse_hex_color must return None for invalid input")
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Type system / GREEN-BY-DESIGN checks
    // ─────────────────────────────────────────────────────────────────────────

    /// `ShapeLayoutOutput` implements `Clone + PartialEq + Eq + Hash`.
    #[test]
    fn test_shape_layout_output_implements_hash_eq_clone() {
        use std::collections::HashSet;
        let output = ShapeLayoutOutput {
            frames: vec![],
            warnings: vec![],
        };
        let output2 = output.clone();
        assert_eq!(output, output2);
        let mut set = HashSet::new();
        set.insert(output);
        assert_eq!(set.len(), 1);
    }

    /// `ShapeUnit` variants can be constructed and compared.
    #[test]
    fn test_shape_unit_variants_constructable() {
        let inches = ShapeUnit::Inches(500);
        let em = ShapeUnit::Em(1000);
        assert!(matches!(inches, ShapeUnit::Inches(500)));
        assert!(matches!(em, ShapeUnit::Em(1000)));
    }

    /// `LayoutWarning::OffCanvas` can be constructed and is hashable.
    #[test]
    fn test_layout_warning_offcanvas_constructable() {
        use std::collections::HashSet;
        use std::sync::Arc;
        let w = LayoutWarning::OffCanvas {
            slide_index: 0,
            shape_type: Arc::from("rect"),
            x_emu: Emu(-457_200),
            y_emu: Emu(914_400),
        };
        let w2 = w.clone();
        assert_eq!(w, w2);
        let mut set = HashSet::new();
        set.insert(w);
        assert_eq!(set.len(), 1);
    }

    /// `LayoutWarning::XrefTargetNotFound` can be constructed and is hashable.
    #[test]
    fn test_layout_warning_xref_not_found_constructable() {
        use std::collections::HashSet;
        use std::sync::Arc;
        let w = LayoutWarning::XrefTargetNotFound {
            target: Arc::from("slide-99"),
            slide_index: 3,
        };
        let w2 = w.clone();
        assert_eq!(w, w2);
        let mut set = HashSet::new();
        set.insert(w);
        assert_eq!(set.len(), 1);
    }
}
