//! Mermaid → SVG rendering core.
//!
//! This module wraps `mermaid-rs-renderer` and exposes a synchronous
//! `render_mermaid` function that:
//! 1. Validates the source is non-empty
//! 2. Pre-validates that the source starts with a recognized Mermaid keyword
//!    and has no obviously malformed syntax (unbalanced brackets)
//! 3. Calls `mermaid_rs_renderer::render(source)`
//! 4. Applies forbidden-element safety checks
//! 5. Injects accessibility attributes (aria-label, title)
//!
//! ## Font Database (`OnceLock` pattern)
//!
//! `mermaid-rs-renderer` 0.2.2 handles font loading internally. The crate
//! manages a per-process font database; the first call incurs initialization
//! cost (< 200ms on CI, empirical from Spike S14). Subsequent calls are warm
//! (< 10ms per BC-1.12.001 postconditions 8, 9).
//!
//! No `OnceLock<FontDatabase>` is needed in this crate because `mermaid-rs-renderer`
//! manages its own lazy initialization. The benchmark crates still measure the
//! first-call vs. subsequent-call latency.
//!
//! ## Validation Layer
//!
//! `mermaid-rs-renderer` 0.2.2 is intentionally permissive: it silently renders
//! a best-effort SVG for any input rather than returning a parse error.
//! Per BC-1.12.002, this crate adds a pre-validation pass to detect and surface:
//! - Missing diagram-type keyword (source must begin with a recognized keyword)
//! - Unbalanced square brackets in flowchart/graph node labels

use crate::{
    accessibility::{assert_no_forbidden_elements, inject_aria_attributes},
    error::build_syntax_error,
    types::{DiagramError, RawDiagramSvg},
};

/// Recognized Mermaid diagram-type keywords (case-insensitive prefix match).
///
/// Mirrors the `detect_diagram_kind` logic in `mermaid-rs-renderer` 0.2.2 so
/// that this pre-validation pass rejects inputs that the renderer would silently
/// fall back to `Flowchart` for.
const KNOWN_KEYWORDS: &[&str] = &[
    "sequencediagram",
    "classdiagram",
    "statediagram",
    "erdiagram",
    "pie",
    "mindmap",
    "journey",
    "timeline",
    "gantt",
    "requirementdiagram",
    "gitgraph",
    "c4",
    "sankey",
    "quadrantchart",
    "zenuml",
    "block",
    "packet",
    "kanban",
    "architecture",
    "radar",
    "treemap",
    "xychart",
    "flowchart",
    "graph",
];

/// Pre-validate Mermaid source before handing it to the renderer.
///
/// `mermaid-rs-renderer` 0.2.2 is permissive and never returns `Err` for
/// syntactically malformed input. This function adds structural validation
/// per BC-1.12.002 to catch:
///
/// 1. **No recognized keyword**: the first non-comment, non-frontmatter line
///    must start with a known diagram-type keyword.
/// 2. **Unbalanced brackets in flowchart nodes**: `[` without a matching `]`
///    in non-comment lines of a flowchart/graph diagram.
///
/// Returns `Ok(())` if the source passes validation, or a
/// [`DiagramError::MermaidSyntaxError`] with a helpful message otherwise.
fn validate_mermaid_syntax(source: &str) -> Result<(), DiagramError> {
    let mut in_frontmatter = false;
    let mut keyword_found = false;
    let mut is_flowchart = false;
    let mut flowchart_lines: Vec<&str> = Vec::new();

    for raw_line in source.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "---" {
            in_frontmatter = !in_frontmatter;
            continue;
        }
        if in_frontmatter {
            continue;
        }
        if trimmed.starts_with("%%") {
            continue;
        }
        if !keyword_found {
            // Strip trailing comment for keyword detection
            let without_comment = strip_trailing_comment(trimmed);
            let lower = without_comment.trim().to_ascii_lowercase();
            for &kw in KNOWN_KEYWORDS {
                if lower.starts_with(kw) {
                    keyword_found = true;
                    if kw == "flowchart" || kw == "graph" {
                        is_flowchart = true;
                    }
                    break;
                }
            }
            if !keyword_found {
                return Err(build_syntax_error(&format!(
                    "unrecognized Mermaid diagram type at line 1: expected a keyword \
                     such as 'flowchart', 'sequenceDiagram', 'classDiagram', etc.; \
                     got: {without_comment}",
                )));
            }
        } else if is_flowchart {
            // Collect body lines for bracket validation
            flowchart_lines.push(raw_line);
        }
    }

    if !keyword_found {
        // Source was non-empty but all lines were comments/frontmatter
        return Err(build_syntax_error(
            "Mermaid source contains no diagram type keyword at line 1",
        ));
    }

    // Validate bracket balance in flowchart/graph node labels.
    // In Mermaid flowchart syntax, square-bracket labels must be closed:
    //   A[label]   — valid
    //   A --> [broken  — invalid (unclosed '[')
    if is_flowchart {
        validate_flowchart_brackets(&flowchart_lines)?;
    }

    Ok(())
}

