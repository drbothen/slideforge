//! Brand synthesizer — `brand.toml` → [`BrandTemplate`] (BC-2.01.002, BC-2.01.004, BC-2.01.005).
//!
//! [`BrandSynthesizer`] is the primary entry point for synthesizing a complete
//! [`BrandTemplate`] from a `brand.toml` file. It implements two separate
//! concerns per Architecture Compliance Rule 2 (STORY-023):
//!
//! 1. **Effectful I/O** — [`BrandSynthesizer::load_from_toml`]: reads the
//!    `brand.toml` file from the filesystem.
//! 2. **Pure synthesis** — [`BrandSynthesizer::synthesize`]: pure function;
//!    takes a [`BrandConfig`] reference, performs color inference and layout
//!    generation, and returns a [`BrandTemplate`]. XML serialization is
//!    deferred to the PPTX exporter (STORY-037).
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
//! | `brand.toml` not found or unreadable | [`BrandError::TomlReadError`] | fatal (exit 4) |
//! | Invalid TOML syntax | [`BrandError::TomlParseError`] | fatal (exit 4) |
//! | Missing `[logo]` section | `E-BRD-001` via `BrandError::LogoRequired` | fatal (exit 4) |
//! | Absent color slot | [`BrandError::MissingColorSlot`] | warning (exit 0) |

use std::sync::Arc;

use slideforge_plugin_api::BrandProvider;
use slideforge_plugin_api::BrandSource;
use slideforge_types::Brand;

use tracing::instrument;

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
/// # fn run() -> Result<(), Box<dyn std::error::Error>> {
/// use slideforge_brand::synthesizer::BrandSynthesizer;
///
/// // Load from file (effectful path):
/// let (template, warnings) = BrandSynthesizer::load_from_toml("path/to/brand.toml")?;
/// for warning in &warnings {
///     eprintln!("warning: {warning}");
/// }
///
/// // Synthesize from already-parsed config (pure path, useful for testing):
/// use slideforge_brand::toml_schema::BrandConfig;
/// // `default()` and `default_minimal()` are equivalent — both include a dummy logo path.
/// let config = BrandConfig::default_minimal();
/// let result = BrandSynthesizer::synthesize(&config);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct BrandSynthesizer;

impl BrandSynthesizer {
    /// Load a [`BrandTemplate`] from a `brand.toml` file at `path`.
    ///
    /// This is the effectful entry point. It reads the file, parses TOML,
    /// checks that the declared logo path exists on the filesystem (EC-005),
    /// and calls [`BrandSynthesizer::synthesize`].
    ///
    /// # Errors
    ///
    /// - [`BrandError::TomlReadError`] if `path` does not exist or cannot be read.
    /// - [`BrandError::TomlParseError`] if the file content is not valid TOML.
    /// - [`BrandError::FileNotFound`] (E-BRD-001) if the declared `[logo].path` does
    ///   not exist on the filesystem. The logo path is resolved relative to the
    ///   directory containing `brand.toml`.
    /// - Propagates all errors from [`BrandSynthesizer::synthesize`].
    ///
    /// On success, returns `(template, warnings)` — see [`BrandSynthesizer::synthesize`].
    #[instrument(skip_all, fields(toml_path = %path))]
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

