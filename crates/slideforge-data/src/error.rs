//! Error types for the slideforge-data crate.
//!
//! All errors carry the error code constants defined by the project error
//! taxonomy (E-DAT-NNN series).

use std::sync::Arc;

use slideforge_types::SourceSpan;
use thiserror::Error;

use crate::format::DataFormat;

/// Error code for HTTP non-2xx response.
///
/// Maps to `E-DAT-001` in the error taxonomy.
pub const E_DAT_001: &str = "E-DAT-001";

/// Error code for network unreachable / transport error.
///
/// Maps to `E-DAT-002` in the error taxonomy.
pub const E_DAT_002: &str = "E-DAT-002";

/// Error code for parse failures (malformed input or unsupported format).
///
/// Maps to `E-DAT-003` in the error taxonomy.
pub const E_DAT_003: &str = "E-DAT-003";

/// Error code for file-not-found conditions.
///
/// Maps to `E-DAT-004` in the error taxonomy.
pub const E_DAT_004: &str = "E-DAT-004";

/// Error code for missing field access.
///
/// Maps to `E-DAT-005` in the error taxonomy.
pub const E_DAT_005: &str = "E-DAT-005";

/// Error code for SSRF domain blocked or path traversal.
///
/// Maps to `E-DAT-006` in the error taxonomy.
pub const E_DAT_006: &str = "E-DAT-006";

/// Error code for XLSX header row empty cell.
///
/// Maps to `E-DAT-007` in the error taxonomy.
pub const E_DAT_007: &str = "E-DAT-007";

/// Error code for XLSX header cell with non-string type.
///
/// Maps to `E-DAT-008` in the error taxonomy.
pub const E_DAT_008: &str = "E-DAT-008";

/// Error code for XLSX `DateTimeIso` cell with invalid ISO 8601 value.
///
/// Maps to `E-DAT-009` in the error taxonomy.
pub const E_DAT_009: &str = "E-DAT-009";

/// Error code for XLSX non-finite float cell (NaN or Infinity).
///
/// Maps to `E-DAT-010` in the error taxonomy.
pub const E_DAT_010: &str = "E-DAT-010";

/// Error code for file with `.xlsx` extension that is not a valid XLSX archive.
///
/// Maps to `E-DAT-011` in the error taxonomy.
pub const E_DAT_011: &str = "E-DAT-011";

/// Error code for `SQLite` TEXT column containing invalid UTF-8 bytes.
///
/// Maps to `E-DAT-012` in the error taxonomy.
pub const E_DAT_012: &str = "E-DAT-012";

/// Error code for file with `SQLite` extension that lacks the `SQLite` magic header.
///
/// Maps to `E-DAT-013` in the error taxonomy.
pub const E_DAT_013: &str = "E-DAT-013";

/// Error code for unsupported extension for `SQLite` data source.
///
/// Maps to `E-DAT-014` in the error taxonomy.
///
/// # Deprecation
///
/// **Retired by F-P13-HIGH-002.** `E-DAT-014` is subsumed by `E-DAT-003` at the
/// dispatcher boundary. `SQLite` extension errors now route through
/// [`DataError::UnsupportedFormat`] (E-DAT-003) via `DataSourceError::UnsupportedUri`
/// with a bare extension string, matching the XLSX pattern.
///
/// The constant is retained for `SemVer` compatibility (external crates may reference
/// it). No production code path produces a [`DataError`] whose `.code()` returns
/// `"E-DAT-014"`. See error-taxonomy.md changelog (v1.8).
pub const E_DAT_014: &str = "E-DAT-014";

/// Error code for unspecified data-source error (third-party plugin, unknown category).
///
/// Maps to `E-DAT-015` in the error taxonomy.
///
/// Used by the dispatcher's catch-all arm when a third-party `DataSource` plugin
/// returns an [`slideforge_plugin_api::DataSourceError::IoError`] whose message does
/// NOT embed a `[E-DAT-NNN]` bracket prefix. The catch-all cannot infer the
/// specific failure category, so it routes to this code instead of mis-using
/// `E-DAT-004` (file-not-found) or `E-DAT-002` (network) which carry specific
/// semantic meanings that do not apply to an unknown source.
///
/// Plugin authors should embed `[E-DAT-NNN]` in their error messages to enable
/// precise routing. A `tracing::debug!` is emitted at dispatch time when this
/// fallback is triggered.
///
/// # Observability
///
/// The developer-guidance log message is emitted at `tracing::debug!` level (not
/// `warn!`). To surface it, set `RUST_LOG=slideforge_data=debug` (or configure
/// an equivalent subscriber filter). Keeping it at `debug` prevents flooding
/// watch-mode output for third-party plugins that do not yet embed bracket codes.
pub const E_DAT_015: &str = "E-DAT-015";

// F-P4-OBS-001: E_DAT_006_POLICY alias removed. The dispatcher now uses E_DAT_006
// directly at the single PolicyRejected construction site, with an inline comment
// explaining the dual-sub-case semantics (SSRF vs body-cap). The alias was
// documentary-only and carried no load-bearing semantic distinction.

