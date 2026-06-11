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

use slideforge_types::{
    AltText, Brand, BulletItem, ContentBlock, Deck, FieldValue, Register, TextTag, Value,
};

use crate::error::LayoutError;

/// Maximum allowed structural nesting depth for `BulletItem.children` chains
/// (F-P1-MED-001 / BC-3.05.001 invariant 4 — bullet structural depth analogue).
///
/// Mirrors [`crate::inline::MAX_INLINE_DEPTH`] (64). Bullet *structural* depth
/// is the length of the `BulletItem → children[0] → children[0] → ...` chain,
/// counted as the number of recursive levels. A depth-65 chain (one beyond this
/// limit) returns `LayoutError::BulletDepthExceeded` before recursing further,
/// preventing stack overflow on adversarially-deep inputs.
///
/// This constant is a Kani candidate (VP-045 analogue for structural depth):
/// the proof would bound the children chain length and verify that traversal
/// always terminates within `MAX_BULLET_DEPTH` frames.
pub const MAX_BULLET_DEPTH: usize = 64;

/// Maximum number of `weighted_composite` component rows that layout will
/// route into Generic-role region slots.
///
/// `slideforge-layout::regions` pre-allocates exactly 5 Generic-role frames
/// for `weighted_composite` (component row slots 0–4). A 6th (or later)
/// `TextTag::Body` block finds all Generic Empty slots consumed and would
/// otherwise fall through to the Phase-3 append fallback, producing a stray
/// full-page-bbox frame with `region_role: None`.
///
/// BC-1.17.003 PC-9 mandates silent drop: components beyond index 4 produce
/// no additional frame. Layout enforces this by counting filled Generic-role
/// Body frames and skipping the `fill_region_slot_or_append` call once the
/// cap is reached. (F-087-P7-001)
///
/// This constant must stay in sync with the 5 Generic-role slot count in
/// `slideforge-layout::regions` (`weighted_composite` arm).
const MAX_WEIGHTED_COMPOSITE_COMPONENT_ROWS: usize = 5;

/// Horizontal x-offset applied per bullet nesting level (depth > 0).
///
/// Value: 457,200 EMU = 0.5 inch at 914,400 EMU/inch.
/// Each depth increment shifts the bullet frame's left edge right by this amount,
/// producing the standard PPTX-style indentation for nested bullets
/// (F-094-P2-001 depth-indentation requirement).
///
/// A bullet at depth 0 has x = `body_bbox.x` (no indentation).
/// A bullet at depth 1 has x = `body_bbox.x` + `BULLET_DEPTH_INDENT_EMU`.
/// A bullet at depth N has x = `body_bbox.x` + N * `BULLET_DEPTH_INDENT_EMU`.
const BULLET_DEPTH_INDENT_EMU: i64 = 457_200;

