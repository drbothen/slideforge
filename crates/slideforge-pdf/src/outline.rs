//! PDF document outline (bookmarks) builder.
//!
//! This module provides [`build_outline_entries`] — a pure function that converts
//! a [`Deck`] + [`LaidOutDeck`] pair into a [`Vec<OutlineEntry>`].  The entries
//! can be inspected in unit tests (count, labels, destination page indices) and
//! are consumed by [`build_krilla_outline`] to produce a krilla [`Outline`] for
//! `Document::set_outline`.
//!
//! ## Why two functions?
//!
//! krilla's [`Outline`] and [`OutlineNode`] have private fields — they cannot be
//! inspected after construction. Splitting the logic into a pure data-building step
//! (`build_outline_entries`) and a krilla-binding step (`build_krilla_outline`)
//! makes the label/destination correctness testable without serialising a full PDF.
//!
//! ## F-045-P1-004 compliance
//!
//! AC-010 postcondition requires that every outline entry's destination page index
//! matches the page on which the corresponding slide is rendered. That property is
//! tested via [`OutlineEntry::page_idx`] in the unit tests below.

use krilla::destination::XyzDestination;
use krilla::geom::Point;
use krilla::outline::{Outline, OutlineNode};
use slideforge_layout::LaidOutDeck;
use slideforge_types::Deck;

// ─── Public data type ─────────────────────────────────────────────────────────

/// A single document outline entry (bookmark) before it is converted into a
/// krilla [`OutlineNode`].
///
/// Carries a human-readable label and the 0-based page index of the destination
/// page (i.e., the page that the bookmark should navigate to when clicked).
///
/// Created by [`build_outline_entries`]; consumed by [`build_krilla_outline`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineEntry {
    /// Bookmark label text.
    ///
    /// Sourced from `deck.slides[source_index].title_str()`; falls back to
    /// `"Slide N"` (1-based) when the semantic slide has no `title` field.
    pub label: String,

    /// 0-based index of the page that this entry navigates to.
    ///
    /// Equal to the position of the laid-out slide in `laid_out.slides`
    /// (i.e., the slide's ordinal in rendered page order, NOT `source_index`).
    pub page_idx: usize,
}

// ─── Pure builder ─────────────────────────────────────────────────────────────

/// Build a list of [`OutlineEntry`] records — one per laid-out slide, in page order.
///
/// ## Label sourcing
///
/// For each `laid_out.slides[page_idx]`:
/// 1. Try `deck.slides[slide.source_index].title_str()` for the label.
/// 2. Fall back to `"Slide N"` (1-based, `N = page_idx + 1`) when `title_str()` returns `None`
///    OR when `source_index` is out-of-bounds in `deck.slides`.
///
/// ## F-045-P1-005 invariant
///
/// The destination page index for entry `i` is always `i` (the position in
/// `laid_out.slides`), regardless of `source_index`. Label sourcing uses
/// `source_index` to look up semantic metadata; page navigation uses the
/// enumerate position.
///
/// When `laid_out.slides[i].source_index != i` (non-identity permutation), the
/// label is drawn from `deck.slides[source_index]` but the page destination still
/// points to page `i`. This is the correct behaviour: the bookmark says "Intro"
/// (from the semantic slide) and navigates to the page where "Intro" was rendered.
#[must_use]
pub fn build_outline_entries(deck: &Deck, laid_out: &LaidOutDeck) -> Vec<OutlineEntry> {
    laid_out
        .slides
        .iter()
        .enumerate()
        .map(|(page_idx, slide)| {
            // F-045-P1-005: label comes from deck.slides[source_index]; destination is page_idx.
            // When source_index is out-of-bounds (e.g., deck has fewer slides than laid_out),
            // the lookup returns None → fallback label.
            let label: String = deck
                .slides
                .get(slide.source_index)
                .and_then(|s| s.title_str())
                .map_or_else(
                    || format!("Slide {}", page_idx + 1),
                    std::borrow::ToOwned::to_owned,
                );

            // debug_assert: document that the identity invariant is the normal case.
            // Non-identity source_index is valid (see F-045-P1-005 note above) but
            // rare — assert at debug level so tests with permuted layouts catch regressions.
            debug_assert!(
                slide.source_index < deck.slides.len() || deck.slides.is_empty(),
                "build_outline_entries: slide at page {page_idx} has source_index {} \
                 which is out-of-bounds for deck.slides.len()={} — fallback label used",
                slide.source_index,
                deck.slides.len(),
            );

            OutlineEntry { label, page_idx }
        })
        .collect()
}

