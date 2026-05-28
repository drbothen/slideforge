//! Color slot inference for synthesized brands (BC-2.01.004).
//!
//! When `brand.toml` omits one or more of the 12 OOXML color slots, this
//! module derives the missing values using a deterministic algorithm (AC-003).
//! For each inferred slot, exactly one `tracing::warn!` is emitted with error
//! code `E-BRD-003`.
//!
//! ## Inference rules (AC-003, BC-2.01.004 postcondition 3)
//!
//! | Slot | If absent, derive from... |
//! |------|--------------------------|
//! | `dk1` | Darkest declared color by luminance; fallback `"#1F2937"` |
//! | `lt1` | Always `"#FFFFFF"` (hardcoded) |
//! | `dk2` | `acc1` if declared; else `dk1` lightened 20% |
//! | `lt2` | Always `"#F9FAFB"` (near-white; 96% lightness) |
//! | `acc2`–`acc6` | `acc1` with 30°/60°/90°/120°/150° hue rotation |
//! | `hlink` | `acc1` darkened 15% |
//! | `fol_hlink` | `hlink` darkened 10% |
//!
//! ## Invariants (BC-2.01.004 invariant 3)
//!
//! All returned hex strings are in `"#RRGGBB"` format with uppercase hex
//! digits. No alpha channel, no CSS named colors, no lowercase hex.

use std::sync::Arc;

use crate::error::BrandError;

// ─── Public entry point ───────────────────────────────────────────────────────

/// Infer all 12 OOXML color slots from a partially-declared palette.
///
/// `declared` is a 12-element array of `Option<&str>` in ECMA-376 slot order:
/// `[dk1, lt1, dk2, lt2, acc1, acc2, acc3, acc4, acc5, acc6, hlink, fol_hlink]`.
///
/// Each `None` element is derived from other declared values using the
/// deterministic rules from AC-003. For each inferred slot, one `E-BRD-003`
/// warning is pushed to `warnings`.
///
/// # Returns
///
/// A fully-populated `[Arc<str>; 12]` array of `"#RRGGBB"` hex strings.
/// The array is always complete regardless of how many slots were declared.
///
/// # Panics
///
/// Does not panic. All inference falls back to hardcoded defaults.
pub fn infer_missing_slots(
    _declared: [Option<&str>; 12],
    _warnings: &mut Vec<BrandError>,
) -> [Arc<str>; 12] {
    todo!()
}

// ─── Color manipulation helpers ───────────────────────────────────────────────

/// Convert a `"#RRGGBB"` hex string to HSL components `(h°, s, l)`.
///
/// `h` is in `[0.0, 360.0)`, `s` and `l` are in `[0.0, 1.0]`.
///
/// # Panics
///
/// Panics in debug builds if `hex` is not a valid `"#RRGGBB"` string.
#[allow(dead_code)]
fn hex_to_hsl(_hex: &str) -> (f32, f32, f32) {
    todo!()
}

/// Convert HSL components `(h°, s, l)` to an uppercase `"#RRGGBB"` hex string.
///
/// `h` is in `[0.0, 360.0)`, `s` and `l` are in `[0.0, 1.0]`.
/// The returned string is always 7 characters and uses uppercase hex digits.
#[allow(dead_code)]
fn hsl_to_hex(_h: f32, _s: f32, _l: f32) -> String {
    todo!()
}

/// Darken a `"#RRGGBB"` hex color by `pct` of its current lightness.
///
/// `pct` is in `[0.0, 1.0]` (e.g., `0.15` = darken by 15%).
/// Returns an uppercase `"#RRGGBB"` string.
#[allow(dead_code)]
pub(crate) fn darken_hex(_hex: &str, _pct: f32) -> String {
    todo!()
}

/// Lighten a `"#RRGGBB"` hex color by `pct` towards white.
///
/// `pct` is in `[0.0, 1.0]` (e.g., `0.20` = blend 20% towards white).
/// Returns an uppercase `"#RRGGBB"` string.
#[allow(dead_code)]
pub(crate) fn lighten_hex(_hex: &str, _pct: f32) -> String {
    todo!()
}

/// Rotate the hue of a `"#RRGGBB"` hex color by `degrees`.
///
/// `degrees` can be any value; it is reduced modulo 360 before application.
/// Saturation and lightness are preserved.
/// Returns an uppercase `"#RRGGBB"` string.
#[allow(dead_code)]
pub(crate) fn rotate_hue(_hex: &str, _degrees: f32) -> String {
    todo!()
}

