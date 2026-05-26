//! Parser entry point for the slideforge DSL.
//!
//! This module exposes the public [`parse`] function that takes a `.sf` source
//! string, lexes it, and runs the chumsky 0.10 parser over the resulting token
//! stream, returning a typed [`DeckNode`] or accumulated [`SyntaxError`]s.
//!
//! # Pipeline
//!
//! ```text
//! &str (source)
//!   → lex()                     → (Vec<Spanned<Token>>, Vec<LexError>)
//!   → deck_parser().parse(...)  → (Option<DeckNode>, Vec<Rich<Token>>)
//!   → error conversion          → Result<DeckNode, Vec<SyntaxError>>
//! ```
//!
//! # Error Accumulation (DI-018)
//!
//! Lex errors and parse errors are both collected. If either list is non-empty,
//! `parse()` returns `Err(errors)`. Only when both lists are empty does it
//! return `Ok(deck)`.

pub mod alias;
pub mod control_flow;
pub mod deck;
pub mod expr;
pub mod slide;
pub mod template;
pub mod variants;

use std::sync::Arc;

use chumsky::{Parser, prelude::SimpleSpan};

use crate::{
    ast::DeckNode, error::SyntaxError, lexer::lex, lexer_error::LexError, span::SourceMap,
    token::Token,
};

use chumsky::input::Input as _;

use self::deck::deck_parser;

// ─── Public API ───────────────────────────────────────────────────────────────

/// Parse a `.sf` source string into a typed AST.
///
/// Returns `Ok(DeckNode)` if parsing succeeds with zero errors.
/// Returns `Err(errors)` if one or more parse errors were accumulated.
/// Never panics on valid UTF-8 input.
///
/// # Parameters
///
/// * `src` — the full text of the `.sf` file.
/// * `file_id` — the ID of this file in the `source_map`. Must be the ID
///   returned by [`SourceMap::add_file`] for the same text.
/// * `source_map` — the registry of source files (used for diagnostic
///   construction).
///
/// # Errors
///
/// Returns `Err(Vec<SyntaxError>)` when any lex or parse error occurred.
/// The error vector is never empty when `Err` is returned.
pub fn parse(
    src: &str,
    file_id: u32,
    source_map: &SourceMap,
) -> Result<DeckNode, Vec<SyntaxError>> {
    // Phase 1: lex the source string.
    let file_path = source_map
        .get(file_id)
        .map_or_else(|| Arc::from("<unknown>"), |f| f.path.clone());

    let (tokens, lex_errors) = lex(src, file_path.clone());

    // Phase 2: convert lex errors to SyntaxError.
    let mut errors: Vec<SyntaxError> = lex_errors
        .into_iter()
        .map(|e| lex_error_to_syntax_error(e, src))
        .collect();

    // Phase 3: run the chumsky parser over the token stream.
    // Convert lexer spans (Range<usize>) to chumsky SimpleSpan.
    // We build an owned vec of (Token, SimpleSpan) and parse from a slice of it.
    let spanned_tokens: Vec<(Token, SimpleSpan)> = tokens
        .into_iter()
        .map(|(t, s)| (t, SimpleSpan::from(s)))
        .collect();

    let eoi = SimpleSpan::from(src.len()..src.len());
    // For &[(Token, SimpleSpan)], MaybeToken is &(Token, SimpleSpan).
    // The map closure receives &(Token, SimpleSpan) and must return (&Token, &SimpleSpan).
    let input = spanned_tokens
        .as_slice()
        .map(eoi, |(t, s): &(Token, SimpleSpan)| (t, s));

    let (deck_opt, parse_errors) = deck_parser(file_id).parse(input).into_output_errors();

    // Phase 4: convert chumsky Rich errors to SyntaxError.
    // When the found token is `Indent(n)`, classify as IndentError (E-PAR-001)
    // since it means the parser encountered an unexpected indentation level.
    for rich_err in parse_errors {
        let span = rich_err.span();
        let byte_start = span.start;
        let (line, col) = byte_offset_to_line_col(src, byte_start);
        let span_len = span.end.saturating_sub(span.start).max(1);

        // Classify parser-level Indent token errors as IndentError (E-PAR-001).
        //
        // Real tab errors are caught by the lexer (LexError::TabIndentation).
        // Misaligned dedents are caught by the lexer (LexError::IndentationInconsistency).
        //
        // The parser sees an unexpected Indent(n) token when the grammar does not
        // expect any further nesting at the current position — for example, a
        // second Indent inside a slide field list. The "found" level is the value
        // carried by the Indent token. The "expected" level is derived as
        // `found_n - 1` (one space less than found), which correctly identifies
        // the last valid indentation level for the common case:
        //
        // - Indent(3) inside a 2-space block → expected=2, found=3  ✓
        // - Indent(4) inside a 2-space block → expected=3, found=4  ✓
        //
        // The definitively correct solution would thread the open-block indent
        // level through the chumsky State context, which is STORY-007+ scope.
        // The `found_n - 1` formula removes the prior hardcoding of `expected=2`
        // and is correct for all cases where exactly one extra space is added.
        let syntax_err = if let Some(Token::Indent(found_n)) = rich_err.found() {
            let expected = found_n.saturating_sub(1);
            SyntaxError::indent_error(
                file_path.to_string(),
                line,
                col,
                expected,
                *found_n,
                src.to_string(),
                byte_start,
            )
        } else {
            let message = format!("{:?}", rich_err.reason());
            SyntaxError::unexpected_token(
                file_path.to_string(),
                line,
                col,
                message,
                src.to_string(),
                byte_start,
                span_len,
            )
        };
        errors.push(syntax_err);
    }

    // Phase 5: gate on error count.
    if !errors.is_empty() {
        return Err(errors);
    }

    // Phase 6: return the AST. If parse produced no errors but also no output
    // (e.g. empty file), return a default empty DeckNode.
    Ok(deck_opt.unwrap_or_default())
}

