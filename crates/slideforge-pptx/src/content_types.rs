//! `[Content_Types].xml` generator for PPTX archives.
//!
//! [`ContentTypesBuilder`] accumulates the set of parts actually written to
//! the ZIP and produces the `[Content_Types].xml` byte sequence that registers
//! every part's MIME content-type. Both `<Default>` and `<Override>` entries
//! are required by the OOXML spec; missing entries cause "repair required"
//! dialogs in some renderers.
//!
//! ## Required entries (AC-003)
//!
//! Every PPTX part must have a corresponding `<Override>` (or `<Default>` for
//! extension-level coverage). Mandatory registrations include:
//! - `ppt/slides/slide*.xml` — one per slide
//! - `ppt/slideLayouts/slideLayout*.xml` — one per layout (31 total)
//! - `ppt/slideMasters/slideMaster1.xml`
//! - `ppt/theme/theme1.xml`
//! - `ppt/notesMasters/notesMaster1.xml`
//! - `ppt/handoutMasters/handoutMaster1.xml`
//! - `docProps/core.xml`
//! - `docProps/app.xml`
//! - `<Default>` for `.rels` and `.xml` extensions

use crate::error::PptxError;

/// Builds the `[Content_Types].xml` part for a PPTX archive.
///
/// The caller registers parts by calling `add_slide`, `add_layout`, etc.
/// The final XML bytes are produced by [`ContentTypesBuilder::build`].
pub struct ContentTypesBuilder {
    /// Number of slides registered so far.
    slide_count: usize,
    /// Number of layouts registered so far.
    layout_count: usize,
    /// Whether a media part (SVG/PNG) has been registered.
    has_media: bool,
}

impl ContentTypesBuilder {
    /// Create a new builder with zero parts registered.
    #[must_use]
    pub fn new() -> Self {
        todo!("ContentTypesBuilder::new")
    }

    /// Register one slide part (`ppt/slides/slide{n}.xml`).
    pub fn add_slide(&mut self) {
        todo!("ContentTypesBuilder::add_slide")
    }

    /// Register one layout part (`ppt/slideLayouts/slideLayout{n}.xml`).
    pub fn add_layout(&mut self) {
        todo!("ContentTypesBuilder::add_layout")
    }

    /// Register a media part (SVG or PNG image in `ppt/media/`).
    ///
    /// `content_type` is the MIME type string (e.g., `"image/svg+xml"`).
    pub fn add_media(&mut self, content_type: &str) {
        todo!("ContentTypesBuilder::add_media — store content_type for Override entry")
    }

    /// Serialise all registered parts into `[Content_Types].xml` bytes.
    ///
    /// Entries are produced in deterministic order: `<Default>` elements first,
    /// then `<Override>` elements in part-path alphabetical order.
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::OoxmlElement`] if serialisation fails.
    pub fn build(self) -> Result<Vec<u8>, PptxError> {
        todo!("ContentTypesBuilder::build — emit XML with all Default + Override entries")
    }
}

impl Default for ContentTypesBuilder {
    fn default() -> Self {
        todo!("ContentTypesBuilder::default — delegate to new()")
    }
}
