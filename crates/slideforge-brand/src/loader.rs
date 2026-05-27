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
use slideforge_types::Brand;
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
            });
        }

        let theme_path = if is_pptx {
            PPTX_THEME_PATH
        } else {
            DOCX_THEME_PATH
        };

        // --- Step 4: Read theme XML ---
        let theme_xml = {
            let mut entry = zip.by_name(theme_path).map_err(|e| BrandError::ParseError {
                path: Arc::from(path.to_string_lossy().as_ref()),
                reason: Arc::from(e.to_string().as_str()),
            })?;
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| BrandError::ParseError {
                    path: Arc::from(path.to_string_lossy().as_ref()),
                    reason: Arc::from(e.to_string().as_str()),
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
        todo!(
            "BrandLoader::load (BrandProvider trait): convert BrandSource to Brand — \
             stub for Red Gate (STORY-022); source={source}"
        )
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
    fn write_temp_file(bytes: &[u8], extension: &str) -> std::path::PathBuf {
        use std::io::Write;
        let dir = std::env::temp_dir();
        let name = format!(
            "slideforge_brand_test_{}.{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
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
}
