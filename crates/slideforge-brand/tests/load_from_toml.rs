//! Integration test for [`slideforge_brand::synthesizer::BrandSynthesizer::load_from_toml`].
//!
//! Exercises the full end-to-end path: on-disk TOML file → parse → synthesize →
//! [`BrandTemplate`] with 31 layouts and populated logo.
//!
//! This is the integration test required by F-PASS2-L1.

use std::io::Write as _;

use slideforge_brand::synthesizer::BrandSynthesizer;

/// F-PASS2-L1 — `load_from_toml` with a full on-disk fixture returns a valid
/// [`BrandTemplate`] with 31 layouts and populated color slots.
#[test]
fn test_load_brand_toml_end_to_end() {
    let mut tmp = tempfile::NamedTempFile::new().expect("tempfile must be created");
    writeln!(
        tmp,
        r##"
[colors]
dk1 = "#1F2937"
lt1 = "#FFFFFF"
dk2 = "#374151"
lt2 = "#F9FAFB"
acc1 = "#3B82F6"
acc2 = "#10B981"
acc3 = "#F59E0B"
acc4 = "#EF4444"
acc5 = "#8B5CF6"
acc6 = "#EC4899"
hlink = "#2563EB"
fol_hlink = "#1D4ED8"

[logo]
path = "logo.png"

[fonts]
heading = "Calibri"
body = "Calibri"

[footer]
text = "Confidential"
show_slide_number = true
show_date = false
"##
    )
    .expect("write to tempfile must succeed");

    let path = tmp
        .path()
        .to_str()
        .expect("tempfile path must be valid UTF-8");
    let (template, warnings) =
        BrandSynthesizer::load_from_toml(path).expect("load_from_toml must succeed for valid TOML");

    // All 12 colors declared → zero warnings
    assert_eq!(
        warnings.len(),
        0,
        "no MissingColorSlot warnings expected when all 12 colors declared"
    );
    // 31 layouts
    assert_eq!(template.layouts.len(), 31, "must have 31 layouts");
    // Logo is present (deferred path, not loaded bytes — synthesis is pure)
    assert!(
        template.logo.is_some(),
        "logo must be Some when [logo].path is declared"
    );
    // Color slots all resolved
    for (i, slot) in template.colors.iter().enumerate() {
        assert!(
            slot.is_resolved(),
            "color slot {i} ('{}') must be resolved",
            slot.name
        );
    }
    // Footer text preserved
    assert_eq!(
        template.footer_text.as_deref(),
        Some("Confidential"),
        "footer text must match [footer].text"
    );
}

/// F-PASS2-L1 — `load_from_toml` with a missing file returns `TomlReadError`.
#[test]
fn test_load_brand_toml_missing_file_returns_error() {
    let result = BrandSynthesizer::load_from_toml("/nonexistent/path/brand.toml");
    assert!(
        result.is_err(),
        "load_from_toml must fail for a missing file"
    );
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("E-BRD-001"),
        "error must be TomlReadError (E-BRD-001), got: {msg}"
    );
}

/// F-PASS2-L1 — `load_from_toml` with invalid TOML content returns `TomlParseError`.
#[test]
fn test_load_brand_toml_invalid_toml_returns_parse_error() {
    let mut tmp = tempfile::NamedTempFile::new().expect("tempfile must be created");
    writeln!(tmp, "not valid toml {{ }} [[[").expect("write must succeed");

    let path = tmp
        .path()
        .to_str()
        .expect("tempfile path must be valid UTF-8");
    let result = BrandSynthesizer::load_from_toml(path);
    assert!(result.is_err(), "load_from_toml must fail for invalid TOML");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("E-BRD-002"),
        "error must be TomlParseError (E-BRD-002), got: {msg}"
    );
}
