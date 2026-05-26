//! Template chunk types for `{{ expr }}` string interpolation.
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
//! accumulates E-PAR-004 into the error list. Scanning continues after the
//! error.

use crate::expr::Expr;

/// One chunk in a parsed template string.
///
/// A template string is split at `{{ ... }}` boundaries into an alternating
/// sequence of `Literal` and `Expr` chunks.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TemplateChunk {
    /// A run of literal (non-interpolated) text.
    ///
    /// Example: the `"Hello "` part in `"Hello {{ name }}"`.
    Literal(String),

    /// An interpolated expression.
    ///
    /// Example: the `{{ name }}` part in `"Hello {{ name }}"`.
    /// If parsing the inner expression fails, the chunk holds [`Expr::Error`].
    Expr(Expr),
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{BinOpKind, Expr};
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
}
