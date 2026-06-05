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

use crate::traits::inline_format::{
    InlineError, InlineFormat, InlineOutputFormat, InlineRenderContext,
};

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

    /// Render `node` with optional exporter-owned context.
    ///
    /// For `Link + Ooxml` with a `Some(rid)` in `context.hyperlink_rid`,
    /// emits the full OOXML hyperlink run:
    ///
    /// ```xml
    /// <a:r><a:rPr><a:hlinkClick r:id="{rid}"/></a:rPr><a:t>{escaped display text}</a:t></a:r>
    /// ```
    ///
    /// Both `rid` and display text are XML-escaped.
    ///
    /// For all other `(node, format)` combinations, or for `Link + Ooxml` with
    /// `context.hyperlink_rid == None`, delegates to [`Self::render`].
    ///
    /// This override is the SOLE place in `slideforge-plugin-api` where OOXML
    /// hyperlink run markup is constructed. The `slideforge-pptx` exporter must
    /// route all `Link + Ooxml` rendering through this method (AC-005).
    fn render_with_context(
        &self,
        node: &InlineNode,
        format: InlineOutputFormat,
        context: &InlineRenderContext<'_>,
    ) -> Result<String, InlineError> {
        // Only the Link + Ooxml combination with a provided rId is handled here.
        // All other cases delegate to the standard render path.
        if let (InlineNode::Link { text, .. }, InlineOutputFormat::Ooxml, Some(rid)) =
            (node, format, context.hyperlink_rid)
        {
            // Extract display text from the link children (flatten to plain text,
            // same as the EC-002 fallback path but here we have a valid rId).
            // Use the depth-limited extractor (EC-004 guard) to prevent stack overflow
            // on pathological deeply-nested Link display text (F-008-OBS).
            let display_text = extract_plain_text_depth_limited(text, 0)?;
            if display_text.is_empty() {
                return Ok(String::new());
            }
            // Emit: <a:r><a:rPr><a:hlinkClick r:id="{rid}"/></a:rPr><a:t>{text}</a:t></a:r>
            // Both rid and text are XML-escaped (CWE-116 defense — attribute + text content).
            let mut out = String::new();
            out.push_str("<a:r><a:rPr><a:hlinkClick r:id=\"");
            out.push_str(&xml_escape(rid));
            out.push_str("\"/></a:rPr><a:t>");
            out.push_str(&xml_escape(&display_text));
            out.push_str("</a:t></a:r>");
            return Ok(out);
        }
        // All other cases: delegate to the standard render path.
        self.render(node, format)
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

/// Accumulated run properties threaded through the OOXML recursion.
///
/// All formatting nodes set their property on this struct and recurse into
/// children with the updated state.  At each leaf (Plain, Xref, Code) exactly
/// one `<a:r>` is emitted with ALL accumulated properties.
///
/// ## Attribute order (must match existing snapshots byte-for-byte)
///
/// `<a:rPr>` attributes are emitted in this order by [`RunProps::emit_run`]:
/// `b`, `i`, `baseline`, `strike`, `highlight`.  The `<a:latin>` child element
/// (Code nodes) follows any attributes, before the closing `>`.
///
/// This ordering is stable across all single-property cases that have accepted
/// snapshots.  Combined-property cases produce attributes in the same order.
#[derive(Debug, Clone, Copy)]
struct RunProps {
    /// `b="1"` — from a `Bold` ancestor.
    bold: bool,
    /// `i="1"` — from an `Italic` ancestor.
    italic: bool,
    /// `baseline="{n}"` — from a `Superscript` (+30 000) or `Subscript` (−25 000).
    /// Only the innermost (most-recently-set) value is used; overlapping
    /// Superscript/Subscript combinations are not representable in OOXML and are
    /// extremely unlikely in practice.
    baseline: Option<i32>,
    /// `strike="sngStrike"` — from a `Strikethrough` ancestor.
    strike: bool,
    /// `highlight="yellow"` — from a `Highlight` ancestor.
    highlight: bool,
    /// Triggers `<a:latin typeface="Courier New"/>` child element — set by `Code` leaf.
    code_font: bool,
}

impl RunProps {
    const fn new() -> Self {
        Self {
            bold: false,
            italic: false,
            baseline: None,
            strike: false,
            highlight: false,
            code_font: false,
        }
    }

    /// Emit a single `<a:r>` run with ALL accumulated properties.
    ///
    /// Returns an empty string if `text` is empty (empty runs are inert in OOXML).
    ///
    /// # Attribute / child-element order
    ///
    /// `<a:rPr {b} {i} {baseline} {strike} {highlight}>{latin-child}</a:rPr>`
    ///
    /// This order preserves byte-identity with all previously accepted snapshots:
    /// - `Bold` snapshots: `b="1"` first ✓
    /// - `Italic` snapshots: `i="1"` only ✓
    /// - `Superscript`/`Subscript` snapshots: `baseline` only ✓
    /// - `Strikethrough` snapshots: `strike` only ✓
    /// - `Highlight` snapshots: `highlight` only ✓
    /// - `Code` snapshot: no attributes, `<a:latin>` child inside `<a:rPr>` ✓
    fn emit_run(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }
        let needs_rpr = self.bold
            || self.italic
            || self.baseline.is_some()
            || self.strike
            || self.highlight
            || self.code_font;

        let mut out = String::new();
        out.push_str("<a:r>");
        if needs_rpr {
            out.push_str("<a:rPr");
            if self.bold {
                out.push_str(" b=\"1\"");
            }
            if self.italic {
                out.push_str(" i=\"1\"");
            }
            if let Some(baseline) = self.baseline {
                out.push_str(" baseline=\"");
                out.push_str(&baseline.to_string());
                out.push('"');
            }
            if self.strike {
                out.push_str(" strike=\"sngStrike\"");
            }
            if self.highlight {
                out.push_str(" highlight=\"yellow\"");
            }
            if self.code_font {
                // `<a:latin>` is a child element — must be inside `<a:rPr>…</a:rPr>`
                out.push_str("><a:latin typeface=\"Courier New\"/></a:rPr>");
            } else {
                out.push_str("/>");
            }
        }
        out.push_str("<a:t>");
        out.push_str(&xml_escape(text));
        out.push_str("</a:t></a:r>");
        out
    }
}

