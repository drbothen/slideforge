//! Error types for the slideforge-data crate.
//!
//! All errors carry the error code constants defined by the project error
//! taxonomy (E-DAT-NNN series).

use thiserror::Error;

/// Error code for file-not-found conditions.
///
/// Maps to `E-DAT-001` in the error taxonomy.
pub const E_DAT_001: &str = "E-DAT-001";

/// Error code for parse failures (malformed input).
///
/// Maps to `E-DAT-003` in the error taxonomy.
pub const E_DAT_003: &str = "E-DAT-003";

/// Error code for field-not-found lookups.
///
/// Maps to `E-DAT-004` in the error taxonomy.
pub const E_DAT_004: &str = "E-DAT-004";

/// Error code for unsupported file formats.
///
/// Maps to `E-DAT-005` in the error taxonomy.
pub const E_DAT_005: &str = "E-DAT-005";

/// Error code for network/SSRF-related errors (reserved for HTTP source).
///
/// Maps to `E-DAT-006` in the error taxonomy.
pub const E_DAT_006: &str = "E-DAT-006";

/// The top-level error type for all `slideforge-data` operations.
///
/// Each variant corresponds to a documented error code in the error taxonomy.
#[derive(Debug, Error)]
pub enum DataError {
    /// The specified file path does not exist or is not accessible.
    ///
    /// Error code: `E-DAT-001`.
    #[error("[{code}] file not found: {path}")]
    FileNotFound {
        /// The error code constant (`E-DAT-001`).
        code: &'static str,
        /// The path that was not found.
        path: String,
    },

    /// The file content could not be parsed as the expected format.
    ///
    /// Error code: `E-DAT-003`.
    #[error("[{code}] parse error for '{path}': {message}")]
    ParseError {
        /// The error code constant (`E-DAT-003`).
        code: &'static str,
        /// The path of the file that failed to parse.
        path: String,
        /// Human-readable description of the parse failure.
        message: String,
    },

    /// A field lookup returned no result.
    ///
    /// Error code: `E-DAT-004`.
    #[error("[{code}] field not found: {field}")]
    FieldNotFound {
        /// The error code constant (`E-DAT-004`).
        code: &'static str,
        /// The field name that was not found.
        field: String,
    },

    /// The file extension is not supported by any registered parser.
    ///
    /// Error code: `E-DAT-005`.
    #[error("[{code}] unsupported format: {extension}")]
    UnsupportedFormat {
        /// The error code constant (`E-DAT-005`).
        code: &'static str,
        /// The file extension that was not recognized.
        extension: String,
    },

    /// An SSRF-blocked or network-level error (reserved for HTTP source).
    ///
    /// Error code: `E-DAT-006`.
    #[error("[{code}] network error or SSRF blocked: {message}")]
    NetworkError {
        /// The error code constant (`E-DAT-006`).
        code: &'static str,
        /// Description of the network failure.
        message: String,
    },

    /// An SSRF attempt was blocked by the security policy.
    ///
    /// Error code: `E-DAT-006` (sub-case of network error).
    #[error("[{code}] SSRF blocked: {uri}")]
    SsrfBlocked {
        /// The error code constant (`E-DAT-006`).
        code: &'static str,
        /// The URI that was blocked.
        uri: String,
    },
}

impl DataError {
    /// Construct a [`DataError::FileNotFound`] with the canonical error code.
    #[must_use]
    pub fn file_not_found(path: impl Into<String>) -> Self {
        DataError::FileNotFound {
            code: E_DAT_001,
            path: path.into(),
        }
    }

    /// Construct a [`DataError::ParseError`] with the canonical error code.
    #[must_use]
    pub fn parse_error(path: impl Into<String>, message: impl Into<String>) -> Self {
        DataError::ParseError {
            code: E_DAT_003,
            path: path.into(),
            message: message.into(),
        }
    }

    /// Construct a [`DataError::FieldNotFound`] with the canonical error code.
    #[must_use]
    pub fn field_not_found(field: impl Into<String>) -> Self {
        DataError::FieldNotFound {
            code: E_DAT_004,
            field: field.into(),
        }
    }

    /// Construct a [`DataError::UnsupportedFormat`] with the canonical error code.
    #[must_use]
    pub fn unsupported_format(extension: impl Into<String>) -> Self {
        DataError::UnsupportedFormat {
            code: E_DAT_005,
            extension: extension.into(),
        }
    }

    /// Return the error code string for this error variant.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            DataError::FileNotFound { .. } => E_DAT_001,
            DataError::ParseError { .. } => E_DAT_003,
            DataError::FieldNotFound { .. } => E_DAT_004,
            DataError::UnsupportedFormat { .. } => E_DAT_005,
            DataError::NetworkError { .. } | DataError::SsrfBlocked { .. } => E_DAT_006,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// test_BC_5_03_001_error_code_file_not_found — E-DAT-001 constant is correct.
    #[test]
    fn test_bc_5_03_001_error_code_file_not_found() {
        assert_eq!(E_DAT_001, "E-DAT-001");
        let err = DataError::file_not_found("/tmp/missing.json");
        assert_eq!(err.code(), "E-DAT-001");
        assert!(err.to_string().contains("E-DAT-001"));
        assert!(err.to_string().contains("missing.json"));
    }

    /// test_BC_5_03_001_error_code_parse_error — E-DAT-003 constant is correct.
    #[test]
    fn test_bc_5_03_001_error_code_parse_error() {
        assert_eq!(E_DAT_003, "E-DAT-003");
        let err = DataError::parse_error("data.json", "unexpected token");
        assert_eq!(err.code(), "E-DAT-003");
        assert!(err.to_string().contains("E-DAT-003"));
        assert!(err.to_string().contains("unexpected token"));
    }

    /// test_BC_5_03_001_error_code_field_not_found — E-DAT-004 constant is correct.
    #[test]
    fn test_bc_5_03_001_error_code_field_not_found() {
        assert_eq!(E_DAT_004, "E-DAT-004");
        let err = DataError::field_not_found("revenue");
        assert_eq!(err.code(), "E-DAT-004");
        assert!(err.to_string().contains("E-DAT-004"));
        assert!(err.to_string().contains("revenue"));
    }

    /// test_BC_5_03_001_error_code_unsupported_format — E-DAT-005 constant is correct.
    #[test]
    fn test_bc_5_03_001_error_code_unsupported_format() {
        assert_eq!(E_DAT_005, "E-DAT-005");
        let err = DataError::unsupported_format(".txt");
        assert_eq!(err.code(), "E-DAT-005");
        assert!(err.to_string().contains("E-DAT-005"));
        assert!(err.to_string().contains(".txt"));
    }

    /// test_BC_5_03_001_error_code_network_error — E-DAT-006 constant is correct.
    #[test]
    fn test_bc_5_03_001_error_code_network_error() {
        assert_eq!(E_DAT_006, "E-DAT-006");
        let err = DataError::NetworkError {
            code: E_DAT_006,
            message: "connection refused".to_owned(),
        };
        assert_eq!(err.code(), "E-DAT-006");
        assert!(err.to_string().contains("E-DAT-006"));
    }

    /// test_BC_5_03_001_error_code_ssrf_blocked — SsrfBlocked uses E-DAT-006.
    #[test]
    fn test_bc_5_03_001_error_code_ssrf_blocked() {
        let err = DataError::SsrfBlocked {
            code: E_DAT_006,
            uri: "http://169.254.169.254".to_owned(),
        };
        assert_eq!(err.code(), "E-DAT-006");
        assert!(err.to_string().contains("E-DAT-006"));
        assert!(err.to_string().contains("169.254.169.254"));
    }
}
