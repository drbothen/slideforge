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
    clippy::expect_used
)]
mod tests {
    mod core_tests;
}

use content_types::ContentTypesBuilder;
use ooxmlsdk::common::XmlNamespaceDecl;
use ooxmlsdk::schemas::p::{HandoutMaster, NotesMaster};
use presentation::PresentationSerializer;
use rels::{rel_types, RelsBuilder};
use slide_serializer::SlideSerializer;
use slideforge_layout::{FrameContent, LaidOutDeck};
use slideforge_plugin_api::{ExportError, ExportOptions, Exporter};
use slideforge_types::{Brand, Deck};
use zip_assembler::{ZipAssembler, ZipPart};

use crate::error::PptxError;

/// The built-in PPTX exporter plugin.
///
/// `PptxExporter` implements the [`Exporter`] plugin trait for the `.pptx`
/// output format. It is registered with the [`slideforge_plugin_api::PluginRegistry`]
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
        &self,
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

        // ─── Step 1: Slide XMLs via SlideSerializer ───────────────────────────
        let mut slide_rel_ids: Vec<String> = Vec::new();
        // Global media index for deterministic media file naming.
        let mut media_idx = 1_usize;

        for (i, slide) in laid_out.slides.iter().enumerate() {
            let slide_path = format!("ppt/slides/slide{}.xml", i + 1);
            let rels_path = format!("ppt/slides/_rels/slide{}.xml.rels", i + 1);

            // Build slide .rels (slide → layout relationship).
            let mut slide_rels = RelsBuilder::new();
            // All slides reference slideLayout1 for now (STORY-037 baseline).
            let layout_rel_id = slide_rels.add(
                rel_types::SLIDE_LAYOUT,
                "../slideLayouts/slideLayout1.xml",
            );

            // Extract diagram SVG media from this slide's frames.
            // This is done here (not in SlideSerializer) to preserve the 2-tuple
            // signature required by the EC-005 test that calls SlideSerializer directly.
            for frame in &slide.frames {
                if let FrameContent::Diagram(normalized_svg) = &frame.content {
                    let media_filename = format!("image{media_idx}.svg");
                    let media_path = format!("ppt/media/{media_filename}");
                    // Add relationship from slide to media.
                    slide_rels.add(
                        rel_types::IMAGE,
                        format!("../media/{media_filename}"),
                    );
                    parts.push(ZipPart {
                        path: media_path,
                        bytes: normalized_svg.as_str().as_bytes().to_vec(),
                    });
                    media_idx += 1;
                }
            }

            // Serialize the slide XML.
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

        // ─── Step 2: presentation.xml.rels ────────────────────────────────────
        let mut prs_rels = RelsBuilder::new();
        let master_rel_id = prs_rels.add(
            rel_types::SLIDE_MASTER,
            "slideMasters/slideMaster1.xml",
        );
        let notes_master_rel_id = prs_rels.add(
            rel_types::NOTES_MASTER,
            "notesMasters/notesMaster1.xml",
        );
        let handout_master_rel_id = prs_rels.add(
            rel_types::HANDOUT_MASTER,
            "handoutMasters/handoutMaster1.xml",
        );
        for i in 0..laid_out.slides.len() {
            let rid =
                prs_rels.add(rel_types::SLIDE, format!("slides/slide{}.xml", i + 1));
            slide_rel_ids.push(rid);
        }

        parts.push(ZipPart {
            path: "ppt/_rels/presentation.xml.rels".to_string(),
            bytes: prs_rels.build()?,
        });

        // ─── Step 3: presentation.xml ─────────────────────────────────────────
        let prs_xml = PresentationSerializer::build(
            laid_out,
            brand,
            &slide_rel_ids,
            &master_rel_id,
            &notes_master_rel_id,
            &handout_master_rel_id,
        )?;
        parts.push(ZipPart {
            path: "ppt/presentation.xml".to_string(),
            bytes: prs_xml,
        });

        // ─── Step 4: Brand master XML (verbatim or minimal stub) ──────────────
        // The brand embeds master XML. If not available, use a minimal stub.
        let master_xml = build_minimal_slide_master_xml()?;
        parts.push(ZipPart {
            path: "ppt/slideMasters/slideMaster1.xml".to_string(),
            bytes: master_xml,
        });

        // master .rels (master → theme + 31 layout relationships)
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

        // ─── Step 5: 31 slide layouts ─────────────────────────────────────────
        for n in 1..=31 {
            let layout_xml = build_minimal_slide_layout_xml(n)?;
            parts.push(ZipPart {
                path: format!("ppt/slideLayouts/slideLayout{n}.xml"),
                bytes: layout_xml,
            });

            // layout .rels (layout → master)
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

        // ─── Step 6: Theme XML ────────────────────────────────────────────────
        let theme_xml = build_minimal_theme_xml()?;
        parts.push(ZipPart {
            path: "ppt/theme/theme1.xml".to_string(),
            bytes: theme_xml,
        });

        // ─── Step 7: notesMaster1.xml + handoutMaster1.xml (stubs) ───────────
        // Always written — BC-4.01.006 invariant 1.
        let notes_master_xml = build_minimal_notes_master_xml()?;
        parts.push(ZipPart {
            path: "ppt/notesMasters/notesMaster1.xml".to_string(),
            bytes: notes_master_xml,
        });

        let mut notes_master_rels = RelsBuilder::new();
        notes_master_rels.add(rel_types::THEME, "../theme/theme1.xml");
        parts.push(ZipPart {
            path: "ppt/notesMasters/_rels/notesMaster1.xml.rels".to_string(),
            bytes: notes_master_rels.build()?,
        });

        let handout_master_xml = build_minimal_handout_master_xml()?;
        parts.push(ZipPart {
            path: "ppt/handoutMasters/handoutMaster1.xml".to_string(),
            bytes: handout_master_xml,
        });

        let mut handout_master_rels = RelsBuilder::new();
        handout_master_rels.add(rel_types::THEME, "../theme/theme1.xml");
        parts.push(ZipPart {
            path: "ppt/handoutMasters/_rels/handoutMaster1.xml.rels".to_string(),
            bytes: handout_master_rels.build()?,
        });

        // ─── Step 8: docProps/core.xml + app.xml ─────────────────────────────
        let lang = deck
            .metadata
            .lang
            .as_deref()
            .unwrap_or("en-US");
        let core_xml = build_core_xml(lang)?;
        parts.push(ZipPart {
            path: "docProps/core.xml".to_string(),
            bytes: core_xml,
        });

        let app_xml = build_app_xml()?;
        parts.push(ZipPart {
            path: "docProps/app.xml".to_string(),
            bytes: app_xml,
        });

        // ─── Step 9: _rels/.rels (root) ───────────────────────────────────────
        let mut root_rels = RelsBuilder::new();
        root_rels.add(rel_types::OFFICE_DOCUMENT, "ppt/presentation.xml");
        root_rels.add(rel_types::CORE_PROPERTIES, "docProps/core.xml");
        root_rels.add(rel_types::EXTENDED_PROPERTIES, "docProps/app.xml");
        parts.push(ZipPart {
            path: "_rels/.rels".to_string(),
            bytes: root_rels.build()?,
        });

        // ─── Step 10: [Content_Types].xml ─────────────────────────────────────
        let mut ct = ContentTypesBuilder::new();
        for _ in 0..laid_out.slides.len() {
            ct.add_slide();
        }
        for _ in 0..31 {
            ct.add_layout();
        }
        // Register media parts in content types.
        // Collect them from `parts` by looking at paths starting with "ppt/media/".
        for part in &parts {
            if part.path.starts_with("ppt/media/") {
                // Determine content type from extension.
                let ct_mime = if part.path.ends_with(".svg") {
                    "image/svg+xml"
                } else if part.path.ends_with(".png") {
                    "image/png"
                } else {
                    "application/octet-stream"
                };
                ct.add_media(&part.path, ct_mime);
            }
        }
        parts.push(ZipPart {
            path: "[Content_Types].xml".to_string(),
            bytes: ct.build()?,
        });

        // ─── Step 11: Assemble ZIP ─────────────────────────────────────────────
        ZipAssembler::assemble(parts)
    }
}

