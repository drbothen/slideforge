//! STORY-086 End-to-End Content Threading Tests.
//!
//! ## Acceptance Criteria covered
//!
//! - **AC-001**: PPTX title placeholder carries a non-empty `<a:r><a:t>` text run
//!   with text "My Title" (LESSON-13 positive content vector).
//! - **AC-002**: PDF byte stream contains text "Heading" and "Body paragraph text"
//!   (LESSON-13 positive content vector).
//! - **AC-003**: DOCX `word/document.xml` Heading1 run carries "Report Title"
//!   as a non-empty `<w:t>` text node (LESSON-13 positive content vector).
//! - **AC-004**: strict + chart WITH alt → `build()` returns `Ok`, PPTX carries
//!   `descr="Bar chart showing Q3 revenue by region"` (accessibility attribute).
//! - **AC-005**: strict + chart WITHOUT alt → `build()` returns
//!   `Err(BuildError::ValidationFailed)` with exactly one `E-A11-001` diagnostic.
//! - **AC-006**: `decorative: true` chart + `strict: true` → `build()` returns `Ok`
//!   (no E-A11-001 emitted).
//! - **AC-007**: bullets slide → PPTX contains ≥3 `<a:r>` text runs for 3 bullet items.
//! - **AC-018**: Wave 4 Gate 3 re-pass — 3-slide deck (title + content + chart-with-alt)
//!   builds cleanly under `strict: true`; all three formats produce non-empty output;
//!   PPTX contains title text run; no E-A11-001 emitted.
//!
//! ## Red Gate discipline (LESSON-13 / LESSON-14)
//!
//! All content-vector tests assert VISIBLE CONTENT (text runs, text nodes), not
//! merely structural validity. The stub `thread_fields_to_blocks` is a no-op:
//! `Slide.blocks = vec![]` → no `FrameContent::TextRun` frames → exporters produce
//! content-empty output → `<a:t>` text assertions FAIL → Red Gate confirmed.
//!
//! ## Architecture compliance (STORY-050)
//!
//! All tests call `slideforge::build()` via the public API only.
//! No imports from `slideforge_pptx`, `slideforge_docx`, `slideforge_pdf`, etc.

#![allow(clippy::unwrap_used)] // integration tests — explicit panic on failure is correct
#![allow(clippy::doc_markdown)] // test doc comments have unquoted identifiers (e.g. E-A11-001)
#![allow(clippy::uninlined_format_args)] // test error messages use named format args

use crate::e2e::{BrandTmpDir, fixture_source, open_zip};

// ─── AC-001: PPTX title placeholder carries "My Title" text run ───────────────

/// AC-001 / BC-1.16.001 postcondition 1 — PPTX `<a:t>` text node contains "My Title".
///
/// Red Gate: stub → `Slide.blocks = vec![]` → no TextRun frame → PPTX slide XML
/// contains no `<a:t>My Title</a:t>` → assertion FAILS.
///
/// After implementation: title ContentBlock → layout TextRun frame → PPTX `<a:t>`
/// carries "My Title" → assertion PASSES.
///
/// Traces: BC-1.16.001 postcondition 1; LESSON-13 positive content vector.
#[test]
fn test_bc_1_16_001_ac001_pptx_title_text_run_is_nonempty() {
    // AC-001: PPTX slide XML must contain a non-empty <a:r><a:t> with "My Title".
    // RED GATE: stub → no content threaded → <a:t> absent → FAILS.
    let brand = BrandTmpDir::new("s086_ac001_pptx");
    let source = fixture_source("story-086-title-slide.sf");
    let opts = brand.build_options("pptx", true); // strict: true per AC

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "AC-001: build() with title slide (strict=true) must return Ok; \
             got Err: {e:?}\n\
             Check: Stage 2b must populate blocks before strict validation runs."
        )
    });

    // PPTX is a ZIP — open and extract slide1.xml.
    let mut archive = open_zip(&output.bytes, "AC-001");
    let slide_xml = read_zip_entry(&mut archive, "ppt/slides/slide1.xml", "AC-001");

    // Assert the XML contains <a:t>My Title</a:t> — the title text run.
    // This assertion FAILS against the stub (no text threaded → no <a:t> run).
    assert!(
        slide_xml.contains("<a:t>My Title</a:t>") || slide_xml.contains(">My Title<"),
        "AC-001 Red Gate: PPTX slide1.xml must contain a text run with 'My Title'. \
         Stub is no-op → Slide.blocks=[] → no TextRun frame → no <a:t> → FAILS. \
         Content found in slide XML (first 500 chars): {:.500}",
        slide_xml
    );

    // Additionally assert the <a:t> element is non-empty (not just structurally present).
    // Searching for the literal text "My Title" already confirms non-emptiness.
    let a_t_count = slide_xml.matches("<a:t>").count();
    assert!(
        a_t_count > 0,
        "AC-001: slide1.xml must contain at least one <a:t> element (text run). \
         Got 0. Stub → no runs → FAILS until Stage 2b ships."
    );
}

