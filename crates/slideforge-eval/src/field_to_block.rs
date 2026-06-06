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
    Block, BulletItem, ContentBlock, Deck, FieldValue, InlineNode, SourceSpan, TextBlock, Value,
    specs::{AltText, ChartSpec, DiagramSpec, ImageSpec},
};

/// Post-eval field-to-block threading pass (Stage 2b, ADR-019).
///
/// Reads resolved field values from every `Slide.fields` in `deck` and
/// populates `Slide.blocks` with typed [`slideforge_types::ContentBlock`]
/// entries derived from those fields. This pass runs AFTER [`eval_deck`]
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
///
/// [`eval_deck`]: slideforge_eval::eval_deck
pub fn thread_fields_to_blocks(deck: &mut Deck) {
    for slide in &mut deck.slides {
        // ── 1. Title ─────────────────────────────────────────────────────────
        if let Some(text) = extract_str_field(slide, "title")
            && !text.trim().is_empty()
        {
            slide.blocks.push(make_text_block(text));
        }

        // ── 2. Subtitle ──────────────────────────────────────────────────────
        if let Some(text) = extract_str_field(slide, "subtitle")
            && !text.trim().is_empty()
        {
            slide.blocks.push(make_text_block(text));
        }

        // ── 3. Body ──────────────────────────────────────────────────────────
        if let Some(text) = extract_str_field(slide, "body")
            && !text.trim().is_empty()
        {
            slide.blocks.push(make_text_block(text));
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
                // Rich-text bullet list from the parser: thread as-is.
                let _ = nodes; // nodes is &Vec<InlineNode>
                // ADR-019 Decision 3.2: FieldValue::Inlines bullets threaded as-is.
                // We don't have structured BulletItem from Inlines — emit as a single
                // BulletItem carrying the inline nodes.
                if let Some(FieldValue::Inlines(inlines)) = slide.fields.get("bullets") {
                    slide.blocks.push(Block {
                        content: ContentBlock::Bullets(vec![BulletItem {
                            inlines: inlines.clone(),
                            children: vec![],
                            span: SourceSpan::default(),
                        }]),
                        label: None,
                        span: SourceSpan::default(),
                    });
                }
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
        // NOTE on alt=None handling (ADR-019 Decision 3 / anti-double-fire):
        // When the author provides neither `alt "..."` nor `decorative true`, Stage 2b
        // produces `alt = None` for the resolved alt field. In this case, Stage 2b does NOT
        // emit a ContentBlock::Chart — the structural placeholder in regions.rs retains
        // `AltText::Unspecified`, and the post-layout validator (`validate_post_layout`)
        // emits exactly one E-A11-001 for the missing alt.
        //
        // If Stage 2b were to emit ContentBlock::Chart { alt: None }, the pre-layout
        // validator (AltTextValidator::validate) would ALSO fire E-A11-001 for the same
        // missing alt, producing a double-diagnostic in the combined strict-mode report.
        // Skipping ContentBlock::Chart creation when alt=None ensures exactly one E-A11-001
        // per missing-alt chart (ADR-019 Decision 3.3 / BC-5.01.001 postcondition 1).
        //
        // For alt=Some(Provided(...)) or alt=Some(Decorative), a ContentBlock::Chart IS
        // emitted so that thread_media_alt_into_frames can thread the author-supplied
        // alt text into the frame, overwriting the Unspecified placeholder.
        if slide_type == "chart" {
            if let Some(chart_type) = extract_str_field(slide, "chart_type") {
                if alt.is_some() {
                    // Author supplied alt or decorative — thread into the chart block.
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
                    // No alt and not decorative: skip ContentBlock::Chart.
                    // The structural placeholder in regions.rs carries AltText::Unspecified,
                    // and validate_post_layout will fire E-A11-001 exactly once.
                    tracing::debug!(
                        slide_type,
                        chart_type,
                        "Stage 2b: chart has no alt and is not decorative — \
                         skipping ContentBlock::Chart; regions.rs Unspecified placeholder \
                         retained; validate_post_layout will emit E-A11-001 (ADR-019 Decision 3.3)"
                    );
                }
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
            // Same anti-double-fire rule as chart: skip when alt=None.
            if let Some(src) = extract_str_field(slide, "src") {
                if alt.is_some() {
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
                    tracing::debug!(
                        slide_type,
                        src,
                        "Stage 2b: image has no alt and is not decorative — \
                         skipping ContentBlock::Image; Unspecified placeholder retained"
                    );
                }
            } else {
                tracing::warn!(
                    slide_type,
                    "Stage 2b: image-type slide has no 'src' field — \
                     skipping ContentBlock::Image construction; \
                     layout will retain AltText::Unspecified structural placeholder (ADR-019 Decision 3.4)"
                );
            }
        } else if slide_type == "diagram" {
            // Diagram: slide_type == "diagram". Same anti-double-fire rule.
            if let Some(source) = extract_str_field(slide, "source") {
                if alt.is_some() {
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
                    tracing::debug!(
                        slide_type,
                        source,
                        "Stage 2b: diagram has no alt and is not decorative — \
                         skipping ContentBlock::Diagram; Unspecified placeholder retained"
                    );
                }
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

/// Construct a `Block` wrapping a `ContentBlock::Text` from a plain string.
fn make_text_block(text: &str) -> Block {
    Block {
        content: ContentBlock::Text(TextBlock {
            inlines: vec![InlineNode::Plain(Arc::from(text))],
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

    #[test]
    fn test_chart_no_alt_skips_block() {
        // When a chart has no alt and is not decorative, Stage 2b does NOT create
        // a ContentBlock::Chart. The post-layout validator will fire E-A11-001 via
        // the AltText::Unspecified structural placeholder from regions.rs.
        // This prevents double-firing: pre-layout + post-layout both emitting E-A11-001
        // for the same missing-alt chart (ADR-019 Decision 3.3).
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
        // No ContentBlock::Chart when alt=None — post-layout handles validation.
        assert_eq!(
            chart_blocks.len(),
            0,
            "Stage 2b must NOT create ContentBlock::Chart when alt=None (no alt, not decorative); \
             post-layout validator catches this via AltText::Unspecified. ADR-019 Decision 3.3."
        );
    }
}
