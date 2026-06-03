//! DOCX `word/document.xml` body serializer.
//!
//! [`DocumentBodySerializer`] transforms the [`slideforge_layout::LaidOutDeck`]
//! into the `<w:body>` XML fragment that forms the main content of a `.docx`
//! document. It enforces the register routing rules from BC-4.02.001:
//!
//! - `Register::Report` content → narrative body paragraphs under per-slide
//!   `<w:p style="Heading1">` headings
//! - `Register::Detail` content → extended sections after the main body,
//!   under `<w:p style="Heading2">` headings
//! - `Register::Notes` content → **never** emitted to `<w:body>`
//!   (BC-4.02.001 invariant 1 / DI-012)
//!
//! Inline formatting (`InlineNode` variants) is mapped to Word `<w:rPr>`
//! properties per the table in STORY-041 § "Inline Formatting".
//!
//! ## Hyperlink relationship tracking
//!
//! Each `InlineNode::Link { url, .. }` generates a relationship entry in
//! `word/_rels/document.xml.rels`. The serializer assigns sequential `rId`
//! values starting at `rId4` (rId1–rId3 are reserved for styles/numbering/settings)
//! and exposes the accumulated relationship map via
//! [`DocumentBodySerializer::relationships`].

use ooxmlsdk::common::XmlNamespaceDecl;
use ooxmlsdk::schemas::schemas_openxmlformats_org_wordprocessingml_2006_main::{
    Body, BodyChoice, Bold, Document, Hyperlink, HyperlinkChoice, Italic, Paragraph,
    ParagraphChoice, ParagraphProperties, ParagraphStyleId, Run, RunChoice, RunFonts,
    RunProperties, Strike, Text, VerticalPositionValues, VerticalTextAlignment,
};
use ooxmlsdk::sdk::SdkType;
use slideforge_layout::LaidOutDeck;
use slideforge_types::InlineNode;
use slideforge_types::register::Register;

use crate::error::ExportError;

/// W namespace URI for Word processing ML.
const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

/// A hyperlink relationship entry for `word/_rels/document.xml.rels`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperlinkRel {
    /// The sequential relationship ID (e.g., `"rId4"`).
    pub r_id: String,
    /// The target URL.
    pub target: String,
}

/// Serializes a [`LaidOutDeck`] into the `word/document.xml` body XML and
/// accumulates hyperlink relationships.
pub struct DocumentBodySerializer {
    /// Hyperlink relationships collected during serialization.
    relationships: Vec<HyperlinkRel>,
    /// Next relationship ID counter.
    ///
    /// Starts at 4 because `word/_rels/document.xml.rels` reserves:
    /// - `rId1` → `word/styles.xml`
    /// - `rId2` → `word/numbering.xml`
    /// - `rId3` → `word/settings.xml`
    ///
    /// Hyperlink IDs must not collide with these fixed relationship IDs.
    next_rel_id: u32,
}

impl DocumentBodySerializer {
    /// Create a new serializer.
    ///
    /// Hyperlink `rId` allocation begins at `rId4` so it cannot collide with
    /// the three fixed document-relationship IDs (styles, numbering, settings).
    #[must_use]
    pub fn new() -> Self {
        Self {
            relationships: Vec::new(),
            // rId1..=rId3 are reserved for fixed document relationships
            // (styles.xml, numbering.xml, settings.xml) in build_document_rels.
            next_rel_id: 4,
        }
    }

