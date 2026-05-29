---
document_type: verification-property
vp_id: VP-021
title: "XLSX: whole-number Float 95.0 loads as Value::Int(95)"
module: slideforge-data
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: true
bc_trace: [BC-1.03.006, DI-004]
anchored_to_bc: BC-1.03.006
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-021: XLSX Whole-Number Float Promotion to Value::Int

## Property Statement

For any calamine `Data::Float(f)` value in a data row cell (row index > 0), the
loading function MUST apply the following promotion rule:
- If `f.fract() == 0.0 && f.is_finite() && f >= i64::MIN as f64 && f <= i64::MAX as f64`
  → `Value::Int(f as i64)`
- If `f.is_nan() || f.is_infinite()` → `Err(DataError::ParseError)` (EC-012)
- Otherwise → `Value::Float(OrderedFloat(f))`

No `Data::Float` value may unconditionally produce `Value::Float`. The promotion path
MUST be taken for every whole-numbered finite float in i64 range.

Formally: `∀ f: f64, f.fract() == 0.0 ∧ f.is_finite() ∧ f ∈ [i64::MIN as f64, i64::MAX as f64]`
→ `promote(Data::Float(f)) == Value::Int(f as i64)`.

## Motivation

BC-1.03.006 AC-BC-003 (Item C) and Invariant 7. calamine surfaces integer cells from
some XLSX files as `Data::Float(95.0)` (e.g., when `rust_xlsxwriter` writes an i32 cell).
Without this promotion rule, `95` written in Excel appears as `Value::Float(95.0)` in
slideforge templates, breaking expressions like `{{ value + 1 }}` (float + int type mismatch)
and snapshot tests that expect `Int(95)`.

## Feasibility Assessment

Kani-amenable: the promotion function `promote_float(f: f64) -> Result<Value, DataError>` is
pure — it takes a float value and returns a Value without any I/O. Kani can symbolically
verify that every f64 in [i64::MIN, i64::MAX] with zero fractional part maps to `Value::Int`.
Kani cannot directly reason over all f64 values but can be bounded over the critical
transition points using `kani::any::<i64>() as f64` to generate whole-number inputs.

`kani_amenable: true` — the promotion logic is a pure function.

## Proof Harness Skeleton

```rust
// crates/slideforge-data/src/proofs/xlsx_float_promotion.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn whole_number_float_promotes_to_int() {
        // Generate any i64 and cast to f64 — this produces a whole-number float
        let n: i64 = kani::any();
        let f = n as f64;
        // If representable back to i64 without loss (i.e., f as i64 == n for typical values)
        // the promotion must produce Value::Int(n)
        kani::assume(f.is_finite() && f.fract() == 0.0);
        kani::assume(f >= i64::MIN as f64 && f <= i64::MAX as f64);
        let result = promote_float(f);
        kani::assert!(matches!(result, Ok(Value::Int(_))));
    }

    #[kani::proof]
    fn non_whole_float_stays_float() {
        // 3.14 has non-zero fractional part — must stay Float
        let result = promote_float(3.14_f64);
        kani::assert!(matches!(result, Ok(Value::Float(_))));
    }

    #[kani::proof]
    fn nan_float_produces_parse_error() {
        let result = promote_float(f64::NAN);
        kani::assert!(matches!(result, Err(DataError::ParseError { .. })));
    }

    #[kani::proof]
    fn infinite_float_produces_parse_error() {
        let result = promote_float(f64::INFINITY);
        kani::assert!(matches!(result, Err(DataError::ParseError { .. })));
    }
}
```

## Test Coverage (before Phase 6)

```rust
// crates/slideforge-data/src/tests/xlsx_float_promotion.rs
#[test]
fn test_whole_number_float_promotes_to_int() {
    assert_eq!(promote_float(95.0).unwrap(), Value::Int(95));
    assert_eq!(promote_float(0.0).unwrap(), Value::Int(0));
    assert_eq!(promote_float(-42.0).unwrap(), Value::Int(-42));
}

#[test]
fn test_non_whole_float_stays_float() {
    assert!(matches!(promote_float(3.14).unwrap(), Value::Float(_)));
}

#[test]
fn test_nan_float_produces_error() {
    assert!(promote_float(f64::NAN).is_err());
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `Data::Float(95.0)` | `Value::Int(95)` | regression (EC-010) |
| `Data::Float(3.14)` | `Value::Float(OrderedFloat(3.14))` | happy-path (EC-011) |
| `Data::Float(f64::NAN)` | `DataError::ParseError` | error (EC-012) |
| `Data::Float(f64::INFINITY)` | `DataError::ParseError` | error (EC-012) |
