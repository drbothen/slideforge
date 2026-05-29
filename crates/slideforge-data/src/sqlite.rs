//! `SQLite` [`DataSource`] implementation.
//!
//! [`SqliteDataSource`] implements the [`slideforge_plugin_api::DataSource`] trait
//! for `SQLite` databases using [`rusqlite`] with the `bundled` feature flag.
//!
//! ## Security: read-only connections
//!
//! The database is always opened with
//! [`rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY`]. This is a hard security
//! requirement from BC-1.03.007 invariant 2. DML statements (INSERT, UPDATE,
//! DELETE) submitted to a read-only connection will be rejected at the
//! driver/OS level, providing defense-in-depth beyond the parser-level
//! SELECT-only enforcement.
//!
//! ## Query requirements
//!
//! The `query` field is required. The DSL parser enforces that the query field
//! is present and is a SELECT statement. As a defense-in-depth measure, if a
//! non-SELECT query somehow reaches `SqliteDataSource::load()`, the read-only
//! connection causes it to fail.
//!
//! ## Row mapping
//!
//! Result rows become [`slideforge_types::Value::Map`] entries keyed by column
//! name (or alias if aliased in SELECT). Duplicate column names in the result
//! set produce [`crate::DataError::ParseError`].
//!
//! ## Bundled `SQLite`
//!
//! `rusqlite` is compiled with `features = ["bundled"]`, shipping the `SQLite`
//! library inside the binary. This eliminates the runtime dependency on
//! `libsqlite3` and ensures reproducible builds across all target platforms
//! (macOS, Linux, Windows) per the supply-chain policy.
//!
//! ## Error codes
//!
//! | Condition | Error |
//! |-----------|-------|
//! | File does not exist | `E-DAT-004` ([`crate::DataError::FileNotFound`]) |
//! | File is not a valid database | `E-DAT-003` ([`crate::DataError::ParseError`]) |
//! | Non-SELECT DML query (defense-in-depth) | `E-DAT-003` ([`crate::DataError::ParseError`]) |
//! | Non-existent table | `E-DAT-003` ([`crate::DataError::ParseError`]) |
//! | Duplicate column names in SELECT | `E-DAT-003` ([`crate::DataError::ParseError`]) |
//! | Zero-row result | `Value::List(vec![])` -- no error |
//!
//! Traces to BC-1.03.007.

use std::sync::Arc;

use rusqlite::types::ValueRef;
use slideforge_plugin_api::{DataSource, DataSourceError, DataSourceOptions};
use slideforge_types::Value;

use crate::DataError;

/// The built-in `SQLite` data source plugin.
///
/// Opens a `SQLite` database in read-only mode and executes a SELECT query,
/// returning the result set as `Value::List(Vec<Value::Map>)`.
/// The plugin identifier is `"sqlite"`.
///
/// ## Usage in the DSL
///
/// ```text
/// @data kpis from "data/metrics.db" query: "SELECT name, value FROM kpis"
/// @data report from "data/app.sqlite3" query: "SELECT * FROM monthly_report"
/// ```
///
/// Traces to BC-1.03.007.
#[derive(Debug, Clone)]
pub struct SqliteDataSource {
    /// The path to the `SQLite` database file.
    ///
    /// Use `":memory:"` for in-memory databases (useful in tests).
    pub path: Arc<str>,

    /// The SELECT query to execute against the database.
    ///
    /// This field is required. The DSL parser ensures that a `query:` field
    /// is always present before `SqliteDataSource::load()` is called. At the
    /// data-layer, an empty `query` string is treated as a defensive error.
    ///
    /// Traces to BC-1.03.007 invariant 4.
    pub query: Arc<str>,
}

impl SqliteDataSource {
    /// Construct a new [`SqliteDataSource`] for the given database path and query.
    #[must_use]
    pub fn new(path: impl Into<Arc<str>>, query: impl Into<Arc<str>>) -> Self {
        SqliteDataSource {
            path: path.into(),
            query: query.into(),
        }
    }
}