// ─── AC-002: PDF contains "Heading" and "Body paragraph text" ────────────────

/// AC-002 / BC-1.16.001 postconditions 1 + 4 — PDF byte stream contains both
/// "Heading" and "Body paragraph text" as visible text content.
///
/// Red Gate: stub → `Slide.blocks = vec![]` → no text-bearing frames → PDF's
/// `draw_text_at_bbox` is never called for these strings → PDF bytes do NOT
/// contain the text → assertion FAILS.
///
/// Note: We search the raw PDF bytes for the text strings. PDF uses a content
/// stream encoding where text may appear as literal text in a `Tj` or `TJ` operator,
/// or within a compressed stream. We check raw bytes as a best-effort approach;
/// the krilla PDF engine writes text in a form readable as UTF-8 substrings.
///
/// Traces: BC-1.16.001 postconditions 1 and 4; LESSON-13 positive content vector.
#[test]
fn test_bc_1_16_001_ac002_pdf_contains_body_text_strings() {
    // AC-002: PDF must contain visible text "Heading" and "Body paragraph text".
    // RED GATE: stub → no TextRun frames → text not drawn → PDF bytes lack strings → FAILS.
    let brand = BrandTmpDir::new("s086_ac002_pdf");
    let source = fixture_source("story-086-content-slide.sf");
    let opts = brand.build_options("pdf", false);

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "AC-002: build() with content slide (pdf, strict=false) must return Ok; \
             got Err: {e:?}"
        )
    });

    assert!(
        !output.bytes.is_empty(),
        "AC-002: PDF bytes must be non-empty"
    );
    assert_eq!(output.extension, "pdf", "AC-002: extension must be 'pdf'");

    // Search raw bytes for the text strings.
    // After Stage 2b: krilla draws "Heading" and "Body paragraph text" via draw_text_at_bbox.
    // The text appears in the PDF content stream.
    let pdf_str = String::from_utf8_lossy(&output.bytes);

    assert!(
        pdf_str.contains("Heading") || output.bytes.windows(7).any(|w| w == b"Heading"),
        "AC-002 Red Gate: PDF must contain text 'Heading'. \
         Stub → Slide.blocks=[] → no draw_text_at_bbox call → absent → FAILS. \
         BC-1.16.001 postcondition 1 (title ContentBlock)."
    );

    assert!(
        pdf_str.contains("Body paragraph text")
            || output
                .bytes
                .windows(19)
                .any(|w| w == b"Body paragraph text"),
        "AC-002 Red Gate: PDF must contain text 'Body paragraph text'. \
         Stub → Slide.blocks=[] → no draw_text_at_bbox call → absent → FAILS. \
         BC-1.16.001 postcondition 4 (body ContentBlock)."
    );
}

// ─── AC-003: DOCX Heading1 run carries "Report Title" ────────────────────────

