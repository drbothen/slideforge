//! Built-in filter functions for the slideforge pipe operator.
//!
//! Filters are applied via the `|` operator in DSL expressions:
//!
//! ```text
//! {{ amount | currency }}
//! {{ name | upper }}
//! {{ items | join(", ") }}
//! ```
//!
//! The public entry point is [`apply_filter`], which dispatches to the
//! appropriate filter implementation by name.
//!
//! # Filter Registry
//!
//! | Filter name    | Input type(s)       | Output type |
//! |---------------|---------------------|-------------|
//! | `upper`        | Str                 | Str         |
//! | `lower`        | Str                 | Str         |
//! | `currency`     | Int or Float        | Str         |
//! | `round`        | Float, arg: Int     | Str         |
//! | `int`          | Str or Float        | Int         |
//! | `float`        | Str or Int          | Float       |
//! | `string`       | Any                 | Str         |
//! | `join`         | List, arg: Str      | Str         |
//! | `length`       | Str or List         | Int         |
//! | `trim`         | Str                 | Str         |
//! | `default`      | Any, arg: Any       | Any         |
//! | `contains`     | Str, arg: Str       | Bool        |
//! | `starts_with`  | Str, arg: Str       | Bool        |
//! | `ends_with`    | Str, arg: Str       | Bool        |
//! | `replace`      | Str, args: Str×2   | Str         |

use std::sync::Arc;

use slideforge_types::{SourceSpan, Value};

use crate::error::EvalError;

// ─── Public list of available filter names (used in error messages) ───────────

/// Sorted list of all built-in filter names, used to construct
/// `EvalError::FilterNotFound.available` messages.
pub const AVAILABLE_FILTERS: &[&str] = &[
    "contains",
    "currency",
    "default",
    "ends_with",
    "float",
    "int",
    "join",
    "length",
    "lower",
    "replace",
    "round",
    "starts_with",
    "string",
    "trim",
    "upper",
];

// ─── Dispatch ────────────────────────────────────────────────────────────────

/// Apply a built-in filter by name.
///
/// Returns `Err(EvalError::FilterNotFound)` when `name` is not in the
/// built-in registry, and other [`EvalError`] variants when the value or
/// arguments have wrong types.
///
/// # Errors
///
/// Returns [`EvalError::FilterNotFound`] if `name` is unrecognised,
/// [`EvalError::TypeMismatch`] if the input or argument types are wrong.
pub fn apply_filter(
    name: &str,
    val: &Value,
    args: &[Value],
    span: SourceSpan,
) -> Result<Value, EvalError> {
    match name {
        "upper" => filter_upper(val, span),
        "lower" => filter_lower(val, span),
        "currency" => filter_currency(val, span),
        "round" => filter_round(val, args, span),
        "int" => filter_int(val, span),
        "float" => filter_float(val, span),
        "string" => filter_string(val, span),
        "join" => filter_join(val, args, span),
        "length" => filter_length(val, span),
        "trim" => filter_trim(val, span),
        "default" => filter_default(val, args, span),
        "contains" => filter_contains(val, args, span),
        "starts_with" => filter_starts_with(val, args, span),
        "ends_with" => filter_ends_with(val, args, span),
        "replace" => filter_replace(val, args, span),
        _ => Err(EvalError::FilterNotFound {
            name: Arc::from(name),
            available: AVAILABLE_FILTERS.join(", "),
            span,
        }),
    }
}

// ─── Individual filter implementations (stubs — Red Gate) ────────────────────

/// Convert a string value to uppercase.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not a [`Value::Str`].
fn filter_upper(val: &Value, span: SourceSpan) -> Result<Value, EvalError> {
    let _ = (val, span);
    todo!("STORY-011: implement filter_upper")
}

/// Convert a string value to lowercase.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not a [`Value::Str`].
fn filter_lower(val: &Value, span: SourceSpan) -> Result<Value, EvalError> {
    let _ = (val, span);
    todo!("STORY-011: implement filter_lower")
}

/// Format a numeric value as a comma-separated currency string with two
/// decimal places (e.g. `1234.5` → `"1,234.50"`).
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not [`Value::Int`] or
/// [`Value::Float`].
fn filter_currency(val: &Value, span: SourceSpan) -> Result<Value, EvalError> {
    let _ = (val, span);
    todo!("STORY-011: implement filter_currency")
}

