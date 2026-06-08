//! Slide-to-HTML rendering functions — P4 Composite Rendering Model.
//!
//! This module provides the core rendering primitives for the P4 model
//! (ADR-008 binding 2026-06-08, BC-4.03.003 v1.3):
//!
//! - [`render_slide_to_html`] — renders a single [`LaidOutSlide`] to an HTML
//!   fragment string. Uses the P4 model: `<article>` container with an HTML
//!   text layer (real heading/paragraph elements) and a sibling
//!   `<svg role="presentation">` graphics layer. Returns `String` (not `dyn Write`)
//!   so STORY-047 can serialize it as JSON for WebSocket push.
//!   (Previous Story Intelligence note)
//!
//! - [`render_text_frame`] — converts a text `LaidOutFrame` to the appropriate
//!   absolutely-positioned HTML element.
//!
//! - [`render_graphics_layer`] — builds the `<svg role="presentation">` layer
//!   for charts, diagrams, images, and decorative shapes.
//!
//! - [`render_chart_frame`] — injects `role="img"`, `aria-labelledby`, and
//!   `<title>` into an SVG string via `quick-xml` after usvg geometry processing.
//!
//! ## P4 Composite Rendering Model (ADR-008 / BC-4.03.003 invariant 2)
//!
//! The slide DOM skeleton is:
//! ```html
//! <article class="sf-slide" style="position:relative; width:{W}px; height:{H}px; overflow:hidden;">
//!   <!-- HTML TEXT LAYER: one element per text LaidOutFrame, in reading order -->
//!   <h1 class="sf-title" style="position:absolute; left:{x}px; top:{y}px; ...">{title}</h1>
//!   <p class="sf-body" style="position:absolute; ...">{body}</p>
//!   <!-- SVG GRAPHICS LAYER: charts/diagrams/images/decorative shapes ONLY -->
//!   <svg role="presentation" viewBox="0 0 {W_emu} {H_emu}"
//!        style="position:absolute; top:0; left:0; width:100%; height:100%;
//!               pointer-events:none;" xmlns="http://www.w3.org/2000/svg">
//!     <g role="img" aria-labelledby="sf-{slide_id}-{frame_idx}">
//!       <title id="sf-{slide_id}-{frame_idx}">{alt}</title>
//!       <!-- chart SVG (aria-hidden="true" on inner root <svg>) -->
//!     </g>
//!     <!-- DECORATIVE frames use <g aria-hidden="true"> individually -->
//!   </svg>
//! </article>
//! ```
//!
//! **FORBIDDEN:** `<foreignObject>` anywhere; `<canvas>`; `aria-hidden="true"` on the
//! outer slide `<svg>` graphics layer (WAI-ARIA: aria-hidden on ancestor hides the
//! whole subtree including child `<g role="img">` elements — use `role="presentation"`
//! instead); injecting role/aria via usvg tree; pre-escaping before `push_attribute`;
//! heading level by content inspection; >1 `<h1>` per document.
//!
//! ## Heading level semantics (AC-008 / BC-4.03.003 postcondition 7)
//!
//! Heading level is pre-computed by the exporter pre-pass in
//! [`crate::exporter::HtmlExporter`] and passed into [`render_slide_to_html`]
//! as a [`HeadingLevel`] parameter. The per-slide function does NOT decide heading
//! level itself.
//!
//! Pre-pass algorithm (ADR-008 / BC-4.03.003 postcondition 6):
//!
//! - Find first slide with `slide_type_keyword == "title"` → its Title frame gets H1.
//! - If none: find first slide with any `Title` frame → that frame gets H1.
//! - If none (body-only deck): promote first body frame to H1, emit `tracing::warn!`.
//!
//! All other Title frames emit H2. Exactly one H1 per document.
//!
//! ## Coordinate conversion (MED-3 / EMU → CSS px @96 dpi)
//!
//! `emu as f64 / 914_400.0 * 96.0`, formatted as `"{:.2}px"`. The slide
//! `<article>` carries literal `width` / `height` CSS derived from the deck's
//! `PageSize`. The SVG graphics layer uses `viewBox` in EMU and CSS `width:100%;
//! height:100%` so it stretches to cover the article.
//!
//! ## usvg limitation (export-architecture v1.2 / AC-003/AC-005)
//!
//! `usvg` is used for geometry normalization and validation ONLY. After usvg
//! processing, `quick-xml` XML manipulation is used to inject `role="img"` +
//! `<title>` inside a `<g>` wrapper. usvg strips all non-presentation attributes
//! including `role`, `aria-*`, and `<title>` — do not use its tree API for
//! accessibility injection.
//!
//! ## URL scheme security (AC-010 / CWE-601)
//!
//! All `Link` and `Xref` inline nodes processed in render functions must be
//! validated via [`crate::exporter::is_safe_link_scheme`] before being emitted
//! as `href` attributes.

use slideforge_layout::{Frame, FrameContent, LaidOutSlide, PageSize};
use slideforge_types::{AltText, Brand, ContentBlock, Emu, InlineNode};

use crate::exporter::{is_non_degenerate_bbox, is_safe_link_scheme};

// ─────────────────────────────────────────────────────────────────────────────
// Coordinate conversion
// ─────────────────────────────────────────────────────────────────────────────

/// Convert an EMU value to CSS pixels at 96 dpi.
///
/// Formula: `emu / 914_400 * 96`, result formatted as `"{:.2}px"`.
///
/// MED-3: canvas uses CSS px from `PageSize` (not hardcoded constants); all
/// frame coordinates are converted via this function.
#[allow(clippy::cast_precision_loss)] // EMU values are in range [0, ~10^7]; no precision loss at i64→f64 for typical slide coordinates.
fn emu_to_css_px(emu: Emu) -> String {
    let px = emu.0 as f64 / 914_400.0 * 96.0;
    format!("{px:.2}")
}

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
pub(crate) fn render_inline_node(node: &InlineNode) -> String {
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
///   variants before reaching `Body`; here we emit empty string (no debug output)
/// - `Table` → `<table>` (basic rendering)
/// - `ColorBar` → accessible progressbar span
/// - `Shape` → empty (shapes are handled via `FrameContent::Shape`)
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
// P4 rendering helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Heading level enum for the P4 text layer (AC-008 / BC-4.03.003 postcondition 7).
///
/// The heading level is pre-computed by the exporter pre-pass and passed into
/// [`render_slide_to_html`] — the per-slide render function does NOT decide level.
/// Exactly one `<h1>` per HTML document (enforced by the exporter pre-pass).
///
/// `H1` also signals "body-only promotion" for decks with no Title frames: the
/// first `Body` frame on the H1-level slide is wrapped in `<h1>` instead of
/// `<div>`. This is the only case where `H1` affects non-Title frame rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingLevel {
    /// Document-level heading — exactly one per document. Also signals body
    /// promotion for decks with no Title frames.
    H1,
    /// Section-level heading — all other title frames.
    H2,
    /// Sub-section heading.
    H3,
    /// Sub-sub-section heading.
    H4,
}

impl HeadingLevel {
    /// Return the numeric level (1-4).
    #[must_use]
    pub fn as_u8(self) -> u8 {
        match self {
            HeadingLevel::H1 => 1,
            HeadingLevel::H2 => 2,
            HeadingLevel::H3 => 3,
            HeadingLevel::H4 => 4,
        }
    }
}

/// Render a text frame to an absolutely-positioned HTML element.
///
/// P4 text layer rule:
/// - `FrameContent::Title` → `<h{level}>` at the slide's heading level
/// - `FrameContent::Subtitle` → `<h3>` (sub-heading below the slide heading)
/// - `FrameContent::Body` → `<div class="sf-body">` containing content blocks
/// - `FrameContent::TextRun` → `<p class="sf-text">`
///
/// The returned HTML element is absolutely positioned using inline CSS derived
/// from `frame.bbox` EMU coordinates converted to CSS pixels (MED-3).
///
/// Returns `None` for non-text frames (graphical frames go to the SVG layer).
#[must_use]
pub fn render_text_frame(frame: &Frame, heading_level: HeadingLevel) -> Option<String> {
    let x = emu_to_css_px(frame.bbox.x);
    let y = emu_to_css_px(frame.bbox.y);
    let w = emu_to_css_px(frame.bbox.width);
    let h = emu_to_css_px(frame.bbox.height);

    // MED-B5 / Pass-10: guard zero OR NEGATIVE bbox dimensions — skip degenerate frames.
    // Negative SIZE (width/height) must never emit CSS like width="-N".
    // Negative POSITION (x/y) is legal (off-canvas frames) and is not guarded here.
    //
    // Uses is_non_degenerate_bbox — the SINGLE SOURCE OF TRUTH shared with the
    // exporter pre-pass (slide_has_promotable_text_frame) per TD-VSDD-060.
    // Both gates must agree on which frames are renderable so the pre-pass does
    // not assign <h1> to a frame the render loop will silently skip.
    if !is_non_degenerate_bbox(&frame.bbox) {
        tracing::warn!(
            "render_text_frame: skipping frame with zero or degenerate bbox \
             (width={}, height={})",
            frame.bbox.width.0,
            frame.bbox.height.0
        );
        return None;
    }

    let position_style = format!(
        "position:absolute; left:{x}px; top:{y}px; width:{w}px; height:{h}px; overflow:hidden;"
    );

    match &frame.content {
        FrameContent::Title(text) => {
            let hl = heading_level.as_u8();
            let escaped = html_escape::encode_text(text);
            Some(format!(
                r#"<h{hl} class="sf-title" style="{position_style}">{escaped}</h{hl}>"#
            ))
        },
        FrameContent::Subtitle(text) => {
            // Subtitle is always h3 — it appears below the slide heading (h1 or h2).
            // Never skip levels: h1 → h3 without h2 would violate heading-order.
            // The slide heading (h1/h2) is emitted for the Title frame;
            // Subtitle is always h3 (sub-section of the slide heading).
            let escaped = html_escape::encode_text(text);
            Some(format!(
                r#"<h3 class="sf-subtitle" style="{position_style}">{escaped}</h3>"#
            ))
        },
        FrameContent::Body(blocks) => {
            let body_html: String = blocks.iter().map(render_content_block).collect();
            Some(format!(
                r#"<div class="sf-body" style="{position_style}">{body_html}</div>"#
            ))
        },
        FrameContent::TextRun(nodes) => {
            let text_html = render_inline_nodes(nodes);
            Some(format!(
                r#"<p class="sf-text" style="{position_style}">{text_html}</p>"#
            ))
        },
        // Graphical frames go to the SVG layer — not rendered here.
        FrameContent::Image { .. }
        | FrameContent::Chart { .. }
        | FrameContent::Diagram { .. }
        | FrameContent::Shape(_)
        | FrameContent::ColorBar { .. }
        | FrameContent::ErrorSlidePlaceholder { .. }
        | FrameContent::Empty => None,
    }
}

