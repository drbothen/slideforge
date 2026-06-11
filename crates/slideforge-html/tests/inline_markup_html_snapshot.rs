//! STORY-081 Red Gate tests — HTML exporter slide-level inline markup rendering.
//!
//! ## Traceability
//!
//! | Test function | AC/EC | BC clause | What is verified |
//! |---|---|---|---|
//! | `test_BC_3_05_001_ac005_html_bold_renders_strong` | AC-005 | BC-3.05.001 PC-4 | Bold → `<strong>` |
//! | `test_BC_3_05_001_ac005_html_italic_renders_em` | AC-005 | BC-3.05.001 PC-4 | Italic → `<em>` |
//! | `test_BC_3_05_001_ac005_html_code_renders_code` | AC-005 | BC-3.05.001 PC-4 | Code → `<code>` |
//! | `test_BC_3_05_001_ac005_html_link_renders_anchor` | AC-005 | BC-3.05.001 PC-4 | Link → `<a href>` |
//! | `test_BC_3_05_001_ac005_html_superscript_renders_sup` | AC-005 | BC-3.05.001 PC-4 | Superscript → `<sup>` |
//! | `test_BC_3_05_001_ac005_html_subscript_renders_sub` | AC-005 | BC-3.05.001 PC-4 | Subscript → `<sub>` |
//! | `test_BC_3_05_001_ac005_html_strikethrough_renders_del` | AC-005 | BC-3.05.001 PC-4 | Strikethrough → `<del>` |
//! | `test_BC_3_05_001_ac005_html_highlight_renders_mark` | AC-005 | BC-3.05.001 PC-4 | Highlight → `<mark>` |
//! | `test_BC_3_05_001_ac005_html_all_8_forms_snapshot` | AC-005 | BC-3.05.001 PC-4 | All 8 forms → snapshot |
//! | `test_BC_3_05_001_ac005_html_bullet_inline_markup_wired` | AC-005 | BC-3.05.001 PC-4 | slide bullet with inline markup renders structural HTML (not literal asterisks) |
//! | `test_BC_3_05_001_ac005_html_no_literal_asterisks_in_slide_output` | AC-005 | BC-3.05.001 PC-4 | no `**` in HTML output for bold slide content |
//!
//! ## Red Gate contract
//!
//! ALL tests MUST FAIL before STORY-081 implementation. Tests that assert
//! `<strong>` / `<em>` / `<del>` / `<mark>` etc. in slide body rendering will
//! fail because the slide-level `FrameContent::TextRun` and `FrameContent::Body`
//! paths currently flatten `InlineNode`s to plain text.
//!
//! Note: `render_inline_node` in `slideforge-html` ALREADY correctly maps each
//! variant. The Red Gate failure is upstream: the slide evaluator still produces
//! `FieldValue::Str` (losing inline structure), so the HTML exporter never
//! receives `InlineNode::Bold` etc. for slide content.

#![allow(non_snake_case)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::sync::Arc;

use slideforge_html::render::render_inline_nodes;
use slideforge_layout::{BoundingBox, Frame, FrameContent, LaidOutSlide, PageSize};
use slideforge_types::{Brand, BrandFonts, BrandPalette, Emu, InlineNode, SourceSpan};

// ─── Fixture helpers ──────────────────────────────────────────────────────────

fn make_brand() -> Brand {
    Brand {
        name: Arc::from("test-brand"),
        palette: BrandPalette {
            primary: Arc::from("#003087"),
            secondary: Arc::from("#0066CC"),
            accent: Arc::from("#FF6B35"),
            neutral: Arc::from("#F5F5F5"),
        },
        fonts: BrandFonts {
            heading: Arc::from("Calibri Light"),
            body: Arc::from("Calibri"),
            mono: Arc::from("Courier New"),
            font_size_emu: 457_200,
        },
        layouts: vec![],
        span: SourceSpan::default(),
    }
}

fn default_page_size() -> PageSize {
    PageSize::default()
}

fn make_bbox() -> BoundingBox {
    BoundingBox {
        x: Emu(0),
        y: Emu(0),
        width: Emu(4_000_000),
        height: Emu(500_000),
    }
}

