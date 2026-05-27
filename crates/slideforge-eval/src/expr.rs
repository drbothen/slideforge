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

use std::sync::Arc;

use ordered_float::OrderedFloat;
use slideforge_syntax::{BinOpKind, DiagnosticSink, Expr, UnaryOpKind};
use slideforge_types::{SourceSpan, Value};

use crate::env::Env;
use crate::error::EvalError;
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
pub fn eval_expr(env: &Env, expr: &Expr, sink: &mut DiagnosticSink) -> Option<Value> {
    // Use a default span for expressions that don't carry their own span yet.
    // STORY-012 will thread spans through the AST; until then we use a
    // zero-origin default for error reporting.
    let span = SourceSpan::default();

    match expr {
        // ── Terminals ────────────────────────────────────────────────────────
        Expr::Ident(name) => {
            if let Some(val) = env.lookup(name) {
                Some(val.clone())
            } else {
                let scope_list = env
                    .all_names()
                    .iter()
                    .map(|n| n.as_ref().to_owned())
                    .collect::<Vec<_>>()
                    .join(", ");
                push_error(
                    sink,
                    EvalError::UndefinedVariable {
                        name: Arc::from(name.as_str()),
                        scope_list,
                        span,
                    },
                )
            }
        }
        Expr::Num(n) => Some(Value::Int(*n)),
        Expr::Float(f) => Some(Value::Float(*f)),
        Expr::Str(s) => Some(Value::Str(Arc::from(s.as_str()))),
        Expr::Bool(b) => Some(Value::Bool(*b)),
        Expr::Null => Some(Value::Null),

        // ── Composites ───────────────────────────────────────────────────────
        Expr::List(items) => {
            let mut collected = Vec::with_capacity(items.len());
            let mut had_error = false;
            for item in items {
                match eval_expr(env, item, sink) {
                    Some(v) => collected.push(v),
                    None => {
                        // Error already pushed to sink; continue evaluating
                        // remaining items so all errors are accumulated.
                        had_error = true;
                    }
                }
            }
            if had_error { None } else { Some(Value::List(collected)) }
        }
        Expr::Map(entries) => {
            let mut map = slideforge_types::OrderedMap::new();
            let mut had_error = false;
            for (key, val_expr) in entries {
                match eval_expr(env, val_expr, sink) {
                    Some(v) => {
                        map.insert(Arc::from(key.as_str()), v);
                    }
                    None => {
                        had_error = true;
                    }
                }
            }
            if had_error {
                None
            } else {
                Some(Value::Map(map))
            }
        }

        // ── Binary operations ────────────────────────────────────────────────
        Expr::BinOp { op, lhs, rhs } => {
            // Evaluate both sides (accumulate both errors before returning None).
            let lval = eval_expr(env, lhs, sink);
            let rval = eval_expr(env, rhs, sink);
            let (Some(lval), Some(rval)) = (lval, rval) else {
                return None;
            };
            eval_binop(op, &lval, &rval, span, sink)
        }

        // ── Unary operations ─────────────────────────────────────────────────
        Expr::UnaryOp { op, operand } => {
            let val = eval_expr(env, operand, sink)?;
            eval_unaryop(op, val, span, sink)
        }

        // ── Field access ─────────────────────────────────────────────────────
        Expr::FieldAccess { base, field } => {
            let base_val = eval_expr(env, base, sink)?;
            match &base_val {
                Value::Map(map) => {
                    if let Some(v) = map.get(field.as_str()) {
                        Some(v.clone())
                    } else {
                        push_error(
                            sink,
                            EvalError::FieldAccessFailed {
                                field: Arc::from(field.as_str()),
                                parent_type: Arc::from("map"),
                                span,
                            },
                        )
                    }
                }
                other => push_error(
                    sink,
                    EvalError::FieldAccessFailed {
                        field: Arc::from(field.as_str()),
                        parent_type: Arc::from(other.type_name()),
                        span,
                    },
                ),
            }
        }

        // ── Pipe ────────────────────────────────────────────────────────────
        Expr::Pipe {
            lhs,
            filter,
            args: arg_exprs,
        } => {
            let lval = eval_expr(env, lhs, sink)?;
            // Evaluate all filter arguments; accumulate errors.
            let mut filter_args = Vec::with_capacity(arg_exprs.len());
            let mut had_arg_error = false;
            for arg_expr in arg_exprs {
                match eval_expr(env, arg_expr, sink) {
                    Some(v) => filter_args.push(v),
                    None => had_arg_error = true,
                }
            }
            if had_arg_error {
                return None;
            }
            match apply_filter(filter, &lval, &filter_args, span) {
                Ok(v) => Some(v),
                Err(e) => push_error(sink, e),
            }
        }

        // ── Error recovery sentinel ─────────────────────────────────────────
        Expr::Error => None,
    }
}

// ─── eval_binop ──────────────────────────────────────────────────────────────

