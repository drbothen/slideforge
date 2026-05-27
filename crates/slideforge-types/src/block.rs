//! Content block types — the structural nodes of a slide's content tree.
//!
//! A slide's body is a sequence of [`ContentBlock`] nodes. Each block
//! represents a distinct structural element (paragraph, bullet list, chart,
//! diagram, shape, math, image, or table).

use std::sync::Arc;

use crate::inline::InlineNode;
use crate::math::MathNode;
use crate::span::SourceSpan;
use crate::specs::{ChartSpec, DiagramSpec, ImageSpec, ShapeSpec, TableSpec};

/// A text paragraph.
///
/// A `TextBlock` is a sequence of inline nodes that form a single paragraph.
/// Paragraphs do not contain sub-paragraphs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextBlock {
    /// The inline nodes forming this paragraph.
    pub inlines: Vec<InlineNode>,
    /// Source location.
    pub span: SourceSpan,
}

/// A single bullet item within a bullet list.
///
/// Bullet items may nest (a bullet item can contain sub-bullets via the
/// `children` field).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BulletItem {
    /// The inline content of this bullet.
    pub inlines: Vec<InlineNode>,
    /// Optional sub-bullets (nested list).
    pub children: Vec<BulletItem>,
    /// Source location.
    pub span: SourceSpan,
}

/// A named, typed block within a slide body.
///
/// The `Block` wrapper pairs a block with its source location and an optional
/// label. The label is used for cross-references and accessibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    /// The content of this block.
    pub content: ContentBlock,
    /// Optional label for cross-referencing (`@label "my-block"`).
    pub label: Option<Arc<str>>,
    /// Source location.
    pub span: SourceSpan,
}

/// The discriminated union of all block content types.
///
/// Exactly 8 variants are defined (AC-005). Additional block types (e.g.,
/// `Code`, `Quote`) are deferred to v1.x stories.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContentBlock {
    /// A text paragraph.
    Text(TextBlock),

    /// A bullet/numbered list.
    Bullets(Vec<BulletItem>),

    /// A chart defined by a [`ChartSpec`].
    Chart(ChartSpec),

    /// A diagram defined by a [`DiagramSpec`] (e.g., Mermaid).
    Diagram(DiagramSpec),

    /// A shape defined by a [`ShapeSpec`].
    Shape(ShapeSpec),

    /// A display-mode math block.
    Math(MathNode),

    /// An embedded image defined by an [`ImageSpec`].
    Image(ImageSpec),

    /// A data table defined by a [`TableSpec`].
    Table(TableSpec),
}

impl ContentBlock {
    /// Return the discriminant name for error messages.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::{ContentBlock, MathNode, SourceSpan};
    /// use std::sync::Arc;
    /// let math = ContentBlock::Math(MathNode::display(Arc::from("x"), SourceSpan::default()));
    /// assert_eq!(math.kind_name(), "Math");
    /// ```
    #[must_use]
    pub fn kind_name(&self) -> &'static str {
        match self {
            ContentBlock::Text(_) => "Text",
            ContentBlock::Bullets(_) => "Bullets",
            ContentBlock::Chart(_) => "Chart",
            ContentBlock::Diagram(_) => "Diagram",
            ContentBlock::Shape(_) => "Shape",
            ContentBlock::Math(_) => "Math",
            ContentBlock::Image(_) => "Image",
            ContentBlock::Table(_) => "Table",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specs::{ChartSpec, DiagramSpec, ImageSpec, ShapeSpec, TableSpec};
    use std::sync::Arc;

    // ──────────────────────────────────────────────────────────────────────────
    // AC-005 — ContentBlock has 8 variants, implements Hash+Eq+Clone+Debug
    // ──────────────────────────────────────────────────────────────────────────

    fn all_content_block_variants() -> Vec<ContentBlock> {
        use crate::specs::AltText;
        vec![
            ContentBlock::Text(TextBlock {
                inlines: vec![],
                span: SourceSpan::default(),
            }),
            ContentBlock::Bullets(vec![]),
            ContentBlock::Chart(ChartSpec {
                chart_type: Arc::from("bar"),
                alt: Some(AltText::Provided(Arc::from("chart"))),
                decorative: false,
                span: SourceSpan::default(),
            }),
            ContentBlock::Diagram(DiagramSpec {
                source: Arc::from("graph TD; A-->B"),
                alt: Some(AltText::Provided(Arc::from("diagram"))),
                decorative: false,
                span: SourceSpan::default(),
            }),
            ContentBlock::Shape(ShapeSpec {
                shape_type: Arc::from("rect"),
                alt: Some(AltText::Provided(Arc::from("shape"))),
                decorative: false,
                span: SourceSpan::default(),
            }),
            ContentBlock::Math(MathNode::display(Arc::from("x^2"), SourceSpan::default())),
            ContentBlock::Image(ImageSpec {
                path: Arc::from("image.png"),
                alt: Some(AltText::Provided(Arc::from("an image"))),
                decorative: false,
                span: SourceSpan::default(),
            }),
            ContentBlock::Table(TableSpec {
                headers: vec![],
                rows: vec![],
                alt: Some(AltText::Provided(Arc::from("table"))),
                span: SourceSpan::default(),
            }),
        ]
    }

    #[test]
    fn test_bc_1_01_005_content_block_exactly_8_variants() {
        assert_eq!(all_content_block_variants().len(), 8);
    }

    #[test]
    fn test_bc_1_01_005_content_block_kind_names() {
        let variants = all_content_block_variants();
        let names: Vec<&str> = variants.iter().map(ContentBlock::kind_name).collect();
        assert!(names.contains(&"Text"));
        assert!(names.contains(&"Bullets"));
        assert!(names.contains(&"Chart"));
        assert!(names.contains(&"Diagram"));
        assert!(names.contains(&"Shape"));
        assert!(names.contains(&"Math"));
        assert!(names.contains(&"Image"));
        assert!(names.contains(&"Table"));
    }

    #[test]
    fn test_bc_1_01_005_content_block_clone() {
        let block = ContentBlock::Bullets(vec![]);
        let block2 = block.clone();
        assert_eq!(block, block2);
    }

    #[test]
    fn test_bc_1_01_005_content_block_hash() {
        use std::collections::HashMap;
        let mut map: HashMap<ContentBlock, &str> = HashMap::new();
        let block = ContentBlock::Bullets(vec![]);
        map.insert(block.clone(), "bullets");
        assert_eq!(map[&block], "bullets");
    }

    #[test]
    fn test_bc_1_01_005_content_block_debug() {
        let block = ContentBlock::Bullets(vec![]);
        let s = format!("{block:?}");
        assert!(s.contains("Bullets"));
    }

    #[test]
    fn test_bc_1_01_005_text_block_fields() {
        let tb = TextBlock {
            inlines: vec![InlineNode::Plain(Arc::from("hello"))],
            span: SourceSpan::default(),
        };
        assert_eq!(tb.inlines.len(), 1);
    }

    #[test]
    fn test_bc_1_01_005_bullet_item_nested() {
        let child = BulletItem {
            inlines: vec![],
            children: vec![],
            span: SourceSpan::default(),
        };
        let parent = BulletItem {
            inlines: vec![],
            children: vec![child],
            span: SourceSpan::default(),
        };
        assert_eq!(parent.children.len(), 1);
    }

    #[test]
    fn test_bc_1_01_005_block_wrapper_fields() {
        let block = Block {
            content: ContentBlock::Bullets(vec![]),
            label: Some(Arc::from("my-list")),
            span: SourceSpan::default(),
        };
        assert_eq!(block.label.as_deref(), Some("my-list"));
    }
}
