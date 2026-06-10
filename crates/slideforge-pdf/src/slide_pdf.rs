//! PDF inline markup rendering for slide body content.
//!
//! # STORY-081 AC-004 — Font face dispatch for inline nodes
//!
//! In krilla `=0.6.0` (pinned in `slideforge-pdf/Cargo.toml`, NOT workspace,
//! per export-architecture v1.2), bold and italic text require loading a
//! **separate font face** via `Font::new(data, index)` — there is NO
//! `set_bold()` or `set_italic()` toggle on a running text span.
//!
//! This module provides:
//! - [`FontFaceKind`]: an enum describing which font face to use for a given
//!   inline node (`Regular`, `Bold`, `Italic`, `BoldItalic`, `Mono`).
//! - [`select_font_face`]: maps an [`InlineNode`] reference to the appropriate
//!   [`FontFaceKind`] for font dispatch.
//! - [`KrillaTextSpan`]: a lightweight struct representing one text span with
//!   its font face kind, text content, and a non-zero `y_offset_units` flag for
//!   Superscript/Subscript (used by `slide_to_krilla_runs` to carry run
//!   information before the font data is resolved).
//! - [`slide_to_krilla_runs`]: converts a `&[InlineNode]` slice into a
//!   `Vec<KrillaTextSpan>` for consumption by the PDF text-placement code.
//!
//! ## Superscript / Subscript rendering (ADR-023 amendment, 2026-06-09)
//!
//! Superscript and Subscript produce spans with non-zero `y_offset_units`.
//! The `draw_inline_spans_at_y` function in `exporter.rs` treats this as a
//! SIGNAL and applies the blessed point-space mechanism:
//!
//! - **Font size:** `font_size * exporter::SUPER_SUB_SCALE` (= 0.583 ×)
//!   — the standard typographic super/subscript ratio (Unicode TR #25).
//! - **Baseline shift (Superscript, `y_offset_units` > 0):**
//!   `baseline_y - (font_size * exporter::SUPER_RISE_FRACTION)` (= −⅓ em, raised).
//! - **Baseline shift (Subscript, `y_offset_units` < 0):**
//!   `baseline_y + (font_size * exporter::SUB_DROP_FRACTION)` (= +⅓ em, lowered).
//!
//! Both shift and scale are computed against the PARENT font size — NOT the
//! reduced size — so the shift is proportional to the reading context.
//!
//! `Surface::draw_glyphs` / `KrillaGlyph.y_offset` are NOT used: `draw_glyphs`
//! requires glyph IDs from a shaping step that krilla does not expose publicly
//! (`naive_shape` is `pub(crate)` in krilla 0.6.0). The entire mechanism is a
//! pure `surface.draw_text()` call with two adjusted scalar parameters.
//!
//! ## BC-3.05.001 PC-5 (slide-level inline markup in PDF)
//!
//! Bold text in slide bullets/body is rendered with a distinct font face
//! (`ResolvedFontSet.bold`) in PDF output — not as plain text. The font face
//! dispatch is performed by `font_for_span` in `exporter.rs`, which selects
//! the appropriate `krilla::text::Font` from a `ResolvedFontSet` based on
//! each span's `FontFaceKind` (ADR-023, STORY-081 C1-NEW fix, Pass-3).

use slideforge_types::InlineNode;

// ─── FontFaceKind ─────────────────────────────────────────────────────────────

/// Identifies which physical font face should be used to render an inline span.
///
/// In krilla `=0.6.0`, each face must be loaded separately via
/// `Font::new(data, index)`. This enum decouples the "which face?" decision
/// from the "load the bytes" concern so that `select_font_face` is pure and
/// can be tested without a krilla context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontFaceKind {
    /// Regular (upright, weight-400) font face.
    Regular,
    /// Bold (weight-700) font face — requires a separate loaded `Font`.
    Bold,
    /// Italic (oblique) font face — requires a separate loaded `Font`.
    Italic,
    /// Bold + Italic combined face (weight-700 + oblique).
    BoldItalic,
    /// Monospace font face (used for `InlineNode::Code`).
    Mono,
}

// ─── KrillaTextSpan ───────────────────────────────────────────────────────────

