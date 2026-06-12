//! Per-slide `slide*.xml` serializer.
//!
//! [`SlideSerializer`] produces the `ppt/slides/slide{n}.xml` bytes for one
//! [`slideforge_layout::LaidOutSlide`] using the `ooxmlsdk` typed API.
//!
//! ## Placeholder mapping (AC-004)
//!
//! Each `FrameContent` variant maps to a specific PPTX placeholder `idx` or
//! produces a `<p:pic>` element with an accessibility `descr` attribute:
//!
//! | `FrameContent` variant | Output element | `<p:ph>` idx / Notes |
//! |------------------------|---------------|----------------------|
//! | `Title`                | `<p:sp>`      | `idx=0`, `type="title"` — Title placeholder |
//! | `Subtitle`             | `<p:sp>`      | `idx=1`, `type="subTitle"` — Subtitle placeholder (S3 / PR-52) |
//! | `Body` / `TextRun`     | `<p:sp>`      | `idx=1`, `type="body"` — Content/body placeholder |
//! | `Diagram`              | `<p:pic>`     | SVG written to `ppt/media/`; `descr` from `AltText` (STORY-039) |
//! | `Image`                | `<p:pic>`     | `descr` from `AltText` via `AltTextEmbedder` (STORY-039) |
//! | `Chart`                | `<p:pic>`     | Placeholder `<p:pic>` with `descr` from `AltText` (STORY-039); SVG embed in future story |
//! | `ColorBar`             | `<p:sp>`      | No `<p:ph>` — free-standing solid-fill rect at proportional width (BC-1.17.002 PC-9) |
//! | `Shape`, `Empty`, `ErrorSlidePlaceholder` | — skipped — | Logged at `debug!` level |
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
    Extents, FillRectangle, GradientFill, GradientFillChoice, GradientStop, GradientStopChoice,
    GradientStopList, Highlight, HighlightChoice, HyperlinkOnClick, LatinFont, LinearGradientFill,
    Offset, ParagraphChoice, PictureLocks, RgbColorModelHex, Run, RunProperties, SolidFill,
    SolidFillChoice, Stretch, TextStrikeValues, Transform2D,
};
use ooxmlsdk::schemas::p::{
    ApplicationNonVisualDrawingProperties, BlipFill, BlipFillChoice, ColorMapOverride,
    ColorMapOverrideChoice, CommonSlideData, GroupShapeProperties, NonVisualDrawingProperties,
    NonVisualGroupShapeDrawingProperties, NonVisualGroupShapeProperties,
    NonVisualPictureDrawingProperties, NonVisualPictureProperties, NonVisualShapeDrawingProperties,
    NonVisualShapeProperties, Picture, PlaceholderShape, PlaceholderValues, Shape, ShapeProperties,
    ShapePropertiesChoice, ShapePropertiesChoice2, ShapeTree, ShapeTreeChoice, Slide, TextBody,
};

use slideforge_layout::{FillSpec, FrameContent, LaidOutSlide, LayoutWarning, ShapeFrame};
use slideforge_plugin_api::inline_formats::{OoxmlRun, render_inline_nodes_to_runs};
use slideforge_types::{AltText, ContentBlock, InlineNode, Rgb, display_text_is_empty};