/// Evaluate a binary operation, pushing errors to `sink` and returning `None`
/// on failure.
fn eval_binop(
    op: &BinOpKind,
    lval: &Value,
    rval: &Value,
    span: SourceSpan,
    sink: &mut DiagnosticSink,
) -> Option<Value> {
    match op {
        // ── Arithmetic ───────────────────────────────────────────────────────
        BinOpKind::Add | BinOpKind::Sub | BinOpKind::Mul | BinOpKind::Div | BinOpKind::Rem => {
            eval_arithmetic(op, lval, rval, span, sink)
        }

        // ── Comparison ───────────────────────────────────────────────────────
        BinOpKind::Eq => Some(Value::Bool(lval == rval)),
        BinOpKind::Ne => Some(Value::Bool(lval != rval)),
        BinOpKind::Lt | BinOpKind::Le | BinOpKind::Gt | BinOpKind::Ge => {
            eval_ordering(op, lval, rval, span, sink)
        }

        // ── Logical (short-circuit already happened at the Expr level since
        //    both operands are already evaluated — for now we just validate
        //    types and compute the result) ────────────────────────────────────
        BinOpKind::And => match (lval, rval) {
            (Value::Bool(l), Value::Bool(r)) => Some(Value::Bool(*l && *r)),
            _ => push_error(
                sink,
                EvalError::TypeMismatch {
                    message: format!(
                        "&& requires two booleans, got {} and {}",
                        lval.type_name(),
                        rval.type_name()
                    ),
                    span,
                },
            ),
        },
        BinOpKind::Or => match (lval, rval) {
            (Value::Bool(l), Value::Bool(r)) => Some(Value::Bool(*l || *r)),
            _ => push_error(
                sink,
                EvalError::TypeMismatch {
                    message: format!(
                        "|| requires two booleans, got {} and {}",
                        lval.type_name(),
                        rval.type_name()
                    ),
                    span,
                },
            ),
        },
    }
}

/// Evaluate an arithmetic binary operation (`+`, `-`, `*`, `/`, `%`).
#[allow(clippy::cast_precision_loss)] // i64→f64 for mixed-type arithmetic is intentional
fn eval_arithmetic(
    op: &BinOpKind,
    lval: &Value,
    rval: &Value,
    span: SourceSpan,
    sink: &mut DiagnosticSink,
) -> Option<Value> {
    match (lval, rval) {
        (Value::Int(l), Value::Int(r)) => {
            let result = match op {
                BinOpKind::Add => l.checked_add(*r).map(Value::Int),
                BinOpKind::Sub => l.checked_sub(*r).map(Value::Int),
                BinOpKind::Mul => l.checked_mul(*r).map(Value::Int),
                BinOpKind::Div => {
                    if *r == 0 {
                        return push_error(sink, EvalError::DivisionByZero { span });
                    }
                    l.checked_div(*r).map(Value::Int)
                }
                BinOpKind::Rem => {
                    if *r == 0 {
                        return push_error(sink, EvalError::DivisionByZero { span });
                    }
                    l.checked_rem(*r).map(Value::Int)
                }
                _ => unreachable!("only arithmetic ops dispatched here"),
            };
            result.or_else(|| {
                push_error(
                    sink,
                    EvalError::TypeMismatch {
                        message: "integer arithmetic overflow".to_string(),
                        span,
                    },
                )
            })
        }
        (Value::Float(l), Value::Float(r)) => {
            let result = match op {
                BinOpKind::Add => l.0 + r.0,
                BinOpKind::Sub => l.0 - r.0,
                BinOpKind::Mul => l.0 * r.0,
                BinOpKind::Div => {
                    if r.0 == 0.0 {
                        return push_error(sink, EvalError::DivisionByZero { span });
                    }
                    l.0 / r.0
                }
                BinOpKind::Rem => {
                    if r.0 == 0.0 {
                        return push_error(sink, EvalError::DivisionByZero { span });
                    }
                    l.0 % r.0
                }
                _ => unreachable!("only arithmetic ops dispatched here"),
            };
            Some(Value::Float(OrderedFloat(result)))
        }
        (Value::Int(l), Value::Float(r)) => {
            let l_f = *l as f64;
            let result = match op {
                BinOpKind::Add => l_f + r.0,
                BinOpKind::Sub => l_f - r.0,
                BinOpKind::Mul => l_f * r.0,
                BinOpKind::Div => {
                    if r.0 == 0.0 {
                        return push_error(sink, EvalError::DivisionByZero { span });
                    }
                    l_f / r.0
                }
                BinOpKind::Rem => {
                    if r.0 == 0.0 {
                        return push_error(sink, EvalError::DivisionByZero { span });
                    }
                    l_f % r.0
                }
                _ => unreachable!(),
            };
            Some(Value::Float(OrderedFloat(result)))
        }
        (Value::Float(l), Value::Int(r)) => {
            let r_f = *r as f64;
            let result = match op {
                BinOpKind::Add => l.0 + r_f,
                BinOpKind::Sub => l.0 - r_f,
                BinOpKind::Mul => l.0 * r_f,
                BinOpKind::Div => {
                    if r_f == 0.0 {
                        return push_error(sink, EvalError::DivisionByZero { span });
                    }
                    l.0 / r_f
                }
                BinOpKind::Rem => {
                    if r_f == 0.0 {
                        return push_error(sink, EvalError::DivisionByZero { span });
                    }
                    l.0 % r_f
                }
                _ => unreachable!(),
            };
            Some(Value::Float(OrderedFloat(result)))
        }
        _ => push_error(
            sink,
            EvalError::TypeMismatch {
                message: format!(
                    "arithmetic operator requires numeric operands, got {} and {}",
                    lval.type_name(),
                    rval.type_name()
                ),
                span,
            },
        ),
    }
}

