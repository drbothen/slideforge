//! DOCX `word/document.xml` body serializer.
//!
//! [`DocumentBodySerializer`] transforms the [`slideforge_layout::LaidOutDeck`]
//! into the `<w:body>` XML fragment that forms the main content of a `.docx`
//! document. It enforces the register routing rules from BC-4.02.001:
//!
//! - `Register::Report` content → narrative body paragraphs under per-slide
//!   `<w:p style="Heading1">` headings
//! - `Register::Detail` content → extended sections after the main body,
//!   under `<w:p style="Heading2">` headings
//! - `Register::Notes` content → **never** emitted to `<w:body>`
//!   (BC-4.02.001 invariant 1 / DI-012)
//!
//! Inline formatting (`InlineNode` variants) is mapped to Word `<w:rPr>`
//! properties per the table in STORY-041 § "Inline Formatting".
//!
//! ## Hyperlink relationship tracking
//!
//! Each `InlineNode::Link { url, .. }` generates a relationship entry in
//! `word/_rels/document.xml.rels`. The serializer assigns sequential `rId`
//! values starting at `rId1` and exposes the accumulated relationship map via
//! [`DocumentBodySerializer::relationships`].

use slideforge_layout::LaidOutDeck;

use crate::error::ExportError;

/// A hyperlink relationship entry for `word/_rels/document.xml.rels`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperlinkRel {
    /// The sequential relationship ID (e.g., `"rId1"`).
    pub r_id: String,
    /// The target URL.
    pub target: String,
}

/// Serializes a [`LaidOutDeck`] into the `word/document.xml` body XML and
/// accumulates hyperlink relationships.
pub struct DocumentBodySerializer {
    /// Hyperlink relationships collected during serialization.
    relationships: Vec<HyperlinkRel>,
}

impl DocumentBodySerializer {
    /// Create a new serializer.
    #[must_use]
    pub fn new() -> Self {
        todo!()
    }

    /// Serialize the laid-out deck into `word/document.xml` bytes.
    ///
    /// After calling this method, [`Self::relationships`] contains all
    /// hyperlink relationships discovered during serialization.
    ///
    /// # Register routing
    ///
    /// - Per slide: emits `<w:p style="Heading1">` from `slide.slide_type_keyword`
    ///   (the slide title as known to layout), then `<w:p style="Normal">` for
    ///   each `Register::Report` entry in `slide.register_content`. If there are
    ///   no Report entries, emits an empty `<w:p/>` (AC-009).
    /// - After all slides: emits `<w:p style="Heading2">` + body paragraphs for
    ///   each `Register::Detail` entry across all slides.
    /// - `Register::Notes` entries are silently skipped (BC-4.02.001 invariant 1).
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::OoxmlError`] if the XML cannot be constructed.
    pub fn serialize(&mut self, deck: &LaidOutDeck) -> Result<Vec<u8>, ExportError> {
        todo!()
    }

    /// Return the hyperlink relationships accumulated during the last call to
    /// [`Self::serialize`].
    ///
    /// Returns an empty slice before [`Self::serialize`] is called.
    #[must_use]
    pub fn relationships(&self) -> &[HyperlinkRel] {
        todo!()
    }
}

impl Default for DocumentBodySerializer {
    fn default() -> Self {
        Self::new()
    }
}