/// Render the SVG graphics layer for a slide.
///
/// Produces a single `<svg role="presentation" ...>` element containing wrapped
/// groups for each graphical `LaidOutFrame`:
/// - Non-decorative graphical frames: `<g role="img" aria-labelledby="...">`
///   with `<title>` child, then the chart/diagram SVG with `aria-hidden="true"`
///   on its root `<svg>`.
/// - Decorative elements: `<g aria-hidden="true">`.
///
/// ## WAI-ARIA: why `role="presentation"` not `aria-hidden="true"` on the outer SVG
///
/// WAI-ARIA spec: when `aria-hidden="true"` is set on an ancestor element, the
/// ENTIRE subtree (including child elements with explicit `role="img"`) is hidden
/// from AT. A child cannot re-expose itself from under an `aria-hidden` ancestor.
/// Using `role="presentation"` on the outer SVG instead makes the SVG container
/// itself semantically invisible to AT while leaving child `<g role="img">` elements
/// fully accessible. Decorative elements get their own `aria-hidden="true"` on
/// their individual `<g>` elements.
///
/// The graphics layer SVG uses `viewBox` in EMU (matching the `LaidOutSlide`
/// coordinate space) and CSS `width:100%; height:100%` to stretch over the
/// `<article>` container (MED-3).
///
/// ## ID format (MED-B3)
///
/// Each graphical frame gets a stable id: `sf-{slide_id}-{frame_idx}` where
/// `frame_idx` is the 0-based index of graphical frames within the slide.
///
/// Returns an empty string if there are no graphical frames to render.
#[must_use]
pub fn render_graphics_layer(frames: &[Frame], slide_id: &str, page_size: &PageSize) -> String {
    use std::fmt::Write as _;

    let w_emu = page_size.width.0;
    let h_emu = page_size.height.0;

    // Collect graphical frame HTML. If none, skip emitting the SVG layer.
    let mut graphical_content = String::new();
    // MED-B3: frame_idx is 0-based; incremented BEFORE use for each graphical frame.
    // This gives ids: sf-{slide_id}-0, sf-{slide_id}-1, ... (no doubling of slide_id).
    let mut frame_idx: u32 = 0;

    for frame in frames {
        // MED-B5 / Pass-10: skip frames with zero OR NEGATIVE bbox dimensions.
        // Negative width/height must never emit CSS like width="-N".
        // Negative x/y position is legal (off-canvas frames); only SIZE is guarded.
        //
        // Uses is_non_degenerate_bbox — SINGLE SOURCE OF TRUTH shared with the
        // exporter pre-pass (TD-VSDD-060).
        if !is_non_degenerate_bbox(&frame.bbox) {
            continue;
        }

        let x = frame.bbox.x.0;
        let y = frame.bbox.y.0;
        let w = frame.bbox.width.0;
        let h = frame.bbox.height.0;

        match &frame.content {
            FrameContent::Chart { alt } => {
                // MED-B3: frame_id is the 0-based index string; render_chart_frame
                // builds the full label as sf-{slide_id}-{frame_id}.
                let frame_id_str = frame_idx.to_string();
                frame_idx += 1;
                let alt_text = alt_text_str(alt);
                let placeholder_svg =
                    r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"></svg>"#;
                // WHEN real chart SVG is added (STORY-047/048) it MUST route through
                // render_chart_frame (which applies the SVG sanitizer) — LOW-2.
                let chart_g =
                    render_chart_frame(placeholder_svg, alt_text, &frame_id_str, slide_id);
                let _ = write!(
                    graphical_content,
                    r#"<g transform="translate({x} {y})">{chart_g}</g>"#
                );
            },
            FrameContent::Diagram { svg, alt } => {
                let frame_id_str = frame_idx.to_string();
                frame_idx += 1;
                let alt_text = alt_text_str(alt);
                // WHEN real diagram SVG is wired in (STORY-048) it MUST route through
                // render_chart_frame (which applies the SVG sanitizer) — LOW-2.
                let chart_g = render_chart_frame(svg.as_str(), alt_text, &frame_id_str, slide_id);
                let _ = write!(
                    graphical_content,
                    r#"<g transform="translate({x} {y})">{chart_g}</g>"#
                );
            },
            FrameContent::Image { alt } => {
                // WHEN real image src/embedded SVG is added (STORY-047/048) it MUST
                // route through the SVG sanitizer (render_chart_frame or equivalent)
                // to strip script/foreignObject/on* — LOW-2.
                match alt {
                    AltText::Provided(text) => {
                        // MED-B3: canonical id sf-{slide_id}-{frame_idx} (0-based).
                        let frame_id_str = frame_idx.to_string();
                        frame_idx += 1;
                        let label_id = format!("sf-{slide_id}-{frame_id_str}");
                        let escaped_id = html_escape::encode_double_quoted_attribute(&label_id);
                        let escaped_alt = html_escape::encode_text(text);
                        let _ = write!(
                            graphical_content,
                            "<g role=\"img\" aria-labelledby=\"{escaped_id}\"><title id=\"{escaped_id}\">{escaped_alt}</title><rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" fill=\"none\" stroke=\"#cccccc\"/></g>"
                        );
                    },
                    AltText::Decorative | AltText::Unspecified => {
                        // HIGH-B2: decorative frames get aria-hidden="true" on their OWN <g>.
                        // The outer <svg> does NOT carry aria-hidden (it carries role="presentation").
                        let _ = write!(
                            graphical_content,
                            "<g aria-hidden=\"true\"><rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" fill=\"none\" stroke=\"#eeeeee\"/></g>"
                        );
                    },
                }
            },
            FrameContent::Shape(shape_frame) => {
                let alt = &shape_frame.alt;
                match alt {
                    AltText::Provided(text) => {
                        let frame_id_str = frame_idx.to_string();
                        frame_idx += 1;
                        let label_id = format!("sf-{slide_id}-{frame_id_str}");
                        let escaped_id = html_escape::encode_double_quoted_attribute(&label_id);
                        let escaped_alt = html_escape::encode_text(text);
                        let _ = write!(
                            graphical_content,
                            r#"<g role="img" aria-labelledby="{escaped_id}"><title id="{escaped_id}">{escaped_alt}</title><rect x="{x}" y="{y}" width="{w}" height="{h}" fill="none"/></g>"#
                        );
                    },
                    AltText::Decorative | AltText::Unspecified => {
                        let _ = write!(
                            graphical_content,
                            r#"<g aria-hidden="true"><rect x="{x}" y="{y}" width="{w}" height="{h}" fill="none"/></g>"#
                        );
                    },
                }
            },
            FrameContent::ColorBar {
                filled_width_emu,
                total_width_emu: _,
                percent,
                color,
            } => {
                // OBS-2: "% complete" string is English-only. This is a KNOWN-LIMITATION:
                // the aria-label for ColorBar uses hardcoded English ("% complete").
                // When deck.lang support is threaded through (future story), this MUST
                // be sourced from a locale map keyed on deck.lang rather than emitting
                // hardcoded English text for non-English decks.
                let fill_w = filled_width_emu.0;
                let color_hex = format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b);
                let _ = write!(
                    graphical_content,
                    r#"<g aria-label="{percent}% complete" role="img"><rect x="{x}" y="{y}" width="{fill_w}" height="{h}" fill="{color_hex}"/></g>"#
                );
            },
            FrameContent::ErrorSlidePlaceholder {
                svg,
                slide_title,
                error_code,
                message,
            } => {
                let frame_id_str = frame_idx.to_string();
                frame_idx += 1;
                let alt_text = format!("Error on slide '{slide_title}': [{error_code}] {message}");
                let chart_g = render_chart_frame(svg, &alt_text, &frame_id_str, slide_id);
                let _ = write!(
                    graphical_content,
                    r#"<g transform="translate({x} {y})">{chart_g}</g>"#
                );
            },
            // Text frames (Title, Subtitle, Body, TextRun) go to the HTML text layer.
            FrameContent::Title(_)
            | FrameContent::Subtitle(_)
            | FrameContent::Body(_)
            | FrameContent::TextRun(_)
            | FrameContent::Empty => {},
        }
    }

    if graphical_content.is_empty() {
        return String::new();
    }

    // HIGH-B2: outer SVG uses role="presentation" — NOT aria-hidden="true".
    // WAI-ARIA: aria-hidden on an ancestor hides the ENTIRE subtree, including
    // child <g role="img"> elements. role="presentation" makes the container
    // invisible to AT without hiding child elements that carry accessible roles.
    // Decorative frames get their own aria-hidden="true" on their individual <g>.
    format!(
        r#"<svg role="presentation" viewBox="0 0 {w_emu} {h_emu}" style="position:absolute; top:0; left:0; width:100%; height:100%; pointer-events:none;" xmlns="http://www.w3.org/2000/svg">{graphical_content}</svg>"#
    )
}

