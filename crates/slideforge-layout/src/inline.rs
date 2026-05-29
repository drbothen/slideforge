//! Inline node validation and xref-target checking — BC-3.05.001 (STORY-028).
//!
//! This module implements the two responsibilities of the layout stage with
//! respect to inline content:
//!
//! 1. **All 12 [`slideforge_types::InlineNode`] variants are
//!    preserved verbatim** in [`crate::types::FrameContent::TextRun`] records
//!    in [`crate::types::LaidOutSlide::frames`]. The layout stage does NOT
//!    produce format-specific markup — it only carries the variant type and its
//!    attributes (AC-005 / BC-3.05.001 postcondition).
//!
//! 2. **Xref target validation (AC-007 / BC-3.05.001 EC-002):** During layout,
//!    `InlineNode::Xref { target }` references are validated against the slide
//!    titles in the [`slideforge_types::Deck`]. Unknown xref targets accumulate
//!    [`crate::types::LayoutWarning::XrefTargetNotFound`] via a warning sink but
//!    do NOT halt layout.
//!
//! ## Nesting (AC-006 / BC-3.05.001 EC-001)
//!
//! Nested inline formatting (e.g., `Bold(vec![Italic(...)])`) is preserved
//! at full nesting depth. Exporters receive the full tree and produce
//! format-specific merged markup (e.g., PPTX `<a:rPr b="1" i="1">`).
//!
//! ## No wildcard catch-all (DI-004)
//!
//! The validation pass MUST handle ALL `InlineNode` variants without a
//! wildcard catch-all. Missing variants are a compile error (AC-005 /
//! architecture rule 4 from the story spec).

use std::collections::HashSet;
use std::sync::Arc;

use slideforge_types::InlineNode;

use crate::error::LayoutError;
use crate::types::LayoutWarning;

/// Maximum safe inline nesting depth (BC-3.05.001 E-LAY-005 / F-MED-006).
///
/// Inline nodes may be arbitrarily nested (e.g., `Bold(Italic(Superscript(...)))`).
/// Beyond this depth the recursive traversal risks stack overflow in deeply-nested
/// inputs. Layout returns `LayoutError::InlineDepthExceeded` rather than recurse
/// further.
///
/// This is a Kani candidate (VP-045): the proof bounds nesting depth and verifies
/// that traversal always terminates within `MAX_INLINE_DEPTH` frames.
pub const MAX_INLINE_DEPTH: usize = 64;

/// Validate a slice of [`InlineNode`]s and collect any xref warnings.
///
/// This function:
/// - Traverses the full `InlineNode` tree (including nested children) without
///   a wildcard match arm — all 12 variants are handled explicitly.
/// - For every `InlineNode::Xref(target)`, checks whether `target` matches one
///   of the `known_slide_titles`. If not, pushes a
///   [`LayoutWarning::XrefTargetNotFound`] into `warnings`.
/// - Returns the original node sequence unchanged (the layout stage preserves
///   inline content verbatim).
/// - Enforces a maximum nesting depth of [`MAX_INLINE_DEPTH`] (64). Exceeding
///   this depth returns `Err(LayoutError::InlineDepthExceeded)` (F-MED-006 /
///   BC-3.05.001 E-LAY-005).
///
/// # Arguments
///
/// * `nodes` — The inline node sequence to validate.
/// * `known_slide_titles` — The set of slide title strings from the deck.
/// * `source_slide_index` — Zero-based index of the slide being validated
///   (interface-definitions.md §8: canonical field name `source_slide_index`).
/// * `warnings` — Mutable sink for accumulated warnings (DI-018).
///
/// # Errors
///
/// Returns `Err(LayoutError::InlineDepthExceeded)` if any path in the inline
/// node tree exceeds [`MAX_INLINE_DEPTH`] nesting levels.
pub fn validate_inline_nodes<S: ::std::hash::BuildHasher>(
    nodes: &[InlineNode],
    known_slide_titles: &HashSet<Arc<str>, S>,
    source_slide_index: usize,
    warnings: &mut Vec<LayoutWarning>,
) -> Result<(), LayoutError> {
    for node in nodes {
        check_inline_node(node, known_slide_titles, source_slide_index, warnings, 0)?;
    }
    Ok(())
}

/// Collect all slide title strings from the deck into a `HashSet`.
///
/// Used to build the known-titles set for [`validate_inline_nodes`] in a
/// single allocation rather than re-scanning the deck for each slide.
///
/// # Arguments
///
/// * `deck` — The semantic deck IR to scan.
///
/// # Returns
///
/// A `HashSet<Arc<str>>` containing every slide title string found in the deck.
/// Slides without a title field are not represented (they cannot be xref targets).
#[must_use]
pub fn collect_slide_titles(deck: &slideforge_types::Deck) -> HashSet<Arc<str>> {
    deck.slides
        .iter()
        .filter_map(|slide| slide.title_str().map(Arc::from))
        .collect()
}

