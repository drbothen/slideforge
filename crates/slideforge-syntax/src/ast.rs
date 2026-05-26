//! Abstract syntax tree (AST) node types for the slideforge DSL.
//!
//! All node types implement `Hash + Eq + Clone + Debug` as required by
//! ADR-013 (comemo cache compatibility) and the Kani proof harness.
//!
//! # AST Structure
//!
//! A parsed `.sf` file produces a [`DeckNode`] which contains:
//!
//! - Optional deck metadata (`slideforge_version`, `lang`, `brand`).
//! - Zero or more [`VarsBlock`] entries (global variable definitions).
//! - Zero or more [`SetRule`] entries (default field overrides per slide type).
//! - One or more [`BlockItem`] entries (slides, `@for` blocks, `@if` blocks).
//!
//! Each [`SlideNode`] holds a slide type keyword and a list of [`FieldNode`]
//! entries mapping field names to [`FieldValue`] variants.
//!
//! # STORY-007 Additions
//!
//! - [`BlockItem`]: replaces the flat `Vec<Spanned<SlideNode>>` in [`DeckNode`].
//! - [`ForNode`]: represents an `@for item in collection:` iteration block.
//! - [`IfNode`]: represents an `@if/@elif/@else` conditional block.
//! - [`FieldValue::Template`]: replaces `FieldValue::Str` for string values
//!   containing `{{ expr }}` interpolation.

use crate::expr::Expr;
use crate::span::Spanned;
use crate::template::TemplateChunk;

// ─── FieldValue ──────────────────────────────────────────────────────────────

/// The value of a DSL field.
///
/// Parsed from the right-hand side of a field assignment:
/// `title "My Title"` → [`FieldValue::Template`] with a single
/// [`TemplateChunk::Literal`] chunk.
/// `title "Hello {{ name }}"` → [`FieldValue::Template`] with a
/// `Literal` chunk and an `Expr` chunk.
///
/// The `Error` sentinel is used by the error-recovery path when parsing fails
/// mid-value. It allows the parser to continue accumulating errors without
/// abandoning the entire parse tree.
///
/// # STORY-007: `Str` replaced by `Template`
///
/// `FieldValue::Str` from STORY-006 is replaced by `FieldValue::Template` so
/// that all string values are treated uniformly: a plain string `"Hello"` is
/// represented as `Template(vec![TemplateChunk::Literal("Hello")])`.
/// This eliminates the need for callers to check both variants.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldValue {
    /// A template string: zero or more [`TemplateChunk`]s.
    ///
    /// Replaces `FieldValue::Str` from STORY-006. A plain string like
    /// `"Hello"` produces a single `Literal` chunk. A string with
    /// interpolation like `"Hello {{ name }}"` produces `[Literal("Hello "),
    /// Expr(Expr::Ident("name"))]`.
    Template(Vec<TemplateChunk>),
    /// An integer literal.
    Num(i64),
    /// A floating-point literal.
    ///
    /// Uses [`ordered_float::OrderedFloat`] so that `FieldValue` remains
    /// `Hash + Eq`, which is required for comemo cache compatibility (ADR-013).
    Float(ordered_float::OrderedFloat<f64>),
    /// A boolean literal (`true` or `false`).
    ///
    /// Stored as a proper `bool` rather than a [`FieldValue::Ident`] so that
    /// the evaluator (STORY-011+) can distinguish `true`/`false` from
    /// user-defined identifiers at the AST level without string comparison.
    Bool(bool),
    /// An unquoted bare identifier.
    Ident(String),
    /// Sentinel produced by the error-recovery path when a value could not be parsed.
    Error,
}

// ─── FieldNode ───────────────────────────────────────────────────────────────

/// A single field assignment inside a slide or vars block.
///
/// Represents one line of the form `<name> <value>` inside an indented block,
/// e.g. `title "Hello World"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldNode {
    /// The field name (e.g. `"title"`, `"body"`, `"footer"`).
    pub name: Spanned<String>,
    /// The field value.
    pub value: Spanned<FieldValue>,
}