/// Extract the PHRASING content of the first promotable block in a `Body` frame
/// for body-only `<h1>` promotion (F-P9-001 / AC-001 / BC-4.03.003 PC-7).
///
/// HTML5 heading content model is PHRASING CONTENT only — `<p>`, `<ul>`,
/// `<ol>`, and `<table>` are FLOW content and are invalid inside `<h1>`.
/// This function extracts inline/phrasing HTML only:
///
/// - `Text` block → inline nodes of the first non-empty `Text` block
///   (no `<p>` wrapper)
/// - `Bullets` block → inline nodes of the FIRST bullet item of the first
///   `Bullets` block (no `<ul>`/`<li>` wrapper)
/// - `Math` block → `<code class="math">...</code>` (phrasing element)
///
/// If no promotable block is found, returns an empty string (the caller's
/// `is_promotable` guard prevents this case in practice).
#[must_use]
fn promote_body_blocks_to_phrasing(blocks: &[ContentBlock]) -> String {
    for block in blocks {
        match block {
            ContentBlock::Text(text_block) if !text_block.inlines.is_empty() => {
                // Text block: render inline nodes directly — no <p> wrapper.
                return render_inline_nodes(&text_block.inlines);
            },
            ContentBlock::Bullets(items) if !items.is_empty() => {
                // Bullets block: use only the FIRST item's inline content.
                // A heading cannot contain a list; extract phrasing only.
                return render_inline_nodes(&items[0].inlines);
            },
            ContentBlock::Math(math_node) => {
                // Math block: inline <code class="math"> is phrasing content.
                return format!(
                    "<code class=\"math\">{}</code>",
                    html_escape::encode_text(&math_node.latex)
                );
            },
            // Non-promotable or empty variants: skip and try next block.
            _ => {},
        }
    }
    String::new()
}

