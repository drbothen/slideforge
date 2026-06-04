//! `NotesSlideSerializer` — produces `ppt/notesSlides/notesSlide{N}.xml` and
//! `ppt/notesSlides/_rels/notesSlide{N}.xml.rels` for each slide that has
//! non-empty speaker notes content.
//!
//! ## Contract (BC-4.01.003 / STORY-040)
//!
//! - One `notesSlide{N}.xml` per slide where `register_content` contains a
//!   non-empty [`slideforge_types::Register::Notes`] entry.
//! - The notes text is placed in the `<p:txBody>` of the body placeholder
//!   (`<p:ph type="body" idx="1">`).
//! - A companion `.rels` file is produced that references both the owning slide
//!   and `notesMaster1.xml`.
//! - Slides WITHOUT notes get no `notesSlide` part (BC-4.01.003 postcondition 2).
//!
//! ## Rich Inline Formatting (F-040-P1-003)
//!
//! Notes content is sourced from ALL `Register::Notes` entries in
//! `register_content` (not just the first). Each `RegisteredContent` entry
//! becomes one or more `<a:p>` paragraphs. Inline formatting is preserved:
//! - `InlineNode::Bold` → `<a:rPr b="1"/>`
//! - `InlineNode::Italic` → `<a:rPr i="1"/>`
//! - `InlineNode::Link` → hyperlink relationship + `<a:rPr>` with `r:id`
//! - All other nodes (Code, Xref, Footnote, etc.) → plain text run

use std::fmt::Write as _;

use slideforge_types::register::RegisteredContent;
use slideforge_types::{InlineNode, Register};

use crate::error::PptxError;
use crate::link_safety::is_safe_link_scheme;
use crate::rels::{RelsBuilder, rel_types};

/// Relationship type for a notesSlide → its owning slide.
///
/// See OOXML spec §12.3.10 — notesSlide part relationship.
const NOTES_SLIDE_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide";

/// Output produced by [`NotesSlideSerializer::build`] for a single slide.
pub struct NotesSlideOutput {
    /// Raw bytes for `ppt/notesSlides/notesSlide{N}.xml`.
    pub xml_bytes: Vec<u8>,
    /// Raw bytes for `ppt/notesSlides/_rels/notesSlide{N}.xml.rels`.
    pub rels_bytes: Vec<u8>,
}

/// Serializes `ppt/notesSlides/notesSlide{N}.xml` for a single slide.
///
/// # Contract
///
/// - Only called when the slide has non-empty `Register::Notes` content.
/// - `slide_index` is 1-based (matches the ZIP file naming convention:
///   `notesSlide1.xml`, `notesSlide2.xml`, …).
/// - `register_content` provides ALL `RegisteredContent` entries for this slide;
///   the serializer filters to `Register::Notes` entries internally, covering
///   ALL Notes entries (not just the first — F-040-P1-003).
pub struct NotesSlideSerializer;

