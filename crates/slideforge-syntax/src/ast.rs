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
//! - An optional [`VariantsBlock`] (named audience variants).
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
//!
//! # STORY-008 Additions
//!
//! - [`VariantsBlock`]: top-level `variants:` block with named audience variants.
//! - [`VariantNode`]: a single named variant with `include_tags`, `exclude_tags`,
//!   `vars`, and optional `inherits` fields.
//! - [`AliasNode`]: an `alias <name> = <type>: ...` declaration (before expansion).
//! - [`SetRuleValue`]: replaces `FieldValue` in [`SetRule`] so that `set` rules
//!   can carry a `BrandRef` without conflating with field values.
//! - [`DeckNode::variants`]: the parsed `VariantsBlock`, if present.
//! - [`DeckNode::variant_names`]: all declared variant names (for validation).
//!
//! # STORY-009 Additions
//!
//! - [`ShapeNode`]: a `shape:` block with `type`, `position`, `fill`, `text`,
//!   and `alt` fields parsed into a typed struct.  Appears inside a
//!   [`SlideNode`] as a new [`FieldValue::Shape`] variant so that the
//!   evaluator can place arbitrary shapes on a slide.

use crate::expr::Expr;
use crate::span::Spanned;
use crate::template::TemplateChunk;

// ─── TemplateValue ───────────────────────────────────────────────────────────

/// A parsed template string: a sequence of [`TemplateChunk`]s.
///
/// This is the canonical representation for any field value that could contain
/// `{{ expr }}` interpolation, `$...$` / `$$...$$` math regions, or plain text.
/// It is separate from the raw `Vec<TemplateChunk>` so that trait bounds
/// (`Hash + Eq + Clone + Debug`) are easily satisfied.
pub type TemplateValue = Vec<TemplateChunk>;

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
    /// A `shape:` block value.
    ///
    /// Produced when a `shape:` block is encountered inside a slide body. The
    /// `FieldNode` name is the synthetic string `"shape"`.
    ///
    /// # STORY-009
    ///
    /// This variant enables the evaluator to place arbitrary shapes on a slide
    /// without conflating them with string or numeric field values.
    Shape(Box<ShapeNode>),
    /// Sentinel produced by the error-recovery path when a value could not be parsed.
    Error,
}

// ─── ShapeNode ───────────────────────────────────────────────────────────────

/// A `shape:` block inside a slide.
///
/// ```sf
/// slide content:
///   shape:
///     type "rectangle"
///     position "50,50,200,100"
///     fill "#0070C0"
///     text "Click here"
///     alt "A blue rectangle labelled Click here"
/// ```
///
/// # Fields
///
/// All five fields are optional at parse time.  The validator (STORY-016)
/// enforces that `alt` is present unless `decorative: true` is set on the
/// enclosing slide.
///
/// # STORY-009
///
/// This node is produced by `parser/shape.rs::shape_block()` and stored as
/// [`FieldValue::Shape`] inside a [`FieldNode`] with the synthetic name
/// `"shape"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapeNode {
    /// The shape type keyword (e.g. `"rectangle"`, `"circle"`, `"line"`).
    ///
    /// Stored as-is; the validator resolves it against the allowed type set.
    pub shape_type: Option<Spanned<String>>,

    /// The position and size specification: `"x,y,width,height"` in EMU units
    /// or a named anchor keyword (e.g. `"center"`, `"full-bleed"`).
    pub position: Option<Spanned<String>>,

    /// The fill color or token reference (e.g. `"#0070C0"`, `"brand.accent"`).
    pub fill: Option<Spanned<String>>,

    /// The text content of the shape, as a parsed template value.
    ///
    /// May contain `{{ expr }}` interpolation and math regions — the full
    /// [`TemplateValue`] representation is used here.
    pub text: Option<Spanned<TemplateValue>>,

    /// Accessibility label for the shape.
    ///
    /// `None` at parse time does NOT mean the shape is decorative — the
    /// validator enforces `alt` presence unless `decorative: true` is set.
    pub alt: Option<Spanned<String>>,
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

// ─── SetRuleValue ─────────────────────────────────────────────────────────────

