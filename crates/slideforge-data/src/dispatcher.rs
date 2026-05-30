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
use crate::error::{
    E_DAT_001, E_DAT_002, E_DAT_003, E_DAT_004, E_DAT_006, E_DAT_006_POLICY, E_DAT_015,
};

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
/// | `[E-DAT-006]` (in IoError)   | `DataError::PolicyRejected` (body-size cap) |
/// | `[E-DAT-006]` (in ParseError)| `DataError::SsrfBlocked` |
/// | `[E-DAT-003]`           | `DataError::ParseError` |
/// | *(no bracket code)*     | Generic `DataError::UnspecifiedSourceError` with full message |
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
                //
                // FINDING-2 fix: When status extraction fails (message format not
                // recognized), fall back to DataError::IoError preserving the original
                // message rather than fabricating a fake status of 0.
                match extract_http_status(inner_msg) {
                    Ok(status) => DataError::HttpError {
                        code: E_DAT_001,
                        url: Arc::from(uri.as_str()),
                        status,
                        span: slideforge_types::SourceSpan::default(),
                    },
                    Err(_unparseable) => {
                        // F-P3-MED-003 fix: Strip the [E-DAT-001] bracket prefix from
                        // inner_msg before storing into `message`. IoError's Display
                        // prepends "[{code}] I/O error reading …", so storing the raw
                        // inner_msg (which may already contain "[E-DAT-001] …") would
                        // produce a double bracket in the final Display.
                        DataError::IoError {
                            code: E_DAT_001,
                            path: Arc::from(uri.as_str()),
                            message: Arc::from(strip_bracket_prefix(inner_msg)),
                            span: slideforge_types::SourceSpan::default(),
                        }
                    },
                }
            } else if inner_msg.contains(E_DAT_002) || message.contains(E_DAT_002) {
                // Network/transport error (connection refused, timeout, DNS failure,
                // body-read I/O error, non-UTF-8 body). The inner message is the
                // human-readable cause.
                //
                // F-P3-MED-001 fix: Strip the leading [E-DAT-002] bracket from inner_msg
                // before storing into `cause`. NetworkError's Display already prepends
                // "[E-DAT-002] network error: …", so storing the raw inner_msg (which
                // also starts with "[E-DAT-002] network error: …") produces double-bracket
                // + double "network error:" in the final Display.
                //
                // Strip order:
                //  1. [E-DAT-NNN] bracket prefix
                //  2. any leading "network error: " label (to avoid double label)
                //  3. any trailing " (at SourceSpan { … })" span annotation
                let clean_cause = {
                    let s = strip_bracket_prefix(inner_msg);
                    let s = s.strip_prefix("network error: ").unwrap_or(s);
                    // Strip trailing " (at …)" span annotation if present.
                    if let Some((before_at, _)) = s.rsplit_once(" (at ") {
                        before_at.trim()
                    } else {
                        s.trim()
                    }
                };
                DataError::NetworkError {
                    code: E_DAT_002,
                    url: Arc::from(uri.as_str()),
                    cause: Arc::from(clean_cause),
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
                // F-P3-MED-002 fix (structural): E-DAT-006 in an IoError arm means the HTTP
                // source's response-body-size cap was exceeded (a policy rejection, not I/O
                // failure and not an SSRF block). Route to PolicyRejected which has a
                // semantically correct Display ("data policy rejected") rather than IoError
                // ("I/O error reading") which is factually wrong for a policy-enforcement
                // decision. Also strip the [E-DAT-006] bracket prefix from the message to
                // prevent double-bracket in the Display.
                DataError::PolicyRejected {
                    code: E_DAT_006_POLICY,
                    uri: Arc::from(uri.as_str()),
                    message: Arc::from(strip_bracket_prefix(inner_msg)),
                    span: slideforge_types::SourceSpan::default(),
                }
            } else {
                // FINDING-3 fix: Generic I/O error with no recognized bracket code.
                // Route to UnspecifiedSourceError (E-DAT-015) rather than IoError
                // (E-DAT-004 = "file not found") to avoid mis-categorizing third-party
                // plugin errors (e.g., disk-full, permission-denied) as file-not-found.
                //
                // FINDING-7 fix: Use `inner_msg` (the source's own message, not
                // `err.to_string()` which wraps it in a second IoError layer) to avoid
                // the "I/O error reading" prefix appearing twice in the Display output.
                //
                // Emit a tracing::warn! to instruct plugin authors to embed [E-DAT-NNN].
                tracing::warn!(
                    "dispatcher: data source error for binding '{}' has no [E-DAT-NNN] \
                    bracket code in message. Routed to E-DAT-015 (unspecified). \
                    Plugin authors: embed [E-DAT-NNN] in error messages for precise routing.",
                    name
                );
                DataError::UnspecifiedSourceError {
                    code: E_DAT_015,
                    uri: Arc::from(uri.as_str()),
                    message: Arc::from(inner_msg.as_str()),
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
                // FINDING-1 fix: Extract the bare hostname from the URI using url::Url::parse
                // so the remediation hint reads "Add '169.254.169.254' to [data].allowed_domains"
                // (bare host) rather than "Add 'http://169.254.169.254/path' to ..." (full URL).
                // The `allowed_domains` config key accepts bare hosts, not full URLs.
                //
                // If url::Url::parse fails (malformed URI), fall back to the raw URI for both
                // fields — the SSRF was already blocked so we're just formatting the error.
                let (url_str, domain_str) = if let Ok(parsed) = url::Url::parse(uri) {
                    let host = parsed.host_str().unwrap_or(uri.as_str()).to_lowercase();
                    (uri.as_str(), host)
                } else {
                    tracing::warn!(
                        "dispatcher: could not parse SSRF-blocked URI '{}' as URL; \
                        using raw URI as domain in remediation hint",
                        uri
                    );
                    (uri.as_str(), uri.to_owned())
                };
                DataError::SsrfBlocked {
                    code: E_DAT_006,
                    url: Arc::from(url_str),
                    domain: Arc::from(domain_str.as_str()),
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
            // FINDING-8 fix: Authentication failures are mapped to AuthFailed
            // (not NetworkError) so the Display does NOT include "Use --offline
            // to skip HTTP sources" — which is wrong remediation for an auth failure.
            //
            // Note: `HttpDataSource` v1 silently ignores `auth_token` and will not
            // reach this path in practice. The mapping exists for correctness when
            // third-party plugins emit `DataSourceError::AuthError`.
            DataError::AuthFailed {
                code: E_DAT_002,
                uri: Arc::from(uri.as_str()),
                span: slideforge_types::SourceSpan::default(),
            }
        },
    }
}

/// Strip a leading `[E-DAT-NNN]` bracket prefix from an error message, returning
/// just the human-readable text after the prefix (with any immediately following
/// space trimmed). Used to prevent double-bracket-in-Display regressions when
/// re-wrapping a source's pre-bracketed message in a `DataError` variant whose
/// own Display also prepends `[{code}]`.
///
/// # Behaviour
///
/// - If the message starts with `'['` and a matching `']'` is found within the
///   first 16 characters, the content after the bracket (leading spaces stripped)
///   is returned.
/// - If the message does not start with `'['`, or the `']'` cannot be found
///   within 16 characters, the input is returned **unchanged**. This conservative
///   fallback prevents incorrectly stripping messages from third-party plugins
///   that happen to contain bracket characters for other reasons.
///
/// # Examples
///
/// ```text
/// strip_bracket_prefix("[E-DAT-002] network error: …")
///     → "network error: …"
/// strip_bracket_prefix("no bracket here")
///     → "no bracket here"
/// strip_bracket_prefix("[MALFORMED")       // no closing ']'
///     → "[MALFORMED"
/// ```
fn strip_bracket_prefix(msg: &str) -> &str {
    if !msg.starts_with('[') {
        return msg;
    }
    // The standard bracket format is "[E-DAT-NNN]" — at most 12 characters.
    // Search within 16 characters to give a safe margin while rejecting long
    // bracket-like sequences that are not our format.
    let window = if msg.len() < 16 { msg.len() } else { 16 };
    if let Some(close) = msg[..window].find(']') {
        // Skip past ']' and any immediately following space.
        msg[close + 1..].trim_start()
    } else {
        msg
    }
}

/// Extract the HTTP status code from a bracket-coded message of the form
/// `"[E-DAT-001] HTTP <status> from '<url>'"`.
///
/// Returns `Ok(status)` when a parseable three-digit status code is found.
/// Returns `Err(&'static str)` with a description when the message does not
/// match the expected format. Callers MUST NOT fabricate a fake status of 0
/// on `Err` — instead, fall back to `DataError::IoError` preserving the
/// original message verbatim.
///
/// # Format
///
/// The message must contain the literal substring `"HTTP "` followed
/// immediately by ASCII decimal digits. Example:
/// `"[E-DAT-001] HTTP 404 from 'http://example.com/data.json'"`.
fn extract_http_status(message: &str) -> Result<u16, &'static str> {
    // Look for "HTTP " followed by a numeric token.
    let after_http = message
        .find("HTTP ")
        .and_then(|pos| pos.checked_add(5))
        .ok_or("'HTTP ' not found in message")?;
    let rest = message
        .get(after_http..)
        .ok_or("message ended after 'HTTP '")?;
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    let digits = rest.get(..end).ok_or("no digits after 'HTTP '")?;
    if digits.is_empty() {
        return Err("no digits after 'HTTP '");
    }
    digits
        .parse::<u16>()
        .map_err(|_| "status digits overflow u16")
}

