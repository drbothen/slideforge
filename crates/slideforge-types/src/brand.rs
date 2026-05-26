//! Brand configuration types.
//!
//! The [`Brand`] type carries the full brand configuration for a slideforge
//! deck: color palette, typography, and slide layout definitions. Brand data
//! is loaded by the `BrandProvider` plugin and applied during layout and export.
//!
//! This module provides skeleton types. Full brand schema definition (font
//! weights, palette derivation, layout geometry) is handled in the brand-spec
//! story.

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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BrandFonts {
    /// The heading typeface name (e.g., `"Calibri Light"`).
    pub heading: Arc<str>,
    /// The body typeface name.
    pub body: Arc<str>,
    /// The monospace typeface name (for code blocks).
    pub mono: Arc<str>,
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
}
