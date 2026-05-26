---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-069
title: "Proptest Suites: brand (VP-012) + pptx (VP-013) + layout (VP-011)"
epic: EPIC-20
wave: 6
points: 5
priority: P0
tdd_mode: facade
status: draft
crate: slideforge-brand
subsystems: [SS-04, SS-05, SS-06]
target_module: slideforge-brand
behavioral_contracts: []
# BC status: pending PO authorship — Wave 6 formal-verification stories prove existing
# behavioral guarantees. BCs exercised: BC-2.01.002 (VP-012), BC-4.01.001 (VP-013),
# DI-009 + DI-012 (VP-011). No new BCs introduced here.
verification_properties: [VP-011, VP-012, VP-013]
nfr_refs: []
assumption_validations: []
risk_mitigations: []
depends_on:
  - STORY-023
  - STORY-026
  - STORY-037
blocks: []
estimated_days: 2
---

# STORY-069: Proptest Suites: brand (VP-012) + pptx (VP-013) + layout (VP-011)

## Summary

Add property-based test suites (proptest) for three determinism and structural
integrity properties spanning `slideforge-brand`, `slideforge-layout`, and
`slideforge-pptx`.

1. **VP-012 (proptest suite):** 12-slot palette round-trip — synthesize brand from
   `brand.toml` → extract palette from resulting template → same colors recovered.
2. **VP-013 (proptest suite):** Every synthesized PPTX is a valid ZIP archive
   containing `[Content_Types].xml`.
3. **VP-011 (proptest suite):** Layout determinism — `LaidOutDeck` always has the
   same slide count as the input `Deck`.

These are proptest suites (not Kani proofs), so they run on ALL platforms including
Windows. They were partially established during Phase 3 (proptest skeletons in STORY-023
and STORY-026), and this story delivers the FULL proptest suites to Phase 6 quality.

**tdd_mode: facade** — combined scaffold+impl delivery. The proptest suites ARE the
product. Quality gate is wave-gate mutation testing rather than Red Gate density.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3,500 |
| `crates/slideforge-brand/tests/proptest_palette.rs` | ~2,500 |
| `crates/slideforge-pptx/tests/proptest_zip.rs` | ~2,000 |
| `crates/slideforge-layout/tests/proptest_slide_count.rs` | ~1,500 |
| Referenced brand source (`synthesis.rs`, extraction) | ~5,000 |
| Referenced layout source | ~3,000 |
| **Total** | **~17,500** |

> 17,500 tokens ≈ 17% of a 100k-token context window. Within the 20-30% budget.

## Acceptance Criteria

### AC-001: VP-012 proptest suite passes (12-slot palette round-trip)
Proptest suite in `crates/slideforge-brand/tests/proptest_palette.rs` with strategy
`arb_brand_toml()` generates random but valid `brand.toml` configurations (all 12
OOXML color slots populated with valid sRGB hex values), synthesizes a brand template,
then extracts the palette from the synthesized template. Asserts all 12 color slots
survive the round-trip within ±1 sRGB unit tolerance. 1,000 cases. Zero counterexamples.
(traces to VP-012 — 12-slot palette round-trip: synthesize → extract → same colors;
traces to BC-2.01.002 postcondition)

### AC-002: VP-013 proptest suite passes (every PPTX is valid ZIP)
Proptest suite in `crates/slideforge-pptx/tests/proptest_zip.rs` with strategy
`arb_laid_out_deck(max_slides: 1..=5)` generates random LaidOutDeck values and
synthesizes a PPTX. For each synthesized output, asserts: (a) the output bytes form
a valid ZIP archive (using `zip` crate reader), (b) the archive contains
`[Content_Types].xml`, (c) the archive contains `ppt/presentation.xml`. 500 cases.
Zero counterexamples.
(traces to VP-013 — every synthesized PPTX is valid ZIP with [Content_Types].xml;
traces to BC-4.01.001 postcondition)

### AC-003: VP-011 proptest suite passes (slide count invariant)
Proptest suite in `crates/slideforge-layout/tests/proptest_slide_count.rs` with strategy
`arb_deck(max_slides: 0..=20)` generates random Decks with 0 to 20 slides and runs
the layout engine. Asserts `laid_out_deck.slides.len() == deck.slides.len()` for every
generated case. 1,000 cases. Zero counterexamples.
(traces to VP-011 — LaidOutDeck slide count equals Deck slide count; traces to DI-009)

### AC-004: proptest strategies use valid type domain inputs
All three `arb_*` strategies generate inputs from the VALID domain of the respective
types. They must not generate inputs that are structurally invalid (e.g., color slots
with values > 0xFFFFFF). Invalid inputs are for fuzz harnesses; proptest strategies
use valid-domain generation.

