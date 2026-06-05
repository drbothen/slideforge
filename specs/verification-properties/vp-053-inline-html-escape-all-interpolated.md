---
document_type: verification-property
vp_id: VP-053
title: "Inline HTML: all interpolated values are HTML-escaped in all output positions"
module: slideforge-plugin-api
tool: proptest
phase: P3
priority: P1
status: draft
kani_amenable: false
bc_trace: [BC-3.05.001, EC-009, EC-010]
anchored_to_bc: BC-3.05.001
traces_to: .factory/specs/verification-properties/VP-INDEX.md
spec_version: "1.0.0"
---

# VP-053: Inline HTML — All Interpolated Values Are HTML-Escaped in All Output Positions

## Property Statement

For any `InlineNode`, `DefaultInlineFormat::render(node, InlineOutputFormat::Html)`
MUST produce output that contains no unescaped occurrences of `&`, `<`, `>`, or `"`
originating from interpolated string values in ANY output position — including:

1. **Text positions** — `Plain` content, `Code` content, `Math.latex` source
   (e.g., `Math { latex: "x < y" }` must NOT produce `x < y` in the HTML span content)
2. **Attribute positions** — `Link.url` in `href="..."`, `Xref.id` in `href="#{id}"`
   (e.g., `Link { url: "https://example.com/?a=1&b=2" }` must produce
   `href="https://example.com/?a=1&amp;b=2"`, not `href="...&b=2"`)

Formally (proptest quantified over arbitrary strings):

```
∀ content: String, ∀ url: String, ∀ id: String, ∀ latex: String:
  let html_plain  = render(Plain(content),       Html)?;
  let html_code   = render(Code(content),        Html)?;
  let html_link   = render(Link { text: plain_nodes, url }, Html)?;
  let html_xref   = render(Xref(id),             Html)?;
  let html_math   = render(Math { latex },       Html)?;
  → none of {html_plain, html_code, html_link, html_xref, html_math}
    contains an unescaped raw `&`, `<`, `>`, or `"` from the interpolated value
```

Characters appearing in HTML structural context (e.g., the `<` in `<strong>`) are
excluded — only interpolated user-data characters are checked.

## Motivation

BC-3.05.001 v1.3.6 tightened the HTML postcondition: HTML-escaping applies to ALL
interpolated values, not only `Plain` text. EC-009 (Link URL with `&`/`<` in href)
and EC-010 (Math LaTeX with `<` in content) were added with canonical test vectors.

The existing VP-043..VP-047 do not cover HTML output — they cover PPTX OOXML rendering
(VP-043/VP-044), layout-stage depth checking (VP-045), xref validation non-traversal
(VP-046), and layout-pass preservation (VP-047). This gap was confirmed by the
product-owner's flag when tightening BC-3.05.001 to v1.3.6.

This property is security-relevant: unescaped interpolated content in HTML output
can enable HTML injection if the output is used in a browser context (web preview,
HTML export). A property-based test provides much stronger coverage than a fixed
set of unit tests because it exercises adversarially-constructed inputs from arbitrary
data sources.

## Feasibility Assessment

Not Kani-amenable: HTML rendering involves string manipulation and format detection
logic — not pure arithmetic over bounded integer domains. However, the escaping
invariant is an ideal `proptest` target because:

- The property holds for ALL string inputs — no domain restriction required
- `proptest` can generate adversarial inputs containing `&`, `<`, `>`, `"`,
  multi-byte UTF-8 sequences, and percent-encoded URL sequences
- The oracle is simple: split rendered HTML on structural `<tag>` markers and
  verify interpolated segments contain no unescaped meta-characters

`kani_amenable: false` — string escaping logic; tested via proptest.

## Proof Harness Skeleton

