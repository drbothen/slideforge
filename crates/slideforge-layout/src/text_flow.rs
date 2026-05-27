//! Text-flow computation for layout frames.
//!
//! `TextFlow` provides a heuristic model of how text fits within a
//! [`BoundingBox`]. It is NOT a pixel-perfect reflow — it uses a fixed
//! estimated character width to detect overflow conditions for the canvas
//! overflow warning (BC-3.03.001).
//!
//! ## Overflow model
//!
//! - Character width estimate: `Emu(76_200)` = 1 inch / 12 chars at 12pt
//!   (72 points/inch × 12pt = 1 inch → 1/12 chars per inch).
//! - Line length: `floor(bbox.width / char_width_emu)` characters per line.
//! - Line count: `ceil(text.chars().count() / chars_per_line)`.
//! - Frame height used: `line_count × line_height_emu` where
//!   `line_height_emu = Emu(152_400)` (12pt × 1.5 leading = 18pt =
//!   18 × 12,700 EMU).
//! - Overflow: `used_height > bbox.height`.
//!
//! Empty text (`""`) always produces `TextOverflow::Fit` with `line_count = 0`
//! (EC-004).

use crate::types::{BoundingBox, Emu, TextFlow, TextOverflow};

/// Estimated character width at 12pt body font size (1 inch / 12 chars).
///
/// `76_200 EMU = 914_400 / 12`. Used by [`compute_text_flow`] as the default
/// per-character width when no font metrics are available.
pub const ESTIMATED_CHAR_WIDTH_EMU: Emu = Emu(76_200);

/// Line height at 12pt with 1.5 leading (18pt = 18 × 12,700 EMU = 228,600 EMU).
const LINE_HEIGHT_EMU: Emu = Emu(228_600);

/// Compute a heuristic [`TextFlow`] for `text` within the given `bounding_box`.
///
/// # Arguments
///
/// * `text` — The text content to measure. May be empty.
/// * `bounding_box` — The frame's position and dimensions.
///
/// # Returns
///
/// A [`TextFlow`] with overflow analysis. Empty text always returns
/// `TextOverflow::Fit` with `line_count = 0` (EC-004).
///
/// # Panics
///
/// Does not panic. Zero-width or zero-height frames are handled gracefully:
/// non-empty text in a zero-width frame produces an overflow result.
#[must_use]
pub fn compute_text_flow(text: &str, bounding_box: BoundingBox) -> TextFlow {
    compute_text_flow_with_char_width(text, bounding_box, ESTIMATED_CHAR_WIDTH_EMU)
}

/// Compute text flow with a custom per-character width (for testing).
///
/// Same as [`compute_text_flow`] but accepts an explicit `char_width_emu`
/// instead of the default `ESTIMATED_CHAR_WIDTH_EMU`.
#[must_use]
pub fn compute_text_flow_with_char_width(
    text: &str,
    bounding_box: BoundingBox,
    char_width_emu: Emu,
) -> TextFlow {
    // EC-004: empty text always fits.
    if text.is_empty() {
        return TextFlow {
            bounding_box,
            overflow: TextOverflow::Fit,
            line_count: 0,
            estimated_char_width_emu: char_width_emu,
        };
    }

    let char_count = text.chars().count();

    // How many characters fit on one line?
    // Guard against zero char_width to avoid division by zero.
    let chars_per_line: usize = if char_width_emu.0 <= 0 {
        1
    } else {
        let cpp = bounding_box.width.0 / char_width_emu.0;
        // cpp >= 0 because both operands are non-negative at this point;
        // usize::try_from is safe here and handles 32-bit targets correctly.
        usize::try_from(cpp).unwrap_or(1).max(1)
    };

    // Ceiling division: ceil(char_count / chars_per_line)
    let line_count = char_count.div_ceil(chars_per_line);

    // Total height used by text.
    // i64::try_from(usize) is infallible on 64-bit targets and saturates to
    // i64::MAX on hypothetical ≥128-bit targets; unwrap_or keeps correctness.
    let line_count_i64 = i64::try_from(line_count).unwrap_or(i64::MAX);
    let used_height = Emu(LINE_HEIGHT_EMU.0.saturating_mul(line_count_i64));

    let overflow = if used_height <= bounding_box.height {
        TextOverflow::Fit
    } else {
        let excess = used_height - bounding_box.height;
        // excess_emu must be non-negative per AC-007
        let excess_emu = if excess.0 < 0 { Emu(0) } else { excess };
        TextOverflow::Overflow { excess_emu }
    };

    TextFlow {
        bounding_box,
        overflow,
        line_count: u32::try_from(line_count).unwrap_or(u32::MAX),
        estimated_char_width_emu: char_width_emu,
    }
}

