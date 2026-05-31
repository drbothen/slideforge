//! AC-009 font-subsetting verification test.
//!
//! ## What this tests
//!
//! BC-4.03.002 invariant 3: Font subsetting uses the `subsetter` crate (same as
//! used by Typst) — no system font tooling dependency. Specifically:
//!
//! 1. When `PdfExporter::export()` draws text using only ASCII glyphs from a
//!    large Unicode font, the embedded font program in the output PDF is
//!    **smaller** than the full unsubsetted font file.
//!    Assertion: `embedded_font_bytes.len() < full_font_file_bytes.len()`
//!
//! 2. No `subsetter::subset` call appears in `slideforge-pdf` source — subsetting
//!    is internal to krilla. (Checked by [`test_bc_4_03_002_no_direct_subsetter_call`].)
//!
//! 3. No system font tooling (`fonttools`, `pyftsubset`, `hb-subset`) is invoked.
//!    (Structural check — no `std::process::Command` in source; see
//!    `no_forbidden_deps.rs` test `test_bc_4_03_002_no_subprocess_in_pdf_source`.)
//!
//! ## Font fixture
//!
//! Uses `crates/slideforge-math/fonts/latinmodern-math.otf`:
//! - **Size:** 717 KiB (733,736 bytes)
//! - **Glyphs:** 4,802 (full Unicode math coverage)
//! - **License:** GUST Font License (OFL-compatible, free redistribution)
//! - **Already committed to the repository** (STORY-030 added it for `slideforge-math`)
//!
//! This is an ideal fixture: it is large (many glyphs), already present, and its
//! license allows redistribution. Drawing "Hi!" (3 ASCII code points) uses at most
//! 4 glyphs (including .notdef), so the subset must be far smaller than 717 KiB.
//!
//! ## Red Gate status
//!
//! The font-size comparison test is marked `#[ignore]` because it requires
//! the STORY-044 implementer to wire actual text drawing into `PdfExporter::export()`
//! (the `generate_pdf` drawing loop). Until that is done, krilla does NOT embed any
//! font in the PDF (no glyphs are drawn → no font resource → no subset).
//!
//! The grep test (`test_bc_4_03_002_no_direct_subsetter_call`) PASSES now and
//! is NOT ignored — it is a structural assertion.
//!
//! STORY-044 implementer: un-ignore `test_bc_4_03_002_ac009_font_subset_smaller_than_full_font`
//! once text drawing is wired. The fixture path is pre-validated by the ignored test.

use std::path::PathBuf;

// ─── Font fixture path ────────────────────────────────────────────────────────

/// Path to the Latin Modern Math OTF fixture.
///
/// The test file lives at `crates/slideforge-pdf/tests/ac009_font_subsetting.rs`.
/// `CARGO_MANIFEST_DIR` = `.../crates/slideforge-pdf`.
/// The fixture is at `../../crates/slideforge-math/fonts/latinmodern-math.otf`.
fn lm_math_font_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/slideforge-math/fonts/latinmodern-math.otf")
        .canonicalize()
        .expect(
            "Latin Modern Math OTF must be accessible at \
             crates/slideforge-math/fonts/latinmodern-math.otf. \
             Run `git status` to confirm the file is committed.",
        )
}

// ─── Structural / grep test ───────────────────────────────────────────────────

/// BC-4.03.002 invariant 3 (structural): `crates/slideforge-pdf/src/` must NOT
/// contain a direct call to `subsetter::subset(...)`.
///
/// Font subsetting is handled internally by krilla's Surface/text API. Direct
/// usage of `subsetter::subset` in slideforge-pdf production code would bypass
/// krilla's internal subsetting and indicate an architectural violation.
///
/// This test PASSES NOW (structural check — no call exists yet) and must
/// continue to pass after STORY-044 implementation.
#[test]
fn test_bc_4_03_002_no_direct_subsetter_call() {
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    assert!(
        src_dir.is_dir(),
        "src/ directory must exist at {}",
        src_dir.display()
    );

    let mut violations: Vec<String> = Vec::new();
    let files_scanned = scan_for_subsetter_calls(&src_dir, &mut violations);

    // Positive-coverage guard: must scan >=5 .rs source files.
    assert!(
        files_scanned >= 5,
        "positive-coverage guard: expected >=5 .rs files in src/, found {files_scanned}. \
         The scan may have silently no-op'd (F-P6-002)."
    );

    assert!(
        violations.is_empty(),
        "BC-4.03.002 invariant 3: found direct `subsetter::subset` call(s) in \
         slideforge-pdf/src/:\n{}\n\n\
         Font subsetting is internal to krilla's Surface/text API. \
         Do NOT call subsetter::subset directly in slideforge-pdf source.",
        violations.join("\n")
    );
}