/// AC-003 / BC-1.16.001 postcondition 1 — DOCX `word/document.xml` contains
/// a non-empty Heading1 paragraph with text "Report Title".
///
/// Red Gate: stub → `Slide.blocks = vec![]` → no `FrameContent::Title` produced →
/// DOCX exporter reads empty title → `<w:t>` node is empty or absent → FAILS.
///
/// The DOCX exporter (`document_body.rs`) reads the slide title from
/// `FrameContent::Title` frames. After Stage 2b, the implementer must ensure
/// that a title `ContentBlock` ultimately populates a `FrameContent::Title` frame
/// so the DOCX exporter can emit a non-empty Heading1 paragraph.
///
/// Traces: BC-1.16.001 postcondition 1; LESSON-13 positive content vector.
#[test]
fn test_bc_1_16_001_ac003_docx_heading1_run_carries_title_text() {
    // AC-003: DOCX word/document.xml must contain a Heading1 paragraph with "Report Title".
    // RED GATE: stub → no FrameContent::Title populated → DOCX exporter gets title="" →
    // Heading1 paragraph has empty <w:t> or absent text → assertion FAILS.
    let brand = BrandTmpDir::new("s086_ac003_docx");
    // Use a fixture with title "Report Title" (the slide has slide type "title").
    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "\n",
        "slide title:\n",
        "  title \"Report Title\"\n",
    );
    let opts = brand.build_options("docx", false);

    let output = slideforge::build(source, &opts).unwrap_or_else(|e| {
        panic!(
            "AC-003: build() with title slide (docx, strict=false) must return Ok; \
             got Err: {e:?}"
        )
    });

    let mut archive = open_zip(&output.bytes, "AC-003");
    let doc_xml = read_zip_entry(&mut archive, "word/document.xml", "AC-003");

    // Assert word/document.xml contains "Report Title" as a non-empty text node.
    // The DOCX exporter emits <w:t>Report Title</w:t> inside a Heading1 paragraph.
    // RED GATE: stub → title="" → <w:t></w:t> → "Report Title" absent → FAILS.
    assert!(
        doc_xml.contains("Report Title"),
        "AC-003 Red Gate: word/document.xml must contain text 'Report Title' in a Heading1 run. \
         Stub → no FrameContent::Title populated → DOCX exporter reads title=\"\" → \
         absent text → FAILS until Stage 2b ships. \
         BC-1.16.001 postcondition 1. \
         doc_xml (first 600 chars): {:.600}",
        doc_xml
    );
}

// ─── AC-004: strict + chart WITH alt → Ok, PPTX carries descr ────────────────

