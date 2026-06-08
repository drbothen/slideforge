//! Slide-to-HTML rendering functions.
//!
//! This module provides the core rendering primitives:
//!
//! - [`render_slide_to_html`] — renders a single [`LaidOutSlide`] to an HTML
//!   fragment string. Returns `String` (not `dyn Write`) so STORY-047 can
//!   serialize it as JSON for WebSocket push. (Previous Story Intelligence note)
//!
//! - [`render_element_to_html`] — renders a single [`Frame`] content to an HTML
//!   fragment string.
//!
//! - [`render_svg_chart`] — injects `role="img"` and `<title>` into an SVG
//!   string via `quick-xml` XML manipulation after usvg geometry processing.
//!   MUST NOT use the usvg tree API for accessibility injection — usvg 0.47.0
//!   strips all non-presentation attributes (AC-003/AC-005 / export-architecture v1.2).
//!
//! ## usvg limitation (export-architecture v1.2)
//!
//! `usvg` is used for geometry normalization and validation ONLY. After usvg
//! processing, `quick-xml` XML manipulation is used to inject `role="img"` on
//! the outer `<svg>` element and prepend a `<title>` child with the alt text.
//! This ensures accessibility attributes survive through to the rendered HTML.
//!
//! ## URL scheme security (AC-010 / CWE-601)
//!
//! All `Link` and `Xref` inline nodes processed in render functions must be
//! validated via [`crate::exporter::is_safe_link_scheme`] before being emitted
//! as `href` attributes.

use slideforge_layout::{Frame, FrameContent, LaidOutSlide};
use slideforge_types::{AltText, Brand, ContentBlock, Emu, InlineNode};

use crate::exporter::is_safe_link_scheme;

/// Default SVG canvas width in EMU (matches `slideforge_layout::DEFAULT_PAGE_WIDTH`).
const CANVAS_WIDTH_EMU: Emu = Emu(9_144_000);
/// Default SVG canvas height in EMU (matches `slideforge_layout::DEFAULT_PAGE_HEIGHT`).
const CANVAS_HEIGHT_EMU: Emu = Emu(5_143_500);

// ─────────────────────────────────────────────────────────────────────────────
// Inline node rendering
// ─────────────────────────────────────────────────────────────────────────────

/// Render a sequence of [`InlineNode`]s to an HTML string.
///
/// All `Link` URLs are validated via [`is_safe_link_scheme`] before becoming
/// `href` attributes. Disallowed schemes are silently dropped with a
/// `tracing::warn!` (AC-010 / CWE-601).
#[must_use]
pub fn render_inline_nodes(nodes: &[InlineNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        out.push_str(&render_inline_node(node));
    }
    out
}

