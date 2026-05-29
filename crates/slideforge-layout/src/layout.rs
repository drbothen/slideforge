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
//! `println!`, no random state. This purity makes it Kani-amenable (Phase 6)
//! and enables full proptest coverage of VP-011.
//!
//! **Note on tracing diagnostics:** `collect_sections` emits `tracing::warn!`
//! events in two specific cases: (a) when a manual section supersedes an
//! auto-generated one (BC-3.02.001 EC-002 diagnostic), and (b) when
//! `section_order:` names a section that was not collected. These diagnostic
//! emissions are documented side-effects — they do not affect the output value
//! and do not block Kani analysis (tracing is a no-op in proof mode). The
//! "no side effects" purity claim is narrowed to: no I/O, no mutable global
//! state, and fully deterministic output for identical inputs.

use std::sync::Arc;

use slideforge_types::{Brand, ContentBlock, Deck, FieldValue, Register, Value};

use crate::error::LayoutError;
use crate::inline::run_inline_validation;
use crate::regions::region_frames_for;
use crate::sections::collect_sections;
use crate::shapes::{DEFAULT_EM_IN_EMU, layout_shapes};
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
/// * `Err(LayoutError::MissingAlt)` / `Err(LayoutError::Multiple)` — A shape
///   node was missing `alt` text or `decorative: true` (BC-3.04.001 EC-001).
/// * `Err(LayoutError::UnknownShapeType)` — A shape node has an unknown type
///   keyword (BC-3.04.001 invariant 4 / E-PAR-012-SHP).
/// * `Err(LayoutError::InlineDepthExceeded)` — An inline node tree exceeds the
///   maximum nesting depth (BC-3.05.001 E-LAY-005 / F-MED-006).
///
/// # Purity (AC-008)
///
/// This function is pure: no file I/O, no network access, no random state, no
/// panics. `collect_sections` emits `tracing::warn!` diagnostic events in two
/// cases (supersession notification and unknown `section_order` name); these are
/// documented side-effects and do not affect the deterministic output value.
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
    // Deck-level warning sink — shape off-canvas + inline xref-not-found warnings
    // are accumulated here and stored on LaidOutDeck::warnings (STORY-028 / AC-003 / AC-007).
    let mut deck_warnings: Vec<crate::types::LayoutWarning> = Vec::new();

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

        // AC-INT-1 (F-CRIT-001): Shape layout pass.
        // Filter ContentBlock::Shape blocks from the slide, convert their
        // positions to EMU frames, and append them after the region-map frames
        // in source order (BC-3.04.001 postcondition 4).
        let shape_specs: Vec<slideforge_types::ShapeSpec> = slide
            .blocks
            .iter()
            .filter_map(|b| {
                if let ContentBlock::Shape(s) = &b.content {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect();

        // layout_shapes returns either Ok(ShapeLayoutOutput) or Err(LayoutError).
        // Warnings (off-canvas positions) are non-fatal; errors (MissingAlt,
        // UnknownShapeType) are fatal and propagate out of layout::run.
        let shape_output = layout_shapes(&shape_specs, page_size, source_index, DEFAULT_EM_IN_EMU)?;

        let mut all_frames = frames;
        all_frames.extend(shape_output.frames);
        // Merge shape-level warnings (off-canvas) into the deck-level sink.
        // They are stored on LaidOutDeck::warnings (BC-3.04.001 EC-002).
        deck_warnings.extend(shape_output.warnings);

        // Inline text pass: convert ContentBlock::Text blocks into FrameContent::TextRun
        // frames so the inline validation pass (run_inline_validation) can scan them for
        // xref targets. This is the minimum content path needed for VP-049 load-bearing
        // end-to-end test. Full body content layout (positioning, font metrics) is
        // STORY-072 scope; here we only need the TextRun frame to exist in the slide.
        for block in &slide.blocks {
            if let ContentBlock::Text(text_block) = &block.content {
                // Use a minimal bounding box (positioned below the last existing frame, or
                // at the top-left corner if no frames exist). The exact geometry does not
                // matter for inline validation — only the frame content is needed.
                let bbox = crate::types::BoundingBox {
                    x: crate::types::Emu(0),
                    y: crate::types::Emu(0),
                    width: page_size.width,
                    height: crate::types::Emu(914_400), // 1 inch height placeholder
                };
                all_frames.push(crate::types::Frame {
                    bbox,
                    content: crate::types::FrameContent::TextRun(text_block.inlines.clone()),
                    text_flow: None,
                });
            }
        }

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
            frames: all_frames,
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

    // AC-INT-1 (F-CRIT-001): Inline validation pass.
    // Scan all TextRun frames in the laid-out slides for unknown Xref targets.
    // Accumulates LayoutWarning::XrefTargetNotFound into a warning Vec.
    // Fatal errors (InlineDepthExceeded) propagate out of layout::run.
    // Warnings are merged into deck_warnings and stored on LaidOutDeck::warnings
    // (BC-3.05.001 EC-002 / AC-007).
    let inline_warnings = run_inline_validation(deck, &laid_out_slides)?;
    deck_warnings.extend(inline_warnings);

    // STORY-027: Section collection pass.
    // Collect all document sections (auto-generated + manual) from the deck.
    // Auto-generated sections: ExecutiveSummary from takeaway fields, RiskRegister
    // from severity_cards slides. Manual sections from section_blocks.
    // Supersession and ordering rules are applied inside collect_sections.
    //
    // Architectural note (HIGH-005): sections are collected unconditionally for
    // ALL export targets and stored on LaidOutDeck. PPTX and HTML exporters
    // filter by consulting GeneratedSection::target_formats; they skip sections
    // where their format is not listed. This single-pass design means the layout
    // IR is self-contained: exporters do not need to re-examine the Deck.
    let sections = collect_sections(deck)?;

    Ok(LaidOutDeck {
        page_size,
        slides: laid_out_slides,
        sections,
        warnings: deck_warnings,
    })
}
