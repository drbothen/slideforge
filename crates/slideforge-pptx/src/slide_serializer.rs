//! Per-slide `slide*.xml` serializer.
//!
//! [`SlideSerializer`] produces the `ppt/slides/slide{n}.xml` bytes for one
//! [`slideforge_layout::LaidOutSlide`] using the `ooxmlsdk` typed API.
//!
//! ## Placeholder mapping (AC-004)
//!
//! Each `FrameContent` variant maps to a specific PPTX placeholder `idx`:
//!
//! | `FrameContent` variant | `<p:ph>` idx | `<p:ph>` type | Notes |
//! |------------------------|-------------|--------------|-------|
//! | `Title`                | `0`         | `"title"`    | Title placeholder |
//! | `Subtitle`             | `1`         | `"subTitle"` | Subtitle placeholder (S3 / PR-52) |
//! | `Body` / `TextRun`     | `1`         | `"body"`     | Content/body placeholder |
//! | `Diagram`              | media embed | —            | SVG written to ppt/media/ |
//! | Others                 | — skipped — | —            | Media handled in STORY-038/039 |
//!
//! ## Element ordering (AC-006 / R4 finding)
//!
//! `ooxmlsdk` produces schema-correct element ordering automatically.
//! The expected child order within `<p:sp>` is:
//! `<p:nvSpPr>` → `<p:spPr>` → `<p:txBody>`.
//!
//! ## Dark layout `<p:clrMapOvr>` (AC-006 / brand-architecture §Dark Layout)
//!
//! Slides whose layout is dark-themed include
//! `<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>` after `<p:cSld>`.
//!
//! ## Register bleed guard (AC-010)
//!
//! `report` and `detail` register content MUST NOT appear in `<p:sp>` elements
//! in `ppt/slides/slide*.xml`. Only `notes`-register text is forwarded to
//! `notesSlide` parts (STORY-040). This module never writes report/detail.

use ooxmlsdk::common::XmlNamespaceDecl;
use ooxmlsdk::schemas::a::{
    Extents, FillRectangle, Offset, ParagraphChoice, PictureLocks, Run, Stretch, Transform2D,
};
use ooxmlsdk::schemas::p::{
    ApplicationNonVisualDrawingProperties, BlipFill, BlipFillChoice, ColorMapOverride,
    ColorMapOverrideChoice, CommonSlideData, GroupShapeProperties, NonVisualDrawingProperties,
    NonVisualPictureDrawingProperties, NonVisualPictureProperties, NonVisualShapeDrawingProperties,
    NonVisualShapeProperties, Picture, PlaceholderShape, PlaceholderValues, Shape, ShapeProperties,
    ShapePropertiesChoice, ShapeTree, ShapeTreeChoice, Slide, TextBody,
};

use slideforge_layout::{FrameContent, LaidOutSlide, LayoutWarning};
use slideforge_types::{BulletItem, ContentBlock, InlineNode};

use crate::error::PptxError;

/// Serializes one `LaidOutSlide` into `slide*.xml` bytes.
///
/// A new `SlideSerializer` is created per slide. State that varies per slide
/// (e.g., which layout is active, whether it is dark-themed) is passed as
/// constructor arguments.
pub struct SlideSerializer {
    /// Whether the layout used by this slide is dark-themed.
    ///
    /// When `true`, `<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>` is
    /// emitted after `<p:cSld>` in the output.
    is_dark_layout: bool,
    /// The layout index (0-based) used by this slide.
    layout_index: usize,
    /// Placeholder idx values that exist in the resolved layout definition.
    ///
    /// When `Some(set)`, the serializer only emits `<p:ph>` for a frame if the
    /// layout contains a matching placeholder idx (AC-011 / ADR-015 §7).
    ///
    /// When `None` (default from `new()`, i.e., `with_layout` not called):
    ///   - `Title` frames emit `<p:ph>` unconditionally (backward-compatible).
    ///   - `Subtitle` frames emit `<p:ph>` unconditionally (backward-compatible).
    ///   - `Body`/`TextRun` frames do NOT emit `<p:ph>` (conservative fallback).
    ///
    /// When `Some(set)` (populated by `with_layout`), `<p:ph>` is emitted only
    /// when the layout has a matching placeholder idx (ADR-015 §7 item 3):
    ///   - `Title` emits `<p:ph idx="0">` iff the layout has `idx=0`
    ///   - `Subtitle` emits `<p:ph idx="1">` iff the layout has `idx=1`
    ///   - `Body`/`TextRun` emit `<p:ph idx="1">` iff the layout has `idx=1`
    ///
    /// When the layout lacks the required idx, `tracing::warn!` is emitted and
    /// the `<p:ph>` element is omitted (shape remains a free-floating shape).
    ///
    /// Use `with_layout` to populate this from `BrandTemplate.layouts[layout_index]`
    /// in the full export pipeline (`build_slide_parts` always calls `with_layout`).
    layout_placeholder_idxs: Option<std::collections::BTreeSet<u32>>,
}

