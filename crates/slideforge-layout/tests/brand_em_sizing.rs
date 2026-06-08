//! STORY-074 integration tests — brand-aware em→EMU conversion.
//!
//! ## Red Gate contract
//!
//! Before the STORY-074 implementation:
//! - `layout::run` passes `DEFAULT_EM_IN_EMU` (457_200) to `layout_shapes`,
//!   not `brand.fonts.font_size_emu`.
//! - Every test that provides a brand with `font_size_emu != 457_200` WILL
//!   FAIL because the shape frames carry `Emu(457_200)` regardless of brand.
//!
//! The implementation closes this by replacing the `DEFAULT_EM_IN_EMU` constant
//! at the `layout::run` call-site with `brand.fonts.font_size_emu`.
//!
//! ## BC coverage
//!
//! | AC | BC clause | Test |
//! |----|-----------|------|
//! | AC-001 | BC-3.04.001 PC-2 brand-aware em resolution | `test_bc_3_04_001_ac001_*` |
//! | AC-003 | BC-3.04.001 Invariant 2 backward compat | `test_bc_3_04_001_ac003_*` |

#![allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::doc_markdown,
    clippy::too_many_lines
)]

use std::sync::Arc;

use slideforge_layout::{FrameContent, run, shapes::layout_shapes};
use slideforge_types::{
    AltText, Block, Brand, BrandFonts, BrandPalette, ContentBlock, Deck, DeckMetadata, Emu,
    FillSpec, OrderedMap, ShapePosition, ShapeSpec, ShapeUnit, Slide, SourceSpan,
};

// ─────────────────────────────────────────────────────────────────────────────
// Test helpers
// ─────────────────────────────────────────────────────────────────────────────

fn make_metadata() -> DeckMetadata {
    DeckMetadata {
        title: Some(Arc::from("STORY-074 Test Deck")),
        slideforge_version: Arc::from("0.1.0"),
        lang: Some(Arc::from("en-US")),
        author: None,
        section_order: None,
    }
}

fn make_deck(slides: Vec<Slide>) -> Deck {
    Deck {
        slides,
        vars: OrderedMap::new(),
        metadata: make_metadata(),
        registers: OrderedMap::new(),
        section_blocks: vec![],
    }
}

fn make_slide_with_shape(shape_spec: ShapeSpec) -> Slide {
    Slide {
        slide_type: Arc::from("title"),
        fields: OrderedMap::new(),
        blocks: vec![Block {
            content: ContentBlock::Shape(shape_spec),
            label: None,
            span: SourceSpan::default(),
        }],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
    }
}

/// Build a brand with a specific body font size in EMU.
fn make_brand_with_font_size_emu(font_size_emu: i64) -> Brand {
    Brand {
        name: Arc::from("test-brand"),
        palette: BrandPalette {
            primary: Arc::from("#003087"),
            secondary: Arc::from("#0066CC"),
            accent: Arc::from("#FF6B35"),
            neutral: Arc::from("#F5F5F5"),
        },
        fonts: BrandFonts {
            heading: Arc::from("Calibri Light"),
            body: Arc::from("Calibri"),
            mono: Arc::from("Courier New"),
            font_size_emu,
        },
        layouts: vec![],
        span: SourceSpan::default(),
    }
}

/// Build a `ShapeSpec` at `x = ShapeUnit::Em(milliem)`, with all other positions
/// in inches, and explicit alt text.
fn em_x_shape_spec(x_milliem: i64) -> ShapeSpec {
    let st = slideforge_types::ShapeType::from_keyword("rect")
        .expect("rect is a valid shape type");
    ShapeSpec {
        shape_type: st,
        position: ShapePosition {
            x: ShapeUnit::Em(x_milliem),
            y: ShapeUnit::Inches(0),
            width: ShapeUnit::Inches(2000),
            height: ShapeUnit::Inches(1000),
        },
        fill: FillSpec::None,
        text: None,
        alt: Some(AltText::Provided(Arc::from("test shape"))),
        decorative: false,
        span: SourceSpan::default(),
    }
}