### AC-005: All three suites are included in `just check`
The three proptest test files are compiled and run as part of `cargo test --workspace`.
No `#[ignore]` attribute without a blocking-dependency comment. The suites are not
gated by any feature flag.

### AC-006: Proptest failure output includes counterexample reproduction seed
All three suites configure proptest `with_source_file` and use `proptest::test_runner::
Config` with `failure_persistence` set to a file path. A failing case produces a
reproducible seed in the test output so CI failures can be reproduced locally.

## Tasks

- [ ] 1. Read `crates/slideforge-brand/src/synthesis.rs` (brand synthesis path)
- [ ] 2. Read `crates/slideforge-brand/src/extraction.rs` (palette extraction)
- [ ] 3. Write `arb_brand_toml()` strategy covering all 12 OOXML color slots
         (`dk1`, `dk2`, `lt1`, `lt2`, `acc1`–`acc6`, `hlink`, `folHlink`)
- [ ] 4. Create `crates/slideforge-brand/tests/proptest_palette.rs` with VP-012 suite
- [ ] 5. Read `crates/slideforge-layout/src/lib.rs` (layout entry point `run()`)
- [ ] 6. Write `arb_deck()` strategy for VP-011 — generates Deck with N slides (N: 0..=20)
         using placeholder SlideType::Blank entries
- [ ] 7. Create `crates/slideforge-layout/tests/proptest_slide_count.rs` with VP-011 suite
- [ ] 8. Read `crates/slideforge-pptx/src/lib.rs` (PPTX synthesis entry point)
- [ ] 9. Write `arb_laid_out_deck()` strategy for VP-013 — generates LaidOutDeck with
         positioned but minimal content (1..=5 slides, no charts/diagrams)
- [ ] 10. Create `crates/slideforge-pptx/tests/proptest_zip.rs` with VP-013 suite
- [ ] 11. Add `proptest = "=1.6"` to `[dev-dependencies]` in slideforge-brand, slideforge-layout,
          slideforge-pptx Cargo.toml files
- [ ] 12. Run `cargo test -p slideforge-brand --test proptest_palette` to verify VP-012
- [ ] 13. Run `cargo test -p slideforge-layout --test proptest_slide_count` to verify VP-011
- [ ] 14. Run `cargo test -p slideforge-pptx --test proptest_zip` to verify VP-013

## Previous Story Intelligence

Proptest skeletons were established during Phase 3:
- STORY-023 introduced `proptest = "=1.6"` to `slideforge-brand` dev-deps and
  a skeleton `proptest_palette.rs` with a stub `arb_brand_toml()`. STORY-069 expands
  this skeleton into a full 1,000-case suite covering all 12 color slots.
- STORY-026 introduced a `proptest_slide_count.rs` skeleton. STORY-069 expands it.
- STORY-037 introduced the VP-013 skeleton in `proptest_zip.rs`. STORY-069 expands it.

If the Phase 3 skeletons already pass their basic assertions, verify that STORY-069
EXPANDS them (more cases, more color slots, more assertion clauses) rather than
duplicating. Do not delete the skeleton — improve it.

## Architecture Compliance Rules

Derived from `architecture/verification-architecture.md`:

1. **proptest on all platforms:** Unlike Kani, proptest runs on Windows too. Do NOT
   add any `#[cfg(not(windows))]` guard to these test files.
2. **Use actual production functions:** VP-012 must call the real `synthesize_brand()`
   and `extract_palette()` — not stubs. The test exercises production code paths.
3. **Valid-domain strategies only:** proptest strategies must generate inputs from the
   valid type domain. Fuzz targets (STORY-071) cover arbitrary byte inputs.
4. **Tolerance for VP-012 color round-trip:** Allow ±1 sRGB unit due to integer
   rounding in OOXML XML encoding. Do NOT assert exact bit-equality — that would be
   a false failure. Assert `|extracted[i] - original[i]| <= 1` for each channel.
5. **Forbidden dependencies for VP-011:** `slideforge-layout` test MUST NOT import
   from `slideforge-pptx` or any exporter. The layout test only needs `slideforge-types`
   and `slideforge-layout`.

## Library and Framework Requirements

| Library | Pinned Version | Role | Notes |
|---------|---------------|------|-------|
| proptest | =1.6 | Property-based testing | Already in slideforge-brand dev-deps from STORY-023 skeleton |
| zip | =4.2.0 | ZIP validation in VP-013 | Already in slideforge-pptx production deps |

No new crate deps required. All three test files use existing workspace members.

## File Structure Requirements

Files to CREATE (or significantly expand from skeleton):
- `crates/slideforge-brand/tests/proptest_palette.rs` — VP-012 full suite
- `crates/slideforge-layout/tests/proptest_slide_count.rs` — VP-011 full suite
- `crates/slideforge-pptx/tests/proptest_zip.rs` — VP-013 full suite

