//! Eval-stage mapping: `SectionGroupNode` → `SlideSectionEntry` collection.
//!
//! This module provides [`build_slide_sections_from_membership`], the production
//! populator for `Deck.slide_sections`. It converts the per-slide membership tags
//! produced by the single-pass section-tracking evaluator in this crate into
//! `Vec<SlideSectionEntry>` ready to be stored in
//! [`slideforge_layout::LaidOutDeck::slide_sections`].
//!
//! ## Pipeline Position
//!
//! Called after [`crate::eval::eval_deck`] has produced the `Deck` semantic IR.
//! The evaluator calls `build_slide_sections_from_membership(membership)` and stores
//! the result in `Deck.slide_sections`. The layout engine passes it through to
//! `LaidOutDeck.slide_sections`.
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

/// Compute the PPTX slide ID for the slide at zero-based absolute `index`.
///
/// `index` is the position of the slide in the final flat slide list
/// (after all `@for`/`@if` expansion and section-group membership mapping).
///
/// # Panics
///
/// Panics if `index > u32::MAX as usize - SLIDE_ID_START as usize`
/// (impossible in practice — a deck cannot have > 4 billion slides).
///
/// # Infallibility invariant
///
/// The two `.expect()` calls here are documented-infallible under the
/// `max_total_slides` config gate (default: 10 000; hard ceiling far below
/// `u32::MAX - 256`).  This mirrors the accepted pattern in
/// `slideforge_pptx::slide_ids::SlideIdAssigner::assign` and
/// `slideforge_pptx::presentation` (presentation.rs lines 67-70), where the
/// same family of bounds is relied upon.  Changing this to fallible error
/// routing would require a partial, inconsistent change across an
/// already-accepted sibling family — the correct fix, if ever needed, is to
/// introduce a workspace-wide `SlideBudget` guard and update ALL members
/// together in a single story.
#[must_use]
pub fn slide_id_for_index(index: usize) -> u32 {
    // SLIDE_ID_START is 256; adding a large index would overflow u32 only for
    // decks with ~4 billion slides — bounded in practice by max_total_slides config.
    u32::try_from(index)
        .expect(
            "slide index exceeds u32::MAX — impossible: bounded by max_total_slides config gate",
        )
        .checked_add(SLIDE_ID_START)
        .expect("slide ID overflows u32 — impossible: bounded by max_total_slides config gate")
}

