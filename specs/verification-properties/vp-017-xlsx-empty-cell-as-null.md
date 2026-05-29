---
document_type: verification-property
vp_id: VP-017
title: "XLSX: empty cells produce null not empty string"
module: slideforge-data
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: false
bc_trace: [BC-1.03.006, DI-004]
anchored_to_bc: BC-1.03.006
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-017: XLSX Empty Cells Produce Null Not Empty String

## Property Statement

For any calamine `Data::Empty` cell in a data row (row index > 0), the loaded
`Value` MUST be `Value::Null` — not `Value::Str(Arc::from(""))`, not omitted,
and not substituted with any phantom value.

Formally: `calamine::Data::Empty ↦ Value::Null` for all data row cells.

## Motivation

BC-1.03.006 Postcondition 4 states "Empty cells are loaded as null (not empty string,
not omitted)." This distinction matters for downstream template logic:
`{{ row.field == null }}` and `{{ row.field == "" }}` are semantically different.
DI-004 ("no implicit type coercion") prohibits promoting empty to empty-string.

## Feasibility Assessment

Not Kani-amenable: `XlsxDataSource::load()` involves calamine file parsing (I/O path).
Verified via a fixture XLSX that contains intentionally empty cells in data rows.

`kani_amenable: false` — file I/O boundary.

## Test Harness

```rust
// crates/slideforge-data/src/tests/xlsx_null.rs
#[test]
fn test_empty_cell_produces_null() {
    // Fixture: sparse.xlsx — header ["col_a","col_b"], row 1 = ["hello", <empty>]
    let path = fixture_path("sparse.xlsx");
    let source = XlsxDataSource::new(&path, None);
    let rows = source.load().expect("load should succeed");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.get("col_a"), Some(&Value::Str(Arc::from("hello"))));
    assert_eq!(row.get("col_b"), Some(&Value::Null));
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| XLSX data cell `Data::Empty` | `Value::Null` in loaded map | happy-path |
| XLSX data cell `Data::String("")` | `Value::Str(Arc::from(""))` — not null (empty string != empty cell) | edge-case |
