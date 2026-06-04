//! Defense-in-depth URL scheme allowlist for PPTX hyperlink embedding.
//!
//! ## Why this module exists (F-040-P2-001 / CWE-601)
//!
//! The DSL parser (`slideforge-syntax`) enforces `E-PAR-022`: any `[text](url)`
//! link with a scheme outside `{"http", "https", "mailto"}` is a fatal parse
//! error and never reaches the IR.  However the exporter also accepts
//! programmatically-constructed `InlineNode::Link` values (e.g., from future
//! pipeline stages, test fixtures, or plugin-produced IR).  A
//! `javascript:`/`data:`/`vbscript:`/`file:` URL reaching the exporter would be
//! embedded verbatim as a `TargetMode="External"` relationship, creating a
//! clickable XSS/CSSI vector in the rendered PPTX.
//!
//! This module is the **single authoritative `SafeUrl` guard** for the
//! `slideforge-pptx` crate.  The future slide-body hyperlink path
//! (tracked as SEC-037-001) MUST route through `is_safe_link_scheme` rather than
//! adding a second ad-hoc check.
//!
//! ## Allowlist mirror policy
//!
//! The allowlist here MUST mirror the parser's `E-PAR-022` allowlist
//! (`slideforge-syntax/src/parser/template.rs` → `ALLOWED_LINK_SCHEMES`).
//! If the parser ever adds `tel:` or removes a scheme, this list must be updated
//! in the same PR.  Divergence is a CWE-601 risk.

/// Permitted URL schemes for external hyperlinks embedded in PPTX parts.
///
/// Mirrors the parser's `E-PAR-022` allowlist (`ALLOWED_LINK_SCHEMES` in
/// `slideforge-syntax::parser::template`).  Comparison is case-insensitive
/// (see [`is_safe_link_scheme`]).
///
/// Disallowed schemes include (but are not limited to): `javascript`, `data`,
/// `vbscript`, `file`.  Any scheme not in this list is rejected.
pub const ALLOWED_LINK_SCHEMES: &[&str] = &["http", "https", "mailto"];

/// Returns `true` if `url` has a scheme that is in the [`ALLOWED_LINK_SCHEMES`]
/// allowlist.
///
/// Extracts the scheme as the substring before the first `':'`.  If the URL
/// contains no `':'`, it has no scheme and is rejected.  Comparison is
/// case-insensitive so `HTTPS://example.com` and `https://example.com` both
/// pass.
///
/// This is the **single `SafeUrl` guard** for the `slideforge-pptx` crate.
/// The future slide-body link path (SEC-037-001) must call this function
/// rather than implementing a separate check.
#[must_use]
pub fn is_safe_link_scheme(url: &str) -> bool {
    let Some(colon_pos) = url.find(':') else {
        // No scheme separator — rejected (no relative URLs in external rels).
        return false;
    };
    let scheme = url[..colon_pos].to_ascii_lowercase();
    ALLOWED_LINK_SCHEMES.contains(&scheme.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_https_url_passes() {
        assert!(is_safe_link_scheme("https://example.com"));
    }

    #[test]
    fn test_safe_http_url_passes() {
        assert!(is_safe_link_scheme("http://example.com"));
    }

    #[test]
    fn test_safe_mailto_passes() {
        assert!(is_safe_link_scheme("mailto:user@example.com"));
    }

    #[test]
    fn test_javascript_scheme_rejected() {
        assert!(!is_safe_link_scheme("javascript:alert(1)"));
    }

    #[test]
    fn test_data_scheme_rejected() {
        assert!(!is_safe_link_scheme("data:text/html,<h1>hi</h1>"));
    }

    #[test]
    fn test_vbscript_scheme_rejected() {
        assert!(!is_safe_link_scheme("vbscript:msgbox(1)"));
    }

    #[test]
    fn test_file_scheme_rejected() {
        assert!(!is_safe_link_scheme("file:///etc/passwd"));
    }

    #[test]
    fn test_no_scheme_rejected() {
        assert!(!is_safe_link_scheme("example.com/path"));
    }

    #[test]
    fn test_case_insensitive_https_passes() {
        assert!(is_safe_link_scheme("HTTPS://example.com"));
    }
}
