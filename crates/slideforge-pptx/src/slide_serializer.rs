//! Per-slide `slide*.xml` serializer.
//!
//! [`SlideSerializer`] produces the `ppt/slides/slide{n}.xml` bytes for one
//! [`slideforge_layout::LaidOutSlide`] using the `ooxmlsdk` typed API.
//!
//! ## Placeholder mapping (AC-004)
//!
//! Each `FrameContent` variant maps to a specific PPTX placeholder `idx`:
//!
//! | `FrameContent` variant | `<p:ph>` idx | Notes |
//! |------------------------|-------------|-------|
//! | `Title` / `Heading`    | `0`         | Title placeholder |
//! | `TextRun` / `Bullets`  | `1`         | Content/body placeholder |
//! | `Speaker` (notes)      | — excluded — | Notes go to `notesSlide` (STORY-040) |
//!
//! ## Element ordering (AC-006 / R4 finding)
//!
//! `ooxmlsdk` produces schema-correct element ordering automatically.
//! The expected child order within `<p:sp>` is:
//! `<p:nvSpPr>` → `<p:spPr>` → `<p:txBody>`.
//!
//! ## Dark layout `<p:clrMapOvr>` (AC-EC-005 / brand-architecture §Dark Layout)
//!
//! Slides whose layout is dark-themed include
//! `<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>` after `<p:cSld>`.
//!
//! ## Register bleed guard (AC-010)
//!
//! `report` and `detail` register content MUST NOT appear in `<p:sp>` elements
//! in `ppt/slides/slide*.xml`. Only `notes`-register text is forwarded to
//! `notesSlide` parts (STORY-040). This module never writes report/detail.

use slideforge_layout::{LaidOutSlide, LayoutWarning};
use slideforge_types::Brand;

use crate::error::PptxError;

/// Serializes one `LaidOutSlide` into `slide*.xml` bytes.
///
/// A new `SlideSerializer` is created per slide. State that varies per slide
/// (e.g., which layout is active, whether it is dark-themed) is passed as
/// constructor arguments.
pub struct SlideSerializer {
    /// Whether the layout used by this slide is dark-themed.
    ///
    /// When `true`, `<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>` is
    /// emitted after `<p:cSld>` in the output.
    is_dark_layout: bool,
    /// The layout index (0-based) used by this slide, for `<p:sp><p:nvSpPr>`.
    layout_index: usize,
}

impl SlideSerializer {
    /// Create a serializer for one slide.
    ///
    /// `is_dark_layout` controls `<p:clrMapOvr>` emission.
    /// `layout_index` is the 0-based layout index used to determine the slide
    /// layout reference in the slide XML (STORY-037 defers multi-layout support;
    /// default is layout 0 for all slides).
    #[must_use]
    pub fn new(is_dark_layout: bool, layout_index: usize) -> Self {
        todo!("SlideSerializer::new — store fields")
    }

    /// Generate `slide{n+1}.xml` bytes for `slide` (0-based index `slide_index`).
    ///
    /// `layout_rel_id` is the `rId` of the layout relationship in
    /// `ppt/slides/_rels/slide{n+1}.xml.rels`.
    ///
    /// Returns the XML bytes and any non-fatal warnings detected during
    /// serialisation (e.g., a frame with zero-height content).
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::OoxmlElement`] if `ooxmlsdk` fails to build a
    /// required element, or [`PptxError::InvalidEmu`] if a frame's bounding
    /// box contains an invalid coordinate.
    pub fn build(
        &self,
        slide: &LaidOutSlide,
        slide_index: usize,
        layout_rel_id: &str,
    ) -> Result<(Vec<u8>, Vec<LayoutWarning>), PptxError> {
        todo!(
            "SlideSerializer::build — construct CT_Slide via ooxmlsdk; \
             map FrameContent variants to <p:sp> elements with correct ph idx; \
             map InlineSpan to <a:r>/<a:rPr>; \
             emit clrMapOvr when is_dark_layout; \
             exclude report/detail register content"
        )
    }
}
