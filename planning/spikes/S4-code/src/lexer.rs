// lexer.rs — Indentation-aware lexer for the slideforge mini-DSL spike.
//
// Architecture: Hand-written (not chumsky). The lexer is responsible for the
// indentation logic and all character-level scanning. It produces a
// Vec<(Token, SimpleSpan)> that the chumsky parser consumes as a flat
// token stream.
//
// Indentation rules:
// - Only space characters (U+0020) are allowed in indentation. Tabs are errors.
// - INDENT is emitted when the column-level increases relative to the top of stack.
// - DEDENT is emitted (possibly multiple times) when column-level decreases.
// - Blank lines and pure-comment lines do NOT affect the indent stack.
// - At EOF, all open indentation levels are closed with DEDENT tokens.

use chumsky::span::SimpleSpan;

use crate::token::{LexError, LexErrorKind, SpannedToken, Token};

/// Construct a SimpleSpan from two usize offsets.
/// In chumsky 0.10, SimpleSpan has public fields; use From<Range<usize>>.
fn span(start: usize, end: usize) -> SimpleSpan {
    (start..end).into()
}

/// Lex the entire source string.
///
/// Returns (tokens, errors). Even if there are lex errors, the token stream
/// is as complete as possible so the parser can also run and collect parse errors.
pub fn lex(src: &str) -> (Vec<SpannedToken>, Vec<LexError>) {
    let mut tokens: Vec<SpannedToken> = Vec::new();
    let mut errors: Vec<LexError> = Vec::new();

    // Indent stack: tracks the space-count at each nesting level. Base is 0.
    let mut indent_stack: Vec<usize> = vec![0];

    // Byte position of the first byte of the current line in `src`.
    let mut line_start_byte: usize = 0;
    // 1-based line number for error messages.
    let mut line_number: usize = 1;

    // Iterate line-by-line. split_inclusive preserves the trailing '\n'.
    for raw_line in src.split_inclusive('\n') {
        // Strip trailing \n and \r for scanning purposes.
        let line = raw_line.trim_end_matches('\n').trim_end_matches('\r');

        // --- Phase 1: Count leading whitespace and detect tabs. ---
        let (indent_spaces, tab_error) = count_indent(line, line_start_byte, line_number);
        if let Some(err) = tab_error {
            errors.push(err);
        }

        let content = &line[indent_spaces..];
        let is_blank = content.trim().is_empty();
        let is_comment = content.starts_with('#');

        // --- Phase 2: Emit INDENT/DEDENT for non-blank, non-comment lines. ---
        if !is_blank && !is_comment {
            emit_indent_dedent(
                indent_spaces,
                &mut indent_stack,
                line_start_byte,
                line_number,
                &mut tokens,
                &mut errors,
            );
        }

        // --- Phase 3: Tokenize content of the line. ---
        if !is_blank && !is_comment {
            let content_start_byte = line_start_byte + indent_spaces;
            lex_line(
                content,
                content_start_byte,
                line_number,
                &mut tokens,
                &mut errors,
            );
            // Emit NEWLINE at end of each non-blank, non-comment logical line.
            let nl_pos = line_start_byte + line.len();
            tokens.push((Token::Newline, span(nl_pos, nl_pos)));
        }

        line_start_byte += raw_line.len();
        line_number += 1;
    }

    // --- Phase 4: Close all open indentation levels at EOF. ---
    let eof_pos = src.len();
    while indent_stack.len() > 1 {
        indent_stack.pop();
        tokens.push((Token::Dedent, span(eof_pos, eof_pos)));
    }

    // Emit EOF token.
    tokens.push((Token::Eof, span(eof_pos, eof_pos)));

    (tokens, errors)
}

/// Count leading spaces on a line. Returns (space_count, Option<LexError>).
/// Tabs anywhere in the leading whitespace produce an error.
fn count_indent(line: &str, line_start_byte: usize, line_num: usize) -> (usize, Option<LexError>) {
    let mut spaces = 0usize;
    let mut error: Option<LexError> = None;

    for (i, ch) in line.char_indices() {
        match ch {
            ' ' => spaces += 1,
            '\t' => {
                let byte_pos = line_start_byte + i;
                if error.is_none() {
                    error = Some(LexError {
                        span: span(byte_pos, byte_pos + 1),
                        kind: LexErrorKind::TabInIndent { line: line_num },
                    });
                }
                // Treat tab as 1 space to recover and continue.
                spaces += 1;
            }
            _ => break,
        }
    }

    (spaces, error)
}

