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
//! ## RISK-3 note (from tech-validation.md)
//!
//! `TagKind` is the variant-bearing enum. `Tag::Figure`-style references
//! **will not compile**. Use `TagKind::Figure` wrapped in
//! `Tag::with(TagKind::Figure)`. `Tag` is the kind + attributes wrapper.

use krilla::tagging::{TagGroup, TagTree};
use slideforge_layout::LaidOutSlide;

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
    /// The returned [`TagTree`] contains a `Document` group wrapping one
    /// `Part` group per slide, with inner groups for each semantic content
    /// element. The caller is responsible for attaching the combined tree to
    /// the `Document` via `Document::set_tag_tree`.
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError::Serialize`] if the tag tree cannot be
    /// constructed (e.g., an unexpected krilla invariant violation).
    pub fn tag_slide(&self, _slide: &LaidOutSlide) -> Result<TagTree, PdfExportError> {
        todo!("STORY-043 Red Gate stub: tag_slide not yet implemented — implement in TDD green phase")
    }

    /// Tag a figure element with an `/Alt` attribute for PDF/UA-1 compliance.
    ///
    /// Returns a [`TagGroup`] using `TagKind::Figure` (wrapped in
    /// `Tag::with(TagKind::Figure)`) with the alt text set on the `Tag`.
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError::Serialize`] if the figure tag cannot be
    /// constructed.
    pub fn tag_figure(&self, _alt: &str) -> Result<TagGroup, PdfExportError> {
        todo!("STORY-043 Red Gate stub: tag_figure not yet implemented")
    }

    /// Tag a table element as `Table → TR → TH/TD` per PDF/UA-1.
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError::Serialize`] if the table tag cannot be
    /// constructed.
    pub fn tag_table(&self) -> Result<TagGroup, PdfExportError> {
        todo!("STORY-043 Red Gate stub: tag_table not yet implemented")
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
        _slide_parts: Vec<TagGroup>,
    ) -> Result<TagTree, PdfExportError> {
        todo!("STORY-043 Red Gate stub: assemble_deck_tag_tree not yet implemented")
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
    use std::sync::Arc;
    use slideforge_layout::{
        types::{LaidOutSlide, Frame, BoundingBox, FrameContent, RegisterSet},
    };
    use slideforge_types::Emu;

    /// Helper: build a minimal 1-frame LaidOutSlide for tag-engine tests.
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

    /// BC-4.03.002 AC-003: `SlideTagEngine::tag_slide()` produces a `TagTree`
    /// with one `TagKind::Part` group per slide + figure/alt tags.
    ///
    /// RED GATE: This test MUST FAIL because `tag_slide` is a `todo!()` stub.
    #[test]
    #[should_panic(expected = "STORY-043 Red Gate stub")]
    fn test_bc_4_03_002_tag_slide_panics_at_stub() {
        let engine = SlideTagEngine::new();
        let slide = minimal_slide();
        // This panics at todo!() — confirms Red Gate is active.
        let _ = engine.tag_slide(&slide);
    }

    /// BC-4.03.002 AC-003 (behavioral assertion): when implemented, `tag_slide`
    /// must return a `TagTree` that contains at least one `Part` group.
    ///
    /// RED GATE: This test MUST FAIL because `tag_slide` is a `todo!()` stub.
    ///
    /// After implementation, the `todo!()` is replaced with real logic and this
    /// test drives correctness: the returned tree must have a Document → Part
    /// hierarchy.
    #[test]
    fn test_bc_4_03_002_tag_slide_produces_one_part_per_slide() {
        let engine = SlideTagEngine::new();
        let slide = minimal_slide();
        // Call tag_slide — will panic at todo!() until implemented.
        // When implemented: assert the TagTree contains exactly one Part group.
        let result = engine.tag_slide(&slide);
        // After implementation the result must be Ok:
        assert!(
            result.is_ok(),
            "tag_slide must succeed for a valid LaidOutSlide"
        );
        // The implementer must also verify the Part count:
        // let tree = result.unwrap();
        // assert that tree has Document → Part structure.
        // (Structural assertion added here by the implementer.)
    }
}