        // EC-005: check logo path existence here (effectful — I/O allowed in loader).
        // `synthesize` is pure and cannot perform this check.
        // The logo path is relative to the directory containing brand.toml.
        if let Some(logo) = config.logo.as_ref()
            && !logo.path.is_empty()
        {
            let brand_toml_dir = std::path::Path::new(path)
                .parent()
                .unwrap_or(std::path::Path::new("."));
            let logo_path = brand_toml_dir.join(&logo.path);
            if !logo_path.exists() {
                return Err(BrandError::FileNotFound {
                    path: Arc::from(logo_path.to_string_lossy().as_ref()),
                    span: slideforge_types::SourceSpan::default(),
                });
            }

            // Security: path-traversal guard (F-PASS11-MED-2).
            // Canonicalize both paths and verify the logo resides inside the brand dir.
            // We only canonicalize after confirming existence (above) so that
            // canonicalize does not fail on a non-existent file.
            let canonical_logo =
                std::fs::canonicalize(&logo_path).map_err(|e| BrandError::TomlReadError {
                    path: Arc::from(logo_path.to_string_lossy().as_ref()),
                    reason: Arc::from(e.to_string().as_str()),
                })?;
            let canonical_brand_dir =
                std::fs::canonicalize(brand_toml_dir).map_err(|e| BrandError::TomlReadError {
                    path: Arc::from(brand_toml_dir.to_string_lossy().as_ref()),
                    reason: Arc::from(e.to_string().as_str()),
                })?;
            // F-PASS13-HIGH-2: On Windows, std::fs::canonicalize may return paths with
            // the `\\?\` extended-length prefix on only one of the two paths, causing
            // starts_with to return false incorrectly. Normalize both paths before
            // comparing by stripping any `\\?\` prefix.
            let canonical_logo_norm = strip_unc_prefix(&canonical_logo);
            let canonical_brand_dir_norm = strip_unc_prefix(&canonical_brand_dir);
            if !canonical_logo_norm.starts_with(&canonical_brand_dir_norm) {
                return Err(BrandError::LogoOutsideBrandDir {
                    logo_path: canonical_logo_norm.to_string_lossy().into_owned(),
                    brand_dir: canonical_brand_dir_norm.to_string_lossy().into_owned(),
                });
            }
        }

        let (mut template, warnings) = Self::synthesize(&config)?;

        // DEF-1 fix: resolve the deferred logo path relative to the brand.toml
        // directory. `synthesize` stores the as-written relative path; here (in the
        // effectful loader) we know the brand.toml directory and can produce the
        // correct filesystem path for the PPTX exporter (STORY-037).
        if let Some(crate::template::LogoAsset::Deferred {
            path: ref mut logo_path,
            ..
        }) = template.logo
        {
            let brand_toml_dir = std::path::Path::new(path)
                .parent()
                .unwrap_or(std::path::Path::new("."));
            let as_written = logo_path.clone();
            *logo_path = brand_toml_dir.join(as_written);
        }

        tracing::debug!(warnings_count = warnings.len(), "load_from_toml succeeded");
        Ok((template, warnings))
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
    #[instrument(skip(config), fields(heading_font = %config.fonts.heading))]
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
        //
        // `path` is initialised to the as-written value here. `load_from_toml`
        // replaces it with the resolved (brand.toml-relative) path after synthesis
        // so that the PPTX exporter can open the file regardless of CWD (DEF-1 fix).
        let logo = config.logo.as_ref().map(|l| LogoAsset::Deferred {
            path: std::path::PathBuf::from(l.path.as_str()),
            original: Arc::from(l.path.as_str()),
        });

        // Populate layout_names from the layout definitions (F-PASS11-LOW-1).
        // For synthesized brands, layout_names mirrors the human-readable names
        // from the generated layouts (31 entries, matching `layouts.len()`).
        // For loaded brands (from .pptx/.docx), layout_names holds ZIP-internal
        // paths populated by BrandLoader — see loader.rs.
        let layout_names: Vec<Arc<str>> = layouts.iter().map(|l| Arc::clone(&l.name)).collect();

        let template = BrandTemplate {
            colors,
            fonts,
            logo,
            footer_text,
            layout_names,
            layouts,
            notes_master_stub,
            handout_master_stub,
            master_ids,
            content_types_layout_entries,
        };
        tracing::debug!(
            layouts_count = template.layouts.len(),
            "synthesize succeeded"
        );
        Ok((template, warnings))
    }
}

/// Strip the Windows extended-length path prefix (`\\?\`) from a path.
///
/// On Windows, `std::fs::canonicalize` returns paths with the `\\?\` prefix.
/// If only one of two paths has the prefix (e.g., due to an intermediate
/// symlink or drive letter difference), `Path::starts_with` returns false
/// incorrectly. Stripping the prefix from both before comparing avoids this.
///
/// On non-Windows platforms this is a no-op that returns the path unchanged.
///
/// Shared with `overlay.rs` (F-025-001, TD-VSDD-060 — single source of truth).
///
/// F-PASS13-HIGH-2 fix.
#[cfg(windows)]
pub(crate) fn strip_unc_prefix(path: &std::path::Path) -> std::path::PathBuf {
    let s = path.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        std::path::PathBuf::from(stripped)
    } else {
        path.to_path_buf()
    }
}

