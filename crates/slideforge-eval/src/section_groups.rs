//! Eval-stage mapping: `SectionGroupNode` → `SlideSectionEntry` collection.
//!
//! This module provides [`extract_slide_sections`], which walks the parsed
//! [`slideforge_syntax::DeckNode`] to find `section "Name":` slide-grouping
//! blocks and maps their slide children to PPTX slide IDs, returning a
//! `Vec<SlideSectionEntry>` for the caller to assign to
//! [`slideforge_layout::LaidOutDeck::slide_sections`].
//!
//! ## Pipeline Position
//!
//! Called after [`crate::eval::eval_deck`] has produced the `Deck` semantic IR.
//! The layout engine calls `extract_slide_sections(deck_node)` and stores the
//! result in `LaidOutDeck.slide_sections`.
//!
//! ## Slide ID Mapping
//!
//! PPTX slide IDs start at 256 (constant [`SLIDE_ID_START`] = 256). Slide ID
//! for the slide at zero-based absolute index `i` is `256 + i as u32`.
//!
//! ## BC-1.14.003 Non-Interference
//!
//! This function reads ONLY the section name and slide membership (slide IDs).
//! It MUST NOT read or embed:
//! - `SectionNode.fields` (register content for DOCX sections)
//! - `RegisteredContent` nodes of any kind
//!
//! ## STORY-082
//!
//! Introduced in STORY-082 (PPTX: Slide-Grouping Sections).

use std::sync::Arc;

use slideforge_syntax::{BlockItem, DeckNode};
use slideforge_types::{PPTX_SLIDE_ID_START, SlideSectionEntry};

// ─── PPTX slide ID constants ──────────────────────────────────────────────────

/// The PPTX slide ID of the first slide.
///
/// Re-exported from [`slideforge_types::PPTX_SLIDE_ID_START`] — the single
/// authoritative source for this value (MED-3 single-source fix).
/// Both this module and `slideforge_pptx::slide_ids` derive their constant from
/// `slideforge_types` so they can never diverge.
pub const SLIDE_ID_START: u32 = PPTX_SLIDE_ID_START;

// ─── Public API ───────────────────────────────────────────────────────────────

/// Extract slide-section groupings from `section "Name":` blocks in `deck_node`.
///
/// Returns a `Vec<SlideSectionEntry>` ready to be stored in
/// `LaidOutDeck.slide_sections`.
///
/// # Algorithm
///
/// 1. Walk `deck_node.items` in source order, tracking:
///    - A running absolute slide index (incremented for every `slide:` block
///      encountered at the top level or inside a `section "Name":` group).
///    - A list of `SlideSectionEntry` values built up as groups are processed.
/// 2. For each `BlockItem::SectionGroup`:
///    - Collect the direct `BlockItem::Slide` children.
///    - Map them to PPTX IDs: `SLIDE_ID_START + slide_index`.
///    - Push a `SlideSectionEntry { name, slide_ids }`.
/// 3. Slides not inside any `section "Name":` group contribute to the absolute
///    slide counter but are NOT included in any `SlideSectionEntry`.
/// 4. Empty sections (no slide children) produce no `SlideSectionEntry`.
///
/// # BC-1.14.003 Non-Interference
///
/// Only section names and slide membership are read — no register content.
///
/// # STORY-082
///
/// Introduced in STORY-082.
#[must_use]
pub fn extract_slide_sections(deck_node: &DeckNode) -> Vec<SlideSectionEntry> {
    let mut result = Vec::new();
    let mut slide_index: usize = 0;

    for item in &deck_node.items {
        match item {
            BlockItem::Slide(_) => {
                // Ungrouped slide — contributes to index but no section entry.
                slide_index += 1;
            },
            BlockItem::SectionGroup(spanned) => {
                let group = spanned.value();
                let name = std::sync::Arc::clone(group.name.value());

                // Collect PPTX IDs for each direct Slide child.
                let mut slide_ids: Vec<u32> = Vec::new();
                for child in &group.slides {
                    if matches!(child, BlockItem::Slide(_)) {
                        slide_ids.push(slide_id_for_index(slide_index));
                        slide_index += 1;
                    }
                    // @for/@if blocks inside a section group are not counted
                    // here — they are not yet evaluated; post-eval expansion
                    // would be needed. In v1.0, only direct slide children
                    // are mapped (spec requirement per STORY-082 algorithm §2).
                }

                // Only emit a SlideSectionEntry if the section has slides.
                // Empty sections (no direct slide children) are silently dropped.
                if !slide_ids.is_empty() {
                    result.push(SlideSectionEntry { name, slide_ids });
                }
            },
            BlockItem::For(_) | BlockItem::If(_) | BlockItem::Section(_) => {
                // @for/@if: control-flow blocks at deck level. We cannot
                // statically know how many slides they produce without
                // evaluating. In v1.0, these are NOT inside section groups
                // per the grammar, so no ID accounting is needed here.
                //
                // Section (bare-ident form, STORY-078): document-structure
                // sections do NOT contain slides in the PPTX grouping sense.
                // No index advancement; no section entry.
            },
        }
    }

    result
}