/// Round a float value to `args[0]` (Int) decimal places and render as a
/// string.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not a [`Value::Float`] or
/// if `args[0]` is not a [`Value::Int`].
fn filter_round(val: &Value, args: &[Value], span: SourceSpan) -> Result<Value, EvalError> {
    let _ = (val, args, span);
    todo!("STORY-011: implement filter_round")
}

/// Parse a string or float value as an integer.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not parseable as `Int`.
fn filter_int(val: &Value, span: SourceSpan) -> Result<Value, EvalError> {
    let _ = (val, span);
    todo!("STORY-011: implement filter_int")
}

/// Parse a string or integer value as a float.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not parseable as `Float`.
fn filter_float(val: &Value, span: SourceSpan) -> Result<Value, EvalError> {
    let _ = (val, span);
    todo!("STORY-011: implement filter_float")
}

/// Convert any value to its string representation.
///
/// # Errors
///
/// This filter does not error — every [`Value`] has a string representation.
fn filter_string(val: &Value, span: SourceSpan) -> Result<Value, EvalError> {
    let _ = (val, span);
    todo!("STORY-011: implement filter_string")
}

/// Join list elements with a separator string.
///
/// `args[0]` is the separator. Each list element is converted to a string
/// via its `string` filter equivalent before joining.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not a [`Value::List`] or
/// `args[0]` is not a [`Value::Str`].
fn filter_join(val: &Value, args: &[Value], span: SourceSpan) -> Result<Value, EvalError> {
    let _ = (val, args, span);
    todo!("STORY-011: implement filter_join")
}

/// Return the length of a string (character count) or list (element count).
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not a [`Value::Str`] or
/// [`Value::List`].
fn filter_length(val: &Value, span: SourceSpan) -> Result<Value, EvalError> {
    let _ = (val, span);
    todo!("STORY-011: implement filter_length")
}

/// Strip leading and trailing ASCII whitespace from a string.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not a [`Value::Str`].
fn filter_trim(val: &Value, span: SourceSpan) -> Result<Value, EvalError> {
    let _ = (val, span);
    todo!("STORY-011: implement filter_trim")
}

/// Return `val` if it is non-null, otherwise return `args[0]`.
///
/// This filter never produces a type error — it is defined for all value types.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] only if `args` is empty (no fallback
/// provided).
fn filter_default(val: &Value, args: &[Value], span: SourceSpan) -> Result<Value, EvalError> {
    let _ = (val, args, span);
    todo!("STORY-011: implement filter_default")
}

/// Return `Bool(true)` if the string `val` contains the substring `args[0]`.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` or `args[0]` is not a
/// [`Value::Str`].
fn filter_contains(val: &Value, args: &[Value], span: SourceSpan) -> Result<Value, EvalError> {
    let _ = (val, args, span);
    todo!("STORY-011: implement filter_contains")
}

/// Return `Bool(true)` if string `val` starts with the prefix `args[0]`.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` or `args[0]` is not a
/// [`Value::Str`].
fn filter_starts_with(
    val: &Value,
    args: &[Value],
    span: SourceSpan,
) -> Result<Value, EvalError> {
    let _ = (val, args, span);
    todo!("STORY-011: implement filter_starts_with")
}

/// Return `Bool(true)` if string `val` ends with the suffix `args[0]`.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` or `args[0]` is not a
/// [`Value::Str`].
fn filter_ends_with(val: &Value, args: &[Value], span: SourceSpan) -> Result<Value, EvalError> {
    let _ = (val, args, span);
    todo!("STORY-011: implement filter_ends_with")
}