/// Build a `ShapeSpec` at `width = ShapeUnit::Em(milliem)`, with x/y/height in inches.
fn em_width_shape_spec(width_milliem: i64) -> ShapeSpec {
    let st = slideforge_types::ShapeType::from_keyword("rect")
        .expect("rect is a valid shape type");
    ShapeSpec {
        shape_type: st,
        position: ShapePosition {
            x: ShapeUnit::Inches(0),
            y: ShapeUnit::Inches(0),
            width: ShapeUnit::Em(width_milliem),
            height: ShapeUnit::Inches(1000),
        },
        fill: FillSpec::None,
        text: None,
        alt: Some(AltText::Provided(Arc::from("test shape"))),
        decorative: false,
        span: SourceSpan::default(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-001 — Brand font_size_emu drives em→EMU conversion via layout::run
// (BC-3.04.001 Postcondition 2 / STORY-074 AC-001)
//
// Canonical test vectors from the story spec AC-001:
//   brand_48pt.font_size_emu = 609_600  → Em(1000) resolves to Emu(609_600)
//   brand_36pt.font_size_emu = 457_200  → Em(1000) resolves to Emu(457_200)
//
// RED GATE RATIONALE:
//   Before the fix, layout::run passes DEFAULT_EM_IN_EMU (457_200) to layout_shapes.
//   With a 48pt brand, the shape x=Em(1000) produces Emu(457_200) instead of
//   Emu(609_600). The assertion `frame.bbox.x == Emu(609_600)` fails.
// ─────────────────────────────────────────────────────────────────────────────

/// AC-001 (canonical) — Shape at `x=Em(1000)` with a 48pt brand (font_size_emu=609_600)
/// resolves to `BoundingBox.x = Emu(609_600)` after `layout::run`.
///
/// Formula: `em_emu = em_milliems * brand_font_size_emu / 1_000`
///          `Emu(609_600) = 1000 * 609_600 / 1_000`
///
/// This is the exact test vector mandated by STORY-074 AC-001.
///
/// RED GATE: fails because `layout::run` uses `DEFAULT_EM_IN_EMU = 457_200`,
/// producing `Emu(457_200)` instead of `Emu(609_600)`.
#[test]
fn test_bc_3_04_001_ac001_layout_run_48pt_brand_em1_resolves_to_609600() {
    let brand_48pt = make_brand_with_font_size_emu(609_600);
    let shape_spec = em_x_shape_spec(1_000); // x = 1em
    let slide = make_slide_with_shape(shape_spec);
    let deck = make_deck(vec![slide]);

    let laid_out = run(&deck, &brand_48pt).expect("layout::run must succeed");
    assert_eq!(laid_out.slides.len(), 1, "one slide in → one laid-out slide");

    // The shape frame is appended after the region-map frames.
    // Find the Shape frame in the laid-out slide.
    let shape_frame = laid_out.slides[0]
        .frames
        .iter()
        .find(|f| matches!(&f.content, FrameContent::Shape(_)))
        .expect("shape frame must be present in LaidOutSlide");

    assert_eq!(
        shape_frame.bbox.x,
        Emu(609_600),
        "AC-001 (canonical): x=Em(1000) with 48pt brand must resolve to Emu(609_600), \
         not Emu(457_200); this test FAILS before STORY-074 implementation \
         (RED GATE: layout::run uses DEFAULT_EM_IN_EMU instead of brand.fonts.font_size_emu)"
    );
}

/// AC-001 — `width=Em(2000)` with a 48pt brand resolves to `Emu(1_219_200)`.
///
/// Formula: `1_219_200 = 2000 * 609_600 / 1_000`
///
/// RED GATE: fails because `layout::run` uses `DEFAULT_EM_IN_EMU = 457_200`,
/// producing `Emu(914_400)` (= 2000 * 457_200 / 1000) instead of `Emu(1_219_200)`.
#[test]
fn test_bc_3_04_001_ac001_layout_run_48pt_brand_em2_width_resolves_to_1219200() {
    let brand_48pt = make_brand_with_font_size_emu(609_600);
    let shape_spec = em_width_shape_spec(2_000); // width = 2em
    let slide = make_slide_with_shape(shape_spec);
    let deck = make_deck(vec![slide]);

    let laid_out = run(&deck, &brand_48pt).expect("layout::run must succeed");

    let shape_frame = laid_out.slides[0]
        .frames
        .iter()
        .find(|f| matches!(&f.content, FrameContent::Shape(_)))
        .expect("shape frame must be present");

    assert_eq!(
        shape_frame.bbox.width,
        Emu(1_219_200),
        "AC-001: width=Em(2000) with 48pt brand must resolve to Emu(1_219_200); \
         RED GATE: fails with Emu(914_400) before STORY-074 implementation"
    );
}

/// AC-001 — Two brands with different `font_size_emu` values produce different
/// EMU coordinates for the same `ShapeUnit::Em` position.
///
/// A 24pt brand (font_size_emu=304_800) and a 48pt brand (font_size_emu=609_600)
/// must produce different frame x coordinates for the same `Em(1000)` input.
///
/// RED GATE: before the fix, both brands use DEFAULT_EM_IN_EMU=457_200 and
/// produce identical frames. The assertion that they differ FAILS.
#[test]
fn test_bc_3_04_001_ac001_different_brands_produce_different_em_resolution() {
    let brand_24pt = make_brand_with_font_size_emu(304_800); // 24pt body font
    let brand_48pt = make_brand_with_font_size_emu(609_600); // 48pt body font

    let make_em_deck = || {
        let shape_spec = em_x_shape_spec(1_000);
        make_deck(vec![make_slide_with_shape(shape_spec)])
    };

    let laid_out_24pt = run(&make_em_deck(), &brand_24pt).expect("24pt layout must succeed");
    let laid_out_48pt = run(&make_em_deck(), &brand_48pt).expect("48pt layout must succeed");

    let x_24pt = laid_out_24pt.slides[0]
        .frames
        .iter()
        .find(|f| matches!(&f.content, FrameContent::Shape(_)))
        .expect("shape frame must be present for 24pt brand")
        .bbox.x;

    let x_48pt = laid_out_48pt.slides[0]
        .frames
        .iter()
        .find(|f| matches!(&f.content, FrameContent::Shape(_)))
        .expect("shape frame must be present for 48pt brand")
        .bbox.x;

    assert_ne!(
        x_24pt, x_48pt,
        "AC-001: brands with different font_size_emu must produce different EMU coordinates \
         for the same Em unit; RED GATE: before fix both produce Emu(457_200)"
    );
    assert_eq!(
        x_24pt, Emu(304_800),
        "24pt brand: x=Em(1000) must resolve to Emu(304_800); \
         RED GATE: fails with Emu(457_200)"
    );
    assert_eq!(
        x_48pt, Emu(609_600),
        "48pt brand: x=Em(1000) must resolve to Emu(609_600); \
         RED GATE: fails with Emu(457_200)"
    );
}

/// AC-001 — `layout_shapes` called directly with 48pt `em_in_emu=609_600` and
/// `ShapeUnit::Em(1000)` produces a frame with `BoundingBox.x = Emu(609_600)`.
///
/// This is a unit test of the `layout_shapes` function itself (not via `layout::run`).
/// Since `layout_shapes` already accepts `em_in_emu: i64`, this test PASSES even
/// before STORY-074 implementation — it confirms the em-resolution math is correct.
///
/// The integration test (`test_bc_3_04_001_ac001_layout_run_48pt_brand_em1_resolves_to_609600`)
/// tests the wiring from `layout::run` → `layout_shapes`.
#[test]
fn test_bc_3_04_001_ac001_layout_shapes_direct_48pt_em1_resolves_to_609600() {
    use slideforge_layout::{PageSize, shapes::DEFAULT_EM_IN_EMU};
    use slideforge_types::Emu as TypesEmu;

    let page = PageSize {
        width: TypesEmu(12_192_000),
        height: TypesEmu(6_858_000),
    };

    let st = slideforge_types::ShapeType::from_keyword("rect").expect("rect is valid");
    let shape_spec = ShapeSpec {
        shape_type: st,
        position: ShapePosition {
            x: ShapeUnit::Em(1_000), // 1em
            y: ShapeUnit::Inches(0),
            width: ShapeUnit::Inches(2_000),
            height: ShapeUnit::Inches(1_000),
        },
        fill: FillSpec::None,
        text: None,
        alt: Some(AltText::Provided(Arc::from("test shape"))),
        decorative: false,
        span: SourceSpan::default(),
    };

    // Call layout_shapes directly with the 48pt em_in_emu
    let output = layout_shapes(&[shape_spec], page, 0, 609_600, 0)
        .expect("layout_shapes must succeed");

    assert_eq!(
        output.frames[0].bbox.x,
        TypesEmu(609_600),
        "layout_shapes with em_in_emu=609_600 must produce BoundingBox.x=Emu(609_600)"
    );

    // Also assert the DEFAULT_EM_IN_EMU constant is still 457_200 (backward compat guard)
    assert_eq!(
        DEFAULT_EM_IN_EMU, 457_200_i64,
        "DEFAULT_EM_IN_EMU must remain 457_200 (backward compat)"
    );
    let _ = DEFAULT_EM_IN_EMU; // suppress dead_code before AC-002 is implemented
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-003 — BrandFonts::default() backward compatibility
// (BC-3.04.001 Invariant 2 / STORY-074 AC-003)
//
// All pre-STORY-074 code paths used DEFAULT_EM_IN_EMU=457_200.
// After the fix, `BrandFonts::default().font_size_emu` must be 457_200
// so that decks with a default brand produce identical frame positions.
// ─────────────────────────────────────────────────────────────────────────────

/// AC-003 (canonical) — A brand with `font_size_emu: 457_200` (the default)
/// produces `Emu(457_200)` for `Em(1000)` — identical to the pre-STORY-074
/// `DEFAULT_EM_IN_EMU` path.
///
/// This test PASSES before AND after STORY-074 implementation:
/// - Before: `layout::run` uses `DEFAULT_EM_IN_EMU=457_200` (same value)
/// - After: `layout::run` uses `brand.fonts.font_size_emu=457_200` (same value)
///
/// Any regression in the wiring that changes the output for the default brand
/// would cause this test to fail POST-IMPLEMENTATION.
#[test]
fn test_bc_3_04_001_ac003_default_brand_em_resolution_unchanged() {
    // Default brand: font_size_emu = 457_200 (same as DEFAULT_EM_IN_EMU)
    let default_brand = make_brand_with_font_size_emu(457_200);
    let shape_spec = em_x_shape_spec(1_000); // x = 1em
    let slide = make_slide_with_shape(shape_spec);
    let deck = make_deck(vec![slide]);

    let laid_out = run(&deck, &default_brand).expect("layout::run must succeed");

    let shape_frame = laid_out.slides[0]
        .frames
        .iter()
        .find(|f| matches!(&f.content, FrameContent::Shape(_)))
        .expect("shape frame must be present");

    assert_eq!(
        shape_frame.bbox.x,
        Emu(457_200),
        "AC-003: default brand (font_size_emu=457_200) must still resolve Em(1000) to \
         Emu(457_200) — backward compatibility with pre-STORY-074 behavior"
    );
}

/// AC-003 — `BrandFonts::default()` has `font_size_emu = 457_200`.
///
/// The `Default` impl added by STORY-074 must set `font_size_emu` to the
/// historical constant value so that code using `..Default::default()` or
/// `BrandFonts::default()` continues to produce the same em resolution.
///
/// RED GATE: before STORY-074 implementation, `BrandFonts` had no `Default` impl,
/// so this test fails to compile (not just fails at runtime).
/// After the field is added WITH the Default impl, this test passes trivially.
#[test]
fn test_bc_3_04_001_ac003_brand_fonts_default_has_correct_font_size_emu() {
    let fonts = BrandFonts::default();
    assert_eq!(
        fonts.font_size_emu,
        457_200,
        "BrandFonts::default().font_size_emu must be 457_200 (AC-003 backward compat)"
    );
}

/// AC-003 — A deck run through the default brand (via `BrandFonts::default()`)
/// produces the same frame coordinates as the pre-STORY-074 `DEFAULT_EM_IN_EMU`
/// constant path.
///
/// This is the regression guard: no test that PASSED before STORY-074 may FAIL after.
///
/// RED GATE: this test PASSES even before the implementation because the brand
/// value (457_200) matches `DEFAULT_EM_IN_EMU` (457_200). It becomes a regression
/// guard once the implementation wires `brand.fonts.font_size_emu` into `layout::run`.
#[test]
fn test_bc_3_04_001_ac003_layout_run_default_brand_matches_old_constant() {
    let brand = Brand {
        name: Arc::from("default-brand"),
        palette: BrandPalette {
            primary: Arc::from("#000000"),
            secondary: Arc::from("#000000"),
            accent: Arc::from("#000000"),
            neutral: Arc::from("#FFFFFF"),
        },
        fonts: BrandFonts::default(),
        layouts: vec![],
        span: SourceSpan::default(),
    };

    let shape_spec = em_x_shape_spec(2_000); // x = 2em
    let slide = make_slide_with_shape(shape_spec);
    let deck = make_deck(vec![slide]);

    let laid_out = run(&deck, &brand).expect("layout::run must succeed with default brand");

    let shape_frame = laid_out.slides[0]
        .frames
        .iter()
        .find(|f| matches!(&f.content, FrameContent::Shape(_)))
        .expect("shape frame must be present");

    // 2em * 457_200 / 1_000 = 914_400
    assert_eq!(
        shape_frame.bbox.x,
        Emu(914_400),
        "AC-003: default brand Em(2000) must resolve to Emu(914_400) = 2 * 457_200; \
         regression guard for existing pre-STORY-074 tests"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-001 edge cases — integer arithmetic invariants
// (BC-3.04.001 Invariant 2 / STORY-074 AC-001 formula)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-001 edge case — `Em(0)` with any brand font size resolves to `Emu(0)`.
///
/// The formula `0 * font_size_emu / 1_000 = 0` must hold for all font sizes.
///
/// RED GATE: Before STORY-074, layout::run uses DEFAULT_EM_IN_EMU for all brands.
/// After the fix, this test continues to pass because `0 * anything = 0`.
/// The test verifies the edge case is handled correctly by the formula.
#[test]
fn test_bc_3_04_001_ac001_em_zero_resolves_to_emu_zero_regardless_of_brand() {
    let brand = make_brand_with_font_size_emu(609_600);
    let st = slideforge_types::ShapeType::from_keyword("rect").expect("rect is valid");
    let shape_spec = ShapeSpec {
        shape_type: st,
        position: ShapePosition {
            x: ShapeUnit::Em(0), // 0em
            y: ShapeUnit::Inches(0),
            width: ShapeUnit::Inches(2_000),
            height: ShapeUnit::Inches(1_000),
        },
        fill: FillSpec::None,
        text: None,
        alt: Some(AltText::Provided(Arc::from("zero em shape"))),
        decorative: false,
        span: SourceSpan::default(),
    };
    let slide = make_slide_with_shape(shape_spec);
    let deck = make_deck(vec![slide]);

    let laid_out = run(&deck, &brand).expect("layout::run must succeed");
    let shape_frame = laid_out.slides[0]
        .frames
        .iter()
        .find(|f| matches!(&f.content, FrameContent::Shape(_)))
        .expect("shape frame must be present");

    assert_eq!(
        shape_frame.bbox.x,
        Emu(0),
        "AC-001 edge case: Em(0) must always resolve to Emu(0) regardless of font_size_emu"
    );
}

/// AC-001 edge case — `Em(500)` (0.5em) with 48pt brand resolves to `Emu(304_800)`.
///
/// Formula: `500 * 609_600 / 1_000 = 304_800`
///
/// This tests fractional em values (sub-1em positions).
///
/// RED GATE: before fix, produces `Emu(228_600)` = 500 * 457_200 / 1_000.
#[test]
fn test_bc_3_04_001_ac001_half_em_with_48pt_brand_resolves_to_304800() {
    let brand_48pt = make_brand_with_font_size_emu(609_600);
    let st = slideforge_types::ShapeType::from_keyword("rect").expect("rect is valid");
    let shape_spec = ShapeSpec {
        shape_type: st,
        position: ShapePosition {
            x: ShapeUnit::Em(500), // 0.5em
            y: ShapeUnit::Inches(0),
            width: ShapeUnit::Inches(2_000),
            height: ShapeUnit::Inches(1_000),
        },
        fill: FillSpec::None,
        text: None,
        alt: Some(AltText::Provided(Arc::from("half-em shape"))),
        decorative: false,
        span: SourceSpan::default(),
    };
    let slide = make_slide_with_shape(shape_spec);
    let deck = make_deck(vec![slide]);

    let laid_out = run(&deck, &brand_48pt).expect("layout::run must succeed");
    let shape_frame = laid_out.slides[0]
        .frames
        .iter()
        .find(|f| matches!(&f.content, FrameContent::Shape(_)))
        .expect("shape frame must be present");

    assert_eq!(
        shape_frame.bbox.x,
        Emu(304_800),
        "AC-001 edge case: Em(500) (0.5em) with 48pt brand must resolve to Emu(304_800); \
         RED GATE: fails with Emu(228_600) before STORY-074 implementation"
    );
}

/// AC-001 edge case — Inches positions are NOT affected by `font_size_emu`.
///
/// A shape at `x=Inches(500)` (0.5 inch) must produce `Emu(457_200)` regardless
/// of which brand font size is configured. The em conversion only affects
/// `ShapeUnit::Em` positions — not `ShapeUnit::Inches`.
///
/// This guards against a regression where the `font_size_emu` change might
/// accidentally touch the `Inches` conversion path.
///
/// RED GATE: this test PASSES both before AND after STORY-074 (no regression
/// expected for Inches positions). It is a defensive invariant guard.
#[test]
fn test_bc_3_04_001_ac001_inches_positions_unaffected_by_font_size_emu() {
    let brand_48pt = make_brand_with_font_size_emu(609_600);
    let st = slideforge_types::ShapeType::from_keyword("rect").expect("rect is valid");
    let shape_spec = ShapeSpec {
        shape_type: st,
        position: ShapePosition {
            x: ShapeUnit::Inches(500), // 0.5 inch = 457_200 EMU
            y: ShapeUnit::Inches(1_000),
            width: ShapeUnit::Inches(2_000),
            height: ShapeUnit::Inches(1_000),
        },
        fill: FillSpec::None,
        text: None,
        alt: Some(AltText::Provided(Arc::from("inches shape"))),
        decorative: false,
        span: SourceSpan::default(),
    };
    let slide = make_slide_with_shape(shape_spec);
    let deck = make_deck(vec![slide]);

    let laid_out = run(&deck, &brand_48pt).expect("layout::run must succeed");
    let shape_frame = laid_out.slides[0]
        .frames
        .iter()
        .find(|f| matches!(&f.content, FrameContent::Shape(_)))
        .expect("shape frame must be present");

    // x=Inches(500) = 500 * 914_400 / 1_000 = 457_200 EMU — independent of brand
    assert_eq!(
        shape_frame.bbox.x,
        Emu(457_200),
        "Inches positions must not be affected by font_size_emu; \
         x=Inches(500) must always produce Emu(457_200)"
    );
}

/// AC-001 integration — Multiple shapes in one slide: em shapes use brand font size,
/// inch shapes use the standard conversion. All shapes in the slide are processed
/// correctly in a single `layout::run` call.
///
/// RED GATE: the Em shape assertion fails before STORY-074 implementation.
#[test]
fn test_bc_3_04_001_ac001_mixed_em_and_inches_shapes_in_one_slide() {
    let brand_48pt = make_brand_with_font_size_emu(609_600);
    let st = slideforge_types::ShapeType::from_keyword("rect").expect("rect is valid");

    // Shape 1: x=Em(1000) — should resolve to Emu(609_600) with 48pt brand
    let em_shape = ShapeSpec {
        shape_type: st,
        position: ShapePosition {
            x: ShapeUnit::Em(1_000),
            y: ShapeUnit::Inches(0),
            width: ShapeUnit::Inches(1_000),
            height: ShapeUnit::Inches(500),
        },
        fill: FillSpec::None,
        text: None,
        alt: Some(AltText::Provided(Arc::from("em shape"))),
        decorative: false,
        span: SourceSpan::default(),
    };

    // Shape 2: x=Inches(1000) — should resolve to Emu(914_400) regardless of brand
    let inches_shape = ShapeSpec {
        shape_type: st,
        position: ShapePosition {
            x: ShapeUnit::Inches(1_000),
            y: ShapeUnit::Inches(0),
            width: ShapeUnit::Inches(1_000),
            height: ShapeUnit::Inches(500),
        },
        fill: FillSpec::None,
        text: None,
        alt: Some(AltText::Provided(Arc::from("inches shape"))),
        decorative: false,
        span: SourceSpan::default(),
    };

    let slide = Slide {
        slide_type: Arc::from("title"),
        fields: OrderedMap::new(),
        blocks: vec![
            Block {
                content: ContentBlock::Shape(em_shape),
                label: None,
                span: SourceSpan::default(),
            },
            Block {
                content: ContentBlock::Shape(inches_shape),
                label: None,
                span: SourceSpan::default(),
            },
        ],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
    };
    let deck = make_deck(vec![slide]);

    let laid_out = run(&deck, &brand_48pt).expect("layout::run must succeed");
    let shape_frames: Vec<_> = laid_out.slides[0]
        .frames
        .iter()
        .filter(|f| matches!(&f.content, FrameContent::Shape(_)))
        .collect();

    assert_eq!(shape_frames.len(), 2, "two shapes → two shape frames");

    // First shape: Em(1000) with 48pt brand
    assert_eq!(
        shape_frames[0].bbox.x,
        Emu(609_600),
        "first shape: x=Em(1000) with 48pt brand must resolve to Emu(609_600); \
         RED GATE: fails with Emu(457_200) before STORY-074 implementation"
    );

    // Second shape: Inches(1000) — unaffected by brand font size
    assert_eq!(
        shape_frames[1].bbox.x,
        Emu(914_400),
        "second shape: x=Inches(1000) must always resolve to Emu(914_400)"
    );
}
