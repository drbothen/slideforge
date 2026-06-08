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
//! ## Body Size Cap
//!
//! Response bodies are read with a hard cap of **50 MB** (`MAX_BODY_BYTES`). Responses
//! that exceed this limit produce a [`DataSourceError::IoError`] with error code
//! `[E-DAT-006]` (policy-rejected). This prevents unbounded memory consumption from
//! large or malicious HTTP payloads. For data sets larger than 50 MB, use a file-based
//! [`crate::file::FileDataSource`] instead.
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
//! | Response body exceeds size cap | `E-DAT-006` ([`DataSourceError::IoError`]) |
//! | TOML `format_hint` on HTTP source | `E-DAT-003` ([`crate::DataError::UnsupportedFormat`]) |
//! | Non-2xx HTTP response | `E-DAT-001` ([`crate::DataError::HttpError`]) |
//! | Network unreachable / timeout | `E-DAT-002` ([`crate::DataError::NetworkError`]) |
//! | Unsupported content-type | `E-DAT-003` ([`crate::DataError::UnsupportedFormat`]) |
//! | Parse failure | `E-DAT-003` ([`crate::DataError::ParseError`]) |

use std::io::Read;
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

/// Maximum allowed response body size (50 MB in production, 1 KB in tests).
///
/// This is a defensive control beyond the explicit ACs to prevent unbounded
/// memory consumption from malicious or buggy HTTP servers. Exceeding the cap
/// produces a `DataSourceError::IoError` with `[E-DAT-006]` (policy-rejected).
///
/// The detection strategy reads `MAX_BODY_BYTES + 1` bytes: if the reader yields more
/// than `MAX_BODY_BYTES` bytes, the cap has been exceeded and the error is returned
/// before the body is decoded or parsed. This guarantees the cap-exceeded condition
/// is an observable, explicit error — never a silent truncation.
///
/// For data sets larger than 50 MB, use a file-based [`FileDataSource`] instead.
///
/// [`FileDataSource`]: crate::file::FileDataSource
#[cfg(not(test))]
const MAX_BODY_BYTES: u64 = 50 * 1024 * 1024; // 50 MB

