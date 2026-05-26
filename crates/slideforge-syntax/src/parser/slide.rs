//! Slide block parser combinator.
//!
//! The `slide_block_parser` function is defined in `deck.rs` (where it
//! participates in the full deck grammar). This module re-exports it for
//! external consumers and provides additional slide-specific helpers.
//!
//! In STORY-006 the slide parser is fully contained within the deck combinator
//! (to avoid combinator lifetime issues with separate function boundaries).
//! This module exists as a named boundary for STORY-007 and later stories that
//! will extend the slide grammar with control flow, variants, and tags.

// Re-export from deck for callers who want the slide parser directly.
// (Not yet needed in STORY-006; the re-export future-proofs the module boundary.)
