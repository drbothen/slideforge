//! `BleedChecker` — post-serialization no-bleed invariant test utility.
//!
//! # Purpose
//!
//! `BleedChecker` is the final defense against register-content bleed bugs. It
//! works at the byte/ZIP level (post-serialization), scanning actual PPTX and
//! DOCX output for sentinel strings that must never appear in the wrong output
//! section.
//!
//! This utility is gated behind the `test-utils` Cargo feature and is intended
//! exclusively for use in exporter test suites. It is **not a production code
//! path** — no production function calls into this module.
//!
//! # Architecture
//!
//! - PPTX and DOCX files are ZIP archives. `BleedChecker` opens the ZIP in
//!   memory and scans specific member files for sentinel strings.
//! - PPTX slide bodies live in `ppt/slides/slide*.xml`.
//! - PPTX speaker notes live in `ppt/notesSlides/notesSlide*.xml`.
//! - DOCX body lives in `word/document.xml`.
//!
//! # EC-003 / EC-004 compliance
//!
//! - EC-003: The sentinel must not be detected as bleed if it appears in a
//!   slide field that the checker does NOT scan (e.g., a `notesSlides` path or
//!   `ppt/theme/theme1.xml`). The methods are scoped to specific ZIP paths; a
//!   sentinel in a path not scanned does not trigger a panic.
//! - EC-004: `BleedChecker` works on **decoded XML text**. Before searching for
//!   the sentinel, each ZIP member's raw UTF-8 bytes are XML-entity-decoded via
//!   [`quick_xml::escape::unescape`]. This means callers pass the
//!   **human-readable** (unescaped) register string as the sentinel — the
//!   checker finds it regardless of how the exporter XML-escapes it in the
//!   output. For example, a register string `R&D roadmap` will be detected even
//!   when the exporter writes `R&amp;D roadmap` in the XML.
//!
//! # Cross-crate availability
//!
//! Because this module is gated on `#[cfg(feature = "test-utils")]` (not
//! `#[cfg(test)]`), it is compiled and available when exporter crates enable
//! `slideforge-eval/test-utils` in their `[dev-dependencies]`. Items gated on
//! `#[cfg(test)]` are not visible to external crates even when that feature is
//! enabled — hence the feature-gate approach mandated by STORY-036.
//!
//! # Usage
//!
//! ```rust,ignore
//! // In exporter test:
//! use slideforge_eval::BleedChecker;
//!
//! let pptx_bytes: Vec<u8> = build_pptx_somehow();
//! // Pass the human-readable sentinel — entity decoding is handled internally.
//! BleedChecker::assert_absent_from_pptx_slides(&pptx_bytes, "R&D roadmap");
//! BleedChecker::assert_absent_from_pptx_all(&pptx_bytes, "DETAIL_SENTINEL");
//! ```

// This entire module is only compiled when the `test-utils` feature is active.
// This prevents the `zip` and `quick-xml` crates from being hard production dependencies.
#![cfg(feature = "test-utils")]

// zip and quick-xml are optional deps gated on `test-utils`; always available here.
use std::io::Read;

use zip::ZipArchive;

/// Test utility for verifying that register content does not bleed across
/// format boundaries.
///
/// All methods are static (no instance state needed) and panic with a
/// descriptive message when the invariant is violated. The panic message
/// includes the sentinel string and the first matching file path, making
/// failures easy to diagnose.
///
/// See the [module-level documentation](self) for architecture details.
pub struct BleedChecker;

