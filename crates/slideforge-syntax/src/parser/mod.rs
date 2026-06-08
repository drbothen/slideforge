//! Parser entry point for the slideforge DSL.
//!
//! This module exposes the public [`parse`] function that takes a `.sf` source
//! string, lexes it, and runs the chumsky 0.10 parser over the resulting token
//! stream, returning a typed [`ParseResult`] or accumulated [`SyntaxError`]s.
//!
//! # Pipeline
//!
//! ```text
//! &str (source)
//!   → lex()                     → (Vec<Spanned<Token>>, Vec<LexError>)
//!   → deck_parser().parse(...)  → (Option<DeckNode>, Vec<Rich<Token>>)
//!   → error conversion          → Result<ParseResult, Vec<SyntaxError>>
//! ```
//!
//! # Error Accumulation (DI-018)
//!
//! Lex errors and parse errors are both collected. If either list is non-empty,
//! `parse()` returns `Err(errors)`. Only when both lists are empty does it
//! return `Ok(ParseResult { deck, warnings })`.
//!
//! # Warnings
//!
//! Non-fatal diagnostics (e.g., missing `slideforge_version`) are returned in
//! `ParseResult::warnings` even on a successful parse. Fatal errors cause an
//! `Err` return; warnings never do.

pub mod alias;
pub mod control_flow;
pub mod deck;
pub mod expr;
pub mod section;
pub mod section_group;
pub mod shape;
pub mod slide;
pub mod template;
pub mod variants;

// STORY-078 Red Gate: failing test suite for section block parser.
// This module references `slideforge_syntax::section::is_register_sub_block_key`
// and `slideforge_syntax::parser::section::section_block_parser`, both of which
// do NOT yet exist — causing compile failures that constitute the Red Gate.
#[cfg(test)]
mod section_tests;

// STORY-077 Red Gate: failing test suite for inline markup parser extension.
// Tests verify that `template_value()` produces structural TemplateChunk variants
// (Bold, Italic, Code, Link, Superscript, Subscript, Strikethrough, Highlight)
// for the corresponding DSL inline markup syntax forms.
// Currently fails because template_value() does not yet recognize these delimiters.
#[cfg(test)]
mod template_inline_markup_tests;

// STORY-082 Red Gate: failing test suite for section group parser (AC-001, AC-002,
// AC-010, AC-011). Tests drive: SectionGroupNode, BlockItem::SectionGroup,
// section_group_parser, E-PAR-023 (empty name), W-PAR-002 (duplicate name).
#[cfg(test)]
mod section_group_tests;

use std::sync::Arc;

use chumsky::{Parser, prelude::SimpleSpan};

use crate::{
    ast::DeckNode, error::SyntaxError, lexer::lex, lexer_error::LexError, span::SourceMap,
    token::Token,
};

use chumsky::input::Input as _;

use self::deck::deck_parser;
use self::template::parse_routing_tag;

// ─── Public API ───────────────────────────────────────────────────────────────

/// The successful result of parsing a `.sf` source file.
///
/// On a successful parse, `parse()` returns `Ok(ParseResult)`.
/// * `deck` — the typed AST.
/// * `warnings` — non-fatal diagnostics accumulated during parsing (e.g., a
///   missing `slideforge_version` declaration).  The parse succeeded even if
///   this list is non-empty.
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// The parsed AST.
    pub deck: DeckNode,
    /// Non-fatal diagnostics (E-PAR-010 missing-version warning, etc.).
    pub warnings: Vec<SyntaxError>,
}

/// Parse a `.sf` source string into a typed AST, accumulating all diagnostics
/// into a [`crate::DiagnosticSink`].
///
/// This is the sink-based entry point for STORY-010's error accumulation
/// infrastructure. All parse-time diagnostics (fatal and non-fatal) are pushed
/// into `sink`. The function returns `Some(DeckNode)` when the parse produced
/// a usable AST (i.e., no fatal errors were encountered), or `None` when any
/// fatal error is present.
///
/// # Parameters
///
/// * `src` — the full text of the `.sf` file.
/// * `file_id` — the ID of this file in the `source_map`.
/// * `source_map` — the registry of source files (used for diagnostic
///   construction).
/// * `sink` — the diagnostic accumulator. All errors and warnings are pushed
///   here; the sink is never cleared by this function.
///
/// # Returns
///
/// * `Some(DeckNode)` — the parse succeeded with zero fatal errors; warnings
///   may have been pushed to `sink`.
/// * `None` — at least one fatal error was pushed to `sink`; the AST is not
///   usable.
#[must_use]
pub fn parse_checked(
    src: &str,
    file_id: u32,
    source_map: &SourceMap,
    sink: &mut crate::sink::DiagnosticSink,
) -> Option<DeckNode> {
    match parse(src, file_id, source_map) {
        Ok(result) => {
            // Push any non-fatal warnings (e.g., missing slideforge_version)
            // into the sink even on a successful parse.
            for warning in result.warnings {
                sink.push(warning);
            }
            Some(result.deck)
        },
        Err(errors) => {
            // Push all fatal errors into the sink and signal failure via None.
            for error in errors {
                sink.push(error);
            }
            None
        },
    }
}

