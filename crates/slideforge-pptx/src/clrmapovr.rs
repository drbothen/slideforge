//! `<p:clrMapOvr>` injection for dark-themed slide layouts.
//!
//! [`ClrMapOvrInjector`] determines whether a slide requires
//! `<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>` in its slide XML.
//! Dark status is determined by `SlideLayoutDef.has_color_override` on the
//! layout resolved for the slide — never by a hardcoded index list.
//!
//! ## ADR-015 §A.4
//!
//! Dark layouts: index 12 (`section_divider`, `has_color_override = true`) and
//! index 22 (`end`, `has_color_override = true`).
//!
//! ## STORY-038 task: AC-006
//!
//! The full end-to-end integration test (deck with `section_divider` slide →
//! export → ZIP → slide XML contains `<p:clrMapOvr>`) is covered by
//! AC-006. This module provides the type that makes the dark-layout determination
//! testable in isolation.

use slideforge_brand::BrandTemplate;

/// Determines whether a slide at a given layout index requires `<p:clrMapOvr>`.
///
/// Reads `SlideLayoutDef.has_color_override` from `brand_template.layouts[layout_index]`.
/// Returns `false` if `layout_index` is out of range (defensive).
pub struct ClrMapOvrInjector;

impl ClrMapOvrInjector {
    /// Return `true` if the slide at `layout_index` requires `<p:clrMapOvr>`.
    ///
    /// `layout_index` is the 0-based index into `brand_template.layouts`.
    /// Returns `false` if the index is out of range.
    #[must_use]
    pub fn needs_clr_map_ovr(brand_template: &BrandTemplate, layout_index: usize) -> bool {
        brand_template
            .layouts
            .get(layout_index)
            .is_some_and(|l| l.has_color_override)
    }
}