/// AC-004 / BC-1.16.001 postcondition 9 + alt-resolution rule postcondition 12 —
/// chart slide WITH `alt "..."` in strict mode builds Ok and PPTX carries the
/// accessibility description.
///
/// Red Gate: stub → `thread_fields_to_blocks` is no-op → chart `ContentBlock`
/// never produced → `thread_media_alt_into_frames` gets no `ContentBlock::Chart` →
/// frame retains structural placeholder `AltText::Decorative` → `validate_post_layout`
/// fires E-A11-001 (stub fires on Decorative) → `build()` returns
/// `Err(ValidationFailed)` in strict mode → unwrap panics → RED GATE.
///
/// After implementation: Stage 2b produces ContentBlock::Chart with
/// `alt = Some(Provided("Bar chart showing Q3 revenue by region"))` →
/// thread_media_alt_into_frames threads it into the frame → Provided alt →
/// no E-A11-001 → build Ok → PASSES.
///
/// Traces: BC-1.16.001 postconditions 9 + 12; BC-5.01.001 postcondition 1;
///         BC-5.02.001 postcondition 7; LESSON-13 positive content vector.
#[test]
fn test_bc_1_16_001_ac004_strict_chart_with_alt_build_ok_and_pptx_carries_descr() {
    // AC-004: strict + chart WITH alt → Ok + PPTX descr attribute.
    // RED GATE: stub is no-op → chart block never produced → Decorative placeholder
    //           retained → validate_post_layout fires E-A11-001 → Err → FAILS.
    let brand = BrandTmpDir::new("s086_ac004_chart_alt");
    let source = fixture_source("story-086-chart-with-alt.sf");
    let opts = brand.build_options("pptx", true); // strict: true

    // Must return Ok — if stub retains Decorative, strict validation fails.
    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "AC-004 Red Gate: build() with chart slide (alt provided, strict=true) must return \
             Ok. Got Err: {e:?}. Stub is no-op → chart ContentBlock not threaded → Decorative \
             placeholder retained → E-A11-001 fired → FAILS until Stage 2b + T5 ship."
        )
    });

    // Verify PPTX output is non-empty.
    assert!(
        !output.bytes.is_empty(),
        "AC-004: PPTX bytes must be non-empty"
    );

    // Open PPTX ZIP and check slide XML for the alt description.
    let mut archive = open_zip(&output.bytes, "AC-004");
    let slide_xml = read_zip_entry(&mut archive, "ppt/slides/slide1.xml", "AC-004");

    // The alt text "Bar chart showing Q3 revenue by region" must appear as a
    // descr="" attribute on the PPTX non-visual object (via AltTextEmbedder).
    assert!(
        slide_xml.contains("Bar chart showing Q3 revenue by region"),
        "AC-004: PPTX slide1.xml must carry the alt description \
         'Bar chart showing Q3 revenue by region'. \
         Got slide_xml (first 800 chars): {:.800}",
        slide_xml
    );
}

// ─── AC-005: strict + chart WITHOUT alt → Err(ValidationFailed) + E-A11-001 ──

/// AC-005 / BC-1.16.001 postcondition 12 — chart WITHOUT alt + strict mode →
/// `Err(BuildError::ValidationFailed)` with exactly one `E-A11-001` diagnostic.
///
/// Red Gate analysis:
/// - PRE-FIX (stub + current alt_text.rs): stub is no-op → no chart ContentBlock →
///   Decorative structural placeholder → validate_post_layout fires on Decorative →
///   `Err(ValidationFailed)` with E-A11-001 → assertion passes (false positive pass).
///
/// Wait — this test actually PASSES against the stub for the wrong reason (the stub
/// fires E-A11-001 on Decorative). This is a TRUE negative test that coincidentally
/// passes. However, after T5 (alt_text.rs fix where Decorative → valid), the stub
/// would STOP firing on Decorative. THEN: stub is no-op → no ContentBlock → no frame
/// overwrite → frame retains Decorative → but Decorative is now valid → no error →
/// `Ok` returned → THIS TEST FAILS.
///
/// So the Red Gate sequence is:
/// 1. PRE-T5 (stub + old alt_text.rs): test passes (wrong reason — Decorative fires)
/// 2. POST-T5 pre-T6 (stub + fixed alt_text.rs): test FAILS (Decorative is valid, no block)
/// 3. POST-T6 (Stage 2b implemented): ContentBlock::Chart with alt=None →
///    thread_media_alt_into_frames maps None → Unspecified → E-A11-001 fired → PASSES correctly
///
/// This is the CORRECT Red Gate: after T5 and before T6, this test is RED.
/// After both T5+T6, this test is GREEN for the right reason.
///
/// Traces: BC-1.16.001 postcondition 12; BC-5.01.001 postcondition 1;
///         BC-5.02.001 postcondition 7.
#[test]
fn test_bc_1_16_001_ac005_strict_chart_no_alt_returns_validation_error() {
    // AC-005: strict + chart WITHOUT alt → Err with exactly 1 E-A11-001.
    // Red Gate: passes against stub now (Decorative fires), fails after T5 (Decorative valid),
    // passes correctly after both T5+T6 (Unspecified fires).
    let brand = BrandTmpDir::new("s086_ac005_chart_no_alt");
    let source = fixture_source("story-086-chart-no-alt.sf");
    let opts = brand.build_options("pptx", true); // strict: true

    let result = slideforge::build(&source, &opts);

    // Must be an error.
    assert!(
        result.is_err(),
        "AC-005: build() with chart slide (no alt, strict=true) must return Err. \
         Got Ok. BC-1.16.001 postcondition 12: absent alt → ValidationFailed."
    );

    let err = result.unwrap_err();

    // Must be ValidationFailed with E-A11-001.
    match err {
        slideforge::error::BuildError::ValidationFailed {
            ref diagnostics, ..
        } => {
            let e_a11_count = diagnostics
                .iter()
                .filter(|d| d.code.as_ref() == "E-A11-001")
                .count();
            assert_eq!(
                e_a11_count, 1,
                "AC-005: ValidationFailed must contain exactly 1 E-A11-001 diagnostic \
                 (one chart, one missing alt). Got {e_a11_count} E-A11-001 in {diagnostics:?}. \
                 BC-5.01.001 postcondition 1; BC-5.02.001 postcondition 7."
            );
        },
        other => {
            panic!(
                "AC-005: build() must return BuildError::ValidationFailed; \
                 got other variant: {other:?}"
            );
        },
    }
}

