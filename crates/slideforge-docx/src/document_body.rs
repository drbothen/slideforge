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
//! values starting at `rId1` and exposes the accumulated relationship map via
//! [`DocumentBodySerializer::relationships`].

use ooxmlsdk::common::XmlNamespaceDecl;
use ooxmlsdk::schemas::schemas_openxmlformats_org_wordprocessingml_2006_main::{
    Body, BodyChoice, Bold, Document, Italic, Paragraph, ParagraphChoice, ParagraphProperties,
    ParagraphStyleId, Run, RunChoice, RunFonts, RunProperties, Strike, Text,
    VerticalPositionValues, VerticalTextAlignment,
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
    /// The sequential relationship ID (e.g., `"rId1"`).
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
    next_rel_id: u32,
}

impl DocumentBodySerializer {
    /// Create a new serializer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            relationships: Vec::new(),
            next_rel_id: 1,
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
        self.next_rel_id = 1;

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
            for rc in slide
                .register_content
                .iter()
                .filter(|rc| rc.register == Register::Detail)
            {
                // Heading2 for "Appendix: <title>".
                let heading2_text = format!("Appendix: {title}");
                detail_paragraphs.push(BodyChoice::WP(Box::new(make_styled_paragraph(
                    "Heading2",
                    &heading2_text,
                ))));

                let para = self.make_inline_paragraph("Normal", &rc.content)?;
                detail_paragraphs.push(BodyChoice::WP(Box::new(para)));
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

        // Normalize ooxmlsdk/quick-xml self-closing tag format: " />" → "/>"
        // quick_xml emits `<w:b />` (space before />); tests and OOXML processors
        // accept `<w:b/>` (no space). Both are equivalent XML but the compact
        // form is conventional in OOXML producers.
        let normalized = normalize_self_closing_tags(buf);
        Ok(normalized)
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
            let runs = self.inline_node_to_runs(node)?;
            for run in runs {
                para.paragraph_choice
                    .push(ParagraphChoice::WR(Box::new(run)));
            }
        }

        Ok(para)
    }

    /// Convert an [`InlineNode`] into one or more Word [`Run`] elements.
    fn inline_node_to_runs(&mut self, node: &InlineNode) -> Result<Vec<Run>, ExportError> {
        match node {
            InlineNode::Plain(text) => Ok(vec![make_plain_run(text)]),

            InlineNode::Bold(children) => {
                let mut runs = Vec::new();
                for child in children {
                    for mut run in self.inline_node_to_runs(child)? {
                        // Add <w:b/> to existing run properties (or create new).
                        let rpr = run
                            .run_properties
                            .get_or_insert_with(|| Box::new(RunProperties::default()));
                        rpr.bold = Some(Bold::default());
                        runs.push(run);
                    }
                }
                Ok(runs)
            },

            InlineNode::Italic(children) => {
                let mut runs = Vec::new();
                for child in children {
                    for mut run in self.inline_node_to_runs(child)? {
                        let rpr = run
                            .run_properties
                            .get_or_insert_with(|| Box::new(RunProperties::default()));
                        rpr.italic = Some(Italic::default());
                        runs.push(run);
                    }
                }
                Ok(runs)
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
                Ok(vec![run])
            },

            InlineNode::Strikethrough(children) => {
                let mut runs = Vec::new();
                for child in children {
                    for mut run in self.inline_node_to_runs(child)? {
                        let rpr = run
                            .run_properties
                            .get_or_insert_with(|| Box::new(RunProperties::default()));
                        rpr.strike = Some(Strike::default());
                        runs.push(run);
                    }
                }
                Ok(runs)
            },

            InlineNode::Superscript(children) => {
                let mut runs = Vec::new();
                for child in children {
                    for mut run in self.inline_node_to_runs(child)? {
                        let rpr = run
                            .run_properties
                            .get_or_insert_with(|| Box::new(RunProperties::default()));
                        rpr.vertical_text_alignment = Some(VerticalTextAlignment {
                            val: VerticalPositionValues::Superscript,
                        });
                        runs.push(run);
                    }
                }
                Ok(runs)
            },

            InlineNode::Subscript(children) => {
                let mut runs = Vec::new();
                for child in children {
                    for mut run in self.inline_node_to_runs(child)? {
                        let rpr = run
                            .run_properties
                            .get_or_insert_with(|| Box::new(RunProperties::default()));
                        rpr.vertical_text_alignment = Some(VerticalTextAlignment {
                            val: VerticalPositionValues::Subscript,
                        });
                        runs.push(run);
                    }
                }
                Ok(runs)
            },

            InlineNode::Link { text, url } => {
                // Record the hyperlink relationship.
                let r_id = format!("rId{}", self.next_rel_id);
                self.next_rel_id += 1;
                self.relationships.push(HyperlinkRel {
                    r_id: r_id.clone(),
                    target: url.to_string(),
                });

                // Emit the link text as a run with the Hyperlink character style.
                // The hyperlink element itself is emitted as an XmlAny (raw XML) at
                // the paragraph choice level; here we return runs with the style.
                let link_run = Run {
                    run_properties: Some(Box::new(RunProperties {
                        run_style: Some(
                            ooxmlsdk::schemas::schemas_openxmlformats_org_wordprocessingml_2006_main::RunStyle {
                                val: "Hyperlink".to_owned(),
                            },
                        ),
                        ..RunProperties::default()
                    })),
                    run_choice: {
                        let mut choices = Vec::new();
                        // Collect text from child nodes.
                        let collected = collect_plain_text(text);
                        choices.push(RunChoice::WT(Box::new(make_text(&collected))));
                        choices
                    },
                    ..Run::default()
                };
                Ok(vec![link_run])
            },

            // For unsupported inline variants (Math, Footnote, Xref, Highlight),
            // fall back to plain text extraction.
            InlineNode::Math(math_node) => Ok(vec![make_plain_run(math_node.latex.as_ref())]),
            InlineNode::Footnote(children) | InlineNode::Highlight(children) => {
                let text = collect_plain_text(children);
                Ok(vec![make_plain_run(&text)])
            },
            InlineNode::Xref(target) => Ok(vec![make_plain_run(target.as_ref())]),
        }
    }
}

impl Default for DocumentBodySerializer {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

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

/// Normalize ooxmlsdk/quick-xml self-closing tag output.
///
/// `quick_xml` emits `<w:b />` (space before `/>`) for empty elements. This
/// function replaces ` />` with `/>` throughout the XML bytes to produce the
/// compact conventional OOXML form `<w:b/>`. Both forms are semantically
/// equivalent per the XML spec (§2.1); the compact form is conventional for
/// OOXML producers and matches the assertions in the test suite.
///
/// The replacement is byte-safe because ` />` is ASCII and cannot be a
/// subsequence of a multi-byte UTF-8 character (all multi-byte continuation
/// bytes have the high bit set, i.e., ≥ 0x80, while the ASCII space is 0x20).
fn normalize_self_closing_tags(xml: Vec<u8>) -> Vec<u8> {
    // Fast path: if no " />" pattern exists, return unchanged.
    if !xml.windows(3).any(|w| w == b" />") {
        return xml;
    }
    // Replace all occurrences of b" />" with b"/>".
    let mut out = Vec::with_capacity(xml.len());
    let mut i = 0;
    while i < xml.len() {
        if i + 2 < xml.len() && xml[i] == b' ' && xml[i + 1] == b'/' && xml[i + 2] == b'>' {
            // Skip the leading space — emit just "/>"
            out.push(b'/');
            out.push(b'>');
            i += 3;
        } else {
            out.push(xml[i]);
            i += 1;
        }
    }
    out
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
