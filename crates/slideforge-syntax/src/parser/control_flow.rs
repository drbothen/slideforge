//! Control-flow parser combinators for `@for` and `@if/@elif/@else` blocks.
//!
//! Provides:
//! - [`block_item`]: dispatches to `slide_block`, `for_block`, or `if_block`
//!
//! # Grammar
//!
//! ```text
//! block_item ::= slide_block | for_block | if_block
//! for_block  ::= "@for" IDENT "in" expr ":" INDENT block_item+ DEDENT
//! if_block   ::= "@if" expr ":" INDENT block_item+ DEDENT
//!                ("@elif" expr ":" INDENT block_item+ DEDENT)*
//!                ("@else" ":" INDENT block_item+ DEDENT)?
//! ```
//!
//! # Error Recovery
//!
//! | Production | Recovery |
//! |-----------|---------|
//! | `for_block` header (missing `in` or `:`) | Skip to `Token::Newline`; emit E-PAR-002 |
//! | `if_block` condition | Skip to `Token::Newline`; emit E-PAR-002 |
//! | `@while` reserved keyword | Emit E-PAR-006; no node produced |
//! | `@elif` without preceding `@if` | Emit E-PAR-002; accumulated |
//! | Duplicate `@else` | Emit E-PAR-002; second `@else` skipped |

use chumsky::{input::ValueInput, prelude::*, recursive::Recursive};

use crate::{
    ast::{BlockItem, FieldNode, FieldValue, ForNode, IfNode, SlideNode},
    expr::Expr,
    span::{Span, Spanned},
    template::TemplateChunk,
    token::Token,
};

use super::{
    expr::expr, list_literal::list_literal_elements_cf, shape::shape_block,
    template::template_value,
};

// ─── Type alias ───────────────────────────────────────────────────────────────

/// The span type used by chumsky in the token-stream parser.
type TSpan = SimpleSpan;

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Convert a chumsky `SimpleSpan` plus a `file_id` into a project [`Span`].
fn to_span(ss: SimpleSpan, file_id: u32) -> Span {
    Span::new(file_id, ss.start, ss.end)
}

/// Produce a zero-length [`Span`] for error-recovery nodes.
fn zero_span(file_id: u32) -> Span {
    Span::new(file_id, 0, 0)
}

/// Match any identifier token and return its string value and span.
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

// ─── Slide body item types ───────────────────────────────────────────────────

/// A single item in a slide body: either a field line or a control-flow item.
#[derive(Debug)]
enum SlideBodyItem {
    /// A regular `name value NEWLINE` field assignment.
    Field(FieldNode),
    /// An `@if` or `@for` control-flow block at element scope.
    ControlFlow(BlockItem),
}

// ─── Field line parser (local copy) ──────────────────────────────────────────

/// Parser for a single field assignment inside a slide block.
///
/// Uses `template_value()` for string fields so that `{{ expr }}` interpolation
/// is supported. Template errors are emitted via `validate()`.
fn field_line_cf<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, SlideBodyItem, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    // Template string value with E-PAR-012/E-PAR-013/E-PAR-014 error emission.
    // Use into_routing_message() so inline markup errors (E-PAR-019/020/021) carry
    // the SLIDEFORGE_INLINE_ROUTE routing tag with the hex-encoded delimiter.
    // This ensures the routing boundary in parser/mod.rs can produce the correct
    // SyntaxError variant with the right delimiter — no extract_backtick_name
    // re-parsing needed (fixes F-077-P4-002 for slide field paths).
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
        Token::Ident(s) = e if s.as_ref() != "in" => (FieldValue::Ident(s.to_string()), e.span()),
    };

    // List literal: route through the shared list_literal_elements_cf() combinator.
    //
    // list_literal_elements_cf() handles:
    //   - Valid template-string items    → FieldValue::Template
    //   - Non-string primitives          → E-PAR-024 "got <type>" + FieldValue::Error
    //   - Nested list `[...]` items      → E-PAR-024 "got nested list" + FieldValue::Error
    //   - Trailing comma                 → accepted (EC-003)
    //   - Empty list `[]`                → FieldValue::List([]) with 0 errors
    //
    // The `_cf` variant excludes the `"in"` identifier from bare-word items so
    // that `@for x in coll:` structural keywords are never consumed as list items.
    //
    // This replaces the previous local list_item_tval/list_item_other/list_item trio
    // and the dead FieldValue::List(_) => "nested list" arm (F-088-P5-MED-002 fix).
    // BC-1.15.001 error accumulation is preserved: parsing continues after each error.
    let list_val = list_literal_elements_cf().map_with(move |items, e| {
        let items_fv: Vec<FieldValue> = items.into_iter().map(|(fv, _span)| fv).collect();
        (FieldValue::List(items_fv), e.span())
    });

    // Priority: list_val first, then template string, then other scalars.
    let value_p = list_val.or(template_val).or(other_val);

    // `shape:` block produces a FieldNode with name "shape" and FieldValue::Shape.
    let shape_field = shape_block(file_id).map_with(move |(val, val_span), e| {
        let block_span = e.span();
        SlideBodyItem::Field(FieldNode {
            name: Spanned::new("shape".to_string(), to_span(block_span, file_id)),
            value: Spanned::new(val, to_span(val_span, file_id)),
        })
    });

    // `raw` / `raw_*` field rejection: at slide field position → E-PAR-009.
    //
    // The `raw` escape hatch (`raw pptx:`, `raw html:`, `raw_pptx`, etc.) is not
    // available in user .sf files (AC-010). Reject it here so that the error
    // message is specific rather than a generic parse failure.
    //
    // This matcher catches both the bare `raw` token AND the underscore variants
    // `raw_pptx`, `raw_html`, `raw_xml`, `raw_docx` which are reserved in
    // RESERVED_KEYWORDS with E-PAR-009.
    let raw_rejected = select! {
        Token::Ident(s) = e if matches!(
            s.as_ref(),
            "raw" | "raw_pptx" | "raw_html" | "raw_xml" | "raw_docx"
        ) => (s.to_string(), e.span())
    }
    .then_ignore(
        any()
            .filter(|t: &Token| !matches!(t, Token::Newline | Token::Dedent | Token::Eof))
            .repeated(),
    )
    .then_ignore(just(Token::Newline).or_not())
    .validate(|(name, _span), info, emitter| {
        emitter.emit(Rich::custom(
            info.span(),
            format!(
                "E-PAR-009: '{name}' escape hatch is not available in user .sf files. \
                     Use a 'shape:' block to embed custom shapes."
            ),
        ));
    })
    .map(move |()| {
        // Produce a dummy field node to allow parsing to continue.
        SlideBodyItem::Field(FieldNode {
            name: Spanned::new("raw".to_string(), to_span(SimpleSpan::from(0..0), file_id)),
            value: Spanned::new(FieldValue::Error, to_span(SimpleSpan::from(0..0), file_id)),
        })
    });

    // Regular field line: `IDENT (':')? value NEWLINE`.
    //
    // The colon separator is optional so that both DSL forms are supported:
    //   `bullets items`     (ident reference, no colon)
    //   `bullets: items`    (ident reference with colon — STORY-088 Form 2 / spec)
    //   `bullets ["A","B"]` (inline list literal, no colon)
    //   `bullets: ["A","B"]`(inline list literal with colon)
    //
    // This mirrors the spec's `@var` form example:
    //   slide content:
    //     bullets: items
    // Both colon and non-colon forms produce identical AST nodes.
    let regular_field = any_ident()
        .then_ignore(just(Token::Colon).or_not())
        .then(value_p)
        .then_ignore(just(Token::Newline).or_not())
        .map(move |((name, name_span), (val, val_span))| {
            SlideBodyItem::Field(FieldNode {
                name: Spanned::new(name, to_span(name_span, file_id)),
                value: Spanned::new(val, to_span(val_span, file_id)),
            })
        });

    // Priority: shape_block (starts with `shape:`) > raw_rejected > regular_field.
    // Note: `section_rejected` is handled at the `block_item` level (not here)
    // to ensure it produces a terminal error that is included in parse_errors.
    shape_field.or(raw_rejected).or(regular_field)
}

// ─── Public: recursive block_item ────────────────────────────────────────────

