//! Top-level deck parser combinator.
//!
//! Implements the `deck()` parser which produces a [`DeckNode`] from a flat
//! stream of [`Token`]s. Error accumulation is performed via chumsky 0.10's
//! `recover_with(skip_then_retry_until(...))` strategy — all errors are
//! collected before returning, never halting on the first error.
//!
//! # Grammar (key productions)
//!
//! ```text
//! deck         ::= version_decl? lang_decl? brand_decl? (vars_block | set_rule | slide_block)*
//! version_decl ::= "slideforge_version" STRING NEWLINE
//! lang_decl    ::= "lang" STRING NEWLINE
//! brand_decl   ::= "brand" STRING NEWLINE
//! vars_block   ::= "vars" ":" INDENT (IDENT ":" value NEWLINE)+ DEDENT
//! set_rule     ::= "set" IDENT ":" IDENT value NEWLINE
//! slide_block  ::= "slide" IDENT ":" INDENT (field_line)+ DEDENT
//! field_line   ::= IDENT value NEWLINE
//! value        ::= STRING | INT | FLOAT | BOOL | IDENT
//! ```

use chumsky::{input::ValueInput, prelude::*};

use crate::{
    ast::{
        AliasNode, BlockItem, DeckNode, FieldNode, FieldValue, SetRule, SetRuleValue, SlideNode,
        VariantsBlock, VarsBlock,
    },
    keywords::{classify_keyword, is_slide_type_keyword},
    known_fields::{known_fields, suggest_type},
    span::{Span, Spanned},
    template::TemplateChunk,
    token::Token,
};

use super::{
    alias::{AliasRegistry, alias_decl},
    control_flow::block_item,
    section::section_block_parser,
    section_group::section_group_parser,
    template::template_value,
    variants::variants_block,
};

// ─── Type aliases ─────────────────────────────────────────────────────────────

/// The span type used by chumsky in the token-stream parser.
///
/// We use `SimpleSpan` (a `Range<usize>`) since the lexer emits
/// `(Token, Range<usize>)` pairs.
type TSpan = SimpleSpan;

/// Convert a chumsky `SimpleSpan` plus a `file_id` into a project [`Span`].
fn to_span(ss: SimpleSpan, file_id: u32) -> Span {
    Span::new(file_id, ss.start, ss.end)
}

// ─── Value parser ────────────────────────────────────────────────────────────

/// Parser for a field value: string literal (with template interpolation),
/// integer, float, bool, identifier, or list literal.
///
/// String literals are parsed through `template_value()` so that `{{ expr }}`
/// interpolation is supported. Template errors are emitted via `validate()`.
///
/// List literals `[item, item, ...]` produce [`FieldValue::List`]. Only string
/// items are valid inside a list literal for `bullets:` and similar fields;
/// non-string items (numbers, bools) emit an `E-PAR-024` diagnostic and are
/// converted to [`FieldValue::Error`] sentinels, but parsing continues so that
/// all errors in the file are accumulated (BC-1.15.001 error-accumulation).
///
/// On error, produces `FieldValue::Error` sentinel for recovery.
fn value_parser<'src, I>()
-> impl Parser<'src, I, (FieldValue, TSpan), extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    recursive(move |value_parser_ref| {
        // Template string: emit any E-PAR-012/E-PAR-013/E-PAR-014 errors via validate().
        let template_val = template_value().validate(
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

        // Non-string field values.
        let other_val = select! {
            Token::IntLit(n) = e => (FieldValue::Num(n), e.span()),
            Token::FloatLit(f) = e => (FieldValue::Float(f), e.span()),
            Token::BoolLit(b) = e => (FieldValue::Bool(b), e.span()),
            Token::Ident(s) = e => (FieldValue::Ident(s.to_string()), e.span()),
        };

        // List items: only template (string) items are valid for bullet-list fields.
        // Non-string list items — integers, floats, booleans, and bare words — emit
        // E-PAR-024 (non-string list item) and are substituted with FieldValue::Error
        // sentinels so that the evaluator's --warn-only path never renders a coerced
        // non-string value as a bullet (BC-1.15.001 error accumulation).
        //
        // AC-005 / EC-002 / EC-005: non-string items must produce a diagnostic with
        // the actual type name, not a silent wrong value or a panic.
        let list_item = value_parser_ref.validate(
            move |(item, item_span): (FieldValue, TSpan), _info, emitter| {
                // Only template strings (quoted string literals) are valid list items.
                // Nested lists are NOT in scope (STORY-088 spec, line ~185);
                // silently accepting them would violate the no-silent-failure ban.
                if let FieldValue::Template(_) = &item {
                    (item, item_span)
                } else {
                    // Non-string item: map to a human-readable type name and emit
                    // E-PAR-024 with the actual type substituted for <type>
                    // (error-taxonomy v2.27 §E-PAR-024 binding format).
                    let type_name = match &item {
                        FieldValue::Num(_) => "integer",
                        FieldValue::Float(_) => "decimal number",
                        FieldValue::Bool(_) => "boolean",
                        FieldValue::Ident(_) => "bare word",
                        FieldValue::Shape(_) => "shape block",
                        // Error sentinel from a nested recovery path — already reported.
                        FieldValue::Error => "error sentinel",
                        // Nested list literals are out of scope (STORY-088 spec);
                        // reject them explicitly so no items are silently dropped.
                        FieldValue::List(_) => "nested list",
                        // Template is matched in the outer arm above.
                        FieldValue::Template(_) => unreachable!(),
                    };
                    emitter.emit(Rich::custom(
                        item_span,
                        format!(
                            "E-PAR-024: non-string list item. \
                             List items must be quoted string literals; got {type_name}. \
                             Wrap the value in quotes to use it as a string."
                        ),
                    ));
                    // Substitute FieldValue::Error so the evaluator's --warn-only
                    // path skips this item rather than coercing it to a wrong value.
                    (FieldValue::Error, item_span)
                }
            },
        );

        // List literal: `[` (list_item (`,` list_item)*)? `]`
        //
        // Following the chumsky 0.10 token-stream idiom from expr.rs lines 96-103:
        // use just(Token::LBracket) / just(Token::Comma) / just(Token::RBracket) —
        // NOT char-stream combinators like just('[').
        //
        // allow_trailing() so that `["A", "B",]` (trailing comma) is accepted.
        // AC-002 (empty list): `[]` produces FieldValue::List(vec![]) without error.
        let list_val = list_item
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map_with(move |items, e| {
                let items_fv: Vec<FieldValue> = items.into_iter().map(|(fv, _span)| fv).collect();
                (FieldValue::List(items_fv), e.span())
            });

        // Priority: list_val first so `[...]` is not misinterpreted as an ident.
        list_val.or(template_val).or(other_val)
    })
}

/// Parser for a `set` rule value: string literal (with template interpolation),
/// integer, float, bool, identifier, or list literal.
///
/// Identical in surface syntax to `value_parser()` but produces [`SetRuleValue`]
/// so that `set` rules carry a distinct type from field values.
///
/// BC-1.08.002: `{{ brand.footer }}` is parsed as
/// `SetRuleValue::Template([TemplateChunk::Expr(Expr::FieldAccess { base:
/// Ident("brand"), field: "footer" })])`. The expression parser handles this
/// naturally via field-access syntax — no special `BrandRef` token is needed.
///
/// # STORY-088 AC-012
///
/// List literals `[item, item, ...]` produce [`SetRuleValue::List`]. Only string
/// items are valid inside a set-rule list literal; non-string items emit E-PAR-024
/// (same rules as `value_parser()`) and are substituted with [`FieldValue::Error`]
/// sentinels (BC-1.15.001 error accumulation). Nested lists are rejected.
fn set_rule_value_parser<'src, I>()
-> impl Parser<'src, I, (SetRuleValue, TSpan), extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    let template_val = template_value().validate(
        move |(chunks, errs): (
            Vec<TemplateChunk>,
            Vec<crate::parser::template::TemplateError>,
        ),
              info,
              emitter| {
            for err in errs {
                emitter.emit(Rich::custom(info.span(), err.into_routing_message()));
            }
            (SetRuleValue::Template(chunks), info.span())
        },
    );

    let other_val = select! {
        Token::IntLit(n) = e => (SetRuleValue::Num(n), e.span()),
        Token::FloatLit(f) = e => (SetRuleValue::Float(f), e.span()),
        Token::BoolLit(b) = e => (SetRuleValue::Bool(b), e.span()),
        Token::Ident(s) = e => (SetRuleValue::Ident(s.to_string()), e.span()),
    };

    // List literal for set-rule position: `["A", "B", ...]` → SetRuleValue::List.
    //
    // LESSON-19: reuse the same E-PAR-024 validation and flat-only list semantics
    // as value_parser()'s list_item validator.  The item type is FieldValue (not
    // SetRuleValue) so that the evaluator can dispatch each item through the
    // standard `eval_field_value_to_value` path rather than duplicating the
    // string-evaluation logic.
    //
    // Any non-Template item emits E-PAR-024 and is substituted with FieldValue::Error.
    // Nested lists ([["A"]]) are structurally rejected by the grammar: the list_item
    // combinator only recognises template_value() + non-string primitive tokens.
    // AC-012 / STORY-088.
    let list_item_for_set = template_value()
        .validate(
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
        )
        .or(
            // Non-string primitive items: emit E-PAR-024 and recover with Error sentinel.
            select! {
                Token::IntLit(_) = e => (FieldValue::Error, e.span()),
                Token::FloatLit(_) = e => (FieldValue::Error, e.span()),
                Token::BoolLit(_) = e => (FieldValue::Error, e.span()),
                Token::Ident(_) = e => (FieldValue::Error, e.span()),
            }
            .validate(move |(_, item_span): (FieldValue, TSpan), _info, emitter| {
                emitter.emit(Rich::custom(
                    item_span,
                    "E-PAR-024: non-string list item. \
                     List items must be quoted string literals. \
                     Wrap the value in quotes to use it as a string."
                        .to_string(),
                ));
                (FieldValue::Error, item_span)
            }),
        );

    let list_val_for_set = list_item_for_set
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LBracket), just(Token::RBracket))
        .map_with(move |items, e| {
            let items_fv: Vec<FieldValue> = items.into_iter().map(|(fv, _span)| fv).collect();
            (SetRuleValue::List(items_fv), e.span())
        });

    // Priority: list_val first so `[...]` is not misinterpreted as an ident.
    list_val_for_set.or(template_val).or(other_val)
}

// ─── Keyword matchers ─────────────────────────────────────────────────────────

/// Match an identifier token whose string value equals `kw`.
///
/// Returns the span of the matched token. This avoids using `select!` with
/// both a guard and a span binding (which would conflict).
fn keyword<'src, I>(
    kw: &'static str,
) -> impl Parser<'src, I, TSpan, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    select! { Token::Ident(s) = e if s.as_ref() == kw => e.span() }
}

/// Match any identifier token and return its string value and span.
fn any_ident<'src, I>()
-> impl Parser<'src, I, (String, TSpan), extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    select! { Token::Ident(s) = e => (s.to_string(), e.span()) }
}

/// Match any string literal and return its content and span.
fn string_lit<'src, I>()
-> impl Parser<'src, I, (String, TSpan), extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    select! { Token::StringLit(s) = e => (s.to_string(), e.span()) }
}

// ─── Field line parser ────────────────────────────────────────────────────────

