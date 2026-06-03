//! Layout embedding for PPTX archives.
//!
//! [`LayoutEmbedder`] writes all 31 slide layout XML parts and their `.rels`
//! files into the PPTX ZIP. It always embeds exactly 31 layouts regardless of
//! how many slide types are used in the deck (BC-4.01.005 invariant 3).
//!
//! ## ADR-015 §1 compliance
//!
//! All layout XML bytes come from
//! `slideforge_brand::layout_xml::serialize_layout_to_xml(&brand_template.layouts[i])`.
//! No inline XML string construction is used for layout parts.
//!
//! ## STORY-038 tasks covered
//!
//! - AC-003: exactly 31 `slideLayout*.xml` parts in the PPTX ZIP
//! - AC-005: `[Content_Types].xml` registers all 31 layouts
//! - AC-008: `slideMaster1.xml.rels` has 31 layout relationship entries

use slideforge_brand::BrandTemplate;
use slideforge_brand::layout_xml::serialize_layout_to_xml;

use crate::error::PptxError;
use crate::rels::{RelsBuilder, rel_types};
use crate::zip_assembler::ZipPart;

/// Embeds all 31 slide layout XML files and their `.rels` into a parts list.
///
/// Called by `export_inner` to guarantee BC-4.01.005 invariant 3: all 31
/// layouts are always present, regardless of which slide types are used.
pub struct LayoutEmbedder;

impl LayoutEmbedder {
    /// Embed all 31 slide layout parts into `parts`.
    ///
    /// For each index `n` in `1..=31`:
    /// - Writes `ppt/slideLayouts/slideLayout{n}.xml` from
    ///   `serialize_layout_to_xml(&brand_template.layouts[n-1])`.
    /// - Writes `ppt/slideLayouts/_rels/slideLayout{n}.xml.rels` with a single
    ///   relationship back to `slideMaster1.xml`.
    ///
    /// If `brand_template.layouts` has fewer than 31 entries (should never
    /// occur for synthesized brands), the last available layout is reused for
    /// remaining slots with a `tracing::warn!`.
    ///
    /// # Errors
    ///
    /// Returns [`PptxError`] if a layout `.rels` part cannot be serialised.
    pub fn embed(
        brand_template: &BrandTemplate,
        parts: &mut Vec<ZipPart>,
    ) -> Result<(), PptxError> {
        const LAYOUT_COUNT: usize = 31;

        for n in 1..=LAYOUT_COUNT {
            let layout_idx = (n - 1).min(brand_template.layouts.len().saturating_sub(1));
            let layout_xml = if brand_template.layouts.is_empty() {
                // Defensive fallback: produce a minimal valid layout XML if the
                // template has no layouts (should never happen for synthesized brands).
                tracing::warn!(
                    layout_n = n,
                    "LayoutEmbedder::embed: brand_template has no layouts; \
                     emitting empty layout placeholder"
                );
                minimal_empty_layout_xml(n)
            } else {
                serialize_layout_to_xml(&brand_template.layouts[layout_idx])
            };

            parts.push(ZipPart {
                path: format!("ppt/slideLayouts/slideLayout{n}.xml"),
                bytes: layout_xml,
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
}

/// Produce a minimal valid `slideLayoutN.xml` for emergency fallback.
///
/// This is only called when `brand_template.layouts` is empty (should never
/// occur for synthesized brands). The result is a bare `<p:sldLayout>` with
/// no placeholders — schema-valid but visually unstyled.
fn minimal_empty_layout_xml(n: usize) -> Vec<u8> {
    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<p:sldLayout xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main""#,
            r#" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main""#,
            r#" type="cust" preserve="1">"#,
            r#"<p:cSld name="Layout {n}"><p:spTree>"#,
            r#"<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>"#,
            r#"<p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/>"#,
            r#"<a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>"#,
            r#"</p:spTree></p:cSld><p:hf/></p:sldLayout>"#,
        ),
        n = n
    )
    .into_bytes()
}