/// The value in a `set` rule.
///
/// Distinguished from [`FieldValue`] because `set` rules may contain
/// `brand.*` field-access references that must NOT be evaluated at parse time.
/// The evaluator (STORY-011) resolves `BrandRef` after brand loading completes.
///
/// BC-1.08.002: `set` rules support `{{ }}` interpolation (→ `Template`) and
/// `brand.*` references (→ stored as `Template` with `Expr::FieldAccess{base:
/// Ident("brand"), field}` — no special `BrandRef` variant needed at parse time
/// since `{{ brand.footer }}` is a legitimate template expression).
///
/// # Design Note
///
/// `set content: footer "{{ brand.footer }}"` is parsed as:
/// `SetRuleValue::Template([TemplateChunk::Expr(Expr::FieldAccess { base:
/// Ident("brand"), field: "footer" })])`.
///
/// The `brand.*` reference is stored as-is — the expression parser naturally
/// produces `Expr::FieldAccess` for `brand.footer`. Evaluation is deferred.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SetRuleValue {
    /// A template string: zero or more [`TemplateChunk`]s.
    ///
    /// Used for both plain string values and `{{ expr }}` interpolations,
    /// including `{{ brand.footer }}` brand-ref style accesses.
    Template(Vec<TemplateChunk>),
    /// An integer literal.
    Num(i64),
    /// A floating-point literal.
    Float(ordered_float::OrderedFloat<f64>),
    /// A boolean literal.
    Bool(bool),
    /// An unquoted bare identifier.
    Ident(String),
    /// Sentinel produced by error recovery.
    Error,
}

// ─── SetRule ─────────────────────────────────────────────────────────────────

/// A `set` rule that overrides a field's default value for a slide type.
///
/// ```text
/// set content: footer "Confidential"
/// set content: footer "{{ brand.footer }}"
/// ```
///
/// This means: for all `slide content:` blocks, the `footer` field defaults to
/// the given value unless overridden in the slide itself.
///
/// # STORY-008: `SetRuleValue`
///
/// The `value` field is now `Spanned<SetRuleValue>` (not `Spanned<FieldValue>`)
/// to cleanly represent the distinction between field values and set-rule
/// values (which may carry brand references evaluated after brand loading).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetRule {
    /// The slide type that this rule applies to (e.g. `"content"`).
    pub slide_type: Spanned<String>,
    /// The field being given a default (e.g. `"footer"`).
    pub field: Spanned<String>,
    /// The default value (template string, number, bool, or brand ref expression).
    pub value: Spanned<SetRuleValue>,
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

/// A `section <type>:` block parsed by `section_block_parser` (STORY-078).
///
/// Each `section <kind>:` block in a `.sf` file produces one `SectionNode`.
/// The `fields` member is populated by the parser and contains every
/// key/value assignment found inside the section body, in source order.
/// Each [`FieldNode`] carries a sub-block key and its associated value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SectionNode {
    /// The section type keyword (e.g., `"appendix"`, `"glossary"`).
    pub kind: Spanned<String>,
    /// Field assignments inside this section block, in source order.
    ///
    /// Populated by `section_block_parser` (STORY-078). Each entry is a
    /// [`FieldNode`] carrying a sub-block key and its parsed value.
    pub fields: Vec<FieldNode>,
}

// ─── SectionGroupNode ────────────────────────────────────────────────────────

