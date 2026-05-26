//! [`DiagramRenderer`] trait — renders a diagram source string to SVG bytes.
//!
//! A `DiagramRenderer` plugin takes raw diagram source (e.g., Mermaid syntax)
//! and options and produces an SVG byte buffer. The built-in renderer targets
//! Mermaid (the exact engine — native Rust port vs. Node.js subprocess — is
//! determined by S14 spike in planning). External plugins can register renderers
//! for other diagram languages (e.g., `PlantUML`, Graphviz DOT).

use thiserror::Error;

/// Options passed to [`DiagramRenderer::render`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct DiagramOptions {
    /// Requested output width in pixels. `None` means use the renderer default.
    pub width_px: Option<u32>,

    /// Requested output height in pixels. `None` means use the renderer default.
    pub height_px: Option<u32>,

    /// The diagram theme to apply (e.g., `"default"`, `"dark"`, `"forest"`).
    /// Interpretation is renderer-specific.
    pub theme: Option<std::sync::Arc<str>>,

    /// The background color as a CSS color string (e.g., `"transparent"`,
    /// `"#ffffff"`). `None` means the renderer default.
    pub background: Option<std::sync::Arc<str>>,
}

/// Error returned by [`DiagramRenderer::render`].
#[derive(Debug, Error)]
pub enum DiagramError {
    /// The diagram source contains a syntax error.
    #[error("diagram syntax error at line {line}: {message}")]
    SyntaxError {
        /// The source line where the error was detected.
        line: u32,
        /// Description of the syntax error.
        message: String,
    },

    /// The renderer does not support the diagram language or feature used in
    /// the source.
    #[error("unsupported diagram feature: {feature}")]
    UnsupportedFeature {
        /// The feature that is not supported.
        feature: String,
    },

    /// An internal rendering error occurred.
    #[error("diagram rendering failed: {message}")]
    RenderError {
        /// Description of the rendering failure.
        message: String,
    },

    /// An external tool required by the renderer (e.g., a Node.js subprocess)
    /// is not available or failed to start.
    #[error("diagram renderer dependency unavailable: {tool}")]
    DependencyUnavailable {
        /// Name of the unavailable tool or dependency.
        tool: String,
    },
}

/// A plugin that renders a diagram source string to SVG bytes.
///
/// The SVG is embedded in all output formats: PPTX (as an image), HTML
/// (inline), and PDF (as vector graphics). The SVG must be self-contained.
///
/// Register implementations with [`crate::PluginRegistry::register_diagram_renderer`].
///
/// ## Thread safety
///
/// All implementations must be `Send + Sync`.
pub trait DiagramRenderer: Send + Sync {
    /// A unique identifier for this diagram renderer plugin (e.g., `"mermaid"`).
    fn id(&self) -> &str;

    /// Render `source` to SVG bytes using `opts`.
    ///
    /// # Errors
    ///
    /// Returns [`DiagramError`] when rendering fails.
    fn render(&self, source: &str, opts: &DiagramOptions) -> Result<Vec<u8>, DiagramError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync + ?Sized>() {}

    #[test]
    fn test_bc_5_02_004_diagram_renderer_trait_is_send_sync() {
        assert_send_sync::<dyn DiagramRenderer>();
    }

    #[test]
    fn test_bc_5_02_004_diagram_options_default() {
        let opts = DiagramOptions::default();
        assert!(opts.width_px.is_none());
        assert!(opts.height_px.is_none());
        assert!(opts.theme.is_none());
        assert!(opts.background.is_none());
    }

    #[test]
    fn test_bc_5_02_004_diagram_error_syntax_error() {
        let err = DiagramError::SyntaxError {
            line: 3,
            message: "unexpected token".to_owned(),
        };
        assert!(err.to_string().contains("syntax error"));
    }

    #[test]
    fn test_bc_5_02_004_diagram_error_unsupported_feature() {
        let err = DiagramError::UnsupportedFeature {
            feature: "subgraph nesting".to_owned(),
        };
        assert!(err.to_string().contains("unsupported diagram feature"));
    }

    #[test]
    fn test_bc_5_02_004_diagram_error_render_error() {
        let err = DiagramError::RenderError {
            message: "SVG serialization failed".to_owned(),
        };
        assert!(err.to_string().contains("diagram rendering failed"));
    }

    #[test]
    fn test_bc_5_02_004_diagram_error_dependency_unavailable() {
        let err = DiagramError::DependencyUnavailable {
            tool: "mermaid-cli".to_owned(),
        };
        assert!(err.to_string().contains("dependency unavailable"));
    }
}
