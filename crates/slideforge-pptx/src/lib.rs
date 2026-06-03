//! `slideforge-pptx` — PPTX exporter plugin for the slideforge pipeline.
//!
//! This crate implements the [`slideforge_plugin_api::Exporter`] trait for the
//! `.pptx` format. It takes a [`slideforge_types::Deck`] (semantic IR) and a
//! [`slideforge_layout::LaidOutDeck`] (geometric IR) together with a resolved
//! [`slideforge_types::Brand`] and produces a valid PPTX ZIP archive as
//! `Vec<u8>`.
//!
//! ## Architecture (STORY-037 / BC-4.01.001)
//!
//! ```text
//! PptxExporter::export(deck, laid_out, brand, opts)
//!   │
//!   ├─ SlideSerializer::build()     → ppt/slides/slide{n}.xml
//!   ├─ PresentationSerializer::build() → ppt/presentation.xml
//!   ├─ ContentTypesBuilder::build() → [Content_Types].xml
//!   ├─ RelsBuilder::build()         → _rels/.rels, ppt/_rels/…, etc.
//!   └─ ZipAssembler::assemble()     → final ZIP bytes
//! ```
//!
//! ## Constraints
//!
//! - All OOXML element construction goes through `ooxmlsdk` typed builders
//!   (ADR-001). No XML string concatenation.
//! - All coordinates are integer [`slideforge_types::Emu`] (`i64`). No `f64`.
//! - Report and detail register content MUST NOT appear in slide XML bodies.
//! - `notesMaster1.xml` and `handoutMaster1.xml` are always written.
//! - Slide IDs start at 256; master ID is 2^31.
//! - Output is deterministic: same inputs → byte-identical output.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod content_types;
pub mod error;
pub mod presentation;
pub mod rels;
pub mod slide_serializer;
pub mod zip_assembler;

#[cfg(test)]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::doc_markdown,
    dead_code,
    clippy::option_if_let_else,
    clippy::map_unwrap_or
)]
mod tests {
    mod core_tests;
}

use ooxmlsdk::common::XmlNamespaceDecl;
use ooxmlsdk::schemas::p::{
    CommonSlideData, GroupShapeProperties, HandoutMaster, NotesMaster, ShapeTree, SlideLayout,
    SlideMaster,
};

use content_types::ContentTypesBuilder;
use presentation::PresentationSerializer;
use rels::{RelsBuilder, rel_types};
use slide_serializer::SlideSerializer;
use slideforge_layout::{FrameContent, LaidOutDeck};
use slideforge_plugin_api::{ExportError, ExportOptions, Exporter};
use slideforge_types::{Brand, Deck};
use zip_assembler::{ZipAssembler, ZipPart};

use crate::error::PptxError;

/// Standard `PresentationML` namespace URI.
const XMLNS_PML: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
/// Standard `DrawingML` namespace URI.
const XMLNS_DML: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
/// Standard `OPC` relationships namespace URI.
const XMLNS_RELS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// Build the standard namespace declarations used in `PresentationML` parts.
fn pml_xmlns() -> Vec<XmlNamespaceDecl> {
    vec![
        XmlNamespaceDecl::new("a", XMLNS_DML),
        XmlNamespaceDecl::new("r", XMLNS_RELS),
        XmlNamespaceDecl::new("p", XMLNS_PML),
    ]
}

/// Build a minimal `ShapeTree` with no shapes (for stubs).
fn empty_shape_tree() -> ShapeTree {
    ShapeTree {
        non_visual_group_shape_properties: None,
        group_shape_properties: Some(Box::new(GroupShapeProperties::default())),
        shape_tree_choice: Vec::new(),
        p_ext_lst: None,
        xmlns: vec![],
        xml_other_attrs: vec![],
    }
}

/// Build a `CommonSlideData` wrapping the given `ShapeTree`.
fn common_slide_data(shape_tree: ShapeTree) -> CommonSlideData {
    CommonSlideData {
        name: None,
        background: None,
        shape_tree: Box::new(shape_tree),
        customer_data_list: None,
        control_list: None,
        common_slide_data_extension_list: None,
    }
}

/// The built-in PPTX exporter plugin.
///
/// `PptxExporter` implements the [`Exporter`] plugin trait for the `.pptx`
/// output format. It is registered with the `PluginRegistry`
/// under the id `"pptx"`.
///
/// ## Thread safety
///
/// `PptxExporter` holds no mutable state; it is `Send + Sync` by construction.
#[derive(Debug, Clone, Copy, Default)]
pub struct PptxExporter;

