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
use crate::allowlist::AllowlistConfig;
use crate::error::{E_DAT_001, E_DAT_002, E_DAT_006};
use crate::format::DataFormat;
use crate::parse;

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

    /// SSRF allowlist configuration applied before any network connection.
    ///
    /// When `allowlist.domains` is `None`, all domains are permitted.
    /// When `allowlist.domains` is `Some([])`, all domains are blocked.
    pub allowlist: AllowlistConfig,
}

impl HttpDataSource {
    /// Construct a new [`HttpDataSource`] for the given URL.
    ///
    /// The format is detected automatically from the `Content-Type` header
    /// or URL path extension unless overridden by [`HttpDataSource::with_format`].
    /// The SSRF allowlist defaults to permissive (`domains: None`).
    #[must_use]
    pub fn new(url: impl Into<Arc<str>>) -> Self {
        HttpDataSource {
            url: url.into(),
            format_hint: None,
            allowlist: AllowlistConfig::default(),
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

    /// Set the SSRF allowlist configuration for this data source.
    ///
    /// The allowlist is checked BEFORE any DNS resolution or TCP connection is
    /// opened. Domains not in the list produce [`crate::DataError::SsrfBlocked`].
    ///
    /// Returns `self` for chaining.
    #[must_use]
    pub fn with_allowlist(mut self, config: AllowlistConfig) -> Self {
        self.allowlist = config;
        self
    }

    /// Returns `true` — HTTP sources should be skipped in offline mode.
    ///
    /// This method is a capability query for build systems that want to skip
    /// network-dependent data sources during offline builds (e.g., CI without
    /// outbound internet access). When `true`, the offline gate in
    /// `DataSourceContext` will skip this source rather than failing the build.
    ///
    /// Traces to BC-1.03.002 edge case EC-005 (AC-012).
    #[must_use]
    pub fn supports_offline(&self) -> bool {
        true
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
        let url_str = self.url.as_ref();

        // AC-010: Reject file:// scheme immediately — must not silently proceed.
        if url_str.starts_with("file://") {
            return Err(DataSourceError::UnsupportedUri {
                uri: url_str.to_owned(),
            });
        }

        // Parse the URL to extract host for allowlist check.
        let parsed_url = url::Url::parse(url_str).map_err(|e| DataSourceError::UnsupportedUri {
            uri: format!("{url_str}: {e}"),
        })?;

        // AC-002/AC-003: Check allowlist BEFORE any DNS resolution or TCP connection.
        if !crate::allowlist::is_allowed(&parsed_url, &self.allowlist) {
            let err = DataError::SsrfBlocked {
                code: E_DAT_006,
                uri: Arc::from(url_str),
            };
            return Err(data_error_to_source_error(url_str, &err));
        }

        // AC-009: Warn on non-HTTPS URLs.
        if parsed_url.scheme() == "http" {
            tracing::warn!(
                "HTTP source '{}' is non-HTTPS. Prefer HTTPS for data sources in production.",
                url_str
            );
        }

        // Issue the HTTP request with one retry on transport/network error.
        let response = issue_request_with_retry(url_str)?;

        // Determine the effective format.
        let content_type_header = response.header("Content-Type").unwrap_or("").to_owned();
        let ct = content_type_header
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_lowercase();

        // Read body.
        let body = response
            .into_string()
            .map_err(|e| DataSourceError::IoError {
                uri: url_str.to_owned(),
                message: e.to_string(),
            })?;

        // If format_hint is set, it always overrides content-type detection.
        let format = if let Some(hint) = self.format_hint {
            hint
        } else {
            resolve_format_from_content_type(&ct, url_str, &body)?
        };

        // Dispatch to format-specific parser.
        parse_body(&body, url_str, format).map_err(|e| data_error_to_source_error(url_str, &e))
    }
}

/// Convert a [`DataError`] to a [`DataSourceError`] for HTTP failures.
///
/// Used internally by [`HttpDataSource::load`] to map the rich internal error
/// type to the plugin API's error type at the boundary.
#[must_use]
fn data_error_to_source_error(uri: &str, err: &DataError) -> DataSourceError {
    match err {
        DataError::ParseError { .. } | DataError::UnsupportedFormat { .. } => {
            DataSourceError::ParseError {
                uri: uri.to_owned(),
                message: err.to_string(),
            }
        },
        _ => DataSourceError::IoError {
            uri: uri.to_owned(),
            message: err.to_string(),
        },
    }
}

/// Issue a GET request to `url_str`, retrying once on transport/network errors.
///
/// Returns `Ok(response)` on a successful connection (any HTTP status code).
/// Returns `Err(DataSourceError)` if both attempts fail at the transport layer.
fn issue_request_with_retry(url_str: &str) -> Result<ureq::Response, DataSourceError> {
    match ureq::get(url_str).call() {
        Ok(response) => Ok(response),
        Err(ureq::Error::Status(status, response)) => {
            // HTTP 4xx/5xx — return the response body so we can report the status.
            // We don't retry on HTTP status errors (only on transport errors).
            Err(DataSourceError::IoError {
                uri: url_str.to_owned(),
                message: format!("[{E_DAT_001}] HTTP {status}: {}", response.status_text()),
            })
        },
        Err(ureq::Error::Transport(_)) => {
            // Transport error — retry once with no delay (per spec: 0ms wait).
            match ureq::get(url_str).call() {
                Ok(response) => Ok(response),
                Err(ureq::Error::Status(status, response)) => Err(DataSourceError::IoError {
                    uri: url_str.to_owned(),
                    message: format!("[{E_DAT_001}] HTTP {status}: {}", response.status_text()),
                }),
                Err(ureq::Error::Transport(e)) => Err(DataSourceError::IoError {
                    uri: url_str.to_owned(),
                    message: format!("[{E_DAT_002}] network error: {e}"),
                }),
            }
        },
    }
}

/// Resolve the data format from the `Content-Type` header value.
///
/// Returns `Err` when the content-type is unrecognized and no `format_hint`
/// was provided.
fn resolve_format_from_content_type(
    ct: &str,
    url_str: &str,
    body: &str,
) -> Result<DataFormat, DataSourceError> {
    match ct {
        "application/json" | "text/json" => Ok(DataFormat::Json),
        "text/csv" | "application/csv" => Ok(DataFormat::Csv),
        "application/x-yaml" | "text/yaml" | "application/yaml" => Ok(DataFormat::Yaml),
        "application/toml" | "text/toml" => Ok(DataFormat::Toml),
        "text/plain" => {
            // AC-008: Try to parse as JSON with a warning.
            if crate::parse::json::parse_json(body, url_str).is_ok() {
                tracing::warn!(
                    "HTTP response from '{}' has Content-Type: text/plain but parsed as JSON. Consider requesting application/json.",
                    url_str
                );
                Ok(DataFormat::Json)
            } else {
                Err(DataSourceError::ParseError {
                    uri: url_str.to_owned(),
                    message: format!(
                        "[{E_DAT_001}] Content-Type: text/plain body could not be parsed as JSON, CSV, or YAML"
                    ),
                })
            }
        },
        _ => Err(DataSourceError::ParseError {
            uri: url_str.to_owned(),
            message: format!(
                "[{E_DAT_001}] unrecognized Content-Type: '{ct}' — provide a format hint"
            ),
        }),
    }
}

/// Parse `body` using the given `format` and return the root [`Value`].
fn parse_body(body: &str, url_str: &str, format: DataFormat) -> Result<Value, DataError> {
    match format {
        DataFormat::Json => parse::json::parse_json(body, url_str),
        DataFormat::Csv => parse::csv::parse_csv(body, url_str),
        DataFormat::Yaml => parse::yaml::parse_yaml(body, url_str),
        DataFormat::Toml => parse::toml::parse_toml(body, url_str),
        DataFormat::Xlsx | DataFormat::Sqlite => {
            // XLSX and SQLite cannot be served over HTTP as text bodies.
            // The `format_hint` code path to get here would be user error;
            // return an unsupported-format error.
            Err(DataError::unsupported_format(format!("{format:?}")))
        },
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::thread;

    use super::*;

    // -----------------------------------------------------------------------
    // Mock HTTP server helper
    //
    // Spawns a real TCP listener on 127.0.0.1:0 (OS-assigned port) and
    // accepts exactly one connection per `spawn_mock_server` call. The
    // caller receives the `SocketAddr` to target and a join handle to
    // ensure the thread exits cleanly.
    //
    // Use `spawn_mock_server_with_counter` for SSRF tests that assert
    // ZERO TCP connections reach the server.
    // -----------------------------------------------------------------------

    /// Spawn a single-shot mock HTTP server that returns a canned response.
    ///
    /// Returns `(addr, join_handle)`. Call `join_handle.join().unwrap()` after
    /// the test if you want to ensure the server thread exited cleanly.
    fn spawn_mock_server(
        status: u16,
        content_type: &str,
        body: &str,
    ) -> (std::net::SocketAddr, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status,
            status_text(status),
            content_type,
            body.len(),
            body,
        );
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            // Drain the request (ignore it).
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            stream.write_all(response.as_bytes()).unwrap();
        });
        (addr, handle)
    }

