//! PDF/UA-1 structure tree generation via `krilla::tagging`.
//!
//! [`SlideTagEngine`] maps the semantic content of a [`LaidOutSlide`] to a
//! PDF structure tree using `krilla`'s tagging module. The resulting
//! [`krilla::tagging::TagTree`] is attached to the `Document` via
//! `Document::set_tag_tree` before `document.finish()` is called.
//!
//! ## Tag hierarchy (BC-4.03.002 AC-003)
//!
//! ```text
//! Document
//!   Part          ← one per slide
//!     H1 / H2 / P / L+LI+LBody  ← per text element's semantic role
//!     Figure (+ /Alt)  ← for image / diagram / chart frames
//!     Table → TR → TH/TD  ← for table frames
//! ```
//!
//! Decorative elements (`AltText::Decorative`) are NOT wrapped in a `Tag` group —
//! they will be marked as PDF Artifacts by the exporter during content drawing.
//!
//! ## Actual krilla 0.6.0 API (verified against local source)
//!
//! `TagKind` is the variant-bearing enum. Each variant wraps a `Tag<kind::*>`
//! typed value. The constructors are either `const` associated items
//! (for tags with no required args: `Tag::<kind::Part>::Part`, `Tag::<kind::P>::P`)
//! or associated fns (for tags with required args:
//! `Tag::<kind::Figure>::Figure(alt_text: Option<String>)`).
//!
//! `TagGroup::new(impl Into<TagKind>)` accepts these typed tags directly via
//! `From` impls.
//!
//! Leaf nodes in the tag tree are `Identifier` values obtained from
//! `surface.start_tagged(ContentTag)`. The tag tree built here contains
//! only group nodes during STORY-043; identifiers are inserted by `PdfExporter`
//! during the surface-drawing pass (STORY-044/STORY-045).
//!
//! ## Decorative frame handling
//!
//! `tag_slide` returns a `PartResult` which includes the `TagGroup` for tagged
//! frames AND a list of frame indices that are decorative. The exporter uses the
//! decorative list to emit `ContentTag::Artifact(ArtifactType::Other)` markers
//! during the drawing pass.

use krilla::tagging::{ListNumbering, TableHeaderScope, Tag, TagGroup, TagTree};
use slideforge_layout::LaidOutSlide;
use slideforge_layout::types::FrameContent;
use slideforge_types::{AltText, ContentBlock};

/// `NonZeroU16` value `2` for H2 headings, resolved at compile time via
/// `match`. The `None` arm is statically unreachable because the literal `2`
/// is non-zero; the compiler proves this during const evaluation.
const H2_LEVEL: std::num::NonZeroU16 = match std::num::NonZeroU16::new(2) {
    Some(v) => v,
    None => unreachable!(),
};

use crate::error::PdfExportError;

/// Result of tagging a single slide.
///
/// Contains the structural `TagGroup` for the slide (a `Part`) plus the set
/// of frame indices that are decorative artifacts (to be excluded from the tag
/// tree and emitted as PDF Artifacts during drawing).
pub struct PartResult {
    /// The `Part` tag group wrapping all tagged content on this slide.
    ///
    /// After drawing, the exporter inserts `Identifier` leaf nodes into the
    /// children of this group (one per non-decorative, non-empty content block)
    /// via `frame_child_part_indices`. This links the structure tree elements to
    /// the marked-content sequences in the PDF content stream (MCID linkage,
    /// BC-4.03.001 F-045-C2).
    pub part: TagGroup,
    /// Frame indices (into `LaidOutSlide::frames`) that are decorative.
    ///
    /// During the drawing pass, the exporter marks content at these indices
    /// as `ContentTag::Artifact(ArtifactType::Other)` instead of linking them
    /// to a tag group identifier.
    pub decorative_frame_indices: Vec<usize>,
    /// Per-frame, per-block child indices into `part.children` for MCID linkage.
    ///
    /// `frame_child_part_indices[frame_idx]` is:
    /// - `None` for empty or decorative frames (no tagged groups in the Part).
    /// - `Some(vec![child_idx])` for single-block frames (`Title`, `Subtitle`,
    ///   `TextRun`, `Image` with alt, `Shape` with alt, `ErrorSlidePlaceholder`): one
    ///   child index pointing to the single group pushed into `part.children`.
    /// - `Some(vec![idx_0, idx_1, ..., idx_N])` for Body frames with N content
    ///   blocks that each push a group: each element is the `part.children`
    ///   index for the corresponding block's structure group.
    ///
    /// During the drawing pass, the exporter iterates the Vec and opens one
    /// `start_tagged(ContentTag::Other)` region per block, inserting the
    /// resulting `Identifier` into `part.children[idx_k]`. This ensures every
    /// structure group (P, L/LI/LBody, Table, Figure) has at least one MCID
    /// leaf — satisfying PDF/UA-1's requirement that grouping elements reference
    /// actual marked content (F-045-C2 / BC-4.03.001).
    pub frame_child_part_indices: Vec<Option<Vec<usize>>>,
}

/// Engine that maps a [`LaidOutSlide`] to a krilla [`TagTree`].
///
/// One `SlideTagEngine` instance is created per export pass. It is stateless
/// between slides: each call to [`SlideTagEngine::tag_slide`] produces an
/// independent sub-tree that is assembled into the deck-level `TagTree` by the
/// [`crate::exporter::PdfExporter`].
pub struct SlideTagEngine;

