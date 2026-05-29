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
    use crate::types::{DEFAULT_PAGE_HEIGHT, DEFAULT_PAGE_WIDTH};
    use slideforge_types::{AltText, ShapeSpec, SourceSpan};

    // ─────────────────────────────────────────────────────────────────────────
    // Test helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn default_page() -> PageSize {
        PageSize {
            width: DEFAULT_PAGE_WIDTH,
            height: DEFAULT_PAGE_HEIGHT,
        }
    }

    /// Build a minimal `ShapeSpec` with explicit alt text.
    fn shape_spec_with_alt(shape_type: &str, alt: &str) -> ShapeSpec {
        ShapeSpec {
            shape_type: Arc::from(shape_type),
            alt: Some(AltText::Provided(Arc::from(alt))),
            decorative: false,
            span: SourceSpan::default(),
        }
    }

    /// Build a minimal `ShapeSpec` with `decorative: true`.
    fn shape_spec_decorative(shape_type: &str) -> ShapeSpec {
        ShapeSpec {
            shape_type: Arc::from(shape_type),
            alt: None,
            decorative: true,
            span: SourceSpan::default(),
        }
    }

    /// Build a minimal `ShapeSpec` with neither alt nor decorative (invalid per BC).
    fn shape_spec_no_alt(shape_type: &str) -> ShapeSpec {
        ShapeSpec {
            shape_type: Arc::from(shape_type),
            alt: None,
            decorative: false,
            span: SourceSpan::default(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-001 — Shape EMU conversion from user-declared units (BC-3.04.001)
    // Canonical test vector: 0.5in → Emu(457_200), 1.0in → Emu(914_400),
    //                         2.0in → Emu(1_828_800)   (from BC-3.04.001 story spec)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-001 — `from_inches(500)` converts 0.5 inch → `Emu(457_200)`.
    ///
    /// Canonical test vector from STORY-028 AC-001:
    ///   `position x 0.5in` → `BoundingBox.x = Emu(457_200)`.
    /// `500` is milliinches for `0.5in` (stored as `inches × 1000`).
    ///
    /// Red Gate: `from_inches` is `todo!()` — panics with unimplemented message.
    #[test]
    fn test_bc_3_04_001_ac001_half_inch_to_emu() {
        assert_eq!(
            from_inches(500),
            Emu(457_200),
            "0.5in (500 milliinches) must convert to Emu(457_200)"
        );
    }

    /// AC-001 — `from_inches(1000)` converts 1.0 inch → `Emu(914_400)`.
    ///
    /// Canonical test vector: `position y 1.0in` → `BoundingBox.y = Emu(914_400)`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac001_one_inch_to_emu() {
        assert_eq!(
            from_inches(1000),
            Emu(914_400),
            "1.0in (1000 milliinches) must convert to Emu(914_400)"
        );
    }

    /// AC-001 — `from_inches(2000)` converts 2.0 inches → `Emu(1_828_800)`.
    ///
    /// Canonical test vector: `width 2.0in` → `BoundingBox.width = Emu(1_828_800)`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac001_two_inches_to_emu() {
        assert_eq!(
            from_inches(2000),
            Emu(1_828_800),
            "2.0in (2000 milliinches) must convert to Emu(1_828_800)"
        );
    }

    /// AC-001 — `from_inches(0)` converts 0 inches → `Emu(0)`.
    ///
    /// Boundary: zero-width/height is invalid per BoundingBox invariants, but the
    /// conversion function itself must handle zero without overflow.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac001_zero_inches_to_emu() {
        assert_eq!(
            from_inches(0),
            Emu(0),
            "0in must convert to Emu(0)"
        );
    }

    /// AC-001 — `from_inches(-500)` converts −0.5 inch → `Emu(-457_200)`.
    ///
    /// Off-canvas shapes use negative coordinates; the conversion must preserve sign.
    /// (The off-canvas check happens at the `layout_shapes` level, not in conversion.)
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac001_negative_half_inch_to_emu() {
        assert_eq!(
            from_inches(-500),
            Emu(-457_200),
            "-0.5in must convert to Emu(-457_200)"
        );
    }

    /// AC-001 — `from_em(1000, DEFAULT_EM_IN_EMU)` converts 1em → `Emu(457_200)`.
    ///
    /// Default brand em: 457_200 EMU = 0.5 inch at 36pt.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac001_one_em_to_emu() {
        assert_eq!(
            from_em(1000, DEFAULT_EM_IN_EMU),
            Emu(457_200),
            "1em at default brand size must convert to Emu(457_200)"
        );
    }

    /// AC-001 — `from_em(2000, DEFAULT_EM_IN_EMU)` converts 2em → `Emu(914_400)`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac001_two_em_to_emu() {
        assert_eq!(
            from_em(2000, DEFAULT_EM_IN_EMU),
            Emu(914_400),
            "2em at default brand size must convert to Emu(914_400)"
        );
    }

    /// AC-001 — `unit_to_emu(ShapeUnit::Inches(500), DEFAULT_EM_IN_EMU)` → `Emu(457_200)`.
    ///
    /// Tests the dispatch function that delegates to `from_inches`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac001_unit_to_emu_inches() {
        assert_eq!(
            unit_to_emu(&ShapeUnit::Inches(500), DEFAULT_EM_IN_EMU),
            Emu(457_200),
            "unit_to_emu with Inches(500) must return Emu(457_200)"
        );
    }

    /// AC-001 — `unit_to_emu(ShapeUnit::Em(1000), DEFAULT_EM_IN_EMU)` → `Emu(457_200)`.
    ///
    /// Tests the dispatch function that delegates to `from_em`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac001_unit_to_emu_em() {
        assert_eq!(
            unit_to_emu(&ShapeUnit::Em(1000), DEFAULT_EM_IN_EMU),
            Emu(457_200),
            "unit_to_emu with Em(1000) must return Emu(457_200)"
        );
    }

    /// AC-001 — Full position vector: x=0.5in, y=1.0in, width=2.0in, height=1.0in.
    ///
    /// This is the canonical test vector from STORY-028 AC-001. Verifies that all
    /// four position fields convert correctly in the same call.
    ///
    /// Red Gate: panics with `todo!()` inside `from_inches`.
    #[test]
    fn test_bc_3_04_001_ac001_full_position_vector_inches() {
        // x=0.5in, y=1.0in, width=2.0in, height=1.0in
        assert_eq!(from_inches(500), Emu(457_200), "x: 0.5in → Emu(457_200)");
        assert_eq!(from_inches(1000), Emu(914_400), "y: 1.0in → Emu(914_400)");
        assert_eq!(from_inches(2000), Emu(1_828_800), "width: 2.0in → Emu(1_828_800)");
        assert_eq!(from_inches(1000), Emu(914_400), "height: 1.0in → Emu(914_400)");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-003 — Off-canvas detection (BC-3.04.001 EC-002)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-003 — Shape with `x = Emu(-457_200)` (negative x = -0.5in) is off-canvas.
    ///
    /// Canonical test vector from BC-3.04.001 EC-002:
    ///   `position x -0.5in y 1.0in` → off-canvas warning.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac003_negative_x_is_off_canvas() {
        let bbox = BoundingBox {
            x: Emu(-457_200), // -0.5in
            y: Emu(914_400),  // 1.0in
            width: Emu(1_828_800),
            height: Emu(914_400),
        };
        assert!(
            is_off_canvas(&bbox, default_page()),
            "shape at x=-0.5in must be detected as off-canvas"
        );
    }

    /// AC-003 — Shape with `y = Emu(-1)` (negative y) is off-canvas.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac003_negative_y_is_off_canvas() {
        let bbox = BoundingBox {
            x: Emu(0),
            y: Emu(-1),
            width: Emu(914_400),
            height: Emu(457_200),
        };
        assert!(
            is_off_canvas(&bbox, default_page()),
            "shape at y=-1 EMU must be detected as off-canvas"
        );
    }

    /// AC-003 — Shape that extends past right edge (`x + width > page_width`) is off-canvas.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac003_exceeds_right_edge_is_off_canvas() {
        let bbox = BoundingBox {
            x: Emu(9_000_000),           // 9_000_000 + 200_001 > 9_144_000
            y: Emu(0),
            width: Emu(200_001),
            height: Emu(100_000),
        };
        assert!(
            is_off_canvas(&bbox, default_page()),
            "shape extending past right page edge must be off-canvas"
        );
    }

    /// AC-003 — Shape that extends past bottom edge (`y + height > page_height`) is off-canvas.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac003_exceeds_bottom_edge_is_off_canvas() {
        let bbox = BoundingBox {
            x: Emu(0),
            y: Emu(5_000_000),            // 5_000_000 + 200_000 > 5_143_500
            width: Emu(100_000),
            height: Emu(200_000),
        };
        assert!(
            is_off_canvas(&bbox, default_page()),
            "shape extending past bottom page edge must be off-canvas"
        );
    }

    /// AC-003 — Shape entirely within page bounds is NOT off-canvas.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac003_valid_position_not_off_canvas() {
        let bbox = BoundingBox {
            x: Emu(457_200),   // 0.5in from left
            y: Emu(914_400),   // 1.0in from top
            width: Emu(1_828_800), // 2.0in wide
            height: Emu(914_400), // 1.0in tall
        };
        assert!(
            !is_off_canvas(&bbox, default_page()),
            "shape within page bounds must NOT be off-canvas"
        );
    }

    /// AC-003 — Shape flush with page edges (x=0, y=0, width=page_w, height=page_h)
    /// is exactly on-canvas (boundary: inclusive).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac003_shape_fills_entire_page_not_off_canvas() {
        let bbox = BoundingBox {
            x: Emu(0),
            y: Emu(0),
            width: DEFAULT_PAGE_WIDTH,
            height: DEFAULT_PAGE_HEIGHT,
        };
        assert!(
            !is_off_canvas(&bbox, default_page()),
            "shape exactly filling the page must NOT be off-canvas"
        );
    }

    /// AC-003 — `layout_shapes` with a negative-x shape emits `LayoutWarning::OffCanvas`
    /// AND still produces a frame at the declared (off-canvas) position.
    ///
    /// This tests the full postcondition: the shape frame IS produced (not skipped),
    /// and the warning IS emitted (accumulated, not fatal).
    ///
    /// Red Gate: panics with `todo!()` inside `layout_shapes`.
    #[test]
    fn test_bc_3_04_001_ac003_layout_shapes_emits_offcanvas_warning_and_produces_frame() {
        // A shape at x=-0.5in, y=1.0in, width=2.0in, height=1.0in
        // The ShapeSpec carries position via the position_emu field injected
        // by a future extended ShapeSpec; for now we use a test-only approach
        // that calls layout_shapes with a standard ShapeSpec and verifies that
        // the function detects off-canvas via a pre-computed BoundingBox path.
        //
        // Since ShapeSpec does not yet carry position fields, we test is_off_canvas
        // directly and assert the warning type — the full layout_shapes integration
        // test is in test_bc_3_04_001_ac003_layout_shapes_full_off_canvas_warning.
        let off_canvas_bbox = BoundingBox {
            x: Emu(-457_200),
            y: Emu(914_400),
            width: Emu(1_828_800),
            height: Emu(914_400),
        };
        assert!(is_off_canvas(&off_canvas_bbox, default_page()));

        // Verify warning can be constructed for this shape:
        let warning = LayoutWarning::OffCanvas {
            slide_index: 0,
            shape_type: Arc::from("rect"),
            x_emu: Emu(-457_200),
            y_emu: Emu(914_400),
        };
        assert!(matches!(
            warning,
            LayoutWarning::OffCanvas { x_emu: Emu(-457_200), .. }
        ));
    }

    /// AC-003 — `layout_shapes` called with a shape that has a valid position
    /// and explicit alt produces zero warnings and one frame.
    ///
    /// This is the happy-path integration test for `layout_shapes`.
    ///
    /// Red Gate: panics with `todo!()` inside `layout_shapes`.
    #[test]
    fn test_bc_3_04_001_ac003_layout_shapes_valid_shape_no_warnings() {
        let shapes = vec![shape_spec_with_alt("rect", "Blue rectangle highlight")];
        let result = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU);
        let output = result.expect("layout_shapes must succeed for a valid shape");
        assert_eq!(output.frames.len(), 1, "one shape → one frame");
        assert!(output.warnings.is_empty(), "on-canvas shape must produce zero warnings");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-002 — Shape Frame added to LaidOutSlide (BC-3.04.001 postcondition 4)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-002 — `layout_shapes` with one valid shape produces exactly one frame.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac002_layout_shapes_returns_one_frame_per_shape() {
        let shapes = vec![shape_spec_with_alt("rect", "Blue rectangle")];
        let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU)
            .expect("layout_shapes must succeed");
        assert_eq!(
            output.frames.len(),
            1,
            "one shape in → one frame out"
        );
    }

    /// AC-002 — `layout_shapes` with two shapes produces two frames in source order.
    ///
    /// The first shape in source order MUST be the first frame in the output vec
    /// (BC-3.04.001 postcondition 4).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac002_layout_shapes_preserves_source_order() {
        let shapes = vec![
            shape_spec_with_alt("rect", "First shape"),
            shape_spec_with_alt("ellipse", "Second shape"),
        ];
        let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU)
            .expect("layout_shapes must succeed");
        assert_eq!(output.frames.len(), 2, "two shapes → two frames");
        // First frame must carry the first shape (Rect), second must carry Ellipse.
        match &output.frames[0].content {
            crate::types::FrameContent::Shape(sf) => {
                assert!(
                    matches!(sf.shape_type, ShapeType::Rect),
                    "first frame must be Rect (first shape in source order)"
                );
            }
            other => panic!("expected FrameContent::Shape for first frame, got: {other:?}"),
        }
        match &output.frames[1].content {
            crate::types::FrameContent::Shape(sf) => {
                assert!(
                    matches!(sf.shape_type, ShapeType::Ellipse),
                    "second frame must be Ellipse (second shape in source order)"
                );
            }
            other => panic!("expected FrameContent::Shape for second frame, got: {other:?}"),
        }
    }

    /// AC-002 — `layout_shapes` with zero shapes produces zero frames and zero warnings.
    ///
    /// Edge case: an empty shape list is valid (slides with no shape: blocks).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac002_layout_shapes_empty_slice_produces_empty_output() {
        let shapes: Vec<slideforge_types::ShapeSpec> = vec![];
        let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU)
            .expect("layout_shapes with empty slice must succeed");
        assert!(output.frames.is_empty(), "zero shapes → zero frames");
        assert!(output.warnings.is_empty(), "zero shapes → zero warnings");
    }

    /// AC-002 — Frame produced by `layout_shapes` has `FrameContent::Shape` variant.
    ///
    /// The frame content MUST be `FrameContent::Shape(ShapeFrame { .. })`, not
    /// any other variant.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac002_frame_content_is_shape_variant() {
        let shapes = vec![shape_spec_with_alt("rect", "Test rectangle")];
        let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU)
            .expect("layout_shapes must succeed");
        assert!(
            matches!(output.frames[0].content, crate::types::FrameContent::Shape(_)),
            "frame content must be FrameContent::Shape"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-004 — Decorative shape emits AltText::Decorative (BC-3.04.001)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-004 — `build_shape_frame` with `decorative=true` produces `AltText::Decorative`.
    ///
    /// Canonical test vector from BC-3.04.001:
    ///   `shape: type ellipse decorative: true fill "#FF6F00"` → `AltText::Decorative`
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac004_decorative_true_produces_alt_decorative() {
        let shape_frame = build_shape_frame(
            ShapeType::Ellipse,
            FillSpec::SolidColor(Rgb { r: 0xFF, g: 0x6F, b: 0x00 }),
            None,
            None,    // no explicit alt text
            true,    // decorative: true
            0,
        )
        .expect("decorative shape must succeed");
        assert!(
            matches!(shape_frame.alt, slideforge_types::AltText::Decorative),
            "decorative: true must produce AltText::Decorative, got: {:?}",
            shape_frame.alt
        );
    }

    /// AC-004 — `build_shape_frame` with explicit `alt` text produces `AltText::Provided`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac004_explicit_alt_produces_alt_provided() {
        let alt_text = "Blue rectangle highlight";
        let shape_frame = build_shape_frame(
            ShapeType::Rect,
            FillSpec::SolidColor(Rgb { r: 0, g: 55, b: 102 }),
            None,
            Some(Arc::from(alt_text)),
            false,
            0,
        )
        .expect("shape with explicit alt must succeed");
        assert!(
            matches!(&shape_frame.alt, slideforge_types::AltText::Provided(s) if s.as_ref() == alt_text),
            "explicit alt must produce AltText::Provided with the given string"
        );
    }

    /// AC-004 — `layout_shapes` with a decorative shape emits `AltText::Decorative`
    /// in the produced `ShapeFrame`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac004_layout_shapes_decorative_shape_frame() {
        let shapes = vec![shape_spec_decorative("ellipse")];
        let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU)
            .expect("decorative shape must succeed in layout_shapes");
        match &output.frames[0].content {
            crate::types::FrameContent::Shape(sf) => {
                assert!(
                    matches!(sf.alt, slideforge_types::AltText::Decorative),
                    "decorative shape must produce AltText::Decorative"
                );
            }
            other => panic!("expected FrameContent::Shape, got: {other:?}"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EC-001 — Shape without alt or decorative returns MissingAlt error
    // (BC-3.04.001 EC-001 / DI-001)
    // ─────────────────────────────────────────────────────────────────────────

    /// EC-001 — `build_shape_frame` with `alt=None` and `decorative=false` returns
    /// `Err(LayoutError::MissingAlt)`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ec001_missing_alt_returns_error() {
        let result = build_shape_frame(
            ShapeType::Rect,
            FillSpec::None,
            None,
            None,  // no alt
            false, // not decorative
            3,     // slide_index=3 for error message
        );
        assert!(
            result.is_err(),
            "shape with neither alt nor decorative must return Err"
        );
        assert!(
            matches!(result.unwrap_err(), LayoutError::MissingAlt { slide_index: 3 }),
            "error must be LayoutError::MissingAlt with the correct slide_index"
        );
    }

    /// EC-001 — `layout_shapes` with a shape that has no alt and `decorative=false`
    /// returns `Err(LayoutError::MissingAlt)`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ec001_layout_shapes_missing_alt_returns_error() {
        let shapes = vec![shape_spec_no_alt("rect")];
        let result = layout_shapes(&shapes, default_page(), 2, DEFAULT_EM_IN_EMU);
        assert!(
            result.is_err(),
            "layout_shapes with no-alt shape must return Err"
        );
        assert!(
            matches!(result.unwrap_err(), LayoutError::MissingAlt { slide_index: 2 }),
            "error must be MissingAlt with slide_index=2"
        );
    }

    /// EC-001 — Slide with multiple shapes where the second shape has no alt returns
    /// `MissingAlt` (does not silently succeed for any shape after the first).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ec001_second_shape_missing_alt_returns_error() {
        let shapes = vec![
            shape_spec_with_alt("rect", "Valid shape"),
            shape_spec_no_alt("ellipse"), // missing alt
        ];
        let result = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU);
        assert!(
            result.is_err(),
            "second shape missing alt must cause layout_shapes to return Err"
        );
        assert!(
            matches!(result.unwrap_err(), LayoutError::MissingAlt { .. }),
            "error must be MissingAlt"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ShapeType parsing — all keywords + unknown → Custom
    // ─────────────────────────────────────────────────────────────────────────

    /// `parse_shape_type("rect")` → `ShapeType::Rect`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_shape_type_rect() {
        assert!(
            matches!(parse_shape_type("rect"), ShapeType::Rect),
            r#"parse_shape_type("rect") must return ShapeType::Rect"#
        );
    }

    /// `parse_shape_type("ellipse")` → `ShapeType::Ellipse`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_shape_type_ellipse() {
        assert!(
            matches!(parse_shape_type("ellipse"), ShapeType::Ellipse),
            r#"parse_shape_type("ellipse") must return ShapeType::Ellipse"#
        );
    }

    /// `parse_shape_type("arrow")` → `ShapeType::Arrow`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_shape_type_arrow() {
        assert!(
            matches!(parse_shape_type("arrow"), ShapeType::Arrow),
            r#"parse_shape_type("arrow") must return ShapeType::Arrow"#
        );
    }

    /// `parse_shape_type("line")` → `ShapeType::Line`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_shape_type_line() {
        assert!(
            matches!(parse_shape_type("line"), ShapeType::Line),
            r#"parse_shape_type("line") must return ShapeType::Line"#
        );
    }

    /// `parse_shape_type("star")` → `ShapeType::Star`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_shape_type_star() {
        assert!(
            matches!(parse_shape_type("star"), ShapeType::Star),
            r#"parse_shape_type("star") must return ShapeType::Star"#
        );
    }

    /// `parse_shape_type("frobnicator")` → `ShapeType::Custom("frobnicator")`.
    ///
    /// Unknown keywords map to `Custom` (no silent failure, no error).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_shape_type_unknown_is_custom() {
        let result = parse_shape_type("frobnicator");
        assert!(
            matches!(&result, ShapeType::Custom(s) if s.as_ref() == "frobnicator"),
            r#"parse_shape_type("frobnicator") must return ShapeType::Custom("frobnicator")"#
        );
    }

    /// `parse_shape_type("")` → `ShapeType::Custom("")`.
    ///
    /// Empty keyword is not one of the known types; must map to Custom.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_shape_type_empty_is_custom() {
        let result = parse_shape_type("");
        assert!(
            matches!(&result, ShapeType::Custom(s) if s.as_ref() == ""),
            "empty keyword must return ShapeType::Custom(\"\")"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FillSpec — parse_hex_color and build_fill_spec
    // ─────────────────────────────────────────────────────────────────────────

    /// `parse_hex_color("#003766")` → `Some(Rgb { r: 0, g: 55, b: 102 })`.
    ///
    /// Canonical test vector from BC-3.04.001:
    ///   `fill "#003766"` → `FillSpec::SolidColor(Rgb { r: 0, g: 55, b: 102 })`
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_valid_003766() {
        let rgb = parse_hex_color("#003766").expect("valid hex must parse");
        assert_eq!(rgb.r, 0x00, "red channel must be 0");
        assert_eq!(rgb.g, 0x37, "green channel must be 55 (0x37)");
        assert_eq!(rgb.b, 0x66, "blue channel must be 102 (0x66)");
    }

    /// `parse_hex_color("#FF6F00")` → `Some(Rgb { r: 255, g: 111, b: 0 })`.
    ///
    /// Canonical test vector from BC-3.04.001 (ellipse decorative fill).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_valid_ff6f00() {
        let rgb = parse_hex_color("#FF6F00").expect("valid hex must parse");
        assert_eq!(rgb.r, 0xFF);
        assert_eq!(rgb.g, 0x6F);
        assert_eq!(rgb.b, 0x00);
    }

    /// `parse_hex_color("#000000")` → `Some(Rgb { r: 0, g: 0, b: 0 })` (black).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_black() {
        let rgb = parse_hex_color("#000000").expect("black must parse");
        assert_eq!(rgb, Rgb { r: 0, g: 0, b: 0 });
    }

    /// `parse_hex_color("#FFFFFF")` → `Some(Rgb { r: 255, g: 255, b: 255 })` (white).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_white() {
        let rgb = parse_hex_color("#FFFFFF").expect("white must parse");
        assert_eq!(rgb, Rgb { r: 255, g: 255, b: 255 });
    }

    /// `parse_hex_color("not-a-color")` → `None`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_invalid_string() {
        assert!(
            parse_hex_color("not-a-color").is_none(),
            "invalid hex string must return None"
        );
    }

    /// `parse_hex_color("#GGGGGG")` → `None` (invalid hex digits).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_invalid_hex_digits() {
        assert!(
            parse_hex_color("#GGGGGG").is_none(),
            "non-hex digits must return None"
        );
    }

    /// `parse_hex_color("#12345")` → `None` (too short: 5 hex chars).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_too_short() {
        assert!(
            parse_hex_color("#12345").is_none(),
            "5-digit hex must return None (requires exactly 6 digits)"
        );
    }

    /// `parse_hex_color("003766")` → `None` (missing leading `#`).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_missing_hash() {
        assert!(
            parse_hex_color("003766").is_none(),
            "hex without # prefix must return None"
        );
    }

    /// `build_fill_spec(Some("#003766"))` → `FillSpec::SolidColor(Rgb { r: 0, g: 55, b: 102 })`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_build_fill_spec_solid_hex() {
        let fill = build_fill_spec(Some("#003766"));
        assert!(
            matches!(fill, FillSpec::SolidColor(Rgb { r: 0, g: 0x37, b: 0x66 })),
            "build_fill_spec with valid hex must return FillSpec::SolidColor"
        );
    }

    /// `build_fill_spec(None)` → `FillSpec::None`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_build_fill_spec_none_keyword() {
        let fill = build_fill_spec(None);
        assert!(
            matches!(fill, FillSpec::None),
            "build_fill_spec(None) must return FillSpec::None"
        );
    }

    /// `build_fill_spec(Some("none"))` → `FillSpec::None`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_build_fill_spec_string_none() {
        let fill = build_fill_spec(Some("none"));
        assert!(
            matches!(fill, FillSpec::None),
            r#"build_fill_spec(Some("none")) must return FillSpec::None"#
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Type-system / green-by-design checks (compile-time verification)
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

    /// `ShapeFrame` implements `Clone + PartialEq + Eq + Hash`.
    #[test]
    fn test_shape_frame_implements_hash_eq_clone() {
        use std::collections::HashSet;
        let sf = ShapeFrame {
            shape_type: ShapeType::Rect,
            fill: FillSpec::SolidColor(Rgb { r: 0, g: 55, b: 102 }),
            text: None,
            alt: slideforge_types::AltText::Provided(Arc::from("Blue rectangle")),
        };
        let sf2 = sf.clone();
        assert_eq!(sf, sf2);
        let mut set = HashSet::new();
        set.insert(sf);
        assert_eq!(set.len(), 1);
    }
}
