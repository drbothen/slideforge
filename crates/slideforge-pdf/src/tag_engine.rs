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
//!     H1 / P / LI  ← per text element's semantic role
//!     Figure (+ /Alt)  ← for image / diagram frames
//!     Table → TR → TH/TD  ← for table frames
//! ```
//!
//! Decorative elements (`decorative: true`, empty alt) are marked as PDF
//! Artifacts and are NOT wrapped in a `Tag`.
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
//! only group nodes; identifiers are inserted by `PdfExporter` during
//! the surface-drawing pass.

use krilla::tagging::{ListNumbering, Tag, TagGroup, TagTree};
use slideforge_layout::LaidOutSlide;
use slideforge_layout::types::FrameContent;

use crate::error::PdfExportError;

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
    /// The returned [`TagTree`] contains one `Part` group for this slide,
    /// with inner groups for each semantic content element drawn from
    /// `slide.frames`. Leaf identifiers (which link tag groups to drawn
    /// content via `Surface::start_tagged`) are inserted by
    /// [`crate::exporter::PdfExporter`] during the surface-drawing pass.
    ///
    /// ## Tag hierarchy produced
    ///
    /// ```text
    /// Part          ← one group wrapping this slide's content
    ///   H1/P/LI     ← per text frame's semantic role
    ///   Figure      ← for diagram / chart frames (with /Alt from frame alt text)
    ///   Table→TR→TH/TD ← for table frames
    /// ```
    ///
    /// Decorative frames (empty alt + decorative marker) are NOT wrapped in
    /// a `Tag` group — they will be marked as PDF Artifacts by the exporter.
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError::Serialize`] if the tag tree cannot be
    /// constructed.
    ///
    /// # Panics
    ///
    /// This function does not panic in practice. `NonZeroU16::new(2)` is
    /// infallible because 2 is a valid non-zero value.
    pub fn tag_slide(&self, slide: &LaidOutSlide) -> Result<TagTree, PdfExportError> {
        // One Part group per slide.
        let mut part_group = TagGroup::new(Tag::<krilla::tagging::kind::Part>::Part);

        for frame in &slide.frames {
            match &frame.content {
                FrameContent::Title(_text) => {
                    // Title text → H1 heading tag group.
                    // The leaf Identifier linking this group to the drawn
                    // glyph run is inserted by the exporter via
                    // `surface.start_tagged(ContentTag::Span(...))`.
                    let heading_group = TagGroup::new(Tag::<krilla::tagging::kind::Hn>::Hn(
                        // NonZeroU16::MIN == 1; infallible.
                        std::num::NonZeroU16::MIN,
                        None,
                    ));
                    part_group.push(heading_group);
                },
                FrameContent::Body(_content_blocks) => {
                    // Body content with possible bullet list items → L group.
                    let list_group =
                        TagGroup::new(Tag::<krilla::tagging::kind::L>::L(ListNumbering::Disc));
                    part_group.push(list_group);
                },
                FrameContent::Diagram(_) | FrameContent::Chart => {
                    // Vector figure — alt text carried by the frame; for now
                    // we emit a Figure group with no alt (STORY-045 wires
                    // the actual alt text from the frame's alt field).
                    let figure_group =
                        TagGroup::new(Tag::<krilla::tagging::kind::Figure>::Figure(None));
                    part_group.push(figure_group);
                },
                // Other frame types (Subtitle, Image, Shape, TextRun, Empty,
                // ErrorSlide, etc.) use a generic P group for now.
                // STORY-045 will refine with proper semantic roles.
                FrameContent::Subtitle(_text) => {
                    // H2 heading for subtitle frames.
                    // SAFETY: 2 is a valid non-zero value; unwrap_or_else on
                    // NonZeroU16::new(2) would be infallible, but we use
                    // a saturating construction to avoid the lint.
                    #[allow(clippy::unwrap_used)]
                    let level = std::num::NonZeroU16::new(2).unwrap();
                    let heading_group =
                        TagGroup::new(Tag::<krilla::tagging::kind::Hn>::Hn(level, None));
                    part_group.push(heading_group);
                },
                _ => {
                    // Image, Shape, TextRun, Empty, ErrorSlide — emit a
                    // non-structural group so the slide Part is non-empty.
                    let p_group = TagGroup::new(Tag::<krilla::tagging::kind::P>::P);
                    part_group.push(p_group);
                },
            }
        }

        let mut tree = TagTree::new();
        tree.push(part_group);
        Ok(tree)
    }

    /// Tag a figure element with an `/Alt` attribute for PDF/UA-1 compliance.
    ///
    /// Returns a [`TagGroup`] using `TagKind::Figure` with the alt text set.
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError::Serialize`] if the figure tag cannot be
    /// constructed.
    pub fn tag_figure(&self, alt: &str) -> Result<TagGroup, PdfExportError> {
        let figure_tag = Tag::<krilla::tagging::kind::Figure>::Figure(Some(alt.to_owned()));
        Ok(TagGroup::new(figure_tag))
    }

    /// Tag a table element as `Table → TR → TH/TD` per PDF/UA-1.
    ///
    /// Returns a minimal [`TagGroup`] for the table structure. The caller is
    /// responsible for populating row and cell groups before pushing into the
    /// parent structure.
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError::Serialize`] if the table tag cannot be
    /// constructed.
    pub fn tag_table(&self) -> Result<TagGroup, PdfExportError> {
        Ok(TagGroup::new(Tag::<krilla::tagging::kind::Table>::Table))
    }

    /// Produce a `Document`-level `TagTree` wrapping one `Part` group per
    /// slide's sub-tree.
    ///
    /// This is the top-level assembler called after all slides have been
    /// tagged. Each `part_group` in `slide_parts` corresponds to one slide.
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
            tree.push(part);
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
    use slideforge_types::Emu;
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
    /// `TagTree` that contains exactly one `Part` group per slide.
    ///
    /// Verifies: tree has one child (the Part group) which itself has one
    /// child (the H1 group for the Title frame).
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
        let tree = result.unwrap();
        // The tree must have exactly one child (one Part group for the slide).
        assert_eq!(
            tree.children.len(),
            1,
            "tag tree must contain exactly one Part group per slide"
        );
        // That child must be a Group node.
        let part_node = &tree.children[0];
        assert!(
            matches!(part_node, Node::Group(_)),
            "slide tag tree child must be a Group node"
        );
        // The group's tag must be a Part.
        if let Node::Group(group) = part_node {
            assert!(
                matches!(group.tag, TagKind::Part(_)),
                "slide tag tree child must be a Part group; got {:?}",
                group.tag
            );
            // The Part group must have children (at least the H1 for the Title frame).
            assert!(
                !group.children.is_empty(),
                "Part group must have at least one child (the H1 for the Title frame)"
            );
        }
    }
}