/// Parser for a single field assignment: `<name> <value> NEWLINE`.
/// Superseded by `control_flow::field_line_cf` in STORY-007 (adds template support).
#[allow(dead_code)]
fn field_line_parser<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, FieldNode, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    any_ident()
        .then(value_parser())
        .then_ignore(just(Token::Newline).or_not())
        .map(move |((name, name_span), (val, val_span))| FieldNode {
            name: Spanned::new(name, to_span(name_span, file_id)),
            value: Spanned::new(val, to_span(val_span, file_id)),
        })
}

// ─── Slide block parser ───────────────────────────────────────────────────────

/// Parser for a `slide <type>:` block with indented fields.
///
/// Pattern: `"slide" IDENT ":" INDENT field_line* DEDENT`
/// Superseded by `control_flow::block_item` slide arm in STORY-007.
#[allow(dead_code)]
fn slide_block_parser<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, (SlideNode, TSpan), extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    keyword("slide")
        .then(any_ident())
        .then_ignore(just(Token::Colon))
        .then_ignore(just(Token::Newline).or_not())
        .then_ignore(select! { Token::Indent(_) => () })
        .then(
            field_line_parser(file_id)
                .recover_with(skip_then_retry_until(
                    any()
                        .filter(|t| !matches!(t, Token::Newline | Token::Dedent))
                        .ignored(),
                    just(Token::Newline).ignored(),
                ))
                .repeated()
                .collect::<Vec<_>>(),
        )
        .then_ignore(just(Token::Dedent))
        .map_with(move |((_slide_kw_span, (kind, kind_span)), fields), e| {
            let slide_span = e.span();
            (
                SlideNode {
                    kind: Spanned::new(kind, to_span(kind_span, file_id)),
                    tags: Vec::new(),
                    fields,
                    inline_items: Vec::new(),
                },
                slide_span,
            )
        })
}

// ─── Vars block parser ────────────────────────────────────────────────────────

/// Parser for a `vars:` block with indented key-value entries.
///
/// Pattern: `"vars" ":" NEWLINE INDENT (IDENT value NEWLINE)+ DEDENT`
fn vars_block_parser<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, VarsBlock, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    // A single vars entry with name collision checking.
    //
    // AC-011/AC-013/AC-014: variable names that collide with slide type
    // keywords (exact match) produce E-PAR-008 (VarNameCollision).
    // The `raw` keyword also produces E-PAR-008 (not E-PAR-009) when used as
    // a variable name, because the collision check fires before the raw-keyword
    // check.
    let vars_entry = any_ident()
        .then_ignore(just(Token::Colon))
        .then(value_parser())
        .then_ignore(just(Token::Newline).or_not())
        .validate(move |((name, name_span), (val, val_span)), info, emitter| {
            // Check for keyword collision.
            //
            // Priority: slide-type keywords → E-PAR-008.
            //           `raw` in RESERVED_KEYWORDS → still E-PAR-008 (AC-011).
            //           Other structural keywords → also E-PAR-008.
            let collision = if is_slide_type_keyword(&name) {
                Some(format!(
                    "E-PAR-008: '{name}' is a built-in slide type keyword and cannot \
                     be used as a variable name. Use a different name, e.g. '{name}_data'."
                ))
            } else if classify_keyword(&name).is_some() {
                // Structural keywords and `raw` all produce E-PAR-008 in vars context.
                Some(format!(
                    "E-PAR-008: '{name}' is a reserved keyword and cannot be used as \
                     a variable name."
                ))
            } else {
                None
            };

            if let Some(msg) = collision {
                emitter.emit(Rich::custom(info.span(), msg));
            }

            (
                Spanned::new(name, to_span(name_span, file_id)),
                Spanned::new(val, to_span(val_span, file_id)),
            )
        });

    keyword("vars")
        .then_ignore(just(Token::Colon))
        .then_ignore(just(Token::Newline).or_not())
        .then_ignore(select! { Token::Indent(_) => () })
        .then(vars_entry.repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::Dedent))
        .map(|(_vars_kw_span, entries)| VarsBlock { entries })
}

// ─── @var inline assignment parser ───────────────────────────────────────────

/// Parser for an `@var ident = value` inline variable assignment at deck level.
///
/// Pattern: `"@" "var" IDENT "=" value NEWLINE`
///
/// Produces a single-entry [`VarsBlock`] equivalent to a `vars:` block
/// with one entry. This is the DSL's shorthand for binding a single variable
/// at deck level.
///
/// # STORY-088
///
/// This production is required for the `@var items = ["A","B","C"]` form
/// that binds a list literal to a variable at deck level. The fixture
/// `story-086-bullets-slide.sf` uses this form; the vars-block form is
/// also supported via [`vars_block_parser`].
///
/// # Variable name collision checking
///
/// Same collision check as `vars_block_parser`: variable names that collide
/// with slide type keywords produce E-PAR-008. The check is identical to
/// the `vars_entry` validator in `vars_block_parser`.
fn at_var_parser<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, VarsBlock, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    // `@` `var` IDENT `=` value NEWLINE
    just(Token::At)
        .ignore_then(keyword("var"))
        .ignore_then(any_ident())
        .then_ignore(just(Token::Eq))
        .then(value_parser())
        .then_ignore(just(Token::Newline).or_not())
        .validate(move |((name, name_span), (val, val_span)), info, emitter| {
            // Same keyword collision check as vars_block_parser vars_entry.
            let collision = if is_slide_type_keyword(&name) {
                Some(format!(
                    "E-PAR-008: '{name}' is a built-in slide type keyword and cannot \
                         be used as a variable name. Use a different name, e.g. '{name}_data'."
                ))
            } else if classify_keyword(&name).is_some() {
                Some(format!(
                    "E-PAR-008: '{name}' is a reserved keyword and cannot be used as \
                         a variable name."
                ))
            } else {
                None
            };

            if let Some(msg) = collision {
                emitter.emit(Rich::custom(info.span(), msg));
            }

            VarsBlock {
                entries: vec![(
                    Spanned::new(name, to_span(name_span, file_id)),
                    Spanned::new(val, to_span(val_span, file_id)),
                )],
            }
        })
}

// ─── Set rule parser ──────────────────────────────────────────────────────────

/// Parser for a `set <type>: <field> <value>` rule.
///
/// Pattern: `"set" IDENT ":" IDENT value NEWLINE`
///
/// STORY-008: The value is now parsed as [`SetRuleValue`] (not [`FieldValue`])
/// so that `set` rule values are clearly distinguished from slide field values.
/// Brand references like `{{ brand.footer }}` are stored as
/// `SetRuleValue::Template([TemplateChunk::Expr(FieldAccess { ... })])` and
/// are NOT evaluated at parse time (BC-1.08.003).
///
/// BC-1.08.001 (EC-001): `set unknown_type: field "x"` emits E-PAR-007
/// "Unknown slide type '`unknown_type`'. Did you mean '...'?" via `validate()`.
fn set_rule_parser<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, Option<SetRule>, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    keyword("set")
        .then(any_ident())
        .then_ignore(just(Token::Colon))
        .then(any_ident())
        .then(set_rule_value_parser())
        .then_ignore(just(Token::Newline).or_not())
        .validate(
            move |(
                ((_set_kw_span, (slide_type, type_span)), (field, field_span)),
                (val, val_span),
            ),
                  info,
                  emitter| {
                // E-PAR-007: validate slide type.
                if known_fields(&slide_type).is_none() {
                    let suggestion = suggest_type(&slide_type)
                        .map_or_else(String::new, |s| format!(" Did you mean '{s}'?"));
                    emitter.emit(Rich::custom(
                        info.span(),
                        format!("E-PAR-007: Unknown slide type '{slide_type}'. {suggestion}"),
                    ));
                    return None;
                }
                Some(SetRule {
                    slide_type: Spanned::new(slide_type, to_span(type_span, file_id)),
                    field: Spanned::new(field, to_span(field_span, file_id)),
                    value: Spanned::new(val, to_span(val_span, file_id)),
                })
            },
        )
}

// ─── @include directive parser ────────────────────────────────────────────────

/// Parser for an `@include "path.sf"` directive.
///
/// The directive is represented as a synthetic `BlockItem::Slide` with
/// `kind == "@include"` and a single `FieldNode` containing the path template.
/// This intermediate representation is consumed by [`crate::include::resolve_includes`]
/// after the initial parse.
///
/// BC-1.06.001: The path may contain `{{ expr }}` interpolation (resolved at
/// include-expansion time against the current vars scope).
fn include_directive_parser<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, BlockItem, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    just(Token::At)
        .ignore_then(keyword("include"))
        .then(template_value().validate(
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
        ))
        .then_ignore(just(Token::Newline).or_not())
        .map_with(move |(_kw_span, (path_val, path_span)), e| {
            let slide_span = e.span();
            // Represent @include as a synthetic slide with kind "@include".
            BlockItem::Slide(Spanned::new(
                SlideNode {
                    kind: Spanned::new("@include".to_string(), to_span(slide_span, file_id)),
                    tags: vec![],
                    fields: vec![FieldNode {
                        name: Spanned::new("path".to_string(), to_span(path_span, file_id)),
                        value: Spanned::new(path_val, to_span(path_span, file_id)),
                    }],
                    inline_items: vec![],
                },
                to_span(slide_span, file_id),
            ))
        })
}

// ─── Deck metadata parsers ───────────────────────────────────────────────────

/// Parser for `slideforge_version "VERSION" NEWLINE`.
fn version_decl_parser<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, Spanned<String>, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    keyword("slideforge_version")
        .then(string_lit())
        .then_ignore(just(Token::Newline).or_not())
        .map(move |(_kw_span, (val, val_span))| Spanned::new(val, to_span(val_span, file_id)))
}

/// Parser for `lang "LANG" NEWLINE`.
fn lang_decl_parser<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, Spanned<String>, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    keyword("lang")
        .then(string_lit())
        .then_ignore(just(Token::Newline).or_not())
        .map(move |(_kw_span, (val, val_span))| Spanned::new(val, to_span(val_span, file_id)))
}

/// Parser for `brand "BRAND" NEWLINE`.
fn brand_decl_parser<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, Spanned<String>, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    keyword("brand")
        .then(string_lit())
        .then_ignore(just(Token::Newline).or_not())
        .map(move |(_kw_span, (val, val_span))| Spanned::new(val, to_span(val_span, file_id)))
}

// ─── Top-level deck enum ──────────────────────────────────────────────────────

/// A single top-level item that can appear in a deck.
#[derive(Debug)]
enum DeckItem {
    /// A `slideforge_version "N"` declaration.
    Version(Spanned<String>),
    /// A `lang "..."` declaration.
    Lang(Spanned<String>),
    /// A `brand "..."` declaration.
    Brand(Spanned<String>),
    /// A `vars:` block.
    Vars(VarsBlock),
    /// A `set <type>: <field> <value>` rule (valid type).
    Set(SetRule),
    /// A `set` rule with an unknown type — already emitted E-PAR-007; discard.
    SetErr,
    /// A `variants:` block.
    Variants(VariantsBlock),
    /// An `alias <name> = <type>: ...` declaration (will be expanded).
    Alias(AliasNode),
    /// An alias declaration with errors — already emitted; discard.
    AliasErr,
    /// An `@include "path"` directive (synthetic placeholder for resolution).
    Include(BlockItem),
    /// A `section <type>: ...` block (STORY-078).
    Section(BlockItem),
    /// A `section "Name": ...` slide-grouping block (STORY-082).
    SectionGroup(BlockItem),
    /// A block item: `slide`, `@for`, or `@if` block.
    Block(BlockItem),
}