/// Run the inline validation pass for all slides in a `LaidOutDeck`.
///
/// This is the entry point called by [`crate::layout::run`] after the per-slide
/// frame pass. It:
/// 1. Calls [`collect_slide_titles`] to build the known-titles set.
/// 2. For each `LaidOutSlide`, scans all `FrameContent::TextRun` frames AND
///    all `FrameContent::Shape` frames whose `text` field is `Some` for
///    `InlineNode` sequences (F-P4-MED-002: shape text bypass fix).
/// 3. Calls [`validate_inline_nodes`] for each inline sequence.
/// 4. Returns the accumulated `Vec<LayoutWarning>`.
///
/// ## Shape text scanning (F-P4-MED-002 / BC-3.05.001 EC-002)
///
/// `FrameContent::Shape(ShapeFrame { text: Some(nodes), .. })` frames are
/// scanned with the same xref-target and depth-bound checks applied to
/// `FrameContent::TextRun` frames. Without this, a user writing
/// `shape: type rect ... text "see {{ xref(\"unknown\") }}"` would bypass
/// the AC-007 xref-target validation gate (BC-3.05.001 EC-002) and the
/// E-LAY-005 depth bound.
///
/// ## Scope — what is scanned and what is not (F-P5-LOW-001)
///
/// **Currently scanned:**
/// - `FrameContent::TextRun(nodes)` — body text frames produced by
///   `ContentBlock::Text` during the synthetic-frame pass in `layout::run`
/// - `FrameContent::Shape(ShapeFrame { text: Some(nodes), .. })` — shape-embedded
///   text (added by F-P4-MED-002)
///
/// **Currently NOT scanned (deferred — STORY-073):**
/// - `ContentBlock::Bullets(Vec<BulletItem>)` — bullets carry `BulletItem.inlines: Vec<InlineNode>`.
///   Bullet-list layout passes are not yet built; once bullets produce frames, this validation must
///   extend to cover them.
///   STORY-073 owns `ContentBlock::Bullets → FrameContent::TextRun frame generation`
///   (body-layout pass). Xref validation inside bullets will be wired in that story.
///
/// Other `ContentBlock` variants (`Text`, `Shape`, `Math`, `Chart`, `Diagram`, `Image`, `Table`)
/// either flow through this validation already (`Text` → `TextRun`; `Shape` → `Shape.text`) or do
/// NOT carry `InlineNode` subtrees and so don't require scanning.
///
/// # Arguments
///
/// * `deck` — The semantic deck IR (used for title collection).
/// * `laid_out_slides` — The laid-out slides (mutably scanned but not modified).
///
/// # Returns
///
/// A `Vec<LayoutWarning>` containing all xref-not-found warnings accumulated
/// across all slides.
///
/// # Errors
///
/// Returns `Err(LayoutError::InlineDepthExceeded)` if any inline node tree in
/// any `TextRun` or `Shape` frame exceeds [`MAX_INLINE_DEPTH`] nesting levels
/// (BC-3.05.001 E-LAY-005 / F-MED-006).
pub fn run_inline_validation(
    deck: &slideforge_types::Deck,
    laid_out_slides: &[crate::types::LaidOutSlide],
) -> Result<Vec<LayoutWarning>, LayoutError> {
    let known_titles = collect_slide_titles(deck);
    let mut warnings = Vec::new();

    for slide in laid_out_slides {
        for frame in &slide.frames {
            match &frame.content {
                crate::types::FrameContent::TextRun(nodes) => {
                    validate_inline_nodes(nodes, &known_titles, slide.source_index, &mut warnings)?;
                }
                crate::types::FrameContent::Shape(shape_frame) => {
                    if let Some(nodes) = &shape_frame.text {
                        validate_inline_nodes(
                            nodes,
                            &known_titles,
                            slide.source_index,
                            &mut warnings,
                        )?;
                    }
                }
                // Variants below carry no InlineNode subtrees in v1.0 and require
                // no inline validation. Listed individually with `|` rather than as
                // a wildcard `_` so the compiler will flag any future FrameContent
                // variant addition that is missing from this arm — the exhaustiveness
                // check is the contract (architecture rule cited in story spec line 370).
                // Bullet-list inline scanning is owned by STORY-073.
                crate::types::FrameContent::Title(_)
                | crate::types::FrameContent::Subtitle(_)
                | crate::types::FrameContent::Body(_)
                | crate::types::FrameContent::Image { .. }
                | crate::types::FrameContent::Chart
                | crate::types::FrameContent::Diagram(_)
                | crate::types::FrameContent::Empty
                | crate::types::FrameContent::ErrorSlidePlaceholder { .. } => {}
            }
        }
    }

    Ok(warnings)
}

/// Check whether a single [`InlineNode`] subtree contains any xref nodes that
/// reference unknown titles, and push warnings into the sink.
///
/// This is the recursive helper for [`validate_inline_nodes`]. It MUST NOT use
/// a wildcard `_ => {}` catch-all — every `InlineNode` variant must be
/// explicitly handled (AC-005 / architecture rule 4).
///
/// ## Math is a leaf (AC-BC-A8 / BC-3.05.001 §J / F-HIGH-002)
///
/// `InlineNode::Math` is treated as a **leaf node** — the layout stage does NOT
/// recurse into the `MathNode` contents. The LaTeX source string inside a `MathNode`
/// is opaque math markup, not a text content tree; it does not contain `InlineNode`
/// children and must not be inspected for xref targets.
///
/// ## Depth bound (F-MED-006 / BC-3.05.001 E-LAY-005)
///
/// When `depth > MAX_INLINE_DEPTH`, returns
/// `Err(LayoutError::InlineDepthExceeded)` rather than recursing further.
/// Depths 0 through `MAX_INLINE_DEPTH` (64) are accepted; depth 65 and above
/// are rejected (BC + VP-045 + AC-BC-A7).
///
/// # Errors
///
/// Returns `Err(LayoutError::InlineDepthExceeded)` when `depth > MAX_INLINE_DEPTH`
/// (i.e., the tree depth being processed is `MAX_INLINE_DEPTH + 1` = 65 or more).
/// Depth 64 (== `MAX_INLINE_DEPTH`) is the last accepted level.
pub fn check_inline_node<S: ::std::hash::BuildHasher>(
    node: &InlineNode,
    known_slide_titles: &HashSet<Arc<str>, S>,
    source_slide_index: usize,
    warnings: &mut Vec<LayoutWarning>,
    depth: usize,
) -> Result<(), LayoutError> {
    if depth > MAX_INLINE_DEPTH {
        return Err(LayoutError::InlineDepthExceeded {
            source_slide_index,
            depth,
            max: MAX_INLINE_DEPTH,
        });
    }

    match node {
        // Leaf variants — no children, no xref.
        // Math is treated as a leaf: the LaTeX source is opaque markup and does
        // not contain InlineNode children (AC-BC-A8 / BC-3.05.001 §J).
        InlineNode::Plain(_) | InlineNode::Code(_) | InlineNode::Math(_) => {},

        // Xref — validate the target.
        InlineNode::Xref(target) => {
            if !known_slide_titles.contains(target.as_ref()) {
                warnings.push(LayoutWarning::XrefTargetNotFound {
                    target: target.clone(),
                    source_slide_index,
                });
            }
        },

        // Container variants — recurse into children.
        InlineNode::Bold(children)
        | InlineNode::Italic(children)
        | InlineNode::Footnote(children)
        | InlineNode::Superscript(children)
        | InlineNode::Subscript(children)
        | InlineNode::Strikethrough(children)
        | InlineNode::Highlight(children) => {
            for child in children {
                check_inline_node(
                    child,
                    known_slide_titles,
                    source_slide_index,
                    warnings,
                    depth + 1,
                )?;
            }
        },

        // Link — recurse into display text children only; URL is not an xref target.
        InlineNode::Link { text, url: _ } => {
            for child in text {
                check_inline_node(
                    child,
                    known_slide_titles,
                    source_slide_index,
                    warnings,
                    depth + 1,
                )?;
            }
        },
    }

    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::doc_markdown
)]
mod tests {
    use super::*;
    use slideforge_types::{InlineNode, MathNode, SourceSpan};
    use std::sync::Arc;

