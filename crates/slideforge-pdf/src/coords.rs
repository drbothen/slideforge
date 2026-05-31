//! EMU-to-PDF coordinate mapping for `slideforge-pdf`.
//!
//! This module provides two **pure** coordinate conversion functions that are
//! the exclusive mechanism for all EMU-to-point conversions in the PDF export
//! path. Both functions are candidates for Kani proofs in Phase 6 (VP-006).
//!
//! ## Conversion rationale
//!
//! OOXML specifies 1 inch = 914,400 EMU and 1 inch = 72 PDF points, therefore
//! 1 point = 914,400 / 72 = 12,700 EMU exactly.
//!
//! ## Y-axis flip
//!
//! The PPTX IR has origin at top-left with Y increasing downward. PDF has origin
//! at bottom-left with Y increasing upward. `ir_y_to_pdf_y` performs the flip:
//!
//! ```text
//! pdf_y = slide_height_pt − ir_y_pt − element_height_pt
//! ```
//!
//! ## Architecture invariant (BC-4.03.005)
//!
//! Every EMU-to-point conversion in `slideforge-pdf` MUST go through
//! [`emu_to_pt`]. Ad-hoc inline `emu / 12700.0` arithmetic is forbidden: it
//! prevents the Kani proof from covering all conversion sites.

use slideforge_types::Emu;

// ─── Standard slide dimensions (16:9 canvas, BC-4.03.005) ────────────────────

/// Standard 16:9 slide width: 10 inches = 720 PDF points = 9,144,000 EMU.
///
/// This matches [`slideforge_layout::types::DEFAULT_PAGE_WIDTH`].
pub const SLIDE_WIDTH_EMU: Emu = Emu(9_144_000);

/// Standard 16:9 slide height: 5.625 inches = 405 PDF points = 5,143,500 EMU.
///
/// This matches [`slideforge_layout::types::DEFAULT_PAGE_HEIGHT`].
pub const SLIDE_HEIGHT_EMU: Emu = Emu(5_143_500);

/// Standard 16:9 slide width in PDF points (720.0 pt).
pub const SLIDE_WIDTH_PT: f32 = 720.0;

/// Standard 16:9 slide height in PDF points (405.0 pt).
pub const SLIDE_HEIGHT_PT: f32 = 405.0;

// ─── Conversion functions ─────────────────────────────────────────────────────

/// Convert an EMU value to PDF user units (points).
///
/// Formula: `emu.0 as f32 / 12_700.0`
///
/// This is a **pure function** with no side effects — it is a candidate for
/// a Kani proof in Phase 6 (VP-006). No logging, no mutation, no I/O.
///
/// # Precision
///
/// The returned `f32` has a rounding error < 0.001 points for all standard
/// slide dimension values (0 ≤ EMU ≤ 9,144,000), as required by
/// BC-4.03.005 postcondition 5 (AC-003).
///
/// # Examples
///
/// ```
/// use slideforge_pdf::coords::emu_to_pt;
/// use slideforge_types::Emu;
///
/// assert_eq!(emu_to_pt(Emu(9_144_000)), 720.0_f32);
/// assert_eq!(emu_to_pt(Emu(0)), 0.0_f32);
/// assert_eq!(emu_to_pt(Emu(12_700)), 1.0_f32);
/// ```
#[inline]
#[must_use]
pub fn emu_to_pt(_emu: Emu) -> f32 {
    // STORY-044 Red Gate: implementation is `todo!()` — this function body
    // will be replaced by the implementer with:
    //   _emu.0 as f32 / 12_700.0
    todo!("STORY-044: implement emu_to_pt — emu.0 as f32 / 12_700.0")
}