/// Render a single [`InlineNode`] to an HTML fragment.
#[must_use]
fn render_inline_node(node: &InlineNode) -> String {
    match node {
        InlineNode::Plain(text) => html_escape::encode_text(text).into_owned(),
        InlineNode::Bold(children) => {
            format!("<strong>{}</strong>", render_inline_nodes(children))
        },
        InlineNode::Italic(children) => {
            format!("<em>{}</em>", render_inline_nodes(children))
        },
        InlineNode::Code(text) => {
            format!("<code>{}</code>", html_escape::encode_text(text))
        },
        InlineNode::Link { url, text } => {
            let inner = render_inline_nodes(text);
            // AC-010 / CWE-601: validate scheme before emitting href.
            // is_safe_link_scheme already emits tracing::warn! on rejection —
            // do NOT log here again (F-006: single warn per rejection, TD-VSDD-060).
            if is_safe_link_scheme(url) {
                let safe_url = html_escape::encode_double_quoted_attribute(url);
                format!(r#"<a href="{safe_url}">{inner}</a>"#)
            } else {
                // Drop href — render as plain span.
                format!("<span>{inner}</span>")
            }
        },
        InlineNode::Superscript(children) => {
            format!("<sup>{}</sup>", render_inline_nodes(children))
        },
        InlineNode::Subscript(children) => {
            format!("<sub>{}</sub>", render_inline_nodes(children))
        },
        InlineNode::Strikethrough(children) => {
            format!("<del>{}</del>", render_inline_nodes(children))
        },
        InlineNode::Highlight(children) => {
            format!("<mark>{}</mark>", render_inline_nodes(children))
        },
        InlineNode::Footnote(children) => {
            format!("<small>{}</small>", render_inline_nodes(children))
        },
        InlineNode::Math(math_node) => {
            // Render math as a <code> element (full MathML is STORY-045 scope).
            format!(
                "<code class=\"math\">{}</code>",
                html_escape::encode_text(&math_node.latex)
            )
        },
        InlineNode::Xref(target) => {
            // Xref becomes a fragment anchor to a slide ID.
            // F-009 (AC-010): route through is_safe_link_scheme even though Xref
            // always prepends '#' (making the full URL fragment-only). This ensures
            // the allowlist logic is the single enforcement point for ALL link/xref
            // rendering — consistent with F-006 single-log-site rule.
            let fragment_url = format!("#{target}");
            let display = html_escape::encode_text(target);
            if is_safe_link_scheme(&fragment_url) {
                let safe_target = html_escape::encode_double_quoted_attribute(target);
                format!("<a href=\"#{safe_target}\">{display}</a>")
            } else {
                // Defensively drop href if fragment URL is somehow rejected.
                format!("<span>{display}</span>")
            }
        },
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ContentBlock rendering (F-003)
// ─────────────────────────────────────────────────────────────────────────────

/// Render a single [`ContentBlock`] to an HTML string.
///
/// Dispatches on the block variant:
/// - `Text` → `<p>` element containing rendered inline nodes
/// - `Bullets` → `<ul><li>` structure with nested sub-bullets
/// - `Math` → code block with math class (full `MathML` rendering is STORY-045 scope)
/// - `Chart` / `Diagram` / `Image` → these are handled via `FrameContent`
///   variants before reaching `Body`; here we emit a placeholder note
/// - `Table` → `<table>` (basic rendering)
/// - `ColorBar` → placeholder text (full bar is via `FrameContent::ColorBar`)
/// - `Shape` → placeholder (shapes are handled via `FrameContent::Shape`)
///
/// F-003: MUST NOT use `format!("{block:?}")` — all variants must render
/// semantic HTML, never Rust debug output.
#[must_use]
pub fn render_content_block(block: &ContentBlock) -> String {
    match block {
        ContentBlock::Text(text_block) => {
            // Render a text paragraph as <p> with inline nodes.
            format!("<p>{}</p>", render_inline_nodes(&text_block.inlines))
        },
        ContentBlock::Bullets(items) => {
            // Render a bullet list as <ul>.
            let mut list = String::from("<ul>\n");
            for item in items {
                list.push_str(&render_bullet_item(item));
            }
            list.push_str("</ul>");
            list
        },
        ContentBlock::Math(math_node) => {
            // Math block — render as code (full MathML is STORY-045 scope).
            format!(
                "<code class=\"math\">{}</code>",
                html_escape::encode_text(&math_node.latex)
            )
        },
        ContentBlock::Chart(_) | ContentBlock::Diagram(_) | ContentBlock::Image(_) => {
            // These variants appear as FrameContent (not Body) after layout.
            // If they somehow appear in a Body block, render as empty (no debug).
            String::new()
        },
        ContentBlock::Table(_) => {
            // Table rendering — placeholder (full table renderer is a future story).
            // Emit a semantic <table> element (even if empty) rather than debug text.
            String::from("<table></table>")
        },
        ContentBlock::ColorBar(spec) => {
            // ColorBar in Body context — render accessible text label for the percent.
            // The visual bar is handled via FrameContent::ColorBar at the slide level.
            format!(
                r#"<span role="progressbar" aria-valuenow="{pct}" aria-valuemin="0" aria-valuemax="100">{pct}%</span>"#,
                pct = spec.percent
            )
        },
        ContentBlock::Shape(_) => {
            // Shape in Body context — shapes are handled via FrameContent::Shape.
            String::new()
        },
    }
}

/// Render a single [`slideforge_types::BulletItem`] and its children to HTML.
///
/// Produces `<li>` elements with nested `<ul>` for sub-bullets.
#[must_use]
fn render_bullet_item(item: &slideforge_types::BulletItem) -> String {
    let content = render_inline_nodes(&item.inlines);
    if item.children.is_empty() {
        format!("<li>{content}</li>\n")
    } else {
        let mut nested = String::from("<ul>\n");
        for child in &item.children {
            nested.push_str(&render_bullet_item(child));
        }
        nested.push_str("</ul>");
        format!("<li>{content}\n{nested}</li>\n")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Render a slide frame as SVG content positioned at the frame's bounding box.
///
/// Returns an SVG fragment string (elements that go inside the outer `<svg>` canvas).
/// Text/heading content is placed inside `<foreignObject>` to preserve semantic HTML
/// structure (headings, paragraphs) within the SVG canvas (BC-4.03.003 invariant 2).
///
/// F-001 (Architecture Compliance Rule 1): every slide is rendered as an `<svg>`
/// element; this function produces the content that goes inside that canvas.
///
/// F-002: Images are rendered as `<image>` inside the SVG — never as bare `<img>`.
#[must_use]
fn render_frame_as_svg_content(frame: &Frame, heading_level: u32) -> String {
    let x = frame.bbox.x.0;
    let y = frame.bbox.y.0;
    let w = frame.bbox.width.0;
    let h = frame.bbox.height.0;

    match &frame.content {
        FrameContent::Title(text) => {
            // Title → <h1> inside <foreignObject> for semantic heading in SVG canvas.
            let escaped = html_escape::encode_text(text);
            format!(
                r#"<foreignObject x="{x}" y="{y}" width="{w}" height="{h}"><h1 xmlns="http://www.w3.org/1999/xhtml">{escaped}</h1></foreignObject>"#
            )
        },
        FrameContent::Subtitle(text) => {
            // Subtitle → <hN> inside <foreignObject>; level determined by caller.
            let escaped = html_escape::encode_text(text);
            let hl = heading_level.clamp(1, 6);
            format!(
                r#"<foreignObject x="{x}" y="{y}" width="{w}" height="{h}"><h{hl} xmlns="http://www.w3.org/1999/xhtml">{escaped}</h{hl}></foreignObject>"#
            )
        },
        FrameContent::Body(blocks) => {
            let body_html: String = blocks.iter().map(render_content_block).collect();
            let escaped_body = body_html; // body_html is already HTML-escaped at block level
            format!(
                r#"<foreignObject x="{x}" y="{y}" width="{w}" height="{h}"><div xmlns="http://www.w3.org/1999/xhtml">{escaped_body}</div></foreignObject>"#
            )
        },
        FrameContent::Image { alt } => {
            // F-002: Image must render as <image> inside SVG — never bare <img>.
            // Since Image frames don't carry a src path (path is resolved elsewhere),
            // we emit an accessible SVG placeholder with the alt text.
            match alt {
                AltText::Provided(text) => {
                    let safe_alt = html_escape::encode_double_quoted_attribute(text);
                    let escaped_text = html_escape::encode_text(text);
                    format!(
                        "<svg x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" \
                         role=\"img\" aria-label=\"{safe_alt}\">\
                         <title>{escaped_text}</title>\
                         <rect width=\"{w}\" height=\"{h}\" fill=\"none\" stroke=\"#cccccc\"/>\
                         </svg>"
                    )
                },
                AltText::Decorative | AltText::Unspecified => {
                    format!(
                        "<svg x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" \
                         role=\"presentation\" aria-hidden=\"true\">\
                         <rect width=\"{w}\" height=\"{h}\" fill=\"none\" stroke=\"#cccccc\"/>\
                         </svg>"
                    )
                },
            }
        },
        FrameContent::Chart { alt } => {
            let alt_text = alt_text_str(alt);
            let placeholder_svg =
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"></svg>"#;
            let chart_svg = render_svg_chart(placeholder_svg, alt_text);
            // Position the chart SVG at the frame's bbox.
            format!(r#"<g transform="translate({x}, {y})">{chart_svg}</g>"#)
        },
        FrameContent::Diagram { svg, alt } => {
            let alt_text = alt_text_str(alt);
            let diagram_svg = render_svg_chart(svg.as_str(), alt_text);
            format!(r#"<g transform="translate({x}, {y})">{diagram_svg}</g>"#)
        },
        FrameContent::Shape(shape_frame) => {
            let alt_text = alt_text_str(&shape_frame.alt);
            let placeholder_svg =
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"></svg>"#;
            let shape_svg = render_svg_chart(placeholder_svg, alt_text);
            format!(r#"<g transform="translate({x}, {y})">{shape_svg}</g>"#)
        },
        FrameContent::TextRun(nodes) => {
            let text_html = format!("<p>{}</p>", render_inline_nodes(nodes));
            format!(
                r#"<foreignObject x="{x}" y="{y}" width="{w}" height="{h}"><div xmlns="http://www.w3.org/1999/xhtml">{text_html}</div></foreignObject>"#
            )
        },
        FrameContent::ColorBar {
            filled_width_emu,
            total_width_emu,
            percent,
            color,
        } => {
            let _ = total_width_emu;
            let color_hex = format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b);
            let fill_w = filled_width_emu.0;
            format!(
                r#"<g transform="translate({x}, {y})"><rect width="{fill_w}" height="{h}" fill="{color_hex}" role="img" aria-label="{percent}% complete"/></g>"#
            )
        },
        FrameContent::Empty => String::new(),
        FrameContent::ErrorSlidePlaceholder {
            svg,
            slide_title,
            error_code,
            message,
        } => {
            let alt_text = format!("Error on slide '{slide_title}': [{error_code}] {message}");
            let error_svg = render_svg_chart(svg, &alt_text);
            format!(r#"<g transform="translate({x}, {y})">{error_svg}</g>"#)
        },
    }
}

/// Render a single slide to an HTML fragment string.
///
/// The returned `String` is a self-contained HTML fragment representing one
/// slide: an `<article>` landmark containing an `<svg>` canvas with frame
/// elements positioned at their bounding-box coordinates.
///
/// Returns `String` (not `dyn Write`) so STORY-047 can serialize it as JSON for
/// WebSocket push without an extra allocation step. (Previous Story Intelligence)
///
/// # Accessibility invariants (BC-4.03.003)
///
/// - Every non-decorative image/chart/diagram has a non-empty `alt` (img) or
///   `<title>` (svg) attribute.
/// - Every decorative element has `alt="" role="presentation"`.
/// - Heading hierarchy starts at `<h1>` for the slide title — no skipped levels
///   (F-004 / BC-4.03.003 postcondition 7 / axe-core heading-order).
/// - No `<canvas>` elements in the output (BC-4.03.003 invariant 2).
///
/// # Architecture (F-001 / AC-006 / ADR-008)
///
/// The slide is rendered as an `<svg>` canvas within `<article>`. Text is in
/// `<foreignObject>` to preserve semantic HTML headings within the SVG.
/// Images are `<image>` elements (never bare `<img>` — F-002).
///
/// # Security (AC-010 / CWE-601)
///
/// All inline `Link`/`Xref` nodes are validated via
/// [`crate::exporter::is_safe_link_scheme`] before becoming `href` attributes.
#[must_use]
pub fn render_slide_to_html(slide: &LaidOutSlide, brand: &Brand) -> String {
    let _ = brand; // Brand used by future template-driven color/font injection.

    // F-004 (BC-4.03.003 PC-7): Compute heading state before rendering.
    // If the slide has no Title frame, a Subtitle frame becomes h1 (not h2).
    let has_title_frame = slide
        .frames
        .iter()
        .any(|f| matches!(&f.content, FrameContent::Title(_)));

    let slide_index = slide.source_index + 1;
    let canvas_w = CANVAS_WIDTH_EMU.0;
    let canvas_h = CANVAS_HEIGHT_EMU.0;

    // Build SVG content (all frames positioned within the canvas).
    let mut svg_content = String::new();
    let mut h1_emitted = false;

    for frame in &slide.frames {
        let heading_level = match &frame.content {
            FrameContent::Subtitle(_) if !has_title_frame && !h1_emitted => {
                h1_emitted = true;
                1 // Promote to h1 (F-004)
            },
            FrameContent::Subtitle(_) => 2,
            FrameContent::Title(_) => {
                h1_emitted = true;
                1
            },
            _ => 1,
        };
        let frame_svg = render_frame_as_svg_content(frame, heading_level);
        if !frame_svg.is_empty() {
            svg_content.push_str(&frame_svg);
            svg_content.push('\n');
        }
    }

    // Use slide.html.jinja template pattern via inline construction.
    // The template defines: <article><svg viewBox="..."><title>...</title>{content}</svg></article>
    // We embed via include_str! in exporter.rs; here we produce the article fragment directly.
    // The slide title is derived from the first Title frame (if any).
    let slide_title = slide
        .frames
        .iter()
        .find_map(|f| {
            if let FrameContent::Title(t) = &f.content {
                Some(t.as_ref())
            } else {
                None
            }
        })
        .unwrap_or("Slide");
    let escaped_title = html_escape::encode_double_quoted_attribute(slide_title);
    let escaped_title_text = html_escape::encode_text(slide_title);

    format!(
        r#"<article id="slide-{slide_index}" aria-label="Slide {slide_index}">
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {canvas_w} {canvas_h}" width="{canvas_w}" height="{canvas_h}" role="img" aria-label="{escaped_title}">
<title>{escaped_title_text}</title>
{svg_content}</svg>
</article>"#
    )
}

/// Render a single [`Frame`] content to an HTML fragment string.
///
/// Dispatches on the [`slideforge_layout::FrameContent`] variant to produce
/// the appropriate HTML element:
///
/// - `Title` → `<h1>` element
/// - `Subtitle` → `<h2>` element
/// - `Body` → `<ul>` / `<ol>` list
/// - `Image { alt, .. }` → `<img>` with alt attribute (or `alt="" role="presentation"`)
/// - `Chart { alt, .. }` → `<svg role="img"><title>alt</title>...</svg>` via [`render_svg_chart`]
/// - `Diagram { svg, alt }` → `<svg role="img"><title>alt</title>...</svg>` via [`render_svg_chart`]
/// - `Shape(_)` → `<svg>` element
/// - `TextRun(_)` → inline HTML
/// - `ColorBar { .. }` → `<div>` with width style representing fill percentage
/// - `ErrorSlidePlaceholder { .. }` → gray error-state slide SVG
/// - `Empty` → empty string
///
/// # Accessibility (AC-003/AC-004/AC-005)
///
/// Accessibility attributes are injected via `quick-xml` XML manipulation for
/// SVG-bearing variants. usvg is used for geometry/normalization only.
#[must_use]
pub fn render_element_to_html(frame: &Frame) -> String {
    match &frame.content {
        FrameContent::Title(text) => {
            format!("<h1>{}</h1>", html_escape::encode_text(text))
        },
        FrameContent::Subtitle(text) => {
            format!("<h2>{}</h2>", html_escape::encode_text(text))
        },
        FrameContent::Body(blocks) => {
            // F-003: render each ContentBlock variant properly via render_content_block.
            // MUST NOT use format!("{block:?}") — Rust debug output is forbidden in
            // production rendering.
            let mut body_html = String::new();
            for block in blocks {
                body_html.push_str(&render_content_block(block));
                body_html.push('\n');
            }
            body_html
        },
        FrameContent::Image { alt } => render_image(alt),
        FrameContent::Chart { alt } => {
            // Generate a placeholder SVG for the chart.
            // In a full implementation this would be the rendered chart SVG.
            // For now produce the minimal SVG needed for accessibility.
            let alt_text = alt_text_str(alt);
            let placeholder_svg =
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"></svg>"#;
            render_svg_chart(placeholder_svg, alt_text)
        },
        FrameContent::Diagram { svg, alt } => {
            let alt_text = alt_text_str(alt);
            render_svg_chart(svg.as_str(), alt_text)
        },
        FrameContent::Shape(shape_frame) => {
            let alt_text = alt_text_str(&shape_frame.alt);
            // Render shape as an SVG rect for accessibility.
            let placeholder_svg =
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"></svg>"#;
            render_svg_chart(placeholder_svg, alt_text)
        },
        FrameContent::TextRun(nodes) => {
            format!("<p>{}</p>", render_inline_nodes(nodes))
        },
        FrameContent::ColorBar {
            filled_width_emu,
            total_width_emu,
            percent,
            color,
        } => {
            // Render a progress bar as a <div> with inline CSS width.
            let pct = if total_width_emu.0 > 0 {
                // Use the canonical percent field (no re-derivation — OBS-P6-002).
                *percent
            } else {
                0
            };
            let _ = filled_width_emu; // carried for exporter convenience
            let _ = total_width_emu;
            let color_hex = format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b);
            format!(
                r#"<div role="progressbar" aria-valuenow="{pct}" aria-valuemin="0" aria-valuemax="100" style="width:{pct}%;background-color:{color_hex}"></div>"#
            )
        },
        FrameContent::Empty => String::new(),
        FrameContent::ErrorSlidePlaceholder {
            svg,
            slide_title,
            error_code,
            message,
        } => {
            // Render error-state SVG with accessible description.
            let alt_text = format!("Error on slide '{slide_title}': [{error_code}] {message}");
            render_svg_chart(svg, &alt_text)
        },
    }
}

/// Render an image element within an SVG canvas.
///
/// F-002 (story forbidden dependency): SVG is ALWAYS embedded as `<svg>`, never
/// `<img>`. Image frames render as an accessible `<svg>` placeholder with the
/// alt text as the `<title>` element.
///
/// - `AltText::Provided(text)` → `<svg role="img"><title>text</title>...</svg>`
/// - `AltText::Decorative` → `<svg role="presentation" aria-hidden="true">...</svg>`
/// - `AltText::Unspecified` → treated as decorative with a `tracing::warn!`
fn render_image(alt: &AltText) -> String {
    match alt {
        AltText::Provided(text) => {
            let safe_alt = html_escape::encode_double_quoted_attribute(text);
            let escaped_text = html_escape::encode_text(text);
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" role="img" aria-label="{safe_alt}"><title>{escaped_text}</title></svg>"#
            )
        },
        AltText::Decorative => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" role="presentation" aria-hidden="true"></svg>"#
                .to_owned()
        },
        AltText::Unspecified => {
            // Treat unspecified as decorative — validator should have caught this.
            tracing::warn!(
                "render_image: AltText::Unspecified encountered; rendering as decorative"
            );
            r#"<svg xmlns="http://www.w3.org/2000/svg" role="presentation" aria-hidden="true"></svg>"#
                .to_owned()
        },
    }
}

/// Extract alt text string from [`AltText`].
fn alt_text_str(alt: &AltText) -> &str {
    match alt {
        AltText::Provided(text) => text.as_ref(),
        AltText::Decorative | AltText::Unspecified => "",
    }
}

/// Returns `true` if the given SVG element local name is on the SVG blocklist.
///
/// Blocked elements are those that can execute code in an HTML context:
/// - `script` — executes JavaScript
/// - `foreignObject` — embeds arbitrary HTML/script content
///
/// F-005 (XSS): these are stripped when inline SVG is embedded in HTML output.
fn is_blocked_svg_element(local_name: &[u8]) -> bool {
    matches!(local_name, b"script" | b"foreignObject")
}

/// Returns `true` if the given attribute key is safe to emit in inline SVG.
///
/// Strips:
/// - Event handler attributes (`on*` — e.g., `onload`, `onclick`, `onmouseover`)
/// - `href` / `xlink:href` with `javascript:` scheme (checked separately at value level)
///
/// F-005 (XSS): event handlers execute JavaScript when inline SVG is parsed by browsers.
fn is_safe_svg_attribute_key(key: &str) -> bool {
    // Block all on* event handler attributes (case-insensitive).
    !key.to_ascii_lowercase().starts_with("on")
}

/// Returns `true` if the given attribute value is safe for the given key.
///
/// For `href` and `xlink:href`, rejects values beginning with `javascript:` (case-insensitive).
/// F-005 (XSS): `javascript:` href in SVG `<a>` or `<use>` executes in browser context.
fn is_safe_svg_attribute_value(key: &str, value: &[u8]) -> bool {
    let key_lower = key.to_ascii_lowercase();
    if key_lower == "href" || key_lower == "xlink:href" {
        let val_str = std::str::from_utf8(value).unwrap_or("");
        return !val_str
            .trim()
            .to_ascii_lowercase()
            .starts_with("javascript:");
    }
    true
}

/// Inject `role=\"img\"` and `<title>alt text</title>` into an SVG string.
///
/// This function performs raw XML manipulation via `quick-xml` to:
/// 1. Parse the SVG string.
/// 2. **Sanitize:** strip `<script>`, `<foreignObject>`, event-handler attributes
///    (`on*`), and `javascript:` hrefs (F-005 / XSS prevention).
/// 3. Inject `role="img"` on the outer `<svg>` element.
/// 4. Prepend a `<title>alt_text</title>` child as the first child of `<svg>`.
/// 5. Mark all inner `<svg>` elements (nested SVGs) with `aria-hidden="true"`.
/// 6. Return the modified SVG string.
///
/// ## Why NOT the usvg tree API (export-architecture v1.2 / AC-003/AC-005)
///
/// usvg 0.47.0 strips all non-presentation attributes during SVG tree
/// processing, including `role`, `aria-*`, and `<title>`. Using the usvg tree
/// API for accessibility injection would silently discard the injected
/// attributes. This function operates on the raw SVG string after any usvg
/// processing is complete.
///
/// ## SVG-within-HTML semantics (BC-4.03.003 invariant 2)
///
/// The outer `<svg>` element receives `role="img"` and a `<title>` child, which
/// exposes the accessible name to assistive technologies. Inner SVG elements
/// (chart axes, diagram sub-groups) receive `aria-hidden="true"` so they do not
/// pollute the accessibility tree (EC-002 / AC-005).
///
/// ## Security (F-005 / XSS)
///
/// Inline SVG executes script in HTML context. This function strips all elements
/// and attributes that can execute code before the SVG is placed in the HTML output.
///
/// # Returns
///
/// A `String` containing the modified SVG markup. On parse error (malformed
/// SVG input), returns the original `svg_str` unchanged and emits a
/// `tracing::warn!`.
// The SVG sanitization + accessibility injection loop is necessarily long due to the
// number of event variants × element types it handles. Splitting it would fragment
// the control flow and harm correctness auditing. Justified exception to too_many_lines.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn render_svg_chart(svg_str: &str, alt_text: &str) -> String {
    use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
    use quick_xml::{Reader, Writer};

    let mut reader = Reader::from_str(svg_str);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Vec::new());
    let mut depth: u32 = 0;
    let mut outer_svg_done = false;
    // F-005: track depth inside blocked elements (script/foreignObject).
    // When blocked_depth > 0, ALL events are skipped until the matching end tag.
    let mut blocked_depth: u32 = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "render_svg_chart: failed to parse SVG input — returning original string unchanged"
                );
                return svg_str.to_owned();
            },
            Ok(Event::Start(elem)) => {
                let local_name = elem.name().local_name();

                // F-005: if we are inside a blocked element, skip all events.
                if blocked_depth > 0 {
                    if is_blocked_svg_element(local_name.as_ref()) {
                        blocked_depth += 1;
                    }
                    continue;
                }

                // F-005: if this element starts a blocked subtree, enter blocked mode.
                if is_blocked_svg_element(local_name.as_ref()) {
                    blocked_depth += 1;
                    tracing::warn!(
                        element = %String::from_utf8_lossy(local_name.as_ref()),
                        "render_svg_chart: F-005 blocked element stripped from SVG"
                    );
                    continue;
                }

                let is_svg = local_name.as_ref() == b"svg";

                if is_svg && !outer_svg_done {
                    // Outer <svg>: inject role="img", sanitize + copy existing attrs.
                    outer_svg_done = true;
                    let mut new_elem = BytesStart::new("svg");
                    // F-010: skip non-UTF-8 keys; F-005: skip event handlers + drop role.
                    for attr in elem.attributes().flatten() {
                        match std::str::from_utf8(attr.key.as_ref()) {
                            Ok("role") => {}, // drop — we inject below
                            Ok(key) if !is_safe_svg_attribute_key(key) => {
                                tracing::warn!(
                                    key = %key,
                                    "render_svg_chart: F-005 event-handler attribute stripped \
                                     from outer <svg>"
                                );
                            },
                            Ok(key) => {
                                if is_safe_svg_attribute_value(key, attr.value.as_ref()) {
                                    new_elem.push_attribute(attr);
                                } else {
                                    tracing::warn!(
                                        key = %key,
                                        "render_svg_chart: F-005 unsafe attribute value stripped \
                                         from outer <svg>"
                                    );
                                }
                            },
                            Err(_) => {
                                tracing::warn!(
                                    "render_svg_chart: non-UTF-8 attribute key skipped \
                                     on outer <svg>"
                                );
                            },
                        }
                    }
                    // Inject role="img".
                    new_elem.push_attribute(("role", "img"));

                    if let Err(e) = writer.write_event(Event::Start(new_elem)) {
                        tracing::warn!(error = %e, "render_svg_chart: write error on outer svg start");
                        return svg_str.to_owned();
                    }

                    // Inject <title>alt_text</title> as first child.
                    let title_start = BytesStart::new("title");
                    if let Err(e) = writer.write_event(Event::Start(title_start)) {
                        tracing::warn!(error = %e, "render_svg_chart: write error on title start");
                        return svg_str.to_owned();
                    }
                    let escaped_alt = html_escape::encode_text(alt_text);
                    if let Err(e) = writer.write_event(Event::Text(BytesText::new(&escaped_alt))) {
                        tracing::warn!(error = %e, "render_svg_chart: write error on title text");
                        return svg_str.to_owned();
                    }
                    let title_end = BytesEnd::new("title");
                    if let Err(e) = writer.write_event(Event::End(title_end)) {
                        tracing::warn!(error = %e, "render_svg_chart: write error on title end");
                        return svg_str.to_owned();
                    }

                    depth = 1;
                } else if is_svg && depth > 0 {
                    // Inner <svg>: inject aria-hidden="true", sanitize + copy attrs.
                    let mut new_elem = BytesStart::new("svg");
                    // F-005 + F-010: sanitize attributes.
                    for attr in elem.attributes().flatten() {
                        match std::str::from_utf8(attr.key.as_ref()) {
                            Ok("aria-hidden") => {}, // drop — we inject below
                            Ok(key) if !is_safe_svg_attribute_key(key) => {
                                tracing::warn!(
                                    key = %key,
                                    "render_svg_chart: F-005 event-handler attribute stripped \
                                     from inner <svg>"
                                );
                            },
                            Ok(key) => {
                                if is_safe_svg_attribute_value(key, attr.value.as_ref()) {
                                    new_elem.push_attribute(attr);
                                } else {
                                    tracing::warn!(
                                        key = %key,
                                        "render_svg_chart: F-005 unsafe attribute value stripped \
                                         from inner <svg>"
                                    );
                                }
                            },
                            Err(_) => {
                                tracing::warn!(
                                    "render_svg_chart: non-UTF-8 attribute key skipped \
                                     on inner <svg>"
                                );
                            },
                        }
                    }
                    new_elem.push_attribute(("aria-hidden", "true"));

                    if let Err(e) = writer.write_event(Event::Start(new_elem)) {
                        tracing::warn!(error = %e, "render_svg_chart: write error on inner svg start");
                        return svg_str.to_owned();
                    }
                    depth += 1;
                } else {
                    // Other element: sanitize attributes (F-005), then emit.
                    let elem_name_bytes = elem.name();
                    let elem_name_str =
                        std::str::from_utf8(elem_name_bytes.as_ref()).unwrap_or("element");
                    let mut new_elem = BytesStart::new(elem_name_str);
                    for attr in elem.attributes().flatten() {
                        match std::str::from_utf8(attr.key.as_ref()) {
                            Ok(key) if !is_safe_svg_attribute_key(key) => {
                                tracing::warn!(
                                    key = %key,
                                    "render_svg_chart: F-005 event-handler attribute stripped"
                                );
                            },
                            Ok(key) => {
                                if is_safe_svg_attribute_value(key, attr.value.as_ref()) {
                                    new_elem.push_attribute(attr);
                                } else {
                                    tracing::warn!(
                                        key = %key,
                                        "render_svg_chart: F-005 unsafe attribute value stripped"
                                    );
                                }
                            },
                            Err(_) => {}, // skip non-UTF-8 key
                        }
                    }
                    if depth > 0 {
                        depth += 1;
                    }
                    if let Err(e) = writer.write_event(Event::Start(new_elem)) {
                        tracing::warn!(error = %e, "render_svg_chart: write error on elem start");
                        return svg_str.to_owned();
                    }
                }
            },
            Ok(Event::Empty(elem)) => {
                let local_name = elem.name().local_name();

                // F-005: if inside a blocked subtree, skip empty elements too.
                if blocked_depth > 0 {
                    continue;
                }

                // F-005: self-closing blocked elements (e.g., <script/>) — skip.
                if is_blocked_svg_element(local_name.as_ref()) {
                    tracing::warn!(
                        element = %String::from_utf8_lossy(local_name.as_ref()),
                        "render_svg_chart: F-005 self-closing blocked element stripped"
                    );
                    continue;
                }

                let is_svg = local_name.as_ref() == b"svg";

                if is_svg && !outer_svg_done {
                    // Outer self-closing <svg/>: inject role="img" and a <title>.
                    outer_svg_done = true;
                    let mut new_elem = BytesStart::new("svg");
                    // F-005 + F-010: sanitize attributes.
                    for attr in elem.attributes().flatten() {
                        match std::str::from_utf8(attr.key.as_ref()) {
                            Ok("role") => {}, // drop — we inject below
                            Ok(key) if !is_safe_svg_attribute_key(key) => {
                                tracing::warn!(
                                    key = %key,
                                    "render_svg_chart: F-005 event-handler stripped from \
                                     self-closing outer <svg/>"
                                );
                            },
                            Ok(key) => {
                                if is_safe_svg_attribute_value(key, attr.value.as_ref()) {
                                    new_elem.push_attribute(attr);
                                } else {
                                    tracing::warn!(
                                        key = %key,
                                        "render_svg_chart: F-005 unsafe value stripped from \
                                         self-closing outer <svg/>"
                                    );
                                }
                            },
                            Err(_) => {
                                tracing::warn!(
                                    "render_svg_chart: non-UTF-8 attribute key skipped \
                                     on self-closing outer <svg/>"
                                );
                            },
                        }
                    }
                    new_elem.push_attribute(("role", "img"));

                    // Convert to a Start/End pair so we can insert <title> inside.
                    if let Err(e) = writer.write_event(Event::Start(new_elem)) {
                        tracing::warn!(error = %e, "render_svg_chart: write error on self-closing outer svg");
                        return svg_str.to_owned();
                    }
                    let title_start = BytesStart::new("title");
                    if let Err(e) = writer.write_event(Event::Start(title_start)) {
                        tracing::warn!(error = %e, "render_svg_chart: write error");
                        return svg_str.to_owned();
                    }
                    let escaped_alt = html_escape::encode_text(alt_text);
                    if let Err(e) = writer.write_event(Event::Text(BytesText::new(&escaped_alt))) {
                        tracing::warn!(error = %e, "render_svg_chart: write error");
                        return svg_str.to_owned();
                    }
                    let title_end = BytesEnd::new("title");
                    if let Err(e) = writer.write_event(Event::End(title_end)) {
                        tracing::warn!(error = %e, "render_svg_chart: write error");
                        return svg_str.to_owned();
                    }
                    let svg_end = BytesEnd::new("svg");
                    if let Err(e) = writer.write_event(Event::End(svg_end)) {
                        tracing::warn!(error = %e, "render_svg_chart: write error");
                        return svg_str.to_owned();
                    }
                } else if is_svg && depth > 0 {
                    // F-007 (EC-002): inner self-closing <svg/> at depth > 0 must
                    // receive aria-hidden="true" (same as inner Start <svg>).
                    let mut new_elem = BytesStart::new("svg");
                    // F-005 + F-010: sanitize attributes.
                    for attr in elem.attributes().flatten() {
                        match std::str::from_utf8(attr.key.as_ref()) {
                            Ok("aria-hidden") => {}, // drop — we inject below
                            Ok(key) if !is_safe_svg_attribute_key(key) => {
                                tracing::warn!(
                                    key = %key,
                                    "render_svg_chart: F-005 event-handler stripped from \
                                     self-closing inner <svg/>"
                                );
                            },
                            Ok(key) => {
                                if is_safe_svg_attribute_value(key, attr.value.as_ref()) {
                                    new_elem.push_attribute(attr);
                                } else {
                                    tracing::warn!(
                                        key = %key,
                                        "render_svg_chart: F-005 unsafe value stripped from \
                                         self-closing inner <svg/>"
                                    );
                                }
                            },
                            Err(_) => {
                                tracing::warn!(
                                    "render_svg_chart: non-UTF-8 attribute key skipped \
                                     on self-closing inner <svg/>"
                                );
                            },
                        }
                    }
                    new_elem.push_attribute(("aria-hidden", "true"));
                    // Self-closing inner <svg/> is emitted as Empty event.
                    if let Err(e) = writer.write_event(Event::Empty(new_elem)) {
                        tracing::warn!(error = %e, "render_svg_chart: write error on inner self-closing svg");
                        return svg_str.to_owned();
                    }
                } else {
                    // Other self-closing element: sanitize attributes (F-005).
                    let elem_name_bytes = elem.name();
                    let elem_name_str =
                        std::str::from_utf8(elem_name_bytes.as_ref()).unwrap_or("element");
                    let mut new_elem = BytesStart::new(elem_name_str);
                    for attr in elem.attributes().flatten() {
                        match std::str::from_utf8(attr.key.as_ref()) {
                            Ok(key) if !is_safe_svg_attribute_key(key) => {
                                tracing::warn!(
                                    key = %key,
                                    "render_svg_chart: F-005 event-handler stripped from empty elem"
                                );
                            },
                            Ok(key) => {
                                if is_safe_svg_attribute_value(key, attr.value.as_ref()) {
                                    new_elem.push_attribute(attr);
                                } else {
                                    tracing::warn!(
                                        key = %key,
                                        "render_svg_chart: F-005 unsafe value stripped from \
                                         empty elem"
                                    );
                                }
                            },
                            Err(_) => {}, // skip non-UTF-8 key
                        }
                    }
                    if let Err(e) = writer.write_event(Event::Empty(new_elem)) {
                        tracing::warn!(error = %e, "render_svg_chart: write error on empty elem");
                        return svg_str.to_owned();
                    }
                }
            },
            Ok(Event::End(elem)) => {
                // F-005: if inside a blocked subtree, handle end tag.
                if blocked_depth > 0 {
                    if is_blocked_svg_element(elem.name().local_name().as_ref()) {
                        blocked_depth = blocked_depth.saturating_sub(1);
                    }
                    continue;
                }
                if depth > 0 {
                    depth = depth.saturating_sub(1);
                }
                if let Err(e) = writer.write_event(Event::End(elem)) {
                    tracing::warn!(error = %e, "render_svg_chart: write error on end elem");
                    return svg_str.to_owned();
                }
            },
            Ok(other) => {
                // F-005: skip text/cdata inside blocked elements.
                if blocked_depth > 0 {
                    continue;
                }
                if let Err(e) = writer.write_event(other) {
                    tracing::warn!(error = %e, "render_svg_chart: write error on other event");
                    return svg_str.to_owned();
                }
            },
        }
    }

    // If we never found an outer <svg>, the input was not SVG at all.
    if !outer_svg_done {
        tracing::warn!(
            "render_svg_chart: no <svg> element found in input — \
             returning original string unchanged (parse error)"
        );
        return svg_str.to_owned();
    }

    match String::from_utf8(writer.into_inner()) {
        Ok(result) => result,
        Err(e) => {
            tracing::warn!(error = %e, "render_svg_chart: UTF-8 encoding error — returning original");
            svg_str.to_owned()
        },
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STORY-046 Red Gate Tests — render.rs
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::doc_markdown, // test doc comments use fn names and HTML that trigger doc_markdown
    non_snake_case
)]
mod tests {
    use std::sync::Arc;