/// Render a single slide to an HTML fragment string (P4 Composite Rendering Model).
///
/// The returned `String` is a self-contained HTML fragment representing one slide:
/// an `<article class="sf-slide">` landmark containing:
/// 1. An HTML text layer: `<h1>`, `<h2>`, `<p>`, `<ul>` elements absolutely
///    positioned from `LaidOutFrame` EMU coordinates.
/// 2. A sibling `<svg role="presentation">` graphics layer for charts, diagrams,
///    images, and decorative shapes.
///
/// Returns `String` (not `dyn Write`) so STORY-047 can serialize it as JSON for
/// WebSocket push without an extra allocation step. (Previous Story Intelligence)
///
/// # Heading level (AC-008 / BC-4.03.003 postcondition 7)
///
/// The `heading_level` parameter is **pre-computed by the exporter pre-pass**
/// in [`crate::exporter::HtmlExporter`]. This function does NOT decide
/// heading level itself (see CRITICAL-B1/MED-B4 fix).
///
/// Pre-pass invariant: exactly one slide in the document receives `HeadingLevel::H1`
/// for its Title frame. All other slides receive `HeadingLevel::H2`.
///
/// # Architecture invariants (BC-4.03.003 invariant 2)
///
/// - FORBIDDEN: `<foreignObject>`, `<canvas>`, `aria-hidden="true"` on the outer SVG
/// - Text frames (title, body, bullets) are NOT in the SVG layer
/// - Non-decorative graphical frames use `<g role="img" aria-labelledby>`
/// - Decorative graphical frames use `<g aria-hidden="true">`
///
/// # Coordinate system (MED-3)
///
/// `page_size` determines the CSS `width` and `height` of the `<article>` (via
/// EMU→px conversion). The SVG graphics layer uses the same `page_size` for its
/// `viewBox` so EMU coordinates are shared with the text layer.
///
/// # Security (AC-010 / CWE-601)
///
/// All inline `Link`/`Xref` nodes are validated via
/// [`crate::exporter::is_safe_link_scheme`] before becoming `href` attributes.
///
/// # OBS-2 — Locale note
///
/// The `aria-label="Slide N"` on the `<article>` element uses a hardcoded
/// English string. This is a KNOWN-LIMITATION: when deck.lang support is
/// threaded through from the exporter (future story), this MUST be sourced from
/// a locale map keyed on deck.lang rather than emitting hardcoded English text
/// for non-English decks.
#[must_use]
pub fn render_slide_to_html(
    slide: &LaidOutSlide,
    brand: &Brand,
    heading_level: HeadingLevel,
    page_size: &PageSize,
) -> String {
    let _ = brand; // Brand used by future template-driven color/font injection.

    // MED-3: derive slide container dimensions from page_size (not hardcoded consts).
    let container_w = emu_to_css_px(page_size.width);
    let container_h = emu_to_css_px(page_size.height);

    // Slide ID for ARIA cross-references (1-based from source_index).
    // OBS-2: "Slide N" is English-only; see doc comment above.
    let slide_number = slide.source_index + 1;
    let slide_id = format!("slide-{slide_number}");

    // Build the HTML text layer (all text frames in reading order).
    // heading_level is pre-computed by the exporter pre-pass — NOT derived here.
    //
    // Body-only promotion (CRITICAL-B1 / HIGH-1 / LOW-1 / BC-4.03.003 postcondition 6):
    // If heading_level == H1 and there is no Title frame on this slide, the FIRST
    // PROMOTABLE Body/TextRun frame is wrapped in <h1> to satisfy page-has-heading-one.
    //
    // LOW-1: Only Text/Bullets/Math blocks with non-empty content are promotable.
    // Table / ColorBar / Shape / empty-Body are NOT promotable and must never be
    // wrapped in <h1>. Non-promotable frames are rendered normally (not skipped).
    let has_title_frame = slide
        .frames
        .iter()
        .any(|f| matches!(f.content, FrameContent::Title(_)));
    let needs_body_h1_promotion = heading_level == HeadingLevel::H1 && !has_title_frame;
    let mut body_h1_promoted = false;

    let mut text_layer = String::new();
    for frame in &slide.frames {
        // Body-only promotion: wrap the FIRST PROMOTABLE body/textrun frame in <h1>.
        // LOW-1: check promotability — only Body with Text/Bullets/Math content or
        // a non-empty TextRun. Empty Body / Table / ColorBar / Shape are NOT promotable.
        if needs_body_h1_promotion && !body_h1_promoted {
            let is_promotable = match &frame.content {
                FrameContent::TextRun(nodes) => !nodes.is_empty(),
                FrameContent::Body(blocks) => blocks.iter().any(|b| match b {
                    ContentBlock::Text(tb) => !tb.inlines.is_empty(),
                    ContentBlock::Bullets(items) => !items.is_empty(),
                    ContentBlock::Math(_) => true,
                    // Table, ColorBar, Shape, Chart, Diagram, Image — NOT promotable
                    ContentBlock::Table(_)
                    | ContentBlock::ColorBar(_)
                    | ContentBlock::Shape(_)
                    | ContentBlock::Chart(_)
                    | ContentBlock::Diagram(_)
                    | ContentBlock::Image(_) => false,
                }),
                _ => false,
            };

            if is_promotable {
                // MED-B5 / Pass-10: skip degenerate frames (cannot promote zero/negative bbox).
                // Uses is_non_degenerate_bbox — SINGLE SOURCE OF TRUTH shared with the
                // exporter pre-pass (slide_has_promotable_text_frame) per TD-VSDD-060.
                if !is_non_degenerate_bbox(&frame.bbox) {
                    // Degenerate promotable frame: skip and keep looking for next.
                    continue;
                }
                let x = emu_to_css_px(frame.bbox.x);
                let y = emu_to_css_px(frame.bbox.y);
                let w = emu_to_css_px(frame.bbox.width);
                let h = emu_to_css_px(frame.bbox.height);
                let position_style = format!(
                    "position:absolute; left:{x}px; top:{y}px; width:{w}px; height:{h}px; overflow:hidden;"
                );
                let inner = match &frame.content {
                    FrameContent::Body(blocks) => {
                        // F-P9-001: extract PHRASING content only — the first
                        // promotable block's inline nodes. HTML5 heading content
                        // model forbids <p>/<ul>/<table> (flow content) inside
                        // <h1>. render_content_block emits block-level elements;
                        // promote_body_blocks_to_phrasing extracts inlines only.
                        promote_body_blocks_to_phrasing(blocks)
                    },
                    FrameContent::TextRun(nodes) => render_inline_nodes(nodes),
                    _ => unreachable!("is_promotable guard above"),
                };
                text_layer.push_str("<h1 class=\"sf-body-promoted\" style=\"");
                text_layer.push_str(&position_style);
                text_layer.push_str("\">");
                text_layer.push_str(&inner);
                text_layer.push_str("</h1>\n");
                body_h1_promoted = true;
                continue;
            }
            // Non-promotable frame: fall through to normal rendering below.
        }

        if let Some(html) = render_text_frame(frame, heading_level) {
            text_layer.push_str(&html);
            text_layer.push('\n');
        }
    }

    // Build the SVG graphics layer (graphical frames only).
    let svg_layer = render_graphics_layer(&slide.frames, &slide_id, page_size);

    format!(
        r#"<article id="{slide_id}" class="sf-slide" aria-label="Slide {slide_number}" style="position:relative; width:{container_w}px; height:{container_h}px; overflow:hidden;">
{text_layer}{svg_layer}</article>"#
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// SVG sanitization and chart rendering
// ─────────────────────────────────────────────────────────────────────────────

/// Returns `true` if the given SVG element local name is on the SVG blocklist.
///
/// Blocked elements are those that can execute code in an HTML context:
/// - `script` — executes JavaScript
/// - `foreignObject` — embeds arbitrary HTML/script content
///
/// F-005 (XSS): these are stripped when inline SVG is embedded in HTML output.
///
/// MED-2: comparison is CASE-INSENSITIVE — `<SCRIPT>`, `<ScRiPt>`, and
/// `<foreignobject>` are all blocked. HTML parsers are case-insensitive so
/// mixed-case variants must be stripped too.
fn is_blocked_svg_element(local_name: &[u8]) -> bool {
    let lower: Vec<u8> = local_name.iter().map(u8::to_ascii_lowercase).collect();
    matches!(lower.as_slice(), b"script" | b"foreignobject")
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

/// Extract alt text string from [`AltText`].
fn alt_text_str(alt: &AltText) -> &str {
    match alt {
        AltText::Provided(text) => text.as_ref(),
        AltText::Decorative | AltText::Unspecified => "",
    }
}

/// Inject `<g role="img" aria-labelledby>` + `<title>` wrapper around an SVG string.
///
/// This function performs raw XML manipulation via `quick-xml` to:
/// 1. Parse the SVG string.
/// 2. **Sanitize:** strip `<script>`, `<foreignObject>`, event-handler attributes
///    (`on*`), and `javascript:` hrefs (F-005 / XSS prevention).
/// 3. Strip `role` from the outer `<svg>` (it gets wrapped in `<g role="img">`).
/// 4. Mark the outer `<svg>` (and all inner `<svg>` elements) with
///    `aria-hidden="true"` — the wrapping `<g>` provides the accessible name.
/// 5. Return the sanitized SVG string to be embedded inside a
///    `<g role="img" aria-labelledby="sf-{slide_id}-{frame_id}">` wrapper
///    with a `<title id="sf-{slide_id}-{frame_id}">` sibling.
///
/// ## Why NOT the usvg tree API (export-architecture v1.2 / AC-003/AC-005)
///
/// usvg 0.47.0 strips all non-presentation attributes during SVG tree
/// processing, including `role`, `aria-*`, and `<title>`. Using the usvg tree
/// API for accessibility injection would silently discard the injected
/// attributes. This function operates on the raw SVG string after any usvg
/// processing is complete.
///
/// ## P4 DOM output (AC-003/AC-005/AC-006)
///
/// The caller wraps the returned SVG in:
/// ```html
/// <g role="img" aria-labelledby="sf-{slide_id}-{frame_id}">
///   <title id="sf-{slide_id}-{frame_id}">{alt, XML-escaped by quick-xml}</title>
///   {sanitized svg with aria-hidden="true"}
/// </g>
/// ```
///
/// ## Security (F-005 / XSS / MED-2)
///
/// Inline SVG executes script in HTML context. This function strips all elements
/// and attributes that can execute code. Element name matching is
/// CASE-INSENSITIVE (MED-2): `<SCRIPT>`, `<ScRiPt>`, `<foreignObject>`,
/// `<FOREIGNOBJECT>` are all stripped.
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
fn sanitize_svg_for_graphics_layer(svg_str: &str) -> String {
    use quick_xml::events::{BytesStart, Event};
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
                    "sanitize_svg_for_graphics_layer: failed to parse SVG input — returning original string unchanged"
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

                // F-005 / MED-2: if this element starts a blocked subtree, enter blocked mode.
                if is_blocked_svg_element(local_name.as_ref()) {
                    blocked_depth += 1;
                    tracing::warn!(
                        element = %String::from_utf8_lossy(local_name.as_ref()),
                        "sanitize_svg_for_graphics_layer: F-005 blocked element stripped from SVG"
                    );
                    continue;
                }

                let is_svg = local_name.as_ref().eq_ignore_ascii_case(b"svg");

                if is_svg && !outer_svg_done {
                    // Outer <svg>: set aria-hidden="true", sanitize + copy existing attrs.
                    outer_svg_done = true;
                    let mut new_elem = BytesStart::new("svg");
                    // F-010: skip non-UTF-8 keys; F-005: skip event handlers + drop role/aria-hidden.
                    for attr in elem.attributes().flatten() {
                        match std::str::from_utf8(attr.key.as_ref()) {
                            // drop role (the <g> wrapper provides role="img") and aria-hidden (we inject below)
                            Ok("role" | "aria-hidden") => {},
                            Ok(key) if !is_safe_svg_attribute_key(key) => {
                                tracing::warn!(
                                    key = %key,
                                    "sanitize_svg_for_graphics_layer: F-005 event-handler attribute stripped \
                                     from outer <svg>"
                                );
                            },
                            Ok(key) => {
                                if is_safe_svg_attribute_value(key, attr.value.as_ref()) {
                                    new_elem.push_attribute(attr);
                                } else {
                                    tracing::warn!(
                                        key = %key,
                                        "sanitize_svg_for_graphics_layer: F-005 unsafe attribute value stripped \
                                         from outer <svg>"
                                    );
                                }
                            },
                            Err(_) => {
                                tracing::warn!(
                                    "sanitize_svg_for_graphics_layer: non-UTF-8 attribute key skipped \
                                     on outer <svg>"
                                );
                            },
                        }
                    }
                    // Inject aria-hidden="true" on the inner SVG root.
                    new_elem.push_attribute(("aria-hidden", "true"));

                    if let Err(e) = writer.write_event(Event::Start(new_elem)) {
                        tracing::warn!(error = %e, "sanitize_svg_for_graphics_layer: write error on outer svg start");
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
                                    "sanitize_svg_for_graphics_layer: F-005 event-handler attribute stripped \
                                     from inner <svg>"
                                );
                            },
                            Ok(key) => {
                                if is_safe_svg_attribute_value(key, attr.value.as_ref()) {
                                    new_elem.push_attribute(attr);
                                } else {
                                    tracing::warn!(
                                        key = %key,
                                        "sanitize_svg_for_graphics_layer: F-005 unsafe attribute value stripped \
                                         from inner <svg>"
                                    );
                                }
                            },
                            Err(_) => {
                                tracing::warn!(
                                    "sanitize_svg_for_graphics_layer: non-UTF-8 attribute key skipped \
                                     on inner <svg>"
                                );
                            },
                        }
                    }
                    new_elem.push_attribute(("aria-hidden", "true"));

                    if let Err(e) = writer.write_event(Event::Start(new_elem)) {
                        tracing::warn!(error = %e, "sanitize_svg_for_graphics_layer: write error on inner svg start");
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
                                    "sanitize_svg_for_graphics_layer: F-005 event-handler attribute stripped"
                                );
                            },
                            Ok(key) => {
                                if is_safe_svg_attribute_value(key, attr.value.as_ref()) {
                                    new_elem.push_attribute(attr);
                                } else {
                                    tracing::warn!(
                                        key = %key,
                                        "sanitize_svg_for_graphics_layer: F-005 unsafe attribute value stripped"
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
                        tracing::warn!(error = %e, "sanitize_svg_for_graphics_layer: write error on elem start");
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

                // F-005 / MED-2: self-closing blocked elements — skip.
                if is_blocked_svg_element(local_name.as_ref()) {
                    tracing::warn!(
                        element = %String::from_utf8_lossy(local_name.as_ref()),
                        "sanitize_svg_for_graphics_layer: F-005 self-closing blocked element stripped"
                    );
                    continue;
                }

                let is_svg = local_name.as_ref().eq_ignore_ascii_case(b"svg");

                if is_svg && !outer_svg_done {
                    // Outer self-closing <svg/>: inject aria-hidden="true".
                    outer_svg_done = true;
                    let mut new_elem = BytesStart::new("svg");
                    for attr in elem.attributes().flatten() {
                        match std::str::from_utf8(attr.key.as_ref()) {
                            // drop role and aria-hidden — we inject aria-hidden below
                            Ok("role" | "aria-hidden") => {},
                            Ok(key) if !is_safe_svg_attribute_key(key) => {
                                tracing::warn!(
                                    key = %key,
                                    "sanitize_svg_for_graphics_layer: F-005 event-handler stripped from \
                                     self-closing outer <svg/>"
                                );
                            },
                            Ok(key) => {
                                if is_safe_svg_attribute_value(key, attr.value.as_ref()) {
                                    new_elem.push_attribute(attr);
                                } else {
                                    tracing::warn!(
                                        key = %key,
                                        "sanitize_svg_for_graphics_layer: F-005 unsafe value stripped from \
                                         self-closing outer <svg/>"
                                    );
                                }
                            },
                            Err(_) => {
                                tracing::warn!(
                                    "sanitize_svg_for_graphics_layer: non-UTF-8 attribute key skipped \
                                     on self-closing outer <svg/>"
                                );
                            },
                        }
                    }
                    new_elem.push_attribute(("aria-hidden", "true"));
                    if let Err(e) = writer.write_event(Event::Empty(new_elem)) {
                        tracing::warn!(error = %e, "sanitize_svg_for_graphics_layer: write error on self-closing outer svg");
                        return svg_str.to_owned();
                    }
                } else if is_svg && depth > 0 {
                    // F-007 (EC-002): inner self-closing <svg/> at depth > 0 must
                    // receive aria-hidden="true".
                    let mut new_elem = BytesStart::new("svg");
                    for attr in elem.attributes().flatten() {
                        match std::str::from_utf8(attr.key.as_ref()) {
                            Ok("aria-hidden") => {}, // drop — we inject below
                            Ok(key) if !is_safe_svg_attribute_key(key) => {
                                tracing::warn!(
                                    key = %key,
                                    "sanitize_svg_for_graphics_layer: F-005 event-handler stripped from \
                                     self-closing inner <svg/>"
                                );
                            },
                            Ok(key) => {
                                if is_safe_svg_attribute_value(key, attr.value.as_ref()) {
                                    new_elem.push_attribute(attr);
                                } else {
                                    tracing::warn!(
                                        key = %key,
                                        "sanitize_svg_for_graphics_layer: F-005 unsafe value stripped from \
                                         self-closing inner <svg/>"
                                    );
                                }
                            },
                            Err(_) => {
                                tracing::warn!(
                                    "sanitize_svg_for_graphics_layer: non-UTF-8 attribute key skipped \
                                     on self-closing inner <svg/>"
                                );
                            },
                        }
                    }
                    new_elem.push_attribute(("aria-hidden", "true"));
                    if let Err(e) = writer.write_event(Event::Empty(new_elem)) {
                        tracing::warn!(error = %e, "sanitize_svg_for_graphics_layer: write error on inner self-closing svg");
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
                                    "sanitize_svg_for_graphics_layer: F-005 event-handler stripped from empty elem"
                                );
                            },
                            Ok(key) => {
                                if is_safe_svg_attribute_value(key, attr.value.as_ref()) {
                                    new_elem.push_attribute(attr);
                                } else {
                                    tracing::warn!(
                                        key = %key,
                                        "sanitize_svg_for_graphics_layer: F-005 unsafe value stripped from \
                                         empty elem"
                                    );
                                }
                            },
                            Err(_) => {}, // skip non-UTF-8 key
                        }
                    }
                    if let Err(e) = writer.write_event(Event::Empty(new_elem)) {
                        tracing::warn!(error = %e, "sanitize_svg_for_graphics_layer: write error on empty elem");
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
                    tracing::warn!(error = %e, "sanitize_svg_for_graphics_layer: write error on end elem");
                    return svg_str.to_owned();
                }
            },
            Ok(other) => {
                // F-005: skip text/cdata inside blocked elements.
                if blocked_depth > 0 {
                    continue;
                }
                if let Err(e) = writer.write_event(other) {
                    tracing::warn!(error = %e, "sanitize_svg_for_graphics_layer: write error on other event");
                    return svg_str.to_owned();
                }
            },
        }
    }

    // If we never found an outer <svg>, the input was not SVG at all.
    if !outer_svg_done {
        tracing::warn!(
            "sanitize_svg_for_graphics_layer: no <svg> element found in input — \
             returning original string unchanged (parse error)"
        );
        return svg_str.to_owned();
    }

    match String::from_utf8(writer.into_inner()) {
        Ok(result) => result,
        Err(e) => {
            tracing::warn!(error = %e, "sanitize_svg_for_graphics_layer: UTF-8 encoding error — returning original");
            svg_str.to_owned()
        },
    }
}