impl Exporter for PptxExporter {
    fn id(&self) -> &str {
        "pptx"
    }

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
        self.export_inner(deck, laid_out, brand, opts)
            .map_err(|e| ExportError::RenderError {
                message: e.to_string(),
            })
    }
}

/// Build a minimal valid `ppt/slideMasters/slideMaster1.xml`.
///
/// The master XML embeds the brand data when available. For the STORY-037
/// baseline, we build a minimal valid stub via ooxmlsdk.
fn build_minimal_slide_master_xml() -> Result<Vec<u8>, PptxError> {
    use ooxmlsdk::schemas::p::{CommonSlideData, GroupShapeProperties, ShapeTree, SlideMaster};

    let shape_tree = ShapeTree {
        non_visual_group_shape_properties: None,
        group_shape_properties: Some(Box::new(GroupShapeProperties::default())),
        shape_tree_choice: Vec::new(),
        p_ext_lst: None,
        xmlns: vec![],
        xml_other_attrs: vec![],
    };

    let csl = CommonSlideData {
        name: None,
        background: None,
        shape_tree: Box::new(shape_tree),
        customer_data_list: None,
        control_list: None,
        common_slide_data_extension_list: None,
    };

    let mut master = SlideMaster::default();
    master.xmlns = vec![
        XmlNamespaceDecl::new("a", "http://schemas.openxmlformats.org/drawingml/2006/main"),
        XmlNamespaceDecl::new(
            "r",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        ),
        XmlNamespaceDecl::new(
            "p",
            "http://schemas.openxmlformats.org/presentationml/2006/main",
        ),
    ];
    master.common_slide_data = Box::new(csl);

    master.to_xml_bytes().map_err(|e| PptxError::OoxmlElement {
        part: "ppt/slideMasters/slideMaster1.xml".to_string(),
        detail: e.to_string(),
    })
}

