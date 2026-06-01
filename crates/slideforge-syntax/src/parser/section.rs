//! Parser combinator for `section <type>:` blocks.
//!
//! Implements `section_block_parser` — the combinator that parses a single
//! `section <type>: INDENT sub_block* DEDENT` production into a [`SectionNode`].
//!
//! # Grammar (DIR-077-001 §3)
//!
//! ```text
//! section_block     ::= "section" IDENT ":" NEWLINE INDENT section_body DEDENT
//! section_body      ::= section_sub_block*
//! section_sub_block ::= IDENT value NEWLINE
//! value             ::= STRING | INT | FLOAT | BOOL | IDENT
//! ```
//!
//! # Key behaviours
//!
//! - Section TYPE is stored verbatim in `SectionNode.kind` — no registry check
//!   at parse time (DIR-077-001-A Ruling 3).
//! - Recognised sub-block keys (`["report", "detail"]`) are stored as
//!   `FieldNode` with `FieldValue::Template`.
//! - Unrecognised keys produce a non-fatal `W-PAR-001` warning via the
//!   `validate()`/`emitter.emit(Rich::custom(...))` idiom (DIR-077-001-A Ruling 2).
//!   The key IS retained in `SectionNode.fields` (not dropped).
//! - A sub-block key that matches a reserved register name used WITHOUT a colon
//!   suffix (EC-006) fires a FATAL `E-PAR-015` error with a corrective hint.
//! - Error recovery via `recover_with(skip_then_retry_until(...))` accumulates
//!   all errors; never bails on first (Q23, LOCKED).
//!
//! # STORY-078
//!
//! Introduced in STORY-078 (Parser: section block syntax). Prerequisite for
//! `STORY-077` (`SectionBlock` IR Extension).

use chumsky::{input::ValueInput, prelude::*};

use crate::{
    ast::{BlockItem, FieldNode, FieldValue, SectionNode},
    section::{is_register_sub_block_key, is_reserved_register_name},
    span::{Span, Spanned},
    template::TemplateChunk,
    token::Token,
};

use super::template::template_value;

// ─── Type aliases ─────────────────────────────────────────────────────────────

/// The span type used by chumsky in the token-stream parser.
type TSpan = SimpleSpan;

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Convert a chumsky `SimpleSpan` plus a `file_id` into a project [`Span`].
fn to_span(ss: SimpleSpan, file_id: u32) -> Span {
    Span::new(file_id, ss.start, ss.end)
}

/// Match an identifier token and return its string value and span.
fn any_ident<'src, I>()
-> impl Parser<'src, I, (String, TSpan), extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    select! { Token::Ident(s) = e => (s.to_string(), e.span()) }
}

/// Match an identifier token whose string value equals `kw`.
fn keyword<'src, I>(
    kw: &'static str,
) -> impl Parser<'src, I, TSpan, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    select! { Token::Ident(s) = e if s.as_ref() == kw => e.span() }
}

// ─── Section sub-block field value parser ─────────────────────────────────────

/// Parser for a field value inside a section sub-block.
///
/// Reuses the EXISTING [`template_value()`] combinator for string values
/// (Architecture Compliance Rule 2 — single source of truth for inline parsing).
/// Non-string literals are also accepted.
///
/// Errors from template parsing are emitted via `validate()`.
fn section_value_parser<'src, I>()
-> impl Parser<'src, I, (FieldValue, TSpan), extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    let template_val = template_value().validate(
        move |(chunks, errs): (Vec<TemplateChunk>, Vec<String>), info, emitter| {
            for msg in errs {
                emitter.emit(Rich::custom(info.span(), msg));
            }
            (FieldValue::Template(chunks), info.span())
        },
    );

    let other_val = select! {
        Token::IntLit(n) = e => (FieldValue::Num(n), e.span()),
        Token::FloatLit(f) = e => (FieldValue::Float(f), e.span()),
        Token::BoolLit(b) = e => (FieldValue::Bool(b), e.span()),
        Token::Ident(s) = e => (FieldValue::Ident(s.to_string()), e.span()),
    };

    template_val.or(other_val)
}

