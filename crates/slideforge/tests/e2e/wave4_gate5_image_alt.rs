//! Wave 4 Gate-5 Red Gate tests — image slide `src`+alt build pipeline.
//!
//! ## The Gate-5 Blocker (BC-1.16.001 PC-10/EC-006)
//!
//! The `image`, `screenshot`, and `bio` slide types expose a keyword mismatch:
//!
//! | Artifact | Field keyword | Status |
//! |----------|--------------|--------|
//! | `slideforge-plugin-api` `image.rs` `FieldDef` | `"image"` | WRONG (pre-fix) |
//! | `slideforge-plugin-api` `screenshot.rs` `FieldDef` | `"image"` | WRONG (pre-fix) |
//! | `slideforge-plugin-api` `bio.rs` `FieldDef` | `"image"` | WRONG (pre-fix) |
//! | `slideforge-syntax` `known_fields.rs` image/screenshot arm | `"image"` | WRONG (pre-fix) |
//! | `slideforge-syntax` `known_fields.rs` bio arm | `"image"` | WRONG (pre-fix) |
//! | `slideforge-eval` `field_to_block.rs:239` threading pass | `"src"` | CORRECT |
//! | BC-1.16.001 PC-10/EC-006 canonical DSL keyword | `"src"` | CORRECT |
//!
//! Because `field_to_block.rs` reads `"src"` but the FieldDef / known_fields
//! declare `"image"`, a user who authors per the advertised schema with
//! `image "photo.png"` gets no `ContentBlock::Image` threaded → layout retains
//! `AltText::Unspecified` on the structural Image placeholder → strict
//! `validate_post_layout` fires `E-A11-001` even when a valid `alt` is provided.
//!
//! ## Fix: align keyword to `src`
//!
//! The implementer must change:
//! 1. `image.rs` `FieldDef { name: Arc::from("image") }` → `"src"`
//! 2. `screenshot.rs` `FieldDef { name: Arc::from("image") }` → `"src"`
//! 3. `bio.rs` `FieldDef { name: Arc::from("image") }` → `"src"`
//! 4. `known_fields.rs` `"image" | "screenshot"` arm: `"image"` → `"src"`
//! 5. `known_fields.rs` `"bio"` arm: `"image"` → `"src"`
//!
//! ## Red Gate discipline
//!
//! **AC-GATE5-001** (primary positive): `src` + `alt` on image slide → build Ok,
//! PPTX carries `descr="A snow-capped mountain peak lit by early morning sun"`.
//! - **Pre-fix behavior**: PASSES (field_to_block.rs already reads "src" correctly).
//! - Rationale for inclusion: Documents the contract that `src` is the canonical
//!   keyword end-to-end. The `known_fields` unit test (below) is the true Red Gate
//!   for the vocabulary mismatch.
//!
//! **AC-GATE5-002** (primary negative / true Red Gate): `image "photo.png"` (the
//! ADVERTISED schema from `image.rs` required_fields) + `alt` → strict build must
//! return `Ok` after fix. Currently returns `Err(ValidationFailed { E-A11-001 })`.
//! - **Pre-fix behavior**: FAILS (image key not threaded → Unspecified → E-A11-001).
//! - **Post-fix behavior**: PASSES (src is the only key, image key is irrelevant;
//!   but the fix ensures docs + known_fields say src so users write src).
//!
//! **AC-GATE5-003** (WCAG regression): `src` + no alt + no decorative → strict build
//! returns `Err(ValidationFailed { E-A11-001 })`. Must pass BOTH before and after fix.
//!
//! **AC-GATE5-004** (screenshot src+alt): screenshot slide with `src` + `alt` →
//! build Ok, PPTX carries the alt as `descr`. Same threading path as image.
//!
//! **AC-GATE5-005** (known_fields unit): `known_fields("image")` must contain `"src"`
//! and must NOT contain `"image"`. Currently FAILS (known_fields has `"image"`).
//! This is the primary load-bearing Red Gate unit test.

#![allow(clippy::unwrap_used)] // integration tests — explicit panics are correct
#![allow(clippy::doc_markdown)] // test doc comments carry unquoted identifiers

use crate::e2e::{BrandTmpDir, fixture_source, open_zip};

// ─── AC-GATE5-001: image + src + alt → build Ok + PPTX carries descr ─────────