use crate::inline::run_inline_validation;
use crate::regions::region_frames_for;
use crate::sections::collect_sections;
use crate::shapes::layout_shapes;
use crate::text_flow::{LINE_HEIGHT_EMU, compute_text_flow};
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
// `layout::run` exceeds the `clippy::too_many_lines` threshold by design. The function
// is the single semantic→geometric IR boundary in the pipeline (AC-008); every slide-
// level concern (page size derivation, region frame allocation, bbox validation, shape
// layout, alt-text threading, TextTag-driven region-slot filling, register copying,
// speaker-note derivation, section collection) must be sequenced here in a documented
// order. Extracting sub-passes into helpers would scatter the invariant ordering across
// multiple call sites and reduce readability for the audit trail. The `allow` is
// justified by the function's deliberately monolithic contract, not as a deferral.
#[allow(clippy::too_many_lines)]
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
        // start as Empty/Image/Chart/Diagram placeholders. Wiring of
        // body/title/chart/image blocks into FrameContent is Stage 2b (ADR-019).
        // See `slideforge-eval::field_to_block::thread_fields_to_blocks`.
        // The region map establishes the geometric foundation; content
        // is threaded from Slide.blocks by thread_media_alt_into_frames below.
        //
        // MED-002: Compute text_flow for text-bearing frames.
        // For title/subtitle frames, extract text from the slide's resolved fields
        // to enable the canvas overflow validator (BC-3.03.001).
        //
        // ALLOWLIST GUARANTEE — BC-1.14.004 no-bleed (STORY-035 F-035-P2-002):
        // Register keys (`notes`, `report`, `detail`) intentionally REMAIN in
        // `slide.fields` after eval; they are NOT removed by the evaluator.
        // Frame construction here is ALLOWLIST-based: it reads ONLY the keys
        // `"title"`, `"subtitle"`, and `"body"` from `slide.fields`. Register
        // keys are never read at this site. Future slide-type authors and body-
        // layout pass implementers MUST follow the same allowlist discipline —
        // do NOT enumerate all `slide.fields` keys or pattern-match on arbitrary
        // fields, or register content will silently bleed into frames.
        // The `test_f004_no_bleed_register_text_not_in_frames` test enforces this.
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
        // Warnings (off-canvas positions) are non-fatal; errors (MissingAlt)
        // are fatal and propagate out of layout::run. UnknownShapeType is
        // unreachable here — ShapeSpec.shape_type is a resolved ShapeType enum,
        // so unknown keywords are rejected at the parse stage (E-PAR-012) before
        // a ShapeSpec is ever constructed.
        //
        // STORY-074 (brand-em-sizing) CLOSED: `BrandFonts.font_size_emu` is now
        // wired here (BC-3.04.001 PC-2 — brand-aware em resolution). The historical
        // `DEFAULT_EM_IN_EMU` constant (457_200 EMU = 36pt body font = 0.5 inch at 914_400 EMU/inch) is no longer
        // used in production code; `BrandFonts::default()` carries the same value for
        // backward compatibility (AC-003).
        //
        // Pass frames.len() as base_index so InvalidBoundingBox.frame_index is
        // slide-wide (region frames + shape-list position) rather than a local
        // sub-list index (F-P20-LOW-002 / BC-3.06.003).
        let shape_output = layout_shapes(
            &shape_specs,
            page_size,
            source_index,
            brand.fonts.font_size_emu,
            frames.len(),
        )?;

        let mut all_frames = frames;

        // STORY-039 AC-005 / EC-006 / EC-007 — Alt-threading pass.
        // Delegates to `thread_media_alt_into_frames` to stay within the
        // `clippy::too_many_lines` budget for `run`.
        thread_media_alt_into_frames(&slide.blocks, &mut all_frames, source_index, keyword_str);

        all_frames.extend(shape_output.frames);
        // Merge shape-level warnings (off-canvas) into the deck-level sink.
        // They are stored on LaidOutDeck::warnings (BC-3.04.001 EC-002).
        deck_warnings.extend(shape_output.warnings);

        // Inline text pass: convert ContentBlock::Text and ContentBlock::Bullets blocks
        // into layout frames, respecting RegionRole-driven region-slot filling.
        //
        // T6b.1 / AC-023 / F-086-P3-HIGH-001 / BC-4.01.001 v1.2 / ADR-019 Decision 3:
        // For tagged text blocks (Title / Subtitle / Body), the implementation FILLS the
        // matching FrameContent::Empty region slot pre-allocated by region_frames_for,
        // preserving that slot's authored bbox and computing its text_flow in-place.
        // Slot selection is ROLE-DRIVEN: each TextTag maps to a RegionRole; the search
        // finds the Empty slot with matching RegionRole, not "first Empty" by position.
        // This guarantees that TextTag::Title always fills the Title-region slot regardless
        // of block processing order (AC-023 / EC-007 / BC-4.01.001 v1.2 invariant 5).
        // A new full-page-bbox frame is NEVER appended when a pre-allocated region slot
        // can absorb the tagged content.
        //
        // Fallback (documented — fires only when no role-matched or generic slot remains):
        // If a slide type genuinely has no remaining Empty region slot matching the tag's
        // role (e.g., a future custom slide type with all slots already filled), the
        // implementation falls back to appending a clamped full-page-bbox frame. This
        // keeps the pipeline non-fatal for unknown extension cases.
        //
        // For ContentBlock::Bullets: one TextRun frame per BulletItem (parent before
        // children, depth-first order). Nesting is structural via BulletItem.children;
        // layout preserves the flat frame sequence for exporters.
        for block in &slide.blocks {
            match &block.content {
                ContentBlock::Text(text_block) => {
                    match text_block.tag {
                        // ── Tagged blocks: fill pre-allocated region slot in-place ──────
                        // Role-driven, NOT position-driven (AC-023 / F-086-P3-HIGH-001 / BC-4.01.001 v1.2 PC-12).
                        // Each tag variant passes its TextTag to fill_region_slot_or_append, which
                        // maps the tag to a RegionRole and searches for the Empty slot carrying that role.
                        // This guarantees that TextTag::Title always fills the Title-role slot regardless
                        // of block processing order — a Body block arriving before Title cannot claim
                        // the Title-role slot. Fallback to Generic/append only when no role-matched slot remains.
                        TextTag::Title => {
                            let text = extract_inline_text_str(&text_block.inlines);
                            let content = crate::types::FrameContent::Title(text);
                            fill_region_slot_or_append(
                                &mut all_frames,
                                TextTag::Title,
                                content,
                                &text_block.inlines,
                                page_size,
                                source_index,
                            )?;
                        },
                        TextTag::Subtitle => {
                            // STORY-081 C3: preserve inline structure for subtitles.
                            // When the subtitle contains inline markup nodes (produced by
                            // eval when `FieldValue::Inlines` is set for the subtitle field),
                            // emit FrameContent::SubtitleInlines to carry the rich structure.
                            // Plain-text subtitles (all Plain nodes, or a single Plain leaf)
                            // continue to use FrameContent::Subtitle(Arc<str>) for backward compat.
                            let content = if text_block.inlines.iter().any(has_non_plain_inline) {
                                crate::types::FrameContent::SubtitleInlines(
                                    text_block.inlines.clone(),
                                )
                            } else {
                                let text = extract_inline_text_str(&text_block.inlines);
                                crate::types::FrameContent::Subtitle(text)
                            };
                            fill_region_slot_or_append(
                                &mut all_frames,
                                TextTag::Subtitle,
                                content,
                                &text_block.inlines,
                                page_size,
                                source_index,
                            )?;
                        },
                        TextTag::Body => {
                            // BC-1.17.003 PC-9 / F-087-P7-001: for `weighted_composite`,
                            // cap component rows at MAX_WEIGHTED_COMPOSITE_COMPONENT_ROWS (5).
                            // The region map pre-allocates exactly 5 Generic-role Empty slots;
                            // once all 5 are filled, additional Body blocks would exhaust all
                            // Empty slots and trigger Phase-3 (appending a stray full-page-bbox
                            // frame with region_role: None). BC-1.17.003 PC-9 mandates silent
                            // drop: components beyond index 4 produce no additional frame.
                            if keyword_str == "weighted_composite" {
                                let filled_generic_body_count = all_frames
                                    .iter()
                                    .filter(|f| {
                                        f.region_role == Some(crate::types::RegionRole::Generic)
                                            && matches!(
                                                f.content,
                                                crate::types::FrameContent::Body(_)
                                            )
                                    })
                                    .count();
                                if filled_generic_body_count
                                    >= MAX_WEIGHTED_COMPOSITE_COMPONENT_ROWS
                                {
                                    // Silent drop: all 5 Generic slots consumed.
                                    // No Phase-3 append; no error. Per BC-1.17.003 PC-9.
                                    continue;
                                }
                            }
                            // Body carries ContentBlock items for rich body content.
                            // Wrap the text block's content as a single ContentBlock::Text.
                            // The PPTX serializer's extract_body_text traverses these ContentBlocks.
                            let content =
                                crate::types::FrameContent::Body(vec![ContentBlock::Text(
                                    text_block.clone(),
                                )]);
                            fill_region_slot_or_append(
                                &mut all_frames,
                                TextTag::Body,
                                content,
                                &text_block.inlines,
                                page_size,
                                source_index,
                            )?;
                        },
                        // STORY-087 pass-2: ColorLabel routes to the Body-role region slot.
                        // Same slot-selection logic as TextTag::Body (fill_region_slot_or_append
                        // maps both to RegionRole::Body per adjudication §4.6), but the
                        // TextTag::ColorLabel value is passed through to preserve semantic
                        // intent for exporters (label-specific font styling, WCAG co-encoding).
                        // BC-1.17.001 PC-8 / BC-1.17.002 PC-9 / BC-1.17.003 PC-9.
                        TextTag::ColorLabel => {
                            let content =
                                crate::types::FrameContent::Body(vec![ContentBlock::Text(
                                    text_block.clone(),
                                )]);
                            fill_region_slot_or_append(
                                &mut all_frames,
                                TextTag::ColorLabel,
                                content,
                                &text_block.inlines,
                                page_size,
                                source_index,
                            )?;
                        },
                        // ── Untagged blocks: always append a TextRun frame (unchanged) ──
                        // TextTag::Untagged has no semantic placeholder — never consumes
                        // a pre-allocated region slot. Uses a clamped full-page-bbox.
                        TextTag::Untagged => {
                            // Clamp the placeholder height to page_height so the bbox always
                            // passes is_valid (F-P4-LOW-001 / BC-3.06.003). For brands with a
                            // canvas_height < 914_400 EMU the unclamped height would violate
                            // y + height <= page_height.
                            let placeholder_height =
                                crate::types::Emu(914_400).min(page_size.height);
                            let bbox = crate::types::BoundingBox {
                                x: crate::types::Emu(0),
                                y: crate::types::Emu(0),
                                width: page_size.width,
                                height: placeholder_height,
                            };
                            // BC-3.06.003 defensive check.
                            let frame_index = all_frames.len();
                            if !bbox.is_valid(page_size.width, page_size.height) {
                                return Err(LayoutError::InvalidBoundingBox {
                                    source_slide_index: source_index,
                                    frame_index,
                                    bbox,
                                });
                            }
                            all_frames.push(crate::types::Frame {
                                bbox,
                                content: crate::types::FrameContent::TextRun(
                                    text_block.inlines.clone(),
                                ),
                                text_flow: None,
                                region_role: None,
                            });
                        },
                    }
                },
                ContentBlock::Bullets(items) if items.is_empty() => {
                    // EC-001 (STORY-073): an empty bullet list produces zero frames,
                    // no error, and no slot consumption — regardless of slide type.
                    // The slot search and E-LAY-008 error path must NOT fire for
                    // empty bullet lists (test_bc_3_05_001_story073_ec001).
                },
                ContentBlock::Bullets(items) => {
                    // F-094-P2-001/002/003 — per-bullet vertical flow + E-LAY-008 error.
                    //
                    // Slot search order (unchanged from P1):
                    //   Phase 1a: Empty Body-role slot — consume it (normal path).
                    //   Phase 1b: Already-filled Body-role slot — borrow bbox, shrink to content,
                    //             start bullet cursor after body content (F-094-P2-002).
                    //   Phase 2:  Empty Generic/None slot — consume it (multi-body fallback).
                    //   Phase 3:  None — Err(BulletsOnContentlessSlideType) (F-094-P2-003).
                    //
                    // Per-bullet vertical flow (F-094-P2-001):
                    //   Each bullet gets a distinct y derived from a flow cursor starting at
                    //   `initial_y_cursor` and advancing by LINE_HEIGHT_EMU per item.
                    //   Depth-indented children get x += BULLET_DEPTH_INDENT_EMU * depth.
                    //
                    // F-094-P1-002 / F-094-P2-003: contentless slide types (title, closing,
                    //   section_break, blank) return Err(BulletsOnContentlessSlideType)
                    //   per E-LAY-008 — NOT InvalidBoundingBox (reserved for geometry bugs).
                    enum BulletSlot {
                        /// Body region bbox + starting y cursor.
                        Region(crate::types::BoundingBox, crate::types::Emu),
                        /// No content region — fire E-LAY-008.
                        Contentless,
                    }

                    let slot: BulletSlot = {
                        // Phase 1a: Empty Body-role match — consume the slot.
                        let exact_idx = all_frames.iter().position(|f| {
                            matches!(f.content, crate::types::FrameContent::Empty)
                                && f.region_role == Some(crate::types::RegionRole::Body)
                        });
                        if let Some(idx) = exact_idx {
                            let bbox = all_frames[idx].bbox;
                            all_frames.remove(idx);
                            // Bullets start at the top of the body region.
                            BulletSlot::Region(bbox, bbox.y)
                        } else {
                            // Phase 1b: already-filled Body-role slot (F-094-P1-003 / F-094-P2-002).
                            // The Body frame has already consumed the Empty Body slot.
                            // Shrink the body frame to its content height and start bullet
                            // cursor immediately after the body content.
                            let filled_body_idx = all_frames.iter().position(|f| {
                                matches!(f.content, crate::types::FrameContent::Body(_))
                                    && f.region_role == Some(crate::types::RegionRole::Body)
                            });
                            if let Some(idx) = filled_body_idx {
                                // Compute body content height from text_flow (F-094-P2-002).
                                // text_flow was set by fill_region_slot_or_append; if absent
                                // (edge case: Body frame with no text_flow), use LINE_HEIGHT_EMU.
                                let body_line_count = all_frames[idx]
                                    .text_flow
                                    .as_ref()
                                    .map_or(1, |tf| i64::from(tf.line_count))
                                    .max(1);
                                let body_content_height = crate::types::Emu(
                                    LINE_HEIGHT_EMU.0.saturating_mul(body_line_count),
                                );
                                // Shrink body frame bbox height to content extent
                                // (clamped to available region height so we never exceed bounds).
                                let body_region_height = all_frames[idx].bbox.height;
                                let shrunk_height = body_content_height.min(body_region_height);
                                // shrunk_height must be > 0 per BC-3.06.003.
                                // LINE_HEIGHT_EMU > 0 and body_line_count >= 1 guarantee this.
                                all_frames[idx].bbox.height = shrunk_height;

                                let body_bbox = all_frames[idx].bbox;
                                // Bullet cursor starts at bottom of shrunk body frame.
                                let initial_y = crate::types::Emu(
                                    body_bbox.y.0.saturating_add(shrunk_height.0),
                                );
                                BulletSlot::Region(body_bbox, initial_y)
                            } else {
                                // Phase 2: Generic/None fallback (mirrors fill_region_slot_or_append).
                                let generic_idx = all_frames.iter().position(|f| {
                                    matches!(f.content, crate::types::FrameContent::Empty)
                                        && matches!(
                                            f.region_role,
                                            Some(crate::types::RegionRole::Generic) | None
                                        )
                                });
                                if let Some(idx) = generic_idx {
                                    let bbox = all_frames[idx].bbox;
                                    all_frames.remove(idx);
                                    // Bullets start at the top of the generic region.
                                    BulletSlot::Region(bbox, bbox.y)
                                } else {
                                    // Phase 3: no content region for bullets (F-094-P2-003 / E-LAY-008).
                                    // Return BulletsOnContentlessSlideType, NOT InvalidBoundingBox.
                                    // InvalidBoundingBox is reserved for internal geometry invariant
                                    // violations; E-LAY-008 is the user-authoring error for this case.
                                    BulletSlot::Contentless
                                }
                            }
                        }
                    };

                    match slot {
                        BulletSlot::Contentless => {
                            return Err(LayoutError::BulletsOnContentlessSlideType {
                                slide_type: Arc::clone(&slide_type_keyword),
                                source_slide_index: source_index,
                                span: block.span.clone(),
                            });
                        },
                        BulletSlot::Region(body_bbox, initial_y) => {
                            // STORY-073 / AC-001 / F-094-P2-001 — produce one FrameContent::TextRun
                            // per BulletItem with a vertical flow cursor so each bullet occupies
                            // a distinct y position (not all stacked at the same coordinates).
                            let mut y_cursor = initial_y;
                            push_bullet_frames(
                                items,
                                &mut all_frames,
                                page_size,
                                source_index,
                                body_bbox,
                                &mut y_cursor,
                            )?;
                        },
                    }
                },
                // Other ContentBlock variants (Chart, Diagram, Shape, Math, Image, Table)
                // are handled elsewhere (shape pass above) or do not carry InlineNode
                // content that needs layout-time TextRun frames.
                _ => {},
            }
        }

        // STORY-087 pass-2: ColorBar materialization pass.
        //
        // After the ContentBlock loop has filled all Text/Bullets slots, process
        // any ContentBlock::ColorBar in the block list. The pass finds the first
        // FrameContent::Empty frame with RegionRole::Generic (the bar-background slot
        // for progress_bar slides) and replaces it with FrameContent::ColorBar{...}.
        //
        // Geometry (integer EMU — no f64, BC-3.06.003 exact arithmetic):
        //   filled_width_emu = (percent as i64 * total_width_emu.0) / 100
        //
        // Color: brand.palette.primary parsed as #RRGGBB hex; falls back to
        // Rgb { r: 0, g: 112, b: 192 } (#0070C0, blue) on parse failure or absence.
        //
        // Blast radius: zero on all existing slide types — no other type produces
        // ContentBlock::ColorBar. The pass is a no-op unless the block list contains
        // a ColorBar block. (Adjudication §4.4.)
        for block in &slide.blocks {
            if let ContentBlock::ColorBar(spec) = &block.content {
                // Find the first Empty Generic-role slot (bar-background frame).
                let generic_slot_idx = all_frames.iter().position(|f| {
                    matches!(f.content, crate::types::FrameContent::Empty)
                        && f.region_role == Some(crate::types::RegionRole::Generic)
                });
                if let Some(idx) = generic_slot_idx {
                    let total_width = all_frames[idx].bbox.width;
                    let filled_width =
                        crate::types::Emu((i64::from(spec.percent) * total_width.0) / 100);
                    let color = parse_hex_color(brand.palette.primary.as_ref()).unwrap_or(
                        crate::types::Rgb {
                            r: 0,
                            g: 112,
                            b: 192,
                        },
                    );
                    all_frames[idx].content = crate::types::FrameContent::ColorBar {
                        filled_width_emu: filled_width,
                        total_width_emu: total_width,
                        percent: spec.percent,
                        color,
                    };
                }
                // At most one ColorBar block per slide (progress_bar has one value field).
                break;
            }
        }

        // BC-1.14.001/002/003 (STORY-035): Copy register-gated content from the
        // semantic slide IR. `slide.register_content` was populated by
        // `slideforge-eval::register_routing::extract_register_content` during
        // `eval_deck` (after all field expressions were resolved). Layout copies
        // it verbatim — zero routing logic is performed here (Option D,
        // architecture-directive STORY-035). The result is ordered
        // Notes < Report < Detail (BC-1.14.004 invariant 3).
        let register_content = slide.register_content.clone();

        // BC-1.14.004 (STORY-035 F-004): derive `speaker_notes` from `register_content`.
        // See `speaker_notes_from_register_content` for the derivation rationale.
        // Must come AFTER register_content is bound above.
        let speaker_notes = speaker_notes_from_register_content(&register_content);

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
            register_content,
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
        // STORY-082 CRIT-2: slide_sections was populated by the evaluator
        // (slideforge-eval::section_groups::build_slide_sections_from_membership)
        // and stored on Deck::slide_sections. Pass it through to LaidOutDeck here
        // so that the PPTX exporter can read it without re-examining the Deck or DeckNode.
        slide_sections: deck.slide_sections.clone(),
    })
}