// ─── AC-006: decorative chart → strict Ok, no E-A11-001 ──────────────────────

/// AC-006 / BC-1.16.001 postcondition 12 + BC-5.01.001 EC-007 — chart with
/// `decorative: true` in strict mode → `Ok(BuildOutput)`, no E-A11-001.
///
/// Red Gate: stub is no-op → no chart ContentBlock → Decorative structural placeholder
/// retained in regions.rs → validate_post_layout (stub) fires E-A11-001 on Decorative →
/// `Err(ValidationFailed)` → `unwrap_or_else` panics → RED GATE.
///
/// After T5 (alt_text.rs fix: Decorative → valid) AND T3 (regions.rs uses Unspecified):
/// The chart frame would carry Unspecified (from regions.rs fix) → E-A11-001 still fires →
/// still Err.
///
/// After T6 (Stage 2b implemented): ContentBlock::Chart with alt=Some(Decorative) produced
/// → thread_media_alt_into_frames threads Decorative into frame → AltText::Decorative on frame
/// → validate_post_layout sees Decorative → no error (after T5 fix) → Ok → PASSES.
///
/// Traces: BC-1.16.001 postcondition 12; BC-5.01.001 EC-007; BC-5.02.001 EC-009.
#[test]
fn test_bc_1_16_001_ac006_decorative_chart_strict_ok_no_e_a11_001() {
    // AC-006: decorative: true chart + strict: true → Ok (no E-A11-001).
    // RED GATE: stub → Decorative placeholder retained → stub fires on Decorative → Err → FAILS.
    let brand = BrandTmpDir::new("s086_ac006_decorative_chart");
    let source = fixture_source("story-086-chart-decorative.sf");
    let opts = brand.build_options("pptx", true); // strict: true

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "AC-006 Red Gate: build() with decorative chart (strict=true) must return Ok. \
             Got Err: {e:?}. Stub fires E-A11-001 on Decorative placeholder → FAILS until \
             T5 (alt_text.rs fix) + T6 (Stage 2b threads Decorative into frame) ship. \
             BC-5.01.001 EC-007; BC-5.02.001 EC-009."
        )
    });

    assert!(
        !output.bytes.is_empty(),
        "AC-006: PPTX bytes must be non-empty for decorative chart build"
    );

    // Confirm no E-A11-001 was emitted (by checking build succeeded — validated above).
    assert_eq!(
        output.extension, "pptx",
        "AC-006: output extension must be 'pptx'"
    );
}

// ─── AC-007: bullets slide → ≥3 text runs in PPTX ───────────────────────────