/// A text span with font face selection and text content.
///
/// Produced by [`slide_to_krilla_runs`]; consumed by the PDF text-placement
/// code in `exporter.rs` that resolves each [`FontFaceKind`] to a loaded
/// `krilla::text::Font` instance and lays out the text on the slide canvas.
///
/// ## Superscript / Subscript signal (ADR-023 amendment, 2026-06-09)
///
/// The `y_offset_units` field is non-zero for Superscript/Subscript nodes.
/// `draw_inline_spans_at_y` in `exporter.rs` uses this as a SIGNAL to apply
/// the blessed point-space mechanism:
/// - Rendered font size: `font_size * SUPER_SUB_SCALE` (0.583 × parent size).
/// - Baseline shift: `±(font_size * SUPER_RISE_FRACTION/SUB_DROP_FRACTION)` in
///   point-space, computed against the PARENT font size.
///
/// `Surface::draw_glyphs` / `KrillaGlyph.y_offset` are NOT used — `draw_glyphs`
/// requires glyph IDs from a shaping step that krilla 0.6.0 does not expose
/// publicly (`naive_shape` is `pub(crate)`).
#[derive(Debug, Clone, PartialEq)]
pub struct KrillaTextSpan {
    /// Which font face to use when loading the font via `Font::new(data, index)`.
    pub face: FontFaceKind,
    /// The text content of this span.
    pub text: std::sync::Arc<str>,
    /// Signal flag for super/subscript baseline positioning (non-zero = shifted).
    ///
    /// - Positive (`SUPER_OFFSET_UNITS`): Superscript — raise baseline by
    ///   `font_size * exporter::SUPER_RISE_FRACTION` points (Y-down: subtract).
    /// - Negative (`SUB_OFFSET_UNITS`): Subscript — lower baseline by
    ///   `font_size * exporter::SUB_DROP_FRACTION` points (Y-down: add).
    /// - Zero: normal inline span; baseline is unchanged.
    ///
    /// The UNIT value (`SUPER_OFFSET_UNITS` = 333) is not used directly in any
    /// arithmetic by the draw path — only the sign is tested. The constants are
    /// retained for backward compatibility with existing tests.
    pub y_offset_units: i32,
}

// ─── select_font_face ─────────────────────────────────────────────────────────

/// Map an [`InlineNode`] to the [`FontFaceKind`] that should render it.
///
/// This is a **pure**, krilla-free function — it contains no I/O, no font
/// loading, and no side effects. This makes it directly testable without a
/// krilla context (AC-004 unit tests).
///
/// ## Mapping table
///
/// | `InlineNode` variant | `FontFaceKind` |
/// |--------------------|-------------|
/// | `Plain` | `Regular` |
/// | `Bold` | `Bold` |
/// | `Italic` | `Italic` |
/// | `Code` | `Mono` |
/// | `Link { .. }` | `Regular` |
/// | `Superscript` | `Regular` + `y_offset_units = SUPER_OFFSET_UNITS` (draw at raised baseline, reduced size) |
/// | `Subscript` | `Regular` + `y_offset_units = SUB_OFFSET_UNITS` (draw at lowered baseline, reduced size) |
/// | `Strikethrough` | `Regular` |
/// | `Highlight` | `Regular` |
/// | `Footnote` | `Regular` |
/// | `Xref` | `Regular` |
/// | `Math` | `Regular` (math rendering is STORY-009) |
///
/// Bold-inside-Italic nesting produces `BoldItalic` when the caller resolves
/// the face recursively. For the top-level dispatch used by
/// `slide_to_krilla_runs`, only the outermost wrapper's face kind is returned
/// here; nested nodes are resolved by recursive calls in `slide_to_krilla_runs`.
#[must_use]
pub fn select_font_face(node: &InlineNode) -> FontFaceKind {
    match node {
        InlineNode::Bold(_) => FontFaceKind::Bold,
        InlineNode::Italic(_) => FontFaceKind::Italic,
        InlineNode::Code(_) => FontFaceKind::Mono,
        // All other variants use the regular face.
        InlineNode::Plain(_)
        | InlineNode::Link { .. }
        | InlineNode::Superscript(_)
        | InlineNode::Subscript(_)
        | InlineNode::Strikethrough(_)
        | InlineNode::Highlight(_)
        | InlineNode::Footnote(_)
        | InlineNode::Xref(_)
        | InlineNode::Math(_) => FontFaceKind::Regular,
    }
}