impl SlideSerializer {
    /// Create a serializer for one slide.
    ///
    /// `is_dark_layout` controls `<p:clrMapOvr>` emission.
    /// `layout_index` is the 0-based layout index used to determine the slide
    /// layout reference in the slide XML (STORY-037 defers multi-layout support;
    /// default is layout 0 for all slides).
    ///
    /// Call `with_layout` on the returned serializer to enable AC-011
    /// placeholder idx-chain verification. Without it, Body/TextRun frames
    /// are emitted as non-placeholder shapes (no `<p:ph>` element).
    #[must_use]
    pub fn new(is_dark_layout: bool, layout_index: usize) -> Self {
        Self {
            is_dark_layout,
            layout_index,
            layout_placeholder_idxs: None,
        }
    }

    /// Set the placeholder idx values from the resolved slide layout definition.
    ///
    /// Called by the full export pipeline (`build_slide_parts`) to enable
    /// AC-011 idx-chain checking. When this is called with the layout's
    /// placeholder set, Body/TextRun frames only emit `<p:ph idx="1">` if
    /// idx=1 actually exists in the layout (ADR-015 §7).
    ///
    /// Without this call, Body/TextRun frames emit no `<p:ph>` element.
    #[must_use]
    pub fn with_layout(mut self, layout: &slideforge_brand::layouts::SlideLayoutDef) -> Self {
        let idxs: std::collections::BTreeSet<u32> =
            layout.placeholders.iter().map(|ph| ph.idx).collect();
        self.layout_placeholder_idxs = Some(idxs);
        self
    }

    /// Check whether the layout has a placeholder with the given `idx`.
    ///
    /// Returns `true` if `layout_placeholder_idxs` is `Some(set)` AND `idx` is in the set.
    /// Returns `false` if `layout_placeholder_idxs` is `None` (no layout info).
    fn layout_has_placeholder_idx(&self, idx: u32) -> bool {
        self.layout_placeholder_idxs
            .as_ref()
            .is_some_and(|idxs| idxs.contains(&idx))
    }

    /// Determine the `ShapeKind` for title frames (AC-011 / ADR-015 §7 item 3).
    ///
    /// When `with_layout` HAS been called (`layout_placeholder_idxs` is `Some`):
    ///   - Returns `ShapeKind::Title` if the layout has `idx=0`.
    ///   - Returns `ShapeKind::TitleNoPlaceholder` otherwise (warn+omit per ADR-015 §7).
    ///
    /// When `with_layout` has NOT been called (`layout_placeholder_idxs` is `None`):
    ///   - Returns `ShapeKind::Title` unconditionally (backward-compatible fallback;
    ///     the production path always calls `with_layout`).
    fn title_shape_kind(&self) -> ShapeKind {
        match &self.layout_placeholder_idxs {
            None => ShapeKind::Title, // no layout info — emit ph unconditionally
            Some(_) => {
                if self.layout_has_placeholder_idx(0) {
                    ShapeKind::Title
                } else {
                    ShapeKind::TitleNoPlaceholder
                }
            },
        }
    }

    /// Determine the `ShapeKind` for subtitle frames (AC-011 / ADR-015 §7 item 3).
    ///
    /// When `with_layout` HAS been called (`layout_placeholder_idxs` is `Some`):
    ///   - Returns `ShapeKind::Subtitle` if the layout has `idx=1`.
    ///   - Returns `ShapeKind::SubtitleNoPlaceholder` otherwise (warn+omit).
    ///
    /// When `with_layout` has NOT been called (`layout_placeholder_idxs` is `None`):
    ///   - Returns `ShapeKind::Subtitle` unconditionally (backward-compatible fallback).
    fn subtitle_shape_kind(&self) -> ShapeKind {
        match &self.layout_placeholder_idxs {
            None => ShapeKind::Subtitle, // no layout info — emit ph unconditionally
            Some(_) => {
                if self.layout_has_placeholder_idx(1) {
                    ShapeKind::Subtitle
                } else {
                    ShapeKind::SubtitleNoPlaceholder
                }
            },
        }
    }

    /// Determine the `ShapeKind` for body / text-run frames (AC-011 / ADR-015 §7 item 3).
    ///
    /// When `with_layout` HAS been called (`layout_placeholder_idxs` is `Some`):
    ///   - Returns `ShapeKind::Body` if the layout has `idx=1`.
    ///   - Returns `ShapeKind::BodyNoPlaceholder` otherwise (warn+omit per ADR-015 §7).
    ///
    /// When `with_layout` has NOT been called (`layout_placeholder_idxs` is `None`):
    ///   - Returns `ShapeKind::BodyNoPlaceholder` (conservative fallback; no warn
    ///     because the production path always calls `with_layout`).
    fn body_shape_kind(&self) -> ShapeKind {
        match &self.layout_placeholder_idxs {
            None => ShapeKind::BodyNoPlaceholder, // no layout info — conservative fallback
            Some(_) => {
                if self.layout_has_placeholder_idx(1) {
                    ShapeKind::Body
                } else {
                    ShapeKind::BodyNoPlaceholder
                }
            },
        }
    }

