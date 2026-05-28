//! SSRF domain allowlist enforcement for HTTP data sources.
//!
//! The allowlist provides a compile-time-configurable list of permitted
//! domains for HTTP requests issued by [`crate::http::HttpDataSource`].
//! When the list is `None`, all domains are permitted (opt-out mode).
//! When the list is `Some`, only the listed domains are permitted (opt-in
//! mode), and any attempt to fetch from an unlisted domain produces a
//! [`crate::DataError::SsrfBlocked`] error.
//!
//! ## SSRF Protection
//!
//! Server-Side Request Forgery (SSRF) attacks trick a server into issuing
//! requests to internal services (e.g., `169.254.169.254` metadata endpoints,
//! `localhost`, or RFC-1918 addresses). The allowlist ensures that only
//! explicitly approved domains can be fetched.
//!
//! Maps to `E-DAT-006` in the error taxonomy.

use std::sync::Arc;

/// Configuration for the HTTP domain allowlist.
///
/// When `domains` is `Some`, only the listed domain strings are permitted.
/// When `domains` is `None`, all domains are permitted (no restriction).
///
/// Domain strings are compared to the host component of the request URL
/// after lower-casing. Subdomains are NOT implicitly allowed by a parent
/// domain entry — each permitted domain must be listed explicitly.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AllowlistConfig {
    /// The list of permitted domains, or `None` to allow all domains.
    ///
    /// Each entry is compared against the lower-cased host of the request URL.
    /// Example: `vec![Arc::from("api.example.com"), Arc::from("data.example.com")]`.
    pub domains: Option<Vec<Arc<str>>>,
}

/// Check whether a URL's host is permitted by the allowlist configuration.
///
/// Returns `true` if the URL is allowed, `false` if it is blocked.
///
/// ## Rules
///
/// - If `config.domains` is `None`, all URLs are allowed.
/// - If `config.domains` is `Some(list)`, the URL host (lower-cased) must
///   appear verbatim in the list.
/// - URLs without a host component (e.g., `file://`) are always blocked when
///   an allowlist is configured.
///
/// # Arguments
///
/// * `url` — The parsed URL to check.
/// * `config` — The allowlist configuration to enforce.
#[must_use]
pub fn is_allowed(_url: &url::Url, _config: &AllowlistConfig) -> bool {
    todo!()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// `test_BC_5_04_001_allowlist_none_permits_all` — when `domains` is `None`, all URLs are allowed.
    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_bc_5_04_001_allowlist_none_permits_all() {
        let config = AllowlistConfig { domains: None };
        let url = url::Url::parse("https://example.com/data.json").unwrap();
        let _ = is_allowed(&url, &config);
    }

    /// `test_BC_5_04_001_allowlist_some_permits_listed_domain` — listed domain is permitted.
    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_bc_5_04_001_allowlist_some_permits_listed_domain() {
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("example.com")]),
        };
        let url = url::Url::parse("https://example.com/data.json").unwrap();
        let _ = is_allowed(&url, &config);
    }

    /// `test_BC_5_04_001_allowlist_some_blocks_unlisted_domain` — unlisted domain is blocked.
    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_bc_5_04_001_allowlist_some_blocks_unlisted_domain() {
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("example.com")]),
        };
        let url = url::Url::parse("https://evil.com/data.json").unwrap();
        let _ = is_allowed(&url, &config);
    }

    /// `test_BC_5_04_001_allowlist_some_blocks_metadata_endpoint` — AWS metadata endpoint is blocked
    /// when an allowlist is configured.
    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_bc_5_04_001_allowlist_some_blocks_metadata_endpoint() {
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("example.com")]),
        };
        let url = url::Url::parse("http://169.254.169.254/latest/meta-data/").unwrap();
        let _ = is_allowed(&url, &config);
    }
}