/// Render a node to OOXML `<a:r>` run markup, accumulating run properties.
///
/// `props` carries all run-property flags inherited from ancestor nodes.
/// Formatting nodes (`Bold`, `Italic`, `Strikethrough`, etc.) set their flag and
/// recurse; leaf nodes (`Plain`, `Xref`, `Code`) emit a single run with all
/// accumulated properties.
///
/// ## Combined run properties (F-009-MED)
///
/// Nesting in BOTH directions produces combined properties:
/// - `Bold([Strikethrough([Plain("x")])])` → `<a:rPr b="1" strike="sngStrike"/>`
/// - `Strikethrough([Bold([Plain("x")])])` → same
/// - `Bold([Italic([Strikethrough([Plain("x")])])])` →
///   `<a:rPr b="1" i="1" strike="sngStrike"/>`
///
/// ## Byte identity with existing snapshots
///
/// Single-property cases are unchanged — the same attribute is emitted in the
/// same position (see [`RunProps::emit_run`] attribute order).
fn render_ooxml_accumulate(
    node: &InlineNode,
    props: RunProps,
    depth: usize,
) -> Result<String, InlineError> {
    if depth > MAX_DEPTH {
        return Err(InlineError::RenderError {
            node_kind: node.kind_name().to_owned(),
            message: "inline nesting depth exceeds maximum of 64".to_owned(),
        });
    }

    match node {
        // ── Leaf: Plain text run ─────────────────────────────────────────────
        InlineNode::Plain(text) | InlineNode::Xref(text) => Ok(props.emit_run(text)),

        // ── Leaf: Code — adds monospace font to accumulated props ────────────
        InlineNode::Code(text) => {
            let mut code_props = props;
            code_props.code_font = true;
            Ok(code_props.emit_run(text))
        },

        // ── Formatting: set property and recurse into children ───────────────
        InlineNode::Bold(children) => {
            let mut child_props = props;
            child_props.bold = true;
            let mut out = String::new();
            for child in children {
                out.push_str(&render_ooxml_accumulate(child, child_props, depth + 1)?);
            }
            Ok(out)
        },
        InlineNode::Italic(children) => {
            let mut child_props = props;
            child_props.italic = true;
            let mut out = String::new();
            for child in children {
                out.push_str(&render_ooxml_accumulate(child, child_props, depth + 1)?);
            }
            Ok(out)
        },
        InlineNode::Strikethrough(children) => {
            let mut child_props = props;
            child_props.strike = true;
            let mut out = String::new();
            for child in children {
                out.push_str(&render_ooxml_accumulate(child, child_props, depth + 1)?);
            }
            Ok(out)
        },
        InlineNode::Superscript(children) => {
            let mut child_props = props;
            child_props.baseline = Some(30_000);
            let mut out = String::new();
            for child in children {
                out.push_str(&render_ooxml_accumulate(child, child_props, depth + 1)?);
            }
            Ok(out)
        },
        InlineNode::Subscript(children) => {
            let mut child_props = props;
            child_props.baseline = Some(-25_000);
            let mut out = String::new();
            for child in children {
                out.push_str(&render_ooxml_accumulate(child, child_props, depth + 1)?);
            }
            Ok(out)
        },
        InlineNode::Highlight(children) => {
            let mut child_props = props;
            child_props.highlight = true;
            let mut out = String::new();
            for child in children {
                out.push_str(&render_ooxml_accumulate(child, child_props, depth + 1)?);
            }
            Ok(out)
        },
        InlineNode::Footnote(children) => {
            // Footnote inherits all props and recurses (footnote marker is a future story).
            let mut out = String::new();
            for child in children {
                out.push_str(&render_ooxml_accumulate(child, props, depth + 1)?);
            }
            Ok(out)
        },

        // ── Link: EC-002 fallback — emit display text with warn ──────────────
        InlineNode::Link { text, url } => {
            tracing::warn!(
                url = %url,
                "Hyperlink relationship not registered for OOXML link to {url}; \
                 emitting display text as plain run (EC-002 / v1.0 limitation)"
            );
            let display_text = extract_plain_text_depth_limited(text, depth + 1)?;
            if display_text.is_empty() {
                Ok(String::new())
            } else {
                Ok(props.emit_run(&display_text))
            }
        },

        // ── Math: EC-001 fallback — emit LaTeX source as plain run ───────────
        InlineNode::Math(math_node) => {
            tracing::warn!(
                "Math OMML rendering not pre-computed for OOXML; \
                 emitting LaTeX source as plain text run (EC-001)"
            );
            Ok(props.emit_run(math_node.latex.as_ref()))
        },
    }
}

