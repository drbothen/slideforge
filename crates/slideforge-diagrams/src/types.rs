//! Core types for `slideforge-diagrams`.
//!
//! Defines [`DiagramLang`], [`RawDiagramSvg`], [`NormalizedDiagramSvg`], and
//! [`DiagramError`].

use std::sync::Arc;

use miette::SourceSpan;
use thiserror::Error;

/// The diagram source language.
///
/// Only `Mermaid` is supported in v1.0. Additional languages (`PlantUML`, Graphviz
/// DOT) may be added via the plugin system in future versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagramLang {
    /// Mermaid diagram syntax — rendered via `mermaid-rs-renderer` 0.2.2.
    Mermaid,
}

impl DiagramLang {
    /// Parse a language keyword string into a [`DiagramLang`] variant.
    ///
    /// Returns `None` for unsupported language keywords.
    #[must_use]
    pub fn from_keyword(kw: &str) -> Option<Self> {
        match kw {
            "mermaid" | "Mermaid" => Some(Self::Mermaid),
            _ => None,
        }
    }

    /// Return the canonical keyword string for this diagram language.
    #[must_use]
    pub fn as_keyword(&self) -> &'static str {
        match self {
            Self::Mermaid => "mermaid",
        }
    }
}

/// A newtype wrapping the raw SVG string produced by the diagram renderer.
///
/// This is the output of `DiagramRendererImpl::render_diagram()` *before*
/// usvg normalization (STORY-034). Accessibility attributes (`aria-label`,
/// `role="img"`, and a `<title>` child element) are injected by
/// [`crate::accessibility::inject_aria_attributes`] before wrapping.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawDiagramSvg(pub String);

impl RawDiagramSvg {
    /// Consume the newtype and return the inner SVG string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }

    /// Return a reference to the inner SVG string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Return `true` if the SVG string is non-empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A newtype wrapping the usvg-normalized SVG string produced by
/// [`crate::normalize::usvg_normalize`].
///
/// This is the output of the mandatory normalization pass (STORY-034, BC-1.12.003)
/// that runs after every successful [`RawDiagramSvg`] is produced. The normalized
/// form is guaranteed to be PPTX-safe:
///
/// - No `<foreignObject>` elements (removed by usvg).
/// - No `<script>` elements (removed by usvg).
/// - No CSS `@keyframes` or class-based `<style>` blocks (inlined by usvg).
/// - Absolute pixel `width` and `height` on the root element (computed from
///   `viewBox` by usvg when the source SVG uses percentage dimensions).
/// - No `<use>` elements (all `href="#symbol"` references inlined by usvg).
///
/// The type system enforces that exporters receive only [`NormalizedDiagramSvg`],
/// never [`RawDiagramSvg`]. Passing raw SVG to an exporter is a compile error.
///
/// ## IR compatibility
///
/// `NormalizedDiagramSvg` uses `Arc<str>` (not `String`) so that cloning is
/// cheap and the type satisfies `Hash + Eq + Clone` for comemo compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedDiagramSvg(pub Arc<str>);

impl NormalizedDiagramSvg {
    /// Return a reference to the inner SVG string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Return `true` if the SVG string is non-empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Error produced by [`crate::DiagramRendererImpl`].
///
/// All variants carry an `error_code` field that maps to the error taxonomy.
/// `MermaidSyntaxError` maps to **E-EXP-008** per BC-1.12.002.
///
/// ## Warn-only mode (AC-007)
///
/// The `--warn-only` flag (AC-007) enables a mode where diagram render failures
/// produce an error-slide placeholder instead of aborting the build. This
/// behavior is intentionally **NOT implemented in this crate**. The
/// `slideforge-diagrams` crate always returns a structured `DiagramError` on
/// failure. The caller — the eval layer (`slideforge-eval`) — inspects the
/// error and decides whether to abort or substitute an `ErrorSlidePlaceholder`
/// based on the current build mode. This separation preserves the single
/// responsibility of the diagram renderer (render or fail) and the eval layer
/// (apply build policy). The eval layer does not exist in STORY-033 scope; it
/// will be implemented in the wave that adds `slideforge-eval` functionality.
#[derive(Debug, Error)]
pub enum DiagramError {
    /// The Mermaid source contains a syntax error.
    ///
    /// Maps to **E-EXP-008**: "Diagram render failed … at source line N".
    #[error("[{error_code}] Diagram render failed: {message} at source line {source_line}")]
    MermaidSyntaxError {
        /// Human-readable error description from the renderer.
        message: Arc<str>,
        /// 1-based line number within the Mermaid source block.
        source_line: u32,
        /// Error code (always `"E-EXP-008"`).
        error_code: &'static str,
    },

    /// The diagram source language is not supported by this renderer.
    ///
    /// Only `Mermaid` is supported in v1.0.
    #[error("unsupported diagram language: {lang}")]
    UnsupportedLanguage {
        /// The language keyword that was requested.
        lang: Arc<str>,
    },

    /// An internal rendering error occurred (not a Mermaid syntax error).
    ///
    /// Maps to **E-EXP-008** generic render failure.
    #[error("diagram rendering failed: {message}")]
    RenderError {
        /// Description of the rendering failure.
        message: Arc<str>,
    },

    /// The diagram source is empty (zero-length or whitespace-only).
    ///
    /// Maps to **E-EXP-008** with "at source line 1".
    #[error("E-EXP-008: Diagram render failed: empty Mermaid source at source line 1")]
    EmptySource,

    /// An SVG post-processing error occurred (e.g., aria injection failure).
    #[error("diagram SVG post-processing failed: {message}")]
    SvgPostProcessingError {
        /// Description of the post-processing failure.
        message: Arc<str>,
    },