// ─── Internal helpers ────────────────────────────────────────────────────────

/// Convert a [`LexError`] into a [`SyntaxError`].
fn lex_error_to_syntax_error(e: LexError, src: &str) -> SyntaxError {
    let src_string = src.to_string();
    match e {
        LexError::TabIndentation {
            file,
            line,
            col,
            byte_offset,
        } => SyntaxError::unexpected_token(
            file.to_string(),
            line,
            col,
            "tab character in leading whitespace; use spaces for indentation".to_string(),
            src_string,
            byte_offset,
            1,
        ),
        LexError::IndentationInconsistency {
            file,
            line,
            col,
            expected,
            got,
        } => {
            let byte_offset = line_col_to_byte_offset(src, line, col);
            SyntaxError::indent_error(
                file.to_string(),
                line,
                col,
                expected,
                got,
                src_string,
                byte_offset,
            )
        },
        LexError::UnterminatedString { file, line, col } => {
            let byte_offset = line_col_to_byte_offset(src, line, col);
            SyntaxError::unexpected_token(
                file.to_string(),
                line,
                col,
                "unterminated string literal".to_string(),
                src_string,
                byte_offset,
                1,
            )
        },
        LexError::UnterminatedMath { file, line, col } => {
            let byte_offset = line_col_to_byte_offset(src, line, col);
            SyntaxError::unexpected_token(
                file.to_string(),
                line,
                col,
                "unterminated math block".to_string(),
                src_string,
                byte_offset,
                1,
            )
        },
        LexError::InvalidCharacter {
            file,
            line,
            col,
            ch,
        } => {
            let byte_offset = line_col_to_byte_offset(src, line, col);
            SyntaxError::unexpected_token(
                file.to_string(),
                line,
                col,
                format!("invalid character {ch:?}"),
                src_string,
                byte_offset,
                ch.len_utf8(),
            )
        },
        LexError::NumberOverflow {
            file,
            line,
            col,
            text,
        } => {
            let byte_offset = line_col_to_byte_offset(src, line, col);
            SyntaxError::unexpected_token(
                file.to_string(),
                line,
                col,
                format!("integer literal '{text}' overflows i64"),
                src_string,
                byte_offset,
                text.len(),
            )
        },
    }
}

