// ast.rs — Typed AST for the slideforge mini-DSL spike.
//
// This is a SUBSET of the full slideforge AST — enough to demonstrate
// that the parser can produce a typed, spanned tree from the token stream.
//
// Every AST node carries a SimpleSpan (byte offsets into original source).
// This satisfies the requirement: "every error carries file:line:col span".
// (file:line:col is computed from byte offsets during error reporting.)

use chumsky::span::SimpleSpan;

/// Top-level document. A slide file is a sequence of top-level items.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub items: Vec<Spanned<TopLevelItem>>,
}

/// Top-level items: metadata block, slide, section, directives.
#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelItem {
    Metadata(Vec<Spanned<Field>>),
    Slide(SlideNode),
    Include(String),
    /// Placeholder produced during error recovery.
    Error,
}

/// A `slide <type>:` block.
#[derive(Debug, Clone, PartialEq)]
pub struct SlideNode {
    /// The slide type keyword (e.g., "title", "content", "metric_tree").
    pub slide_type: Spanned<String>,
    /// Optional string argument (e.g., `slide "Quoted Title":` uses the type
    /// "title" with an explicit title string).
    pub label: Option<Spanned<String>>,
    /// The body fields of the slide.
    pub fields: Vec<Spanned<Field>>,
}

/// A field inside a slide or metadata block.
/// Fields are key:value pairs where value can be various scalar or block types.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub key: Spanned<String>,
    pub value: Spanned<Value>,
}

/// A field value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Double-quoted string, possibly with {{ }} interpolation markers
    /// preserved as-is (interpolation expansion happens in eval phase).
    Str(String),
    /// Bare word / color name / identifier used as a value (e.g., `color blue`).
    Ident(String),
    /// Integer (preserved as string, no coercion).
    Int(String),
    /// Float (preserved as string, no coercion).
    Float(String),
    Bool(bool),
    /// Nested block (e.g., `notes:\n  ...\n`).
    Block(Vec<Spanned<Field>>),
    /// Bullet list: `- "item"` repeated.
    List(Vec<Spanned<Value>>),
    /// Multi-line text (from triple-quoted string or indented block).
    MultilineText(String),
    /// Error placeholder from parser recovery.
    Error,
}

/// A value annotated with its source span.
pub type Spanned<T> = (T, SimpleSpan);