    /// usvg normalization failed on the raw SVG produced by the renderer.
    ///
    /// Maps to **E-EXP-004**: the diagram SVG could not be normalized into
    /// PPTX-safe form because usvg rejected the input as malformed SVG. This
    /// is not a user-authored syntax error (the user writes Mermaid source) —
    /// it is a rendering pipeline failure where the underlying renderer produced
    /// SVG that usvg cannot parse.
    ///
    /// In strict mode: exit code 3.
    /// In warn-only mode: error-slide placeholder substituted.
    ///
    /// The `source_id` field carries the diagram identifier (e.g., slide title
    /// or diagram label) to aid debugging. The `span` field carries the byte
    /// offset within the raw SVG where the parse failure occurred (if available
    /// from usvg; otherwise a zero-length span at offset 0 is used).
    #[error("[E-EXP-004] SVG normalization failed for diagram '{source_id}': {cause}")]
    SvgNormalizationFailed {
        /// Identifier of the diagram whose SVG could not be normalized
        /// (e.g., the slide title or diagram alt-text label).
        source_id: Arc<str>,
        /// Human-readable description of the usvg parse/normalization failure.
        cause: Arc<str>,
        /// Byte offset span within the raw SVG where the failure was detected.
        ///
        /// Set to a zero-length span at offset 0 when usvg does not provide
        /// positional information about the failure.
        span: SourceSpan,
    },
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // DiagramLang tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_001_diagram_lang_from_keyword_mermaid() {
        assert_eq!(
            DiagramLang::from_keyword("mermaid"),
            Some(DiagramLang::Mermaid)
        );
        assert_eq!(
            DiagramLang::from_keyword("Mermaid"),
            Some(DiagramLang::Mermaid)
        );
    }

    #[test]
    fn test_bc_1_12_001_diagram_lang_from_keyword_unknown_returns_none() {
        assert_eq!(DiagramLang::from_keyword("plantuml"), None);
        assert_eq!(DiagramLang::from_keyword("dot"), None);
        assert_eq!(DiagramLang::from_keyword(""), None);
    }

    #[test]
    fn test_bc_1_12_001_diagram_lang_as_keyword_roundtrip() {
        let lang = DiagramLang::Mermaid;
        let kw = lang.as_keyword();
        assert_eq!(DiagramLang::from_keyword(kw), Some(DiagramLang::Mermaid));
    }

    // -----------------------------------------------------------------------
    // RawDiagramSvg newtype tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_001_raw_diagram_svg_into_string() {
        let svg = RawDiagramSvg("<svg/>".to_owned());
        assert_eq!(svg.into_string(), "<svg/>");
    }

    #[test]
    fn test_bc_1_12_001_raw_diagram_svg_as_str() {
        let svg = RawDiagramSvg("<svg/>".to_owned());
        assert_eq!(svg.as_str(), "<svg/>");
    }

    #[test]
    fn test_bc_1_12_001_raw_diagram_svg_is_empty_false_for_nonempty() {
        let svg = RawDiagramSvg("<svg/>".to_owned());
        assert!(!svg.is_empty());
    }

    #[test]
    fn test_bc_1_12_001_raw_diagram_svg_is_empty_true_for_empty() {
        let svg = RawDiagramSvg(String::new());
        assert!(svg.is_empty());
    }

    // -----------------------------------------------------------------------
    // DiagramError display tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_002_diagram_error_syntax_error_message_contains_e_exp_008() {
        let err = DiagramError::MermaidSyntaxError {
            message: Arc::from("Undefined node 'B'"),
            source_line: 3,
            error_code: "E-EXP-008",
        };
        let msg = err.to_string();
        assert!(
            msg.contains("E-EXP-008"),
            "error must contain E-EXP-008; got: {msg}"
        );
        assert!(
            msg.contains("source line 3"),
            "error must contain source line 3; got: {msg}"
        );
        assert!(
            msg.contains("Undefined node 'B'"),
            "error must contain error message; got: {msg}"
        );
    }

    #[test]
    fn test_bc_1_12_002_diagram_error_empty_source_message() {
        let err = DiagramError::EmptySource;
        let msg = err.to_string();
        assert!(
            msg.contains("E-EXP-008"),
            "EmptySource error must contain E-EXP-008; got: {msg}"
        );
        assert!(
            msg.contains("source line 1"),
            "EmptySource error must mention source line 1; got: {msg}"
        );
        assert!(
            msg.contains("empty"),
            "EmptySource error must contain 'empty'; got: {msg}"
        );
    }

    #[test]
    fn test_bc_1_12_002_diagram_error_unsupported_language_message() {
        let err = DiagramError::UnsupportedLanguage {
            lang: Arc::from("plantuml"),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("plantuml"),
            "error must mention the language; got: {msg}"
        );
        assert!(
            msg.contains("unsupported"),
            "error must say 'unsupported'; got: {msg}"
        );
    }

    #[test]
    fn test_bc_1_12_002_diagram_error_render_error_message() {
        let err = DiagramError::RenderError {
            message: Arc::from("backend failure"),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("backend failure"),
            "error must contain message; got: {msg}"
        );
        assert!(
            msg.contains("failed"),
            "error must say 'failed'; got: {msg}"
        );
    }

    #[test]
    fn test_bc_1_12_002_diagram_error_svg_post_processing_error_message() {
        let err = DiagramError::SvgPostProcessingError {
            message: Arc::from("no <svg> element"),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("no <svg> element"),
            "error must contain message; got: {msg}"
        );
    }
}
