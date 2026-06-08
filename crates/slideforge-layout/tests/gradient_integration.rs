//! STORY-072: shape: Gradient Fills — layout integration tests (Red Gate).
//!
//! Integration tests for AC-003 (layout passthrough) for STORY-072.
//!
//! These tests verify that `layout::run()` passes `FillSpec::Gradient` through
//! to `ShapeFrame.fill` in `LaidOutDeck` without modification.
//!
//! ## Red Gate status
//!
//! All tests in this file exercise `FillSpec::Gradient` which is now present
//! in the type (added as the first STORY-072 task). The layout passthrough
//! is variant-agnostic (passes `fill` verbatim), so these tests PASS after the
//! type variant is added.
//!
//! The load-bearing Red Gate tests are in the exporter crates (PPTX, HTML, DOCX).
//! These integration tests verify the layout→exporter contract is upheld.
//!
//! ## Traceability
//!
//! | Test | AC | BC-3.04.001 clause |
//! |---|---|---|
//! | `test_BC_3_04_001_ac003_gradient_passthrough_fill_preserved` | AC-003 | postcondition 3 |
//! | `test_BC_3_04_001_ac003_gradient_passthrough_from_to_colors_verbatim` | AC-003 | postcondition 3 |
//! | `test_BC_3_04_001_ac003_gradient_passthrough_alt_text_preserved` | AC-003 + AC-005 | invariant 1 |
//! | `test_BC_3_04_001_ac003_gradient_with_no_alt_returns_missing_alt_error` | AC-005 | EC-001 |
//! | `test_BC_3_04_001_ec006_gradient_decorative_shape_passthrough` | EC-006 | STORY-072 EC-006 |

#![allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    non_snake_case
)]

use std::sync::Arc;

use slideforge_layout::{
    FillSpec, FrameContent, PageSize,
    shapes::{DEFAULT_EM_IN_EMU, layout_shapes},
};
use slideforge_types::{AltText, Rgb, ShapePosition, ShapeSpec, ShapeUnit, SourceSpan};

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn default_page() -> PageSize {
    PageSize {
        width: slideforge_layout::types::DEFAULT_PAGE_WIDTH,
        height: slideforge_layout::types::DEFAULT_PAGE_HEIGHT,
    }
}

fn default_position() -> ShapePosition {
    ShapePosition {
        x: ShapeUnit::Inches(500),        // 0.5in
        y: ShapeUnit::Inches(1_000),      // 1.0in
        width: ShapeUnit::Inches(2_000),  // 2.0in
        height: ShapeUnit::Inches(1_000), // 1.0in
    }
}

fn gradient_shape_spec(from: Rgb, to: Rgb, alt: &str) -> ShapeSpec {
    let st =
        slideforge_types::ShapeType::from_keyword("rect").expect("rect must be a valid shape type");
    ShapeSpec {
        shape_type: st,
        position: default_position(),
        fill: FillSpec::Gradient { from, to },
        text: None,
        alt: Some(AltText::Provided(Arc::from(alt))),
        decorative: false,
        span: SourceSpan::default(),
    }
}

fn gradient_shape_spec_decorative(from: Rgb, to: Rgb) -> ShapeSpec {
    let st =
        slideforge_types::ShapeType::from_keyword("rect").expect("rect must be a valid shape type");
    ShapeSpec {
        shape_type: st,
        position: default_position(),
        fill: FillSpec::Gradient { from, to },
        text: None,
        alt: None,
        decorative: true,
        span: SourceSpan::default(),
    }
}