/// Parse a `.sf` source string into a typed AST.
///
/// Returns `Ok(ParseResult)` if parsing succeeds with zero fatal errors.
/// Returns `Err(errors)` if one or more fatal parse errors were accumulated.
/// Never panics on valid UTF-8 input.
///
/// Non-fatal diagnostics (e.g., missing `slideforge_version`) are returned in
/// [`ParseResult::warnings`] on a successful parse.
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
// parse() is a multi-phase error routing function; its length is inherent to
// the number of error code branches it must handle. Extracting phases would
// require threading extra state across helper boundaries — less readable, not safer.
#[allow(clippy::too_many_lines)]
pub fn parse(
    src: &str,
    file_id: u32,
    source_map: &SourceMap,
) -> Result<ParseResult, Vec<SyntaxError>> {
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

    // Phase 3: pre-parse version gate — fail-fast on incompatible version.
    let version_gate_result = pre_parse_version_gate(src, &tokens, &file_path, &mut errors);
    if version_gate_result == VersionGateResult::FatalVersionError {
        return Err(errors);
    }

    // Phase 4: run the chumsky parser over the token stream.
    let spanned_tokens: Vec<(Token, SimpleSpan)> = tokens
        .into_iter()
        .map(|(t, s)| (t, SimpleSpan::from(s)))
        .collect();
    let eoi = SimpleSpan::from(src.len()..src.len());
    let input = spanned_tokens
        .as_slice()
        .map(eoi, |(t, s): &(Token, SimpleSpan)| (t, s));

    let (deck_opt, parse_errors) = deck_parser(file_id).parse(input).into_output_errors();

    // Phase 5: convert chumsky Rich errors to SyntaxError.
    // W-PAR-* → non-fatal warnings; E-PAR-019/020/021 → fatal inline-markup errors;
    // Indent(n) → IndentError; all others → fatal errors.
    let mut parse_time_warnings: Vec<SyntaxError> = Vec::new();

    for rich_err in parse_errors {
        let span = rich_err.span();
        let byte_start = span.start;
        let (line, col) = byte_offset_to_line_col(src, byte_start);
        let span_len = span.end.saturating_sub(span.start).max(1);

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
            // Strip the `Custom("...")` debug wrapper that chumsky adds to
            // `Rich::custom` reasons (F-077-P5-001 fix: prevents sentinel leak).
            let raw_message = format!("{:?}", rich_err.reason());
            let message = strip_custom_wrapper(&raw_message);

            if message.contains("W-PAR-002") {
                // W-PAR-002: duplicate section group name — route to dedicated variant.
                // Extract name from the message (between first and second single-quote).
                let dup_name = extract_w_par_002_name(&message).unwrap_or_default();
                let warning = SyntaxError::duplicate_section_group_name(
                    file_path.to_string(),
                    line,
                    col,
                    dup_name.to_string(),
                    message,
                    src.to_string(),
                    byte_start,
                    span_len,
                );
                parse_time_warnings.push(warning);
                continue;
            }
            if message.contains("W-PAR-") {
                let warning = SyntaxError::unexpected_token(
                    file_path.to_string(),
                    line,
                    col,
                    message,
                    src.to_string(),
                    byte_start,
                    span_len,
                );
                parse_time_warnings.push(warning);
                continue;
            }
            if message.contains("E-PAR-023") {
                // E-PAR-023: empty section group name — route to dedicated variant.
                let syntax_err = SyntaxError::empty_section_group_name(
                    file_path.to_string(),
                    line,
                    col,
                    message,
                    src.to_string(),
                    byte_start,
                    span_len,
                );
                errors.push(syntax_err);
                continue;
            }

            // E-PAR-019/020/021: route via sentinel → InlineMarkupRoute → fatal error.
            //
            // error-taxonomy.md:24 — "Parse Errors (E-PAR) — Always fatal. Build halts
            // with accumulated errors. No output produced." (exit 1, notes 55/57/59).
            // STORY-077 EC-007: "in strict mode (default) the build fails with the
            // accumulated error."
            //
            // KEEP the `continue` so we don't double-push via `errors.push(syntax_err)`
            // below. Error ACCUMULATION is preserved (all E-PAR-019/020/021 errors in a
            // field are collected before the gate at Phase 6 halts the build). Only the
            // DESTINATION sink changes: warnings → errors (fatal path). (F-077-P14-001)
            if let Some(route) = parse_routing_tag(&message) {
                let error = inline_markup_route_to_error(
                    route, &file_path, line, col, src, byte_start, span_len,
                );
                errors.push(error);
                continue;
            }

            if message.contains("E-PAR-008") {
                let name = extract_quoted_name(&message).unwrap_or_default();
                SyntaxError::var_name_collision(
                    file_path.to_string(),
                    line,
                    col,
                    name.to_string(),
                    message.clone(),
                    src.to_string(),
                    byte_start,
                    span_len,
                )
            } else if message.contains("E-PAR-009") {
                SyntaxError::raw_keyword(
                    file_path.to_string(),
                    line,
                    col,
                    message.clone(),
                    src.to_string(),
                    byte_start,
                    span_len,
                )
            } else if message.contains("E-PAR-006") {
                let keyword = extract_quoted_name(&message).unwrap_or_default();
                SyntaxError::reserved_keyword(
                    file_path.to_string(),
                    line,
                    col,
                    keyword.to_string(),
                    message.clone(),
                    src.to_string(),
                    byte_start,
                    span_len,
                )
            } else {
                SyntaxError::unexpected_token(
                    file_path.to_string(),
                    line,
                    col,
                    message,
                    src.to_string(),
                    byte_start,
                    span_len,
                )
            }
        };
        errors.push(syntax_err);
    }

    // Phase 6: gate on error count.
    if !errors.is_empty() {
        return Err(errors);
    }

    // Phase 7: accumulate warnings (W-PAR-* from Phase 5 + missing-version advisory).
    let mut warnings: Vec<SyntaxError> = parse_time_warnings;
    let deck = deck_opt.unwrap_or_default();
    if version_gate_result == VersionGateResult::MissingVersion && !deck.items.is_empty() {
        warnings.push(SyntaxError::version_error(
            file_path.to_string(),
            "E-PAR-010: missing slideforge_version declaration — \
             add 'slideforge_version \"1\"' as the first line of your .sf file"
                .to_string(),
            false, // non-fatal: missing version is a warning, not a hard error
            src.to_string(),
            0,
        ));
    }

    // Phase 8: return the AST with any accumulated warnings.
    Ok(ParseResult { deck, warnings })
}

// ─── Version gate ────────────────────────────────────────────────────────────

/// Result of the pre-parse version gate scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VersionGateResult {
    /// Version is present and compatible (major == 1).
    Compatible,
    /// Version is present but incompatible — a fatal error was pushed.
    FatalVersionError,
    /// No version declaration was found.
    MissingVersion,
}