impl SlideTagEngine {
    /// Construct a new `SlideTagEngine`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Build the PDF structure tag tree for a single slide.
    ///
    /// Returns a [`PartResult`] containing:
    /// - The `Part` `TagGroup` for this slide, populated with structural groups
    ///   for each non-decorative, non-empty frame.
    /// - The list of frame indices that are decorative (to be marked as
    ///   PDF Artifacts during the drawing pass in STORY-044/STORY-045).
    ///
    /// ## Tag hierarchy produced
    ///
    /// ```text
    /// Part          ← one group wrapping this slide's content
    ///   H1          ← for Title frames
    ///   H2          ← for Subtitle frames
    ///   P           ← for body paragraph blocks and TextRun frames
    ///   L (Disc)    ← for body bullet-list blocks, with LI+LBody children
    ///   Figure+Alt  ← for Image, Diagram, Chart frames (alt from frame content)
    ///   Table→TR→TH/TD ← for Table frames
    ///   P           ← for Shape, ErrorSlidePlaceholder, and unknown frames
    /// ```
    ///
    /// Decorative frames (frames containing `FrameContent::Image { alt: AltText::Decorative }`
    /// or `FrameContent::Shape` with `AltText::Decorative`)
    /// are NOT added to the Part group.
    ///
    /// Empty frames (`FrameContent::Empty`) are skipped.
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError::Serialize`] if the tag tree cannot be
    /// constructed.
    ///
    /// # Panics
    ///
    /// Does not panic. The `H2_LEVEL` constant is evaluated at compile time
    /// using a `match` expression that proves `NonZeroU16::new(2)` is `Some(_)`
    /// at compile time — no runtime panic path exists.
    pub fn tag_slide(&self, slide: &LaidOutSlide) -> Result<PartResult, PdfExportError> {
        self.tag_slide_with_title(slide, None)
    }

    /// Build the PDF structure tag tree for a single slide, with an explicit
    /// heading title string for PDF/UA-1 compliance.
    ///
    /// `slide_title` is the text to attach as the `/T` (Title) attribute on any
    /// `Hn` structure element produced for this slide. It is sourced from
    /// `deck.slides[slide.source_index].title_str()` in the exporter, with
    /// `"Slide N"` as the fallback when `title_str()` returns `None`.
    ///
    /// Per ISO 14289-1 §7.1: every `H1`–`H6` structure element must carry a
    /// `/Title` attribute (`/T` key in the `StructElem` dictionary). krilla's
    /// `Validator::UA1` reports `MissingHeadingTitle` when this attribute is
    /// absent or empty.
    ///
    /// When `slide_title` is `None` the behaviour is identical to
    /// [`tag_slide`](Self::tag_slide) — the `Hn` tag is built without a `/T`
    /// attribute. Callers that do not have access to the deck's semantic layer
    /// (e.g., unit tests operating on `LaidOutSlide` directly) should use
    /// [`tag_slide`](Self::tag_slide) or pass `None` explicitly.
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError::Serialize`] if the tag tree cannot be
    /// constructed.
    pub fn tag_slide_with_title(
        &self,
        slide: &LaidOutSlide,
        slide_title: Option<&str>,
    ) -> Result<PartResult, PdfExportError> {
        // One Part group per slide — wraps all structural children.
        let mut part_group = TagGroup::new(Tag::<krilla::tagging::kind::Part>::Part);
        let mut decorative_frame_indices: Vec<usize> = Vec::new();
        // frame_child_part_indices[frame_idx] = Some(child_idx in part_group.children)
        // when frame_idx contributes at least one child to the Part, None otherwise.
        // This is used by the exporter for F-045-C2 MCID linkage: after drawing each
        // non-decorative frame via `surface.start_tagged(ContentTag::Other)`, the
        // returned `Identifier` is inserted as a leaf node into `part_group.children[child_idx]`.
        let mut frame_child_part_indices: Vec<Option<Vec<usize>>> = vec![None; slide.frames.len()];

        for (frame_idx, frame) in slide.frames.iter().enumerate() {
            match &frame.content {
                // ── Empty frames: skip (no tag, not an artifact) ──────────────
                FrameContent::Empty => {
                    // Intentionally omitted from the tag tree. An empty frame
                    // has no renderable content and no semantic role.
                },

                // ── Title → H1 ───────────────────────────────────────────────
                //
                // AC-011 (BC-4.03.001): ISO 14289-1 §7.1 requires every Hn structure
                // element to carry a /Title attribute (/T key in the StructElem dictionary).
                // The title text is sourced from `deck.slides[source_index].title_str()` by
                // the exporter and passed as `slide_title`. When `slide_title` is None
                // (e.g., unit tests that do not have deck access), Hn is built without /T.
                FrameContent::Title(_text) => {
                    let child_idx = part_group.children.len();
                    let heading_group = TagGroup::new(Tag::<krilla::tagging::kind::Hn>::Hn(
                        // NonZeroU16::MIN == 1 (H1); infallible construction.
                        std::num::NonZeroU16::MIN,
                        slide_title.map(std::borrow::ToOwned::to_owned),
                    ));
                    part_group.push(heading_group);
                    frame_child_part_indices[frame_idx] = Some(vec![child_idx]);
                },

                // ── Subtitle → H2 ────────────────────────────────────────────
                //
                // OBS-012 / AC-011: body H2-H6 /Title attribute values must be the
                // first inline run of that frame — NOT the slide title re-used.
                //
                // Using the slide_title here was incorrect: a subtitle frame such as
                // "Q4 2025 Highlights" must carry its OWN text as the H2 /Title, not
                // the parent slide's H1 title.
                //
                // The subtitle text is the first (and typically only) inline run of
                // the frame; `subtitle_text` is the `Arc<str>` stored in the variant.
                FrameContent::Subtitle(subtitle_text) => {
                    let child_idx = part_group.children.len();
                    let heading_group = TagGroup::new(Tag::<krilla::tagging::kind::Hn>::Hn(
                        H2_LEVEL,
                        Some(subtitle_text.as_ref().to_owned()),
                    ));
                    part_group.push(heading_group);
                    frame_child_part_indices[frame_idx] = Some(vec![child_idx]);
                },

                // ── Body content → P per paragraph or L+LI+LBody per list ────
                //
                // Body frames may push MULTIPLE children — one per content block.
                //
                // F-P3-001 fix: we record a Vec of per-block child indices so the
                // exporter can open an independent `start_tagged` / `end_tagged`
                // region for EACH block and insert its Identifier into the correct
                // structure group. Recording only the first child index left the
                // L/LI/LBody group (and any 2nd+ block) with no MCID leaf —
                // a grouping element referencing no marked content, rejected by
                // veraPDF --flavour ua1.
                FrameContent::Body(content_blocks) => {
                    if content_blocks.is_empty() {
                        // No blocks — emit a single generic P so Part remains non-empty.
                        // Single-element Vec keeps the MCID linkage contract uniform.
                        let child_idx = part_group.children.len();
                        part_group.push(TagGroup::new(Tag::<krilla::tagging::kind::P>::P));
                        frame_child_part_indices[frame_idx] = Some(vec![child_idx]);
                    } else {
                        let mut block_child_indices: Vec<usize> = Vec::new();
                        for block in content_blocks {
                            let group = self.tag_content_block(block)?;
                            if let Some(g) = group {
                                let child_idx = part_group.children.len();
                                part_group.push(g);
                                block_child_indices.push(child_idx);
                            }
                        }
                        if !block_child_indices.is_empty() {
                            frame_child_part_indices[frame_idx] = Some(block_child_indices);
                        }
                        // If all blocks produced None (e.g., all decorative shapes),
                        // frame_child_part_indices[frame_idx] stays None — the exporter
                        // draws the frame without tagging (correct: nothing was emitted).
                    }
                },

                // ── Image → Figure+Alt (or Artifact if decorative/unspecified) ──
                FrameContent::Image { alt } => {
                    match alt {
                        slideforge_types::AltText::Decorative => {
                            // Decorative image: mark as Artifact.
                            decorative_frame_indices.push(frame_idx);
                        },
                        // STORY-086 stub: Unspecified = pipeline placeholder; treat as
                        // Artifact (same as Decorative) until Stage 2b threads real alt.
                        slideforge_types::AltText::Unspecified => {
                            decorative_frame_indices.push(frame_idx);
                        },
                        slideforge_types::AltText::Provided(alt_str) => {
                            let child_idx = part_group.children.len();
                            part_group.push(self.tag_figure(Some(alt_str.as_ref()))?);
                            frame_child_part_indices[frame_idx] = Some(vec![child_idx]);
                        },
                    }
                },

                // ── Diagram / Chart → Figure+Alt (or Artifact if decorative/unspecified) ──
                //
                // STORY-039: both `FrameContent::Diagram` and `FrameContent::Chart`
                // carry `alt: AltText`. Same branching logic:
                // - `AltText::Provided(s)` → tagged Figure with /Alt (BC-4.03.001 AC-004).
                // - `AltText::Decorative` → Artifact (explicit opt-out).
                // - `AltText::Unspecified` → Artifact (pipeline placeholder — STORY-086).
                FrameContent::Diagram { alt, .. } | FrameContent::Chart { alt } => match alt {
                    slideforge_types::AltText::Provided(alt_str) => {
                        let child_idx = part_group.children.len();
                        part_group.push(self.tag_figure(Some(alt_str.as_ref()))?);
                        frame_child_part_indices[frame_idx] = Some(vec![child_idx]);
                    },
                    // Decorative = author opt-out; Unspecified = pipeline placeholder (no alt).
                    // Both are marked as PDF Artifacts — not in the structure tree.
                    // The post-layout validator fires E-A11-001 for Unspecified in strict mode.
                    slideforge_types::AltText::Decorative
                    | slideforge_types::AltText::Unspecified => {
                        decorative_frame_indices.push(frame_idx);
                    },
                },

                // ── Shape → Figure+Alt or Artifact if decorative/unspecified ──
                FrameContent::Shape(shape_frame) => {
                    match &shape_frame.alt {
                        // Decorative or Unspecified shape: not in the tag tree — mark as Artifact.
                        // Unspecified = no author alt data; Decorative = author opt-out.
                        slideforge_types::AltText::Decorative
                        | slideforge_types::AltText::Unspecified => {
                            decorative_frame_indices.push(frame_idx);
                        },
                        slideforge_types::AltText::Provided(alt) => {
                            let child_idx = part_group.children.len();
                            part_group.push(self.tag_figure(Some(alt))?);
                            frame_child_part_indices[frame_idx] = Some(vec![child_idx]);
                        },
                    }
                },

                // ── TextRun → P ───────────────────────────────────────────────
                // STORY-086 / ADR-019: Stage 2b populates ContentBlock::Text blocks
                // whose inlines are threaded into TextRun frames. Set /ActualText on
                // the P structure element so the content is accessible in the structure
                // tree (PDF/UA-1 compliance; also makes text findable in raw PDF bytes
                // for integration test verification, as structure dictionaries are
                // stored uncompressed even when content streams use FlateDecode).
                FrameContent::TextRun(inlines) => {
                    let child_idx = part_group.children.len();
                    let actual_text: String =
                        inlines.iter().fold(String::new(), |mut acc, node| {
                            if let slideforge_types::InlineNode::Plain(s) = node {
                                acc.push_str(s.as_ref());
                            }
                            acc
                        });
                    let mut p_group = TagGroup::new(Tag::<krilla::tagging::kind::P>::P);
                    if !actual_text.is_empty() {
                        p_group.tag.set_actual_text(Some(actual_text));
                    }
                    part_group.push(p_group);
                    frame_child_part_indices[frame_idx] = Some(vec![child_idx]);
                },

                // ── ErrorSlidePlaceholder → P (error text is readable) ────────
                FrameContent::ErrorSlidePlaceholder { .. } => {
                    let child_idx = part_group.children.len();
                    let p_group = TagGroup::new(Tag::<krilla::tagging::kind::P>::P);
                    part_group.push(p_group);
                    frame_child_part_indices[frame_idx] = Some(vec![child_idx]);
                },
            }
        }

        Ok(PartResult {
            part: part_group,
            decorative_frame_indices,
            frame_child_part_indices,
        })
    }

    /// Build a tag group for a single [`ContentBlock`] from a Body frame.
    ///
    /// Returns:
    /// - `Ok(Some(TagGroup))` for structured content (P, L+LI+LBody, Table, Figure).
    /// - `Ok(None)` for blocks that do not produce a tag (e.g., decorative Shape).
    ///
    /// ## OBS-P5-001 — Single source of truth
    ///
    /// The boolean "does this block produce a structure group?" decision is
    /// delegated to [`slideforge_types::ContentBlock::produces_structure_group`],
    /// which is the single authoritative predicate shared with
    /// `slideforge_pdf::exporter::block_is_structure_producing` (the draw loop).
    ///
    /// This function retains the **`TagKind` mapping** (P / L / Table / Figure) —
    /// only the boolean gating is unified. If `produces_structure_group()` returns
    /// `false`, this function immediately returns `Ok(None)` without inspecting
    /// the variant further. If it returns `true`, the variant-specific match arms
    /// build the correct `TagGroup` structure.
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError::Serialize`] if the tag group cannot be built.
    fn tag_content_block(
        &self,
        block: &slideforge_types::ContentBlock,
    ) -> Result<Option<TagGroup>, PdfExportError> {
        // OBS-P5-001: delegate the boolean "structure-producing?" decision to the
        // single authoritative predicate on ContentBlock. This ensures lockstep
        // with `block_is_structure_producing` in the draw loop — both call the
        // same function from the same single exhaustive match in slideforge-types.
        if !block.produces_structure_group() {
            return Ok(None);
        }

        match block {
            // Text paragraph → P, with /ActualText attribute.
            //
            // /ActualText is set so that the paragraph text is stored in the uncompressed
            // structure dictionary — making it findable in raw PDF bytes even when the
            // content stream uses FlateDecode compression. This is consistent with the
            // TextRun → P path in `tag_slide_with_title` (which sets /ActualText for
            // the same reason). PDF/UA-1 compliance benefits from /ActualText on P elements
            // when the content stream glyphs may differ from logical reading order
            // (BC-4.03.001 / STORY-086).
            ContentBlock::Text(text_block) => {
                let mut p_group = TagGroup::new(Tag::<krilla::tagging::kind::P>::P);
                let actual_text: String =
                    text_block
                        .inlines
                        .iter()
                        .fold(String::new(), |mut acc, node| {
                            if let slideforge_types::InlineNode::Plain(s) = node {
                                acc.push_str(s.as_ref());
                            }
                            acc
                        });
                if !actual_text.is_empty() {
                    p_group.tag.set_actual_text(Some(actual_text));
                }
                Ok(Some(p_group))
            },

            // Bullet list → L (Disc) with LI+LBody for each item.
            // Empty Bullets is already filtered out by produces_structure_group() above.
            ContentBlock::Bullets(items) => {
                let mut list_group =
                    TagGroup::new(Tag::<krilla::tagging::kind::L>::L(ListNumbering::Disc));
                for _item in items {
                    let mut li_group = TagGroup::new(Tag::<krilla::tagging::kind::LI>::LI);
                    let lbody_group = TagGroup::new(Tag::<krilla::tagging::kind::LBody>::LBody);
                    li_group.push(lbody_group);
                    list_group.push(li_group);
                }
                Ok(Some(list_group))
            },

            // Shape in body → Figure+Alt.
            // produces_structure_group() already guarantees alt is Provided here.
            ContentBlock::Shape(shape_spec) => match &shape_spec.alt {
                Some(AltText::Provided(alt)) => Ok(Some(self.tag_figure(Some(alt))?)),
                // Unreachable: produces_structure_group() returns false for non-Provided alt.
                Some(AltText::Decorative | AltText::Unspecified) | None => Ok(None),
            },

            // Chart in body → Figure+Alt.
            // produces_structure_group() already guarantees alt is Provided here.
            //
            // F-045-I1 (alt-lie fix): `.or(Some("chart"))` was a placeholder that
            // produced a /Figure with the generic string "chart" as alt text.
            // This is an "alt lie" — the string describes the element type, not the
            // chart content. PDF/UA-1 requires meaningful alt text.
            ContentBlock::Chart(chart_spec) => match chart_spec.alt.as_ref() {
                Some(AltText::Provided(s)) => Ok(Some(self.tag_figure(Some(s))?)),
                // Unreachable: produces_structure_group() returns false for non-Provided alt.
                Some(AltText::Decorative | AltText::Unspecified) | None => Ok(None),
            },

            // Diagram in body → Figure+Alt.
            // produces_structure_group() already guarantees alt is Provided here.
            //
            // F-045-I1 (alt-lie fix): `.or(Some("diagram"))` was a placeholder.
            // Same rationale as the Chart arm above.
            ContentBlock::Diagram(diagram_spec) => match diagram_spec.alt.as_ref() {
                Some(AltText::Provided(s)) => Ok(Some(self.tag_figure(Some(s))?)),
                // Unreachable: produces_structure_group() returns false for non-Provided alt.
                Some(AltText::Decorative | AltText::Unspecified) | None => Ok(None),
            },

            // Image in body → Figure+Alt.
            // produces_structure_group() already guarantees alt is Provided here.
            ContentBlock::Image(image_spec) => match &image_spec.alt {
                Some(AltText::Provided(alt)) => Ok(Some(self.tag_figure(Some(alt))?)),
                // Unreachable: produces_structure_group() returns false for non-Provided alt.
                Some(AltText::Decorative | AltText::Unspecified) | None => Ok(None),
            },

            // Math block → P (math content is inline text; STORY-045 will
            // use TagKind::Formula for display-mode math blocks)
            ContentBlock::Math(_math_node) => {
                Ok(Some(TagGroup::new(Tag::<krilla::tagging::kind::P>::P)))
            },

            // Table → Table → TR → TH + TD*
            ContentBlock::Table(table_spec) => {
                let table_group = self.tag_table(table_spec)?;
                Ok(Some(table_group))
            },
        }
    }

    /// Build a `Table → TR → TH/TD` tag group structure from a [`slideforge_types::TableSpec`].
    ///
    /// Produces:
    /// ```text
    /// Table
    ///   TR  ← header row (if headers non-empty)
    ///     TH (Column)  ← one per header cell
    ///   TR  ← one per data row
    ///     TD  ← one per data cell
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError::Serialize`] if the table structure cannot be built.
    pub fn tag_table(
        &self,
        table_spec: &slideforge_types::TableSpec,
    ) -> Result<TagGroup, PdfExportError> {
        let mut table_group = TagGroup::new(Tag::<krilla::tagging::kind::Table>::Table);

        // Header row: TH cells with Column scope.
        if !table_spec.headers.is_empty() {
            let mut header_tr = TagGroup::new(Tag::<krilla::tagging::kind::TR>::TR);
            for _header_cell in &table_spec.headers {
                let th_group = TagGroup::new(Tag::<krilla::tagging::kind::TH>::TH(
                    TableHeaderScope::Column,
                ));
                header_tr.push(th_group);
            }
            table_group.push(header_tr);
        }

        // Data rows.
        for row in &table_spec.rows {
            let mut data_tr = TagGroup::new(Tag::<krilla::tagging::kind::TR>::TR);
            for _cell in row {
                let td_group = TagGroup::new(Tag::<krilla::tagging::kind::TD>::TD);
                data_tr.push(td_group);
            }
            // Only push TR if it has cells.
            if !row.is_empty() {
                table_group.push(data_tr);
            }
        }

        Ok(table_group)
    }

    /// Tag a figure element with an optional `/Alt` attribute for PDF/UA-1 compliance.
    ///
    /// Returns a [`TagGroup`] using `TagKind::Figure` with the alt text set when
    /// provided. Pass `Some(alt)` for accessible figures (including diagrams, whose
    /// `DiagramSpec.alt` is now threaded through the IR by STORY-039); pass `None`
    /// only for genuinely decorative figures where no alt text is appropriate.
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError::Serialize`] if the figure tag cannot be
    /// constructed.
    pub fn tag_figure(&self, alt: Option<&str>) -> Result<TagGroup, PdfExportError> {
        let figure_tag =
            Tag::<krilla::tagging::kind::Figure>::Figure(alt.map(std::borrow::ToOwned::to_owned));
        Ok(TagGroup::new(figure_tag))
    }

    /// Produce a `Document`-level `TagTree` wrapping one `Part` group per slide.
    ///
    /// Each `part_group` in `slide_parts` corresponds to one slide. Groups that
    /// have no children (e.g., slides with only decorative/empty frames) are
    /// omitted from the assembled tree since krilla discards childless groups.
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError::Serialize`] on failure.
    pub fn assemble_deck_tag_tree(
        &self,
        slide_parts: Vec<TagGroup>,
    ) -> Result<TagTree, PdfExportError> {
        let mut tree = TagTree::new();
        for part in slide_parts {
            // Only push Part groups that have children; empty groups are
            // discarded by krilla anyway, but we skip them here for clarity.
            if !part.children.is_empty() {
                tree.push(part);
            }
        }
        Ok(tree)
    }
}

impl Default for SlideTagEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slideforge_layout::types::{BoundingBox, Frame, FrameContent, LaidOutSlide, RegisterSet};
    use slideforge_types::{Emu, SourceSpan, TableSpec};
    use std::sync::Arc;

    /// Helper: build a minimal 1-frame [`LaidOutSlide`] for tag-engine tests.
    fn minimal_slide() -> LaidOutSlide {
        LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("title"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(914_400),
                },
                content: FrameContent::Title(Arc::from("Red Gate Test Slide")),
                text_flow: None,
            }],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        }
    }

    /// BC-4.03.002 AC-003 (behavioral assertion): `tag_slide` returns a
    /// `PartResult` whose `part` contains exactly one H1 child for a Title frame.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_bc_4_03_002_tag_slide_produces_one_part_per_slide() {
        use krilla::tagging::{Node, TagKind};

        let engine = SlideTagEngine::new();
        let slide = minimal_slide();
        let result = engine.tag_slide(&slide);
        assert!(
            result.is_ok(),
            "tag_slide must succeed for a valid LaidOutSlide"
        );
        let part_result = result.unwrap();
        let part = part_result.part;

        // The Part group must have children (at least the H1 for the Title frame).
        assert!(
            !part.children.is_empty(),
            "Part group must have at least one child (the H1 for the Title frame)"
        );

        // That Part must have an Hn (H1) child.
        let first_child = &part.children[0];
        assert!(
            matches!(first_child, Node::Group(_)),
            "first child of Part must be a Group node"
        );
        if let Node::Group(group) = first_child {
            assert!(
                matches!(group.tag, TagKind::Hn(_)),
                "Title frame must produce an Hn tag group; got {:?}",
                group.tag
            );
        }
    }

    /// BC-4.03.002 AC-003 (behavioral): decorative Image frames produce
    /// a non-empty `decorative_frame_indices` list and are NOT in the Part group.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_bc_4_03_002_decorative_frame_excluded_from_tag_tree() {
        let engine = SlideTagEngine::new();
        let decorative_slide = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("photo"),
            frames: vec![
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(0),
                        width: Emu(9_144_000),
                        height: Emu(914_400),
                    },
                    // Non-decorative title frame.
                    content: FrameContent::Title(Arc::from("Photo Slide")),
                    text_flow: None,
                },
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(914_400),
                        width: Emu(9_144_000),
                        height: Emu(5_143_500),
                    },
                    // Decorative image: AltText::Decorative (STORY-039 IR reshape).
                    content: FrameContent::Image {
                        alt: slideforge_types::AltText::Decorative,
                    },
                    text_flow: None,
                },
            ],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        };

        let result = engine.tag_slide(&decorative_slide).unwrap();

        // Frame 1 (index 1, the decorative image) must be in decorative_frame_indices.
        assert!(
            result.decorative_frame_indices.contains(&1),
            "decorative image frame (index 1) must appear in decorative_frame_indices"
        );
        // The Part group must have exactly ONE child (the H1 for the title),
        // NOT two (the image must be excluded from the Part).
        assert_eq!(
            result.part.children.len(),
            1,
            "Part must contain only the H1 (title), not the decorative image"
        );
    }

    /// BC-4.03.002 AC-003 (behavioral): Figure frame with alt text gets
    /// `TagKind::Figure` with the correct alt text.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_bc_4_03_002_figure_alt_text_threaded_from_frame() {
        use krilla::tagging::{Node, TagKind};

        let engine = SlideTagEngine::new();
        let figure_slide = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("photo"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(5_143_500),
                },
                // Non-decorative image with real alt text (STORY-039 IR reshape).
                content: FrameContent::Image {
                    alt: slideforge_types::AltText::Provided(Arc::from(
                        "A mountain landscape at sunrise",
                    )),
                },
                text_flow: None,
            }],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        };

        let result = engine.tag_slide(&figure_slide).unwrap();

        // No decorative frames.
        assert!(
            result.decorative_frame_indices.is_empty(),
            "non-decorative image must not appear in decorative_frame_indices"
        );

        // The Part must have exactly one child: the Figure group.
        assert_eq!(
            result.part.children.len(),
            1,
            "Part must contain exactly one child (the Figure group)"
        );

        let first_child = &result.part.children[0];
        if let Node::Group(group) = first_child {
            assert!(
                matches!(group.tag, TagKind::Figure(_)),
                "Image frame with alt must produce TagKind::Figure; got {:?}",
                group.tag
            );
            // Verify the alt text is present.
            let alt = group.tag.alt_text();
            assert_eq!(
                alt,
                Some("A mountain landscape at sunrise"),
                "Figure alt text must match the frame alt field"
            );
        } else {
            panic!("expected Group node for Figure, got Leaf");
        }
    }

    /// BC-4.03.002 AC-003 (structural): `tag_table` builds Table → TR → TH/TD.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_bc_4_03_002_tag_table_builds_table_tr_th_td() {
        use krilla::tagging::{Node, TagKind};

        let engine = SlideTagEngine::new();
        let table_spec = TableSpec {
            headers: vec![Arc::from("Name"), Arc::from("Value")],
            rows: vec![
                vec![Arc::from("Alice"), Arc::from("42")],
                vec![Arc::from("Bob"), Arc::from("99")],
            ],
            alt: None,
            span: SourceSpan::default(),
        };

        let table_group = engine.tag_table(&table_spec).unwrap();

        // Top-level must be a Table.
        assert!(
            matches!(table_group.tag, TagKind::Table(_)),
            "tag_table must produce a Table group"
        );

        // Must have 3 TR children: 1 header row + 2 data rows.
        assert_eq!(
            table_group.children.len(),
            3,
            "Table must have 3 TR children (1 header + 2 data)"
        );

        // First TR must have 2 TH children.
        if let Node::Group(header_tr) = &table_group.children[0] {
            assert!(
                matches!(header_tr.tag, TagKind::TR(_)),
                "first child of Table must be TR"
            );
            assert_eq!(
                header_tr.children.len(),
                2,
                "header TR must have 2 TH cells"
            );
            for th_node in &header_tr.children {
                if let Node::Group(th) = th_node {
                    assert!(
                        matches!(th.tag, TagKind::TH(_)),
                        "header cells must be TH; got {:?}",
                        th.tag
                    );
                }
            }
        }

        // Second TR must have 2 TD children.
        if let Node::Group(data_tr) = &table_group.children[1] {
            assert!(
                matches!(data_tr.tag, TagKind::TR(_)),
                "data rows must be TR"
            );
            assert_eq!(data_tr.children.len(), 2, "data TR must have 2 TD cells");
            for td_node in &data_tr.children {
                if let Node::Group(td) = td_node {
                    assert!(
                        matches!(td.tag, TagKind::TD(_)),
                        "data cells must be TD; got {:?}",
                        td.tag
                    );
                }
            }
        }
    }

    /// BC-4.03.002 AC-003 (structural): `assemble_deck_tag_tree` produces a
    /// `TagTree` with one child per slide that has non-empty content.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_bc_4_03_002_assemble_deck_tag_tree_one_part_per_slide() {
        use krilla::tagging::Node;

        let engine = SlideTagEngine::new();
        let slide = minimal_slide();

        // Tag two copies of the same slide.
        let part1 = engine.tag_slide(&slide).unwrap().part;
        let part2 = engine.tag_slide(&slide).unwrap().part;

        let tree = engine.assemble_deck_tag_tree(vec![part1, part2]).unwrap();

        assert_eq!(
            tree.children.len(),
            2,
            "deck tag tree must contain one Part per slide"
        );
        for child in &tree.children {
            assert!(
                matches!(child, Node::Group(_)),
                "each deck-level tag tree child must be a Group node"
            );
        }
    }

    /// BC-4.03.002 AC-003 (guard): an empty `ContentBlock::Bullets` list must
    /// NOT produce a childless `L` group — `tag_content_block` must return
    /// `Ok(None)` for an empty bullet list.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_bc_4_03_002_empty_bullets_produces_no_tag_group() {
        let engine = SlideTagEngine::new();
        let result = engine
            .tag_content_block(&slideforge_types::ContentBlock::Bullets(vec![]))
            .unwrap();
        assert!(
            result.is_none(),
            "empty Bullets([]) must return Ok(None), not a childless L group"
        );
    }

    /// F-P3-001 (multi-block Body MCID linkage): a Body frame with TWO content blocks
    /// (Text + Bullets) must record a SEPARATE Part child index for EACH block.
    ///
    /// ## What this drives
    ///
    /// `frame_child_part_indices[body_frame_idx]` must be `Some(vec![p_idx, l_idx])`
    /// — one index per block — so that the exporter can open an independent
    /// `start_tagged` / `end_tagged` region for each block and insert the resulting
    /// `Identifier` into the correct group.  Recording only the first-block index
    /// leaves the L/LI/LBody group with no linked marked content, which veraPDF
    /// UA-1 rejects.
    ///
    /// The assertion is against `frame_child_part_indices`, not byte-substring
    /// presence, so it exercises the tag-tree API directly (TD-VSDD-059
    /// load-bearing assertion requirement).
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_f_p3_001_multi_block_body_records_per_block_child_indices() {
        use krilla::tagging::{Node, TagKind};
        use slideforge_types::{BulletItem, ContentBlock, InlineNode, SourceSpan, TextBlock, TextTag};

        let engine = SlideTagEngine::new();

        // Slide: one Title frame (frame 0) + one Body frame (frame 1) with
        // a Text block AND a Bullets block — two distinct content blocks.
        let slide = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("content"),
            frames: vec![
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(0),
                        width: Emu(9_144_000),
                        height: Emu(914_400),
                    },
                    content: FrameContent::Title(Arc::from("Multi-Block Body Test")),
                    text_flow: None,
                },
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(914_400),
                        width: Emu(9_144_000),
                        height: Emu(3_657_600),
                    },
                    content: FrameContent::Body(vec![
                        ContentBlock::Text(TextBlock {
                            inlines: vec![InlineNode::Plain(Arc::from("Paragraph text"))],
                            tag: TextTag::Untagged,
                            span: SourceSpan::default(),
                        }),
                        ContentBlock::Bullets(vec![BulletItem {
                            inlines: vec![InlineNode::Plain(Arc::from("Bullet item"))],
                            children: vec![],
                            span: SourceSpan::default(),
                        }]),
                    ]),
                    text_flow: None,
                },
            ],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        };

        let result = engine.tag_slide(&slide).unwrap();

        // frame 0 (Title → H1) must have exactly one child index.
        assert_eq!(
            result.frame_child_part_indices.len(),
            2,
            "frame_child_part_indices must have one entry per frame"
        );

        // Title frame (0): single-block mapping — one child index.
        let title_indices = result.frame_child_part_indices[0].as_ref().unwrap();
        assert_eq!(
            title_indices.len(),
            1,
            "Title frame must record exactly 1 child index; got {}",
            title_indices.len()
        );
        let title_child_idx = title_indices[0];

        // Body frame (1): MULTI-block mapping — TWO child indices, one per block.
        let body_indices = result.frame_child_part_indices[1]
            .as_ref()
            .expect("Body frame must record Some(Vec) of child indices, not None");
        assert_eq!(
            body_indices.len(),
            2,
            "Body frame with Text+Bullets must record 2 child indices (one per block); got {}",
            body_indices.len()
        );

        let p_child_idx = body_indices[0]; // Text block → P group
        let l_child_idx = body_indices[1]; // Bullets block → L group

        // The two indices must be DISTINCT — each block occupies a separate
        // position in part.children (otherwise they'd share the same group,
        // leaving one group with no Identifier).
        assert_ne!(
            p_child_idx, l_child_idx,
            "each body block must occupy a DISTINCT Part child index; \
             both were {p_child_idx} — the second block has no MCID linkage slot"
        );

        // The total number of Part children must equal Title(1) + P(1) + L(1) = 3.
        // (title_child_idx + two body block children)
        assert_eq!(
            result.part.children.len(),
            3,
            "Part must have 3 children: H1 (title) + P (text block) + L (bullets block); \
             got {}",
            result.part.children.len()
        );

        // Verify the tag kinds at the recorded positions:
        // title_child_idx → Hn
        if let Node::Group(g) = &result.part.children[title_child_idx] {
            assert!(
                matches!(g.tag, TagKind::Hn(_)),
                "title child must be Hn; got {:?}",
                g.tag
            );
        } else {
            panic!("title child must be a Group node");
        }
        // p_child_idx → P
        if let Node::Group(g) = &result.part.children[p_child_idx] {
            assert!(
                matches!(g.tag, TagKind::P(_)),
                "text block child must be P; got {:?}",
                g.tag
            );
        } else {
            panic!("text block child must be a Group node");
        }
        // l_child_idx → L
        if let Node::Group(g) = &result.part.children[l_child_idx] {
            assert!(
                matches!(g.tag, TagKind::L(_)),
                "bullets block child must be L; got {:?}",
                g.tag
            );
        } else {
            panic!("bullets block child must be a Group node");
        }
    }

    /// F-P3-001 (single-block Body backward compatibility): a Body frame with a
    /// SINGLE content block must continue to work — `frame_child_part_indices[frame]`
    /// is `Some(vec![idx])` with exactly one element, and the Part has the correct
    /// single-child group.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_f_p3_001_single_block_body_backward_compat() {
        use krilla::tagging::{Node, TagKind};
        use slideforge_types::{ContentBlock, InlineNode, SourceSpan, TextBlock, TextTag};

        let engine = SlideTagEngine::new();

        let slide = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("content"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(914_400),
                },
                content: FrameContent::Body(vec![ContentBlock::Text(TextBlock {
                    inlines: vec![InlineNode::Plain(Arc::from("Single paragraph"))],
                    tag: TextTag::Untagged,
                    span: SourceSpan::default(),
                })]),
                text_flow: None,
            }],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        };

        let result = engine.tag_slide(&slide).unwrap();

        let body_indices = result.frame_child_part_indices[0]
            .as_ref()
            .expect("single-block Body must have Some(Vec) child indices");
        assert_eq!(
            body_indices.len(),
            1,
            "single-block Body must record exactly 1 child index"
        );

        // The Part must have exactly one child (the P group).
        assert_eq!(
            result.part.children.len(),
            1,
            "Part must have 1 child for single-block Body"
        );
        if let Node::Group(g) = &result.part.children[body_indices[0]] {
            assert!(
                matches!(g.tag, TagKind::P(_)),
                "single-block Body child must be P; got {:?}",
                g.tag
            );
        }
    }
}