    /// Serialize the laid-out deck into `word/document.xml` bytes.
    ///
    /// After calling this method, [`Self::relationships`] contains all
    /// hyperlink relationships discovered during serialization.
    ///
    /// # Register routing
    ///
    /// - Per slide: emits `<w:p style="Heading1">` from the slide title
    ///   (taken from the first `FrameContent::Title` frame), then
    ///   `<w:p style="Normal">` for each `Register::Report` entry in
    ///   `slide.register_content`. If there are no Report entries, emits
    ///   an empty `<w:p/>` (AC-009).
    /// - After all slides: emits `<w:p style="Heading2">` + body paragraphs
    ///   for each `Register::Detail` entry across all slides.
    /// - `Register::Notes` entries are silently skipped (BC-4.02.001
    ///   invariant 1).
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::OoxmlError`] if the XML cannot be constructed.
    pub fn serialize(&mut self, deck: &LaidOutDeck) -> Result<Vec<u8>, ExportError> {
        // Reset state for a fresh serialization.
        self.relationships.clear();
        // rId1..=rId3 are reserved for styles/numbering/settings (see build_document_rels).
        self.next_rel_id = 4;

        let mut body_paragraphs: Vec<BodyChoice> = Vec::new();
        let mut detail_paragraphs: Vec<BodyChoice> = Vec::new();

        for slide in &deck.slides {
            // Extract title from frames — first Title frame wins.
            let title = slide
                .frames
                .iter()
                .find_map(|f| {
                    if let slideforge_layout::types::FrameContent::Title(t) = &f.content {
                        Some(t.as_ref())
                    } else {
                        None
                    }
                })
                .unwrap_or("");

            // Emit Heading1 paragraph for the slide title.
            body_paragraphs.push(BodyChoice::WP(Box::new(make_styled_paragraph(
                "Heading1", title,
            ))));

            // Collect report entries for this slide.
            let report_entries: Vec<&slideforge_types::RegisteredContent> = slide
                .register_content
                .iter()
                .filter(|rc| rc.register == Register::Report)
                .collect();

            if report_entries.is_empty() {
                // AC-009: emit empty Normal paragraph so Word doesn't collapse
                // the heading spacing when no report content follows it.
                body_paragraphs.push(BodyChoice::WP(Box::new(make_empty_paragraph())));
            } else {
                for rc in &report_entries {
                    let para = self.make_inline_paragraph("Normal", &rc.content)?;
                    body_paragraphs.push(BodyChoice::WP(Box::new(para)));
                }
            }

            // Collect detail entries for this slide into the extended section.
            // Emit ONE "Appendix: <title>" Heading2 per slide that has detail
            // content, then emit each detail entry as a Normal paragraph.
            // (F-041-003: heading must not repeat for every detail entry.)
            let detail_entries: Vec<&slideforge_types::RegisteredContent> = slide
                .register_content
                .iter()
                .filter(|rc| rc.register == Register::Detail)
                .collect();

            if !detail_entries.is_empty() {
                // One Heading2 per slide, hoisted out of the per-entry loop.
                let heading2_text = format!("Appendix: {title}");
                detail_paragraphs.push(BodyChoice::WP(Box::new(make_styled_paragraph(
                    "Heading2",
                    &heading2_text,
                ))));

                for rc in detail_entries {
                    let para = self.make_inline_paragraph("Normal", &rc.content)?;
                    detail_paragraphs.push(BodyChoice::WP(Box::new(para)));
                }
            }
        }

        // Append detail paragraphs after all slides' narrative sections.
        body_paragraphs.extend(detail_paragraphs);

        let body = Body {
            body_choice: body_paragraphs,
            ..Body::default()
        };

        let document = Document {
            xmlns: vec![XmlNamespaceDecl::new("w", W_NS)],
            xml_header: ooxmlsdk::common::XmlHeaderType::Standalone,
            body: Some(Box::new(body)),
            ..Document::default()
        };

        let mut buf: Vec<u8> = Vec::new();
        document
            .write_type_xml(&mut buf, "")
            .map_err(|e| ExportError::OoxmlError {
                message: format!("document.xml write error: {e}"),
            })?;

        // Note: quick_xml emits `<w:b />` (with space before />) for empty
        // elements. Both `<w:b/>` and `<w:b />` are equivalent per XML §2.1.
        // We do NOT normalize the byte stream here — a global byte-replace of
        // " />" would corrupt user text content that contains the literal " />"
        // (F-041-002). Assertions and snapshot tests must accept both forms.
        Ok(buf)
    }

    /// Return the hyperlink relationships accumulated during the last call to
    /// [`Self::serialize`].
    ///
    /// Returns an empty slice before [`Self::serialize`] is called.
    #[must_use]
    pub fn relationships(&self) -> &[HyperlinkRel] {
        &self.relationships
    }

    /// Build a paragraph with the given style from a sequence of inline nodes.
    fn make_inline_paragraph(
        &mut self,
        style: &str,
        nodes: &[InlineNode],
    ) -> Result<Paragraph, ExportError> {
        let mut para = Paragraph {
            paragraph_properties: Some(Box::new(ParagraphProperties {
                paragraph_style_id: Some(ParagraphStyleId {
                    val: style.to_owned(),
                }),
                ..ParagraphProperties::default()
            })),
            ..Paragraph::default()
        };

        for node in nodes {
            let choices = self.inline_node_to_paragraph_choices(node)?;
            para.paragraph_choice.extend(choices);
        }

        Ok(para)
    }

    /// Convert an [`InlineNode`] into one or more [`ParagraphChoice`] elements.
    ///
    /// Most variants produce `ParagraphChoice::WR` items. `InlineNode::Link`
    /// produces a single `ParagraphChoice::WHyperlink` that wraps the run
    /// inside a `<w:hyperlink r:id="...">` element so the link is clickable.
    fn inline_node_to_paragraph_choices(
        &mut self,
        node: &InlineNode,
    ) -> Result<Vec<ParagraphChoice>, ExportError> {
        match node {
            InlineNode::Plain(text) => {
                Ok(vec![ParagraphChoice::WR(Box::new(make_plain_run(text)))])
            },

            InlineNode::Bold(children) => {
                let mut choices = Vec::new();
                for child in children {
                    for choice in self.inline_node_to_paragraph_choices(child)? {
                        // Add <w:b/> to any WR run choices; pass others through unchanged.
                        let choice = apply_run_property(choice, |rpr| {
                            rpr.bold = Some(Bold::default());
                        });
                        choices.push(choice);
                    }
                }
                Ok(choices)
            },

            InlineNode::Italic(children) => {
                let mut choices = Vec::new();
                for child in children {
                    for choice in self.inline_node_to_paragraph_choices(child)? {
                        let choice = apply_run_property(choice, |rpr| {
                            rpr.italic = Some(Italic::default());
                        });
                        choices.push(choice);
                    }
                }
                Ok(choices)
            },

            InlineNode::Code(text) => {
                let run = Run {
                    run_properties: Some(Box::new(RunProperties {
                        run_fonts: Some(RunFonts {
                            ascii: Some("Courier New".to_owned()),
                            high_ansi: Some("Courier New".to_owned()),
                            ..RunFonts::default()
                        }),
                        ..RunProperties::default()
                    })),
                    run_choice: vec![RunChoice::WT(Box::new(make_text(text)))],
                    ..Run::default()
                };
                Ok(vec![ParagraphChoice::WR(Box::new(run))])
            },

            InlineNode::Strikethrough(children) => {
                let mut choices = Vec::new();
                for child in children {
                    for choice in self.inline_node_to_paragraph_choices(child)? {
                        let choice = apply_run_property(choice, |rpr| {
                            rpr.strike = Some(Strike::default());
                        });
                        choices.push(choice);
                    }
                }
                Ok(choices)
            },

            InlineNode::Superscript(children) => {
                let mut choices = Vec::new();
                for child in children {
                    for choice in self.inline_node_to_paragraph_choices(child)? {
                        let choice = apply_run_property(choice, |rpr| {
                            rpr.vertical_text_alignment = Some(VerticalTextAlignment {
                                val: VerticalPositionValues::Superscript,
                            });
                        });
                        choices.push(choice);
                    }
                }
                Ok(choices)
            },

            InlineNode::Subscript(children) => {
                let mut choices = Vec::new();
                for child in children {
                    for choice in self.inline_node_to_paragraph_choices(child)? {
                        let choice = apply_run_property(choice, |rpr| {
                            rpr.vertical_text_alignment = Some(VerticalTextAlignment {
                                val: VerticalPositionValues::Subscript,
                            });
                        });
                        choices.push(choice);
                    }
                }
                Ok(choices)
            },

            InlineNode::Link { text, url } => {
                // Record the hyperlink relationship.
                let r_id = format!("rId{}", self.next_rel_id);
                self.next_rel_id += 1;
                self.relationships.push(HyperlinkRel {
                    r_id: r_id.clone(),
                    target: url.to_string(),
                });

                // Emit the link text as a run with the Hyperlink character style,
                // wrapped in a <w:hyperlink r:id="..."> element so the link is
                // clickable in Word and LibreOffice (EC-003 requirement).
                let link_run = Run {
                    run_properties: Some(Box::new(RunProperties {
                        run_style: Some(
                            ooxmlsdk::schemas::schemas_openxmlformats_org_wordprocessingml_2006_main::RunStyle {
                                val: "Hyperlink".to_owned(),
                            },
                        ),
                        ..RunProperties::default()
                    })),
                    run_choice: vec![RunChoice::WT(Box::new(make_text(&collect_plain_text(text))))],
                    ..Run::default()
                };

                let hyperlink = Hyperlink {
                    id: Some(r_id),
                    hyperlink_choice: vec![HyperlinkChoice::WR(Box::new(link_run))],
                    ..Hyperlink::default()
                };

                Ok(vec![ParagraphChoice::WHyperlink(Box::new(hyperlink))])
            },

            // For unsupported inline variants (Math, Footnote, Xref, Highlight),
            // fall back to plain text extraction.
            InlineNode::Math(math_node) => Ok(vec![ParagraphChoice::WR(Box::new(make_plain_run(
                math_node.latex.as_ref(),
            )))]),
            InlineNode::Footnote(children) | InlineNode::Highlight(children) => {
                let text = collect_plain_text(children);
                Ok(vec![ParagraphChoice::WR(Box::new(make_plain_run(&text)))])
            },
            InlineNode::Xref(target) => Ok(vec![ParagraphChoice::WR(Box::new(make_plain_run(
                target.as_ref(),
            )))]),
        }
    }
}

impl Default for DocumentBodySerializer {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Apply a run-property mutation to a [`ParagraphChoice`] if it is a `WR` variant.
///
/// For non-`WR` choices (e.g., `WHyperlink`), the choice passes through
/// unchanged — caller is responsible for handling those cases if needed.
fn apply_run_property<F>(choice: ParagraphChoice, f: F) -> ParagraphChoice
where
    F: FnOnce(&mut RunProperties),
{
    match choice {
        ParagraphChoice::WR(mut run) => {
            let rpr = run
                .run_properties
                .get_or_insert_with(|| Box::new(RunProperties::default()));
            f(rpr);
            ParagraphChoice::WR(run)
        },
        other => other,
    }
}

/// Build a paragraph with the given style containing a single plain-text run.
fn make_styled_paragraph(style: &str, text: &str) -> Paragraph {
    Paragraph {
        paragraph_properties: Some(Box::new(ParagraphProperties {
            paragraph_style_id: Some(ParagraphStyleId {
                val: style.to_owned(),
            }),
            ..ParagraphProperties::default()
        })),
        paragraph_choice: vec![ParagraphChoice::WR(Box::new(Run {
            run_choice: vec![RunChoice::WT(Box::new(make_text(text)))],
            ..Run::default()
        }))],
        ..Paragraph::default()
    }
}

/// Build an empty paragraph (no runs, no style).
fn make_empty_paragraph() -> Paragraph {
    Paragraph::default()
}

/// Build a plain-text run with no run properties.
fn make_plain_run(text: &str) -> Run {
    Run {
        run_choice: vec![RunChoice::WT(Box::new(make_text(text)))],
        ..Run::default()
    }
}

/// Build a `<w:t>` element with `xml:space="preserve"` when the text has
/// leading or trailing whitespace (per STORY-041 requirement).
fn make_text(text: &str) -> Text {
    let needs_preserve = text.starts_with(' ')
        || text.ends_with(' ')
        || text.starts_with('\t')
        || text.ends_with('\t');

    Text {
        xml_content: Some(text.to_owned()),
        space: if needs_preserve {
            Some(ooxmlsdk::schemas::xml::SpaceProcessingModeValues::Preserve)
        } else {
            None
        },
        ..Text::default()
    }
}

/// Recursively extract plain text from a slice of inline nodes (used for
/// fallback text in link display content and unsupported variants).
fn collect_plain_text(nodes: &[InlineNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            InlineNode::Bold(ch)
            | InlineNode::Italic(ch)
            | InlineNode::Strikethrough(ch)
            | InlineNode::Superscript(ch)
            | InlineNode::Subscript(ch)
            | InlineNode::Footnote(ch)
            | InlineNode::Highlight(ch) => out.push_str(&collect_plain_text(ch)),
            InlineNode::Link { text, .. } => out.push_str(&collect_plain_text(text)),
            InlineNode::Math(m) => out.push_str(&m.latex),
            InlineNode::Plain(t) | InlineNode::Code(t) | InlineNode::Xref(t) => {
                out.push_str(t);
            },
        }
    }
    out
}
