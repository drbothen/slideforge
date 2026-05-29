//! Performance benchmarks for `slideforge-data` DataSource implementations.
//!
//! These benchmarks enforce the NFR performance gates from BC-1.03.006 and
//! BC-1.03.007. Run with:
//!
//! ```text
//! cargo bench -p slideforge-data
//! ```
//!
//! ## NFR targets
//!
//! | Benchmark        | Target    | NFR       |
//! |------------------|-----------|-----------|
//! | `xlsx_10k_rows`  | < 2_000ms | NFR-036   |
//! | `sqlite_10k_rows`| < 500ms   | NFR-037   |
//!
//! ## NFR-038 note (memory profiling)
//!
//! NFR-038 requires peak memory measurement during large-dataset load.
//! Memory profiling is NOT measurable via criterion alone — it requires a
//! memory profiler (e.g., `heaptrack`, `dhat`, or `cargo-flamegraph`).
//! See: `/Users/jmagady/Dev/slideforge/.factory/stories/stories/STORY-020-datasource-excel-sqlite.md`
//! AC-014 for the NFR-038 requirement. When a memory profiler is wired into
//! CI, add a bench here using the 10k-row fixture already defined below.

use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rusqlite::Connection;
use rust_xlsxwriter::Workbook;
use slideforge_data::{SqliteDataSource, XlsxDataSource};
use slideforge_plugin_api::{DataSource, DataSourceOptions};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// XLSX fixture: 10k rows × 5 cols (NFR-036 canonical bench shape).
// ---------------------------------------------------------------------------

/// Generate a 10k-row XLSX workbook into a tempfile and return the dir + path.
///
/// Shape: headers `["id", "name", "value", "flag", "score"]`, 10_000 data rows.
///
/// Traces to BC-1.03.006 AC-014, NFR-036.
fn make_xlsx_10k_rows() -> (TempDir, PathBuf) {
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();

    // Header row — NFR-036: 10k rows × 5 cols
    ws.write_string(0, 0, "id").unwrap();
    ws.write_string(0, 1, "name").unwrap();
    ws.write_string(0, 2, "value").unwrap();
    ws.write_string(0, 3, "flag").unwrap();
    ws.write_string(0, 4, "score").unwrap();

    for i in 1_u32..=10_000 {
        ws.write_number(i, 0, f64::from(i)).unwrap();
        ws.write_string(i, 1, format!("item_{i}")).unwrap();
        ws.write_number(i, 2, f64::from(i) * 1.5).unwrap();
        ws.write_boolean(i, 3, i % 2 == 0).unwrap();
        ws.write_number(i, 4, f64::from(i % 100)).unwrap();
    }

    let buf = wb.save_to_buffer().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bench_10k.xlsx");
    std::fs::write(&path, buf).unwrap();
    (dir, path)
}

/// Benchmark: load a 10k-row XLSX workbook via `XlsxDataSource`.
///
/// NFR-036: must complete in < 2_000ms on the CI baseline hardware.
/// Run this bench in CI and assert `p50 < 2_000ms` in the bench regression step.
///
/// Traces to BC-1.03.006 AC-014, NFR-036.
fn bench_xlsx_10k_rows(c: &mut Criterion) {
    let (_dir, path) = make_xlsx_10k_rows();
    let path_str = path.to_str().expect("bench tempfile path must be UTF-8");
    let src = XlsxDataSource::new(path_str);
    let opts = DataSourceOptions::default();

    c.bench_function("xlsx_10k_rows", |b| {
        b.iter(|| {
            let result = src.load("", &opts).expect("xlsx 10k bench must not fail");
            // Positive-coverage row count assertion: guards against silent partial-load
            // regressions (off-by-one, early-break, missing column conversion).
            // F-MED-1: bench must assert full 10k load, not just non-zero.
            let n = match &result {
                slideforge_types::Value::List(rows) => rows.len(),
                _ => panic!("xlsx bench must return Value::List"),
            };
            assert_eq!(n, 10_000, "bench must load all 10_000 rows (got {n})");
            std::hint::black_box(n)
        })
    });
}

// ---------------------------------------------------------------------------
// SQLite fixture: 10k rows (NFR-037 canonical bench shape).
// ---------------------------------------------------------------------------

/// Populate an in-memory SQLite database with 10k rows and write to tempfile.
///
/// Table: `metrics(id INTEGER, name TEXT, value REAL, flag INTEGER, score INTEGER)`.
///
/// Uses `VACUUM INTO` (SQLite 3.27+) to copy in-memory DB to file, same pattern
/// as production tests.
///
/// Traces to BC-1.03.007 AC-014, NFR-037.
fn make_sqlite_10k_rows() -> (TempDir, PathBuf) {
    let conn = Connection::open_in_memory().expect("in-memory SQLite must open");
    conn.execute_batch(
        "CREATE TABLE metrics (
            id    INTEGER PRIMARY KEY,
            name  TEXT,
            value REAL,
            flag  INTEGER,
            score INTEGER
        );",
    )
    .unwrap();

    // Batch insert via transaction for speed (avoids 10k fsync operations).
    // NFR-037: 10k rows per the performance gate.
    let tx = conn.unchecked_transaction().unwrap();
    {
        let mut stmt = conn
            .prepare("INSERT INTO metrics (id, name, value, flag, score) VALUES (?1,?2,?3,?4,?5)")
            .unwrap();
        for i in 1i64..=10_000 {
            stmt.execute(rusqlite::params![
                i,
                format!("item_{i}"),
                i as f64 * 1.5,
                i % 2,
                i % 100,
            ])
            .unwrap();
        }
    }
    tx.commit().unwrap();

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bench_10k.db");
    conn.execute_batch(&format!(
        "VACUUM INTO '{}';",
        path.to_str().unwrap().replace('\'', "''")
    ))
    .expect("VACUUM INTO must succeed");

    (dir, path)
}

/// Benchmark: load a 10k-row SQLite table via `SqliteDataSource`.
///
/// NFR-037: must complete in < 500ms on CI baseline hardware.
/// Run this bench in CI and assert `p50 < 500ms` in the bench regression step.
///
/// Traces to BC-1.03.007 AC-014, NFR-037.
fn bench_sqlite_10k_rows(c: &mut Criterion) {
    let (_dir, path) = make_sqlite_10k_rows();
    let path_str = path.to_str().expect("bench tempfile path must be UTF-8");
    let src = SqliteDataSource::new(
        path_str,
        "SELECT id, name, value, flag, score FROM metrics ORDER BY id",
    );
    let opts = DataSourceOptions::default();

    c.bench_function("sqlite_10k_rows", |b| {
        b.iter(|| {
            let result = src.load("", &opts).expect("sqlite 10k bench must not fail");
            // Positive-coverage row count assertion: guards against silent partial-load
            // regressions (off-by-one, early-break, missing column conversion).
            // F-MED-1: bench must assert full 10k load, not just non-zero.
            let n = match &result {
                slideforge_types::Value::List(rows) => rows.len(),
                _ => panic!("sqlite bench must return Value::List"),
            };
            assert_eq!(n, 10_000, "bench must load all 10_000 rows (got {n})");
            std::hint::black_box(n)
        })
    });
}

criterion_group!(benches, bench_xlsx_10k_rows, bench_sqlite_10k_rows);
criterion_main!(benches);