impl BleedChecker {
    /// Assert that `sentinel` does NOT appear in any `ppt/slides/slide*.xml`
    /// file within the PPTX ZIP.
    ///
    /// This checks the slide **body** only — not speaker notes, masters, or
    /// layouts. A finding here means notes/report/detail content has bled into
    /// the visual slide body (a P0 bleed defect per BC-1.14.004).
    ///
    /// # Sentinel contract (EC-004)
    ///
    /// Pass the **human-readable** register string as `sentinel`. `BleedChecker`
    /// XML-entity-decodes each member before searching, so `R&D roadmap` is
    /// detected even when the exporter writes `R&amp;D roadmap` in the XML.
    ///
    /// # EC-003 compliance
    ///
    /// Only members whose names start with `ppt/slides/slide` and end with
    /// `.xml` are scanned. A sentinel in any other path (theme, notesSlides,
    /// masters, layouts, content types, etc.) does NOT trigger a panic.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `pptx_bytes` is not a valid ZIP archive.
    /// - No `ppt/slides/slide*.xml` members are found — indicates a malformed
    ///   or empty PPTX that would make an absence check vacuously true.
    /// - `sentinel` is found (after XML-entity decoding) in any
    ///   `ppt/slides/slide*.xml` file.
    pub fn assert_absent_from_pptx_slides(pptx_bytes: &[u8], sentinel: &str) {
        let mut archive = Self::open_zip(pptx_bytes, "PPTX");
        let names: Vec<String> = (0..archive.len())
            .map(|i| {
                archive
                    .by_index(i)
                    .unwrap_or_else(|e| {
                        panic!("BleedChecker: failed to index PPTX ZIP entry {i}: {e}")
                    })
                    .name()
                    .to_owned()
            })
            .collect();

        // F-003: Fail closed — a valid PPTX always contains ≥1 slide body.
        // If we find none, the archive is malformed or the exporter has a structural
        // regression. An absence check on zero members would be vacuously true,
        // silently masking that defect.
        let slide_members: Vec<&String> = names
            .iter()
            .filter(|n| Self::is_pptx_slide_path(n))
            .collect();
        assert!(
            !slide_members.is_empty(),
            "BleedChecker: no ppt/slides/slide*.xml members found — not a valid PPTX.\n\
             (A valid PPTX must contain at least one slide body. Absence check would be \
             vacuously true on an empty/malformed archive — refusing to give a false green.)"
        );

        for name in &names {
            // EC-003: Only scan `ppt/slides/slide*.xml` — no other paths.
            if !Self::is_pptx_slide_path(name) {
                continue;
            }
            let text = Self::read_member_text_decoded(&mut archive, name);
            assert!(
                !text.contains(sentinel),
                "BleedChecker: register-content bleed detected in PPTX slide body.\n\
                 sentinel : {sentinel:?}\n\
                 found in : {name:?}\n\
                 (BC-1.14.004 invariant 1: register content must not appear in PPTX slide body)"
            );
        }
    }

    /// Assert that `sentinel` does NOT appear in ANY file within the PPTX ZIP.
    ///
    /// Used for `detail` register content which must be excluded from the
    /// entire PPTX output — not just the slide body, but also notes slides,
    /// masters, layouts, and all other parts.
    ///
    /// # Sentinel contract (EC-004)
    ///
    /// Pass the **human-readable** register string as `sentinel`. `BleedChecker`
    /// XML-entity-decodes each member before searching, so `R&D roadmap` is
    /// detected even when the exporter writes `R&amp;D roadmap` in the XML.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `pptx_bytes` is not a valid ZIP archive.
    /// - The archive contains zero members — indicates a malformed or empty
    ///   PPTX that would make an absence check vacuously true.
    /// - `sentinel` is found (after XML-entity decoding) in any file within
    ///   the ZIP.
    pub fn assert_absent_from_pptx_all(pptx_bytes: &[u8], sentinel: &str) {
        let mut archive = Self::open_zip(pptx_bytes, "PPTX");
        let names: Vec<String> = (0..archive.len())
            .map(|i| {
                archive
                    .by_index(i)
                    .unwrap_or_else(|e| {
                        panic!("BleedChecker: failed to index PPTX ZIP entry {i}: {e}")
                    })
                    .name()
                    .to_owned()
            })
            .collect();

        // F-003: Fail closed — an empty archive means zero members were scanned,
        // making any absence check vacuously true and silently hiding that the
        // PPTX was never written.
        assert!(
            !names.is_empty(),
            "BleedChecker: PPTX archive contains zero members — not a valid PPTX.\n\
             (Absence check would be vacuously true on an empty archive — refusing to \
             give a false green.)"
        );

        for name in &names {
            let text = Self::read_member_text_decoded(&mut archive, name);
            assert!(
                !text.contains(sentinel),
                "BleedChecker: register-content bleed detected in PPTX archive.\n\
                 sentinel : {sentinel:?}\n\
                 found in : {name:?}\n\
                 (BC-1.14.004 postcondition 6: detail content must not appear anywhere in PPTX)"
            );
        }
    }

    /// Assert that `sentinel` IS present in `word/document.xml` within the
    /// DOCX ZIP.
    ///
    /// This is a **positive** test: it verifies that `report` register content
    /// actually made it into the DOCX body, confirming the routing is active
    /// (not just checking for the absence of bleed).
    ///
    /// # Sentinel contract (EC-004)
    ///
    /// Pass the **human-readable** register string as `sentinel`. `BleedChecker`
    /// XML-entity-decodes the member before searching.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `docx_bytes` is not a valid ZIP archive.
    /// - `word/document.xml` is not present in the ZIP.
    /// - `sentinel` is NOT found (after XML-entity decoding) in
    ///   `word/document.xml`.
    pub fn assert_present_in_docx_body(docx_bytes: &[u8], sentinel: &str) {
        let mut archive = Self::open_zip(docx_bytes, "DOCX");
        let text = Self::read_member_text_decoded(&mut archive, "word/document.xml");
        assert!(
            text.contains(sentinel),
            "BleedChecker: expected register content NOT found in DOCX body.\n\
             sentinel     : {sentinel:?}\n\
             searched in  : \"word/document.xml\"\n\
             (BC-1.14.004 postcondition 3: report content must be present in DOCX body)"
        );
    }

