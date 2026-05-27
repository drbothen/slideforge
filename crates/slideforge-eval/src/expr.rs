//! Expression evaluation for the slideforge DSL.
//!
//! The entry point is [`eval_expr`], which evaluates a [`slideforge_syntax::Expr`]
//! AST node in the context of a [`crate::Env`] and accumulates any errors into
//! a [`slideforge_syntax::DiagnosticSink`].
//!
//! # Error handling
//!
//! `eval_expr` returns `Option<Value>`: `None` signals that evaluation failed
//! and an error has been pushed to `sink`. Callers must propagate `None`
//! upward without short-circuiting — errors are accumulated, not thrown.
//!
//! # Design note: string concatenation
//!
//! The existing [`slideforge_syntax::BinOpKind`] does NOT have a `Tilde` /
//! `Concat` variant. If a string-concatenation operator is added in a future
//! story, a corresponding `BinOpKind::Concat` must be added to
//! `slideforge-syntax` and this module must be updated.
//!
//! TODO(STORY-012): Implement `BinOpKind::Concat` / tilde operator once the
//! parser gains the `~` token and `BinOpKind` gains the variant.

// Imports used by the stub function bodies once implemented (STORY-011).
// The `allow` attributes suppress unused-import warnings during the Red Gate phase.
#[allow(unused_imports)]
use std::sync::Arc;

#[allow(unused_imports)]
use slideforge_syntax::{BinOpKind, DiagnosticSink, Expr, UnaryOpKind};
#[allow(unused_imports)]
use slideforge_types::{SourceSpan, Value};

use crate::env::Env;
use crate::error::EvalError;
#[allow(unused_imports)]
use crate::filters::apply_filter;

// ─── eval_expr ───────────────────────────────────────────────────────────────

/// Evaluate a DSL expression and return the resulting [`Value`].
///
/// Returns `None` if evaluation fails; the error is pushed to `sink` before
/// returning. Multiple independent sub-expressions may fail independently —
/// the caller should continue evaluating siblings and accumulate all errors
/// rather than short-circuiting on the first `None`.
///
/// # Errors pushed to `sink`
///
/// - [`EvalError::UndefinedVariable`] — `Expr::Ident` not found in `env`
/// - [`EvalError::TypeMismatch`] — binary/unary operator applied to wrong types
/// - [`EvalError::DivisionByZero`] — integer or float division by zero
/// - [`EvalError::FilterNotFound`] — pipe references an unknown filter
/// - [`EvalError::FieldAccessFailed`] — dot-access on non-map or missing field
pub fn eval_expr(
    env: &Env,
    expr: &Expr,
    sink: &mut DiagnosticSink,
) -> Option<Value> {
    let _ = (env, expr, sink);
    todo!("STORY-011: implement eval_expr")
}

// ─── push_eval_error ─────────────────────────────────────────────────────────

