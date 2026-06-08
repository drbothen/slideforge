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
use slideforge_types::{AltText, Brand, InlineNode};

use crate::exporter::is_safe_link_scheme;

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
            if is_safe_link_scheme(url) {
                let safe_url = html_escape::encode_double_quoted_attribute(url);
                format!(r#"<a href="{safe_url}">{inner}</a>"#)
            } else {
                // Drop href — render as plain span.
                tracing::warn!(
                    rejected_scheme = %extract_scheme(url),
                    url = %url,
                    "AC-010: link URL with disallowed scheme rejected (CWE-601)"
                );
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
            // Xref becomes an anchor to a slide ID.
            let safe_target = html_escape::encode_double_quoted_attribute(target);
            let display = html_escape::encode_text(target);
            format!("<a href=\"#{safe_target}\">{display}</a>")
        },
    }
}

/// Extract the URL scheme (everything before the first `:`), or `"<no-scheme>"`.
fn extract_scheme(url: &str) -> &str {
    url.find(':').map_or("<no-scheme>", |pos| &url[..pos])
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Render a single slide to an HTML fragment string.
///
/// The returned `String` is a self-contained HTML fragment representing one
/// slide: an `<article>` landmark containing frame elements for the slide
/// visuals, with correct ARIA roles, alt attributes, and heading hierarchy.
///
/// Returns `String` (not `dyn Write`) so STORY-047 can serialize it as JSON for
/// WebSocket push without an extra allocation step. (Previous Story Intelligence)
///
/// # Accessibility invariants (BC-4.03.003)
///
/// - Every non-decorative image/chart/diagram has a non-empty `alt` (img) or
///   `<title>` (svg) attribute.
/// - Every decorative element has `alt="" role="presentation"`.
/// - Heading hierarchy starts at `<h1>` for the slide title.
/// - No `<canvas>` elements in the output.
///
/// # Security (AC-010 / CWE-601)
///
/// All inline `Link`/`Xref` nodes are validated via
/// [`crate::exporter::is_safe_link_scheme`] before becoming `href` attributes.
#[must_use]
pub fn render_slide_to_html(slide: &LaidOutSlide, _brand: &Brand) -> String {
    let mut frames_html = String::new();
    for frame in &slide.frames {
        frames_html.push_str(&render_element_to_html(frame));
        frames_html.push('\n');
    }

    let slide_index = slide.source_index + 1;
    format!(
        r#"<article id="slide-{slide_index}" aria-label="Slide {slide_index}">
{frames_html}</article>"#
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
            use std::fmt::Write as _;
            let mut items = String::new();
            for block in blocks {
                // Render each content block as a list item.
                // ContentBlock is from slideforge-types; render text content.
                let block_text = format!("{block:?}");
                let _ = writeln!(items, "<li>{}</li>", html_escape::encode_text(&block_text));
            }
            format!("<ul>\n{items}</ul>")
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

/// Render an image element with correct alt attribute.
///
/// - `AltText::Provided(text)` → `<img alt="text">` (non-empty alt)
/// - `AltText::Decorative` → `<img alt="" role="presentation">`
/// - `AltText::Unspecified` → `<img alt="">` (treated as decorative per WCAG)
fn render_image(alt: &AltText) -> String {
    match alt {
        AltText::Provided(text) => {
            let safe_alt = html_escape::encode_double_quoted_attribute(text);
            format!(r#"<img alt="{safe_alt}">"#)
        },
        AltText::Decorative => r#"<img alt="" role="presentation">"#.to_owned(),
        AltText::Unspecified => {
            // Treat unspecified as decorative — validator should have caught this.
            tracing::warn!(
                "render_image: AltText::Unspecified encountered; rendering as decorative"
            );
            r#"<img alt="" role="presentation">"#.to_owned()
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

/// Inject `role=\"img\"` and `<title>alt text</title>` into an SVG string.
///
/// This function performs raw XML manipulation via `quick-xml` to:
/// 1. Parse the SVG string.
/// 2. Inject `role="img"` on the outer `<svg>` element.
/// 3. Prepend a `<title>alt_text</title>` child as the first child of `<svg>`.
/// 4. Mark all inner `<svg>` elements (nested SVGs) with `aria-hidden="true"`.
/// 5. Return the modified SVG string.
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
/// # Returns
///
/// A `String` containing the modified SVG markup. On parse error (malformed
/// SVG input), returns the original `svg_str` unchanged and emits a
/// `tracing::warn!`.
#[must_use]
pub fn render_svg_chart(svg_str: &str, alt_text: &str) -> String {
    use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
    use quick_xml::{Reader, Writer};

    let mut reader = Reader::from_str(svg_str);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Vec::new());
    let mut depth: u32 = 0;
    let mut outer_svg_done = false;

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
                let is_svg = local_name.as_ref() == b"svg";

                if is_svg && !outer_svg_done {
                    // Outer <svg>: inject role="img", remove existing role if present.
                    outer_svg_done = true;
                    let mut new_elem = BytesStart::new("svg");
                    // Copy existing attributes, except role (we re-inject it).
                    for attr in elem.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        if key != "role" {
                            new_elem.push_attribute(attr);
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
                    // Inner <svg>: inject aria-hidden="true", remove existing aria-hidden.
                    let mut new_elem = BytesStart::new("svg");
                    for attr in elem.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        if key != "aria-hidden" {
                            new_elem.push_attribute(attr);
                        }
                    }
                    new_elem.push_attribute(("aria-hidden", "true"));

                    if let Err(e) = writer.write_event(Event::Start(new_elem)) {
                        tracing::warn!(error = %e, "render_svg_chart: write error on inner svg start");
                        return svg_str.to_owned();
                    }
                    depth += 1;
                } else {
                    if depth > 0 {
                        depth += 1;
                    }
                    if let Err(e) = writer.write_event(Event::Start(elem)) {
                        tracing::warn!(error = %e, "render_svg_chart: write error on elem start");
                        return svg_str.to_owned();
                    }
                }
            },
            Ok(Event::Empty(elem)) => {
                let local_name = elem.name().local_name();
                let is_svg = local_name.as_ref() == b"svg";

                if is_svg && !outer_svg_done {
                    // Outer self-closing <svg/>: inject role="img" and a <title>.
                    outer_svg_done = true;
                    let mut new_elem = BytesStart::new("svg");
                    for attr in elem.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        if key != "role" {
                            new_elem.push_attribute(attr);
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
                } else if let Err(e) = writer.write_event(Event::Empty(elem)) {
                    tracing::warn!(error = %e, "render_svg_chart: write error on empty elem");
                    return svg_str.to_owned();
                }
            },
            Ok(Event::End(elem)) => {
                if depth > 0 {
                    depth = depth.saturating_sub(1);
                }
                if let Err(e) = writer.write_event(Event::End(elem)) {
                    tracing::warn!(error = %e, "render_svg_chart: write error on end elem");
                    return svg_str.to_owned();
                }
            },
            Ok(other) => {
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
    /// image frame produces a non-empty alt attribute.
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
        // Non-decorative image must have a non-empty alt
        let doc = scraper::Html::parse_document(&result);
        let sel_empty = scraper::Selector::parse("img[alt=\"\"]").expect("valid selector");
        assert_eq!(
            doc.select(&sel_empty).count(),
            0,
            "non-decorative image must not produce img[alt=\"\"]; got: {result}"
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
    /// image produces `alt=""` and `role="presentation"`.
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
        let doc = scraper::Html::parse_document(&result);
        let sel_empty_alt = scraper::Selector::parse("img[alt=\"\"]").expect("valid selector");
        assert!(
            doc.select(&sel_empty_alt).count() > 0,
            "decorative image must produce img[alt=\"\"]; got: {result}"
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
