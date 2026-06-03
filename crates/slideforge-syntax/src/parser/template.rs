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
//! emits E-PAR-019. An empty inline markup span emits E-PAR-020.
//!
//! Note: E-PAR-004 is owned by slideforge-eval (`IncludeCycle`). These
//! template-parsing codes (E-PAR-012, 013, 014, 019, 020) are distinct.
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

/// Produce an E-PAR-019 error message for an unclosed inline markup span.
fn unclosed_inline_msg(delimiter: &str) -> String {
    format!(
        "E-PAR-019: unclosed inline markup delimiter `{delimiter}` — \
         add a closing `{delimiter}` after the markup text, \
         e.g., `{delimiter}text{delimiter}`"
    )
}

/// Produce an E-PAR-020 error message for an empty inline markup span.
fn empty_inline_msg(delimiter: &str) -> String {
    format!(
        "E-PAR-020: empty inline markup span `{delimiter}{delimiter}` — \
         spans must contain at least one character"
    )
}

/// Produce an E-PAR-021 error message for inline-markup nesting depth exceeded.
///
/// `open_offset` is the absolute byte offset (within the field-value string)
/// of the opening delimiter that pushed recursion over the cap. Threaded from
/// the parent `scan_template_chunks` call via `call_site_offset` (OBS-P8-A fix).
fn nesting_depth_exceeded_msg(open_offset: usize, depth: usize) -> String {
    format!(
        "E-PAR-021: Inline-markup nesting depth exceeded at byte offset {open_offset}: \
         depth {depth} exceeds maximum of {MAX_INLINE_NESTING}. \
         Flatten or reduce nested inline markup."
    )
}

/// Maximum recursion depth for `scan_template_chunks`.
///
/// Prevents crafted inputs (e.g. thousands of alternating `^_` pairs) from
/// exhausting the stack via unbounded mutual recursion (F-077-P7-002).
/// When this cap is reached an E-PAR-021 diagnostic is accumulated and the
/// remainder of the input is treated as a `Literal` chunk.
const MAX_INLINE_NESTING: usize = 64;

/// The semantic kind of a [`TemplateError`].
///
/// Using a typed enum avoids embedding the diagnostic code as a string prefix in
/// the message and allows the error-conversion boundary in `parser/mod.rs` to
/// produce the correct [`crate::error::SyntaxError`] variant without
/// fragile `message.contains("E-PAR-NNN")` string checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateErrorKind {
    /// `{{` without closing `}}` (E-PAR-012).
    UnterminatedInterpolation,
    /// `{{ }}` with empty expression (E-PAR-013).
    EmptyInterpolation,
    /// `$...$` or `$$...$$` without closing delimiter (E-PAR-014).
    UnterminatedMath,
    /// Opening inline markup delimiter without a closing counterpart (E-PAR-019).
    ///
    /// Carries the delimiter string (e.g. `"**"`, `"_"`).
    UnclosedInlineMarkup(String),
    /// Opening and closing inline markup delimiters with nothing between them (E-PAR-020).
    ///
    /// Carries the delimiter string (e.g. `"**"`, `"_"`).
    EmptyInlineMarkupSpan(String),
    /// Inline markup nesting depth exceeded `MAX_INLINE_NESTING` (E-PAR-021).
    ///
    /// Carries the depth at which the cap was triggered.
    InlineNestingDepthExceeded(usize),
}

/// A parse error produced by `scan_template_chunks`, carrying both the byte
/// offset of the opening delimiter (relative to the field-value string content),
/// a typed [`TemplateErrorKind`] for structured routing, and the human-readable
/// message.
///
/// The byte offset is used by `section_value_parser` (and other callers that
/// need sub-span precision) to translate the offset into a [`SimpleSpan`] that
/// points at the OPENING delimiter rather than the whole string literal token.
/// Callers that need to route to a specific [`crate::error::SyntaxError`] variant
/// should use `.into_routing_message()` and inspect `.kind`.
#[derive(Debug, Clone)]
pub struct TemplateError {
    /// Byte offset of the problematic construct within the field-value string.
    pub byte_offset: usize,
    /// Semantic kind used for structured error routing (no string-matching needed).
    pub kind: TemplateErrorKind,
    /// Human-readable error message (E-PAR-012, 013, 014, 019, 020).
    pub message: String,
}

