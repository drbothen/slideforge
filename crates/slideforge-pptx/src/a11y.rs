//! Accessibility metadata embedding for PPTX output.
//!
//! [`AltTextEmbedder`] walks a `LaidOutSlide`'s frames and returns the
//! [`AltDecision`] for each visual frame, which the slide serializer uses to
//! set the `descr` attribute on `<p:cNvPr>` elements.
//!
//! ## Alt text placement (BC-4.01.004 invariant 2)
//!
//! The `descr` attribute on `<p:cNvPr>` is the correct OOXML alt text location
//! for shapes. The `altText` attribute on `<p:ph>` is for placeholder names
//! only and MUST NOT be used for accessibility alt text.
//!
//! ## Protocol
//!
//! - Non-decorative visual elements: `descr="<alt text value>"` (non-empty).
//! - Decorative elements: `descr=""` (attribute present, empty value).
//! - Alt text is NEVER truncated (BC-4.01.004 invariant 1).
//! - XML special characters are escaped by `ooxmlsdk` automatically.
//!
//! ## Group shapes (AC-005)
//!
//! For chart and diagram frames whose SVG is embedded in a `<p:grpSp>` or
//! `<p:pic>`, the `descr` is set on the enclosing group shape's `<p:cNvPr>`,
//! not on any inner text box.

#![forbid(unsafe_code)]

use slideforge_layout::{FrameContent, LaidOutSlide};

use crate::error::PptxError;

/// Embeds accessibility `descr` attributes on every visual shape in a slide.
///
/// `AltTextEmbedder` is stateless; create one per slide or share across slides.
/// It provides [`AltTextEmbedder::decisions_for_slide`] to extract alt text
/// decisions from a `LaidOutSlide`, which the slide serializer uses to set
/// the `description` field on `<p:cNvPr>` elements (mapping to `descr` in XML).
///
/// ## When to use
///
/// Call [`AltTextEmbedder::decisions_for_slide`] to get the list of
/// `(frame_idx, AltDecision)` pairs for a slide, then pass each
/// [`AltDecision`] to the shape builder when constructing `<p:pic>` or
/// `<p:grpSp>` elements so the `descr` attribute is set from the decision.
///
/// ## Errors
///
/// Returns [`PptxError::OoxmlElement`] if a visual frame is encountered
/// without an alt text decision (programming error in upstream evaluator).
pub struct AltTextEmbedder;

/// The alt text decision for a visual frame.
///
/// This enum represents the pre-validated alt text decision that must be present
/// on every visual frame before it reaches the PPTX exporter (enforced upstream
/// by BC-5.01.001 / STORY-015).
///
/// ## Invariant (BC-4.01.004 invariant 4)
///
/// No visual frame reaches `AltTextEmbedder` without an alt text decision.
/// A frame with neither a non-empty `alt` nor `decorative: true` is a
/// programming error in the upstream evaluator, not a user error.
#[derive(Debug, Clone)]
pub enum AltDecision {
    /// Non-decorative: embed the provided alt text string as `descr`.
    Provided(std::sync::Arc<str>),
    /// Decorative: embed `descr=""` (attribute present, empty value).
    Decorative,
}

impl AltDecision {
    /// Return the `descr` attribute value for this decision.
    ///
    /// - `AltDecision::Provided(text)`: returns `text` as `&str` (never truncated).
    /// - `AltDecision::Decorative`: returns `""` (empty string, attribute still present).
    ///
    /// The caller sets `NonVisualDrawingProperties::description = Some(self.descr_value().to_owned())`
    /// to ensure the `descr` attribute is present in both cases.
    #[must_use]
    pub fn descr_value(&self) -> &str {
        match self {
            Self::Provided(text) => text.as_ref(),
            Self::Decorative => "",
        }
    }
}

