---
document_type: verification-property
vp_id: VP-036
title: "SQLite: non-existent table produces actual SQLite error, not DML message"
module: slideforge-data
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: false
bc_trace: [BC-1.03.007]
anchored_to_bc: BC-1.03.007
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-036: SQLite Non-Existent Table Produces Actual SQLite Error, Not DML Message

## Property Statement

When a `SELECT` query references a table that does not exist in the database, the
loader MUST return `Err(DataError::ParseError)` with the ACTUAL rusqlite error message
(e.g., `"SQLite error: no such table: 'events'"`) — NOT the generic DML rejection
message (`"only SELECT queries are allowed"`). The DML message MUST ONLY appear when
the query does not start with `SELECT` or `WITH`.

Formally: `trim(query).to_uppercase().starts_with("SELECT") ∧ query_fails_with_rusqlite_error(E)`
→ `Err(DataError::ParseError)` with message containing `E.to_string()`, not the DML message.

## Motivation

BC-1.03.007 AC-BC-009 (Item I). This is the second half of the DML-rejection
differentiation: VP-029 proves DML keywords produce the DML message; VP-036 proves
that other query failures (table not found, syntax error) produce the ACTUAL rusqlite
error. The adversary pass 1 finding F-MED-2 identified that wrapping both cases under
the same DML message string would confuse users: "only SELECT queries are allowed" is
a misleading error when the table literally does not exist.

## Feasibility Assessment

Not Kani-amenable: requires a live SQLite database with the query executed against it.

`kani_amenable: false` — SQLite I/O.

## Test Harness

```rust
// crates/slideforge-data/src/tests/sqlite_errors.rs
#[test]
fn test_nonexistent_table_produces_actual_sqlite_error_not_dml_message() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    // Create a DB with table 'metrics', NOT 'events'
    conn.execute_batch("CREATE TABLE metrics (x INTEGER);").unwrap();
    let path = save_to_temp(&conn);

    let source = SqliteDataSource::new(&path, "SELECT * FROM events");
    let err = source.load().expect_err("nonexistent table should produce error");
    match err {
        DataError::ParseError { message } => {
            // Must contain actual SQLite error
            assert!(
                message.contains("no such table") || message.contains("events"),
                "message must reflect actual error: {message}"
            );
            // Must NOT contain the DML rejection message
            assert!(
                !message.contains("only SELECT queries"),
                "must not be DML message: {message}"
            );
        }
        _ => panic!("expected ParseError, got {:?}", err),
    }
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `SELECT * FROM nonexistent_table` | `DataError::ParseError` with "no such table: 'nonexistent_table'" | error (EC-004) |
| `SELECT * FROM events` (table missing) | Actual rusqlite error, NOT DML message | error (EC-004, not EC-003) |
