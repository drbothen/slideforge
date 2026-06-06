//! PDF pipeline E2E tests — STORY-050 AC-003, AC-008 (PDF portion).
//!
//! ## Acceptance Criteria covered
//!
//! - **AC-003**: `build()` with PDF format returns `Ok(output)` where `output.bytes`
//!   begins with `%PDF-`.
//! - **AC-008 (PDF)**: the 3-slide fixture produces a 3-page PDF.

#![allow(clippy::unwrap_used)]

use crate::e2e::{BrandTmpDir, fixture_source};

/// AC-003: `build()` with PDF format returns `Ok(BuildOutput)` where
/// `output.bytes` begins with `%PDF-`.
///
/// Traceability: BC-5.02.001 postcondition 4.
#[test]
fn test_bc_5_02_001_ac003_pdf_build_returns_ok_with_pdf_header() {
    let brand = BrandTmpDir::new("ac003_pdf");
    let source = fixture_source("test-3slide.sf");
    let opts = brand.build_options("pdf", false);

    let result = slideforge::build(&source, &opts);

    let output = result.unwrap_or_else(|e| {
        panic!(
            "AC-003: build() with PDF format must return Ok; got Err: {e:?}\n\
             Diagnose: check which pipeline stage failed — particularly the PDF exporter."
        )
    });

    assert!(
        !output.bytes.is_empty(),
        "AC-003: BuildOutput.bytes must be non-empty"
    );
    assert_eq!(
        output.extension, "pdf",
        "AC-003: BuildOutput.extension must be 'pdf'"
    );

    let header = &output.bytes[..output.bytes.len().min(8)];
    assert!(
        header.starts_with(b"%PDF-"),
        "AC-003: PDF output must begin with '%PDF-' per ISO 32000-1 §7.5.2; \
         got first 8 bytes: {header:?}. \
         Check that the PDF exporter emits a valid PDF header."
    );
}

/// AC-003 + AC-008 (PDF): the 3-slide fixture produces a 3-page PDF.
///
/// Page count verified by counting `/Type /Page\n` (space-separated) or
/// `/Type/Page\n` entries in the raw PDF bytes — one per page object.
///
/// Traceability: BC-5.02.001 postcondition 4, STORY-050 AC-003, AC-008.
#[test]
fn test_bc_5_02_001_ac008_pdf_page_count_matches_slide_count() {
    let brand = BrandTmpDir::new("ac008_pdf");
    let source = fixture_source("test-3slide.sf");
    let opts = brand.build_options("pdf", false);

    let result = slideforge::build(&source, &opts);
    let output = result.unwrap_or_else(|e| {
        panic!(
            "AC-008 (PDF): build() must return Ok for 3-slide fixture; \
             got Err: {e:?}"
        )
    });

    // Count `/Type /Page` occurrences (with trailing newline or space) in the raw
    // PDF bytes. Two patterns cover the standard PDF space-vs-no-space variants.
    let pdf = &output.bytes;
    let count_needle = |needle: &[u8]| pdf.windows(needle.len()).filter(|w| *w == needle).count();

    let page_count = count_needle(b"/Type /Page\n")
        + count_needle(b"/Type /Page\r")
        + count_needle(b"/Type /Page ")
        + count_needle(b"/Type/Page\n")
        + count_needle(b"/Type/Page\r")
        + count_needle(b"/Type/Page ");

    assert_eq!(
        page_count, 3,
        "AC-008 PDF: 3-slide fixture must produce 3 pages; found {page_count} '/Type /Page' \
         occurrences in PDF. \
         If this fails: the PDF exporter may not be producing one page per slide, \
         or the page-count heuristic needs adjustment."
    );
}