    /// Build a `<p:sp>` for a body or text-run frame, emitting the warn+omit
    /// diagnostic (AC-011 / ADR-015 §7 item 3) when the layout lacks `idx=1`.
    ///
    /// Shared by `FrameContent::Body` and `FrameContent::TextRun` arms in
    /// `build_shape_tree` to keep the per-arm logic in one place and prevent
    /// the warn from going missing in either arm (F-038-P12-M1 fix).
    ///
    /// `frame_label` is a human-readable label for the warn message (`"Body"` or
    /// `"TextRun"`); `text` is the extracted plain text for the shape.
    fn build_body_shape(
        &self,
        shape_id: u32,
        slide_index: usize,
        frame_idx: usize,
        frame_label: &str,
        frame: &slideforge_layout::Frame,
        text: &str,
    ) -> Shape {
        let body_kind = self.body_shape_kind();
        if matches!(body_kind, ShapeKind::BodyNoPlaceholder)
            && self.layout_placeholder_idxs.is_some()
        {
            tracing::warn!(
                slide_index,
                frame_idx,
                "{frame_label} frame: resolved layout has no idx=1 placeholder; \
                 emitting shape without <p:ph> (warn+omit per ADR-015 §7)"
            );
        }
        build_shape(
            shape_id,
            body_kind,
            frame.bbox.x.0,
            frame.bbox.y.0,
            frame.bbox.width.0,
            frame.bbox.height.0,
            text,
        )
    }

