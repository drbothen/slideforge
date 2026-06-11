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
//! ## Rich Inline Formatting (ADR-024 / F-P16-M1 fix)
//!
//! Notes content is sourced from ALL `Register::Notes` entries in
//! `register_content` (not just the first). Each `RegisteredContent` entry
//! becomes one or more `<a:p>` paragraphs. Inline formatting is preserved via
//! the unified [`render_inline_nodes_to_runs`] engine (ADR-024):
//!
//! - `InlineNode::Bold` → `<a:rPr b="1"/>` run
//! - `InlineNode::Italic` → `<a:rPr i="1"/>` run
//! - `InlineNode::Link` → hyperlink relationship + `<a:rPr>` with `r:id`
//! - `InlineNode::Bold([Link{...}])` → run with BOTH `b="1"` AND `<a:hlinkClick>`
//!   (F-P16-M1 fix: formatting inside link display text is now preserved)
//! - All other nodes → via [`render_inline_nodes_to_runs`] single engine
//!
//! ## Architecture (ADR-024)
//!
//! The notes path NO LONGER uses `dispatch_inline_nodes_to_ooxml` or
//! `DefaultInlineFormat::render_with_context`. Both body and notes paths now
//! call `render_inline_nodes_to_runs` with a per-consumer resolver closure.
//! The resolver closure carries the `notesSlide{N}.xml.rels`-scoped rId namespace;
//! no cross-part leakage is possible by construction (ADR-024 INV-10).

use slideforge_plugin_api::inline_formats::{render_inline_nodes_to_runs, serialize_ooxml_run};
use slideforge_types::register::RegisteredContent;
use slideforge_types::{InlineNode, Register, display_text_is_empty};

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
    /// Inline formatting is rendered by the unified [`render_inline_nodes_to_runs`]
    /// engine (ADR-024). The notes-slide resolver closure maps URLs to
    /// `notesSlide{N}.xml.rels`-scoped rIds (rId3, rId4, …) computed from
    /// `unique_hlinks` (ADR-024 INV-10 — no cross-part leakage).
    ///
    /// The `.rels` file references:
    /// - `rId1` → the owning slide (`../slides/slide{N}.xml`)
    /// - `rId2` → the notes master (`../notesMasters/notesMaster1.xml`)
    /// - `rId3..` → external hyperlinks (one per unique safe-scheme URL)
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
        //
        // ADR-024 Risk 4 (strongest anti-orphan): `collect_hyperlink_urls` visits
        // the SAME URL set that `render_inline_nodes_to_runs` will query via the
        // resolver closure — top-level Links + Links inside formatting wrappers,
        // but NOT Links inside another Link's display text (INV-5 / F-040-P3-001).
        // The collector and the engine share the same traversal rule, so registration
        // and emission are symmetric by construction: no orphan rel, no dangling rId.
        let mut hlink_urls: Vec<String> = Vec::new();
        for entry in &notes_entries {
            collect_hyperlink_urls(entry, &mut hlink_urls);
        }
        // Deduplicate while preserving order (URL order → rId order).
        let unique_hlinks: Vec<String> = deduplicate_preserve_order(hlink_urls);

        let xml_bytes = Self::build_xml(&notes_entries, &unique_hlinks);
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
    fn build_xml(notes_entries: &[&[InlineNode]], hlink_urls: &[String]) -> Vec<u8> {
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
        // STORY-094 T-006 / BC-4.01.001 AC-004 — CT_GroupShape mandatory first child.
        //
        // REND-003 fix: `<p:nvGrpSpPr>` must be the FIRST child of `<p:spTree>`.
        // This satisfies ECMA-376 CT_GroupShape schema ordering for notesSlideN.xml.
        // The structure mirrors master/layout serializers (STORY-094 spec §AC-004).
        xml.push_str("      <p:nvGrpSpPr>");
        xml.push_str("<p:cNvPr id=\"1\" name=\"\"/>");
        xml.push_str("<p:cNvGrpSpPr/>");
        xml.push_str("<p:nvPr/>");
        xml.push_str("</p:nvGrpSpPr>\n");
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
        xml.push_str("          <p:cNvPr id=\"2\" name=\"Slide Image Placeholder 1\"/>\n");
        xml.push_str("          <p:cNvSpPr><a:spLocks noGrp=\"1\"/></p:cNvSpPr>\n");
        xml.push_str("          <p:nvPr><p:ph type=\"sldImg\"/></p:nvPr>\n");
        xml.push_str("        </p:nvSpPr>\n");
        xml.push_str("        <p:spPr/>\n");
        xml.push_str("      </p:sp>\n");

        // Notes body placeholder — ph is self-closing inside nvPr;
        // txBody is a SIBLING of nvSpPr under p:sp (CT_Shape ordering:
        // nvSpPr → spPr → txBody).
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
            // Build the notes-slide resolver closure (ADR-024 INV-10).
            // Maps a URL to its rId in notesSlide{N}.xml.rels namespace:
            //   rId3 = index 0, rId4 = index 1, ...
            // Only URLs that were registered in unique_hlinks get an rId;
            // all others (unsafe-scheme, empty-display-text) return None.
            let resolver = |url: &str| -> Option<String> {
                hlink_urls
                    .iter()
                    .position(|u| u == url)
                    .map(|idx| format!("rId{}", idx + 3))
            };

            for entry in notes_entries {
                xml.push_str("          <a:p>");

                // ADR-024: use the unified engine for inline run generation.
                // This is the single call site that replaces dispatch_inline_nodes_to_ooxml.
                match render_inline_nodes_to_runs(entry, &resolver) {
                    Ok(runs) => {
                        for run in &runs {
                            xml.push_str(&serialize_ooxml_run(run));
                        }
                    },
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "render_inline_nodes_to_runs failed for notes OOXML; \
                             emitting empty paragraph"
                        );
                    },
                }

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

