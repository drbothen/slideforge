//! [`Exporter`] trait — produces output format bytes from the slideforge IR.
//!
//! An `Exporter` plugin takes the semantic [`Deck`] and geometric [`LaidOutDeck`]
//! IRs together with the resolved [`Brand`] and produces a byte buffer in the
//! target format (e.g., `.pptx`, `.docx`, `.pdf`, `.html`). Built-in exporters
//! cover all five v1.0 output formats. External plugins can add more.

use slideforge_layout::LaidOutDeck;
use slideforge_types::{Brand, Deck};
use thiserror::Error;

/// Options that control [`Exporter::export`] behavior.
///
/// These options apply to a single export invocation. Plugin-level defaults can
/// be configured in the brand file or slideforge workspace config.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ExportOptions {
    /// When `true`, the exporter operates in strict mode: any validation warning
    /// is treated as an error and export fails.
    pub strict: bool,

    /// Optional output path hint. Most exporters ignore this (the CLI handles
    /// path routing), but some (e.g., the HTML exporter with asset embedding)
    /// may use it to resolve relative asset paths.
    pub output_path_hint: Option<std::sync::Arc<str>>,

    /// When `true`, embed all assets (fonts, images) inline in the output.
    /// Interpretation is format-specific.
    pub embed_assets: bool,
}

/// Error returned by [`Exporter::export`] when output cannot be produced.
///
/// `#[non_exhaustive]` allows adding variants in minor releases without
/// breaking downstream plugin authors or consumer crates.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ExportError {
    /// The export operation failed due to an I/O error (e.g., cannot read a
    /// font file, cannot write to output stream).
    #[error("export I/O error: {message}")]
    IoError {
        /// A description of the I/O failure.
        message: String,
    },

    /// The deck content is invalid for this output format (e.g., a slide type
    /// that the format does not support).
    #[error("export validation error: {message}")]
    ValidationError {
        /// A description of the validation failure.
        message: String,
    },

    /// A required dependency (e.g., a font, a template, an embedded tool) is
    /// missing or not available.
    #[error("export dependency missing: {dependency}")]
    DependencyMissing {
        /// Name of the missing dependency.
        dependency: String,
    },

    /// An internal rendering error occurred inside the exporter plugin.
    #[error("export rendering error: {message}")]
    RenderError {
        /// A description of the rendering failure.
        message: String,
    },
}

/// A plugin that produces an output format from the slideforge IR.
///
/// Implement this trait to add a new output format (e.g., a Beamer LaTeX
/// exporter, a reveal.js HTML exporter, or a custom proprietary format).
/// Register with [`crate::PluginRegistry::register_exporter`].
///
/// ## Thread safety
///
/// All implementations must be `Send + Sync`.
///
/// ## Example
///
/// ```rust
/// use slideforge_plugin_api::{Exporter, ExportError, ExportOptions};
/// use slideforge_layout::LaidOutDeck;
/// use slideforge_types::{Brand, Deck};
///
/// struct NullExporter;
///
/// impl Exporter for NullExporter {
///     fn id(&self) -> &str { "null" }
///     fn extension(&self) -> &str { "bin" }
///     fn export(
///         &self,
///         _deck: &Deck,
///         _laid_out: &LaidOutDeck,
///         _brand: &Brand,
///         _opts: &ExportOptions,
///     ) -> Result<Vec<u8>, ExportError> {
///         Ok(vec![])
///     }
/// }
/// ```
pub trait Exporter: Send + Sync {
    /// A unique identifier for this exporter plugin (e.g., `"pptx"`, `"pdf"`).
    fn id(&self) -> &str;

    /// The default file extension for this format, without the leading dot
    /// (e.g., `"pptx"`, `"html"`, `"pdf"`).
    fn extension(&self) -> &str;

    /// Produce output format bytes from the IR.
    ///
    /// # Parameters
    ///
    /// - `deck` — the semantic, pre-layout IR
    /// - `laid_out` — the geometric, post-layout IR
    /// - `brand` — resolved brand configuration
    /// - `opts` — per-export options
    ///
    /// # Errors
    ///
    /// Returns [`ExportError`] when the output cannot be produced.
    fn export(
        &self,
        deck: &Deck,
        laid_out: &LaidOutDeck,
        brand: &Brand,
        opts: &ExportOptions,
    ) -> Result<Vec<u8>, ExportError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync + ?Sized>() {}

    #[test]
    fn test_bc_5_02_002_exporter_trait_is_send_sync() {
        assert_send_sync::<dyn Exporter>();
    }

    #[test]
    fn test_bc_5_02_002_export_options_default() {
        let opts = ExportOptions::default();
        assert!(!opts.strict);
        assert!(opts.output_path_hint.is_none());
        assert!(!opts.embed_assets);
    }

    #[test]
    fn test_bc_5_02_002_export_error_io_error() {
        let err = ExportError::IoError {
            message: "disk full".to_owned(),
        };
        assert!(err.to_string().contains("I/O error"));
    }

    #[test]
    fn test_bc_5_02_002_export_error_validation_error() {
        let err = ExportError::ValidationError {
            message: "unsupported slide type".to_owned(),
        };
        assert!(err.to_string().contains("validation error"));
    }

    #[test]
    fn test_bc_5_02_002_export_error_dependency_missing() {
        let err = ExportError::DependencyMissing {
            dependency: "Calibri Light".to_owned(),
        };
        assert!(err.to_string().contains("dependency missing"));
    }

    #[test]
    fn test_bc_5_02_002_export_error_render_error() {
        let err = ExportError::RenderError {
            message: "SVG serialization failed".to_owned(),
        };
        assert!(err.to_string().contains("rendering error"));
    }
}