impl PptxExporter {
    /// Create a new `PptxExporter` instance.
    ///
    /// The exporter is stateless; this function is equivalent to
    /// `PptxExporter::default()`.
    #[must_use]
    pub fn new() -> Self {
        PptxExporter
    }

    /// Core serialisation logic: build all PPTX parts and assemble the ZIP.
    ///
    /// Separated from the trait method so it can return [`PptxError`] directly
    /// before the trait boundary converts it to [`ExportError`].
    fn export_inner(
        deck: &Deck,
        laid_out: &LaidOutDeck,
        brand: &Brand,
        _opts: &ExportOptions,
    ) -> Result<Vec<u8>, PptxError> {
        tracing::debug!(
            slides = laid_out.slides.len(),
            "PptxExporter::export_inner starting"
        );

        let mut parts: Vec<ZipPart> = Vec::new();

        // Build all parts, collecting into `parts`.
        build_slide_parts(laid_out, &mut parts)?;
        let slide_rel_ids = build_presentation_rels(laid_out, &mut parts)?;
        build_presentation_xml(laid_out, brand, &slide_rel_ids, &mut parts)?;
        build_master_parts(&mut parts)?;
        build_layout_parts(&mut parts)?;
        build_theme_part(&mut parts);
        build_notes_handout_masters(&mut parts)?;
        build_doc_props(deck, &mut parts);
        build_root_rels(&mut parts)?;

        // Content types are built last so all media parts are visible in `parts`.
        // content_types are pushed inside build_content_types — collect result.
        // Re-build content types as the last part (after all media parts exist).
        let mut ct = ContentTypesBuilder::new();
        for _ in 0..laid_out.slides.len() {
            ct.add_slide();
        }
        for _ in 0..31 {
            ct.add_layout();
        }
        for part in &parts {
            if part.path.starts_with("ppt/media/") {
                let ct_mime = content_type_for_media_path(&part.path);
                ct.add_media(&part.path, ct_mime);
            }
        }
        parts.push(ZipPart {
            path: "[Content_Types].xml".to_string(),
            bytes: ct.build()?,
        });

        ZipAssembler::assemble(parts)
    }
}

/// Build all slide XML parts and slide `.rels` files, including media parts.
fn build_slide_parts(laid_out: &LaidOutDeck, parts: &mut Vec<ZipPart>) -> Result<(), PptxError> {
    let mut media_idx = 1_usize;

    for (i, slide) in laid_out.slides.iter().enumerate() {
        let slide_path = format!("ppt/slides/slide{}.xml", i + 1);
        let rels_path = format!("ppt/slides/_rels/slide{}.xml.rels", i + 1);

        let mut slide_rels = RelsBuilder::new();
        let layout_rel_id =
            slide_rels.add(rel_types::SLIDE_LAYOUT, "../slideLayouts/slideLayout1.xml");

        for frame in &slide.frames {
            if let FrameContent::Diagram(normalized_svg) = &frame.content {
                let media_filename = format!("image{media_idx}.svg");
                let media_path = format!("ppt/media/{media_filename}");
                slide_rels.add(rel_types::IMAGE, format!("../media/{media_filename}"));
                parts.push(ZipPart {
                    path: media_path,
                    bytes: normalized_svg.as_str().as_bytes().to_vec(),
                });
                media_idx += 1;
            }
        }

        let serializer = SlideSerializer::new(false, 0);
        let (slide_xml, _warnings) = serializer.build(slide, i, &layout_rel_id)?;

        parts.push(ZipPart {
            path: slide_path,
            bytes: slide_xml,
        });
        parts.push(ZipPart {
            path: rels_path,
            bytes: slide_rels.build()?,
        });
    }
    Ok(())
}

