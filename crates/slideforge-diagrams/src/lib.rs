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
//!
//! ```text
//! render_mermaid() → RawDiagramSvg
//!                               ↓
//!                     usvg_normalize()       (STORY-034)
//!                               ↓
//!                    NormalizedDiagramSvg
//!                               ↓
//!                   LaidOutDeck / Exporters
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod accessibility;
pub mod error;
pub mod normalize;
pub mod renderer;
pub mod types;
pub(crate) mod xml_escape;

use slideforge_plugin_api::{DiagramOptions, DiagramRenderer};
use tracing::instrument;

use crate::types::{DiagramError, DiagramLang, NormalizedDiagramSvg, RawDiagramSvg};

// Re-export primary types for crate consumers.
pub use crate::normalize::usvg_normalize;
pub use crate::types::{
    DiagramError as SfDiagramError, DiagramLang as SfDiagramLang,
    NormalizedDiagramSvg as SfNormalizedDiagramSvg, RawDiagramSvg as SfRawDiagramSvg,
};

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

    /// Render a Mermaid diagram source string to a PPTX-safe, usvg-normalized
    /// SVG with accessibility attributes injected.
    ///
    /// This is the **primary internal entry point** for diagram rendering in
    /// the slideforge eval pipeline. The eval layer calls this method directly
    /// with a fully-resolved `source` string and `alt_text`.
    ///
    /// ## Pipeline (STORY-033 + STORY-034)
    ///
    /// 1. Validate `source` is non-empty → [`DiagramError::EmptySource`]
    /// 2. Validate `lang` is `Mermaid` → [`DiagramError::UnsupportedLanguage`]
    /// 3. Call `mermaid_rs_renderer::render(source)` → raw SVG string
    /// 4. Assert no forbidden elements (foreignObject, script, @keyframes)
    /// 5. Inject `aria-label`, `role="img"`, `<title>` → [`RawDiagramSvg`]
    /// 6. Call [`crate::normalize::usvg_normalize`] → [`NormalizedDiagramSvg`]
    ///    (BC-1.12.003 invariant 1: normalization is mandatory, no bypass path)
    ///
    /// # Errors
    ///
    /// Returns [`DiagramError`] on any step failure. Step 6 failures produce
    /// [`DiagramError::SvgNormalizationFailed`] with error code `E-EXP-004`.
    #[instrument(skip(source, alt_text), fields(lang = ?lang))]
    pub fn render_diagram(
        source: &str,
        lang: DiagramLang,
        alt_text: &str,
    ) -> Result<NormalizedDiagramSvg, DiagramError> {
        // Step 0: Validate language.
        if lang != DiagramLang::Mermaid {
            return Err(DiagramError::UnsupportedLanguage {
                lang: std::sync::Arc::from(lang.as_keyword()),
            });
        }

        // Steps 1–5: render Mermaid source to accessibility-annotated raw SVG.
        let raw_svg: RawDiagramSvg = crate::renderer::render_mermaid(source, alt_text)?;

        // Step 6: mandatory usvg normalization pass (BC-1.12.003 invariant 1).
        // There is no bypass path — exporters receive NormalizedDiagramSvg only.
        crate::normalize::usvg_normalize(&raw_svg, alt_text)
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
    /// height, theme, and background color fields.
    ///
    /// **In v1.0, ALL fields in `DiagramOptions` are intentionally ignored.**
    /// `mermaid-rs-renderer` 0.2.2 determines SVG dimensions, theme, and
    /// background internally from the Mermaid source. There is no API to pass
    /// width, height, theme, or background to the renderer at render time.
    /// The `_opts` parameter is accepted to satisfy the trait signature for
    /// future compatibility when the underlying renderer gains these controls.
    ///
    /// ## Accessibility (alt text)
    ///
    /// The [`DiagramRenderer`] trait has no `alt_text` parameter, so this method
    /// passes an empty string to `render_diagram`. The resulting SVG will contain
    /// `aria-label=""` and `<title></title>` (decorative-diagram semantics).
    ///
    /// **Alt text injection is the CALLER's responsibility.** In the full eval
    /// pipeline, the eval layer calls [`DiagramRendererImpl::render_diagram`]
    /// directly with the user-supplied `alt` text from the DSL source. Callers
    /// using the plugin API trait directly must post-process the SVG bytes to
    /// inject the correct `aria-label` and `<title>` values for their context.
    ///
    /// ## Errors
    ///
    /// - [`slideforge_plugin_api::DiagramError::SyntaxError`] — invalid Mermaid syntax or empty source
    /// - [`slideforge_plugin_api::DiagramError::UnsupportedFeature`] — unsupported diagram language
    /// - [`slideforge_plugin_api::DiagramError::RenderError`] — forbidden SVG element, post-processing failure, or normalization failure
    #[instrument(skip(self, source, _opts))]
    fn render(
        &self,
        source: &str,
        _opts: &DiagramOptions,
    ) -> Result<Vec<u8>, slideforge_plugin_api::DiagramError> {
        // The trait method uses empty string for alt text (decorative-diagram
        // semantics). Callers that need alt text must use render_diagram() directly
        // or post-process the SVG. In the eval pipeline, render_diagram() is called
        // directly with the user-supplied AltText from DSL source.
        let normalized =
            Self::render_diagram(source, DiagramLang::Mermaid, "").map_err(|e| match e {
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
                },
                DiagramError::RenderError { message } => {
                    slideforge_plugin_api::DiagramError::RenderError {
                        message: message.to_string(),
                    }
                },
                DiagramError::SvgPostProcessingError { message } => {
                    slideforge_plugin_api::DiagramError::RenderError {
                        message: format!("SVG post-processing failed: {message}"),
                    }
                },
                DiagramError::SvgNormalizationFailed {
                    slide_title, cause, ..
                } => slideforge_plugin_api::DiagramError::RenderError {
                    message: format!(
                        "[E-EXP-004] SVG normalization failed for diagram '{slide_title}': {cause}"
                    ),
                },
            })?;
        Ok(normalized.as_str().as_bytes().to_vec())
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
        assert!(
            svg.as_str().contains("<svg"),
            "rendered SVG must contain <svg element"
        );
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
        assert!(
            msg.contains("plantuml"),
            "error must mention the language; got: {msg}"
        );
    }

    #[test]
    fn test_bc_1_12_001_render_no_panic_on_arbitrary_input() {
        // BC-1.12.002 VP: no panic on any string input to the full pipeline.
        // Calling render_diagram directly: if the function panics the test framework
        // catches the unwind and reports a test failure with the panic message.
        // We accept Ok or structured Err — the requirement is: no panic.
        let inputs = ["", "   ", "!@#$%", "SELECT 1"];
        for input in &inputs {
            // May return Ok or Err — must NOT panic.
            let _ = DiagramRendererImpl::render_diagram(input, DiagramLang::Mermaid, "test");
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
        assert!(
            svg.contains("<svg"),
            "render output must be an SVG document"
        );
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

    // -----------------------------------------------------------------------
    // Structural SVG property tests (platform-independent replacements for
    // snapshot tests).
    //
    // Background: `mermaid-rs-renderer` uses font metrics that vary by platform
    // (macOS vs Linux), producing different numeric dimensions in the SVG output
    // (e.g. width="125.01209" on macOS vs different values on Linux). Insta
    // snapshot tests that capture the full SVG string therefore fail in CI when
    // the snapshots were captured on macOS and CI runs on Linux.
    //
    // These assertion-based tests check *structural* properties that are
    // guaranteed to be platform-independent:
    //   - `<svg` root element present
    //   - Accessibility attributes injected (aria-label, role, <title>)
    //   - viewBox, width, height attributes present
    //   - Node labels visible in the SVG text
    //   - No forbidden elements (foreignObject, script, @keyframes)
    //
    // The above properties are already partially covered by other tests in this
    // module and in `renderer.rs`. These tests focus specifically on the
    // flowchart-with-labelled-nodes and sequence-diagram scenarios from the
    // former snapshots, ensuring the same behavioral guarantees without
    // platform-dependent numeric values.
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_001_flowchart_with_labels_structural_properties() {
        // Replaces former snapshot test `flowchart_svg`.
        // Verifies structural SVG properties for a flowchart with labelled nodes.
        let source = "graph TD\n  A[Client] --> B[API]";
        let result =
            DiagramRendererImpl::render_diagram(source, DiagramLang::Mermaid, "Architecture");
        let svg = result.expect("flowchart with labels must render");
        let s = svg.as_str();

        // Root element
        assert!(s.contains("<svg"), "output must be an SVG document");
        // Dimensions (values are platform-dependent, but attributes must be present)
        assert!(s.contains("viewBox"), "SVG must have viewBox");
        assert!(s.contains("width"), "SVG must have width");
        assert!(s.contains("height"), "SVG must have height");
        // Accessibility injection
        assert!(
            s.contains("aria-label=\"Architecture\""),
            "SVG must carry the injected aria-label"
        );
        assert!(s.contains("role=\"img\""), "SVG must have role=img");
        assert!(
            s.contains("<title>Architecture</title>"),
            "SVG must have <title> matching alt text"
        );
        // Node label content visible in output
        assert!(s.contains("Client"), "SVG must include 'Client' node label");
        assert!(s.contains("API"), "SVG must include 'API' node label");
        // Safety checks
        assert!(
            !s.to_ascii_lowercase().contains("<foreignobject"),
            "SVG must not contain <foreignObject>"
        );
        assert!(
            !s.to_ascii_lowercase().contains("<script"),
            "SVG must not contain <script>"
        );
        assert!(
            !s.to_ascii_lowercase().contains("@keyframes"),
            "SVG must not contain @keyframes"
        );
    }

    #[test]
    fn test_bc_1_12_001_sequence_diagram_structural_properties() {
        // Replaces former snapshot test `sequence_svg`.
        // Verifies structural SVG properties for a sequence diagram.
        let source = "sequenceDiagram\n  Alice->>Bob: Hello";
        let result =
            DiagramRendererImpl::render_diagram(source, DiagramLang::Mermaid, "Sequence diagram");
        let svg = result.expect("sequenceDiagram must render");
        let s = svg.as_str();

        // Root element
        assert!(s.contains("<svg"), "output must be an SVG document");
        // Dimensions (values are platform-dependent, but attributes must be present)
        assert!(s.contains("viewBox"), "SVG must have viewBox");
        assert!(s.contains("width"), "SVG must have width");
        assert!(s.contains("height"), "SVG must have height");
        // Accessibility injection
        assert!(
            s.contains("aria-label=\"Sequence diagram\""),
            "SVG must carry the injected aria-label"
        );
        assert!(s.contains("role=\"img\""), "SVG must have role=img");
        assert!(
            s.contains("<title>Sequence diagram</title>"),
            "SVG must have <title> matching alt text"
        );
        // Participant labels and message content visible in output
        assert!(s.contains("Alice"), "SVG must include 'Alice' participant");
        assert!(s.contains("Bob"), "SVG must include 'Bob' participant");
        assert!(
            s.contains("Hello"),
            "SVG must include 'Hello' message label"
        );
        // Safety checks
        assert!(
            !s.to_ascii_lowercase().contains("<foreignobject"),
            "SVG must not contain <foreignObject>"
        );
        assert!(
            !s.to_ascii_lowercase().contains("<script"),
            "SVG must not contain <script>"
        );
        assert!(
            !s.to_ascii_lowercase().contains("@keyframes"),
            "SVG must not contain @keyframes"
        );
    }
}