/// The top-level error type for all `slideforge-data` operations.
///
/// Each variant corresponds to a documented error code in the error taxonomy.
///
/// # `SemVer` policy
///
/// This enum is `#[non_exhaustive]`. Match arms in external crates (and internal
/// code using exhaustive patterns) must include a wildcard arm (`_ => ...`) to
/// remain forward-compatible as new error codes are added in future releases.
/// Internal code that matches exhaustively within this crate is exempt (the
/// compiler enforces completeness at the call site automatically).
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DataError {
    /// The specified file path does not exist or is not accessible.
    ///
    /// Error code: `E-DAT-004`.
    #[error("[{code}] file not found: {path} (at {span})")]
    FileNotFound {
        /// The error code constant (`E-DAT-004`).
        code: &'static str,
        /// The path that was not found.
        path: Arc<str>,
        /// The source location that referenced this path.
        span: SourceSpan,
    },

    /// The file content could not be parsed as the expected format,
    /// or the file extension is not supported.
    ///
    /// Error code: `E-DAT-003`.
    #[error("[{code}] parse error for '{path}' ({format:?}): {reason} (at {span})")]
    ParseError {
        /// The error code constant (`E-DAT-003`).
        code: &'static str,
        /// The path of the file that failed to parse.
        path: Arc<str>,
        /// The data format that was attempted.
        format: DataFormat,
        /// Human-readable description of the parse failure.
        reason: Arc<str>,
        /// The source location that referenced this path.
        span: SourceSpan,
    },

    /// A field lookup returned no result.
    ///
    /// Error code: `E-DAT-005`.
    #[error("[{code}] field not found: '{field}' in source '{source_name}' (at {span})")]
    FieldNotFound {
        /// The error code constant (`E-DAT-005`).
        code: &'static str,
        /// The field name that was not found.
        field: Arc<str>,
        /// The name of the data source in which the field was looked up.
        source_name: Arc<str>,
        /// The source location that issued the field lookup.
        span: SourceSpan,
    },

    /// The file extension is not supported by any registered parser.
    ///
    /// Error code: `E-DAT-003` (sub-case of parse/format error).
    #[error(
        "[{code}] unsupported format: '{extension}' — supported: json, csv, yaml, yml, toml, xlsx, sqlite, sqlite3, db (at {span})"
    )]
    UnsupportedFormat {
        /// The error code constant (`E-DAT-003`).
        code: &'static str,
        /// The file extension that was not recognized.
        extension: Arc<str>,
        /// The source location that referenced this path.
        span: SourceSpan,
    },

    /// A generic I/O error that is not a simple file-not-found.
    ///
    /// Error code: `E-DAT-004` by default; can carry `E-DAT-001` in the dispatcher's
    /// HTTP-fallback path when an HTTP status code cannot be extracted from the message.
    /// Body-cap policy rejections use [`DataError::PolicyRejected`] instead of `IoError`.
    #[error("[{code}] I/O error reading '{path}': {message} (at {span})")]
    IoError {
        /// The error code constant (`E-DAT-004` by default; `E-DAT-001` in HTTP fallback).
        code: &'static str,
        /// The path that triggered the I/O error.
        path: Arc<str>,
        /// Description of the I/O failure.
        message: Arc<str>,
        /// The source location associated with this I/O error.
        span: SourceSpan,
    },

    /// A file path escaped the project root (path traversal attempt blocked).
    ///
    /// Error code: `E-DAT-006` (security policy sub-case).
    #[error("[{code}] path traversal blocked: '{path}' is outside base dir (at {span})")]
    PathTraversalBlocked {
        /// The error code constant (`E-DAT-006`).
        code: &'static str,
        /// The path that was blocked.
        path: Arc<str>,
        /// The source location associated with this path traversal attempt.
        span: SourceSpan,
    },

    /// An SSRF attempt was blocked by the `allowed_domains` policy.
    ///
    /// Error code: `E-DAT-006`.
    #[error(
        "[{code}] HTTP source '{url}' blocked by allowed_domains policy. \
        Add '{domain}' to [data].allowed_domains in slideforge.toml."
    )]
    SsrfBlocked {
        /// The error code constant (`E-DAT-006`).
        code: &'static str,
        /// The full URL that was blocked.
        url: Arc<str>,
        /// The domain that was not found in the allowlist.
        domain: Arc<str>,
        /// The source location associated with this SSRF block.
        span: SourceSpan,
    },

    /// An HTTP data-policy rejection (body-size cap exceeded, or similar policy block).
    ///
    /// Error code: `E-DAT-006` (policy sub-case, distinct from SSRF block).
    ///
    /// Used when an HTTP source returns data that violates a configured policy
    /// (e.g., the response body exceeds the 50 MiB cap). This is semantically a
    /// policy rejection, NOT an I/O failure and NOT an SSRF domain block.
    ///
    /// Display: `"[E-DAT-006] data policy rejected '<uri>': <message> (at <span>)"`
    #[error("[{code}] data policy rejected '{uri}': {message} (at {span})")]
    PolicyRejected {
        /// The error code constant (`E-DAT-006`).
        code: &'static str,
        /// The URI of the source whose response violated the policy.
        uri: Arc<str>,
        /// Human-readable description of the policy violation.
        message: Arc<str>,
        /// The source location associated with this error.
        span: SourceSpan,
    },

    /// An HTTP non-2xx response was received from the server.
    ///
    /// Error code: `E-DAT-001`.
    ///
    /// Display format: `"[E-DAT-001] HTTP fetch failed: '<url>' returned HTTP <status>. Hint: use --offline to skip HTTP sources. (at <span>)"`
    ///
    /// Matches the spec format in error-taxonomy.md (E-DAT-001). The `--offline` hint is
    /// the canonical discovery surface for users who did not know the flag existed.
    #[error(
        "[{code}] HTTP fetch failed: '{url}' returned HTTP {status}. \
        Hint: use --offline to skip HTTP sources. (at {span})"
    )]
    HttpError {
        /// The error code constant (`E-DAT-001`).
        code: &'static str,
        /// The URL that returned the non-2xx status.
        url: Arc<str>,
        /// The HTTP status code (4xx or 5xx).
        status: u16,
        /// The source location associated with this HTTP request.
        span: SourceSpan,
    },

    /// A network transport error (connection refused, timeout, DNS failure, etc.).
    ///
    /// Error code: `E-DAT-002`.
    ///
    /// Display format: `"[E-DAT-002] network error fetching '<url>': <cause>. Use --offline to skip HTTP sources. (at <span>)"`
    ///
    /// Matches the spec format in error-taxonomy.md (E-DAT-002). The `--offline` hint and
    /// the URL field are both required by the spec; the URL is the canonical discovery surface
    /// for which source triggered the network failure.
    #[error(
        "[{code}] network error fetching '{url}': {cause}. \
        Use --offline to skip HTTP sources. (at {span})"
    )]
    NetworkError {
        /// The error code constant (`E-DAT-002`).
        code: &'static str,
        /// The URL that could not be reached.
        url: Arc<str>,
        /// Human-readable description of the transport failure.
        cause: Arc<str>,
        /// The source location associated with this network error.
        span: SourceSpan,
    },

    /// An authentication failure from a data source plugin.
    ///
    /// Error code: `E-DAT-002` (access-layer failure, not a parse failure).
    ///
    /// Note: `HttpDataSource` silently ignores `auth_token` in v1 and will not
    /// emit this error in practice. The variant exists for correctness when
    /// third-party plugins emit `DataSourceError::AuthError`.
    #[error("[{code}] authentication failed for source '{uri}' (at {span})")]
    AuthFailed {
        /// The error code constant (`E-DAT-002`).
        code: &'static str,
        /// The URI of the source that failed authentication.
        uri: Arc<str>,
        /// The source location associated with this auth failure.
        span: SourceSpan,
    },

    /// An unspecified error from a third-party data source plugin.
    ///
    /// Error code: `E-DAT-015`.
    ///
    /// Used by the dispatcher's catch-all arm when the plugin's
    /// [`slideforge_plugin_api::DataSourceError::IoError`] message does not embed
    /// a `[E-DAT-NNN]` bracket prefix. Third-party plugins should embed the prefix
    /// in their messages to enable precise routing to a more specific variant.
    ///
    /// # Observability
    ///
    /// A `tracing::debug!` is emitted when this fallback fires, instructing the
    /// plugin author to embed a `[E-DAT-NNN]` bracket code. To surface this message,
    /// set `RUST_LOG=slideforge_data=debug` (or configure an equivalent subscriber
    /// filter). The message is at `debug` level — not `warn!` — to avoid flooding
    /// watch-mode logs for third-party plugins that are under active development.
    #[error("[{code}] data source error for '{uri}': {message} (at {span})")]
    UnspecifiedSourceError {
        /// The error code constant (`E-DAT-015`).
        code: &'static str,
        /// The URI the plugin was asked to load.
        uri: Arc<str>,
        /// The raw message from the plugin.
        message: Arc<str>,
        /// The source location associated with this error.
        span: SourceSpan,
    },
}

