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
    pub part: TagGroup,
    /// Frame indices (into `LaidOutSlide::frames`) that are decorative.
    ///
    /// During the drawing pass, the exporter marks content at these indices
    /// as `ContentTag::Artifact(ArtifactType::Other)` instead of linking them
    /// to a tag group identifier.
    pub decorative_frame_indices: Vec<usize>,
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
    /// Decorative frames (frames containing `FrameContent::Image { alt }`
    /// where alt is empty, or `FrameContent::Shape` with `AltText::Decorative`)
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
        // One Part group per slide — wraps all structural children.
        let mut part_group = TagGroup::new(Tag::<krilla::tagging::kind::Part>::Part);
        let mut decorative_frame_indices: Vec<usize> = Vec::new();

        for (frame_idx, frame) in slide.frames.iter().enumerate() {
            match &frame.content {
                // ── Empty frames: skip (no tag, not an artifact) ──────────────
                FrameContent::Empty => {
                    // Intentionally omitted from the tag tree. An empty frame
                    // has no renderable content and no semantic role.
                },

                // ── Title → H1 ───────────────────────────────────────────────
                FrameContent::Title(_text) => {
                    let heading_group = TagGroup::new(Tag::<krilla::tagging::kind::Hn>::Hn(
                        // NonZeroU16::MIN == 1 (H1); infallible construction.
                        std::num::NonZeroU16::MIN,
                        None,
                    ));
                    part_group.push(heading_group);
                },

                // ── Subtitle → H2 ────────────────────────────────────────────
                FrameContent::Subtitle(_text) => {
                    let heading_group =
                        TagGroup::new(Tag::<krilla::tagging::kind::Hn>::Hn(H2_LEVEL, None));
                    part_group.push(heading_group);
                },

                // ── Body content → P per paragraph or L+LI+LBody per list ────
                FrameContent::Body(content_blocks) => {
                    if content_blocks.is_empty() {
                        // No blocks — emit a generic P so Part remains non-empty.
                        part_group.push(TagGroup::new(Tag::<krilla::tagging::kind::P>::P));
                    } else {
                        for block in content_blocks {
                            let group = self.tag_content_block(block)?;
                            if let Some(g) = group {
                                part_group.push(g);
                            }
                        }
                    }
                },

                // ── Image → Figure+Alt (or Artifact if decorative) ───────────
                FrameContent::Image { alt } => {
                    if alt.is_empty() {
                        // Empty alt on Image = decorative; mark as Artifact.
                        decorative_frame_indices.push(frame_idx);
                    } else {
                        part_group.push(self.tag_figure(Some(alt))?);
                    }
                },

                // ── Diagram → Artifact (frame-level, v1 layout) ──────────────
                FrameContent::Diagram(_diagram_svg) => {
                    // F-045-I2 / BC-4.03.001 invariant-3 investigation result:
                    //
                    // `FrameContent::Diagram` IS emitted by the v1 layout engine
                    // (regions.rs:280) as `FrameContent::Diagram(empty_placeholder())`.
                    // The frame carries NO alt text — the original `DiagramSpec.alt`
                    // is not threaded through the geometric IR in v1.
                    //
                    // Emitting `tag_figure(None)` would produce a /Figure with no /Alt
                    // — a BC-4.03.001 invariant-3 violation ("every Figure has non-empty
                    // /Alt").  Emitting a placeholder string is an "alt lie".
                    //
                    // Resolution: treat ALL frame-level Diagram frames as Artifacts in v1.
                    // The empty-placeholder SVG has no meaningful user content to describe;
                    // the semantic content comes from the surrounding text, which IS tagged.
                    // The IR-threading story (planned for a future wave) will add an
                    // `alt: Option<Arc<str>>` field to `FrameContent::Diagram` — at that
                    // point this arm will be updated to:
                    //   - `AltText::Provided(s)` → `part_group.push(self.tag_figure(Some(s))?)`.
                    //   - `AltText::Decorative` / None → `decorative_frame_indices.push(frame_idx)`.
                    //
                    // This is NOT modifying slideforge-layout (which is STORY-073's territory).
                    // It is a correct tagging decision within the current IR constraint.
                    decorative_frame_indices.push(frame_idx);
                },

                // ── Chart → Artifact (frame-level, v1 layout) ────────────────
                FrameContent::Chart => {
                    // F-045-I2 / BC-4.03.001 invariant-3 investigation result:
                    //
                    // `FrameContent::Chart` IS emitted by the v1 layout engine
                    // (regions.rs:264) but carries NO data and NO alt text in the
                    // geometric IR.  Same rationale as `FrameContent::Diagram` above.
                    //
                    // Treat ALL frame-level Chart frames as Artifacts in v1.
                    // When the IR-threading story ships, this arm will be updated to
                    // use the real alt from `ChartSpec.alt`.
                    //
                    // This is NOT modifying slideforge-layout (STORY-073 constraint).
                    decorative_frame_indices.push(frame_idx);
                },

                // ── Shape → Figure+Alt or Artifact if decorative ──────────────
                FrameContent::Shape(shape_frame) => {
                    match &shape_frame.alt {
                        slideforge_types::AltText::Decorative => {
                            // Decorative shape: not in the tag tree — mark as Artifact.
                            decorative_frame_indices.push(frame_idx);
                        },
                        slideforge_types::AltText::Provided(alt) => {
                            part_group.push(self.tag_figure(Some(alt))?);
                        },
                    }
                },

                // ── TextRun → P ───────────────────────────────────────────────
                FrameContent::TextRun(_inlines) => {
                    let p_group = TagGroup::new(Tag::<krilla::tagging::kind::P>::P);
                    part_group.push(p_group);
                },

                // ── ErrorSlidePlaceholder → P (error text is readable) ────────
                FrameContent::ErrorSlidePlaceholder { .. } => {
                    let p_group = TagGroup::new(Tag::<krilla::tagging::kind::P>::P);
                    part_group.push(p_group);
                },
            }
        }

        Ok(PartResult {
            part: part_group,
            decorative_frame_indices,
        })
    }

    /// Build a tag group for a single [`ContentBlock`] from a Body frame.
    ///
    /// Returns:
    /// - `Ok(Some(TagGroup))` for structured content (P, L+LI+LBody, Table, Figure).
    /// - `Ok(None)` for blocks that do not produce a tag (e.g., decorative Shape).
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError::Serialize`] if the tag group cannot be built.
    fn tag_content_block(
        &self,
        block: &slideforge_types::ContentBlock,
    ) -> Result<Option<TagGroup>, PdfExportError> {
        match block {
            // Text paragraph → P
            ContentBlock::Text(_text_block) => {
                Ok(Some(TagGroup::new(Tag::<krilla::tagging::kind::P>::P)))
            },

            // Bullet list → L (Disc) with LI+LBody for each item.
            // An empty items list produces no tag group: a childless L element
            // has no semantic value in a PDF structure tree and is structurally odd.
            ContentBlock::Bullets(items) => {
                if items.is_empty() {
                    return Ok(None);
                }
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

            // Shape in body → Figure+Alt or skip if decorative
            ContentBlock::Shape(shape_spec) => match &shape_spec.alt {
                Some(AltText::Decorative) | None => Ok(None),
                Some(AltText::Provided(alt)) => Ok(Some(self.tag_figure(Some(alt))?)),
            },

            // Chart in body → Figure+Alt, or omit if decorative / no alt.
            //
            // F-045-I1 (alt-lie fix): `.or(Some("chart"))` was a placeholder that
            // produced a /Figure with the generic string "chart" as alt text.
            // This is an "alt lie" — the string describes the element type, not the
            // chart content.  PDF/UA-1 requires meaningful alt text.
            //
            // Correct behavior:
            //   - `AltText::Provided(s)` → emit Figure with real alt text.
            //   - `AltText::Decorative` or `alt: None` → omit from tag tree
            //     (return `Ok(None)`); a chart with no author-supplied alt is
            //     semantically inaccessible and must not pretend otherwise.
            ContentBlock::Chart(chart_spec) => {
                match chart_spec.alt.as_ref() {
                    Some(AltText::Provided(s)) => Ok(Some(self.tag_figure(Some(s))?)),
                    Some(AltText::Decorative) | None => Ok(None),
                }
            },

            // Diagram in body → Figure+Alt, or omit if decorative / no alt.
            //
            // F-045-I1 (alt-lie fix): `.or(Some("diagram"))` was a placeholder.
            // Same rationale as the Chart arm above.
            ContentBlock::Diagram(diagram_spec) => {
                match diagram_spec.alt.as_ref() {
                    Some(AltText::Provided(s)) => Ok(Some(self.tag_figure(Some(s))?)),
                    Some(AltText::Decorative) | None => Ok(None),
                }
            },

            // Image in body → Figure+Alt
            ContentBlock::Image(image_spec) => match &image_spec.alt {
                Some(AltText::Decorative) | None => Ok(None),
                Some(AltText::Provided(alt)) => Ok(Some(self.tag_figure(Some(alt))?)),
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
    /// provided. Pass `Some(alt)` for accessible figures; `None` for figures
    /// where alt text is not yet available (e.g., diagrams before STORY-045
    /// threads the original `DiagramSpec.alt` through the IR).
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
                    // Decorative image: empty alt string.
                    content: FrameContent::Image { alt: Arc::from("") },
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
                // Non-decorative image with real alt text.
                content: FrameContent::Image {
                    alt: Arc::from("A mountain landscape at sunrise"),
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
}