impl TemplateError {
    /// Construct a new `TemplateError` with the byte offset of the opening
    /// delimiter, the semantic kind, and the human-readable error message.
    fn new(byte_offset: usize, kind: TemplateErrorKind, message: String) -> Self {
        Self {
            byte_offset,
            kind,
            message,
        }
    }

    /// Produce a routing-tagged message that encodes the [`TemplateErrorKind`] and
    /// (for inline markup errors) the delimiter string in a pipe-separated prefix.
    ///
    /// Format: `SLIDEFORGE_INLINE_ROUTE|<KIND>|<DELIM_HEX>|<ORIGINAL_MESSAGE>`
    ///
    /// - `SLIDEFORGE_INLINE_ROUTE|` is the sentinel (unique prefix not present in
    ///   any normal E-PAR-NNN message text).
    /// - `<KIND>` is `UnclosedInlineMarkup`, `EmptyInlineMarkupSpan`, or
    ///   `InlineNestingDepthExceeded`.
    /// - `<DELIM_HEX>` is the delimiter bytes hex-encoded so `|` cannot appear in
    ///   the delimiter field (e.g., `` ` `` → `60`, `**` → `2a2a`, `_` → `5f`).
    ///   For `InlineNestingDepthExceeded` this field is empty (no delimiter applies).
    /// - `<ORIGINAL_MESSAGE>` is the full human-readable E-PAR-019 / E-PAR-020 /
    ///   E-PAR-021 message, preserved for the `message` field of the produced
    ///   `SyntaxError`.
    ///
    /// Hex encoding avoids all possible delimiter-vs-separator conflicts regardless
    /// of which ASCII punctuation characters are used as inline markup delimiters.
    ///
    /// The routing boundary in `parser/mod.rs` calls `parse_routing_tag()` which
    /// extracts `KIND` and `DELIM_HEX`, hex-decodes the delimiter, and produces
    /// the correct [`crate::error::SyntaxError`] variant — no `message.contains()`
    /// or `extract_backtick_name` re-parsing needed.  This fixes F-077-P4-002.
    ///
    /// Non-inline-markup errors (E-PAR-012, 013, 014) do not need the prefix
    /// because their routing in mod.rs is already correct; this method returns
    /// the plain message for those kinds.
    #[must_use]
    pub fn into_routing_message(self) -> String {
        match &self.kind {
            TemplateErrorKind::UnclosedInlineMarkup(delim) => {
                let hex = hex_encode(delim.as_bytes());
                format!(
                    "SLIDEFORGE_INLINE_ROUTE|UnclosedInlineMarkup|{hex}|{msg}",
                    msg = self.message
                )
            },
            TemplateErrorKind::EmptyInlineMarkupSpan(delim) => {
                let hex = hex_encode(delim.as_bytes());
                format!(
                    "SLIDEFORGE_INLINE_ROUTE|EmptyInlineMarkupSpan|{hex}|{msg}",
                    msg = self.message
                )
            },
            // E-PAR-021: nesting depth exceeded — route through the sentinel so
            // parser/mod.rs produces the dedicated InlineNestingDepthExceeded variant
            // with code "E-PAR-021", not the UnexpectedToken catch-all (E-PAR-002).
            // The "delimiter" slot is empty (no delimiter applies here); the
            // original message carries all the diagnostic text the variant needs.
            TemplateErrorKind::InlineNestingDepthExceeded(_) => {
                format!(
                    "SLIDEFORGE_INLINE_ROUTE|InlineNestingDepthExceeded||{msg}",
                    msg = self.message
                )
            },
            // E-PAR-012/013/014: no routing-tag prefix needed. These errors flow
            // through the generic UnexpectedToken arm in parser/mod.rs, which
            // renders the message text directly — no message.contains() dispatch.
            TemplateErrorKind::UnterminatedInterpolation
            | TemplateErrorKind::EmptyInterpolation
            | TemplateErrorKind::UnterminatedMath => self.message,
        }
    }
}

// ─── Hex encoding helpers ─────────────────────────────────────────────────────

/// Encode `bytes` as a lowercase hex string.
///
/// Used by [`TemplateError::into_routing_message`] to encode the delimiter
/// safely — the hex representation can only contain `[0-9a-f]` so it never
/// conflicts with the `|` separator used in the routing tag.
fn hex_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
            use std::fmt::Write as _;
            let _ = write!(acc, "{b:02x}");
            acc
        })
}