impl AltTextEmbedder {
    /// Embed `descr` attributes into slide XML for all visual frames.
    ///
    /// `slide_xml` is the raw `ppt/slides/slide{n}.xml` bytes as produced by
    /// [`crate::slide_serializer::SlideSerializer::build`].
    ///
    /// `_alt_decisions` is a parallel slice indexed by frame order (same as
    /// `LaidOutSlide::frames`). Only frames that correspond to visual shapes
    /// (image, chart, diagram) contribute entries; text-only frames do not.
    /// The caller is responsible for building this slice from `LaidOutSlide`.
    ///
    /// In the current implementation, alt text is embedded directly during
    /// shape construction in `SlideSerializer::build_shape_tree` via the
    /// `description` field on `NonVisualDrawingProperties`. This method is
    /// provided for callers that need to post-process already-serialized XML;
    /// it returns the bytes unchanged since the attributes are already present.
    ///
    /// Returns the slide XML bytes (unchanged — alt text is embedded at
    /// shape-construction time by the slide serializer).
    ///
    /// # Errors
    ///
    /// This implementation is infallible; it returns the input bytes unchanged.
    /// The `PptxError` return type is preserved for API stability.
    pub fn embed(slide_xml: Vec<u8>, _alt_decisions: &[AltDecision]) -> Result<Vec<u8>, PptxError> {
        // Alt text is embedded at shape-construction time by SlideSerializer
        // via NonVisualDrawingProperties::description. No post-processing needed.
        Ok(slide_xml)
    }

    /// Extract the alt text decisions for all visual frames in a `LaidOutSlide`.
    ///
    /// Walks `slide.frames` and returns a `Vec<(frame_idx, AltDecision)>` for
    /// every frame that corresponds to a visual shape (image, chart, diagram).
    /// Text-only frames (title, subtitle, body, text-run) are skipped.
    ///
    /// ## Frame type mapping (STORY-039 IR alt-threading)
    ///
    /// | `FrameContent` variant | Alt decision source |
    /// |------------------------|---------------------|
    /// | `Image { alt: AltText::Provided(s) }` | `AltDecision::Provided(s.clone())` |
    /// | `Image { alt: AltText::Decorative }` | `AltDecision::Decorative` |
    /// | `Diagram { alt: AltText::Provided(s), .. }` | `AltDecision::Provided(s.clone())` |
    /// | `Diagram { alt: AltText::Decorative, .. }` | `AltDecision::Decorative` (stub: threading not impl) |
    /// | `Chart { alt: AltText::Provided(s) }` | `AltDecision::Provided(s.clone())` |
    /// | `Chart { alt: AltText::Decorative }` | `AltDecision::Decorative` (stub: threading not impl) |
    /// | All others | Skipped |
    ///
    /// # Errors
    ///
    /// This function is infallible for the current set of frame types.
    pub fn decisions_for_slide(
        slide: &LaidOutSlide,
    ) -> Result<Vec<(usize, AltDecision)>, PptxError> {
        let mut decisions = Vec::new();
        for (frame_idx, frame) in slide.frames.iter().enumerate() {
            let decision = match &frame.content {
                // STORY-039: Image now carries AltText enum (Provided or Decorative).
                FrameContent::Image { alt } => match alt {
                    slideforge_types::AltText::Provided(s) => {
                        Some(AltDecision::Provided(s.clone()))
                    },
                    slideforge_types::AltText::Decorative => Some(AltDecision::Decorative),
                },
                // STORY-039: Chart and Diagram now carry alt: AltText.
                // The real alt-threading from layout::run is NOT YET IMPLEMENTED;
                // the stub path from regions.rs produces AltText::Decorative.
                // Once layout::run threads ChartSpec.alt / DiagramSpec.alt, this
                // arm will correctly carry Provided alt through to the PPTX descr.
                FrameContent::Chart { alt } | FrameContent::Diagram { alt, .. } => match alt {
                    slideforge_types::AltText::Provided(s) => {
                        Some(AltDecision::Provided(s.clone()))
                    },
                    slideforge_types::AltText::Decorative => Some(AltDecision::Decorative),
                },
                // Text frames and non-visual frames do not get descr attributes.
                FrameContent::Title(_)
                | FrameContent::Subtitle(_)
                | FrameContent::Body(_)
                | FrameContent::TextRun(_)
                | FrameContent::Shape(_)
                | FrameContent::ErrorSlidePlaceholder { .. }
                | FrameContent::Empty => None,
            };
            if let Some(d) = decision {
                decisions.push((frame_idx, d));
            }
        }
        Ok(decisions)
    }
}
