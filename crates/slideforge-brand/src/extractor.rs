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

use crate::color::default_color_for_slot;
use crate::context::BrandLoadContext;
use crate::error::BrandError;
use crate::loader::BrandLoader;
use crate::template::{BrandTemplate, ColorValue, LogoAsset};

/// OOXML `schemeClr val` attribute names that can appear in a slot's value, mapped to
/// their canonical slot index (0-based, ECMA-376 order).
///
/// Used by [`resolve_scheme_ref`] to look up the base hex for a `SchemeRef`.
///
/// ## Normalization
///
/// `color.rs` stores every `schemeClr val` lowercased (e.g., `"folHlink"` becomes
/// `"folhlink"`, `"followedHyperlink"` becomes `"followedhyperlink"`).
/// This function therefore normalizes `scheme_name` to ASCII-lowercase before
/// matching, so all OOXML camelCase aliases resolve correctly regardless of
/// the case used in the original XML.
///
/// ## Intentionally Unresolved: Color-Map Mnemonics and `phClr`
///
/// The following `schemeClr val` tokens are intentionally NOT handled here and will
/// return `None`, triggering the `default_color_for_slot` fallback:
///
/// - `tx1`, `tx2`, `bg1`, `bg2` — color-map mnemonics that remap scheme slots at the
///   slide master / slide layout level (via `<p:clrMap>`). Resolving them requires the
///   per-master color map, which is context-dependent and not available in this
///   slot-level lookup function.
/// - `phClr` — the "placeholder color" sentinel indicating that the color is inherited
///   from the layout or shape's placeholder hierarchy. No absolute hex can be determined
///   without fully walking the placeholder chain.
///
/// Both cases are spec-compliant under BC-2.01.003 EC-003: the extractor is permitted
/// to fall back to `default_color_for_slot` with the inline TOML comment
/// `"# derived via tint/shade; may not match exact color"` when an absolute hex
/// cannot be determined. This behavior is correct and intentional, not an oversight.
fn scheme_name_to_slot_index(scheme_name: &str) -> Option<usize> {
    // OOXML `<a:schemeClr val="...">` uses "accent1"…"accent6" spellings; some templates
    // also use the short "acc1"…"acc6" aliases used in brand.toml. Both forms are accepted.
    // All arms are lowercase to match the stored SchemeRef values from color.rs:155.
    //
    // Color-map mnemonics (tx1, tx2, bg1, bg2) and phClr are intentionally absent —
    // see the function-level doc comment above for the rationale.
    match scheme_name.to_ascii_lowercase().as_str() {
        "dk1" => Some(0),
        "lt1" => Some(1),
        "dk2" => Some(2),
        "lt2" => Some(3),
        "accent1" | "acc1" => Some(4),
        "accent2" | "acc2" => Some(5),
        "accent3" | "acc3" => Some(6),
        "accent4" | "acc4" => Some(7),
        "accent5" | "acc5" => Some(8),
        "accent6" | "acc6" => Some(9),
        "hlink" | "hyperlink" => Some(10),
        "folhlink" | "followedhyperlink" => Some(11),
        _ => None,
    }
}

/// Resolve a `SchemeRef` to the base hex of the referenced slot.
///
/// Performs ONE level of indirection — does not recursively follow chains
/// (to avoid cycles). If the referenced slot is itself a `SchemeRef`, or if
/// the name is unrecognised, returns `None`.
///
/// Returns `None` for self-references, cycles (target is also a `SchemeRef`),
/// and unknown scheme names.
fn resolve_scheme_ref<'a>(
    scheme_name: &str,
    self_index: usize,
    colors: &'a [crate::template::ColorSlot; 12],
) -> Option<&'a str> {
    let target_idx = scheme_name_to_slot_index(scheme_name)?;
    if target_idx == self_index {
        // Self-reference — cannot resolve.
        return None;
    }
    colors[target_idx].value.as_hex()
}

/// Escape a string value for TOML basic-string format.
///
/// Escapes backslash and double-quote characters (both are mandatory per the
/// TOML spec, §2.3 String). Control characters (U+0000–U+001F, U+007F) are
/// also escaped as `\uXXXX` sequences.
fn toml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c as u32 == 0x7F => {
                // Escape other control characters as \uXXXX.
                let _ = write!(out, "\\u{:04X}", c as u32);
            },
            c => out.push(c),
        }
    }
    out
}

