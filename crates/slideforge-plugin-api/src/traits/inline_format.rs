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

// ─────────────────────────────────────────────────────────────────────────────
// InlineRenderContext
// ─────────────────────────────────────────────────────────────────────────────

/// Context passed to [`InlineFormat::render_with_context`] for runs that need
/// exporter-owned package state — specifically, the pre-registered relationship
/// ID for a hyperlink.
///
/// This struct is passed by reference and is zero-cost when unused: the default
/// implementation of `render_with_context` ignores it and delegates to `render`.
///
/// ## OOXML hyperlinks (F-001 / AC-005 / BC-5.02.002)
///
/// OOXML hyperlinks require a relationship ID (`rId`) that must be pre-registered
/// in the `.rels` file by the exporter before the XML body is serialized.
/// The `InlineFormat::render` signature has no mechanism to receive this state,
/// so exporters that need proper hyperlink runs must call `render_with_context`
/// and supply the pre-registered `rId` via `hyperlink_rid`.
///
/// - Non-OOXML formats (HTML, Markdown) set `hyperlink_rid = None`; the
///   formatter ignores it and produces native link markup.
/// - OOXML exporters without a relationship manager set `hyperlink_rid = None`;
///   the formatter falls back to the display-text-plus-warn behavior from `render`.
#[derive(Debug, Default, Clone, Copy)]
pub struct InlineRenderContext<'a> {
    /// Relationship ID (`rId`) pre-registered by the exporter for a `Link` node's URL.
    ///
    /// - `Some(rid)` — emit a full `<a:r><a:rPr><a:hlinkClick r:id="{rid}"/>` run.
    /// - `None` — fall back to the display-text plain-run behavior from `render`.
    ///
    /// Only meaningful for OOXML output; HTML and Markdown formatters must ignore
    /// this field.
    pub hyperlink_rid: Option<&'a str>,
}

/// The output format for an inline formatting operation.
///
/// `#[non_exhaustive]` — external [`InlineFormat`] plugins pattern-match
/// this enum in their `render` implementations. Adding a new output format
/// (e.g., `LaTeX`, `PlainText`) is a minor-release addition; the attribute
/// prevents those plugins from silently ignoring the new variant. Plugin
/// authors must add a wildcard arm that returns [`InlineError::UnsupportedNode`]
/// or a format-specific error rather than silently falling back.
#[non_exhaustive]
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
        // This match is exhaustive within the defining crate. The `#[non_exhaustive]`
        // attribute only forces wildcard arms in EXTERNAL crates that match this enum.
        // External InlineFormat plugins must include a wildcard arm that returns an
        // appropriate InlineError for unrecognised variants.
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

    /// Render `node` to a string in `format`, with optional exporter-owned context.
    ///
    /// The default implementation ignores `context` and delegates to [`Self::render`].
    /// Implementations that need `context` (e.g., to embed a hyperlink
    /// relationship ID for OOXML) SHOULD override this method.
    ///
    /// ## Overriding
    ///
    /// Override this method to handle the `Link + Ooxml` case when a relationship
    /// ID is available in `context.hyperlink_rid`. All other cases must still
    /// delegate to [`Self::render`].
    ///
    /// See [`InlineRenderContext`] for the full contract.
    ///
    /// # Errors
    ///
    /// Same error conditions as [`Self::render`].
    fn render_with_context(
        &self,
        node: &InlineNode,
        format: InlineOutputFormat,
        context: &InlineRenderContext<'_>,
    ) -> Result<String, InlineError> {
        let _ = context; // default: ignore context, delegate to render
        self.render(node, format)
    }
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

    // ── F-001: InlineRenderContext + render_with_context ──────────────────────

    /// F-001 (CRIT): `InlineRenderContext` must exist and expose `hyperlink_rid: Option<&'a str>`.
    /// This test fails before F-001 because `InlineRenderContext` does not exist.
    #[test]
    fn test_f001_inline_render_context_struct_exists() {
        let ctx: InlineRenderContext<'_> = InlineRenderContext::default();
        assert!(
            ctx.hyperlink_rid.is_none(),
            "Default context must have no hyperlink_rid"
        );
    }

    /// F-001 (CRIT): `InlineRenderContext` with a `hyperlink_rid` can be constructed.
    #[test]
    fn test_f001_inline_render_context_with_rid() {
        let rid = "rId3";
        let ctx = InlineRenderContext {
            hyperlink_rid: Some(rid),
        };
        assert_eq!(ctx.hyperlink_rid, Some("rId3"));
    }

    /// F-001 (CRIT): `InlineRenderContext` is `Copy` (carries only `Option<&'a str>`).
    #[test]
    fn test_f001_inline_render_context_is_copy() {
        let ctx = InlineRenderContext {
            hyperlink_rid: Some("rId5"),
        };
        let ctx2 = ctx; // Copy
        let ctx3 = ctx; // Still usable
        assert_eq!(ctx2.hyperlink_rid, ctx3.hyperlink_rid);
    }

    /// F-001 (CRIT): The `InlineFormat` trait has a `render_with_context` method
    /// with the correct signature. We verify via a concrete implementation.
    /// This test fails before F-001 because the method does not exist on the trait.
    #[test]
    fn test_f001_render_with_context_exists_on_trait() {
        use slideforge_types::InlineNode;
        use std::sync::Arc;

        // A minimal stub implementation — only needs to compile and return Ok.
        struct StubFormatter;
        impl InlineFormat for StubFormatter {
            fn id(&self) -> &'static str {
                "stub"
            }
            fn render(
                &self,
                _node: &InlineNode,
                _format: InlineOutputFormat,
            ) -> Result<String, InlineError> {
                Ok(String::new())
            }
            // Inherits the default `render_with_context` — must compile.
        }

        let node = InlineNode::Plain(Arc::from("hello"));
        let ctx = InlineRenderContext::default();
        let result = StubFormatter.render_with_context(&node, InlineOutputFormat::Html, &ctx);
        // The default implementation delegates to render(), which returns Ok("").
        assert!(
            result.is_ok(),
            "render_with_context default must succeed: {result:?}"
        );
    }
}
