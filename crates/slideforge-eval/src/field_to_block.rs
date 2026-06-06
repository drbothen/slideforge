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
//!
//! ## Current state (STORY-086 stub)
//!
//! This module is a **no-op stub**. The function signature is present so that:
//! 1. `build_inner` in `slideforge/src/lib.rs` can wire the Stage 2b call site.
//! 2. The failing test suite can be written against the correct API surface.
//!
//! The implementation is in the **implementing phase** (STORY-086 T6). Until
//! T6 completes, `thread_fields_to_blocks` does nothing and `Slide.blocks`
//! remains `vec![]` after the call — causing all content-threading tests to fail
//! (Red Gate discipline, LESSON-17).

use slideforge_types::Deck;

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
/// # AltText contract
///
/// When constructing `ContentBlock::Chart`, `ContentBlock::Image`, or
/// `ContentBlock::Diagram`, the threading pass uses the following alt-resolution
/// precedence (see ADR-019 Decision 4, AltText state machine):
///
/// 1. `Slide.fields["decorative"] == Value::Bool(true)` → `ContentBlock.alt = Some(AltText::Decorative)`
/// 2. `Slide.fields["alt"] == Value::Str(s)` (non-empty, non-whitespace) → `ContentBlock.alt = Some(AltText::Provided(Arc::from(s)))`
/// 3. Neither present → `ContentBlock.alt = None`
///
/// The layout `thread_media_alt_into_frames` function then maps `None` to
/// `AltText::Unspecified` on the resulting frame (ADR-019 Decision 5).
///
/// # Idempotency
///
/// If `Slide.blocks` is already non-empty for a slide, this function
/// appends to it rather than replacing it. In practice, `eval_deck`
/// always produces `Slide.blocks = vec![]`, so this is a no-op guard.
///
/// # Current state (STORY-086 stub)
///
/// **This function is a no-op.** `Slide.blocks` is not populated by this call.
/// The implementation is delivered in STORY-086 T6 (implementer phase).
/// All tests that assert on `Slide.blocks` content WILL FAIL until T6 ships.
///
/// [`eval_deck`]: slideforge_eval::eval_deck
pub fn thread_fields_to_blocks(_deck: &mut Deck) {
    // STORY-086 stub: no-op. Red Gate: all content-threading tests must fail
    // until the implementer fills this body with the fields→blocks mapping
    // per ADR-019 Decision 3 and BC-1.16.001 postconditions 1–14.
}