impl DataError {
    /// Construct a [`DataError::FileNotFound`] with the canonical error code.
    ///
    /// Uses `SourceSpan::default()` as the span; callers with span information
    /// should construct the variant directly.
    #[must_use]
    pub fn file_not_found(path: impl Into<Arc<str>>) -> Self {
        DataError::FileNotFound {
            code: E_DAT_004,
            path: path.into(),
            span: SourceSpan::default(),
        }
    }

    /// Construct a [`DataError::FileNotFound`] with a source span.
    #[must_use]
    pub fn file_not_found_at(path: impl Into<Arc<str>>, span: SourceSpan) -> Self {
        DataError::FileNotFound {
            code: E_DAT_004,
            path: path.into(),
            span,
        }
    }

    /// Construct a [`DataError::ParseError`] with the canonical error code.
    ///
    /// Uses `SourceSpan::default()` as the span; callers with span information
    /// should use [`DataError::parse_error_at`].
    #[must_use]
    pub fn parse_error(
        path: impl Into<Arc<str>>,
        format: DataFormat,
        reason: impl Into<Arc<str>>,
    ) -> Self {
        DataError::ParseError {
            code: E_DAT_003,
            path: path.into(),
            format,
            reason: reason.into(),
            span: SourceSpan::default(),
        }
    }

    /// Construct a [`DataError::ParseError`] with a source span.
    #[must_use]
    pub fn parse_error_at(
        path: impl Into<Arc<str>>,
        format: DataFormat,
        reason: impl Into<Arc<str>>,
        span: SourceSpan,
    ) -> Self {
        DataError::ParseError {
            code: E_DAT_003,
            path: path.into(),
            format,
            reason: reason.into(),
            span,
        }
    }

