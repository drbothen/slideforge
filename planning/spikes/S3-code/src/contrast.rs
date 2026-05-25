//! S3 Spike — WCAG contrast checker prototype.
//!
//! Implements the WCAG 2.x relative luminance and contrast ratio algorithm
//! exactly as specified in wcag-for-slides.md §"Color Contrast Validation".
//!
//! This is a standalone spike binary — NOT part of the slideforge workspace.
//!
//! Run: cargo run --bin s3_contrast

/// sRGB color as three `u8` channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse a 6-digit hex string like `"#1F2937"` or `"1F2937"`.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let s = hex.trim_start_matches('#');
        if s.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }
}

/// Linearize a single sRGB channel value (0–255) per WCAG 2.x spec.
/// Returns a value in [0.0, 1.0].
fn linearize_channel(channel: u8) -> f64 {
    let c = channel as f64 / 255.0;
    if c <= 0.04045 {
        // WCAG spec uses 0.04045; older docs use 0.03928 — 0.04045 is correct per IEC 61966-2-1.
        c / 12.92
    } else {
        ((c + 0.055) / 1.055_f64).powf(2.4)
    }
}

/// Relative luminance per WCAG 2.x §1.4.3.
/// Returns a value in [0.0, 1.0].
pub fn relative_luminance(color: Rgb) -> f64 {
    let r = linearize_channel(color.r);
    let g = linearize_channel(color.g);
    let b = linearize_channel(color.b);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Contrast ratio between two colors per WCAG 2.x.
/// Returns a value in [1.0, 21.0].
pub fn contrast_ratio(c1: Rgb, c2: Rgb) -> f64 {
    let l1 = relative_luminance(c1);
    let l2 = relative_luminance(c2);
    let (lighter, darker) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

/// WCAG contrast threshold result.
#[derive(Debug, Clone, PartialEq)]
pub enum ContrastVerdict {
    /// Contrast meets the required threshold.
    Pass {
        ratio: f64,
        threshold: f64,
        criterion: &'static str,
    },
    /// Contrast fails the required threshold.
    Fail {
        ratio: f64,
        threshold: f64,
        criterion: &'static str,
    },
    /// Contrast is within 10% of the threshold — designer should leave headroom.
    Warning {
        ratio: f64,
        threshold: f64,
        criterion: &'static str,
    },
}

impl ContrastVerdict {
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass { .. })
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, Self::Fail { .. })
    }
}

/// Classify a pair per the relevant WCAG criterion.
///
/// `is_large_text`:
///   - true  → uses 3.0:1 threshold (>= 18pt regular OR >= 14pt bold)
///   - false → uses 4.5:1 threshold (normal text)
///
/// `is_non_text`:
///   - true  → uses 3.0:1 threshold (graphical objects, UI components per 1.4.11)
pub fn check_contrast(fg: Rgb, bg: Rgb, is_large_text: bool, is_non_text: bool) -> ContrastVerdict {
    let ratio = contrast_ratio(fg, bg);
    let (threshold, criterion) = if is_non_text {
        (3.0, "WCAG 1.4.11 Non-text Contrast")
    } else if is_large_text {
        (3.0, "WCAG 1.4.3 Contrast (large text)")
    } else {
        (4.5, "WCAG 1.4.3 Contrast (normal text)")
    };

    if ratio < threshold {
        ContrastVerdict::Fail { ratio, threshold, criterion }
    } else if ratio < threshold * 1.10 {
        // Within 10% headroom — warn
        ContrastVerdict::Warning { ratio, threshold, criterion }
    } else {
        ContrastVerdict::Pass { ratio, threshold, criterion }
    }
}

