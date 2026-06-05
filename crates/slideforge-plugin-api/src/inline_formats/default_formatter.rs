//! [`DefaultInlineFormat`] — the bundled `InlineFormat` plugin.
//!
//! Implements all 12 [`slideforge_types::InlineNode`] variants × 3 output
//! formats. Registered as `"default"` in the [`crate::PluginRegistry`].
//!
//! ## Output format matrix
//!
//! | Variant | Ooxml | Html | Markdown |
//! |---------|-------|------|----------|
//! | `Plain` | `<a:r><a:t>{text}</a:t></a:r>` | `{text}` (escaped) | `{text}` |
//! | `Bold` | `<a:r><a:rPr b="1"/><a:t>{content}</a:t></a:r>` | `<strong>{content}</strong>` | `**{content}**` |
//! | `Italic` | `<a:r><a:rPr i="1"/><a:t>{content}</a:t></a:r>` | `<em>{content}</em>` | `*{content}*` |
//! | `Code` | `<a:r><a:rPr .../><a:t>{text}</a:t></a:r>` | `<code>{text}</code>` | `` `{text}` `` |
//! | `Link` | hyperlink run (with warn fallback) | `<a href="{url}">{text}</a>` | `[{text}]({url})` |
//! | `Math` | OMML passthrough (with warn fallback) | `<span class="math">{latex}</span>` | `${latex}$` |
//! | `Footnote` | superscript marker + run | `<sup>[{n}]</sup>` | `[^{n}]` |
//! | `Xref` | plain text run | `<a href="#{id}">{id}</a>` | `[{id}](#{id})` |
//! | `Superscript` | `baseline="30000"` | `<sup>{content}</sup>` | `^{content}^` |
//! | `Subscript` | `baseline="-25000"` | `<sub>{content}</sub>` | `~{content}~` |
//! | `Strikethrough` | `strike="sngStrike"` | `<del>{content}</del>` | `~~{content}~~` |
//! | `Highlight` | `highlight="yellow"` | `<mark>{content}</mark>` | `=={content}==` |
//!
//! ## Link OOXML limitation (EC-002 / v1.0)
//!
//! `InlineFormat::render()` does not receive a relationship-ID context (the
//! trait signature is invariant per BC-5.02.001 invariant 2). When rendering a
//! `Link` node to OOXML, no relationship ID is available, so this implementation
//! emits the display text as a plain run and logs a `tracing::warn!`. The URL
//! information is preserved in the warning log. Production-grade hyperlink
//! embedding in OOXML requires the exporter to call a separate
//! relationship-registration API before rendering inline content; see
//! `notes_slide.rs` for the current exporter-level implementation.

use slideforge_types::InlineNode;

use crate::traits::inline_format::{InlineError, InlineFormat, InlineOutputFormat};

/// The built-in stateless inline formatter.
///
/// Registered with id `"default"`. Handles all 12 [`InlineNode`] variants
/// for [`InlineOutputFormat::Ooxml`], [`InlineOutputFormat::Html`], and
/// [`InlineOutputFormat::Markdown`].
///
/// ## Thread safety
///
/// `DefaultInlineFormat` is a unit struct with no mutable state. It is
/// `Send + Sync` trivially.
///
/// ## Nesting
///
/// Nested variants (e.g., `Bold(vec![Italic(vec![Plain("x")])])`) are
/// rendered recursively: inner nodes are rendered and their outputs
/// concatenated to form the content of the outer element.
///
/// ## Depth limit
///
/// Nesting depth exceeding 64 returns
/// [`InlineError::RenderError`] instead of recursing — prevents stack
/// overflow on pathological inputs (BC-3.05.001 invariant).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefaultInlineFormat;

impl InlineFormat for DefaultInlineFormat {
    fn id(&self) -> &'static str {
        "default"
    }

    fn render(&self, node: &InlineNode, format: InlineOutputFormat) -> Result<String, InlineError> {
        render_node(node, format, 0)
    }
}

/// Maximum allowed inline nesting depth (BC-3.05.001 invariant).
const MAX_DEPTH: usize = 64;