/// AC-GATE5-001 / BC-1.16.001 PC-10 — `image` slide with canonical `src` keyword
/// and valid `alt` in strict mode builds `Ok` and the PPTX `slide1.xml` carries
/// `descr="A snow-capped mountain peak lit by early morning sun"`.
///
/// ## Threading path
///
/// 1. Parser: `src "photo.png"` stored as `FieldValue::Literal(Value::Str("photo.png"))`
///    under key `"src"`. Parser accepts any identifier as a field name on slide blocks
///    (no field-name validation at parse time — only alias/set-rule parsers use known_fields).
/// 2. `field_to_block.rs:239`: finds `slide.fields.get("src")` → emits
///    `ContentBlock::Image { alt: Some(AltText::Provided("A snow-capped…")), path: "photo.png" }`.
/// 3. `regions.rs` `"image"` arm: provides `FrameContent::Image { alt: Unspecified }`.
/// 4. `thread_media_alt_into_frames`: finds the Image frame → overwrites with
///    `AltText::Provided("A snow-capped mountain peak lit by early morning sun")`.
/// 5. `validate_post_layout`: sees `Provided` → no E-A11-001 → strict gate passes.
/// 6. PPTX `AltTextEmbedder::decisions_for_slide`: `Image { alt: Provided(s) }` →
///    `AltDecision::Provided(s)` → `descr_value()` = the alt string.
/// 7. `SlideSerializer`: emits `<p:pic>` with `descr="A snow-capped mountain peak…"`.
///
/// ## Current behavior (pre-fix)
///
/// PASSES — the `src` keyword is already correctly read by `field_to_block.rs`.
/// This test documents the contract for the `src` path (canonical end-to-end).
/// After the fix (known_fields + FieldDef aligned to "src"), this continues to pass.
///
/// Traces: BC-1.16.001 PC-10; BC-5.01.001 postcondition 1; BC-5.02.001 PC-7.
#[test]
fn test_bc_1_16_001_wave4_gate5_image_src_alt_build_ok_pptx_carries_descr() {
    // AC-GATE5-001: image slide with src="photo.png" + alt (canonical, strict=true).
    // Pre-fix: PASSES (field_to_block.rs already reads "src").
    // Post-fix: PASSES (docs + known_fields aligned to "src").
    let brand = BrandTmpDir::new("w4g5_ac001_image_src_alt");
    let source = fixture_source("wave4-image-src-with-alt.sf");
    let opts = brand.build_options("pptx", true); // strict: true

    // Must return Ok — if strict fires E-A11-001, the threading is broken.
    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "AC-GATE5-001: build() with image slide (src + alt, strict=true) must return Ok. \
             Got Err: {e:?}. \
             Check: field_to_block.rs must read 'src' → ContentBlock::Image → alt threaded → \
             AltText::Provided → no E-A11-001. \
             BC-1.16.001 PC-10."
        )
    });

    assert!(
        !output.bytes.is_empty(),
        "AC-GATE5-001: PPTX bytes must be non-empty"
    );
    assert_eq!(
        output.extension, "pptx",
        "AC-GATE5-001: output extension must be 'pptx'"
    );

    // Open PPTX ZIP and extract slide1.xml.
    let mut archive = open_zip(&output.bytes, "AC-GATE5-001");
    let slide_xml = read_zip_entry(&mut archive, "ppt/slides/slide1.xml", "AC-GATE5-001");

    // PRIMARY LOAD-BEARING ASSERTION: the alt text must appear as a `descr` attribute
    // on the PPTX image shape. AltTextEmbedder maps AltText::Provided(s) →
    // AltDecision::Provided(s) → SlideSerializer writes descr="s" on <p:cNvPr>.
    assert!(
        slide_xml.contains("A snow-capped mountain peak lit by early morning sun"),
        "AC-GATE5-001: PPTX slide1.xml must carry the alt text \
         'A snow-capped mountain peak lit by early morning sun' as a descr attribute. \
         Threading path: src field → ContentBlock::Image → thread_media_alt_into_frames \
         → AltText::Provided → AltTextEmbedder → descr. \
         BC-1.16.001 PC-10; BC-5.02.001 PC-7. \
         slide1.xml (first 800 chars): {:.800}",
        slide_xml
    );
}

// ─── AC-GATE5-002: image + `image` keyword (advertised schema) → E-A11-001 ────

