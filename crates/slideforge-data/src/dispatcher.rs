//! Dispatcher for loading all configured `DataSource` instances.
//!
//! STORY-021 / BC-1.03.004. Currently a Red Gate stub — implementer will body
//! in the green-phase.
//!
//! # Design: offline-capability flag
//!
//! The [`load_all`] function receives an `offline_capable` boolean per source
//! (alongside the source itself). This is necessary because
//! [`slideforge_plugin_api::DataSource`] does not currently declare a
//! `supports_offline()` method — `supports_offline()` is a concrete method
//! only on [`crate::http::HttpDataSource`]. The implementer's first task is
//! to decide whether to:
//!
//! a) Add `fn supports_offline(&self) -> bool { false }` as a defaulted method
//!    to the `DataSource` trait in `slideforge-plugin-api`, and then use
//!    `source.supports_offline()` directly in the dispatcher loop, OR
//!
//! b) Keep the explicit `bool` flag in the input tuple (current interface).
//!
//! The production-grade approach is (a): it eliminates the boolean duplication
//! and allows third-party plugins to declare their own offline semantics.
//! Option (b) is valid as a stepping stone if the trait change requires a
//! separate PR.
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
use slideforge_plugin_api::DataSource;
use slideforge_types::Value;

use crate::DataError;
use crate::context::DataSourceContext;

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
    let _ = sources;
    let _ = ctx;
    todo!("BC-1.03.004 — STORY-021 — Red Gate: load_all dispatcher not yet implemented")
}

/// Convert a [`slideforge_plugin_api::DataSourceError`] to a [`DataError`].
///
/// Used internally to map the plugin-API error type (returned by
/// `DataSource::load`) to the crate-internal `DataError` type that carries
/// richer span and code information.
///
/// This is a stub helper that the implementer will body during the green phase.
#[must_use]
#[allow(dead_code)]
fn map_source_error(
    name: &Arc<str>,
    err: &slideforge_plugin_api::DataSourceError,
) -> DataError {
    let _ = name;
    let _ = err;
    todo!("BC-1.03.004 — STORY-021 — Red Gate: map_source_error not yet implemented")
}