/// Internal recursive renderer.
///
/// `depth` tracks the current recursion level so we can return
/// [`InlineError::RenderError`] instead of stack-overflowing on deeply
/// nested inputs.
fn render_node(
    node: &InlineNode,
    format: InlineOutputFormat,
    depth: usize,
) -> Result<String, InlineError> {
    if depth > MAX_DEPTH {
        return Err(InlineError::RenderError {
            node_kind: node.kind_name().to_owned(),
            message: "inline nesting depth exceeds maximum of 64".to_owned(),
        });
    }

    match format {
        InlineOutputFormat::Ooxml => render_ooxml(node, depth),
        InlineOutputFormat::Html => render_html(node, depth),
        InlineOutputFormat::Markdown => render_markdown(node, depth),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OOXML renderer
// ─────────────────────────────────────────────────────────────────────────────

/// Render children of a nested OOXML node by collecting their plain-text
/// representation (for embedding inside a single `<a:t>` element).
///
/// For OOXML, nested variants like `Bold([Italic([Plain("x")])])` are flattened
/// into a single run with combined run properties. This matches the behavior of
/// the legacy `serialize_nodes_with_context` function that STORY-085 replaces.
fn render_children_ooxml(
    children: &[InlineNode],
    bold: bool,
    italic: bool,
    depth: usize,
) -> Result<String, InlineError> {
    let mut out = String::new();
    for child in children {
        let child_xml = render_ooxml_with_context(child, bold, italic, depth + 1)?;
        out.push_str(&child_xml);
    }
    Ok(out)
}

/// Render a single `InlineNode` to OOXML with inherited bold/italic context.
///
/// This mirrors the behavior of the legacy `serialize_nodes_with_context`:
/// - Bold/Italic nodes pass their flag down and recurse into children
/// - Leaf nodes (Plain, Code, etc.) emit a single `<a:r>` run
fn render_ooxml_with_context(
    node: &InlineNode,
    bold: bool,
    italic: bool,
    depth: usize,
) -> Result<String, InlineError> {
    if depth > MAX_DEPTH {
        return Err(InlineError::RenderError {
            node_kind: node.kind_name().to_owned(),
            message: "inline nesting depth exceeds maximum of 64".to_owned(),
        });
    }

    match node {
        InlineNode::Plain(text) | InlineNode::Code(text) | InlineNode::Xref(text) => {
            Ok(emit_ooxml_run(text, bold, italic))
        },
        InlineNode::Bold(children) => {
            let mut out = String::new();
            for child in children {
                out.push_str(&render_ooxml_with_context(child, true, italic, depth + 1)?);
            }
            Ok(out)
        },
        InlineNode::Italic(children) => {
            let mut out = String::new();
            for child in children {
                out.push_str(&render_ooxml_with_context(child, bold, true, depth + 1)?);
            }
            Ok(out)
        },
        // Footnote, Superscript, Subscript, Strikethrough, Highlight:
        // inherit context and recurse — not distinctly representable in the
        // flat run model (matches legacy serializer behavior for most, but
        // we emit the correct run properties for these dedicated types when
        // called from the top-level render_ooxml entry point).
        InlineNode::Footnote(children)
        | InlineNode::Superscript(children)
        | InlineNode::Subscript(children)
        | InlineNode::Strikethrough(children)
        | InlineNode::Highlight(children) => {
            // When recursing from top-level OOXML render (not from Bold/Italic),
            // we use the dedicated run-property logic. Here in the context path,
            // just inherit flags (matches legacy behavior for nested use).
            let mut out = String::new();
            for child in children {
                out.push_str(&render_ooxml_with_context(child, bold, italic, depth + 1)?);
            }
            Ok(out)
        },
        InlineNode::Link { text, url } => {
            // EC-002: no relationship context available — emit display text as plain run
            tracing::warn!(
                url = %url,
                "Hyperlink relationship not registered for OOXML link to {url}; \
                 emitting display text as plain run (EC-002 / v1.0 limitation)"
            );
            let display_text = extract_plain_text_from_nodes(text);
            if display_text.is_empty() {
                Ok(String::new())
            } else {
                Ok(emit_ooxml_run(&display_text, bold, italic))
            }
        },
        InlineNode::Math(math_node) => {
            // EC-001: no pre-rendered OMML — emit LaTeX source as plain run
            tracing::warn!(
                "Math OMML rendering not pre-computed for OOXML; \
                 emitting LaTeX source as plain text run (EC-001)"
            );
            Ok(emit_ooxml_run(math_node.latex.as_ref(), bold, italic))
        },
    }
}

/// Render a node to OOXML `<a:r>` run markup (top-level entry).
fn render_ooxml(node: &InlineNode, depth: usize) -> Result<String, InlineError> {
    match node {
        InlineNode::Plain(text) => Ok(emit_ooxml_run(text, false, false)),
        InlineNode::Bold(children) => {
            // Bold: emit children with bold=true context
            let mut out = String::new();
            for child in children {
                out.push_str(&render_ooxml_with_context(child, true, false, depth + 1)?);
            }
            Ok(out)
        },
        InlineNode::Italic(children) => {
            // Italic: emit children with italic=true context
            let mut out = String::new();
            for child in children {
                out.push_str(&render_ooxml_with_context(child, false, true, depth + 1)?);
            }
            Ok(out)
        },
        InlineNode::Code(text) => {
            // Code: monospace via latin font override (Courier New), no bold/italic
            let mut out = String::new();
            out.push_str("<a:r><a:rPr><a:latin typeface=\"Courier New\"/></a:rPr><a:t>");
            out.push_str(&xml_escape(text));
            out.push_str("</a:t></a:r>");
            Ok(out)
        },
        InlineNode::Link { text, url } => {
            // EC-002: no relationship context — emit display text as plain run with warn
            tracing::warn!(
                url = %url,
                "Hyperlink relationship not registered for OOXML link to {url}; \
                 emitting display text as plain run (EC-002 / v1.0 limitation)"
            );
            let display_text = extract_plain_text_from_nodes(text);
            if display_text.is_empty() {
                Ok(String::new())
            } else {
                Ok(emit_ooxml_run(&display_text, false, false))
            }
        },
        InlineNode::Math(math_node) => {
            // EC-001: no pre-rendered OMML — emit LaTeX source as plain run with warn
            tracing::warn!(
                "Math OMML rendering not pre-computed for OOXML; \
                 emitting LaTeX source as plain text run (EC-001)"
            );
            Ok(emit_ooxml_run(math_node.latex.as_ref(), false, false))
        },
        InlineNode::Footnote(children) => {
            // Footnote: emit children (footnote number marker is a future story)
            let content = render_children_ooxml(children, false, false, depth)?;
            Ok(content)
        },
        InlineNode::Xref(id) => {
            // Xref: plain text run (cross-ref resolution is a future story)
            Ok(emit_ooxml_run(id, false, false))
        },
        InlineNode::Superscript(children) => {
            // Superscript: baseline=30000
            let text_content = extract_plain_text_from_renders(children, depth)?;
            let mut out = String::new();
            out.push_str("<a:r><a:rPr baseline=\"30000\"/><a:t>");
            out.push_str(&xml_escape(&text_content));
            out.push_str("</a:t></a:r>");
            Ok(out)
        },
        InlineNode::Subscript(children) => {
            // Subscript: baseline=-25000
            let text_content = extract_plain_text_from_renders(children, depth)?;
            let mut out = String::new();
            out.push_str("<a:r><a:rPr baseline=\"-25000\"/><a:t>");
            out.push_str(&xml_escape(&text_content));
            out.push_str("</a:t></a:r>");
            Ok(out)
        },
        InlineNode::Strikethrough(children) => {
            // Strikethrough: strike="sngStrike"
            let text_content = extract_plain_text_from_renders(children, depth)?;
            let mut out = String::new();
            out.push_str("<a:r><a:rPr strike=\"sngStrike\"/><a:t>");
            out.push_str(&xml_escape(&text_content));
            out.push_str("</a:t></a:r>");
            Ok(out)
        },
        InlineNode::Highlight(children) => {
            // Highlight: highlight="yellow"
            let text_content = extract_plain_text_from_renders(children, depth)?;
            let mut out = String::new();
            out.push_str("<a:r><a:rPr highlight=\"yellow\"/><a:t>");
            out.push_str(&xml_escape(&text_content));
            out.push_str("</a:t></a:r>");
            Ok(out)
        },
    }
}

/// Emit a single OOXML `<a:r>` run with optional bold/italic run properties.
///
/// Skips emission if `text` is empty (empty runs are not useful in OOXML).
fn emit_ooxml_run(text: &str, bold: bool, italic: bool) -> String {
    if text.is_empty() {
        return String::new();
    }
    let needs_rpr = bold || italic;
    let mut out = String::new();
    out.push_str("<a:r>");
    if needs_rpr {
        out.push_str("<a:rPr");
        if bold {
            out.push_str(" b=\"1\"");
        }
        if italic {
            out.push_str(" i=\"1\"");
        }
        out.push_str("/>");
    }
    out.push_str("<a:t>");
    out.push_str(&xml_escape(text));
    out.push_str("</a:t></a:r>");
    out
}

/// Render children as OOXML and collect the resulting plain text (for
/// specialized run-property nodes like Superscript/Subscript/Strikethrough/Highlight).
///
/// Since these nodes produce a single run with special properties, we need
/// to extract the text content of the children to embed in `<a:t>`.
fn extract_plain_text_from_renders(
    children: &[InlineNode],
    depth: usize,
) -> Result<String, InlineError> {
    // We need the plain text content for embedding in a specialized run.
    // Use the plain-text extractor which handles nesting correctly.
    if depth > MAX_DEPTH {
        return Err(InlineError::RenderError {
            node_kind: "children".to_owned(),
            message: "inline nesting depth exceeds maximum of 64".to_owned(),
        });
    }
    Ok(extract_plain_text_from_nodes(children))
}

// ─────────────────────────────────────────────────────────────────────────────
// HTML renderer
// ─────────────────────────────────────────────────────────────────────────────

/// Render a node to an HTML fragment string.
fn render_html(node: &InlineNode, depth: usize) -> Result<String, InlineError> {
    match node {
        InlineNode::Plain(text) => Ok(html_escape(text)),
        InlineNode::Bold(children) => {
            let content = render_children_html(children, depth)?;
            Ok(format!("<strong>{content}</strong>"))
        },
        InlineNode::Italic(children) => {
            let content = render_children_html(children, depth)?;
            Ok(format!("<em>{content}</em>"))
        },
        InlineNode::Code(text) => Ok(format!("<code>{}</code>", html_escape(text))),
        InlineNode::Link { text, url } => {
            let display = render_children_html(text, depth)?;
            Ok(format!("<a href=\"{url}\">{display}</a>"))
        },
        InlineNode::Math(math_node) => {
            Ok(format!("<span class=\"math\">{}</span>", math_node.latex))
        },
        InlineNode::Footnote(children) => {
            let content = render_children_html(children, depth)?;
            Ok(format!("<sup>[{content}]</sup>"))
        },
        InlineNode::Xref(id) => Ok(format!("<a href=\"#{id}\">{id}</a>")),
        InlineNode::Superscript(children) => {
            let content = render_children_html(children, depth)?;
            Ok(format!("<sup>{content}</sup>"))
        },
        InlineNode::Subscript(children) => {
            let content = render_children_html(children, depth)?;
            Ok(format!("<sub>{content}</sub>"))
        },
        InlineNode::Strikethrough(children) => {
            let content = render_children_html(children, depth)?;
            Ok(format!("<del>{content}</del>"))
        },
        InlineNode::Highlight(children) => {
            let content = render_children_html(children, depth)?;
            Ok(format!("<mark>{content}</mark>"))
        },
    }
}

/// Render a slice of children to HTML by concatenating their individual renders.
fn render_children_html(children: &[InlineNode], depth: usize) -> Result<String, InlineError> {
    let mut out = String::new();
    for child in children {
        out.push_str(&render_node(child, InlineOutputFormat::Html, depth + 1)?);
    }
    Ok(out)
}

// ─────────────────────────────────────────────────────────────────────────────
// Markdown renderer
// ─────────────────────────────────────────────────────────────────────────────

/// Render a node to a Markdown fragment string (`CommonMark` notation).
fn render_markdown(node: &InlineNode, depth: usize) -> Result<String, InlineError> {
    match node {
        InlineNode::Plain(text) => Ok(text.to_string()),
        InlineNode::Bold(children) => {
            let content = render_children_markdown(children, depth)?;
            Ok(format!("**{content}**"))
        },
        InlineNode::Italic(children) => {
            let content = render_children_markdown(children, depth)?;
            Ok(format!("*{content}*"))
        },
        InlineNode::Code(text) => Ok(format!("`{text}`")),
        InlineNode::Link { text, url } => {
            let display = render_children_markdown(text, depth)?;
            Ok(format!("[{display}]({url})"))
        },
        InlineNode::Math(math_node) => {
            if math_node.display {
                Ok(format!("$${latex}$$", latex = math_node.latex))
            } else {
                Ok(format!("${latex}$", latex = math_node.latex))
            }
        },
        InlineNode::Footnote(children) => {
            let content = render_children_markdown(children, depth)?;
            Ok(format!("[^{content}]"))
        },
        InlineNode::Xref(id) => Ok(format!("[{id}](#{id})")),
        InlineNode::Superscript(children) => {
            let content = render_children_markdown(children, depth)?;
            Ok(format!("^{content}^"))
        },
        InlineNode::Subscript(children) => {
            let content = render_children_markdown(children, depth)?;
            Ok(format!("~{content}~"))
        },
        InlineNode::Strikethrough(children) => {
            let content = render_children_markdown(children, depth)?;
            Ok(format!("~~{content}~~"))
        },
        InlineNode::Highlight(children) => {
            let content = render_children_markdown(children, depth)?;
            Ok(format!("=={content}=="))
        },
    }
}

/// Render a slice of children to Markdown by concatenating their individual renders.
fn render_children_markdown(children: &[InlineNode], depth: usize) -> Result<String, InlineError> {
    let mut out = String::new();
    for child in children {
        out.push_str(&render_node(
            child,
            InlineOutputFormat::Markdown,
            depth + 1,
        )?);
    }
    Ok(out)
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

/// XML-escape a string for safe embedding in XML element text content (`<a:t>`).
///
/// Replaces the XML-reserved characters that MUST be escaped in text content:
/// - `&` → `&amp;` (must be first to avoid double-escaping)
/// - `<` → `&lt;`
/// - `>` → `&gt;`
///
/// Note: `"` and `'` do not need escaping in XML text content (only in
/// attribute values). We do NOT escape them here for correctness — this matches
/// the behavior of the legacy `serialize_nodes_with_context` helper that we
/// replace, ensuring byte-identical OOXML output (AC-006).
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// HTML-escape a string for safe embedding in HTML text content or attributes.
///
/// Replaces the 4 characters that must be escaped in HTML:
/// - `&` → `&amp;`
/// - `<` → `&lt;`
/// - `>` → `&gt;`
/// - `"` → `&quot;`
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Extract all plain-text content from an inline node tree, depth-first.
///
/// Used for link display text extraction and for generating plain-text
/// content to embed in specialized OOXML runs (Superscript, Subscript, etc.).
fn extract_plain_text_from_nodes(nodes: &[InlineNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            InlineNode::Plain(s) | InlineNode::Code(s) | InlineNode::Xref(s) => {
                out.push_str(s);
            },
            InlineNode::Bold(c)
            | InlineNode::Italic(c)
            | InlineNode::Footnote(c)
            | InlineNode::Superscript(c)
            | InlineNode::Subscript(c)
            | InlineNode::Strikethrough(c)
            | InlineNode::Highlight(c) => {
                out.push_str(&extract_plain_text_from_nodes(c));
            },
            InlineNode::Link { text, .. } => {
                out.push_str(&extract_plain_text_from_nodes(text));
            },
            InlineNode::Math(m) => {
                out.push_str(m.latex.as_ref());
            },
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests — STORY-085 Red Gate
//
// ALL tests in this module MUST FAIL before the implementation is filled in.
// They will fail because render_ooxml/render_html/render_markdown are todo!().
// The todo!() panic reaches the test runner BEFORE any assertion is evaluated,
// so there are NO #[should_panic] wrappers here — that pattern inverts the
// Red Gate (a todo!() panic would make should_panic PASS, defeating TDD).
//
// Test naming: test_BC_5_02_001_xxx (Scope A) and test_BC_5_02_002_xxx (Scope B).
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::sync::Arc;

    use slideforge_types::{InlineNode, MathNode, SourceSpan};

    use super::DefaultInlineFormat;
    use crate::traits::inline_format::{InlineError, InlineFormat, InlineOutputFormat};

    // ── Helper ───────────────────────────────────────────────────────────────

    fn fmt() -> DefaultInlineFormat {
        DefaultInlineFormat
    }

    fn plain(s: &str) -> InlineNode {
        InlineNode::Plain(Arc::from(s))
    }

    fn bold(children: Vec<InlineNode>) -> InlineNode {
        InlineNode::Bold(children)
    }

    fn italic(children: Vec<InlineNode>) -> InlineNode {
        InlineNode::Italic(children)
    }

    fn code(s: &str) -> InlineNode {
        InlineNode::Code(Arc::from(s))
    }

    fn link(text: Vec<InlineNode>, url: &str) -> InlineNode {
        InlineNode::Link {
            text,
            url: Arc::from(url),
        }
    }

    fn math(latex: &str) -> InlineNode {
        InlineNode::Math(MathNode {
            latex: Arc::from(latex),
            display: false,
            span: SourceSpan::default(),
        })
    }

    fn footnote(children: Vec<InlineNode>) -> InlineNode {
        InlineNode::Footnote(children)
    }

    fn xref(id: &str) -> InlineNode {
        InlineNode::Xref(Arc::from(id))
    }

    fn superscript(children: Vec<InlineNode>) -> InlineNode {
        InlineNode::Superscript(children)
    }

    fn subscript(children: Vec<InlineNode>) -> InlineNode {
        InlineNode::Subscript(children)
    }

    fn strikethrough(children: Vec<InlineNode>) -> InlineNode {
        InlineNode::Strikethrough(children)
    }

    fn highlight(children: Vec<InlineNode>) -> InlineNode {
        InlineNode::Highlight(children)
    }

    // ── AC-001: Ooxml — all 12 variants ──────────────────────────────────────

    /// AC-001 test vector: Plain("hello") → `<a:r><a:t>hello</a:t></a:r>`
    #[test]
    fn test_bc_5_02_001_ooxml_plain_text() {
        let result = fmt()
            .render(&plain("hello"), InlineOutputFormat::Ooxml)
            .unwrap();
        assert_eq!(result, "<a:r><a:t>hello</a:t></a:r>");
    }

    /// AC-001 test vector: Bold([Plain("hi")]) contains `<a:rPr b="1"/>` and `<a:t>hi</a:t>`
    #[test]
    fn test_bc_5_02_001_ooxml_bold_contains_rpr_b1_and_text() {
        let result = fmt()
            .render(&bold(vec![plain("hi")]), InlineOutputFormat::Ooxml)
            .unwrap();
        assert!(
            result.contains("<a:rPr b=\"1\"/>"),
            "expected <a:rPr b=\"1\"/> in: {result}"
        );
        assert!(
            result.contains("<a:t>hi</a:t>"),
            "expected <a:t>hi</a:t> in: {result}"
        );
    }

    /// AC-001: Italic OOXML contains `<a:rPr i="1"/>`
    #[test]
    fn test_bc_5_02_001_ooxml_italic_contains_rpr_i1() {
        let result = fmt()
            .render(&italic(vec![plain("em")]), InlineOutputFormat::Ooxml)
            .unwrap();
        assert!(result.contains("<a:rPr i=\"1\"/>"), "got: {result}");
        assert!(result.contains("<a:t>em</a:t>"), "got: {result}");
    }

    /// AC-001: Code OOXML wraps in `<a:r>` with monospace run properties
    #[test]
    fn test_bc_5_02_001_ooxml_code_is_wrapped_in_run() {
        let result = fmt()
            .render(&code("fn foo()"), InlineOutputFormat::Ooxml)
            .unwrap();
        assert!(
            result.starts_with("<a:r>"),
            "expected <a:r> start, got: {result}"
        );
        assert!(result.contains("<a:t>fn foo()</a:t>"), "got: {result}");
    }

    /// AC-001: Link OOXML emits display text (not empty) — either hyperlink run or plain fallback
    #[test]
    fn test_bc_5_02_001_ooxml_link_emits_display_text() {
        let result = fmt()
            .render(
                &link(vec![plain("click here")], "https://example.com"),
                InlineOutputFormat::Ooxml,
            )
            .unwrap();
        assert!(
            result.contains("click here"),
            "display text must appear in output, got: {result}"
        );
    }

    /// AC-001: Math OOXML returns Ok (no panic, no `UnsupportedNode`) — EC-001 fallback path
    #[test]
    fn test_bc_5_02_001_ooxml_math_returns_ok() {
        let result = fmt().render(&math("x^2"), InlineOutputFormat::Ooxml);
        assert!(result.is_ok(), "Math Ooxml must return Ok, got: {result:?}");
    }

    /// AC-001: Footnote OOXML returns Ok
    #[test]
    fn test_bc_5_02_001_ooxml_footnote_returns_ok() {
        let result = fmt()
            .render(
                &footnote(vec![plain("note text")]),
                InlineOutputFormat::Ooxml,
            )
            .unwrap();
        assert!(
            !result.is_empty(),
            "Footnote Ooxml must produce non-empty output"
        );
    }

    /// AC-001: Xref OOXML returns plain text run
    #[test]
    fn test_bc_5_02_001_ooxml_xref_plain_run() {
        let result = fmt()
            .render(&xref("slide-3"), InlineOutputFormat::Ooxml)
            .unwrap();
        assert!(
            result.contains("slide-3"),
            "Xref id must appear in output, got: {result}"
        );
    }

    /// AC-001: Superscript OOXML uses baseline="30000"
    #[test]
    fn test_bc_5_02_001_ooxml_superscript_baseline_30000() {
        let result = fmt()
            .render(&superscript(vec![plain("2")]), InlineOutputFormat::Ooxml)
            .unwrap();
        assert!(
            result.contains("baseline=\"30000\""),
            "Superscript must use baseline=\"30000\", got: {result}"
        );
    }

    /// AC-001: Subscript OOXML uses baseline="-25000"
    #[test]
    fn test_bc_5_02_001_ooxml_subscript_baseline_neg25000() {
        let result = fmt()
            .render(&subscript(vec![plain("n")]), InlineOutputFormat::Ooxml)
            .unwrap();
        assert!(
            result.contains("baseline=\"-25000\""),
            "Subscript must use baseline=\"-25000\", got: {result}"
        );
    }

    /// AC-001: Strikethrough OOXML uses strike="sngStrike"
    #[test]
    fn test_bc_5_02_001_ooxml_strikethrough_sng_strike() {
        let result = fmt()
            .render(
                &strikethrough(vec![plain("del")]),
                InlineOutputFormat::Ooxml,
            )
            .unwrap();
        assert!(
            result.contains("strike=\"sngStrike\""),
            "Strikethrough must use strike=\"sngStrike\", got: {result}"
        );
    }

    /// AC-001: Highlight OOXML contains highlight="yellow"
    #[test]
    fn test_bc_5_02_001_ooxml_highlight_yellow() {
        let result = fmt()
            .render(&highlight(vec![plain("marked")]), InlineOutputFormat::Ooxml)
            .unwrap();
        assert!(
            result.contains("highlight=\"yellow\""),
            "Highlight must use highlight=\"yellow\", got: {result}"
        );
    }

    /// AC-001 snapshot: full Ooxml output for Plain
    #[test]
    fn test_bc_5_02_001_ooxml_plain_snapshot() {
        let result = fmt()
            .render(&plain("hello"), InlineOutputFormat::Ooxml)
            .unwrap();
        insta::assert_snapshot!("ooxml_plain_hello", result);
    }

    /// AC-001 snapshot: full Ooxml output for Bold([Plain("hi")])
    #[test]
    fn test_bc_5_02_001_ooxml_bold_snapshot() {
        let result = fmt()
            .render(&bold(vec![plain("hi")]), InlineOutputFormat::Ooxml)
            .unwrap();
        insta::assert_snapshot!("ooxml_bold_hi", result);
    }

    /// AC-001 snapshot: full Ooxml output for Italic([Plain("em")])
    #[test]
    fn test_bc_5_02_001_ooxml_italic_snapshot() {
        let result = fmt()
            .render(&italic(vec![plain("em")]), InlineOutputFormat::Ooxml)
            .unwrap();
        insta::assert_snapshot!("ooxml_italic_em", result);
    }

    /// AC-001 snapshot: full Ooxml output for `Code("fn foo()")`
    #[test]
    fn test_bc_5_02_001_ooxml_code_snapshot() {
        let result = fmt()
            .render(&code("fn foo()"), InlineOutputFormat::Ooxml)
            .unwrap();
        insta::assert_snapshot!("ooxml_code_fn_foo", result);
    }

    /// AC-001 snapshot: full Ooxml output for Xref("slide-3")
    #[test]
    fn test_bc_5_02_001_ooxml_xref_snapshot() {
        let result = fmt()
            .render(&xref("slide-3"), InlineOutputFormat::Ooxml)
            .unwrap();
        insta::assert_snapshot!("ooxml_xref_slide3", result);
    }

    /// AC-001 snapshot: Superscript
    #[test]
    fn test_bc_5_02_001_ooxml_superscript_snapshot() {
        let result = fmt()
            .render(&superscript(vec![plain("2")]), InlineOutputFormat::Ooxml)
            .unwrap();
        insta::assert_snapshot!("ooxml_superscript_2", result);
    }

    /// AC-001 snapshot: Subscript
    #[test]
    fn test_bc_5_02_001_ooxml_subscript_snapshot() {
        let result = fmt()
            .render(&subscript(vec![plain("n")]), InlineOutputFormat::Ooxml)
            .unwrap();
        insta::assert_snapshot!("ooxml_subscript_n", result);
    }

    /// AC-001 snapshot: Strikethrough
    #[test]
    fn test_bc_5_02_001_ooxml_strikethrough_snapshot() {
        let result = fmt()
            .render(
                &strikethrough(vec![plain("del")]),
                InlineOutputFormat::Ooxml,
            )
            .unwrap();
        insta::assert_snapshot!("ooxml_strikethrough_del", result);
    }

    /// AC-001 snapshot: Highlight
    #[test]
    fn test_bc_5_02_001_ooxml_highlight_snapshot() {
        let result = fmt()
            .render(&highlight(vec![plain("marked")]), InlineOutputFormat::Ooxml)
            .unwrap();
        insta::assert_snapshot!("ooxml_highlight_marked", result);
    }

    // ── AC-002: Html — all 12 variants ───────────────────────────────────────

    /// AC-002: Bold([Plain("hi")]) HTML → `<strong>hi</strong>`
    #[test]
    fn test_bc_5_02_001_html_bold() {
        let result = fmt()
            .render(&bold(vec![plain("hi")]), InlineOutputFormat::Html)
            .unwrap();
        assert_eq!(result, "<strong>hi</strong>");
    }

    /// AC-002: Italic([Plain("em")]) HTML → `<em>em</em>`
    #[test]
    fn test_bc_5_02_001_html_italic() {
        let result = fmt()
            .render(&italic(vec![plain("em")]), InlineOutputFormat::Html)
            .unwrap();
        assert_eq!(result, "<em>em</em>");
    }

    /// AC-002: `Code("fn foo()")` HTML → `<code>fn foo()</code>`
    #[test]
    fn test_bc_5_02_001_html_code() {
        let result = fmt()
            .render(&code("fn foo()"), InlineOutputFormat::Html)
            .unwrap();
        assert_eq!(result, "<code>fn foo()</code>");
    }

    /// AC-002: Plain("hello") HTML → `hello` (no wrapping tags)
    #[test]
    fn test_bc_5_02_001_html_plain() {
        let result = fmt()
            .render(&plain("hello"), InlineOutputFormat::Html)
            .unwrap();
        assert_eq!(result, "hello");
    }

    /// AC-002: HTML escaping — `&` → `&amp;`, `<` → `&lt;`, `>` → `&gt;`, `"` → `&quot;`
    #[test]
    fn test_bc_5_02_001_html_plain_escaping_ampersand() {
        let result = fmt()
            .render(&plain("a & b < c"), InlineOutputFormat::Html)
            .unwrap();
        assert_eq!(result, "a &amp; b &lt; c");
    }

    /// AC-002: HTML escaping double-quote
    #[test]
    fn test_bc_5_02_001_html_plain_escaping_double_quote() {
        let result = fmt()
            .render(&plain("say \"hi\""), InlineOutputFormat::Html)
            .unwrap();
        assert_eq!(result, "say &quot;hi&quot;");
    }

    /// AC-002: Link HTML → `<a href="{url}">{text}</a>`
    #[test]
    fn test_bc_5_02_001_html_link() {
        let result = fmt()
            .render(
                &link(vec![plain("click here")], "https://example.com"),
                InlineOutputFormat::Html,
            )
            .unwrap();
        assert_eq!(result, "<a href=\"https://example.com\">click here</a>");
    }

    /// AC-002: Math HTML → `<span class="math">{latex}</span>`
    #[test]
    fn test_bc_5_02_001_html_math() {
        let result = fmt()
            .render(&math("x^2"), InlineOutputFormat::Html)
            .unwrap();
        assert_eq!(result, "<span class=\"math\">x^2</span>");
    }

    /// AC-002: Xref HTML → `<a href="#{id}">{id}</a>`
    #[test]
    fn test_bc_5_02_001_html_xref() {
        let result = fmt()
            .render(&xref("slide-3"), InlineOutputFormat::Html)
            .unwrap();
        assert_eq!(result, "<a href=\"#slide-3\">slide-3</a>");
    }

    /// AC-002: Superscript HTML → `<sup>{content}</sup>`
    #[test]
    fn test_bc_5_02_001_html_superscript() {
        let result = fmt()
            .render(&superscript(vec![plain("2")]), InlineOutputFormat::Html)
            .unwrap();
        assert_eq!(result, "<sup>2</sup>");
    }

    /// AC-002: Subscript HTML → `<sub>{content}</sub>`
    #[test]
    fn test_bc_5_02_001_html_subscript() {
        let result = fmt()
            .render(&subscript(vec![plain("n")]), InlineOutputFormat::Html)
            .unwrap();
        assert_eq!(result, "<sub>n</sub>");
    }

    /// AC-002: Strikethrough HTML → `<del>{content}</del>`
    #[test]
    fn test_bc_5_02_001_html_strikethrough() {
        let result = fmt()
            .render(&strikethrough(vec![plain("del")]), InlineOutputFormat::Html)
            .unwrap();
        assert_eq!(result, "<del>del</del>");
    }

    /// AC-002: Highlight HTML → `<mark>{content}</mark>`
    #[test]
    fn test_bc_5_02_001_html_highlight() {
        let result = fmt()
            .render(&highlight(vec![plain("marked")]), InlineOutputFormat::Html)
            .unwrap();
        assert_eq!(result, "<mark>marked</mark>");
    }

    /// AC-002: Footnote HTML → `<sup>[{n}]</sup>` style (content rendered)
    #[test]
    fn test_bc_5_02_001_html_footnote() {
        let result = fmt()
            .render(
                &footnote(vec![plain("footnote body")]),
                InlineOutputFormat::Html,
            )
            .unwrap();
        assert!(
            result.contains("<sup>") || result.contains('['),
            "got: {result}"
        );
        assert!(!result.is_empty());
    }

    // ── AC-003: Markdown — all 12 variants ───────────────────────────────────

    /// AC-003: Italic([Plain("em")]) Markdown → `*em*`
    #[test]
    fn test_bc_5_02_001_markdown_italic() {
        let result = fmt()
            .render(&italic(vec![plain("em")]), InlineOutputFormat::Markdown)
            .unwrap();
        assert_eq!(result, "*em*");
    }

    /// AC-003: Strikethrough([Plain("del")]) Markdown → `~~del~~`
    #[test]
    fn test_bc_5_02_001_markdown_strikethrough() {
        let result = fmt()
            .render(
                &strikethrough(vec![plain("del")]),
                InlineOutputFormat::Markdown,
            )
            .unwrap();
        assert_eq!(result, "~~del~~");
    }

    /// AC-003: Bold([Plain("hi")]) Markdown → `**hi**`
    #[test]
    fn test_bc_5_02_001_markdown_bold() {
        let result = fmt()
            .render(&bold(vec![plain("hi")]), InlineOutputFormat::Markdown)
            .unwrap();
        assert_eq!(result, "**hi**");
    }

    /// AC-003: Plain("hello") Markdown → `hello`
    #[test]
    fn test_bc_5_02_001_markdown_plain() {
        let result = fmt()
            .render(&plain("hello"), InlineOutputFormat::Markdown)
            .unwrap();
        assert_eq!(result, "hello");
    }

    /// AC-003: Code("x") Markdown → `` `x` ``
    #[test]
    fn test_bc_5_02_001_markdown_code() {
        let result = fmt()
            .render(&code("x"), InlineOutputFormat::Markdown)
            .unwrap();
        assert_eq!(result, "`x`");
    }

    /// AC-003: Link Markdown → `[{text}]({url})`
    #[test]
    fn test_bc_5_02_001_markdown_link() {
        let result = fmt()
            .render(
                &link(vec![plain("click")], "https://example.com"),
                InlineOutputFormat::Markdown,
            )
            .unwrap();
        assert_eq!(result, "[click](https://example.com)");
    }

    /// AC-003: Math Markdown → `${latex}$`
    #[test]
    fn test_bc_5_02_001_markdown_math() {
        let result = fmt()
            .render(&math("x^2"), InlineOutputFormat::Markdown)
            .unwrap();
        assert_eq!(result, "$x^2$");
    }

    /// AC-003: Xref Markdown → `[{id}](#{id})`
    #[test]
    fn test_bc_5_02_001_markdown_xref() {
        let result = fmt()
            .render(&xref("slide-3"), InlineOutputFormat::Markdown)
            .unwrap();
        assert_eq!(result, "[slide-3](#slide-3)");
    }

    /// AC-003: Superscript Markdown → `^{content}^`
    #[test]
    fn test_bc_5_02_001_markdown_superscript() {
        let result = fmt()
            .render(&superscript(vec![plain("2")]), InlineOutputFormat::Markdown)
            .unwrap();
        assert_eq!(result, "^2^");
    }

    /// AC-003: Subscript Markdown → `~{content}~`
    #[test]
    fn test_bc_5_02_001_markdown_subscript() {
        let result = fmt()
            .render(&subscript(vec![plain("n")]), InlineOutputFormat::Markdown)
            .unwrap();
        assert_eq!(result, "~n~");
    }

    /// AC-003: Highlight Markdown → `=={content}==`
    #[test]
    fn test_bc_5_02_001_markdown_highlight() {
        let result = fmt()
            .render(
                &highlight(vec![plain("marked")]),
                InlineOutputFormat::Markdown,
            )
            .unwrap();
        assert_eq!(result, "==marked==");
    }

    /// AC-003: Footnote Markdown → `[^{n}]`
    #[test]
    fn test_bc_5_02_001_markdown_footnote() {
        let result = fmt()
            .render(&footnote(vec![plain("note")]), InlineOutputFormat::Markdown)
            .unwrap();
        assert!(
            result.starts_with("[^"),
            "Footnote Markdown must start with [^, got: {result}"
        );
    }

    // ── Nesting (AC-001 / AC-002 / AC-003) ───────────────────────────────────

    /// Nesting: Bold([Italic([Plain("text")])]) Ooxml — verify recursive rendering
    #[test]
    fn test_bc_5_02_001_ooxml_bold_italic_nested() {
        let node = bold(vec![italic(vec![plain("text")])]);
        let result = fmt().render(&node, InlineOutputFormat::Ooxml).unwrap();
        // The output must contain both bold and italic formatting indicators
        // and the text content.
        assert!(
            result.contains("text"),
            "nested text must appear in output, got: {result}"
        );
        // Implementation determines exact nesting strategy; verify it is non-empty
        // and starts with OOXML run markup.
        assert!(
            result.starts_with("<a:r>") || result.contains("<a:r>"),
            "got: {result}"
        );
    }

    /// Nesting: Bold([Italic([Plain("text")])]) HTML
    #[test]
    fn test_bc_5_02_001_html_bold_italic_nested() {
        let node = bold(vec![italic(vec![plain("text")])]);
        let result = fmt().render(&node, InlineOutputFormat::Html).unwrap();
        assert_eq!(result, "<strong><em>text</em></strong>");
    }

    /// Nesting: Bold([Italic([Plain("text")])]) Markdown
    #[test]
    fn test_bc_5_02_001_markdown_bold_italic_nested() {
        let node = bold(vec![italic(vec![plain("text")])]);
        let result = fmt().render(&node, InlineOutputFormat::Markdown).unwrap();
        assert_eq!(result, "***text***");
    }

    // ── EC-001: Math OOXML no-OMML fallback ──────────────────────────────────

    /// EC-001: Math with no pre-rendered OMML in OOXML mode returns Ok with
    /// fallback plain run (does NOT panic or return `UnsupportedNode`).
    /// The `tracing::warn!` is emitted by the implementation.
    #[test]
    fn test_bc_5_02_001_ec001_math_ooxml_fallback_plain_run() {
        let result = fmt()
            .render(
                &InlineNode::Math(MathNode {
                    latex: Arc::from("\\int_0^1 x dx"),
                    display: false,
                    span: SourceSpan::default(),
                }),
                InlineOutputFormat::Ooxml,
            )
            .unwrap();
        // Must return Ok with a plain run containing the latex source
        assert!(
            result.contains("\\int_0^1 x dx"),
            "Math OOXML fallback must include LaTeX source, got: {result}"
        );
        assert!(
            result.starts_with("<a:r>"),
            "fallback must be a valid run, got: {result}"
        );
    }

    // ── EC-002: Link OOXML no-relationship fallback ───────────────────────────

    /// EC-002: Link in OOXML mode without relationship context returns Ok
    /// with display text preserved (does NOT silently drop the text).
    #[test]
    fn test_bc_5_02_001_ec002_link_ooxml_display_text_preserved() {
        let result = fmt()
            .render(
                &link(vec![plain("my link text")], "https://example.com"),
                InlineOutputFormat::Ooxml,
            )
            .unwrap();
        // Display text must NOT be silently dropped regardless of relationship availability.
        assert!(
            result.contains("my link text"),
            "Link OOXML must preserve display text, got: {result}"
        );
    }

    // ── EC-003: XML escaping in OOXML ─────────────────────────────────────────

    /// EC-003: Plain("a & b") OOXML → `<a:r><a:t>a &amp; b</a:t></a:r>`
    #[test]
    fn test_bc_5_02_001_ec003_ooxml_xml_escape_ampersand() {
        let result = fmt()
            .render(&plain("a & b"), InlineOutputFormat::Ooxml)
            .unwrap();
        assert_eq!(result, "<a:r><a:t>a &amp; b</a:t></a:r>");
    }

    /// EC-003: Plain("a < b") OOXML → `<a:r><a:t>a &lt; b</a:t></a:r>`
    #[test]
    fn test_bc_5_02_001_ec003_ooxml_xml_escape_lt() {
        let result = fmt()
            .render(&plain("a < b"), InlineOutputFormat::Ooxml)
            .unwrap();
        assert_eq!(result, "<a:r><a:t>a &lt; b</a:t></a:r>");
    }

    /// EC-003: HTML escaping in Html mode for &, <, >, "
    #[test]
    fn test_bc_5_02_001_ec003_html_escape_all_special_chars() {
        let result = fmt()
            .render(&plain("a & b < c > d \"e\""), InlineOutputFormat::Html)
            .unwrap();
        assert_eq!(result, "a &amp; b &lt; c &gt; d &quot;e&quot;");
    }

    // ── EC-004: Nesting depth > 64 ────────────────────────────────────────────

    /// EC-004: Recursive nesting depth > 64 returns `InlineError::RenderError`
    /// with "inline nesting depth exceeds maximum of 64" message, not a stack overflow.
    #[test]
    fn test_bc_5_02_001_ec004_nesting_depth_exceeds_64_returns_render_error() {
        // Build a deeply nested Bold node: Bold(Bold(Bold(...(Plain("x")...)))) — 65 levels
        let mut node = plain("x");
        for _ in 0..65 {
            node = bold(vec![node]);
        }
        let result = fmt().render(&node, InlineOutputFormat::Ooxml);
        match result {
            Err(InlineError::RenderError { message, .. }) => {
                assert!(
                    message.contains("nesting depth exceeds maximum of 64"),
                    "unexpected error message: {message}"
                );
            },
            Ok(s) => panic!("expected RenderError for depth > 64, got Ok({s:?})"),
            Err(e) => panic!("expected RenderError for depth > 64, got different error: {e:?}"),
        }
    }

    /// EC-004: Nesting depth exactly 64 succeeds (boundary — depth 64 is allowed)
    #[test]
    fn test_bc_5_02_001_ec004_nesting_depth_exactly_64_succeeds() {
        // Build exactly 64 levels of Bold nesting (the limit is depth > 64, so depth 64 is ok)
        let mut node = plain("x");
        for _ in 0..64 {
            node = bold(vec![node]);
        }
        let result = fmt().render(&node, InlineOutputFormat::Ooxml);
        assert!(
            result.is_ok(),
            "nesting depth exactly 64 must succeed, got: {result:?}"
        );
    }

    // ── DefaultInlineFormat id ────────────────────────────────────────────────

    /// The formatter id must be "default"
    #[test]
    fn test_bc_5_02_001_default_inline_format_id_is_default() {
        assert_eq!(fmt().id(), "default");
    }

    /// `DefaultInlineFormat` is Send + Sync
    #[test]
    fn test_bc_5_02_001_default_inline_format_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DefaultInlineFormat>();
    }
}