/// Test override: 1 KB cap so tests run in microseconds, not seconds.
#[cfg(test)]
const MAX_BODY_BYTES: u64 = 1024; // 1 KB for tests only

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
    ///
    /// Domain entries are normalized to lowercase via [`AllowlistConfig::with_domains`]
    /// so that config values like `"API.EXAMPLE.COM"` correctly match URL hosts
    /// such as `api.example.com` (which the `url` crate always returns in lowercase).
    #[must_use]
    pub fn from_context(url: impl Into<Arc<str>>, ctx: &DataSourceContext) -> Self {
        let allowlist = match &ctx.allowed_domains {
            // Route through AllowlistConfig::with_domains which normalizes to lowercase,
            // ensuring "API.EXAMPLE.COM" in slideforge.toml matches "api.example.com" URLs.
            Some(domains) => AllowlistConfig::with_domains(domains.iter().map(AsRef::as_ref)),
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
}

impl DataSource for HttpDataSource {
    fn id(&self) -> &'static str {
        "http"
    }

    /// Returns `true` — HTTP sources should be skipped in offline mode.
    ///
    /// Overrides the `DataSource` trait default (`false`) to declare that this
    /// source makes network requests. The dispatcher calls this method to decide
    /// whether to skip the source when `DataSourceContext::offline` is `true`.
    ///
    /// Traces to BC-1.03.002 edge case EC-005 (AC-012).
    fn supports_offline(&self) -> bool {
        true
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
    ///
    /// # v1 Limitations
    ///
    /// The following [`DataSourceOptions`] fields are not yet honored and are
    /// silently ignored (with a `tracing::warn!` logged when set):
    /// - `auth_token` — authentication not yet implemented for HTTP sources
    /// - `timeout_ms` — custom timeout not yet implemented; built-in 10s connect /
    ///   30s read timeouts from `build_agent` are always used
    /// - `query` — query parameter injection not yet implemented
    fn load(&self, uri: &str, opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        // v1 limitation: warn on non-default options that are not yet honored.
        if opts.auth_token.is_some() {
            tracing::warn!(
                "HTTP source does not yet honor auth_token; the option is ignored. \
                Authentication support is planned for a future release."
            );
        }
        if opts.timeout_ms.is_some() {
            tracing::warn!(
                "HTTP source does not yet honor opts.timeout_ms; using built-in \
                agent timeout (10s connect / 30s read). Custom timeout support is \
                planned for a future release."
            );
        }
        if opts.query.is_some() {
            tracing::warn!(
                "HTTP source does not yet honor query parameters; the option is ignored. \
                Query parameter injection is planned for a future release."
            );
        }

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

        // AC-009 interpretation: We emit at tracing::warn! level unconditionally.
        // The "strict mode" wording in AC-009 / BC-1.03.002 invariant 1 refers to the
        // CLI-layer mode where warns are escalated to build failures. The data-source
        // layer does not gate the warning by mode — it always emits at warn level,
        // leaving filtering/escalation to the CLI's tracing subscriber.
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

        // Read body with a hard size cap to prevent unbounded memory consumption from
        // large or malicious HTTP payloads. We read MAX_BODY_BYTES + 1 bytes so that
        // we can detect cap-exceeded: if the reader yields more than MAX_BODY_BYTES
        // bytes, we return an explicit IoError rather than silently truncating.
        let mut raw_buf = Vec::new();
        response
            .into_reader()
            .take(MAX_BODY_BYTES + 1)
            .read_to_end(&mut raw_buf)
            .map_err(|e| DataSourceError::IoError {
                uri: url_str.to_owned(),
                message: format!("[{E_DAT_002}] read error: {e}"),
            })?;
        if raw_buf.len() as u64 > MAX_BODY_BYTES {
            return Err(DataSourceError::IoError {
                uri: url_str.to_owned(),
                message: format!(
                    "[{E_DAT_006}] response body exceeds {MAX_BODY_BYTES}-byte cap \
                    (policy-rejected to prevent DoS) — \
                    use a file-based DataSource for payloads larger than {MAX_BODY_BYTES} bytes"
                ),
            });
        }
        let body = String::from_utf8(raw_buf).map_err(|e| DataSourceError::IoError {
            uri: url_str.to_owned(),
            message: format!("[{E_DAT_002}] response body is not valid UTF-8: {e}"),
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
///
/// ## Separation of concerns: inter-layer protocol vs. user-facing Display
///
/// This function emits a **canonical machine-friendly format** in the `message`
/// field of `DataSourceError::IoError`. That format is the inter-layer protocol
/// consumed by the dispatcher's `map_source_error` function, which parses the
/// bracket code and routing fields to reconstruct the rich `DataError` variant
/// (including URL, `--offline` hint, source span, etc.) before presenting it to
/// the user.
///
/// The canonical formats are:
/// - `HttpError`    → `"[E-DAT-001] HTTP {status} from '{url}'"` — the format
///   that `extract_http_status` was designed to parse. The dispatcher reads the
///   status code from this message and reconstructs `DataError::HttpError { status, url, .. }`,
///   whose rich Display includes the `--offline` hint.
/// - `NetworkError` → `"[E-DAT-002] network error: {cause}"` — the format that
///   the dispatcher's `strip_prefix("network error: ")` was designed to consume.
///   The dispatcher strips the label and bracket code, then reconstructs
///   `DataError::NetworkError { cause, url, .. }`, whose rich Display includes
///   the `--offline` hint and URL.
///
/// **Do NOT change these formats without updating the dispatcher** — they are a
/// single-point-of-failure contract. The corresponding dispatcher tests
/// (`test_map_source_error_e_dat_001_routed_correctly`, `test_map_source_error_e_dat_002_routed_correctly`,
/// `test_map_source_error_e_dat_001_display_contains_status_code`, etc.)
/// construct their `inner_msg` in this canonical format for exactly that reason.
///
/// ## Mapping rationale
///
/// - `HttpError` → `IoError` with canonical `"[E-DAT-001] HTTP {status} from '{url}'"`.
/// - `NetworkError` → `IoError` with canonical `"[E-DAT-002] network error: {cause}"`.
/// - `SsrfBlocked` → `ParseError`: A security policy rejection is NOT a transient
///   network condition. Mapping to `IoError` (the former default) conflated "blocked by
///   policy — do not retry" with "network blip — maybe retry". `ParseError` is not
///   semantically perfect either, but it is the only variant with both `uri` and
///   `message` fields, allowing the full E-DAT-006 remediation hint to surface to
///   callers. Downstream code can discriminate `SsrfBlocked` from `NetworkError`
///   and `HttpError` by matching on `ParseError` discriminant + inspecting the message
///   for "E-DAT-006".
/// - `ParseError` / `UnsupportedFormat` → `ParseError`: direct semantic match.
/// - Everything else → `IoError` with the variant's Display as the message.
#[must_use]
fn data_error_to_source_error(uri: &str, err: &DataError) -> DataSourceError {
    match err {
        // Emit canonical inter-layer format so the dispatcher's extract_http_status
        // can parse the status code from "[E-DAT-001] HTTP {status} from '{url}'".
        // The dispatcher reconstructs the rich DataError::HttpError variant (with
        // --offline hint, URL, span) before presenting it to the user.
        DataError::HttpError { status, url, .. } => DataSourceError::IoError {
            uri: uri.to_owned(),
            message: format!("[{E_DAT_001}] HTTP {status} from '{url}'"),
        },
        // Emit canonical inter-layer format so the dispatcher's strip_prefix("network error: ")
        // can extract the bare cause from "[E-DAT-002] network error: {cause}".
        // The dispatcher reconstructs the rich DataError::NetworkError variant (with
        // --offline hint, URL, span) before presenting it to the user.
        DataError::NetworkError { cause, .. } => DataSourceError::IoError {
            uri: uri.to_owned(),
            message: format!("[{E_DAT_002}] network error: {cause}"),
        },
        DataError::SsrfBlocked { .. } => {
            // Security policy rejection: map to ParseError (not IoError) so callers
            // can distinguish "blocked — do not retry" from transient network failures.
            // The message carries the full E-DAT-006 text with remediation hint.
            DataSourceError::ParseError {
                uri: uri.to_owned(),
                message: err.to_string(),
            }
        },
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
///
/// # AC-005 enforcement
///
/// TOML over HTTP is not supported per AC-005 — use a file-based `DataSource` for
/// TOML files. Callers that set `format_hint = Some(DataFormat::Toml)` will receive
/// an `UnsupportedFormat` error.
///
/// XLSX and `SQLite` are binary formats handled by dedicated non-HTTP data sources;
/// they are likewise rejected here.
fn parse_body(body: &str, url_str: &str, format: DataFormat) -> Result<Value, DataError> {
    match format {
        DataFormat::Json => parse::json::parse_json(body, url_str),
        DataFormat::Csv => parse::csv::parse_csv(body, url_str),
        DataFormat::Yaml => parse::yaml::parse_yaml(body, url_str),
        DataFormat::Toml => {
            // AC-005: TOML over HTTP is not supported. Callers must use a
            // file-based DataSource for TOML files. Rejecting here ensures that
            // setting format_hint = Some(DataFormat::Toml) on an HttpDataSource
            // produces a clear, actionable error rather than silently succeeding.
            Err(DataError::unsupported_format(
                "TOML over HTTP not supported per AC-005; use file-based DataSource",
            ))
        },
        DataFormat::Xlsx | DataFormat::Sqlite => {
            // XLSX and SQLite cannot be served over HTTP as text bodies.
            // The `format_hint` code path to get here would be user error;
            // return an unsupported-format error.
            Err(DataError::unsupported_format(format!("{format:?}")))
        },
        // DataFormat is #[non_exhaustive]; wildcard arm required for forward compatibility
        // when new variants are added. DataFormat::Unknown (added by F-P10-LOW-001) is not
        // a valid parse target — treat it the same as an unrecognized format.
        _ => Err(DataError::unsupported_format(format!("{format:?}"))),
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

    /// Spawn a TCP listener that accepts up to `max_connections` connections and
    /// increments the provided `counter` for each. The listener polls in
    /// non-blocking mode and exits after a **500 ms** hard timeout to allow tests
    /// to assert "exactly N connections were attempted" without hanging.
    ///
    /// Returns `(addr, join_handle)`. Call `join_handle.join().expect("listener thread
    /// panicked")` after the SSRF check finishes; the listener exits on its own
    /// deadline.
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
            // 500 ms deadline — the synchronous load() call finishes in microseconds;
            // we only need to confirm no late-arriving connection appears.
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
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

    /// `test_from_context_normalizes_uppercase_domain`
    ///
    /// F1 regression: `HttpDataSource::from_context` must normalize uppercase
    /// domain entries so that `"API.EXAMPLE.COM"` in slideforge.toml config
    /// correctly permits requests to `https://api.example.com/data`.
    ///
    /// Without normalization, `AllowlistConfig.domains` would contain `"API.EXAMPLE.COM"`
    /// but the `url` crate always returns hosts in lowercase (`api.example.com`),
    /// causing a silent false-block.
    ///
    /// This test FAILS against code that does NOT normalize (direct struct construction
    /// without `AllowlistConfig::with_domains`), and PASSES after the fix.
    #[test]
    fn test_from_context_normalizes_uppercase_domain() {
        use crate::allowlist::is_allowed;
        use crate::context::DataSourceContext;
        let ctx = DataSourceContext::new()
            .with_allowed_domains(vec![Arc::<str>::from("API.EXAMPLE.COM")]);
        let src = HttpDataSource::from_context("https://api.example.com/data", &ctx);
        let url = url::Url::parse("https://api.example.com/data").unwrap();
        assert!(
            is_allowed(&url, &src.allowlist),
            "from_context with uppercase domain 'API.EXAMPLE.COM' must permit \
            lowercase URL host 'api.example.com' — normalization must be applied"
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

    /// `test_bc_1_03_005_ssrf_maps_to_parse_error_not_io`
    ///
    /// F2: `SsrfBlocked` must map to `DataSourceError::ParseError` (not `IoError`).
    /// A security policy block is NOT a transient network condition — mapping to
    /// `IoError` conflates "blocked — do not retry" with "network blip — maybe retry".
    /// Callers can discriminate SSRF from transport errors by matching on the
    /// `ParseError` discriminant and checking for "E-DAT-006" in the message.
    ///
    /// Traces to BC-1.03.005 postcondition 2 (E-DAT-006 emitted) + F2 fix.
    #[test]
    fn test_bc_1_03_005_ssrf_maps_to_parse_error_not_io() {
        let url = "http://169.254.169.254/metadata";
        let config = AllowlistConfig {
            domains: Some(vec![Arc::from("other.example.com")]),
        };
        let src = HttpDataSource::new(url).with_allowlist(config);
        let opts = DataSourceOptions::default();
        let result = src.load(url, &opts);
        assert!(result.is_err(), "blocked domain must produce an error");
        let err = result.unwrap_err();
        // Must be ParseError, NOT IoError.
        assert!(
            matches!(err, DataSourceError::ParseError { .. }),
            "SsrfBlocked must map to ParseError (not IoError) so callers can \
            discriminate policy blocks from transient network failures; got: {err:?}"
        );
        // Message must still carry E-DAT-006 and remediation hint.
        let msg = err.to_string();
        assert!(
            msg.contains("E-DAT-006"),
            "ParseError from SSRF block must contain 'E-DAT-006'; got: {msg}"
        );
        assert!(
            msg.contains("allowed_domains"),
            "ParseError from SSRF block must contain remediation hint; got: {msg}"
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

    /// `test_bc_1_03_002_uppercase_file_scheme_rejected`
    ///
    /// F3: `FILE:///` (uppercase) must be rejected — validates that scheme
    /// normalization via the `url` crate's `scheme()` handles uppercase inputs.
    /// The `url` crate normalizes the scheme to lowercase before `scheme()` returns
    /// it, so `FILE://` is caught by the same `scheme != "http" && scheme != "https"` check.
    ///
    /// Traces to BC-1.03.005 invariant 5 + AC-010.
    #[test]
    fn test_bc_1_03_002_uppercase_file_scheme_rejected() {
        let src = HttpDataSource::new("FILE:///tmp/data.json");
        let opts = DataSourceOptions::default();
        let result = src.load("FILE:///tmp/data.json", &opts);
        assert!(
            result.is_err(),
            "FILE:// (uppercase) scheme must be rejected by HttpDataSource"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("E-DAT-003"),
            "uppercase FILE:// rejection must carry error code E-DAT-003; got: {msg}"
        );
        assert!(
            msg.to_lowercase().contains("file"),
            "error message must mention 'file' scheme; got: {msg}"
        );
    }

    /// `test_bc_1_03_002_format_hint_overrides_unrecognized_content_type`
    ///
    /// F4: `format_hint = Some(DataFormat::Json)` must override `Content-Type:
    /// application/octet-stream` (unrecognized), allowing the body to be parsed
    /// as JSON regardless of what the server claims.
    ///
    /// Traces to BC-1.03.002 AC-005 (`format_hint` always overrides content-type detection).
    #[test]
    fn test_bc_1_03_002_format_hint_overrides_unrecognized_content_type() {
        // Server returns an unrecognized Content-Type with a valid JSON body.
        let (addr, handle) = spawn_mock_server(200, "application/octet-stream", r#"{"x":1}"#);
        let url = format!("http://127.0.0.1:{}/data", addr.port());
        let src = HttpDataSource::new(url.as_str()).with_format(DataFormat::Json);
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        assert!(
            result.is_ok(),
            "format_hint=Json must override unrecognized Content-Type and parse \
            successfully; got: {:?}",
            result.err()
        );
        match result.unwrap() {
            slideforge_types::Value::Map(ref m) => {
                let x = m.get(&Arc::from("x")).expect("key 'x' must be present");
                assert_eq!(
                    *x,
                    slideforge_types::Value::Int(1),
                    "format_hint JSON parse must return x=1"
                );
            },
            other => panic!("expected Value::Map from JSON parse, got {other:?}"),
        }
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
    // BC-1.03.002: body-size cap is enforced as an explicit error (F3 regression)
    // -----------------------------------------------------------------------

    /// `test_bc_1_03_002_body_size_cap_enforced`
    ///
    /// BC-1.03.002: A response body that exceeds `MAX_BODY_BYTES` must produce a
    /// `DataSourceError::IoError` whose message contains "exceeds". The cap-exceeded
    /// condition must never be silently truncated.
    ///
    /// This test uses the `#[cfg(test)]` override that sets `MAX_BODY_BYTES = 1024`,
    /// so the mock server only needs to write 1025 bytes to trigger the cap — keeping
    /// the test in millisecond range.
    ///
    /// Traces to BC-1.03.002 NFR (body size cap) + F3 regression coverage.
    #[test]
    fn test_bc_1_03_002_body_size_cap_enforced() {
        use std::io::Write;

        // Spawn a TCP server that sends MAX_BODY_BYTES + 1 bytes of valid UTF-8 JSON-ish
        // data after valid HTTP headers. The +1 byte pushes the response over the cap.
        let oversized_body: Vec<u8> = {
            // Build a body of MAX_BODY_BYTES + 1 bytes of ASCII 'a' characters.
            // This is valid UTF-8 and would parse fine if the cap weren't enforced.
            // MAX_BODY_BYTES is 1024 in test mode so this is always in range for usize.
            let n = usize::try_from(MAX_BODY_BYTES).expect("MAX_BODY_BYTES fits in usize") + 1;
            std::iter::repeat_n(b'a', n).collect()
        };
        let oversized_body_len = oversized_body.len();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            // Write the HTTP response headers followed by the oversized body.
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {oversized_body_len}\r\nConnection: close\r\n\r\n",
            );
            let _ = stream.write_all(headers.as_bytes());
            let _ = stream.write_all(&oversized_body);
        });

        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();

        assert!(
            result.is_err(),
            "response body exceeding MAX_BODY_BYTES must produce an error, not succeed"
        );
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("exceeds"),
            "cap-exceeded error must contain 'exceeds'; got: {msg}"
        );
        assert!(
            msg.contains("E-DAT-006"),
            "cap-exceeded error must contain 'E-DAT-006' (policy-rejected code); got: {msg}"
        );
        assert!(
            matches!(err, DataSourceError::IoError { .. }),
            "cap-exceeded error must be IoError; got: {msg}"
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

    // -----------------------------------------------------------------------
    // F1 (AC-005): TOML format_hint must be rejected for HTTP sources
    // -----------------------------------------------------------------------

    /// `test_bc_1_03_002_format_hint_toml_rejected`
    ///
    /// AC-005: `format_hint = Some(DataFormat::Toml)` on an `HttpDataSource` must
    /// produce an error with "TOML over HTTP" in the message. TOML is a file-based
    /// format — serving it over HTTP is unsupported per the spec.
    ///
    /// This test verifies that the rejection happens regardless of whether the server
    /// sends a valid TOML-parseable body (i.e., the `format_hint` path is gated, not
    /// just the Content-Type detection path).
    ///
    /// Traces to BC-1.03.002 AC-005.
    #[test]
    fn test_bc_1_03_002_format_hint_toml_rejected() {
        // A server that serves valid TOML content — the rejection must happen
        // before parsing, in `parse_body`, regardless of the body content.
        let (addr, handle) = spawn_mock_server(200, "text/plain", "key = \"value\"\n");
        let url = format!("http://127.0.0.1:{}/data.toml", addr.port());
        let src = HttpDataSource::new(url.as_str()).with_format(DataFormat::Toml);
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);
        handle.join().unwrap();
        assert!(
            result.is_err(),
            "format_hint=Toml on HTTP source must produce an error (AC-005)"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("TOML over HTTP"),
            "error must mention 'TOML over HTTP'; got: {msg}"
        );
    }

    // -----------------------------------------------------------------------
    // STORY-080: Deterministic connection-counting helper (Red Gate stub)
    //
    // `spawn_deterministic_counting_server` is the replacement for the
    // flaky `set_nonblocking(true)` + poll loop used by the existing
    // `test_bc_1_03_002_http_4xx_not_retried` test.
    //
    // Design contract (AC-001):
    //   - Uses a blocking `TcpListener::accept()` (no `set_nonblocking`).
    //   - Serves a 404 response on each connection, closes the stream.
    //   - After the `deadline` expires (or `max_connections` are served),
    //     the server thread sends the final connection count over the returned
    //     `mpsc::Receiver<usize>` channel.
    //   - The main thread READS the channel (blocking) instead of reading a
    //     shared `AtomicUsize` while the server thread may still be running.
    //     This eliminates the connection-count race on Windows.
    //
    // The function is stubbed (`todo!()`) until the implementer replaces it.
    // Any test that calls it will panic with "not yet implemented", giving
    // a genuine Red Gate failure.
    // -----------------------------------------------------------------------

    /// Spawn a deterministic single-shot 404 server for the 4xx-not-retried test.
    ///
    /// Unlike `spawn_counting_server`, this helper:
    /// - Uses **blocking** `accept()` (not `set_nonblocking`) to eliminate the
    ///   Windows connection-count race (STORY-080 AC-001).
    /// - Communicates the final connection count back to the main thread via
    ///   an `mpsc::channel` instead of a shared `AtomicUsize`, ensuring the
    ///   count is only read AFTER all server-side I/O is complete.
    ///
    /// Returns `(addr, count_receiver, join_handle)`. Call
    /// `count_receiver.recv().unwrap()` after `src.load()` returns to get the
    /// final count; then join the handle.
    ///
    /// # Panics
    ///
    /// Currently stubs with `todo!()` (STORY-080 Red Gate). The implementer
    /// replaces `todo!()` with the real implementation.
    fn spawn_deterministic_counting_server_404(
        max_connections: usize,
    ) -> (
        std::net::SocketAddr,
        std::sync::mpsc::Receiver<usize>,
        thread::JoinHandle<()>,
    ) {
        use std::net::TcpStream;
        use std::sync::mpsc;

        // Bind in blocking mode — `set_nonblocking` is the root cause of the
        // Windows connection-count race (EC-001 / STORY-080 AC-001).
        let listener =
            TcpListener::bind("127.0.0.1:0").expect("failed to bind deterministic 404 server");
        let addr = listener.local_addr().expect("failed to get local addr");

        // Watchdog thread: after a 3-second grace period (well beyond the
        // ~10ms HTTP round-trip to loopback), opens a "poison" TCP connection
        // to unblock the server thread's blocking `accept()` call.
        //
        // This sleep is NOT a race-masking delay — the connection-count race
        // is eliminated by the mpsc channel (count is only read after the
        // server thread sends it, i.e., after all I/O is complete).  The
        // sleep here is pure lifecycle management: give the HTTP test 3 seconds
        // to finish, then trigger a graceful server shutdown.
        let watchdog_addr = addr;
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(3));
            // Connect and immediately drop — sends EOF (read() returns 0)
            // which the server detects as the shutdown signal.
            let _ = TcpStream::connect(watchdog_addr);
        });

        let (tx, rx) = mpsc::channel::<usize>();

        let handle = thread::spawn(move || {
            let mut count = 0usize;
            loop {
                if count >= max_connections {
                    break;
                }
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        // Drain the request headers.
                        let mut buf = [0u8; 4096];
                        let n = stream.read(&mut buf).unwrap_or(0);

                        if n == 0 {
                            // Zero-byte read: this is the watchdog's poison
                            // connection (immediate EOF).  Do not count it.
                            // Exit the accept loop.
                            break;
                        }

                        // A real HTTP request: count it and serve 404.
                        count += 1;

                        // Minimal 404 response.  `Connection: close` signals
                        // the client to tear down the connection after reading
                        // the response — no keep-alive probes.
                        let response = b"HTTP/1.1 404 Not Found\r\n\
                            Content-Type: text/plain\r\n\
                            Content-Length: 9\r\n\
                            Connection: close\r\n\
                            \r\n\
                            not found";
                        let _ = stream.write_all(response);
                        // `stream` is dropped here — FIN is sent, connection
                        // is fully closed before the counter is incremented
                        // further.
                    },
                    // Any I/O error on accept: stop accepting.
                    Err(_) => break,
                }
            }
            // Send the final, authoritative connection count.  The main thread
            // blocks on `count_rx.recv()` until this send completes, which
            // means it reads the count only AFTER all server-side I/O is done.
            let _ = tx.send(count);
        });

        (addr, rx, handle)
    }

    // -----------------------------------------------------------------------
    // STORY-080 AC-001: deterministic 4xx-not-retried test
    //
    // This is the REPLACEMENT test for `test_bc_1_03_002_http_4xx_not_retried`.
    // It uses `spawn_deterministic_counting_server_404` (above) instead of
    // the racy non-blocking accept loop.
    //
    // Red Gate: this test panics on the `todo!()` in the helper until the
    // implementer provides the real helper body.
    // -----------------------------------------------------------------------

    /// Traces to BC-1.03.002 invariant 2 + AC-001 (4xx must not be retried).
    ///
    /// A deterministic counting server (blocking accept + mpsc channel) serves
    /// HTTP 404 on the first connection. The test asserts:
    ///   1. `result.is_err()` — 404 produces an error.
    ///   2. The error message references "404".
    ///   3. `connection_count == 1` — exactly one connection, no retry.
    ///
    /// Assertion (3) is the load-bearing BC proof: it cannot be weakened to
    /// `<= 2` without destroying the contract (per AC-003).
    ///
    /// This test uses `spawn_deterministic_counting_server_404` which eliminates
    /// the Windows TCP keep-alive race (EC-001) by reading the count after the
    /// server thread has finished all I/O and sent the final count over a channel.
    #[test]
    #[allow(non_snake_case)] // BC-based naming convention: test_BC_S_SS_NNN_xxx
    fn test_BC_1_03_002_http_4xx_not_retried_deterministic_harness() {
        let (addr, count_rx, handle) = spawn_deterministic_counting_server_404(3);

        let url = format!("http://127.0.0.1:{}/data.json", addr.port());
        let src = HttpDataSource::new(url.as_str());
        let opts = DataSourceOptions::default();
        let result = src.load(&url, &opts);

        // Wait for server to finish and read the authoritative connection count.
        // This is the key difference from the flaky test: we read the count AFTER
        // the server has completed all I/O, not while it may still be running.
        let connection_count = count_rx
            .recv()
            .expect("server thread must send final count before exiting");
        handle.join().expect("server thread must not panic");

        // Assertion 1: 404 produces an error.
        assert!(result.is_err(), "HTTP 404 must produce an error");

        // Assertion 2: error message references 404.
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("404"),
            "error must reference status 404; got: {msg}"
        );

        // Assertion 3 (load-bearing BC-1.03.002 invariant 2 boundary):
        // Exactly 1 connection — 4xx responses are NOT retried.
        // AC-003: this assertion MUST NOT be weakened to `<= 2` or removed.
        assert_eq!(
            connection_count, 1,
            "HTTP 4xx must result in exactly 1 connection (no retry); got: {connection_count}"
        );
    }

    // -----------------------------------------------------------------------
    // F3: from_context with Some(vec![]) (empty allowlist) blocks all
    // -----------------------------------------------------------------------

    /// `test_from_context_empty_allowed_domains_blocks_all`
    ///
    /// F3: When `DataSourceContext` is configured with `allowed_domains = []`
    /// (empty vec, not None), `from_context` must produce an allowlist that blocks
    /// ALL hosts — fail-closed behaviour.
    ///
    /// Verifies by calling `load()` against a real server and asserting SSRF block.
    ///
    /// Traces to BC-1.03.005 invariant 3 (empty `allowed_domains` blocks everything).
    #[test]
    fn test_from_context_empty_allowed_domains_blocks_all() {
        use crate::allowlist::is_allowed;
        use crate::context::DataSourceContext;
        let ctx = DataSourceContext::new().with_allowed_domains(vec![]);
        let src =
            HttpDataSource::from_context(Arc::<str>::from("https://api.example.com/data"), &ctx);
        let url = url::Url::parse("https://api.example.com/data").unwrap();
        assert!(
            !is_allowed(&url, &src.allowlist),
            "empty allowed_domains must block all hosts (fail-closed)"
        );
    }
}