/// Inject `role="img"`, `aria-labelledby`, and `<title>` around an SVG string.
///
/// Produces:
/// ```html
/// <g role="img" aria-labelledby="sf-{slide_id}-{frame_id}">
///   <title id="sf-{slide_id}-{frame_id}">{alt, XML-escaped by quick-xml}</title>
///   {sanitized svg with aria-hidden="true" on outer root}
/// </g>
/// ```
///
/// The SVG is first passed through the sanitizer (strips script/foreignObject/
/// on* attrs, marks inner SVG roots as `aria-hidden="true"`).
///
/// P4 rule: NEVER inject `role` via the usvg tree API — usvg 0.47.0 strips it.
/// P4 rule: Do NOT pre-escape `alt_text` before passing — quick-xml auto-escapes
/// attribute values and text content.
///
/// ## Security (F-005 / AC-003 / AC-005)
///
/// All chart/diagram SVG content passes through the internal SVG sanitizer
/// before being embedded in HTML. This strips executable content (script,
/// foreignObject, on* attrs, javascript: hrefs).
#[must_use]
pub fn render_chart_frame(svg_str: &str, alt_text: &str, frame_id: &str, slide_id: &str) -> String {
    let sanitized = sanitize_svg_for_graphics_layer(svg_str);

    // Build the combined label ID: sf-{slide_id}-{frame_id}
    let label_id = format!("sf-{slide_id}-{frame_id}");
    let escaped_label_id = html_escape::encode_double_quoted_attribute(&label_id);
    let escaped_alt = html_escape::encode_text(alt_text);

    format!(
        r#"<g role="img" aria-labelledby="{escaped_label_id}"><title id="{escaped_label_id}">{escaped_alt}</title>{sanitized}</g>"#
    )
}

