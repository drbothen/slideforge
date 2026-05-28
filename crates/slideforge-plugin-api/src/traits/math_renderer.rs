//! [`MathRenderer`] trait — renders a [`MathNode`] (LaTeX) to a math output format.
//!
//! A `MathRenderer` plugin takes a [`MathNode`] (which carries the LaTeX source)
//! and a target [`MathOutputFormat`] and produces the appropriate bytes:
//! - [`MathOutputFormat::Omml`] — Office Open Math Markup Language (for PPTX/DOCX)
//! - [`MathOutputFormat::MathMl`] — `MathML` Core Level 1 (for HTML)
//! - [`MathOutputFormat::Pdf`] — PDF vector-path SVG (for PDF embedding)
//!
//! The built-in renderer (`slideforge-math`) is a hand-written LaTeX subset
//! parser with renderers for OMML, `MathML`, and PDF vector-path SVG. External
//! plugins can substitute or augment the math rendering pipeline.

use slideforge_types::MathNode;
use thiserror::Error;

/// The output format for a math rendering operation.
///
/// Each exporter requests the format appropriate for its output target:
/// PPTX/DOCX exporters request `Omml`; HTML exporters request `MathMl`;
/// PDF exporters request `Pdf`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathOutputFormat {
    /// Office Open Math Markup Language, the native math format for OOXML
    /// (PPTX and DOCX). Bytes are valid OMML XML.
    Omml,

    /// `MathML` Core Level 1 (`http://www.w3.org/1998/Math/MathML`), the W3C
    /// standard math format for HTML. Bytes are valid UTF-8 `MathML` XML.
    MathMl,

    /// PDF vector-path SVG bytes — all glyphs converted to `<path>` outlines,
    /// no `<text>` elements. Implemented in STORY-030 by `pdf_paths::render_pdf_paths`.
    Pdf,
}

impl std::fmt::Display for MathOutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MathOutputFormat::Omml => write!(f, "OMML"),
            MathOutputFormat::MathMl => write!(f, "MathML"),
            MathOutputFormat::Pdf => write!(f, "PDF"),
        }
    }
}

/// Error returned by [`MathRenderer::render`].
#[derive(Debug, Error)]
pub enum MathError {
    /// The LaTeX source in the [`MathNode`] contains a syntax error.
    #[error("LaTeX syntax error: {message}")]
    SyntaxError {
        /// Description of the syntax error.
        message: String,
    },

    /// The requested [`MathOutputFormat`] is not supported by this renderer.
    #[error("unsupported math output format: {format}")]
    UnsupportedFormat {
        /// The requested format that is not supported.
        format: MathOutputFormat,
    },

    /// An internal rendering error occurred.
    #[error("math rendering failed: {message}")]
    RenderError {
        /// Description of the rendering failure.
        message: String,
    },

    /// PDF path output still contains `<text>` elements after SVG normalisation.
    ///
    /// Returned by `render_pdf_paths` (STORY-030) when `usvg` normalisation
    /// cannot remove all `<text>` elements from the intermediate SVG, indicating
    /// that the glyph outline engine did not fully convert a character to paths.
    /// Callers should emit `E-EXP-004` in response (EC-005).
    #[error("PDF path output contains residual <text> elements after SVG normalisation")]
    TextRemainsInPathOutput,

    /// A Greek, operator, or symbol command name was empty (zero-length string).
    ///
    /// Returned by the PDF path renderer when `MathNode::Greek`, `Operator`, or
    /// `Symbol` carries an empty `Arc<str>`.  An empty command name has no
    /// defined Unicode mapping and cannot produce a glyph (EC-005).
    #[error("math command name is empty — cannot resolve Unicode glyph")]
    EmptyCommandName,

    /// A command name was not found in the Unicode symbol tables.
    ///
    /// The PDF path renderer only supports commands that have a canonical
    /// Unicode mapping.  For text-based operators (`lim`, `max`, …) the Latin
    /// characters are passed through directly; for everything else an explicit
    /// mapping must exist in `symbols.rs`.
    #[error("unsupported math symbol: \\{name}")]
    UnsupportedSymbol {
        /// The command name that has no Unicode mapping.
        name: String,
    },

    /// The [`crate::MathAst`] passed to `render_pdf_paths` contains zero nodes.
    ///
    /// An empty AST cannot produce meaningful vector-path output. Callers should
    /// surface this as a user-visible error rather than silently emitting a
    /// degenerate 1×16-px SVG (finding F-S030-P3-L2).
    #[error("math expression is empty — no nodes to render")]
    EmptyAst,
}

/// A plugin that renders a [`MathNode`] to bytes in a target math format.
///
/// Math rendering is output-format-specific: PPTX/DOCX require OMML; HTML
/// requires `MathML` Core Level 1; PDF requires vector-path SVG. The renderer is
/// called once per math node per output format.
///
/// Register implementations with [`crate::PluginRegistry::register_math_renderer`].
///
/// ## Thread safety
///
/// All implementations must be `Send + Sync`.
pub trait MathRenderer: Send + Sync {
    /// A unique identifier for this math renderer (e.g., `"slideforge-builtin"`).
    fn id(&self) -> &str;

    /// Render `node` to bytes in the specified `format`.
    ///
    /// # Errors
    ///
    /// Returns [`MathError`] when the LaTeX source is invalid or the format is
    /// not supported.
    fn render(&self, node: &MathNode, format: MathOutputFormat) -> Result<Vec<u8>, MathError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync + ?Sized>() {}

    #[test]
    fn test_bc_5_02_006_math_renderer_trait_is_send_sync() {
        assert_send_sync::<dyn MathRenderer>();
    }

    #[test]
    fn test_bc_5_02_006_math_output_format_display() {
        assert_eq!(MathOutputFormat::Omml.to_string(), "OMML");
        assert_eq!(MathOutputFormat::MathMl.to_string(), "MathML");
        assert_eq!(MathOutputFormat::Pdf.to_string(), "PDF");
    }

    #[test]
    fn test_bc_5_02_006_math_output_format_eq() {
        assert_eq!(MathOutputFormat::Omml, MathOutputFormat::Omml);
        assert_ne!(MathOutputFormat::Omml, MathOutputFormat::MathMl);
        assert_ne!(MathOutputFormat::MathMl, MathOutputFormat::Pdf);
    }

    #[test]
    fn test_bc_5_02_006_math_output_format_copy() {
        let fmt = MathOutputFormat::Omml;
        let fmt2 = fmt; // Copy — no move
        let fmt3 = fmt; // Still usable
        assert_eq!(fmt2, MathOutputFormat::Omml);
        assert_eq!(fmt3, MathOutputFormat::Omml);
    }

    #[test]
    fn test_bc_5_02_006_math_error_syntax_error() {
        let err = MathError::SyntaxError {
            message: "unmatched brace".to_owned(),
        };
        assert!(err.to_string().contains("syntax error"));
    }

    #[test]
    fn test_bc_5_02_006_math_error_unsupported_format() {
        let err = MathError::UnsupportedFormat {
            format: MathOutputFormat::Pdf,
        };
        assert!(err.to_string().contains("unsupported math output format"));
    }

    #[test]
    fn test_bc_5_02_006_math_error_render_error() {
        let err = MathError::RenderError {
            message: "KaTeX internal failure".to_owned(),
        };
        assert!(err.to_string().contains("math rendering failed"));
    }
}
