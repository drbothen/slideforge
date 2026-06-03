//! Per-slide `slide*.xml` serializer.
//!
//! [`SlideSerializer`] produces the `ppt/slides/slide{n}.xml` bytes for one
//! [`slideforge_layout::LaidOutSlide`] using the `ooxmlsdk` typed API.
//!
//! ## Placeholder mapping (AC-004)
//!
//! Each `FrameContent` variant maps to a specific PPTX placeholder `idx`:
//!
//! | `FrameContent` variant | `<p:ph>` idx | Notes |
//! |------------------------|-------------|-------|
//! | `Title` / `Subtitle`   | `0`         | Title placeholder |
//! | `Body` / `TextRun`     | `1`         | Content/body placeholder |
//! | `Diagram`              | media embed | SVG written to ppt/media/ |
//! | Others                 | — skipped — | Media handled in STORY-038/039 |
//!
//! ## Element ordering (AC-006 / R4 finding)
//!
//! `ooxmlsdk` produces schema-correct element ordering automatically.
//! The expected child order within `<p:sp>` is:
//! `<p:nvSpPr>` → `<p:spPr>` → `<p:txBody>`.
//!
//! ## Dark layout `<p:clrMapOvr>` (AC-EC-005 / brand-architecture §Dark Layout)
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
}

impl SlideSerializer {
    /// Create a serializer for one slide.
    ///
    /// `is_dark_layout` controls `<p:clrMapOvr>` emission.
    /// `layout_index` is the 0-based layout index used to determine the slide
    /// layout reference in the slide XML (STORY-037 defers multi-layout support;
    /// default is layout 0 for all slides).
    #[must_use]
    pub fn new(is_dark_layout: bool, layout_index: usize) -> Self {
        Self {
            is_dark_layout,
            layout_index,
        }
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

        // Build the shape tree: one shape per text frame.
        let mut shape_tree = ShapeTree {
            non_visual_group_shape_properties: None,
            group_shape_properties: Some(Box::new(GroupShapeProperties::default())),
            shape_tree_choice: Vec::new(),
            p_ext_lst: None,
            xmlns: vec![],
            xml_other_attrs: vec![],
        };

        // Shape ID counter: start at 1.
        let mut shape_id: u32 = 1;

        for (frame_idx, frame) in slide.frames.iter().enumerate() {
            match &frame.content {
                FrameContent::Title(_) | FrameContent::Subtitle(_) => {
                    let text = match &frame.content {
                        FrameContent::Title(t) | FrameContent::Subtitle(t) => t.to_string(),
                        _ => String::new(),
                    };
                    validate_emu(slide_index, frame_idx, &frame.bbox)?;
                    let sp = build_shape(
                        shape_id,
                        0,
                        frame.bbox.x.0,
                        frame.bbox.y.0,
                        frame.bbox.width.0,
                        frame.bbox.height.0,
                        &text,
                    );
                    shape_tree
                        .shape_tree_choice
                        .push(ShapeTreeChoice::PSp(Box::new(sp)));
                    shape_id += 1;
                },

                FrameContent::Body(blocks) => {
                    let text = extract_body_text(blocks);
                    validate_emu(slide_index, frame_idx, &frame.bbox)?;
                    let sp = build_shape(
                        shape_id,
                        1,
                        frame.bbox.x.0,
                        frame.bbox.y.0,
                        frame.bbox.width.0,
                        frame.bbox.height.0,
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
                    let sp = build_shape(
                        shape_id,
                        1,
                        frame.bbox.x.0,
                        frame.bbox.y.0,
                        frame.bbox.width.0,
                        frame.bbox.height.0,
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
                FrameContent::Diagram(_) => {
                    // Find the rId for this frame from the caller-supplied map.
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

                // Image, Chart, Shape, ErrorSlidePlaceholder, Empty: skipped in STORY-037.
                FrameContent::Image { .. }
                | FrameContent::Chart
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
}

/// Validate EMU coordinates for a frame bounding box.
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

/// Build a single `<p:sp>` shape for a placeholder.
fn build_shape(shape_id: u32, ph_idx: u32, x: i64, y: i64, cx: i64, cy: i64, text: &str) -> Shape {
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

    // Placeholder shape type based on idx.
    let ph_type = if ph_idx == 0 {
        Some(PlaceholderValues::Title)
    } else {
        Some(PlaceholderValues::Body)
    };

    let ph = PlaceholderShape {
        r#type: ph_type,
        orientation: None,
        size: None,
        index: Some(ph_idx),
        has_custom_prompt: None,
        extension_list_with_modification: None,
    };

    let nv_pr = ApplicationNonVisualDrawingProperties {
        is_photo: None,
        user_drawn: None,
        placeholder_shape: Some(Box::new(ph)),
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
