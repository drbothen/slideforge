//! Internal math AST types.
//!
//! The math pipeline uses a two-step process:
//!
//! 1. **Parser** (`parser.rs`) converts raw LaTeX source into a [`MathAst`].
//! 2. **Renderer** (`omml.rs`) walks the [`MathAst`] to produce OMML XML.
//!
//! [`MathAst`] is distinct from [`slideforge_types::MathNode`] — the types
//! crate's `MathNode` carries only raw LaTeX source and metadata; the AST is
//! internal to `slideforge-math`.

use std::sync::Arc;

/// The rendering mode for a math expression.
///
/// Determines whether the OMML output is wrapped in an inline `<m:oMath>` or
/// a block-level `<m:oMathPara>` element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathMode {
    /// Inline math: `$...$`. Rendered as `<m:oMath>` inside a run.
    Inline,
    /// Display (block) math: `$$...$$`. Rendered as `<m:oMathPara>`.
    Display,
}

/// The kind of accent applied to a math node (e.g., hat, bar, tilde).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccentKind {
    /// `\hat{x}` — circumflex accent.
    Hat,
    /// `\bar{x}` — overbar accent.
    Bar,
    /// `\tilde{x}` — tilde accent.
    Tilde,
    /// `\vec{x}` — vector arrow.
    Vec,
    /// `\dot{x}` — dot accent.
    Dot,
    /// `\ddot{x}` — double dot (dieresis) accent.
    Ddot,
}

/// A single node in the math AST.
///
/// All variants derive `Hash + Eq + Clone + Debug` for comemo compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MathNode {
    /// A plain text token (identifier, digit, operator character).
    ///
    /// Rendered in math (italic) style by default.
    Text(Arc<str>),

    /// An upright text run from `\text{...}` (roman/plain style, not italic).
    ///
    /// Distinct from [`MathNode::Text`] which represents math identifiers
    /// (rendered italic). `TextRun` content appears in OMML with
    /// `<m:sty m:val="p"/>` to produce upright/roman rendering.
    TextRun(Arc<str>),

    /// Superscript: `base^{sup}`.
    Superscript {
        /// The base expression.
        base: Box<MathNode>,
        /// The superscript expression.
        sup: Box<MathNode>,
    },

    /// Subscript: `base_{sub}`.
    Subscript {
        /// The base expression.
        base: Box<MathNode>,
        /// The subscript expression.
        sub: Box<MathNode>,
    },

    /// Fraction: `\frac{num}{denom}`.
    Fraction {
        /// The numerator expression.
        num: Box<MathNode>,
        /// The denominator expression.
        denom: Box<MathNode>,
    },

    /// Square root or nth root: `\sqrt[index]{radicand}`.
    Sqrt {
        /// Optional index for nth root (e.g., `\sqrt[3]{x}` gives `index = Some(3-node)`).
        index: Option<Box<MathNode>>,
        /// The radicand expression.
        radicand: Box<MathNode>,
    },

    /// A large operator such as `\sum`, `\prod`, `\int`, `\lim`, etc.
    ///
    /// The string holds the canonical command name (without backslash).
    Operator(Arc<str>),

    /// A Greek letter such as `\alpha`, `\beta`, `\gamma`, etc.
    ///
    /// The string holds the canonical command name (without backslash).
    Greek(Arc<str>),

    /// A math symbol such as `\cdot`, `\times`, `\infty`, `\pm`, etc.
    ///
    /// The string holds the canonical command name (without backslash).
    Symbol(Arc<str>),

    /// An accent applied to an inner expression.
    Accent {
        /// The kind of accent.
        kind: AccentKind,
        /// The inner expression being accented.
        inner: Box<MathNode>,
    },

    /// A braced group `{...}` containing zero or more nodes.
    Group(Vec<MathNode>),

    /// A delimited expression `\left( ... \right)` etc.
    Delimiter {
        /// The left delimiter character (e.g., `"("`, `"["`, `"\\{"`).
        left: Arc<str>,
        /// The right delimiter character (e.g., `")"`, `"]"`, `"\\}"`).
        right: Arc<str>,
        /// The contents between the delimiters.
        inner: Vec<MathNode>,
    },

    /// An aligned multi-line expression (e.g., `\begin{align} ... \end{align}`).
    ///
    /// Each outer `Vec<MathNode>` is one row; the inner `Vec<MathNode>` is the
    /// sequence of nodes in that row (columns separated by `&`).
    Align(Vec<Vec<MathNode>>),

    /// A cases environment: `\begin{cases} result & condition \\ ... \end{cases}`.
    ///
    /// Each tuple is `(result-nodes, condition-nodes)`.  The parser stores the
    /// expression appearing BEFORE the `&` column separator as `result` and the
    /// expression AFTER `&` as `condition`.  This matches the LaTeX convention:
    ///
    /// ```text
    /// \begin{cases}
    ///   x   & \text{if } y > 0  \\
    ///   -x  & \text{if } y \leq 0
    /// \end{cases}
    /// ```
    ///
    /// Here `x` / `-x` are the *results* and `y > 0` / `y \leq 0` are the
    /// *conditions*. Renderers MUST emit result first and condition second to
    /// match the expected left-to-right column order in the typeset output.
    Cases(Vec<(Vec<MathNode>, Vec<MathNode>)>),

    /// A horizontal space (from `\,`, `\;`, `\quad`, `\qquad`, etc.).
    Space,
}

