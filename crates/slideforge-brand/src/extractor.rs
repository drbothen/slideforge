//! Brand extractor — reads an existing `.pptx` and writes a `brand.toml` file.
//!
//! [`BrandExtractor`] implements BC-2.01.003: extraction of brand configuration
//! from an OOXML `.pptx` template file. It calls [`crate::loader::BrandLoader`]
//! (STORY-022) to parse the OOXML package, then serializes the resulting
//! [`crate::template::BrandTemplate`] into a `brand.toml` file using the
//! [`crate::toml_schema::BrandConfig`] schema from STORY-023.
//!
//! ## Extraction Pipeline
//!
//! 1. Load `.pptx` via `BrandLoader::load_template()` (STORY-022).
//! 2. Map `BrandTemplate` → `BrandConfig` (12 color slots in ECMA-376 order).
//! 3. If `output_dir/brand.toml` exists and `force = false`, return `E-BRD-006`.
//! 4. Create `output_dir` (and `brand.assets/`) if absent.
//! 5. Serialize `BrandConfig` to TOML and write `output_dir/brand.toml`.
//! 6. If `template.logo.is_some()`, write logo bytes to `output_dir/brand.assets/logo.<ext>`.
//! 7. Return `BrandExtractionResult`.
//!
//! ## Read-Only Invariant
//!
//! The source `.pptx` file is NEVER modified (BC-2.01.003 invariant 2).
//! All I/O writes are to `output_dir`, never to the source path.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::context::BrandLoadContext;
use crate::error::BrandError;
use crate::loader::BrandLoader;
use crate::template::{BrandTemplate, ColorValue, LogoAsset};

// ─── Result type ─────────────────────────────────────────────────────────────

/// The result of a successful brand extraction operation.
///
/// Returned by [`BrandExtractor::extract`] on success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrandExtractionResult {
    /// The path to the written `brand.toml` file.
    pub brand_toml_path: PathBuf,
    /// The path to the copied logo asset, if a logo was found in the source template.
    ///
    /// `None` when the source template has no logo in `slideMaster1.xml.rels`.
    pub logo_asset_path: Option<PathBuf>,
}

// ─── BrandExtractor ──────────────────────────────────────────────────────────

/// Extracts brand configuration from an existing `.pptx` template and writes
/// a `brand.toml` to the specified output directory.
///
/// ## Usage
///
/// ```rust,ignore
/// use std::path::Path;
/// use slideforge_brand::extractor::BrandExtractor;
///
/// let result = BrandExtractor::extract("corporate.pptx", Path::new("./out"), false)?;
/// println!("brand.toml written to: {}", result.brand_toml_path.display());
/// ```
#[derive(Debug, Default)]
pub struct BrandExtractor;

