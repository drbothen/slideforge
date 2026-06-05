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
//! becomes one or more `<a:p>` paragraphs. Inline formatting is preserved via
//! dispatch through the [`InlineFormat`] plugin (BC-5.02.002 dog-fooding guarantee):
//! - `InlineNode::Bold` → `<a:rPr b="1"/>` (via [`InlineFormat::render_with_context`])
//! - `InlineNode::Italic` → `<a:rPr i="1"/>` (via [`InlineFormat::render_with_context`])
//! - `InlineNode::Link` → hyperlink relationship + `<a:rPr>` with `r:id`
//!   (rId pre-registered by exporter, passed via `InlineRenderContext::hyperlink_rid`)
//! - All other nodes → via [`InlineFormat::render_with_context`] at the single
//!   `AC-005-DISPATCH-SITE`. Registry routing: the formatter is resolved by the caller
//!   (F-006). Currently `DefaultInlineFormat` is passed directly; future: registry lookup.

use slideforge_plugin_api::traits::inline_format::{
    InlineFormat, InlineOutputFormat, InlineRenderContext,
};
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
    /// `inline_format` is the registry-resolved `InlineFormat` plugin (F-006).
    /// The caller (typically `PptxExporter::export_inner`) resolves this from
    /// the `PluginRegistry` by looking up id `"default"`. All OOXML run emission
    /// is dispatched through this reference — zero hardcoding inside the serializer.
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
        inline_format: &dyn InlineFormat,
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

        let xml_bytes = Self::build_xml(&notes_entries, &unique_hlinks, inline_format);
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
        notes_entries: &[&[InlineNode]],
        hlink_urls: &[String],
        inline_format: &dyn InlineFormat,
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
        // Schema-valid grpSpPr: CT_GroupShapeProperties allows only <a:xfrm> (and
        // a few others) — there is NO child element named <a:grpSpPr> in that type.
        // Emit a single <a:xfrm> with all four required children (off/ext/chOff/chExt).
        // This matches the canonical form in slideforge-brand::write_grpsppr.
        xml.push_str("      <p:grpSpPr>\n");
        xml.push_str("        <a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/>");
        xml.push_str("<a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm>\n");
        xml.push_str("      </p:grpSpPr>\n");

        // Slide image placeholder — ph is self-closing inside nvPr (no txBody).
        xml.push_str("      <p:sp>\n");
        xml.push_str("        <p:nvSpPr>\n");
        // cNvPr @name is free-text / non-schema-significant.  Use a fixed role
        // name ("Slide Image Placeholder 1") that matches the notesMaster template
        // rather than varying it by slide index — a slide-index-derived name would
        // differ across slides without adding semantic value and would cause rId
        // count mismatches to appear larger than they are in diff output.
        // (F-040-P3-001 tidy — confirmed cosmetic-only, no schema impact.)
        xml.push_str("          <p:cNvPr id=\"2\" name=\"Slide Image Placeholder 1\"/>\n");
        xml.push_str("          <p:cNvSpPr><a:spLocks noGrp=\"1\"/></p:cNvSpPr>\n");
        xml.push_str("          <p:nvPr><p:ph type=\"sldImg\"/></p:nvPr>\n");
        xml.push_str("        </p:nvSpPr>\n");
        xml.push_str("        <p:spPr/>\n");
        xml.push_str("      </p:sp>\n");

        // Notes body placeholder — ph is self-closing inside nvPr;
        // txBody is a SIBLING of nvSpPr under p:sp (CT_Shape ordering:
        // nvSpPr → spPr → txBody).
        // cNvPr @name is fixed ("Notes Placeholder 2") matching the notesMaster
        // template — slide-index variation is non-schema-significant and confusing.
        // (F-040-P3-001 tidy — cosmetic, no schema impact.)
        xml.push_str("      <p:sp>\n");
        xml.push_str("        <p:nvSpPr>\n");
        xml.push_str("          <p:cNvPr id=\"3\" name=\"Notes Placeholder 2\"/>\n");
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
                // Dispatch through DefaultInlineFormat for all non-Link nodes
                // (BC-5.02.002 dog-fooding guarantee).
                dispatch_inline_nodes_to_ooxml(entry, hlink_urls, inline_format, &mut xml);
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

