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
    todo!(
        "STORY-082: implement extract_slide_sections — walk deck_node.items, \
         find SectionGroup blocks, map slide children to PPTX IDs starting at 256, \
         return Vec<SlideSectionEntry>. \
         BC-1.14.003: do NOT read register content from SectionNode.fields."
    )
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
    todo!(
        "STORY-082: implement slide_id_for_index — return SLIDE_ID_START + index as u32"
    )
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
    todo!(
        "STORY-082: implement count_direct_slides — count BlockItem::Slide items in `items`"
    )
}
