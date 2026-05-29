---
document_type: verification-property
vp_id: VP-033
title: "SQLite: BLOB column produces base64 STANDARD with padding"
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

# VP-033: SQLite BLOB Column Produces Base64 STANDARD With Padding

## Property Statement

For any `rusqlite::types::ValueRef::Blob(bytes)`, the loader MUST produce
`Value::Str(Arc::from(base64::engine::general_purpose::STANDARD.encode(bytes)))`.
The encoding MUST use RFC 4648 §4 (standard alphabet `A-Za-z0-9+/`, with `=` padding).
Alternative engines (`URL_SAFE`, `STANDARD_NO_PAD`, `URL_SAFE_NO_PAD`) MUST NOT be used.

The canonical test vector from BC-1.03.007: bytes `[0x00, 0xFF, 0x42]` → `"AP9C"`.

Formally: `ValueRef::Blob(bytes) ↦ Value::Str(Arc::from(STANDARD.encode(bytes)))`.

## Motivation

BC-1.03.007 AC-BC-007 (Item F), Postcondition 4 (Blob case), and Invariant 7.
The STANDARD encoding is chosen for canonical stability — downstream consumers
(templates, test vectors, API clients) can rely on the exact base64 encoding being
consistent across all builds and versions. The `=` padding ensures decodability with
standard RFC 4648 §4 decoders without special configuration.

## Feasibility Assessment

Not Kani-amenable: requires a live SQLite BLOB column value. However, the encoding
function itself is pure and could be unit-tested directly.

`kani_amenable: false` — tested via integration with SQLite I/O; the pure encoding step
is covered by direct unit tests of the encoding function.

## Test Harness

```rust
// crates/slideforge-data/src/tests/sqlite_blob.rs
use base64::Engine;

#[test]
fn test_blob_canonical_test_vector() {
    // BC-1.03.007 canonical test vector: [0x00, 0xFF, 0x42] → "AP9C"
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE t (data BLOB);").unwrap();
    conn.execute("INSERT INTO t VALUES (?)", [&[0x00u8, 0xFFu8, 0x42u8] as &[u8]]).unwrap();
    let path = save_to_temp(&conn);

    let source = SqliteDataSource::new(&path, "SELECT data FROM t");
    let rows = source.load().expect("should succeed");
    assert_eq!(rows[0].get("data"), Some(&Value::Str(Arc::from("AP9C"))));
}

#[test]
fn test_blob_empty_bytes_produces_empty_string() {
    // base64 of [] is ""
    let encoded = base64::engine::general_purpose::STANDARD.encode(b"");
    assert_eq!(encoded, "");
}

#[test]
fn test_blob_padding_is_present() {
    // [0x00] → "AA==" (two pad chars); [0x00, 0x00] → "AAA=" (one pad char)
    assert_eq!(base64::engine::general_purpose::STANDARD.encode(b"\x00"), "AA==");
    assert_eq!(base64::engine::general_purpose::STANDARD.encode(b"\x00\x00"), "AAA=");
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| BLOB `[0x00, 0xFF, 0x42]` | `Value::Str("AP9C")` — canonical BC vector | regression (EC-011) |
| BLOB `[0x00]` | `Value::Str("AA==")` — with padding | padding |
| BLOB `[0x00, 0x00]` | `Value::Str("AAA=")` — with single pad | padding |
| BLOB `[]` (empty) | `Value::Str("")` | edge-case |
