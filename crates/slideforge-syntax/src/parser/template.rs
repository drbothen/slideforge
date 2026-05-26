//! Template value parser combinator.
//!
//! Implements `template_value()` which splits a quoted string on `{{ ... }}`
//! boundaries and parses the inner content with `expr()`.
//!
//! # Design
//!
//! The lexer emits the entire content of a string literal as a single
//! `Token::StringLit(content)` token (without `{{ }}`-splitting). Template
//! parsing is therefore done at the parser level by scanning the raw string
//! content for `{{ ... }}` substrings.
//!
//! For each `{{ inner }}` found, the inner text is re-lexed with
//! [`crate::lexer::lex`] and parsed with `expr()`. This keeps the grammar
//! clean and the lexer simple.
//!
//! # Error Recovery
//!
//! A malformed `{{` without matching `}}` emits E-PAR-004 and treats the
//! chunk as literal text, allowing parsing to continue (error accumulation).

use std::sync::Arc;

use chumsky::{input::ValueInput, prelude::*};

use crate::{expr::Expr, template::TemplateChunk, token::Token};

use super::expr::expr;

// ─── Type alias ───────────────────────────────────────────────────────────────

/// The span type used by chumsky in the token-stream parser.
type TSpan = SimpleSpan;

// ─── Error message helpers ────────────────────────────────────────────────────

/// Produce an E-PAR-004 error message for an unterminated `{{ ... }}`.
fn unterminated_interpolation_msg() -> String {
    "E-PAR-004: unterminated `{{` interpolation — missing `}}` to close the expression".to_string()
}

/// Produce an E-PAR-004 error message for an empty `{{ }}`.
fn empty_interpolation_msg() -> String {
    "E-PAR-004: empty expression in `{{ }}` — an expression is required between `{{` and `}}`"
        .to_string()
}

// ─── Template string splitter ────────────────────────────────────────────────

/// Represents one segment of a raw template string.
#[derive(Debug)]
enum RawChunk<'s> {
    /// Plain literal text.
    Literal(&'s str),
    /// Inner content of a `{{ inner }}` interpolation.
    Interpolation(&'s str),
    /// A `{{` with no matching `}}` — error case.
    UnterminatedInterp,
    /// A `{{ }}` with empty / whitespace-only inner content — error case.
    EmptyInterp,
}

/// Split `s` into a sequence of [`RawChunk`]s.
///
/// The function scans `s` for `{{` and `}}` boundaries. It handles:
///
/// - `{{ inner }}` → `RawChunk::Interpolation(inner.trim())`
/// - `{{ }}`        → `RawChunk::EmptyInterp`
/// - `{{ <EOF>`    → `RawChunk::UnterminatedInterp`
/// - Plain text    → `RawChunk::Literal(...)`
fn split_template(s: &str) -> Vec<RawChunk<'_>> {
    let mut chunks = Vec::new();
    let mut remaining = s;

    while !remaining.is_empty() {
        match remaining.find("{{") {
            None => {
                // No more `{{` — the rest is a literal.
                chunks.push(RawChunk::Literal(remaining));
                break;
            },
            Some(open_pos) => {
                // Emit the literal prefix (may be empty if `{{` is at start).
                if open_pos > 0 {
                    chunks.push(RawChunk::Literal(&remaining[..open_pos]));
                }
                // Slice past `{{`.
                let after_open = &remaining[open_pos + 2..];
                match after_open.find("}}") {
                    None => {
                        // `{{` with no `}}` — unterminated interpolation.
                        chunks.push(RawChunk::UnterminatedInterp);
                        // Consume the rest of the string.
                        remaining = "";
                    },
                    Some(close_pos) => {
                        let inner = &after_open[..close_pos];
                        if inner.trim().is_empty() {
                            chunks.push(RawChunk::EmptyInterp);
                        } else {
                            chunks.push(RawChunk::Interpolation(inner));
                        }
                        // Advance past `}}`.
                        remaining = &after_open[close_pos + 2..];
                    },
                }
            },
        }
    }

    chunks
}