/// Strip the Windows extended-length path prefix (no-op on non-Windows).
///
/// See the `#[cfg(windows)]` variant for details.
#[cfg(not(windows))]
pub(crate) fn strip_unc_prefix(path: &std::path::Path) -> std::path::PathBuf {
    path.to_path_buf()
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
    #[instrument(skip(self, source), fields(brand_source = ?source))]
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
                        // F-PASS13-OBS-2: All remaining variants (including
                        // LogoOutsideBrandDir — a security-relevant path-traversal
                        // guard) map to ValidationError because
                        // slideforge_plugin_api::BrandError has no SecurityError
                        // variant. The E-BRD-007 error message makes the security
                        // nature explicit to callers. Adding a SecurityError variant
                        // to plugin-api is tracked for a future story that adds
                        // structured security event observability.
                        _ => slideforge_plugin_api::BrandError::ValidationError {
                            message: e.to_string(),
                        },
                    }
                })?;

                // Convert BrandTemplate → slideforge_types::Brand.
                // BrandPalette uses ECMA-376 slot names, NOT positional indices,
                // to match the mapping in `brand_from_template` (loader.rs):
                //   dk2  → primary   (main brand color, not dk1 which is text black)
                //   acc1 → secondary (primary accent; first visible brand color)
                //   acc2 → accent    (secondary accent)
                //   lt2  → neutral   (light background / muted tone)
                //
                // Full 12-slot data is retained in BrandTemplate for the PPTX
                // exporter (STORY-037); Brand carries the semantic palette only.
                let slot_hex = |name: &str, fallback: &str| -> String {
                    template
                        .color_by_name(name)
                        .and_then(|s| s.hex())
                        .unwrap_or(fallback)
                        .to_owned()
                };
                let primary = slot_hex("dk2", "#000000");
                let secondary = slot_hex("acc1", "#808080");
                let accent = slot_hex("acc2", "#808080");
                let neutral = slot_hex("lt2", "#F5F5F5");

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
        match result.expect_err("synthesize must fail when logo path absent") {
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
    ///
    /// The test also creates a dummy logo file alongside the brand.toml so that the
    /// EC-005 logo-path existence check in `load_from_toml` passes.
    #[test]
    fn test_brand_provider_load_toml_file_returns_ok() {
        use slideforge_plugin_api::BrandProvider as _;
        use std::io::Write as _;
        // Use a temp directory so brand.toml and logo.png can coexist.
        let tmp_dir = tempfile::tempdir().expect("tempdir must be created");
        let brand_toml_path = tmp_dir.path().join("brand.toml");
        let logo_path = tmp_dir.path().join("logo.png");

        // Write a minimal PNG header as the logo fixture (EC-005 requires the file exists).
        std::fs::write(&logo_path, b"\x89PNG\r\n\x1a\n").expect("logo fixture write must succeed");

        let mut tmp =
            std::fs::File::create(&brand_toml_path).expect("brand.toml create must succeed");
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
        .expect("write to brand.toml must succeed");
        let path = brand_toml_path
            .to_str()
            .expect("brand.toml path must be valid UTF-8");

        let synth = BrandSynthesizer;
        let source = slideforge_plugin_api::BrandSource::TomlFile(Arc::from(path));
        let brand = synth
            .load(&source)
            .expect("BrandProvider::load(TomlFile) must return Ok(Brand)");
        // Verify the palette maps slot names consistent with `brand_from_template`
        // in loader.rs (dk2→primary, acc1→secondary, acc2→accent, lt2→neutral).
        // For this minimal config (dk1="#1F2937", lt1="#FFFFFF", acc1="#3B82F6"):
        //   dk2 is inferred as acc1 = "#3B82F6" (inference rule: dk2 → acc1 if declared)
        //   acc1 = "#3B82F6" (declared)
        //   acc2 is inferred via hue-rotation of acc1
        //   lt2 is inferred as "#F9FAFB"
        assert_eq!(
            brand.palette.primary.as_ref(),
            "#3B82F6",
            "primary must map to dk2 (inferred from acc1 when dk2 absent)"
        );
        assert_eq!(
            brand.palette.secondary.as_ref(),
            "#3B82F6",
            "secondary must map to acc1 from brand.toml"
        );
        // Fonts must be preserved
        assert_eq!(
            brand.fonts.heading.as_ref(),
            "Calibri",
            "heading font must match [fonts].heading from brand.toml"
        );
    }

    /// F1-REGRESSION — `BrandPalette` slot mapping is consistent between synthesizer and loader.
    ///
    /// When a brand.toml declares `dk2` explicitly, `Brand::palette.primary` MUST equal
    /// the dk2 hex value — NOT dk1. This regression test guards against reintroduction
    /// of the positional-index mapping bug where slot[0] (dk1) was used instead of dk2.
    ///
    /// This is a load-bearing assertion per TD-VSDD-059: it asserts on an actual palette
    /// field value, exercising the full `BrandProvider::load` → inference → `BrandPalette`
    /// construction path.
    #[test]
    fn test_f1_regression_brand_palette_primary_maps_to_dk2_not_dk1() {
        use slideforge_plugin_api::BrandProvider as _;
        use std::io::Write as _;
        let tmp_dir = tempfile::tempdir().expect("tempdir must be created");
        let brand_toml_path = tmp_dir.path().join("brand.toml");
        let logo_path = tmp_dir.path().join("logo.png");
        std::fs::write(&logo_path, b"\x89PNG\r\n\x1a\n").expect("logo fixture write");

        let mut tmp =
            std::fs::File::create(&brand_toml_path).expect("brand.toml create must succeed");
        writeln!(
            tmp,
            r##"
[colors]
dk1 = "#000000"
lt1 = "#FFFFFF"
dk2 = "#003087"
lt2 = "#F5F5F5"
acc1 = "#0066CC"
acc2 = "#FF6B35"

[logo]
path = "logo.png"

[fonts]
heading = "Calibri"
body = "Calibri"
"##
        )
        .expect("write to brand.toml must succeed");

        let path = brand_toml_path
            .to_str()
            .expect("brand.toml path must be valid UTF-8");
        let synth = BrandSynthesizer;
        let source = slideforge_plugin_api::BrandSource::TomlFile(Arc::from(path));
        let brand = synth
            .load(&source)
            .expect("BrandProvider::load(TomlFile) must return Ok(Brand)");

        // PRIMARY MUST BE dk2 — this is the regression assertion for F1.
        assert_eq!(
            brand.palette.primary.as_ref(),
            "#003087",
            "palette.primary must equal dk2 ('#003087'), not dk1 ('#000000') — \
             F1 regression: positional slot[0] mapping was a bug"
        );
        assert_eq!(
            brand.palette.secondary.as_ref(),
            "#0066CC",
            "palette.secondary must equal acc1"
        );
        assert_eq!(
            brand.palette.accent.as_ref(),
            "#FF6B35",
            "palette.accent must equal acc2"
        );
        assert_eq!(
            brand.palette.neutral.as_ref(),
            "#F5F5F5",
            "palette.neutral must equal lt2"
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
    ///
    /// Also verifies the message is context-neutral (F-RV3-001): must accurately
    /// describe BOTH the brand synthesis context AND the per-slide overlay context,
    /// and must NOT contain synthesis-specific-only phrasing that would mislead
    /// overlay users.
    #[test]
    fn test_f13_logo_required_has_span_field() {
        let err = BrandError::LogoRequired {
            span: slideforge_types::SourceSpan::default(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("E-BRD-001"),
            "LogoRequired must contain E-BRD-001, got: {msg}"
        );
        assert!(
            msg.contains("logo path"),
            "LogoRequired message must mention logo path, got: {msg}"
        );
        // F-RV3-001: message must cover the brand synthesis context.
        assert!(
            msg.contains("brand.toml"),
            "LogoRequired message must mention brand.toml (synthesis context), got: {msg}"
        );
        // F-RV3-001: message must cover the per-slide overlay context.
        assert!(
            msg.contains("brand_overlay"),
            "LogoRequired message must mention brand_overlay: (overlay context), got: {msg}"
        );
        // F-RV3-001: must NOT contain the old synthesis-only phrasing that misled overlay users.
        assert!(
            !msg.contains("Synthesized brand requires"),
            "LogoRequired message must NOT contain 'Synthesized brand requires' \
             (synthesis-only phrasing misleads overlay users), got: {msg}"
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
        match result.expect_err("F14: synthesize must fail for empty logo path") {
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

    /// DEF-1 — `load_from_toml` resolves the logo path relative to the brand.toml directory.
    ///
    /// After `load_from_toml`, `LogoAsset::Deferred.path` must be the resolved path
    /// (i.e., `<brand.toml directory>/logo.png`), not the as-written relative string.
    /// The as-written value is preserved in `original` (and returned by `deferred_path()`).
    #[test]
    fn test_def1_load_from_toml_resolves_logo_path_relative_to_brand_toml_dir() {
        use std::io::Write as _;
        let tmp_dir = tempfile::tempdir().expect("tempdir must be created");
        let brand_toml_path = tmp_dir.path().join("brand.toml");
        let logo_path = tmp_dir.path().join("logo.png");

        std::fs::write(&logo_path, b"\x89PNG\r\n\x1a\n").expect("logo fixture write must succeed");

        let mut tmp =
            std::fs::File::create(&brand_toml_path).expect("brand.toml create must succeed");
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
        .expect("write to brand.toml must succeed");

        let path_str = brand_toml_path
            .to_str()
            .expect("brand.toml path must be valid UTF-8");
        let (template, _) =
            BrandSynthesizer::load_from_toml(path_str).expect("load_from_toml must succeed");

        let logo = template
            .logo
            .expect("DEF-1: logo must be Some after load_from_toml");
        // original (as-written) is preserved
        assert_eq!(
            logo.deferred_path(),
            Some("logo.png"),
            "DEF-1: original as-written path must be preserved in deferred_path()"
        );
        // resolved path must be relative to the brand.toml directory, not bare "logo.png"
        let resolved = logo
            .resolved_path()
            .expect("DEF-1: resolved_path() must return Some for Deferred logo");
        let expected_resolved = tmp_dir.path().join("logo.png");
        assert_eq!(
            resolved,
            expected_resolved.as_path(),
            "DEF-1: resolved path must be brand.toml-relative, got: {resolved:?}"
        );
    }

    // ─── F-PASS11-LOW-3: snapshot all 31 layouts ─────────────────────────────

    /// F-PASS11-LOW-3 — snapshot all 31 layouts as a determinism guard.
    ///
    /// The three existing snapshots covered only dark/standard `layout_xml` variants.
    /// This test snapshots the complete layout taxonomy: all 31 names, indices,
    /// OOXML types, and color-override flags. The insta snapshot serves as a
    /// regression guard — any change to `generate_all_layouts` that alters the
    /// taxonomy will be caught here.
    #[test]
    fn test_snapshot_all_31_layouts() {
        let config = full_12_color_config();
        let (template, _) = BrandSynthesizer::synthesize(&config).expect("synthesize must succeed");
        assert_eq!(template.layouts.len(), 31, "Expected 31 layouts");

        let lines: Vec<String> = template
            .layouts
            .iter()
            .map(|l| {
                format!(
                    "=== {} (idx={}) type={:?} dark={} ===",
                    l.name,
                    l.index,
                    l.ooxml_type.as_deref().unwrap_or("custom"),
                    l.has_color_override,
                )
            })
            .collect();
        let combined = lines.join("\n");

        insta::assert_snapshot!("all_31_layouts", combined);
    }

    // ─── F-PASS11-LOW-1: layout_names populated from layouts ─────────────────

    /// F-PASS11-LOW-1 — synthesized [`BrandTemplate`] has `layout_names` populated from layouts.
    ///
    /// The dead-state finding required eliminating `layout_names: vec![]` when
    /// `layouts` has 31 entries. After the fix, `layout_names.len() == layouts.len()`
    /// and each name matches `layouts[i].name`.
    #[test]
    fn test_layout_names_populated_from_layouts() {
        let config = full_12_color_config();
        let (template, _) = BrandSynthesizer::synthesize(&config).expect("synthesize must succeed");
        assert_eq!(
            template.layout_names.len(),
            template.layouts.len(),
            "layout_names must have same length as layouts"
        );
        assert_eq!(
            template.layout_names.len(),
            31,
            "both layout_names and layouts must have 31 entries"
        );
        for (i, (name, layout)) in template
            .layout_names
            .iter()
            .zip(template.layouts.iter())
            .enumerate()
        {
            assert_eq!(
                name.as_ref(),
                layout.name.as_ref(),
                "layout_names[{i}] must match layouts[{i}].name"
            );
        }
    }

    // ─── F-PASS11-MED-2: path traversal tests ────────────────────────────────

    /// F-PASS11-MED-2 — logo path with `../..` escaping brand dir is rejected.
    #[test]
    fn test_logo_path_traversal_rejected_with_dotdot() {
        use std::io::Write as _;
        let tmp_dir = tempfile::tempdir().expect("tempdir must be created");
        // Create a file outside the brand dir that the traversal would reach
        let outside_file = tmp_dir.path().join("outside.txt");
        std::fs::write(&outside_file, b"secret").expect("outside file write must succeed");

        // brand.toml lives in a subdirectory
        let brand_dir = tmp_dir.path().join("brand");
        std::fs::create_dir_all(&brand_dir).expect("brand dir create must succeed");
        let brand_toml_path = brand_dir.join("brand.toml");

        let mut f =
            std::fs::File::create(&brand_toml_path).expect("brand.toml create must succeed");
        writeln!(
            f,
            r##"
[colors]
dk1 = "#1F2937"
lt1 = "#FFFFFF"
acc1 = "#3B82F6"

[logo]
path = "../outside.txt"

[fonts]
heading = "Calibri"
body = "Calibri"
"##
        )
        .expect("write to brand.toml must succeed");

        let path_str = brand_toml_path
            .to_str()
            .expect("brand.toml path must be valid UTF-8");
        let result = BrandSynthesizer::load_from_toml(path_str);
        assert!(
            result.is_err(),
            "F-PASS11-MED-2: logo path escaping brand dir must be rejected"
        );
        match result.expect_err("F-PASS11-MED-2: logo path escaping brand dir must be rejected") {
            BrandError::LogoOutsideBrandDir { .. } => {},
            other => {
                panic!("F-PASS11-MED-2: expected BrandError::LogoOutsideBrandDir, got: {other:?}")
            },
        }
    }

    /// F-PASS11-MED-2 — symlink inside brand dir pointing to outside file is rejected.
    #[test]
    #[cfg(unix)] // symlinks on Unix only
    fn test_logo_symlink_to_outside_dir_rejected() {
        use std::io::Write as _;
        let tmp_dir = tempfile::tempdir().expect("tempdir must be created");

        // Create the target file outside the brand dir
        let outside_file = tmp_dir.path().join("secret.png");
        std::fs::write(&outside_file, b"\x89PNG\r\n\x1a\n").expect("secret file write");

        // Brand dir
        let brand_dir = tmp_dir.path().join("brand");
        std::fs::create_dir_all(&brand_dir).expect("brand dir create");

        // Create a symlink inside brand_dir pointing to the outside file
        let symlink_path = brand_dir.join("logo.png");
        std::os::unix::fs::symlink(&outside_file, &symlink_path)
            .expect("symlink create must succeed");

        let brand_toml_path = brand_dir.join("brand.toml");
        let mut f = std::fs::File::create(&brand_toml_path).expect("brand.toml create");
        writeln!(
            f,
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
        .expect("write brand.toml");

        let path_str = brand_toml_path.to_str().expect("path must be UTF-8");
        let result = BrandSynthesizer::load_from_toml(path_str);
        assert!(
            result.is_err(),
            "F-PASS11-MED-2: symlink pointing outside brand dir must be rejected"
        );
        match result.expect_err("F-PASS11-MED-2: symlink outside brand dir must be rejected") {
            BrandError::LogoOutsideBrandDir { .. } => {},
            other => {
                panic!("F-PASS11-MED-2: expected LogoOutsideBrandDir for symlink, got: {other:?}")
            },
        }
    }

    /// F-PASS11-MED-2 — logo inside brand dir is accepted (positive case).
    #[test]
    fn test_logo_inside_brand_dir_accepted() {
        use std::io::Write as _;
        let tmp_dir = tempfile::tempdir().expect("tempdir must be created");
        let logo_path = tmp_dir.path().join("logo.png");
        std::fs::write(&logo_path, b"\x89PNG\r\n\x1a\n").expect("logo write");

        let brand_toml_path = tmp_dir.path().join("brand.toml");
        let mut f = std::fs::File::create(&brand_toml_path).expect("brand.toml create");
        writeln!(
            f,
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
        .expect("write brand.toml");

        let path_str = brand_toml_path.to_str().expect("path must be UTF-8");
        let result = BrandSynthesizer::load_from_toml(path_str);
        assert!(
            result.is_ok(),
            "F-PASS11-MED-2: logo inside brand dir must be accepted"
        );
    }

    // ─── VP-012: proptest — determinism round-trip ─────────────────────────────

    /// Exercises VP-012: palette determinism under randomised `BrandConfig` input.
    ///
    /// Strategy: arbitrary optional hex color strings → `BrandConfig` → synthesize
    /// twice with the same config → assert equal color hex values in both results.
    ///
    /// Generator produces valid `"#RRGGBB"` strings with uppercase hex digits.
    /// Both uppercase and lowercase are accepted by `validate_hex` (F-PASS11-LOW-4:
    /// lowercase is normalised to uppercase); this generator uses uppercase for
    /// consistency with the BC-2.01.004 invariant 3 postcondition.
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

            /// F-PASS5-LOW-2 — invalid hex in acc1 still yields deterministic results.
            ///
            /// Exercises BC-2.01.004 invariant 3 (AC-006): invalid hex values are
            /// treated as absent and inference continues deterministically, emitting
            /// an E-BRD-005 warning for each invalid slot.
            ///
            /// Strategy: fix `acc1` to a known-invalid string (e.g. `"garbage"`),
            /// synthesize twice, and assert that both runs produce identical output
            /// and identical warning counts.
            #[test]
            fn test_bc_2_01_004_invalid_hex_synthesis_is_deterministic(
                invalid_acc1 in prop::string::string_regex("[^#][a-z]{5,10}").expect("valid regex"),
            ) {
                let config = BrandConfig {
                    colors: ColorConfig {
                        dk1: None,
                        lt1: None,
                        dk2: None,
                        lt2: None,
                        acc1: Some(invalid_acc1),
                        acc2: None,
                        acc3: None,
                        acc4: None,
                        acc5: None,
                        acc6: None,
                        hlink: None,
                        fol_hlink: None,
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
                        prop_assert_eq!(t1, t2,
                            "synthesize must be deterministic even with invalid hex input");
                        prop_assert_eq!(w1.len(), w2.len(),
                            "warning count must be deterministic for same invalid-hex input");
                        // At least one E-BRD-005 warning must be emitted for the invalid acc1
                        let has_e_brd_005 = w1.iter().any(|w| {
                            matches!(w, crate::error::BrandError::InvalidHexColor { .. })
                        });
                        prop_assert!(has_e_brd_005,
                            "invalid hex acc1 must yield an E-BRD-005 (InvalidHexColor) warning; \
                             got warnings: {w1:?}");
                    }
                    (Err(_), Err(_)) => {
                        // Deterministic failure is also acceptable.
                    }
                    (Ok(_), Err(e)) | (Err(e), Ok(_)) => {
                        prop_assert!(false,
                            "synthesize produced inconsistent Ok/Err for same invalid-hex input: \
                             {e:?}");
                    }
                }
            }
        }
    }

    /// F-PASS13-HIGH-2 — `strip_unc_prefix` removes `\\?\` prefix on Windows.
    ///
    /// On Windows, `std::fs::canonicalize` may return paths with the `\\?\`
    /// extended-length prefix. This test verifies the helper correctly strips
    /// the prefix so that `starts_with` comparison works even when only one
    /// path has the prefix. On non-Windows, the helper is a no-op.
    #[cfg(windows)]
    #[test]
    fn test_strip_unc_prefix_removes_windows_unc_prefix() {
        use std::path::Path;

        let with_prefix = Path::new(r"\\?\C:\Users\brand\logo.png");
        let without_prefix = Path::new(r"C:\Users\brand\logo.png");

        let stripped = strip_unc_prefix(with_prefix);
        assert_eq!(
            stripped,
            std::path::PathBuf::from(r"C:\Users\brand\logo.png"),
            "strip_unc_prefix must remove \\\\?\\ prefix"
        );

        // Path without prefix is returned unchanged.
        let unchanged = strip_unc_prefix(without_prefix);
        assert_eq!(
            unchanged,
            std::path::PathBuf::from(r"C:\Users\brand\logo.png"),
            "path without prefix must be returned unchanged"
        );

        // After stripping, starts_with comparison is correct.
        let logo_norm = strip_unc_prefix(Path::new(r"\\?\C:\Users\brand\logo.png"));
        let dir_norm = strip_unc_prefix(Path::new(r"C:\Users\brand"));
        assert!(
            logo_norm.starts_with(&dir_norm),
            "after stripping prefix, logo must start_with brand dir"
        );
    }

    /// F-PASS13-HIGH-2 — `strip_unc_prefix` is a no-op on non-Windows.
    #[cfg(not(windows))]
    #[test]
    fn test_strip_unc_prefix_noop_on_non_windows() {
        use std::path::Path;

        let path = Path::new("/tmp/brand/logo.png");
        let result = strip_unc_prefix(path);
        assert_eq!(
            result, path,
            "strip_unc_prefix must be a no-op on non-Windows"
        );
    }

    // ─── F-PASS17-OBS-1: traced_test — instrument spans fire ──────────────────

    /// F-PASS17-OBS-1 — `synthesize` emits a `tracing::debug!` event on success.
    ///
    /// Asserts that `tracing::debug!("synthesize succeeded")` fires when
    /// `BrandSynthesizer::synthesize` completes without error, providing
    /// load-bearing evidence for the `#[instrument]` annotation.
    #[tracing_test::traced_test]
    #[test]
    fn test_f_pass17_obs_1_synthesize_span_emits_debug_event() {
        let config = minimal_config();
        let result = BrandSynthesizer::synthesize(&config);
        assert!(
            result.is_ok(),
            "synthesize must succeed with minimal config"
        );
        assert!(
            logs_contain("synthesize succeeded"),
            "synthesize must emit a 'synthesize succeeded' debug event"
        );
    }

    /// F-PASS17-OBS-1 — `load_from_toml` emits a `tracing::debug!` event on success.
    ///
    /// Writes a minimal brand.toml to a tempdir (with a real logo file) and
    /// asserts that `tracing::debug!("load_from_toml succeeded")` fires.
    #[tracing_test::traced_test]
    #[test]
    fn test_f_pass17_obs_1_load_from_toml_span_emits_debug_event() {
        let tmp_dir = tempfile::tempdir().expect("tempdir must be created");
        let logo_path = tmp_dir.path().join("logo.png");
        std::fs::write(&logo_path, b"PNG").expect("write logo");
        let toml_content = "[logo]\npath = \"logo.png\"\n[colors]\nacc1 = \"#3B82F6\"\n";
        let toml_file = tmp_dir.path().join("brand.toml");
        std::fs::write(&toml_file, toml_content).expect("write brand.toml");
        let result =
            BrandSynthesizer::load_from_toml(toml_file.to_str().expect("valid utf-8 path"));
        assert!(
            result.is_ok(),
            "load_from_toml must succeed with minimal brand.toml"
        );
        assert!(
            logs_contain("load_from_toml succeeded"),
            "load_from_toml must emit a 'load_from_toml succeeded' debug event"
        );
    }
}