/// The result of parsing a single math expression.
///
/// Produced by [`crate::parser::parse`]; consumed by [`crate::omml`] and the
/// `MathML` path (future story).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MathAst {
    /// The rendering mode (inline vs. display).
    pub mode: MathMode,
    /// The top-level sequence of parsed nodes.
    pub nodes: Vec<MathNode>,
}

impl AccentKind {
    /// Return the Unicode combining character for use as the accent mark in
    /// `MathML` `<mover>`.
    ///
    /// These are proper Unicode combining diacritical marks (Category Mn).
    /// Using combining marks produces correct rendering in browsers and screen
    /// readers; spacing variants (like `^` U+005E) are typographically wrong.
    ///
    /// | AccentKind | Char     | Unicode name |
    /// |------------|----------|--------------|
    /// | Hat        | U+0302   | COMBINING CIRCUMFLEX ACCENT |
    /// | Bar        | U+0305   | COMBINING OVERLINE |
    /// | Tilde      | U+0303   | COMBINING TILDE |
    /// | Vec        | U+20D7   | COMBINING RIGHT ARROW ABOVE |
    /// | Dot        | U+0307   | COMBINING DOT ABOVE |
    /// | Ddot       | U+0308   | COMBINING DIAERESIS |
    #[must_use]
    pub fn mathml_combining_char(&self) -> &'static str {
        match self {
            AccentKind::Hat => "\u{0302}",   // ◌̂ COMBINING CIRCUMFLEX ACCENT
            AccentKind::Bar => "\u{0305}",   // ◌̅ COMBINING OVERLINE
            AccentKind::Tilde => "\u{0303}", // ◌̃ COMBINING TILDE
            AccentKind::Vec => "\u{20D7}",   // ◌⃗ COMBINING RIGHT ARROW ABOVE
            AccentKind::Dot => "\u{0307}",   // ◌̇ COMBINING DOT ABOVE
            AccentKind::Ddot => "\u{0308}",  // ◌̈ COMBINING DIAERESIS
        }
    }

    /// Return the Unicode combining character for use in OMML `<m:acc>`.
    ///
    /// OMML uses the same combining-mark approach as `MathML` for accent glyphs.
    /// This method is an alias of [`AccentKind::mathml_combining_char`]; the
    /// two-method design allows future divergence if OMML requires different
    /// mapping without breaking `MathML`.
    #[must_use]
    pub fn omml_combining_char(&self) -> &'static str {
        // Currently identical to MathML combining chars.
        self.mathml_combining_char()
    }
}

impl MathAst {
    /// Construct a new [`MathAst`].
    #[must_use]
    pub fn new(mode: MathMode, nodes: Vec<MathNode>) -> Self {
        MathAst { mode, nodes }
    }
}
