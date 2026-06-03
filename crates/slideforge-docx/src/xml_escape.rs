//! Shared XML character-escaping helpers for DOCX part generation.
//!
//! ## XML 1.0 validity
//!
//! XML 1.0 §2.2 defines legal characters as:
//! `#x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]`
//!
//! Control characters `#x0–#x8`, `#xB–#xC`, and `#xE–#x1F` are not legal XML
//! character data. Values sourced from brand configuration (font names, palette
//! values) or user text must have these stripped before embedding in XML to
//! prevent malformed output (SEC-002 / CWE-116).
//!
//! ## Usage
//!
//! - [`xml_attr_escape`] — embed a value in an XML attribute (`w:val="..."`)
//! - [`xml_content_escape`] — embed a value as XML element text content

/// Escape a string for embedding in an XML attribute value, stripping
/// XML-1.0-invalid control characters (SEC-002 / CWE-116).
///
/// Escapes `&`, `<`, `>`, `"`, and `'` per XML spec, and silently drops
/// any code point that is not legal in XML 1.0 (i.e., `#x0–#x8`, `#xB`,
/// `#xC`, `#xE–#x1F`, and `#xFFFE–#xFFFF`).
///
/// Permitted controls that are kept: `#x9` (tab), `#xA` (LF), `#xD` (CR).
#[must_use]
pub fn xml_attr_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if is_xml10_invalid(ch) {
            // Strip silently — invalid characters must not reach XML output.
            continue;
        }
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

/// Escape a string for embedding as XML element text content, stripping
/// XML-1.0-invalid control characters (SEC-002 / CWE-116).
///
/// Escapes `&`, `<`, and `>` per XML spec, and silently drops any code point
/// that is not legal in XML 1.0.
///
/// Permitted controls that are kept: `#x9` (tab), `#xA` (LF), `#xD` (CR).
#[must_use]
pub fn xml_content_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if is_xml10_invalid(ch) {
            // Strip silently — invalid characters must not reach XML output.
            continue;
        }
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
    out
}

/// Return a copy of `s` with all XML-1.0-invalid control characters removed.
///
/// This is used for text content that goes directly into `ooxmlsdk` data
/// fields (e.g., `<w:t>` element content) which is not processed through
/// `xml_content_escape`. Both paths must strip invalid control chars to
/// ensure well-formed XML output (SEC-002 / CWE-116).
#[must_use]
pub fn strip_xml10_invalid_chars(s: &str) -> String {
    if !s.chars().any(is_xml10_invalid) {
        return s.to_owned();
    }
    s.chars().filter(|&ch| !is_xml10_invalid(ch)).collect()
}

/// Returns `true` for code points that are illegal in XML 1.0 character data.
///
/// XML 1.0 §2.2 Char production:
/// ```text
/// Char ::= #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]
/// ```
///
/// This function returns `true` (must strip) for:
/// - `#x0–#x8`  (C0 controls, NUL through BS)
/// - `#xB–#xC`  (VT, FF)
/// - `#xE–#x1F` (C0 controls, SO through US)
/// - `#xFFFE` and `#xFFFF` (non-characters)
///
/// Note: `#x9` (tab), `#xA` (LF), and `#xD` (CR) are legal XML and are
/// explicitly kept.
#[inline]
fn is_xml10_invalid(ch: char) -> bool {
    let cp = ch as u32;
    matches!(cp,
        0x0000..=0x0008 // NUL–BS
        | 0x000B..=0x000C // VT, FF
        | 0x000E..=0x001F // SO–US
        | 0xFFFE | 0xFFFF // XML non-characters
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attr_escape_xml_metacharacters() {
        assert_eq!(
            xml_attr_escape("a&b<c>d\"e'f"),
            "a&amp;b&lt;c&gt;d&quot;e&apos;f"
        );
    }

    #[test]
    fn attr_escape_strips_nul() {
        assert_eq!(xml_attr_escape("ab\x00cd"), "abcd");
    }

    #[test]
    fn attr_escape_strips_esc() {
        assert_eq!(xml_attr_escape("ab\x1Bcd"), "abcd");
    }

    #[test]
    fn attr_escape_strips_form_feed() {
        assert_eq!(xml_attr_escape("ab\x0Ccd"), "abcd");
    }

    #[test]
    fn attr_escape_keeps_tab_lf_cr() {
        // Tab (#x9), LF (#xA), CR (#xD) are legal XML 1.0.
        let s = "a\tb\nc\rd";
        let result = xml_attr_escape(s);
        assert_eq!(result, "a\tb\nc\rd", "tab/LF/CR must NOT be stripped");
    }

    #[test]
    fn content_escape_xml_metacharacters() {
        assert_eq!(xml_content_escape("a&b<c>d"), "a&amp;b&lt;c&gt;d");
    }

    #[test]
    fn content_escape_strips_nul() {
        assert_eq!(xml_content_escape("hello\x00world"), "helloworld");
    }

    #[test]
    fn content_escape_strips_vt() {
        assert_eq!(xml_content_escape("hello\x0Bworld"), "helloworld");
    }
}
