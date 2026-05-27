//! Integration tests for the eval → validate pipeline.
//!
//! These tests exercise the cross-crate boundary between `slideforge-eval`
//! and `slideforge-validate`. They verify that a `Deck` produced by
//! `eval_deck` can be fed directly to the validator pipeline and that
//! validators emit the expected diagnostics (or none) based on the
//! evaluator's output.
//!
//! # FINDING-003
//!
//! Wave 2 adversary finding: no cross-crate eval→validate integration test
//! existed. This file closes that gap.
//!
//! # Expected behaviour of `blocks: vec![]` in eval output (F-004)
//!
//! `eval_deck` produces `Slide` values with `blocks: vec![]`. This is
//! intentional: block-level content (images, charts, shapes) is populated
//! by later pipeline stages (Wave 3+). As a result:
//!
//! - `AltTextValidator` and `CanvasOverflowValidator` will find no blocks to
//!   check and emit zero diagnostics — this is correct, not a validator bug.
//! - `LangValidator` inspects `deck.metadata.lang`, which IS populated by
//!   `eval_deck` from `DeckNode.lang`, so it works correctly on eval output.
//!
//! Tests that verify `AltTextValidator` behaviour on eval-produced decks
//! include a comment noting this expected empty-blocks property.

#![allow(clippy::unwrap_used)] // test code only

use slideforge_eval::{EvalConfig, eval_deck};
use slideforge_plugin_api::{Validator, ValidatorOptions};
use slideforge_syntax::span::Span;
use slideforge_syntax::{BlockItem, DeckNode, SlideNode, Spanned};
use slideforge_validate::{LangValidator, inject_lang_default};

// ─── Helpers ────────────────────────────────────────────────────────────────

fn dummy_span() -> Span {
    Span::new(0, 0, 0)
}

fn default_config() -> EvalConfig {
    EvalConfig::default()
}

fn default_opts() -> ValidatorOptions {
    ValidatorOptions::default()
}

/// Build a minimal `DeckNode` with the given slide types and no `lang` declaration.
fn deck_node_no_lang(slide_kinds: &[&str]) -> DeckNode {
    let items = slide_kinds
        .iter()
        .map(|kind| {
            BlockItem::Slide(Spanned::new(
                SlideNode {
                    kind: Spanned::new((*kind).to_string(), dummy_span()),
                    tags: vec![],
                    fields: vec![],
                    inline_items: vec![],
                },
                dummy_span(),
            ))
        })
        .collect();
    DeckNode {
        items,
        ..DeckNode::default()
    }
}

/// Build a minimal `DeckNode` with the given `lang` value.
fn deck_node_with_lang(lang: &str, slide_kinds: &[&str]) -> DeckNode {
    let items = slide_kinds
        .iter()
        .map(|kind| {
            BlockItem::Slide(Spanned::new(
                SlideNode {
                    kind: Spanned::new((*kind).to_string(), dummy_span()),
                    tags: vec![],
                    fields: vec![],
                    inline_items: vec![],
                },
                dummy_span(),
            ))
        })
        .collect();
    DeckNode {
        items,
        lang: Some(Spanned::new(lang.to_string(), dummy_span())),
        ..DeckNode::default()
    }
}

// ─── FINDING-003: eval → LangValidator integration ───────────────────────────

/// FINDING-003 (1/4): `eval_deck` with no `lang` → `LangValidator` emits E-A11-003.
///
/// Pipeline:
///   1. Construct a `DeckNode` with vars but no `lang` declaration.
///   2. Run `eval_deck` to produce a `Deck`.
///   3. Run `LangValidator.validate()` on the resulting `Deck`.
///   4. Assert E-A11-003 is emitted (missing lang).
#[test]
fn test_eval_validate_lang_missing_emits_e_a11_003() {
    let deck_node = deck_node_no_lang(&["title", "content"]);
    let config = default_config();
    let mut sink = slideforge_syntax::DiagnosticSink::new();

    let deck = eval_deck(&deck_node, &config, &mut sink)
        .expect("eval_deck must succeed for a valid 2-slide deck with no lang");
    assert!(
        sink.is_empty(),
        "eval must not push any errors for this deck"
    );

    // lang must be None on the eval-produced deck (no lang was declared).
    assert!(
        deck.metadata.lang.is_none(),
        "eval-produced deck must have lang=None when DeckNode has no lang declaration; got {:?}",
        deck.metadata.lang
    );

    // Now validate — LangValidator must detect the missing lang.
    let diags = LangValidator.validate(&deck, &default_opts());
    let e_a11_003: Vec<_> = diags
        .iter()
        .filter(|d| d.code.as_ref() == "E-A11-003")
        .collect();

    assert_eq!(
        e_a11_003.len(),
        1,
        "LangValidator must emit exactly 1 E-A11-003 for an eval-produced deck with no lang; \
         got {} diagnostic(s): {diags:?}",
        diags.len()
    );
}

