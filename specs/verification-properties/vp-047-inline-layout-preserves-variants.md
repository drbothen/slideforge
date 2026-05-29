---
document_type: verification-property
vp_id: VP-047
title: "Inline: all 12 variants survive layout pass in FrameContent::TextRun"
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

# VP-047: All 12 Inline Variants Survive Layout Pass in FrameContent::TextRun

## Property Statement

The layout stage MUST preserve `InlineNode` sequences verbatim in `FrameContent::TextRun`.
For any `Vec<InlineNode>` containing all 12 variants (Plain, Bold, Italic, Code, Link,
Math, Footnote, Xref, Superscript, Subscript, Strikethrough, Highlight), the layout output
MUST contain a `FrameContent::TextRun` with the same 12 InlineNode variants in the same
order — no variant may be dropped, collapsed, or transformed by the layout stage.

Formally: `layout_inline_run(nodes) == FrameContent::TextRun(nodes.clone())`
(the layout stage is transparent for inline content).

## Motivation

BC-3.05.001 Invariant 6. The layout stage does NOT produce format-specific markup —
that is the exporter's responsibility. If the layout stage drops or transforms any
InlineNode variant, exporters receive incomplete input and produce incorrect output.
This property is an exhaustive coverage check across all 12 variants.

## Feasibility Assessment

Not Kani-amenable: the layout pipeline involves slide IR construction.
Verified via a unit test constructing a `Vec<InlineNode>` with one instance of each
of the 12 variants and verifying the `FrameContent::TextRun` preserves all of them.

`kani_amenable: false` — IR transformation, not pure arithmetic.

## Test Harness

```rust
// crates/slideforge-layout/src/tests/inline_preservation.rs
#[test]
fn test_all_12_variants_survive_layout_pass() {
    let nodes = vec![
        InlineNode::Plain(Arc::from("plain")),
        InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold"))]),
        InlineNode::Italic(vec![InlineNode::Plain(Arc::from("italic"))]),
        InlineNode::Code(Arc::from("code()")),
        InlineNode::Link { text: vec![InlineNode::Plain(Arc::from("link"))], url: Arc::from("https://example.com") },
        InlineNode::Math(MathNode { source: Arc::from(r"E = mc^2") }),
        InlineNode::Footnote(vec![InlineNode::Plain(Arc::from("footnote"))]),
        InlineNode::Xref(Arc::from("slide-1")),
        InlineNode::Superscript(vec![InlineNode::Plain(Arc::from("sup"))]),
        InlineNode::Subscript(vec![InlineNode::Plain(Arc::from("sub"))]),
        InlineNode::Strikethrough(vec![InlineNode::Plain(Arc::from("strike"))]),
        InlineNode::Highlight(vec![InlineNode::Plain(Arc::from("highlight"))]),
    ];
    assert_eq!(nodes.len(), 12, "must have exactly 12 variants");

    let frame_content = layout_inline_run(&nodes, &BrandConfig::default());
    match frame_content {
        FrameContent::TextRun(preserved) => {
            assert_eq!(preserved.len(), 12, "must preserve all 12 inline variants");
            // Verify each variant is preserved in order
            assert!(matches!(preserved[0], InlineNode::Plain(_)));
            assert!(matches!(preserved[1], InlineNode::Bold(_)));
            assert!(matches!(preserved[2], InlineNode::Italic(_)));
            assert!(matches!(preserved[3], InlineNode::Code(_)));
            assert!(matches!(preserved[4], InlineNode::Link { .. }));
            assert!(matches!(preserved[5], InlineNode::Math(_)));
            assert!(matches!(preserved[6], InlineNode::Footnote(_)));
            assert!(matches!(preserved[7], InlineNode::Xref(_)));
            assert!(matches!(preserved[8], InlineNode::Superscript(_)));
            assert!(matches!(preserved[9], InlineNode::Subscript(_)));
            assert!(matches!(preserved[10], InlineNode::Strikethrough(_)));
            assert!(matches!(preserved[11], InlineNode::Highlight(_)));
        }
        _ => panic!("expected FrameContent::TextRun"),
    }
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `Vec<InlineNode>` with all 12 variants | `FrameContent::TextRun` preserving all 12, in order | exhaustive |
| `Vec<InlineNode>` with single `Plain` | `FrameContent::TextRun` with same `Plain` node | happy-path |
