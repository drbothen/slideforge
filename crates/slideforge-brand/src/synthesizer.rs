//! Brand synthesizer — `brand.toml` → [`BrandTemplate`] (BC-2.01.002, BC-2.01.004, BC-2.01.005).
//!
//! [`BrandSynthesizer`] is the primary entry point for synthesizing a complete
//! [`BrandTemplate`] from a `brand.toml` file. It implements two separate
//! concerns per Architecture Compliance Rule 2 (STORY-023):
//!
//! 1. **Effectful I/O** — [`BrandSynthesizer::load_from_toml`]: reads the
//!    `brand.toml` file from the filesystem.
//! 2. **Pure synthesis** — [`BrandSynthesizer::synthesize`]: pure function;
//!    takes a [`BrandConfig`] reference, performs color inference, layout
//!    generation, and XML serialization, and returns a [`BrandTemplate`].
//!
//! The separation means `synthesize` is directly unit-testable without any
//! filesystem dependency (AC-007 determinism test, VP-012 proptest).
//!
//! ## `BrandProvider` trait implementation
//!
//! `BrandSynthesizer` implements the `slideforge_plugin_api::BrandProvider`
//! trait, which provides the unified plugin API entry point via
//! `BrandProvider::load(source: &BrandSource)`.
//!
//! ## Error taxonomy
//!
//! | Condition | Error | Severity |
//! |-----------|-------|---------|
//! | `brand.toml` not found | [`BrandError::FileNotFound`] | fatal (exit 4) |
//! | Invalid TOML syntax | [`BrandError::ParseError`] | fatal (exit 4) |
//! | Missing `[logo]` section | `E-BRD-001` via `BrandError::LogoRequired` | fatal (exit 4) |
//! | Absent color slot | [`BrandError::MissingColorSlot`] | warning (exit 0) |

use std::sync::Arc;

use slideforge_plugin_api::BrandProvider;
use slideforge_plugin_api::BrandSource;
use slideforge_types::Brand;

use crate::error::BrandError;
use crate::inference;
use crate::layout_xml::{
    HANDOUT_MASTER_STUB, NOTES_MASTER_STUB, generate_content_types_layout_entries,
};
use crate::layouts::generate_all_layouts;
use crate::template::{
    BrandFonts, BrandTemplate, COLOR_SLOT_NAMES, ColorSlot, ColorValue, LogoAsset, MasterIds,
};
use crate::toml_schema::BrandConfig;

// ─── BrandSynthesizer ─────────────────────────────────────────────────────────

/// Synthesizes a [`BrandTemplate`] from a `brand.toml` file or a [`BrandConfig`].
///
/// The synthesizer is the single implementation of the `BrandProvider` trait
/// for TOML-based brands. It also accepts PPTX/DOCX sources by delegating to
/// [`crate::loader::BrandLoader`].
///
/// ## Usage
///
/// ```no_run
/// use slideforge_brand::synthesizer::BrandSynthesizer;
///
/// // Load from file (effectful path):
/// let template = BrandSynthesizer::load_from_toml("path/to/brand.toml")
///     .expect("brand load failed");
///
/// // Synthesize from already-parsed config (pure path, useful for testing):
/// use slideforge_brand::toml_schema::BrandConfig;
/// let config = BrandConfig::default();
/// let result = BrandSynthesizer::synthesize(&config);
/// ```
#[derive(Debug, Default)]
pub struct BrandSynthesizer;

impl BrandSynthesizer {
    /// Load a [`BrandTemplate`] from a `brand.toml` file at `path`.
    ///
    /// This is the effectful entry point. It reads the file, parses TOML,
    /// and calls [`BrandSynthesizer::synthesize`].
    ///
    /// # Errors
    ///
    /// - [`BrandError::TomlReadError`] if `path` does not exist or cannot be read.
    /// - [`BrandError::TomlParseError`] if the file content is not valid TOML.
    /// - Propagates all errors from [`BrandSynthesizer::synthesize`].
    ///
    /// On success, returns `(template, warnings)` — see [`BrandSynthesizer::synthesize`].
    pub fn load_from_toml(path: &str) -> Result<(BrandTemplate, Vec<BrandError>), BrandError> {
        let content = std::fs::read_to_string(path).map_err(|e| BrandError::TomlReadError {
            path: Arc::from(path),
            reason: Arc::from(e.to_string().as_str()),
        })?;
        let config: BrandConfig =
            toml::from_str(&content).map_err(|e| BrandError::TomlParseError {
                path: Arc::from(path),
                reason: Arc::from(e.to_string().as_str()),
            })?;
        Self::synthesize(&config)
    }

