//! `brand.toml` schema — strongly-typed `serde` structs for brand synthesis input.
//!
//! [`BrandConfig`] is the root deserialization target for a `brand.toml` file.
//! It captures all four top-level sections (`[colors]`, `[fonts]`, `[logo]`,
//! `[footer]`) as defined by BC-2.01.002 postcondition 1.
//!
//! All fields are optional at the schema level. Required-field enforcement is
//! performed by `BrandSynthesizer::synthesize` (BC-2.01.002 / BC-2.01.004).
//!
//! ## Ownership design
//!
//! Schema structs use `String` (and `Option<String>`) for all string fields.
//! The synthesizer converts them to `Arc<str>` when building the `BrandTemplate`.
//! This separation avoids the need for a custom serde impl for `Arc<str>`.
//!
//! ## Example `brand.toml`
//!
//! ```toml
//! [colors]
//! dk1 = "#1F2937"
//! lt1 = "#FFFFFF"
//! acc1 = "#3B82F6"
//!
//! [fonts]
//! heading = "Aptos Display"
//! body = "Aptos"
//!
//! [logo]
//! path = "brand.assets/logo.png"
//!
//! [footer]
//! text = "Confidential"
//! show_slide_number = true
//! show_date = false
//! ```

use serde::{Deserialize, Serialize};

// ─── Root ─────────────────────────────────────────────────────────────────────

/// Root brand configuration parsed from `brand.toml`.
///
/// All sections and their fields are optional at the schema level. The
/// synthesizer applies defaults and inference for any absent values.
///
/// Deserialize via `toml::from_str::<BrandConfig>(&content)`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BrandConfig {
    /// The `[colors]` section — 12 OOXML theme color slots.
    ///
    /// Any absent slot is deterministically inferred by the synthesizer
    /// (AC-003). All 12 absent → 12 E-BRD-003 warnings with default baseline.
    #[serde(default)]
    pub colors: ColorConfig,

    /// The `[fonts]` section — heading and body font names.
    ///
    /// Absent fields fall back to `"Calibri"`.
    #[serde(default)]
    pub fonts: FontConfig,

    /// The `[logo]` section — path to the logo image.
    ///
    /// Required for synthesized brands (AC-004). Absent → `E-BRD-001` fatal error.
    #[serde(default)]
    pub logo: Option<LogoConfig>,

    /// The `[footer]` section — footer text and visibility flags.
    #[serde(default)]
    pub footer: FooterConfig,
}

impl BrandConfig {
    /// Construct a minimal valid `BrandConfig` for testing.
    ///
    /// Sets `acc1 = "#3B82F6"` and a dummy logo path so the synthesizer can
    /// run. Only `acc1` and `dk1` are specified; the remaining 10 slots are
    /// inferred.
    #[must_use]
    pub fn default_minimal() -> Self {
        Self {
            colors: ColorConfig {
                dk1: Some("#1F2937".to_owned()),
                acc1: Some("#3B82F6".to_owned()),
                ..Default::default()
            },
            fonts: FontConfig::default(),
            logo: Some(LogoConfig {
                path: "brand.assets/logo.png".to_owned(),
            }),
            footer: FooterConfig::default(),
        }
    }
}

// ─── [colors] ────────────────────────────────────────────────────────────────

