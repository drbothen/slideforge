//! Parser combinator for `shape:` blocks.
//!
//! A `shape:` block is an indented sub-block inside a slide that describes a
//! free-form shape to be rendered on that slide.  It maps to a [`ShapeNode`]
//! stored as [`FieldValue::Shape`] inside a field node.
//!
//! # Grammar
//!
//! ```text
//! shape_block ::= "shape" ":" NEWLINE INDENT shape_field* DEDENT
//! shape_field  ::= ("type" | "position" | "fill" | "text" | "alt") STRING NEWLINE
//! ```
//!
//! All five recognized field names are optional at parse time.  Any unknown
//! field name inside a `shape:` block emits E-PAR-002 and is skipped (error
//! recovery).
//!
//! # STORY-009
//!
//! This module is new in STORY-009.  The `shape_block` combinator is registered
//! in `parser/slide.rs` so that `shape:` is accepted wherever a `FieldNode` can
//! appear inside a slide body.

use chumsky::{input::ValueInput, prelude::*};

use crate::{
    ast::{FieldValue, ShapeNode},
    span::{Span, Spanned},
    template::TemplateChunk,
    token::Token,
};

use super::template::template_value;

// ─── Type alias ───────────────────────────────────────────────────────────────

/// The span type used by chumsky in the token-stream parser.
type TSpan = SimpleSpan;

/// Convert a chumsky `SimpleSpan` plus a `file_id` into a project [`Span`].
///
/// Used by the shape block parser when constructing `Spanned<T>` nodes.
/// Declared here so the implementer can use it without re-deriving it.
fn to_span(ss: SimpleSpan, file_id: u32) -> Span {
    Span::new(file_id, ss.start, ss.end)
}

// ─── Internal types ───────────────────────────────────────────────────────────

/// The parsed result of a single recognized field inside a `shape:` block.
///
/// Used internally by [`shape_block`] to accumulate fields before constructing
/// the final [`ShapeNode`].
#[derive(Debug)]
enum ShapeFieldItem {
    /// `type "..."` — the shape category (rectangle, circle, etc.).
    Type(String, TSpan),
    /// `position "..."` — position/size descriptor (e.g. "50,50,200,100").
    Position(String, TSpan),
    /// `fill "..."` — fill color or image reference.
    Fill(String, TSpan),
    /// `text "..."` — text content, parsed as a template value.
    Text(Vec<crate::template::TemplateChunk>, TSpan),
    /// `alt "..."` — accessibility alt text.
    Alt(String, TSpan),
}

// ─── Public combinator ────────────────────────────────────────────────────────

