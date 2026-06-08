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
//! license allows redistribution. Drawing "Hi" (2 ASCII code points) uses at most
//! 3 glyphs (including .notdef), so the subset must be far smaller than 717 KiB.

use std::path::PathBuf;
use std::sync::Arc;

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
#[allow(clippy::unwrap_used)]
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

// ─── AC-009 behavioral test — drives PdfExporter::export() ───────────────────

/// BC-4.03.002 AC-009: Embedded font in PDF exported by `PdfExporter::export()`
/// is a true subset — far smaller than the unsubsetted font file.
///
/// ## What this tests (F-044-002 fix; F1 robustness hardening)
///
/// Drives `PdfExporter::export_uncompressed()` on a fixture `LaidOutDeck`
/// containing a Title frame with text "Hi". The exporter is constructed with
/// `PdfExporter::with_font_path(lm_math_font_path())` so font resolution loads
/// the Latin Modern Math OTF directly — bypassing the brand family-name lookup
/// that would fail in CI/headless environments.
///
/// The export uses **uncompressed content streams** (`compress_content_streams:
/// false`). Note: this flag controls PDF *content-stream* compression (drawing
/// operators), not font-program streams, which may remain Flate-compressed
/// independently at the discretion of the PDF engine. The `< 100_000` bound is
/// robust regardless: a full-font embed exceeds 100 KB whether the font program
/// is compressed (~300–440 KB) or fully uncompressed (~733 KB). A 2-glyph subset
/// comfortably passes in either case (~10–30 KiB). This closes the F1 false-green
/// vector from the old `< 733_736` ceiling (36× too loose):
///
/// - **With old bound:** A full 733 KB font compressed to ~300–440 KB still passed
///   `< 733_736` → false-green.
/// - **With new bound:** `< 100_000` correctly distinguishes subset-scale from
///   full-font-scale, independent of content-stream compression.
///
/// ## Assertions (belt-and-suspenders)
///
/// 1. `embedded_total > 0`: a font stream was embedded (non-vacuous guard).
/// 2. `embedded_total < 100_000`: the embedded font is subset-scale, NOT full-font
///    scale. Rationale: 2–3 glyphs from a 4,802-glyph font subseted to ~10–30 KiB
///    (Latin Modern Math OTF, krilla internal subsetting). 100 KB is a safe ceiling
///    that a full embed (~300–440 KB compressed, ~733 KB uncompressed) would
///    exceed, while a 2-glyph subset comfortably passes.
/// 3. `embedded_total < full_font_size`: belt-and-suspenders against the uncompressed
///    full font baseline.
///
/// ## Font fixture
///
/// `crates/slideforge-math/fonts/latinmodern-math.otf` — 717 KiB, 4,802 glyphs.
/// Drawing only "Hi" (ASCII H=72, i=105 → 2 used glyphs + .notdef) produces a
/// font subset of order 10–30 KiB uncompressed.
///
/// ## Test seam
///
/// `PdfExporter::export_uncompressed` (`#[doc(hidden)] pub`) calls
/// `generate_pdf_inner` with `SerializeSettings { compress_content_streams:
/// false, .. }`. This makes the embedded byte sizes directly observable.
/// `PdfExporter::with_font_path` sets the explicit font path for deterministic
/// font resolution in CI/headless environments.
#[test]
#[allow(clippy::unwrap_used)]
#[allow(clippy::cast_precision_loss)] // usize→f64 only used for diagnostic eprintln! KiB display; not load-bearing
fn test_bc_4_03_002_ac009_font_subset_smaller_than_full_font() {
    use slideforge_layout::types::{
        BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize, RegisterSet,
    };
    use slideforge_pdf::PdfExporter;
    use slideforge_plugin_api::ExportOptions;
    use slideforge_types::{
        Brand, BrandFonts, BrandPalette, Deck, DeckMetadata, Emu, OrderedMap, SourceSpan,
    };

    // Subset-scale ceiling for assertion 2 (declared here to satisfy
    // clippy::items_after_statements — consts must precede let-bindings).
    //
    // A 2-glyph subset of a 4,802-glyph font is ~10–30 KiB uncompressed.
    // 100,000 bytes (≈97.7 KiB) is chosen as the ceiling because:
    //   - A full uncompressed LM Math font is 733,736 bytes — fails this bound.
    //   - A `FlateDecode`-compressed full LM Math is ~300–440 KB — also fails.
    //   - A 2–3 glyph uncompressed subset is typically 10–30 KB — passes easily.
    // If subsetting regresses (full font embedded), assertion 2 fails.
    const SUBSET_SCALE_CEILING: usize = 100_000;

    // ── 1. Load the fixture font and measure its full size ─────────────────
    let font_path = lm_math_font_path();
    let full_font_size = usize::try_from(
        std::fs::metadata(&font_path)
            .unwrap_or_else(|e| panic!("cannot stat LM Math fixture: {e}"))
            .len(),
    )
    .expect("font file size fits in usize on all supported platforms (test-only conversion)");

    assert!(
        full_font_size > 100_000,
        "Fixture too small ({full_font_size} bytes): not a valid large-Unicode-font fixture"
    );

    // ── 2. Build a LaidOutDeck with a Title frame containing "Hi" ─────────
    //
    // The exporter draws Title frames via surface.draw_text(), which triggers
    // krilla's internal subsetting. Using the LM Math font via the font
    // override seam ensures deterministic font resolution.
    let laid_out = LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("title"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(914_400),
                },
                content: FrameContent::Title(Arc::from("Hi")),
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        }],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],    };

    let deck = Deck {
        slides: vec![],
        vars: OrderedMap::new(),
        metadata: DeckMetadata {
            title: Some(Arc::from("AC-009 Test Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        },
        registers: OrderedMap::new(),
        section_blocks: vec![],
    };

    let brand = Brand {
        name: Arc::from("TestBrand"),
        palette: BrandPalette {
            primary: Arc::from("#003087"),
            secondary: Arc::from("#FFFFFF"),
            accent: Arc::from("#F5A623"),
            neutral: Arc::from("#F0F0F0"),
        },
        fonts: BrandFonts {
            heading: Arc::from("NoSuchFont_AC009"),
            body: Arc::from("NoSuchFont_AC009"),
            mono: Arc::from("Courier"),
            font_size_emu: 457_200,
        },
        layouts: vec![],
        span: SourceSpan::default(),
    };

    // ── 3. Export via export_uncompressed() using the font override seam ──
    //
    // export_uncompressed() calls generate_pdf_inner with
    // compress_content_streams: false so stream byte counts directly reflect
    // the raw font program size — no FlateDecode inflation.
    //
    // PdfExporter::with_font_path loads the LM Math font directly without
    // calling system_font_fallback(). This makes the test deterministic in
    // any environment (CI/headless/developer workstation).
    let exporter = PdfExporter::with_font_path(font_path.clone());
    let opts = ExportOptions::default();

    let pdf_bytes = exporter
        .export_uncompressed(&deck, &laid_out, &brand, &opts)
        .unwrap_or_else(|e| {
            panic!("PdfExporter::export_uncompressed must succeed for AC-009 test: {e:?}")
        });

    assert!(
        pdf_bytes.starts_with(b"%PDF-"),
        "PDF output must start with %PDF- header"
    );

    // ── 4. Measure embedded stream total size (uncompressed) ──────────────
    //
    // Because compress_content_streams: false was used, stream bytes are raw
    // (not FlateDecode-encoded). The sum of all stream sizes gives an upper
    // bound on the font program bytes actually embedded.
    //
    // For a 2-glyph Latin Modern Math subset (uncompressed), the embedded
    // font stream should be ~10–30 KiB. Even counting all content streams
    // (page structure, background) the total should be well under 100 KiB.
    let embedded_total = measure_embedded_streams_size(&pdf_bytes);

    eprintln!(
        "[AC-009] uncompressed embedded_total = {embedded_total} bytes ({:.1} KiB)",
        embedded_total as f64 / 1024.0
    );

    // ── AC-009 assertion 1: non-vacuous guard ────────────────────────────
    // A zero total means no font was embedded — the text drawing path is a no-op.
    assert!(
        embedded_total > 0,
        "AC-009 FAILED (non-vacuous guard): embedded stream total is 0 — \
         PdfExporter::export_uncompressed() did NOT embed any font data. \
         Text drawing may not be reaching krilla's surface.draw_text()."
    );

    // ── AC-009 assertion 2: subset-scale bound (the critical one) ────────
    // See SUBSET_SCALE_CEILING constant declared above for full rationale.
    assert!(
        embedded_total < SUBSET_SCALE_CEILING,
        "AC-009 FAILED (subset-scale bound): uncompressed embedded stream total \
         ({embedded_total} bytes, {:.1} KiB) is NOT subset-scale (must be < \
         {SUBSET_SCALE_CEILING} bytes / {:.1} KiB). \
         Expected krilla to subset the font to the 2 glyphs used in 'Hi' (~10–30 KiB). \
         If embedded_total ≈ 733_736, the full font was embedded without subsetting.",
        embedded_total as f64 / 1024.0,
        SUBSET_SCALE_CEILING as f64 / 1024.0,
    );

    // ── AC-009 assertion 3: belt-and-suspenders vs full-font baseline ────
    assert!(
        embedded_total < full_font_size,
        "AC-009 FAILED: uncompressed embedded streams ({embedded_total} bytes) is NOT \
         smaller than the full uncompressed font ({full_font_size} bytes). \
         Subsetting did not occur."
    );

    eprintln!(
        "[AC-009] PASS: PdfExporter::export_uncompressed() embedded {embedded_total} bytes \
         ({:.1} KiB) — subset-scale ({SUBSET_SCALE_CEILING} bytes ceiling) and \
         < full LM Math font {full_font_size} bytes (krilla subsetting confirmed)",
        embedded_total as f64 / 1024.0
    );
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Measure total bytes in all `stream`...`endstream` blocks in a PDF byte slice.
///
/// This is a best-effort heuristic for estimating the total embedded data
/// (font programs, image data, content streams) in the PDF. It sums the raw
/// byte count between each `stream\n` and the corresponding `endstream`,
/// which approximates the sizes of all embedded resources.
///
/// For the AC-009 assertion, this gives an upper bound: if ALL streams combined
/// are smaller than the full font file, subsetting definitely occurred.
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
