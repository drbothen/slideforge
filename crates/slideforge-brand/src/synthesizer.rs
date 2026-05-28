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

use slideforge_plugin_api::BrandProvider;
use slideforge_plugin_api::BrandSource;
use slideforge_types::Brand;

use crate::error::BrandError;
use crate::template::BrandTemplate;
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
    /// - [`BrandError::FileNotFound`] if `path` does not exist.
    /// - [`BrandError::ParseError`] if the file content is not valid TOML.
    /// - Propagates all errors from [`BrandSynthesizer::synthesize`].
    pub fn load_from_toml(_path: &str) -> Result<BrandTemplate, BrandError> {
        todo!()
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
    /// 4. Serialize each layout to XML via `layout_xml::serialize_layout_to_xml`.
    /// 5. Assemble the [`BrandTemplate`].
    ///
    /// # Errors
    ///
    /// Returns `Err(BrandError)` for fatal conditions (e.g., missing logo path).
    /// Color slot inference warnings are logged via `tracing::warn!` and do NOT
    /// cause an error return — build continues with inferred values.
    pub fn synthesize(_config: &BrandConfig) -> Result<BrandTemplate, BrandError> {
        todo!()
    }
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
    fn load(
        &self,
        _source: &BrandSource,
    ) -> Result<Brand, slideforge_plugin_api::BrandError> {
        todo!()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toml_schema::{ColorConfig, FooterConfig, FontConfig, LogoConfig};

    /// Construct a minimal config directly without calling todo!() helpers.
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
        assert!(result.is_err(), "synthesize must fail when logo path absent");
        match result.unwrap_err() {
            BrandError::LogoRequired => {}
            other => panic!("expected BrandError::LogoRequired, got: {other:?}"),
        }
    }

    /// BC-2.01.002 invariant 4 / AC-007 — synthesis is deterministic (same input → same output).
    #[test]
    fn test_bc_2_01_002_invariant_synthesis_is_deterministic() {
        let config = full_12_color_config();
        let t1 = BrandSynthesizer::synthesize(&config)
            .expect("synthesize must succeed with full config");
        let t2 = BrandSynthesizer::synthesize(&config)
            .expect("second synthesize must succeed");
        // Color slots must be identical
        assert_eq!(t1.colors.len(), t2.colors.len(), "color count must be equal");
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
        let result = BrandSynthesizer::synthesize(&config)
            .expect("synthesize must succeed");
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
        let result = BrandSynthesizer::synthesize(&config)
            .expect("synthesize must succeed");
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
        let result = BrandSynthesizer::synthesize(&config)
            .expect("synthesize must succeed");
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
        let result = BrandSynthesizer::synthesize(&config)
            .expect("synthesize must succeed");
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
        let result = BrandSynthesizer::synthesize(&config)
            .expect("synthesize must succeed");
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
        let result = BrandSynthesizer::synthesize(&config)
            .expect("synthesize must succeed");
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
        let result = BrandSynthesizer::synthesize(&config)
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
    /// populated in BrandTemplate (some inferred).
    #[test]
    fn test_bc_2_01_005_synthesize_with_inferred_colors() {
        let config = minimal_config(); // only dk1, lt1, acc1 declared; 9 slots inferred
        let result = BrandSynthesizer::synthesize(&config)
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

    // ─── VP-012: proptest — determinism round-trip ─────────────────────────────

    /// Exercises VP-012: palette determinism under randomised BrandConfig input.
    ///
    /// Strategy: arbitrary optional hex color strings → BrandConfig → synthesize
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
            prop_oneof![
                Just(None),
                arb_hex_color().prop_map(Some),
            ]
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
                    (Ok(t1), Ok(t2)) => {
                        prop_assert_eq!(t1.layouts.len(), t2.layouts.len(),
                            "layout count must be deterministic");
                        for i in 0..12 {
                            let h1 = t1.colors[i].hex();
                            let h2 = t2.colors[i].hex();
                            prop_assert_eq!(h1, h2);
                        }
                        for i in 0..t1.layouts.len() {
                            let n1 = t1.layouts[i].name.as_ref();
                            let n2 = t2.layouts[i].name.as_ref();
                            prop_assert_eq!(n1, n2);
                        }
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
