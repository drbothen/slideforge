//! `SectionListBuilder` — produces the `<p:sectionLst>` element in
//! `ppt/presentation.xml` when the deck contains PPTX slide sections.
//!
//! ## What is a PPTX slide section?
//!
//! A PPTX slide section is a grouping of slides declared with
//! `section "Name": ...` syntax at the deck level. The PPTX serializes these
//! as `<p:section name="…" id="{GUID}"><p:sldIdLst>…</p:sldIdLst></p:section>`
//! inside `<p:sectionLst>` in `presentation.xml`.
//!
//! ## Data source (UPSTREAM GAP — see upstream_data_verification in tests)
//!
//! PPTX slide sections are a distinct concept from the DOCX/PDF document
//! sections stored in `LaidOutDeck::sections` (`GeneratedSection` in
//! `slideforge_layout::sections`). Those are DOCX/PDF document-structure
//! sections (`methodology`, `executive_summary`, etc.) — NOT slide groupings.
//!
//! The `section "Name":` DSL construct that produces PPTX slide sections is
//! NOT yet threaded through the pipeline as of the current IR:
//!
//! - `Deck::section_blocks` carries DOCX/PDF `SectionBlock`s (e.g.,
//!   `section methodology:`) — not PPTX slide groupings.
//! - `LaidOutDeck::sections` carries `GeneratedSection`s for DOCX/PDF output.
//! - Neither `LaidOutSlide` nor `LaidOutDeck` has a `pptx_sections` field.
//!
//! ## Contract (BC-4.01.003 / STORY-040)
//!
//! - When `sections` is empty → no `<p:sectionLst>` in `presentation.xml`.
//! - When `sections` is non-empty → emit `<p:sectionLst>` with one
//!   `<p:section>` per entry. Names are XML-escaped. GUIDs are deterministic
//!   (SHA-256-seeded), NOT random UUIDs.
//!
//! ## Deterministic GUID generation
//!
//! Section GUIDs are derived from the section name via SHA-256:
//! `sha2::Sha256::digest(name.as_bytes())` → first 16 bytes → RFC-4122
//! UUID v4-format string (with version/variant bits set). This guarantees
//! identical GUIDs for identical section names across builds.
//!
//! ## Stub status (STORY-040 Red Gate)
//!
//! All public entry points are `todo!()` stubs.  Tests in
//! `src/tests/notes_tests.rs` call these functions and MUST FAIL until the
//! implementer fills in the production code.

use crate::error::PptxError;

/// A PPTX slide section: a named grouping of consecutive slides.
///
/// Used as input to [`SectionListBuilder::build`]. Each entry maps a section
/// name to the ordered list of 1-based slide IDs (starting at 256) that belong
/// to it.
#[derive(Debug, Clone)]
pub struct PptxSection {
    /// The section name as declared in the DSL (pre-XML-escape).
    pub name: String,
    /// The PPTX slide IDs that belong to this section (1-based, starting at 256).
    pub slide_ids: Vec<u32>,
}

/// Builds the `<p:sectionLst>` XML fragment for `presentation.xml`.
pub struct SectionListBuilder;

impl SectionListBuilder {
    /// Produce the `<p:sectionLst>` XML bytes for the given sections.
    ///
    /// When `sections` is empty, returns `None` (no sectionLst element needed).
    /// When non-empty, returns `Some(bytes)` containing the `<p:sectionLst>` XML.
    ///
    /// ## Deterministic GUIDs
    ///
    /// Each section GUID is derived from the section name via `sha2::Sha256`
    /// (NOT `Uuid::new_v4()`). Same name → same GUID across builds.
    ///
    /// ## XML escaping
    ///
    /// Section names are XML-escaped: `&` → `&amp;`, `<` → `&lt;`, etc.
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::OoxmlElement`] if XML serialization fails.
    ///
    /// # Stub
    ///
    /// This function is unimplemented (STORY-040 Red Gate).
    pub fn build(
        _sections: &[PptxSection],
    ) -> Result<Option<Vec<u8>>, PptxError> {
        todo!(
            "STORY-040 stub: SectionListBuilder::build is not yet implemented. \
             Implement this to produce <p:sectionLst> XML with deterministic GUIDs \
             (BC-4.01.003 postcondition 5, invariant 3)."
        )
    }

    /// Compute a deterministic GUID string from a section name.
    ///
    /// Uses SHA-256 to hash the UTF-8 bytes of `name`, takes the first 16
    /// bytes of the digest, and formats them as an RFC-4122 UUID v4-format
    /// string (with version nibble = 4 and variant bits = 10xx).
    ///
    /// # Stub
    ///
    /// This function is unimplemented (STORY-040 Red Gate).
    pub fn guid_for_name(_name: &str) -> String {
        todo!(
            "STORY-040 stub: SectionListBuilder::guid_for_name is not yet implemented. \
             Implement this using sha2::Sha256 (deterministic, NOT Uuid::new_v4) \
             (BC-4.01.003 invariant 3)."
        )
    }
}
