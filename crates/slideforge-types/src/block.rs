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

    /// Return `true` if this block produces a PDF structure group (MCID / tag tree child).
    ///
    /// This is the **single authoritative predicate** for the structure-producing
    /// decision (OBS-P5-001 fix). Both the tag engine (`tag_content_block` in
    /// `slideforge-pdf`) and the draw loop (`draw_body_blocks_tagged` in
    /// `slideforge-pdf`) MUST call this method rather than independently
    /// encoding the same logic. A future `ContentBlock` variant that fails to
    /// update this match will produce a compiler error — enforcing lockstep.
    ///
    /// ## Decision table
    ///
    /// | Block kind                              | Returns  |
    /// |-----------------------------------------|----------|
    /// | `Text(_)`                               | `true`   |
    /// | `Math(_)`                               | `true`   |
    /// | `Table(_)`                              | `true`   |
    /// | `Bullets(items)` if non-empty           | `true`   |
    /// | `Bullets(items)` if empty               | `false`  |
    /// | `Image(alt: Provided(_))`               | `true`   |
    /// | `Image(alt: Decorative \| None)`        | `false`  |
    /// | `Chart(alt: Provided(_))`               | `true`   |
    /// | `Chart(alt: Decorative \| None)`        | `false`  |
    /// | `Diagram(alt: Provided(_))`             | `true`   |
    /// | `Diagram(alt: Decorative \| None)`      | `false`  |
    /// | `Shape(alt: Provided(_))`               | `true`   |
    /// | `Shape(alt: Decorative \| None)`        | `false`  |
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::{ContentBlock, MathNode, SourceSpan};
    /// use std::sync::Arc;
    ///
    /// let math = ContentBlock::Math(MathNode::display(Arc::from("x^2"), SourceSpan::default()));
    /// assert!(math.produces_structure_group());
    ///
    /// let empty_bullets = ContentBlock::Bullets(vec![]);
    /// assert!(!empty_bullets.produces_structure_group());
    /// ```
    #[must_use]
    pub fn produces_structure_group(&self) -> bool {
        use crate::specs::AltText;

        match self {
            // Always structure-producing: textual/tabular content always gets a tag.
            ContentBlock::Text(_) | ContentBlock::Math(_) | ContentBlock::Table(_) => true,

            // Structure-producing only when non-empty: an empty list has no semantic value.
            ContentBlock::Bullets(items) => !items.is_empty(),

            // Visual elements: structure-producing only when the author explicitly provides
            // meaningful alt text. Decorative elements and elements with no alt text are
            // marked as PDF Artifacts and do not participate in the structure tree.
            ContentBlock::Image(spec) => matches!(&spec.alt, Some(AltText::Provided(_))),
            ContentBlock::Chart(spec) => matches!(spec.alt.as_ref(), Some(AltText::Provided(_))),
            ContentBlock::Diagram(spec) => matches!(spec.alt.as_ref(), Some(AltText::Provided(_))),
            ContentBlock::Shape(spec) => matches!(&spec.alt, Some(AltText::Provided(_))),
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
                shape_type: crate::shape_types::ShapeType::Rect,
                position: crate::specs::ShapePosition {
                    x: crate::specs::ShapeUnit::Inches(500),
                    y: crate::specs::ShapeUnit::Inches(1000),
                    width: crate::specs::ShapeUnit::Inches(2000),
                    height: crate::specs::ShapeUnit::Inches(1000),
                },
                fill: crate::shape_types::FillSpec::None,
                text: None,
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

    // ──────────────────────────────────────────────────────────────────────────
    // OBS-P5-001 — produces_structure_group() single-source-of-truth predicate
    // ──────────────────────────────────────────────────────────────────────────
    //
    // This test is the LOAD-BEARING assertion that both call sites in
    // slideforge-pdf (tag_content_block and draw_body_blocks_tagged) depend on.
    // It exhaustively covers every ContentBlock variant so that adding a new
    // variant forces an update here AND a compiler error in the match —
    // preventing silent desync.

    /// OBS-P5-001 — Text block is always structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_text_always_true() {
        use crate::inline::InlineNode;
        let block = ContentBlock::Text(TextBlock {
            inlines: vec![InlineNode::Plain(Arc::from("hello"))],
            span: SourceSpan::default(),
        });
        assert!(
            block.produces_structure_group(),
            "Text block must always produce a structure group"
        );
    }

    /// OBS-P5-001 — Math block is always structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_math_always_true() {
        use crate::math::MathNode;
        let block = ContentBlock::Math(MathNode::display(Arc::from("x^2"), SourceSpan::default()));
        assert!(
            block.produces_structure_group(),
            "Math block must always produce a structure group"
        );
    }

    /// OBS-P5-001 — Table block is always structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_table_always_true() {
        use crate::specs::TableSpec;
        let block = ContentBlock::Table(TableSpec {
            headers: vec![Arc::from("Col A")],
            rows: vec![vec![Arc::from("val")]],
            alt: None,
            span: SourceSpan::default(),
        });
        assert!(
            block.produces_structure_group(),
            "Table block must always produce a structure group"
        );
    }

    /// OBS-P5-001 — Non-empty Bullets is structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_nonempty_bullets_true() {
        use crate::inline::InlineNode;
        let block = ContentBlock::Bullets(vec![BulletItem {
            inlines: vec![InlineNode::Plain(Arc::from("item"))],
            children: vec![],
            span: SourceSpan::default(),
        }]);
        assert!(
            block.produces_structure_group(),
            "Non-empty Bullets must produce a structure group"
        );
    }

    /// OBS-P5-001 — Empty Bullets is NOT structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_empty_bullets_false() {
        let block = ContentBlock::Bullets(vec![]);
        assert!(
            !block.produces_structure_group(),
            "Empty Bullets must NOT produce a structure group"
        );
    }

    /// OBS-P5-001 — Image with Provided alt is structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_image_provided_alt_true() {
        use crate::specs::{AltText, ImageSpec};
        let block = ContentBlock::Image(ImageSpec {
            path: Arc::from("photo.png"),
            alt: Some(AltText::Provided(Arc::from("A photo of the campus"))),
            decorative: false,
            span: SourceSpan::default(),
        });
        assert!(
            block.produces_structure_group(),
            "Image with Provided alt must produce a structure group"
        );
    }

    /// OBS-P5-001 — Image with Decorative alt is NOT structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_image_decorative_false() {
        use crate::specs::{AltText, ImageSpec};
        let block = ContentBlock::Image(ImageSpec {
            path: Arc::from("decoration.png"),
            alt: Some(AltText::Decorative),
            decorative: true,
            span: SourceSpan::default(),
        });
        assert!(
            !block.produces_structure_group(),
            "Image with Decorative alt must NOT produce a structure group"
        );
    }

    /// OBS-P5-001 — Image with no alt (None) is NOT structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_image_no_alt_false() {
        use crate::specs::ImageSpec;
        let block = ContentBlock::Image(ImageSpec {
            path: Arc::from("photo.png"),
            alt: None,
            decorative: false,
            span: SourceSpan::default(),
        });
        assert!(
            !block.produces_structure_group(),
            "Image with no alt (None) must NOT produce a structure group"
        );
    }

    /// OBS-P5-001 — Chart with Provided alt is structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_chart_provided_alt_true() {
        use crate::specs::{AltText, ChartSpec};
        let block = ContentBlock::Chart(ChartSpec {
            chart_type: Arc::from("bar"),
            alt: Some(AltText::Provided(Arc::from("Revenue by quarter"))),
            decorative: false,
            span: SourceSpan::default(),
        });
        assert!(
            block.produces_structure_group(),
            "Chart with Provided alt must produce a structure group"
        );
    }

    /// OBS-P5-001 — Chart with Decorative alt is NOT structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_chart_decorative_false() {
        use crate::specs::{AltText, ChartSpec};
        let block = ContentBlock::Chart(ChartSpec {
            chart_type: Arc::from("line"),
            alt: Some(AltText::Decorative),
            decorative: true,
            span: SourceSpan::default(),
        });
        assert!(
            !block.produces_structure_group(),
            "Chart with Decorative alt must NOT produce a structure group"
        );
    }

    /// OBS-P5-001 — Chart with no alt (None) is NOT structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_chart_no_alt_false() {
        use crate::specs::ChartSpec;
        let block = ContentBlock::Chart(ChartSpec {
            chart_type: Arc::from("pie"),
            alt: None,
            decorative: false,
            span: SourceSpan::default(),
        });
        assert!(
            !block.produces_structure_group(),
            "Chart with no alt (None) must NOT produce a structure group"
        );
    }

    /// OBS-P5-001 — Diagram with Provided alt is structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_diagram_provided_alt_true() {
        use crate::specs::{AltText, DiagramSpec};
        let block = ContentBlock::Diagram(DiagramSpec {
            source: Arc::from("graph TD; A-->B"),
            alt: Some(AltText::Provided(Arc::from("Dependency graph"))),
            decorative: false,
            span: SourceSpan::default(),
        });
        assert!(
            block.produces_structure_group(),
            "Diagram with Provided alt must produce a structure group"
        );
    }

    /// OBS-P5-001 — Diagram with Decorative alt is NOT structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_diagram_decorative_false() {
        use crate::specs::{AltText, DiagramSpec};
        let block = ContentBlock::Diagram(DiagramSpec {
            source: Arc::from("graph TD; A-->B"),
            alt: Some(AltText::Decorative),
            decorative: true,
            span: SourceSpan::default(),
        });
        assert!(
            !block.produces_structure_group(),
            "Diagram with Decorative alt must NOT produce a structure group"
        );
    }

    /// OBS-P5-001 — Diagram with no alt (None) is NOT structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_diagram_no_alt_false() {
        use crate::specs::DiagramSpec;
        let block = ContentBlock::Diagram(DiagramSpec {
            source: Arc::from("graph TD; A-->B"),
            alt: None,
            decorative: false,
            span: SourceSpan::default(),
        });
        assert!(
            !block.produces_structure_group(),
            "Diagram with no alt (None) must NOT produce a structure group"
        );
    }

    /// OBS-P5-001 — Shape with Provided alt is structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_shape_provided_alt_true() {
        use crate::shape_types::{FillSpec, ShapeType};
        use crate::specs::{AltText, ShapePosition, ShapeSpec, ShapeUnit};
        let block = ContentBlock::Shape(ShapeSpec {
            shape_type: ShapeType::Rect,
            position: ShapePosition {
                x: ShapeUnit::Inches(500),
                y: ShapeUnit::Inches(500),
                width: ShapeUnit::Inches(2000),
                height: ShapeUnit::Inches(1000),
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from(
                "A blue rectangle highlighting the key metric",
            ))),
            decorative: false,
            span: SourceSpan::default(),
        });
        assert!(
            block.produces_structure_group(),
            "Shape with Provided alt must produce a structure group"
        );
    }

    /// OBS-P5-001 — Shape with Decorative alt is NOT structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_shape_decorative_false() {
        use crate::shape_types::{FillSpec, ShapeType};
        use crate::specs::{AltText, ShapePosition, ShapeSpec, ShapeUnit};
        let block = ContentBlock::Shape(ShapeSpec {
            shape_type: ShapeType::Ellipse,
            position: ShapePosition {
                x: ShapeUnit::Inches(500),
                y: ShapeUnit::Inches(500),
                width: ShapeUnit::Inches(1000),
                height: ShapeUnit::Inches(1000),
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Decorative),
            decorative: true,
            span: SourceSpan::default(),
        });
        assert!(
            !block.produces_structure_group(),
            "Shape with Decorative alt must NOT produce a structure group"
        );
    }

    /// OBS-P5-001 — Shape with no alt (None) is NOT structure-producing.
    #[test]
    fn test_obs_p5_001_produces_structure_group_shape_no_alt_false() {
        use crate::shape_types::{FillSpec, ShapeType};
        use crate::specs::{ShapePosition, ShapeSpec, ShapeUnit};
        let block = ContentBlock::Shape(ShapeSpec {
            shape_type: ShapeType::Rect,
            position: ShapePosition {
                x: ShapeUnit::Inches(0),
                y: ShapeUnit::Inches(0),
                width: ShapeUnit::Inches(1000),
                height: ShapeUnit::Inches(1000),
            },
            fill: FillSpec::None,
            text: None,
            alt: None,
            decorative: false,
            span: SourceSpan::default(),
        });
        assert!(
            !block.produces_structure_group(),
            "Shape with no alt (None) must NOT produce a structure group"
        );
    }
}
