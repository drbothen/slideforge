//! Brand configuration types.
//!
//! The [`Brand`] type carries the full brand configuration for a slideforge
//! deck: color palette, typography, and slide layout definitions. Brand data
//! is loaded by the `BrandProvider` plugin and applied during layout and export.
//!
//! This module provides the production brand schema: palette, typography
//! (including [`BrandFonts::font_size_emu`] for brand-aware em resolution),
//! and named slide layout definitions.

use std::sync::Arc;

use crate::emu::Emu;
use crate::span::SourceSpan;

/// A brand color palette entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BrandPalette {
    /// The primary brand color as a CSS hex string (e.g., `"#003087"`).
    pub primary: Arc<str>,
    /// The secondary brand color.
    pub secondary: Arc<str>,
    /// The accent color.
    pub accent: Arc<str>,
    /// The neutral / background color.
    pub neutral: Arc<str>,
}

/// Brand font configuration.
///
/// ## STORY-074 — brand-aware em resolution
///
/// The `font_size_emu` field carries the brand's body font size expressed in
/// EMU (English Metric Units). It is used by the layout engine to resolve
/// `ShapeUnit::Em` measurements: `em_emu = em_milliems * font_size_emu / 1_000`
/// (BC-3.04.001 Postcondition 2).
///
/// Default: `457_200` EMU (36pt body font = 0.5 inch at `914_400` EMU/inch), matching the historical
/// `DEFAULT_EM_IN_EMU` constant in `slideforge-layout::shapes`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BrandFonts {
    /// The heading typeface name (e.g., `"Calibri Light"`).
    pub heading: Arc<str>,
    /// The body typeface name.
    pub body: Arc<str>,
    /// The monospace typeface name (for code blocks).
    pub mono: Arc<str>,
    /// The body font size expressed in EMU, used to resolve `ShapeUnit::Em`
    /// measurements at layout time (BC-3.04.001 PC-2).
    ///
    /// `1em = font_size_emu` EMU. The formula for a `ShapeUnit::Em(milliem)` is:
    /// `emu = milliem * font_size_emu / 1_000` (integer division, no `f64`).
    ///
    /// Default: `457_200` (36pt body font at `914_400` EMU/inch, matching the
    /// historical `DEFAULT_EM_IN_EMU` constant in `slideforge-layout::shapes`).
    pub font_size_emu: i64,
}

impl Default for BrandFonts {
    /// Returns a `BrandFonts` with generic typeface names and the default
    /// `font_size_emu` of `457_200` (36pt body font = 0.5 inch at `914_400` EMU/inch).
    fn default() -> Self {
        Self {
            heading: Arc::from("Calibri Light"),
            body: Arc::from("Calibri"),
            mono: Arc::from("Courier New"),
            font_size_emu: 457_200,
        }
    }
}

/// A named slide layout definition.
///
/// Layout definitions describe the geometric constraints for each slide type:
/// where placeholders are positioned, their sizes, and their semantic roles.
/// Full geometric fields are added in the layout-spec story.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LayoutDefinition {
    /// The layout name / slide-type keyword this layout applies to.
    pub name: Arc<str>,
    /// Width of the slide canvas this layout is designed for.
    pub canvas_width: Emu,
    /// Height of the slide canvas this layout is designed for.
    pub canvas_height: Emu,
    /// Source location (from the `.toml` brand file).
    pub span: SourceSpan,
}

