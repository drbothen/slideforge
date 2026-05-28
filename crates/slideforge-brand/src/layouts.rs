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

    /// Build a BrandConfig directly (without calling default_minimal() which is
    /// todo!()) so layout tests can run purely against generate_all_layouts().
    fn minimal_config_direct() -> BrandConfig {
        BrandConfig {
            colors: crate::toml_schema::ColorConfig {
                acc1: Some("#3B82F6".to_owned()),
                dk1: Some("#1F2937".to_owned()),
                ..Default::default()
            },
            fonts: crate::toml_schema::FontConfig {
                heading: "Calibri".to_owned(),
                body: "Calibri".to_owned(),
            },
            logo: Some(crate::toml_schema::LogoConfig {
                path: "test-logo.png".to_owned(),
            }),
            footer: crate::toml_schema::FooterConfig {
                text: String::new(),
                show_slide_number: true,
                show_date: false,
            },
        }
    }

    // NOTE: FontConfig::default() and FooterConfig::default() are todo!().
    // We use direct construction above to avoid hitting those todo!() bodies.

    /// BC-2.01.005 postcondition 1 — exactly 31 layouts generated.
    #[test]
    fn test_bc_2_01_005_generate_31_layouts() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        assert_eq!(layouts.len(), 31, "must generate exactly 31 layouts");
    }

    /// BC-2.01.005 postcondition 2 — SL-01..SL-11 (indices 0–10) have non-None `ooxml_type`.
    #[test]
    fn test_bc_2_01_005_11_standard_layouts_present() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        let standard_names = [
            "Title Slide",
            "Title and Content",
            "Section Header",
            "Two Content",
            "Comparison",
            "Title Only",
            "Blank",
            "Content with Caption",
            "Picture with Caption",
            "Vertical Title and Text",
            "Vertical Text",
        ];
        for (i, expected_name) in standard_names.iter().enumerate() {
            assert_eq!(
                layouts[i].name.as_ref(),
                *expected_name,
                "layout index {i} (0-based) must be '{expected_name}'"
            );
        }
    }

    /// BC-2.01.005 postcondition 2 — standard layouts have correct ECMA-376 ooxml_type values.
    #[test]
    fn test_bc_2_01_005_standard_layouts_have_ooxml_types() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        let expected_types = [
            "title",
            "obj",
            "secHead",
            "twoObj",
            "twoColTx",
            "titleOnly",
            "blank",
            "objTx",
            "picTx",
            "vertTitleAndTx",
            "vertTx",
        ];
        for (i, expected_type) in expected_types.iter().enumerate() {
            assert_eq!(
                layouts[i].ooxml_type.as_deref(),
                Some(*expected_type),
                "layout index {i} must have ooxml_type = Some(\"{expected_type}\")"
            );
        }
    }

    /// BC-2.01.005 postcondition 3 — CL-01..CL-20 (indices 11–30) names start with "SF ".
    #[test]
    fn test_bc_2_01_005_20_sf_custom_layouts_present() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        let sf_names = [
            "SF Section Divider",
            "SF Stat Grid",
            "SF Quote",
            "SF Timeline",
            "SF Agenda",
            "SF TOC",
            "SF Bio",
            "SF Team Grid",
            "SF Comparison Table",
            "SF Full-Bleed Image",
            "SF End Slide",
            "SF Data",
            "SF Diagram",
            "SF Chart",
            "SF Map",
            "SF Risk Register",
            "SF Executive Summary",
            "SF Two Column",
            "SF Methodology",
            "SF Appendix",
        ];
        assert_eq!(
            layouts[11..].len(),
            20,
            "must have exactly 20 SF custom layouts"
        );
        for (i, expected_name) in sf_names.iter().enumerate() {
            let layout_idx = i + 11;
            assert_eq!(
                layouts[layout_idx].name.as_ref(),
                *expected_name,
                "layout index {layout_idx} must be '{expected_name}'"
            );
            assert!(
                layouts[layout_idx].name.starts_with("SF "),
                "layout index {layout_idx} name must start with 'SF '"
            );
        }
    }

    /// BC-2.01.005 invariant 2 / AC-010 — Blank layout (SL-07, index 6) has zero placeholders.
    #[test]
    fn test_bc_2_01_005_blank_layout_has_zero_placeholders() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        let blank = &layouts[6]; // SL-07 (0-based index 6)
        assert_eq!(blank.name.as_ref(), "Blank", "index 6 must be Blank layout");
        assert_eq!(
            blank.placeholders.len(),
            0,
            "Blank layout must have zero placeholders"
        );
    }

    /// BC-2.01.005 / AC-009 — CL-01 "SF Section Divider" (index 11) has dark color override.
    #[test]
    fn test_bc_2_01_005_cl01_section_divider_dark_layout() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        let cl01 = &layouts[11]; // CL-01 (0-based index 11)
        assert_eq!(
            cl01.name.as_ref(),
            "SF Section Divider",
            "index 11 must be SF Section Divider"
        );
        assert!(
            cl01.has_color_override,
            "SF Section Divider must have has_color_override = true"
        );
        assert_eq!(
            cl01.color_override_bg.as_deref(),
            Some("dk2"),
            "SF Section Divider bg override must be 'dk2'"
        );
        assert_eq!(
            cl01.color_override_tx.as_deref(),
            Some("lt1"),
            "SF Section Divider tx override must be 'lt1'"
        );
    }

    /// BC-2.01.005 / AC-009 — CL-11 "SF End Slide" (index 21) has dark color override.
    #[test]
    fn test_bc_2_01_005_cl11_end_slide_dark_layout() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        let cl11 = &layouts[21]; // CL-11 (0-based index 21)
        assert_eq!(
            cl11.name.as_ref(),
            "SF End Slide",
            "index 21 must be SF End Slide"
        );
        assert!(
            cl11.has_color_override,
            "SF End Slide must have has_color_override = true"
        );
        assert_eq!(
            cl11.color_override_bg.as_deref(),
            Some("dk2"),
            "SF End Slide bg override must be 'dk2'"
        );
        assert_eq!(
            cl11.color_override_tx.as_deref(),
            Some("lt1"),
            "SF End Slide tx override must be 'lt1'"
        );
    }

    /// BC-2.01.005 — layout indices are 1-based and sequential (1..=31).
    #[test]
    fn test_bc_2_01_005_layout_indices_are_sequential() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        assert_eq!(layouts[0].index, 1, "first layout must have index 1");
        assert_eq!(layouts[30].index, 31, "last layout must have index 31");
        for (i, layout) in layouts.iter().enumerate() {
            assert_eq!(
                layout.index,
                i + 1,
                "layout at position {i} must have 1-based index {}",
                i + 1
            );
        }
    }

    /// BC-2.01.005 — non-dark layouts do NOT have color overrides.
    #[test]
    fn test_bc_2_01_005_non_dark_layouts_have_no_color_override() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        for (i, layout) in layouts.iter().enumerate() {
            // Only indices 11 (CL-01) and 21 (CL-11) should be dark
            if i != 11 && i != 21 {
                assert!(
                    !layout.has_color_override,
                    "layout index {i} ('{}') must NOT have has_color_override = true",
                    layout.name
                );
            }
        }
    }

    /// BC-2.01.005 invariant 2 / AC-010 — every layout except Blank (SL-07) has at
    /// least one placeholder.
    #[test]
    fn test_bc_2_01_005_non_blank_layouts_have_at_least_one_placeholder() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        for (i, layout) in layouts.iter().enumerate() {
            if i == 6 {
                // Blank layout — zero placeholders is correct
                continue;
            }
            assert!(
                !layout.placeholders.is_empty(),
                "layout index {i} ('{}') must have at least one placeholder",
                layout.name
            );
        }
    }

    /// BC-2.01.005 invariant 4 / AC-010 — no placeholder uses "Shape N" as its
    /// accessibility name (DI-001 accessibility requirement).
    #[test]
    fn test_bc_2_01_005_layout_color_references_use_scheme_slots() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        for (i, layout) in layouts.iter().enumerate() {
            for ph in &layout.placeholders {
                let name = ph.accessibility_name.as_ref();
                // Must not match the forbidden "Shape N" pattern
                let is_shape_n = name.starts_with("Shape ")
                    && name[6..].chars().all(|c| c.is_ascii_digit());
                assert!(
                    !is_shape_n,
                    "layout index {i} ('{}') has a placeholder with forbidden name '{}' (must not be 'Shape N')",
                    layout.name,
                    name
                );
                // Accessibility name must be non-empty
                assert!(
                    !name.is_empty(),
                    "layout index {i} ('{}') has a placeholder with empty accessibility_name",
                    layout.name
                );
            }
        }
    }

    /// BC-2.01.005 — generation is deterministic (same config → same Vec ordering).
    #[test]
    fn test_bc_2_01_005_generation_deterministic() {
        let config = minimal_config_direct();
        let layouts_a = generate_all_layouts(&config);
        let layouts_b = generate_all_layouts(&config);
        assert_eq!(
            layouts_a.len(),
            layouts_b.len(),
            "generate_all_layouts must be deterministic in length"
        );
        for (i, (a, b)) in layouts_a.iter().zip(layouts_b.iter()).enumerate() {
            assert_eq!(
                a.name.as_ref(),
                b.name.as_ref(),
                "layout index {i}: name must be deterministic"
            );
            assert_eq!(
                a.index, b.index,
                "layout index {i}: index field must be deterministic"
            );
            assert_eq!(
                a.ooxml_type.as_deref(),
                b.ooxml_type.as_deref(),
                "layout index {i}: ooxml_type must be deterministic"
            );
            assert_eq!(
                a.has_color_override, b.has_color_override,
                "layout index {i}: has_color_override must be deterministic"
            );
        }
    }
}
