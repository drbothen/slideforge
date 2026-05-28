//! Error-slide placeholder SVG generation (STORY-032).
//!
//! When the chart pipeline is in warn-only mode and chart data is empty, the
//! layout engine substitutes a [`FrameContent::ErrorSlidePlaceholder`] in place
//! of the chart frame. This module generates the SVG content for that
//! placeholder.
//!
//! ## Visual specification
//!
//! The placeholder is a slide-sized SVG rectangle with:
//! - Light gray background (`#F0F0F0`)
//! - Centered error code in bold (e.g., `E-LAY-003`)
//! - Centered human-readable message below the code
//! - No `<script>` or `<foreignObject>` elements (PPTX-safe)
//! - An `aria-label` attribute for screen reader accessibility
//!
//! ## Dimensions
//!
//! The SVG uses the chart renderer's default dimensions:
//! - Width: [`crate::types::InternalChartSpec::DEFAULT_WIDTH`] (800 px)
//! - Height: [`crate::types::InternalChartSpec::DEFAULT_HEIGHT`] (450 px)

/// XML-escape the five predefined XML entities in `s`.
///
/// Replaces `&`, `<`, `>`, `"`, and `'` with their XML entity equivalents.
/// This ensures that user-controlled strings (slide titles, error messages)
/// cannot inject raw XML into the SVG output.
fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