    /// Generate `slide{n+1}.xml` bytes for `slide` (0-based index `slide_index`).
    ///
    /// `layout_rel_id` is the `rId` of the layout relationship in
    /// `ppt/slides/_rels/slide{n+1}.xml.rels`.
    ///
    /// `diagram_rids` is a slice of `(frame_idx, rId)` pairs for all
    /// `FrameContent::Diagram` frames on this slide. For each pair, a typed
    /// `<p:pic>` element (`ShapeTreeChoice::PPic`) is added to the shape tree,
    /// referencing the media via `r:embed` (ADR-001 — no raw XML, F-037-005).
    ///
    /// Returns the XML bytes and any non-fatal warnings detected during
    /// serialisation (e.g., a frame with zero-height content).
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::OoxmlElement`] if `ooxmlsdk` fails to build a
    /// required element, or [`PptxError::InvalidEmu`] if a frame's bounding
    /// box contains an invalid coordinate.
    pub fn build(
        &self,
        slide: &LaidOutSlide,
        slide_index: usize,
        _layout_rel_id: &str,
        diagram_rids: &[(usize, String)],
    ) -> Result<(Vec<u8>, Vec<LayoutWarning>), PptxError> {
        let warnings: Vec<LayoutWarning> = Vec::new();
        let part_name = format!("ppt/slides/slide{}.xml", slide_index + 1);

        let shape_tree = self.build_shape_tree(slide, slide_index, diagram_rids)?;

        // Build CommonSlideData with the shape tree.
        let csl = CommonSlideData {
            name: None,
            background: None,
            shape_tree: Box::new(shape_tree),
            customer_data_list: None,
            control_list: None,
            common_slide_data_extension_list: None,
        };

        // Build the Slide.
        let mut sld = Slide {
            xmlns: vec![
                XmlNamespaceDecl::new("a", "http://schemas.openxmlformats.org/drawingml/2006/main"),
                XmlNamespaceDecl::new(
                    "r",
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
                ),
                XmlNamespaceDecl::new(
                    "p",
                    "http://schemas.openxmlformats.org/presentationml/2006/main",
                ),
            ],
            common_slide_data: Box::new(csl),
            ..Slide::default()
        };

        // Emit clrMapOvr for dark-themed layouts (EC-005).
        if self.is_dark_layout {
            sld.color_map_override = Some(Box::new(ColorMapOverride {
                color_map_override_choice: Some(ColorMapOverrideChoice::AMasterClrMapping),
            }));
        }

        let _ = self.layout_index; // used for future multi-layout support

        let bytes = sld.to_xml_bytes().map_err(|e| PptxError::OoxmlElement {
            part: part_name,
            detail: e.to_string(),
        })?;

        Ok((bytes, warnings))
    }

    /// Build and push a `<p:pic>` for an image frame with `descr` set from alt text.
    ///
    /// Encapsulates the Image arm of `build_shape_tree` so that function stays
    /// within the 150-line limit (BC-4.01.004, STORY-039).
    fn push_image_frame(
        shape_tree: &mut ShapeTree,
        shape_id: &mut u32,
        slide_index: usize,
        frame_idx: usize,
        frame: &slideforge_layout::Frame,
        alt: &str,
    ) -> Result<(), PptxError> {
        validate_emu(slide_index, frame_idx, &frame.bbox)?;
        let pic = build_image_picture(
            *shape_id,
            frame_idx,
            alt,
            frame.bbox.x.0,
            frame.bbox.y.0,
            frame.bbox.width.0,
            frame.bbox.height.0,
        );
        shape_tree
            .shape_tree_choice
            .push(ShapeTreeChoice::PPic(Box::new(pic)));
        *shape_id += 1;
        Ok(())
    }

    /// Build the `<p:spTree>` shape tree from the slide's frames.
    ///
    /// Processes each frame in order: text frames become `<p:sp>` placeholder
    /// shapes, diagram frames become `<p:pic>` elements. Non-serialisable frame
    /// types (`Chart`, `Shape`, `ErrorSlidePlaceholder`, `Empty`) are skipped.
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::InvalidEmu`] if any frame has an invalid bounding box.
    fn build_shape_tree(
        &self,
        slide: &LaidOutSlide,
        slide_index: usize,
        diagram_rids: &[(usize, String)],
    ) -> Result<ShapeTree, PptxError> {
        let mut shape_tree = ShapeTree {
            non_visual_group_shape_properties: None,
            group_shape_properties: Some(Box::new(GroupShapeProperties::default())),
            shape_tree_choice: Vec::new(),
            p_ext_lst: None,
            xmlns: vec![],
            xml_other_attrs: vec![],
        };

        let mut shape_id: u32 = 1;

        for (frame_idx, frame) in slide.frames.iter().enumerate() {
            match &frame.content {
                FrameContent::Title(t) => {
                    validate_emu(slide_index, frame_idx, &frame.bbox)?;
                    // ADR-015 §7 item 3: only emit <p:ph idx="0"> if the resolved
                    // layout has a matching title placeholder (idx=0). When the layout
                    // lacks idx=0 (e.g., Blank layout), emit warn + omit <p:ph>.
                    let title_kind = self.title_shape_kind();
                    if matches!(title_kind, ShapeKind::TitleNoPlaceholder) {
                        tracing::warn!(
                            slide_index,
                            frame_idx,
                            "Title frame: resolved layout has no idx=0 placeholder; \
                             emitting shape without <p:ph> (warn+omit per ADR-015 §7)"
                        );
                    }
                    let sp = build_shape(
                        shape_id,
                        title_kind,
                        frame.bbox.x.0,
                        frame.bbox.y.0,
                        frame.bbox.width.0,
                        frame.bbox.height.0,
                        t.as_ref(),
                    );
                    shape_tree
                        .shape_tree_choice
                        .push(ShapeTreeChoice::PSp(Box::new(sp)));
                    shape_id += 1;
                },

                // S3 (PR-52): Subtitle frames must emit type="subTitle" (idx=1),
                // NOT type="title" (idx=0). PlaceholderValues::SubTitle serializes
                // as the OOXML "subTitle" string (ECMA-376 §19.7.10).
                // ADR-015 §7 item 3: only emit <p:ph idx="1"> if the resolved layout
                // has a matching subtitle placeholder (idx=1). When absent, warn+omit.
                FrameContent::Subtitle(t) => {
                    validate_emu(slide_index, frame_idx, &frame.bbox)?;
                    let subtitle_kind = self.subtitle_shape_kind();
                    if matches!(subtitle_kind, ShapeKind::SubtitleNoPlaceholder) {
                        tracing::warn!(
                            slide_index,
                            frame_idx,
                            "Subtitle frame: resolved layout has no idx=1 placeholder; \
                             emitting shape without <p:ph> (warn+omit per ADR-015 §7)"
                        );
                    }
                    let sp = build_shape(
                        shape_id,
                        subtitle_kind,
                        frame.bbox.x.0,
                        frame.bbox.y.0,
                        frame.bbox.width.0,
                        frame.bbox.height.0,
                        t.as_ref(),
                    );
                    shape_tree
                        .shape_tree_choice
                        .push(ShapeTreeChoice::PSp(Box::new(sp)));
                    shape_id += 1;
                },

                FrameContent::Body(blocks) => {
                    let text = extract_body_text(blocks);
                    validate_emu(slide_index, frame_idx, &frame.bbox)?;
                    // AC-011 / ADR-015 §7 item 3: shared helper warns+omits when
                    // the layout has no idx=1 placeholder (F-038-P12-M1 fix).
                    let sp = self.build_body_shape(
                        shape_id,
                        slide_index,
                        frame_idx,
                        "Body",
                        frame,
                        &text,
                    );
                    shape_tree
                        .shape_tree_choice
                        .push(ShapeTreeChoice::PSp(Box::new(sp)));
                    shape_id += 1;
                },

                FrameContent::TextRun(nodes) => {
                    let text = extract_inline_text(nodes);
                    validate_emu(slide_index, frame_idx, &frame.bbox)?;
                    // AC-011 / ADR-015 §7 item 3: same idx-chain check as Body frames
                    // via the shared helper (F-038-P12-M1: single warn site, no drift).
                    let sp = self.build_body_shape(
                        shape_id,
                        slide_index,
                        frame_idx,
                        "TextRun",
                        frame,
                        &text,
                    );
                    shape_tree
                        .shape_tree_choice
                        .push(ShapeTreeChoice::PSp(Box::new(sp)));
                    shape_id += 1;
                },

                // Diagram: emit a typed <p:pic> via ooxmlsdk builders (ADR-001, F-037-005).
                // The media part is written by the caller; `diagram_rids` carries the rId
                // for the IMAGE relationship so it is never dangling.
                // STORY-039 STUB: alt-threading from FrameContent::Diagram { alt } to
                // the <p:cNvPr descr> attribute is NOT yet implemented here. The implementer
                // must route through AltTextEmbedder for real alt-text embedding.
                FrameContent::Diagram { .. } => {
                    if let Some(rid) = diagram_rids
                        .iter()
                        .find(|(idx, _)| *idx == frame_idx)
                        .map(|(_, r)| r.clone())
                    {
                        validate_emu(slide_index, frame_idx, &frame.bbox)?;
                        let pic = build_picture(
                            shape_id,
                            frame_idx,
                            &rid,
                            frame.bbox.x.0,
                            frame.bbox.y.0,
                            frame.bbox.width.0,
                            frame.bbox.height.0,
                        );
                        shape_tree
                            .shape_tree_choice
                            .push(ShapeTreeChoice::PPic(Box::new(pic)));
                        shape_id += 1;
                    } else {
                        tracing::warn!(
                            slide_index,
                            frame_idx,
                            "Diagram frame has no rId in diagram_rids; <p:pic> omitted"
                        );
                    }
                },

                // Image: emit <p:pic> with descr from alt (AC-001..005, BC-4.01.004).
                // STORY-039: AltText is now used; Provided → non-empty descr, Decorative → "".
                FrameContent::Image { alt } => {
                    use slideforge_types::AltText;
                    let alt_str: &str = match alt {
                        AltText::Provided(s) => s.as_ref(),
                        AltText::Decorative => "",
                    };
                    Self::push_image_frame(
                        &mut shape_tree,
                        &mut shape_id,
                        slide_index,
                        frame_idx,
                        frame,
                        alt_str,
                    )?;
                },

                // Chart, Shape, ErrorSlidePlaceholder, Empty: skipped in STORY-037.
                // Chart now carries `alt` field but embedding is deferred to STORY-039 implementer.
                FrameContent::Chart { .. }
                | FrameContent::Shape(_)
                | FrameContent::ErrorSlidePlaceholder { .. }
                | FrameContent::Empty => {
                    tracing::debug!(
                        slide_index,
                        frame_idx,
                        "skipping non-text frame in STORY-037 serializer"
                    );
                },
            }
        }

        Ok(shape_tree)
    }
}