/// Thread author-supplied alt text from semantic blocks into region-map frame placeholders.
///
/// # STORY-039 AC-005 / EC-006 / EC-007
///
/// Region-map frames for `chart`, `diagram`, and `image`/`screenshot`/`bio` slide
/// types carry `AltText::Unspecified` as a structural placeholder (ADR-019 Decision 5.1).
/// This function reads each `ContentBlock::Chart` / `Diagram` / `Image` block's `.alt`
/// field and overwrites the corresponding placeholder frame.
///
/// ## Mapping rule (lossless — no normalisation, no truncation)
///
/// | `spec.alt` value       | Frame alt written    | Side-effect              |
/// |------------------------|----------------------|--------------------------|
/// | `Some(Provided(s))`    | `Provided(s)`        | none                     |
/// | `Some(Decorative)`     | `Decorative`         | none                     |
/// | `None`                 | `Unspecified`        | `tracing::warn!` (EC-006/EC-007): no author alt threaded |
///
/// ## Scope discipline
///
/// ONLY the `alt` field is written. The SVG payload of `FrameContent::Diagram`
/// and any image-path data are not populated here — those are separate pipeline
/// concerns (STORY-027 / future chart-content scope).
///
/// ## Matching strategy
///
/// For each block, the FIRST frame of the matching `FrameContent` variant in
/// `frames` is updated. Each built-in slide type has at most one `Chart`,
/// `Diagram`, or `Image` frame from the region map, so first-match is correct.
/// Shape frames (appended after this pass) never carry these variants, so there
/// is no collision risk.
fn thread_media_alt_into_frames(
    blocks: &[slideforge_types::Block],
    frames: &mut [crate::types::Frame],
    source_slide_index: usize,
    slide_type: &str,
) {
    use slideforge_types::Block;

    for Block { content, .. } in blocks {
        match content {
            ContentBlock::Chart(spec) => {
                let resolved_alt = if let Some(alt) = &spec.alt {
                    alt.clone()
                } else {
                    tracing::warn!(
                        source_slide_index,
                        slide_type,
                        "EC-006: ChartSpec.alt is None — no author alt text was threaded \
                         into this ContentBlock; emitting AltText::Unspecified; \
                         validate_post_layout will emit E-A11-001 in strict mode (ADR-019 Decision 5.2)"
                    );
                    AltText::Unspecified
                };
                match frames
                    .iter_mut()
                    .find(|f| matches!(f.content, FrameContent::Chart { .. }))
                {
                    Some(frame) => {
                        frame.content = FrameContent::Chart { alt: resolved_alt };
                    },
                    None => {
                        tracing::warn!(
                            source_slide_index,
                            slide_type,
                            "ContentBlock::Chart present but no matching Chart region frame \
                             found — author alt text silently dropped"
                        );
                    },
                }
            },
            ContentBlock::Diagram(spec) => {
                let resolved_alt = if let Some(alt) = &spec.alt {
                    alt.clone()
                } else {
                    tracing::warn!(
                        source_slide_index,
                        slide_type,
                        "EC-007: DiagramSpec.alt is None — no author alt text was threaded \
                         into this ContentBlock; emitting AltText::Unspecified; \
                         validate_post_layout will emit E-A11-001 in strict mode (ADR-019 Decision 5.2)"
                    );
                    AltText::Unspecified
                };
                match frames
                    .iter_mut()
                    .find(|f| matches!(f.content, FrameContent::Diagram { .. }))
                {
                    Some(frame) => {
                        // Update only the alt field; preserve the existing SVG.
                        if let FrameContent::Diagram { alt, .. } = &mut frame.content {
                            *alt = resolved_alt;
                        }
                    },
                    None => {
                        tracing::warn!(
                            source_slide_index,
                            slide_type,
                            "ContentBlock::Diagram present but no matching Diagram region frame \
                             found — author alt text silently dropped"
                        );
                    },
                }
            },
            ContentBlock::Image(spec) => {
                let resolved_alt = if let Some(alt) = &spec.alt {
                    alt.clone()
                } else {
                    tracing::warn!(
                        source_slide_index,
                        slide_type,
                        "ImageSpec.alt is None — no author alt text was threaded \
                         into this ContentBlock; emitting AltText::Unspecified; \
                         validate_post_layout will emit E-A11-001 in strict mode (ADR-019 Decision 5.2)"
                    );
                    AltText::Unspecified
                };
                match frames
                    .iter_mut()
                    .find(|f| matches!(f.content, FrameContent::Image { .. }))
                {
                    Some(frame) => {
                        frame.content = FrameContent::Image { alt: resolved_alt };
                    },
                    None => {
                        tracing::warn!(
                            source_slide_index,
                            slide_type,
                            "ContentBlock::Image present but no matching Image region frame \
                             found — author alt text silently dropped"
                        );
                    },
                }
            },
            // Other ContentBlock variants are handled in their respective passes.
            _ => {},
        }
    }
}

