//! Dispatcher for loading all configured `DataSource` instances.
//!
//! STORY-021 / BC-1.03.004.
//!
//! # Design: trait-method offline gate
//!
//! The [`load_all`] function calls `source.supports_offline()` on each source to
//! determine whether to skip it when `ctx.offline == true`. This is the coordination
//! protocol mandated by the story spec and the plugin-first architecture principle:
//!
//! > "All functionality goes through plugin traits defined in `slideforge-plugin-api`.
//! > If a bundled plugin needs to bypass the trait API, the API is wrong and must be fixed."
//!
//! `DataSource::supports_offline()` defaults to `false` (file-like), so all existing
//! implementors are backward-compatible without any changes. `HttpDataSource` overrides
//! to `true`. Third-party plugin authors override to `true` for any network-dependent source.
//!
//! # Offline gate semantics
//!
//! When `ctx.offline == true` and `source.supports_offline() == true`:
//! - The source is skipped silently.
//! - The source's data name is NOT added to the scope map.
//! - No error is emitted for the skip.
//! - Any `{{ name.field }}` reference in the evaluator will produce
//!   `E-EVL-001` (undefined variable) — the evaluator is responsible for
//!   that error, not this dispatcher.
//!
//! When `ctx.offline == false`, ALL sources are loaded unconditionally.
//!
//! # Error-code mapping (`DataSourceError` → `DataError`)
//!
//! The `DataSource` trait returns [`slideforge_plugin_api::DataSourceError`], which
//! carries the error codes embedded in the `message` field as `[E-DAT-NNN]` bracket
//! prefixes. The dispatcher maps these back to the appropriate [`DataError`] variant by
//! inspecting the `DataSourceError` discriminant and extracting the bracket prefix from
//! the message.
//!
//! Rationale for this mapping approach (DECISION-2):
//! - Smallest change surface: no new fields on `DataSourceError` in `slideforge-plugin-api`.
//! - All built-in sources already embed the bracket code in the message (established by
//!   STORY-018/019/020). Third-party plugins that follow the convention get correct mapping.
//! - Pattern-match on the discriminant first to determine the error category; then
//!   inspect the message prefix to route to the most specific `DataError` variant.
//! - Fallback: sources whose messages do NOT embed a bracket code receive a generic
//!   mapping that preserves the full error message without losing information.
//!
//! # Partial-load semantics
//!
//! If a non-skipped source fails to load (e.g., file not found), its error is
//! appended to the returned `Vec<DataError>` and iteration continues. The scope
//! map will contain only the successfully loaded sources. The caller decides
//! whether to abort (strict mode) or continue with the partial scope
//! (warn-only mode).
//!
//! # Return value
//!
//! `(scope, errors)` where:
//! - `scope: IndexMap<Arc<str>, Value>` — successfully loaded data, keyed by
//!   the data name declared in the `@data` directive.
//! - `errors: Vec<DataError>` — one entry per source that failed to load.

use std::sync::Arc;

use indexmap::IndexMap;
use slideforge_plugin_api::{DataSource, DataSourceError, DataSourceOptions};
use slideforge_types::Value;

use crate::DataError;
use crate::context::DataSourceContext;
use crate::error::{E_DAT_001, E_DAT_002, E_DAT_003, E_DAT_004, E_DAT_006};

