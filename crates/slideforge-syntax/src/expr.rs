//! Expression AST types for the slideforge DSL.
//!
//! The [`Expr`] enum represents all expression forms in the DSL's computation
//! layer: identifiers, literals, binary operations, unary operations, list
//! literals, field access, and pipe expressions.
//!
//! # Architecture Constraints (SS-01)
//!
//! `slideforge-syntax` is **Pure Core** — all types here are data-only with no
//! I/O, side effects, or runtime state. Evaluation semantics live in
//! `slideforge-eval` (STORY-011/012).
//!
//! # Design
//!
//! - [`Expr::Error`] is the error-recovery sentinel. It must be used instead of
//!   `Option<Expr>` because `None` conflates "absent" with "parse failed".
//! - All types implement `Hash + Eq + Clone + Debug` for comemo cache
//!   compatibility (ADR-013) and Kani proof harness requirements.
//! - [`Expr::Float`] uses [`ordered_float::OrderedFloat`] so the whole enum is
//!   `Hash + Eq`.

use ordered_float::OrderedFloat;

// ─── BinOpKind ───────────────────────────────────────────────────────────────

/// The operator in a binary expression.
///
/// Precedence (highest to lowest when parsing):
/// 1. Multiplicative: `Mul`, `Div`, `Rem`
/// 2. Additive: `Add`, `Sub`
/// 3. Comparison: `Eq`, `Ne`, `Lt`, `Le`, `Gt`, `Ge`
/// 4. Logical: `And`, `Or`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinOpKind {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Rem,
    /// `==`
    Eq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `&&` or `and`
    And,
    /// `||` or `or`
    Or,
}

// ─── UnaryOpKind ─────────────────────────────────────────────────────────────

/// The operator in a unary expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnaryOpKind {
    /// `!` — logical NOT
    Not,
    /// `-` — arithmetic negation
    Neg,
}

// ─── Expr ────────────────────────────────────────────────────────────────────