/// Parse a `shape:` block into a [`FieldValue::Shape`] node.
///
/// Matches `"shape" ":" NEWLINE INDENT shape_field* DEDENT` and produces a
/// `(FieldValue::Shape(Box<ShapeNode>), TSpan)` pair for consumption by the
/// slide body parser.
///
/// # Error recovery
///
/// Unknown field names inside the block emit E-PAR-002 and are skipped; the
/// parser continues collecting recognized fields.
///
/// # STORY-009
///
/// Parses the five recognized shape fields:
/// `type`, `position`, `fill`, `text`, `alt` — all optional.
#[must_use]
pub fn shape_block<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, (FieldValue, TSpan), extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    // Match a keyword identifier.
    let kw = |name: &'static str| {
        select! { Token::Ident(s) = e if s.as_ref() == name => e.span() }
    };

    // Match any identifier (for skipping unknown field names).
    let any_ident = select! { Token::Ident(_) => () };

    // Match a string literal and return its content + span.
    let string_field = select! { Token::StringLit(s) = e => (s.to_string(), e.span()) };

    // A template string value (for "text" field).
    let template_val = template_value().validate(move |(chunks, errs), info, emitter| {
        for msg in errs {
            emitter.emit(Rich::custom(info.span(), msg));
        }
        (chunks, info.span())
    });

    let type_field = kw("type")
        .ignore_then(string_field)
        .then_ignore(just(Token::Newline).or_not())
        .map(|(s, sp)| ShapeFieldItem::Type(s, sp));

    let position_field = kw("position")
        .ignore_then(string_field)
        .then_ignore(just(Token::Newline).or_not())
        .map(|(s, sp)| ShapeFieldItem::Position(s, sp));

    let fill_field = kw("fill")
        .ignore_then(string_field)
        .then_ignore(just(Token::Newline).or_not())
        .map(|(s, sp)| ShapeFieldItem::Fill(s, sp));

    let text_field = kw("text")
        .ignore_then(template_val)
        .then_ignore(just(Token::Newline).or_not())
        .map(|(chunks, sp)| ShapeFieldItem::Text(chunks, sp));

    let alt_field = kw("alt")
        .ignore_then(string_field)
        .then_ignore(just(Token::Newline).or_not())
        .map(|(s, sp)| ShapeFieldItem::Alt(s, sp));

    // Raw keyword rejection inside shape: block — E-PAR-009.
    //
    // The `raw` escape hatch (`raw`, `raw_pptx`, `raw_html`, `raw_xml`,
    // `raw_docx`) is not available in user .sf files (AC-010). Reject it with
    // E-PAR-009 before the generic unknown_field path can produce E-PAR-002.
    let raw_in_shape = select! {
        Token::Ident(s) = e if matches!(
            s.as_ref(),
            "raw" | "raw_pptx" | "raw_html" | "raw_xml" | "raw_docx"
        ) => (s.to_string(), e.span())
    }
    .then(
        any()
            .filter(|t: &Token| !matches!(t, Token::Newline | Token::Dedent))
            .repeated(),
    )
    .then_ignore(just(Token::Newline).or_not())
    .validate(|((name, _span), _rest), info, emitter| {
        emitter.emit(Rich::custom(
            info.span(),
            format!(
                "E-PAR-009: '{name}' keyword is not available in user .sf files. \
                     Use the shape: DSL instead."
            ),
        ));
    })
    .map(|()| None::<ShapeFieldItem>);

    // Unknown field — skip the name + value + newline (error recovery).
    let unknown_field = any_ident
        .then(
            any()
                .filter(|t: &Token| !matches!(t, Token::Newline | Token::Dedent))
                .repeated(),
        )
        .then_ignore(just(Token::Newline).or_not())
        .validate(|_, info, emitter| {
            emitter.emit(Rich::custom(
                info.span(),
                "E-PAR-002: unknown field in shape: block",
            ));
        })
        .map(|()| None::<ShapeFieldItem>);

    let recognized_field = type_field
        .or(position_field)
        .or(fill_field)
        .or(text_field)
        .or(alt_field)
        .map(Some);

    // Priority: recognized fields > raw rejection (E-PAR-009) > generic unknown (E-PAR-002).
    let shape_field = recognized_field.or(raw_in_shape).or(unknown_field);

    // The full `shape:` block.
    select! { Token::Ident(s) = e if s.as_ref() == "shape" => e.span() }
        .then_ignore(just(Token::Colon))
        .then_ignore(just(Token::Newline).or_not())
        .then_ignore(select! { Token::Indent(_) => () })
        .then(
            shape_field
                .repeated()
                .collect::<Vec<Option<ShapeFieldItem>>>(),
        )
        .then_ignore(just(Token::Dedent))
        .map_with(move |(_kw_span, items), e| {
            let block_span = e.span();
            let mut node = ShapeNode {
                shape_type: None,
                position: None,
                fill: None,
                text: None,
                alt: None,
            };
            for item in items.into_iter().flatten() {
                match item {
                    ShapeFieldItem::Type(s, sp) => {
                        node.shape_type = Some(Spanned::new(s, to_span(sp, file_id)));
                    },
                    ShapeFieldItem::Position(s, sp) => {
                        node.position = Some(Spanned::new(s, to_span(sp, file_id)));
                    },
                    ShapeFieldItem::Fill(s, sp) => {
                        node.fill = Some(Spanned::new(s, to_span(sp, file_id)));
                    },
                    ShapeFieldItem::Text(chunks, sp) => {
                        node.text = Some(Spanned::new(chunks, to_span(sp, file_id)));
                    },
                    ShapeFieldItem::Alt(s, sp) => {
                        node.alt = Some(Spanned::new(s, to_span(sp, file_id)));
                    },
                }
            }
            (FieldValue::Shape(Box::new(node)), block_span)
        })
}

