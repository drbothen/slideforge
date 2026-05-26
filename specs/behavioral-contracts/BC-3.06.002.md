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

# BC-3.06.002: Layout Transformation Is Deterministic (Same Inputs Produce Identical LaidOutDeck)

## Description

`layout::run(deck, brand)` is a pure deterministic function: given identical `Deck` and
`Brand` inputs (byte-for-byte equal under their `Hash + Eq` implementations), it must
produce an identical `LaidOutDeck` — including all `BoundingBox` coordinates, `TextFlow`
values, `PageSize`, and `speaker_notes` content. No randomness, no global mutable state,
no timestamp injection, no filesystem reads occur inside `layout::run`. This property
enables reproducible builds and is required for comemo incremental compilation.

## Preconditions

1. `deck_a.eq(&deck_b)` is `true` (same `Deck` IR, field-for-field).
2. `brand_a.eq(&brand_b)` is `true` (same `Brand` configuration, including page size and font metrics).
3. Both calls are made in the same process or across processes on the same platform (cross-platform
   EMU values are guaranteed to be identical because `Emu::from_inches` uses integer truncation,
   not floating-point rounding that varies by FPU mode).

## Postconditions

1. `layout::run(&deck_a, &brand_a) == layout::run(&deck_b, &brand_b)` when
   `deck_a == deck_b` and `brand_a == brand_b`.
2. This equality holds for all nested fields: `LaidOutDeck.page_size`, every
   `LaidOutSlide.frames`, every `Frame.bounding_box`, every `TextFlow` estimation.
3. Two successive calls with the same inputs in the same process produce the same result
   (no internal mutation of global state between calls).

## Invariants

1. `layout::run` performs no I/O — no file reads, no network access, no system clock
   access, no environment variable reads. (DI-009)
2. All coordinate values in the output are derived solely from the input `Deck` and `Brand`,
   using integer EMU arithmetic only (`Emu(i64)`). No `f64` intermediate values accumulate
   rounding errors. (DI-010)
3. Region maps for all 31 slide types are `const` or pure `fn` — no mutable lookup tables,
   no lazy initialization with nondeterministic order.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Same deck run twice in the same process without mutation | Both calls return `Ok(laid_out_deck)` with `laid_out_deck_a == laid_out_deck_b` |
| EC-002 | Deck with `@for`-expanded 50 slides (large output) | Determinism holds at scale; all 50 `LaidOutSlide` values are identical across runs |
| EC-003 | Brand with non-default page size (non-standard dimensions) | Determinism holds; region maps scale proportionally and identically |
| EC-004 | Deck with empty `speaker_notes` on some slides | `LaidOutSlide.speaker_notes == None` deterministically for those slides |
| EC-005 | Two threads call `layout::run` concurrently with the same inputs | No race condition; both return identical results (pure function, no shared mutable state) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Two `Deck::clone()` instances with 3 slides, default brand | `run(deck_a, brand) == run(deck_b, brand)` | happy-path |
| Same deck, default brand, called twice sequentially | `result_1 == result_2` | happy-path |
| Deck with 31 different slide types (one of each), default brand | `run(deck, brand) == run(deck.clone(), brand.clone())` | happy-path |
| Proptest: arbitrary `(Deck, Brand)` pair | `run(deck.clone(), brand.clone()) == run(deck, brand)` for all inputs | property-test |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-011 | `layout::run` is referentially transparent for equal inputs | proptest: `run(d.clone(), b.clone()) == run(d, b)` for Arbitrary Deck and Brand |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 |
| Capability Anchor Justification | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 — "Each type enforces its own required fields and layout rules via the SlideType plugin trait." Determinism is a direct consequence of the SlideType region-map contract: same type + same page size = same BoundingBox, always. |
| L2 Domain Invariants | DI-009 (Two-IR Model Integrity — layout produces a complete, stable LaidOutDeck with no side effects); DI-010 (Integer EMU for All Coordinates — integer arithmetic eliminates FPU-induced nondeterminism); DI-011 (Hash + Eq + Clone on all IR types — enables equality assertion in tests and comemo cache keys) |
| Architecture Module | `slideforge-layout` crate — `layout::run()` function; SS-05 (Layout Engine) |
| Stories | STORY-026 |

## Related BCs

- BC-3.06.001 — composes with (determinism implies slide count is also invariant across calls)
- BC-3.06.003 — composes with (deterministic coordinates are also valid coordinates)
- BC-4.01.002 — downstream of (PPTX multi-renderer fidelity requires deterministic layout output; SSIM/PSNR comparisons would be undefined if layout were nondeterministic)

## Architecture Anchors

- `architecture/module-decomposition.md` — SS-05 Layout Engine; pure-core classification
- `architecture/purity-boundary-map.md` — `layout::run` is on the pure side of the effectful/pure boundary

## Story Anchor

STORY-026

## VP Anchors

VP-011