/// Convert an IR Y-coordinate to a PDF Y-coordinate with Y-axis flip.
///
/// PDF origin is at the **bottom-left**; PPTX IR origin is at the
/// **top-left**. This function applies the inversion:
///
/// ```text
/// pdf_y = emu_to_pt(slide_h) − emu_to_pt(ir_y) − emu_to_pt(element_h)
/// ```
///
/// This is a **pure function** with no side effects — it is a candidate for
/// a Kani proof in Phase 6 (VP-006). No logging, no mutation, no I/O.
///
/// # Parameters
///
/// - `ir_y` — Y position of the element's top edge in IR coordinates.
/// - `element_h` — height of the element.
/// - `slide_h` — height of the slide (use `SLIDE_HEIGHT_EMU` for standard
///   16:9; use the brand's slide height for non-standard sizes).
///
/// # Examples
///
/// ```
/// use slideforge_pdf::coords::{ir_y_to_pdf_y, SLIDE_HEIGHT_EMU};
/// use slideforge_types::Emu;
///
/// // Top-left element of height 100pt: pdf_y = 405 − 0 − 100 = 305
/// assert_eq!(
///     ir_y_to_pdf_y(Emu(0), Emu(100 * 12_700), SLIDE_HEIGHT_EMU),
///     305.0_f32,
/// );
/// ```
#[inline]
#[must_use]
pub fn ir_y_to_pdf_y(_ir_y: Emu, _element_h: Emu, _slide_h: Emu) -> f32 {
    // STORY-044 Red Gate: implementation is `todo!()` — this function body
    // will be replaced by the implementer with:
    //   emu_to_pt(_slide_h) - emu_to_pt(_ir_y) - emu_to_pt(_element_h)
    todo!("STORY-044: implement ir_y_to_pdf_y — emu_to_pt(slide_h) - emu_to_pt(ir_y) - emu_to_pt(element_h)")
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── AC-001: emu_to_pt canonical test vectors ──────────────────────────────
    //
    // All tests in this section call `emu_to_pt()` or `ir_y_to_pdf_y()` which
    // have `todo!()` bodies. A `todo!()` panics at runtime. nextest/libtest
    // catches the panic and reports the test as FAILED. This is the correct
    // Red Gate behavior: tests FAIL before implementation, PASS after.
    //
    // Do NOT use `#[should_panic]` here — that would make the test PASS on the
    // `todo!()` panic, defeating the Red Gate.

    /// BC-4.03.005 AC-001 / postcondition 1: slide width (10 inches) converts to 720.0pt.
    ///
    /// Exercises VP-006. FAILS at Red Gate (todo!() panic). PASSES after implementation.
    #[test]
    fn test_bc_4_03_005_emu_to_pt_slide_width_720() {
        assert_eq!(
            emu_to_pt(SLIDE_WIDTH_EMU),
            720.0_f32,
            "emu_to_pt(9_144_000 EMU) must equal 720.0 PDF points (10 inches)"
        );
    }

    /// BC-4.03.005 AC-001 / postcondition 2: slide height (5.625 inches) converts to 405.0pt.
    ///
    /// FAILS at Red Gate (todo!() panic). PASSES after implementation.
    #[test]
    fn test_bc_4_03_005_emu_to_pt_slide_height_405() {
        assert_eq!(
            emu_to_pt(SLIDE_HEIGHT_EMU),
            405.0_f32,
            "emu_to_pt(5_143_500 EMU) must equal 405.0 PDF points (5.625 inches)"
        );
    }

    /// BC-4.03.005 AC-001: 1 PDF point = 12,700 EMU (canonical unit).
    ///
    /// FAILS at Red Gate (todo!() panic). PASSES after implementation.
    #[test]
    fn test_bc_4_03_005_emu_to_pt_one_point() {
        assert_eq!(
            emu_to_pt(Emu(12_700)),
            1.0_f32,
            "emu_to_pt(12_700 EMU) must equal 1.0 PDF point"
        );
    }

    /// BC-4.03.005 AC-001 / EC-001: origin maps to 0.0pt.
    ///
    /// FAILS at Red Gate (todo!() panic). PASSES after implementation.
    #[test]
    fn test_bc_4_03_005_emu_to_pt_zero_is_zero() {
        assert_eq!(
            emu_to_pt(Emu(0)),
            0.0_f32,
            "emu_to_pt(0) must equal 0.0"
        );
    }

    // ── AC-002: ir_y_to_pdf_y canonical test vectors ──────────────────────────

    /// BC-4.03.005 AC-002 / postcondition 3 / EC-002: top-left element of 100pt height
    /// maps to pdf_y = 405 − 0 − 100 = 305.0pt.
    ///
    /// FAILS at Red Gate (todo!() panic). PASSES after implementation.
    #[test]
    fn test_bc_4_03_005_ir_y_to_pdf_y_top_left_element_305() {
        let result = ir_y_to_pdf_y(Emu(0), Emu(100 * 12_700), SLIDE_HEIGHT_EMU);
        assert!(
            (result - 305.0_f32).abs() < 0.001,
            "ir_y_to_pdf_y(0, 100pt, slide_h) must equal 305.0; got {result}"
        );
    }

    /// BC-4.03.005 AC-002 / postcondition 4 / EC-003: bottom-edge element maps to pdf_y = 0.0.
    ///
    /// Element at ir_y = slide_height − element_height fills the bottom of the slide.
    /// FAILS at Red Gate (todo!() panic). PASSES after implementation.
    #[test]
    fn test_bc_4_03_005_ir_y_to_pdf_y_bottom_edge_zero() {
        let element_h = Emu(100 * 12_700);
        let ir_y = Emu(SLIDE_HEIGHT_EMU.0 - element_h.0);
        let result = ir_y_to_pdf_y(ir_y, element_h, SLIDE_HEIGHT_EMU);
        assert!(
            result.abs() < 0.001,
            "ir_y_to_pdf_y(slide_h - 100pt, 100pt, slide_h) must equal 0.0; got {result}"
        );
    }

    /// BC-4.03.005 AC-002 / AC-008 / EC-004: zero-height element at ir_y=0 gives 405.0pt.
    ///
    /// `ir_y_to_pdf_y(Emu(0), Emu(0), SLIDE_H) == SLIDE_HEIGHT_PT - 0 - 0 == 405.0`
    /// FAILS at Red Gate (todo!() panic). PASSES after implementation.
    #[test]
    fn test_bc_4_03_005_ir_y_to_pdf_y_zero_height_at_origin_is_slide_height() {
        let result = ir_y_to_pdf_y(Emu(0), Emu(0), SLIDE_HEIGHT_EMU);
        assert!(
            (result - SLIDE_HEIGHT_PT).abs() < 0.001,
            "ir_y_to_pdf_y(0, 0, slide_h) must equal SLIDE_HEIGHT_PT ({SLIDE_HEIGHT_PT}); got {result}"
        );
    }

    // ── AC-003: Rounding error < 0.001 for 0..=9_144_000 EMU ─────────────────

    /// BC-4.03.005 AC-003 / postcondition 5: `emu_to_pt` rounding error < 0.001pt
    /// for all standard slide dimension values.
    ///
    /// Samples the range 0..=9_144_000 at 1,000-EMU intervals (~9,145 samples)
    /// and compares `f32` output against `f64` reference for each.
    ///
    /// FAILS at Red Gate (todo!() panic on first call). PASSES after implementation.
    #[test]
    fn test_bc_4_03_005_emu_to_pt_rounding_error_below_0_001() {
        let mut max_err = 0.0_f64;
        let mut worst_emu = 0_i64;

        // Step by 1000 EMU to keep runtime reasonable while covering the full
        // standard slide width (9,144,000 EMU) with ~9,145 samples.
        let mut emu_val = 0_i64;
        loop {
            if emu_val > 9_144_000 {
                break;
            }
            // This call panics with todo!() before implementation — test FAILS.
            let f32_result = emu_to_pt(Emu(emu_val)) as f64;
            let f64_reference = emu_val as f64 / 12_700.0_f64;
            let err = (f32_result - f64_reference).abs();
            if err > max_err {
                max_err = err;
                worst_emu = emu_val;
            }
            emu_val += 1000;
        }

        assert!(
            max_err < 0.001,
            "emu_to_pt rounding error exceeds 0.001pt: max_err={max_err:.6} at EMU={worst_emu}"
        );
    }

    // ── AC-007: 4:3 slide size coordinate mapping ─────────────────────────────

    /// BC-4.03.005 AC-007 / EC-005: 4:3 slide (7,200,000 × 5,400,000 EMU).
    ///
    /// `ir_y_to_pdf_y` must use the supplied `slide_h`, not the hard-coded 16:9 constant.
    /// FAILS at Red Gate (todo!() panic). PASSES after implementation.
    #[test]
    fn test_bc_4_03_005_ir_y_to_pdf_y_4x3_slide_mapping() {
        // 4:3 slide dimensions: 7_200_000 × 5_400_000 EMU (common presentation 4:3)
        // emu_to_pt(7_200_000) ≈ 566.929pt
        // emu_to_pt(5_400_000) ≈ 425.197pt
        let slide_w_4x3 = Emu(7_200_000);
        let slide_h_4x3 = Emu(5_400_000);

        let expected_slide_w_pt = slide_w_4x3.0 as f32 / 12_700.0_f32;
        let expected_slide_h_pt = slide_h_4x3.0 as f32 / 12_700.0_f32;

        // Verify width conversion — panics before implementation.
        let actual_w = emu_to_pt(slide_w_4x3);
        assert!(
            (actual_w - expected_slide_w_pt).abs() < 0.001,
            "4:3 slide width: expected {expected_slide_w_pt:.3}pt, got {actual_w:.3}pt"
        );

        let actual_h = emu_to_pt(slide_h_4x3);
        assert!(
            (actual_h - expected_slide_h_pt).abs() < 0.001,
            "4:3 slide height: expected {expected_slide_h_pt:.3}pt, got {actual_h:.3}pt"
        );

        // Element at top-left with height 100pt in 4:3 slide.
        let element_h = Emu(100 * 12_700);
        let expected_pdf_y = expected_slide_h_pt - 100.0_f32;
        let actual_pdf_y = ir_y_to_pdf_y(Emu(0), element_h, slide_h_4x3);

        assert!(
            (actual_pdf_y - expected_pdf_y).abs() < 0.001,
            "4:3 slide: ir_y_to_pdf_y(0, 100pt, slide_h_4x3) must be {expected_pdf_y:.3}pt; \
             got {actual_pdf_y:.3}pt. The function MUST use the supplied slide_h, not \
             the hardcoded SLIDE_HEIGHT_EMU constant."
        );
    }

    // ── AC-006: Integration — no element outside canvas after conversion ───────

    /// BC-4.03.005 AC-006 / invariant 3: For a fixture deck, all element bounding
    /// boxes converted via `ir_y_to_pdf_y` lie within [0, SLIDE_HEIGHT_PT].
    ///
    /// Uses a hand-crafted set of representative `(ir_y, element_h)` pairs.
    /// FAILS at Red Gate (todo!() panic). PASSES after implementation.
    #[test]
    fn test_bc_4_03_005_no_element_outside_canvas_after_conversion() {
        // Representative (ir_y_emu, element_h_emu) pairs.
        // All satisfy: ir_y + element_h <= SLIDE_HEIGHT_EMU (5_143_500).
        let test_cases: &[(i64, i64)] = &[
            (0, 72 * 12_700),
            (72 * 12_700, 333 * 12_700),
            (SLIDE_HEIGHT_EMU.0 - 12_700, 12_700),
            (0, SLIDE_HEIGHT_EMU.0),
            (0, 0),
            (SLIDE_HEIGHT_EMU.0 / 2, 0),
            (SLIDE_HEIGHT_EMU.0, 0),
        ];

        for &(ir_y_val, elem_h_val) in test_cases {
            // Panics with todo!() before implementation.
            let pdf_y = ir_y_to_pdf_y(Emu(ir_y_val), Emu(elem_h_val), SLIDE_HEIGHT_EMU);
            assert!(
                pdf_y >= -0.001,
                "pdf_y must be >= 0.0 for ir_y={ir_y_val}, elem_h={elem_h_val}: got {pdf_y}"
            );
            assert!(
                pdf_y <= SLIDE_HEIGHT_PT + 0.001,
                "pdf_y must be <= {SLIDE_HEIGHT_PT} for ir_y={ir_y_val}, elem_h={elem_h_val}: \
                 got {pdf_y}"
            );
        }
    }

    // ── AC-004 / AC-005: Pure-function structural check ───────────────────────

    /// BC-4.03.005 AC-004 / AC-005: `emu_to_pt` must be a pure function.
    ///
    /// Repeated calls with identical inputs must return identical results.
    /// FAILS at Red Gate (todo!() panic). PASSES after implementation.
    #[test]
    fn test_bc_4_03_005_emu_to_pt_is_deterministic() {
        // Both calls panic with todo!() before implementation.
        let a = emu_to_pt(Emu(914_400));
        let b = emu_to_pt(Emu(914_400));
        assert_eq!(a, b, "emu_to_pt must be deterministic (pure function)");
        assert!(
            (a - 72.0_f32).abs() < 0.001,
            "emu_to_pt(914_400) must equal 72.0pt (1 inch); got {a}"
        );
    }

    /// BC-4.03.005 AC-004 / AC-005: `ir_y_to_pdf_y` must be a pure function.
    ///
    /// FAILS at Red Gate (todo!() panic). PASSES after implementation.
    #[test]
    fn test_bc_4_03_005_ir_y_to_pdf_y_is_deterministic() {
        let a = ir_y_to_pdf_y(Emu(12_700), Emu(25_400), SLIDE_HEIGHT_EMU);
        let b = ir_y_to_pdf_y(Emu(12_700), Emu(25_400), SLIDE_HEIGHT_EMU);
        assert_eq!(a, b, "ir_y_to_pdf_y must be deterministic (pure function)");
        // Expected: 405.0 - 1.0 - 2.0 = 402.0
        assert!(
            (a - 402.0_f32).abs() < 0.001,
            "ir_y_to_pdf_y(1pt, 2pt, slide_h) must equal 402.0pt; got {a}"
        );
    }
}
