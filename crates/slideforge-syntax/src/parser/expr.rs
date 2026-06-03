//! Expression parser combinator.
//!
//! Implements `expr()` using manual precedence climbing via `foldl`/`foldr`
//! (chumsky 0.10.1 pattern). `chumsky::pratt` is NOT available in 0.10.1.
//!
//! # Precedence Levels (highest binds tightest)
//!
//! 1. Primary: literals, identifiers, `(expr)`, list literals
//! 2. Field access: `ident.field.nested`
//! 3. Unary: `!expr`, `-expr`
//! 4. Multiplicative: `*`, `/`, `%`
//! 5. Additive: `+`, `-`
//! 6. Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
//! 7. Logical AND: `&&` / `and`
//! 8. Logical OR: `||` / `or`
//! 9. Pipe: `| filter` / `| filter(args)`

use chumsky::{input::ValueInput, prelude::*, recursive::Recursive};

use crate::{
    expr::{BinOpKind, Expr, UnaryOpKind},
    token::Token,
};

// ─── Type alias ───────────────────────────────────────────────────────────────

/// The span type used by chumsky in the token-stream parser.
type TSpan = SimpleSpan;

// ─── Token-to-operator mapping ────────────────────────────────────────────────

/// Map a comparison-level operator token to the corresponding [`BinOpKind`].
fn tok_to_cmp_op(tok: &Token) -> BinOpKind {
    match tok {
        Token::EqEq => BinOpKind::Eq,
        Token::BangEq => BinOpKind::Ne,
        Token::Lt => BinOpKind::Lt,
        Token::LtEq => BinOpKind::Le,
        Token::Gt => BinOpKind::Gt,
        Token::GtEq => BinOpKind::Ge,
        _ => unreachable!("tok_to_cmp_op called with non-comparison token"),
    }
}

/// Map an additive-level operator token to the corresponding [`BinOpKind`].
fn tok_to_add_op(tok: &Token) -> BinOpKind {
    match tok {
        Token::Plus => BinOpKind::Add,
        Token::Minus => BinOpKind::Sub,
        _ => unreachable!("tok_to_add_op called with non-additive token"),
    }
}

/// Map a multiplicative-level operator token to the corresponding [`BinOpKind`].
fn tok_to_mul_op(tok: &Token) -> BinOpKind {
    match tok {
        Token::Star => BinOpKind::Mul,
        Token::Slash => BinOpKind::Div,
        Token::Percent => BinOpKind::Rem,
        _ => unreachable!("tok_to_mul_op called with non-multiplicative token"),
    }
}

// ─── Public combinator ───────────────────────────────────────────────────────

/// Build the expression parser.
///
/// Implements manual precedence climbing using `foldl`/`foldr` — the correct
/// pattern for chumsky 0.10.1 (which does NOT have a `pratt` module).
///
/// On parse failure, returns `Expr::Error` via `.or_not().map(Option::unwrap_or(Expr::Error))`.
#[must_use]
pub fn expr<'src, I>() -> impl Parser<'src, I, Expr, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    // recursive() lets the combinator refer to itself for nested expressions.
    recursive(|e| expr_inner(e))
}

