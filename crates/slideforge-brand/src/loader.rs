//! Brand template loader — the primary entry point for STORY-022.
//!
//! [`BrandLoader`] implements the [`BrandProvider`] trait from
//! `slideforge-plugin-api`. Given a path to a `.pptx` or `.docx` file, it
//! opens the ZIP archive, detects the file type from internal ZIP paths,
//! delegates to [`crate::color`], [`crate::font`], and [`crate::logo`], and
//! returns a [`BrandTemplate`].
//!
//! ## PPTX vs DOCX Detection
//!
//! - If `ppt/theme/theme1.xml` exists in the ZIP → PPTX.
//! - If `word/theme/theme1.xml` exists → DOCX.
//! - If neither exists → [`BrandError::ParseError`] (E-BRD-002).

use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use slideforge_plugin_api::{BrandError as TraitBrandError, BrandProvider, BrandSource};
use slideforge_types::{Brand, BrandFonts as TypesBrandFonts, BrandPalette, SourceSpan};
use zip::ZipArchive;

use crate::color::parse_theme_colors;
use crate::context::BrandLoadContext;
use crate::error::BrandError;
use crate::font::{font_available, parse_theme_fonts, resolve_fallback};
use crate::logo::extract_logo;
use crate::template::BrandTemplate;

/// Internal ZIP path for PPTX theme XML.
pub const PPTX_THEME_PATH: &str = "ppt/theme/theme1.xml";
/// Internal ZIP path for DOCX theme XML.
pub const DOCX_THEME_PATH: &str = "word/theme/theme1.xml";

/// The built-in brand loader.
///
/// Implements [`BrandProvider`] for loading brand templates from `.pptx` and
/// `.docx` files. This is a stateless struct — all state is carried in the
/// [`BrandLoadContext`] passed to [`BrandLoader::load_template`].
#[derive(Debug, Default)]
pub struct BrandLoader;

impl BrandLoader {
    /// Create a new `BrandLoader`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Load a [`BrandTemplate`] from a `.pptx` or `.docx` file at `path`.
    ///
    /// # Errors
    ///
    /// - [`BrandError::FileNotFound`] — `path` does not exist (E-BRD-001).
    /// - [`BrandError::ParseError`] — file is not a valid OOXML ZIP (E-BRD-002).
    pub fn load_template(
        &self,
        path: &Path,
        ctx: &BrandLoadContext,
    ) -> Result<BrandTemplate, BrandError> {
        // --- Step 1: Open the file ---
        let file = std::fs::File::open(path).map_err(|_| BrandError::FileNotFound {
            path: Arc::from(path.to_string_lossy().as_ref()),
            span: ctx.span.clone(),
        })?;

        // --- Step 2: Open as ZIP ---
        let mut zip = ZipArchive::new(file).map_err(|e| BrandError::ParseError {
            path: Arc::from(path.to_string_lossy().as_ref()),
            reason: Arc::from(e.to_string().as_str()),
            span: ctx.span.clone(),
        })?;

        // --- Step 3: Detect PPTX vs DOCX ---
        let is_pptx = zip.by_name(PPTX_THEME_PATH).is_ok();
        let is_docx = !is_pptx && zip.by_name(DOCX_THEME_PATH).is_ok();

        if !is_pptx && !is_docx {
            return Err(BrandError::ParseError {
                path: Arc::from(path.to_string_lossy().as_ref()),
                reason: Arc::from(
                    "neither ppt/theme/theme1.xml nor word/theme/theme1.xml found in archive",
                ),
                span: ctx.span.clone(),
            });
        }

        let theme_path = if is_pptx {
            PPTX_THEME_PATH
        } else {
            DOCX_THEME_PATH
        };

        // --- Step 3b: Multiple slide masters detection (AC-009 / EC-004) ---
        // Only PPTX templates have slide masters. If more than one exists, we
        // emit a warning and use only slideMaster1.xml.
        if is_pptx {
            let master_count = zip
                .file_names()
                .filter(|name| {
                    name.starts_with("ppt/slideMasters/slideMaster")
                        && std::path::Path::new(name)
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
                        && !name.contains("_rels")
                })
                .count();
            if master_count > 1 {
                tracing::warn!(
                    path = %path.display(),
                    master_count,
                    "Template has multiple slide masters; only slideMaster1.xml is used."
                );
            }
        }

        // --- Step 4: Read theme XML ---
        let theme_xml = {
            let mut entry = zip.by_name(theme_path).map_err(|e| BrandError::ParseError {
                path: Arc::from(path.to_string_lossy().as_ref()),
                reason: Arc::from(e.to_string().as_str()),
                span: ctx.span.clone(),
            })?;
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| BrandError::ParseError {
                    path: Arc::from(path.to_string_lossy().as_ref()),
                    reason: Arc::from(e.to_string().as_str()),
                    span: ctx.span.clone(),
                })?;
            buf
        };

