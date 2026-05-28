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

/// Slot name constants in ECMA-376 order (matches `COLOR_SLOT_NAMES` from template.rs).
///
/// Position 11 is `"folHlink"` (camelCase) — the canonical ECMA-376 OOXML name.
/// The TOML field is `fol_hlink` (snake_case, via serde rename), but the color slot
/// name stored in [`crate::template::ColorSlot`] and in BC-2.01.002 must be `"folHlink"`.
const SLOT_NAMES: [&str; 12] = [
    "dk1",
    "lt1",
    "dk2",
    "lt2",
    "acc1",
    "acc2",
    "acc3",
    "acc4",
    "acc5",
    "acc6",
    "hlink",
    "folHlink",
];

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
    declared: [Option<&str>; 12],
    warnings: &mut Vec<BrandError>,
) -> [Arc<str>; 12] {
    // Fallback defaults
    const DEFAULT_DK1: &str = "#1F2937";
    const DEFAULT_LT1: &str = "#FFFFFF";
    const DEFAULT_LT2: &str = "#F9FAFB";
    const DEFAULT_ACC1: &str = "#3B82F6";

    // Build results array — start by cloning declared values.
    let mut result: [Option<Arc<str>>; 12] = [
        None, None, None, None, None, None, None, None, None, None, None, None,
    ];
    for (i, v) in declared.iter().enumerate() {
        if let Some(hex) = v {
            result[i] = Some(Arc::from(*hex));
        }
    }

    // Helper: emit warning and set inferred value.
    let infer = |result: &mut [Option<Arc<str>>; 12],
                 warnings: &mut Vec<BrandError>,
                 idx: usize,
                 value: &str,
                 derivation: &str| {
        tracing::warn!(
            "E-BRD-003: Color slot '{}' not declared in brand.toml. Using inferred value '{}'.",
            SLOT_NAMES[idx],
            value,
        );
        warnings.push(BrandError::MissingColorSlot {
            slot_name: Arc::from(SLOT_NAMES[idx]),
            inferred_hex: Arc::from(value),
            derivation: Arc::from(derivation),
        });
        result[idx] = Some(Arc::from(value));
    };

    // ── Index references ──
    // 0 dk1, 1 lt1, 2 dk2, 3 lt2, 4 acc1, 5 acc2, 6 acc3, 7 acc4, 8 acc5, 9 acc6,
    // 10 hlink, 11 fol_hlink

    // Rule: dk1 — darkest declared color; fallback "#1F2937"
    if result[0].is_none() {
        let darkest = darkest_declared_owned(&declared);
        infer(
            &mut result,
            warnings,
            0,
            &darkest,
            "darkest declared color by luminance; fallback #1F2937",
        );
    }

    // Rule: lt1 — always "#FFFFFF"
    if result[1].is_none() {
        infer(&mut result, warnings, 1, DEFAULT_LT1, "hardcoded #FFFFFF");
    }

    // Rule: acc1 — if still None, use default (needed for downstream rules)
    // (no warning: acc1 may be user-declared; if not, it's inferred below)
    let acc1_effective = if let Some(ref v) = result[4] {
        v.as_ref().to_owned()
    } else {
        DEFAULT_ACC1.to_owned()
    };

    // Rule: dk2 — acc1 if declared; else dk1 lightened 20%
    if result[2].is_none() {
        let dk2_val = if result[4].is_some() {
            // acc1 is declared — use acc1 value
            acc1_effective.clone()
        } else {
            // acc1 not declared — lighten dk1 by 20%
            let dk1_hex = result[0].as_ref().map_or(DEFAULT_DK1, |v| v.as_ref());
            lighten_hex(dk1_hex, 0.20)
        };
        infer(
            &mut result,
            warnings,
            2,
            &dk2_val,
            "acc1 if declared; else dk1 lightened 20%",
        );
    }

    // Rule: lt2 — always "#F9FAFB"
    if result[3].is_none() {
        infer(
            &mut result,
            warnings,
            3,
            DEFAULT_LT2,
            "hardcoded #F9FAFB near-white",
        );
    }

    // Rule: acc1 — if absent, infer from default
    if result[4].is_none() {
        infer(
            &mut result,
            warnings,
            4,
            DEFAULT_ACC1,
            "default accent baseline",
        );
    }

    // Re-read acc1 for downstream rotations
    let acc1_hex = result[4]
        .as_ref()
        .map_or(DEFAULT_ACC1, |v| v.as_ref())
        .to_owned();

    // Rule: acc2..acc6 — hue rotation 30°/60°/90°/120°/150° from acc1
    let rotations: [f32; 5] = [30.0, 60.0, 90.0, 120.0, 150.0];
    for (offset, &degrees) in rotations.iter().enumerate() {
        let idx = 5 + offset; // acc2=5..acc6=9
        if result[idx].is_none() {
            let rotated = rotate_hue(&acc1_hex, degrees);
            let derivation = format!("acc1 hue rotated {degrees}°");
            infer(&mut result, warnings, idx, &rotated, &derivation);
        }
    }

    // Rule: hlink — acc1 darkened 15%
    if result[10].is_none() {
        let hlink_val = darken_hex(&acc1_hex, 0.15);
        infer(&mut result, warnings, 10, &hlink_val, "acc1 darkened 15%");
    }

    // Rule: fol_hlink — hlink darkened 10%
    if result[11].is_none() {
        let hlink_hex = result[10].as_ref().map_or("", |v| v.as_ref()).to_owned();
        let fol_hlink_val = darken_hex(&hlink_hex, 0.10);
        infer(
            &mut result,
            warnings,
            11,
            &fol_hlink_val,
            "hlink darkened 10%",
        );
    }

    // Unwrap all — every slot is now populated
    result.map(|v| v.expect("all slots must be populated by inference"))
}

