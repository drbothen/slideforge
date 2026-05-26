//! Hand-written byte-level lexer for the slideforge DSL.
//!
//! # Design Rationale
//!
//! The lexer is hand-written (not a chumsky combinator) for three reasons:
//!
//! 1. **Tab detection must happen at the byte level** before the combinator
//!    layer runs, so that it is deterministic and amenable to formal
//!    verification (VP-001).
//! 2. **Mode switching** (`$...$` math mode) is cleaner as explicit state
//!    than as chumsky combinators.
//! 3. **Indentation tracking** requires an imperative stack that is awkward
//!    to express in a combinator grammar.
//!
//! # Error Accumulation (DI-018)
//!
//! The lexer **never fails fast**. Every error is pushed into an internal
//! `Vec<LexError>` and scanning continues. The final `(tokens, errors)` pair
//! lets the parser decide whether to proceed despite errors.
//!
//! # Spanned Tokens
//!
//! Every token carries a `Range<usize>` of byte offsets into the source
//! string (see [`Spanned`]). The lexer pre-computes line starts once so
//! that byte offsets can be converted to `(line, col)` in O(log n) when
//! constructing error values.

use std::sync::Arc;

use crate::lexer_error::LexError;
use crate::token::{LexerMode, Spanned, Token};

// ─── Public API ──────────────────────────────────────────────────────────────

/// Lex a slideforge DSL source string into a stream of spanned tokens.
///
/// Returns a `(tokens, errors)` pair. When `errors` is non-empty the token
/// stream may still be partially or fully populated — the lexer accumulates
/// all errors in a single pass (DI-018) and continues scanning after each one.
///
/// # Parameters
///
/// * `source` — the full text of the `.sf` file, already loaded into memory.
/// * `file` — the file path used in error messages (typically the `.sf`
///   file path, or `"<unknown>"` for synthetic input).
///
/// # Guarantees
///
/// * The last token in the returned `tokens` vec is always [`Token::Eof`].
/// * Tab characters in leading whitespace always produce
///   [`LexError::TabIndentation`] entries (AC-002).
/// * Tab characters inside string literals or comments never produce errors
///   (AC-004, AC-005).
/// * The function is pure and non-recursive — it will not overflow the stack
///   on large files (EC-009).
#[must_use]
pub fn lex(source: &str, file: Arc<str>) -> (Vec<Spanned<Token>>, Vec<LexError>) {
    let mut state = LexerState::new(source, file);
    state.run();
    (state.tokens, state.errors)
}

// ─── Internal helpers ─────────────────────────────────────────────────────