/// Derive `speaker_notes` from the canonical `register_content` vector.
///
/// # Derivation rationale (BC-1.14.004 / STORY-035 F-004)
///
/// `register_content` (populated by `eval_deck` via
/// `slideforge-eval::register_routing::extract_register_content`) is the CANONICAL
/// source for all register-gated content. `speaker_notes` is a PPTX/HTML convenience
/// field that must be derived from the Notes entry in `register_content` — NOT read
/// independently from `slide.fields`. This ensures there is ONE source of truth for
/// register text (BC-1.14.004 invariant 3 — no bleed between `register_content` and
/// any other field).
///
/// # Returns
///
/// - `None` if `register_content` has no Notes entry.
/// - `None` if the Notes entry has no plain-text inline nodes and no content.
/// - `Some(text)` where `text` is all [`slideforge_types::InlineNode::Plain`] nodes
///   from the Notes entry concatenated in order.
fn speaker_notes_from_register_content(
    register_content: &[slideforge_types::RegisteredContent],
) -> Option<Arc<str>> {
    register_content
        .iter()
        .find(|rc| rc.register == Register::Notes)
        .and_then(|rc| {
            let text: String = rc
                .content
                .iter()
                .filter_map(|node| {
                    if let slideforge_types::InlineNode::Plain(s) = node {
                        Some(s.as_ref())
                    } else {
                        None
                    }
                })
                .collect();
            if text.is_empty() && rc.content.is_empty() {
                None
            } else {
                Some(Arc::from(text.as_str()))
            }
        })
}

