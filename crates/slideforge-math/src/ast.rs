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
    Text(Arc<str>),

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

    /// A cases environment: `\begin{cases} lhs & rhs \\ ... \end{cases}`.
    ///
    /// Each tuple is `(condition-nodes, result-nodes)`.
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

impl MathAst {
    /// Construct a new [`MathAst`].
    #[must_use]
    pub fn new(mode: MathMode, nodes: Vec<MathNode>) -> Self {
        MathAst { mode, nodes }
    }
}
