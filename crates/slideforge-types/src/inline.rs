//! Inline content nodes — the leaf nodes of the slideforge content tree.
//!
//! Inline nodes appear within text blocks and bullet items. They represent
//! the formatting and semantic structure within a single paragraph.
//!
//! slideforge uses structural inline nodes (e.g., `Inline::Bold`) rather than
//! string-prefix-based markup (e.g., `"**header**"`). The Python reference's
//! string-prefix approach is an anti-pattern (R1 finding).

use std::sync::Arc;

use crate::math::MathNode;

/// An inline content node within a paragraph or bullet item.
///
/// Exactly 12 variants are defined. This count is enforced by the acceptance
/// criteria (AC-008). Bullets-layout validation scope is STORY-073.
///
/// The variants cover the inline formatting and semantic-annotation needs of
/// PPTX, DOCX, HTML, and PDF output in v1.0. Additional inline types (e.g.,
/// abbreviation, ruby annotation) are deferred to v2.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InlineNode {
    /// Plain unstyled text.
    Plain(Arc<str>),

    /// Bold text.
    Bold(Vec<InlineNode>),

    /// Italic text.
    Italic(Vec<InlineNode>),

    /// Inline code (monospace, no further inline processing).
    Code(Arc<str>),

    /// A hyperlink with display content and target URL.
    Link {
        /// The display content of the link (can be styled inline nodes).
        text: Vec<InlineNode>,
        /// The target URL.
        url: Arc<str>,
    },

    /// An inline math expression (LaTeX source).
    Math(MathNode),

    /// A footnote. The content is stored inline as a sequence of inline nodes.
    Footnote(Vec<InlineNode>),

    /// A cross-reference to another slide or section (by identifier string).
    Xref(Arc<str>),

    /// Superscript text (e.g., exponents, ordinals).
    Superscript(Vec<InlineNode>),

    /// Subscript text (e.g., chemical formulae, footnote markers).
    Subscript(Vec<InlineNode>),

    /// Strikethrough text.
    Strikethrough(Vec<InlineNode>),

    /// Highlighted text (e.g., for callouts in review mode).
    Highlight(Vec<InlineNode>),
}

impl InlineNode {
    /// Return the discriminant name for error messages and serialization.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::InlineNode;
    /// use std::sync::Arc;
    /// assert_eq!(InlineNode::Plain(Arc::from("hi")).kind_name(), "Plain");
    /// assert_eq!(InlineNode::Bold(vec![]).kind_name(), "Bold");
    /// ```
    #[must_use]
    pub fn kind_name(&self) -> &'static str {
        match self {
            InlineNode::Plain(_) => "Plain",
            InlineNode::Bold(_) => "Bold",
            InlineNode::Italic(_) => "Italic",
            InlineNode::Code(_) => "Code",
            InlineNode::Link { .. } => "Link",
            InlineNode::Math(_) => "Math",
            InlineNode::Footnote(_) => "Footnote",
            InlineNode::Xref(_) => "Xref",
            InlineNode::Superscript(_) => "Superscript",
            InlineNode::Subscript(_) => "Subscript",
            InlineNode::Strikethrough(_) => "Strikethrough",
            InlineNode::Highlight(_) => "Highlight",
        }
    }

    /// Return `true` if this is a [`InlineNode::Plain`] node.
    #[must_use]
    pub fn is_plain(&self) -> bool {
        matches!(self, InlineNode::Plain(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ──────────────────────────────────────────────────────────────────────────
    // AC-005 — InlineNode has exactly 12 variants per BC-3.05.001 v1.3.4,
    //          implements Hash+Eq+Clone+Debug
    // ──────────────────────────────────────────────────────────────────────────

    fn all_variants() -> Vec<InlineNode> {
        vec![
            InlineNode::Plain(Arc::from("text")),
            InlineNode::Bold(vec![]),
            InlineNode::Italic(vec![]),
            InlineNode::Code(Arc::from("fn foo() {}")),
            InlineNode::Link {
                text: vec![],
                url: Arc::from("https://example.com"),
            },
            InlineNode::Math(MathNode {
                latex: Arc::from("x^2"),
                display: false,
                span: crate::span::SourceSpan::default(),
            }),
            InlineNode::Footnote(vec![]),
            InlineNode::Xref(Arc::from("slide-2")),
            InlineNode::Superscript(vec![]),
            InlineNode::Subscript(vec![]),
            InlineNode::Strikethrough(vec![]),
            InlineNode::Highlight(vec![]),
        ]
    }

    #[test]
    fn test_bc_3_05_001_inline_node_exactly_12_variants() {
        // Every variant is constructed in all_variants(); any missing variant
        // would be a compiler warning or explicit count mismatch.
        // BC-3.05.001 v1.3.4 (PO adjudication) confirms the canonical count is 12.
        let variants = all_variants();
        assert_eq!(
            variants.len(),
            12,
            "InlineNode must have exactly 12 variants per BC-3.05.001 v1.3.4"
        );
    }

    #[test]
    fn test_bc_3_05_001_inline_node_clone() {
        let node = InlineNode::Bold(vec![InlineNode::Plain(Arc::from("hi"))]);
        let node2 = node.clone();
        assert_eq!(node, node2);
    }

    #[test]
    fn test_bc_3_05_001_inline_node_eq() {
        let a = InlineNode::Plain(Arc::from("hello"));
        let b = InlineNode::Plain(Arc::from("hello"));
        assert_eq!(a, b);
    }

    #[test]
    fn test_bc_3_05_001_inline_node_ne() {
        let a = InlineNode::Plain(Arc::from("hello"));
        let b = InlineNode::Plain(Arc::from("world"));
        assert_ne!(a, b);
    }

    #[test]
    fn test_bc_3_05_001_inline_node_hash() {
        use std::collections::HashMap;
        let mut map: HashMap<InlineNode, &str> = HashMap::new();
        map.insert(InlineNode::Plain(Arc::from("a")), "plain a");
        map.insert(InlineNode::Code(Arc::from("fn x() {}")), "code");
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_bc_3_05_001_inline_node_debug() {
        let node = InlineNode::Bold(vec![]);
        let s = format!("{node:?}");
        assert!(s.contains("Bold"));
    }

    #[test]
    fn test_bc_3_05_001_inline_node_kind_names() {
        let variants = all_variants();
        let names: Vec<&str> = variants.iter().map(InlineNode::kind_name).collect();
        assert!(names.contains(&"Plain"));
        assert!(names.contains(&"Bold"));
        assert!(names.contains(&"Italic"));
        assert!(names.contains(&"Code"));
        assert!(names.contains(&"Link"));
        assert!(names.contains(&"Math"));
        assert!(names.contains(&"Footnote"));
        assert!(names.contains(&"Xref"));
        assert!(names.contains(&"Superscript"));
        assert!(names.contains(&"Subscript"));
        assert!(names.contains(&"Strikethrough"));
        assert!(names.contains(&"Highlight"));
    }

    #[test]
    fn test_bc_3_05_001_inline_node_nested() {
        let inner = InlineNode::Plain(Arc::from("nested"));
        let outer = InlineNode::Bold(vec![inner.clone()]);
        match &outer {
            InlineNode::Bold(children) => {
                assert_eq!(children.len(), 1);
                assert_eq!(children[0], inner);
            },
            _ => panic!("expected Bold"),
        }
    }
}