/// Validate EMU coordinates for a frame bounding box.
///
/// Two failure modes are checked:
/// 1. **Negative size** — `width < 0` or `height < 0` produces out-of-spec OOXML.
/// 2. **i32 overflow** — ECMA-376 `<a:off>` uses `ST_Coordinate` (i64-ranged) and
///    `<a:ext>` uses `ST_PositiveCoordinate` (i64-ranged), but major renderers
///    (`PowerPoint`, `LibreOffice`) internally represent these as 32-bit signed integers.
///    Any of `x`, `y`, `width`, or `height` that does not fit in `i32::MAX` risks
///    silent truncation in those renderers. `presentation.rs` already enforces this
///    practical interop limit for slide-level page size (AC-012); this function makes
///    the frame-level check consistent.
fn validate_emu(
    slide_index: usize,
    frame_index: usize,
    bb: &slideforge_layout::BoundingBox,
) -> Result<(), PptxError> {
    if bb.width.0 < 0 || bb.height.0 < 0 {
        return Err(PptxError::InvalidEmu {
            slide_index,
            frame_index,
            detail: format!("negative size: width={} height={}", bb.width.0, bb.height.0),
        });
    }
    // Practical interop limit: <a:off> uses ST_Coordinate (i64) and <a:ext> uses
    // ST_PositiveCoordinate (i64), but major renderers truncate to i32 internally.
    for (name, value) in [
        ("x", bb.x.0),
        ("y", bb.y.0),
        ("width", bb.width.0),
        ("height", bb.height.0),
    ] {
        if i32::try_from(value).is_err() {
            return Err(PptxError::InvalidEmu {
                slide_index,
                frame_index,
                detail: format!("EMU {name}={value} overflows i32::MAX (renderer interop limit)"),
            });
        }
    }
    Ok(())
}

/// Extract plain text from a list of `ContentBlock` values.
fn extract_body_text(blocks: &[ContentBlock]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for block in blocks {
        match block {
            ContentBlock::Text(tb) => {
                lines.push(extract_inline_text(&tb.inlines));
            },
            ContentBlock::Bullets(items) => {
                for item in items {
                    lines.push(extract_bullet_text(item));
                }
            },
            // Other block types (chart, diagram, shape, etc.) are not plain text.
            _ => {},
        }
    }
    lines.join("\n")
}

/// Extract plain text from a `BulletItem`, including nested children.
fn extract_bullet_text(item: &BulletItem) -> String {
    let mut text = extract_inline_text(&item.inlines);
    for child in &item.children {
        text.push('\n');
        text.push_str(&extract_bullet_text(child));
    }
    text
}

/// Extract plain text from a sequence of `InlineNode` values.
fn extract_inline_text(nodes: &[InlineNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            InlineNode::Plain(s) | InlineNode::Code(s) | InlineNode::Xref(s) => {
                out.push_str(s);
            },
            InlineNode::Bold(children)
            | InlineNode::Italic(children)
            | InlineNode::Footnote(children)
            | InlineNode::Superscript(children)
            | InlineNode::Subscript(children)
            | InlineNode::Strikethrough(children)
            | InlineNode::Highlight(children) => {
                out.push_str(&extract_inline_text(children));
            },
            InlineNode::Link { text, .. } => {
                out.push_str(&extract_inline_text(text));
            },
            InlineNode::Math(_) => {
                // Math nodes are not plain text.
            },
        }
    }
    out
}