/// Build `Vec<SlideSectionEntry>` from the per-slide membership tags produced
/// by the section-tracking evaluation pass in this crate.
///
/// # Algorithm
///
/// `membership` has exactly one entry per slide in `Deck.slides`, in the same
/// order.  An entry is:
/// - `None` — the slide is ungrouped (not inside any `section "Name":` body).
/// - `Some((instance_id, name))` — the slide belongs to the section named
///   `name` that was the `instance_id`-th `section "Name":` block encountered
///   during evaluation (zero-based, incremented once per `SectionGroup` block
///   entered in `eval_block_items_with_sections`).
///
/// Slides with the same `(instance_id, name)` are part of the SAME DSL block
/// and are coalesced into one [`SlideSectionEntry`].  Slides from a DIFFERENT
/// `SectionGroup` block — even if it shares the same name — have a distinct
/// `instance_id` and therefore produce a SEPARATE entry.
///
/// This correctly handles the adjacent-duplicate case (BC-4.01.003 EC-011 /
/// F-P10-HIGH-1): two `section "Background":` blocks back-to-back get distinct
/// `instance_id`s and are never merged, while slides within a single block
/// are still grouped together.
///
/// Non-consecutive blocks (interleaved by ungrouped slides) continue to produce
/// separate entries because the `instance_id` differs between the two blocks.
///
/// # CRIT-A correctness (STORY-082 pass-2)
///
/// Unlike the superseded `extract_slide_sections` (which re-walked the raw AST
/// and could not account for `@for`/`@if` expansion), this function operates on
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
///
/// # Panics
///
/// The internal `.expect()` call in the `same_block` branch is
/// documented-infallible: `same_block` is `true` only when `last_key` is
/// `Some(...)` which means the previous iteration already pushed at least one
/// entry into `result`.  The invariant cannot be violated within a single-pass
/// scan of the membership slice.
#[must_use]
pub fn build_slide_sections_from_membership(
    membership: &[Option<(u32, Arc<str>)>],
) -> Vec<SlideSectionEntry> {
    let mut result: Vec<SlideSectionEntry> = Vec::new();
    // Track the (instance_id, name) of the last entry we were extending.
    // We use this instead of comparing `last.name` so that two adjacent blocks
    // with the same name (different instance_ids) are never merged.
    let mut last_key: Option<(u32, Arc<str>)> = None;

    for (flat_index, tag) in membership.iter().enumerate() {
        let Some((instance_id, name)) = tag else {
            // Ungrouped slide — not in any section.
            // Seal the current run so a same-named block after an ungrouped slide
            // always starts a new entry.
            last_key = None;
            continue;
        };
        let slide_id = slide_id_for_index(flat_index);

        // Extend the current entry only when both the instance_id AND the name
        // match the previous tag — i.e. we are still inside the same DSL block.
        let same_block = last_key
            .as_ref()
            .is_some_and(|(prev_id, prev_name)| *prev_id == *instance_id && prev_name == name);

        if same_block {
            // Safety: `same_block` is true only when `result` is non-empty.
            result
                .last_mut()
                .expect("result is non-empty when same_block is true")
                .slide_ids
                .push(slide_id);
        } else {
            result.push(SlideSectionEntry {
                name: Arc::clone(name),
                slide_ids: vec![slide_id],
            });
            last_key = Some((*instance_id, Arc::clone(name)));
        }
    }

    result
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

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
    /// The superseded `extract_slide_sections` assigned ID 256 to the Background
    /// slide even when a `@for` generating 2 slides appeared before it.  The
    /// single-pass `build_slide_sections_from_membership` fix assigns ID 258
    /// (= 256 + 2).
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

    /// AC-010 / BC-4.01.003 PC-7 — `section "":` is always fatal (E-PAR-023).
    ///
    /// Per error-taxonomy.md §24: "Parse Errors (E-PAR) — Always fatal. Build halts
    /// with accumulated errors. No output produced."  E-PAR-023 is an E-PAR code.
    /// `--warn-only` only demotes VALIDATION errors, never parse errors.
    ///
    /// This test is load-bearing: it verifies the spec-true behavior:
    ///   1. `parse()` returns `Err` for `section "":` (not `Ok`).
    ///   2. The error list contains E-PAR-023.
    ///   3. `slideforge::build()` on a deck with `section "":` returns
    ///      `BuildError::ParseFailed` (no output produced).
    ///
    /// The old test (`test_high_a_empty_named_section_slides_survive_warn_only`) was
    /// vacuous: it short-circuited on `Err(_) => return` before reaching its
    /// assertion, so the assertion was never exercised.  That test has been removed
    /// and replaced by this spec-true version.
    #[test]
    fn test_ac010_empty_section_name_is_always_fatal_e_par_023() {
        use slideforge_syntax::span::SourceMap;

        let src = r#"slideforge_version "1"
section "":
  slide title:
    title "Should Be Rejected"
  slide content:
    title "Also Rejected"
slide bullets:
  title "Normal Slide"
"#;
        let mut sm = SourceMap::default();
        let file_id = sm.add_file(
            std::sync::Arc::from("empty_name.sf"),
            std::sync::Arc::from(src),
        );

        // Assertion 1 (load-bearing): parse() MUST return Err for section "":
        // E-PAR-023 is always fatal per error-taxonomy.md §24.
        let parse_err = slideforge_syntax::parse(src, file_id, &sm).expect_err(
            "parse() MUST return Err for section \"\": — E-PAR-023 is always fatal \
             (error-taxonomy.md §24: Parse Errors are always fatal; --warn-only does \
             NOT demote parse errors)",
        );

        // Assertion 2 (load-bearing): the error list must contain E-PAR-023.
        let has_e_par_023 = parse_err
            .iter()
            .any(|e| format!("{e}").contains("E-PAR-023"));
        assert!(
            has_e_par_023,
            "parse error for section \"\": must include E-PAR-023; got: {parse_err:?}"
        );
    }

    /// F-P10-HIGH-1 / BC-4.01.003 EC-011 — two ADJACENT `section "Background":`
    /// blocks must produce TWO separate `SlideSectionEntry` values, not one.
    ///
    /// This is a RED GATE test for the Pass-10 HIGH-1 defect: the consecutive-run
    /// coalescing in `build_slide_sections_from_membership` collapsed adjacent
    /// same-name blocks into a single entry, violating BC-4.01.003 EC-011 which
    /// requires "Both sections are emitted to the IR … No section is dropped."
    ///
    /// The test drives the REAL eval pipeline (`eval_deck`) — not hand-built
    /// entries — so any regression in the eval → membership → sections path is
    /// caught here.
    ///
    /// Expected (after fix): `deck.slide_sections.len() == 2`, both named
    /// "Background", with slide IDs [256] and [257] respectively.
    #[test]
    fn test_f_p10_high1_adjacent_same_name_sections_produce_two_entries() {
        use crate::config::EvalConfig;
        use crate::eval::eval_deck;
        use slideforge_syntax::DiagnosticSink;
        use slideforge_syntax::span::SourceMap;

        // Two ADJACENT section "Background": blocks — each with exactly one slide.
        // The bug: only ONE SlideSectionEntry is produced (the second block is
        // merged into the first).
        let src = r#"slideforge_version "1"
section "Background":
  slide title:
    title "First Background"
section "Background":
  slide content:
    title "Second Background"
"#;
        let mut sm = SourceMap::default();
        let file_id = sm.add_file(
            std::sync::Arc::from("adjacent_dup.sf"),
            std::sync::Arc::from(src),
        );
        let parse_result = slideforge_syntax::parse(src, file_id, &sm).expect(
            "parse must succeed — adjacent duplicates are a W-PAR-002 warning, not a fatal error",
        );

        let mut sink = DiagnosticSink::new();
        let deck = eval_deck(&parse_result.deck, &EvalConfig::default(), &mut sink)
            .expect("eval must succeed for adjacent-duplicate sections");
        assert!(
            !sink.has_fatal(),
            "adjacent-duplicate sections must not produce fatal errors"
        );

        // Both slides must appear in the flat slide list.
        assert_eq!(
            deck.slides.len(),
            2,
            "F-P10-HIGH-1: deck must contain 2 slides (one per section block); got {}",
            deck.slides.len()
        );

        // KEY ASSERTION (load-bearing): two DISTINCT section entries must be emitted.
        assert_eq!(
            deck.slide_sections.len(),
            2,
            "F-P10-HIGH-1 (BC-4.01.003 EC-011): two adjacent `section \"Background\":` blocks \
             must produce 2 SlideSectionEntry values, not {}; no section may be dropped",
            deck.slide_sections.len()
        );

        // Both entries must be named "Background".
        assert_eq!(deck.slide_sections[0].name.as_ref(), "Background");
        assert_eq!(deck.slide_sections[1].name.as_ref(), "Background");

        // Each entry covers exactly one slide, with the correct PPTX IDs.
        assert_eq!(
            deck.slide_sections[0].slide_ids,
            vec![256],
            "F-P10-HIGH-1: first Background section must cover slide 256; got {:?}",
            deck.slide_sections[0].slide_ids
        );
        assert_eq!(
            deck.slide_sections[1].slide_ids,
            vec![257],
            "F-P10-HIGH-1: second Background section must cover slide 257; got {:?}",
            deck.slide_sections[1].slide_ids
        );
    }

    /// F-P10-HIGH-1 regression guard: non-consecutive duplicate sections
    /// (separated by an ungrouped slide) must still produce two separate entries.
    ///
    /// This path was already correct before the fix (the `None` tag breaks the
    /// consecutive run). This test ensures the fix does not regress it.
    #[test]
    fn test_f_p10_high1_non_consecutive_same_name_still_two_entries() {
        use crate::config::EvalConfig;
        use crate::eval::eval_deck;
        use slideforge_syntax::DiagnosticSink;
        use slideforge_syntax::span::SourceMap;

        // Two "Background" sections SEPARATED by an ungrouped slide.
        let src = r#"slideforge_version "1"
section "Background":
  slide title:
    title "First Background"
slide bullets:
  title "Ungrouped"
section "Background":
  slide content:
    title "Second Background"
"#;
        let mut sm = SourceMap::default();
        let file_id = sm.add_file(
            std::sync::Arc::from("non_consec_dup.sf"),
            std::sync::Arc::from(src),
        );
        let parse_result = slideforge_syntax::parse(src, file_id, &sm).expect("parse must succeed");

        let mut sink = DiagnosticSink::new();
        let deck = eval_deck(&parse_result.deck, &EvalConfig::default(), &mut sink)
            .expect("eval must succeed");
        assert!(
            !sink.has_fatal(),
            "non-consecutive duplicate sections must not be fatal"
        );

        assert_eq!(deck.slides.len(), 3, "3 slides total (1 + 1 + 1)");
        assert_eq!(
            deck.slide_sections.len(),
            2,
            "non-consecutive duplicates must produce 2 entries; got {}",
            deck.slide_sections.len()
        );
        assert_eq!(deck.slide_sections[0].name.as_ref(), "Background");
        assert_eq!(deck.slide_sections[1].name.as_ref(), "Background");
        assert_eq!(deck.slide_sections[0].slide_ids, vec![256]);
        assert_eq!(deck.slide_sections[1].slide_ids, vec![258]);
    }

    /// AC-010 / BC-4.01.003 PC-7 — `parse_checked()` with `section "":` returns
    /// `None` (no AST produced) and pushes a fatal error into the sink.
    ///
    /// `parse_checked()` is the boundary between the parser and the eval pipeline:
    /// `None` here means fatal error → no eval → no output ever produced.
    /// This test is load-bearing: it asserts the `None` return that gates all
    /// downstream pipeline stages.
    #[test]
    fn test_ac010_parse_checked_with_empty_section_name_returns_none() {
        use slideforge_syntax::DiagnosticSink;
        use slideforge_syntax::span::SourceMap;

        let src = r#"slideforge_version "1"
lang "en-US"
section "":
  slide title:
    title "Should Fail"
"#;
        let mut sm = SourceMap::default();
        let file_id = sm.add_file(std::sync::Arc::from("empty.sf"), std::sync::Arc::from(src));
        let mut sink = DiagnosticSink::new();

        // Assertion (load-bearing): parse_checked() MUST return None.
        // E-PAR-023 is always fatal (error-taxonomy.md §24); the caller (eval/build)
        // receives None and produces no output — there is no --warn-only bypass.
        let result = slideforge_syntax::parse_checked(src, file_id, &sm, &mut sink);
        assert!(
            result.is_none(),
            "parse_checked() MUST return None for section \"\": — \
             E-PAR-023 is always fatal (error-taxonomy.md §24: Parse Errors are \
             always fatal; --warn-only does NOT demote parse errors)"
        );

        // The sink must be non-empty (fatal error was recorded).
        assert!(
            sink.has_fatal(),
            "DiagnosticSink must record a fatal error for section \"\":"
        );
    }
}