    use slideforge_layout::{BoundingBox, Frame, FrameContent, LaidOutSlide};
    use slideforge_types::AltText;
    use slideforge_types::{
        Brand, BrandFonts, BrandPalette, Emu, NormalizedDiagramSvg, SourceSpan,
    };

    use super::{render_element_to_html, render_slide_to_html, render_svg_chart};

    // ─────────────────────────────────────────────────────────────────────────
    // Fixtures
    // ─────────────────────────────────────────────────────────────────────────

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
                heading: Arc::from("Calibri"),
                body: Arc::from("Calibri"),
                mono: Arc::from("Courier New"),
            },
            layouts: vec![],
            span: SourceSpan::default(),
        }
    }

    fn make_bbox_full() -> BoundingBox {
        BoundingBox {
            x: Emu(0),
            y: Emu(0),
            width: Emu(9_144_000),
            height: Emu(5_143_500),
        }
    }

    fn make_slide(frames: Vec<Frame>) -> LaidOutSlide {
        LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("content"),
            frames,
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-003 / render_svg_chart: role="img" and <title> injection
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 postcondition 6 — `render_svg_chart` injects `role="img"` on
    /// the outer `<svg>` element.
    #[test]
    fn test_BC_4_03_003_render_svg_chart_injects_role_img() {
        let svg_in = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect x="0" y="0" width="100" height="100"/></svg>"#;
        let result = render_svg_chart(svg_in, "Revenue by quarter");
        assert!(
            result.contains(r#"role="img""#),
            "render_svg_chart must inject role=\"img\" on outer <svg>; got: {result}"
        );
    }

    /// BC-4.03.003 postcondition 6 — `render_svg_chart` prepends `<title>alt_text</title>`
    /// as the first child of the outer `<svg>`.
    #[test]
    fn test_BC_4_03_003_render_svg_chart_injects_title_element() {
        let svg_in = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect x="0" y="0" width="100" height="100"/></svg>"#;
        let result = render_svg_chart(svg_in, "Revenue by quarter");
        assert!(
            result.contains("<title>Revenue by quarter</title>"),
            "render_svg_chart must inject <title>Revenue by quarter</title>; got: {result}"
        );
    }

    /// BC-4.03.003 test vector — chart alt text "Revenue by quarter" →
    /// `<title>Revenue by quarter</title>` (canonical TV from BC).
    #[test]
    fn test_BC_4_03_003_render_svg_chart_canonical_tv_revenue_by_quarter() {
        let svg_in = r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"></svg>"#;
        let result = render_svg_chart(svg_in, "Revenue by quarter");
        // Canonical test vector from BC-4.03.003
        assert!(
            result.contains("<title>Revenue by quarter</title>"),
            "BC-4.03.003 canonical TV: chart with alt 'Revenue by quarter' must produce \
             <title>Revenue by quarter</title>; got: {result}"
        );
        assert!(
            result.contains(r#"role="img""#),
            "BC-4.03.003 canonical TV: outer <svg> must have role=\"img\"; got: {result}"
        );
    }

    /// BC-4.03.003 EC-002 — nested SVG elements inside the chart get
    /// `aria-hidden="true"`.
    #[test]
    fn test_BC_4_03_003_ec_002_nested_svg_gets_aria_hidden() {
        let svg_in = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><svg width="50" height="50"><rect/></svg></svg>"#;
        let result = render_svg_chart(svg_in, "Complex chart");

        let _doc = scraper::Html::parse_document(&result);
        // The outer svg gets role=img; the inner svg gets aria-hidden=true.
        // Parse the SVG as HTML (scraper understands both).
        assert!(
            result.contains(r#"aria-hidden="true""#),
            "EC-002: nested <svg> inside chart must have aria-hidden=\"true\"; got: {result}"
        );
    }

    /// BC-4.03.003 — `render_svg_chart` on malformed SVG returns original string
    /// unchanged (no panic) and emits a `tracing::warn!`.
    #[tracing_test::traced_test]
    #[test]
    fn test_BC_4_03_003_render_svg_chart_malformed_svg_returns_unchanged() {
        let bad_svg = "this is not svg at all";
        let result = render_svg_chart(bad_svg, "alt");
        // Must return the original unchanged, not panic
        assert_eq!(
            result, bad_svg,
            "render_svg_chart must return the original string unchanged on parse error"
        );
        // Must emit a tracing::warn!
        assert!(
            logs_contain("warn") || logs_contain("malformed") || logs_contain("parse"),
            "render_svg_chart must emit a tracing::warn! on SVG parse error"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-003 / render_element_to_html: non-decorative image alt
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 postcondition 4 — `render_element_to_html` for a non-decorative
    /// image frame produces a non-empty accessible name.
    ///
    /// F-002: Images are rendered as SVG (not bare <img>). The accessible name is
    /// expressed via `role="img"` and `<title>alt text</title>` on the SVG element.
    #[test]
    fn test_BC_4_03_003_render_element_non_decorative_image_has_non_empty_alt() {
        let frame = Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Image {
                alt: AltText::Provided(Arc::from("A bar chart showing quarterly revenue")),
            },
            text_flow: None,
            region_role: None,
        };
        let result = render_element_to_html(&frame);
        // F-002: image is now SVG, not <img>. Must have role="img" on SVG element.
        assert!(
            !result.contains("<img "),
            "F-002: image must not be a bare <img>; got: {result}"
        );
        assert!(
            result.contains("<svg"),
            "non-decorative image must produce an <svg> element; got: {result}"
        );
        assert!(
            result.contains("A bar chart showing quarterly revenue"),
            "non-decorative image alt text must appear in rendered output; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-004 / render_element_to_html: decorative image
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 postcondition 5 — `render_element_to_html` for a decorative
    /// image produces `role="presentation"`.
    ///
    /// F-002: Images are rendered as SVG (not bare <img>). Decorative images use
    /// `role="presentation"` and `aria-hidden="true"` on the SVG element.
    #[test]
    fn test_BC_4_03_003_render_element_decorative_image_empty_alt_and_role() {
        let frame = Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Image {
                alt: AltText::Decorative,
            },
            text_flow: None,
            region_role: None,
        };
        let result = render_element_to_html(&frame);
        // F-002: image is now SVG. Check for role="presentation" on SVG element.
        assert!(
            !result.contains("<img "),
            "F-002: decorative image must not be a bare <img>; got: {result}"
        );
        assert!(
            result.contains("<svg"),
            "decorative image must produce an <svg> element; got: {result}"
        );
        assert!(
            result.contains(r#"role="presentation""#),
            "decorative image must produce role=\"presentation\"; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-005 / render_element_to_html: Chart SVG
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 postcondition 6 — `render_element_to_html` for a Chart frame
    /// produces `<svg role="img"><title>alt text</title>`.
    #[test]
    fn test_BC_4_03_003_render_element_chart_svg_has_role_img_and_title() {
        let frame = Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Chart {
                alt: AltText::Provided(Arc::from("Revenue by quarter")),
            },
            text_flow: None,
            region_role: None,
        };
        let result = render_element_to_html(&frame);
        assert!(
            result.contains(r#"role="img""#),
            "Chart frame must produce <svg role=\"img\">; got: {result}"
        );
        assert!(
            result.contains("<title>Revenue by quarter</title>"),
            "Chart frame must produce <title>Revenue by quarter</title>; got: {result}"
        );
    }

    /// BC-4.03.003 postcondition 6 — `render_element_to_html` for a Diagram frame
    /// produces `<svg role="img"><title>alt text</title>`.
    #[test]
    fn test_BC_4_03_003_render_element_diagram_svg_has_role_img_and_title() {
        let svg_str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"><rect x="0" y="0" width="400" height="300"/></svg>"#;
        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(svg_str));
        let frame = Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Diagram {
                svg: normalized,
                alt: AltText::Provided(Arc::from("Mermaid flowchart")),
            },
            text_flow: None,
            region_role: None,
        };
        let result = render_element_to_html(&frame);
        assert!(
            result.contains(r#"role="img""#),
            "Diagram frame must produce <svg role=\"img\">; got: {result}"
        );
        assert!(
            result.contains("<title>Mermaid flowchart</title>"),
            "Diagram frame must produce <title>Mermaid flowchart</title>; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-006 / render_slide_to_html: No <canvas>
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 invariant 2 — `render_slide_to_html` produces no `<canvas>`
    /// elements.
    #[test]
    fn test_BC_4_03_003_render_slide_to_html_no_canvas_elements() {
        let slide = make_slide(vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Title(Arc::from("Test")),
            text_flow: None,
            region_role: None,
        }]);
        let brand = make_brand();
        let result = render_slide_to_html(&slide, &brand);

        let doc = scraper::Html::parse_document(&result);
        let sel = scraper::Selector::parse("canvas").expect("valid selector");
        assert_eq!(
            doc.select(&sel).count(),
            0,
            "render_slide_to_html must not produce <canvas> elements; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-005 / render_element_to_html: inner SVG gets aria-hidden
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 EC-002 — when a Diagram frame contains nested SVG elements,
    /// inner SVG elements must have `aria-hidden="true"`.
    #[test]
    fn test_BC_4_03_003_ec_002_diagram_inner_svg_gets_aria_hidden() {
        // SVG with a nested <svg> element (complex chart structure)
        let nested_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"><svg width="200" height="150"><rect/></svg></svg>"#;
        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(nested_svg));
        let frame = Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Diagram {
                svg: normalized,
                alt: AltText::Provided(Arc::from("Complex nested diagram")),
            },
            text_flow: None,
            region_role: None,
        };
        let result = render_element_to_html(&frame);
        // EC-002: inner svg elements must have aria-hidden="true"
        assert!(
            result.contains(r#"aria-hidden="true""#),
            "EC-002: inner <svg> elements must have aria-hidden=\"true\"; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Snapshot tests (insta) — per Test Strategy
    // ─────────────────────────────────────────────────────────────────────────

    /// Snapshot test: title slide renders to expected HTML structure.
    /// Per Test Strategy: "insta snapshots for rendered HTML of reference slide types".
    /// Snapshot will be empty/unreviewed on first run — fails until implementation
    /// exists (Red Gate).
    #[test]
    fn test_BC_4_03_003_snapshot_title_slide_html() {
        let slide = make_slide(vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Title(Arc::from("Hello World")),
            text_flow: None,
            region_role: None,
        }]);
        let brand = make_brand();
        let result = render_slide_to_html(&slide, &brand);
        insta::assert_snapshot!("title_slide_html", result);
    }

    /// Snapshot test: content slide with subtitle renders correctly.
    #[test]
    fn test_BC_4_03_003_snapshot_content_slide_html() {
        let slide = make_slide(vec![
            Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(1_000_000),
                },
                content: FrameContent::Title(Arc::from("Content Slide")),
                text_flow: None,
                region_role: None,
            },
            Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(1_000_000),
                    width: Emu(9_144_000),
                    height: Emu(4_143_500),
                },
                content: FrameContent::Subtitle(Arc::from("A subtitle here")),
                text_flow: None,
                region_role: None,
            },
        ]);
        let brand = make_brand();
        let result = render_slide_to_html(&slide, &brand);
        insta::assert_snapshot!("content_slide_html", result);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-001 / F-002 — SVG canvas rendering and image-as-svg
    // ─────────────────────────────────────────────────────────────────────────

    /// F-001 (AC-006 / BC-4.03.003 invariant 2): render_slide_to_html must produce
    /// an outer <svg> canvas containing all frame elements (not bare flow HTML).
    #[test]
    fn test_F001_render_slide_to_html_produces_svg_canvas() {
        let slide = make_slide(vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Title(Arc::from("SVG Canvas Test")),
            text_flow: None,
            region_role: None,
        }]);
        let brand = make_brand();
        let result = render_slide_to_html(&slide, &brand);
        // Must contain an <svg> canvas inside the <article>
        assert!(
            result.contains("<svg"),
            "F-001: render_slide_to_html must produce an <svg> canvas element; got: {result}"
        );
        // Must not contain bare <h1> at article level (h1 goes inside SVG/foreignObject)
        // — actually AC-008 still requires h1 via accessible heading. Per ADR-008
        //   we must have an outer <svg> canvas in the article.
        let doc = scraper::Html::parse_document(&result);
        let sel_article = scraper::Selector::parse("article").expect("valid");
        let sel_svg = scraper::Selector::parse("article svg").expect("valid");
        assert!(
            doc.select(&sel_article).count() > 0,
            "F-001: must have <article> landmark"
        );
        assert!(
            doc.select(&sel_svg).count() > 0,
            "F-001: <article> must contain an <svg> canvas; got: {result}"
        );
    }

    /// F-001: the slide.html.jinja template must be used (SVG has viewBox attribute).
    #[test]
    fn test_F001_svg_canvas_has_view_box() {
        let slide = make_slide(vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Title(Arc::from("ViewBox Test")),
            text_flow: None,
            region_role: None,
        }]);
        let brand = make_brand();
        let result = render_slide_to_html(&slide, &brand);
        assert!(
            result.contains("viewBox"),
            "F-001: SVG canvas must have a viewBox attribute; got: {result}"
        );
    }

    /// F-002 (story forbidden dependency): Image frames must NOT render as bare
    /// `<img>` elements. SVG is always embedded as <svg>, never <img>.
    /// For Image frames, the HTML exporter must emit an SVG <image> element or
    /// otherwise avoid bare <img src="..."> with missing src.
    #[test]
    fn test_F002_image_frame_renders_as_svg_not_bare_img() {
        let frame = Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Image {
                alt: AltText::Provided(Arc::from("A photo")),
            },
            text_flow: None,
            region_role: None,
        };
        let result = render_element_to_html(&frame);
        // The result must NOT be a bare <img> with no src attribute.
        // Per story: "image is embedded as <svg>, never <img>"
        // An <svg> wrapping an <image> element is acceptable.
        // A bare <img alt="..."> with no src is forbidden.
        assert!(
            !result.contains("<img "),
            "F-002: Image frame must not render as bare <img>; must use <svg><image>; got: {result}"
        );
        assert!(
            result.contains("<svg"),
            "F-002: Image frame must render within an <svg> element; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-004 — heading hierarchy: <h2> must not appear without a preceding <h1>
    // ─────────────────────────────────────────────────────────────────────────

    /// F-004: A slide that starts with Subtitle only (no Title frame) must NOT
    /// produce a bare <h2> without a preceding <h1>. The exporter must ensure
    /// heading levels start at h1 (axe-core heading-order rule / BC-4.03.003 PC-7).
    /// Note: headings are inside <foreignObject> within the SVG canvas.
    #[test]
    fn test_F004_subtitle_only_slide_must_not_produce_bare_h2() {
        // A slide with ONLY a Subtitle frame (no Title) — simulates the bug path.
        let slide = make_slide(vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Subtitle(Arc::from("Just a subtitle")),
            text_flow: None,
            region_role: None,
        }]);
        let brand = make_brand();
        let result = render_slide_to_html(&slide, &brand);
        // Headings inside <foreignObject>: check raw HTML for h2 without h1.
        let has_h2 = result.contains("<h2 ") || result.contains("<h2>");
        let has_h1 = result.contains("<h1 ") || result.contains("<h1>");
        if has_h2 {
            assert!(
                has_h1,
                "F-004: heading hierarchy violation — h2 present without h1; \
                 axe-core heading-order would fail; got: {result}"
            );
        }
        // The subtitle-only slide must render as h1 (promoted).
        assert!(
            has_h1,
            "F-004: Subtitle-only slide must produce h1 (promoted, no Title frame); \
             got: {result}"
        );
    }

    /// F-004: A slide with Title then Subtitle must produce h1 before h2 (valid).
    /// Note: headings are inside <foreignObject> within the SVG canvas (F-001 rework).
    #[test]
    fn test_F004_title_then_subtitle_produces_h1_then_h2() {
        let slide = make_slide(vec![
            Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Title(Arc::from("Main Title")),
                text_flow: None,
                region_role: None,
            },
            Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Subtitle(Arc::from("Sub heading")),
                text_flow: None,
                region_role: None,
            },
        ]);
        let brand = make_brand();
        let result = render_slide_to_html(&slide, &brand);
        // Headings are inside <foreignObject>. Check the raw HTML for h1 and h2 tags.
        assert!(
            result.contains("<h1 ") || result.contains("<h1>"),
            "F-004: Title must produce h1 (possibly inside foreignObject); got: {result}"
        );
        assert!(
            result.contains("<h2 ") || result.contains("<h2>"),
            "F-004: Subtitle after Title must produce h2 (possibly inside foreignObject); got: {result}"
        );
        // Verify h1 comes before h2 in document order.
        let h1_pos = result.find("<h1").unwrap_or(usize::MAX);
        let h2_pos = result.find("<h2").unwrap_or(usize::MAX);
        assert!(
            h1_pos < h2_pos,
            "F-004: h1 must appear before h2 in document order; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-003 — FrameContent::Body must render ContentBlock variants properly
    // ─────────────────────────────────────────────────────────────────────────

    /// F-003: Body frame with a Text block must render a <p> element containing
    /// the text content — NOT the Rust Debug representation like
    /// "Text(TextBlock { inlines: [...] })".
    #[test]
    fn test_F003_body_text_block_renders_as_p_not_debug() {
        use slideforge_types::{ContentBlock, InlineNode, SourceSpan, TextBlock, TextTag};
        let frame = Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Body(vec![ContentBlock::Text(TextBlock {
                inlines: vec![InlineNode::Plain(Arc::from("Hello world"))],
                tag: TextTag::Body,
                span: SourceSpan::default(),
            })]),
            text_flow: None,
            region_role: None,
        };
        let result = render_element_to_html(&frame);
        assert!(
            !result.contains("TextBlock"),
            "F-003: Body must NOT render debug output (TextBlock); got: {result}"
        );
        assert!(
            !result.contains("inlines:"),
            "F-003: Body must NOT render debug output (inlines:); got: {result}"
        );
        assert!(
            result.contains("Hello world"),
            "F-003: Body text must appear in rendered output; got: {result}"
        );
        assert!(
            result.contains("<p>"),
            "F-003: Body Text block must render as <p>; got: {result}"
        );
    }

    /// F-003: Body frame with a Bullets block must render a <ul><li> structure.
    #[test]
    fn test_F003_body_bullets_block_renders_as_ul_li() {
        use slideforge_types::{BulletItem, ContentBlock, InlineNode, SourceSpan};
        let frame = Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Body(vec![ContentBlock::Bullets(vec![
                BulletItem {
                    inlines: vec![InlineNode::Plain(Arc::from("First item"))],
                    children: vec![],
                    span: SourceSpan::default(),
                },
                BulletItem {
                    inlines: vec![InlineNode::Plain(Arc::from("Second item"))],
                    children: vec![],
                    span: SourceSpan::default(),
                },
            ])]),
            text_flow: None,
            region_role: None,
        };
        let result = render_element_to_html(&frame);
        assert!(
            !result.contains("BulletItem"),
            "F-003: Bullets must NOT render debug output; got: {result}"
        );
        assert!(
            result.contains("<ul>") || result.contains("<li>"),
            "F-003: Bullets must render as <ul>/<li>; got: {result}"
        );
        assert!(
            result.contains("First item"),
            "F-003: Bullet item text must appear; got: {result}"
        );
        assert!(
            result.contains("Second item"),
            "F-003: Bullet item text must appear; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-005 — SVG sanitization: strip <script>, <foreignObject>, event handlers
    // ─────────────────────────────────────────────────────────────────────────

    /// F-005 (XSS): render_svg_chart must strip <script> elements from input SVG.
    #[test]
    fn test_F005_render_svg_chart_strips_script_elements() {
        let malicious_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><script>alert('xss')</script><rect x="0" y="0" width="100" height="100"/></svg>"#;
        let result = render_svg_chart(malicious_svg, "chart");
        assert!(
            !result.contains("<script>"),
            "F-005: <script> elements must be stripped from SVG; got: {result}"
        );
        assert!(
            !result.contains("alert("),
            "F-005: script content must be stripped; got: {result}"
        );
    }

    /// F-005 (XSS): render_svg_chart must strip <foreignObject> elements.
    #[test]
    fn test_F005_render_svg_chart_strips_foreign_object() {
        let malicious_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><foreignObject width="100" height="100"><div>evil</div></foreignObject></svg>"#;
        let result = render_svg_chart(malicious_svg, "chart");
        assert!(
            !result.contains("foreignObject"),
            "F-005: <foreignObject> must be stripped from SVG; got: {result}"
        );
    }

    /// F-005 (XSS): render_svg_chart must strip event handler attributes (onload=, onclick=).
    #[test]
    fn test_F005_render_svg_chart_strips_event_handlers() {
        let malicious_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" onload="alert(1)"><rect onclick="evil()"/></svg>"#;
        let result = render_svg_chart(malicious_svg, "chart");
        assert!(
            !result.contains("onload="),
            "F-005: onload= event handler must be stripped; got: {result}"
        );
        assert!(
            !result.contains("onclick="),
            "F-005: onclick= event handler must be stripped; got: {result}"
        );
    }

    /// F-005 (XSS): render_svg_chart must strip href="javascript:..." attributes.
    #[test]
    fn test_F005_render_svg_chart_strips_javascript_href() {
        let malicious_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><a href="javascript:alert(1)"><rect/></a></svg>"#;
        let result = render_svg_chart(malicious_svg, "chart");
        assert!(
            !result.contains("javascript:"),
            "F-005: javascript: href must be stripped; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-007 — self-closing nested <svg/> inside chart must get aria-hidden="true"
    // ─────────────────────────────────────────────────────────────────────────

    /// F-007 (EC-002): a self-closing nested `<svg/>` (Event::Empty) inside the
    /// outer `<svg>` must receive `aria-hidden="true"`, not pass through untouched.
    #[test]
    fn test_F007_self_closing_nested_svg_gets_aria_hidden() {
        // The outer <svg> is a Start event; the inner <svg/> is an Empty event.
        let svg_in =
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><svg/></svg>"#;
        let result = render_svg_chart(svg_in, "complex chart");
        // The inner self-closing <svg/> must now have aria-hidden="true".
        // There are two svg elements; the inner one must have aria-hidden.
        assert!(
            result.contains("aria-hidden=\"true\""),
            "F-007: self-closing inner <svg/> must have aria-hidden=\"true\"; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-009 — InlineNode::Xref must route through is_safe_link_scheme
    // ─────────────────────────────────────────────────────────────────────────

    /// F-009: Xref with a safe fragment target (#slide-3) must produce
    /// `<a href="#slide-3">`.
    #[test]
    fn test_F009_xref_safe_fragment_renders_anchor() {
        use super::render_inline_node;
        use slideforge_types::InlineNode;
        let node = InlineNode::Xref(Arc::from("slide-3"));
        let result = render_inline_node(&node);
        assert!(
            result.contains("href=\"#slide-3\""),
            "F-009: Xref to 'slide-3' must render as href=\"#slide-3\"; got: {result}"
        );
    }

    /// F-009: Xref validation — even though Xref always prepends '#', the target
    /// must not start with '//' or '\' after prepending (guards against crafted
    /// targets that could bypass the fragment prefix). A plain text Xref target
    /// such as "slide-3" must pass cleanly.
    #[tracing_test::traced_test]
    #[test]
    fn test_F009_xref_routes_through_allowlist() {
        use super::render_inline_node;
        use slideforge_types::InlineNode;
        // Normal Xref — must produce a valid anchor, no warn emitted.
        let node = InlineNode::Xref(Arc::from("intro"));
        let result = render_inline_node(&node);
        assert!(
            result.contains("href=\"#intro\""),
            "F-009: Xref 'intro' must render as href=\"#intro\""
        );
        assert!(
            !logs_contain("rejected"),
            "F-009: safe Xref must not emit warn"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-010 — non-UTF-8 attribute keys must not silently degrade
    // ─────────────────────────────────────────────────────────────────────────

    /// F-010: render_svg_chart processes SVG with a standard ASCII attribute key —
    /// must NOT use unwrap_or("") for key parsing (covered by code fix; this test
    /// verifies the happy path still works correctly).
    #[test]
    fn test_F010_svg_attr_key_standard_ascii_processed_correctly() {
        let svg_in = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect x="0" y="0" width="100" height="100"/></svg>"#;
        let result = render_svg_chart(svg_in, "test");
        assert!(
            result.contains(r#"role="img""#),
            "F-010: SVG with standard attribute keys must still get role=\"img\" injected"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Snapshot tests (insta) — per Test Strategy
    // ─────────────────────────────────────────────────────────────────────────

    /// Snapshot test: chart slide with SVG role="img" and <title>.
    #[test]
    fn test_BC_4_03_003_snapshot_chart_slide_html() {
        let slide = make_slide(vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Chart {
                alt: AltText::Provided(Arc::from("Revenue by quarter")),
            },
            text_flow: None,
            region_role: None,
        }]);
        let brand = make_brand();
        let result = render_slide_to_html(&slide, &brand);
        insta::assert_snapshot!("chart_slide_html", result);
    }
}