impl DataSource for SqliteDataSource {
    fn id(&self) -> &'static str {
        "sqlite"
    }

    /// Open the `SQLite` database in read-only mode and execute the query.
    ///
    /// # Steps
    ///
    /// 1. Open the database with `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX`.
    /// 2. Prepare the query statement.
    /// 3. Extract column names; check for duplicates.
    /// 4. Step through the result set; convert each row to `Value::Map`.
    /// 5. Return `Value::List(rows)`.
    ///
    /// Zero-row results return `Value::List(vec![])` without error (AC-012).
    ///
    /// # Errors
    ///
    /// Returns [`DataSourceError`] on file-not-found, corrupt database,
    /// DML query (defense-in-depth), non-existent table, or duplicate column names.
    ///
    /// Traces to BC-1.03.007 postconditions 1-4.
    fn load(&self, uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        let path_str: &str = if uri.is_empty() {
            self.path.as_ref()
        } else {
            uri
        };
        todo!(
            "BC-1.03.007: open with SQLITE_OPEN_READ_ONLY, execute query, \
            build Value::List(Vec<Value::Map>); path={path_str:?}, query={:?}",
            self.query
        )
    }
}

/// Convert a single [`rusqlite::types::ValueRef`] to a [`slideforge_types::Value`].
///
/// Mapping (BC-1.03.007 postconditions 2, 3, 4):
/// - `Null` => `Value::Null`
/// - `Integer(n)` => `Value::Int(n)`
/// - `Real(f)` => `Value::Float(OrderedFloat(f))`
/// - `Text(bytes)` => `Value::Str(Arc<str>)` (UTF-8 decoded; invalid bytes replaced with U+FFFD)
/// - `Blob(bytes)` => `Value::Str(Arc<str>)` (base64-encoded via `base64::engine::general_purpose::STANDARD`)
///
/// Traces to BC-1.03.007 postconditions 2, 3, 4.
#[allow(dead_code)] // Implementer will call this from DataSource::load
fn convert_rusqlite_value(_val: ValueRef<'_>) -> Value {
    todo!(
        "BC-1.03.007: map rusqlite::types::ValueRef variant to the correct Value variant \
        (Null=>Null, Integer=>Int, Real=>Float, Text=>Str, Blob=>Str base64)"
    )
}

/// Check column names for duplicates; return the first duplicate found.
///
/// Called after `stmt.column_names()` to enforce BC-1.03.007 edge case EC-006 (AC-013).
/// If a duplicate is found, the caller returns `DataError::ParseError` with
/// `E-DAT-003: SELECT result has duplicate column name '<name>'; use aliases`.
///
/// Returns `None` if all names are unique.
///
/// Traces to BC-1.03.007 edge case EC-006.
#[allow(dead_code)] // Implementer will call this from DataSource::load
fn find_duplicate_column<'a>(names: &[&'a str]) -> Option<&'a str> {
    todo!(
        "BC-1.03.007 AC-013: scan column name list for duplicates; \
        return the first duplicate name or None if all unique; names={names:?}"
    )
}