impl NotesSlideSerializer {
    /// Build `notesSlide{N}.xml` and its companion `.rels` file.
    ///
    /// Sources ALL `Register::Notes` entries from `register_content`
    /// (F-040-P1-003: multi-entry, rich-formatting fix). Each entry is
    /// emitted as one `<a:p>` paragraph with inline formatting preserved.
    ///
    /// The `.rels` file references:
    /// - `rId1` → the owning slide (`../slides/slide{N}.xml`)
    /// - `rId2` → the notes master (`../notesMasters/notesMaster1.xml`)
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::OoxmlElement`] if XML or rels serialization fails.
    pub fn build(
        slide_index: usize,
        register_content: &[RegisteredContent],
    ) -> Result<NotesSlideOutput, PptxError> {
        // Filter to Notes-register entries; collect inline content per entry.
        let notes_entries: Vec<&[InlineNode]> = register_content
            .iter()
            .filter(|rc| rc.register == Register::Notes)
            .map(|rc| rc.content.as_slice())
            .collect();

        // Collect hyperlink targets for relationship registration.
        // Each hyperlink URL gets a unique rId starting from rId3
        // (rId1 = slide, rId2 = notesMaster).
        let mut hlink_urls: Vec<String> = Vec::new();
        for entry in &notes_entries {
            collect_hyperlink_urls(entry, &mut hlink_urls);
        }
        // Deduplicate while preserving order (URL order → rId order).
        let unique_hlinks: Vec<String> = deduplicate_preserve_order(hlink_urls);

        let xml_bytes = Self::build_xml(slide_index, &notes_entries, &unique_hlinks);
        let rels_bytes = Self::build_rels(slide_index, &unique_hlinks)?;
        Ok(NotesSlideOutput {
            xml_bytes,
            rels_bytes,
        })
    }

    /// Generate the `notesSlide{N}.xml` bytes.
    ///
    /// Uses string-based XML construction with XML-escaping for all user text.
    /// This is the bounded exception for `ooxmlsdk` usage: `ooxmlsdk 0.6.1`
    /// does not expose typed builders for `p:notes` (notesSlide root element) —
    /// same precedent as `docProps/core.xml` in `build_doc_props` (ADR-001).
    ///
    /// ## OOXML structure
    ///
    /// Per the OOXML schema (`CT_Shape` / `CT_Placeholder`), `<p:ph>` is a
    /// non-textual placeholder descriptor.  It is **not** a container and
    /// **cannot** hold `<p:txBody>` as a child.  The correct structure is:
    ///
    /// ```xml
    /// <p:sp>
    ///   <p:nvSpPr>
    ///     <p:cNvPr id="3" name="Notes Placeholder 2"/>
    ///     <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
    ///     <p:nvPr><p:ph type="body" idx="1"/></p:nvPr>  <!-- self-closing -->
    ///   </p:nvSpPr>
    ///   <p:spPr/>
    ///   <p:txBody>                <!-- sibling of nvSpPr, direct child of p:sp -->
    ///     <a:bodyPr/>
    ///     <a:lstStyle/>
    ///     <a:p>...</a:p>          <!-- one per Register::Notes entry -->
    ///   </p:txBody>
    /// </p:sp>
    /// ```
    fn build_xml(
        slide_index: usize,
        notes_entries: &[&[InlineNode]],
        hlink_urls: &[String],
    ) -> Vec<u8> {
        let mut xml = String::new();

        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        xml.push_str("<p:notes");
        xml.push_str(" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\"");
        xml.push_str(" xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\"");
        xml.push_str(
            " xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"",
        );
        xml.push_str(">\n");
        xml.push_str("  <p:cSld>\n");
        xml.push_str("    <p:spTree>\n");
        // grpSpPr with identity transform (required by schema)
        xml.push_str("      <p:grpSpPr>\n");
        xml.push_str(
            "        <a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/></a:xfrm>\n",
        );
        xml.push_str(
            "        <a:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/>",
        );
        xml.push_str(
            "<a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></a:grpSpPr>\n",
        );
        xml.push_str("      </p:grpSpPr>\n");

        // Slide image placeholder — ph is self-closing inside nvPr (no txBody).
        xml.push_str("      <p:sp>\n");
        xml.push_str("        <p:nvSpPr>\n");
        // `writeln!` to `String` is infallible (fmt::Write for String never returns Err).
        let _ = writeln!(
            xml,
            "          <p:cNvPr id=\"2\" name=\"Slide Image Placeholder {slide_index}\"/>"
        );
        xml.push_str("          <p:cNvSpPr><a:spLocks noGrp=\"1\"/></p:cNvSpPr>\n");
        xml.push_str("          <p:nvPr><p:ph type=\"sldImg\"/></p:nvPr>\n");
        xml.push_str("        </p:nvSpPr>\n");
        xml.push_str("        <p:spPr/>\n");
        xml.push_str("      </p:sp>\n");

        // Notes body placeholder — ph is self-closing inside nvPr;
        // txBody is a SIBLING of nvSpPr under p:sp (CT_Shape ordering:
        // nvSpPr → spPr → txBody).
        let body_ph_idx = slide_index + 1;
        xml.push_str("      <p:sp>\n");
        xml.push_str("        <p:nvSpPr>\n");
        // `writeln!` to `String` is infallible (fmt::Write for String never returns Err).
        let _ = writeln!(
            xml,
            "          <p:cNvPr id=\"3\" name=\"Notes Placeholder {body_ph_idx}\"/>"
        );
        xml.push_str("          <p:cNvSpPr><a:spLocks noGrp=\"1\"/></p:cNvSpPr>\n");
        xml.push_str("          <p:nvPr><p:ph type=\"body\" idx=\"1\"/></p:nvPr>\n");
        xml.push_str("        </p:nvSpPr>\n");
        xml.push_str("        <p:spPr/>\n");
        xml.push_str("        <p:txBody>\n");
        xml.push_str("          <a:bodyPr/>\n");
        xml.push_str("          <a:lstStyle/>\n");

        if notes_entries.is_empty() {
            // Ensure at least one paragraph so txBody is schema-valid.
            xml.push_str("          <a:p/>\n");
        } else {
            for entry in notes_entries {
                xml.push_str("          <a:p>");
                // Assign rIds to hyperlinks: rId3, rId4, ...
                // We need to find which rId each URL maps to.
                serialize_inline_nodes_to_xml(entry, hlink_urls, &mut xml);
                xml.push_str("</a:p>\n");
            }
        }

        xml.push_str("        </p:txBody>\n");
        xml.push_str("      </p:sp>\n");
        xml.push_str("    </p:spTree>\n");
        xml.push_str("  </p:cSld>\n");
        xml.push_str("</p:notes>");

        xml.into_bytes()
    }

