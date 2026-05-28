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
    // Use a temp directory so both the brand.toml and the logo file can coexist.
    let tmp_dir = tempfile::tempdir().expect("tempdir must be created");
    let brand_toml_path = tmp_dir.path().join("brand.toml");
    let logo_path = tmp_dir.path().join("logo.png");

    // Write a minimal 1×1 PNG (smallest valid PNG header) as the logo fixture.
    // EC-005: load_from_toml must not fail when the logo file exists.
    std::fs::write(&logo_path, b"\x89PNG\r\n\x1a\n").expect("logo fixture write must succeed");

    let mut brand_toml =
        std::fs::File::create(&brand_toml_path).expect("brand.toml create must succeed");
    writeln!(
        brand_toml,
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
    .expect("write to brand.toml must succeed");

    let path = brand_toml_path
        .to_str()
        .expect("brand.toml path must be valid UTF-8");
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
    let err = result.expect_err("load_from_toml must fail for a missing file");
    let msg = err.to_string();
    assert!(
        msg.contains("E-BRD-001"),
        "error must be TomlReadError (E-BRD-001), got: {msg}"
    );
}

/// F-PASS3-OBS1 — `load_from_toml` with a TOML that has no `[colors]` section
/// yields 12 `MissingColorSlot` warnings (one per slot).
///
/// This documents that `load_from_toml` propagates synthesis warnings through
/// its return tuple, satisfying AC-005 for the TOML loading path.
#[test]
fn test_load_brand_toml_empty_colors_section_yields_12_warnings() {
    let tmp_dir = tempfile::tempdir().expect("tempdir must be created");
    let brand_toml_path = tmp_dir.path().join("brand.toml");
    let logo_path = tmp_dir.path().join("logo.png");

    // Write a dummy logo file so EC-005 check passes.
    std::fs::write(&logo_path, b"\x89PNG\r\n\x1a\n").expect("logo fixture write must succeed");

    let mut tmp = std::fs::File::create(&brand_toml_path).expect("brand.toml create must succeed");
    std::io::Write::write_all(
        &mut tmp,
        br#"
[logo]
path = "logo.png"
"#,
    )
    .expect("write to tempfile must succeed");

    let path = brand_toml_path
        .to_str()
        .expect("brand.toml path must be valid UTF-8");
    let (template, warnings) = BrandSynthesizer::load_from_toml(path)
        .expect("load_from_toml must succeed even with missing colors");

    assert_eq!(
        warnings.len(),
        12,
        "exactly 12 MissingColorSlot warnings expected when [colors] section is absent, got {}: {:?}",
        warnings.len(),
        warnings
    );
    // Template must still have 31 layouts (synthesis always generates them)
    assert_eq!(
        template.layouts.len(),
        31,
        "must have 31 layouts even when colors are inferred"
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
    let msg = result.expect_err("load_from_toml must fail for invalid TOML").to_string();
    assert!(
        msg.contains("E-BRD-002"),
        "error must be TomlParseError (E-BRD-002), got: {msg}"
    );
}

/// EC-005 — `load_from_toml` returns `FileNotFound` when the declared logo path
/// does not exist on the filesystem.
///
/// This test validates BC-2.01.002 EC-005 (logo file not found). The logo path
/// is resolved relative to the directory containing `brand.toml`.
#[test]
fn test_load_brand_toml_missing_logo_returns_file_not_found() {
    let tmp_dir = tempfile::tempdir().expect("tempdir must be created");
    let brand_toml_path = tmp_dir.path().join("brand.toml");

    // Write a brand.toml that references a logo file that does NOT exist.
    std::fs::write(
        &brand_toml_path,
        r##"
[colors]
dk1 = "#1F2937"
acc1 = "#3B82F6"

[logo]
path = "nonexistent_logo.png"

[fonts]
heading = "Calibri"
body = "Calibri"
"##,
    )
    .expect("brand.toml write must succeed");

    let path = brand_toml_path
        .to_str()
        .expect("brand.toml path must be valid UTF-8");
    let result = BrandSynthesizer::load_from_toml(path);

    assert!(
        result.is_err(),
        "EC-005: load_from_toml must fail when the logo file does not exist"
    );
    let err = result.expect_err("EC-005: load_from_toml must fail when logo file does not exist");
    let msg = err.to_string();
    assert!(
        msg.contains("E-BRD-001"),
        "EC-005: error must be FileNotFound (E-BRD-001), got: {msg}"
    );
    assert!(
        msg.contains("nonexistent_logo.png"),
        "EC-005: error message must contain the missing logo path, got: {msg}"
    );
}