/// Find the darkest color (by luminance) among declared slots.
///
/// Returns `"#1F2937"` (the fallback dark) if `slots` contains no `Some` values.
#[allow(dead_code)]
fn darkest_declared(_slots: &[Option<&str>]) -> &'static str {
    todo!()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// BC-2.01.004 — when all 12 slots declared, zero E-BRD-003 warnings emitted.
    #[test]
    fn test_bc_2_01_004_all_12_declared_no_inference() {
        let declared: [Option<&str>; 12] = [
            Some("#1F2937"),
            Some("#FFFFFF"),
            Some("#374151"),
            Some("#F9FAFB"),
            Some("#3B82F6"),
            Some("#10B981"),
            Some("#F59E0B"),
            Some("#EF4444"),
            Some("#8B5CF6"),
            Some("#EC4899"),
            Some("#2563EB"),
            Some("#1D4ED8"),
        ];
        let mut warnings = Vec::new();
        let _ = infer_missing_slots(declared, &mut warnings); // todo!() body — test will fail
        assert_eq!(warnings.len(), 0, "no warnings when all 12 declared");
    }

    /// BC-2.01.004 / AC-003 — `lt1` is always `"#FFFFFF"` regardless of input.
    #[test]
    fn test_bc_2_01_004_lt1_always_ffffff() {
        let declared: [Option<&str>; 12] = [
            Some("#000000"),
            None, // lt1 absent → must become "#FFFFFF"
            None,
            None,
            Some("#3B82F6"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ];
        let mut warnings = Vec::new();
        let result = infer_missing_slots(declared, &mut warnings);
        assert_eq!(result[1].as_ref(), "#FFFFFF", "lt1 must always be #FFFFFF");
    }

    /// BC-2.01.004 / AC-003 — 12 E-BRD-003 warnings when all slots absent.
    #[test]
    fn test_bc_2_01_004_empty_colors_produces_12_warnings() {
        let declared: [Option<&str>; 12] = [None; 12];
        let mut warnings = Vec::new();
        let _ = infer_missing_slots(declared, &mut warnings);
        assert_eq!(warnings.len(), 12, "12 E-BRD-003 warnings for 12 absent slots");
    }

    /// BC-2.01.004 / AC-003 — `dk2` inferred as `acc1` when `acc1` declared.
    #[test]
    fn test_bc_2_01_004_dk2_from_acc1_when_available() {
        let declared: [Option<&str>; 12] = [
            Some("#1F2937"), // dk1
            Some("#FFFFFF"), // lt1
            None,            // dk2 absent → infer from acc1
            None,
            Some("#3B82F6"), // acc1 declared
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ];
        let mut warnings = Vec::new();
        let result = infer_missing_slots(declared, &mut warnings);
        // dk2 should be acc1 value when acc1 is declared
        assert_eq!(result[2].as_ref(), "#3B82F6", "dk2 should equal acc1 when acc1 declared");
    }

    /// BC-2.01.004 / AC-003 — `acc2` has hue rotated 30° from `acc1`.
    #[test]
    fn test_bc_2_01_004_acc2_hue_rotation_30_degrees() {
        // Only acc1 declared; acc2..acc6 all absent.
        let declared: [Option<&str>; 12] = [
            Some("#1F2937"),
            Some("#FFFFFF"),
            None,
            None,
            Some("#3B82F6"), // acc1
            None,            // acc2 absent → rotate 30°
            None,
            None,
            None,
            None,
            None,
            None,
        ];
        let mut warnings = Vec::new();
        let result = infer_missing_slots(declared, &mut warnings);
        // acc2 must differ from acc1 and be a valid uppercase hex color
        let acc2 = result[5].as_ref();
        assert_ne!(acc2, "#3B82F6", "acc2 must differ from acc1 after 30° rotation");
        assert!(acc2.starts_with('#'), "acc2 must start with #");
        assert_eq!(acc2.len(), 7, "acc2 must be 7-char #RRGGBB");
        assert!(
            acc2[1..].chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()),
            "acc2 must be uppercase hex: {acc2}"
        );
    }

    /// BC-2.01.004 invariant 3 — all inferred colors are uppercase `"#RRGGBB"`.
    #[test]
    fn test_bc_2_01_004_invariant_inferred_colors_are_uppercase_hex() {
        let declared: [Option<&str>; 12] = [None; 12];
        let mut warnings = Vec::new();
        let result = infer_missing_slots(declared, &mut warnings);
        for (i, hex) in result.iter().enumerate() {
            let h = hex.as_ref();
            assert!(
                h.starts_with('#') && h.len() == 7,
                "slot {i}: expected #RRGGBB, got '{h}'"
            );
            assert!(
                h[1..].chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()),
                "slot {i}: hex must be uppercase, got '{h}'"
            );
        }
    }

    /// BC-2.01.004 — `hlink` inferred as `acc1` darkened 15%.
    #[test]
    fn test_bc_2_01_004_hlink_darkened_from_acc1() {
        let declared: [Option<&str>; 12] = [
            Some("#1F2937"),
            Some("#FFFFFF"),
            None,
            None,
            Some("#3B82F6"), // acc1
            None,
            None,
            None,
            None,
            None,
            None, // hlink absent
            None,
        ];
        let mut warnings = Vec::new();
        let result = infer_missing_slots(declared, &mut warnings);
        // hlink is at index 10; must be acc1 darkened 15% → different from acc1
        let hlink = result[10].as_ref();
        assert_ne!(hlink, "#3B82F6", "hlink must be darkened from acc1");
    }

    /// BC-2.01.004 — `fol_hlink` inferred as `hlink` darkened 10%.
    #[test]
    fn test_bc_2_01_004_fol_hlink_darkened_from_hlink() {
        let declared: [Option<&str>; 12] = [
            Some("#1F2937"),
            Some("#FFFFFF"),
            None,
            None,
            Some("#3B82F6"), // acc1
            None,
            None,
            None,
            None,
            None,
            None, // hlink absent — inferred
            None, // fol_hlink absent — inferred from inferred hlink
        ];
        let mut warnings = Vec::new();
        let result = infer_missing_slots(declared, &mut warnings);
        // fol_hlink (index 11) must differ from hlink (index 10)
        assert_ne!(
            result[11].as_ref(),
            result[10].as_ref(),
            "fol_hlink must differ from hlink after 10% darkening"
        );
    }
}
