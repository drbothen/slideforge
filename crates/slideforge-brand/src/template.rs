//! Brand template types extracted from OOXML `.pptx` / `.docx` files.
//!
//! [`BrandTemplate`] is the central struct produced by [`crate::loader::BrandLoader`].
//! It carries all 12 OOXML scheme color slots, heading/body font names, an optional
//! logo asset, optional footer text, and the list of layout names discovered in the
//! template.

use std::sync::Arc;

/// The ECMA-376 OOXML scheme color slot names, in mandatory sequential order.
///
/// Position 0 is `dk1`, position 11 is `folHlink`. This order is enforced by
/// BC-2.01.001 invariant 1 and DI-015.
pub const COLOR_SLOT_NAMES: [&str; 12] = [
    "dk1", "lt1", "dk2", "lt2", "acc1", "acc2", "acc3", "acc4", "acc5", "acc6", "hlink", "folHlink",
];

/// The resolved or unresolved color value stored in an OOXML theme color slot.
///
/// OOXML theme color slots can contain three kinds of color references:
///
/// - [`ColorValue::Hex`]: an absolute hex color (from `<a:srgbClr>` or `<a:sysClr lastClr>`).
///   Always in `"#RRGGBB"` format with uppercase hex digits.
/// - [`ColorValue::SchemeRef`]: a relative scheme color reference (from `<a:schemeClr val="...">`,
///   e.g., `"dk1"`, `"accent1"`).  The slot value is **not** an absolute hex and cannot be
///   resolved without a rendering context.  Stored as the lowercase `val` attribute string.
///
/// The `Hex` variant is the common case for well-formed brand templates.
/// The `SchemeRef` variant indicates a self-referential or relative color that was
/// extracted with a `tracing::warn!` and should be reviewed in brand.toml.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ColorValue {
    /// An absolute hex color value, e.g., `"#003087"`.
    ///
    /// Always `#RRGGBB` (7 characters, uppercase hex).
    Hex(Arc<str>),
    /// A scheme-relative color reference, e.g., `"dk1"` from `<a:schemeClr val="dk1">`.
    ///
    /// Cannot be resolved to an absolute hex without a rendering context.
    /// The string is the lowercase `val` attribute from `<a:schemeClr>`.
    SchemeRef(Arc<str>),
}

impl ColorValue {
    /// Returns `true` if this value is an absolute hex color (i.e., can be used directly
    /// as a CSS/OOXML color value without further resolution).
    #[must_use]
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Hex(_))
    }

    /// Returns the hex color string if this is a [`ColorValue::Hex`] variant,
    /// or `None` if it is a [`ColorValue::SchemeRef`].
    ///
    /// The returned string is always in `"#RRGGBB"` format (uppercase hex digits).
    #[must_use]
    pub fn as_hex(&self) -> Option<&str> {
        match self {
            Self::Hex(h) => Some(h.as_ref()),
            Self::SchemeRef(_) => None,
        }
    }

    /// Returns the scheme reference string if this is a [`ColorValue::SchemeRef`] variant,
    /// or `None` if it is a [`ColorValue::Hex`].
    #[must_use]
    pub fn as_scheme_ref(&self) -> Option<&str> {
        match self {
            Self::SchemeRef(r) => Some(r.as_ref()),
            Self::Hex(_) => None,
        }
    }
}

/// A single OOXML theme color slot.
///
/// Each slot has a canonical ECMA-376 name (e.g., `"dk1"`) and a [`ColorValue`]
/// that is either a resolved absolute hex (`#RRGGBB`) or an unresolved scheme
/// color reference.
///
/// Use [`ColorSlot::hex`] for the common case of an absolute color.
/// Use [`ColorSlot::is_resolved`] to detect whether the slot can be used directly.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColorSlot {
    /// The OOXML slot name (e.g., `"dk1"`, `"acc1"`, `"hlink"`).
    pub name: Arc<str>,
    /// The color value — either an absolute hex or a relative scheme reference.
    ///
    /// Most OOXML brand templates produce [`ColorValue::Hex`] values.
    /// [`ColorValue::SchemeRef`] only appears when the source template used a
    /// self-referential `<a:schemeClr>` element.
    pub value: ColorValue,
}