Files to MODIFY:
- `crates/slideforge-brand/Cargo.toml` — ensure `proptest = "=1.6"` in dev-deps
- `crates/slideforge-layout/Cargo.toml` — add `proptest = "=1.6"` to dev-deps
- `crates/slideforge-pptx/Cargo.toml` — add `proptest = "=1.6"` to dev-deps

Files to NOT touch:
- Any production source files. proptest suites call production functions.

## Implementation Notes

### VP-012 Strategy and Suite

```rust
// crates/slideforge-brand/tests/proptest_palette.rs
use proptest::prelude::*;
use slideforge_brand::{BrandConfig, synthesize_brand, extract_palette};

/// The 12 OOXML color slot names (canonical order from brand-architecture.md)
const COLOR_SLOTS: [&str; 12] = [
    "dk1", "dk2", "lt1", "lt2",
    "acc1", "acc2", "acc3", "acc4", "acc5", "acc6",
    "hlink", "folHlink",
];

/// Generate a random valid sRGB color as a hex string (e.g., "#1A2B3C")
fn arb_color() -> impl Strategy<Value = String> {
    (0u8..=255u8, 0u8..=255u8, 0u8..=255u8)
        .prop_map(|(r, g, b)| format!("#{:02X}{:02X}{:02X}", r, g, b))
}

/// Generate a BrandConfig with all 12 color slots filled
fn arb_brand_config() -> impl Strategy<Value = BrandConfig> {
    // 12 independent color strategies
    (
        arb_color(), arb_color(), arb_color(), arb_color(),
        arb_color(), arb_color(), arb_color(), arb_color(),
        arb_color(), arb_color(), arb_color(), arb_color(),
    )
    .prop_map(|(dk1, dk2, lt1, lt2, acc1, acc2, acc3, acc4, acc5, acc6, hlink, fol)| {
        BrandConfig {
            colors: std::collections::HashMap::from([
                ("dk1".to_string(), dk1),
                ("dk2".to_string(), dk2),
                ("lt1".to_string(), lt1),
                ("lt2".to_string(), lt2),
                ("acc1".to_string(), acc1),
                ("acc2".to_string(), acc2),
                ("acc3".to_string(), acc3),
                ("acc4".to_string(), acc4),
                ("acc5".to_string(), acc5),
                ("acc6".to_string(), acc6),
                ("hlink".to_string(), hlink),
                ("folHlink".to_string(), fol),
            ]),
            fonts: Default::default(),
            ..Default::default()
        }
    })
}

proptest! {
    #![proptest_config(proptest::test_runner::Config {
        cases: 1_000,
        failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::WithSource("proptest-regressions"))),
        ..Default::default()
    })]

    #[test]
    fn palette_survives_round_trip(config in arb_brand_config()) {
        // Synthesize a brand template
        let template = synthesize_brand(&config)
            .expect("synthesis must not fail for valid config");

        // Extract palette from the synthesized template
        let extracted = extract_palette(&template)
            .expect("extraction must not fail on synthesized template");

        // All 12 slots must be recoverable
        for slot in COLOR_SLOTS {
            let original = config.colors.get(slot).expect("slot must exist");
            let recovered = extracted.get(slot).expect("slot must be in extracted palette");

            // Allow ±1 sRGB unit tolerance for integer rounding in OOXML XML encoding
            let orig_rgb = parse_hex_color(original);
            let recov_rgb = parse_hex_color(recovered);
            prop_assert!(
                orig_rgb.iter().zip(recov_rgb.iter()).all(|(o, r)| (*o as i16 - *r as i16).abs() <= 1),
                "Color slot {} failed round-trip: original={}, recovered={}",
                slot, original, recovered
            );
        }
    }
}

fn parse_hex_color(hex: &str) -> [u8; 3] {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
    [r, g, b]
}
```

### VP-011 Strategy and Suite

```rust
// crates/slideforge-layout/tests/proptest_slide_count.rs
use proptest::prelude::*;
use slideforge_layout::run as layout_run;
use slideforge_types::{Deck, Slide, SlideType, Brand};

fn arb_deck(max_slides: usize) -> impl Strategy<Value = Deck> {
    proptest::collection::vec(
        Just(Slide {
            slide_type: SlideType::Blank,
            blocks: vec![],
            ..Slide::default()
        }),
        0..=max_slides,
    )
    .prop_map(|slides| Deck {
        slides,
        metadata: Default::default(),
    })
}

proptest! {
    #![proptest_config(proptest::test_runner::Config {
        cases: 1_000,
        failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::WithSource("proptest-regressions"))),
        ..Default::default()
    })]

    #[test]
    fn slide_count_invariant(deck in arb_deck(20)) {
        let expected_count = deck.slides.len();
        let brand = Brand::default();
        let result = layout_run(&deck, &brand);
        prop_assert!(result.is_ok(), "layout must not fail for valid Deck");
        let laid_out = result.unwrap();
        prop_assert_eq!(
            laid_out.slides.len(),
            expected_count,
            "LaidOutDeck slide count must equal Deck slide count"
        );
    }
}
```