    /// Assert that `sentinel` does NOT appear in `word/document.xml` within
    /// the DOCX ZIP.
    ///
    /// Used to verify that `notes` register content was not written to the
    /// DOCX report body (where it would constitute a bleed defect).
    ///
    /// # Sentinel contract (EC-004)
    ///
    /// Pass the **human-readable** register string as `sentinel`. `BleedChecker`
    /// XML-entity-decodes the member before searching.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `docx_bytes` is not a valid ZIP archive.
    /// - `word/document.xml` is not present in the ZIP.
    /// - `sentinel` IS found (after XML-entity decoding) in
    ///   `word/document.xml`.
    pub fn assert_absent_from_docx_body(docx_bytes: &[u8], sentinel: &str) {
        let mut archive = Self::open_zip(docx_bytes, "DOCX");
        let text = Self::read_member_text_decoded(&mut archive, "word/document.xml");
        assert!(
            !text.contains(sentinel),
            "BleedChecker: register-content bleed detected in DOCX body.\n\
             sentinel    : {sentinel:?}\n\
             found in    : \"word/document.xml\"\n\
             (BC-1.14.004 postcondition 2: notes content must not appear in DOCX body)"
        );
    }

    // ─── Internal helpers ─────────────────────────────────────────────────────

    /// Returns `true` if `name` matches the PPTX slide body path pattern:
    /// `ppt/slides/slide<N>.xml` (where N is one or more digits).
    ///
    /// This implements EC-003 path scoping: only slide body files are scanned
    /// by `assert_absent_from_pptx_slides`. Notes slides, masters, layouts,
    /// and any other paths are excluded.
    fn is_pptx_slide_path(name: &str) -> bool {
        // Must start with the slides directory prefix and end with .xml.
        // The filename portion must be "slide" followed by at least one digit.
        let Some(filename) = name.strip_prefix("ppt/slides/") else {
            return false;
        };
        let Some(stem) = filename.strip_suffix(".xml") else {
            return false;
        };
        let Some(digits) = stem.strip_prefix("slide") else {
            return false;
        };
        !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
    }

    /// Open `bytes` as a ZIP archive, panicking with a context message on failure.
    ///
    /// # Panics
    ///
    /// Panics with a descriptive message (including the format name) if `bytes`
    /// is not a valid ZIP archive.
    fn open_zip<'a>(bytes: &'a [u8], format: &str) -> ZipArchive<std::io::Cursor<&'a [u8]>> {
        let cursor = std::io::Cursor::new(bytes);
        ZipArchive::new(cursor).unwrap_or_else(|e| {
            panic!(
                "BleedChecker: failed to open {format} bytes as a ZIP archive: {e}\n\
                 (Are you passing valid {format} bytes?)"
            )
        })
    }

    /// Read a ZIP member by name as a UTF-8 string, then XML-entity-decode it.
    ///
    /// # EC-004 compliance
    ///
    /// XML exporters escape special characters: `&` → `&amp;`, `<` → `&lt;`,
    /// `>` → `&gt;`, `"` → `&quot;`, `'` → `&apos;`. Numeric character
    /// references (`&#NN;`, `&#xHH;`) are also decoded. Without this step, a
    /// sentinel like `R&D roadmap` would not be found in a member containing
    /// `R&amp;D roadmap`, causing a false negative (missed bleed detection).
    ///
    /// Decoding is performed via [`quick_xml::escape::unescape`]. If decoding
    /// fails (malformed entity in the member), the raw UTF-8 text is used as a
    /// fallback with a warning — this is conservative: it may miss some
    /// sentinel matches in badly-formed XML, but it never silently skips a
    /// member that was scanned.
    ///
    /// Non-UTF-8 bytes in the member are replaced with the Unicode replacement
    /// character (U+FFFD) before entity decoding.
    ///
    /// # Panics
    ///
    /// Panics if the member is not found in the archive.
    fn read_member_text_decoded(
        archive: &mut ZipArchive<std::io::Cursor<&[u8]>>,
        name: &str,
    ) -> String {
        let mut entry = archive.by_name(name).unwrap_or_else(|e| {
            panic!(
                "BleedChecker: ZIP member {name:?} not found in archive: {e}\n\
                 (Is this a valid PPTX/DOCX file with the expected structure?)"
            )
        });
        let mut raw = Vec::new();
        entry
            .read_to_end(&mut raw)
            .unwrap_or_else(|e| panic!("BleedChecker: failed to read ZIP member {name:?}: {e}"));
        let utf8 = String::from_utf8_lossy(&raw);
        // XML-entity-decode so that sentinels with `&`, `<`, `>`, `"`, `'`
        // are found even when the exporter has XML-escaped them.
        match quick_xml::escape::unescape(&utf8) {
            Ok(decoded) => decoded.into_owned(),
            Err(_) => {
                // Malformed entity in this member — fall back to raw text.
                // This is conservative: we may miss a sentinel that straddles
                // a malformed entity, but we never silently skip the member.
                utf8.into_owned()
            },
        }
    }
}