/// Load all configured data sources, applying the offline gate.
///
/// # Parameters
///
/// - `sources` — name-source pairs: `(data_name, source)`. Each source declares
///   its own offline capability via [`DataSource::supports_offline`]. This is the
///   canonical coordination protocol — no out-of-band boolean flags are accepted.
/// - `ctx` — the evaluation context carrying the offline flag and SSRF allowlist.
///
/// # Offline gate
///
/// For each source where `source.supports_offline() == true` AND `ctx.offline == true`,
/// the source is skipped: no network request is made, and the source's data
/// name is absent from the returned scope.
///
/// # Partial-load semantics
///
/// On source load failure, the error is collected and iteration continues.
/// The returned scope contains all SUCCESSFULLY loaded sources.
///
/// # Errors
///
/// Never returns `Err` itself. Errors from individual source loads are
/// accumulated in the returned `Vec<DataError>` and passed to the caller for
/// decision (abort-in-strict-mode vs. continue-in-warn-mode).
#[must_use]
pub fn load_all(
    sources: &[(Arc<str>, Box<dyn DataSource>)],
    ctx: &DataSourceContext,
) -> (IndexMap<Arc<str>, Value>, Vec<DataError>) {
    let mut scope: IndexMap<Arc<str>, Value> = IndexMap::new();
    let mut errors: Vec<DataError> = Vec::new();

    let opts = DataSourceOptions::default();

    for (name, source) in sources {
        // Offline gate (BC-1.03.004 postcondition 1 + 2 + invariant 1):
        // Skip network-capable sources silently when in offline mode.
        // `supports_offline()` is the canonical trait-method coordination protocol.
        // The source's data name is NOT added to scope — no empty-value substitution.
        if ctx.offline && source.supports_offline() {
            continue;
        }

        // Call the source with an empty URI so each source uses its own pre-configured
        // address:
        // - `HttpDataSource`: when `uri` is empty it falls back to `self.url`
        //   (the URL baked in at construction time from the `@data` directive).
        // - `FileDataSource`: when `uri` is empty it falls back to `self.path`
        //   (the path baked in at construction time via `FileDataSource::new(path)`).
        // - `XlsxDataSource` / `SqliteDataSource`: both have an internal path/query
        //   baked in at construction; `load("", opts)` uses those fields.
        // - Mock sources in tests: all mocks ignore the `uri` parameter and return
        //   their pre-configured value regardless.
        //
        // Passing the data scope name as the URI was tried but breaks SSRF detection:
        // `HttpDataSource::load("imds", opts)` tries to parse "imds" as a URL, which
        // fails with "relative URL without a base" before the SSRF allowlist is checked.
        match source.load("", &opts) {
            Ok(value) => {
                scope.insert(Arc::clone(name), value);
            },
            Err(e) => {
                errors.push(map_source_error(name, &e));
            },
        }
    }

    (scope, errors)
}