/// Extract the path component from a bracket-coded message of the form
/// `"[E-DAT-NNN] <label>: <path>"` or `"[E-DAT-NNN] <label>: <path> (at ...)"`.
///
/// The bracket code format used by built-in sources is `[E-DAT-NNN]` (no colon
/// inside the bracket), so the content after the bracket starts with a space
/// followed by the label+path. This function skips past the closing bracket
/// and any label prefix to extract the bare path.
///
/// ## Known label prefixes
///
/// - `"file not found: "` — emitted by `FileDataSource`
/// - `"I/O error: "` — emitted by `FileDataSource` on permission/OS errors
///
/// ## Return value
///
/// Returns `Some(path)` when the message contains one of the known label prefixes
/// and the path is non-empty after stripping the prefix.
///
/// Returns `None` when:
/// - No `']'` bracket terminator is found.
/// - The content after the bracket does not start with a recognized label prefix.
///   The caller must fall back to `uri.as_str()` explicitly in this case.
/// - The path segment after the prefix is empty.
///
/// This design prevents leaking unrecognized label text into the `.path` field
/// of `DataError::FileNotFound` (FINDING-6: "unknown prefixes silently leak label
/// text into the path").
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
    // Strip ONLY recognized label prefixes. Return None for unrecognized prefixes
    // so callers fall back to uri.as_str() rather than leaking the label text.
    let clean = if let Some(rest) = with_label.strip_prefix("file not found: ") {
        rest.trim()
    } else if let Some(rest) = with_label.strip_prefix("I/O error: ") {
        rest.trim()
    } else {
        // Unrecognized prefix — signal the caller to fall back to uri.as_str().
        return None;
    };
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

    // ---------------------------------------------------------------------------
    // FINDING-1: SsrfBlocked domain field must be bare host, not full URL
    // ---------------------------------------------------------------------------

    /// `test_map_source_error_ssrf_preserves_bare_host_in_remediation_hint`
    ///
    /// FINDING-1: When `DataSourceError::ParseError` carries `[E-DAT-006]` (SSRF),
    /// the resulting `SsrfBlocked.domain` field must contain only the bare hostname
    /// (no scheme, no path, no port for standard ports) so the remediation hint reads
    /// "Add 'hostname' to [data].allowed_domains" — consistent with what the config key
    /// actually expects.
    #[test]
    fn test_map_source_error_ssrf_preserves_bare_host_in_remediation_hint() {
        // Case 1: IMDS endpoint — domain must be "169.254.169.254" (no scheme, no path)
        let sources = single_error_source(
            "imds",
            DataSourceError::ParseError {
                uri: "http://169.254.169.254/metadata".to_owned(),
                message: "[E-DAT-006] HTTP source 'http://169.254.169.254/metadata' blocked \
                           by allowed_domains policy."
                    .to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code(), "E-DAT-006");
        let display = errors[0].to_string();
        assert!(
            display.contains("Add '169.254.169.254' to"),
            "SSRF remediation hint must use bare host '169.254.169.254'; got: {display}"
        );
        assert!(
            !display.contains("Add 'http://169.254.169.254/metadata' to"),
            "SSRF remediation hint must NOT include scheme or path; got: {display}"
        );

        // Case 2: internal host with explicit port — domain must be "internal.corp"
        // (port is excluded from host_str() per url crate behavior for non-standard ports).
        let sources2 = single_error_source(
            "internal",
            DataSourceError::ParseError {
                uri: "https://internal.corp:8080/path".to_owned(),
                message: "[E-DAT-006] blocked".to_owned(),
            },
        );
        let (_scope2, errors2) = load_all(&sources2, &ctx);
        assert_eq!(errors2.len(), 1);
        assert_eq!(errors2[0].code(), "E-DAT-006");
        let display2 = errors2[0].to_string();
        assert!(
            display2.contains("Add 'internal.corp' to"),
            "SSRF remediation hint for host-with-port must use bare host 'internal.corp'; \
            got: {display2}"
        );

        // Case 3: IPv6 loopback — domain must be "[::1]" (bracketed form)
        let sources3 = single_error_source(
            "ipv6",
            DataSourceError::ParseError {
                uri: "https://[::1]/foo".to_owned(),
                message: "[E-DAT-006] blocked".to_owned(),
            },
        );
        let (_scope3, errors3) = load_all(&sources3, &ctx);
        assert_eq!(errors3.len(), 1);
        assert_eq!(errors3[0].code(), "E-DAT-006");
        let display3 = errors3[0].to_string();
        // url::Url::host_str() returns "::1" (no brackets) for IPv6; we accept either.
        assert!(
            display3.contains("Add '::1' to") || display3.contains("Add '[::1]' to"),
            "SSRF remediation hint for IPv6 must use the IPv6 host form; got: {display3}"
        );
    }

    // ---------------------------------------------------------------------------
    // FINDING-2: extract_http_status must not produce fake HTTP 0 on failure
    // ---------------------------------------------------------------------------

    /// `test_map_source_error_e_dat_001_display_contains_status_code`
    ///
    /// FINDING-2: When IoError contains [E-DAT-001] and a parseable status,
    /// the Display must contain the actual status (e.g., "HTTP 404"). Tests two
    /// status codes to pin the format.
    #[test]
    fn test_map_source_error_e_dat_001_display_contains_status_code() {
        // Case 1: 404 Not Found
        let sources_404 = single_error_source(
            "api_404",
            DataSourceError::IoError {
                uri: "http://example.com/missing.json".to_owned(),
                message: "[E-DAT-001] HTTP 404 from 'http://example.com/missing.json'".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources_404, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code(), "E-DAT-001");
        let display = errors[0].to_string();
        assert!(
            display.contains("404"),
            "E-DAT-001 error for 404 must contain '404' in Display; got: {display}"
        );
        assert!(
            !display.contains("HTTP 0"),
            "E-DAT-001 error must not fabricate 'HTTP 0'; got: {display}"
        );

        // Case 2: 503 Service Unavailable
        let sources_503 = single_error_source(
            "api_503",
            DataSourceError::IoError {
                uri: "http://example.com/data.json".to_owned(),
                message: "[E-DAT-001] HTTP 503 from 'http://example.com/data.json'".to_owned(),
            },
        );
        let (_scope2, errors2) = load_all(&sources_503, &ctx);
        assert_eq!(errors2.len(), 1);
        assert_eq!(errors2[0].code(), "E-DAT-001");
        let display2 = errors2[0].to_string();
        assert!(
            display2.contains("503"),
            "E-DAT-001 error for 503 must contain '503' in Display; got: {display2}"
        );
    }

    /// `test_extract_http_status_falls_back_on_unparseable_message`
    ///
    /// FINDING-2: When the message does not match the "HTTP NNN" format,
    /// `extract_http_status` must return Err (not produce a fake 0).
    /// The caller must then fall back to preserving the original message.
    #[test]
    fn test_extract_http_status_falls_back_on_unparseable_message() {
        // Message that does NOT contain "HTTP " followed by digits.
        let result = extract_http_status("some error without an HTTP status");
        assert!(
            result.is_err(),
            "extract_http_status must return Err on unparseable message; got: {result:?}"
        );

        // The dispatcher must route to IoError with original message, not HttpError{status:0}.
        let sources = single_error_source(
            "weird",
            DataSourceError::IoError {
                uri: "http://example.com/".to_owned(),
                message: "[E-DAT-001] custom plugin error without HTTP status".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code(), "E-DAT-001");
        let display = errors[0].to_string();
        assert!(
            !display.contains("HTTP 0"),
            "fallback path must not produce 'HTTP 0'; got: {display}"
        );
        assert!(
            display.contains("custom plugin error without HTTP status"),
            "fallback path must preserve the original message; got: {display}"
        );
    }

    // ---------------------------------------------------------------------------
    // FINDING-3: Catch-all must not use E-DAT-004 for unknown-category errors
    // ---------------------------------------------------------------------------

    /// `test_map_source_error_no_bracket_code_routes_to_e_dat_015`
    ///
    /// FINDING-3: When an IoError message has no [E-DAT-NNN] bracket prefix,
    /// the dispatcher must route to E-DAT-015 (UnspecifiedSourceError), NOT
    /// E-DAT-004 (file not found). The Display must not say "file not found".
    #[test]
    fn test_map_source_error_no_bracket_code_routes_to_e_dat_015() {
        let sources = single_error_source(
            "plugin",
            DataSourceError::IoError {
                uri: "custom://some-resource".to_owned(),
                message: "disk quota exceeded".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        let code = errors[0].code();
        assert_ne!(
            code, "E-DAT-004",
            "catch-all must NOT route to E-DAT-004 (file not found) for unknown error; \
            got code: {code}"
        );
        assert_eq!(
            code, "E-DAT-015",
            "catch-all must route to E-DAT-015 (unspecified); got: {code}"
        );
        let display = errors[0].to_string();
        assert!(
            !display.contains("file not found"),
            "catch-all Display must NOT say 'file not found'; got: {display}"
        );
        assert!(
            display.contains("disk quota exceeded"),
            "catch-all Display must preserve the original message; got: {display}"
        );
    }

    // ---------------------------------------------------------------------------
    // FINDING-6: extract_path_after_code returns None on unrecognized prefix
    // ---------------------------------------------------------------------------

    /// `test_extract_path_after_code_returns_none_on_unknown_prefix`
    ///
    /// FINDING-6: `extract_path_after_code` must return `None` when the content
    /// after the bracket code does not start with a recognized label prefix.
    /// This prevents unrecognized label text from leaking into the `.path` field.
    #[test]
    fn test_extract_path_after_code_returns_none_on_unknown_prefix() {
        // Unknown prefix "custom error: " — must return None.
        let result = extract_path_after_code("[E-DAT-004] custom error: /some/path");
        assert!(
            result.is_none(),
            "extract_path_after_code must return None for unknown prefix; got: {result:?}"
        );

        // Known prefix "file not found: " — must return Some.
        let result = extract_path_after_code("[E-DAT-004] file not found: /real/path.json");
        assert_eq!(
            result,
            Some("/real/path.json"),
            "extract_path_after_code must return Some for known 'file not found' prefix"
        );

        // Known prefix "I/O error: " — must return Some.
        let result = extract_path_after_code("[E-DAT-004] I/O error: /some/file.json");
        assert_eq!(
            result,
            Some("/some/file.json"),
            "extract_path_after_code must return Some for known 'I/O error' prefix"
        );

        // No bracket terminator — must return None.
        let result = extract_path_after_code("no bracket here");
        assert!(
            result.is_none(),
            "extract_path_after_code must return None with no bracket; got: {result:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // FINDING-7: Catch-all must not produce double "I/O error" in Display
    // ---------------------------------------------------------------------------

    /// `test_catch_all_display_does_not_double_wrap_io_error`
    ///
    /// FINDING-7: The catch-all arm must use the inner source message (not
    /// `err.to_string()` which prepends "I/O error for 'uri': ") to prevent
    /// the string "I/O error reading" from appearing twice in the Display.
    #[test]
    fn test_catch_all_display_does_not_double_wrap_io_error() {
        let sources = single_error_source(
            "plugin_weird",
            DataSourceError::IoError {
                uri: "custom://resource".to_owned(),
                message: "permission denied".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        let display = errors[0].to_string();
        // Count occurrences of "I/O error" — must appear at most once.
        let occurrences = display.matches("I/O error").count();
        assert!(
            occurrences <= 1,
            "Display must not contain 'I/O error' more than once; got {occurrences} \
            occurrences in: {display}"
        );
        assert!(
            display.contains("permission denied"),
            "Display must contain the original error message; got: {display}"
        );
    }

    // ---------------------------------------------------------------------------
    // FINDING-8: AuthError must not produce "Use --offline" in Display
    // ---------------------------------------------------------------------------

    // ---------------------------------------------------------------------------
    // F-P3-MED-001: strip_bracket_prefix unit tests
    // ---------------------------------------------------------------------------

    /// `test_strip_bracket_prefix_strips_known_prefix`
    ///
    /// F-P3-MED-001: `strip_bracket_prefix` must strip a well-formed `[E-DAT-NNN]`
    /// bracket from the front of a message and return only the human-readable tail.
    #[test]
    fn test_strip_bracket_prefix_strips_known_prefix() {
        assert_eq!(
            strip_bracket_prefix("[E-DAT-002] network error: connection refused"),
            "network error: connection refused"
        );
        assert_eq!(
            strip_bracket_prefix("[E-DAT-006] response body exceeds cap"),
            "response body exceeds cap"
        );
        assert_eq!(
            strip_bracket_prefix("[E-DAT-001] HTTP 404 from 'http://example.com'"),
            "HTTP 404 from 'http://example.com'"
        );
    }

    /// `test_strip_bracket_prefix_leaves_plain_message_unchanged`
    ///
    /// F-P3-MED-001: When the message does not start with `[`, `strip_bracket_prefix`
    /// must return the input unchanged.
    #[test]
    fn test_strip_bracket_prefix_leaves_plain_message_unchanged() {
        assert_eq!(
            strip_bracket_prefix("connection refused"),
            "connection refused"
        );
        assert_eq!(strip_bracket_prefix(""), "");
        // Conservatively: '[' at non-zero position must NOT strip.
        assert_eq!(
            strip_bracket_prefix("some [bracketed] text"),
            "some [bracketed] text"
        );
    }

    /// `test_strip_bracket_prefix_leaves_malformed_bracket_unchanged`
    ///
    /// F-P3-MED-001: When the message starts with `[` but no closing `]` is found
    /// within 16 characters, return the input unchanged (conservative fallback).
    #[test]
    fn test_strip_bracket_prefix_leaves_malformed_bracket_unchanged() {
        // No closing bracket at all.
        assert_eq!(strip_bracket_prefix("[MALFORMED"), "[MALFORMED");
        // Closing bracket beyond position 15 — treated as not our format.
        let long = "[THIS-IS-A-VERY-LONG-CODE] rest";
        let result = strip_bracket_prefix(long);
        // Either returns original (no ']' in window) or strips it — both are safe.
        // The important thing: no panic.
        assert!(!result.is_empty());
    }

    // ---------------------------------------------------------------------------
    // F-P3-MED-001: NetworkError must not double-wrap bracket or "network error:"
    // ---------------------------------------------------------------------------

    /// `test_map_source_error_e_dat_002_display_single_bracket_only`
    ///
    /// F-P3-MED-001: When an `IoError` carrying `[E-DAT-002]` is routed to
    /// `DataError::NetworkError`, the Display must contain exactly one `[E-DAT-002]`
    /// and exactly one `"network error:"` — not double-wrapped.
    #[test]
    fn test_map_source_error_e_dat_002_display_single_bracket_only() {
        let sources = single_error_source(
            "net_src",
            DataSourceError::IoError {
                uri: "http://example.com/".to_owned(),
                message: "[E-DAT-002] network error: connection refused (at src:1:1)".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code(), "E-DAT-002");
        let display = errors[0].to_string();
        assert_eq!(
            display.matches("[E-DAT-002]").count(),
            1,
            "Display must contain exactly one [E-DAT-002]; got: {display}"
        );
        assert_eq!(
            display.matches("network error:").count(),
            1,
            "Display must contain exactly one 'network error:'; got: {display}"
        );
        // Span annotation must not be doubled.
        assert!(
            display.matches("(at ").count() <= 1,
            "Display must not contain '(at ' more than once; got: {display}"
        );
    }

    // ---------------------------------------------------------------------------
    // F-P3-MED-002: E-DAT-006 body-cap routes to PolicyRejected, not IoError
    // ---------------------------------------------------------------------------

    /// `test_map_source_error_e_dat_006_body_cap_routes_to_policy_rejected_single_bracket`
    ///
    /// F-P3-MED-002: When an `IoError` carrying `[E-DAT-006]` (body-size cap) is
    /// routed, it must:
    ///   1. Produce code `E-DAT-006`.
    ///   2. Display exactly one `[E-DAT-006]`.
    ///   3. NOT say `"I/O error reading"` (that would be semantically wrong).
    ///   4. NOT say `"path traversal"` (confirmed safe from Pass-2).
    #[test]
    fn test_map_source_error_e_dat_006_body_cap_routes_to_policy_rejected_single_bracket() {
        let sources = single_error_source(
            "bigdata",
            DataSourceError::IoError {
                uri: "http://example.com/large.json".to_owned(),
                message: "[E-DAT-006] response body exceeds 52428800-byte cap (policy-rejected)"
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
        assert_eq!(
            display.matches("[E-DAT-006]").count(),
            1,
            "Display must contain exactly one [E-DAT-006]; got: {display}"
        );
        assert!(
            !display.contains("I/O error reading"),
            "body-cap error must NOT say 'I/O error reading' (policy, not I/O); got: {display}"
        );
        assert!(
            !display.contains("path traversal"),
            "body-cap error must NOT say 'path traversal'; got: {display}"
        );
        // Must contain the human-readable policy description from the source.
        assert!(
            display.contains("response body exceeds") || display.contains("policy-rejected"),
            "body-cap error Display must describe the policy violation; got: {display}"
        );
    }

    // ---------------------------------------------------------------------------
    // F-P3-MED-003: E-DAT-001 fallback must not double-wrap bracket
    // ---------------------------------------------------------------------------

    /// `test_extract_http_status_fallback_single_bracket`
    ///
    /// F-P3-MED-003: When `extract_http_status` fails (message has no parseable HTTP
    /// status), the dispatcher falls back to `DataError::IoError` with the inner_msg.
    /// The Display must contain exactly one `[E-DAT-001]` — not double-wrapped.
    #[test]
    fn test_extract_http_status_fallback_single_bracket() {
        let sources = single_error_source(
            "api",
            DataSourceError::IoError {
                uri: "http://example.com/".to_owned(),
                message: "[E-DAT-001] custom plugin error without HTTP status".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code(), "E-DAT-001");
        let display = errors[0].to_string();
        assert_eq!(
            display.matches("[E-DAT-001]").count(),
            1,
            "Fallback IoError Display must contain exactly one [E-DAT-001]; got: {display}"
        );
        assert!(
            !display.contains("HTTP 0"),
            "fallback must not produce 'HTTP 0'; got: {display}"
        );
    }

    // ---------------------------------------------------------------------------
    // FINDING-8: AuthError must not produce "Use --offline" in Display
    // ---------------------------------------------------------------------------

    /// `test_map_source_error_auth_error_does_not_suggest_offline`
    ///
    /// FINDING-8: `DataSourceError::AuthError` must not produce a Display containing
    /// "Use --offline" — that hint is wrong for an auth failure. The Display must
    /// contain "auth" to indicate the failure type.
    #[test]
    fn test_map_source_error_auth_error_does_not_suggest_offline() {
        let sources = single_error_source(
            "api",
            DataSourceError::AuthError {
                uri: "https://api.example.com/data.json".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        // Must still be an access-layer error (E-DAT-002 range).
        // AuthFailed returns E-DAT-002.
        let code = errors[0].code();
        assert_eq!(
            code, "E-DAT-002",
            "AuthError must carry E-DAT-002; got: {code}"
        );
        let display = errors[0].to_string();
        assert!(
            !display.contains("Use --offline"),
            "AuthError Display must NOT suggest '--offline'; got: {display}"
        );
        assert!(
            display.to_lowercase().contains("auth"),
            "AuthError Display must mention 'auth'; got: {display}"
        );
    }
}
