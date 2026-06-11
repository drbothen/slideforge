//! STORY-039 regression guards: `layout::run` alt-threading for Chart / Diagram / Image.
//!
//! ## What these tests prove
//!
//! `layout::run` threads the alt text from the semantic `ChartSpec` / `DiagramSpec` /
//! `ImageSpec` carried on `Slide.blocks` through to the corresponding `FrameContent`
//! variant in `LaidOutSlide.frames`. These tests guard against regressions that would
//! cause `layout::run` to revert to emitting `AltText::Decorative` placeholders
//! regardless of the author-supplied `.alt` field.
//!
//! All four tests are GREEN. They will FAIL if the alt-threading logic in `layout.rs`
//! is removed or regresses — that is their purpose as regression guards.
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
    AltText, Block, Brand, BrandFonts, BrandPalette, ChartSpec, ColorBarSpec, ContentBlock, Deck,
    DeckMetadata, DiagramSpec, ImageSpec, InlineNode, OrderedMap, Slide, SourceSpan, TextBlock,
    TextTag,
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
            font_size_emu: 457_200,
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
        slide_sections: vec![],
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
        field_spans: OrderedMap::new(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 1 — Chart slide-type: provided alt threads through layout::run
// ─────────────────────────────────────────────────────────────────────────────

/// STORY-039 AC-005 / EC-006 regression guard — `layout::run` threads
/// `ChartSpec.alt = Some(AltText::Provided("Revenue by region"))` from the
/// semantic block into the `FrameContent::Chart { alt }` frame it produces.
///
/// ## What this test guards
///
/// `layout::run` iterates `slide.blocks` for `ContentBlock::Chart` and replaces the
/// layout placeholder's alt with `spec.alt.unwrap_or(AltText::Decorative)`. This test
/// will FAIL if that threading is removed or regresses, detecting any change that
/// causes the frame to revert to carrying `AltText::Decorative` instead of the
/// author-supplied `AltText::Provided("Revenue by region")`.
///
/// ## Threading contract
///
/// In the slide-processing loop in `layout::run`, for each `ContentBlock::Chart(spec)`,
/// the `FrameContent::Chart` frame's `alt` is replaced with
/// `spec.alt.unwrap_or(AltText::Decorative)` (emitting `tracing::warn!` when
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

    // Regression guard: layout::run must thread ChartSpec.alt into the frame.
    assert_eq!(
        chart_frame_alt[0],
        &AltText::Provided(Arc::clone(&provided_alt)),
        "FrameContent::Chart.alt must be Provided(\"Revenue by region\") — \
         threaded from ChartSpec.alt by layout::run; \
         got: {:?} (regression: alt not threaded through layout::run)",
        chart_frame_alt[0]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 2 — Diagram slide-type: provided alt threads through layout::run
// ─────────────────────────────────────────────────────────────────────────────

/// STORY-039 AC-005 regression guard — `layout::run` threads
/// `DiagramSpec.alt = Some(AltText::Provided("Architecture overview"))` into
/// the `FrameContent::Diagram { alt, .. }` frame it produces.
///
/// ## What this test guards
///
/// `layout::run` iterates `ContentBlock::Diagram` blocks and replaces the layout
/// placeholder's `alt` with `spec.alt.unwrap_or(AltText::Decorative)`. This test
/// will FAIL if that threading is removed or regresses, detecting any change that
/// causes the Diagram frame to revert to carrying `AltText::Decorative` instead of
/// the author-supplied `AltText::Provided("Architecture overview")`.
///
/// ## Threading contract
///
/// In the slide-processing loop in `layout::run`, for each `ContentBlock::Diagram(spec)`,
/// the `FrameContent::Diagram` frame's `alt` is replaced with
/// `spec.alt.unwrap_or(AltText::Decorative)` (emitting `tracing::warn!` when
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

    // Regression guard: layout::run must thread DiagramSpec.alt into the frame.
    assert_eq!(
        diagram_frame_alt[0],
        &AltText::Provided(Arc::clone(&provided_alt)),
        "FrameContent::Diagram.alt must be Provided(\"Architecture overview\") — \
         threaded from DiagramSpec.alt by layout::run; \
         got: {:?} (regression: alt not threaded through layout::run)",
        diagram_frame_alt[0]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 3 — Screenshot slide-type: provided alt threads through layout::run
// ─────────────────────────────────────────────────────────────────────────────

/// STORY-039 AC-005 regression guard — `layout::run` threads
/// `ImageSpec.alt = Some(AltText::Provided("Dashboard screenshot"))` from the
/// semantic block into the `FrameContent::Image { alt }` frame it produces.
///
/// Uses the `"screenshot"` slide type, which maps to a two-frame layout
/// (title + `FrameContent::Image` placeholder). The `"bio"` slide type
/// would also qualify (it has a `FrameContent::Image` frame at index 0).
///
/// ## What this test guards
///
/// `layout::run` iterates `ContentBlock::Image` blocks and replaces the layout
/// placeholder's `alt` with `spec.alt.unwrap_or(AltText::Decorative)`. This test
/// will FAIL if that threading is removed or regresses, detecting any change that
/// causes the Image frame to revert to carrying `AltText::Decorative` instead of
/// the author-supplied `AltText::Provided("Dashboard screenshot")`.
///
/// ## Threading contract
///
/// In the slide-processing loop in `layout::run`, for each `ContentBlock::Image(spec)`,
/// the `FrameContent::Image` frame's `alt` is replaced with
/// `spec.alt.unwrap_or(AltText::Decorative)` (emitting `tracing::warn!` when
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

    // Regression guard: layout::run must thread ImageSpec.alt into the frame.
    assert_eq!(
        image_frame_alts[0],
        &AltText::Provided(Arc::clone(&provided_alt)),
        "FrameContent::Image.alt must be Provided(\"Dashboard screenshot\") — \
         threaded from ImageSpec.alt by layout::run; \
         got: {:?} (regression: alt not threaded through layout::run)",
        image_frame_alts[0]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 4 — EC-006: Chart with alt == None maps to AltText::Unspecified
// ─────────────────────────────────────────────────────────────────────────────

/// STORY-039 EC-006 regression guard (UPDATED by STORY-086 / ADR-019 Decision 5.2) —
/// When `ChartSpec.alt` is `None` (author did not supply alt text), `layout::run` must
/// produce `FrameContent::Chart { alt: AltText::Unspecified }`.
///
/// ## ADR-019 Decision 5.2 update
///
/// Prior to STORY-086, `None → Decorative` was used as a fallback sentinel.
/// ADR-019 Decision 5.2 replaces this with `None → Unspecified` to distinguish:
/// - `Decorative` = author explicitly opted out (decorative: true)
/// - `Unspecified` = no author alt data threaded (pipeline gap, triggers E-A11-001)
///
/// ## What this test guards
///
/// `layout::run` maps `ChartSpec.alt = None` to `AltText::Unspecified` and emits
/// `tracing::warn!` per EC-006 / EC-007. This test ensures the fallback cannot
/// regress to `Decorative` (which would make missing-alt go undetected by the
/// post-layout validator) or to a raw `None` propagation.
///
/// ## Threading contract
///
/// `FrameContent::Chart { alt: AltText::Unspecified }` is produced when
/// `ChartSpec.alt` is `None`. The `tracing::warn!` sentinel is emitted per EC-006.
/// The post-layout validator sees `Unspecified` and emits `E-A11-001` in strict mode.
#[test]
fn test_bc_3_06_039_ec006_chart_alt_none_maps_to_unspecified() {
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

    let result =
        run(&deck, &brand).expect("layout::run must succeed for chart slide with alt: None");

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

    // EC-006 / ADR-019 Decision 5.2: alt: None in ChartSpec must map to
    // AltText::Unspecified in the frame (pipeline gap — no author alt threaded).
    // Regression guard: ensures None maps to Unspecified (not Decorative or raw None).
    // Unspecified triggers E-A11-001 in the post-layout validator (strict mode).
    assert_eq!(
        chart_frame_alt[0],
        &AltText::Unspecified,
        "EC-006 / ADR-019: FrameContent::Chart.alt must be AltText::Unspecified when \
         ChartSpec.alt is None; got: {:?}",
        chart_frame_alt[0]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-095-P1-002 fix: ColorBar alt derived from TextTag::ColorLabel
// ─────────────────────────────────────────────────────────────────────────────

/// F-095-P1-002 / F-095-P1-006 RED GATE: `layout::run` must derive
/// `FrameContent::ColorBar.alt = AltText::Provided(label)` from the
/// `ContentBlock::Text(TextBlock { tag: TextTag::ColorLabel, .. })` block on the
/// same slide.
///
/// ## What this test guards
///
/// Before this fix, `layout::run` hardcoded `alt: AltText::Unspecified` for every
/// `FrameContent::ColorBar`, regardless of whether the slide had a `ColorLabel` block.
/// That caused ALL production `ColorBar` frames to be emitted as `/Artifact` (invisible
/// to assistive technology) in the PDF exporter, violating WCAG SC 1.1.1 for progress
/// bar slides.
///
/// This test builds a `progress_bar` slide with a `TextTag::ColorLabel` block ("75%
/// complete") alongside a `ContentBlock::ColorBar { percent: 75 }`, runs `layout::run`,
/// and asserts that the resulting `FrameContent::ColorBar.alt` carries
/// `AltText::Provided("75% complete")`.
///
/// The test FAILS before the fix (alt is `Unspecified`) and PASSES after the fix
/// (alt is `Provided("75% complete")`).
///
/// ## TD-VSDD-059 compliance
///
/// This is a load-bearing test: removing the `TextTag::ColorLabel` → `alt` derivation
/// logic from `layout.rs` causes this test to fail (reverts to `Unspecified`).
#[test]
fn test_f095_p1_002_color_bar_alt_derived_from_color_label_block() {
    // Build a TextTag::ColorLabel block with "75% complete" plain text.
    let label_text = Arc::from("75% complete");
    let label_block = ContentBlock::Text(TextBlock {
        inlines: vec![InlineNode::Plain(Arc::clone(&label_text))],
        tag: TextTag::ColorLabel,
        span: SourceSpan::default(),
    });
    // ColorBar block at 75%.
    let color_bar_block = ContentBlock::ColorBar(ColorBarSpec { percent: 75 });

    // Build the slide with BOTH blocks (ColorLabel must come before ColorBar to
    // mirror production eval output, but layout must scan all blocks regardless of order).
    let slide = Slide {
        slide_type: Arc::from("progress_bar"),
        fields: OrderedMap::new(),
        blocks: vec![
            Block {
                content: label_block,
                label: None,
                span: SourceSpan::default(),
            },
            Block {
                content: color_bar_block,
                label: None,
                span: SourceSpan::default(),
            },
        ],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
        field_spans: OrderedMap::new(),
    };
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result = run(&deck, &brand)
        .expect("layout::run must succeed for progress_bar with ColorLabel + ColorBar");

    // Extract FrameContent::ColorBar frames from the laid-out slide.
    let color_bar_alts: Vec<&AltText> = result.slides[0]
        .frames
        .iter()
        .filter_map(|f| match &f.content {
            FrameContent::ColorBar { alt, .. } => Some(alt),
            _ => None,
        })
        .collect();

    assert_eq!(
        color_bar_alts.len(),
        1,
        "progress_bar slide must produce exactly 1 FrameContent::ColorBar frame; \
         got {} (total frames: {})",
        color_bar_alts.len(),
        result.slides[0].frames.len()
    );

    // F-095-P1-002 guard: layout::run must derive alt from TextTag::ColorLabel.
    // Before fix: AltText::Unspecified (hardcoded). After fix: AltText::Provided("75% complete").
    assert_eq!(
        color_bar_alts[0],
        &AltText::Provided(Arc::clone(&label_text)),
        "F-095-P1-002: FrameContent::ColorBar.alt must be Provided(\"75% complete\") \
         derived from TextTag::ColorLabel block; got: {:?} \
         (regression: layout.rs still hardcodes AltText::Unspecified for ColorBar)",
        color_bar_alts[0]
    );
}

/// F-095-P1-002 fallback: when NO `TextTag::ColorLabel` block is present on the
/// slide, `FrameContent::ColorBar.alt` must be `AltText::Unspecified` (safe
/// fallback — the PDF exporter emits `/Artifact`, which is the correct PDF/UA-1
/// behavior for an unmarked progress bar).
///
/// This test ensures the fallback branch is exercised and guards against
/// accidentally emitting `AltText::Decorative` (which would be incorrect: the
/// bar IS meaningful content, it just lacks a label from the author).
#[test]
fn test_f095_p1_002_color_bar_alt_unspecified_when_no_color_label_block() {
    // ColorBar block only — no ColorLabel block on the slide.
    let color_bar_block = ContentBlock::ColorBar(ColorBarSpec { percent: 40 });
    let slide = slide_with_block("progress_bar", color_bar_block);
    let deck = make_deck(vec![slide]);
    let brand = make_brand();

    let result =
        run(&deck, &brand).expect("layout::run must succeed for progress_bar with ColorBar only");

    let color_bar_alts: Vec<&AltText> = result.slides[0]
        .frames
        .iter()
        .filter_map(|f| match &f.content {
            FrameContent::ColorBar { alt, .. } => Some(alt),
            _ => None,
        })
        .collect();

    assert_eq!(
        color_bar_alts.len(),
        1,
        "progress_bar slide must produce exactly 1 FrameContent::ColorBar frame; \
         got {} (total frames: {})",
        color_bar_alts.len(),
        result.slides[0].frames.len()
    );

    // When no ColorLabel block, alt must fall back to Unspecified (not Decorative).
    assert_eq!(
        color_bar_alts[0],
        &AltText::Unspecified,
        "F-095-P1-002 fallback: FrameContent::ColorBar.alt must be AltText::Unspecified \
         when no TextTag::ColorLabel block is present; got: {:?}",
        color_bar_alts[0]
    );
}
