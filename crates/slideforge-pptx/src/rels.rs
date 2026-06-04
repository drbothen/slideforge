//! Relationship file (`.rels`) generator for PPTX archives.
//!
//! Every XML part in an OPC package has a paired `.rels` file that maps
//! relationship IDs (`rId1`, `rId2`, …) to target part paths. Missing or
//! incorrect `.rels` files cause broken hyperlinks, missing layouts, and
//! "repair required" dialogs.
//!
//! [`RelsBuilder`] constructs the XML for a single `.rels` file. Callers
//! instantiate one builder per `.rels` file, add relationships in logical
//! order, then call [`RelsBuilder::build`] to obtain the serialised bytes.
//!
//! ## Relationship ID convention (AC-009)
//!
//! IDs are sequential `rId1`, `rId2`, … per `.rels` file, assigned in
//! the order relationships are added. The OOXML spec requires IDs to be
//! unique within a single `.rels` file but imposes no global uniqueness
//! constraint.

use ooxmlsdk::schemas::opc_relationships::{Relationship, Relationships, TargetMode};

use crate::error::PptxError;

/// A single relationship entry for a `.rels` file.
///
/// Corresponds to one `<Relationship>` element in the XML.
pub struct RelEntry {
    /// The relationship ID within this `.rels` file (e.g., `"rId1"`).
    pub id: String,
    /// The OPC relationship type URI.
    pub rel_type: String,
    /// Target part path, relative to the owning part's directory.
    pub target: String,
}

/// Builds the XML for a single `.rels` file.
///
/// Relationships are recorded in the order they are added; the final XML
/// emits them in that same order so output is deterministic.
pub struct RelsBuilder {
    /// Accumulated relationships.
    relationships: Vec<RelEntry>,
}

impl RelsBuilder {
    /// Create an empty builder for one `.rels` file.
    #[must_use]
    pub fn new() -> Self {
        Self {
            relationships: Vec::new(),
        }
    }

    /// Add a relationship and return the assigned `rId` string.
    ///
    /// IDs are assigned sequentially: first call returns `"rId1"`, second
    /// returns `"rId2"`, and so on.
    pub fn add(&mut self, rel_type: impl Into<String>, target: impl Into<String>) -> String {
        let id = format!("rId{}", self.relationships.len() + 1);
        self.relationships.push(RelEntry {
            id: id.clone(),
            rel_type: rel_type.into(),
            target: target.into(),
        });
        id
    }

    /// Add an external hyperlink relationship and return the assigned `rId` string.
    ///
    /// External hyperlinks require `TargetMode="External"` in the `.rels` XML so
    /// that consuming applications treat the target as an absolute URL rather than
    /// a relative package path. This is the correct mechanism for `<a:hlinkClick>`
    /// relationships in slide and notesSlide XML.
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::OoxmlElement`] if the internal entry cannot be pushed
    /// (in practice this never fails — the error path is for API consistency).
    pub fn add_external_hyperlink(&mut self, url: impl Into<String>) -> String {
        let id = format!("rId{}", self.relationships.len() + 1);
        self.relationships.push(RelEntry {
            id: id.clone(),
            rel_type: rel_types::HYPERLINK.to_string(),
            target: format!("EXTERNAL:{}", url.into()), // sentinel for build()
        });
        id
    }

    /// Serialise all relationships into `.rels` XML bytes.
    ///
    /// Uses the ooxmlsdk `Relationships` type for schema-correct serialisation.
    ///
    /// Entries whose target starts with `"EXTERNAL:"` (added via
    /// [`add_external_hyperlink`][RelsBuilder::add_external_hyperlink]) are
    /// emitted with `TargetMode="External"` and the sentinel prefix stripped.
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::OoxmlElement`] if serialisation fails.
    pub fn build(self) -> Result<Vec<u8>, PptxError> {
        let mut rels = Relationships {
            xmlns: vec![ooxmlsdk::common::XmlNamespaceDecl::new(
                "",
                "http://schemas.openxmlformats.org/package/2006/relationships",
            )],
            ..Relationships::default()
        };

        for entry in self.relationships {
            let (target_mode, actual_target) =
                if let Some(url) = entry.target.strip_prefix("EXTERNAL:") {
                    (Some(TargetMode::External), url.to_string())
                } else {
                    (None, entry.target)
                };
            rels.relationship.push(Relationship {
                id: entry.id,
                r#type: entry.rel_type,
                target: actual_target,
                target_mode,
            });
        }

        rels.to_xml_bytes().map_err(|e| PptxError::OoxmlElement {
            part: ".rels".to_string(),
            detail: e.to_string(),
        })
    }
}

impl Default for RelsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Well-known `OPC` / `PresentationML` relationship type URIs.
///
/// Using named constants avoids typos in long URI strings across the codebase.
pub mod rel_types {
    /// Root `.rels` → `presentation.xml`.
    pub const OFFICE_DOCUMENT: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";

    /// `presentation.xml` → `slideMaster1.xml`.
    pub const SLIDE_MASTER: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster";

    /// `slideMaster1.xml` → `slideLayout*.xml`.
    pub const SLIDE_LAYOUT: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout";

    /// `presentation.xml` → `slide*.xml`.
    pub const SLIDE: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide";

    /// `slideLayout*.xml` → `slideMaster1.xml`.
    pub const SLIDE_MASTER_FROM_LAYOUT: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster";

    /// Any part → `theme1.xml`.
    pub const THEME: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme";

    /// `presentation.xml` → `notesMaster1.xml`.
    pub const NOTES_MASTER: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesMaster";

    /// `presentation.xml` → `handoutMaster1.xml`.
    pub const HANDOUT_MASTER: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/handoutMaster";

    /// Any part → `core.xml` (Dublin Core metadata).
    pub const CORE_PROPERTIES: &str =
        "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties";

    /// Any part → `app.xml` (extended properties).
    pub const EXTENDED_PROPERTIES: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties";

    /// `slide{N}.xml` → `notesSlide{N}.xml` (slide's own notes part).
    ///
    /// This is the SLIDE-side relationship (F-040-P1-001): each slide that has
    /// non-empty speaker notes MUST reference its notesSlide via its own `.rels`
    /// file at `ppt/slides/_rels/slide{N}.xml.rels`. Without this relationship
    /// `PowerPoint` cannot discover the slide's notes — the notesSlide→slide
    /// back-rel alone is insufficient.
    pub const NOTES_SLIDE: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide";

    /// Slide → image/media part.
    pub const IMAGE: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";

    /// External hyperlink target.
    ///
    /// Used in `<a:hlinkClick>` runs via `RelsBuilder::add_external_hyperlink`.
    /// The corresponding `.rels` entry must have `TargetMode="External"`.
    pub const HYPERLINK: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";
}
