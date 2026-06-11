//! Post-eval field-to-block threading pass (Stage 2b, ADR-019).
//!
//! This module provides [`thread_fields_to_blocks`], the single-responsibility
//! bridge between the semantic IR (field-resolved [`Deck`]) and the layout stage.
//! It reads resolved `Slide.fields` and populates `Slide.blocks` with typed
//! [`slideforge_types::ContentBlock`] entries.
//!
//! ## Pipeline position (ADR-019 Decision 1)
//!
//! ```text
//! Stage 2a: eval_deck      → Deck (fields resolved, blocks = vec![])
//! Stage 2b: thread_fields  → Deck (blocks populated)           ← THIS MODULE
//! Stage 3:  brand load
//! Stage 6:  layout         → LaidOutDeck
//! ```

use std::sync::Arc;

use slideforge_types::{
    Block, BulletItem, ColorBarSpec, ContentBlock, Deck, FieldValue, InlineNode, OrderedMap,
    SourceSpan, TextBlock, TextTag, Value,
    specs::{AltText, ChartSpec, DiagramSpec, ImageSpec},
};

/// Maximum number of component rows rendered for a `weighted_composite` slide.
///
/// `slideforge-layout` pre-allocates exactly 5 Generic-role region slots for
/// `weighted_composite` (see `slideforge_layout::regions` — "component row slot
/// 0" through "component row slot 4"). Threading more than 5 components would
/// generate Body blocks that find no pre-allocated Generic slot and fall through
/// to the layout Phase-3 fallback (appending stray full-page frames).
///
/// BC-1.17.003 PC-9 mandates silent drop: components beyond index 4 produce no
/// additional frame. This constant is the single source of truth for the cap.
/// (F-087-P7-001)
const MAX_WEIGHTED_COMPOSITE_COMPONENT_ROWS: usize = 5;

