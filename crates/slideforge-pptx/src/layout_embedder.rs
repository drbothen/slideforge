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
//! - AC-004: `<p:sldLayoutIdLst>` in `slideMaster1.xml` has 31 entries
//! - AC-005: `[Content_Types].xml` registers all 31 layouts
//! - AC-008: `slideMaster1.xml.rels` has 31 layout relationship entries

use slideforge_brand::BrandTemplate;

use crate::error::PptxError;
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
    /// # Errors
    ///
    /// Returns [`PptxError`] if a layout XML part cannot be assembled.
    pub fn embed(_brand_template: &BrandTemplate, _parts: &mut Vec<ZipPart>) -> Result<(), PptxError> {
        todo!(
            "STORY-038 Step 4 — implement LayoutEmbedder::embed: \
             iterate brand_template.layouts[0..31], call serialize_layout_to_xml, \
             write slideLayout{{n}}.xml + .rels; see ADR-015 §1"
        )
    }
}
