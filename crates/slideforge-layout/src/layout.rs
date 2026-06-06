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
    AltText, Brand, BulletItem, ContentBlock, Deck, FieldValue, Register, Value,
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
        // DEFERRED: brand-aware em conversion requires a `BrandFonts.font_size_emu`
        // field that does not yet exist on the type. `DEFAULT_EM_IN_EMU` (457_200 EMU
        // = 0.5 inch at 36pt) is used as a safe constant until that field is added.
        // Tracked: STORY-074 (brand-em-sizing) will add `BrandFonts.font_size_emu`
        // and wire it here. BC-3.04.001 PC-2 mandates brand-aware em resolution;
        // this deferral is structural (missing type field), not a design choice.
        // Pass frames.len() as base_index so InvalidBoundingBox.frame_index is
        // slide-wide (region frames + shape-list position) rather than a local
        // sub-list index (F-P20-LOW-002 / BC-3.06.003).
        let shape_output = layout_shapes(
            &shape_specs,
            page_size,
            source_index,
            DEFAULT_EM_IN_EMU,
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
        // into FrameContent::TextRun frames so the inline validation pass
        // (run_inline_validation) can scan them for xref targets and depth violations
        // (BC-3.05.001 EC-002 / AC-003 / STORY-073).
        //
        // For ContentBlock::Text: one TextRun frame per block.
        // For ContentBlock::Bullets: one TextRun frame per BulletItem (parent before
        // children, depth-first order). Nesting is structural via BulletItem.children;
        // layout preserves the flat frame sequence for exporters.
        for block in &slide.blocks {
            match &block.content {
                ContentBlock::Text(text_block) => {
                    // Clamp the placeholder height to page_height so the bbox always
                    // passes is_valid (F-P4-LOW-001 / BC-3.06.003). For brands with a
                    // canvas_height < 914_400 EMU the unclamped height would violate
                    // y + height <= page_height, triggering the InvalidBoundingBox
                    // defensive check below.
                    let placeholder_height = crate::types::Emu(914_400).min(page_size.height);
                    let bbox = crate::types::BoundingBox {
                        x: crate::types::Emu(0),
                        y: crate::types::Emu(0),
                        width: page_size.width,
                        height: placeholder_height,
                    };
                    // BC-3.06.003 defensive check: the clamped bbox must still satisfy
                    // all invariants (non-zero dimensions, within page bounds).
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
                        content: crate::types::FrameContent::TextRun(text_block.inlines.clone()),
                        text_flow: None,
                    });
                },
                ContentBlock::Bullets(items) => {
                    // STORY-073 / AC-001 — produce one FrameContent::TextRun per
                    // BulletItem, recursively visiting children depth-first.
                    // Source order is preserved: parent frame before child frames.
                    // The inline content (BulletItem.inlines) is carried verbatim —
                    // no inline processing occurs at layout time (BC-3.05.001 invariant 6).
                    push_bullet_frames(items, &mut all_frames, page_size, source_index)?;
                },
                // Other ContentBlock variants (Chart, Diagram, Shape, Math, Image, Table)
                // are handled elsewhere (shape pass above) or do not carry InlineNode
                // content that needs layout-time TextRun frames.
                _ => {},
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

/// Recursively emit one [`crate::types::FrameContent::TextRun`] frame per
/// [`BulletItem`], depth-first.
///
/// Traversal order: parent item then children (in source order), recursively.
/// This produces a flat `Vec<Frame>` sequence that preserves the source order of
/// all bullet items including nested sub-bullets (STORY-073 / AC-001 / EC-003).
///
/// Each frame carries the bullet item's `inlines` sequence verbatim — no inline
/// processing occurs at layout time (BC-3.05.001 invariant 6).
///
/// The bounding box for each frame is a full-width placeholder clamped to the
/// page height (same rule as `ContentBlock::Text` frames — BC-3.06.003 / F-P4-LOW-001).
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
/// - `Err(LayoutError::InvalidBoundingBox)` — the clamped placeholder bbox
///   violates BC-3.06.003 invariants (should never trigger in practice).
fn push_bullet_frames(
    items: &[BulletItem],
    frames: &mut Vec<crate::types::Frame>,
    page_size: PageSize,
    source_slide_index: usize,
) -> Result<(), LayoutError> {
    push_bullet_frames_inner(items, frames, page_size, source_slide_index, 0)
}

/// Inner recursive implementation for [`push_bullet_frames`] with an explicit
/// `current_depth` parameter for the structural depth bound (F-P1-MED-001).
fn push_bullet_frames_inner(
    items: &[BulletItem],
    frames: &mut Vec<crate::types::Frame>,
    page_size: PageSize,
    source_slide_index: usize,
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
        // Clamp placeholder height to page height — same rule as ContentBlock::Text
        // (F-P4-LOW-001 / BC-3.06.003).
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
        // Produce one TextRun frame carrying BulletItem.inlines verbatim.
        frames.push(crate::types::Frame {
            bbox,
            content: crate::types::FrameContent::TextRun(item.inlines.clone()),
            text_flow: None,
        });
        // Recurse into children (depth-first, source order), incrementing depth.
        push_bullet_frames_inner(
            &item.children,
            frames,
            page_size,
            source_slide_index,
            current_depth + 1,
        )?;
    }
    Ok(())
}