    /// Synthesize a [`BrandTemplate`] from an already-parsed [`BrandConfig`].
    ///
    /// This is a **pure function** — no filesystem I/O. Same input always
    /// produces the same output (AC-007 determinism invariant).
    ///
    /// Steps:
    /// 1. Validate required fields (logo path — AC-004).
    /// 2. Build the 12-slot color palette via `inference::infer_missing_slots`.
    /// 3. Generate 31 layout definitions via `layouts::generate_all_layouts`.
    /// 4. Assemble the [`BrandTemplate`].
    ///
    /// Layout XML serialization is NOT performed here — the PPTX exporter
    /// (STORY-037) calls `layout_xml::serialize_layout_to_xml` directly when
    /// building each ZIP entry.
    ///
    /// # Errors
    ///
    /// Returns `Err(BrandError)` for fatal conditions (e.g., missing logo path).
    ///
    /// On success, returns `Ok((template, warnings))` where `warnings` contains
    /// zero or more [`BrandError::MissingColorSlot`] entries for each color slot
    /// that was absent in the config and was inferred (AC-002, AC-005, AC-007).
    /// These are cosmetic warnings — the build continues with inferred values.
    pub fn synthesize(
        config: &BrandConfig,
    ) -> Result<(BrandTemplate, Vec<BrandError>), BrandError> {
        // Step 1: Validate required fields (AC-004 — logo required)
        // Also reject empty path strings (F14: empty path passes is_none() check
        // but is equally unusable — reject immediately).
        if config.logo.as_ref().is_none_or(|l| l.path.is_empty()) {
            return Err(BrandError::LogoRequired {
                span: slideforge_types::SourceSpan::default(),
            });
        }

        // Step 2: Build 12-slot color palette (inference)
        let declared = config.colors.as_slot_array();
        let mut warnings: Vec<BrandError> = Vec::new();
        let hex_slots = inference::infer_missing_slots(declared, &mut warnings);

        // Build ColorSlot array from inferred hex strings
        let colors = build_color_slots(&hex_slots);

        // Step 3: Build fonts
        let fonts = BrandFonts {
            heading: Arc::from(config.fonts.heading.as_str()),
            body: Arc::from(config.fonts.body.as_str()),
        };

        // Step 4: Footer text
        let footer_text = if config.footer.text.is_empty() {
            None
        } else {
            Some(Arc::from(config.footer.text.as_str()))
        };

        // Step 5: Generate 31 layout definitions
        let layouts = generate_all_layouts(config);

        // Step 6: Layout XML serialization — called by STORY-037 PPTX exporter directly.
        // The exporter calls serialize_layout_to_xml per layout when building the ZIP
        // package. There is no need to exercise every layout here; the layout_xml
        // unit tests cover serialization exhaustively.

        // Step 7: Generate Content_Types entries (AC-013)
        let content_types_layout_entries =
            Arc::from(generate_content_types_layout_entries(layouts.len()).as_str());

        // Step 8: Master/handout stubs (AC-012)
        let notes_master_stub = NOTES_MASTER_STUB.to_vec();
        let handout_master_stub = HANDOUT_MASTER_STUB.to_vec();

        // Step 9: Master IDs (AC-011)
        let master_ids = MasterIds::default();

        // Step 10: Carry logo path as a deferred-load asset (F5 fix).
        // The bytes are NOT read here — synthesize() is a pure function with no I/O.
        // The PPTX exporter (STORY-037) reads the bytes when building the ZIP package.
        let logo = config.logo.as_ref().map(|l| LogoAsset::Deferred {
            path: Arc::from(l.path.as_str()),
        });

        let template = BrandTemplate {
            colors,
            fonts,
            logo,
            footer_text,
            layout_names: vec![],
            layouts,
            notes_master_stub,
            handout_master_stub,
            master_ids,
            content_types_layout_entries,
        };
        Ok((template, warnings))
    }
}

/// Build the `[ColorSlot; 12]` array from inferred hex strings.
///
/// Each slot is named per ECMA-376 order from `COLOR_SLOT_NAMES`.
fn build_color_slots(hex_slots: &[Arc<str>; 12]) -> [ColorSlot; 12] {
    // Use std::array::from_fn — stable in Rust 1.63+
    std::array::from_fn(|i| ColorSlot {
        name: Arc::from(COLOR_SLOT_NAMES[i]),
        value: ColorValue::Hex(Arc::clone(&hex_slots[i])),
    })
}

// ─── BrandProvider trait impl ─────────────────────────────────────────────────