/// Inner implementation of the expression parser, parameterised on a recursive
/// self-reference `e` (provided by `recursive()`).
fn expr_inner<'src, I>(
    e: Recursive<dyn Parser<'src, I, Expr, extra::Err<Rich<'src, Token, TSpan>>> + 'src>,
) -> impl Parser<'src, I, Expr, extra::Err<Rich<'src, Token, TSpan>>> + Clone + 'src
where
    I: ValueInput<'src, Token = Token, Span = TSpan> + 'src,
{
    // ── Level 1: primary ──────────────────────────────────────────────────
    // Parenthesised subexpression.
    let paren = e
        .clone()
        .delimited_by(just(Token::LParen), just(Token::RParen));

    // List literal: `[` (expr (`,` expr)*)? `]`
    let list = e
        .clone()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LBracket), just(Token::RBracket))
        .map(Expr::List);

    // Atom: integer, float, bool, null, string, identifier, paren, list.
    let atom = select! {
        Token::IntLit(n) => Expr::Num(n),
        Token::FloatLit(f) => Expr::Float(f),
        Token::BoolLit(b) => Expr::Bool(b),
        Token::StringLit(s) => Expr::Str(s.to_string()),
    }
    .or(select! { Token::Ident(s) if s.as_ref() == "null" => Expr::Null })
    .or(select! { Token::Ident(s) => Expr::Ident(s.to_string()) })
    .or(paren)
    .or(list);

    // ── Level 2a: function call  `ident(arg, ...)` ───────────────────────
    //
    // A postfix call is an identifier immediately followed by a parenthesised
    // comma-separated argument list: `ref("slide-1")`, `figref(3)`,
    // `footnote("see appendix")`. This is the only supported call form in v1
    // (Q1 decision: no user-defined functions; only built-in pseudo-functions
    // ref/footnote/figref for inline-markup cross-references and footnotes).
    //
    // Grammar: Ident `(` (expr (`,` expr)*)? `)` → Expr::Call
    //
    // Placement: postfix/primary level (Level 2a), BEFORE field access (Level 2).
    // A call `ref("id")` is not a valid target for field access, but the grammar
    // allows it compositionally: `ref("id").field` would parse as FieldAccess on
    // a Call, which `eval_expr` would then reject as UnsupportedBuiltinCall.
    //
    // The call parser is deliberately NARROW: it matches ONLY when the atom is
    // an `Expr::Ident` (a named identifier). Arbitrary postfix-call on non-ident
    // expressions (e.g., `(expr)(args)`) is NOT supported in v1 and is not parsed
    // here. This matches the Q1 decision: only built-in pseudo-functions, no
    // first-class functions.
    let call_args = e
        .clone()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LParen), just(Token::RParen));

    let call_or_atom = atom.then(call_args.or_not()).map(|(base, args_opt)| {
        match (base, args_opt) {
            // Ident followed by `(args)` → function call.
            (Expr::Ident(name), Some(args)) => Expr::Call { func: name, args },
            // Any non-Call case: return the base expression unchanged.
            //
            // This covers two sub-cases that share the same result:
            //   (a) Ident NOT followed by `(` → normal variable reference (leave as Ident).
            //   (b) Non-Ident expression followed by `(...)` → unsupported call form;
            //       silently drop the args and return the base. Prevents confusing parse
            //       errors; the eval stage can report the semantic issue if needed.
            //
            // Clippy note: `None` and `Some(_)` branches are intentionally merged here
            // because the result is the same (return `other`). The distinction is documented
            // above for readability, not for behavior — merging avoids the `match_same_arms`
            // lint without losing clarity.
            (other, _) => other,
        }
    });

    // ── Level 2: field access  `ident.field.nested` ───────────────────────
    let field_access = call_or_atom.foldl(
        just(Token::Dot)
            .ignore_then(select! { Token::Ident(s) => s.to_string() })
            .repeated(),
        |base, field| Expr::FieldAccess {
            base: Box::new(base),
            field,
        },
    );

    // ── Level 3: unary `!` ────────────────────────────────────────────────
    let unary = just(Token::Bang)
        .repeated()
        .foldr(field_access, |_bang, operand| Expr::UnaryOp {
            op: UnaryOpKind::Not,
            operand: Box::new(operand),
        });

    // ── Level 4: multiplicative `*`, `/`, `%` ────────────────────────────
    let product = unary.clone().foldl(
        choice((just(Token::Star), just(Token::Slash), just(Token::Percent)))
            .then(unary)
            .repeated(),
        |lhs, (op_tok, rhs)| Expr::BinOp {
            op: tok_to_mul_op(&op_tok),
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
    );

    // ── Level 5: additive `+`, `-` ────────────────────────────────────────
    let sum = product.clone().foldl(
        choice((just(Token::Plus), just(Token::Minus)))
            .then(product)
            .repeated(),
        |lhs, (op_tok, rhs)| Expr::BinOp {
            op: tok_to_add_op(&op_tok),
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
    );

    // ── Level 6: comparison `==`, `!=`, `<`, `>`, `<=`, `>=` ─────────────
    let comparison = sum.clone().foldl(
        choice((
            just(Token::EqEq),
            just(Token::BangEq),
            just(Token::LtEq),
            just(Token::GtEq),
            just(Token::Lt),
            just(Token::Gt),
        ))
        .then(sum)
        .repeated(),
        |lhs, (op_tok, rhs)| Expr::BinOp {
            op: tok_to_cmp_op(&op_tok),
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
    );

    // ── Level 7: logical AND `and` / `&&` ────────────────────────────────
    let logical_and = comparison.clone().foldl(
        just(Token::And).then(comparison).repeated(),
        |lhs, (_and_tok, rhs)| Expr::BinOp {
            op: BinOpKind::And,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
    );

    // ── Level 8: logical OR `or` / `||` ──────────────────────────────────
    let logical_or = logical_and.clone().foldl(
        just(Token::Or).then(logical_and).repeated(),
        |lhs, (_or_tok, rhs)| Expr::BinOp {
            op: BinOpKind::Or,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
    );

    // ── Level 9: pipe `| filter` / `| filter(args)` ──────────────────────
    let filter_args = e
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .or_not()
        .map(Option::unwrap_or_default);

    logical_or.clone().foldl(
        just(Token::Pipe)
            .ignore_then(select! { Token::Ident(s) => s.to_string() })
            .then(filter_args)
            .repeated(),
        |lhs, (filter, args)| Expr::Pipe {
            lhs: Box::new(lhs),
            filter,
            args,
        },
    )
}
