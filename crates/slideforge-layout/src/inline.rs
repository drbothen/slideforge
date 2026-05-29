//! Inline node validation and xref-target checking — BC-3.05.001 (STORY-028).
//!
//! This module implements the two responsibilities of the layout stage with
//! respect to inline content:
//!
//! 1. **All 11 (actually 12) [`slideforge_types::InlineNode`] variants are
//!    preserved verbatim** in [`crate::types::FrameContent::TextRun`] records
//!    in [`crate::types::LaidOutSlide::frames`]. The layout stage does NOT
//!    produce format-specific markup — it only carries the variant type and its
//!    attributes (AC-005 / BC-3.05.001 postcondition).
//!
//! 2. **Xref target validation (AC-007 / BC-3.05.001 EC-002):** During layout,
//!    `InlineNode::Xref { target }` references are validated against the slide
//!    titles in the [`slideforge_types::Deck`]. Unknown xref targets accumulate
//!    [`crate::types::LayoutWarning::XrefTargetNotFound`] via a warning sink but
//!    do NOT halt layout.
//!
//! ## Nesting (AC-006 / BC-3.05.001 EC-001)
//!
//! Nested inline formatting (e.g., `Bold(vec![Italic(...)])`) is preserved
//! at full nesting depth. Exporters receive the full tree and produce
//! format-specific merged markup (e.g., PPTX `<a:rPr b="1" i="1">`).
//!
//! ## No wildcard catch-all (DI-004)
//!
//! The validation pass MUST handle ALL `InlineNode` variants without a
//! wildcard catch-all. Missing variants are a compile error (AC-005 /
//! architecture rule 4 from the story spec).

use std::collections::HashSet;
use std::sync::Arc;

use slideforge_types::InlineNode;

use crate::types::LayoutWarning;

/// Validate a slice of [`InlineNode`]s and collect any xref warnings.
///
/// This function:
/// - Traverses the full `InlineNode` tree (including nested children) without
///   a wildcard match arm — all 12 variants are handled explicitly.
/// - For every `InlineNode::Xref(target)`, checks whether `target` matches one
///   of the `known_slide_titles`. If not, pushes a
///   [`LayoutWarning::XrefTargetNotFound`] into `warnings`.
/// - Returns the original node sequence unchanged (the layout stage preserves
///   inline content verbatim).
///
/// # Arguments
///
/// * `nodes` — The inline node sequence to validate.
/// * `known_slide_titles` — The set of slide title strings from the deck.
/// * `slide_index` — Zero-based index of the slide being validated.
/// * `warnings` — Mutable sink for accumulated warnings (DI-018).
///
/// # Green-by-design self-check (BC-5.38.005)
///
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — NO. Involves recursive tree
/// traversal and match on all 12 variants. `todo!()`.
pub fn validate_inline_nodes(
    nodes: &[InlineNode],
    known_slide_titles: &HashSet<Arc<str>>,
    slide_index: usize,
    warnings: &mut Vec<LayoutWarning>,
) {
    todo!(
        "BC-3.05.001: validate_inline_nodes — traverse all InlineNode variants (no wildcard), \
         validate Xref targets, push XrefTargetNotFound warnings for unknown targets"
    )
}

/// Collect all slide title strings from the deck into a `HashSet`.
///
/// Used to build the known-titles set for [`validate_inline_nodes`] in a
/// single allocation rather than re-scanning the deck for each slide.
///
/// # Arguments
///
/// * `deck` — The semantic deck IR to scan.
///
/// # Returns
///
/// A `HashSet<Arc<str>>` containing every slide title string found in the deck.
/// Slides without a title field are not represented (they cannot be xref targets).
///
/// # Green-by-design self-check (BC-5.38.005)
///
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — NO. Involves iteration + field
/// lookup + string construction. `todo!()`.
#[must_use]
pub fn collect_slide_titles(deck: &slideforge_types::Deck) -> HashSet<Arc<str>> {
    todo!(
        "BC-3.05.001: collect_slide_titles — scan Deck.slides for title fields, \
         return HashSet<Arc<str>> of all resolved title strings"
    )
}

/// Run the inline validation pass for all slides in a `LaidOutDeck`.
///
/// This is the entry point called by [`crate::layout::run`] after the per-slide
/// frame pass. It:
/// 1. Calls [`collect_slide_titles`] to build the known-titles set.
/// 2. For each `LaidOutSlide`, scans all `FrameContent::TextRun` frames for
///    `InlineNode` sequences.
/// 3. Calls [`validate_inline_nodes`] for each inline sequence.
/// 4. Returns the accumulated `Vec<LayoutWarning>`.
///
/// # Arguments
///
/// * `deck` — The semantic deck IR (used for title collection).
/// * `laid_out_slides` — The laid-out slides (mutably scanned but not modified).
///
/// # Returns
///
/// A `Vec<LayoutWarning>` containing all xref-not-found warnings accumulated
/// across all slides.
///
/// # Green-by-design self-check (BC-5.38.005)
///
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — NO. Involves iteration over slides
/// and frames, pattern matching on FrameContent variants. `todo!()`.
#[must_use]
pub fn run_inline_validation(
    deck: &slideforge_types::Deck,
    laid_out_slides: &[crate::types::LaidOutSlide],
) -> Vec<LayoutWarning> {
    todo!(
        "BC-3.05.001: run_inline_validation — collect slide titles, scan FrameContent::TextRun \
         frames, validate Xref nodes, return warnings"
    )
}

