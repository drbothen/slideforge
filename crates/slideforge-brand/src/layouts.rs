//! Slide layout definitions for the 31-layout brand template (BC-2.01.005).
//!
//! [`generate_all_layouts`] produces all 31 [`SlideLayoutDef`] entries in index
//! order: 11 standard OOXML layouts (SL-01 through SL-11) followed by 20 custom
//! `SF *` layouts (CL-01 through CL-20) as specified in Spike S5.
//!
//! ## Layout Taxonomy
//!
//! | Index | ID | Name | OOXML type |
//! |-------|----|------|-----------|
//! | 1 | SL-01 | Title Slide | `title` |
//! | 2 | SL-02 | Title and Content | `obj` |
//! | 3 | SL-03 | Section Header | `secHead` |
//! | 4 | SL-04 | Two Content | `twoObj` |
//! | 5 | SL-05 | Comparison | `twoColTx` |
//! | 6 | SL-06 | Title Only | `titleOnly` |
//! | 7 | SL-07 | Blank | `blank` |
//! | 8 | SL-08 | Content with Caption | `objTx` |
//! | 9 | SL-09 | Picture with Caption | `picTx` |
//! | 10 | SL-10 | Vertical Title and Text | `vertTitleAndTx` |
//! | 11 | SL-11 | Vertical Text | `vertTx` |
//! | 12 | CL-01 | SF Section Divider | — (dark layout) |
//! | … | … | … | … |
//! | 31 | CL-20 | SF Appendix | — |

use std::sync::Arc;

use crate::toml_schema::BrandConfig;

// ─── Types ────────────────────────────────────────────────────────────────────

/// A single slide layout definition.
///
/// Produced by [`generate_all_layouts`] and stored in [`crate::template::BrandTemplate`].
/// The PPTX exporter (STORY-037) serializes each `SlideLayoutDef` to a
/// `slideLayoutN.xml` file inside the OOXML ZIP package.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlideLayoutDef {
    /// 1-based layout index (1–31).
    ///
    /// Used to determine the OOXML file name `slideLayout{index}.xml` and
    /// the OOXML layout ID (= `layout_id_start + index - 1`).
    pub index: usize,

    /// Human-readable layout name (e.g., `"Title Slide"` or `"SF Section Divider"`).
    ///
    /// Standard layouts use Office display names.
    /// Custom layouts always start with `"SF "` (two letters + space).
    pub name: Arc<str>,

    /// OOXML `<p:sldLayout type="...">` attribute value.
    ///
    /// `Some("title")`, `Some("obj")`, etc. for standard layouts.
    /// `None` for custom `SF *` layouts (they use `type="custom"`).
    pub ooxml_type: Option<Arc<str>>,

    /// Placeholder definitions for this layout.
    ///
    /// The Blank layout (SL-07) has an empty `Vec`.
    /// All other layouts have at least one placeholder (AC-010 invariant).
    pub placeholders: Vec<LayoutPlaceholder>,

    /// Whether this layout has a dark color map override.
    ///
    /// `true` for CL-01 ("SF Section Divider") and CL-11 ("SF End Slide").
    /// These layouts carry `<p:clrMapOvr>` with `bg1="dk2" tx1="lt1"` (AC-009).
    pub has_color_override: bool,

    /// Background color token for the color map override.
    ///
    /// `Some("dk2")` for dark layouts; `None` for all other layouts.
    pub color_override_bg: Option<Arc<str>>,

    /// Text color token for the color map override.
    ///
    /// `Some("lt1")` for dark layouts; `None` for all other layouts.
    pub color_override_tx: Option<Arc<str>>,
}

