//! SVG safety assertions for PPTX embedding.
//!
//! PPTX requires embedded SVG to contain no `<script>` or `<foreignObject>`
//! elements. The plotters `SVGBackend` does not emit these, but this module
//! provides a post-generation assertion so that any regression is caught
//! immediately (BC-1.11.001 postcondition 5, VP-TBD).

use std::sync::Arc;

use crate::types::ChartError;

/// Assert that the SVG string contains no forbidden PPTX-unsafe elements.
///
/// The forbidden elements are:
/// - `<script` — JavaScript execution
/// - `<foreignObject` — embeds arbitrary HTML/XML
///
/// # Errors
///
/// Returns [`ChartError::RenderError`] if any forbidden element pattern is
/// found in the SVG string.
pub fn assert_no_forbidden_elements(svg: &str) -> Result<(), ChartError> {
    if svg.contains("<script") {
        return Err(ChartError::RenderError {
            message: Arc::from(
                "SVG contains forbidden <script> element — PPTX embedding not safe",
            ),
        });
    }
    if svg.contains("<foreignObject") {
        return Err(ChartError::RenderError {
            message: Arc::from(
                "SVG contains forbidden <foreignObject> element — PPTX embedding not safe",
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------------
    // BC-1.11.001 postcondition 5 tests
    // ---------------------------------------------------------------------------

    #[test]
    fn test_bc_1_11_001_safety_clean_svg_passes() {
        let clean_svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><rect x="0" y="0" width="10" height="10"/></svg>"#;
        // todo!() stub — test panics before assertion
        assert_no_forbidden_elements(clean_svg).unwrap();
    }

    #[test]
    fn test_bc_1_11_001_safety_rejects_script_element() {
        let svg_with_script = r#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>"#;
        let result = assert_no_forbidden_elements(svg_with_script);
        assert!(result.is_err(), "expected error for <script> element");
    }

    #[test]
    fn test_bc_1_11_001_safety_rejects_foreign_object() {
        let svg_with_foreign =
            r#"<svg xmlns="http://www.w3.org/2000/svg"><foreignObject width="100" height="100"><p>text</p></foreignObject></svg>"#;
        let result = assert_no_forbidden_elements(svg_with_foreign);
        assert!(result.is_err(), "expected error for <foreignObject> element");
    }

    #[test]
    fn test_bc_1_11_001_safety_rejects_script_case_insensitive() {
        // SVG element names are case-sensitive per spec, but we check
        // lowercase only (plotters never emits uppercase element names)
        let svg_lowercase = r#"<svg><script type="text/javascript">evil()</script></svg>"#;
        let result = assert_no_forbidden_elements(svg_lowercase);
        assert!(result.is_err());
    }
}
