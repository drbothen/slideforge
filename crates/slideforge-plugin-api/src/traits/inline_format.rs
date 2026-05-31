//! [`InlineFormat`] trait — renders an [`InlineNode`] to a target format string.
//!
//! An `InlineFormat` plugin takes an [`InlineNode`] (e.g., `Bold`, `Italic`,
//! `Code`) and a target [`InlineOutputFormat`] and produces the appropriate
//! string representation:
//! - [`InlineOutputFormat::Ooxml`] — OOXML `<a:r>` runs (for PPTX/DOCX)
//! - [`InlineOutputFormat::Html`] — HTML element string (e.g., `<strong>...</strong>`)
//! - [`InlineOutputFormat::Markdown`] — Markdown notation (e.g., `**bold**`)
//!
//! The built-in formatter handles all 12 `InlineNode` variants across all three
//! output formats. External plugins can override formatting for specific inline
//! types or add new output formats.

use slideforge_types::InlineNode;
use thiserror::Error;

/// The output format for an inline formatting operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InlineOutputFormat {
    /// OOXML run markup (for PPTX and DOCX exporters). The output is a valid
    /// OOXML XML fragment (e.g., `<a:r><a:rPr b="1"/><a:t>bold</a:t></a:r>`).
    Ooxml,

    /// HTML element string (for the HTML exporter and web preview). The output
    /// is a valid HTML fragment (e.g., `<strong>bold</strong>`).
    Html,

    /// Markdown notation (for the DOCX report-mode exporter and debugging).
    /// The output uses `CommonMark` syntax (e.g., `**bold**`, `` `code` ``).
    Markdown,
}

impl std::fmt::Display for InlineOutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InlineOutputFormat::Ooxml => write!(f, "OOXML"),
            InlineOutputFormat::Html => write!(f, "HTML"),
            InlineOutputFormat::Markdown => write!(f, "Markdown"),
        }
    }
}

/// Error returned by [`InlineFormat::render`].
///
/// `#[non_exhaustive]` allows adding variants in minor releases without
/// breaking downstream plugin authors or consumer crates.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum InlineError {
    /// The [`InlineNode`] variant is not supported by this formatter for the
    /// requested output format.
    #[error("unsupported inline node '{node_kind}' for format '{format}'")]
    UnsupportedNode {
        /// The `kind_name()` of the inline node.
        node_kind: String,
        /// The requested output format.
        format: InlineOutputFormat,
    },

    /// The inline node's content is invalid or cannot be serialized.
    #[error("inline render error for '{node_kind}': {message}")]
    RenderError {
        /// The `kind_name()` of the inline node.
        node_kind: String,
        /// Description of the rendering failure.
        message: String,
    },
}

/// A plugin that renders an [`InlineNode`] to a format-specific string.
///
/// Each exporter calls the inline formatter to convert inline nodes to the
/// appropriate format-specific markup. The registry returns the first registered
/// formatter; the built-in formatter handles all cases.
///
/// Register implementations with [`crate::PluginRegistry::register_inline_format`].
///
/// ## Thread safety
///
/// All implementations must be `Send + Sync`.
pub trait InlineFormat: Send + Sync {
    /// A unique identifier for this inline format plugin (e.g., `"default"`).
    fn id(&self) -> &str;

    /// Render `node` to a string in the specified `format`.
    ///
    /// # Errors
    ///
    /// Returns [`InlineError`] when the node type is not supported in the
    /// requested format, or when rendering fails internally.
    fn render(&self, node: &InlineNode, format: InlineOutputFormat) -> Result<String, InlineError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync + ?Sized>() {}

    #[test]
    fn test_bc_5_02_010_inline_format_trait_is_send_sync() {
        assert_send_sync::<dyn InlineFormat>();
    }

    #[test]
    fn test_bc_5_02_010_inline_output_format_display() {
        assert_eq!(InlineOutputFormat::Ooxml.to_string(), "OOXML");
        assert_eq!(InlineOutputFormat::Html.to_string(), "HTML");
        assert_eq!(InlineOutputFormat::Markdown.to_string(), "Markdown");
    }

    #[test]
    fn test_bc_5_02_010_inline_output_format_eq() {
        assert_eq!(InlineOutputFormat::Ooxml, InlineOutputFormat::Ooxml);
        assert_ne!(InlineOutputFormat::Ooxml, InlineOutputFormat::Html);
        assert_ne!(InlineOutputFormat::Html, InlineOutputFormat::Markdown);
    }

    #[test]
    fn test_bc_5_02_010_inline_output_format_copy() {
        let fmt = InlineOutputFormat::Html;
        let fmt2 = fmt; // Copy — no move
        let fmt3 = fmt; // Still usable
        assert_eq!(fmt2, InlineOutputFormat::Html);
        assert_eq!(fmt3, InlineOutputFormat::Html);
    }

    #[test]
    fn test_bc_5_02_010_inline_error_unsupported_node() {
        let err = InlineError::UnsupportedNode {
            node_kind: "Footnote".to_owned(),
            format: InlineOutputFormat::Ooxml,
        };
        assert!(err.to_string().contains("unsupported inline node"));
        assert!(err.to_string().contains("Footnote"));
    }

    #[test]
    fn test_bc_5_02_010_inline_error_render_error() {
        let err = InlineError::RenderError {
            node_kind: "Math".to_owned(),
            message: "LaTeX source is empty".to_owned(),
        };
        assert!(err.to_string().contains("inline render error"));
    }
}