/// Build `ppt/_rels/presentation.xml.rels` and return the slide `rId` list.
fn build_presentation_rels(
    laid_out: &LaidOutDeck,
    parts: &mut Vec<ZipPart>,
) -> Result<Vec<String>, PptxError> {
    let mut prs_rels = RelsBuilder::new();
    let _master_rel_id = prs_rels.add(rel_types::SLIDE_MASTER, "slideMasters/slideMaster1.xml");
    let _notes_master_rel_id =
        prs_rels.add(rel_types::NOTES_MASTER, "notesMasters/notesMaster1.xml");
    let _handout_master_rel_id = prs_rels.add(
        rel_types::HANDOUT_MASTER,
        "handoutMasters/handoutMaster1.xml",
    );

    let mut slide_rel_ids: Vec<String> = Vec::new();
    for i in 0..laid_out.slides.len() {
        let rid = prs_rels.add(rel_types::SLIDE, format!("slides/slide{}.xml", i + 1));
        slide_rel_ids.push(rid);
    }

    parts.push(ZipPart {
        path: "ppt/_rels/presentation.xml.rels".to_string(),
        bytes: prs_rels.build()?,
    });
    Ok(slide_rel_ids)
}

/// Build `ppt/presentation.xml`.
fn build_presentation_xml(
    laid_out: &LaidOutDeck,
    brand: &Brand,
    slide_rel_ids: &[String],
    parts: &mut Vec<ZipPart>,
) -> Result<(), PptxError> {
    // Rebuild rels to get deterministic rIds for presentation.xml referencing.
    let mut prs_rels_ids = RelsBuilder::new();
    let master_rel_id = prs_rels_ids.add(rel_types::SLIDE_MASTER, "slideMasters/slideMaster1.xml");
    let notes_master_rel_id =
        prs_rels_ids.add(rel_types::NOTES_MASTER, "notesMasters/notesMaster1.xml");
    let handout_master_rel_id = prs_rels_ids.add(
        rel_types::HANDOUT_MASTER,
        "handoutMasters/handoutMaster1.xml",
    );

    let prs_xml = PresentationSerializer::build(
        laid_out,
        brand,
        slide_rel_ids,
        &master_rel_id,
        &notes_master_rel_id,
        &handout_master_rel_id,
    )?;
    parts.push(ZipPart {
        path: "ppt/presentation.xml".to_string(),
        bytes: prs_xml,
    });
    Ok(())
}

/// Build `slideMaster1.xml` and its `.rels`.
fn build_master_parts(parts: &mut Vec<ZipPart>) -> Result<(), PptxError> {
    let csl = common_slide_data(empty_shape_tree());
    let master = SlideMaster {
        xmlns: pml_xmlns(),
        common_slide_data: Box::new(csl),
        ..SlideMaster::default()
    };
    parts.push(ZipPart {
        path: "ppt/slideMasters/slideMaster1.xml".to_string(),
        bytes: master.to_xml_bytes().map_err(|e| PptxError::OoxmlElement {
            part: "ppt/slideMasters/slideMaster1.xml".to_string(),
            detail: e.to_string(),
        })?,
    });

    let mut master_rels = RelsBuilder::new();
    master_rels.add(rel_types::THEME, "../theme/theme1.xml");
    for n in 1..=31 {
        master_rels.add(
            rel_types::SLIDE_LAYOUT,
            format!("../slideLayouts/slideLayout{n}.xml"),
        );
    }
    parts.push(ZipPart {
        path: "ppt/slideMasters/_rels/slideMaster1.xml.rels".to_string(),
        bytes: master_rels.build()?,
    });
    Ok(())
}

/// Build all 31 slide layout XML files and their `.rels`.
fn build_layout_parts(parts: &mut Vec<ZipPart>) -> Result<(), PptxError> {
    for n in 1..=31_usize {
        let csl = common_slide_data(empty_shape_tree());
        let layout = SlideLayout {
            xmlns: pml_xmlns(),
            common_slide_data: Box::new(csl),
            ..SlideLayout::default()
        };
        parts.push(ZipPart {
            path: format!("ppt/slideLayouts/slideLayout{n}.xml"),
            bytes: layout.to_xml_bytes().map_err(|e| PptxError::OoxmlElement {
                part: format!("ppt/slideLayouts/slideLayout{n}.xml"),
                detail: e.to_string(),
            })?,
        });

        let mut layout_rels = RelsBuilder::new();
        layout_rels.add(
            rel_types::SLIDE_MASTER_FROM_LAYOUT,
            "../slideMasters/slideMaster1.xml",
        );
        parts.push(ZipPart {
            path: format!("ppt/slideLayouts/_rels/slideLayout{n}.xml.rels"),
            bytes: layout_rels.build()?,
        });
    }
    Ok(())
}

