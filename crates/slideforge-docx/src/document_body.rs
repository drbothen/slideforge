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
    Body, BodyChoice, Bold, Document, Highlight, HighlightColorValues, Hyperlink, HyperlinkChoice,
    Italic, Languages, NumberingId, NumberingLevelReference, NumberingProperties,
    PageSize as OoxmlPageSize, Paragraph, ParagraphChoice, ParagraphProperties, ParagraphStyleId,
    Run, RunChoice, RunFonts, RunProperties, SectionProperties, Shading, ShadingPatternValues,
    Strike, Text, VerticalPositionValues, VerticalTextAlignment,
};
use ooxmlsdk::sdk::SdkType;
use slideforge_layout::LaidOutDeck;
use slideforge_types::register::Register;
use slideforge_types::{Deck, FieldValue, InlineNode};

use crate::auto_sections::AutoSectionSerializer;
use crate::error::ExportError;
use crate::manual_sections::ManualSectionSerializer;
use crate::section_order::SectionOrderer;
use crate::xml_escape::strip_xml10_invalid_chars;

/// Allowlist of permitted link URL schemes at the exporter layer (defense-in-depth).
///
/// Mirrors `slideforge_syntax::parser::template::ALLOWED_LINK_SCHEMES` (E-PAR-022).
/// The parse-layer allowlist covers DSL-sourced input; this check covers
/// programmatic `InlineNode::Link` callers that bypass the parser.
///
/// Comparison is case-insensitive (scheme is lowercased before checking).
const ALLOWED_LINK_SCHEMES: &[&str] = &["http", "https", "mailto"];

/// Extract the URL scheme (the portion before the first `:`), lowercased.
///
/// Returns `"(none)"` when the URL contains no `:` separator.
fn extract_url_scheme(url: &str) -> String {
    match url.find(':') {
        Some(pos) => url[..pos].to_lowercase(),
        None => "(none)".to_string(),
    }
}

/// W namespace URI for Word processing ML.
const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

/// R namespace URI for Office document relationships.
///
/// Required for `r:id` attributes used in `<w:hyperlink r:id="...">` elements
/// (BC-4.02.001 / F-DOCX-001). Without this declaration, any document
/// containing an `InlineNode::Link` would produce namespace-malformed XML.
const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// A hyperlink relationship entry for `word/_rels/document.xml.rels`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperlinkRel {
    /// The sequential relationship ID (e.g., `"rId4"`).
    pub r_id: String,
    /// The target URL.
    pub target: String,
}

/// Left x-coordinate (in EMU) of the body content area.
///
/// Bullet frames produced by the layout engine start at this x-position for
/// depth-0 (ilvl=0) items. Each additional nesting level adds one
/// `BULLET_DEPTH_INDENT_EMU` to the left edge.
const BULLET_BODY_LEFT_EMU: i64 = 457_200;

/// EMU added per bullet nesting level.
///
/// Depth = `(frame.bbox.x - BULLET_BODY_LEFT_EMU) / BULLET_DEPTH_INDENT_EMU`.
/// Clamped to 0 for frames whose x is at or left of the body edge.
const BULLET_DEPTH_INDENT_EMU: i64 = 457_200;

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
    /// BCP-47 language tag for `<w:lang w:val="..."/>` on every run.
    ///
    /// Set from `DeckMetadata.lang` at the start of each `serialize` call.
    /// Falls back to [`crate::DEFAULT_DECK_LANG`] when the deck carries no `lang`
    /// declaration (BC-5.01.004 / BC-5.01.005).
    lang: String,
}

impl DocumentBodySerializer {
    /// Create a new serializer.
    ///
    /// Hyperlink `rId` allocation begins at `rId4` so it cannot collide with
    /// the three fixed document-relationship IDs (styles, numbering, settings).
    /// The `lang` field is set to [`crate::DEFAULT_DECK_LANG`] and overwritten at the
    /// start of each [`Self::serialize`] call.
    #[must_use]
    pub fn new() -> Self {
        Self {
            relationships: Vec::new(),
            // rId1..=rId3 are reserved for fixed document relationships
            // (styles.xml, numbering.xml, settings.xml) in build_document_rels.
            next_rel_id: 4,
            lang: crate::DEFAULT_DECK_LANG.to_owned(),
        }
    }