/// FINDING-003 (2/4): `eval_deck` with `lang "en"` → `LangValidator` emits no E-A11-003.
#[test]
fn test_eval_validate_lang_present_no_e_a11_003() {
    let deck_node = deck_node_with_lang("en", &["title"]);
    let config = default_config();
    let mut sink = slideforge_syntax::DiagnosticSink::new();

    let deck = eval_deck(&deck_node, &config, &mut sink).expect("eval_deck must succeed");
    assert!(sink.is_empty(), "eval must not push any errors");
    assert_eq!(
        deck.metadata.lang.as_deref(),
        Some("en"),
        "lang must be propagated from DeckNode to Deck; got {:?}",
        deck.metadata.lang
    );

    let diags = LangValidator.validate(&deck, &default_opts());
    assert!(
        diags.is_empty(),
        "LangValidator must emit no E-A11-003 when lang is set; got: {diags:?}"
    );
}

/// FINDING-003 (3/4): `inject_lang_default` sets `lang` to `"en"` on an
/// eval-produced deck that had no lang declaration.
///
/// This tests the full recommended pipeline:
///   eval_deck → LangValidator.validate (emits E-A11-003) → inject_lang_default
///
/// After injection, `deck.metadata.lang` must be `Some("en")`.
#[test]
fn test_eval_validate_inject_lang_default_after_eval() {
    let deck_node = deck_node_no_lang(&["content"]);
    let config = default_config();
    let mut sink = slideforge_syntax::DiagnosticSink::new();

    let mut deck = eval_deck(&deck_node, &config, &mut sink).expect("eval_deck must succeed");
    assert!(sink.is_empty(), "eval must not push errors");

    // Step 1: validate — should emit E-A11-003.
    let diags = LangValidator.validate(&deck, &default_opts());
    assert!(
        !diags.is_empty(),
        "LangValidator must emit E-A11-003 before inject_lang_default"
    );

    // Step 2: inject default lang.
    let injected = inject_lang_default(&mut deck);
    assert!(
        injected,
        "inject_lang_default must return true (lang was absent)"
    );

    // After injection, lang must be "en".
    assert_eq!(
        deck.metadata.lang.as_deref(),
        Some("en"),
        "lang must be 'en' after inject_lang_default; got {:?}",
        deck.metadata.lang
    );

    // Step 3: re-validate — no E-A11-003 after injection.
    let diags_after = LangValidator.validate(&deck, &default_opts());
    assert!(
        diags_after.is_empty(),
        "LangValidator must emit no E-A11-003 after inject_lang_default; got: {diags_after:?}"
    );
}

/// FINDING-003 (4/4): eval-produced slides have `blocks: vec![]` (F-004).
///
/// `AltTextValidator` inspects `slide.blocks`; for eval-produced slides it
/// will find no blocks and emit zero diagnostics. This is correct and expected:
/// blocks are populated by later pipeline stages (Wave 3+).
///
/// This test documents and asserts the F-004 property to prevent future
/// regressions where someone mistakes "zero alt-text warnings" for
/// "alt-text validator is broken".
#[test]
fn test_eval_produced_slides_have_empty_blocks_f004() {
    use slideforge_validate::AltTextValidator;

    let deck_node = deck_node_no_lang(&["content", "bullets"]);
    let config = default_config();
    let mut sink = slideforge_syntax::DiagnosticSink::new();

    let deck = eval_deck(&deck_node, &config, &mut sink).expect("eval_deck must succeed");
    assert!(sink.is_empty(), "eval must not push errors");

    // Verify F-004: all eval-produced slides have empty blocks.
    for (i, slide) in deck.slides.iter().enumerate() {
        assert!(
            slide.blocks.is_empty(),
            "eval-produced slide[{i}] (type='{}') must have empty blocks (F-004); \
             blocks are populated by layout stage (Wave 3+), not by eval_deck",
            slide.slide_type
        );
    }

    // Consequence: AltTextValidator finds nothing to flag.
    // This is expected behaviour — not a validator defect.
    let alt_diags = AltTextValidator.validate(&deck, &default_opts());
    assert!(
        alt_diags.is_empty(),
        "AltTextValidator must emit no diagnostics for eval-produced deck \
         (slides have empty blocks — F-004); got: {alt_diags:?}"
    );
}

// ─── Zero-slide integration ──────────────────────────────────────────────────

