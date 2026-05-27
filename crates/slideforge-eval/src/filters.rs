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
///
/// # DI-004 — `| bool` is deliberately absent
///
/// `| bool` is NOT in this list and MUST NOT be added. Boolean conversion via
/// implicit coercion (`"true" → true`, `"yes" → true`, `"NO" → false`) is
/// explicitly forbidden by BC-1.02.003 invariant 1.
///
/// The only valid boolean-producing path from a string is an explicit comparison:
/// `{{ v == "true" }}` or `{{ v == "yes" }}`.
///
/// Note: the BC-1.02.003 text lists `| bool` in a filter table, but story spec
/// AC-006 and architecture rule 2 override that text — `| bool` is forbidden.
/// This registry is the authoritative source-of-truth for which filters exist.
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
        "contains" => filter_contains(val, args, &span),
        "starts_with" => filter_starts_with(val, args, &span),
        "ends_with" => filter_ends_with(val, args, &span),
        "replace" => filter_replace(val, args, &span),
        _ => Err(EvalError::FilterNotFound {
            name: Arc::from(name),
            available: AVAILABLE_FILTERS.join(", "),
            span,
        }),
    }
}

// ─── Individual filter implementations ────────────────────────────────────────

/// Convert a string value to uppercase.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not a [`Value::Str`].
fn filter_upper(val: &Value, span: SourceSpan) -> Result<Value, EvalError> {
    match val {
        Value::Str(s) => Ok(Value::Str(Arc::from(s.to_uppercase().as_str()))),
        other => Err(EvalError::TypeMismatch {
            message: format!("upper requires a string, got {}", other.type_name()),
            span,
        }),
    }
}

/// Convert a string value to lowercase.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not a [`Value::Str`].
fn filter_lower(val: &Value, span: SourceSpan) -> Result<Value, EvalError> {
    match val {
        Value::Str(s) => Ok(Value::Str(Arc::from(s.to_lowercase().as_str()))),
        other => Err(EvalError::TypeMismatch {
            message: format!("lower requires a string, got {}", other.type_name()),
            span,
        }),
    }
}

/// Format a numeric value as a comma-separated currency string.
///
/// - Integer input (`Value::Int`): no decimal places (e.g. `1000000` → `"1,000,000"`).
/// - Float input (`Value::Float`): two decimal places (e.g. `1234.5` → `"1,234.50"`).
///
/// This follows AC-007 (BC-1.02.004): `arr = 1000000` / `"${{ arr | currency }}"` →
/// `"$1,000,000"` — integer inputs must NOT have spurious `.00` appended.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not [`Value::Int`] or
/// [`Value::Float`].
fn filter_currency(val: &Value, span: SourceSpan) -> Result<Value, EvalError> {
    match val {
        Value::Int(n) => Ok(Value::Str(Arc::from(format_currency_int(*n).as_str()))),
        Value::Float(f) => Ok(Value::Str(Arc::from(format_currency_float(f.0).as_str()))),
        other => Err(EvalError::TypeMismatch {
            message: format!("currency requires int or float, got {}", other.type_name()),
            span,
        }),
    }
}

/// Format an integer as a comma-separated currency string with NO decimal places.
///
/// Example: `1000000` → `"1,000,000"`, `-1234` → `"-1,234"`.
fn format_currency_int(n: i64) -> String {
    let sign = if n < 0 { "-" } else { "" };
    // u64::MAX > i64::MAX so abs() of any negative i64 except MIN fits.
    // i64::MIN is -9_223_372_036_854_775_808 whose abs() overflows i64; handle separately.
    let magnitude: u64 = if n == i64::MIN {
        9_223_372_036_854_775_808_u64
    } else {
        n.unsigned_abs()
    };
    let int_str = magnitude.to_string();
    format!("{sign}{}", insert_thousands_separators(&int_str))
}

