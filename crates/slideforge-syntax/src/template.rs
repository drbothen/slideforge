//! Template chunk types for `{{ expr }}` string interpolation and math mode.
//!
//! A DSL field value like `"Hello {{ name }}"` is parsed as a
//! `Vec<TemplateChunk>` — a sequence of literal text segments and expression
//! interpolation segments.
//!
//! # Design
//!
//! Plain strings with no `{{ ... }}` are represented as a single-element vec:
//! `vec![TemplateChunk::Literal("Hello")]`. This keeps the type system uniform
//! — callers never need to inspect both `FieldValue::Str` and
//! `FieldValue::Template`.
//!
//! # Error Recovery
//!
//! A malformed `{{ ... }}` (missing `}}`) produces
//! `TemplateChunk::Expr(Expr::Error)` at the position of the failed chunk and
//! accumulates E-PAR-012 into the error list. An empty `{{ }}` accumulates
//! E-PAR-013. An unterminated math block (`$`/`$$`) accumulates E-PAR-014.
//! Scanning continues after each error.
//!
//! # STORY-009: Math mode chunks
//!
//! Fields containing `$...$` (inline) or `$$...$$` (display) math delimiters
//! produce [`TemplateChunk::MathInline`] or [`TemplateChunk::MathDisplay`]
//! respectively.  Inside a math region the only interpolation form is `@{var}`
//! (not `{{ var }}`); those produce [`TemplateChunk::MathInterp`].
//!
//! # STORY-077: Inline markup chunks (DIR-077-002 §3)
//!
//! Fields containing inline markup delimiters produce structural `TemplateChunk`
//! variants at parse time. These mirror `slideforge_types::InlineNode` structurally
//! but use only `TemplateChunk` children — preserving the crate boundary (syntax
//! must NOT depend on types). The eval stage converts these to `InlineNode` via
//! `chunks_to_inline_nodes` (in `slideforge-eval`).
//!
//! | DSL syntax | TemplateChunk variant |
//! |---|---|
//! | `**text**` | `Bold(Vec<TemplateChunk>)` |
//! | `_text_` | `Italic(Vec<TemplateChunk>)` |
//! | `` `text` `` | `Code(String)` (verbatim — no inner markup) |
//! | `[text](url)` | `Link { text: Vec<TemplateChunk>, url: String }` |
//! | `^text^` | `Superscript(Vec<TemplateChunk>)` |
//! | `~text~` | `Subscript(Vec<TemplateChunk>)` |
//! | `~~text~~` | `Strikethrough(Vec<TemplateChunk>)` (checked before `~`) |
//! | `==text==` | `Highlight(Vec<TemplateChunk>)` |

use crate::expr::Expr;