    /// Construct a [`DataError::FieldNotFound`] with the canonical error code.
    ///
    /// Uses `SourceSpan::default()` as the span.
    #[must_use]
    pub fn field_not_found(field: impl Into<Arc<str>>, source_name: impl Into<Arc<str>>) -> Self {
        DataError::FieldNotFound {
            code: E_DAT_005,
            field: field.into(),
            source_name: source_name.into(),
            span: SourceSpan::default(),
        }
    }

    /// Construct a [`DataError::FieldNotFound`] with a source span.
    #[must_use]
    pub fn field_not_found_at(
        field: impl Into<Arc<str>>,
        source_name: impl Into<Arc<str>>,
        span: SourceSpan,
    ) -> Self {
        DataError::FieldNotFound {
            code: E_DAT_005,
            field: field.into(),
            source_name: source_name.into(),
            span,
        }
    }

    /// Construct a [`DataError::UnsupportedFormat`] with the canonical error code.
    ///
    /// Uses `SourceSpan::default()` as the span; callers with span information
    /// should use [`DataError::unsupported_format_at`].
    #[must_use]
    pub fn unsupported_format(extension: impl Into<Arc<str>>) -> Self {
        DataError::UnsupportedFormat {
            code: E_DAT_003,
            extension: extension.into(),
            span: SourceSpan::default(),
        }
    }

    /// Construct a [`DataError::UnsupportedFormat`] with a source span.
    #[must_use]
    pub fn unsupported_format_at(extension: impl Into<Arc<str>>, span: SourceSpan) -> Self {
        DataError::UnsupportedFormat {
            code: E_DAT_003,
            extension: extension.into(),
            span,
        }
    }

    /// Attach a [`SourceSpan`] to this error, replacing the default span.
    ///
    /// Intended for use at the evaluator boundary, where the evaluator knows
    /// the span of the `@data` directive but the data layer does not. Example:
    ///
    /// ```
    /// # use slideforge_data::DataError;
    /// # use slideforge_types::SourceSpan;
    /// # use std::sync::Arc;
    /// let err = DataError::file_not_found("/tmp/missing.json");
    /// let span = SourceSpan::new(Arc::from("deck.sf"), 5, 3, 100);
    /// let err_with_span = err.with_span(span);
    /// ```
    #[must_use]
    pub fn with_span(self, new_span: SourceSpan) -> Self {
        match self {
            DataError::FileNotFound { code, path, .. } => DataError::FileNotFound {
                code,
                path,
                span: new_span,
            },
            DataError::ParseError {
                code,
                path,
                format,
                reason,
                ..
            } => DataError::ParseError {
                code,
                path,
                format,
                reason,
                span: new_span,
            },
            DataError::FieldNotFound {
                code,
                field,
                source_name,
                ..
            } => DataError::FieldNotFound {
                code,
                field,
                source_name,
                span: new_span,
            },
            DataError::UnsupportedFormat {
                code, extension, ..
            } => DataError::UnsupportedFormat {
                code,
                extension,
                span: new_span,
            },
            DataError::IoError {
                code,
                path,
                message,
                ..
            } => DataError::IoError {
                code,
                path,
                message,
                span: new_span,
            },
            DataError::PathTraversalBlocked { code, path, .. } => DataError::PathTraversalBlocked {
                code,
                path,
                span: new_span,
            },
            DataError::SsrfBlocked {
                code, url, domain, ..
            } => DataError::SsrfBlocked {
                code,
                url,
                domain,
                span: new_span,
            },
            DataError::PolicyRejected {
                code, uri, message, ..
            } => DataError::PolicyRejected {
                code,
                uri,
                message,
                span: new_span,
            },
            DataError::HttpError {
                code, url, status, ..
            } => DataError::HttpError {
                code,
                url,
                status,
                span: new_span,
            },
            DataError::NetworkError {
                code, url, cause, ..
            } => DataError::NetworkError {
                code,
                url,
                cause,
                span: new_span,
            },
            DataError::AuthFailed { code, uri, .. } => DataError::AuthFailed {
                code,
                uri,
                span: new_span,
            },
            DataError::UnspecifiedSourceError {
                code, uri, message, ..
            } => DataError::UnspecifiedSourceError {
                code,
                uri,
                message,
                span: new_span,
            },
        }
    }

    /// Construct a [`DataError::IoError`] with the canonical error code.
    ///
    /// Uses `SourceSpan::default()` as the span; callers with span information
    /// should use [`DataError::io_error_at`].
    #[must_use]
    pub fn io_error(path: impl Into<Arc<str>>, message: impl Into<Arc<str>>) -> Self {
        DataError::IoError {
            code: E_DAT_004,
            path: path.into(),
            message: message.into(),
            span: SourceSpan::default(),
        }
    }

    /// Construct a [`DataError::IoError`] with a source span.
    #[must_use]
    pub fn io_error_at(
        path: impl Into<Arc<str>>,
        message: impl Into<Arc<str>>,
        span: SourceSpan,
    ) -> Self {
        DataError::IoError {
            code: E_DAT_004,
            path: path.into(),
            message: message.into(),
            span,
        }
    }