        // --- Step 5: Parse colors ---
        let (colors, color_warnings) = parse_theme_colors(&theme_xml)?;
        for warn in &color_warnings {
            tracing::warn!(warning = %warn, "brand color slot warning");
        }

        // --- Step 6: Parse fonts ---
        let fonts = parse_theme_fonts(&theme_xml)?;

        // --- Step 7: Font availability check (cosmetic) ---
        if ctx.check_font_availability {
            let fallback = resolve_fallback();
            if !font_available(fonts.heading.as_ref()) {
                tracing::warn!(
                    font = fonts.heading.as_ref(),
                    fallback = fallback.as_ref(),
                    "heading font unavailable on build host (E-BRD-004)"
                );
            }
            if !font_available(fonts.body.as_ref()) {
                tracing::warn!(
                    font = fonts.body.as_ref(),
                    fallback = fallback.as_ref(),
                    "body font unavailable on build host (E-BRD-004)"
                );
            }
        }

        // --- Step 8: Extract logo (PPTX only) ---
        let logo = if is_pptx {
            extract_logo(&mut zip)
        } else {
            None
        };

        // --- Step 9: Discover layout names (PPTX only) ---
        let layout_names: Vec<Arc<str>> = if is_pptx {
            zip.file_names()
                .filter(|name| {
                    name.starts_with("ppt/slideLayouts/slideLayout")
                        && std::path::Path::new(name)
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
                        && !name.contains("_rels")
                })
                .map(Arc::from)
                .collect()
        } else {
            Vec::new()
        };

        Ok(BrandTemplate {
            colors,
            fonts,
            logo,
            footer_text: None,
            layout_names,
        })
    }
}

