//! DOCX-exporter error type.
//!
//! [`ExportError`] is the crate-level error enum returned by
//! [`crate::DocxExporter`] when DOCX output cannot be produced. It is defined
//! here as a data type and is fully defined (no business logic).

use thiserror::Error;

/// Errors that can be returned by the DOCX exporter.
///
/// `#[non_exhaustive]` allows adding variants in minor releases without
/// breaking downstream code. Mirrors the shape of
/// [`slideforge_plugin_api::ExportError`] but is crate-local for richer
/// DOCX-specific context.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ExportError {
    /// A ZIP archive I/O error occurred while assembling the `.docx` file.
    #[error("docx zip error: {message}")]
    ZipError {
        /// Description of the ZIP failure.
        message: String,
    },

    /// An OOXML element construction error occurred while building Word XML parts.
    #[error("docx ooxml error: {message}")]
    OoxmlError {
        /// Description of the OOXML construction failure.
        message: String,
    },

    /// The target output directory or path is not accessible.
    #[error("docx output path error: {message}")]
    OutputPathError {
        /// Description of the path/I/O failure.
        message: String,
    },

    /// The [`slideforge_layout::LaidOutDeck`] provided to the exporter is
    /// invalid or incompatible with DOCX output requirements.
    #[error("docx validation error: {message}")]
    ValidationError {
        /// Description of the validation failure.
        message: String,
    },
}

impl From<ExportError> for slideforge_plugin_api::ExportError {
    fn from(e: ExportError) -> Self {
        match e {
            ExportError::ZipError { message } | ExportError::OutputPathError { message } => {
                slideforge_plugin_api::ExportError::IoError { message }
            },
            ExportError::OoxmlError { message } => {
                slideforge_plugin_api::ExportError::RenderError { message }
            },
            ExportError::ValidationError { message } => {
                slideforge_plugin_api::ExportError::ValidationError { message }
            },
        }
    }
}
