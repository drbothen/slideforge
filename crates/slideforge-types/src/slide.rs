//! The `Slide` type — a single slide in the semantic IR.
//!
//! A [`Slide`] is the semantic, pre-layout representation of one slide. It
//! carries the slide type keyword, field values, content blocks, register,
//! tags, and source location.

use std::sync::Arc;

use crate::block::Block;
use crate::inline::InlineNode;
use crate::ordered_map::OrderedMap;
use crate::register::Register;
use crate::span::SourceSpan;
use crate::value::Value;

/// An interpolated string part — either a literal or an expression.
///
/// Field values like titles may contain `{{ expr }}` interpolations. Before
/// evaluation, they are stored as a `Vec<StringPart>`. After evaluation, they
/// resolve to a plain `Arc<str>`.
///
/// The DSL supports full expressions inside `{{ }}`, not just variable names.
/// For example: `{{ count + 1 }}`, `{{ name | upper }}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StringPart {
    /// A literal text fragment.
    Literal(Arc<str>),
    /// A `{{ expr }}` expression fragment (full expression, not just a variable).
    Expr(Arc<str>),
}

/// A field value in a slide header — either a literal [`Value`], a raw
/// expression string, or an interpolated string (sequence of [`StringPart`]).
///
/// Slide field values (e.g., `title`, `subtitle`, `speaker`) may be:
/// - A plain `Value` (integer, boolean, null, etc.) — stored as `Literal`
/// - A raw expression string (`{{ count + 1 }}`) — stored as `Expr`
/// - A string with mixed literals and `{{ expr }}` parts — stored as `Interpolated`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldValue {
    /// A resolved scalar value (no further evaluation needed).
    Literal(Value),
    /// A raw expression string to be evaluated (e.g., `"{{ count + 1 }}"`).
    Expr(Arc<str>),
    /// An interpolated string (mix of literal fragments and expression fragments).
    Interpolated(Vec<StringPart>),
    /// An inline content sequence (for rich-text field values).
    Inlines(Vec<InlineNode>),
}

/// A single slide in the semantic IR.
///
/// `Slide` is produced by the parser and consumed by the evaluator, layout
/// engine, and exporters. After evaluation, variable references in `fields`
/// are resolved to concrete [`Value`]s.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slide {
    /// The slide type keyword (e.g., `"title"`, `"bullets"`, `"chart"`).
    pub slide_type: Arc<str>,

    /// Slide header fields (e.g., `title`, `subtitle`, `speaker`).
    /// Field values may contain unresolved interpolations at parse time.
    pub fields: OrderedMap<Arc<str>, FieldValue>,

    /// The body content blocks of this slide.
    pub blocks: Vec<Block>,

    /// The writing register for register-gated content.
    ///
    /// `None` means the slide appears in all registers. A `Some(Register::Notes)`
    /// value means the slide is gated to presenter-notes output only.
    pub register: Option<Register>,

    /// User-defined tags for filtering and grouping.
    pub tags: Vec<Arc<str>>,

    /// Source location of the slide declaration.
    pub source_span: SourceSpan,
}