/// AC-007 / BC-1.16.001 postcondition 7 — bullets slide produces ≥3 `<a:r>` text
/// runs in PPTX output.
///
/// Red Gate: stub → `Slide.blocks = vec![]` → no ContentBlock::Bullets → no
/// BulletItem frames → no `<a:r>` runs in slide XML → `a_r_count < 3` → FAILS.
///
/// After Stage 2b: bullets field → ContentBlock::Bullets([A, B, C]) → layout
/// produces 3 TextRun frames → PPTX serializer emits 3 `<a:r><a:t>` runs → PASSES.
///
/// Traces: BC-1.16.001 postcondition 7; BC-1.16.001 invariant 2;
///         LESSON-13 positive content vector (bullet text assertions).
#[test]
fn test_bc_1_16_001_ac007_bullets_slide_produces_ge3_text_runs_in_pptx() {
    // AC-007: bullets slide → ≥3 <a:r> runs in PPTX slide XML.
    // RED GATE: stub → no ContentBlock::Bullets → no runs → assertion fails.
    let brand = BrandTmpDir::new("s086_ac007_bullets");
    let source = fixture_source("story-086-bullets-slide.sf");
    let opts = brand.build_options("pptx", false);

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!("AC-007: build() with bullets slide must return Ok; got Err: {e:?}")
    });

    let mut archive = open_zip(&output.bytes, "AC-007");
    let slide_xml = read_zip_entry(&mut archive, "ppt/slides/slide1.xml", "AC-007");

    // Count <a:r> run elements — each bullet item produces one TextRun frame
    // which serializes as one <a:r><a:t>item text</a:t></a:r>.
    let a_r_count = slide_xml.matches("<a:r>").count();

    // Must have at least 3 runs (one per bullet item: "Item A", "Item B", "Item C").
    // FAILS against stub (no runs → a_r_count == 0).
    assert!(
        a_r_count >= 3,
        "AC-007 Red Gate: PPTX slide1.xml must contain ≥3 <a:r> text runs for 3 bullet items. \
         Got {a_r_count} runs. Stub → Slide.blocks=[] → no TextRun frames → 0 runs → FAILS. \
         BC-1.16.001 postcondition 7."
    );

    // Also assert each bullet text appears in the XML.
    assert!(
        slide_xml.contains("Item A"),
        "AC-007: slide XML must contain bullet text 'Item A'; got {a_r_count} runs"
    );
    assert!(
        slide_xml.contains("Item B"),
        "AC-007: slide XML must contain bullet text 'Item B'"
    );
    assert!(
        slide_xml.contains("Item C"),
        "AC-007: slide XML must contain bullet text 'Item C'"
    );
}

// ─── AC-018: Wave 4 Gate 3 re-pass ───────────────────────────────────────────

/// AC-018 / STORY-086 Wave 4 Gate 3 re-pass — 3-slide deck (title + content +
/// chart-with-alt) builds cleanly under `strict: true`; all three formats
/// produce non-empty output with visible content; no E-A11-001.
///
/// Red Gate: stub → Stage 2b is no-op → chart ContentBlock never created →
/// Decorative structural placeholder retained → strict validation fires E-A11-001
/// on Decorative (current stub behavior) → `Err(ValidationFailed)` → FAILS.
///
/// After implementation: Stage 2b threads chart with Provided alt → Unspecified
/// placeholder overwritten → no false-positive → all three formats build → PASSES.
///
/// Traces: BC-5.02.001 postcondition 5; BC-5.01.001 postcondition 1 (no false-positive);
///         LESSON-13/LESSON-14 positive content vectors.
#[test]
fn test_bc_5_02_001_ac018_wave4_gate3_repass_pptx_strict_ok_nonempty() {
    // AC-018: Wave 4 Gate 3 re-pass — PPTX format.
    // RED GATE: stub → E-A11-001 on Decorative placeholder → Err → FAILS.
    let brand = BrandTmpDir::new("s086_ac018_pptx");
    let source = fixture_source("story-086-wave4-gate3.sf");
    let opts = brand.build_options("pptx", true); // strict: true

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "AC-018 (PPTX) Red Gate: 3-slide fixture (strict=true) must build Ok. \
             Got Err: {e:?}. Stage 2b stub is no-op → chart alt not threaded → \
             Decorative placeholder retained → E-A11-001 fired → FAILS."
        )
    });

    assert!(
        !output.bytes.is_empty(),
        "AC-018 PPTX: bytes must be non-empty"
    );

    // Assert PPTX title text run is present.
    let mut archive = open_zip(&output.bytes, "AC-018-pptx");
    let slide_xml = read_zip_entry(&mut archive, "ppt/slides/slide1.xml", "AC-018-pptx");
    assert!(
        slide_xml.contains("Annual Review"),
        "AC-018 PPTX: slide1.xml must contain title text 'Annual Review'. \
         Stub → no TextRun for title → absent → FAILS."
    );
}