fn make_slide_with_text_run(nodes: Vec<InlineNode>) -> LaidOutSlide {
    LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("content"),
        frames: vec![
            Frame {
                bbox: make_bbox(),
                content: FrameContent::Title(Arc::from("Test Slide")),
                text_flow: None,
                region_role: None,
            },
            Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(600_000),
                    width: Emu(4_000_000),
                    height: Emu(2_000_000),
                },
                content: FrameContent::TextRun(nodes),
                text_flow: None,
                region_role: None,
            },
        ],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    }
}

// ─── Unit tests: render_inline_node produces correct HTML elements ────────────

/// AC-005: `InlineNode::Bold` renders as `<strong>` via `render_inline_nodes`.
///
/// Red Gate: `render_inline_nodes` already returns `<strong>`. This test passes
/// for the unit function. The Red Gate failure is at the pipeline level (slide
/// eval produces Str not Inlines → HTML exporter never receives Bold nodes for
/// slide content). See `test_BC_3_05_001_ac005_html_bullet_inline_markup_wired`.
#[test]
fn test_BC_3_05_001_ac005_html_bold_renders_strong() {
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Plain(Arc::from(
        "bold text",
    ))])];
    let html = render_inline_nodes(&nodes);
    assert!(
        html.contains("<strong>"),
        "AC-005: Bold must render as <strong>; got: {html:?}"
    );
    assert!(
        html.contains("</strong>"),
        "AC-005: Bold must close </strong>; got: {html:?}"
    );
    assert!(
        html.contains("bold text"),
        "AC-005: Bold must contain 'bold text'; got: {html:?}"
    );
}

/// AC-005: `InlineNode::Italic` renders as `<em>`.
#[test]
fn test_BC_3_05_001_ac005_html_italic_renders_em() {
    let nodes = vec![InlineNode::Italic(vec![InlineNode::Plain(Arc::from(
        "italic text",
    ))])];
    let html = render_inline_nodes(&nodes);
    assert!(
        html.contains("<em>") && html.contains("</em>"),
        "AC-005: Italic must render as <em>; got: {html:?}"
    );
}

/// AC-005: `InlineNode::Code` renders as `<code>`.
#[test]
fn test_BC_3_05_001_ac005_html_code_renders_code() {
    let nodes = vec![InlineNode::Code(Arc::from("fn foo() {}"))];
    let html = render_inline_nodes(&nodes);
    assert!(
        html.contains("<code>") && html.contains("</code>"),
        "AC-005: Code must render as <code>; got: {html:?}"
    );
    assert!(
        html.contains("fn foo()"),
        "AC-005: Code must contain code text; got: {html:?}"
    );
}

/// AC-005: `InlineNode::Link` renders as `<a href="...">`.
#[test]
fn test_BC_3_05_001_ac005_html_link_renders_anchor() {
    let nodes = vec![InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("click here"))],
        url: Arc::from("https://example.com"),
    }];
    let html = render_inline_nodes(&nodes);
    assert!(
        html.contains("<a ") && html.contains("href="),
        "AC-005: Link must render as <a href=...>; got: {html:?}"
    );
    assert!(
        html.contains("https://example.com"),
        "AC-005: Link must contain URL; got: {html:?}"
    );
    assert!(
        html.contains("click here"),
        "AC-005: Link must contain display text; got: {html:?}"
    );
}

/// AC-005: `InlineNode::Superscript` renders as `<sup>`.
#[test]
fn test_BC_3_05_001_ac005_html_superscript_renders_sup() {
    let nodes = vec![InlineNode::Superscript(vec![InlineNode::Plain(Arc::from(
        "2",
    ))])];
    let html = render_inline_nodes(&nodes);
    assert!(
        html.contains("<sup>") && html.contains("</sup>"),
        "AC-005: Superscript must render as <sup>; got: {html:?}"
    );
}

/// AC-005: `InlineNode::Subscript` renders as `<sub>`.
#[test]
fn test_BC_3_05_001_ac005_html_subscript_renders_sub() {
    let nodes = vec![InlineNode::Subscript(vec![InlineNode::Plain(Arc::from(
        "2",
    ))])];
    let html = render_inline_nodes(&nodes);
    assert!(
        html.contains("<sub>") && html.contains("</sub>"),
        "AC-005: Subscript must render as <sub>; got: {html:?}"
    );
}

