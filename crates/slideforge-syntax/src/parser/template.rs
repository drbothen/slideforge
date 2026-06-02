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
//! A malformed `{{` without matching `}}` emits E-PAR-012 and treats the
//! chunk as an `Expr::Error` sentinel, allowing parsing to continue (error
//! accumulation). An empty `{{ }}` emits E-PAR-013. An unterminated math
//! block (`$` or `$$`) emits E-PAR-014. An unclosed inline markup span
//! emits E-PAR-015. An empty inline markup span emits E-PAR-016.
//!
//! Note: E-PAR-004 is owned by slideforge-eval (`IncludeCycle`). These
//! template-parsing codes (E-PAR-012 to E-PAR-016) are distinct.
//!
//! # STORY-077: Inline markup support (DIR-077-002 §3)
//!
//! `template_value()` now recognises inline markup delimiters (`**`, `_`,
//! `` ` ``, `[text](url)`, `^`, `~~`, `~`, `==`) and produces the
//! corresponding [`TemplateChunk`] structural variants. Inline markup and
//! `{{ }}` interpolation compose: `**{{ expr }}**` produces a
//! `Bold([Expr(expr)])` chunk.
//!
//! Math regions (`$...$`, `$$...$$`) disable ALL text-mode inline markup —
//! the content between math delimiters is verbatim LaTeX.

use std::sync::Arc;

use chumsky::{input::ValueInput, prelude::*};

use crate::{expr::Expr, template::TemplateChunk, token::Token};

use super::expr::expr;

// ─── Type alias ───────────────────────────────────────────────────────────────

/// The span type used by chumsky in the token-stream parser.
type TSpan = SimpleSpan;

// ─── Error message helpers ────────────────────────────────────────────────────

/// Produce an E-PAR-012 error message for an unterminated `{{ ... }}`.
fn unterminated_interpolation_msg() -> String {
    "E-PAR-012: unterminated `{{` interpolation — missing `}}` to close the expression".to_string()
}

/// Produce an E-PAR-013 error message for an empty `{{ }}`.
fn empty_interpolation_msg() -> String {
    "E-PAR-013: empty expression in `{{ }}` — an expression is required between `{{` and `}}`"
        .to_string()
}

/// Produce an E-PAR-014 error message for an unterminated math block.
fn unterminated_math_msg(is_display: bool) -> String {
    if is_display {
        "E-PAR-014: unterminated `$$` math block — missing closing `$$`".to_string()
    } else {
        "E-PAR-014: unterminated `$` math block — missing closing `$`".to_string()
    }
}

/// Produce an E-PAR-015 error message for an unclosed inline markup span.
fn unclosed_inline_msg(delimiter: &str) -> String {
    format!(
        "E-PAR-015: unclosed inline markup delimiter `{delimiter}` — \
         add a closing `{delimiter}` after the markup text, \
         e.g., `{delimiter}text{delimiter}`"
    )
}

/// Produce an E-PAR-016 error message for an empty inline markup span.
fn empty_inline_msg(delimiter: &str) -> String {
    format!(
        "E-PAR-016: empty inline markup span `{delimiter}{delimiter}` — \
         spans must contain at least one character"
    )
}

/// A parse error produced by `scan_template_chunks`, carrying both the byte
/// offset of the opening delimiter (relative to the field-value string content)
/// and the human-readable message.
///
/// The byte offset is used by `section_value_parser` (and other callers that
/// need sub-span precision) to translate the offset into a [`SimpleSpan`] that
/// points at the OPENING delimiter rather than the whole string literal token.
/// Callers that only need the message may use `.into_message()`.
#[derive(Debug, Clone)]
pub struct TemplateError {
    /// Byte offset of the problematic construct within the field-value string.
    pub byte_offset: usize,
    /// Human-readable error message (E-PAR-012 through E-PAR-016).
    pub message: String,
}

impl TemplateError {
    /// Construct a new `TemplateError` with the byte offset of the opening
    /// delimiter and the human-readable error message.
    fn new(byte_offset: usize, message: String) -> Self {
        Self {
            byte_offset,
            message,
        }
    }

