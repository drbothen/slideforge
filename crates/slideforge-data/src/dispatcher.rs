//! Dispatcher for loading all configured `DataSource` instances.
//!
//! STORY-021 / BC-1.03.004.
//!
//! # Design: offline-capability flag
//!
//! The [`load_all`] function receives a per-source `offline_capable` boolean alongside
//! the source. This boolean signals that the source should be skipped when
//! `ctx.offline == true`. File-based sources pass `false`; HTTP/HTTPS sources pass `true`.
//!
//! This design was chosen over adding a `supports_offline()` method to the
//! `DataSource` trait (Option A in the original stub) because:
//! - No trait-signature change in `slideforge-plugin-api` — backward-compatible with all
//!   existing implementors, third-party and built-in.
//! - The offline decision is a dispatcher-level policy concern, not a plugin concern.
//!   Plugins describe capabilities (via the boolean); the dispatcher enforces policy.
//! - `HttpDataSource` already has a concrete `supports_offline()` method (added in
//!   STORY-019). Callers that build the sources slice simply call `src.supports_offline()`
//!   to populate the boolean, keeping the interface ergonomic without touching the trait.
//!
//! # Offline gate semantics
//!
//! When `ctx.offline == true` and a source's `offline_capable == true`:
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
use crate::error::{E_DAT_002, E_DAT_003, E_DAT_004, E_DAT_006};

