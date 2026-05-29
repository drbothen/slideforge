---
document_type: verification-property
vp_id: VP-024
title: "XLSX: valid DateTimeIso passes through as Value::Str"
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

# VP-024: XLSX Valid DateTimeIso Passes Through as Value::Str

## Property Statement

For any calamine `Data::DateTimeIso(s)` cell where `s` is a valid ISO 8601 date/time
string (parseable by at least one of: `chrono::DateTime::parse_from_rfc3339(s)`,
`chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")`, or
`chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")`), the loader MUST
produce `Value::Str(Arc::from(s))` — the original string is preserved verbatim.

Formally: `valid_iso8601(s) → load(Data::DateTimeIso(s)) == Ok(Value::Str(Arc::from(s)))`.

## Motivation

BC-1.03.006 AC-BC-004 (Item D) and Postcondition 6. calamine's `DateTimeIso` variant
indicates the cell was formatted as a date/time in the source XLSX. Valid ISO 8601
strings pass through as `Value::Str` to preserve round-trip fidelity in templates
(`{{ row.date }}` renders the date string the user wrote). Conversion to a `DateTime`
object is not part of the v1.0 contract; string preservation is.

## Feasibility Assessment

Not Kani-amenable: validation requires calling chrono parsing functions which involve
complex string logic not amenable to Kani bounded model checking within reasonable
bounds.

`kani_amenable: false` — chrono parsing is effectful from Kani's perspective (complex
string parsing).

## Test Harness

```rust
// crates/slideforge-data/src/tests/xlsx_datetime.rs
#[test]
fn test_valid_datetimeiso_rfc3339_passes_through() {
    let s = "2024-01-15T09:30:00+00:00";
    let result = validate_and_load_datetimeiso(s).expect("valid RFC 3339 should succeed");
    assert_eq!(result, Value::Str(Arc::from(s)));
}

#[test]
fn test_valid_datetimeiso_date_only_passes_through() {
    let s = "2024-01-15";
    let result = validate_and_load_datetimeiso(s).expect("valid date-only should succeed");
    assert_eq!(result, Value::Str(Arc::from(s)));
}

#[test]
fn test_valid_datetimeiso_local_datetime_passes_through() {
    let s = "2024-01-15T09:30:00";
    let result = validate_and_load_datetimeiso(s).expect("valid local datetime should succeed");
    assert_eq!(result, Value::Str(Arc::from(s)));
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `Data::DateTimeIso("2024-01-15T09:30:00+00:00")` | `Value::Str("2024-01-15T09:30:00+00:00")` | happy-path |
| `Data::DateTimeIso("2024-01-15")` | `Value::Str("2024-01-15")` | happy-path |
| `Data::DateTimeIso("2024-01-15T09:30:00")` | `Value::Str("2024-01-15T09:30:00")` | happy-path |