/// Return `true` if the inline slice contains any non-Plain node (i.e., has rich markup).
///
/// Used by the subtitle routing path (STORY-081 C3) to decide whether to emit
/// `FrameContent::SubtitleInlines` (rich) or `FrameContent::Subtitle` (plain).
/// A subtitle that contains only `InlineNode::Plain` nodes can be represented
/// as a plain `Arc<str>` without loss; a subtitle with Bold/Italic/Code/etc.
/// MUST be represented as `SubtitleInlines` to preserve inline structure.
pub(crate) fn has_non_plain_inline(node: &slideforge_types::InlineNode) -> bool {
    use slideforge_types::InlineNode;
    !matches!(node, InlineNode::Plain(_))
}

/// Extract plain text from a slice of [`slideforge_types::InlineNode`] values into an [`Arc<str>`].
///
/// Used by the `TextTag` routing pass to produce the `Arc<str>` carried by
/// `FrameContent::Title` and `FrameContent::Subtitle`. Nested inline formatting
/// (Bold, Italic, etc.) is flattened to plain text for these variants, which is
/// semantically correct — the placeholder text is the canonical string label;
/// inline formatting is not preserved in OOXML title placeholders.
///
/// Plain text is concatenated in source order. Math nodes are omitted.
fn extract_inline_text_str(nodes: &[slideforge_types::InlineNode]) -> Arc<str> {
    let mut out = String::new();
    for node in nodes {
        extract_inline_text_recursive(node, &mut out);
    }
    Arc::from(out.as_str())
}

