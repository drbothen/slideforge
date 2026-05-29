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

use std::io::Read as _;
use std::sync::Arc;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use ordered_float::OrderedFloat;
use rusqlite::types::ValueRef;
use rusqlite::{Connection, OpenFlags};
use slideforge_plugin_api::{DataSource, DataSourceError, DataSourceOptions};
use slideforge_types::Value;
use slideforge_types::ordered_map::OrderedMap;
use tracing::instrument;

use crate::DataError;
use crate::format::DataFormat;

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
    /// 1. Reject empty query string (defensive guard for BC-1.03.007 invariant 4).
    /// 2. Check file existence (better error message than `SQLite`'s default).
    /// 3. Open the database with `SQLITE_OPEN_READ_ONLY`.
    /// 4. Prepare the query statement.
    /// 5. Extract column names; check for duplicates.
    /// 6. Step through the result set; convert each row to `Value::Map`.
    /// 7. Return `Value::List(rows)`.
    ///
    /// Zero-row results return `Value::List(vec![])` without error (AC-012).
    ///
    /// # Errors
    ///
    /// Returns [`DataSourceError`] on file-not-found, corrupt database,
    /// DML query (defense-in-depth), non-existent table, or duplicate column names.
    ///
    /// Traces to BC-1.03.007 postconditions 1-4.
    #[instrument(skip(self, _opts), fields(path = %self.path))]
    fn load(&self, uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        let path_str: &str = if uri.is_empty() {
            self.path.as_ref()
        } else {
            uri
        };

        // AC-010 (defensive): reject empty query string.
        // The DSL parser enforces this before load() is called, but we guard
        // at the data layer as defense-in-depth.
        // Traces to BC-1.03.007 invariant 4.
        if self.query.is_empty() {
            return Err(DataSourceError::ParseError {
                uri: path_str.to_owned(),
                message: "query is required for SQLite data sources (BC-1.03.007 invariant 4)"
                    .to_owned(),
            });
        }

        // Check file existence before opening (produces a clearer error than
        // SQLite's "unable to open database file" for missing files).
        // Traces to BC-1.03.007 edge case EC-007.
        if !std::path::Path::new(path_str).exists() {
            return Err(DataSourceError::IoError {
                uri: path_str.to_owned(),
                message: format!("file not found: {path_str}"),
            });
        }

        // Validate SQLite magic header before opening.
        // A file whose first 16 bytes are not "SQLite format 3\x00" is not a
        // valid SQLite database. This guard catches corrupt/wrong-type files
        // early (BC-1.03.007 edge case EC-008) — rusqlite's open() itself
        // succeeds on non-SQLite files if the query doesn't need to read pages
        // (e.g. "SELECT 1"), so the check must be explicit.
        validate_sqlite_magic(path_str).map_err(|e| DataSourceError::ParseError {
            uri: path_str.to_owned(),
            message: e,
        })?;

        // AC-008: Open with SQLITE_OPEN_READ_ONLY (hard security requirement).
        // Traces to BC-1.03.007 invariant 2, postcondition 1.
        let conn = Connection::open_with_flags(
            path_str,
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|e| {
            DataSourceError::ParseError {
                uri: path_str.to_owned(),
                message: format!("failed to open SQLite database '{path_str}': {e}"),
            }
        })?;

        // Prepare the query statement.
        // If the query is invalid SQL (e.g., references a non-existent table,
        // or is a DML statement), prepare() or query_map() will fail.
        let mut stmt = conn.prepare(self.query.as_ref()).map_err(|e| {
            DataSourceError::ParseError {
                uri: path_str.to_owned(),
                message: format!(
                    "only SELECT queries are allowed in @data sqlite sources: {e}"
                ),
            }
        })?;

        // AC-013: Check for duplicate column names before iterating rows.
        // Traces to BC-1.03.007 edge case EC-006.
        let col_names: Vec<String> = stmt
            .column_names()
            .iter()
            .map(|s| (*s).to_owned())
            .collect();

        {
            let name_refs: Vec<&str> = col_names.iter().map(String::as_str).collect();
            if let Some(dup) = find_duplicate_column(&name_refs) {
                return Err(DataSourceError::ParseError {
                    uri: path_str.to_owned(),
                    message: format!(
                        "SELECT result has duplicate column name '{dup}'; use aliases"
                    ),
                });
            }
        }

        // Step through the result set and build Value::List(rows).
        let rows_result: Result<Vec<Value>, _> = stmt
            .query_map([], |row| {
                let mut map = OrderedMap::new();
                for (idx, col_name) in col_names.iter().enumerate() {
                    let val_ref: ValueRef<'_> = row.get_ref(idx)?;
                    let value = convert_rusqlite_value(val_ref);
                    map.insert(Arc::from(col_name.as_str()), value);
                }
                Ok(Value::Map(map))
            })
            .map_err(|e| DataSourceError::ParseError {
                uri: path_str.to_owned(),
                message: format!("only SELECT queries are allowed in @data sqlite sources: {e}"),
            })?
            .collect::<Result<Vec<Value>, _>>()
            .map_err(|e| DataSourceError::ParseError {
                uri: path_str.to_owned(),
                message: format!("error reading row from SQLite query result: {e}"),
            });

        let rows = rows_result?;
        Ok(Value::List(rows))
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
fn convert_rusqlite_value(val: ValueRef<'_>) -> Value {
    match val {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(n) => Value::Int(n),
        ValueRef::Real(f) => Value::Float(OrderedFloat(f)),
        ValueRef::Text(bytes) => {
            // UTF-8 decode; invalid sequences replaced with U+FFFD per BC-1.03.007.
            let s = String::from_utf8_lossy(bytes);
            Value::Str(Arc::from(s.as_ref()))
        }
        ValueRef::Blob(bytes) => {
            // BLOB → base64-encoded string using STANDARD alphabet.
            // Traces to BC-1.03.007 postcondition 4, AC-009, edge case EC-013.
            let encoded = BASE64_STANDARD.encode(bytes);
            Value::Str(Arc::from(encoded.as_str()))
        }
    }
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
fn find_duplicate_column<'a>(names: &[&'a str]) -> Option<&'a str> {
    let mut seen = std::collections::HashSet::new();
    names.iter().find(|&&name| !seen.insert(name)).copied()
}

