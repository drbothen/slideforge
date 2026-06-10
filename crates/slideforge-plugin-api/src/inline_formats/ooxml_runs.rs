//! Neutral typed OOXML run intermediate representation.
//!
//! [`OoxmlRun`] is the canonical intermediate representation shared by the
//! slide-body and notes-slide PPTX serializers. Neither serializer constructs
//! `<a:r>` markup directly; both call [`render_inline_nodes_to_runs`] and then
//! serialize the resulting `Vec<OoxmlRun>`.
//!
//! ## Design rationale (ADR-024)
//!
//! Having two generators for the same `&[InlineNode]` → OOXML input domain
//! caused three independent divergence findings (ADV-P11-HIGH-001, F-P15-HIGH-001,
//! F-P16-M1). `OoxmlRun` + `render_inline_nodes_to_runs` is the single engine
//! that both consumers use; divergence via a second implementation path is now
//! impossible by construction.
//!
//! ## Child-element ordering (ADR-024 INV-2 / ADV-P11-HIGH-001)
//!
//! [`serialize_ooxml_run`] is the SINGLE site for `<a:rPr>` construction on the
//! notes path. Attribute and child-element ordering is structurally determined by
//! the function's source order (not by any caller):
//!
//! Attributes first (`b`, `i`, `baseline`, `strike`), then child elements:
//! 1. `<a:highlight><a:srgbClr val="FFFF00"/></a:highlight>` (highlight)
//! 2. `<a:latin typeface="Courier New"/>` (code font)
//! 3. `<a:hlinkClick r:id="..."/>` (hyperlink — MUST be last child)
//!
//! `<a:highlight>` is a CHILD ELEMENT of `<a:rPr>` — never the invalid
//! `highlight="yellow"` attribute (ADV-P11-HIGH-001).
//!
//! ## Hyperlink invariant (ADR-024 INV-4)
//!
//! Every `<a:hlinkClick>` run references a registered External relationship, AND
//! every registered External relationship is referenced by at least one
//! `<a:hlinkClick>` run (no orphan, no dangling).
//!
//! The count-equality form (`external_rel_count == hlinkclick_count`) is FALSE
//! for multi-leaf link display text: a single External rel can back N
//! `<a:hlinkClick>` runs when the display text tree has multiple leaf nodes.
//! The correct invariant is the reference-set form: all-rels-referenced AND
//! all-clicks-backed.

use crate::traits::inline_format::InlineError;
use slideforge_types::InlineNode;

/// Maximum allowed inline nesting depth (BC-3.05.001 invariant).
const MAX_DEPTH: usize = 64;

/// A single OOXML text run, ready for serialization into `<a:r>` markup.
///
/// Represents the complete run-property state at a single leaf in the inline
/// node tree plus the text content. All eight OOXML run-property axes are
/// captured independently — they are not mutually exclusive and may be combined
/// (e.g., bold + italic + strikethrough in a single run).
///
/// `OoxmlRun` is the canonical intermediate representation shared by the slide-body
/// and notes-slide PPTX serializers. Neither serializer constructs `<a:r>` markup
/// directly; both call [`render_inline_nodes_to_runs`] and then serialize the
/// resulting `Vec<OoxmlRun>`.
///
/// # Hash/Eq/Clone (ADR-013 / comemo compatibility)
///
/// `OoxmlRun` derives `Hash + Eq + Clone` for comemo compatibility. The order
/// of runs in `Vec<OoxmlRun>` is deterministic (document-order depth-first
/// traversal from [`render_inline_nodes_to_runs`]).
///
/// # Multiple bool fields
///
/// OOXML run properties are independent, combinable axes — they are NOT
/// mutually exclusive states. A run can simultaneously be bold, italic,
/// struck-through, and highlighted. Bit-flags would reduce readability and
/// add unsafe index arithmetic. Six named booleans + `Option<i32>` baseline is
/// the clearest representation for a six-axis accumulator.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OoxmlRun {
    /// The plain-text content of the run. XML-escaped by [`serialize_ooxml_run`].
    pub text: String,
    /// `b="1"` — from a `Bold` ancestor.
    pub bold: bool,
    /// `i="1"` — from an `Italic` ancestor.
    pub italic: bool,
    /// `baseline="{n}"` — from a `Superscript` (+30 000) or `Subscript` (−25 000).
    /// `None` when no baseline shift applies.
    pub baseline: Option<i32>,
    /// `strike="sngStrike"` — from a `Strikethrough` ancestor.
    pub strike: bool,
    /// Triggers `<a:highlight><a:srgbClr val="FFFF00"/></a:highlight>` child element
    /// inside `<a:rPr>` — from a `Highlight` ancestor.
    ///
    /// ADV-P11-HIGH-001: `DrawingML` `CT_TextCharacterProperties` (`<a:rPr>`) has NO
    /// `highlight` attribute. The schema-correct form is the `<a:highlight>` child element.
    pub highlight: bool,
    /// Triggers `<a:latin typeface="Courier New"/>` child element — set by `Code` leaf.
    pub code_font: bool,
    /// Relationship ID for `<a:hlinkClick r:id="..."/>` child element of `<a:rPr>`.
    ///
    /// `None` for all non-hyperlink contexts.
    pub hyperlink_rid: Option<String>,
}