impl ColorSlot {
    /// Returns the hex color string if the slot holds an absolute hex value.
    ///
    /// Returns `None` when the slot holds a [`ColorValue::SchemeRef`] (a relative
    /// scheme color that cannot be resolved without a rendering context).
    ///
    /// The returned string is always in `"#RRGGBB"` format (uppercase hex digits).
    #[must_use]
    pub fn hex(&self) -> Option<&str> {
        self.value.as_hex()
    }

    /// Returns `true` if the slot holds a resolved absolute hex color.
    ///
    /// Returns `false` when the slot holds a scheme color reference that cannot be
    /// used directly without resolving the reference against the active theme.
    #[must_use]
    pub fn is_resolved(&self) -> bool {
        self.value.is_resolved()
    }
}

/// Brand font configuration extracted from the OOXML theme.
///
/// Heading font maps to `<a:majorFont>` and body font maps to `<a:minorFont>`
/// in the OOXML `theme1.xml`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BrandFonts {
    /// The heading (major) font typeface name (e.g., `"Calibri Light"`).
    pub heading: Arc<str>,
    /// The body (minor) font typeface name (e.g., `"Calibri"`).
    pub body: Arc<str>,
}

/// A logo image extracted from the slide master relationships, or a deferred
/// path reference for synthesized brands.
///
/// ## Variants
///
/// - [`LogoAsset::Loaded`]: bytes already in memory (from `.pptx`/`.docx` extraction).
/// - [`LogoAsset::Deferred`]: path recorded during synthesis; bytes loaded on demand
///   by the PPTX exporter (STORY-037) when building the ZIP package. This keeps
///   [`BrandSynthesizer::synthesize`] pure (no filesystem I/O).
///
/// The logo is optional — if no `[logo]` section in `brand.toml` and no image
/// relationship in the source template, [`BrandTemplate::logo`] is `None`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogoAsset {
    /// Logo bytes already loaded into memory.
    ///
    /// Produced by [`crate::loader::BrandLoader`] when extracting from
    /// a `.pptx` or `.docx` template (STORY-022).
    Loaded {
        /// Raw image bytes read from the ZIP archive.
        bytes: Vec<u8>,
        /// MIME type of the image (e.g., `"image/png"`, `"image/jpeg"`).
        media_type: Arc<str>,
        /// The ZIP-internal path of the image file (e.g., `"ppt/media/image1.png"`).
        original_path: Arc<str>,
    },
    /// Logo path recorded for deferred loading by the PPTX exporter.
    ///
    /// Produced by [`crate::synthesizer::BrandSynthesizer::synthesize`] when
    /// the `[logo]` section declares a path. The exporter reads the bytes from
    /// the filesystem when building the output ZIP package (STORY-037).
    Deferred {
        /// File system path to the logo image (from `brand.toml` `[logo].path`).
        path: Arc<str>,
    },
}

impl LogoAsset {
    /// Returns the raw bytes if this asset is [`LogoAsset::Loaded`], or `None`
    /// if it is [`LogoAsset::Deferred`] (bytes not yet read from disk).
    #[must_use]
    pub fn loaded_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Loaded { bytes, .. } => Some(bytes),
            Self::Deferred { .. } => None,
        }
    }

    /// Returns the file path for a [`LogoAsset::Deferred`] asset, or `None`
    /// if this is a [`LogoAsset::Loaded`] asset.
    #[must_use]
    pub fn deferred_path(&self) -> Option<&str> {
        match self {
            Self::Deferred { path } => Some(path.as_ref()),
            Self::Loaded { .. } => None,
        }
    }

    /// Returns `true` if this asset's bytes are already in memory.
    #[must_use]
    pub fn is_loaded(&self) -> bool {
        matches!(self, Self::Loaded { .. })
    }
}

/// OOXML master and layout ID constraints (BC-2.01.005 invariant 3 / AC-011).
///
/// Slide master IDs start at `2^31`. Layout IDs start at `2^31 + 1` and
/// increment by 1 per layout. Slide IDs in generated decks start at 256.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MasterIds {
    /// The slide master ID: always `2^31 = 2_147_483_648`.
    pub master_id: u32,

    /// The starting layout ID: always `2^31 + 1 = 2_147_483_649`.
    pub layout_id_start: u32,

    /// The starting slide ID for generated slides: always `256`.
    pub slide_id_start: u32,
}

impl Default for MasterIds {
    fn default() -> Self {
        Self {
            master_id: 2u32.pow(31),
            layout_id_start: 2u32.pow(31) + 1,
            slide_id_start: 256,
        }
    }
}