/// Recursive helper for [`extract_inline_text_str`].
fn extract_inline_text_recursive(node: &slideforge_types::InlineNode, out: &mut String) {
    use slideforge_types::InlineNode;
    match node {
        InlineNode::Plain(s) | InlineNode::Code(s) | InlineNode::Xref(s) => {
            out.push_str(s);
        },
        InlineNode::Bold(children)
        | InlineNode::Italic(children)
        | InlineNode::Footnote(children)
        | InlineNode::Superscript(children)
        | InlineNode::Subscript(children)
        | InlineNode::Strikethrough(children)
        | InlineNode::Highlight(children) => {
            for child in children {
                extract_inline_text_recursive(child, out);
            }
        },
        InlineNode::Link { text, .. } => {
            for child in text {
                extract_inline_text_recursive(child, out);
            }
        },
        InlineNode::Math(_) => {
            // Math nodes are not extracted as plain text for title/subtitle frames.
        },
    }
}

/// Fill the `FrameContent::Empty` region slot whose [`crate::types::RegionRole`]
/// matches the given [`TextTag`], preserving the slot's authored bbox and computing
/// `text_flow` in-place.
///
/// # T6b.1 / AC-023 / BC-4.01.001 v1.2 / ADR-019 Decision 3
///
/// `region_frames_for` pre-allocates geometry slots as `FrameContent::Empty`, each
/// with a `region_role` identifying which semantic content it expects. This function
/// is the injection point where tagged text content (Title / Subtitle / Body) claims
/// the CORRECT slot IN-PLACE, replacing `FrameContent::Empty` with the supplied
/// `content` variant while preserving the slot's authored bbox.
///
/// ## Why this matters for visual parity
///
/// The authored bbox encodes the PPTX placeholder geometry (type, idx, position) as
/// specified in ADR-015 §7. Appending a new full-page-bbox frame instead of filling
/// the slot would discard this authored geometry and produce incorrect OOXML placeholder
/// coordinates (visual-parity-contract §Positional layout ±4pt).
///
/// ## Slot selection (role-driven, NOT position-driven) — AC-023 / F-086-P3-HIGH-001
///
/// Selection priority (checked in order):
/// 1. **Exact role match:** finds the first `FrameContent::Empty` slot whose
///    `region_role` matches the requested role (`Title` → `RegionRole::Title`,
///    `Subtitle` → `RegionRole::Subtitle`, `Body` → `RegionRole::Body`).
/// 2. **Generic fallback:** if no exact-role Empty slot exists, fills the first
///    `FrameContent::Empty` slot with `region_role == Some(RegionRole::Generic)`
///    or `region_role == None`. This handles multi-body layouts (e.g., `two_col`
///    second column) and forward-compatibility with unknown custom types.
/// 3. **Append fallback:** if no Empty slot exists at all, appends a clamped
///    full-page-bbox frame. Fires only for slide types that pre-fill all slots
///    with non-Empty content.
///
/// This role-driven selection guarantees that a `TextTag::Title` block arriving
/// after a `TextTag::Body` block in the input still claims the title-region slot
/// (AC-023 invariant 5 / EC-007).
///
/// ## Fallback (documented — fires only when no Empty slot remains)
///
/// When no `FrameContent::Empty` slot exists in `frames` (e.g., a slide type with all
/// region slots already filled by prior blocks, or a custom slide type with no Empty
/// placeholders at all), a clamped full-page-bbox frame is appended. This fallback is
/// intentional for forward-compatibility with future slide types that may pre-fill all
/// their region slots with non-Empty content.
///
/// # Errors
///
/// Returns `Err(LayoutError::InvalidBoundingBox)` if the fallback full-page-bbox fails
/// the BC-3.06.003 defensive check (should never occur in practice).
fn fill_region_slot_or_append(
    frames: &mut Vec<crate::types::Frame>,
    tag: TextTag,
    content: crate::types::FrameContent,
    inlines: &[slideforge_types::InlineNode],
    page_size: PageSize,
    source_slide_index: usize,
) -> Result<(), LayoutError> {
    // Map the TextTag to the expected RegionRole.
    let expected_role = match tag {
        TextTag::Title => crate::types::RegionRole::Title,
        TextTag::Subtitle => crate::types::RegionRole::Subtitle,
        // STORY-087 pass-2: ColorLabel maps to Body role — same slot routing as
        // TextTag::Body but with a semantically distinct tag (adjudication §4.6).
        // This ensures status/progress_bar/weighted_composite label text claims the
        // pre-allocated Body-role region slot.
        TextTag::Body | TextTag::ColorLabel => crate::types::RegionRole::Body,
        // Untagged blocks are never routed through this function — they are
        // handled separately by the TextTag::Untagged arm in layout::run.
        TextTag::Untagged => crate::types::RegionRole::Generic,
    };

    // Phase 1: exact role match — find the first Empty slot with the matching role.
    // This is the primary path for all tagged blocks (Title / Subtitle / Body).
    // Order-independent: a Body block processed before a Title block will NOT claim
    // the Title-role slot; it searches for a Body-role slot instead.
    let exact_match_idx = frames.iter().position(|f| {
        matches!(f.content, crate::types::FrameContent::Empty)
            && f.region_role == Some(expected_role)
    });

    if let Some(idx) = exact_match_idx {
        let flat_text = collect_plain_text(inlines);
        let bbox = frames[idx].bbox;
        frames[idx].text_flow = Some(compute_text_flow(&flat_text, bbox));
        frames[idx].content = content;
        return Ok(());
    }

    // Phase 2: generic fallback — fill the first Empty slot with Generic or None role.
    // Handles multi-body layouts where a second Body block fills a Generic column slot
    // (e.g., `two_col` right column) or an untyped slot in an unknown custom type.
    let generic_match_idx = frames.iter().position(|f| {
        matches!(f.content, crate::types::FrameContent::Empty)
            && matches!(
                f.region_role,
                Some(crate::types::RegionRole::Generic) | None
            )
    });

    if let Some(idx) = generic_match_idx {
        let flat_text = collect_plain_text(inlines);
        let bbox = frames[idx].bbox;
        frames[idx].text_flow = Some(compute_text_flow(&flat_text, bbox));
        frames[idx].content = content;
        return Ok(());
    }

    // Phase 3: no Empty slot of any kind remains — append a clamped full-page-bbox frame.
    // This fires only when a slide type has no pre-allocated Empty placeholders
    // (e.g., all slots are Image/Chart/Diagram, or a future custom type). It
    // preserves pipeline non-fatality for extension cases.
    let placeholder_height = crate::types::Emu(914_400).min(page_size.height);
    let bbox = crate::types::BoundingBox {
        x: crate::types::Emu(0),
        y: crate::types::Emu(0),
        width: page_size.width,
        height: placeholder_height,
    };
    // BC-3.06.003 defensive check.
    let frame_index = frames.len();
    if !bbox.is_valid(page_size.width, page_size.height) {
        return Err(LayoutError::InvalidBoundingBox {
            source_slide_index,
            frame_index,
            bbox,
        });
    }
    let flat_text = collect_plain_text(inlines);
    let text_flow = Some(compute_text_flow(&flat_text, bbox));
    frames.push(crate::types::Frame {
        bbox,
        content,
        text_flow,
        region_role: None,
    });
    Ok(())
}

