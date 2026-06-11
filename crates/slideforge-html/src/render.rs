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
//!   absolutely-positioned HTML element. Subtitle with Title on same slide renders
//!   as `<h{title_level+1}>` (one level below Title, e.g. Title=h1 → Subtitle=h2,
//!   Title=h2 → Subtitle=h3 — CRIT-1 fix). Subtitle without Title on same slide
//!   renders as `<p class="sf-subtitle">` (not any heading) to prevent heading-order
//!   skips (MED-1 / BC-4.03.003 invariant 4 / WCAG heading-order).
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
            // BC-3.05.001 PC-4: Footnote → <span role="note"> (not <small>).
            // role="note" conveys note semantics to assistive technology;
            // <small> is a generic presentational element with no semantic role.
            format!(
                r#"<span role="note">{}</span>"#,
                render_inline_nodes(children)
            )
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
/// - `FrameContent::Subtitle` (with Title on same slide) → `<h{level+1}>` (one level
///   below the slide Title heading; Title precedes it so no heading-level skip).
///   Examples: Title=h1 → Subtitle=h2; Title=h2 → Subtitle=h3. Capped at h6.
/// - `FrameContent::Subtitle` (WITHOUT Title on same slide) → `<p class="sf-subtitle">`
///   (no anchoring heading: emitting any `<hN>` here would create a heading skip when
///   the document's only h1 lives on a different slide — BC-4.03.003 postcondition 2
///   + invariant 4 / AC-007 / WCAG heading-order / MED-1 fix)
/// - `FrameContent::Body` → `<div class="sf-body">` containing content blocks
/// - `FrameContent::TextRun` → `<p class="sf-text">`
///
/// ## Heading-order invariant (CRIT-1 + MED-1)
///
/// The `has_title_frame` parameter carries whether the *same slide* (not the whole
/// document) contains at least one `FrameContent::Title(_)` frame. It is computed
/// by the caller (`render_slide_to_html`) before the frame iteration loop and
/// threaded in here so `render_text_frame` can make the correct Subtitle decision
/// without re-scanning frames.
///
/// Invariant: Subtitle is only a heading element when `has_title_frame == true`, and
/// its level is always exactly `heading_level + 1` (the slide's Title level + 1).
/// This guarantees no level skip regardless of whether the Title is h1 (title slide)
/// or h2 (content slide). When `has_title_frame == false`, Subtitle becomes
/// `<p class="sf-subtitle">`, which introduces no heading level and therefore cannot
/// create a heading skip.
///
/// The returned HTML element is absolutely positioned using inline CSS derived
/// from `frame.bbox` EMU coordinates converted to CSS pixels (MED-3).
///
/// Returns `None` for non-text frames (graphical frames go to the SVG layer).
#[must_use]
pub fn render_text_frame(
    frame: &Frame,
    heading_level: HeadingLevel,
    has_title_frame: bool,
) -> Option<String> {
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
            // CRIT-1 / MED-1 fix (BC-4.03.003 PC-7 + invariant 4 / AC-007 / WCAG heading-order):
            //
            // The Subtitle heading level MUST be exactly one level below the slide's own
            // Title frame heading level — NOT a hardcoded <h3>.
            //
            // Rationale: when Title = <h1> (title-type/cover slide), emitting <h3> for
            // Subtitle creates an h1→h3 skip with NO h2 between them — a WCAG
            // `heading-order` violation. The Pass-11 fix only handled Subtitle-WITHOUT-Title
            // (→ <p>); this symmetric case (Title+Subtitle at h1 level → h1→h3 skip)
            // was missed for 19 adversary passes.
            //
            // Correct mapping:
            //   Title = H1 (title slide)   → Subtitle = <h2>  (1+1=2)
            //   Title = H2 (content slide) → Subtitle = <h3>  (2+1=3)
            //   Title = H3                 → Subtitle = <h4>  (3+1=4, capped at 6)
            //   Title = H4                 → Subtitle = <h5>  (4+1=5, capped at 6)
            //
            // Without Title on the same slide: render as <p class="sf-subtitle"> (no heading),
            // which cannot cause any skip (MED-1 fix: preserved).
            //
            // Defensive cap: heading_level.as_u8() is at most 4 (HeadingLevel::H4 max),
            // so sub_level ≤ 5 which is already within the h1-h6 range. The cap at 6
            // documents the defensive intent and guards any future HeadingLevel variant
            // that could exceed H4.
            let escaped = html_escape::encode_text(text);
            if has_title_frame {
                // sub_level = title_level + 1, capped at h6.
                let sub_level = (heading_level.as_u8() + 1).min(6);
                Some(format!(
                    r#"<h{sub_level} class="sf-subtitle" style="{position_style}">{escaped}</h{sub_level}>"#
                ))
            } else {
                Some(format!(
                    r#"<p class="sf-subtitle" style="{position_style}">{escaped}</p>"#
                ))
            }
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
        // STORY-081 C3: SubtitleInlines carries rich inline structure.
        // Render like Subtitle but using render_inline_nodes for the content.
        FrameContent::SubtitleInlines(nodes) => {
            let inline_html = render_inline_nodes(nodes);
            if has_title_frame {
                let sub_level = (heading_level.as_u8() + 1).min(6);
                Some(format!(
                    r#"<h{sub_level} class="sf-subtitle" style="{position_style}">{inline_html}</h{sub_level}>"#
                ))
            } else {
                Some(format!(
                    r#"<p class="sf-subtitle" style="{position_style}">{inline_html}</p>"#
                ))
            }
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

/// Emit an SVG fragment for a `FrameContent::Shape` frame.
///
/// Handles both gradient (`FillSpec::Gradient`) and non-gradient fills. Extracted
/// to keep [`render_graphics_layer`] within the `clippy::too_many_lines` limit.
///
/// ## AC-004 (STORY-072)
///
/// `FillSpec::Gradient` → SVG-native `<defs><linearGradient id="sf-grad-{slide_id}-{grad_idx}">...
/// </linearGradient></defs>` followed by `<rect fill="url(#sf-grad-...)"/>`.
/// CSS `background` is NOT used — it has no effect on SVG geometry elements.
/// The gradient flows top-to-bottom (`x1="0" y1="0" x2="0" y2="1"`) matching
/// PPTX ang=5400000 and the `to bottom` direction used by `css_linear_gradient_background`.
///
/// `FillSpec::SolidColor` → SVG `fill="#RRGGBB"` attribute.
/// `FillSpec::None` → SVG `fill="none"` attribute.
///
/// Called by [`render_graphics_layer`] for `FrameContent::Shape` frames.
/// `grad_idx` is incremented each time a gradient def is emitted so multiple
/// gradient shapes on the same slide receive distinct `<linearGradient>` ids.
fn render_shape_svg(
    out: &mut String,
    frame_idx: &mut u32,
    grad_idx: &mut u32,
    slide_id: &str,
    alt: &AltText,
    fill: &slideforge_layout::FillSpec,
    bbox: (i64, i64, i64, i64), // (x, y, w, h) in EMU
) {
    use std::fmt::Write as _;
    let (x, y, w, h) = bbox;

    // For gradient fills, emit a <defs> block BEFORE the <g> so the
    // gradient def is available when the <rect> references it.
    // SVG spec allows <defs> anywhere within an <svg> element.
    let fill_attr = match fill {
        slideforge_layout::FillSpec::Gradient { from, to } => {
            let gid = format!("sf-grad-{slide_id}-{grad_idx}");
            *grad_idx += 1;
            let escaped_gid = html_escape::encode_double_quoted_attribute(&gid);
            let _ = write!(
                out,
                "<defs><linearGradient id=\"{escaped_gid}\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\">\
<stop offset=\"0\" stop-color=\"#{:02X}{:02X}{:02X}\"/>\
<stop offset=\"1\" stop-color=\"#{:02X}{:02X}{:02X}\"/>\
</linearGradient></defs>",
                from.r, from.g, from.b, to.r, to.g, to.b
            );
            format!("fill=\"url(#{escaped_gid})\"")
        },
        slideforge_layout::FillSpec::SolidColor(rgb) => {
            format!("fill=\"#{:02X}{:02X}{:02X}\"", rgb.r, rgb.g, rgb.b)
        },
        slideforge_layout::FillSpec::None => "fill=\"none\"".to_owned(),
    };
    match alt {
        AltText::Provided(text) => {
            let idx = frame_idx.to_string();
            *frame_idx += 1;
            let label_id = format!("sf-{slide_id}-{idx}");
            let eid = html_escape::encode_double_quoted_attribute(&label_id);
            let ealt = html_escape::encode_text(text);
            let _ = write!(
                out,
                r#"<g role="img" aria-labelledby="{eid}"><title id="{eid}">{ealt}</title><rect x="{x}" y="{y}" width="{w}" height="{h}" {fill_attr}/></g>"#
            );
        },
        AltText::Decorative | AltText::Unspecified => {
            let _ = write!(
                out,
                r#"<g aria-hidden="true"><rect x="{x}" y="{y}" width="{w}" height="{h}" {fill_attr}/></g>"#
            );
        },
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
// STORY-081 C3 added SubtitleInlines arm (3 lines) pushing past the 150-line threshold.
// The function body is a single exhaustive match over FrameContent variants with no
// hidden complexity — extracting sub-functions would scatter the variant logic.
#[allow(clippy::too_many_lines)]
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
    // grad_idx is a per-slide counter for linearGradient def ids (AC-004 STORY-072).
    // Incremented each time a gradient shape is emitted so multiple gradient shapes
    // on one slide get distinct ids: sf-grad-{slide_id}-0, sf-grad-{slide_id}-1, ...
    let mut grad_idx: u32 = 0;

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
                // HIGH-1: branch on alt — decorative/unspecified frames MUST NOT emit
                // role="img" with an empty <title> (WCAG 1.1.1 / axe-core svg-img-alt).
                // BC-4.03.003 PC-6: role="img" is scoped to NON-decorative frames only.
                let placeholder_svg =
                    r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"></svg>"#;
                match alt {
                    AltText::Provided(text) => {
                        // MED-B3: frame_id is the 0-based index string; render_chart_frame
                        // builds the full label as sf-{slide_id}-{frame_id}.
                        let frame_id_str = frame_idx.to_string();
                        frame_idx += 1;
                        // WHEN real chart SVG is added (STORY-047/048) it MUST route through
                        // render_chart_frame (which applies the SVG sanitizer) — LOW-2.
                        let chart_g = render_chart_frame(
                            placeholder_svg,
                            text.as_ref(),
                            &frame_id_str,
                            slide_id,
                        );
                        let _ = write!(
                            graphical_content,
                            r#"<g transform="translate({x} {y})">{chart_g}</g>"#
                        );
                    },
                    AltText::Decorative | AltText::Unspecified => {
                        // Decorative/unspecified: aria-hidden="true" on the <g>, NO role="img",
                        // NO empty <title>. frame_idx NOT consumed (mirrors Image/Shape pattern).
                        // SVG still routes through sanitize_svg_for_graphics_layer so
                        // script/foreignObject/on*/javascript: stripping is preserved (HIGH-1).
                        let sanitized = sanitize_svg_for_graphics_layer(placeholder_svg);
                        let _ = write!(
                            graphical_content,
                            r#"<g aria-hidden="true"><g transform="translate({x} {y})">{sanitized}</g></g>"#
                        );
                    },
                }
            },
            FrameContent::Diagram { svg, alt } => {
                // HIGH-1: branch on alt — decorative/unspecified frames MUST NOT emit
                // role="img" with an empty <title> (WCAG 1.1.1 / axe-core svg-img-alt).
                // BC-4.03.003 PC-6: role="img" is scoped to NON-decorative frames only.
                match alt {
                    AltText::Provided(text) => {
                        let frame_id_str = frame_idx.to_string();
                        frame_idx += 1;
                        // WHEN real diagram SVG is wired in (STORY-048) it MUST route through
                        // render_chart_frame (which applies the SVG sanitizer) — LOW-2.
                        let chart_g = render_chart_frame(
                            svg.as_str(),
                            text.as_ref(),
                            &frame_id_str,
                            slide_id,
                        );
                        let _ = write!(
                            graphical_content,
                            r#"<g transform="translate({x} {y})">{chart_g}</g>"#
                        );
                    },
                    AltText::Decorative | AltText::Unspecified => {
                        // Decorative/unspecified: aria-hidden="true" on the <g>, NO role="img",
                        // NO empty <title>. frame_idx NOT consumed (mirrors Image/Shape pattern).
                        // SVG still routes through sanitize_svg_for_graphics_layer so
                        // script/foreignObject/on*/javascript: stripping is preserved (HIGH-1).
                        let sanitized = sanitize_svg_for_graphics_layer(svg.as_str());
                        let _ = write!(
                            graphical_content,
                            r#"<g aria-hidden="true"><g transform="translate({x} {y})">{sanitized}</g></g>"#
                        );
                    },
                }
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
                render_shape_svg(
                    &mut graphical_content,
                    &mut frame_idx,
                    &mut grad_idx,
                    slide_id,
                    &shape_frame.alt,
                    &shape_frame.fill,
                    (x, y, w, h),
                );
            },
            FrameContent::ColorBar {
                filled_width_emu,
                total_width_emu: _,
                percent,
                color,
                alt: _, // STORY-095: alt is used by PDF exporter only; HTML uses aria-label
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
            // Text frames (Title, Subtitle, SubtitleInlines, Body, TextRun) go to the HTML text layer.
            FrameContent::Title(_)
            | FrameContent::Subtitle(_)
            // STORY-081 C3: SubtitleInlines is a text frame — goes to the text layer.
            | FrameContent::SubtitleInlines(_)
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
/// Render a single slide to HTML with an optional title inline override.
///
/// When `title_inlines_override` is `Some(nodes)`, the `FrameContent::Title`
/// arm renders via `render_inline_nodes` instead of plain escaped text.
/// This is used by the HTML exporter (STORY-081 I2) to wire the `title_inlines`
/// shadow field from the semantic deck for DOCX/HTML/PDF rich-title rendering.
///
/// When `title_inlines_override` is `None`, rendering is identical to
/// [`render_slide_to_html`] (plain-text title).
#[must_use]
pub(crate) fn render_slide_to_html_with_title_override(
    slide: &LaidOutSlide,
    brand: &Brand,
    heading_level: HeadingLevel,
    page_size: &PageSize,
    title_inlines_override: Option<&[InlineNode]>,
) -> String {
    let _ = brand; // Brand used by future template-driven color/font injection.

    // MED-3: derive slide container dimensions from page_size (not hardcoded consts).
    let container_w = emu_to_css_px(page_size.width);
    let container_h = emu_to_css_px(page_size.height);

    // Slide ID for ARIA cross-references (1-based from source_index).
    // OBS-2: "Slide N" is English-only; see doc comment above.
    let slide_number = slide.source_index + 1;
    let slide_id = format!("slide-{slide_number}");

    // STORY-081 I2: if title_inlines_override is Some, intercept Title frame rendering
    // inside render_slide_to_html_inner.
    render_slide_to_html_inner(
        slide,
        heading_level,
        page_size,
        &slide_id,
        slide_number,
        &container_w,
        &container_h,
        title_inlines_override,
    )
}

/// Render a single laid-out slide to an HTML fragment.
///
/// Uses `render_slide_to_html_inner` with no title-inlines override.
/// For rich title rendering (STORY-081 I2), the HTML exporter uses an
/// internal variant that accepts a `title_inlines_override` parameter.
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

    render_slide_to_html_inner(
        slide,
        heading_level,
        page_size,
        &slide_id,
        slide_number,
        &container_w,
        &container_h,
        None, // no title_inlines override for backward-compat callers
    )
}

/// Inner implementation for slide rendering, shared by `render_slide_to_html`
/// and `render_slide_to_html_with_title_override`.
#[allow(clippy::too_many_arguments)]
fn render_slide_to_html_inner(
    slide: &LaidOutSlide,
    heading_level: HeadingLevel,
    page_size: &PageSize,
    slide_id: &str,
    slide_number: usize,
    container_w: &str,
    container_h: &str,
    title_inlines_override: Option<&[InlineNode]>,
) -> String {
    use std::fmt::Write as _;

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
    // Pass-14 / CRIT-1: has_title_frame must use the SAME non-degenerate-bbox guard
    // as compute_heading_levels steps 1/2 (TD-VSDD-060 / LESSON-19).
    // A degenerate Title frame is invisible (render_text_frame returns None for it);
    // counting it as a "title frame present" would incorrectly gate Subtitle as <h3>
    // and suppress body-promotion, both inconsistent with what the render loop emits.
    let has_title_frame = slide
        .frames
        .iter()
        .any(|f| matches!(f.content, FrameContent::Title(_)) && is_non_degenerate_bbox(&f.bbox));
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

        // STORY-081 I2: if this is a Title frame AND title_inlines_override is Some,
        // render with rich inline content instead of plain escaped text.
        if matches!(frame.content, FrameContent::Title(_))
            && is_non_degenerate_bbox(&frame.bbox)
            && let Some(inlines) = title_inlines_override
            && !inlines.is_empty()
        {
            let x = emu_to_css_px(frame.bbox.x);
            let y = emu_to_css_px(frame.bbox.y);
            let w = emu_to_css_px(frame.bbox.width);
            let h = emu_to_css_px(frame.bbox.height);
            let position_style = format!(
                "position:absolute; left:{x}px; top:{y}px; width:{w}px; height:{h}px; overflow:hidden;"
            );
            let hl = heading_level.as_u8();
            let inline_html = render_inline_nodes(inlines);
            let _ = write!(
                text_layer,
                r#"<h{hl} class="sf-title" style="{position_style}">{inline_html}</h{hl}>"#
            );
            text_layer.push('\n');
            continue;
        }

        if let Some(html) = render_text_frame(frame, heading_level, has_title_frame) {
            text_layer.push_str(&html);
            text_layer.push('\n');
        }
    }

    // Build the SVG graphics layer (graphical frames only).
    let svg_layer = render_graphics_layer(&slide.frames, slide_id, page_size);

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
// STORY-072: FillSpec::Gradient — CSS linear-gradient helper (utility / test surface)
// ─────────────────────────────────────────────────────────────────────────────

/// Produce the CSS `background` property value string for a linear gradient.
///
/// ## Note (STORY-072 adv-P2 HIGH-001)
///
/// This function is NOT called by `render_shape_svg`. SVG `<rect>` elements
/// are SVG geometry nodes — CSS `background` has no effect on them. Gradient
/// fills on shapes are rendered via SVG-native `<linearGradient>` defs +
/// `fill="url(#...)"` (see `render_shape_svg`).
///
/// This function is retained as a public utility for callers that render
/// gradients onto CSS box-model elements (e.g. HTML `<div>` overlays in
/// non-SVG rendering paths) and as a stable test surface for the CSS
/// gradient format contract.
///
/// ## Contract
///
/// Returns a CSS string of the form:
/// `linear-gradient(to bottom, #RRGGBB, #RRGGBB)`
///
/// - Direction is fixed as `to bottom` (top-to-bottom, v1.0).
/// - Both hex values are uppercase 6-digit (`#RRGGBB`).
///
/// ## Examples
///
/// ```
/// use slideforge_html::render::css_linear_gradient_background;
/// use slideforge_types::Rgb;
/// let css = css_linear_gradient_background(
///     Rgb { r: 255, g: 0, b: 0 },
///     Rgb { r: 0, g: 0, b: 255 },
/// );
/// assert_eq!(css, "linear-gradient(to bottom, #FF0000, #0000FF)");
/// ```
#[must_use]
pub fn css_linear_gradient_background(
    from: slideforge_types::Rgb,
    to: slideforge_types::Rgb,
) -> String {
    format!(
        "linear-gradient(to bottom, #{:02X}{:02X}{:02X}, #{:02X}{:02X}{:02X})",
        from.r, from.g, from.b, to.r, to.g, to.b
    )
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
                font_size_emu: 457_200,
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
        // has_title_frame=true: irrelevant (returns None before reaching Subtitle arm).
        let result = render_text_frame(&frame, HeadingLevel::H1, true);
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
        // has_title_frame=true: Title frame; has_title_frame does not affect Title arm.
        let result = render_text_frame(&frame, HeadingLevel::H1, true);
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
        // has_title_frame=false: testing Body frame in isolation; has_title_frame
        // does not affect the Body arm.
        let result =
            render_text_frame(&frame, HeadingLevel::H2, false).expect("body frame must render");
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
        // has_title_frame=false: testing Body frame in isolation; has_title_frame
        // does not affect the Body arm.
        let result =
            render_text_frame(&frame, HeadingLevel::H2, false).expect("body frame must render");
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
        // has_title_frame=true: this test models the case where the slide has a Title
        // frame; Subtitle with Title on same slide must render as <h3>.
        let result =
            render_text_frame(&frame, HeadingLevel::H2, true).expect("subtitle must render");
        assert!(
            result.contains("<h3"),
            "F-004: Subtitle frame with Title on same slide must render as <h3>; got: {result}"
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
    // HIGH-1 — decorative/unspecified Chart + Diagram must use aria-hidden, not
    // empty role="img" (WCAG 1.1.1 / axe-core svg-img-alt / BC-4.03.003 PC-6).
    //
    // These tests cover every combination of {Chart,Diagram} × {Decorative,
    // Unspecified} and verify:
    //   - NO role="img" emitted
    //   - NO <title> element emitted
    //   - <g aria-hidden="true"> IS emitted
    //   - SVG still routes through the sanitizer (script tags stripped)
    //   - frame_idx NOT consumed for decorative frames
    //   - Provided-alt Chart/Diagram regression: role="img" + <title> still present
    // ─────────────────────────────────────────────────────────────────────────

    /// HIGH-1: decorative Chart emits aria-hidden, no role="img", no empty <title>.
    #[test]
    fn test_HIGH1_decorative_chart_uses_aria_hidden_not_role_img() {
        let frames = vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Chart {
                alt: AltText::Decorative,
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-dec", &page_size);

        assert!(
            result.contains(r#"aria-hidden="true""#),
            "HIGH-1: decorative Chart must emit aria-hidden=\"true\"; got: {result}"
        );
        assert!(
            !result.contains(r#"role="img""#),
            "HIGH-1: decorative Chart must NOT emit role=\"img\"; got: {result}"
        );
        assert!(
            !result.contains("<title>"),
            "HIGH-1: decorative Chart must NOT emit <title>; got: {result}"
        );
    }

    /// HIGH-1: unspecified-alt Chart emits aria-hidden, no role="img", no empty <title>.
    #[test]
    fn test_HIGH1_unspecified_chart_uses_aria_hidden_not_role_img() {
        let frames = vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Chart {
                alt: AltText::Unspecified,
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-unspec", &page_size);

        assert!(
            result.contains(r#"aria-hidden="true""#),
            "HIGH-1: unspecified Chart must emit aria-hidden=\"true\"; got: {result}"
        );
        assert!(
            !result.contains(r#"role="img""#),
            "HIGH-1: unspecified Chart must NOT emit role=\"img\"; got: {result}"
        );
        assert!(
            !result.contains("<title>"),
            "HIGH-1: unspecified Chart must NOT emit <title>; got: {result}"
        );
    }

    /// HIGH-1: decorative Diagram emits aria-hidden, no role="img", no empty <title>.
    #[test]
    fn test_HIGH1_decorative_diagram_uses_aria_hidden_not_role_img() {
        use slideforge_types::NormalizedDiagramSvg;
        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100"></svg>"#,
        ));
        let frames = vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Diagram {
                svg: normalized,
                alt: AltText::Decorative,
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-dec-diag", &page_size);

        assert!(
            result.contains(r#"aria-hidden="true""#),
            "HIGH-1: decorative Diagram must emit aria-hidden=\"true\"; got: {result}"
        );
        assert!(
            !result.contains(r#"role="img""#),
            "HIGH-1: decorative Diagram must NOT emit role=\"img\"; got: {result}"
        );
        assert!(
            !result.contains("<title>"),
            "HIGH-1: decorative Diagram must NOT emit <title>; got: {result}"
        );
    }

    /// HIGH-1: unspecified-alt Diagram emits aria-hidden, no role="img", no empty <title>.
    #[test]
    fn test_HIGH1_unspecified_diagram_uses_aria_hidden_not_role_img() {
        use slideforge_types::NormalizedDiagramSvg;
        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100"></svg>"#,
        ));
        let frames = vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Diagram {
                svg: normalized,
                alt: AltText::Unspecified,
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-unspec-diag", &page_size);

        assert!(
            result.contains(r#"aria-hidden="true""#),
            "HIGH-1: unspecified Diagram must emit aria-hidden=\"true\"; got: {result}"
        );
        assert!(
            !result.contains(r#"role="img""#),
            "HIGH-1: unspecified Diagram must NOT emit role=\"img\"; got: {result}"
        );
        assert!(
            !result.contains("<title>"),
            "HIGH-1: unspecified Diagram must NOT emit <title>; got: {result}"
        );
    }

    /// HIGH-1 (regression): Provided-alt Chart still emits role="img" + non-empty <title>.
    #[test]
    fn test_HIGH1_provided_alt_chart_regression_still_has_role_img_and_title() {
        use scraper::{Html, Selector};
        let frames = vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Chart {
                alt: AltText::Provided(Arc::from("Revenue chart")),
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-prov-chart", &page_size);

        let doc = Html::parse_fragment(&result);
        let g_sel = Selector::parse(r#"g[role="img"]"#).expect("valid selector");
        let title_sel = Selector::parse("title").expect("valid selector");

        let g_nodes: Vec<_> = doc.select(&g_sel).collect();
        assert_eq!(
            g_nodes.len(),
            1,
            "HIGH-1 regression: provided-alt Chart must have exactly one <g role=\"img\">; got: {result}"
        );
        let title_nodes: Vec<_> = doc.select(&title_sel).collect();
        assert_eq!(
            title_nodes.len(),
            1,
            "HIGH-1 regression: provided-alt Chart must have exactly one <title>; got: {result}"
        );
        let title_text: String = title_nodes[0].text().collect();
        assert!(
            !title_text.trim().is_empty(),
            "HIGH-1 regression: provided-alt Chart <title> must be non-empty; got: {result}"
        );
        assert_eq!(
            title_text.trim(),
            "Revenue chart",
            "HIGH-1 regression: Chart <title> must equal the provided alt text; got: {title_text:?}"
        );
    }

    /// HIGH-1 (regression): Provided-alt Diagram still emits role="img" + non-empty <title>.
    #[test]
    fn test_HIGH1_provided_alt_diagram_regression_still_has_role_img_and_title() {
        use scraper::{Html, Selector};
        use slideforge_types::NormalizedDiagramSvg;
        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100"></svg>"#,
        ));
        let frames = vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Diagram {
                svg: normalized,
                alt: AltText::Provided(Arc::from("Architecture diagram")),
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-prov-diag", &page_size);

        let doc = Html::parse_fragment(&result);
        let g_sel = Selector::parse(r#"g[role="img"]"#).expect("valid selector");
        let title_sel = Selector::parse("title").expect("valid selector");

        let g_nodes: Vec<_> = doc.select(&g_sel).collect();
        assert_eq!(
            g_nodes.len(),
            1,
            "HIGH-1 regression: provided-alt Diagram must have exactly one <g role=\"img\">; got: {result}"
        );
        let title_nodes: Vec<_> = doc.select(&title_sel).collect();
        assert_eq!(
            title_nodes.len(),
            1,
            "HIGH-1 regression: provided-alt Diagram must have exactly one <title>; got: {result}"
        );
        let title_text: String = title_nodes[0].text().collect();
        assert_eq!(
            title_text.trim(),
            "Architecture diagram",
            "HIGH-1 regression: Diagram <title> must equal the provided alt text; got: {title_text:?}"
        );
    }

    /// HIGH-1 (sanitizer): decorative Chart with embedded <script> — script is stripped.
    ///
    /// The decorative path MUST still route through sanitize_svg_for_graphics_layer
    /// so XSS vectors are removed even when the frame is aria-hidden.
    #[test]
    fn test_HIGH1_decorative_chart_svg_still_sanitized_script_stripped() {
        // The Chart arm uses a placeholder SVG — but we verify the Diagram arm
        // with an attacker-controlled SVG containing a <script> tag to confirm
        // sanitize_svg_for_graphics_layer is invoked on the decorative path.
        use slideforge_types::NormalizedDiagramSvg;
        let malicious_svg = Arc::from(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script><rect width="100" height="50"/></svg>"#,
        );
        let normalized = NormalizedDiagramSvg::from_normalized_string(malicious_svg);
        let frames = vec![Frame {
            bbox: make_bbox_full(),
            content: FrameContent::Diagram {
                svg: normalized,
                alt: AltText::Decorative,
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-xss", &page_size);

        // Must be aria-hidden (decorative path)
        assert!(
            result.contains(r#"aria-hidden="true""#),
            "HIGH-1 sanitizer: decorative Diagram must be aria-hidden; got: {result}"
        );
        // <script> must be stripped even on the decorative path
        assert!(
            !result.to_lowercase().contains("<script"),
            "HIGH-1 sanitizer: <script> must be stripped even in decorative/aria-hidden path; got: {result}"
        );
    }

    /// HIGH-1 (frame_idx): decorative Chart does NOT consume a frame_idx slot.
    ///
    /// A provided-alt Chart after a decorative Chart must still get frame_idx=0,
    /// not frame_idx=1 (decorative frames mirror Image/Shape pattern — no id assigned).
    #[test]
    fn test_HIGH1_decorative_chart_does_not_consume_frame_idx() {
        let frames = vec![
            Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(4_500_000),
                    height: Emu(5_143_500),
                },
                content: FrameContent::Chart {
                    alt: AltText::Decorative,
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
                    alt: AltText::Provided(Arc::from("The real chart")),
                },
                text_flow: None,
                region_role: None,
            },
        ];
        let page_size = make_page_size();
        let result = render_graphics_layer(&frames, "slide-idx", &page_size);

        // The provided-alt chart is the FIRST non-decorative frame → frame_idx = 0.
        assert!(
            result.contains("sf-slide-idx-0"),
            "HIGH-1 frame_idx: first non-decorative Chart after a decorative one must get frame_idx=0; got: {result}"
        );
        // frame_idx=1 must NOT appear (would indicate decorative frame consumed an index).
        assert!(
            !result.contains("sf-slide-idx-1"),
            "HIGH-1 frame_idx: decorative Chart must NOT consume a frame_idx slot; got: {result}"
        );
    }

    /// HIGH-1 (invariant): no graphical arm may emit role="img" with an empty <title>.
    ///
    /// This sweeps ALL non-text FrameContent variants with Provided alt to verify
    /// the accessible name is always non-empty when role="img" is present.
    #[test]
    fn test_HIGH1_invariant_no_role_img_with_empty_title_for_any_arm() {
        use scraper::{Html, Selector};
        use slideforge_layout::{FillSpec, ShapeFrame, ShapeType};
        use slideforge_types::NormalizedDiagramSvg;

        let normalized_svg = NormalizedDiagramSvg::from_normalized_string(Arc::from(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"></svg>"#,
        ));

        let graphical_frames: Vec<Frame> = vec![
            Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Chart {
                    alt: AltText::Provided(Arc::from("Chart alt")),
                },
                text_flow: None,
                region_role: None,
            },
            Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Diagram {
                    svg: normalized_svg,
                    alt: AltText::Provided(Arc::from("Diagram alt")),
                },
                text_flow: None,
                region_role: None,
            },
            Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Image {
                    alt: AltText::Provided(Arc::from("Image alt")),
                },
                text_flow: None,
                region_role: None,
            },
            Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Shape(ShapeFrame {
                    shape_type: ShapeType::Rect,
                    fill: FillSpec::None,
                    text: None,
                    alt: AltText::Provided(Arc::from("Shape alt")),
                }),
                text_flow: None,
                region_role: None,
            },
        ];

        let page_size = make_page_size();
        let result = render_graphics_layer(&graphical_frames, "slide-inv", &page_size);

        let doc = Html::parse_fragment(&result);
        let title_sel = Selector::parse("title").expect("valid selector");

        for title_node in doc.select(&title_sel) {
            let title_text: String = title_node.text().collect();
            assert!(
                !title_text.trim().is_empty(),
                "HIGH-1 invariant: every <title> inside a role=\"img\" element must be non-empty; got empty title in: {result}"
            );
        }
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
        // has_title_frame=true: irrelevant (returns None on degenerate bbox before Subtitle arm).
        let result = render_text_frame(&frame, HeadingLevel::H2, true);
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
        // has_title_frame=false: irrelevant (returns None on degenerate bbox before Subtitle arm).
        let result = render_text_frame(&frame, HeadingLevel::H2, false);
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
        // has_title_frame=true: Title frame; has_title_frame does not affect Title arm.
        let result = render_text_frame(&frame, HeadingLevel::H2, true);
        // Negative x/y position: frame should still render (off-canvas frames are legal)
        assert!(
            result.is_some(),
            "B5: negative x/y position must not skip the frame (negative SIZE is degenerate, not negative position); got: {result:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MED-1 — heading-order: Subtitle without Title must NOT emit <h3>
    // (BC-4.03.003 postcondition 2 + invariant 4 / AC-007 / WCAG heading-order)
    // ─────────────────────────────────────────────────────────────────────────

    /// MED-1: a slide with ONLY a Subtitle frame (no Title frame) must NOT emit
    /// `<h3>` — that would create an h1→h3 heading skip (axe-core `heading-order`
    /// violation). The Subtitle must render as `<p class="sf-subtitle">` when there
    /// is no Title frame on the same slide.
    #[test]
    fn test_MED1_subtitle_without_title_renders_as_p_not_h3() {
        let slide = make_title_slide_type(
            "title",
            vec![Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(5_143_500),
                },
                content: FrameContent::Subtitle(Arc::from("Standalone subtitle")),
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        // H2 slide heading, no Title frame — heading level does NOT matter for this rule.
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);

        let doc = scraper::Html::parse_document(&result);

        // Must NOT contain <h3>.
        let h3_sel = scraper::Selector::parse("h3").expect("valid selector");
        assert_eq!(
            doc.select(&h3_sel).count(),
            0,
            "MED-1: Subtitle without Title must NOT render as <h3>; got: {result}"
        );

        // Must contain <p class="sf-subtitle">.
        let p_sel = scraper::Selector::parse("p.sf-subtitle").expect("valid selector");
        assert_eq!(
            doc.select(&p_sel).count(),
            1,
            "MED-1: Subtitle without Title must render as <p class=\"sf-subtitle\">; got: {result}"
        );

        // The <p class="sf-subtitle"> text must equal the subtitle text.
        let p_node = doc.select(&p_sel).next().expect("p.sf-subtitle must exist");
        let p_text: String = p_node.text().collect();
        assert_eq!(
            p_text.trim(),
            "Standalone subtitle",
            "MED-1: <p class=\"sf-subtitle\"> text must match subtitle content; got: {p_text:?}"
        );
    }

    /// CRIT-1: Title-type slide (Title=h1) + Subtitle must emit h1 → h2 (NOT h1 → h3).
    ///
    /// This is the CRIT-1 defect: the Subtitle arm previously hardcoded `<h3>` when
    /// `has_title_frame == true`, regardless of whether the Title was h1 or h2.
    ///
    /// On a title-type slide (heading_level = H1):
    ///   - Title → `<h1>`
    ///   - Subtitle → `<h2>` (one level below Title)
    ///
    /// Previously emitting `<h3>` here created an h1→h3 skip (no h2 between them),
    /// violating WCAG `heading-order`, BC-4.03.003 PC-7, and ADR-008 heading-rule table.
    ///
    /// Blocked 19 adversary passes because no test exercised Title+Subtitle at H1 level.
    #[test]
    fn test_CRIT1_title_slide_with_subtitle_emits_h1_then_h2_not_h3() {
        let slide = make_title_slide_type(
            "title",
            vec![
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(0),
                        width: Emu(9_144_000),
                        height: Emu(1_500_000),
                    },
                    content: FrameContent::Title(Arc::from("The Main Title")),
                    text_flow: None,
                    region_role: None,
                },
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(1_500_000),
                        width: Emu(9_144_000),
                        height: Emu(3_643_500),
                    },
                    content: FrameContent::Subtitle(Arc::from("A descriptive subtitle")),
                    text_flow: None,
                    region_role: None,
                },
            ],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        // H1 level: this is the title-type (cover) slide.
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H1, &page_size);

        let doc = scraper::Html::parse_document(&result);

        // Must have exactly one <h1> (Title).
        let h1_sel = scraper::Selector::parse("h1").expect("valid selector");
        assert_eq!(
            doc.select(&h1_sel).count(),
            1,
            "CRIT-1: title-type slide must have exactly one h1; got: {result}"
        );

        // Must have exactly one <h2> (Subtitle — one level below h1 Title).
        let h2_sel = scraper::Selector::parse("h2").expect("valid selector");
        assert_eq!(
            doc.select(&h2_sel).count(),
            1,
            "CRIT-1: Subtitle after h1 Title must produce <h2>, not <h3>; got: {result}"
        );

        // Must NOT have <h3> (h1→h3 is a heading-level skip → WCAG violation).
        let h3_sel = scraper::Selector::parse("h3").expect("valid selector");
        assert_eq!(
            doc.select(&h3_sel).count(),
            0,
            "CRIT-1: <h3> must NOT appear on a title+subtitle (h1) slide (h1→h3 skip); got: {result}"
        );

        // Must NOT have <p class="sf-subtitle"> (there IS a Title frame present).
        let p_sel = scraper::Selector::parse("p.sf-subtitle").expect("valid selector");
        assert_eq!(
            doc.select(&p_sel).count(),
            0,
            "CRIT-1: Subtitle with Title present must render as heading, not <p>; got: {result}"
        );

        // h1 must appear before h2 in document order.
        let h1_pos = result.find("<h1").expect("<h1> must be present");
        let h2_pos = result.find("<h2").expect("<h2> must be present");
        assert!(
            h1_pos < h2_pos,
            "CRIT-1: h1 must appear before h2 in document order; got: {result}"
        );
    }

    /// CRIT-1 (regression for content slide): Title+Subtitle at H2 must still emit h2 → h3.
    ///
    /// After the CRIT-1 fix (Subtitle level = Title level + 1):
    ///   - Title H2 → `<h2>`, Subtitle → `<h3>` (2+1=3). No change for content slides.
    #[test]
    fn test_CRIT1_content_slide_with_subtitle_still_emits_h2_then_h3() {
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
                    content: FrameContent::Title(Arc::from("Content Heading")),
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
                    content: FrameContent::Subtitle(Arc::from("Content subtitle")),
                    text_flow: None,
                    region_role: None,
                },
            ],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        // H2 level: content slide.
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);

        let doc = scraper::Html::parse_document(&result);

        // Must have h2 (Title) and h3 (Subtitle).
        let h2_sel = scraper::Selector::parse("h2").expect("valid selector");
        assert_eq!(
            doc.select(&h2_sel).count(),
            1,
            "CRIT-1 regression: content slide Title must render as <h2>; got: {result}"
        );
        let h3_sel = scraper::Selector::parse("h3").expect("valid selector");
        assert_eq!(
            doc.select(&h3_sel).count(),
            1,
            "CRIT-1 regression: content slide Subtitle after h2 must render as <h3>; got: {result}"
        );

        // h2 before h3 in document order.
        let h2_pos = result.find("<h2").expect("<h2> must be present");
        let h3_pos = result.find("<h3").expect("<h3> must be present");
        assert!(
            h2_pos < h3_pos,
            "CRIT-1 regression: h2 must precede h3; got: {result}"
        );
    }

    /// MED-1 (regression): Title + Subtitle slide must STILL render h2 → h3.
    /// This is the existing correct behavior — must not regress.
    #[test]
    fn test_MED1_subtitle_with_title_still_renders_h3() {
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
                    content: FrameContent::Title(Arc::from("Section Title")),
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
                    content: FrameContent::Subtitle(Arc::from("Section subtitle")),
                    text_flow: None,
                    region_role: None,
                },
            ],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);

        let doc = scraper::Html::parse_document(&result);

        // Must contain <h2> (Title).
        let h2_sel = scraper::Selector::parse("h2").expect("valid selector");
        assert_eq!(
            doc.select(&h2_sel).count(),
            1,
            "MED-1 regression: Title must still render as <h2>; got: {result}"
        );

        // Must contain <h3> (Subtitle with Title present).
        let h3_sel = scraper::Selector::parse("h3").expect("valid selector");
        assert_eq!(
            doc.select(&h3_sel).count(),
            1,
            "MED-1 regression: Subtitle after Title must still render as <h3>; got: {result}"
        );

        // Must NOT contain <p class="sf-subtitle">.
        let p_sel = scraper::Selector::parse("p.sf-subtitle").expect("valid selector");
        assert_eq!(
            doc.select(&p_sel).count(),
            0,
            "MED-1 regression: Subtitle WITH Title must NOT render as <p class=\"sf-subtitle\">; got: {result}"
        );

        // h2 must precede h3 in document order.
        let h2_pos = result.find("<h2").expect("<h2> must be present");
        let h3_pos = result.find("<h3").expect("<h3> must be present");
        assert!(
            h2_pos < h3_pos,
            "MED-1 regression: h2 must appear before h3 in document order; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CRIT-1 + MED-1 / heading-order invariant — no level skips in any
    // representative deck shape.
    //
    // Heading-order invariant (BC-4.03.003 invariant 4 / AC-007 / WCAG AA /
    // ADR-008 heading-rule table):
    //
    //   FOR ANY LaidOutDeck rendered to HTML:
    //   1. Exactly one non-empty <h1> per document.
    //   2. No heading level skip: an <hN> may not appear in document order unless
    //      <h(N-1)> precedes it somewhere in the same document (WCAG heading-order).
    //
    // This invariant is proved by enumerating ALL heading-emitting code paths:
    //   Path A: Title frame → <h{level}> where level ∈ {1, 2}
    //           (pre-pass ensures exactly one H1 in the document)
    //   Path B: Subtitle frame WITH Title on same slide → <h{level+1}>
    //           where level = the slide's Title heading level.
    //           - Title=H1 → Subtitle=h2 (1+1=2): no skip (h1 → h2 is sequential)
    //           - Title=H2 → Subtitle=h3 (2+1=3): no skip (h2 → h3 is sequential)
    //           The Path A heading always precedes Path B on the same slide.
    //           CRIT-1 fix: was hardcoded <h3>, now derived as level+1. The old
    //           code emitted h1→h3 on title-type slides (WCAG violation for 19 passes).
    //   Path C: Subtitle frame WITHOUT Title on same slide → <p class="sf-subtitle">
    //           (no heading emitted → no skip possible: MED-1 fix)
    //   Path D: Body/TextRun H1-promoted → <h1>
    //           (only when no Title exists in the entire deck → first heading is h1)
    //   Path E: No other FrameContent types emit heading elements
    //           (Body→<div>, TextRun→<p>, Image/Chart/Diagram/Shape/ColorBar→SVG layer)
    //
    // No h4/h5/h6 are emitted in practice:
    //   - HeadingLevel::H4 exists in the enum but the exporter pre-pass only assigns
    //     H1 or H2 to Title frames.
    //   - Subtitle receives <h{level+1}>: with level ∈ {1, 2}, sub_level ∈ {2, 3}.
    //   - Body promotion uses <h1> only.
    //
    // Therefore the only possible per-slide heading sequences are:
    //   []       (no text frames; chart-only or image-only slide)
    //   [1]      (title-type slide: Title → h1, no Subtitle)
    //   [1, 2]   (title-type slide: Title → h1, Subtitle → h2, CRIT-1 fixed path)
    //   [2]      (content slide: Title → h2 only)
    //   [2, 3]   (content slide: Title → h2, Subtitle → h3)
    //   [1]      (body-only deck: promoted Body/TextRun → h1)
    //   []       (subtitle-only or body-only: no heading emitted)
    //
    // Sequence [1, 3] is IMPOSSIBLE: Subtitle-only (no Title) → <p> (MED-1);
    // Title+Subtitle at H1 → <h1><h2> (CRIT-1); never h1→h3.
    //
    // Per-slide, sequences [2] and [2, 3] both appear in documents where h1 exists
    // on a different (earlier) slide — document-level no-skip is preserved because
    // the pre-pass guarantees h1 precedes any h2 in the document.
    //
    // Holistic deck-shape proof table (all reachable shapes):
    //   Shape                    | Per-slide headings | No skip?
    //   ─────────────────────────┼────────────────────┼─────────
    //   title-only               | [1]                | yes
    //   title+subtitle (h1 slide)| [1, 2]             | yes (h1→h2)
    //   content (h2 slide)       | [2]                | yes (h1 on prior slide)
    //   content+subtitle (h2)    | [2, 3]             | yes (h2→h3)
    //   subtitle-only            | [] (→ <p>)         | yes (no heading)
    //   body-only                | [1] (promoted)     | yes
    //   chart-only               | []                 | yes (no heading)
    //   multi-title (degenerate) | [2] per slide      | yes (h1 on first slide)
    //   degenerate-title (zero)  | [] → body promoted | yes
    //   empty deck               | []                 | yes (no heading)
    // ─────────────────────────────────────────────────────────────────────────

    /// Helper: count all heading elements of the given level in HTML.
    fn count_headings(html: &str, level: u8) -> usize {
        let doc = scraper::Html::parse_document(html);
        let sel = scraper::Selector::parse(&format!("h{level}")).expect("valid selector");
        doc.select(&sel).count()
    }

    /// Helper: assert no heading level is skipped within a single slide's HTML.
    ///
    /// Checks that if hN is present then h(N-1) is also present on the SAME slide.
    /// This is stricter than `assert_no_orphan_h3`: it covers all levels 2-6.
    ///
    /// Use for slides where ALL levels must be anchored locally (e.g., a title+subtitle
    /// slide at h1 where h1 → h2, both on the same slide). Do NOT use for content
    /// slides rendered in isolation at H2 level — those legitimately have h2+h3
    /// without a local h1 (the h1 lives on the title slide earlier in the document).
    fn assert_no_heading_skip_strict(html: &str, context: &str) {
        for level in 2u8..=6 {
            if count_headings(html, level) > 0 {
                assert!(
                    count_headings(html, level - 1) > 0,
                    "heading-order violation: h{level} present but no h{} on same slide (level skip → WCAG heading-order); {context}",
                    level - 1
                );
            }
        }
    }

    /// Helper: assert that within a rendered slide HTML, if h3 appears then h2
    /// (or h1) must also appear on the same slide — otherwise it is a heading skip.
    ///
    /// This is the per-slide variant of the WCAG heading-order invariant for the
    /// h3 level specifically:
    ///   h3 without h2/h1 on the same slide → axe-core `heading-order` violation.
    ///
    /// The document-level invariant (h1 precedes all h2, which precede all h3) is
    /// upheld by the exporter pre-pass (exactly one H1 per document; H1-level slide
    /// precedes H2-level slides in document order). The per-slide test here proves
    /// no slide can emit an orphan h3.
    ///
    /// For slides rendered in isolation at H2 (content slides), h2+h3 without a
    /// local h1 is correct — the h1 lives on the title slide.
    fn assert_no_orphan_h3(html: &str, context: &str) {
        let h3_count = count_headings(html, 3);
        if h3_count > 0 {
            let h2_count = count_headings(html, 2);
            let h1_count = count_headings(html, 1);
            assert!(
                h2_count > 0 || h1_count > 0,
                "heading-order violation: h3 present but no h2/h1 on same slide (orphan h3 → level skip); {context}"
            );
        }
    }

    /// Heading-order invariant: title-only slide (h1). Exactly one h1, no orphan h3.
    #[test]
    fn test_heading_order_invariant_title_only_slide() {
        let slide = make_title_slide_type(
            "title",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Title(Arc::from("Document Title")),
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H1, &page_size);

        assert_no_orphan_h3(&result, &format!("title-only slide; html={result}"));

        assert_eq!(
            count_headings(&result, 1),
            1,
            "title-only slide must have exactly one h1; html={result}"
        );
        assert_eq!(
            count_headings(&result, 3),
            0,
            "title-only slide must have no h3; html={result}"
        );
    }

    /// Heading-order invariant: title-type slide (h1) + Subtitle → h1 then h2.
    ///
    /// CRIT-1 regression test: before the fix, Title+Subtitle at H1 produced
    /// `<h1>` then `<h3>` — a heading skip that axe-core flags as `heading-order`
    /// violation. After CRIT-1 fix, Subtitle level = Title level + 1 = 2 → `<h2>`.
    ///
    /// This test was the missing coverage that allowed CRIT-1 to survive 19 adversary
    /// passes. It is now part of the heading-order invariant family.
    #[test]
    fn test_heading_order_invariant_title_slide_with_subtitle_h1_then_h2() {
        let slide = make_title_slide_type(
            "title",
            vec![
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(0),
                        width: Emu(9_144_000),
                        height: Emu(1_500_000),
                    },
                    content: FrameContent::Title(Arc::from("Cover Title")),
                    text_flow: None,
                    region_role: None,
                },
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(1_500_000),
                        width: Emu(9_144_000),
                        height: Emu(3_643_500),
                    },
                    content: FrameContent::Subtitle(Arc::from("Cover subtitle")),
                    text_flow: None,
                    region_role: None,
                },
            ],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        // H1 level: title-type (cover) slide — the most common real-world shape.
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H1, &page_size);

        // No heading skip (the key invariant). Use strict check: on an h1 slide,
        // both h1 and h2 must be locally present so no skip is possible.
        assert_no_heading_skip_strict(&result, &format!("title+subtitle h1 slide; html={result}"));

        // Exactly h1 (Title) and h2 (Subtitle = title_level + 1 = 1+1 = 2).
        assert_eq!(
            count_headings(&result, 1),
            1,
            "title+subtitle h1 slide must have exactly one h1; html={result}"
        );
        assert_eq!(
            count_headings(&result, 2),
            1,
            "title+subtitle h1 slide must have exactly one h2 (Subtitle); html={result}"
        );
        // Critical: NO h3. h1→h3 was the CRIT-1 bug.
        assert_eq!(
            count_headings(&result, 3),
            0,
            "title+subtitle h1 slide must have NO h3 (h1→h3 was the CRIT-1 bug); html={result}"
        );
        // h1 before h2 in document order.
        let h1_pos = result.find("<h1").expect("<h1> must be present");
        let h2_pos = result.find("<h2").expect("<h2> must be present");
        assert!(
            h1_pos < h2_pos,
            "title+subtitle h1 slide: h1 must precede h2 in document order; html={result}"
        );
    }

    /// Heading-order invariant: subtitle-only slide (no Title) must emit <p>, not <h3>.
    /// Before MED-1 fix: this would emit <h3> → h1→h3 skip (h1 on title slide, h3
    /// here, no h2 anywhere on this slide).
    /// After MED-1 fix: <p class="sf-subtitle"> — no heading skip possible.
    #[test]
    fn test_heading_order_invariant_subtitle_only_slide() {
        let slide = make_title_slide_type(
            "title",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Subtitle(Arc::from("Only a subtitle")),
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        // H2 level — even though the pre-pass might assign H2, Subtitle without Title → <p>.
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);

        assert_no_orphan_h3(&result, &format!("subtitle-only slide; html={result}"));

        // Specifically: no h3 (was the bug).
        assert_eq!(
            count_headings(&result, 3),
            0,
            "subtitle-only slide must have no <h3> (would be orphan h3); html={result}"
        );
        // Must have <p class="sf-subtitle"> instead.
        let doc = scraper::Html::parse_document(&result);
        let p_sel = scraper::Selector::parse("p.sf-subtitle").expect("valid selector");
        assert_eq!(
            doc.select(&p_sel).count(),
            1,
            "subtitle-only slide must have one <p class=\"sf-subtitle\">; html={result}"
        );
    }

    /// Heading-order invariant: body-only slide with H1 promotion. No orphan h3.
    #[test]
    fn test_heading_order_invariant_body_only_slide() {
        use slideforge_types::{ContentBlock, InlineNode, SourceSpan, TextBlock, TextTag};

        let slide = make_title_slide_type(
            "content",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Body(vec![ContentBlock::Text(TextBlock {
                    inlines: vec![InlineNode::Plain(Arc::from("Body text content"))],
                    tag: TextTag::Untagged,
                    span: SourceSpan::default(),
                })]),
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H1, &page_size);

        assert_no_orphan_h3(&result, &format!("body-only slide; html={result}"));

        // Body-only H1 promotion: exactly one h1.
        assert_eq!(
            count_headings(&result, 1),
            1,
            "body-only slide (H1 promotion) must have exactly one h1; html={result}"
        );
        assert_eq!(
            count_headings(&result, 3),
            0,
            "body-only slide must have no h3; html={result}"
        );
    }

    /// Heading-order invariant: chart-only slide (no text). No headings at all.
    #[test]
    fn test_heading_order_invariant_chart_only_slide() {
        let slide = make_title_slide_type(
            "content",
            vec![Frame {
                bbox: make_bbox_full(),
                content: FrameContent::Chart {
                    alt: AltText::Provided(Arc::from("Revenue chart")),
                },
                text_flow: None,
                region_role: None,
            }],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        // H2 level (not H1 so no body promotion triggered).
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);

        assert_no_orphan_h3(&result, &format!("chart-only slide; html={result}"));
        for level in 1u8..=6 {
            assert_eq!(
                count_headings(&result, level),
                0,
                "chart-only slide must have no h{level}; html={result}"
            );
        }
    }

    /// Heading-order invariant: mixed slide (Title h2 + Subtitle h3 + Body + Chart).
    /// Subtitle has Title on the same slide → renders as h3. No orphan h3.
    #[test]
    fn test_heading_order_invariant_mixed_title_subtitle_body_slide() {
        use slideforge_types::{ContentBlock, InlineNode, SourceSpan, TextBlock, TextTag};

        let slide = make_title_slide_type(
            "content",
            vec![
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(0),
                        width: Emu(9_144_000),
                        height: Emu(800_000),
                    },
                    content: FrameContent::Title(Arc::from("Mixed Title")),
                    text_flow: None,
                    region_role: None,
                },
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(800_000),
                        width: Emu(9_144_000),
                        height: Emu(600_000),
                    },
                    content: FrameContent::Subtitle(Arc::from("Mixed Subtitle")),
                    text_flow: None,
                    region_role: None,
                },
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(1_400_000),
                        width: Emu(9_144_000),
                        height: Emu(2_000_000),
                    },
                    content: FrameContent::Body(vec![ContentBlock::Text(TextBlock {
                        inlines: vec![InlineNode::Plain(Arc::from("Body content"))],
                        tag: TextTag::Untagged,
                        span: SourceSpan::default(),
                    })]),
                    text_flow: None,
                    region_role: None,
                },
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(3_400_000),
                        width: Emu(9_144_000),
                        height: Emu(1_700_000),
                    },
                    content: FrameContent::Chart {
                        alt: AltText::Provided(Arc::from("A chart")),
                    },
                    text_flow: None,
                    region_role: None,
                },
            ],
        );
        let brand = make_brand();
        let page_size = make_page_size();
        let result = render_slide_to_html(&slide, &brand, HeadingLevel::H2, &page_size);

        // Title present → Subtitle renders as h3. No orphan h3 (h2 is present).
        assert_no_orphan_h3(&result, &format!("mixed slide; html={result}"));

        // Must have exactly one h2 (Title) and one h3 (Subtitle).
        assert_eq!(
            count_headings(&result, 2),
            1,
            "mixed slide must have exactly one h2; html={result}"
        );
        assert_eq!(
            count_headings(&result, 3),
            1,
            "mixed slide must have exactly one h3; html={result}"
        );
        // h2 must appear before h3 in document order.
        let h2_pos = result.find("<h2").expect("<h2> must be present");
        let h3_pos = result.find("<h3").expect("<h3> must be present");
        assert!(
            h2_pos < h3_pos,
            "mixed slide: h2 must precede h3 in document order; html={result}"
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

    // ─── STORY-081 C3: SubtitleInlines HTML rendering ────────────────────────

    /// STORY-081 C3 — HTML: `FrameContent::SubtitleInlines` renders inline markup
    /// richly (e.g., `<strong>` for Bold) rather than flattening to plain text.
    ///
    /// ## RED GATE (pre-C3 fix)
    ///
    /// Before fix: `FrameContent::SubtitleInlines` variant did not exist;
    /// subtitle inline structure was dropped at layout seam → rendered as plain text.
    ///
    /// After fix: `SubtitleInlines` renders via `render_inline_nodes` →
    /// Bold nodes produce `<strong>`, Italic → `<em>`, etc.
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    #[test]
    fn test_story_081_c3_subtitle_inlines_renders_strong_in_html() {
        use slideforge_types::InlineNode;

        let bbox = BoundingBox {
            x: Emu(0),
            y: Emu(0),
            width: Emu(9_144_000),
            height: Emu(914_400),
        };

        let frame = Frame {
            bbox,
            content: FrameContent::SubtitleInlines(vec![
                InlineNode::Bold(vec![InlineNode::Plain(Arc::from("Bold Subtitle"))]),
                InlineNode::Plain(Arc::from(" — rest")),
            ]),
            text_flow: None,
            region_role: None,
        };

        // With a title frame present: SubtitleInlines renders as <h{level+1}>
        let result = render_text_frame(&frame, HeadingLevel::H1, true)
            .expect("SubtitleInlines with title must render");

        assert!(
            result.contains("<strong>"),
            "STORY-081 C3 RED GATE: SubtitleInlines must render Bold node as <strong>.\n\
             Before C3 fix: SubtitleInlines variant didn't exist → rendered as plain text.\n\
             After C3 fix: render_inline_nodes called → Bold → <strong>.\n\
             Got: {result}"
        );
        assert!(
            result.contains("Bold Subtitle"),
            "SubtitleInlines must include 'Bold Subtitle' text content. Got: {result}"
        );
        assert!(
            result.contains("sf-subtitle"),
            "SubtitleInlines must have sf-subtitle class. Got: {result}"
        );

        // Without title: renders as <p class="sf-subtitle">
        let result_no_title = render_text_frame(&frame, HeadingLevel::H1, false)
            .expect("SubtitleInlines without title must render");
        assert!(
            result_no_title.contains("<p"),
            "SubtitleInlines without title must render as <p>. Got: {result_no_title}"
        );
        assert!(
            result_no_title.contains("<strong>"),
            "SubtitleInlines without title must still render Bold as <strong>. Got: {result_no_title}"
        );
    }

    // ─── STORY-081 I2: HTML/PDF rich title via title_inlines override ─────────

    /// STORY-081 I2 — HTML: `render_slide_to_html_with_title_override` renders
    /// the title frame with inline markup when `title_inlines_override` is Some.
    ///
    /// ## RED GATE (pre-I2 fix)
    ///
    /// Before fix: HTML exporter called `render_slide_to_html` without title_inlines.
    /// `FrameContent::Title(plain_text)` arm only rendered escaped plain text.
    /// After fix: `render_slide_to_html_with_title_override` intercepts Title frames
    /// and renders inline content via `render_inline_nodes` when override is present.
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    #[test]
    fn test_story_081_i2_html_rich_title_with_title_inlines_override() {
        use slideforge_types::InlineNode;

        let bbox = BoundingBox {
            x: Emu(0),
            y: Emu(0),
            width: Emu(9_144_000),
            height: Emu(914_400),
        };

        let slide = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("title"),
            frames: vec![Frame {
                bbox,
                content: FrameContent::Title(Arc::from("Plain Title")),
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        };

        let brand = Brand {
            name: Arc::from("test"),
            palette: slideforge_types::BrandPalette {
                primary: Arc::from("#000"),
                secondary: Arc::from("#fff"),
                accent: Arc::from("#f00"),
                neutral: Arc::from("#eee"),
            },
            fonts: slideforge_types::BrandFonts {
                heading: Arc::from("Helvetica"),
                body: Arc::from("Helvetica"),
                mono: Arc::from("Courier"),
                font_size_emu: 457_200,
            },
            layouts: vec![],
            span: slideforge_types::SourceSpan::default(),
        };
        let page_size = PageSize::default();

        // Without title_inlines override: renders plain title text.
        let plain_result = render_slide_to_html(&slide, &brand, HeadingLevel::H1, &page_size);
        assert!(
            plain_result.contains("Plain Title"),
            "Without override: plain title text must appear. Got: {plain_result:.500}"
        );
        assert!(
            !plain_result.contains("<strong>"),
            "Without override: no <strong> in title. Got: {plain_result:.500}"
        );

        // With title_inlines override: renders rich inline content.
        let title_inlines = vec![InlineNode::Bold(vec![InlineNode::Plain(Arc::from(
            "Rich Title",
        ))])];
        let rich_result = super::render_slide_to_html_with_title_override(
            &slide,
            &brand,
            HeadingLevel::H1,
            &page_size,
            Some(&title_inlines),
        );

        assert!(
            rich_result.contains("<strong>"),
            "STORY-081 I2 RED GATE: With title_inlines override, <strong> must appear in title.\n\
             Before I2 fix: render_slide_to_html always used plain text for FrameContent::Title.\n\
             After I2 fix: render_slide_to_html_with_title_override intercepts Title frames.\n\
             Got: {rich_result:.500}"
        );
        assert!(
            rich_result.contains("Rich Title"),
            "Title text content 'Rich Title' must be present. Got: {rich_result:.500}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-P17-002a: HTML Footnote must render as <span role="note">
    // ─────────────────────────────────────────────────────────────────────────

    /// F-P17-002a / BC-3.05.001 PC-4:
    /// `InlineNode::Footnote(children)` must render as `<span role="note">…</span>`.
    ///
    /// ## Red Gate
    ///
    /// FAILS against the current implementation (render.rs line ~168) which
    /// emits `<small>…</small>` — a generic presentational element that does
    /// not convey note semantics to assistive technology.
    ///
    /// PASSES after the Footnote arm is changed to emit
    /// `<span role="note">…</span>`, recursing children for inner formatting.
    #[test]
    fn test_f_p17_002a_footnote_renders_as_span_role_note_not_small() {
        use slideforge_types::InlineNode;

        // Footnote with plain text child.
        let footnote =
            InlineNode::Footnote(vec![InlineNode::Plain(Arc::from("This is a footnote."))]);

        let rendered = super::render_inline_node(&footnote);

        // Must use <span role="note">, not <small>.
        assert!(
            rendered.contains(r#"<span role="note">"#),
            "F-P17-002a RED GATE: InlineNode::Footnote must render as \
             <span role=\"note\"> (BC-3.05.001 PC-4). \
             Before fix: emits <small>. \
             After fix: emits <span role=\"note\">. \
             Got: {rendered}"
        );
        assert!(
            !rendered.contains("<small>"),
            "F-P17-002a: <small> must NOT be emitted for Footnote after the fix. \
             Got: {rendered}"
        );
        // Display text must still be present.
        assert!(
            rendered.contains("This is a footnote."),
            "F-P17-002a: footnote text must appear inside <span role=\"note\">. \
             Got: {rendered}"
        );
    }

    /// F-P17-002a — Inner formatting inside Footnote must be preserved.
    /// `Footnote([Bold([Plain("note")])])` → `<span role="note"><strong>note</strong></span>`.
    #[test]
    fn test_f_p17_002a_footnote_inner_formatting_preserved() {
        use slideforge_types::InlineNode;

        let footnote = InlineNode::Footnote(vec![InlineNode::Bold(vec![InlineNode::Plain(
            Arc::from("note"),
        )])]);

        let rendered = super::render_inline_node(&footnote);

        assert!(
            rendered.contains(r#"<span role="note">"#),
            "F-P17-002a (formatting): must use <span role=\"note\">. Got: {rendered}"
        );
        assert!(
            rendered.contains("<strong>note</strong>"),
            "F-P17-002a (formatting): inner Bold must render as <strong>. Got: {rendered}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STORY-072 — FillSpec::Gradient HTML tests (Red Gate)
// ─────────────────────────────────────────────────────────────────────────────
//
// These tests cover AC-004 (HTML CSS linear-gradient) for STORY-072.
//
// | Test | AC | Clause |
// |---|---|---|
// | test_BC_3_04_001_ac004_html_css_linear_gradient_background_format | AC-004 | postcondition 5 |
// | test_BC_3_04_001_ac004_html_css_linear_gradient_direction_to_bottom | AC-004 | postcondition 5 |
// | test_BC_3_04_001_ac004_html_css_gradient_stop_colors_uppercase_hex | AC-004 | postcondition 5 |
// | test_BC_3_04_001_ec005_html_same_from_to_css_gradient_valid | EC-005 | STORY-072 EC-005 |
// | test_BC_3_04_001_ac004_html_gradient_shape_frame_emits_linear_gradient_style | AC-004/OBS-002 | SVG linearGradient paint (adv-P2 HIGH-001) |
// | test_BC_3_04_001_ac004_html_gradient_shape_alt_accessible_name | AC-004 + AC-005 | invariant 1 |
// | test_BC_3_04_001_ac005_html_gradient_decorative_shape_aria_hidden | AC-005 | BC-3.04.001 invariant 1 |

#[cfg(test)]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used,
    non_snake_case
)]
mod story_072_tests {
    use std::sync::Arc;

    use slideforge_layout::{
        BoundingBox, FillSpec, Frame, FrameContent, PageSize, ShapeFrame, ShapeType,
    };
    use slideforge_types::{AltText, Emu, Rgb};

    use super::{css_linear_gradient_background, render_graphics_layer};

    // ─── Helpers ─────────────────────────────────────────────────────────────

    fn gradient_bbox() -> BoundingBox {
        BoundingBox {
            x: Emu(914_400),
            y: Emu(914_400),
            width: Emu(914_400),
            height: Emu(914_400),
        }
    }

    fn default_page() -> PageSize {
        PageSize {
            width: Emu(9_144_000),
            height: Emu(6_858_000),
        }
    }

    fn gradient_shape_frame(from: Rgb, to: Rgb, alt: AltText) -> Frame {
        Frame {
            bbox: gradient_bbox(),
            content: FrameContent::Shape(ShapeFrame {
                shape_type: ShapeType::Rect,
                fill: FillSpec::Gradient { from, to },
                text: None,
                alt,
            }),
            text_flow: None,
            region_role: None,
        }
    }

    // ─── CSS stub tests (Red Gate via todo!()) ────────────────────────────────

    /// AC-004 (STORY-072) — `css_linear_gradient_background` returns the correct
    /// CSS format: `linear-gradient(to bottom, #FF0000, #0000FF)`.
    ///
    /// Fails on the `todo!()` stub (panic = test failure). Passes once implemented.
    #[test]
    fn test_BC_3_04_001_ac004_html_css_linear_gradient_background_format() {
        let result =
            css_linear_gradient_background(Rgb { r: 255, g: 0, b: 0 }, Rgb { r: 0, g: 0, b: 255 });
        assert_eq!(
            result, "linear-gradient(to bottom, #FF0000, #0000FF)",
            "css_linear_gradient_background must return correct CSS format"
        );
    }

    /// AC-004 (STORY-072) — `css_linear_gradient_background` always uses
    /// `to bottom` direction (v1.0 fixed top-to-bottom).
    ///
    /// Fails on the `todo!()` stub (panic = test failure). Passes once implemented.
    #[test]
    fn test_BC_3_04_001_ac004_html_css_linear_gradient_direction_to_bottom() {
        let result =
            css_linear_gradient_background(Rgb { r: 0, g: 255, b: 0 }, Rgb { r: 0, g: 0, b: 255 });
        assert!(
            result.contains("to bottom"),
            "css_linear_gradient_background must use 'to bottom' direction; got: {result}"
        );
    }

    /// AC-004 (STORY-072) — `css_linear_gradient_background` uses uppercase hex.
    ///
    /// Fails on the `todo!()` stub (panic = test failure). Passes once implemented.
    #[test]
    fn test_BC_3_04_001_ac004_html_css_gradient_stop_colors_uppercase_hex() {
        let result = css_linear_gradient_background(
            Rgb {
                r: 255,
                g: 111,
                b: 0,
            }, // #FF6F00
            Rgb {
                r: 0,
                g: 55,
                b: 102,
            }, // #003766
        );
        // Both colors must appear as uppercase hex in the output.
        assert!(
            result.contains("FF6F00"),
            "from color must be uppercase hex 'FF6F00'; got: {result}"
        );
        assert!(
            result.contains("003766"),
            "to color must be uppercase hex '003766'; got: {result}"
        );
    }

    /// EC-005 (STORY-072) — `css_linear_gradient_background` with same from=to
    /// colors produces a valid CSS gradient string (no error, no panic).
    ///
    /// Fails on the `todo!()` stub (panic = test failure). Passes once implemented.
    #[test]
    fn test_BC_3_04_001_ec005_html_same_from_to_css_gradient_valid() {
        let same = Rgb { r: 255, g: 0, b: 0 };
        let result = css_linear_gradient_background(same, same);
        // Same color appears twice in the gradient.
        assert!(
            result.contains("FF0000"),
            "same from==to gradient must still appear in output; got: {result}"
        );
    }

    // ─── render_graphics_layer integration tests (Red Gate via assertion) ────

    /// AC-004 (STORY-072) / OBS-002 — `render_graphics_layer` with a gradient `ShapeFrame`
    /// paints the shape via SVG-native `<linearGradient>` + `fill="url(#sf-grad-..."`.
    ///
    /// Load-bearing paint assertion (adv-P2 HIGH-001): asserts BOTH:
    /// 1. A `<linearGradient` element exists in the output (the def is present), AND
    /// 2. The `<rect>` references it with `fill="url(#sf-grad` (the paint is wired up).
    ///
    /// A regression to the old CSS-background form (which used `style="background: ..."`
    /// on the SVG `<rect>` and painted NOTHING in browsers) will fail this test because
    /// neither `<linearGradient` nor `fill="url(#sf-grad` would be present.
    #[test]
    fn test_BC_3_04_001_ac004_html_gradient_shape_frame_emits_linear_gradient_style() {
        let from = Rgb { r: 255, g: 0, b: 0 };
        let to = Rgb { r: 0, g: 0, b: 255 };
        let frames = vec![gradient_shape_frame(
            from,
            to,
            AltText::Provided(Arc::from("Red-to-blue gradient")),
        )];
        let html = render_graphics_layer(&frames, "test-slide", &default_page());

        // OBS-002 load-bearing: assert SVG-native gradient def is emitted.
        assert!(
            html.contains("<linearGradient"),
            "HTML graphics layer must contain '<linearGradient' element for FillSpec::Gradient; \
             CSS background on <rect> painted nothing (HIGH-001). \
             Got HTML snippet: {}",
            &html[..html.len().min(800)]
        );
        // OBS-002 load-bearing: assert the <rect> actually references the gradient def.
        assert!(
            html.contains("fill=\"url(#sf-grad"),
            "HTML graphics layer <rect> must reference the gradient def via fill=\"url(#sf-grad...\"; \
             gradient def without a reference paints nothing (HIGH-001). \
             Got HTML snippet: {}",
            &html[..html.len().min(800)]
        );
    }

    /// AC-004 (STORY-072) — `render_graphics_layer` gradient `style` contains both
    /// `from` and `to` hex colors.
    ///
    /// RED GATE: assertion fails because no gradient CSS is currently emitted.
    #[test]
    fn test_BC_3_04_001_ac004_html_gradient_shape_from_to_colors_in_style() {
        let from = Rgb {
            r: 255,
            g: 111,
            b: 0,
        }; // #FF6F00
        let to = Rgb {
            r: 0,
            g: 55,
            b: 102,
        }; // #003766
        let frames = vec![gradient_shape_frame(
            from,
            to,
            AltText::Provided(Arc::from("Orange to dark blue")),
        )];
        let html = render_graphics_layer(&frames, "test-slide", &default_page());

        assert!(
            html.contains("FF6F00"),
            "HTML gradient must include from color 'FF6F00'; got: {}",
            &html[..html.len().min(600)]
        );
        assert!(
            html.contains("003766"),
            "HTML gradient must include to color '003766'; got: {}",
            &html[..html.len().min(600)]
        );
    }

    /// AC-004 + AC-005 (STORY-072) — Gradient shape with `AltText::Provided` emits
    /// `role="img"` and accessible `<title>` in the HTML output.
    ///
    /// RED GATE: the Shape arm currently emits `role="img"` for provided alt text,
    /// so this test PASSES today. It becomes load-bearing to ensure the gradient
    /// implementation doesn't regress the existing accessibility pattern.
    ///
    /// This is NOT a Red Gate test — it exercises behavior that already works.
    /// Included for completeness and regression protection.
    #[test]
    fn test_BC_3_04_001_ac004_html_gradient_shape_alt_accessible_name() {
        let alt_text = "Red to blue gradient background";
        let frames = vec![gradient_shape_frame(
            Rgb { r: 255, g: 0, b: 0 },
            Rgb { r: 0, g: 0, b: 255 },
            AltText::Provided(Arc::from(alt_text)),
        )];
        let html = render_graphics_layer(&frames, "test-slide", &default_page());

        // The alt text must appear in a <title> element (existing pattern).
        assert!(
            html.contains(alt_text),
            "HTML gradient shape must contain accessible alt text '{alt_text}'; got: {}",
            &html[..html.len().min(600)]
        );
        assert!(
            html.contains("role=\"img\""),
            "HTML gradient shape with provided alt must have role=\"img\"; got: {}",
            &html[..html.len().min(600)]
        );
    }

    /// AC-005 (STORY-072) — Decorative gradient shape emits `aria-hidden="true"`.
    ///
    /// RED GATE: `AltText::Decorative` already emits `aria-hidden="true"` in the
    /// current Shape arm. This test is GREEN today (existing behavior). Load-bearing
    /// to guard against gradient implementation regressions.
    #[test]
    fn test_BC_3_04_001_ac005_html_gradient_decorative_shape_aria_hidden() {
        let frames = vec![gradient_shape_frame(
            Rgb { r: 0, g: 255, b: 0 },
            Rgb { r: 0, g: 0, b: 255 },
            AltText::Decorative,
        )];
        let html = render_graphics_layer(&frames, "test-slide", &default_page());

        assert!(
            html.contains("aria-hidden=\"true\""),
            "Decorative gradient shape must emit aria-hidden=\"true\"; got: {}",
            &html[..html.len().min(600)]
        );
    }
}