/// AC-005: `InlineNode::Strikethrough` renders as `<del>`.
#[test]
fn test_BC_3_05_001_ac005_html_strikethrough_renders_del() {
    let nodes = vec![InlineNode::Strikethrough(vec![InlineNode::Plain(
        Arc::from("old text"),
    )])];
    let html = render_inline_nodes(&nodes);
    assert!(
        html.contains("<del>") && html.contains("</del>"),
        "AC-005: Strikethrough must render as <del>; got: {html:?}"
    );
}

/// AC-005: `InlineNode::Highlight` renders as `<mark>`.
#[test]
fn test_BC_3_05_001_ac005_html_highlight_renders_mark() {
    let nodes = vec![InlineNode::Highlight(vec![InlineNode::Plain(Arc::from(
        "highlighted",
    ))])];
    let html = render_inline_nodes(&nodes);
    assert!(
        html.contains("<mark>") && html.contains("</mark>"),
        "AC-005: Highlight must render as <mark>; got: {html:?}"
    );
}

/// AC-005 snapshot: all 8 inline markup forms produce the correct semantic HTML
/// when rendered via `render_inline_nodes`.
#[test]
fn test_BC_3_05_001_ac005_html_all_8_forms_snapshot() {
    let nodes = vec![
        InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold"))]),
        InlineNode::Plain(Arc::from(" ")),
        InlineNode::Italic(vec![InlineNode::Plain(Arc::from("italic"))]),
        InlineNode::Plain(Arc::from(" ")),
        InlineNode::Code(Arc::from("code")),
        InlineNode::Plain(Arc::from(" ")),
        InlineNode::Link {
            text: vec![InlineNode::Plain(Arc::from("link"))],
            url: Arc::from("https://example.com"),
        },
        InlineNode::Plain(Arc::from(" ")),
        InlineNode::Superscript(vec![InlineNode::Plain(Arc::from("sup"))]),
        InlineNode::Plain(Arc::from(" ")),
        InlineNode::Subscript(vec![InlineNode::Plain(Arc::from("sub"))]),
        InlineNode::Plain(Arc::from(" ")),
        InlineNode::Strikethrough(vec![InlineNode::Plain(Arc::from("del"))]),
        InlineNode::Plain(Arc::from(" ")),
        InlineNode::Highlight(vec![InlineNode::Plain(Arc::from("highlight"))]),
    ];

    let html = render_inline_nodes(&nodes);

    // Assert all 8 semantic HTML elements are present.
    assert!(html.contains("<strong>"), "Bold must produce <strong>");
    assert!(html.contains("<em>"), "Italic must produce <em>");
    assert!(html.contains("<code>"), "Code must produce <code>");
    assert!(html.contains("<a "), "Link must produce <a>");
    assert!(html.contains("<sup>"), "Superscript must produce <sup>");
    assert!(html.contains("<sub>"), "Subscript must produce <sub>");
    assert!(html.contains("<del>"), "Strikethrough must produce <del>");
    assert!(html.contains("<mark>"), "Highlight must produce <mark>");

    // Snapshot for regression detection.
    insta::assert_snapshot!(html);
}

/// AC-005 pipeline test: a slide with a `FrameContent::TextRun` containing
/// `InlineNode::Bold` must render the full slide HTML with `<strong>` in the output.
///
/// **Red Gate failure:** This test constructs a `LaidOutSlide` with a `TextRun`
/// frame carrying `InlineNode::Bold`. After STORY-081 is complete, the HTML
/// exporter must use `render_inline_nodes` on the `TextRun` nodes.
/// Before STORY-081, `extract_inline_text` flattens to plain text → no `<strong>`.
#[test]
fn test_BC_3_05_001_ac005_html_bullet_inline_markup_wired() {
    use slideforge_html::render::HeadingLevel;
    use slideforge_html::render::render_slide_to_html;

    let nodes = vec![
        InlineNode::Bold(vec![InlineNode::Plain(Arc::from("Key finding"))]),
        InlineNode::Plain(Arc::from(": revenue up 12%")),
    ];
    let slide = make_slide_with_text_run(nodes);
    let brand = make_brand();
    let page_size = default_page_size();

    let html = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);

    assert!(
        html.contains("<strong>"),
        "AC-005: slide TextRun with Bold InlineNode must produce <strong> in HTML output; \
         got (excerpt): {}",
        &html[..html.len().min(500)]
    );
    assert!(
        html.contains("Key finding"),
        "AC-005: bold text content must appear in HTML output"
    );
}