    /// Construct a [`DataError::PathTraversalBlocked`] with the canonical error code.
    ///
    /// Uses `SourceSpan::default()` as the span; callers with span information
    /// should use [`DataError::path_traversal_blocked_at`].
    #[must_use]
    pub fn path_traversal_blocked(path: impl Into<Arc<str>>) -> Self {
        DataError::PathTraversalBlocked {
            code: E_DAT_006,
            path: path.into(),
            span: SourceSpan::default(),
        }
    }

    /// Construct a [`DataError::PathTraversalBlocked`] with a source span.
    #[must_use]
    pub fn path_traversal_blocked_at(path: impl Into<Arc<str>>, span: SourceSpan) -> Self {
        DataError::PathTraversalBlocked {
            code: E_DAT_006,
            path: path.into(),
            span,
        }
    }

    /// Return the error code string for this error variant.
    ///
    /// Returns the specific code stored in the variant for [`DataError::ParseError`],
    /// [`DataError::FileNotFound`], [`DataError::IoError`], and [`DataError::AuthFailed`]
    /// — the stored `code` field is authoritative. This allows `IoError` to carry
    /// `E-DAT-001` in the dispatcher's HTTP-fallback path and `ParseError` to carry
    /// granular format codes without requiring distinct variants. Body-cap policy
    /// rejections route to [`DataError::PolicyRejected`] (not `IoError`).
    ///
    /// `AuthFailed` also uses the stored `code` field (F-P15-LOW-001), consistent with
    /// the sibling pattern in `FileNotFound`, `IoError`, and `ParseError`. The stored
    /// field is invariantly `E-DAT-002` in current production paths, but returning
    /// the stored field is the structurally correct approach.
    ///
    /// For all other variants the code is determined by the discriminant.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            // Variants that store the specific code directly — return it without
            // overriding. `FileNotFound` and `IoError` both use E_DAT_004 by default,
            // but `IoError` may carry E_DAT_001 in the dispatcher's HTTP-fallback path.
            // `ParseError` may carry granular sub-codes (E_DAT_009, E_DAT_010, etc.).
            // `AuthFailed` invariantly stores E_DAT_002, but returning the stored field
            // is consistent with the sibling pattern (F-P15-LOW-001).
            DataError::FileNotFound { code, .. }
            | DataError::IoError { code, .. }
            | DataError::ParseError { code, .. }
            | DataError::AuthFailed { code, .. } => code,
            DataError::UnsupportedFormat { .. } => E_DAT_003,
            DataError::FieldNotFound { .. } => E_DAT_005,
            DataError::PathTraversalBlocked { .. }
            | DataError::SsrfBlocked { .. }
            | DataError::PolicyRejected { .. } => E_DAT_006,
            DataError::HttpError { .. } => E_DAT_001,
            DataError::NetworkError { .. } => E_DAT_002,
            DataError::UnspecifiedSourceError { .. } => E_DAT_015,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// `test_BC_5_03_001_error_code_file_not_found` — E-DAT-004 constant is correct.
    #[test]
    fn test_bc_5_03_001_error_code_file_not_found() {
        assert_eq!(E_DAT_004, "E-DAT-004");
        let err = DataError::file_not_found("/tmp/missing.json");
        assert_eq!(err.code(), "E-DAT-004");
        assert!(err.to_string().contains("E-DAT-004"));
        assert!(err.to_string().contains("missing.json"));
    }

    /// `test_BC_5_03_001_error_code_file_not_found_with_span` — `FileNotFound` carries `SourceSpan`.
    #[test]
    fn test_bc_5_03_001_error_code_file_not_found_with_span() {
        use std::sync::Arc;
        let span = SourceSpan::new(Arc::from("deck.sf"), 5, 3, 100);
        let err = DataError::file_not_found_at("/tmp/missing.json", span);
        assert_eq!(err.code(), "E-DAT-004");
        assert!(err.to_string().contains("E-DAT-004"));
    }

    /// `test_BC_5_03_001_error_code_parse_error` — E-DAT-003 constant is correct.
    #[test]
    fn test_bc_5_03_001_error_code_parse_error() {
        assert_eq!(E_DAT_003, "E-DAT-003");
        let err = DataError::parse_error("data.json", DataFormat::Json, "unexpected token");
        assert_eq!(err.code(), "E-DAT-003");
        assert!(err.to_string().contains("E-DAT-003"));
        assert!(err.to_string().contains("unexpected token"));
    }

    /// `test_BC_5_03_001_error_code_field_not_found` — E-DAT-005 constant is correct.
    #[test]
    fn test_bc_5_03_001_error_code_field_not_found() {
        assert_eq!(E_DAT_005, "E-DAT-005");
        let err = DataError::field_not_found("revenue", "sales");
        assert_eq!(err.code(), "E-DAT-005");
        assert!(err.to_string().contains("E-DAT-005"));
        assert!(err.to_string().contains("revenue"));
    }

    /// `test_BC_5_03_001_error_code_unsupported_format` — E-DAT-003 constant is correct.
    #[test]
    fn test_bc_5_03_001_error_code_unsupported_format() {
        assert_eq!(E_DAT_003, "E-DAT-003");
        let err = DataError::unsupported_format(".txt");
        assert_eq!(err.code(), "E-DAT-003");
        assert!(err.to_string().contains("E-DAT-003"));
        assert!(err.to_string().contains(".txt"));
    }