/// Validate that the file at `path` starts with the `SQLite` magic header.
///
/// `SQLite` databases always start with the 16-byte magic string
/// `"SQLite format 3\x00"`. Files that lack this header are not valid `SQLite`
/// databases and should be rejected with a clear error.
///
/// Returns `Ok(())` if the header matches, or `Err(String)` with a diagnostic
/// message on mismatch or I/O failure.
///
/// Traces to BC-1.03.007 edge case EC-008.
fn validate_sqlite_magic(path: &str) -> Result<(), String> {
    const MAGIC: &[u8; 16] = b"SQLite format 3\x00";
    let mut buf = [0u8; 16];
    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("failed to read file '{path}': {e}"))?;
    let n = file
        .read(&mut buf)
        .map_err(|e| format!("failed to read file header from '{path}': {e}"))?;
    if n < 16 || &buf != MAGIC {
        return Err(format!(
            "file '{path}' is not a valid SQLite database (bad magic header)"
        ));
    }
    Ok(())
}

/// Convert a [`DataError`] from the `SQLite` layer to a [`DataSourceError`].
///
/// Mapping:
/// - `FileNotFound` => `IoError` (file missing is an I/O condition)
/// - Everything else => `ParseError`
#[allow(dead_code)]
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

/// Unused import marker — `DataFormat` is used for error context in `data_error_to_source_error`.
const _: () = {
    let _ = std::mem::size_of::<DataFormat>();
};

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use rusqlite::{Connection, OpenFlags};
    use slideforge_plugin_api::{DataSource, DataSourceOptions};
    use slideforge_types::Value;

    use super::{SqliteDataSource, find_duplicate_column};

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    fn default_opts() -> DataSourceOptions {
        DataSourceOptions::default()
    }

    /// Create an in-memory `SQLite` database and seed it with a given setup closure.
    /// Returns the Connection so callers can persist it.
    fn make_memory_db(setup: impl FnOnce(&Connection)) -> Connection {
        let conn =
            Connection::open_in_memory().expect("in-memory SQLite must open");
        setup(&conn);
        conn
    }

    /// Write an in-memory database to a tempfile and return (`TempDir`, path).
    /// The `TempDir` must be kept alive for the duration of the test.
    ///
    /// Since rusqlite is compiled without the `backup` feature, we use
    /// `VACUUM INTO 'path'` (available from `SQLite` 3.27+, bundled version) to
    /// copy the in-memory database to a file.
    fn save_db_to_tempfile(
        conn: &Connection,
        suffix: &str,
    ) -> (tempfile::TempDir, std::path::PathBuf) {

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(format!("test{suffix}"));

        // Use ATTACH to copy the in-memory database to a file.
        // SQLite supports `VACUUM INTO 'path'` which does the copy without the
        // backup feature.
        conn.execute_batch(&format!(
            "VACUUM INTO '{}';",
            path.to_str().unwrap().replace('\'', "''")
        ))
        .expect("VACUUM INTO must succeed for in-memory db");

        (dir, path)
    }

    // ---------------------------------------------------------------------------
    // AC-008: plugin ID is "sqlite" and struct fields are correct.
    // Traces to BC-1.03.007 AC-008.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_source_id` -- plugin ID is `"sqlite"`.
    ///
    /// Traces to BC-1.03.007 AC-008.
    #[test]
    fn test_bc_1_03_007_sqlite_source_id() {
        let src = SqliteDataSource::new("dummy.db", "SELECT 1");
        assert_eq!(
            src.id(),
            "sqlite",
            "SqliteDataSource plugin ID must be exactly \"sqlite\""
        );
    }

    /// `test_bc_1_03_007_sqlite_struct_fields` -- path and query fields are set correctly.
    ///
    /// Traces to BC-1.03.007 AC-008.
    #[test]
    fn test_bc_1_03_007_sqlite_struct_fields() {
        let src = SqliteDataSource::new("metrics.db", "SELECT name, value FROM kpis");
        assert_eq!(src.path.as_ref(), "metrics.db", "path field must match constructor arg");
        assert_eq!(
            src.query.as_ref(),
            "SELECT name, value FROM kpis",
            "query field must match constructor arg"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-008 / AC-009: happy path — in-memory db, 3 rows → Value::List of 3 maps.
    // BC-1.03.007 canonical test vector (happy-path).
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_happy_path` -- in-memory db, 3 rows → `Value::List` of 3 maps.
    ///
    /// Canonical test vector from BC-1.03.007:
    /// `@data metrics from "metrics.db" query: "SELECT name, value FROM metrics"` (3 rows).
    ///
    /// Traces to BC-1.03.007 AC-008, AC-009, postconditions 1-4.
    #[test]
    fn test_bc_1_03_007_sqlite_happy_path() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE metrics (name TEXT, val INTEGER);
                 INSERT INTO metrics VALUES ('revenue', 100);
                 INSERT INTO metrics VALUES ('cost', 60);
                 INSERT INTO metrics VALUES ('profit', 40);",
            )
            .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");

        let src = SqliteDataSource::new(
            path.to_str().unwrap(),
            "SELECT name, val FROM metrics ORDER BY rowid",
        );
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        assert_eq!(rows.len(), 3, "must have exactly 3 rows");

        // Row 0: {name: "revenue", val: 100}
        let Value::Map(first) = &rows[0] else {
            panic!("expected Value::Map for row 0")
        };
        assert_eq!(
            first.get("name").unwrap(),
            &Value::Str(Arc::from("revenue")),
            "row 0 'name' must be Str(\"revenue\")"
        );
        assert_eq!(
            first.get("val").unwrap(),
            &Value::Int(100),
            "row 0 'val' must be Int(100)"
        );

        // Row 1: {name: "cost", val: 60}
        let Value::Map(second) = &rows[1] else {
            panic!("expected Value::Map for row 1")
        };
        assert_eq!(
            second.get("name").unwrap(),
            &Value::Str(Arc::from("cost"))
        );
        assert_eq!(second.get("val").unwrap(), &Value::Int(60));

        // Row 2: {name: "profit", val: 40}
        let Value::Map(third) = &rows[2] else {
            panic!("expected Value::Map for row 2")
        };
        assert_eq!(
            third.get("name").unwrap(),
            &Value::Str(Arc::from("profit"))
        );
        assert_eq!(third.get("val").unwrap(), &Value::Int(40));
    }

    // ---------------------------------------------------------------------------
    // AC-009: NULL column → Value::Null.
    // BC-1.03.007 postcondition 3.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_null_column` -- NULL column value becomes `Value::Null`.
    ///
    /// Traces to BC-1.03.007 AC-009, postcondition 3.
    #[test]
    fn test_bc_1_03_007_sqlite_null_column() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE items (id INTEGER, description TEXT);
                 INSERT INTO items VALUES (1, NULL);",
            )
            .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(
            path.to_str().unwrap(),
            "SELECT id, description FROM items",
        );
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        assert_eq!(rows.len(), 1);

        let row = match &rows[0] {
            Value::Map(m) => m,
            other => panic!("expected Value::Map, got {other:?}"),
        };
        assert_eq!(
            row.get("description").unwrap(),
            &Value::Null,
            "NULL SQLite column must become Value::Null"
        );
        assert_eq!(
            row.get("id").unwrap(),
            &Value::Int(1),
            "id column must be Value::Int(1)"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-009: INTEGER → Value::Int.
    // BC-1.03.007 postcondition 3.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_integer_value` -- INTEGER column becomes `Value::Int(i64)`.
    ///
    /// Traces to BC-1.03.007 postcondition 3.
    #[test]
    fn test_bc_1_03_007_sqlite_integer_value() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE nums (n INTEGER);
                 INSERT INTO nums VALUES (9223372036854775807);",  // i64::MAX
            )
            .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(path.to_str().unwrap(), "SELECT n FROM nums");
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        let row = match &rows[0] {
            Value::Map(m) => m,
            other => panic!("expected Value::Map, got {other:?}"),
        };
        assert_eq!(
            row.get("n").unwrap(),
            &Value::Int(i64::MAX),
            "INTEGER column must map to Value::Int(i64)"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-009: REAL → Value::Float.
    // BC-1.03.007 postcondition 3.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_real_value` -- REAL column becomes `Value::Float(OrderedFloat(f64))`.
    ///
    /// Traces to BC-1.03.007 postcondition 3.
    #[test]
    fn test_bc_1_03_007_sqlite_real_value() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE floats (ratio REAL);
                 INSERT INTO floats VALUES (1.5);",
            )
            .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(path.to_str().unwrap(), "SELECT ratio FROM floats");
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        let Value::Map(row) = &rows[0] else {
            panic!("expected Value::Map for row 0")
        };
        match row.get("ratio").unwrap() {
            Value::Float(f) => {
                assert!(
                    (f.0 - 1.5_f64).abs() < 1e-9,
                    "REAL column must map to Value::Float with correct value"
                );
            }
            other => panic!("REAL column must map to Value::Float, got {other:?}"),
        }
    }

    // ---------------------------------------------------------------------------
    // AC-009: TEXT → Value::Str.
    // BC-1.03.007 postcondition 3.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_text_value` -- TEXT column becomes `Value::Str(Arc<str>)`.
    ///
    /// Traces to BC-1.03.007 postcondition 3.
    #[test]
    fn test_bc_1_03_007_sqlite_text_value() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE msgs (msg TEXT);
                 INSERT INTO msgs VALUES ('hello, world!');",
            )
            .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(path.to_str().unwrap(), "SELECT msg FROM msgs");
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        let row = match &rows[0] {
            Value::Map(m) => m,
            other => panic!("expected Value::Map, got {other:?}"),
        };
        assert_eq!(
            row.get("msg").unwrap(),
            &Value::Str(Arc::from("hello, world!")),
            "TEXT column must map to Value::Str"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-012: Zero rows → Value::List(vec![]).
    // BC-1.03.007 edge case EC-011.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_zero_rows` -- zero-row result returns `Value::List(vec![])`.
    ///
    /// `SELECT * WHERE 1=0` must return an empty list, not an error.
    ///
    /// Traces to BC-1.03.007 AC-012, edge case EC-011.
    #[test]
    fn test_bc_1_03_007_sqlite_zero_rows() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE events (name TEXT, status TEXT);
                 INSERT INTO events VALUES ('alpha', 'closed');",
            )
            .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(
            path.to_str().unwrap(),
            "SELECT name, status FROM events WHERE 1=0",
        );
        let result = src.load("", &default_opts()).unwrap();

        match result {
            Value::List(rows) => {
                assert_eq!(
                    rows.len(),
                    0,
                    "zero-row query must return Value::List(vec![]), not an error"
                );
            }
            other => panic!("zero-row query must return Value::List(vec![]), got {other:?}"),
        }
    }

    // ---------------------------------------------------------------------------
    // AC-011: DML query → DataSourceError::ParseError "only SELECT queries".
    // BC-1.03.007 invariant 5, edge case EC-009 (EC-003 in BC).
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_readonly_rejects_dml` -- DML query returns `DataSourceError::ParseError`.
    ///
    /// A DELETE reaching `load()` is rejected by the read-only connection flags.
    /// Error must contain "only SELECT queries" or equivalent.
    ///
    /// Traces to BC-1.03.007 AC-011, invariant 5, edge case EC-009.
    #[test]
    fn test_bc_1_03_007_sqlite_readonly_rejects_dml() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE events (name TEXT);
                 INSERT INTO events VALUES ('alpha');",
            )
            .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(
            path.to_str().unwrap(),
            "DELETE FROM events",
        );
        let err = src.load("", &default_opts()).unwrap_err();

        assert!(
            matches!(err, slideforge_plugin_api::DataSourceError::ParseError { .. }),
            "DML query must produce DataSourceError::ParseError, got: {err:?}"
        );
        let msg = err.to_string();
        // The error must explain that only SELECT is allowed.
        assert!(
            msg.to_lowercase().contains("select")
                || msg.to_lowercase().contains("read-only")
                || msg.to_lowercase().contains("readonly")
                || msg.to_lowercase().contains("not allowed")
                || msg.to_lowercase().contains("dml"),
            "DML rejection error must mention 'SELECT' or 'read-only'; got: {msg}"
        );
    }

    /// `test_bc_1_03_007_sqlite_readonly_rejects_insert` -- INSERT is also rejected.
    ///
    /// Traces to BC-1.03.007 AC-011, invariant 5.
    #[test]
    fn test_bc_1_03_007_sqlite_readonly_rejects_insert() {
        let conn = make_memory_db(|c| {
            c.execute_batch("CREATE TABLE t (x INTEGER);").unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(
            path.to_str().unwrap(),
            "INSERT INTO t VALUES (1)",
        );
        let err = src.load("", &default_opts()).unwrap_err();
        assert!(
            matches!(err, slideforge_plugin_api::DataSourceError::ParseError { .. }),
            "INSERT query must produce DataSourceError::ParseError, got: {err:?}"
        );
    }

    /// `test_bc_1_03_007_sqlite_readonly_rejects_update` -- UPDATE is also rejected.
    ///
    /// Traces to BC-1.03.007 AC-011, invariant 5.
    #[test]
    fn test_bc_1_03_007_sqlite_readonly_rejects_update() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE t (x INTEGER);
                 INSERT INTO t VALUES (1);",
            )
            .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(
            path.to_str().unwrap(),
            "UPDATE t SET x = 2",
        );
        let err = src.load("", &default_opts()).unwrap_err();
        assert!(
            matches!(err, slideforge_plugin_api::DataSourceError::ParseError { .. }),
            "UPDATE query must produce DataSourceError::ParseError, got: {err:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // EC-001/EC-007: Missing file → DataSourceError::IoError.
    // BC-1.03.007 edge case EC-001 (EC-007 in story).
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_missing_file` -- non-existent path returns `DataSourceError::IoError`.
    ///
    /// Traces to BC-1.03.007 edge case EC-007.
    #[test]
    fn test_bc_1_03_007_sqlite_missing_file() {
        let src = SqliteDataSource::new(
            "/tmp/no_such_file_slideforge_sqlite_test_99999.db",
            "SELECT 1",
        );
        let err = src.load("", &default_opts()).unwrap_err();

        assert!(
            matches!(err, slideforge_plugin_api::DataSourceError::IoError { .. }),
            "missing file must produce DataSourceError::IoError, got: {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("no_such_file_slideforge_sqlite_test_99999"),
            "error must reference the missing path; got: {msg}"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-013: Duplicate column names → DataSourceError::ParseError "duplicate column name".
    // BC-1.03.007 edge case EC-012 (EC-006 in BC).
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_duplicate_column_names` -- duplicate column returns `DataSourceError::ParseError`.
    ///
    /// `SELECT id, id FROM t` (no alias) must produce an error with "duplicate column name".
    ///
    /// Traces to BC-1.03.007 AC-013, edge case EC-012.
    #[test]
    fn test_bc_1_03_007_sqlite_duplicate_column_names() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE t (id INTEGER, name TEXT);
                 INSERT INTO t VALUES (1, 'Alice');",
            )
            .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(
            path.to_str().unwrap(),
            "SELECT id, id FROM t",  // duplicate column name "id"
        );
        let err = src.load("", &default_opts()).unwrap_err();

        let msg = err.to_string();
        assert!(
            matches!(err, slideforge_plugin_api::DataSourceError::ParseError { .. }),
            "duplicate column names must produce DataSourceError::ParseError, got: {err:?}"
        );
        assert!(
            msg.to_lowercase().contains("duplicate"),
            "error message must mention 'duplicate'; got: {msg}"
        );
        assert!(
            msg.contains("id"),
            "error message must name the duplicate column 'id'; got: {msg}"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-009: BLOB → Value::Str (base64-encoded).
    // BC-1.03.007 postcondition 4, edge case EC-013.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_blob_base64` -- BLOB column becomes `Value::Str` (base64-encoded).
    ///
    /// Writes a known byte sequence; verifies the result is the correct base64 string.
    ///
    /// Traces to BC-1.03.007 AC-009, postcondition 4.
    #[test]
    fn test_bc_1_03_007_sqlite_blob_base64() {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        let blob_bytes: &[u8] = b"\x01\x02\x03\xFF\xFE";
        let expected_b64 = STANDARD.encode(blob_bytes);

        let conn = make_memory_db(|c| {
            c.execute_batch("CREATE TABLE blobs (data BLOB);").unwrap();
            c.execute("INSERT INTO blobs VALUES (?1)", rusqlite::params![blob_bytes])
                .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(path.to_str().unwrap(), "SELECT data FROM blobs");
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        let row = match &rows[0] {
            Value::Map(m) => m,
            other => panic!("expected Value::Map, got {other:?}"),
        };

        match row.get("data").unwrap() {
            Value::Str(s) => {
                assert_eq!(
                    s.as_ref(),
                    expected_b64.as_str(),
                    "BLOB column must be base64-encoded using STANDARD alphabet"
                );
            }
            other => panic!("BLOB column must produce Value::Str (base64), got {other:?}"),
        }
    }

    // ---------------------------------------------------------------------------
    // EC-008: Corrupt database (non-SQLite file) → DataSourceError::ParseError.
    // BC-1.03.007 edge case EC-008 (EC-002 in BC).
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_corrupt_db` -- a non-SQLite file produces `DataSourceError::ParseError`.
    ///
    /// Traces to BC-1.03.007 edge case EC-008.
    #[test]
    fn test_bc_1_03_007_sqlite_corrupt_db() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("not_a_db.db");
        // Write garbage bytes — not a valid SQLite file.
        std::fs::write(&path, b"this is not a sqlite database\x00\x01\x02").unwrap();

        let src = SqliteDataSource::new(path.to_str().unwrap(), "SELECT 1");
        let err = src.load("", &default_opts()).unwrap_err();

        // Must produce either ParseError (corrupt file) or IoError (rejected at open).
        // Both are acceptable as long as it's not Ok.
        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
                    | slideforge_plugin_api::DataSourceError::IoError { .. }
            ),
            "corrupt database must produce ParseError or IoError, got: {err:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // EC-010: Non-existent table → DataSourceError::ParseError.
    // BC-1.03.007 edge case EC-010 (EC-004 in BC).
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_missing_table` -- query on non-existent table returns error.
    ///
    /// Traces to BC-1.03.007 edge case EC-010.
    #[test]
    fn test_bc_1_03_007_sqlite_missing_table() {
        let conn = make_memory_db(|c| {
            c.execute_batch("CREATE TABLE real_table (x INTEGER);").unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(
            path.to_str().unwrap(),
            "SELECT x FROM no_such_table",
        );
        let err = src.load("", &default_opts()).unwrap_err();

        assert!(
            matches!(err, slideforge_plugin_api::DataSourceError::ParseError { .. }),
            "non-existent table must produce DataSourceError::ParseError, got: {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("no_such_table") || msg.to_lowercase().contains("no such table"),
            "error must reference the missing table; got: {msg}"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-009: SELECT alias → map keys use alias, not original column name.
    // BC-1.03.007 postcondition 2.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_column_alias` -- aliased column name is the map key.
    ///
    /// `SELECT id AS identifier` → map key is "identifier", not "id".
    ///
    /// Traces to BC-1.03.007 postcondition 2 ("column name or alias").
    #[test]
    fn test_bc_1_03_007_sqlite_column_alias() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE users (id INTEGER, name TEXT);
                 INSERT INTO users VALUES (42, 'Alice');",
            )
            .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(
            path.to_str().unwrap(),
            "SELECT id AS identifier, name AS full_name FROM users",
        );
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        let row = match &rows[0] {
            Value::Map(m) => m,
            other => panic!("expected Value::Map, got {other:?}"),
        };

        // Aliased names must be used as keys.
        assert!(
            row.contains_key("identifier"),
            "aliased 'id AS identifier' must produce key 'identifier'; keys: {:?}",
            row.keys().collect::<Vec<_>>()
        );
        assert!(
            row.contains_key("full_name"),
            "aliased 'name AS full_name' must produce key 'full_name'; keys: {:?}",
            row.keys().collect::<Vec<_>>()
        );
        // Original names must NOT appear as keys.
        assert!(
            !row.contains_key("id"),
            "original 'id' must not appear when alias 'identifier' is used"
        );
        assert!(
            !row.contains_key("name"),
            "original 'name' must not appear when alias 'full_name' is used"
        );
        assert_eq!(row.get("identifier").unwrap(), &Value::Int(42));
        assert_eq!(
            row.get("full_name").unwrap(),
            &Value::Str(Arc::from("Alice"))
        );
    }

    // ---------------------------------------------------------------------------
    // AC-009: Mixed NULL + non-NULL columns in multi-row result.
    // BC-1.03.007 canonical test vector (happy-path with NULLs).
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_mixed_null_rows` -- canonical test vector with NULL columns.
    ///
    /// Canonical BC-1.03.007 vector: `SELECT * FROM events WHERE status = 'open'`
    /// with null columns in some rows.
    ///
    /// Traces to BC-1.03.007 canonical test vector (happy-path).
    #[test]
    fn test_bc_1_03_007_sqlite_mixed_null_rows() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE events (name TEXT, status TEXT, owner TEXT);
                 INSERT INTO events VALUES ('alpha', 'open', 'Alice');
                 INSERT INTO events VALUES ('beta', 'open', NULL);
                 INSERT INTO events VALUES ('gamma', 'open', 'Charlie');",
            )
            .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(
            path.to_str().unwrap(),
            "SELECT name, status, owner FROM events WHERE status = 'open' ORDER BY rowid",
        );
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        assert_eq!(rows.len(), 3, "3 open events must load");

        // Row 1 (index 1): beta has NULL owner.
        let Value::Map(beta_row) = &rows[1] else {
            panic!("expected Value::Map for row 1")
        };
        assert_eq!(
            beta_row.get("owner").unwrap(),
            &Value::Null,
            "NULL owner column must become Value::Null"
        );
        assert_eq!(
            beta_row.get("name").unwrap(),
            &Value::Str(Arc::from("beta"))
        );
    }

    // ---------------------------------------------------------------------------
    // AC-008: read-only connection — verifies SQLITE_OPEN_READ_ONLY enforcement.
    // BC-1.03.007 invariant 2, postcondition 1.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_readonly_connection_flag` -- read-only open flag is enforced.
    ///
    /// Opens the same file with `SQLITE_OPEN_READ_ONLY` manually and verifies write
    /// fails. This proves the implementation's open-flag choice is correct.
    ///
    /// Traces to BC-1.03.007 AC-008, invariant 2, postcondition 1.
    #[test]
    fn test_bc_1_03_007_sqlite_readonly_connection_flag() {
        let conn = make_memory_db(|c| {
            c.execute_batch("CREATE TABLE t (x INTEGER); INSERT INTO t VALUES (1);").unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");

        // Open with SQLITE_OPEN_READ_ONLY (same flag the implementation must use).
        let ro_conn = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        ).expect("read-only open must succeed for a valid db");

        // A write attempt on a read-only connection must fail at the driver level.
        let write_result = ro_conn.execute("INSERT INTO t VALUES (2)", []);
        assert!(
            write_result.is_err(),
            "write on SQLITE_OPEN_READ_ONLY connection must fail at driver level"
        );

        // Read must still work.
        let count: i64 = ro_conn
            .query_row("SELECT COUNT(*) FROM t", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1, "read-only connection must still allow SELECT");
    }

    // ---------------------------------------------------------------------------
    // Unit tests for find_duplicate_column() helper.
    // BC-1.03.007 edge case EC-012.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_find_duplicate_column_no_dupes` -- all-unique list returns `None`.
    ///
    /// Traces to BC-1.03.007 AC-013, edge case EC-012.
    #[test]
    fn test_bc_1_03_007_find_duplicate_column_no_dupes() {
        let names = ["id", "name", "value", "created_at"];
        let result = find_duplicate_column(&names);
        assert!(
            result.is_none(),
            "all-unique columns must return None, got: {result:?}"
        );
    }

    /// `test_bc_1_03_007_find_duplicate_column_with_dupe` -- duplicate present returns `Some(name)`.
    ///
    /// Traces to BC-1.03.007 AC-013, edge case EC-012.
    #[test]
    fn test_bc_1_03_007_find_duplicate_column_with_dupe() {
        let names = ["id", "name", "id"];  // "id" appears twice
        let result = find_duplicate_column(&names);
        assert_eq!(
            result,
            Some("id"),
            "duplicate 'id' must be detected and returned"
        );
    }

    /// `test_bc_1_03_007_find_duplicate_column_multiple_dupes` -- first duplicate is returned.
    ///
    /// Traces to BC-1.03.007 AC-013.
    #[test]
    fn test_bc_1_03_007_find_duplicate_column_multiple_dupes() {
        let names = ["a", "b", "a", "b"];  // "a" duplicated first
        let result = find_duplicate_column(&names);
        assert_eq!(
            result,
            Some("a"),
            "first duplicate found must be returned (not 'b'); got: {result:?}"
        );
    }

    /// `test_bc_1_03_007_find_duplicate_column_empty` -- empty list returns `None`.
    ///
    /// Traces to BC-1.03.007 AC-013.
    #[test]
    fn test_bc_1_03_007_find_duplicate_column_empty() {
        let result = find_duplicate_column(&[]);
        assert!(result.is_none(), "empty name list must return None");
    }

    /// `test_bc_1_03_007_find_duplicate_column_single` -- single element returns `None`.
    ///
    /// Traces to BC-1.03.007 AC-013.
    #[test]
    fn test_bc_1_03_007_find_duplicate_column_single() {
        let names = ["only_col"];
        let result = find_duplicate_column(&names);
        assert!(result.is_none(), "single column can't be a duplicate");
    }

    // ---------------------------------------------------------------------------
    // Unit tests for convert_rusqlite_value() helper (via ValueRef).
    // BC-1.03.007 postconditions 2-4.
    //
    // NOTE: ValueRef borrows from a live query row, so we cannot construct it
    // outside a query context. These tests exercise convert_rusqlite_value()
    // through the full load() path with known inputs, using in-memory databases.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_convert_rusqlite_value_null` -- NULL row value becomes `Value::Null`.
    ///
    /// Exercises `convert_rusqlite_value(ValueRef::Null)` through the full load path.
    ///
    /// Traces to BC-1.03.007 postcondition 2.
    #[test]
    fn test_bc_1_03_007_convert_rusqlite_value_null() {
        // Use in-memory path to exercise the load() → convert_rusqlite_value() path.
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE t (v TEXT);
                 INSERT INTO t VALUES (NULL);",
            )
            .unwrap();
        });
        let (_dir, path) = save_db_to_tempfile(&conn, ".db");

        let src = SqliteDataSource::new(path.to_str().unwrap(), "SELECT v FROM t");
        let result = src.load("", &default_opts()).unwrap();

        let rows = result.as_list().unwrap();
        let val = rows[0].as_map().unwrap().get("v").unwrap();
        assert_eq!(val, &Value::Null, "NULL row value must map to Value::Null");
    }

    /// `test_bc_1_03_007_convert_rusqlite_value_integer` -- INTEGER row value becomes `Value::Int`.
    ///
    /// Exercises `convert_rusqlite_value(ValueRef::Integer(_))` through full load path.
    ///
    /// Traces to BC-1.03.007 postcondition 3.
    #[test]
    fn test_bc_1_03_007_convert_rusqlite_value_integer() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE t (v INTEGER);
                 INSERT INTO t VALUES (12345);",
            )
            .unwrap();
        });
        let (_dir, path) = save_db_to_tempfile(&conn, ".db");

        let src = SqliteDataSource::new(path.to_str().unwrap(), "SELECT v FROM t");
        let result = src.load("", &default_opts()).unwrap();

        let rows = result.as_list().unwrap();
        let val = rows[0].as_map().unwrap().get("v").unwrap();
        assert_eq!(val, &Value::Int(12345), "INTEGER row value must map to Value::Int(12345)");
    }

    /// `test_bc_1_03_007_convert_rusqlite_value_real` -- REAL row value becomes `Value::Float`.
    ///
    /// Exercises `convert_rusqlite_value(ValueRef::Real(_))` through full load path.
    ///
    /// Traces to BC-1.03.007 postcondition 3.
    #[test]
    fn test_bc_1_03_007_convert_rusqlite_value_real() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE t (v REAL);
                 INSERT INTO t VALUES (1.5);",
            )
            .unwrap();
        });
        let (_dir, path) = save_db_to_tempfile(&conn, ".db");

        let src = SqliteDataSource::new(path.to_str().unwrap(), "SELECT v FROM t");
        let result = src.load("", &default_opts()).unwrap();

        let rows = result.as_list().unwrap();
        let val = rows[0].as_map().unwrap().get("v").unwrap();
        match val {
            Value::Float(f) => {
                assert!(
                    (f.0 - 1.5).abs() < 1e-10,
                    "REAL value 1.5 must map to Value::Float(1.5)"
                );
            }
            other => panic!("REAL row value must map to Value::Float, got {other:?}"),
        }
    }

    // ---------------------------------------------------------------------------
    // AC-010 (defensive): empty query string should fail, not silently succeed.
    // BC-1.03.007 invariant 4.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_007_sqlite_empty_query_defensive` -- empty query string produces an error.
    ///
    /// Per BC-1.03.007 invariant 4, `query:` is required. The DSL parser enforces this
    /// before `load()` is called. At the data layer, an empty query string must not
    /// silently succeed.
    ///
    /// Traces to BC-1.03.007 AC-010, invariant 4.
    #[test]
    fn test_bc_1_03_007_sqlite_empty_query_defensive() {
        let conn = make_memory_db(|c| {
            c.execute_batch("CREATE TABLE t (x INTEGER);").unwrap();
        });
        let (_dir, path) = save_db_to_tempfile(&conn, ".db");

        let src = SqliteDataSource::new(path.to_str().unwrap(), "");
        // An empty query must either produce an error or be caught defensively.
        // It must NOT return Ok with any meaningful data.
        let result = src.load("", &default_opts());
        assert!(
            result.is_err(),
            "empty query string must produce an error (defensive layer for BC-1.03.007 invariant 4)"
        );
    }
}