/// Collect all **safe-scheme** hyperlink URLs from inline nodes, descending through
/// formatting wrappers to reach nested `Link` nodes (ADV-P14-MED-001 fix).
///
/// ## Registration scope (ADR-024 Risk 4 — anti-orphan by construction)
///
/// This function visits the SAME URL set that [`render_inline_nodes_to_runs`] will
/// query via its resolver closure:
/// - Top-level `Link` nodes (existing EC-004 path).
/// - `Link` nodes nested inside formatting wrappers (`Bold`, `Italic`, etc.).
/// - NOT Links inside another Link's display text (INV-5 / F-040-P3-001).
///
/// By keeping the collector and the engine's traversal rule identical, registration
/// and emission are symmetric by construction: every registered URL gets at least one
/// `<a:hlinkClick>`, and every `<a:hlinkClick>` references a registered URL.
///
/// ## Orphan-rel invariant preserved (F-085-P6-001)
///
/// When a `Link` is found, its own display `text` children are NOT recursed into.
/// A Link inside another Link's display text would create an orphan rel (the
/// engine only emits `<a:hlinkClick>` inherited from the outer rId, not a second one).
/// The F-040-P3-001 case (outer Link + nested Link in display text) still produces
/// exactly 1 rel and N `<a:hlinkClick>` runs (all backed by the same rel) — reference-set
/// invariant holds.
///
/// ## Unsafe-scheme filtering (F-040-P2-001 / CWE-601)
///
/// URLs whose scheme is not in [`crate::link_safety::ALLOWED_LINK_SCHEMES`] are
/// skipped here and produce a `tracing::warn!` in the engine.
///
/// ## Display-text emptiness guard (F-P5-001)
///
/// A `Link` whose display text is empty produces no run and no rel.
fn collect_hyperlink_urls(nodes: &[InlineNode], urls: &mut Vec<String>) {
    for node in nodes {
        collect_hyperlink_urls_from_node(node, urls);
    }
}

/// Collect safe-scheme, non-empty-display-text hyperlink URLs from a single
/// [`InlineNode`], descending through formatting wrappers to reach nested `Link` nodes.
fn collect_hyperlink_urls_from_node(node: &InlineNode, urls: &mut Vec<String>) {
    match node {
        InlineNode::Link { url, text } => {
            // F-040-P2-001 defense-in-depth: only register URLs with safe schemes.
            // F-P5-001: guard that the Link's display text is non-empty.
            if is_safe_link_scheme(url.as_ref()) && !display_text_is_empty(text) {
                urls.push(url.as_ref().to_owned());
            }
            // F-040-P3-001 + F-085-P6-001: do NOT recurse into `text` children here.
            // A Link inside a Link's display text would be an orphan rel.
        },
        // ADV-P14-MED-001: descend through formatting wrappers to reach nested Links.
        InlineNode::Bold(children)
        | InlineNode::Italic(children)
        | InlineNode::Footnote(children)
        | InlineNode::Superscript(children)
        | InlineNode::Subscript(children)
        | InlineNode::Strikethrough(children)
        | InlineNode::Highlight(children) => {
            for child in children {
                collect_hyperlink_urls_from_node(child, urls);
            }
        },
        // Leaf nodes with no Link potential: no registration.
        InlineNode::Plain(_) | InlineNode::Code(_) | InlineNode::Xref(_) | InlineNode::Math(_) => {
        },
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