/// Format a `f64` as a comma-separated currency string with 2 decimal places.
///
/// Example: `1234.5` → `"1,234.50"`.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
// Truncation and sign-loss are acceptable here: `rounded.abs()` is always
// non-negative and within u64 range for any currency value.
fn format_currency_float(amount: f64) -> String {
    // Round to 2 decimal places first.
    let rounded = (amount * 100.0).round() / 100.0;
    let integer_part = rounded.abs().floor() as u64;
    let frac_part = ((rounded.abs() - rounded.abs().floor()) * 100.0).round() as u64;

    let int_formatted = insert_thousands_separators(&integer_part.to_string());
    let sign = if amount < 0.0 { "-" } else { "" };
    format!("{sign}{int_formatted}.{frac_part:02}")
}

/// Insert thousands separators (commas) into a decimal integer string.
///
/// Example: `"1000000"` → `"1,000,000"`.
fn insert_thousands_separators(digits: &str) -> String {
    let mut with_commas = String::new();
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            with_commas.push(',');
        }
        with_commas.push(ch);
    }
    with_commas.chars().rev().collect()
}

/// Round a float value to `args[0]` (Int) decimal places and render as a
/// string.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not a [`Value::Float`] or
/// if `args[0]` is not a [`Value::Int`].
fn filter_round(val: &Value, args: &[Value], span: SourceSpan) -> Result<Value, EvalError> {
    let f = match val {
        Value::Float(f) => f.0,
        other => {
            return Err(EvalError::TypeMismatch {
                message: format!("round requires a float, got {}", other.type_name()),
                span,
            });
        },
    };
    let raw_decimals = match args.first() {
        Some(Value::Int(n)) => *n,
        Some(other) => {
            return Err(EvalError::TypeMismatch {
                message: format!("round argument must be an int, got {}", other.type_name()),
                span,
            });
        },
        None => {
            return Err(EvalError::TypeMismatch {
                message: "round requires one argument (number of decimal places)".to_string(),
                span,
            });
        },
    };
    // Clamp to [0, 18] — powi takes i32 but 18 decimal places is the practical
    // precision limit for f64. The max(0) makes negative decimals behave as 0.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let decimals = raw_decimals.clamp(0, 18) as u32;
    #[allow(clippy::cast_possible_wrap)]
    let factor = 10_f64.powi(decimals as i32);
    let rounded = (f * factor).round() / factor;
    Ok(Value::Str(Arc::from(
        format!("{:.prec$}", rounded, prec = decimals as usize).as_str(),
    )))
}

/// Parse a string or float value as an integer.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not parseable as `Int`.
#[allow(clippy::cast_possible_truncation)] // f64→i64 truncation is the defined behavior of `int` filter
fn filter_int(val: &Value, span: SourceSpan) -> Result<Value, EvalError> {
    match val {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(f.0 as i64)),
        Value::Str(s) => s
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| EvalError::TypeMismatch {
                message: format!("cannot parse {s:?} as int"),
                span,
            }),
        other => Err(EvalError::TypeMismatch {
            message: format!("int requires a string or float, got {}", other.type_name()),
            span,
        }),
    }
}

/// Parse a string or integer value as a float.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not parseable as `Float`.
#[allow(clippy::cast_precision_loss)] // i64→f64 precision loss is the defined behavior of `float` filter
fn filter_float(val: &Value, span: SourceSpan) -> Result<Value, EvalError> {
    use ordered_float::OrderedFloat;
    match val {
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::Int(n) => Ok(Value::Float(OrderedFloat(*n as f64))),
        Value::Str(s) => s
            .parse::<f64>()
            .map(|f| Value::Float(OrderedFloat(f)))
            .map_err(|_| EvalError::TypeMismatch {
                message: format!("cannot parse {s:?} as float"),
                span,
            }),
        other => Err(EvalError::TypeMismatch {
            message: format!("float requires a string or int, got {}", other.type_name()),
            span,
        }),
    }
}