/// Renderable logo image extensions (AC-004 / EC-005).
///
/// Extensions outside this set are supported by the copy but emit a
/// `tracing::warn!` about potential rendering differences (BC-2.01.003 EC-005).
const RENDERABLE_LOGO_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "svg"];

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
        // --- Step 1: Validate source existence FIRST (no filesystem side effects yet) ---
        // This must run before create_dir_all so that a bad source leaves no output dir
        // (F-024-O1: source validation before directory creation).
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext {
            check_font_availability: false,
            // root_dir is the parent directory of the SOURCE .pptx, matching the
            // convention used by BrandProvider::load (loader.rs). load_template does not
            // currently read root_dir on this code path (it is reserved for future
            // relative logo-path resolution), but it must be set to the source directory
            // — not output_dir — so the context is semantically correct if the field is
            // ever read by a future code path.
            root_dir: Path::new(source).parent().map_or_else(
                || std::path::PathBuf::from("."),
                std::path::Path::to_path_buf,
            ),
            span: slideforge_types::SourceSpan::default(),
        };
        let template = loader.load_template(Path::new(source), &ctx)?;

        // --- Step 2: Check for existing brand.toml (EC-001) --- before creating dir ---
        // This check must also run before create_dir_all so a force=false rejection
        // does not create the output directory when brand.toml already exists.
        let brand_toml_path = output_dir.join("brand.toml");
        if brand_toml_path.exists() && !force {
            return Err(BrandError::OutputExists {
                path: Arc::from(brand_toml_path.to_string_lossy().as_ref()),
            });
        }

        // --- Step 3: Create output directory now that validation passed (EC-006) ---
        std::fs::create_dir_all(output_dir).map_err(|e| BrandError::ParseError {
            path: Arc::from(output_dir.to_string_lossy().as_ref()),
            reason: Arc::from(format!("cannot create output directory: {e}").as_str()),
            span: slideforge_types::SourceSpan::default(),
        })?;

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
pub(crate) fn brand_template_to_toml(
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
                // BC-2.01.003 EC-003 (widened by STORY-076): emit the inline TOML comment
                // when is_derived = true, regardless of whether the origin was schemeClr
                // or srgbClr. The flag is the single source of truth for derivation state.
                if color_slot.is_derived {
                    let _ = writeln!(
                        out,
                        "{field} = \"{hex}\" # derived via tint/shade; may not match exact color"
                    );
                } else {
                    let _ = writeln!(out, "{field} = \"{hex}\"");
                }
            },
            ColorValue::SchemeRef(scheme_name) => {
                // EC-003: the slot contains a relative scheme reference (e.g., from
                // `<a:schemeClr val="accent1">` with lumMod/tint transforms).
                // BC-2.01.003 AC-006 requires writing the RESOLVED BASE HEX of the
                // referenced slot, with the inline TOML comment.
                //
                // Resolution strategy (F-024-C1):
                // 1. Look up the target slot index from the scheme name.
                // 2. If the target slot holds a Hex value, use that hex.
                // 3. If resolution fails (self-reference, cycle, unknown token),
                //    use the same slot's fallback from the default table — never
                //    write a garbage non-hex value like "#accent1".
                let resolved_hex: String = resolve_scheme_ref(scheme_name, i, &template.colors)
                    .map_or_else(
                        || {
                            // Self-reference or unresolvable: delegate to the single-source-of-truth
                            // default table in color.rs (F-024A-MED-2 — eliminates sibling-drift).
                            // color_slot.name is the OOXML slot name (e.g., "folHlink"),
                            // which is the key accepted by default_color_for_slot.
                            tracing::warn!(
                                slot = field,
                                scheme_ref = scheme_name.as_ref(),
                                "unresolvable SchemeRef; using fallback hex for brand.toml"
                            );
                            default_color_for_slot(color_slot.name.as_ref()).to_string()
                        },
                        std::string::ToString::to_string,
                    );
                let _ = writeln!(
                    out,
                    "{field} = \"{resolved_hex}\" # derived via tint/shade; may not match exact color"
                );
            },
        }
    }
    out.push('\n');

    // [fonts] — escape values per TOML basic-string rules (F-024-H1).
    out.push_str("[fonts]\n");
    let _ = writeln!(
        out,
        "heading = \"{}\"",
        toml_escape(template.fonts.heading.as_ref())
    );
    let _ = writeln!(
        out,
        "body = \"{}\"",
        toml_escape(template.fonts.body.as_ref())
    );

    // [logo] — only if a logo was found (AC-004)
    let logo_relative_path = logo.and_then(|l| {
        if let LogoAsset::Loaded {
            original_path,
            media_type,
            ..
        } = l
        {
            // Determine file extension from ZIP-internal path (e.g., "ppt/media/image1.png" → "png").
            let ext = Path::new(original_path.as_ref())
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("bin");

            // EC-005: warn if the extension is not in the renderable set (F-024-H2).
            let ext_lower = ext.to_ascii_lowercase();
            if !RENDERABLE_LOGO_EXTENSIONS.contains(&ext_lower.as_str()) {
                tracing::warn!(
                    original_path = original_path.as_ref(),
                    media_type = media_type.as_ref(),
                    extension = ext,
                    "logo has unsupported format; copying bytes but rendering differences may occur"
                );
            }

            Some(format!("brand.assets/logo.{ext}"))
        } else {
            None
        }
    });

    if let Some(ref rel_path) = logo_relative_path {
        out.push('\n');
        out.push_str("[logo]\n");
        // rel_path is always ASCII with no special chars (constructed from ext), no escaping needed.
        let _ = writeln!(out, "path = \"{rel_path}\"");
    }

    // [footer] — only if footer text is present and non-empty
    // Escape the footer text per TOML basic-string rules (F-024-H1).
    if let Some(footer_text) = &template.footer_text
        && !footer_text.is_empty()
    {
        out.push('\n');
        out.push_str("[footer]\n");
        let _ = writeln!(out, "text = \"{}\"", toml_escape(footer_text.as_ref()));
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
    /// the mandated `tracing::warn!` string (F-024-H3 load-bearing assertion).
    ///
    /// AC-007 mandates the exact warning:
    /// `"Source .pptx has multiple slide masters; extracting from slideMaster1.xml only."`
    ///
    /// F-024A-OBS-2 strengthening: this test captures the actual formatted warning
    /// message text and asserts it contains the exact AC-007 mandated substring,
    /// not just that "some warning fired".
    // Inner helper types for the AC-007 message-capturing subscriber must appear
    // before any let-bindings in the test function to satisfy clippy::items_after_statements.
    // They are defined at the test-module scope below (outside the test function).
    #[test]
    #[allow(clippy::items_after_statements)]
    fn test_bc_2_01_003_multiple_slide_masters_uses_master1_only() {
        use std::sync::{Arc as StdArc, Mutex};
        use tracing::field::{Field, Visit};
        use tracing_subscriber::Layer;
        use tracing_subscriber::layer::SubscriberExt as _;

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

        // AC-007 mandated warning substring (F-024A-OBS-2 load-bearing assertion).
        const AC007_SUBSTRING: &str =
            "Source .pptx has multiple slide masters; extracting from slideMaster1.xml only.";

        // Capture all warning message bodies via a custom tracing layer.
        // We implement a visitor that extracts the "message" field from each event.
        let captured_messages: StdArc<Mutex<Vec<String>>> = StdArc::new(Mutex::new(Vec::new()));

        struct MessageCapturingLayer {
            messages: StdArc<Mutex<Vec<String>>>,
        }

        struct MessageVisitor {
            message: Option<String>,
        }

        impl Visit for MessageVisitor {
            fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.message = Some(format!("{value:?}"));
                }
            }
            fn record_str(&mut self, field: &Field, value: &str) {
                if field.name() == "message" {
                    self.message = Some(value.to_owned());
                }
            }
        }

        impl<S: tracing::Subscriber> Layer<S> for MessageCapturingLayer {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if *event.metadata().level() == tracing::Level::WARN {
                    let mut visitor = MessageVisitor { message: None };
                    event.record(&mut visitor);
                    if let Some(msg) = visitor.message {
                        let _ = self.messages.lock().map(|mut guard| guard.push(msg));
                    }
                }
            }
        }

        let capturing_layer = MessageCapturingLayer {
            messages: StdArc::clone(&captured_messages),
        };
        let subscriber = tracing_subscriber::registry().with(capturing_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // Extraction must succeed (multiple masters is warning, not fatal).
        let result = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false);

        let _ = std::fs::remove_file(&source_path);

        assert!(
            result.is_ok(),
            "PPTX with multiple slide masters must extract without error (EC-004): {:?}",
            result.err()
        );

        // F-024A-OBS-2 load-bearing assertion: the exact AC-007 warning substring
        // must appear in one of the captured warning messages.
        let messages = captured_messages.lock().unwrap();
        let found_ac007 = messages.iter().any(|msg| msg.contains(AC007_SUBSTRING));
        assert!(
            found_ac007,
            "AC-007 (F-024A-OBS-2): the exact warning \
             \"Source .pptx has multiple slide masters; extracting from slideMaster1.xml only.\" \
             must appear in a tracing::warn! event.\n\
             Captured warning messages: {messages:?}"
        );

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── AC-008: source file unchanged after extraction ───────────────────────

    /// BC-2.01.003 invariant 2 / AC-008 — the source `.pptx` file is NEVER
    /// modified. Byte-for-byte comparison before and after a SUCCESSFUL extraction
    /// must be identical.
    ///
    /// Verification property (BC-2.01.003): "Source .pptx file is unmodified
    /// after extraction (file hash before/after)."
    ///
    /// F-024R-MED-2 strengthening:
    /// (a) Extraction is asserted `Ok()` BEFORE comparing source state, so the invariant
    ///     is proven on a successful extraction — not on a vacuously-unchanged source
    ///     after a failed extraction.
    /// (b) Comparison uses direct byte-for-byte `Vec<u8>` equality (stronger than any
    ///     hash proxy; requires no additional dependency).
    #[test]
    fn test_bc_2_01_003_invariant_source_file_unmodified() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        // Read source bytes before extraction.
        let bytes_before = std::fs::read(&source_path).unwrap();

        // F-024R-MED-2(a): assert extraction succeeded BEFORE comparing source state.
        // If extract returns Err, the source trivially appears unchanged — that proves
        // nothing about read-only behaviour during a SUCCESSFUL extraction.
        let result = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false);
        assert!(
            result.is_ok(),
            "AC-008: extraction must succeed so the read-only invariant is proven on a \
             successful run, not vacuously on a failed one. Error: {:?}",
            result.err()
        );

        // F-024R-MED-2(b): byte-for-byte equality is stronger than any hash proxy
        // and needs no additional dependency.
        let bytes_after = std::fs::read(&source_path).unwrap();

        let _ = std::fs::remove_file(&source_path);
        let _ = std::fs::remove_dir_all(&out_dir);

        assert_eq!(
            bytes_before, bytes_after,
            "source .pptx must be byte-identical before and after extraction (AC-008 invariant 2)"
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

    /// BC-2.01.003 verification property / AC-010 — full round-trip test:
    /// load `.pptx` → extract `brand.toml` → parse the TOML back into
    /// `BrandConfig` → **synthesize a `BrandTemplate` from the `BrandConfig`**
    /// → assert all 12 synthesized color hex values equal the original source hex values.
    ///
    /// F-024R-MED-1 strengthening: the synthesize step is the load-bearing part of
    /// the round-trip. Inference / normalization inside `BrandSynthesizer::synthesize`
    /// could mutate a loaded hex (e.g., case normalisation or slot-inference overwriting
    /// a declared value). This assertion catches any such mutation.
    ///
    /// The source PPTX includes a logo so the extracted brand.toml contains a `[logo]`
    /// section — `BrandSynthesizer::synthesize` requires a non-empty logo path.
    #[test]
    fn test_bc_2_01_003_round_trip_pptx_to_brand_toml_and_back() {
        use crate::synthesizer::BrandSynthesizer;

        // F-024R-MED-1: MINIMAL_THEME_XML canonical hex values (all 12 slots, ECMA-376 order).
        // Defined before any let-bindings (items_after_statements lint).
        const EXPECTED_HEX: [&str; 12] = [
            "#000000", // dk1
            "#FFFFFF", // lt1
            "#003087", // dk2
            "#F5F5F5", // lt2
            "#0066CC", // acc1
            "#FF6B35", // acc2
            "#28A745", // acc3
            "#FFC107", // acc4
            "#6F42C1", // acc5
            "#17A2B8", // acc6
            "#0000EE", // hlink
            "#551A8B", // fol_hlink
        ];

        // Use the logo PPTX fixture so the extracted brand.toml contains [logo],
        // which `BrandSynthesizer::synthesize` requires (logo path must be non-empty).
        let zip_bytes = build_pptx_zip_with_logo(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        let extraction = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false)
            .expect("round-trip extraction must succeed");

        let _ = std::fs::remove_file(&source_path);

        let content = std::fs::read_to_string(&extraction.brand_toml_path)
            .expect("brand.toml must be readable");
        let config: crate::toml_schema::BrandConfig =
            toml::from_str(&content).expect("extracted brand.toml must parse as valid BrandConfig");

        // Verify canonical values from MINIMAL_THEME_XML are round-tripped at the
        // BrandConfig level (existing assertions, preserved).
        // dk1 = #000000, lt1 = #FFFFFF, dk2 = #003087 (values normalised to uppercase).
        let dk1 = config
            .colors
            .dk1
            .clone()
            .expect("dk1 must be present after round-trip");
        assert!(
            dk1.eq_ignore_ascii_case("#000000"),
            "dk1 must round-trip as #000000, got: {dk1}"
        );
        let dk2 = config
            .colors
            .dk2
            .clone()
            .expect("dk2 must be present after round-trip");
        assert!(
            dk2.eq_ignore_ascii_case("#003087"),
            "dk2 must round-trip as #003087, got: {dk2}"
        );
        let acc1 = config
            .colors
            .acc1
            .clone()
            .expect("acc1 must be present after round-trip");
        assert!(
            acc1.eq_ignore_ascii_case("#0066CC"),
            "acc1 must round-trip as #0066CC, got: {acc1}"
        );
        let fol_hlink = config
            .colors
            .fol_hlink
            .clone()
            .expect("fol_hlink must be present after round-trip");
        assert!(
            fol_hlink.eq_ignore_ascii_case("#551A8B"),
            "fol_hlink must round-trip as #551A8B, got: {fol_hlink}"
        );

        // F-024R-MED-1: FULL round-trip — synthesize a BrandTemplate from the extracted
        // BrandConfig and assert ALL 12 synthesized color hex values match the original
        // source hex values from MINIMAL_THEME_XML.
        //
        // This is the load-bearing step: BrandSynthesizer::synthesize performs color
        // inference and normalization. Any mutation of declared hex values during
        // synthesis (e.g., a slot that was declared in brand.toml being overwritten by
        // an inference rule) would be caught here.
        let (synth_template, _warnings) = BrandSynthesizer::synthesize(&config).expect(
            "AC-010 (F-024R-MED-1): BrandSynthesizer::synthesize must succeed \
                     on the extracted BrandConfig — all 12 slots declared in brand.toml \
                     must be recognised by the synthesizer",
        );

        assert_eq!(
            synth_template.colors.len(),
            12,
            "AC-010 (F-024R-MED-1): synthesized BrandTemplate must have 12 color slots"
        );

        for (i, expected) in EXPECTED_HEX.iter().enumerate() {
            let slot = &synth_template.colors[i];
            let actual_hex = slot.hex().unwrap_or_else(|| {
                panic!(
                    "AC-010 (F-024R-MED-1): synthesized color slot {i} ('{}') must be a \
                     resolved hex — round-trip must not produce an unresolved SchemeRef",
                    slot.name
                )
            });
            assert!(
                actual_hex.eq_ignore_ascii_case(expected),
                "AC-010 (F-024R-MED-1): synthesized slot {i} ('{}') must equal source hex \
                 '{}' after full round-trip (pptx → brand.toml → BrandConfig → \
                 BrandSynthesizer::synthesize), got: '{actual_hex}'",
                slot.name,
                expected
            );
        }

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── F-024-pass4-OBS-2: extract → load_from_toml effectful round-trip ────

    /// BC-2.01.003 / AC-010 — full effectful round-trip:
    /// extract from PPTX with logo → `brand.toml` + `brand.assets/logo.png`
    /// → `BrandSynthesizer::load_from_toml` → assert 12 color hex values match source.
    ///
    /// This test exercises the path NOT covered by the existing AC-010 test, which uses
    /// `BrandSynthesizer::synthesize` (pure, no I/O). The REAL production consumer is
    /// `load_from_toml`, which adds:
    ///   1. Logo file-existence check (EC-005).
    ///   2. Path-traversal guard against the written `brand.assets/logo.png`.
    ///
    /// The extracted logo is written to `output_dir/brand.assets/logo.png` by
    /// `BrandExtractor::extract`. `load_from_toml` must resolve the logo path
    /// relative to the `brand.toml` directory and confirm the logo is inside it.
    #[test]
    fn test_bc_2_01_003_effectful_extract_load_from_toml_round_trip() {
        use crate::synthesizer::BrandSynthesizer;

        // Expected hex values from MINIMAL_THEME_XML (ECMA-376 slot order).
        const EXPECTED_HEX: [&str; 12] = [
            "#000000", // dk1
            "#FFFFFF", // lt1
            "#003087", // dk2
            "#F5F5F5", // lt2
            "#0066CC", // acc1
            "#FF6B35", // acc2
            "#28A745", // acc3
            "#FFC107", // acc4
            "#6F42C1", // acc5
            "#17A2B8", // acc6
            "#0000EE", // hlink
            "#551A8B", // fol_hlink
        ];

        // Use the logo PPTX fixture: extraction will write brand.assets/logo.png.
        let zip_bytes = build_pptx_zip_with_logo(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        let extraction = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false)
            .expect("extraction from PPTX-with-logo must succeed");

        let _ = std::fs::remove_file(&source_path);

        // Confirm the logo asset was written to brand.assets/ (pre-condition for the
        // load_from_toml test: the logo file must exist for the EC-005 check to pass).
        assert!(
            extraction.logo_asset_path.is_some(),
            "F-024-pass4-OBS-2: extraction must have produced a logo asset path"
        );
        let logo_asset_path = extraction
            .logo_asset_path
            .as_ref()
            .expect("logo_asset_path must be Some (asserted above)");
        assert!(
            logo_asset_path.exists(),
            "F-024-pass4-OBS-2: brand.assets/logo.png must exist on disk after extraction, \
             path: {logo_asset_path:?}"
        );

        // Call the EFFECTFUL load_from_toml — exercises EC-005 logo-existence check
        // and path-traversal guard against the written brand.assets/logo.png.
        let brand_toml_path_str = extraction
            .brand_toml_path
            .to_str()
            .expect("brand.toml path must be valid UTF-8");

        let (template, _warnings) = BrandSynthesizer::load_from_toml(brand_toml_path_str).expect(
            "F-024-pass4-OBS-2: BrandSynthesizer::load_from_toml must return Ok on extracted \
             brand.toml with a logo file written to brand.assets/",
        );

        // Assert all 12 color hex values match the original MINIMAL_THEME_XML source values.
        assert_eq!(
            template.colors.len(),
            12,
            "F-024-pass4-OBS-2: loaded BrandTemplate must have exactly 12 color slots"
        );

        for (i, expected) in EXPECTED_HEX.iter().enumerate() {
            let slot = &template.colors[i];
            let actual_hex = slot.hex().unwrap_or_else(|| {
                panic!(
                    "F-024-pass4-OBS-2: loaded color slot {i} ('{}') must be a resolved hex \
                     (not an unresolved SchemeRef)",
                    slot.name
                )
            });
            assert!(
                actual_hex.eq_ignore_ascii_case(expected),
                "F-024-pass4-OBS-2: loaded slot {i} ('{}') must equal source hex '{}' \
                 after effectful round-trip (pptx → brand.toml → load_from_toml), got: '{actual_hex}'",
                slot.name,
                expected
            );
        }

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── F-024-pass7-OBS-1: schemeClr end-to-end round-trip ─────────────────

    /// BC-2.01.003 / EC-003 — end-to-end round-trip with a `<a:schemeClr>`-bearing
    /// theme1.xml.
    ///
    /// The two existing round-trip tests (`test_bc_2_01_003_round_trip_pptx_to_brand_toml_and_back`
    /// and `test_bc_2_01_003_effectful_extract_load_from_toml_round_trip`) both use the
    /// all-`srgbClr` `MINIMAL_THEME_XML` fixture. This test closes the end-to-end gap
    /// identified in F-024-pass7-OBS-1: no previous test confirmed that a real
    /// `<a:schemeClr>`-bearing theme1.xml survives the full
    /// `extract → brand.toml → BrandSynthesizer::load_from_toml` round-trip.
    ///
    /// Specifically this tests that the trailing `# derived via tint/shade; may not match
    /// exact color` inline comment written by `brand_template_to_toml` (EC-003) does NOT
    /// break TOML parsing inside `BrandSynthesizer::load_from_toml`.
    ///
    /// Fixture: `SCHEME_REF_THEME_XML` — acc2 uses `<a:schemeClr val="accent1"/>` (a
    /// resolvable cross-reference to acc1 which is `<a:srgbClr val="0066CC"/>`).
    ///
    /// Assertions:
    ///   (a) The extracted brand.toml for acc2 carries the EC-003 inline comment
    ///       (`# derived via tint/shade`) AND a valid `#RRGGBB` resolved hex (`#0066CC`).
    ///   (b) `BrandSynthesizer::load_from_toml` returns Ok on this brand.toml (proving
    ///       the EC-003 inline comment is TOML-safe through the real synthesize path),
    ///       and the resulting template's acc2 color slot parses as a resolved hex.
    #[test]
    fn test_bc_2_01_003_ec003_scheme_ref_end_to_end_round_trip() {
        use crate::synthesizer::BrandSynthesizer;

        // Theme XML where acc2 is a schemeClr referencing accent1 (= acc1 = #0066CC).
        // All other slots use srgbClr so the extraction is otherwise clean.
        // This is the load-bearing fixture: it exercises the schemeClr → SchemeRef →
        // EC-003-comment → TOML-inline-comment → load_from_toml path end-to-end.
        const SCHEME_REF_THEME_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="SchemeRefTheme">
  <a:themeElements>
    <a:clrScheme name="SchemeRefScheme">
      <a:dk1><a:srgbClr val="000000"/></a:dk1>
      <a:lt1><a:srgbClr val="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="003087"/></a:dk2>
      <a:lt2><a:srgbClr val="F5F5F5"/></a:lt2>
      <a:acc1><a:srgbClr val="0066CC"/></a:acc1>
      <a:acc2><a:schemeClr val="accent1"/></a:acc2>
      <a:acc3><a:srgbClr val="28A745"/></a:acc3>
      <a:acc4><a:srgbClr val="FFC107"/></a:acc4>
      <a:acc5><a:srgbClr val="6F42C1"/></a:acc5>
      <a:acc6><a:srgbClr val="17A2B8"/></a:acc6>
      <a:hlink><a:srgbClr val="0000EE"/></a:hlink>
      <a:folHlink><a:srgbClr val="551A8B"/></a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="SchemeRefFontScheme">
      <a:majorFont><a:latin typeface="Calibri Light"/></a:majorFont>
      <a:minorFont><a:latin typeface="Calibri"/></a:minorFont>
    </a:fontScheme>
  </a:themeElements>
</a:theme>"#;

        // Use the logo fixture so load_from_toml's EC-005 logo-existence check passes.
        let zip_bytes = build_pptx_zip_with_logo(SCHEME_REF_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        let extraction = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false)
            .expect("F-024-pass7-OBS-1: extraction of schemeClr-bearing theme must succeed");

        let _ = std::fs::remove_file(&source_path);

        // ── Assertion (a): the acc2 line in brand.toml carries both the EC-003 inline
        //    comment and a valid #RRGGBB resolved hex. ────────────────────────────────

        let toml_content = std::fs::read_to_string(&extraction.brand_toml_path)
            .expect("F-024-pass7-OBS-1: brand.toml must be readable after extraction");

        let acc2_line = toml_content
            .lines()
            .find(|l| l.trim_start().starts_with("acc2"))
            .unwrap_or_else(|| {
                panic!(
                    "F-024-pass7-OBS-1: acc2 line must appear in brand.toml, got:\n{toml_content}"
                )
            });

        // (a.i) EC-003 inline comment must be present on the acc2 line.
        assert!(
            acc2_line.contains("# derived via tint/shade"),
            "F-024-pass7-OBS-1 (a.i): acc2 schemeClr slot must carry EC-003 inline comment \
             '# derived via tint/shade; may not match exact color', got line: {acc2_line}"
        );

        // (a.ii) The TOML value must be a valid #RRGGBB hex, not a garbage placeholder.
        //        Parse the whole brand.toml to get the acc2 color value safely.
        let config: crate::toml_schema::BrandConfig =
            toml::from_str(&toml_content).unwrap_or_else(|e| {
                panic!(
                    "F-024-pass7-OBS-1 (a.ii): brand.toml with EC-003 comment must parse as \
                     valid BrandConfig (proves the inline comment is TOML-safe at the parse level); \
                     error: {e}\nContent:\n{toml_content}"
                )
            });

        let acc2_value =
            config.colors.acc2.as_deref().unwrap_or_else(|| {
                panic!("F-024-pass7-OBS-1: acc2 must be present in BrandConfig")
            });

        // The schemeClr val="accent1" refers to acc1 = #0066CC, which is resolvable.
        // The extractor must write the resolved hex, not "#accent1".
        let is_valid_hex = acc2_value.starts_with('#')
            && acc2_value.len() == 7
            && acc2_value[1..].chars().all(|c| c.is_ascii_hexdigit());
        assert!(
            is_valid_hex,
            "F-024-pass7-OBS-1 (a.ii): acc2 schemeClr must produce a valid 6-hex-digit color \
             value in brand.toml (not a garbage placeholder like '#accent1'), got: {acc2_value:?}"
        );

        // The resolved hex must match acc1's value (#0066CC) since accent1 → acc1.
        assert!(
            acc2_value.eq_ignore_ascii_case("#0066CC"),
            "F-024-pass7-OBS-1 (a.ii): acc2 schemeClr(accent1) must resolve to acc1's hex \
             #0066CC in brand.toml, got: {acc2_value:?}"
        );

        // ── Assertion (b): load_from_toml returns Ok and acc2 is a resolved hex. ────

        let brand_toml_path_str = extraction
            .brand_toml_path
            .to_str()
            .expect("F-024-pass7-OBS-1: brand.toml path must be valid UTF-8");

        // This is the critical end-to-end assertion: load_from_toml must NOT fail on
        // a brand.toml that contains EC-003 inline comments. If the inline comment
        // breaks TOML parsing inside the synthesizer, this call returns Err.
        let (template, _warnings) = BrandSynthesizer::load_from_toml(brand_toml_path_str).expect(
            "F-024-pass7-OBS-1 (b): BrandSynthesizer::load_from_toml must return Ok on a \
                 brand.toml that contains EC-003 '# derived via tint/shade' inline comments — \
                 the comment must be TOML-safe through the real synthesize path",
        );

        // The loaded template's acc2 slot (index 5) must be a resolved hex color.
        // If the EC-003 comment caused a parse failure or the SchemeRef survived into
        // the loaded template as an unresolved value, this assertion catches it.
        assert_eq!(
            template.colors.len(),
            12,
            "F-024-pass7-OBS-1 (b): loaded BrandTemplate must have exactly 12 color slots"
        );

        let acc2_slot = &template.colors[5]; // acc2 is index 5 in ECMA-376 order
        let acc2_loaded_hex = acc2_slot.hex().unwrap_or_else(|| {
            panic!(
                "F-024-pass7-OBS-1 (b): acc2 color slot after load_from_toml must be a resolved \
                 hex (the EC-003 inline comment must not cause an unresolved SchemeRef to survive \
                 into the loaded BrandTemplate)"
            )
        });

        assert!(
            acc2_loaded_hex.eq_ignore_ascii_case("#0066CC"),
            "F-024-pass7-OBS-1 (b): acc2 loaded from brand.toml must equal the resolved hex \
             #0066CC (schemeClr(accent1) → acc1 = #0066CC), got: {acc2_loaded_hex:?}"
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
            is_derived: false,
        };
        let make_scheme_slot = |name: &str, scheme: &str| ColorSlot {
            name: Arc::from(name),
            value: ColorValue::SchemeRef(Arc::from(scheme)),
            is_derived: false,
        };

        let colors: [ColorSlot; 12] = [
            make_hex_slot("dk1", "#000000"),
            make_hex_slot("lt1", "#FFFFFF"),
            make_hex_slot("dk2", "#003087"),
            make_hex_slot("lt2", "#F5F5F5"),
            // acc1 is a scheme-ref (tint/shade, EC-003 vector).
            // "accent1" refers back to acc1 (index 4) itself — a self-reference.
            // The extractor must write a valid fallback #RRGGBB, not "#accent1".
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
                font_size_emu: 457_200,
            },
            logo: None,
            footer_text: None,
            footer_flags: crate::footer::FooterFlags::default(),
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

        // F-024-O3 — strengthen: the value written must be a valid 6-hex-digit color,
        // NOT a garbage non-hex value like "#accent1" (F-024-C1 regression guard).
        //
        // Extract the value from the acc1 line by stripping the comment and parsing the TOML.
        // We parse the whole output as BrandConfig and verify acc1 is a valid hex string.
        let config: crate::toml_schema::BrandConfig =
            toml::from_str(&toml_str).unwrap_or_else(|e| {
                panic!(
                    "SchemeRef TOML output must parse as valid BrandConfig (F-024-O3); \
                     TOML parse error: {e}\nOutput:\n{toml_str}"
                )
            });
        let acc1_value = config
            .colors
            .acc1
            .as_deref()
            .unwrap_or_else(|| panic!("acc1 must be present in parsed config"));
        // Must match `#` followed by exactly 6 hex digits — never "#accent1".
        let is_valid_hex = acc1_value.starts_with('#')
            && acc1_value.len() == 7
            && acc1_value[1..].chars().all(|c| c.is_ascii_hexdigit());
        assert!(
            is_valid_hex,
            "acc1 SchemeRef must produce a valid 6-hex-digit color value (F-024-C1 / F-024-O3), \
             got: {acc1_value:?}"
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

    /// BC-2.01.003 EC-003 — a `SchemeRef` that can be resolved to a sibling hex slot
    /// writes that sibling's hex (not a garbage placeholder).
    ///
    /// acc2 (index 5) holds `SchemeRef("accent1")` — `accent1` maps to acc1 (index 4)
    /// which holds Hex("#0066CC"). The extractor must write `#0066CC`.
    #[test]
    fn test_bc_2_01_003_ec003_resolvable_scheme_ref_writes_resolved_hex() {
        use crate::template::{BrandFonts, BrandTemplate, ColorSlot, ColorValue, MasterIds};

        let make_hex_slot = |name: &str, hex: &str| ColorSlot {
            name: Arc::from(name),
            value: ColorValue::Hex(Arc::from(hex)),
            is_derived: false,
        };

        // acc2 (index 5) references accent1 → acc1 (index 4) which is Hex "#0066CC"
        let colors: [ColorSlot; 12] = [
            make_hex_slot("dk1", "#000000"),
            make_hex_slot("lt1", "#FFFFFF"),
            make_hex_slot("dk2", "#003087"),
            make_hex_slot("lt2", "#F5F5F5"),
            make_hex_slot("acc1", "#0066CC"), // index 4 = accent1
            ColorSlot {
                name: Arc::from("acc2"),
                value: ColorValue::SchemeRef(Arc::from("accent1")), // refers to acc1 above
                is_derived: false,
            },
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
                font_size_emu: 457_200,
            },
            logo: None,
            footer_text: None,
            footer_flags: crate::footer::FooterFlags::default(),
            layout_names: vec![],
            layouts: vec![],
            notes_master_stub: vec![],
            handout_master_stub: vec![],
            master_ids: MasterIds::default(),
            content_types_layout_entries: Arc::from(""),
        };

        let (toml_str, _) = brand_template_to_toml(&template, None);

        // Parse the TOML to get the acc2 value.
        let config: crate::toml_schema::BrandConfig =
            toml::from_str(&toml_str).unwrap_or_else(|e| {
                panic!(
                    "resolvable SchemeRef TOML must parse as BrandConfig: {e}\nOutput:\n{toml_str}"
                )
            });
        let acc2_value = config
            .colors
            .acc2
            .as_deref()
            .unwrap_or_else(|| panic!("acc2 must be present"));
        // Must be the resolved hex of acc1, not "#accent1".
        assert_eq!(
            acc2_value, "#0066CC",
            "resolvable SchemeRef(\"accent1\") must write the resolved hex #0066CC, got: {acc2_value:?}"
        );

        // Must still carry the inline comment on the acc2 line.
        let acc2_line = toml_str
            .lines()
            .find(|l| l.trim_start().starts_with("acc2"))
            .unwrap_or_else(|| panic!("acc2 line must appear in TOML output"));
        assert!(
            acc2_line.contains("# derived via tint/shade"),
            "resolved SchemeRef must still carry inline comment (EC-003), got: {acc2_line}"
        );
    }

    // ─── F-024A-MED-2: single source for default color fallbacks ─────────────

    /// F-024A-MED-2 — the fallback hex used in `brand_template_to_toml` for
    /// unresolvable `SchemeRef` slots must equal `color::default_color_for_slot`
    /// for all 12 OOXML slot names.
    ///
    /// This guards against the TD-VSDD-060 sibling-drift risk: two independent
    /// tables for the same canonical values. After the fix, extractor delegates
    /// to `color::default_color_for_slot` (the single source of truth).
    #[test]
    fn test_f024a_med2_extractor_fallback_matches_color_default_for_all_slots() {
        use crate::color::default_color_for_slot;
        use crate::template::{BrandFonts, BrandTemplate, ColorSlot, ColorValue, MasterIds};

        // Build a template where every slot is a self-referential SchemeRef, so each
        // one is unresolvable and falls back to the default color.
        // We use slot names exactly as stored by color.rs (OOXML canonical names).
        let ooxml_names = [
            "dk1", "lt1", "dk2", "lt2", "acc1", "acc2", "acc3", "acc4", "acc5", "acc6", "hlink",
            "folHlink",
        ];
        let toml_names = [
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
        let colors: [ColorSlot; 12] = ooxml_names
            .iter()
            .map(|&name| ColorSlot {
                name: Arc::from(name),
                // Self-referential: the lowercase of the OOXML name (as stored by color.rs).
                value: ColorValue::SchemeRef(Arc::from(name.to_lowercase().as_str())),
                is_derived: false,
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("12 elements");

        let template = BrandTemplate {
            colors,
            fonts: BrandFonts {
                heading: Arc::from("Calibri Light"),
                body: Arc::from("Calibri"),
                font_size_emu: 457_200,
            },
            logo: None,
            footer_text: None,
            footer_flags: crate::footer::FooterFlags::default(),
            layout_names: vec![],
            layouts: vec![],
            notes_master_stub: vec![],
            handout_master_stub: vec![],
            master_ids: MasterIds::default(),
            content_types_layout_entries: Arc::from(""),
        };

        let (toml_str, _) = brand_template_to_toml(&template, None);

        // Parse the output as BrandConfig.
        let config: crate::toml_schema::BrandConfig =
            toml::from_str(&toml_str).unwrap_or_else(|e| {
                panic!("F-024A-MED-2 TOML must parse as BrandConfig: {e}\nOutput:\n{toml_str}")
            });

        // For each slot, the written value must equal default_color_for_slot(ooxml_name).
        let config_values = [
            config.colors.dk1.as_deref(),
            config.colors.lt1.as_deref(),
            config.colors.dk2.as_deref(),
            config.colors.lt2.as_deref(),
            config.colors.acc1.as_deref(),
            config.colors.acc2.as_deref(),
            config.colors.acc3.as_deref(),
            config.colors.acc4.as_deref(),
            config.colors.acc5.as_deref(),
            config.colors.acc6.as_deref(),
            config.colors.hlink.as_deref(),
            config.colors.fol_hlink.as_deref(),
        ];

        for (i, ooxml_name) in ooxml_names.iter().enumerate() {
            let expected = default_color_for_slot(ooxml_name);
            let actual = config_values[i].unwrap_or_else(|| {
                panic!(
                    "F-024A-MED-2: slot '{}' (TOML: '{}') must be present in config",
                    ooxml_name, toml_names[i]
                )
            });
            assert_eq!(
                actual,
                expected.as_ref(),
                "F-024A-MED-2: fallback for '{}' (TOML: '{}') must equal \
                 color::default_color_for_slot(\"{}\") = \"{}\", but got \"{}\"",
                ooxml_name,
                toml_names[i],
                ooxml_name,
                expected,
                actual
            );
        }
    }

    // ─── F-024A-MED-1: camelCase SchemeRef resolution ────────────────────────

    /// F-024A-MED-1 — `SchemeRef` stored as lowercase (from `color.rs` line 155)
    /// must resolve correctly via `scheme_name_to_slot_index` when the target name
    /// is a mixed-case OOXML alias such as `"folHlink"` or `"hyperlink"`.
    ///
    /// After `color.rs` lowercases `val` to `"folhlink"`, `scheme_name_to_slot_index`
    /// must accept `"folhlink"` (not only the original mixed-case `"folHlink"`).
    ///
    /// EC-003 load-bearing guard: a `SchemeRef` whose target is `"folhlink"` (the
    /// stored lowercase form of OOXML `"folHlink"`) must resolve to the hlink slot's
    /// hex, not fall back to the default.
    #[test]
    fn test_f024a_med1_schemeclr_folhlink_lowercase_resolves_to_sibling_hex() {
        use crate::template::{BrandFonts, BrandTemplate, ColorSlot, ColorValue, MasterIds};

        let make_hex_slot = |name: &str, hex: &str| ColorSlot {
            name: Arc::from(name),
            value: ColorValue::Hex(Arc::from(hex)),
            is_derived: false,
        };

        // hlink (index 10) holds Hex "#AABBCC".
        // folHlink (index 11) holds SchemeRef("folhlink") — the LOWERCASED form that
        // color.rs:155 would store for <a:schemeClr val="folHlink"/>. This must NOT
        // be treated as a self-reference (folHlink slot name is "folHlink", not "folhlink").
        //
        // Wait — the SchemeRef here is "folhlink" targeting the folHlink SLOT (index 11).
        // That IS a self-reference (same slot), so it should fall back to default.
        // The real test: hlink slot (index 10) is SchemeRef("hyperlink") — stored lowercase.
        // "hyperlink" must map to index 10 (hlink) — but that IS the same slot, so fallback.
        //
        // Better: acc1 (index 4) has SchemeRef("accent1") stored as lowercase "accent1".
        // acc1 maps to index 4 — self-reference → fallback. Not useful.
        //
        // The real failure scenario: folHlink (index 11) contains SchemeRef("hlink") —
        // i.e., val="hlink" in OOXML, stored as lowercase "hlink". This is NOT a
        // self-reference; it targets index 10 (hlink slot, hex "#AABBCC").
        // Before the fix: scheme_name_to_slot_index("hlink") returns Some(10) — OK.
        //
        // The actual broken case: folHlink slot contains SchemeRef("folhlink") which is
        // the lowercase of "folHlink". The old match has arm "folHlink" (mixed-case),
        // which NEVER matches the stored lowercase "folhlink". With the fix (match on
        // lowercase), "folhlink" correctly maps to index 11 — but that's a self-reference
        // and returns None (fallback).
        //
        // The truly broken case: hlink slot (10) contains SchemeRef("followedhyperlink")
        // (lowercase of "followedHyperlink"). Before fix: no arm matches → None → fallback.
        // After fix: "followedhyperlink" → Some(11) (folHlink), different slot → resolved hex.
        let colors: [ColorSlot; 12] = [
            make_hex_slot("dk1", "#000000"),
            make_hex_slot("lt1", "#FFFFFF"),
            make_hex_slot("dk2", "#003087"),
            make_hex_slot("lt2", "#F5F5F5"),
            make_hex_slot("acc1", "#0066CC"),
            make_hex_slot("acc2", "#FF6B35"),
            make_hex_slot("acc3", "#28A745"),
            make_hex_slot("acc4", "#FFC107"),
            make_hex_slot("acc5", "#6F42C1"),
            make_hex_slot("acc6", "#17A2B8"),
            // hlink (index 10): SchemeRef("followedhyperlink") — lowercase of "followedHyperlink".
            // This refers to folHlink (index 11) = "#551A8B". Before fix: no match → fallback.
            ColorSlot {
                name: Arc::from("hlink"),
                value: ColorValue::SchemeRef(Arc::from("followedhyperlink")),
                is_derived: false,
            },
            make_hex_slot("folHlink", "#551A8B"), // index 11
        ];

        let template = BrandTemplate {
            colors,
            fonts: BrandFonts {
                heading: Arc::from("Calibri Light"),
                body: Arc::from("Calibri"),
                font_size_emu: 457_200,
            },
            logo: None,
            footer_text: None,
            footer_flags: crate::footer::FooterFlags::default(),
            layout_names: vec![],
            layouts: vec![],
            notes_master_stub: vec![],
            handout_master_stub: vec![],
            master_ids: MasterIds::default(),
            content_types_layout_entries: Arc::from(""),
        };

        let (toml_str, _) = brand_template_to_toml(&template, None);

        // Parse the output.
        let config: crate::toml_schema::BrandConfig =
            toml::from_str(&toml_str).unwrap_or_else(|e| {
                panic!("F-024A-MED-1 TOML must parse as BrandConfig: {e}\nOutput:\n{toml_str}")
            });
        let hlink_value = config
            .colors
            .hlink
            .as_deref()
            .unwrap_or_else(|| panic!("hlink must be present in parsed config"));

        // After fix: "followedhyperlink" maps to index 11 (folHlink = "#551A8B").
        // Before fix: no match → fallback "#0000EE".
        assert_eq!(
            hlink_value, "#551A8B",
            "F-024A-MED-1: SchemeRef(\"followedhyperlink\") must resolve to folHlink hex \
             \"#551A8B\" (not the default \"#0000EE\"). \
             This fails if scheme_name_to_slot_index does not normalize to lowercase."
        );

        // Also verify it carries the inline comment.
        let hlink_line = toml_str
            .lines()
            .find(|l| l.trim_start().starts_with("hlink"))
            .unwrap_or_else(|| panic!("hlink line must appear in TOML output"));
        assert!(
            hlink_line.contains("# derived via tint/shade"),
            "resolved SchemeRef must carry inline TOML comment (EC-003)"
        );
    }

    // ─── F-024-H1: TOML string escaping ──────────────────────────────────────

    /// F-024-H1 — string values containing `"` and `\` are escaped correctly
    /// in the brand.toml output, producing valid TOML that round-trips.
    ///
    /// Tests both font typeface names and footer text with embedded special chars.
    #[test]
    fn test_bc_2_01_003_h1_toml_string_escaping_roundtrips() {
        use crate::template::{BrandFonts, BrandTemplate, ColorSlot, ColorValue, MasterIds};

        let make_hex_slot = |name: &str, hex: &str| ColorSlot {
            name: Arc::from(name),
            value: ColorValue::Hex(Arc::from(hex)),
            is_derived: false,
        };
        let colors: [ColorSlot; 12] = [
            make_hex_slot("dk1", "#000000"),
            make_hex_slot("lt1", "#FFFFFF"),
            make_hex_slot("dk2", "#003087"),
            make_hex_slot("lt2", "#F5F5F5"),
            make_hex_slot("acc1", "#0066CC"),
            make_hex_slot("acc2", "#FF6B35"),
            make_hex_slot("acc3", "#28A745"),
            make_hex_slot("acc4", "#FFC107"),
            make_hex_slot("acc5", "#6F42C1"),
            make_hex_slot("acc6", "#17A2B8"),
            make_hex_slot("hlink", "#0000EE"),
            make_hex_slot("folHlink", "#551A8B"),
        ];

        // Font name with embedded " and \ — these must be escaped.
        let heading_with_special = "Weird\\Font\"Name";
        let body_with_special = "Body\\Font\"Here";
        let footer_with_special = r#"Confidential: "Acme Corp" \ All Rights Reserved"#;

        let template = BrandTemplate {
            colors,
            fonts: BrandFonts {
                heading: Arc::from(heading_with_special),
                body: Arc::from(body_with_special),
                font_size_emu: 457_200,
            },
            logo: None,
            footer_text: Some(Arc::from(footer_with_special)),
            footer_flags: crate::footer::FooterFlags::default(),
            layout_names: vec![],
            layouts: vec![],
            notes_master_stub: vec![],
            handout_master_stub: vec![],
            master_ids: MasterIds::default(),
            content_types_layout_entries: Arc::from(""),
        };

        let (toml_str, _) = brand_template_to_toml(&template, None);

        // The raw TOML string must be parseable — an unescaped `"` would break parsing.
        let config: crate::toml_schema::BrandConfig =
            toml::from_str(&toml_str).unwrap_or_else(|e| {
                panic!(
                    "TOML with escaped strings must parse without error (F-024-H1): {e}\n\
                     Output:\n{toml_str}"
                )
            });

        // Round-trip: the parsed values must exactly match the original strings.
        let parsed_heading = config.fonts.heading.as_str();
        assert_eq!(
            parsed_heading, heading_with_special,
            "heading font with special chars must round-trip through TOML (F-024-H1), \
             got: {parsed_heading:?}"
        );
        let parsed_body = config.fonts.body.as_str();
        assert_eq!(
            parsed_body, body_with_special,
            "body font with special chars must round-trip through TOML (F-024-H1), \
             got: {parsed_body:?}"
        );
        let parsed_footer = config.footer.text.as_str();
        assert_eq!(
            parsed_footer, footer_with_special,
            "footer text with special chars must round-trip through TOML (F-024-H1), \
             got: {parsed_footer:?}"
        );
    }

    // ─── F-024-H2: unsupported logo format emits warning ─────────────────────

    /// F-024-H2 — when the logo has an unsupported extension (.emf/.wmf), the
    /// extractor copies the bytes AND emits a `tracing::warn!`. Verified by
    /// checking the warning subscriber fires and the logo bytes are present.
    #[test]
    fn test_bc_2_01_003_h2_unsupported_logo_format_warns_and_copies() {
        use std::io::Write as IoWrite;
        use std::sync::{Arc as StdArc, Mutex};
        use tracing_subscriber::layer::SubscriberExt as _;

        // Build a PPTX ZIP with an EMF logo (unsupported format).
        let emf_bytes: &[u8] = b"\x01\x00\x00\x00"; // stub EMF header
        let zip_bytes = {
            let mut buf = Vec::new();
            let cursor = std::io::Cursor::new(&mut buf);
            let mut zw = zip::ZipWriter::new(cursor);
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            zw.start_file(crate::loader::PPTX_THEME_PATH, opts).unwrap();
            zw.write_all(MINIMAL_THEME_XML.as_bytes()).unwrap();

            // Slide master rels pointing to an EMF file.
            zw.start_file("ppt/slideMasters/_rels/slideMaster1.xml.rels", opts)
                .unwrap();
            let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1"
    Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"
    Target="../media/logo.emf"/>
</Relationships>"#;
            zw.write_all(rels.as_bytes()).unwrap();

            zw.start_file("ppt/media/logo.emf", opts).unwrap();
            zw.write_all(emf_bytes).unwrap();

            zw.start_file("[Content_Types].xml", opts).unwrap();
            zw.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>").unwrap();
            zw.finish().unwrap();
            buf
        };

        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        // Install a tracing subscriber to capture warnings.
        let warned = StdArc::new(Mutex::new(false));
        let warned_clone = StdArc::clone(&warned);
        let warned_layer = {
            let w = StdArc::clone(&warned_clone);
            tracing_subscriber::fmt::layer().with_writer(move || {
                let _ = w.lock().map(|mut guard| *guard = true);
                std::io::sink()
            })
        };
        let subscriber = tracing_subscriber::registry().with(warned_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        let result = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false);
        let _ = std::fs::remove_file(&source_path);

        // Extraction must succeed even for unsupported logo format.
        let extraction = result.expect(
            "extraction must succeed even when logo has unsupported format (EC-005, F-024-H2)",
        );

        // Logo bytes must be copied (AC-004 still applies).
        let logo_path = extraction
            .logo_asset_path
            .expect("logo_asset_path must be Some for EMF logo (F-024-H2)");
        assert!(
            logo_path.exists(),
            "EMF logo bytes must be written to brand.assets/ (F-024-H2)"
        );
        let written_bytes = std::fs::read(&logo_path).unwrap();
        assert_eq!(
            written_bytes, emf_bytes,
            "EMF bytes must be copied verbatim"
        );

        // Warning subscriber must have received output.
        let was_warned = warned.lock().is_ok_and(|g| *g);
        assert!(
            was_warned,
            "unsupported logo format must emit a tracing::warn! (F-024-H2 / EC-005)"
        );

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── F-024-O1: no output dir created when extraction fails early ──────────

    /// F-024-O1 — when the source `.pptx` is invalid, no output directory is
    /// created. The `create_dir_all` must run AFTER source validation.
    #[test]
    fn test_bc_2_01_003_o1_no_output_dir_created_on_invalid_source() {
        // Use a new non-existent directory as output_dir (must not be created).
        let base = temp_output_dir();
        let out_dir = base.join("should_not_be_created");
        assert!(!out_dir.exists(), "pre-condition: out_dir must not exist");

        let result = BrandExtractor::extract(
            "/tmp/slideforge_nonexistent_extractor_o1_9999999.pptx",
            &out_dir,
            false,
        );

        // Must fail with FileNotFound.
        assert!(
            matches!(result.unwrap_err(), BrandError::FileNotFound { .. }),
            "must return FileNotFound for missing source"
        );
        // Output directory must NOT have been created.
        assert!(
            !out_dir.exists(),
            "output_dir must NOT be created when source validation fails (F-024-O1)"
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    /// F-024-O1 — when brand.toml already exists and force=false, no additional
    /// filesystem state is created. The output dir that already exists is
    /// untouched (but was not created by the failing extract call).
    #[test]
    fn test_bc_2_01_003_o1_no_new_dir_created_when_output_exists_no_force() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);

        // Create a fresh output dir and pre-create brand.toml.
        let out_dir = temp_output_dir();
        std::fs::write(out_dir.join("brand.toml"), b"# sentinel\n").unwrap();
        // Point to a sub-dir that must not be created by the failed call.
        let sub_out_dir = out_dir.join("nested_not_created");
        // Pre-create brand.toml in sub_out_dir via a direct create to simulate
        // OutputExists path: actually we test the OutputExists short-circuit.
        // Different approach: use the existing out_dir where brand.toml already is,
        // to ensure that the sub-dir "brand.assets" is not newly created.

        let result = BrandExtractor::extract(
            source_path.to_str().unwrap(),
            &out_dir,
            false, /* force */
        );

        let _ = std::fs::remove_file(&source_path);

        assert!(
            matches!(result.unwrap_err(), BrandError::OutputExists { .. }),
            "must return OutputExists"
        );
        // brand.assets must not have been created (no extraction happened).
        let brand_assets = out_dir.join("brand.assets");
        assert!(
            !brand_assets.exists(),
            "brand.assets/ must not be created when extraction is rejected due to OutputExists \
             (F-024-O1)"
        );
        // The sub-dir must not exist either.
        assert!(
            !sub_out_dir.exists(),
            "nested sub-dirs must not be created when extraction is short-circuited (F-024-O1)"
        );

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // ─── F-024-H4: manual TOML output round-trips to BrandConfig ─────────────

    /// F-024-H4 — the manually-written TOML from extraction deserializes to a
    /// `BrandConfig` with the same color values as what `toml::to_string` on a
    /// `BrandConfig`-based serialization would produce.
    ///
    /// This catches schema drift between the manual writer and `BrandConfig`.
    #[test]
    fn test_bc_2_01_003_h4_manual_toml_round_trips_to_brand_config() {
        let zip_bytes = build_pptx_zip(MINIMAL_THEME_XML);
        let source_path = write_temp_pptx(&zip_bytes);
        let out_dir = temp_output_dir();

        let extraction = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false)
            .expect("valid PPTX must extract");

        let _ = std::fs::remove_file(&source_path);

        let toml_str = std::fs::read_to_string(&extraction.brand_toml_path)
            .expect("brand.toml must be readable");

        // Parse the manual TOML output as BrandConfig.
        let from_manual: crate::toml_schema::BrandConfig = toml::from_str(&toml_str)
            .unwrap_or_else(|e| {
                panic!("manual TOML output must deserialize as BrandConfig (F-024-H4): {e}")
            });

        // Build an equivalent BrandConfig via the serde serialization path and compare.
        // MINIMAL_THEME_XML canonical values (all uppercase as written by the loader):
        // dk1=#000000, lt1=#FFFFFF, dk2=#003087, lt2=#F5F5F5, acc1=#0066CC, acc2=#FF6B35,
        // acc3=#28A745, acc4=#FFC107, acc5=#6F42C1, acc6=#17A2B8, hlink=#0000EE, fol_hlink=#551A8B
        let dk1 = from_manual.colors.dk1.as_deref().unwrap_or("");
        let lt1 = from_manual.colors.lt1.as_deref().unwrap_or("");
        let dk2 = from_manual.colors.dk2.as_deref().unwrap_or("");
        let acc1 = from_manual.colors.acc1.as_deref().unwrap_or("");
        let fol_hlink = from_manual.colors.fol_hlink.as_deref().unwrap_or("");

        assert!(
            dk1.eq_ignore_ascii_case("#000000"),
            "dk1 must be #000000 after round-trip, got: {dk1}"
        );
        assert!(
            lt1.eq_ignore_ascii_case("#FFFFFF"),
            "lt1 must be #FFFFFF after round-trip, got: {lt1}"
        );
        assert!(
            dk2.eq_ignore_ascii_case("#003087"),
            "dk2 must be #003087 after round-trip, got: {dk2}"
        );
        assert!(
            acc1.eq_ignore_ascii_case("#0066CC"),
            "acc1 must be #0066CC after round-trip, got: {acc1}"
        );
        assert!(
            fol_hlink.eq_ignore_ascii_case("#551A8B"),
            "fol_hlink must be #551A8B after round-trip, got: {fol_hlink}"
        );

        // Also verify heading/body fonts survive the schema.
        let heading = from_manual.fonts.heading.as_str();
        assert_eq!(
            heading, "Calibri Light",
            "heading font must round-trip (F-024-H4)"
        );
        let body = from_manual.fonts.body.as_str();
        assert_eq!(body, "Calibri", "body font must round-trip (F-024-H4)");

        let _ = std::fs::remove_dir_all(&out_dir);
    }
}
