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

/// Produce an E-PAR-004 error message for an unterminated math block.
fn unterminated_math_msg(is_display: bool) -> String {
    if is_display {
        "E-PAR-004: unterminated `$$` math block — missing closing `$$`".to_string()
    } else {
        "E-PAR-004: unterminated `$` math block — missing closing `$`".to_string()
    }
}

// ─── Template string splitter ────────────────────────────────────────────────

/// Represents one segment of a raw template string.
#[derive(Debug)]
enum RawChunk<'s> {
    /// Plain literal text.
    Literal(&'s str),
    /// Inner content of a `{{ inner }}` interpolation (text mode only).
    Interpolation(&'s str),
    /// A `{{` with no matching `}}` — error case.
    UnterminatedInterp,
    /// A `{{ }}` with empty / whitespace-only inner content — error case.
    EmptyInterp,
    /// Inner content of a `$...$` inline math region (raw LaTeX, may contain
    /// [`MathInterpSegment`] sub-chunks).
    MathInline(Vec<MathSegment<'s>>),
    /// Inner content of a `$$...$$` display math region (raw LaTeX, may
    /// contain [`MathInterpSegment`] sub-chunks).
    MathDisplay(Vec<MathSegment<'s>>),
    /// A `$` or `$$` with no matching closing delimiter — error case (E-PAR-004).
    UnterminatedMath {
        /// Whether this was a display (`$$`) or inline (`$`) delimiter.
        is_display: bool,
        /// The content between the opening delimiter and end-of-string.
        content: Vec<MathSegment<'s>>,
    },
}

/// A segment inside a math region.
#[derive(Debug)]
enum MathSegment<'s> {
    /// Raw LaTeX text.
    Content(&'s str),
    /// `@{inner}` interpolation inside math mode.
    Interp(&'s str),
}

/// Split `s` into a sequence of [`RawChunk`]s.
///
/// Scans `s` for `$$...$$`, `$...$`, `{{ ... }}`, and `@{...}` boundaries.
/// Priority order for delimiter detection:
/// 1. `$$` (display math) — checked before `$` to avoid false positives
/// 2. `$` (inline math)
/// 3. `{{` (text-mode interpolation)
///
/// Inside math regions, `{{ ... }}` is treated as literal LaTeX text (not an
/// interpolation), and `@{...}` is an interpolation.
fn split_template(s: &str) -> Vec<RawChunk<'_>> {
    let mut chunks = Vec::new();
    let mut pos = 0usize;
    let bytes = s.as_bytes();
    let len = s.len();

    // Track the start of the current literal segment.
    let mut lit_start = 0usize;

    while pos < len {
        // ── Check for `$$` (display math) ─────────────────────────────────
        if bytes.get(pos) == Some(&b'$') && bytes.get(pos + 1) == Some(&b'$') {
            // Flush any pending literal.
            if pos > lit_start {
                chunks.push(RawChunk::Literal(&s[lit_start..pos]));
            }
            let content_start = pos + 2;
            // Search for closing `$$`.
            if let Some(rel) = find_str(s, content_start, "$$") {
                let math_content = &s[content_start..rel];
                let segs = parse_math_segments(math_content);
                chunks.push(RawChunk::MathDisplay(segs));
                pos = rel + 2;
            } else {
                // Unterminated `$$` — emit as an error chunk (E-PAR-004).
                let math_content = &s[content_start..];
                let segs = parse_math_segments(math_content);
                chunks.push(RawChunk::UnterminatedMath {
                    is_display: true,
                    content: segs,
                });
                pos = len;
            }
            lit_start = pos;
            continue;
        }

        // ── Check for `$` (inline math) ───────────────────────────────────
        if bytes.get(pos) == Some(&b'$') {
            // Flush any pending literal.
            if pos > lit_start {
                chunks.push(RawChunk::Literal(&s[lit_start..pos]));
            }
            let content_start = pos + 1;
            // Search for closing `$` (not `$$`).
            if let Some(close) = find_single_dollar(s, content_start) {
                let math_content = &s[content_start..close];
                let segs = parse_math_segments(math_content);
                chunks.push(RawChunk::MathInline(segs));
                pos = close + 1;
            } else {
                // Unterminated `$` — emit as an error chunk (E-PAR-004).
                let math_content = &s[content_start..];
                let segs = parse_math_segments(math_content);
                chunks.push(RawChunk::UnterminatedMath {
                    is_display: false,
                    content: segs,
                });
                pos = len;
            }
            lit_start = pos;
            continue;
        }

        // ── Check for `{{` (text-mode interpolation) ──────────────────────
        if bytes.get(pos) == Some(&b'{') && bytes.get(pos + 1) == Some(&b'{') {
            // Flush any pending literal.
            if pos > lit_start {
                chunks.push(RawChunk::Literal(&s[lit_start..pos]));
            }
            let after_open = pos + 2;
            match find_str(s, after_open, "}}") {
                None => {
                    chunks.push(RawChunk::UnterminatedInterp);
                    pos = len;
                },
                Some(close_pos) => {
                    let inner = &s[after_open..close_pos];
                    if inner.trim().is_empty() {
                        chunks.push(RawChunk::EmptyInterp);
                    } else {
                        chunks.push(RawChunk::Interpolation(inner));
                    }
                    pos = close_pos + 2;
                },
            }
            lit_start = pos;
            continue;
        }

        // Advance by one byte (we only check ASCII delimiters above).
        pos += 1;
    }

    // Flush any remaining literal.
    if lit_start < len {
        chunks.push(RawChunk::Literal(&s[lit_start..]));
    }

    chunks
}

/// Find the byte position of the first occurrence of `needle` in `s` at or
/// after `start`.  Returns `None` if not found.
fn find_str(s: &str, start: usize, needle: &str) -> Option<usize> {
    s[start..].find(needle).map(|rel| rel + start)
}

/// Find the closing single `$` at or after `start`.
///
/// A `$$` pair (two consecutive `$`) is NOT treated as a closing delimiter for
/// inline math — it would mean nested display math inside inline math, which
/// is treated as a literal.  We search for a lone `$` that is not immediately
/// followed by another `$`.
fn find_single_dollar(s: &str, start: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut pos = start;
    while pos < s.len() {
        if bytes[pos] == b'$' {
            // A lone `$` closes inline math.  A `$$` is content.
            if bytes.get(pos + 1) != Some(&b'$') {
                return Some(pos);
            }
            // Skip the `$$` pair as content.
            pos += 2;
        } else {
            pos += 1;
        }
    }
    None
}

/// Parse a math content string into [`MathSegment`]s, splitting on `@{...}`.
///
/// `{{ ... }}` inside math content is treated as literal LaTeX text (the `{{`
/// is valid LaTeX `\left\{` style notation) — it is NOT interpreted as a
/// text-mode interpolation.
fn parse_math_segments(content: &str) -> Vec<MathSegment<'_>> {
    let mut segs = Vec::new();
    let bytes = content.as_bytes();
    let len = content.len();
    let mut pos = 0usize;
    let mut lit_start = 0usize;

    while pos < len {
        // Check for `@{` — math-mode interpolation.
        if bytes.get(pos) == Some(&b'@') && bytes.get(pos + 1) == Some(&b'{') {
            // Flush any pending literal content.
            if pos > lit_start {
                segs.push(MathSegment::Content(&content[lit_start..pos]));
            }
            let inner_start = pos + 2;
            // Search for closing `}`.
            match content[inner_start..].find('}') {
                None => {
                    // Unterminated @{ — treat rest as content.
                    segs.push(MathSegment::Content(&content[inner_start..]));
                    pos = len;
                },
                Some(rel) => {
                    let inner = &content[inner_start..inner_start + rel];
                    segs.push(MathSegment::Interp(inner));
                    pos = inner_start + rel + 1;
                },
            }
            lit_start = pos;
        } else {
            pos += 1;
        }
    }

    // Flush remaining literal.
    if lit_start < len {
        segs.push(MathSegment::Content(&content[lit_start..]));
    }

    segs
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

// ─── Math segment processor ─────────────────────────────────────────────────

/// Convert [`MathSegment`]s from a `$...$` or `$$...$$` region into
/// [`TemplateChunk`]s.
///
/// `is_display` is `true` for `$$...$$` and `false` for `$...$`.
///
/// The entire math region is collapsed into one chunk:
/// - If the region has no `@{...}` interpolations, it becomes a single
///   [`TemplateChunk::MathInline`] / [`TemplateChunk::MathDisplay`] containing
///   the concatenated raw content.
/// - If the region has `@{...}` interpolations, each segment is emitted as a
///   separate chunk: `Content` → `MathInline`/`MathDisplay`, `Interp` → `MathInterp`.
fn process_math_segments(
    segs: Vec<MathSegment<'_>>,
    is_display: bool,
    chunks: &mut Vec<TemplateChunk>,
    _errors: &mut Vec<String>,
) {
    // Check whether there are any @{} interpolations.
    let has_interp = segs.iter().any(|s| matches!(s, MathSegment::Interp(_)));

    if has_interp {
        // Has interpolations — emit mixed chunks.
        for seg in segs {
            match seg {
                MathSegment::Content(c) => {
                    if !c.is_empty() {
                        if is_display {
                            chunks.push(TemplateChunk::MathDisplay(c.to_string()));
                        } else {
                            chunks.push(TemplateChunk::MathInline(c.to_string()));
                        }
                    }
                },
                MathSegment::Interp(inner) => {
                    if let Ok(expr_val) = parse_inner_expr(inner) {
                        chunks.push(TemplateChunk::MathInterp(expr_val));
                    } else {
                        chunks.push(TemplateChunk::MathInterp(Expr::Error));
                    }
                },
            }
        }
    } else {
        // No interpolations — concatenate all content and emit one chunk.
        let raw: String = segs
            .iter()
            .filter_map(|s| {
                if let MathSegment::Content(c) = s {
                    Some(*c)
                } else {
                    None
                }
            })
            .collect();
        if is_display {
            chunks.push(TemplateChunk::MathDisplay(raw));
        } else {
            chunks.push(TemplateChunk::MathInline(raw));
        }
    }
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
                RawChunk::MathInline(segs) => {
                    process_math_segments(segs, false, &mut chunks, &mut errors);
                },
                RawChunk::MathDisplay(segs) => {
                    process_math_segments(segs, true, &mut chunks, &mut errors);
                },
                RawChunk::UnterminatedMath {
                    is_display,
                    content,
                } => {
                    // Emit E-PAR-004 for the unterminated delimiter, then
                    // produce a math chunk with the partial content so that
                    // error recovery produces a meaningful AST.
                    errors.push(unterminated_math_msg(is_display));
                    process_math_segments(content, is_display, &mut chunks, &mut errors);
                },
            }
        }

        (chunks, errors)
    })
}
