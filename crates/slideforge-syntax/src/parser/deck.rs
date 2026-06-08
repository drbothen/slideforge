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
/// integer, float, bool, or identifier.
///
/// String literals are parsed through `template_value()` so that `{{ expr }}`
/// interpolation is supported. Template errors are emitted via `validate()`.
///
/// On error, produces `FieldValue::Error` sentinel for recovery.
fn value_parser<'src, I>()
-> impl Parser<'src, I, (FieldValue, TSpan), extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
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

    template_val.or(other_val)
}

/// Parser for a `set` rule value: string literal (with template interpolation),
/// integer, float, bool, or identifier.
///
/// Identical in surface syntax to `value_parser()` but produces [`SetRuleValue`]
/// so that `set` rules carry a distinct type from field values.
///
/// BC-1.08.002: `{{ brand.footer }}` is parsed as
/// `SetRuleValue::Template([TemplateChunk::Expr(Expr::FieldAccess { base:
/// Ident("brand"), field: "footer" })])`. The expression parser handles this
/// naturally via field-access syntax — no special `BrandRef` token is needed.
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

    template_val.or(other_val)
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
    // STORY-082: section group (quoted-string form: `section "Name":`)
    let section_group = section_group_parser(file_id).map(DeckItem::SectionGroup);
    // Use block_item() to handle slide, @for, and @if at deck level.
    let block = block_item(file_id).map(DeckItem::Block);

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
                        // STORY-082 AC-010: discard empty-name SectionGroupNode.
                        // The parser emitted E-PAR-023 for empty names; we must
                        // not produce any AST node for the rejected block.
                        if let crate::ast::BlockItem::SectionGroup(ref spanned) = bi {
                            let name = spanned.value().name.value();
                            if name.is_empty() {
                                // Empty name was already reported by section_group_parser.
                                // Drop the sentinel node — do NOT push to deck.items.
                                continue;
                            }
                            // STORY-082 AC-011: duplicate name detection (W-PAR-002).
                            // Both sections are emitted; warning is cosmetic (exit 0).
                            let name_arc: std::sync::Arc<str> = std::sync::Arc::clone(name);
                            if seen_section_group_names.contains(&name_arc) {
                                emitter.emit(Rich::custom(
                                    SimpleSpan::from(0usize..0usize),
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
}