/// A `section "Name":` slide-grouping block (STORY-082).
///
/// ```sf
/// section "Background":
///   slide title:
///     title "Background"
///   slide content:
///     title "Details"
/// ```
///
/// This is syntactically DISTINCT from [`SectionNode`] (which uses a bare
/// identifier after `section`). `SectionGroupNode` carries a quoted name and
/// child slide nodes. It is used by the PPTX exporter to populate
/// `<p14:sectionLst>` in `presentation.xml`.
///
/// # STORY-082
///
/// Introduced in STORY-082 (PPTX: Slide-Grouping Sections).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SectionGroupNode {
    /// The section group name as declared in the DSL (quoted string).
    ///
    /// Must be non-empty (enforced at parse time via E-PAR-023). An empty
    /// name is rejected and no `SectionGroupNode` is produced for it.
    pub name: Spanned<std::sync::Arc<str>>,
    /// The slide children of this section group, in source order.
    ///
    /// Each entry is a [`BlockItem::Slide`] (or nested control-flow block).
    /// After evaluation, the PPTX exporter maps these to slide IDs.
    pub slides: Vec<BlockItem>,
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
/// | `Section` | `section <type>: ...` (STORY-078) |
/// | `SectionGroup` | `section "Name": ...` (STORY-082) |
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlockItem {
    /// A `slide <type>:` block.
    Slide(Spanned<SlideNode>),
    /// An `@for item in collection:` iteration block.
    For(Spanned<ForNode>),
    /// An `@if/@elif/@else` conditional block.
    If(Spanned<IfNode>),
    /// A `section <type>:` block (STORY-078).
    Section(Spanned<SectionNode>),
    /// A `section "Name":` slide-grouping block (STORY-082).
    ///
    /// Syntactically distinct from [`BlockItem::Section`]: the name is a
    /// quoted string (not a bare identifier). Children are slide blocks.
    SectionGroup(Spanned<SectionGroupNode>),
}

// ─── VariantNode ─────────────────────────────────────────────────────────────

/// A single named audience variant inside a `variants:` block.
///
/// ```text
/// variants:
///   exec:
///     include_tags: [executive]
///     vars:
///       color: "navy"
///   internal:
///     exclude_tags: [confidential]
///     inherits: "exec"
/// ```
///
/// # STORY-008
///
/// The evaluator (STORY-011) applies the variant's `include_tags` / `exclude_tags`
/// filters and merges `vars` into the active scope. This node is purely structural
/// — no filtering or vars-merging occurs at parse time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariantNode {
    /// The variant name (e.g. `"exec"`, `"internal"`).
    pub name: Spanned<String>,
    /// Slides whose tags include ALL of these tags are kept; all others removed.
    pub include_tags: Vec<Spanned<String>>,
    /// Slides whose tags include ANY of these tags are removed.
    pub exclude_tags: Vec<Spanned<String>>,
    /// Variant-scoped variable overrides (merged on top of deck-level vars).
    pub vars: Vec<(Spanned<String>, Spanned<FieldValue>)>,
    /// Optional name of another variant to inherit from.
    ///
    /// Cycle detection (`exec` ↔ `internal`) is performed at parse time and
    /// results in E-VAR-001.
    pub inherits: Option<Spanned<String>>,
}

// ─── VariantsBlock ───────────────────────────────────────────────────────────

/// A `variants:` block with one or more named audience variants.
///
/// ```text
/// variants:
///   exec:
///     include_tags: [executive]
///   internal:
///     exclude_tags: [confidential]
/// ```
///
/// At most one `variants:` block is valid per deck; duplicate blocks are an
/// error at the eval stage (STORY-011). The parser collects all of them but the
/// evaluator enforces the uniqueness invariant.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariantsBlock {
    /// The named variants, in source order.
    pub variants: Vec<VariantNode>,
}

// ─── AliasNode ───────────────────────────────────────────────────────────────

/// An `alias <name> = <base_type>: <preset_fields>` declaration.
///
/// ```text
/// alias exec_title = title:
///   footer "Internal Only"
/// ```
///
/// After parse, the `AliasNode` is consumed by the alias expansion pass
/// (inside `IncludeResolver` / deck post-processing). It does **not** appear
/// in the final `DeckNode` — aliases are fully expanded so that `slide
/// exec_title:` in the source becomes `slide title:` with `footer` preset.
///
/// The `AliasNode` is exposed in the AST as an intermediate form so that tests
/// can exercise the alias-declaration parsing independently of expansion.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AliasNode {
    /// The user-defined alias name (e.g. `"exec_title"`).
    ///
    /// Must NOT collide with a built-in slide type (→ E-PAR-006).
    pub name: Spanned<String>,
    /// The base slide type being aliased (e.g. `"title"`).
    pub base_type: Spanned<String>,
    /// The preset field assignments for this alias.
    ///
    /// All field names must be valid fields of `base_type` (→ E-PAR-011 if not).
    pub preset_fields: Vec<FieldNode>,
}