/// The `[colors]` section of `brand.toml`.
///
/// Contains all 12 OOXML theme color slots as optional hex strings.
/// Field names match OOXML canonical names (`dk1`, `lt1`, …, `fol_hlink`).
/// The `fol_hlink` TOML name maps to OOXML `folHlink` via `#[serde(rename = "...")]`.
///
/// String fields use `Option<String>` for serde compatibility. The synthesizer
/// converts them to `Arc<str>` when building the `BrandTemplate`.
///
/// All absent fields produce one `E-BRD-003` warning each and are inferred
/// by the deterministic algorithm in `inference::infer_missing_slots`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ColorConfig {
    /// Dark 1 — primary dark color (e.g., text on light backgrounds).
    ///
    /// Mapped to OOXML `<a:dk1>` element.
    #[serde(default)]
    pub dk1: Option<String>,

    /// Light 1 — primary light color (e.g., background).
    ///
    /// Mapped to OOXML `<a:lt1>` element.
    #[serde(default)]
    pub lt1: Option<String>,

    /// Dark 2 — secondary dark color.
    ///
    /// Mapped to OOXML `<a:dk2>` element.
    #[serde(default)]
    pub dk2: Option<String>,

    /// Light 2 — secondary light color (e.g., near-white background).
    ///
    /// Mapped to OOXML `<a:lt2>` element.
    #[serde(default)]
    pub lt2: Option<String>,

    /// Accent 1 — primary brand accent color.
    ///
    /// Mapped to OOXML `<a:accent1>` element.
    #[serde(default)]
    pub acc1: Option<String>,

    /// Accent 2 — secondary accent color.
    ///
    /// Mapped to OOXML `<a:accent2>` element.
    #[serde(default)]
    pub acc2: Option<String>,

    /// Accent 3 — tertiary accent color.
    ///
    /// Mapped to OOXML `<a:accent3>` element.
    #[serde(default)]
    pub acc3: Option<String>,

    /// Accent 4 — quaternary accent color.
    ///
    /// Mapped to OOXML `<a:accent4>` element.
    #[serde(default)]
    pub acc4: Option<String>,

    /// Accent 5 — quinary accent color.
    ///
    /// Mapped to OOXML `<a:accent5>` element.
    #[serde(default)]
    pub acc5: Option<String>,

    /// Accent 6 — senary accent color.
    ///
    /// Mapped to OOXML `<a:accent6>` element.
    #[serde(default)]
    pub acc6: Option<String>,

    /// Hyperlink color.
    ///
    /// Mapped to OOXML `<a:hlink>` element.
    #[serde(default)]
    pub hlink: Option<String>,

    /// Followed hyperlink color.
    ///
    /// TOML field name `fol_hlink` maps to OOXML `folHlink` via serde rename.
    #[serde(default, rename = "fol_hlink")]
    pub fol_hlink: Option<String>,
}

impl ColorConfig {
    /// Return all 12 slots as an array of `Option<&str>` in ECMA-376 order.
    ///
    /// Order: `[dk1, lt1, dk2, lt2, acc1, acc2, acc3, acc4, acc5, acc6, hlink, fol_hlink]`.
    #[must_use]
    pub fn as_slot_array(&self) -> [Option<&str>; 12] {
        [
            self.dk1.as_deref(),
            self.lt1.as_deref(),
            self.dk2.as_deref(),
            self.lt2.as_deref(),
            self.acc1.as_deref(),
            self.acc2.as_deref(),
            self.acc3.as_deref(),
            self.acc4.as_deref(),
            self.acc5.as_deref(),
            self.acc6.as_deref(),
            self.hlink.as_deref(),
            self.fol_hlink.as_deref(),
        ]
    }
}

// ─── [fonts] ─────────────────────────────────────────────────────────────────

/// The `[fonts]` section of `brand.toml`.
///
/// Specifies heading and body font typeface names. Both default to `"Calibri"`
/// when absent (BC-2.01.002 postcondition — `[fonts]` optional with fallbacks).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontConfig {
    /// Heading font typeface name (maps to OOXML `<a:majorFont>`).
    ///
    /// Fallback: `"Calibri"`.
    #[serde(default = "default_heading_font")]
    pub heading: String,

    /// Body font typeface name (maps to OOXML `<a:minorFont>`).
    ///
    /// Fallback: `"Calibri"`.
    #[serde(default = "default_body_font")]
    pub body: String,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            heading: default_heading_font(),
            body: default_body_font(),
        }
    }
}

/// Default heading font fallback.
fn default_heading_font() -> String {
    "Calibri".to_owned()
}

/// Default body font fallback.
fn default_body_font() -> String {
    "Calibri".to_owned()
}

// ─── [logo] ──────────────────────────────────────────────────────────────────

