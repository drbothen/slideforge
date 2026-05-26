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

/// An interpolated string part — either a literal or a variable reference.
///
/// Field values like titles may contain `{{ var }}` interpolations. Before
/// evaluation, they are stored as a `Vec<StringPart>`. After evaluation, they
/// resolve to a plain `Arc<str>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StringPart {
    /// A literal text fragment.
    Literal(Arc<str>),
    /// A `{{ var }}` interpolation reference.
    Var(Arc<str>),
}

/// A field value in a slide header — either a [`Value`] or an interpolated
/// string (sequence of [`StringPart`]).
///
/// Slide field values (e.g., `title`, `subtitle`, `speaker`) may be:
/// - A plain `Value` (integer, boolean, null, etc.)
/// - A string with optional `{{ var }}` interpolations (a `Vec<StringPart>`)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldValue {
    /// A resolved scalar value.
    Value(Value),
    /// An interpolated string (mix of literals and variable references).
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
    pub register: Register,

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
    /// fields.insert(Arc::from("title"), FieldValue::Value(Value::Str(Arc::from("My Slide"))));
    /// let slide = Slide {
    ///     slide_type: Arc::from("title"),
    ///     fields,
    ///     blocks: vec![],
    ///     register: Register::Notes,
    ///     tags: vec![],
    ///     source_span: SourceSpan::default(),
    /// };
    /// assert_eq!(slide.title_str(), Some("My Slide"));
    /// ```
    #[must_use]
    pub fn title_str(&self) -> Option<&str> {
        match self.fields.get("title") {
            Some(FieldValue::Value(Value::Str(s))) => Some(s.as_ref()),
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
            register: Register::Notes,
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
        assert_eq!(slide.register, Register::Notes);
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
            .insert(Arc::from("title"), FieldValue::Value(Value::Str(Arc::from("Hello World"))));
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
    fn test_bc_1_01_002_string_part_var() {
        let part = StringPart::Var(Arc::from("company_name"));
        assert!(matches!(part, StringPart::Var(_)));
    }

    #[test]
    fn test_bc_1_01_002_field_value_variants() {
        let fv_value = FieldValue::Value(Value::Int(42));
        let fv_interpolated = FieldValue::Interpolated(vec![StringPart::Literal(Arc::from("hi"))]);
        let fv_inlines = FieldValue::Inlines(vec![]);
        assert!(matches!(fv_value, FieldValue::Value(_)));
        assert!(matches!(fv_interpolated, FieldValue::Interpolated(_)));
        assert!(matches!(fv_inlines, FieldValue::Inlines(_)));
    }

    #[test]
    fn test_bc_1_01_002_slide_with_tags() {
        let mut slide = make_minimal_slide();
        slide.tags.push(Arc::from("intro"));
        slide.tags.push(Arc::from("keynote"));
        assert_eq!(slide.tags.len(), 2);
    }
}