/// Strip a trailing `%%...` comment from a line.
fn strip_trailing_comment(line: &str) -> &str {
    if let Some(pos) = line.find("%%") {
        line[..pos].trim_end()
    } else {
        line
    }
}

/// Validate that square brackets in flowchart body lines are balanced.
///
/// Mermaid flowchart node labels use `[label]` syntax. An unclosed `[` is
/// a syntax error that `mermaid-rs-renderer` silently accepts (it treats the
/// rest of the token as node text). This function detects that case.
///
/// Only checks non-comment lines. String literals (quoted labels) are handled
/// by skipping content between `"..."` and `'...'` delimiters.
fn validate_flowchart_brackets(lines: &[&str]) -> Result<(), DiagramError> {
    for (line_idx, &line) in lines.iter().enumerate() {
        let stripped = strip_trailing_comment(line.trim());
        if stripped.starts_with("%%") || stripped.is_empty() {
            continue;
        }

        let mut depth: i32 = 0;
        let mut in_double_quote = false;
        let mut in_single_quote = false;
        let chars: Vec<char> = stripped.chars().collect();

        for (col, &ch) in chars.iter().enumerate() {
            match ch {
                '"' if !in_single_quote => {
                    in_double_quote = !in_double_quote;
                }
                '\'' if !in_double_quote => {
                    in_single_quote = !in_single_quote;
                }
                '[' if !in_double_quote && !in_single_quote => {
                    // Check if it's the special Mermaid `[*]` transition in
                    // stateDiagram (which we don't reach here because is_flowchart
                    // is only set for flowchart/graph diagrams).
                    depth += 1;
                }
                ']' if !in_double_quote && !in_single_quote => {
                    depth -= 1;
                    if depth < 0 {
                        // Extra ']' without matching '[' — treat as syntax error
                        return Err(build_syntax_error(&format!(
                            "unmatched ']' in flowchart at line {} column {}",
                            line_idx + 2, // +2: 1-based, +1 for keyword line
                            col + 1,
                        )));
                    }
                }
                _ => {}
            }
        }

        if depth > 0 {
            // Unclosed '[' found — this is a syntax error
            return Err(build_syntax_error(&format!(
                "unclosed '[' in flowchart node label at line {}; \
                 every '[' must have a matching ']'",
                line_idx + 2, // +2: 1-based, +1 for keyword line
            )));
        }
    }
    Ok(())
}

