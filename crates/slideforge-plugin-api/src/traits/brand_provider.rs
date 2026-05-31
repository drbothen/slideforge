//! [`BrandProvider`] trait — loads brand configuration from a source.
//!
//! A `BrandProvider` plugin resolves a [`BrandSource`] (a `.toml` brand file,
//! an existing `.pptx` template, or a `.docx` template) and returns a resolved
//! [`Brand`] configuration. The built-in provider handles all three source types.
//! External plugins can register providers for proprietary brand systems.

use std::sync::Arc;

use slideforge_types::Brand;
use thiserror::Error;

/// The source from which brand configuration is loaded.
///
/// Brand can be synthesized from a TOML file, extracted from an existing PPTX
/// template, or extracted from a DOCX template. The bidirectional bridge is
/// described in the planning decisions (Q4 Brand decision).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BrandSource {
    /// Load brand from an existing `.pptx` template file.
    ///
    /// The provider extracts the theme colors, fonts, and layout definitions
    /// from the OOXML theme part of the PPTX file.
    PptxFile(Arc<str>),

    /// Load brand from an existing `.docx` template file.
    ///
    /// The provider extracts the theme and style definitions from the OOXML
    /// theme part of the DOCX file.
    DocxFile(Arc<str>),

    /// Load brand from a slideforge brand TOML file.
    ///
    /// The TOML file uses the slideforge brand schema (colors, fonts, layout
    /// geometry). This is the primary brand authoring format.
    TomlFile(Arc<str>),
}

impl std::fmt::Display for BrandSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrandSource::PptxFile(path) => write!(f, "pptx:{path}"),
            BrandSource::DocxFile(path) => write!(f, "docx:{path}"),
            BrandSource::TomlFile(path) => write!(f, "toml:{path}"),
        }
    }
}

/// Error returned by [`BrandProvider::load`].
///
/// `#[non_exhaustive]` allows adding variants in minor releases without
/// breaking downstream plugin authors or consumer crates.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BrandError {
    /// The source file does not exist or cannot be opened.
    #[error("brand source not found: {uri}")]
    SourceNotFound {
        /// Description of the source that was not found.
        uri: String,
    },

    /// The source file exists but its content is invalid or cannot be parsed.
    #[error("brand source parse error for '{uri}': {message}")]
    ParseError {
        /// The source URI that failed to parse.
        uri: String,
        /// Description of the parse failure.
        message: String,
    },

    /// The brand configuration is valid syntactically but fails semantic
    /// validation (e.g., a required color is missing, an invalid hex value).
    #[error("brand validation error: {message}")]
    ValidationError {
        /// Description of the semantic validation failure.
        message: String,
    },

    /// An I/O error occurred while reading the source.
    #[error("brand I/O error for '{uri}': {message}")]
    IoError {
        /// The source URI that produced the I/O error.
        uri: String,
        /// Description of the I/O failure.
        message: String,
    },
}

/// A plugin that loads a [`Brand`] from a [`BrandSource`].
///
/// The resolved [`Brand`] is passed to layout and export plugins. The registry
/// lookup returns the first registered provider; all built-in source types are
/// handled by the single built-in provider.
///
/// Register implementations with [`crate::PluginRegistry::register_brand_provider`].
///
/// ## Thread safety
///
/// All implementations must be `Send + Sync`.
pub trait BrandProvider: Send + Sync {
    /// A unique identifier for this brand provider plugin (e.g., `"default"`).
    fn id(&self) -> &str;

    /// Load a [`Brand`] from `source`.
    ///
    /// # Errors
    ///
    /// Returns [`BrandError`] when the source cannot be read or parsed.
    fn load(&self, source: &BrandSource) -> Result<Brand, BrandError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync + ?Sized>() {}

    #[test]
    fn test_bc_5_02_007_brand_provider_trait_is_send_sync() {
        assert_send_sync::<dyn BrandProvider>();
    }

    #[test]
    fn test_bc_5_02_007_brand_source_pptx_file_display() {
        let src = BrandSource::PptxFile(Arc::from("template.pptx"));
        assert_eq!(src.to_string(), "pptx:template.pptx");
    }

    #[test]
    fn test_bc_5_02_007_brand_source_docx_file_display() {
        let src = BrandSource::DocxFile(Arc::from("template.docx"));
        assert_eq!(src.to_string(), "docx:template.docx");
    }

    #[test]
    fn test_bc_5_02_007_brand_source_toml_file_display() {
        let src = BrandSource::TomlFile(Arc::from("brand.toml"));
        assert_eq!(src.to_string(), "toml:brand.toml");
    }

    #[test]
    fn test_bc_5_02_007_brand_source_eq() {
        let a = BrandSource::TomlFile(Arc::from("brand.toml"));
        let b = BrandSource::TomlFile(Arc::from("brand.toml"));
        assert_eq!(a, b);
    }

    #[test]
    fn test_bc_5_02_007_brand_source_ne_different_variant() {
        let a = BrandSource::TomlFile(Arc::from("brand.toml"));
        let b = BrandSource::PptxFile(Arc::from("brand.toml"));
        assert_ne!(a, b);
    }

    #[test]
    fn test_bc_5_02_007_brand_error_source_not_found() {
        let err = BrandError::SourceNotFound {
            uri: "brand.toml".to_owned(),
        };
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_bc_5_02_007_brand_error_parse_error() {
        let err = BrandError::ParseError {
            uri: "brand.toml".to_owned(),
            message: "invalid hex color".to_owned(),
        };
        assert!(err.to_string().contains("parse error"));
    }

    #[test]
    fn test_bc_5_02_007_brand_error_validation_error() {
        let err = BrandError::ValidationError {
            message: "primary color is required".to_owned(),
        };
        assert!(err.to_string().contains("validation error"));
    }

    #[test]
    fn test_bc_5_02_007_brand_error_io_error() {
        let err = BrandError::IoError {
            uri: "brand.toml".to_owned(),
            message: "permission denied".to_owned(),
        };
        assert!(err.to_string().contains("I/O error"));
    }
}
