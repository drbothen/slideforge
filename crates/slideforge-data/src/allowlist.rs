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
pub fn is_allowed(url: &url::Url, config: &AllowlistConfig) -> bool {
    // No allowlist configured — permit all.
    let Some(domains) = &config.domains else {
        return true;
    };

    // Allowlist is present. If empty, fail-closed (block all).
    if domains.is_empty() {
        return false;
    }

    // Build the match key: "host" or "host:port" depending on whether the URL
    // carries an explicit port component.
    // URL with no host is always blocked when allowlist is set.
    let Some(host) = url.host_str() else {
        return false;
    };
    let domain_key = match url.port() {
        Some(p) => format!("{host}:{p}"),
        None => host.to_owned(),
    };

    // Exact match against each entry.
    domains.iter().any(|d| d.as_ref() == domain_key)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // BC-1.03.005: Allowlist precondition / postcondition / invariant tests
    // -----------------------------------------------------------------------

    /// `test_BC_1_03_005_allowed_domain_passes`
    ///
    /// Postcondition: when `domains` contains the URL host, `is_allowed()` returns `true`.
    ///
    /// AC-003 test vector: domains=`["api.example.com"]`, URL=https://api.example.com/data → true
    #[test]
    fn test_bc_1_03_005_allowed_domain_passes() {
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("api.example.com")]),
        };
        let url = url::Url::parse("https://api.example.com/data").unwrap();
        assert!(is_allowed(&url, &config), "listed domain must be permitted");
    }

    /// `test_BC_1_03_005_blocked_domain`
    ///
    /// Postcondition: when `domains` does NOT contain the URL host, `is_allowed()` returns `false`.
    ///
    /// AC-003 test vector: domains=`["api.example.com"]`, URL=https://other.example.com/data → false
    #[test]
    fn test_bc_1_03_005_blocked_domain() {
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("api.example.com")]),
        };
        let url = url::Url::parse("https://other.example.com/data").unwrap();
        assert!(
            !is_allowed(&url, &config),
            "unlisted domain must be blocked"
        );
    }

    /// `test_BC_1_03_005_empty_list_blocks_all`
    ///
    /// Invariant (BC-1.03.005 invariant 3 + AC-004): `allowed_domains = []` blocks ALL domains.
    ///
    /// AC-004 test vector: domains=[], URL=https://api.example.com/data → false
    #[test]
    fn test_bc_1_03_005_empty_list_blocks_all() {
        let config = AllowlistConfig {
            domains: Some(vec![]),
        };
        let url = url::Url::parse("https://api.example.com/data").unwrap();
        assert!(
            !is_allowed(&url, &config),
            "empty allowlist must block all domains"
        );
    }

    /// `test_BC_1_03_005_absent_allowlist_permits_all`
    ///
    /// Invariant (BC-1.03.005 invariant 2 + AC-004): absent `allowed_domains` (None) permits all.
    ///
    /// AC-004 test vector: domains=None, URL=https://api.example.com/data → true
    #[test]
    fn test_bc_1_03_005_absent_allowlist_permits_all() {
        let config = AllowlistConfig { domains: None };
        let url = url::Url::parse("https://api.example.com/data").unwrap();
        assert!(
            is_allowed(&url, &config),
            "absent allowlist (None) must permit all domains"
        );
    }

    /// `test_BC_1_03_005_port_mismatch_blocked`
    ///
    /// Invariant (BC-1.03.005 invariant 4 + AC-003): exact host+port match required.
    /// `api.example.com` does NOT match `api.example.com:8443`.
    ///
    /// AC-003 test vector: domains=`["api.example.com"]`, URL=https://api.example.com:8443/data → false
    #[test]
    fn test_bc_1_03_005_port_mismatch_blocked() {
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("api.example.com")]),
        };
        let url = url::Url::parse("https://api.example.com:8443/data").unwrap();
        assert!(
            !is_allowed(&url, &config),
            "domain without port must NOT match domain with port"
        );
    }

    /// `test_BC_1_03_005_subdomain_not_matched`
    ///
    /// Invariant (BC-1.03.005 invariant 4 + AC-003): exact match required; subdomain not included.
    /// `api.example.com` does NOT match `sub.api.example.com`.
    ///
    /// AC-003 test vector: domains=`["api.example.com"]`, URL=https://sub.api.example.com/data → false
    #[test]
    fn test_bc_1_03_005_subdomain_not_matched() {
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("api.example.com")]),
        };
        let url = url::Url::parse("https://sub.api.example.com/data").unwrap();
        assert!(
            !is_allowed(&url, &config),
            "subdomain must NOT be matched by parent domain entry"
        );
    }

    /// `test_BC_1_03_005_multiple_domains_any_match`
    ///
    /// Postcondition: when any domain in the list matches, `is_allowed()` returns `true`.
    ///
    /// Test vector: domains=`["a.com","b.com"]`, URL=https://b.com/data → true
    #[test]
    fn test_bc_1_03_005_multiple_domains_any_match() {
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("a.com"), Arc::from("b.com")]),
        };
        let url = url::Url::parse("https://b.com/data").unwrap();
        assert!(
            is_allowed(&url, &config),
            "URL matching any entry in the list must be permitted"
        );
    }

    /// `test_BC_1_03_005_domain_with_port_matches_exactly`
    ///
    /// Postcondition: a `"host:port"` entry in `domains` matches a URL with that exact host+port.
    ///
    /// Test vector: domains=`["api.example.com:8443"]`, URL=https://api.example.com:8443/data → true
    #[test]
    fn test_bc_1_03_005_domain_with_port_matches_exactly() {
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("api.example.com:8443")]),
        };
        let url = url::Url::parse("https://api.example.com:8443/data").unwrap();
        assert!(
            is_allowed(&url, &config),
            "host:port entry must match URL with the same host:port"
        );
    }

    /// `test_BC_1_03_005_aws_metadata_blocked`
    ///
    /// Security invariant: AWS instance metadata endpoint is blocked when any allowlist is configured.
    /// (Regression guard for SSRF protection.)
    #[test]
    fn test_bc_1_03_005_aws_metadata_blocked() {
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("api.example.com")]),
        };
        let url = url::Url::parse("http://169.254.169.254/latest/meta-data/").unwrap();
        assert!(
            !is_allowed(&url, &config),
            "AWS metadata endpoint must be blocked when an allowlist is configured"
        );
    }

    /// `test_BC_1_03_005_parent_domain_not_matched`
    ///
    /// Precondition violation: listing a parent domain does NOT grant access to a subdomain.
    /// `example.com` does NOT allow `api.example.com`.
    #[test]
    fn test_bc_1_03_005_parent_domain_not_matched() {
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("example.com")]),
        };
        let url = url::Url::parse("https://api.example.com/data").unwrap();
        assert!(
            !is_allowed(&url, &config),
            "parent domain must NOT match subdomain — exact match required"
        );
    }

    /// `test_BC_1_03_005_multiple_domains_first_match`
    ///
    /// Edge case: the matched domain is the first entry (not just last-wins).
    #[test]
    fn test_bc_1_03_005_multiple_domains_first_match() {
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("a.com"), Arc::from("b.com")]),
        };
        let url = url::Url::parse("https://a.com/data").unwrap();
        assert!(
            is_allowed(&url, &config),
            "URL matching the first entry must be permitted"
        );
    }

    /// `test_BC_1_03_005_invariant_empty_domains_blocks_localhost`
    ///
    /// Invariant: empty allowlist blocks even localhost (no implicit local bypass).
    #[test]
    fn test_bc_1_03_005_invariant_empty_domains_blocks_localhost() {
        let config = AllowlistConfig {
            domains: Some(vec![]),
        };
        let url = url::Url::parse("http://localhost/data").unwrap();
        assert!(
            !is_allowed(&url, &config),
            "empty allowlist must block localhost"
        );
    }
}