impl OoxmlRun {
    /// Construct an `OoxmlRun` with all formatting flags off.
    #[must_use]
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            bold: false,
            italic: false,
            baseline: None,
            strike: false,
            highlight: false,
            code_font: false,
            hyperlink_rid: None,
        }
    }
}

/// Serialize a single [`OoxmlRun`] to `<a:r>…</a:r>` markup.
///
/// This is the SINGLE site for `<a:rPr>` construction on the notes serialization
/// path (ADR-024 INV-2). Child ordering is structurally guaranteed by source order:
///
/// 1. Attributes: `b`, `i`, `baseline`, `strike` (all are XML attributes on `<a:rPr>`)
/// 2. Child elements in `CT_TextCharacterProperties` order:
///    a. `<a:highlight><a:srgbClr val="FFFF00"/></a:highlight>` (when `highlight=true`)
///    b. `<a:latin typeface="Courier New"/>` (when `code_font=true`)
///    c. `<a:hlinkClick r:id="..."/>` (when `hyperlink_rid` is `Some`) — LAST child
///
/// Returns an empty string if `text` is empty (empty runs are inert in OOXML).
#[must_use]
pub fn serialize_ooxml_run(run: &OoxmlRun) -> String {
    if run.text.is_empty() {
        return String::new();
    }

    let needs_rpr = run.bold
        || run.italic
        || run.baseline.is_some()
        || run.strike
        || run.highlight
        || run.code_font
        || run.hyperlink_rid.is_some();

    let mut out = String::new();
    out.push_str("<a:r>");

    if needs_rpr {
        let has_children = run.highlight || run.code_font || run.hyperlink_rid.is_some();

        out.push_str("<a:rPr");

        // Attributes first (b, i, baseline, strike — XML attributes on <a:rPr>).
        if run.bold {
            out.push_str(" b=\"1\"");
        }
        if run.italic {
            out.push_str(" i=\"1\"");
        }
        if let Some(baseline) = run.baseline {
            out.push_str(" baseline=\"");
            out.push_str(&baseline.to_string());
            out.push('"');
        }
        if run.strike {
            out.push_str(" strike=\"sngStrike\"");
        }
        // NOTE: `highlight` is NOT an attribute on <a:rPr> — it is a child element.
        // ADV-P11-HIGH-001: `CT_TextCharacterProperties` has no `highlight` attribute.

        if has_children {
            // Close the opening tag (not self-closing) — child elements follow.
            out.push('>');

            if run.highlight {
                // ADV-P11-HIGH-001: schema-correct background highlight child element.
                // `<a:highlight>` with `<a:srgbClr val="FFFF00"/>` = yellow background.
                // Risk 3 per ADR-024: NEVER emit highlight="yellow" as an attribute.
                out.push_str("<a:highlight><a:srgbClr val=\"FFFF00\"/></a:highlight>");
            }
            if run.code_font {
                out.push_str("<a:latin typeface=\"Courier New\"/>");
            }
            if let Some(rid) = &run.hyperlink_rid {
                // `<a:hlinkClick>` is the last child element of `<a:rPr>` per the
                // DrawingML CT_TextCharacterProperties schema ordering.
                out.push_str("<a:hlinkClick r:id=\"");
                out.push_str(&xml_escape(rid));
                out.push_str("\"/>");
            }

            out.push_str("</a:rPr>");
        } else {
            // No child elements — self-close.
            out.push_str("/>");
        }
    }

    out.push_str("<a:t>");
    out.push_str(&xml_escape(&run.text));
    out.push_str("</a:t></a:r>");
    out
}

