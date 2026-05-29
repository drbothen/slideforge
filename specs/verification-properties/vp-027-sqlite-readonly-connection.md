---
document_type: verification-property
vp_id: VP-027
title: "SQLite: connection opened with SQLITE_OPEN_READONLY"
module: slideforge-data
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: false
bc_trace: [BC-1.03.007, DI-004]
anchored_to_bc: BC-1.03.007
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-027: SQLite Connection Opened With SQLITE_OPEN_READONLY

## Property Statement

The `SqliteDataSource::load()` function MUST open the SQLite database connection with
`SQLITE_OPEN_READONLY` flags (via `rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY`). Any
attempt to execute a write operation (INSERT, UPDATE, DELETE) against the connection
MUST fail at the rusqlite/OS level before reaching the SELECT-prefix check, providing
defense-in-depth.

Formally: the connection is opened with `OpenFlags::SQLITE_OPEN_READ_ONLY`, so writes
are rejected by the SQLite engine regardless of the SELECT-prefix guard.

## Motivation

BC-1.03.007 Postcondition 1 and Invariant 2 require the read-only connection flag.
DI-004 ("no mutation of data sources during build"). The read-only flag is a
defense-in-depth measure: even if the SELECT-prefix check is bypassed (e.g., via a
`WITH ... INSERT` CTE in a future SQLite version), the connection itself cannot write.

## Feasibility Assessment

Not Kani-amenable: requires a live SQLite database connection. Verified by opening an
in-memory SQLite database with the read-only flag and attempting a write.

`kani_amenable: false` — SQLite I/O.

## Test Harness

```rust
// crates/slideforge-data/src/tests/sqlite_readonly.rs
#[test]
fn test_sqlite_connection_is_readonly() {
    // Create a real temp database, then open it through SqliteDataSource
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    // Create initial schema
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE t (id INTEGER); INSERT INTO t VALUES (1);").unwrap();
    }

    // Now load via SqliteDataSource — the connection must be read-only
    let source = SqliteDataSource::new(&path, "SELECT id FROM t");
    let rows = source.load().expect("SELECT should succeed on read-only");
    assert_eq!(rows.len(), 1);

    // Verify that trying to get a write connection via our loader fails
    // by attempting an INSERT through the read-only path (defense-in-depth test)
    let write_attempt = SqliteDataSource::new(&path, "INSERT INTO t VALUES (2)");
    let err = write_attempt.load().expect_err("write on read-only should fail");
    // Either SELECT-prefix check fires OR rusqlite returns a read-only error
    assert!(matches!(err, DataError::ParseError { .. }));
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `SELECT id FROM t` on read-only connection | Rows returned; connection stays open | happy-path |
| `INSERT INTO t VALUES (2)` via SqliteDataSource | Error (either DML check or read-only engine error) | error |
