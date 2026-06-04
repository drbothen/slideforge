//! STORY-039 Red Gate: `layout::run` alt-threading for Chart / Diagram / Image.
//!
//! ## What these tests prove
//!
//! `layout::run` currently produces placeholder frames with `AltText::Decorative`
//! for every chart, diagram, and image slide-type frame — it never reads the
//! semantic spec's `.alt` field. These tests assert the INTENDED behaviour: the
//! alt text from the semantic `ChartSpec` / `DiagramSpec` / `ImageSpec` carried on
//! `Slide.blocks` must be threaded through to the corresponding `FrameContent`
//! variant in `LaidOutSlide.frames`.
//!
//! All four tests MUST FAIL until the implementer wires
//!   `layout.rs` → thread `ChartSpec.alt / DiagramSpec.alt / ImageSpec.alt`
//! into the frame produced for each slide type.
//!
//! ## BC traceability
//!
//! - BC-3.06.NNN (story-039 AC-005 / EC-006 / EC-007)
//! - STORY-039: "layout.rs — thread ChartSpec.alt / DiagramSpec.alt / ImageSpec.alt through run"
//!
//! ## SID-1 compliance
//!
//! These tests exercise the production code path through `layout::run` (NOT
//! hand-constructed `LaidOutDeck`). They do NOT mock or bypass `layout::run`.

#![allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used
)]

use std::sync::Arc;

use slideforge_layout::{FrameContent, run};
use slideforge_types::{
    AltText, Block, Brand, BrandFonts, BrandPalette, ChartSpec, ContentBlock, Deck, DeckMetadata,
    DiagramSpec, ImageSpec, OrderedMap, Slide, SourceSpan,
};

// ─────────────────────────────────────────────────────────────────────────────
// Test helpers — mirror the pattern from bullets_layout_integration.rs
// ─────────────────────────────────────────────────────────────────────────────

fn make_brand() -> Brand {
    Brand {
        name: Arc::from("test-brand"),
        palette: BrandPalette {
            primary: Arc::from("#003087"),
            secondary: Arc::from("#0066CC"),
            accent: Arc::from("#FF6B35"),
            neutral: Arc::from("#F5F5F5"),
        },
        fonts: BrandFonts {
            heading: Arc::from("Calibri"),
            body: Arc::from("Calibri"),
            mono: Arc::from("Courier New"),
        },
        layouts: vec![],
        span: SourceSpan::default(),
    }
}