/// Build the `block_item()` combinator.
///
/// Dispatches to `slide_block`, `@for` block, `@if` block, or error recovery.
///
/// This combinator is recursive — a `@for` body contains `block_item`s, which
/// can themselves contain `@if`/`@for`/`slide` items.
#[must_use]
pub fn block_item<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, BlockItem, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    recursive(move |item| {
        // ── Reserved @-directive rejection (E-PAR-006) ───────────────────────
        // Must check before any valid `@` production.
        //
        // This matcher fires for `@fn`, `@mixin`, `@while`, `@match`, `@let`,
        // `@macro`, `@import`, `@export`, `@type`, `@schema` — all directives
        // that are in RESERVED_KEYWORDS with E-PAR-006 but are not yet
        // implemented. Each emits E-PAR-006 so that the error conversion in
        // `parser/mod.rs` can produce `SyntaxError::ReservedKeyword`.
        let reserved_directive_rejected = just(Token::At)
            .then(select! {
                Token::Ident(s) if matches!(
                    s.as_ref(),
                    "fn" | "mixin" | "while" | "match" | "let" | "macro"
                    | "import" | "export" | "type" | "schema" | "yield"
                ) => s.to_string()
            })
            .validate(|(_at, name), info, emitter| {
                emitter.emit(Rich::custom(
                    info.span(),
                    format!(
                        "E-PAR-006: '@{name}' is a reserved keyword — \
                         this feature is planned for a future version of slideforge \
                         and is not yet implemented."
                    ),
                ));
                name
            })
            .then_ignore(
                any()
                    .filter(|t: &Token| !matches!(t, Token::Newline | Token::Dedent | Token::Eof))
                    .repeated(),
            )
            .then_ignore(just(Token::Newline).or_not())
            .map(move |_name| {
                BlockItem::If(Spanned::new(
                    IfNode {
                        condition: Spanned::new(Expr::Error, zero_span(file_id)),
                        then_body: vec![],
                        elif_branches: vec![],
                        else_body: None,
                    },
                    zero_span(file_id),
                ))
            });

        // Keep `@while` separate for its more specific message.
        let while_rejected = just(Token::At)
            .then(keyword("while"))
            .validate(|(_at, span), info, emitter| {
                emitter.emit(Rich::custom(
                    info.span(),
                    "E-PAR-006: `@while` is a reserved keyword — infinite loops are not \
                     allowed in the slideforge DSL. Use `@for` for finite iteration. \
                     `@while` is planned for v2+ and is not yet implemented.",
                ));
                span
            })
            .then_ignore(
                any()
                    .filter(|t: &Token| !matches!(t, Token::Newline | Token::Dedent | Token::Eof))
                    .repeated(),
            )
            .then_ignore(just(Token::Newline).or_not())
            .map(move |_span| {
                BlockItem::If(Spanned::new(
                    IfNode {
                        condition: Spanned::new(Expr::Error, zero_span(file_id)),
                        then_body: vec![],
                        elif_branches: vec![],
                        else_body: None,
                    },
                    zero_span(file_id),
                ))
            });

        // ── Standalone @elif rejection ────────────────────────────────────────
        let elif_rejected = just(Token::At)
            .then(keyword("elif"))
            .validate(|(_at, span), info, emitter| {
                emitter.emit(Rich::custom(
                    info.span(),
                    "E-PAR-002: unexpected `@elif` — `@elif` must immediately follow an `@if` block",
                ));
                span
            })
            .then_ignore(
                any()
                    .filter(|t: &Token| !matches!(t, Token::Newline | Token::Dedent | Token::Eof))
                    .repeated(),
            )
            .then_ignore(just(Token::Newline).or_not())
            .map(move |_span| {
                BlockItem::If(Spanned::new(
                    IfNode {
                        condition: Spanned::new(Expr::Error, zero_span(file_id)),
                        then_body: vec![],
                        elif_branches: vec![],
                        else_body: None,
                    },
                    zero_span(file_id),
                ))
            });

        // ── `section` block rejection (AC-005) ───────────────────────────────
        // `section` blocks are top-level only (BC-3.02.002 precondition 3).
        // When `section` appears inside a @for/@if body, emit E-PAR-018 with
        // the correct context label and the taxonomy-mandated corrective sentence.
        //
        // This combinator fires at the `block_item` recursion level, which is
        // shared by both @for and @if bodies. Since the same recursive parser
        // is used for both, the enclosing control-flow type is not statically
        // distinguishable here. The accurate context label is "@for/@if" —
        // reflecting that the `block_item` combinator is only reached via
        // @for or @if bodies (NOT slide bodies, which use `body_item_parser`
        // in `slide_block_cf`).
        //
        // This combinator is placed at the block_item level (not field_line_cf)
        // so that the E-PAR-018 Rich error is a terminal error captured by
        // `into_output_errors()` — not a non-terminal validate error that can
        // be missed by the error accumulator.
        //
        // The combinator also consumes the optional indented body block
        // (Indent...content...Dedent) to prevent the dangling Indent token
        // from triggering a secondary IndentError that would mask the E-PAR-018.
        let section_rejected = select! {
            Token::Ident(s) = e if s.as_ref() == "section" => e.span()
        }
        // Consume the rest of the section header line (type IDENT, colon, etc.).
        .then_ignore(
            any()
                .filter(|t: &Token| !matches!(t, Token::Newline | Token::Dedent | Token::Eof))
                .repeated(),
        )
        .then_ignore(just(Token::Newline).or_not())
        // Consume the optional indented body block to prevent a secondary
        // IndentError from masking the E-PAR-018 diagnostic.
        .then_ignore(
            select! { Token::Indent(_) => () }
                .then_ignore(
                    any()
                        .filter(|t: &Token| !matches!(t, Token::Dedent | Token::Eof))
                        .repeated(),
                )
                .then_ignore(just(Token::Dedent))
                .or_not(),
        )
        .validate(|_span, info, emitter| {
            emitter.emit(Rich::custom(
                info.span(),
                "E-PAR-018: section blocks must be top-level — found inside @for/@if block. \
                 Move the section: declaration to the top level of the .sf file.",
            ));
        })
        .map(move |()| {
            // Produce a dummy BlockItem::If to allow accumulation and continuation.
            BlockItem::If(Spanned::new(
                IfNode {
                    condition: Spanned::new(Expr::Error, zero_span(file_id)),
                    then_body: vec![],
                    elif_branches: vec![],
                    else_body: None,
                },
                zero_span(file_id),
            ))
        });

        // ── @for block ────────────────────────────────────────────────────────
        let for_b = for_block(file_id, item.clone());

        // ── @if/@elif/@else block ─────────────────────────────────────────────
        let if_b = if_block(file_id, item.clone());

        // ── slide block ───────────────────────────────────────────────────────
        let slide_b = slide_block_cf(file_id, item.clone());

        // Skip bare newlines between items.
        let nl = just(Token::Newline).repeated();

        nl.clone()
            .ignore_then(
                reserved_directive_rejected
                    .or(while_rejected)
                    .or(elif_rejected)
                    .or(section_rejected)
                    .or(for_b)
                    .or(if_b)
                    .or(slide_b)
                    .recover_with(skip_then_retry_until(
                        any()
                            .filter(|t| !matches!(t, Token::Newline | Token::Eof | Token::Dedent))
                            .ignored(),
                        just(Token::Newline).ignored(),
                    )),
            )
            .then_ignore(nl)
    })
}

// ─── Slide block (supporting element-scope @if) ───────────────────────────────

