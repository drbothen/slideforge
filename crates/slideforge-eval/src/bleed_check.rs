//! BleedChecker — post-serialization no-bleed invariant test utility.
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
//!   slide field that the checker does NOT scan (e.g., a `title` field). The
//!   methods are scoped to specific ZIP paths; a sentinel in a path not scanned
//!   does not trigger a panic.
//! - EC-004: `BleedChecker` works on decoded XML text (via [`zip`] member
//!   extraction to `String`), not raw bytes. This means XML-escaped content
//!   like `&amp;` in the raw bytes will be present as `&amp;` in the string —
//!   if the sentinel contains `&`, the search correctly finds the escaped form.
//!   Callers should use sentinels that are XML-safe (no `<`, `>`, `&`, `"`, `'`).
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
//! BleedChecker::assert_absent_from_pptx_slides(&pptx_bytes, "NOTES_SENTINEL");
//! BleedChecker::assert_absent_from_pptx_all(&pptx_bytes, "DETAIL_SENTINEL");
//! ```

// This entire module is only compiled when the `test-utils` feature is active.
// This prevents the `zip` crate from being a hard production dependency.
#![cfg(feature = "test-utils")]

// zip is an optional dep gated on `test-utils`; it is always available here.
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
    /// # Panics
    ///
    /// Panics if:
    /// - `pptx_bytes` is not a valid ZIP archive.
    /// - `sentinel` is found in any `ppt/slides/slide*.xml` file.
    pub fn assert_absent_from_pptx_slides(pptx_bytes: &[u8], sentinel: &str) {
        todo!(
            "BleedChecker::assert_absent_from_pptx_slides — to be implemented by STORY-036 implementer. \
             Must: (1) open pptx_bytes as a ZIP archive, (2) iterate all members matching \
             `ppt/slides/slide*.xml`, (3) read each member as UTF-8 text, (4) assert sentinel \
             is absent from the text. Sentinel: {:?}",
            sentinel
        )
    }

    /// Assert that `sentinel` does NOT appear in ANY file within the PPTX ZIP.
    ///
    /// Used for `detail` register content which must be excluded from the
    /// entire PPTX output — not just the slide body, but also notes slides,
    /// masters, layouts, and all other parts.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `pptx_bytes` is not a valid ZIP archive.
    /// - `sentinel` is found in any file within the ZIP.
    pub fn assert_absent_from_pptx_all(pptx_bytes: &[u8], sentinel: &str) {
        todo!(
            "BleedChecker::assert_absent_from_pptx_all — to be implemented by STORY-036 implementer. \
             Must: (1) open pptx_bytes as a ZIP archive, (2) iterate ALL members, (3) read each \
             as UTF-8 text (skip binary members gracefully), (4) assert sentinel is absent from \
             every member. Sentinel: {:?}",
            sentinel
        )
    }

    /// Assert that `sentinel` IS present in `word/document.xml` within the
    /// DOCX ZIP.
    ///
    /// This is a **positive** test: it verifies that `report` register content
    /// actually made it into the DOCX body, confirming the routing is active
    /// (not just checking for the absence of bleed).
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `docx_bytes` is not a valid ZIP archive.
    /// - `word/document.xml` is not present in the ZIP.
    /// - `sentinel` is NOT found in `word/document.xml`.
    pub fn assert_present_in_docx_body(docx_bytes: &[u8], sentinel: &str) {
        todo!(
            "BleedChecker::assert_present_in_docx_body — to be implemented by STORY-036 implementer. \
             Must: (1) open docx_bytes as a ZIP archive, (2) find `word/document.xml`, (3) read it \
             as UTF-8 text, (4) assert sentinel IS present. Panics if absent. Sentinel: {:?}",
            sentinel
        )
    }

    /// Assert that `sentinel` does NOT appear in `word/document.xml` within
    /// the DOCX ZIP.
    ///
    /// Used to verify that `notes` register content was not written to the
    /// DOCX report body (where it would constitute a bleed defect).
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `docx_bytes` is not a valid ZIP archive.
    /// - `word/document.xml` is not present in the ZIP.
    /// - `sentinel` IS found in `word/document.xml`.
    pub fn assert_absent_from_docx_body(docx_bytes: &[u8], sentinel: &str) {
        todo!(
            "BleedChecker::assert_absent_from_docx_body — to be implemented by STORY-036 implementer. \
             Must: (1) open docx_bytes as a ZIP archive, (2) find `word/document.xml`, (3) read it \
             as UTF-8 text, (4) assert sentinel is absent. Sentinel: {:?}",
            sentinel
        )
    }

    // ─── Internal helpers (to be implemented alongside the stubs above) ───────

    /// Open `bytes` as a ZIP archive.
    ///
    /// # Panics
    ///
    /// Panics with a descriptive message if `bytes` is not a valid ZIP.
    #[allow(dead_code)]
    fn open_zip(bytes: &[u8]) -> ZipArchive<std::io::Cursor<&[u8]>> {
        todo!("open_zip: wrap bytes in Cursor, call ZipArchive::new, panic with context on error")
    }

    /// Read a ZIP member by name as a UTF-8 string.
    ///
    /// # Panics
    ///
    /// Panics if the member is not found or cannot be decoded as UTF-8.
    #[allow(dead_code)]
    fn read_member_text(archive: &mut ZipArchive<std::io::Cursor<&[u8]>>, name: &str) -> String {
        todo!(
            "read_member_text: call archive.by_name(name), read to bytes, \
             decode as UTF-8, panic with context on error. Member: {:?}",
            name
        )
    }
}