fn make_deck(slides: Vec<Slide>) -> Deck {
    Deck {
        slides,
        vars: OrderedMap::new(),
        metadata: DeckMetadata {
            title: Some(Arc::from("Alt Threading Test Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        },
        registers: OrderedMap::new(),
        section_blocks: vec![],
    }
}

/// Build a slide of the given `slide_type` with a single `ContentBlock`.
fn slide_with_block(slide_type: &str, block: ContentBlock) -> Slide {
    Slide {
        slide_type: Arc::from(slide_type),
        fields: OrderedMap::new(),
        blocks: vec![Block {
            content: block,
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

// ─────────────────────────────────────────────────────────────────────────────
// Test 1 — Chart slide-type: provided alt threads through layout::run
// ─────────────────────────────────────────────────────────────────────────────

/// STORY-039 AC-005 / EC-006 Red Gate — `layout::run` must thread
/// `ChartSpec.alt = Some(AltText::Provided("Revenue by region"))` from the
/// semantic block into the `FrameContent::Chart { alt }` frame it produces.
///
/// ## Why this FAILS at Red Gate
///
/// `region_frames_for("chart", ...)` returns a `FrameContent::Chart { alt: AltText::Decorative }`
/// placeholder. `layout::run` does NOT iterate `slide.blocks` for `ContentBlock::Chart`
/// and therefore never overwrites the placeholder alt with the author-supplied text.
/// The assertion `alt == AltText::Provided("Revenue by region")` fails because the
/// frame still carries `AltText::Decorative`.
///
/// ## What the implementer must do
///
/// In the slide-processing loop in `layout::run`, for each `ContentBlock::Chart(spec)`,
/// find the `FrameContent::Chart` frame in the placeholder list and replace its `alt`
/// with `spec.alt.unwrap_or(AltText::Decorative)` (emitting `tracing::warn!` when
/// `spec.alt` is `None`).
#[test]
fn test_bc_3_06_039_ac005_chart_provided_alt_threads_through_layout_run() {
    let provided_alt = Arc::from("Revenue by region");
    let chart_block = ContentBlock::Chart(ChartSpec {
        chart_type: Arc::from("bar"),
        alt: Some(AltText::Provided(Arc::clone(&provided_alt))),
        decorative: false,
        span: SourceSpan::default(),
    });
    let slide = slide_with_block("chart", chart_block);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand)
        .expect("layout::run must succeed for chart slide with provided alt text");

    // Find the FrameContent::Chart frame produced for this slide.
    let chart_frame_alt: Vec<&AltText> = result.slides[0]
        .frames
        .iter()
        .filter_map(|f| match &f.content {
            FrameContent::Chart { alt } => Some(alt),
            _ => None,
        })
        .collect();

    assert_eq!(
        chart_frame_alt.len(),
        1,
        "chart slide must produce exactly 1 FrameContent::Chart frame; \
         got {} (total frames: {})",
        chart_frame_alt.len(),
        result.slides[0].frames.len()
    );

    // RED GATE assertion: layout::run does NOT thread alt yet → still Decorative → FAILS.
    assert_eq!(
        chart_frame_alt[0],
        &AltText::Provided(Arc::clone(&provided_alt)),
        "FrameContent::Chart.alt must be Provided(\"Revenue by region\") — \
         threaded from ChartSpec.alt by layout::run; \
         got: {:?} (EXPECTED FAILURE at Red Gate: layout::run does not thread alt)",
        chart_frame_alt[0]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 2 — Diagram slide-type: provided alt threads through layout::run
// ─────────────────────────────────────────────────────────────────────────────

/// STORY-039 AC-005 Red Gate — `layout::run` must thread
/// `DiagramSpec.alt = Some(AltText::Provided("Architecture overview"))` into
/// the `FrameContent::Diagram { alt, .. }` frame it produces.
///
/// ## Why this FAILS at Red Gate
///
/// `region_frames_for("diagram", ...)` returns a
/// `FrameContent::Diagram { svg: empty_placeholder(), alt: AltText::Decorative }`.
/// `layout::run` does NOT iterate `ContentBlock::Diagram` blocks and therefore
/// the placeholder alt is never overwritten. The assertion that
/// `alt == AltText::Provided("Architecture overview")` fails.
///
/// ## What the implementer must do
///
/// In the slide-processing loop in `layout::run`, for each `ContentBlock::Diagram(spec)`,
/// find the `FrameContent::Diagram` frame in the placeholder list and replace its `alt`
/// with `spec.alt.unwrap_or(AltText::Decorative)` (emitting `tracing::warn!` when
/// `spec.alt` is `None`). The `svg` field stays as the existing placeholder; SVG
/// rendering is a separate pipeline concern.
#[test]
fn test_bc_3_06_039_ac005_diagram_provided_alt_threads_through_layout_run() {
    let provided_alt = Arc::from("Architecture overview");
    let diagram_block = ContentBlock::Diagram(DiagramSpec {
        source: Arc::from("graph TD; A-->B"),
        alt: Some(AltText::Provided(Arc::clone(&provided_alt))),
        decorative: false,
        span: SourceSpan::default(),
    });
    let slide = slide_with_block("diagram", diagram_block);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand)
        .expect("layout::run must succeed for diagram slide with provided alt text");

    // Find the FrameContent::Diagram frame produced for this slide.
    let diagram_frame_alt: Vec<&AltText> = result.slides[0]
        .frames
        .iter()
        .filter_map(|f| match &f.content {
            FrameContent::Diagram { alt, .. } => Some(alt),
            _ => None,
        })
        .collect();

    assert_eq!(
        diagram_frame_alt.len(),
        1,
        "diagram slide must produce exactly 1 FrameContent::Diagram frame; \
         got {} (total frames: {})",
        diagram_frame_alt.len(),
        result.slides[0].frames.len()
    );

    // RED GATE assertion: layout::run does NOT thread alt yet → still Decorative → FAILS.
    assert_eq!(
        diagram_frame_alt[0],
        &AltText::Provided(Arc::clone(&provided_alt)),
        "FrameContent::Diagram.alt must be Provided(\"Architecture overview\") — \
         threaded from DiagramSpec.alt by layout::run; \
         got: {:?} (EXPECTED FAILURE at Red Gate: layout::run does not thread alt)",
        diagram_frame_alt[0]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 3 — Screenshot slide-type: provided alt threads through layout::run
// ─────────────────────────────────────────────────────────────────────────────

/// STORY-039 AC-005 Red Gate — `layout::run` must thread
/// `ImageSpec.alt = Some(AltText::Provided("Dashboard screenshot"))` from the
/// semantic block into the `FrameContent::Image { alt }` frame it produces.
///
/// Uses the `"screenshot"` slide type, which maps to a two-frame layout
/// (title + `FrameContent::Image` placeholder). The `"bio"` slide type
/// would also qualify (it has a `FrameContent::Image` frame at index 0).
///
/// ## Why this FAILS at Red Gate
///
/// `region_frames_for("screenshot", ...)` returns a
/// `FrameContent::Image { alt: AltText::Decorative }` placeholder at frame index 1.
/// `layout::run` does NOT iterate `ContentBlock::Image` blocks and therefore the
/// placeholder alt is never overwritten. The assertion fails.
///
/// ## What the implementer must do
///
/// In the slide-processing loop in `layout::run`, for each `ContentBlock::Image(spec)`,
/// find the `FrameContent::Image` frame in the placeholder list and replace its `alt`
/// with `spec.alt.unwrap_or(AltText::Decorative)` (emitting `tracing::warn!` when
/// `spec.alt` is `None`).
#[test]
fn test_bc_3_06_039_ac005_image_provided_alt_threads_through_layout_run() {
    let provided_alt = Arc::from("Dashboard screenshot");
    let image_block = ContentBlock::Image(ImageSpec {
        path: Arc::from("screenshots/dashboard.png"),
        alt: Some(AltText::Provided(Arc::clone(&provided_alt))),
        decorative: false,
        span: SourceSpan::default(),
    });
    let slide = slide_with_block("screenshot", image_block);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand)
        .expect("layout::run must succeed for screenshot slide with provided alt text");

    // Find all FrameContent::Image frames produced for this slide.
    let image_frame_alts: Vec<&AltText> = result.slides[0]
        .frames
        .iter()
        .filter_map(|f| match &f.content {
            FrameContent::Image { alt } => Some(alt),
            _ => None,
        })
        .collect();

    assert_eq!(
        image_frame_alts.len(),
        1,
        "screenshot slide must produce exactly 1 FrameContent::Image frame; \
         got {} (total frames: {})",
        image_frame_alts.len(),
        result.slides[0].frames.len()
    );

    // RED GATE assertion: layout::run does NOT thread alt yet → still Decorative → FAILS.
    assert_eq!(
        image_frame_alts[0],
        &AltText::Provided(Arc::clone(&provided_alt)),
        "FrameContent::Image.alt must be Provided(\"Dashboard screenshot\") — \
         threaded from ImageSpec.alt by layout::run; \
         got: {:?} (EXPECTED FAILURE at Red Gate: layout::run does not thread alt)",
        image_frame_alts[0]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 4 — EC-006: Chart with alt == None maps to AltText::Decorative
// ─────────────────────────────────────────────────────────────────────────────

/// STORY-039 EC-006 Red Gate — When `ChartSpec.alt` is `None` (author did not
/// supply alt text), `layout::run` must produce `FrameContent::Chart { alt: AltText::Decorative }`.
///
/// This is the None → Decorative fallback rule specified by EC-006 / EC-007.
///
/// ## Why this PASSES at Red Gate (but for the wrong reason)
///
/// The placeholder produced by `region_frames_for("chart", ...)` already carries
/// `AltText::Decorative`, so this test passes even without the threading
/// implementation — it passes because the placeholder is NEVER overwritten, which
/// is coincidentally the correct outcome for the `alt: None` case.
///
/// However, the implementer MUST NOT use this as justification to skip threading:
/// once tests 1–3 are implemented (overwriting the placeholder for `Provided` alt),
/// this test becomes the regression guard ensuring `alt: None` still falls back to
/// `Decorative` and does NOT erroneously carry `None` or an empty string.
///
/// This test is included in the Red Gate suite so the implementer cannot accidentally
/// break the fallback while fixing tests 1–3. A correct implementation must pass all
/// four tests simultaneously.
///
/// ## What the implementer must produce
///
/// `FrameContent::Chart { alt: AltText::Decorative }` — this is already the
/// placeholder value, so the implementer only needs to ensure that `None` is
/// explicitly mapped to `Decorative` (not left as a silent empty or propagated as
/// `None`). The `tracing::warn!` must also be emitted per EC-006 spec.
#[test]
fn test_bc_3_06_039_ec006_chart_alt_none_maps_to_decorative() {
    // ChartSpec with alt: None — author provided NO alt text.
    let chart_block = ContentBlock::Chart(ChartSpec {
        chart_type: Arc::from("pie"),
        alt: None,
        decorative: false,
        span: SourceSpan::default(),
    });
    let slide = slide_with_block("chart", chart_block);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand)
        .expect("layout::run must succeed for chart slide with alt: None");

    // Find the FrameContent::Chart frame.
    let chart_frame_alt: Vec<&AltText> = result.slides[0]
        .frames
        .iter()
        .filter_map(|f| match &f.content {
            FrameContent::Chart { alt } => Some(alt),
            _ => None,
        })
        .collect();

    assert_eq!(
        chart_frame_alt.len(),
        1,
        "chart slide must produce exactly 1 FrameContent::Chart frame; \
         got {} (total frames: {})",
        chart_frame_alt.len(),
        result.slides[0].frames.len()
    );

    // EC-006: alt: None in ChartSpec must map to AltText::Decorative in the frame.
    // NOTE: This assertion coincidentally passes at Red Gate because the placeholder
    // already carries Decorative. It becomes a regression guard after tests 1–3 are
    // implemented to ensure None still produces Decorative (not a propagated None).
    assert_eq!(
        chart_frame_alt[0],
        &AltText::Decorative,
        "EC-006: FrameContent::Chart.alt must be AltText::Decorative when ChartSpec.alt is None; \
         got: {:?}",
        chart_frame_alt[0]
    );
}