fn gradient_shape_spec_no_alt(from: Rgb, to: Rgb) -> ShapeSpec {
    let st =
        slideforge_types::ShapeType::from_keyword("rect").expect("rect must be a valid shape type");
    ShapeSpec {
        shape_type: st,
        position: default_position(),
        fill: FillSpec::Gradient { from, to },
        text: None,
        alt: None,
        decorative: false,
        span: SourceSpan::default(),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

/// AC-003 (STORY-072) — `layout_shapes` passes `FillSpec::Gradient` through to
/// `ShapeFrame.fill` in the output frame verbatim.
///
/// The layout stage MUST NOT transform or drop the gradient fill — it is a pure
/// passthrough (BC-3.04.001 postcondition 3).
///
/// This test PASSES after the `FillSpec::Gradient` variant is added to the enum,
/// since `layout_shapes` already stores `fill` verbatim via `build_shape_frame`.
#[test]
fn test_BC_3_04_001_ac003_gradient_passthrough_fill_preserved() {
    let from = Rgb { r: 255, g: 0, b: 0 };
    let to = Rgb { r: 0, g: 0, b: 255 };
    let shapes = vec![gradient_shape_spec(from, to, "Gradient background")];
    let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0)
        .expect("layout_shapes must succeed for gradient shape with alt text");

    assert_eq!(output.frames.len(), 1, "one shape → one frame");
    match &output.frames[0].content {
        FrameContent::Shape(sf) => {
            assert!(
                matches!(&sf.fill, FillSpec::Gradient { .. }),
                "ShapeFrame.fill must be FillSpec::Gradient after layout passthrough; got: {:?}",
                sf.fill
            );
        },
        other => panic!("expected FrameContent::Shape, got: {other:?}"),
    }
}

/// AC-003 (STORY-072) — `layout_shapes` preserves the exact `from` and `to` Rgb values
/// of the gradient fill verbatim (no normalization or mutation).
///
/// Canonical test vector from BC-3.04.001 / STORY-072 AC-001:
///   `fill gradient #FF0000 to #0000FF` →
///   `FillSpec::Gradient { from: Rgb { r: 255, g: 0, b: 0 }, to: Rgb { r: 0, g: 0, b: 255 } }`
#[test]
fn test_BC_3_04_001_ac003_gradient_passthrough_from_to_colors_verbatim() {
    let from = Rgb { r: 255, g: 0, b: 0 }; // #FF0000
    let to = Rgb { r: 0, g: 0, b: 255 }; // #0000FF
    let shapes = vec![gradient_shape_spec(from, to, "Red-to-blue gradient")];
    let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0)
        .expect("layout_shapes must succeed");

    match &output.frames[0].content {
        FrameContent::Shape(sf) => match &sf.fill {
            FillSpec::Gradient {
                from: out_from,
                to: out_to,
            } => {
                assert_eq!(
                    *out_from, from,
                    "FillSpec::Gradient.from must be preserved verbatim; \
                     expected Rgb {{r:255, g:0, b:0}}, got: {out_from:?}"
                );
                assert_eq!(
                    *out_to, to,
                    "FillSpec::Gradient.to must be preserved verbatim; \
                     expected Rgb {{r:0, g:0, b:255}}, got: {out_to:?}"
                );
            },
            other => panic!("expected FillSpec::Gradient, got: {other:?}"),
        },
        other => panic!("expected FrameContent::Shape, got: {other:?}"),
    }
}

/// AC-003 + AC-005 (STORY-072) — Alt text for a gradient shape is preserved in
/// the output `ShapeFrame.alt` field.
///
/// The gradient fill variant does not affect the alt-text resolution contract
/// (BC-3.04.001 invariant 1).
#[test]
fn test_BC_3_04_001_ac003_gradient_passthrough_alt_text_preserved() {
    let alt_text = "Gradient background, red top to blue bottom";
    let shapes = vec![gradient_shape_spec(
        Rgb { r: 255, g: 0, b: 0 },
        Rgb { r: 0, g: 0, b: 255 },
        alt_text,
    )];
    let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0)
        .expect("layout_shapes must succeed");

    match &output.frames[0].content {
        FrameContent::Shape(sf) => match &sf.alt {
            AltText::Provided(s) => {
                assert_eq!(
                    s.as_ref(),
                    alt_text,
                    "alt text must be preserved verbatim for gradient shapes"
                );
            },
            other => panic!("expected AltText::Provided for gradient shape, got: {other:?}"),
        },
        other => panic!("expected FrameContent::Shape, got: {other:?}"),
    }
}

