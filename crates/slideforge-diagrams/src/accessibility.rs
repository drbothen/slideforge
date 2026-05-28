//! SVG accessibility attribute injection for diagram output.
//!
//! WCAG AA requirements for SVG diagrams (BC-1.12.001 postcondition 7):
//! - `<title>{alt text}</title>` as the first child of the root `<svg>` element
//! - `aria-label="{alt text}"` attribute on the root `<svg>` element
//! - `role="img"` attribute on the root `<svg>` element
//!
//! This module mirrors the pattern established in `slideforge-charts::accessibility`
//! (STORY-031) and applies it to diagram SVG output.

use std::sync::Arc;

use crate::types::{DiagramError, RawDiagramSvg};

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
/// SVG string produced by `mermaid-rs-renderer`.
///
/// The function:
/// 1. Locates the opening `<svg` tag
/// 2. Appends `aria-label="..."` and `role="img"` attributes before the `>`
/// 3. Inserts `<title>...</title>` as the first child of the `<svg>` element
///
/// Per BC-1.12.001 postcondition 7, if `alt_text` is empty (decorative diagram),
/// both `aria-label` and `<title>` are set to empty strings.
///
/// # Errors
///
/// Returns [`DiagramError::SvgPostProcessingError`] if the SVG string does not
/// contain a root `<svg` element.
pub fn inject_aria_attributes(svg: &str, alt_text: &str) -> Result<RawDiagramSvg, DiagramError> {
    // Find the opening <svg tag.
    let svg_start = svg
        .find("<svg")
        .ok_or_else(|| DiagramError::SvgPostProcessingError {
            message: Arc::from("SVG string does not contain a root <svg> element"),
        })?;

    // Find the closing > of the opening tag. Must be after <svg.
    let tag_close =
        svg[svg_start..]
            .find('>')
            .ok_or_else(|| DiagramError::SvgPostProcessingError {
                message: Arc::from("SVG opening tag is not properly closed"),
            })?
            + svg_start;

    // The opening tag is svg[svg_start..=tag_close].
    let opening_tag = &svg[svg_start..=tag_close];

    // Guard: if aria-label= already exists in the opening tag, skip injection
    // to avoid duplicate attributes (which are invalid in XML/SVG).
    if opening_tag.contains("aria-label=") {
        return Ok(RawDiagramSvg(svg.to_owned()));
    }

    let escaped_attr = xml_attr_escape(alt_text);
    let escaped_text = xml_text_escape(alt_text);
    let title_element = format!("<title>{escaped_text}</title>");

    // Inject attributes before the closing > of the <svg tag.
    let insertion_point = tag_close; // position of '>'
    let (before_close, from_close) = svg.split_at(insertion_point);

    // from_close starts with '>'. Strip it and rebuild.
    let rest = &from_close[1..];

    let result =
        format!(r#"{before_close} aria-label="{escaped_attr}" role="img">{title_element}{rest}"#);

    Ok(RawDiagramSvg(result))
}

/// Assert that the SVG contains no forbidden elements.
///
/// Forbidden elements per BC-1.12.001 postconditions 2, 3, 4:
/// - `<foreignObject>` (PPTX unsafe)
/// - `<script>` (security and PPTX unsafe)
/// - `@keyframes` CSS animations (PPTX unsafe)
///
/// # Errors
///
/// Returns [`DiagramError::SvgPostProcessingError`] if any forbidden element
/// is found in the SVG.
pub fn assert_no_forbidden_elements(svg: &str) -> Result<(), DiagramError> {
    let lower = svg.to_ascii_lowercase();
    if lower.contains("<foreignobject") {
        return Err(DiagramError::SvgPostProcessingError {
            message: Arc::from("SVG contains forbidden <foreignObject> element — not PPTX-safe"),
        });
    }
    if lower.contains("<script") {
        return Err(DiagramError::SvgPostProcessingError {
            message: Arc::from("SVG contains forbidden <script> element — not PPTX-safe"),
        });
    }
    if lower.contains("@keyframes") {
        return Err(DiagramError::SvgPostProcessingError {
            message: Arc::from("SVG contains forbidden @keyframes CSS animation — not PPTX-safe"),
        });
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // BC-1.12.001 postcondition 7: aria-label injected
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_001_accessibility_aria_label_injected() {
        let raw_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 600" width="800" height="600"></svg>"#;
        let result = inject_aria_attributes(raw_svg, "Architecture Diagram");
        let svg = result.unwrap();
        assert!(
            svg.as_str()
                .contains(r#"aria-label="Architecture Diagram""#),
            "SVG must contain aria-label; got: {}",
            &svg.as_str()[..svg.as_str().len().min(300)]
        );
    }

    #[test]
    fn test_bc_1_12_001_accessibility_role_img_injected() {
        let raw_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 600" width="800" height="600"></svg>"#;
        let result = inject_aria_attributes(raw_svg, "Architecture Diagram");
        let svg = result.unwrap();
        assert!(
            svg.as_str().contains(r#"role="img""#),
            "SVG must contain role=\"img\"; got: {}",
            &svg.as_str()[..svg.as_str().len().min(300)]
        );
    }

    #[test]
    fn test_bc_1_12_001_accessibility_title_element_injected() {
        let raw_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 600" width="800" height="600"></svg>"#;
        let result = inject_aria_attributes(raw_svg, "Architecture Diagram");
        let svg = result.unwrap();
        assert!(
            svg.as_str().contains("<title>Architecture Diagram</title>"),
            "SVG must contain <title> element; got: {}",
            &svg.as_str()[..svg.as_str().len().min(300)]
        );
    }

    #[test]
    fn test_bc_1_12_001_accessibility_decorative_empty_alt() {
        // AC-005: decorative diagram → aria-label="" and <title></title>
        let raw_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 600" width="800" height="600"></svg>"#;
        let result = inject_aria_attributes(raw_svg, "");
        let svg = result.unwrap();
        assert!(
            svg.as_str().contains(r#"aria-label="""#),
            "decorative SVG must have empty aria-label; got: {}",
            &svg.as_str()[..svg.as_str().len().min(300)]
        );
        assert!(
            svg.as_str().contains("<title></title>"),
            "decorative SVG must have empty <title>; got: {}",
            &svg.as_str()[..svg.as_str().len().min(300)]
        );
    }

    #[test]
    fn test_bc_1_12_001_accessibility_alt_text_xml_escaped_in_aria_label() {
        // alt text with XML special chars must be escaped in aria-label
        let raw_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 600" width="800" height="600"></svg>"#;
        let result = inject_aria_attributes(raw_svg, "Revenue < Cost & Profit > Zero");
        let svg = result.unwrap();
        let content = svg.as_str();
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
    }

    #[test]
    fn test_bc_1_12_001_accessibility_alt_text_xml_escaped_in_title() {
        let raw_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 600" width="800" height="600"></svg>"#;
        let result = inject_aria_attributes(raw_svg, "Revenue < Cost");
        let svg = result.unwrap();
        let content = svg.as_str();
        let title_start = content.find("<title>").expect("<title> must be present");
        let title_end = content.find("</title>").expect("</title> must be present");
        let title_content = &content[title_start..title_end];
        assert!(
            title_content.contains("&lt;"),
            "<title> must escape '<' as '&lt;'; got: {title_content}"
        );
    }

    #[test]
    fn test_bc_1_12_001_accessibility_no_svg_tag_returns_error() {
        let not_svg = "not an svg string";
        let result = inject_aria_attributes(not_svg, "alt");
        assert!(result.is_err(), "must return error for non-SVG input");
    }

    #[test]
    fn test_bc_1_12_001_accessibility_no_duplicate_aria_label_if_already_present() {
        // Guard: if aria-label= already exists in the opening <svg> tag,
        // inject_aria_attributes must not inject a second aria-label (XML
        // duplicate attributes are invalid).
        let svg_with_aria = r#"<svg xmlns="http://www.w3.org/2000/svg" aria-label="existing" role="img"><title>existing</title></svg>"#;
        let result = inject_aria_attributes(svg_with_aria, "new text");
        let svg = result.unwrap();
        let content = svg.as_str();
        // Count occurrences of aria-label= — must be exactly 1.
        let count = content.matches("aria-label=").count();
        assert_eq!(
            count, 1,
            "must not inject duplicate aria-label; got {count} occurrences in: {content}"
        );
        // Must preserve the original aria-label value.
        assert!(
            content.contains(r#"aria-label="existing""#),
            "must preserve original aria-label; got: {content}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-1.12.001 postconditions 2, 3, 4: assert_no_forbidden_elements
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_001_no_forbidden_elements_clean_svg_passes() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><rect x="0" y="0" width="100" height="100"/></svg>"#;
        let result = assert_no_forbidden_elements(svg);
        assert!(result.is_ok(), "clean SVG must pass; got: {result:?}");
    }

    #[test]
    fn test_bc_1_12_001_no_foreign_object_rejects_foreign_object() {
        // BC-1.12.001 postcondition 2: no <foreignObject>
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><foreignObject width="100" height="50"><div>text</div></foreignObject></svg>"#;
        let result = assert_no_forbidden_elements(svg);
        assert!(result.is_err(), "SVG with <foreignObject> must be rejected");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("foreignObject"),
            "error must mention foreignObject; got: {msg}"
        );
    }

    #[test]
    fn test_bc_1_12_001_no_script_rejects_script_element() {
        // BC-1.12.001 postcondition 3: no <script>
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert('xss')</script></svg>"#;
        let result = assert_no_forbidden_elements(svg);
        assert!(result.is_err(), "SVG with <script> must be rejected");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("script"),
            "error must mention script; got: {msg}"
        );
    }

    #[test]
    fn test_bc_1_12_001_no_keyframes_rejects_animation() {
        // BC-1.12.001 postcondition 4: no @keyframes
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><style>@keyframes pulse { from { opacity: 1; } to { opacity: 0; } }</style></svg>"#;
        let result = assert_no_forbidden_elements(svg);
        assert!(result.is_err(), "SVG with @keyframes must be rejected");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("keyframes"),
            "error must mention keyframes; got: {msg}"
        );
    }
}