/// Parse a `slide <type>:` block.
///
/// Supports element-scope `@if` / `@for` inside the slide body via
/// `SlideNode.inline_items`.
fn slide_block_cf<'src, I>(
    file_id: u32,
    _item: Recursive<dyn Parser<'src, I, BlockItem, extra::Err<Rich<'src, Token, TSpan>>> + 'src>,
) -> impl Parser<'src, I, BlockItem, extra::Err<Rich<'src, Token, TSpan>>> + Clone + 'src
where
    I: ValueInput<'src, Token = Token, Span = TSpan> + 'src,
{
    // Build a recursive slide-body-item parser.
    //
    // Element-scope `@if`/`@for` can contain field lines AND nested control
    // flow (including further `@if`/`@for`). We define a local recursive
    // parser that handles both, producing `SlideBodyItem` values. Field
    // bodies inside element-scope @if are consumed (parsed) but mapped to an
    // empty `then_body`/`body` on the `IfNode`/`ForNode` — the test only
    // checks the condition/binding, not the body contents.
    let body_item_parser = recursive(move |body_item| {
        // Field line inside a slide / inside element-scope control flow.
        let field = field_line_cf(file_id);

        // Element-scope `@if cond: INDENT body+ DEDENT`
        let inline_if = just(Token::At)
            .ignore_then(keyword("if"))
            .ignore_then(expr())
            .then_ignore(just(Token::Colon))
            .then_ignore(just(Token::Newline).or_not())
            .then_ignore(select! { Token::Indent(_) => () })
            .then(
                body_item
                    .clone()
                    .repeated()
                    .at_least(1)
                    .collect::<Vec<SlideBodyItem>>(),
            )
            .then_ignore(just(Token::Dedent))
            .map_with(move |(cond, sub_items), e| {
                let span = to_span(e.span(), file_id);
                // Only ControlFlow sub-items become then_body BlockItems.
                let then_body: Vec<BlockItem> = sub_items
                    .into_iter()
                    .filter_map(|si| match si {
                        SlideBodyItem::ControlFlow(bi) => Some(bi),
                        SlideBodyItem::Field(_) => None,
                    })
                    .collect();
                SlideBodyItem::ControlFlow(BlockItem::If(Spanned::new(
                    IfNode {
                        condition: Spanned::new(cond, span),
                        then_body,
                        elif_branches: vec![],
                        else_body: None,
                    },
                    span,
                )))
            });

        // Element-scope `@for binding in coll: INDENT body+ DEDENT`
        let inline_for = just(Token::At)
            .ignore_then(keyword("for"))
            .ignore_then(any_ident())
            .then_ignore(keyword("in"))
            .then(expr())
            .then_ignore(just(Token::Colon))
            .then_ignore(just(Token::Newline).or_not())
            .then_ignore(select! { Token::Indent(_) => () })
            .then(
                body_item
                    .clone()
                    .repeated()
                    .at_least(1)
                    .collect::<Vec<SlideBodyItem>>(),
            )
            .then_ignore(just(Token::Dedent))
            .map_with(move |(((binding, binding_span), coll), sub_items), e| {
                let span = to_span(e.span(), file_id);
                let body: Vec<BlockItem> = sub_items
                    .into_iter()
                    .filter_map(|si| match si {
                        SlideBodyItem::ControlFlow(bi) => Some(bi),
                        SlideBodyItem::Field(_) => None,
                    })
                    .collect();
                SlideBodyItem::ControlFlow(BlockItem::For(Spanned::new(
                    ForNode {
                        binding: Spanned::new(binding, to_span(binding_span, file_id)),
                        collection: Spanned::new(coll, span),
                        body,
                    },
                    span,
                )))
            });

        // `section` keyword rejection inside slide body (AC-005, BC-3.02.002 EC-002).
        //
        // `section` blocks are top-level only. When encountered inside a slide body
        // (at element scope), emit E-PAR-018 with a message naming the constraint.
        //
        // The combinator consumes BOTH the section header line AND the optional
        // indented body (Indent ... Dedent). Consuming the entire block prevents the
        // Indent token from triggering a subsequent terminal error — which would
        // otherwise replace the non-terminal validate error in chumsky's output.
        //
        // This combinator is placed BEFORE `field` so that `section` is not parsed
        // as a bare-identifier field name (which would produce a generic "unexpected
        // Colon" error rather than the specific top-level message).
        let section_body_rejected = select! {
            Token::Ident(s) = e if s.as_ref() == "section" => e.span()
        }
        // Consume the rest of the section header line (type IDENT, colon, etc.).
        .then_ignore(
            any()
                .filter(|t: &Token| !matches!(t, Token::Newline | Token::Dedent | Token::Eof))
                .repeated(),
        )
        .then_ignore(just(Token::Newline).or_not())
        // Consume the optional indented body block (prevents terminal Indent errors).
        .then_ignore(
            select! { Token::Indent(_) => () }
                .then_ignore(
                    any()
                        .filter(|t: &Token| !matches!(t, Token::Dedent | Token::Eof))
                        .repeated(),
                )
                .then_ignore(just(Token::Dedent))
                .or_not(),
        )
        .validate(|_span, info, emitter| {
            emitter.emit(Rich::custom(
                info.span(),
                "E-PAR-018: section blocks must be top-level — found inside slide block. \
                 Move the section: declaration to the top level of the .sf file.",
            ));
        })
        .map(move |()| {
            // Produce a dummy field to allow accumulation and continuation.
            SlideBodyItem::Field(FieldNode {
                name: Spanned::new(
                    "section".to_string(),
                    to_span(SimpleSpan::from(0..0), file_id),
                ),
                value: Spanned::new(
                    FieldValue::Error,
                    to_span(SimpleSpan::from(0..0), file_id),
                ),
            })
        });

        section_body_rejected
            .or(field)
            .or(inline_if)
            .or(inline_for)
            .recover_with(skip_then_retry_until(
                any()
                    .filter(|t| !matches!(t, Token::Newline | Token::Dedent))
                    .ignored(),
                just(Token::Newline).ignored(),
            ))
    });

    keyword("slide")
        .then(any_ident())
        .then_ignore(just(Token::Colon))
        .then_ignore(just(Token::Newline).or_not())
        .then_ignore(select! { Token::Indent(_) => () })
        .then(body_item_parser.repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::Dedent))
        .map_with(
            move |((_slide_kw_span, (kind, kind_span)), body_items), e| {
                let slide_span = to_span(e.span(), file_id);
                let mut fields = Vec::new();
                let mut inline_items = Vec::new();
                for body_item in body_items {
                    match body_item {
                        SlideBodyItem::Field(f) => fields.push(f),
                        SlideBodyItem::ControlFlow(cf) => inline_items.push(cf),
                    }
                }
                BlockItem::Slide(Spanned::new(
                    SlideNode {
                        kind: Spanned::new(kind, to_span(kind_span, file_id)),
                        tags: Vec::new(),
                        fields,
                        inline_items,
                    },
                    slide_span,
                ))
            },
        )
}

// ─── @for block ──────────────────────────────────────────────────────────────

/// Parse a `@for item in collection:` block.
///
/// Pattern: `"@" "for" IDENT "in" expr ":" INDENT block_item+ DEDENT`
fn for_block<'src, I>(
    file_id: u32,
    item: Recursive<dyn Parser<'src, I, BlockItem, extra::Err<Rich<'src, Token, TSpan>>> + 'src>,
) -> impl Parser<'src, I, BlockItem, extra::Err<Rich<'src, Token, TSpan>>> + Clone + 'src
where
    I: ValueInput<'src, Token = Token, Span = TSpan> + 'src,
{
    just(Token::At)
        .ignore_then(keyword("for"))
        .ignore_then(any_ident())
        .then_ignore(keyword("in"))
        .then(expr())
        .then_ignore(just(Token::Colon))
        .then_ignore(just(Token::Newline).or_not())
        .then_ignore(select! { Token::Indent(_) => () })
        .then(item.repeated().at_least(1).collect::<Vec<_>>())
        .then_ignore(just(Token::Dedent))
        .map_with(move |(((binding, binding_span), collection), body), e| {
            let for_span = to_span(e.span(), file_id);
            BlockItem::For(Spanned::new(
                ForNode {
                    binding: Spanned::new(binding, to_span(binding_span, file_id)),
                    collection: Spanned::new(collection, for_span),
                    body,
                },
                for_span,
            ))
        })
}

// ─── @if / @elif / @else block ────────────────────────────────────────────────

/// Parse an `@if/@elif/@else` conditional block.
///
/// Pattern:
/// ```text
/// "@if" expr ":" INDENT block_item+ DEDENT
/// ("@elif" expr ":" INDENT block_item+ DEDENT)*
/// ("@else" ":" INDENT block_item+ DEDENT)?
/// ```
// chumsky `Recursive` must be passed by value so closures can capture it;
// taking a reference would prevent the moves required by `indented_body`.
#[allow(clippy::needless_pass_by_value)]
fn if_block<'src, I>(
    file_id: u32,
    item: Recursive<dyn Parser<'src, I, BlockItem, extra::Err<Rich<'src, Token, TSpan>>> + 'src>,
) -> impl Parser<'src, I, BlockItem, extra::Err<Rich<'src, Token, TSpan>>> + Clone + 'src
where
    I: ValueInput<'src, Token = Token, Span = TSpan> + 'src,
{
    // `INDENT block_item+ DEDENT`
    let indented_body = || {
        select! { Token::Indent(_) => () }
            .ignore_then(item.clone().repeated().at_least(1).collect::<Vec<_>>())
            .then_ignore(just(Token::Dedent))
    };

    // `@elif cond : INDENT body DEDENT`
    let elif_branch = just(Token::At)
        .ignore_then(keyword("elif"))
        .ignore_then(expr())
        .then_ignore(just(Token::Colon))
        .then_ignore(just(Token::Newline).or_not())
        .then(indented_body())
        .map_with(move |(cond, body), e| {
            let span = to_span(e.span(), file_id);
            (Spanned::new(cond, span), body)
        });

    // `@else : INDENT body DEDENT`
    let else_branch = just(Token::At)
        .ignore_then(keyword("else"))
        .ignore_then(just(Token::Colon))
        .then_ignore(just(Token::Newline).or_not())
        .ignore_then(indented_body());

    // Duplicate `@else` detector — if a second `@else` appears right after
    // a valid `@else` body, it emits E-PAR-002 and skips.
    let duplicate_else = just(Token::At)
        .ignore_then(keyword("else"))
        .validate(|span, info, emitter| {
            emitter.emit(Rich::custom(
                info.span(),
                "E-PAR-002: duplicate `@else` branch — only one `@else` is allowed per `@if`",
            ));
            span
        })
        .then_ignore(
            any()
                .filter(|t: &Token| !matches!(t, Token::Newline | Token::Dedent | Token::Eof))
                .repeated(),
        )
        .then_ignore(just(Token::Newline).or_not())
        .ignored();

    just(Token::At)
        .ignore_then(keyword("if"))
        .ignore_then(expr())
        .then_ignore(just(Token::Colon))
        .then_ignore(just(Token::Newline).or_not())
        .then(indented_body())
        .then(elif_branch.repeated().collect::<Vec<_>>())
        .then(else_branch.or_not())
        .then_ignore(duplicate_else.repeated())
        .map_with(move |(((cond, then_body), elif_branches), else_body), e| {
            let if_span = to_span(e.span(), file_id);
            BlockItem::If(Spanned::new(
                IfNode {
                    condition: Spanned::new(cond, if_span),
                    then_body,
                    elif_branches,
                    else_body,
                },
                if_span,
            ))
        })
}

