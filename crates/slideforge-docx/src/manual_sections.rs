//! Manually-authored document section serializer for DOCX output.
//!
//! [`ManualSectionSerializer`] converts a [`slideforge_layout::sections::GeneratedSection`]
//! with [`slideforge_layout::sections::SectionSource::ManuallyAuthored`] into
//! `<w:body>` XML: a `Heading1` paragraph followed by the section's content.
//!
//! ## Invariant (BC-4.02.002 invariant 3)
//!
//! Manual sections are NOT merged with auto-generated sections of the same
//! logical name. Each manual section is serialized at its declared position as
//! determined by the [`crate::section_order::SectionOrderer`].
//!
//! ## Content blocks
//!
//! Content paragraphs are sourced from
//! [`slideforge_layout::sections::GeneratedSection::register_content`]: only
//! `Register::Report` and `Register::Detail` entries are emitted.
//! `Register::Notes` is excluded per BC-4.02.001 invariant 1 / DI-012.

use ooxmlsdk::schemas::schemas_openxmlformats_org_wordprocessingml_2006_main::BodyChoice;
use slideforge_layout::sections::GeneratedSection;

use crate::error::ExportError;

/// Serializes manually-authored [`GeneratedSection`] entries into `<w:body>` XML.
///
/// Emits a `Heading1` paragraph using the section's `heading` field, followed
/// by one paragraph per item in `register_content` (Report and Detail only).
pub struct ManualSectionSerializer;

impl ManualSectionSerializer {
    /// Create a new serializer.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Serialize one manually-authored section into a list of [`BodyChoice`] elements.
    ///
    /// Emits:
    /// 1. `Heading1` paragraph with `section.heading` as text.
    /// 2. One `Normal` paragraph per `Register::Report` entry in
    ///    `section.register_content`.
    ///
    /// `Register::Notes` entries are silently skipped (BC-4.02.001 invariant 1).
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::OoxmlError`] if OOXML construction fails.
    pub fn serialize_section(
        &self,
        section: &GeneratedSection,
    ) -> Result<Vec<BodyChoice>, ExportError> {
        todo!(
            "ManualSectionSerializer::serialize_section — not yet implemented (STORY-042 Step 4). \
             Heading: {:?}, register_content entries: {}",
            section.heading,
            section.register_content.len()
        )
    }
}

impl Default for ManualSectionSerializer {
    fn default() -> Self {
        Self::new()
    }
}