    // ─────────────────────────────────────────────────────────────────────────
    // Test helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn empty_titles() -> HashSet<Arc<str>> {
        HashSet::new()
    }

    fn titles_with(names: &[&str]) -> HashSet<Arc<str>> {
        names.iter().map(|s| Arc::from(*s)).collect()
    }

    /// Build one of every InlineNode variant (12 total).
    /// Used to verify exhaustive handling without wildcard.
    fn all_12_variants() -> Vec<InlineNode> {
        vec![
            InlineNode::Plain(Arc::from("plain text")),
            InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold content"))]),
            InlineNode::Italic(vec![InlineNode::Plain(Arc::from("italic content"))]),
            InlineNode::Code(Arc::from("fn foo() {}")),
            InlineNode::Link {
                text: vec![InlineNode::Plain(Arc::from("link text"))],
                url: Arc::from("https://example.com"),
            },
            InlineNode::Math(MathNode {
                latex: Arc::from("x^2 + y^2"),
                display: false,
                span: SourceSpan::default(),
            }),
            InlineNode::Footnote(vec![InlineNode::Plain(Arc::from("footnote content"))]),
            InlineNode::Xref(Arc::from("introduction")), // "introduction" is in known_titles
            InlineNode::Superscript(vec![InlineNode::Plain(Arc::from("2"))]),
            InlineNode::Subscript(vec![InlineNode::Plain(Arc::from("n"))]),
            InlineNode::Strikethrough(vec![InlineNode::Plain(Arc::from("deleted text"))]),
            InlineNode::Highlight(vec![InlineNode::Plain(Arc::from("highlighted"))]),
        ]
    }

    fn make_deck_with_titles(titles: &[&str]) -> slideforge_types::Deck {
        use slideforge_types::Slide;
        use slideforge_types::ordered_map::OrderedMap;
        use slideforge_types::slide::FieldValue;
        use slideforge_types::value::Value;

        let slides = titles
            .iter()
            .map(|t| Slide {
                slide_type: Arc::from("bullets"),
                fields: {
                    let mut m = OrderedMap::new();
                    m.insert(
                        Arc::from("title"),
                        FieldValue::Literal(Value::Str(Arc::from(*t))),
                    );
                    m
                },
                blocks: vec![],
                register: None,
                tags: vec![],
                source_span: SourceSpan::default(),
            })
            .collect();

        slideforge_types::Deck {
            slides,
            vars: OrderedMap::new(),
            metadata: slideforge_types::DeckMetadata {
                title: None,
                slideforge_version: Arc::from("0.1.0"),
                lang: Some(Arc::from("en-US")),
                author: None,
                section_order: None,
            },
            registers: OrderedMap::new(),
            section_blocks: vec![],
        }
    }

    fn make_laid_out_slide_with_text_run(
        source_slide_index: usize,
        nodes: Vec<InlineNode>,
    ) -> crate::types::LaidOutSlide {
        use crate::types::{BoundingBox, Emu, Frame, FrameContent, LaidOutSlide};

        LaidOutSlide {
            source_index: source_slide_index,
            slide_type_keyword: Arc::from("bullets"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(5_143_500),
                },
                content: FrameContent::TextRun(nodes),
                text_flow: None,
            }],
            speaker_notes: None,
            register_tags: vec![],
        }
    }

    fn make_laid_out_slide_empty(source_slide_index: usize) -> crate::types::LaidOutSlide {
        use crate::types::{BoundingBox, Emu, Frame, FrameContent, LaidOutSlide};

        LaidOutSlide {
            source_index: source_slide_index,
            slide_type_keyword: Arc::from("title"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(5_143_500),
                },
                content: FrameContent::Empty,
                text_flow: None,
            }],
            speaker_notes: None,
            register_tags: vec![],
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-005 — All 12 InlineNode variants survive validation unchanged
    // (BC-3.05.001 postcondition)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-005 — `validate_inline_nodes` handles all 12 variants without wildcard.
    /// With a known Xref target ("introduction"), produces zero warnings.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac005_all_12_inline_variants_no_warnings_with_known_xref() {
        let nodes = all_12_variants();
        let known = titles_with(&["introduction"]);
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &known, 0, &mut warnings).expect("must not error");
        assert!(
            warnings.is_empty(),
            "all 12 variants with known xref target must produce zero warnings; \
             got: {warnings:?}"
        );
    }

    /// AC-005 — `validate_inline_nodes` preserves nodes unchanged (no mutation).
    ///
    /// The layout stage is a read-only pass; it MUST NOT mutate the inline
    /// content sequence.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac005_nodes_unchanged_after_validation() {
        let original = vec![
            InlineNode::Plain(Arc::from("hello")),
            InlineNode::Bold(vec![InlineNode::Plain(Arc::from("world"))]),
        ];
        let nodes_copy = original.clone();
        let known = empty_titles();
        let mut warnings = vec![];
        validate_inline_nodes(&nodes_copy, &known, 0, &mut warnings).expect("must not error");
        assert_eq!(
            nodes_copy, original,
            "validate_inline_nodes must not mutate the node sequence"
        );
    }

    /// AC-005 — All 12 variant types are present in `all_12_variants()`.
    ///
    /// This is a compile-time/count check: if InlineNode ever gains a 13th variant
    /// and the exhaustive match in `validate_inline_nodes` is missing it,
    /// the compiler will emit an error. The count assertion here is secondary.
    ///
    /// This test does NOT exercise `validate_inline_nodes` directly — it verifies
    /// the test helper construction covers the correct count.
    #[test]
    fn test_bc_3_05_001_ac005_helper_covers_all_12_variants() {
        let variants = all_12_variants();
        assert_eq!(
            variants.len(),
            12,
            "all_12_variants() must yield exactly 12 InlineNode values"
        );
    }

    /// AC-005 — Each of the 12 kind names is present in the test vector.
    ///
    /// Verifies the test helper names are correct so implementers can match
    /// against them.
    #[test]
    fn test_bc_3_05_001_ac005_all_kind_names_present() {
        let variants = all_12_variants();
        let names: Vec<&str> = variants.iter().map(InlineNode::kind_name).collect();
        for expected in &[
            "Plain",
            "Bold",
            "Italic",
            "Code",
            "Link",
            "Math",
            "Footnote",
            "Xref",
            "Superscript",
            "Subscript",
            "Strikethrough",
            "Highlight",
        ] {
            assert!(
                names.contains(expected),
                "kind name '{expected}' must be present in all_12_variants()"
            );
        }
    }

    /// AC-005 — `validate_inline_nodes` on an empty node slice produces zero warnings.
    ///
    /// Edge case: empty text run (e.g., blank body block).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac005_empty_nodes_produces_no_warnings() {
        let nodes: Vec<InlineNode> = vec![];
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &empty_titles(), 0, &mut warnings).expect("must not error");
        assert!(
            warnings.is_empty(),
            "empty node slice must produce zero warnings"
        );
    }

    /// AC-005 — Plain text node produces no warnings.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac005_plain_node_no_warning() {
        let nodes = vec![InlineNode::Plain(Arc::from("Hello world"))];
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &empty_titles(), 0, &mut warnings).expect("must not error");
        assert!(warnings.is_empty(), "Plain node must produce no warnings");
    }

    /// AC-005 — Code node produces no warnings.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac005_code_node_no_warning() {
        let nodes = vec![InlineNode::Code(Arc::from("let x = 42;"))];
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &empty_titles(), 0, &mut warnings).expect("must not error");
        assert!(warnings.is_empty(), "Code node must produce no warnings");
    }

    /// AC-005 — Math node produces no warnings.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac005_math_node_no_warning() {
        let nodes = vec![InlineNode::Math(MathNode {
            latex: Arc::from("E = mc^2"),
            display: true,
            span: SourceSpan::default(),
        })];
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &empty_titles(), 0, &mut warnings).expect("must not error");
        assert!(warnings.is_empty(), "Math node must produce no warnings");
    }

    /// AC-005 — Link node produces no warnings.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac005_link_node_no_warning() {
        let nodes = vec![InlineNode::Link {
            text: vec![InlineNode::Plain(Arc::from("See report"))],
            url: Arc::from("https://example.com"),
        }];
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &empty_titles(), 0, &mut warnings).expect("must not error");
        assert!(warnings.is_empty(), "Link node must produce no warnings");
    }

    /// AC-005 — Superscript node produces no warnings.
    ///
    /// Canonical test vector from BC-3.05.001:
    ///   `superscript: "2"` → `<a:rPr baseline="30000">`; `<sup>2</sup>`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac005_superscript_node_no_warning() {
        let nodes = vec![InlineNode::Superscript(vec![InlineNode::Plain(Arc::from(
            "2",
        ))])];
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &empty_titles(), 0, &mut warnings).expect("must not error");
        assert!(
            warnings.is_empty(),
            "Superscript node must produce no warnings"
        );
    }

    /// AC-005 — Subscript node produces no warnings.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac005_subscript_node_no_warning() {
        let nodes = vec![InlineNode::Subscript(vec![InlineNode::Plain(Arc::from(
            "n",
        ))])];
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &empty_titles(), 0, &mut warnings).expect("must not error");
        assert!(
            warnings.is_empty(),
            "Subscript node must produce no warnings"
        );
    }

    /// AC-005 — Strikethrough node produces no warnings.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac005_strikethrough_node_no_warning() {
        let nodes = vec![InlineNode::Strikethrough(vec![InlineNode::Plain(
            Arc::from("deprecated text"),
        )])];
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &empty_titles(), 0, &mut warnings).expect("must not error");
        assert!(
            warnings.is_empty(),
            "Strikethrough node must produce no warnings"
        );
    }

    /// AC-005 — Highlight node produces no warnings.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac005_highlight_node_no_warning() {
        let nodes = vec![InlineNode::Highlight(vec![InlineNode::Plain(Arc::from(
            "important!",
        ))])];
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &empty_titles(), 0, &mut warnings).expect("must not error");
        assert!(
            warnings.is_empty(),
            "Highlight node must produce no warnings"
        );
    }

    /// AC-005 — Footnote node produces no warnings.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac005_footnote_node_no_warning() {
        let nodes = vec![InlineNode::Footnote(vec![InlineNode::Plain(Arc::from(
            "See appendix.",
        ))])];
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &empty_titles(), 0, &mut warnings).expect("must not error");
        assert!(
            warnings.is_empty(),
            "Footnote node must produce no warnings"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-006 — Nested inline formatting preserved (BC-3.05.001 EC-001)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-006 — Nested `Bold(vec![Italic(vec![Plain])])` survives validation
    /// unchanged (no warnings, no mutation).
    ///
    /// Canonical edge case from BC-3.05.001 EC-001:
    ///   `italic: bold: "text"` → Both applied: PPTX `<a:rPr b="1" i="1">`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac006_nested_bold_italic_preserved() {
        let inner = InlineNode::Plain(Arc::from("doubly styled"));
        let italic = InlineNode::Italic(vec![inner.clone()]);
        let bold = InlineNode::Bold(vec![italic.clone()]);
        let nodes = vec![bold.clone()];

        let nodes_copy = nodes.clone();
        let mut warnings = vec![];
        validate_inline_nodes(&nodes_copy, &empty_titles(), 0, &mut warnings)
            .expect("must not error");

        // 1. No warnings produced
        assert!(
            warnings.is_empty(),
            "nested bold-italic must produce zero warnings"
        );
        // 2. Node structure preserved (no mutation)
        assert_eq!(
            nodes_copy, nodes,
            "validate_inline_nodes must not mutate nested nodes"
        );
        // 3. Verify the nesting depth is intact
        match &nodes_copy[0] {
            InlineNode::Bold(bold_children) => {
                assert_eq!(bold_children.len(), 1, "Bold must have one child");
                match &bold_children[0] {
                    InlineNode::Italic(italic_children) => {
                        assert_eq!(italic_children.len(), 1, "Italic must have one child");
                        assert!(
                            matches!(&italic_children[0], InlineNode::Plain(s) if s.as_ref() == "doubly styled"),
                            "innermost Plain node must be unchanged"
                        );
                    },
                    other => panic!("expected Italic, got: {other:?}"),
                }
            },
            other => panic!("expected Bold, got: {other:?}"),
        }
    }

    /// AC-006 — Deeply nested `Highlight(Superscript(Plain))` is preserved.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac006_nested_highlight_superscript_preserved() {
        let nodes = vec![InlineNode::Highlight(vec![InlineNode::Superscript(vec![
            InlineNode::Plain(Arc::from("42")),
        ])])];
        let nodes_copy = nodes.clone();
        let mut warnings = vec![];
        validate_inline_nodes(&nodes_copy, &empty_titles(), 0, &mut warnings)
            .expect("must not error");
        assert!(warnings.is_empty());
        assert_eq!(
            nodes_copy, nodes,
            "nested Highlight(Superscript(Plain)) must be unchanged"
        );
    }

    /// AC-006 — `check_inline_node` on a nested `Bold` traverses the children.
    ///
    /// The recursive helper must descend into Bold's children, not only the
    /// top-level node. Verify by nesting an unknown Xref inside Bold.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac006_check_inline_node_recurses_into_bold_children() {
        let unknown_xref = InlineNode::Xref(Arc::from("nonexistent-slide"));
        let bold = InlineNode::Bold(vec![unknown_xref]);
        let known = empty_titles();
        let mut warnings = vec![];
        check_inline_node(&bold, &known, 1, &mut warnings, 0).expect("must not error");
        assert_eq!(
            warnings.len(),
            1,
            "unknown Xref nested inside Bold must be found by recursive traversal"
        );
        assert!(
            matches!(
                &warnings[0],
                LayoutWarning::XrefTargetNotFound { target, source_slide_index: 1 }
                if target.as_ref() == "nonexistent-slide"
            ),
            "warning must reference the nested unknown xref target"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-007 — Xref target validation at layout time (BC-3.05.001 EC-002)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-007 — `validate_inline_nodes` with a known xref target produces no warning.
    ///
    /// Canonical test: xref to "introduction" where "introduction" is in known titles.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac007_known_xref_target_produces_no_warning() {
        let nodes = vec![InlineNode::Xref(Arc::from("introduction"))];
        let known = titles_with(&["introduction", "conclusion"]);
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &known, 0, &mut warnings).expect("must not error");
        assert!(
            warnings.is_empty(),
            "Xref to known title must produce zero warnings; got: {warnings:?}"
        );
    }

    /// AC-007 — `validate_inline_nodes` with unknown xref target pushes
    /// `LayoutWarning::XrefTargetNotFound`.
    ///
    /// Canonical test vector from BC-3.05.001 EC-002:
    ///   Xref to "slide-title-that-does-not-exist" → E-EVL-001-class warning.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac007_unknown_xref_target_produces_warning() {
        let nodes = vec![InlineNode::Xref(Arc::from("nonexistent-slide"))];
        let known = titles_with(&["introduction"]);
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &known, 2, &mut warnings).expect("must not error");
        assert_eq!(
            warnings.len(),
            1,
            "one unknown Xref must produce exactly one warning"
        );
        assert!(
            matches!(
                &warnings[0],
                LayoutWarning::XrefTargetNotFound { target, source_slide_index: 2 }
                if target.as_ref() == "nonexistent-slide"
            ),
            "warning must be XrefTargetNotFound with correct target and source_slide_index"
        );
    }

    /// AC-007 — Two unknown xref targets in the same node slice accumulate
    /// two separate warnings (DI-018: accumulate ALL errors).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac007_multiple_unknown_xrefs_accumulate_all_warnings() {
        let nodes = vec![
            InlineNode::Xref(Arc::from("missing-slide-1")),
            InlineNode::Xref(Arc::from("missing-slide-2")),
        ];
        let known = empty_titles();
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &known, 5, &mut warnings).expect("must not error");
        assert_eq!(
            warnings.len(),
            2,
            "two unknown Xrefs must produce two warnings (accumulate ALL, not stop at first)"
        );
        let targets: Vec<&str> = warnings
            .iter()
            .filter_map(|w| {
                if let LayoutWarning::XrefTargetNotFound { target, .. } = w {
                    Some(target.as_ref())
                } else {
                    None
                }
            })
            .collect();
        assert!(
            targets.contains(&"missing-slide-1"),
            "first unknown target must be warned"
        );
        assert!(
            targets.contains(&"missing-slide-2"),
            "second unknown target must be warned"
        );
    }

    /// AC-007 — Mixed slice: one known xref and one unknown xref produces exactly
    /// one warning (for the unknown one only).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac007_known_and_unknown_xref_produces_one_warning() {
        let nodes = vec![
            InlineNode::Xref(Arc::from("introduction")), // known
            InlineNode::Xref(Arc::from("nonexistent")),  // unknown
        ];
        let known = titles_with(&["introduction"]);
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &known, 0, &mut warnings).expect("must not error");
        assert_eq!(
            warnings.len(),
            1,
            "only the unknown xref must generate a warning"
        );
        assert!(
            matches!(&warnings[0], LayoutWarning::XrefTargetNotFound { target, .. } if target.as_ref() == "nonexistent"),
            "warning must reference 'nonexistent'"
        );
    }

    /// AC-007 — `check_inline_node` on a known Xref target produces no warning.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac007_check_inline_node_known_xref_no_warning() {
        let node = InlineNode::Xref(Arc::from("executive-summary"));
        let known = titles_with(&["executive-summary"]);
        let mut warnings = vec![];
        check_inline_node(&node, &known, 0, &mut warnings, 0).expect("must not error");
        assert!(
            warnings.is_empty(),
            "known Xref must produce no warning from check_inline_node"
        );
    }

    /// AC-007 — `check_inline_node` on an unknown Xref target pushes one warning.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_ac007_check_inline_node_unknown_xref_pushes_warning() {
        let node = InlineNode::Xref(Arc::from("slide-99"));
        let known = empty_titles();
        let mut warnings = vec![];
        check_inline_node(&node, &known, 7, &mut warnings, 0).expect("must not error");
        assert_eq!(warnings.len(), 1);
        assert!(matches!(
            &warnings[0],
            LayoutWarning::XrefTargetNotFound { target, source_slide_index: 7 }
            if target.as_ref() == "slide-99"
        ));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // collect_slide_titles
    // ─────────────────────────────────────────────────────────────────────────

    /// `collect_slide_titles` returns a set containing the slide title strings.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_collect_slide_titles_returns_title_set() {
        let deck = make_deck_with_titles(&["Introduction", "Methodology", "Conclusion"]);
        let titles = collect_slide_titles(&deck);
        assert_eq!(titles.len(), 3, "must collect all three slide titles");
        assert!(titles.contains::<str>("Introduction"));
        assert!(titles.contains::<str>("Methodology"));
        assert!(titles.contains::<str>("Conclusion"));
    }

    /// `collect_slide_titles` on an empty deck returns an empty set.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_collect_slide_titles_empty_deck_returns_empty_set() {
        let deck = make_deck_with_titles(&[]);
        let titles = collect_slide_titles(&deck);
        assert!(titles.is_empty(), "empty deck must produce empty title set");
    }

    /// `collect_slide_titles` on a deck with one slide returns a set of size 1.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_collect_slide_titles_single_slide() {
        let deck = make_deck_with_titles(&["Executive Summary"]);
        let titles = collect_slide_titles(&deck);
        assert_eq!(titles.len(), 1);
        assert!(titles.contains("Executive Summary"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // run_inline_validation — integration tests
    // ─────────────────────────────────────────────────────────────────────────

    /// `run_inline_validation` on a deck with no `TextRun` frames returns no warnings.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_run_inline_validation_no_text_run_frames_no_warnings() {
        let deck = make_deck_with_titles(&["Introduction"]);
        let slides = vec![make_laid_out_slide_empty(0)];
        let warnings =
            run_inline_validation(&deck, &slides).expect("run_inline_validation must not error");
        assert!(
            warnings.is_empty(),
            "no TextRun frames → zero warnings; got: {warnings:?}"
        );
    }

    /// `run_inline_validation` on a deck with one `TextRun` frame containing no
    /// xref nodes returns no warnings.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_run_inline_validation_text_run_no_xref_no_warnings() {
        let deck = make_deck_with_titles(&["Introduction"]);
        let nodes = vec![
            InlineNode::Plain(Arc::from("Hello")),
            InlineNode::Bold(vec![InlineNode::Plain(Arc::from("world"))]),
        ];
        let slides = vec![make_laid_out_slide_with_text_run(0, nodes)];
        let warnings =
            run_inline_validation(&deck, &slides).expect("run_inline_validation must not error");
        assert!(
            warnings.is_empty(),
            "TextRun with no xref nodes must produce zero warnings"
        );
    }

    /// `run_inline_validation` detects an unknown xref target in a `TextRun` frame.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_run_inline_validation_unknown_xref_in_text_run_produces_warning() {
        let deck = make_deck_with_titles(&["Introduction"]);
        let nodes = vec![
            InlineNode::Plain(Arc::from("See also: ")),
            InlineNode::Xref(Arc::from("slide-that-does-not-exist")),
        ];
        let slides = vec![make_laid_out_slide_with_text_run(0, nodes)];
        let warnings =
            run_inline_validation(&deck, &slides).expect("run_inline_validation must not error");
        assert_eq!(
            warnings.len(),
            1,
            "one unknown xref must produce one warning; got: {warnings:?}"
        );
        assert!(
            matches!(
                &warnings[0],
                LayoutWarning::XrefTargetNotFound { target, .. }
                if target.as_ref() == "slide-that-does-not-exist"
            ),
            "warning must reference the unknown xref target"
        );
    }

    /// `run_inline_validation` with a known xref target produces no warnings.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_run_inline_validation_known_xref_no_warning() {
        let deck = make_deck_with_titles(&["Introduction", "Methodology"]);
        let nodes = vec![InlineNode::Xref(Arc::from("Methodology"))];
        let slides = vec![make_laid_out_slide_with_text_run(1, nodes)];
        let warnings =
            run_inline_validation(&deck, &slides).expect("run_inline_validation must not error");
        assert!(
            warnings.is_empty(),
            "Xref to known title must not generate a warning"
        );
    }

    /// `run_inline_validation` accumulates warnings across multiple slides.
    ///
    /// Two slides each with one unknown xref → two warnings total.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_05_001_run_inline_validation_accumulates_across_slides() {
        let deck = make_deck_with_titles(&["Slide 0", "Slide 1"]);
        let nodes0 = vec![InlineNode::Xref(Arc::from("missing-from-slide-0"))];
        let nodes1 = vec![InlineNode::Xref(Arc::from("missing-from-slide-1"))];
        let slides = vec![
            make_laid_out_slide_with_text_run(0, nodes0),
            make_laid_out_slide_with_text_run(1, nodes1),
        ];
        let warnings =
            run_inline_validation(&deck, &slides).expect("run_inline_validation must not error");
        assert_eq!(
            warnings.len(),
            2,
            "unknown xrefs on two slides must accumulate two warnings"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // VP-043 — All 12 inline variants → distinct, non-empty (BC-3.05.001 v1.3.3)
    // ─────────────────────────────────────────────────────────────────────────

    /// VP-043 — All 12 inline variants produce distinct kind names (AC-005 /
    /// BC-3.05.001 v1.3.3). F-HIGH-003: comment now cites 12 variants per spec.
    #[test]
    fn test_vp_043_all_12_inline_variant_kind_names_distinct() {
        let variants = all_12_variants();
        // Must be exactly 12 variants per BC-3.05.001 v1.3.3
        assert_eq!(
            variants.len(),
            12,
            "all_12_variants() must yield exactly 12 InlineNode values (BC-3.05.001 v1.3.3)"
        );
        // All kind names must be distinct
        let mut names: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for v in &variants {
            let name = v.kind_name();
            assert!(
                names.insert(name),
                "duplicate kind_name '{name}' — each variant must have a unique discriminant"
            );
        }
        assert_eq!(names.len(), 12, "must have 12 distinct kind names");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // VP-044 — String-prefix bold NOT applied (canonical CLAUDE.md forbidden pattern)
    // ─────────────────────────────────────────────────────────────────────────

    /// VP-044 — A `Plain` node containing `"**bold**"` does NOT produce bold markup;
    /// it is treated as literal text (no string-prefix bold pattern per CLAUDE.md).
    ///
    /// The correct way to produce bold is `InlineNode::Bold(vec![InlineNode::Plain(...)])`.
    /// A `Plain` node with markdown-style prefix MUST be passed through unchanged.
    #[test]
    fn test_vp_044_string_prefix_bold_not_applied() {
        // A Plain node containing markdown-style "**bold**" must not be mutated
        // into a Bold variant by validate_inline_nodes.
        let nodes = vec![InlineNode::Plain(Arc::from("**bold**"))];
        let known = empty_titles();
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &known, 0, &mut warnings).expect("must not error");
        // The node must still be Plain (not Bold) — no string-prefix transformation.
        assert!(
            matches!(&nodes[0], InlineNode::Plain(s) if s.as_ref() == "**bold**"),
            "validate_inline_nodes must NOT transform Plain('**bold**') into Bold; \
             string-prefix bold is a forbidden pattern (CLAUDE.md R1)"
        );
        assert!(
            warnings.is_empty(),
            "markdown-prefix string must produce no warnings"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // VP-045 — Depth 65 → InlineDepthExceeded (Kani candidate / F-MED-006)
    // ─────────────────────────────────────────────────────────────────────────

    /// VP-045 / AC-BC-A7 — A nesting depth of `MAX_INLINE_DEPTH + 1` (65) triggers
    /// `LayoutError::InlineDepthExceeded` (BC-3.05.001 E-LAY-005 / F-MED-006).
    ///
    /// This test builds a chain of 65 nested `Bold` nodes and verifies that
    /// `validate_inline_nodes` returns the hard error rather than recursing.
    #[test]
    fn test_vp_045_depth_65_returns_inline_depth_exceeded() {
        // Build a chain of MAX_INLINE_DEPTH + 1 = 65 nested Bold nodes.
        let mut node = InlineNode::Plain(Arc::from("leaf"));
        for _ in 0..=MAX_INLINE_DEPTH {
            node = InlineNode::Bold(vec![node]);
        }
        let nodes = vec![node];
        let known = empty_titles();
        let mut warnings = vec![];
        let result = validate_inline_nodes(&nodes, &known, 3, &mut warnings);
        assert!(result.is_err(), "depth-65 nesting must return Err");
        match result.unwrap_err() {
            crate::error::LayoutError::InlineDepthExceeded {
                source_slide_index,
                depth,
                max,
            } => {
                assert_eq!(source_slide_index, 3, "slide index must be 3");
                assert_eq!(
                    depth,
                    65,
                    "reported depth must be exactly 65 (BC literal: first rejected tree depth)"
                );
                assert_eq!(max, MAX_INLINE_DEPTH, "max must equal MAX_INLINE_DEPTH");
            },
            other => panic!("expected InlineDepthExceeded, got: {other:?}"),
        }
    }

    /// VP-045 / AC-BC-A7 / F-P11-MED-001 — Depth exactly `MAX_INLINE_DEPTH` (64)
    /// MUST be accepted (BC-3.05.001 E-LAY-005).
    ///
    /// Builds exactly 64 nested `Bold` nodes wrapping a single `Plain` leaf.
    /// The `Plain` leaf is reached at recursion depth 64 (== `MAX_INLINE_DEPTH`),
    /// which is the last accepted level under the corrected `depth > MAX_INLINE_DEPTH`
    /// guard. If the old `depth >= MAX_INLINE_DEPTH` guard were in place this test
    /// would fail — it is the load-bearing regression guard for F-P11-MED-001.
    #[test]
    fn test_vp_045_depth_64_accepted() {
        // Build exactly MAX_INLINE_DEPTH = 64 nested Bold nodes wrapping a Plain leaf.
        // The Plain leaf is reached at depth 64 — the boundary that MUST be accepted.
        let mut node = InlineNode::Plain(Arc::from("leaf"));
        for _ in 0..MAX_INLINE_DEPTH {
            node = InlineNode::Bold(vec![node]);
        }
        let nodes = vec![node];
        let known = empty_titles();
        let mut warnings = vec![];
        let result = validate_inline_nodes(&nodes, &known, 0, &mut warnings);
        assert!(
            result.is_ok(),
            "depth-64 (== MAX_INLINE_DEPTH) MUST be accepted per BC + VP-045 + AC-BC-A7; got: {result:?}"
        );
    }

    /// VP-045 — Depth `MAX_INLINE_DEPTH` (64) does NOT trigger depth error.
    ///
    /// The boundary is inclusive: depth 64 (== MAX_INLINE_DEPTH) is accepted;
    /// depth 65 (== MAX_INLINE_DEPTH + 1) is the first rejected level.
    /// This test uses 63 nested Bolds (tree depth 63) to confirm the near-boundary
    /// case also passes — depth-64 acceptance is covered by
    /// `test_vp_045_depth_64_accepted`.
    #[test]
    fn test_vp_045_depth_63_does_not_exceed_limit() {
        // Build exactly MAX_INLINE_DEPTH - 1 = 63 nested Bold nodes.
        let mut node = InlineNode::Plain(Arc::from("leaf"));
        for _ in 0..(MAX_INLINE_DEPTH - 1) {
            node = InlineNode::Bold(vec![node]);
        }
        let nodes = vec![node];
        let known = empty_titles();
        let mut warnings = vec![];
        let result = validate_inline_nodes(&nodes, &known, 0, &mut warnings);
        assert!(
            result.is_ok(),
            "depth-(MAX_INLINE_DEPTH - 1) must NOT trigger InlineDepthExceeded; got: {result:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // VP-046 — Xref inside MathNode NOT flagged (BC-3.05.001 §J / AC-BC-A8)
    // ─────────────────────────────────────────────────────────────────────────

    /// VP-046 / AC-BC-A8 / F-HIGH-002 — A `Math` node is treated as a leaf.
    ///
    /// Even if the LaTeX source of a `MathNode` looks like a slide title string
    /// (e.g., `"Introduction"`), it MUST NOT produce a `XrefTargetNotFound`
    /// warning because the layout pass treats `Math` as an opaque leaf.
    ///
    /// BC-3.05.001 §J: Math mode content is opaque; xref validation MUST NOT
    /// inspect `MathNode` contents.
    #[test]
    fn test_vp_046_xref_inside_math_node_not_flagged() {
        // A MathNode whose latex source equals a non-existent slide title.
        // The layout stage must NOT inspect the latex string for xref targets.
        let nodes = vec![InlineNode::Math(MathNode {
            latex: Arc::from("slide-title-that-does-not-exist"),
            display: false,
            span: SourceSpan::default(),
        })];
        let known = empty_titles(); // "slide-title-that-does-not-exist" is NOT a known title
        let mut warnings = vec![];
        validate_inline_nodes(&nodes, &known, 0, &mut warnings).expect("must not error");
        assert!(
            warnings.is_empty(),
            "Math node content must NOT be inspected for xref targets; \
             got warnings: {warnings:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // VP-047 — 12 variants survive layout pass (TextRun carries tree unchanged)
    // ─────────────────────────────────────────────────────────────────────────

    /// VP-047 — `run_inline_validation` on a `TextRun` containing all 12 inline
    /// variants preserves the node tree verbatim (no mutation by the layout pass).
    #[test]
    fn test_vp_047_all_12_variants_survive_layout_pass() {
        let original_nodes = all_12_variants();
        let nodes_before = original_nodes.clone();
        let deck = make_deck_with_titles(&["introduction"]); // xref target is known
        let slide = make_laid_out_slide_with_text_run(0, original_nodes);
        let slides = vec![slide];
        let warnings =
            run_inline_validation(&deck, &slides).expect("run_inline_validation must not error");
        assert!(
            warnings.is_empty(),
            "all 12 variants with known xref target must produce zero warnings; got: {warnings:?}"
        );
        // Verify nodes in the frame are unchanged by inspecting the frame content.
        match &slides[0].frames[0].content {
            crate::types::FrameContent::TextRun(nodes) => {
                assert_eq!(
                    nodes, &nodes_before,
                    "TextRun node tree must be unchanged after run_inline_validation"
                );
            },
            other => panic!("expected TextRun, got: {other:?}"),
        }
    }
}
