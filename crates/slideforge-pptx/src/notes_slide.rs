//! `NotesSlideSerializer` — produces `ppt/notesSlides/notesSlide{N}.xml` and
//! `ppt/notesSlides/_rels/notesSlide{N}.xml.rels` for each slide that has
//! non-empty speaker notes content.
//!
//! ## Contract (BC-4.01.003 / STORY-040)
//!
//! - One `notesSlide{N}.xml` per slide where `register_content` contains a
//!   non-empty [`slideforge_types::Register::Notes`] entry.
//! - The notes text is placed in the `<p:txBody>` of the body placeholder
//!   (`<p:ph type="body" idx="1">`).
//! - A companion `.rels` file is produced that references both the owning slide
//!   and `notesMaster1.xml`.
//! - Slides WITHOUT notes get no `notesSlide` part (BC-4.01.003 postcondition 2).
//!
//! ## Stub status (STORY-040 Red Gate)
//!
//! All public entry points are `todo!()` stubs.  Tests in
//! `src/tests/notes_tests.rs` call these functions and MUST FAIL until the
//! implementer fills in the production code.

use crate::error::PptxError;

/// Output produced by [`NotesSlideSerializer::build`] for a single slide.
pub struct NotesSlideOutput {
    /// Raw bytes for `ppt/notesSlides/notesSlide{N}.xml`.
    pub xml_bytes: Vec<u8>,
    /// Raw bytes for `ppt/notesSlides/_rels/notesSlide{N}.xml.rels`.
    pub rels_bytes: Vec<u8>,
}

/// Serializes `ppt/notesSlides/notesSlide{N}.xml` for a single slide.
///
/// # Contract
///
/// - Only called when the slide has non-empty `Register::Notes` content.
/// - `slide_index` is 1-based (matches the ZIP file naming convention:
///   `notesSlide1.xml`, `notesSlide2.xml`, …).
/// - `notes_text` is the plain-text string from the Notes register entry.
/// - `slide_rel_id` is the `rId` string used in the `.rels` file to reference
///   the owning slide.
/// - `notes_master_rel_id` is the `rId` for `notesMaster1.xml`.
pub struct NotesSlideSerializer;

impl NotesSlideSerializer {
    /// Build `notesSlide{N}.xml` and its companion `.rels` file.
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::OoxmlElement`] if XML serialization fails.
    ///
    /// # Stub
    ///
    /// This function is unimplemented (STORY-040 Red Gate).
    pub fn build(
        _slide_index: usize,
        _notes_text: &str,
        _slide_rel_id: &str,
        _notes_master_rel_id: &str,
    ) -> Result<NotesSlideOutput, PptxError> {
        todo!(
            "STORY-040 stub: NotesSlideSerializer::build is not yet implemented. \
             Implement this to produce notesSlide XML with notes text in body placeholder \
             (BC-4.01.003 postconditions 1 and 6)."
        )
    }
}