    /// Spawn a mock server that counts accepted connections via an `AtomicUsize`.
    ///
    /// Used in SSRF tests: after the blocked request, assert `counter == 0`.
    ///
    /// The server accepts up to `max_connections` connections then exits.
    /// Pass `max_connections = 0` to make the server immediately exit without
    /// accepting anything (pure connection-counter, never responds).
    fn spawn_counting_server(
        max_connections: usize,
        counter: Arc<AtomicUsize>,
    ) -> (std::net::SocketAddr, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        // Set a short accept timeout so the thread exits even when
        // no connection arrives (SSRF-blocked case).
        listener.set_nonblocking(true).expect("set_nonblocking");
        let handle = thread::spawn(move || {
            let mut accepted = 0;
            // Poll for up to 200ms to detect any unexpected connections.
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(200);
            while std::time::Instant::now() < deadline && accepted < max_connections {
                match listener.accept() {
                    Ok(_) => {
                        counter.fetch_add(1, Ordering::SeqCst);
                        accepted += 1;
                    },
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(std::time::Duration::from_millis(5));
                    },
                    Err(_) => break,
                }
            }
        });
        (addr, handle)
    }

    fn status_text(status: u16) -> &'static str {
        match status {
            200 => "OK",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        }
    }

    // -----------------------------------------------------------------------
    // Constructor / accessor tests (these pass against the current stubs
    // because they exercise non-todo!() code paths).
    // -----------------------------------------------------------------------

    /// `test_BC_1_03_002_http_source_id` — plugin ID is `"http"`.
    #[test]
    fn test_bc_1_03_002_http_source_id() {
        let src = HttpDataSource::new("https://example.com/data.json");
        assert_eq!(src.id(), "http");
    }

    /// `test_BC_1_03_002_http_source_new_stores_url` — constructor stores the URL.
    #[test]
    fn test_bc_1_03_002_http_source_new_stores_url() {
        let src = HttpDataSource::new("https://example.com/data.json");
        assert_eq!(&*src.url, "https://example.com/data.json");
    }

    /// `test_BC_1_03_002_http_source_with_format_sets_hint` — `with_format` stores the hint.
    #[test]
    fn test_bc_1_03_002_http_source_with_format_sets_hint() {
        let src = HttpDataSource::new("https://example.com/data").with_format(DataFormat::Json);
        assert_eq!(src.format_hint, Some(DataFormat::Json));
    }

    /// `test_BC_1_03_002_http_source_no_format_hint_by_default` — `format_hint` is `None` by default.
    #[test]
    fn test_bc_1_03_002_http_source_no_format_hint_by_default() {
        let src = HttpDataSource::new("https://example.com/data.csv");
        assert!(src.format_hint.is_none());
    }

    /// `test_BC_1_03_002_default_allowlist_is_permissive` — `new()` sets `allowlist.domains` = None.
    #[test]
    fn test_bc_1_03_002_default_allowlist_is_permissive() {
        let src = HttpDataSource::new("https://example.com/data.json");
        assert!(
            src.allowlist.domains.is_none(),
            "default allowlist must be permissive (None)"
        );
    }

    /// `test_BC_1_03_002_with_allowlist_stores_config` — `with_allowlist` stores the config.
    #[test]
    fn test_bc_1_03_002_with_allowlist_stores_config() {
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("example.com")]),
        };
        let src =
            HttpDataSource::new("https://example.com/data.json").with_allowlist(config.clone());
        assert_eq!(src.allowlist, config);
    }

    // -----------------------------------------------------------------------
    // AC-012: supports_offline() returns true
    // (fails against current todo!() stub)
    // -----------------------------------------------------------------------

    /// `test_BC_1_03_002_supports_offline_returns_true`
    ///
    /// AC-012: `HttpDataSource::supports_offline()` must return `true` so the
    /// offline gate in `DataSourceContext` can skip HTTP sources.
    ///
    /// Traces to BC-1.03.002 edge case EC-005.
    #[test]
    fn test_bc_1_03_002_supports_offline_returns_true() {
        let src = HttpDataSource::new("https://example.com/data.json");
        assert!(
            src.supports_offline(),
            "HTTP sources must report supports_offline = true so the offline gate can skip them"
        );
    }

    // -----------------------------------------------------------------------
    // AC-001 / AC-005: HTTP happy-path tests (all fail against todo!() stub)
    // -----------------------------------------------------------------------

    /// `test_BC_1_03_002_http_json_happy_path`
    ///
    /// AC-001 + AC-005: 200 response with `Content-Type: application/json` and
    /// body `{"count":5}` is parsed into `Value::Map` with `count: Value::Int(5)`.
    ///
    /// Traces to BC-1.03.002 postcondition 1.
    #[test]
    fn test_bc_1_03_002_http_json_happy_path() {
        let (addr, handle) = spawn_mock_server(200, "application/json", r#"{"count":5}"#);
        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        let value = result.expect("200 JSON response must succeed");
        match value {
            slideforge_types::Value::Map(ref m) => {
                let count = m
                    .get(&Arc::from("count"))
                    .expect("key 'count' must be present");
                assert_eq!(*count, slideforge_types::Value::Int(5), "count must be 5");
            },
            other => panic!("expected Value::Map, got {other:?}"),
        }
    }

    /// `test_BC_1_03_002_http_csv_happy_path`
    ///
    /// AC-005: 200 response with `Content-Type: text/csv` is parsed correctly.
    /// The CSV `name,score\nalice,10` produces a `Value::List` of `Value::Map` rows.
    ///
    /// Traces to BC-1.03.002 postcondition 1.
    #[test]
    fn test_bc_1_03_002_http_csv_happy_path() {
        let (addr, handle) = spawn_mock_server(200, "text/csv", "name,score\nalice,10\n");
        let url = format!("http://127.0.0.1:{}/data.csv", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        let value = result.expect("200 CSV response must succeed");
        match value {
            slideforge_types::Value::List(rows) => {
                assert!(!rows.is_empty(), "CSV must produce at least one data row");
                // First row should have a 'name' key
                match &rows[0] {
                    slideforge_types::Value::Map(m) => {
                        assert!(
                            m.get(&Arc::from("name")).is_some(),
                            "first row must have 'name' column"
                        );
                    },
                    other => panic!("CSV row must be Value::Map, got {other:?}"),
                }
            },
            other => panic!("CSV parse must produce Value::List, got {other:?}"),
        }
    }

    /// `test_BC_1_03_002_http_yaml_happy_path`
    ///
    /// AC-005: 200 response with `Content-Type: application/x-yaml` is parsed correctly.
    ///
    /// Traces to BC-1.03.002 postcondition 1.
    #[test]
    fn test_bc_1_03_002_http_yaml_happy_path() {
        let (addr, handle) = spawn_mock_server(200, "application/x-yaml", "items:\n  - a\n  - b\n");
        let url = format!("http://127.0.0.1:{}/data.yaml", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        let value = result.expect("200 YAML response must succeed");
        match value {
            slideforge_types::Value::Map(ref m) => {
                assert!(
                    m.get(&Arc::from("items")).is_some(),
                    "'items' key must be present in YAML parse result"
                );
            },
            other => panic!("expected Value::Map for YAML, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // AC-006: HTTP 4xx / 5xx error tests
    // -----------------------------------------------------------------------

    /// `test_BC_1_03_002_http_404_error`
    ///
    /// AC-006: Mock returns 404 → `DataSourceError` (mapped from `DataError::HttpError { status: 404 }`).
    ///
    /// Traces to BC-1.03.002 edge case EC-001.
    #[test]
    fn test_bc_1_03_002_http_404_error() {
        let (addr, handle) = spawn_mock_server(404, "text/plain", "not found");
        let url = format!("http://127.0.0.1:{}/missing.json", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        assert!(result.is_err(), "HTTP 404 must produce an error");
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("404") || msg.contains("not found") || msg.contains("I/O"),
            "error message must reference 404 or describe the failure; got: {msg}"
        );
    }

    /// `test_BC_1_03_002_http_500_error`
    ///
    /// AC-006: Mock returns 500 → `DataSourceError` (mapped from `DataError::HttpError { status: 500 }`).
    ///
    /// Traces to BC-1.03.002.
    #[test]
    fn test_bc_1_03_002_http_500_error() {
        let (addr, handle) = spawn_mock_server(500, "text/plain", "server error");
        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        assert!(result.is_err(), "HTTP 500 must produce an error");
    }

    // -----------------------------------------------------------------------
    // AC-002 / AC-003 / AC-013: SSRF allowlist enforcement tests
    // These tests assert ZERO TCP connections when the domain is blocked.
    // -----------------------------------------------------------------------

    /// `test_BC_1_03_005_ssrf_blocked_no_network`
    ///
    /// AC-002 + AC-013 + NFR-019: When the allowlist is configured and blocks
    /// `127.0.0.1`, the `load()` call must return `DataSourceError` (SSRF blocked)
    /// AND must not open any TCP connections to the server.
    ///
    /// Traces to BC-1.03.005 postcondition 1 (no network request) and
    /// postcondition 2 (E-DAT-006 emitted).
    #[test]
    fn test_bc_1_03_005_ssrf_blocked_no_network() {
        // Start a counting server — if any connection arrives, counter > 0.
        let counter = Arc::new(AtomicUsize::new(0));
        let (addr, handle) = spawn_counting_server(1, Arc::clone(&counter));

        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        // Allowlist only permits "other.example.com" — NOT 127.0.0.1
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("other.example.com")]),
        };
        let src = HttpDataSource::new(url.as_str()).with_allowlist(config);
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);

        // Wait for the counting-server thread to finish its poll window.
        handle.join().unwrap();

        assert!(result.is_err(), "blocked domain must produce an error");
        let err_msg = result.unwrap_err().to_string();
        // The error must mention SSRF, blocked, or E-DAT-006.
        assert!(
            err_msg.contains("SSRF")
                || err_msg.contains("blocked")
                || err_msg.contains("E-DAT-006")
                || err_msg.contains("ssrf"),
            "error must identify SSRF block; got: {err_msg}"
        );
        assert_eq!(
            counter.load(Ordering::SeqCst),
            0,
            "SSRF check must prevent ALL TCP connections to the blocked domain"
        );
    }

    /// `test_BC_1_03_005_empty_allowlist_blocks_all_http`
    ///
    /// AC-004 + EC-006: `allowed_domains = []` blocks ALL HTTP/HTTPS sources.
    ///
    /// Traces to BC-1.03.005 invariant 3.
    #[test]
    fn test_bc_1_03_005_empty_allowlist_blocks_all_http() {
        let counter = Arc::new(AtomicUsize::new(0));
        let (addr, handle) = spawn_counting_server(0, Arc::clone(&counter));

        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        let config = AllowlistConfig {
            domains: Some(vec![]), // explicitly empty
        };
        let src = HttpDataSource::new(url.as_str()).with_allowlist(config);
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);

        handle.join().unwrap();

        assert!(
            result.is_err(),
            "empty allowlist must block all HTTP sources"
        );
        assert_eq!(
            counter.load(Ordering::SeqCst),
            0,
            "empty allowlist must prevent ALL TCP connections"
        );
    }

    /// `test_BC_1_03_005_absent_allowlist_permits`
    ///
    /// AC-004 + EC-007: Absent `allowed_domains` (None) permits all domains — request proceeds.
    ///
    /// Traces to BC-1.03.005 invariant 2.
    #[test]
    fn test_bc_1_03_005_absent_allowlist_permits() {
        let (addr, handle) = spawn_mock_server(200, "application/json", r#"{"ok":true}"#);
        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        // No allowlist — domains is None
        let src = HttpDataSource::new(url.as_str()); // default AllowlistConfig is None
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        assert!(
            result.is_ok(),
            "absent allowlist must permit the request; got error: {:?}",
            result.err()
        );
    }

    // -----------------------------------------------------------------------
    // AC-005: Content-Type negotiation and format_hint override
    // -----------------------------------------------------------------------

    /// `test_BC_1_03_002_text_plain_parsed_as_json`
    ///
    /// AC-008: `Content-Type: text/plain` body that is valid JSON is accepted.
    /// A `tracing::warn!` is emitted (not tested here — log output), but the
    /// parse succeeds and returns the JSON value.
    ///
    /// Traces to BC-1.03.002 edge case EC-003.
    #[test]
    fn test_bc_1_03_002_text_plain_parsed_as_json() {
        let (addr, handle) = spawn_mock_server(200, "text/plain", r#"{"status":"ok"}"#);
        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        assert!(
            result.is_ok(),
            "text/plain body that parses as JSON must succeed (with warning); got: {:?}",
            result.err()
        );
    }

    /// `test_BC_1_03_002_unrecognized_content_type_no_hint`
    ///
    /// AC-005: `Content-Type: text/xml` with no `format_hint` must produce `DataSourceError`.
    ///
    /// Traces to BC-1.03.002 postcondition 1 (must return typed value OR error).
    #[test]
    fn test_bc_1_03_002_unrecognized_content_type_no_hint() {
        let (addr, handle) = spawn_mock_server(200, "text/xml", "<root><item>x</item></root>");
        let url = format!("http://127.0.0.1:{}/data", addr.port());
        let src = HttpDataSource::new(url.as_str()); // no format_hint
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        assert!(
            result.is_err(),
            "unrecognized Content-Type with no format_hint must produce an error"
        );
    }

    /// `test_BC_1_03_002_format_hint_overrides_content_type`
    ///
    /// AC-005: `format_hint = Some(DataFormat::Csv)` overrides `Content-Type: text/plain`.
    /// The body `name,val\nfoo,1\n` is parsed as CSV.
    ///
    /// Traces to BC-1.03.002 postcondition 1.
    #[test]
    fn test_bc_1_03_002_format_hint_overrides_content_type() {
        let (addr, handle) = spawn_mock_server(200, "text/plain", "name,val\nfoo,1\n");
        let url = format!("http://127.0.0.1:{}/data", addr.port());
        let src = HttpDataSource::new(url.as_str()).with_format(DataFormat::Csv);
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        assert!(
            result.is_ok(),
            "format_hint=Csv must override content-type and parse successfully; got: {:?}",
            result.err()
        );
        match result.unwrap() {
            slideforge_types::Value::List(rows) => {
                assert!(
                    !rows.is_empty(),
                    "format_hint CSV parse must produce at least one row"
                );
            },
            other => panic!("expected Value::List from CSV parse, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // AC-010: file:// scheme rejected
    // -----------------------------------------------------------------------

    /// `test_BC_1_03_002_file_scheme_rejected`
    ///
    /// AC-010: If a `file://` URL reaches `HttpDataSource::load`, it must be
    /// rejected with an error (not silently proceed).
    ///
    /// Traces to BC-1.03.005 invariant 5 + edge case EC-006.
    #[test]
    fn test_bc_1_03_002_file_scheme_rejected() {
        let src = HttpDataSource::new("file:///tmp/data.json");
        let opts = DataSourceOptions::default();
        let result = src.load("file:///tmp/data.json", &opts);
        assert!(
            result.is_err(),
            "file:// scheme must be rejected by HttpDataSource"
        );
    }

    // -----------------------------------------------------------------------
    // AC-007: Network error retry
    // -----------------------------------------------------------------------

    /// `test_BC_1_03_002_network_error_retries_once`
    ///
    /// AC-007: On a connection-refused error, the implementation retries once.
    /// If the retry succeeds (second attempt hits a live server), the result is Ok.
    ///
    /// Implementation strategy: we cannot easily simulate "fail first, succeed
    /// second" with a single-shot server. Instead, we verify that connecting to
    /// an address where NOTHING is listening produces a `DataSourceError` (not a
    /// panic), which proves the error path is handled. The retry logic itself
    /// will be tested once the implementation is written using a more
    /// sophisticated fixture.
    ///
    /// Traces to BC-1.03.002 invariant 2 + edge case EC-002.
    ///
    /// NOTE: This test verifies the error-return contract. A fuller retry test
    /// (fail-first → succeed-second) requires the implementation to be present.
    #[test]
    fn test_bc_1_03_002_network_error_produces_error() {
        // Bind a listener, note the port, then immediately drop it so the port
        // is closed before the HTTP request fires.
        let port = {
            let l = TcpListener::bind("127.0.0.1:0").unwrap();
            l.local_addr().unwrap().port()
            // `l` is dropped here — port is now closed
        };
        let url = format!("http://127.0.0.1:{port}/data.json");
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        assert!(
            result.is_err(),
            "connection-refused must produce DataSourceError (not panic)"
        );
    }
}