/// Convert any value to its string representation.
///
/// This filter never errors — every [`Value`] has a string representation.
/// The `Result` return type is kept for signature uniformity with other
/// filter functions; the `Err` branch is unreachable.
#[allow(clippy::unnecessary_wraps)]
fn filter_string(val: &Value, _span: SourceSpan) -> Result<Value, EvalError> {
    let s = value_to_display_string(val);
    Ok(Value::Str(Arc::from(s.as_str())))
}

/// Produce a human-readable string representation of any [`Value`].
///
/// Used by [`filter_string`] and [`filter_join`] for element coercion.
fn value_to_display_string(val: &Value) -> String {
    match val {
        Value::Str(s) => s.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(f) => format_float_display(f.0),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        Value::List(_) => "[list]".to_string(),
        Value::Map(_) => "[map]".to_string(),
    }
}

/// Format a `f64` for display — avoids scientific notation for normal values.
pub(crate) fn format_float_display(f: f64) -> String {
    // Use default Display for f64, which avoids scientific notation for
    // values in the normal range.  For values that would use scientific
    // notation we fall back to a precision-limited format.
    let s = format!("{f}");
    if s.contains('e') || s.contains('E') {
        format!("{f:.10}")
    } else {
        s
    }
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
    let items = match val {
        Value::List(v) => v,
        other => {
            return Err(EvalError::TypeMismatch {
                message: format!("join requires a list, got {}", other.type_name()),
                span,
            });
        },
    };
    let sep = match args.first() {
        Some(Value::Str(s)) => s.as_ref(),
        Some(other) => {
            return Err(EvalError::TypeMismatch {
                message: format!("join separator must be a string, got {}", other.type_name()),
                span,
            });
        },
        None => {
            return Err(EvalError::TypeMismatch {
                message: "join requires one argument (separator string)".to_string(),
                span,
            });
        },
    };
    let parts: Vec<String> = items.iter().map(value_to_display_string).collect();
    Ok(Value::Str(Arc::from(parts.join(sep).as_str())))
}

/// Return the length of a string (character count) or list (element count).
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not a [`Value::Str`] or
/// [`Value::List`].
fn filter_length(val: &Value, span: SourceSpan) -> Result<Value, EvalError> {
    match val {
        Value::Str(s) => {
            let len = s.chars().count();
            // Safety: usize → i64 is lossless for strings that can actually
            // exist in memory (max usize ≤ i64::MAX on all supported platforms).
            #[allow(clippy::cast_possible_wrap)]
            Ok(Value::Int(len as i64))
        },
        Value::List(v) =>
        {
            #[allow(clippy::cast_possible_wrap)]
            Ok(Value::Int(v.len() as i64))
        },
        other => Err(EvalError::TypeMismatch {
            message: format!(
                "length requires a string or list, got {}",
                other.type_name()
            ),
            span,
        }),
    }
}

/// Strip leading and trailing ASCII whitespace from a string.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` is not a [`Value::Str`].
fn filter_trim(val: &Value, span: SourceSpan) -> Result<Value, EvalError> {
    match val {
        Value::Str(s) => Ok(Value::Str(Arc::from(s.trim()))),
        other => Err(EvalError::TypeMismatch {
            message: format!("trim requires a string, got {}", other.type_name()),
            span,
        }),
    }
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
    let Some(fallback) = args.first() else {
        return Err(EvalError::TypeMismatch {
            message: "default requires one argument (fallback value)".to_string(),
            span,
        });
    };
    if val.is_null() {
        Ok(fallback.clone())
    } else {
        Ok(val.clone())
    }
}

/// Return `Bool(true)` if the string `val` contains the substring `args[0]`.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` or `args[0]` is not a
/// [`Value::Str`].
fn filter_contains(val: &Value, args: &[Value], span: &SourceSpan) -> Result<Value, EvalError> {
    let s = require_str(val, "contains", span)?;
    let sub = require_str_arg(args, 0, "contains", span)?;
    Ok(Value::Bool(s.contains(sub.as_ref())))
}