use crate::error::PptxError;
use crate::link_safety::is_safe_link_scheme;
use crate::xml_escape::strip_xml10_invalid_chars;

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

    /// BCP-47 language tag to set on every `<a:rPr lang="...">` (STORY-096 AC-003).
    ///
    /// Populated by `with_lang` from `deck.metadata.lang` (defaulting to `"en-US"`).
    /// When `None` (default from `new()`), `<a:rPr>` elements are emitted without
    /// a `lang` attribute (pre-STORY-096 behavior preserved for backward compatibility
    /// in any test that does not call `with_lang`).
    lang: Option<std::sync::Arc<str>>,
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
    ///
    /// Call `with_lang` to set the BCP-47 language tag on all `<a:rPr>` elements
    /// (STORY-096 AC-003 / BC-5.01.005 postcondition 1).
    #[must_use]
    pub fn new(is_dark_layout: bool, layout_index: usize) -> Self {
        Self {
            is_dark_layout,
            layout_index,
            layout_placeholder_idxs: None,
            lang: None,
        }
    }

    /// Set the BCP-47 language tag to emit on every `<a:rPr lang="...">` element.
    ///
    /// Called by `build_slide_parts` with `deck.metadata.lang` (defaulting to
    /// `"en-US"` when unset) so every text run in the PPTX carries the correct
    /// language attribute for spell-check and accessibility (STORY-096 AC-003,
    /// BC-5.01.005 postcondition 1).
    #[must_use]
    pub fn with_lang(mut self, lang: &std::sync::Arc<str>) -> Self {
        self.lang = Some(std::sync::Arc::clone(lang));
        self
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

    /// Build a `<p:sp>` for a frame using pre-built inline runs.
    ///
    /// Used for both `FrameContent::TextRun` and `FrameContent::Body` frames that
    /// carry `Vec<InlineNode>` with markup (STORY-081 AC-002 / C3 fix). Uses
    /// `build_shape_with_runs` instead of `build_shape` so that OOXML run
    /// properties (bold, italic, strike) are preserved in the output.
    ///
    /// `frame_label` is a human-readable label for the warn message (`"Body"` or
    /// `"TextRun"`); the warn fires when the layout lacks `idx=1` placeholder
    /// (AC-011 / ADR-015 §7 item 3 / F-038-P12-M1).
    fn build_body_shape_with_runs(
        &self,
        shape_id: u32,
        slide_index: usize,
        frame_idx: usize,
        frame_label: &str,
        frame: &slideforge_layout::Frame,
        runs: Vec<Run>,
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
        build_shape_with_runs(
            shape_id,
            body_kind,
            frame.bbox.x.0,
            frame.bbox.y.0,
            frame.bbox.width.0,
            frame.bbox.height.0,
            runs,
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
    /// `hlink_map` is a slice of `(url, rId)` pairs for External hyperlinks on this
    /// slide. Each entry was produced by `slide_rels.add_external_hyperlink(url)` in
    /// `build_slide_parts` BEFORE this method is called. The rIds are sequential and
    /// stable: the same URL always maps to the same rId within a single export call.
    ///
    /// The caller (`build_slide_parts` in `lib.rs`) constructs this map by:
    /// 1. Calling `collect_hyperlink_urls_from_slide` to get unique safe-scheme URLs.
    /// 2. Calling `slide_rels.add_external_hyperlink(url)` for each, collecting rIds.
    /// 3. Passing the `(url, rId)` pairs here so the serializer can wire
    ///    `<a:hlinkClick r:id="rIdN">` onto the correct runs. EC-004.
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
        hlink_map: &[(String, String)],
    ) -> Result<(Vec<u8>, Vec<LayoutWarning>), PptxError> {
        let warnings: Vec<LayoutWarning> = Vec::new();
        let part_name = format!("ppt/slides/slide{}.xml", slide_index + 1);

        let shape_tree = self.build_shape_tree(slide, slide_index, diagram_rids, hlink_map)?;

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
            "Image",
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
    /// shapes, diagram/image/chart frames become `<p:pic>` elements with `descr`
    /// set from `AltTextEmbedder` (the single authoritative descr-setting path,
    /// ADR-001 / BC-4.01.004). Non-serialisable frame types (`Shape`,
    /// `ErrorSlidePlaceholder`, `Empty`) are skipped.
    ///
    /// ## `AltTextEmbedder` authority (BC-4.01.004 / STORY-039)
    ///
    /// `AltTextEmbedder::decisions_for_slide` is called once per slide to extract
    /// the alt text decisions for all visual frames. The resulting `Vec<(frame_idx,
    /// AltDecision)>` is used to look up the `descr` value for each image/chart/diagram
    /// frame. This ensures there is ONE code path for all `descr` decisions —
    /// `SlideSerializer` never sets `descr` inline; it always routes through the
    /// embedder.
    ///
    /// `hlink_map` — the `(url, rId)` pairs for External hyperlinks on this slide,
    /// pre-registered by the caller in `slide_rels`. Used to resolve rIds for
    /// `InlineNode::Link` nodes in body and subtitle-inlines frames. EC-004.
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::InvalidEmu`] if any frame has an invalid bounding box.
    #[allow(clippy::too_many_lines)]
    fn build_shape_tree(
        &self,
        slide: &LaidOutSlide,
        slide_index: usize,
        diagram_rids: &[(usize, String)],
        hlink_map: &[(String, String)],
    ) -> Result<ShapeTree, PptxError> {
        // AltTextEmbedder is the SINGLE AUTHORITATIVE path for descr decisions.
        // Compute decisions once for all visual frames before the frame loop.
        // This satisfies ADR-001 (no parallel inline descr-setting logic) and
        // ensures AltTextEmbedder is never dead code (F-039-I2 resolution).
        let alt_decisions =
            crate::a11y::AltTextEmbedder::decisions_for_slide(slide).map_err(|e| {
                PptxError::OoxmlElement {
                    part: format!("ppt/slides/slide{}.xml", slide_index + 1),
                    detail: e.to_string(),
                }
            })?;

        // STORY-094 T-005 / BC-4.01.001 AC-003 — CT_GroupShape mandatory first child.
        //
        // REND-003 fix: `<p:nvGrpSpPr>` must be the FIRST child of `<p:spTree>`.
        // The ooxmlsdk `ShapeTree.non_visual_group_shape_properties` field serialises
        // before `group_shape_properties`, satisfying ECMA-376 CT_GroupShape ordering.
        // Structure: <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/>
        // </p:nvGrpSpPr>
        let nvgrpsppr = NonVisualGroupShapeProperties {
            non_visual_drawing_properties: Box::new(NonVisualDrawingProperties {
                id: 1,
                name: String::new(),
                ..NonVisualDrawingProperties::default()
            }),
            non_visual_group_shape_drawing_properties: Box::new(
                NonVisualGroupShapeDrawingProperties::default(),
            ),
            application_non_visual_drawing_properties: Box::new(
                ApplicationNonVisualDrawingProperties::default(),
            ),
        };
        let mut shape_tree = ShapeTree {
            non_visual_group_shape_properties: Some(Box::new(nvgrpsppr)),
            group_shape_properties: Some(Box::new(GroupShapeProperties::default())),
            shape_tree_choice: Vec::new(),
            p_ext_lst: None,
            xmlns: vec![],
            xml_other_attrs: vec![],
        };

        // F-094-P1-005 fix: seed shape_id at 2.
        // The group `<p:nvGrpSpPr>` uses `id=1` (NonVisualDrawingProperties { id: 1 }).
        // If shapes also start at id=1, the first shape and the group share the same
        // cNvPr id — a schema violation (ECMA-376 §19.3.1.13: cNvPr ids must be unique
        // within a presentation part). Starting at 2 avoids the collision.
        let mut shape_id: u32 = 2;
        // STORY-094 T-007 / BC-4.01.001 AC-002 — deduplicate body placeholder (REND-001).
        //
        // ECMA-376 §19.3.1.33: placeholder `idx` must be unique within a slide. Both
        // `FrameContent::Body` and `FrameContent::TextRun` call `body_shape_kind()`
        // which returns `ShapeKind::Body` (idx=1) when the layout has `idx=1`.
        // When a slide has BOTH frame types, two shapes emit `<p:ph idx="1"/>` —
        // a schema violation. This flag tracks whether the body ph descriptor has been
        // emitted so that TextRun frames defer to `ShapeKind::BodyNoPlaceholder` once
        // a Body frame has already claimed the idx=1 slot. Body frames always claim
        // the slot first (they are authoritative carriers of the body placeholder).
        let mut body_ph_emitted: bool = false;

        for (frame_idx, frame) in slide.frames.iter().enumerate() {
            // Look up the alt decision for this frame (visual frames only).
            // For non-visual frames (text, shape, etc.) the lookup returns None.
            let alt_decision_str: Option<String> = alt_decisions
                .iter()
                .find(|(idx, _)| *idx == frame_idx)
                .map(|(_, d)| d.descr_value().to_owned());

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

                // STORY-081 C3: SubtitleInlines — rich subtitle with inline structure.
                // PPTX subtitle placeholders (idx=1, type="subTitle") support multiple runs
                // with <a:rPr> formatting. Unlike the title single-run constraint, subtitle
                // allows inline formatting. We use build_shape_with_runs with ShapeKind::Subtitle.
                FrameContent::SubtitleInlines(nodes) => {
                    validate_emu(slide_index, frame_idx, &frame.bbox)?;
                    let subtitle_kind = self.subtitle_shape_kind();
                    if matches!(subtitle_kind, ShapeKind::SubtitleNoPlaceholder) {
                        tracing::warn!(
                            slide_index,
                            frame_idx,
                            "SubtitleInlines frame: resolved layout has no idx=1 placeholder; \
                             emitting shape without <p:ph> (warn+omit per ADR-015 §7)"
                        );
                    }
                    // ADR-024: use unified engine via nodes_to_body_runs.
                    let runs: Vec<Run> = nodes_to_body_runs(nodes, hlink_map, self.lang.as_deref());
                    let sp = build_shape_with_runs(
                        shape_id,
                        subtitle_kind,
                        frame.bbox.x.0,
                        frame.bbox.y.0,
                        frame.bbox.width.0,
                        frame.bbox.height.0,
                        runs,
                    );
                    shape_tree
                        .shape_tree_choice
                        .push(ShapeTreeChoice::PSp(Box::new(sp)));
                    shape_id += 1;
                },

                FrameContent::Body(blocks) => {
                    validate_emu(slide_index, frame_idx, &frame.bbox)?;
                    // STORY-081 C3 fix: route FrameContent::Body ContentBlock::Text inlines
                    // through the unified ADR-024 engine so bold/italic/strikethrough are
                    // preserved in the PPTX output. The old extract_body_text path (plain
                    // text only) is replaced by extract_body_runs → nodes_to_body_runs
                    // which preserves inline structure. AC-002 / BC-3.05.001 PC-1.
                    // EC-004: pass hlink_map so Link nodes wire <a:hlinkClick>.
                    let runs: Vec<Run> = extract_body_runs(blocks, hlink_map, self.lang.as_deref());
                    // AC-011 / ADR-015 §7 item 3: shared helper warns+omits when
                    // the layout has no idx=1 placeholder (F-038-P12-M1 fix).
                    let sp = self.build_body_shape_with_runs(
                        shape_id,
                        slide_index,
                        frame_idx,
                        "Body",
                        frame,
                        runs,
                    );
                    shape_tree
                        .shape_tree_choice
                        .push(ShapeTreeChoice::PSp(Box::new(sp)));
                    shape_id += 1;
                    // STORY-094 T-007: Body frames are the authoritative carrier of the
                    // body placeholder (idx=1). Mark it claimed so TextRun frames on the
                    // same slide do not emit a second idx=1 descriptor (REND-001 secondary).
                    if matches!(self.body_shape_kind(), ShapeKind::Body) {
                        body_ph_emitted = true;
                    }
                },

                FrameContent::TextRun(nodes) => {
                    validate_emu(slide_index, frame_idx, &frame.bbox)?;
                    // STORY-081 AC-002: route FrameContent::TextRun inlines through the
                    // unified ADR-024 engine (nodes_to_body_runs) to preserve
                    // bold/italic/strikethrough in the PPTX output. The old
                    // extract_inline_text path (plain text only) is no longer used.
                    let runs: Vec<Run> = nodes_to_body_runs(nodes, hlink_map, self.lang.as_deref());
                    // STORY-094 T-007 / BC-4.01.001 AC-002 — deduplicate body ph idx.
                    //
                    // TextRun frames must NOT emit <p:ph idx="1"/> when a Body frame has
                    // already claimed the body placeholder on this slide. Determine the
                    // effective shape kind: use BodyNoPlaceholder if body_ph_emitted,
                    // otherwise delegate to the standard body_shape_kind() check.
                    // This is the root-cause fix: no post-filter, no XML manipulation —
                    // we suppress the ph descriptor at the source (ShapeKind selection).
                    let textrun_kind = if body_ph_emitted {
                        ShapeKind::BodyNoPlaceholder
                    } else {
                        // First TextRun on a slide with no Body frame: may claim idx=1.
                        let kind = self.body_shape_kind();
                        if matches!(kind, ShapeKind::Body) {
                            body_ph_emitted = true;
                        }
                        kind
                    };
                    // AC-011 / ADR-015 §7 item 3: same idx-chain check as Body frames
                    // via the shared helper (F-038-P12-M1: single warn site, no drift).
                    let sp = {
                        if matches!(textrun_kind, ShapeKind::BodyNoPlaceholder)
                            && self.layout_placeholder_idxs.is_some()
                            && body_ph_emitted
                        {
                            // body_ph_emitted means a Body frame already took idx=1;
                            // TextRun does not warn for this case (it is by design).
                        } else if matches!(textrun_kind, ShapeKind::BodyNoPlaceholder)
                            && self.layout_placeholder_idxs.is_some()
                        {
                            tracing::warn!(
                                slide_index,
                                frame_idx,
                                "TextRun frame: resolved layout has no idx=1 placeholder; \
                                 emitting shape without <p:ph> (warn+omit per ADR-015 §7)"
                            );
                        }
                        build_shape_with_runs(
                            shape_id,
                            textrun_kind,
                            frame.bbox.x.0,
                            frame.bbox.y.0,
                            frame.bbox.width.0,
                            frame.bbox.height.0,
                            runs,
                        )
                    };
                    shape_tree
                        .shape_tree_choice
                        .push(ShapeTreeChoice::PSp(Box::new(sp)));
                    shape_id += 1;
                },

                // Diagram: emit a typed <p:pic> via ooxmlsdk builders (ADR-001, F-037-005).
                // The media part is written by the caller; `diagram_rids` carries the rId
                // for the IMAGE relationship so it is never dangling.
                // STORY-039: alt is threaded from FrameContent::Diagram { alt } to
                // the <p:cNvPr descr> attribute via AltTextEmbedder (BC-4.01.004 AC-005).
                FrameContent::Diagram { .. } => {
                    // descr value resolved by AltTextEmbedder::decisions_for_slide above.
                    // SEC-039-002: None is impossible per AltTextEmbedder contract, but if the
                    // coupling ever breaks a silent descr="" would be an unobservable a11y failure.
                    // Emit a structured error log so the regression is visible in traces.
                    let alt_str = if let Some(s) = alt_decision_str.as_deref() {
                        s
                    } else {
                        tracing::error!(
                            slide_index,
                            frame_idx,
                            "AltTextEmbedder returned None for Diagram frame — \
                             AltTextEmbedder::decisions_for_slide coupling invariant violated; \
                             falling back to descr=\"\" (SEC-039-002)"
                        );
                        ""
                    };
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
                            alt_str,
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

                // Image: emit <p:pic> with descr from AltTextEmbedder decision.
                // The descr value was resolved above via decisions_for_slide (single path).
                FrameContent::Image { .. } => {
                    // alt_decision_str is always Some for Image frames (guaranteed by
                    // AltTextEmbedder::decisions_for_slide which covers all Image variants).
                    // SEC-039-002: None is impossible per AltTextEmbedder contract, but if the
                    // coupling ever breaks a silent descr="" would be an unobservable a11y failure.
                    // Emit a structured error log so the regression is visible in traces.
                    let alt_str = if let Some(s) = alt_decision_str.as_deref() {
                        s
                    } else {
                        tracing::error!(
                            slide_index,
                            frame_idx,
                            "AltTextEmbedder returned None for Image frame — \
                             AltTextEmbedder::decisions_for_slide coupling invariant violated; \
                             falling back to descr=\"\" (SEC-039-002)"
                        );
                        ""
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

                // Chart: emit a <p:pic> placeholder with descr from AltTextEmbedder.
                // The chart SVG/media embedding is handled by a later story; this ensures
                // the enclosing shape always carries the accessibility descr attribute.
                // descr value is resolved by AltTextEmbedder::decisions_for_slide above —
                // NOT set inline here (ADR-001: single authoritative path).
                FrameContent::Chart { .. } => {
                    // alt_decision_str is always Some for Chart frames (guaranteed by
                    // AltTextEmbedder::decisions_for_slide which covers all Chart variants).
                    // SEC-039-002: None is impossible per AltTextEmbedder contract, but if the
                    // coupling ever breaks a silent descr="" would be an unobservable a11y failure.
                    // Emit a structured error log so the regression is visible in traces.
                    let alt_str = if let Some(s) = alt_decision_str.as_deref() {
                        s
                    } else {
                        tracing::error!(
                            slide_index,
                            frame_idx,
                            "AltTextEmbedder returned None for Chart frame — \
                             AltTextEmbedder::decisions_for_slide coupling invariant violated; \
                             falling back to descr=\"\" (SEC-039-002)"
                        );
                        ""
                    };
                    validate_emu(slide_index, frame_idx, &frame.bbox)?;
                    let pic = build_image_picture(
                        shape_id,
                        frame_idx,
                        "Chart",
                        alt_str,
                        frame.bbox.x.0,
                        frame.bbox.y.0,
                        frame.bbox.width.0,
                        frame.bbox.height.0,
                    );
                    shape_tree
                        .shape_tree_choice
                        .push(ShapeTreeChoice::PPic(Box::new(pic)));
                    shape_id += 1;
                },

                // Shape frames — STORY-072: FillSpec::Gradient and FillSpec::SolidColor
                // emit <p:sp> with <a:gradFill> or <a:solidFill> respectively.
                // FillSpec::None emits no element (transparent shape placeholder).
                FrameContent::Shape(shape_frame) => {
                    validate_emu(slide_index, frame_idx, &frame.bbox)?;
                    let sp_opt = build_shape_fill_sp(
                        shape_id,
                        frame_idx,
                        frame.bbox.x.0,
                        frame.bbox.y.0,
                        frame.bbox.width.0,
                        frame.bbox.height.0,
                        shape_frame,
                    );
                    if let Some(sp) = sp_opt {
                        shape_tree
                            .shape_tree_choice
                            .push(ShapeTreeChoice::PSp(Box::new(sp)));
                        shape_id += 1;
                    } else {
                        tracing::debug!(
                            slide_index,
                            frame_idx,
                            "no PPTX element emitted for Shape with FillSpec::None"
                        );
                    }
                },

                // ErrorSlidePlaceholder, Empty: no PPTX element emitted.
                FrameContent::ErrorSlidePlaceholder { .. } | FrameContent::Empty => {
                    tracing::debug!(
                        slide_index,
                        frame_idx,
                        "no PPTX element emitted for ErrorSlidePlaceholder/Empty frame"
                    );
                },

                // ColorBar — solid-fill <p:sp> for the progress_bar filled portion.
                //
                // BC-1.17.002 PC-9: the filled bar must appear in the PPTX slide XML
                // as a non-placeholder <p:sp> with <a:solidFill>. The unfilled portion
                // shows through to the slide background (no second shape needed).
                //
                // Element structure:
                //   <p:sp>
                //     <p:nvSpPr>…no <p:ph>…</p:nvSpPr>
                //     <p:spPr>
                //       <a:xfrm><a:off x=… y=…/><a:ext cx=filled_width cy=height/></a:xfrm>
                //       <a:prstGeom prst="rect"/>
                //       <a:solidFill><a:srgbClr val="RRGGBB"/></a:solidFill>
                //     </p:spPr>
                //   </p:sp>
                //
                // Traceability: BC-1.17.002 PC-9; architect pass-2 adjudication §6 (STORY-087).
                FrameContent::ColorBar {
                    filled_width_emu,
                    total_width_emu: _,
                    percent: _,
                    color,
                    alt: _, // STORY-095: alt is for PDF /Figure tagging; PPTX uses shape alt
                } => {
                    validate_emu(slide_index, frame_idx, &frame.bbox)?;
                    let sp = build_color_bar_shape(
                        shape_id,
                        frame.bbox.x.0,
                        frame.bbox.y.0,
                        filled_width_emu.0,
                        frame.bbox.height.0,
                        *color,
                    );
                    shape_tree
                        .shape_tree_choice
                        .push(ShapeTreeChoice::PSp(Box::new(sp)));
                    shape_id += 1;
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

/// Extract structured OOXML runs from a list of `ContentBlock` values.
///
/// STORY-081 C3 fix: replaces [`extract_body_text`] in the `FrameContent::Body`
/// arm of `build_shape_tree`. Converts each `ContentBlock::Text` inline node
/// sequence through [`nodes_to_body_runs`] (unified ADR-024 engine) so that
/// bold/italic/strikethrough formatting AND hyperlink wiring are preserved as
/// OOXML run properties.
///
/// `ContentBlock::Bullets` items are also converted through the same run-property
/// path so bullet inline markup reaches the PPTX output.
///
/// `hlink_map` is the `(url, rId)` lookup table for External hyperlinks on this slide.
/// Passed through to [`nodes_to_body_runs`] for EC-004 body-path hyperlink wiring. EC-004.
fn extract_body_runs(
    blocks: &[ContentBlock],
    hlink_map: &[(String, String)],
    lang: Option<&str>,
) -> Vec<Run> {
    let mut runs: Vec<Run> = Vec::new();
    for block in blocks {
        match block {
            ContentBlock::Text(tb) => {
                // ADR-024: use unified engine via nodes_to_body_runs.
                runs.extend(nodes_to_body_runs(&tb.inlines, hlink_map, lang));
            },
            ContentBlock::Bullets(items) => {
                for item in items {
                    // ADR-024: use unified engine for each item's inlines.
                    runs.extend(nodes_to_body_runs(&item.inlines, hlink_map, lang));
                    // Nested children are rendered after the parent item.
                    for child in &item.children {
                        runs.extend(nodes_to_body_runs(&child.inlines, hlink_map, lang));
                    }
                }
            },
            // Non-text blocks do not contribute runs.
            ContentBlock::Chart(_)
            | ContentBlock::Diagram(_)
            | ContentBlock::Math(_)
            | ContentBlock::Image(_)
            | ContentBlock::Table(_)
            | ContentBlock::Shape(_)
            | ContentBlock::ColorBar(_) => {},
        }
    }
    runs
}

// ─── ADR-024: OoxmlRun → typed ooxmlsdk Run converter ───────────────────────

/// Convert a neutral [`OoxmlRun`] (from [`render_inline_nodes_to_runs`]) to a
/// typed `ooxmlsdk::schemas::a::Run` for schema-correct body-path output.
///
/// This is the body-path half of the ADR-024 unified engine: the engine produces
/// `Vec<OoxmlRun>` format-agnostically; this function converts to the ooxmlsdk
/// typed form for the slide-body serializer, delegating child-ordering enforcement
/// to ooxmlsdk (ADR-024 INV-2 body variant).
///
/// `lang` is a BCP-47 language tag (e.g. `"en-US"`) emitted as the `lang` attribute
/// on every `<a:rPr>` element (AC-003 / STORY-096).  When `Some`, the tag is set on
/// all runs including plain-text runs so spell-check and accessibility tools have a
/// language anchor. When `None` no `lang` attribute is written.
///
/// The text is sanitized via [`strip_xml10_invalid_chars`] for XML 1.0 safety.
fn ooxml_run_to_ooxmlsdk(run: &OoxmlRun, lang: Option<&str>) -> Run {
    let sanitized = strip_xml10_invalid_chars(&run.text);

    let has_properties = run.bold
        || run.italic
        || run.strike
        || run.code_font
        || run.baseline.is_some()
        || run.highlight
        || run.hyperlink_rid.is_some();

    let run_properties = if has_properties {
        let mut rpr = RunProperties {
            language: lang.map(str::to_owned),
            ..RunProperties::default()
        };
        if run.bold {
            rpr.bold = Some(true);
        }
        if run.italic {
            rpr.italic = Some(true);
        }
        if run.strike {
            rpr.strike = Some(TextStrikeValues::SingleStrike);
        }
        if let Some(baseline) = run.baseline {
            rpr.baseline = Some(baseline);
        }
        if run.code_font {
            rpr.a_latin = Some(LatinFont {
                typeface: Some("Courier New".to_owned()),
                ..LatinFont::default()
            });
        }
        if run.highlight {
            let yellow = RgbColorModelHex {
                val: "FFFF00".to_owned(),
                ..RgbColorModelHex::default()
            };
            rpr.a_highlight = Some(Box::new(Highlight {
                highlight_choice: Some(HighlightChoice::ASrgbClr(Box::new(yellow))),
                ..Highlight::default()
            }));
        }
        if let Some(rid) = &run.hyperlink_rid {
            rpr.a_hlink_click = Some(Box::new(HyperlinkOnClick {
                id: Some(rid.clone()),
                ..HyperlinkOnClick::default()
            }));
        }
        Some(Box::new(rpr))
    } else {
        // Plain run — emit rPr with lang so accessibility tools have a language anchor.
        Some(Box::new(RunProperties {
            language: lang.map(str::to_owned),
            ..RunProperties::default()
        }))
    };

    Run {
        run_properties,
        text: sanitized,
        xmlns: vec![],
        xml_other_children: vec![],
    }
}

/// Convert a slice of [`InlineNode`]s to typed ooxmlsdk `Run` objects using the
/// unified [`render_inline_nodes_to_runs`] engine with the slide-body resolver.
///
/// This is the ADR-024 unified body path: we call the single engine and convert
/// each `OoxmlRun` to a typed `Run` via `ooxml_run_to_ooxmlsdk`.
///
/// `hlink_map` is the `(url, rId)` lookup table for External hyperlinks on this slide.
/// The resolver closure is constructed here from `hlink_map` and passed to the engine.
///
/// `lang` is a BCP-47 language tag forwarded to [`ooxml_run_to_ooxmlsdk`] for
/// `<a:rPr lang="...">` emission (AC-003 / STORY-096). Pass `None` to omit.
fn nodes_to_body_runs(
    nodes: &[InlineNode],
    hlink_map: &[(String, String)],
    lang: Option<&str>,
) -> Vec<Run> {
    let resolver = |url: &str| -> Option<String> {
        hlink_map
            .iter()
            .find(|(u, _)| u == url)
            .map(|(_, r)| r.clone())
    };
    match render_inline_nodes_to_runs(nodes, &resolver) {
        Ok(ooxml_runs) => ooxml_runs
            .iter()
            .map(|r| ooxml_run_to_ooxmlsdk(r, lang))
            .collect(),
        Err(e) => {
            tracing::warn!(
                error = %e,
                "render_inline_nodes_to_runs failed for body OOXML; emitting empty runs"
            );
            Vec::new()
        },
    }
}

/// Collect all safe-scheme, non-empty-display-text hyperlink URLs from a slide's
/// body frames, descending through formatting wrappers to reach nested `Link` nodes.
///
/// ## Registration scope (ADV-P14-MED-001)
///
/// After the ADV-P14-MED-001 fix, registration covers both top-level `Link` nodes
/// AND `Link` nodes nested inside formatting wrappers (`Bold`, `Italic`, etc.).
/// The unified [`render_inline_nodes_to_runs`] engine resolves rIds at the `Link` arm
/// regardless of nesting depth. Registration and emission are symmetric:
///
/// ## Reference-set invariant (BC-3.05.001 HI-1)
///
/// Every `<a:hlinkClick>` references a registered External rel (no dangling
/// rId), and every registered External rel is referenced ≥1× by a hlinkClick
/// run (no orphan rel).  A single External rel may back N `<a:hlinkClick>`
/// runs for multi-leaf link display text (e.g., `[click **here** now](url)`
/// → 1 rel / 3 hlinkClick runs, all same rId — BC-3.05.001 EC-012).
/// Count-equality (`external_rel_count == hlinkclick_count ∀`) is **FALSE**
/// and was retired in BC-3.05.001 HI-1.
///
/// ## Orphan-rel invariant (F-085-P6-001 preserved)
///
/// When a `Link` is found by the collector, its display `text` children are NOT
/// recursed into — a Link inside a Link's display text would create an orphan rel
/// (the unified engine emits `<a:hlinkClick>` only for the outer Link, not for Links
/// inside display text). F-040-P3-001 unchanged.
///
/// ## Unsafe-scheme filtering (F-040-P2-001 / CWE-601)
///
/// URLs whose scheme is not in `ALLOWED_LINK_SCHEMES` are skipped.
///
/// ## Empty display text guard (F-P5-001)
///
/// A `Link` whose display text is empty produces no run, so no rel should be registered.
///
/// Returns a deduplicated list (first-occurrence order) of safe-scheme, non-empty-display
/// hyperlink URLs found in the slide's inline content.
pub(crate) fn collect_hyperlink_urls_from_slide(slide: &LaidOutSlide) -> Vec<String> {
    let mut urls: Vec<String> = Vec::new();
    for frame in &slide.frames {
        match &frame.content {
            FrameContent::TextRun(nodes) | FrameContent::SubtitleInlines(nodes) => {
                for node in nodes {
                    collect_link_urls_from_node(node, &mut urls);
                }
            },
            FrameContent::Body(blocks) => {
                for block in blocks {
                    match block {
                        ContentBlock::Text(tb) => {
                            for node in &tb.inlines {
                                collect_link_urls_from_node(node, &mut urls);
                            }
                        },
                        ContentBlock::Bullets(items) => {
                            for item in items {
                                for node in &item.inlines {
                                    collect_link_urls_from_node(node, &mut urls);
                                }
                                for child in &item.children {
                                    for node in &child.inlines {
                                        collect_link_urls_from_node(node, &mut urls);
                                    }
                                }
                            }
                        },
                        _ => {},
                    }
                }
            },
            // Title, Subtitle (plain strings), Diagram, Image, Chart, Shape,
            // Empty, ErrorSlidePlaceholder — no inline nodes to collect from.
            _ => {},
        }
    }
    // Deduplicate while preserving first-occurrence order.
    let mut seen = std::collections::HashSet::new();
    urls.retain(|u| seen.insert(u.clone()));
    urls
}

/// Collect safe-scheme, non-empty-display-text hyperlink URLs from an [`InlineNode`],
/// descending through formatting wrappers to reach nested `Link` nodes.
///
/// ## Wrapper descent (ADV-P14-MED-001)
///
/// `Bold([Link{url}])`, `Italic([Link{url}])`, `Strikethrough([Link{url}])`,
/// `Superscript([Link{url}])`, `Subscript([Link{url}])`, `Highlight([Link{url}])`,
/// and `Footnote([Link{url}])` are all traversed so the inner `Link` URL is
/// registered. The unified [`render_inline_nodes_to_runs`] engine threads the rId
/// through the accumulated `RunProps::hyperlink_rid` field, so the emitted run
/// carries BOTH the formatting property (e.g. `b="1"`) AND `<a:hlinkClick>`.
///
/// ## Orphan-rel invariant (F-085-P6-001)
///
/// When a `Link` is found, its own display `text` children are NOT recursed into
/// — a Link inside another Link's display text produces no second rel (that would
/// be an orphan rel because the dispatcher never emits nested-link-in-display-text
/// as a separate `<a:hlinkClick>`). The F-040-P3-001 invariant is preserved:
///
/// ## Reference-set invariant (BC-3.05.001 HI-1)
///
/// Every `<a:hlinkClick>` references a registered External rel (no dangling
/// rId), and every registered External rel is referenced ≥1× by a hlinkClick
/// run (no orphan rel).  A single External rel may back N `<a:hlinkClick>`
/// runs for multi-leaf link display text.  Count-equality
/// (`external_rel_count == hlinkclick_count ∀`) is **FALSE** and was retired
/// in BC-3.05.001 HI-1.
///
/// ## Safe-scheme + non-empty display text guards
///
/// Same as the top-level case: unsafe-scheme URLs are skipped (CWE-601 /
/// F-040-P2-001); Links with empty display text produce no run and therefore
/// no rel (F-P5-001).
fn collect_link_urls_from_node(node: &InlineNode, urls: &mut Vec<String>) {
    match node {
        InlineNode::Link { url, text } => {
            // Register this Link if safe-scheme and non-empty display text.
            // Do NOT recurse into `text` children — a Link inside a Link's display
            // text would be an orphan rel (dispatcher emits no second hlinkClick).
            if is_safe_link_scheme(url.as_ref()) && !display_text_is_empty(text) {
                urls.push(url.as_ref().to_owned());
            }
        },
        // ADV-P14-MED-001: Descend through formatting wrappers to reach nested Links.
        InlineNode::Bold(children)
        | InlineNode::Italic(children)
        | InlineNode::Strikethrough(children)
        | InlineNode::Superscript(children)
        | InlineNode::Subscript(children)
        | InlineNode::Highlight(children)
        | InlineNode::Footnote(children) => {
            for child in children {
                collect_link_urls_from_node(child, urls);
            }
        },
        // Leaf nodes that cannot contain a Link: no registration, no recursion.
        InlineNode::Plain(_) | InlineNode::Code(_) | InlineNode::Xref(_) | InlineNode::Math(_) => {
        },
    }
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
    // SEC-100 / CWE-116: strip XML-1.0-invalid control characters before placing
    // user-controlled text in Run.text. ooxmlsdk writes Run.text directly as
    // element content; invalid chars (U+0000–U+0008, U+000B, U+000C, U+000E–U+001F,
    // U+FFFE, U+FFFF) would produce malformed XML and enable XML injection.
    // Mirrors the DOCX SEC-002 fix in slideforge-docx/src/document_body.rs.
    let sanitized_text = strip_xml10_invalid_chars(text);
    let run = Run {
        run_properties: Some(Box::default()),
        text: sanitized_text,
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

/// Build a `<p:sp>` shape with pre-built `<a:r>` runs (for inline markup).
///
/// Mirrors `build_shape` but accepts a `Vec<Run>` so that formatted runs
/// produced by the unified [`nodes_to_body_runs`] engine can be embedded directly
/// without flattening to plain text first.
///
/// Used by the `FrameContent::TextRun` and `FrameContent::Body` arms (STORY-081 AC-002).
fn build_shape_with_runs(
    shape_id: u32,
    kind: ShapeKind,
    x: i64,
    y: i64,
    cx: i64,
    cy: i64,
    runs: Vec<Run>,
) -> Shape {
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

    let paragraph_choices: Vec<ParagraphChoice> = runs
        .into_iter()
        .map(|r| ParagraphChoice::AR(Box::new(r)))
        .collect();
    let para = ooxmlsdk::schemas::a::Paragraph {
        paragraph_choice: paragraph_choices,
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

/// Build a solid-fill `<p:sp>` for a `FrameContent::ColorBar` frame.
///
/// Produces a free-standing (non-placeholder) rectangle with `<a:solidFill>`
/// sized to the proportional filled width computed by the layout engine.
/// The unfilled portion of the bar is left to the slide background.
///
/// ## OOXML contract (BC-1.17.002 PC-9)
///
/// - No `<p:ph>` element — this is not a placeholder.
/// - `<a:prstGeom prst="rect"/>` — rect preset geometry.
/// - `<a:solidFill><a:srgbClr val="RRGGBB"/></a:solidFill>` — explicit color.
/// - `cx = filled_width_emu` (may be 0 when `value == 0`; `PowerPoint` renders 0-width rects).
/// - No `<p:txBody>` — the bar is geometry only; text is in the adjacent `ColorLabel` frame.
///
/// ## Arguments
///
/// - `shape_id` — numeric shape ID for `<p:cNvPr id="…">`.
/// - `x, y` — top-left position in EMU (from the bar-background frame bbox).
/// - `cx` — filled width in EMU (`filled_width_emu` from `FrameContent::ColorBar`).
/// - `cy` — bar height in EMU (= bar-background frame bbox height).
/// - `color` — fill color from the `FrameContent::ColorBar.color` field.
fn build_color_bar_shape(shape_id: u32, x: i64, y: i64, cx: i64, cy: i64, color: Rgb) -> Shape {
    let cnv_pr = NonVisualDrawingProperties {
        id: shape_id,
        name: format!("ColorBar {shape_id}"),
        description: None,
        hidden: None,
        title: None,
        hyperlink_on_click: None,
        hyperlink_on_hover: None,
        non_visual_drawing_properties_extension_list: None,
        xmlns: vec![],
    };

    let cnv_sp_pr = NonVisualShapeDrawingProperties::default();

    // No <p:ph> — this is a free-standing fill shape, not a placeholder.
    let nv_pr = ApplicationNonVisualDrawingProperties {
        is_photo: None,
        user_drawn: None,
        placeholder_shape: None,
        application_non_visual_drawing_properties_choice: None,
        p_cust_data_lst: None,
        p_ext_lst: None,
    };

    let nv_sp_pr = NonVisualShapeProperties {
        non_visual_drawing_properties: Box::new(cnv_pr),
        non_visual_shape_drawing_properties: Box::new(cnv_sp_pr),
        application_non_visual_drawing_properties: Box::new(nv_pr),
    };

    // Transform: position at frame origin, width = filled_width_emu.
    let xfrm = Transform2D {
        rotation: None,
        horizontal_flip: None,
        vertical_flip: None,
        offset: Some(Offset { x, y }),
        extents: Some(Extents { cx, cy }),
        xmlns: vec![],
    };

    // Solid fill using the brand-derived (or default) color.
    // The hex value is uppercase 6 hex digits per ECMA-376 ST_HexColorRGB.
    let hex_val = format!("{:02X}{:02X}{:02X}", color.r, color.g, color.b);
    let srgb = RgbColorModelHex {
        val: hex_val,
        legacy_spreadsheet_color_index: None,
        rgb_color_model_hex_choice: vec![],
        xmlns: vec![],
        xml_other_attrs: vec![],
    };
    let solid_fill = SolidFill {
        solid_fill_choice: Some(SolidFillChoice::ASrgbClr(Box::new(srgb))),
        xmlns: vec![],
        xml_other_attrs: vec![],
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
        shape_properties_choice2: Some(ShapePropertiesChoice2::ASolidFill(Box::new(solid_fill))),
        shape_properties_choice3: None,
        black_white_mode: None,
        a_ln: None,
        a_scene3d: None,
        a_sp3d: None,
        a_ext_lst: None,
        xmlns: vec![],
    };

    // No text body — the bar is geometry-only.
    // Label text is in the adjacent ColorLabel (Body-role) frame.
    Shape {
        use_background_fill: None,
        non_visual_shape_properties: Box::new(nv_sp_pr),
        shape_properties: Box::new(sp_pr),
        shape_style: None,
        text_body: None,
        extension_list_with_modification: None,
    }
}

/// Build a `<p:sp>` element for a `FrameContent::Shape` frame (STORY-072 AC-004).
///
/// Returns `None` when `fill` is `FillSpec::None` — transparent shapes emit no element.
/// Returns `Some(Shape)` for `SolidColor` and `Gradient` fills.
///
/// ## OOXML element ordering (schema-significant)
///
/// `<p:nvSpPr>` → `<p:spPr>` → `<p:txBody>` (no txBody for fill-only shapes).
///
/// ## Alt text placement (BC-4.01.004 invariant 2)
///
/// Alt text goes on `<p:cNvPr descr="...">`, NOT on `<p:ph altText>`.
/// `AltText::Provided` → `descr="<alt_text>"`.
/// `AltText::Decorative | Unspecified` → `descr=""` (empty attribute present).
fn build_shape_fill_sp(
    shape_id: u32,
    frame_idx: usize,
    x: i64,
    y: i64,
    cx: i64,
    cy: i64,
    shape_frame: &ShapeFrame,
) -> Option<Shape> {
    // Determine the fill choice; return None for transparent shapes.
    let fill_choice: ShapePropertiesChoice2 = match &shape_frame.fill {
        FillSpec::None => return None,
        FillSpec::SolidColor(rgb) => {
            let hex_val = format!("{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b);
            let srgb = RgbColorModelHex {
                val: hex_val,
                legacy_spreadsheet_color_index: None,
                rgb_color_model_hex_choice: vec![],
                xmlns: vec![],
                xml_other_attrs: vec![],
            };
            ShapePropertiesChoice2::ASolidFill(Box::new(SolidFill {
                solid_fill_choice: Some(SolidFillChoice::ASrgbClr(Box::new(srgb))),
                xmlns: vec![],
                xml_other_attrs: vec![],
            }))
        },
        FillSpec::Gradient { from, to } => {
            // AC-004 (STORY-072): emit <a:gradFill> with two stops and <a:lin ang="5400000">
            // (90° = top-to-bottom, per OOXML 1/60000-degree units: 90 * 60000 = 5400000).
            let make_stop = |pos: i32, rgb: &Rgb| GradientStop {
                position: pos,
                gradient_stop_choice: Some(GradientStopChoice::ASrgbClr(Box::new(
                    RgbColorModelHex {
                        val: format!("{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b),
                        legacy_spreadsheet_color_index: None,
                        rgb_color_model_hex_choice: vec![],
                        xmlns: vec![],
                        xml_other_attrs: vec![],
                    },
                ))),
                xmlns: vec![],
                xml_other_attrs: vec![],
            };
            let gs_lst = GradientStopList {
                a_gs: vec![make_stop(0, from), make_stop(100_000, to)],
            };
            let lin = LinearGradientFill {
                // 5400000 = 90 degrees (top-to-bottom) in 1/60000-degree OOXML units.
                angle: Some(5_400_000_i32),
                scaled: None,
            };
            ShapePropertiesChoice2::AGradFill(Box::new(GradientFill {
                flip: None,
                rotate_with_shape: None,
                gradient_stop_list: Some(gs_lst),
                gradient_fill_choice: Some(GradientFillChoice::ALin(Box::new(lin))),
                a_tile_rect: None,
            }))
        },
    };

    // Alt text: BC-4.01.004 invariant 2 — descr on <p:cNvPr>.
    let alt_text_value: Option<String> = match &shape_frame.alt {
        AltText::Provided(text) => Some(text.to_string()),
        AltText::Decorative | AltText::Unspecified => Some(String::new()),
    };

    let cnv_pr = NonVisualDrawingProperties {
        id: shape_id,
        name: format!("Shape {frame_idx}"),
        description: alt_text_value,
        hidden: None,
        title: None,
        hyperlink_on_click: None,
        hyperlink_on_hover: None,
        non_visual_drawing_properties_extension_list: None,
        xmlns: vec![],
    };

    let cnv_sp_pr = NonVisualShapeDrawingProperties::default();

    // No <p:ph> — free-standing shape, not a placeholder.
    let nv_pr = ApplicationNonVisualDrawingProperties {
        is_photo: None,
        user_drawn: None,
        placeholder_shape: None,
        application_non_visual_drawing_properties_choice: None,
        p_cust_data_lst: None,
        p_ext_lst: None,
    };

    let nv_sp_pr = NonVisualShapeProperties {
        non_visual_drawing_properties: Box::new(cnv_pr),
        non_visual_shape_drawing_properties: Box::new(cnv_sp_pr),
        application_non_visual_drawing_properties: Box::new(nv_pr),
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
        shape_properties_choice2: Some(fill_choice),
        shape_properties_choice3: None,
        black_white_mode: None,
        a_ln: None,
        a_scene3d: None,
        a_sp3d: None,
        a_ext_lst: None,
        xmlns: vec![],
    };

    Some(Shape {
        use_background_fill: None,
        non_visual_shape_properties: Box::new(nv_sp_pr),
        shape_properties: Box::new(sp_pr),
        shape_style: None,
        text_body: None,
        extension_list_with_modification: None,
    })
}

/// Build a typed `<p:pic>` element for a media frame with accessibility metadata.
///
/// `shape_id` is the numeric shape ID for `<p:cNvPr id="...">`.
/// `frame_idx` is the 0-based frame index, used for the shape name.
/// `kind` is the human-readable media type label used in `<p:cNvPr name="...">`.
///
///   - Pass `"Image"` for image frames → `name="Image N"`.
///   - Pass `"Chart"` for chart frames → `name="Chart N"`.
///
///   This matches how `build_picture` names diagram frames (`"Diagram N"`), ensuring
///   consistent `PowerPoint` Selection Pane and accessibility-tree labels.
///
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
    kind: &str,
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
        name: format!("{kind} {frame_idx}"),
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
/// `alt_text` is the accessibility alt text for `descr` (BC-4.01.004, STORY-039).
///
///   - Non-empty: `descr="<alt_text>"` (BC-4.01.004 postcondition 1).
///   - Empty: `descr=""` (BC-4.01.004 postcondition 2 — attribute present, empty value).
///
/// Position and size are given in integer EMU.
///
/// The resulting element embeds the media via `<a:blip r:embed="..."/>` inside
/// `<p:blipFill>` with a stretch fill, and positions it via `<a:xfrm>` inside
/// `<p:spPr>` with a rect preset geometry — matching the schema produced by the
/// previous raw-XML path but through typed ooxmlsdk builders.
///
/// Alt text is NEVER truncated (BC-4.01.004 invariant 1). XML special characters
/// in `alt_text` are escaped by `ooxmlsdk` automatically (BC-4.01.004 EC-001).
///
/// The `descr` attribute is placed on `<p:cNvPr>` (BC-4.01.004 invariant 2),
/// NOT on `<p:ph altText>`. `AltTextEmbedder` is the single authoritative
/// descr-setting path (ADR-001).
fn build_picture(
    shape_id: u32,
    frame_idx: usize,
    r_embed: &str,
    alt_text: &str,
    x: i64,
    y: i64,
    cx: i64,
    cy: i64,
) -> Picture {
    // --- Non-visual properties ---
    // BC-4.01.004 invariant 2: descr is on cNvPr, not on ph.altText.
    // Some("") for decorative (descr="" — attribute present, empty value).
    // Some(non_empty) for non-decorative (descr="alt text").
    // ooxmlsdk writes Some(v) as descr="v" for any v, including empty string.
    let cnv_pr = NonVisualDrawingProperties {
        id: shape_id,
        name: format!("Diagram {frame_idx}"),
        description: Some(alt_text.to_owned()),
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

#[cfg(test)]
#[allow(clippy::missing_docs_in_private_items, clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use slideforge_types::{InlineNode, MathNode, SourceSpan};

    fn make_math_node(latex: &str) -> InlineNode {
        InlineNode::Math(MathNode {
            latex: Arc::from(latex),
            display: false,
            span: SourceSpan::default(),
        })
    }

    /// STORY-081 I1 — PPTX body path: `InlineNode::Math` is handled without panic,
    /// and the LaTeX source is emitted as a plain text run (EC-001 fallback from the
    /// unified [`render_inline_nodes_to_runs`] engine via [`super::nodes_to_body_runs`]).
    ///
    /// Intent: Math in PPTX body does not panic. The production body path emits the
    /// LaTeX source as a diagnostic fallback text run so the content is not silently
    /// dropped. Full OMML rendering is deferred to STORY-009.
    ///
    /// This test exercises the PRODUCTION code path (`nodes_to_body_runs` →
    /// `render_inline_nodes_to_runs`) — not a test-only helper — so a regression
    /// in the unified engine will cause this test to fail.
    #[test]
    fn test_story_081_i1_pptx_math_in_body_returns_latex_run_no_panic() {
        let latex = "x^2 + y^2 = z^2";
        let math_node = make_math_node(latex);
        // Drive through the production body-path engine (hlink_map = empty, lang = None).
        let runs = super::nodes_to_body_runs(&[math_node], &[], None);
        // EC-001 fallback: the unified engine emits the LaTeX source as a plain text run.
        assert_eq!(
            runs.len(),
            1,
            "STORY-081 I1: InlineNode::Math must produce exactly 1 EC-001 fallback run \
             containing the LaTeX source via the unified engine. Got {} runs.",
            runs.len()
        );
        assert_eq!(
            runs[0].text, latex,
            "STORY-081 I1: Math fallback run must carry the LaTeX source text; got {:?}",
            runs[0].text
        );
    }

    /// STORY-081 I1 — PPTX body path: Bold + Math mixed inline.
    ///
    /// Bold produces a run with `bold=true`; Math produces a run with the LaTeX
    /// source as text (EC-001 fallback). Both are driven through the production
    /// `nodes_to_body_runs` → `render_inline_nodes_to_runs` engine.
    #[test]
    fn test_story_081_i1_pptx_bold_plus_math_both_produce_runs() {
        let bold_node = InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold"))]);
        let math_node = make_math_node("\\alpha");

        let bold_runs = super::nodes_to_body_runs(&[bold_node], &[], None);
        let math_runs = super::nodes_to_body_runs(&[math_node], &[], None);

        assert!(!bold_runs.is_empty(), "Bold must produce at least one run");
        let has_bold = bold_runs.iter().any(|r| {
            r.run_properties
                .as_ref()
                .and_then(|rpr| rpr.bold)
                .unwrap_or(false)
        });
        assert!(has_bold, "Bold run must have bold=true in run_properties");

        // EC-001: Math produces the LaTeX source as a plain text run.
        assert_eq!(
            math_runs.len(),
            1,
            "Math must produce exactly 1 EC-001 fallback run; got {} runs",
            math_runs.len()
        );
        assert_eq!(
            math_runs[0].text, "\\alpha",
            "Math fallback run must carry the LaTeX source; got {:?}",
            math_runs[0].text
        );
    }
}
