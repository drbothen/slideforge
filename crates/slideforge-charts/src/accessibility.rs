//! SVG accessibility attribute injection.
//!
//! The WCAG AA requirements for SVG charts (BC-1.11.001 postcondition 4):
//! - `<title>{alt text}</title>` as the first child of the root `<svg>` element
//! - `aria-label="{alt text}"` attribute on the root `<svg>` element
//! - `role="img"` attribute on the root `<svg>` element

use std::sync::Arc;

use crate::types::{ChartError, ChartSvg};

/// Escape a string for use in an XML attribute value (double-quote delimited).
///
/// Replaces `&`, `<`, `>`, `"` with their XML entity equivalents.
fn xml_attr_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Escape a string for use as XML text content (inside element body).
///
/// Replaces `&`, `<`, `>` with their XML entity equivalents.
fn xml_text_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Inject ARIA accessibility attributes and a `<title>` element into a raw
/// SVG string produced by the plotters backend.
///
/// The function:
/// 1. Locates the opening `<svg` tag
/// 2. Appends `aria-label="..."` and `role="img"` attributes before the `>`
/// 3. Inserts `<title>...</title>` as the first child of the `<svg>` element
///
/// # Errors
///
/// Returns [`ChartError::RenderError`] if the SVG string does not contain a
/// root `<svg` element (which would indicate a plotters backend failure).
pub fn inject_aria_attributes(svg: &str, alt: &str) -> Result<ChartSvg, ChartError> {
    // Find the opening <svg tag.
    let svg_start = svg.find("<svg").ok_or_else(|| ChartError::RenderError {
        message: Arc::from("SVG string does not contain a root <svg> element"),
    })?;

    // Find the closing > of the opening tag. This must be after <svg.
    let tag_close = svg[svg_start..]
        .find('>')
        .ok_or_else(|| ChartError::RenderError {
            message: Arc::from("SVG opening tag is not properly closed"),
        })?
        + svg_start;

    let escaped_attr = xml_attr_escape(alt);
    let escaped_text = xml_text_escape(alt);
    let title_element = format!("<title>{escaped_text}</title>");

    // Inject attributes before the closing > of the <svg tag.
    // We insert before the `>` (or `/>` for self-closing, but plotters always
    // produces a non-self-closing root element).
    let insertion_point = tag_close; // position of '>'
    let (before_close, from_close) = svg.split_at(insertion_point);

    // Build the new SVG: tag attributes + close + title element + rest.
    // from_close starts with '>'.
    let rest = &from_close[1..]; // strip the original '>'

    let result =
        format!(r#"{before_close} aria-label="{escaped_attr}" role="img">{title_element}{rest}"#);

    Ok(ChartSvg(result))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------------
    // BC-1.11.001 postcondition 4 tests
    // ---------------------------------------------------------------------------

    #[test]
    fn test_bc_1_11_001_accessibility_injects_aria_label() {
        // todo!() stub — test will panic before assertion is reached.
        let raw_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 450" width="800px" height="450px"></svg>"#;
        let result = inject_aria_attributes(raw_svg, "Q1 Revenue Chart");
        let svg = result.unwrap();
        assert!(svg.as_str().contains("aria-label=\"Q1 Revenue Chart\""));
    }

    #[test]
    fn test_bc_1_11_001_accessibility_injects_role_img() {
        let raw_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 450" width="800px" height="450px"></svg>"#;
        let result = inject_aria_attributes(raw_svg, "Q1 Revenue Chart");
        let svg = result.unwrap();
        assert!(svg.as_str().contains("role=\"img\""));
    }

    #[test]
    fn test_bc_1_11_001_accessibility_injects_title_element() {
        let raw_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 450" width="800px" height="450px"></svg>"#;
        let result = inject_aria_attributes(raw_svg, "Q1 Revenue Chart");
        let svg = result.unwrap();
        assert!(svg.as_str().contains("<title>Q1 Revenue Chart</title>"));
    }

    #[test]
    fn test_bc_1_11_001_accessibility_alt_text_escaping() {
        // alt text with XML special chars must be escaped in both aria-label and <title>.
        // FINDING-003: This test was previously a no-op — it must assert the escaped entities.
        let raw_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 450" width="800px" height="450px"></svg>"#;
        let result = inject_aria_attributes(raw_svg, "Revenue < Cost & Profit > Zero");
        let svg = result.unwrap();
        let content = svg.as_str();

        // aria-label must contain XML-escaped entities (attribute context).
        assert!(
            content.contains("&lt;"),
            "aria-label must escape '<' as '&lt;'; got: {content}"
        );
        assert!(
            content.contains("&amp;"),
            "aria-label must escape '&' as '&amp;'; got: {content}"
        );
        assert!(
            content.contains("&gt;"),
            "aria-label must escape '>' as '&gt;'; got: {content}"
        );

        // <title> content must also be escaped (text content context).
        let title_start = content
            .find("<title>")
            .expect("<title> element must be present");
        let title_end = content
            .find("</title>")
            .expect("</title> element must be present");
        let title_content = &content[title_start..title_end];
        assert!(
            title_content.contains("&lt;"),
            "<title> must escape '<' as '&lt;'; got title: {title_content}"
        );
        assert!(
            title_content.contains("&amp;"),
            "<title> must escape '&' as '&amp;'; got title: {title_content}"
        );
        assert!(
            title_content.contains("&gt;"),
            "<title> must escape '>' as '&gt;'; got title: {title_content}"
        );
    }

    #[test]
    fn test_bc_1_11_001_accessibility_no_svg_tag_returns_error() {
        let not_svg = "not an svg string";
        let result = inject_aria_attributes(not_svg, "alt");
        assert!(result.is_err());
    }
}
