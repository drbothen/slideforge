---
document_type: verification-property
vp_id: VP-018
title: "XLSX: missing file always produces E-DAT-004"
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

# VP-018: XLSX Missing File Always Produces E-DAT-004

## Property Statement

When `@data name from "path.xlsx"` is evaluated and the file at `path` does not exist
on the filesystem, the result MUST be `Err(DataError::FileNotFound)` (mapped to error
code E-DAT-004). The error MUST carry the missing path in its message. Build exit code
is 2.

Formally: `!path.exists() ↦ Err(DataError::FileNotFound { path })`.

## Motivation

BC-1.03.006 EC-001 states "E-DAT-004: file data source not found; exit 2". A missing
file must produce a named, actionable error rather than a panic or an opaque I/O error.
This property is exercised before any calamine parsing occurs — the file existence check
is the first gate.

## Feasibility Assessment

Not Kani-amenable: property depends on filesystem state (file existence). Verified via
unit test pointing at a guaranteed-nonexistent temp path.

`kani_amenable: false` — filesystem I/O.

## Test Harness

```rust
// crates/slideforge-data/src/tests/xlsx_errors.rs
#[test]
fn test_missing_file_produces_edat004() {
    let path = std::path::Path::new("/tmp/slideforge_test_nonexistent_abc123.xlsx");
    assert!(!path.exists());
    let source = XlsxDataSource::new(path, None);
    let err = source.load().expect_err("should fail with FileNotFound");
    assert!(matches!(err, DataError::FileNotFound { .. }));
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Path to nonexistent `.xlsx` file | `DataError::FileNotFound`; message contains path | error |