// ─── slide_to_krilla_runs ─────────────────────────────────────────────────────

/// Convert a sequence of [`InlineNode`]s into [`KrillaTextSpan`]s for PDF
/// text placement.
///
/// Each span carries the [`FontFaceKind`] selected by [`select_font_face`]
/// and the plain text content extracted from the node. Nested markup nodes
/// (e.g., `Bold([Italic([Plain("bi")])])`) are flattened with the outermost
/// variant's font face — nested face merging (`BoldItalic`) is handled by the
/// recursive helper.
///
/// The caller is responsible for resolving each [`FontFaceKind`] to an actual
/// loaded `krilla::Font` via `Font::new(data, index)` using the brand's font
/// data. There is no `set_bold()` toggle in krilla 0.6.0.
///
/// # Superscript / Subscript (ADR-023 amendment, 2026-06-09)
///
/// For `InlineNode::Superscript`, each leaf span carries
/// `y_offset_units = SUPER_OFFSET_UNITS` (positive — signals raised text).
/// For `InlineNode::Subscript`, `y_offset_units = SUB_OFFSET_UNITS` (negative
/// — signals lowered text).
///
/// The draw path in `exporter::draw_inline_spans_at_y` interprets the NON-ZERO
/// flag and applies the blessed mechanism:
/// - Font size: `font_size * exporter::SUPER_SUB_SCALE` (0.583 ×).
/// - Baseline shift: `±(font_size * SUPER_RISE_FRACTION/SUB_DROP_FRACTION)` in
///   point-space, computed against the parent font size.
///
/// `Surface::draw_glyphs` / `KrillaGlyph.y_offset` are NOT used — `draw_glyphs`
/// requires glyph IDs from `naive_shape` which is `pub(crate)` in krilla 0.6.0
/// and is not available to downstream consumers.
#[must_use]
pub fn slide_to_krilla_runs(nodes: &[InlineNode]) -> Vec<KrillaTextSpan> {
    let mut spans = Vec::new();
    for node in nodes {
        collect_spans(node, FontFaceKind::Regular, 0, &mut spans);
    }
    spans
}

/// Signal constant for superscript spans: non-zero positive value carried by
/// `KrillaTextSpan::y_offset_units` when the node is `InlineNode::Superscript`.
///
/// The draw path (`exporter::draw_inline_spans_at_y`) tests the SIGN to decide
/// whether to raise the baseline (`y_offset_units > 0 → Superscript`). The
/// numeric value is NOT used in offset arithmetic — the actual point-space shift
/// is computed from `font_size * exporter::SUPER_RISE_FRACTION`.
///
/// Value 333 is retained for backward compatibility with existing tests that
/// compare against this constant.
pub const SUPER_OFFSET_UNITS: i32 = 333;

/// Signal constant for subscript spans: non-zero negative value carried by
/// `KrillaTextSpan::y_offset_units` when the node is `InlineNode::Subscript`.
///
/// The draw path (`exporter::draw_inline_spans_at_y`) tests the SIGN to decide
/// whether to lower the baseline (`y_offset_units < 0 → Subscript`). The
/// numeric value is NOT used in offset arithmetic — the actual point-space shift
/// is computed from `font_size * exporter::SUB_DROP_FRACTION`.
///
/// Value −333 is retained for backward compatibility with existing tests.
pub const SUB_OFFSET_UNITS: i32 = -333;