/// Pre-compute the byte offset of the first character on each line.
///
/// `line_starts[0]` is always `0` (the first line starts at offset 0).
/// `line_starts[n]` is the offset of the character *after* the `\n` that
/// ends line `n-1`.
fn compute_line_starts(src: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(src.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

/// Convert a byte offset into a 1-based `(line, col)` pair.
///
/// Uses a binary search over `line_starts` — O(log n) per call.
fn offset_to_line_col(offset: usize, line_starts: &[usize]) -> (u32, u32) {
    // partition_point returns the number of elements ≤ offset, which is one
    // past the line index.
    let line_idx = line_starts
        .partition_point(|&s| s <= offset)
        .saturating_sub(1);
    let col = offset - line_starts[line_idx];
    // Both are 1-indexed. Files with more than u32::MAX lines are not supported;
    // the truncation is intentional for practical source files.
    #[allow(clippy::cast_possible_truncation)]
    let line = line_idx as u32 + 1;
    #[allow(clippy::cast_possible_truncation)]
    let col_u32 = col as u32 + 1;
    (line, col_u32)
}

// ─── Lexer state ──────────────────────────────────────────────────────────

/// Internal mutable state for the lexer.
struct LexerState<'src> {
    /// The source text being scanned.
    src: &'src str,
    /// Source bytes for fast byte-level access.
    bytes: &'src [u8],
    /// Current byte offset into `src`.
    pos: usize,
    /// Current lexer mode.
    mode: LexerMode,
    /// Indentation stack; the bottom is always `0`.
    indent_stack: Vec<usize>,
    /// Accumulated errors (never cleared, only pushed).
    errors: Vec<LexError>,
    /// Accumulated tokens.
    tokens: Vec<Spanned<Token>>,
    /// Pre-computed line starts for O(log n) line:col lookups.
    line_starts: Vec<usize>,
    /// Source file name (carried into error values).
    file: Arc<str>,
}

impl<'src> LexerState<'src> {
    fn new(src: &'src str, file: Arc<str>) -> Self {
        let line_starts = compute_line_starts(src);
        LexerState {
            src,
            bytes: src.as_bytes(),
            pos: 0,
            mode: LexerMode::Text,
            indent_stack: vec![0],
            errors: Vec::new(),
            tokens: Vec::new(),
            line_starts,
            file,
        }
    }

    // ── Primitive accessors ──────────────────────────────────────────────

    /// Peek at the byte at `pos + offset` without advancing.
    fn peek(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.pos + offset).copied()
    }

    /// Return the current byte, or `None` if at end of input.
    fn current(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    /// Convert the current `pos` to `(line, col)`.
    fn line_col(&self) -> (u32, u32) {
        offset_to_line_col(self.pos, &self.line_starts)
    }

    /// Emit a token with the span `[start, self.pos)`.
    fn emit(&mut self, tok: Token, start: usize) {
        self.tokens.push((tok, start..self.pos));
    }

    // ── Main scan loop ───────────────────────────────────────────────────

    /// Drive the lexer until the full source has been consumed.
    fn run(&mut self) {
        while self.pos < self.src.len() {
            // Every top-level iteration starts at the beginning of a line's
            // indentation or somewhere inside a line.  We dispatch based on
            // what character is next.
            self.scan_line();
        }
        // Emit pending Dedents for any remaining indentation levels.
        while self.indent_stack.last().copied().unwrap_or(0) > 0 {
            self.indent_stack.pop();
            let end = self.pos;
            self.tokens.push((Token::Dedent, end..end));
        }
        self.emit(Token::Eof, self.pos);
    }

    /// Scan one logical line: indentation, then tokens until newline / EOF.
    fn scan_line(&mut self) {
        // ── Step 0: strip a leading `\r` (Windows `\r\n` line endings) ──
        // The `\r` is consumed silently; only the `\n` is treated as the
        // line terminator so that Windows files produce identical token
        // streams to Unix files.
        if self.current() == Some(b'\r') {
            self.pos += 1;
        }

        // ── Step 1: scan leading whitespace (indentation) ───────────────
        let indent_start = self.pos;
        let mut indent_spaces: usize = 0;
        let mut had_tab = false;

        // Scan the run of spaces and tabs at the start of the line.
        loop {
            match self.current() {
                Some(b' ') => {
                    self.pos += 1;
                    indent_spaces += 1;
                },
                Some(b'\t') => {
                    // Tab in leading whitespace — record error, skip the byte.
                    let (line, col) = self.line_col();
                    self.errors.push(LexError::TabIndentation {
                        file: Arc::clone(&self.file),
                        line,
                        col,
                        byte_offset: self.pos,
                    });
                    self.pos += 1;
                    had_tab = true;
                    // Do NOT count tabs toward indent_spaces — treat them as
                    // zero-width so the indent stack stays consistent.
                },
                _ => break,
            }
        }

        // If the line is blank or a comment, skip indentation processing.
        match self.current() {
            None => return,
            Some(b'\r' | b'\n') => {
                // Blank line (Unix `\n` or Windows `\r\n`).
                // Skip `\r` if present, then the `\n`.
                if self.current() == Some(b'\r') {
                    self.pos += 1;
                }
                if self.current() == Some(b'\n') {
                    self.pos += 1;
                }
                return;
            },
            Some(b'#') => {
                // Comment line — skip all bytes up to (not including) the
                // line terminator, then consume it (handling `\r\n`).
                while matches!(self.current(), Some(b) if b != b'\n' && b != b'\r') {
                    self.pos += 1;
                }
                if self.current() == Some(b'\r') {
                    self.pos += 1;
                }
                if self.current() == Some(b'\n') {
                    self.pos += 1;
                }
                return;
            },
            _ => {},
        }

        // ── Step 2: emit Indent / Dedent tokens ─────────────────────────
        if !had_tab {
            // Only process indentation changes when the line had no tabs
            // (tab errors are already accumulated; we skip indent adjustment
            // on tab lines to avoid corrupting the stack).
            let current_level = *self.indent_stack.last().unwrap_or(&0);

            if indent_spaces > current_level {
                // Indentation increased — push new level.
                self.indent_stack.push(indent_spaces);
                self.emit(Token::Indent(indent_spaces), indent_start);
            } else if indent_spaces < current_level {
                // Indentation decreased — pop until we match or detect error.
                let mut matched = false;
                while let Some(&top) = self.indent_stack.last() {
                    if top == indent_spaces {
                        matched = true;
                        break;
                    }
                    if top < indent_spaces {
                        // Overshot — the current indent doesn't match any
                        // level on the stack.
                        break;
                    }
                    self.indent_stack.pop();
                    let end = self.pos;
                    self.tokens.push((Token::Dedent, end..end));
                }
                if !matched {
                    // Current level doesn't match any stack entry.
                    let expected = *self.indent_stack.last().unwrap_or(&0);
                    let (line, col) = offset_to_line_col(indent_start, &self.line_starts);
                    self.errors.push(LexError::IndentationInconsistency {
                        file: Arc::clone(&self.file),
                        line,
                        col,
                        expected,
                        got: indent_spaces,
                    });
                }
            }
            // If indent_spaces == current_level: same level, no tokens emitted.
        }

        // ── Step 3: scan tokens on the remainder of the line ────────────
        self.scan_line_tokens();
    }

    /// Scan tokens from the current position up to (but not including) the
    /// next unescaped newline or EOF. Emits a `Newline` token at the end.
    fn scan_line_tokens(&mut self) {
        loop {
            match self.current() {
                None => {
                    // EOF inside line — no Newline token needed; `run()` will
                    // emit pending Dedents and Eof.
                    return;
                },
                Some(b'\r') => {
                    // Bare `\r` mid-line (unusual) or the `\r` in a `\r\n`
                    // pair that wasn't consumed by `scan_line`.  Skip it; the
                    // `\n` will be seen on the next iteration and emit the
                    // `Newline` token as usual.
                    self.pos += 1;
                },
                Some(b'\n') => {
                    let start = self.pos;
                    self.pos += 1;
                    self.tokens.push((Token::Newline, start..self.pos));
                    return;
                },
                Some(b'#') => {
                    // Comment: skip to end of line (including any `\r`).
                    while matches!(self.current(), Some(b) if b != b'\n' && b != b'\r') {
                        self.pos += 1;
                    }
                },
                Some(b' ' | b'\t') => {
                    // Inline whitespace (spaces / tabs *between* tokens on a
                    // line are not indentation and are silently skipped; tabs
                    // here are fine per AC-004 semantics for mid-line content).
                    self.pos += 1;
                },
                Some(b'"') => self.scan_string(),
                Some(b'$') => self.scan_dollar(),
                Some(b'@') => self.scan_at(),
                Some(b'{') => self.scan_open_brace(),
                Some(b'}') => {
                    let start = self.pos;
                    self.pos += 1;
                    // Check for `}}`
                    if self.current() == Some(b'}') {
                        self.pos += 1;
                    }
                    self.emit(Token::CloseBrace, start);
                },
                Some(b':') => {
                    let start = self.pos;
                    self.pos += 1;
                    self.emit(Token::Colon, start);
                },
                Some(b'|') => {
                    let start = self.pos;
                    self.pos += 1;
                    self.emit(Token::Pipe, start);
                },
                Some(b'-' | b'0'..=b'9') => self.scan_number(),
                Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => self.scan_ident(),
                Some(other) => {
                    // Any byte that doesn't start a recognized token.
                    let (line, col) = self.line_col();
                    // Decode the full char (may be multi-byte UTF-8).  Use
                    // `get` to avoid panicking if `pos` is not on a char
                    // boundary; fall back to interpreting the raw byte as a
                    // Latin-1 scalar value so we advance at least one byte.
                    let ch = self
                        .src
                        .get(self.pos..)
                        .and_then(|s| s.chars().next())
                        .unwrap_or(other as char);
                    self.errors.push(LexError::InvalidCharacter {
                        file: Arc::clone(&self.file),
                        line,
                        col,
                        ch,
                    });
                    // Advance past this code-point.
                    self.pos += ch.len_utf8();
                },
            }
        }
    }

    // ── Token-specific scanners ──────────────────────────────────────────

    /// Scan a double-quoted string literal.
    ///
    /// Tabs inside a string literal are preserved and do NOT produce a
    /// `LexError::TabIndentation` (AC-004 / EC-001).
    fn scan_string(&mut self) {
        let start = self.pos;
        let (line, col) = self.line_col();
        self.pos += 1; // consume opening `"`
        let content_start = self.pos;

        loop {
            match self.current() {
                None | Some(b'\n') => {
                    // Unterminated string.
                    self.errors.push(LexError::UnterminatedString {
                        file: Arc::clone(&self.file),
                        line,
                        col,
                    });
                    let content = &self.src[content_start..self.pos];
                    self.tokens
                        .push((Token::StringLit(Arc::from(content)), start..self.pos));
                    return;
                },
                Some(b'\\') => {
                    // Escape sequence — skip the backslash, then skip the
                    // FULL escaped character as a UTF-8 code-point.  A bare
                    // `self.pos += 1` would split multi-byte sequences (e.g.
                    // `\ñ` is 2 UTF-8 bytes) and corrupt the char-boundary
                    // invariant, causing a panic on the next `&self.src[pos..]`.
                    self.pos += 1; // skip `\`
                    // Do NOT consume a line terminator or EOF as the escaped
                    // character: `"hello\` followed by newline must produce
                    // UnterminatedString, not silently continue on the next
                    // line.  Leave pos unchanged so the next loop iteration
                    // hits the `\n` / `\r` / EOF branch and emits the error.
                    match self.current() {
                        None | Some(b'\n') => {
                            // Let next iteration trigger unterminated-string.
                        },
                        _ => {
                            if let Some(s) = self.src.get(self.pos..)
                                && let Some(ch) = s.chars().next()
                            {
                                self.pos += ch.len_utf8();
                            }
                        },
                    }
                },
                Some(b'"') => {
                    let content = &self.src[content_start..self.pos];
                    self.pos += 1; // consume closing `"`
                    self.tokens
                        .push((Token::StringLit(Arc::from(content)), start..self.pos));
                    return;
                },
                Some(_) => {
                    // Advance by one full Unicode code-point.  Use a safe
                    // get + chars().next() to avoid panicking if `pos` somehow
                    // lands on a non-char boundary (defensive guard).
                    let Some(ch) = self.src.get(self.pos..).and_then(|s| s.chars().next()) else {
                        break;
                    };
                    self.pos += ch.len_utf8();
                },
            }
        }
    }

    /// Scan a `$` or `$$` delimiter and switch lexer mode accordingly.
    fn scan_dollar(&mut self) {
        let start = self.pos;
        self.pos += 1; // consume first `$`

        if self.current() == Some(b'$') {
            // `$$` — display math.
            self.pos += 1;
            match self.mode {
                LexerMode::MathDisplay => {
                    // Closing `$$` — return to Text mode.
                    self.mode = LexerMode::Text;
                    self.emit(Token::DollarDouble, start);
                },
                LexerMode::Text | LexerMode::Math => {
                    // Opening `$$`.
                    self.mode = LexerMode::MathDisplay;
                    self.emit(Token::DollarDouble, start);
                    self.scan_math_content(false);
                },
            }
        } else {
            // Single `$` — inline math.
            match self.mode {
                LexerMode::Math => {
                    // Closing `$` — return to Text mode.
                    self.mode = LexerMode::Text;
                    self.emit(Token::DollarSingle, start);
                },
                LexerMode::Text | LexerMode::MathDisplay => {
                    // Opening `$`.
                    self.mode = LexerMode::Math;
                    self.emit(Token::DollarSingle, start);
                    self.scan_math_content(true);
                },
            }
        }
    }

    /// Collect math content until the matching closing delimiter.
    ///
    /// `single` is `true` for `$...$` and `false` for `$$...$$`.
    ///
    /// Emits a `MathContent` token, then the closing delimiter token, then
    /// resets the mode.
    ///
    /// This function is **iterative**, not recursive.  Recursion was removed
    /// to satisfy EC-009 (iterative, not recursive) and to prevent stack
    /// overflow on deeply-nested `@{...}` interpolations inside long math
    /// blocks.  When `@{...}` is found, we emit content + `AtBrace`, scan the
    /// interpolation body, then `continue` the outer loop from the new
    /// position instead of calling `scan_math_content` again.
    fn scan_math_content(&mut self, single: bool) {
        let (open_line, open_col) = self.line_col();
        // `segment_start` tracks the beginning of the current raw-math slice.
        // It is reset to `self.pos` each time we emit a `MathContent` chunk
        // (after an `@{...}` interpolation or at the start of the function).
        let mut segment_start = self.pos;

        loop {
            match self.current() {
                None => {
                    // EOF without closing delimiter.
                    let content = &self.src[segment_start..self.pos];
                    if !content.is_empty() {
                        self.tokens.push((
                            Token::MathContent(Arc::from(content)),
                            segment_start..self.pos,
                        ));
                    }
                    self.errors.push(LexError::UnterminatedMath {
                        file: Arc::clone(&self.file),
                        line: open_line,
                        col: open_col,
                    });
                    self.mode = LexerMode::Text;
                    return;
                },
                Some(b'$') if single => {
                    // Potential closing `$` for inline math.
                    if self.peek(1) == Some(b'$') {
                        // This is `$$` — not our closing delimiter; treat as content.
                        self.pos += 2;
                    } else {
                        let content = &self.src[segment_start..self.pos];
                        if !content.is_empty() {
                            self.tokens.push((
                                Token::MathContent(Arc::from(content)),
                                segment_start..self.pos,
                            ));
                        }
                        let close_start = self.pos;
                        self.pos += 1;
                        self.mode = LexerMode::Text;
                        self.emit(Token::DollarSingle, close_start);
                        return;
                    }
                },
                Some(b'$') if !single => {
                    // Potential closing `$$` for display math.
                    if self.peek(1) == Some(b'$') {
                        let content = &self.src[segment_start..self.pos];
                        if !content.is_empty() {
                            self.tokens.push((
                                Token::MathContent(Arc::from(content)),
                                segment_start..self.pos,
                            ));
                        }
                        let close_start = self.pos;
                        self.pos += 2;
                        self.mode = LexerMode::Text;
                        self.emit(Token::DollarDouble, close_start);
                        return;
                    }
                    self.pos += 1;
                },
                Some(b'@') if self.peek(1) == Some(b'{') => {
                    // `@{` inside math — emit content accumulated so far,
                    // emit `AtBrace`, scan the interpolation body, then
                    // continue the loop (iterative, not recursive).
                    if self.pos > segment_start {
                        let content = &self.src[segment_start..self.pos];
                        self.tokens.push((
                            Token::MathContent(Arc::from(content)),
                            segment_start..self.pos,
                        ));
                    }
                    let ab_start = self.pos;
                    self.pos += 2;
                    self.emit(Token::AtBrace, ab_start);
                    self.scan_interpolation_body();
                    // Reset the segment start to resume accumulating math
                    // content after the closing `}` of the interpolation.
                    segment_start = self.pos;
                    // `continue` — re-enter the loop without recursing.
                },
                Some(_) => {
                    // Advance by one full Unicode code-point.
                    let Some(ch) = self.src.get(self.pos..).and_then(|s| s.chars().next()) else {
                        break;
                    };
                    self.pos += ch.len_utf8();
                },
            }
        }
    }

    /// Scan the body of a `@{ ... }` interpolation: tokens up to `}`.
    fn scan_interpolation_body(&mut self) {
        loop {
            match self.current() {
                None | Some(b'\n') => return,
                Some(b'}') => {
                    let start = self.pos;
                    self.pos += 1;
                    self.emit(Token::CloseBrace, start);
                    return;
                },
                Some(b' ' | b'\t') => {
                    self.pos += 1;
                },
                Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => self.scan_ident(),
                Some(b'0'..=b'9' | b'-') => self.scan_number(),
                Some(b'"') => self.scan_string(),
                Some(_) => {
                    // Advance by one full Unicode code-point.
                    let Some(ch) = self.src.get(self.pos..).and_then(|s| s.chars().next()) else {
                        break;
                    };
                    self.pos += ch.len_utf8();
                },
            }
        }
    }

    /// Scan an `@` directive or `@{` interpolation.
    fn scan_at(&mut self) {
        let start = self.pos;
        self.pos += 1; // consume `@`

        // Check for `@{` — math interpolation.
        if self.current() == Some(b'{') {
            self.pos += 1;
            self.emit(Token::AtBrace, start);
            return;
        }

        // Otherwise emit the `@` token and let the next iteration scan the
        // following identifier.
        self.emit(Token::At, start);
    }

    /// Scan `{{` or a bare `{`.
    fn scan_open_brace(&mut self) {
        let start = self.pos;
        self.pos += 1; // consume first `{`
        if self.current() == Some(b'{') {
            self.pos += 1;
            self.emit(Token::DoubleBrace, start);
        } else {
            // Bare `{` — not a recognized token on its own; emit as invalid.
            let (line, col) = offset_to_line_col(start, &self.line_starts);
            self.errors.push(LexError::InvalidCharacter {
                file: Arc::clone(&self.file),
                line,
                col,
                ch: '{',
            });
        }
    }

    /// Scan an integer or floating-point number literal.
    fn scan_number(&mut self) {
        let start = self.pos;
        // Optional leading minus.
        if self.current() == Some(b'-') {
            self.pos += 1;
            // Must be followed by a digit.
            if !matches!(self.current(), Some(b'0'..=b'9')) {
                // Not a number — emit `-` as invalid and backtrack handled
                // by the next scan_line_tokens iteration.
                let (line, col) = offset_to_line_col(start, &self.line_starts);
                self.errors.push(LexError::InvalidCharacter {
                    file: Arc::clone(&self.file),
                    line,
                    col,
                    ch: '-',
                });
                return;
            }
        }
        while matches!(self.current(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        // Check for decimal point.
        if self.current() == Some(b'.') && matches!(self.peek(1), Some(b'0'..=b'9')) {
            self.pos += 1; // consume `.`
            while matches!(self.current(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
            let text = &self.src[start..self.pos];
            // f64 parsing: the only failure mode is a NaN/Infinity literal,
            // which cannot arise from digit-only input.  Overflow produces
            // f64::INFINITY rather than an error; we preserve that as-is
            // since there is no meaningful recovery value.
            let val: f64 = text.parse().unwrap_or(0.0);
            self.emit(Token::FloatLit(ordered_float::OrderedFloat(val)), start);
        } else {
            let text = &self.src[start..self.pos];
            if let Ok(val) = text.parse::<i64>() {
                self.emit(Token::IntLit(val), start);
            } else {
                // Integer too large for i64 — emit an error and use 0 as a
                // recovery value so that scanning continues (DI-018).
                let (line, col) = offset_to_line_col(start, &self.line_starts);
                self.errors.push(LexError::NumberOverflow {
                    file: Arc::clone(&self.file),
                    line,
                    col,
                    text: Arc::from(text),
                });
                self.emit(Token::IntLit(0), start);
            }
        }
    }

    /// Scan an identifier or keyword (`true`, `false`).
    fn scan_ident(&mut self) {
        let start = self.pos;
        while matches!(
            self.current(),
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-')
        ) {
            self.pos += 1;
        }
        let text = &self.src[start..self.pos];
        let tok = match text {
            "true" => Token::BoolLit(true),
            "false" => Token::BoolLit(false),
            other => Token::Ident(Arc::from(other)),
        };
        self.emit(tok, start);
    }
}

// ─── Unit Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

    fn lex_str(s: &str) -> (Vec<Spanned<Token>>, Vec<LexError>) {
        lex(s, Arc::from("test.sf"))
    }

    // ── AC-001 — error accumulation ───────────────────────────────────────

    #[test]
    fn test_bc_1_01_003_error_accumulation_does_not_fail_fast() {
        // Two tab-indented lines — BOTH errors must appear in the vec.
        let src = "\tline1: a\n\tline2: b\n";
        let (tokens, errors) = lex_str(src);
        assert_eq!(errors.len(), 2, "should accumulate both errors");
        // Token stream should still be non-empty (at minimum contains Eof).
        assert!(!tokens.is_empty());
    }

    // ── AC-002/AC-003 — tab in leading whitespace ─────────────────────────

    #[test]
    fn test_bc_1_01_003_tab_leading_whitespace_produces_error() {
        let src = "\tfield: value\n";
        let (_, errors) = lex_str(src);
        assert_eq!(errors.len(), 1);
        let LexError::TabIndentation { line, col, .. } = &errors[0] else {
            panic!("expected TabIndentation, got {:?}", errors[0]);
        };
        assert_eq!(*line, 1);
        assert_eq!(*col, 1);
    }

    #[test]
    fn test_bc_1_01_003_tab_error_message_format() {
        let src = "\tfield: value\n";
        let (_, errors) = lex_str(src);
        assert_eq!(errors.len(), 1);
        let msg = errors[0].to_string();
        assert!(
            msg.contains("test.sf:1:1"),
            "message must include file:line:col, got: {msg}"
        );
        assert!(
            msg.contains("spaces for indentation"),
            "message must mention spaces, got: {msg}"
        );
    }

    // ── AC-004 — tab inside string literal ────────────────────────────────

    #[test]
    fn test_bc_1_01_003_tab_inside_string_no_error() {
        let src = "title \"contains\ttab\"\n";
        let (tokens, errors) = lex_str(src);
        assert!(errors.is_empty(), "tab in string should not produce error");
        let has_string_lit = tokens
            .iter()
            .any(|(t, _)| matches!(t, Token::StringLit(s) if s.contains('\t')));
        assert!(
            has_string_lit,
            "token stream should contain StringLit with tab"
        );
    }

    // ── AC-005 — tab inside comment ───────────────────────────────────────

    #[test]
    fn test_bc_1_01_003_tab_inside_comment_no_error() {
        let src = "# comment\twith tab\n";
        let (_, errors) = lex_str(src);
        assert!(errors.is_empty(), "tab in comment should not produce error");
    }

    // ── AC-006 — mixed tab + indentation errors both accumulated ──────────

    #[test]
    fn test_bc_1_01_003_multiple_tab_lines_all_accumulated() {
        let src = "\tfield1: value\n\tfield2: value\n";
        let (_, errors) = lex_str(src);
        assert_eq!(errors.len(), 2, "should accumulate both tab errors");
        assert!(
            errors
                .iter()
                .all(|e| matches!(e, LexError::TabIndentation { .. }))
        );
    }

    #[test]
    fn test_bc_1_01_003_mixed_space_and_tab_errors_both_accumulated() {
        // Line 1: tab — produces TabIndentation (indent stack remains [0])
        // Line 2: 4-space indent — valid new level, stack becomes [0, 4]
        // Line 3: 2-space indent — dedent but 2 is not on stack [0, 4], produces IndentationInconsistency
        let src = "\tfield1: val\n    field2: val\n  field3: val\n";
        let (_, errors) = lex_str(src);
        assert!(
            errors.len() >= 2,
            "should have both tab and indentation errors; got {errors:?}"
        );
        let has_tab = errors
            .iter()
            .any(|e| matches!(e, LexError::TabIndentation { .. }));
        let has_indent = errors
            .iter()
            .any(|e| matches!(e, LexError::IndentationInconsistency { .. }));
        assert!(has_tab, "should have tab error");
        assert!(has_indent, "should have indentation inconsistency error");
    }

    // ── AC-008 — token variants ────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_003_ident_token() {
        let src = "title\n";
        let (tokens, errors) = lex_str(src);
        assert!(errors.is_empty());
        assert!(
            tokens
                .iter()
                .any(|(t, _)| matches!(t, Token::Ident(s) if s.as_ref() == "title"))
        );
    }

    #[test]
    fn test_bc_1_01_003_bool_lit_true() {
        let src = "true\n";
        let (tokens, errors) = lex_str(src);
        assert!(errors.is_empty());
        assert!(
            tokens
                .iter()
                .any(|(t, _)| matches!(t, Token::BoolLit(true)))
        );
    }

    #[test]
    fn test_bc_1_01_003_bool_lit_false() {
        let src = "false\n";
        let (tokens, errors) = lex_str(src);
        assert!(errors.is_empty());
        assert!(
            tokens
                .iter()
                .any(|(t, _)| matches!(t, Token::BoolLit(false)))
        );
    }

    #[test]
    fn test_bc_1_01_003_int_lit() {
        let src = "42\n";
        let (tokens, errors) = lex_str(src);
        assert!(errors.is_empty());
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::IntLit(42))));
    }

    #[test]
    fn test_bc_1_01_003_float_lit() {
        let src = "1.5\n";
        let (tokens, errors) = lex_str(src);
        assert!(errors.is_empty());
        assert!(tokens.iter().any(|(t, _)| {
            if let Token::FloatLit(f) = t {
                (f.0 - 1.5_f64).abs() < 1e-9
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_bc_1_01_003_colon_token() {
        let src = "key: val\n";
        let (tokens, errors) = lex_str(src);
        assert!(errors.is_empty());
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Colon)));
    }

    #[test]
    fn test_bc_1_01_003_pipe_token() {
        let src = "a | b\n";
        let (tokens, errors) = lex_str(src);
        assert!(errors.is_empty());
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Pipe)));
    }

    // ── AC-009 — spanned tokens carry position ────────────────────────────

    #[test]
    fn test_bc_1_01_003_spanned_tokens_carry_position() {
        let src = "title \"My Deck\"\n";
        let (tokens, _) = lex_str(src);
        for (_, span) in &tokens {
            assert!(
                span.end >= span.start,
                "span must be valid (end >= start), got {span:?}"
            );
        }
        // `title` starts at byte 0
        let (_, title_span) = tokens
            .iter()
            .find(|(t, _)| matches!(t, Token::Ident(s) if s.as_ref() == "title"))
            .expect("title token not found");
        assert_eq!(title_span.start, 0);
    }

    // ── AC-010 — math mode switching ──────────────────────────────────────

    #[test]
    fn test_bc_1_01_003_math_mode_switch_dollar_single() {
        // Inside a string literal, `$` does NOT switch mode.
        let src = "title \"$x^2$\"\n";
        let (tokens, errors) = lex_str(src);
        assert!(errors.is_empty());
        let has_string = tokens.iter().any(|(t, _)| matches!(t, Token::StringLit(_)));
        assert!(has_string);
    }

    #[test]
    fn test_bc_1_01_003_math_mode_standalone_dollar() {
        // Outside a string, `$` triggers math mode.
        let src = "body $x^2$\n";
        let (tokens, errors) = lex_str(src);
        assert!(
            errors.is_empty(),
            "no errors expected for valid math, got: {errors:?}"
        );
        let has_dollar_single = tokens.iter().any(|(t, _)| matches!(t, Token::DollarSingle));
        assert!(has_dollar_single, "should produce DollarSingle token");
        let has_math_content = tokens
            .iter()
            .any(|(t, _)| matches!(t, Token::MathContent(_)));
        assert!(has_math_content, "should produce MathContent token");
    }

    #[test]
    fn test_bc_1_01_003_math_mode_display_dollar_double() {
        let src = "body $$E = mc^2$$\n";
        let (tokens, errors) = lex_str(src);
        assert!(errors.is_empty(), "got errors: {errors:?}");
        let has_dollar_double = tokens.iter().any(|(t, _)| matches!(t, Token::DollarDouble));
        assert!(has_dollar_double, "should produce DollarDouble token");
    }

    // ── AC-011 — {{ / @{ distinction ─────────────────────────────────────

    #[test]
    fn test_bc_1_01_003_double_brace_text_mode() {
        // `{{` outside a string produces DoubleBrace.
        let src = "{{ name }}\n";
        let (tokens, errors) = lex_str(src);
        assert!(errors.is_empty(), "got errors: {errors:?}");
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::DoubleBrace)));
    }

    #[test]
    fn test_bc_1_01_003_at_brace_vs_double_brace_mode_distinction() {
        let src = "body @{ var }\n";
        let (tokens, errors) = lex_str(src);
        assert!(errors.is_empty(), "got errors: {errors:?}");
        let has_at_brace = tokens.iter().any(|(t, _)| matches!(t, Token::AtBrace));
        assert!(has_at_brace, "@{{ should produce AtBrace token");
    }

    // ── EC-005 — empty source file ────────────────────────────────────────

    #[test]
    fn test_bc_1_01_003_empty_source_file_eof_only() {
        let (tokens, errors) = lex_str("");
        assert!(errors.is_empty());
        // Should contain at least Eof.
        assert!(!tokens.is_empty());
        assert!(
            matches!(tokens.last(), Some((Token::Eof, _))),
            "last token must be Eof"
        );
    }

    // ── EC-006 — comment-only file ────────────────────────────────────────

    #[test]
    fn test_bc_1_01_003_comment_only_no_errors() {
        let src = "# This is a comment\n# Another comment\n";
        let (_, errors) = lex_str(src);
        assert!(errors.is_empty(), "comment-only file should have no errors");
    }

    // ── Indentation tokens ────────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_003_indent_token_on_nesting() {
        let src = "slide title:\n  title \"Hello\"\n";
        let (tokens, errors) = lex_str(src);
        assert!(errors.is_empty(), "got errors: {errors:?}");
        let has_indent = tokens.iter().any(|(t, _)| matches!(t, Token::Indent(2)));
        assert!(has_indent, "should produce Indent(2) token");
    }

    #[test]
    fn test_bc_1_01_003_dedent_token_on_unnesting() {
        let src = "slide title:\n  title \"Hello\"\nnext_field: val\n";
        let (tokens, errors) = lex_str(src);
        assert!(errors.is_empty(), "got errors: {errors:?}");
        let has_dedent = tokens.iter().any(|(t, _)| matches!(t, Token::Dedent));
        assert!(has_dedent, "should produce Dedent token when block closes");
    }

    // ── Unterminated string ───────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_003_unterminated_string_produces_error() {
        let src = "title \"unterminated\n";
        let (_, errors) = lex_str(src);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, LexError::UnterminatedString { .. })),
            "should have UnterminatedString error"
        );
    }

    // ── Unterminated math ─────────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_003_unterminated_math_produces_error() {
        let src = "body $x^2\n";
        let (_, errors) = lex_str(src);
        // Unterminated inline math — the newline ends the line without closing $.
        // Depending on lexer policy (math content spans multiple lines or not),
        // the error may come at EOF or on next newline.
        // The key invariant: at least one UnterminatedMath or UnterminatedString.
        let has_error = errors.iter().any(|e| {
            matches!(
                e,
                LexError::UnterminatedMath { .. } | LexError::UnterminatedString { .. }
            )
        });
        assert!(
            has_error,
            "unterminated math should produce error, got: {errors:?}"
        );
    }

    // ── @{ in math mode ───────────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_003_at_brace_inside_math_mode() {
        let src = "body $@{var}$\n";
        let (tokens, errors) = lex_str(src);
        assert!(errors.is_empty(), "got errors: {errors:?}");
        let has_at_brace = tokens.iter().any(|(t, _)| matches!(t, Token::AtBrace));
        assert!(has_at_brace, "should produce AtBrace inside math mode");
    }

    // ── FIX-001: UTF-8 multi-byte escape in string literals ──────────────

    /// Regression test for F-001: `\` followed by a multi-byte UTF-8
    /// character must not panic.  Previously the lexer advanced `pos` by
    /// exactly 1 after the backslash, which could land on a non-char
    /// boundary and cause a panic on the next `&src[pos..]` slice.
    #[test]
    fn test_escape_multibyte_utf8() {
        // `ñ` is U+00F1, encoded as 0xC3 0xB1 (2 bytes).
        // The escaped sequence `\ñ` must be skipped without panicking.
        let src = "title \"hello\\ñworld\"\n";
        let (tokens, errors) = lex_str(src);
        assert!(
            errors.is_empty(),
            "multi-byte UTF-8 escape must not produce errors, got: {errors:?}"
        );
        let has_string = tokens.iter().any(|(t, _)| {
            if let Token::StringLit(s) = t {
                s.contains("hello") && s.contains("world")
            } else {
                false
            }
        });
        assert!(
            has_string,
            "should produce a StringLit containing the content around the escape"
        );
    }

    // ── FIX-003: Windows \r\n line endings ───────────────────────────────

    /// Windows files use `\r\n` as line terminators.  The lexer must produce
    /// the same token stream as the equivalent Unix (`\n`) file — no errors
    /// and no extra tokens.
    #[test]
    fn test_windows_line_endings() {
        let unix_src = "title \"hello\"\n";
        let windows_src = "title \"hello\"\r\n";
        let (unix_tokens, unix_errors) = lex_str(unix_src);
        let (win_tokens, win_errors) = lex_str(windows_src);
        assert!(
            unix_errors.is_empty(),
            "unix: unexpected errors: {unix_errors:?}"
        );
        assert!(
            win_errors.is_empty(),
            "windows: unexpected errors: {win_errors:?}"
        );
        // Token kinds must match (spans differ due to the extra `\r` byte, so
        // we compare only the token variants, not the byte ranges).
        let unix_kinds: Vec<_> = unix_tokens.iter().map(|(t, _)| format!("{t:?}")).collect();
        let win_kinds: Vec<_> = win_tokens.iter().map(|(t, _)| format!("{t:?}")).collect();
        assert_eq!(
            unix_kinds, win_kinds,
            "Windows and Unix line endings must produce identical token streams"
        );
    }

    // ── FIX-004: Integer overflow produces LexError, not silent 0 ────────

    /// An integer literal that does not fit in `i64` must produce a
    /// `LexError::NumberOverflow` error rather than silently clamping to `0`.
    #[test]
    fn test_number_overflow() {
        // 10^22 — well beyond i64::MAX (≈ 9.2 × 10^18).
        let src = "count 9999999999999999999999\n";
        let (tokens, errors) = lex_str(src);
        let has_overflow = errors
            .iter()
            .any(|e| matches!(e, LexError::NumberOverflow { .. }));
        assert!(
            has_overflow,
            "overflowing integer literal must produce NumberOverflow error, got: {errors:?}"
        );
        // A recovery `IntLit(0)` should still be in the token stream so that
        // parsing can continue (DI-018 error accumulation).
        let has_int_lit = tokens.iter().any(|(t, _)| matches!(t, Token::IntLit(_)));
        assert!(has_int_lit, "should still emit a recovery IntLit token");
    }

    // ── Backslash-before-newline must not bypass single-line enforcement ──

    #[test]
    fn test_backslash_before_newline_unterminated() {
        // `"hello\` followed by a newline must produce UnterminatedString, NOT
        // silently stitch lines together.  This guards against the escape-handler
        // consuming `\n` as the "escaped character" and continuing on the next
        // line, which would violate the DSL single-line string contract.
        let src = "title \"hello\\\nworld\"\n";
        let (_, errors) = lex_str(src);
        assert!(
            !errors.is_empty(),
            "backslash before newline must produce at least one error"
        );
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, LexError::UnterminatedString { .. })),
            "expected UnterminatedString error, got: {errors:?}"
        );
    }

    #[test]
    fn test_backslash_at_eof_unterminated() {
        // `"hello\` with no following character at all must produce
        // UnterminatedString, not loop indefinitely or panic.
        let src = "title \"hello\\";
        let (_, errors) = lex_str(src);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, LexError::UnterminatedString { .. })),
            "backslash at EOF must produce UnterminatedString; got: {errors:?}"
        );
    }

    // ── Snapshot test ─────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_003_snapshot_minimal_deck() {
        let src = r#"
slideforge_version "1"
lang "en-US"
title "Test"

slide title:
  title "Hello"
"#;
        let (tokens, errors) = lex_str(src);
        assert!(
            errors.is_empty(),
            "minimal deck should have no lex errors, got: {errors:?}"
        );
        insta::assert_debug_snapshot!("minimal_deck_tokens", tokens);
    }
}