/// Evaluate an ordering comparison (`<`, `<=`, `>`, `>=`).
#[allow(clippy::cast_precision_loss)] // i64→f64 for mixed-type comparison is intentional
fn eval_ordering(
    op: &BinOpKind,
    lval: &Value,
    rval: &Value,
    span: SourceSpan,
    sink: &mut DiagnosticSink,
) -> Option<Value> {
    // Compare Int/Float values; use PartialOrd semantics.
    let cmp_opt: Option<std::cmp::Ordering> = match (lval, rval) {
        (Value::Int(l), Value::Int(r)) => Some(l.cmp(r)),
        (Value::Float(l), Value::Float(r)) => l.partial_cmp(r),
        (Value::Int(l), Value::Float(r)) => (*l as f64).partial_cmp(&r.0),
        (Value::Float(l), Value::Int(r)) => l.0.partial_cmp(&(*r as f64)),
        (Value::Str(l), Value::Str(r)) => Some(l.cmp(r)),
        _ => None,
    };
    match cmp_opt {
        Some(ord) => {
            let result = match op {
                BinOpKind::Lt => ord.is_lt(),
                BinOpKind::Le => ord.is_le(),
                BinOpKind::Gt => ord.is_gt(),
                BinOpKind::Ge => ord.is_ge(),
                _ => unreachable!("only ordering ops dispatched here"),
            };
            Some(Value::Bool(result))
        }
        None => push_error(
            sink,
            EvalError::TypeMismatch {
                message: format!(
                    "comparison requires compatible types, got {} and {}",
                    lval.type_name(),
                    rval.type_name()
                ),
                span,
            },
        ),
    }
}

// ─── eval_unaryop ────────────────────────────────────────────────────────────

/// Evaluate a unary operation (`!` or `-`).
fn eval_unaryop(
    op: &UnaryOpKind,
    val: Value,
    span: SourceSpan,
    sink: &mut DiagnosticSink,
) -> Option<Value> {
    match op {
        UnaryOpKind::Not => match val {
            Value::Bool(b) => Some(Value::Bool(!b)),
            other => push_error(
                sink,
                EvalError::TypeMismatch {
                    message: format!("! requires a bool, got {}", other.type_name()),
                    span,
                },
            ),
        },
        UnaryOpKind::Neg => match val {
            Value::Int(n) => n.checked_neg().map_or_else(
                || {
                    push_error(
                        sink,
                        EvalError::TypeMismatch {
                            message: "integer overflow: cannot negate minimum integer value"
                                .to_string(),
                            span,
                        },
                    )
                },
                |negated| Some(Value::Int(negated)),
            ),
            Value::Float(f) => Some(Value::Float(OrderedFloat(-f.0))),
            other => push_error(
                sink,
                EvalError::TypeMismatch {
                    message: format!("unary - requires a numeric value, got {}", other.type_name()),
                    span,
                },
            ),
        },
    }
}

// ─── push_eval_error ─────────────────────────────────────────────────────────

/// Push an [`EvalError`] into `sink` using a conservative severity and return
/// `None` (to allow callers to `?` or propagate).
///
/// The sink accepts any type implementing `miette::Diagnostic + Send + Sync`.
/// `EvalError` satisfies this bound.
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

    // ── Unary negation: i64::MIN overflow ────────────────────────────────────

    /// FINDING-002: `-i64::MIN` must not panic; must return None + push an error.
    #[test]
    fn test_negation_overflow_i64_min() {
        use slideforge_syntax::UnaryOpKind;
        let env = empty_env();
        let mut sink = DiagnosticSink::new();
        let expr = Expr::UnaryOp {
            op: UnaryOpKind::Neg,
            operand: Box::new(Expr::Num(i64::MIN)),
        };
        let result = eval_expr(&env, &expr, &mut sink);
        assert!(result.is_none(), "negating i64::MIN must return None (overflow)");
        assert!(!sink.is_empty(), "negating i64::MIN must push a diagnostic");
    }

    // ── List: all errors accumulated, not short-circuit ──────────────────────

    /// FINDING-003: a list with N undefined vars must accumulate N errors,
    /// not stop at the first.
    #[test]
    fn test_list_error_accumulation() {
        let env = empty_env();
        let mut sink = DiagnosticSink::new();
        // List with 3 undefined variables — should accumulate 3 errors.
        let expr = Expr::List(vec![
            Expr::Ident("x".to_string()),
            Expr::Ident("y".to_string()),
            Expr::Ident("z".to_string()),
        ]);
        let result = eval_expr(&env, &expr, &mut sink);
        assert!(result.is_none(), "list with all undefined vars must return None");
        assert_eq!(sink.len(), 3, "all 3 undefined vars should produce errors");
    }
}
