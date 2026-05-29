---
document_type: verification-property
vp_id: VP-044
title: "Inline: Bold via markdown pattern does not trigger b=1 in output"
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

# VP-044: Bold Via Markdown Pattern Does Not Trigger b=1 In Output

## Property Statement

A text string like `"**bold using markdown"` (or `"**text**"`) passed as a `Plain`
inline node MUST be rendered as literal plain text — the asterisks MUST appear verbatim
in the output and the `b="1"` OOXML attribute MUST NOT be added. The parser MUST NOT
interpret markdown-style inline markup.

Formally: `render_inline_pptx(InlineNode::Plain(Arc::from("**text**")))` produces XML
without `b="1"` and with the literal `**text**` content.

## Motivation

BC-3.05.001 Invariant 2 and the CLAUDE.md forbidden pattern: "String-prefix-based bold
(`**header**`) is an anti-pattern." DI-004 ("values are not implicitly transformed")
extends to inline parsing: the `**` prefix must not implicitly mark a run as bold. The
parser only recognizes structural `InlineNode::Bold(...)` — not markdown syntax.

## Feasibility Assessment

Not Kani-amenable: PPTX rendering involves XML serialization.

`kani_amenable: false` — XML rendering path.

## Test Harness

```rust
// crates/slideforge-pptx/src/tests/inline_markdown.rs
#[test]
fn test_double_asterisk_does_not_produce_bold() {
    let node = InlineNode::Plain(Arc::from("**bold using markdown"));
    let xml = render_inline_pptx(&node);
    assert!(
        !xml.contains(r#"b="1""#),
        "markdown-style bold must not produce b=\"1\": {xml}"
    );
    assert!(
        xml.contains("**bold using markdown") || xml.contains("**bold"),
        "literal asterisks must appear in output: {xml}"
    );
}

#[test]
fn test_double_asterisk_wrapped_does_not_produce_bold() {
    let node = InlineNode::Plain(Arc::from("**text**"));
    let xml = render_inline_pptx(&node);
    assert!(!xml.contains(r#"b="1""#), "no bold from markdown pattern: {xml}");
}

#[test]
fn test_lint_warning_is_issued_for_markdown_pattern() {
    // Per BC-3.05.001 EC-008, a lint warning must be issued
    let node = InlineNode::Plain(Arc::from("**bold using markdown"));
    let (xml, warnings) = render_inline_pptx_with_warnings(&node);
    assert!(
        warnings.iter().any(|w| w.contains("**") || w.contains("markdown")),
        "must issue lint warning for markdown-style bold: {:?}", warnings
    );
    let _ = xml; // suppress unused warning
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `Plain(Arc::from("**bold using markdown"))` | No `b="1"` in XML; literal `**` rendered; lint warning | edge-case (EC-008) |
| `Plain(Arc::from("**text**"))` | No `b="1"` in XML; both `**` sequences rendered literally | edge-case |
| `Bold(vec![Plain("text")])` | `b="1"` present — this IS bold via structural node | happy-path (contrast) |