/// Push an [`EvalError`] into `sink` using a conservative severity and return
/// `None` (to allow callers to `?` or propagate).
///
/// The sink accepts any type implementing `miette::Diagnostic + Send + Sync`.
/// `EvalError` satisfies this bound.
// Used by eval_expr once implemented (STORY-011 Green phase).
#[allow(dead_code)]
fn push_error(sink: &mut DiagnosticSink, err: EvalError) -> Option<Value> {
    use slideforge_syntax::error::ParseSeverity;
    sink.push_with_severity(err, ParseSeverity::Error);
    None
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use indexmap::IndexMap;
    use slideforge_syntax::{BinOpKind, DiagnosticSink, Expr};
    use slideforge_types::{OrderedMap, SourceSpan, Value};

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

    // ── Arithmetic: multiply ─────────────────────────────────────────────────

    /// BC-2.02.001: `n * 2` with env `n=5` → `Value::Int(10)`
    #[test]
    #[should_panic(expected = "STORY-011: implement eval_expr")]
    fn test_bc_2_02_001_arithmetic_mul() {
        let env = env_with(&[("n", Value::Int(5))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::BinOp {
            op: BinOpKind::Mul,
            lhs: Box::new(Expr::Ident("n".to_string())),
            rhs: Box::new(Expr::Num(2)),
        };
        let result = eval_expr(&env, &expr, &mut sink);
        assert!(sink.is_empty(), "no errors expected for valid multiply");
        assert_eq!(result, Some(Value::Int(10)));
    }

    // ── Arithmetic: division by zero ──────────────────────────────────────────

    /// BC-2.02.004: `10 / 0` → `None` + sink has `DivisionByZero` diagnostic
    #[test]
    #[should_panic(expected = "STORY-011: implement eval_expr")]
    fn test_bc_2_02_004_arithmetic_div_zero() {
        let env = empty_env();
        let mut sink = DiagnosticSink::new();
        let expr = Expr::BinOp {
            op: BinOpKind::Div,
            lhs: Box::new(Expr::Num(10)),
            rhs: Box::new(Expr::Num(0)),
        };
        let result = eval_expr(&env, &expr, &mut sink);
        assert_eq!(result, None, "division by zero must return None");
        assert!(!sink.is_empty(), "division by zero must push a diagnostic");
    }

    // ── Field access: success ────────────────────────────────────────────────

    /// BC-2.02.006: `item.price` with `item = Map { "price": Int(42) }` → `Some(Int(42))`
    #[test]
    #[should_panic(expected = "STORY-011: implement eval_expr")]
    fn test_bc_2_02_006_dot_access() {
        let mut map = OrderedMap::new();
        map.insert(Arc::from("price"), Value::Int(42));
        let env = env_with(&[("item", Value::Map(map))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::FieldAccess {
            base: Box::new(Expr::Ident("item".to_string())),
            field: "price".to_string(),
        };
        let result = eval_expr(&env, &expr, &mut sink);
        assert!(sink.is_empty(), "no errors expected for valid field access");
        assert_eq!(result, Some(Value::Int(42)));
    }

    // ── Field access: missing field ───────────────────────────────────────────

    /// BC-2.02.006: `item.missing` → `None` + sink has `FieldAccessFailed`
    #[test]
    #[should_panic(expected = "STORY-011: implement eval_expr")]
    fn test_bc_2_02_006_dot_access_missing() {
        let mut map = OrderedMap::new();
        map.insert(Arc::from("price"), Value::Int(42));
        let env = env_with(&[("item", Value::Map(map))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::FieldAccess {
            base: Box::new(Expr::Ident("item".to_string())),
            field: "missing".to_string(),
        };
        let result = eval_expr(&env, &expr, &mut sink);
        assert_eq!(result, None, "missing field must return None");
        assert!(!sink.is_empty(), "missing field must push a diagnostic");
    }

    // ── Pipe: success ────────────────────────────────────────────────────────

    /// BC-2.02.007: `name | upper` with `name="world"` → `Some(Str("WORLD"))`
    #[test]
    #[should_panic(expected = "STORY-011: implement eval_expr")]
    fn test_bc_2_02_007_pipe_chain() {
        let env = env_with(&[("name", Value::Str(Arc::from("world")))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Pipe {
            lhs: Box::new(Expr::Ident("name".to_string())),
            filter: "upper".to_string(),
            args: vec![],
        };
        let result = eval_expr(&env, &expr, &mut sink);
        assert!(sink.is_empty(), "no errors expected for valid pipe");
        assert_eq!(result, Some(Value::Str(Arc::from("WORLD"))));
    }

    // ── Pipe: unknown filter ──────────────────────────────────────────────────

    /// BC-2.02.007: `name | bogus` → `None` + sink has `FilterNotFound`
    #[test]
    #[should_panic(expected = "STORY-011: implement eval_expr")]
    fn test_bc_2_02_007_pipe_unknown_filter() {
        let env = env_with(&[("name", Value::Str(Arc::from("world")))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Pipe {
            lhs: Box::new(Expr::Ident("name".to_string())),
            filter: "bogus".to_string(),
            args: vec![],
        };
        let result = eval_expr(&env, &expr, &mut sink);
        assert_eq!(result, None, "unknown filter must return None");
        assert!(!sink.is_empty(), "unknown filter must push a diagnostic");
    }

    // ── Undefined variable ───────────────────────────────────────────────────

    /// BC-2.02.002: `greeting` with empty env → `None` + sink has `UndefinedVariable`
    #[test]
    #[should_panic(expected = "STORY-011: implement eval_expr")]
    fn test_bc_2_02_002_undefined_var() {
        let env = empty_env();
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("greeting".to_string());
        let result = eval_expr(&env, &expr, &mut sink);
        assert_eq!(result, None, "undefined variable must return None");
        assert!(!sink.is_empty(), "undefined variable must push a diagnostic");
    }

    // ── Undefined variable: scope list in diagnostic ──────────────────────────

    /// BC-2.02.002: diagnostic for undefined `c` in env {a, b} must mention a, b
    #[test]
    #[should_panic(expected = "STORY-011: implement eval_expr")]
    fn test_bc_2_02_002_undefined_var_scope_listed() {
        let env = env_with(&[("a", Value::Int(1)), ("b", Value::Int(2))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("c".to_string());
        let result = eval_expr(&env, &expr, &mut sink);
        assert_eq!(result, None);
        assert!(!sink.is_empty());
        // The diagnostic message or help text must mention the in-scope variables.
        let diag = &sink.errors()[0];
        let msg = diag.to_string();
        let help = diag.help().map(|h| h.to_string()).unwrap_or_default();
        let combined = format!("{msg} {help}");
        assert!(
            combined.contains('a') || combined.contains('b'),
            "diagnostic must mention in-scope variables 'a' and 'b'; got: {combined}"
        );
    }

    // ── Two errors accumulated ────────────────────────────────────────────────

    /// BC-2.02.002: evaluating two undefined vars in sequence must accumulate both errors
    #[test]
    #[should_panic(expected = "STORY-011: implement eval_expr")]
    fn test_bc_2_02_002_two_errors_accumulated() {
        let env = empty_env();
        let mut sink = DiagnosticSink::new();

        // Evaluate first undefined variable.
        let expr1 = Expr::Ident("x".to_string());
        let r1 = eval_expr(&env, &expr1, &mut sink);
        assert_eq!(r1, None);

        // Evaluate second undefined variable — sink must have 2 diagnostics now.
        let expr2 = Expr::Ident("x".to_string());
        let r2 = eval_expr(&env, &expr2, &mut sink);
        assert_eq!(r2, None);

        assert_eq!(
            sink.len(),
            2,
            "two separate undefined-variable expressions must produce 2 diagnostics"
        );
    }

    // ── Type mismatch: arithmetic on non-numeric ──────────────────────────────

    /// BC-2.02.003: `"NO" + 1` → `None` + sink has `TypeMismatch`
    #[test]
    #[should_panic(expected = "STORY-011: implement eval_expr")]
    fn test_bc_2_02_003_type_mismatch_arith() {
        let env = empty_env();
        let mut sink = DiagnosticSink::new();
        let expr = Expr::BinOp {
            op: BinOpKind::Add,
            lhs: Box::new(Expr::Str("NO".to_string())),
            rhs: Box::new(Expr::Num(1)),
        };
        let result = eval_expr(&env, &expr, &mut sink);
        assert_eq!(result, None, "type mismatch must return None");
        assert!(!sink.is_empty(), "type mismatch must push a diagnostic");
    }

    // ── Push_error helper: verify it returns None ─────────────────────────────

    #[test]
    fn test_push_error_returns_none() {
        use crate::error::EvalError;
        let mut sink = DiagnosticSink::new();
        let result = push_error(
            &mut sink,
            EvalError::DivisionByZero {
                span: SourceSpan::default(),
            },
        );
        assert_eq!(result, None, "push_error must always return None");
        assert_eq!(sink.len(), 1, "push_error must push exactly one diagnostic");
    }
}
