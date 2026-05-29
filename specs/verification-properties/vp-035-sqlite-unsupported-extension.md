---
document_type: verification-property
vp_id: VP-035
title: "SQLite: extension .db3 produces UnsupportedFormat"
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

# VP-035: SQLite Extension .db3 Produces UnsupportedFormat

## Property Statement

When the data source path has an extension NOT in the accepted set (`.db`, `.sqlite`,
`.sqlite3`), the loader MUST return `Err(DataError::UnsupportedFormat)` before opening
the file. Specifically, extensions such as `.db3`, `.s3db`, `.sl3`, or any other
extension MUST be rejected. The error message MUST cite the offending extension and the
accepted set.

Formally: `path.ext ∉ {.db, .sqlite, .sqlite3}` (case-insensitive)
→ `Err(DataError::UnsupportedFormat)` before file open.

## Motivation

BC-1.03.007 Invariant 8 and EC-010. The accepted extension set is CLOSED at `.db`,
`.sqlite`, and `.sqlite3`. Other common SQLite file extensions (`.db3`, `.s3db`, `.sl3`)
are deliberately rejected to ensure the user specifies a canonical path. This prevents
ambiguity and ensures the magic-byte check is only applied to files expected to be SQLite.

## Feasibility Assessment

Not Kani-amenable: extension checking involves path string manipulation before I/O.
The pure extension-check function is straightforward and could be unit-tested independently.

`kani_amenable: false` — path manipulation + file I/O context.

## Test Harness

```rust
// crates/slideforge-data/src/tests/sqlite_extension.rs
#[test]
fn test_db3_extension_produces_unsupported_format() {
    let path = std::path::Path::new("data.db3");
    let err = SqliteDataSource::check_extension(path).expect_err("should reject .db3");
    assert!(matches!(err, DataError::UnsupportedFormat { .. }));
}

#[test]
fn test_accepted_extensions_pass_check() {
    for ext in &["data.db", "data.sqlite", "data.sqlite3", "DATA.DB", "DATA.SQLITE"] {
        let path = std::path::Path::new(ext);
        assert!(
            SqliteDataSource::check_extension(path).is_ok(),
            "should accept extension in: {ext}"
        );
    }
}

#[test]
fn test_unknown_extension_produces_unsupported_format() {
    for bad_ext in &["data.json", "data.csv", "data.s3db", "data.sl3", "data"] {
        let path = std::path::Path::new(bad_ext);
        let err = SqliteDataSource::check_extension(path).expect_err("should reject: {bad_ext}");
        assert!(matches!(err, DataError::UnsupportedFormat { .. }), "input: {bad_ext}");
    }
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Path `"data.db3"` | `DataError::UnsupportedFormat` citing `.db3` and accepted set | error (EC-010) |
| Path `"data.s3db"` | `DataError::UnsupportedFormat` | error (EC-010) |
| Path `"data.db"` | `Ok(())` — accepted | happy-path |
| Path `"data.sqlite3"` | `Ok(())` — accepted | happy-path |