/// The complete brand configuration for a deck.
///
/// `Brand` is loaded by the `BrandProvider` plugin from a `.toml` brand file
/// and applied during layout and export. The brand data is immutable once
/// loaded.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Brand {
    /// The brand name (e.g., the organization name).
    pub name: Arc<str>,
    /// The color palette.
    pub palette: BrandPalette,
    /// The font configuration.
    pub fonts: BrandFonts,
    /// Named slide layout definitions.
    pub layouts: Vec<LayoutDefinition>,
    /// Source location.
    pub span: SourceSpan,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emu::{SLIDE_HEIGHT, SLIDE_WIDTH};
    use std::sync::Arc;

    fn make_brand() -> Brand {
        Brand {
            name: Arc::from("acme"),
            palette: BrandPalette {
                primary: Arc::from("#003087"),
                secondary: Arc::from("#0066CC"),
                accent: Arc::from("#FF6B35"),
                neutral: Arc::from("#F5F5F5"),
            },
            fonts: BrandFonts {
                heading: Arc::from("Calibri Light"),
                body: Arc::from("Calibri"),
                mono: Arc::from("Courier New"),
                font_size_emu: 457_200,
            },
            layouts: vec![],
            span: SourceSpan::default(),
        }
    }

    #[test]
    fn test_bc_1_01_brand_fields() {
        let brand = make_brand();
        assert_eq!(brand.name.as_ref(), "acme");
        assert_eq!(brand.palette.primary.as_ref(), "#003087");
        assert_eq!(brand.fonts.heading.as_ref(), "Calibri Light");
    }

    #[test]
    fn test_bc_1_01_brand_clone() {
        let brand = make_brand();
        let brand2 = brand.clone();
        assert_eq!(brand, brand2);
    }

    #[test]
    fn test_bc_1_01_brand_hash() {
        use std::collections::HashSet;
        let brand = make_brand();
        let mut set: HashSet<Brand> = HashSet::new();
        set.insert(brand);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_01_brand_debug() {
        let brand = make_brand();
        let s = format!("{brand:?}");
        assert!(s.contains("Brand"));
    }

    #[test]
    fn test_bc_1_01_layout_definition_fields() {
        let layout = LayoutDefinition {
            name: Arc::from("title"),
            canvas_width: SLIDE_WIDTH,
            canvas_height: SLIDE_HEIGHT,
            span: SourceSpan::default(),
        };
        assert_eq!(layout.name.as_ref(), "title");
        assert_eq!(layout.canvas_width, SLIDE_WIDTH);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // STORY-074 — AC-003: BrandFonts.font_size_emu field + Default impl
    // (BC-3.04.001 Postcondition 2 / STORY-074 AC-003)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-003 / STORY-074 — `BrandFonts::default()` sets `font_size_emu` to `457_200`.
    ///
    /// `457_200` EMU = 0.5 inch at `914_400` EMU/inch, matching the historical
    /// `DEFAULT_EM_IN_EMU` constant in `slideforge-layout::shapes`. This ensures
    /// backward compatibility: decks that never set a brand font size continue to
    /// resolve em units with the same value as before STORY-074.
    ///
    /// Red Gate: before STORY-074 implementation this test fails because
    /// `BrandFonts` did not implement `Default`.
    ///
    /// AC-003 traces to: BC-3.04.001 Invariant 2 (integer EMU, no regressions).
    #[test]
    fn test_bc_3_04_001_ac003_brand_fonts_default_font_size_emu_is_457200() {
        let fonts = BrandFonts::default();
        assert_eq!(
            fonts.font_size_emu, 457_200,
            "BrandFonts::default().font_size_emu must be 457_200 (STORY-074 AC-003)"
        );
    }

    /// AC-003 / STORY-074 — `BrandFonts` with non-default `font_size_emu` round-trips
    /// through `Debug + Clone + PartialEq + Eq + Hash` (comemo compatibility per DI-010).
    ///
    /// Red Gate: before STORY-074 implementation the `font_size_emu` field does
    /// not exist, so this test fails to compile.
    #[test]
    fn test_bc_3_04_001_ac003_brand_fonts_font_size_emu_derives_traits() {
        use std::collections::HashSet;
        let fonts = BrandFonts {
            heading: Arc::from("Calibri Light"),
            body: Arc::from("Calibri"),
            mono: Arc::from("Courier New"),
            font_size_emu: 609_600, // 48pt — non-default
        };
        // Clone
        let fonts2 = fonts.clone();
        assert_eq!(
            fonts, fonts2,
            "BrandFonts must implement PartialEq for Clone equality"
        );
        // Hash
        let mut set: HashSet<BrandFonts> = HashSet::new();
        set.insert(fonts2);
        assert_eq!(
            set.len(),
            1,
            "BrandFonts must be hashable (comemo / DI-010)"
        );
        // Debug
        let s = format!("{fonts:?}");
        assert!(
            s.contains("609600"),
            "Debug output must include font_size_emu value"
        );
    }

    /// AC-003 / STORY-074 — `BrandFonts` with `font_size_emu: 609_600` is NOT equal
    /// to `BrandFonts` with `font_size_emu: 457_200`.
    ///
    /// Ensures the field participates in `PartialEq` (not accidentally ignored).
    ///
    /// Red Gate: before STORY-074 implementation the field does not exist.
    #[test]
    fn test_bc_3_04_001_ac003_brand_fonts_different_font_size_emu_not_equal() {
        let fonts_36pt = BrandFonts {
            heading: Arc::from("Calibri Light"),
            body: Arc::from("Calibri"),
            mono: Arc::from("Courier New"),
            font_size_emu: 457_200,
        };
        let fonts_48pt = BrandFonts {
            heading: Arc::from("Calibri Light"),
            body: Arc::from("Calibri"),
            mono: Arc::from("Courier New"),
            font_size_emu: 609_600,
        };
        assert_ne!(
            fonts_36pt, fonts_48pt,
            "BrandFonts with different font_size_emu values must not be equal"
        );
    }

    /// AC-003 / STORY-074 — `make_brand()` helper produces a brand whose
    /// `fonts.font_size_emu` equals `457_200` (backward-compatible default).
    ///
    /// This guards all pre-STORY-074 tests that construct `Brand` manually:
    /// they now set `font_size_emu: 457_200` and must match the expected default.
    ///
    /// Red Gate: before STORY-074 implementation the field does not exist on
    /// `BrandFonts`, so structural construction fails to compile.
    #[test]
    fn test_bc_3_04_001_ac003_make_brand_helper_has_default_font_size_emu() {
        let brand = make_brand();
        assert_eq!(
            brand.fonts.font_size_emu, 457_200,
            "make_brand() helper must carry font_size_emu: 457_200 (backward compat)"
        );
    }
}