/// Convert a [`slideforge_plugin_api::DataSourceError`] to a [`DataError`].
///
/// Maps the plugin-API error type (returned by `DataSource::load`) to the
/// crate-internal `DataError` type that carries the structured error codes
/// required by AC-009 and the error taxonomy.
///
/// ## Mapping strategy (DECISION-2)
///
/// The built-in sources embed error codes as `[E-DAT-NNN]` bracket prefixes
/// inside the `message` field of [`DataSourceError`]. The mapping strategy:
///
/// 1. Match on the `DataSourceError` discriminant for the broad error category.
/// 2. Inspect the message for the specific bracket code to route to the most
///    precise `DataError` variant.
/// 3. Wrap the full error message in the target variant so no information is lost.
/// 4. Thread `name` (the `@data` binding name) into each error for debuggability.
///
/// This approach avoids adding a new structural field to `DataSourceError` in
/// `slideforge-plugin-api` while still preserving the error codes surfaced in
/// `DataError::Display` for AC-009 verification.
///
/// ## Code-to-variant routing
///
/// | Bracket code in message | Routed to variant |
/// |-------------------------|-------------------|
/// | `[E-DAT-001]`           | `DataError::HttpError` |
/// | `[E-DAT-002]`           | `DataError::NetworkError` |
/// | `[E-DAT-004]`           | `DataError::FileNotFound` |
/// | `[E-DAT-006]` (in IoError)   | `DataError::IoError` (body-size cap, policy-rejected) |
/// | `[E-DAT-006]` (in ParseError)| `DataError::SsrfBlocked` |
/// | `[E-DAT-003]`           | `DataError::ParseError` |
/// | *(no bracket code)*     | Generic `DataError::IoError` with full message |
#[must_use]
fn map_source_error(name: &Arc<str>, err: &DataSourceError) -> DataError {
    let message: Arc<str> = Arc::from(err.to_string());

    match err {
        DataSourceError::IoError {
            uri,
            message: inner_msg,
        } => {
            // Route by bracket code embedded in the message.
            // Priority order: most-specific codes first.
            if inner_msg.contains(E_DAT_001) || message.contains(E_DAT_001) {
                // HTTP 4xx/5xx response (non-2xx). Extract the status code from the
                // bracket-coded message. Message format from HttpDataSource:
                // "[E-DAT-001] HTTP <status> from '<url>'"
                let status = extract_http_status(inner_msg).unwrap_or(0);
                DataError::HttpError {
                    code: E_DAT_001,
                    url: Arc::from(uri.as_str()),
                    status,
                    span: slideforge_types::SourceSpan::default(),
                }
            } else if inner_msg.contains(E_DAT_002) || message.contains(E_DAT_002) {
                // Network/transport error (connection refused, timeout, DNS failure,
                // body-read I/O error, non-UTF-8 body). The inner message is the
                // human-readable cause.
                DataError::NetworkError {
                    code: E_DAT_002,
                    url: Arc::from(uri.as_str()),
                    cause: Arc::from(inner_msg.as_str()),
                    span: slideforge_types::SourceSpan::default(),
                }
            } else if inner_msg.contains(E_DAT_004) || message.contains(E_DAT_004) {
                // File-not-found: surface as FileNotFound.
                // FINDING-4 fix: extract the actual missing path from the bracket-coded
                // message tail (after "[E-DAT-004] file not found: <path>") so users
                // see the real path, not just the URI.
                let actual_path = extract_path_after_code(inner_msg).unwrap_or(uri.as_str());
                DataError::FileNotFound {
                    code: E_DAT_004,
                    path: Arc::from(actual_path),
                    span: slideforge_types::SourceSpan::default(),
                }
            } else if inner_msg.contains(E_DAT_006) || message.contains(E_DAT_006) {
                // FINDING-2 fix: E-DAT-006 in an IoError arm means the HTTP source's
                // response-body-size cap was exceeded (policy-rejected). This is NOT a
                // path traversal — route to a generic IoError that preserves the
                // bracket code verbatim and does NOT claim "path traversal blocked".
                DataError::IoError {
                    code: E_DAT_006,
                    path: Arc::from(uri.as_str()),
                    message: Arc::from(inner_msg.as_str()),
                    span: slideforge_types::SourceSpan::default(),
                }
            } else {
                // Generic I/O error: preserve full message. Include the binding name
                // so error messages identify which `@data` binding failed (FINDING-6).
                DataError::IoError {
                    code: E_DAT_004,
                    path: Arc::from(uri.as_str()),
                    message: Arc::from(format!("data binding '{name}': {message}")),
                    span: slideforge_types::SourceSpan::default(),
                }
            }
        },

        DataSourceError::ParseError {
            uri,
            message: inner_msg,
        } => {
            // SSRF blocks are mapped to ParseError (not IoError) in the HTTP source
            // (F2 fix in STORY-019) so callers can distinguish policy blocks from
            // transient network failures. Re-map them back to SsrfBlocked here so
            // the dispatcher's DataError carries the correct E-DAT-006 code.
            if inner_msg.contains(E_DAT_006) || message.contains(E_DAT_006) {
                // Reconstruct SsrfBlocked: extract domain from the full message if possible;
                // fall back to using the URI as the domain.
                let domain: Arc<str> = Arc::from(uri.as_str());
                DataError::SsrfBlocked {
                    code: E_DAT_006,
                    url: Arc::from(uri.as_str()),
                    domain,
                    span: slideforge_types::SourceSpan::default(),
                }
            } else {
                // Generic parse or unsupported-format error.
                // FINDING-3 fix: infer format from URI extension instead of
                // hardcoding DataFormat::Json as a placeholder.
                let format = crate::format::DataFormat::from_path(std::path::Path::new(uri))
                    .unwrap_or(crate::format::DataFormat::Json);
                DataError::ParseError {
                    code: E_DAT_003,
                    path: Arc::from(uri.as_str()),
                    format,
                    reason: Arc::clone(&message),
                    span: slideforge_types::SourceSpan::default(),
                }
            }
        },

        DataSourceError::UnsupportedUri { uri } => DataError::UnsupportedFormat {
            code: E_DAT_003,
            extension: Arc::from(uri.as_str()),
            span: slideforge_types::SourceSpan::default(),
        },

        DataSourceError::AuthError { uri } => {
            // Authentication failures are mapped to NetworkError (E-DAT-002) since
            // they represent an access-layer failure, not a parse failure.
            // Thread the binding name for debuggability (FINDING-6).
            DataError::NetworkError {
                code: E_DAT_002,
                url: Arc::from(uri.as_str()),
                cause: Arc::from(format!("data binding '{name}': {err}")),
                span: slideforge_types::SourceSpan::default(),
            }
        },
    }
}