/// Return `Bool(true)` if string `val` starts with the prefix `args[0]`.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` or `args[0]` is not a
/// [`Value::Str`].
fn filter_starts_with(val: &Value, args: &[Value], span: &SourceSpan) -> Result<Value, EvalError> {
    let s = require_str(val, "starts_with", span)?;
    let prefix = require_str_arg(args, 0, "starts_with", span)?;
    Ok(Value::Bool(s.starts_with(prefix.as_ref())))
}

/// Return `Bool(true)` if string `val` ends with the suffix `args[0]`.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val` or `args[0]` is not a
/// [`Value::Str`].
fn filter_ends_with(val: &Value, args: &[Value], span: &SourceSpan) -> Result<Value, EvalError> {
    let s = require_str(val, "ends_with", span)?;
    let suffix = require_str_arg(args, 0, "ends_with", span)?;
    Ok(Value::Bool(s.ends_with(suffix.as_ref())))
}

/// Replace all occurrences of substring `args[0]` with `args[1]` in `val`.
///
/// # Errors
///
/// Returns [`EvalError::TypeMismatch`] if `val`, `args[0]`, or `args[1]` is
/// not a [`Value::Str`].
fn filter_replace(val: &Value, args: &[Value], span: &SourceSpan) -> Result<Value, EvalError> {
    let s = require_str(val, "replace", span)?;
    let from = require_str_arg(args, 0, "replace", span)?;
    let to = require_str_arg(args, 1, "replace", span)?;
    Ok(Value::Str(Arc::from(
        s.replace(from.as_ref(), to.as_ref()).as_str(),
    )))
}

// ─── Helper utilities ────────────────────────────────────────────────────────

/// Require `val` to be a [`Value::Str`], returning a reference to the inner
/// `Arc<str>`. Returns a [`EvalError::TypeMismatch`] if the variant is wrong.
fn require_str<'v>(
    val: &'v Value,
    filter_name: &str,
    span: &SourceSpan,
) -> Result<&'v Arc<str>, EvalError> {
    match val {
        Value::Str(s) => Ok(s),
        other => Err(EvalError::TypeMismatch {
            message: format!(
                "{filter_name} requires a string input, got {}",
                other.type_name()
            ),
            span: span.clone(),
        }),
    }
}