/// Decode a hex string produced by [`hex_encode`] back to a `String`.
///
/// Returns `None` if `hex` is not valid lowercase hex or if the decoded bytes
/// are not valid UTF-8.
fn hex_decode(hex: &str) -> Option<String> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    let bytes: Option<Vec<u8>> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect();
    String::from_utf8(bytes?).ok()
}

// ─── Routing tag parser ───────────────────────────────────────────────────────

/// The decoded payload of an inline-markup routing tag embedded in a chumsky
/// `Rich::custom` message by [`TemplateError::into_routing_message`].
///
/// Callers in `parser/mod.rs` extract this from the message string to route
/// E-PAR-019 / E-PAR-020 / E-PAR-021 diagnostics to the correct
/// [`crate::error::SyntaxError`] variant without fragile `message.contains("E-PAR-NNN")`
/// checks.
///
/// E-PAR-019/020 variants carry `(delimiter, clean_message)`.
/// E-PAR-021 carries only `(clean_message)` — no delimiter applies.
///
/// The caller in `parser/mod.rs` uses `clean_message` for the `message:` field of
/// the `SyntaxError` — NOT the raw tagged blob — so the routing sentinel never
/// appears in user-facing output.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum InlineMarkupRoute {
    /// E-PAR-019 — unclosed inline markup delimiter.
    ///
    /// Fields: `(delimiter, clean_message)`.
    UnclosedInlineMarkup(String, String),
    /// E-PAR-020 — empty inline markup span.
    ///
    /// Fields: `(delimiter, clean_message)`.
    EmptyInlineMarkupSpan(String, String),
    /// E-PAR-021 — inline-markup nesting depth exceeded.
    ///
    /// Field: `clean_message` (depth info is embedded in the message text).
    InlineNestingDepthExceeded(String),
}

