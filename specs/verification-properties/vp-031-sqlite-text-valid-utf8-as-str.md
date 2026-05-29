---
document_type: verification-property
vp_id: VP-031
title: "SQLite: TEXT column with valid UTF-8 produces Value::Str"
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

# VP-031: SQLite TEXT Column With Valid UTF-8 Produces Value::Str

## Property Statement

For any `rusqlite::types::ValueRef::Text(bytes)` where `std::str::from_utf8(bytes)`
returns `Ok(s)`, the loader MUST produce `Value::Str(Arc::from(s))`. The decode MUST
use `std::str::from_utf8`, NOT `String::from_utf8_lossy`.

Formally: `ValueRef::Text(bytes) ∧ from_utf8(bytes) == Ok(s) → Value::Str(Arc::from(s))`.

## Motivation

BC-1.03.007 AC-BC-006 (Item E) and Postcondition 4 (Text case). Valid UTF-8 text
passes through as a string. This is the happy-path complement to VP-032 (invalid UTF-8
produces ParseError). Together they specify the complete behavior of TEXT column handling.

## Feasibility Assessment

Not Kani-amenable: requires a live SQLite TEXT value.

`kani_amenable: false` — SQLite I/O path.

## Test Harness

```rust
// crates/slideforge-data/src/tests/sqlite_text.rs
#[test]
fn test_valid_utf8_text_produces_value_str() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE t (msg TEXT); INSERT INTO t VALUES ('hello world');").unwrap();
    let path = save_to_temp(&conn);
    let source = SqliteDataSource::new(&path, "SELECT msg FROM t");
    let rows = source.load().expect("should succeed");
    assert_eq!(rows[0].get("msg"), Some(&Value::Str(Arc::from("hello world"))));
}

#[test]
fn test_unicode_text_passes_through() {
    // Non-ASCII valid UTF-8: emoji and CJK
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE t (msg TEXT); INSERT INTO t VALUES ('こんにちは 🌸');").unwrap();
    let path = save_to_temp(&conn);
    let source = SqliteDataSource::new(&path, "SELECT msg FROM t");
    let rows = source.load().expect("should succeed");
    assert_eq!(rows[0].get("msg"), Some(&Value::Str(Arc::from("こんにちは 🌸"))));
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `ValueRef::Text(b"hello world")` | `Value::Str(Arc::from("hello world"))` | happy-path |
| `ValueRef::Text("こんにちは 🌸".as_bytes())` | `Value::Str(Arc::from("こんにちは 🌸"))` | unicode |
