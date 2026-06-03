//! Error types for the `slideforge-pptx` crate.
//!
//! [`PptxError`] covers all failure modes in PPTX ZIP assembly, OOXML element
//! construction, and relationship chain building. It is distinct from the
//! plugin-level [`slideforge_plugin_api::ExportError`] — the exporter converts
//! `PptxError` into `ExportError` at the trait boundary.

use thiserror::Error;

/// All error conditions that can occur during PPTX serialization.
///
/// Each variant carries enough context for the exporter to surface a useful
/// `ExportError` message to the CLI layer. No variant is `#[non_exhaustive]`
/// because this is an internal type — only `slideforge-pptx` itself constructs
/// these values.
#[derive(Debug, Error)]
pub enum PptxError {
    /// Failed to write a byte stream to the in-memory ZIP archive.
    #[error("zip I/O error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// An OOXML element required by the PPTX spec is missing or malformed.
    ///
    /// `part` identifies the ZIP part (e.g., `"ppt/slides/slide1.xml"`)
    /// and `detail` describes the specific failure.
    #[error("OOXML element error in {part}: {detail}")]
    OoxmlElement {
        /// The ZIP part path where the error occurred.
        part: String,
        /// A description of the element error.
        detail: String,
    },

    /// A relationship reference (`r:id`) could not be resolved in a `.rels` file.
    ///
    /// `rels_path` is the path of the `.rels` file, `rid` is the unresolved id.
    #[error("unresolved relationship r:id={rid} in {rels_path}")]
    UnresolvedRelationship {
        /// Path of the `.rels` file that contained the unresolved reference.
        rels_path: String,
        /// The relationship id that could not be resolved.
        rid: String,
    },

    /// The `BrandTemplate` supplied to the exporter carries no slide layouts, so
    /// the required 31 slide layouts cannot be embedded into the PPTX archive.
    ///
    /// This error is returned by [`crate::layout_embedder::LayoutEmbedder::embed`]
    /// when `brand_template.layouts.is_empty()`. A correctly synthesised brand
    /// (via `brand_template_from_brand`) always produces exactly 31 layouts, so
    /// this variant is a hard invariant guard rather than a normal error path.
    ///
    /// `part` is `"layouts"` in this context.
    #[error("brand is missing required PPTX part: {part}")]
    MissingBrandPart {
        /// Name of the missing brand part.
        ///
        /// For the empty-layouts guard this is `"layouts"`.
        /// For other potential missing-part errors, this is the descriptive part name
        /// (e.g., `"master_xml"`).
        part: String,
    },

    /// The `LaidOutDeck` contained an EMU coordinate value that is invalid for
    /// the PPTX exporter (e.g., a negative size dimension).
    #[error("invalid EMU coordinate in slide {slide_index}, frame {frame_index}: {detail}")]
    InvalidEmu {
        /// Zero-based index of the slide containing the invalid coordinate.
        slide_index: usize,
        /// Zero-based index of the frame within the slide.
        frame_index: usize,
        /// A description of the validation failure.
        detail: String,
    },

    /// An I/O error occurred while writing to the output sink.
    #[error("output I/O error: {0}")]
    Io(#[from] std::io::Error),
}
