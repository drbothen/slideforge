---
document_type: verification-property
vp_id: VP-049
title: "LaidOutDeck.warnings is populated with XrefTargetNotFound and OffCanvas warnings from layout::run"
module: slideforge-layout
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: false
bc_trace: [BC-3.04.001, BC-3.05.001]
anchored_to_bc: BC-3.04.001
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-049: LaidOutDeck.warnings Populated — Warnings Not Dropped by layout::run

## Property Statement

The layout runner (`layout::run`) MUST accumulate ALL `LayoutWarning` values produced
by its sub-passes into `LaidOutDeck.warnings: Vec<LayoutWarning>` and return them to
the caller. Specifically:

1. `LayoutWarning::XrefTargetNotFound` events produced by `run_inline_validation` MUST
   appear in `LaidOutDeck.warnings`.
2. `LayoutWarning::OffCanvas` events produced by `layout_shapes` MUST appear in
   `LaidOutDeck.warnings`.
3. The `LaidOutDeck.warnings` field MUST exist on the struct and MUST be populated by
   `layout::run` — it MUST NOT be silently omitted or dropped between sub-passes.

A `layout::run` implementation that discards warnings without propagating them to
`LaidOutDeck.warnings` violates BC-3.04.001 Postcondition 7 and BC-3.05.001 Invariant 7.

## Motivation

BC-3.04.001 Postcondition 7 (Item O), BC-3.05.001 Invariant 7. `LayoutWarning` values
represent non-fatal issues (off-canvas shapes, broken cross-references) that the caller
must be able to inspect. Exporters and CLI tools decide whether to surface or suppress
individual warning types. If `layout::run` drops warnings internally, exporters have no
opportunity to surface them, and users cannot diagnose layout issues (e.g., a shape that
renders partially off-slide, or a cross-reference link that silently goes nowhere).

CLAUDE.md Rule 4: "AI-built defects are the AI's responsibility to fix." Silent warning
drops discovered in review MUST be fixed in scope, not deferred.

## Feasibility Assessment

Not Kani-amenable: the property involves stateful accumulation across multiple sub-passes
of `layout::run` (inline validation pass + shape layout pass). The logic iterates over
slides and shapes, collecting warnings from two different sources. This pipeline behavior
is verified via unit tests that:

1. Inject a slide with a missing xref target.
2. Inject a slide with an off-canvas shape.
3. Assert that `LaidOutDeck.warnings` contains both `XrefTargetNotFound` and `OffCanvas`
   entries.

`kani_amenable: false` — multi-pass layout pipeline, not pure arithmetic.

## Test Harness

```rust
// crates/slideforge-layout/src/tests/warnings_populated.rs

#[test]
fn test_xref_target_not_found_reaches_laidoutdeck() {
    // BC-3.04.001 EC-016, BC-3.05.001 EC-002 canonical test vector
    let slide = SlideIr {
        shapes: vec![ShapeSpec {
            text: Some(vec![InlineNode::Xref(Arc::from("missing-slide"))]),
            alt: Some(AltText::Provided(Arc::from("box"))),
            ..default_shape()
        }],
        ..default_slide()
    };
    let deck = Deck { slides: vec![slide], ..default_deck() };
    let laid_out = layout::run(&deck, &BrandConfig::default()).expect("layout should succeed");

    let xref_warning = laid_out.warnings.iter().find(|w| {
        matches!(w, LayoutWarning::XrefTargetNotFound { target, .. } if target == "missing-slide")
    });
    assert!(
        xref_warning.is_some(),
        "XrefTargetNotFound warning must be in LaidOutDeck.warnings, not dropped"
    );
}

#[test]
fn test_off_canvas_warning_reaches_laidoutdeck() {
    // BC-3.04.001 EC-017 canonical test vector: shape at x=11in on a 10in canvas
    let slide = SlideIr {
        shapes: vec![ShapeSpec {
            position: ShapePosition {
                x: ShapeUnit::Inches(11_000), // 11in
                y: ShapeUnit::Inches(0),
                width: ShapeUnit::Inches(1_000),
                height: ShapeUnit::Inches(1_000),
            },
            alt: Some(AltText::Provided(Arc::from("off-canvas rect"))),
            ..default_shape()
        }],
        ..default_slide()
    };
    let deck = Deck { slides: vec![slide], ..default_deck() };
    let laid_out = layout::run(&deck, &BrandConfig::default()).expect("layout should succeed");

    let off_canvas_warning = laid_out.warnings.iter().find(|w| {
        matches!(w, LayoutWarning::OffCanvas { .. })
    });
    assert!(
        off_canvas_warning.is_some(),
        "OffCanvas warning must be in LaidOutDeck.warnings, not dropped"
    );
}

#[test]
fn test_clean_deck_has_empty_warnings() {
    // A deck with no issues must produce an empty warnings vec (not None or uninitialized)
    let deck = Deck { slides: vec![default_slide_with_alt()], ..default_deck() };
    let laid_out = layout::run(&deck, &BrandConfig::default()).expect("layout should succeed");
    assert!(
        laid_out.warnings.is_empty(),
        "LaidOutDeck.warnings must be an empty Vec (not absent) when no warnings occur"
    );
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Slide with `Xref("missing")` in shape text field | `LaidOutDeck.warnings` contains `LayoutWarning::XrefTargetNotFound { target: "missing", source_slide_index: 0 }` | warning propagation (EC-016) |
| Shape at `x=11in` on a 10in canvas | `LaidOutDeck.warnings` contains `LayoutWarning::OffCanvas { source_slide_index: 0, .. }` | warning propagation (EC-017) |
| Clean deck with valid shapes | `LaidOutDeck.warnings` is `vec![]` (empty, not absent) | happy-path |
| Slide with both a missing xref and off-canvas shape | `LaidOutDeck.warnings` contains both `XrefTargetNotFound` and `OffCanvas` entries | combined |
