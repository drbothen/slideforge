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
/// The public API surfaces a `Result` return type on [`AltTextEmbedder::decisions_for_slide`]
/// for forward-compatibility with future frame types that may require fallible processing.
/// The current implementation is infallible — every `FrameContent` variant maps to either
/// a concrete `AltDecision` or is skipped. No `PptxError` is returned at this time.
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
    /// | `Diagram { alt: AltText::Decorative, .. }` | `AltDecision::Decorative` |
    /// | `Chart { alt: AltText::Provided(s) }` | `AltDecision::Provided(s.clone())` |
    /// | `Chart { alt: AltText::Decorative }` | `AltDecision::Decorative` |
    /// | All others | Skipped |
    ///
    /// # Errors
    ///
    /// Currently infallible — returns `Ok` unconditionally. The `Result` wrapper is
    /// retained for forward-compatibility: future frame types (e.g., video, 3D model)
    /// may require fallible alt text extraction. Callers should propagate `?` normally.
    pub fn decisions_for_slide(
        slide: &LaidOutSlide,
    ) -> Result<Vec<(usize, AltDecision)>, PptxError> {
        let mut decisions = Vec::new();
        for (frame_idx, frame) in slide.frames.iter().enumerate() {
            let decision = match &frame.content {
                // STORY-039: Image, Chart, and Diagram all carry `alt: AltText`.
                // Single arm: Provided → AltDecision::Provided, Decorative → AltDecision::Decorative.
                FrameContent::Image { alt }
                | FrameContent::Chart { alt }
                | FrameContent::Diagram { alt, .. } => match alt {
                    slideforge_types::AltText::Provided(s) => {
                        Some(AltDecision::Provided(s.clone()))
                    },
                    // Decorative = author opt-out; Unspecified = pipeline placeholder (no author alt).
                    // Both are treated as the Artifact/Decorative path for PPTX a11y output
                    // (no descr attribute). The post-layout validator fires E-A11-001 for
                    // Unspecified in strict mode (ADR-019 Decision 5.3).
                    slideforge_types::AltText::Decorative
                    | slideforge_types::AltText::Unspecified => Some(AltDecision::Decorative),
                },
                // Text frames and non-visual frames do not get descr attributes.
                // STORY-087 pass-2: ColorBar is geometry-only; no descr attribute.
                FrameContent::Title(_)
                | FrameContent::Subtitle(_)
                | FrameContent::Body(_)
                | FrameContent::TextRun(_)
                | FrameContent::Shape(_)
                | FrameContent::ErrorSlidePlaceholder { .. }
                | FrameContent::Empty
                | FrameContent::ColorBar { .. } => None,
            };
            if let Some(d) = decision {
                decisions.push((frame_idx, d));
            }
        }
        Ok(decisions)
    }
}