/// Emit INDENT or DEDENT tokens based on the new indentation level vs. stack.
fn emit_indent_dedent(
    new_spaces: usize,
    stack: &mut Vec<usize>,
    line_start_byte: usize,
    line_num: usize,
    tokens: &mut Vec<SpannedToken>,
    errors: &mut Vec<LexError>,
) {
    let current = *stack.last().unwrap();

    if new_spaces > current {
        // Opening a new block.
        stack.push(new_spaces);
        tokens.push((Token::Indent, span(line_start_byte, line_start_byte + new_spaces)));
    } else if new_spaces < current {
        // Closing one or more blocks.
        loop {
            let top = *stack.last().unwrap();
            if top <= new_spaces {
                break;
            }
            stack.pop();
            tokens.push((Token::Dedent, span(line_start_byte, line_start_byte)));
        }
        // Verify the new level matches something on the stack.
        if *stack.last().unwrap() != new_spaces {
            errors.push(LexError {
                span: span(line_start_byte, line_start_byte + new_spaces),
                kind: LexErrorKind::UnmatchedDedent {
                    line: line_num,
                    found_spaces: new_spaces,
                },
            });
            // Push to recover — treat this as a new valid level.
            stack.push(new_spaces);
        }
    }
    // new_spaces == current: same level, no token.
}

/// Tokenize the non-whitespace content of a single line.
/// `byte_offset` is the byte position of `line[0]` in the original source.
fn lex_line(
    line: &str,
    byte_offset: usize,
    line_num: usize,
    tokens: &mut Vec<SpannedToken>,
    errors: &mut Vec<LexError>,
) {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;

    while i < len {
        let abs = byte_offset + i; // absolute byte position in original src

        match bytes[i] {
            // Inline whitespace — skip.
            b' ' | b'\t' => { i += 1; }

            // Comment — rest of line.
            b'#' => { i = len; }

            // Colon.
            b':' => {
                tokens.push((Token::Colon, span(abs, abs + 1)));
                i += 1;
            }

            // Dash (bullet list item).
            b'-' => {
                tokens.push((Token::Dash, span(abs, abs + 1)));
                i += 1;
            }

            // Percent.
            b'%' => {
                tokens.push((Token::Percent, span(abs, abs + 1)));
                i += 1;
            }

            // Square brackets.
            b'[' => { tokens.push((Token::LBracket, span(abs, abs + 1))); i += 1; }
            b']' => { tokens.push((Token::RBracket, span(abs, abs + 1))); i += 1; }

            // `{{` interpolation open, or bare `{` error.
            b'{' => {
                if i + 1 < len && bytes[i + 1] == b'{' {
                    tokens.push((Token::LBrace, span(abs, abs + 2)));
                    i += 2;
                } else {
                    errors.push(LexError {
                        span: span(abs, abs + 1),
                        kind: LexErrorKind::UnexpectedChar { ch: '{', line: line_num, col: i + 1 },
                    });
                    i += 1;
                }
            }

            // `}}` interpolation close, or bare `}` error.
            b'}' => {
                if i + 1 < len && bytes[i + 1] == b'}' {
                    tokens.push((Token::RBrace, span(abs, abs + 2)));
                    i += 2;
                } else {
                    errors.push(LexError {
                        span: span(abs, abs + 1),
                        kind: LexErrorKind::UnexpectedChar { ch: '}', line: line_num, col: i + 1 },
                    });
                    i += 1;
                }
            }

            // `@{` math interpolation open, or `@include`, `@import`, `@if`, `@elif`, `@else`, `@for`.
            b'@' => {
                if i + 1 < len && bytes[i + 1] == b'{' {
                    tokens.push((Token::AtBrace, span(abs, abs + 2)));
                    i += 2;
                } else {
                    let rest = &line[i..];
                    if let Some(tok) = match_at_directive(rest) {
                        let tok_len = directive_len(&tok);
                        tokens.push((tok, span(abs, abs + tok_len)));
                        i += tok_len;
                    } else {
                        errors.push(LexError {
                            span: span(abs, abs + 1),
                            kind: LexErrorKind::UnexpectedChar { ch: '@', line: line_num, col: i + 1 },
                        });
                        i += 1;
                    }
                }
            }

            // `$$` display math, or `$` inline math.
            b'$' => {
                if i + 1 < len && bytes[i + 1] == b'$' {
                    tokens.push((Token::DollarDollar, span(abs, abs + 2)));
                    i += 2;
                } else {
                    tokens.push((Token::Dollar, span(abs, abs + 1)));
                    i += 1;
                }
            }

            // Double-quoted string (single or triple).
            b'"' => {
                if i + 2 < len && bytes[i + 1] == b'"' && bytes[i + 2] == b'"' {
                    // Triple-quoted: scan in-line only for the spike.
                    // Real implementation would scan across lines using the full src.
                    let rest = &line[i..];
                    let (tok, advance, maybe_err) = lex_triple_string_inline(rest, abs, line_num);
                    if let Some(err) = maybe_err {
                        errors.push(err);
                    }
                    let tok_end = abs + advance;
                    tokens.push((tok, span(abs, tok_end)));
                    i += advance;
                } else {
                    let rest = &line[i..];
                    let (tok, advance, maybe_err) = lex_string(rest, abs, line_num);
                    if let Some(err) = maybe_err {
                        errors.push(err);
                    }
                    let tok_end = abs + advance;
                    tokens.push((tok, span(abs, tok_end)));
                    i += advance;
                }
            }

            // Digit: integer or float. Preserved as string — no numeric coercion.
            b'0'..=b'9' => {
                let start = i;
                i += 1;
                while i < len && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                // Check for float: `<digits>.<digit>`.
                let is_float = i + 1 < len && bytes[i] == b'.' && bytes[i + 1].is_ascii_digit();
                if is_float {
                    i += 1; // consume '.'
                    while i < len && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                    let text = &line[start..i];
                    let span_start = byte_offset + start;
                    let span_end = byte_offset + i;
                    tokens.push((Token::Float(text.to_string()), span(span_start, span_end)));
                } else {
                    let text = &line[start..i];
                    let span_start = byte_offset + start;
                    let span_end = byte_offset + i;
                    tokens.push((Token::Int(text.to_string()), span(span_start, span_end)));
                }
            }

            // Identifier or keyword: [a-zA-Z_][a-zA-Z0-9_]*
            b if b.is_ascii_alphabetic() || b == b'_' => {
                let start = i;
                i += 1;
                while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let text = &line[start..i];
                let span_start = byte_offset + start;
                let span_end = byte_offset + i;
                let tok = classify_keyword(text);
                tokens.push((tok, span(span_start, span_end)));
            }

            // Unexpected character.
            b => {
                errors.push(LexError {
                    span: span(abs, abs + 1),
                    kind: LexErrorKind::UnexpectedChar {
                        ch: b as char,
                        line: line_num,
                        col: i + 1,
                    },
                });
                i += 1;
            }
        }
    }
}