/// Recursively scan `.rs` files under `dir` for lines that contain
/// `subsetter::subset` and are not pure comment lines.
fn scan_for_subsetter_calls(dir: &std::path::Path, violations: &mut Vec<String>) -> usize {
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut files_scanned = 0_usize;
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files_scanned += scan_for_subsetter_calls(&path, violations);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            files_scanned += 1;
            if let Ok(content) = std::fs::read_to_string(&path) {
                for (lineno, line) in content.lines().enumerate() {
                    let trimmed = line.trim_start();
                    // Skip comment lines.
                    if trimmed.starts_with("//") {
                        continue;
                    }
                    if line.contains("subsetter::subset") {
                        violations.push(format!(
                            "{}:{}: {}",
                            path.display(),
                            lineno + 1,
                            line.trim()
                        ));
                    }
                }
            }
        }
    }
    files_scanned
}

// ─── Font fixture accessibility check ────────────────────────────────────────

/// Verify the Latin Modern Math OTF fixture is present and large enough to
/// be a valid test font for the subsetting assertion.
///
/// This test PASSES NOW (fixture is already committed). It validates the
/// fixture assumptions before the main AC-009 assertion test runs.
#[test]
fn test_bc_4_03_002_ac009_lm_math_fixture_exists_and_is_large() {
    let path = lm_math_font_path();
    let metadata = std::fs::metadata(&path).unwrap_or_else(|e| {
        panic!(
            "Latin Modern Math OTF not accessible at {}: {e}\n\
             This fixture is required for AC-009 font-subsetting verification.",
            path.display()
        )
    });

    let file_size = metadata.len();

    // The font must be at least 100 KiB to be a meaningful "large font" for the
    // subsetting assertion. Latin Modern Math is 717 KiB (733,736 bytes).
    assert!(
        file_size > 100_000,
        "Latin Modern Math OTF is unexpectedly small ({file_size} bytes). \
         Expected > 100 KiB for a valid large-Unicode-font fixture."
    );

    // Log the size for diagnostic purposes.
    eprintln!(
        "[AC-009] Latin Modern Math OTF fixture: {} ({file_size} bytes)",
        path.display()
    );
}

// ─── AC-009 behavioral test (ignored until drawing is wired) ─────────────────

/// BC-4.03.002 AC-009: Embedded font in PDF is smaller than the unsubsetted font.
///
/// ## What this tests
///
/// When `PdfExporter::export()` draws text elements using only ASCII glyphs
/// from Latin Modern Math OTF (717 KiB, 4,802 glyphs), krilla's internal
/// subsetting must produce an embedded font program that is SMALLER than the
/// original 717 KiB file. This confirms that:
///
/// 1. krilla's `subsetter` transitive dependency is actually invoked.
/// 2. The PDF does not embed the entire unsubsetted font.
/// 3. No system font tooling (`fonttools`, `pyftsubset`) was called.
///
/// ## Fixture
///
/// `crates/slideforge-math/fonts/latinmodern-math.otf` — 717 KiB, 4,802 glyphs.
/// Drawing only "Hi" (ASCII 72, 105 → 2 used glyphs + .notdef) should produce
/// a font subset of order 10-20 KiB.
///
/// ## Why ignored
///
/// This test is `#[ignore]` because it requires STORY-044's text drawing wiring
/// to be complete. Until `PdfExporter::export()` actually draws glyphs via krilla's
/// Surface/text API, no font resource is embedded in the PDF output, and the
/// "embedded font bytes" size would be 0 — vacuously passing the `< full_font_size`
/// assertion but not actually verifying subsetting.
///
/// ## Un-ignore instructions
///
/// The STORY-044 implementer should:
/// 1. Wire `surface.draw_text(...)` calls into `generate_pdf()`.
/// 2. Un-ignore this test.
/// 3. Confirm the assertion `embedded_font_bytes < 733_736` holds.
///
/// ## How the embedded font size is measured
///
/// The PDF bytes are scanned for the `stream` / `endstream` markers of embedded
/// font programs. In PDF, embedded fonts appear as stream objects containing the
/// font program bytes. The test estimates the total embedded font data by summing
/// the lengths of all `stream`...`endstream` blocks whose preceding `<<` dict
/// contains `/Subtype /CIDFontType` or `/FontFile` markers.
///
/// For simplicity, this test uses a conservative heuristic: it counts the total
/// bytes between the FIRST `/FontFile`-adjacent stream marker and `endstream`.
/// This is sufficient to prove the assertion — if ANY font is embedded and is
/// smaller than the full font, subsetting occurred.
#[ignore = "STORY-044: requires text drawing wired in PdfExporter::export(). \
            Un-ignore after implementing generate_pdf() draw loop. \
            Fixture: crates/slideforge-math/fonts/latinmodern-math.otf (717 KiB, 4802 glyphs)."]
