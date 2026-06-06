//! PPTX pipeline E2E tests — STORY-050 AC-001, AC-008 (PPTX portion).
//!
//! ## Acceptance Criteria covered
//!
//! - **AC-001**: `build()` with PPTX format returns `Ok(output)` where `output.bytes`
//!   is a valid PPTX ZIP containing `[Content_Types].xml` and `ppt/presentation.xml`.
//! - **AC-008 (PPTX)**: the 3-slide fixture produces 3 slide XML entries in the ZIP.
//!
//! ## Architecture compliance
//!
//! All tests call `slideforge::build()` via the public API only.
//! No imports from `slideforge_pptx`, `slideforge_docx`, `slideforge_pdf`, etc.

#![allow(clippy::unwrap_used)] // integration tests — explicit panic on failure is correct

use crate::e2e::{BrandTmpDir, assert_zip_contains, fixture_source, open_zip};

/// AC-001: `build()` with PPTX format returns `Ok(BuildOutput)` where
/// `output.bytes` is a valid ZIP containing `[Content_Types].xml` and
/// `ppt/presentation.xml`.
///
/// Traceability: BC-5.02.001 postcondition 4.
#[test]
fn test_bc_5_02_001_ac001_pptx_build_returns_ok_with_valid_zip() {
    let brand = BrandTmpDir::new("ac001_pptx");
    let source = fixture_source("test-3slide.sf");
    let opts = brand.build_options("pptx", false);

    let result = slideforge::build(&source, &opts);

    let output = result.unwrap_or_else(|e| {
        panic!(
            "AC-001: build() with PPTX format must return Ok; got Err: {e:?}\n\
             Diagnose: check which pipeline stage failed (brand/parse/eval/validate/layout/export)."
        )
    });

    assert!(
        !output.bytes.is_empty(),
        "AC-001: BuildOutput.bytes must be non-empty (actual PPTX content)"
    );
    assert_eq!(
        output.extension, "pptx",
        "AC-001: BuildOutput.extension must be 'pptx'"
    );

    // Validate ZIP structure.
    let mut archive = open_zip(&output.bytes, "AC-001");
    assert_zip_contains(&mut archive, "[Content_Types].xml", "AC-001");
    assert_zip_contains(&mut archive, "ppt/presentation.xml", "AC-001");
}

/// AC-001 (extension variant): `BuildOptions::format = None` defaults to PPTX.
///
/// `build()` must produce a PPTX when `format` is `None` (the documented default).
/// Traceability: `BuildOptions::format` documentation: "If None, the pipeline defaults to 'pptx'."
#[test]
fn test_bc_5_02_001_ac001_pptx_default_format_when_none() {
    let brand = BrandTmpDir::new("ac001_pptx_default");
    let source = fixture_source("test-3slide.sf");
    let opts = slideforge::BuildOptions {
        brand_source: Some(slideforge_plugin_api::BrandSource::TomlFile(
            std::sync::Arc::from(brand.brand_toml_path.to_string_lossy().as_ref()),
        )),
        format: None, // must default to "pptx"
        strict: false,
    };

    let result = slideforge::build(&source, &opts);

    let output = result.unwrap_or_else(|e| {
        panic!(
            "AC-001 (default format): build() with format=None must default to PPTX; \
             got Err: {e:?}"
        )
    });
    assert_eq!(
        output.extension, "pptx",
        "AC-001: format=None must produce extension='pptx'"
    );
    let mut archive = open_zip(&output.bytes, "AC-001-default");
    assert_zip_contains(&mut archive, "[Content_Types].xml", "AC-001-default");
}

/// AC-008 (PPTX): the 3-slide fixture produces a PPTX with 3 slide XML entries.
///
/// Verified by counting `ppt/slides/slide*.xml` entries in the ZIP.
/// Traceability: BC-5.02.001 postcondition 4, STORY-050 AC-008.
#[test]
fn test_bc_5_02_001_ac008_pptx_slide_count_matches_fixture() {
    let brand = BrandTmpDir::new("ac008_pptx");
    let source = fixture_source("test-3slide.sf");
    let opts = brand.build_options("pptx", false);

    let result = slideforge::build(&source, &opts);
    let output = result.unwrap_or_else(|e| {
        panic!(
            "AC-008 (PPTX): build() must return Ok for 3-slide fixture; \
             got Err: {e:?}"
        )
    });

    let archive = open_zip(&output.bytes, "AC-008-pptx");
    let slide_count = archive
        .file_names()
        .filter(|name| {
            name.starts_with("ppt/slides/slide")
                && std::path::Path::new(name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
        })
        .count();

    assert_eq!(
        slide_count, 3,
        "AC-008: 3-slide fixture must produce exactly 3 slide XML entries in PPTX ZIP; \
         got {slide_count}. \
         If this fails: the pipeline may be producing fewer slides than the fixture defines."
    );
}