/// Scan a double-quoted string starting at `fragment[0] = '"'`.
/// Returns (Token::Str, bytes_consumed, Option<LexError>).
fn lex_string(fragment: &str, start_abs: usize, line_num: usize) -> (Token, usize, Option<LexError>) {
    debug_assert!(fragment.starts_with('"'));
    let bytes = fragment.as_bytes();
    let mut i = 1usize; // skip opening '"'
    let mut content = String::new();
    loop {
        if i >= bytes.len() {
            return (
                Token::Str(content),
                i,
                Some(LexError {
                    span: span(start_abs, start_abs + i),
                    kind: LexErrorKind::UnterminatedString { line: line_num },
                }),
            );
        }
        match bytes[i] {
            b'"' => { i += 1; break; }
            b'\\' if i + 1 < bytes.len() => {
                match bytes[i + 1] {
                    b'"'  => { content.push('"');  i += 2; }
                    b'\\' => { content.push('\\'); i += 2; }
                    b'n'  => { content.push('\n'); i += 2; }
                    b't'  => { content.push('\t'); i += 2; }
                    c     => { content.push(c as char); i += 2; }
                }
            }
            ch => { content.push(ch as char); i += 1; }
        }
    }
    (Token::Str(content), i, None)
}

/// Scan a triple-quoted string within a single line fragment (spike simplification).
/// Real implementation would scan across physical lines.
fn lex_triple_string_inline(fragment: &str, start_abs: usize, start_line: usize) -> (Token, usize, Option<LexError>) {
    debug_assert!(fragment.starts_with(r#"""""#));
    let bytes = fragment.as_bytes();
    let mut i = 3usize; // skip opening `"""`
    loop {
        if i + 3 > bytes.len() {
            let content = fragment[3..].to_string();
            return (
                Token::Str(content),
                fragment.len(),
                Some(LexError {
                    span: span(start_abs, start_abs + fragment.len()),
                    kind: LexErrorKind::UnterminatedTripleString { start_line },
                }),
            );
        }
        if &bytes[i..i + 3] == b"\"\"\"" {
            let content = fragment[3..i].to_string();
            return (Token::Str(content), i + 3, None);
        }
        i += 1;
    }
}

/// Classify a bareword as a keyword token or plain Ident.
fn classify_keyword(word: &str) -> Token {
    match word {
        "true"  => Token::True,
        "false" => Token::False,
        "in"    => Token::In,
        _       => Token::Ident(word.to_string()),
    }
}

/// Match `@directive` at the start of `rest`. Returns the token or None.
fn match_at_directive(rest: &str) -> Option<Token> {
    if rest.starts_with("@include") { return Some(Token::Include); }
    if rest.starts_with("@import")  { return Some(Token::Import); }
    if rest.starts_with("@elif")    { return Some(Token::Elif); }
    if rest.starts_with("@else")    { return Some(Token::Else); }
    if rest.starts_with("@if")      { return Some(Token::If); }
    if rest.starts_with("@for")     { return Some(Token::For); }
    None
}

/// Return the byte length of the directive keyword.
fn directive_len(tok: &Token) -> usize {
    match tok {
        Token::Include => 8,  // "@include"
        Token::Import  => 7,  // "@import"
        Token::If      => 3,  // "@if"
        Token::Elif    => 5,  // "@elif"
        Token::Else    => 5,  // "@else"
        Token::For     => 4,  // "@for"
        _              => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok_kinds(src: &str) -> Vec<Token> {
        let (toks, errs) = lex(src);
        assert!(errs.is_empty(), "unexpected lex errors: {errs:?}");
        toks.into_iter().map(|(t, _)| t).collect()
    }

    #[test]
    fn test_simple_assignment() {
        let src = "title: \"Hello\"\n";
        let kinds = tok_kinds(src);
        assert_eq!(
            kinds,
            vec![
                Token::Ident("title".into()),
                Token::Colon,
                Token::Str("Hello".into()),
                Token::Newline,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_indent_dedent() {
        let src = "slide title:\n    title: \"Hello\"\n";
        let kinds = tok_kinds(src);
        assert!(kinds.contains(&Token::Indent), "missing INDENT: {kinds:?}");
        assert!(kinds.contains(&Token::Dedent), "missing DEDENT: {kinds:?}");
    }

    #[test]
    fn test_tab_in_indent_produces_error() {
        let src = "\ttitle: \"Hello\"\n";
        let (_toks, errs) = lex(src);
        assert!(
            errs.iter().any(|e| matches!(e.kind, LexErrorKind::TabInIndent { .. })),
            "expected TabInIndent error, got: {errs:?}"
        );
    }

    #[test]
    fn test_blank_lines_ignored_for_indentation() {
        // Blank line between two statements in the same block — should NOT close/open indent.
        let src = "slide title:\n    title: \"T1\"\n\n    subtitle: \"S1\"\n";
        let (toks, errs) = lex(src);
        assert!(errs.is_empty(), "unexpected errors: {errs:?}");
        let indent_count = toks.iter().filter(|(t, _)| *t == Token::Indent).count();
        let dedent_count = toks.iter().filter(|(t, _)| *t == Token::Dedent).count();
        assert_eq!(indent_count, 1, "expected 1 INDENT, got {indent_count}");
        assert_eq!(dedent_count, 1, "expected 1 DEDENT, got {dedent_count}");
    }

    #[test]
    fn test_no_implicit_type_coercion() {
        // "NO" must stay as Ident("NO"), not a boolean.
        let src = "flag: NO\n";
        let (toks, _) = lex(src);
        assert!(
            toks.iter().any(|(t, _)| *t == Token::Ident("NO".to_string())),
            "NO should be Ident(\"NO\"), not coerced: {toks:?}"
        );
        assert!(
            !toks.iter().any(|(t, _)| matches!(t, Token::True | Token::False)),
            "NO must not produce a boolean token"
        );

        // "1.10" must stay as Float("1.10"), not rounded.
        let src2 = "val: 1.10\n";
        let (toks2, _) = lex(src2);
        assert!(
            toks2.iter().any(|(t, _)| *t == Token::Float("1.10".to_string())),
            "1.10 should produce Float(\"1.10\"), got: {toks2:?}"
        );
    }

    #[test]
    fn test_collect_multiple_errors() {
        // Two tab errors on separate lines — both should be collected.
        let src = "\ttitle: \"A\"\n\tsubtitle: \"B\"\n";
        let (_toks, errs) = lex(src);
        let tab_errors: Vec<_> = errs.iter()
            .filter(|e| matches!(e.kind, LexErrorKind::TabInIndent { .. }))
            .collect();
        assert!(tab_errors.len() >= 2, "expected >= 2 tab errors, got {}", tab_errors.len());
    }

    #[test]
    fn test_spans_are_valid_byte_offsets() {
        let src = "title: \"Hello\"\nsubtitle: \"World\"\n";
        let (toks, _) = lex(src);
        for (tok, s) in &toks {
            assert!(
                s.start <= src.len(),
                "token {tok:?} span.start={} > src.len()={}", s.start, src.len()
            );
            assert!(
                s.end <= src.len(),
                "token {tok:?} span.end={} > src.len()={}", s.end, src.len()
            );
            assert!(s.start <= s.end, "token {tok:?} has inverted span");
        }
    }
}