/// Extract the HTTP status code from a bracket-coded message of the form
/// `"[E-DAT-001] HTTP <status> from '<url>'"`.
///
/// Returns `None` if the message does not contain a parseable three-digit status.
fn extract_http_status(message: &str) -> Option<u16> {
    // Look for "HTTP " followed by a numeric token.
    let after_http = message.find("HTTP ")?.checked_add(5)?;
    let rest = message.get(after_http..)?;
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest.get(..end)?.parse().ok()
}

/// Extract the path component from a bracket-coded message of the form
/// `"[E-DAT-NNN] <label>: <path>"` or `"[E-DAT-NNN] <label>: <path> (at ...)"`.
///
/// The bracket code format used by built-in sources is `[E-DAT-NNN]` (no colon
/// inside the bracket), so the content after the bracket starts with a space
/// followed by the label+path. This function skips past the closing bracket
/// and any label prefix to extract the bare path.
///
/// Returns `None` if the expected pattern is not found.
fn extract_path_after_code(message: &str) -> Option<&str> {
    // Bracket codes end with ']'. Find the first ']' and skip past it and
    // any leading space so we're positioned at the label + path content.
    let close_bracket = message.find(']')?;
    let after_bracket = message.get(close_bracket.checked_add(1)?..)?.trim_start();
    // Strip everything after " (at " if present (span annotation).
    let with_label = if let Some(at_pos) = after_bracket.find(" (at ") {
        after_bracket.get(..at_pos)?
    } else {
        after_bracket
    };
    // Strip known label prefixes that precede the path.
    let clean = with_label
        .trim_start_matches("file not found: ")
        .trim_start_matches("I/O error: ")
        .trim();
    if clean.is_empty() { None } else { Some(clean) }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::unnecessary_literal_bound
)]
mod tests {
    use std::sync::Arc;

    use slideforge_plugin_api::{DataSource, DataSourceError, DataSourceOptions};
    use slideforge_types::Value;

    use super::*;
    use crate::context::DataSourceContext;

    // ---------------------------------------------------------------------------
    // Stub DataSources for dispatcher unit tests.
    //
    // FINDING-7 fix: `id()` returns a static plugin-type identifier per the
    // `DataSource` trait contract ("lowercase ASCII with hyphens, e.g. 'json',
    // 'csv', 'http'"). The binding name is NOT the plugin id.
    // ---------------------------------------------------------------------------

    /// Minimal file-like stub (supports_offline = false, the default).
    struct StubSource {
        result: Result<Value, DataSourceError>,
    }

    impl DataSource for StubSource {
        /// Plugin type identifier — static string, NOT the binding name.
        fn id(&self) -> &str {
            "stub-file"
        }

        fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
            match &self.result {
                Ok(v) => Ok(v.clone()),
                Err(e) => Err(match e {
                    DataSourceError::IoError { uri, message } => DataSourceError::IoError {
                        uri: uri.clone(),
                        message: message.clone(),
                    },
                    DataSourceError::ParseError { uri, message } => DataSourceError::ParseError {
                        uri: uri.clone(),
                        message: message.clone(),
                    },
                    DataSourceError::UnsupportedUri { uri } => {
                        DataSourceError::UnsupportedUri { uri: uri.clone() }
                    },
                    DataSourceError::AuthError { uri } => {
                        DataSourceError::AuthError { uri: uri.clone() }
                    },
                }),
            }
        }
    }

    /// Network-capable stub (supports_offline = true, simulates HTTP source).
    struct OnlineStubSource {
        result: Result<Value, DataSourceError>,
    }

    impl DataSource for OnlineStubSource {
        /// Plugin type identifier — static string, NOT the binding name.
        fn id(&self) -> &str {
            "stub-http"
        }

        fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
            match &self.result {
                Ok(v) => Ok(v.clone()),
                Err(e) => Err(match e {
                    DataSourceError::IoError { uri, message } => DataSourceError::IoError {
                        uri: uri.clone(),
                        message: message.clone(),
                    },
                    DataSourceError::ParseError { uri, message } => DataSourceError::ParseError {
                        uri: uri.clone(),
                        message: message.clone(),
                    },
                    DataSourceError::UnsupportedUri { uri } => {
                        DataSourceError::UnsupportedUri { uri: uri.clone() }
                    },
                    DataSourceError::AuthError { uri } => {
                        DataSourceError::AuthError { uri: uri.clone() }
                    },
                }),
            }
        }

        fn supports_offline(&self) -> bool {
            true
        }
    }

    /// `test_load_all_empty_sources` — zero sources returns empty scope and zero errors.
    #[test]
    fn test_load_all_empty_sources() {
        let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![];
        let ctx = DataSourceContext::new();
        let (scope, errors) = load_all(&sources, &ctx);
        assert!(scope.is_empty());
        assert!(errors.is_empty());
    }

    /// `test_load_all_online_loads_all` — all sources loaded when offline=false.
    #[test]
    fn test_load_all_online_loads_all() {
        let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![
            (
                Arc::from("a"),
                Box::new(StubSource {
                    result: Ok(Value::Int(1)),
                }),
            ),
            (
                Arc::from("b"),
                Box::new(OnlineStubSource {
                    result: Ok(Value::Int(2)),
                }),
            ),
        ];
        let ctx = DataSourceContext::new();
        let (scope, errors) = load_all(&sources, &ctx);
        assert_eq!(scope.len(), 2);
        assert!(errors.is_empty());
    }

    /// `test_load_all_offline_skips_capable` — offline gate skips `supports_offline=true` sources.
    ///
    /// FINDING-8 fix: also assert the skipped binding is absent from scope AND
    /// the loaded binding IS in scope when mixing offline-capable and file sources.
    #[test]
    fn test_load_all_offline_skips_capable() {
        let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![
            (
                Arc::from("file_src"),
                Box::new(StubSource {
                    result: Ok(Value::Int(7)),
                }),
            ),
            (
                Arc::from("net"),
                Box::new(OnlineStubSource {
                    result: Ok(Value::Int(99)),
                }),
            ),
        ];
        let ctx = DataSourceContext::new().with_offline(true);
        let (scope, errors) = load_all(&sources, &ctx);
        // The online (supports_offline=true) source must be absent.
        assert!(
            scope.get(&Arc::from("net")).is_none(),
            "supports_offline source 'net' must be skipped — not in scope"
        );
        // The file source must be present.
        assert!(
            scope.contains_key(&Arc::from("file_src")),
            "file source 'file_src' must still be loaded when offline=true"
        );
        assert!(errors.is_empty(), "skip must produce zero errors");
    }

    /// `test_load_all_partial_error` — failed source adds to error vec; successful source in scope.
    ///
    /// FINDING-8 fix: assert error code and message content, not just error count.
    #[test]
    fn test_load_all_partial_error() {
        let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![
            (
                Arc::from("ok"),
                Box::new(StubSource {
                    result: Ok(Value::Int(1)),
                }),
            ),
            (
                Arc::from("bad"),
                Box::new(StubSource {
                    result: Err(DataSourceError::IoError {
                        uri: "bad".to_owned(),
                        message: format!("[{E_DAT_004}] file not found: /real/path.json"),
                    }),
                }),
            ),
        ];
        let ctx = DataSourceContext::new();
        let (scope, errors) = load_all(&sources, &ctx);
        assert!(
            scope.contains_key(&Arc::from("ok")),
            "successful source must be in scope"
        );
        assert!(
            !scope.contains_key(&Arc::from("bad")),
            "failed source must not be in scope"
        );
        assert_eq!(errors.len(), 1, "exactly one error expected");
        // FINDING-8: assert the code and message content, not just the count.
        assert_eq!(
            errors[0].code(),
            "E-DAT-004",
            "partial-load error must carry code E-DAT-004; got: {}",
            errors[0].code()
        );
        assert!(
            errors[0].to_string().contains("file not found")
                || errors[0].to_string().contains("E-DAT-004"),
            "error message must reference file-not-found condition; got: {}",
            errors[0]
        );
    }

    // ---------------------------------------------------------------------------
    // Unit tests for map_source_error routing (FINDING-1, 2, 3, 4)
    // ---------------------------------------------------------------------------

    /// Helper: build a named source slice with one StubSource returning the given error.
    fn single_error_source(
        name: &str,
        err: DataSourceError,
    ) -> Vec<(Arc<str>, Box<dyn DataSource>)> {
        vec![(Arc::from(name), Box::new(StubSource { result: Err(err) }))]
    }

    /// `test_map_source_error_e_dat_001_routed_correctly`
    ///
    /// FINDING-1: IoError with [E-DAT-001] in message must produce code "E-DAT-001",
    /// NOT "E-DAT-004". Verifies HttpError routing in `map_source_error`.
    #[test]
    fn test_map_source_error_e_dat_001_routed_correctly() {
        let sources = single_error_source(
            "api",
            DataSourceError::IoError {
                uri: "http://example.com/data.json".to_owned(),
                message: "[E-DAT-001] HTTP 404 from 'http://example.com/data.json'".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].code(),
            "E-DAT-001",
            "E-DAT-001 in IoError message must route to HttpError; got code: {}, display: {}",
            errors[0].code(),
            errors[0]
        );
    }

    /// `test_map_source_error_e_dat_002_routed_correctly`
    ///
    /// FINDING-1: IoError with [E-DAT-002] in message must produce code "E-DAT-002",
    /// NOT "E-DAT-004". Verifies NetworkError routing in `map_source_error`.
    #[test]
    fn test_map_source_error_e_dat_002_routed_correctly() {
        let sources = single_error_source(
            "live",
            DataSourceError::IoError {
                uri: "http://example.com/".to_owned(),
                message: "[E-DAT-002] connection refused: tcp connect error".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].code(),
            "E-DAT-002",
            "E-DAT-002 in IoError message must route to NetworkError; got code: {}, display: {}",
            errors[0].code(),
            errors[0]
        );
    }

    /// `test_map_source_error_e_dat_006_io_error_not_path_traversal`
    ///
    /// FINDING-2: IoError with [E-DAT-006] in message (HTTP body-size cap) must NOT
    /// produce a Display containing "path traversal blocked" or "outside base dir".
    /// It must route to IoError variant (code E-DAT-006), not PathTraversalBlocked.
    #[test]
    fn test_map_source_error_e_dat_006_io_error_not_path_traversal() {
        let sources = single_error_source(
            "bigdata",
            DataSourceError::IoError {
                uri: "http://example.com/large.json".to_owned(),
                message: "[E-DAT-006] response body exceeds 52428800-byte cap (policy-rejected) — \
                    use a file-based DataSource for payloads larger than 52428800 bytes"
                    .to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].code(),
            "E-DAT-006",
            "body-cap IoError must carry E-DAT-006; got: {}",
            errors[0].code()
        );
        let display = errors[0].to_string();
        assert!(
            !display.contains("path traversal"),
            "body-cap error must NOT say 'path traversal'; got: {display}"
        );
        assert!(
            !display.contains("outside base dir"),
            "body-cap error must NOT say 'outside base dir'; got: {display}"
        );
    }

    /// `test_map_source_error_parse_error_format_inferred_from_extension`
    ///
    /// FINDING-3: ParseError for a CSV URI must surface DataFormat::Csv, not Json.
    /// Verifies the format-inference fix in `map_source_error`.
    #[test]
    fn test_map_source_error_parse_error_format_inferred_from_extension() {
        let sources = single_error_source(
            "sales",
            DataSourceError::ParseError {
                uri: "/data/sales.csv".to_owned(),
                message: "[E-DAT-003] parse error: unexpected token at line 2".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        // The Display must contain "Csv" not "Json" for a .csv URI.
        let display = errors[0].to_string();
        assert!(
            display.contains("Csv"),
            "ParseError for .csv URI must show Csv format; got: {display}"
        );
        assert!(
            !display.contains("(Json)"),
            "ParseError for .csv URI must NOT show Json format; got: {display}"
        );
    }

    /// `test_map_source_error_parse_error_format_inferred_yaml`
    ///
    /// FINDING-3: ParseError for a YAML URI must surface DataFormat::Yaml.
    #[test]
    fn test_map_source_error_parse_error_format_inferred_yaml() {
        let sources = single_error_source(
            "config",
            DataSourceError::ParseError {
                uri: "/etc/config.yaml".to_owned(),
                message: "[E-DAT-003] parse error: unexpected indent".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        let display = errors[0].to_string();
        assert!(
            display.contains("Yaml"),
            "ParseError for .yaml URI must show Yaml format; got: {display}"
        );
    }

    /// `test_map_source_error_parse_error_format_inferred_toml`
    ///
    /// FINDING-3: ParseError for a TOML URI must surface DataFormat::Toml.
    #[test]
    fn test_map_source_error_parse_error_format_inferred_toml() {
        let sources = single_error_source(
            "brand",
            DataSourceError::ParseError {
                uri: "/project/brand.toml".to_owned(),
                message: "[E-DAT-003] invalid TOML: expected key".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        let display = errors[0].to_string();
        assert!(
            display.contains("Toml"),
            "ParseError for .toml URI must show Toml format; got: {display}"
        );
    }

    /// `test_map_source_error_file_not_found_extracts_real_path`
    ///
    /// FINDING-4: IoError with [E-DAT-004] must extract the actual missing path from
    /// the message instead of using the URI as the path. Both the URI and the real path
    /// must be visible to the caller via the error display.
    #[test]
    fn test_map_source_error_file_not_found_extracts_real_path() {
        let sources = single_error_source(
            "data",
            DataSourceError::IoError {
                uri: "mock://missing".to_owned(),
                message: "[E-DAT-004] file not found: /real/path.json (at mock)".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code(), "E-DAT-004");
        let display = errors[0].to_string();
        assert!(
            display.contains("/real/path.json"),
            "file-not-found error must contain the actual path '/real/path.json'; got: {display}"
        );
    }

    /// `test_stub_source_id_returns_static_plugin_id`
    ///
    /// FINDING-7: Mock plugin `id()` must return a static plugin-type id,
    /// NOT the binding name. Verify the stubs in this test module comply.
    #[test]
    fn test_stub_source_id_returns_static_plugin_id() {
        let file_src = StubSource {
            result: Ok(Value::Int(0)),
        };
        let http_src = OnlineStubSource {
            result: Ok(Value::Int(0)),
        };
        // ids must be static plugin-type strings, not binding names.
        assert!(!file_src.id().is_empty(), "StubSource id must not be empty");
        assert!(
            !http_src.id().is_empty(),
            "OnlineStubSource id must not be empty"
        );
        // They must be static plugin type identifiers, not dynamic binding names.
        assert_eq!(file_src.id(), "stub-file");
        assert_eq!(http_src.id(), "stub-http");
    }
}
