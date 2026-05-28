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
//!
//! ## Cross-platform determinism
//!
//! HSL arithmetic uses `f32`. The snapshot test `test_inference_snapshot_deterministic`
//! is asserted on whatever platform CI runs the snapshot job (currently
//! `Linux x86_64`). For full cross-platform determinism verification, a multi-OS
//! snapshot job (matrix: linux-x86_64, linux-arm64, macos-arm64, windows-x86_64)
//! would need to assert byte-equality of the rendered XML. This is a CI matrix
//! improvement tracked outside this story; the f32 arithmetic in question is
//! deterministic per IEEE 754 across all supported architectures, so the risk
//! is theoretical rather than empirical.
//!
//! If strict cross-platform bit-exact color determinism is required in a
//! future version, migrate to integer arithmetic (fixed-point HSL in
//! [0, 3600] degrees / [0, 1000] saturation+lightness) or use a soft-float
//! library.

use std::sync::Arc;

use crate::error::BrandError;

// ─── Hex validation ──────────────────────────────────────────────────────────

/// Validate and normalise a user-declared hex color string from `brand.toml`.
///
/// Accepts `"#RRGGBB"` hex strings with 6 ASCII hex digits (upper or lowercase).
/// Lowercase digits are accepted and normalised to uppercase on return
/// (F-PASS11-LOW-4: user-typed lowercase hex like `"#3b82f6"` → `"#3B82F6"`).
///
/// Returns `Err(BrandError::InvalidHexColor)` for any structurally invalid input:
/// - Named CSS colors (`"red"`)
/// - Short hex (`"#3B82F"`, 5 hex digits)
/// - Alpha hex (`"#3B82F6FF"`, 8 hex digits)
/// - Empty string (`""`)
/// - Non-hex digit in value body (`"#ZZZZZZ"`)
///
/// All returned `Arc<str>` values are uppercase `"#RRGGBB"` (invariant 3).
/// This is called for every user-declared slot before it enters the inference
/// pipeline. Values that pass validation are stored normalised to uppercase.
pub(crate) fn validate_hex(slot_name: &str, value: &str) -> Result<Arc<str>, BrandError> {
    if value.len() != 7 || !value.starts_with('#') {
        return Err(BrandError::InvalidHexColor {
            slot_name: Arc::from(slot_name),
            value: Arc::from(value),
        });
    }
    let hex_digits = &value[1..];
    if !hex_digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(BrandError::InvalidHexColor {
            slot_name: Arc::from(slot_name),
            value: Arc::from(value),
        });
    }
    // Normalise to uppercase (F-PASS11-LOW-4): user-typed lowercase is accepted
    // and stored as uppercase so that all downstream invariants (invariant 3) hold.
    Ok(Arc::from(value.to_ascii_uppercase().as_str()))
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Slot name constants in ECMA-376 order (matches `COLOR_SLOT_NAMES` from template.rs).
///
/// Position 11 is `"folHlink"` (camelCase) — the canonical ECMA-376 OOXML name.
/// The TOML field is `fol_hlink` (`snake_case`, via serde rename), but the color slot
/// name stored in [`crate::template::ColorSlot`] and in BC-2.01.002 must be `"folHlink"`.
const SLOT_NAMES: [&str; 12] = [
    "dk1", "lt1", "dk2", "lt2", "acc1", "acc2", "acc3", "acc4", "acc5", "acc6", "hlink", "folHlink",
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

    // Build results array — validate and clone declared values.
    // Invalid hex values produce BrandError::InvalidHexColor and are treated as absent
    // (inference continues with the remaining slots).
    // Track which slots had validation failures: a slot that was declared but invalid
    // is "declared" in the BC-2.01.004 invariant sense — it must NOT also emit a
    // MissingColorSlot warning (only one warning per slot, OBS-1 fix).
    let mut result: [Option<Arc<str>>; 12] = [
        None, None, None, None, None, None, None, None, None, None, None, None,
    ];
    let mut had_validation_failure: [bool; 12] = [false; 12];
    for (i, v) in declared.iter().enumerate() {
        if let Some(hex) = v {
            match validate_hex(SLOT_NAMES[i], hex) {
                Ok(validated) => result[i] = Some(validated),
                Err(e) => {
                    warnings.push(e);
                    had_validation_failure[i] = true;
                },
            }
        }
    }

    // Helper: emit warning and set inferred value.
    // Skips MissingColorSlot emission for slots that already received an
    // InvalidHexColor warning (one warning per slot — BC-2.01.004 invariant 1).
    let infer = |result: &mut [Option<Arc<str>>; 12],
                 warnings: &mut Vec<BrandError>,
                 idx: usize,
                 value: &str,
                 derivation: &str| {
        if !had_validation_failure[idx] {
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
        }
        result[idx] = Some(Arc::from(value));
    };

    // ── Index references ──
    // 0 dk1, 1 lt1, 2 dk2, 3 lt2, 4 acc1, 5 acc2, 6 acc3, 7 acc4, 8 acc5, 9 acc6,
    // 10 hlink, 11 fol_hlink

    // Rule: dk1 — darkest declared color; fallback "#1F2937"
    // Use the post-validation `result` array so only valid uppercase hex values
    // participate in the luminance comparison (F6/F8 fix).
    if result[0].is_none() {
        let darkest = darkest_from_result(&result);
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
/// Returns `(0.0, 0.0, 0.0)` (black) for any malformed input — this function is
/// called from pure inference paths only, where all inputs have been validated
/// by `validate_hex` at the entry point.
#[allow(clippy::many_single_char_names)]
fn hex_to_hsl(hex: &str) -> (f32, f32, f32) {
    // Parse #RRGGBB — strip optional '#' prefix.
    let hex = hex.trim_start_matches('#');
    // Guard: inputs shorter than 6 chars (e.g. from tests or edge cases) return black
    // rather than panicking on out-of-bounds slice indexing (F9 fix).
    if hex.len() < 6 {
        return (0.0, 0.0, 0.0);
    }
    // Safe parse: bad input returns 0 (black component).
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
/// Extreme inputs (`pct = 1.0`) silently clamp to `#000000` in release builds.
/// In debug builds, an out-of-range `pct` triggers `debug_assert!` to surface
/// unintentional misuse (F-PASS11-LOW-5).
/// Returns an uppercase `"#RRGGBB"` string.
#[allow(clippy::many_single_char_names)]
pub(crate) fn darken_hex(hex: &str, pct: f32) -> String {
    debug_assert!(
        (0.0..=1.0).contains(&pct),
        "darken_hex: percentage out of range: {pct} (expected 0.0..=1.0)"
    );
    let (hue, sat, lum) = hex_to_hsl(hex);
    hsl_to_hex(hue, sat, (lum * (1.0 - pct)).max(0.0))
}

/// Lighten a `"#RRGGBB"` hex color by `pct` towards white.
///
/// `pct` is in `[0.0, 1.0]` (e.g., `0.20` = blend 20% towards white).
/// Extreme inputs (`pct = 1.0`) silently clamp to `#FFFFFF` in release builds.
/// In debug builds, an out-of-range `pct` triggers `debug_assert!` to surface
/// unintentional misuse (F-PASS11-LOW-5).
/// Returns an uppercase `"#RRGGBB"` string.
#[allow(clippy::many_single_char_names)]
pub(crate) fn lighten_hex(hex: &str, pct: f32) -> String {
    debug_assert!(
        (0.0..=1.0).contains(&pct),
        "lighten_hex: percentage out of range: {pct} (expected 0.0..=1.0)"
    );
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

/// Find the darkest color (by luminance) among post-validation result slots.
///
/// Operates on the `[Option<Arc<str>>; 12]` result array (post-validation,
/// so all values are valid uppercase `"#RRGGBB"` or `None`).
///
/// Returns `"#1F2937"` (the fallback dark) if all slots are `None`.
/// The returned string is always uppercase (F8 fix).
#[allow(clippy::many_single_char_names)]
fn darkest_from_result(slots: &[Option<Arc<str>>; 12]) -> String {
    /// Compute relative luminance (WCAG formula) for a hex color.
    ///
    /// Returns `1.0` (maximum luminance, treated as "not dark") for any input
    /// shorter than 6 hex digits, so short/invalid inputs are never chosen as
    /// the darkest color — a safe fallback (F9 fix).
    fn luminance(hex: &str) -> f32 {
        let hex = hex.trim_start_matches('#');
        // Guard: shorter than 6 chars → return max luminance (not dark).
        if hex.len() < 6 {
            return 1.0;
        }
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

    let mut darkest: Option<(Arc<str>, f32)> = None;
    for slot in slots.iter().flatten() {
        let lum = luminance(slot.as_ref());
        match darkest {
            None => darkest = Some((Arc::clone(slot), lum)),
            Some((_, prev_lum)) if lum < prev_lum => darkest = Some((Arc::clone(slot), lum)),
            _ => {},
        }
    }
    match darkest {
        // All values are already validated uppercase — return as-is (F8).
        Some((hex, _)) => hex.as_ref().to_owned(),
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

    // ─── F6: validate_hex tests ───────────────────────────────────────────────

    /// F6 — valid uppercase hex passes validation.
    #[test]
    fn test_f6_validate_hex_valid_uppercase() {
        assert!(validate_hex("acc1", "#3B82F6").is_ok());
        assert!(validate_hex("dk1", "#1F2937").is_ok());
        assert!(validate_hex("lt1", "#FFFFFF").is_ok());
        assert!(validate_hex("lt2", "#F9FAFB").is_ok());
    }

    /// F6 — CSS named color is rejected.
    #[test]
    fn test_f6_validate_hex_rejects_named_color() {
        let err = validate_hex("acc1", "red").expect_err("named color must be rejected");
        assert!(
            matches!(err, crate::error::BrandError::InvalidHexColor { .. }),
            "named color must produce InvalidHexColor, got: {err:?}"
        );
    }

    /// F6 — 5-char hex (short) is rejected.
    #[test]
    fn test_f6_validate_hex_rejects_short_hex() {
        let err = validate_hex("acc1", "#3B82F").expect_err("short hex must be rejected");
        assert!(
            matches!(err, crate::error::BrandError::InvalidHexColor { .. }),
            "short hex must produce InvalidHexColor, got: {err:?}"
        );
    }

    /// F-PASS11-LOW-4 — lowercase hex is accepted and normalised to uppercase.
    ///
    /// This test replaces the old rejection test. User-typed lowercase hex
    /// (`"#3b82f6"`) is now accepted and stored as `"#3B82F6"`.
    #[test]
    fn test_validate_hex_accepts_lowercase_and_normalizes() {
        let result = validate_hex("acc1", "#3b82f6")
            .expect("F-PASS11-LOW-4: lowercase hex must be accepted and normalised");
        assert_eq!(
            result.as_ref(),
            "#3B82F6",
            "F-PASS11-LOW-4: lowercase hex must normalise to uppercase"
        );
    }

    /// F-PASS11-LOW-4 — mixed-case hex is accepted and normalised to uppercase.
    #[test]
    fn test_validate_hex_accepts_mixed_case_and_normalizes() {
        let result = validate_hex("acc1", "#3B82f6")
            .expect("mixed-case hex must be accepted and normalised");
        assert_eq!(
            result.as_ref(),
            "#3B82F6",
            "mixed-case hex must normalise to uppercase"
        );
    }

    /// F-PASS11-LOW-4 — uppercase hex passes through unchanged (idempotent).
    #[test]
    fn test_validate_hex_uppercase_passthrough() {
        let result = validate_hex("acc1", "#3B82F6").expect("uppercase hex must pass through");
        assert_eq!(
            result.as_ref(),
            "#3B82F6",
            "uppercase hex must be returned unchanged"
        );
    }

    /// F6 — alpha hex (8 digits) is rejected.
    #[test]
    fn test_f6_validate_hex_rejects_alpha_hex() {
        let err = validate_hex("acc1", "#3B82F6FF").expect_err("alpha hex must be rejected");
        assert!(
            matches!(err, crate::error::BrandError::InvalidHexColor { .. }),
            "alpha hex must produce InvalidHexColor, got: {err:?}"
        );
    }

    /// F6 — empty string is rejected.
    #[test]
    fn test_f6_validate_hex_rejects_empty_string() {
        let err = validate_hex("acc1", "").expect_err("empty string must be rejected");
        assert!(
            matches!(err, crate::error::BrandError::InvalidHexColor { .. }),
            "empty string must produce InvalidHexColor, got: {err:?}"
        );
    }

    /// F6 — invalid hex value (non-hex digit) is rejected.
    #[test]
    fn test_f6_validate_hex_rejects_non_hex_digit() {
        let err = validate_hex("acc1", "#ZZZZZZ").expect_err("non-hex digits must be rejected");
        assert!(
            matches!(err, crate::error::BrandError::InvalidHexColor { .. }),
            "non-hex digit must produce InvalidHexColor, got: {err:?}"
        );
    }

    /// F6 + F8 — invalid hex in declared slot produces warning (not silent black fallback).
    #[test]
    fn test_f6_invalid_declared_hex_produces_warning_not_silent_black() {
        // "red" is not a valid hex — must produce InvalidHexColor warning,
        // then dk1 is treated as absent and inferred (not silently set to black).
        let declared: [Option<&str>; 12] = [
            Some("red"), // invalid → warning + treated as absent
            Some("#FFFFFF"),
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
        // Must have at least one InvalidHexColor warning for "red"
        let has_invalid_hex = warnings
            .iter()
            .any(|w| matches!(w, crate::error::BrandError::InvalidHexColor { .. }));
        assert!(
            has_invalid_hex,
            "F6: invalid hex 'red' must produce InvalidHexColor warning, got: {warnings:?}"
        );
        // dk1 must be inferred (not black #000000 and not "red")
        let dk1 = result[0].as_ref();
        assert_ne!(
            dk1, "#000000",
            "F6: dk1 must not be #000000 (silent black fallback)"
        );
        assert_ne!(dk1, "red", "F6: dk1 must not be the invalid 'red' value");
    }

    // ─── F-PASS11-LOW-5: darken_hex / lighten_hex clamp tests ────────────────

    /// F-PASS11-LOW-5 — `darken_hex` with 0% is identity.
    #[test]
    fn test_darken_hex_zero_pct_is_identity() {
        let result = darken_hex("#3B82F6", 0.0);
        assert_eq!(
            result, "#3B82F6",
            "darken_hex with 0% must return input unchanged"
        );
    }

    /// F-PASS11-LOW-5 — `lighten_hex` with 0% is identity.
    #[test]
    fn test_lighten_hex_zero_pct_is_identity() {
        let result = lighten_hex("#3B82F6", 0.0);
        assert_eq!(
            result, "#3B82F6",
            "lighten_hex with 0% must return input unchanged"
        );
    }

    /// F-PASS11-LOW-5 — `darken_hex` with 100% produces black (release mode only).
    ///
    /// In debug mode, `debug_assert!` would fire for extreme inputs like `1.0`,
    /// so this test is guarded by `#[cfg(not(debug_assertions))]`.
    /// In release mode, the function clamps to `#000000`.
    #[test]
    #[cfg(not(debug_assertions))]
    fn test_darken_hex_one_pct_is_black_in_release_only() {
        let result = darken_hex("#FF0000", 1.0);
        assert_eq!(
            result, "#000000",
            "darken_hex with 100% must clamp to #000000 in release mode"
        );
    }

    /// F-PASS11-LOW-5 — `lighten_hex` with 100% produces white.
    #[test]
    fn test_lighten_hex_one_pct_is_white() {
        // pct=1.0 is within [0.0, 1.0] — debug_assert! must NOT fire.
        // In release, this clamps to #FFFFFF.
        let result = lighten_hex("#000000", 1.0);
        assert_eq!(
            result, "#FFFFFF",
            "lighten_hex with 100% must clamp to #FFFFFF"
        );
    }

    /// OBS-1 / BC-2.01.004 invariant 1 — invalid hex slot produces exactly ONE warning.
    ///
    /// When a slot is declared but has an invalid value (e.g. `acc1 = "red"`), the
    /// code must emit only the `InvalidHexColor` warning and suppress the subsequent
    /// `MissingColorSlot` warning for the same slot. "Declared but invalid" counts as
    /// "declared" for the invariant: one warning per slot, not two.
    #[test]
    fn test_obs1_invalid_hex_produces_exactly_one_warning_per_slot() {
        // acc1 declared as invalid; all other optional slots absent.
        // dk1 and lt1 are also absent (they will each produce a MissingColorSlot).
        let declared: [Option<&str>; 12] = [
            None,        // dk1 absent → MissingColorSlot (1 warning)
            None,        // lt1 absent → MissingColorSlot (1 warning)
            None,        // dk2 absent
            None,        // lt2 absent
            Some("red"), // acc1 INVALID → InvalidHexColor only (NOT also MissingColorSlot)
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ];
        let mut warnings = Vec::new();
        let _ = infer_missing_slots(declared, &mut warnings);

        // Exactly one warning for the acc1 slot (InvalidHexColor), not two (InvalidHexColor + MissingColorSlot).
        let invalid_hex_for_acc1 = warnings
            .iter()
            .filter(|w| {
                matches!(w, crate::error::BrandError::InvalidHexColor { slot_name, .. }
                    if slot_name.as_ref() == "acc1")
            })
            .count();
        let missing_slot_for_acc1 = warnings
            .iter()
            .filter(|w| {
                matches!(w, crate::error::BrandError::MissingColorSlot { slot_name, .. }
                    if slot_name.as_ref() == "acc1")
            })
            .count();

        assert_eq!(
            invalid_hex_for_acc1, 1,
            "OBS-1: exactly one InvalidHexColor warning for invalid acc1, got: {warnings:?}"
        );
        assert_eq!(
            missing_slot_for_acc1, 0,
            "OBS-1: MissingColorSlot must NOT be emitted for a slot that had InvalidHexColor \
             (one warning per slot — BC-2.01.004 invariant 1), got: {warnings:?}"
        );
    }
}
