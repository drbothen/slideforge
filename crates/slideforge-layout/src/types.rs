//! Layout IR types — `LaidOutDeck`, `LaidOutSlide`, `Frame`, `BoundingBox`,
//! `TextFlow`, `PageSize`.
//!
//! These types carry the *geometric* representation of a presentation after
//! the layout engine has computed EMU coordinates for every content frame.
//! They are distinct from the semantic IR types in `slideforge-types::deck`:
//! the layout IR adds positioning, page size, and text-flow analysis.
//!
//! ## Design Invariants
//!
//! - All coordinate and size fields use [`Emu`] (English Metric Units, `i64`).
//!   No `f32`, `f64`, or bare `i64` for spatial values.
//! - All types implement `Hash + Eq + Clone + Debug` for comemo compatibility
//!   (AC-010) and `proptest` `Arbitrary` derivability (VP-011).
//! - `Arc<str>` for string fields (avoids allocation on clone).

use std::sync::Arc;

use slideforge_types::ContentBlock;
pub use slideforge_types::Emu;

/// The default canvas width for the layout engine (10 inches = 9,144,000 EMU).
///
/// This is the canvas width used by the layout engine when the active `Brand`
/// does not specify custom dimensions (AC-004). It matches
/// `slideforge_types::CANVAS_WIDTH` (9,144,000 EMU).
///
/// ## Relationship to PPTX native dimensions
///
/// The PPTX native slide width (`slideforge_types::SLIDE_WIDTH`) is
/// 12,192,000 EMU (13.33 inches, the `PowerPoint` widescreen default). The
/// layout engine operates at 9,144,000 EMU (10 inches) as its design canvas
/// and the PPTX exporter scales coordinates to the PPTX native dimensions at
/// export time. This distinction exists because the DSL design canvas matches
/// the print-inch coordinate space (1 inch = 914,400 EMU), whereas PPTX
/// encodes slides as 13.33 × 7.5 inches at the same 914,400 EMU/inch scale.
pub const DEFAULT_PAGE_WIDTH: Emu = Emu(9_144_000);

/// The default canvas height for the layout engine (5.625 inches = 5,143,500 EMU).
///
/// This is the canvas height used by the layout engine when the active `Brand`
/// does not specify custom dimensions (AC-004).
///
/// ## Relationship to PPTX native dimensions
///
/// The PPTX native slide height (`slideforge_types::SLIDE_HEIGHT`) is
/// 6,858,000 EMU (7.5 inches). The layout engine uses 5,143,500 EMU
/// (5.625 inches) as its 16:9 design canvas height — a 3/4 scale of the PPTX
/// native height. The PPTX exporter rescales all vertical coordinates by the
/// ratio `SLIDE_HEIGHT / DEFAULT_PAGE_HEIGHT` (≈ 1.333) at export time.
pub const DEFAULT_PAGE_HEIGHT: Emu = Emu(5_143_500);

/// The standard 4:3 page height (7.5 inches = 6,858,000 EMU).
///
/// Width remains `DEFAULT_PAGE_WIDTH` (9,144,000 EMU). Use this constant when
/// the brand selects the standard 4:3 aspect ratio.
pub const STANDARD_4X3_PAGE_HEIGHT: Emu = Emu(6_858_000);

/// A set of [`RegisterTag`] values for a laid-out slide.
///
/// `RegisterSet` is a named alias for `Vec<RegisterTag>` used in
/// [`LaidOutSlide::register_tags`]. Using a named alias makes the field's
/// semantic intent explicit at call sites and allows future change to a
/// `SmallVec` or a bitset without breaking the public API.
pub type RegisterSet = Vec<RegisterTag>;

/// A writing register tag attached to a laid-out slide.
///
/// `RegisterTag` identifies the output register(s) a slide belongs to,
/// derived from [`slideforge_types::Slide::register`] during layout.
/// Exporters use this to decide whether to include the slide at a given
/// output fidelity level.
///
/// Ordered `Notes < Report < Detail` (increasing verbosity), matching
/// [`slideforge_types::Register`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RegisterTag {
    /// Presenter notes register — visible in speaker view only.
    Notes,
    /// Reader register — included in document exports.
    Report,
    /// Detail register — document-only, excluded from slide view.
    Detail,
}

