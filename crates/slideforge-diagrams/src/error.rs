//! E-EXP-008 error construction with line number extraction.
//!
//! The `mermaid-rs-renderer` error messages may include line number information.
//! This module extracts the line number from the error string, falling back to 1
//! if no line number is found (conservative fallback per BC-1.12.002).

use std::sync::Arc;

use crate::types::DiagramError;

/// Extract a 1-based line number from a renderer error message string.
///
/// Attempts to parse the first occurrence of patterns like:
/// - `"line 3"` or `"at line 3"`
/// - `"line: 3"`
/// - A bare integer after "line"
///
/// Returns `1` if no line number can be extracted (conservative fallback per
/// BC-1.12.002 invariant 1: "The error always includes a line number within
/// the Mermaid source block").
#[must_use]
pub fn extract_source_line(error_message: &str) -> u32 {
    // Walk the string looking for "line" followed by optional whitespace/colon
    // and then a decimal number.
    let lower = error_message.to_ascii_lowercase();
    let mut pos = 0;
    while let Some(rel) = lower[pos..].find("line") {
        let abs = pos + rel;
        // Word-boundary guard: reject "line" when preceded by an alphanumeric
        // character, which would indicate it is a suffix of a longer word such
        // as "timeline", "outline", "deadline", etc.
        if abs > 0 && lower.as_bytes()[abs - 1].is_ascii_alphanumeric() {
            pos = abs + 1;
            continue;
        }
        // Skip past "line"
        let after = abs + 4;
        if after >= lower.len() {
            break;
        }
        // Skip optional whitespace and colon
        let rest = lower[after..].trim_start_matches([' ', ':', '\t']);
        // Try to parse a leading integer
        let digit_end = rest
            .char_indices()
            .take_while(|(_, c)| c.is_ascii_digit())
            .last()
            .map_or(0, |(i, _)| i + 1);
        if digit_end > 0 {
            let num_str = &rest[..digit_end];
            if let Ok(n) = num_str.parse::<u32>()
                && n >= 1
            {
                return n;
            }
        }
        pos = abs + 1;
    }
    // Conservative fallback per BC-1.12.002 invariant 1.
    1
}

/// Build a [`DiagramError::MermaidSyntaxError`] from a renderer error message.
///
/// Extracts the line number from `raw_message` and constructs the structured
/// `E-EXP-008` error variant.
#[must_use]
pub fn build_syntax_error(raw_message: &str) -> DiagramError {
    let source_line = extract_source_line(raw_message);
    DiagramError::MermaidSyntaxError {
        message: Arc::from(raw_message),
        source_line,
        error_code: "E-EXP-008",
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // BC-1.12.002 postcondition 2: extract_source_line
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_002_extract_line_number_from_line_n_pattern() {
        // "line 3" pattern
        assert_eq!(extract_source_line("parse error at line 3: unexpected token"), 3);
    }

    #[test]
    fn test_bc_1_12_002_extract_line_number_from_line_colon_n_pattern() {
        // "line: 5" pattern
        assert_eq!(extract_source_line("error: line: 5 unexpected"), 5);
    }

    #[test]
    fn test_bc_1_12_002_extract_line_number_fallback_to_1_when_not_found() {
        // No line number in message — must fall back to 1
        assert_eq!(extract_source_line("some error with no line info"), 1);
    }

    #[test]
    fn test_bc_1_12_002_extract_line_number_fallback_to_1_for_empty_string() {
        assert_eq!(extract_source_line(""), 1);
    }

    #[test]
    fn test_bc_1_12_002_extract_line_number_uses_first_line_occurrence() {
        // Multiple "line N" occurrences — must use the first one
        let msg = "error at line 7: also mentioned at line 12";
        assert_eq!(extract_source_line(msg), 7);
    }

    #[test]
    fn test_bc_1_12_002_extract_line_number_at_line_1_pattern() {
        assert_eq!(extract_source_line("at line 1: empty diagram"), 1);
    }

    #[test]
    fn test_bc_1_12_002_extract_line_number_from_large_line_number() {
        assert_eq!(extract_source_line("syntax error at line 100"), 100);
    }

    // -----------------------------------------------------------------------
    // FINDING-003: word-boundary check — "line" as suffix must not match
    // -----------------------------------------------------------------------

    #[test]
    fn test_finding_003_timeline_word_is_not_a_line_reference() {
        // "timeline: 3" contains "line" as a suffix of "timeline".
        // The extractor must NOT return 3 — it should fall back to 1.
        assert_eq!(
            extract_source_line("timeline: 3"),
            1,
            "\"timeline: 3\" must not be mistaken for \"line 3\""
        );
    }

    #[test]
    fn test_finding_003_outline_word_is_not_a_line_reference() {
        assert_eq!(
            extract_source_line("outline: 7 sections"),
            1,
            "\"outline\" suffix must not match as a line number"
        );
    }

    #[test]
    fn test_finding_003_deadline_word_is_not_a_line_reference() {
        assert_eq!(
            extract_source_line("past deadline: 42"),
            1,
            "\"deadline\" suffix must not match as a line number"
        );
    }

    #[test]
    fn test_finding_003_standalone_line_still_matches() {
        // "line" preceded by a space (non-alphanumeric) must still work.
        assert_eq!(
            extract_source_line("error at line 8"),
            8,
            "standalone \"line 8\" must still return 8"
        );
    }

    // -----------------------------------------------------------------------
    // BC-1.12.002 postcondition 1: build_syntax_error
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_002_build_syntax_error_contains_e_exp_008() {
        let err = build_syntax_error("parse error at line 3: unexpected '['");
        let msg = err.to_string();
        assert!(msg.contains("E-EXP-008"), "must contain E-EXP-008; got: {msg}");
    }

    #[test]
    fn test_bc_1_12_002_build_syntax_error_line_number_extracted() {
        let err = build_syntax_error("parse error at line 5: broken arrow");
        let msg = err.to_string();
        assert!(msg.contains("source line 5"), "must contain 'source line 5'; got: {msg}");
    }

    #[test]
    fn test_bc_1_12_002_build_syntax_error_line_fallback_when_not_found() {
        let err = build_syntax_error("some opaque error with no line");
        let msg = err.to_string();
        assert!(msg.contains("source line 1"), "must fall back to source line 1; got: {msg}");
    }

    #[test]
    fn test_bc_1_12_002_build_syntax_error_preserves_original_message() {
        let raw = "Undefined node 'Q'";
        let err = build_syntax_error(raw);
        let msg = err.to_string();
        assert!(
            msg.contains(raw) || msg.contains("Undefined"),
            "error must include original renderer message; got: {msg}"
        );
    }
}