    /// Consume self and return the message string.
    #[must_use]
    pub fn into_message(self) -> String {
        self.message
    }
}

// ─── Inner expression parser ─────────────────────────────────────────────────

/// Lex and parse `inner_src` as an expression.
///
/// Returns `Ok(Expr)` on success, or `Err(())` on lex/parse failure.
/// The caller is responsible for converting errors to the appropriate
/// E-PAR-012/E-PAR-013 `SyntaxError` entries.
///
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

// ─── Math segment types ───────────────────────────────────────────────────────

/// A segment inside a math region.
#[derive(Debug)]
enum MathSegment<'s> {
    /// Raw LaTeX text.
    Content(&'s str),
    /// `@{inner}` interpolation inside math mode.
    Interp(&'s str),
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
            if pos > lit_start {
                segs.push(MathSegment::Content(&content[lit_start..pos]));
            }
            let inner_start = pos + 2;
            match content[inner_start..].find('}') {
                None => {
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

    if lit_start < len {
        segs.push(MathSegment::Content(&content[lit_start..]));
    }

    segs
}

/// Convert [`MathSegment`]s from a `$...$` or `$$...$$` region into
/// [`TemplateChunk`]s.
fn process_math_segments(
    segs: Vec<MathSegment<'_>>,
    is_display: bool,
    chunks: &mut Vec<TemplateChunk>,
    _errors: &mut Vec<TemplateError>,
) {
    let has_interp = segs.iter().any(|s| matches!(s, MathSegment::Interp(_)));

    if has_interp {
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

// ─── Unified template scanner (STORY-077 / DIR-077-002 §3) ───────────────────

/// Scan `s` for ALL template constructs and produce a `Vec<TemplateChunk>`.
///
/// This is the unified Phase-1 scanner that handles (in priority order):
///   1. `$$...$$` display math (verbatim content)
///   2. `$...$` inline math (verbatim content)
///   3. `{{ ... }}` text-mode interpolation
///   4. Inline markup delimiters (`~~`, `~`, `**`, `_`, `` ` ``, `[](`, `^`, `==`)
///
/// The `close_on` parameter is the closing delimiter for the enclosing markup
/// span (e.g., `"**"` when scanning the interior of a Bold span). When `None`
/// is passed, scanning continues until the end of `s`.
///
/// # Inline markup composition with `{{ }}`
///
/// `**{{ client }}**` is handled by a single pass: when the scanner is inside
/// a `**` bold span (i.e., `close_on = Some("**")`), it recognises `{{ client }}`
/// as an `Expr` chunk child of the `Bold` variant. This is correct: the recursive
/// call to `scan_template_chunks(inner, None, errors)` where `inner = "{{ client }}"`
/// will find the interpolation and produce `[Expr(Ident("client"))]`.
///
/// # Returns
///
/// A `(Vec<TemplateChunk>, consumed_bytes)` tuple. `consumed_bytes` is the
/// number of bytes consumed from `s` (used by the caller to advance its position
/// when a closing delimiter was found).
// The scanner is inherently long: it handles 11 distinct delimiter patterns in a
// single unified pass (no split possible without losing the composition guarantee).
// Splitting into sub-functions would require threading all mutable state (chunks,
// pos, lit_start, errors) as parameters — no cleaner than the current form.
#[allow(clippy::too_many_lines)]
fn scan_template_chunks(
    s: &str,
    close_on: Option<&str>,
    errors: &mut Vec<TemplateError>,
) -> (Vec<TemplateChunk>, usize) {
    let bytes = s.as_bytes();
    let len = s.len();
    let mut chunks = Vec::new();
    let mut pos = 0usize;
    let mut lit_start = 0usize;

    /// Flush accumulated literal bytes as a `Literal` chunk.
    ///
    /// Does NOT update `lit_start` — the caller updates it after advancing `pos`.
    macro_rules! flush_lit {
        () => {
            if pos > lit_start {
                chunks.push(TemplateChunk::Literal(s[lit_start..pos].to_string()));
            }
        };
    }

    while pos < len {
        // ── Check for closing delimiter (recursive call context) ─────────────
        if let Some(close) = close_on
            && s[pos..].starts_with(close)
        {
            flush_lit!();
            return (chunks, pos + close.len());
        }

        // ── `$$` — display math (check before `$`) ───────────────────────────
        if bytes.get(pos) == Some(&b'$') && bytes.get(pos + 1) == Some(&b'$') {
            flush_lit!();
            let content_start = pos + 2;
            if let Some(rel) = find_str(s, content_start, "$$") {
                let math_content = &s[content_start..rel];
                let segs = parse_math_segments(math_content);
                process_math_segments(segs, true, &mut chunks, errors);
                pos = rel + 2;
            } else {
                errors.push(TemplateError::new(pos, unterminated_math_msg(true)));
                let segs = parse_math_segments(&s[content_start..]);
                process_math_segments(segs, true, &mut chunks, errors);
                pos = len;
            }
            lit_start = pos;
            continue;
        }

        // ── `$` — inline math ─────────────────────────────────────────────────
        if bytes.get(pos) == Some(&b'$') {
            flush_lit!();
            let content_start = pos + 1;
            if let Some(close) = find_single_dollar(s, content_start) {
                let math_content = &s[content_start..close];
                let segs = parse_math_segments(math_content);
                process_math_segments(segs, false, &mut chunks, errors);
                pos = close + 1;
            } else {
                errors.push(TemplateError::new(pos, unterminated_math_msg(false)));
                let segs = parse_math_segments(&s[content_start..]);
                process_math_segments(segs, false, &mut chunks, errors);
                pos = len;
            }
            lit_start = pos;
            continue;
        }

        // ── `{{` — text-mode interpolation ───────────────────────────────────
        if bytes.get(pos) == Some(&b'{') && bytes.get(pos + 1) == Some(&b'{') {
            flush_lit!();
            let after_open = pos + 2;
            match find_str(s, after_open, "}}") {
                None => {
                    chunks.push(TemplateChunk::Expr(Expr::Error));
                    errors.push(TemplateError::new(pos, unterminated_interpolation_msg()));
                    pos = len;
                },
                Some(close_pos) => {
                    let inner = &s[after_open..close_pos];
                    if inner.trim().is_empty() {
                        chunks.push(TemplateChunk::Expr(Expr::Error));
                        errors.push(TemplateError::new(pos, empty_interpolation_msg()));
                    } else if let Ok(expr_val) = parse_inner_expr(inner) {
                        chunks.push(TemplateChunk::Expr(expr_val));
                    } else {
                        chunks.push(TemplateChunk::Expr(Expr::Error));
                        errors.push(TemplateError::new(pos, unterminated_interpolation_msg()));
                    }
                    pos = close_pos + 2;
                },
            }
            lit_start = pos;
            continue;
        }

        // ── `~~` — Strikethrough (MUST check before `~`) ─────────────────────
        if bytes.get(pos) == Some(&b'~') && bytes.get(pos + 1) == Some(&b'~') {
            flush_lit!();
            let open_pos = pos; // byte offset of opening `~~`
            let inner_start = pos + 2;
            let rest = &s[inner_start..];
            if let Some(close_rel) = rest.find("~~") {
                let inner = &rest[..close_rel];
                if inner.is_empty() {
                    errors.push(TemplateError::new(open_pos, empty_inline_msg("~~")));
                } else {
                    let (children, _) = scan_template_chunks(inner, None, errors);
                    chunks.push(TemplateChunk::Strikethrough(children));
                }
                pos = inner_start + close_rel + 2;
            } else {
                // Unclosed `~~` — error recovery.
                errors.push(TemplateError::new(open_pos, unclosed_inline_msg("~~")));
                if !rest.is_empty() {
                    let (children, _) = scan_template_chunks(rest, None, errors);
                    chunks.push(TemplateChunk::Strikethrough(children));
                }
                pos = len;
            }
            lit_start = pos;
            continue;
        }

        // ── `~` — Subscript ──────────────────────────────────────────────────
        if bytes.get(pos) == Some(&b'~') {
            flush_lit!();
            let open_pos = pos; // byte offset of opening `~`
            let inner_start = pos + 1;
            let rest = &s[inner_start..];
            if let Some(close_rel) = rest.find('~') {
                let inner = &rest[..close_rel];
                if inner.is_empty() {
                    errors.push(TemplateError::new(open_pos, empty_inline_msg("~")));
                } else {
                    let (children, _) = scan_template_chunks(inner, None, errors);
                    chunks.push(TemplateChunk::Subscript(children));
                }
                pos = inner_start + close_rel + 1;
            } else {
                errors.push(TemplateError::new(open_pos, unclosed_inline_msg("~")));
                if !rest.is_empty() {
                    let (children, _) = scan_template_chunks(rest, None, errors);
                    chunks.push(TemplateChunk::Subscript(children));
                }
                pos = len;
            }
            lit_start = pos;
            continue;
        }

        // ── `**` — Bold ──────────────────────────────────────────────────────
        if bytes.get(pos) == Some(&b'*') && bytes.get(pos + 1) == Some(&b'*') {
            flush_lit!();
            let open_pos = pos; // byte offset of opening `**`
            let inner_start = pos + 2;
            let rest = &s[inner_start..];
            if rest.contains("**") {
                // Scan the interior recursively, stopping at `**`.
                let (children, consumed) = scan_template_chunks(rest, Some("**"), errors);
                if children.is_empty() {
                    errors.push(TemplateError::new(open_pos, empty_inline_msg("**")));
                } else {
                    chunks.push(TemplateChunk::Bold(children));
                }
                pos = inner_start + consumed;
            } else {
                // Unclosed `**` — error recovery: treat everything as Bold child.
                // `open_pos` points to the opening `**` (DIR-077-002 §5 span requirement).
                errors.push(TemplateError::new(open_pos, unclosed_inline_msg("**")));
                if !rest.is_empty() {
                    let (children, _) = scan_template_chunks(rest, None, errors);
                    chunks.push(TemplateChunk::Bold(children));
                }
                pos = len;
            }
            lit_start = pos;
            continue;
        }

        // ── Single `*` — NOT italic (DIR-077-002 §1 rule 2) — treat as literal ─
        // Falls through to the byte-advance below.

        // ── `_` — Italic ─────────────────────────────────────────────────────
        //
        // A `_` is an italic opener only when it is a left-flanking delimiter:
        // the character BEFORE it must NOT be an ASCII alphanumeric character.
        // This prevents false positives for word-internal underscores such as
        // `SENTINEL_NOTES` or `snake_case` identifiers, which are common in
        // DSL field values (e.g., field names, enum-like sentinel strings).
        //
        // CommonMark §6.1 left-flanking rule (simplified): a `_` starts an
        // emphasis run only if it is NOT preceded directly by a Unicode
        // alphanumeric. Using ASCII alphanumeric as the guard is sufficient
        // for the DSL's identifier and sentinel-string use cases.
        if bytes.get(pos) == Some(&b'_') {
            // Check left-flanking: if the byte immediately before pos is
            // alphanumeric or `_`, treat this `_` as a literal.
            let prev_is_word = pos > 0
                && bytes
                    .get(pos - 1)
                    .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_');
            if prev_is_word {
                // Word-internal `_` — not an italic marker; advance as literal.
                pos += 1;
                continue;
            }
            flush_lit!();
            let open_pos = pos; // byte offset of opening `_`
            let inner_start = pos + 1;
            let rest = &s[inner_start..];
            if rest.contains('_') {
                let (children, consumed) = scan_template_chunks(rest, Some("_"), errors);
                if children.is_empty() {
                    errors.push(TemplateError::new(open_pos, empty_inline_msg("_")));
                } else {
                    chunks.push(TemplateChunk::Italic(children));
                }
                pos = inner_start + consumed;
            } else {
                errors.push(TemplateError::new(open_pos, unclosed_inline_msg("_")));
                if !rest.is_empty() {
                    let (children, _) = scan_template_chunks(rest, None, errors);
                    chunks.push(TemplateChunk::Italic(children));
                }
                pos = len;
            }
            lit_start = pos;
            continue;
        }

        // ── `` ` `` — Code span (verbatim — no inner markup or `{{ }}`) ───────
        if bytes.get(pos) == Some(&b'`') {
            flush_lit!();
            let open_pos = pos; // byte offset of opening backtick
            let inner_start = pos + 1;
            if let Some(close_rel) = s[inner_start..].find('`') {
                let inner = &s[inner_start..inner_start + close_rel];
                if inner.is_empty() {
                    errors.push(TemplateError::new(open_pos, empty_inline_msg("`")));
                } else {
                    // Verbatim: no further processing of the content.
                    chunks.push(TemplateChunk::Code(inner.to_string()));
                }
                pos = inner_start + close_rel + 1;
            } else {
                errors.push(TemplateError::new(open_pos, unclosed_inline_msg("`")));
                let inner = &s[inner_start..];
                if !inner.is_empty() {
                    chunks.push(TemplateChunk::Code(inner.to_string()));
                }
                pos = len;
            }
            lit_start = pos;
            continue;
        }

        // ── `[text](url)` — Link ──────────────────────────────────────────────
        if bytes.get(pos) == Some(&b'[') {
            // Detect `[text](url)` pattern: look for `](`  followed by `)`.
            if let Some(bracket_close_rel) = s[pos + 1..].find("](") {
                let bracket_close = pos + 1 + bracket_close_rel;
                let url_start = bracket_close + 2;
                if let Some(paren_close_rel) = s[url_start..].find(')') {
                    let paren_close = url_start + paren_close_rel;
                    let text_inner = &s[pos + 1..bracket_close];
                    let url = &s[url_start..paren_close];
                    flush_lit!();
                    // The link text IS processed for nested markup + interpolation.
                    let (text_children, _) = scan_template_chunks(text_inner, None, errors);
                    chunks.push(TemplateChunk::Link {
                        text: text_children,
                        url: url.to_string(),
                    });
                    pos = paren_close + 1;
                    lit_start = pos;
                    continue;
                }
            }
            // Not a valid link syntax — fall through as literal.
        }

        // ── `^` — Superscript ────────────────────────────────────────────────
        if bytes.get(pos) == Some(&b'^') {
            flush_lit!();
            let open_pos = pos; // byte offset of opening `^`
            let inner_start = pos + 1;
            let rest = &s[inner_start..];
            if rest.contains('^') {
                let (children, consumed) = scan_template_chunks(rest, Some("^"), errors);
                if children.is_empty() {
                    errors.push(TemplateError::new(open_pos, empty_inline_msg("^")));
                } else {
                    chunks.push(TemplateChunk::Superscript(children));
                }
                pos = inner_start + consumed;
            } else {
                errors.push(TemplateError::new(open_pos, unclosed_inline_msg("^")));
                if !rest.is_empty() {
                    let (children, _) = scan_template_chunks(rest, None, errors);
                    chunks.push(TemplateChunk::Superscript(children));
                }
                pos = len;
            }
            lit_start = pos;
            continue;
        }

        // ── `==` — Highlight ─────────────────────────────────────────────────
        if bytes.get(pos) == Some(&b'=') && bytes.get(pos + 1) == Some(&b'=') {
            flush_lit!();
            let open_pos = pos; // byte offset of opening `==`
            let inner_start = pos + 2;
            let rest = &s[inner_start..];
            if rest.contains("==") {
                let (children, consumed) = scan_template_chunks(rest, Some("=="), errors);
                if children.is_empty() {
                    errors.push(TemplateError::new(open_pos, empty_inline_msg("==")));
                } else {
                    chunks.push(TemplateChunk::Highlight(children));
                }
                pos = inner_start + consumed;
            } else {
                errors.push(TemplateError::new(open_pos, unclosed_inline_msg("==")));
                if !rest.is_empty() {
                    let (children, _) = scan_template_chunks(rest, None, errors);
                    chunks.push(TemplateChunk::Highlight(children));
                }
                pos = len;
            }
            lit_start = pos;
            continue;
        }

        // No special form at this position — advance by one byte.
        pos += 1;
    }

    // Flush any trailing literal.
    flush_lit!();

    (chunks, pos)
}

// ─── String search helpers ────────────────────────────────────────────────────

/// Find the byte position of the first occurrence of `needle` in `s` at or
/// after `start`.  Returns `None` if not found.
fn find_str(s: &str, start: usize, needle: &str) -> Option<usize> {
    s[start..].find(needle).map(|rel| rel + start)
}

/// Find the closing single `$` at or after `start`.
///
/// A `$$` pair is NOT treated as a closing delimiter for inline math.
fn find_single_dollar(s: &str, start: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut pos = start;
    while pos < s.len() {
        if bytes[pos] == b'$' {
            if bytes.get(pos + 1) != Some(&b'$') {
                return Some(pos);
            }
            pos += 2;
        } else {
            pos += 1;
        }
    }
    None
}

// ─── Public combinator ───────────────────────────────────────────────────────

/// Parse a field value that may contain `{{ expr }}` template interpolation
/// and inline markup (`**bold**`, `_italic_`, `` `code` ``, `[text](url)`,
/// `^sup^`, `~sub~`, `~~del~~`, `==highlight==`).
///
/// Matches a `Token::StringLit` and scans its content for all special forms
/// in a single unified pass. Each segment becomes the appropriate
/// [`TemplateChunk`] variant. Errors (unterminated constructs, empty spans)
/// are accumulated — never fail-on-first.
///
/// On success, returns a `(Vec<TemplateChunk>, Vec<TemplateError>)` where the
/// first element is the chunk sequence and the second is accumulated errors.
/// Each [`TemplateError`] carries the byte offset of the problematic construct
/// within the field-value string and the human-readable message. Callers that
/// only need the message may call [`TemplateError::into_message()`]; callers
/// that need sub-span precision (e.g., `section_value_parser`) use the offset
/// to create a [`SimpleSpan`] pointing at the opening delimiter.
///
/// # DIR-077-002 §3: Two-phase inline markup architecture
///
/// Phase 1 (this function, parse time): produces structural `TemplateChunk`
/// variants. Phase 2 (eval time, `chunks_to_inline_nodes` in `slideforge-eval`):
/// converts them to `slideforge_types::InlineNode`.
///
/// Math regions (`$...$`, `$$...$$`) disable ALL text-mode inline markup —
/// `**bold**` inside `$...$` is verbatim LaTeX content.
#[must_use]
pub fn template_value<'src, I>()
-> impl Parser<'src, I, (Vec<TemplateChunk>, Vec<TemplateError>), extra::Err<Rich<'src, Token, TSpan>>>
+ Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    select! {
        Token::StringLit(s) => s.to_string()
    }
    .map(|content| {
        let mut errors: Vec<TemplateError> = Vec::new();
        let (chunks, _) = scan_template_chunks(&content, None, &mut errors);
        (chunks, errors)
    })
}

// ─── Test access ─────────────────────────────────────────────────────────────

/// Test-only re-export of `parse_inner_expr`.
///
/// Exposed as `pub(crate)` for `template_inline_markup_tests` which needs
/// to verify that `Expr::Call` is produced for `ref("id")`, `footnote("t")`,
/// etc., without going through the full DSL string-escaping round-trip.
/// (The DSL lexer stores `StringLit` tokens verbatim, so testing
/// `{{ ref("id") }}` through `parse_template_value` would require DSL-level
/// escaping of the inner `"`. Using `parse_inner_expr_for_test` directly
/// avoids that indirection and tests the `expr()` combinator at the call-site
/// where `scan_template_chunks` invokes it.)
#[cfg(test)]
pub(crate) fn parse_inner_expr_for_test(inner_src: &str) -> Result<Expr, ()> {
    parse_inner_expr(inner_src)
}
