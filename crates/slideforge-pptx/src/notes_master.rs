//! `NotesMasterSerializer` — produces `ppt/notesMasters/notesMaster1.xml`
//! and `ppt/handoutMasters/handoutMaster1.xml`.
//!
//! ## Contract (BC-4.01.006 / STORY-040)
//!
//! Both master parts are ALWAYS emitted — no conditional omission even when
//! the deck has zero speaker notes (BC-4.01.006 invariant 1).
//!
//! `notesMaster1.xml` replaces the empty stub from STORY-037 with a minimal
//! valid notes master containing the required placeholder types (`sldImg` and
//! `body`). If the brand template provides a pre-serialized notes master, that
//! is used verbatim.
//!
//! `handoutMaster1.xml` is a minimal valid stub (always generated, always
//! present).
//!
//! ## Stub status (STORY-040 Red Gate)
//!
//! All public entry points are `todo!()` stubs.  Tests in
//! `src/tests/notes_tests.rs` call these functions and MUST FAIL until the
//! implementer fills in the production code.

use crate::error::PptxError;

/// Produces `ppt/notesMasters/notesMaster1.xml` and
/// `ppt/handoutMasters/handoutMaster1.xml`.
pub struct NotesMasterSerializer;

impl NotesMasterSerializer {
    /// Build a minimal valid `notesMaster1.xml`.
    ///
    /// If `brand_notes_master_bytes` is non-empty, returns those bytes verbatim
    /// (preserving brand-specific styling). Otherwise generates a minimal valid
    /// notes master from scratch.
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::OoxmlElement`] if XML serialization fails.
    ///
    /// # Stub
    ///
    /// This function is unimplemented (STORY-040 Red Gate).
    pub fn build_notes_master(
        _brand_notes_master_bytes: &[u8],
    ) -> Result<Vec<u8>, PptxError> {
        todo!(
            "STORY-040 stub: NotesMasterSerializer::build_notes_master is not yet implemented. \
             Implement this to produce a minimal valid notesMaster1.xml with sldImg and body \
             placeholders (BC-4.01.006 postcondition 1)."
        )
    }

    /// Build a minimal valid `handoutMaster1.xml`.
    ///
    /// If `brand_handout_master_bytes` is non-empty, returns those bytes verbatim.
    /// Otherwise generates a minimal valid handout master stub.
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::OoxmlElement`] if XML serialization fails.
    ///
    /// # Stub
    ///
    /// This function is unimplemented (STORY-040 Red Gate).
    pub fn build_handout_master(
        _brand_handout_master_bytes: &[u8],
    ) -> Result<Vec<u8>, PptxError> {
        todo!(
            "STORY-040 stub: NotesMasterSerializer::build_handout_master is not yet implemented. \
             Implement this to produce a minimal valid handoutMaster1.xml \
             (BC-4.01.006 postcondition 2)."
        )
    }
}
