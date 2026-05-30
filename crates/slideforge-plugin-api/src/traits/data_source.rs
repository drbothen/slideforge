//! [`DataSource`] trait — fetches external data for use in slideforge expressions.
//!
//! A `DataSource` plugin resolves a URI (e.g., `"data/sales.json"`) and returns a
//! [`Value`] that the evaluator can bind to a variable. Built-in implementations
//! handle JSON, CSV, YAML, TOML, `SQLite`, HTTP, and XLSX. External plugins can
//! add more source types without modifying the core.

use slideforge_types::Value;
use thiserror::Error;

/// Options passed to [`DataSource::load`] at evaluation time.
///
/// This struct carries per-call configuration that may differ between
/// evaluations of the same data source plugin (e.g., query parameters,
/// timeout overrides, or authentication headers for HTTP sources).
///
/// ## Plugin convention for ignored options
///
/// Plugins SHOULD emit `tracing::warn!` for options they silently ignore
/// **if and only if** the option is conceptually applicable to the plugin.
/// For example:
/// - An `SQLite` plugin that receives a non-empty `query` but only supports
///   `SELECT *` should warn.
/// - An HTTP plugin that receives `auth_token` but cannot authenticate with
///   it should warn.
///
/// File-based sources (XLSX, file-format CSV/JSON/YAML/TOML, etc.) MAY
/// silently ignore options that have no semantic meaning for their format
/// (e.g., `timeout_ms` on a local-file read, `auth_token` on a plain
/// XLSX or JSON file). No warning is required in these cases.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct DataSourceOptions {
    /// Optional timeout in milliseconds. `None` means use the plugin default.
    pub timeout_ms: Option<u64>,

    /// Optional authentication token for sources that require it (e.g., HTTP
    /// APIs). The interpretation is plugin-specific.
    pub auth_token: Option<std::sync::Arc<str>>,

    /// Optional query string or filter expression. Interpretation is
    /// plugin-specific (e.g., a SQL `WHERE` clause for `SQLite` sources).
    pub query: Option<std::sync::Arc<str>>,
}

/// Error returned by [`DataSource::load`] when data cannot be fetched or parsed.
///
/// # `SemVer` policy
///
/// This enum is `#[non_exhaustive]`. External plugin authors and internal callers
/// that pattern-match on this enum must include a wildcard arm (`_ => ...`) to
/// remain forward-compatible as new variants are added in future releases.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DataSourceError {
    /// The URI scheme or format is not supported by this plugin.
    #[error("unsupported URI scheme or format: {uri}")]
    UnsupportedUri {
        /// The URI that could not be resolved.
        uri: String,
    },

    /// The data at the URI could not be parsed as the expected format.
    #[error("data parse error for '{uri}': {message}")]
    ParseError {
        /// The URI of the source that failed to parse.
        uri: String,
        /// A human-readable description of the parse failure.
        message: String,
    },

    /// An I/O error occurred while reading the data (e.g., file not found,
    /// network timeout).
    #[error("I/O error for '{uri}': {message}")]
    IoError {
        /// The URI that produced the I/O error.
        uri: String,
        /// Description of the I/O failure.
        message: String,
    },

    /// Authentication failed for the data source.
    #[error("authentication failed for '{uri}'")]
    AuthError {
        /// The URI for which authentication failed.
        uri: String,
    },
}

/// A plugin that fetches external data and returns a [`Value`] to the evaluator.
///
/// Implement this trait to add new data source types (e.g., a custom database
/// connector, a REST API client, or a proprietary file format reader). Register
/// your implementation with [`crate::PluginRegistry::register_data_source`].
///
/// ## Thread safety
///
/// All implementations must be `Send + Sync` because the plugin registry is
/// constructed once at process start and shared across threads.
///
/// ## Example
///
/// ```rust
/// use slideforge_plugin_api::{DataSource, DataSourceError, DataSourceOptions};
/// use slideforge_types::Value;
///
/// struct NullSource;
///
/// impl DataSource for NullSource {
///     fn id(&self) -> &str { "null" }
///     fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
///         Ok(Value::Null)
///     }
/// }
/// ```
pub trait DataSource: Send + Sync {
    /// A unique identifier for this data source plugin.
    ///
    /// The identifier is used to look up the plugin in the [`crate::PluginRegistry`]
    /// and to map `@data` URIs to the correct implementation. Convention:
    /// lowercase ASCII with hyphens (e.g., `"json"`, `"csv"`, `"http"`).
    fn id(&self) -> &str;

