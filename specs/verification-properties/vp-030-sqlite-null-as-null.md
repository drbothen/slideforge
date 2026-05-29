---
document_type: verification-property
vp_id: VP-030
title: "SQLite: NULL db values produce null in data value tree"
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

# VP-030: SQLite NULL DB Values Produce Null in Data Value Tree

## Property Statement

For any `rusqlite::types::ValueRef::Null` column value in a query result row, the
loaded `Value` MUST be `Value::Null` — not `Value::Str(Arc::from(""))`, not omitted,
and not substituted with a zero value or empty collection.

Formally: `ValueRef::Null ↦ Value::Null` in all rows.

## Motivation

BC-1.03.007 Postcondition 3. SQLite NULL semantics are semantically distinct from empty
string and zero. DI-004 prohibits coercion: `NULL → ""` or `NULL → 0` are both silent
type coercions. Templates using `{{ row.col == null }}` vs `{{ row.col == "" }}` depend
on this distinction.

## Feasibility Assessment

Not Kani-amenable: requires a live SQLite row with a NULL column.

`kani_amenable: false` — SQLite I/O.

## Test Harness

```rust
// crates/slideforge-data/src/tests/sqlite_null.rs
#[test]
fn test_sqlite_null_column_produces_value_null() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE t (a TEXT, b TEXT); INSERT INTO t VALUES ('hello', NULL);").unwrap();
    let path = save_to_temp(&conn);
    let source = SqliteDataSource::new(&path, "SELECT a, b FROM t");
    let rows = source.load().expect("should succeed");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get("a"), Some(&Value::Str(Arc::from("hello"))));
    assert_eq!(rows[0].get("b"), Some(&Value::Null));
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SQLite row with `NULL` column | `Value::Null` in loaded map | happy-path |
| SQLite row with `""` (empty string) | `Value::Str(Arc::from(""))` — not null | edge-case (nulls != empty strings) |