/// Convert a [`DataError`] from the `SQLite` layer to a [`DataSourceError`].
///
/// Mapping:
/// - `FileNotFound` => `IoError` (file missing is an I/O condition)
/// - Everything else => `ParseError`
#[allow(dead_code)] // Implementer will call this from DataSource::load
fn data_error_to_source_error(path: &str, err: &DataError) -> DataSourceError {
    match err {
        DataError::FileNotFound { .. } => DataSourceError::IoError {
            uri: path.to_owned(),
            message: err.to_string(),
        },
        _ => DataSourceError::ParseError {
            uri: path.to_owned(),
            message: err.to_string(),
        },
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    /// `test_BC_1_03_007_sqlite_source_id` -- plugin ID is `"sqlite"`.
    ///
    /// Traces to BC-1.03.007 AC-008.
    #[test]
    fn test_bc_1_03_007_sqlite_source_id() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_007_sqlite_happy_path` -- in-memory db, 3 rows returns `Value::List` of 3 maps.
    ///
    /// Setup: in-memory db, table `metrics(name TEXT, val INTEGER)`, 3 rows.
    /// Expect: `Value::List` of 3 `Value::Map` entries with correct keys/values.
    ///
    /// Traces to BC-1.03.007 AC-008, AC-009.
    #[test]
    fn test_bc_1_03_007_sqlite_happy_path() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_007_sqlite_null_column` -- NULL column value becomes `Value::Null`.
    ///
    /// Traces to BC-1.03.007 AC-009.
    #[test]
    fn test_bc_1_03_007_sqlite_null_column() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_007_sqlite_zero_rows` -- zero-row result returns `Value::List(vec![])`.
    ///
    /// SELECT with `WHERE 1=0` must return an empty list, not an error.
    ///
    /// Traces to BC-1.03.007 AC-012, edge case EC-011.
    #[test]
    fn test_bc_1_03_007_sqlite_zero_rows() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_007_sqlite_readonly_rejects_dml` -- DML query returns `DataError::ParseError`.
    ///
    /// A DELETE/INSERT/UPDATE reaching `load()` is rejected by the read-only
    /// connection flags. Error message must contain "only SELECT queries".
    ///
    /// Traces to BC-1.03.007 AC-011, invariant 5, edge case EC-009.
    #[test]
    fn test_bc_1_03_007_sqlite_readonly_rejects_dml() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_007_sqlite_missing_file` -- non-existent path returns `DataError::FileNotFound`.
    ///
    /// Traces to BC-1.03.007 edge case EC-007.
    #[test]
    fn test_bc_1_03_007_sqlite_missing_file() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_007_sqlite_duplicate_column_names` -- duplicate column returns `DataError::ParseError`.
    ///
    /// `SELECT id, id FROM t` (no alias) must produce an error with "duplicate column name".
    ///
    /// Traces to BC-1.03.007 AC-013, edge case EC-012.
    #[test]
    fn test_bc_1_03_007_sqlite_duplicate_column_names() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_007_sqlite_blob_base64` -- BLOB column becomes `Value::Str` (base64-encoded).
    ///
    /// Traces to BC-1.03.007 AC-009, postcondition 4.
    #[test]
    fn test_bc_1_03_007_sqlite_blob_base64() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_007_sqlite_integer_value` -- INTEGER column becomes `Value::Int(i64)`.
    ///
    /// Traces to BC-1.03.007 postcondition 3.
    #[test]
    fn test_bc_1_03_007_sqlite_integer_value() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_007_sqlite_real_value` -- REAL column becomes `Value::Float(OrderedFloat(f64))`.
    ///
    /// Traces to BC-1.03.007 postcondition 3.
    #[test]
    fn test_bc_1_03_007_sqlite_real_value() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_007_sqlite_text_value` -- TEXT column becomes `Value::Str(Arc<str>)`.
    ///
    /// Traces to BC-1.03.007 postcondition 3.
    #[test]
    fn test_bc_1_03_007_sqlite_text_value() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_007_convert_rusqlite_value_null` -- `ValueRef::Null` becomes `Value::Null`.
    #[test]
    fn test_bc_1_03_007_convert_rusqlite_value_null() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_007_convert_rusqlite_value_integer` -- `ValueRef::Integer` becomes `Value::Int`.
    #[test]
    fn test_bc_1_03_007_convert_rusqlite_value_integer() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_007_convert_rusqlite_value_real` -- `ValueRef::Real` becomes `Value::Float`.
    #[test]
    fn test_bc_1_03_007_convert_rusqlite_value_real() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_007_find_duplicate_column_no_dupes` -- all-unique list returns `None`.
    #[test]
    fn test_bc_1_03_007_find_duplicate_column_no_dupes() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_007_find_duplicate_column_with_dupe` -- duplicate present returns `Some(name)`.
    #[test]
    fn test_bc_1_03_007_find_duplicate_column_with_dupe() {
        panic!("not yet implemented (Red Gate)")
    }
}
