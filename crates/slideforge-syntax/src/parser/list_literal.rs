//! Shared list-literal element validator used by ALL four value-position parsers.
//!
//! This module provides [`list_literal_elements`] and [`list_literal_elements_cf`],
//! two thin wrappers over a single internal combinator that produces the element
//! list for a `[item, item, ...]` list literal.
//!
//! # Motivation
//!
//! Four value-position parsers in the slideforge DSL accept list literals:
//!
//! 1. `deck.rs::value_parser` — `vars:` block and `@var` deck-level assignments
//! 2. `control_flow.rs::field_line_cf` — slide-body field assignments (production path)
//! 3. `deck.rs::set_rule_value_parser` — `set <type>: <field> [...]` rules
//! 4. `variants.rs::variant_value` — `variants: <name>: vars: <key>: [...]` entries
//!
//! Previously each site duplicated the list-item validation logic, causing
//! message inconsistencies (F-088-P5-MED-001 and F-088-P5-MED-002): sites 3
//! and 4 used a generic message without the per-type `got <type>` substitution
//! required by error-taxonomy v2.28 §E-PAR-024, and site 2 produced a cryptic
//! `ExpectedFound(LBracket)` for nested-list attempts instead of E-PAR-024
//! "nested list".
//!
//! This single shared combinator eliminates the quad-duplication. All four
//! sites route through it, guaranteeing identical E-PAR-024 emission (including
//! `got <type>` and `got nested list`) for the same malformed input in any
//! value-position context.
//!
//! # Grammar
//!
//! ```text
//! list_literal ::= "[" (list_item ("," list_item)*)? ","? "]"
//! list_item    ::= template_string   -- valid: FieldValue::Template
//!                | INT               -- invalid: E-PAR-024 "got integer"
//!                | FLOAT             -- invalid: E-PAR-024 "got decimal number"
//!                | BOOL              -- invalid: E-PAR-024 "got boolean"
//!                | IDENT             -- invalid: E-PAR-024 "got bare word"
//!                | "[" ...           -- invalid: E-PAR-024 "got nested list"
//! ```
//!
//! # Error Contract (BC-1.15.001 error accumulation)
//!
//! - Non-string items emit E-PAR-024 with `got <type>` and continue parsing.
//! - Nested list items `[` emit E-PAR-024 with `got nested list` and the inner
//!   list is consumed to resume after `]`.
//! - Parsing never fails on the first error — all items are inspected.

use chumsky::{input::ValueInput, prelude::*};

use crate::{ast::FieldValue, template::TemplateChunk, token::Token};

use super::template::template_value;

// ─── Type alias ───────────────────────────────────────────────────────────────

/// The span type used by chumsky in the token-stream parser.
type TSpan = SimpleSpan;

// ─── Internal implementation ──────────────────────────────────────────────────

