//! Shared XML escaping helpers for SVG attribute values and text content.
//!
//! Both `accessibility.rs` and `normalize.rs` need to escape user-supplied
//! strings before injecting them into SVG. This module provides the canonical
//! implementation so neither module duplicates it.
//!
//! ## Control characters
//!
//! XML 1.0 forbids the following code points in any context
//! (`\x00`–`\x08`, `\x0B`, `\x0C`, `\x0E`–`\x1F`). These are replaced with
//! U+FFFD (REPLACEMENT CHARACTER) so that the output is always well-formed XML.
//! `\x09` (HT), `\x0A` (LF), and `\x0D` (CR) are valid XML characters and are
//! passed through unchanged.

/// Escape a string for use in an XML attribute value (double-quote delimited).
///
/// Replaces `&`, `<`, `>`, `"` with their XML entity equivalents.
/// Replaces XML-forbidden control characters with U+FFFD.
pub(crate) fn xml_attr_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            // XML 1.0 §2.2 forbidden control characters (except HT/LF/CR)
            '\x00'..='\x08' | '\x0B' | '\x0C' | '\x0E'..='\x1F' => out.push('\u{FFFD}'),
            _ => out.push(ch),
        }
    }
    out
}

/// Escape a string for use as XML text content (inside element body).
///
/// Replaces `&`, `<`, `>` with their XML entity equivalents.
/// Replaces XML-forbidden control characters with U+FFFD.
pub(crate) fn xml_text_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            // XML 1.0 §2.2 forbidden control characters (except HT/LF/CR)
            '\x00'..='\x08' | '\x0B' | '\x0C' | '\x0E'..='\x1F' => out.push('\u{FFFD}'),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_attr_escape_basic_entities() {
        assert_eq!(xml_attr_escape("a&b"), "a&amp;b");
        assert_eq!(xml_attr_escape("a<b"), "a&lt;b");
        assert_eq!(xml_attr_escape("a>b"), "a&gt;b");
        assert_eq!(xml_attr_escape(r#"a"b"#), "a&quot;b");
    }

    #[test]
    fn test_xml_text_escape_basic_entities() {
        assert_eq!(xml_text_escape("a&b"), "a&amp;b");
        assert_eq!(xml_text_escape("a<b"), "a&lt;b");
        assert_eq!(xml_text_escape("a>b"), "a&gt;b");
        // double-quote is NOT escaped in text content
        assert_eq!(xml_text_escape(r#"a"b"#), r#"a"b"#);
    }

    #[test]
    fn test_xml_attr_escape_control_chars_replaced_with_replacement_char() {
        // NUL, BEL, BS are forbidden in XML 1.0
        let s = "\x00\x07\x08";
        let out = xml_attr_escape(s);
        assert_eq!(out, "\u{FFFD}\u{FFFD}\u{FFFD}");
    }

    #[test]
    fn test_xml_attr_escape_vt_ff_forbidden() {
        // VT (0x0B) and FF (0x0C) are forbidden
        let s = "\x0B\x0C";
        let out = xml_attr_escape(s);
        assert_eq!(out, "\u{FFFD}\u{FFFD}");
    }

    #[test]
    fn test_xml_attr_escape_other_c0_forbidden() {
        // 0x0E–0x1F (excluding HT=0x09, LF=0x0A, CR=0x0D which are valid)
        let s = "\x0E\x1F";
        let out = xml_attr_escape(s);
        assert_eq!(out, "\u{FFFD}\u{FFFD}");
    }

    #[test]
    fn test_xml_attr_escape_valid_whitespace_passes_through() {
        // HT, LF, CR are valid XML characters — must not be replaced
        let s = "\t\n\r";
        let out = xml_attr_escape(s);
        assert_eq!(out, "\t\n\r");
    }

    #[test]
    fn test_xml_text_escape_control_chars_replaced_with_replacement_char() {
        let s = "\x00\x0B\x1F";
        let out = xml_text_escape(s);
        assert_eq!(out, "\u{FFFD}\u{FFFD}\u{FFFD}");
    }
}
