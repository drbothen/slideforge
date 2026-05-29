---
document_type: verification-property
vp_id: VP-020
title: "XLSX: non-string header cell (Int) produces ParseError"
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

# VP-020: XLSX Non-String Header Cell Produces ParseError

## Property Statement

When a cell in the XLSX header row is parsed by calamine as any type other than
`Data::String` (specifically: `Data::Int`, `Data::Float`, `Data::Bool`, `Data::DateTime`,
or `Data::DateTimeIso`), `XlsxDataSource::load()` MUST return `Err(DataError::ParseError)`
with a message that names the calamine type and the column index. The implementer MUST NOT
call `.to_string()` on the non-string cell value and silently accept the result as a header.

Formally: `header[i] ∈ {Data::Int, Data::Float, Data::Bool, Data::DateTime, Data::DateTimeIso}`
→ `Err(DataError::ParseError)` with message citing column `i` and calamine type name.

## Motivation

BC-1.03.006 AC-BC-002 (Item B) and Invariant 6. DI-004 ("no implicit type coercion")
extends to header type checking: coercing an integer `2024` to string `"2024"` as a
column name is an implicit conversion. The forbidden pattern in CLAUDE.md states that
implicit type coercion — in any direction — is disallowed. Column headers that are not
explicitly authored as strings must be rejected rather than stringified.

## Feasibility Assessment

Not Kani-amenable: requires calamine to parse a fixture XLSX file with typed header cells.

`kani_amenable: false` — file I/O path.

## Test Harness

```rust
// crates/slideforge-data/src/tests/xlsx_header_errors.rs
#[test]
fn test_integer_header_cell_produces_parse_error() {
    // Fixture: int_header.xlsx — header row: [Int(2024), "value"]
    // Created with rust_xlsxwriter writing an integer cell in row 0, col 0
    let path = fixture_path("int_header.xlsx");
    let source = XlsxDataSource::new(&path, None);
    let err = source.load().expect_err("should fail");
    match err {
        DataError::ParseError { message } => {
            assert!(
                message.contains("column 0"),
                "must cite column index: {message}"
            );
            assert!(
                message.contains("Int") || message.contains("integer"),
                "must name calamine type: {message}"
            );
        }
        _ => panic!("expected ParseError, got {:?}", err),
    }
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| XLSX header cell is calamine `Data::Int(2024)` | `DataError::ParseError` with type `Int` and column 0 | error (EC-008) |
| XLSX header cell is calamine `Data::Float(3.14)` | `DataError::ParseError` with type `Float` and column index | error (EC-008) |
| XLSX header cell is calamine `Data::Bool(true)` | `DataError::ParseError` with type `Bool` and column index | error (EC-008) |