/// Dispatch a flat sequence of `InlineNode`s to OOXML run strings.
///
/// All nodes — including [`InlineNode::Link`] — are dispatched through
/// [`DefaultInlineFormat::render_with_context`] (BC-5.02.002 dog-fooding /
/// AC-005). For `Link` nodes with a safe URL scheme, the pre-registered
/// relationship ID is passed via [`InlineRenderContext::hyperlink_rid`] so
/// `DefaultInlineFormat` can emit the full OOXML hyperlink run. For unsafe-scheme
/// URLs, `hyperlink_rid` is `None` and the formatter falls back to the
/// display-text-plus-warn behavior.
///
/// This is the SINGLE dispatch call site in `slideforge-pptx/src/` for inline
/// OOXML construction (AC-005 / BC-5.02.002 postcondition 5). The call site
/// is marked with `// AC-005-DISPATCH-SITE` so the AC-005 audit test can
/// exempt it while flagging any other hand-constructed run markup.
fn dispatch_inline_nodes_to_ooxml(
    nodes: &[InlineNode],
    hlink_urls: &[String],
    inline_format: &dyn InlineFormat,
    out: &mut String,
) {
    let formatter = inline_format;
    for node in nodes {
        // Build the render context for this node.
        // For Link nodes with a safe scheme: supply the pre-registered rId.
        // For all other nodes (and unsafe-scheme Links): supply no rId.
        let hyperlink_rid: Option<String> = if let InlineNode::Link { url, .. } = node {
            if is_safe_link_scheme(url.as_ref()) {
                // F-040-P2-001 (CWE-601): only safe-scheme URLs are in hlink_urls.
                // rId3 = index 0, rId4 = index 1, ...
                hlink_urls
                    .iter()
                    .position(|u| u == url.as_ref())
                    .map(|idx| format!("rId{}", idx + 3))
            } else {
                // Unsafe scheme: warn and let render_with_context fall back to plain text.
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
                None
            }
        } else {
            None
        };

        let ctx = InlineRenderContext {
            hyperlink_rid: hyperlink_rid.as_deref(),
        };

        // AC-005-DISPATCH-SITE: single InlineFormat dispatch call in slideforge-pptx/src/.
        match formatter.render_with_context(node, InlineOutputFormat::Ooxml, &ctx) {
            Ok(rendered) => out.push_str(&rendered),
            Err(e) => {
                tracing::warn!(
                    node_kind = node.kind_name(),
                    error = %e,
                    "DefaultInlineFormat::render_with_context failed for notes OOXML; skipping node"
                );
            },
        }
    }
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
///
/// ## F-040-P3-001: No recursion into a `Link`'s display text
///
/// The serializer (`serialize_nodes_with_context`) flattens a `Link`'s display
/// `text` children to plain text via `extract_plain_text` — it does NOT recurse
/// through them with `serialize_nodes_with_context`.  Therefore any nested `Link`
/// inside display text is never serialized as an `<a:hlinkClick>`.
///
/// Consequence: we must NOT descend into `text` here either.  Collecting a nested
/// URL would assign it an rId in the `.rels` file with no corresponding
/// `<a:hlinkClick>` referencing it — an orphan External relationship that OOXML
/// linters flag (rId count ≠ hlinkClick count).
///
/// Fix (option b — least change): collect ONLY the outer `Link`'s URL (when its
/// scheme is safe).  Nested `Link` nodes inside display text are ignored; they
/// will be rendered as plain text by the serializer (consistent behavior).
fn collect_hyperlink_urls(nodes: &[InlineNode], urls: &mut Vec<String>) {
    for node in nodes {
        match node {
            InlineNode::Link { url, text: _ } => {
                // F-040-P2-001 defense-in-depth: only register URLs with safe schemes.
                // Unsafe-scheme URLs are NOT added to the hlink_urls list; the Link
                // arm in serialize_nodes_with_context will emit them as plain text runs.
                if is_safe_link_scheme(url.as_ref()) {
                    urls.push(url.as_ref().to_owned());
                }
                // F-040-P3-001: Do NOT recurse into `text` children here.
                // The serializer flattens display text via extract_plain_text, so any
                // nested Link inside display text produces NO hlinkClick.  Collecting
                // its URL here would create an orphan External rel (rId with no
                // referencing hlinkClick).  Nested URLs in display text are intentionally
                // rendered as plain text — consistent with the serializer's behavior.
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
