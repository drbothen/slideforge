---
document_type: verification-property
vp_id: VP-038
title: "Shape: without alt or decorative always produces LayoutError::MissingAlt"
module: slideforge-layout
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: false
bc_trace: [BC-3.04.001, DI-001]
anchored_to_bc: BC-3.04.001
traces_to: .factory/specs/verification-properties/VP-INDEX.md
spec_version: "1.0.1"
---

# VP-038: Shape Without Alt or Decorative Always Produces LayoutError::MissingAlt

## Property Statement

For any `ShapeSpec` where `alt == None` and `decorative == false`, the layout stage's
shape processing function MUST produce `LayoutError::MissingAlt { slide_index, span }`.
This error MUST carry the `SourceSpan` of the offending `shape:` block.

Formally: `shape.alt == None ∧ shape.decorative == false`
→ `layout_shape(shape) ∈ Errors where errors.any(|e| matches!(e, LayoutError::MissingAlt { .. }))`.

## Motivation

BC-3.04.001 Invariant 1 and Postcondition 6 (multi-error accumulation). DI-001 (alt
text required on visual elements). Every shape without alt is an accessibility violation.
The layout stage acts as a defensive check — even if parse/validation stages miss the
error (e.g., due to a bug), layout must catch it. The error must carry a span so users
see which shape:block is the offender.

## Feasibility Assessment

Not Kani-amenable: the layout stage processes `ShapeSpec` values that are constructed
through parsing. The property is verified via unit test constructing a `ShapeSpec`
with `alt: None, decorative: false` directly and passing it to the layout function.

`kani_amenable: false` — layout involves slide IR state, not pure arithmetic.

## Test Harness

```rust
// crates/slideforge-layout/src/tests/shape_alt.rs
#[test]
fn test_shape_without_alt_produces_missing_alt_error() {
    let shape = ShapeSpec {
        shape_type: ShapeType::Rect,
        position: ShapePosition { x: ShapeUnit::Inches(500), y: ShapeUnit::Inches(1000),
                                   width: ShapeUnit::Inches(2000), height: ShapeUnit::Inches(1000) },
        fill: FillSpec::None,
        text: None,
        alt: None,           // Missing!
        decorative: false,   // Not decorative!
        span: SourceSpan::default(),
    };
    let errors = layout_shape(0, &shape, &BrandConfig::default());
    assert!(
        errors.iter().any(|e| matches!(e, LayoutError::MissingAlt { slide_index: 0, .. })),
        "must produce MissingAlt error"
    );
}

#[test]
fn test_shape_with_decorative_true_does_not_produce_missing_alt() {
    let shape = ShapeSpec {
        alt: None,
        decorative: true,   // Decorative — no alt required
        ..default_shape()
    };
    let errors = layout_shape(0, &shape, &BrandConfig::default());
    assert!(
        !errors.iter().any(|e| matches!(e, LayoutError::MissingAlt { .. })),
        "decorative shape must not produce MissingAlt"
    );
}

#[test]
fn test_missing_alt_carries_span() {
    let expected_span = SourceSpan::new(42, 100);
    let shape = ShapeSpec { alt: None, decorative: false, span: expected_span, ..default_shape() };
    let errors = layout_shape(0, &shape, &BrandConfig::default());
    let err = errors.iter().find(|e| matches!(e, LayoutError::MissingAlt { .. }))
        .expect("must produce MissingAlt");
    if let LayoutError::MissingAlt { span, .. } = err {
        assert_eq!(*span, expected_span);
    }
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `ShapeSpec { alt: None, decorative: false }` | `LayoutError::MissingAlt` with slide index and span | error |
| `ShapeSpec { alt: None, decorative: true }` | No `MissingAlt` error | edge-case |
| `ShapeSpec { alt: Some(AltText::Provided(Arc::from("desc"))), decorative: false }` | No `MissingAlt` error | happy-path |

## Changelog

| Version | Date | Change |
|---------|------|--------|
| v1.0.1 | 2026-05-29 | pass-6 drift fix (F-P6-MED-001): AltText::Text → AltText::Provided to match canonical specs.rs variants |
| v1.0.0 | — | Initial version |