/// Render a sequence of [`InlineNode`]s to a flat list of typed OOXML runs.
///
/// This is the SINGLE inline-run generator for all PPTX serializers (ADR-024).
/// Both the slide-body path and the notes-slide path call this function.
///
/// # Parameters
///
/// - `nodes`: the top-level inline node slice to render.
/// - `hlink_resolver`: a callback that maps a URL to a pre-registered relationship
///   ID, or `None` if the URL has not been registered or is not safe-scheme.
///   The resolver is called once per `Link` node encountered during recursive
///   descent (both top-level and inside formatting wrappers). The notes path and
///   body path provide different resolver closures backed by their respective
///   `unique_hlinks` / `hlink_map` data structures.
///
/// # Returns
///
/// A `Vec<OoxmlRun>` in document order. Empty runs (where `text` is empty after
/// plain-text extraction) are omitted.
///
/// # Errors
///
/// Returns [`InlineError::RenderError`] if inline nesting depth exceeds 64
/// (BC-3.05.001 invariant).
///
/// # Invariants
///
/// - INV-5 (ADR-024): `hlink_resolver` is NOT called for `Link` nodes that
///   appear inside another `Link`'s display-text children (F-040-P3-001 rule).
/// - INV-6 (ADR-024): A `Link` inside another `Link`'s display text inherits
///   the outer `Link`'s `hyperlink_rid` on its leaf runs (Pass-15 behavior).
/// - INV-8 (ADR-024): `InlineNode::Math` produces a `tracing::warn!` and an
///   `OoxmlRun` with the LaTeX source as text (EC-001 fallback).
pub fn render_inline_nodes_to_runs(
    nodes: &[InlineNode],
    hlink_resolver: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<OoxmlRun>, InlineError> {
    let mut runs = Vec::new();
    render_nodes_recursive(nodes, &RunProps::new(), hlink_resolver, false, 0, &mut runs)?;
    Ok(runs)
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal recursive engine
// ─────────────────────────────────────────────────────────────────────────────

/// Accumulated run properties threaded through the recursive descent.
///
/// Formatting nodes set their property on this struct and recurse into
/// children with the updated state. At each leaf exactly one `OoxmlRun` is
/// produced with ALL accumulated properties.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
struct RunProps {
    /// `b="1"` — accumulated from a `Bold` ancestor node.
    bold: bool,
    /// `i="1"` — accumulated from an `Italic` ancestor node.
    italic: bool,
    /// `baseline="{n}"` — +30 000 for Superscript, −25 000 for Subscript.
    baseline: Option<i32>,
    /// `strike="sngStrike"` — accumulated from a `Strikethrough` ancestor.
    strike: bool,
    /// `<a:highlight>` child element — accumulated from a `Highlight` ancestor.
    highlight: bool,
    /// `<a:latin typeface="Courier New"/>` — set by `Code` leaf.
    code_font: bool,
    /// Set when inside a `Link`'s display-text recursion (INV-5: suppress
    /// nested resolver calls) or when inheriting an outer `Link`'s rId (INV-6).
    hyperlink_rid: Option<String>,
}

impl RunProps {
    /// Construct a `RunProps` with all formatting flags off and no hyperlink.
    fn new() -> Self {
        Self {
            bold: false,
            italic: false,
            baseline: None,
            strike: false,
            highlight: false,
            code_font: false,
            hyperlink_rid: None,
        }
    }

    /// Convert accumulated run properties and a text string into an [`OoxmlRun`].
    fn to_ooxml_run(&self, text: String) -> OoxmlRun {
        OoxmlRun {
            text,
            bold: self.bold,
            italic: self.italic,
            baseline: self.baseline,
            strike: self.strike,
            highlight: self.highlight,
            code_font: self.code_font,
            hyperlink_rid: self.hyperlink_rid.clone(),
        }
    }
}

/// Internal recursive renderer. Appends `OoxmlRun` entries to `runs`.
///
/// `inside_link_display_text`: when `true`, we are recursing into a `Link`'s
/// display-text children. In this context:
/// - `Link` nodes do NOT call `hlink_resolver` (INV-5 — F-040-P3-001).
/// - `Link` nodes inherit the accumulated `props.hyperlink_rid` (INV-6).
fn render_nodes_recursive(
    nodes: &[InlineNode],
    props: &RunProps,
    hlink_resolver: &dyn Fn(&str) -> Option<String>,
    inside_link_display_text: bool,
    depth: usize,
    runs: &mut Vec<OoxmlRun>,
) -> Result<(), InlineError> {
    for node in nodes {
        render_node_recursive(
            node,
            props,
            hlink_resolver,
            inside_link_display_text,
            depth,
            runs,
        )?;
    }
    Ok(())
}

/// Dispatch a single [`InlineNode`] to the appropriate run-building logic,
/// updating `props` for formatting nodes and appending leaf runs to `runs`.
fn render_node_recursive(
    node: &InlineNode,
    props: &RunProps,
    hlink_resolver: &dyn Fn(&str) -> Option<String>,
    inside_link_display_text: bool,
    depth: usize,
    runs: &mut Vec<OoxmlRun>,
) -> Result<(), InlineError> {
    if depth > MAX_DEPTH {
        return Err(InlineError::RenderError {
            node_kind: node.kind_name().to_owned(),
            message: "inline nesting depth exceeds maximum of 64".to_owned(),
        });
    }

    match node {
        // ── Leaf: Plain text and Xref ─────────────────────────────────────
        InlineNode::Plain(text) | InlineNode::Xref(text) => {
            let run = props.to_ooxml_run(text.as_ref().to_owned());
            if !run.text.is_empty() {
                runs.push(run);
            }
        },

        // ── Leaf: Code — adds monospace font to accumulated props ─────────
        InlineNode::Code(text) => {
            let mut code_props = props.clone();
            code_props.code_font = true;
            let run = code_props.to_ooxml_run(text.as_ref().to_owned());
            if !run.text.is_empty() {
                runs.push(run);
            }
        },

        // ── Formatting wrappers ───────────────────────────────────────────
        InlineNode::Bold(children) => {
            let mut child_props = props.clone();
            child_props.bold = true;
            render_nodes_recursive(
                children,
                &child_props,
                hlink_resolver,
                inside_link_display_text,
                depth + 1,
                runs,
            )?;
        },
        InlineNode::Italic(children) => {
            let mut child_props = props.clone();
            child_props.italic = true;
            render_nodes_recursive(
                children,
                &child_props,
                hlink_resolver,
                inside_link_display_text,
                depth + 1,
                runs,
            )?;
        },
        InlineNode::Strikethrough(children) => {
            let mut child_props = props.clone();
            child_props.strike = true;
            render_nodes_recursive(
                children,
                &child_props,
                hlink_resolver,
                inside_link_display_text,
                depth + 1,
                runs,
            )?;
        },
        InlineNode::Superscript(children) => {
            let mut child_props = props.clone();
            child_props.baseline = Some(30_000);
            render_nodes_recursive(
                children,
                &child_props,
                hlink_resolver,
                inside_link_display_text,
                depth + 1,
                runs,
            )?;
        },
        InlineNode::Subscript(children) => {
            let mut child_props = props.clone();
            child_props.baseline = Some(-25_000);
            render_nodes_recursive(
                children,
                &child_props,
                hlink_resolver,
                inside_link_display_text,
                depth + 1,
                runs,
            )?;
        },
        InlineNode::Highlight(children) => {
            let mut child_props = props.clone();
            child_props.highlight = true;
            render_nodes_recursive(
                children,
                &child_props,
                hlink_resolver,
                inside_link_display_text,
                depth + 1,
                runs,
            )?;
        },
        InlineNode::Footnote(children) => {
            // Footnote renders its inner body content inline.
            // Numbered-marker form deferred per STORY-085 spec F-010.
            tracing::debug!("Footnote marker numbering deferred");
            render_nodes_recursive(
                children,
                props,
                hlink_resolver,
                inside_link_display_text,
                depth + 1,
                runs,
            )?;
        },

        // ── Link: the F-P16-M1 fix lives here ────────────────────────────
        InlineNode::Link { url, text } => {
            // INV-5 (F-040-P3-001): Do NOT call hlink_resolver for a Link that
            // appears inside another Link's display-text children.
            // INV-6: Inherit the outer Link's hyperlink_rid on nested runs.
            let rid = if inside_link_display_text {
                // Already inside a Link's display text — inherit the outer rId.
                // No resolver call for the inner link (INV-5).
                props.hyperlink_rid.clone()
            } else {
                // Top-level (or inside formatting wrappers, but not in link
                // display text) — call the resolver.
                hlink_resolver(url.as_ref())
            };

            if rid.is_none() && !inside_link_display_text {
                // EC-002 fallback — no rId available (or unsafe-scheme URL already
                // filtered by the resolver closure).
                tracing::warn!(
                    url = %url,
                    "Hyperlink relationship not registered for OOXML link to {url}; \
                     emitting display text as plain run (EC-002 / v1.0 limitation)"
                );
            }

            // Recurse into the link's display text with the resolved rId
            // threaded through. This is the core F-P16-M1 fix: display-text
            // children are recursed with full RunProps accumulation (not flattened
            // to plain text), so Bold/Italic/etc. inside link display text is
            // preserved.
            let mut link_props = props.clone();
            link_props.hyperlink_rid = rid;

            // Mark that we are inside a Link's display text so nested Links
            // suppress resolver calls (INV-5).
            render_nodes_recursive(
                text,
                &link_props,
                hlink_resolver,
                true, // inside_link_display_text = true for all descendants
                depth + 1,
                runs,
            )?;
        },

        // ── Math: EC-001 fallback ─────────────────────────────────────────
        InlineNode::Math(math_node) => {
            tracing::warn!(
                "Math OMML rendering not pre-computed for OOXML; \
                 emitting LaTeX source as plain text run (EC-001)"
            );
            let run = props.to_ooxml_run(math_node.latex.as_ref().to_owned());
            if !run.text.is_empty() {
                runs.push(run);
            }
        },
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helper
// ─────────────────────────────────────────────────────────────────────────────

/// XML-escape a string for safe embedding in XML element text content (`<a:t>`)
/// or attribute values (`r:id="..."`).
///
/// Replaces the XML-reserved characters:
/// - `&` → `&amp;` (must be first to avoid double-escaping)
/// - `<` → `&lt;`
/// - `>` → `&gt;`
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests — ADR-024 Step 1 + Step 2
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::sync::Arc;

    use slideforge_types::{InlineNode, MathNode, SourceSpan};

    use super::{OoxmlRun, render_inline_nodes_to_runs, serialize_ooxml_run};

    // ── Helper constructors ───────────────────────────────────────────────

    fn plain_run(text: &str) -> OoxmlRun {
        OoxmlRun::plain(text)
    }

    fn inline_plain(s: &str) -> InlineNode {
        InlineNode::Plain(Arc::from(s))
    }

    fn inline_bold(children: Vec<InlineNode>) -> InlineNode {
        InlineNode::Bold(children)
    }

    fn inline_italic(children: Vec<InlineNode>) -> InlineNode {
        InlineNode::Italic(children)
    }

    fn inline_link(text: Vec<InlineNode>, url: &str) -> InlineNode {
        InlineNode::Link {
            text,
            url: Arc::from(url),
        }
    }

    fn inline_code(s: &str) -> InlineNode {
        InlineNode::Code(Arc::from(s))
    }

    fn inline_math(latex: &str) -> InlineNode {
        InlineNode::Math(MathNode {
            latex: Arc::from(latex),
            display: false,
            span: SourceSpan::default(),
        })
    }

    fn inline_highlight(children: Vec<InlineNode>) -> InlineNode {
        InlineNode::Highlight(children)
    }

    fn inline_superscript(children: Vec<InlineNode>) -> InlineNode {
        InlineNode::Superscript(children)
    }

    fn inline_subscript(children: Vec<InlineNode>) -> InlineNode {
        InlineNode::Subscript(children)
    }

    fn inline_strikethrough(children: Vec<InlineNode>) -> InlineNode {
        InlineNode::Strikethrough(children)
    }

    fn no_resolver(url: &str) -> Option<String> {
        let _ = url;
        None
    }

    fn fixed_resolver(rid: &'static str) -> impl Fn(&str) -> Option<String> {
        move |_url| Some(rid.to_owned())
    }

    // ── Step 1: serialize_ooxml_run unit tests ────────────────────────────

    /// All properties off → `<a:r><a:t>{text}</a:t></a:r>`
    #[test]
    fn test_adr024_step1_serialize_plain() {
        let run = plain_run("hello");
        assert_eq!(serialize_ooxml_run(&run), "<a:r><a:t>hello</a:t></a:r>");
    }

    /// Empty text → empty string (no run emitted)
    #[test]
    fn test_adr024_step1_serialize_empty_text_returns_empty() {
        let run = plain_run("");
        assert_eq!(serialize_ooxml_run(&run), "");
    }

    /// `bold=true` → `<a:rPr b="1"/>` before `<a:t>`
    #[test]
    fn test_adr024_step1_serialize_bold() {
        let run = OoxmlRun {
            bold: true,
            ..OoxmlRun::plain("hi")
        };
        let xml = serialize_ooxml_run(&run);
        assert!(xml.contains("<a:rPr b=\"1\"/>"), "got: {xml}");
        assert!(xml.contains("<a:t>hi</a:t>"), "got: {xml}");
        // rPr must appear before t
        let rpr_pos = xml.find("<a:rPr").unwrap();
        let t_pos = xml.find("<a:t>").unwrap();
        assert!(rpr_pos < t_pos, "rPr must precede t in: {xml}");
    }

    /// `italic=true` → `<a:rPr i="1"/>`
    #[test]
    fn test_adr024_step1_serialize_italic() {
        let run = OoxmlRun {
            italic: true,
            ..OoxmlRun::plain("em")
        };
        let xml = serialize_ooxml_run(&run);
        assert!(xml.contains("<a:rPr i=\"1\"/>"), "got: {xml}");
        assert!(xml.contains("<a:t>em</a:t>"), "got: {xml}");
    }

    /// `bold=true, italic=true` → `<a:rPr b="1" i="1"/>`
    #[test]
    fn test_adr024_step1_serialize_bold_italic_combined() {
        let run = OoxmlRun {
            bold: true,
            italic: true,
            ..OoxmlRun::plain("bi")
        };
        let xml = serialize_ooxml_run(&run);
        assert!(xml.contains("<a:rPr b=\"1\" i=\"1\"/>"), "got: {xml}");
    }

    /// `baseline=Some(30000)` → `<a:rPr baseline="30000"/>`
    #[test]
    fn test_adr024_step1_serialize_baseline_superscript() {
        let run = OoxmlRun {
            baseline: Some(30_000),
            ..OoxmlRun::plain("sup")
        };
        let xml = serialize_ooxml_run(&run);
        assert!(xml.contains("baseline=\"30000\""), "got: {xml}");
    }

    /// `baseline=Some(-25000)` → `<a:rPr baseline="-25000"/>`
    #[test]
    fn test_adr024_step1_serialize_baseline_subscript() {
        let run = OoxmlRun {
            baseline: Some(-25_000),
            ..OoxmlRun::plain("sub")
        };
        let xml = serialize_ooxml_run(&run);
        assert!(xml.contains("baseline=\"-25000\""), "got: {xml}");
    }

    /// `strike=true` → `<a:rPr strike="sngStrike"/>`
    #[test]
    fn test_adr024_step1_serialize_strike() {
        let run = OoxmlRun {
            strike: true,
            ..OoxmlRun::plain("del")
        };
        let xml = serialize_ooxml_run(&run);
        assert!(xml.contains("strike=\"sngStrike\""), "got: {xml}");
    }

    /// `highlight=true` → `<a:rPr><a:highlight><a:srgbClr val="FFFF00"/></a:highlight></a:rPr>`
    /// (child element form, NOT attribute — ADV-P11-HIGH-001 / Risk 3 per ADR-024)
    #[test]
    fn test_adr024_step1_serialize_highlight_child_element_not_attribute() {
        let run = OoxmlRun {
            highlight: true,
            ..OoxmlRun::plain("marked")
        };
        let xml = serialize_ooxml_run(&run);
        // Must use the child element form.
        assert!(
            xml.contains("<a:highlight>"),
            "ADV-P11-HIGH-001: must use <a:highlight> child element; got: {xml}"
        );
        assert!(
            xml.contains("FFFF00"),
            "must contain FFFF00 color; got: {xml}"
        );
        // Must NOT use the invalid attribute form.
        assert!(
            !xml.contains("highlight=\"yellow\""),
            "ADV-P11-HIGH-001: must NOT emit highlight attribute; got: {xml}"
        );
    }

    /// `code_font=true` → `<a:rPr><a:latin typeface="Courier New"/></a:rPr>`
    #[test]
    fn test_adr024_step1_serialize_code_font() {
        let run = OoxmlRun {
            code_font: true,
            ..OoxmlRun::plain("fn foo()")
        };
        let xml = serialize_ooxml_run(&run);
        assert!(
            xml.contains("<a:latin typeface=\"Courier New\"/>"),
            "got: {xml}"
        );
    }

    /// `hyperlink_rid=Some("rId3")` → `<a:rPr><a:hlinkClick r:id="rId3"/></a:rPr>`
    #[test]
    fn test_adr024_step1_serialize_hyperlink_rid() {
        let run = OoxmlRun {
            hyperlink_rid: Some("rId3".to_owned()),
            ..OoxmlRun::plain("click")
        };
        let xml = serialize_ooxml_run(&run);
        assert!(xml.contains("<a:hlinkClick r:id=\"rId3\"/>"), "got: {xml}");
    }

    /// Combined: bold + `hyperlink_rid` → `<a:rPr b="1"><a:hlinkClick r:id="rId3"/></a:rPr>`
    #[test]
    fn test_adr024_step1_serialize_bold_hyperlink_combined() {
        let run = OoxmlRun {
            bold: true,
            hyperlink_rid: Some("rId3".to_owned()),
            ..OoxmlRun::plain("here")
        };
        let xml = serialize_ooxml_run(&run);
        assert!(xml.contains("b=\"1\""), "got: {xml}");
        assert!(xml.contains("<a:hlinkClick r:id=\"rId3\"/>"), "got: {xml}");
        // b attribute must appear before hlinkClick child.
        let b_pos = xml.find("b=\"1\"").unwrap();
        let hlink_pos = xml.find("<a:hlinkClick").unwrap();
        assert!(
            b_pos < hlink_pos,
            "b attribute must precede hlinkClick in: {xml}"
        );
    }

    /// Child element ordering: highlight before `code_font` before `hlinkClick`.
    #[test]
    fn test_adr024_step1_serialize_child_element_ordering() {
        let run = OoxmlRun {
            highlight: true,
            code_font: true,
            hyperlink_rid: Some("rId5".to_owned()),
            ..OoxmlRun::plain("x")
        };
        let xml = serialize_ooxml_run(&run);
        let highlight_pos = xml.find("<a:highlight>").unwrap();
        let latin_pos = xml.find("<a:latin").unwrap();
        let hlink_pos = xml.find("<a:hlinkClick").unwrap();
        assert!(
            highlight_pos < latin_pos,
            "highlight must precede latin in: {xml}"
        );
        assert!(
            latin_pos < hlink_pos,
            "latin must precede hlinkClick in: {xml}"
        );
    }

    /// Text XML-escaped: `&` → `&amp;`, `<` → `&lt;`, `>` → `&gt;`
    #[test]
    fn test_adr024_step1_serialize_text_xml_escaped() {
        let run = plain_run("a & b < c > d");
        let xml = serialize_ooxml_run(&run);
        assert!(xml.contains("a &amp; b &lt; c &gt; d"), "got: {xml}");
    }

    // ── Step 2: render_inline_nodes_to_runs unit tests ─────────────────────

    /// `Plain("hello")` → one `OoxmlRun { text: "hello", all flags off }`
    #[test]
    fn test_adr024_step2_plain() {
        let nodes = vec![inline_plain("hello")];
        let runs = render_inline_nodes_to_runs(&nodes, &no_resolver).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0], plain_run("hello"));
    }

    /// `Bold([Plain("hi")])` → `OoxmlRun { text: "hi", bold: true, .. }`
    #[test]
    fn test_adr024_step2_bold_plain() {
        let nodes = vec![inline_bold(vec![inline_plain("hi")])];
        let runs = render_inline_nodes_to_runs(&nodes, &no_resolver).unwrap();
        assert_eq!(runs.len(), 1);
        assert!(runs[0].bold, "expected bold=true; got: {:?}", runs[0]);
        assert_eq!(runs[0].text, "hi");
    }

    /// `Bold([Italic([Plain("x")])])` → `OoxmlRun { bold: true, italic: true, text: "x" }`
    #[test]
    fn test_adr024_step2_bold_italic_nested() {
        let nodes = vec![inline_bold(vec![inline_italic(vec![inline_plain("x")])])];
        let runs = render_inline_nodes_to_runs(&nodes, &no_resolver).unwrap();
        assert_eq!(runs.len(), 1);
        assert!(runs[0].bold, "expected bold; got: {:?}", runs[0]);
        assert!(runs[0].italic, "expected italic; got: {:?}", runs[0]);
        assert_eq!(runs[0].text, "x");
    }

    /// `Link` with resolver returning `Some("rId3")` → run has `hyperlink_rid = Some("rId3")`
    #[test]
    fn test_adr024_step2_link_with_resolver() {
        let nodes = vec![inline_link(
            vec![inline_plain("click")],
            "https://example.com",
        )];
        let runs = render_inline_nodes_to_runs(&nodes, &fixed_resolver("rId3")).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "click");
        assert_eq!(
            runs[0].hyperlink_rid,
            Some("rId3".to_owned()),
            "expected hyperlink_rid=Some(rId3); got: {:?}",
            runs[0]
        );
    }

    /// `Link` with resolver returning `None` → run has `hyperlink_rid = None`
    #[test]
    fn test_adr024_step2_link_no_resolver() {
        let nodes = vec![inline_link(
            vec![inline_plain("click")],
            "javascript:alert(1)",
        )];
        let runs = render_inline_nodes_to_runs(&nodes, &no_resolver).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "click");
        assert_eq!(
            runs[0].hyperlink_rid, None,
            "expected hyperlink_rid=None for unsafe URL; got: {:?}",
            runs[0]
        );
    }

    /// F-P16-M1 REGRESSION TEST: `Bold([Link{text:[Plain("here")]}])` with resolver
    /// returning `Some("rId3")` → `OoxmlRun { text: "here", bold: true, hyperlink_rid: Some("rId3") }`
    ///
    /// This is the core case that F-P16-M1 broke (notes silently dropped
    /// formatting inside link display text). The unified engine fixes it.
    #[test]
    fn test_adr024_step2_bold_wrapping_link_preserves_both_bold_and_hyperlink_rid() {
        let nodes = vec![inline_bold(vec![inline_link(
            vec![inline_plain("here")],
            "https://example.com",
        )])];
        let runs = render_inline_nodes_to_runs(&nodes, &fixed_resolver("rId3")).unwrap();
        assert_eq!(runs.len(), 1, "expected 1 run; got: {runs:?}");
        assert_eq!(runs[0].text, "here");
        assert!(
            runs[0].bold,
            "F-P16-M1: bold must be preserved on link run; got: {:?}",
            runs[0]
        );
        assert_eq!(
            runs[0].hyperlink_rid,
            Some("rId3".to_owned()),
            "F-P16-M1: hyperlink_rid must be Some(rId3); got: {:?}",
            runs[0]
        );
    }

    /// INV-5 + INV-6: `Link{url:U1, text:[Link{url:U2, text:[Plain("inner")]}]}` →
    /// inner link's URL is NOT passed to resolver; inner runs inherit outer rId.
    #[test]
    fn test_adr024_step2_nested_link_in_display_text_inherits_outer_rid() {
        let resolver_calls = std::cell::Cell::new(0u32);
        let resolver = |_url: &str| {
            resolver_calls.set(resolver_calls.get() + 1);
            Some("rId3".to_owned())
        };

        let inner_link = inline_link(vec![inline_plain("inner")], "https://inner.test");
        let outer_link = inline_link(vec![inner_link], "https://outer.test");
        let nodes = vec![outer_link];

        let runs = render_inline_nodes_to_runs(&nodes, &resolver).unwrap();

        // Resolver must be called ONCE (for the outer link only — INV-5).
        assert_eq!(
            resolver_calls.get(),
            1,
            "INV-5: resolver must be called once (outer link only); \
             got {} calls",
            resolver_calls.get()
        );

        // The inner run must inherit the outer rId (INV-6).
        assert_eq!(runs.len(), 1);
        assert_eq!(
            runs[0].hyperlink_rid,
            Some("rId3".to_owned()),
            "INV-6: inner run must inherit outer rId; got: {:?}",
            runs[0]
        );
        assert_eq!(runs[0].text, "inner");
    }

    /// Math node → `OoxmlRun { text: latex_source }` (EC-001 fallback, INV-8)
    #[test]
    fn test_adr024_step2_math_fallback() {
        let nodes = vec![inline_math("x^2")];
        let runs = render_inline_nodes_to_runs(&nodes, &no_resolver).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "x^2");
    }

    /// Nesting depth > 64 → `InlineError::RenderError`
    #[test]
    fn test_adr024_step2_depth_limit_exceeded() {
        // Build a node nested 65 levels deep (exceeds MAX_DEPTH=64).
        let mut node = inline_plain("leaf");
        for _ in 0..65 {
            node = inline_bold(vec![node]);
        }
        let nodes = vec![node];
        let result = render_inline_nodes_to_runs(&nodes, &no_resolver);
        assert!(
            result.is_err(),
            "expected RenderError at depth>64; got: {result:?}"
        );
    }

    /// All 8 inline forms produce at least one run with correct properties.
    #[test]
    fn test_adr024_step2_all_8_forms() {
        // Superscript
        let nodes = vec![inline_superscript(vec![inline_plain("sup")])];
        let runs = render_inline_nodes_to_runs(&nodes, &no_resolver).unwrap();
        assert_eq!(runs[0].baseline, Some(30_000), "superscript baseline");

        // Subscript
        let nodes = vec![inline_subscript(vec![inline_plain("sub")])];
        let runs = render_inline_nodes_to_runs(&nodes, &no_resolver).unwrap();
        assert_eq!(runs[0].baseline, Some(-25_000), "subscript baseline");

        // Strikethrough
        let nodes = vec![inline_strikethrough(vec![inline_plain("del")])];
        let runs = render_inline_nodes_to_runs(&nodes, &no_resolver).unwrap();
        assert!(runs[0].strike, "strikethrough strike flag");

        // Highlight
        let nodes = vec![inline_highlight(vec![inline_plain("hi")])];
        let runs = render_inline_nodes_to_runs(&nodes, &no_resolver).unwrap();
        assert!(runs[0].highlight, "highlight flag");

        // Code
        let nodes = vec![inline_code("fn x()")];
        let runs = render_inline_nodes_to_runs(&nodes, &no_resolver).unwrap();
        assert!(runs[0].code_font, "code_font flag");
    }

    /// Multi-leaf link display text: `[click **here** now](url)` →
    /// 3 runs all with `hyperlink_rid = Some("rId3")` (INV-2, Risk 2 per ADR-024).
    #[test]
    fn test_adr024_step2_multi_leaf_link_display_text() {
        let nodes = vec![inline_link(
            vec![
                inline_plain("click "),
                inline_bold(vec![inline_plain("here")]),
                inline_plain(" now"),
            ],
            "https://example.com",
        )];
        let runs = render_inline_nodes_to_runs(&nodes, &fixed_resolver("rId3")).unwrap();
        assert_eq!(runs.len(), 3, "expected 3 runs; got: {runs:?}");
        for (i, run) in runs.iter().enumerate() {
            assert_eq!(
                run.hyperlink_rid,
                Some("rId3".to_owned()),
                "run[{i}] must have hyperlink_rid=Some(rId3); got: {run:?}"
            );
        }
        // Bold must be preserved on the middle run.
        let mid = &runs[1];
        assert!(mid.bold, "middle run must be bold; got: {mid:?}");
    }

    /// Reference-set invariant (INV-4 per ADR-024): for multi-leaf link display text,
    /// count-equality (`rel_count == hlinkclick_count`) is FALSE (1 rel, 3 clicks),
    /// but the reference-set invariant holds (1 rel, all 3 clicks reference it).
    #[test]
    fn test_adr024_step2_multi_leaf_link_reference_set_invariant() {
        let nodes = vec![inline_link(
            vec![inline_plain("a"), inline_plain("b"), inline_plain("c")],
            "https://example.com",
        )];
        let runs = render_inline_nodes_to_runs(&nodes, &fixed_resolver("rId3")).unwrap();
        // All 3 runs reference rId3.
        let rids: std::collections::HashSet<&str> = runs
            .iter()
            .filter_map(|r| r.hyperlink_rid.as_deref())
            .collect();
        assert_eq!(
            rids.len(),
            1,
            "all runs must reference the same rId; got: {rids:?}"
        );
        assert!(rids.contains("rId3"), "all runs reference rId3");
        assert_eq!(runs.len(), 3, "3 leaf runs; got: {runs:?}");
    }

    /// Empty display text link → zero runs produced.
    #[test]
    fn test_adr024_step2_empty_display_text_link_produces_no_runs() {
        let nodes = vec![inline_link(vec![], "https://example.com")];
        let runs = render_inline_nodes_to_runs(&nodes, &fixed_resolver("rId3")).unwrap();
        assert_eq!(
            runs.len(),
            0,
            "empty display text must produce 0 runs; got: {runs:?}"
        );
    }

    /// `serialize_ooxml_run` round-trip: render then serialize produces correct XML.
    #[test]
    fn test_adr024_step2_bold_link_serialize_produces_correct_xml() {
        let nodes = vec![inline_bold(vec![inline_link(
            vec![inline_plain("here")],
            "https://example.com",
        )])];
        let runs = render_inline_nodes_to_runs(&nodes, &fixed_resolver("rId3")).unwrap();
        assert_eq!(runs.len(), 1);
        let xml = serialize_ooxml_run(&runs[0]);
        // Must have b="1" AND hlinkClick.
        assert!(xml.contains("b=\"1\""), "got: {xml}");
        assert!(xml.contains("<a:hlinkClick r:id=\"rId3\"/>"), "got: {xml}");
        assert!(xml.contains("<a:t>here</a:t>"), "got: {xml}");
    }
}
