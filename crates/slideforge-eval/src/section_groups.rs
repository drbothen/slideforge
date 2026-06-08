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

use slideforge_layout::SlideSectionEntry;
use slideforge_syntax::{BlockItem, DeckNode};

// ─── PPTX slide ID constants ──────────────────────────────────────────────────

/// The PPTX slide ID of the first slide.
///
/// All slide IDs are assigned sequentially starting from this value.
/// Matches `slideforge_pptx::slide_ids::SLIDE_ID_START`.
pub const SLIDE_ID_START: u32 = 256;

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