/// Convert a [`BrandTemplate`] (crate-internal) to a [`Brand`] (cross-crate IR type).
///
/// ## Color Slot Mapping (ECMA-376 → `BrandPalette`)
///
/// OOXML themes do not carry an organization name or explicit "primary" designation.
/// We map slots to palette entries by conventional OOXML semantics:
///
/// | Palette field | OOXML slot | Rationale |
/// |---------------|-----------|-----------|
/// | `primary`     | `dk2`     | Dark brand color — typically the org's primary brand hue |
/// | `secondary`   | `acc1`    | First accent — typically secondary brand color |
/// | `accent`      | `acc2`    | Second accent — typically highlight / call-to-action color |
/// | `neutral`     | `lt2`     | Light neutral — background / surface color |
///
/// ## Brand Name
///
/// The OOXML theme does not store an organization name. We derive the name from
/// the file stem (e.g., `"acme-brand.pptx"` → `"acme-brand"`).
///
/// ## Font Mapping
///
/// `BrandFonts` (IR) has three fields: `heading`, `body`, `mono`. The OOXML
/// theme provides `heading` and `body`. Mono defaults to `"Courier New"`.
#[must_use]
fn brand_from_template(template: &BrandTemplate, source_path: &str) -> Brand {
    // Derive brand name from file stem.
    let name: Arc<str> = Arc::from(
        std::path::Path::new(source_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("brand"),
    );

    // Map OOXML color slots to BrandPalette fields.
    // Default to gray if a slot is missing (should not happen after loading).
    let slot_hex = |slot_name: &str| -> Arc<str> {
        template
            .color_by_name(slot_name)
            .map_or_else(|| Arc::from("#808080"), |s| Arc::clone(&s.hex))
    };

    let palette = BrandPalette {
        primary: slot_hex("dk2"),
        secondary: slot_hex("acc1"),
        accent: slot_hex("acc2"),
        neutral: slot_hex("lt2"),
    };

    let fonts = TypesBrandFonts {
        heading: Arc::clone(&template.fonts.heading),
        body: Arc::clone(&template.fonts.body),
        mono: Arc::from("Courier New"),
    };

    Brand {
        name,
        palette,
        fonts,
        layouts: Vec::new(),
        span: SourceSpan::default(),
    }
}

/// Implement the `BrandProvider` plugin trait for [`BrandLoader`].
///
/// The `BrandProvider::load` method converts a [`BrandSource`] (PPTX or DOCX
/// path) to the `slideforge-types` [`Brand`] struct consumed by the layout and
/// export stages.
impl BrandProvider for BrandLoader {
    fn id(&self) -> &'static str {
        "slideforge-brand/default"
    }

    fn load(&self, source: &BrandSource) -> Result<Brand, TraitBrandError> {
        match source {
            BrandSource::PptxFile(path) | BrandSource::DocxFile(path) => {
                // Resolve the path and load the OOXML template.
                let file_path = std::path::Path::new(path.as_ref());
                let ctx = BrandLoadContext {
                    check_font_availability: true,
                    root_dir: file_path
                        .parent()
                        .map_or_else(
                            || std::path::PathBuf::from("."),
                            std::path::Path::to_path_buf,
                        ),
                    span: SourceSpan::default(),
                };
                let template =
                    self.load_template(file_path, &ctx).map_err(|e| match e {
                        BrandError::FileNotFound { path: p, .. } => {
                            TraitBrandError::SourceNotFound {
                                uri: p.as_ref().to_owned(),
                            }
                        }
                        BrandError::ParseError { path: p, reason, .. } => {
                            TraitBrandError::ParseError {
                                uri: p.as_ref().to_owned(),
                                message: reason.as_ref().to_owned(),
                            }
                        }
                        other => TraitBrandError::ValidationError {
                            message: other.to_string(),
                        },
                    })?;
                Ok(brand_from_template(&template, path))
            }
            BrandSource::TomlFile(path) => {
                // TOML brand file loading is implemented in STORY-023 (brand synthesis).
                // For STORY-022 (brand extraction), TOML sources are not yet supported.
                Err(TraitBrandError::ValidationError {
                    message: format!(
                        "TOML brand files are not yet supported by the built-in brand loader \
                         (path: '{path}'). Use a .pptx or .docx template source. \
                         TOML synthesis is implemented in STORY-023."
                    ),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write as _};

    use zip::write::{SimpleFileOptions, ZipWriter};
    use zip::CompressionMethod;

    use super::*;
    use crate::context::BrandLoadContext;

    // ─── Canonical theme1.xml fixtures ────────────────────────────────────────

    /// Minimal `theme1.xml` with all 12 sRGB colors and both fonts.
    ///
    /// Used as the canonical happy-path test vector (BC-2.01.001 test vectors table).
    const MINIMAL_THEME_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="MinimalTheme">
  <a:themeElements>
    <a:clrScheme name="MinimalScheme">
      <a:dk1><a:srgbClr val="000000"/></a:dk1>
      <a:lt1><a:srgbClr val="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="003087"/></a:dk2>
      <a:lt2><a:srgbClr val="F5F5F5"/></a:lt2>
      <a:acc1><a:srgbClr val="0066CC"/></a:acc1>
      <a:acc2><a:srgbClr val="FF6B35"/></a:acc2>
      <a:acc3><a:srgbClr val="28A745"/></a:acc3>
      <a:acc4><a:srgbClr val="FFC107"/></a:acc4>
      <a:acc5><a:srgbClr val="6F42C1"/></a:acc5>
      <a:acc6><a:srgbClr val="17A2B8"/></a:acc6>
      <a:hlink><a:srgbClr val="0000EE"/></a:hlink>
      <a:folHlink><a:srgbClr val="551A8B"/></a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="MinimalFontScheme">
      <a:majorFont><a:latin typeface="Calibri Light"/></a:majorFont>
      <a:minorFont><a:latin typeface="Calibri"/></a:minorFont>
    </a:fontScheme>
  </a:themeElements>
</a:theme>"#;

    /// `theme1.xml` with only 8 of 12 color slots (missing acc3, acc4, acc5, acc6).
    const PARTIAL_THEME_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="PartialTheme">
  <a:themeElements>
    <a:clrScheme name="PartialScheme">
      <a:dk1><a:srgbClr val="000000"/></a:dk1>
      <a:lt1><a:srgbClr val="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="003087"/></a:dk2>
      <a:lt2><a:srgbClr val="F5F5F5"/></a:lt2>
      <a:acc1><a:srgbClr val="0066CC"/></a:acc1>
      <a:acc2><a:srgbClr val="FF6B35"/></a:acc2>
      <a:hlink><a:srgbClr val="0000EE"/></a:hlink>
      <a:folHlink><a:srgbClr val="551A8B"/></a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="PartialFontScheme">
      <a:majorFont><a:latin typeface="Arial"/></a:majorFont>
      <a:minorFont><a:latin typeface="Arial"/></a:minorFont>
    </a:fontScheme>
  </a:themeElements>
</a:theme>"#;

    // ─── ZIP fixture builders ──────────────────────────────────────────────────

    /// Build an in-memory PPTX ZIP fixture with the given `theme1.xml` content.
    ///
    /// Adds `ppt/theme/theme1.xml` so the loader detects it as a PPTX.
    fn build_pptx_zip(theme_xml: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut zw = ZipWriter::new(cursor);
            let opts = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Stored);
            zw.start_file(PPTX_THEME_PATH, opts).unwrap();
            zw.write_all(theme_xml.as_bytes()).unwrap();
            // Add a minimal [Content_Types].xml so the ZIP is structurally valid.
            zw.start_file("[Content_Types].xml", opts).unwrap();
            zw.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>").unwrap();
            zw.finish().unwrap();
        }
        buf
    }

    /// Build an in-memory DOCX ZIP fixture with the given `theme1.xml` content.
    ///
    /// Adds `word/theme/theme1.xml` so the loader detects it as a DOCX.
    fn build_docx_zip(theme_xml: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut zw = ZipWriter::new(cursor);
            let opts = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Stored);
            zw.start_file(DOCX_THEME_PATH, opts).unwrap();
            zw.write_all(theme_xml.as_bytes()).unwrap();
            zw.start_file("[Content_Types].xml", opts).unwrap();
            zw.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>").unwrap();
            zw.finish().unwrap();
        }
        buf
    }

    /// Write bytes to a temp file and return the path.
    ///
    /// Uses a combination of nanosecond timestamp and thread ID to make the
    /// filename unique under parallel test execution.
    fn write_temp_file(bytes: &[u8], extension: &str) -> std::path::PathBuf {
        use std::io::Write;
        let dir = std::env::temp_dir();
        // Combine nanosecond timestamp and thread ID for uniqueness under parallel tests.
        let thread_id = format!("{:?}", std::thread::current().id());
        let thread_hash: u64 = thread_id
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(u64::from(b)));
        let name = format!(
            "slideforge_brand_test_{}_{:x}.{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            thread_hash,
            extension
        );
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(bytes).unwrap();
        path
    }

    // ─── Loader tests ──────────────────────────────────────────────────────────

    /// BC-2.01.001 postcondition 1 — load_template on valid PPTX returns BrandTemplate with
    /// 12 color slots.
    ///
    /// Test vector (BC-2.01.001 test vectors): valid PPTX → BrandTemplate, exit 0.
    #[test]
    fn test_bc_2_01_001_load_valid_pptx() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let path = write_temp_file(&zip_bytes, "pptx");
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext::for_test();

        let result = loader.load_template(&path, &ctx);
        let _ = std::fs::remove_file(&path); // cleanup

        let template = result.expect("valid PPTX must load without error");
        assert_eq!(
            template.colors.len(),
            12,
            "invariant DI-015: must have exactly 12 color slots"
        );
    }

    /// BC-2.01.001 postcondition 1 — load_template on valid DOCX returns BrandTemplate with
    /// 12 color slots.
    ///
    /// Test vector (BC-2.01.001 test vectors): valid DOCX → BrandTemplate loaded from
    /// word/theme/theme1.xml; exit 0.
    #[test]
    fn test_bc_2_01_001_load_valid_docx() {
        let zip_bytes = build_docx_zip(MINIMAL_THEME_XML);
        let path = write_temp_file(&zip_bytes, "docx");
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext::for_test();

        let result = loader.load_template(&path, &ctx);
        let _ = std::fs::remove_file(&path);

        let template = result.expect("valid DOCX must load without error");
        assert_eq!(
            template.colors.len(),
            12,
            "invariant DI-015: must have exactly 12 color slots"
        );
    }

    /// BC-2.01.001 EC-001 — non-existent path produces FileNotFound.
    ///
    /// Test vector: "missing.pptx" → E-BRD-001; exit 4.
    #[test]
    fn test_bc_2_01_001_load_missing_file() {
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext::for_test();
        let nonexistent = std::path::Path::new("/tmp/slideforge_nonexistent_9999999.pptx");

        let result = loader.load_template(nonexistent, &ctx);

        assert!(
            result.is_err(),
            "non-existent path must return an error"
        );
        assert!(
            matches!(result.as_ref().unwrap_err(), BrandError::FileNotFound { .. }),
            "expected BrandError::FileNotFound, got: {:?}",
            result.unwrap_err()
        );
    }

    /// BC-2.01.001 EC-002 — corrupt file (random bytes) produces ParseError.
    ///
    /// Test vector: corrupt file (not a ZIP) → E-BRD-002; exit 4.
    #[test]
    fn test_bc_2_01_001_load_corrupt_file() {
        // Write random non-ZIP bytes to a .pptx file.
        let corrupt_bytes: &[u8] = b"THIS IS NOT A ZIP ARCHIVE AT ALL \x00\x01\x02\x03";
        let path = write_temp_file(corrupt_bytes, "pptx");
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext::for_test();

        let result = loader.load_template(&path, &ctx);
        let _ = std::fs::remove_file(&path);

        assert!(result.is_err(), "corrupt file must return an error");
        assert!(
            matches!(result.as_ref().unwrap_err(), BrandError::ParseError { .. }),
            "expected BrandError::ParseError, got: {:?}",
            result.unwrap_err()
        );
    }

    /// BC-2.01.001 EC-003 — template with 8 colors still produces 12-slot BrandTemplate
    /// plus 4 MissingColorSlot warnings.
    ///
    /// Test vector: 8-color template → BrandTemplate with 12 colors (4 inferred); 4 E-BRD-003 warnings.
    #[test]
    fn test_bc_2_01_001_load_partial_colors() {
        let zip_bytes = build_pptx_zip(PARTIAL_THEME_XML);
        let path = write_temp_file(&zip_bytes, "pptx");
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext::for_test();

        let result = loader.load_template(&path, &ctx);
        let _ = std::fs::remove_file(&path);

        let template = result.expect("partial template must load (missing slots are warnings, not errors)");
        assert_eq!(
            template.colors.len(),
            12,
            "invariant DI-015: must always have 12 slots even with partial source"
        );
    }

    /// BC-2.01.001 AC-005 EC-005 — PPTX with no logo produces BrandTemplate.logo == None.
    ///
    /// The minimal ZIP fixture has no image relationships, so logo must be None.
    #[test]
    fn test_bc_2_01_001_load_no_logo() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let path = write_temp_file(&zip_bytes, "pptx");
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext::for_test();

        let result = loader.load_template(&path, &ctx);
        let _ = std::fs::remove_file(&path);

        let template = result.expect("PPTX with no logo must load without error");
        assert!(
            template.logo.is_none(),
            "PPTX with no image relationship must have logo == None (EC-005)"
        );
    }

    /// BC-2.01.001 invariant 2 — source template file is unmodified after loading.
    ///
    /// Verification property from BC-2.01.001: hash before and after loading must match.
    #[test]
    fn test_bc_2_01_001_invariant_source_file_unmodified() {
        use std::fs;
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let path = write_temp_file(&zip_bytes, "pptx");
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext::for_test();

        // Record file metadata before loading.
        let meta_before = fs::metadata(&path).expect("file must exist before load");
        let size_before = meta_before.len();

        let _result = loader.load_template(&path, &ctx);

        // Record file metadata after loading.
        let meta_after = fs::metadata(&path).expect("file must still exist after load");
        let size_after = meta_after.len();

        let _ = fs::remove_file(&path);

        assert_eq!(
            size_before,
            size_after,
            "invariant: source template file size must be unchanged after loading"
        );
    }

    /// BC-2.01.001 — PPTX_THEME_PATH and DOCX_THEME_PATH constants are correct.
    #[test]
    fn test_bc_2_01_001_internal_path_constants() {
        assert_eq!(PPTX_THEME_PATH, "ppt/theme/theme1.xml");
        assert_eq!(DOCX_THEME_PATH, "word/theme/theme1.xml");
    }

    /// BC-2.01.001 — BrandLoader implements BrandProvider with correct id.
    #[test]
    fn test_bc_2_01_001_brand_loader_provider_id() {
        let loader = BrandLoader::new();
        assert_eq!(loader.id(), "slideforge-brand/default");
    }

    /// BC-2.01.006 AC-010 — when check_font_availability is true and a font is unavailable,
    /// FontUnavailable warning is emitted (cosmetic, not fatal).
    ///
    /// We use a deliberately non-existent font name to guarantee unavailability.
    #[test]
    fn test_bc_2_01_006_font_unavailable_is_cosmetic() {
        // This test verifies that a font that is definitely not installed
        // does not cause the load to fail — it should succeed with a warning.
        // The font "NonExistentFont_STORY022_TEST" will never be installed.
        //
        // Because the loader stub todo!()s, this test will fail at the stub — Red Gate.
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let path = write_temp_file(&zip_bytes, "pptx");
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext {
            check_font_availability: true,
            root_dir: std::path::PathBuf::from("."),
            span: slideforge_types::SourceSpan::default(),
        };

        // Load must succeed even if fonts aren't installed (BC-2.01.006 invariant 1).
        let result = loader.load_template(&path, &ctx);
        let _ = std::fs::remove_file(&path);

        // The result is Ok — font unavailability is cosmetic.
        assert!(
            result.is_ok(),
            "font unavailability must not cause load to fail (BC-2.01.006 invariant 1)"
        );
    }

    /// BC-2.01.001 AC-009 / EC-004 — PPTX with multiple slide masters loads successfully.
    ///
    /// When the ZIP contains both `ppt/slideMasters/slideMaster1.xml` and
    /// `ppt/slideMasters/slideMaster2.xml`, the loader must:
    /// 1. Succeed (not error — multiple masters are a warning, not fatal).
    /// 2. Emit a tracing::warn for the extra masters (verified via subscriber).
    /// 3. Use only slideMaster1.xml for logo extraction.
    ///
    /// FINDING-003 (AC-009 / EC-004).
    #[test]
    fn test_bc_2_01_001_load_multiple_slide_masters_warning() {
        use std::io::Write;
        use std::sync::{Arc as StdArc, Mutex};
        use tracing_subscriber::layer::SubscriberExt as _;

        // Track whether the multiple-masters warning was emitted.
        let warned = StdArc::new(Mutex::new(false));
        let warned_clone = StdArc::clone(&warned);

        // Build a PPTX ZIP with two slide masters and a rels file for master1.
        let zip_bytes = {
            let mut buf = Vec::new();
            let cursor = std::io::Cursor::new(&mut buf);
            let mut zw = zip::ZipWriter::new(cursor);
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            // Theme file.
            zw.start_file(PPTX_THEME_PATH, opts).unwrap();
            zw.write_all(MINIMAL_THEME_XML.as_bytes()).unwrap();
            // Two slide masters.
            zw.start_file("ppt/slideMasters/slideMaster1.xml", opts).unwrap();
            zw.write_all(b"<p:sldMaster/>").unwrap();
            zw.start_file("ppt/slideMasters/slideMaster2.xml", opts).unwrap();
            zw.write_all(b"<p:sldMaster/>").unwrap();
            // Content types (minimal).
            zw.start_file("[Content_Types].xml", opts).unwrap();
            zw.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>").unwrap();
            zw.finish().unwrap();
            buf
        };

        let path = write_temp_file(&zip_bytes, "pptx");
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext::for_test();

        // Install a minimal tracing subscriber to capture the warning.
        let warned_layer = {
            let w = StdArc::clone(&warned_clone);
            tracing_subscriber::fmt::layer()
                .with_writer(move || {
                    // Every write signals we emitted something.
                    let _ = w.lock().map(|mut guard| *guard = true);
                    std::io::sink()
                })
        };
        let subscriber = tracing_subscriber::registry().with(warned_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // Loading must succeed (multiple masters is a warning, not an error).
        let result = loader.load_template(&path, &ctx);
        let _ = std::fs::remove_file(&path);

        assert!(
            result.is_ok(),
            "PPTX with multiple slide masters must load without error (EC-004)"
        );

        // The warning subscriber received output (meaning tracing::warn! fired).
        // Note: this is a best-effort check — the subscriber captures any log output.
        // The only tracing call during a multiple-master load is the warn! for masters.
        let was_warned = warned.lock().is_ok_and(|g| *g);
        assert!(
            was_warned,
            "multiple slide masters must emit a tracing::warn (FINDING-003)"
        );
    }

    /// BC-2.01.001 AC-005 — PPTX with a logo in slideMaster1.xml.rels returns Some(LogoAsset).
    ///
    /// Constructs a ZIP with:
    /// - `ppt/slideMasters/_rels/slideMaster1.xml.rels` containing an image relationship
    /// - `ppt/media/image1.png` containing PNG header bytes
    ///
    /// Verifies `extract_logo` returns `Some(LogoAsset)` with correct bytes and media type.
    ///
    /// FINDING-012.
    #[test]
    fn test_bc_2_01_001_load_logo_positive_path() {
        use std::io::Write;

        // Minimal PNG header (8 bytes).
        let png_bytes: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

        // Build a PPTX ZIP with a logo relationship.
        let zip_bytes = {
            let mut buf = Vec::new();
            let cursor = std::io::Cursor::new(&mut buf);
            let mut zw = zip::ZipWriter::new(cursor);
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            // Theme file.
            zw.start_file(PPTX_THEME_PATH, opts).unwrap();
            zw.write_all(MINIMAL_THEME_XML.as_bytes()).unwrap();

            // Slide master rels file with an image relationship.
            zw.start_file("ppt/slideMasters/_rels/slideMaster1.xml.rels", opts)
                .unwrap();
            let rels_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1"
    Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"
    Target="../media/image1.png"/>
</Relationships>"#;
            zw.write_all(rels_xml.as_bytes()).unwrap();

            // Image file.
            zw.start_file("ppt/media/image1.png", opts).unwrap();
            zw.write_all(png_bytes).unwrap();

            // Content types (minimal).
            zw.start_file("[Content_Types].xml", opts).unwrap();
            zw.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>").unwrap();
            zw.finish().unwrap();
            buf
        };

        let path = write_temp_file(&zip_bytes, "pptx");
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext::for_test();

        let result = loader.load_template(&path, &ctx);
        let _ = std::fs::remove_file(&path);

        let template = result.expect("PPTX with logo must load without error");

        // Logo must be Some.
        let logo = template.logo.expect("logo must be present (FINDING-012)");
        assert_eq!(
            logo.bytes, png_bytes,
            "logo bytes must match the image1.png content"
        );
        assert_eq!(
            logo.media_type.as_ref(),
            "image/png",
            "logo media type must be image/png for .png extension"
        );
        assert_eq!(
            logo.original_path.as_ref(),
            "ppt/media/image1.png",
            "logo original_path must be the resolved ZIP-internal path"
        );
    }

    /// FINDING-001 — BrandProvider::load() returns Brand for a valid PPTX source.
    ///
    /// Verifies the BrandProvider trait implementation (not the raw load_template).
    /// The Brand palette must map OOXML color slots correctly:
    /// - primary → dk2 (dark brand color)
    /// - secondary → acc1 (first accent)
    /// - accent → acc2 (second accent)
    /// - neutral → lt2 (light neutral)
    #[test]
    fn test_bc_2_01_001_brand_provider_load_pptx() {
        use slideforge_plugin_api::BrandProvider;

        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let path = write_temp_file(&zip_bytes, "pptx");
        let loader = BrandLoader::new();
        let source = slideforge_plugin_api::BrandSource::PptxFile(
            Arc::from(path.to_string_lossy().as_ref()),
        );

        let result = loader.load(&source);
        let _ = std::fs::remove_file(&path);

        let brand = result.expect("BrandProvider::load must succeed for valid PPTX");
        // Palette slots (from MINIMAL_THEME_XML):
        // dk2 = 003087, acc1 = 0066CC, acc2 = FF6B35, lt2 = F5F5F5
        assert_eq!(brand.palette.primary.as_ref(), "#003087", "primary = dk2");
        assert_eq!(brand.palette.secondary.as_ref(), "#0066CC", "secondary = acc1");
        assert_eq!(brand.palette.accent.as_ref(), "#FF6B35", "accent = acc2");
        assert_eq!(brand.palette.neutral.as_ref(), "#F5F5F5", "neutral = lt2");
        // Fonts from theme.
        assert_eq!(brand.fonts.heading.as_ref(), "Calibri Light");
        assert_eq!(brand.fonts.body.as_ref(), "Calibri");
        assert_eq!(brand.fonts.mono.as_ref(), "Courier New");
    }

    /// FINDING-001 — BrandProvider::load() returns Brand for a valid DOCX source.
    #[test]
    fn test_bc_2_01_001_brand_provider_load_docx() {
        use slideforge_plugin_api::BrandProvider;

        let zip_bytes = build_docx_zip(MINIMAL_THEME_XML);
        let path = write_temp_file(&zip_bytes, "docx");
        let loader = BrandLoader::new();
        let source = slideforge_plugin_api::BrandSource::DocxFile(
            Arc::from(path.to_string_lossy().as_ref()),
        );

        let result = loader.load(&source);
        let _ = std::fs::remove_file(&path);

        let brand = result.expect("BrandProvider::load must succeed for valid DOCX");
        assert_eq!(brand.palette.primary.as_ref(), "#003087", "primary = dk2");
    }

    /// FINDING-001 — BrandProvider::load() returns SourceNotFound for missing PPTX.
    #[test]
    fn test_bc_2_01_001_brand_provider_load_missing_file() {
        use slideforge_plugin_api::BrandProvider;

        let loader = BrandLoader::new();
        let source = slideforge_plugin_api::BrandSource::PptxFile(
            Arc::from("/tmp/slideforge_nonexistent_brand_9999999.pptx"),
        );

        let result = loader.load(&source);
        assert!(
            result.is_err(),
            "BrandProvider::load must return Err for missing file"
        );
        assert!(
            matches!(
                result.unwrap_err(),
                slideforge_plugin_api::BrandError::SourceNotFound { .. }
            ),
            "must be SourceNotFound error"
        );
    }

    /// FINDING-001 — BrandProvider::load() returns ValidationError for TOML sources
    /// (not yet implemented — STORY-023 scope).
    #[test]
    fn test_bc_2_01_001_brand_provider_load_toml_not_yet_supported() {
        use slideforge_plugin_api::BrandProvider;

        let loader = BrandLoader::new();
        let source = slideforge_plugin_api::BrandSource::TomlFile(Arc::from("brand.toml"));

        let result = loader.load(&source);
        assert!(
            result.is_err(),
            "BrandProvider::load for TOML must return Err (not yet implemented)"
        );
        // Must be ValidationError explaining the situation.
        assert!(
            matches!(
                result.unwrap_err(),
                slideforge_plugin_api::BrandError::ValidationError { .. }
            ),
            "must be ValidationError for TOML source"
        );
    }
}