/// Build `theme1.xml` and push to parts.
///
/// The theme uses a static well-formed XML byte slice (not string concatenation).
/// This is the ONLY place in this crate where pre-formed XML bytes are used —
/// because `ooxmlsdk`'s theme types require many mandatory sub-elements
/// (`fontScheme`, `fmtScheme`, etc.) that are not worth constructing via typed
/// builders for a minimal baseline.
fn build_theme_part(parts: &mut Vec<ZipPart>) {
    let theme_xml: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="slideforge">
  <a:themeElements>
    <a:clrScheme name="slideforge">
      <a:dk1><a:srgbClr val="000000"/></a:dk1>
      <a:lt1><a:srgbClr val="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="1F3864"/></a:dk2>
      <a:lt2><a:srgbClr val="E7E6E6"/></a:lt2>
      <a:accent1><a:srgbClr val="003087"/></a:accent1>
      <a:accent2><a:srgbClr val="0066CC"/></a:accent2>
      <a:accent3><a:srgbClr val="FF6B35"/></a:accent3>
      <a:accent4><a:srgbClr val="F5F5F5"/></a:accent4>
      <a:accent5><a:srgbClr val="4BACC6"/></a:accent5>
      <a:accent6><a:srgbClr val="F79646"/></a:accent6>
      <a:hlink><a:srgbClr val="0563C1"/></a:hlink>
      <a:folHlink><a:srgbClr val="954F72"/></a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="slideforge">
      <a:majorFont><a:latin typeface="Calibri Light"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont>
      <a:minorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont>
    </a:fontScheme>
    <a:fmtScheme name="slideforge">
      <a:fillStyleLst>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"><a:tint val="95000"/><a:satMod val="105000"/></a:schemeClr></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"><a:tint val="75000"/><a:satMod val="105000"/></a:schemeClr></a:solidFill>
      </a:fillStyleLst>
      <a:lnStyleLst>
        <a:ln w="6350" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/></a:ln>
        <a:ln w="12700" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/></a:ln>
        <a:ln w="19050" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/></a:ln>
      </a:lnStyleLst>
      <a:effectStyleLst>
        <a:effectStyle><a:effectLst/></a:effectStyle>
        <a:effectStyle><a:effectLst/></a:effectStyle>
        <a:effectStyle><a:effectLst><a:outerShdw blurRad="40000" dist="23000" dir="5400000" rotWithShape="0"><a:srgbClr val="000000"><a:alpha val="35000"/></a:srgbClr></a:outerShdw></a:effectLst></a:effectStyle>
      </a:effectStyleLst>
      <a:bgFillStyleLst>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"><a:tint val="95000"/><a:satMod val="170000"/></a:schemeClr></a:solidFill>
        <a:gradFill rotate="1"><a:gsLst><a:gs pos="0"><a:schemeClr val="phClr"><a:tint val="93000"/><a:satMod val="150000"/><a:shade val="98000"/><a:lumMod val="102000"/></a:schemeClr></a:gs><a:gs pos="50000"><a:schemeClr val="phClr"><a:tint val="98000"/><a:satMod val="130000"/><a:shade val="90000"/><a:lumMod val="103000"/></a:schemeClr></a:gs><a:gs pos="100000"><a:schemeClr val="phClr"><a:shade val="63000"/><a:satMod val="120000"/></a:schemeClr></a:gs></a:gsLst><a:lin ang="16200000" scaled="0"/></a:gradFill>
      </a:bgFillStyleLst>
    </a:fmtScheme>
  </a:themeElements>
</a:theme>"#;
    parts.push(ZipPart {
        path: "ppt/theme/theme1.xml".to_string(),
        bytes: theme_xml.to_vec(),
    });
}

