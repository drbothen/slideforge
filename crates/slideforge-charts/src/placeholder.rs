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
/// The SVG includes:
/// - `aria-label` attribute containing `slide_title` for accessibility
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
    svg.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1280 720" role="img" aria-label="Error placeholder for slide: "#);
    svg.push_str(&escaped_title);
    svg.push_str("\">\n");
    svg.push_str("  <rect width=\"100%\" height=\"100%\" fill=\"#F3F4F6\" stroke=\"#9CA3AF\" stroke-width=\"2\"/>\n");
    svg.push_str("  <text x=\"640\" y=\"300\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"24\" font-weight=\"bold\" fill=\"#374151\">");
    svg.push_str(&escaped_code);
    svg.push_str("</text>\n");
    svg.push_str("  <text x=\"640\" y=\"350\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"20\" fill=\"#374151\">");
    svg.push_str(&escaped_title);
    svg.push_str("</text>\n");
    svg.push_str("  <text x=\"640\" y=\"400\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"16\" fill=\"#6B7280\">");
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
}