/// The geometric, post-layout intermediate representation of a presentation.
///
/// `LaidOutDeck` is produced by [`crate::layout::run`] from a
/// [`slideforge_types::Deck`]. It carries precise EMU coordinates for every
/// element on every slide, together with the resolved page dimensions.
///
/// ## Slide count invariant (AC-002 / BC-3.06.001)
///
/// `slides.len()` MUST equal `Deck.slides.len()`. The layout engine returns
/// [`crate::error::LayoutError::SlideCountMismatch`] rather than produce a
/// deck with a different count.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LaidOutDeck {
    /// The page dimensions for all slides in this deck.
    pub page_size: PageSize,
    /// The laid-out slides, one per entry in [`slideforge_types::Deck::slides`].
    pub slides: Vec<LaidOutSlide>,
}

/// Slide page dimensions in EMU.
///
/// The layout engine derives `PageSize` from the active `Brand`'s canvas
/// configuration. When the brand specifies no override, the default is
/// widescreen 16:9: `{ width: DEFAULT_PAGE_WIDTH, height: DEFAULT_PAGE_HEIGHT }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageSize {
    /// Slide width in English Metric Units.
    pub width: Emu,
    /// Slide height in English Metric Units.
    pub height: Emu,
}

impl Default for PageSize {
    /// Returns the standard widescreen 16:9 page size (10 × 5.625 inches).
    fn default() -> Self {
        Self {
            width: DEFAULT_PAGE_WIDTH,
            height: DEFAULT_PAGE_HEIGHT,
        }
    }
}

/// A single slide in the geometric, post-layout IR.
///
/// A `LaidOutSlide` corresponds to exactly one [`slideforge_types::Slide`] in
/// the semantic IR. The `source_index` field provides the back-reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LaidOutSlide {
    /// Zero-based index of the corresponding slide in [`slideforge_types::Deck::slides`].
    pub source_index: usize,

    /// The DSL keyword identifying the slide's visual type (e.g., `"title"`,
    /// `"content"`, `"stat_callout"`).
    ///
    /// This is a copy of [`slideforge_types::Slide::slide_type`].
    pub slide_type_keyword: Arc<str>,

    /// The positioned content regions for this slide.
    ///
    /// Frames are ordered semantically (title before body, body before notes).
    /// Each frame carries a [`BoundingBox`] and a [`FrameContent`] variant.
    pub frames: Vec<Frame>,

    /// Resolved speaker notes content for this slide, if any.
    ///
    /// `None` when the slide has no presenter notes register content.
    pub speaker_notes: Option<Arc<str>>,

    /// The register tags for this slide, derived from the semantic slide's
    /// [`slideforge_types::Slide::register`] field during layout.
    ///
    /// An empty [`RegisterSet`] means the slide is unregistered (appears in all outputs).
    pub register_tags: RegisterSet,
}

/// A positioned content region within a laid-out slide.
///
/// A `Frame` pairs a bounding box (position + size in EMU) with the semantic
/// content assigned to that region, and an optional [`TextFlow`] analysis for
/// text-bearing frames.
///
/// The layout engine produces one `Frame` per content region defined by the
/// slide type's region map. `text_flow` is `Some` for [`FrameContent::Title`],
/// [`FrameContent::Subtitle`], and [`FrameContent::Body`] frames where text
/// content is known at layout time; `None` for image/chart/diagram/shape frames
/// and for frames where content was not populated (e.g., `FrameContent::Empty`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Frame {
    /// The position and dimensions of this content region.
    pub bbox: BoundingBox,

    /// The content assigned to this frame.
    pub content: FrameContent,

    /// Heuristic text-flow analysis for this frame, if it bears text content.
    ///
    /// `None` for non-text frames (images, charts, diagrams, shapes) and for
    /// frames where no text content was populated at layout time.
    pub text_flow: Option<TextFlow>,
}