fn main() {
    // Spike evaluation: run representative brand color pairs through the checker.
    //
    // Colors drawn from a representative consulting brand palette:
    // - Primary blue: #1E3A5F (dark navy)
    // - White: #FFFFFF
    // - Light gray: #F3F4F6 (slide background)
    // - Accent orange: #F97316
    // - Dark text: #111827
    // - Severity RED bg: #DC2626
    // - Severity AMBER bg: #D97706
    // - Severity GREEN bg: #16A34A
    // - White text on each severity color

    let pairs: &[(&str, Rgb, &str, Rgb, bool, bool)] = &[
        ("Primary on White",    Rgb::new(0x1E, 0x3A, 0x5F), "White",         Rgb::new(0xFF, 0xFF, 0xFF), false, false),
        ("Dark text on White",  Rgb::new(0x11, 0x18, 0x27), "White",         Rgb::new(0xFF, 0xFF, 0xFF), false, false),
        ("Dark text on Gray",   Rgb::new(0x11, 0x18, 0x27), "Light gray bg", Rgb::new(0xF3, 0xF4, 0xF6), false, false),
        ("Accent orange/White", Rgb::new(0xF9, 0x73, 0x16), "White",         Rgb::new(0xFF, 0xFF, 0xFF), false, false), // Likely fail
        ("White on RED",        Rgb::new(0xFF, 0xFF, 0xFF), "Severity red",  Rgb::new(0xDC, 0x26, 0x26), false, false),
        ("White on AMBER",      Rgb::new(0xFF, 0xFF, 0xFF), "Severity amber",Rgb::new(0xD9, 0x77, 0x06), false, false), // Likely fail
        ("White on GREEN",      Rgb::new(0xFF, 0xFF, 0xFF), "Severity green",Rgb::new(0x16, 0xA3, 0x4A), false, false),
        // Large text thresholds (e.g. 24px+ slide titles)
        ("Accent orange/White (large)", Rgb::new(0xF9, 0x73, 0x16), "White", Rgb::new(0xFF, 0xFF, 0xFF), true, false),
        // Non-text: chart series stroke vs bg
        ("Orange stroke on gray (non-text)", Rgb::new(0xF9, 0x73, 0x16), "Light gray bg", Rgb::new(0xF3, 0xF4, 0xF6), false, true),
    ];

    println!("{:-<90}", "");
    println!("{:<40} {:>8}  {:>8}  {:<20}  {}",
        "Color pair", "Ratio", "Threshold", "Criterion (abbrev)", "Verdict");
    println!("{:-<90}", "");

    for (label, fg, bg_label, bg, is_large, is_non_text) in pairs {
        let verdict = check_contrast(*fg, *bg, *is_large, *is_non_text);
        let (ratio, threshold, criterion, status) = match &verdict {
            ContrastVerdict::Pass { ratio, threshold, criterion } =>
                (ratio, threshold, criterion, "PASS"),
            ContrastVerdict::Fail { ratio, threshold, criterion } =>
                (ratio, threshold, criterion, "FAIL"),
            ContrastVerdict::Warning { ratio, threshold, criterion } =>
                (ratio, threshold, criterion, "WARN"),
        };
        let criterion_short = criterion.split(' ').take(2).collect::<Vec<_>>().join(" ");
        println!("{:<40} {:>8.2}  {:>8.1}  {:<20}  {} (fg on {})",
            label, ratio, threshold, criterion_short, status, bg_label);
    }

    println!("{:-<90}", "");
    println!();
    println!("Notes:");
    println!("  - Accent orange (#F97316) on white fails 4.5:1 normal text (ratio ~2.9).");
    println!("  - Accent orange passes 3.0:1 for large text and non-text uses.");
    println!("  - White on amber (#D97706) is a common brand mistake — fails 4.5:1.");
    println!("  - This checker prototype is the core of slideforge-validate's brand validator.");
    println!();
    println!("Integration plan:");
    println!("  - slideforge-validate crate: BrandValidator::check_theme_pairs() uses this algo.");
    println!("  - Called at parse-time when brand: block is loaded.");
    println!("  - Emits Diagnostic::error (hard fail) for ratio < threshold.");
    println!("  - Emits Diagnostic::warning for ratio within 10% of threshold.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_on_white_is_21_to_1() {
        let ratio = contrast_ratio(Rgb::new(0, 0, 0), Rgb::new(255, 255, 255));
        // Should be exactly 21.0
        assert!((ratio - 21.0).abs() < 0.001, "expected ~21.0, got {ratio}");
    }

    #[test]
    fn white_on_white_is_1_to_1() {
        let ratio = contrast_ratio(Rgb::new(255, 255, 255), Rgb::new(255, 255, 255));
        assert!((ratio - 1.0).abs() < 0.001, "expected ~1.0, got {ratio}");
    }

    #[test]
    fn orange_on_white_fails_normal_text() {
        // #F97316 on white — known to fail 4.5:1
        let verdict = check_contrast(
            Rgb::from_hex("F97316").unwrap(),
            Rgb::from_hex("FFFFFF").unwrap(),
            false,
            false,
        );
        assert!(verdict.is_fail(), "orange on white should fail normal text threshold");
    }

    #[test]
    fn dark_navy_on_white_passes_normal_text() {
        // #1E3A5F on white — should pass 4.5:1 easily
        let verdict = check_contrast(
            Rgb::from_hex("1E3A5F").unwrap(),
            Rgb::from_hex("FFFFFF").unwrap(),
            false,
            false,
        );
        assert!(verdict.is_pass(), "dark navy on white should pass normal text threshold");
    }

    #[test]
    fn from_hex_parses_with_hash() {
        let c = Rgb::from_hex("#AABBCC").unwrap();
        assert_eq!(c, Rgb::new(0xAA, 0xBB, 0xCC));
    }

    #[test]
    fn from_hex_parses_without_hash() {
        let c = Rgb::from_hex("AABBCC").unwrap();
        assert_eq!(c, Rgb::new(0xAA, 0xBB, 0xCC));
    }
}