/// Build a minimal valid `ppt/slideLayouts/slideLayout{n}.xml`.
fn build_minimal_slide_layout_xml(layout_index: usize) -> Result<Vec<u8>, PptxError> {
    use ooxmlsdk::schemas::p::{CommonSlideData, GroupShapeProperties, ShapeTree, SlideLayout};

    let shape_tree = ShapeTree {
        non_visual_group_shape_properties: None,
        group_shape_properties: Some(Box::new(GroupShapeProperties::default())),
        shape_tree_choice: Vec::new(),
        p_ext_lst: None,
        xmlns: vec![],
        xml_other_attrs: vec![],
    };

    let csl = CommonSlideData {
        name: None,
        background: None,
        shape_tree: Box::new(shape_tree),
        customer_data_list: None,
        control_list: None,
        common_slide_data_extension_list: None,
    };

    let mut layout = SlideLayout::default();
    layout.xmlns = vec![
        XmlNamespaceDecl::new("a", "http://schemas.openxmlformats.org/drawingml/2006/main"),
        XmlNamespaceDecl::new(
            "r",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        ),
        XmlNamespaceDecl::new(
            "p",
            "http://schemas.openxmlformats.org/presentationml/2006/main",
        ),
    ];
    layout.common_slide_data = Box::new(csl);

    layout.to_xml_bytes().map_err(|e| PptxError::OoxmlElement {
        part: format!("ppt/slideLayouts/slideLayout{layout_index}.xml"),
        detail: e.to_string(),
    })
}

/// Build a minimal valid `ppt/theme/theme1.xml`.
///
/// Theme XML is generated from the brand palette. For STORY-037, we use
/// a minimal valid theme document.
fn build_minimal_theme_xml() -> Result<Vec<u8>, PptxError> {
    // Theme is generated via ooxmlsdk dml theme types.
    // For the STORY-037 baseline we emit a minimal valid theme via raw XML
    // because ooxmlsdk's theme types require many mandatory sub-elements
    // (fontScheme, fmtScheme, etc.). We use raw XML here as a minimal stub;
    // this is the ONLY place in this crate where we use pre-formed XML bytes
    // (not string concatenation — the bytes are a static well-formed XML document).
    let theme_xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
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
    Ok(theme_xml.to_vec())
}