/// Position and dimensions of a content region, in English Metric Units.
///
/// All fields MUST satisfy the following invariants (AC-014 / BC-3.06.003):
/// - `x >= 0`
/// - `y >= 0`
/// - `width > 0`
/// - `height > 0`
/// - `x + width <= page_width`
/// - `y + height <= page_height`
///
/// Violations are detected by the post-layout integrity check inside
/// `layout::run` and returned as
/// [`crate::error::LayoutError::InvalidBoundingBox`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoundingBox {
    /// Horizontal position from slide left edge.
    pub x: Emu,
    /// Vertical position from slide top edge.
    pub y: Emu,
    /// Width of the content region.
    pub width: Emu,
    /// Height of the content region.
    pub height: Emu,
}

impl BoundingBox {
    /// Returns `true` if this bounding box satisfies all invariants for the
    /// given page dimensions.
    ///
    /// Checks: `x >= 0`, `y >= 0`, `width > 0`, `height > 0`,
    /// `x + width <= page_width`, `y + height <= page_height`.
    ///
    /// Uses saturating arithmetic to avoid overflow on extreme EMU values.
    #[must_use]
    pub fn is_valid(&self, page_width: Emu, page_height: Emu) -> bool {
        self.x >= Emu(0)
            && self.y >= Emu(0)
            && self.width > Emu(0)
            && self.height > Emu(0)
            && Emu(self.x.0.saturating_add(self.width.0)) <= page_width
            && Emu(self.y.0.saturating_add(self.height.0)) <= page_height
    }
}

/// The semantic content of a positioned layout frame.
///
/// Each variant carries the data that the exporter will render into the frame's
/// bounding box. The variants cover the full range of content types defined by
/// the 31 built-in slide types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FrameContent {
    /// A primary slide title.
    Title(Arc<str>),
    /// A subtitle or secondary heading (e.g., on `TitleSlide`).
    Subtitle(Arc<str>),
    /// Structured body content (bullet list, numbered list, etc.).
    Body(Vec<ContentBlock>),
    /// An image placeholder with required alt text.
    Image {
        /// Required accessibility alt text for this image.
        alt: Arc<str>,
    },
    /// A chart rendered from a chart spec.
    Chart,
    /// A diagram rendered from a diagram source (e.g., Mermaid).
    Diagram,
    /// A shape from the shape DSL.
    Shape,
    /// An empty placeholder (present in the layout but no content assigned).
    Empty,
}

/// Text-flow analysis result for a text frame.
///
/// `TextFlow` is a heuristic model of how text will flow within a frame's
/// bounding box. It is not a pixel-perfect reflow — it uses a fixed estimated
/// character width (76,200 EMU/char at 12pt) to detect overflow conditions for
/// the canvas overflow warning (BC-3.03.001).
///
/// ## Overflow model
///
/// The overflow detection is intentionally heuristic:
/// - Assumes monospace-like character widths (76,200 EMU = 1 inch / 12 chars)
/// - Line breaks at bounding box width
/// - Does not account for variable-width fonts, kerning, or ligatures
/// - Is acceptable per BC-3.03.001 which requires an "EMU estimate", not exact
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextFlow {
    /// The bounding box within which the text must fit.
    pub bounding_box: BoundingBox,

    /// Whether the text fits, is truncated, or overflows the frame.
    pub overflow: TextOverflow,

    /// Estimated number of lines the text requires.
    ///
    /// Zero when `text` is empty (EC-004).
    pub line_count: u32,

    /// Estimated width per character in EMU.
    ///
    /// Default: `Emu(76_200)` (1 inch / 12 chars at 12pt).
    pub estimated_char_width_emu: Emu,
}

/// Whether text fits within its frame, is truncated, or overflows.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextOverflow {
    /// The text fits within the bounding box without truncation or overflow.
    Fit,
    /// The text is longer than the bounding box allows; excess text is hidden.
    ///
    /// Reserved for STORY-027 (content-resolution wrapping mode). When a
    /// slide is configured with `overflow: truncate`, the layout engine sets
    /// this variant instead of [`TextOverflow::Overflow`]. The validator
    /// suppresses the canvas-overflow warning for `Truncate` frames because
    /// the author has explicitly opted in to clipping.
    ///
    /// Not produced by the current layout engine — all overflow currently
    /// results in `TextOverflow::Overflow`. This variant exists in the type
    /// to preserve exhaustive match coverage when STORY-027 activates it.
    Truncate,
    /// The text exceeds the bounding box height by `excess_emu`.
    ///
    /// `excess_emu` is always non-negative. It is used by the canvas overflow
    /// validator (BC-3.03.001) to compute an EMU estimate for warning messages.
    Overflow {
        /// How many EMU the text extends beyond the frame's bottom edge.
        excess_emu: Emu,
    },
}