// ─── Color manipulation helpers ───────────────────────────────────────────────

/// Convert a `"#RRGGBB"` hex string to HSL components `(h°, s, l)`.
///
/// `h` is in `[0.0, 360.0)`, `s` and `l` are in `[0.0, 1.0]`.
///
/// # Panics
///
/// Panics in debug builds if `hex` is not a valid `"#RRGGBB"` string.
#[allow(clippy::many_single_char_names)]
fn hex_to_hsl(hex: &str) -> (f32, f32, f32) {
    // Parse #RRGGBB
    let hex = hex.trim_start_matches('#');
    // Safe parse: bad input returns black
    let r = f32::from(u8::from_str_radix(&hex[..2], 16).unwrap_or(0)) / 255.0;
    let g = f32::from(u8::from_str_radix(&hex[2..4], 16).unwrap_or(0)) / 255.0;
    let b = f32::from(u8::from_str_radix(&hex[4..6], 16).unwrap_or(0)) / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let l = f32::midpoint(max, min);

    if delta < f32::EPSILON {
        // Achromatic
        return (0.0, 0.0, l);
    }

    let s = if l > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };

    let h = if (max - r).abs() < f32::EPSILON {
        let mut h = (g - b) / delta;
        if g < b {
            h += 6.0;
        }
        h / 6.0 * 360.0
    } else if (max - g).abs() < f32::EPSILON {
        ((b - r) / delta + 2.0) / 6.0 * 360.0
    } else {
        ((r - g) / delta + 4.0) / 6.0 * 360.0
    };

    (h, s, l)
}

