//! DOCX pipeline E2E tests — STORY-050 AC-002, AC-008 (DOCX portion).
//!
//! ## Acceptance Criteria covered
//!
//! - **AC-002**: `build()` with DOCX format returns `Ok(output)` where `output.bytes`
//!   is a valid DOCX ZIP containing `[Content_Types].xml` and `word/document.xml`.
//! - **AC-008 (DOCX)**: the 3-slide fixture produces a DOCX with body paragraphs.

#![allow(clippy::unwrap_used)]

use crate::e2e::{BrandTmpDir, assert_zip_contains, fixture_source, open_zip};

/// AC-002: `build()` with DOCX format returns `Ok(BuildOutput)` where
/// `output.bytes` is a valid ZIP containing `[Content_Types].xml` and
/// `word/document.xml`.
///
/// Traceability: BC-5.02.001 postcondition 4.
#[test]
fn test_bc_5_02_001_ac002_docx_build_returns_ok_with_valid_zip() {
    let brand = BrandTmpDir::new("ac002_docx");
    let source = fixture_source("test-3slide.sf");
    let opts = brand.build_options("docx", false);

    let result = slideforge::build(&source, &opts);

    let output = result.unwrap_or_else(|e| {
        panic!(
            "AC-002: build() with DOCX format must return Ok; got Err: {e:?}\n\
             Diagnose: check which pipeline stage failed — particularly the DOCX exporter."
        )
    });

    assert!(
        !output.bytes.is_empty(),
        "AC-002: BuildOutput.bytes must be non-empty (actual DOCX content)"
    );
    assert_eq!(
        output.extension, "docx",
        "AC-002: BuildOutput.extension must be 'docx'"
    );

    let mut archive = open_zip(&output.bytes, "AC-002");
    assert_zip_contains(&mut archive, "[Content_Types].xml", "AC-002");
    assert_zip_contains(&mut archive, "word/document.xml", "AC-002");
}

/// AC-008 (DOCX): the 3-slide fixture produces a DOCX whose `word/document.xml`
/// contains at least one `<w:p>` paragraph element.
///
/// Traceability: BC-5.02.001 postcondition 4, STORY-050 AC-008.
#[test]
fn test_bc_5_02_001_ac008_docx_has_body_paragraphs() {
    let brand = BrandTmpDir::new("ac008_docx");
    let source = fixture_source("test-3slide.sf");
    let opts = brand.build_options("docx", false);

    let result = slideforge::build(&source, &opts);
    let output = result.unwrap_or_else(|e| {
        panic!(
            "AC-008 (DOCX): build() must return Ok for 3-slide fixture; \
             got Err: {e:?}"
        )
    });

    let mut archive = open_zip(&output.bytes, "AC-008-docx");
    let mut doc_xml_file = archive
        .by_name("word/document.xml")
        .unwrap_or_else(|e| panic!("AC-008 DOCX: word/document.xml not found in ZIP: {e}"));

    let mut doc_xml = String::new();
    std::io::Read::read_to_string(&mut doc_xml_file, &mut doc_xml)
        .unwrap_or_else(|e| panic!("AC-008 DOCX: cannot read word/document.xml: {e}"));

    assert!(
        doc_xml.contains("<w:p"),
        "AC-008 DOCX: word/document.xml must contain at least one <w:p> paragraph; \
         got a document with no paragraph content. \
         Check that the DOCX exporter serialises slide content to body paragraphs."
    );
}
