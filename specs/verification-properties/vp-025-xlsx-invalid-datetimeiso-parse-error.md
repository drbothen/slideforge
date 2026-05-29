---
document_type: verification-property
vp_id: VP-025
title: "XLSX: invalid DateTimeIso string produces ParseError"
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

# VP-025: XLSX Invalid DateTimeIso String Produces ParseError

## Property Statement

For any calamine `Data::DateTimeIso(s)` cell where `s` fails ALL three ISO 8601 parse
attempts (RFC 3339, date-only, local datetime), the loader MUST return
`Err(DataError::ParseError)` with a message conforming to EC-009 format, citing the
cell coordinate and the offending string value. The string `s` MUST NOT be allowed to
pass through as `Value::Str`.

Formally: `!valid_iso8601(s) → load(Data::DateTimeIso(s)) == Err(DataError::ParseError { .. })`.

## Motivation

BC-1.03.006 AC-BC-004 (Item D). calamine claiming a cell is `DateTimeIso` is not
authoritative — some XLSX files contain cells with the DateTime type but malformed
or user-entered strings that are not valid ISO 8601. Passing such strings through
silently would allow corrupt data into templates, where it would produce silent wrong
output rather than a clear build error. Defense-in-depth requires validation even
when calamine has already classified the cell type.

## Feasibility Assessment

Not Kani-amenable: requires chrono parsing logic.

`kani_amenable: false` — string validation path.

## Test Harness

```rust
// crates/slideforge-data/src/tests/xlsx_datetime.rs
#[test]
fn test_invalid_datetimeiso_produces_parse_error() {
    let s = "not-a-date";
    let err = validate_and_load_datetimeiso(s).expect_err("invalid ISO should fail");
    match err {
        DataError::ParseError { message } => {
            assert!(message.contains("invalid ISO 8601"), "message: {message}");
            assert!(message.contains("not-a-date"), "message: {message}");
        }
        _ => panic!("expected ParseError"),
    }
}

#[test]
fn test_garbage_datetimeiso_produces_parse_error() {
    // calamine may produce DateTimeIso for any cell formatted as date
    for bad in &["garbage", "2024/01/15", "15-01-2024", "01-15-2024", ""] {
        let err = validate_and_load_datetimeiso(bad).expect_err("should fail");
        assert!(matches!(err, DataError::ParseError { .. }), "input: {bad}");
    }
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `Data::DateTimeIso("not-a-date")` | `DataError::ParseError` citing EC-009 format | error (EC-009) |
| `Data::DateTimeIso("")` | `DataError::ParseError` | error (EC-009) |
| `Data::DateTimeIso("2024/01/15")` (slash-separated) | `DataError::ParseError` | error (EC-009) |