// ─── Section block parser ─────────────────────────────────────────────────────

/// Build the `section_block_parser` combinator.
///
/// Produces a [`BlockItem::Section`] from the grammar:
/// ```text
/// section_block ::= "section" IDENT ":" NEWLINE INDENT sub_block* DEDENT
/// sub_block     ::= IDENT value NEWLINE
/// ```
///
/// # Warning routing (DIR-077-001-A Ruling 2)
///
/// Unrecognised sub-block keys emit `W-PAR-001` diagnostics via
/// `emitter.emit(Rich::custom(...))` inside the `.validate()` closure. These
/// are distinguished from fatal errors by the `"W-PAR-"` prefix and are
/// routed into [`crate::parser::ParseResult::warnings`] by the error conversion
/// boundary in `parser/mod.rs`.
///
/// # Error recovery (Q23, LOCKED)
///
/// `recover_with(skip_then_retry_until(...))` is applied to each sub-block
/// parse so that all errors in the body are accumulated rather than
/// aborting on the first malformed field.
#[must_use]
pub fn section_block_parser<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, BlockItem, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    // A single sub-block field: `IDENT ":" value NEWLINE`.
    //
    // The colon after the key is required syntax — sub-block assignments always
    // use `key: value` form (consistent with the vars-block grammar).
    //
    // The `.validate()` closure checks the IDENT against the fixed compile-time
    // set `SECTION_REGISTER_KEYS = ["report", "detail"]`:
    //   - Recognised key        → stored as FieldNode, no diagnostic.
    //   - Unrecognised key that IS a reserved register name (EC-006) →
    //     FATAL E-PAR-015 with corrective hint.
    //   - Other unrecognised key (EC-005) →
    //     NON-FATAL W-PAR-001 warning; key retained in AST.
    let sub_block = any_ident()
        .then_ignore(just(Token::Colon))
        .then(section_value_parser())
        .then_ignore(just(Token::Newline).or_not())
        .validate(move |((key, key_span), (val, val_span)), info, emitter| {
            if !is_register_sub_block_key(&key) {
                if is_reserved_register_name(&key) {
                    // EC-006: reserved register name used without colon — FATAL.
                    emitter.emit(Rich::custom(
                        info.span(),
                        format!(
                            "E-PAR-015: Key '{key}' is a reserved register name — \
                             use '{key}:' register syntax or choose a different key."
                        ),
                    ));
                } else {
                    // EC-005: unrecognised key — non-fatal W-PAR-001 warning.
                    // The "W-PAR-" prefix causes the conversion boundary in
                    // parser/mod.rs to route this into ParseResult::warnings.
                    emitter.emit(Rich::custom(
                        key_span,
                        format!("W-PAR-001: Unrecognized section sub-block key '{key}' — ignored"),
                    ));
                }
            }
            FieldNode {
                name: Spanned::new(key, to_span(key_span, file_id)),
                value: Spanned::new(val, to_span(val_span, file_id)),
            }
        });

    // Accumulate all sub-block errors via recovery — never bail-on-first (Q23).
    let sub_block_with_recovery = sub_block.recover_with(skip_then_retry_until(
        any()
            .filter(|t| !matches!(t, Token::Newline | Token::Dedent))
            .ignored(),
        just(Token::Newline).ignored(),
    ));

    keyword("section")
        .then(any_ident())
        .then_ignore(just(Token::Colon))
        .then_ignore(just(Token::Newline).or_not())
        .then_ignore(select! { Token::Indent(_) => () })
        .then(sub_block_with_recovery.repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::Dedent))
        .map_with(move |((_section_kw_span, (kind, kind_span)), fields), e| {
            let section_span = to_span(e.span(), file_id);
            BlockItem::Section(Spanned::new(
                SectionNode {
                    kind: Spanned::new(kind, to_span(kind_span, file_id)),
                    fields,
                },
                section_span,
            ))
        })
}
