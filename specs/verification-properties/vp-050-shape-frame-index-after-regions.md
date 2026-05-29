---
document_type: verification-property
vp_id: VP-050
title: "Shape frames in LaidOutDeck.frames appear at index >= region_count (after all placeholder frames)"
module: slideforge-layout
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: false
bc_trace: [BC-3.04.001]
anchored_to_bc: BC-3.04.001
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-050: Shape Frame Index >= region_count (Shape Frames Appended After Placeholders)

## Property Statement

In a `LaidOutDeck`, all `FrameContent::Shape` frames for a given slide MUST appear
at indices >= `region_count` for that slide. No shape frame may interleave with or
precede a placeholder frame.

Formally, for any slide `s` in `LaidOutDeck`:
- `s.frames[0..s.region_count]` contains ONLY placeholder frames (`FrameContent::Placeholder`)
- `s.frames[s.region_count..]` contains ALL shape frames (`FrameContent::Shape`)
- The first shape frame's 0-based index in `s.frames` MUST be >= `s.region_count`

Canonical test vector (BC-3.04.001 EC-013):
- `title_content` slide: `region_count == 1`, 1 shape block
- Result: `frames[0]` = title placeholder, `frames[1]` = shape frame
- `shape_frame_index (1) >= region_count (1)` — PASS

## Motivation

BC-3.04.001 Postcondition 3 and EC-013. PPTX placeholder inheritance relies on
placeholder frames occupying the first `region_count` slots in the frames array.
If a custom shape frame were inserted before a placeholder, the PPTX exporter
would misalign the placeholder-to-layout mapping, producing broken slides where
placeholder content appears in the wrong visual position or is dropped entirely.
The ordering invariant makes the frame array structure predictable for all downstream
exporters.

## Feasibility Assessment

Not Kani-amenable: the ordering property involves the full layout pass over a slide's
`Vec<ShapeSpec>` and the interplay between placeholder region assignment and shape frame
appending. The logic depends on `Vec` ordering during iteration, which is not amenable
to Kani symbolic reasoning without unrolling the full layout pass. Verified via unit test.

`kani_amenable: false` — frame ordering is a structural property of the layout pipeline,
not a pure arithmetic invariant.

## Test Harness

```rust
// crates/slideforge-layout/src/tests/frame_order.rs

#[test]
fn test_title_content_slide_shape_frame_after_placeholder() {
    // BC-3.04.001 EC-013 canonical test vector
    // title_content slide: 1 placeholder region + 1 shape block
    let slide = SlideIr {
        slide_type: SlideType::TitleContent,
        region_count: 1,
        shapes: vec![
            ShapeSpec {
                position: ShapePosition {
                    x: ShapeUnit::Inches(1_000),
                    y: ShapeUnit::Inches(1_000),
                    width: ShapeUnit::Inches(2_000),
                    height: ShapeUnit::Inches(1_000),
                },
                alt: Some(AltText::Described(Arc::from("blue rect"))),
                fill: FillSpec::SolidColor(Rgb { r: 0, g: 55, b: 102 }),
                ..default_shape()
            },
        ],
        ..default_slide()
    };
    let deck = Deck { slides: vec![slide], ..default_deck() };
    let laid_out = layout::run(&deck, &BrandConfig::default()).expect("layout should succeed");

    let laid_slide = &laid_out.slides[0];
    // frames[0] must be a placeholder frame
    assert!(
        matches!(laid_slide.frames[0].content, FrameContent::Placeholder { .. }),
        "frames[0] must be a placeholder frame (region_count=1)"
    );
    // frames[1] must be the shape frame
    assert!(
        matches!(laid_slide.frames[1].content, FrameContent::Shape(_)),
        "frames[1] must be the shape frame (appended after placeholder)"
    );
    // shape_frame_index (1) >= region_count (1)
    let first_shape_idx = laid_slide.frames.iter().position(|f| {
        matches!(f.content, FrameContent::Shape(_))
    }).expect("must have at least one shape frame");
    assert!(
        first_shape_idx >= laid_slide.region_count,
        "shape frame index ({first_shape_idx}) must be >= region_count ({})",
        laid_slide.region_count
    );
}

#[test]
fn test_two_regions_two_shapes_ordering() {
    // title_body_content slide (2 regions) + 2 shape blocks
    // frames[0], frames[1] = placeholders; frames[2], frames[3] = shapes
    let slide = SlideIr {
        slide_type: SlideType::TitleBodyContent,
        region_count: 2,
        shapes: vec![
            ShapeSpec { alt: Some(AltText::Described(Arc::from("s1"))), ..default_shape() },
            ShapeSpec { alt: Some(AltText::Described(Arc::from("s2"))), ..default_shape() },
        ],
        ..default_slide()
    };
    let deck = Deck { slides: vec![slide], ..default_deck() };
    let laid_out = layout::run(&deck, &BrandConfig::default()).expect("layout should succeed");
    let laid_slide = &laid_out.slides[0];

    for i in 0..2 {
        assert!(matches!(laid_slide.frames[i].content, FrameContent::Placeholder { .. }));
    }
    for i in 2..4 {
        assert!(matches!(laid_slide.frames[i].content, FrameContent::Shape(_)));
    }
}

#[test]
fn test_zero_shapes_only_placeholder_frames() {
    // A slide with no shape blocks must have only placeholder frames
    let slide = SlideIr {
        region_count: 1,
        shapes: vec![],
        ..default_slide()
    };
    let deck = Deck { slides: vec![slide], ..default_deck() };
    let laid_out = layout::run(&deck, &BrandConfig::default()).expect("layout should succeed");
    let laid_slide = &laid_out.slides[0];
    assert_eq!(laid_slide.frames.len(), 1);
    assert!(matches!(laid_slide.frames[0].content, FrameContent::Placeholder { .. }));
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `title_content` slide (`region_count=1`) + 1 shape block | `frames[0]`=placeholder, `frames[1]`=shape; first shape index (1) >= region_count (1) | canonical (EC-013) |
| `title_body_content` slide (`region_count=2`) + 2 shape blocks | `frames[0..2]`=placeholders, `frames[2..4]`=shapes; first shape index (2) >= region_count (2) | ordering |
| Slide with `region_count=1`, 0 shape blocks | `frames` has 1 entry, all placeholders; no shape frames | edge-case |
| Slide with `region_count=0`, 1 shape block | `frames[0]`=shape; shape index (0) >= region_count (0) | zero-regions |
