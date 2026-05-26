//! Math expression nodes.
//!
//! slideforge supports LaTeX math via `$...$` (inline) and `$$...$$` (display)
//! delimiters. Math content is stored as raw LaTeX source in [`MathNode`] and
//! rendered to MathML/OMML/HTML by the `MathRenderer` plugin at export time.

use std::sync::Arc;

use crate::span::SourceSpan;

/// A LaTeX math expression.
///
/// The renderer plugin resolves the LaTeX to format-specific output
/// (`MathML` for HTML, `OMML` for DOCX/PPTX, `KaTeX` HTML for web preview).
///
/// # Examples
///
/// Inline math: `$x^2 + y^2 = z^2$` produces a `MathNode` with `display = false`.
/// Display math: `$$\int_0^\infty e^{-x}\,dx = 1$$` produces `display = true`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MathNode {
    /// The raw LaTeX source. Not pre-validated at parse time; validation
    /// and rendering happen in the `MathRenderer` plugin.
    pub latex: Arc<str>,

    /// `true` for display (block) math; `false` for inline math.
    pub display: bool,

    /// Source location for error reporting.
    pub span: SourceSpan,
}

impl MathNode {
    /// Construct a new inline math node.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::{MathNode, SourceSpan};
    /// use std::sync::Arc;
    /// let node = MathNode::inline(Arc::from("x^2"), SourceSpan::default());
    /// assert!(!node.display);
    /// ```
    #[must_use]
    pub fn inline(latex: Arc<str>, span: SourceSpan) -> Self {
        MathNode {
            latex,
            display: false,
            span,
        }
    }

    /// Construct a new display (block) math node.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::{MathNode, SourceSpan};
    /// use std::sync::Arc;
    /// let node = MathNode::display(Arc::from(r"\int_0^\infty"), SourceSpan::default());
    /// assert!(node.display);
    /// ```
    #[must_use]
    pub fn display(latex: Arc<str>, span: SourceSpan) -> Self {
        MathNode {
            latex,
            display: true,
            span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ──────────────────────────────────────────────────────────────────────────
    // AC-006 — MathNode fields correct, implements Hash+Eq+Clone+Debug
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_006_math_node_fields() {
        let node = MathNode {
            latex: Arc::from("x^2"),
            display: false,
            span: SourceSpan::default(),
        };
        assert_eq!(node.latex.as_ref(), "x^2");
        assert!(!node.display);
    }

    #[test]
    fn test_bc_1_01_006_math_node_inline_constructor() {
        let node = MathNode::inline(Arc::from("y = mx + b"), SourceSpan::default());
        assert!(!node.display);
        assert_eq!(node.latex.as_ref(), "y = mx + b");
    }

    #[test]
    fn test_bc_1_01_006_math_node_display_constructor() {
        let node = MathNode::display(Arc::from(r"\sum_{i=0}^n i"), SourceSpan::default());
        assert!(node.display);
    }

    #[test]
    fn test_bc_1_01_006_math_node_clone() {
        let node = MathNode::inline(Arc::from("e^{i\u{03C0}}"), SourceSpan::default());
        let node2 = node.clone();
        assert_eq!(node, node2);
    }

    #[test]
    fn test_bc_1_01_006_math_node_eq() {
        let a = MathNode::inline(Arc::from("a"), SourceSpan::default());
        let b = MathNode::inline(Arc::from("a"), SourceSpan::default());
        assert_eq!(a, b);
    }

    #[test]
    fn test_bc_1_01_006_math_node_ne() {
        let a = MathNode::inline(Arc::from("a"), SourceSpan::default());
        let b = MathNode::display(Arc::from("a"), SourceSpan::default());
        // display flag differs
        assert_ne!(a, b);
    }

    #[test]
    fn test_bc_1_01_006_math_node_hash() {
        use std::collections::HashMap;
        let mut map: HashMap<MathNode, &str> = HashMap::new();
        let node = MathNode::inline(Arc::from("x"), SourceSpan::default());
        map.insert(node.clone(), "x inline");
        assert_eq!(map[&node], "x inline");
    }

    #[test]
    fn test_bc_1_01_006_math_node_debug() {
        let node = MathNode::inline(Arc::from("x"), SourceSpan::default());
        let s = format!("{node:?}");
        assert!(s.contains("MathNode"));
    }
}