// ─── Inner expression parser ─────────────────────────────────────────────────

/// Lex and parse `inner_src` as an expression.
///
/// Returns `Ok(Expr)` on success, or `Err(())` on lex/parse failure.
/// The caller is responsible for converting errors to the appropriate
/// E-PAR-004 `SyntaxError` entries.
fn parse_inner_expr(inner_src: &str) -> Result<Expr, ()> {
    use crate::lexer::lex;
    use chumsky::input::Input as _;

    // Trim whitespace before lexing: the full DSL lexer is indentation-sensitive,
    // so leading/trailing spaces inside `{{ ... }}` must be stripped to prevent
    // the lexer from emitting spurious `Indent`/`Dedent` tokens.
    let trimmed = inner_src.trim();
    if trimmed.is_empty() {
        return Err(());
    }

    let (tokens, lex_errors) = lex(trimmed, Arc::from("<template-expr>"));
    if !lex_errors.is_empty() {
        return Err(());
    }

    // Filter out structural tokens that are not valid in expression context.
    // The DSL lexer emits Newline/Eof at line boundaries and Indent/Dedent for
    // indentation levels. These structural sentinels must be stripped before the
    // expression parser runs so that `name`, `item.name`, and `x + 1` parse
    // cleanly without consuming any structural tokens.
    let spanned: Vec<(Token, SimpleSpan)> = tokens
        .into_iter()
        .filter(|(t, _)| {
            !matches!(
                t,
                Token::Newline | Token::Eof | Token::Indent(_) | Token::Dedent
            )
        })
        .map(|(t, s)| (t, SimpleSpan::from(s)))
        .collect();

    let eoi = SimpleSpan::from(trimmed.len()..trimmed.len());
    let input = spanned
        .as_slice()
        .map(eoi, |(t, s): &(Token, SimpleSpan)| (t, s));

    let (result, parse_errors) = expr().parse(input).into_output_errors();
    if !parse_errors.is_empty() || result.is_none() {
        return Err(());
    }
    Ok(result.unwrap_or(Expr::Error))
}

// ─── Public combinator ───────────────────────────────────────────────────────

/// Parse a field value that may contain `{{ expr }}` template interpolation.
///
/// Matches a `Token::StringLit` and splits its content on `{{ ... }}`
/// boundaries. Each segment becomes either a [`TemplateChunk::Literal`] or a
/// [`TemplateChunk::Expr`]. Errors (unterminated `{{`, empty `{{ }}`) are
/// accumulated via the chumsky emission mechanism.
///
/// On success, returns a `Vec<TemplateChunk>`. A plain string with no
/// interpolation produces a single `[TemplateChunk::Literal(content)]`.
#[must_use]
pub fn template_value<'src, I>()
-> impl Parser<'src, I, (Vec<TemplateChunk>, Vec<String>), extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    select! {
        Token::StringLit(s) => s.to_string()
    }
    .map(|content| {
        let raw_chunks = split_template(&content);
        let mut chunks: Vec<TemplateChunk> = Vec::new();
        let mut errors: Vec<String> = Vec::new();

        for raw in raw_chunks {
            match raw {
                RawChunk::Literal(text) => {
                    if !text.is_empty() {
                        chunks.push(TemplateChunk::Literal(text.to_string()));
                    }
                },
                RawChunk::Interpolation(inner) => {
                    if let Ok(expr_val) = parse_inner_expr(inner) {
                        chunks.push(TemplateChunk::Expr(expr_val));
                    } else {
                        chunks.push(TemplateChunk::Expr(Expr::Error));
                        errors.push(unterminated_interpolation_msg());
                    }
                },
                RawChunk::UnterminatedInterp => {
                    chunks.push(TemplateChunk::Expr(Expr::Error));
                    errors.push(unterminated_interpolation_msg());
                },
                RawChunk::EmptyInterp => {
                    chunks.push(TemplateChunk::Expr(Expr::Error));
                    errors.push(empty_interpolation_msg());
                },
            }
        }

        (chunks, errors)
    })
}