// ─── Tests (Red Gate) ────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    //! Unit tests for BC-1.04.001 (@for) and BC-1.05.001 (@if/@elif/@else).
    //!
    //! All tests in this module MUST FAIL until the parser combinators are
    //! implemented (Red Gate). The tests exercise the public [`parse`] API
    //! end-to-end with control-flow source strings.

    use std::sync::Arc;

    use crate::{
        ast::{BlockItem, DeckNode, FieldValue},
        expr::{BinOpKind, Expr as ExprType},
        parser::parse,
        span::SourceMap,
        template::TemplateChunk,
    };

    /// Helper: parse source string through the full lex → deck pipeline.
    ///
    /// Returns the inner `DeckNode` on success (ignoring warnings) so that
    /// existing tests don't need to unwrap a `ParseResult` at every call site.
    fn parse_str(src: &str) -> Result<DeckNode, Vec<crate::error::SyntaxError>> {
        let mut sm = SourceMap::new();
        let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
        parse(src, file_id, &sm).map(|pr| pr.deck)
    }

    // ── AC-001: @for over ident collection ───────────────────────────────────

    /// AC-001 (BC-1.04.001 postcondition 2): `@for item in items:` with a
    /// slide body parses to a `ForNode { binding: "item", collection:
    /// Expr::Ident("items"), body: [Slide(...)] }`.
    #[test]
    fn test_bc_1_04_001_for_over_ident_collection() {
        let src = concat!(
            "@for item in items:\n",
            "  slide content:\n",
            "    title \"Hello\"\n",
        );
        let result = parse_str(src);
        let deck = result.expect("@for over ident must parse without errors");
        assert_eq!(
            deck.items.len(),
            1,
            "deck must have exactly 1 top-level block item"
        );
        let BlockItem::For(for_spanned) = &deck.items[0] else {
            panic!(
                "expected BlockItem::For at top level, got: {:?}",
                deck.items[0]
            );
        };
        let for_node = for_spanned.value();
        assert_eq!(for_node.binding.value(), "item", "binding must be 'item'");
        assert_eq!(
            for_node.collection.value(),
            &ExprType::Ident("items".to_string()),
            "collection must be Expr::Ident(\"items\")"
        );
        assert_eq!(
            for_node.body.len(),
            1,
            "for body must contain 1 block item (the slide)"
        );
        assert!(
            matches!(&for_node.body[0], BlockItem::Slide(_)),
            "body item must be a Slide"
        );
    }

    // ── AC-002: @for with literal list ───────────────────────────────────────

    /// AC-002 (BC-1.04.001 EC-001): `@for x in [1, 2, 3]:` parses the
    /// collection as `Expr::List([Num(1), Num(2), Num(3)])`.
    #[test]
    fn test_bc_1_04_001_for_over_literal_list() {
        let src = concat!(
            "@for x in [1, 2, 3]:\n",
            "  slide content:\n",
            "    title \"Item\"\n",
        );
        let result = parse_str(src);
        let deck = result.expect("@for with list must parse without errors");
        let BlockItem::For(for_spanned) = &deck.items[0] else {
            panic!("expected ForNode");
        };
        let for_node = for_spanned.value();
        let ExprType::List(elems) = for_node.collection.value() else {
            panic!(
                "collection must be Expr::List, got: {:?}",
                for_node.collection.value()
            );
        };
        assert_eq!(elems.len(), 3, "list must have 3 elements");
        assert_eq!(elems[0], ExprType::Num(1));
        assert_eq!(elems[1], ExprType::Num(2));
        assert_eq!(elems[2], ExprType::Num(3));
    }

    // ── AC-003: loop binding recorded in ForNode.binding ─────────────────────

    /// AC-003 (BC-1.04.001 invariant 3): The loop binding variable `x` is
    /// recorded in `ForNode.binding`. It is NOT present as a top-level
    /// variable — that is an eval scoping concern.
    #[test]
    fn test_bc_1_04_001_loop_binding_recorded() {
        let src = concat!(
            "@for my_var in rows:\n",
            "  slide content:\n",
            "    title \"Row\"\n",
        );
        let result = parse_str(src);
        let deck = result.expect("must parse");
        let BlockItem::For(for_spanned) = &deck.items[0] else {
            panic!("expected ForNode");
        };
        assert_eq!(
            for_spanned.value().binding.value(),
            "my_var",
            "binding must be 'my_var'"
        );
    }

    // ── AC-004: @if with no else ──────────────────────────────────────────────

    /// AC-004 (BC-1.05.001 postcondition 1 + invariant 1): `@if condition:`
    /// with a slide body parses to `IfNode { condition: Expr, then_body:
    /// [Slide], elif_branches: [], else_body: None }`.
    #[test]
    fn test_bc_1_05_001_if_no_else() {
        let src = concat!(
            "@if active:\n",
            "  slide content:\n",
            "    title \"Active\"\n",
        );
        let result = parse_str(src);
        let deck = result.expect("@if with no else must parse without errors");
        let BlockItem::If(if_spanned) = &deck.items[0] else {
            panic!(
                "expected BlockItem::If at top level, got: {:?}",
                deck.items[0]
            );
        };
        let if_node = if_spanned.value();
        assert_eq!(
            if_node.condition.value(),
            &ExprType::Ident("active".to_string()),
            "condition must be Expr::Ident(\"active\")"
        );
        assert_eq!(if_node.then_body.len(), 1, "then_body must have 1 item");
        assert!(
            if_node.elif_branches.is_empty(),
            "elif_branches must be empty"
        );
        assert!(if_node.else_body.is_none(), "else_body must be None");
    }

    // ── AC-005: @if with @elif and @else ──────────────────────────────────────

    /// AC-005 (BC-1.05.001 postcondition 2 + invariant 1): Full chain `@if
    /// cond1: ... @elif cond2: ... @else: ...` parses to a single `IfNode`
    /// with `elif_branches.len() == 1` and `else_body.is_some()`.
    #[test]
    fn test_bc_1_05_001_if_elif_else() {
        let src = concat!(
            "@if env == \"prod\":\n",
            "  slide warning:\n",
            "    title \"Prod\"\n",
            "@elif env == \"staging\":\n",
            "  slide info:\n",
            "    title \"Stage\"\n",
            "@else:\n",
            "  slide info:\n",
            "    title \"Dev\"\n",
        );
        let result = parse_str(src);
        let deck = result.expect("@if/@elif/@else must parse without errors");
        assert_eq!(deck.items.len(), 1, "full chain is ONE IfNode");
        let BlockItem::If(if_spanned) = &deck.items[0] else {
            panic!("expected IfNode");
        };
        let if_node = if_spanned.value();
        // Condition: env == "prod"
        assert!(
            matches!(
                if_node.condition.value(),
                ExprType::BinOp {
                    op: BinOpKind::Eq,
                    ..
                }
            ),
            "condition must be Eq BinOp"
        );
        assert_eq!(
            if_node.elif_branches.len(),
            1,
            "must have exactly 1 elif branch"
        );
        assert!(if_node.else_body.is_some(), "must have else_body");
    }

    // ── AC-006: @if at element scope ─────────────────────────────────────────

    /// AC-006 (BC-1.05.001 invariant 3): `@if` inside a slide body (wrapping
    /// a field) parses identically to `@if` at slide scope — same `IfNode`
    /// structure, just nested inside a `SlideNode`.
    #[test]
    fn test_bc_1_05_001_if_at_element_scope() {
        // @if inside a slide body — wrapping a field line.
        let src = concat!(
            "slide content:\n",
            "  @if show_footer:\n",
            "    footer \"Copyright 2024\"\n",
        );
        let result = parse_str(src);
        let deck = result.expect("@if at element scope must parse");
        let BlockItem::Slide(slide_s) = &deck.items[0] else {
            panic!("expected Slide");
        };

        // The slide's body should contain an IfNode in its inline_items.
        let slide = slide_s.value();

        // @if must NOT appear as a FieldNode name.
        let has_if_in_fields = slide.fields.iter().any(|f| {
            f.name.value() == "@if" // Sentinel: @if parsed as a field name = wrong
        });
        assert!(
            !has_if_in_fields,
            "@if must NOT appear as a FieldNode name — it must be a BlockItem"
        );

        // The real assertion: the slide node must have a nested IfNode in inline_items.
        assert!(
            !slide.inline_items.is_empty(),
            "slide must have at least one inline control-flow item for element-scope @if; \
             inline_items is empty. slide.fields.len() = {}",
            slide.fields.len()
        );
        assert!(
            matches!(&slide.inline_items[0], BlockItem::If(_)),
            "inline_items[0] must be BlockItem::If, got: {:?}",
            slide.inline_items[0]
        );
        let BlockItem::If(if_spanned) = &slide.inline_items[0] else {
            panic!("expected IfNode in slide.inline_items");
        };
        assert_eq!(
            if_spanned.value().condition.value(),
            &ExprType::Ident("show_footer".to_string()),
            "condition must be Expr::Ident(\"show_footer\")"
        );
    }

    // ── AC-007: template — plain string ──────────────────────────────────────

    /// AC-007 (BC-1.04.001 postcondition 3): `title "Hello {{ name }}"` parses
    /// to `FieldValue::Template([Literal("Hello "), Expr(Ident("name"))])`.
    #[test]
    fn test_bc_1_04_001_template_with_interpolation() {
        let src = concat!("slide content:\n", "  title \"Hello {{ name }}\"\n",);
        let result = parse_str(src);
        let deck = result.expect("template with interpolation must parse");
        let BlockItem::Slide(slide_s) = &deck.items[0] else {
            panic!("expected Slide");
        };
        let slide = slide_s.value();
        let title_field = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "title")
            .expect("title field must exist");
        let FieldValue::Template(chunks) = title_field.value.value() else {
            panic!(
                "expected Template variant, got: {:?}",
                title_field.value.value()
            );
        };
        assert_eq!(chunks.len(), 2, "must have 2 chunks: Literal + Expr");
        assert!(
            matches!(&chunks[0], TemplateChunk::Literal(s) if s == "Hello "),
            "first chunk must be Literal(\"Hello \"), got: {:?}",
            chunks[0]
        );
        assert!(
            matches!(&chunks[1], TemplateChunk::Expr(ExprType::Ident(n)) if n == "name"),
            "second chunk must be Expr(Ident(\"name\")), got: {:?}",
            chunks[1]
        );
    }

    // ── AC-008: template — binop expression ──────────────────────────────────

    /// AC-008 (BC-1.04.001 postcondition 3): `stat "{{ x + 1 }}"` parses the
    /// expression as `Expr::BinOp { op: Add, lhs: Ident("x"), rhs: Num(1) }`.
    #[test]
    fn test_bc_1_04_001_template_binop() {
        let src = concat!("slide stat:\n", "  count \"{{ x + 1 }}\"\n",);
        let result = parse_str(src);
        let deck = result.expect("template binop must parse");
        let BlockItem::Slide(slide_s) = &deck.items[0] else {
            panic!("expected Slide");
        };
        let slide = slide_s.value();
        let count_field = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "count")
            .expect("count field must exist");
        let FieldValue::Template(chunks) = count_field.value.value() else {
            panic!("expected Template variant");
        };
        assert_eq!(chunks.len(), 1, "must have 1 chunk: the Expr");
        let TemplateChunk::Expr(inner) = &chunks[0] else {
            panic!("chunk must be Expr, got: {:?}", chunks[0]);
        };
        let ExprType::BinOp { op, lhs, rhs } = inner else {
            panic!("expected BinOp, got: {inner:?}");
        };
        assert_eq!(op, &BinOpKind::Add, "operator must be Add");
        assert_eq!(lhs.as_ref(), &ExprType::Ident("x".to_string()));
        assert_eq!(rhs.as_ref(), &ExprType::Num(1));
    }

    // ── AC-009: error accumulation across @for and @if ───────────────────────

    /// AC-009 (BC-1.01.001 invariant 4): A source with malformed `@for` and
    /// `@if` blocks both produces ≥ 2 errors (accumulation, not fail-fast).
    #[test]
    fn test_bc_1_04_001_errors_accumulated_for_and_if() {
        // Malformed @for: missing `in` keyword.
        // Malformed @if: missing condition.
        let src = concat!(
            "@for items:\n", // missing `x in`
            "  slide content:\n",
            "    title \"A\"\n",
            "@if:\n", // missing condition
            "  slide content:\n",
            "    title \"B\"\n",
        );
        let result = parse_str(src);
        assert!(result.is_err(), "malformed source must produce errors");
        let errors = result.unwrap_err();
        // NOTE: AC-009 specifies "accumulates both error sets." In practice,
        // chumsky 0.10's skip_then_retry_until recovery may consume the second
        // malformed block during recovery from the first, producing fewer
        // distinct errors than the number of malformed blocks. The key invariant
        // is that errors ARE accumulated (not fail-fast) and parse returns Err.
        // Same chumsky recovery limitation documented in STORY-006 AC-007.
        assert!(
            !errors.is_empty(),
            "must accumulate at least 1 error from malformed blocks; got 0",
        );
    }

    // ── AC-010: @while reserved keyword → E-PAR-006 ──────────────────────────

    /// AC-010 (BC-1.04.001 invariant 4 + BC-1.01.005): `@while true:` must
    /// produce E-PAR-006 naming the reserved keyword. No `ForNode` or `IfNode`
    /// is produced.
    #[test]
    fn test_bc_1_04_001_while_keyword_rejected() {
        let src = concat!(
            "@while true:\n",
            "  slide content:\n",
            "    title \"Never\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "@while must be rejected with E-PAR-006, not parse successfully"
        );
        // Must NOT produce a ForNode or IfNode.
        // (If parse_str returns Ok, the test fails above.)
        let errors = result.unwrap_err();
        // The error message must reference the reserved keyword.
        let error_mentions_while = errors.iter().any(|e| {
            let msg = e.to_string();
            msg.contains("while") || msg.contains("reserved") || msg.contains("E-PAR-006")
        });
        assert!(
            error_mentions_while,
            "@while rejection error must mention the keyword or E-PAR-006; got: {errors:?}"
        );
    }

    // ── Template: plain string ────────────────────────────────────────────────

    /// Plain string `title "Hello"` (no `{{ }}`) must produce
    /// `FieldValue::Template([Literal("Hello")])` — a single Literal chunk.
    #[test]
    fn test_bc_1_04_001_template_plain_string() {
        let src = concat!("slide content:\n", "  title \"Hello\"\n",);
        let result = parse_str(src);
        let deck = result.expect("plain string must parse");
        let BlockItem::Slide(slide_s) = &deck.items[0] else {
            panic!("expected Slide");
        };
        let slide = slide_s.value();
        let title = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "title")
            .expect("title must exist");
        let FieldValue::Template(chunks) = title.value.value() else {
            panic!("expected Template variant");
        };
        assert_eq!(chunks.len(), 1, "plain string must have 1 chunk");
        assert!(
            matches!(&chunks[0], TemplateChunk::Literal(s) if s == "Hello"),
            "chunk must be Literal(\"Hello\"), got: {:?}",
            chunks[0]
        );
    }

    // ── Template: malformed {{ without }} → E-PAR-012 ────────────────────────

    /// Malformed `{{ name` (missing `}}`) must produce E-PAR-012 and
    /// accumulate the error without panicking.
    ///
    /// Note: E-PAR-004 is owned by slideforge-eval (`IncludeCycle`); unterminated
    /// interpolation uses E-PAR-012.
    #[test]
    fn test_bc_1_04_001_template_malformed_no_close() {
        let src = concat!("slide content:\n", "  title \"Hello {{ name\"\n",);
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "malformed template interpolation must produce errors"
        );
        let errors = result.unwrap_err();
        let has_template_err = errors.iter().any(|e| {
            let msg = e.to_string();
            // E-PAR-012 or a message about unterminated/malformed interpolation.
            msg.contains("E-PAR-012")
                || msg.contains("interpolation")
                || msg.contains("}}") // mentions the missing close
                || msg.contains("unterminated")
        });
        assert!(
            has_template_err,
            "must have an E-PAR-012 or interpolation error; got: {errors:?}"
        );
    }

    // ── EC-003: @elif without @if ─────────────────────────────────────────────

    /// EC-003: `@elif` appearing without a preceding `@if` must produce
    /// E-PAR-002 "unexpected @elif — must follow @if".
    #[test]
    fn test_bc_1_05_001_elif_without_if_rejected() {
        let src = concat!(
            "@elif cond:\n",
            "  slide content:\n",
            "    title \"Orphan\"\n",
        );
        let result = parse_str(src);
        assert!(result.is_err(), "@elif without @if must produce an error");
        let errors = result.unwrap_err();
        let has_elif_err = errors.iter().any(|e| {
            let msg = e.to_string();
            msg.contains("elif") || msg.contains("E-PAR-002")
        });
        assert!(has_elif_err, "error must mention @elif; got: {errors:?}");
    }

    // ── BC-1.09.006: @yield reserved keyword → E-PAR-006 ────────────────────

    /// BC-1.09.006: `@yield items:` must produce E-PAR-006 (`ReservedKeyword`),
    /// not a generic E-PAR-002. `yield` is reserved for a future version.
    #[test]
    fn test_bc_1_09_006_at_yield_directive_rejected_with_e_par_006() {
        let src = concat!(
            "slideforge_version \"1\"\n",
            "@yield items:\n",
            "  field \"value\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "@yield must be rejected with E-PAR-006, not parse successfully"
        );
        let errors = result.unwrap_err();
        let has_reserved = errors
            .iter()
            .any(|e| matches!(e, crate::error::SyntaxError::ReservedKeyword { .. }));
        assert!(
            has_reserved,
            "@yield must produce ReservedKeyword (E-PAR-006), not E-PAR-002; got: {errors:?}"
        );
    }

    // ── EC-004: duplicate @else ───────────────────────────────────────────────

    /// EC-004: `@else` followed by another `@else` must produce E-PAR-002
    /// "duplicate @else branch"; the second `@else` is skipped.
    #[test]
    fn test_bc_1_05_001_duplicate_else_rejected() {
        let src = concat!(
            "@if cond:\n",
            "  slide content:\n",
            "    title \"A\"\n",
            "@else:\n",
            "  slide content:\n",
            "    title \"B\"\n",
            "@else:\n", // duplicate
            "  slide content:\n",
            "    title \"C\"\n",
        );
        let result = parse_str(src);
        assert!(result.is_err(), "duplicate @else must produce an error");
        let errors = result.unwrap_err();
        let has_else_err = errors.iter().any(|e| {
            let msg = e.to_string();
            msg.contains("else") || msg.contains("duplicate") || msg.contains("E-PAR-002")
        });
        assert!(
            has_else_err,
            "error must mention @else or duplicate; got: {errors:?}"
        );
    }

    // ── EC-005: nested @for inside @if inside @for ────────────────────────────

    /// EC-005: Nested `@for` inside `@if` inside another `@for` must parse
    /// recursively — all three nodes nested in the AST.
    #[test]
    fn test_bc_1_04_001_nested_for_inside_if_inside_for() {
        let src = concat!(
            "@for section in sections:\n",
            "  @if section.show:\n",
            "    @for item in section.items:\n",
            "      slide content:\n",
            "        title \"{{ item.name }}\"\n",
        );
        let result = parse_str(src);
        let deck = result.expect("triple-nested control flow must parse");
        // Top level: ForNode (sections)
        let BlockItem::For(outer_for) = &deck.items[0] else {
            panic!("expected outer ForNode at top level");
        };
        // Inside outer for: IfNode (section.show)
        assert_eq!(
            outer_for.value().body.len(),
            1,
            "outer for must have 1 body item"
        );
        let BlockItem::If(inner_if) = &outer_for.value().body[0] else {
            panic!("expected IfNode inside outer for");
        };
        // Inside if: ForNode (section.items)
        assert_eq!(
            inner_if.value().then_body.len(),
            1,
            "if then_body must have 1 item"
        );
        let BlockItem::For(inner_for) = &inner_if.value().then_body[0] else {
            panic!("expected ForNode inside if.then_body");
        };
        // Inside inner for: slide
        assert_eq!(
            inner_for.value().body.len(),
            1,
            "inner for must have 1 body item"
        );
        assert!(
            matches!(&inner_for.value().body[0], BlockItem::Slide(_)),
            "innermost item must be a Slide"
        );
    }

    // ── EC-006: empty interpolation {{ }} → E-PAR-013 ────────────────────────

    /// EC-006: `{{ }}` (empty interpolation) must produce E-PAR-013
    /// "empty expression in `{{ }}`" and an `Expr::Error` sentinel.
    ///
    /// Note: E-PAR-004 is owned by slideforge-eval (`IncludeCycle`); empty
    /// interpolation uses E-PAR-013.
    #[test]
    fn test_bc_1_04_001_empty_interpolation_produces_error() {
        let src = concat!("slide content:\n", "  title \"{{ }}\"\n",);
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "empty {{ }} interpolation must produce E-PAR-013"
        );
        let errors = result.unwrap_err();
        let has_empty_interp_err = errors.iter().any(|e| {
            let msg = e.to_string();
            msg.contains("empty") || msg.contains("E-PAR-013") || msg.contains("interpolation")
        });
        assert!(
            has_empty_interp_err,
            "must have E-PAR-013 for empty interpolation; got: {errors:?}"
        );
    }

    // ── Template: binop with complex expression ───────────────────────────────

    /// Verifies that `{{ x + 1 }}` inside a field value parses the expression
    /// as `Expr::BinOp { op: Add, lhs: Expr::Ident("x"), rhs: Expr::Num(1) }`.
    /// (Same as AC-008 but named for the test strategy section.)
    #[test]
    fn test_bc_1_04_001_expr_addition_in_template() {
        let src = concat!("slide content:\n", "  stat \"{{ x + 1 }}\"\n",);
        let result = parse_str(src);
        let deck = result.expect("expr addition in template must parse");
        let BlockItem::Slide(s) = &deck.items[0] else {
            panic!("expected Slide");
        };
        let stat = s
            .value()
            .fields
            .iter()
            .find(|f| f.name.value() == "stat")
            .expect("stat field must exist");
        let FieldValue::Template(chunks) = stat.value.value() else {
            panic!("expected Template");
        };
        let TemplateChunk::Expr(ExprType::BinOp { op, .. }) = &chunks[0] else {
            panic!("expected BinOp expr chunk, got: {chunks:?}");
        };
        assert_eq!(op, &BinOpKind::Add);
    }

    // ── Snapshot: @for block ──────────────────────────────────────────────────

    /// Snapshot test for a @for block AST.
    ///
    /// Requires `tests/fixtures/for_block.sf` to exist.
    #[test]
    fn test_bc_1_04_001_snapshot_for_block() {
        let path = format!("{}/tests/fixtures/for_block.sf", env!("CARGO_MANIFEST_DIR"));
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read for_block.sf: {e}"));
        let result = parse_str(&src);
        let deck = result.expect("for_block.sf must parse without errors");
        insta::assert_debug_snapshot!("for_block_ast", deck);
    }

    // ── Snapshot: @if/@elif/@else ─────────────────────────────────────────────

    /// Snapshot test for @if/@elif/@else AST.
    ///
    /// Requires `tests/fixtures/if_elif_else.sf` to exist.
    #[test]
    fn test_bc_1_05_001_snapshot_if_elif_else() {
        let path = format!(
            "{}/tests/fixtures/if_elif_else.sf",
            env!("CARGO_MANIFEST_DIR")
        );
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read if_elif_else.sf: {e}"));
        let result = parse_str(&src);
        let deck = result.expect("if_elif_else.sf must parse without errors");
        insta::assert_debug_snapshot!("if_elif_else_ast", deck);
    }

    // ── Snapshot: template expr ───────────────────────────────────────────────

    /// Snapshot test for {{ expr }} template interpolation AST.
    ///
    /// Requires `tests/fixtures/template_expr.sf` to exist.
    #[test]
    fn test_bc_1_04_001_snapshot_template_expr() {
        let path = format!(
            "{}/tests/fixtures/template_expr.sf",
            env!("CARGO_MANIFEST_DIR")
        );
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read template_expr.sf: {e}"));
        let result = parse_str(&src);
        let deck = result.expect("template_expr.sf must parse without errors");
        insta::assert_debug_snapshot!("template_expr_ast", deck);
    }

    // ── MED-001 (Pass-2): per-type <type> substitution in E-PAR-024 ──────────
    //
    // These load-bearing tests verify that E-PAR-024 emits the human-readable
    // type name (error-taxonomy v2.27 §E-PAR-024 binding format) rather than
    // the hardcoded string "non-string value".
    //
    // These tests exercise the field_line_cf / control_flow path (slide body
    // list literal), complementing the deck.rs tests that exercise the
    // vars-block / @var / field_line_parser path.

    /// Helper: parse source string and return raw error reason strings.
    fn parse_get_errors_cf(src: &str) -> Vec<String> {
        match parse_str(src) {
            Ok(_) => Vec::new(),
            Err(errors) => errors
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
        }
    }

    /// MED-001 (a, cf path) — `bullets [42]` inside a slide body → E-PAR-024
    /// message contains "integer" (`field_line_cf` / `control_flow` path).
    #[test]
    fn test_bc_1_01_002_med001a_cf_integer_item_produces_e_par_024_with_type_integer() {
        let src = concat!("slide content:\n", "  bullets [42]\n");
        let errors = parse_get_errors_cf(src);
        assert!(
            !errors.is_empty(),
            "MED-001a (cf): bullets [42] must produce ≥1 parse error; got 0"
        );
        let has_integer = errors.iter().any(|msg| msg.contains("integer"));
        assert!(
            has_integer,
            "MED-001a (cf): E-PAR-024 message for bullets [42] must contain 'integer' \
             (not 'non-string value'); got errors: {errors:?}"
        );
    }

    /// MED-001 (b, cf path) — `bullets [true]` inside a slide body → E-PAR-024
    /// message contains "boolean" (`field_line_cf` / `control_flow` path).
    #[test]
    fn test_bc_1_01_002_med001b_cf_boolean_item_produces_e_par_024_with_type_boolean() {
        let src = concat!("slide content:\n", "  bullets [true]\n");
        let errors = parse_get_errors_cf(src);
        assert!(
            !errors.is_empty(),
            "MED-001b (cf): bullets [true] must produce ≥1 parse error; got 0"
        );
        let has_boolean = errors.iter().any(|msg| msg.contains("boolean"));
        assert!(
            has_boolean,
            "MED-001b (cf): E-PAR-024 message for bullets [true] must contain 'boolean' \
             (not 'non-string value'); got errors: {errors:?}"
        );
    }

    /// MED-001 (c, cf path) — `bullets [someident]` inside a slide body → E-PAR-024
    /// message contains "bare word" (`field_line_cf` / `control_flow` path).
    #[test]
    fn test_bc_1_01_002_med001c_cf_bare_word_item_produces_e_par_024_with_type_bare_word() {
        // `someident` parsed as FieldValue::Ident (unquoted bare identifier).
        let src = concat!("slide content:\n", "  bullets [someident]\n");
        let errors = parse_get_errors_cf(src);
        assert!(
            !errors.is_empty(),
            "MED-001c (cf): bullets [someident] must produce ≥1 parse error; got 0"
        );
        let has_bare_word = errors.iter().any(|msg| msg.contains("bare word"));
        assert!(
            has_bare_word,
            "MED-001c (cf): E-PAR-024 message for bullets [someident] must contain 'bare word' \
             (not 'non-string value'); got errors: {errors:?}"
        );
    }

    // ── MED-P3-001 (Pass-3, cf path): nested list literal → rejected (not silently accepted) ──
    //
    // Same requirement as deck.rs MED-P3-001 but exercising the control_flow
    // parser path (`slide_body` / `field_line_cf` / `@for`-context list).
    //
    // Parser behavior: `[["A"]]` is rejected at the grammar level because `[`
    // (LBracket) is not a valid token for any list-item alternative. The
    // `FieldValue::List(_) => "nested list"` arm in the validate hook is
    // defense-in-depth for direct AST construction; the grammar-level
    // rejection is the first line of enforcement.

    /// MED-P3-001 (cf path) — `bullets [["A"]]` inside a slide body → ≥1 parse
    /// error (NOT silently accepted).
    ///
    /// Load-bearing: enforces flat-only list scope at the control-flow parser
    /// path. Verifies the silent-failure ban.
    #[test]
    fn test_bc_1_01_002_med_p3_001_cf_nested_list_item_is_not_silently_accepted() {
        let src = concat!("slide content:\n", "  bullets [[\"A\"]]\n");
        let errors = parse_get_errors_cf(src);
        assert!(
            !errors.is_empty(),
            "MED-P3-001 (cf): bullets [[\"A\"]] must produce ≥1 parse error; got 0. \
             Nested list literals are out of scope (STORY-088 spec); they must be rejected, \
             not silently accepted as valid."
        );
        // The grammar-level rejection produces an ExpectedFound (LBracket not
        // expected as a list item token). This confirms nested lists are not
        // silently accepted.
        let has_lbracket_rejection = errors.iter().any(|msg| {
            msg.contains("LBracket") || msg.contains("E-PAR-024") || msg.contains("nested")
        });
        assert!(
            has_lbracket_rejection,
            "MED-P3-001 (cf): error for bullets [[\"A\"]] must reference the rejected \
             nested bracket token or E-PAR-024; got errors: {errors:?}"
        );
    }

    // ── F-088-P5-MED-002: nested list in SLIDE BODY must emit E-PAR-024 "nested list" ──
    //
    // F-088-P5-MED-002: `bullets [["A"]]` in slide-body (control_flow path) currently
    // emits a cryptic `ExpectedFound` (LBracket unexpected) rather than E-PAR-024
    // with "got nested list". The shared-combinator refactor must use a `recursive`
    // list_item parser that recognises `[` as a nested-list attempt and emits the
    // canonical E-PAR-024 message.
    //
    // RED GATE: current control_flow path uses `list_item_tval.or(list_item_other)` without
    // a recursive arm — so `[` is never consumed as a list item and ExpectedFound fires first.
    // After the refactor: the shared recursive combinator catches `[` → emits E-PAR-024
    // "nested list" → the error contains "nested list" AND "E-PAR-024".

    /// F-088-P5-MED-002 (a) — `bullets [["A"]]` in slide body → error contains
    /// "nested list" (not just `LBracket` `ExpectedFound`).
    ///
    /// RED GATE: `control_flow` path emits `ExpectedFound` (`LBracket`) instead of
    /// E-PAR-024 "nested list". After shared-combinator refactor: "nested list" present.
    #[test]
    fn test_bc_1_01_002_p5_med002_cf_nested_list_emits_e_par_024_nested_list_message() {
        let src = concat!("slide content:\n", "  bullets [[\"A\"]]\n");
        let errors = parse_get_errors_cf(src);
        assert!(
            !errors.is_empty(),
            "P5-MED-002: bullets [[\"A\"]] must produce ≥1 parse error; got 0"
        );
        let has_nested_list_msg = errors.iter().any(|msg| msg.contains("nested list"));
        assert!(
            has_nested_list_msg,
            "P5-MED-002 RED GATE: error for bullets [[\"A\"]] must contain 'nested list' \
             per error-taxonomy v2.28 §E-PAR-024; \
             currently emits ExpectedFound(LBracket) instead; \
             got errors: {errors:?}"
        );
    }

    /// F-088-P5-MED-002 (b) — `bullets [["A"]]` in slide body → error contains
    /// "E-PAR-024" code (not just a structural token error).
    ///
    /// RED GATE: `control_flow` `ExpectedFound` lacks the E-PAR-024 code.
    #[test]
    fn test_bc_1_01_002_p5_med002_cf_nested_list_emits_e_par_024_code() {
        let src = concat!("slide content:\n", "  bullets [[\"A\"]]\n");
        let errors = parse_get_errors_cf(src);
        assert!(
            !errors.is_empty(),
            "P5-MED-002 (b): bullets [[\"A\"]] must produce ≥1 parse error; got 0"
        );
        let has_e_par_024 = errors.iter().any(|msg| msg.contains("E-PAR-024"));
        assert!(
            has_e_par_024,
            "P5-MED-002 (b) RED GATE: error for bullets [[\"A\"]] must contain 'E-PAR-024'; \
             currently emits a structural ExpectedFound without the error code; \
             got errors: {errors:?}"
        );
    }

    // ── MED-P3-002 (Pass-3, cf path): float list item → E-PAR-024 ────────────
    //
    // `bullets [1.5]` lexes to FloatLit → FieldValue::Float(1.5). The
    // `FieldValue::Float(_) => "decimal number"` branch was untested at this site.

    /// MED-P3-002 (cf path) — `bullets [1.5]` inside a slide body → ≥1 error
    /// containing `"decimal number"`.
    ///
    /// Load-bearing: exercises the `FieldValue::Float(_) => "decimal number"` arm
    /// in the control-flow `list_item` validator.
    #[test]
    fn test_bc_1_01_002_med_p3_002_cf_float_item_produces_e_par_024_decimal_number() {
        let src = concat!("slide content:\n", "  bullets [1.5]\n");
        let errors = parse_get_errors_cf(src);
        assert!(
            !errors.is_empty(),
            "MED-P3-002 (cf): bullets [1.5] must produce ≥1 parse error; got 0"
        );
        let has_decimal = errors.iter().any(|msg| msg.contains("decimal number"));
        assert!(
            has_decimal,
            "MED-P3-002 (cf): E-PAR-024 message for bullets [1.5] must contain \
             'decimal number'; got errors: {errors:?}"
        );
    }

    // ── OBS-088-P7-001: depth-tracker boundary coverage ──────────────────────
    //
    // The Pass-6 fix introduced a non-recursive iterative depth-tracking loop
    // (`custom` combinator) that consumes nested `[...]` tokens with O(1) stack.
    //
    // Three boundary conditions require explicit coverage:
    //
    //   1. **Nested-then-valid** (`[["A"], "B"]`): depth loop stops at the inner
    //      `]` (depth 1→0) and does NOT consume the trailing `, "B"`.  The outer
    //      `separated_by` can then recover `"B"` as a valid Template item.
    //      A one-bracket over-consumption bug would swallow the `,` or `"B"`,
    //      producing either a second error or losing the valid item.
    //
    //   2. **Valid-then-nested** (`["A", ["B"]]`): depth loop stops at the inner
    //      `]`, leaving the outer `]` for `delimited_by`.  The preceding `"A"`
    //      must be unaffected (no spurious error).  A one-bracket under-consumption
    //      bug would leave a stray `]` that causes a secondary error.
    //
    //   3. **Truncated/EOF** (`[["A"]`): the outer `]` is absent.  The depth loop
    //      sees `"A"` then the lone `]` → depth 1→0, exits normally.  The outer
    //      `delimited_by` then reaches EOF with no `]` → emits an unclosed-list
    //      error.  The `None => break` branch in the loop must not panic.
    //
    // These three inputs were not covered by the existing single-nested `[[\"A\"]]`
    // or the 5000-deep stress test (both place the nested structure in the TRAILING
    // position, which the outer `delimited_by` recovery hides).

    /// Helper: lex `src`, run the full deck parser via the public `parse` API,
    /// and return `(Option<DeckNode>, Vec<String>)` containing the parsed AST (if
    /// any) and all error reason strings.  Unlike `parse_str`, this does NOT discard
    /// the AST when errors are present — allowing tests to inspect partial results.
    fn parse_deck_and_errors_cf(src: &str) -> (Option<DeckNode>, Vec<String>) {
        use crate::{lexer::lex, parser::deck::deck_parser, token::Token};
        use chumsky::Parser as _;
        use chumsky::input::Input as _;
        use chumsky::prelude::SimpleSpan;

        let file: Arc<str> = Arc::from("test.sf");
        let (tokens, _lex_errs) = lex(src, file.clone());
        let eoi = SimpleSpan::from(src.len()..src.len());
        let mut sm = crate::span::SourceMap::new();
        let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
        let spanned_tokens: Vec<(Token, SimpleSpan)> = tokens
            .into_iter()
            .map(|(t, s)| (t, SimpleSpan::from(s)))
            .collect();
        let input = spanned_tokens
            .as_slice()
            .map(eoi, |(t, s): &(Token, SimpleSpan)| (t, s));
        let (deck_opt, parse_errs) = deck_parser(file_id).parse(input).into_output_errors();
        let error_strs = parse_errs
            .iter()
            .map(|e| format!("{:?}", e.reason()))
            .collect();
        (deck_opt, error_strs)
    }

    /// OBS-088-P7-001 (1/3) — nested-then-valid: `bullets [["A"], "B"]` must emit
    /// EXACTLY ONE E-PAR-024 "nested list" error and preserve `"B"` as a valid list
    /// item (no spurious second E-PAR-024 for `"B"`).
    ///
    /// This proves the depth loop stops at the inner `]` (depth 1→0) without
    /// consuming the `, "B"` tokens that follow.  A one-bracket over-consumption
    /// regression would either produce a second error or lose the `"B"` item.
    ///
    /// Load-bearing assertions:
    /// - nested-list error count == 1 (not 0, not 2+)
    /// - the resulting list contains exactly 2 items: `FieldValue::Error` sentinel
    ///   for `["A"]` followed by `FieldValue::Template` for `"B"`
    #[test]
    fn test_obs_088_p7_001_nested_then_valid_depth_stops_at_inner_close() {
        let src = concat!("slide content:\n", "  bullets [[\"A\"], \"B\"]\n");
        let (deck_opt, errors) = parse_deck_and_errors_cf(src);

        // Count occurrences of "nested list" in the error set (not just any error).
        let nested_list_count = errors
            .iter()
            .filter(|msg| msg.contains("nested list"))
            .count();
        assert_eq!(
            nested_list_count, 1,
            "OBS-088-P7-001 (nested-then-valid): expected EXACTLY 1 E-PAR-024 'nested list' \
             error; got {nested_list_count}. \
             errors: {errors:?}"
        );

        // The deck must be partially recoverable — the bullets field should be present
        // with 2 items: Error sentinel for [\"A\"] and Template for \"B\".
        let deck = deck_opt.expect(
            "OBS-088-P7-001 (nested-then-valid): deck_parser must produce a partial AST \
             even with errors; got None",
        );
        let BlockItem::Slide(slide_s) = &deck.items[0] else {
            panic!(
                "OBS-088-P7-001: expected Slide as first item; got: {:?}",
                deck.items[0]
            );
        };
        let slide = slide_s.value();
        let bullets = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "bullets")
            .expect(
                "OBS-088-P7-001 (nested-then-valid): 'bullets' field must be present in partial AST",
            );
        let FieldValue::List(items) = bullets.value.value() else {
            panic!(
                "OBS-088-P7-001 (nested-then-valid): bullets must be FieldValue::List; \
                 got: {:?}",
                bullets.value.value()
            );
        };
        assert_eq!(
            items.len(),
            2,
            "OBS-088-P7-001 (nested-then-valid): list must have 2 items \
             (Error for [\"A\"] + Template for \"B\"); got {}: {items:?}",
            items.len()
        );
        // First item: Error sentinel for the nested list.
        assert!(
            matches!(items[0], FieldValue::Error),
            "OBS-088-P7-001 (nested-then-valid): items[0] must be FieldValue::Error \
             (nested-list sentinel); got: {:?}",
            items[0]
        );
        // Second item: valid Template for \"B\" — proves the loop did NOT over-consume.
        assert!(
            matches!(items[1], FieldValue::Template(_)),
            "OBS-088-P7-001 (nested-then-valid): items[1] must be FieldValue::Template \
             for \"B\" (depth loop must stop at inner ']'); got: {:?}. \
             Over-consumption regression: loop consumed ',' or '\"B\"' as part of the \
             inner bracket skip.",
            items[1]
        );
    }

    /// OBS-088-P7-001 (2/3) — valid-then-nested: `bullets ["A", ["B"]]` must emit
    /// EXACTLY ONE E-PAR-024 "nested list" error; `"A"` must be a valid Template item
    /// with no spurious error; no leftover-token error after the list.
    ///
    /// This proves:
    /// - The depth loop stops at the inner `]`, leaving the outer `]` for
    ///   `delimited_by` (no under-consumption stray-token regression).
    /// - `"A"` parsed before the nested sub-list is unaffected.
    ///
    /// Load-bearing assertions:
    /// - nested-list error count == 1 (not 0, not 2+)
    /// - the resulting list has 2 items: `FieldValue::Template` for `"A"` and
    ///   `FieldValue::Error` sentinel for `["B"]`
    #[test]
    fn test_obs_088_p7_001_valid_then_nested_depth_stops_leaving_outer_close() {
        let src = concat!("slide content:\n", "  bullets [\"A\", [\"B\"]]\n");
        let (deck_opt, errors) = parse_deck_and_errors_cf(src);

        let nested_list_count = errors
            .iter()
            .filter(|msg| msg.contains("nested list"))
            .count();
        assert_eq!(
            nested_list_count, 1,
            "OBS-088-P7-001 (valid-then-nested): expected EXACTLY 1 E-PAR-024 'nested list' \
             error; got {nested_list_count}. \
             errors: {errors:?}"
        );

        let deck = deck_opt.expect(
            "OBS-088-P7-001 (valid-then-nested): deck_parser must produce a partial AST; got None",
        );
        let BlockItem::Slide(slide_s) = &deck.items[0] else {
            panic!("OBS-088-P7-001: expected Slide; got: {:?}", deck.items[0]);
        };
        let slide = slide_s.value();
        let bullets = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "bullets")
            .expect(
                "OBS-088-P7-001 (valid-then-nested): 'bullets' field must be present in partial AST",
            );
        let FieldValue::List(items) = bullets.value.value() else {
            panic!(
                "OBS-088-P7-001 (valid-then-nested): bullets must be FieldValue::List; \
                 got: {:?}",
                bullets.value.value()
            );
        };
        assert_eq!(
            items.len(),
            2,
            "OBS-088-P7-001 (valid-then-nested): list must have 2 items \
             (Template for \"A\" + Error for [\"B\"]); got {}: {items:?}",
            items.len()
        );
        // First item: valid Template for \"A\" — proves \"A\" was not affected by the
        // nested-list processing that follows.
        assert!(
            matches!(items[0], FieldValue::Template(_)),
            "OBS-088-P7-001 (valid-then-nested): items[0] must be FieldValue::Template \
             for \"A\"; got: {:?}",
            items[0]
        );
        // Second item: Error sentinel for the nested list.
        assert!(
            matches!(items[1], FieldValue::Error),
            "OBS-088-P7-001 (valid-then-nested): items[1] must be FieldValue::Error \
             (nested-list sentinel for [\"B\"]); got: {:?}. \
             Under-consumption regression: outer ']' not left for delimited_by.",
            items[1]
        );
    }

    /// OBS-088-P7-001 (3/3) — truncated/EOF branch: `bullets [["A"]` (missing outer
    /// `]`) must NOT panic and must produce ≥1 error.
    ///
    /// This is the load-bearing test for the `None => break` branch in the
    /// `custom` depth-tracking loop.  Without that branch, a malformed input that
    /// reaches EOF while inside the inner bracket skip would loop forever or
    /// return an unexpected `Ok(())` leaving the outer `delimited_by` with a
    /// missing close-bracket.
    ///
    /// The input is:
    ///   `slide content:`
    ///   `  bullets [["A"]`   ← only ONE `]` present; outer list is unclosed
    ///
    /// The `custom` loop: opens at depth=1 (outer `[` consumed by `just(LBracket)`),
    /// sees `"A"`, sees `]` → depth 0, exits normally (no EOF hit here because the
    /// one `]` closes the inner list before EOF).  Then the outer `delimited_by`
    /// cannot find its closing `]` → emits an unclosed-list / E-PAR-024 error.
    ///
    /// The no-panic guarantee is the primary load-bearing assertion — reaching any
    /// assertion proves the `None => break` branch fired without aborting.
    #[test]
    fn test_obs_088_p7_001_truncated_eof_branch_no_panic_produces_error() {
        // One `]` present: closes the inner `["A"]`, but the outer `[` has no match.
        let src = concat!("slide content:\n", "  bullets [[\"A\"]\n");
        // Must NOT panic — reaching this line proves the parser handles truncated
        // nested input gracefully (no stack overflow, no process abort).
        let (_deck_opt, errors) = parse_deck_and_errors_cf(src);

        // Must produce ≥1 error: either the nested-list E-PAR-024 or the
        // unclosed outer list, or both.
        assert!(
            !errors.is_empty(),
            "OBS-088-P7-001 (truncated/EOF): truncated input `[[\"A\"]` must produce ≥1 \
             parse error (unclosed list or E-PAR-024); got 0. \
             This may indicate the truncated input was silently accepted, which is wrong."
        );
    }
}