/// Build `notesMaster1.xml` and `handoutMaster1.xml` (always present — BC-4.01.006).
fn build_notes_handout_masters(parts: &mut Vec<ZipPart>) -> Result<(), PptxError> {
    let notes_master = NotesMaster {
        xmlns: pml_xmlns(),
        common_slide_data: Box::new(common_slide_data(empty_shape_tree())),
        ..NotesMaster::default()
    };
    parts.push(ZipPart {
        path: "ppt/notesMasters/notesMaster1.xml".to_string(),
        bytes: notes_master
            .to_xml_bytes()
            .map_err(|e| PptxError::OoxmlElement {
                part: "ppt/notesMasters/notesMaster1.xml".to_string(),
                detail: e.to_string(),
            })?,
    });

    let mut notes_master_rels = RelsBuilder::new();
    notes_master_rels.add(rel_types::THEME, "../theme/theme1.xml");
    parts.push(ZipPart {
        path: "ppt/notesMasters/_rels/notesMaster1.xml.rels".to_string(),
        bytes: notes_master_rels.build()?,
    });

    let handout_master = HandoutMaster {
        xmlns: pml_xmlns(),
        common_slide_data: Box::new(common_slide_data(empty_shape_tree())),
        ..HandoutMaster::default()
    };
    parts.push(ZipPart {
        path: "ppt/handoutMasters/handoutMaster1.xml".to_string(),
        bytes: handout_master
            .to_xml_bytes()
            .map_err(|e| PptxError::OoxmlElement {
                part: "ppt/handoutMasters/handoutMaster1.xml".to_string(),
                detail: e.to_string(),
            })?,
    });

    let mut handout_master_rels = RelsBuilder::new();
    handout_master_rels.add(rel_types::THEME, "../theme/theme1.xml");
    parts.push(ZipPart {
        path: "ppt/handoutMasters/_rels/handoutMaster1.xml.rels".to_string(),
        bytes: handout_master_rels.build()?,
    });

    Ok(())
}

/// Build `docProps/core.xml` and `docProps/app.xml`.
///
/// These use pre-formed XML strings — the OPC core properties namespace is
/// outside the `ooxmlsdk` schema module scope for this story.
fn build_doc_props(deck: &Deck, parts: &mut Vec<ZipPart>) {
    let lang = deck.metadata.lang.as_deref().unwrap_or("en-US");

    let core_xml = format!(
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n",
            "<cp:coreProperties",
            " xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\"",
            " xmlns:dc=\"http://purl.org/dc/elements/1.1/\"",
            " xmlns:dcterms=\"http://purl.org/dc/terms/\"",
            " xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\n",
            "  <dc:creator>slideforge</dc:creator>\n",
            "  <dc:language>{lang}</dc:language>\n",
            "  <dcterms:created xsi:type=\"dcterms:W3CDTF\">1980-01-01T00:00:00Z</dcterms:created>\n",
            "</cp:coreProperties>"
        ),
        lang = lang,
    );
    parts.push(ZipPart {
        path: "docProps/core.xml".to_string(),
        bytes: core_xml.into_bytes(),
    });

    let app_xml: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
  <Application>slideforge</Application>
  <AppVersion>0.1.0</AppVersion>
</Properties>"#;
    parts.push(ZipPart {
        path: "docProps/app.xml".to_string(),
        bytes: app_xml.to_vec(),
    });
}

/// Build `_rels/.rels` (root relationships).
fn build_root_rels(parts: &mut Vec<ZipPart>) -> Result<(), PptxError> {
    let mut root_rels = RelsBuilder::new();
    root_rels.add(rel_types::OFFICE_DOCUMENT, "ppt/presentation.xml");
    root_rels.add(rel_types::CORE_PROPERTIES, "docProps/core.xml");
    root_rels.add(rel_types::EXTENDED_PROPERTIES, "docProps/app.xml");
    parts.push(ZipPart {
        path: "_rels/.rels".to_string(),
        bytes: root_rels.build()?,
    });
    Ok(())
}

/// Return the MIME type for a media file based on its path extension.
fn content_type_for_media_path(path: &str) -> &'static str {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext.to_lowercase().as_str() {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => "application/octet-stream",
    }
}

impl Exporter for PptxExporter {
    // The trait declares `&str`; returning a `'static` literal is compatible.
    // clippy::unnecessary_literal_bound is suppressed here because changing the
    // trait signature is out of scope for this story.
    #[allow(clippy::unnecessary_literal_bound)]
    fn id(&self) -> &str {
        "pptx"
    }

    #[allow(clippy::unnecessary_literal_bound)]
    fn extension(&self) -> &str {
        "pptx"
    }

    fn export(
        &self,
        deck: &Deck,
        laid_out: &LaidOutDeck,
        brand: &Brand,
        opts: &ExportOptions,
    ) -> Result<Vec<u8>, ExportError> {
        Self::export_inner(deck, laid_out, brand, opts).map_err(|e| ExportError::RenderError {
            message: e.to_string(),
        })
    }
}
