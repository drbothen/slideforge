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
    Block, BulletItem, ContentBlock, Deck, FieldValue, InlineNode, SourceSpan, TextBlock, TextTag,
    Value,
    specs::{AltText, ChartSpec, DiagramSpec, ImageSpec},
};

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
        if let Some(text) = extract_str_field(slide, "subtitle")
            && !text.trim().is_empty()
        {
            slide
                .blocks
                .push(make_text_block_tagged(text, TextTag::Subtitle));
        }

        // ── 3. Body ──────────────────────────────────────────────────────────
        // TextTag::Body routes to FrameContent::Body (PPTX body placeholder,
        // DOCX Normal paragraph) per BC-4.01.001 v1.2 postcondition 11 / BC-4.02.001 v1.2 PC-10.
        if let Some(text) = extract_str_field(slide, "body")
            && !text.trim().is_empty()
        {
            slide
                .blocks
                .push(make_text_block_tagged(text, TextTag::Body));
        }

        // ── 4. Bullets ───────────────────────────────────────────────────────
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
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use slideforge_types::{Deck, DeckMetadata, FieldValue, OrderedMap, Slide, Value};

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
}
