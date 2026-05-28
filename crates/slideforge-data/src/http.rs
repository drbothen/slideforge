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
//! 2. The `Content-Type` response header (`application/json`, `text/csv`,
//!    `application/x-yaml`, etc.). JSON, CSV, and YAML are supported.
//!    TOML over HTTP is not supported (AC-005). Use a file-based `DataSource`
//!    for TOML files.
//!
//! ## Offline Mode
//!
//! [`HttpDataSource::supports_offline`] returns `true`, signaling to the offline
//! gate in `DataSourceContext` that HTTP sources should be skipped in offline mode.
//!
//! ## Retry Policy
//!
//! On transport errors (connection refused, timeout, etc.) the request is retried
//! once with no delay (AC-007). HTTP 5xx responses are also retried once (AC-006).
//! HTTP 4xx responses are never retried (client error — retry would not help).
//!
//! ## Context Threading
//!
//! To thread the SSRF allowlist from a [`crate::context::DataSourceContext`] into
//! an `HttpDataSource`, use [`HttpDataSource::from_context`] instead of
//! [`HttpDataSource::new`].
//!
//! ## Error Codes
//!
//! | Condition | Error |
//! |-----------|-------|
//! | Domain not in allowlist | `E-DAT-006` ([`crate::DataError::SsrfBlocked`]) |
//! | Non-2xx HTTP response | `E-DAT-001` ([`crate::DataError::HttpError`]) |
//! | Network unreachable / timeout | `E-DAT-002` ([`crate::DataError::NetworkError`]) |
//! | Unsupported content-type | `E-DAT-003` ([`crate::DataError::UnsupportedFormat`]) |
//! | Parse failure | `E-DAT-003` ([`crate::DataError::ParseError`]) |

use std::sync::Arc;
use std::time::Duration;

use slideforge_plugin_api::{DataSource, DataSourceError, DataSourceOptions};
use slideforge_types::Value;

use crate::DataError;
use crate::allowlist::AllowlistConfig;
use crate::context::DataSourceContext;
use crate::error::{E_DAT_001, E_DAT_002, E_DAT_003, E_DAT_006};
use crate::format::DataFormat;
use crate::parse;

/// The built-in HTTP/HTTPS data source plugin.
///
/// Fetches data over HTTP or HTTPS and parses the response body as JSON,
/// CSV, or YAML. The plugin identifier is `"http"`.
///
/// See the [module documentation](self) for details on SSRF protection,
/// format detection, and error codes.
#[derive(Debug, Clone)]
pub struct HttpDataSource {
    /// The default URL to fetch when the `load()` `uri` parameter is empty.
    ///
    /// In normal plugin-registry usage the `uri` parameter passed to `load()` is
    /// the authoritative source URL. `self.url` acts as a constructor-time default
    /// for ergonomic construction and for the plugin-registry enumeration path.
    pub url: Arc<str>,

    /// Optional format hint that overrides content-type detection.
    ///
    /// When set, the response body is parsed as this format regardless of
    /// the server's `Content-Type` header.
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
    /// unless overridden by [`HttpDataSource::with_format`].
    /// The SSRF allowlist defaults to permissive (`domains: None`).
    #[must_use]
    pub fn new(url: impl Into<Arc<str>>) -> Self {
        HttpDataSource {
            url: url.into(),
            format_hint: None,
            allowlist: AllowlistConfig::default(),
        }
    }