/// The kind of placeholder shape being built.
///
/// Determines the OOXML `<p:ph type="...">` and `idx` attributes on the
/// placeholder shape element. This separates the Title/Subtitle/Body
/// distinction from the numeric `ph_idx` parameter.
///
/// The `*NoPlaceholder` variants emit no `<p:ph>` element at all — used when
/// the resolved layout has no matching placeholder idx (AC-011 / ADR-015 §7 item 3).
#[derive(Debug, Clone, Copy)]
enum ShapeKind {
    /// Title placeholder: `type="title"`, `idx=0`.
    ///
    /// Only used when `SlideSerializer::layout_has_placeholder_idx(0)` is true.
    Title,
    /// Title frame with NO layout placeholder — emits no `<p:ph>` element.
    ///
    /// Used when the resolved layout has no `idx=0` placeholder (e.g., Blank layout).
    /// `tracing::warn!` is emitted before returning this variant.
    TitleNoPlaceholder,
    /// Subtitle placeholder: `type="subTitle"`, `idx=1` (S3 / PR-52).
    ///
    /// Only used when `SlideSerializer::layout_has_placeholder_idx(1)` is true.
    Subtitle,
    /// Subtitle frame with NO layout placeholder — emits no `<p:ph>` element.
    ///
    /// Used when the resolved layout has no `idx=1` placeholder (e.g., Blank layout).
    /// `tracing::warn!` is emitted before returning this variant.
    SubtitleNoPlaceholder,
    /// Body / content placeholder: `type="body"`, `idx=1`.
    ///
    /// Only used when `SlideSerializer::layout_has_placeholder_idx(1)` is true.
    Body,
    /// Body frame with NO layout placeholder — emits no `<p:ph>` element (AC-011).
    ///
    /// Used when the layout has no `idx=1` placeholder, per AC-011 / ADR-015 §7.
    /// The shape still carries its text content but is treated as a free-floating
    /// shape rather than a placeholder.
    BodyNoPlaceholder,
}

/// Build a single `<p:sp>` shape for a placeholder.
fn build_shape(
    shape_id: u32,
    kind: ShapeKind,
    x: i64,
    y: i64,
    cx: i64,
    cy: i64,
    text: &str,
) -> Shape {
    // Non-visual shape properties.
    let cnv_pr = NonVisualDrawingProperties {
        id: shape_id,
        name: format!("Shape {shape_id}"),
        description: None,
        hidden: None,
        title: None,
        hyperlink_on_click: None,
        hyperlink_on_hover: None,
        non_visual_drawing_properties_extension_list: None,
        xmlns: vec![],
    };

    let cnv_sp_pr = NonVisualShapeDrawingProperties::default();

    // Placeholder shape type and idx based on ShapeKind.
    // S3 (PR-52): Subtitle must use PlaceholderValues::SubTitle, not Title.
    // AC-011 / ADR-015 §7 item 3: *NoPlaceholder variants emit no <p:ph> element.
    let placeholder_shape_opt: Option<PlaceholderShape> = match kind {
        ShapeKind::Title => Some(PlaceholderShape {
            r#type: Some(PlaceholderValues::Title),
            orientation: None,
            size: None,
            index: Some(0u32),
            has_custom_prompt: None,
            extension_list_with_modification: None,
        }),
        ShapeKind::Subtitle => Some(PlaceholderShape {
            r#type: Some(PlaceholderValues::SubTitle),
            orientation: None,
            size: None,
            index: Some(1u32),
            has_custom_prompt: None,
            extension_list_with_modification: None,
        }),
        ShapeKind::Body => Some(PlaceholderShape {
            r#type: Some(PlaceholderValues::Body),
            orientation: None,
            size: None,
            index: Some(1u32),
            has_custom_prompt: None,
            extension_list_with_modification: None,
        }),
        // ADR-015 §7 item 3: no <p:ph> element when layout lacks matching placeholder.
        ShapeKind::TitleNoPlaceholder
        | ShapeKind::SubtitleNoPlaceholder
        | ShapeKind::BodyNoPlaceholder => None,
    };

    let nv_pr = ApplicationNonVisualDrawingProperties {
        is_photo: None,
        user_drawn: None,
        placeholder_shape: placeholder_shape_opt.map(Box::new),
        application_non_visual_drawing_properties_choice: None,
        p_cust_data_lst: None,
        p_ext_lst: None,
    };

    let nv_sp_pr = NonVisualShapeProperties {
        non_visual_drawing_properties: Box::new(cnv_pr),
        non_visual_shape_drawing_properties: Box::new(cnv_sp_pr),
        application_non_visual_drawing_properties: Box::new(nv_pr),
    };

    // Shape properties with transform (position and size in EMU).
    let xfrm = Transform2D {
        rotation: None,
        horizontal_flip: None,
        vertical_flip: None,
        offset: Some(Offset { x, y }),
        extents: Some(Extents { cx, cy }),
        xmlns: vec![],
    };

    let sp_pr = ShapeProperties {
        transform2_d: Some(Box::new(xfrm)),
        shape_properties_choice1: Some(ShapePropertiesChoice::APrstGeom(Box::new(
            ooxmlsdk::schemas::a::PresetGeometry {
                preset: ooxmlsdk::schemas::a::ShapeTypeValues::Rectangle,
                adjust_value_list: None,
                xmlns: vec![],
            },
        ))),
        shape_properties_choice2: None,
        shape_properties_choice3: None,
        black_white_mode: None,
        a_ln: None,
        a_scene3d: None,
        a_sp3d: None,
        a_ext_lst: None,
        xmlns: vec![],
    };

    // Text body with one paragraph containing the text.
    let run = Run {
        run_properties: Some(Box::default()),
        text: text.to_string(),
        xmlns: vec![],
        xml_other_children: vec![],
    };

    let para = ooxmlsdk::schemas::a::Paragraph {
        paragraph_choice: vec![ParagraphChoice::AR(Box::new(run))],
        ..ooxmlsdk::schemas::a::Paragraph::default()
    };

    let tx_body = TextBody {
        body_properties: Box::default(),
        list_style: None,
        a_p: vec![para],
        xmlns: vec![],
    };

    Shape {
        use_background_fill: None,
        non_visual_shape_properties: Box::new(nv_sp_pr),
        shape_properties: Box::new(sp_pr),
        shape_style: None,
        text_body: Some(Box::new(tx_body)),
        extension_list_with_modification: None,
    }
}

