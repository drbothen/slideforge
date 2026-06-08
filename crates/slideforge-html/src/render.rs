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

use slideforge_layout::{Frame, LaidOutSlide};
use slideforge_types::Brand;

/// Render a single slide to an HTML fragment string.
///
/// The returned `String` is a self-contained HTML fragment representing one
/// slide: an `<article>` landmark containing an `<svg>` canvas element for the
/// slide visuals, with correct ARIA roles, alt attributes, and heading hierarchy.
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
pub fn render_slide_to_html(slide: &LaidOutSlide, brand: &Brand) -> String {
    todo!("AC-002/AC-003/AC-004/AC-005/AC-006: render slide to HTML fragment; \
           use minijinja slide.html.jinja template; no <canvas>; \
           lang comes from brand/deck metadata; \
           non-decorative SVG gets role=img+<title> via render_svg_chart; \
           decorative gets alt= role=presentation; \
           inject accessibility attributes via quick-xml, NOT usvg tree API")
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
    todo!("AC-003/AC-004/AC-005/AC-006: dispatch on FrameContent variant; \
           for SVG-bearing variants (Chart, Diagram, ErrorSlidePlaceholder) delegate to render_svg_chart; \
           inject role/aria via quick-xml, NOT usvg; \
           decorative elements get alt= role=presentation; \
           non-decorative get non-empty alt or <title>")
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
    todo!("AC-003/AC-005: use quick-xml to parse svg_str; \
           inject role=img on outer <svg>; \
           prepend <title>alt_text</title> as first child; \
           mark nested <svg> elements aria-hidden=true; \
           return modified SVG string; \
           on parse error: tracing::warn! + return svg_str unchanged")
}

// ─────────────────────────────────────────────────────────────────────────────
// STORY-046 Red Gate Tests — render.rs
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used
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

        let doc = scraper::Html::parse_document(&result);
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
