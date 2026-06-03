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
        // Use the crate-level constant so this loop is always in sync with
        // `build_master_parts` (which writes the corresponding rels entries).
        let available_len = brand_template.layouts.len();
        for n in 1..=crate::LAYOUT_COUNT {
            let layout_idx = (n - 1).min(available_len.saturating_sub(1));
            let layout_xml = if available_len == 0 {
                // Defensive fallback: produce a minimal valid layout XML if the
                // template has no layouts (should never happen for synthesized brands).
                tracing::warn!(
                    layout_n = n,
                    "LayoutEmbedder::embed: brand_template has no layouts; \
                     emitting empty layout placeholder"
                );
                minimal_empty_layout_xml(n)
            } else {
                // Partial-reuse path: when the brand template has fewer than
                // LAYOUT_COUNT layouts, slots beyond the available set reuse
                // the last layout. Emit a warning as promised by the docstring
                // (no-silent-fallback rule: F-P5-LOW-001).
                if n > available_len {
                    tracing::warn!(
                        layout_slot = n,
                        available_layouts = available_len,
                        reused_layout_idx = layout_idx,
                        "LayoutEmbedder::embed: brand_template has fewer than {} layouts \
                         (has {}); slot {} reuses layout index {} (partial-reuse fallback)",
                        crate::LAYOUT_COUNT,
                        available_len,
                        n,
                        layout_idx,
                    );
                }
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

#[cfg(test)]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used
)]
mod tests {
    use std::sync::Arc;

    use slideforge_brand::layout_xml::{
        HANDOUT_MASTER_STUB, NOTES_MASTER_STUB, generate_content_types_layout_entries,
    };
    use slideforge_brand::layouts::generate_all_layouts;
    use slideforge_brand::template::{BrandFonts, ColorSlot, ColorValue, MasterIds};
    use slideforge_brand::toml_schema::BrandConfig;
    use slideforge_brand::{BrandTemplate, FooterFlags};
    use tracing_test::traced_test;

    use super::LayoutEmbedder;
    use crate::LAYOUT_COUNT;

    /// Build a `BrandTemplate` with exactly `layout_count` layouts by taking the
    /// first `layout_count` entries from `generate_all_layouts`.
    fn brand_template_with_n_layouts(layout_count: usize) -> BrandTemplate {
        let config = BrandConfig::default_minimal();
        let all_layouts = generate_all_layouts(&config);
        let layouts = all_layouts.into_iter().take(layout_count).collect();
        BrandTemplate {
            colors: std::array::from_fn(|i| ColorSlot {
                name: Arc::from(slideforge_brand::template::COLOR_SLOT_NAMES[i]),
                value: ColorValue::Hex(Arc::from("003087")),
                is_derived: false,
            }),
            fonts: BrandFonts {
                heading: Arc::from("Calibri"),
                body: Arc::from("Calibri"),
            },
            logo: None,
            footer_text: None,
            footer_flags: FooterFlags::default(),
            layout_names: vec![],
            layouts,
            notes_master_stub: NOTES_MASTER_STUB.to_vec(),
            handout_master_stub: HANDOUT_MASTER_STUB.to_vec(),
            master_ids: MasterIds::default(),
            content_types_layout_entries: Arc::from(
                generate_content_types_layout_entries(LAYOUT_COUNT).as_str(),
            ),
        }
    }

    /// F-P5-LOW-001 load-bearing test: partial-reuse path emits `tracing::warn!`.
    ///
    /// CONTRACT (two simultaneous assertions):
    /// (a) `embed` still produces exactly `LAYOUT_COUNT` (31) layout XML parts
    ///     even when the brand template has only 30 layouts.
    /// (b) A `tracing::warn!` is emitted for the reused slot (slot 31 reuses
    ///     layout index 29, the last available layout).
    ///
    /// LOAD-BEARING: if the `tracing::warn!` in the partial-reuse branch is
    /// silently removed, `logs_contain("partial-reuse fallback")` will fail
    /// this test — satisfying TD-VSDD-059.
    #[traced_test]
    #[test]
    fn test_f_p5_low_001_partial_reuse_emits_warn_and_produces_31_parts() {
        // 30 layouts — one short of the required 31.
        let template = brand_template_with_n_layouts(30);
        assert_eq!(
            template.layouts.len(),
            30,
            "precondition: template must have exactly 30 layouts"
        );

        let mut parts = Vec::new();
        LayoutEmbedder::embed(&template, &mut parts).expect("embed must succeed");

        // (a) Assert `LAYOUT_COUNT` xml parts produced (not counting the .rels parts).
        let xml_parts_count = parts
            .iter()
            .filter(|p| {
                p.path.starts_with("ppt/slideLayouts/slideLayout")
                    && std::path::Path::new(&p.path)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
                    && !p.path.contains("_rels")
            })
            .count();
        assert_eq!(
            xml_parts_count, LAYOUT_COUNT,
            "embed must produce exactly {LAYOUT_COUNT} slideLayout*.xml parts \
             even when the brand template has only 30 layouts; got {xml_parts_count}"
        );

        // (b) Assert the partial-reuse warning fired.
        // The warn! message contains "partial-reuse fallback" — if the warn is
        // silently removed, this assertion fails (TD-VSDD-059 load-bearing gate).
        assert!(
            logs_contain("partial-reuse fallback"),
            "embed must emit a tracing::warn! containing 'partial-reuse fallback' \
             when the brand template has fewer than {LAYOUT_COUNT} layouts; \
             no such log was captured. This means the no-silent-fallback contract \
             (F-P5-LOW-001) is violated."
        );
    }

    /// Regression guard: a full 31-layout template must NOT emit the partial-reuse warn.
    ///
    /// Verifies the partial-reuse warn fires ONLY when there is an actual shortfall,
    /// not on every `embed` call.
    #[traced_test]
    #[test]
    fn test_f_p5_low_001_full_template_does_not_emit_partial_reuse_warn() {
        let template = brand_template_with_n_layouts(31);
        assert_eq!(
            template.layouts.len(),
            31,
            "precondition: template must have exactly 31 layouts"
        );

        let mut parts = Vec::new();
        LayoutEmbedder::embed(&template, &mut parts).expect("embed must succeed");

        assert!(
            !logs_contain("partial-reuse fallback"),
            "embed must NOT emit the partial-reuse warn when the brand template has \
             exactly {LAYOUT_COUNT} layouts — warn must only fire on shortfall"
        );
    }

    /// Verify the `.rels` companion parts are also produced (`LAYOUT_COUNT` of them).
    #[test]
    fn test_embed_produces_layout_count_rels_parts() {
        let template = brand_template_with_n_layouts(31);
        let mut parts = Vec::new();
        LayoutEmbedder::embed(&template, &mut parts).expect("embed must succeed");

        let rels_count = parts
            .iter()
            .filter(|p| {
                p.path.starts_with("ppt/slideLayouts/_rels/slideLayout")
                    && p.path.ends_with(".xml.rels")
            })
            .count();
        assert_eq!(
            rels_count, LAYOUT_COUNT,
            "embed must produce exactly {LAYOUT_COUNT} .rels companion parts; got {rels_count}"
        );
    }
}