/// Cross-crate: eval_deck with 0 items → ZeroSlideValidator emits E-LAY-002.
#[test]
fn test_eval_validate_zero_slide_deck() {
    use slideforge_validate::ZeroSlideValidator;

    let deck_node = DeckNode::default();
    let config = default_config();
    let mut sink = slideforge_syntax::DiagnosticSink::new();

    let deck = eval_deck(&deck_node, &config, &mut sink)
        .expect("eval_deck on empty deck must return Some");
    assert!(sink.is_empty(), "eval must not push errors for empty deck");
    assert_eq!(deck.slides.len(), 0, "deck must have 0 slides");

    let diags = ZeroSlideValidator.validate(&deck, &default_opts());
    assert!(
        !diags.is_empty(),
        "ZeroSlideValidator must emit E-LAY-002 for a deck with 0 slides; got: {diags:?}"
    );
    assert_eq!(
        diags[0].code.as_ref(),
        "E-LAY-002",
        "diagnostic code must be E-LAY-002; got: {}",
        diags[0].code
    );
}

/// Cross-crate: eval_deck with slides → ZeroSlideValidator emits nothing.
#[test]
fn test_eval_validate_non_empty_deck_no_zero_slide_error() {
    use slideforge_validate::ZeroSlideValidator;

    let deck_node = deck_node_no_lang(&["title"]);
    let config = default_config();
    let mut sink = slideforge_syntax::DiagnosticSink::new();

    let deck = eval_deck(&deck_node, &config, &mut sink).expect("eval_deck must succeed");
    assert_eq!(deck.slides.len(), 1, "deck must have 1 slide");

    let diags = ZeroSlideValidator.validate(&deck, &default_opts());
    assert!(
        diags.is_empty(),
        "ZeroSlideValidator must emit no diagnostics for a non-empty deck; got: {diags:?}"
    );
}

// ─── Lang propagation integration ────────────────────────────────────────────

/// Cross-crate: eval_deck propagates `lang "en-US"` to DeckMetadata.lang.
///
/// This is a round-trip test: DeckNode.lang → eval_deck → Deck.metadata.lang.
#[test]
fn test_eval_deck_lang_propagation() {
    let deck_node = deck_node_with_lang("en-US", &["title"]);
    let config = default_config();
    let mut sink = slideforge_syntax::DiagnosticSink::new();

    let deck = eval_deck(&deck_node, &config, &mut sink).expect("eval_deck must succeed");
    assert!(sink.is_empty(), "eval must not push errors");

    assert_eq!(
        deck.metadata.lang.as_deref(),
        Some("en-US"),
        "lang must round-trip from DeckNode to Deck.metadata.lang; got {:?}",
        deck.metadata.lang
    );

    // Inject default — must be a no-op since lang is already set.
    let mut deck_mut = deck;
    let injected = inject_lang_default(&mut deck_mut);
    assert!(
        !injected,
        "inject_lang_default must return false when lang is already set"
    );
    assert_eq!(
        deck_mut.metadata.lang.as_deref(),
        Some("en-US"),
        "lang must be unchanged after inject_lang_default when already set; got {:?}",
        deck_mut.metadata.lang
    );
}

// ─── slideforge_eval public API surface ──────────────────────────────────────

/// Verify that `EvalConfig` is accessible from the public API and has the
/// expected default values (used by the integration test helpers above).
#[test]
fn test_eval_config_default_accessible() {
    let cfg = EvalConfig::default();
    // These invariants are exercised by the integration tests above;
    // this test documents the expected defaults explicitly.
    assert!(
        cfg.max_total_slides.is_none(),
        "default EvalConfig must have max_total_slides=None"
    );
    assert!(
        cfg.large_deck_warn_threshold > 0,
        "default EvalConfig must have a positive large_deck_warn_threshold; got {}",
        cfg.large_deck_warn_threshold
    );
}

/// Verify `eval_deck` is re-exported from the crate root (public API surface).
///
/// This is a compile-time test — if `eval_deck` is not re-exported, the `use`
/// statement above will fail to compile.
#[test]
fn test_eval_deck_is_public_api() {
    // The `use slideforge_eval::eval_deck` at the top of this file is the
    // actual compile-time assertion. This test simply ensures the import is
    // exercised (otherwise the compiler may warn about an unused import).
    let deck_node = DeckNode::default();
    let mut sink = slideforge_syntax::DiagnosticSink::new();
    let deck = eval_deck(&deck_node, &EvalConfig::default(), &mut sink);
    assert!(
        deck.is_some(),
        "eval_deck public API must return Some for empty deck"
    );
}