/// Scan the raw token stream (before the chumsky parse) for a
/// `slideforge_version` declaration.
///
/// If found with a forward-incompatible major version (≠ 1), pushes a fatal
/// [`SyntaxError::VersionError`] to `errors` and returns
/// [`VersionGateResult::FatalVersionError`].
///
/// If found and compatible, returns [`VersionGateResult::Compatible`] —
/// no error is pushed.
///
/// If not found, returns [`VersionGateResult::MissingVersion`] — the caller
/// is responsible for emitting the non-fatal E-PAR-010 warning after the
/// chumsky parse succeeds.
///
/// This implements the **fail-fast version gate** described in BC-1.13.001:
/// AC-002 "no further parsing occurs" when an incompatible version is declared.
fn pre_parse_version_gate(
    src: &str,
    tokens: &[(Token, std::ops::Range<usize>)],
    file_path: &str,
    errors: &mut Vec<SyntaxError>,
) -> VersionGateResult {
    // Scan token pairs: Ident("slideforge_version") followed by StringLit(ver).
    //
    // EC-002: if `slideforge_version` appears AFTER a `slide` token has been
    // seen, it is a misplaced declaration — emit E-PAR-010 fatal and return.
    let mut slide_seen = false;
    let mut i = 0usize;
    while i < tokens.len() {
        let (tok, span) = &tokens[i];

        // Track whether we have seen any `slide` keyword so far.
        if let Token::Ident(name) = tok
            && name.as_ref() == "slide"
        {
            slide_seen = true;
        }

        if let Token::Ident(name) = tok
            && name.as_ref() == "slideforge_version"
        {
            // EC-002: version declaration after a slide block is a fatal error.
            if slide_seen {
                errors.push(SyntaxError::version_error(
                    file_path.to_string(),
                    "E-PAR-010: version declaration must appear in deck metadata at top of file \
                     — 'slideforge_version' found after a slide block"
                        .to_string(),
                    true,
                    src.to_string(),
                    span.start,
                ));
                return VersionGateResult::FatalVersionError;
            }

            // Look for the StringLit immediately after (skipping nothing —
            // the lexer emits them consecutively on the same line).
            if let Some((Token::StringLit(ver), ver_span)) = tokens.get(i + 1) {
                let ver_str = ver.trim().to_string();
                let major_str = ver_str.split('.').next().unwrap_or("");
                let is_whitespace_only = ver_str.trim().is_empty();
                let is_non_numeric = major_str.parse::<u64>().is_err();

                if is_whitespace_only || is_non_numeric {
                    errors.push(SyntaxError::version_error(
                        file_path.to_string(),
                        format!(
                            "E-PAR-010: invalid version string '{ver_str}' — \
                                 the version must be a numeric major version, e.g. \"1\""
                        ),
                        true,
                        src.to_string(),
                        ver_span.start,
                    ));
                    return VersionGateResult::FatalVersionError;
                }

                let major: u64 = major_str.parse().unwrap_or(0);

                // Compatibility rules:
                // - major == 1 → compatible (stable release series).
                // - major == 0 with a minor component → compatible pre-1.0
                //   development version (e.g. "0.1.0" for the slideforge
                //   project's own development builds). Allows developers to
                //   write `.sf` files with the current project version without
                //   a gate error.
                // - major == 0 with NO minor component (bare "0") → rejected.
                // - major >= 2 → forward-incompatible; rejected.
                if major == 1 {
                    // Stable v1.x series: compatible.
                    return VersionGateResult::Compatible;
                }
                // major == 0 or major >= 2: reject.
                let msg = if major >= 2 {
                    format!(
                        "E-PAR-010: forward-incompatible version '{ver_str}' — \
                         this build of slideforge supports version 1.x only"
                    )
                } else {
                    format!(
                        "E-PAR-010: invalid version string '{ver_str}' — \
                         the version must be a numeric major version (1.x), e.g. \"1\" or \"1.0\""
                    )
                };
                errors.push(SyntaxError::version_error(
                    file_path.to_string(),
                    msg,
                    true,
                    src.to_string(),
                    ver_span.start,
                ));
                return VersionGateResult::FatalVersionError;
            }
            // slideforge_version with no following string — unusual; let
            // the chumsky parser generate the appropriate error.
            return VersionGateResult::Compatible;
        }
        i += 1;
    }
    VersionGateResult::MissingVersion
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

/// Convert an [`InlineMarkupRoute`] to the appropriate [`SyntaxError`] fatal error variant.
///
/// Extracted from `parse()` to keep that function within the clippy line-count limit.
/// All three routes (E-PAR-019 / E-PAR-020 / E-PAR-021) are ALWAYS FATAL per
/// error-taxonomy.md:24 ("Parse Errors (E-PAR) — Always fatal. Build halts with
/// accumulated errors. No output produced."). They are routed to the `errors` vec
/// (fatal path), which causes `parse()` to return `Err` after Phase 6 gate.
///
/// Error accumulation is preserved: all malformed-markup errors in a field are
/// collected before the gate halts the build (not fail-on-first). (F-077-P14-001)
fn inline_markup_route_to_error(
    route: self::template::InlineMarkupRoute,
    file_path: &str,
    line: u32,
    col: u32,
    src: &str,
    byte_start: usize,
    span_len: usize,
) -> SyntaxError {
    use self::template::InlineMarkupRoute;
    match route {
        InlineMarkupRoute::UnclosedInlineMarkup(delimiter, clean_msg) => {
            SyntaxError::unclosed_inline_markup(
                file_path.to_string(),
                line,
                col,
                delimiter.clone(),
                clean_msg,
                src.to_string(),
                byte_start,
                delimiter.len().max(1),
            )
        },
        InlineMarkupRoute::EmptyInlineMarkupSpan(delimiter, clean_msg) => {
            SyntaxError::empty_inline_markup_span(
                file_path.to_string(),
                line,
                col,
                delimiter.clone(),
                clean_msg,
                src.to_string(),
                byte_start,
                delimiter.len().max(1),
            )
        },
        // E-PAR-021: same sink as 019/020; real offset in `clean_msg` (OBS-P8-A).
        InlineMarkupRoute::InlineNestingDepthExceeded(clean_msg) => {
            SyntaxError::inline_nesting_depth_exceeded(
                file_path.to_string(),
                line,
                col,
                clean_msg,
                src.to_string(),
                byte_start,
                span_len,
            )
        },
        // E-PAR-022: disallowed link URL scheme. Fatal like all E-PAR codes.
        // The user-facing message is from SyntaxError::DisallowedLinkUrlScheme's #[error(...)].
        InlineMarkupRoute::DisallowedLinkUrlScheme(scheme) => {
            SyntaxError::disallowed_link_url_scheme(
                file_path.to_string(),
                line,
                col,
                scheme,
                src.to_string(),
                byte_start,
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

/// Strip the `Custom("...")` wrapper that chumsky's `{:?}` debug format adds to
/// `Rich::custom` error reasons.
///
/// When a parser calls `Rich::custom(span, "some message")`, the `reason()` is
/// `RichReason::Custom("some message")`.  Formatting that with `{:?}` produces
/// the string `Custom("some message")` — including the `Custom(` prefix and `")`
/// suffix that must not appear in user-facing diagnostics.
///
/// This function detects that wrapper and returns the inner string (unescaping
/// simple `\"` sequences).  If `raw` does not match the `Custom("...")` pattern,
/// it is returned unchanged — covering `Expected`, `Many`, and other reason
/// variants that produce a different `{:?}` format.
///
/// # F-077-P5-001
///
/// This was the root cause of the sentinel leak: `raw_message` was used directly
/// as the `message:` field of every `SyntaxError`, causing `Custom("...")` (and,
/// for inline-markup errors, `Custom("SLIDEFORGE_INLINE_ROUTE|...")`) to appear
/// verbatim in rendered diagnostics.
fn strip_custom_wrapper(raw: &str) -> String {
    // The pattern is: Custom("...")
    // The inner string may contain escaped characters (e.g. `\"` for a literal
    // double-quote inside the message, `\\` for a backslash).
    // We only need to handle the common E-PAR-NNN messages which never contain
    // literal double-quotes in the inner text, so a simple prefix/suffix strip
    // is sufficient for production use.  For robustness, also unescape `\"`.
    const PREFIX: &str = "Custom(\"";
    const SUFFIX: &str = "\")";
    if raw.starts_with(PREFIX) && raw.ends_with(SUFFIX) {
        let inner = &raw[PREFIX.len()..raw.len() - SUFFIX.len()];
        // Unescape the two sequences that `{:?}` escapes inside a String:
        // `\"` → `"` and `\\` → `\`.
        inner.replace("\\\"", "\"").replace("\\\\", "\\")
    } else {
        raw.to_string()
    }
}

/// Extract the first single-quoted name from `msg` (e.g. `"'chart'"` → `"chart"`).
///
/// Used to pull the variable/keyword name out of structured E-PAR-NNN messages
/// so the typed `SyntaxError` variants can carry the right `name` / `keyword`
/// fields.
fn extract_quoted_name(msg: &str) -> Option<&str> {
    let start = msg.find('\'')? + 1;
    let rest = &msg[start..];
    let end = rest.find('\'')?;
    Some(&rest[..end])
}

/// Extract the section group name from a W-PAR-002 message.
///
/// W-PAR-002 messages are formatted as:
/// `"W-PAR-002: Duplicate section group name '{name}'. ..."`
///
/// This helper extracts `name` from between the first pair of single-quotes.
fn extract_w_par_002_name(msg: &str) -> Option<&str> {
    // Same logic as extract_quoted_name — find the first '...' span.
    extract_quoted_name(msg)
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
        error::SyntaxError,
        span::SourceMap,
        template::TemplateChunk,
    };
    use std::sync::Arc;

    /// Helper: add a file to a fresh [`SourceMap`] and call [`parse`].
    fn parse_str(src: &str) -> Result<ParseResult, Vec<SyntaxError>> {
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
        let deck = result.unwrap().deck;
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
        assert_eq!(
            r1.ok().map(|p| p.deck),
            r2.ok().map(|p| p.deck),
            "same source must produce identical ASTs"
        );
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
        if let Ok(pr) = result {
            assert!(
                pr.deck.items.is_empty(),
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
        let deck = result.unwrap().deck;
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
        let deck = result.unwrap().deck;
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
        let deck = result.unwrap().deck;
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
        let deck = result.expect("must parse").deck;
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

    // ════════════════════════════════════════════════════════════════════════════
    // STORY-009 Failing Tests (Red Gate)
    // All tests below MUST FAIL until the implementer fills in the stubs.
    // ════════════════════════════════════════════════════════════════════════════

    // ── AC-001: slideforge_version "1" → DeckNode.version = Some("1") ─────────

    #[test]
    fn test_bc_1_09_001_version_1_accepted_stored_in_deck_node() {
        // AC-001: `slideforge_version "1"` is already partially parsed by the
        // existing version_decl_parser, but the VERSION GATE logic (reject v2+,
        // warn on missing) is NOT yet implemented. This test verifies that the
        // existing parse stores "1" correctly AND that the version gate does not
        // erroneously reject v1.
        let src = concat!(
            "slideforge_version \"1\"\n",
            "slide title:\n",
            "  title \"T\"\n",
        );
        let result = parse_str(src);
        assert!(result.is_ok(), "version 1 must be accepted: {result:?}");
        let deck = result.unwrap().deck;
        assert_eq!(
            deck.version.as_ref().map(|v| v.value().as_str()),
            Some("1"),
            "DeckNode.version must be Some(\"1\")"
        );
    }

    // ── AC-002: slideforge_version "2" → E-PAR-010 fatal ─────────────────────

    #[test]
    fn test_bc_1_09_010_version_2_rejected_with_e_par_010() {
        // AC-002: `slideforge_version "2"` must produce E-PAR-010 and halt parsing.
        // The version gate logic is a stub — this test FAILS until implemented.
        let src = concat!(
            "slideforge_version \"2\"\n",
            "slide title:\n",
            "  title \"T\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "version 2 must be rejected with E-PAR-010; got Ok"
        );
        let errors = result.unwrap_err();
        let has_version_error = errors
            .iter()
            .any(|e| matches!(e, SyntaxError::VersionError { .. }));
        assert!(
            has_version_error,
            "must have a VersionError (E-PAR-010) for version 2; got: {errors:?}"
        );
    }

    // ── AC-003: Missing version → E-PAR-010 warning (non-fatal) ─────────────
    //
    // Per BC-1.09.010 / BC-1.13.001 postcondition 2, a missing
    // `slideforge_version` declaration is a non-fatal WARNING.  The parser
    // returns Ok (parse succeeds), but ParseResult::warnings contains a
    // VersionError with is_fatal=false.

    #[test]
    fn test_bc_1_09_010_missing_version_parses_ok_with_none() {
        // AC-003: a deck with no `slideforge_version` parses successfully;
        // DeckNode.version is None AND warnings contain E-PAR-010.
        let src = concat!("slide title:\n", "  title \"No version\"\n",);
        let result = parse_str(src);
        assert!(
            result.is_ok(),
            "missing version is a warning per BC-1.13.001 — parse must succeed; got: {result:?}"
        );
        let pr = result.unwrap();
        assert!(
            pr.deck.version.is_none(),
            "version must be None when slideforge_version is absent"
        );
        let has_version_warning = pr.warnings.iter().any(|e| {
            matches!(
                e,
                SyntaxError::VersionError {
                    is_fatal: false,
                    ..
                }
            )
        });
        assert!(
            has_version_warning,
            "missing version must emit E-PAR-010 warning (non-fatal); got warnings: {:?}",
            pr.warnings
        );
    }

    // ── AC-004: slideforge_version "1.2" → accepted (major "1" matches) ──────

    #[test]
    fn test_bc_1_09_001_version_1_dot_2_accepted() {
        // AC-004: "1.2" — the major version is "1", which is compatible.
        // This requires the version gate to parse the major component.
        // FAILS until the version gate is implemented.
        let src = concat!(
            "slideforge_version \"1.2\"\n",
            "slide title:\n",
            "  title \"T\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_ok(),
            "version 1.2 must be accepted (major=1): {result:?}"
        );
    }

    // ── AC-005: version "2" + --warn-only → still fatal ──────────────────────

    #[test]
    fn test_bc_1_09_010_version_2_is_always_fatal() {
        // AC-005: E-PAR-010 for forward-incompatible versions is ALWAYS fatal —
        // it must not be demoted even if --warn-only is in effect.
        // Verified via SyntaxError::is_always_fatal().
        let err = SyntaxError::version_error(
            "test.sf".to_string(),
            "forward-incompatible version 2".to_string(),
            true, // is_fatal=true for v2+
            "slideforge_version \"2\"\n".to_string(),
            0,
        );
        assert!(
            err.is_always_fatal(),
            "VersionError with is_fatal=true must report is_always_fatal()=true"
        );
    }

    // ── AC-006: $x^2$ → MathInline("x^2") ───────────────────────────────────

    #[test]
    fn test_bc_1_09_006_math_inline_dollar_single_produces_math_inline_chunk() {
        // AC-006: `title "$x^2$"` → FieldValue::Template([MathInline("x^2")])
        // FAILS until the math mode parser is implemented.
        let src = concat!("slide title:\n", "  title \"$x^2$\"\n",);
        let result = parse_str(src);
        assert!(result.is_ok(), "math inline must parse: {result:?}");
        let deck = result.unwrap().deck;
        let BlockItem::Slide(slide_s) = &deck.items[0] else {
            panic!("expected Slide");
        };
        let title_field = slide_s
            .value()
            .fields
            .iter()
            .find(|f| f.name.value() == "title")
            .expect("title field must exist");
        let FieldValue::Template(chunks) = title_field.value.value() else {
            panic!("title must be Template");
        };
        assert_eq!(
            chunks.len(),
            1,
            "must have exactly 1 chunk (the math inline)"
        );
        assert!(
            matches!(&chunks[0], TemplateChunk::MathInline(s) if s == "x^2"),
            "chunk must be MathInline(\"x^2\"); got: {:?}",
            chunks[0]
        );
    }

    // ── AC-007: $$\sum_{i=0}^{n} i$$ → MathDisplay ───────────────────────────

    #[test]
    fn test_bc_1_09_007_math_display_dollar_double_produces_math_display_chunk() {
        // AC-007: `body "$$\\sum_{i=0}^{n} i$$"` → MathDisplay("\\sum_{i=0}^{n} i")
        // FAILS until math mode parser is implemented.
        let src = concat!("slide content:\n", "  body \"$$\\\\sum_{i=0}^{n} i$$\"\n",);
        let result = parse_str(src);
        assert!(result.is_ok(), "math display must parse: {result:?}");
        let deck = result.unwrap().deck;
        let BlockItem::Slide(slide_s) = &deck.items[0] else {
            panic!("expected Slide");
        };
        let body_field = slide_s
            .value()
            .fields
            .iter()
            .find(|f| f.name.value() == "body")
            .expect("body field must exist");
        let FieldValue::Template(chunks) = body_field.value.value() else {
            panic!("body must be Template");
        };
        let has_math_display = chunks
            .iter()
            .any(|c| matches!(c, TemplateChunk::MathDisplay(_)));
        assert!(
            has_math_display,
            "must have at least one MathDisplay chunk; got: {chunks:?}"
        );
    }

    // ── AC-008: @{var} in math → MathInterp; {{ var }} in math → literal ─────

    #[test]
    fn test_bc_1_09_008_at_brace_in_math_produces_math_interp_chunk() {
        // AC-008: `title "$@{base}^2$"` → [MathInterp(Expr::Ident("base"))]
        // FAILS until math mode interpolation is implemented.
        let src = concat!("slide title:\n", "  title \"$@{base}^2$\"\n",);
        let result = parse_str(src);
        assert!(result.is_ok(), "math with @{{}} must parse: {result:?}");
        let deck = result.unwrap().deck;
        let BlockItem::Slide(slide_s) = &deck.items[0] else {
            panic!("expected Slide");
        };
        let title = slide_s
            .value()
            .fields
            .iter()
            .find(|f| f.name.value() == "title")
            .expect("title must exist");
        let FieldValue::Template(chunks) = title.value.value() else {
            panic!("title must be Template");
        };
        let has_math_interp = chunks
            .iter()
            .any(|c| matches!(c, TemplateChunk::MathInterp(_)));
        assert!(
            has_math_interp,
            "must have at least one MathInterp chunk; got: {chunks:?}"
        );
    }

    #[test]
    fn test_bc_1_09_008_double_brace_in_math_is_literal_not_interp() {
        // AC-008: `{{ var }}` inside `$...$` must be treated as literal LaTeX text,
        // NOT as a text-mode interpolation.
        // FAILS until math mode parser correctly handles {{ inside math.
        let src = concat!("slide title:\n", "  title \"${{ x }}^2$\"\n",);
        let result = parse_str(src);
        assert!(
            result.is_ok(),
            "{{ }} in math must parse (as literal): {result:?}"
        );
        let deck = result.unwrap().deck;
        let BlockItem::Slide(slide_s) = &deck.items[0] else {
            panic!("expected Slide");
        };
        let title = slide_s
            .value()
            .fields
            .iter()
            .find(|f| f.name.value() == "title")
            .expect("title must exist");
        let FieldValue::Template(chunks) = title.value.value() else {
            panic!("title must be Template");
        };
        // Must NOT have a non-math Expr chunk (which would indicate {{ }} was
        // misinterpreted as text-mode interpolation inside math).
        let has_text_mode_expr = chunks.iter().any(|c| matches!(c, TemplateChunk::Expr(_)));
        assert!(
            !has_text_mode_expr,
            "{{ }} inside math must NOT produce a text-mode Expr chunk; got: {chunks:?}"
        );
        // The {{ x }} must appear as part of a MathInline literal or similar.
        let has_math_inline = chunks
            .iter()
            .any(|c| matches!(c, TemplateChunk::MathInline(_)));
        assert!(
            has_math_inline,
            "must have at least one MathInline chunk for ${{ x }}^2$; got: {chunks:?}"
        );
    }

    // ── AC-010: raw pptx: → E-PAR-009 ────────────────────────────────────────

    #[test]
    fn test_bc_1_09_009_raw_pptx_rejected_with_e_par_009() {
        // AC-010: `raw pptx:` as a field name must produce E-PAR-009 (RawKeyword).
        // FAILS until the raw-keyword check is implemented.
        let src = concat!("slide content:\n", "  raw pptx: \"<a:sp/>\"\n",);
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "raw pptx: must be rejected with E-PAR-009; got Ok"
        );
        let errors = result.unwrap_err();
        let has_raw_error = errors
            .iter()
            .any(|e| matches!(e, SyntaxError::RawKeyword { .. }));
        assert!(
            has_raw_error,
            "must have RawKeyword (E-PAR-009) error; got: {errors:?}"
        );
    }

    #[test]
    fn test_bc_1_09_009_bare_raw_field_rejected_with_e_par_009() {
        // AC-010 (edge case): `raw` alone as a field name must also be rejected.
        let src = concat!("slide content:\n", "  raw \"value\"\n",);
        let result = parse_str(src);
        // Either E-PAR-009 or E-PAR-002 is acceptable (raw may be caught at
        // different phases), but no successful parse is allowed.
        assert!(
            result.is_err(),
            "bare 'raw' as field name must be rejected; got Ok"
        );
    }

    // ── AC-011: vars: { raw: "data" } → E-PAR-008 (not E-PAR-009) ────────────

    #[test]
    fn test_bc_1_09_008_raw_in_vars_is_e_par_008_not_e_par_009() {
        // AC-011: `vars: { raw: "data" }` uses `raw` as a variable name.
        // Per AC-011, this must produce E-PAR-008 (VarNameCollision), NOT
        // E-PAR-009 (RawKeyword).
        // FAILS until vars-name collision check is implemented.
        let src = concat!(
            "vars:\n",
            "  raw: \"data\"\n",
            "slide title:\n",
            "  title \"T\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "vars: {{ raw: ... }} must be rejected; got Ok"
        );
        let errors = result.unwrap_err();
        // Must NOT produce E-PAR-009 (that's for raw pptx: usage).
        let has_raw_keyword_err = errors
            .iter()
            .any(|e| matches!(e, SyntaxError::RawKeyword { .. }));
        assert!(
            !has_raw_keyword_err,
            "vars: {{ raw: ... }} must NOT produce E-PAR-009 (that's for field position); got: {errors:?}"
        );
        // Must produce E-PAR-008.
        let has_var_collision = errors
            .iter()
            .any(|e| matches!(e, SyntaxError::VarNameCollision { .. }));
        assert!(
            has_var_collision,
            "vars: {{ raw: ... }} must produce E-PAR-008 (VarNameCollision); got: {errors:?}"
        );
    }

    // ── AC-012: @fn compute(x): → E-PAR-006 ──────────────────────────────────

    #[test]
    fn test_bc_1_09_006_at_fn_directive_rejected_with_e_par_006() {
        // AC-012: `@fn compute(x):` must produce E-PAR-006 (ReservedKeyword).
        // FAILS until reserved-keyword check is implemented.
        let src = concat!(
            "slideforge_version \"1\"\n",
            "@fn compute(x):\n",
            "  x * 2\n",
            "slide title:\n",
            "  title \"T\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "@fn must be rejected with E-PAR-006; got Ok"
        );
        let errors = result.unwrap_err();
        let has_reserved = errors
            .iter()
            .any(|e| matches!(e, SyntaxError::ReservedKeyword { .. }));
        assert!(
            has_reserved,
            "must have ReservedKeyword (E-PAR-006) for @fn; got: {errors:?}"
        );
    }

    // ── AC-013: vars: { chart: "data" } → E-PAR-008 ──────────────────────────

    #[test]
    fn test_bc_1_09_008_vars_chart_collision_emits_e_par_008() {
        // AC-013: `chart` is a slide type keyword — using it as a var name must
        // produce E-PAR-008.
        // FAILS until vars name collision check is implemented.
        let src = concat!(
            "vars:\n",
            "  chart: \"data\"\n",
            "slide title:\n",
            "  title \"T\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "vars: {{ chart: ... }} must be rejected; got Ok"
        );
        let errors = result.unwrap_err();
        let has_collision = errors
            .iter()
            .any(|e| matches!(e, SyntaxError::VarNameCollision { name, .. } if name == "chart"));
        assert!(
            has_collision,
            "must have VarNameCollision for 'chart'; got: {errors:?}"
        );
    }

    // ── AC-014: vars: { chart_data: "x" } → no error (suffix, not exact) ─────

    #[test]
    fn test_bc_1_09_008_vars_chart_data_suffix_is_not_a_collision() {
        // AC-014: `chart_data` is NOT a slide type keyword — it must be accepted.
        // This test verifies the suffix rule: only EXACT matches collide.
        let src = concat!(
            "vars:\n",
            "  chart_data: \"x\"\n",
            "slide title:\n",
            "  title \"T\"\n",
        );
        let result = parse_str(src);
        // Since the vars-collision check is not yet implemented, this may or may
        // not parse successfully — but it must NOT emit VarNameCollision for
        // chart_data.
        if let Err(errors) = &result {
            let has_false_positive = errors.iter().any(
                |e| matches!(e, SyntaxError::VarNameCollision { name, .. } if name == "chart_data"),
            );
            assert!(
                !has_false_positive,
                "chart_data must NOT trigger VarNameCollision; got: {errors:?}"
            );
        }
        // If it does parse OK: verify no collision error was accumulated.
        // (This arm is a no-op for now since the assertion above guards the Err path.)
    }

    // ── EC-001: empty math delimiter $$ → parse error or empty content ────────

    #[test]
    fn test_bc_1_09_006_empty_inline_math_is_error_or_empty() {
        // EC-001: `title "$$"` — empty inline math delimiter.
        // Must either produce an error OR produce an empty MathInline("") chunk,
        // but must NOT silently parse as a plain string.
        let src = concat!("slide title:\n", "  title \"$$\"\n",);
        let result = parse_str(src);
        if let Ok(pr) = result {
            // If it parsed OK, the chunk must be an empty MathInline, not a Literal.
            let deck = pr.deck;
            let BlockItem::Slide(slide_s) = &deck.items[0] else {
                panic!("expected Slide");
            };
            let title = slide_s
                .value()
                .fields
                .iter()
                .find(|f| f.name.value() == "title")
                .expect("title must exist");
            let FieldValue::Template(chunks) = title.value.value() else {
                panic!("title must be Template");
            };
            // Must not be a plain Literal containing "$$"
            let is_plain_literal =
                chunks.len() == 1 && matches!(&chunks[0], TemplateChunk::Literal(s) if s == "$$");
            assert!(
                !is_plain_literal,
                "empty $$ must NOT be treated as a plain string literal; got: {chunks:?}"
            );
        }
        // Err is also acceptable — the key is it's not silently misclassified.
    }

    // ── EC-002: nested math delimiters $a $b$ c$ → error ─────────────────────

    #[test]
    fn test_bc_1_09_006_nested_math_delimiters_are_errors() {
        // EC-002: `$a $b$ c$` — nested/unclosed math delimiters should produce
        // a parse error (the grammar does not allow nesting).
        let src = concat!("slide title:\n", "  title \"$a $b$ c$\"\n",);
        let result = parse_str(src);
        // May be an error or a degraded parse — must not silently succeed with
        // the outer dollar signs treated as normal text-mode literal.
        if let Ok(pr) = result {
            let deck = pr.deck;
            let BlockItem::Slide(slide_s) = &deck.items[0] else {
                panic!("expected Slide");
            };
            let title = slide_s
                .value()
                .fields
                .iter()
                .find(|f| f.name.value() == "title")
                .expect("title must exist");
            let FieldValue::Template(chunks) = title.value.value() else {
                panic!("title must be Template");
            };
            // If we reach here, there must be at least one MathInline chunk.
            let has_math = chunks
                .iter()
                .any(|c| matches!(c, TemplateChunk::MathInline(_)));
            assert!(
                has_math,
                "nested math source must produce at least one MathInline chunk"
            );
        }
    }

    // ── EC-003: slideforge_version with only whitespace → E-PAR-010 ──────────

    #[test]
    fn test_bc_1_09_010_version_whitespace_only_is_error() {
        // EC-003: `slideforge_version "  "` (whitespace-only) must be rejected.
        // Either E-PAR-010 or E-PAR-002 is acceptable; Ok is NOT.
        let src = concat!(
            "slideforge_version \"  \"\n",
            "slide title:\n",
            "  title \"T\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "whitespace-only version must be rejected; got Ok"
        );
    }

    // ── EC-004: slideforge_version "0" → E-PAR-010 ───────────────────────────

    #[test]
    fn test_bc_1_09_010_version_0_rejected() {
        // EC-004: version "0" is below the minimum supported major version.
        // Must produce E-PAR-010.
        let src = concat!(
            "slideforge_version \"0\"\n",
            "slide title:\n",
            "  title \"T\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "version 0 must be rejected with E-PAR-010; got Ok"
        );
        let errors = result.unwrap_err();
        let has_version_error = errors
            .iter()
            .any(|e| matches!(e, SyntaxError::VersionError { .. }));
        assert!(
            has_version_error,
            "version 0 must produce VersionError; got: {errors:?}"
        );
    }

    // ── EC-005: slideforge_version "abc" → E-PAR-010 ─────────────────────────

    #[test]
    fn test_bc_1_09_010_version_non_numeric_rejected() {
        // EC-005: `slideforge_version "abc"` — non-numeric version must be rejected.
        let src = concat!(
            "slideforge_version \"abc\"\n",
            "slide title:\n",
            "  title \"T\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "non-numeric version must be rejected; got Ok"
        );
    }

    // ── EC-006: @mixin → E-PAR-006 ───────────────────────────────────────────

    #[test]
    fn test_bc_1_09_006_at_mixin_directive_rejected_with_e_par_006() {
        // EC-006: `@mixin my_style:` at deck level must produce E-PAR-006.
        let src = concat!(
            "slideforge_version \"1\"\n",
            "@mixin my_style:\n",
            "  color: \"blue\"\n",
            "slide title:\n",
            "  title \"T\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "@mixin must be rejected with E-PAR-006; got Ok"
        );
        let errors = result.unwrap_err();
        let has_reserved = errors
            .iter()
            .any(|e| matches!(e, SyntaxError::ReservedKeyword { .. }));
        assert!(
            has_reserved,
            "must have ReservedKeyword (E-PAR-006) for @mixin; got: {errors:?}"
        );
    }

    // ── EC-007: vars: { title: "val" } → E-PAR-008 ───────────────────────────

    #[test]
    fn test_bc_1_09_008_vars_title_collision_emits_e_par_008() {
        // EC-007: `title` is a slide type keyword — using it as a vars name must
        // produce E-PAR-008.
        let src = concat!(
            "vars:\n",
            "  title: \"My Document\"\n",
            "slide title:\n",
            "  title \"{{ title }}\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "vars: {{ title: ... }} must be rejected; got Ok"
        );
        let errors = result.unwrap_err();
        let has_collision = errors
            .iter()
            .any(|e| matches!(e, SyntaxError::VarNameCollision { name, .. } if name == "title"));
        assert!(
            has_collision,
            "must have VarNameCollision for 'title'; got: {errors:?}"
        );
    }

    // ── Snapshot: math inline chunk ───────────────────────────────────────────

    #[test]
    fn test_bc_1_09_006_math_inline_chunk_snapshot() {
        // Snapshot test: MathInline("x^2") must render consistently.
        // FAILS until math mode parser is implemented.
        let chunk = TemplateChunk::MathInline("x^2".to_string());
        // Use insta for snapshot pinning (will auto-create snapshot on first run
        // once the implementation is in place).
        insta::assert_debug_snapshot!("math_inline_chunk_x_squared", chunk);
    }

    #[test]
    fn test_bc_1_09_007_math_display_chunk_snapshot() {
        // Snapshot test: MathDisplay content must render consistently.
        let chunk = TemplateChunk::MathDisplay(r"\sum_{i=0}^{n} i".to_string());
        insta::assert_debug_snapshot!("math_display_chunk_sum", chunk);
    }

    #[test]
    fn test_bc_1_09_008_math_interp_chunk_snapshot() {
        use crate::expr::Expr;
        // Snapshot test: MathInterp(Expr::Ident("base")) must render consistently.
        let chunk = TemplateChunk::MathInterp(Expr::Ident("base".to_string()));
        insta::assert_debug_snapshot!("math_interp_chunk_base", chunk);
    }

    // ── EC-002: slideforge_version after slide block → E-PAR-010 fatal ────────

    #[test]
    fn test_bc_1_09_010_version_after_slide_is_fatal() {
        // EC-002: `slideforge_version "1"` appearing AFTER a slide block must be
        // a fatal error (E-PAR-010). The version declaration must appear in deck
        // metadata at the top of the file — placing it after slides is invalid.
        let src = concat!(
            "slide title:\n",
            "  title \"Hello\"\n",
            "slideforge_version \"1\"\n",
        );
        let result = parse_str(src);
        assert!(result.is_err(), "version after slide must be fatal");
        let errors = result.unwrap_err();
        let has_version_error = errors
            .iter()
            .any(|e| matches!(e, SyntaxError::VersionError { .. }));
        assert!(
            has_version_error,
            "must produce E-PAR-010 for misplaced version; got: {errors:?}"
        );
    }

    // ── EC-006: comment containing the word 'raw' → no error ─────────────────

    #[test]
    fn test_bc_1_09_009_comment_with_raw_word_no_error() {
        // EC-006: A comment containing the word `raw` must NOT produce E-PAR-009.
        // The `raw` keyword is only rejected at field-name position; inside a
        // `#`-comment it must be ignored entirely.
        let src = concat!(
            "slideforge_version \"1\"\n",
            "# raw OOXML here\n",
            "slide title:\n",
            "  title \"Test\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_ok(),
            "comment containing 'raw' must NOT produce any error; got: {result:?}"
        );
    }

    // ── EC-007: two var-name collisions accumulated ───────────────────────────

    #[test]
    fn test_bc_1_09_008_two_var_collisions_accumulated() {
        // EC-007: two vars entries with reserved slide-type names must produce
        // ≥ 2 VarNameCollision errors (accumulation, not fail-fast).
        let src = concat!(
            "slideforge_version \"1\"\n",
            "vars:\n",
            "  title: \"My Deck\"\n",
            "  chart: \"data\"\n",
            "slide title:\n",
            "  title \"Test\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "two var collisions must produce Err; got Ok"
        );
        let errors = result.unwrap_err();
        let collision_count = errors
            .iter()
            .filter(|e| matches!(e, SyntaxError::VarNameCollision { .. }))
            .count();
        assert!(
            collision_count >= 2,
            "must accumulate >= 2 VarNameCollision errors; got {collision_count}: {errors:?}"
        );
    }

    // ── EC-008: raw_pptx as field name → E-PAR-009 ───────────────────────────

    #[test]
    fn test_bc_1_09_009_raw_pptx_variant_rejected_with_e_par_009() {
        // EC-008: `raw_pptx "value"` as a slide field name must be rejected.
        // The `raw_pptx` identifier is in RESERVED_KEYWORDS with E-PAR-009.
        // The field-line parser's raw_rejected arm catches `raw` prefix tokens;
        // `raw_pptx` is a single identifier and must be caught by keyword check.
        let src = concat!(
            "slideforge_version \"1\"\n",
            "slide content:\n",
            "  raw_pptx \"<a:sp/>\"\n",
        );
        let result = parse_str(src);
        // raw_pptx is a reserved identifier — must NOT parse as a plain field.
        // The parser may produce E-PAR-009 or E-PAR-002 depending on how the
        // field-line combinator handles it; Ok is NOT acceptable.
        assert!(
            result.is_err(),
            "raw_pptx as field name must be rejected; got Ok"
        );
    }
}