/// AC-GATE5-002 / BC-1.16.001 PC-10 — TRUE RED GATE for the Gate-5 defect.
///
/// An `image` slide authored with the keyword `image "screenshot.png"` (as
/// advertised by `image.rs::required_fields()[1].name = "image"` and
/// `known_fields("image")` which lists `"image"`) plus a valid `alt "..."` MUST
/// return `Ok(BuildOutput)` with `descr` set AFTER the fix. Before the fix, it
/// fires `E-A11-001` because `field_to_block.rs` reads `"src"` and `"image"` is
/// a different key that is silently ignored by the threading pass.
///
/// ## Red Gate: why this test fails pre-fix
///
/// 1. User writes `image "screenshot.png"` → stored under key `"image"`.
/// 2. `field_to_block.rs:239`: `extract_str_field(slide, "src")` → `None`
///    (key is `"image"`, not `"src"`). Warning emitted; no ContentBlock::Image.
/// 3. `regions.rs`: provides `FrameContent::Image { alt: Unspecified }`.
/// 4. `thread_media_alt_into_frames`: no ContentBlock::Image to thread → Image
///    frame remains `AltText::Unspecified`.
/// 5. `validate_post_layout`: `Unspecified` → E-A11-001 fired.
/// 6. Strict gate: `Err(ValidationFailed { E-A11-001 })` returned.
///
/// This test currently FAILS at the `unwrap_or_else` panic (build returns Err).
/// After the fix (image.rs FieldDef + known_fields aligned to "src"), users are
/// guided to write `src` and the documented schema matches threading.
///
/// Note: the test itself uses the BROKEN `image` keyword to DEMONSTRATE the defect.
/// The positive test (AC-GATE5-001) uses the CANONICAL `src` keyword.
///
/// Traces: BC-1.16.001 PC-10; BC-5.01.001 postcondition 1 (E-A11-001 NOT fired
/// for a provided alt); BC-5.02.001 PC-7.
#[test]
fn test_bc_1_16_001_wave4_gate5_image_advertised_keyword_fails_e_a11_001() {
    // AC-GATE5-002: image slide with `image "screenshot.png"` (advertised schema) + alt.
    // RED GATE (pre-fix): field_to_block.rs reads "src" NOT "image" → no ContentBlock::Image
    //   → Image frame stays Unspecified → E-A11-001 fires → Err(ValidationFailed).
    // POST-FIX: after known_fields + FieldDef aligned to "src", users write "src".
    //   The `image` keyword still silently passes through (parser accepts any field name)
    //   but won't be threaded. This test documents the pre-fix defect.
    let brand = BrandTmpDir::new("w4g5_ac002_image_wrong_key");
    let source = fixture_source("wave4-image-advertised-schema.sf");
    let opts = brand.build_options("pptx", true); // strict: true

    // Pre-fix: build returns Err(ValidationFailed { E-A11-001 }) — image key not threaded.
    // This assertion FAILS (panics) when the test is run pre-fix:
    let output = slideforge::build(&source, &opts).unwrap_or_else(|e| {
        panic!(
            "AC-GATE5-002 RED GATE: build() with image slide using 'image' field keyword \
             and a valid alt must return Ok after the fix. \
             Got Err: {e:?}. \
             Pre-fix defect: field_to_block.rs reads 'src' not 'image' → \
             ContentBlock::Image never produced → AltText::Unspecified retained → \
             validate_post_layout fires E-A11-001 even though alt is provided. \
             Fix: align image.rs + screenshot.rs + bio.rs FieldDef + known_fields.rs to 'src'. \
             BC-1.16.001 PC-10; Gate-5 blocker."
        )
    });

    // If build returns Ok (post-fix or if the keyword was already "src"), verify descr.
    let mut archive = open_zip(&output.bytes, "AC-GATE5-002");
    let slide_xml = read_zip_entry(&mut archive, "ppt/slides/slide1.xml", "AC-GATE5-002");

    // The alt text must appear in the PPTX as a descr attribute.
    assert!(
        slide_xml.contains("The main dashboard showing Q3 metrics"),
        "AC-GATE5-002: PPTX slide1.xml must carry the alt text \
         'The main dashboard showing Q3 metrics' as a descr attribute. \
         slide1.xml (first 800 chars): {:.800}",
        slide_xml
    );
}

// ─── AC-GATE5-003: WCAG regression — src + no alt → E-A11-001 ────────────────