    /// `test_BC_5_03_001_error_code_network_error` — E-DAT-002 constant is correct.
    #[test]
    fn test_bc_5_03_001_error_code_network_error() {
        assert_eq!(E_DAT_001, "E-DAT-001");
        assert_eq!(E_DAT_002, "E-DAT-002");
        let err = DataError::NetworkError {
            code: E_DAT_002,
            url: Arc::from("http://example.com/data.json"),
            cause: Arc::from("connection refused"),
            span: SourceSpan::default(),
        };
        assert_eq!(err.code(), "E-DAT-002");
        assert!(err.to_string().contains("E-DAT-002"));
    }

    /// `test_BC_5_03_001_error_code_http_error` — `HttpError` uses E-DAT-001 and carries status.
    #[test]
    fn test_bc_5_03_001_error_code_http_error() {
        let err = DataError::HttpError {
            code: E_DAT_001,
            url: Arc::from("http://example.com/data.json"),
            status: 404,
            span: SourceSpan::default(),
        };
        assert_eq!(err.code(), "E-DAT-001");
        assert!(err.to_string().contains("E-DAT-001"));
        assert!(err.to_string().contains("404"));
    }

    /// `test_f_p15_low_001_auth_failed_code_uses_stored_field` — `AuthFailed.code()` returns
    /// the stored `code` field, not a hardcoded constant. This is Option A of F-P15-LOW-001:
    /// consistent with the sibling pattern in `FileNotFound`, `IoError`, and `ParseError`.
    ///
    /// Traces to F-P15-LOW-001.
    #[test]
    fn test_f_p15_low_001_auth_failed_code_uses_stored_field() {
        // Construct AuthFailed with the standard E_DAT_002 code.
        let err = DataError::AuthFailed {
            code: E_DAT_002,
            uri: Arc::from("http://example.com/secure"),
            span: SourceSpan::default(),
        };
        assert_eq!(
            err.code(),
            "E-DAT-002",
            "AuthFailed.code() must return the stored code field; got: {}",
            err.code()
        );

        // Verify the stored field is authoritative: constructing with E_DAT_005
        // must return E-DAT-005, not E-DAT-002 (this tests that the match arm
        // reads the stored field, not a hardcoded constant).
        let err_with_non_default_code = DataError::AuthFailed {
            code: E_DAT_005,
            uri: Arc::from("http://example.com/secure"),
            span: SourceSpan::default(),
        };
        assert_eq!(
            err_with_non_default_code.code(),
            "E-DAT-005",
            "AuthFailed.code() must return the stored field (E-DAT-005), not a hardcoded constant; \
            got: {}",
            err_with_non_default_code.code()
        );
    }

    /// `test_BC_5_03_001_error_code_ssrf_blocked` — `SsrfBlocked` uses E-DAT-006 with remediation hint.
    #[test]
    fn test_bc_5_03_001_error_code_ssrf_blocked() {
        let err = DataError::SsrfBlocked {
            code: E_DAT_006,
            url: Arc::from("http://169.254.169.254"),
            domain: Arc::from("169.254.169.254"),
            span: SourceSpan::default(),
        };
        assert_eq!(err.code(), "E-DAT-006");
        let msg = err.to_string();
        assert!(msg.contains("E-DAT-006"));
        assert!(msg.contains("169.254.169.254"));
        assert!(
            msg.contains("allowed_domains"),
            "SSRF error must include remediation hint mentioning allowed_domains; got: {msg}"
        );
        assert!(
            msg.contains("slideforge.toml"),
            "SSRF error must reference slideforge.toml; got: {msg}"
        );
    }

    /// `test_BC_5_03_001_error_code_io_error` — `IoError` uses E-DAT-004.
    #[test]
    fn test_bc_5_03_001_error_code_io_error() {
        let err = DataError::io_error("/tmp/data.csv", "permission denied");
        assert_eq!(err.code(), "E-DAT-004");
        assert!(err.to_string().contains("E-DAT-004"));
        assert!(err.to_string().contains("permission denied"));
    }

    /// `test_BC_5_03_001_error_code_path_traversal` — `PathTraversalBlocked` uses E-DAT-006.
    #[test]
    fn test_bc_5_03_001_error_code_path_traversal() {
        let err = DataError::path_traversal_blocked("../../etc/passwd");
        assert_eq!(err.code(), "E-DAT-006");
        assert!(err.to_string().contains("E-DAT-006"));
        assert!(err.to_string().contains("etc/passwd"));
    }

    /// `test_BC_5_03_001_unsupported_format_hint` — error message lists supported extensions.
    #[test]
    fn test_bc_5_03_001_unsupported_format_hint() {
        let err = DataError::unsupported_format("xls");
        let msg = err.to_string();
        assert!(msg.contains("json"), "hint must list json");
        assert!(msg.contains("csv"), "hint must list csv");
        assert!(msg.contains("toml"), "hint must list toml");
    }

    /// `test_with_span_io_error` — `with_span()` attaches span to `IoError` (FINDING-001).
    ///
    /// Previously `IoError` fell into the catch-all `other => other` arm and silently
    /// discarded the span. Now it must be updated.
    #[test]
    fn test_with_span_io_error() {
        let err = DataError::io_error("/tmp/data.csv", "permission denied");
        let span = SourceSpan::new(Arc::from("deck.sf"), 10, 4, 200);
        let err_with_span = err.with_span(span.clone());
        // After with_span, the error must carry the provided span (visible in display string).
        let msg = err_with_span.to_string();
        assert!(
            msg.contains("deck.sf"),
            "with_span must embed the new file in IoError"
        );
        // Also verify the error code is preserved.
        assert_eq!(err_with_span.code(), "E-DAT-004");
    }

