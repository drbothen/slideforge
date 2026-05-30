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
    E_DAT_001, E_DAT_002, E_DAT_003, E_DAT_004, E_DAT_006, E_DAT_007, E_DAT_008, E_DAT_009,
    E_DAT_010, E_DAT_011, E_DAT_012, E_DAT_013, E_DAT_014, E_DAT_015,
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
/// | Bracket code + label in message                   | Routed to variant |
/// |---------------------------------------------------|-------------------|
/// | `[E-DAT-001]` + HTTP status                       | `DataError::HttpError` |
/// | `[E-DAT-001]` + no parseable status               | `DataError::IoError` (code `E-DAT-001`) |
/// | `[E-DAT-002]`                                     | `DataError::NetworkError` |
/// | `[E-DAT-004]` + `"file not found: "` label        | `DataError::FileNotFound` |
/// | `[E-DAT-004]` + any other label (I/O error, etc.) | `DataError::IoError` |
/// | `[E-DAT-006]` (in IoError) + `"path traversal blocked: '"` | `DataError::PathTraversalBlocked` |
/// | `[E-DAT-006]` (in IoError) + any other label      | `DataError::PolicyRejected` (body-size cap) |
/// | `[E-DAT-006]` (in ParseError)                     | `DataError::SsrfBlocked` |
/// | `[E-DAT-003]`                                     | `DataError::ParseError` |
/// | *(no bracket code)*                               | `DataError::UnspecifiedSourceError` |
// This function is a routing table with one arm per DataSourceError variant and one
// sub-arm per bracket code. Splitting it into smaller functions would require passing
// the same arguments through multiple layers of helper calls without improving
// readability. The line count is marginally over the lint threshold; suppress here.
#[allow(clippy::too_many_lines)]
#[must_use]
fn map_source_error(name: &Arc<str>, err: &DataSourceError) -> DataError {
    match err {
        DataSourceError::IoError {
            uri,
            message: inner_msg,
        } => {
            // Route by bracket code embedded in the message.
            // Priority order: most-specific codes first.
            //
            // F-P5-LOW-004: Use anchored bracket-prefix checks (`starts_with("[E-DAT-NNN]")`)
            // rather than unanchored `contains("E-DAT-NNN")` substring matches. The unanchored
            // form would misroute a message like `"[E-DAT-006] policy rejected; see also E-DAT-001
            // in docs"` to the E-DAT-001 arm instead of the E-DAT-006 arm.
            //
            // Helper: returns true when `msg` starts with the `[E-DAT-NNN]` bracket for `code`.
            fn has_bracket(msg: &str, code: &str) -> bool {
                msg.starts_with(&format!("[{code}]"))
            }
            // F-P6-MED-001 fix: removed `|| has_bracket(message.as_ref(), E_DAT_NNN)` dead code
            // from all IoError sub-arms. DataSourceError::IoError Display always starts with
            // "I/O error for '…'" — never with "[E-DAT-NNN]" — so the second clause could
            // never be true. Only `inner_msg` carries the bracket code from built-in sources.
            if has_bracket(inner_msg, E_DAT_001) {
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
                    Err(reason) => {
                        // F-P9-LOW-002 fix: log the unparseable reason for debuggability
                        // instead of silently discarding it. The previous `Err(_unparseable)`
                        // binding dropped the description string, making future diagnosis harder.
                        tracing::debug!(
                            "dispatcher: extract_http_status fell back to IoError: \
                            {reason}; message: {inner_msg}"
                        );
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
            } else if has_bracket(inner_msg, E_DAT_002) {
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
            } else if has_bracket(inner_msg, E_DAT_004) {
                // Route to FileNotFound OR IoError depending on the label in the message.
                //
                // F-P4-HIGH-001 fix: `[E-DAT-004]` is shared by two `DataError` variants:
                //   - `FileNotFound` ("file not found: <path>") — file absent
                //   - `IoError` ("I/O error reading '<path>': <msg>") — OS error (permission, etc.)
                //
                // The dispatcher must inspect the label AFTER the bracket code to choose
                // the correct variant. `message_is_file_not_found` returns true only for
                // the "file not found:" label; all other labels (I/O error reading, failed
                // to open file, etc.) map to `DataError::IoError`.
                //
                // F-P7-MED-001 fix: the enclosing `else if has_bracket(inner_msg, E_DAT_004)`
                // already guarantees the bracket is present — the inner re-test was dead code.
                // Replaced with a direct reference to `inner_msg.as_str()`.
                let source_msg: &str = inner_msg.as_str();
                if message_is_file_not_found(source_msg) {
                    // FINDING-4 fix: extract the actual missing path from the bracket-coded
                    // message tail (after "[E-DAT-004] file not found: <path>") so users
                    // see the real path, not just the URI.
                    let actual_path = extract_path_after_code(source_msg).unwrap_or(uri.as_str());
                    DataError::FileNotFound {
                        code: E_DAT_004,
                        path: Arc::from(actual_path),
                        span: slideforge_types::SourceSpan::default(),
                    }
                } else {
                    // I/O error (e.g., permission denied, header read failure).
                    // Extract path from the message; fall back to the URI.
                    let path = extract_path_after_code(source_msg).unwrap_or(uri.as_str());
                    // Strip the bracket code + label from the message to avoid double-prefix
                    // when stored into IoError (whose Display prepends "[E-DAT-004] I/O error
                    // reading…").
                    let clean_msg = strip_bracket_prefix(source_msg);
                    // Also strip any canonical label prefixes so the stored message is the
                    // bare OS reason string (e.g., "permission denied").
                    let bare_msg = extract_bare_io_reason(clean_msg);
                    DataError::IoError {
                        code: E_DAT_004,
                        path: Arc::from(path),
                        message: Arc::from(bare_msg),
                        span: slideforge_types::SourceSpan::default(),
                    }
                }
            } else if has_bracket(inner_msg, E_DAT_006) {
                // F-P3-MED-002 fix (structural): E-DAT-006 in an IoError arm covers two
                // distinct security sub-cases that share the same bracket code:
                //
                //   1. Path traversal blocked — `FileDataSource::load_path` returns
                //      `DataError::PathTraversalBlocked` which is translated via
                //      `err.to_string()` into a message containing the literal substring
                //      "path traversal blocked: '". Must route to PathTraversalBlocked.
                //
                //   2. Body-size cap (or other HTTP policy rejection) — the HTTP source
                //      emits "[E-DAT-006] response body exceeds … cap". Must route to
                //      PolicyRejected, which has a semantically correct Display
                //      ("data policy rejected") rather than IoError ("I/O error reading").
                //
                // F-P7-HIGH-001 fix: inspect the message BEFORE routing to PolicyRejected.
                // Without this check, path-traversal events were silently recharacterized
                // as policy rejections — misleading the user and losing the PathTraversalBlocked
                // variant identity (so callers matching on PathTraversalBlocked never saw it).
                if message_is_path_traversal(inner_msg) {
                    // Extract the traversal-attempt path from the message.
                    // Format emitted by DataError::PathTraversalBlocked Display:
                    // "[E-DAT-006] path traversal blocked: '<path>' is outside base dir (at ...)"
                    let path = extract_path_from_traversal_msg(inner_msg).unwrap_or(uri.as_str());
                    DataError::PathTraversalBlocked {
                        code: E_DAT_006,
                        path: Arc::from(path),
                        span: slideforge_types::SourceSpan::default(),
                    }
                } else {
                    // Body-cap policy rejection (vs. SSRF — both share the code but route
                    // to different DataError variants: SsrfBlocked for SSRF domain blocks,
                    // PolicyRejected for body-size cap and similar policy rejections).
                    // Strip the [E-DAT-006] bracket prefix from the message to prevent
                    // double-bracket in the Display.
                    DataError::PolicyRejected {
                        code: E_DAT_006,
                        uri: Arc::from(uri.as_str()),
                        message: Arc::from(strip_bracket_prefix(inner_msg)),
                        span: slideforge_types::SourceSpan::default(),
                    }
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
                // F-P4-LOW-004 fix: emit at `debug!` (not `warn!`) so watch-mode doesn't
                // flood logs when a third-party plugin doesn't embed [E-DAT-NNN].
                // This is developer-guidance, not an operational alert. Plugin authors who
                // consult the dispatcher docs or trace output will see it at debug level.
                tracing::debug!(
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
            // F-P5-LOW-004: anchored bracket check — same discipline as IoError arm.
            // F-P6-MED-001: removed dead-code `|| message.starts_with(...)` clause;
            // DataSourceError::ParseError Display starts with "data parse error for '...'"
            // (never with "[E-DAT-006]"), so only `inner_msg` carries the bracket code.
            if inner_msg.starts_with(&format!("[{E_DAT_006}]")) {
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
                //
                // F-P6-HIGH-002 fix: `inner_msg` is the source's raw message (e.g.,
                // "[E-DAT-003] unexpected token at line 2"). `message` = err.to_string()
                // wraps it in DataSourceError::ParseError's Display:
                // "data parse error for 'sales.csv': [E-DAT-003] unexpected token at line 2".
                // If we store `message` directly into `reason`, DataError::ParseError's Display
                // prepends "[E-DAT-003] parse error for 'sales.csv' (Csv): " — producing
                // double brackets and double "parse error for". Fix: use `inner_msg` directly
                // and strip the bracket prefix, matching the pattern applied in the IoError arm.
                // F-P10-HIGH-001 fix: Preserve granular bracket codes (E-DAT-007..E-DAT-014)
                // embedded in the source message instead of hardcoding E_DAT_003. The built-in
                // XLSX source emits "[E-DAT-007] xlsx empty header" and the SQLite source emits
                // "[E-DAT-013] sqlite bad header magic" — these were previously downgraded to
                // E-DAT-003 by the hardcoded fallback, preventing callers from distinguishing
                // specific parse failures from generic parse errors.
                //
                // Rationale: `parse_e_dat_code` inspects the leading bracket of `inner_msg`
                // and returns the matching constant when recognized; falls back to E_DAT_003
                // for messages without a known bracket prefix (third-party plugins, generic
                // parse errors).
                let preserved_code = parse_e_dat_code(inner_msg).unwrap_or(E_DAT_003);
                let clean_reason = strip_bracket_prefix(inner_msg).trim();
                // Also strip any leading "data parse error for '<uri>': " label in case a
                // source has already wrapped its message through DataSourceError::ParseError.
                let clean_reason = if let Some(after_label) = clean_reason
                    .strip_prefix("data parse error for '")
                    .and_then(|s| s.split_once("': ").map(|(_, after)| after))
                {
                    after_label
                } else {
                    clean_reason
                };
                // F-P9-MED-002 fix: Strip any trailing " (at ...)" span annotation from
                // the reason string. Without this step, a plugin emitting
                // "[E-DAT-003] unexpected token (at line 2:5)" would store "unexpected token
                // (at line 2:5)" as the reason, and DataError::ParseError's Display would
                // prepend its own "(at <span>)" producing a double-annotation.
                // Mirrors the identical strip applied in the NetworkError arm.
                let clean_reason = if let Some((before_at, _)) = clean_reason.rsplit_once(" (at ") {
                    before_at.trim()
                } else {
                    clean_reason
                };
                // F-P10-LOW-001 fix: Use DataFormat::Unknown for URIs without a recognizable
                // extension (e.g., HTTP URIs without a file extension) so the format annotation
                // in the Display reads "(Unknown)" rather than falsely reporting "(Json)".
                // DataFormat::Unknown was added to explicitly represent the "no extension /
                // unrecognized extension" case in error messages.
                let format = crate::format::DataFormat::from_path(std::path::Path::new(uri))
                    .unwrap_or(crate::format::DataFormat::Unknown);
                DataError::ParseError {
                    code: preserved_code,
                    path: Arc::from(uri.as_str()),
                    format,
                    reason: Arc::from(clean_reason),
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

        // F-P5-MED-004: `DataSourceError` is `#[non_exhaustive]`. This wildcard arm
        // is required to remain forward-compatible when new variants are added in
        // future releases. Unknown variants are mapped to `DataError::UnspecifiedSourceError`
        // (E-DAT-015) — the same conservative fallback used for third-party IoError
        // messages that lack a bracket code. The full `err.to_string()` is preserved
        // so no information is lost.
        _ => {
            tracing::warn!(
                "dispatcher: unrecognized DataSourceError variant for binding '{}'. \
                DataSourceError is #[non_exhaustive]; update map_source_error when a \
                new variant is added. Falling back to E-DAT-015.",
                name
            );
            DataError::UnspecifiedSourceError {
                code: E_DAT_015,
                uri: Arc::from(""),
                message: Arc::from(err.to_string().as_str()),
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
/// - If the message starts with `'['` and a matching `']'` is found at a
///   character position within the first 32 bytes (UTF-8 safe via
///   `char_indices`), the content after the bracket (leading spaces stripped)
///   is returned.
/// - If the message does not start with `'['`, or the `']'` cannot be found
///   within the first 32 bytes, the input is returned **unchanged**. This
///   conservative fallback prevents incorrectly stripping messages from
///   third-party plugins that happen to contain bracket characters for other
///   reasons.
///
/// # UTF-8 safety
///
/// The position limit is enforced via `char_indices()` iteration, which always
/// lands on valid UTF-8 character boundaries. Direct byte-slice indexing
/// (`msg[..N]`) is intentionally avoided here because a plugin may emit a
/// message whose Nth byte falls inside a multi-byte character (e.g., UTF-8
/// sequences for non-ASCII characters) — byte-slicing at such a position would
/// panic with "byte index N is not a char boundary". Traces to F-P9-HIGH-001.
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
    // The standard bracket format is "[E-DAT-NNN]" — at most 12 characters.
    // Use char_indices() to find ']' within the first 32 bytes so we never
    // slice at a non-character boundary (UTF-8 safety, F-P9-HIGH-001 fix).
    // A bracket code like "[E-DAT-015]" is 12 chars; 32 bytes is a generous
    // upper bound that rejects long non-code bracket sequences without risk
    // of splitting a multi-byte character.
    const BRACKET_BYTE_LIMIT: usize = 32;
    if !msg.starts_with('[') {
        return msg;
    }
    let close = msg
        .char_indices()
        .take_while(|(byte_pos, _)| *byte_pos < BRACKET_BYTE_LIMIT)
        .find(|(_, ch)| *ch == ']')
        .map(|(byte_pos, _)| byte_pos);
    if let Some(close_pos) = close {
        // Skip past ']' (1 byte — ']' is ASCII) and any immediately following space.
        let stripped = msg[close_pos + 1..].trim_start();
        // F-P4-LOW-002 fix: if stripping leaves an empty string (e.g., input "[E-DAT-006]"
        // with no body), fall back to the original input to avoid emitting an empty message
        // that would produce double-space in Display format.
        if stripped.is_empty() { msg } else { stripped }
    } else {
        msg
    }
}

/// Inspect the leading `[E-DAT-NNN]` bracket prefix of a message and return the
/// matching error code constant, or `None` when the bracket does not match any
/// of the `E_DAT_001` through `E_DAT_015` constants defined in [`crate::error`].
///
/// ## Purpose (F-P10-HIGH-001)
///
/// The `ParseError` generic-fallback branch in `map_source_error` previously
/// hardcoded `E_DAT_003` regardless of the bracket code embedded in the
/// source message. This caused granular codes such as `E-DAT-007` (XLSX empty
/// header) and `E-DAT-013` (`SQLite` bad magic) to be silently downgraded to the
/// generic `E-DAT-003`.
///
/// Callers use this helper as:
/// ```text
/// parse_e_dat_code(inner_msg).unwrap_or(E_DAT_003)
/// ```
/// to preserve the granular code when it is recognized, and fall back to the
/// generic parse-error code only for messages that lack a known bracket prefix.
///
/// ## Recognized codes
///
/// All `E_DAT_001` through `E_DAT_015` constants from [`crate::error`] are recognized.
/// Unknown bracket formats (e.g., `[E-DAT-099]` or plugin-specific codes) return `None`.
///
/// ## Relationship to `strip_bracket_prefix`
///
/// This function inspects the bracket but does NOT strip it — it only returns the
/// matching constant. The caller strips the bracket separately using
/// [`strip_bracket_prefix`] when constructing the stored `reason` field.
fn parse_e_dat_code(msg: &str) -> Option<&'static str> {
    // Fast path: if the message does not start with '[', there is no bracket code.
    if !msg.starts_with('[') {
        return None;
    }
    // Recognize all defined E_DAT_NNN constants via anchored starts_with checks.
    // Order: most-specific codes first (E_DAT_007..E_DAT_015 before the generic ones)
    // to avoid a shorter prefix matching where a longer one applies. In practice all
    // codes have the same prefix length ("E-DAT-0NN") so order is unambiguous, but
    // longest-first is a good defensive practice.
    //
    // Each check uses format!("[{code}]") to guarantee the bracket wraps the code,
    // matching the canonical `[E-DAT-NNN]` format emitted by all built-in sources.
    let candidates: &[&'static str] = &[
        E_DAT_001, E_DAT_002, E_DAT_003, E_DAT_004, E_DAT_006, E_DAT_007, E_DAT_008, E_DAT_009,
        E_DAT_010, E_DAT_011, E_DAT_012, E_DAT_013, E_DAT_014, E_DAT_015,
    ];
    for &code in candidates {
        if msg.starts_with(&format!("[{code}]")) {
            return Some(code);
        }
    }
    None
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

/// Extract the bare OS reason string from a bracket-stripped I/O error message.
///
/// Built-in sources format I/O error messages as:
/// - `"I/O error reading '<path>': <reason> (at ...)"`
/// - `"I/O error: <reason>"` — legacy label (no path component; full segment after
///   the prefix is treated as the reason string)
/// - `"failed to open file '<path>': <reason> (at ...)"`
/// - `"failed to read file header '<path>': <reason> (at ...)"`
/// - `"failed to read file header from '<path>': <reason> (at ...)"`
///
/// This function strips the label + path prefix and the trailing span annotation
/// to return just the OS reason string (e.g., `"permission denied"`).
///
/// When no recognized prefix is found, returns the input trimmed.
fn extract_bare_io_reason(clean_msg: &str) -> &str {
    /// Strip a quoted-path prefix of the form `"<label>'<path>': "` from `s`
    /// and return everything after the `": "` separator.
    ///
    /// Uses `rsplit_once("': ")` (last match) rather than `split_once` (first match)
    /// so that paths containing an apostrophe (e.g., `/tmp/Bob's_data.csv`) are handled
    /// correctly. The canonical `"': "` separator always appears AFTER the path, so
    /// anchoring on the last occurrence is structurally correct.
    ///
    /// F-P10-MED-002 fix: sibling of `extract_path_after_code` which received an
    /// identical `rsplit_once` fix in F-P9-MED-001. The two helpers must agree.
    fn strip_quoted_label<'a>(s: &'a str, label: &str) -> Option<&'a str> {
        s.strip_prefix(label)
            .and_then(|rest| rest.rsplit_once("': ").map(|(_, after)| after))
    }
    // Legacy label: "I/O error: <reason>" — no path component; the entire segment
    // after the prefix is treated as the reason string.
    fn strip_unquoted_label<'a>(s: &'a str, label: &str) -> Option<&'a str> {
        s.strip_prefix(label)
    }
    let after_label = strip_quoted_label(clean_msg, "I/O error reading '")
        .or_else(|| strip_quoted_label(clean_msg, "failed to open file '"))
        .or_else(|| strip_quoted_label(clean_msg, "failed to read file header from '"))
        .or_else(|| strip_quoted_label(clean_msg, "failed to read file header '"))
        .or_else(|| strip_unquoted_label(clean_msg, "I/O error: "))
        .unwrap_or(clean_msg);
    // Strip trailing " (at …)" span annotation.
    if let Some((before, _)) = after_label.rsplit_once(" (at ") {
        before.trim()
    } else {
        after_label.trim()
    }
}

/// Classify whether a bracket-coded message indicates "file not found" or a
/// generic I/O error.
///
/// Built-in sources embed `[E-DAT-004]` for both file-not-found (the `FileNotFound`
/// variant) and general I/O errors (the `IoError` variant, e.g., permission denied).
/// The dispatcher must inspect the label that follows the bracket code to choose the
/// correct target variant.
///
/// ## Decision table
///
/// | Label prefix after `[E-DAT-004]` | Variant |
/// |-----------------------------------|---------|
/// | `"file not found: "`              | `DataError::FileNotFound` |
/// | `"I/O error reading '"`           | `DataError::IoError` |
/// | `"I/O error: "`                   | `DataError::IoError` (legacy label) |
/// | `"failed to open file '"`         | `DataError::IoError` (`SQLite` magic-check) |
/// | `"failed to read file header '"`  | `DataError::IoError` (`SQLite` magic-check, open) |
/// | `"failed to read file header from '"` | `DataError::IoError` (`SQLite` magic-check, read) |
/// | *(anything else)*                 | `DataError::IoError` (conservative default) |
///
/// Returns `true` when the message is a "file not found" message, `false` for any
/// I/O error variant.
fn message_is_file_not_found(message: &str) -> bool {
    // Skip the bracket code to the label text.
    let after_bracket = match message.find(']') {
        Some(pos) => message
            .get(pos.saturating_add(1)..)
            .unwrap_or("")
            .trim_start(),
        None => return false,
    };
    // Only the "file not found:" label routes to FileNotFound.
    // All other labels (I/O error reading, failed to open file, etc.) route to IoError.
    after_bracket.starts_with("file not found: ")
}

/// Classify whether a bracket-coded `[E-DAT-006]` message is a path-traversal event.
///
/// `FileDataSource::load_path` returns [`DataError::PathTraversalBlocked`] when a path
/// escapes the sandbox. That variant's Display is:
/// `"[E-DAT-006] path traversal blocked: '<path>' is outside base dir (at <span>)"`.
///
/// When this string is translated through `file.rs`'s catch-all arm (`err.to_string()`)
/// into a `DataSourceError::IoError` message, it preserves the `"path traversal blocked: '"`
/// literal substring. The dispatcher inspects that literal to distinguish path-traversal
/// blocks from body-cap policy rejections, which share the same `[E-DAT-006]` bracket code.
///
/// Returns `true` only for path-traversal messages, `false` for body-cap or any other
/// `[E-DAT-006]` sub-case.
///
/// ## Anchored semantics
///
/// This function uses an **anchored** check: it locates the closing `]` of the bracket
/// code, strips leading whitespace from the remainder, and then applies `starts_with`.
/// This mirrors the pattern used by [`message_is_file_not_found`] and prevents a
/// substring that appears only in the *body* of a different error message (e.g., a
/// "see also: path traversal blocked" docs reference in a body-cap message) from
/// yielding a false positive.  Messages without a `]` bracket are rejected (returns
/// `false`), because the caller guarantees that `[E-DAT-006]` context has already been
/// established by the bracket-routing step.
///
/// ## Decision table
///
/// | Label after `]` + `trim_start`          | Returns |
/// |-----------------------------------------|---------|
/// | `"path traversal blocked: '"`           | `true`  |
/// | *(anything else, e.g., body-cap)*        | `false` |
/// | *(no `]` in message)*                   | `false` |
///
/// Traces to F-P7-HIGH-001 (path traversal silently re-routed to `PolicyRejected`).
/// Anchoring fix traces to F-P8-LOW-001.
fn message_is_path_traversal(message: &str) -> bool {
    // Skip past the closing bracket of the bracket code (e.g., "[E-DAT-006]"),
    // then trim leading whitespace to land on the label text.  Apply starts_with
    // so only the label position is tested, not arbitrary substrings in the body.
    let after_bracket = match message.find(']') {
        Some(pos) => message
            .get(pos.saturating_add(1)..)
            .unwrap_or("")
            .trim_start(),
        None => return false,
    };
    after_bracket.starts_with("path traversal blocked: '")
}

/// Extract the traversal-attempt path from a `PathTraversalBlocked` Display message.
///
/// The format emitted by [`DataError::PathTraversalBlocked`] is:
/// `"[E-DAT-006] path traversal blocked: '<path>' is outside base dir (at <span>)"`.
///
/// This function extracts the `<path>` substring that appears between
/// `"path traversal blocked: '"` and the canonical suffix `"' is outside base dir"`.
/// Using `rsplit_once("' is outside base dir")` to locate the closing boundary is
/// structurally anchored: it matches on the canonical suffix regardless of whether
/// the path itself contains single-quote characters (e.g., `/tmp/foo's_dir/file.json`).
/// An earlier implementation used `find('\'')` which would truncate on the first
/// apostrophe in the path.  Traces to F-P8-OBS (apostrophe truncation) and
/// F-P8-LOW-002 (missing direct unit tests).
///
/// If extraction fails for any reason (unexpected format, missing marker, missing
/// suffix, or empty path after extraction), returns `None` and the caller falls back
/// to the URI.
///
/// Traces to F-P7-HIGH-001.
fn extract_path_from_traversal_msg(message: &str) -> Option<&str> {
    // Locate the opening marker: everything after it is the quoted path followed by
    // "' is outside base dir (at <span>)".
    let marker = "path traversal blocked: '";
    let start = message.find(marker)?.checked_add(marker.len())?;
    let rest = message.get(start..)?;
    // Use rsplit_once on the canonical closing suffix so paths containing apostrophes
    // are handled correctly (F-P8-OBS fix: find('\'') would truncate at first apostrophe).
    let (path_raw, _) = rest.rsplit_once("' is outside base dir")?;
    let path = path_raw.trim();
    if path.is_empty() { None } else { Some(path) }
}

/// Extract the path component from a bracket-coded message of the form
/// `"[E-DAT-NNN] <label>: <path>"`, `"[E-DAT-NNN] <label>: '<path>'"`,
/// or `"[E-DAT-NNN] <label>: <path> (at ...)"`.
///
/// The bracket code format used by built-in sources is `[E-DAT-NNN]` (no colon
/// inside the bracket), so the content after the bracket starts with a space
/// followed by the label+path. This function skips past the closing bracket
/// and any label prefix to extract the bare path.
///
/// ## Known label prefixes
///
/// - `"file not found: "` — emitted by `FileDataSource`
/// - `"I/O error: "` — emitted by `FileDataSource` on permission/OS errors (legacy)
/// - `"I/O error reading '"` — canonical `DataError::IoError` Display format
/// - `"failed to open file '"` — emitted by `validate_sqlite_magic` (open failure)
/// - `"failed to read file header from '"` — emitted by `validate_sqlite_magic` (read failure)
/// - `"failed to read file header '"` — retained for backward compatibility
///
/// ## Quoted-path extraction (apostrophe safety)
///
/// For label prefixes that use single-quote delimiters (e.g., `"I/O error reading '"`),
/// the closing boundary is the canonical `"': "` suffix (quote, colon, space) rather
/// than the first apostrophe character. This prevents truncation when the path itself
/// contains an apostrophe (e.g., `/tmp/Bob's_data.json`). The canonical message format
/// emitted by all built-in sources places exactly one `"': "` between the path and the
/// OS reason string. `rsplit_once("': ")` is used so even a path containing `"': "`
/// is handled correctly by anchoring on the LAST occurrence of the separator.
/// Traces to F-P9-MED-001.
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
    // Use rsplit_once so paths containing " (at " are handled correctly
    // (F-P4-LOW-001 fix: first-match `find` would truncate such paths).
    let with_label = if let Some((before_at, _)) = after_bracket.rsplit_once(" (at ") {
        before_at
    } else {
        after_bracket
    };
    // Strip ONLY recognized label prefixes. Return None for unrecognized prefixes
    // so callers fall back to uri.as_str() rather than leaking the label text.
    //
    // Labels that end with a bare colon+space have a plain path following them.
    // Labels that end with a single-quote have a quoted path delimited by the
    // canonical "': " separator. We use rsplit_once("': ") to anchor on the
    // LAST such separator so paths containing apostrophes or "': " are not
    // truncated. Traces to F-P9-MED-001 (apostrophe truncation fix).
    let clean = if let Some(rest) = with_label.strip_prefix("file not found: ") {
        rest.trim()
    } else if let Some(rest) = with_label.strip_prefix("I/O error: ") {
        rest.trim()
    } else if let Some(rest) = with_label.strip_prefix("I/O error reading '") {
        // Quoted path: anchor on the canonical "': " closing separator.
        rest.rsplit_once("': ")
            .map_or(rest, |(path_raw, _)| path_raw)
            .trim()
    } else if let Some(rest) = with_label.strip_prefix("failed to open file '") {
        rest.rsplit_once("': ")
            .map_or(rest, |(path_raw, _)| path_raw)
            .trim()
    } else if let Some(rest) = with_label.strip_prefix("failed to read file header from '") {
        rest.rsplit_once("': ")
            .map_or(rest, |(path_raw, _)| path_raw)
            .trim()
    } else if let Some(rest) = with_label.strip_prefix("failed to read file header '") {
        rest.rsplit_once("': ")
            .map_or(rest, |(path_raw, _)| path_raw)
            .trim()
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
                    // F-P5-MED-004: DataSourceError is #[non_exhaustive]; wildcard required.
                    _ => DataSourceError::IoError {
                        uri: "stub-unreachable".to_owned(),
                        message: format!("stub: unhandled DataSourceError variant: {e}"),
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
                    // F-P5-MED-004: DataSourceError is #[non_exhaustive]; wildcard required.
                    _ => DataSourceError::IoError {
                        uri: "stub-unreachable".to_owned(),
                        message: format!("stub: unhandled DataSourceError variant: {e}"),
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
    /// within the first 32 bytes, return the input unchanged (conservative fallback).
    ///
    /// F-P4-LOW-003: Pin the exact return value for the long-bracket case (not just
    /// non-empty). `strip_bracket_prefix` is documented to return the original string
    /// unchanged when no `]` appears within the byte-limit window.
    ///
    /// F-P9-HIGH-001: The window is now 32 bytes (char_indices-based, UTF-8 safe).
    #[test]
    fn test_strip_bracket_prefix_leaves_malformed_bracket_unchanged() {
        // No closing bracket at all — returns input unchanged.
        assert_eq!(strip_bracket_prefix("[MALFORMED"), "[MALFORMED");
        // Closing bracket beyond position 31 (byte index) — no `]` in the 32-byte window,
        // so the function returns the original string unchanged (documented contract).
        // "[THIS-IS-A-VERY-LONG-BRACKET-CODE]" is 35 bytes, so ']' is at byte 34.
        let long = "[THIS-IS-A-VERY-LONG-BRACKET-CODE] rest";
        assert_eq!(
            strip_bracket_prefix(long),
            long,
            "strip_bracket_prefix must return the original string unchanged when ']' \
            is not within the first 32 bytes (no-bracket-in-window contract)"
        );
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

    // ---------------------------------------------------------------------------
    // F-P4-HIGH-001: permission-denied I/O error must route to DataError::IoError,
    // not DataError::FileNotFound.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_004_dispatcher_routes_permission_denied_to_ioerror_not_filenotfound`
    ///
    /// F-P4-HIGH-001: When an `IoError` message contains `[E-DAT-004]` but with the
    /// "I/O error reading" label (not "file not found"), the dispatcher must route to
    /// `DataError::IoError`, NOT `DataError::FileNotFound`.
    ///
    /// This test exercises the label-discrimination logic in the E-DAT-004 routing arm
    /// via a synthetic message that matches the canonical `DataError::IoError` Display
    /// format, avoiding a live `chmod 0o000` dependency (which is platform-specific).
    ///
    /// The live-file variant is covered by the `#[cfg(unix)]` test below.
    #[test]
    fn test_bc_1_03_004_dispatcher_routes_ioerror_label_to_ioerror_variant() {
        // Simulate what file.rs produces when DataError::IoError is translated via
        // err.to_string(): "[E-DAT-004] I/O error reading '/tmp/locked.csv': permission denied (at ...)"
        let sources = single_error_source(
            "locked",
            DataSourceError::IoError {
                uri: "/tmp/locked.csv".to_owned(),
                message: "[E-DAT-004] I/O error reading '/tmp/locked.csv': \
                    permission denied (at src:1:1)"
                    .to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1, "expected exactly one error");
        // Must NOT be FileNotFound — permission denied is an I/O error, not absence.
        assert!(
            !matches!(errors[0], DataError::FileNotFound { .. }),
            "permission-denied error must NOT route to FileNotFound; got: {:?}",
            errors[0]
        );
        // Must be IoError.
        assert!(
            matches!(errors[0], DataError::IoError { .. }),
            "permission-denied error must route to IoError; got: {:?}",
            errors[0]
        );
        assert_eq!(
            errors[0].code(),
            "E-DAT-004",
            "IoError code must be E-DAT-004; got: {}",
            errors[0].code()
        );
        let display = errors[0].to_string();
        assert!(
            display.contains("permission denied"),
            "IoError Display must include the OS reason 'permission denied'; got: {display}"
        );
        assert!(
            display.contains("/tmp/locked.csv"),
            "IoError Display must include the path; got: {display}"
        );
        // Must NOT say "file not found" — that implies the file is absent.
        assert!(
            !display.contains("file not found"),
            "permission-denied error Display must NOT say 'file not found'; got: {display}"
        );
    }

    /// `test_bc_1_03_004_dispatcher_routes_permission_denied_to_ioerror_not_filenotfound`
    ///
    /// F-P4-HIGH-001 (live-file variant): Use a real file with `chmod 0o000` on Unix
    /// to confirm the full pipeline from `FileDataSource::load` through the dispatcher
    /// routes to `DataError::IoError`, not `DataError::FileNotFound`.
    ///
    /// Skipped on non-Unix platforms (Windows does not honour `chmod 0o000` at the OS
    /// level — the equivalent is a read-locked file which requires different setup).
    #[test]
    #[cfg(unix)]
    fn test_bc_1_03_004_dispatcher_routes_permission_denied_to_ioerror_not_filenotfound() {
        use std::os::unix::fs::PermissionsExt as _;

        use crate::file::FileDataSource;

        // F-P5-OBS-002: RAII guard restores permissions even if the test panics.
        // Without this guard, a mid-test panic would leave the file mode 0o000,
        // causing NamedTempFile's destructor to fail to remove the file.
        struct PermissionGuard<'a> {
            path: &'a str,
            restore_mode: u32,
        }
        impl Drop for PermissionGuard<'_> {
            fn drop(&mut self) {
                use std::os::unix::fs::PermissionsExt as _;
                let _ = std::fs::set_permissions(
                    self.path,
                    std::fs::Permissions::from_mode(self.restore_mode),
                );
            }
        }

        // Use a `.json` suffix so FileDataSource recognises the extension and
        // proceeds to the open() call where EACCES is triggered. Without an
        // extension the source returns UnsupportedFormat before any I/O attempt.
        let tmp = tempfile::Builder::new()
            .suffix(".json")
            .tempfile()
            .expect("tmpfile");
        let path = tmp.path().to_str().expect("utf8 path").to_owned();

        // Write something so the file exists (file-not-found wouldn't reproduce the bug).
        std::fs::write(&path, b"{}").expect("write");

        // Remove all permissions so any open attempt is EACCES.
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000))
            .expect("chmod 0o000");

        // RAII guard: restore mode 0o644 on drop (covers both normal exit and panic).
        let guard = PermissionGuard {
            path: &path,
            restore_mode: 0o644,
        };

        let source = FileDataSource::new(path.as_str());
        let sources: Vec<(Arc<str>, Box<dyn slideforge_plugin_api::DataSource>)> =
            vec![(Arc::from("locked"), Box::new(source))];
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);

        // Guard's Drop will restore permissions at end of scope.
        drop(guard);

        assert_eq!(errors.len(), 1, "expected exactly one error");
        assert!(
            !matches!(errors[0], DataError::FileNotFound { .. }),
            "permission-denied on existing file must NOT route to FileNotFound; got: {:?}",
            errors[0]
        );
        assert!(
            matches!(errors[0], DataError::IoError { .. }),
            "permission-denied on existing file must route to IoError; got: {:?}",
            errors[0]
        );
    }

    // ---------------------------------------------------------------------------
    // F-P4-LOW-001: extract_path_after_code handles paths containing " (at "
    // ---------------------------------------------------------------------------

    /// `test_extract_path_after_code_handles_path_containing_at_in_label`
    ///
    /// F-P4-LOW-001: `extract_path_after_code` uses `rsplit_once(" (at ")` so the
    /// trailing span annotation (always last) is identified correctly even when the
    /// path itself contains the substring `" (at "`.
    #[test]
    fn test_extract_path_after_code_handles_path_containing_at_in_label() {
        // Path contains " (at noon)" — first-match `find` would truncate here.
        let msg = "[E-DAT-004] file not found: /tmp/oddly named (at noon)/file.json \
                   (at SourceSpan { line: 1, col: 1 })";
        let result = extract_path_after_code(msg);
        assert_eq!(
            result,
            Some("/tmp/oddly named (at noon)/file.json"),
            "extract_path_after_code must use rsplit_once so paths with ' (at ' are not \
            truncated; got: {result:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // F-P4-LOW-002: strip_bracket_prefix empty-body falls back to input
    // ---------------------------------------------------------------------------

    // ---------------------------------------------------------------------------
    // F-P9-HIGH-001: strip_bracket_prefix must not panic on non-ASCII UTF-8
    // ---------------------------------------------------------------------------

    /// `test_strip_bracket_prefix_handles_non_ascii_message_safely`
    ///
    /// F-P9-HIGH-001: `strip_bracket_prefix` must NOT panic when the message
    /// contains multi-byte UTF-8 characters that straddle the byte-limit boundary.
    ///
    /// The old implementation used `msg[..16]` byte-slice indexing, which panics
    /// with "byte index N is not a char boundary" when a multi-byte character
    /// (e.g., `Ω` = 2 bytes UTF-8, `🚨` = 4 bytes UTF-8) starts before byte 16
    /// and ends after it. The fix uses `char_indices().take_while(|(i, _)| *i < 32)`,
    /// which always yields valid char boundaries.
    ///
    /// Traces to F-P9-HIGH-001 (DoS via third-party plugin emitting non-ASCII message).
    #[test]
    fn test_strip_bracket_prefix_handles_non_ascii_message_safely() {
        // Case 1: 15 ASCII chars then a 2-byte UTF-8 char (Ω = U+03A9 = 0xCE 0xA9)
        // at byte offset 15.  The old msg[..16] would slice mid-character (byte 16
        // is inside Ω's 2-byte sequence), causing a panic.
        // "[abcdefghijklmnΩ pad after" — '[' at 0, Ω starts at byte 15, ']' absent.
        let msg_omega = "[abcdefghijklmnΩ pad after";
        // Must NOT panic — returns unchanged (no ']' within first 32 bytes at a char boundary).
        let result = strip_bracket_prefix(msg_omega);
        assert_eq!(
            result, msg_omega,
            "non-ASCII message without ']' must be returned unchanged; got: {result:?}"
        );

        // Case 2: Well-formed bracket followed by emoji in the body — must strip bracket.
        // "[E-DAT-001] error with emoji 🚨 in the middle"
        let msg_emoji = "[E-DAT-001] error with emoji \u{1F6A8} in the middle";
        let result2 = strip_bracket_prefix(msg_emoji);
        assert_eq!(
            result2, "error with emoji \u{1F6A8} in the middle",
            "bracket strip must succeed when body contains multi-byte emoji; got: {result2:?}"
        );
        // Most importantly: must not panic.

        // Case 3: Well-formed bracket followed by an accented character — must strip bracket.
        // "[E-DAT-001] ñ-prefix message"
        let msg_tilde = "[E-DAT-001] \u{00F1}-prefix message";
        let result3 = strip_bracket_prefix(msg_tilde);
        assert_eq!(
            result3, "\u{00F1}-prefix message",
            "bracket strip must return ñ-prefix body correctly; got: {result3:?}"
        );
    }

    /// `test_strip_bracket_prefix_empty_body_falls_back_to_input`
    ///
    /// F-P4-LOW-002: When stripping leaves an empty string (e.g., input `"[E-DAT-006]"`
    /// with no body), the function must return the original input rather than an empty
    /// string. This prevents double-space in Display output like `"[E-DAT-006]  (at ...)"`.
    #[test]
    fn test_strip_bracket_prefix_empty_body_falls_back_to_input() {
        // No body after the bracket — stripping would yield ""; fall back to input.
        assert_eq!(
            strip_bracket_prefix("[E-DAT-006]"),
            "[E-DAT-006]",
            "strip_bracket_prefix must return original input when stripping yields empty string"
        );
        // Normal case still strips correctly.
        assert_eq!(
            strip_bracket_prefix("[E-DAT-006] body exceeded"),
            "body exceeded"
        );
    }

    // ---------------------------------------------------------------------------
    // F-P9-MED-001: extract_path_after_code must handle apostrophe in quoted path
    // ---------------------------------------------------------------------------

    /// `test_extract_path_after_code_handles_apostrophe_in_quoted_path`
    ///
    /// F-P9-MED-001: `extract_path_after_code` must return the FULL path when
    /// the path itself contains an apostrophe character. The old implementation
    /// used `rest.split('\'').next()` which truncated at the first apostrophe.
    /// The fix uses `rsplit_once("': ")` anchored on the canonical `"': "` separator.
    ///
    /// Canonical message format emitted by built-in sources:
    ///   `"[E-DAT-004] I/O error reading '/tmp/Bob's_data.json': permission denied (at ...)"`
    ///
    /// Expected: `Some("/tmp/Bob's_data.json")` — full path preserved.
    #[test]
    fn test_extract_path_after_code_handles_apostrophe_in_quoted_path() {
        // Core case: apostrophe inside quoted path, canonical "': " separator present.
        let msg = "[E-DAT-004] I/O error reading '/tmp/Bob's_data.json': \
                   permission denied (at SourceSpan { line: 1, col: 1 })";
        let result = extract_path_after_code(msg);
        assert_eq!(
            result,
            Some("/tmp/Bob's_data.json"),
            "path with apostrophe must be returned in full via rsplit_once(\"': \"); \
            got: {result:?}"
        );

        // Variant: "failed to open file" label with apostrophe in path.
        let msg2 = "[E-DAT-004] failed to open file '/data/O'Brien_db.sqlite': \
                    no such file or directory (at SourceSpan { line: 3, col: 1 })";
        let result2 = extract_path_after_code(msg2);
        assert_eq!(
            result2,
            Some("/data/O'Brien_db.sqlite"),
            "apostrophe in path under 'failed to open file' label must be handled; \
            got: {result2:?}"
        );

        // Variant: "failed to read file header from" label with apostrophe in path.
        let msg3 = "[E-DAT-004] failed to read file header from '/tmp/O'Brien.sqlite': \
                    unexpected end of file (at SourceSpan { line: 1, col: 1 })";
        let result3 = extract_path_after_code(msg3);
        assert_eq!(
            result3,
            Some("/tmp/O'Brien.sqlite"),
            "apostrophe in path under 'failed to read file header from' label must be handled; \
            got: {result3:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // F-P5-MED-001: dispatcher recognizes "failed to read file header from '"
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_004_dispatcher_routes_sqlite_header_failure_correctly`
    ///
    /// F-P5-MED-001: When a SQLite magic-check failure emits
    /// `"[E-DAT-004] failed to read file header from '<path>': <reason>"`,
    /// the dispatcher must:
    ///   1. Route to `DataError::IoError` (not `FileNotFound`).
    ///   2. Carry code `"E-DAT-004"`.
    ///   3. Include the file path EXACTLY ONCE in the Display.
    ///   4. NOT contain `"I/O error reading"` followed by `"failed to read file header"`
    ///      (no double-label wrapping).
    #[test]
    fn test_bc_1_03_004_dispatcher_routes_sqlite_header_failure_correctly() {
        let path = "/data/mydb.sqlite";
        let inner = format!(
            "[{E_DAT_004}] failed to read file header from '{path}': \
            unexpected end of file (at SourceSpan {{ line: 1, col: 1 }})"
        );
        let sources = single_error_source(
            "mydb",
            DataSourceError::IoError {
                uri: path.to_owned(),
                message: inner,
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);

        assert_eq!(errors.len(), 1, "expected exactly one error");

        // Must carry E-DAT-004 (IoError code, not FileNotFound).
        assert_eq!(
            errors[0].code(),
            "E-DAT-004",
            "SQLite header-read failure must carry E-DAT-004; got: {}",
            errors[0].code()
        );

        // Must be IoError, NOT FileNotFound.
        assert!(
            matches!(errors[0], DataError::IoError { .. }),
            "SQLite header-read failure must route to DataError::IoError; got: {:?}",
            errors[0]
        );

        let display = errors[0].to_string();

        // Path must appear exactly once.
        assert_eq!(
            display.matches(path).count(),
            1,
            "path must appear exactly once in Display (no duplicate); got: {display}"
        );

        // Must NOT contain double-label "I/O error reading … failed to read file header".
        assert!(
            !(display.contains("I/O error reading")
                && display.contains("failed to read file header")),
            "Display must not double-wrap the label; got: {display}"
        );
    }

    // ---------------------------------------------------------------------------
    // F-P5-MED-003: extract_bare_io_reason handles "I/O error: " legacy label
    // ---------------------------------------------------------------------------

    /// `test_extract_bare_io_reason_handles_legacy_io_error_label`
    ///
    /// F-P5-MED-003: `extract_bare_io_reason` must handle the legacy
    /// `"I/O error: <reason>"` label (path NOT quoted) and return the bare
    /// reason string, stripping any trailing span annotation.
    #[test]
    fn test_extract_bare_io_reason_handles_legacy_io_error_label() {
        // Legacy label: "I/O error: <path> <reason>" — path is unquoted.
        assert_eq!(
            extract_bare_io_reason("I/O error: permission denied"),
            "permission denied",
            "legacy 'I/O error: ' label must be stripped; reason must be returned"
        );

        // With trailing span annotation.
        assert_eq!(
            extract_bare_io_reason("I/O error: access denied (at src:3:1)"),
            "access denied",
            "trailing span annotation must be stripped from legacy label result"
        );

        // Regular "I/O error reading" label still works.
        assert_eq!(
            extract_bare_io_reason("I/O error reading '/tmp/data.csv': disk full (at src:1:1)"),
            "disk full",
            "quoted 'I/O error reading' label must still be handled"
        );
    }

    // ---------------------------------------------------------------------------
    // F-P5-LOW-002: extract_bare_io_reason unknown-label fallthrough
    // ---------------------------------------------------------------------------

    /// `test_extract_bare_io_reason_unknown_label_falls_through`
    ///
    /// F-P5-LOW-002: When the message does not start with any recognized label,
    /// `extract_bare_io_reason` must return the input trimmed (with the trailing
    /// span annotation stripped), preserving the label text intact.
    #[test]
    fn test_extract_bare_io_reason_unknown_label_falls_through() {
        let input = "some custom error: details (at SourceSpan { line: 5, col: 2 })";
        let result = extract_bare_io_reason(input);
        assert_eq!(
            result, "some custom error: details",
            "unknown label must fall through with span stripped and label preserved; \
            got: {result:?}"
        );

        // No trailing span — returns trimmed input unchanged.
        let no_span = "some custom error: details";
        let result2 = extract_bare_io_reason(no_span);
        assert_eq!(
            result2, "some custom error: details",
            "input without trailing span must be returned trimmed; got: {result2:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // F-P5-LOW-004: routing uses anchored bracket check not substring
    // ---------------------------------------------------------------------------

    /// `test_map_source_error_routes_by_anchored_bracket_not_substring`
    ///
    /// F-P5-LOW-004: A message that starts with `[E-DAT-006]` but mentions
    /// `"E-DAT-001"` in the body (e.g., in a docs reference) must route to
    /// `PolicyRejected` (E-DAT-006), NOT to `HttpError` (E-DAT-001).
    /// This test pins the anchored-bracket routing discipline.
    #[test]
    fn test_map_source_error_routes_by_anchored_bracket_not_substring() {
        let sources = single_error_source(
            "policy_with_ref",
            DataSourceError::IoError {
                uri: "http://example.com/large.json".to_owned(),
                message: "[E-DAT-006] policy rejected; see also E-DAT-001 in docs".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);

        assert_eq!(errors.len(), 1, "expected exactly one error");

        // Must route to PolicyRejected (E-DAT-006), NOT HttpError (E-DAT-001).
        assert_eq!(
            errors[0].code(),
            "E-DAT-006",
            "message starting with [E-DAT-006] must route to PolicyRejected (E-DAT-006), \
            not to HttpError (E-DAT-001); got code: {}",
            errors[0].code()
        );
        assert!(
            matches!(errors[0], DataError::PolicyRejected { .. }),
            "must be DataError::PolicyRejected, not DataError::HttpError; got: {:?}",
            errors[0]
        );
    }

    // ---------------------------------------------------------------------------
    // F-P6-HIGH-002: ParseError arm must not produce double-bracket or double label
    // ---------------------------------------------------------------------------

    /// `test_map_source_error_parse_error_display_single_bracket_and_prefix`
    ///
    /// F-P6-HIGH-002: When a source emits `DataSourceError::ParseError` whose
    /// message already contains "[E-DAT-003] …", the mapped `DataError::ParseError`
    /// Display must have exactly one "[E-DAT-003]", one "parse error for", and the
    /// path mentioned exactly once.
    #[test]
    fn test_map_source_error_parse_error_display_single_bracket_and_prefix() {
        let sources = single_error_source(
            "csv_binding",
            DataSourceError::ParseError {
                uri: "sales.csv".to_owned(),
                message: "[E-DAT-003] unexpected token at line 2".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);

        assert_eq!(errors.len(), 1, "expected exactly one error");
        let display = errors[0].to_string();

        assert_eq!(
            display.matches("[E-DAT-003]").count(),
            1,
            "Display must contain exactly one '[E-DAT-003]' bracket; got: {display:?}"
        );
        assert_eq!(
            display.matches("parse error for").count(),
            1,
            "Display must contain exactly one 'parse error for' phrase; got: {display:?}"
        );
        assert_eq!(
            display.matches("sales.csv").count(),
            1,
            "Display must mention the path exactly once (not duplicated); got: {display:?}"
        );
        assert!(
            display.contains("unexpected token at line 2"),
            "Display must preserve the original reason text; got: {display:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // F-P9-MED-002: ParseError arm must strip trailing "(at ...)" span annotation
    // ---------------------------------------------------------------------------

    /// `test_map_source_error_parse_error_strips_trailing_at_annotation`
    ///
    /// F-P9-MED-002: When a `DataSourceError::ParseError` message already contains
    /// a trailing `"(at line 2:5)"` span annotation, the mapped `DataError::ParseError`
    /// Display must contain the `"(at "` substring at most once — from the outer
    /// `DataError::ParseError` Display, not duplicated from the source message.
    ///
    /// This test uses a message with an embedded span annotation:
    ///   `"[E-DAT-003] unexpected token (at line 2:5)"`
    /// and asserts:
    ///   - `display.matches("(at ").count() <= 1`
    ///   - The original reason text `"unexpected token"` is preserved.
    #[test]
    fn test_map_source_error_parse_error_strips_trailing_at_annotation() {
        let sources = single_error_source(
            "csv_span",
            DataSourceError::ParseError {
                uri: "sales.csv".to_owned(),
                // Message already contains a "(at ...)" span annotation.
                message: "[E-DAT-003] unexpected token (at line 2:5)".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1, "expected exactly one error");

        let display = errors[0].to_string();

        // The "(at " substring must appear at most once in the final Display.
        let at_count = display.matches("(at ").count();
        assert!(
            at_count <= 1,
            "Display must contain '(at ' at most once (no double span annotation); \
            found {at_count} occurrences in: {display:?}"
        );

        // The original reason text must be preserved.
        assert!(
            display.contains("unexpected token"),
            "Display must preserve the original reason text 'unexpected token'; got: {display:?}"
        );

        // Code must be E-DAT-003.
        assert_eq!(
            errors[0].code(),
            "E-DAT-003",
            "ParseError must carry code E-DAT-003; got: {}",
            errors[0].code()
        );
    }

    // ---------------------------------------------------------------------------
    // F-P7-HIGH-001: PathTraversalBlocked must not be re-routed to PolicyRejected
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_004_dispatcher_routes_path_traversal_correctly`
    ///
    /// F-P7-HIGH-001: When `FileDataSource` emits a `DataSourceError::IoError` whose
    /// message is the `DataError::PathTraversalBlocked` Display string (e.g., generated
    /// by `file.rs`'s catch-all arm via `err.to_string()`), the dispatcher must:
    ///   1. Route to `DataError::PathTraversalBlocked` — NOT `DataError::PolicyRejected`.
    ///   2. Preserve the variant identity so callers matching `PathTraversalBlocked` see it.
    ///   3. Display "path traversal blocked" — NOT "data policy rejected".
    ///   4. Carry code `E-DAT-006` (single bracket, not doubled).
    ///   5. The path in the error must be the traversal-attempt path, not the URI.
    ///
    /// This is an integration test that drives a real `FileDataSource` with `base_dir`
    /// set to a `TempDir` and an absolute path pointing outside it, exercises the full
    /// dispatcher pipeline, and asserts on the mapped `DataError` variant.
    ///
    /// Traces to BC-1.03.004, F-P7-HIGH-001.
    #[test]
    fn test_bc_1_03_004_dispatcher_routes_path_traversal_correctly() {
        use crate::file::FileDataSource;

        // Use two separate TempDirs: one as the "sandbox" base_dir, one as the "outside"
        // directory containing a real JSON file. The file must exist so the containment
        // check fires before file-not-found (preventing confounding with FileNotFound).
        let sandbox = tempfile::TempDir::new().expect("sandbox temp dir");
        let outside = tempfile::TempDir::new().expect("outside temp dir");
        let outside_json = outside.path().join("secret.json");
        std::fs::write(&outside_json, br#"{"secret": 42}"#).expect("write outside json");

        // Build a FileDataSource pointing at the outside file, then use load_path to
        // trigger PathTraversalBlocked. We then simulate what file.rs's load() does:
        // translate the DataError into a DataSourceError::IoError via err.to_string().
        let src = FileDataSource::new("");
        let traversal_err = src
            .load_path(&outside_json, Some(sandbox.path()))
            .expect_err("path outside sandbox must return Err");

        // Confirm it is a PathTraversalBlocked at the DataError level.
        assert_eq!(
            traversal_err.code(),
            "E-DAT-006",
            "FileDataSource::load_path must return E-DAT-006 for traversal; got: {traversal_err:?}"
        );
        assert!(
            matches!(traversal_err, crate::DataError::PathTraversalBlocked { .. }),
            "FileDataSource::load_path must return PathTraversalBlocked; got: {traversal_err:?}"
        );

        // Simulate file.rs catch-all: err.to_string() → DataSourceError::IoError.
        // This is what the dispatcher receives when FileDataSource::load() is called.
        let outside_path_str = outside_json.to_string_lossy().to_string();
        // Use a sentinel URI that is DISTINCT from the traversal path. This is load-bearing
        // for Assertion 5 below: the test must prove that the extracted path came from the
        // message (via extract_path_from_traversal_msg), NOT from the URI fallback.
        // If the assertion passes with a sentinel URI, the extractor is doing real work.
        // Traces to F-P8-LOW-003 (URI-vs-extractor ambiguity fix).
        let sentinel_uri = "stub://unrelated-uri-not-the-traversal-path".to_owned();
        let simulated_io_error = slideforge_plugin_api::DataSourceError::IoError {
            uri: sentinel_uri.clone(),
            message: traversal_err.to_string(),
        };

        // Now run the full dispatcher with a stub that returns this DataSourceError.
        let sources: Vec<(Arc<str>, Box<dyn slideforge_plugin_api::DataSource>)> = vec![(
            Arc::from("secret"),
            Box::new(StubSource {
                result: Err(simulated_io_error),
            }),
        )];
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);

        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error from dispatcher"
        );

        // Assertion 1: variant identity is PathTraversalBlocked (not PolicyRejected).
        assert!(
            matches!(errors[0], DataError::PathTraversalBlocked { .. }),
            "dispatcher must route path-traversal IoError to PathTraversalBlocked, \
            not PolicyRejected; got: {:?}",
            errors[0]
        );

        // Assertion 2: code is E-DAT-006 (no double bracket).
        assert_eq!(
            errors[0].code(),
            "E-DAT-006",
            "PathTraversalBlocked must carry E-DAT-006; got: {}",
            errors[0].code()
        );

        let display = errors[0].to_string();

        // Assertion 3: Display says "path traversal blocked", not "data policy rejected".
        assert!(
            display.contains("path traversal blocked"),
            "Display must say 'path traversal blocked'; got: {display}"
        );
        assert!(
            !display.contains("data policy rejected"),
            "Display must NOT say 'data policy rejected' for path traversal; got: {display}"
        );

        // Assertion 4: exactly one [E-DAT-006] bracket (no double bracket).
        assert_eq!(
            display.matches("[E-DAT-006]").count(),
            1,
            "Display must contain exactly one [E-DAT-006]; got: {display}"
        );

        // Assertion 5: the path in the error is the traversal-attempt path extracted from
        // the message by extract_path_from_traversal_msg, NOT the sentinel URI fallback.
        // Because the sentinel URI differs from the traversal path, the only way
        // Display can contain outside_path_str is if the extractor was load-bearing.
        assert!(
            display.contains(outside_path_str.as_str()),
            "Display must contain the traversal-attempt path '{outside_path_str}' \
            (extracted from message, not from URI); got: {display}"
        );
        assert!(
            !display.contains(sentinel_uri.as_str()),
            "Display must NOT contain the sentinel URI '{sentinel_uri}' — \
            the traversal path must come from the message extractor, not the URI fallback; \
            got: {display}"
        );
    }

    // ---------------------------------------------------------------------------
    // F-P8-LOW-001 + F-P8-LOW-002: message_is_path_traversal anchored semantics
    //                               and extract_path_from_traversal_msg direct tests
    // ---------------------------------------------------------------------------

    /// `test_message_is_path_traversal_recognizes_canonical_message`
    ///
    /// F-P8-LOW-001: The canonical `PathTraversalBlocked` Display message must be
    /// recognized as a path-traversal event.  The check is label-position anchored:
    /// the label text must appear immediately after the `]` bracket (post trim_start),
    /// not anywhere in the body.
    #[test]
    fn test_message_is_path_traversal_recognizes_canonical_message() {
        let msg = "[E-DAT-006] path traversal blocked: '/etc/passwd' is outside base dir \
                   (at SourceSpan { line: 1, col: 1 })";
        assert!(
            message_is_path_traversal(msg),
            "canonical PathTraversalBlocked display must be recognized; got false for: {msg:?}"
        );
    }

    /// `test_message_is_path_traversal_rejects_body_cap_message`
    ///
    /// F-P8-LOW-001: A body-cap policy message that mentions "path traversal blocked"
    /// only in its *body* (not as the label immediately after `]`) must NOT be
    /// recognized as a path-traversal event.  This is the false-positive the anchored
    /// check prevents.
    #[test]
    fn test_message_is_path_traversal_rejects_body_cap_message() {
        let msg = "[E-DAT-006] response body exceeds 52428800-byte cap \
                   (policy-rejected — see also: path traversal blocked: '<x>' in unrelated docs)";
        assert!(
            !message_is_path_traversal(msg),
            "body-cap message with 'path traversal blocked' in body must return false; \
            got true for: {msg:?}"
        );
    }

    /// `test_message_is_path_traversal_rejects_ssrf_message`
    ///
    /// F-P8-LOW-001: An SSRF-blocked policy message must not be recognized as
    /// path-traversal even though it carries the same `[E-DAT-006]` bracket code.
    #[test]
    fn test_message_is_path_traversal_rejects_ssrf_message() {
        let msg = "[E-DAT-006] HTTP source 'http://example.com' blocked by allowed_domains \
                   policy. Add 'example.com' to the allowed list to permit this source.";
        assert!(
            !message_is_path_traversal(msg),
            "SSRF policy message must return false; got true for: {msg:?}"
        );
    }

    /// `test_message_is_path_traversal_rejects_no_bracket`
    ///
    /// F-P8-LOW-001: A message with no `]` bracket (i.e., not prefixed with a bracket
    /// code) must return `false`.  The helper assumes the caller has already established
    /// `[E-DAT-006]` context via the bracket-routing step; a bare string without a
    /// bracket guard is rejected defensively.
    #[test]
    fn test_message_is_path_traversal_rejects_no_bracket() {
        let msg = "path traversal blocked: '/x'";
        assert!(
            !message_is_path_traversal(msg),
            "message without bracket prefix must return false; got true for: {msg:?}"
        );
    }

    /// `test_extract_path_from_traversal_msg_extracts_well_formed`
    ///
    /// F-P8-LOW-002: The canonical well-formed `PathTraversalBlocked` Display message
    /// must yield `Some("/etc/passwd")`.
    #[test]
    fn test_extract_path_from_traversal_msg_extracts_well_formed() {
        let msg = "[E-DAT-006] path traversal blocked: '/etc/passwd' is outside base dir \
                   (at SourceSpan { line: 1, col: 1 })";
        let result = extract_path_from_traversal_msg(msg);
        assert_eq!(
            result,
            Some("/etc/passwd"),
            "well-formed message must extract path '/etc/passwd'; got: {result:?}"
        );
    }

    /// `test_extract_path_from_traversal_msg_returns_none_missing_closing_suffix`
    ///
    /// F-P8-LOW-002: When the canonical closing suffix `"' is outside base dir"` is absent
    /// (e.g., the path's closing quote is missing), the function must return `None`.
    #[test]
    fn test_extract_path_from_traversal_msg_returns_none_missing_closing_suffix() {
        // No closing "' is outside base dir" — rsplit_once will return None.
        let msg = "[E-DAT-006] path traversal blocked: '/etc/passwd is outside base dir \
                   (at SourceSpan { line: 1, col: 1 })";
        let result = extract_path_from_traversal_msg(msg);
        assert_eq!(
            result, None,
            "message missing closing suffix must return None; got: {result:?}"
        );
    }

    /// `test_extract_path_from_traversal_msg_returns_none_empty_path`
    ///
    /// F-P8-LOW-002: A message with an empty quoted path (`''`) must return `None`
    /// because an empty path is not actionable.
    #[test]
    fn test_extract_path_from_traversal_msg_returns_none_empty_path() {
        let msg = "[E-DAT-006] path traversal blocked: '' is outside base dir \
                   (at SourceSpan { line: 1, col: 1 })";
        let result = extract_path_from_traversal_msg(msg);
        assert_eq!(
            result, None,
            "message with empty path must return None; got: {result:?}"
        );
    }

    /// `test_extract_path_from_traversal_msg_returns_none_missing_marker`
    ///
    /// F-P8-LOW-002: A message that does not contain the `"path traversal blocked: '"`
    /// marker must return `None`.
    #[test]
    fn test_extract_path_from_traversal_msg_returns_none_missing_marker() {
        let msg = "some other error message without the path traversal marker";
        let result = extract_path_from_traversal_msg(msg);
        assert_eq!(
            result, None,
            "message without traversal marker must return None; got: {result:?}"
        );
    }

    /// `test_extract_path_from_traversal_msg_handles_apostrophe_in_path`
    ///
    /// F-P8-OBS: A path containing an apostrophe (e.g., `/tmp/foo's_dir/file.json`)
    /// must be extracted correctly.  The earlier `find('\'')` implementation would
    /// have truncated at the first apostrophe and returned `Some("/tmp/foo")`.
    /// The `rsplit_once("' is outside base dir")` fix extracts the full path.
    #[test]
    fn test_extract_path_from_traversal_msg_handles_apostrophe_in_path() {
        let msg = "[E-DAT-006] path traversal blocked: '/tmp/foo's_dir/file.json' \
                   is outside base dir (at SourceSpan { line: 1, col: 1 })";
        let result = extract_path_from_traversal_msg(msg);
        assert_eq!(
            result,
            Some("/tmp/foo's_dir/file.json"),
            "path with apostrophe must be extracted in full; got: {result:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // F-P6-MED-001: DataSourceError Display contract — never starts with '['
    // ---------------------------------------------------------------------------

    /// `test_data_source_error_display_never_starts_with_bracket`
    ///
    /// F-P6-MED-001: The dead-code removal of `|| has_bracket(message.as_ref(), …)` is
    /// safe only because DataSourceError's Display prefixes always start with a literal
    /// word (e.g., "data parse error for '…'", "I/O error for '…'"), never with "[".
    /// This test pins that upstream contract so a future Display change doesn't silently
    /// reactivate the dead-code assumption.
    #[test]
    fn test_data_source_error_display_never_starts_with_bracket() {
        let variants: Vec<DataSourceError> = vec![
            DataSourceError::ParseError {
                uri: "test.csv".to_owned(),
                message: "[E-DAT-003] some parse failure".to_owned(),
            },
            DataSourceError::IoError {
                uri: "test.csv".to_owned(),
                message: "[E-DAT-004] some io failure".to_owned(),
            },
            DataSourceError::UnsupportedUri {
                uri: "ftp://test.csv".to_owned(),
            },
            DataSourceError::AuthError {
                uri: "https://api.example.com/".to_owned(),
            },
        ];

        for variant in &variants {
            let display = variant.to_string();
            assert!(
                !display.starts_with('['),
                "DataSourceError::Display must never start with '['; display started with \
                '[', which would incorrectly make the dead-code `has_bracket(message.as_ref(), \
                ...)` clauses live again; display: {display:?}",
            );
        }
    }

    // ---------------------------------------------------------------------------
    // F-P10-HIGH-001: ParseError arm must preserve granular E-DAT-007..014 codes
    // ---------------------------------------------------------------------------

    /// `test_map_source_error_parse_error_preserves_e_dat_007`
    ///
    /// F-P10-HIGH-001: `ParseError` with `[E-DAT-007]` (XLSX empty header) must
    /// produce `DataError::ParseError.code() == "E-DAT-007"`, NOT "E-DAT-003".
    /// This test drives `parse_e_dat_code` through the dispatcher's full routing path.
    #[test]
    fn test_map_source_error_parse_error_preserves_e_dat_007() {
        let sources = single_error_source(
            "sheet",
            DataSourceError::ParseError {
                uri: "/data/report.xlsx".to_owned(),
                message: "[E-DAT-007] xlsx empty header: column 2 header cell is empty".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].code(),
            "E-DAT-007",
            "ParseError with [E-DAT-007] must carry E-DAT-007, not E-DAT-003; got: {}, display: {}",
            errors[0].code(),
            errors[0]
        );
    }

    /// `test_map_source_error_parse_error_preserves_granular_codes`
    ///
    /// F-P10-HIGH-001: Table-driven test that E-DAT-008 through E-DAT-014 are all
    /// preserved by the dispatcher's `ParseError` branch. Each code corresponds to a
    /// distinct XLSX or SQLite parse failure from the error taxonomy.
    #[test]
    fn test_map_source_error_parse_error_preserves_granular_codes() {
        use crate::error::{
            E_DAT_008, E_DAT_009, E_DAT_010, E_DAT_011, E_DAT_012, E_DAT_013, E_DAT_014,
        };
        // Table: (code_constant, human_readable_message)
        let cases: &[(&str, &str)] = &[
            (
                E_DAT_008,
                "[E-DAT-008] xlsx header cell non-string type: Int at column 1",
            ),
            (
                E_DAT_009,
                "[E-DAT-009] xlsx DateTimeIso cell invalid ISO 8601: 'not-a-date'",
            ),
            (
                E_DAT_010,
                "[E-DAT-010] xlsx non-finite float cell: NaN in column 3 row 2",
            ),
            (
                E_DAT_011,
                "[E-DAT-011] xlsx bad ZIP magic: file is not a valid xlsx archive",
            ),
            (
                E_DAT_012,
                "[E-DAT-012] sqlite TEXT column invalid UTF-8 bytes at row 5",
            ),
            (
                E_DAT_013,
                "[E-DAT-013] sqlite bad header magic: not a sqlite3 database",
            ),
            (
                E_DAT_014,
                "[E-DAT-014] sqlite unsupported extension '.xls' (expected .sqlite, .db)",
            ),
        ];

        let ctx = DataSourceContext::new();
        for (expected_code, msg) in cases {
            let sources = single_error_source(
                "granular",
                DataSourceError::ParseError {
                    uri: "/data/source.xlsx".to_owned(),
                    message: msg.to_string(),
                },
            );
            let (_scope, errors) = load_all(&sources, &ctx);
            assert_eq!(
                errors.len(),
                1,
                "expected exactly one error for code {expected_code}"
            );
            assert_eq!(
                errors[0].code(),
                *expected_code,
                "ParseError with {expected_code} must carry that code, not E-DAT-003; \
                got: {}, message: {}, display: {}",
                errors[0].code(),
                msg,
                errors[0]
            );
        }
    }

    /// `test_parse_e_dat_code_returns_correct_constants`
    ///
    /// F-P10-HIGH-001: Unit test for `parse_e_dat_code` directly — validates that each
    /// known bracket prefix maps to the correct constant and unknown prefixes return None.
    #[test]
    fn test_parse_e_dat_code_returns_correct_constants() {
        use crate::error::{
            E_DAT_008, E_DAT_009, E_DAT_010, E_DAT_011, E_DAT_012, E_DAT_013, E_DAT_014,
        };
        // Known codes must match.
        assert_eq!(
            parse_e_dat_code("[E-DAT-001] some http error"),
            Some(E_DAT_001)
        );
        assert_eq!(parse_e_dat_code("[E-DAT-003] parse error"), Some(E_DAT_003));
        assert_eq!(
            parse_e_dat_code("[E-DAT-007] xlsx empty header"),
            Some(E_DAT_007)
        );
        assert_eq!(
            parse_e_dat_code("[E-DAT-008] non-string header"),
            Some(E_DAT_008)
        );
        assert_eq!(
            parse_e_dat_code("[E-DAT-009] invalid ISO date"),
            Some(E_DAT_009)
        );
        assert_eq!(parse_e_dat_code("[E-DAT-010] NaN float"), Some(E_DAT_010));
        assert_eq!(
            parse_e_dat_code("[E-DAT-011] bad zip magic"),
            Some(E_DAT_011)
        );
        assert_eq!(
            parse_e_dat_code("[E-DAT-012] invalid utf8"),
            Some(E_DAT_012)
        );
        assert_eq!(
            parse_e_dat_code("[E-DAT-013] bad sqlite magic"),
            Some(E_DAT_013)
        );
        assert_eq!(
            parse_e_dat_code("[E-DAT-014] unsupported ext"),
            Some(E_DAT_014)
        );
        assert_eq!(parse_e_dat_code("[E-DAT-015] unspecified"), Some(E_DAT_015));
        // Unknown prefix must return None.
        assert_eq!(parse_e_dat_code("[E-DAT-099] unknown code"), None);
        assert_eq!(parse_e_dat_code("no bracket at all"), None);
        assert_eq!(parse_e_dat_code(""), None);
        // Bracket code in body but NOT at start: must return None.
        assert_eq!(parse_e_dat_code("prefix text [E-DAT-007] body"), None);
    }

    // ---------------------------------------------------------------------------
    // F-P10-MED-002: extract_bare_io_reason must use rsplit_once (apostrophe safety)
    // ---------------------------------------------------------------------------

    /// `test_extract_bare_io_reason_handles_apostrophe_in_quoted_path`
    ///
    /// F-P10-MED-002: `extract_bare_io_reason` inner helper `strip_quoted_label` used
    /// `split_once("': ")` (first match) instead of `rsplit_once("': ")` (last match).
    /// For a path like `/tmp/Bob's_data.csv`, the first `"': "` separator would split
    /// at the apostrophe, yielding only the OS reason AFTER the apostrophe — not the
    /// correct reason text after the path.
    ///
    /// Expected: `"permission denied"` (the OS reason after the canonical `"': "` separator).
    #[test]
    fn test_extract_bare_io_reason_handles_apostrophe_in_quoted_path() {
        // Input: bracket already stripped. Path contains apostrophe.
        // Format: "I/O error reading '<path-with-apostrophe>': <reason>"
        let input = "I/O error reading '/tmp/Bob's_data.csv': permission denied";
        let result = extract_bare_io_reason(input);
        assert_eq!(
            result, "permission denied",
            "extract_bare_io_reason must strip 'I/O error reading' label and return only the \
            OS reason even when the path contains an apostrophe; got: {result:?}"
        );

        // Variant: "failed to open file" label with apostrophe in path.
        let input2 = "failed to open file '/data/O'Brien_db.sqlite': no such file or directory";
        let result2 = extract_bare_io_reason(input2);
        assert_eq!(
            result2, "no such file or directory",
            "extract_bare_io_reason must handle apostrophe in path under 'failed to open file' \
            label; got: {result2:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // F-P10-LOW-001: ParseError for no-extension URI uses DataFormat::Unknown
    // ---------------------------------------------------------------------------

    /// `test_map_source_error_parse_error_no_extension_uses_unknown_format`
    ///
    /// F-P10-LOW-001: When a `ParseError` has an HTTP URI with no file extension
    /// (e.g., `"http://api.example.com/data"`), the format fallback must be
    /// `DataFormat::Unknown`, NOT `DataFormat::Json`. The Display must NOT contain
    /// `"(Json)"` as that would be misleading for actual CSV/YAML from HTTP.
    #[test]
    fn test_map_source_error_parse_error_no_extension_uses_unknown_format() {
        let sources = single_error_source(
            "api_no_ext",
            DataSourceError::ParseError {
                uri: "http://api.example.com/data".to_owned(),
                message: "[E-DAT-003] unexpected token at offset 42".to_owned(),
            },
        );
        let ctx = DataSourceContext::new();
        let (_scope, errors) = load_all(&sources, &ctx);
        assert_eq!(errors.len(), 1);
        let display = errors[0].to_string();
        assert!(
            !display.contains("(Json)"),
            "ParseError for no-extension HTTP URI must NOT report '(Json)'; got: {display}"
        );
        assert!(
            display.contains("(Unknown)"),
            "ParseError for no-extension HTTP URI must report '(Unknown)'; got: {display}"
        );
        assert_eq!(
            errors[0].code(),
            "E-DAT-003",
            "code must still be E-DAT-003; got: {}",
            errors[0].code()
        );
    }

    // ---------------------------------------------------------------------------
    // F-P10-LOW-003: extract_path_after_code handles apostrophe + " (at " combined
    // ---------------------------------------------------------------------------

    /// `test_extract_path_after_code_handles_apostrophe_and_at_combined`
    ///
    /// F-P10-LOW-003: Confirms that `rsplit_once(" (at ")` (span strip) and
    /// `rsplit_once("': ")` (path/reason strip) compose correctly when a path contains
    /// BOTH an apostrophe and the substring `" (at "`.
    ///
    /// Input format:
    ///   `"[E-DAT-004] I/O error reading '/tmp/Bob's (at) folder/data.csv': \
    ///    permission denied (at SourceSpan { line: 1, col: 1 })"`
    ///
    /// Expected: `Some("/tmp/Bob's (at) folder/data.csv")`
    ///
    /// The `rsplit_once(" (at ")` strips the LAST `" (at "` (the trailing span annotation),
    /// leaving `"[E-DAT-004] I/O error reading '/tmp/Bob's (at) folder/data.csv': \
    /// permission denied"`. Then `rsplit_once("': ")` anchors on the LAST `"': "` to
    /// extract the path correctly.
    #[test]
    fn test_extract_path_after_code_handles_apostrophe_and_at_combined() {
        let msg = "[E-DAT-004] I/O error reading '/tmp/Bob's (at) folder/data.csv': \
                   permission denied (at SourceSpan { line: 1, col: 1 })";
        let result = extract_path_after_code(msg);
        assert_eq!(
            result,
            Some("/tmp/Bob's (at) folder/data.csv"),
            "path with apostrophe and ' (at ' substring must be extracted correctly \
            via rsplit_once composition; got: {result:?}"
        );
    }
}
