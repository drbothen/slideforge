//! High-level evaluation helpers for the slideforge expression evaluator.
//!
//! This module provides convenience wrappers over [`crate::expr::eval_expr`]
//! for use by the slide builder and template renderer.

use std::sync::Arc;

use slideforge_syntax::{DiagnosticSink, Expr};
use slideforge_types::Value;

use crate::env::Env;
use crate::expr::eval_expr;
use crate::filters::format_float_display;

// ─── eval_expr_to_string ────────────────────────────────────────────────────

/// Evaluate `expr` in `env` and coerce the result to an `Arc<str>`.
///
/// This is the function used by the template renderer for `{{ expr }}`
/// interpolation sites. The coercion rules are:
///
/// | Value variant | String representation    |
/// |--------------|--------------------------|
/// | `Str(s)`     | `s` (no quotes)          |
/// | `Int(n)`     | decimal representation    |
/// | `Float(f)`   | decimal representation    |
/// | `Bool(b)`    | `"true"` or `"false"`    |
/// | `Null`       | `""` (empty string)      |
/// | `List(_)`    | Error: E-EVL-003 (use `\| join` to convert)  |
/// | `Map(_)`     | Error: E-EVL-003 (use dot access for fields)  |
///
/// Returns `None` if expression evaluation fails; the error is pushed to
/// `sink` by the underlying [`crate::expr::eval_expr`] call.
///
/// # Errors pushed to `sink`
///
/// Delegates entirely to [`crate::expr::eval_expr`].
pub fn eval_expr_to_string(env: &Env, expr: &Expr, sink: &mut DiagnosticSink) -> Option<Arc<str>> {
    use crate::error::EvalError;
    use slideforge_syntax::error::ParseSeverity;
    use slideforge_types::SourceSpan;

    let val = eval_expr(env, expr, sink)?;
    match val {
        Value::Str(s) => Some(s),
        Value::Int(n) => Some(Arc::from(n.to_string().as_str())),
        Value::Float(f) => Some(Arc::from(format_float_display(f.0).as_str())),
        Value::Bool(b) => Some(Arc::from(if b { "true" } else { "false" })),
        Value::Null => Some(Arc::from("")),
        Value::List(_) | Value::Map(_) => {
            sink.push_with_severity(
                EvalError::TypeMismatch {
                    message: format!(
                        "cannot coerce {} to string for interpolation",
                        val.type_name()
                    ),
                    span: SourceSpan::default(),
                },
                ParseSeverity::Error,
            );
            None
        },
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use indexmap::IndexMap;
    use ordered_float::OrderedFloat;
    use slideforge_syntax::{DiagnosticSink, Expr};
    use slideforge_types::Value;

    use super::*;
    use crate::env::Env;

    fn empty_env() -> Env {
        Env::new(IndexMap::new())
    }

    fn env_with(pairs: &[(&str, Value)]) -> Env {
        let mut vars = IndexMap::new();
        for (k, v) in pairs {
            vars.insert(Arc::from(*k), v.clone());
        }
        Env::new(vars)
    }

    // ── Int → string ──────────────────────────────────────────────────────────

    /// BC-2.02.008: `Int(42)` coerces to `"42"`
    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_int() {
        let env = env_with(&[("n", Value::Int(42))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("n".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(sink.is_empty());
        assert_eq!(result, Some(Arc::from("42")));
    }

    // ── Str → string ──────────────────────────────────────────────────────────

    /// BC-2.02.008: `Str("hello")` coerces to `"hello"` (no quotes added)
    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_str() {
        let env = env_with(&[("msg", Value::Str(Arc::from("hello")))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("msg".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(sink.is_empty());
        assert_eq!(result, Some(Arc::from("hello")));
    }

    // ── Null → empty string ───────────────────────────────────────────────────

    /// BC-2.02.008: `Null` coerces to `""` (empty string, not "null")
    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_null() {
        let env = env_with(&[("nothing", Value::Null)]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("nothing".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(sink.is_empty());
        assert_eq!(result, Some(Arc::from("")));
    }

    // ── Bool → "true"/"false" ────────────────────────────────────────────────

    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_bool_true() {
        let env = env_with(&[("flag", Value::Bool(true))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("flag".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(sink.is_empty());
        assert_eq!(result, Some(Arc::from("true")));
    }

    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_bool_false() {
        let env = env_with(&[("flag", Value::Bool(false))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("flag".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(sink.is_empty());
        assert_eq!(result, Some(Arc::from("false")));
    }

    // ── Float → decimal string ────────────────────────────────────────────────

    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_float() {
        // Use a value that is not an approximation of a well-known constant
        // (avoids clippy::approx_constant lint).
        let test_val = 1.234_567_89_f64;
        let env = env_with(&[("x", Value::Float(OrderedFloat(test_val)))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("x".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(sink.is_empty());
        let s = result.expect("float must produce Some(string)");
        // Parse it back to verify it's a valid float representation.
        let reparsed: f64 = s.parse().expect("float string must be parseable");
        assert!((reparsed - test_val).abs() < 1e-10);
    }

    // ── Undefined var → None + error ─────────────────────────────────────────

    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_undefined_var_returns_none() {
        let env = empty_env();
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("nonexistent".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert_eq!(result, None, "undefined var must return None");
        assert!(!sink.is_empty(), "undefined var must push a diagnostic");
    }

    // ── Literal Null expr → empty string ──────────────────────────────────────

    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_literal_null() {
        let env = empty_env();
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Null;
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(sink.is_empty());
        assert_eq!(result, Some(Arc::from("")));
    }

    // ── List/Map → error (FINDING-005) ───────────────────────────────────────

    /// FINDING-005: `eval_expr_to_string` must return None + push error for List values.
    #[test]
    fn test_eval_expr_to_string_list_returns_error() {
        let env = env_with(&[("items", Value::List(vec![Value::Int(1)]))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("items".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(
            result.is_none(),
            "List value must return None from eval_expr_to_string"
        );
        assert!(!sink.is_empty(), "List value must push a diagnostic");
    }

    /// FINDING-005: `eval_expr_to_string` must return None + push error for Map values.
    #[test]
    fn test_eval_expr_to_string_map_returns_error() {
        use slideforge_types::OrderedMap;
        let mut map = OrderedMap::new();
        map.insert(Arc::from("k"), Value::Int(1));
        let env = env_with(&[("obj", Value::Map(map))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("obj".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(
            result.is_none(),
            "Map value must return None from eval_expr_to_string"
        );
        assert!(!sink.is_empty(), "Map value must push a diagnostic");
    }
}
