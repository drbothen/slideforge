//! The core layout transformation: `Deck → LaidOutDeck`.
//!
//! This module exports a single public function: [`run`], which is the
//! **only** function in the slideforge pipeline that crosses the semantic
//! (pre-layout) to geometric (post-layout) boundary.
//!
//! ## Contract
//!
//! - **Input:** A fully evaluated [`slideforge_types::Deck`] and a
//!   [`slideforge_types::Brand`] configuration.
//! - **Output:** A [`crate::types::LaidOutDeck`] with precisely-positioned
//!   [`crate::types::Frame`]s in EMU coordinates, or a
//!   [`crate::error::LayoutError`] on invariant violation.
//!
//! ## Invariants (all enforced at runtime by post-layout integrity check)
//!
//! | Invariant | Reference | Violation → |
//! |-----------|-----------|-------------|
//! | Slide count preserved | BC-3.06.001, VP-011 | `LayoutError::SlideCountMismatch` |
//! | Deterministic output | BC-3.06.002 | (proptest verified) |
//! | Valid EMU bounding boxes | BC-3.06.003 | `LayoutError::InvalidBoundingBox` |
//! | No unknown slide types | EC-002 | `LayoutError::UnknownSlideType` |
//! | Non-zero slide count | EC-001 | `LayoutError::EmptyDeck` |
//!
//! ## Purity (AC-008)
//!
//! `layout::run` is a **pure function**: no file I/O, no network access, no
//! `tracing` spans, no `println!`, no random state. This purity makes it
//! Kani-amenable (Phase 6) and enables full proptest coverage of VP-011.

use std::sync::Arc;

use slideforge_types::{Brand, Deck, FieldValue, Register, Value};

use crate::error::LayoutError;
use crate::regions::region_frames_for;
use crate::sections::collect_sections;
use crate::text_flow::compute_text_flow;
use crate::types::{
    DEFAULT_PAGE_HEIGHT, DEFAULT_PAGE_WIDTH, FrameContent, LaidOutDeck, LaidOutSlide, PageSize,
    RegisterSet, RegisterTag,
};