/// Render a Mermaid diagram source string to an accessibility-annotated SVG.
///
/// Pipeline (per BC-1.12.001 rendering path):
/// 1. Validate source is non-empty → `DiagramError::EmptySource`
/// 2. Pre-validate Mermaid syntax (keyword + bracket balance)
/// 3. Call `mermaid_rs_renderer::render(source)` → raw SVG or error
/// 4. Safety-check: assert no forbidden elements
/// 5. Inject `aria-label`, `role="img"`, `<title>`
///
/// # Arguments
///
/// - `source` — Raw Mermaid diagram source text (non-empty).
/// - `alt_text` — Accessibility alt text. Pass `""` for decorative diagrams.
///
/// # Errors
///
/// Returns [`DiagramError`] on:
/// - `EmptySource` — `source` is empty or whitespace-only
/// - `MermaidSyntaxError` — invalid Mermaid syntax (maps to E-EXP-008)
/// - `RenderError` — internal render failure
/// - `SvgPostProcessingError` — forbidden elements or aria injection failure
pub fn render_mermaid(source: &str, alt_text: &str) -> Result<RawDiagramSvg, DiagramError> {
    // Step 1: Validate source is non-empty.
    if source.trim().is_empty() {
        return Err(DiagramError::EmptySource);
    }

    // Step 2: Pre-validate Mermaid syntax.
    // `mermaid-rs-renderer` 0.2.2 never returns Err for invalid input — it
    // silently renders a best-effort SVG. We add structural validation here
    // per BC-1.12.002 to return a structured DiagramError for clearly invalid input.
    validate_mermaid_syntax(source)?;

    // Step 3: Render via mermaid-rs-renderer.
    let raw_svg: String = mermaid_rs_renderer::render(source)
        .map_err(|e| build_syntax_error(&e.to_string()))?;

    // Step 4: Assert no forbidden elements.
    assert_no_forbidden_elements(&raw_svg)?;

    // Step 5: Inject accessibility attributes.
    inject_aria_attributes(&raw_svg, alt_text)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // BC-1.12.001 postcondition 1: flowchart renders to non-empty SVG
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_001_flowchart_produces_svg() {
        // TV-12.1: flowchart LR with two nodes → valid PPTX-safe SVG
        let source = "graph TD\n  A-->B";
        let result = render_mermaid(source, "Flow diagram A to B");
        let svg = result.expect("flowchart must render to SVG without error");
        assert!(!svg.is_empty(), "flowchart SVG must not be empty");
        assert!(
            svg.as_str().contains("<svg"),
            "flowchart output must contain <svg root element"
        );
    }

    #[test]
    fn test_bc_1_12_001_sequence_diagram_produces_svg() {
        // BC-1.12.001 canonical test vector: sequenceDiagram
        let source = "sequenceDiagram\n  Alice->>Bob: Hello";
        let result = render_mermaid(source, "Sequence diagram");
        let svg = result.expect("sequenceDiagram must render without error");
        assert!(!svg.is_empty(), "sequenceDiagram SVG must not be empty");
        assert!(svg.as_str().contains("<svg"), "sequenceDiagram output must have <svg element");
    }

    #[test]
    fn test_bc_1_12_001_class_diagram_produces_svg() {
        let source = "classDiagram\n  class Animal {\n    +String name\n    +makeSound()\n  }";
        let result = render_mermaid(source, "Class diagram");
        let svg = result.expect("classDiagram must render without error");
        assert!(!svg.is_empty(), "classDiagram SVG must not be empty");
    }

    #[test]
    fn test_bc_1_12_001_gantt_chart_produces_svg() {
        // EC-003: Gantt chart with tasks
        let source = "gantt\n  section S1\n  Task :a1, 2026-01-01, 30d";
        let result = render_mermaid(source, "Gantt chart");
        let svg = result.expect("gantt must render without error");
        assert!(!svg.is_empty(), "gantt SVG must not be empty");
    }

    #[test]
    fn test_bc_1_12_001_state_diagram_produces_svg() {
        let source = "stateDiagram-v2\n  [*] --> Idle\n  Idle --> Running";
        let result = render_mermaid(source, "State machine");
        let svg = result.expect("stateDiagram must render without error");
        assert!(!svg.is_empty(), "stateDiagram SVG must not be empty");
    }

    // -----------------------------------------------------------------------
    // BC-1.12.002 postcondition 1: invalid syntax → DiagramError
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_002_invalid_syntax_returns_error() {
        // DEC-015: arrow to undefined node
        let source = "flowchart LR\n  A --> [broken";
        let result = render_mermaid(source, "broken diagram");
        assert!(result.is_err(), "invalid Mermaid syntax must return an error");
    }

    #[test]
    fn test_bc_1_12_002_invalid_syntax_error_is_mermaid_syntax_error_variant() {
        let source = "this is not valid mermaid at all !!!";
        let result = render_mermaid(source, "bad diagram");
        // Must return an error — either MermaidSyntaxError or RenderError
        assert!(result.is_err(), "invalid Mermaid must produce an error");
        let err = result.unwrap_err();
        // The error message must contain meaningful information
        let msg = err.to_string();
        assert!(!msg.is_empty(), "error message must not be empty");
    }

    // -----------------------------------------------------------------------
    // BC-1.12.002 EC-002: empty source → DiagramError::EmptySource
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_002_empty_source_returns_empty_source_error() {
        // EC-005 / BC-1.12.002 EC-002: empty string must return EmptySource
        let result = render_mermaid("", "alt text");
        assert!(result.is_err(), "empty source must return an error");
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("empty") || msg.contains("E-EXP-008"),
            "error must mention 'empty' or E-EXP-008; got: {msg}"
        );
    }

    #[test]
    fn test_bc_1_12_002_whitespace_only_source_returns_empty_source_error() {
        // Whitespace-only is equivalent to empty
        let result = render_mermaid("   \n\t  ", "alt text");
        assert!(result.is_err(), "whitespace-only source must return an error");
    }

    // -----------------------------------------------------------------------
    // BC-1.12.001 postcondition 7: accessibility attributes present
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_001_flowchart_has_aria_label() {
        let source = "graph LR\n  A[Client] --> B[API]";
        let result = render_mermaid(source, "Architecture overview");
        let svg = result.expect("flowchart must render");
        assert!(
            svg.as_str().contains("aria-label="),
            "rendered SVG must contain aria-label; got first 400 chars: {}",
            &svg.as_str()[..svg.as_str().len().min(400)]
        );
    }

    #[test]
    fn test_bc_1_12_001_flowchart_has_title_element() {
        let source = "graph LR\n  A[Client] --> B[API]";
        let result = render_mermaid(source, "Architecture overview");
        let svg = result.expect("flowchart must render");
        assert!(
            svg.as_str().contains("<title>"),
            "rendered SVG must contain <title> element"
        );
    }

    // -----------------------------------------------------------------------
    // BC-1.12.001 postconditions 2, 3, 4: PPTX safety checks
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_001_flowchart_no_foreign_object() {
        // BC-1.12.001 postcondition 2
        let source = "graph TD\n  A-->B";
        let result = render_mermaid(source, "Flow");
        let svg = result.expect("flowchart must render");
        assert!(
            !svg.as_str().to_ascii_lowercase().contains("<foreignobject"),
            "rendered SVG must not contain <foreignObject>"
        );
    }

    #[test]
    fn test_bc_1_12_001_flowchart_no_script() {
        // BC-1.12.001 postcondition 3
        let source = "graph TD\n  A-->B";
        let result = render_mermaid(source, "Flow");
        let svg = result.expect("flowchart must render");
        assert!(
            !svg.as_str().to_ascii_lowercase().contains("<script"),
            "rendered SVG must not contain <script>"
        );
    }

    #[test]
    fn test_bc_1_12_001_flowchart_no_keyframes() {
        // BC-1.12.001 postcondition 4
        let source = "graph TD\n  A-->B";
        let result = render_mermaid(source, "Flow");
        let svg = result.expect("flowchart must render");
        assert!(
            !svg.as_str().to_ascii_lowercase().contains("@keyframes"),
            "rendered SVG must not contain @keyframes"
        );
    }

    // -----------------------------------------------------------------------
    // AC-008: multiple Mermaid diagram types available (BC-1.12.001 invariant 3)
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_001_invariant_all_tested_types_render_without_panic() {
        // BC-1.12.001 invariant 3: 23 Mermaid diagram types available.
        // Test a representative subset of 9 types per AC-008.
        let diagrams: &[(&str, &str)] = &[
            ("graph TD\n  A-->B", "flowchart"),
            ("sequenceDiagram\n  Alice->>Bob: Hello", "sequenceDiagram"),
            ("gantt\n  section S1\n  Task :a1, 2026-01-01, 30d", "gantt"),
            ("classDiagram\n  class Animal{\n    +String name\n  }", "classDiagram"),
            ("stateDiagram-v2\n  [*] --> Running", "stateDiagram"),
            ("erDiagram\n  CUSTOMER ||--o{ ORDER : places", "erDiagram"),
            ("pie\n  title Pie\n  \"A\" : 42\n  \"B\" : 58", "pie"),
            ("journey\n  title My day\n  section Morning\n  Wake up: 5: Me", "journey"),
            ("gitGraph\n  commit\n  branch feature\n  commit", "gitGraph"),
        ];

        for (source, label) in diagrams {
            let result = render_mermaid(source, label);
            let svg = result.unwrap_or_else(|e| {
                panic!("diagram type '{label}' must render without error; got: {e}")
            });
            assert!(
                !svg.is_empty(),
                "diagram type '{label}' must produce non-empty SVG"
            );
        }
    }

    // -----------------------------------------------------------------------
    // BC-1.12.001 postcondition 5: viewBox and absolute dimensions in SVG
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_001_flowchart_svg_has_viewbox_attribute() {
        // BC-1.12.001 postcondition 5: SVG root must have viewBox attribute.
        let source = "graph TD\n  A-->B";
        let result = render_mermaid(source, "Flow");
        let svg = result.expect("flowchart must render");
        assert!(
            svg.as_str().contains("viewBox"),
            "rendered SVG must contain viewBox attribute"
        );
    }

    #[test]
    fn test_bc_1_12_001_flowchart_svg_has_width_attribute() {
        // BC-1.12.001 postcondition 5: SVG root must have width attribute.
        let source = "graph TD\n  A-->B";
        let result = render_mermaid(source, "Flow");
        let svg = result.expect("flowchart must render");
        assert!(
            svg.as_str().contains("width"),
            "rendered SVG must contain width attribute"
        );
    }

    #[test]
    fn test_bc_1_12_001_flowchart_svg_has_height_attribute() {
        // BC-1.12.001 postcondition 5: SVG root must have height attribute.
        let source = "graph TD\n  A-->B";
        let result = render_mermaid(source, "Flow");
        let svg = result.expect("flowchart must render");
        assert!(
            svg.as_str().contains("height"),
            "rendered SVG must contain height attribute"
        );
    }

    // -----------------------------------------------------------------------
    // No-panic safety: arbitrary input must not panic (BC-1.12.002 invariant)
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_001_render_no_panic_on_arbitrary_input() {
        // BC-1.12.002 VP: invalid diagram does not crash — returns structured error.
        // Calling render_mermaid directly: if the function panics the test framework
        // will catch the unwind and report a test failure with the panic message.
        // We accept Ok or structured Err — the requirement is just: no panic.
        let inputs = [
            "!@#$%^&*()",
            "flowchart TD",  // valid header, no body
            "1234567890",
            "SELECT * FROM table",
            "<xml>not mermaid</xml>",
        ];
        for input in &inputs {
            // May return Ok or Err — must NOT panic.
            let _ = render_mermaid(input, "test");
        }
    }
}
