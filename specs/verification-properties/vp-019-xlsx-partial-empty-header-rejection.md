---
document_type: verification-property
vp_id: VP-019
title: "XLSX: partial-empty header row produces ParseError (no phantom column)"
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

# VP-019: XLSX Partial-Empty Header Row Produces ParseError (No Phantom Column)

## Property Statement

When the XLSX header row contains one or more `Data::Empty` cells while at least one
other header cell is non-empty, `XlsxDataSource::load()` MUST return
`Err(DataError::ParseError)` with a message conforming to EC-007 format. The result
MUST NOT contain any key of the form `"__empty_<N>"` or any other phantom column name
invented by the loader.

Formally: `∃ i : header[i] == Data::Empty ∧ ∃ j≠i : header[j] ≠ Data::Empty`
→ `Err(DataError::ParseError)` with message citing column index `i`.

## Motivation

BC-1.03.006 AC-BC-001 (Item A) and Invariant 5 forbid phantom column name invention.
The canonical principle in CLAUDE.md ("no silent fallback") applies: inventing
`"__empty_0"` silently corrupts data access semantics without surfacing an authoring error.
DI-004 (no implicit type coercion) extends to naming — inferring a column name where none
was provided is a form of coercion.

## Feasibility Assessment

Not Kani-amenable: verifying the absence of phantom keys requires checking result data
structure content after file I/O.

`kani_amenable: false` — I/O path; property verified by fixture test.

## Test Harness

```rust
// crates/slideforge-data/src/tests/xlsx_header_errors.rs
#[test]
fn test_partial_empty_header_produces_parse_error_no_phantom() {
    // Fixture: partial_header.xlsx — header row: ["name", <empty>, "score"]
    let path = fixture_path("partial_header.xlsx");
    let source = XlsxDataSource::new(&path, None);
    let err = source.load().expect_err("should fail with ParseError");
    match err {
        DataError::ParseError { message } => {
            assert!(message.contains("empty cell"), "message: {message}");
            assert!(message.contains("column 1"), "message: {message}");
            // Crucially: no phantom key was produced
        }
        _ => panic!("expected ParseError, got {:?}", err),
    }
}

#[test]
fn test_partial_empty_header_no_phantom_key_in_error_path() {
    // Ensure the error path does not produce any data with phantom keys
    let path = fixture_path("partial_header.xlsx");
    let source = XlsxDataSource::new(&path, None);
    let result = source.load();
    assert!(result.is_err());
    // If it somehow succeeds (it must not), check for phantom keys
    if let Ok(rows) = result {
        for row in &rows {
            for key in row.keys() {
                assert!(!key.starts_with("__empty"), "phantom key found: {key}");
            }
        }
    }
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| XLSX header `["name", <empty>, "score"]` | `DataError::ParseError` citing column 1; no `__empty_1` key | error (EC-007) |
| XLSX header `[<empty>, "value"]` | `DataError::ParseError` citing column 0 | error (EC-007) |