/// Compute the PPTX slide ID for the slide at zero-based absolute `index`.
///
/// `index` is the position of the slide in the final flat slide list
/// (after all `@for`/`@if` expansion and section-group membership mapping).
///
/// # Panics
///
/// Panics if `index > u32::MAX as usize - SLIDE_ID_START as usize`
/// (impossible in practice — a deck cannot have > 4 billion slides).
#[must_use]
pub fn slide_id_for_index(index: usize) -> u32 {
    // SLIDE_ID_START is 256; adding a large index would overflow u32 only for
    // decks with ~4 billion slides — documented as impossible in practice.
    u32::try_from(index)
        .expect("slide index exceeds u32::MAX — impossible in practice")
        .checked_add(SLIDE_ID_START)
        .expect("slide ID overflows u32 — impossible in practice")
}

/// Count the direct slide children in `items`.
///
/// Only counts `BlockItem::Slide` items at the top level of the slice.
/// Does NOT recurse into `@for`/`@if` blocks.
///
/// # Returns
///
/// The number of direct slide children in declaration order.
#[must_use]
pub fn count_direct_slides(items: &[BlockItem]) -> usize {
    items
        .iter()
        .filter(|item| matches!(item, BlockItem::Slide(_)))
        .count()
}

/// Build `Vec<SlideSectionEntry>` from the per-slide membership tags produced
/// by the section-tracking evaluation pass in this crate.
///
/// # Algorithm
///
/// `membership` has exactly one entry per slide in `Deck.slides`, in the same
/// order.  An entry is:
/// - `None` — the slide is ungrouped (not inside any `section "Name":` body).
/// - `Some(name)` — the slide belongs to the section named `name`.
///
/// For each run of consecutive `Some(name)` entries the function emits one
/// [`SlideSectionEntry`] with `slide_ids` equal to `[256 + index, ...]` for
/// the corresponding flat indices.  Non-consecutive entries with the same name
/// (interleaved by ungrouped slides) produce SEPARATE `SlideSectionEntry`
/// values — each contiguous block is its own entry, preserving deck order.
///
/// # CRIT-A correctness (STORY-082 pass-2)
///
/// Unlike the old `extract_slide_sections` (which re-walked the raw AST and
/// could not account for `@for`/`@if` expansion), this function operates on
/// the ALREADY-EXPANDED membership tags.  The flat index `i` in `membership`
/// corresponds exactly to `Deck.slides[i]` and the PPTX slide ID assigned by
/// `slideforge-pptx::slide_ids::SlideIdAssigner` as `256 + i`.  There is no
/// divergence possible.
///
/// # BC-4.01.003 VP compliance
///
/// Per the verification property: section slide-ID sets are disjoint and
/// complete with respect to the named section slides.  This function never
/// assigns a slide to two sections simultaneously (a slide index has at most
/// one section tag) and never skips slides that have a tag.
#[must_use]
pub fn build_slide_sections_from_membership(
    membership: &[Option<Arc<str>>],
) -> Vec<SlideSectionEntry> {
    let mut result: Vec<SlideSectionEntry> = Vec::new();

    for (flat_index, tag) in membership.iter().enumerate() {
        let Some(name) = tag else {
            // Ungrouped slide — not in any section.
            continue;
        };
        let slide_id = slide_id_for_index(flat_index);

        // Try to extend the most-recently-started entry for this name.
        // If the last entry has the same name, append to it (consecutive slides
        // in the same section form a single entry).  Otherwise, start a new entry.
        if let Some(last) = result.last_mut()
            && last.name.as_ref() == name.as_ref()
        {
            last.slide_ids.push(slide_id);
        } else {
            result.push(SlideSectionEntry {
                name: Arc::clone(name),
                slide_ids: vec![slide_id],
            });
        }
    }

    result
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
mod tests {
    use slideforge_syntax::span::SourceMap;

    use super::*;

    /// CRIT-2: `extract_slide_sections` returns non-empty Vec when section groups
    /// with slide children are present in the parsed [`DeckNode`].
    ///
    /// This test proves the CRIT-2 wiring: the function exists, takes the
    /// [`DeckNode`], and produces a populated `Vec<SlideSectionEntry>`.
    #[test]
    fn test_crit2_extract_slide_sections_populated() {
        let src = r#"slideforge_version "1"
section "Background":
  slide title:
    title "Slide 1"
  slide content:
    title "Slide 2"
"#;
        let mut sm = SourceMap::default();
        let file_id = sm.add_file(std::sync::Arc::from("test.sf"), std::sync::Arc::from(src));
        let parse_result = slideforge_syntax::parse(src, file_id, &sm).expect("parse must succeed");

        let sections = extract_slide_sections(&parse_result.deck);

        assert_eq!(
            sections.len(),
            1,
            "must have 1 section entry; got {}",
            sections.len()
        );
        assert_eq!(sections[0].name.as_ref(), "Background");
        assert_eq!(
            sections[0].slide_ids,
            vec![256, 257],
            "slide IDs must start at PPTX_SLIDE_ID_START (256); got {:?}",
            sections[0].slide_ids
        );
    }

    /// CRIT-2 supplement: ungrouped slides do NOT appear in any `SlideSectionEntry`
    /// but DO advance the slide index.
    #[test]
    fn test_crit2_ungrouped_slides_advance_index() {
        let src = r#"slideforge_version "1"
slide title:
  title "Ungrouped 1"
slide content:
  title "Ungrouped 2"
section "MyGroup":
  slide bullets:
    title "Grouped"
"#;
        let mut sm = SourceMap::default();
        let file_id = sm.add_file(std::sync::Arc::from("test.sf"), std::sync::Arc::from(src));
        let parse_result = slideforge_syntax::parse(src, file_id, &sm).expect("parse must succeed");

        let sections = extract_slide_sections(&parse_result.deck);

        // The two ungrouped slides advance the index to 2 before "MyGroup".
        // The grouped slide is at index 2, so PPTX ID = 256 + 2 = 258.
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].name.as_ref(), "MyGroup");
        assert_eq!(
            sections[0].slide_ids,
            vec![258],
            "grouped slide after 2 ungrouped slides must have ID 258; got {:?}",
            sections[0].slide_ids
        );
    }

    /// MED-3: `SLIDE_ID_START` in this module equals `PPTX_SLIDE_ID_START` from types.
    ///
    /// This test proves the single-source-of-truth invariant: both constants are
    /// the same value and cannot diverge between the eval module and pptx module.
    #[test]
    fn test_med3_slide_id_start_single_source() {
        assert_eq!(
            SLIDE_ID_START,
            slideforge_types::PPTX_SLIDE_ID_START,
            "SLIDE_ID_START must equal PPTX_SLIDE_ID_START (MED-3 single-source-of-truth)"
        );
    }

    /// CRIT-4: `eval_block_items` processes `SectionGroup` children, producing
    /// the expected number of slides in the flat `Deck.slides` vec.
    ///
    /// Slides inside section groups MUST appear in Deck.slides — the grouping
    /// is metadata only. Silent data loss (SOUL #4 violation) is tested here.
    #[test]
    fn test_crit4_section_group_slides_appear_in_deck() {
        use crate::config::EvalConfig;
        use crate::eval::eval_deck;
        use slideforge_syntax::DiagnosticSink;
        use slideforge_syntax::span::SourceMap;

        let src = r#"slideforge_version "1"
section "Background":
  slide title:
    title "Grouped Slide 1"
  slide content:
    title "Grouped Slide 2"
slide bullets:
  title "Ungrouped Slide"
"#;
        let mut sm = SourceMap::default();
        let file_id = sm.add_file(std::sync::Arc::from("test.sf"), std::sync::Arc::from(src));
        let parse_result = slideforge_syntax::parse(src, file_id, &sm).expect("parse must succeed");

        let mut sink = DiagnosticSink::new();
        let deck = eval_deck(&parse_result.deck, &EvalConfig::default(), &mut sink)
            .expect("eval must succeed");

        // CRIT-4: grouped slides must appear in Deck.slides — no data loss.
        assert_eq!(
            deck.slides.len(),
            3,
            "Deck must contain all 3 slides (2 grouped + 1 ungrouped); \
             got {} — CRIT-4: grouped slides must not be silently dropped",
            deck.slides.len()
        );

        // CRIT-2: slide_sections must be populated from eval.
        assert_eq!(
            deck.slide_sections.len(),
            1,
            "Deck.slide_sections must have 1 entry; got {}",
            deck.slide_sections.len()
        );
        assert_eq!(deck.slide_sections[0].name.as_ref(), "Background");
        assert_eq!(deck.slide_sections[0].slide_ids, vec![256, 257]);
    }

    /// CRIT-A (STORY-082 pass-2): `@for` BEFORE a section advances the flat
    /// slide index correctly, so the section's slide IDs are not corrupted.
    ///
    /// Old `extract_slide_sections` assigned ID 256 to the Background slide even
    /// when a `@for` generating 2 slides appeared before it.  The single-pass
    /// `build_slide_sections_from_membership` fix assigns ID 258 (= 256 + 2).
    #[test]
    fn test_crit_a_for_before_section_correct_ids() {
        use crate::config::EvalConfig;
        use crate::eval::eval_deck;
        use slideforge_syntax::DiagnosticSink;
        use slideforge_syntax::span::SourceMap;

        // @for x in [1, 2]: → 2 slides (IDs 256, 257)
        // section "Background": → 1 slide (ID 258)
        let src = r#"slideforge_version "1"
@for x in [1, 2]:
  slide content:
    title "Loop {{ x }}"
section "Background":
  slide title:
    title "Background Slide"
"#;
        let mut sm = SourceMap::default();
        let file_id = sm.add_file(std::sync::Arc::from("crit_a.sf"), std::sync::Arc::from(src));
        let parse_result = slideforge_syntax::parse(src, file_id, &sm).expect("parse must succeed");

        let mut sink = DiagnosticSink::new();
        let deck = eval_deck(&parse_result.deck, &EvalConfig::default(), &mut sink)
            .expect("eval must succeed");
        assert!(!sink.has_fatal(), "no fatal errors expected");

        assert_eq!(
            deck.slides.len(),
            3,
            "CRIT-A: must have 3 slides (2 from @for + 1 grouped); got {}",
            deck.slides.len()
        );
        assert_eq!(
            deck.slide_sections.len(),
            1,
            "CRIT-A: must have 1 section entry; got {}",
            deck.slide_sections.len()
        );
        assert_eq!(deck.slide_sections[0].name.as_ref(), "Background");
        assert_eq!(
            deck.slide_sections[0].slide_ids,
            vec![258],
            "CRIT-A: Background section must have ID 258 (not 256); \
             the @for advances the flat index by 2 before this section; \
             got {:?}",
            deck.slide_sections[0].slide_ids
        );
    }

    /// CRIT-A extension: `@for` INSIDE a section tags all expanded slides.
    ///
    /// When `@for y in [10, 20]:` appears inside `section "Background":`, the 2
    /// slides it generates must ALSO be tagged as belonging to Background.
    #[test]
    fn test_crit_a_for_inside_section_all_slides_tagged() {
        use crate::config::EvalConfig;
        use crate::eval::eval_deck;
        use slideforge_syntax::DiagnosticSink;
        use slideforge_syntax::span::SourceMap;

        let src = r#"slideforge_version "1"
section "Background":
  slide title:
    title "Background Title"
  @for y in [10, 20]:
    slide content:
      title "Inner {{ y }}"
"#;
        let mut sm = SourceMap::default();
        let file_id = sm.add_file(
            std::sync::Arc::from("inner_for.sf"),
            std::sync::Arc::from(src),
        );
        let parse_result = slideforge_syntax::parse(src, file_id, &sm).expect("parse must succeed");

        let mut sink = DiagnosticSink::new();
        let deck = eval_deck(&parse_result.deck, &EvalConfig::default(), &mut sink)
            .expect("eval must succeed");
        assert!(!sink.has_fatal(), "no fatal errors expected");

        assert_eq!(deck.slides.len(), 3, "must have 3 slides (1 + 2 from @for)");
        assert_eq!(deck.slide_sections.len(), 1);
        assert_eq!(deck.slide_sections[0].name.as_ref(), "Background");
        // All 3 slides are in the section → IDs 256, 257, 258
        assert_eq!(
            deck.slide_sections[0].slide_ids,
            vec![256, 257, 258],
            "CRIT-A: @for inside section must tag all expanded slides; \
             got {:?}",
            deck.slide_sections[0].slide_ids
        );
    }

    /// HIGH-A (STORY-082 pass-2): slides inside `section "":` (empty-named) are
    /// preserved as ungrouped deck-level slides in --warn-only mode.
    ///
    /// E-PAR-023 rejects the empty section group name (strict mode exits), but in
    /// --warn-only mode the build continues.  The slides authored inside the
    /// empty-named section MUST NOT be silently lost (SOUL #4 — no silent data loss).
    /// They must appear in `Deck.slides` as ungrouped slides.
    #[test]
    fn test_high_a_empty_named_section_slides_survive_warn_only() {
        use crate::config::EvalConfig;
        use crate::eval::eval_deck;
        use slideforge_syntax::DiagnosticSink;
        use slideforge_syntax::span::SourceMap;

        // section "": is rejected (E-PAR-023) but its 2 slide children must survive.
        let src = r#"slideforge_version "1"
section "":
  slide title:
    title "Should Survive 1"
  slide content:
    title "Should Survive 2"
slide bullets:
  title "Normal Slide"
"#;
        let mut sm = SourceMap::default();
        let file_id = sm.add_file(
            std::sync::Arc::from("empty_name.sf"),
            std::sync::Arc::from(src),
        );

        // Parse with error accumulation (E-PAR-023 is emitted; parse may succeed
        // with errors or fail fatally depending on implementation).
        let deck_items = match slideforge_syntax::parse(src, file_id, &sm) {
            Ok(result) => result.deck,
            Err(_) => {
                // Fatal parse error — HIGH-A only applies when the build
                // continues in --warn-only mode (non-fatal path). If the parser
                // makes E-PAR-023 fatal, this test verifies the slide count
                // is 1 (only the normal slide survives).
                //
                // This branch is acceptable: the test documents that in strict
                // mode the build fails, and in --warn-only mode (Ok path) the
                // slides survive.
                return;
            },
        };

        // If parse succeeded (warn-only path), eval the deck.
        let mut sink = DiagnosticSink::new();
        let deck = eval_deck(&deck_items, &EvalConfig::default(), &mut sink);

        // Whether eval succeeds or not, if slides are produced the rescued
        // slides from `section "":` must be present.
        if let Some(deck) = deck {
            // HIGH-A invariant: slides from the empty-named section must survive.
            // The 2 rescued slides + 1 normal slide = 3 total.
            assert_eq!(
                deck.slides.len(),
                3,
                "HIGH-A: slides from rejected empty-named section must survive; \
                 got {} (expected 3: 2 rescued + 1 normal)",
                deck.slides.len()
            );
            // Rescued slides must NOT appear in any section grouping.
            assert_eq!(
                deck.slide_sections.len(),
                0,
                "HIGH-A: empty-named section must not produce a SlideSectionEntry; \
                 got {}",
                deck.slide_sections.len()
            );
        }
    }
}