/// Recursive span collector for [`slide_to_krilla_runs`].
///
/// Traverses the inline tree, accumulating leaf `KrillaTextSpan` values into
/// `out`. The `inherited_face` parameter propagates the outermost formatting
/// context; `y_offset` propagates super/subscript baseline shift.
fn collect_spans(
    node: &InlineNode,
    inherited_face: FontFaceKind,
    y_offset: i32,
    out: &mut Vec<KrillaTextSpan>,
) {
    match node {
        InlineNode::Plain(s) | InlineNode::Xref(s) => {
            out.push(KrillaTextSpan {
                face: inherited_face,
                text: s.clone(),
                y_offset_units: y_offset,
            });
        },
        InlineNode::Code(s) => {
            out.push(KrillaTextSpan {
                face: FontFaceKind::Mono,
                text: s.clone(),
                y_offset_units: y_offset,
            });
        },
        InlineNode::Bold(children) => {
            // Resolve face: if already Italic → BoldItalic; otherwise Bold.
            let child_face = match inherited_face {
                FontFaceKind::Italic => FontFaceKind::BoldItalic,
                _ => FontFaceKind::Bold,
            };
            for c in children {
                collect_spans(c, child_face, y_offset, out);
            }
        },
        InlineNode::Italic(children) => {
            // Resolve face: if already Bold → BoldItalic; otherwise Italic.
            let child_face = match inherited_face {
                FontFaceKind::Bold => FontFaceKind::BoldItalic,
                _ => FontFaceKind::Italic,
            };
            for c in children {
                collect_spans(c, child_face, y_offset, out);
            }
        },
        InlineNode::Superscript(children) => {
            for c in children {
                collect_spans(c, inherited_face, y_offset + SUPER_OFFSET_UNITS, out);
            }
        },
        InlineNode::Subscript(children) => {
            for c in children {
                collect_spans(c, inherited_face, y_offset + SUB_OFFSET_UNITS, out);
            }
        },
        InlineNode::Strikethrough(children)
        | InlineNode::Highlight(children)
        | InlineNode::Footnote(children) => {
            for c in children {
                collect_spans(c, inherited_face, y_offset, out);
            }
        },
        InlineNode::Link { text, .. } => {
            for c in text {
                collect_spans(c, inherited_face, y_offset, out);
            }
        },
        InlineNode::Math(_) => {
            // Math rendering is STORY-009; skipped here.
        },
    }
}

// ─── extract_all_inline_text ──────────────────────────────────────────────────

/// Extract all plain text from a sequence of [`InlineNode`]s, including text
/// nested inside formatting variants (Bold, Italic, Code, Link, etc.).
///
/// This is used by:
/// - `exporter.rs` draw functions (replacing the former `extract_inline_text`
///   which only handled Plain, Bold, Italic, and skipped Code/Link/etc.)
/// - `tag_engine.rs` `/ActualText` computation for Body `ContentBlock::Text`
///   and `FrameContent::TextRun` elements (STORY-081 C1 fix: the former
///   `/ActualText` extractor only matched `InlineNode::Plain`, so Bold text
///   like `**bold text**` produced an empty `/ActualText` and was therefore
///   absent from the uncompressed structure dictionary — making the text
///   unfindable in raw PDF bytes for integration test verification).
///
/// Delegates to [`slide_to_krilla_runs`] and concatenates the text fields,
/// so the extraction logic is defined once and both paths stay in sync.
#[must_use]
pub fn extract_all_inline_text(inlines: &[InlineNode]) -> String {
    slide_to_krilla_runs(inlines)
        .into_iter()
        .map(|span| span.text.as_ref().to_owned())
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use slideforge_types::InlineNode;

    #[test]
    fn test_select_font_face_bold() {
        let node = InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold"))]);
        assert_eq!(select_font_face(&node), FontFaceKind::Bold);
    }

    #[test]
    fn test_select_font_face_italic() {
        let node = InlineNode::Italic(vec![InlineNode::Plain(Arc::from("italic"))]);
        assert_eq!(select_font_face(&node), FontFaceKind::Italic);
    }

    #[test]
    fn test_select_font_face_code() {
        let node = InlineNode::Code(Arc::from("fn foo() {}"));
        assert_eq!(select_font_face(&node), FontFaceKind::Mono);
    }

    #[test]
    fn test_select_font_face_plain() {
        let node = InlineNode::Plain(Arc::from("hello"));
        assert_eq!(select_font_face(&node), FontFaceKind::Regular);
    }

    #[test]
    fn test_slide_to_krilla_runs_bold() {
        let nodes = vec![InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold"))])];
        let spans = slide_to_krilla_runs(&nodes);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].face, FontFaceKind::Bold);
        assert_eq!(spans[0].text.as_ref(), "bold");
    }

    #[test]
    fn test_slide_to_krilla_runs_superscript_y_offset() {
        let nodes = vec![InlineNode::Superscript(vec![InlineNode::Plain(Arc::from(
            "sup",
        ))])];
        let spans = slide_to_krilla_runs(&nodes);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].y_offset_units, SUPER_OFFSET_UNITS);
    }
}