// ─── Helpers exposed for tests ────────────────────────────────────────────────

/// Construct a placeholder empty [`ShapeNode`].
///
/// Used by tests to verify that `ShapeNode` can be constructed with all-`None`
/// fields (the minimum valid parse state before validation).
#[must_use]
pub fn empty_shape_node() -> ShapeNode {
    ShapeNode {
        shape_type: None,
        position: None,
        fill: None,
        text: None,
        alt: None,
    }
}

/// Construct a fully-populated [`ShapeNode`] for snapshot testing.
///
/// All five fields are set to representative values.  Used by the snapshot
/// test to pin the rendered debug output and catch regressions.
#[must_use]
pub fn full_shape_node_fixture(file_id: u32) -> ShapeNode {
    let span = Span::new(file_id, 0, 1);
    ShapeNode {
        shape_type: Some(Spanned::new("rectangle".to_string(), span)),
        position: Some(Spanned::new("50,50,200,100".to_string(), span)),
        fill: Some(Spanned::new("#0070C0".to_string(), span)),
        text: Some(Spanned::new(
            vec![TemplateChunk::Literal("Click here".to_string())],
            span,
        )),
        alt: Some(Spanned::new(
            "A blue rectangle labelled Click here".to_string(),
            span,
        )),
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{
        ast::{BlockItem, FieldValue},
        error::SyntaxError,
        parser::parse,
        span::SourceMap,
    };
    use std::sync::Arc;

    /// Helper: add a file to a fresh [`SourceMap`] and call [`parse`].
    ///
    /// Returns the inner `DeckNode` on success (ignoring warnings).
    fn parse_str(src: &str) -> Result<crate::ast::DeckNode, Vec<SyntaxError>> {
        let mut sm = SourceMap::new();
        let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
        parse(src, file_id, &sm).map(|pr| pr.deck)
    }

    // ── AC-009: shape: block with 5 fields → ShapeNode ───────────────────────

    #[test]
    fn test_bc_1_09_009_shape_block_all_five_fields_parsed() {
        // AC-009: a `shape:` block with all 5 fields must produce a ShapeNode
        // with all five fields populated.
        let src = concat!(
            "slide content:\n",
            "  shape:\n",
            "    type \"rectangle\"\n",
            "    position \"50,50,200,100\"\n",
            "    fill \"#0070C0\"\n",
            "    text \"Click here\"\n",
            "    alt \"A blue rectangle\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_ok(),
            "shape: block with all 5 fields must parse without errors: {result:?}"
        );
        let deck = result.unwrap();
        let BlockItem::Slide(slide_s) = &deck.items[0] else {
            panic!("expected Slide block item");
        };
        let slide = slide_s.value();
        // Find the "shape" field node.
        let shape_field = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "shape")
            .expect("shape field must exist in slide");
        let FieldValue::Shape(shape_node) = shape_field.value.value() else {
            panic!(
                "shape field must have FieldValue::Shape, got {:?}",
                shape_field.value.value()
            );
        };
        assert!(shape_node.shape_type.is_some(), "shape.type must be parsed");
        assert_eq!(shape_node.shape_type.as_ref().unwrap().value(), "rectangle");
        assert!(
            shape_node.position.is_some(),
            "shape.position must be parsed"
        );
        assert_eq!(
            shape_node.position.as_ref().unwrap().value(),
            "50,50,200,100"
        );
        assert!(shape_node.fill.is_some(), "shape.fill must be parsed");
        assert_eq!(shape_node.fill.as_ref().unwrap().value(), "#0070C0");
        assert!(shape_node.text.is_some(), "shape.text must be parsed");
        assert!(shape_node.alt.is_some(), "shape.alt must be parsed");
        assert_eq!(shape_node.alt.as_ref().unwrap().value(), "A blue rectangle");
    }

    // ── AC-009: shape: block with partial fields → ShapeNode with Nones ──────

    #[test]
    fn test_bc_1_09_009_shape_block_partial_fields_parsed() {
        // Only type and alt — the other three must be None.
        let src = concat!(
            "slide content:\n",
            "  shape:\n",
            "    type \"circle\"\n",
            "    alt \"A decorative circle\"\n",
        );
        let result = parse_str(src);
        assert!(
            result.is_ok(),
            "shape: block with partial fields must parse: {result:?}"
        );
        let deck = result.unwrap();
        let BlockItem::Slide(slide_s) = &deck.items[0] else {
            panic!("expected Slide");
        };
        let shape_field = slide_s
            .value()
            .fields
            .iter()
            .find(|f| f.name.value() == "shape")
            .expect("shape field must exist");
        let FieldValue::Shape(shape_node) = shape_field.value.value() else {
            panic!("must be FieldValue::Shape");
        };
        assert_eq!(shape_node.shape_type.as_ref().unwrap().value(), "circle");
        assert!(
            shape_node.position.is_none(),
            "position must be None when not specified"
        );
        assert!(
            shape_node.fill.is_none(),
            "fill must be None when not specified"
        );
        assert!(
            shape_node.text.is_none(),
            "text must be None when not specified"
        );
        assert_eq!(
            shape_node.alt.as_ref().unwrap().value(),
            "A decorative circle"
        );
    }

    // ── ShapeNode struct: all-None construction ───────────────────────────────

    #[test]
    fn test_bc_1_09_009_empty_shape_node_constructible() {
        let node = empty_shape_node();
        assert!(node.shape_type.is_none());
        assert!(node.position.is_none());
        assert!(node.fill.is_none());
        assert!(node.text.is_none());
        assert!(node.alt.is_none());
    }

    // ── ShapeNode struct: Hash + Eq + Clone + Debug ───────────────────────────

    #[test]
    fn test_bc_1_09_009_shape_node_derives_hash_eq_clone_debug() {
        use std::collections::HashSet;
        let node = empty_shape_node();
        let node2 = node.clone();
        assert_eq!(node, node2);
        let _ = format!("{node:?}");
        let mut set = HashSet::new();
        set.insert(node);
        assert_eq!(set.len(), 1);
    }

    // ── FieldValue::Shape variant is constructible ────────────────────────────

    #[test]
    fn test_bc_1_09_009_field_value_shape_variant_constructible() {
        use std::collections::HashSet;
        let node = empty_shape_node();
        let fv = FieldValue::Shape(Box::new(node));
        let fv2 = fv.clone();
        assert_eq!(fv, fv2);
        let _ = format!("{fv:?}");
        let mut set = HashSet::new();
        set.insert(fv);
        assert_eq!(set.len(), 1);
    }

    // ── EC-005: `raw` field inside shape: block → E-PAR-009 ──────────────────

    #[test]
    fn test_bc_1_09_009_shape_block_raw_field_emits_e_par_009() {
        // EC-005: a `raw` field name inside a shape: block must produce E-PAR-009
        // (RawKeyword), not E-PAR-002 (generic unknown field).
        // The `raw` identifier is reserved across all contexts — including shape
        // blocks. The shape block parser has a dedicated raw-rejection path that
        // fires before the generic unknown_field fallback.
        let src = concat!("slide content:\n", "  shape:\n", "    raw \"value\"\n",);
        let result = parse_str(src);
        assert!(
            result.is_err(),
            "raw field inside shape: block must produce an error; got Ok"
        );
        let errors = result.unwrap_err();
        let has_raw_keyword_err = errors
            .iter()
            .any(|e| matches!(e, SyntaxError::RawKeyword { .. }));
        assert!(
            has_raw_keyword_err,
            "raw inside shape: block must produce E-PAR-009 (RawKeyword), not E-PAR-002; \
             got: {errors:?}"
        );
    }

    // ── EC-005b: snapshot test ────────────────────────────────────────────────

    #[test]
    fn test_bc_1_09_009_shape_block_snapshot() {
        // Snapshot test: fully-populated ShapeNode must render consistently.
        let node = full_shape_node_fixture(0);
        insta::assert_debug_snapshot!("shape_block_full_fixture", node);
    }
}
