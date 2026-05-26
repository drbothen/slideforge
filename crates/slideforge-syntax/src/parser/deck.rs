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
//! vars_block   ::= "vars" ":" INDENT (IDENT value NEWLINE)+ DEDENT
//! set_rule     ::= "set" IDENT ":" IDENT value NEWLINE
//! slide_block  ::= "slide" IDENT ":" INDENT (field_line)+ DEDENT
//! field_line   ::= IDENT value NEWLINE
//! value        ::= STRING | NUMBER | IDENT
//! ```

use chumsky::{input::ValueInput, prelude::*};

use crate::{
    ast::{DeckNode, FieldNode, FieldValue, SetRule, SlideNode, VarsBlock},
    span::{Span, Spanned},
    token::Token,
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

/// Parser for a field value: string literal, integer, float, bool, or identifier.
///
/// On error, produces `FieldValue::Error` sentinel for recovery.
fn value_parser<'src, I>()
-> impl Parser<'src, I, (FieldValue, TSpan), extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    select! {
        Token::StringLit(s) = e => (FieldValue::Str(s.to_string()), e.span()),
        Token::IntLit(n) = e => (FieldValue::Num(n), e.span()),
        Token::BoolLit(b) = e => (FieldValue::Ident(b.to_string()), e.span()),
        Token::Ident(s) = e => (FieldValue::Ident(s.to_string()), e.span()),
    }
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
    keyword("vars")
        .then_ignore(just(Token::Colon))
        .then_ignore(just(Token::Newline).or_not())
        .then_ignore(select! { Token::Indent(_) => () })
        .then(
            any_ident()
                .then(value_parser())
                .then_ignore(just(Token::Newline).or_not())
                .map(move |((name, name_span), (val, val_span))| {
                    (
                        Spanned::new(name, to_span(name_span, file_id)),
                        Spanned::new(val, to_span(val_span, file_id)),
                    )
                })
                .repeated()
                .collect::<Vec<_>>(),
        )
        .then_ignore(just(Token::Dedent))
        .map(|(_vars_kw_span, entries)| VarsBlock { entries })
}

// ─── Set rule parser ──────────────────────────────────────────────────────────

/// Parser for a `set <type>: <field> <value>` rule.
///
/// Pattern: `"set" IDENT ":" IDENT value NEWLINE`
fn set_rule_parser<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, SetRule, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    keyword("set")
        .then(any_ident())
        .then_ignore(just(Token::Colon))
        .then(any_ident())
        .then(value_parser())
        .then_ignore(just(Token::Newline).or_not())
        .map(
            move |(
                ((_set_kw_span, (slide_type, type_span)), (field, field_span)),
                (val, val_span),
            )| {
                SetRule {
                    slide_type: Spanned::new(slide_type, to_span(type_span, file_id)),
                    field: Spanned::new(field, to_span(field_span, file_id)),
                    value: Spanned::new(val, to_span(val_span, file_id)),
                }
            },
        )
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
    /// A `set <type>: <field> <value>` rule.
    Set(SetRule),
    /// A `slide <type>:` block, paired with its chumsky span.
    Slide(SlideNode, TSpan),
}

// ─── Public deck parser ───────────────────────────────────────────────────────

/// Build the top-level deck parser.
///
/// Returns a chumsky parser that consumes a [`ValueInput`] token stream and
/// produces a [`DeckNode`].
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
    let set = set_rule_parser(file_id).map(DeckItem::Set);
    let slide = slide_block_parser(file_id).map(|(s, sp)| DeckItem::Slide(s, sp));

    let item = version
        .or(lang)
        .or(brand)
        .or(vars)
        .or(set)
        .or(slide)
        .recover_with(skip_then_retry_until(
            any()
                .filter(|t| !matches!(t, Token::Newline | Token::Eof))
                .ignored(),
            just(Token::Newline).ignored(),
        ));

    nl.clone()
        .repeated()
        .ignore_then(item.padded_by(nl.repeated()).repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::Eof).or_not())
        .map(move |items| {
            let mut deck = DeckNode::default();
            for item in items {
                match item {
                    DeckItem::Version(v) => deck.version = Some(v),
                    DeckItem::Lang(l) => deck.lang = Some(l),
                    DeckItem::Brand(b) => deck.brand = Some(b),
                    DeckItem::Vars(vb) => deck.vars.push(vb),
                    DeckItem::Set(sr) => deck.set_rules.push(sr),
                    DeckItem::Slide(s, sp) => {
                        deck.slides.push(Spanned::new(s, to_span(sp, file_id)));
                    },
                }
            }
            deck
        })
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
        assert_eq!(deck.slides.len(), 1, "should have exactly 1 slide");
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
        // Either Ok with empty slides or None — must not panic.
        if let Some(d) = deck {
            assert!(d.slides.is_empty(), "empty source must produce 0 slides");
        }
    }

    // ── AC-010: slide fields captured ────────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_slide_fields_captured() {
        let src = concat!(
            "slide content:\n",
            "  title \"Hello\"\n",
            "  footer \"Slide 1\"\n",
        );
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "lex errors: {lex_errs:?}");
        assert_eq!(parse_err_count, 0, "parse errors must be 0");
        let deck = deck.expect("must parse");
        assert_eq!(deck.slides.len(), 1);
        let slide = deck.slides[0].value();
        assert_eq!(slide.fields.len(), 2, "slide must have 2 fields");
        let title_field = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "title")
            .expect("title field must exist");
        assert_eq!(
            title_field.value.value(),
            &FieldValue::Str("Hello".to_string())
        );
        let footer_field = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "footer")
            .expect("footer field must exist");
        assert_eq!(
            footer_field.value.value(),
            &FieldValue::Str("Slide 1".to_string())
        );
    }

    // ── AC-002: vars block parsed ─────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_vars_block_parsed() {
        let src = concat!(
            "vars:\n",
            "  client \"Acme\"\n",
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
            &FieldValue::Str("Acme".to_string())
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
        let (deck, lex_errs, parse_err_count) = parse_src(src);
        assert!(lex_errs.is_empty(), "lex errors: {lex_errs:?}");
        assert_eq!(parse_err_count, 0, "parse errors must be 0");
        let deck = deck.expect("must parse");
        assert_eq!(deck.set_rules.len(), 1, "should have 1 set rule");
        let sr = &deck.set_rules[0];
        assert_eq!(sr.slide_type.value(), "content");
        assert_eq!(sr.field.value(), "footer");
        assert_eq!(sr.value.value(), &FieldValue::Str("Default".to_string()));
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