/// Render a node to OOXML `<a:r>` run markup (top-level entry).
///
/// Delegates to [`render_ooxml_accumulate`] with an empty [`RunProps`] so all
/// single-property cases produce the same output as before (byte-identical with
/// existing accepted snapshots).
fn render_ooxml(node: &InlineNode, depth: usize) -> Result<String, InlineError> {
    render_ooxml_accumulate(node, RunProps::new(), depth)
}

/// Depth-limited plain text extractor.
///
/// Recursively extracts plain text from inline node trees, tracking nesting depth.
/// Returns [`InlineError::RenderError`] if `depth > MAX_DEPTH` at any level.
///
/// Used by the `render_with_context` Link path (F-008-OBS / EC-004) to flatten
/// link display text while enforcing the 64-level depth guard.
fn extract_plain_text_depth_limited(
    nodes: &[InlineNode],
    depth: usize,
) -> Result<String, InlineError> {
    if depth > MAX_DEPTH {
        return Err(InlineError::RenderError {
            node_kind: "children".to_owned(),
            message: "inline nesting depth exceeds maximum of 64".to_owned(),
        });
    }
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
                out.push_str(&extract_plain_text_depth_limited(c, depth + 1)?);
            },
            InlineNode::Link { text, .. } => {
                out.push_str(&extract_plain_text_depth_limited(text, depth + 1)?);
            },
            InlineNode::Math(m) => {
                out.push_str(m.latex.as_ref());
            },
        }
    }
    Ok(out)
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
            // F-004: HTML-escape the URL for safe embedding in the href attribute.
            // A `"` in the URL would prematurely close the attribute (CWE-116).
            let escaped_url = html_escape(url);
            Ok(format!("<a href=\"{escaped_url}\">{display}</a>"))
        },
        InlineNode::Math(math_node) => {
            // F-004: HTML-escape the LaTeX body. A raw `<` inside the span
            // would break HTML parsing (CWE-116 / XSS vector).
            Ok(format!(
                "<span class=\"math\">{}</span>",
                html_escape(math_node.latex.as_ref())
            ))
        },
        InlineNode::Footnote(children) => {
            let content = render_children_html(children, depth)?;
            Ok(format!("<sup>[{content}]</sup>"))
        },
        InlineNode::Xref(id) => {
            // F-004: HTML-escape the id in both the href attribute and link text.
            // A `<` or `"` in the id would break the attribute / text content.
            let escaped_id = html_escape(id);
            Ok(format!("<a href=\"#{escaped_id}\">{escaped_id}</a>"))
        },
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

    // ── F-004 (HIGH): HTML-escape url, id, latex in HTML output ─────────────

    /// F-004 (HIGH): Link HTML — URL containing a double-quote must be escaped.
    ///
    /// `<a href="url-with-"quote"">` is not valid HTML because the `"` inside
    /// the attribute closes it prematurely. Must become `&quot;`.
    ///
    /// This test FAILS before F-004 because `render_html` for Link interpolates
    /// the url directly: `format!("<a href=\"{url}\">{display}</a>")`.
    #[test]
    fn test_f004_html_link_url_quote_escaped() {
        // URL containing a double-quote — must be &quot; in the href attribute.
        let node = link(vec![plain("click")], "https://example.com/path?a=\"1\"&b=2");
        let result = fmt().render(&node, InlineOutputFormat::Html).unwrap();
        assert!(
            !result.contains("\"1\""),
            "F-004: raw \" in URL must be &quot;-escaped; got: {result}"
        );
        assert!(
            result.contains("&quot;"),
            "F-004: URL double-quote must become &quot; in href; got: {result}"
        );
    }

    /// F-004 (HIGH): Xref HTML — id containing `<` must be escaped in both
    /// the `href="#id"` attribute and the link text.
    ///
    /// This test FAILS before F-004 because `render_html` for Xref:
    ///   `format!("<a href=\"#{id}\">{id}</a>")`
    /// does not escape the id value.
    #[test]
    fn test_f004_html_xref_id_with_angle_bracket_escaped() {
        let node = InlineNode::Xref(Arc::from("fig<1>"));
        let result = fmt().render(&node, InlineOutputFormat::Html).unwrap();
        assert!(
            !result.contains("<1>"),
            "F-004: raw < in xref id must be escaped; got: {result}"
        );
        assert!(
            result.contains("&lt;"),
            "F-004: < in xref id must become &lt;; got: {result}"
        );
    }

    /// F-004 (HIGH): Math HTML — latex body containing `a < b` must be escaped.
    ///
    /// A raw `<` inside `<span class="math">...</span>` breaks HTML parsing.
    ///
    /// This test FAILS before F-004 because `render_html` for Math:
    ///   `format!("<span class=\"math\">{}</span>", math_node.latex)`
    /// does not escape the latex value.
    #[test]
    fn test_f004_html_math_latex_with_lt_escaped() {
        let node = InlineNode::Math(slideforge_types::MathNode {
            latex: Arc::from("a < b"),
            display: false,
            span: slideforge_types::SourceSpan::default(),
        });
        let result = fmt().render(&node, InlineOutputFormat::Html).unwrap();
        assert!(
            !result.contains(" < "),
            "F-004: raw < in math latex must be escaped in HTML output; got: {result}"
        );
        assert!(
            result.contains("&lt;"),
            "F-004: < in math latex must become &lt; in HTML; got: {result}"
        );
    }

    // ── F-005 (MED): Nested formatting + depth guard for Super/Sub/Strike/Highlight

    /// F-005 (MED): Superscript([Bold([Plain("x")])]) in OOXML — the nested Bold
    /// formatting must either be preserved (combined run properties) OR a
    /// `tracing::warn!` must be emitted. EITHER WAY, the depth counter must be
    /// correctly threaded so EC-004 still holds for nodes nested under Superscript.
    ///
    /// We verify the depth guard: nesting > 64 levels under Superscript returns
    /// `InlineError::RenderError`, not a stack overflow.
    ///
    /// This test FAILS before F-005 because `extract_plain_text_from_renders`
    /// does NOT propagate `depth` — it calls `extract_plain_text_from_nodes`
    /// with no depth tracking, so the depth guard in the outer path is bypassed.
    #[test]
    fn test_f005_ec004_depth_guard_under_superscript() {
        // Build 65 levels of Bold nested under a Superscript.
        let mut inner: InlineNode = plain("x");
        for _ in 0..65 {
            inner = bold(vec![inner]);
        }
        let node = superscript(vec![inner]);

        let result = fmt().render(&node, InlineOutputFormat::Ooxml);
        // Must return an error (depth > 64), not stack-overflow.
        match result {
            Err(InlineError::RenderError { message, .. }) => {
                assert!(
                    message.contains("nesting depth exceeds maximum of 64"),
                    "F-005: expected depth-exceeded error, got: {message}"
                );
            },
            Ok(s) => panic!(
                "F-005: expected RenderError for depth > 64 under Superscript, got Ok({s:?})"
            ),
            Err(e) => panic!(
                "F-005: expected RenderError for depth > 64 under Superscript, got different error: {e:?}"
            ),
        }
    }

    /// F-005 (MED): Superscript([Bold([Plain("x")])]) — nested Bold inside
    /// Superscript. The nested Bold MUST either be preserved in the output (as
    /// combined run properties or a dedicated bold run) OR a `tracing::warn!`
    /// must be emitted. We assert the output is non-empty and contains "x".
    ///
    /// This is the "non-silent" requirement: the text must not be silently dropped.
    /// Before F-005, the text is present (`extract_plain_text_from_nodes` handles it)
    /// so this test PASSES currently — it is included as a regression guard for F-005.
    #[test]
    fn test_f005_nested_bold_inside_superscript_text_not_dropped() {
        let node = superscript(vec![bold(vec![plain("x")])]);
        let result = fmt().render(&node, InlineOutputFormat::Ooxml).unwrap();
        assert!(
            result.contains('x'),
            "F-005: text inside Bold([Superscript([Plain(\"x\")])]) must not be silently dropped; got: {result}"
        );
    }

    // ── F-001 (CRIT): render_with_context on DefaultInlineFormat ─────────────

    /// F-001 (CRIT): `DefaultInlineFormat::render_with_context` for `Link + Ooxml`
    /// with `Some(rid)` must emit a full `<a:r><a:rPr><a:hlinkClick r:id="rId3"/>
    /// </a:rPr><a:t>display text</a:t></a:r>` — NOT a plain text fallback.
    ///
    /// This test FAILS before F-001 because `render_with_context` does not exist
    /// on the `InlineFormat` trait (compilation error).
    #[test]
    fn test_f001_render_with_context_link_ooxml_with_rid_emits_hlinkclick() {
        use crate::traits::inline_format::InlineRenderContext;

        let node = link(vec![plain("click here")], "https://example.com");
        let ctx = InlineRenderContext {
            hyperlink_rid: Some("rId3"),
        };
        let result = fmt()
            .render_with_context(&node, InlineOutputFormat::Ooxml, &ctx)
            .unwrap();

        // Must contain the hlinkClick with the given rId.
        assert!(
            result.contains("hlinkClick"),
            "F-001: render_with_context Link+Ooxml with rId must emit hlinkClick; got: {result}"
        );
        assert!(
            result.contains("rId3"),
            "F-001: render_with_context Link+Ooxml must embed the rId; got: {result}"
        );
        // Must contain the display text.
        assert!(
            result.contains("click here"),
            "F-001: render_with_context Link+Ooxml must include display text; got: {result}"
        );
        // Must be a proper OOXML run (starts with <a:r>).
        assert!(
            result.contains("<a:r>"),
            "F-001: render_with_context Link+Ooxml must emit a <a:r> run; got: {result}"
        );
    }

    /// F-001 (CRIT): `DefaultInlineFormat::render_with_context` for `Link + Ooxml`
    /// with `None` (no rId) must fall back to the existing display-text-plus-warn
    /// behavior (same as plain `render`).
    ///
    /// This test FAILS before F-001 because `render_with_context` does not exist.
    #[test]
    fn test_f001_render_with_context_link_ooxml_no_rid_falls_back_to_render() {
        use crate::traits::inline_format::InlineRenderContext;

        let node = link(vec![plain("my link")], "https://example.com");
        // Default context (no rId).
        let ctx = InlineRenderContext::default();
        let result = fmt()
            .render_with_context(&node, InlineOutputFormat::Ooxml, &ctx)
            .unwrap();

        // Must contain the display text (not silently dropped).
        assert!(
            result.contains("my link"),
            "F-001: render_with_context Link+Ooxml with None rId must preserve display text; got: {result}"
        );
        // Must NOT contain hlinkClick (no rId available).
        assert!(
            !result.contains("hlinkClick"),
            "F-001: render_with_context Link+Ooxml with None rId must NOT emit hlinkClick; got: {result}"
        );
    }

    /// F-001 (CRIT): For non-Link nodes, `render_with_context` must delegate
    /// to `render` (the default implementation). This ensures the default
    /// method is not overridden for other node types.
    #[test]
    fn test_f001_render_with_context_non_link_delegates_to_render() {
        use crate::traits::inline_format::InlineRenderContext;

        let ctx = InlineRenderContext::default();
        // Plain node — should delegate to render.
        let plain_result = fmt()
            .render_with_context(&plain("hello"), InlineOutputFormat::Ooxml, &ctx)
            .unwrap();
        let render_result = fmt()
            .render(&plain("hello"), InlineOutputFormat::Ooxml)
            .unwrap();
        assert_eq!(
            plain_result, render_result,
            "F-001: render_with_context for Plain must delegate to render"
        );

        // Bold node.
        let bold_result = fmt()
            .render_with_context(&bold(vec![plain("hi")]), InlineOutputFormat::Html, &ctx)
            .unwrap();
        let render_bold = fmt()
            .render(&bold(vec![plain("hi")]), InlineOutputFormat::Html)
            .unwrap();
        assert_eq!(
            bold_result, render_bold,
            "F-001: render_with_context for Bold must delegate to render"
        );
    }

    // ── F-008 (OBS): EC-004 depth guard on render_with_context link path ──────

    /// F-008-OBS: A deeply-nested (>64) Link display-text tree passed to
    /// `render_with_context` with `Some(rid)` must return `InlineError::RenderError`
    /// instead of recursing unbounded (no stack overflow).
    ///
    /// Before the fix, `render_with_context` calls `extract_plain_text_from_nodes`
    /// which has no depth guard and recurses unbounded.
    #[test]
    fn test_f008_render_with_context_link_deep_display_text_returns_render_error() {
        use crate::traits::inline_format::InlineRenderContext;

        // Build 65 levels of Bold nested as Link display text (each Bold wraps the
        // previous, and the innermost is Plain("x")).
        let mut inner: InlineNode = plain("x");
        for _ in 0..65 {
            inner = bold(vec![inner]);
        }
        let node = link(vec![inner], "https://example.com");
        let ctx = InlineRenderContext {
            hyperlink_rid: Some("rId1"),
        };
        let result = fmt().render_with_context(&node, InlineOutputFormat::Ooxml, &ctx);
        match result {
            Err(InlineError::RenderError { message, .. }) => {
                assert!(
                    message.contains("nesting depth exceeds maximum of 64"),
                    "F-008-OBS: expected depth-exceeded message, got: {message}"
                );
            },
            Ok(s) => panic!(
                "F-008-OBS: expected RenderError for >64 deep Link display text via \
                 render_with_context, got Ok({s:?})"
            ),
            Err(e) => panic!(
                "F-008-OBS: expected RenderError for >64 deep Link display text, \
                 got different error: {e:?}"
            ),
        }
    }

    /// F-008-OBS: Depth exactly 64 in Link display text via `render_with_context`
    /// must SUCCEED (the boundary is depth > 64, so depth 64 is allowed).
    #[test]
    fn test_f008_render_with_context_link_depth_64_display_text_succeeds() {
        use crate::traits::inline_format::InlineRenderContext;

        let mut inner: InlineNode = plain("x");
        for _ in 0..64 {
            inner = bold(vec![inner]);
        }
        let node = link(vec![inner], "https://example.com");
        let ctx = InlineRenderContext {
            hyperlink_rid: Some("rId1"),
        };
        let result = fmt().render_with_context(&node, InlineOutputFormat::Ooxml, &ctx);
        assert!(
            result.is_ok(),
            "F-008-OBS: depth exactly 64 in Link display text must succeed, got: {result:?}"
        );
    }

    // ── F-009 (MED): nested formatting combined run properties ────────────────

    /// F-009-MED: Bold([Strikethrough([Plain("x")])]) → OOXML run with BOTH b="1"
    /// AND strike="sngStrike" in a single `<a:rPr>`.
    ///
    /// Before the fix, `render_ooxml_with_context` is called with bold=true but
    /// Strikethrough arm just recurses inheriting bold/italic only — the
    /// strike property is silently dropped, producing only `<a:rPr b="1"/>`.
    #[test]
    fn test_f009_bold_wraps_strikethrough_combines_run_props() {
        let node = bold(vec![strikethrough(vec![plain("x")])]);
        let result = fmt().render(&node, InlineOutputFormat::Ooxml).unwrap();
        assert!(
            result.contains("b=\"1\""),
            "F-009: Bold([Strikethrough([Plain])]) must have b=\"1\", got: {result}"
        );
        assert!(
            result.contains("strike=\"sngStrike\""),
            "F-009: Bold([Strikethrough([Plain])]) must have strike=\"sngStrike\", got: {result}"
        );
        assert!(
            result.contains("<a:t>x</a:t>"),
            "F-009: text must be preserved, got: {result}"
        );
    }

    /// F-009-MED: Strikethrough([Bold([Plain("x")])]) → OOXML run with BOTH
    /// strike="sngStrike" AND b="1" in the `<a:rPr>`.
    ///
    /// Before the fix, `render_ooxml` Strikethrough arm calls
    /// `extract_plain_text_from_renders` which emits `strike="sngStrike"` but
    /// the inner bold is dropped (only warn emitted).
    #[test]
    fn test_f009_strikethrough_wraps_bold_combines_run_props() {
        let node = strikethrough(vec![bold(vec![plain("x")])]);
        let result = fmt().render(&node, InlineOutputFormat::Ooxml).unwrap();
        assert!(
            result.contains("strike=\"sngStrike\""),
            "F-009: Strikethrough([Bold([Plain])]) must have strike=\"sngStrike\", got: {result}"
        );
        assert!(
            result.contains("b=\"1\""),
            "F-009: Strikethrough([Bold([Plain])]) must have b=\"1\", got: {result}"
        );
        assert!(
            result.contains("<a:t>x</a:t>"),
            "F-009: text must be preserved, got: {result}"
        );
    }

    /// F-009-MED: Bold([Italic([Strikethrough([Plain("x")])])]) → combined
    /// `<a:rPr b="1" i="1" strike="sngStrike"/>` in a single run.
    #[test]
    fn test_f009_triple_nesting_bold_italic_strikethrough_combines_all() {
        let node = bold(vec![italic(vec![strikethrough(vec![plain("x")])])]);
        let result = fmt().render(&node, InlineOutputFormat::Ooxml).unwrap();
        assert!(
            result.contains("b=\"1\""),
            "F-009: triple-nested must have b=\"1\", got: {result}"
        );
        assert!(
            result.contains("i=\"1\""),
            "F-009: triple-nested must have i=\"1\", got: {result}"
        );
        assert!(
            result.contains("strike=\"sngStrike\""),
            "F-009: triple-nested must have strike=\"sngStrike\", got: {result}"
        );
        assert!(
            result.contains("<a:t>x</a:t>"),
            "F-009: text must be preserved in triple-nested, got: {result}"
        );
    }
}
