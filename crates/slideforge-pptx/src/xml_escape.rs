//! Shared XML character-escaping helpers for PPTX part generation.
//!
//! ## XML 1.0 validity
//!
//! XML 1.0 §2.2 defines legal characters as:
//! `#x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]`
//!
//! Control characters `#x0–#x8`, `#xB–#xC`, and `#xE–#x1F` are not legal XML
//! character data. Values sourced from user text must have these stripped before
//! embedding in XML to prevent malformed output (SEC-100 / CWE-116).

/// Return a copy of `s` with all XML-1.0-invalid control characters removed.
///
/// Used for text that goes directly into `ooxmlsdk` data fields (e.g., `<a:r>`
/// `text` field) which is not processed through attribute or content escaping.
/// Both paths must strip invalid control characters to ensure well-formed XML
/// output (SEC-100 / CWE-116).
///
/// Permitted controls that are kept: `#x9` (tab), `#xA` (LF), `#xD` (CR).
#[must_use]
pub(crate) fn strip_xml10_invalid_chars(s: &str) -> String {
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
    fn strip_nul_from_plain_text() {
        assert_eq!(strip_xml10_invalid_chars("ab\x00cd"), "abcd");
    }

    #[test]
    fn strip_soh_u0001() {
        // SEC-100: U+0001 (SOH) must be stripped.
        assert_eq!(strip_xml10_invalid_chars("ab\x01cd"), "abcd");
    }

    #[test]
    fn strip_vt_ff() {
        assert_eq!(strip_xml10_invalid_chars("ab\x0B\x0Ccd"), "abcd");
    }

    #[test]
    fn strip_c0_controls() {
        // #x0E–#x1F must all be stripped.
        let s: String = (0x0Eu8..=0x1Fu8).map(|b| b as char).collect();
        let result = strip_xml10_invalid_chars(&s);
        assert!(result.is_empty(), "all C0 controls must be stripped");
    }

    #[test]
    fn keep_tab_lf_cr() {
        // #x9 (tab), #xA (LF), #xD (CR) are legal XML 1.0.
        let s = "a\tb\nc\rd";
        assert_eq!(
            strip_xml10_invalid_chars(s),
            s,
            "tab/LF/CR must NOT be stripped"
        );
    }

    #[test]
    fn passthrough_when_no_invalid_chars() {
        // Fast path: no allocation when input is already valid.
        let s = "Hello, World!";
        assert_eq!(strip_xml10_invalid_chars(s), s);
    }

    #[test]
    fn strip_fffe_ffff_noncharacters() {
        let s = "a\u{FFFE}b\u{FFFF}c";
        assert_eq!(strip_xml10_invalid_chars(s), "abc");
    }
}