// ─── VarsBlock ───────────────────────────────────────────────────────────────

/// A `vars:` block defining global named variables.
///
/// ```sf
/// vars:
///   client: "Acme Corp"
///   year: 2024
/// ```
///
/// Each entry is a `(name, value)` pair. Duplicate keys are allowed at the
/// parse level; the evaluator (STORY-007+) enforces uniqueness.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VarsBlock {
    /// The entries in this vars block, in source order.
    pub entries: Vec<(Spanned<String>, Spanned<FieldValue>)>,
}

// ─── SetRule ─────────────────────────────────────────────────────────────────

/// A `set` rule that overrides a field's default value for a slide type.
///
/// ```sf
/// set content: footer "Confidential"
/// ```
///
/// This means: for all `slide content:` blocks, the `footer` field defaults to
/// `"Confidential"` unless overridden in the slide itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetRule {
    /// The slide type that this rule applies to (e.g. `"content"`).
    pub slide_type: Spanned<String>,
    /// The field being given a default (e.g. `"footer"`).
    pub field: Spanned<String>,
    /// The default value.
    pub value: Spanned<FieldValue>,
}

// ─── SlideNode ───────────────────────────────────────────────────────────────

/// A single `slide <type>:` block.
///
/// ```sf
/// slide title:
///   title "Introduction"
///   subtitle "An Overview"
/// ```
///
/// The `kind` field holds the slide-type keyword (e.g. `"title"`, `"content"`).
/// `tags` is reserved for future tagging syntax (STORY-008); it is always empty
/// in STORY-006.
///
/// # STORY-007: Element-scope control flow
///
/// `inline_items` holds any `@if` or `@for` blocks that appear inside the slide
/// body at element scope (between or alongside field lines). Field lines remain
/// in `fields` in source order; control-flow items are in `inline_items` in
/// source order. Both vecs together represent the full body in insertion order.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlideNode {
    /// The slide type keyword.
    pub kind: Spanned<String>,
    /// Tags applied to this slide (always empty in STORY-006; extended in STORY-008).
    pub tags: Vec<Spanned<String>>,
    /// Field assignments inside this slide block, in source order.
    pub fields: Vec<FieldNode>,
    /// Element-scope control-flow items (`@if`, `@for`) inside this slide body,
    /// in source order (STORY-007+). Always empty in slides with no control flow.
    pub inline_items: Vec<BlockItem>,
}

// ─── SectionNode ─────────────────────────────────────────────────────────────

/// A `section <type>:` block (future; placeholder for STORY-008+).
///
/// Included here so that [`BlockItem::Section`] can reference it, completing
/// the block item discriminant set. The `fields` member is intentionally
/// empty for the STORY-007 scope.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SectionNode {
    /// The section type keyword.
    pub kind: Spanned<String>,
    /// Field assignments inside this section block, in source order.
    pub fields: Vec<FieldNode>,
}

// ─── ForNode ──────────────────────────────────────────────────────────────────

/// An `@for item in collection:` iteration block.
///
/// ```sf
/// @for slide in slides:
///   slide content:
///     title "{{ slide.title }}"
/// ```
///
/// The body is a `Vec<BlockItem>` — nested `@for`, `@if`, and `slide` blocks
/// are all valid inside a `@for` body, including recursive nesting.
///
/// # Evaluation Scope
///
/// The `binding` name is recorded in the AST so the evaluator (STORY-012) can
/// introduce it into the inner scope. The parser does NOT enforce uniqueness —
/// that is an eval concern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForNode {
    /// The loop binding variable name.
    ///
    /// In `@for item in items:` this is `"item"`.
    pub binding: Spanned<String>,
    /// The collection expression being iterated.
    ///
    /// May be an identifier (`Expr::Ident("items")`), a list literal
    /// (`Expr::List([...])`), or any other expression.
    pub collection: Spanned<Expr>,
    /// The body items, in source order.
    pub body: Vec<BlockItem>,
}