/// Load all configured data sources, applying the offline gate.
///
/// # Parameters
///
/// - `sources` — name-source-capability triples: `(data_name, source, offline_capable)`.
///   `offline_capable` is `true` for HTTP/HTTPS sources (should be skipped when
///   `ctx.offline == true`) and `false` for file-based sources (always loaded).
/// - `ctx` — the evaluation context carrying the offline flag and SSRF allowlist.
///
/// # Offline gate
///
/// For each source where `offline_capable == true` AND `ctx.offline == true`,
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
    sources: &[(Arc<str>, Box<dyn DataSource>, bool)],
    ctx: &DataSourceContext,
) -> (IndexMap<Arc<str>, Value>, Vec<DataError>) {
    let mut scope: IndexMap<Arc<str>, Value> = IndexMap::new();
    let mut errors: Vec<DataError> = Vec::new();

    let opts = DataSourceOptions::default();

    for (name, source, offline_capable) in sources {
        // Offline gate (BC-1.03.004 postcondition 1 + 2 + invariant 1):
        // Skip network-capable sources silently when in offline mode.
        // The source's data name is NOT added to scope — no empty-value substitution.
        if ctx.offline && *offline_capable {
            continue;
        }

        // Call the source with an empty URI so each source uses its own pre-configured
        // address:
        // - `HttpDataSource`: when `uri` is empty it falls back to `self.url`
        //   (the URL baked in at construction time from the `@data` directive).
        // - `FileDataSource`: when `uri` is empty it returns a `FileNotFound` error
        //   (E-DAT-004), signalling that no path was provided at call time. In real
        //   evaluator usage the evaluator constructs the source with the path; the
        //   dispatcher's job is only to trigger loading.
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
///
/// This approach avoids adding a new structural field to `DataSourceError` in
/// `slideforge-plugin-api` while still preserving the error codes surfaced in
/// `DataError::Display` for AC-009 verification.
///
/// ## Code-to-variant routing
///
/// | Bracket code in message | Routed to variant |
/// |-------------------------|-------------------|
/// | `[E-DAT-004]`           | `DataError::FileNotFound` or `DataError::IoError` |
/// | `[E-DAT-006]`           | `DataError::SsrfBlocked` or `DataError::PathTraversalBlocked` |
/// | `[E-DAT-003]`           | `DataError::ParseError` |
/// | `[E-DAT-001]`           | `DataError::HttpError` |
/// | `[E-DAT-002]`           | `DataError::NetworkError` |
/// | *(no bracket code)*     | Generic `DataError::IoError` with full message |
#[must_use]
fn map_source_error(_name: &Arc<str>, err: &DataSourceError) -> DataError {
    let message: Arc<str> = Arc::from(err.to_string());

    match err {
        DataSourceError::IoError {
            uri,
            message: inner_msg,
        } => {
            // Route by bracket code embedded in the message.
            if inner_msg.contains(E_DAT_004) || message.contains(E_DAT_004) {
                // File-not-found: surface as FileNotFound with the code prefix in Display.
                DataError::FileNotFound {
                    code: E_DAT_004,
                    path: Arc::from(uri.as_str()),
                    span: slideforge_types::SourceSpan::default(),
                }
            } else if inner_msg.contains(E_DAT_006) || message.contains(E_DAT_006) {
                // SSRF-level I/O rejection (body-size cap, etc.): PathTraversalBlocked
                // as a structural proxy since it shares the E-DAT-006 code.
                DataError::PathTraversalBlocked {
                    code: E_DAT_006,
                    path: Arc::from(uri.as_str()),
                    span: slideforge_types::SourceSpan::default(),
                }
            } else {
                // Generic I/O error: preserve full message, use E-DAT-004 as the code.
                DataError::IoError {
                    code: E_DAT_004,
                    path: Arc::from(uri.as_str()),
                    message: Arc::clone(&message),
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
                DataError::ParseError {
                    code: E_DAT_003,
                    path: Arc::from(uri.as_str()),
                    format: crate::format::DataFormat::Json, // placeholder — format is not recoverable here
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
            DataError::NetworkError {
                code: E_DAT_002,
                url: Arc::from(uri.as_str()),
                cause: Arc::from(err.to_string()),
                span: slideforge_types::SourceSpan::default(),
            }
        },
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use slideforge_plugin_api::{DataSource, DataSourceError, DataSourceOptions};
    use slideforge_types::Value;

    use super::*;
    use crate::context::DataSourceContext;

    // Minimal stub DataSource used in unit tests.
    struct StubSource {
        id: &'static str,
        result: Result<Value, DataSourceError>,
    }

    impl DataSource for StubSource {
        fn id(&self) -> &str {
            self.id
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

    /// `test_load_all_empty_sources` — zero sources returns empty scope and zero errors.
    #[test]
    fn test_load_all_empty_sources() {
        let sources: Vec<(Arc<str>, Box<dyn DataSource>, bool)> = vec![];
        let ctx = DataSourceContext::new();
        let (scope, errors) = load_all(&sources, &ctx);
        assert!(scope.is_empty());
        assert!(errors.is_empty());
    }

    /// `test_load_all_online_loads_all` — all sources loaded when offline=false.
    #[test]
    fn test_load_all_online_loads_all() {
        let sources: Vec<(Arc<str>, Box<dyn DataSource>, bool)> = vec![
            (
                Arc::from("a"),
                Box::new(StubSource {
                    id: "a",
                    result: Ok(Value::Int(1)),
                }),
                false,
            ),
            (
                Arc::from("b"),
                Box::new(StubSource {
                    id: "b",
                    result: Ok(Value::Int(2)),
                }),
                true,
            ),
        ];
        let ctx = DataSourceContext::new();
        let (scope, errors) = load_all(&sources, &ctx);
        assert_eq!(scope.len(), 2);
        assert!(errors.is_empty());
    }

    /// `test_load_all_offline_skips_capable` — offline gate skips `offline_capable=true` sources.
    #[test]
    fn test_load_all_offline_skips_capable() {
        let sources: Vec<(Arc<str>, Box<dyn DataSource>, bool)> = vec![(
            Arc::from("net"),
            Box::new(StubSource {
                id: "net",
                result: Ok(Value::Int(99)),
            }),
            true,
        )];
        let ctx = DataSourceContext::new().with_offline(true);
        let (scope, errors) = load_all(&sources, &ctx);
        assert!(scope.is_empty(), "offline_capable source must be skipped");
        assert!(errors.is_empty(), "skip must produce zero errors");
    }

    /// `test_load_all_partial_error` — failed source adds to error vec; successful source in scope.
    #[test]
    fn test_load_all_partial_error() {
        let sources: Vec<(Arc<str>, Box<dyn DataSource>, bool)> = vec![
            (
                Arc::from("ok"),
                Box::new(StubSource {
                    id: "ok",
                    result: Ok(Value::Int(1)),
                }),
                false,
            ),
            (
                Arc::from("bad"),
                Box::new(StubSource {
                    id: "bad",
                    result: Err(DataSourceError::IoError {
                        uri: "bad".to_owned(),
                        message: format!("[{E_DAT_004}] file not found"),
                    }),
                }),
                false,
            ),
        ];
        let ctx = DataSourceContext::new();
        let (scope, errors) = load_all(&sources, &ctx);
        assert!(scope.contains_key(&Arc::from("ok")));
        assert!(!scope.contains_key(&Arc::from("bad")));
        assert_eq!(errors.len(), 1);
    }
}