#[cfg(test)]
#[allow(clippy::missing_docs_in_private_items)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // BC-3.06.NNN — layout IR type unit tests
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-3.06.003 — `BoundingBox`: valid coordinates.
    #[test]
    fn test_bc_3_06_003_bounding_box_emu_values() {
        let bbox = BoundingBox {
            x: Emu(457_200),
            y: Emu(1_600_200),
            width: Emu(8_229_600),
            height: Emu(1_143_000),
        };
        assert_eq!(bbox.x, Emu(457_200));
        assert_eq!(bbox.y, Emu(1_600_200));
        assert_eq!(bbox.width, Emu(8_229_600));
        assert_eq!(bbox.height, Emu(1_143_000));
    }

    /// BC-3.06.003 — `BoundingBox::is_valid` returns true for valid box.
    #[test]
    fn test_bc_3_06_003_bounding_box_is_valid_passes() {
        let bbox = BoundingBox {
            x: Emu(0),
            y: Emu(0),
            width: Emu(9_144_000),
            height: Emu(5_143_500),
        };
        assert!(bbox.is_valid(DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT));
    }

    /// BC-3.06.003 — `BoundingBox::is_valid` returns false for negative x.
    #[test]
    fn test_bc_3_06_003_bounding_box_invalid_negative_x() {
        let bbox = BoundingBox {
            x: Emu(-1),
            y: Emu(0),
            width: Emu(100_000),
            height: Emu(100_000),
        };
        assert!(!bbox.is_valid(DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT));
    }

    /// BC-3.06.003 — `BoundingBox::is_valid` returns false for zero width.
    #[test]
    fn test_bc_3_06_003_bounding_box_invalid_zero_width() {
        let bbox = BoundingBox {
            x: Emu(0),
            y: Emu(0),
            width: Emu(0),
            height: Emu(100_000),
        };
        assert!(!bbox.is_valid(DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT));
    }

    /// BC-3.06.003 — `BoundingBox::is_valid` returns false when box exceeds page.
    #[test]
    fn test_bc_3_06_003_bounding_box_invalid_exceeds_page() {
        let bbox = BoundingBox {
            x: Emu(9_000_000),
            y: Emu(0),
            width: Emu(200_000), // x + width = 9_200_000 > DEFAULT_PAGE_WIDTH
            height: Emu(100_000),
        };
        assert!(!bbox.is_valid(DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT));
    }

    /// AC-004 — `PageSize::default()` returns the 16:9 widescreen dimensions.
    #[test]
    fn test_bc_3_06_001_page_size_default_widescreen() {
        let ps = PageSize::default();
        assert_eq!(
            ps.width, DEFAULT_PAGE_WIDTH,
            "default page width must be DEFAULT_PAGE_WIDTH (9_144_000 EMU)"
        );
        assert_eq!(
            ps.height, DEFAULT_PAGE_HEIGHT,
            "default page height must be DEFAULT_PAGE_HEIGHT (5_143_500 EMU)"
        );
    }

    /// AC-004 — Standard 4:3 height constant is correct.
    #[test]
    fn test_bc_3_06_001_page_size_standard_4x3() {
        // 4:3 is same width, taller height
        let ps_4x3 = PageSize {
            width: DEFAULT_PAGE_WIDTH,
            height: STANDARD_4X3_PAGE_HEIGHT,
        };
        assert_eq!(ps_4x3.height, Emu(6_858_000));
    }

    /// AC-010 — `LaidOutDeck` implements `Clone + PartialEq + Eq + Hash`.
    #[test]
    fn test_bc_3_06_001_laid_out_deck_implements_hash_eq_clone() {
        use std::collections::HashSet;

        let deck = LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![],
        };
        let deck2 = deck.clone();
        assert_eq!(deck, deck2);

        let mut set = HashSet::new();
        set.insert(deck);
        assert_eq!(set.len(), 1);
    }

    /// AC-010 — `LaidOutSlide` implements `Clone + PartialEq + Eq + Hash`.
    #[test]
    fn test_bc_3_06_001_laid_out_slide_implements_hash_eq_clone() {
        use std::collections::HashSet;
        use std::sync::Arc;

        let slide = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("title"),
            frames: vec![],
            speaker_notes: None,
            register_tags: vec![],
        };
        let slide2 = slide.clone();
        assert_eq!(slide, slide2);

        let mut set = HashSet::new();
        set.insert(slide);
        assert_eq!(set.len(), 1);
    }

    /// AC-010 — `Frame` implements `Clone + PartialEq + Eq + Hash`.
    #[test]
    fn test_bc_3_06_001_frame_implements_hash_eq_clone() {
        use std::collections::HashSet;

        let frame = Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(1_000_000),
                height: Emu(500_000),
            },
            content: FrameContent::Empty,
            text_flow: None,
        };
        let frame2 = frame.clone();
        assert_eq!(frame, frame2);

        let mut set = HashSet::new();
        set.insert(frame);
        assert_eq!(set.len(), 1);
    }

    /// AC-010 — `TextFlow` implements `Clone + PartialEq + Eq + Hash`.
    #[test]
    fn test_bc_3_06_001_text_flow_implements_hash_eq_clone() {
        use std::collections::HashSet;

        let tf = TextFlow {
            bounding_box: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(1_000_000),
                height: Emu(500_000),
            },
            overflow: TextOverflow::Fit,
            line_count: 0,
            estimated_char_width_emu: Emu(76_200),
        };
        let tf2 = tf.clone();
        assert_eq!(tf, tf2);

        let mut set = HashSet::new();
        set.insert(tf);
        assert_eq!(set.len(), 1);
    }

    /// AC-003 / AC-007 — `TextOverflow` variants cover `Fit`, `Truncate`, `Overflow`.
    #[test]
    fn test_bc_3_06_001_text_overflow_variants_exist() {
        let fit = TextOverflow::Fit;
        let trunc = TextOverflow::Truncate;
        let over = TextOverflow::Overflow {
            excess_emu: Emu(500_000),
        };
        // Variants can be pattern-matched
        assert!(matches!(fit, TextOverflow::Fit));
        assert!(matches!(trunc, TextOverflow::Truncate));
        assert!(matches!(over, TextOverflow::Overflow { .. }));
    }

    /// `FrameContent` variants cover all required content types.
    #[test]
    fn test_bc_3_06_001_frame_content_variants_exist() {
        use std::sync::Arc;

        let title = FrameContent::Title(Arc::from("Hello"));
        let subtitle = FrameContent::Subtitle(Arc::from("World"));
        let body = FrameContent::Body(vec![]);
        let image = FrameContent::Image {
            alt: Arc::from("A bar chart"),
        };
        let chart = FrameContent::Chart;
        let diagram = FrameContent::Diagram;
        let shape = FrameContent::Shape;
        let empty = FrameContent::Empty;

        assert!(matches!(title, FrameContent::Title(_)));
        assert!(matches!(subtitle, FrameContent::Subtitle(_)));
        assert!(matches!(body, FrameContent::Body(_)));
        assert!(matches!(image, FrameContent::Image { .. }));
        assert!(matches!(chart, FrameContent::Chart));
        assert!(matches!(diagram, FrameContent::Diagram));
        assert!(matches!(shape, FrameContent::Shape));
        assert!(matches!(empty, FrameContent::Empty));
    }

    /// AC-003 — EMU values are correct for the 16:9 default constants.
    #[test]
    fn test_bc_3_06_001_default_page_constants() {
        assert_eq!(DEFAULT_PAGE_WIDTH, Emu(9_144_000));
        assert_eq!(DEFAULT_PAGE_HEIGHT, Emu(5_143_500));
        assert_eq!(STANDARD_4X3_PAGE_HEIGHT, Emu(6_858_000));
    }
}