/// AC-005 (STORY-072) / EC-001 — A gradient shape without `alt` or `decorative: true`
/// produces `LayoutError::MissingAlt` (same as solid shapes).
///
/// The `FillSpec` variant does NOT exempt a shape from the alt requirement.
/// (BC-3.04.001 precondition 3 / invariant 1 / EC-001).
#[test]
fn test_BC_3_04_001_ac003_gradient_with_no_alt_returns_missing_alt_error() {
    use slideforge_layout::LayoutError;

    let shapes = vec![gradient_shape_spec_no_alt(
        Rgb { r: 255, g: 0, b: 0 },
        Rgb { r: 0, g: 0, b: 255 },
    )];
    let result = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0);
    assert!(
        result.is_err(),
        "gradient shape without alt or decorative must produce an error"
    );
    match result.unwrap_err() {
        LayoutError::Multiple { inner } => {
            assert!(
                inner
                    .iter()
                    .any(|e| matches!(e, LayoutError::MissingAlt { .. })),
                "error must contain MissingAlt for gradient shape without alt; got: {inner:?}"
            );
        },
        other => panic!("expected LayoutError::Multiple containing MissingAlt, got: {other:?}"),
    }
}

/// EC-006 (STORY-072) — `decorative: true` gradient shape produces
/// `AltText::Decorative` in the output frame.
///
/// Gradient fill does not affect the decorative semantics contract.
#[test]
fn test_BC_3_04_001_ec006_gradient_decorative_shape_passthrough() {
    let shapes = vec![gradient_shape_spec_decorative(
        Rgb { r: 0, g: 255, b: 0 },
        Rgb { r: 0, g: 0, b: 255 },
    )];
    let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0)
        .expect("decorative gradient shape must succeed");

    match &output.frames[0].content {
        FrameContent::Shape(sf) => {
            assert!(
                matches!(sf.alt, AltText::Decorative),
                "decorative gradient shape must produce AltText::Decorative; got: {:?}",
                sf.alt
            );
            // The fill must still be the gradient (not downgraded at layout time).
            assert!(
                matches!(sf.fill, FillSpec::Gradient { .. }),
                "decorative gradient shape must preserve FillSpec::Gradient in output; got: {:?}",
                sf.fill
            );
        },
        other => panic!("expected FrameContent::Shape, got: {other:?}"),
    }
}

/// EC-005 (STORY-072) — Same `from` and `to` color (flat gradient) is valid.
///
/// `layout_shapes` must not error for a gradient where `from == to`.
#[test]
fn test_BC_3_04_001_ec005_gradient_layout_same_from_to_valid() {
    let same = Rgb { r: 255, g: 0, b: 0 };
    let shapes = vec![gradient_shape_spec(same, same, "Flat gradient")];
    let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0)
        .expect("flat gradient (same from==to) must succeed");

    match &output.frames[0].content {
        FrameContent::Shape(sf) => {
            assert!(
                matches!(
                    &sf.fill,
                    FillSpec::Gradient { from, to } if from == to && *from == same
                ),
                "flat gradient must be preserved as FillSpec::Gradient; got: {:?}",
                sf.fill
            );
        },
        other => panic!("expected FrameContent::Shape, got: {other:?}"),
    }
}

/// EC-004 (STORY-072) — Slide with two shapes, one gradient and one missing alt,
/// accumulates both errors (DI-018 / BC-3.04.001 postcondition 6).
///
/// The gradient shape has correct alt text; the second shape has neither alt
/// nor decorative. Both errors must be returned together.
#[test]
fn test_BC_3_04_001_ec004_gradient_multi_shape_errors_accumulated() {
    use slideforge_layout::LayoutError;

    let st = slideforge_types::ShapeType::from_keyword("rect").expect("rect must be valid");
    let gradient_ok = gradient_shape_spec(
        Rgb { r: 255, g: 0, b: 0 },
        Rgb { r: 0, g: 0, b: 255 },
        "Good gradient",
    );
    let solid_no_alt = ShapeSpec {
        shape_type: st,
        position: default_position(),
        fill: FillSpec::SolidColor(Rgb { r: 0, g: 0, b: 255 }),
        text: None,
        alt: None,
        decorative: false, // missing alt — error
        span: SourceSpan::default(),
    };
    let shapes = vec![gradient_ok, solid_no_alt];
    let result = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0);
    assert!(result.is_err(), "must fail: second shape is missing alt");
    match result.unwrap_err() {
        LayoutError::Multiple { inner } => {
            assert!(
                inner
                    .iter()
                    .any(|e| matches!(e, LayoutError::MissingAlt { .. })),
                "must contain MissingAlt for the no-alt solid shape; got: {inner:?}"
            );
        },
        other => panic!("expected LayoutError::Multiple, got: {other:?}"),
    }
}