/// AC-GATE5-003 / BC-5.01.001 postcondition 1 — WCAG regression test.
///
/// An `image` slide with canonical `src` keyword but NO alt text and NO
/// `decorative: true` must fire `E-A11-001` in strict mode.
///
/// ## Threading path (pre-fix and post-fix)
///
/// 1. `field_to_block.rs`: finds `"src"` → emits `ContentBlock::Image { alt: None }`.
/// 2. `thread_media_alt_into_frames`: `alt: None` → warn + leave Unspecified.
/// 3. `validate_post_layout`: `Unspecified` → E-A11-001 fired.
/// 4. Strict gate: `Err(ValidationFailed)`.
///
/// This test must PASS both before and after the fix — it's a regression guard
/// ensuring the fix doesn't accidentally suppress the WCAG alt-text requirement.
///
/// Traces: BC-5.01.001 postcondition 1; BC-5.02.001 PC-7; WCAG 1.1.1.
#[test]
fn test_bc_1_16_001_wave4_gate5_image_src_no_alt_fails_e_a11_001() {
    // AC-GATE5-003: image slide with src but NO alt → Err(ValidationFailed { E-A11-001 }).
    // Passes both before and after the fix.
    let brand = BrandTmpDir::new("w4g5_ac003_image_no_alt");
    let source = fixture_source("wave4-image-src-no-alt.sf");
    let opts = brand.build_options("pptx", true); // strict: true

    let result = slideforge::build(&source, &opts);

    // Must be an error.
    assert!(
        result.is_err(),
        "AC-GATE5-003: build() with image slide (src + NO alt, strict=true) must return Err. \
         Got Ok. WCAG 1.1.1 requires alt text on all non-decorative images. \
         BC-5.01.001 postcondition 1."
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
                "AC-GATE5-003: ValidationFailed must contain exactly 1 E-A11-001 diagnostic \
                 (one image slide, one missing alt). Got {e_a11_count} E-A11-001 in {diagnostics:?}. \
                 BC-5.01.001 postcondition 1; WCAG 1.1.1."
            );
        },
        other => {
            panic!(
                "AC-GATE5-003: build() must return BuildError::ValidationFailed; \
                 got other variant: {other:?}"
            );
        },
    }
}

// ─── AC-GATE5-004: screenshot + src + alt → build Ok + PPTX carries descr ────

/// AC-GATE5-004 / BC-1.16.001 PC-10 — screenshot slide with canonical `src` and
/// valid `alt` in strict mode builds Ok and PPTX carries the alt as `descr`.
///
/// Screenshot shares the same threading path as image:
/// `field_to_block.rs:233` — `matches!(slide_type, "image" | "screenshot" | "bio")`
///
/// Current behavior (pre-fix): PASSES (threading reads "src" correctly).
/// After fix (known_fields + FieldDef aligned to "src"): continues to PASS.
///
/// Traces: BC-1.16.001 PC-10; BC-5.02.001 PC-7.
#[test]
fn test_bc_1_16_001_wave4_gate5_screenshot_src_alt_build_ok_pptx_carries_descr() {
    // AC-GATE5-004: screenshot slide with src + alt in strict mode → Ok + descr.
    let brand = BrandTmpDir::new("w4g5_ac004_screenshot_src_alt");
    let source = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "\n",
        "slide screenshot:\n",
        "  title \"Dashboard Overview\"\n",
        "  src \"dashboard.png\"\n",
        "  alt \"Main analytics dashboard with revenue chart and KPI tiles\"\n",
    );
    let opts = brand.build_options("pptx", true); // strict: true

    let output = slideforge::build(source, &opts).unwrap_or_else(|e| {
        panic!(
            "AC-GATE5-004: build() with screenshot slide (src + alt, strict=true) must return Ok. \
             Got Err: {e:?}. \
             Check: field_to_block.rs matches 'screenshot' in the image-type branch → \
             ContentBlock::Image → threading → AltText::Provided → no E-A11-001. \
             BC-1.16.001 PC-10."
        )
    });

    assert!(
        !output.bytes.is_empty(),
        "AC-GATE5-004: PPTX bytes must be non-empty for screenshot slide"
    );

    let mut archive = open_zip(&output.bytes, "AC-GATE5-004");
    let slide_xml = read_zip_entry(&mut archive, "ppt/slides/slide1.xml", "AC-GATE5-004");

    // The alt text must appear in the PPTX as a descr attribute.
    assert!(
        slide_xml.contains("Main analytics dashboard with revenue chart and KPI tiles"),
        "AC-GATE5-004: PPTX slide1.xml must carry the alt text \
         'Main analytics dashboard with revenue chart and KPI tiles' as a descr attribute. \
         slide1.xml (first 800 chars): {:.800}",
        slide_xml
    );
}

