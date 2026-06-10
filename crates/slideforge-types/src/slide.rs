//! The `Slide` type — a single slide in the semantic IR.
//!
//! A [`Slide`] is the semantic, pre-layout representation of one slide. It
//! carries the slide type keyword, field values, content blocks, register,
//! tags, source location, and post-evaluation register content.

use std::sync::Arc;

use crate::block::Block;
use crate::inline::InlineNode;
use crate::ordered_map::OrderedMap;
use crate::register::{Register, RegisteredContent};
use crate::slide_overlay::SlideOverlay;
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
    /// A list of per-item inline content sequences.
    ///
    /// Produced by `eval_slide_node` when a `bullets:` (or other
    /// `INLINE_CONTENT_FIELDS`) field is written as a list-literal
    /// (`bullets: ["**bold**", "_italic_", "plain"]`) AND at least one item
    /// contains inline markup variants. Each `Vec<InlineNode>` corresponds to
    /// one bullet item.
    ///
    /// Plain-only list-literals produce `Literal(Value::List([Str(...)]))` instead
    /// (STORY-088 behaviour is preserved for plain items).
    ///
    /// `Hash + Eq + Clone` are derived from the inner `Vec<Vec<InlineNode>>`.
    /// `InlineNode` already derives `Hash + Eq + Clone` (comemo / ADR-013 requirement).
    InlinesList(Vec<Vec<InlineNode>>),
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

    /// Per-slide brand overlay metadata (from `brand_overlay:` DSL block).
    ///
    /// - `None` — no `brand_overlay:` block on this slide; deck-level brand applies.
    /// - `Some(overlay)` — this slide has a `brand_overlay:` block. The overlay
    ///   is stored as parse-time metadata; `slideforge-brand` interprets it at
    ///   export time via `resolve_overlay()`.
    ///
    /// See [`SlideOverlay`] for the single-master invariant enforcement (BC-2.02.002).
    pub overlay: Option<SlideOverlay>,

    /// Post-evaluation register-gated content for this slide.
    ///
    /// Populated by `slideforge-eval::register_routing::extract_register_content`
    /// after all field expressions are resolved. Initialized to `vec![]` by the
    /// parser; `eval_deck` populates it as a post-evaluation annotation.
    ///
    /// The layout engine copies this field verbatim into
    /// `LaidOutSlide::register_content` without re-deriving routing logic.
    /// Layout MUST NOT call any extraction function — it reads this field only.
    ///
    /// # Ordering invariant (BC-1.14.004 invariant 3)
    ///
    /// Entries are ordered `Notes < Report < Detail` regardless of field
    /// insertion order in `slide.fields`. This ordering is enforced by
    /// `slideforge-eval::register_routing::extract_register_content`.
    pub register_content: Vec<RegisteredContent>,
}

impl Slide {
    /// Return the slide's title field as a string slice, if it is a plain
    /// `Value::Str` field value. Returns `None` if absent or not yet resolved.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::{Slide, FieldValue, Value, Register, SourceSpan, OrderedMap};
    /// use slideforge_types::slide_overlay::SlideOverlay;
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
    ///     overlay: None,
    ///     register_content: vec![],
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
            overlay: None,
            register_content: vec![],
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
        slide.fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from("Hello World"))),
        );
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
        let fv_inlines_list = FieldValue::InlinesList(vec![]);
        assert!(matches!(fv_literal, FieldValue::Literal(_)));
        assert!(matches!(fv_expr, FieldValue::Expr(_)));
        assert!(matches!(fv_interpolated, FieldValue::Interpolated(_)));
        assert!(matches!(fv_inlines, FieldValue::Inlines(_)));
        assert!(matches!(fv_inlines_list, FieldValue::InlinesList(_)));
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
            overlay: None,
            register_content: vec![],
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

    // ──────────────────────────────────────────────────────────────────────────
    // STORY-025 / AC-011: Slide.overlay field — Red Gate tests
    // These tests MUST FAIL until the overlay field is wired to resolve_overlay.
    // They exercise the type presence and semantic properties of Slide.overlay.
    // ──────────────────────────────────────────────────────────────────────────

    /// AC-011 / BC-2.02.001 invariant 2: `Slide` has `overlay: Option<SlideOverlay>` field.
    ///
    /// Verifies the field compiles, can be set to `None`, and round-trips through
    /// `Hash + Clone` correctly.
    #[test]
    fn test_bc_2_02_001_slide_overlay_none_is_default() {
        let slide = make_minimal_slide();
        assert!(
            slide.overlay.is_none(),
            "Slide with no brand_overlay block must have overlay = None"
        );
    }

    /// AC-011 / BC-2.02.001 invariant 2: a `Slide` with `Some(SlideOverlay)` hashes and
    /// clones correctly (comemo compatibility).
    #[test]
    fn test_bc_2_02_001_slide_has_overlay_field_hash_clone() {
        use crate::slide_overlay::SlideOverlay;
        use std::collections::HashMap;

        let overlay = SlideOverlay {
            logo_path: Some(Arc::from("client-logo.png")),
            footer_text: Some(Arc::from("CONFIDENTIAL")),
            confidentiality: None,
            span: SourceSpan::default(),
        };
        let slide = Slide {
            slide_type: Arc::from("bullets"),
            fields: OrderedMap::new(),
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: Some(overlay.clone()),
            register_content: vec![],
        };
        let slide2 = slide.clone();
        assert_eq!(slide, slide2, "Slide with overlay must equal its clone");

        // Must be hashable for comemo.
        let mut map: HashMap<Slide, u32> = HashMap::new();
        map.insert(slide.clone(), 42);
        assert_eq!(map[&slide2], 42);
    }

    /// BC-2.02.002: `Slide.overlay` uses `SlideOverlay` — a type with NO master-path field.
    ///
    /// This is a structural compile-time test: if `SlideOverlay` had a `master_path`
    /// field, constructing it exhaustively here would fail to compile. The fact that
    /// the 4-field construction compiles proves the structural absence of master-switch
    /// capability (DI-016).
    #[test]
    fn test_bc_2_02_002_slide_overlay_type_has_no_master_path() {
        use crate::slide_overlay::SlideOverlay;

        // Exhaustive SlideOverlay construction — if any extra field (master_path,
        // template_path, layout_idx) were present, this would fail to compile.
        let overlay = SlideOverlay {
            logo_path: None,
            footer_text: None,
            confidentiality: None,
            span: SourceSpan::default(),
        };
        let slide = Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: Some(overlay),
            register_content: vec![],
        };
        // Structural invariant: overlay is metadata only, no master reference.
        assert!(slide.overlay.is_some());
        assert!(
            slide
                .overlay
                .as_ref()
                .expect("overlay was set Some above")
                .is_empty()
        );
    }
}