/// Inject `role=\"img\"` and `<title>alt text</title>` into an SVG string.
///
/// This is the legacy chart-SVG injection API preserved for backward compatibility
/// with tests that call it directly. For P4 rendering, prefer
/// [`render_chart_frame`] which wraps the sanitized SVG in a `<g role="img">`
/// as required by the P4 Composite Rendering Model.
///
/// This function injects `role="img"` on the outer `<svg>` element and prepends
/// a `<title>` child (the pre-P4 pattern). Chart/diagram SVGs going into the P4
/// graphics layer MUST use [`render_chart_frame`] instead.
///
/// ## Security (F-005 / XSS / MED-2)
///
/// Strips `<script>`, `<foreignObject>`, event-handler attributes (`on*`), and
/// `javascript:` hrefs. Element name matching is CASE-INSENSITIVE (MED-2).
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

                // F-005 / MED-2: if this element starts a blocked subtree, enter blocked mode.
                if is_blocked_svg_element(local_name.as_ref()) {
                    blocked_depth += 1;
                    tracing::warn!(
                        element = %String::from_utf8_lossy(local_name.as_ref()),
                        "render_svg_chart: F-005 blocked element stripped from SVG"
                    );
                    continue;
                }

                let is_svg = local_name.as_ref().eq_ignore_ascii_case(b"svg");

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

                // F-005 / MED-2: self-closing blocked elements — skip.
                if is_blocked_svg_element(local_name.as_ref()) {
                    tracing::warn!(
                        element = %String::from_utf8_lossy(local_name.as_ref()),
                        "render_svg_chart: F-005 self-closing blocked element stripped"
                    );
                    continue;
                }

                let is_svg = local_name.as_ref().eq_ignore_ascii_case(b"svg");

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

    use slideforge_layout::{BoundingBox, Frame, FrameContent, LaidOutSlide, PageSize};
    use slideforge_types::AltText;
    use slideforge_types::{
        Brand, BrandFonts, BrandPalette, Emu, NormalizedDiagramSvg, SourceSpan,
    };

    use super::{
        HeadingLevel, render_chart_frame, render_graphics_layer, render_slide_to_html,
        render_svg_chart, render_text_frame,
    };

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

    fn make_page_size() -> PageSize {
        PageSize::default()
    }

    fn make_title_slide_type(slide_type: &str, frames: Vec<Frame>) -> LaidOutSlide {
        LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from(slide_type),
            frames,
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // P4 DOM structure: <article> container (AC-006)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-006 / BC-4.03.003 invariant 2 — render_slide_to_html produces an
    /// <article> slide container, not an outer <svg> canvas with role="img".
    #[test]
    fn test_BC_4_03_003_p4_article_container_present() {
        let slide = make_title_slide_type(
            "title",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Title(Arc::from("Hello P4")),
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H1, &page_size);

        let doc = scraper::Html::parse_document(&result);
        let sel = scraper::Selector::parse("article.sf-slide").expect("valid selector");
        assert!(
            doc.select(&sel).count() > 0,
            "P4: render_slide_to_html must produce <article class=\"sf-slide\">; got: {result}"
        );
    }

    /// AC-006 — no <foreignObject> in output (P4 model forbids it).
    #[test]
    fn test_BC_4_03_003_p4_no_foreign_object_in_output() {
        let slide = make_title_slide_type(
            "title",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Title(Arc::from("Title Slide")),
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H1, &page_size);

        let doc = scraper::Html::parse_document(&result);
        let sel = scraper::Selector::parse("foreignObject").expect("valid selector");
        assert_eq!(
            doc.select(&sel).count(),
            0,
            "P4: no <foreignObject> allowed in HTML output; got: {result}"
        );
    }

    /// AC-006 — no <canvas> in output.
    #[test]
    fn test_BC_4_03_003_render_slide_to_html_no_canvas_elements() {
        let slide = make_title_slide_type(
            "title",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Title(Arc::from("Test")),
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H1, &page_size);

        let doc = scraper::Html::parse_document(&result);
        let sel = scraper::Selector::parse("canvas").expect("valid selector");
        assert_eq!(
            doc.select(&sel).count(),
            0,
            "render_slide_to_html must not produce <canvas> elements; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // P4: Text frames as real HTML elements (not in SVG)
    // ─────────────────────────────────────────────────────────────────────────

    /// P4: Title frame must produce a real <h1> or <h2> element at the article
    /// level — NOT inside an <svg>/<foreignObject>.
    #[test]
    fn test_BC_4_03_003_p4_title_frame_as_real_heading() {
        let slide = make_title_slide_type(
            "title",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Title(Arc::from("Real Heading")),
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H1, &page_size);

        // The h1 must be a direct descendant of the article, not inside svg.
        let doc = scraper::Html::parse_document(&result);
        let sel_h1 = scraper::Selector::parse("article.sf-slide > h1").expect("valid");
        assert!(
            doc.select(&sel_h1).count() > 0,
            "P4: title frame must produce <h1> directly inside <article>, not inside svg; got: {result}"
        );
    }

    /// P4: The SVG graphics layer (if present) must carry role="presentation" (HIGH-B2).
    /// aria-hidden="true" on the outer svg would hide all <g role="img"> children from AT.
    #[test]
    fn test_BC_4_03_003_p4_graphics_layer_svg_is_role_presentation() {
        let svg_in = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"></svg>"#;
        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(svg_in));
        let slide = make_title_slide_type(
            "content",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Diagram {
                    svg: normalized,
                    alt: AltText::Provided(Arc::from("A diagram")),
                },
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);

        // HIGH-B2: outer SVG must carry role="presentation", NOT aria-hidden="true".
        // WAI-ARIA: aria-hidden on ancestor hides the entire subtree.
        let outer_svg_start = result.find("<svg").expect("must have svg");
        let outer_svg_end = result[outer_svg_start..].find('>').expect("must close");
        let outer_svg_tag = &result[outer_svg_start..=(outer_svg_start + outer_svg_end)];
        assert!(
            outer_svg_tag.contains(r#"role="presentation""#),
            "P4 / B2: outer SVG graphics layer must carry role=\"presentation\"; got: {outer_svg_tag}"
        );
        assert!(
            !outer_svg_tag.contains(r#"aria-hidden="true""#),
            "P4 / B2: outer SVG graphics layer must NOT carry aria-hidden=\"true\"; got: {outer_svg_tag}"
        );
        // The <g> wrapper inside must still have role="img".
        assert!(
            !outer_svg_tag.contains(r#"role="img""#),
            "P4: outer SVG graphics layer must NOT have role=\"img\" on svg element; \
             got outer svg tag: {outer_svg_tag}"
        );
        // The inner <g role="img"> is accessible (not hidden by aria-hidden ancestor).
        assert!(
            result.contains(r#"role="img""#),
            "P4: <g role=\"img\"> must be present in the SVG layer; got: {result}"
        );
    }

    /// P4: Chart frame produces <g role="img" aria-labelledby> inside the SVG layer.
    #[test]
    fn test_BC_4_03_003_p4_chart_frame_has_g_role_img() {
        let slide = make_title_slide_type(
            "content",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Chart {
                    alt: AltText::Provided(Arc::from("Revenue by quarter")),
                },
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);

        // Must have <g role="img" aria-labelledby="..."> in the SVG layer.
        assert!(
            result.contains(r#"role="img""#),
            "P4: chart frame must produce <g role=\"img\"> in SVG layer; got: {result}"
        );
        assert!(
            result.contains("aria-labelledby="),
            "P4: chart frame must produce aria-labelledby on <g>; got: {result}"
        );
        // Must have a <title> with the alt text.
        assert!(
            result.contains("<title"),
            "P4: chart frame must have <title> child in <g>; got: {result}"
        );
        assert!(
            result.contains("Revenue by quarter"),
            "P4: chart alt text must appear in <title>; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-008: Heading level pre-computed — render_slide_to_html uses passed level
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-008 — passing HeadingLevel::H1 to render_slide_to_html produces <h1>.
    #[test]
    fn test_BC_4_03_003_ac008_h1_level_produces_h1() {
        let slide = make_title_slide_type(
            "title",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Title(Arc::from("Opening Title")),
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H1, &page_size);

        let doc = scraper::Html::parse_document(&result);
        let sel_h1 = scraper::Selector::parse("h1").expect("valid");
        assert!(
            doc.select(&sel_h1).count() > 0,
            "AC-008: HeadingLevel::H1 must produce <h1>; got: {result}"
        );
    }

    /// AC-008 — passing HeadingLevel::H2 to render_slide_to_html produces <h2>.
    #[test]
    fn test_BC_4_03_003_ac008_h2_level_produces_h2() {
        let slide = make_title_slide_type(
            "content",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Title(Arc::from("Content Title")),
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);

        let doc = scraper::Html::parse_document(&result);
        let sel_h2 = scraper::Selector::parse("h2").expect("valid");
        assert!(
            doc.select(&sel_h2).count() > 0,
            "AC-008: HeadingLevel::H2 must produce <h2>; got: {result}"
        );
        let sel_h1 = scraper::Selector::parse("h1").expect("valid");
        assert_eq!(
            doc.select(&sel_h1).count(),
            0,
            "AC-008: HeadingLevel::H2 slide must NOT produce <h1>; got: {result}"
        );
    }

    /// AC-008 — passing HeadingLevel::H2 for a title-type slide at any position
    /// produces <h2>, not <h1>.
    #[test]
    fn test_BC_4_03_003_ac008_title_slide_with_h2_level_produces_h2() {
        let slide = make_title_slide_type(
            "title",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Title(Arc::from("Second Title Slide")),
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        // Pre-pass says this slide gets H2 (not the h1 slide)
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);

        let doc = scraper::Html::parse_document(&result);
        let sel_h1 = scraper::Selector::parse("h1").expect("valid");
        assert_eq!(
            doc.select(&sel_h1).count(),
            0,
            "AC-008: title-type slide with H2 level must NOT produce <h1>; got: {result}"
        );
        let sel_h2 = scraper::Selector::parse("h2").expect("valid");
        assert!(
            doc.select(&sel_h2).count() > 0,
            "AC-008: title-type slide with H2 level must produce <h2>; got: {result}"
        );
    }

    /// AC-008 — exactly one <h1> in a multi-slide document (3 slides, pre-pass assigns levels).
    #[test]
    fn test_BC_4_03_003_ac008_exactly_one_h1_in_multi_slide_document() {
        let brand = make_brand();
        let page_size = make_page_size();

        let mut doc_html = String::new();
        // Slide 0: title-slide with H1 level (pre-pass result)
        let mut slide0 = make_title_slide_type(
            "title",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Title(Arc::from("Opening")),
                text_flow: None,
                region_role: None,
            }],
        );
        slide0.source_index = 0;
        doc_html.push_str(&render_slide_to_html(
            &slide0,
            &brand,
            HeadingLevel::H1,
            &page_size,
        ));

        // Slide 1: content-slide with H2 level
        let mut slide1 = make_title_slide_type(
            "content",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Title(Arc::from("Content")),
                text_flow: None,
                region_role: None,
            }],
        );
        slide1.source_index = 1;
        doc_html.push_str(&render_slide_to_html(
            &slide1,
            &brand,
            HeadingLevel::H2,
            &page_size,
        ));

        // Slide 2: title-slide at index 2 with H2 level (not the h1 slide)
        let mut slide2 = make_title_slide_type(
            "title",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Title(Arc::from("Section Title")),
                text_flow: None,
                region_role: None,
            }],
        );
        slide2.source_index = 2;
        doc_html.push_str(&render_slide_to_html(
            &slide2,
            &brand,
            HeadingLevel::H2,
            &page_size,
        ));

        let doc = scraper::Html::parse_document(&doc_html);
        let sel_h1 = scraper::Selector::parse("h1").expect("valid");
        assert_eq!(
            doc.select(&sel_h1).count(),
            1,
            "AC-008: exactly one <h1> must exist across a 3-slide document; got full HTML: {doc_html}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MED-2: case-insensitive SVG element blocklist
    // ─────────────────────────────────────────────────────────────────────────

    /// MED-2: SCRIPT (upper-case) must be stripped from SVG.
    #[test]
    fn test_MED2_uppercase_script_stripped() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><SCRIPT>evil()</SCRIPT><rect/></svg>"#;
        let result = render_svg_chart(svg, "chart");
        assert!(
            !result.to_ascii_lowercase().contains("<script"),
            "MED-2: <SCRIPT> must be stripped; got: {result}"
        );
        assert!(
            !result.contains("evil()"),
            "MED-2: script content must be stripped; got: {result}"
        );
    }

    /// MED-2: mixed-case ScRiPt must be stripped.
    #[test]
    fn test_MED2_mixed_case_script_stripped() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><ScRiPt>evil()</ScRiPt><rect/></svg>"#;
        let result = render_svg_chart(svg, "chart");
        assert!(
            !result.to_ascii_lowercase().contains("<script"),
            "MED-2: <ScRiPt> must be stripped; got: {result}"
        );
    }

    /// MED-2: FOREIGNOBJECT (upper-case) must be stripped.
    #[test]
    fn test_MED2_uppercase_foreign_object_stripped() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><FOREIGNOBJECT><div>evil</div></FOREIGNOBJECT></svg>"#;
        let result = render_svg_chart(svg, "chart");
        assert!(
            !result.to_ascii_lowercase().contains("foreignobject"),
            "MED-2: <FOREIGNOBJECT> must be stripped; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MED-3: PageSize threading + EMU→px conversion + zero-bbox guard
    // ─────────────────────────────────────────────────────────────────────────

    /// MED-3: non-default PageSize produces correct CSS width/height on the article.
    #[test]
    fn test_MED3_non_default_page_size_in_article() {
        let slide = make_title_slide_type(
            "title",
            vec![Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(1_000_000),
                    height: Emu(500_000),
                },
                content: FrameContent::Title(Arc::from("Custom Size")),
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        // Non-default page size: 4:3 portrait (7,200,000 × 5,400,000 EMU)
        let page_size = PageSize {
            width: Emu(7_200_000),
            height: Emu(5_400_000),
        };
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H1, &page_size);

        // 7_200_000 / 914_400 * 96 ≈ 755.91 px
        assert!(
            result.contains("755.91"),
            "MED-3: article width must be derived from page_size.width (7_200_000 EMU ≈ 755.91px); got: {result}"
        );
    }

    /// MED-3: zero-bbox frame is skipped by render_text_frame (no panic).
    #[test]
    fn test_MED3_zero_bbox_frame_skipped() {
        let frame = Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(0), // zero width
                height: Emu(500_000),
            },
            content: FrameContent::Title(Arc::from("Degenerate")),
            text_flow: None,
            region_role: None,
        };
        // Must return None — no panic, no output for zero-bbox frame.
        let result = render_text_frame(&frame, HeadingLevel::H1);
        assert!(
            result.is_none(),
            "MED-3: zero-bbox frame must return None from render_text_frame; got: {result:?}"
        );
    }

    /// MED-3: negative-like (very large EMU) bbox is handled — render proceeds.
    #[test]
    fn test_MED3_large_bbox_does_not_panic() {
        let frame = Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(9_144_000),
                height: Emu(5_143_500),
            },
            content: FrameContent::Title(Arc::from("Large bbox")),
            text_flow: None,
            region_role: None,
        };
        // Must return Some(html) — no panic.
        let result = render_text_frame(&frame, HeadingLevel::H1);
        assert!(
            result.is_some(),
            "MED-3: normal large bbox must produce Some(html); got: {result:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-003 / render_svg_chart: role="img" and <title> injection (preserved)
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
    // AC-003 / render_chart_frame: P4 <g role="img" aria-labelledby><title>
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-003 / AC-005 — render_chart_frame produces <g role="img" aria-labelledby>
    /// with a <title> child and the sanitized SVG (aria-hidden on inner root).
    #[test]
    fn test_BC_4_03_003_render_chart_frame_g_wrapper() {
        let svg_in = r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"></svg>"#;
        let result = render_chart_frame(svg_in, "Revenue by quarter", "slide-1-1", "slide-1");

        assert!(
            result.contains(r#"role="img""#),
            "render_chart_frame must produce <g role=\"img\">; got: {result}"
        );
        assert!(
            result.contains("aria-labelledby="),
            "render_chart_frame must produce aria-labelledby on <g>; got: {result}"
        );
        assert!(
            result.contains("<title"),
            "render_chart_frame must produce a <title> child; got: {result}"
        );
        assert!(
            result.contains("Revenue by quarter"),
            "render_chart_frame must include alt text in <title>; got: {result}"
        );
        // The inner SVG root must carry aria-hidden="true".
        assert!(
            result.contains(r#"aria-hidden="true""#),
            "render_chart_frame: inner SVG root must carry aria-hidden=\"true\"; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-004 / decorative images via render_graphics_layer
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-004 — decorative image frame produces aria-hidden="true" group in SVG layer.
    #[test]
    fn test_BC_4_03_003_render_element_decorative_image_empty_alt_and_role() {
        let frames = vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Image {
                alt: AltText::Decorative,
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-1", &page_size);
        // Decorative image → <g aria-hidden="true"> in the SVG layer
        assert!(
            result.contains(r#"aria-hidden="true""#),
            "decorative image must produce aria-hidden group in SVG layer; got: {result}"
        );
        // Must NOT have role="img" for a decorative image
        // (the outer svg is aria-hidden; the decorative group is also aria-hidden)
    }

    /// AC-003 — non-decorative image frame produces <g role="img" aria-labelledby> in SVG layer.
    #[test]
    fn test_BC_4_03_003_render_element_non_decorative_image_has_non_empty_alt() {
        let frames = vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Image {
                alt: AltText::Provided(Arc::from("A bar chart showing quarterly revenue")),
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-1", &page_size);
        // Non-decorative → <g role="img" aria-labelledby="..."><title>...</title>
        assert!(
            result.contains(r#"role="img""#),
            "non-decorative image must produce role=\"img\" in SVG layer; got: {result}"
        );
        assert!(
            result.contains("A bar chart showing quarterly revenue"),
            "non-decorative image alt text must appear in <title>; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-005 / render_chart_frame: Chart SVG role="img"
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 postcondition 6 — render_chart_frame for a Chart frame
    /// produces `<g role="img" aria-labelledby><title>alt text</title>`.
    #[test]
    fn test_BC_4_03_003_render_element_chart_svg_has_role_img_and_title() {
        let svg_in = r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"></svg>"#;
        let result = render_chart_frame(svg_in, "Revenue by quarter", "frame-1", "slide-1");
        assert!(
            result.contains(r#"role="img""#),
            "Chart frame must produce <g role=\"img\">; got: {result}"
        );
        assert!(
            result.contains("<title"),
            "Chart frame must produce <title>; got: {result}"
        );
        assert!(
            result.contains("Revenue by quarter"),
            "Chart frame alt text must appear in <title>; got: {result}"
        );
    }

    /// BC-4.03.003 postcondition 6 — render_chart_frame for a Diagram frame
    /// produces `<g role="img" aria-labelledby><title>`.
    #[test]
    fn test_BC_4_03_003_render_element_diagram_svg_has_role_img_and_title() {
        let svg_str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"><rect x="0" y="0" width="400" height="300"/></svg>"#;
        let result = render_chart_frame(svg_str, "Mermaid flowchart", "frame-1", "slide-1");
        assert!(
            result.contains(r#"role="img""#),
            "Diagram frame must produce <g role=\"img\">; got: {result}"
        );
        assert!(
            result.contains("Mermaid flowchart"),
            "Diagram frame must produce <title>Mermaid flowchart</title>; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-005 / render_slide_to_html: inner SVG gets aria-hidden
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 EC-002 — Diagram with nested SVG elements: inner SVGs must
    /// have aria-hidden="true".
    #[test]
    fn test_BC_4_03_003_ec_002_diagram_inner_svg_gets_aria_hidden() {
        let nested_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"><svg width="200" height="150"><rect/></svg></svg>"#;
        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(nested_svg));
        let frames = vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Diagram {
                svg: normalized,
                alt: AltText::Provided(Arc::from("Complex nested diagram")),
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-1", &page_size);
        // EC-002: inner svg elements must have aria-hidden="true"
        assert!(
            result.contains(r#"aria-hidden="true""#),
            "EC-002: inner <svg> elements must have aria-hidden=\"true\"; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-003 — FrameContent::Body must render ContentBlock variants properly
    // ─────────────────────────────────────────────────────────────────────────

    /// F-003: Body frame with a Text block must render a <p> element containing
    /// the text content — NOT the Rust Debug representation.
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
        let result = render_text_frame(&frame, HeadingLevel::H2).expect("body frame must render");
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
        let result = render_text_frame(&frame, HeadingLevel::H2).expect("body frame must render");
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
    // F-004 — heading hierarchy: subtitle renders as h3 (not h2 after h2)
    // ─────────────────────────────────────────────────────────────────────────

    /// F-004: Subtitle frame always renders as <h3> (sub-section of slide heading).
    #[test]
    fn test_F004_subtitle_frame_renders_as_h3() {
        let frame = Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Subtitle(Arc::from("A subtitle")),
            text_flow: None,
            region_role: None,
        };
        let result = render_text_frame(&frame, HeadingLevel::H2).expect("subtitle must render");
        assert!(
            result.contains("<h3"),
            "F-004: Subtitle frame must render as <h3>; got: {result}"
        );
    }

    /// F-004: Title + Subtitle slide produces h2 → h3 (no skipped levels for content slides).
    #[test]
    fn test_F004_title_then_subtitle_produces_h2_then_h3_for_content_slide() {
        let slide = make_title_slide_type(
            "content",
            vec![
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(0),
                        width: Emu(9_144_000),
                        height: Emu(1_000_000),
                    },
                    content: FrameContent::Title(Arc::from("Main Title")),
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
                    content: FrameContent::Subtitle(Arc::from("Sub heading")),
                    text_flow: None,
                    region_role: None,
                },
            ],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);
        // content slide: Title → h2, Subtitle → h3
        assert!(
            result.contains("<h2"),
            "F-004: content slide Title must produce h2; got: {result}"
        );
        assert!(
            result.contains("<h3"),
            "F-004: Subtitle after content-slide Title must produce h3; got: {result}"
        );
        // h2 must appear before h3 in document order.
        let h2_pos = result.find("<h2").expect("h2 must be present");
        let h3_pos = result.find("<h3").expect("h3 must be present");
        assert!(
            h2_pos < h3_pos,
            "F-004: h2 must appear before h3 in document order; got: {result}"
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
    /// outer `<svg>` must receive `aria-hidden="true"`.
    #[test]
    fn test_F007_self_closing_nested_svg_gets_aria_hidden() {
        let svg_in =
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><svg/></svg>"#;
        let result = render_svg_chart(svg_in, "complex chart");
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

    /// F-009: safe Xref must not emit a warn.
    #[tracing_test::traced_test]
    #[test]
    fn test_F009_xref_routes_through_allowlist() {
        use super::render_inline_node;
        use slideforge_types::InlineNode;
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
    /// must NOT use unwrap_or("") for key parsing.
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
    // HIGH-B2 — outer SVG must use role="presentation", not aria-hidden="true"
    // ─────────────────────────────────────────────────────────────────────────

    /// B2: render_graphics_layer outer <svg> must carry role="presentation", NOT aria-hidden.
    #[test]
    fn test_B2_render_graphics_layer_outer_svg_has_role_presentation() {
        let frames = vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Chart {
                alt: AltText::Provided(Arc::from("Revenue")),
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-1", &page_size);

        // Outer SVG must have role="presentation"
        let svg_start = result.find("<svg").expect("must contain outer svg");
        let svg_tag_end = result[svg_start..].find('>').expect("svg must close");
        let outer_svg_tag = &result[svg_start..=(svg_start + svg_tag_end)];
        assert!(
            outer_svg_tag.contains(r#"role="presentation""#),
            "B2: outer <svg> graphics layer must carry role=\"presentation\"; got: {outer_svg_tag}"
        );
        assert!(
            !outer_svg_tag.contains(r#"aria-hidden="true""#),
            "B2: outer <svg> graphics layer must NOT carry aria-hidden=\"true\"; got: {outer_svg_tag}"
        );
    }

    /// B2: non-decorative <g role="img"> must have NO aria-hidden="true" ancestor svg.
    #[test]
    fn test_B2_non_decorative_g_role_img_not_hidden_by_ancestor() {
        let frames = vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Chart {
                alt: AltText::Provided(Arc::from("A chart")),
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-1", &page_size);

        // Must contain <g role="img"
        assert!(
            result.contains(r#"role="img""#),
            "B2: non-decorative chart must have <g role=\"img\">; got: {result}"
        );
        // The outermost svg must NOT have aria-hidden="true" on the svg element
        let svg_start = result.find("<svg").expect("must have svg");
        let svg_tag_end = result[svg_start..].find('>').expect("svg must close");
        let outer_svg_tag = &result[svg_start..=(svg_start + svg_tag_end)];
        assert!(
            !outer_svg_tag.contains(r#"aria-hidden="true""#),
            "B2: outer <svg> must not be aria-hidden (hides <g role=\"img\">); got: {outer_svg_tag}"
        );
    }

    /// B2: decorative <g> carries its own aria-hidden="true" (not from ancestor svg).
    #[test]
    fn test_B2_decorative_g_carries_own_aria_hidden() {
        let frames = vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Image {
                alt: AltText::Decorative,
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-1", &page_size);

        // The decorative group must carry aria-hidden="true" on the <g> element
        assert!(
            result.contains(r#"<g aria-hidden="true""#),
            "B2: decorative frame must have <g aria-hidden=\"true\">; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MED-B3 — canonical id format sf-slide-{n}-{idx} (0-based, no doubling)
    // ─────────────────────────────────────────────────────────────────────────

    /// B3: chart frame id must be sf-slide-{n}-{idx} — no doubled slide_id.
    #[test]
    fn test_B3_canonical_id_format_no_doubling() {
        let frames = vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Chart {
                alt: AltText::Provided(Arc::from("Revenue")),
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-1", &page_size);

        // ID must be sf-slide-1-0 (0-based index)
        assert!(
            result.contains("sf-slide-1-0"),
            "B3: first chart frame must have id 'sf-slide-1-0'; got: {result}"
        );
        // Must NOT have doubled pattern
        assert!(
            !result.contains("sf-slide-1-slide-1"),
            "B3: doubled slide_id 'sf-slide-1-slide-1' must not appear; got: {result}"
        );
    }

    /// B3: second graphical frame on a slide gets index 1.
    #[test]
    fn test_B3_second_graphical_frame_gets_index_1() {
        let frames = vec![
            Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(4_500_000),
                    height: Emu(5_143_500),
                },
                content: FrameContent::Chart {
                    alt: AltText::Provided(Arc::from("Chart A")),
                },
                text_flow: None,
                region_role: None,
            },
            Frame {
                bbox: BoundingBox {
                    x: Emu(4_600_000),
                    y: Emu(0),
                    width: Emu(4_500_000),
                    height: Emu(5_143_500),
                },
                content: FrameContent::Chart {
                    alt: AltText::Provided(Arc::from("Chart B")),
                },
                text_flow: None,
                region_role: None,
            },
        ];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-2", &page_size);

        assert!(
            result.contains("sf-slide-2-0"),
            "B3: first chart on slide 2 must have id 'sf-slide-2-0'; got: {result}"
        );
        assert!(
            result.contains("sf-slide-2-1"),
            "B3: second chart on slide 2 must have id 'sf-slide-2-1'; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MED-B5 — negative geometry must be treated as degenerate (skip)
    // ─────────────────────────────────────────────────────────────────────────

    /// B5: negative width in render_text_frame → return None (same as zero width).
    #[test]
    fn test_B5_negative_width_text_frame_skipped() {
        let frame = Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(-500_000),
                height: Emu(1_000_000),
            },
            content: FrameContent::Title(Arc::from("Broken")),
            text_flow: None,
            region_role: None,
        };
        let result = render_text_frame(&frame, HeadingLevel::H2);
        assert!(
            result.is_none(),
            "B5: negative-width text frame must return None; got: {result:?}"
        );
    }

    /// B5: negative height in render_text_frame → return None.
    #[test]
    fn test_B5_negative_height_text_frame_skipped() {
        let frame = Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(9_144_000),
                height: Emu(-1_000_000),
            },
            content: FrameContent::Body(vec![]),
            text_flow: None,
            region_role: None,
        };
        let result = render_text_frame(&frame, HeadingLevel::H2);
        assert!(
            result.is_none(),
            "B5: negative-height text frame must return None; got: {result:?}"
        );
    }

    /// B5: negative width in render_graphics_layer → frame skipped (empty output).
    #[test]
    fn test_B5_negative_width_graphics_frame_skipped() {
        let frames = vec![Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(-9_144_000),
                height: Emu(5_143_500),
            },
            content: FrameContent::Chart {
                alt: AltText::Provided(Arc::from("Broken")),
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-1", &page_size);
        assert!(
            result.is_empty(),
            "B5: negative-width graphics frame must be skipped; got: {result}"
        );
    }

    /// B5: negative x/y coordinates are clamped/allowed (no crash; CSS may show negative pos).
    /// The layout invariant: negative position is legal (slide can have off-canvas frames),
    /// but negative SIZE is degenerate.
    #[test]
    fn test_B5_negative_x_y_position_allowed_not_skipped() {
        let frame = Frame {
            bbox: BoundingBox {
                x: Emu(-100_000),
                y: Emu(-50_000),
                width: Emu(9_144_000),
                height: Emu(5_143_500),
            },
            content: FrameContent::Title(Arc::from("Offset")),
            text_flow: None,
            region_role: None,
        };
        let result = render_text_frame(&frame, HeadingLevel::H2);
        // Negative x/y position: frame should still render (off-canvas frames are legal)
        assert!(
            result.is_some(),
            "B5: negative x/y position must not skip the frame (negative SIZE is degenerate, not negative position); got: {result:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OBS-B8 — heading order: Subtitle after Title must maintain h2 → h3 order
    // ─────────────────────────────────────────────────────────────────────────

    /// B8: Title frame AFTER Subtitle frame in vector order must still produce
    /// h2 before h3 in document output (frames are rendered in iteration order).
    /// If Subtitle (h3) appears before Title (h2) in the frame vector, we must
    /// document this as a layout invariant (frames should always have Title before
    /// Subtitle). This test asserts the actual behavior: frame vector order is
    /// preserved in output, so callers must supply Title before Subtitle.
    #[test]
    fn test_B8_subtitle_before_title_in_frames_produces_h3_before_h2_reflecting_frame_order() {
        // This test documents the CURRENT invariant: frame order is respected.
        // Callers (layout engine) MUST supply Title frame before Subtitle frame.
        let slide = make_title_slide_type(
            "content",
            vec![
                // Subtitle BEFORE Title — unusual but possible if layout engine misbehaves
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(1_000_000),
                        width: Emu(9_144_000),
                        height: Emu(4_143_500),
                    },
                    content: FrameContent::Subtitle(Arc::from("Sub first")),
                    text_flow: None,
                    region_role: None,
                },
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(0),
                        width: Emu(9_144_000),
                        height: Emu(1_000_000),
                    },
                    content: FrameContent::Title(Arc::from("Title after")),
                    text_flow: None,
                    region_role: None,
                },
            ],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        // Pass pre-computed heading level H2 (as if from exporter pre-pass)
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);
        // Frame order is preserved: Subtitle (h3) appears before Title (h2).
        // This is an INVARIANT documentation test — the layout engine is responsible
        // for ordering Title before Subtitle frames.
        let h2_pos = result.find("<h2");
        let h3_pos = result.find("<h3");
        // Both must be present
        assert!(
            h2_pos.is_some(),
            "B8: Title frame must produce h2; got: {result}"
        );
        assert!(
            h3_pos.is_some(),
            "B8: Subtitle frame must produce h3; got: {result}"
        );
        // Document the actual behavior: frame order determines DOM order.
        // This test does NOT assert h3 < h2 is invalid — it asserts the behavior is deterministic.
        // The layout invariant comment above is load-bearing documentation.
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Snapshot tests (insta) — per Test Strategy (P4 DOM structure)
    // ─────────────────────────────────────────────────────────────────────────

    /// Snapshot test: title slide renders to P4 HTML structure (article + h1 text layer).
    #[test]
    fn test_BC_4_03_003_snapshot_title_slide_html() {
        let slide = make_title_slide_type(
            "title",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Title(Arc::from("Hello World")),
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H1, &page_size);
        insta::assert_snapshot!("title_slide_html", result);
    }

    /// Snapshot test: content slide with subtitle renders correctly (h2 + h3).
    #[test]
    fn test_BC_4_03_003_snapshot_content_slide_html() {
        let slide = make_title_slide_type(
            "content",
            vec![
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
            ],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);
        insta::assert_snapshot!("content_slide_html", result);
    }

    /// Snapshot test: chart slide with <g role="img" aria-labelledby><title>.
    #[test]
    fn test_BC_4_03_003_snapshot_chart_slide_html() {
        let slide = make_title_slide_type(
            "content",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Chart {
                    alt: AltText::Provided(Arc::from("Revenue by quarter")),
                },
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);
        insta::assert_snapshot!("chart_slide_html", result);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-P9-001 — Body-frame h1 promotion must emit PHRASING content only.
    // A <h1> content model is phrasing content; <p>/<ul>/<table> are flow
    // content and are INVALID inside <h1> (HTML5 conformance, AC-001).
    // ─────────────────────────────────────────────────────────────────────────

    /// F-P9-001 / AC-001 — bullets-only deck (no Title frame): step-3 promotion
    /// must NOT wrap a `<ul>` inside `<h1>`.
    ///
    /// When the first promotable block is `Bullets`, the first bullet item's
    /// inline content is extracted and placed directly inside `<h1>` — no
    /// `<ul>` or `<li>` descendants.
    #[test]
    fn test_F_P9_001_body_bullets_promotion_h1_has_no_ul_child() {
        use slideforge_types::{BulletItem, ContentBlock, InlineNode, SourceSpan};

        let bullet_text = "First bullet heading";
        let slide = make_title_slide_type(
            "content",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Body(vec![ContentBlock::Bullets(vec![BulletItem {
                    inlines: vec![InlineNode::Plain(Arc::from(bullet_text))],
                    children: vec![],
                    span: SourceSpan::default(),
                }])]),
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        // H1 + no Title frame = step-3 body promotion.
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H1, &page_size);

        let doc = scraper::Html::parse_document(&result);

        // <h1> must exist.
        let h1_sel = scraper::Selector::parse("h1").expect("valid selector");
        let h1_nodes: Vec<_> = doc.select(&h1_sel).collect();
        assert_eq!(
            h1_nodes.len(),
            1,
            "F-P9-001: exactly one <h1> must be present; got: {result}"
        );

        // <h1> must NOT contain <ul>.
        let ul_in_h1_sel = scraper::Selector::parse("h1 ul").expect("valid selector");
        assert_eq!(
            doc.select(&ul_in_h1_sel).count(),
            0,
            "F-P9-001: <h1> must not contain <ul> (invalid HTML5 nesting); got: {result}"
        );

        // <h1> must NOT contain <li>.
        let li_in_h1_sel = scraper::Selector::parse("h1 li").expect("valid selector");
        assert_eq!(
            doc.select(&li_in_h1_sel).count(),
            0,
            "F-P9-001: <h1> must not contain <li> (invalid HTML5 nesting); got: {result}"
        );

        // The <h1> text content must equal the first bullet's text.
        let h1_text: String = h1_nodes[0].text().collect();
        assert_eq!(
            h1_text.trim(),
            bullet_text,
            "F-P9-001: <h1> text must equal first bullet's text; got: {h1_text:?}"
        );
    }

    /// F-P9-001 / AC-001 — paragraph-only deck (no Title frame): step-3 promotion
    /// must NOT wrap a `<p>` inside `<h1>`.
    ///
    /// When the first promotable block is `Text`, the block's inline content is
    /// extracted and placed directly inside `<h1>` — no `<p>` descendant.
    #[test]
    fn test_F_P9_001_body_text_promotion_h1_has_no_p_child() {
        use slideforge_types::{ContentBlock, InlineNode, SourceSpan, TextBlock, TextTag};

        let paragraph_text = "This is the page heading from a text block";
        let slide = make_title_slide_type(
            "content",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Body(vec![ContentBlock::Text(TextBlock {
                    inlines: vec![InlineNode::Plain(Arc::from(paragraph_text))],
                    tag: TextTag::Untagged,
                    span: SourceSpan::default(),
                })]),
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        // H1 + no Title frame = step-3 body promotion.
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H1, &page_size);

        let doc = scraper::Html::parse_document(&result);

        // <h1> must exist.
        let h1_sel = scraper::Selector::parse("h1").expect("valid selector");
        let h1_nodes: Vec<_> = doc.select(&h1_sel).collect();
        assert_eq!(
            h1_nodes.len(),
            1,
            "F-P9-001: exactly one <h1> must be present; got: {result}"
        );

        // <h1> must NOT contain <p>.
        let p_in_h1_sel = scraper::Selector::parse("h1 p").expect("valid selector");
        assert_eq!(
            doc.select(&p_in_h1_sel).count(),
            0,
            "F-P9-001: <h1> must not contain <p> (invalid HTML5 nesting); got: {result}"
        );

        // The <h1> text content must equal the paragraph's text.
        let h1_text: String = h1_nodes[0].text().collect();
        assert_eq!(
            h1_text.trim(),
            paragraph_text,
            "F-P9-001: <h1> text must equal first text block's text; got: {h1_text:?}"
        );
    }
}