    /// Construct an [`HttpDataSource`] from a [`DataSourceContext`], threading
    /// the context's `allowed_domains` into the SSRF allowlist.
    ///
    /// Consumers MUST use this constructor when building an `HttpDataSource`
    /// from evaluator context. Using [`HttpDataSource::new`] directly bypasses
    /// the `[data].allowed_domains` policy from `slideforge.toml`.
    #[must_use]
    pub fn from_context(url: impl Into<Arc<str>>, ctx: &DataSourceContext) -> Self {
        let allowlist = match &ctx.allowed_domains {
            Some(domains) => AllowlistConfig {
                domains: Some(domains.clone()),
            },
            None => AllowlistConfig::default(),
        };
        HttpDataSource {
            url: url.into(),
            format_hint: None,
            allowlist,
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
    fn id(&self) -> &'static str {
        "http"
    }

    /// Fetch data from the HTTP/HTTPS URL and return the parsed [`Value`].
    ///
    /// The `uri` parameter is the authoritative source URL for this call.
    /// When `uri` is non-empty it takes precedence over `self.url`. When
    /// `uri` is empty, `self.url` is used as a fallback default.
    ///
    /// # Steps
    ///
    /// 1. Resolve the effective URL (prefer `uri` over `self.url`).
    /// 2. Parse the URL to extract the scheme and domain.
    /// 3. Reject any non-http/https scheme with `UnsupportedFormat`.
    /// 4. Check the domain against the SSRF allowlist.
    /// 5. Issue a synchronous GET request via [`ureq`] with one retry on 5xx or transport error.
    /// 6. Detect the response format from `Content-Type`.
    /// 7. Parse the response body and return the [`Value`].
    ///
    /// # Errors
    ///
    /// Returns [`DataSourceError`] on SSRF block, HTTP error, network error,
    /// unsupported content-type, or parse failure.
    fn load(&self, uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        // Use uri as the primary URL; fall back to self.url for ergonomic constructor usage.
        let url_str: &str = if uri.is_empty() {
            self.url.as_ref()
        } else {
            uri
        };

        // Parse the URL to extract scheme and host. The `url` crate normalizes the
        // scheme to lowercase, which also handles FILE://, FTP://, etc.
        let parsed_url = url::Url::parse(url_str).map_err(|e| DataSourceError::UnsupportedUri {
            uri: format!("{url_str}: {e}"),
        })?;

        // AC-010: Reject any scheme that is not http or https.
        // Checking scheme() after parsing (not a byte-prefix) normalizes case.
        let scheme = parsed_url.scheme();
        if scheme != "http" && scheme != "https" {
            let err = DataError::unsupported_format(scheme);
            return Err(DataSourceError::ParseError {
                uri: url_str.to_owned(),
                message: err.to_string(),
            });
        }

        // AC-002/AC-003: Check allowlist BEFORE any DNS resolution or TCP connection.
        if !crate::allowlist::is_allowed(&parsed_url, &self.allowlist) {
            let domain: Arc<str> = parsed_url
                .host_str()
                .unwrap_or(url_str)
                .to_lowercase()
                .into();
            let err = DataError::SsrfBlocked {
                code: E_DAT_006,
                url: Arc::from(url_str),
                domain,
                span: slideforge_types::SourceSpan::default(),
            };
            return Err(data_error_to_source_error(url_str, &err));
        }

        // AC-009: Warn on non-HTTPS URLs.
        if scheme == "http" {
            tracing::warn!(
                "HTTP source '{}' is non-HTTPS. Prefer HTTPS for data sources in production.",
                url_str
            );
        }

        // Issue the HTTP request with one retry on transport/network error or 5xx.
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

/// Build the shared [`ureq::Agent`] used for all HTTP requests.
///
/// Security properties baked in:
/// - `redirects(0)` — redirects are NEVER followed. A 3xx response is treated
///   as an [`DataError::HttpError`] (E-DAT-001) with the redirect status code.
///   This prevents SSRF bypass via redirect: an allowlisted server cannot
///   transparently bounce the request to a blocked domain.
/// - `timeout_connect(10s)` — prevents indefinite hang on slow-loris / slow
///   connect attacks (BC-1.03.002 NFR requirement).
/// - `timeout_read(30s)` — prevents indefinite hang during body transfer.
fn build_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .redirects(0)
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(30))
        .build()
}

/// Issue a GET request to `url_str`, retrying once on transport errors or HTTP 5xx.
///
/// Retry policy (AC-006 + AC-007):
/// - Transport errors (connection refused, timeout, DNS failure): retry once.
/// - HTTP 5xx (server error): retry once — server may be transiently overloaded.
/// - HTTP 3xx (redirect): return immediately as E-DAT-001 — redirects are
///   never followed to prevent SSRF bypass via allowlisted server redirect.
/// - HTTP 4xx (client error): return immediately — retry cannot help.
///
/// The request is issued via [`build_agent`] which enforces `redirects(0)` and
/// connect/read timeouts.
///
/// Returns `Ok(response)` on a successful 2xx response.
/// Returns `Err(DataSourceError)` if both attempts fail.
/// Map an `Ok(ureq::Response)` to an error if the status is not 2xx.
///
/// With `redirects(0)`, ureq returns 3xx responses as `Ok(response)` rather
/// than following the redirect. We must detect them here and convert them to
/// `Err(HttpError)` so the SSRF invariant holds: the redirect target is never
/// fetched regardless of allowlist membership.
fn check_response_status(
    url_str: &str,
    response: ureq::Response,
) -> Result<ureq::Response, DataSourceError> {
    let status = response.status();
    if (200..300).contains(&(status as usize)) {
        Ok(response)
    } else {
        // 3xx, 4xx, 5xx — treat as HttpError.
        // 3xx in particular: redirect not followed for SSRF safety.
        Err(data_error_to_source_error(
            url_str,
            &DataError::HttpError {
                code: E_DAT_001,
                url: Arc::from(url_str),
                status,
                span: slideforge_types::SourceSpan::default(),
            },
        ))
    }
}

/// Issue a GET request to `url_str`, retrying once on transport errors or HTTP 5xx.
///
/// Retry policy (AC-006 + AC-007):
/// - Transport errors (connection refused, timeout, DNS failure): retry once.
/// - HTTP 5xx (server error): retry once — server may be transiently overloaded.
/// - HTTP 3xx (redirect): return immediately as E-DAT-001 — redirects are
///   never followed to prevent SSRF bypass via allowlisted server redirect.
///   Note: with `redirects(0)`, ureq returns 3xx as `Ok(response)`, so we
///   detect them in [`check_response_status`] and convert to an error.
/// - HTTP 4xx (client error): return immediately — retry cannot help.
///
/// The request is issued via [`build_agent`] which enforces `redirects(0)` and
/// connect/read timeouts.
///
/// Returns `Ok(response)` on a successful 2xx response.
/// Returns `Err(DataSourceError)` if both attempts fail.
fn issue_request_with_retry(url_str: &str) -> Result<ureq::Response, DataSourceError> {
    let agent = build_agent();
    match agent.get(url_str).call() {
        Ok(response) => {
            // With redirects(0), 3xx comes back as Ok — check and reject.
            check_response_status(url_str, response)
        },
        Err(ureq::Error::Status(status, _response)) if status >= 500 => {
            // HTTP 5xx — retry once per AC-006.
            let agent2 = build_agent();
            match agent2.get(url_str).call() {
                Ok(response) => check_response_status(url_str, response),
                Err(ureq::Error::Status(status2, _)) => Err(data_error_to_source_error(
                    url_str,
                    &DataError::HttpError {
                        code: E_DAT_001,
                        url: Arc::from(url_str),
                        status: status2,
                        span: slideforge_types::SourceSpan::default(),
                    },
                )),
                Err(ureq::Error::Transport(e)) => Err(data_error_to_source_error(
                    url_str,
                    &DataError::NetworkError {
                        code: E_DAT_002,
                        url: Arc::from(url_str),
                        cause: Arc::<str>::from(e.to_string()),
                        span: slideforge_types::SourceSpan::default(),
                    },
                )),
            }
        },
        Err(ureq::Error::Status(status, _)) => {
            // HTTP 4xx (or other non-5xx errors) — return immediately, no retry.
            Err(data_error_to_source_error(
                url_str,
                &DataError::HttpError {
                    code: E_DAT_001,
                    url: Arc::from(url_str),
                    status,
                    span: slideforge_types::SourceSpan::default(),
                },
            ))
        },
        Err(ureq::Error::Transport(_)) => {
            // Transport error — retry once with no delay (per spec: 0ms wait, AC-007).
            let agent2 = build_agent();
            match agent2.get(url_str).call() {
                Ok(response) => check_response_status(url_str, response),
                Err(ureq::Error::Status(status, _)) => Err(data_error_to_source_error(
                    url_str,
                    &DataError::HttpError {
                        code: E_DAT_001,
                        url: Arc::from(url_str),
                        status,
                        span: slideforge_types::SourceSpan::default(),
                    },
                )),
                Err(ureq::Error::Transport(e)) => Err(data_error_to_source_error(
                    url_str,
                    &DataError::NetworkError {
                        code: E_DAT_002,
                        url: Arc::from(url_str),
                        cause: Arc::<str>::from(e.to_string()),
                        span: slideforge_types::SourceSpan::default(),
                    },
                )),
            }
        },
    }
}

/// Resolve the data format from the `Content-Type` header value.
///
/// Supported content-types: JSON, CSV, YAML (AC-005).
/// TOML over HTTP is not supported — use a file-based `DataSource` for TOML.
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
        "text/plain" => {
            // AC-008: Try to parse as JSON with a warning.
            if crate::parse::json::parse_json(body, url_str).is_ok() {
                tracing::warn!(
                    "HTTP response from '{}' has Content-Type: text/plain but parsed as JSON. \
                    Consider requesting application/json.",
                    url_str
                );
                Ok(DataFormat::Json)
            } else {
                Err(DataSourceError::ParseError {
                    uri: url_str.to_owned(),
                    message: format!(
                        "[{E_DAT_003}] Content-Type: text/plain body could not be parsed as JSON"
                    ),
                })
            }
        },
        _ => Err(DataSourceError::ParseError {
            uri: url_str.to_owned(),
            message: format!(
                "[{E_DAT_003}] unrecognized Content-Type: '{ct}' — provide a format hint"
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

    use tracing_test::traced_test;

    use super::*;

    // -----------------------------------------------------------------------
    // Mock HTTP server helper
    //
    // Spawns a real TCP listener on 127.0.0.1:0 (OS-assigned port) and
    // accepts exactly one connection per `spawn_mock_server` call. The
    // caller receives the `SocketAddr` to target and a join handle to
    // ensure the thread exits cleanly.
    //
    // Use `spawn_counting_server` for SSRF tests that assert ZERO TCP
    // connections reach the server.
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

    /// Spawn a mock server that accepts up to `max_connections` connections and
    /// counts each one via `counter`.
    ///
    /// The server uses a channel-based done signal to avoid the 200ms race window:
    /// it polls until `done_rx` signals or a generous 5-second hard timeout elapses.
    /// After the test sends on `done_tx`, the server stops accepting new connections.
    ///
    /// Returns `(addr, join_handle, done_tx)`.
    fn spawn_counting_server(
        max_connections: usize,
        counter: Arc<AtomicUsize>,
    ) -> (std::net::SocketAddr, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        // Non-blocking so we can poll without the accept() blocking forever.
        listener.set_nonblocking(true).expect("set_nonblocking");
        let handle = thread::spawn(move || {
            let mut accepted = 0;
            // Hard 5-second timeout prevents the thread from hanging forever,
            // while eliminating the 200ms race window of the previous design.
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while std::time::Instant::now() < deadline && accepted < max_connections.max(1) {
                match listener.accept() {
                    Ok(_) => {
                        counter.fetch_add(1, Ordering::SeqCst);
                        accepted += 1;
                        if accepted >= max_connections {
                            break;
                        }
                    },
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No connection yet — yield and retry.
                        thread::sleep(std::time::Duration::from_millis(5));
                    },
                    Err(_) => break,
                }
            }
        });
        (addr, handle)
    }

    /// Spawn a mock server that serves `n` sequential responses (for retry tests).
    ///
    /// `responses` is a vec of `(status, content_type, body)` tuples served in
    /// order. Each connection gets the next response. The server exits after all
    /// responses are consumed or a 5-second hard timeout.
    fn spawn_multi_shot_server(
        responses: Vec<(u16, &'static str, &'static str)>,
    ) -> (std::net::SocketAddr, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            for (status, content_type, body) in responses {
                let response = format!(
                    "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    status,
                    status_text(status),
                    content_type,
                    body.len(),
                    body,
                );
                let Ok((mut stream, _)) = listener.accept() else {
                    break;
                };
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let _ = stream.write_all(response.as_bytes());
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
    // Constructor / accessor tests
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
    // F12: from_context threads allowed_domains
    // -----------------------------------------------------------------------

    /// `test_from_context_threads_allowed_domains`
    ///
    /// F12: `HttpDataSource::from_context` must copy `ctx.allowed_domains` into
    /// `self.allowlist`. Verifies that the policy is propagated correctly.
    #[test]
    fn test_from_context_threads_allowed_domains() {
        use crate::context::DataSourceContext;
        let ctx = DataSourceContext::new()
            .with_allowed_domains(vec![Arc::from("example.com"), Arc::from("api.example.com")]);
        let src = HttpDataSource::from_context("https://example.com/data.json", &ctx);
        assert_eq!(
            src.allowlist.domains,
            Some(vec![Arc::from("example.com"), Arc::from("api.example.com")]),
            "from_context must thread allowed_domains into allowlist"
        );
    }

    /// `test_from_context_none_allowed_domains_is_permissive`
    ///
    /// F12: When `ctx.allowed_domains` is `None`, `from_context` produces a
    /// permissive allowlist (all domains permitted).
    #[test]
    fn test_from_context_none_allowed_domains_is_permissive() {
        use crate::context::DataSourceContext;
        let ctx = DataSourceContext::new(); // allowed_domains = None
        let src = HttpDataSource::from_context("https://example.com/data.json", &ctx);
        assert!(
            src.allowlist.domains.is_none(),
            "from_context with None allowed_domains must produce permissive allowlist"
        );
    }

    // -----------------------------------------------------------------------
    // AC-001 / AC-005: HTTP happy-path tests
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
    // AC-006: HTTP 4xx / 5xx error tests — status code preserved
    // -----------------------------------------------------------------------

    /// `test_BC_1_03_002_http_404_error`
    ///
    /// AC-006: Mock returns 404 → error message must contain "404".
    /// 4xx responses are NOT retried.
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
            msg.contains("404"),
            "error message must contain status code 404; got: {msg}"
        );
    }

    /// `test_BC_1_03_002_http_500_error`
    ///
    /// AC-006: Mock returns 500 twice (both attempts fail) → error message must
    /// contain "500". Tests that 5xx is retried once then reported.
    ///
    /// Traces to BC-1.03.002.
    #[test]
    fn test_bc_1_03_002_http_500_error() {
        // Serve 500 twice so both the initial attempt AND the retry see 500.
        let (addr, handle) = spawn_multi_shot_server(vec![
            (500, "text/plain", "server error"),
            (500, "text/plain", "server error"),
        ]);
        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        assert!(result.is_err(), "HTTP 500 must produce an error");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("500"),
            "error message must contain status code 500; got: {msg}"
        );
    }

    /// `test_BC_1_03_002_http_500_retried_succeeds`
    ///
    /// AC-006: Mock returns 500 on first attempt, 200 on second → `load()` must succeed.
    /// Verifies that 5xx responses trigger one retry.
    #[test]
    fn test_bc_1_03_002_http_500_retried_succeeds() {
        let (addr, handle) = spawn_multi_shot_server(vec![
            (500, "text/plain", "server error"),
            (200, "application/json", r#"{"retried":true}"#),
        ]);
        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        assert!(
            result.is_ok(),
            "5xx-then-200 must succeed on retry; got: {:?}",
            result.err()
        );
    }

    // -----------------------------------------------------------------------
    // AC-002 / AC-003 / AC-013: SSRF allowlist enforcement tests
    // -----------------------------------------------------------------------

    /// `test_BC_1_03_005_ssrf_blocked_no_network`
    ///
    /// AC-002 + AC-013 + NFR-019: When the allowlist blocks `127.0.0.1`, the
    /// `load()` call must return an SSRF error AND must not open any TCP connections.
    ///
    /// Traces to BC-1.03.005 postcondition 1 + postcondition 2 (E-DAT-006 emitted).
    #[test]
    fn test_bc_1_03_005_ssrf_blocked_no_network() {
        let counter = Arc::new(AtomicUsize::new(0));
        let (addr, handle) = spawn_counting_server(1, Arc::clone(&counter));

        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("other.example.com")]),
        };
        let src = HttpDataSource::new(url.as_str()).with_allowlist(config);
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);

        // Signal the counting-server to stop, then join.
        handle.join().unwrap();

        assert!(result.is_err(), "blocked domain must produce an error");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("SSRF")
                || err_msg.contains("blocked")
                || err_msg.contains("E-DAT-006")
                || err_msg.contains("ssrf")
                || err_msg.contains("allowed_domains"),
            "error must identify SSRF block; got: {err_msg}"
        );
        assert_eq!(
            counter.load(Ordering::SeqCst),
            0,
            "SSRF check must prevent ALL TCP connections to the blocked domain"
        );
    }

    /// `test_BC_1_03_005_ssrf_error_contains_remediation_hint`
    ///
    /// BC-1.03.005: SSRF error message must contain the remediation hint
    /// `"Add '<domain>' to [data].allowed_domains in slideforge.toml."`
    #[test]
    fn test_bc_1_03_005_ssrf_error_contains_remediation_hint() {
        let url = "http://169.254.169.254/metadata";
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("other.example.com")]),
        };
        let src = HttpDataSource::new(url).with_allowlist(config);
        let opts = DataSourceOptions::default();
        let result = src.load(url, &opts);
        assert!(result.is_err(), "blocked domain must produce an error");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("allowed_domains"),
            "SSRF error must mention allowed_domains; got: {msg}"
        );
        assert!(
            msg.contains("slideforge.toml"),
            "SSRF error must reference slideforge.toml; got: {msg}"
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
            domains: Some(vec![]),
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
        let src = HttpDataSource::new(url.as_str());
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

    /// `test_BC_1_03_002_text_plain_not_json_returns_error`
    ///
    /// F15: `Content-Type: text/plain` body that is NOT valid JSON must produce
    /// a `DataSourceError` with error code E-DAT-003.
    ///
    /// Traces to BC-1.03.002 edge case EC-003.
    #[test]
    fn test_bc_1_03_002_text_plain_not_json_returns_error() {
        let (addr, handle) = spawn_mock_server(200, "text/plain", "this is not json at all!!!");
        let url = format!("http://127.0.0.1:{}/data", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        assert!(
            result.is_err(),
            "text/plain non-JSON body must produce an error"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("E-DAT-003"),
            "error must carry code E-DAT-003; got: {msg}"
        );
    }

    /// `test_BC_1_03_002_unrecognized_content_type_no_hint`
    ///
    /// AC-005: `Content-Type: text/xml` with no `format_hint` must produce `DataSourceError`
    /// with error code E-DAT-003.
    ///
    /// Traces to BC-1.03.002 postcondition 1 (must return typed value OR error).
    #[test]
    fn test_bc_1_03_002_unrecognized_content_type_no_hint() {
        let (addr, handle) = spawn_mock_server(200, "text/xml", "<root><item>x</item></root>");
        let url = format!("http://127.0.0.1:{}/data", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        assert!(
            result.is_err(),
            "unrecognized Content-Type with no format_hint must produce an error"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("E-DAT-003"),
            "unrecognized content-type error must carry code E-DAT-003; got: {msg}"
        );
    }

    /// `test_BC_1_03_002_format_hint_overrides_content_type`
    ///
    /// AC-005: `format_hint = Some(DataFormat::Csv)` overrides `Content-Type: text/plain`.
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
    // AC-010: Non-http/https scheme rejected (F6 — scheme normalization)
    // -----------------------------------------------------------------------

    /// `test_BC_1_03_002_file_scheme_rejected`
    ///
    /// AC-010: If a `file://` URL reaches `HttpDataSource::load`, it must be
    /// rejected with an error (not silently proceed). Scheme check uses the
    /// url crate's normalized `scheme()`, so `FILE://` is also caught.
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

    /// `test_bc_1_03_002_ftp_scheme_rejected`
    ///
    /// AC-010: Non-http/https schemes (e.g. ftp://) must be rejected.
    #[test]
    fn test_bc_1_03_002_ftp_scheme_rejected() {
        let src = HttpDataSource::new("ftp://example.com/data.json");
        let opts = DataSourceOptions::default();
        let result = src.load("ftp://example.com/data.json", &opts);
        assert!(
            result.is_err(),
            "ftp:// scheme must be rejected by HttpDataSource"
        );
    }

    // -----------------------------------------------------------------------
    // AC-007 / F16: uri parameter is the primary URL
    // -----------------------------------------------------------------------

    /// `test_bc_1_03_002_uri_parameter_is_primary`
    ///
    /// F16: When `uri` is non-empty, it must be used as the fetch URL, not
    /// `self.url`. The result should reflect the target identified by `uri`.
    #[test]
    fn test_bc_1_03_002_uri_parameter_is_primary() {
        // self.url points to a dead port; uri points to the live server.
        let (addr, handle) = spawn_mock_server(200, "application/json", r#"{"uri_used":true}"#);
        let live_url = format!("http://127.0.0.1:{}/data.json", addr.port());
        // self.url is intentionally different (dead target).
        let src = HttpDataSource::new("http://127.0.0.1:1/dead.json");
        let opts = DataSourceOptions::default();
        // Pass the live URL as the uri parameter — it must be used.
        let result = src.load(&live_url, &opts);
        handle.join().unwrap();
        assert!(
            result.is_ok(),
            "uri parameter must override self.url; got: {:?}",
            result.err()
        );
    }

    // -----------------------------------------------------------------------
    // AC-007: Network error retry
    // -----------------------------------------------------------------------

    /// `test_BC_1_03_002_network_error_produces_error`
    ///
    /// AC-007: On a connection-refused error, the implementation retries once
    /// and the error path produces a `DataSourceError` (not a panic).
    ///
    /// Traces to BC-1.03.002 invariant 2 + edge case EC-002.
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

    /// `test_bc_1_03_002_network_error_retries_once`
    ///
    /// AC-007: On a transport error, the implementation retries once.
    /// If the retry succeeds (second attempt hits a live server), the result is Ok.
    /// Verifies that exactly 2 connection attempts are made: 1 initial + 1 retry.
    ///
    /// Strategy: a TCP server that drops the first connection immediately, then
    /// serves a valid 200 response on the second.
    ///
    /// Traces to BC-1.03.002 invariant 2 + edge case EC-002.
    #[test]
    fn test_bc_1_03_002_network_error_retries_once() {
        let attempt_counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&attempt_counter);

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = thread::spawn(move || {
            // First connection: accept but immediately drop (simulates reset/abrupt close).
            {
                let Ok((stream, _)) = listener.accept() else {
                    return;
                };
                counter_clone.fetch_add(1, Ordering::SeqCst);
                // Drop `stream` immediately — this causes the client to receive
                // a connection reset, which ureq treats as a transport error.
                drop(stream);
            }

            // Second connection: serve a proper 200 JSON response.
            {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                counter_clone.fetch_add(1, Ordering::SeqCst);
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let body = r#"{"retried":true}"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });

        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();

        let attempts = attempt_counter.load(Ordering::SeqCst);
        assert!(
            result.is_ok(),
            "retry after transport error must succeed when second attempt serves 200; got: {:?}",
            result.err()
        );
        assert_eq!(
            attempts, 2,
            "exactly 2 connection attempts must be made (1 initial + 1 retry); got: {attempts}"
        );
    }

    // -----------------------------------------------------------------------
    // F1 (AC-008): text/plain warning emission asserted via tracing-test
    // -----------------------------------------------------------------------

    /// `test_bc_1_03_002_text_plain_warning_emitted`
    ///
    /// F1: When a `text/plain` response body parses as JSON, a `tracing::warn!`
    /// containing "text/plain but parsed as JSON" and "Consider requesting
    /// application/json" must be emitted.
    ///
    /// Traces to BC-1.03.002 edge case EC-003 + AC-008.
    #[traced_test]
    #[test]
    fn test_bc_1_03_002_text_plain_warning_emitted() {
        let (addr, handle) = spawn_mock_server(200, "text/plain", r#"{"status":"ok"}"#);
        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        assert!(
            result.is_ok(),
            "text/plain JSON must succeed (with warning); got: {:?}",
            result.err()
        );
        assert!(
            logs_contain("text/plain but parsed as JSON"),
            "warning must mention 'text/plain but parsed as JSON'"
        );
        assert!(
            logs_contain("Consider requesting application/json"),
            "warning must contain 'Consider requesting application/json'"
        );
    }

    // -----------------------------------------------------------------------
    // F2 (AC-009): non-HTTPS warning emission asserted via tracing-test
    // -----------------------------------------------------------------------

    /// `test_bc_1_03_002_http_warning_emitted`
    ///
    /// F2 (AC-009): When a request uses `http://` (non-HTTPS), a `tracing::warn!`
    /// containing "HTTP source" and "Prefer HTTPS" must be emitted.
    ///
    /// Traces to BC-1.03.002 AC-009.
    #[traced_test]
    #[test]
    fn test_bc_1_03_002_http_warning_emitted() {
        let (addr, handle) = spawn_mock_server(200, "application/json", r#"{"ok":true}"#);
        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        assert!(
            result.is_ok(),
            "http:// request must succeed (with warning); got: {:?}",
            result.err()
        );
        assert!(
            logs_contain("HTTP source"),
            "warning must contain 'HTTP source'"
        );
        assert!(
            logs_contain("Prefer HTTPS"),
            "warning must contain 'Prefer HTTPS'"
        );
    }

    // -----------------------------------------------------------------------
    // F4 (AC-010): scheme rejection tests with error code assertion
    // -----------------------------------------------------------------------

    /// `test_bc_1_03_002_file_scheme_rejected_with_code`
    ///
    /// F4 (AC-010): `file://` URLs must be rejected with an error message
    /// containing "E-DAT-003" and "file".
    ///
    /// Traces to BC-1.03.005 invariant 5 + edge case EC-006.
    #[test]
    fn test_bc_1_03_002_file_scheme_rejected_with_code() {
        let src = HttpDataSource::new("file:///tmp/data.json");
        let opts = DataSourceOptions::default();
        let result = src.load("file:///tmp/data.json", &opts);
        assert!(
            result.is_err(),
            "file:// scheme must be rejected by HttpDataSource"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("E-DAT-003"),
            "file:// rejection must carry error code E-DAT-003; got: {msg}"
        );
        assert!(
            msg.contains("file"),
            "error message must mention 'file' scheme; got: {msg}"
        );
    }

    /// `test_bc_1_03_002_ftp_scheme_rejected_with_code`
    ///
    /// F4 (AC-010): `ftp://` URLs must be rejected with an error message
    /// containing "E-DAT-003" and "ftp".
    ///
    /// Traces to BC-1.03.005 invariant 5.
    #[test]
    fn test_bc_1_03_002_ftp_scheme_rejected_with_code() {
        let src = HttpDataSource::new("ftp://example.com/data.json");
        let opts = DataSourceOptions::default();
        let result = src.load("ftp://example.com/data.json", &opts);
        assert!(
            result.is_err(),
            "ftp:// scheme must be rejected by HttpDataSource"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("E-DAT-003"),
            "ftp:// rejection must carry error code E-DAT-003; got: {msg}"
        );
        assert!(
            msg.contains("ftp"),
            "error message must mention 'ftp' scheme; got: {msg}"
        );
    }

    // -----------------------------------------------------------------------
    // F5: HttpError message single-prefix assertion
    // -----------------------------------------------------------------------

    /// `test_bc_1_03_002_http_error_message_single_prefix`
    ///
    /// F5: A 404 response must produce an error message containing "[E-DAT-001]"
    /// exactly once — not double-prefixed.
    ///
    /// Traces to BC-1.03.002 edge case EC-001.
    #[test]
    fn test_bc_1_03_002_http_error_message_single_prefix() {
        let (addr, handle) = spawn_mock_server(404, "text/plain", "not found");
        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        assert!(result.is_err(), "404 must produce an error");
        let msg = result.unwrap_err().to_string();
        let count = msg.matches("[E-DAT-001]").count();
        assert_eq!(
            count, 1,
            "error message must contain '[E-DAT-001]' exactly once; got {count} times in: {msg}"
        );
    }

    // -----------------------------------------------------------------------
    // F6: Content-Type alias coverage
    // -----------------------------------------------------------------------

    /// `test_bc_1_03_002_content_type_aliases`
    ///
    /// F6: All Content-Type aliases must be recognized correctly:
    /// text/json, application/csv, text/yaml, application/yaml.
    ///
    /// Uses a for loop over alias table — one failing entry fails the whole test.
    ///
    /// Traces to BC-1.03.002 AC-005.
    #[test]
    fn test_bc_1_03_002_content_type_aliases() {
        // (content_type, body, description)
        let cases: &[(&str, &str, &str)] = &[
            (
                "text/json",
                r#"{"alias":"text_json"}"#,
                "text/json must parse as JSON",
            ),
            (
                "application/csv",
                "col\nval\n",
                "application/csv must parse as CSV",
            ),
            ("text/yaml", "key: value\n", "text/yaml must parse as YAML"),
            (
                "application/yaml",
                "key: value\n",
                "application/yaml must parse as YAML",
            ),
        ];

        for (ct, body, desc) in cases {
            let (addr, handle) = spawn_mock_server(200, ct, body);
            let url = format!("http://127.0.0.1:{}/data", addr.port());
            let src = HttpDataSource::new(url.as_str());
            let opts = DataSourceOptions::default();
            let result = src.load(&url, &opts);
            handle.join().unwrap();
            assert!(result.is_ok(), "{desc}: got error: {:?}", result.err());
        }
    }

    // -----------------------------------------------------------------------
    // BC-1.03.005 invariant 1: No traffic reaches a blocked domain via redirect
    // -----------------------------------------------------------------------

    /// `test_bc_1_03_005_redirect_to_blocked_domain_not_followed`
    ///
    /// BC-1.03.005 invariant 1: "No network traffic reaches a blocked domain."
    ///
    /// An allowlisted server at port A returns `302 Location: http://127.0.0.1:B/`.
    /// Port B is NOT in the allowlist. After `load()` returns `Err`, the connection
    /// counter on B must be 0 — the redirect was never followed.
    ///
    /// This test validates that the SSRF protection cannot be bypassed by a
    /// compromised or malicious allowlisted server issuing an open redirect.
    ///
    /// Without the `redirects(0)` fix, ureq follows up to 5 redirects silently,
    /// causing the redirect target to be fetched regardless of the allowlist.
    #[test]
    fn test_bc_1_03_005_redirect_to_blocked_domain_not_followed() {
        use std::io::Write;

        // Server A: allowlisted. Returns 302 Location pointing to server B.
        let listener_a = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr_a = listener_a.local_addr().unwrap();

        // Server B: NOT allowlisted. Counts how many TCP connections it receives.
        let counter_b = Arc::new(AtomicUsize::new(0));
        let (addr_b, handle_b) = spawn_counting_server(1, Arc::clone(&counter_b));

        // Spin up server A in a thread: serve one 302 redirect to server B.
        let redirect_response = format!(
            "HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:{}/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            addr_b.port()
        );
        let handle_a = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener_a.accept() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let _ = stream.write_all(redirect_response.as_bytes());
            }
        });

        // Configure allowlist: only allow server A's address, NOT server B.
        let allowed_host: Arc<str> = format!("127.0.0.1:{}", addr_a.port()).into();
        let config = AllowlistConfig {
            domains: Some(vec![allowed_host]),
        };
        let url_a = format!("http://127.0.0.1:{}/", addr_a.port());
        let src = HttpDataSource::new(url_a.as_str()).with_allowlist(config);
        let opts = DataSourceOptions::default();

        let result = src.load(&url_a, &opts);

        handle_a.join().unwrap();
        handle_b.join().unwrap();

        // The request must fail — 302 with redirects(0) produces E-DAT-001.
        assert!(
            result.is_err(),
            "302 redirect must produce an error when redirects are blocked"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("302") || msg.contains("E-DAT-001"),
            "error must reference the 302 status or E-DAT-001; got: {msg}"
        );

        // Critical: zero connections must have reached server B.
        assert_eq!(
            counter_b.load(Ordering::SeqCst),
            0,
            "redirect target (server B, not in allowlist) must receive ZERO TCP connections"
        );
    }

    // -----------------------------------------------------------------------
    // BC-1.03.002: 4xx responses are not retried (load-bearing counter test)
    // -----------------------------------------------------------------------

    /// `test_bc_1_03_002_http_4xx_not_retried`
    ///
    /// AC-006: HTTP 4xx responses must be returned immediately — no retry.
    ///
    /// A counting server serves a 404 on the first connection. The test asserts
    /// that exactly one connection was made (initial request only, no retry).
    ///
    /// This is the load-bearing test for the "4xx not retried" contract: a
    /// simpler test that only asserts `is_err()` cannot prove the retry count.
    ///
    /// Traces to BC-1.03.002 retry policy.
    #[test]
    fn test_bc_1_03_002_http_4xx_not_retried() {
        use std::io::Write;

        // A server that counts connections and serves 404 on each.
        let connection_counter = Arc::new(AtomicUsize::new(0));
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let counter_clone = Arc::clone(&connection_counter);

        // The server accepts up to 3 connections (more than enough to detect a retry)
        // and serves 404 for each. If only 1 connection is made, no retry happened.
        let handle = thread::spawn(move || {
            // Accept up to 3 connections with a 2-second hard timeout per accept.
            listener
                .set_nonblocking(true)
                .expect("set_nonblocking failed");
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
            let mut accepted = 0usize;
            while std::time::Instant::now() < deadline && accepted < 3 {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        counter_clone.fetch_add(1, Ordering::SeqCst);
                        accepted += 1;
                        let mut buf = [0u8; 4096];
                        let _ = stream.read(&mut buf);
                        let response = b"HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 9\r\nConnection: close\r\n\r\nnot found";
                        let _ = stream.write_all(response);
                    },
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(std::time::Duration::from_millis(5));
                    },
                    Err(_) => break,
                }
            }
        });

        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();

        // Must be an error.
        assert!(result.is_err(), "HTTP 404 must produce an error");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("404"),
            "error must reference status 404; got: {msg}"
        );

        // Critical: exactly 1 connection — no retry on 4xx.
        assert_eq!(
            connection_counter.load(Ordering::SeqCst),
            1,
            "HTTP 4xx must result in exactly 1 connection (no retry); got: {}",
            connection_counter.load(Ordering::SeqCst)
        );
    }
}