/// Convert a [`Vec<OutlineEntry>`] into a krilla [`Outline`].
///
/// Each entry maps to one top-level [`OutlineNode`] with:
/// - `text` = `entry.label`
/// - `destination` = [`XyzDestination`] pointing to the top-left of page `entry.page_idx`
///
/// The krilla [`Outline`] is consumed by `Document::set_outline`.
#[must_use]
pub fn build_krilla_outline(entries: &[OutlineEntry]) -> Outline {
    let mut outline = Outline::new();
    for entry in entries {
        // XyzDestination top-left: Point::from_xy(0.0, 0.0) in krilla's Y-down surface
        // coordinate system. krilla applies the PDF Y-flip internally during serialisation.
        let dest = XyzDestination::new(entry.page_idx, Point::from_xy(0.0, 0.0));
        let node = OutlineNode::new(entry.label.clone(), dest);
        outline.push_child(node);
    }
    outline
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use slideforge_layout::types::{
        BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize, RegisterSet,
    };
    use slideforge_types::{
        Deck, DeckMetadata, Emu, FieldValue, OrderedMap, Slide, SourceSpan, Value,
    };

    use super::*;

    // ── helpers ────────────────────────────────────────────────────────────────

    fn page_size() -> PageSize {
        PageSize::default()
    }

    fn slide_with_title(title: &str, slide_type: &str) -> Slide {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from(title))),
        );
        Slide {
            slide_type: Arc::from(slide_type),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }
    }

    fn slide_without_title(slide_type: &str) -> Slide {
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

    fn deck_with_slides(slides: Vec<Slide>) -> Deck {
        Deck {
            slides,
            vars: OrderedMap::new(),
            metadata: DeckMetadata {
                title: Some(Arc::from("Outline Test Deck")),
                slideforge_version: Arc::from("0.1.0"),
                lang: Some(Arc::from("en-US")),
                author: None,
                section_order: None,
            },
            registers: OrderedMap::new(),
            section_blocks: vec![],
        }
    }

    fn title_frame(page_idx: usize, source_index: usize) -> LaidOutSlide {
        LaidOutSlide {
            source_index,
            slide_type_keyword: Arc::from("title"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(914_400),
                },
                content: FrameContent::Title(Arc::from(format!("Slide {page_idx}"))),
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        }
    }

    fn n_slide_laid_out_deck(n: usize) -> LaidOutDeck {
        // Identity mapping: laid_out.slides[i].source_index == i.
        let slides = (0..n).map(|i| title_frame(i, i)).collect();
        LaidOutDeck {
            page_size: page_size(),
            slides,
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],
        }
    }

    // ── F-045-P1-004: 3-slide titled deck → exactly 3 entries in source order ─

    /// F-045-P1-004(a): a 3-slide titled deck produces exactly 3 outline entries
    /// with labels in source order.
    ///
    /// AC-010 postcondition: entry count == slide count; labels match slide titles.
    #[test]
    fn test_build_outline_entries_three_titled_slides_count_and_order() {
        let deck = deck_with_slides(vec![
            slide_with_title("Overview", "title"),
            slide_with_title("Data", "content"),
            slide_with_title("Summary", "title"),
        ]);
        let laid_out = n_slide_laid_out_deck(3);

        let entries = build_outline_entries(&deck, &laid_out);

        // Count: exactly one entry per slide.
        assert_eq!(
            entries.len(),
            3,
            "build_outline_entries: expected 3 entries for 3-slide deck, got {}",
            entries.len()
        );

        // Order and labels.
        assert_eq!(entries[0].label, "Overview", "entry[0] label mismatch");
        assert_eq!(entries[1].label, "Data", "entry[1] label mismatch");
        assert_eq!(entries[2].label, "Summary", "entry[2] label mismatch");
    }

    // ── F-045-P1-004(b): untitled slide → "Slide N" fallback ──────────────────

    /// F-045-P1-004(b): an untitled slide produces a fallback label "Slide N"
    /// (1-based N = `page_idx` + 1).
    #[test]
    fn test_build_outline_entries_untitled_slide_fallback_label() {
        let deck = deck_with_slides(vec![slide_without_title("title")]);
        let laid_out = n_slide_laid_out_deck(1);

        let entries = build_outline_entries(&deck, &laid_out);

        assert_eq!(entries.len(), 1, "expected 1 entry");
        assert_eq!(
            entries[0].label, "Slide 1",
            "untitled slide must produce fallback label 'Slide 1'"
        );
    }

    // ── F-045-P1-004(c): destination page index correctness ───────────────────

    /// F-045-P1-004(c): the destination `page_idx` of entry `i` must equal `i`
    /// (the 0-based rendered page index), NOT `source_index`.
    ///
    /// AC-010 postcondition: destination → page correspondence.
    #[test]
    fn test_build_outline_entries_destination_page_indices_are_correct() {
        let deck = deck_with_slides(vec![
            slide_with_title("Alpha", "title"),
            slide_with_title("Beta", "content"),
            slide_with_title("Gamma", "title"),
        ]);
        let laid_out = n_slide_laid_out_deck(3);

        let entries = build_outline_entries(&deck, &laid_out);

        // Each entry's page_idx must equal its position in the entries vec.
        for (i, entry) in entries.iter().enumerate() {
            assert_eq!(
                entry.page_idx, i,
                "entry[{i}]: expected page_idx={i}, got page_idx={}",
                entry.page_idx
            );
        }
    }

    // ── F-045-P1-005: non-identity source_index permutation ───────────────────

    /// F-045-P1-005: when `laid_out.slides[i].source_index != i` (non-identity
    /// permutation), the label is sourced from `deck.slides[source_index]` but
    /// the destination page index is still `i` (the render position).
    ///
    /// Scenario: 2-slide deck where the slide rendered at page 0 was sourced from
    /// `deck.slides[1]` and the slide rendered at page 1 was sourced from
    /// `deck.slides[0]`.  Labels should be swapped relative to deck order; page
    /// destinations should remain 0 and 1 respectively.
    #[test]
    fn test_build_outline_entries_non_identity_source_index_permutation() {
        // Semantic deck: slides in order [First, Second].
        let deck = deck_with_slides(vec![
            slide_with_title("First", "title"),
            slide_with_title("Second", "title"),
        ]);

        // Laid-out deck: page 0 renders source_index=1 ("Second"),
        //                 page 1 renders source_index=0 ("First").
        // This can happen when slides are reordered by layout logic.
        let mut laid_out = n_slide_laid_out_deck(2);
        laid_out.slides[0].source_index = 1; // page 0 → "Second"
        laid_out.slides[1].source_index = 0; // page 1 → "First"

        let entries = build_outline_entries(&deck, &laid_out);

        assert_eq!(entries.len(), 2, "expected 2 entries");

        // Labels follow source_index (semantic metadata).
        assert_eq!(
            entries[0].label, "Second",
            "page 0 label must come from source_index=1 ('Second')"
        );
        assert_eq!(
            entries[1].label, "First",
            "page 1 label must come from source_index=0 ('First')"
        );

        // Destination page indices are the render positions (not source_index).
        assert_eq!(
            entries[0].page_idx, 0,
            "page 0 destination must be page_idx=0 (render position)"
        );
        assert_eq!(
            entries[1].page_idx, 1,
            "page 1 destination must be page_idx=1 (render position)"
        );
    }
}