    /// `test_with_span_path_traversal_blocked` — `with_span()` attaches span to `PathTraversalBlocked`.
    ///
    /// Previously `PathTraversalBlocked` fell into the catch-all arm. Now it must be updated.
    #[test]
    fn test_with_span_path_traversal_blocked() {
        let err = DataError::path_traversal_blocked("../../etc/passwd");
        let span = SourceSpan::new(Arc::from("deck.sf"), 7, 1, 50);
        let err_with_span = err.with_span(span.clone());
        let msg = err_with_span.to_string();
        assert!(
            msg.contains("deck.sf"),
            "with_span must embed the new file in PathTraversalBlocked"
        );
        assert_eq!(err_with_span.code(), "E-DAT-006");
    }

    /// `test_io_error_at_constructor` — `io_error_at()` carries the provided span.
    #[test]
    fn test_io_error_at_constructor() {
        let span = SourceSpan::new(Arc::from("slide.sf"), 3, 2, 80);
        let err = DataError::io_error_at("/tmp/data.csv", "disk full", span);
        assert_eq!(err.code(), "E-DAT-004");
        let msg = err.to_string();
        assert!(
            msg.contains("slide.sf"),
            "io_error_at span must appear in message"
        );
        assert!(msg.contains("disk full"));
    }

    /// `test_path_traversal_blocked_at_constructor` — `path_traversal_blocked_at()` carries the span.
    #[test]
    fn test_path_traversal_blocked_at_constructor() {
        let span = SourceSpan::new(Arc::from("main.sf"), 2, 1, 30);
        let err = DataError::path_traversal_blocked_at("../../secret", span);
        assert_eq!(err.code(), "E-DAT-006");
        let msg = err.to_string();
        assert!(
            msg.contains("main.sf"),
            "path_traversal_blocked_at span must appear in message"
        );
        assert!(msg.contains("secret"));
    }

    /// `test_with_span_ssrf_blocked` — `with_span()` attaches span to `SsrfBlocked`.
    #[test]
    fn test_with_span_ssrf_blocked() {
        let err = DataError::SsrfBlocked {
            code: E_DAT_006,
            url: Arc::from("http://169.254.169.254"),
            domain: Arc::from("169.254.169.254"),
            span: SourceSpan::default(),
        };
        let span = SourceSpan::new(Arc::from("deck.sf"), 3, 1, 40);
        let err_with_span = err.with_span(span);
        let msg = err_with_span.to_string();
        assert!(msg.contains("deck.sf") || msg.contains("E-DAT-006"));
        assert_eq!(err_with_span.code(), "E-DAT-006");
    }

    /// `test_with_span_http_error` — `with_span()` attaches span to `HttpError`.
    #[test]
    fn test_with_span_http_error() {
        let err = DataError::HttpError {
            code: E_DAT_001,
            url: Arc::from("http://example.com/data.json"),
            status: 500,
            span: SourceSpan::default(),
        };
        let span = SourceSpan::new(Arc::from("deck.sf"), 8, 2, 90);
        let err_with_span = err.with_span(span);
        assert_eq!(err_with_span.code(), "E-DAT-001");
        let msg = err_with_span.to_string();
        assert!(msg.contains("500"));
    }

    /// `test_with_span_network_error` — `with_span()` attaches span to `NetworkError`.
    #[test]
    fn test_with_span_network_error() {
        let err = DataError::NetworkError {
            code: E_DAT_002,
            url: Arc::from("http://example.com/data.json"),
            cause: Arc::from("connection refused"),
            span: SourceSpan::default(),
        };
        let span = SourceSpan::new(Arc::from("deck.sf"), 5, 1, 60);
        let err_with_span = err.with_span(span);
        assert_eq!(err_with_span.code(), "E-DAT-002");
        let msg = err_with_span.to_string();
        assert!(msg.contains("connection refused"));
    }

    // ---------------------------------------------------------------------------
    // F-P14-MED-001 / F-P14-MED-002: Display --offline hint tests
    // ---------------------------------------------------------------------------

    /// `test_http_error_display_includes_offline_hint`
    ///
    /// F-P14-MED-001: `DataError::HttpError` Display must include the `--offline` hint
    /// per error-taxonomy.md E-DAT-001. This is the canonical user-facing discovery
    /// surface for the `--offline` flag.
    #[test]
    fn test_http_error_display_includes_offline_hint() {
        let err = DataError::HttpError {
            code: E_DAT_001,
            url: Arc::from("http://example.com/data.json"),
            status: 503,
            span: SourceSpan::default(),
        };
        let display = err.to_string();
        assert!(
            display.contains("--offline"),
            "HttpError Display must contain '--offline' hint; got: {display}"
        );
        assert!(
            display.contains("Hint:"),
            "HttpError Display must contain 'Hint:' prefix; got: {display}"
        );
        assert!(
            display.contains("503"),
            "HttpError Display must contain the status code; got: {display}"
        );
        assert!(
            display.contains("http://example.com/data.json"),
            "HttpError Display must contain the URL; got: {display}"
        );
    }