/// Post-eval field-to-block threading pass (Stage 2b, ADR-019).
///
/// Reads resolved field values from every `Slide.fields` in `deck` and
/// populates `Slide.blocks` with typed [`slideforge_types::ContentBlock`]
/// entries derived from those fields. This pass runs AFTER `eval_deck`
/// completes and BEFORE `layout::run` — it is the single-responsibility bridge
/// between the semantic IR (field-resolved `Deck`) and the geometric IR
/// (`LaidOutDeck`).
///
/// # Purity contract
///
/// This function is **pure** in the architectural sense: it performs no I/O,
/// no filesystem access, no network calls, and has no global mutable state.
/// Its output is fully determined by its input. This makes it amenable to
/// Kani bounded-model checking and property-based testing.
///
/// # Fields→Blocks mapping (ADR-019 Decision 3)
///
/// Text fields (`"title"`, `"subtitle"`, `"body"`) are mapped to
/// `ContentBlock::Text`. Bullet lists are mapped to `ContentBlock::Bullets`.
/// Media types (`"chart_type"`, `"src"`, `"source"`) produce
/// `ContentBlock::Chart`, `ContentBlock::Image`, or `ContentBlock::Diagram`
/// respectively.
///
/// Empty strings (after trim) are silently skipped — no `ContentBlock` is
/// emitted for empty-string field values (BC-1.16.001 EC-001).
///
/// # `AltText` contract (decorative-first, BC-1.16.001 PC-12)
///
/// When constructing `ContentBlock::Chart`, `ContentBlock::Image`, or
/// `ContentBlock::Diagram`, the threading pass uses the following alt-resolution
/// precedence (BC-1.16.001 PC-12 / EC-004, ADR-019 Decision 4):
///
/// 1. `Slide.fields["decorative"] == Value::Bool(true)` → `ContentBlock.alt = Some(AltText::Decorative)`
///    (wins unconditionally, even if `alt` is also set — see W-A11-002 below).
/// 2. `Slide.fields["alt"] == Value::Str(s)` (non-empty, non-whitespace) → `ContentBlock.alt = Some(AltText::Provided(Arc::from(s.trim())))`
/// 3. Neither present → `ContentBlock.alt = None`
///
/// When both `decorative: true` AND a non-empty `alt` are present, rule 1 wins
/// and `tracing::warn!(code = "W-A11-002", ...)` is emitted to surface the
/// conflict to the author (error-taxonomy v2.17 W-A11-002). This warning is
/// scoped to the MEDIA path only (chart/image/diagram); non-media slides may
/// carry inert `decorative:` / `alt:` fields without triggering it
/// (F-086-P13-OBS-001 fix).
///
/// The layout `thread_media_alt_into_frames` function then maps `None` to
/// `AltText::Unspecified` on the resulting frame (ADR-019 Decision 5).
///
/// # Block ordering (ADR-019 Decision 3.6)
///
/// Within a single slide, blocks are appended in canonical order:
/// 1. Title block (if applicable)
/// 2. Subtitle block (if applicable)
/// 3. Body block (if applicable)
/// 4. Bullets block (if applicable)
/// 5. Chart / Image / Diagram block (if applicable, at most one per slide)
///
/// Shape blocks are **not** populated by this pass (BC-1.16.001 invariant 4).
///
/// # Idempotency
///
/// If `Slide.blocks` is already non-empty for a slide, this function
/// appends to it rather than replacing it. In practice, `eval_deck`
/// always produces `Slide.blocks = vec![]`, so this is a no-op guard.
// The function is the chartered Stage 2b threading pass (ADR-019 Decision 3).
// It handles 6 logical sections (title, subtitle, body, bullets, media, color-coded
// types) each requiring their own match/if-let chains. Extracting further would
// scatter the field-threading contract across multiple files and harm readability.
// The allow is justified by the function's chartered monolithic threading contract.
#[allow(clippy::too_many_lines)]
pub fn thread_fields_to_blocks(deck: &mut Deck) {
    for slide in &mut deck.slides {
        // ── 1. Title ─────────────────────────────────────────────────────────
        // TextTag::Title is set so layout.rs routes this block to FrameContent::Title
        // instead of the generic FrameContent::TextRun (ADR-019 Decision 3 / AC-019).
        if let Some(text) = extract_str_field(slide, "title")
            && !text.trim().is_empty()
        {
            slide
                .blocks
                .push(make_text_block_tagged(text, TextTag::Title));
        }

        // ── 2. Subtitle ──────────────────────────────────────────────────────
        // TextTag::Subtitle routes to FrameContent::Subtitle (PPTX subTitle placeholder,
        // DOCX Heading2 paragraph) per BC-4.01.001 v1.2 postcondition 10 / BC-4.02.001 v1.2 PC-9.
        //
        // STORY-081 C2 fix: also handle FieldValue::Inlines (produced by eval_slide_node
        // when subtitle contains inline markup like `_italic subtitle_`). Previously
        // extract_str_field returned None for Inlines, silently dropping the subtitle.
        match slide.fields.get("subtitle") {
            Some(FieldValue::Inlines(nodes)) if !nodes.is_empty() => {
                slide.blocks.push(make_text_block_tagged_inlines(
                    nodes.clone(),
                    TextTag::Subtitle,
                ));
            },
            _ => {
                if let Some(text) = extract_str_field(slide, "subtitle")
                    && !text.trim().is_empty()
                {
                    slide
                        .blocks
                        .push(make_text_block_tagged(text, TextTag::Subtitle));
                }
            },
        }

        // ── 3. Body ──────────────────────────────────────────────────────────
        // TextTag::Body routes to FrameContent::Body (PPTX body placeholder,
        // DOCX Normal paragraph) per BC-4.01.001 v1.2 postcondition 11 / BC-4.02.001 v1.2 PC-10.
        //
        // STORY-081 C2 fix: also handle FieldValue::Inlines (produced by eval_slide_node
        // when body contains inline markup like `**bold body text**`). Previously
        // extract_str_field returned None for Inlines, silently dropping the body.
        // This is the primary failing path identified by adversary finding C2.
        match slide.fields.get("body") {
            Some(FieldValue::Inlines(nodes)) if !nodes.is_empty() => {
                slide
                    .blocks
                    .push(make_text_block_tagged_inlines(nodes.clone(), TextTag::Body));
            },
            _ => {
                if let Some(text) = extract_str_field(slide, "body")
                    && !text.trim().is_empty()
                {
                    slide
                        .blocks
                        .push(make_text_block_tagged(text, TextTag::Body));
                }
            },
        }

        // ── 4a. Caption ──────────────────────────────────────────────────────
        // TextTag::Untagged routes to FrameContent::TextRun (generic body-style).
        // STORY-081 P31-MED-001 fix: caption field was in INLINE_CONTENT_FIELDS so
        // eval_slide_node correctly produced FieldValue::Inlines — but thread_fields_to_blocks
        // had no match arm for it, silently dropping inline nodes before they reached exporters.
        match slide.fields.get("caption") {
            Some(FieldValue::Inlines(nodes)) if !nodes.is_empty() => {
                slide.blocks.push(make_text_block_tagged_inlines(
                    nodes.clone(),
                    TextTag::Untagged,
                ));
            },
            _ => {
                if let Some(text) = extract_str_field(slide, "caption")
                    && !text.trim().is_empty()
                {
                    slide
                        .blocks
                        .push(make_text_block_tagged(text, TextTag::Untagged));
                }
            },
        }

        // ── 4b. Description ──────────────────────────────────────────────────
        // TextTag::Untagged routes to FrameContent::TextRun (generic body-style).
        // STORY-081 P31-MED-001 fix: description field was in INLINE_CONTENT_FIELDS so
        // eval_slide_node correctly produced FieldValue::Inlines — but thread_fields_to_blocks
        // had no match arm for it, silently dropping inline nodes before they reached exporters.
        match slide.fields.get("description") {
            Some(FieldValue::Inlines(nodes)) if !nodes.is_empty() => {
                slide.blocks.push(make_text_block_tagged_inlines(
                    nodes.clone(),
                    TextTag::Untagged,
                ));
            },
            _ => {
                if let Some(text) = extract_str_field(slide, "description")
                    && !text.trim().is_empty()
                {
                    slide
                        .blocks
                        .push(make_text_block_tagged(text, TextTag::Untagged));
                }
            },
        }

        // ── 5. Bullets ───────────────────────────────────────────────────────
        match slide.fields.get("bullets") {
            Some(FieldValue::Literal(Value::List(items))) => {
                let bullet_items: Vec<BulletItem> = items
                    .iter()
                    .filter_map(|v| {
                        if let Value::Str(s) = v {
                            Some(BulletItem {
                                inlines: vec![InlineNode::Plain(Arc::clone(s))],
                                children: vec![],
                                span: SourceSpan::default(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                slide.blocks.push(Block {
                    content: ContentBlock::Bullets(bullet_items),
                    label: None,
                    span: SourceSpan::default(),
                });
            },
            Some(FieldValue::Inlines(nodes)) => {
                // ADR-019 Decision 3.2: FieldValue::Inlines bullets threaded as-is.
                // We don't have structured BulletItem from Inlines — emit as a single
                // BulletItem carrying the already-bound inline nodes directly, without
                // redundantly re-fetching slide.fields.get("bullets").
                slide.blocks.push(Block {
                    content: ContentBlock::Bullets(vec![BulletItem {
                        inlines: nodes.clone(),
                        children: vec![],
                        span: SourceSpan::default(),
                    }]),
                    label: None,
                    span: SourceSpan::default(),
                });
            },
            Some(FieldValue::InlinesList(items)) => {
                // STORY-081×STORY-088: list-literal bullets with inline markup.
                // Each Vec<InlineNode> in items is one bullet item with full inline
                // structure. Build BulletItem directly from the preserved nodes.
                let bullet_items: Vec<BulletItem> = items
                    .iter()
                    .filter(|item_nodes| !item_nodes.is_empty())
                    .map(|item_nodes| BulletItem {
                        inlines: item_nodes.clone(),
                        children: vec![],
                        span: SourceSpan::default(),
                    })
                    .collect();
                slide.blocks.push(Block {
                    content: ContentBlock::Bullets(bullet_items),
                    label: None,
                    span: SourceSpan::default(),
                });
            },
            _ => {},
        }

        // ── 5. Media (chart / image / diagram) ───────────────────────────────
        //
        // `resolve_alt` and `is_decorative` are called ONLY inside each media
        // branch — NOT unconditionally here — so that the W-A11-002 warning
        // (emitted by `resolve_alt` when both `decorative: true` AND a non-empty
        // `alt` are set) fires only when the slide is actually a media slide.
        // Non-media slides may carry inert `decorative:` / `alt:` fields without
        // triggering the conflict warning (F-086-P13-OBS-001 fix;
        // error-taxonomy v2.17 W-A11-002 scopes this warning to the MEDIA path).
        let slide_type: &str = slide.slide_type.as_ref();

        // Chart: slide_type == "chart" (or any ChartRenderer surface type).
        // For v1.0, keyed on slide_type == "chart".
        //
        // NOTE on alt=None handling (ADR-019 Decision 3 / ADR-018 v1.2 Decision-3):
        // Stage 2b emits ContentBlock::Chart even when alt=None. The pre-layout
        // AltTextValidator::validate() is restricted to ContentBlock::Shape ONLY (ADR-018 v1.2
        // Decision-3), so no double-fire occurs. thread_media_alt_into_frames maps None →
        // AltText::Unspecified on the frame, and validate_post_layout emits exactly one
        // E-A11-001 for the missing alt (BC-5.01.001 postcondition 1 / AC-005).
        //
        // This design change (architect-pass-1-adjudication Issue 1 verdict CODE-CONFORMS)
        // supersedes the prior anti-double-fire skip logic.
        if slide_type == "chart" {
            // Resolve alt/decorative inside the media branch (F-086-P13-OBS-001):
            // W-A11-002 fires only for chart slides, not for non-media slides.
            let alt = resolve_alt(slide);
            let decorative = is_decorative(slide);
            if let Some(chart_type) = extract_str_field(slide, "chart_type") {
                // Always emit the ContentBlock::Chart, even when alt=None.
                // Pre-layout validate() is restricted to Shape; post-layout fires exactly once.
                slide.blocks.push(Block {
                    content: ContentBlock::Chart(ChartSpec {
                        chart_type: Arc::from(chart_type),
                        alt,
                        decorative,
                        span: SourceSpan::default(),
                    }),
                    label: None,
                    span: SourceSpan::default(),
                });
            } else {
                tracing::warn!(
                    slide_type,
                    "Stage 2b: chart slide has no 'chart_type' field — \
                     skipping ContentBlock::Chart construction; \
                     layout will retain AltText::Unspecified structural placeholder (ADR-019 Decision 3.3)"
                );
            }
        } else if matches!(slide_type, "image" | "screenshot" | "bio") {
            // Image: slide_type in {image, screenshot, bio}.
            // Resolve alt/decorative inside the media branch (F-086-P13-OBS-001).
            let alt = resolve_alt(slide);
            let decorative = is_decorative(slide);
            // Same rule as chart: emit unconditionally; pre-layout validate() is Shape-only.
            if let Some(src) = extract_str_field(slide, "src") {
                // Always emit the ContentBlock::Image, even when alt=None.
                slide.blocks.push(Block {
                    content: ContentBlock::Image(ImageSpec {
                        path: Arc::from(src),
                        alt,
                        decorative,
                        span: SourceSpan::default(),
                    }),
                    label: None,
                    span: SourceSpan::default(),
                });
            } else {
                tracing::warn!(
                    slide_type,
                    "Stage 2b: image-type slide has no 'src' field — \
                     skipping ContentBlock::Image construction; \
                     layout will retain AltText::Unspecified structural placeholder (ADR-019 Decision 3.4)"
                );
            }
        } else if slide_type == "diagram" {
            // Diagram: slide_type == "diagram".
            // Resolve alt/decorative inside the media branch (F-086-P13-OBS-001).
            let alt = resolve_alt(slide);
            let decorative = is_decorative(slide);
            // Same rule as chart/image: emit unconditionally; pre-layout validate() is Shape-only.
            if let Some(source) = extract_str_field(slide, "source") {
                // Always emit the ContentBlock::Diagram, even when alt=None.
                slide.blocks.push(Block {
                    content: ContentBlock::Diagram(DiagramSpec {
                        source: Arc::from(source),
                        alt,
                        decorative,
                        span: SourceSpan::default(),
                    }),
                    label: None,
                    span: SourceSpan::default(),
                });
            } else {
                tracing::warn!(
                    slide_type,
                    "Stage 2b: diagram slide has no 'source' field — \
                     skipping ContentBlock::Diagram construction; \
                     layout will retain AltText::Unspecified structural placeholder (ADR-019 Decision 3.5)"
                );
            }
        }

        // ── 6. Color-coded slide types (status / progress_bar / weighted_composite / stat_callout)
        //
        // These types carry fields beyond title/subtitle/body that encode WCAG co-encoding
        // text (`label`) and numeric state (`value`, `components`). Stage 2b threads them
        // into ContentBlocks so that `layout::run` can fill the pre-allocated region slots.
        //
        // Threading contract per architect pass-2 adjudication §4.2 (STORY-087):
        //   "label"       → ContentBlock::Text(TextTag::ColorLabel) — Body-role slot
        //   "value"       → ContentBlock::ColorBar(ColorBarSpec)    — Generic-role bar-fill
        //   "components"  → ContentBlock::Text(TextTag::Body) × N  — Generic-role row slots
        //   stat fields   → ContentBlock::Text(TextTag::Body) × N  — Generic-role stat slots
        //
        // Purity: this block is a pure extension; no I/O, no global state, no existing
        // slide-type code path is modified (blast radius = zero per adjudication §5).
        match slide_type {
            "status" => {
                // Thread "label" → ColorLabel (Body-role slot: wide right-side frame).
                if let Some(text) = extract_str_field(slide, "label")
                    && !text.trim().is_empty()
                {
                    slide
                        .blocks
                        .push(make_text_block_tagged(text, TextTag::ColorLabel));
                }
            },
            "progress_bar" => {
                // Thread "label" → ColorLabel (Body-role slot: label below bar).
                if let Some(text) = extract_str_field(slide, "label")
                    && !text.trim().is_empty()
                {
                    slide
                        .blocks
                        .push(make_text_block_tagged(text, TextTag::ColorLabel));
                }
                // Thread "value" → ColorBar (Generic-role slot: bar-fill geometry).
                // ValueRangeValidator already rejects out-of-range values at Stage 5;
                // clamp defensively here to prevent layout panics on invalid inputs.
                if let Some(FieldValue::Literal(Value::Int(v))) = slide.fields.get("value") {
                    let pct = (*v).clamp(0, 100);
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let percent = pct as u8;
                    slide.blocks.push(Block {
                        content: ContentBlock::ColorBar(ColorBarSpec { percent }),
                        label: None,
                        span: SourceSpan::default(),
                    });
                }
            },
            "weighted_composite" => {
                // Thread "label" (aggregate) → ColorLabel (Body-role slot).
                if let Some(text) = extract_str_field(slide, "label")
                    && !text.trim().is_empty()
                {
                    slide
                        .blocks
                        .push(make_text_block_tagged(text, TextTag::ColorLabel));
                }
                // Thread each component → TextTag::Body (one per component, Generic-role slots).
                // compose_component_row_text formats: "<name>: <score>/100 (wt: <weight>) — <label>".
                //
                // BC-1.17.003 PC-9: cap at MAX_WEIGHTED_COMPOSITE_COMPONENT_ROWS (5) — exactly the
                // number of Generic-role slots pre-allocated in `slideforge-layout/src/regions.rs`
                // for `weighted_composite`. Components beyond index 4 produce no Body block and
                // therefore claim no additional frame. (F-087-P7-001)
                if let Some(FieldValue::Literal(Value::List(components))) =
                    slide.fields.get("components")
                {
                    for comp_val in components
                        .iter()
                        .take(MAX_WEIGHTED_COMPOSITE_COMPONENT_ROWS)
                    {
                        if let Value::Map(comp) = comp_val {
                            let row_text = compose_component_row_text(comp);
                            if !row_text.is_empty() {
                                slide
                                    .blocks
                                    .push(make_text_block_tagged(&row_text, TextTag::Body));
                            }
                        }
                    }
                }
            },
            "stat_callout" => {
                // Thread stat_1/label_1/stat_2/label_2/stat_3/label_3 → TextTag::Body
                // (each claims a Generic-role slot in registration order).
                for field_name in &[
                    "stat_1", "label_1", "stat_2", "label_2", "stat_3", "label_3",
                ] {
                    if let Some(text) = extract_str_field(slide, field_name)
                        && !text.trim().is_empty()
                    {
                        slide
                            .blocks
                            .push(make_text_block_tagged(text, TextTag::Body));
                    }
                }
            },
            _ => {},
        }
    }
}

/// Compose a single accessible text string for one `weighted_composite` component.
///
/// Format: `"<name>: <score>/100 (wt: <weight>) — <label>"`.
///
/// Any absent field is omitted from the output. The string is non-empty when at
/// least the `name` or `score` field is present. An empty string signals to the
/// caller that no `ContentBlock::Text` should be emitted for this component.
///
/// This is a pure helper — no I/O, no global state.
fn compose_component_row_text(comp: &OrderedMap<Arc<str>, Value>) -> String {
    use std::fmt::Write as _;

    let name = match comp.get("name") {
        Some(Value::Str(s)) => s.as_ref().to_owned(),
        _ => String::new(),
    };
    let score = match comp.get("score") {
        Some(Value::Int(n)) => Some(*n),
        _ => None,
    };
    // Weight is display-only (formatted as-is); i64→f64 precision loss is
    // acceptable here because the value is user-supplied and only used for
    // human-readable output in the composed text string.
    #[allow(clippy::cast_precision_loss)]
    let weight: Option<f64> = match comp.get("weight") {
        Some(Value::Float(f)) => Some(f.0),
        Some(Value::Int(n)) => Some(*n as f64),
        _ => None,
    };
    let label = match comp.get("label") {
        Some(Value::Str(s)) => s.as_ref().to_owned(),
        _ => String::new(),
    };

    if name.is_empty() && score.is_none() {
        return String::new();
    }

    let mut out = String::new();
    if !name.is_empty() {
        out.push_str(&name);
    }
    if let Some(s) = score {
        if !out.is_empty() {
            out.push_str(": ");
        }
        // Use write! to avoid the format_push_string lint (clippy::pedantic).
        let _ = write!(out, "{s}/100");
    }
    if let Some(w) = weight {
        let _ = write!(out, " (wt: {w})");
    }
    if !label.is_empty() {
        out.push_str(" \u{2014} "); // em dash
        out.push_str(&label);
    }
    out
}

/// Extract a `Str` value from `slide.fields[key]` as a `&str`.
///
/// Returns `None` only if the field is absent or not a `Literal(Str)`.
/// An empty string is returned as `Some("")` — callers must apply their own
/// empty/whitespace guard as required by their contract.
fn extract_str_field<'s>(slide: &'s slideforge_types::Slide, key: &str) -> Option<&'s str> {
    match slide.fields.get(key) {
        Some(FieldValue::Literal(Value::Str(s))) => Some(s.as_ref()),
        _ => None,
    }
}

/// Resolve the `alt` field from a slide into an `Option<AltText>`.
///
/// Precedence (BC-1.16.001 PC-12, ADR-019 Decision 4 — decorative-first):
/// 1. `decorative: true` → `Some(AltText::Decorative)` (decorative opt-out
///    wins unconditionally, even when a non-empty `alt` is also present).
/// 2. `alt "..."` (non-empty after trim) → `Some(AltText::Provided(Arc::from(s.trim())))`.
/// 3. Neither → `None`.
///
/// When both `decorative: true` AND a non-empty `alt` are present (conflict
/// case), this function:
/// - Returns `Some(AltText::Decorative)` (decorative wins — BC-1.16.001 PC-12 / EC-004).
/// - Emits `tracing::warn!(code = "W-A11-002", ...)` to surface the conflict
///   so authors can clean up the contradictory field pair. The warning is only
///   emitted when BOTH are set; single-field cases are silent.
///
/// The former "alt-first" rule (BC-3.04.001 Invariant 11) is superseded by
/// BC-1.16.001 PC-12 per architect pass-5 adjudication (finding F-086-P5-CRIT-001).
fn resolve_alt(slide: &slideforge_types::Slide) -> Option<AltText> {
    let has_decorative = is_decorative(slide);
    let alt_str = extract_str_field(slide, "alt");
    let has_alt = alt_str.is_some_and(|s| !s.trim().is_empty());

    if has_decorative {
        if has_alt {
            // Both fields set: decorative wins, but warn the author (W-A11-002).
            // error-taxonomy v2.17 W-A11-002: decorative opt-out takes precedence
            // over a provided alt string; the alt string is discarded.
            tracing::warn!(
                code = "W-A11-002",
                slide_type = %slide.slide_type,
                "accessibility conflict: both `decorative: true` and a non-empty `alt` \
                 are set on this slide; the decorative opt-out wins (BC-1.16.001 PC-12 / EC-004) \
                 and the alt string is discarded. Remove one field to silence this warning."
            );
        }
        Some(AltText::Decorative)
    } else if let Some(s) = alt_str
        && !s.trim().is_empty()
    {
        Some(AltText::Provided(Arc::from(s.trim())))
    } else {
        None
    }
}

/// Return `true` if the slide has `decorative: true` in its fields.
fn is_decorative(slide: &slideforge_types::Slide) -> bool {
    matches!(
        slide.fields.get("decorative"),
        Some(FieldValue::Literal(Value::Bool(true)))
    )
}

/// Construct a [`slideforge_types::Block`] wrapping a `ContentBlock::Text` with the
/// specified [`TextTag`].
///
/// Stage 2b callers use this to set the correct semantic tag when constructing text
/// blocks from DSL fields (`title`, `subtitle`, `body`). The tag drives the routing
/// decision in `layout.rs`:
/// - `TextTag::Title` → `FrameContent::Title` (PPTX `type="title"`, DOCX Heading 1)
/// - `TextTag::Subtitle` → `FrameContent::Subtitle` (PPTX `type="subTitle"`, DOCX Heading 2)
/// - `TextTag::Body` → `FrameContent::Body` (PPTX `type="body"`, DOCX Normal)
/// - `TextTag::Untagged` → `FrameContent::TextRun` (generic inline run)
fn make_text_block_tagged(text: &str, tag: TextTag) -> Block {
    Block {
        content: ContentBlock::Text(TextBlock {
            inlines: vec![InlineNode::Plain(Arc::from(text.trim()))],
            tag,
            span: SourceSpan::default(),
        }),
        label: None,
        span: SourceSpan::default(),
    }
}

/// Build a typed `Block` from an already-evaluated inline node sequence.
///
/// Used when the field value is `FieldValue::Inlines` (produced by `eval_slide_node`
/// for fields carrying inline markup). Carries the `Vec<InlineNode>` verbatim so
/// that exporters receive the full structural information (bold, italic, etc.)
/// instead of a flattened plain-text string.
///
/// STORY-081 C2 fix: `body` and `subtitle` fields with inline markup must reach
/// the layout pass as `ContentBlock::Text` with non-plain inline nodes, not be
/// silently dropped by `extract_str_field` which only matched `Literal(Str)`.
fn make_text_block_tagged_inlines(inlines: Vec<InlineNode>, tag: TextTag) -> Block {
    Block {
        content: ContentBlock::Text(TextBlock {
            inlines,
            tag,
            span: SourceSpan::default(),
        }),
        label: None,
        span: SourceSpan::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slideforge_types::{Deck, DeckMetadata, FieldValue, InlineNode, OrderedMap, Slide, Value};

    fn make_metadata() -> DeckMetadata {
        DeckMetadata {
            title: Some(Arc::from("Test")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        }
    }

    fn make_deck(slides: Vec<Slide>) -> Deck {
        Deck {
            slides,
            vars: OrderedMap::new(),
            metadata: make_metadata(),
            registers: OrderedMap::new(),
            section_blocks: vec![],
            slide_sections: vec![],
        }
    }

    fn make_slide(slide_type: &str) -> Slide {
        Slide {
            slide_type: Arc::from(slide_type),
            fields: OrderedMap::new(),
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }
    }

    #[test]
    fn test_empty_deck_is_noop() {
        let mut deck = make_deck(vec![]);
        thread_fields_to_blocks(&mut deck);
        assert_eq!(deck.slides.len(), 0);
    }

    #[test]
    fn test_empty_title_skipped() {
        let mut slide = make_slide("title");
        slide.fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from(""))),
        );
        let mut deck = make_deck(vec![slide]);
        thread_fields_to_blocks(&mut deck);
        assert_eq!(deck.slides[0].blocks.len(), 0);
    }

    #[test]
    fn test_whitespace_title_skipped() {
        let mut slide = make_slide("title");
        slide.fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from("   "))),
        );
        let mut deck = make_deck(vec![slide]);
        thread_fields_to_blocks(&mut deck);
        assert_eq!(deck.slides[0].blocks.len(), 0);
    }

    #[test]
    fn test_title_produces_text_block() {
        let mut slide = make_slide("title");
        slide.fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from("Hello World"))),
        );
        let mut deck = make_deck(vec![slide]);
        thread_fields_to_blocks(&mut deck);
        assert_eq!(deck.slides[0].blocks.len(), 1);
        assert!(matches!(
            deck.slides[0].blocks[0].content,
            ContentBlock::Text(_)
        ));
    }

    #[test]
    fn test_chart_with_alt_produces_provided() {
        let mut slide = make_slide("chart");
        slide.fields.insert(
            Arc::from("chart_type"),
            FieldValue::Literal(Value::Str(Arc::from("bar"))),
        );
        slide.fields.insert(
            Arc::from("alt"),
            FieldValue::Literal(Value::Str(Arc::from("Revenue chart"))),
        );
        let mut deck = make_deck(vec![slide]);
        thread_fields_to_blocks(&mut deck);
        let chart_blocks: Vec<_> = deck.slides[0]
            .blocks
            .iter()
            .filter(|b| matches!(b.content, ContentBlock::Chart(_)))
            .collect();
        assert_eq!(chart_blocks.len(), 1);
        if let ContentBlock::Chart(spec) = &chart_blocks[0].content {
            assert!(matches!(&spec.alt, Some(AltText::Provided(_))));
        }
    }

    #[test]
    fn test_chart_decorative_produces_decorative() {
        let mut slide = make_slide("chart");
        slide.fields.insert(
            Arc::from("chart_type"),
            FieldValue::Literal(Value::Str(Arc::from("area"))),
        );
        slide.fields.insert(
            Arc::from("decorative"),
            FieldValue::Literal(Value::Bool(true)),
        );
        let mut deck = make_deck(vec![slide]);
        thread_fields_to_blocks(&mut deck);
        let chart_blocks: Vec<_> = deck.slides[0]
            .blocks
            .iter()
            .filter(|b| matches!(b.content, ContentBlock::Chart(_)))
            .collect();
        assert_eq!(chart_blocks.len(), 1);
        if let ContentBlock::Chart(spec) = &chart_blocks[0].content {
            assert!(matches!(&spec.alt, Some(AltText::Decorative)));
        }
    }

    /// Issue 1 adjudication (architect-pass-1) — Stage 2b MUST emit `ContentBlock::Chart`
    /// even when `alt = None`. The chart block carries `AltText::Unspecified` to signal
    /// the post-layout validator that alt resolution is pending.
    ///
    /// RED GATE (Issue 1): this test is INVERTED from the prior version. Current HEAD
    /// ba3bcc93 does NOT emit a chart block when alt=None — it asserts 0 chart blocks.
    /// This test now asserts 1 chart block with `alt = None` (which maps to Unspecified
    /// in the frame). The test will FAIL against the current code that skips the block.
    ///
    /// Traces: BC-1.16.001 PC-9; architect-pass-1-adjudication Issue 1 verdict CODE-CONFORMS.
    #[test]
    fn test_bc_1_16_001_chart_no_alt_emits_block_with_none_alt() {
        // Issue 1 Red Gate: Stage 2b MUST emit ContentBlock::Chart even when alt=None.
        // Current code skips the block (anti-double-fire logic) — this MUST be inverted.
        // After fix: Chart block IS emitted with alt=None (layout maps None→Unspecified).
        let mut slide = make_slide("chart");
        slide.fields.insert(
            Arc::from("chart_type"),
            FieldValue::Literal(Value::Str(Arc::from("bar"))),
        );
        let mut deck = make_deck(vec![slide]);
        thread_fields_to_blocks(&mut deck);
        let chart_blocks: Vec<_> = deck.slides[0]
            .blocks
            .iter()
            .filter(|b| matches!(b.content, ContentBlock::Chart(_)))
            .collect();
        // RED GATE: current code skips the block → chart_blocks.len() == 0 → FAILS.
        // After implementation: Stage 2b emits ContentBlock::Chart with alt=None.
        assert_eq!(
            chart_blocks.len(),
            1,
            "Issue 1 RED GATE: Stage 2b MUST emit ContentBlock::Chart even when alt=None. \
             Current code skips the block (anti-double-fire logic). \
             After fix: chart block IS emitted with alt=None (no alt, not decorative). \
             Pre-layout AltTextValidator::validate() is restricted to Shape blocks only; \
             post-layout validate_post_layout() fires E-A11-001 for AltText::Unspecified. \
             BC-1.16.001 PC-9; architect-pass-1 Issue 1 verdict. \
             Got chart_blocks.len() = {} (expected 1).",
            chart_blocks.len()
        );
        // The emitted block must have alt=None (no alt supplied).
        if let ContentBlock::Chart(spec) = &chart_blocks[0].content {
            assert!(
                spec.alt.is_none(),
                "Issue 1: emitted ContentBlock::Chart must have alt=None when no alt was supplied; \
                 got alt={:?}",
                spec.alt
            );
        }
    }

    /// STORY-081×STORY-088: `FieldValue::InlinesList` bullets → `ContentBlock::Bullets`
    /// with per-item `BulletItem` carrying `InlineNode::Bold` (not `Plain("**bold**")`).
    ///
    /// This is the Stage 2b threading test for the fix: list-literal bullets with
    /// markup must produce proper `InlineNode` structure in `ContentBlock::Bullets`.
    #[test]
    fn test_inlines_list_bullets_produce_bold_bullet_items() {
        let mut slide = make_slide("content");
        // Simulate what eval_slide_node now produces for:
        //   bullets: ["**Key finding**: up 12%", "_note_", "plain item"]
        let item0_nodes = vec![
            InlineNode::Bold(vec![InlineNode::Plain(Arc::from("Key finding"))]),
            InlineNode::Plain(Arc::from(": up 12%")),
        ];
        let item1_nodes = vec![InlineNode::Italic(vec![InlineNode::Plain(Arc::from(
            "note",
        ))])];
        let item2_nodes = vec![InlineNode::Plain(Arc::from("plain item"))];
        slide.fields.insert(
            Arc::from("bullets"),
            FieldValue::InlinesList(vec![
                item0_nodes.clone(),
                item1_nodes.clone(),
                item2_nodes.clone(),
            ]),
        );
        let mut deck = make_deck(vec![slide]);
        thread_fields_to_blocks(&mut deck);

        assert_eq!(
            deck.slides[0].blocks.len(),
            1,
            "must produce exactly 1 block"
        );
        let block = &deck.slides[0].blocks[0];
        let ContentBlock::Bullets(bullet_items) = &block.content else {
            panic!("must be ContentBlock::Bullets; got: {:?}", block.content);
        };

        assert_eq!(
            bullet_items.len(),
            3,
            "must have 3 bullet items (one per InlinesList entry)"
        );

        // Item 0: first node must be InlineNode::Bold (not Plain("**Key finding**...")).
        assert!(
            matches!(bullet_items[0].inlines[0], InlineNode::Bold(_)),
            "item 0 first inline must be Bold; got: {:?}",
            bullet_items[0].inlines[0]
        );

        // Item 1: first node must be InlineNode::Italic.
        assert!(
            matches!(bullet_items[1].inlines[0], InlineNode::Italic(_)),
            "item 1 first inline must be Italic; got: {:?}",
            bullet_items[1].inlines[0]
        );

        // Item 2: first node must be Plain("plain item").
        assert!(
            matches!(&bullet_items[2].inlines[0], InlineNode::Plain(s) if s.as_ref() == "plain item"),
            "item 2 must be Plain(\"plain item\"); got: {:?}",
            bullet_items[2].inlines[0]
        );
    }

    /// P31-MED-001: caption `FieldValue::Inlines` must reach `ContentBlock::Text` (Untagged).
    /// Tests the new match arm added in STORY-081 fix-burst to prevent silent inline drop.
    #[test]
    fn test_p31_med_001_caption_inlines_produces_text_block() {
        let mut slide = make_slide("content");
        slide.fields.insert(
            Arc::from("caption"),
            FieldValue::Inlines(vec![InlineNode::Bold(vec![InlineNode::Plain(Arc::from(
                "bold caption",
            ))])]),
        );
        let mut deck = make_deck(vec![slide]);
        thread_fields_to_blocks(&mut deck);
        assert_eq!(
            deck.slides[0].blocks.len(),
            1,
            "caption FieldValue::Inlines must produce exactly 1 ContentBlock::Text"
        );
        let block = &deck.slides[0].blocks[0];
        let ContentBlock::Text(text_block) = &block.content else {
            panic!("expected ContentBlock::Text; got {:?}", block.content);
        };
        assert_eq!(
            text_block.tag,
            TextTag::Untagged,
            "caption must use TextTag::Untagged"
        );
        assert!(
            matches!(text_block.inlines[0], InlineNode::Bold(_)),
            "first inline must be Bold; got {:?}",
            text_block.inlines[0]
        );
    }

    /// P31-MED-001: description `FieldValue::Inlines` must reach `ContentBlock::Text` (Untagged).
    /// Tests the new match arm added in STORY-081 fix-burst to prevent silent inline drop.
    #[test]
    fn test_p31_med_001_description_inlines_produces_text_block() {
        let mut slide = make_slide("content");
        slide.fields.insert(
            Arc::from("description"),
            FieldValue::Inlines(vec![InlineNode::Italic(vec![InlineNode::Plain(
                Arc::from("italic description"),
            )])]),
        );
        let mut deck = make_deck(vec![slide]);
        thread_fields_to_blocks(&mut deck);
        assert_eq!(
            deck.slides[0].blocks.len(),
            1,
            "description FieldValue::Inlines must produce exactly 1 ContentBlock::Text"
        );
        let block = &deck.slides[0].blocks[0];
        let ContentBlock::Text(text_block) = &block.content else {
            panic!("expected ContentBlock::Text; got {:?}", block.content);
        };
        assert_eq!(
            text_block.tag,
            TextTag::Untagged,
            "description must use TextTag::Untagged"
        );
        assert!(
            matches!(text_block.inlines[0], InlineNode::Italic(_)),
            "first inline must be Italic; got {:?}",
            text_block.inlines[0]
        );
    }

    /// P31-MED-001: plain-string caption still reaches `ContentBlock::Text` (backwards compat).
    #[test]
    fn test_p31_med_001_caption_plain_str_produces_text_block() {
        let mut slide = make_slide("content");
        slide.fields.insert(
            Arc::from("caption"),
            FieldValue::Literal(Value::Str(Arc::from("plain caption text"))),
        );
        let mut deck = make_deck(vec![slide]);
        thread_fields_to_blocks(&mut deck);
        assert_eq!(deck.slides[0].blocks.len(), 1);
        assert!(matches!(
            deck.slides[0].blocks[0].content,
            ContentBlock::Text(_)
        ));
    }

    /// P31-MED-001: plain-string description still reaches `ContentBlock::Text` (backwards compat).
    #[test]
    fn test_p31_med_001_description_plain_str_produces_text_block() {
        let mut slide = make_slide("content");
        slide.fields.insert(
            Arc::from("description"),
            FieldValue::Literal(Value::Str(Arc::from("plain description text"))),
        );
        let mut deck = make_deck(vec![slide]);
        thread_fields_to_blocks(&mut deck);
        assert_eq!(deck.slides[0].blocks.len(), 1);
        assert!(matches!(
            deck.slides[0].blocks[0].content,
            ContentBlock::Text(_)
        ));
    }
}
