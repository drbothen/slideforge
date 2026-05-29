---
document_type: verification-property
vp_id: VP-029
title: "SQLite: DML in query: field produces E-DAT-003"
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

# VP-029: SQLite DML in Query Field Produces E-DAT-003

## Property Statement

When the `query:` field starts with a DML keyword (`DELETE`, `INSERT`, `UPDATE`,
`DROP`, `CREATE`) after trimming leading whitespace (case-insensitive), the loader
MUST return `Err(DataError::ParseError)` with message `"only SELECT queries are
allowed in @data sqlite sources"` and MUST NOT execute the query against the database.

Formally: `trim(query).to_uppercase().starts_with(K)` where `K ∈ {DELETE, INSERT, UPDATE, DROP, CREATE}`
→ `Err(DataError::ParseError)` with DML-rejection message BEFORE any `connection.prepare()` call.

## Motivation

BC-1.03.007 AC-BC-009 (Item I) and Invariant 5. The DML-rejection check must fire
BEFORE any database access to prevent a time-of-check/time-of-use race and to produce
the correct error message. Non-DML query failures (table not found, syntax error) must
produce the ACTUAL rusqlite error message, not the DML message — this differentiation
was explicitly codified as AC-BC-009 to fix ambiguity identified in adversary pass 1 (F-MED-2).

## Feasibility Assessment

Not Kani-amenable: check depends on string prefix matching and database state.

`kani_amenable: false` — string-matching + I/O path.

## Test Harness

```rust
// crates/slideforge-data/src/tests/sqlite_dml.rs
#[test]
fn test_delete_query_produces_dml_error() {
    let source = SqliteDataSource::new(&fixture_path("test.db"), "DELETE FROM events");
    let err = source.load().expect_err("DML should be rejected");
    match err {
        DataError::ParseError { message } => {
            assert!(message.contains("only SELECT queries"), "message: {message}");
        }
        _ => panic!("expected ParseError"),
    }
}

#[test]
fn test_insert_query_produces_dml_error() {
    let source = SqliteDataSource::new(&fixture_path("test.db"), "INSERT INTO t VALUES (1)");
    let err = source.load().expect_err("DML should be rejected");
    assert!(matches!(err, DataError::ParseError { .. }));
}

#[test]
fn test_case_insensitive_dml_detection() {
    for prefix in &["delete from t", "DELETE FROM t", "  DELETE FROM t", "Delete From t"] {
        let source = SqliteDataSource::new(&fixture_path("test.db"), prefix);
        let err = source.load().expect_err("DML should be rejected regardless of case");
        assert!(matches!(err, DataError::ParseError { .. }), "prefix: {prefix}");
    }
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `"DELETE FROM events"` | `DataError::ParseError`: "only SELECT queries allowed" | error (EC-003) |
| `"INSERT INTO t VALUES (1)"` | `DataError::ParseError`: "only SELECT queries allowed" | error (EC-003) |
| `"  delete from t"` (leading spaces, lowercase) | Same DML error | error (EC-003) |
| `"UPDATE t SET x=1"` | Same DML error | error (EC-003) |
