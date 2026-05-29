---
document_type: verification-property
vp_id: VP-026
title: "XLSX: correct extension + wrong magic bytes produces ParseError"
module: slideforge-data
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: false
bc_trace: [BC-1.03.006]
anchored_to_bc: BC-1.03.006
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-026: XLSX Correct Extension + Wrong Magic Bytes Produces ParseError

## Property Statement

When a file has a `.xlsx` extension but its first 4 bytes are NOT `[0x50, 0x4B, 0x03, 0x04]`
(the ZIP local file header magic), the loader MUST return `Err(DataError::ParseError)`
with a message conforming to EC-013 (citing the path, the extension, and indicating
the ZIP magic bytes were not found). The loader MUST NOT proceed to pass the file to
calamine, which would produce an opaque internal error.

Formally: `path.ext == ".xlsx" ∧ file_bytes[0..4] != [0x50, 0x4B, 0x03, 0x04]`
→ `Err(DataError::ParseError)` before calamine is invoked.

## Motivation

BC-1.03.006 AC-BC-005 (Item H) and Postcondition 9. The two-phase check (extension
first, then magic bytes) prevents calamine from producing confusing internal errors
when presented with corrupt, misnamed, or encrypted files. A user who accidentally
points `@data` at a CSV renamed to `.xlsx` gets a clear error message rather than a
cryptic ZIP decompression failure.

## Feasibility Assessment

Not Kani-amenable: requires filesystem I/O (writing a non-ZIP file, then reading it).
Verified via a unit test that creates a temp file with `.xlsx` extension containing
non-ZIP content.

`kani_amenable: false` — filesystem I/O path.

## Test Harness

```rust
// crates/slideforge-data/src/tests/xlsx_magic.rs
#[test]
fn test_xlsx_wrong_magic_bytes_produces_parse_error() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fake.xlsx");
    let mut f = std::fs::File::create(&path).unwrap();
    // Write non-ZIP content (PNG magic bytes, for example)
    f.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]).unwrap();
    drop(f);

    let source = XlsxDataSource::new(&path, None);
    let err = source.load().expect_err("should fail with ParseError");
    match err {
        DataError::ParseError { message } => {
            assert!(
                message.contains("xlsx") || message.contains("XLSX"),
                "message: {message}"
            );
            assert!(
                message.contains("ZIP") || message.contains("magic"),
                "message: {message}"
            );
        }
        _ => panic!("expected ParseError, got {:?}", err),
    }
}

#[test]
fn test_xlsx_correct_magic_bytes_proceeds_to_parse() {
    // A real XLSX file (fixture) must NOT be rejected by magic-byte check
    let path = fixture_path("kpis.xlsx");
    let source = XlsxDataSource::new(&path, None);
    // Should succeed (or fail for a different reason — not magic bytes)
    // Magic-byte check must not false-positive on valid XLSX
    let result = source.load();
    assert!(result.is_ok(), "valid XLSX should pass magic check: {:?}", result.err());
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `.xlsx` file with PNG header bytes | `DataError::ParseError` citing ZIP magic check | error (EC-013) |
| `.xlsx` file with all-zero bytes | `DataError::ParseError` | error (EC-013) |
| Valid `.xlsx` file (ZIP structure) | Proceeds normally; no magic-byte rejection | regression |