/// Parse a `#RRGGBB` hex color string into an [`crate::types::Rgb`] value.
///
/// Returns `None` if the string is not a valid 7-character `#RRGGBB` hex color.
/// Used by the `ColorBar` materialization pass to derive the bar fill color from
/// the brand's primary palette color (adjudication §4.4, STORY-087).
fn parse_hex_color(hex: &str) -> Option<crate::types::Rgb> {
    let hex = hex.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(crate::types::Rgb { r, g, b })
}

/// Collect all plain-text from a slice of inline nodes into a single `String`.
///
/// Used by [`fill_region_slot_or_append`] to produce the plain-text string
/// passed to [`compute_text_flow`] when filling or appending a region slot.
/// Nested formatting (Bold, Italic, etc.) is flattened to plain text; Math
/// nodes are omitted (same semantics as [`extract_inline_text_str`]).
fn collect_plain_text(nodes: &[slideforge_types::InlineNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        extract_inline_text_recursive(node, &mut out);
    }
    out
}

/// Emit one [`crate::types::FrameContent::TextRun`] frame per [`BulletItem`],
/// depth-first, with a vertical flow cursor (F-094-P2-001).
///
/// Traversal order: parent item then children (in source order), recursively.
/// This produces a flat `Vec<Frame>` sequence that preserves the source order of
/// all bullet items including nested sub-bullets (STORY-073 / AC-001 / EC-003).
///
/// Each frame carries the bullet item's `inlines` sequence verbatim — no inline
/// processing occurs at layout time (BC-3.05.001 invariant 6).
///
/// # Per-bullet vertical flow (F-094-P2-001)
///
/// Each bullet item occupies a distinct vertical slice of `body_bbox`:
/// - `y = *y_cursor` — current cursor position.
/// - `height = LINE_HEIGHT_EMU` — one line per bullet item.
/// - `*y_cursor` advances by `LINE_HEIGHT_EMU` after each item.
/// - If `y_cursor` exceeds `body_bbox.y + body_bbox.height`, bullets are clipped
///   to the region boundary (no underflow — canvas-overflow validator territory).
///
/// # Depth indentation (F-094-P2-001)
///
/// Child bullets at `depth > 0` are indented by `BULLET_DEPTH_INDENT_EMU * depth`:
/// - `x = body_bbox.x + BULLET_DEPTH_INDENT_EMU * depth`
/// - `width = body_bbox.width - BULLET_DEPTH_INDENT_EMU * depth` (clamped >= 1)
///
/// # Pre-condition
///
/// `body_bbox` must be a valid region-map bbox (caller has already validated via
/// the slot-search logic; only regions passing `is_valid` reach this function).
///
/// # Structural depth bound (F-P1-MED-001)
///
/// The `current_depth` parameter tracks how many `children` levels have been
/// entered. When `current_depth` would exceed [`MAX_BULLET_DEPTH`], the function
/// returns `Err(LayoutError::BulletDepthExceeded { depth })` BEFORE recursing
/// further, preventing stack overflow on adversarially-deep inputs.
/// Depth 0 through [`MAX_BULLET_DEPTH`] (64) are accepted; depth 65 and above
/// are rejected.
///
/// # Errors
///
/// - `Err(LayoutError::BulletDepthExceeded { depth })` — the `BulletItem.children`
///   chain exceeds [`MAX_BULLET_DEPTH`] structural levels (F-P1-MED-001).
/// - `Err(LayoutError::InvalidBoundingBox)` — the finalized per-bullet bbox
///   violates BC-3.06.003 invariants (e.g. region map produced negative coords).
fn push_bullet_frames(
    items: &[BulletItem],
    frames: &mut Vec<crate::types::Frame>,
    page_size: PageSize,
    source_slide_index: usize,
    body_bbox: crate::types::BoundingBox,
    y_cursor: &mut crate::types::Emu,
) -> Result<(), LayoutError> {
    push_bullet_frames_inner(
        items,
        frames,
        page_size,
        source_slide_index,
        body_bbox,
        y_cursor,
        0,
    )
}