/// Helper for HSL-to-RGB conversion.
fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// Convert HSL components `(h°, s, l)` to an uppercase `"#RRGGBB"` hex string.
///
/// `h` is in `[0.0, 360.0)`, `s` and `l` are in `[0.0, 1.0]`.
/// The returned string is always 7 characters and uses uppercase hex digits.
#[allow(
    clippy::many_single_char_names,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn hsl_to_hex(hue: f32, sat: f32, lum: f32) -> String {
    /// Convert a [0.0, 1.0] float channel to [0, 255] u8.
    /// Safety: value is clamped before calling so truncation is safe.
    fn to_byte(v: f32) -> u8 {
        (v.clamp(0.0, 1.0) * 255.0).round() as u8
    }

    let (red, grn, blu) = if sat < f32::EPSILON {
        // Achromatic
        let v = to_byte(lum);
        (v, v, v)
    } else {
        let q = if lum < 0.5 {
            lum * (1.0 + sat)
        } else {
            lum + sat - lum * sat
        };
        let p = 2.0 * lum - q;
        let h_norm = hue / 360.0;

        let rv = to_byte(hue_to_rgb(p, q, h_norm + 1.0 / 3.0));
        let gv = to_byte(hue_to_rgb(p, q, h_norm));
        let bv = to_byte(hue_to_rgb(p, q, h_norm - 1.0 / 3.0));
        (rv, gv, bv)
    };

    format!("#{red:02X}{grn:02X}{blu:02X}")
}

/// Darken a `"#RRGGBB"` hex color by `pct` of its current lightness.
///
/// `pct` is in `[0.0, 1.0]` (e.g., `0.15` = darken by 15%).
/// Returns an uppercase `"#RRGGBB"` string.
#[allow(clippy::many_single_char_names)]
pub(crate) fn darken_hex(hex: &str, pct: f32) -> String {
    let (hue, sat, lum) = hex_to_hsl(hex);
    hsl_to_hex(hue, sat, (lum * (1.0 - pct)).max(0.0))
}

/// Lighten a `"#RRGGBB"` hex color by `pct` towards white.
///
/// `pct` is in `[0.0, 1.0]` (e.g., `0.20` = blend 20% towards white).
/// Returns an uppercase `"#RRGGBB"` string.
#[allow(clippy::many_single_char_names)]
pub(crate) fn lighten_hex(hex: &str, pct: f32) -> String {
    let (hue, sat, lum) = hex_to_hsl(hex);
    hsl_to_hex(hue, sat, (lum + (1.0 - lum) * pct).min(1.0))
}

/// Rotate the hue of a `"#RRGGBB"` hex color by `degrees`.
///
/// `degrees` can be any value; it is reduced modulo 360 before application.
/// Saturation and lightness are preserved.
/// Returns an uppercase `"#RRGGBB"` string.
#[allow(clippy::many_single_char_names)]
pub(crate) fn rotate_hue(hex: &str, degrees: f32) -> String {
    let (hue, sat, lum) = hex_to_hsl(hex);
    hsl_to_hex((hue + degrees).rem_euclid(360.0), sat, lum)
}