// ─── AC-GATE5-005: known_fields("image") must contain "src" not "image" ───────

/// AC-GATE5-005 / BC-1.16.001 PC-10 — PRIMARY LOAD-BEARING RED GATE UNIT TEST.
///
/// `known_fields("image")` must contain `"src"` and must NOT contain `"image"`.
/// `known_fields("screenshot")` must contain `"src"` and must NOT contain `"image"`.
/// `known_fields("bio")` must contain `"src"` and must NOT contain `"image"`.
///
/// ## Why this is the load-bearing Red Gate
///
/// The threading defect (field_to_block.rs reads "src" while everything else says
/// "image") is a keyword mismatch. The canonical keyword per BC-1.16.001 PC-10 is
/// `src`. The fix must update known_fields.rs (vocabulary registry) so that:
/// - Alias validation accepts `src` on image/screenshot/bio
/// - Set-rule validation accepts `src` on image/screenshot/bio
/// - The DSL reference documentation generated from known_fields is correct
///
/// This test exercises the fix at the lowest level (unit test on known_fields).
/// It currently FAILS because known_fields.rs has `"image"` for these types.
///
/// Traces: BC-1.16.001 PC-10; BC-1.09.002 (known_fields accuracy invariant).
#[test]
fn test_bc_1_16_001_wave4_gate5_known_fields_image_uses_src_not_image_keyword() {
    // AC-GATE5-005 RED GATE: known_fields for image/screenshot/bio must use "src".
    // FAILS pre-fix: known_fields.rs lists "image" for image/screenshot/bio arms.
    // PASSES post-fix: known_fields.rs updated to list "src".

    let image_fields = slideforge_syntax::known_fields::known_fields("image")
        .expect("AC-GATE5-005: 'image' must be a known slide type");

    // PRIMARY ASSERTION: "src" must be in known_fields for image slides.
    // RED GATE: known_fields currently has "image" not "src" → assertion FAILS.
    assert!(
        image_fields.contains(&"src"),
        "AC-GATE5-005 RED GATE: known_fields(\"image\") must contain \"src\" (canonical \
         media-source keyword per BC-1.16.001 PC-10/EC-006 and field_to_block.rs:239). \
         Currently has \"image\" instead. \
         Fix: update known_fields.rs image/screenshot arm to list \"src\" not \"image\"."
    );

    // SECONDARY ASSERTION: "image" must NOT be in known_fields for image slides.
    // After the fix the FieldDef keyword is "src"; "image" is removed.
    assert!(
        !image_fields.contains(&"image"),
        "AC-GATE5-005: known_fields(\"image\") must NOT contain \"image\" (stale keyword). \
         After the fix, \"image\" is replaced by \"src\" in the field registry. \
         BC-1.16.001 PC-10."
    );

    let screenshot_fields = slideforge_syntax::known_fields::known_fields("screenshot")
        .expect("AC-GATE5-005: 'screenshot' must be a known slide type");

    assert!(
        screenshot_fields.contains(&"src"),
        "AC-GATE5-005 RED GATE: known_fields(\"screenshot\") must contain \"src\". \
         screenshot.rs FieldDef uses \"image\"; must be updated to \"src\". \
         BC-1.16.001 PC-10."
    );

    assert!(
        !screenshot_fields.contains(&"image"),
        "AC-GATE5-005: known_fields(\"screenshot\") must NOT contain \"image\" after fix."
    );

    let bio_fields = slideforge_syntax::known_fields::known_fields("bio")
        .expect("AC-GATE5-005: 'bio' must be a known slide type");

    assert!(
        bio_fields.contains(&"src"),
        "AC-GATE5-005 RED GATE: known_fields(\"bio\") must contain \"src\". \
         bio.rs optional FieldDef uses \"image\"; must be updated to \"src\". \
         BC-1.16.001 PC-10."
    );

    assert!(
        !bio_fields.contains(&"image"),
        "AC-GATE5-005: known_fields(\"bio\") must NOT contain \"image\" after fix."
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
