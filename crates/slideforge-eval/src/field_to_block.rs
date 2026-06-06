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
/// # `AltText` contract
///
/// When constructing `ContentBlock::Chart`, `ContentBlock::Image`, or
/// `ContentBlock::Diagram`, the threading pass uses the following alt-resolution
/// precedence (see ADR-019 Decision 4, `AltText` state machine):
///
/// 1. `Slide.fields["decorative"] == Value::Bool(true)` → `ContentBlock.alt = Some(AltText::Decorative)`
/// 2. `Slide.fields["alt"] == Value::Str(s)` (non-empty, non-whitespace) → `ContentBlock.alt = Some(AltText::Provided(Arc::from(s)))`
/// 3. Neither present → `ContentBlock.alt = None`
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
        let alt = resolve_alt(slide);
        let decorative = is_decorative(slide);
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

/// Extract a non-empty `Str` value from `slide.fields[key]` as a `&str`.
///
/// Returns `None` if the field is absent, not a `Literal(Str)`, or empty.
fn extract_str_field<'s>(slide: &'s slideforge_types::Slide, key: &str) -> Option<&'s str> {
    match slide.fields.get(key) {
        Some(FieldValue::Literal(Value::Str(s))) => Some(s.as_ref()),
        _ => None,
    }
}

/// Resolve the `alt` field from a slide into an `Option<AltText>`.
///
/// Precedence (ADR-019 Decision 4):
/// 1. `decorative: true` → `Some(AltText::Decorative)` (checked by caller
///    via [`is_decorative`]; here we handle the alt-wins case).
/// 2. `alt "..."` (non-empty after trim) → `Some(AltText::Provided(s))`.
/// 3. Neither → `None`.
///
/// When both `decorative: true` AND a non-empty `alt` are present, `alt`
/// takes precedence per BC-3.04.001 Invariant 11: returns
/// `Some(AltText::Provided(s))` so the pre-layout validator can emit W-A11-002.
fn resolve_alt(slide: &slideforge_types::Slide) -> Option<AltText> {
    let has_decorative = is_decorative(slide);
    let alt_str = extract_str_field(slide, "alt");

    match alt_str {
        Some(s) if !s.trim().is_empty() => {
            // alt takes precedence over decorative (BC-3.04.001 Invariant 11).
            Some(AltText::Provided(Arc::from(s)))
        },
        _ => {
            if has_decorative {
                Some(AltText::Decorative)
            } else {
                None
            }
        },
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
            inlines: vec![InlineNode::Plain(Arc::from(text))],
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