/// Build an error-slide placeholder SVG string.
///
/// Produces a PPTX-safe, self-contained SVG that renders a light gray
/// background with the error code and message centered on screen. The output
/// is suitable for embedding in a [`slideforge_layout::FrameContent::ErrorSlidePlaceholder`]
/// variant.
///
/// # Arguments
///
/// * `slide_title` — Title of the chart slide where empty data was detected.
/// * `error_code` — The error taxonomy code (e.g., `"E-LAY-003"`).
/// * `message` — The human-readable error message to display on the placeholder.
///
/// # Returns
///
/// A complete SVG string (`<svg ...>...</svg>`) with no external references.
/// The SVG dimensions match the chart renderer default (800 × 450 px) so that
/// the placeholder fits the same slot as a rendered chart.
///
/// The SVG includes:
/// - Explicit `width="800"` and `height="450"` attributes (HIGH-001 / MED-006)
/// - `viewBox="0 0 800 450"` matching the chart default (HIGH-001)
/// - A top-level `<title>` child element with `slide_title` (HIGH-002)
/// - `aria-label` attribute on the root for screen-reader accessibility
/// - `role="img"` for ARIA semantics
/// - A light gray background `<rect>`
/// - `<text>` elements for `error_code` and `message`
///
/// # PPTX Safety
///
/// The returned SVG must not contain `<script>` or `<foreignObject>` elements.
/// All user-supplied strings are XML-escaped before inclusion.
#[must_use]
pub fn build_error_slide_placeholder_svg(
    slide_title: &str,
    error_code: &str,
    message: &str,
) -> String {
    let escaped_title = xml_escape(slide_title);
    let escaped_code = xml_escape(error_code);
    let escaped_msg = xml_escape(message);
    let mut svg = String::new();
    // HIGH-001/MED-006: dimensions match chart default (800×450). Explicit width+height
    // added alongside viewBox for SVG consumers that do not process viewBox alone.
    svg.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="450" viewBox="0 0 800 450" role="img" aria-label="Error placeholder for slide: "#);
    svg.push_str(&escaped_title);
    svg.push_str("\">\n");
    // HIGH-002: <title> child element for accessibility parity with chart SVGs.
    svg.push_str("  <title>");
    svg.push_str(&escaped_title);
    svg.push_str("</title>\n");
    svg.push_str("  <rect width=\"100%\" height=\"100%\" fill=\"#F3F4F6\" stroke=\"#9CA3AF\" stroke-width=\"2\"/>\n");
    // Text positions scaled proportionally from 1280×720 → 800×450:
    //   x: 640 → 400  (center of 800 px width)
    //   y: 300 → 188, 350 → 225, 400 → 263
    svg.push_str("  <text x=\"400\" y=\"188\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"24\" font-weight=\"bold\" fill=\"#374151\">");
    svg.push_str(&escaped_code);
    svg.push_str("</text>\n");
    svg.push_str("  <text x=\"400\" y=\"225\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"20\" fill=\"#374151\">");
    svg.push_str(&escaped_title);
    svg.push_str("</text>\n");
    svg.push_str("  <text x=\"400\" y=\"263\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"16\" fill=\"#6B7280\">");
    svg.push_str(&escaped_msg);
    svg.push_str("</text>\n");
    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // BC-1.11.002 AC-004 — placeholder SVG structure
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-1.11.002 AC-004 — Placeholder SVG is non-empty.
    ///
    /// Red Gate: fails until `build_error_slide_placeholder_svg` is implemented.
    #[test]
    fn test_bc_1_11_002_placeholder_svg_nonempty() {
        let svg = build_error_slide_placeholder_svg(
            "Revenue Chart",
            "E-LAY-003",
            "Chart data is empty for slide 'Revenue Chart'",
        );
        assert!(!svg.is_empty(), "placeholder SVG must not be empty");
    }

    /// BC-1.11.002 AC-004 — Placeholder SVG contains a root `<svg` element.
    ///
    /// Red Gate: fails until `build_error_slide_placeholder_svg` is implemented.
    #[test]
    fn test_bc_1_11_002_placeholder_svg_has_root_element() {
        let svg = build_error_slide_placeholder_svg(
            "Revenue Chart",
            "E-LAY-003",
            "Chart data is empty for slide 'Revenue Chart'",
        );
        assert!(
            svg.contains("<svg"),
            "placeholder SVG must contain a root <svg element"
        );
    }

    /// BC-1.11.002 AC-004 — Placeholder SVG contains the error code.
    ///
    /// Red Gate: fails until `build_error_slide_placeholder_svg` is implemented.
    #[test]
    fn test_bc_1_11_002_placeholder_svg_contains_error_code() {
        let svg =
            build_error_slide_placeholder_svg("KPI Dashboard", "E-LAY-003", "Chart data is empty");
        assert!(
            svg.contains("E-LAY-003"),
            "placeholder SVG must render the error code; got: {}",
            &svg[..svg.len().min(400)]
        );
    }

    /// BC-1.11.002 AC-004 — Placeholder SVG contains the message text.
    ///
    /// Red Gate: fails until `build_error_slide_placeholder_svg` is implemented.
    #[test]
    fn test_bc_1_11_002_placeholder_svg_contains_message() {
        let msg = "Chart data is empty for slide 'KPI Dashboard'";
        let svg = build_error_slide_placeholder_svg("KPI Dashboard", "E-LAY-003", msg);
        assert!(
            svg.contains("Chart data is empty"),
            "placeholder SVG must render the message; got: {}",
            &svg[..svg.len().min(400)]
        );
    }

    /// BC-1.11.002 AC-004 — Placeholder SVG must NOT contain `<script`.
    ///
    /// Red Gate: fails until `build_error_slide_placeholder_svg` is implemented.
    #[test]
    fn test_bc_1_11_002_placeholder_svg_no_script() {
        let svg = build_error_slide_placeholder_svg("Slide", "E-LAY-003", "empty data");
        assert!(
            !svg.contains("<script"),
            "placeholder SVG must not contain <script (PPTX safety)"
        );
    }

    /// BC-1.11.002 AC-004 — Placeholder SVG must NOT contain `<foreignObject`.
    ///
    /// Red Gate: fails until `build_error_slide_placeholder_svg` is implemented.
    #[test]
    fn test_bc_1_11_002_placeholder_svg_no_foreign_object() {
        let svg = build_error_slide_placeholder_svg("Slide", "E-LAY-003", "empty data");
        assert!(
            !svg.contains("<foreignObject"),
            "placeholder SVG must not contain <foreignObject (PPTX safety)"
        );
    }

    /// BC-1.11.002 AC-004 — Placeholder SVG contains aria-label.
    ///
    /// Red Gate: fails until `build_error_slide_placeholder_svg` is implemented.
    #[test]
    fn test_bc_1_11_002_placeholder_svg_has_aria_label() {
        let svg =
            build_error_slide_placeholder_svg("Revenue Chart", "E-LAY-003", "Chart data is empty");
        assert!(
            svg.contains("aria-label"),
            "placeholder SVG must contain an aria-label attribute for accessibility"
        );
    }

    /// BC-1.11.002 AC-004 / AC-005 — `slide_title` containing `<script>alert()</script>`
    /// must be XML-escaped in the SVG output so no literal `<script>` element appears.
    ///
    /// This is the primary security assertion: user-controlled strings must never
    /// be injected as raw XML. The test uses a title that is the worst-case XSS
    /// payload (`<script>alert()</script>`) to verify the implementation escapes
    /// `<`, `>`, and `&` characters before embedding them in SVG attributes or
    /// text content.
    ///
    /// Red Gate: fails until `build_error_slide_placeholder_svg` is implemented.
    #[test]
    fn test_bc_1_11_002_placeholder_xml_escapes_user_input() {
        let malicious_title = "<script>alert('xss')</script>";
        let svg = build_error_slide_placeholder_svg(
            malicious_title,
            "E-LAY-003",
            "Chart data is empty for this slide",
        );

        // The escaped form of < is &lt;. If the output contains &lt;script
        // then the title was properly escaped.
        assert!(
            svg.contains("&lt;script") || svg.contains("&lt;SCRIPT"),
            "slide_title '<script>...' must be XML-escaped to '&lt;script' in SVG; \
             got first 600 chars: {}",
            &svg[..svg.len().min(600)]
        );

        // There must be no literal unescaped <script element in the SVG output.
        // We check case-insensitively because browsers parse HTML tags case-insensitively.
        let lower = svg.to_ascii_lowercase();
        // Remove any properly-escaped &lt; occurrences before checking for raw <script.
        let without_escaped = lower.replace("&lt;", "").replace("&gt;", "");
        assert!(
            !without_escaped.contains("<script"),
            "after removing &lt;/&gt; escapes, no literal <script> element must remain; \
             SVG first 600 chars: {}",
            &svg[..svg.len().min(600)]
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HIGH-001 / MED-006 — Placeholder SVG dimensions: 800×450 with explicit attrs
    // ─────────────────────────────────────────────────────────────────────────

    /// HIGH-001 — Placeholder SVG uses 800×450 viewBox (matches chart default, not 1280×720).
    #[test]
    fn test_bc_1_11_002_placeholder_svg_has_800x450_viewbox() {
        let svg = build_error_slide_placeholder_svg("Test", "E-LAY-003", "empty");
        assert!(
            svg.contains("viewBox=\"0 0 800 450\""),
            "placeholder SVG must have viewBox=\"0 0 800 450\" (chart default size); \
             got: {}",
            &svg[..svg.len().min(300)]
        );
    }

    /// MED-006 — Placeholder SVG has explicit width and height attributes.
    #[test]
    fn test_bc_1_11_002_placeholder_svg_has_explicit_width_height() {
        let svg = build_error_slide_placeholder_svg("Test", "E-LAY-003", "empty");
        assert!(
            svg.contains("width=\"800\""),
            "placeholder SVG must have explicit width=\"800\" attribute; got: {}",
            &svg[..svg.len().min(300)]
        );
        assert!(
            svg.contains("height=\"450\""),
            "placeholder SVG must have explicit height=\"450\" attribute; got: {}",
            &svg[..svg.len().min(300)]
        );
    }

    /// HIGH-002 — Placeholder SVG has a top-level `<title>` child element.
    ///
    /// Provides accessibility parity with rendered chart SVGs which also have
    /// a `<title>` element.
    #[test]
    fn test_bc_1_11_002_placeholder_svg_has_title_element() {
        let svg =
            build_error_slide_placeholder_svg("Revenue Chart", "E-LAY-003", "Chart data is empty");
        assert!(
            svg.contains("<title>"),
            "placeholder SVG must contain a <title> element for accessibility; \
             got first 400 chars: {}",
            &svg[..svg.len().min(400)]
        );
        // Title element must contain the (escaped) slide title text.
        // "Revenue Chart" has no special chars so will appear verbatim.
        assert!(
            svg.contains("<title>Revenue Chart</title>"),
            "placeholder <title> must contain the slide title text; got: {}",
            &svg[..svg.len().min(600)]
        );
    }

    /// HIGH-002 — `<title>` element content is XML-escaped when slide title has special chars.
    #[test]
    fn test_bc_1_11_002_placeholder_title_element_escapes_slide_title() {
        let svg = build_error_slide_placeholder_svg("Revenue & Profit <Q3>", "E-LAY-003", "empty");
        assert!(
            svg.contains("<title>Revenue &amp; Profit &lt;Q3&gt;</title>"),
            "placeholder <title> must XML-escape the slide title; got: {}",
            &svg[..svg.len().min(600)]
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MED-003 — xml_escape per-entity and double-escape prevention
    // ─────────────────────────────────────────────────────────────────────────

    /// MED-003 — `xml_escape` escapes `&` → `&amp;`.
    #[test]
    fn test_placeholder_xml_escape_ampersand() {
        // Access xml_escape indirectly via build_error_slide_placeholder_svg.
        // The title "A & B" should appear as "A &amp; B" in the SVG.
        let svg = build_error_slide_placeholder_svg("A & B", "E-LAY-003", "empty");
        assert!(
            svg.contains("A &amp; B"),
            "& must be escaped to &amp; in SVG output; got: {}",
            &svg[..svg.len().min(400)]
        );
    }

    /// MED-003 — `xml_escape` escapes `<` → `&lt;`.
    #[test]
    fn test_placeholder_xml_escape_less_than() {
        let svg = build_error_slide_placeholder_svg("A < B", "E-LAY-003", "empty");
        assert!(
            svg.contains("A &lt; B"),
            "< must be escaped to &lt;; got: {}",
            &svg[..svg.len().min(400)]
        );
    }

    /// MED-003 — `xml_escape` escapes `>` → `&gt;`.
    #[test]
    fn test_placeholder_xml_escape_greater_than() {
        let svg = build_error_slide_placeholder_svg("A > B", "E-LAY-003", "empty");
        assert!(
            svg.contains("A &gt; B"),
            "> must be escaped to &gt;; got: {}",
            &svg[..svg.len().min(400)]
        );
    }

    /// MED-003 — `xml_escape` escapes `"` → `&quot;`.
    #[test]
    fn test_placeholder_xml_escape_double_quote() {
        let svg = build_error_slide_placeholder_svg("Say \"hi\"", "E-LAY-003", "empty");
        assert!(
            svg.contains("Say &quot;hi&quot;"),
            "\" must be escaped to &quot;; got: {}",
            &svg[..svg.len().min(400)]
        );
    }

    /// MED-003 — `xml_escape` escapes `'` → `&apos;`.
    #[test]
    fn test_placeholder_xml_escape_single_quote() {
        let svg = build_error_slide_placeholder_svg("It's here", "E-LAY-003", "empty");
        assert!(
            svg.contains("It&apos;s here"),
            "' must be escaped to &apos;; got: {}",
            &svg[..svg.len().min(400)]
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OBS-003 — Placeholder SVG passes the safety::assert_no_forbidden_elements check
    // ─────────────────────────────────────────────────────────────────────────

    /// OBS-003 — `build_error_slide_placeholder_svg` output passes the safety check.
    ///
    /// The placeholder SVG must not contain `<script>` or `<foreignObject>` elements,
    /// as enforced by [`crate::safety::assert_no_forbidden_elements`]. This test
    /// ensures the placeholder path is covered by the same safety gate as rendered
    /// chart SVGs — any future change to the template that accidentally introduces
    /// a forbidden element will be caught here.
    #[test]
    fn test_placeholder_svg_passes_safety_check() {
        let svg = build_error_slide_placeholder_svg("Test Slide", "E-LAY-003", "test message");
        assert!(
            crate::safety::assert_no_forbidden_elements(&svg).is_ok(),
            "placeholder SVG must pass the PPTX safety check (no <script> or <foreignObject>); \
             got SVG: {}",
            &svg[..svg.len().min(400)]
        );
    }

    /// MED-003 — Double-escape trap: `&amp;` input must not become `&amp;amp;`.
    ///
    /// If the input already contains `&amp;` (e.g., a string that was pre-escaped),
    /// `xml_escape` must only escape the `&` in `&amp;` — not the already-escaped
    /// entity sequence. The `&` in `&amp;` is a real `&` character and must be
    /// escaped to `&amp;`. Result: `&amp;amp;`.
    ///
    /// Wait — `&amp;` as input is a 5-character string: `&`, `a`, `m`, `p`, `;`.
    /// `xml_escape` sees the `&` and escapes it to `&amp;`, so the output is
    /// `&amp;amp;` (the correct behavior for a raw string `"&amp;"` passed in).
    /// This test verifies that the escaped `&` is not double-skipped.
    #[test]
    fn test_placeholder_xml_escape_ampersand_in_already_escaped_input() {
        // Input: literal "&amp;" (the 5-char sequence & a m p ;)
        // Expected output: "&amp;amp;" (the & at start gets escaped to &amp;,
        // yielding &amp;amp;)
        let svg = build_error_slide_placeholder_svg("&amp;", "E-LAY-003", "empty");
        // The & at the start of "&amp;" must be escaped.
        assert!(
            svg.contains("&amp;amp;"),
            "& in input '&amp;' must be escaped to &amp;amp; (not left as &amp;); \
             got: {}",
            &svg[..svg.len().min(400)]
        );
        // The output must NOT contain a raw unescaped & in the text content positions.
        // (It's OK in attribute values like xmlns= etc.)
    }
}
