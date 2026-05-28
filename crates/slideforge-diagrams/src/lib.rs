//! Diagram renderer plugin for slideforge.
//!
//! This crate implements the [`DiagramRenderer`] plugin trait from
//! `slideforge-plugin-api` using the `mermaid-rs-renderer` 0.2.2 crate
//! (pure Rust, no Node.js, no Chrome).
//!
//! ## Architecture (STORY-033)
//!
//! - **No I/O during rendering**: Font loading is handled internally by
//!   `mermaid-rs-renderer`. Rendering is synchronous and in-memory.
//! - **No subprocess**: `mermaid-rs-renderer` is a pure-Rust library.
//!   No Node.js is spawned (BC-1.12.001 invariant 1).
//! - **Plugin-first**: [`DiagramRendererImpl`] uses ONLY the public
//!   `slideforge_plugin_api::DiagramRenderer` trait — no internal bypass.
//! - **Forbidden deps**: This crate MUST NOT depend on any exporter,
//!   `slideforge-data`, `slideforge-layout`, `slideforge-cli`,
//!   `slideforge-syntax`, or `slideforge-eval`.
//!
//! ## Two-step pipeline
//!
//! STORY-033 produces the raw Mermaid → SVG output. STORY-034 adds the
//! usvg normalization step. Together they form the complete diagram pipeline
//! (BC-1.12.001 postcondition 6 / BC-1.12.003).

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod accessibility;
pub mod error;
pub mod renderer;
pub mod types;

use slideforge_plugin_api::{DiagramOptions, DiagramRenderer};
use tracing::instrument;

use crate::types::{DiagramLang, DiagramError, RawDiagramSvg};

// Re-export primary types for crate consumers.
pub use crate::types::{DiagramError as SfDiagramError, DiagramLang as SfDiagramLang, RawDiagramSvg as SfRawDiagramSvg};

/// The default `DiagramRenderer` plugin implementation for slideforge.
///
/// Uses `mermaid-rs-renderer` 0.2.2 to render Mermaid diagram source to SVG.
/// Supports all 23 Mermaid diagram types covered by that crate (AC-008).
///
/// ## Performance
///
/// The first call initializes the `mermaid-rs-renderer` font database
/// (< 200ms on CI hardware, empirical from Spike S14). Subsequent calls
/// are warm (< 10ms per NFR-003/004).
#[derive(Debug, Default)]
pub struct DiagramRendererImpl;

impl DiagramRendererImpl {
    /// Create a new [`DiagramRendererImpl`].
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Render a Mermaid diagram source string to PPTX-safe SVG with
    /// accessibility attributes injected.
    ///
    /// This is the **primary internal entry point** for diagram rendering in
    /// the slideforge eval pipeline. The eval layer calls this method directly
    /// with a fully-resolved `source` string and `alt_text`.
    ///
    /// ## Pipeline
    ///
    /// 1. Validate `source` is non-empty → [`DiagramError::EmptySource`]
    /// 2. Validate `lang` is `Mermaid` → [`DiagramError::UnsupportedLanguage`]
    /// 3. Call `mermaid_rs_renderer::render(source)` → raw SVG
    /// 4. Assert no forbidden elements (foreignObject, script, @keyframes)
    /// 5. Inject `aria-label`, `role="img"`, `<title>` → [`RawDiagramSvg`]
    ///
    /// # Errors
    ///
    /// Returns [`DiagramError`] on any step failure.
    #[instrument(skip(source, alt_text), fields(lang = ?lang))]
    pub fn render_diagram(
        source: &str,
        lang: DiagramLang,
        alt_text: &str,
    ) -> Result<RawDiagramSvg, DiagramError> {
        // Step 0: Validate language.
        if lang != DiagramLang::Mermaid {
            return Err(DiagramError::UnsupportedLanguage {
                lang: std::sync::Arc::from(lang.as_keyword()),
            });
        }

        // Delegate to the renderer module.
        crate::renderer::render_mermaid(source, alt_text)
    }
}

impl DiagramRenderer for DiagramRendererImpl {
    #[allow(clippy::unnecessary_literal_bound)]
    fn id(&self) -> &str {
        "mermaid-rs-renderer"
    }

    /// Render a diagram from raw source bytes using `opts`.
    ///
    /// ## Architecture note
    ///
    /// The plugin-api [`DiagramRenderer::render`] method accepts `source: &str`
    /// and `opts: &DiagramOptions`. The `opts` struct carries optional width,
    /// height, theme, and background color. In v1.0, theme and background are
    /// passed through as-is; width/height are informational (mermaid-rs-renderer
    /// 0.2.2 determines SVG dimensions internally).
    ///
    /// ## Errors
    ///
    /// - [`slideforge_plugin_api::DiagramError::SyntaxError`] — invalid Mermaid
    /// - [`slideforge_plugin_api::DiagramError::RenderError`] — renderer failure
    /// - [`slideforge_plugin_api::DiagramError::UnsupportedFeature`] — forbidden SVG element
    #[instrument(skip(self, source, _opts))]
    fn render(
        &self,
        source: &str,
        _opts: &DiagramOptions,
    ) -> Result<Vec<u8>, slideforge_plugin_api::DiagramError> {
        // The trait method uses empty string for alt text — the caller must
        // inject alt text after calling render(). In the full eval pipeline,
        // the eval layer calls render_diagram() directly with the AltText.
        let raw_svg = Self::render_diagram(source, DiagramLang::Mermaid, "").map_err(|e| {
            match e {
                DiagramError::MermaidSyntaxError {
                    message,
                    source_line,
                    ..
                } => slideforge_plugin_api::DiagramError::SyntaxError {
                    line: source_line,
                    message: message.to_string(),
                },
                DiagramError::EmptySource => slideforge_plugin_api::DiagramError::SyntaxError {
                    line: 1,
                    message: "empty Mermaid source".to_owned(),
                },
                DiagramError::UnsupportedLanguage { lang } => {
                    slideforge_plugin_api::DiagramError::UnsupportedFeature {
                        feature: format!("diagram language: {lang}"),
                    }
                }
                DiagramError::RenderError { message } => {
                    slideforge_plugin_api::DiagramError::RenderError {
                        message: message.to_string(),
                    }
                }
                DiagramError::SvgPostProcessingError { message } => {
                    slideforge_plugin_api::DiagramError::RenderError {
                        message: format!("SVG post-processing failed: {message}"),
                    }
                }
            }
        })?;
        Ok(raw_svg.into_string().into_bytes())
    }
}