/// Build the core list-literal elements parser.
///
/// `exclude_in`: when `true`, the bare-word alternative rejects the literal
/// identifier string `"in"` (for the `control_flow` slide-body path where `in`
/// is a structural keyword in `@for x in coll:` expressions).
///
/// # Recursion structure
///
/// The `recursive` combinator here wraps the **single item** parser — NOT the
/// full `[...] ` list parser. The recursive variable `single_item_ref` is used
/// by `nested_item` to consume and discard the contents of a nested `[...]`,
/// enabling a canonical E-PAR-024 "nested list" error rather than the cryptic
/// `ExpectedFound(LBracket)` that would result from a non-recursive grammar
/// (F-088-P5-MED-002 fix).
fn list_literal_elements_impl<'src, I>(
    exclude_in: bool,
) -> impl Parser<'src, I, Vec<(FieldValue, TSpan)>, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    // `recursive` wraps the SINGLE-ITEM parser. The full list `[items...]` is
    // assembled outside the recursive closure using `separated_by`.
    let single_item = recursive(move |single_item_ref| {
        // ── Valid item: template string (quoted literal or {{ expr }}) ──────────
        let template_item = template_value().validate(
            move |(chunks, errs): (
                Vec<TemplateChunk>,
                Vec<crate::parser::template::TemplateError>,
            ),
                  info,
                  emitter| {
                for err in errs {
                    emitter.emit(Rich::custom(info.span(), err.into_routing_message()));
                }
                (FieldValue::Template(chunks), info.span())
            },
        );

        // ── Invalid: nested list `[...]` ─────────────────────────────────────────
        //
        // Recognise `[` at list-item position, consume the nested items recursively,
        // and emit E-PAR-024 "got nested list". This replaces the previous grammar-level
        // rejection (ExpectedFound(LBracket)) with a canonical E-PAR-024 diagnostic
        // (F-088-P5-MED-002 fix).
        let nested_item = single_item_ref
            .clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map_with(|_inner, e| (FieldValue::Error, e.span()))
            .validate(|(_, item_span): (FieldValue, TSpan), _info, emitter| {
                emitter.emit(Rich::custom(
                    item_span,
                    "E-PAR-024: non-string list item. \
                     List items must be quoted string literals; got nested list. \
                     Wrap the value in quotes to use it as a string."
                        .to_string(),
                ));
                (FieldValue::Error, item_span)
            });

        // ── Invalid: non-string primitives (Int/Float/Bool/Ident) ────────────────
        //
        // The ident variant optionally excludes `"in"` for the control_flow path.
        // The validate() emits E-PAR-024 with the per-type `got <type>` substitution
        // (error-taxonomy v2.28 §E-PAR-024 binding format).
        let primitive_item = select! {
            Token::IntLit(n) = e => (FieldValue::Num(n), e.span()),
            Token::FloatLit(f) = e => (FieldValue::Float(f), e.span()),
            Token::BoolLit(b) = e => (FieldValue::Bool(b), e.span()),
            Token::Ident(s) = e => (FieldValue::Ident(s.to_string()), e.span()),
        }
        // For the control_flow path, exclude the `"in"` identifier so that `@for`
        // structural keywords are not consumed as bare-word list elements.
        .filter(move |(item, _span): &(FieldValue, TSpan)| {
            if exclude_in {
                !matches!(item, FieldValue::Ident(s) if s == "in")
            } else {
                true
            }
        })
        .validate(
            move |(item, item_span): (FieldValue, TSpan), _info, emitter| {
                let type_name = match &item {
                    FieldValue::Num(_) => "integer",
                    FieldValue::Float(_) => "decimal number",
                    FieldValue::Bool(_) => "boolean",
                    FieldValue::Ident(_) => "bare word",
                    FieldValue::Shape(_) => "shape block",
                    // Error sentinel — already reported.
                    FieldValue::Error => "error sentinel",
                    // Defense-in-depth for direct AST construction.
                    FieldValue::List(_) => "nested list",
                    // Template is matched above.
                    FieldValue::Template(_) => unreachable!(
                        "Template is handled by template_item — should not reach here"
                    ),
                };
                emitter.emit(Rich::custom(
                    item_span,
                    format!(
                        "E-PAR-024: non-string list item. \
                         List items must be quoted string literals; got {type_name}. \
                         Wrap the value in quotes to use it as a string."
                    ),
                ));
                (FieldValue::Error, item_span)
            },
        );

        // Priority: template (valid) → nested list detection → primitive rejection.
        template_item.or(nested_item).or(primitive_item)
    });

    // Wrap the single-item parser in `[` ... `]` with comma separation and
    // optional trailing comma. Token-stream idiom: just(Token::LBracket) etc.
    // allow_trailing() accepts `["A", "B",]` (EC-003 compliance).
    single_item
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LBracket), just(Token::RBracket))
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Build a list-literal elements parser for `vars:`, `@var`, `set_rule`, and
/// `variant_value` contexts (positions 1, 3, 4).
///
/// All identifier tokens are treated as bare words. No keyword exclusion.
///
/// Returns a parser matching `"[" (item ("," item)*)? ","? "]"` and producing
/// `Vec<(FieldValue, TSpan)>`. Callers wrap into `FieldValue::List` or
/// `SetRuleValue::List` as appropriate.
///
/// # Example
///
/// ```ignore
/// let list_val = list_literal_elements()
///     .map_with(|items, e| {
///         let items_fv = items.into_iter().map(|(fv, _)| fv).collect();
///         (FieldValue::List(items_fv), e.span())
///     });
/// ```
#[must_use]
pub fn list_literal_elements<'src, I>()
-> impl Parser<'src, I, Vec<(FieldValue, TSpan)>, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    list_literal_elements_impl(false)
}

/// Build a list-literal elements parser for the `control_flow` slide-body context
/// (position 2).
///
/// Identical to [`list_literal_elements`] but excludes the `"in"` identifier from
/// bare-word items to avoid consuming the `@for x in coll:` structural keyword as
/// a list element.
#[must_use]
pub fn list_literal_elements_cf<'src, I>()
-> impl Parser<'src, I, Vec<(FieldValue, TSpan)>, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    list_literal_elements_impl(true)
}
