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

use ooxmlsdk::schemas::opc_content_types::{Default as CtDefault, Override, Types, TypesChoice};

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
    /// Media parts: (path, mime_type) pairs.
    media_parts: Vec<(String, String)>,
}

impl ContentTypesBuilder {
    /// Create a new builder with zero parts registered.
    #[must_use]
    pub fn new() -> Self {
        Self {
            slide_count: 0,
            layout_count: 0,
            media_parts: Vec::new(),
        }
    }

    /// Register one slide part (`ppt/slides/slide{n}.xml`).
    pub fn add_slide(&mut self) {
        self.slide_count += 1;
    }

    /// Register one layout part (`ppt/slideLayouts/slideLayout{n}.xml`).
    pub fn add_layout(&mut self) {
        self.layout_count += 1;
    }

    /// Register a media part (SVG or PNG image in `ppt/media/`).
    ///
    /// `path` is the part path inside the ZIP (e.g., `"ppt/media/image1.svg"`).
    /// `content_type` is the MIME type string (e.g., `"image/svg+xml"`).
    pub fn add_media(&mut self, path: impl Into<String>, content_type: impl Into<String>) {
        self.media_parts.push((path.into(), content_type.into()));
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
        let mut types = Types::default();
        types.xmlns = vec![ooxmlsdk::common::XmlNamespaceDecl::new(
            "",
            "http://schemas.openxmlformats.org/package/2006/content-types",
        )];

        // --- Default entries (extension-level) ---
        types.types_choice.push(TypesChoice::Default(Box::new(CtDefault {
            extension: "rels".to_string(),
            content_type: "application/vnd.openxmlformats-package.relationships+xml".to_string(),
        })));
        types.types_choice.push(TypesChoice::Default(Box::new(CtDefault {
            extension: "xml".to_string(),
            content_type: "application/xml".to_string(),
        })));

        // --- Override entries (sorted alphabetically) ---
        // Collect all override (path, content_type) pairs for sorting.
        let mut overrides: Vec<(String, String)> = Vec::new();

        // presentation.xml
        overrides.push((
            "/ppt/presentation.xml".to_string(),
            "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"
                .to_string(),
        ));

        // slides (1..=n)
        for n in 1..=self.slide_count {
            overrides.push((
                format!("/ppt/slides/slide{n}.xml"),
                "application/vnd.openxmlformats-officedocument.presentationml.slide+xml"
                    .to_string(),
            ));
        }

        // slideLayouts (1..=m)
        for n in 1..=self.layout_count {
            overrides.push((
                format!("/ppt/slideLayouts/slideLayout{n}.xml"),
                "application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"
                    .to_string(),
            ));
        }

        // slideMaster
        overrides.push((
            "/ppt/slideMasters/slideMaster1.xml".to_string(),
            "application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"
                .to_string(),
        ));

        // theme
        overrides.push((
            "/ppt/theme/theme1.xml".to_string(),
            "application/vnd.openxmlformats-officedocument.theme+xml".to_string(),
        ));

        // notesMaster
        overrides.push((
            "/ppt/notesMasters/notesMaster1.xml".to_string(),
            "application/vnd.openxmlformats-officedocument.presentationml.notesMaster+xml"
                .to_string(),
        ));

        // handoutMaster
        overrides.push((
            "/ppt/handoutMasters/handoutMaster1.xml".to_string(),
            "application/vnd.openxmlformats-officedocument.presentationml.handoutMaster+xml"
                .to_string(),
        ));

        // docProps/core.xml
        overrides.push((
            "/docProps/core.xml".to_string(),
            "application/vnd.openxmlformats-package.core-properties+xml".to_string(),
        ));

        // docProps/app.xml
        overrides.push((
            "/docProps/app.xml".to_string(),
            "application/vnd.openxmlformats-officedocument.extended-properties+xml".to_string(),
        ));

        // media parts
        for (path, ct) in &self.media_parts {
            overrides.push((format!("/{path}"), ct.clone()));
        }

        // Sort alphabetically for determinism.
        overrides.sort_by(|a, b| a.0.cmp(&b.0));

        for (part_name, content_type) in overrides {
            types.types_choice.push(TypesChoice::Override(Box::new(Override {
                part_name,
                content_type,
            })));
        }

        types
            .to_xml_bytes()
            .map_err(|e| PptxError::OoxmlElement {
                part: "[Content_Types].xml".to_string(),
                detail: e.to_string(),
            })
    }
}

impl Default for ContentTypesBuilder {
    fn default() -> Self {
        Self::new()
    }
}