/// The complete brand template extracted from a `.pptx` or `.docx` file,
/// or synthesized from a `brand.toml` (STORY-023).
///
/// This struct is produced by both [`crate::loader::BrandLoader`] (STORY-022)
/// and [`crate::synthesizer::BrandSynthesizer`] (STORY-023), and consumed by
/// the PPTX exporter (STORY-037).
///
/// ## Color Slot Invariant
///
/// The `colors` array always has exactly 12 entries in ECMA-376 sequential order
/// (dk1, lt1, dk2, lt2, acc1–acc6, hlink, folHlink). If the source template has
/// fewer than 12 color slots, the missing slots are inferred and
/// [`crate::error::BrandError::MissingColorSlot`] warnings are emitted. This is
/// guaranteed by DI-015 (BC-2.01.001 invariant 1).
///
/// ## Layout Invariant (STORY-023)
///
/// For synthesized brands, `layouts` always contains exactly 31 entries
/// (BC-2.01.005 postcondition 1). For loaded brands (from .pptx/.docx), the
/// count reflects the source template.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BrandTemplate {
    /// All 12 OOXML scheme color slots in ECMA-376 sequential order.
    ///
    /// Invariant: `colors.len() == 12` always.
    pub colors: [ColorSlot; 12],

    /// Heading and body font configuration.
    pub fonts: BrandFonts,

    /// Optional logo asset extracted from the slide master relationships.
    ///
    /// `None` if no image relationship was found (EC-005 — not an error for
    /// loaded brands). For synthesized brands, a missing logo is a fatal error
    /// (AC-004, BC-2.01.002 edge case EC-005).
    pub logo: Option<LogoAsset>,

    /// Optional footer text (from the deck-level footer field).
    pub footer_text: Option<Arc<str>>,

    /// Slide layout XML names discovered in `ppt/slideLayouts/slideLayout*.xml`.
    ///
    /// For loaded brands (STORY-022): ZIP-internal paths.
    /// For synthesized brands (STORY-023): not used — `layouts` field is populated instead.
    pub layout_names: Vec<Arc<str>>,

    /// Structured layout definitions for all 31 slide layouts.
    ///
    /// Populated by [`crate::synthesizer::BrandSynthesizer`] (STORY-023).
    /// Empty `Vec` for brands loaded from `.pptx`/`.docx` until STORY-037
    /// adds layout extraction.
    ///
    /// Invariant for synthesized brands: `layouts.len() == 31` always.
    pub layouts: Vec<crate::layouts::SlideLayoutDef>,

    /// Serialized XML bytes for `notesMaster1.xml` (AC-012).
    ///
    /// Always populated for synthesized brands. Empty for loaded brands
    /// until STORY-040 adds master extraction.
    pub notes_master_stub: Vec<u8>,

    /// Serialized XML bytes for `handoutMaster1.xml` (AC-012).
    ///
    /// Always populated for synthesized brands. Empty for loaded brands
    /// until STORY-040 adds master extraction.
    pub handout_master_stub: Vec<u8>,

    /// OOXML master and layout ID constraints (AC-011, BC-2.01.005 invariant 3).
    pub master_ids: MasterIds,

    /// `[Content_Types].xml` registration fragment for all 31 layouts (AC-013).
    ///
    /// Contains one `<Override PartName="...">` entry per layout.
    /// Populated by the synthesizer; empty for loaded brands.
    pub content_types_layout_entries: Arc<str>,
}

impl BrandTemplate {
    /// Returns the [`ColorSlot`] at a given ECMA-376 position index (0–11).
    ///
    /// # Panics
    ///
    /// Panics if `index >= 12`. All indices 0–11 are always valid because the
    /// `colors` array is always fully populated.
    #[must_use]
    pub fn color_at(&self, index: usize) -> &ColorSlot {
        &self.colors[index]
    }