    /// Generate the `.rels` bytes for a single notesSlide part.
    ///
    /// Relationships:
    /// - `rId1` → the owning slide (`../slides/slide{N}.xml`)
    /// - `rId2` → the notes master (`../notesMasters/notesMaster1.xml`)
    /// - `rId3..` → external hyperlinks (one per unique URL in notes content)
    fn build_rels(slide_index: usize, hlink_urls: &[String]) -> Result<Vec<u8>, PptxError> {
        let mut rels = RelsBuilder::new();
        // rId1 → owning slide
        rels.add(
            NOTES_SLIDE_REL_TYPE,
            format!("../slides/slide{slide_index}.xml"),
        );
        // rId2 → notesMaster
        rels.add(rel_types::NOTES_MASTER, "../notesMasters/notesMaster1.xml");
        // rId3.. → external hyperlinks with TargetMode="External"
        for url in hlink_urls {
            rels.add_external_hyperlink(url.as_str());
        }
        rels.build().map_err(|e| PptxError::OoxmlElement {
            part: format!("ppt/notesSlides/_rels/notesSlide{slide_index}.xml.rels"),
            detail: format!("RelsBuilder::build failed: {e}"),
        })
    }
}

/// Serialize a flat sequence of `InlineNode`s to `<a:r>` XML run strings.
///
/// This function handles the mapping from slideforge's rich inline tree to
/// OOXML's flat run model. The context parameters `bold` and `italic` track
/// inherited formatting from ancestor nodes.
///
/// Hyperlink support: for `Link` nodes, the URL is looked up in `hlink_urls`
/// to find the corresponding `rId` (rId3 for index 0, rId4 for index 1, etc.).
fn serialize_inline_nodes_to_xml(nodes: &[InlineNode], hlink_urls: &[String], out: &mut String) {
    serialize_nodes_with_context(nodes, false, false, hlink_urls, out);
}

/// Recursive helper for inline node serialization.
///
/// `bold` and `italic` are inherited formatting flags from ancestor Bold/Italic nodes.
fn serialize_nodes_with_context(
    nodes: &[InlineNode],
    bold: bool,
    italic: bool,
    hlink_urls: &[String],
    out: &mut String,
) {
    for node in nodes {
        match node {
            InlineNode::Plain(text) => {
                emit_run(text, bold, italic, None, out);
            },
            InlineNode::Code(text) => {
                // Code runs: emit as plain text (no separate code formatting in notes)
                emit_run(text, bold, italic, None, out);
            },
            InlineNode::Xref(text) => {
                // Cross-reference: emit as plain text
                emit_run(text, bold, italic, None, out);
            },
            InlineNode::Bold(children) => {
                serialize_nodes_with_context(children, true, italic, hlink_urls, out);
            },
            InlineNode::Italic(children) => {
                serialize_nodes_with_context(children, bold, true, hlink_urls, out);
            },
            InlineNode::Footnote(children)
            | InlineNode::Superscript(children)
            | InlineNode::Subscript(children)
            | InlineNode::Strikethrough(children)
            | InlineNode::Highlight(children) => {
                // These formatting types are not distinctly representable in notes
                // run properties at this time; emit as plain text with inherited context.
                serialize_nodes_with_context(children, bold, italic, hlink_urls, out);
            },
            InlineNode::Link { text, url } => {
                // F-040-P2-001 (CWE-601) defense-in-depth: check scheme before embedding.
                // Safe URLs are in hlink_urls (collected by collect_hyperlink_urls) and
                // get a clickable hlinkClick. Unsafe-scheme URLs are not in hlink_urls
                // (filtered at collection time) — they degrade gracefully to a plain text
                // run with a tracing::warn! so the export does not silently embed a
                // dangerous TargetMode="External" rel.
                if is_safe_link_scheme(url.as_ref()) {
                    // Find the rId for this URL (rId3 = index 0, rId4 = index 1, …)
                    let rid_index = hlink_urls.iter().position(|u| u == url.as_ref());
                    let hlink_rid = rid_index.map(|idx| format!("rId{}", idx + 3));
                    // Emit the link as a hyperlink run wrapping the display text.
                    // OOXML hyperlink in a notesSlide uses <a:hlinkClick r:id="rIdN"/>
                    // on the run's <a:rPr>. We emit the display text with the link rId.
                    if let Some(rid) = &hlink_rid {
                        // Wrap in <a:r> with <a:rPr> carrying the hyperlink reference.
                        out.push_str("<a:r><a:rPr");
                        if bold {
                            out.push_str(" b=\"1\"");
                        }
                        if italic {
                            out.push_str(" i=\"1\"");
                        }
                        out.push_str("><a:hlinkClick r:id=\"");
                        out.push_str(&xml_escape(rid));
                        out.push_str("\"/></a:rPr><a:t>");
                        // Emit display text (recursively extracting plain text).
                        let display_text = extract_plain_text(text);
                        out.push_str(&xml_escape(&display_text));
                        out.push_str("</a:t></a:r>");
                    } else {
                        // No rId found (shouldn't happen for a safe URL if hlink_urls is complete);
                        // emit as plain text run.
                        let display_text = extract_plain_text(text);
                        emit_run(&display_text, bold, italic, None, out);
                    }
                } else {
                    // Unsafe scheme: extract for the warning then degrade to plain text.
                    // Emit display text as a plain run; do NOT emit hlinkClick or External rel.
                    let scheme_end = url.find(':').unwrap_or(0);
                    let scheme = if scheme_end > 0 {
                        &url[..scheme_end]
                    } else {
                        "(none)"
                    };
                    tracing::warn!(
                        url_scheme = scheme,
                        "notes link has disallowed URL scheme; \
                         embedding as plain text run (no External rel emitted). \
                         SEC-037-001 / CWE-601 / F-040-P2-001"
                    );
                    let display_text = extract_plain_text(text);
                    emit_run(&display_text, bold, italic, None, out);
                }
            },
            InlineNode::Math(math_node) => {
                // Math nodes: emit LaTeX source as plain text (math rendering
                // for notes is a Phase 6 concern; notes are speaker-guidance text).
                emit_run(math_node.latex.as_ref(), bold, italic, None, out);
            },
        }
    }
}

