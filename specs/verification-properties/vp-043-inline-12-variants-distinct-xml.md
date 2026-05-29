---
document_type: verification-property
vp_id: VP-043
title: "Inline: all 12 variant types produce distinct non-empty XML in PPTX output"
module: slideforge-pptx
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: false
bc_trace: [BC-3.05.001, DI-004]
anchored_to_bc: BC-3.05.001
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-043: All 12 Inline Variant Types Produce Distinct Non-Empty XML in PPTX Output

## Property Statement

For each of the 12 `InlineNode` variants (Plain, Bold, Italic, Code, Link, Math,
Footnote, Xref, Superscript, Subscript, Strikethrough, Highlight), rendering to PPTX
XML via the PPTX inline renderer MUST produce non-empty XML output that is distinct per
variant. No two variants may produce identical XML for the same text content.

Formally: for all `v₁ ≠ v₂ ∈ InlineNode variants`,
`render_inline_pptx(v₁) ≠ render_inline_pptx(v₂)` (for comparable text content).

## Motivation

BC-3.05.001 Invariants 1-2 and the per-format postconditions. All 12 types ship in v1.0
with no deferral. If any variant produces the same XML as another, the rendering logic
is broken (e.g., Bold and Plain both producing `<a:t>text</a:t>` with no `b="1"`).
Snapshot tests per inline type serve as both correctness validation and regression detection.

## Feasibility Assessment

Not Kani-amenable: PPTX rendering involves XML serialization, not pure arithmetic.
Verified via snapshot tests (one per inline type) using `insta` or equivalent snapshot
testing library.

`kani_amenable: false` — XML serialization; tested via snapshot tests.

## Test Harness

```rust
// crates/slideforge-pptx/src/tests/inline_rendering.rs
#[test]
fn test_bold_produces_b1_attribute() {
    let node = InlineNode::Bold(vec![InlineNode::Plain(Arc::from("Key finding"))]);
    let xml = render_inline_pptx(&node);
    assert!(xml.contains(r#"b="1""#), "Bold must produce b=\"1\": {xml}");
}

#[test]
fn test_italic_produces_i1_attribute() {
    let node = InlineNode::Italic(vec![InlineNode::Plain(Arc::from("text"))]);
    let xml = render_inline_pptx(&node);
    assert!(xml.contains(r#"i="1""#), "Italic must produce i=\"1\": {xml}");
}

#[test]
fn test_superscript_produces_baseline_30000() {
    let node = InlineNode::Superscript(vec![InlineNode::Plain(Arc::from("2"))]);
    let xml = render_inline_pptx(&node);
    assert!(xml.contains("baseline=\"30000\""), "Superscript must set baseline 30000: {xml}");
}

#[test]
fn test_subscript_produces_baseline_neg25000() {
    let node = InlineNode::Subscript(vec![InlineNode::Plain(Arc::from("2"))]);
    let xml = render_inline_pptx(&node);
    assert!(xml.contains("baseline=\"-25000\""), "Subscript must set baseline -25000: {xml}");
}

#[test]
fn test_strikethrough_produces_sngstrike() {
    let node = InlineNode::Strikethrough(vec![InlineNode::Plain(Arc::from("text"))]);
    let xml = render_inline_pptx(&node);
    assert!(xml.contains("strike=\"sngStrike\""), "Strikethrough must set sngStrike: {xml}");
}

// Additional snapshot tests for: Code (monospace font), Link (hlinkClick), Xref (internal link),
// Highlight (highlight attr), Footnote (note annotation), Math (OMML oMath block)
```

## Test Vectors

| Input | Expected XML Marker | Category |
|-------|--------------------|---------| 
| `Bold(vec![Plain("Key finding")])` | `b="1"` in rPr | happy-path |
| `Italic(vec![Plain("text")])` | `i="1"` in rPr | happy-path |
| `Code(Arc::from("let x = 1"))` | monospace font run | happy-path |
| `Superscript(vec![Plain("2")])` | `baseline="30000"` | happy-path |
| `Subscript(vec![Plain("2")])` | `baseline="-25000"` | happy-path |
| `Strikethrough(vec![Plain("text")])` | `strike="sngStrike"` | happy-path |