/// Require `args[index]` to be a [`Value::Str`], returning a reference to the
/// inner `Arc<str>`. Returns a [`EvalError::TypeMismatch`] if missing or wrong
/// type.
fn require_str_arg<'a>(
    args: &'a [Value],
    index: usize,
    filter_name: &str,
    span: &SourceSpan,
) -> Result<&'a Arc<str>, EvalError> {
    match args.get(index) {
        Some(Value::Str(s)) => Ok(s),
        Some(other) => Err(EvalError::TypeMismatch {
            message: format!(
                "{filter_name} argument {index} must be a string, got {}",
                other.type_name()
            ),
            span: span.clone(),
        }),
        None => Err(EvalError::TypeMismatch {
            message: format!("{filter_name} requires at least {} argument(s)", index + 1),
            span: span.clone(),
        }),
    }
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
    fn test_filter_upper() {
        let val = Value::Str(Arc::from("hello"));
        let result = apply_filter("upper", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("HELLO")));
    }

    // ── filter_lower ─────────────────────────────────────────────────────────

    #[test]
    fn test_filter_lower() {
        let val = Value::Str(Arc::from("WORLD"));
        let result = apply_filter("lower", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("world")));
    }

    // ── filter_currency ───────────────────────────────────────────────────────

    #[test]
    fn test_filter_currency_int_no_decimals() {
        // AC-007 (BC-1.02.004): integer input → no decimal places.
        let val = Value::Int(1234);
        let result = apply_filter("currency", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("1,234")));
    }

    #[test]
    fn test_filter_currency_large_int() {
        // AC-007 core case: 1,000,000 must format as "1,000,000" (not "1,000,000.00").
        let val = Value::Int(1_000_000);
        let result = apply_filter("currency", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("1,000,000")));
    }

    #[test]
    fn test_filter_currency_float_has_decimals() {
        // Float input → two decimal places, unchanged.
        let val = Value::Float(OrderedFloat(1234.5_f64));
        let result = apply_filter("currency", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("1,234.50")));
    }

    // ── filter_round ──────────────────────────────────────────────────────────

    #[test]
    fn test_filter_round_2() {
        // Use a value that is not an approximation of any well-known constant.
        let val = Value::Float(OrderedFloat(12.345_67_f64));
        let args = vec![Value::Int(2)];
        let result = apply_filter("round", &val, &args, span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("12.35")));
    }

    // ── filter_int ───────────────────────────────────────────────────────────

    #[test]
    fn test_filter_int() {
        let val = Value::Str(Arc::from("42"));
        let result = apply_filter("int", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    // ── filter_float ─────────────────────────────────────────────────────────

    #[test]
    fn test_filter_float() {
        // Use a value that is not an approximation of a well-known constant.
        let val = Value::Str(Arc::from("1.234"));
        let result = apply_filter("float", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Float(OrderedFloat(1.234_f64)));
    }

    // ── filter_string ────────────────────────────────────────────────────────

    #[test]
    fn test_filter_string() {
        let val = Value::Int(99);
        let result = apply_filter("string", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("99")));
    }

    // ── filter_join ──────────────────────────────────────────────────────────

    #[test]
    fn test_filter_join() {
        let val = Value::List(vec![Value::Str(Arc::from("a")), Value::Str(Arc::from("b"))]);
        let args = vec![Value::Str(Arc::from(", "))];
        let result = apply_filter("join", &val, &args, span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("a, b")));
    }

    // ── filter_length ─────────────────────────────────────────────────────────

    #[test]
    fn test_filter_length_list() {
        let val = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = apply_filter("length", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_filter_length_str() {
        let val = Value::Str(Arc::from("hello"));
        let result = apply_filter("length", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    // ── filter_trim ──────────────────────────────────────────────────────────

    #[test]
    fn test_filter_trim() {
        let val = Value::Str(Arc::from("  hi  "));
        let result = apply_filter("trim", &val, &[], span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("hi")));
    }

    // ── filter_default ───────────────────────────────────────────────────────

    #[test]
    fn test_filter_default_used() {
        // Null → return the fallback argument.
        let val = Value::Null;
        let args = vec![Value::Str(Arc::from("fallback"))];
        let result = apply_filter("default", &val, &args, span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("fallback")));
    }

    #[test]
    fn test_filter_default_not_used() {
        // Non-null → return the value unchanged.
        let val = Value::Str(Arc::from("val"));
        let args = vec![Value::Str(Arc::from("fallback"))];
        let result = apply_filter("default", &val, &args, span()).unwrap();
        assert_eq!(result, Value::Str(Arc::from("val")));
    }

    // ── filter_contains ──────────────────────────────────────────────────────

    #[test]
    fn test_filter_contains() {
        let val = Value::Str(Arc::from("hello world"));
        let args = vec![Value::Str(Arc::from("world"))];
        let result = apply_filter("contains", &val, &args, span()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // ── filter_starts_with ───────────────────────────────────────────────────

    #[test]
    fn test_filter_starts_with() {
        let val = Value::Str(Arc::from("hello"));
        let args = vec![Value::Str(Arc::from("he"))];
        let result = apply_filter("starts_with", &val, &args, span()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // ── filter_ends_with ─────────────────────────────────────────────────────

    #[test]
    fn test_filter_ends_with() {
        let val = Value::Str(Arc::from("hello"));
        let args = vec![Value::Str(Arc::from("lo"))];
        let result = apply_filter("ends_with", &val, &args, span()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // ── filter_replace ───────────────────────────────────────────────────────

    #[test]
    fn test_filter_replace() {
        let val = Value::Str(Arc::from("foo bar"));
        let args = vec![Value::Str(Arc::from("bar")), Value::Str(Arc::from("baz"))];
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
