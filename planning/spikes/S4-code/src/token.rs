// token.rs — Token definitions for the slideforge mini-DSL spike.
//
// Tokens are produced by the hand-written lexer (lexer.rs) and consumed
// by the chumsky 0.10 parser (parser.rs).
//
// Every token carries no span of its own — spans are attached externally
// as (Token, SimpleSpan) pairs in the lexer output. This keeps the token
// enum lightweight and Clone-friendly.

use chumsky::span::SimpleSpan;

/// Every token the slideforge DSL lexer can produce.
///
/// The lexer inserts synthetic INDENT / DEDENT tokens around indented blocks,
/// so the parser sees a Python-style INDENT/DEDENT delimiter structure rather
/// than raw whitespace.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    // --- Structural (synthetic) ---
    /// Emitted when indentation increases (entering a block).
    Indent,
    /// Emitted when indentation decreases (leaving a block).
    Dedent,
    /// End of a logical line (emitted after each non-blank, non-comment line
    /// before its trailing DEDENT tokens).
    Newline,

    // --- Literals ---
    /// Bare identifier or keyword: `slide`, `title`, `blue`, `true`, etc.
    Ident(String),
    /// Double-quoted string: `"Hello"`.
    Str(String),
    /// Integer literal (value preserved as string to avoid coercion).
    /// Rationale: "1.10" must stay "1.10" — no implicit type coercion.
    Int(String),
    /// Float literal (value preserved as string — no implicit coercion).
    Float(String),
    /// Boolean `true`.
    True,
    /// Boolean `false`.
    False,

    // --- Punctuation ---
    Colon,      // `:`
    Dash,       // `-` (bullet list items)
    Percent,    // `%` (e.g., `width: 50%`)
    LBracket,   // `[`
    RBracket,   // `]`
    LBrace,     // `{{` (interpolation open)
    RBrace,     // `}}` (interpolation close)
    AtBrace,    // `@{` (math interpolation open)
    DollarDollar, // `$$` (display math)
    Dollar,     // `$` (inline math)

    // --- Directives ---
    Include,    // `@include`
    Import,     // `@import`
    If,         // `@if`
    Elif,       // `@elif`
    Else,       // `@else`
    For,        // `@for`
    In,         // `in` (used in @for)

    // --- End of input ---
    Eof,

    // --- Error placeholder (for error-recovery in parser) ---
    Error,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Indent => write!(f, "INDENT"),
            Token::Dedent => write!(f, "DEDENT"),
            Token::Newline => write!(f, "NEWLINE"),
            Token::Ident(s) => write!(f, "'{s}'"),
            Token::Str(s) => write!(f, "\"{s}\""),
            Token::Int(s) => write!(f, "{s}"),
            Token::Float(s) => write!(f, "{s}"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Colon => write!(f, ":"),
            Token::Dash => write!(f, "-"),
            Token::Percent => write!(f, "%"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::AtBrace => write!(f, "@{{"),
            Token::DollarDollar => write!(f, "$$"),
            Token::Dollar => write!(f, "$"),
            Token::Include => write!(f, "@include"),
            Token::Import => write!(f, "@import"),
            Token::If => write!(f, "@if"),
            Token::Elif => write!(f, "@elif"),
            Token::Else => write!(f, "@else"),
            Token::For => write!(f, "@for"),
            Token::In => write!(f, "in"),
            Token::Eof => write!(f, "EOF"),
            Token::Error => write!(f, "<error>"),
        }
    }
}

/// A token annotated with its source span (byte offsets into original `&str`).
pub type SpannedToken = (Token, SimpleSpan);

/// A lex-level error with a byte-offset span.
#[derive(Debug, Clone)]
pub struct LexError {
    pub span: SimpleSpan,
    pub kind: LexErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexErrorKind {
    /// Tab character found in indentation — only spaces are allowed.
    TabInIndent { line: usize },
    /// Indentation level does not match any previously-opened level (dedent mismatch).
    UnmatchedDedent { line: usize, found_spaces: usize },
    /// Unterminated double-quoted string.
    UnterminatedString { line: usize },
    /// Unterminated triple-quoted string (multiline).
    UnterminatedTripleString { start_line: usize },
    /// Unterminated `{{` interpolation — missing `}}`.
    UnterminatedInterpolation { line: usize },
    /// Unexpected character that cannot start any token.
    UnexpectedChar { ch: char, line: usize, col: usize },
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = self.span.start;
        match &self.kind {
            LexErrorKind::TabInIndent { line } => {
                write!(f, "[byte {start}, line {line}] tab character in indentation — use spaces only (hint: configure your editor to expand tabs to spaces)")
            }
            LexErrorKind::UnmatchedDedent { line, found_spaces } => {
                write!(f, "[byte {start}, line {line}] dedent to {found_spaces} spaces does not match any open indentation level (hint: check that indentation is a multiple of 4 spaces)")
            }
            LexErrorKind::UnterminatedString { line } => {
                write!(f, "[byte {start}, line {line}] unterminated string — missing closing '\"' (hint: strings cannot span multiple lines; use triple quotes for multiline text)")
            }
            LexErrorKind::UnterminatedTripleString { start_line } => {
                write!(f, "[byte {start}, line {start_line}] unterminated triple-quoted string — missing closing '\"\"\"'")
            }
            LexErrorKind::UnterminatedInterpolation { line } => {
                write!(f, "[byte {start}, line {line}] unterminated interpolation — missing closing '}}}}' (hint: use '{{{{ }}}}' for literal braces)")
            }
            LexErrorKind::UnexpectedChar { ch, line, col } => {
                write!(f, "[byte {start}, line {line}, col {col}] unexpected character '{ch}'")
            }
        }
    }
}
