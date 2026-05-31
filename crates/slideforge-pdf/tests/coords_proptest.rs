//! Property-based tests for `slideforge_pdf::coords`.
//!
//! ## Coverage
//!
//! - AC-003 (VP-006): For all valid `(ir_y, element_h, slide_h)` tuples
//!   satisfying `ir_y + element_h <= slide_h`, `ir_y_to_pdf_y()` returns a
//!   non-negative value.
//!
//! ## Red Gate status
//!
//! All tests in this file MUST FAIL before implementation because
//! `emu_to_pt` and `ir_y_to_pdf_y` have `todo!()` bodies.
//! After implementation (TDD green), these tests MUST PASS.
//!
//! ## proptest version
//!
//! Uses `proptest =1.5.0` (pinned in `[dev-dependencies]`).

use proptest::prelude::*;
use slideforge_pdf::coords::{emu_to_pt, ir_y_to_pdf_y};
use slideforge_types::Emu;

// ─── Helper ───────────────────────────────────────────────────────────────────

/// Upper bound for EMU values in the proptest strategy.
///
/// Capped at 100 inches (91,440,000 EMU) — well beyond any realistic slide
/// dimension but bounded to prevent integer overflow in test arithmetic.
const MAX_TEST_EMU: i64 = 91_440_000;

// ─── Property: ir_y_to_pdf_y >= 0 for valid inputs ───────────────────────────

proptest! {
    /// BC-4.03.005 AC-003 / invariant 3 / VP-006: For all valid `(ir_y, element_h,
    /// slide_h)` satisfying `ir_y + element_h <= slide_h`, `ir_y_to_pdf_y` returns
    /// a value >= 0.0.
    ///
    /// This property encodes the geometric invariant that after the Y-axis flip,
    /// no element whose top-left corner is within the slide canvas and whose
    /// height fits within the remaining canvas can have a negative PDF Y
    /// coordinate.
    ///
    /// Proof sketch:
    ///   pdf_y = slide_h_pt - ir_y_pt - elem_h_pt
    ///         = (slide_h - ir_y - elem_h)_pt / 12700
    ///   Since ir_y + elem_h <= slide_h (precondition), slide_h - ir_y - elem_h >= 0,
    ///   therefore pdf_y >= 0.0.
    ///
    /// ## Red Gate
    ///
    /// MUST FAIL at Red Gate because `ir_y_to_pdf_y` panics with `todo!()`.
    /// After implementation this property must pass across ≥1000 random cases.
    #[test]
    fn test_bc_4_03_005_proptest_ir_y_to_pdf_y_nonneg_for_valid_inputs(
        // slide_h: the full slide height in EMU — at least 1pt (12_700 EMU), at most 100 inches.
        slide_h_emu in 12_700_i64..=MAX_TEST_EMU,
        // ir_y: the element's top edge, 0 ≤ ir_y ≤ slide_h.
        ir_y_fraction in 0.0_f64..=1.0_f64,
        // element_h fills the remaining space from ir_y to slide_h.
        elem_h_fraction in 0.0_f64..=1.0_f64,
    ) {
        // Derive ir_y and element_h so that ir_y + element_h <= slide_h.
        let ir_y_emu = (ir_y_fraction * slide_h_emu as f64) as i64;
        let remaining = slide_h_emu - ir_y_emu;
        let elem_h_emu = (elem_h_fraction * remaining as f64) as i64;

        // Precondition: ir_y + elem_h <= slide_h (guaranteed by construction).
        prop_assume!(ir_y_emu + elem_h_emu <= slide_h_emu);

        let pdf_y = ir_y_to_pdf_y(Emu(ir_y_emu), Emu(elem_h_emu), Emu(slide_h_emu));

        prop_assert!(
            pdf_y >= -0.001_f32,
            "ir_y_to_pdf_y({ir_y_emu}, {elem_h_emu}, {slide_h_emu}) returned {pdf_y} < 0.0; \
             expected non-negative result for valid canvas position (BC-4.03.005 invariant 3)"
        );

        // Also verify the upper bound: pdf_y <= slide_h_pt.
        let slide_h_pt = emu_to_pt(Emu(slide_h_emu));
        prop_assert!(
            pdf_y <= slide_h_pt + 0.001_f32,
            "ir_y_to_pdf_y({ir_y_emu}, {elem_h_emu}, {slide_h_emu}) returned {pdf_y} > \
             slide_h_pt={slide_h_pt}; element is outside the canvas"
        );
    }
}

proptest! {
    /// BC-4.03.005 AC-001 / AC-003: `emu_to_pt` is monotonically non-decreasing.
    ///
    /// For all a, b ≥ 0 with a ≤ b: `emu_to_pt(a) ≤ emu_to_pt(b)`.
    ///
    /// This is a weaker but useful property: the conversion must not invert order.
    ///
    /// ## Red Gate
    ///
    /// MUST FAIL because `emu_to_pt` panics with `todo!()`.
    #[test]
    fn test_bc_4_03_005_proptest_emu_to_pt_monotone(
        a_emu in 0_i64..=MAX_TEST_EMU,
        b_emu in 0_i64..=MAX_TEST_EMU,
    ) {
        let (lo, hi) = if a_emu <= b_emu {
            (a_emu, b_emu)
        } else {
            (b_emu, a_emu)
        };

        let lo_pt = emu_to_pt(Emu(lo));
        let hi_pt = emu_to_pt(Emu(hi));

        prop_assert!(
            lo_pt <= hi_pt + 0.001_f32,
            "emu_to_pt must be monotonically non-decreasing: \
             emu_to_pt({lo}) = {lo_pt} > emu_to_pt({hi}) = {hi_pt}"
        );
    }
}