/// A placeholder shape within a slide layout.
///
/// Each placeholder carries a semantic type, an OOXML index, a semantic
/// accessibility name, and position/size in EMU (English Metric Units).
///
/// Used by the PPTX exporter (STORY-037) to generate `<p:sp>` placeholder
/// elements inside `<p:spTree>` for each layout.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LayoutPlaceholder {
    /// OOXML placeholder type string (e.g., `"ctrTitle"`, `"body"`, `"pic"`).
    ///
    /// `"title"` or `"ctrTitle"` for title placeholders; `"body"` for content.
    pub ph_type: Arc<str>,

    /// OOXML placeholder index (`idx` attribute).
    ///
    /// The title/center-title placeholder has `idx = 0`.
    /// Body/content placeholders start at `idx = 1`.
    pub idx: u32,

    /// Semantic accessibility name for `<p:cNvPr name="...">`.
    ///
    /// Must be a descriptive label, NOT `"Shape N"` (AC-010 invariant,
    /// DI-001 accessibility requirement).
    pub accessibility_name: Arc<str>,

    /// X position in EMU.
    pub x: i64,

    /// Y position in EMU.
    pub y: i64,

    /// Width in EMU.
    pub cx: i64,

    /// Height in EMU.
    pub cy: i64,
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Generate all 31 slide layout definitions.
///
/// Returns a `Vec<SlideLayoutDef>` with exactly 31 entries in index order
/// (1-based). The first 11 are standard OOXML layouts; the remaining 20
/// are custom `SF *` layouts.
///
/// `config` is used to parameterize layout geometry or color choices in the
/// future; the current stub ignores it.
///
/// # Contract
///
/// - `result.len() == 31` always (BC-2.01.005 postcondition 1)
/// - `result[0..11]` all have `ooxml_type = Some(...)` (BC-2.01.005 postcondition 2)
/// - `result[11..31]` all have names starting with `"SF "` (BC-2.01.005 postcondition 3)
/// - `result[6].placeholders.is_empty()` (Blank layout — AC-010)
/// - `result[11].has_color_override && result[21].has_color_override` (AC-009)
#[must_use]
pub fn generate_all_layouts(_config: &BrandConfig) -> Vec<SlideLayoutDef> {
    todo!()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_config() -> BrandConfig {
        BrandConfig::default_minimal()
    }

    /// BC-2.01.005 postcondition 1 — exactly 31 layouts generated.
    #[test]
    fn test_bc_2_01_005_generates_exactly_31_layouts() {
        let _config = minimal_config();
        // Deferred until BrandConfig::default_minimal() is implemented.
        // Will call: generate_all_layouts(&config) and assert len == 31.
    }

    /// BC-2.01.005 postcondition 2 — SL-01..SL-11 have non-None `ooxml_type`.
    #[test]
    fn test_bc_2_01_005_standard_layouts_have_ooxml_types() {
        // Deferred until generate_all_layouts() is implemented.
        // Expected: layouts[0].ooxml_type == Some("title"), etc.
    }

    /// BC-2.01.005 postcondition 3 — CL-01..CL-20 names start with "SF ".
    #[test]
    fn test_bc_2_01_005_custom_layouts_have_sf_prefix() {
        // Deferred until generate_all_layouts() is implemented.
        // Expected: all layouts[11..31].name starts with "SF ".
    }

    /// BC-2.01.005 invariant 2 / AC-010 — Blank layout has zero placeholders.
    #[test]
    fn test_bc_2_01_005_blank_layout_has_zero_placeholders() {
        // SL-07 (index 6 in 0-based) must have placeholders.len() == 0.
        // Deferred until generate_all_layouts() is implemented.
    }

    /// BC-2.01.005 / AC-009 — CL-01 has `has_color_override` = true and bg = "dk2".
    #[test]
    fn test_bc_2_01_005_cl01_section_divider_dark_layout() {
        // layouts[11] (CL-01) must have has_color_override=true, color_override_bg=Some("dk2").
        // Deferred until generate_all_layouts() is implemented.
    }

    /// BC-2.01.005 / AC-009 — CL-11 has `has_color_override` = true.
    #[test]
    fn test_bc_2_01_005_cl11_end_slide_dark_layout() {
        // layouts[21] (CL-11) must have has_color_override=true.
        // Deferred until generate_all_layouts() is implemented.
    }

    /// BC-2.01.005 — layout indices are 1-based and sequential.
    #[test]
    fn test_bc_2_01_005_layout_indices_are_sequential() {
        // Expected: layouts[0].index == 1, layouts[30].index == 31.
        // Deferred until generate_all_layouts() is implemented.
    }

    /// BC-2.01.005 — standard layout `ooxml_type` values are correct ECMA-376 strings.
    #[test]
    fn test_bc_2_01_005_standard_layout_ooxml_type_values() {
        // Expected: ["title","obj","secHead","twoObj","twoColTx","titleOnly",
        //            "blank","objTx","picTx","vertTitleAndTx","vertTx"]
        // Deferred until generate_all_layouts() is implemented.
    }

    /// BC-2.01.005 invariant 4 / AC-010 — no placeholder has `accessibility_name` "Shape N".
    #[test]
    fn test_bc_2_01_005_no_shape_n_accessibility_names() {
        // Deferred until generate_all_layouts() is implemented.
    }
}
