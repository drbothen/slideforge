// S2 Spike — Coordinate Mapping Sample
//
// Demonstrates EMU → PDF user unit (points) coordinate mapping and Y-axis flip.
// Throwaway code — for illustrative and proof-of-concept purposes only.
// Production implementation lives in crates/slideforge-pdf/src/coords.rs (Phase 4).

/// Standard OOXML measurement unit: 1 inch = 914400 EMU
pub const EMU_PER_INCH: i64 = 914_400;

/// PDF user unit: 1 point = 1/72 inch = 12700 EMU
pub const EMU_PER_POINT: i64 = 12_700;

/// Standard 16:9 slide dimensions in EMU
/// Width:  9144000 EMU = 10.0 in = 720.0 pt
/// Height: 5143500 EMU ≈ 5.625 in ≈ 405.0 pt
pub const SLIDE_WIDTH_EMU: i64 = 9_144_000;
pub const SLIDE_HEIGHT_EMU: i64 = 5_143_500;

pub const SLIDE_WIDTH_PT: f32 = 720.0;
pub const SLIDE_HEIGHT_PT: f32 = 405.0;

/// New-type wrapping an EMU value to prevent accidental mixing of units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Emu(pub i64);

/// A point value in PDF user space (1/72 inch).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Pt(pub f32);

/// Axis-aligned rectangle in PDF user space (bottom-left origin, Y up).
#[derive(Debug, Clone, Copy)]
pub struct PdfRect {
    /// Left edge (X from left)
    pub x: Pt,
    /// Bottom edge (Y from bottom of page)
    pub y: Pt,
    pub width: Pt,
    pub height: Pt,
}

/// Convert EMU value to PDF user units (points, 1/72 inch).
///
/// Exact conversion: EMU / 12700 = points
/// This is lossless for all standard OOXML slide/shape coordinates.
pub fn emu_to_pt(emu: Emu) -> Pt {
    Pt(emu.0 as f32 / EMU_PER_POINT as f32)
}

/// Convert an IR element bounding box (top-left origin, Y down) to a PDF rectangle
/// (bottom-left origin, Y up).
///
/// IR layout uses (0,0) at top-left corner, Y increases downward.
/// PDF uses (0,0) at bottom-left corner, Y increases upward.
///
/// # Arguments
/// * `ir_x` — left edge of element in EMU (IR space)
/// * `ir_y` — top edge of element in EMU (IR space)
/// * `width` — element width in EMU
/// * `height` — element height in EMU
/// * `page_height_pt` — page height in points (for Y flip)
pub fn ir_rect_to_pdf(
    ir_x: Emu,
    ir_y: Emu,
    width: Emu,
    height: Emu,
    page_height_pt: Pt,
) -> PdfRect {
    let x = emu_to_pt(ir_x);
    let w = emu_to_pt(width);
    let h = emu_to_pt(height);
    // PDF Y: page_height - ir_top - element_height
    let y = Pt(page_height_pt.0 - emu_to_pt(ir_y).0 - h.0);
    PdfRect { x, y, width: w, height: h }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slide_width_converts_to_720pt() {
        let pt = emu_to_pt(Emu(SLIDE_WIDTH_EMU));
        assert!((pt.0 - 720.0).abs() < 0.001, "expected 720.0pt, got {}", pt.0);
    }

    #[test]
    fn slide_height_converts_to_405pt() {
        let pt = emu_to_pt(Emu(SLIDE_HEIGHT_EMU));
        assert!((pt.0 - 405.0).abs() < 0.001, "expected 405.0pt, got {}", pt.0);
    }

    #[test]
    fn element_at_ir_origin_has_pdf_y_at_page_top_minus_height() {
        // IR element at (0, 0), 100pt tall, on 405pt-high page
        // PDF y should be 405 - 0 - 100 = 305
        let rect = ir_rect_to_pdf(
            Emu(0),
            Emu(0),
            Emu(100 * EMU_PER_POINT),
            Emu(100 * EMU_PER_POINT),
            Pt(SLIDE_HEIGHT_PT),
        );
        assert!((rect.y.0 - 305.0).abs() < 0.001, "expected y=305.0, got {}", rect.y.0);
        assert!((rect.x.0 - 0.0).abs() < 0.001, "expected x=0.0, got {}", rect.x.0);
    }

    #[test]
    fn element_at_ir_bottom_has_pdf_y_zero() {
        // IR element touching the bottom: ir_y = SLIDE_HEIGHT - element_height
        let elem_h_emu = 50 * EMU_PER_POINT;
        let ir_y = Emu(SLIDE_HEIGHT_EMU - elem_h_emu);
        let rect = ir_rect_to_pdf(
            Emu(0),
            ir_y,
            Emu(100 * EMU_PER_POINT),
            Emu(elem_h_emu),
            Pt(SLIDE_HEIGHT_PT),
        );
        assert!(
            rect.y.0.abs() < 0.01,
            "bottom element should have PDF y ≈ 0, got {}",
            rect.y.0
        );
    }

    #[test]
    fn one_inch_is_72_points() {
        let pt = emu_to_pt(Emu(EMU_PER_INCH));
        assert!((pt.0 - 72.0).abs() < 0.001, "1 inch should be 72 points, got {}", pt.0);
    }
}