#[cfg(test)]
#[allow(clippy::missing_docs_in_private_items)]
mod tests {
    use super::*;
    use crate::types::DEFAULT_PAGE_WIDTH;

    fn make_bbox(width_emu: i64, height_emu: i64) -> BoundingBox {
        BoundingBox {
            x: Emu(0),
            y: Emu(0),
            width: Emu(width_emu),
            height: Emu(height_emu),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-007 / EC-004 — TextFlow computation
    // ─────────────────────────────────────────────────────────────────────────

    /// EC-004 — Empty text always produces TextOverflow::Fit with line_count 0.
    #[test]
    fn test_bc_3_06_001_text_flow_empty_text_fits() {
        let bbox = make_bbox(8_229_600, 3_657_600);
        let tf = compute_text_flow("", bbox);
        assert!(
            matches!(tf.overflow, TextOverflow::Fit),
            "empty text must produce TextOverflow::Fit"
        );
        assert_eq!(tf.line_count, 0, "empty text must have line_count = 0");
    }

    /// AC-007 — Short text within bounds produces Fit.
    #[test]
    fn test_bc_3_06_001_text_flow_short_text_fits() {
        // 50 chars of text in a tall, wide frame — easily fits.
        let text = "Hello, world! This is a short slide title.";
        let bbox = make_bbox(8_229_600, 3_657_600);
        let tf = compute_text_flow(text, bbox);
        assert!(
            matches!(tf.overflow, TextOverflow::Fit),
            "short text must fit in a large frame; got {:?}",
            tf.overflow
        );
        assert!(tf.line_count >= 1, "non-empty text must have line_count >= 1");
    }

    /// AC-007 — Long text exceeding the frame produces TextOverflow::Overflow
    /// with non-negative excess_emu.
    #[test]
    fn test_bc_3_06_001_text_flow_overflow_detected() {
        // Very narrow, very short frame — 2000 chars of text will overflow.
        let long_text: String = "a".repeat(2000);
        let narrow_bbox = make_bbox(
            457_200,  // 0.5in wide
            228_600,  // just 1 line tall
        );
        let tf = compute_text_flow(&long_text, narrow_bbox);
        match &tf.overflow {
            TextOverflow::Overflow { excess_emu } => {
                assert!(
                    excess_emu.0 >= 0,
                    "excess_emu must be non-negative; got {excess_emu:?}"
                );
            },
            other => panic!("expected TextOverflow::Overflow, got {other:?}"),
        }
        assert!(tf.line_count > 1, "long text must produce multiple lines");
    }

    /// AC-007 — excess_emu is always non-negative.
    #[test]
    fn test_bc_3_06_001_text_flow_excess_emu_non_negative() {
        let text = "x".repeat(500);
        // Tiny frame — definitely overflows
        let bbox = make_bbox(100_000, 100_000);
        let tf = compute_text_flow(&text, bbox);
        if let TextOverflow::Overflow { excess_emu } = &tf.overflow {
            assert!(
                excess_emu.0 >= 0,
                "excess_emu must be non-negative per AC-007"
            );
        }
        // If it somehow fits — that's fine too (just verify line_count > 0)
        assert!(tf.line_count > 0);
    }

    /// ESTIMATED_CHAR_WIDTH_EMU constant is correct (76_200 EMU).
    #[test]
    fn test_bc_3_06_001_estimated_char_width_constant() {
        assert_eq!(ESTIMATED_CHAR_WIDTH_EMU, Emu(76_200));
        // Verify the derivation: 914_400 / 12 = 76_200
        assert_eq!(Emu(914_400).0 / 12, ESTIMATED_CHAR_WIDTH_EMU.0);
    }

    /// A single word fits in the full-width frame.
    #[test]
    fn test_bc_3_06_001_text_flow_single_word_fits() {
        let bbox = make_bbox(DEFAULT_PAGE_WIDTH.0, 685_800);
        let tf = compute_text_flow("Hello", bbox);
        assert!(matches!(tf.overflow, TextOverflow::Fit));
        assert_eq!(tf.line_count, 1);
    }

    /// TextFlow with custom char width behaves correctly.
    #[test]
    fn test_bc_3_06_001_text_flow_custom_char_width() {
        let text = "a".repeat(10);
        let bbox = make_bbox(1_000_000, 5_000_000);
        // char_width = 100_000 EMU → chars_per_line = 1_000_000 / 100_000 = 10
        // 10 chars / 10 per line = 1 line
        let tf = compute_text_flow_with_char_width(&text, bbox, Emu(100_000));
        assert_eq!(tf.line_count, 1);
        assert!(matches!(tf.overflow, TextOverflow::Fit));
    }
}