/// Emit a single `<a:r>` run with optional bold/italic run properties.
///
/// If `text` is empty, no run element is emitted (empty runs are not useful).
/// The `r:id` parameter is reserved for future use (e.g., hyperlink embedding
/// outside the Link node path).
fn emit_run(text: &str, bold: bool, italic: bool, _rid: Option<&str>, out: &mut String) {
    if text.is_empty() {
        return;
    }
    let needs_rpr = bold || italic;
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
}

/// Collect all **safe-scheme** hyperlink URLs from an inline node tree, depth-first.
///
/// URLs whose scheme is not in [`crate::link_safety::ALLOWED_LINK_SCHEMES`] are
/// silently skipped here (they will also be handled gracefully in
/// [`serialize_nodes_with_context`] — the `Link` arm falls through to a plain text
/// run when no `rId` is found).  A `tracing::warn!` is emitted for each rejected
/// URL so the caller has an audit trail.
///
/// This is the defense-in-depth guard at the PPTX exporter boundary
/// (F-040-P2-001 / CWE-601).  The parser already enforces `E-PAR-022`; this guard
/// catches programmatically-constructed IR that bypasses the parser.
fn collect_hyperlink_urls(nodes: &[InlineNode], urls: &mut Vec<String>) {
    for node in nodes {
        match node {
            InlineNode::Link { url, text } => {
                // F-040-P2-001 defense-in-depth: only register URLs with safe schemes.
                // Unsafe-scheme URLs are NOT added to the hlink_urls list; the Link
                // arm in serialize_nodes_with_context will emit them as plain text runs.
                if is_safe_link_scheme(url.as_ref()) {
                    urls.push(url.as_ref().to_owned());
                }
                // Always recurse into the link's display text (may contain nested inlines).
                collect_hyperlink_urls(text, urls);
            },
            InlineNode::Bold(c)
            | InlineNode::Italic(c)
            | InlineNode::Footnote(c)
            | InlineNode::Superscript(c)
            | InlineNode::Subscript(c)
            | InlineNode::Strikethrough(c)
            | InlineNode::Highlight(c) => {
                collect_hyperlink_urls(c, urls);
            },
            InlineNode::Plain(_)
            | InlineNode::Code(_)
            | InlineNode::Xref(_)
            | InlineNode::Math(_) => {},
        }
    }
}

/// Extract all plain text from an inline node tree, depth-first.
///
/// Used for link display text extraction.
fn extract_plain_text(nodes: &[InlineNode]) -> String {
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
                out.push_str(&extract_plain_text(c));
            },
            InlineNode::Link { text, .. } => {
                out.push_str(&extract_plain_text(text));
            },
            InlineNode::Math(m) => {
                out.push_str(m.latex.as_ref());
            },
        }
    }
    out
}

/// Deduplicate a `Vec<String>` while preserving the first-occurrence order.
fn deduplicate_preserve_order(items: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    result
}

/// XML-escape a string for safe embedding in XML element text content.
///
/// Replaces the 5 XML-reserved characters:
/// - `&` → `&amp;` (must be first to avoid double-escaping)
/// - `<` → `&lt;`
/// - `>` → `&gt;`
/// - `"` → `&quot;`
/// - `'` → `&apos;`
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
