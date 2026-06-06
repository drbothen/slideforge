//! Layout IR types — `LaidOutDeck`, `LaidOutSlide`, `Frame`, `BoundingBox`,
//! `TextFlow`, `PageSize`, `ShapeFrame`, `ShapeType`, `FillSpec`, `Rgb`,
//! `LayoutWarning`.
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
use slideforge_types::InlineNode;
pub use slideforge_types::NormalizedDiagramSvg;
pub use slideforge_types::RegisteredContent;
// Re-export shape/warning types relocated to slideforge-types (STORY-028 pass-2).
// Downstream code that imports these through slideforge-layout sees no change.
pub use slideforge_types::{AltText, FillSpec, LayoutWarning, Rgb, ShapeType};

use crate::sections::GeneratedSection;

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
///
/// ## Document sections (STORY-027 / BC-3.02.001 / BC-3.02.002)
///
/// `sections` holds the assembled document sections for DOCX and PDF output.
/// The list is populated by [`crate::sections::collect_sections`] during
/// `layout::run`. PPTX and HTML exporters filter out sections where their
/// format is absent from [`crate::sections::GeneratedSection::target_formats`].
///
/// ## Non-fatal warnings (STORY-028 / BC-3.04.001 EC-002 / BC-3.05.001 EC-002)
///
/// `warnings` accumulates all non-fatal diagnostics produced during layout:
/// off-canvas shape positions ([`LayoutWarning::OffCanvas`]) and unresolved
/// xref targets ([`LayoutWarning::XrefTargetNotFound`]). Exporters and
/// validators may inspect this field to surface warnings to the user without
/// halting the pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LaidOutDeck {
    /// The page dimensions for all slides in this deck.
    pub page_size: PageSize,
    /// The laid-out slides, one per entry in [`slideforge_types::Deck::slides`].
    pub slides: Vec<LaidOutSlide>,
    /// The assembled document sections for DOCX and PDF output.
    ///
    /// Populated by [`crate::sections::collect_sections`] during layout.
    /// An empty `Vec` means the deck has no sections (no `takeaway:` fields,
    /// no `severity_cards` slides, and no manual `section:` blocks).
    pub sections: Vec<GeneratedSection>,
    /// Non-fatal diagnostics accumulated during layout.
    ///
    /// Includes off-canvas shape warnings ([`LayoutWarning::OffCanvas`]) and
    /// unresolved xref warnings ([`LayoutWarning::XrefTargetNotFound`]).
    /// An empty `Vec` means the layout was clean.
    ///
    /// Populated by [`crate::layout::run`] from the shape layout pass
    /// (BC-3.04.001 EC-002) and the inline validation pass
    /// (BC-3.05.001 EC-002 / AC-007).
    pub warnings: Vec<LayoutWarning>,
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

    /// Register-tagged content blocks for this slide.
    ///
    /// Populated by the `extract_register_content` pass in `slideforge-eval`
    /// (STORY-035 / BC-1.14.001/002/003). Each entry carries a [`RegisteredContent`]
    /// value that pairs a [`slideforge_types::Register`] tag with its evaluated
    /// inline content.
    ///
    /// Exporters read only the entries for their allowed registers:
    /// - PPTX: reads `Notes` entries only (speaker notes)
    /// - DOCX: reads `Notes`, `Report`, and `Detail` entries
    /// - PDF: reads `Report` and `Detail` entries
    /// - HTML/preview: reads `Notes` only (presenter panel)
    ///
    /// An empty `Vec` means the slide has no register-gated content — all content
    /// is visual and stored in `frames`.
    ///
    /// # Invariant (BC-1.14.004 invariant 3)
    ///
    /// No text present in any `RegisteredContent` entry may also appear in
    /// `frames`. Register fields (`notes`, `report`, `detail`) intentionally
    /// REMAIN in `slide.fields` after eval — they are NOT removed by the
    /// evaluator. The no-bleed guarantee is enforced by ALLOWLIST-based frame
    /// construction in the layout engine: frames read only `title`, `subtitle`,
    /// and `body` from `slide.fields` and never enumerate arbitrary keys (see
    /// `layout.rs` ALLOWLIST GUARANTEE comment). This field is the single
    /// authoritative source for all register-gated content for exporters.
    pub register_content: Vec<RegisteredContent>,
}

