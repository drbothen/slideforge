---
document_type: verification-property
vp_id: VP-034
title: "SQLite: file with .db extension but non-SQLite magic bytes produces ParseError"
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

# VP-034: SQLite File With Accepted Extension But Non-SQLite Magic Bytes Produces ParseError

## Property Statement

When a file has an accepted extension (`.db`, `.sqlite`, or `.sqlite3`) but its first
16 bytes do NOT match `b"SQLite format 3\0"` (the canonical SQLite file header),
`SqliteDataSource::load()` MUST return `Err(DataError::ParseError)` with a message
conforming to EC-009 (citing the path, the extension, and "SQLite file header not found").
The loader MUST NOT proceed to rusqlite open, which would produce confusing internal errors.

Formally: `path.ext ∈ {.db, .sqlite, .sqlite3} ∧ file_bytes[0..16] != b"SQLite format 3\0"`
→ `Err(DataError::ParseError)` before rusqlite is invoked.

## Motivation

BC-1.03.007 AC-BC-008 (Item H) and Postcondition 7. The two-phase check (extension,
then magic bytes) maintains symmetry with XLSX's approach (BC-1.03.006 VP-026) and
prevents rusqlite from producing cryptic errors when given a corrupt or misnamed file.
A user pointing `@data` at a JSON file renamed to `.db` gets a clear, actionable error.

## Feasibility Assessment

Not Kani-amenable: requires filesystem I/O (writing a non-SQLite file with a `.db` extension).

`kani_amenable: false` — filesystem I/O.

## Test Harness

```rust
// crates/slideforge-data/src/tests/sqlite_magic.rs
#[test]
fn test_db_file_wrong_magic_bytes_produces_parse_error() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fake.db");
    let mut f = std::fs::File::create(&path).unwrap();
    // Write PNG magic bytes (definitely not SQLite)
    f.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
                  0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52]).unwrap();
    drop(f);

    let source = SqliteDataSource::new(&path, "SELECT 1");
    let err = source.load().expect_err("wrong magic should produce ParseError");
    match err {
        DataError::ParseError { message } => {
            assert!(
                message.contains("SQLite") || message.contains("sqlite"),
                "message must cite SQLite: {message}"
            );
            assert!(
                message.contains("header") || message.contains("magic"),
                "message must cite header/magic: {message}"
            );
        }
        _ => panic!("expected ParseError, got {:?}", err),
    }
}

#[test]
fn test_valid_sqlite_db_passes_magic_check() {
    // A genuine SQLite file must NOT be rejected by the magic-byte check
    let path = fixture_path("test.db");
    let source = SqliteDataSource::new(&path, "SELECT 1");
    // Should succeed or fail for a non-magic-byte reason
    // Magic-byte check must not false-positive on valid SQLite
    let result = source.load();
    // The query "SELECT 1" should succeed on any valid SQLite file
    assert!(result.is_ok(), "valid SQLite should pass magic check: {:?}", result.err());
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `.db` file with PNG magic bytes | `DataError::ParseError` citing SQLite header | error (EC-009) |
| `.sqlite` file with all-zero bytes | `DataError::ParseError` | error (EC-009) |
| Valid SQLite `.db` file | Proceeds normally; no rejection | regression |