```rust
// crates/slideforge-plugin-api/src/tests/inline_html_escape.rs
use proptest::prelude::*;
use std::sync::Arc;
use crate::inline_formats::DefaultInlineFormat;
use slideforge_plugin_api::traits::inline_format::{InlineFormat, InlineOutputFormat};
use slideforge_types::inline::{InlineNode, MathNode};
use slideforge_types::span::SourceSpan;

/// Returns true if the string contains a raw (unescaped) HTML meta-character
/// that is NOT part of a structural HTML tag or entity.
fn contains_unescaped_meta(s: &str) -> bool {
    // Strip all `<tag ...>` and `</tag>` structural elements, then check remnant
    // for unescaped chars. Simplified oracle — replace structural tags first.
    let without_tags = strip_html_tags(s);
    // After stripping tags, any remaining `<` or `>` is unescaped content
    // Also check `&` not followed by entity pattern (e.g., `&amp;`, `&lt;`, etc.)
    let unescaped_angle = without_tags.contains('<') || without_tags.contains('>');
    let unescaped_amp = without_tags.contains('&')
        && !without_tags.contains("&amp;")
        && !without_tags.contains("&lt;")
        && !without_tags.contains("&gt;")
        && !without_tags.contains("&quot;");
    // Simplified check — full impl in test crate
    unescaped_angle || unescaped_amp
}

proptest! {
    /// VP-053: Plain text content is escaped in all output positions
    #[test]
    fn prop_plain_html_escapes_meta_chars(content in ".*") {
        let node = InlineNode::Plain(Arc::from(content.as_str()));
        let html = DefaultInlineFormat.render(&node, InlineOutputFormat::Html)
            .expect("Plain html render must not fail");
        // After stripping structural <span>/<strong>/etc, no raw & < > " from content
        let content_in_output = extract_text_content(&html);
        prop_assert!(
            !content_in_output.contains('<') && !content_in_output.contains('>'),
            "Unescaped < or > in Plain HTML output: {html:?} (input: {content:?})"
        );
        // & must be escaped (any & in input must appear as &amp; in output)
        if content.contains('&') {
            prop_assert!(
                html.contains("&amp;"),
                "& in Plain input must produce &amp; in HTML output: {html:?}"
            );
        }
    }

    /// EC-009: Link URL with & and < in href attribute must be escaped
    #[test]
    fn prop_link_url_escaped_in_href(
        url in "https://[a-z]+\\.example\\.com/\\?[a-z]=1&[a-z]=2.*",
        text in "[a-z]+"
    ) {
        let node = InlineNode::Link {
            text: vec![InlineNode::Plain(Arc::from(text.as_str()))],
            url: Arc::from(url.as_str()),
        };
        let html = DefaultInlineFormat.render(&node, InlineOutputFormat::Html)
            .expect("Link html render must not fail");
        // The href attribute value must have & escaped
        if url.contains('&') {
            prop_assert!(
                !html.contains(&format!("href=\"{url}\"")),
                "Unescaped URL in href: {html:?}"
            );
            prop_assert!(
                html.contains("&amp;") || !html.contains('&'),
                "& in URL must be escaped as &amp; in href: {html:?}"
            );
        }
        // href must not contain unescaped < or >
        if url.contains('<') || url.contains('>') {
            prop_assert!(
                !html.contains("<\"") && !html.contains(">\""),
                "Unescaped < or > in href attribute: {html:?}"
            );
        }
    }

    /// EC-010: Math LaTeX with < in content must be escaped in HTML span
    #[test]
    fn prop_math_latex_escaped_in_html_span(latex in ".*[<>&\"]+.*") {
        let node = InlineNode::Math(MathNode::inline(Arc::from(latex.as_str()), SourceSpan::default()));
        let html = DefaultInlineFormat.render(&node, InlineOutputFormat::Html)
            .expect("Math html render must not fail");
        // LaTeX content inside <span class="math"> must not contain unescaped < > &
        if latex.contains('<') {
            prop_assert!(
                !html.contains(&latex),
                "Unescaped LaTeX content in Math HTML span: {html:?}"
            );
        }
    }

    /// Xref id is escaped in href attribute position
    #[test]
    fn prop_xref_id_escaped_in_href(id in "[a-z<>&\"]+") {
        let node = InlineNode::Xref(Arc::from(id.as_str()));
        let html = DefaultInlineFormat.render(&node, InlineOutputFormat::Html)
            .expect("Xref html render must not fail");
        // href="#{id}" must have id HTML-escaped
        if id.contains('<') || id.contains('&') || id.contains('"') {
            prop_assert!(
                !html.contains(&format!("href=\"#{id}\"")),
                "Unescaped Xref id in href: {html:?}"
            );
        }
    }
}
```

## Test Vectors (Canonical — BC-3.05.001 v1.3.6)

| Input Node | Expected HTML Output | Edge Case ID | Category |
|-----------|---------------------|-------------|----------|
| `Plain("a & b < c > d")` | `a &amp; b &lt; c &gt; d` (no tags — or inside structural tag) | — | text-position escaping |
| `Link { text: [Plain("click")], url: "https://x.com/?a=1&b=2" }` | `href="https://x.com/?a=1&amp;b=2"` | EC-009 | attribute-position escaping |
| `Link { text: [Plain("x")], url: "https://x.com/?q=a<b" }` | `href="https://x.com/?q=a&lt;b"` | EC-009 | attribute `<` escaping |
| `Math { latex: "x < y" }` | `<span class="math">x &lt; y</span>` | EC-010 | text-position math escaping |
| `Xref("sec&one")` | `href="#sec&amp;one"` | — | attribute-position xref escaping |
| `Code("fn f() -> &str")` | `<code>fn f() -&gt; &amp;str</code>` | — | text-position code escaping |
| `Plain("no meta chars")` | `no meta chars` | — | happy-path (no escaping needed) |

## BC Traceability

| BC | Invariant/Postcondition | Covered |
|----|------------------------|---------|
| BC-3.05.001 v1.3.6 | HTML postcondition: ALL interpolated values HTML-escaped | YES |
| BC-3.05.001 EC-009 | Link URL `&`/`<` in href attribute position | YES |
| BC-3.05.001 EC-010 | Math LaTeX `<` in HTML span content | YES |
