---
document_type: verification-property
vp_id: VP-016
title: "XLSX: first row becomes map keys"
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

# VP-016: XLSX First Row Becomes Map Keys

## Property Statement

For any `.xlsx` file with a valid header row (all cells non-empty, String-typed), the
`XlsxDataSource::load()` function MUST bind the first row cell values as the key set
of every subsequent row map. Each subsequent row is represented as a
`HashMap<Arc<str>, Value>` keyed exactly by the header strings, in column order.

Formally: given header row `[h₀, h₁, ..., hₙ]` and data row at index `i`,
`result[i].keys() == {h₀, h₁, ..., hₙ}`.

## Motivation

BC-1.03.006 Postcondition 2 states "The first row is used as map keys (column headers)."
This is the foundational contract for all downstream `{{ name[0].ColumnHeader }}` access
patterns. If header extraction is incorrect, every data-driven slide silently produces
wrong output. The unit test uses a fixture XLSX to verify key set equality.

## Feasibility Assessment

Not Kani-amenable: `XlsxDataSource::load()` performs file I/O (reads XLSX bytes via
`calamine`). The property is verified via a deterministic unit test with a fixture XLSX
file created by `rust_xlsxwriter`. The fixture is reproducible and checked into the test
fixtures directory.

`kani_amenable: false` — file I/O crosses the pure/effectful boundary.

## Test Harness

```rust
// crates/slideforge-data/src/tests/xlsx_header.rs
#[test]
fn test_header_row_becomes_map_keys() {
    // Fixture: kpis.xlsx with header ["name", "value", "unit"] and 2 data rows
    let path = fixture_path("kpis.xlsx");
    let source = XlsxDataSource::new(&path, None);
    let rows = source.load().expect("load should succeed");
    assert_eq!(rows.len(), 2);
    for row in &rows {
        let keys: std::collections::BTreeSet<_> = row.keys().map(|k| k.as_ref()).collect();
        assert_eq!(keys, ["name", "unit", "value"].into_iter().collect());
    }
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `kpis.xlsx` headers `["name","value","unit"]`, 2 data rows | Each row map has exactly keys `name`, `value`, `unit` | happy-path |
| Single data row | `result.len() == 1`; keys match header | happy-path |
