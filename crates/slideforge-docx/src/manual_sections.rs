//! Manually-authored document section serializer for DOCX output.
//!
//! [`ManualSectionSerializer`] converts a [`slideforge_layout::sections::GeneratedSection`]
//! with [`slideforge_layout::sections::SectionSource::ManuallyAuthored`] into
//! `<w:body>` XML: a `Heading1` paragraph followed by the section's content.
//!
//! ## Invariant (BC-4.02.002 invariant 3)
//!
//! Manual sections are NOT merged with auto-generated sections of the same
//! logical name. Each manual section is serialized after all auto-generated
//! sections, in the deck source order as determined by
//! [`crate::section_order::SectionOrderer`].
//!
//! ## Content blocks
//!
//! Content paragraphs are sourced from
//! [`slideforge_layout::sections::GeneratedSection::register_content`]: only
//! `Register::Report` and `Register::Detail` entries are emitted.
//! `Register::Notes` is excluded per BC-4.02.001 invariant 1 / DI-012.

use ooxmlsdk::schemas::schemas_openxmlformats_org_wordprocessingml_2006_main::{
    BodyChoice, Languages, Paragraph, ParagraphChoice, ParagraphProperties, ParagraphStyleId, Run,
    RunChoice, RunProperties, Text,
};
use slideforge_layout::sections::GeneratedSection;
use slideforge_types::InlineNode;
use slideforge_types::register::Register;

use crate::error::ExportError;
use crate::xml_escape::strip_xml10_invalid_chars;

/// Serializes manually-authored [`GeneratedSection`] entries into `<w:body>` XML.
///
/// Emits a `Heading1` paragraph using the section's `heading` field, followed
/// by one paragraph per item in `register_content` (Report and Detail only).
pub struct ManualSectionSerializer {
    /// BCP-47 language tag threaded to every emitted run (BC-5.01.005 PC-4).
    lang: String,
}

impl ManualSectionSerializer {
    /// Create a new serializer with the given BCP-47 language tag.
    ///
    /// The `lang` value is placed in `<w:rPr><w:lang w:val="..."/>` on every
    /// run emitted by this serializer (BC-5.01.005 PC-4 universality).
    #[must_use]
    pub fn new(lang: &str) -> Self {
        Self {
            lang: lang.to_owned(),
        }
    }

    /// Serialize one manually-authored section into a list of [`BodyChoice`] elements.
    ///
    /// Emits:
    /// 1. `Heading1` paragraph with `section.heading` as text.
    /// 2. One `Normal` paragraph per `Register::Report` or `Register::Detail` entry in
    ///    `section.register_content`.
    ///
    /// `Register::Notes` entries are silently skipped (BC-4.02.001 invariant 1).
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::OoxmlError`] if OOXML construction fails.
    pub fn serialize_section(
        &self,
        section: &GeneratedSection,
    ) -> Result<Vec<BodyChoice>, ExportError> {
        let mut output: Vec<BodyChoice> = Vec::new();

        // Heading1 paragraph for the section heading.
        output.push(BodyChoice::WP(Box::new(make_styled_paragraph(
            "Heading1",
            section.heading.as_ref(),
            &self.lang,
        ))));

        // Emit one Normal paragraph per Report/Detail entry in register_content.
        // Notes entries are skipped (BC-4.02.001 invariant 1 / DI-012).
        for rc in &section.register_content {
            if rc.register == Register::Notes {
                continue;
            }
            let text = collect_plain_text(&rc.content);
            output.push(BodyChoice::WP(Box::new(make_styled_paragraph(
                "Normal", &text, &self.lang,
            ))));
        }

        Ok(output)
    }
}

impl Default for ManualSectionSerializer {
    fn default() -> Self {
        Self::new(crate::DEFAULT_DECK_LANG)
    }
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Build a paragraph with the given style containing a single plain-text run.
///
/// The run carries `<w:rPr><w:lang w:val="LANG"/></w:rPr>` so that every run
/// emitted by this serializer satisfies the BC-5.01.005 PC-4 universality
/// requirement (all runs must have lang — mirrors `make_lang_run` in
/// `document_body.rs`).
fn make_styled_paragraph(style: &str, text: &str, lang: &str) -> Paragraph {
    let sanitized = strip_xml10_invalid_chars(text);
    let needs_preserve = sanitized.starts_with(' ')
        || sanitized.ends_with(' ')
        || sanitized.starts_with('\t')
        || sanitized.ends_with('\t');

    Paragraph {
        paragraph_properties: Some(Box::new(ParagraphProperties {
            paragraph_style_id: Some(ParagraphStyleId {
                val: style.to_owned(),
            }),
            ..ParagraphProperties::default()
        })),
        paragraph_choice: vec![ParagraphChoice::WR(Box::new(Run {
            run_properties: Some(Box::new(RunProperties {
                languages: Some(Languages {
                    val: Some(lang.to_owned()),
                    ..Languages::default()
                }),
                ..RunProperties::default()
            })),
            run_choice: vec![RunChoice::WT(Box::new(Text {
                xml_content: Some(sanitized),
                space: if needs_preserve {
                    Some(ooxmlsdk::schemas::xml::SpaceProcessingModeValues::Preserve)
                } else {
                    None
                },
                ..Text::default()
            }))],
            ..Run::default()
        }))],
        ..Paragraph::default()
    }
}

/// Recursively extract plain text from a slice of inline nodes.
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