#[test]
fn test_bc_5_02_001_ac018_wave4_gate3_repass_pdf_strict_ok_nonempty() {
    // AC-018: Wave 4 Gate 3 re-pass — PDF format.
    let brand = BrandTmpDir::new("s086_ac018_pdf");
    let source = fixture_source("story-086-wave4-gate3.sf");
    let opts = brand.build_options("pdf", true); // strict: true

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "AC-018 (PDF) Red Gate: 3-slide fixture (strict=true, pdf) must build Ok. \
             Got Err: {e:?}"
        )
    });

    assert!(
        !output.bytes.is_empty(),
        "AC-018 PDF: bytes must be non-empty"
    );
    assert_eq!(
        output.extension, "pdf",
        "AC-018 PDF: extension must be 'pdf'"
    );

    // Assert PDF contains title text.
    let pdf_str = String::from_utf8_lossy(&output.bytes);
    assert!(
        pdf_str.contains("Annual Review")
            || output.bytes.windows(13).any(|w| w == b"Annual Review"),
        "AC-018 PDF: PDF must contain text 'Annual Review'. \
         Stub → no draw_text_at_bbox call for title → absent → FAILS."
    );
}

#[test]
fn test_bc_5_02_001_ac018_wave4_gate3_repass_docx_strict_ok_nonempty() {
    // AC-018: Wave 4 Gate 3 re-pass — DOCX format.
    let brand = BrandTmpDir::new("s086_ac018_docx");
    let source = fixture_source("story-086-wave4-gate3.sf");
    let opts = brand.build_options("docx", true); // strict: true

    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "AC-018 (DOCX) Red Gate: 3-slide fixture (strict=true, docx) must build Ok. \
             Got Err: {e:?}"
        )
    });

    assert!(
        !output.bytes.is_empty(),
        "AC-018 DOCX: bytes must be non-empty"
    );
    assert_eq!(
        output.extension, "docx",
        "AC-018 DOCX: extension must be 'docx'"
    );

    let mut archive = open_zip(&output.bytes, "AC-018-docx");
    let doc_xml = read_zip_entry(&mut archive, "word/document.xml", "AC-018-docx");

    // Assert DOCX contains the title text "Annual Review" in a Heading1 run.
    assert!(
        doc_xml.contains("Annual Review"),
        "AC-018 DOCX: word/document.xml must contain title text 'Annual Review'. \
         Stub → no FrameContent::Title → DOCX exporter reads title=\"\" → absent → FAILS."
    );
}

// ─── Helper: read a file from a ZIP archive ──────────────────────────────────

/// Read a named entry from a ZIP archive into a `String`.
///
/// Panics with a descriptive message if the entry is absent or unreadable.
fn read_zip_entry(
    archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
    name: &str,
    label: &str,
) -> String {
    let mut entry = archive
        .by_name(name)
        .unwrap_or_else(|e| panic!("{label}: ZIP entry '{name}' not found: {e}"));
    let mut content = String::new();
    std::io::Read::read_to_string(&mut entry, &mut content)
        .unwrap_or_else(|e| panic!("{label}: cannot read ZIP entry '{name}': {e}"));
    content
}