// ─── IfNode ───────────────────────────────────────────────────────────────────

/// An `@if/@elif/@else` conditional block.
///
/// ```sf
/// @if env == "prod":
///   slide warning:
///     title "Production!"
/// @elif env == "staging":
///   slide info:
///     title "Staging"
/// @else:
///   slide info:
///     title "Development"
/// ```
///
/// The `elif_branches` field is a vec of `(condition_expr, body)` pairs,
/// preserving source order. The `else_body` is `None` if no `@else:` clause
/// was present.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IfNode {
    /// The `@if` condition expression.
    pub condition: Spanned<Expr>,
    /// The `@if` body items.
    pub then_body: Vec<BlockItem>,
    /// Zero or more `@elif condition: body` branches, in source order.
    pub elif_branches: Vec<(Spanned<Expr>, Vec<BlockItem>)>,
    /// The optional `@else: body` branch.
    pub else_body: Option<Vec<BlockItem>>,
}

// ─── BlockItem ────────────────────────────────────────────────────────────────

/// A single item in a block body (deck, `@for` body, or `@if` body).
///
/// Replaces the flat `Vec<Spanned<SlideNode>>` that was used in [`DeckNode`]
/// in STORY-006. This allows `@for` and `@if` nodes to appear at the same
/// syntactic level as `slide` blocks.
///
/// # STORY-007 Variants
///
/// | Variant | DSL construct |
/// |---------|--------------|
/// | `Slide` | `slide <type>: ...` |
/// | `For`   | `@for x in coll: ...` |
/// | `If`    | `@if cond: ... @elif ... @else ...` |
/// | `Section` | `section <type>: ...` (future, STORY-008+) |
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlockItem {
    /// A `slide <type>:` block.
    Slide(Spanned<SlideNode>),
    /// An `@for item in collection:` iteration block.
    For(Spanned<ForNode>),
    /// An `@if/@elif/@else` conditional block.
    If(Spanned<IfNode>),
    /// A `section <type>:` block (STORY-008+ placeholder).
    Section(Spanned<SectionNode>),
}

// ─── DeckNode ────────────────────────────────────────────────────────────────