// ─── Public deck parser ───────────────────────────────────────────────────────

/// Build the top-level deck parser.
///
/// Returns a chumsky parser that consumes a [`ValueInput`] token stream and
/// produces a [`DeckNode`].
///
/// # STORY-008 Additions
///
/// - `variants:` block → populates `DeckNode.variants` and `DeckNode.variant_names`.
/// - `alias` declarations → registered in `AliasRegistry`; subsequent `slide
///   <alias_name>:` blocks are expanded at parse time.
/// - `@include "path.sf"` directives → emitted as synthetic `BlockItem::Slide`
///   placeholders for post-parse resolution by [`crate::include::resolve_includes`].
/// - `set` rules with unknown slide types → E-PAR-007 emitted via `validate()`.
pub fn deck_parser<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, DeckNode, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    // Skip bare newlines between top-level items.
    let nl = just(Token::Newline).ignored();

    let version = version_decl_parser(file_id).map(DeckItem::Version);
    let lang = lang_decl_parser(file_id).map(DeckItem::Lang);
    let brand = brand_decl_parser(file_id).map(DeckItem::Brand);
    let vars = vars_block_parser(file_id).map(DeckItem::Vars);
    // STORY-088: @var ident = value deck-level inline assignment.
    let at_var = at_var_parser(file_id).map(DeckItem::Vars);
    let set = set_rule_parser(file_id).map(|opt| match opt {
        Some(sr) => DeckItem::Set(sr),
        None => DeckItem::SetErr,
    });
    // STORY-008: variants block
    let variants = variants_block(file_id).map(|(vb, _cycle_err)| DeckItem::Variants(vb));
    // STORY-008: alias declarations
    let alias = alias_decl(file_id).map(|opt| match opt {
        Some(a) => DeckItem::Alias(a),
        None => DeckItem::AliasErr,
    });
    // STORY-008: @include directive
    let include = include_directive_parser(file_id).map(DeckItem::Include);
    // STORY-078: section block (bare-ident form: `section methodology:`)
    let section = section_block_parser(file_id).map(DeckItem::Section);
    // Use block_item() to handle slide, @for, and @if at deck level.
    let block = block_item(file_id).map(DeckItem::Block);
    // STORY-082: section group (quoted-string form: `section "Name":`)
    // block_item(file_id) is passed as the sub-parser so that section bodies
    // can contain the same slide/control-flow productions as the top-level deck.
    let section_group =
        section_group_parser(file_id, block_item(file_id)).map(DeckItem::SectionGroup);

    // Orphan Dedent tokens can appear at deck level when a malformed indented
    // block (e.g. a `@for` with a bad header) leaves its body's closing Dedents
    // in the token stream. We silently consume them here to allow the parser to
    // continue and report subsequent errors independently.
    let orphan_dedent = just(Token::Dedent).ignored();

    let item = orphan_dedent
        .clone()
        .repeated()
        .ignore_then(
            version
                .or(lang)
                .or(brand)
                .or(vars)
                .or(at_var)
                .or(set)
                .or(variants)
                .or(alias)
                .or(include)
                // STORY-082: section_group must come BEFORE section to avoid
                // ambiguity — both start with "section" keyword. The section_group
                // parser matches `section STRING:` (quoted string); the section
                // parser matches `section IDENT:` (bare identifier). They are
                // disambiguated by the second token: STRING vs IDENT. chumsky's
                // or() is ordered — section_group is tried first so that quoted
                // strings don't fall through to section_block_parser.
                .or(section_group)
                .or(section)
                .or(block)
                .recover_with(skip_then_retry_until(
                    any()
                        .filter(|t| !matches!(t, Token::Newline | Token::Eof | Token::Dedent))
                        .ignored(),
                    just(Token::Newline).ignored(),
                )),
        )
        .then_ignore(orphan_dedent.repeated());

    nl.clone()
        .repeated()
        .ignore_then(item.padded_by(nl.repeated()).repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::Eof).or_not())
        .validate(move |items, _info, emitter| {
            let mut deck = DeckNode::default();
            // Alias registry — local to this parse, consumed during expansion.
            let mut alias_reg = AliasRegistry::new();
            // STORY-082: duplicate section-group name detection (AC-011 / EC-011).
            // Tracks section group names seen so far; emits W-PAR-002 on collision.
            let mut seen_section_group_names: std::collections::HashSet<std::sync::Arc<str>> =
                std::collections::HashSet::new();

            for item in items {
                match item {
                    DeckItem::Version(v) => deck.version = Some(v),
                    DeckItem::Lang(l) => deck.lang = Some(l),
                    DeckItem::Brand(b) => deck.brand = Some(b),
                    DeckItem::Vars(vb) => deck.vars.push(vb),
                    DeckItem::Set(sr) => deck.set_rules.push(sr),
                    DeckItem::SetErr | DeckItem::AliasErr => {}, // already reported (E-PAR-007 / E-PAR-006 / E-PAR-011)
                    DeckItem::Variants(vb) => {
                        // Register all variant names for CLI validation (BC-1.07.004).
                        for v in &vb.variants {
                            deck.variant_names.push(v.name.value().clone());
                        }
                        deck.variants = Some(vb);
                    },
                    DeckItem::Alias(a) => {
                        // EC-006: reject alias-of-alias before registering.
                        // Transitive aliasing (alias A = B where B is also an alias)
                        // is not supported in v1.0. Check must happen at registration
                        // time because the base_type parser runs before the AliasRegistry
                        // has visibility into previously registered aliases.
                        if let Some(err_msg) = alias_reg.check_alias_of_alias(a.base_type.value()) {
                            // Emit E-PAR-011 with an alias-span approximation (0-offset).
                            emitter.emit(Rich::custom(
                                SimpleSpan::from(0usize..0usize),
                                format!("{err_msg} (in alias '{}')", a.name.value()),
                            ));
                            // Do NOT register the invalid alias — drop it.
                        } else {
                            // Register the alias for subsequent slide expansion.
                            alias_reg.register(a);
                        }
                    },
                    DeckItem::Include(bi) => {
                        // @include placeholder — passed through for post-parse resolution.
                        deck.items.push(bi);
                    },
                    DeckItem::Section(bi) => {
                        // Section blocks are top-level items; no alias expansion needed.
                        deck.items.push(bi);
                    },
                    DeckItem::SectionGroup(bi) => {
                        // STORY-082 AC-010: discard empty-name SectionGroupNode (sentinel).
                        // E-PAR-023 is always fatal (error-taxonomy.md §24 — "Parse Errors
                        // (E-PAR) — Always fatal. Build halts with accumulated errors. No
                        // output produced."). parse() returns Err when E-PAR-023 is emitted,
                        // so the AST is never used after an empty-named section is encountered.
                        // The sentinel (slides: vec![]) is simply dropped here; no rescue is
                        // needed because no output path survives a fatal parse error.
                        if let crate::ast::BlockItem::SectionGroup(ref spanned) = bi {
                            let group_node = spanned.value();
                            let name = group_node.name.value();
                            if name.is_empty() {
                                // Discard the sentinel. E-PAR-023 already halts the build.
                                continue;
                            }
                            // STORY-082 AC-011: duplicate name detection (W-PAR-002).
                            // Both sections are emitted; warning is cosmetic (exit 0).
                            // HIGH-2 fix: use the real name span so the rendered message
                            // has correct file:line:col (not 0..0 sentinel span).
                            let name_arc: std::sync::Arc<str> = std::sync::Arc::clone(name);
                            if seen_section_group_names.contains(&name_arc) {
                                // The name span from the AST carries the real byte offset.
                                let name_span = group_node.name.span();
                                let name_simple_span =
                                    SimpleSpan::from(name_span.start..name_span.end);
                                emitter.emit(Rich::custom(
                                    name_simple_span,
                                    format!(
                                        "W-PAR-002: Duplicate section group name '{name_arc}'. \
                                         Both sections are emitted with the same GUID. \
                                         Consider using distinct names."
                                    ),
                                ));
                            } else {
                                seen_section_group_names.insert(name_arc);
                            }
                        }
                        deck.items.push(bi);
                    },
                    DeckItem::Block(bi) => {
                        // Expand alias references in slide blocks.
                        let expanded = expand_block_item(bi, &alias_reg);
                        deck.items.push(expanded);
                    },
                }
            }
            deck
        })
}

// ─── Alias expansion helper ───────────────────────────────────────────────────

