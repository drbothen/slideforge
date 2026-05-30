//! Evaluation context for data source operations.
//!
//! [`DataSourceContext`] carries per-evaluation configuration that is
//! threaded from the evaluator down into data source plugins. It extends
//! the plugin API's [`slideforge_plugin_api::DataSourceOptions`] with
//! slideforge-internal fields that are not part of the public plugin
//! interface (e.g., the SSRF allowlist, project root path, etc.).

use std::path::PathBuf;
use std::sync::Arc;

/// Evaluation context passed from the slideforge evaluator to data source
/// plugin calls.
///
/// This struct is an internal implementation detail of the `slideforge-data`
/// crate. It is NOT part of the public plugin API — plugins receive the
/// subset of fields exposed via [`slideforge_plugin_api::DataSourceOptions`].
///
/// ## Fields
///
/// - `base_dir` — the project root directory used for resolving relative
///   file paths in [`crate::file::FileDataSource`].
/// - `allowed_domains` — the SSRF allowlist for HTTP data sources. `None`
///   means all domains are permitted; `Some(list)` restricts to the listed
///   domains only.
/// - `offline` — when `true`, any data source that declares itself as
///   network-dependent (via the `supports_offline` flag in
///   [`crate::dispatcher`]) is skipped without making a network request.
///   File-based sources are always loaded regardless of this flag.
///   Traces to BC-1.03.004.
#[derive(Debug, Clone, Default)]
pub struct DataSourceContext {
    /// The project root directory used for resolving relative file paths.
    ///
    /// When `Some`, [`crate::file::FileDataSource`] resolves relative paths
    /// against this directory and enforces path containment. When `None`,
    /// relative paths are resolved against the current working directory and
    /// no containment check is applied.
    pub base_dir: Option<PathBuf>,

    /// The SSRF allowlist for HTTP data sources.
    ///
    /// When `None`, all domains are permitted (open mode — suitable for
    /// local development but should not be used in CI or production builds
    /// that process untrusted `.sf` files).
    ///
    /// When `Some(domains)`, only the listed domain strings are permitted.
    /// Each entry is matched against the lower-cased host component of the
    /// request URL. Subdomains are NOT implicitly included — each permitted
    /// subdomain must be listed explicitly.
    pub allowed_domains: Option<Vec<Arc<str>>>,

    /// When `true`, data sources that flag themselves as network-dependent
    /// (HTTP/HTTPS sources) are skipped without making any network request.
    ///
    /// File-based sources (`FileDataSource`, `XlsxDataSource`,
    /// `SqliteDataSource`) are always loaded regardless of this flag.
    ///
    /// Set to `true` by the CLI when the `--offline` flag is passed.
    /// Traces to BC-1.03.004.
    pub offline: bool,
}

impl DataSourceContext {
    /// Construct a [`DataSourceContext`] with all fields unset (permissive defaults).
    #[must_use]
    pub fn new() -> Self {
        DataSourceContext::default()
    }

    /// Set the base directory for relative path resolution.
    ///
    /// Returns `self` for chaining.
    #[must_use]
    pub fn with_base_dir(mut self, base_dir: PathBuf) -> Self {
        self.base_dir = Some(base_dir);
        self
    }

    /// Enable offline mode, causing network-dependent data sources to be skipped.
    ///
    /// When offline mode is enabled, sources marked as network-dependent in the
    /// [`crate::dispatcher`] are skipped without making any network request.
    /// File-based sources are unaffected.
    ///
    /// Returns `self` for chaining.
    #[must_use]
    pub fn with_offline(mut self, offline: bool) -> Self {
        self.offline = offline;
        self
    }

    /// Set the SSRF domain allowlist.
    ///
    /// Each entry is **normalized to lowercase** before storing, ensuring that
    /// user-supplied values like `"API.EXAMPLE.COM"` from `slideforge.toml` match
    /// the lowercase host components returned by the `url` crate.
    ///
    /// Returns `self` for chaining.
    #[must_use]
    pub fn with_allowed_domains(mut self, domains: Vec<Arc<str>>) -> Self {
        self.allowed_domains = Some(
            domains
                .into_iter()
                .map(|d| Arc::<str>::from(d.to_lowercase()))
                .collect(),
        );
        self
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// `test_BC_5_04_003_context_default_has_no_base_dir` — default context has no `base_dir`.
    #[test]
    fn test_bc_5_04_003_context_default_has_no_base_dir() {
        let ctx = DataSourceContext::default();
        assert!(ctx.base_dir.is_none());
    }

    /// `test_BC_5_04_003_context_default_has_no_allowed_domains` — default context permits all
    /// domains (allowlist is `None`).
    #[test]
    fn test_bc_5_04_003_context_default_has_no_allowed_domains() {
        let ctx = DataSourceContext::default();
        assert!(ctx.allowed_domains.is_none());
    }

    /// `test_BC_5_04_003_context_with_base_dir` — `with_base_dir` sets the field.
    #[test]
    fn test_bc_5_04_003_context_with_base_dir() {
        let ctx = DataSourceContext::new().with_base_dir(PathBuf::from("/tmp/project"));
        assert_eq!(ctx.base_dir, Some(PathBuf::from("/tmp/project")));
    }

    /// `test_BC_5_04_003_context_with_allowed_domains` — `with_allowed_domains` sets the list.
    #[test]
    fn test_bc_5_04_003_context_with_allowed_domains() {
        let domains = vec![Arc::from("example.com"), Arc::from("api.example.com")];
        let ctx = DataSourceContext::new().with_allowed_domains(domains);
        // Entries are stored lowercase — these are already lowercase, so unchanged.
        assert_eq!(
            ctx.allowed_domains,
            Some(vec![Arc::from("example.com"), Arc::from("api.example.com")])
        );
    }

    /// `test_BC_5_04_003_context_with_allowed_domains_normalizes_uppercase`
    ///
    /// F1 regression: `with_allowed_domains` must lowercase entries so that
    /// user config values like `"API.EXAMPLE.COM"` (from slideforge.toml) correctly
    /// match `api.example.com` URL hosts returned by the `url` crate.
    ///
    /// This test FAILS against code that stores entries without normalization,
    /// and PASSES after the lowercase fix.
    #[test]
    fn test_bc_5_04_003_context_with_allowed_domains_normalizes_uppercase() {
        let ctx = DataSourceContext::new().with_allowed_domains(vec![
            Arc::from("API.EXAMPLE.COM"),
            Arc::from("Data.Example.Com"),
        ]);
        let stored = ctx.allowed_domains.unwrap();
        assert_eq!(
            stored[0].as_ref(),
            "api.example.com",
            "with_allowed_domains must lowercase uppercase entries"
        );
        assert_eq!(
            stored[1].as_ref(),
            "data.example.com",
            "with_allowed_domains must lowercase mixed-case entries"
        );
    }
}