    /// Load data from `uri` and return the result as a [`Value`].
    ///
    /// The `uri` is the raw string from the `@data` directive in the `.sf`
    /// source file. Implementations should parse the URI, fetch the data,
    /// and return the resolved value.
    ///
    /// When `uri` is empty, implementations should use their own internally
    /// configured address (e.g., the URL or path baked in at construction time).
    ///
    /// # Errors
    ///
    /// Returns [`DataSourceError`] when the URI cannot be resolved, the data
    /// cannot be parsed, or an I/O error occurs.
    fn load(&self, uri: &str, opts: &DataSourceOptions) -> Result<Value, DataSourceError>;

    /// Returns `true` if this source would make a network request that the
    /// `--offline` flag should suppress.
    ///
    /// The dispatcher calls this method to determine whether to skip a source
    /// when `DataSourceContext::offline` is `true`. Sources that read from the
    /// local filesystem (JSON, CSV, YAML, TOML, XLSX, `SQLite`) return `false`
    /// (the default) and are always loaded. Sources that require network access
    /// (HTTP/HTTPS) return `true` and are silently skipped in offline mode.
    ///
    /// Third-party plugin authors: override this method and return `true` for
    /// any source that would open a TCP connection or otherwise depend on
    /// network availability.
    ///
    /// ## Coordination protocol
    ///
    /// This method is the canonical offline-capability declaration for the
    /// plugin-first architecture. The dispatcher uses `supports_offline()` as
    /// its sole gate — no out-of-band boolean flags are accepted in the sources
    /// slice. This ensures that a third-party `DataSource` implementation can
    /// participate in offline-mode coordination without any changes to the
    /// dispatcher or the calling code.
    ///
    /// Traces to BC-1.03.004 AC-012 + Architecture Compliance Rules.
    #[must_use]
    fn supports_offline(&self) -> bool {
        false
    }
}

#[cfg(test)]
#[allow(clippy::unnecessary_literal_bound)]
mod tests {
    use super::*;

    /// Compile-time assertion: `dyn DataSource` is `Send + Sync`.
    fn assert_send_sync<T: Send + Sync + ?Sized>() {}

    #[test]
    fn test_bc_5_02_001_data_source_trait_is_send_sync() {
        assert_send_sync::<dyn DataSource>();
    }

    #[test]
    fn test_bc_5_02_001_data_source_options_default() {
        let opts = DataSourceOptions::default();
        assert!(opts.timeout_ms.is_none());
        assert!(opts.auth_token.is_none());
        assert!(opts.query.is_none());
    }

    #[test]
    fn test_bc_5_02_001_data_source_error_unsupported_uri() {
        let err = DataSourceError::UnsupportedUri {
            uri: "unknown://foo".to_owned(),
        };
        assert!(err.to_string().contains("unsupported"));
    }

    #[test]
    fn test_bc_5_02_001_data_source_error_parse_error() {
        let err = DataSourceError::ParseError {
            uri: "data.json".to_owned(),
            message: "invalid JSON".to_owned(),
        };
        assert!(err.to_string().contains("parse error"));
    }

    #[test]
    fn test_bc_5_02_001_data_source_error_io_error() {
        let err = DataSourceError::IoError {
            uri: "data.csv".to_owned(),
            message: "file not found".to_owned(),
        };
        assert!(err.to_string().contains("I/O error"));
    }

    #[test]
    fn test_bc_5_02_001_data_source_error_auth_error() {
        let err = DataSourceError::AuthError {
            uri: "https://api.example.com".to_owned(),
        };
        assert!(err.to_string().contains("authentication failed"));
    }

    struct StubDataSource;

    impl DataSource for StubDataSource {
        fn id(&self) -> &str {
            "stub"
        }

        fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
            Ok(Value::Null)
        }
    }

    #[test]
    fn test_bc_5_02_001_data_source_id_method() {
        let src = StubDataSource;
        assert_eq!(src.id(), "stub");
    }

    #[test]
    fn test_bc_5_02_001_data_source_load_returns_ok() {
        let src = StubDataSource;
        let opts = DataSourceOptions::default();
        let result = src.load("test://stub", &opts);
        assert!(result.is_ok());
    }
}