/// Expand alias references inside a `BlockItem`.
///
/// For `BlockItem::Slide` items whose `kind` matches a registered alias name,
/// the slide is replaced with the expanded `SlideNode` (base type + merged fields).
/// Other `BlockItem` variants are returned unchanged.
///
/// Recursively processes `@for` and `@if` body items.
fn expand_block_item(item: BlockItem, reg: &AliasRegistry) -> BlockItem {
    match item {
        BlockItem::Slide(spanned) => {
            let alias_name = spanned.value().kind.value().clone();
            if reg.contains(&alias_name) {
                // Expand the alias.
                let span = spanned.span();
                let kind_span = spanned.value().kind.span();
                let file_id = kind_span.file_id;
                let explicit_fields = spanned.into_parts().0.fields;
                if let Some(expanded_slide) =
                    reg.expand(&alias_name, explicit_fields, kind_span, file_id)
                {
                    return BlockItem::Slide(Spanned::new(expanded_slide, span));
                }
                // reg.contains() was true but reg.expand() returned None — this is
                // logically unreachable, but we must return something.
                // Return an empty error-recovery slide.
                return BlockItem::Slide(Spanned::new(
                    SlideNode {
                        kind: Spanned::new(alias_name, kind_span),
                        tags: vec![],
                        fields: vec![],
                        inline_items: vec![],
                    },
                    span,
                ));
            }
            // No alias match — return unchanged.
            BlockItem::Slide(spanned)
        },
        BlockItem::For(spanned) => {
            let span = spanned.span();
            let mut for_node = spanned.into_parts().0;
            let expanded_body: Vec<BlockItem> = for_node
                .body
                .drain(..)
                .map(|bi| expand_block_item(bi, reg))
                .collect();
            for_node.body = expanded_body;
            BlockItem::For(Spanned::new(for_node, span))
        },
        BlockItem::If(spanned) => {
            let span = spanned.span();
            let mut if_node = spanned.into_parts().0;
            let expanded_then: Vec<BlockItem> = if_node
                .then_body
                .drain(..)
                .map(|bi| expand_block_item(bi, reg))
                .collect();
            if_node.then_body = expanded_then;
            let expanded_elif: Vec<(_, Vec<BlockItem>)> = if_node
                .elif_branches
                .drain(..)
                .map(|(cond, body)| {
                    (
                        cond,
                        body.into_iter()
                            .map(|bi| expand_block_item(bi, reg))
                            .collect(),
                    )
                })
                .collect();
            if_node.elif_branches = expanded_elif;
            let expanded_else = if_node.else_body.map(|body| {
                body.into_iter()
                    .map(|bi| expand_block_item(bi, reg))
                    .collect()
            });
            if_node.else_body = expanded_else;
            BlockItem::If(Spanned::new(if_node, span))
        },
        // Section (bare-ident form, STORY-078) and SectionGroup (quoted-string
        // form, STORY-082) are returned unchanged — neither contains slides
        // that require alias expansion. SectionGroup slide children are
        // expanded during the STORY-082 eval pass.
        BlockItem::Section(_) | BlockItem::SectionGroup(_) => item,
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{lexer::lex, span::SourceMap};
    use std::sync::Arc;

    /// Helper: lex the given source and run the deck parser.
    ///
    /// Returns `(deck_option, lex_errors, parse_error_count)`.
    fn parse_src(src: &str) -> (Option<DeckNode>, Vec<crate::lexer_error::LexError>, usize) {
        let file: Arc<str> = Arc::from("test.sf");
        let (tokens, lex_errs) = lex(src, file.clone());
        let eoi = SimpleSpan::from(src.len()..src.len());

        let mut sm = SourceMap::new();
        let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));

        // Convert lexer spans to SimpleSpan and build owned (Token, SimpleSpan) vec.
        let spanned_tokens: Vec<(Token, SimpleSpan)> = tokens
            .into_iter()
            .map(|(t, s)| (t, SimpleSpan::from(s)))
            .collect();
        let input = spanned_tokens
            .as_slice()
            .map(eoi, |(t, s): &(Token, SimpleSpan)| (t, s));

        let (deck, parse_errs) = deck_parser(file_id).parse(input).into_output_errors();
        (deck, lex_errs, parse_errs.len())
    }

    // ── AC-001: minimal deck parses with 0 errors ────────────────────────────

    #[test]
    fn test_bc_1_01_001_minimal_deck_parses_ok() {
        let src = concat!(
            "slideforge_version \"1\"\n",
            "lang \"en-US\"\n",
            "slide title:\n",
            "  title \"Hello\"\n",
        );
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(
            lex_errs.is_empty(),
            "lex errors must be empty: {lex_errs:?}"
        );
        assert_eq!(parse_err_count, 0, "parse errors must be 0");
        let deck = deck.expect("deck must parse successfully");
        assert_eq!(
            deck.items.len(),
            1,
            "should have exactly 1 block item (slide)"
        );
        assert_eq!(deck.version.as_ref().map(|v| v.value().as_str()), Some("1"));
        assert_eq!(
            deck.lang.as_ref().map(|l| l.value().as_str()),
            Some("en-US")
        );
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
        let (deck1, _, _) = parse_src(src);
        let (deck2, _, _) = parse_src(src);
        assert_eq!(
            deck1, deck2,
            "parsing the same source twice must produce the same AST"
        );
    }

    // ── AC-009: empty file does not panic ─────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_empty_file_no_panic() {
        let (deck, _lex_errs, _parse_errs) = parse_src("");
        // Either Ok with empty items or None — must not panic.
        if let Some(d) = deck {
            assert!(
                d.items.is_empty(),
                "empty source must produce 0 block items"
            );
        }
    }

    // ── AC-010: slide fields captured ────────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_slide_fields_captured() {
        use crate::ast::BlockItem;
        use crate::template::TemplateChunk;
        let src = concat!(
            "slide content:\n",
            "  title \"Hello\"\n",
            "  footer \"Slide 1\"\n",
        );
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "lex errors: {lex_errs:?}");
        assert_eq!(parse_err_count, 0, "parse errors must be 0");
        let deck = deck.expect("must parse");
        assert_eq!(deck.items.len(), 1);
        let BlockItem::Slide(slide_s) = &deck.items[0] else {
            panic!("expected Slide item");
        };
        let slide = slide_s.value();
        assert_eq!(slide.fields.len(), 2, "slide must have 2 fields");
        let title_field = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "title")
            .expect("title field must exist");
        assert_eq!(
            title_field.value.value(),
            &FieldValue::Template(vec![TemplateChunk::Literal("Hello".to_string())])
        );
        let footer_field = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "footer")
            .expect("footer field must exist");
        assert_eq!(
            footer_field.value.value(),
            &FieldValue::Template(vec![TemplateChunk::Literal("Slide 1".to_string())])
        );
    }

    // ── AC-002: vars block parsed ─────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_vars_block_parsed() {
        use crate::template::TemplateChunk;
        // vars entries use IDENT ":" value syntax per the grammar spec.
        let src = concat!(
            "vars:\n",
            "  client: \"Acme\"\n",
            "slide title:\n",
            "  title \"Test\"\n",
        );
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "lex errors: {lex_errs:?}");
        assert_eq!(parse_err_count, 0, "parse errors must be 0");
        let deck = deck.expect("must parse");
        assert_eq!(deck.vars.len(), 1, "should have 1 vars block");
        let vb = &deck.vars[0];
        assert_eq!(vb.entries.len(), 1, "should have 1 entry");
        assert_eq!(vb.entries[0].0.value(), "client");
        assert_eq!(
            vb.entries[0].1.value(),
            &FieldValue::Template(vec![TemplateChunk::Literal("Acme".to_string())])
        );
    }

    // ── AC-002: set rule parsed ────────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_set_rule_parsed() {
        use crate::ast::SetRuleValue;
        use crate::template::TemplateChunk;
        let src = concat!(
            "set content: footer \"Default\"\n",
            "slide content:\n",
            "  title \"Test\"\n",
        );
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "lex errors: {lex_errs:?}");
        assert_eq!(parse_err_count, 0, "parse errors must be 0");
        let deck = deck.expect("must parse");
        assert_eq!(deck.set_rules.len(), 1, "should have 1 set rule");
        let sr = &deck.set_rules[0];
        assert_eq!(sr.slide_type.value(), "content");
        assert_eq!(sr.field.value(), "footer");
        assert_eq!(
            sr.value.value(),
            &SetRuleValue::Template(vec![TemplateChunk::Literal("Default".to_string())])
        );
    }

    // ── AC-002: all deck metadata preserved ──────────────────────────────────

    #[test]
    fn test_bc_1_01_001_deck_metadata_preserved() {
        let src = concat!(
            "slideforge_version \"1\"\n",
            "lang \"en-US\"\n",
            "brand \"my-brand.pptx\"\n",
            "slide title:\n",
            "  title \"Test\"\n",
        );
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "lex errors: {lex_errs:?}");
        assert_eq!(parse_err_count, 0, "parse errors must be 0");
        let deck = deck.expect("must parse");
        assert_eq!(deck.version.as_ref().map(|v| v.value().as_str()), Some("1"));
        assert_eq!(
            deck.lang.as_ref().map(|l| l.value().as_str()),
            Some("en-US")
        );
        assert_eq!(
            deck.brand.as_ref().map(|b| b.value().as_str()),
            Some("my-brand.pptx")
        );
    }

    // ── AC-008: parse returns Err on error ────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_parse_returns_err_on_lex_error() {
        // A tab-indented line produces a lex error.
        let src = "\tfield: value\n";
        let (_deck, lex_errs, _parse_errs) = parse_src(src);
        assert!(
            !lex_errs.is_empty(),
            "tab indentation must produce a lex error"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STORY-088 RED GATE TESTS
    // Traces to BC-1.01.002 (field-value parser — list-literal extension).
    // All tests below MUST FAIL before implementation starts.
    // ═══════════════════════════════════════════════════════════════════════

    // ── AC-001: bullets: ["A","B","C"] parses to FieldValue::List ────────────

    /// BC-1.01.002 AC-001 — `bullets: ["Item A", "Item B", "Item C"]` parses
    /// without error and the field value is `FieldValue::List(Vec<FieldValue>)`
    /// where each inner item is a `FieldValue::Template`.
    ///
    /// RED GATE: `value_parser()` has no `[...]` arm → parser emits error(s) and
    /// produces `FieldValue::Error` instead of `FieldValue::List` → assertion FAILS.
    #[test]
    fn test_bc_1_01_002_ac001_bullets_field_list_literal_parses_to_field_value_list() {
        // Slide body syntax: `name value` without colon (same as `title "My Slide"`).
        let src = concat!(
            "slide content:\n",
            "  title \"My Slide\"\n",
            "  bullets [\"Item A\", \"Item B\", \"Item C\"]\n",
        );
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(
            lex_errs.is_empty(),
            "AC-001: no lex errors expected; got: {lex_errs:?}"
        );
        assert_eq!(
            parse_err_count, 0,
            "AC-001: no parse errors expected for valid list literal; got {parse_err_count} errors"
        );

        let deck = deck.expect("AC-001: deck must parse successfully");
        assert_eq!(deck.items.len(), 1, "AC-001: deck must have 1 slide");

        let crate::ast::BlockItem::Slide(slide_spanned) = &deck.items[0] else {
            panic!("AC-001: expected BlockItem::Slide");
        };
        let slide = slide_spanned.value();

        let bullets_field = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "bullets")
            .expect("AC-001: 'bullets' field must be present");

        // The value MUST be FieldValue::List with 3 items.
        // RED GATE: without FieldValue::List parser arm this will be FieldValue::Error.
        let FieldValue::List(items) = bullets_field.value.value() else {
            panic!(
                "AC-001 RED GATE: bullets field must be FieldValue::List; got: {:?}. \
                 value_parser() has no [...]  arm yet — implement FieldValue::List arm \
                 in deck.rs to pass this test.",
                bullets_field.value.value()
            );
        };
        assert_eq!(items.len(), 3, "AC-001: list must have 3 items");

        // Each item must be a FieldValue::Template wrapping a single Literal chunk.
        for (i, item) in items.iter().enumerate() {
            let FieldValue::Template(chunks) = item else {
                panic!("AC-001: list item[{i}] must be FieldValue::Template; got: {item:?}");
            };
            assert_eq!(chunks.len(), 1, "AC-001: each item must have 1 chunk");
        }
    }

    // ── AC-002: empty list `bullets: []` parses to FieldValue::List([]) ──────

    /// BC-1.01.002 AC-002 — `bullets: []` (empty list) parses to
    /// `FieldValue::List(vec![])` without error.
    ///
    /// RED GATE: `value_parser()` has no `[...]` arm → parse error, no List produced.
    #[test]
    fn test_bc_1_01_002_ac002_empty_list_literal_parses_to_field_value_list_empty() {
        let src = concat!("slide content:\n", "  bullets []\n",);
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "AC-002: no lex errors expected");
        assert_eq!(
            parse_err_count, 0,
            "AC-002: empty list literal must parse without error; got {parse_err_count} errors"
        );

        let deck = deck.expect("AC-002: deck must parse");
        let crate::ast::BlockItem::Slide(slide_spanned) = &deck.items[0] else {
            panic!("AC-002: expected Slide");
        };
        let bullets_field = slide_spanned
            .value()
            .fields
            .iter()
            .find(|f| f.name.value() == "bullets")
            .expect("AC-002: 'bullets' field must exist");

        let FieldValue::List(items) = bullets_field.value.value() else {
            panic!(
                "AC-002 RED GATE: empty [] must produce FieldValue::List([]); got: {:?}",
                bullets_field.value.value()
            );
        };
        assert!(
            items.is_empty(),
            "AC-002: FieldValue::List from [] must be empty; got {items:?}"
        );
    }

    // ── AC-003: single-item list `bullets: ["Only"]` ──────────────────────────

    /// BC-1.01.002 AC-003 — `bullets: ["Only"]` parses to
    /// `FieldValue::List(vec![FieldValue::Template(...)])`.
    /// A single-item list must NOT be treated as a bare string.
    ///
    /// RED GATE: `value_parser()` has no `[...]` arm → parse error, no List produced.
    #[test]
    fn test_bc_1_01_002_ac003_single_item_list_parses_as_list_not_bare_string() {
        let src = concat!("slide content:\n", "  bullets [\"Only\"]\n",);
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "AC-003: no lex errors expected");
        assert_eq!(
            parse_err_count, 0,
            "AC-003: single-item list must parse without error; got {parse_err_count} errors"
        );

        let deck = deck.expect("AC-003: deck must parse");
        let crate::ast::BlockItem::Slide(slide_spanned) = &deck.items[0] else {
            panic!("AC-003: expected Slide");
        };
        let bullets_field = slide_spanned
            .value()
            .fields
            .iter()
            .find(|f| f.name.value() == "bullets")
            .expect("AC-003: 'bullets' field must exist");

        let FieldValue::List(items) = bullets_field.value.value() else {
            panic!(
                "AC-003 RED GATE: single-item [\"Only\"] must produce FieldValue::List; \
                 NOT FieldValue::Ident(\"Only\") or FieldValue::Template. Got: {:?}",
                bullets_field.value.value()
            );
        };
        assert_eq!(
            items.len(),
            1,
            "AC-003: single-item list must have exactly 1 item"
        );
    }

    // ── AC-004: list-literal in full slide block (indentation integration) ────

    /// BC-1.01.002 AC-004 — list-literal parses inside a standard slide block
    /// with multiple fields; no off-by-one indentation errors.
    ///
    /// RED GATE: `value_parser()` has no `[...]` arm → the `bullets:` field fails to
    /// parse and may corrupt subsequent field parsing.
    #[test]
    fn test_bc_1_01_002_ac004_list_literal_in_full_slide_block_with_multiple_fields() {
        let src = concat!(
            "slide content:\n",
            "  title \"Agenda\"\n",
            "  bullets [\"Step 1\", \"Step 2\", \"Step 3\"]\n",
            "  body \"See notes.\"\n",
        );
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "AC-004: no lex errors expected");
        assert_eq!(
            parse_err_count, 0,
            "AC-004: all 4 fields must parse without error; got {parse_err_count} errors. \
             Check that the list-literal arm does not break indentation tracking or corrupt \
             subsequent field parsing."
        );

        let deck = deck.expect("AC-004: deck must parse");
        let crate::ast::BlockItem::Slide(slide_spanned) = &deck.items[0] else {
            panic!("AC-004: expected Slide");
        };
        let slide = slide_spanned.value();

        assert_eq!(slide.fields.len(), 3, "AC-004: slide must have 3 fields");

        let bullets_field = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "bullets")
            .expect("AC-004: 'bullets' field must exist");

        let FieldValue::List(items) = bullets_field.value.value() else {
            panic!(
                "AC-004 RED GATE: bullets field must be FieldValue::List; got: {:?}",
                bullets_field.value.value()
            );
        };
        assert_eq!(items.len(), 3, "AC-004: list must have 3 items");

        // Verify the other fields parsed correctly (no partial-parse contamination).
        let title_field = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "title")
            .expect("AC-004: 'title' field must be present after list-literal field");
        assert!(
            matches!(title_field.value.value(), FieldValue::Template(_)),
            "AC-004: title field must be FieldValue::Template, not contaminated; \
             got: {:?}",
            title_field.value.value()
        );
    }

    // ── AC-005: non-string list items produce E-PAR diagnostic, not panic ─────

    /// BC-1.01.002 AC-005 / BC-1.15.001 (error accumulation) — `bullets: [42, true]`
    /// produces a parser diagnostic, NOT a panic. Error accumulation: the parser
    /// continues and reports all errors in the file.
    ///
    /// RED GATE: `value_parser()` has no `[...]` arm at all → the error we get may be
    /// different (the `[` token is unexpected) but the test still requires that:
    /// (a) no panic occurs, and (b) parse errors are non-zero.
    ///
    /// POST-IMPLEMENTATION expectation: specific E-PAR diagnostic for non-string items.
    #[test]
    fn test_bc_1_01_002_ac005_non_string_list_items_produce_epar_not_panic() {
        // This must NOT panic — error accumulation must continue after the bad items.
        let src = concat!("slide content:\n", "  bullets [42, true]\n",);
        let (_deck, lex_errs, parse_err_count) = parse_src(src);
        // No lex errors (the tokens are valid; the semantic error is at parse level).
        assert!(
            lex_errs.is_empty(),
            "AC-005: [42, true] must not produce lex errors (tokens are valid)"
        );
        // Parser MUST emit at least one error (non-string items in bullet list).
        // This also guards against the "list parsed silently as string" regression.
        assert!(
            parse_err_count > 0,
            "AC-005 RED GATE: non-string list items [42, true] must produce ≥1 parser \
             diagnostic. Got 0 errors — this means the items were silently accepted, \
             which violates the type-validation rule."
        );
        // No panic occurred (test reaches this point without unwinding).
    }

    // ── AC-006: vars-block list assignment form ───────────────────────────────

    /// BC-1.01.002 AC-006 — `vars: items: ["A","B","C"]` (vars-block list assignment)
    /// parses without error. The vars-block value parser must accept `[...]` tokens.
    ///
    /// RED GATE: `value_parser()` has no `[...]` arm → parse error; the vars-block
    /// entry fails to parse → `deck.vars[0].entries[0].1` is NOT `FieldValue::List`.
    ///
    /// Note: This tests the `vars:` block form (with `:` separator and indented
    /// block body). The `@var ident = [...]` form (at deck-level) is tested
    /// separately in the E2E tests (AC-007 fixture).
    #[test]
    fn test_bc_1_01_002_ac006_vars_block_list_assignment_parses_to_fieldvalue_list() {
        let src = concat!(
            "vars:\n",
            "  items: [\"Item A\", \"Item B\", \"Item C\"]\n",
            "slide content:\n",
            "  title \"My Slide\"\n",
            "  bullets: items\n",
        );
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "AC-006: no lex errors expected");
        assert_eq!(
            parse_err_count, 0,
            "AC-006: vars-block list assignment must parse without error; got {parse_err_count}"
        );

        let deck = deck.expect("AC-006: deck must parse");
        assert_eq!(deck.vars.len(), 1, "AC-006: must have 1 vars block");
        let vb = &deck.vars[0];
        assert_eq!(vb.entries.len(), 1, "AC-006: vars block must have 1 entry");

        let (_name, value_spanned) = &vb.entries[0];
        let FieldValue::List(items) = value_spanned.value() else {
            panic!(
                "AC-006 RED GATE: vars-block list value must be FieldValue::List; got: {:?}. \
                 value_parser() (used in vars_block_parser) has no [...]  arm yet.",
                value_spanned.value()
            );
        };
        assert_eq!(items.len(), 3, "AC-006: list must have 3 items");
    }

    /// BC-1.01.002 AC-006 regression guard — `bullets items` (ident reference,
    /// NO colon) continues to parse correctly after the list-literal arm is added.
    ///
    /// Note: the slide body field syntax uses `name value` without colon (e.g.
    /// `title "My Slide"`). The vars-block syntax uses `name: value` with colon.
    /// This test exercises the slide-body `bullets items` (ident reference)
    /// path that currently works (`FieldValue::Ident`) to guard against
    /// regressions introduced by the list-literal arm addition.
    ///
    /// GREEN GATE: This test SHOULD PASS today (the ident path already works).
    /// After implementation it must also pass (regression guard).
    #[test]
    fn test_bc_1_01_002_ac006_regression_bullets_ident_reference_still_parses() {
        // Slide body syntax: `name value` without colon.
        let src = concat!(
            "vars:\n",
            "  items: \"placeholder\"\n",
            "slide content:\n",
            "  title \"Agenda\"\n",
            "  bullets items\n",
        );
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "AC-006 regression: no lex errors");
        assert_eq!(
            parse_err_count, 0,
            "AC-006 regression: 'bullets items' (ident reference, no colon) must still parse; \
             got {parse_err_count} errors. Adding the list-literal arm must NOT break the \
             existing FieldValue::Ident path for slide body fields."
        );

        let deck = deck.expect("AC-006 regression: deck must parse");
        let crate::ast::BlockItem::Slide(slide_spanned) = &deck.items[0] else {
            panic!("AC-006 regression: expected Slide");
        };
        let bullets_field = slide_spanned
            .value()
            .fields
            .iter()
            .find(|f| f.name.value() == "bullets")
            .expect("AC-006 regression: 'bullets' field must exist");

        assert!(
            matches!(bullets_field.value.value(), FieldValue::Ident(_)),
            "AC-006 regression: 'bullets items' must produce FieldValue::Ident; \
             got: {:?}",
            bullets_field.value.value()
        );
    }

    // ── EC-001: empty list ────────────────────────────────────────────────────

    /// BC-1.01.002 EC-001 — `bullets: []` edge case: empty list is valid.
    /// Covered by AC-002 above; this explicit edge-case test verifies the
    /// empty-list → `FieldValue::List(vec![])` path directly.
    ///
    /// RED GATE: `value_parser()` has no `[...]` arm → parse error.
    #[test]
    fn test_bc_1_01_002_ec001_empty_list_produces_field_value_list_empty() {
        let src = concat!("slide content:\n", "  bullets []\n");
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "EC-001: no lex errors");
        assert_eq!(
            parse_err_count, 0,
            "EC-001: empty list [] is valid; must produce 0 parse errors"
        );
        let deck = deck.expect("EC-001: must parse");
        let crate::ast::BlockItem::Slide(s) = &deck.items[0] else {
            panic!("EC-001: expected Slide");
        };
        let f = s
            .value()
            .fields
            .iter()
            .find(|f| f.name.value() == "bullets")
            .expect("EC-001: bullets field");
        let FieldValue::List(items) = f.value.value() else {
            panic!(
                "EC-001 RED GATE: [] must be FieldValue::List([]); got: {:?}",
                f.value.value()
            );
        };
        assert!(
            items.is_empty(),
            "EC-001: FieldValue::List from [] must be empty"
        );
    }

    // ── EC-003: trailing comma ────────────────────────────────────────────────

    /// BC-1.01.002 EC-003 — `bullets: ["A","B",]` trailing comma.
    /// The parser must either accept the trailing comma (preferred via
    /// `allow_trailing()`) or produce a useful diagnostic — NOT a silent wrong parse.
    ///
    /// RED GATE: `value_parser()` has no `[...]` arm → parse error (different error,
    /// but test still verifies no panic and no silent wrong value).
    #[test]
    fn test_bc_1_01_002_ec003_trailing_comma_does_not_panic() {
        let src = concat!("slide content:\n", "  bullets [\"A\", \"B\",]\n",);
        // Must not panic regardless of parse result.
        let (deck_opt, lex_errs, _parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "EC-003: no lex errors expected");

        // If parse succeeds, the trailing comma must be silently accepted (allow_trailing).
        if let Some(deck) = deck_opt {
            let crate::ast::BlockItem::Slide(s) = &deck.items[0] else {
                return; // parse recovered to empty slide — acceptable
            };
            if let Some(f) = s
                .value()
                .fields
                .iter()
                .find(|f| f.name.value() == "bullets")
            {
                // If list parsed, must have 2 items (trailing comma consumed, not counted).
                if let FieldValue::List(items) = f.value.value() {
                    assert_eq!(
                        items.len(),
                        2,
                        "EC-003: trailing comma must not add an extra empty item; \
                         got {items:?}"
                    );
                }
            }
        }
        // No panic = test passes (primary assertion for EC-003 at Red Gate).
    }

    // ── EC-004: empty-string item ─────────────────────────────────────────────

    /// BC-1.01.002 EC-004 — `bullets: [""]` single empty-string item.
    /// Must parse to `FieldValue::List(vec![FieldValue::Template(vec![])])`.
    ///
    /// RED GATE: `value_parser()` has no `[...]` arm → parse error.
    #[test]
    fn test_bc_1_01_002_ec004_empty_string_item_is_valid() {
        let src = concat!("slide content:\n", "  bullets [\"\"]\n",);
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "EC-004: no lex errors");
        assert_eq!(
            parse_err_count, 0,
            "EC-004: empty-string item [\"\"] must parse without error; \
             got {parse_err_count} errors"
        );
        let deck = deck.expect("EC-004: must parse");
        let crate::ast::BlockItem::Slide(s) = &deck.items[0] else {
            panic!("EC-004: expected Slide");
        };
        let f = s
            .value()
            .fields
            .iter()
            .find(|f| f.name.value() == "bullets")
            .expect("EC-004: bullets field");
        let FieldValue::List(items) = f.value.value() else {
            panic!(
                "EC-004 RED GATE: [\"\"] must produce FieldValue::List; got: {:?}",
                f.value.value()
            );
        };
        assert_eq!(
            items.len(),
            1,
            "EC-004: single-item list from [\"\"] must have 1 item"
        );
        assert!(
            matches!(items[0], FieldValue::Template(_)),
            "EC-004: empty-string item must be FieldValue::Template; got: {:?}",
            items[0]
        );
    }

    // ── EC-005: mixed types — all errors collected ────────────────────────────

    /// BC-1.01.002 EC-005 / BC-1.15.001 (error accumulation) —
    /// `bullets: ["A", 42, "C"]` (mixed types): all errors are collected.
    /// The parser must NOT fail-on-first — it must report the error for `42` AND
    /// continue to parse `"C"`.
    ///
    /// RED GATE: `value_parser()` has no `[...]` arm → parse error on the `[` token
    /// itself, not specifically on the `42` item. The test verifies ≥1 parse error.
    #[test]
    fn test_bc_1_01_002_ec005_mixed_type_list_all_errors_collected() {
        let src = concat!("slide content:\n", "  bullets [\"A\", 42, \"C\"]\n",);
        let (_deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "EC-005: no lex errors");
        // Must produce at least one parse error (for the `42` non-string item).
        assert!(
            parse_err_count > 0,
            "EC-005 RED GATE: mixed-type list [\"A\", 42, \"C\"] must produce ≥1 parse error \
             for the non-string item '42'. Got 0 errors — the item was silently accepted."
        );
        // The test does NOT assert the number of errors precisely because the
        // exact error count depends on recovery strategy. The invariant is ≥1.
    }

    // ── Invariant: FieldValue::List content assertion ─────────────────────────

    /// BC-1.01.002 invariant — a parsed `FieldValue::List` contains exactly
    /// the string items written in the source (LESSON-14: assert content, not
    /// mere presence).
    ///
    /// RED GATE: `value_parser()` has no `[...]` arm → `FieldValue::List` not produced.
    #[test]
    fn test_bc_1_01_002_invariant_parsed_list_contains_exact_string_content() {
        let src = concat!(
            "slide content:\n",
            "  bullets [\"Alpha\", \"Beta\", \"Gamma\"]\n",
        );
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "invariant: no lex errors");
        assert_eq!(
            parse_err_count, 0,
            "invariant: list with 3 strings must parse without error"
        );

        let deck = deck.expect("invariant: deck must parse");
        let crate::ast::BlockItem::Slide(slide_spanned) = &deck.items[0] else {
            panic!("invariant: expected Slide");
        };
        let bullets_field = slide_spanned
            .value()
            .fields
            .iter()
            .find(|f| f.name.value() == "bullets")
            .expect("invariant: bullets field");

        let FieldValue::List(items) = bullets_field.value.value() else {
            panic!(
                "invariant RED GATE: bullets must be FieldValue::List; got: {:?}",
                bullets_field.value.value()
            );
        };

        // LESSON-14: assert the CONTENT, not just the structure.
        assert_eq!(items.len(), 3, "invariant: list must have exactly 3 items");

        // Extract the literal text from each item's Template chunks.
        let texts: Vec<String> = items
            .iter()
            .map(|item| {
                let FieldValue::Template(chunks) = item else {
                    panic!("invariant: each list item must be FieldValue::Template; got: {item:?}");
                };
                let crate::template::TemplateChunk::Literal(s) = &chunks[0] else {
                    panic!("invariant: each item chunk must be Literal; got: {chunks:?}");
                };
                s.clone()
            })
            .collect();

        assert_eq!(texts[0], "Alpha", "invariant: first item must be 'Alpha'");
        assert_eq!(texts[1], "Beta", "invariant: second item must be 'Beta'");
        assert_eq!(texts[2], "Gamma", "invariant: third item must be 'Gamma'");
    }

    // ── HIGH-1: bullets: [...] WITH COLON (spec-canonical form) ─────────────

    /// BC-1.01.002 HIGH-1 — `bullets: ["A","B","C"]` (WITH colon separator, the
    /// spec-canonical form) parses to `FieldValue::List` with 3 items and 0 errors.
    ///
    /// The DSL spec and AC-007 specify the colon form as canonical. This test
    /// ensures the colon form (not just the no-colon form) is covered by unit tests.
    ///
    /// Both colon and no-colon forms produce identical AST nodes; both are accepted.
    #[test]
    fn test_bc_1_01_002_high1_bullets_colon_form_parses_to_field_value_list() {
        // Spec-canonical form: `bullets: ["A","B","C"]` (WITH colon).
        let src = concat!(
            "slide content:\n",
            "  title \"My Slide\"\n",
            "  bullets: [\"A\", \"B\", \"C\"]\n",
        );
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(
            lex_errs.is_empty(),
            "HIGH-1: no lex errors expected for colon form; got: {lex_errs:?}"
        );
        assert_eq!(
            parse_err_count, 0,
            "HIGH-1: bullets: [...] (WITH colon) must produce 0 parse errors; \
             got {parse_err_count} errors"
        );

        let deck = deck.expect("HIGH-1: deck must parse successfully");
        let crate::ast::BlockItem::Slide(slide_spanned) = &deck.items[0] else {
            panic!("HIGH-1: expected BlockItem::Slide");
        };
        let bullets_field = slide_spanned
            .value()
            .fields
            .iter()
            .find(|f| f.name.value() == "bullets")
            .expect("HIGH-1: 'bullets' field must be present");

        let FieldValue::List(items) = bullets_field.value.value() else {
            panic!(
                "HIGH-1: bullets: [...] (WITH colon) must produce FieldValue::List; \
                 got: {:?}",
                bullets_field.value.value()
            );
        };
        assert_eq!(
            items.len(),
            3,
            "HIGH-1: colon-form list must have 3 items; got: {items:?}"
        );
        // Each item must be a FieldValue::Template wrapping a single Literal chunk.
        for (i, item) in items.iter().enumerate() {
            assert!(
                matches!(item, FieldValue::Template(_)),
                "HIGH-1: item[{i}] must be FieldValue::Template; got: {item:?}"
            );
        }
    }

    /// BC-1.01.002 HIGH-1 regression — the no-colon form `bullets ["A","B","C"]` must
    /// still work after the colon-form acceptance is confirmed.
    ///
    /// Both forms produce identical AST nodes. This test is the regression guard
    /// ensuring the no-colon path is not broken when the colon path is added.
    #[test]
    fn test_bc_1_01_002_high1_bullets_no_colon_form_still_parses() {
        // No-colon form: `bullets ["A","B","C"]` (without colon).
        let src = concat!(
            "slide content:\n",
            "  title \"My Slide\"\n",
            "  bullets [\"A\", \"B\", \"C\"]\n",
        );
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(
            lex_errs.is_empty(),
            "HIGH-1 no-colon: no lex errors expected"
        );
        assert_eq!(
            parse_err_count, 0,
            "HIGH-1 no-colon: bullets [...] (WITHOUT colon) must still parse; \
             got {parse_err_count} errors"
        );
        let deck = deck.expect("HIGH-1 no-colon: deck must parse");
        let crate::ast::BlockItem::Slide(slide_spanned) = &deck.items[0] else {
            panic!("HIGH-1 no-colon: expected BlockItem::Slide");
        };
        let bullets_field = slide_spanned
            .value()
            .fields
            .iter()
            .find(|f| f.name.value() == "bullets")
            .expect("HIGH-1 no-colon: 'bullets' field must be present");
        assert!(
            matches!(bullets_field.value.value(), FieldValue::List(_)),
            "HIGH-1 no-colon: bullets [...] must still produce FieldValue::List; \
             got: {:?}",
            bullets_field.value.value()
        );
    }

    // ── HIGH-2: @var ident = [...] unit tests ────────────────────────────────

    /// BC-1.01.002 HIGH-2 (a) — `@var items = ["A","B","C"]` produces a
    /// `VarsBlock` entry whose value is `FieldValue::List` with 3 items and 0 errors.
    ///
    /// This is the DIRECT unit test for `at_var_parser`. SID-1: a non-ignored
    /// behavior must be driven by a unit test, not only by cross-crate E2E.
    #[test]
    fn test_bc_1_01_002_high2a_at_var_list_assignment_produces_fieldvalue_list() {
        let src = concat!(
            "@var items = [\"A\", \"B\", \"C\"]\n",
            "slide content:\n",
            "  title \"My Slide\"\n",
        );
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(
            lex_errs.is_empty(),
            "HIGH-2a: no lex errors expected; got: {lex_errs:?}"
        );
        assert_eq!(
            parse_err_count, 0,
            "HIGH-2a: @var items = [...] must produce 0 parse errors; \
             got {parse_err_count} errors"
        );

        let deck = deck.expect("HIGH-2a: deck must parse successfully");
        assert_eq!(
            deck.vars.len(),
            1,
            "HIGH-2a: deck must have 1 vars block (from @var)"
        );
        let vb = &deck.vars[0];
        assert_eq!(vb.entries.len(), 1, "HIGH-2a: vars block must have 1 entry");

        let (_name, value_spanned) = &vb.entries[0];
        let FieldValue::List(items) = value_spanned.value() else {
            panic!(
                "HIGH-2a: @var items = [...] value must be FieldValue::List; got: {:?}",
                value_spanned.value()
            );
        };
        assert_eq!(
            items.len(),
            3,
            "HIGH-2a: @var list must have 3 items; got: {items:?}"
        );
        for (i, item) in items.iter().enumerate() {
            assert!(
                matches!(item, FieldValue::Template(_)),
                "HIGH-2a: item[{i}] must be FieldValue::Template; got: {item:?}"
            );
        }
    }

    /// BC-1.01.002 HIGH-2 (b) — `@var content = ["x"]` where `content` is a
    /// reserved slide-type keyword produces E-PAR-008.
    ///
    /// This is the DIRECT unit test for the keyword-collision branch of
    /// `at_var_parser`. SID-1: the collision check is a non-ignored behavior
    /// that must be driven by a unit test.
    #[test]
    fn test_bc_1_01_002_high2b_at_var_keyword_collision_emits_e_par_008() {
        // `content` is a reserved slide-type keyword — @var content = [...] must
        // emit E-PAR-008 (VarNameCollision).
        let src = "@var content = [\"x\"]\n";
        let file: std::sync::Arc<str> = std::sync::Arc::from("test.sf");
        let (tokens, lex_errs) = crate::lexer::lex(src, file.clone());
        assert!(lex_errs.is_empty(), "HIGH-2b: no lex errors expected");
        let eoi = SimpleSpan::from(src.len()..src.len());
        let mut sm = SourceMap::new();
        let file_id = sm.add_file(std::sync::Arc::from("test.sf"), std::sync::Arc::from(src));
        let spanned_tokens: Vec<(Token, SimpleSpan)> = tokens
            .into_iter()
            .map(|(t, s)| (t, SimpleSpan::from(s)))
            .collect();
        let input = spanned_tokens
            .as_slice()
            .map(eoi, |(t, s): &(Token, SimpleSpan)| (t, s));
        let (_deck_opt, parse_errs) = deck_parser(file_id).parse(input).into_output_errors();
        assert!(
            !parse_errs.is_empty(),
            "HIGH-2b: @var content = [...] must emit E-PAR-008 (content is a slide-type keyword); \
             got 0 parse errors"
        );
        // Rich::custom reasons are accessible via format!("{:?}", e.reason()).
        let has_e_par_008 = parse_errs
            .iter()
            .any(|e| format!("{:?}", e.reason()).contains("E-PAR-008"));
        assert!(
            has_e_par_008,
            "HIGH-2b: @var content = [...] must emit E-PAR-008 for keyword collision; \
             got errors: {parse_errs:?}"
        );
    }

    // ── MED-1: AC-005/EC-005 strengthened to assert E-PAR-024 code ──────────

    /// BC-1.01.002 MED-1 / AC-005 strengthened — `bullets [42, true]` produces
    /// a diagnostic containing the code `E-PAR-024` (LESSON-14 load-bearing).
    ///
    /// This test is the load-bearing form of AC-005: it asserts the SPECIFIC error
    /// code emitted, not merely that errors > 0. If the emitted code changes from
    /// E-PAR-024 to something else, this test will fail.
    #[test]
    fn test_bc_1_01_002_med1_non_string_list_items_emit_e_par_024_code() {
        let src = concat!("slide content:\n", "  bullets [42, true]\n");
        let file: Arc<str> = Arc::from("test.sf");
        let (tokens, lex_errs) = lex(src, file.clone());
        assert!(lex_errs.is_empty(), "MED-1: no lex errors expected");
        let eoi = SimpleSpan::from(src.len()..src.len());
        let mut sm = SourceMap::new();
        let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
        let spanned_tokens: Vec<(Token, SimpleSpan)> = tokens
            .into_iter()
            .map(|(t, s)| (t, SimpleSpan::from(s)))
            .collect();
        let input = spanned_tokens
            .as_slice()
            .map(eoi, |(t, s): &(Token, SimpleSpan)| (t, s));
        let (_deck_opt, parse_errs) = deck_parser(file_id).parse(input).into_output_errors();

        assert!(
            !parse_errs.is_empty(),
            "MED-1: non-string list items [42, true] must produce ≥1 parse error; got 0"
        );
        // Rich::custom reasons are accessible via format!("{:?}", e.reason()).
        let has_e_par_024 = parse_errs
            .iter()
            .any(|e| format!("{:?}", e.reason()).contains("E-PAR-024"));
        assert!(
            has_e_par_024,
            "MED-1: at least one error must contain 'E-PAR-024' for non-string list items; \
             got: {parse_errs:?}"
        );
    }

    /// BC-1.01.002 MED-1 / EC-005 strengthened — `bullets ["A", 42, "C"]`
    /// (mixed types) must include `E-PAR-024` in the emitted diagnostics.
    #[test]
    fn test_bc_1_01_002_med1_mixed_type_list_emits_e_par_024_code() {
        let src = concat!("slide content:\n", "  bullets [\"A\", 42, \"C\"]\n");
        let file: Arc<str> = Arc::from("test.sf");
        let (tokens, lex_errs) = lex(src, file.clone());
        assert!(lex_errs.is_empty(), "MED-1 EC-005: no lex errors expected");
        let eoi = SimpleSpan::from(src.len()..src.len());
        let mut sm = SourceMap::new();
        let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
        let spanned_tokens: Vec<(Token, SimpleSpan)> = tokens
            .into_iter()
            .map(|(t, s)| (t, SimpleSpan::from(s)))
            .collect();
        let input = spanned_tokens
            .as_slice()
            .map(eoi, |(t, s): &(Token, SimpleSpan)| (t, s));
        let (_deck_opt, parse_errs) = deck_parser(file_id).parse(input).into_output_errors();

        assert!(
            !parse_errs.is_empty(),
            "MED-1 EC-005: mixed-type list [\"A\", 42, \"C\"] must produce ≥1 parse error; got 0"
        );
        // Rich::custom reasons are accessible via format!("{:?}", e.reason()).
        let has_e_par_024 = parse_errs
            .iter()
            .any(|e| format!("{:?}", e.reason()).contains("E-PAR-024"));
        assert!(
            has_e_par_024,
            "MED-1 EC-005: at least one error must contain 'E-PAR-024'; \
             got: {parse_errs:?}"
        );
    }

    // ── MED-2: no-coercion test ──────────────────────────────────────────────

    /// BC-1.01.002 MED-2 / Architecture Compliance Rule #3 — no implicit type
    /// coercion: `bullets ["NO", "1.10"]` must produce `FieldValue::List` whose
    /// item literals are EXACTLY the strings "NO" and "1.10" — not `Bool(false)`,
    /// not `Float(1.1)`, not any normalized/coerced value.
    ///
    /// CLAUDE.md Forbidden Pattern: "Implicit type coercion (`NO` → bool, `1.10`
    /// → float) — All values stay as-is; strings are strings (R3 finding)."
    #[test]
    fn test_bc_1_01_002_med2_no_coercion_of_string_literals_in_list() {
        use crate::template::TemplateChunk;
        // These look like a boolean and a float but are quoted strings — must NOT be coerced.
        let src = concat!("slide content:\n", "  bullets [\"NO\", \"1.10\"]\n");
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "MED-2: no lex errors expected");
        assert_eq!(
            parse_err_count, 0,
            "MED-2: quoted strings \"NO\" and \"1.10\" must parse without error; \
             got {parse_err_count} errors"
        );

        let deck = deck.expect("MED-2: deck must parse");
        let crate::ast::BlockItem::Slide(slide_spanned) = &deck.items[0] else {
            panic!("MED-2: expected Slide");
        };
        let bullets_field = slide_spanned
            .value()
            .fields
            .iter()
            .find(|f| f.name.value() == "bullets")
            .expect("MED-2: 'bullets' field must exist");

        let FieldValue::List(items) = bullets_field.value.value() else {
            panic!(
                "MED-2: bullets must be FieldValue::List; got: {:?}",
                bullets_field.value.value()
            );
        };
        assert_eq!(
            items.len(),
            2,
            "MED-2: list must have 2 items; got: {items:?}"
        );

        // Item 0: MUST be Template("NO"), NOT Bool(false).
        let FieldValue::Template(chunks0) = &items[0] else {
            panic!(
                "MED-2: \"NO\" must be FieldValue::Template (not Bool/coerced); \
                 got: {:?}. Architecture Rule: no implicit type coercion.",
                items[0]
            );
        };
        assert_eq!(chunks0.len(), 1, "MED-2: \"NO\" must have 1 chunk");
        let TemplateChunk::Literal(s0) = &chunks0[0] else {
            panic!("MED-2: \"NO\" chunk must be Literal; got: {:?}", chunks0[0]);
        };
        assert_eq!(
            s0, "NO",
            "MED-2: literal must be exactly \"NO\", not coerced"
        );

        // Item 1: MUST be Template("1.10"), NOT Float(1.1).
        let FieldValue::Template(chunks1) = &items[1] else {
            panic!(
                "MED-2: \"1.10\" must be FieldValue::Template (not Float/coerced); \
                 got: {:?}. Architecture Rule: no implicit type coercion.",
                items[1]
            );
        };
        assert_eq!(chunks1.len(), 1, "MED-2: \"1.10\" must have 1 chunk");
        let TemplateChunk::Literal(s1) = &chunks1[0] else {
            panic!(
                "MED-2: \"1.10\" chunk must be Literal; got: {:?}",
                chunks1[0]
            );
        };
        assert_eq!(
            s1, "1.10",
            "MED-2: literal must be exactly \"1.10\", not coerced to float 1.1"
        );
    }

    // ── MED-001 (Pass-2): per-type <type> substitution in E-PAR-024 ──────────
    //
    // These load-bearing tests verify that E-PAR-024 emits the human-readable
    // type name (error-taxonomy v2.27 §E-PAR-024 binding format) rather than
    // the hardcoded string "non-string value".
    //
    // Each test targets one FieldValue variant and asserts the type name appears
    // in the emitted diagnostic. All tests use the deck-level parser path
    // (value_parser() via vars_block_parser / at_var_parser / field_line_parser).

    /// Helper: raw-parse and return error reason strings for inspection.
    fn parse_get_errors(src: &str) -> Vec<String> {
        let file: Arc<str> = Arc::from("test.sf");
        let (tokens, _lex_errs) = lex(src, file.clone());
        let eoi = SimpleSpan::from(src.len()..src.len());
        let mut sm = SourceMap::new();
        let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
        let spanned_tokens: Vec<(Token, SimpleSpan)> = tokens
            .into_iter()
            .map(|(t, s)| (t, SimpleSpan::from(s)))
            .collect();
        let input = spanned_tokens
            .as_slice()
            .map(eoi, |(t, s): &(Token, SimpleSpan)| (t, s));
        let (_deck_opt, parse_errs) = deck_parser(file_id).parse(input).into_output_errors();
        parse_errs
            .iter()
            .map(|e| format!("{:?}", e.reason()))
            .collect()
    }

    /// MED-001 (a) — `bullets [42]` → E-PAR-024 message contains "integer".
    ///
    /// Verifies the <type> substitution in the E-PAR-024 binding format for
    /// integer (`FieldValue::Num`) items (error-taxonomy v2.27 §E-PAR-024).
    #[test]
    fn test_bc_1_01_002_med001a_integer_item_produces_e_par_024_with_type_integer() {
        let src = concat!("slide content:\n", "  bullets [42]\n");
        let errors = parse_get_errors(src);
        assert!(
            !errors.is_empty(),
            "MED-001a: bullets [42] must produce ≥1 parse error; got 0"
        );
        let has_integer = errors.iter().any(|msg| msg.contains("integer"));
        assert!(
            has_integer,
            "MED-001a: E-PAR-024 message for bullets [42] must contain 'integer' \
             (not 'non-string value'); got errors: {errors:?}"
        );
    }

    /// MED-001 (b) — `bullets [true]` → E-PAR-024 message contains "boolean".
    ///
    /// Verifies the <type> substitution for boolean (`FieldValue::Bool`) items.
    #[test]
    fn test_bc_1_01_002_med001b_boolean_item_produces_e_par_024_with_type_boolean() {
        let src = concat!("slide content:\n", "  bullets [true]\n");
        let errors = parse_get_errors(src);
        assert!(
            !errors.is_empty(),
            "MED-001b: bullets [true] must produce ≥1 parse error; got 0"
        );
        let has_boolean = errors.iter().any(|msg| msg.contains("boolean"));
        assert!(
            has_boolean,
            "MED-001b: E-PAR-024 message for bullets [true] must contain 'boolean' \
             (not 'non-string value'); got errors: {errors:?}"
        );
    }

    /// MED-001 (c) — `bullets [someident]` → E-PAR-024 message contains "bare word".
    ///
    /// Verifies the <type> substitution for bare-identifier (`FieldValue::Ident`) items.
    #[test]
    fn test_bc_1_01_002_med001c_bare_word_item_produces_e_par_024_with_type_bare_word() {
        // `someident` is parsed as FieldValue::Ident (unquoted bare identifier).
        let src = concat!("slide content:\n", "  bullets [someident]\n");
        let errors = parse_get_errors(src);
        assert!(
            !errors.is_empty(),
            "MED-001c: bullets [someident] must produce ≥1 parse error; got 0"
        );
        let has_bare_word = errors.iter().any(|msg| msg.contains("bare word"));
        assert!(
            has_bare_word,
            "MED-001c: E-PAR-024 message for bullets [someident] must contain 'bare word' \
             (not 'non-string value'); got errors: {errors:?}"
        );
    }

    // ── MED-P3-001 (Pass-3): nested list literal → rejected (not silently accepted) ──
    //
    // STORY-088 spec line ~185: "Nested list-literals are NOT in scope; only
    // flat lists." The no-silent-failure ban requires that `[["A"]]` is never
    // silently accepted and passed through as a valid bullet list.
    //
    // Parser behavior: `[["A"]]` is rejected at the grammar level because `[`
    // (LBracket) is not a valid token for any list-item alternative in the
    // `value_parser` list_item grammar (chumsky produces an `ExpectedFound`
    // error before the validate() hook fires). This is structurally correct:
    // the grammar enforces flat-only lists, meaning `FieldValue::List(_)` as
    // a list item is unreachable from user input.
    //
    // The `FieldValue::List(_) => "nested list"` arm in the validate hook is
    // defense-in-depth for direct AST construction in tests; the grammar-level
    // rejection is the first line of enforcement.
    //
    // This test verifies the ENFORCED PROPERTY: `[["A"]]` is not silently accepted
    // (produces ≥1 error). The exact error code is ExpectedFound from chumsky,
    // not E-PAR-024, because the grammar rejects it before the validator fires.

    /// MED-P3-001 (deck path) — `bullets [["A"]]` → ≥1 parse error (NOT silently
    /// accepted).
    ///
    /// Load-bearing: enforces flat-only list scope at the deck-level parser.
    /// Verifies the silent-failure ban: nested syntax must produce a diagnostic,
    /// not silently produce a valid AST.
    #[test]
    fn test_bc_1_01_002_med_p3_001_deck_nested_list_item_is_not_silently_accepted() {
        let src = concat!("slide content:\n", "  bullets [[\"A\"]]\n");
        let errors = parse_get_errors(src);
        assert!(
            !errors.is_empty(),
            "MED-P3-001 (deck): bullets [[\"A\"]] must produce ≥1 parse error; got 0. \
             Nested list literals are out of scope (STORY-088 spec); they must be rejected, \
             not silently accepted as valid."
        );
        // The grammar-level rejection produces an ExpectedFound (LBracket not
        // expected as a list item token). This confirms nested lists are not
        // silently accepted — an error IS produced, just not E-PAR-024
        // (chumsky rejects the token before the validate() hook can fire).
        let has_lbracket_rejection = errors.iter().any(|msg| {
            msg.contains("LBracket") || msg.contains("E-PAR-024") || msg.contains("nested")
        });
        assert!(
            has_lbracket_rejection,
            "MED-P3-001 (deck): error for bullets [[\"A\"]] must reference the rejected \
             nested bracket token or E-PAR-024; got errors: {errors:?}"
        );
    }

    // ── MED-P3-002 (Pass-3): float list item → E-PAR-024 "decimal number" ────
    //
    // `bullets [1.5]` lexes to FloatLit → FieldValue::Float(1.5) as a list item.
    // The `FieldValue::Float(_) => "decimal number"` branch exists but was
    // previously untested. This load-bearing test closes that coverage gap.

    /// MED-P3-002 (deck path) — `bullets [1.5]` → ≥1 error containing
    /// `"E-PAR-024"` AND `"decimal number"`.
    ///
    /// Load-bearing: exercises the `FieldValue::Float(_) => "decimal number"`
    /// arm in `value_parser`'s `list_item` validator (deck-level parser path).
    #[test]
    fn test_bc_1_01_002_med_p3_002_deck_float_item_produces_e_par_024_decimal_number() {
        let src = concat!("slide content:\n", "  bullets [1.5]\n");
        let errors = parse_get_errors(src);
        assert!(
            !errors.is_empty(),
            "MED-P3-002 (deck): bullets [1.5] must produce ≥1 parse error; got 0"
        );
        let has_decimal = errors.iter().any(|msg| msg.contains("decimal number"));
        assert!(
            has_decimal,
            "MED-P3-002 (deck): E-PAR-024 message for bullets [1.5] must contain \
             'decimal number'; got errors: {errors:?}"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STORY-088 AC-012: set-rule list-literal default
    // Traces to BC-1.01.002 — set_rule_value_parser() is a value-position
    // parser surface for the field-value grammar.
    // RED GATE: set_rule_value_parser() has no list arm → these tests FAIL.
    // ═══════════════════════════════════════════════════════════════════════

    /// Helper: raw-parse and return (`deck_opt`, `parse_errors` count) for set-rule test.
    fn parse_set_rule_list(src: &str) -> (Option<DeckNode>, Vec<String>) {
        let file: Arc<str> = Arc::from("test.sf");
        let (tokens, _lex_errs) = lex(src, file.clone());
        let eoi = SimpleSpan::from(src.len()..src.len());
        let mut sm = SourceMap::new();
        let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
        let spanned_tokens: Vec<(Token, SimpleSpan)> = tokens
            .into_iter()
            .map(|(t, s)| (t, SimpleSpan::from(s)))
            .collect();
        let input = spanned_tokens
            .as_slice()
            .map(eoi, |(t, s): &(Token, SimpleSpan)| (t, s));
        let (deck_opt, parse_errs) = deck_parser(file_id).parse(input).into_output_errors();
        let reasons: Vec<String> = parse_errs
            .iter()
            .map(|e| format!("{:?}", e.reason()))
            .collect();
        (deck_opt, reasons)
    }

    /// BC-1.01.002 AC-012 (positive) — `set content: bullets ["Step 1", "Step 2"]`
    /// parses without error and produces a set rule whose value is a list type.
    ///
    /// RED GATE: `set_rule_value_parser()` has no list-literal arm → the `[` token
    /// is unexpected → parse error → no set-rule value produced.
    #[test]
    fn test_bc_1_01_002_ac012_set_rule_list_literal_parses_to_list_value() {
        use crate::ast::SetRuleValue;
        let src = concat!(
            "set content: bullets [\"Step 1\", \"Step 2\", \"Step 3\"]\n",
            "slide content:\n",
            "  title \"Test\"\n",
        );
        let (deck, errors) = parse_set_rule_list(src);
        assert!(
            errors.is_empty(),
            "AC-012: set content: bullets [...] must parse without error; \
             got errors: {errors:?}\n\
             RED GATE: set_rule_value_parser() has no list arm — add SetRuleValue::List \
             and a list-literal arm."
        );
        let deck = deck.expect("AC-012: deck must parse");
        assert_eq!(deck.set_rules.len(), 1, "AC-012: deck must have 1 set rule");
        let sr = &deck.set_rules[0];
        assert_eq!(
            sr.slide_type.value(),
            "content",
            "AC-012: slide_type must be 'content'"
        );
        assert_eq!(
            sr.field.value(),
            "bullets",
            "AC-012: field must be 'bullets'"
        );
        // The value must be SetRuleValue::List with 3 items.
        // RED GATE: without the list arm this will be SetRuleValue::Error.
        let SetRuleValue::List(items) = sr.value.value() else {
            panic!(
                "AC-012 RED GATE: set-rule value must be SetRuleValue::List; got: {:?}. \
                 Add SetRuleValue::List variant and set_rule_value_parser() list arm.",
                sr.value.value()
            );
        };
        assert_eq!(
            items.len(),
            3,
            "AC-012: SetRuleValue::List must have 3 items; got: {items:?}"
        );
        for (i, item) in items.iter().enumerate() {
            assert!(
                matches!(item, FieldValue::Template(_)),
                "AC-012: item[{i}] must be FieldValue::Template; got: {item:?}"
            );
        }
    }

    /// BC-1.01.002 AC-012 (empty list) — `set content: bullets []` parses to an
    /// empty list value with 0 errors.
    ///
    /// RED GATE: no list arm in `set_rule_value_parser()`.
    #[test]
    fn test_bc_1_01_002_ac012_set_rule_empty_list_literal_is_valid() {
        use crate::ast::SetRuleValue;
        let src = concat!(
            "set content: bullets []\n",
            "slide content:\n",
            "  title \"T\"\n",
        );
        let (deck, errors) = parse_set_rule_list(src);
        assert!(
            errors.is_empty(),
            "AC-012 empty: set content: bullets [] must parse without error; got: {errors:?}"
        );
        let deck = deck.expect("AC-012 empty: deck must parse");
        let sr = &deck.set_rules[0];
        let SetRuleValue::List(items) = sr.value.value() else {
            panic!(
                "AC-012 empty RED GATE: empty [] must produce SetRuleValue::List([]); got: {:?}",
                sr.value.value()
            );
        };
        assert!(
            items.is_empty(),
            "AC-012 empty: list must be empty; got {items:?}"
        );
    }

    /// BC-1.01.002 AC-012 (error case) — `set content: bullets [42, true]` produces
    /// E-PAR-024 (non-string list element). Error accumulation applies.
    ///
    /// RED GATE: no list arm → the error is a different token-unexpected error.
    /// After implementation: specific E-PAR-024 errors are present.
    #[test]
    fn test_bc_1_01_002_ac012_set_rule_non_string_list_items_produce_e_par_024() {
        let src = concat!(
            "set content: bullets [42, true]\n",
            "slide content:\n",
            "  title \"T\"\n",
        );
        let (_deck, errors) = parse_set_rule_list(src);
        // Must produce at least one parse error (E-PAR-024 or token-unexpected).
        assert!(
            !errors.is_empty(),
            "AC-012 error: set content: bullets [42, true] must produce ≥1 parse error; got 0"
        );
        // After implementation: E-PAR-024 must be present.
        let has_e_par_024 = errors.iter().any(|msg| msg.contains("E-PAR-024"));
        assert!(
            has_e_par_024,
            "AC-012 error RED GATE: errors must include E-PAR-024 for non-string items; \
             got: {errors:?}"
        );
    }

    /// BC-1.01.002 AC-012 (trailing comma) — `set content: bullets ["A", "B",]`
    /// trailing comma is accepted (`allow_trailing`).
    ///
    /// RED GATE: no list arm → different error.
    #[test]
    fn test_bc_1_01_002_ac012_set_rule_list_trailing_comma_accepted() {
        use crate::ast::SetRuleValue;
        let src = concat!(
            "set content: bullets [\"A\", \"B\",]\n",
            "slide content:\n",
            "  title \"T\"\n",
        );
        let (deck, errors) = parse_set_rule_list(src);
        assert!(
            errors.is_empty(),
            "AC-012 trailing comma: trailing comma must be accepted; got errors: {errors:?}"
        );
        let deck = deck.expect("AC-012 trailing comma: deck must parse");
        let SetRuleValue::List(items) = deck.set_rules[0].value.value() else {
            panic!(
                "AC-012 trailing comma RED GATE: must be SetRuleValue::List; got: {:?}",
                deck.set_rules[0].value.value()
            );
        };
        assert_eq!(
            items.len(),
            2,
            "AC-012 trailing comma: trailing comma must not add extra item; got {items:?}"
        );
    }
}
