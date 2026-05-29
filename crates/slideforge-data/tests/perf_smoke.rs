//! Performance smoke tests for NFR-036 (XLSX 10k rows < 2000ms) and
//! NFR-037 (`SQLite` 10k rows < 500ms).
//!
//! These tests are `#[ignore]` by default so that `cargo nextest run` / `cargo test`
//! in CI does not run them in the normal test suite (they take 0.5–2s each and
//! depend on I/O). CI invokes them explicitly via:
//!
//! ```text
//! cargo test --test perf_smoke -- --ignored
//! ```
//!
//! ## AC-014 / NFR-036 / NFR-037
//!
//! The benchmarks in `benches/data_sources.rs` measure mean latency via criterion
//! but criterion alone does not fail CI on a threshold regression without extra
//! wiring. These smoke tests are the CI-gateable enforcement mechanism:
//! they run the same 10k-row workload once, measure wall-clock time, and
//! `assert!(elapsed < threshold)`.
//!
//! CI integration: `cargo test --test perf_smoke -- --ignored --test-threads 1`
//! is wired into `.github/workflows/ci.yml` under the `perf-smoke` job
//! (F-LOW-P5-1 fix, STORY-020 pass-5). The job runs on `ubuntu-latest`
//! (linux-x86_64) sequentially (`--test-threads 1`) to reduce timer noise
//! from shared CI runners.
//!
//! ## NFR-038 (memory profiling)
//!
//! Memory peak measurement requires a dedicated profiler (heaptrack / dhat /
//! cargo-flamegraph). Enforcement is deferred — see NFR-038 in the NFR catalog
//! for the tracking item. CI gate for peak memory will be added when a
//! platform-portable measurement approach is validated.
//!
//! Traces to BC-1.03.006 AC-014 (XLSX), BC-1.03.007 AC-014 (`SQLite`),
//! NFR-036, NFR-037.

// Integration test file: unwrap() in fixture helpers is acceptable.
// This suppression mirrors the pattern in sqlite::tests and xlsx::tests.
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::time::Instant;

use rusqlite::Connection;
use rust_xlsxwriter::Workbook;
use slideforge_data::{SqliteDataSource, XlsxDataSource};
use slideforge_plugin_api::{DataSource, DataSourceOptions};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// XLSX 10k-row fixture (mirrors bench shape from benches/data_sources.rs).
// ---------------------------------------------------------------------------

fn make_xlsx_10k_rows() -> (TempDir, PathBuf) {
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();

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
    let path = dir.path().join("perf_smoke_10k.xlsx");
    std::fs::write(&path, buf).unwrap();
    (dir, path)
}

// ---------------------------------------------------------------------------
// SQLite 10k-row fixture (mirrors bench shape from benches/data_sources.rs).
// ---------------------------------------------------------------------------

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

    let tx = conn.unchecked_transaction().unwrap();
    {
        let mut stmt = conn
            .prepare(
                "INSERT INTO metrics (id, name, value, flag, score) VALUES (?1,?2,?3,?4,?5)",
            )
            .unwrap();
        // Cast i64 → f64 for fixture data; minor precision loss is acceptable
        // for test scaffolding (value column is not checked for exact equality).
        #[allow(clippy::cast_precision_loss)]
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
    let path = dir.path().join("perf_smoke_10k.db");
    conn.execute_batch(&format!(
        "VACUUM INTO '{}';",
        path.to_str().unwrap().replace('\'', "''")
    ))
    .expect("VACUUM INTO must succeed");

    (dir, path)
}

// ---------------------------------------------------------------------------
// NFR-036: XLSX 10k rows < 2000ms
// ---------------------------------------------------------------------------

/// `nfr_036_xlsx_10k_rows_under_2000ms` — AC-014 / NFR-036 CI gate.
///
/// Loads a 10k-row × 5-col XLSX file via `XlsxDataSource` and asserts the
/// wall-clock elapsed time is under 2000ms.
///
/// This test is `#[ignore]` by default. Run explicitly via:
/// ```text
/// cargo test --test perf_smoke -- --ignored
/// ```
///
/// CI wiring: invoked in the `perf-smoke` job of `.github/workflows/ci.yml`
/// (wired by F-LOW-P5-1, STORY-020 pass-5).
///
/// Traces to BC-1.03.006 AC-014, NFR-036.
#[test]
#[ignore = "perf-gate: run with 'cargo test --test perf_smoke -- --ignored'"]
fn nfr_036_xlsx_10k_rows_under_2000ms() {
    let (_dir, path) = make_xlsx_10k_rows();
    let path_str = path.to_str().expect("tempfile path must be UTF-8");
    let src = XlsxDataSource::new(path_str);
    let opts = DataSourceOptions::default();

    let start = Instant::now();
    let result = src.load("", &opts).expect("XLSX 10k load must not fail");
    let elapsed = start.elapsed();

    // Correctness: full 10k rows loaded.
    let n = match &result {
        slideforge_types::Value::List(rows) => rows.len(),
        other => panic!("XlsxDataSource must return Value::List, got: {other:?}"),
    };
    assert_eq!(n, 10_000, "NFR-036: must load all 10_000 rows (got {n})");

    // NFR-036 threshold: < 2000ms wall-clock.
    assert!(
        elapsed.as_millis() < 2_000,
        "NFR-036 FAIL: XlsxDataSource 10k-row load took {}ms, threshold is 2000ms",
        elapsed.as_millis()
    );
}

// ---------------------------------------------------------------------------
// NFR-037: SQLite 10k rows < 500ms
// ---------------------------------------------------------------------------

/// `nfr_037_sqlite_10k_rows_under_500ms` — AC-014 / NFR-037 CI gate.
///
/// Loads a 10k-row `SQLite` table via `SqliteDataSource` and asserts the
/// wall-clock elapsed time is under 500ms.
///
/// This test is `#[ignore]` by default. Run explicitly via:
/// ```text
/// cargo test --test perf_smoke -- --ignored
/// ```
///
/// CI wiring: invoked in the `perf-smoke` job of `.github/workflows/ci.yml`
/// (wired by F-LOW-P5-1, STORY-020 pass-5).
///
/// Traces to BC-1.03.007 AC-014, NFR-037.
#[test]
#[ignore = "perf-gate: run with 'cargo test --test perf_smoke -- --ignored'"]
fn nfr_037_sqlite_10k_rows_under_500ms() {
    let (_dir, path) = make_sqlite_10k_rows();
    let path_str = path.to_str().expect("tempfile path must be UTF-8");
    let src = SqliteDataSource::new(
        path_str,
        "SELECT id, name, value, flag, score FROM metrics ORDER BY id",
    );
    let opts = DataSourceOptions::default();

    let start = Instant::now();
    let result = src.load("", &opts).expect("SQLite 10k load must not fail");
    let elapsed = start.elapsed();

    // Correctness: full 10k rows loaded.
    let n = match &result {
        slideforge_types::Value::List(rows) => rows.len(),
        other => panic!("SqliteDataSource must return Value::List, got: {other:?}"),
    };
    assert_eq!(n, 10_000, "NFR-037: must load all 10_000 rows (got {n})");

    // NFR-037 threshold: < 500ms wall-clock.
    assert!(
        elapsed.as_millis() < 500,
        "NFR-037 FAIL: SqliteDataSource 10k-row load took {}ms, threshold is 500ms",
        elapsed.as_millis()
    );
}