/// The `[logo]` section of `brand.toml`.
///
/// The `path` field is required for synthesized brands (AC-004). If absent,
/// `BrandSynthesizer::synthesize` returns `BrandError::LogoRequired`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogoConfig {
    /// File path to the logo image (e.g., `"brand.assets/logo.png"`).
    ///
    /// Relative paths are resolved from the directory containing `brand.toml`.
    pub path: String,
}

// ─── [footer] ────────────────────────────────────────────────────────────────

/// The `[footer]` section of `brand.toml`.
///
/// Controls the footer text and visibility flags applied to all slide layouts.
/// All fields are optional with documented defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FooterConfig {
    /// Footer text string (e.g., `"Confidential"`).
    ///
    /// Default: `""` (empty — no footer text shown).
    #[serde(default = "default_footer_text")]
    pub text: String,

    /// Whether to show the slide number in the footer.
    ///
    /// Default: `true`.
    #[serde(default = "default_show_slide_number")]
    pub show_slide_number: bool,

    /// Whether to show the date in the footer.
    ///
    /// Default: `false`.
    #[serde(default)]
    pub show_date: bool,
}

impl Default for FooterConfig {
    fn default() -> Self {
        Self {
            text: default_footer_text(),
            show_slide_number: default_show_slide_number(),
            show_date: false,
        }
    }
}

/// Default footer text (empty string).
fn default_footer_text() -> String {
    String::new()
}

