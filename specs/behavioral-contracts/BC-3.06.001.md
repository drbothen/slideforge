---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-25T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-05
capability: CAP-010
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-3.06.001: Layout Transformation Preserves Slide Count

## Description

`layout::run(deck, brand)` must produce a `LaidOutDeck` whose `slides` vector contains
exactly as many elements as the input `Deck.slides` vector. No slides may be dropped,
duplicated, or reordered during the transformation. If the function cannot satisfy this
invariant due to an internal bug, it must return `Err(LayoutError::SlideCountMismatch)`
rather than silently producing incorrect output.

## Preconditions

1. `deck` is a fully evaluated `Deck` IR — all `{{ }}` interpolations resolved, all
   `@for` / `@if` expansions materialized, all validation passes cleared.
2. `deck.slides.len() >= 1` (zero-slide decks are caught upstream by BC-3.03.004;
   the layout engine receives only non-empty decks under normal operation).
3. `brand` is a fully loaded `Brand` value with at least a default page size.

## Postconditions

1. On success, `laid_out_deck.slides.len() == deck.slides.len()`.
2. Each `LaidOutSlide` at index `i` has `source_index == i`, maintaining a 1:1 ordered
   correspondence with `deck.slides[i]`.
3. On any internal failure that would produce an incorrect slide count,
   `layout::run` returns `Err(LayoutError::SlideCountMismatch { expected: usize, actual: usize })`
   with `expected == deck.slides.len()`.
4. No slide is silently dropped — partial output is never written to the `LaidOutDeck`.

## Invariants

1. The slide count invariant must hold for ALL input sizes: 0-slide (returns Err),
   1-slide, N-slide, and very large decks. (DI-009)
2. `layout::run` is a pure function — it has no side effects, no I/O, and cannot
   produce non-deterministic slide counts. (DI-009)
3. Slide ordering in `LaidOutDeck` matches `Deck` ordering. Layout never reorders slides.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Zero-slide `Deck` input (defensive case; caught upstream) | `Err(LayoutError::EmptyDeck)` returned; no `LaidOutDeck` produced |
| EC-002 | `@for` loop over 100-item collection expanded before layout | `laid_out_deck.slides.len() == 100`; no slide dropped |
| EC-003 | `@if`-excluded slides already removed from `Deck` before layout | Layout only sees included slides; slide count is the post-evaluation count |
| EC-004 | Deck with exactly 1 slide | `laid_out_deck.slides.len() == 1`; `source_index == 0` |
| EC-005 | Internal region map missing for a slide type (bug path) | `Err(LayoutError::UnknownSlideType { kind })` returned; partial `LaidOutDeck` never written |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `Deck { slides: vec![slide_title, slide_content, slide_closing] }` | `Ok(LaidOutDeck { slides: [s0, s1, s2] })`; `slides.len() == 3` | happy-path |
| `Deck { slides: vec![slide] }` | `Ok(LaidOutDeck { slides: [s0] })`; `slides.len() == 1` | happy-path |
| `Deck { slides: vec![] }` | `Err(LayoutError::EmptyDeck)` | edge-case |
| Proptest: arbitrary `Deck` with 1..=50 slides and valid brand | `result.map(|d| d.slides.len()) == Ok(deck.slides.len())` for all inputs | property-test |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-011 | `layout::run` preserves slide count for all valid non-empty `Deck` inputs | proptest (Arbitrary Deck with bounded size); Kani-amenable pure function |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 |
| Capability Anchor Justification | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 — "Each type enforces its own required fields and layout rules via the SlideType plugin trait." The slide-count invariant is a direct consequence of the SlideType-driven layout producing exactly one LaidOutSlide per input Slide. |
| L2 Domain Invariants | DI-009 (Two-IR Model Integrity — Deck and LaidOutDeck are distinct and complete); DI-011 (All IR Types Must Implement Hash + Eq + Clone — required for proptest Arbitrary) |
| Architecture Module | `slideforge-layout` crate — `layout::run()` function; SS-05 (Layout Engine) |
| Stories | STORY-026 |

## Related BCs

- BC-3.06.002 — composes with (determinism requires slide count preservation as a precondition)
- BC-3.06.003 — composes with (valid EMU coordinates presuppose slide was produced, not dropped)
- BC-3.03.004 — depends on (zero-slide decks are caught by validation before reaching layout)
- BC-4.01.001 — downstream of (PPTX exporter consumes `LaidOutDeck`; correct slide count is a precondition for PPTX correctness)

## Architecture Anchors

- `architecture/module-decomposition.md` — SS-05 Layout Engine subsystem
- `architecture/purity-boundary-map.md` — `layout::run` is classified as pure core

## Story Anchor

STORY-026

## VP Anchors

VP-011