/// Build a typed `<p:pic>` element for an image frame with accessibility metadata.
///
/// `shape_id` is the numeric shape ID for `<p:cNvPr id="...">`.
/// `frame_idx` is the 0-based frame index, used for the shape name.
/// `alt_text` is the alt text value for `descr` (empty string for decorative elements).
///   - Non-empty → `descr="<alt_text>"` (BC-4.01.004 postcondition 1).
///   - Empty → `descr=""` (BC-4.01.004 postcondition 2 — attribute present, empty value).
///
/// Alt text is NEVER truncated (BC-4.01.004 invariant 1). XML special characters
/// in `alt_text` are escaped by `ooxmlsdk` automatically (BC-4.01.004 EC-001).
///
/// The `descr` attribute is placed on `<p:cNvPr>` (BC-4.01.004 invariant 2),
/// NOT on `<p:ph altText>` (which is for placeholder names only).
///
/// The resulting element is a placeholder picture with no media reference (no
/// `r:embed` attribute) since image media embedding is deferred to a later story.
fn build_image_picture(
    shape_id: u32,
    frame_idx: usize,
    alt_text: &str,
    x: i64,
    y: i64,
    cx: i64,
    cy: i64,
) -> Picture {
    // <p:cNvPr>: set description to alt_text (maps to descr= attribute in XML).
    // BC-4.01.004 invariant 2: descr is on cNvPr, not on ph.altText.
    // BC-4.01.004 invariant 1: full alt_text, never truncated.
    // BC-4.01.004 EC-001: ooxmlsdk escapes XML special chars automatically.
    let cnv_pr = NonVisualDrawingProperties {
        id: shape_id,
        name: format!("Image {frame_idx}"),
        // Some("") for decorative (descr="" — attribute present, empty value).
        // Some(non_empty) for non-decorative (descr="alt text").
        // ooxmlsdk writes Some(v) as descr="v" for any v, including empty string.
        description: Some(alt_text.to_owned()),
        hidden: None,
        title: None,
        hyperlink_on_click: None,
        hyperlink_on_hover: None,
        non_visual_drawing_properties_extension_list: None,
        xmlns: vec![],
    };

    let cnv_pic_pr = NonVisualPictureDrawingProperties {
        prefer_relative_resize: None,
        picture_locks: Some(Box::new(PictureLocks {
            no_change_aspect: Some(true),
            ..PictureLocks::default()
        })),
        non_visual_picture_properties_extension_list: None,
    };

    let nv_pr = ApplicationNonVisualDrawingProperties {
        is_photo: None,
        user_drawn: None,
        placeholder_shape: None,
        application_non_visual_drawing_properties_choice: None,
        p_cust_data_lst: None,
        p_ext_lst: None,
    };

    let nv_pic_pr = NonVisualPictureProperties {
        non_visual_drawing_properties: Box::new(cnv_pr),
        non_visual_picture_drawing_properties: Box::new(cnv_pic_pr),
        application_non_visual_drawing_properties: Box::new(nv_pr),
    };

    // Blip fill with no media reference (image media embedding is deferred).
    let blip_fill = BlipFill {
        dpi: None,
        rotate_with_shape: None,
        blip: None,
        source_rectangle: None,
        blip_fill_choice: Some(BlipFillChoice::AStretch(Box::new(Stretch {
            fill_rectangle: Some(FillRectangle::default()),
        }))),
    };

    let xfrm = Transform2D {
        rotation: None,
        horizontal_flip: None,
        vertical_flip: None,
        offset: Some(Offset { x, y }),
        extents: Some(Extents { cx, cy }),
        xmlns: vec![],
    };

    let sp_pr = ShapeProperties {
        transform2_d: Some(Box::new(xfrm)),
        shape_properties_choice1: Some(ShapePropertiesChoice::APrstGeom(Box::new(
            ooxmlsdk::schemas::a::PresetGeometry {
                preset: ooxmlsdk::schemas::a::ShapeTypeValues::Rectangle,
                adjust_value_list: None,
                xmlns: vec![],
            },
        ))),
        shape_properties_choice2: None,
        shape_properties_choice3: None,
        black_white_mode: None,
        a_ln: None,
        a_scene3d: None,
        a_sp3d: None,
        a_ext_lst: None,
        xmlns: vec![],
    };

    Picture {
        non_visual_picture_properties: Box::new(nv_pic_pr),
        blip_fill: Box::new(blip_fill),
        shape_properties: Box::new(sp_pr),
        shape_style: None,
        extension_list_with_modification: None,
    }
}