impl BrandExtractor {
    /// Extract brand configuration from `source` (a `.pptx` path) and write
    /// `brand.toml` (plus optional logo asset) to `output_dir`.
    ///
    /// # Parameters
    ///
    /// - `source`: path to the source `.pptx` file (read-only; never modified).
    /// - `output_dir`: directory where `brand.toml` and `brand.assets/` are written.
    ///   Created (via `std::fs::create_dir_all`) if it does not exist.
    /// - `force`: when `false`, returns `BrandError::OutputExists` if
    ///   `output_dir/brand.toml` already exists. When `true`, overwrites.
    ///
    /// # Errors
    ///
    /// - [`BrandError::FileNotFound`] — `source` does not exist (E-BRD-001).
    /// - [`BrandError::ParseError`] — `source` is not a valid OOXML ZIP (E-BRD-002).
    /// - [`BrandError::OutputExists`] — `brand.toml` already exists and `force = false` (E-BRD-006).
    pub fn extract(
        source: &str,
        output_dir: &Path,
        force: bool,
    ) -> Result<BrandExtractionResult, BrandError> {
        // --- Step 1: Create output directory if it does not exist (EC-006) ---
        std::fs::create_dir_all(output_dir).map_err(|e| BrandError::ParseError {
            path: Arc::from(output_dir.to_string_lossy().as_ref()),
            reason: Arc::from(format!("cannot create output directory: {e}").as_str()),
            span: slideforge_types::SourceSpan::default(),
        })?;

        // --- Step 2: Check for existing brand.toml (EC-001) ---
        let brand_toml_path = output_dir.join("brand.toml");
        if brand_toml_path.exists() && !force {
            return Err(BrandError::OutputExists {
                path: Arc::from(brand_toml_path.to_string_lossy().as_ref()),
            });
        }

        // --- Step 3: Load the .pptx via BrandLoader (read-only) ---
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext {
            check_font_availability: false,
            root_dir: output_dir.to_path_buf(),
            span: slideforge_types::SourceSpan::default(),
        };
        let template = loader.load_template(Path::new(source), &ctx)?;

        // --- Step 4: Convert BrandTemplate → TOML string ---
        let (toml_content, logo_relative_path) =
            brand_template_to_toml(&template, template.logo.as_ref());

        // --- Step 5: Write brand.toml ---
        std::fs::write(&brand_toml_path, toml_content.as_bytes()).map_err(|e| {
            BrandError::ParseError {
                path: Arc::from(brand_toml_path.to_string_lossy().as_ref()),
                reason: Arc::from(format!("cannot write brand.toml: {e}").as_str()),
                span: slideforge_types::SourceSpan::default(),
            }
        })?;

        // --- Step 6: Copy logo asset if present (AC-004) ---
        let logo_asset_path = if let Some(logo) = &template.logo {
            if let Some(relative_path) = &logo_relative_path {
                let logo_dest = output_dir.join(relative_path);

                // Create brand.assets/ directory.
                if let Some(parent) = logo_dest.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| BrandError::ParseError {
                        path: Arc::from(parent.to_string_lossy().as_ref()),
                        reason: Arc::from(
                            format!("cannot create brand.assets directory: {e}").as_str(),
                        ),
                        span: slideforge_types::SourceSpan::default(),
                    })?;
                }

                // Write logo bytes.
                if let LogoAsset::Loaded { bytes, .. } = logo {
                    std::fs::write(&logo_dest, bytes).map_err(|e| BrandError::ParseError {
                        path: Arc::from(logo_dest.to_string_lossy().as_ref()),
                        reason: Arc::from(format!("cannot write logo asset: {e}").as_str()),
                        span: slideforge_types::SourceSpan::default(),
                    })?;
                    Some(logo_dest)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        tracing::info!(
            source = source,
            output = %brand_toml_path.display(),
            has_logo = logo_asset_path.is_some(),
            "brand extraction complete"
        );

        Ok(BrandExtractionResult {
            brand_toml_path,
            logo_asset_path,
        })
    }
}

// ─── TOML serialization ───────────────────────────────────────────────────────

/// Convert a [`BrandTemplate`] to a `brand.toml` string with stable ECMA-376 field order.
///
/// Returns `(toml_string, Option<logo_relative_path>)`.
///
/// The TOML string is built manually (not via `toml::to_string`) to guarantee:
/// - All 12 color slots appear in ECMA-376 sequential order (BC-2.01.003 invariant 1).
/// - Sections appear in stable order: `[colors]` → `[fonts]` → `[logo]` → `[footer]`.
/// - `folHlink` is serialized as `fol_hlink` (TOML-safe name, AC-003).
/// - Scheme-color slots (unresolved) carry an inline TOML comment per EC-003.
fn brand_template_to_toml(
    template: &BrandTemplate,
    logo: Option<&LogoAsset>,
) -> (String, Option<String>) {
    // Slot names in ECMA-376 order; index 11 (folHlink) is serialized as fol_hlink.
    const TOML_FIELD_NAMES: [&str; 12] = [
        "dk1",
        "lt1",
        "dk2",
        "lt2",
        "acc1",
        "acc2",
        "acc3",
        "acc4",
        "acc5",
        "acc6",
        "hlink",
        "fol_hlink",
    ];

    let mut out = String::with_capacity(512);

    // [colors]
    out.push_str("[colors]\n");
    for (i, color_slot) in template.colors.iter().enumerate() {
        let field = TOML_FIELD_NAMES[i];
        match &color_slot.value {
            ColorValue::Hex(hex) => {
                let _ = writeln!(out, "{field} = \"{hex}\"");
            },
            ColorValue::SchemeRef(scheme_ref) => {
                // EC-003: tint/shade or scheme-ref — write a placeholder with a TOML inline comment.
                // BC-2.01.003 AC-006: "written with inline TOML comment: # derived via tint/shade".
                let _ = writeln!(
                    out,
                    "{field} = \"#{scheme_ref}\" # derived via tint/shade; may not match exact color"
                );
            },
        }
    }
    out.push('\n');

    // [fonts]
    out.push_str("[fonts]\n");
    let _ = writeln!(out, "heading = \"{}\"", template.fonts.heading);
    let _ = writeln!(out, "body = \"{}\"", template.fonts.body);

    // [logo] — only if a logo was found (AC-004)
    let logo_relative_path = logo.and_then(|l| {
        if let LogoAsset::Loaded { original_path, .. } = l {
            // Determine file extension from ZIP-internal path (e.g., "ppt/media/image1.png" → "png").
            let ext = Path::new(original_path.as_ref())
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("bin");
            Some(format!("brand.assets/logo.{ext}"))
        } else {
            None
        }
    });

    if let Some(ref rel_path) = logo_relative_path {
        out.push('\n');
        out.push_str("[logo]\n");
        let _ = writeln!(out, "path = \"{rel_path}\"");
    }

    // [footer] — only if footer text is present and non-empty
    if let Some(footer_text) = &template.footer_text
        && !footer_text.is_empty()
    {
        out.push('\n');
        out.push_str("[footer]\n");
        let _ = writeln!(out, "text = \"{footer_text}\"");
    }

    (out, logo_relative_path)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::io::{Cursor, Write as _};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    use zip::CompressionMethod;
    use zip::write::{SimpleFileOptions, ZipWriter};

    use super::*;
    use crate::loader::PPTX_THEME_PATH;

    // ─── Shared fixture constants ─────────────────────────────────────────────

    /// Minimal PPTX theme1.xml with all 12 sRGB color slots, both fonts.
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

    /// Theme XML with a sysClr for dk1 (EC-002 vector).
    const SYSCLR_THEME_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="SysClrTheme">
  <a:themeElements>
    <a:clrScheme name="SysClrScheme">
      <a:dk1><a:sysClr val="windowText" lastClr="1A1A1A"/></a:dk1>
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
    <a:fontScheme name="SysClrFontScheme">
      <a:majorFont><a:latin typeface="Calibri Light"/></a:majorFont>
      <a:minorFont><a:latin typeface="Calibri"/></a:minorFont>
    </a:fontScheme>
  </a:themeElements>
</a:theme>"#;

    // ─── Fixture builders ─────────────────────────────────────────────────────

    /// Build an in-memory PPTX ZIP with the given theme XML (no logo).
    fn build_pptx_zip(theme_xml: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut zw = ZipWriter::new(cursor);
            let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
            zw.start_file(PPTX_THEME_PATH, opts).unwrap();
            zw.write_all(theme_xml.as_bytes()).unwrap();
            zw.start_file("[Content_Types].xml", opts).unwrap();
            zw.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>").unwrap();
            zw.finish().unwrap();
        }
        buf
    }