// ─── DeckNode ────────────────────────────────────────────────────────────────

/// The top-level AST node representing an entire `.sf` file.
///
/// All fields are optional at the parse level. Required-field enforcement is
/// delegated to the validation stage (STORY-016).
///
/// # STORY-008 Additions
///
/// - `variants`: the optional `variants:` block, if present in source.
/// - `variant_names`: all declared variant names, collected for use by the
///   `--variant` CLI flag validator (BC-1.07.004).
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
    /// The optional `variants:` block (STORY-008).
    ///
    /// At most one `variants:` block is expected per deck; the evaluator enforces
    /// the uniqueness invariant.
    pub variants: Option<VariantsBlock>,
    /// All declared variant names, in source order (STORY-008).
    ///
    /// Used by the CLI (`--variant <name>`) to validate the supplied name
    /// without traversing `variants.variants`.
    pub variant_names: Vec<String>,
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
#[allow(clippy::unwrap_used)]
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
        // STORY-008: variants fields default to None/empty
        assert!(d.variants.is_none());
        assert!(d.variant_names.is_empty());
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
                SetRuleValue::Template(vec![TemplateChunk::Literal("Conf".to_string())]),
                dummy_span(),
            ),
        };
        let r2 = r.clone();
        assert_eq!(r, r2);
        let _ = format!("{r:?}");
    }

    // ── STORY-008: SetRuleValue ────────────────────────────────────────────────

    #[test]
    fn test_bc_1_08_001_set_rule_value_hash_eq_clone_debug() {
        let v = SetRuleValue::Template(vec![TemplateChunk::Literal("Conf".to_string())]);
        let v2 = v.clone();
        assert_eq!(v, v2);
        let _ = format!("{v:?}");
        let mut set = HashSet::new();
        set.insert(v);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_08_001_set_rule_value_all_variants_constructible() {
        let _ = SetRuleValue::Template(vec![TemplateChunk::Literal("x".to_string())]);
        let _ = SetRuleValue::Num(42);
        let _ = SetRuleValue::Float(ordered_float::OrderedFloat(1.5_f64));
        let _ = SetRuleValue::Bool(true);
        let _ = SetRuleValue::Ident("foo".to_string());
        let _ = SetRuleValue::Error;
    }

    // ── STORY-008: VariantNode ────────────────────────────────────────────────

    #[test]
    fn test_bc_1_07_001_variant_node_hash_eq_clone_debug() {
        let v = VariantNode {
            name: Spanned::new("exec".to_string(), dummy_span()),
            include_tags: vec![Spanned::new("executive".to_string(), dummy_span())],
            exclude_tags: vec![],
            vars: vec![],
            inherits: None,
        };
        let v2 = v.clone();
        assert_eq!(v, v2);
        let _ = format!("{v:?}");
        let mut set = HashSet::new();
        set.insert(v);
        assert_eq!(set.len(), 1);
    }

    // ── STORY-008: VariantsBlock ──────────────────────────────────────────────

    #[test]
    fn test_bc_1_07_001_variants_block_hash_eq_clone_debug() {
        let vb = VariantsBlock {
            variants: vec![VariantNode {
                name: Spanned::new("exec".to_string(), dummy_span()),
                include_tags: vec![],
                exclude_tags: vec![],
                vars: vec![],
                inherits: None,
            }],
        };
        let vb2 = vb.clone();
        assert_eq!(vb, vb2);
        let _ = format!("{vb:?}");
    }

    // ── STORY-008: AliasNode ──────────────────────────────────────────────────

    #[test]
    fn test_bc_1_09_001_alias_node_hash_eq_clone_debug() {
        let a = AliasNode {
            name: Spanned::new("exec_title".to_string(), dummy_span()),
            base_type: Spanned::new("title".to_string(), dummy_span()),
            preset_fields: vec![FieldNode {
                name: Spanned::new("footer".to_string(), dummy_span()),
                value: Spanned::new(
                    FieldValue::Template(vec![TemplateChunk::Literal("Internal".to_string())]),
                    dummy_span(),
                ),
            }],
        };
        let a2 = a.clone();
        assert_eq!(a, a2);
        let _ = format!("{a:?}");
        let mut set = HashSet::new();
        set.insert(a);
        assert_eq!(set.len(), 1);
    }

    // ── STORY-008: DeckNode variant fields ────────────────────────────────────

    #[test]
    fn test_bc_1_07_004_deck_node_variant_names_field() {
        let mut d = DeckNode::default();
        assert!(d.variant_names.is_empty());
        d.variant_names.push("exec".to_string());
        d.variant_names.push("internal".to_string());
        assert_eq!(d.variant_names.len(), 2);
        assert_eq!(d.variant_names[0], "exec");
    }

    #[test]
    fn test_bc_1_07_001_deck_node_variants_field() {
        let mut d = DeckNode::default();
        assert!(d.variants.is_none());
        d.variants = Some(VariantsBlock {
            variants: vec![VariantNode {
                name: Spanned::new("exec".to_string(), dummy_span()),
                include_tags: vec![],
                exclude_tags: vec![],
                vars: vec![],
                inherits: None,
            }],
        });
        assert!(d.variants.is_some());
        assert_eq!(d.variants.as_ref().unwrap().variants.len(), 1);
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

    // ── STORY-009: ShapeNode and FieldValue::Shape ────────────────────────────

    #[test]
    fn test_bc_1_09_009_shape_node_all_fields_constructible() {
        // AC-009: ShapeNode must be constructible with all five fields set.
        let span = dummy_span();
        let node = ShapeNode {
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
        };
        assert_eq!(node.shape_type.as_ref().unwrap().value(), "rectangle");
        assert_eq!(node.position.as_ref().unwrap().value(), "50,50,200,100");
        assert_eq!(node.fill.as_ref().unwrap().value(), "#0070C0");
        assert!(node.text.is_some());
        assert_eq!(
            node.alt.as_ref().unwrap().value(),
            "A blue rectangle labelled Click here"
        );
    }

    #[test]
    fn test_bc_1_09_009_shape_node_all_none_is_valid_at_parse_time() {
        // AC-009: all fields are optional at parse time (validator enforces alt later).
        let node = ShapeNode {
            shape_type: None,
            position: None,
            fill: None,
            text: None,
            alt: None,
        };
        assert!(node.shape_type.is_none());
        assert!(node.position.is_none());
        assert!(node.fill.is_none());
        assert!(node.text.is_none());
        assert!(node.alt.is_none());
    }

    #[test]
    fn test_bc_1_09_009_shape_node_derives_hash_eq_clone_debug() {
        // AC-015: ShapeNode must implement Hash + Eq + Clone + Debug.
        let node = ShapeNode {
            shape_type: None,
            position: None,
            fill: None,
            text: None,
            alt: None,
        };
        let node2 = node.clone();
        assert_eq!(node, node2);
        let _ = format!("{node:?}");
        let mut set = HashSet::new();
        set.insert(node);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_09_009_field_value_shape_variant_hash_eq_clone_debug() {
        // AC-015: FieldValue::Shape must implement Hash + Eq + Clone + Debug.
        let node = ShapeNode {
            shape_type: None,
            position: None,
            fill: None,
            text: None,
            alt: None,
        };
        let fv = FieldValue::Shape(Box::new(node));
        let fv2 = fv.clone();
        assert_eq!(fv, fv2);
        let _ = format!("{fv:?}");
        let mut set = HashSet::new();
        set.insert(fv);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_09_009_field_value_shape_is_distinct_from_other_variants() {
        // FieldValue::Shape must not equal FieldValue::Ident("shape") or Template.
        let shape_fv = FieldValue::Shape(Box::new(ShapeNode {
            shape_type: None,
            position: None,
            fill: None,
            text: None,
            alt: None,
        }));
        let ident_fv = FieldValue::Ident("shape".to_string());
        assert_ne!(shape_fv, ident_fv);
        let template_fv = FieldValue::Template(vec![TemplateChunk::Literal("shape".to_string())]);
        assert_ne!(shape_fv, template_fv);
    }
}