/// Replace all occurrences of substring `args[0]` with `args[1]` in `val`.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val`, `args[0]`, or `args[1]` is
/// not a [`Value::Str`].
fn filter_replace(val: &Value, args: &[Value], span: SourceSpan) -> Result<Value, EvalError> {
    let _ = (val, args, span);
    todo!("STORY-011: implement filter_replace")
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;

    fn span() -> SourceSpan {
        SourceSpan::default()
    }

    // ── filter_upper ─────────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_upper")]
    fn test_filter_upper() {
        let val = Value::Str(Arc::from("hello"));
        let result = apply_filter("upper", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("HELLO")));
    }

    // ── filter_lower ─────────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_lower")]
    fn test_filter_lower() {
        let val = Value::Str(Arc::from("WORLD"));
        let result = apply_filter("lower", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("world")));
    }

    // ── filter_currency ───────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_currency")]
    fn test_filter_currency() {
        let val = Value::Int(1234);
        let result = apply_filter("currency", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("1,234.00")));
    }

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_currency")]
    fn test_filter_currency_float() {
        let val = Value::Float(OrderedFloat(1234.5_f64));
        let result = apply_filter("currency", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("1,234.50")));
    }

    // ── filter_round ──────────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_round")]
    fn test_filter_round_2() {
        // Use a value that is not an approximation of any well-known constant.
        let val = Value::Float(OrderedFloat(12.345_67_f64));
        let args = vec![Value::Int(2)];
        let result = apply_filter("round", &val, &args, span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("12.35")));
    }

    // ── filter_int ───────────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_int")]
    fn test_filter_int() {
        let val = Value::Str(Arc::from("42"));
        let result = apply_filter("int", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    // ── filter_float ─────────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_float")]
    fn test_filter_float() {
        // Use a value that is not an approximation of a well-known constant.
        let val = Value::Str(Arc::from("1.234"));
        let result = apply_filter("float", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Float(OrderedFloat(1.234_f64)));
    }

    // ── filter_string ────────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_string")]
    fn test_filter_string() {
        let val = Value::Int(99);
        let result = apply_filter("string", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("99")));
    }

    // ── filter_join ──────────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_join")]
    fn test_filter_join() {
        let val = Value::List(vec![
            Value::Str(Arc::from("a")),
            Value::Str(Arc::from("b")),
        ]);
        let args = vec![Value::Str(Arc::from(", "))];
        let result = apply_filter("join", &val, &args, span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("a, b")));
    }

    // ── filter_length ─────────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_length")]
    fn test_filter_length_list() {
        let val = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = apply_filter("length", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_length")]
    fn test_filter_length_str() {
        let val = Value::Str(Arc::from("hello"));
        let result = apply_filter("length", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    // ── filter_trim ──────────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_trim")]
    fn test_filter_trim() {
        let val = Value::Str(Arc::from("  hi  "));
        let result = apply_filter("trim", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("hi")));
    }

    // ── filter_default ───────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_default")]
    fn test_filter_default_used() {
        // Null → return the fallback argument.
        let val = Value::Null;
        let args = vec![Value::Str(Arc::from("fallback"))];
        let result = apply_filter("default", &val, &args, span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("fallback")));
    }

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_default")]
    fn test_filter_default_not_used() {
        // Non-null → return the value unchanged.
        let val = Value::Str(Arc::from("val"));
        let args = vec![Value::Str(Arc::from("fallback"))];
        let result = apply_filter("default", &val, &args, span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("val")));
    }

    // ── filter_contains ──────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_contains")]
    fn test_filter_contains() {
        let val = Value::Str(Arc::from("hello world"));
        let args = vec![Value::Str(Arc::from("world"))];
        let result = apply_filter("contains", &val, &args, span()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // ── filter_starts_with ───────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_starts_with")]
    fn test_filter_starts_with() {
        let val = Value::Str(Arc::from("hello"));
        let args = vec![Value::Str(Arc::from("he"))];
        let result = apply_filter("starts_with", &val, &args, span()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // ── filter_ends_with ─────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_ends_with")]
    fn test_filter_ends_with() {
        let val = Value::Str(Arc::from("hello"));
        let args = vec![Value::Str(Arc::from("lo"))];
        let result = apply_filter("ends_with", &val, &args, span()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // ── filter_replace ───────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "STORY-011: implement filter_replace")]
    fn test_filter_replace() {
        let val = Value::Str(Arc::from("foo bar"));
        let args = vec![
            Value::Str(Arc::from("bar")),
            Value::Str(Arc::from("baz")),
        ];
        let result = apply_filter("replace", &val, &args, span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("foo baz")));
    }

    // ── filter_not_found ─────────────────────────────────────────────────────

    #[test]
    fn test_filter_unknown_returns_filter_not_found_error() {
        // This must succeed immediately (no todo!), because the dispatch
        // returns Err before calling any individual filter.
        let val = Value::Str(Arc::from("anything"));
        let result = apply_filter("bogus_filter_xyz", &val, &[], span());
        assert!(
            matches!(result, Err(EvalError::FilterNotFound { .. })),
            "unknown filter must return FilterNotFound error, got: {result:?}"
        );
    }
}