/// Try to parse an inline-markup routing tag from `msg`.
///
/// Returns `Some(InlineMarkupRoute)` when the message contains the
/// `SLIDEFORGE_INLINE_ROUTE|` sentinel produced by
/// [`TemplateError::into_routing_message`], otherwise `None`.
///
/// The `msg` parameter is the `format!("{:?}", rich_err.reason())` output from
/// chumsky, which wraps the raw message string in `Custom("...")` debug format.
/// Because the routing tag uses only printable ASCII (`[A-Z_|0-9a-f]`), the
/// `{:?}` escaping does not alter it — the sentinel and hex payload survive
/// verbatim.
///
/// Expected format: `...SLIDEFORGE_INLINE_ROUTE|<KIND>|<DELIM_HEX>|<MESSAGE>...`
#[must_use]
pub(super) fn parse_routing_tag(msg: &str) -> Option<InlineMarkupRoute> {
    const SENTINEL: &str = "SLIDEFORGE_INLINE_ROUTE|";
    let tag_start = msg.find(SENTINEL)?;
    let rest = &msg[tag_start + SENTINEL.len()..];
    // Expected format after sentinel: "<KIND>|<DELIM_HEX>|<ORIGINAL_MESSAGE>"
    let (kind_str, rest2) = rest.split_once('|')?;
    let (delim_hex, original_msg) = rest2.split_once('|')?;
    let delim = hex_decode(delim_hex)?;
    // `original_msg` is the clean human-readable E-PAR-019 / E-PAR-020 text.
    // It is passed into the SyntaxError `message:` field so users never see the
    // sentinel or hex payload.  Previously bound to `_original_msg` and discarded,
    // which was the root cause of F-077-P5-001.
    let clean_msg = original_msg.to_string();
    match kind_str {
        "UnclosedInlineMarkup" => Some(InlineMarkupRoute::UnclosedInlineMarkup(delim, clean_msg)),
        "EmptyInlineMarkupSpan" => Some(InlineMarkupRoute::EmptyInlineMarkupSpan(delim, clean_msg)),
        // E-PAR-021: the "delimiter" field is empty (encoded as empty hex ""); we
        // just need the clean message. `delim` will be an empty string here because
        // into_routing_message emits `InlineNestingDepthExceeded||<message>` with
        // an empty hex payload — hex_decode("") returns Some("") (empty length is a
        // multiple of 2), so `delim` is the empty string "".
        // The empty-string value is safe: InlineNestingDepthExceeded has no delimiter.
        "InlineNestingDepthExceeded" => {
            Some(InlineMarkupRoute::InlineNestingDepthExceeded(clean_msg))
        },
        _ => None,
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
/// The `call_site_offset` parameter is the absolute byte offset (within the
/// field-value string) of the opening delimiter that triggered THIS recursive
/// call. The top-level call passes `0` (no opener). Recursive calls pass the
/// `open_pos` of the delimiter that initiated the span — this offset is used
/// in the E-PAR-021 depth-exceeded diagnostic to point at the actual over-cap
/// construct rather than a misleading byte offset 0 (OBS-P8-A fix).
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
    depth: usize,
    call_site_offset: usize,
) -> (Vec<TemplateChunk>, usize) {
    // F-077-P7-002 / OBS-P8-A: cap recursion depth to prevent stack overflow
    // from crafted inputs with deeply-nested alternating delimiters (e.g. `^_^_^_…`).
    // `call_site_offset` is the absolute byte offset of the opening delimiter that
    // triggered THIS call — it points at the actual over-cap construct rather than
    // the misleading hardcoded 0 from the previous implementation.
    if depth >= MAX_INLINE_NESTING {
        errors.push(TemplateError::new(
            call_site_offset,
            TemplateErrorKind::InlineNestingDepthExceeded(depth),
            nesting_depth_exceeded_msg(call_site_offset, depth),
        ));
        // Treat the entire remainder as a literal (in addition to the diagnostic).
        let remainder = s.to_string();
        let consumed = s.len();
        let mut fallback = Vec::new();
        if !remainder.is_empty() {
            fallback.push(TemplateChunk::Literal(remainder));
        }
        return (fallback, consumed);
    }

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
        // Guard with `is_char_boundary` before slicing: `pos` may be inside a
        // multi-byte UTF-8 char (after a byte-level advance through literal text).
        // All closing delimiters are ASCII so `starts_with` is pure byte comparison.
        if let Some(close) = close_on
            && s.is_char_boundary(pos)
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
                errors.push(TemplateError::new(
                    pos,
                    TemplateErrorKind::UnterminatedMath,
                    unterminated_math_msg(true),
                ));
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
                errors.push(TemplateError::new(
                    pos,
                    TemplateErrorKind::UnterminatedMath,
                    unterminated_math_msg(false),
                ));
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
                    errors.push(TemplateError::new(
                        pos,
                        TemplateErrorKind::UnterminatedInterpolation,
                        unterminated_interpolation_msg(),
                    ));
                    pos = len;
                },
                Some(close_pos) => {
                    let inner = &s[after_open..close_pos];
                    if inner.trim().is_empty() {
                        chunks.push(TemplateChunk::Expr(Expr::Error));
                        errors.push(TemplateError::new(
                            pos,
                            TemplateErrorKind::EmptyInterpolation,
                            empty_interpolation_msg(),
                        ));
                    } else if let Ok(expr_val) = parse_inner_expr(inner) {
                        chunks.push(TemplateChunk::Expr(expr_val));
                    } else {
                        chunks.push(TemplateChunk::Expr(Expr::Error));
                        errors.push(TemplateError::new(
                            pos,
                            TemplateErrorKind::UnterminatedInterpolation,
                            unterminated_interpolation_msg(),
                        ));
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
                    errors.push(TemplateError::new(
                        open_pos,
                        TemplateErrorKind::EmptyInlineMarkupSpan("~~".to_string()),
                        empty_inline_msg("~~"),
                    ));
                } else {
                    let (children, _) =
                        scan_template_chunks(inner, None, errors, depth + 1, open_pos);
                    chunks.push(TemplateChunk::Strikethrough(children));
                }
                pos = inner_start + close_rel + 2;
            } else {
                // Unclosed `~~` — error recovery.
                errors.push(TemplateError::new(
                    open_pos,
                    TemplateErrorKind::UnclosedInlineMarkup("~~".to_string()),
                    unclosed_inline_msg("~~"),
                ));
                if !rest.is_empty() {
                    let (children, _) =
                        scan_template_chunks(rest, None, errors, depth + 1, open_pos);
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
                    errors.push(TemplateError::new(
                        open_pos,
                        TemplateErrorKind::EmptyInlineMarkupSpan("~".to_string()),
                        empty_inline_msg("~"),
                    ));
                } else {
                    let (children, _) =
                        scan_template_chunks(inner, None, errors, depth + 1, open_pos);
                    chunks.push(TemplateChunk::Subscript(children));
                }
                pos = inner_start + close_rel + 1;
            } else {
                errors.push(TemplateError::new(
                    open_pos,
                    TemplateErrorKind::UnclosedInlineMarkup("~".to_string()),
                    unclosed_inline_msg("~"),
                ));
                if !rest.is_empty() {
                    let (children, _) =
                        scan_template_chunks(rest, None, errors, depth + 1, open_pos);
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
                let (children, consumed) =
                    scan_template_chunks(rest, Some("**"), errors, depth + 1, open_pos);
                if children.is_empty() {
                    errors.push(TemplateError::new(
                        open_pos,
                        TemplateErrorKind::EmptyInlineMarkupSpan("**".to_string()),
                        empty_inline_msg("**"),
                    ));
                } else {
                    chunks.push(TemplateChunk::Bold(children));
                }
                pos = inner_start + consumed;
            } else {
                // Unclosed `**` — error recovery: treat everything as Bold child.
                // `open_pos` points to the opening `**` (DIR-077-002 §5 span requirement).
                errors.push(TemplateError::new(
                    open_pos,
                    TemplateErrorKind::UnclosedInlineMarkup("**".to_string()),
                    unclosed_inline_msg("**"),
                ));
                if !rest.is_empty() {
                    let (children, _) =
                        scan_template_chunks(rest, None, errors, depth + 1, open_pos);
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
                let (children, consumed) =
                    scan_template_chunks(rest, Some("_"), errors, depth + 1, open_pos);
                if children.is_empty() {
                    errors.push(TemplateError::new(
                        open_pos,
                        TemplateErrorKind::EmptyInlineMarkupSpan("_".to_string()),
                        empty_inline_msg("_"),
                    ));
                } else {
                    chunks.push(TemplateChunk::Italic(children));
                }
                pos = inner_start + consumed;
            } else {
                errors.push(TemplateError::new(
                    open_pos,
                    TemplateErrorKind::UnclosedInlineMarkup("_".to_string()),
                    unclosed_inline_msg("_"),
                ));
                if !rest.is_empty() {
                    let (children, _) =
                        scan_template_chunks(rest, None, errors, depth + 1, open_pos);
                    chunks.push(TemplateChunk::Italic(children));
                }
                pos = len;
            }
            lit_start = pos;
            continue;
        }

        // ── `` ` `` — Code span (verbatim — no inner markup or `{{ }}`) ───────
        // Backtick spans are verbatim — no recursive scan of the content, so
        // `call_site_offset` does not propagate here.
        if bytes.get(pos) == Some(&b'`') {
            flush_lit!();
            let open_pos = pos; // byte offset of opening backtick
            let inner_start = pos + 1;
            if let Some(close_rel) = s[inner_start..].find('`') {
                let inner = &s[inner_start..inner_start + close_rel];
                if inner.is_empty() {
                    errors.push(TemplateError::new(
                        open_pos,
                        TemplateErrorKind::EmptyInlineMarkupSpan("`".to_string()),
                        empty_inline_msg("`"),
                    ));
                } else {
                    // Verbatim: no further processing of the content.
                    chunks.push(TemplateChunk::Code(inner.to_string()));
                }
                pos = inner_start + close_rel + 1;
            } else {
                errors.push(TemplateError::new(
                    open_pos,
                    TemplateErrorKind::UnclosedInlineMarkup("`".to_string()),
                    unclosed_inline_msg("`"),
                ));
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
                let link_open_pos = pos; // byte offset of `[`
                if let Some(paren_close_rel) = s[url_start..].find(')') {
                    let paren_close = url_start + paren_close_rel;
                    let text_inner = &s[pos + 1..bracket_close];
                    let url = &s[url_start..paren_close];
                    flush_lit!();
                    // The link text IS processed for nested markup + interpolation.
                    let (text_children, _) =
                        scan_template_chunks(text_inner, None, errors, depth + 1, link_open_pos);
                    chunks.push(TemplateChunk::Link {
                        text: text_children,
                        url: url.to_string(),
                    });
                    pos = paren_close + 1;
                    lit_start = pos;
                    continue;
                }
                // Clear link intent (`[…](` found) but no closing `)` —
                // DIR-077-002 §5: same class as unclosed bold, same recovery.
                // Push E-PAR-019 at the `[` position and continue accumulating
                // the remainder as literal text (error accumulation, not fail-on-first).
                flush_lit!();
                errors.push(TemplateError::new(
                    link_open_pos,
                    TemplateErrorKind::UnclosedInlineMarkup("[".to_string()),
                    unclosed_inline_msg("["),
                ));
                // Recovery: emit the remainder as literal (matching bold recovery style).
                let remainder = &s[link_open_pos..];
                if !remainder.is_empty() {
                    chunks.push(TemplateChunk::Literal(remainder.to_string()));
                }
                pos = len;
                lit_start = pos;
                continue;
            }
            // Not a valid link syntax (no `](`) — fall through as literal.
        }

        // ── `^` — Superscript ────────────────────────────────────────────────
        if bytes.get(pos) == Some(&b'^') {
            flush_lit!();
            let open_pos = pos; // byte offset of opening `^`
            let inner_start = pos + 1;
            let rest = &s[inner_start..];
            if rest.contains('^') {
                let (children, consumed) =
                    scan_template_chunks(rest, Some("^"), errors, depth + 1, open_pos);
                if children.is_empty() {
                    errors.push(TemplateError::new(
                        open_pos,
                        TemplateErrorKind::EmptyInlineMarkupSpan("^".to_string()),
                        empty_inline_msg("^"),
                    ));
                } else {
                    chunks.push(TemplateChunk::Superscript(children));
                }
                pos = inner_start + consumed;
            } else {
                errors.push(TemplateError::new(
                    open_pos,
                    TemplateErrorKind::UnclosedInlineMarkup("^".to_string()),
                    unclosed_inline_msg("^"),
                ));
                if !rest.is_empty() {
                    let (children, _) =
                        scan_template_chunks(rest, None, errors, depth + 1, open_pos);
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
                let (children, consumed) =
                    scan_template_chunks(rest, Some("=="), errors, depth + 1, open_pos);
                if children.is_empty() {
                    errors.push(TemplateError::new(
                        open_pos,
                        TemplateErrorKind::EmptyInlineMarkupSpan("==".to_string()),
                        empty_inline_msg("=="),
                    ));
                } else {
                    chunks.push(TemplateChunk::Highlight(children));
                }
                pos = inner_start + consumed;
            } else {
                errors.push(TemplateError::new(
                    open_pos,
                    TemplateErrorKind::UnclosedInlineMarkup("==".to_string()),
                    unclosed_inline_msg("=="),
                ));
                if !rest.is_empty() {
                    let (children, _) =
                        scan_template_chunks(rest, None, errors, depth + 1, open_pos);
                    chunks.push(TemplateChunk::Highlight(children));
                }
                pos = len;
            }
            lit_start = pos;
            continue;
        }

        // No special form at this position — advance by the full UTF-8 char width
        // (F-077-P7-001). A single-byte advance would leave `pos` at a mid-char
        // byte offset, causing the next `s[pos..]` slice to panic.
        //
        // Use the leading byte to determine char width per RFC 3629:
        //   110xxxxx → 2 bytes, 1110xxxx → 3 bytes, 11110xxx → 4 bytes.
        //   0xxxxxxx (ASCII) and continuation bytes / invalid → 1 byte.
        let advance = match bytes[pos] {
            0xC0..=0xDF => 2,
            0xE0..=0xEF => 3,
            0xF0..=0xF7 => 4,
            _ => 1, // ASCII (0x00-0x7F), continuation bytes, or invalid — advance 1
        };
        pos += advance;
    }

    // Flush any trailing literal.
    // `pos` is always at a char boundary here (all delimiters are ASCII, and
    // the default advance step above always lands on a boundary).
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
/// need to route to a specific [`crate::error::SyntaxError`] variant use
/// [`TemplateError::into_routing_message()`]; callers that need sub-span
/// precision (e.g., `section_value_parser`) use the offset to create a
/// [`SimpleSpan`] pointing at the opening delimiter.
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
        // Top-level call: depth=0, call_site_offset=0 (no enclosing opener).
        let (chunks, _) = scan_template_chunks(&content, None, &mut errors, 0, 0);
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
