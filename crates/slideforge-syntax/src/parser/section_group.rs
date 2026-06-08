//! Parser combinator for `section "Name":` slide-grouping blocks.
//!
//! Implements `section_group_parser` — the combinator that parses a single
//! `section "Name": INDENT slide_block* DEDENT` production into a
//! [`SectionGroupNode`].
//!
//! # Grammar (STORY-082 §1)
//!
//! ```text
//! section_group ::= "section" STRING ":" NEWLINE INDENT slide_block* DEDENT
//! ```
//!
//! This form is DISTINCT from the `section IDENT:` form (STORY-078).
//! Disambiguation: if the token after `section` is a STRING literal, this
//! parser fires; if it is an IDENT, the existing `section_block_parser` fires.
//!
//! # Error Cases
//!
//! - Empty quoted name (`section "":`) → `ParseError::EmptySectionGroupName`
//!   (E-PAR-023). Error accumulated; no `SectionGroupNode` produced.
//! - Duplicate name in the same deck → `ParseWarning::DuplicateSectionGroupName`
//!   (W-PAR-002). Warning accumulated; both nodes emitted.
//!
//! # STORY-082
//!
//! Introduced in STORY-082 (PPTX: Slide-Grouping Sections).

use std::sync::Arc;

use chumsky::{input::ValueInput, prelude::*};

use crate::{
    ast::{BlockItem, SectionGroupNode},
    span::{Span, Spanned},
    token::Token,
};

// ─── Type aliases ─────────────────────────────────────────────────────────────

/// The span type used by chumsky in the token-stream parser.
type TSpan = SimpleSpan;

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Convert a chumsky `SimpleSpan` plus a `file_id` into a project [`Span`].
fn to_span(ss: SimpleSpan, file_id: u32) -> Span {
    Span::new(file_id, ss.start, ss.end)
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

// ─── Section group parser ─────────────────────────────────────────────────────

/// Build the `section_group_parser` combinator.
///
/// Produces a [`BlockItem::SectionGroup`] from the grammar:
/// ```text
/// section_group ::= "section" STRING ":" NEWLINE INDENT block_item* DEDENT
/// ```
///
/// The `block_item_sub` parameter is the same `block_item` combinator used at
/// deck level. This allows the section body to contain `slide`, `@for`, and
/// `@if` blocks just like the top-level deck.
///
/// # Empty name detection (AC-010 / EC-010)
///
/// An empty quoted name (`""`) emits `E-PAR-023` via `validate()` and produces
/// no `SectionGroupNode` for the rejected block. Error is accumulated.
///
/// # Duplicate name detection (AC-011 / EC-011)
///
/// Duplicate name detection across the whole deck is performed at the deck level
/// by the post-parse validation pass in `deck_parser`. This combinator emits the
/// `SectionGroupNode` without checking for duplicates; the deck-level pass emits
/// `W-PAR-002`.
///
/// # STORY-082
///
/// Introduced in STORY-082.
#[must_use]
pub fn section_group_parser<'src, I, P>(
    file_id: u32,
    block_item_sub: P,
) -> impl Parser<'src, I, BlockItem, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
    P: Parser<'src, I, BlockItem, extra::Err<Rich<'src, Token, TSpan>>> + Clone + 'src,
{
    // Match a STRING token and return its content and span.
    let string_token = select! { Token::StringLit(s) = e => (s.to_string(), e.span()) };

    let nl = just(Token::Newline).ignored();

    keyword("section")
        .then(string_token)
        .then_ignore(just(Token::Colon))
        .then_ignore(just(Token::Newline).or_not())
        .then_ignore(select! { Token::Indent(_) => () })
        // Parse actual slide/control-flow children using the real block_item sub-parser.
        .then(
            nl.clone()
                .repeated()
                .ignore_then(block_item_sub)
                .then_ignore(nl.repeated())
                .repeated()
                .collect::<Vec<BlockItem>>(),
        )
        .then_ignore(just(Token::Dedent))
        .validate(move |((kw_span, (name_str, name_tspan)), slide_children), info, emitter| {
            let group_span = to_span(info.span(), file_id);

            if name_str.is_empty() {
                // E-PAR-023: empty section group name rejected.
                // Emit via the structured routing message that parser/mod.rs converts
                // to SyntaxError::EmptySectionGroupName.
                emitter.emit(Rich::custom(
                    name_tspan,
                    format!(
                        "E-PAR-023: section group name must be non-empty (at {:?}). \
                         Provide a quoted, non-empty name, e.g. section \"Background\":",
                        to_span(name_tspan, file_id)
                    ),
                ));
                // Produce a sentinel BlockItem that is filtered out in the deck pass.
                // Using a real node with a sentinel name is safer than returning None.
                // The deck_parser discards nodes with empty names.
                return BlockItem::SectionGroup(Spanned::new(
                    SectionGroupNode {
                        name: Spanned::new(Arc::from(""), to_span(name_tspan, file_id)),
                        slides: vec![],
                    },
                    group_span,
                ));
            }

            let _ = kw_span; // consumed by parser; span is in info.span()
            BlockItem::SectionGroup(Spanned::new(
                SectionGroupNode {
                    name: Spanned::new(Arc::from(name_str.as_str()), to_span(name_tspan, file_id)),
                    slides: slide_children,
                },
                group_span,
            ))
        })
}