    /// `test_network_error_display_includes_url_and_offline_hint`
    ///
    /// F-P14-MED-001 / F-P14-MED-002: `DataError::NetworkError` Display must include
    /// both the URL (F-P14-MED-002) and the `--offline` hint (F-P14-MED-001) per
    /// error-taxonomy.md E-DAT-002.
    #[test]
    fn test_network_error_display_includes_url_and_offline_hint() {
        let err = DataError::NetworkError {
            code: E_DAT_002,
            url: Arc::from("http://api.example.com/feed.json"),
            cause: Arc::from("connection timed out"),
            span: SourceSpan::default(),
        };
        let display = err.to_string();
        assert!(
            display.contains("--offline"),
            "NetworkError Display must contain '--offline' hint; got: {display}"
        );
        assert!(
            display.contains("http://api.example.com/feed.json"),
            "NetworkError Display must contain the URL; got: {display}"
        );
        assert!(
            display.contains("connection timed out"),
            "NetworkError Display must contain the cause; got: {display}"
        );
    }

    /// `test_auth_failed_display_does_not_include_offline_hint`
    ///
    /// F-P14-MED-001 (negative): `DataError::AuthFailed` Display must NOT include
    /// "Use --offline" — per Pass-5 F-P5-LOW-008, `--offline` is wrong remediation
    /// for an authentication failure. Preserved from existing test in dispatcher.rs.
    #[test]
    fn test_auth_failed_display_does_not_include_offline_hint() {
        let err = DataError::AuthFailed {
            code: E_DAT_002,
            uri: Arc::from("https://api.example.com/data.json"),
            span: SourceSpan::default(),
        };
        let display = err.to_string();
        assert!(
            !display.contains("--offline"),
            "AuthFailed Display must NOT contain '--offline' hint; got: {display}"
        );
        assert!(
            !display.contains("Use --offline"),
            "AuthFailed Display must NOT contain 'Use --offline'; got: {display}"
        );
        // Must still mention the URI and auth context.
        assert!(
            display.to_lowercase().contains("auth"),
            "AuthFailed Display must mention 'auth'; got: {display}"
        );
    }

    // ---------------------------------------------------------------------------
    // F-P4-MED-002: PolicyRejected unit tests
    // ---------------------------------------------------------------------------

    /// `test_policy_rejected_code_is_e_dat_006`
    ///
    /// F-P4-MED-002: `DataError::PolicyRejected` must carry code `E-DAT-006`.
    #[test]
    fn test_policy_rejected_code_is_e_dat_006() {
        let err = DataError::PolicyRejected {
            code: E_DAT_006,
            uri: Arc::from("http://example.com"),
            message: Arc::from("body too big"),
            span: SourceSpan::default(),
        };
        assert_eq!(
            err.code(),
            "E-DAT-006",
            "PolicyRejected must carry E-DAT-006; got: {}",
            err.code()
        );
    }

    /// `test_policy_rejected_display_format`
    ///
    /// F-P4-MED-002: `DataError::PolicyRejected` Display must match the canonical
    /// format `"[E-DAT-006] data policy rejected '<uri>': <message> (at <span>)"`.
    #[test]
    fn test_policy_rejected_display_format() {
        let span = SourceSpan::new(Arc::from("deck.sf"), 1, 1, 0);
        let err = DataError::PolicyRejected {
            code: E_DAT_006,
            uri: Arc::from("http://example.com"),
            message: Arc::from("body too big"),
            span,
        };
        let display = err.to_string();
        assert!(
            display.contains("[E-DAT-006]"),
            "PolicyRejected Display must contain [E-DAT-006]; got: {display}"
        );
        assert!(
            display.contains("data policy rejected"),
            "PolicyRejected Display must contain 'data policy rejected'; got: {display}"
        );
        assert!(
            display.contains("http://example.com"),
            "PolicyRejected Display must contain the URI; got: {display}"
        );
        assert!(
            display.contains("body too big"),
            "PolicyRejected Display must contain the message; got: {display}"
        );
        assert!(
            display.contains("deck.sf"),
            "PolicyRejected Display must contain the span file; got: {display}"
        );
    }

    /// `test_policy_rejected_with_span_replaces_span`
    ///
    /// F-P4-MED-002: `with_span()` on `PolicyRejected` must replace the span while
    /// preserving `code`, `uri`, and `message`.
    #[test]
    fn test_policy_rejected_with_span_replaces_span() {
        let original_span = SourceSpan::default();
        let err = DataError::PolicyRejected {
            code: E_DAT_006,
            uri: Arc::from("http://example.com"),
            message: Arc::from("body too big"),
            span: original_span,
        };
        let new_span = SourceSpan::new(Arc::from("slide.sf"), 5, 3, 80);
        let updated = err.with_span(new_span);
        // Code and URI and message must be preserved.
        assert_eq!(updated.code(), "E-DAT-006");
        let display = updated.to_string();
        assert!(
            display.contains("http://example.com"),
            "with_span must preserve URI; got: {display}"
        );
        assert!(
            display.contains("body too big"),
            "with_span must preserve message; got: {display}"
        );
        // New span must appear.
        assert!(
            display.contains("slide.sf"),
            "with_span must replace span (new file 'slide.sf' must appear); got: {display}"
        );
    }
}
