---
document_type: verification-property
vp_id: VP-042
title: "Shape: slide with N shapes all missing alt returns Vec with N MissingAlt errors"
module: slideforge-layout
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: false
bc_trace: [BC-3.04.001, DI-018]
anchored_to_bc: BC-3.04.001
traces_to: .factory/specs/verification-properties/VP-INDEX.md
spec_version: "1.0.1"
---

# VP-042: Shape Multi-Error Accumulation — N Missing Alts Produce N Errors

## Property Statement

When a slide contains N shapes each with `alt == None` and `decorative == false`, the
layout function MUST return a `Vec<LayoutError>` containing exactly N `MissingAlt`
entries — one per shape, in source order. The function MUST NOT bail on the first
missing-alt shape.

Formally: `slide.shapes.all(|s| s.alt == None ∧ s.decorative == false) ∧ slide.shapes.len() == N`
→ `layout_slide(slide).errors.count(|e| matches!(e, MissingAlt { .. })) == N`.

## Motivation

BC-3.04.001 Postcondition 6 and Invariant (multi-error, per DI-018). DI-018 requires
error accumulation across a slide's shape set before returning. If the layout function
bails on the first missing-alt error, the user only sees one error per build invocation
and must fix-and-rebuild N times for N missing-alt shapes — poor UX. Accumulation lets
the user fix all accessibility errors in one pass.

## Feasibility Assessment

Not Kani-amenable: the accumulation logic involves iterating over a `Vec<ShapeSpec>`
in layout IR state. The property is verified via unit test constructing a slide with
2+ shapes all lacking alt text.

`kani_amenable: false` — layout pipeline, not pure arithmetic.

## Test Harness

```rust
// crates/slideforge-layout/src/tests/shape_multi_error.rs
#[test]
fn test_two_shapes_missing_alt_produces_two_errors() {
    let shapes = vec![
        ShapeSpec { alt: None, decorative: false, span: span(10, 20), ..default_shape() },
        ShapeSpec { alt: None, decorative: false, span: span(30, 40), ..default_shape() },
    ];
    let slide = SlideIr { shapes, ..default_slide() };
    let errors = layout_slide(0, &slide, &BrandConfig::default());
    let missing_alt_count = errors.iter()
        .filter(|e| matches!(e, LayoutError::MissingAlt { .. }))
        .count();
    assert_eq!(missing_alt_count, 2, "must accumulate both MissingAlt errors");
}

#[test]
fn test_three_shapes_one_with_alt_produces_two_errors() {
    let shapes = vec![
        ShapeSpec { alt: None, decorative: false, ..default_shape() },
        ShapeSpec { alt: Some(AltText::Provided(Arc::from("desc"))), ..default_shape() },
        ShapeSpec { alt: None, decorative: false, ..default_shape() },
    ];
    let slide = SlideIr { shapes, ..default_slide() };
    let errors = layout_slide(0, &slide, &BrandConfig::default());
    let missing_alt_count = errors.iter()
        .filter(|e| matches!(e, LayoutError::MissingAlt { .. }))
        .count();
    assert_eq!(missing_alt_count, 2, "only the 2 shapes without alt produce errors");
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Slide with 2 shapes, both `alt: None` | `Vec` with 2 `MissingAlt` entries | multi-error (EC-010) |
| Slide with 1 shape `alt: None` | `Vec` with 1 `MissingAlt` entry | single-error |
| Slide with 2 shapes, 1 has alt | `Vec` with 1 `MissingAlt` entry | mixed |

## Changelog

| Version | Date | Change |
|---------|------|--------|
| v1.0.1 | 2026-05-29 | pass-6 drift fix (F-P6-MED-001): AltText::Text → AltText::Provided to match canonical specs.rs variants |
| v1.0.0 | — | Initial version |