impl BrandProvider for BrandSynthesizer {
    fn id(&self) -> &'static str {
        "slideforge-brand-synthesizer"
    }

    /// Load a [`Brand`] from `source`.
    ///
    /// Routes `BrandSource::TomlFile` to [`BrandSynthesizer::load_from_toml`]
    /// and converts the resulting [`BrandTemplate`] to `slideforge_types::Brand`.
    /// Routes `BrandSource::PptxFile` and `BrandSource::DocxFile` to
    /// [`crate::loader::BrandLoader`].
    ///
    /// # Errors
    ///
    /// Returns `slideforge_plugin_api::BrandError` on failure. All
    /// `slideforge_brand::BrandError` variants are mapped to the appropriate
    /// plugin-API error variant.
    fn load(&self, source: &BrandSource) -> Result<Brand, slideforge_plugin_api::BrandError> {
        match source {
            BrandSource::TomlFile(path) => {
                let (template, _warnings) = Self::load_from_toml(path.as_ref()).map_err(|e| {
                    // Map slideforge-brand BrandError to plugin-api BrandError.
                    match &e {
                        crate::error::BrandError::TomlReadError { path, reason } => {
                            slideforge_plugin_api::BrandError::IoError {
                                uri: path.as_ref().to_owned(),
                                message: reason.as_ref().to_owned(),
                            }
                        },
                        crate::error::BrandError::TomlParseError { path, reason } => {
                            slideforge_plugin_api::BrandError::ParseError {
                                uri: path.as_ref().to_owned(),
                                message: reason.as_ref().to_owned(),
                            }
                        },
                        crate::error::BrandError::LogoRequired { .. } => {
                            slideforge_plugin_api::BrandError::ValidationError {
                                message: e.to_string(),
                            }
                        },
                        _ => slideforge_plugin_api::BrandError::ValidationError {
                            message: e.to_string(),
                        },
                    }
                })?;

                // Convert BrandTemplate → slideforge_types::Brand.
                // BrandPalette maps to the first 4 ECMA-376 color slots:
                //   slot 0 (dk1)  → primary
                //   slot 1 (lt1)  → secondary
                //   slot 2 (dk2)  → accent
                //   slot 3 (lt2)  → neutral
                //
                // These are the canonical "theme-defining" colors per ECMA-376.
                // Full 12-slot data is retained in BrandTemplate for the PPTX
                // exporter (STORY-037); Brand carries the semantic palette only.
                let primary = template.colors[0].hex().unwrap_or("#000000").to_owned();
                let secondary = template.colors[1].hex().unwrap_or("#FFFFFF").to_owned();
                let accent = template.colors[2].hex().unwrap_or("#808080").to_owned();
                let neutral = template.colors[3].hex().unwrap_or("#F5F5F5").to_owned();

                let brand = Brand {
                    name: Arc::from("synthesized"),
                    palette: slideforge_types::BrandPalette {
                        primary: Arc::from(primary.as_str()),
                        secondary: Arc::from(secondary.as_str()),
                        accent: Arc::from(accent.as_str()),
                        neutral: Arc::from(neutral.as_str()),
                    },
                    fonts: slideforge_types::BrandFonts {
                        heading: Arc::clone(&template.fonts.heading),
                        body: Arc::clone(&template.fonts.body),
                        mono: Arc::from("Courier New"),
                    },
                    layouts: vec![],
                    span: slideforge_types::SourceSpan::default(),
                };
                Ok(brand)
            },
            // PptxFile and DocxFile extraction are implemented in STORY-024.
            // Return a clear, actionable error with the unsupported source type.
            BrandSource::PptxFile(path) => {
                Err(slideforge_plugin_api::BrandError::ValidationError {
                    message: format!(
                        "PPTX template extraction is not yet supported for '{path}'. \
                         Use a brand.toml file (BrandSource::TomlFile) or wait for STORY-024."
                    ),
                })
            },
            BrandSource::DocxFile(path) => {
                Err(slideforge_plugin_api::BrandError::ValidationError {
                    message: format!(
                        "DOCX template extraction is not yet supported for '{path}'. \
                         Use a brand.toml file (BrandSource::TomlFile) or wait for STORY-024."
                    ),
                })
            },
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toml_schema::{ColorConfig, FontConfig, FooterConfig, LogoConfig};

    /// Construct a minimal config for tests with just the required fields populated.
    fn minimal_config() -> BrandConfig {
        BrandConfig {
            colors: ColorConfig {
                acc1: Some("#3B82F6".to_owned()),
                dk1: Some("#1F2937".to_owned()),
                lt1: Some("#FFFFFF".to_owned()),
                ..Default::default()
            },
            fonts: FontConfig {
                heading: "Calibri".to_owned(),
                body: "Calibri".to_owned(),
            },
            logo: Some(LogoConfig {
                path: "test-logo.png".to_owned(),
            }),
            footer: FooterConfig {
                text: String::new(),
                show_slide_number: true,
                show_date: false,
            },
        }
    }

    /// Full config with all 12 colors declared.
    fn full_12_color_config() -> BrandConfig {
        BrandConfig {
            colors: ColorConfig {
                dk1: Some("#1F2937".to_owned()),
                lt1: Some("#FFFFFF".to_owned()),
                dk2: Some("#374151".to_owned()),
                lt2: Some("#F9FAFB".to_owned()),
                acc1: Some("#3B82F6".to_owned()),
                acc2: Some("#10B981".to_owned()),
                acc3: Some("#F59E0B".to_owned()),
                acc4: Some("#EF4444".to_owned()),
                acc5: Some("#8B5CF6".to_owned()),
                acc6: Some("#EC4899".to_owned()),
                hlink: Some("#2563EB".to_owned()),
                fol_hlink: Some("#1D4ED8".to_owned()),
            },
            fonts: FontConfig {
                heading: "Calibri".to_owned(),
                body: "Calibri".to_owned(),
            },
            logo: Some(LogoConfig {
                path: "test-logo.png".to_owned(),
            }),
            footer: FooterConfig {
                text: String::new(),
                show_slide_number: true,
                show_date: false,
            },
        }
    }

    /// BC-2.01.002 / AC-004 — `synthesize` returns error when logo absent.
    #[test]
    fn test_bc_2_01_002_ac004_synthesize_rejects_missing_logo() {
        let config = BrandConfig {
            colors: ColorConfig::default(),
            fonts: FontConfig {
                heading: "Calibri".to_owned(),
                body: "Calibri".to_owned(),
            },
            logo: None, // no logo → LogoRequired error
            footer: FooterConfig {
                text: String::new(),
                show_slide_number: true,
                show_date: false,
            },
        };
        let result = BrandSynthesizer::synthesize(&config);
        assert!(
            result.is_err(),
            "synthesize must fail when logo path absent"
        );
        match result.unwrap_err() {
            BrandError::LogoRequired { .. } => {},
            other => panic!("expected BrandError::LogoRequired, got: {other:?}"),
        }
    }

    /// BC-2.01.002 invariant 4 / AC-007 — synthesis is deterministic (same input → same output).
    #[test]
    fn test_bc_2_01_002_invariant_synthesis_is_deterministic() {
        let config = full_12_color_config();
        let (t1, _) = BrandSynthesizer::synthesize(&config)
            .expect("synthesize must succeed with full config");
        let (t2, _) =
            BrandSynthesizer::synthesize(&config).expect("second synthesize must succeed");
        // Color slots must be identical
        assert_eq!(
            t1.colors.len(),
            t2.colors.len(),
            "color count must be equal"
        );
        for i in 0..12 {
            assert_eq!(
                t1.colors[i].hex(),
                t2.colors[i].hex(),
                "slot {i} hex must be deterministic"
            );
        }
        // Layout count and names must be identical
        assert_eq!(
            t1.layouts.len(),
            t2.layouts.len(),
            "layout count must be deterministic"
        );
        for i in 0..t1.layouts.len() {
            assert_eq!(
                t1.layouts[i].name.as_ref(),
                t2.layouts[i].name.as_ref(),
                "layout {i} name must be deterministic"
            );
        }
    }

    /// BC-2.01.002 — synthesized `BrandTemplate` has exactly 12 color slots.
    #[test]
    fn test_bc_2_01_002_synthesized_template_has_12_color_slots() {
        let config = full_12_color_config();
        let (result, _) = BrandSynthesizer::synthesize(&config).expect("synthesize must succeed");
        assert_eq!(
            result.colors.len(),
            12,
            "synthesized BrandTemplate must have exactly 12 color slots"
        );
    }

    /// BC-2.01.005 — synthesized `BrandTemplate` has exactly 31 layouts.
    #[test]
    fn test_bc_2_01_005_synthesized_template_has_31_layouts() {
        let config = full_12_color_config();
        let (result, _) = BrandSynthesizer::synthesize(&config).expect("synthesize must succeed");
        assert_eq!(
            result.layouts.len(),
            31,
            "synthesized BrandTemplate must have exactly 31 layouts"
        );
    }

    /// AC-011 — synthesized `BrandTemplate` has `master_id = 2^31`.
    #[test]
    fn test_bc_2_01_005_ac011_master_id_is_2_pow_31() {
        let config = full_12_color_config();
        let (result, _) = BrandSynthesizer::synthesize(&config).expect("synthesize must succeed");
        assert_eq!(
            result.master_ids.master_id,
            2u32.pow(31),
            "master_id must be 2^31 = 2147483648"
        );
    }

    /// AC-011 — synthesized `BrandTemplate` has `layout_id_start = 2^31 + 1`.
    #[test]
    fn test_bc_2_01_005_ac011_layout_id_start_is_2_pow_31_plus_1() {
        let config = full_12_color_config();
        let (result, _) = BrandSynthesizer::synthesize(&config).expect("synthesize must succeed");
        assert_eq!(
            result.master_ids.layout_id_start,
            2u32.pow(31) + 1,
            "layout_id_start must be 2^31 + 1"
        );
    }

    /// AC-012 — synthesized `BrandTemplate` has non-empty `notes_master_stub`.
    #[test]
    fn test_bc_2_01_002_ac012_notes_master_stub_present() {
        let config = full_12_color_config();
        let (result, _) = BrandSynthesizer::synthesize(&config).expect("synthesize must succeed");
        assert!(
            !result.notes_master_stub.is_empty(),
            "notes_master_stub must be non-empty in synthesized BrandTemplate"
        );
        assert!(
            result.notes_master_stub.starts_with(b"<?xml"),
            "notes_master_stub must be valid XML"
        );
    }

    /// AC-012 — synthesized `BrandTemplate` has non-empty `handout_master_stub`.
    #[test]
    fn test_bc_2_01_002_ac012_handout_master_stub_present() {
        let config = full_12_color_config();
        let (result, _) = BrandSynthesizer::synthesize(&config).expect("synthesize must succeed");
        assert!(
            !result.handout_master_stub.is_empty(),
            "handout_master_stub must be non-empty in synthesized BrandTemplate"
        );
        assert!(
            result.handout_master_stub.starts_with(b"<?xml"),
            "handout_master_stub must be valid XML"
        );
    }

    /// BC-2.01.005 / AC-013 — synthesized `BrandTemplate` has non-empty
    /// `content_types_layout_entries` with 31 layout references.
    #[test]
    fn test_bc_2_01_005_synthesize_produces_complete_brand_template() {
        let config = full_12_color_config();
        let (result, _) = BrandSynthesizer::synthesize(&config)
            .expect("synthesize with logo + all 12 colors must succeed");
        // 12 color slots
        assert_eq!(result.colors.len(), 12, "must have 12 color slots");
        // 31 layouts
        assert_eq!(result.layouts.len(), 31, "must have 31 layouts");
        // master IDs
        assert_eq!(result.master_ids.master_id, 2u32.pow(31));
        assert_eq!(result.master_ids.layout_id_start, 2u32.pow(31) + 1);
        assert_eq!(result.master_ids.slide_id_start, 256);
        // Master stubs present
        assert!(!result.notes_master_stub.is_empty());
        assert!(!result.handout_master_stub.is_empty());
        // Content types entries present
        assert!(
            !result.content_types_layout_entries.is_empty(),
            "content_types_layout_entries must be non-empty"
        );
        assert!(
            result.content_types_layout_entries.contains("slideLayout"),
            "content_types_layout_entries must reference slideLayout XML files"
        );
    }

    /// BC-2.01.005 — synthesize with minimal (inferred) colors → all 12 color slots
    /// populated in `BrandTemplate` (some inferred).
    #[test]
    fn test_bc_2_01_005_synthesize_with_inferred_colors() {
        let config = minimal_config(); // only dk1, lt1, acc1 declared; 9 slots inferred
        let (result, _) = BrandSynthesizer::synthesize(&config)
            .expect("synthesize with minimal config must succeed");
        // All 12 color slots must be present and populated
        assert_eq!(
            result.colors.len(),
            12,
            "must always have exactly 12 color slots even with inferred values"
        );
        // Each slot must be a resolved hex
        for (i, slot) in result.colors.iter().enumerate() {
            assert!(
                slot.is_resolved(),
                "color slot {i} ('{}') must be a resolved hex color",
                slot.name
            );
            let hex = slot.hex().expect("slot must have a hex value");
            assert!(
                hex.starts_with('#') && hex.len() == 7,
                "color slot {i}: expected #RRGGBB, got '{hex}'"
            );
        }
    }

    /// `BrandProvider::id()` returns the expected plugin ID.
    #[test]
    fn test_brand_provider_id_is_correct() {
        let synth = BrandSynthesizer;
        assert_eq!(synth.id(), "slideforge-brand-synthesizer");
    }

    /// `BrandProvider` trait is implemented (compile-time duck-type check).
    #[test]
    fn test_brand_synthesizer_implements_brand_provider() {
        fn assert_brand_provider<T: BrandProvider>() {}
        assert_brand_provider::<BrandSynthesizer>();
    }

    /// BC-2.01.005 / AC-005 — `synthesize` with empty `[colors]` section yields 12
    /// `MissingColorSlot` warnings (one per slot) and returns 12 color slots.
    ///
    /// This is the synthesizer-level coverage for AC-005 (empty colors).
    #[test]
    fn test_bc_2_01_005_synthesize_empty_colors_yields_12_warnings() {
        let config = BrandConfig {
            colors: ColorConfig::default(), // all 12 slots absent
            fonts: FontConfig {
                heading: "Calibri".to_owned(),
                body: "Calibri".to_owned(),
            },
            logo: Some(LogoConfig {
                path: "test-logo.png".to_owned(),
            }),
            footer: FooterConfig::default(),
        };
        let (template, warnings) = BrandSynthesizer::synthesize(&config)
            .expect("synthesize must succeed even with no colors");
        assert_eq!(
            warnings.len(),
            12,
            "AC-005: exactly 12 MissingColorSlot warnings expected when all slots absent"
        );
        assert!(
            warnings
                .iter()
                .all(|w| matches!(w, BrandError::MissingColorSlot { .. })),
            "AC-005: all warnings must be MissingColorSlot"
        );
        assert_eq!(
            template.colors.len(),
            12,
            "AC-005: synthesized template must always have 12 color slots"
        );
        // All inferred slots must be resolved hex colors
        for (i, slot) in template.colors.iter().enumerate() {
            assert!(
                slot.is_resolved(),
                "AC-005: inferred color slot {i} ('{}') must be a resolved hex",
                slot.name
            );
        }
    }

    /// F-PASS2-H1 — `BrandProvider::load(BrandSource::TomlFile)` returns `Ok(Brand)`.
    ///
    /// Writes a minimal brand.toml to a tempfile and calls `BrandSynthesizer::load`.
    /// Verifies the returned `Brand` has the expected color palette slots populated.
    #[test]
    fn test_brand_provider_load_toml_file_returns_ok() {
        use slideforge_plugin_api::BrandProvider as _;
        use std::io::Write as _;
        // Write a minimal brand.toml to a temp file.
        let mut tmp = tempfile::NamedTempFile::new().expect("tempfile must be created");
        writeln!(
            tmp,
            r##"
[colors]
dk1 = "#1F2937"
lt1 = "#FFFFFF"
acc1 = "#3B82F6"

[logo]
path = "logo.png"

[fonts]
heading = "Calibri"
body = "Calibri"
"##
        )
        .expect("write to tempfile must succeed");
        let path = tmp
            .path()
            .to_str()
            .expect("tempfile path must be valid UTF-8");

        let synth = BrandSynthesizer;
        let source = slideforge_plugin_api::BrandSource::TomlFile(Arc::from(path));
        let brand = synth
            .load(&source)
            .expect("BrandProvider::load(TomlFile) must return Ok(Brand)");
        // Verify the palette was populated from the color slots.
        // slot 0 (dk1) → primary = "#1F2937"
        assert_eq!(
            brand.palette.primary.as_ref(),
            "#1F2937",
            "primary color must match dk1 from brand.toml"
        );
        // slot 1 (lt1) → secondary = "#FFFFFF"
        assert_eq!(
            brand.palette.secondary.as_ref(),
            "#FFFFFF",
            "secondary color must match lt1 from brand.toml"
        );
        // Fonts must be preserved
        assert_eq!(
            brand.fonts.heading.as_ref(),
            "Calibri",
            "heading font must match [fonts].heading from brand.toml"
        );
    }

    /// F-PASS2-H1 — `BrandProvider::load(BrandSource::PptxFile)` returns `Err` with
    /// a clear, actionable message referencing STORY-024.
    #[test]
    fn test_brand_provider_load_pptx_file_returns_err_pending_story_024() {
        use slideforge_plugin_api::BrandProvider as _;
        let synth = BrandSynthesizer;
        let source = slideforge_plugin_api::BrandSource::PptxFile(Arc::from("template.pptx"));
        let err = synth
            .load(&source)
            .expect_err("PptxFile must return Err (not yet supported)");
        let msg = err.to_string();
        assert!(
            msg.contains("PPTX") || msg.contains("pptx"),
            "error message must mention PPTX, got: {msg}"
        );
        assert!(
            msg.contains("STORY-024"),
            "error message must reference STORY-024, got: {msg}"
        );
    }

    /// F-PASS2-M4 — Snapshot-based determinism guard for color inference.
    ///
    /// Snapshots the inferred color palette for a fixed input on the dev platform.
    /// CI running on Linux/macOS/Windows will catch cross-platform f32 HSL divergence
    /// if the platform produces different output for the same input.
    ///
    /// The snapshot is blessed on the dev platform with `INSTA_UPDATE=unseen`.
    #[test]
    fn test_inference_snapshot_deterministic() {
        use crate::toml_schema::{ColorConfig, FontConfig, FooterConfig, LogoConfig};
        let config = BrandConfig {
            colors: ColorConfig {
                dk1: Some("#1F2937".to_owned()),
                lt1: Some("#FFFFFF".to_owned()),
                acc1: Some("#3B82F6".to_owned()),
                // All other slots absent — will be inferred
                ..ColorConfig::default()
            },
            fonts: FontConfig {
                heading: "Calibri".to_owned(),
                body: "Calibri".to_owned(),
            },
            logo: Some(LogoConfig {
                path: "logo.png".to_owned(),
            }),
            footer: FooterConfig::default(),
        };
        let (template, warnings) =
            BrandSynthesizer::synthesize(&config).expect("synthesize must succeed");

        // Collect hex values for all 12 slots for snapshot comparison.
        let hex_slots: Vec<(&str, &str)> = template
            .colors
            .iter()
            .map(|s| (s.name.as_ref(), s.hex().unwrap_or("UNRESOLVED")))
            .collect();
        let warning_count = warnings.len();

        // Render as a stable text snapshot for cross-platform determinism verification.
        let snapshot_text = format!(
            "warnings: {warning_count}\ncolors:\n{}",
            hex_slots
                .iter()
                .map(|(name, hex)| format!("  {name}: {hex}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        insta::assert_snapshot!("inference_snapshot_deterministic", snapshot_text);
    }

    /// F13 — `BrandError::LogoRequired` now carries a `span` field.
    #[test]
    fn test_f13_logo_required_has_span_field() {
        let err = BrandError::LogoRequired {
            span: slideforge_types::SourceSpan::default(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("E-BRD-001"),
            "LogoRequired must contain E-BRD-001"
        );
        assert!(
            msg.contains("logo path"),
            "LogoRequired message must mention logo path"
        );
    }

    /// F14 — empty logo path string is rejected (produces `LogoRequired`, not `LogoAsset::Deferred`).
    #[test]
    fn test_f14_empty_logo_path_is_rejected() {
        let config = BrandConfig {
            colors: ColorConfig {
                acc1: Some("#3B82F6".to_owned()),
                ..Default::default()
            },
            fonts: FontConfig::default(),
            logo: Some(LogoConfig {
                path: String::new(), // empty path → must be rejected
            }),
            footer: FooterConfig::default(),
        };
        let result = BrandSynthesizer::synthesize(&config);
        assert!(
            result.is_err(),
            "F14: empty logo path must be rejected (LogoRequired error)"
        );
        match result.unwrap_err() {
            BrandError::LogoRequired { .. } => {},
            other => {
                panic!("F14: expected BrandError::LogoRequired for empty path, got: {other:?}")
            },
        }
    }

    /// F4 — `synthesize` propagates `MissingColorSlot` warnings in the Ok tuple.
    ///
    /// When color slots are absent, the warnings must be programmatically visible
    /// to the caller (not silently dropped). Spec line 246: return includes warnings.
    #[test]
    fn test_f4_synthesize_propagates_warnings_in_ok_tuple() {
        let config = minimal_config(); // dk1, lt1, acc1 declared; 9 slots absent
        let (_, warnings) = BrandSynthesizer::synthesize(&config)
            .expect("synthesize must succeed with minimal config");
        assert!(
            !warnings.is_empty(),
            "F4: warnings must be non-empty when color slots are inferred"
        );
        // All warnings must be MissingColorSlot variants
        for w in &warnings {
            assert!(
                matches!(w, BrandError::MissingColorSlot { .. }),
                "F4: all warnings must be BrandError::MissingColorSlot, got: {w:?}"
            );
        }
    }

    /// F4 — all-12-declared config produces zero warnings.
    #[test]
    fn test_f4_synthesize_no_warnings_when_all_12_declared() {
        let config = full_12_color_config();
        let (_, warnings) = BrandSynthesizer::synthesize(&config).expect("synthesize must succeed");
        assert_eq!(
            warnings.len(),
            0,
            "F4: zero warnings when all 12 color slots declared"
        );
    }

    /// F5 — synthesized `BrandTemplate.logo` is `Some(LogoAsset::Deferred)`, not `None`.
    ///
    /// The PPTX exporter (STORY-037) reads the bytes from the declared path.
    /// Synthesis itself must not return `None` when a logo path is declared.
    #[test]
    fn test_f5_synthesized_logo_is_some_deferred() {
        let config = minimal_config(); // has logo path "test-logo.png"
        let (result, _) = BrandSynthesizer::synthesize(&config).expect("synthesize must succeed");
        let logo = result
            .logo
            .expect("F5: logo must be Some when [logo].path is declared");
        assert!(
            matches!(logo, crate::template::LogoAsset::Deferred { .. }),
            "F5: synthesized logo must be LogoAsset::Deferred (not Loaded — no I/O in pure fn)"
        );
        assert_eq!(
            logo.deferred_path(),
            Some("test-logo.png"),
            "F5: deferred path must match [logo].path from brand.toml"
        );
    }

    // ─── VP-012: proptest — determinism round-trip ─────────────────────────────

    /// Exercises VP-012: palette determinism under randomised `BrandConfig` input.
    ///
    /// Strategy: arbitrary optional hex color strings → `BrandConfig` → synthesize
    /// twice with the same config → assert equal color hex values in both results.
    ///
    /// Generator produces valid `"#RRGGBB"` strings with uppercase hex digits to
    /// match the `InvalidHexColor` validation contract.
    #[cfg(test)]
    mod proptest_vp012 {
        use super::*;
        use proptest::prelude::*;

        /// Generate a random valid 6-digit uppercase hex color string.
        fn arb_hex_color() -> impl Strategy<Value = String> {
            proptest::collection::vec(
                prop_oneof![
                    Just('0'),
                    Just('1'),
                    Just('2'),
                    Just('3'),
                    Just('4'),
                    Just('5'),
                    Just('6'),
                    Just('7'),
                    Just('8'),
                    Just('9'),
                    Just('A'),
                    Just('B'),
                    Just('C'),
                    Just('D'),
                    Just('E'),
                    Just('F'),
                ],
                6,
            )
            .prop_map(|chars| format!("#{}", chars.iter().collect::<String>()))
        }

        /// Generate an optional hex color (Some or None with equal probability).
        fn arb_opt_hex() -> impl Strategy<Value = Option<String>> {
            prop_oneof![Just(None), arb_hex_color().prop_map(Some),]
        }

        proptest! {
            /// VP-012 — `synthesize(config) == synthesize(config)` for any valid config.
            ///
            /// Exercises BC-2.01.002 invariant 4 (AC-007) under randomised input.
            #[test]
            fn test_bc_2_01_002_invariant_synthesis_is_deterministic_proptest(
                dk1  in arb_opt_hex(),
                lt1  in arb_opt_hex(),
                dk2  in arb_opt_hex(),
                lt2  in arb_opt_hex(),
                acc1 in arb_hex_color(),  // acc1 required so hlink/fol_hlink derivation works
                acc2 in arb_opt_hex(),
                acc3 in arb_opt_hex(),
                acc4 in arb_opt_hex(),
                acc5 in arb_opt_hex(),
                acc6 in arb_opt_hex(),
                hlink     in arb_opt_hex(),
                fol_hlink in arb_opt_hex(),
            ) {
                let config = BrandConfig {
                    colors: ColorConfig {
                        dk1,
                        lt1,
                        dk2,
                        lt2,
                        acc1: Some(acc1),
                        acc2,
                        acc3,
                        acc4,
                        acc5,
                        acc6,
                        hlink,
                        fol_hlink,
                    },
                    fonts: FontConfig {
                        heading: "Calibri".to_owned(),
                        body: "Calibri".to_owned(),
                    },
                    logo: Some(LogoConfig {
                        path: "test.png".to_owned(),
                    }),
                    footer: FooterConfig {
                        text: String::new(),
                        show_slide_number: true,
                        show_date: false,
                    },
                };

                let r1 = BrandSynthesizer::synthesize(&config);
                let r2 = BrandSynthesizer::synthesize(&config);

                match (r1, r2) {
                    (Ok((t1, w1)), Ok((t2, w2))) => {
                        // Full structural equality via derived PartialEq covers all fields:
                        // colors (12 slots), fonts, logo, footer_text, layouts, master IDs, etc.
                        prop_assert_eq!(t1, t2, "synthesize must be fully deterministic");
                        // Warning count must also be deterministic for identical inputs.
                        prop_assert_eq!(w1.len(), w2.len(),
                            "synthesize produced inconsistent warning counts for same input");
                    }
                    (Err(_), Err(_)) => {
                        // Both fail the same way — deterministic failure is also acceptable.
                    }
                    (Ok(_), Err(e)) | (Err(e), Ok(_)) => {
                        prop_assert!(false,
                            "synthesize produced inconsistent Ok/Err for same input: {e:?}");
                    }
                }
            }
        }
    }
}