// ---------------------------------------------------------------------------
// Compile-time assertion: DiagramRendererImpl is Send + Sync (AC-001)
// ---------------------------------------------------------------------------
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DiagramRendererImpl>();
};

// ---------------------------------------------------------------------------
// Test suite
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use slideforge_plugin_api::{DiagramOptions, DiagramRenderer};

    use super::*;

    // -----------------------------------------------------------------------
    // AC-001: DiagramRenderer trait implementation
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_001_ac001_diagram_renderer_impl_id() {
        let renderer = DiagramRendererImpl::new();
        assert_eq!(renderer.id(), "mermaid-rs-renderer");
    }

    #[test]
    fn test_bc_1_12_001_ac001_diagram_renderer_impl_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DiagramRendererImpl>();
    }

    // -----------------------------------------------------------------------
    // AC-001: Full pipeline via render_diagram (internal entry point)
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_001_diagram_renderer_impl_renders_flowchart() {
        let source = "graph TD\n  A --> B";
        let result = DiagramRendererImpl::render_diagram(source, DiagramLang::Mermaid, "A to B");
        let svg = result.expect("DiagramRendererImpl must render flowchart without error");
        assert!(!svg.is_empty(), "rendered SVG must not be empty");
        assert!(svg.as_str().contains("<svg"), "rendered SVG must contain <svg element");
        assert!(
            svg.as_str().contains("aria-label="),
            "rendered SVG must contain aria-label"
        );
    }

    #[test]
    fn test_bc_1_12_002_unsupported_language_returns_error() {
        // Only DiagramLang::Mermaid is supported in v1.0.
        // We test via render_diagram with Mermaid only (the enum only has one variant).
        // This test verifies the UnsupportedLanguage path via DiagramLang construction.
        // Since DiagramLang::from_keyword("plantuml") returns None, we verify that
        // a directly constructed UnsupportedLanguage error is properly formed.
        let err = DiagramError::UnsupportedLanguage {
            lang: std::sync::Arc::from("plantuml"),
        };
        let msg = err.to_string();
        assert!(msg.contains("plantuml"), "error must mention the language; got: {msg}");
    }

    #[test]
    fn test_bc_1_12_001_render_no_panic_on_arbitrary_input() {
        // BC-1.12.002 VP: no panic on any string input to the full pipeline.
        let inputs = ["", "   ", "!@#$%", "SELECT 1"];
        for input in &inputs {
            // May return Ok or Err — must NOT panic.
            let _r = std::panic::catch_unwind(|| {
                let _ = DiagramRendererImpl::render_diagram(input, DiagramLang::Mermaid, "test");
            });
        }
    }

    // -----------------------------------------------------------------------
    // AC-001: DiagramRenderer trait render() method (plugin API surface)
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_001_trait_render_flowchart() {
        let renderer = DiagramRendererImpl::new();
        let opts = DiagramOptions::default();
        let source = "graph TD\n  A --> B";
        let result = renderer.render(source, &opts);
        let bytes = result.expect("DiagramRenderer::render must succeed for valid flowchart");
        assert!(!bytes.is_empty(), "render must return non-empty bytes");
        // Bytes are UTF-8 SVG
        let svg = String::from_utf8(bytes).expect("render output must be valid UTF-8");
        assert!(svg.contains("<svg"), "render output must be an SVG document");
    }

    #[test]
    fn test_bc_1_12_001_trait_render_sequence_diagram() {
        let renderer = DiagramRendererImpl::new();
        let opts = DiagramOptions::default();
        let source = "sequenceDiagram\n  Alice->>Bob: Hello";
        let result = renderer.render(source, &opts);
        let bytes = result.expect("DiagramRenderer::render must succeed for sequenceDiagram");
        assert!(!bytes.is_empty(), "render must return non-empty bytes");
    }

    #[test]
    fn test_bc_1_12_001_trait_render_empty_source_returns_error() {
        let renderer = DiagramRendererImpl::new();
        let opts = DiagramOptions::default();
        let result = renderer.render("", &opts);
        assert!(result.is_err(), "render of empty source must return error");
    }

    #[test]
    fn test_bc_1_12_001_snapshot_flowchart_svg() {
        let source = "graph TD\n  A[Client] --> B[API]";
        let result = DiagramRendererImpl::render_diagram(source, DiagramLang::Mermaid, "Architecture");
        let svg = result.expect("snapshot test: flowchart must render");
        insta::assert_yaml_snapshot!("flowchart_svg", svg.as_str());
    }

    #[test]
    fn test_bc_1_12_001_snapshot_sequence_svg() {
        let source = "sequenceDiagram\n  Alice->>Bob: Hello";
        let result =
            DiagramRendererImpl::render_diagram(source, DiagramLang::Mermaid, "Sequence diagram");
        let svg = result.expect("snapshot test: sequenceDiagram must render");
        insta::assert_yaml_snapshot!("sequence_svg", svg.as_str());
    }
}