    /// Serialize the laid-out deck into `word/document.xml` bytes.
    ///
    /// After calling this method, [`Self::relationships`] contains all
    /// hyperlink relationships discovered during serialization.
    ///
    /// `semantic_deck` is the original semantic IR ([`slideforge_types::Deck`])
    /// used for STORY-081 I2 dual-title: when a slide title carries inline markup
    /// (e.g., `**Bold Title**`), the `title_inlines` shadow field in the semantic
    /// slide is used to emit a rich Heading1 paragraph instead of plain text.
    /// The PPTX exporter ignores this shadow field (PPTX single-run constraint).
    ///
    /// # Register routing
    ///
    /// - Per slide: emits `<w:p style="Heading1">` from the slide title, then
    ///   `<w:p style="Normal">` for each `Register::Report` entry.
    ///   If there are no Report entries, emits an empty `<w:p/>` (AC-009).
    /// - After all slides: emits `<w:p style="Heading2">` + body paragraphs
    ///   for each `Register::Detail` entry across all slides.
    /// - `Register::Notes` entries are silently skipped (BC-4.02.001 invariant 1).
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::OoxmlError`] if the XML cannot be constructed.
    // `serialize` is a single-pass accumulator over slides; splitting it would
    // obscure the sequential semantics of the body/detail paragraph routing
    // (BC-4.02.001 invariant 1) without improving readability.
    #[allow(clippy::too_many_lines)]
    pub fn serialize(
        &mut self,
        deck: &LaidOutDeck,
        semantic_deck: &Deck,
    ) -> Result<Vec<u8>, ExportError> {
        // Reset state for a fresh serialization.
        self.relationships.clear();
        // rId1..=rId3 are reserved for styles/numbering/settings (see build_document_rels).
        self.next_rel_id = 4;
        // Capture the deck language for run-level <w:lang> emission.
        // The canonical default is "en" per BC-5.01.004 / BC-5.01.005.
        semantic_deck
            .metadata
            .lang
            .as_deref()
            .unwrap_or(crate::DEFAULT_DECK_LANG)
            .clone_into(&mut self.lang);

        let mut body_paragraphs: Vec<BodyChoice> = Vec::new();
        let mut detail_paragraphs: Vec<BodyChoice> = Vec::new();

        for slide in &deck.slides {
            // ── Title (FrameContent::Title → Heading1) ───────────────────────
            // Extract title from FrameContent::Title frames (tag-driven, STORY-086 / ADR-019
            // Decision 3 / BC-4.02.001 v1.2 postcondition 8). No positional TextRun fallback —
            // after Stage 2b, the title field always produces TextTag::Title → FrameContent::Title.
            // AC-023: routing is tag-driven, NOT position-driven.
            //
            // STORY-081 I2 (dual-title): check for preserved inline title structure.
            // If the semantic slide has a "title_inlines" shadow field (set by eval_slide_node
            // when the title contains inline markup), use it for the Heading1 paragraph so
            // DOCX renders **Bold Title** as bold rather than plain text.
            // The PPTX exporter ignores this shadow field (PPTX single-run constraint).
            // Extract plain-text title (used for both the Heading1 paragraph fallback
            // and for the "Appendix: {title}" detail section heading below).
            let title: &str = slide
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

            // STORY-081 I2 (dual-title): check for preserved inline title structure.
            // If the semantic slide has a "title_inlines" shadow field (set by eval_slide_node
            // when the title contains inline markup), use it for the Heading1 paragraph so
            // DOCX renders **Bold Title** as bold rather than plain text.
            // The PPTX exporter ignores this shadow field (PPTX single-run constraint).
            let title_inlines_opt: Option<&[slideforge_types::InlineNode]> =
                semantic_deck.slides.get(slide.source_index).and_then(|s| {
                    if let Some(FieldValue::Inlines(nodes)) = s.fields.get("title_inlines") {
                        Some(nodes.as_slice())
                    } else {
                        None
                    }
                });

            if let Some(inlines) = title_inlines_opt {
                // Title has inline markup: emit Heading1 with rich inline runs.
                let para = self
                    .make_inline_paragraph("Heading1", inlines)
                    .map_err(|e| ExportError::OoxmlError {
                        message: format!("Heading1 inline title paragraph: {e}"),
                    })?;
                body_paragraphs.push(BodyChoice::WP(Box::new(para)));
            } else {
                // Plain title path (no inline markup, or no shadow field).
                body_paragraphs.push(BodyChoice::WP(Box::new(
                    self.make_styled_lang_paragraph("Heading1", title),
                )));
            }

            // ── Subtitle (FrameContent::Subtitle / SubtitleInlines → Heading2) ───
            // BC-4.02.001 v1.2 postcondition 9: subtitle field → Heading2 paragraph.
            // STORY-081 C3: SubtitleInlines carries rich inline structure;
            // render it with make_inline_paragraph so Bold/Italic/Code are preserved.
            for frame in &slide.frames {
                match &frame.content {
                    slideforge_layout::types::FrameContent::Subtitle(t) => {
                        body_paragraphs.push(BodyChoice::WP(Box::new(
                            self.make_styled_lang_paragraph("Heading2", t.as_ref()),
                        )));
                    },
                    slideforge_layout::types::FrameContent::SubtitleInlines(nodes) => {
                        let para = self.make_inline_paragraph("Heading2", nodes).map_err(|e| {
                            ExportError::OoxmlError {
                                message: format!("Heading2 inline subtitle paragraph: {e}"),
                            }
                        })?;
                        body_paragraphs.push(BodyChoice::WP(Box::new(para)));
                    },
                    _ => {},
                }
            }

            // ── Body (FrameContent::Body → Normal paragraphs) ─────────────────
            // BC-4.02.001 v1.2 postcondition 10: body field → FrameContent::Body →
            // Normal-styled paragraphs (no Heading style). Body text MUST NOT appear
            // in a Heading1 paragraph (AC-020: tag-driven routing invariant).
            //
            // STORY-081 AC-003: use make_inline_paragraph to preserve inline markup
            // (Bold, Italic, Highlight, etc.) instead of discarding non-Plain nodes.
            for frame in &slide.frames {
                if let slideforge_layout::types::FrameContent::Body(blocks) = &frame.content {
                    for content_block in blocks {
                        if let slideforge_types::ContentBlock::Text(text_block) = content_block
                            && !text_block.inlines.is_empty()
                        {
                            let para = self
                                .make_inline_paragraph("Normal", &text_block.inlines)
                                .map_err(|e| ExportError::OoxmlError {
                                    message: format!("Body frame inline paragraph: {e}"),
                                })?;
                            body_paragraphs.push(BodyChoice::WP(Box::new(para)));
                        }
                    }
                }
            }

            // ── TextRun (FrameContent::TextRun → bullet list paragraphs) ──────
            // BC-4.02.001 v1.2: STORY-073 — the layout stage converts each
            // BulletItem in ContentBlock::Bullets to a FrameContent::TextRun frame
            // (one per bullet item). The DOCX exporter renders each TextRun as a
            // list-style paragraph with `<w:numPr>` for Word 365 list recognition.
            //
            // Nesting depth (ilvl) is derived from the frame's bbox x-coordinate:
            //   ilvl = (x - BULLET_BODY_LEFT_EMU) / BULLET_DEPTH_INDENT_EMU
            // clamped to [0, 2].
            //
            // An empty InlineNode slice (`inlines.is_empty()`) means an empty bullet
            // string (EC-005). The paragraph is emitted with numPr anyway so the list
            // position is preserved; the run carries an empty `<w:t/>`.
            //
            // STORY-081×STORY-088: TextRun frames from markup-bearing list-literal
            // bullets carry InlineNode::Bold / Italic etc. — rendered with
            // make_inline_paragraph so markup is preserved (not stripped to plain text).
            for frame in &slide.frames {
                if let slideforge_layout::types::FrameContent::TextRun(inlines) = &frame.content {
                    // Derive nesting level from x-coordinate.
                    // `depth` is i64 in [0, 2] after the min() clamp; i32::try_from
                    // will always succeed because the value is bounded to ≤2.
                    let raw_x = frame.bbox.x.0;
                    let ilvl: i32 = if raw_x <= BULLET_BODY_LEFT_EMU {
                        0
                    } else {
                        let depth = (raw_x - BULLET_BODY_LEFT_EMU) / BULLET_DEPTH_INDENT_EMU;
                        // Clamp to OOXML max ilvl=2; result fits i32 trivially.
                        i32::try_from(depth.min(2)).unwrap_or(2)
                    };

                    let para = self.make_bullet_paragraph(ilvl, inlines).map_err(|e| {
                        ExportError::OoxmlError {
                            message: format!("TextRun bullet paragraph: {e}"),
                        }
                    })?;
                    body_paragraphs.push(BodyChoice::WP(Box::new(para)));
                }
            }

            // ── ColorBar (FrameContent::ColorBar → percentage text fallback) ───
            // BC-1.17.002 PC-9 (DOCX clause): "DOCX: percentage text fallback".
            // DOCX has no native progress-bar element, so the DOCX exporter emits
            // the bar percentage as a standalone Normal-styled paragraph containing
            // exactly "{percent}%" (e.g., `<w:t>75%</w:t>`). This is a DEDICATED
            // bar representation from the ColorBar frame — distinct from the
            // ColorLabel text block ("75% complete") which is routed via Body above.
            // The discriminating test (`test_BC_1_17_002_docx_bar_percentage_text_in_document_xml`)
            // asserts `<w:t>75%</w:t>` specifically to distinguish this from the label.
            for frame in &slide.frames {
                if let slideforge_layout::types::FrameContent::ColorBar { percent, .. } =
                    &frame.content
                {
                    // OBS-P6-002: use the canonical `percent` field carried from
                    // `ColorBarSpec.percent` through layout::run. Do NOT re-derive
                    // from (filled_width_emu * 100) / total_width_emu — that second
                    // integer floor produces an off-by-one error when total_width_emu
                    // is not divisible by 100.
                    let percent_text = format!("{percent}%");
                    body_paragraphs.push(BodyChoice::WP(Box::new(
                        self.make_styled_lang_paragraph("Normal", &percent_text),
                    )));
                }
            }

            // ── Shape (FrameContent::Shape → DOCX solid fallback) ───────────────
            // AC-004 (STORY-072): Gradient shapes use solid fallback (see helper).
            for frame in &slide.frames {
                if let slideforge_layout::types::FrameContent::Shape(sf) = &frame.content
                    && let slideforge_layout::types::FillSpec::Gradient { from, .. } = &sf.fill
                {
                    body_paragraphs.push(BodyChoice::WP(Box::new(
                        make_gradient_solid_fallback_paragraph(*from),
                    )));
                }
            }

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
                detail_paragraphs.push(BodyChoice::WP(Box::new(
                    self.make_styled_lang_paragraph("Heading2", &heading2_text),
                )));

                for rc in detail_entries {
                    let para = self.make_inline_paragraph("Normal", &rc.content)?;
                    detail_paragraphs.push(BodyChoice::WP(Box::new(para)));
                }
            }
        }

        // Append detail paragraphs after all slides' narrative sections.
        body_paragraphs.extend(detail_paragraphs);

        // ── Section serialization (BC-4.02.002) ──────────────────────────────
        // Append auto-generated sections (executive_summary, risk_register, …)
        // followed by manually-authored sections, in canonical order.
        // (SectionOrderer: auto before manual, per postcondition 5.)
        let orderer = SectionOrderer::new(deck);
        let auto_ser = AutoSectionSerializer::new(&self.lang);
        let manual_ser = ManualSectionSerializer::new(&self.lang);

        for section in orderer.ordered_sections() {
            use slideforge_layout::sections::SectionSource;
            let section_elements = match section.source {
                SectionSource::AutoGenerated => auto_ser.serialize_section(section)?,
                SectionSource::ManuallyAuthored => manual_ser.serialize_section(section)?,
            };
            body_paragraphs.extend(section_elements);
        }

        // ── sectPr: page dimensions from the layout page size ────────────────
        // BC-4.02.001 postcondition 2: the DOCX body must end with <w:sectPr>
        // carrying <w:pgSz w:w="W" w:h="H"/> so Word 365 knows the page canvas.
        // Dimensions are converted from EMU to twentieths-of-a-point (twips):
        //   twips = EMU × 1440 / 914_400 = EMU / 635  (exact for standard sizes)
        // EMU values are always positive (validated by the layout engine), so
        // the try_from conversion from i64 to u32 is always successful in practice.
        let width_twips = u32::try_from(deck.page_size.width.0 / 635).unwrap_or(14_400);
        let height_twips = u32::try_from(deck.page_size.height.0 / 635).unwrap_or(8_100);
        let sect_pr = SectionProperties {
            w_pg_sz: Some(OoxmlPageSize {
                width: Some(width_twips),
                height: Some(height_twips),
                ..OoxmlPageSize::default()
            }),
            ..SectionProperties::default()
        };

        let body = Body {
            body_choice: body_paragraphs,
            w_sect_pr: Some(Box::new(sect_pr)),
            ..Body::default()
        };

        let document = Document {
            // Declare both the w: namespace (Word processing ML) and the r:
            // namespace (Office relationships). The r: namespace is required
            // for `r:id` attributes on `<w:hyperlink>` elements (F-DOCX-001).
            // Omitting it produces namespace-malformed XML for any deck that
            // contains an InlineNode::Link.
            xmlns: vec![
                XmlNamespaceDecl::new("w", W_NS),
                XmlNamespaceDecl::new("r", R_NS),
            ],
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

    /// Build a bullet list paragraph for a `FrameContent::TextRun` frame.
    ///
    /// Produces a `Normal`-styled paragraph with `<w:numPr>` carrying the
    /// supplied `ilvl` and `numId=BULLET_NUM_ID`. Run content is emitted via
    /// [`Self::inline_node_to_paragraph_choices`] so inline markup (Bold,
    /// Italic, etc.) is preserved. An empty `inlines` slice yields a paragraph
    /// with numPr but no run content (EC-005: empty bullet string preserved).
    fn make_bullet_paragraph(
        &mut self,
        ilvl: i32,
        inlines: &[InlineNode],
    ) -> Result<Paragraph, ExportError> {
        let num_pr = NumberingProperties {
            numbering_level_reference: Some(NumberingLevelReference { val: ilvl }),
            numbering_id: Some(NumberingId {
                val: crate::numbering::BULLET_NUM_ID,
            }),
            ..NumberingProperties::default()
        };

        let mut para = Paragraph {
            paragraph_properties: Some(Box::new(ParagraphProperties {
                paragraph_style_id: Some(ParagraphStyleId {
                    val: "Normal".to_owned(),
                }),
                numbering_properties: Some(Box::new(num_pr)),
                ..ParagraphProperties::default()
            })),
            ..Paragraph::default()
        };

        for node in inlines {
            let choices = self.inline_node_to_paragraph_choices(node)?;
            para.paragraph_choice.extend(choices);
        }

        // EC-005: if the inlines slice was empty, emit a single empty run so the
        // list item is structurally complete and not a bare <w:p/>.
        if inlines.is_empty() {
            para.paragraph_choice
                .push(ParagraphChoice::WR(Box::new(self.make_lang_run(""))));
        }

        Ok(para)
    }

    /// Build a styled paragraph with a single plain-text run carrying `<w:lang>`.
    ///
    /// Replaces the free-function `make_styled_paragraph` at all call sites
    /// inside the serializer so that heading and normal paragraphs also carry
    /// the deck language on their runs (BC-5.01.005 PC-4 universality).
    fn make_styled_lang_paragraph(&self, style: &str, text: &str) -> Paragraph {
        Paragraph {
            paragraph_properties: Some(Box::new(ParagraphProperties {
                paragraph_style_id: Some(ParagraphStyleId {
                    val: style.to_owned(),
                }),
                ..ParagraphProperties::default()
            })),
            paragraph_choice: vec![ParagraphChoice::WR(Box::new(self.make_lang_run(text)))],
            ..Paragraph::default()
        }
    }

    /// Build a plain-text run with `<w:lang w:val="LANG"/>` in `<w:rPr>`.
    ///
    /// This is the single run-construction entry point for all text runs
    /// emitted within this serializer. Having lang on every run satisfies
    /// BC-5.01.005 PC-4 and the universality assertion in
    /// `test_BC_5_01_005_runs_have_lang_attribute_all_runs`.
    fn make_lang_run(&self, text: &str) -> Run {
        Run {
            run_properties: Some(Box::new(RunProperties {
                languages: Some(Languages {
                    val: Some(self.lang.clone()),
                    ..Languages::default()
                }),
                ..RunProperties::default()
            })),
            run_choice: vec![RunChoice::WT(Box::new(make_text(text)))],
            ..Run::default()
        }
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
            InlineNode::Plain(text) => Ok(vec![ParagraphChoice::WR(Box::new(
                self.make_lang_run(text),
            ))]),

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
                        languages: Some(Languages {
                            val: Some(self.lang.clone()),
                            ..Languages::default()
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
                // SEC-001 (defense-in-depth): validate the URL scheme at the
                // exporter layer. The parse-layer allowlist (E-PAR-022) covers
                // DSL-sourced input; programmatic InlineNode::Link callers
                // bypass the parser and reach us directly.
                let scheme = extract_url_scheme(url);
                if !ALLOWED_LINK_SCHEMES.contains(&scheme.as_str()) {
                    return Err(ExportError::ValidationError {
                        message: format!(
                            "link URL scheme '{scheme}' is not permitted in DOCX export \
                             (SEC-001 / CWE-601). Allowed schemes: http, https, mailto. \
                             URL: {url}"
                        ),
                    });
                }

                // Record the hyperlink relationship.
                let r_id = format!("rId{}", self.next_rel_id);
                self.next_rel_id += 1;
                self.relationships.push(HyperlinkRel {
                    r_id: r_id.clone(),
                    target: url.to_string(),
                });

                // Build the hyperlink choices by recursing display-text children
                // through the structured run builder (F-P17-001 / BC-3.05.001 PC-3).
                let hyperlink_choices = self.build_hyperlink_display_runs(text)?;

                let hyperlink = Hyperlink {
                    id: Some(r_id),
                    hyperlink_choice: hyperlink_choices,
                    ..Hyperlink::default()
                };

                Ok(vec![ParagraphChoice::WHyperlink(Box::new(hyperlink))])
            },

            // AC-003 STORY-081: Highlight → `<w:highlight w:val="yellow"/>` using
            // ooxmlsdk typed builders (NOT a plain text fallback).
            InlineNode::Highlight(children) => {
                let mut choices = Vec::new();
                for child in children {
                    let child_choices = self.inline_node_to_paragraph_choices(child)?;
                    for choice in child_choices {
                        // Apply highlight to each WR run produced by children.
                        let highlighted = apply_run_property(choice, |rpr| {
                            rpr.highlight = Some(Highlight {
                                val: HighlightColorValues::Yellow,
                            });
                        });
                        choices.push(highlighted);
                    }
                }
                Ok(choices)
            },
            // For unsupported inline variants (Math, Footnote, Xref),
            // fall back to plain text extraction (BC-3.05.001 PC-3 degraded path).
            InlineNode::Math(math_node) => Ok(vec![ParagraphChoice::WR(Box::new(
                self.make_lang_run(math_node.latex.as_ref()),
            ))]),
            InlineNode::Footnote(children) => {
                let text = collect_plain_text(children);
                Ok(vec![ParagraphChoice::WR(Box::new(
                    self.make_lang_run(&text),
                ))])
            },
            InlineNode::Xref(target) => Ok(vec![ParagraphChoice::WR(Box::new(
                self.make_lang_run(target.as_ref()),
            ))]),
        }
    }

    /// Build `HyperlinkChoice` entries for the display-text children of a
    /// [`InlineNode::Link`] node.
    ///
    /// Recurses each child through [`Self::inline_node_to_paragraph_choices`] so
    /// that formatting INSIDE the link display text (e.g. `[**here**](url)`)
    /// produces structured runs with run-properties (`<w:b/>`, `<w:i/>`, …) rather
    /// than being silently dropped by a plain-text flatten.
    ///
    /// - `WR` run results: receive the Hyperlink character style (if not already
    ///   styled) and become `HyperlinkChoice::WR`.
    /// - `WHyperlink` results: flattened into individual `WR` runs to avoid
    ///   nested `<w:hyperlink>` (invalid OOXML).
    /// - Empty display text: a single empty run with the Hyperlink style is
    ///   emitted to keep the element structurally valid.
    ///
    /// This is extracted from the `Link` arm of `inline_node_to_paragraph_choices`
    /// purely to keep that function under the `clippy::too_many_lines` limit.
    fn build_hyperlink_display_runs(
        &mut self,
        display_text: &[InlineNode],
    ) -> Result<Vec<HyperlinkChoice>, ExportError> {
        // F-P17-001 / BC-3.05.001 PC-3: formatting-inside-link-display-text must
        // not be silently dropped. Recurse through `inline_node_to_paragraph_choices`
        // and wrap the resulting WR runs inside `<w:hyperlink>` as HyperlinkChoice::WR.
        // Each run also gets the Hyperlink character style injected so the link text
        // appears blue+underlined even when inner nodes have their own formatting.
        let mut choices: Vec<HyperlinkChoice> = Vec::new();
        for child in display_text {
            let child_choices = self.inline_node_to_paragraph_choices(child)?;
            for choice in child_choices {
                match choice {
                    ParagraphChoice::WR(mut run) => {
                        let rpr = run
                            .run_properties
                            .get_or_insert_with(|| Box::new(RunProperties::default()));
                        if rpr.run_style.is_none() {
                            rpr.run_style = Some(
                                ooxmlsdk::schemas::schemas_openxmlformats_org_wordprocessingml_2006_main::RunStyle {
                                    val: "Hyperlink".to_owned(),
                                },
                            );
                        }
                        choices.push(HyperlinkChoice::WR(run));
                    },
                    ParagraphChoice::WHyperlink(inner_hl) => {
                        // Nested hyperlink (Link inside link display text): flatten WR
                        // runs to avoid invalid nested <w:hyperlink> elements.
                        for inner_choice in inner_hl.hyperlink_choice {
                            if let HyperlinkChoice::WR(run) = inner_choice {
                                choices.push(HyperlinkChoice::WR(run));
                            }
                        }
                    },
                    // Other ParagraphChoice variants are not produced by
                    // inline_node_to_paragraph_choices for any current InlineNode
                    // variant — drop defensively.
                    _ => {},
                }
            }
        }

        // If empty display text, emit a single empty run to keep the element valid.
        if choices.is_empty() {
            let empty_run = Run {
                run_properties: Some(Box::new(RunProperties {
                    run_style: Some(
                        ooxmlsdk::schemas::schemas_openxmlformats_org_wordprocessingml_2006_main::RunStyle {
                            val: "Hyperlink".to_owned(),
                        },
                    ),
                    languages: Some(Languages {
                        val: Some(self.lang.clone()),
                        ..Languages::default()
                    }),
                    ..RunProperties::default()
                })),
                run_choice: vec![RunChoice::WT(Box::new(make_text("")))],
                ..Run::default()
            };
            choices.push(HyperlinkChoice::WR(Box::new(empty_run)));
        }

        Ok(choices)
    }
}

impl Default for DocumentBodySerializer {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Apply a run-property mutation to a [`ParagraphChoice`].
///
/// - For `WR` variants: mutates the run's `<w:rPr>` directly.
/// - For `WHyperlink` variants: applies the property to every `WR` run INSIDE
///   the hyperlink's `hyperlink_choice` list. This preserves clickability (the
///   `r:id` relationship wiring on the `<w:hyperlink>` element stays intact)
///   while ensuring that formatting applied to the hyperlink wrapper — e.g.
///   `Bold([Link{...}])` → `<w:b/>` inside `<w:hyperlink>` — is not silently
///   discarded. (F-P13-001 fix.)
fn apply_run_property<F>(choice: ParagraphChoice, f: F) -> ParagraphChoice
where
    F: Fn(&mut RunProperties),
{
    match choice {
        ParagraphChoice::WR(mut run) => {
            let rpr = run
                .run_properties
                .get_or_insert_with(|| Box::new(RunProperties::default()));
            f(rpr);
            ParagraphChoice::WR(run)
        },
        ParagraphChoice::WHyperlink(mut hyperlink) => {
            // Apply the property to every WR run inside the hyperlink so that
            // formatting wrappers (Bold, Italic, etc.) are not silently dropped
            // when they wrap a Link node. The hyperlink's r:id (clickability)
            // remains on the <w:hyperlink> element and is unaffected.
            for hc in &mut hyperlink.hyperlink_choice {
                if let HyperlinkChoice::WR(run) = hc {
                    let rpr = run
                        .run_properties
                        .get_or_insert_with(|| Box::new(RunProperties::default()));
                    f(rpr);
                }
            }
            ParagraphChoice::WHyperlink(hyperlink)
        },
        // Other ParagraphChoice variants (WBookmarkStart, WBookmarkEnd, etc.) are
        // not produced by inline_node_to_paragraph_choices and pass through unchanged.
        other => other,
    }
}

/// Build an empty paragraph (no runs, no style).
fn make_empty_paragraph() -> Paragraph {
    Paragraph::default()
}

/// Build a shaded Normal paragraph for the DOCX gradient solid fallback (STORY-072 AC-004).
///
/// DOCX does not support shape gradient fills natively. Each gradient shape is
/// downgraded to a solid `<w:shd w:val="clear" w:fill="RRGGBB"/>` paragraph
/// using the `from` color. A [`tracing::warn!`] is also emitted.
fn make_gradient_solid_fallback_paragraph(from: slideforge_layout::types::Rgb) -> Paragraph {
    tracing::warn!(
        "DOCX gradient fill downgraded to solid \
         (DOCX does not support shape gradient fills)"
    );
    let hex_fill = format!("{:02X}{:02X}{:02X}", from.r, from.g, from.b);
    let shading = Shading {
        val: ShadingPatternValues::Clear,
        color: None,
        theme_color: None,
        theme_tint: None,
        theme_shade: None,
        fill: Some(hex_fill),
        theme_fill: None,
        theme_fill_tint: None,
        theme_fill_shade: None,
    };
    let ppr = ParagraphProperties {
        paragraph_style_id: Some(ParagraphStyleId {
            val: "Normal".into(),
        }),
        shading: Some(shading),
        ..ParagraphProperties::default()
    };
    Paragraph {
        paragraph_properties: Some(Box::new(ppr)),
        paragraph_choice: vec![],
        ..Paragraph::default()
    }
}

/// Build a `<w:t>` element with `xml:space="preserve"` when the text has
/// leading or trailing whitespace (per STORY-041 requirement).
///
/// SEC-002 / CWE-116: XML-1.0-invalid control characters are stripped before
/// the text is stored. ooxmlsdk serializes `xml_content` directly as element
/// text, so the stripping must happen here rather than at the escape layer.
fn make_text(text: &str) -> Text {
    // Strip XML-1.0-invalid control chars (SEC-002).
    let sanitized = strip_xml10_invalid_chars(text);

    let needs_preserve = sanitized.starts_with(' ')
        || sanitized.ends_with(' ')
        || sanitized.starts_with('\t')
        || sanitized.ends_with('\t');

    Text {
        xml_content: Some(sanitized),
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