/// An expression in the slideforge DSL computation layer.
///
/// Used in `@for`, `@if`, `{{ expr }}` interpolations, and `vars:` values.
///
/// # Parsing
///
/// The expression parser is implemented in `parser/expr.rs` using manual
/// precedence climbing via `foldl`/`foldr` (chumsky 0.10.1 pattern).
/// `chumsky::pratt` is NOT available in chumsky 0.10.1.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    // ── Terminals ─────────────────────────────────────────────────────────
    /// An unquoted identifier: `items`, `name`, `row`.
    Ident(String),

    /// An integer literal: `42`, `0`, `-1`.
    Num(i64),

    /// A floating-point literal: `3.14`, `0.5`.
    Float(OrderedFloat<f64>),

    /// A string literal: `"hello"`.
    Str(String),

    /// A boolean literal: `true` or `false`.
    Bool(bool),

    /// The null literal: `null`.
    Null,

    // ── Composites ────────────────────────────────────────────────────────
    /// A list literal: `[1, 2, 3]`.
    List(Vec<Expr>),

    /// A map literal: `{key: val, ...}`.
    ///
    /// Keys are always string identifiers. Values are arbitrary expressions.
    Map(Vec<(String, Expr)>),

    /// Field access: `row.name`, `item.price`.
    FieldAccess {
        /// The base expression (usually an `Ident`).
        base: Box<Expr>,
        /// The field being accessed.
        field: String,
    },

    /// A binary operation: `x + 1`, `env == "prod"`.
    BinOp {
        /// The operator.
        op: BinOpKind,
        /// The left-hand side.
        lhs: Box<Expr>,
        /// The right-hand side.
        rhs: Box<Expr>,
    },

    /// A unary operation: `!flag`, `-count`.
    UnaryOp {
        /// The operator.
        op: UnaryOpKind,
        /// The operand.
        operand: Box<Expr>,
    },

    /// A pipe expression: `items | upper`, `amount | pad(10)`.
    ///
    /// The `filter` is the name of the built-in filter function.
    /// `args` are optional arguments passed to the filter.
    Pipe {
        /// The expression being piped.
        lhs: Box<Expr>,
        /// The filter name.
        filter: String,
        /// Optional arguments to the filter.
        args: Vec<Expr>,
    },

    /// A function call: `ref("slide-1")`, `footnote("see appendix")`, `figref(3)`.
    ///
    /// Used for the built-in cross-reference and footnote pseudo-functions that
    /// appear inside `{{ ... }}` interpolations:
    ///   - `{{ ref("id") }}` → `InlineNode::Xref(Arc::from("id"))`
    ///   - `{{ footnote("text") }}` → `InlineNode::Footnote([Plain("text")])`
    ///   - `{{ figref(n) }}` → `InlineNode::Xref(Arc::from("fig-N"))`
    ///
    /// Unknown calls are evaluated at eval time: `eval_expr` maps them to
    /// `EvalError::UnsupportedBuiltinCall` (E-EVL-007) and returns `None`
    /// rather than panicking. This is NOT a user-callable function mechanism —
    /// no user-defined functions exist in v1 (Q1 decision: deferred to v2).
    ///
    /// # Grammar position
    ///
    /// Call is a postfix/primary form: an identifier immediately followed by
    /// a parenthesised comma-separated argument list. It sits at the same level
    /// as `FieldAccess` (postfix after atom) and is parsed before unary / binary
    /// operators. Call composes with pipe: `ref("id") | upper` is a valid but
    /// semantically unusual expression.
    ///
    /// # Comemo / Kani compatibility
    ///
    /// Derives `Hash + Eq + Clone + Debug` (same as all other `Expr` variants),
    /// required by ADR-013 and the Kani proof harness.
    Call {
        /// The function name (e.g., `"ref"`, `"footnote"`, `"figref"`).
        func: String,
        /// The positional arguments.
        args: Vec<Expr>,
    },

    // ── Error recovery ────────────────────────────────────────────────────
    /// Error sentinel produced by the error-recovery path.
    ///
    /// This must be used (not `Option<Expr>`) to clearly distinguish
    /// "parse failed" from "expression is absent" — two very different
    /// situations at the AST level.
    Error,
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_bc_1_04_001_expr_derives_hash_eq_clone_debug() {
        let e = Expr::Ident("x".to_string());
        let e2 = e.clone();
        assert_eq!(e, e2);
        let _ = format!("{e:?}");
        let mut set = HashSet::new();
        set.insert(e);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_04_001_all_expr_variants_constructible() {
        let _ = Expr::Ident("items".to_string());
        let _ = Expr::Num(42);
        let _ = Expr::Float(OrderedFloat(std::f64::consts::PI));
        let _ = Expr::Str("hello".to_string());
        let _ = Expr::Bool(true);
        let _ = Expr::Bool(false);
        let _ = Expr::Null;
        let _ = Expr::List(vec![Expr::Num(1), Expr::Num(2)]);
        let _ = Expr::Map(vec![("key".to_string(), Expr::Num(1))]);
        let _ = Expr::FieldAccess {
            base: Box::new(Expr::Ident("row".to_string())),
            field: "name".to_string(),
        };
        let _ = Expr::BinOp {
            op: BinOpKind::Add,
            lhs: Box::new(Expr::Ident("x".to_string())),
            rhs: Box::new(Expr::Num(1)),
        };
        let _ = Expr::UnaryOp {
            op: UnaryOpKind::Not,
            operand: Box::new(Expr::Bool(true)),
        };
        let _ = Expr::Pipe {
            lhs: Box::new(Expr::Ident("name".to_string())),
            filter: "upper".to_string(),
            args: vec![],
        };
        let _ = Expr::Call {
            func: "ref".to_string(),
            args: vec![Expr::Str("slide-1".to_string())],
        };
        let _ = Expr::Error;
    }

    #[test]
    fn test_bc_1_04_001_bin_op_kind_all_variants() {
        let ops = [
            BinOpKind::Add,
            BinOpKind::Sub,
            BinOpKind::Mul,
            BinOpKind::Div,
            BinOpKind::Rem,
            BinOpKind::Eq,
            BinOpKind::Ne,
            BinOpKind::Lt,
            BinOpKind::Le,
            BinOpKind::Gt,
            BinOpKind::Ge,
            BinOpKind::And,
            BinOpKind::Or,
        ];
        for op in &ops {
            let op2 = op.clone();
            assert_eq!(op, &op2);
        }
    }

    #[test]
    fn test_bc_1_04_001_unary_op_kind_all_variants() {
        let _ = UnaryOpKind::Not;
        let _ = UnaryOpKind::Neg;
    }

    #[test]
    fn test_bc_1_04_001_expr_error_sentinel_distinct_from_bool_false() {
        // Error must be distinguishable from Null and from Bool(false).
        assert_ne!(Expr::Error, Expr::Null);
        assert_ne!(Expr::Error, Expr::Bool(false));
    }

    #[test]
    fn test_bc_1_04_001_expr_list_empty_is_valid() {
        // EC-001: @for with empty literal list must parse to Expr::List([]).
        let e = Expr::List(vec![]);
        let _ = format!("{e:?}");
        assert_eq!(e, Expr::List(vec![]));
    }

    #[test]
    fn test_bc_1_04_001_expr_nested_binop_structure() {
        // (x + 1) == 2 — verifies deep nesting doesn't cause issues.
        let inner = Expr::BinOp {
            op: BinOpKind::Add,
            lhs: Box::new(Expr::Ident("x".to_string())),
            rhs: Box::new(Expr::Num(1)),
        };
        let outer = Expr::BinOp {
            op: BinOpKind::Eq,
            lhs: Box::new(inner),
            rhs: Box::new(Expr::Num(2)),
        };
        let outer2 = outer.clone();
        assert_eq!(outer, outer2);
    }
}