/// The semantic role of a pre-allocated region slot in a slide's region map.
///
/// `RegionRole` is set by [`crate::regions::region_frames_for`] on each
/// `FrameContent::Empty` slot to identify which tag of content should claim
/// that slot. The layout engine's [`crate::layout::fill_region_slot_or_append`]
/// uses this role — not slot position — to route `TextTag`-bearing blocks into
/// the correct geometry region, ensuring tag-driven slot selection regardless of
/// block processing order (AC-023 / BC-4.01.001 v1.2 invariant 5).
///
/// ## Why this matters
///
/// Without an explicit role, "first Empty slot" selection causes position-driven
/// bugs: a `TextTag::Body` block arriving before `TextTag::Title` in the block
/// list would claim the title-region geometry (slot 0), and the title would land
/// in the body-region geometry (slot 1). `RegionRole` makes slot selection
/// order-independent.
///
/// ## Lifecycle
///
/// - **Set by:** `region_frames_for` on every `FrameContent::Empty` placeholder.
/// - **Consumed by:** `fill_region_slot_or_append` to find the matching slot.
/// - **Cleared to `None` after filling:** once a slot is filled, `region_role`
///   becomes semantically irrelevant (the slot is no longer `Empty`). The field
///   is left as-is after filling; callers must not rely on it post-fill.
/// - **`None` for non-region-map frames:** appended frames (shapes, inline text
///   runs, dynamically-generated frames) carry `region_role: None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionRole {
    /// The frame is the slide's primary title region.
    ///
    /// Claimed by `TextTag::Title` blocks via `fill_region_slot_or_append`.
    Title,
    /// The frame is a subtitle or secondary heading region.
    ///
    /// Claimed by `TextTag::Subtitle` blocks via `fill_region_slot_or_append`.
    Subtitle,
    /// The frame is the slide's primary body (content) region.
    ///
    /// Claimed by `TextTag::Body` blocks via `fill_region_slot_or_append`.
    Body,
    /// The frame is a generic slot without a fixed tag role.
    ///
    /// Used for multi-slot layouts (e.g., `stat_callout` stat/label regions,
    /// `two_col` second column) where the slot is not semantically typed.
    /// `fill_region_slot_or_append` falls back to this slot for any tag that
    /// has no `RegionRole::Title`/`Subtitle`/`Body` match remaining.
    Generic,
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

    /// The semantic role of this pre-allocated region slot.
    ///
    /// Set by [`crate::regions::region_frames_for`] to enable tag-driven slot
    /// selection in [`crate::layout::fill_region_slot_or_append`] (AC-023 /
    /// BC-4.01.001 v1.2 invariant 5). `None` for appended frames (shapes,
    /// inline text runs, dynamically-generated frames) that are not pre-allocated
    /// region slots.
    ///
    /// See [`RegionRole`] for the full lifecycle description.
    pub region_role: Option<RegionRole>,
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

// ─────────────────────────────────────────────────────────────────────────────
// BC-3.04.001 — Shape layout IR types (STORY-028)
// ─────────────────────────────────────────────────────────────────────────────
//
// `Rgb`, `FillSpec`, `ShapeType`, and `LayoutWarning` are defined in
// `slideforge_types::shape_types` and re-exported above via
// `pub use slideforge_types::{FillSpec, LayoutWarning, Rgb, ShapeType}`.
//
// They live in slideforge-types (the leaf IR crate) so that `ShapeSpec` (the
// pre-layout semantic type) can carry a resolved `ShapeType` directly without a
// circular dependency. slideforge-layout re-exports them for backward compat.

