//! HTTP/HTTPS [`DataSource`] implementation.
//!
//! [`HttpDataSource`] implements the [`slideforge_plugin_api::DataSource`] trait
//! for `http://` and `https://` URIs. It uses [`ureq`] as a synchronous HTTP
//! client and enforces the SSRF allowlist from [`crate::allowlist`].
//!
//! ## SSRF Protection
//!
//! Before issuing any HTTP request, `HttpDataSource` checks the target domain
//! against the allowlist configured in [`crate::context::DataSourceContext`].
//! Domains not in the allowlist are rejected with [`crate::DataError::SsrfBlocked`].
//!
//! ## Format Detection
//!
//! The response body format is determined by:
//! 1. The `format_hint` field if set (explicit override).
//! 2. The `Content-Type` response header (`application/json`, `text/csv`, etc.).
//! 3. The URL path extension (`.json`, `.csv`, `.yaml`, `.toml`) as fallback.
//!
//! ## Offline Mode
//!
//! [`HttpDataSource::supports_offline`] always returns `false`. HTTP sources
//! require network connectivity and cannot be satisfied from a local cache
//! in the current implementation.
//!
//! ## Error Codes
//!
//! | Condition | Error |
//! |-----------|-------|
//! | Domain not in allowlist | `E-DAT-006` ([`crate::DataError::SsrfBlocked`]) |
//! | Non-2xx HTTP response | `E-DAT-001` ([`crate::DataError::NetworkError`]) |
//! | Network unreachable / timeout | `E-DAT-002` ([`crate::DataError::NetworkError`]) |
//! | Unsupported content-type | `E-DAT-003` ([`crate::DataError::UnsupportedFormat`]) |
//! | Parse failure | `E-DAT-003` ([`crate::DataError::ParseError`]) |

use std::sync::Arc;

use slideforge_plugin_api::{DataSource, DataSourceError, DataSourceOptions};
use slideforge_types::Value;

use crate::DataError;
use crate::format::DataFormat;

/// The built-in HTTP/HTTPS data source plugin.
///
/// Fetches data over HTTP or HTTPS and parses the response body as JSON,
/// CSV, YAML, or TOML. The plugin identifier is `"http"`.
///
/// See the [module documentation](self) for details on SSRF protection,
/// format detection, and error codes.
#[derive(Debug, Clone)]
pub struct HttpDataSource {
    /// The URL to fetch. Must use `http://` or `https://` scheme.
    pub url: Arc<str>,

    /// Optional format hint that overrides content-type and URL-extension
    /// detection. When set, the response body is parsed as this format
    /// regardless of the server's `Content-Type` header.
    pub format_hint: Option<DataFormat>,
}

impl HttpDataSource {
    /// Construct a new [`HttpDataSource`] for the given URL.
    ///
    /// The format is detected automatically from the `Content-Type` header
    /// or URL path extension unless overridden by [`HttpDataSource::with_format`].
    #[must_use]
    pub fn new(url: impl Into<Arc<str>>) -> Self {
        HttpDataSource {
            url: url.into(),
            format_hint: None,
        }
    }

    /// Override the automatic format detection with an explicit [`DataFormat`].
    ///
    /// Returns `self` for chaining.
    #[must_use]
    pub fn with_format(mut self, format: DataFormat) -> Self {
        self.format_hint = Some(format);
        self
    }

    /// Returns `false` — HTTP sources always require network connectivity.
    ///
    /// This method is a capability query for build systems that want to skip
    /// network-dependent data sources during offline builds (e.g., CI without
    /// outbound internet access). HTTP sources cannot be satisfied from a
    /// local cache in the current implementation.
    #[must_use]
    pub fn supports_offline(&self) -> bool {
        todo!()
    }
}

impl DataSource for HttpDataSource {
    #[allow(clippy::unnecessary_literal_bound)]
    fn id(&self) -> &str {
        "http"
    }

    /// Fetch data from the HTTP/HTTPS URL and return the parsed [`Value`].
    ///
    /// # Steps
    ///
    /// 1. Parse the URL and extract the domain.
    /// 2. Check the domain against the SSRF allowlist.
    /// 3. Issue a synchronous GET request via [`ureq`].
    /// 4. Detect the response format from `Content-Type` or URL extension.
    /// 5. Parse the response body and return the [`Value`].
    ///
    /// # Errors
    ///
    /// Returns [`DataSourceError`] on SSRF block, HTTP error, network error,
    /// unsupported content-type, or parse failure.
    fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        todo!()
    }
}

/// Convert a [`DataError`] to a [`DataSourceError`] for HTTP failures.
///
/// Used internally by [`HttpDataSource::load`] to map the rich internal error
/// type to the plugin API's error type at the boundary.
#[must_use]
#[allow(dead_code)]
fn data_error_to_source_error(_uri: &str, _err: &DataError) -> DataSourceError {
    todo!()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// `test_BC_5_04_002_http_source_id` — plugin ID is `"http"`.
    #[test]
    fn test_bc_5_04_002_http_source_id() {
        let src = HttpDataSource::new("https://example.com/data.json");
        assert_eq!(src.id(), "http");
    }

    /// `test_BC_5_04_002_http_source_new_stores_url` — constructor stores the URL.
    #[test]
    fn test_bc_5_04_002_http_source_new_stores_url() {
        let src = HttpDataSource::new("https://example.com/data.json");
        assert_eq!(&*src.url, "https://example.com/data.json");
    }

    /// `test_BC_5_04_002_http_source_with_format_sets_hint` — `with_format` stores the hint.
    #[test]
    fn test_bc_5_04_002_http_source_with_format_sets_hint() {
        let src = HttpDataSource::new("https://example.com/data").with_format(DataFormat::Json);
        assert_eq!(src.format_hint, Some(DataFormat::Json));
    }

    /// `test_BC_5_04_002_http_source_no_format_hint_by_default` — `format_hint` is `None` by default.
    #[test]
    fn test_bc_5_04_002_http_source_no_format_hint_by_default() {
        let src = HttpDataSource::new("https://example.com/data.csv");
        assert!(src.format_hint.is_none());
    }

    /// `test_BC_5_04_002_supports_offline_panics` — `supports_offline()` is not yet implemented.
    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_bc_5_04_002_supports_offline_panics() {
        let src = HttpDataSource::new("https://example.com/data.json");
        let _ = src.supports_offline();
    }

    /// `test_BC_5_04_002_load_panics` — `load()` is not yet implemented.
    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_bc_5_04_002_load_panics() {
        let src = HttpDataSource::new("https://example.com/data.json");
        let opts = DataSourceOptions::default();
        let _ = src.load("https://example.com/data.json", &opts);
    }
}
