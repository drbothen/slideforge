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
//! - One or more [`SlideNode`] entries (the actual slides).
//!
//! Each [`SlideNode`] holds a slide type keyword and a list of [`FieldNode`]
//! entries mapping field names to [`FieldValue`] variants.

use crate::span::Spanned;

// ─── FieldValue ──────────────────────────────────────────────────────────────

/// The value of a DSL field.
///
/// Parsed from the right-hand side of a field assignment:
/// `title "My Title"` → [`FieldValue::Str`].
///
/// The `Error` sentinel is used by the error-recovery path when parsing fails
/// mid-value. It allows the parser to continue accumulating errors without
/// abandoning the entire parse tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldValue {
    /// A double-quoted string literal (quotes already stripped).
    Str(String),
    /// An integer literal.
    Num(i64),
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
///   client "Acme Corp"
///   year 2024
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlideNode {
    /// The slide type keyword.
    pub kind: Spanned<String>,
    /// Tags applied to this slide (always empty in STORY-006; extended in STORY-008).
    pub tags: Vec<Spanned<String>>,
    /// Field assignments inside this slide block, in source order.
    pub fields: Vec<FieldNode>,
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
    /// All `slide` blocks, in source order.
    pub slides: Vec<Spanned<SlideNode>>,
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;
    use std::collections::HashSet;

    fn dummy_span() -> Span {
        Span::new(0, 0, 0)
    }

    #[test]
    fn test_bc_1_01_001_field_value_hash_eq_clone_debug() {
        let v = FieldValue::Str("hello".to_string());
        let v2 = v.clone();
        assert_eq!(v, v2);
        let _ = format!("{v:?}");
        let mut set = HashSet::new();
        set.insert(v);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_01_001_field_value_variants_all_constructible() {
        let _ = FieldValue::Str("s".to_string());
        let _ = FieldValue::Num(42);
        let _ = FieldValue::Ident("foo".to_string());
        let _ = FieldValue::Error;
    }

    #[test]
    fn test_bc_1_01_001_deck_node_default_empty() {
        let d = DeckNode::default();
        assert!(d.version.is_none());
        assert!(d.lang.is_none());
        assert!(d.brand.is_none());
        assert!(d.vars.is_empty());
        assert!(d.set_rules.is_empty());
        assert!(d.slides.is_empty());
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
            value: Spanned::new(FieldValue::Str("Conf".to_string()), dummy_span()),
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
}