/// One chunk in a parsed template string.
///
/// A template string is split at `{{ ... }}` boundaries (text mode) or
/// `$...$` / `$$...$$` boundaries (math mode) into an alternating sequence of
/// chunk variants.
///
/// # STORY-009 Additions
///
/// | Variant | Source construct |
/// |---------|----------------|
/// | `MathInline(s)` | `$...$` inline math region (raw LaTeX content) |
/// | `MathDisplay(s)` | `$$...$$` display math region (raw LaTeX content) |
/// | `MathInterp(e)` | `@{expr}` variable interpolation inside a math region |
///
/// # STORY-077 Additions (DIR-077-002 §3)
///
/// | Variant | Source construct |
/// |---------|----------------|
/// | `Bold(children)` | `**text**` bold span |
/// | `Italic(children)` | `_text_` italic span |
/// | `Code(s)` | `` `text` `` code span (verbatim) |
/// | `Link { text, url }` | `[text](url)` hyperlink |
/// | `Superscript(children)` | `^text^` superscript |
/// | `Subscript(children)` | `~text~` subscript |
/// | `Strikethrough(children)` | `~~text~~` strikethrough |
/// | `Highlight(children)` | `==text==` highlighted |
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TemplateChunk {
    /// A run of literal (non-interpolated) text.
    ///
    /// Example: the `"Hello "` part in `"Hello {{ name }}"`.
    Literal(String),

    /// An interpolated expression in text mode.
    ///
    /// Example: the `{{ name }}` part in `"Hello {{ name }}"`.
    /// If parsing the inner expression fails, the chunk holds [`Expr::Error`].
    Expr(Expr),

    /// The raw LaTeX content of an inline `$...$` math region.
    ///
    /// Example: `title "$x^2$"` → `[MathInline("x^2")]`.
    ///
    /// The delimiters themselves are stripped; only the content is stored.
    /// `@{var}` interpolations inside the math region are represented as
    /// separate [`TemplateChunk::MathInterp`] chunks (not embedded in this
    /// string).
    MathInline(String),

    /// The raw LaTeX content of a display `$$...$$` math region.
    ///
    /// Example: `body "$$\\sum_{i=0}^{n} i$$"` → `[MathDisplay("\\sum_{i=0}^{n} i")]`.
    ///
    /// The delimiters themselves are stripped; only the content is stored.
    MathDisplay(String),

    /// A variable interpolation inside a math region: `@{expr}`.
    ///
    /// Math mode uses `@{var}` for interpolation (not `{{ var }}`).
    /// Example: `title "$@{base}^2$"` → `[MathInterp(Expr::Ident("base"))]`.
    ///
    /// `{{ var }}` inside a math region is treated as literal text (the `{{`
    /// characters become part of the LaTeX source), not an interpolation.
    MathInterp(Expr),

    // ── STORY-077: Inline markup variants (DIR-077-002 §3) ────────────────────
    /// Bold text span: `**text**`.
    ///
    /// Children may contain any `TemplateChunk` variant (including `Italic`,
    /// `Expr`, `Code`, `Link`, etc.) but NOT another `Bold` at the same nesting
    /// level (a closing `**` ends the bold span).
    ///
    /// Eval-time mapping: `Bold(children)` → `InlineNode::Bold(chunks_to_inline_nodes(children))`
    Bold(Vec<TemplateChunk>),

    /// Italic text span: `_text_`.
    ///
    /// Single-underscore pair. Double-underscore is NOT supported in v1.
    /// (DIR-077-002 §1 disambiguation rule 2: single `*` is NOT italic — only `_`.)
    ///
    /// Eval-time mapping: `Italic(children)` → `InlineNode::Italic(chunks_to_inline_nodes(children))`
    Italic(Vec<TemplateChunk>),

    /// Inline code span: `` `text` ``.
    ///
    /// Content is verbatim — no further inline markup processing occurs inside
    /// a code span (`{{ }}` interpolation does NOT fire inside a code span).
    ///
    /// Eval-time mapping: `Code(s)` → `InlineNode::Code(Arc::from(s))`
    Code(String),

    /// Hyperlink: `[text](url)`.
    ///
    /// The `text` portion IS further processed for nested inline markup.
    /// The `url` portion is verbatim text (not processed for markup or interpolation).
    ///
    /// Eval-time mapping: `Link { text, url }` → `InlineNode::Link { text: chunks_to_inline_nodes(text), url: Arc::from(url) }`
    Link {
        /// The display content of the link (processed for nested inline markup).
        text: Vec<TemplateChunk>,
        /// The target URL (verbatim — not processed for markup or `{{ }}`).
        url: String,
    },

    /// Superscript span: `^text^`.
    ///
    /// Eval-time mapping: `Superscript(children)` → `InlineNode::Superscript(chunks_to_inline_nodes(children))`
    Superscript(Vec<TemplateChunk>),

    /// Subscript span: `~text~`.
    ///
    /// Single-tilde pair. The parser MUST check `~~` (Strikethrough) BEFORE
    /// checking `~` (Subscript) to resolve the disambiguation (DIR-077-002 §1
    /// disambiguation rule 1).
    ///
    /// Eval-time mapping: `Subscript(children)` → `InlineNode::Subscript(chunks_to_inline_nodes(children))`
    Subscript(Vec<TemplateChunk>),

    /// Strikethrough span: `~~text~~`.
    ///
    /// Double-tilde pair. Consumed greedily before single-tilde subscript.
    ///
    /// Eval-time mapping: `Strikethrough(children)` → `InlineNode::Strikethrough(chunks_to_inline_nodes(children))`
    Strikethrough(Vec<TemplateChunk>),

    /// Highlighted span: `==text==`.
    ///
    /// Double-equals pair.
    ///
    /// Eval-time mapping: `Highlight(children)` → `InlineNode::Highlight(chunks_to_inline_nodes(children))`
    Highlight(Vec<TemplateChunk>),
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{BinOpKind, Expr};
    #[allow(unused_imports)]
    use std::collections::HashSet;

    #[test]
    fn test_bc_1_04_001_template_chunk_derives_hash_eq_clone_debug() {
        let c = TemplateChunk::Literal("hello".to_string());
        let c2 = c.clone();
        assert_eq!(c, c2);
        let _ = format!("{c:?}");
        let mut set = HashSet::new();
        set.insert(c);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_04_001_template_chunk_literal_and_expr_constructible() {
        let lit = TemplateChunk::Literal("text".to_string());
        let expr_chunk = TemplateChunk::Expr(Expr::Ident("name".to_string()));
        let _ = format!("{lit:?}");
        let _ = format!("{expr_chunk:?}");
    }

    #[test]
    fn test_bc_1_04_001_template_chunk_expr_with_binop() {
        // AC-008: {{ x + 1 }} parses to Expr::BinOp { op: Add, ... }
        let binop = Expr::BinOp {
            op: BinOpKind::Add,
            lhs: Box::new(Expr::Ident("x".to_string())),
            rhs: Box::new(Expr::Num(1)),
        };
        let chunk = TemplateChunk::Expr(binop.clone());
        let TemplateChunk::Expr(inner) = &chunk else {
            panic!("expected Expr chunk");
        };
        assert_eq!(inner, &binop);
    }

    #[test]
    fn test_bc_1_04_001_template_chunk_error_sentinel() {
        // Malformed {{ ... }} must produce Expr::Error in the chunk.
        let chunk = TemplateChunk::Expr(Expr::Error);
        assert_eq!(chunk, TemplateChunk::Expr(Expr::Error));
    }

    #[test]
    fn test_bc_1_04_001_plain_string_is_single_literal_chunk() {
        // A plain string "Hello" with no interpolation is a single Literal chunk.
        let chunks = [TemplateChunk::Literal("Hello".to_string())];
        assert_eq!(chunks.len(), 1);
        assert!(matches!(&chunks[0], TemplateChunk::Literal(s) if s == "Hello"));
    }

    // ── STORY-009: new math chunk variants ───────────────────────────────────

    #[test]
    fn test_bc_1_09_006_math_inline_chunk_derives_hash_eq_clone_debug() {
        // AC-006: MathInline chunk must implement Hash + Eq + Clone + Debug.
        let c = TemplateChunk::MathInline("x^2".to_string());
        let c2 = c.clone();
        assert_eq!(c, c2);
        let _ = format!("{c:?}");
        let mut set = HashSet::new();
        set.insert(c);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_09_007_math_display_chunk_derives_hash_eq_clone_debug() {
        // AC-007: MathDisplay chunk must implement Hash + Eq + Clone + Debug.
        let c = TemplateChunk::MathDisplay(r"\sum_{i=0}^{n} i".to_string());
        let c2 = c.clone();
        assert_eq!(c, c2);
        let _ = format!("{c:?}");
        let mut set = HashSet::new();
        set.insert(c);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_09_008_math_interp_chunk_derives_hash_eq_clone_debug() {
        // AC-008: MathInterp chunk must implement Hash + Eq + Clone + Debug.
        let c = TemplateChunk::MathInterp(Expr::Ident("base".to_string()));
        let c2 = c.clone();
        assert_eq!(c, c2);
        let _ = format!("{c:?}");
        let mut set = HashSet::new();
        set.insert(c);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_09_006_math_inline_holds_raw_latex_content() {
        // AC-006: MathInline stores the raw LaTeX content between the $ delimiters.
        let c = TemplateChunk::MathInline("x^2 + y^2 = z^2".to_string());
        let TemplateChunk::MathInline(content) = &c else {
            panic!("expected MathInline");
        };
        assert_eq!(content, "x^2 + y^2 = z^2");
    }

    #[test]
    fn test_bc_1_09_007_math_display_holds_raw_latex_content() {
        // AC-007: MathDisplay stores the raw LaTeX content between the $$ delimiters.
        let c = TemplateChunk::MathDisplay(r"\frac{1}{2}".to_string());
        let TemplateChunk::MathDisplay(content) = &c else {
            panic!("expected MathDisplay");
        };
        assert_eq!(content, r"\frac{1}{2}");
    }

    #[test]
    fn test_bc_1_09_008_math_interp_holds_expression() {
        // AC-008: MathInterp holds a parsed Expr for the variable reference.
        let expr = Expr::Ident("my_var".to_string());
        let c = TemplateChunk::MathInterp(expr.clone());
        let TemplateChunk::MathInterp(inner) = &c else {
            panic!("expected MathInterp");
        };
        assert_eq!(inner, &expr);
    }

    #[test]
    fn test_bc_1_09_006_math_inline_distinct_from_literal() {
        // MathInline("x^2") must not equal Literal("x^2").
        let math = TemplateChunk::MathInline("x^2".to_string());
        let lit = TemplateChunk::Literal("x^2".to_string());
        assert_ne!(math, lit, "MathInline and Literal must be distinct chunks");
    }

    #[test]
    fn test_bc_1_09_007_math_display_distinct_from_math_inline() {
        // MathDisplay("x") must not equal MathInline("x").
        let display = TemplateChunk::MathDisplay("x".to_string());
        let inline = TemplateChunk::MathInline("x".to_string());
        assert_ne!(
            display, inline,
            "MathDisplay and MathInline must be distinct"
        );
    }

    #[test]
    fn test_bc_1_09_008_all_math_chunk_variants_in_hash_set_are_distinct() {
        // All five chunk variants must be distinct in a HashSet.
        let chunks: Vec<TemplateChunk> = vec![
            TemplateChunk::Literal("x".to_string()),
            TemplateChunk::Expr(Expr::Ident("x".to_string())),
            TemplateChunk::MathInline("x".to_string()),
            TemplateChunk::MathDisplay("x".to_string()),
            TemplateChunk::MathInterp(Expr::Ident("x".to_string())),
        ];
        let mut set = HashSet::new();
        for c in &chunks {
            set.insert(c.clone());
        }
        assert_eq!(
            set.len(),
            5,
            "all 5 TemplateChunk variants with same inner value must be distinct"
        );
    }
}
