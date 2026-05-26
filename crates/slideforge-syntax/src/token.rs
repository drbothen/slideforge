//! Token types for the slideforge DSL lexer.
//!
//! The [`Token`] enum covers every surface token that the hand-written lexer
//! can emit. The chumsky 0.10 parser (STORY-006+) consumes a
//! `Vec<Spanned<Token>>` as its input stream.
//!
//! # Design Notes
//!
//! * String values use `Arc<str>` throughout — consistent with the rest of the
//!   IR type system and required for comemo cache compatibility.
//! * Floating-point literals use [`ordered_float::OrderedFloat<f64>`] so that
//!   `Token` is `Hash + Eq`, which chumsky 0.10 requires for its input stream.
//! * `Spanned<T>` mirrors the convention used by chumsky 0.10's `SimpleSpan` /
//!   `Range<usize>` approach. The range is a byte-offset range into the source
//!   string.

use std::sync::Arc;

use ordered_float::OrderedFloat;

/// A half-open byte-offset range `[start, end)` into the source string.
///
/// This is the same span representation used by chumsky 0.10 — it maps
/// directly to `chumsky::span::SimpleSpan`.
pub type Spanned<T> = (T, std::ops::Range<usize>);

/// The lexer mode at the point where a token is produced.
///
/// The lexer maintains a mode stack so that `$...$` and `$$...$$` regions
/// are tokenized differently from surrounding text. The parser (STORY-006+)
/// imports this type to validate mode-specific token sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LexerMode {
    /// Normal DSL text outside any math delimiter.
    Text,
    /// Inside a single-dollar inline math region (`$...$`).
    Math,
    /// Inside a double-dollar display math region (`$$...$$`).
    MathDisplay,
}

/// A single token emitted by the slideforge lexer.
///
/// Every variant either carries no payload (punctuation tokens) or carries a
/// value using `Arc<str>` / `OrderedFloat<f64>` / `i64` / `bool` so that the
/// whole enum is `Hash + Eq + Clone + Debug`.
///
/// # Token Inventory (AC-008)
///
/// | Variant | DSL surface |
/// |---------|------------|
/// | `Ident` | An unquoted identifier: `title`, `slide`, `@for`, etc. |
/// | `StringLit` | A quoted string value: `"My Deck"` |
/// | `IntLit` | An integer literal: `42` |
/// | `FloatLit` | A floating-point literal: `3.14` |
/// | `BoolLit` | `true` or `false` |
/// | `At` | The bare `@` character (directive prefix) |
/// | `DollarSingle` | `$` — inline math delimiter |
/// | `DollarDouble` | `$$` — display math delimiter |
/// | `DoubleBrace` | `{{` — text-mode interpolation open |
/// | `CloseBrace` | `}}` or `}` — interpolation close |
/// | `AtBrace` | `@{` — math-mode variable interpolation |
/// | `Colon` | `:` |
/// | `Pipe` | `\|` |
/// | `Newline` | End of a logical line |
/// | `Indent` | Indentation increased to `n` spaces |
/// | `Dedent` | Indentation decreased (one token per level popped) |
/// | `Eof` | End of the token stream |
/// | `MathContent` | Raw LaTeX content collected in Math/MathDisplay mode |
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    // ── Value-carrying tokens ─────────────────────────────────────────────

    /// An unquoted identifier or keyword (`title`, `slide`, `for`, …).
    Ident(Arc<str>),

    /// A double-quoted string literal.
    ///
    /// The value is the *contents* of the string (quotes stripped, escape
    /// sequences preserved as-is for now; the parser resolves escapes).
    StringLit(Arc<str>),

    /// An integer literal (`42`, `-1`).
    IntLit(i64),

    /// A floating-point literal (`3.14`, `0.5`).
    FloatLit(OrderedFloat<f64>),

    /// A boolean literal (`true` or `false`).
    BoolLit(bool),

    /// Raw LaTeX content collected while the lexer is in [`LexerMode::Math`]
    /// or [`LexerMode::MathDisplay`].
    MathContent(Arc<str>),

    // ── Punctuation / delimiter tokens ────────────────────────────────────

    /// The `@` character — directive prefix.
    At,

    /// A single `$` — inline math delimiter.
    DollarSingle,

    /// A `$$` — display math delimiter.
    DollarDouble,

    /// `{{` — text-mode variable interpolation open delimiter.
    DoubleBrace,

    /// `}}` or `}` — interpolation close delimiter.
    CloseBrace,

    /// `@{` — math-mode variable interpolation open delimiter.
    AtBrace,

    /// `:` — field separator.
    Colon,

    /// `|` — pipe / variant separator.
    Pipe,

    // ── Structural tokens ─────────────────────────────────────────────────

    /// End of a logical line.
    Newline,

    /// Indentation level increased; carries the **absolute** number of leading
    /// spaces on the current line.
    ///
    /// `Indent(n)` means "we are now at indentation level `n`". Using the
    /// absolute level (not a delta) keeps the parser's grammar context-free.
    Indent(usize),

    /// Indentation level decreased by one step.
    ///
    /// The lexer emits one `Dedent` for each level popped off the indent
    /// stack. It carries no data — the parser maintains its own context
    /// stack and pops when it sees `Dedent`.
    Dedent,

    /// End of the token stream.
    Eof,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc_1_01_003_token_derives_hash_eq_clone_debug() {
        use std::collections::HashSet;
        let t = Token::Ident(Arc::from("title"));
        let t2 = t.clone();
        assert_eq!(t, t2);
        let _ = format!("{t:?}");
        // Hash — just verify it compiles and doesn't panic
        let mut set = HashSet::new();
        set.insert(t);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_01_003_token_float_lit_is_ordered() {
        let a = Token::FloatLit(OrderedFloat(1.0_f64)); // use 1.0, not PI approximation
        let b = Token::FloatLit(OrderedFloat(1.0_f64));
        assert_eq!(a, b);
    }

    #[test]
    fn test_bc_1_01_003_lexer_mode_variants() {
        assert_ne!(LexerMode::Text, LexerMode::Math);
        assert_ne!(LexerMode::Math, LexerMode::MathDisplay);
        assert_ne!(LexerMode::Text, LexerMode::MathDisplay);
    }

    #[test]
    fn test_bc_1_01_003_spanned_type_alias() {
        let s: Spanned<Token> = (Token::Eof, 0..0);
        assert_eq!(s.1, 0..0);
    }

    #[test]
    fn test_bc_1_01_003_all_token_variants_constructible() {
        // Verify every AC-008 variant is reachable at the type level.
        let _variants: &[Token] = &[
            Token::Ident(Arc::from("x")),
            Token::StringLit(Arc::from("hello")),
            Token::IntLit(42),
            Token::FloatLit(OrderedFloat(1.5_f64)),
            Token::BoolLit(true),
            Token::MathContent(Arc::from("x^2")),
            Token::At,
            Token::DollarSingle,
            Token::DollarDouble,
            Token::DoubleBrace,
            Token::CloseBrace,
            Token::AtBrace,
            Token::Colon,
            Token::Pipe,
            Token::Newline,
            Token::Indent(4),
            Token::Dedent,
            Token::Eof,
        ];
    }
}