/// Transform a fully evaluated `Deck` into a geometric `LaidOutDeck`.
///
/// `run` is the sole function that crosses the semantic → geometric IR
/// boundary. Every exporter consumes `LaidOutDeck`; none may access `Deck`
/// directly after layout has run.
///
/// # Arguments
///
/// * `deck` — The semantic IR produced by the evaluator. All expressions must
///   be resolved; no `{{ }}` interpolations may remain.
/// * `brand` — The brand configuration loaded by the `BrandProvider`. If the
///   brand has no canvas layout overrides, the default 16:9 widescreen page
///   size is used.
///
/// # Returns
///
/// * `Ok(LaidOutDeck)` — All slides laid out with valid EMU coordinates.
/// * `Err(LayoutError::EmptyDeck)` — The deck contains zero slides (EC-001).
/// * `Err(LayoutError::UnknownSlideType)` — A slide's type keyword has no
///   registered region map (EC-002).
/// * `Err(LayoutError::SlideCountMismatch)` — Internal bug: produced slide
///   count does not equal input count (BC-3.06.001 defensive check).
/// * `Err(LayoutError::InvalidBoundingBox)` — A produced bounding box
///   violates coordinate invariants (BC-3.06.003 defensive check).
///
/// # Purity (AC-008)
///
/// This function is pure: no I/O, no side effects, no panics.
///
/// # Errors
///
/// Returns [`LayoutError`] on any invariant violation. See variants above.
pub fn run(deck: &Deck, brand: &Brand) -> Result<LaidOutDeck, LayoutError> {
    // EC-001: reject empty decks.
    if deck.slides.is_empty() {
        return Err(LayoutError::EmptyDeck {
            source_slide_index: 0,
        });
    }

    // AC-004: derive page size from brand's first layout definition, or fall
    // back to the default 16:9 widescreen.
    let page_size = if let Some(layout_def) = brand.layouts.first() {
        PageSize {
            width: layout_def.canvas_width,
            height: layout_def.canvas_height,
        }
    } else {
        PageSize {
            width: DEFAULT_PAGE_WIDTH,
            height: DEFAULT_PAGE_HEIGHT,
        }
    };

    // Layout each slide.
    let mut laid_out_slides: Vec<LaidOutSlide> = Vec::with_capacity(deck.slides.len());

    for (source_index, slide) in deck.slides.iter().enumerate() {
        let slide_type_keyword: Arc<str> = Arc::clone(&slide.slide_type);
        let keyword_str: &str = slide_type_keyword.as_ref();

        // EC-002: unknown slide type → error.
        let frames =
            region_frames_for(keyword_str, page_size.width, page_size.height).ok_or_else(|| {
                LayoutError::UnknownSlideType {
                    source_slide_index: source_index,
                    slide_type_keyword: keyword_str.to_owned(),
                }
            })?;

        // BC-3.06.003: validate every bounding box produced by the region map.
        for (frame_index, frame) in frames.iter().enumerate() {
            if !frame.bbox.is_valid(page_size.width, page_size.height) {
                return Err(LayoutError::InvalidBoundingBox {
                    source_slide_index: source_index,
                    frame_index,
                    bbox: frame.bbox,
                });
            }
        }

        // NOTE: FrameContent variants in the frames produced by region_frames_for
        // start as Empty/Image/Chart/Diagram placeholders. Richer content population
        // (e.g., wiring slide body blocks into FrameContent::Body) is STORY-027
        // scope. The region map establishes the geometric foundation; content
        // resolution is a separate pass in Phase 3.
        //
        // MED-002: Compute text_flow for text-bearing frames.
        // For title/subtitle frames, extract text from the slide's resolved fields
        // to enable the canvas overflow validator (BC-3.03.001).
        let frames: Vec<_> = frames
            .into_iter()
            .enumerate()
            .map(|(frame_idx, mut frame)| {
                let text: Option<&str> = match &frame.content {
                    // For title-region frames (index 0), check slide "title" field.
                    FrameContent::Empty if frame_idx == 0 => match slide.fields.get("title") {
                        Some(FieldValue::Literal(Value::Str(s))) => Some(s.as_ref()),
                        _ => None,
                    },
                    // For subtitle/body frames (index 1+), check "subtitle" then "body".
                    FrameContent::Empty if frame_idx == 1 => match slide.fields.get("subtitle") {
                        Some(FieldValue::Literal(Value::Str(s))) => Some(s.as_ref()),
                        _ => match slide.fields.get("body") {
                            Some(FieldValue::Literal(Value::Str(s))) => Some(s.as_ref()),
                            _ => None,
                        },
                    },
                    // Non-text frames (Image, Chart, Diagram, Shape) get no text_flow.
                    _ => None,
                };
                if let Some(t) = text {
                    frame.text_flow = Some(compute_text_flow(t, frame.bbox));
                }
                frame
            })
            .collect();

        // Extract speaker notes from the slide's "notes" field, if present and
        // resolved to a plain string value.
        let speaker_notes: Option<Arc<str>> = match slide.fields.get("notes") {
            Some(FieldValue::Literal(Value::Str(s))) => Some(Arc::clone(s)),
            _ => None,
        };

        // Derive register_tags from the semantic slide's register field.
        // An unregistered slide (register: None) produces an empty Vec.
        let register_tags: RegisterSet = match slide.register {
            Some(Register::Notes) => vec![RegisterTag::Notes],
            Some(Register::Report) => vec![RegisterTag::Report],
            Some(Register::Detail) => vec![RegisterTag::Detail],
            None => vec![],
        };

        laid_out_slides.push(LaidOutSlide {
            source_index,
            slide_type_keyword,
            frames,
            speaker_notes,
            register_tags,
        });
    }

    // BC-3.06.001 defensive check: slide count must be preserved exactly.
    if laid_out_slides.len() != deck.slides.len() {
        return Err(LayoutError::SlideCountMismatch {
            expected: deck.slides.len(),
            actual: laid_out_slides.len(),
            source_slide_index: 0,
        });
    }

    // STORY-027: Section collection pass.
    // Collect all document sections (auto-generated + manual) from the deck.
    // Auto-generated sections: ExecutiveSummary from takeaway fields, RiskRegister
    // from severity_cards slides. Manual sections from section_blocks (when Deck gains
    // that field). Supersession and ordering rules are applied inside collect_sections.
    let sections = collect_sections(deck);

    Ok(LaidOutDeck {
        page_size,
        slides: laid_out_slides,
        sections,
    })
}