/// Find the darkest color (by luminance) among declared slots.
///
/// Returns `"#1F2937"` (the fallback dark) if `slots` contains no `Some` values.
#[allow(clippy::many_single_char_names)]
fn darkest_declared_owned(slots: &[Option<&str>; 12]) -> String {
    /// Compute relative luminance (WCAG formula) for a hex color.
    fn luminance(hex: &str) -> f32 {
        let hex = hex.trim_start_matches('#');
        let to_linear = |channel: u8| -> f32 {
            let srgb = f32::from(channel) / 255.0;
            if srgb <= 0.04045 {
                srgb / 12.92
            } else {
                ((srgb + 0.055) / 1.055_f32).powf(2.4)
            }
        };
        let red = u8::from_str_radix(&hex[..2], 16).unwrap_or(0);
        let grn = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let blu = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        0.2126 * to_linear(red) + 0.7152 * to_linear(grn) + 0.0722 * to_linear(blu)
    }

    let mut darkest: Option<(&str, f32)> = None;
    for slot in slots.iter().flatten() {
        let lum = luminance(slot);
        match darkest {
            None => darkest = Some((slot, lum)),
            Some((_, prev_lum)) if lum < prev_lum => darkest = Some((slot, lum)),
            _ => {},
        }
    }
    match darkest {
        Some((hex, _)) => hex.to_owned(),
        None => "#1F2937".to_owned(),
    }
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
        let _ = infer_missing_slots(declared, &mut warnings);
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
        assert_eq!(
            warnings.len(),
            12,
            "12 E-BRD-003 warnings for 12 absent slots"
        );
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
        assert_eq!(
            result[2].as_ref(),
            "#3B82F6",
            "dk2 should equal acc1 when acc1 declared"
        );
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
        assert_ne!(
            acc2, "#3B82F6",
            "acc2 must differ from acc1 after 30° rotation"
        );
        assert!(acc2.starts_with('#'), "acc2 must start with #");
        assert_eq!(acc2.len(), 7, "acc2 must be 7-char #RRGGBB");
        assert!(
            acc2[1..]
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()),
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
                h[1..]
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()),
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

    // ─── New behavioral tests for Red Gate ────────────────────────────────────

    /// BC-2.01.004 — all 12 slots provided → `ColorScheme` identical to input;
    /// zero warnings emitted (EC-003).
    #[test]
    fn test_bc_2_01_004_all_12_slots_provided_no_change() {
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
        let result = infer_missing_slots(declared, &mut warnings);
        // Zero warnings (EC-003: all 12 declared)
        assert_eq!(
            warnings.len(),
            0,
            "no E-BRD-003 warnings when all 12 declared"
        );
        // Returned values equal the declared input
        assert_eq!(result[0].as_ref(), "#1F2937", "dk1 must be unchanged");
        assert_eq!(result[1].as_ref(), "#FFFFFF", "lt1 must be unchanged");
        assert_eq!(result[4].as_ref(), "#3B82F6", "acc1 must be unchanged");
        assert_eq!(result[10].as_ref(), "#2563EB", "hlink must be unchanged");
        assert_eq!(
            result[11].as_ref(),
            "#1D4ED8",
            "fol_hlink must be unchanged"
        );
    }

    /// BC-2.01.004 / AC-003 — `lt2` is always `"#F9FAFB"` when absent (96% lightness
    /// near-white fallback).
    #[test]
    fn test_bc_2_01_004_lt2_inferred_as_near_white() {
        let declared: [Option<&str>; 12] = [
            Some("#1F2937"),
            Some("#FFFFFF"),
            None,
            None, // lt2 absent → must become "#F9FAFB"
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
        assert_eq!(
            result[3].as_ref(),
            "#F9FAFB",
            "lt2 must be inferred as #F9FAFB"
        );
    }

    /// BC-2.01.004 / AC-003 — `dk1` inferred as darkest declared color when absent;
    /// falls back to `"#1F2937"` when no colors declared.
    #[test]
    fn test_bc_2_01_004_dk1_inferred_fallback_when_no_colors_declared() {
        let declared: [Option<&str>; 12] = [None; 12];
        let mut warnings = Vec::new();
        let result = infer_missing_slots(declared, &mut warnings);
        // dk1 must use fallback when no colors at all
        let dk1 = result[0].as_ref();
        assert_eq!(
            dk1, "#1F2937",
            "dk1 fallback when all slots absent must be #1F2937"
        );
    }

    /// BC-2.01.004 invariant — inference is deterministic (same input → same output).
    #[test]
    fn test_bc_2_01_004_inference_is_deterministic() {
        let declared: [Option<&str>; 12] = [
            Some("#1F2937"),
            None,
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
        let mut w1 = Vec::new();
        let r1 = infer_missing_slots(declared, &mut w1);
        let mut w2 = Vec::new();
        let r2 = infer_missing_slots(declared, &mut w2);
        // Every slot must be equal across two independent calls
        for i in 0..12 {
            assert_eq!(
                r1[i].as_ref(),
                r2[i].as_ref(),
                "slot {i} must be deterministic across two identical calls"
            );
        }
        assert_eq!(w1.len(), w2.len(), "warning count must be deterministic");
    }

    /// BC-2.01.004 / AC-003 — `hlink` defaults to `acc1` when `hlink` absent and
    /// acc1 declared (hlink = acc1 darkened 15%).
    #[test]
    fn test_bc_2_01_004_hlink_defaults_to_acc1_darkened() {
        let declared: [Option<&str>; 12] = [
            Some("#1F2937"),
            Some("#FFFFFF"),
            None,
            None,
            Some("#3B82F6"), // acc1 = #3B82F6
            None,
            None,
            None,
            None,
            None,
            None, // hlink absent → acc1 darkened 15%
            None,
        ];
        let mut warnings = Vec::new();
        let result = infer_missing_slots(declared, &mut warnings);
        // hlink at index 10 must be a valid #RRGGBB (the darkened acc1)
        let hlink = result[10].as_ref();
        assert!(hlink.starts_with('#'), "hlink must start with #");
        assert_eq!(hlink.len(), 7, "hlink must be 7-char #RRGGBB");
        // Must be different from acc1 (darkened)
        assert_ne!(
            hlink, "#3B82F6",
            "hlink must differ from acc1 (should be darkened)"
        );
    }

    /// BC-2.01.004 — missing `[colors]` section (all None) produces 12 E-BRD-003
    /// warnings — one per inferred slot (EC-002).
    #[test]
    fn test_bc_2_01_004_no_required_slot_missing_section_produces_12_warnings() {
        let declared: [Option<&str>; 12] = [None; 12];
        let mut warnings = Vec::new();
        let _ = infer_missing_slots(declared, &mut warnings);
        assert_eq!(
            warnings.len(),
            12,
            "empty [colors] section must produce exactly 12 E-BRD-003 warnings"
        );
        // Each warning must be a MissingColorSlot variant
        for (i, warn) in warnings.iter().enumerate() {
            assert!(
                matches!(warn, crate::error::BrandError::MissingColorSlot { .. }),
                "warning {i} must be BrandError::MissingColorSlot, got: {warn:?}"
            );
        }
    }

    /// BC-2.01.004 — acc2..acc6 are generated from acc1 with 30°/60°/90°/120°/150° rotations.
    #[test]
    #[allow(clippy::similar_names)]
    fn test_bc_2_01_004_infer_missing_acc_slots_from_acc1() {
        let declared: [Option<&str>; 12] = [
            Some("#1F2937"),
            Some("#FFFFFF"),
            None,
            None,
            Some("#3B82F6"), // acc1 only; acc2..acc6 absent
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
        let acc1 = result[4].as_ref(); // should be the declared "#3B82F6"
        assert_eq!(acc1, "#3B82F6", "acc1 must not be changed");
        // acc2 (index 5) through acc6 (index 9) must each be distinct from acc1
        // and valid uppercase hex
        for (idx, slot) in result[5..=9].iter().enumerate() {
            let slot_idx = idx + 5;
            let acc = slot.as_ref();
            assert_ne!(acc, acc1, "acc slot {slot_idx} must differ from acc1");
            assert!(
                acc.starts_with('#'),
                "acc slot {slot_idx} must start with #"
            );
            assert_eq!(acc.len(), 7, "acc slot {slot_idx} must be 7-char #RRGGBB");
            assert!(
                acc[1..]
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()),
                "acc slot {slot_idx} must be uppercase hex"
            );
        }
        // acc2 through acc6 must also all be distinct from each other
        // (different hue rotations: 30°, 60°, 90°, 120°, 150°)
        let accs: Vec<&str> = (5..=9).map(|i| result[i].as_ref()).collect();
        let unique: std::collections::HashSet<&str> = accs.iter().copied().collect();
        assert_eq!(
            unique.len(),
            5,
            "acc2..acc6 must all be distinct colors (different hue rotations)"
        );
    }
}