/// Check whether a single [`InlineNode`] subtree contains any xref nodes that
/// reference unknown titles, and push warnings into the sink.
///
/// This is the recursive helper for [`validate_inline_nodes`]. It MUST NOT use
/// a wildcard `_ => {}` catch-all — every `InlineNode` variant must be
/// explicitly handled (AC-005 / architecture rule 4).
///
/// # Green-by-design self-check (BC-5.38.005)
///
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — NO. Recursive, all 12 variants
/// must be listed. `todo!()`.
pub fn check_inline_node(
    node: &InlineNode,
    known_slide_titles: &HashSet<Arc<str>>,
    slide_index: usize,
    warnings: &mut Vec<LayoutWarning>,
) {
    todo!(
        "BC-3.05.001: check_inline_node — match on ALL InlineNode variants (no wildcard), \
         recurse into nested children, push XrefTargetNotFound for unknown Xref targets"
    )
}

#[cfg(test)]
#[allow(clippy::missing_docs_in_private_items, clippy::unwrap_used)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use slideforge_types::{InlineNode, MathNode, SourceSpan};
    #[allow(unused_imports)]
    use std::sync::Arc;

    // ─────────────────────────────────────────────────────────────────────────
    // AC-005 — All 11/12 InlineNode variants represented (BC-3.05.001)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-005 — `validate_inline_nodes` handles all 12 InlineNode variants
    /// without wildcard. An empty warning list is returned for valid nodes.
    ///
    /// Red Gate: `validate_inline_nodes` is `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac005_all_12_inline_variants_survive_validation() {
        panic!(
            "not yet implemented (Red Gate): validate_inline_nodes must handle all 12 InlineNode \
             variants (Plain, Bold, Italic, Code, Link, Math, Footnote, Xref, Superscript, \
             Subscript, Strikethrough, Highlight) without a wildcard catch-all"
        )
    }

    /// AC-005 — `validate_inline_nodes` returns unchanged nodes (no mutation).
    ///
    /// Red Gate: `validate_inline_nodes` is `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac005_nodes_unchanged_after_validation() {
        panic!(
            "not yet implemented (Red Gate): validate_inline_nodes must not mutate the node \
             sequence — layout preserves inline content verbatim"
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-006 — Nested inline formatting preserved (BC-3.05.001 EC-001)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-006 — Nested Bold(Italic(Plain)) tree survives validation unchanged.
    ///
    /// Red Gate: `validate_inline_nodes` is `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac006_nested_bold_italic_preserved() {
        panic!(
            "not yet implemented (Red Gate): validate_inline_nodes must traverse nested Bold(Italic) \
             and preserve nesting depth"
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-007 — Xref target validation at layout time (BC-3.05.001 EC-002)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-007 — `validate_inline_nodes` with a known xref target produces no warning.
    ///
    /// Red Gate: `validate_inline_nodes` is `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac007_known_xref_target_produces_no_warning() {
        panic!(
            "not yet implemented (Red Gate): validate_inline_nodes with a known xref target must \
             produce zero warnings"
        )
    }

    /// AC-007 — `validate_inline_nodes` with unknown xref target pushes
    /// `LayoutWarning::XrefTargetNotFound`.
    ///
    /// Red Gate: `validate_inline_nodes` is `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac007_unknown_xref_target_produces_warning() {
        panic!(
            "not yet implemented (Red Gate): validate_inline_nodes with an unknown xref target \
             must push LayoutWarning::XrefTargetNotFound into the warnings sink"
        )
    }

    /// AC-007 — Multiple unknown xref targets accumulate multiple warnings.
    ///
    /// Red Gate: `validate_inline_nodes` is `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac007_multiple_unknown_xrefs_accumulate_warnings() {
        panic!(
            "not yet implemented (Red Gate): validate_inline_nodes must accumulate ALL unknown \
             xref warnings — not stop at the first one"
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // collect_slide_titles
    // ─────────────────────────────────────────────────────────────────────────

    /// `collect_slide_titles` returns a set containing the deck's slide titles.
    ///
    /// Red Gate: `collect_slide_titles` is `todo!()`.
    #[test]
    fn test_collect_slide_titles_returns_title_set() {
        panic!(
            "not yet implemented (Red Gate): collect_slide_titles must return a HashSet containing \
             all resolved slide title strings from the deck"
        )
    }

    /// `collect_slide_titles` on an empty deck returns an empty set.
    ///
    /// Red Gate: `collect_slide_titles` is `todo!()`.
    #[test]
    fn test_collect_slide_titles_empty_deck_returns_empty_set() {
        panic!(
            "not yet implemented (Red Gate): collect_slide_titles on a zero-slide deck must \
             return an empty HashSet"
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // run_inline_validation integration
    // ─────────────────────────────────────────────────────────────────────────

    /// `run_inline_validation` returns no warnings for a deck with no TextRun frames.
    ///
    /// Red Gate: `run_inline_validation` is `todo!()`.
    #[test]
    fn test_run_inline_validation_no_text_run_frames_no_warnings() {
        panic!(
            "not yet implemented (Red Gate): run_inline_validation on a LaidOutDeck with no \
             TextRun frames must return an empty warnings Vec"
        )
    }

    /// `run_inline_validation` returns a warning for an unknown xref in a TextRun frame.
    ///
    /// Red Gate: `run_inline_validation` is `todo!()`.
    #[test]
    fn test_run_inline_validation_unknown_xref_in_text_run_produces_warning() {
        panic!(
            "not yet implemented (Red Gate): run_inline_validation must detect unknown xref \
             targets in FrameContent::TextRun frames and accumulate warnings"
        )
    }
}
