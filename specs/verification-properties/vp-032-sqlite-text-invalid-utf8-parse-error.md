---
document_type: verification-property
vp_id: VP-032
title: "SQLite: TEXT column with invalid UTF-8 produces ParseError"
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

# VP-032: SQLite TEXT Column With Invalid UTF-8 Produces ParseError

## Property Statement

For any `rusqlite::types::ValueRef::Text(bytes)` where `std::str::from_utf8(bytes)`
returns `Err(_)`, the loader MUST return `Err(DataError::ParseError)` with a message
conforming to EC-008 (citing the column name and row index). The loader MUST NOT use
`String::from_utf8_lossy`, which would silently substitute U+FFFD replacement characters.

Formally: `ValueRef::Text(bytes) ∧ from_utf8(bytes) == Err(_) → Err(DataError::ParseError { .. })`.

## Motivation

BC-1.03.007 AC-BC-006 (Item E) and Invariant 6. `String::from_utf8_lossy` is
FORBIDDEN for TEXT column decoding. The canonical principle (CLAUDE.md: "no silent
fallback") requires that invalid UTF-8 in a TEXT column produce a visible build error
rather than silently corrupted data (U+FFFD in the output). SQLite TEXT columns are
spec'd to contain valid UTF-8 by the SQLite documentation, so invalid bytes indicate
data corruption or a misuse of the BLOB type — the build must fail.

## Feasibility Assessment

Not Kani-amenable: requires injecting raw non-UTF-8 bytes into a SQLite TEXT cell.
This requires using `rusqlite`'s raw value injection or a BLOB-to-TEXT cast at the
engine level.

`kani_amenable: false` — SQLite I/O with raw byte injection.

## Test Harness

```rust
// crates/slideforge-data/src/tests/sqlite_text_invalid_utf8.rs
#[test]
fn test_invalid_utf8_text_column_produces_parse_error() {
    // SQLite allows raw byte sequences in TEXT columns at the C API level.
    // Use rusqlite's raw `execute` with BLOB bytes reinterpreted as TEXT.
    // Byte sequence [0xC0, 0x80] is a well-known overlong encoding (invalid UTF-8).
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE t (msg BLOB);").unwrap();
    // Insert invalid UTF-8 bytes
    conn.execute("INSERT INTO t VALUES (?)", [&[0xC0u8, 0x80u8] as &[u8]]).unwrap();
    let path = save_to_temp(&conn);

    // When rusqlite returns this as Text bytes (via raw query), loader must fail
    let source = SqliteDataSource::new(&path, "SELECT CAST(msg AS TEXT) as msg FROM t");
    let err = source.load().expect_err("invalid UTF-8 must produce ParseError");
    match err {
        DataError::ParseError { message } => {
            assert!(
                message.contains("invalid UTF-8") || message.contains("UTF-8"),
                "message: {message}"
            );
        }
        _ => panic!("expected ParseError, got {:?}", err),
    }
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `ValueRef::Text([0xC0, 0x80])` (overlong encoding) | `DataError::ParseError` citing column name + row | error (EC-008) |
| `ValueRef::Text([0xFF, 0xFE])` (invalid sequence) | `DataError::ParseError` | error (EC-008) |