### VP-013 Strategy and Suite

```rust
// crates/slideforge-pptx/tests/proptest_zip.rs
use proptest::prelude::*;
use slideforge_pptx::synthesize_pptx;
use slideforge_types::{LaidOutDeck, LaidOutSlide, Brand};
use std::io::Cursor;

fn arb_laid_out_deck(max_slides: usize) -> impl Strategy<Value = LaidOutDeck> {
    proptest::collection::vec(
        Just(LaidOutSlide { shapes: vec![], ..LaidOutSlide::default() }),
        1..=max_slides,
    )
    .prop_map(|slides| LaidOutDeck {
        slides,
        brand: Brand::default(),
    })
}

proptest! {
    #![proptest_config(proptest::test_runner::Config {
        cases: 500,
        failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::WithSource("proptest-regressions"))),
        ..Default::default()
    })]

    #[test]
    fn pptx_is_valid_zip_with_content_types(deck in arb_laid_out_deck(5)) {
        let bytes = synthesize_pptx(&deck).expect("synthesis must not fail for valid deck");

        // Validate ZIP structure
        let cursor = Cursor::new(&bytes);
        let mut zip = zip::ZipArchive::new(cursor)
            .expect("output must be a valid ZIP archive");

        // [Content_Types].xml is required by the OPC specification
        prop_assert!(
            zip.by_name("[Content_Types].xml").is_ok(),
            "synthesized PPTX must contain [Content_Types].xml"
        );

        // ppt/presentation.xml is required for PPTX
        prop_assert!(
            zip.by_name("ppt/presentation.xml").is_ok(),
            "synthesized PPTX must contain ppt/presentation.xml"
        );
    }
}
```

## Dependencies

### Dependency Justification

- STORY-069 depends on STORY-023 because VP-012 calls `synthesize_brand()` and
  `extract_palette()` implemented in that story. Without complete brand synthesis,
  the round-trip test has nothing to prove.
- STORY-069 depends on STORY-026 because VP-011 calls the layout engine `run()`
  implemented there. The slide count invariant is a postcondition of that function.
- STORY-069 depends on STORY-037 because VP-013 calls `synthesize_pptx()` from
  that story. Without complete PPTX serialization, the ZIP structure test is vacuous.
- STORY-069 does not block any other story.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | BrandConfig with all colors set to `#000000` — VP-012 | Round-trip: all slots recovered as `#000000`; tolerance check passes |
| EC-002 | BrandConfig with all colors set to `#FFFFFF` — VP-012 | Round-trip: all 12 slots recovered as `#FFFFFF` |
| EC-003 | Deck with 0 slides — VP-011 | layout_run returns Ok(LaidOutDeck { slides: [] }); count == 0 |
| EC-004 | Deck with 20 slides (max strategy bound) — VP-011 | layout_run returns LaidOutDeck with 20 slides |
| EC-005 | LaidOutDeck with 1 slide — VP-013 | synthesize_pptx returns valid ZIP |
| EC-006 | LaidOutDeck with 5 slides (max strategy bound) — VP-013 | synthesize_pptx returns valid ZIP with all slides present |
| EC-007 | proptest finds counterexample (regression) | Seed saved to `proptest-regressions/` file; CI logs reproducible seed |
| EC-008 | Color round-trip off by exactly 1 unit (±1 tolerance) — VP-012 | proptest: tolerance clause passes |

## Test Strategy

- All three suites are proptest property-based tests (not Kani). They run on all
  5 CI platforms including Windows.
- VP-012: 1,000 cases × 12-slot round-trip. Each case exercises the full brand
  synthesis → OOXML XML encoding → extraction pipeline.
- VP-011: 1,000 cases × 0–20 slides. Each case exercises the layout engine.
- VP-013: 500 cases × 1–5 slides (lower case count due to PPTX synthesis overhead).
- Failure persistence: counterexample seeds saved to `proptest-regressions/` for
  reproducibility. These files are gitignored in CI but kept locally.

---

*Subsystem anchor justification: SS-04 (Brand) owns VP-012; SS-05 (Layout Engine)
owns VP-011; SS-06 (PPTX Export) owns VP-013 — each VP targets the canonical subsystem
for the property being tested, per ARCH-INDEX Subsystem Registry.*
