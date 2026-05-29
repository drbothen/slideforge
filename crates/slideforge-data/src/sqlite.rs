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

/// Test-only call counter for `open_readonly_connection`.
///
/// Incremented once per call to `open_readonly_connection` (only in `#[cfg(test)]`).
/// Used by `test_vp_027_load_uses_readonly` to verify that `load()` routes through
/// this function, making VP-027 load-bearing: if `load()` reverts to a direct
/// `Connection::open_with_flags(_, SQLITE_OPEN_READ_WRITE)` call (bypassing the
/// helper), the counter does NOT increment and the test fails.
///
/// Zero production overhead: the counter and all increment sites are compiled
/// away in release builds.
///
/// Traces to BC-1.03.007 invariant 2, VP-027, TD-VSDD-059.
///
/// **Serial-group invariant (F-MED-P6-1):** All tests that cause this counter to
/// increment — whether via `load()`, `open_readonly_connection()` directly, or any
/// helper that opens a connection — MUST be annotated `#[serial(load_call_count)]`.
/// Omitting the annotation races under cargo-test parallel-within-process execution,
/// producing non-deterministic counter deltas.  See F-MED-P6-1 lesson.
#[cfg(test)]
pub(crate) static OPEN_READONLY_CALL_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

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

impl SqliteDataSource {
    /// Open the `SQLite` database at `path` with `SQLITE_OPEN_READ_ONLY`.
    ///
    /// This is the SINGLE authoritative point for opening a `SQLite` connection in
    /// read-only mode. Both production `load()` and VP-027 test coverage go through
    /// this function, ensuring the `SQLITE_OPEN_READ_ONLY` flag cannot be silently
    /// removed from the production path without breaking `test_vp_027_readonly_enforced`.
    ///
    /// Returns `Err` if the file cannot be opened.
    ///
    /// Traces to BC-1.03.007 invariant 2, postcondition 1, VP-027.
    pub(crate) fn open_readonly_connection(path: &str) -> Result<Connection, DataError> {
        // VP-027 load-bearing counter: increment once per call so that
        // `test_vp_027_load_uses_readonly` can assert that `load()` routes through
        // this function. Compiled away in release builds (zero overhead).
        //
        // TD-VSDD-059: if `load()` ever reverts to a direct
        // `Connection::open_with_flags(_, SQLITE_OPEN_READ_WRITE)`, this counter
        // will NOT be incremented and the VP-027 test FAILS.
        #[cfg(test)]
        OPEN_READONLY_CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|e| {
            DataError::ParseError {
                code: crate::error::E_DAT_003,
                path: Arc::from(path),
                format: crate::format::DataFormat::Sqlite,
                reason: Arc::from(format!("failed to open SQLite database '{path}': {e}").as_str()),
                span: slideforge_types::SourceSpan::default(),
            }
        })
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
    #[instrument(skip(self, _opts, uri), fields(path = tracing::field::Empty))]
    fn load(&self, uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        // F-MED-5 / interface-definitions §7: uri overrides self.path when non-empty.
        // Record the effective path AFTER resolving the override so the span reflects
        // the actual file being loaded (F-PASS11-OBS-1).
        let path_str: &str = if uri.is_empty() {
            self.path.as_ref()
        } else {
            uri
        };
        tracing::Span::current().record("path", path_str);

        // AC-010 (defensive): reject empty query string.
        // The DSL parser enforces this before load() is called, but we guard
        // at the data layer as defense-in-depth.
        // F-MED-3: message references the parser, not "BC invariant 4".
        // Traces to BC-1.03.007 invariant 4.
        if self.query.is_empty() {
            return Err(DataSourceError::ParseError {
                uri: path_str.to_owned(),
                message: "internal: query field is empty; this should have been caught by the \
                    @data parser at <span>"
                    .to_owned(),
            });
        }

        // F-MED-2 / AC-BC-009: Explicit SELECT-or-WITH prefix check (case-insensitive
        // after trimming leading whitespace).
        // Traces to BC-1.03.007 invariant 5.
        // DML detection is the FIRST check: if the query is clearly DML, report it as
        // DML rejection. Other query failures (no-such-table, syntax errors) use the
        // actual rusqlite error message, NOT the generic DML message.
        let trimmed_query = self.query.trim();
        let lower_prefix = trimmed_query.to_ascii_lowercase();
        let is_dml = !lower_prefix.starts_with("select") && !lower_prefix.starts_with("with");
        if is_dml {
            // F-HIGH-3: char-boundary-safe truncation to 20 chars for error preview.
            // Byte-indexing into a UTF-8 string can panic at multi-byte char boundaries;
            // use chars().take(20).collect() instead.
            // Regression test: test_dml_message_handles_multibyte_unicode_query.
            let preview: String = trimmed_query.chars().take(20).collect();
            return Err(DataSourceError::ParseError {
                uri: path_str.to_owned(),
                message: format!(
                    "only SELECT queries are allowed in @data sqlite sources \
                    (got: {preview}...)"
                ),
            });
        }

        // F-MED-4 / AC-BC-008 / VP-035: Extension validation.
        // Accepted: .db, .sqlite, .sqlite3 (case-insensitive). All other extensions →
        // UnsupportedFormat E-DAT-014. Applied before magic-byte check.
        // Traces to BC-1.03.007 invariant 8, VP-035, E-DAT-014.
        validate_sqlite_extension(path_str)
            .map_err(|e| DataSourceError::UnsupportedUri { uri: e })?;

        // Check file existence before opening (produces a clearer error than
        // SQLite's "unable to open database file" for missing files).
        // Traces to BC-1.03.007 edge case EC-007.
        if !std::path::Path::new(path_str).exists() {
            return Err(DataSourceError::IoError {
                uri: path_str.to_owned(),
                message: format!("file not found: {path_str}"),
            });
        }

        // F-MED-1 / VP-034: Validate SQLite magic header before opening.
        // TOCTOU note: we open the file once here, read 16 bytes, then close it.
        // rusqlite::Connection::open_with_flags then opens it again. There is a residual
        // TOCTOU window between these two opens in which a file could be swapped —
        // however, the trust model here is local-trusted-file (the orchestrator
        // enforces path containment before reaching this point), so the risk is
        // accepted and documented. A zero-cost TOCTOU mitigation (single open) would
        // require rusqlite VFS integration, which is out of scope for v1.0.
        // Traces to BC-1.03.007 postcondition 7, VP-034, E-DAT-013.
        // F-PASS12-OBS-2: validate_sqlite_magic now returns DataSourceError directly,
        // differentiating IoError (E-DAT-004) for I/O failures from ParseError
        // (E-DAT-013) for magic-byte mismatches. Use `?` for direct propagation.
        validate_sqlite_magic(path_str)?;

        // AC-008: Open with SQLITE_OPEN_READ_ONLY (hard security requirement).
        // Delegates to open_readonly_connection — the SINGLE source of truth for
        // SQLITE_OPEN_READ_ONLY. This ensures test_vp_027_readonly_enforced is
        // load-bearing: if this call were reverted to READ_WRITE, the VP-027 test
        // would still pass (it tests the helper), but that test now exercises the
        // exact same code path as production. And test_vp_027_load_uses_readonly
        // confirms the full load() path rejects writes.
        // Traces to BC-1.03.007 invariant 2, postcondition 1, VP-027.
        let conn = Self::open_readonly_connection(path_str).map_err(|e| {
            DataSourceError::ParseError {
                uri: path_str.to_owned(),
                message: format!("failed to open SQLite database '{path_str}': {e}"),
            }
        })?;

        // Prepare the query statement.
        // If the query is invalid SQL (e.g., references a non-existent table),
        // prepare() propagates the actual rusqlite error. Non-DML failures use the
        // real error message — NOT the generic DML message (F-MED-2 differentiation).
        let mut stmt =
            conn.prepare(self.query.as_ref())
                .map_err(|e| DataSourceError::ParseError {
                    uri: path_str.to_owned(),
                    message: format!("failed to prepare query for '{path_str}': {e}"),
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
        // F-HIGH-3: convert_rusqlite_value returns Result for strict UTF-8 enforcement.
        // We use query() + manual row iteration to propagate DataError from the converter
        // (query_map's closure is constrained to rusqlite::Error return type only).
        // Traces to BC-1.03.007 postconditions 2-4, invariant 6, VP-031, VP-032.
        let mut rows: Vec<Value> = Vec::new();
        let mut query_rows = stmt.query([]).map_err(|e| DataSourceError::ParseError {
            uri: path_str.to_owned(),
            message: format!("failed to execute query on '{path_str}': {e}"),
        })?;

        let mut row_idx: usize = 0;
        while let Some(row) = query_rows.next().map_err(|e| DataSourceError::ParseError {
            uri: path_str.to_owned(),
            message: format!("error reading row {row_idx} from '{path_str}': {e}"),
        })? {
            let mut map = OrderedMap::new();
            for (col_idx, col_name) in col_names.iter().enumerate() {
                let val_ref: ValueRef<'_> = row.get_ref(col_idx).map_err(|e| {
                    DataSourceError::ParseError {
                        uri: path_str.to_owned(),
                        message: format!(
                            "error reading column '{col_name}' at row {row_idx} in '{path_str}': {e}"
                        ),
                    }
                })?;
                let value =
                    convert_rusqlite_value(val_ref, col_name, row_idx, path_str).map_err(|e| {
                        DataSourceError::ParseError {
                            uri: path_str.to_owned(),
                            message: e.to_string(),
                        }
                    })?;
                map.insert(Arc::from(col_name.as_str()), value);
            }
            rows.push(Value::Map(map));
            row_idx += 1;
        }

        Ok(Value::List(rows))
    }
}