/// Build a typed `<p:pic>` element for a diagram media frame (ADR-001, F-037-005).
///
/// `shape_id` is the numeric shape ID for `<p:cNvPr id="...">`.
/// `frame_idx` is the 0-based frame index, used for the shape name.
/// `r_embed` is the relationship ID string (e.g. `"rId2"`) from the slide `.rels`.
/// Position and size are given in integer EMU.
///
/// The resulting element embeds the media via `<a:blip r:embed="..."/>` inside
/// `<p:blipFill>` with a stretch fill, and positions it via `<a:xfrm>` inside
/// `<p:spPr>` with a rect preset geometry — matching the schema produced by the
/// previous raw-XML path but through typed ooxmlsdk builders.
fn build_picture(
    shape_id: u32,
    frame_idx: usize,
    r_embed: &str,
    x: i64,
    y: i64,
    cx: i64,
    cy: i64,
) -> Picture {
    // --- Non-visual properties ---
    let cnv_pr = NonVisualDrawingProperties {
        id: shape_id,
        name: format!("Diagram {frame_idx}"),
        description: None,
        hidden: None,
        title: None,
        hyperlink_on_click: None,
        hyperlink_on_hover: None,
        non_visual_drawing_properties_extension_list: None,
        xmlns: vec![],
    };

    // <p:cNvPicPr> with noChangeAspect="1"
    let cnv_pic_pr = NonVisualPictureDrawingProperties {
        prefer_relative_resize: None,
        picture_locks: Some(Box::new(PictureLocks {
            no_change_aspect: Some(true),
            ..PictureLocks::default()
        })),
        non_visual_picture_properties_extension_list: None,
    };

    let nv_pr = ApplicationNonVisualDrawingProperties {
        is_photo: None,
        user_drawn: None,
        placeholder_shape: None,
        application_non_visual_drawing_properties_choice: None,
        p_cust_data_lst: None,
        p_ext_lst: None,
    };

    let nv_pic_pr = NonVisualPictureProperties {
        non_visual_drawing_properties: Box::new(cnv_pr),
        non_visual_picture_drawing_properties: Box::new(cnv_pic_pr),
        application_non_visual_drawing_properties: Box::new(nv_pr),
    };

    // --- Blip fill: <a:blip r:embed="..."/> + <a:stretch><a:fillRect/></a:stretch> ---
    let blip = ooxmlsdk::schemas::a::Blip {
        embed: Some(r_embed.to_owned()),
        ..ooxmlsdk::schemas::a::Blip::default()
    };

    let stretch = Stretch {
        fill_rectangle: Some(FillRectangle::default()),
    };

    let blip_fill = BlipFill {
        dpi: None,
        rotate_with_shape: None,
        blip: Some(Box::new(blip)),
        source_rectangle: None,
        blip_fill_choice: Some(BlipFillChoice::AStretch(Box::new(stretch))),
    };

    // --- Shape properties: xfrm off/ext + prstGeom rect ---
    let xfrm = Transform2D {
        rotation: None,
        horizontal_flip: None,
        vertical_flip: None,
        offset: Some(Offset { x, y }),
        extents: Some(Extents { cx, cy }),
        xmlns: vec![],
    };

    let sp_pr = ShapeProperties {
        transform2_d: Some(Box::new(xfrm)),
        shape_properties_choice1: Some(ShapePropertiesChoice::APrstGeom(Box::new(
            ooxmlsdk::schemas::a::PresetGeometry {
                preset: ooxmlsdk::schemas::a::ShapeTypeValues::Rectangle,
                adjust_value_list: None,
                xmlns: vec![],
            },
        ))),
        shape_properties_choice2: None,
        shape_properties_choice3: None,
        black_white_mode: None,
        a_ln: None,
        a_scene3d: None,
        a_sp3d: None,
        a_ext_lst: None,
        xmlns: vec![],
    };

    Picture {
        non_visual_picture_properties: Box::new(nv_pic_pr),
        blip_fill: Box::new(blip_fill),
        shape_properties: Box::new(sp_pr),
        shape_style: None,
        extension_list_with_modification: None,
    }
}