/// The top-level AST node representing an entire `.sf` file.
///
/// All fields are optional at the parse level. Required-field enforcement is
/// delegated to the validation stage (STORY-016).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct DeckNode {
    /// `slideforge_version "1"` declaration, if present.
    pub version: Option<Spanned<String>>,
    /// `lang "en-US"` declaration, if present.
    pub lang: Option<Spanned<String>>,
    /// `brand "template.pptx"` declaration, if present.
    pub brand: Option<Spanned<String>>,
    /// All `vars:` blocks, in source order.
    pub vars: Vec<VarsBlock>,
    /// All `set` rules, in source order.
    pub set_rules: Vec<SetRule>,
    /// All top-level block items (slides, `@for`, `@if` blocks), in source
    /// order.
    ///
    /// This replaces `slides: Vec<Spanned<SlideNode>>` from STORY-006.
    /// Callers that only need slides can filter with:
    /// ```rust,ignore
    /// deck.items.iter().filter_map(|i| {
    ///     if let BlockItem::Slide(s) = i { Some(s) } else { None }
    /// })
    /// ```
    pub items: Vec<BlockItem>,
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{BinOpKind, Expr};
    use crate::span::Span;
    use crate::template::TemplateChunk;
    use std::collections::HashSet;

    fn dummy_span() -> Span {
        Span::new(0, 0, 0)
    }

    #[test]
    fn test_bc_1_01_001_field_value_hash_eq_clone_debug() {
        let v = FieldValue::Template(vec![TemplateChunk::Literal("hello".to_string())]);
        let v2 = v.clone();
        assert_eq!(v, v2);
        let _ = format!("{v:?}");
        let mut set = HashSet::new();
        set.insert(v);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_01_001_field_value_variants_all_constructible() {
        let _ = FieldValue::Template(vec![TemplateChunk::Literal("s".to_string())]);
        let _ = FieldValue::Template(vec![
            TemplateChunk::Literal("Hello ".to_string()),
            TemplateChunk::Expr(Expr::Ident("name".to_string())),
        ]);
        let _ = FieldValue::Num(42);
        let _ = FieldValue::Float(ordered_float::OrderedFloat(1.5_f64));
        let _ = FieldValue::Bool(true);
        let _ = FieldValue::Bool(false);
        let _ = FieldValue::Ident("foo".to_string());
        let _ = FieldValue::Error;
    }

    #[test]
    fn test_bc_1_04_001_field_value_template_with_binop() {
        // AC-008: title "{{ x + 1 }}" → Template([Expr(BinOp { Add, x, 1 })])
        let binop = Expr::BinOp {
            op: BinOpKind::Add,
            lhs: Box::new(Expr::Ident("x".to_string())),
            rhs: Box::new(Expr::Num(1)),
        };
        let v = FieldValue::Template(vec![TemplateChunk::Expr(binop)]);
        let v2 = v.clone();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_bc_1_01_001_deck_node_default_empty() {
        let d = DeckNode::default();
        assert!(d.version.is_none());
        assert!(d.lang.is_none());
        assert!(d.brand.is_none());
        assert!(d.vars.is_empty());
        assert!(d.set_rules.is_empty());
        // STORY-007: items replaces slides
        assert!(d.items.is_empty());
    }

    #[test]
    fn test_bc_1_01_001_deck_node_hash_eq_clone_debug() {
        let d = DeckNode::default();
        let d2 = d.clone();
        assert_eq!(d, d2);
        let _ = format!("{d:?}");
        let mut set = HashSet::new();
        set.insert(d);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_01_001_slide_node_hash_eq_clone_debug() {
        let s = SlideNode {
            kind: Spanned::new("title".to_string(), dummy_span()),
            tags: vec![],
            fields: vec![],
            inline_items: vec![],
        };
        let s2 = s.clone();
        assert_eq!(s, s2);
        let _ = format!("{s:?}");
    }

    #[test]
    fn test_bc_1_01_001_set_rule_hash_eq_clone_debug() {
        let r = SetRule {
            slide_type: Spanned::new("content".to_string(), dummy_span()),
            field: Spanned::new("footer".to_string(), dummy_span()),
            value: Spanned::new(
                FieldValue::Template(vec![TemplateChunk::Literal("Conf".to_string())]),
                dummy_span(),
            ),
        };
        let r2 = r.clone();
        assert_eq!(r, r2);
        let _ = format!("{r:?}");
    }

    #[test]
    fn test_bc_1_01_001_vars_block_hash_eq_clone_debug() {
        let vb = VarsBlock { entries: vec![] };
        let vb2 = vb.clone();
        assert_eq!(vb, vb2);
        let _ = format!("{vb:?}");
    }

    // ── STORY-007: ForNode, IfNode, BlockItem ──────────────────────────────

    #[test]
    fn test_bc_1_04_001_for_node_derives_hash_eq_clone_debug() {
        let n = ForNode {
            binding: Spanned::new("item".to_string(), dummy_span()),
            collection: Spanned::new(Expr::Ident("items".to_string()), dummy_span()),
            body: vec![],
        };
        let n2 = n.clone();
        assert_eq!(n, n2);
        let _ = format!("{n:?}");
        let mut set = HashSet::new();
        set.insert(n);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_05_001_if_node_derives_hash_eq_clone_debug() {
        let n = IfNode {
            condition: Spanned::new(
                Expr::BinOp {
                    op: BinOpKind::Eq,
                    lhs: Box::new(Expr::Ident("env".to_string())),
                    rhs: Box::new(Expr::Str("prod".to_string())),
                },
                dummy_span(),
            ),
            then_body: vec![],
            elif_branches: vec![],
            else_body: None,
        };
        let n2 = n.clone();
        assert_eq!(n, n2);
        let _ = format!("{n:?}");
    }

    #[test]
    fn test_bc_1_04_001_block_item_all_variants_constructible() {
        let slide_node = SlideNode {
            kind: Spanned::new("title".to_string(), dummy_span()),
            tags: vec![],
            fields: vec![],
            inline_items: vec![],
        };
        let for_node = ForNode {
            binding: Spanned::new("x".to_string(), dummy_span()),
            collection: Spanned::new(Expr::Ident("items".to_string()), dummy_span()),
            body: vec![],
        };
        let if_node = IfNode {
            condition: Spanned::new(Expr::Bool(true), dummy_span()),
            then_body: vec![],
            elif_branches: vec![],
            else_body: None,
        };
        let section_node = SectionNode {
            kind: Spanned::new("intro".to_string(), dummy_span()),
            fields: vec![],
        };

        let items: Vec<BlockItem> = vec![
            BlockItem::Slide(Spanned::new(slide_node, dummy_span())),
            BlockItem::For(Spanned::new(for_node, dummy_span())),
            BlockItem::If(Spanned::new(if_node, dummy_span())),
            BlockItem::Section(Spanned::new(section_node, dummy_span())),
        ];
        for item in &items {
            let _ = format!("{item:?}");
        }
        assert_eq!(items.len(), 4);
    }

    #[test]
    fn test_bc_1_04_001_deck_node_items_field_replaces_slides() {
        // Verify DeckNode has `items` (not `slides`) field after STORY-007.
        let mut d = DeckNode::default();
        assert!(d.items.is_empty());

        let slide_node = SlideNode {
            kind: Spanned::new("title".to_string(), dummy_span()),
            tags: vec![],
            fields: vec![],
            inline_items: vec![],
        };
        d.items
            .push(BlockItem::Slide(Spanned::new(slide_node, dummy_span())));
        assert_eq!(d.items.len(), 1);
    }

    #[test]
    fn test_bc_1_04_001_for_node_with_list_collection() {
        // AC-002: @for x in [1, 2, 3]: → collection = Expr::List([Num(1), Num(2), Num(3)])
        let n = ForNode {
            binding: Spanned::new("x".to_string(), dummy_span()),
            collection: Spanned::new(
                Expr::List(vec![Expr::Num(1), Expr::Num(2), Expr::Num(3)]),
                dummy_span(),
            ),
            body: vec![],
        };
        let Expr::List(elems) = n.collection.value() else {
            panic!("expected List collection");
        };
        assert_eq!(elems.len(), 3);
        assert_eq!(elems[0], Expr::Num(1));
        assert_eq!(elems[1], Expr::Num(2));
        assert_eq!(elems[2], Expr::Num(3));
    }

    #[test]
    fn test_bc_1_05_001_if_node_with_elif_and_else() {
        // AC-005: @if cond1: ... @elif cond2: ... @else: ...
        let n = IfNode {
            condition: Spanned::new(Expr::Ident("cond1".to_string()), dummy_span()),
            then_body: vec![],
            elif_branches: vec![(
                Spanned::new(Expr::Ident("cond2".to_string()), dummy_span()),
                vec![],
            )],
            else_body: Some(vec![]),
        };
        assert_eq!(n.elif_branches.len(), 1);
        assert!(n.else_body.is_some());
    }

    #[test]
    fn test_bc_1_04_001_for_node_empty_list_is_valid() {
        // EC-001: @for x in []: → valid parse, empty collection detection is EVAL concern.
        let n = ForNode {
            binding: Spanned::new("x".to_string(), dummy_span()),
            collection: Spanned::new(Expr::List(vec![]), dummy_span()),
            body: vec![],
        };
        let Expr::List(elems) = n.collection.value() else {
            panic!("expected List");
        };
        assert!(elems.is_empty());
    }
}