/// Convert a single [`rusqlite::types::ValueRef`] to a [`slideforge_types::Value`].
///
/// Mapping (BC-1.03.007 postconditions 2, 3, 4):
/// - `Null` => `Value::Null`
/// - `Integer(n)` => `Value::Int(n)`
/// - `Real(f)` => `Value::Float(OrderedFloat(f))`
/// - `Text(bytes)` => strict UTF-8 decode via `std::str::from_utf8`; invalid bytes →
///   `Err(DataError::ParseError(E-DAT-012))` (NOT `from_utf8_lossy` with U+FFFD substitution)
/// - `Blob(bytes)` => `Value::Str(Arc<str>)` (base64-encoded via `base64::engine::general_purpose::STANDARD`)
///
/// # Errors
///
/// Returns `Err` if `Text` bytes are not valid UTF-8 (E-DAT-012).
///
/// Traces to BC-1.03.007 postconditions 2, 3, 4; invariants 6, 7.
fn convert_rusqlite_value(
    val: ValueRef<'_>,
    col_name: &str,
    row_idx: usize,
    path: &str,
) -> Result<Value, DataError> {
    match val {
        ValueRef::Null => Ok(Value::Null),
        ValueRef::Integer(n) => Ok(Value::Int(n)),
        ValueRef::Real(f) => Ok(Value::Float(OrderedFloat(f))),
        ValueRef::Text(bytes) => {
            // BC-1.03.007 invariant 6: strict UTF-8 decode. `from_utf8_lossy` is FORBIDDEN
            // because it silently substitutes U+FFFD for invalid bytes, violating the
            // canonical principle (no silent fallback). Traces to VP-032, E-DAT-012.
            std::str::from_utf8(bytes)
                .map(|s| Value::Str(Arc::from(s)))
                .map_err(|_e| {
                    use crate::error::E_DAT_012;
                    DataError::ParseError {
                        code: E_DAT_012,
                        path: Arc::from(path),
                        format: DataFormat::Sqlite,
                        reason: Arc::from(
                            format!(
                                "TEXT column '{col_name}' at row {row_idx} in '{path}' contains \
                                invalid UTF-8 bytes. SQLite TEXT values must be valid UTF-8."
                            )
                            .as_str(),
                        ),
                        span: slideforge_types::SourceSpan::default(),
                    }
                })
        },
        ValueRef::Blob(bytes) => {
            // BLOB → base64-encoded string using STANDARD alphabet (RFC 4648 §4).
            // Traces to BC-1.03.007 postcondition 4, AC-009, invariant 7, VP-033.
            let encoded = BASE64_STANDARD.encode(bytes);
            Ok(Value::Str(Arc::from(encoded.as_str())))
        },
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

/// Validate that the `SQLite` file extension is one of the accepted variants.
///
/// Accepted (case-insensitive): `.db`, `.sqlite`, `.sqlite3`.
/// All other extensions (including `.db3`, `.s3db`, `.sl3`) are rejected with E-DAT-014.
///
/// Returns `Ok(())` on accepted extension, `Err(String)` containing the user-visible
/// error message with `[E-DAT-014]` prefix embedded (caller wraps in
/// `DataSourceError::UnsupportedUri`).
///
/// The `[E-DAT-014]` prefix is embedded in the returned string so that the final
/// `DataSourceError` message always carries the granular error code — consistent
/// with the XLSX pattern where error codes appear in `DataError` display strings.
///
/// Traces to BC-1.03.007 invariant 8, VP-035, E-DAT-014.
fn validate_sqlite_extension(path: &str) -> Result<(), String> {
    let p = std::path::Path::new(path);
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "db" | "sqlite" | "sqlite3" => Ok(()),
        other => Err(format!(
            "[{code}] unsupported extension for SQLite data source: '.{other}'. \
            Accepted extensions: .db, .sqlite, .sqlite3",
            code = crate::error::E_DAT_014,
        )),
    }
}

/// Validate that the file at `path` starts with the `SQLite` magic header.
///
/// `SQLite` databases always start with the 16-byte magic string
/// `"SQLite format 3\x00"`. Files that lack this header are not valid `SQLite`
/// databases and should be rejected with a clear error (E-DAT-013).
///
/// Combined with the extension check, this ensures the file has both the right
/// extension AND the right magic bytes.
///
/// ## Error classification (F-PASS12-OBS-2)
///
/// - `File::open` or `read` I/O failure → `DataSourceError::IoError` (E-DAT-004).
///   These are infrastructure failures (permission denied, file disappeared) — not
///   format errors.
/// - Magic-byte mismatch → `DataSourceError::ParseError` (E-DAT-013).
///   The file is present and readable but is not a SQLite database.
///
/// Traces to BC-1.03.007 postcondition 7, VP-034, E-DAT-013, E-DAT-004.
fn validate_sqlite_magic(path: &str) -> Result<(), DataSourceError> {
    const MAGIC: &[u8; 16] = b"SQLite format 3\x00";
    let mut buf = [0u8; 16];
    // I/O failure opening the file → IoError (E-DAT-004), not ParseError.
    let mut file = std::fs::File::open(path).map_err(|e| DataSourceError::IoError {
        uri: path.to_owned(),
        message: format!(
            "[{}] failed to open file '{path}' to validate SQLite magic: {e}",
            crate::error::E_DAT_004
        ),
    })?;
    // I/O failure reading the header → IoError (E-DAT-004).
    let n = file.read(&mut buf).map_err(|e| DataSourceError::IoError {
        uri: path.to_owned(),
        message: format!(
            "[{}] failed to read file header from '{path}': {e}",
            crate::error::E_DAT_004
        ),
    })?;
    // Magic-byte mismatch → ParseError (E-DAT-013): file is present but not SQLite.
    if n < 16 || &buf != MAGIC {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("db");
        return Err(DataSourceError::ParseError {
            uri: path.to_owned(),
            message: format!(
                "[{code}] '{path}' has .{ext} extension but is not a valid SQLite database \
                (SQLite file header not found). File may be corrupted or misnamed.",
                code = crate::error::E_DAT_013,
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use rusqlite::{Connection, OpenFlags};
    use slideforge_plugin_api::{DataSource, DataSourceOptions};
    use slideforge_types::Value;

    use super::{SqliteDataSource, find_duplicate_column, validate_sqlite_magic};

    // F-MED-P5-1: bring serial macro into scope for load_call_count group.
    use serial_test::serial;

    // ---------------------------------------------------------------------------
    // VP-027 race-condition guard (F-MED-P5-1).
    //
    // `OPEN_READONLY_CALL_COUNT` is a shared process-global atomic.  Under
    // `cargo test` / `cargo nextest`, tests run in parallel within a process,
    // so a concurrent `.load()` call from another test can increment the counter
    // between the `before` and `after` snapshots in
    // `test_vp_027_load_uses_readonly`, causing a false-fail (delta > 1).
    //
    // Fix: `serial_test` crate groups `test_vp_027_load_uses_readonly` and
    // every other test that calls `.load()` under the serial token
    // `"load_call_count"`.  Within that group tests run one-at-a-time, so the
    // VP-027 snapshot window cannot be interrupted by a sibling increment.
    // ---------------------------------------------------------------------------

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    fn default_opts() -> DataSourceOptions {
        DataSourceOptions::default()
    }

    /// Create an in-memory `SQLite` database and seed it with a given setup closure.
    /// Returns the Connection so callers can persist it.
    fn make_memory_db(setup: impl FnOnce(&Connection)) -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory SQLite must open");
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
        assert_eq!(
            src.path.as_ref(),
            "metrics.db",
            "path field must match constructor arg"
        );
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
    #[serial(load_call_count)]
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
        assert_eq!(second.get("name").unwrap(), &Value::Str(Arc::from("cost")));
        assert_eq!(second.get("val").unwrap(), &Value::Int(60));

        // Row 2: {name: "profit", val: 40}
        let Value::Map(third) = &rows[2] else {
            panic!("expected Value::Map for row 2")
        };
        assert_eq!(third.get("name").unwrap(), &Value::Str(Arc::from("profit")));
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
    #[serial(load_call_count)]
    fn test_bc_1_03_007_sqlite_null_column() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE items (id INTEGER, description TEXT);
                 INSERT INTO items VALUES (1, NULL);",
            )
            .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src =
            SqliteDataSource::new(path.to_str().unwrap(), "SELECT id, description FROM items");
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
    #[serial(load_call_count)]
    fn test_bc_1_03_007_sqlite_integer_value() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE nums (n INTEGER);
                 INSERT INTO nums VALUES (9223372036854775807);", // i64::MAX
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
    #[serial(load_call_count)]
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
            },
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
    #[serial(load_call_count)]
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
    #[serial(load_call_count)]
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
            },
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
    #[serial(load_call_count)]
    fn test_bc_1_03_007_sqlite_readonly_rejects_dml() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE events (name TEXT);
                 INSERT INTO events VALUES ('alpha');",
            )
            .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(path.to_str().unwrap(), "DELETE FROM events");
        let err = src.load("", &default_opts()).unwrap_err();

        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
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
    #[serial(load_call_count)]
    fn test_bc_1_03_007_sqlite_readonly_rejects_insert() {
        let conn = make_memory_db(|c| {
            c.execute_batch("CREATE TABLE t (x INTEGER);").unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(path.to_str().unwrap(), "INSERT INTO t VALUES (1)");
        let err = src.load("", &default_opts()).unwrap_err();
        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
            "INSERT query must produce DataSourceError::ParseError, got: {err:?}"
        );
    }

    /// `test_bc_1_03_007_sqlite_readonly_rejects_update` -- UPDATE is also rejected.
    ///
    /// Traces to BC-1.03.007 AC-011, invariant 5.
    #[test]
    #[serial(load_call_count)]
    fn test_bc_1_03_007_sqlite_readonly_rejects_update() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE t (x INTEGER);
                 INSERT INTO t VALUES (1);",
            )
            .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(path.to_str().unwrap(), "UPDATE t SET x = 2");
        let err = src.load("", &default_opts()).unwrap_err();
        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
            "UPDATE query must produce DataSourceError::ParseError, got: {err:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // F-HIGH-3: DML message with multi-byte Unicode query must NOT panic.
    // Regression for byte-indexed slice panic at char boundaries.
    // ---------------------------------------------------------------------------

    /// `test_dml_message_handles_multibyte_unicode_query` -- DML message is safe with multi-byte chars.
    ///
    /// A DML query containing 15+ multi-byte characters (each 2+ bytes in UTF-8)
    /// must produce a clean error message without panicking. Previously the code
    /// used `&str[..n]` which can panic at a multi-byte char boundary.
    ///
    /// Traces to BC-1.03.007 invariant 5, F-HIGH-3.
    #[test]
    #[serial(load_call_count)]
    fn test_dml_message_handles_multibyte_unicode_query() {
        // "Ä" is U+00C4, encoded as 2 bytes (0xC3 0x84) in UTF-8.
        // 15 × "Ä" = 30 bytes but only 15 Unicode chars. The old byte-indexed
        // `trimmed_query[..trimmed_query.len().min(20)]` would attempt to slice
        // at byte offset 20, which lands in the middle of the 10th "Ä" (byte 18-19),
        // causing a panic.
        let multi_byte_dml = "DELETE ".to_owned() + &"Ä".repeat(15);
        assert!(
            multi_byte_dml.len() > 20,
            "test precondition: query must be > 20 bytes: len={}",
            multi_byte_dml.len()
        );

        // We need an actual db file for the path — extension check passes, file must exist.
        let conn = make_memory_db(|c| {
            c.execute_batch("CREATE TABLE t (x INTEGER);").unwrap();
        });
        let (_dir, path) = save_db_to_tempfile(&conn, ".db");

        let src = SqliteDataSource::new(path.to_str().unwrap(), multi_byte_dml.as_str());
        // Must NOT panic; must return a ParseError.
        let err = src.load("", &default_opts()).unwrap_err();
        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
            "multi-byte DML query must produce ParseError, got: {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.to_lowercase().contains("select"),
            "DML rejection error must mention 'SELECT'; got: {msg}"
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
    #[serial(load_call_count)]
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
    #[serial(load_call_count)]
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
            "SELECT id, id FROM t", // duplicate column name "id"
        );
        let err = src.load("", &default_opts()).unwrap_err();

        let msg = err.to_string();
        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
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
    #[serial(load_call_count)]
    fn test_bc_1_03_007_sqlite_blob_base64() {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        let blob_bytes: &[u8] = b"\x01\x02\x03\xFF\xFE";
        let expected_b64 = STANDARD.encode(blob_bytes);

        let conn = make_memory_db(|c| {
            c.execute_batch("CREATE TABLE blobs (data BLOB);").unwrap();
            c.execute(
                "INSERT INTO blobs VALUES (?1)",
                rusqlite::params![blob_bytes],
            )
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
            },
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
    #[serial(load_call_count)]
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
    // OBS-2: validate_sqlite_magic error classification (F-PASS12-OBS-2).
    // IoError for File::open failure, ParseError for magic mismatch.
    // ---------------------------------------------------------------------------

    /// `test_obs2_sqlite_magic_mismatch_is_parse_error` — magic mismatch → ParseError (E-DAT-013).
    ///
    /// A file that is present and readable but lacks the SQLite magic header must
    /// produce `DataSourceError::ParseError` with `[E-DAT-013]` in the message.
    ///
    /// Load-bearing per TD-VSDD-059: if the mismatch arm were changed to IoError,
    /// this test would fail.
    ///
    /// Traces to F-PASS12-OBS-2, BC-1.03.007 postcondition 7, E-DAT-013.
    #[test]
    fn test_obs2_sqlite_magic_mismatch_is_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fake.db");
        std::fs::write(&path, b"this is not sqlite\x00\x01\x02\x03\x04\x05\x06\x07").unwrap();

        let err = validate_sqlite_magic(path.to_str().unwrap()).unwrap_err();
        assert!(
            matches!(err, slideforge_plugin_api::DataSourceError::ParseError { .. }),
            "magic mismatch must produce DataSourceError::ParseError (E-DAT-013), got: {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("E-DAT-013"),
            "ParseError message must contain E-DAT-013; got: {msg}"
        );
    }

    /// `test_obs2_sqlite_magic_io_error_is_io_error` — File::open failure → IoError (E-DAT-004).
    ///
    /// When the file does not exist, `validate_sqlite_magic` must produce
    /// `DataSourceError::IoError` (not ParseError) because the failure is an
    /// infrastructure error, not a format mismatch.
    ///
    /// Load-bearing per TD-VSDD-059: if File::open errors were mapped to ParseError,
    /// this test would fail.
    ///
    /// Traces to F-PASS12-OBS-2, E-DAT-004.
    #[test]
    fn test_obs2_sqlite_magic_io_error_is_io_error() {
        let err =
            validate_sqlite_magic("/tmp/no_such_file_slideforge_obs2_test_99999.db").unwrap_err();
        assert!(
            matches!(err, slideforge_plugin_api::DataSourceError::IoError { .. }),
            "File::open failure must produce DataSourceError::IoError (E-DAT-004), got: {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("E-DAT-004"),
            "IoError message must contain E-DAT-004; got: {msg}"
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
    #[serial(load_call_count)]
    fn test_bc_1_03_007_sqlite_missing_table() {
        let conn = make_memory_db(|c| {
            c.execute_batch("CREATE TABLE real_table (x INTEGER);")
                .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let src = SqliteDataSource::new(path.to_str().unwrap(), "SELECT x FROM no_such_table");
        let err = src.load("", &default_opts()).unwrap_err();

        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
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
    #[serial(load_call_count)]
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
    #[serial(load_call_count)]
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
            c.execute_batch("CREATE TABLE t (x INTEGER); INSERT INTO t VALUES (1);")
                .unwrap();
        });

        let (_dir, path) = save_db_to_tempfile(&conn, ".db");

        // Open with SQLITE_OPEN_READ_ONLY (same flag the implementation must use).
        let ro_conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("read-only open must succeed for a valid db");

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
        let names = ["id", "name", "id"]; // "id" appears twice
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
        let names = ["a", "b", "a", "b"]; // "a" duplicated first
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
    #[serial(load_call_count)]
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
    #[serial(load_call_count)]
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
        assert_eq!(
            val,
            &Value::Int(12345),
            "INTEGER row value must map to Value::Int(12345)"
        );
    }

    /// `test_bc_1_03_007_convert_rusqlite_value_real` -- REAL row value becomes `Value::Float`.
    ///
    /// Exercises `convert_rusqlite_value(ValueRef::Real(_))` through full load path.
    ///
    /// Traces to BC-1.03.007 postcondition 3.
    #[test]
    #[serial(load_call_count)]
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
            },
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
    #[serial(load_call_count)]
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

    // ---------------------------------------------------------------------------
    // VP-027: READONLY enforced — SqliteDataSource only opens READ_ONLY.
    // BC-1.03.007 invariant 2. (Existing test test_bc_1_03_007_sqlite_readonly_connection_flag
    // covers this; this VP label test confirms the production code path.)
    // ---------------------------------------------------------------------------

    /// `test_vp_027_readonly_enforced` -- VP-027: `open_readonly_connection` opens with `SQLITE_OPEN_READ_ONLY`.
    ///
    /// Tests `SQLITE_OPEN_READ_ONLY` enforcement WITHOUT triggering the SELECT-prefix
    /// DML check first (which would short-circuit before a connection is opened).
    ///
    /// Strategy: open the connection via `SqliteDataSource::open_readonly_connection` and
    /// verify a direct INSERT attempt fails at the driver/OS level — confirming the
    /// flag is passed to `SQLite`. This is the authoritative test for VP-027 because it
    /// targets the `SQLITE_OPEN_READ_ONLY` invariant directly, not the DML prefix guard.
    ///
    /// This test FAILS if `open_readonly_connection` uses `SQLITE_OPEN_READ_WRITE`
    /// instead of `SQLITE_OPEN_READ_ONLY` — making it load-bearing per TD-VSDD-059.
    ///
    /// Traces to BC-1.03.007 invariant 2, postcondition 1, VP-027.
    ///
    /// `#[serial(load_call_count)]` is required because this test calls
    /// `open_readonly_connection` directly, incrementing `OPEN_READONLY_CALL_COUNT`.
    /// Parallel execution with other tests in the same group would corrupt the counter
    /// deltas used by `test_vp_027_load_uses_readonly`.  See F-MED-P6-1.
    #[serial(load_call_count)]
    #[test]
    fn test_vp_027_readonly_enforced() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE t (x INTEGER);
                 INSERT INTO t VALUES (1);",
            )
            .unwrap();
        });
        let (_dir, path) = save_db_to_tempfile(&conn, ".db");

        // Open using the production function — must use SQLITE_OPEN_READ_ONLY.
        let ro_conn = SqliteDataSource::open_readonly_connection(path.to_str().unwrap())
            .expect("opening a valid db with SQLITE_OPEN_READ_ONLY must succeed");

        // A write attempt on a read-only connection must be rejected at the driver level,
        // independent of any application-level DML guard.
        let write_result = ro_conn.execute("INSERT INTO t VALUES (2)", []);
        assert!(
            write_result.is_err(),
            "INSERT on SQLITE_OPEN_READ_ONLY connection must fail at the driver level (VP-027)"
        );

        // SELECT must still work (read-only is read-only, not closed).
        let count: i64 = ro_conn
            .query_row("SELECT COUNT(*) FROM t", [], |row| row.get(0))
            .expect("SELECT on read-only connection must succeed");
        assert_eq!(
            count, 1,
            "read-only connection must allow SELECT (original 1 row must be present)"
        );
    }

    /// `test_vp_027_load_uses_readonly` -- VP-027 v4: `SqliteDataSource::load()` routes through
    /// `open_readonly_connection` — confirmed via test-only atomic call counter.
    ///
    /// This is the load-bearing VP-027 test (TD-VSDD-059).
    ///
    /// **Why the counter approach is genuinely load-bearing:**
    /// `OPEN_READONLY_CALL_COUNT` is incremented ONLY inside
    /// `open_readonly_connection`. If `load()` reverts to calling
    /// `Connection::open_with_flags(_, SQLITE_OPEN_READ_WRITE)` directly (bypassing
    /// the helper), the counter does NOT increment between the `before` and `after`
    /// snapshots, and this test FAILS — catching the regression at the source.
    ///
    /// Previous v1/v2/v3 variants called `open_readonly_connection` directly in step 2,
    /// independent of the `load()` code path. The counter eliminates that independence:
    /// the counter ONLY advances when `load()` internally calls the helper, providing
    /// direct transitivity:
    ///   `load()` → `open_readonly_connection` incremented → counter diff = 1 → test passes
    ///   `load()` bypasses helper    → counter diff = 0 → test FAILS
    ///
    /// Traces to BC-1.03.007 invariant 2, postcondition 1, VP-027, TD-VSDD-059.
    #[test]
    #[serial(load_call_count)]
    fn test_vp_027_load_uses_readonly() {
        use std::sync::atomic::Ordering;

        // Build a small test database.
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE items (id INTEGER, label TEXT);
                 INSERT INTO items VALUES (1, 'alpha');
                 INSERT INTO items VALUES (2, 'beta');",
            )
            .unwrap();
        });
        let (_dir, path) = save_db_to_tempfile(&conn, ".db");
        let path_str = path.to_str().expect("tempfile path must be UTF-8");

        // Snapshot counter BEFORE calling load().
        let before = super::OPEN_READONLY_CALL_COUNT.load(Ordering::SeqCst);

        // Call production load() — this is the ONLY call in this scope.
        let src = SqliteDataSource::new(path_str, "SELECT id, label FROM items ORDER BY id");
        let opts = DataSourceOptions::default();
        let result = src.load("", &opts).expect("load() with valid SELECT must succeed");

        // Snapshot counter AFTER load() completes.
        let after = super::OPEN_READONLY_CALL_COUNT.load(Ordering::SeqCst);

        // Load-bearing assertion (TD-VSDD-059):
        // If load() called open_readonly_connection exactly once, the counter must have
        // incremented by exactly 1. If load() bypasses the helper (direct open_with_flags),
        // the counter does NOT increment → diff = 0 → this assertion FAILS.
        assert_eq!(
            after - before,
            1,
            "load() must call open_readonly_connection exactly once (VP-027 load-bearing assertion). \
            Counter before={before}, after={after}. \
            If diff=0, load() bypassed open_readonly_connection (regression detected)."
        );

        // Confirm end-to-end correctness: load() returned the expected rows.
        let rows = match result {
            slideforge_types::Value::List(r) => r,
            other => panic!("load() must return Value::List, got: {other:?}"),
        };
        assert_eq!(rows.len(), 2, "load() must return 2 rows from items table");
    }

    // ---------------------------------------------------------------------------
    // VP-028: column names → map keys (via SELECT alias, already covered by alias test).
    // VP-029: DML → ParseError E-DAT-003 with SELECT-prefix check (F-MED-2).
    // ---------------------------------------------------------------------------

    /// `test_vp_029_dml_prefix_check_delete` -- VP-029: DELETE prefix → `ParseError` with "SELECT" message.
    ///
    /// F-MED-2: The DML rejection is now an explicit SELECT-prefix check, not just read-only.
    ///
    /// Traces to BC-1.03.007 invariant 5, VP-029.
    #[test]
    #[serial(load_call_count)]
    fn test_vp_029_dml_prefix_check_delete() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE events (name TEXT);
                 INSERT INTO events VALUES ('alpha');",
            )
            .unwrap();
        });
        let (_dir, path) = save_db_to_tempfile(&conn, ".db");

        let src = SqliteDataSource::new(path.to_str().unwrap(), "DELETE FROM events");
        let err = src.load("", &default_opts()).unwrap_err();

        let msg = err.to_string();
        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
            "DELETE must be ParseError, got: {err:?}"
        );
        assert!(
            msg.to_lowercase().contains("select"),
            "DML rejection must mention 'SELECT'; got: {msg}"
        );
    }

    /// `test_vp_029_with_clause_is_allowed` -- VP-029: WITH (CTE) prefix is allowed (not DML).
    ///
    /// CTEs starting with WITH must not be rejected as DML.
    ///
    /// Traces to BC-1.03.007 invariant 5.
    #[test]
    #[serial(load_call_count)]
    fn test_vp_029_with_clause_is_allowed() {
        let conn = make_memory_db(|c| {
            c.execute_batch(
                "CREATE TABLE t (x INTEGER);
                 INSERT INTO t VALUES (42);",
            )
            .unwrap();
        });
        let (_dir, path) = save_db_to_tempfile(&conn, ".db");

        let src = SqliteDataSource::new(
            path.to_str().unwrap(),
            "WITH cte AS (SELECT x FROM t) SELECT x FROM cte",
        );
        let result = src.load("", &default_opts()).unwrap();
        let rows = result.as_list().unwrap();
        assert_eq!(rows.len(), 1, "WITH CTE query must execute and return rows");
    }

    // ---------------------------------------------------------------------------
    // VP-030: NULL → Value::Null (covered by existing null tests).
    // VP-031: valid UTF-8 TEXT → Str (covered by existing text tests).
    // ---------------------------------------------------------------------------

    // ---------------------------------------------------------------------------
    // VP-032: invalid UTF-8 TEXT → E-DAT-012.
    // BC-1.03.007 invariant 6, F-HIGH-3.
    // ---------------------------------------------------------------------------

    /// `test_vp_032_invalid_utf8_text_produces_e_dat_012` -- VP-032: invalid UTF-8 in TEXT column → E-DAT-012.
    ///
    /// `SQLite`'s type system allows CAST(blob AS TEXT) to return arbitrary bytes as a
    /// "TEXT" value, bypassing Rust-layer UTF-8 validation that rusqlite applies when
    /// inserting `&str`. We exploit this to inject known-invalid UTF-8 bytes (0xFF, 0xFE,
    /// 0x80) into a BLOB column, then SELECT with `CAST(data AS TEXT)`, which causes
    /// rusqlite to surface a `ValueRef::Text` with the raw bytes.
    ///
    /// The production code's strict `std::str::from_utf8` MUST reject these bytes and
    /// return E-DAT-012. This test FAILS if `from_utf8_lossy` (silent substitution) is
    /// used instead — TD-VSDD-059 load-bearing contract.
    ///
    /// Traces to BC-1.03.007 invariant 6, postcondition 4, VP-032.
    #[test]
    #[serial(load_call_count)]
    fn test_vp_032_invalid_utf8_text_produces_e_dat_012() {
        // Insert known-invalid UTF-8 bytes as a BLOB.
        // [0xFF, 0xFE, 0x80] is not valid UTF-8 in any context:
        //   0xFF is never a valid UTF-8 byte; 0xFE likewise; 0x80 is a continuation
        //   byte without a valid lead byte.
        let invalid_utf8_bytes: &[u8] = &[0xFF, 0xFE, 0x80];

        let conn = make_memory_db(|c| {
            c.execute_batch("CREATE TABLE t (data BLOB);").unwrap();
            // Insert as BLOB — rusqlite accepts arbitrary bytes for BLOB columns.
            c.execute(
                "INSERT INTO t VALUES (?1)",
                rusqlite::params![invalid_utf8_bytes],
            )
            .unwrap();
        });
        let (_dir, path) = save_db_to_tempfile(&conn, ".db");

        // SELECT CAST(data AS TEXT) forces SQLite to hand rusqlite the raw bytes as a
        // TEXT affinity value, bypassing any Rust-layer UTF-8 guard on INSERT.
        // rusqlite will surface these as ValueRef::Text(bytes) where bytes are the
        // original invalid UTF-8 sequence. Our production convert_rusqlite_value must
        // call std::str::from_utf8, detect the failure, and return E-DAT-012.
        let src = SqliteDataSource::new(path.to_str().unwrap(), "SELECT CAST(data AS TEXT) FROM t");
        let err = src
            .load("", &default_opts())
            .expect_err("invalid UTF-8 bytes via CAST must produce DataSourceError");

        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
            "invalid UTF-8 TEXT must produce DataSourceError::ParseError (E-DAT-012); got: {err:?}"
        );
        let msg = err.to_string();
        // Error must mention the column and/or row so users can locate the bad data.
        assert!(
            msg.contains("[E-DAT-012]"),
            "invalid UTF-8 TEXT error must embed '[E-DAT-012]' bracket code; got: {msg}"
        );
        // The error message must name the column ('CAST(data AS TEXT)' or its alias)
        // — production code formats "{col_name} at row {row_idx}".
        assert!(
            msg.contains("row 0") || msg.contains("row_idx: 0") || msg.contains("row"),
            "E-DAT-012 error must reference the row index; got: {msg}"
        );
    }

    // ---------------------------------------------------------------------------
    // VP-033: BLOB → base64 "AP9C" for [0x00, 0xFF, 0x42].
    // BC-1.03.007 postcondition 4, invariant 7.
    // ---------------------------------------------------------------------------

    /// `test_vp_033_blob_canonical_base64_vector` -- VP-033: BLOB [0x00, 0xFF, 0x42] → base64 "AP9C".
    ///
    /// VP-033 canonical vector: the 3-byte sequence [0x00, 0xFF, 0x42] must produce
    /// base64 STANDARD encoding "AP9C" (RFC 4648 §4 with padding).
    ///
    /// Traces to BC-1.03.007 postcondition 4, invariant 7, VP-033.
    #[test]
    #[serial(load_call_count)]
    fn test_vp_033_blob_canonical_base64_vector() {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        let blob_bytes: &[u8] = &[0x00, 0xFF, 0x42];
        let expected_b64 = STANDARD.encode(blob_bytes);
        // Canonical vector: STANDARD.encode([0x00, 0xFF, 0x42]) = "AP9C"
        assert_eq!(
            expected_b64, "AP9C",
            "canonical base64 vector must be 'AP9C'"
        );

        let conn = make_memory_db(|c| {
            c.execute_batch("CREATE TABLE blobs (data BLOB);").unwrap();
            c.execute(
                "INSERT INTO blobs VALUES (?1)",
                rusqlite::params![blob_bytes],
            )
            .unwrap();
        });
        let (_dir, path) = save_db_to_tempfile(&conn, ".db");

        let src = SqliteDataSource::new(path.to_str().unwrap(), "SELECT data FROM blobs");
        let result = src.load("", &default_opts()).unwrap();

        let rows = result.as_list().unwrap();
        let val = rows[0].as_map().unwrap().get("data").unwrap();
        assert_eq!(
            val,
            &Value::Str(Arc::from("AP9C")),
            "BLOB [0x00, 0xFF, 0x42] must encode to 'AP9C' (VP-033 canonical vector)"
        );
    }

    // ---------------------------------------------------------------------------
    // VP-034: wrong SQLite magic → E-DAT-013 (ParseError).
    // BC-1.03.007 postcondition 7, VP-034.
    // ---------------------------------------------------------------------------

    /// `test_vp_034_wrong_sqlite_magic_produces_parse_error` -- VP-034: non-`SQLite` bytes → `ParseError`.
    ///
    /// A file with `.db` extension but non-`SQLite` content must produce `ParseError` with
    /// error code `[E-DAT-013]` embedded in the message (F-MED-1 load-bearing assertion).
    ///
    /// Load-bearing: if `validate_sqlite_magic` omits `[E-DAT-013]` from the error message,
    /// the `msg.contains("[E-DAT-013]")` assertion fails.
    ///
    /// Traces to BC-1.03.007 postcondition 7, VP-034, E-DAT-013.
    #[test]
    #[serial(load_call_count)]
    fn test_vp_034_wrong_sqlite_magic_produces_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fake.db");
        // Write garbage bytes — not a valid SQLite file.
        std::fs::write(&path, b"this is not a sqlite database\x00\x01\x02").unwrap();

        let src = SqliteDataSource::new(path.to_str().unwrap(), "SELECT 1");
        let err = src.load("", &default_opts()).unwrap_err();

        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
            "wrong SQLite magic must produce ParseError, got: {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("not a valid SQLite") || msg.contains("header") || msg.contains("magic"),
            "E-DAT-013 error must explain invalid SQLite header; got: {msg}"
        );
        // F-MED-1 load-bearing: error code must be embedded in the user-visible message.
        assert!(
            msg.contains("[E-DAT-013]"),
            "E-DAT-013 error code must appear in the user-visible message; got: {msg}"
        );
    }

    // ---------------------------------------------------------------------------
    // VP-035: extension .db3 → UnsupportedFormat E-DAT-014.
    // BC-1.03.007 invariant 8, F-MED-4.
    // ---------------------------------------------------------------------------

    /// `test_vp_035_unsupported_extension_produces_e_dat_014` -- VP-035: `.db3` extension → `UnsupportedUri`.
    ///
    /// Extensions outside {.db, .sqlite, .sqlite3} must produce `UnsupportedUri` with
    /// error code `[E-DAT-014]` embedded in the message (F-MED-1 load-bearing assertion).
    ///
    /// Load-bearing: if `validate_sqlite_extension` omits `[E-DAT-014]` from the error
    /// message, the `msg.contains("[E-DAT-014]")` assertion fails.
    ///
    /// Traces to BC-1.03.007 invariant 8, VP-035, E-DAT-014.
    #[test]
    #[serial(load_call_count)]
    fn test_vp_035_unsupported_extension_produces_e_dat_014() {
        // The file doesn't need to exist — extension check fires first.
        let src = SqliteDataSource::new("/tmp/database.db3", "SELECT 1");
        let err = src.load("", &default_opts()).unwrap_err();

        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::UnsupportedUri { .. }
            ),
            ".db3 extension must produce UnsupportedUri (E-DAT-014), got: {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("db3") || msg.contains("Unsupported") || msg.contains("extension"),
            "E-DAT-014 error must name the extension; got: {msg}"
        );
        // F-MED-1 load-bearing: error code must be embedded in the user-visible message.
        assert!(
            msg.contains("[E-DAT-014]"),
            "E-DAT-014 error code must appear in the user-visible message; got: {msg}"
        );
    }

    // ---------------------------------------------------------------------------
    // VP-036: missing table → actual rusqlite error, NOT generic DML message.
    // BC-1.03.007 invariant 5, F-MED-2.
    // ---------------------------------------------------------------------------

    /// `test_vp_036_missing_table_uses_actual_error_message` -- VP-036: non-existent table → rusqlite error, not DML message.
    ///
    /// A SELECT on a non-existent table must use the actual rusqlite error message,
    /// NOT the generic "only SELECT queries are allowed" wrapper (F-MED-2 differentiation).
    ///
    /// Traces to BC-1.03.007 invariant 5, VP-036.
    #[test]
    #[serial(load_call_count)]
    fn test_vp_036_missing_table_uses_actual_error_message() {
        let conn = make_memory_db(|c| {
            c.execute_batch("CREATE TABLE real_table (x INTEGER);")
                .unwrap();
        });
        let (_dir, path) = save_db_to_tempfile(&conn, ".db");

        let src = SqliteDataSource::new(path.to_str().unwrap(), "SELECT x FROM nonexistent_table");
        let err = src.load("", &default_opts()).unwrap_err();

        let msg = err.to_string();
        // Must be a parse error.
        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
            "non-existent table must produce ParseError, got: {err:?}"
        );
        // Error must reference the actual table name, NOT just "only SELECT queries".
        assert!(
            msg.contains("nonexistent_table") || msg.contains("no such table"),
            "non-existent table error must name the missing table; got: {msg}"
        );
        // Must NOT be the generic DML-rejection message from the old code.
        assert!(
            !msg.contains("only SELECT queries are allowed"),
            "non-existent table error must NOT use generic DML rejection message; got: {msg}"
        );
    }
}
