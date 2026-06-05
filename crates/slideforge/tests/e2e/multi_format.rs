//! Multi-format E2E tests — STORY-050 AC-008 (cross-format), EC-005.
//!
//! ## Acceptance Criteria covered
//!
//! - **AC-008**: the same fixture produces valid output in PPTX, DOCX, and PDF.
//! - **EC-005**: all 3 formats can be built in sequence from the same fixture.
//!
//! ## RECONCILIATION FLAG
//!
//! The spec mentions `BuildOptions::all_formats()` for a single-call multi-format
//! API. This method does NOT exist. Three separate `build()` calls are used.
//!
//! ## Orchestrator decision required
//!
//! Should `BuildOptions::all_formats()` (returning `Vec<BuildOutput>`) be added?

#![allow(clippy::unwrap_used)]

use crate::e2e::{BrandTmpDir, assert_zip_contains, fixture_source, open_zip};

/// AC-008: the same 3-slide fixture produces structurally valid output in all
/// three formats via three separate `build()` calls.
///
/// Traceability: BC-5.02.001 postcondition 4, STORY-050 AC-008.
#[test]
fn test_bc_5_02_001_ac008_multi_format_all_three_produce_valid_output() {
    let source = fixture_source("test-3slide.sf");

    let pptx_brand = BrandTmpDir::new("ac008_multi_pptx");
    let pptx_result = slideforge::build(&source, &pptx_brand.build_options("pptx", false));
    let pptx_output = pptx_result.unwrap_or_else(|e| panic!("AC-008 PPTX: {e:?}"));

    let docx_brand = BrandTmpDir::new("ac008_multi_docx");
    let docx_result = slideforge::build(&source, &docx_brand.build_options("docx", false));
    let docx_output = docx_result.unwrap_or_else(|e| panic!("AC-008 DOCX: {e:?}"));

    let pdf_brand = BrandTmpDir::new("ac008_multi_pdf");
    let pdf_result = slideforge::build(&source, &pdf_brand.build_options("pdf", false));
    let pdf_output = pdf_result.unwrap_or_else(|e| panic!("AC-008 PDF: {e:?}"));

    // PPTX structural checks.
    assert_eq!(pptx_output.extension, "pptx");
    let mut pptx_archive = open_zip(&pptx_output.bytes, "AC-008-pptx");
    assert_zip_contains(&mut pptx_archive, "[Content_Types].xml", "AC-008-pptx");
    assert_zip_contains(&mut pptx_archive, "ppt/presentation.xml", "AC-008-pptx");

    // DOCX structural checks.
    assert_eq!(docx_output.extension, "docx");
    let mut docx_archive = open_zip(&docx_output.bytes, "AC-008-docx");
    assert_zip_contains(&mut docx_archive, "[Content_Types].xml", "AC-008-docx");
    assert_zip_contains(&mut docx_archive, "word/document.xml", "AC-008-docx");

    // PDF structural checks.
    assert_eq!(pdf_output.extension, "pdf");
    assert!(
        pdf_output.bytes.starts_with(b"%PDF-"),
        "AC-008: PDF output must start with '%PDF-'"
    );

    // PPTX and PDF have distinct file headers.
    assert_ne!(
        &pptx_output.bytes[..4],
        &pdf_output.bytes[..4],
        "AC-008: PPTX and PDF outputs must have distinct headers"
    );
}

/// EC-005: all 3 formats can be built in sequence from the same fixture.
///
/// This is the correct implementation of the spec's `BuildOptions::all_formats()`
/// scenario (the method does not exist — 3 separate calls are used).
///
/// Traceability: STORY-050 EC-005.
#[test]
fn test_bc_5_02_001_ec005_all_formats_built_from_single_fixture() {
    let source = fixture_source("test-3slide.sf");
    for format in &["pptx", "docx", "pdf"] {
        let brand = BrandTmpDir::new(&format!("ec005_{format}"));
        let opts = brand.build_options(format, false);
        let result = slideforge::build(&source, &opts);
        assert!(
            result.is_ok(),
            "EC-005: build() with format='{format}' must succeed; got: {result:?}",
        );
        let output = result.unwrap();
        assert!(
            !output.bytes.is_empty(),
            "EC-005: {format} output must be non-empty"
        );
    }
}

/// AC-008 (no state bleed): two sequential builds of the same format must
/// produce outputs with the same structure (no shared mutable state between calls).
#[test]
fn test_bc_5_02_001_ac008_sequential_builds_produce_independent_outputs() {
    let source = fixture_source("test-3slide.sf");

    let brand1 = BrandTmpDir::new("ac008_seq1");
    let result1 = slideforge::build(&source, &brand1.build_options("pptx", false));
    let output1 = result1.unwrap_or_else(|e| panic!("AC-008 seq1: {e:?}"));

    let brand2 = BrandTmpDir::new("ac008_seq2");
    let result2 = slideforge::build(&source, &brand2.build_options("pptx", false));
    let output2 = result2.unwrap_or_else(|e| panic!("AC-008 seq2: {e:?}"));

    // Both must produce PPTX with the same extension.
    assert_eq!(
        output1.extension, output2.extension,
        "AC-008: extensions must match"
    );
    // Both must produce valid ZIPs with the same required entries.
    let mut arch1 = open_zip(&output1.bytes, "AC-008-seq1");
    let mut arch2 = open_zip(&output2.bytes, "AC-008-seq2");
    assert_zip_contains(&mut arch1, "[Content_Types].xml", "AC-008-seq1");
    assert_zip_contains(&mut arch2, "[Content_Types].xml", "AC-008-seq2");
}