// =============================================================================
// F-P13-002: load-bearing combined-form assertion — HTML
//
// `Bold([Italic([Plain("x")])])` → `render_inline_nodes` must produce
// nested `<strong><em>…</em></strong>`, NOT flat siblings like
// `<strong></strong><em>…</em>`.
//
// HTML renders nested elements per the nesting structure of the InlineNode
// tree. If the Bold arm were to flatten children before delegating to Italic,
// or if the Italic arm produced a sibling rather than a nested element, this
// assertion would fail.
// =============================================================================

/// F-P13-002 load-bearing: `Bold([Italic([Plain("x")])])` → HTML renders as
/// `<strong><em>…</em></strong>` (nested, not flat siblings).
///
/// Exercises `render_inline_nodes` in the HTML crate. The Bold arm must wrap
/// the output of its children with `<strong>…</strong>`, and the Italic arm
/// (nested inside Bold) wraps with `<em>…</em>`. The result is
/// `<strong><em>bi</em></strong>`.
///
/// A regression that flattens children or reverses nesting would produce
/// `<strong></strong><em>bi</em>` or `<em><strong>bi</strong></em>` —
/// both would FAIL at least one of the structure assertions below.
#[test]
fn test_f_p13_002_html_bold_italic_combined_form_nested_strong_em() {
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Italic(vec![
        InlineNode::Plain(Arc::from("bi")),
    ])])];
    let html = render_inline_nodes(&nodes);

    // The output must contain <strong> wrapping <em>.
    assert!(
        html.contains("<strong>") && html.contains("<em>"),
        "F-P13-002: Bold([Italic([Plain])]) must produce both <strong> and <em>; \
         got: {html:?}"
    );

    // Verify nesting: <strong> must appear BEFORE <em> in the output.
    // This ensures the structure is <strong><em>…</em></strong> not the reverse.
    let strong_pos = html.find("<strong>").expect("<strong> must be present");
    let em_pos = html.find("<em>").expect("<em> must be present");
    assert!(
        strong_pos < em_pos,
        "F-P13-002: <strong> must appear before <em> (nested: <strong><em>…</em></strong>); \
         got: {html:?}"
    );

    // The text content must appear inside both elements.
    assert!(
        html.contains("bi"),
        "F-P13-002: text 'bi' must be present in HTML output; got: {html:?}"
    );
}

/// AC-005 anti-pattern rejection: the HTML output for a slide with bold markup
/// must NOT contain literal `**` asterisks anywhere.
///
/// **Red Gate failure:** Currently `TextRun` nodes are flattened to plain text
/// by `extract_inline_text`, which discards the markup structure. After STORY-081,
/// the exporter must call `render_inline_nodes` on the `TextRun` `Vec<InlineNode>`.
#[test]
fn test_BC_3_05_001_ac005_html_no_literal_asterisks_in_slide_output() {
    use slideforge_html::render::HeadingLevel;
    use slideforge_html::render::render_slide_to_html;

    // This simulates the pre-STORY-081 state that MUST change:
    // if a slide with a bullet "**bold**" somehow produces literal asterisks
    // in the HTML output, that is the R1 anti-pattern.
    // We construct the InlineNode tree directly (post-eval) to test the exporter layer.
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold"))])];
    let slide = make_slide_with_text_run(nodes);
    let brand = make_brand();
    let page_size = default_page_size();

    let html = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);

    // The HTML output must NOT contain literal ** — if it does, the exporter
    // is falling back to plain text extraction instead of rendering inline nodes.
    assert!(
        !html.contains("**"),
        "AC-005: HTML slide output must not contain literal '**'; \
         this indicates the exporter is not calling render_inline_nodes on TextRun frames. \
         HTML (excerpt): {}",
        &html[..html.len().min(500)]
    );
}
