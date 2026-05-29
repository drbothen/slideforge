---
document_type: verification-property
vp_id: VP-028
title: "SQLite: result row column names become map keys"
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

# VP-028: SQLite Result Row Column Names Become Map Keys

## Property Statement

For any successful `SELECT` query returning rows, each row in the result list MUST be
a `HashMap<Arc<str>, Value>` where the key set equals the column names (or aliases)
from the SELECT clause. Column ordering in the map keys matches the SELECT column order.

Formally: given `SELECT a, b AS alias FROM t` returning N rows,
`result[i].keys() == {"a", "alias"}` for all `i ∈ [0, N)`.

## Motivation

BC-1.03.007 Postcondition 2. This is the foundational property for all downstream
`{{ metrics[0].name }}` access patterns. If column name extraction is incorrect (e.g.,
using raw column indices instead of names), all template variable access breaks silently.

## Feasibility Assessment

Not Kani-amenable: requires a live SQLite query.

`kani_amenable: false` — SQLite I/O path.

## Test Harness

```rust
// crates/slideforge-data/src/tests/sqlite_keys.rs
#[test]
fn test_select_column_names_become_map_keys() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE metrics (name TEXT, value INTEGER);
                        INSERT INTO metrics VALUES ('revenue', 1000000);").unwrap();
    let path = /* temp file path backed by the in-memory db */ ...;

    let source = SqliteDataSource::new(&path, "SELECT name, value FROM metrics");
    let rows = source.load().expect("should succeed");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert!(row.contains_key("name"), "must have 'name' key");
    assert!(row.contains_key("value"), "must have 'value' key");
    assert_eq!(row.len(), 2, "must have exactly 2 keys");
}

#[test]
fn test_select_alias_becomes_map_key() {
    // SELECT name AS label FROM t — key must be 'label' not 'name'
    let source = SqliteDataSource::new(&path, "SELECT name AS label FROM metrics");
    let rows = source.load().expect("should succeed");
    assert!(rows[0].contains_key("label"), "alias must be map key");
    assert!(!rows[0].contains_key("name"), "original name must not appear");
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `SELECT name, value FROM metrics` (3 rows) | Each row map has keys `name`, `value` | happy-path |
| `SELECT name AS label FROM t` | Row map key is `label`, not `name` | alias coverage |