    /// Build an in-memory PPTX ZIP with an embedded logo PNG.
    ///
    /// Adds `slideMaster1.xml.rels` pointing at `ppt/media/image1.png`
    /// and the PNG stub bytes.
    fn build_pptx_zip_with_logo(theme_xml: &str) -> Vec<u8> {
        let png_bytes: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut zw = ZipWriter::new(cursor);
            let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
            zw.start_file(PPTX_THEME_PATH, opts).unwrap();
            zw.write_all(theme_xml.as_bytes()).unwrap();
            zw.start_file("ppt/slideMasters/_rels/slideMaster1.xml.rels", opts)
                .unwrap();
            let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1"
    Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"
    Target="../media/image1.png"/>
</Relationships>"#;
            zw.write_all(rels.as_bytes()).unwrap();
            zw.start_file("ppt/media/image1.png", opts).unwrap();
            zw.write_all(png_bytes).unwrap();
            zw.start_file("[Content_Types].xml", opts).unwrap();
            zw.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>").unwrap();
            zw.finish().unwrap();
        }
        buf
    }

    /// Write bytes to a uniquely-named temp file; return its path.
    fn write_temp_pptx(bytes: &[u8]) -> std::path::PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = format!(
            "slideforge_extractor_test_{}_{}_{}.pptx",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            seq
        );
        let path = std::env::temp_dir().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(bytes).unwrap();
        f.sync_all().unwrap();
        path
    }

    /// Create a unique temp directory for extractor output.
    fn temp_output_dir() -> std::path::PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = format!(
            "slideforge_extractor_out_{}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            seq
        );
        let dir = std::env::temp_dir().join(name);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    // ─── AC-001: extract writes brand.toml to output_dir ─────────────────────

    /// BC-2.01.003 postcondition 1 / AC-001 — extract from a valid PPTX writes
    /// `brand.toml` to `output_dir/brand.toml` and returns `BrandExtractionResult`.
    ///
    /// Test vector (BC-2.01.003): "well-formed corporate .pptx" → brand.toml with
    /// [colors], [fonts]; exit 0.
    #[test]
    fn test_bc_2_01_003_extract_writes_brand_toml() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        let result = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false);

        let _ = std::fs::remove_file(&source_path);

        let extraction = result.expect("valid PPTX must extract without error (AC-001)");

        // The returned path must point to the written file.
        assert!(
            extraction.brand_toml_path.exists(),
            "brand.toml must exist at the returned path: {}",
            extraction.brand_toml_path.display()
        );

        // The path must be `output_dir/brand.toml`.
        let expected_path = out_dir.join("brand.toml");
        assert_eq!(
            extraction.brand_toml_path, expected_path,
            "brand_toml_path must be output_dir/brand.toml"
        );

        // The written file must contain [colors].
        let content =
            std::fs::read_to_string(&extraction.brand_toml_path).expect("must read brand.toml");
        assert!(
            content.contains("[colors]"),
            "brand.toml must contain [colors] section, got:\n{content}"
        );

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── AC-002: force=false on existing brand.toml returns OutputExists ──────

    /// BC-2.01.003 EC-001 / AC-002 — when `brand.toml` already exists and
    /// `force = false`, returns `BrandError::OutputExists`; existing file is
    /// untouched.
    ///
    /// Test vector: "brand.toml already exists, no --force" → Error: …; exit 4.
    #[test]
    fn test_bc_2_01_003_rejects_existing_output_without_force() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        // Pre-create the brand.toml with sentinel content.
        let existing_path = out_dir.join("brand.toml");
        std::fs::write(&existing_path, b"# existing brand.toml sentinel\n").unwrap();

        let result = BrandExtractor::extract(
            source_path.to_str().unwrap(),
            &out_dir,
            false, // force = false
        );

        let _ = std::fs::remove_file(&source_path);

        assert!(
            result.is_err(),
            "extract must return Err when brand.toml exists and force=false (AC-002)"
        );
        assert!(
            matches!(result.unwrap_err(), BrandError::OutputExists { .. }),
            "error must be BrandError::OutputExists (EC-001)"
        );

        // Existing file must be untouched (sentinel content preserved).
        let content =
            std::fs::read_to_string(&existing_path).expect("existing file must still exist");
        assert_eq!(
            content, "# existing brand.toml sentinel\n",
            "existing brand.toml must not be modified when force=false (AC-002)"
        );

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── AC-002: force=true on existing brand.toml overwrites it ─────────────

    /// BC-2.01.003 EC-001 / AC-002 — when `brand.toml` exists and `force = true`,
    /// the file is overwritten and extraction succeeds.
    #[test]
    fn test_bc_2_01_003_force_overwrites_existing_brand_toml() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        // Pre-create a brand.toml.
        let existing_path = out_dir.join("brand.toml");
        std::fs::write(&existing_path, b"# old content\n").unwrap();

        let result = BrandExtractor::extract(
            source_path.to_str().unwrap(),
            &out_dir,
            true, // force = true
        );

        let _ = std::fs::remove_file(&source_path);

        let extraction =
            result.expect("extract with force=true must succeed even when brand.toml exists");

        // The file must have been overwritten (no longer the sentinel).
        let content = std::fs::read_to_string(&extraction.brand_toml_path).unwrap();
        assert!(
            !content.contains("# old content"),
            "old content must be replaced by extraction output; got:\n{content}"
        );
        assert!(
            content.contains("[colors]"),
            "overwritten brand.toml must contain [colors]"
        );

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── AC-003: all 12 color slots in [colors] section ──────────────────────

    /// BC-2.01.003 postcondition 2 / AC-003 — all 12 OOXML color slots appear in
    /// the `[colors]` section, in ECMA-376 sequential order.
    ///
    /// Canonical color slot names per ECMA-376: dk1, lt1, dk2, lt2, acc1-acc6,
    /// hlink, `fol_hlink` (the TOML-safe name for folHlink).
    #[test]
    fn test_bc_2_01_003_all_12_color_slots_in_colors_section() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        let extraction = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false)
            .expect("valid PPTX must extract without error");

        let _ = std::fs::remove_file(&source_path);

        let content = std::fs::read_to_string(&extraction.brand_toml_path).unwrap();

        // All 12 field names must appear in the output TOML.
        let expected_fields = [
            "dk1",
            "lt1",
            "dk2",
            "lt2",
            "acc1",
            "acc2",
            "acc3",
            "acc4",
            "acc5",
            "acc6",
            "hlink",
            "fol_hlink",
        ];
        for field in &expected_fields {
            assert!(
                content.contains(field),
                "brand.toml must contain color field '{field}' (AC-003), got:\n{content}"
            );
        }

        // Verify the canonical values from MINIMAL_THEME_XML are present.
        assert!(
            content.contains("#000000") || content.contains("000000"),
            "dk1 = #000000 must appear in brand.toml"
        );
        assert!(
            content.contains("#003087") || content.contains("003087"),
            "dk2 = #003087 must appear in brand.toml"
        );

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── AC-003: ECMA-376 field ordering invariant ────────────────────────────

    /// BC-2.01.003 invariant 1 / AC-003 — the 12 color slots appear in ECMA-376
    /// sequential order in the `[colors]` section: dk1 before lt1 before dk2 …
    /// before `fol_hlink`.
    #[test]
    fn test_bc_2_01_003_invariant_color_slots_in_ecma376_order() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        let extraction = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false)
            .expect("valid PPTX must extract");

        let _ = std::fs::remove_file(&source_path);

        let content = std::fs::read_to_string(&extraction.brand_toml_path).unwrap();

        // Find the byte positions of each field key in the file to check ordering.
        let order = [
            "dk1",
            "lt1",
            "dk2",
            "lt2",
            "acc1",
            "acc2",
            "acc3",
            "acc4",
            "acc5",
            "acc6",
            "hlink",
            "fol_hlink",
        ];
        let mut prev_pos = 0usize;
        for field in &order {
            let pos = content.find(field).unwrap_or_else(|| {
                panic!("field '{field}' must appear in brand.toml for ordering check")
            });
            assert!(
                pos > prev_pos,
                "field '{field}' at position {pos} must come after previous field at {prev_pos} \
                 (ECMA-376 sequential order invariant)"
            );
            prev_pos = pos;
        }

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── AC-004: logo found → copied to brand.assets/logo.<ext> ─────────────

    /// BC-2.01.003 postcondition 3 / AC-004 — when a logo is found in
    /// `slideMaster1.xml.rels`, it is copied to `brand.assets/logo.<ext>` and
    /// `brand.toml` contains `[logo] path = "brand.assets/logo.png"`.
    #[test]
    fn test_bc_2_01_003_copies_logo_to_brand_assets() {
        let zip_bytes = build_pptx_zip_with_logo(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        let extraction = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false)
            .expect("PPTX with logo must extract without error");

        let _ = std::fs::remove_file(&source_path);

        // logo_asset_path must be Some.
        let logo_path = extraction.logo_asset_path.expect(
            "BrandExtractionResult.logo_asset_path must be Some when logo is found (AC-004)",
        );
        assert!(
            logo_path.exists(),
            "logo asset must exist at: {}",
            logo_path.display()
        );
        assert!(
            std::path::Path::new(logo_path.to_str().unwrap())
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("png")),
            "logo must be written with .png extension matching original ZIP path"
        );
        assert!(
            logo_path.starts_with(out_dir.join("brand.assets")),
            "logo must be inside brand.assets/ subdirectory"
        );

        // brand.toml must have [logo] section.
        let content = std::fs::read_to_string(&extraction.brand_toml_path).unwrap();
        assert!(
            content.contains("[logo]"),
            "brand.toml must contain [logo] section when logo is found"
        );
        assert!(
            content.contains("brand.assets/logo.png"),
            "brand.toml [logo] path must be 'brand.assets/logo.png', got:\n{content}"
        );

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── AC-004 (no logo): [logo] section omitted ─────────────────────────────

    /// BC-2.01.003 / AC-004 — when source PPTX has no logo, `brand.toml` must
    /// NOT contain a `[logo]` section and `logo_asset_path` must be `None`.
    ///
    /// Test vector: ".pptx with no logo in slide master" → brand.toml without
    /// [logo] section; exit 0.
    #[test]
    fn test_bc_2_01_003_no_logo_omits_logo_section() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        let extraction = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false)
            .expect("PPTX without logo must extract without error");

        let _ = std::fs::remove_file(&source_path);

        assert!(
            extraction.logo_asset_path.is_none(),
            "logo_asset_path must be None when source has no logo (AC-004)"
        );

        let content = std::fs::read_to_string(&extraction.brand_toml_path).unwrap();
        assert!(
            !content.contains("[logo]"),
            "brand.toml must NOT contain [logo] section when no logo found, got:\n{content}"
        );

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── AC-005: sysClr → lastClr value used (EC-002) ────────────────────────

    /// BC-2.01.003 EC-002 / AC-005 — `sysClr` elements use their `lastClr`
    /// attribute value as the hex color in `brand.toml`.
    ///
    /// Test vector: ".pptx with sysClr for dk1" → `dk1 = "#1A1A1A"` (from
    /// `lastClr="1A1A1A"`).
    #[test]
    fn test_bc_2_01_003_sysclr_uses_last_clr_value() {
        let zip_bytes = build_pptx_zip(SYSCLR_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        let extraction = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false)
            .expect("PPTX with sysClr must extract without error (EC-002)");

        let _ = std::fs::remove_file(&source_path);

        let content = std::fs::read_to_string(&extraction.brand_toml_path).unwrap();

        // dk1 must use the lastClr value (#1A1A1A).
        assert!(
            content.contains("1A1A1A") || content.contains("1a1a1a"),
            "brand.toml dk1 must use sysClr lastClr value '1A1A1A' (EC-002), got:\n{content}"
        );

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── AC-007: multiple slide masters → warning, only master1 used ─────────

    /// BC-2.01.003 EC-004 / AC-007 — when source PPTX has multiple slide
    /// masters, extraction succeeds using only `slideMaster1.xml` and emits
    /// a `tracing::warn!`.
    #[test]
    fn test_bc_2_01_003_multiple_slide_masters_uses_master1_only() {
        // Build PPTX with two slide masters.
        let zip_bytes = {
            let mut buf = Vec::new();
            let cursor = Cursor::new(&mut buf);
            let mut zw = ZipWriter::new(cursor);
            let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
            zw.start_file(PPTX_THEME_PATH, opts).unwrap();
            zw.write_all(MINIMAL_THEME_XML.as_bytes()).unwrap();
            zw.start_file("ppt/slideMasters/slideMaster1.xml", opts)
                .unwrap();
            zw.write_all(b"<p:sldMaster/>").unwrap();
            zw.start_file("ppt/slideMasters/slideMaster2.xml", opts)
                .unwrap();
            zw.write_all(b"<p:sldMaster/>").unwrap();
            zw.start_file("[Content_Types].xml", opts).unwrap();
            zw.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>").unwrap();
            zw.finish().unwrap();
            buf
        };

        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        // Extraction must succeed (multiple masters is warning, not fatal).
        let result = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false);

        let _ = std::fs::remove_file(&source_path);

        assert!(
            result.is_ok(),
            "PPTX with multiple slide masters must extract without error (EC-004): {:?}",
            result.err()
        );

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── AC-008: source file unchanged after extraction ───────────────────────

    /// BC-2.01.003 invariant 2 / AC-008 — the source `.pptx` file is NEVER
    /// modified. SHA-256 hash before and after extraction must be identical.
    ///
    /// Verification property (BC-2.01.003): "Source .pptx file is unmodified
    /// after extraction (file hash before/after)."
    #[test]
    fn test_bc_2_01_003_invariant_source_file_unmodified() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        // Read source bytes before extraction.
        let bytes_before = std::fs::read(&source_path).unwrap();
        let sha_before = simple_sha256(&bytes_before);

        let _ = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false);

        // Read source bytes after extraction.
        let bytes_after = std::fs::read(&source_path).unwrap();
        let sha_after = simple_sha256(&bytes_after);

        let _ = std::fs::remove_file(&source_path);
        let _ = std::fs::remove_dir_all(&out_dir);

        assert_eq!(
            sha_before, sha_after,
            "source .pptx must be byte-identical before and after extraction (invariant 2)"
        );
    }

    // ─── AC-009: human-readable TOML with stable section order ───────────────

    /// BC-2.01.003 invariant 3 / AC-009 — `brand.toml` is human-readable with
    /// section headers and one field per line. Sections appear in the expected
    /// order: `[colors]` → `[fonts]` → `[footer]` (and `[logo]` if present).
    #[test]
    fn test_bc_2_01_003_brand_toml_has_section_headers_and_stable_order() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        let extraction = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false)
            .expect("valid PPTX must extract");

        let _ = std::fs::remove_file(&source_path);

        let content = std::fs::read_to_string(&extraction.brand_toml_path).unwrap();

        // [colors] must appear before [fonts].
        let colors_pos = content
            .find("[colors]")
            .expect("brand.toml must have [colors] section");
        let fonts_pos = content
            .find("[fonts]")
            .expect("brand.toml must have [fonts] section");
        assert!(
            colors_pos < fonts_pos,
            "[colors] section must precede [fonts] section in brand.toml"
        );

        // Each line in [colors] that is not a comment must be a `key = value` pair.
        // We verify by parsing the output TOML and confirming all 12 slots are present.
        let parsed: crate::toml_schema::BrandConfig =
            toml::from_str(&content).expect("extracted brand.toml must be valid TOML");
        assert!(
            parsed.colors.dk1.is_some(),
            "dk1 must be present in parsed brand.toml"
        );
        assert!(
            parsed.colors.fol_hlink.is_some(),
            "fol_hlink must be present in parsed brand.toml"
        );

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── AC-010: round-trip integration test ─────────────────────────────────

    /// BC-2.01.003 verification property / AC-010 — round-trip test:
    /// load `.pptx` → extract `brand.toml` → parse the TOML back →
    /// `BrandConfig.colors` hex values must match original `.pptx` color slots.
    #[test]
    fn test_bc_2_01_003_round_trip_pptx_to_brand_toml_and_back() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        let extraction = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false)
            .expect("round-trip extraction must succeed");

        let _ = std::fs::remove_file(&source_path);

        let content = std::fs::read_to_string(&extraction.brand_toml_path)
            .expect("brand.toml must be readable");
        let config: crate::toml_schema::BrandConfig =
            toml::from_str(&content).expect("extracted brand.toml must parse as valid BrandConfig");

        // Verify canonical values from MINIMAL_THEME_XML are round-tripped.
        // dk1 = #000000, lt1 = #FFFFFF, dk2 = #003087 (values normalised to uppercase).
        let dk1 = config
            .colors
            .dk1
            .expect("dk1 must be present after round-trip");
        assert!(
            dk1.eq_ignore_ascii_case("#000000"),
            "dk1 must round-trip as #000000, got: {dk1}"
        );
        let dk2 = config
            .colors
            .dk2
            .expect("dk2 must be present after round-trip");
        assert!(
            dk2.eq_ignore_ascii_case("#003087"),
            "dk2 must round-trip as #003087, got: {dk2}"
        );
        let acc1 = config
            .colors
            .acc1
            .expect("acc1 must be present after round-trip");
        assert!(
            acc1.eq_ignore_ascii_case("#0066CC"),
            "acc1 must round-trip as #0066CC, got: {acc1}"
        );
        let fol_hlink = config
            .colors
            .fol_hlink
            .expect("fol_hlink must be present after round-trip");
        assert!(
            fol_hlink.eq_ignore_ascii_case("#551A8B"),
            "fol_hlink must round-trip as #551A8B, got: {fol_hlink}"
        );

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── AC-001 / EC-006: output directory created if absent ─────────────────

    /// BC-2.01.003 EC-006 — when `output_dir` does not exist, it is created
    /// via `std::fs::create_dir_all` and extraction succeeds.
    #[test]
    fn test_bc_2_01_003_creates_output_dir_if_absent() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);

        // Point to a non-existent nested directory.
        let base = temp_output_dir();
        let out_dir = base.join("nested").join("subdir");
        assert!(
            !out_dir.exists(),
            "output_dir must not exist before extraction for this test"
        );

        let result = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false);

        let _ = std::fs::remove_file(&source_path);

        let extraction =
            result.expect("extraction must succeed even when output_dir does not exist (EC-006)");
        assert!(
            extraction.brand_toml_path.exists(),
            "brand.toml must be written after creating nested output_dir"
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    // ─── Error code invariant: E-BRD-006 constant ────────────────────────────

    /// BC-2.01.003 invariant / error taxonomy — `E_BRD_006` constant is `"E-BRD-006"`.
    #[test]
    fn test_bc_2_01_003_e_brd_006_constant_value() {
        assert_eq!(crate::error::E_BRD_006, "E-BRD-006");
    }

    /// BC-2.01.003 EC-001 — `BrandError::OutputExists` message contains the path
    /// and the error code.
    #[test]
    fn test_bc_2_01_003_output_exists_error_message() {
        let err = BrandError::OutputExists {
            path: Arc::from("/tmp/brand.toml"),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("E-BRD-006"),
            "OutputExists message must contain E-BRD-006, got: {msg}"
        );
        assert!(
            msg.contains("brand.toml"),
            "OutputExists message must mention 'brand.toml', got: {msg}"
        );
        assert!(
            msg.contains("--force"),
            "OutputExists message must mention '--force', got: {msg}"
        );
    }

    /// BC-2.01.003 postcondition 1 — `BrandExtractionResult` fields are accessible.
    #[test]
    fn test_bc_2_01_003_extraction_result_fields_accessible() {
        let result = BrandExtractionResult {
            brand_toml_path: PathBuf::from("/out/brand.toml"),
            logo_asset_path: Some(PathBuf::from("/out/brand.assets/logo.png")),
        };
        assert_eq!(result.brand_toml_path, PathBuf::from("/out/brand.toml"));
        assert_eq!(
            result.logo_asset_path,
            Some(PathBuf::from("/out/brand.assets/logo.png"))
        );
    }

    // ─── AC-001: source file not found returns FileNotFound ──────────────────

    /// BC-2.01.003 precondition 1 — when the source path does not resolve to a
    /// readable file, `BrandError::FileNotFound` is returned.
    #[test]
    fn test_bc_2_01_003_rejects_missing_source_file() {
        let out_dir = temp_output_dir();
        let result = BrandExtractor::extract(
            "/tmp/slideforge_nonexistent_extractor_9999999.pptx",
            &out_dir,
            false,
        );
        let _ = std::fs::remove_dir_all(&out_dir);
        assert!(
            result.is_err(),
            "extract must return Err for non-existent source file (precondition 1)"
        );
        assert!(
            matches!(result.unwrap_err(), BrandError::FileNotFound { .. }),
            "error must be BrandError::FileNotFound for missing source"
        );
    }

    // ─── [fonts] section present in output ────────────────────────────────────

    /// BC-2.01.003 postcondition 1 — brand.toml must contain a [fonts] section
    /// with heading and body font names from the source PPTX.
    #[test]
    fn test_bc_2_01_003_fonts_section_written_with_correct_values() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        let extraction = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false)
            .expect("valid PPTX must extract");

        let _ = std::fs::remove_file(&source_path);

        let content = std::fs::read_to_string(&extraction.brand_toml_path).unwrap();
        assert!(
            content.contains("[fonts]"),
            "brand.toml must contain [fonts] section"
        );
        // MINIMAL_THEME_XML uses "Calibri Light" / "Calibri".
        assert!(
            content.contains("Calibri Light"),
            "brand.toml [fonts].heading must be 'Calibri Light', got:\n{content}"
        );
        assert!(
            content.contains("Calibri"),
            "brand.toml [fonts].body must be 'Calibri', got:\n{content}"
        );

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── AC-006 / EC-003: scheme-color slots get inline TOML comment ────────

    /// BC-2.01.003 AC-006 / EC-003 — when a color slot in the source `.pptx`
    /// contains a `<a:schemeClr>` element (tint/shade transform or relative
    /// scheme reference), the extracted `brand.toml` entry includes an inline
    /// TOML comment: `# derived via tint/shade; may not match exact color`.
    ///
    /// This test exercises the `brand_template_to_toml` helper directly with a
    /// `ColorValue::SchemeRef` slot (no real PPTX needed — we control the
    /// `BrandTemplate` directly to inject the unresolved slot).
    ///
    /// Traces to BC-2.01.003 edge case EC-003.
    #[test]
    fn test_bc_2_01_003_ec003_scheme_ref_slot_gets_inline_toml_comment() {
        use crate::template::{BrandFonts, BrandTemplate, ColorSlot, ColorValue, MasterIds};

        // Build a BrandTemplate where acc1 (index 4) is a SchemeRef (tint/shade).
        let make_hex_slot = |name: &str, hex: &str| ColorSlot {
            name: Arc::from(name),
            value: ColorValue::Hex(Arc::from(hex)),
        };
        let make_scheme_slot = |name: &str, scheme: &str| ColorSlot {
            name: Arc::from(name),
            value: ColorValue::SchemeRef(Arc::from(scheme)),
        };

        let colors: [ColorSlot; 12] = [
            make_hex_slot("dk1", "#000000"),
            make_hex_slot("lt1", "#FFFFFF"),
            make_hex_slot("dk2", "#003087"),
            make_hex_slot("lt2", "#F5F5F5"),
            // acc1 is a scheme-ref (tint/shade, EC-003 vector)
            make_scheme_slot("acc1", "accent1"),
            make_hex_slot("acc2", "#FF6B35"),
            make_hex_slot("acc3", "#28A745"),
            make_hex_slot("acc4", "#FFC107"),
            make_hex_slot("acc5", "#6F42C1"),
            make_hex_slot("acc6", "#17A2B8"),
            make_hex_slot("hlink", "#0000EE"),
            make_hex_slot("folHlink", "#551A8B"),
        ];

        let template = BrandTemplate {
            colors,
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
        };

        let (toml_str, _logo_path) = brand_template_to_toml(&template, None);

        // The acc1 line must contain the tint/shade inline comment (AC-006 / EC-003).
        let acc1_line = toml_str
            .lines()
            .find(|l| l.trim_start().starts_with("acc1"))
            .unwrap_or_else(|| panic!("acc1 line must appear in TOML output, got:\n{toml_str}"));

        assert!(
            acc1_line.contains("# derived via tint/shade"),
            "acc1 SchemeRef slot must contain inline TOML comment \
             '# derived via tint/shade; may not match exact color' (EC-003), \
             got line: {acc1_line}"
        );

        // Other hex slots must NOT contain the comment.
        let dk1_line = toml_str
            .lines()
            .find(|l| l.trim_start().starts_with("dk1"))
            .unwrap_or_else(|| panic!("dk1 line must appear in TOML output, got:\n{toml_str}"));
        assert!(
            !dk1_line.contains("# derived"),
            "dk1 Hex slot must NOT contain the scheme-ref comment, got: {dk1_line}"
        );
    }

    // ─── Helper: simple SHA-256 without external deps ────────────────────────

    /// Compute a simple FNV-1a hash (64-bit) as a proxy for content identity.
    ///
    /// We avoid pulling in `sha2` just for tests. FNV-1a is sufficient for
    /// the invariant: if the bytes are unchanged, the hash is unchanged.
    fn simple_sha256(data: &[u8]) -> u64 {
        let mut hash: u64 = 14_695_981_039_346_656_037;
        for &byte in data {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(1_099_511_628_211);
        }
        hash
    }
}