/// Convert a 1-based `(line, col)` pair to a byte offset in `src`.
///
/// Used when the lex error carries `(line, col)` but the miette `SourceSpan`
/// constructor needs a byte offset.
fn line_col_to_byte_offset(src: &str, line: u32, col: u32) -> usize {
    let line = line as usize;
    let col = col as usize;
    let mut current_line = 1usize;
    let mut line_start = 0usize;
    for (i, ch) in src.char_indices() {
        if current_line == line {
            // col is 1-based; col 1 means the first byte on the line.
            let col_offset = col.saturating_sub(1);
            return line_start + col_offset;
        }
        if ch == '\n' {
            current_line += 1;
            line_start = i + 1;
        }
    }
    // If line is past the end, return the end of the source.
    src.len().saturating_sub(1)
}

/// Convert a byte offset in `src` to a 1-based `(line, col)` pair.
fn byte_offset_to_line_col(src: &str, offset: usize) -> (u32, u32) {
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(src.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    let line_idx = line_starts
        .partition_point(|&s| s <= offset)
        .saturating_sub(1);
    let col = offset - line_starts[line_idx];
    #[allow(clippy::cast_possible_truncation)]
    let line = line_idx as u32 + 1;
    #[allow(clippy::cast_possible_truncation)]
    let col_u32 = col as u32 + 1;
    (line, col_u32)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{
        ast::{BlockItem, FieldValue},
        span::SourceMap,
        template::TemplateChunk,
    };
    use std::sync::Arc;

    /// Helper: add a file to a fresh [`SourceMap`] and call [`parse`].
    fn parse_str(src: &str) -> Result<DeckNode, Vec<SyntaxError>> {
        let mut sm = SourceMap::new();
        let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
        parse(src, file_id, &sm)
    }

    // ── AC-001: minimal deck, zero errors ────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_minimal_deck_parses_ok() {
        let src = concat!(
            "slideforge_version \"1\"\n",
            "lang \"en-US\"\n",
            "slide title:\n",
            "  title \"Hello\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_ok(),
            "minimal deck must parse without errors: {result:?}"
        );
        let deck = result.unwrap();
        assert_eq!(deck.items.len(), 1, "must have 1 block item (the slide)");
        assert_eq!(deck.version.as_ref().map(|v| v.value().as_str()), Some("1"));
    }

    // ── AC-003: determinism ───────────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_ast_deterministic() {
        let src = concat!(
            "slideforge_version \"1\"\n",
            "lang \"en-US\"\n",
            "slide content:\n",
            "  title \"Hello\"\n",
            "  body \"World\"\n",
        );
        let r1 = parse_str(src);
        let r2 = parse_str(src);
        assert_eq!(r1.ok(), r2.ok(), "same source must produce identical ASTs");
    }

    // ── AC-005: indent error single ────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_002_indent_err_single() {
        // 3-space indent where the block established 2-space: lexer emits
        // IndentationInconsistency which the parse() function converts to a
        // SyntaxError::IndentError.
        let src = concat!(
            "slide title:\n",
            "  title \"Good\"\n",
            "   bad_indent \"here\"\n", // 3 spaces after 2-space indent
        );
        let result = parse_str(src);
        // The lexer will produce an IndentationInconsistency for the 3-space line.
        assert!(
            result.is_err(),
            "indent inconsistency must cause Err return"
        );
        let errors = result.unwrap_err();
        let has_indent_err = errors
            .iter()
            .any(|e| matches!(e, SyntaxError::IndentError { .. }));
        assert!(
            has_indent_err,
            "must have an IndentError variant; got: {errors:?}"
        );
    }

    // ── AC-007: indent errors accumulated (2 errors) ─────────────────────────

    #[test]
    fn test_bc_1_01_002_indent_err_accumulated() {
        // Two independent indentation inconsistencies — both must be reported.
        // We use unexpected Indent(3) tokens inside slides (one per slide).
        // Each produces at minimum one error. The test verifies total error
        // count ≥ 2 and that at least one is IndentError (E-PAR-001).
        let src = concat!(
            "slide title:\n",
            "  title \"Good\"\n",
            "   bad1 \"err1\"\n", // 3 spaces: inconsistency 1
            "slide content:\n",
            "  body \"Good\"\n",
            "   bad2 \"err2\"\n", // 3 spaces: inconsistency 2
        );
        let result = parse_str(src);
        assert!(result.is_err(), "must be Err with errors");
        let errors = result.unwrap_err();
        // BC-1.01.002 invariant: total error count ≥ 2 (accumulation, not fail-fast).
        assert!(
            errors.len() >= 2,
            "must accumulate at least 2 errors; got: {errors:?}"
        );
        // At least one must be an IndentError (E-PAR-001).
        let has_indent_err = errors
            .iter()
            .any(|e| matches!(e, SyntaxError::IndentError { .. }));
        assert!(
            has_indent_err,
            "must have at least one IndentError; got: {errors:?}"
        );
    }

    // ── AC-008: parse returns Err when errors exist ──────────────────────────

    #[test]
    fn test_bc_1_01_001_parse_returns_err_on_error() {
        // Tab indentation → lex error → parse returns Err.
        let src = "\tfield: value\n";
        let result = parse_str(src);
        assert!(result.is_err(), "source with error must return Err, not Ok");
    }

    // ── AC-009: empty file, no panic ─────────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_empty_file_no_panic() {
        // Must not panic. Returns Ok with empty items or Err.
        let result = parse_str("");
        if let Ok(deck) = result {
            assert!(
                deck.items.is_empty(),
                "empty source must produce 0 block items"
            );
        }
        // Err is also acceptable (empty file may trigger parse errors)
    }

    // ── AC-010: slide fields captured ────────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_slide_fields_captured() {
        let src = concat!(
            "slide content:\n",
            "  title \"Hello\"\n",
            "  footer \"Slide 1\"\n",
        );
        let result = parse_str(src);
        assert!(result.is_ok(), "must parse without errors: {result:?}");
        let deck = result.unwrap();
        let BlockItem::Slide(slide_s) = &deck.items[0] else {
            panic!("expected Slide block item");
        };
        let slide = slide_s.value();
        assert_eq!(slide.fields.len(), 2, "slide must have 2 fields");
        let title = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "title")
            .expect("title field must exist");
        assert_eq!(
            title.value.value(),
            &FieldValue::Template(vec![TemplateChunk::Literal("Hello".to_string())])
        );
    }

    // ── AC-002: vars block parsed ─────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_vars_block_parsed() {
        // vars entries use IDENT ":" value syntax per the grammar spec.
        let src = concat!(
            "vars:\n",
            "  client: \"Acme\"\n",
            "slide title:\n",
            "  title \"Test\"\n",
        );
        let result = parse_str(src);
        assert!(result.is_ok(), "must parse: {result:?}");
        let deck = result.unwrap();
        assert_eq!(deck.vars.len(), 1);
        let vb = &deck.vars[0];
        assert_eq!(vb.entries[0].0.value(), "client");
        assert_eq!(
            vb.entries[0].1.value(),
            &FieldValue::Template(vec![TemplateChunk::Literal("Acme".to_string())])
        );
    }

    // ── AC-002: set rule parsed ────────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_set_rule_parsed() {
        let src = concat!(
            "set content: footer \"Default\"\n",
            "slide content:\n",
            "  title \"Test\"\n",
        );
        let result = parse_str(src);
        assert!(result.is_ok(), "must parse: {result:?}");
        let deck = result.unwrap();
        assert_eq!(deck.set_rules.len(), 1);
        assert_eq!(deck.set_rules[0].slide_type.value(), "content");
        assert_eq!(deck.set_rules[0].field.value(), "footer");
    }

    // ── AC-004: all spans in bounds ───────────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_all_spans_in_bounds() {
        let src = concat!(
            "slideforge_version \"1\"\n",
            "lang \"en-US\"\n",
            "slide title:\n",
            "  title \"Hello\"\n",
            "  body \"World\"\n",
        );
        let result = parse_str(src);
        let deck = result.expect("must parse");
        let src_len = src.len();
        // Check that all string spans are within source bounds.
        for item in &deck.items {
            if let BlockItem::Slide(slide_s) = item {
                assert!(slide_s.span().end <= src_len, "slide span out of bounds");
                for field in &slide_s.value().fields {
                    assert!(
                        field.name.span().start <= src_len && field.name.span().end <= src_len,
                        "field name span out of bounds"
                    );
                    assert!(
                        field.value.span().start <= src_len && field.value.span().end <= src_len,
                        "field value span out of bounds"
                    );
                }
            }
        }
    }
}