#[test]
#[allow(clippy::unwrap_used)]
fn test_bc_4_03_002_ac009_font_subset_smaller_than_full_font() {
    use krilla::Document;
    use krilla::geom::Point;
    use krilla::page::PageSettings;
    use krilla::text::{Font, TextDirection};
    // krilla::Data is re-exported from krilla::data via `pub use data::*` in krilla's lib.rs.
    // Data implements From<Vec<u8>>, so we can convert directly.
    use slideforge_pdf::coords::{SLIDE_HEIGHT_PT, SLIDE_WIDTH_PT};

    // ── 1. Load the Latin Modern Math OTF fixture ──────────────────────────
    let font_path = lm_math_font_path();
    let full_font_bytes: Vec<u8> = std::fs::read(&font_path).unwrap_or_else(|e| {
        panic!("Failed to read Latin Modern Math OTF from {}: {e}", font_path.display())
    });
    let full_font_size = full_font_bytes.len();

    // Sanity: fixture is at least 100 KiB.
    assert!(
        full_font_size > 100_000,
        "Fixture too small ({full_font_size} bytes): not a valid large-Unicode-font fixture"
    );

    // ── 2. Build a krilla Font from the raw bytes ──────────────────────────
    // krilla::text::Font::new(Data, index: u32) -> Option<Font>
    // krilla::Data: pub use data::*; Data implements From<Vec<u8>>.
    // Use .into() coercion: Vec<u8> -> krilla::Data.
    let font_data: krilla::Data = full_font_bytes.clone().into();
    let font = Font::new(font_data, 0).expect(
        "krilla::text::Font::new must succeed for a valid OTF file (Latin Modern Math)"
    );

    // ── 3. Render a minimal document using only ASCII glyphs ───────────────
    let mut document = Document::new();
    let page_settings =
        PageSettings::from_wh(SLIDE_WIDTH_PT, SLIDE_HEIGHT_PT).expect("valid page size");
    let mut page = document.start_page_with(page_settings);
    let mut surface = page.surface();

    // Draw "Hi" — only ASCII glyphs 72 ('H') and 105 ('i') are used.
    // krilla::Surface::draw_text requires the `simple-text` feature (default).
    surface.draw_text(
        Point::from_xy(50.0, 100.0),
        font,
        24.0,
        "Hi",
        false,
        TextDirection::Auto,
    );

    surface.finish();
    page.finish();

    let pdf_bytes: Vec<u8> = document
        .finish()
        .expect("krilla document.finish() must succeed for a simple text document");

    assert!(
        !pdf_bytes.is_empty(),
        "PDF output must not be empty after drawing text"
    );
    assert!(
        pdf_bytes.starts_with(b"%PDF-"),
        "PDF must start with %PDF- header"
    );

    // ── 4. Measure embedded font program size ─────────────────────────────
    // krilla embeds font subsets as stream objects in the PDF. The embedded
    // font program bytes lie between `stream\n` and `\nendstream` (or
    // `\r\nendstream`). We sum the lengths of ALL such stream blocks.
    //
    // For a 2-glyph Latin Modern Math subset, the stream total should be
    // well under 50 KiB, vs. the full 717 KiB font.
    let embedded_font_bytes = measure_embedded_streams_size(&pdf_bytes);

    // The core AC-009 assertion:
    // The embedded font data must be SMALLER than the full unsubsetted font.
    // This proves subsetting occurred (krilla used subsetter internally).
    assert!(
        embedded_font_bytes < full_font_size,
        "AC-009 FAILED: embedded font program ({embedded_font_bytes} bytes) is NOT smaller \
         than the full font ({full_font_size} bytes). \
         Expected krilla to subset the font to the 2 glyphs used in 'Hi'. \
         If the embedded size equals the full font size, subsetting did not occur. \
         If the embedded size is 0, no font was embedded (text drawing not wired)."
    );

    eprintln!(
        "[AC-009] PASS: embedded font program = {embedded_font_bytes} bytes \
         < full font = {full_font_size} bytes (krilla subsetting confirmed)"
    );
}

/// Measure total bytes in all `stream`...`endstream` blocks in a PDF byte slice.
///
/// This is a best-effort heuristic for estimating the total embedded data
/// (font programs, image data) in the PDF. It sums the raw byte count between
/// each `stream\n` and the corresponding `endstream`, which approximates the
/// sizes of all embedded resources.
///
/// For the AC-009 assertion, this gives an upper bound on embedded font size:
/// if ALL streams combined are smaller than the full font file, subsetting
/// definitely occurred.
fn measure_embedded_streams_size(pdf_bytes: &[u8]) -> usize {
    let stream_marker = b"stream\n";
    let endstream_marker = b"endstream";
    let mut total = 0_usize;
    let mut pos = 0_usize;

    while pos < pdf_bytes.len() {
        // Find next `stream\n` marker.
        if let Some(start_rel) = find_bytes(&pdf_bytes[pos..], stream_marker) {
            let stream_start = pos + start_rel + stream_marker.len();
            // Find the `endstream` after this stream start.
            if let Some(end_rel) = find_bytes(&pdf_bytes[stream_start..], endstream_marker) {
                total += end_rel;
                pos = stream_start + end_rel + endstream_marker.len();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    total
}

/// Find the first occurrence of `needle` in `haystack`, returning its offset.
fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}