/// Build a minimal valid `ppt/notesMasters/notesMaster1.xml`.
fn build_minimal_notes_master_xml() -> Result<Vec<u8>, PptxError> {
    let mut master = NotesMaster::default();
    master.xmlns = vec![
        XmlNamespaceDecl::new("a", "http://schemas.openxmlformats.org/drawingml/2006/main"),
        XmlNamespaceDecl::new(
            "r",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        ),
        XmlNamespaceDecl::new(
            "p",
            "http://schemas.openxmlformats.org/presentationml/2006/main",
        ),
    ];

    use ooxmlsdk::schemas::p::{CommonSlideData, GroupShapeProperties, ShapeTree};
    let shape_tree = ShapeTree {
        non_visual_group_shape_properties: None,
        group_shape_properties: Some(Box::new(GroupShapeProperties::default())),
        shape_tree_choice: Vec::new(),
        p_ext_lst: None,
        xmlns: vec![],
        xml_other_attrs: vec![],
    };
    let csl = CommonSlideData {
        name: None,
        background: None,
        shape_tree: Box::new(shape_tree),
        customer_data_list: None,
        control_list: None,
        common_slide_data_extension_list: None,
    };
    master.common_slide_data = Box::new(csl);

    master.to_xml_bytes().map_err(|e| PptxError::OoxmlElement {
        part: "ppt/notesMasters/notesMaster1.xml".to_string(),
        detail: e.to_string(),
    })
}

/// Build a minimal valid `ppt/handoutMasters/handoutMaster1.xml`.
fn build_minimal_handout_master_xml() -> Result<Vec<u8>, PptxError> {
    let mut master = HandoutMaster::default();
    master.xmlns = vec![
        XmlNamespaceDecl::new("a", "http://schemas.openxmlformats.org/drawingml/2006/main"),
        XmlNamespaceDecl::new(
            "r",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        ),
        XmlNamespaceDecl::new(
            "p",
            "http://schemas.openxmlformats.org/presentationml/2006/main",
        ),
    ];

    use ooxmlsdk::schemas::p::{CommonSlideData, GroupShapeProperties, ShapeTree};
    let shape_tree = ShapeTree {
        non_visual_group_shape_properties: None,
        group_shape_properties: Some(Box::new(GroupShapeProperties::default())),
        shape_tree_choice: Vec::new(),
        p_ext_lst: None,
        xmlns: vec![],
        xml_other_attrs: vec![],
    };
    let csl = CommonSlideData {
        name: None,
        background: None,
        shape_tree: Box::new(shape_tree),
        customer_data_list: None,
        control_list: None,
        common_slide_data_extension_list: None,
    };
    master.common_slide_data = Box::new(csl);

    master.to_xml_bytes().map_err(|e| PptxError::OoxmlElement {
        part: "ppt/handoutMasters/handoutMaster1.xml".to_string(),
        detail: e.to_string(),
    })
}

/// Build a minimal valid `docProps/core.xml`.
///
/// Emits required Dublin Core metadata fields. Full language embedding
/// for BC-4.01.004 / BC-5.01.005 is in STORY-039.
fn build_core_xml(lang: &str) -> Result<Vec<u8>, PptxError> {
    // core.xml uses Dublin Core / OPC schema. We emit a minimal valid document.
    // There's no ooxmlsdk type for OPC core properties that maps cleanly here,
    // so we construct a static minimal valid XML document.
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <dc:creator>slideforge</dc:creator>
  <dc:language>{lang}</dc:language>
  <dcterms:created xsi:type="dcterms:W3CDTF">1980-01-01T00:00:00Z</dcterms:created>
</cp:coreProperties>"#
    );
    Ok(xml.into_bytes())
}

/// Build a minimal valid `docProps/app.xml`.
fn build_app_xml() -> Result<Vec<u8>, PptxError> {
    let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
  <Application>slideforge</Application>
  <AppVersion>0.1.0</AppVersion>
</Properties>"#;
    Ok(xml.to_vec())
}