/// Inner recursive implementation for [`push_bullet_frames`] with an explicit
/// `current_depth` parameter for the structural depth bound (F-P1-MED-001).
fn push_bullet_frames_inner(
    items: &[BulletItem],
    frames: &mut Vec<crate::types::Frame>,
    page_size: PageSize,
    source_slide_index: usize,
    body_bbox: crate::types::BoundingBox,
    y_cursor: &mut crate::types::Emu,
    current_depth: usize,
) -> Result<(), LayoutError> {
    // F-P1-MED-001: reject structural nesting deeper than MAX_BULLET_DEPTH BEFORE
    // recursing. The check fires at entry so the first call at a too-deep level
    // returns the error rather than pushing frames and then recursing further.
    if current_depth > MAX_BULLET_DEPTH {
        return Err(LayoutError::BulletDepthExceeded {
            source_slide_index,
            depth: current_depth,
        });
    }

    for item in items {
        // F-094-P2-001 — compute per-bullet bbox from flow cursor + depth indentation.
        //
        // x offset: body_bbox.x + BULLET_DEPTH_INDENT_EMU * current_depth.
        // Width: body_bbox.width minus indent, clamped to >= 1 EMU (BC-3.06.003 width > 0).
        // y: current cursor value.
        // height: LINE_HEIGHT_EMU (one visual line per bullet item).
        //
        // Vertical overflow: if the cursor already exceeds the body region bottom,
        // additional bullets are placed at the region bottom with height 1 (minimal
        // valid frame). The canvas-overflow validator (E-LAY-001) will flag this
        // separately; layout does NOT silently clip or error here.
        // current_depth is always ≤ MAX_BULLET_DEPTH (64); i64::try_from is infallible
        // for any usize value that fits in 64 bits (all contemporary platforms).
        let depth_as_i64 = i64::try_from(current_depth).unwrap_or(i64::MAX);
        let depth_offset_emu =
            crate::types::Emu(BULLET_DEPTH_INDENT_EMU.saturating_mul(depth_as_i64));
        // Clamp bullet_x to [body_bbox.x, page_width - 1] so that extreme depth
        // indentation never causes is_valid to fail before the structural depth guard
        // can fire (F-094-P2-001, BC-3.06.003). The canvas-overflow validator
        // (E-LAY-001) flags visible clipping separately.
        let raw_x = body_bbox.x.0.saturating_add(depth_offset_emu.0);
        let max_x = page_size.width.0.saturating_sub(1).max(0);
        let bullet_x = crate::types::Emu(raw_x.min(max_x));
        // Width: from clamped x to the page right edge, minimum 1 EMU.
        let bullet_width = crate::types::Emu((page_size.width.0 - bullet_x.0).max(1));

        // Clamp bullet_y to [0, page_height - 1] so that overflowed y_cursor values
        // (more bullets than fit on the page) still produce a valid minimal frame.
        // The comment above documents the design intent: overflow bullets are clamped
        // to the page bottom with height 1, not rejected as InvalidBoundingBox.
        let max_y = page_size.height.0.saturating_sub(1).max(0);
        let bullet_y = crate::types::Emu(y_cursor.0.min(max_y));
        // Height: from clamped y to the page bottom edge, minimum 1 EMU.
        let bullet_height =
            crate::types::Emu((page_size.height.0 - bullet_y.0).clamp(1, LINE_HEIGHT_EMU.0));

        // BC-3.06.003 defensive check: the clamping above ensures is_valid passes
        // for any structurally valid (depth ≤ MAX_BULLET_DEPTH) input. A failure
        // here indicates a logic bug in the clamping itself.
        let bbox = crate::types::BoundingBox {
            x: bullet_x,
            y: bullet_y,
            width: bullet_width,
            height: bullet_height,
        };
        debug_assert!(
            bbox.is_valid(page_size.width, page_size.height),
            "clamped bullet bbox must be valid: {bbox:?} on page {}×{}",
            page_size.width.0,
            page_size.height.0,
        );

        // Advance cursor for the next bullet (before pushing, so child bullets
        // continue from after the parent's line).
        *y_cursor = crate::types::Emu(y_cursor.0.saturating_add(bullet_height.0));

        // Produce one TextRun frame carrying BulletItem.inlines verbatim.
        frames.push(crate::types::Frame {
            bbox,
            content: crate::types::FrameContent::TextRun(item.inlines.clone()),
            text_flow: None,
            region_role: None,
        });
        // Recurse into children (depth-first, source order), incrementing depth.
        push_bullet_frames_inner(
            &item.children,
            frames,
            page_size,
            source_slide_index,
            body_bbox,
            y_cursor,
            current_depth + 1,
        )?;
    }
    Ok(())
}