/// A fully-positioned shape from the `shape:` DSL block.
///
/// Produced by [`crate::shapes::layout_shapes`] during the layout pass.
/// Carries the shape's type, fill specification, optional text content,
/// and resolved accessibility alt text.
///
/// Implements `Hash + Eq + Clone + Debug` for comemo compatibility (AC-010).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapeFrame {
    /// The geometric shape kind.
    pub shape_type: ShapeType,
    /// The fill specification for this shape.
    pub fill: FillSpec,
    /// Optional inline text content rendered inside the shape.
    ///
    /// `None` means the shape has no text label.
    pub text: Option<Vec<InlineNode>>,
    /// Accessibility alt text for this shape (BC-3.04.001 precondition 3 /
    /// AC-004: `decorative: true` → `AltText::Decorative`).
    pub alt: slideforge_types::AltText,
}

// `LayoutWarning` is defined in `slideforge_types::shape_types` and re-exported
// above. See the comment block preceding `ShapeFrame` for context.

// ─────────────────────────────────────────────────────────────────────────────
// FrameContent
// ─────────────────────────────────────────────────────────────────────────────

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
    ///
    /// `AltText::Provided(s)` — non-empty alt text for this image (BC-4.01.004).
    /// `AltText::Decorative` — image explicitly marked decorative (`decorative: true`);
    /// the PPTX exporter emits `descr=""` and the PDF exporter emits an Artifact tag.
    Image {
        /// Accessibility alt text for this image (STORY-039 IR alt-threading).
        ///
        /// Threaded from `ImageSpec.alt` in the semantic IR via `layout::run`.
        /// When `ImageSpec.alt` is `None` (upstream validator miss), layout maps it
        /// to `AltText::Unspecified` and emits `tracing::warn!` (EC-006/EC-007).
        alt: AltText,
    },
    /// A chart rendered from a chart spec.
    ///
    /// `AltText::Provided(s)` — non-empty alt text for this chart (BC-4.01.004 EC-005).
    /// `AltText::Decorative` — chart explicitly marked decorative.
    ///
    /// Alt text is threaded from `ChartSpec.alt` via `layout::run` (STORY-039).
    /// When `ChartSpec.alt` is `None`, maps to `AltText::Unspecified` + `tracing::warn!`.
    Chart {
        /// Accessibility alt text for the chart, threaded from `ChartSpec.alt`.
        alt: AltText,
    },
    /// A diagram rendered from a diagram source (e.g., Mermaid).
    ///
    /// Carries the PPTX-safe, usvg-normalized SVG payload produced by
    /// `slideforge_diagrams::normalize::usvg_normalize` (BC-1.12.003 invariant 1).
    /// The type system enforces that only a [`NormalizedDiagramSvg`] — never a
    /// raw SVG string — can be stored in this frame, preventing un-normalized
    /// SVG from reaching exporters.
    ///
    /// `AltText::Provided(s)` — non-empty alt text for this diagram (BC-4.01.004 EC-005).
    /// `AltText::Decorative` — diagram explicitly marked decorative.
    ///
    /// Alt text is threaded from `DiagramSpec.alt` via `layout::run` (STORY-039).
    /// When `DiagramSpec.alt` is `None`, maps to `AltText::Unspecified` + `tracing::warn!`.
    Diagram {
        /// The PPTX-safe, usvg-normalized SVG payload.
        svg: NormalizedDiagramSvg,
        /// Accessibility alt text, threaded from `DiagramSpec.alt`.
        alt: AltText,
    },
    /// A shape from the shape DSL (BC-3.04.001, STORY-028).
    ///
    /// Carries the fully-resolved shape geometry, fill, text content, and
    /// accessibility alt text. Produced by [`crate::shapes::layout_shapes`]
    /// and appended after all placeholder frames in [`LaidOutSlide::frames`]
    /// (AC-002 / BC-3.04.001 postcondition 4).
    Shape(ShapeFrame),
    /// A rich inline text run (BC-3.05.001, STORY-028).
    ///
    /// Carries a sequence of [`InlineNode`] values that have been validated by
    /// the inline pass ([`crate::inline`]). The layout stage preserves the
    /// `InlineNode` sequence verbatim; exporters translate each variant to
    /// format-specific markup (OMML for PPTX, HTML tags for HTML, etc.).
    ///
    /// Produced for text blocks that carry inline formatting (bold, italic, code,
    /// xref, etc.). Plain-text-only blocks continue to use
    /// `FrameContent::Title` / `FrameContent::Subtitle` / `FrameContent::Body`.
    TextRun(Vec<InlineNode>),
    /// An empty placeholder (present in the layout but no content assigned).
    Empty,
    /// An error-slide placeholder produced when a pipeline error occurs in
    /// warn-only mode (e.g., empty chart data detected before rendering).
    ///
    /// This variant is produced instead of the normal chart/diagram content
    /// when the pipeline encounters a recoverable error in
    /// `slideforge_validate::ValidationMode::WarnOnly` mode. Exporters
    /// (STORY-037 and later) render this as a light gray slide with the error
    /// message overlaid as text.
    ///
    /// ## When produced
    ///
    /// Currently produced by the empty-data guard (STORY-032, BC-1.11.002
    /// postcondition 3) when `Value::List([])` is detected before
    /// `ChartRendererImpl::dispatch_and_process` is called.
    ///
    /// ## Exporter contract
    ///
    /// The `svg` field is a pre-rendered, PPTX-safe SVG string produced by
    /// `slideforge_charts::placeholder::build_error_slide_placeholder_svg`.
    /// Exporters that do not support inline SVG fallback to rendering
    /// `error_code` and `message` as plain text on a gray background.
    ErrorSlidePlaceholder {
        /// Pre-rendered error-slide SVG from
        /// `slideforge_charts::placeholder::build_error_slide_placeholder_svg`.
        ///
        /// Self-contained SVG; no external references. PPTX-safe (no `<script>`
        /// or `<foreignObject>`).
        svg: Arc<str>,
        /// The title of the slide where the error occurred.
        slide_title: Arc<str>,
        /// The error taxonomy code (e.g., `"E-LAY-003"`).
        error_code: Arc<str>,
        /// Human-readable error message displayed on the placeholder slide.
        message: Arc<str>,
    },
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
    /// When a slide is configured with `overflow: truncate`, the layout engine
    /// sets this variant instead of [`TextOverflow::Overflow`]. The validator
    /// suppresses the canvas-overflow warning for `Truncate` frames because the
    /// author has explicitly opted in to clipping.
    ///
    /// Not produced by the current layout engine — all overflow currently
    /// results in `TextOverflow::Overflow`. This variant exists in the type to
    /// preserve exhaustive match coverage; it will be activated in the
    /// content-resolution wrapping story (deferred to v1.x — story TBD).
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
            sections: vec![],
            warnings: vec![],
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
            register_content: vec![],
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
            region_role: None,
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
            alt: slideforge_types::AltText::Provided(Arc::from("A bar chart")),
        };
        let chart = FrameContent::Chart {
            alt: slideforge_types::AltText::Decorative,
        };
        // Construct a minimal NormalizedDiagramSvg for the Diagram variant test.
        let normalized_svg = NormalizedDiagramSvg::from_normalized_string(Arc::from(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><title>test</title></svg>"#,
        ));
        let diagram = FrameContent::Diagram {
            svg: normalized_svg,
            alt: slideforge_types::AltText::Decorative,
        };
        // BC-3.04.001 / STORY-028: Shape now carries a ShapeFrame.
        let shape = FrameContent::Shape(ShapeFrame {
            shape_type: ShapeType::Rect,
            fill: FillSpec::SolidColor(Rgb {
                r: 0,
                g: 55,
                b: 102,
            }),
            text: None,
            alt: slideforge_types::AltText::Provided(Arc::from("Blue rectangle")),
        });
        // BC-3.05.001 / STORY-028: TextRun carries a Vec<InlineNode>.
        let text_run = FrameContent::TextRun(vec![]);
        let empty = FrameContent::Empty;

        assert!(matches!(title, FrameContent::Title(_)));
        assert!(matches!(subtitle, FrameContent::Subtitle(_)));
        assert!(matches!(body, FrameContent::Body(_)));
        assert!(matches!(image, FrameContent::Image { .. }));
        assert!(matches!(chart, FrameContent::Chart { .. }));
        assert!(matches!(diagram, FrameContent::Diagram { .. }));
        assert!(matches!(shape, FrameContent::Shape(_)));
        assert!(matches!(text_run, FrameContent::TextRun(_)));
        assert!(matches!(empty, FrameContent::Empty));
    }

    /// BC-1.11.002 AC-004 / STORY-032 — `FrameContent::ErrorSlidePlaceholder` variant exists
    /// with `svg`, `slide_title`, `error_code`, and `message` fields.
    ///
    /// This test exercises the type in isolation — it does not depend on any
    /// implementation in `slideforge-charts`.
    #[test]
    fn test_bc_1_11_002_error_slide_placeholder_variant_exists() {
        use std::sync::Arc;

        let placeholder = FrameContent::ErrorSlidePlaceholder {
            svg: Arc::from("<svg/>"),
            slide_title: Arc::from("Revenue Chart"),
            error_code: Arc::from("E-LAY-003"),
            message: Arc::from("Chart data is empty for slide 'Revenue Chart'"),
        };
        assert!(
            matches!(placeholder, FrameContent::ErrorSlidePlaceholder { .. }),
            "ErrorSlidePlaceholder variant must be pattern-matchable"
        );
    }

    /// BC-1.11.002 AC-004 — `FrameContent::ErrorSlidePlaceholder` implements
    /// `Clone + PartialEq + Eq + Hash` (comemo AC-010 requirement).
    #[test]
    fn test_bc_1_11_002_error_slide_placeholder_implements_hash_eq_clone() {
        use std::collections::HashSet;
        use std::sync::Arc;

        let p1 = FrameContent::ErrorSlidePlaceholder {
            svg: Arc::from("<svg/>"),
            slide_title: Arc::from("Revenue Chart"),
            error_code: Arc::from("E-LAY-003"),
            message: Arc::from("Chart data is empty for slide 'Revenue Chart'"),
        };
        let p2 = p1.clone();
        assert_eq!(p1, p2, "ErrorSlidePlaceholder must implement PartialEq");

        let mut set = HashSet::new();
        set.insert(p1);
        assert_eq!(set.len(), 1, "ErrorSlidePlaceholder must be hashable");
    }

    /// F-HIGH-003: `FrameContent::Diagram` must carry a `NormalizedDiagramSvg`
    /// payload. This test verifies the type-level contract by constructing the
    /// variant and extracting the payload.
    #[test]
    fn test_frame_content_diagram_carries_normalized_svg() {
        use std::sync::Arc;

        let svg_str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300" aria-label="test" role="img"><title>test</title><rect x="0" y="0" width="400" height="300"/></svg>"#;
        let normalized = NormalizedDiagramSvg::from_normalized_string(Arc::from(svg_str));
        let content = FrameContent::Diagram {
            svg: normalized,
            alt: slideforge_types::AltText::Decorative,
        };

        match content {
            FrameContent::Diagram { svg, .. } => {
                assert!(
                    svg.as_str().contains("<svg"),
                    "FrameContent::Diagram payload must be an SVG; got: {}",
                    svg.as_str()
                );
                assert!(
                    svg.as_str().contains("<title>"),
                    "FrameContent::Diagram payload must contain <title>"
                );
            },
            other => panic!("expected FrameContent::Diagram, got: {other:?}"),
        }
    }

    /// AC-003 — EMU values are correct for the 16:9 default constants.
    #[test]
    fn test_bc_3_06_001_default_page_constants() {
        assert_eq!(DEFAULT_PAGE_WIDTH, Emu(9_144_000));
        assert_eq!(DEFAULT_PAGE_HEIGHT, Emu(5_143_500));
        assert_eq!(STANDARD_4X3_PAGE_HEIGHT, Emu(6_858_000));
    }
}
