---
document_type: verification-property
vp_id: VP-046
title: "Inline: Xref inside MathNode is NOT flagged by xref validation pass"
module: slideforge-layout
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: false
bc_trace: [BC-3.05.001]
anchored_to_bc: BC-3.05.001
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-046: Xref Inside MathNode Is NOT Flagged by Xref Validation Pass

## Property Statement

The layout-stage xref validation pass MUST NOT traverse into `InlineNode::Math(MathNode)`
content when checking for unresolved xref targets. An `Xref` expression embedded within
a `MathNode` (if representable at the IR level) MUST NOT produce an
`LayoutWarning::XrefTargetNotFound` warning.

Formally: the xref validation function traverses ONLY top-level `InlineNode` sequences
and does NOT recurse into `MathNode` content. Any xref inside a `MathNode` is invisible
to the top-level xref validator.

## Motivation

BC-3.05.001 Invariant 5 and EC-007. This is an explicit v1.0 scope boundary: math
rendering (BC-1.10.003) is a separate validation surface. The layout xref pass must not
require LaTeX AST parsing. The boundary is intentional and documented — future maintainers
should not add math-xref validation without a dedicated story anchored to BC-1.10.003.

This property validates a DELIBERATE NON-BEHAVIOR, not a missing feature. The test
verifies that the validator correctly stops at the `Math` variant boundary.

## Feasibility Assessment

Not Kani-amenable: xref validation involves slide state (slide titles for xref resolution).
The non-traversal property is verified by constructing a `MathNode` containing xref-like
content and verifying that no `XrefTargetNotFound` warning is produced.

`kani_amenable: false` — slide-state-dependent validation.

## Test Harness

```rust
// crates/slideforge-layout/src/tests/inline_xref.rs
#[test]
fn test_xref_inside_math_is_not_validated() {
    // Create a slide with a Math node; the MathNode contains a fake xref-like identifier
    // The top-level xref validator must not descend into MathNode
    let inline = InlineNode::Math(MathNode {
        source: Arc::from(r"\xref{nonexistent-slide}"), // LaTeX — not traversed
    });
    let slide_titles: std::collections::HashSet<Arc<str>> = Default::default();
    let warnings = validate_xrefs(0, &[inline], &slide_titles);
    assert!(
        warnings.iter().all(|w| !matches!(w, LayoutWarning::XrefTargetNotFound { .. })),
        "xref inside MathNode must not produce XrefTargetNotFound: {:?}", warnings
    );
}

#[test]
fn test_top_level_xref_to_nonexistent_slide_does_produce_warning() {
    // Contrast: top-level Xref IS validated
    let inline = InlineNode::Xref(Arc::from("nonexistent-slide"));
    let slide_titles: std::collections::HashSet<Arc<str>> = Default::default();
    let warnings = validate_xrefs(0, &[inline], &slide_titles);
    assert!(
        warnings.iter().any(|w| matches!(w, LayoutWarning::XrefTargetNotFound { .. })),
        "top-level Xref to missing slide must warn: {:?}", warnings
    );
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `InlineNode::Math(MathNode { source: "\\xref{missing}" })` | No `XrefTargetNotFound` warning | boundary (EC-007) |
| `InlineNode::Xref(Arc::from("missing-slide"))` (top-level) | `LayoutWarning::XrefTargetNotFound` warning | contrast |