    /// Returns the [`ColorSlot`] with the given OOXML slot name.
    ///
    /// Returns `None` only if `name` is not a recognized ECMA-376 slot name.
    /// Because all 12 slots are always present, any valid slot name will match.
    #[must_use]
    pub fn color_by_name(&self, name: &str) -> Option<&ColorSlot> {
        self.colors.iter().find(|s| s.name.as_ref() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_color_slot(name: &str, hex: &str) -> ColorSlot {
        ColorSlot {
            name: Arc::from(name),
            value: ColorValue::Hex(Arc::from(hex)),
        }
    }

    fn make_all_slots() -> [ColorSlot; 12] {
        [
            make_color_slot("dk1", "#000000"),
            make_color_slot("lt1", "#FFFFFF"),
            make_color_slot("dk2", "#003087"),
            make_color_slot("lt2", "#F5F5F5"),
            make_color_slot("acc1", "#0066CC"),
            make_color_slot("acc2", "#FF6B35"),
            make_color_slot("acc3", "#28A745"),
            make_color_slot("acc4", "#FFC107"),
            make_color_slot("acc5", "#6F42C1"),
            make_color_slot("acc6", "#17A2B8"),
            make_color_slot("hlink", "#0000EE"),
            make_color_slot("folHlink", "#551A8B"),
        ]
    }

    /// Helper: construct a minimal `BrandTemplate` for tests.
    fn make_template() -> BrandTemplate {
        BrandTemplate {
            colors: make_all_slots(),
            fonts: BrandFonts {
                heading: Arc::from("Calibri Light"),
                body: Arc::from("Calibri"),
            },
            logo: None,
            footer_text: None,
            layout_names: vec![],
            layouts: vec![],
            notes_master_stub: vec![],
            handout_master_stub: vec![],
            master_ids: MasterIds::default(),
            content_types_layout_entries: Arc::from(""),
        }
    }

    /// BC-2.01.001 postcondition 1 — `BrandTemplate` can be constructed with all fields.
    #[test]
    fn test_bc_2_01_001_brand_template_default_construction() {
        let template = make_template();
        assert_eq!(template.colors.len(), 12);
        assert!(template.logo.is_none());
        assert!(template.footer_text.is_none());
    }

    /// BC-2.01.001 invariant 1 — colors array always has exactly 12 entries.
    #[test]
    fn test_bc_2_01_001_invariant_always_12_color_slots() {
        let template = make_template();
        // The array type [ColorSlot; 12] enforces this at compile time, but we
        // also assert it at runtime to make it load-bearing per TD-VSDD-059.
        assert_eq!(
            template.colors.len(),
            12,
            "invariant DI-015: always 12 color slots"
        );
    }

    /// AC-011 — `MasterIds::default()` has correct OOXML-mandated values.
    #[test]
    fn test_bc_2_01_005_master_ids_default_values() {
        let ids = MasterIds::default();
        assert_eq!(ids.master_id, 2u32.pow(31), "master_id must be 2^31");
        assert_eq!(
            ids.layout_id_start,
            2u32.pow(31) + 1,
            "layout_id_start must be 2^31 + 1"
        );
        assert_eq!(ids.slide_id_start, 256, "slide_id_start must be 256");
    }

    /// BC-2.01.001 — color slot names match ECMA-376 sequential order.
    #[test]
    fn test_bc_2_01_001_color_order_matches_ecma_376() {
        let expected = COLOR_SLOT_NAMES;
        let slots = make_all_slots();
        for (i, expected_name) in expected.iter().enumerate() {
            assert_eq!(
                slots[i].name.as_ref(),
                *expected_name,
                "position {i}: expected slot '{expected_name}', got '{}'",
                slots[i].name
            );
        }
    }

    /// BC-2.01.001 — `color_by_name` returns correct slot.
    #[test]
    fn test_bc_2_01_001_color_by_name_lookup() {
        let template = make_template();
        let slot = template
            .color_by_name("acc1")
            .expect("acc1 must be present");
        assert_eq!(slot.hex(), Some("#0066CC"));
    }

    /// BC-2.01.001 — `color_by_name` returns None for unknown slot.
    #[test]
    fn test_bc_2_01_001_color_by_name_unknown_returns_none() {
        let template = make_template();
        assert!(template.color_by_name("nonexistent").is_none());
    }

    /// BC-2.01.001 AC-005 — logo is Option<LogoAsset>.
    #[test]
    fn test_bc_2_01_001_logo_asset_construction() {
        let logo = LogoAsset::Loaded {
            bytes: vec![0x89, 0x50, 0x4E, 0x47],
            media_type: Arc::from("image/png"),
            original_path: Arc::from("ppt/media/image1.png"),
        };
        assert_eq!(logo.loaded_bytes().map(<[u8]>::len), Some(4));
        assert!(logo.is_loaded());
        assert_eq!(logo.deferred_path(), None);
    }

    /// F5 — `LogoAsset::Deferred` carries the path and has no loaded bytes.
    #[test]
    fn test_logo_asset_deferred_variant() {
        let logo = LogoAsset::Deferred {
            path: Arc::from("brand.assets/logo.png"),
        };
        assert!(!logo.is_loaded());
        assert_eq!(logo.deferred_path(), Some("brand.assets/logo.png"));
        assert_eq!(logo.loaded_bytes(), None);
    }

    /// BC-2.01.001 AC-006 — `layout_names` stored as Vec<Arc<str>> using ZIP-internal paths.
    ///
    /// `layout_names` stores the ZIP-internal path of each slide layout XML file
    /// (e.g., `"ppt/slideLayouts/slideLayout1.xml"`), not friendly semantic names.
    /// These paths are produced by the loader's layout discovery step and consumed
    /// by STORY-023 for layout-to-taxonomy mapping.
    #[test]
    fn test_bc_2_01_001_layout_names_stored_correctly() {
        let template = BrandTemplate {
            colors: make_all_slots(),
            fonts: BrandFonts {
                heading: Arc::from("Arial"),
                body: Arc::from("Arial"),
            },
            logo: None,
            footer_text: None,
            layout_names: vec![
                Arc::from("ppt/slideLayouts/slideLayout1.xml"),
                Arc::from("ppt/slideLayouts/slideLayout2.xml"),
                Arc::from("ppt/slideLayouts/slideLayout3.xml"),
            ],
            layouts: vec![],
            notes_master_stub: vec![],
            handout_master_stub: vec![],
            master_ids: MasterIds::default(),
            content_types_layout_entries: Arc::from(""),
        };
        assert_eq!(template.layout_names.len(), 3);
        // Values are ZIP-internal paths, not friendly semantic names.
        assert_eq!(
            template.layout_names[0].as_ref(),
            "ppt/slideLayouts/slideLayout1.xml"
        );
        assert_eq!(
            template.layout_names[2].as_ref(),
            "ppt/slideLayouts/slideLayout3.xml"
        );
    }

    /// DI-015 — `COLOR_SLOT_NAMES` constant has exactly 12 entries.
    #[test]
    fn test_bc_2_01_001_color_slot_names_constant_length() {
        assert_eq!(
            COLOR_SLOT_NAMES.len(),
            12,
            "ECMA-376 defines exactly 12 theme color slots"
        );
    }

    // ─── FINDING-001: ColorValue enum tests ──────────────────────────────────

    /// FINDING-001 — `ColorValue::Hex` `is_resolved` returns true.
    #[test]
    fn test_finding_001_color_value_hex_is_resolved() {
        let v = ColorValue::Hex(Arc::from("#003087"));
        assert!(v.is_resolved(), "Hex variant must be resolved");
        assert_eq!(v.as_hex(), Some("#003087"));
        assert_eq!(v.as_scheme_ref(), None);
    }

    /// FINDING-001 — `ColorValue::SchemeRef` `is_resolved` returns false.
    #[test]
    fn test_finding_001_color_value_scheme_ref_not_resolved() {
        let v = ColorValue::SchemeRef(Arc::from("dk1"));
        assert!(!v.is_resolved(), "SchemeRef variant must NOT be resolved");
        assert_eq!(v.as_hex(), None);
        assert_eq!(v.as_scheme_ref(), Some("dk1"));
    }

    /// FINDING-001 — `ColorSlot::hex()` returns Some for Hex variant.
    #[test]
    fn test_finding_001_color_slot_hex_method_hex_variant() {
        let slot = ColorSlot {
            name: Arc::from("dk2"),
            value: ColorValue::Hex(Arc::from("#003087")),
        };
        assert_eq!(slot.hex(), Some("#003087"));
        assert!(slot.is_resolved());
    }

    /// FINDING-001 — `ColorSlot::hex()` returns None for `SchemeRef` variant.
    #[test]
    fn test_finding_001_color_slot_hex_method_scheme_ref_variant() {
        let slot = ColorSlot {
            name: Arc::from("dk1"),
            value: ColorValue::SchemeRef(Arc::from("dk1")),
        };
        assert_eq!(
            slot.hex(),
            None,
            "SchemeRef slot must return None from hex()"
        );
        assert!(!slot.is_resolved(), "SchemeRef slot must not be resolved");
    }
}