impl Slide {
    /// Return the slide's title field as a string slice, if it is a plain
    /// `Value::Str` field value. Returns `None` if absent or not yet resolved.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::{Slide, FieldValue, Value, Register, SourceSpan, OrderedMap};
    /// use std::sync::Arc;
    ///
    /// let mut fields = OrderedMap::new();
    /// fields.insert(Arc::from("title"), FieldValue::Literal(Value::Str(Arc::from("My Slide"))));
    /// let slide = Slide {
    ///     slide_type: Arc::from("title"),
    ///     fields,
    ///     blocks: vec![],
    ///     register: None,
    ///     tags: vec![],
    ///     source_span: SourceSpan::default(),
    /// };
    /// assert_eq!(slide.title_str(), Some("My Slide"));
    /// ```
    #[must_use]
    pub fn title_str(&self) -> Option<&str> {
        match self.fields.get("title") {
            Some(FieldValue::Literal(Value::Str(s))) => Some(s.as_ref()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn make_minimal_slide() -> Slide {
        Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // AC-002 — Slide has correct fields, implements Hash+Eq+Clone+Debug
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_002_slide_fields_present() {
        let slide = make_minimal_slide();
        assert_eq!(slide.slide_type.as_ref(), "title");
        assert!(slide.fields.is_empty());
        assert!(slide.blocks.is_empty());
        assert!(slide.register.is_none());
        assert!(slide.tags.is_empty());
    }

    #[test]
    fn test_bc_1_01_002_slide_clone() {
        let slide = make_minimal_slide();
        let slide2 = slide.clone();
        assert_eq!(slide, slide2);
    }

    #[test]
    fn test_bc_1_01_002_slide_eq() {
        let a = make_minimal_slide();
        let b = make_minimal_slide();
        assert_eq!(a, b);
    }

    #[test]
    fn test_bc_1_01_002_slide_hash() {
        use std::collections::HashMap;
        let mut map: HashMap<Slide, &str> = HashMap::new();
        map.insert(make_minimal_slide(), "slide");
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_bc_1_01_002_slide_debug() {
        let slide = make_minimal_slide();
        let s = format!("{slide:?}");
        assert!(s.contains("Slide"));
    }

    #[test]
    fn test_bc_1_01_002_slide_title_str_present() {
        let mut slide = make_minimal_slide();
        slide
            .fields
            .insert(Arc::from("title"), FieldValue::Literal(Value::Str(Arc::from("Hello World"))));
        assert_eq!(slide.title_str(), Some("Hello World"));
    }

    #[test]
    fn test_bc_1_01_002_slide_title_str_absent() {
        let slide = make_minimal_slide();
        assert_eq!(slide.title_str(), None);
    }

    #[test]
    fn test_bc_1_01_002_string_part_literal() {
        let part = StringPart::Literal(Arc::from("hello"));
        assert!(matches!(part, StringPart::Literal(_)));
    }

    #[test]
    fn test_bc_1_01_002_string_part_expr() {
        // StringPart::Expr holds a full expression (not just a var name)
        let part = StringPart::Expr(Arc::from("count + 1"));
        assert!(matches!(part, StringPart::Expr(_)));
    }

    #[test]
    fn test_bc_1_01_002_field_value_variants() {
        let fv_literal = FieldValue::Literal(Value::Int(42));
        let fv_expr = FieldValue::Expr(Arc::from("{{ count + 1 }}"));
        let fv_interpolated = FieldValue::Interpolated(vec![StringPart::Literal(Arc::from("hi"))]);
        let fv_inlines = FieldValue::Inlines(vec![]);
        assert!(matches!(fv_literal, FieldValue::Literal(_)));
        assert!(matches!(fv_expr, FieldValue::Expr(_)));
        assert!(matches!(fv_interpolated, FieldValue::Interpolated(_)));
        assert!(matches!(fv_inlines, FieldValue::Inlines(_)));
    }

    #[test]
    fn test_bc_1_01_002_slide_register_is_option() {
        // AC-002: register is Option<Register>
        let slide_no_register = make_minimal_slide();
        assert!(slide_no_register.register.is_none());

        let slide_with_register = Slide {
            slide_type: Arc::from("bullets"),
            fields: OrderedMap::new(),
            blocks: vec![],
            register: Some(Register::Notes),
            tags: vec![],
            source_span: SourceSpan::default(),
        };
        assert_eq!(slide_with_register.register, Some(Register::Notes));
    }

    #[test]
    fn test_bc_1_01_002_slide_with_tags() {
        let mut slide = make_minimal_slide();
        slide.tags.push(Arc::from("intro"));
        slide.tags.push(Arc::from("keynote"));
        assert_eq!(slide.tags.len(), 2);
    }
}