/// Default for `show_slide_number` (true).
fn default_show_slide_number() -> bool {
    true
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// BC-2.01.002 — `BrandConfig` deserializes from a complete brand.toml.
    #[test]
    fn test_bc_2_01_002_deserializes_complete_brand_toml() {
        let toml_str = concat!(
            "[colors]\n",
            "dk1 = \"#1F2937\"\n",
            "lt1 = \"#FFFFFF\"\n",
            "dk2 = \"#374151\"\n",
            "lt2 = \"#F9FAFB\"\n",
            "acc1 = \"#3B82F6\"\n",
            "acc2 = \"#10B981\"\n",
            "acc3 = \"#F59E0B\"\n",
            "acc4 = \"#EF4444\"\n",
            "acc5 = \"#8B5CF6\"\n",
            "acc6 = \"#EC4899\"\n",
            "hlink = \"#2563EB\"\n",
            "fol_hlink = \"#1D4ED8\"\n",
            "\n",
            "[fonts]\n",
            "heading = \"Aptos Display\"\n",
            "body = \"Aptos\"\n",
            "\n",
            "[logo]\n",
            "path = \"brand.assets/logo.png\"\n",
            "\n",
            "[footer]\n",
            "text = \"Confidential\"\n",
            "show_slide_number = true\n",
            "show_date = false\n",
        );
        let config: BrandConfig = toml::from_str(toml_str).expect("should parse");
        assert_eq!(config.colors.dk1.as_deref(), Some("#1F2937"));
        assert_eq!(config.colors.acc1.as_deref(), Some("#3B82F6"));
        assert_eq!(config.colors.fol_hlink.as_deref(), Some("#1D4ED8"));
        assert_eq!(config.fonts.heading.as_str(), "Aptos Display");
        assert_eq!(
            config.logo.as_ref().map(|l| l.path.as_str()),
            Some("brand.assets/logo.png")
        );
        assert_eq!(config.footer.text.as_str(), "Confidential");
        assert!(config.footer.show_slide_number);
        assert!(!config.footer.show_date);
    }

    /// BC-2.01.002 — empty `brand.toml` yields all-None colors and default fonts.
    #[test]
    fn test_bc_2_01_002_empty_brand_toml_yields_defaults() {
        let config: BrandConfig = toml::from_str("").expect("empty TOML should parse");
        assert!(config.colors.dk1.is_none(), "dk1 should be None");
        assert!(config.colors.acc1.is_none(), "acc1 should be None");
        assert!(config.logo.is_none(), "logo should be None");
    }

    /// BC-2.01.002 — `fol_hlink` TOML field maps correctly (serde rename).
    #[test]
    fn test_bc_2_01_002_fol_hlink_serde_rename() {
        let toml_str = "[colors]\nfol_hlink = \"#1D4ED8\"\n";
        let config: BrandConfig = toml::from_str(toml_str).expect("should parse");
        assert_eq!(config.colors.fol_hlink.as_deref(), Some("#1D4ED8"));
    }

    /// BC-2.01.002 — `FontConfig` falls back to "Calibri" when not specified.
    #[test]
    fn test_bc_2_01_002_font_defaults_to_calibri() {
        let config: BrandConfig = toml::from_str("").expect("should parse");
        assert_eq!(config.fonts.heading.as_str(), "Calibri");
        assert_eq!(config.fonts.body.as_str(), "Calibri");
    }

    /// BC-2.01.002 — `FooterConfig` defaults: `show_slide_number=true`, `show_date=false`.
    #[test]
    fn test_bc_2_01_002_footer_defaults() {
        let config: BrandConfig = toml::from_str("").expect("should parse");
        assert_eq!(config.footer.text.as_str(), "");
        assert!(
            config.footer.show_slide_number,
            "show_slide_number default is true"
        );
        assert!(!config.footer.show_date, "show_date default is false");
    }

    /// BC-2.01.002 — `ColorConfig::as_slot_array` API exists (stub check).
    #[test]
    fn test_bc_2_01_002_color_config_as_slot_array_exists() {
        // as_slot_array() is todo!() — verify the type exists and the method is callable.
        // This test will panic when the todo!() fires at runtime (Red Gate behavior).
        let _ = std::panic::catch_unwind(|| {
            let config = ColorConfig::default();
            let _ = config.as_slot_array();
        });
        // We only assert the method exists at compile time (covered by compilation).
    }

    // ─── New behavioral tests for Red Gate ────────────────────────────────────

    /// BC-2.01.002 — `parse_full_brand_toml`: all 12 color fields, fonts, logo, footer
    /// populated correctly from a complete brand.toml.
    #[test]
    fn test_bc_2_01_002_parse_full_brand_toml() {
        let toml_str = concat!(
            "[colors]\n",
            "dk1 = \"#1F2937\"\n",
            "lt1 = \"#FFFFFF\"\n",
            "dk2 = \"#374151\"\n",
            "lt2 = \"#F9FAFB\"\n",
            "acc1 = \"#3B82F6\"\n",
            "acc2 = \"#10B981\"\n",
            "acc3 = \"#F59E0B\"\n",
            "acc4 = \"#EF4444\"\n",
            "acc5 = \"#8B5CF6\"\n",
            "acc6 = \"#EC4899\"\n",
            "hlink = \"#2563EB\"\n",
            "fol_hlink = \"#1D4ED8\"\n",
            "\n",
            "[fonts]\n",
            "heading = \"Aptos Display\"\n",
            "body = \"Aptos\"\n",
            "\n",
            "[logo]\n",
            "path = \"brand.assets/logo.png\"\n",
            "\n",
            "[footer]\n",
            "text = \"Confidential\"\n",
            "show_slide_number = true\n",
            "show_date = false\n",
        );
        let config: BrandConfig = toml::from_str(toml_str).expect("should parse");
        // All 12 color slots are present
        assert_eq!(config.colors.dk1.as_deref(), Some("#1F2937"), "dk1");
        assert_eq!(config.colors.lt1.as_deref(), Some("#FFFFFF"), "lt1");
        assert_eq!(config.colors.dk2.as_deref(), Some("#374151"), "dk2");
        assert_eq!(config.colors.lt2.as_deref(), Some("#F9FAFB"), "lt2");
        assert_eq!(config.colors.acc1.as_deref(), Some("#3B82F6"), "acc1");
        assert_eq!(config.colors.acc2.as_deref(), Some("#10B981"), "acc2");
        assert_eq!(config.colors.acc3.as_deref(), Some("#F59E0B"), "acc3");
        assert_eq!(config.colors.acc4.as_deref(), Some("#EF4444"), "acc4");
        assert_eq!(config.colors.acc5.as_deref(), Some("#8B5CF6"), "acc5");
        assert_eq!(config.colors.acc6.as_deref(), Some("#EC4899"), "acc6");
        assert_eq!(config.colors.hlink.as_deref(), Some("#2563EB"), "hlink");
        assert_eq!(
            config.colors.fol_hlink.as_deref(),
            Some("#1D4ED8"),
            "fol_hlink"
        );
        // Fonts
        assert_eq!(config.fonts.heading.as_str(), "Aptos Display");
        assert_eq!(config.fonts.body.as_str(), "Aptos");
        // Logo
        assert_eq!(
            config.logo.as_ref().map(|l| l.path.as_str()),
            Some("brand.assets/logo.png")
        );
        // Footer
        assert_eq!(config.footer.text.as_str(), "Confidential");
        assert!(config.footer.show_slide_number);
        assert!(!config.footer.show_date);
    }

    /// BC-2.01.002 — `parse_minimal_brand_toml`: only `[colors] dk1` → other fields None/default.
    #[test]
    fn test_bc_2_01_002_parse_minimal_brand_toml() {
        let toml_str = "[colors]\ndk1 = \"#1F2937\"\n";
        let config: BrandConfig = toml::from_str(toml_str).expect("should parse");
        assert_eq!(
            config.colors.dk1.as_deref(),
            Some("#1F2937"),
            "dk1 must be set"
        );
        // All other color slots are absent
        assert!(config.colors.lt1.is_none(), "lt1 should be None");
        assert!(config.colors.acc1.is_none(), "acc1 should be None");
        assert!(config.colors.hlink.is_none(), "hlink should be None");
        // Fonts fall back to defaults
        assert_eq!(config.fonts.heading.as_str(), "Calibri");
        assert_eq!(config.fonts.body.as_str(), "Calibri");
        // No logo
        assert!(config.logo.is_none(), "logo should be None when absent");
    }

    /// BC-2.01.002 — `as_slot_array` returns the 12 slots in ECMA-376 order.
    #[test]
    fn test_bc_2_01_002_color_config_as_slot_array_returns_12_in_order() {
        let toml_str = concat!(
            "[colors]\n",
            "dk1 = \"#1F2937\"\n",
            "lt1 = \"#FFFFFF\"\n",
            "dk2 = \"#374151\"\n",
            "lt2 = \"#F9FAFB\"\n",
            "acc1 = \"#3B82F6\"\n",
            "acc2 = \"#10B981\"\n",
            "acc3 = \"#F59E0B\"\n",
            "acc4 = \"#EF4444\"\n",
            "acc5 = \"#8B5CF6\"\n",
            "acc6 = \"#EC4899\"\n",
            "hlink = \"#2563EB\"\n",
            "fol_hlink = \"#1D4ED8\"\n",
        );
        let config: BrandConfig = toml::from_str(toml_str).expect("should parse");
        // as_slot_array() is todo!() — will panic (Red Gate)
        let slots = config.colors.as_slot_array();
        // ECMA-376 order: dk1, lt1, dk2, lt2, acc1..acc6, hlink, fol_hlink
        assert_eq!(slots[0], Some("#1F2937"), "index 0 = dk1");
        assert_eq!(slots[1], Some("#FFFFFF"), "index 1 = lt1");
        assert_eq!(slots[4], Some("#3B82F6"), "index 4 = acc1");
        assert_eq!(slots[11], Some("#1D4ED8"), "index 11 = fol_hlink");
    }

    /// BC-2.01.002 — `default_minimal()` constructs a `BrandConfig` usable by tests.
    #[test]
    fn test_bc_2_01_002_default_minimal_is_callable() {
        let config = BrandConfig::default_minimal();
        // After implementation: acc1 = "#3B82F6", logo = Some(...), dk1 set
        assert!(
            config.colors.acc1.is_some(),
            "default_minimal must have acc1"
        );
        assert!(
            config.logo.is_some(),
            "default_minimal must have a logo path"
        );
    }
}
