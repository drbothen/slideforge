//! Per-slide brand overlay — loading and resolution for `slideforge-brand`.
//!
//! This module interprets a [`slideforge_types::SlideOverlay`] (thin parse-time
//! metadata from `slideforge-types`) into a fully-loaded [`BrandOverlay`] (with
//! logo bytes read from disk) at export time.
//!
//! ## Architecture: No Circular Dependency
//!
//! `slideforge-types` is a leaf crate. `slideforge-brand` depends on it. The flow
//! is unidirectional:
//!
//! ```text
//! DSL parser  →  SlideOverlay (in slideforge-types)
//!             →  resolve_overlay() (in slideforge-brand)
//!             →  BrandOverlay (with loaded bytes)
//!             →  PPTX exporter (STORY-037)
//! ```
//!
//! ## Single-Master Invariant (BC-2.02.002)
//!
//! `BrandOverlay` contains ONLY shape-override data (logo bytes, footer text,
//! confidentiality string). It contains NO master reference, no layout switch,
//! and no template path. The type-level absence of these fields is the
//! compile-time enforcement of DI-016.
//!
//! ## `resolve_overlay` (STORY-025)
//!
//! Reads logo bytes from disk, infers MIME type, and passes through footer text
//! and confidentiality strings. Returns `Ok(None)` for empty overlays (no-op)
//! and `Err(BrandError::FileNotFound)` for missing logo paths.

use std::path::Path;
use std::sync::Arc;

use slideforge_types::SlideOverlay;

use crate::error::BrandError;

// ─── BrandOverlay ────────────────────────────────────────────────────────────

/// A fully-loaded per-slide brand overlay — ready for use by the PPTX exporter.
///
/// Produced by [`resolve_overlay`] from a [`SlideOverlay`] parse-time stub.
/// Consumed by the PPTX exporter (STORY-037) to apply shape-level overrides on
/// individual slides without modifying the slide master or layout.
///
/// ## Single-Master Invariant
///
/// This struct deliberately contains NO field for master path, layout index, or
/// template reference. The absence of such fields is the structural (compile-time)
/// enforcement of BC-2.02.002 (DI-016): it is impossible to represent a master
/// switch via a `BrandOverlay`.
///
/// ## `Some("")` vs `None` for `footer_text`
///
/// - `None` — no footer change; deck-level brand footer text is used unchanged.
/// - `Some("")` — explicitly clears the footer text content (placeholder structure
///   preserved; BC-2.02.001 invariant 3).
/// - `Some(text)` — replaces footer placeholder text on this slide only.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BrandOverlay {
    /// Optional logo override for this slide.
    ///
    /// `None` — no logo override; deck-level brand logo is used.
    /// `Some(logo)` — this slide's logo placeholder is replaced with these bytes.
    pub logo: Option<LogoOverride>,

    /// Optional footer text override for this slide.
    ///
    /// `None` — no change; deck-level brand footer applies.
    /// `Some("")` — clears footer text content (placeholder structure preserved).
    /// `Some(text)` — replaces footer text on this slide.
    pub footer_text: Option<Arc<str>>,

    /// Optional confidentiality label for this slide.
    ///
    /// `None` — no confidentiality label shape added.
    /// `Some(text)` — adds/updates confidentiality label (positioned by PPTX exporter).
    pub confidentiality: Option<Arc<str>>,
}

// ─── LogoOverride ────────────────────────────────────────────────────────────

/// A fully-loaded logo override for a single slide.
///
/// Mirrors [`crate::template::LogoAsset::Loaded`] but represents a *per-slide*
/// override rather than the deck-level brand logo. Produced by [`resolve_overlay`]
/// after reading the image bytes from the filesystem path declared in
/// [`SlideOverlay::logo_path`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogoOverride {
    /// The filesystem path that was resolved (original value from `SlideOverlay`).
    ///
    /// Preserved for diagnostics and error messages.
    pub path: Arc<str>,

    /// Raw bytes of the logo image file.
    ///
    /// Read from `path` at brand-apply time by [`resolve_overlay`].
    pub bytes: Vec<u8>,

    /// MIME type inferred from the file extension (e.g., `"image/png"`, `"image/jpeg"`).
    pub media_type: Arc<str>,
}

// ─── infer_media_type ────────────────────────────────────────────────────────

/// Infer the MIME type of a logo image from its file extension.
///
/// Supports `.png` → `"image/png"`, `.jpg`/`.jpeg` → `"image/jpeg"`,
/// `.gif` → `"image/gif"`, `.svg` → `"image/svg+xml"`. Unknown extensions
/// return `"application/octet-stream"`.
#[must_use]
pub fn infer_media_type(path: &Path) -> Arc<str> {
    let mime = match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    };
    Arc::from(mime)
}

// ─── resolve_overlay ─────────────────────────────────────────────────────────

/// Resolve a parse-time [`SlideOverlay`] into a fully-loaded [`BrandOverlay`].
///
/// Reads logo bytes from the filesystem if `raw.logo_path` is set. Validates
/// that the logo file exists — returns `Err(BrandError::FileNotFound)` if not.
///
/// Returns `Ok(None)` if all fields in `raw` are `None` (empty overlay — no-op).
/// The caller (PPTX exporter, STORY-037) skips overlay application in this case.
///
/// # Arguments
///
/// * `raw` — the parse-time `SlideOverlay` from `slideforge-types::Slide`.
/// * `root_dir` — the directory containing the `.sf` source file, used to
///   resolve relative logo paths.
///
/// # Errors
///
/// - [`BrandError::FileNotFound`] — if `raw.logo_path` is set and the file does
///   not exist at the resolved path (BC-2.02.001 edge case EC-001, `E-BRD-001`).
pub fn resolve_overlay(
    raw: &SlideOverlay,
    root_dir: &Path,
) -> Result<Option<BrandOverlay>, BrandError> {
    // Empty overlay (all fields None) → no-op per BC-2.02.001 EC-004.
    if raw.is_empty() {
        return Ok(None);
    }

    // Resolve logo if a path is declared.
    let logo = match raw.logo_path.as_ref() {
        None => None,
        Some(logo_path_str) => {
            let logo_path = root_dir.join(logo_path_str.as_ref());
            if !logo_path.exists() {
                return Err(BrandError::FileNotFound {
                    path: Arc::clone(logo_path_str),
                    span: raw.span.clone(),
                });
            }
            let bytes = std::fs::read(&logo_path).map_err(|_| BrandError::FileNotFound {
                path: Arc::clone(logo_path_str),
                span: raw.span.clone(),
            })?;
            let media_type = infer_media_type(&logo_path);
            Some(LogoOverride {
                path: Arc::clone(logo_path_str),
                bytes,
                media_type,
            })
        },
    };

    Ok(Some(BrandOverlay {
        logo,
        footer_text: raw.footer_text.clone(),
        confidentiality: raw.confidentiality.clone(),
    }))
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use slideforge_types::{SlideOverlay, SourceSpan};
    use tempfile::tempdir;

    use super::*;

    // ── Helpers ────────────────────────────────────────────────────────────────

    /// Create a minimal `SlideOverlay` with all fields `None`.
    fn empty_overlay() -> SlideOverlay {
        SlideOverlay {
            logo_path: None,
            footer_text: None,
            confidentiality: None,
            span: SourceSpan::default(),
        }
    }

    // ── AC-001 / BC-2.02.001 precondition 2: BrandOverlay struct fields ────────

    /// AC-001: `BrandOverlay` has the correct fields (logo, `footer_text`, confidentiality).
    /// This is a compile-time structural test. If the fields were wrong, this would
    /// not compile.
    #[test]
    fn test_bc_2_02_001_brand_overlay_fields_present() {
        let overlay = BrandOverlay {
            logo: None,
            footer_text: Some(Arc::from("CONFIDENTIAL")),
            confidentiality: Some(Arc::from("DO NOT DISTRIBUTE")),
        };
        assert!(overlay.logo.is_none());
        assert_eq!(overlay.footer_text.as_deref(), Some("CONFIDENTIAL"));
        assert_eq!(
            overlay.confidentiality.as_deref(),
            Some("DO NOT DISTRIBUTE")
        );
    }

    /// AC-001: `LogoOverride` has path, bytes, and `media_type` fields.
    #[test]
    fn test_bc_2_02_001_logo_override_fields_present() {
        let logo = LogoOverride {
            path: Arc::from("client-logo.png"),
            bytes: vec![0x89, 0x50, 0x4E, 0x47], // PNG magic
            media_type: Arc::from("image/png"),
        };
        assert_eq!(logo.path.as_ref(), "client-logo.png");
        assert_eq!(logo.bytes.len(), 4);
        assert_eq!(logo.media_type.as_ref(), "image/png");
    }

    /// AC-008 / BC-2.02.002 invariant: `BrandOverlay` has NO master-switching field.
    ///
    /// Exhaustive struct construction — any undeclared field would fail to compile.
    #[test]
    fn test_bc_2_02_002_invariant_brand_overlay_no_master_field() {
        // Exhaustive — if template_path, master_path, or layout_idx existed on
        // BrandOverlay, this would not compile (unknown field error).
        let overlay = BrandOverlay {
            logo: None,
            footer_text: None,
            confidentiality: None,
        };
        // Verify the struct compiles with exactly the 3 expected fields.
        assert!(overlay.logo.is_none());
    }

    /// AC-001: `BrandOverlay` implements `Hash + Clone + Eq + Debug`.
    #[test]
    fn test_bc_2_02_001_brand_overlay_hash_clone() {
        use std::collections::HashMap;
        let overlay = BrandOverlay {
            logo: Some(LogoOverride {
                path: Arc::from("logo.png"),
                bytes: vec![1, 2, 3],
                media_type: Arc::from("image/png"),
            }),
            footer_text: None,
            confidentiality: None,
        };
        let overlay2 = overlay.clone();
        assert_eq!(overlay, overlay2);
        let mut map: HashMap<BrandOverlay, u32> = HashMap::new();
        map.insert(overlay.clone(), 99);
        assert_eq!(map[&overlay2], 99);
    }

    // ── AC-006 / EC-004: empty overlay returns Ok(None) ──────────────────────

    /// AC-006 / BC-2.02.001 EC-004: `resolve_overlay` with all-None overlay
    /// returns `Ok(None)` (no overlay to apply).
    #[test]
    fn test_bc_2_02_001_resolve_overlay_empty_returns_none() {
        let overlay = empty_overlay();
        let dir = tempdir().expect("tempdir");
        let result = resolve_overlay(&overlay, dir.path());
        assert!(result.is_ok(), "empty overlay must not error: {result:?}");
        assert!(
            result.expect("checked is_ok above").is_none(),
            "empty overlay must return None"
        );
    }

    // ── AC-005 / EC-001: missing logo → BrandError::FileNotFound ─────────────

    /// AC-005 / BC-2.02.001 EC-001: a `brand_overlay:` with a nonexistent logo
    /// path produces `BrandError::FileNotFound` (E-BRD-001, fatal, exit 4).
    #[test]
    fn test_bc_2_02_001_resolve_overlay_missing_logo_file_not_found() {
        let overlay = SlideOverlay {
            logo_path: Some(Arc::from("absolutely-does-not-exist-7f3a.png")),
            footer_text: None,
            confidentiality: None,
            span: SourceSpan::default(),
        };
        let dir = tempdir().expect("tempdir");
        let result = resolve_overlay(&overlay, dir.path());
        match result {
            Err(BrandError::FileNotFound { path, .. }) => {
                assert!(
                    path.contains("absolutely-does-not-exist-7f3a.png"),
                    "FileNotFound path must contain the logo path, got: {path}"
                );
            },
            other => panic!("expected BrandError::FileNotFound, got: {other:?}"),
        }
    }

    // ── AC-002 / BC-2.02.001 postcondition 1: logo bytes loaded from path ─────

    /// AC-002 / BC-2.02.001 postcondition 1: `resolve_overlay` with a valid logo
    /// path loads the bytes and returns `BrandOverlay.logo = Some(LogoOverride { bytes })`.
    #[test]
    fn test_bc_2_02_001_resolve_overlay_logo_path_loads_bytes() {
        let dir = tempdir().expect("tempdir");
        let logo_path = dir.path().join("test-logo.png");
        let png_bytes = b"\x89PNG\r\n\x1a\n"; // minimal PNG header
        std::fs::write(&logo_path, png_bytes).expect("write png");

        let file_name = logo_path
            .file_name()
            .expect("path has a file name")
            .to_str()
            .expect("file name is valid UTF-8");
        let overlay = SlideOverlay {
            logo_path: Some(Arc::from(file_name)),
            footer_text: None,
            confidentiality: None,
            span: SourceSpan::default(),
        };
        let result = resolve_overlay(&overlay, dir.path());
        let brand_overlay = result
            .expect("no error")
            .expect("Some overlay — logo path was set");
        let logo = brand_overlay.logo.expect("logo must be present");
        assert_eq!(
            logo.bytes.as_slice(),
            png_bytes.as_slice(),
            "logo bytes must match file content"
        );
        assert_eq!(
            logo.media_type.as_ref(),
            "image/png",
            "media type must be image/png for .png extension"
        );
    }

    // ── AC-003 / BC-2.02.001 postcondition 2: footer_text semantics ───────────

    /// AC-003 / BC-2.02.001 postcondition 2: `footer_text: Some("")` in
    /// `SlideOverlay` produces `BrandOverlay.footer_text = Some("")` — NOT `None`.
    ///
    /// `Some("")` clears footer text content; `None` means no change. These must be
    /// distinct values (BC-2.02.001 invariant 3).
    #[test]
    fn test_bc_2_02_001_resolve_overlay_footer_text_some_empty_distinct_from_none() {
        let overlay = SlideOverlay {
            logo_path: None,
            footer_text: Some(Arc::from("")), // explicitly empty — clears footer text
            confidentiality: None,
            span: SourceSpan::default(),
        };
        let dir = tempdir().expect("tempdir");
        let result = resolve_overlay(&overlay, dir.path());
        let brand_overlay = result
            .expect("no error")
            .expect("Some overlay — Some('') is not empty; it has explicit intent");
        assert_eq!(
            brand_overlay.footer_text,
            Some(Arc::from("")),
            "footer_text Some('') must be preserved as Some(''), not converted to None"
        );
    }

    /// AC-003 / BC-2.02.001 postcondition 2: `footer_text: None` in `SlideOverlay`
    /// produces an empty overlay (all-None → `Ok(None)`).
    #[test]
    fn test_bc_2_02_001_resolve_overlay_footer_text_none() {
        let overlay = SlideOverlay {
            logo_path: None,
            footer_text: None,
            confidentiality: None,
            span: SourceSpan::default(),
        };
        let dir = tempdir().expect("tempdir");
        let result = resolve_overlay(&overlay, dir.path());
        // All fields None → Ok(None) (no-op overlay).
        assert!(
            result.is_ok(),
            "all-None overlay must not error: {result:?}"
        );
        assert!(
            result.expect("checked is_ok above").is_none(),
            "all-None overlay must return Ok(None)"
        );
    }

    // ── AC-004 / BC-2.02.001 postcondition 3: confidentiality ────────────────

    /// AC-004 / BC-2.02.001 postcondition 3: confidentiality text is passed
    /// through to `BrandOverlay.confidentiality`.
    #[test]
    fn test_bc_2_02_001_resolve_overlay_confidentiality_passthrough() {
        let overlay = SlideOverlay {
            logo_path: None,
            footer_text: None,
            confidentiality: Some(Arc::from("CONFIDENTIAL — DO NOT DISTRIBUTE")),
            span: SourceSpan::default(),
        };
        let dir = tempdir().expect("tempdir");
        let result = resolve_overlay(&overlay, dir.path());
        let brand_overlay = result
            .expect("no error")
            .expect("Some overlay — confidentiality was set");
        assert_eq!(
            brand_overlay.confidentiality.as_deref(),
            Some("CONFIDENTIAL — DO NOT DISTRIBUTE"),
            "confidentiality text must pass through unchanged"
        );
    }

    // ── AC-010 / BC-2.02.002 EC-002: all slides have overlays ────────────────

    /// AC-010 / BC-2.02.002 EC-002: `resolve_overlay` is called N times for an
    /// N-slide deck (e.g., 10 slides all with overlays). The function is pure
    /// per-slide — no shared state. Each call returns independently with the
    /// correct per-slide footer text.
    #[test]
    fn test_bc_2_02_002_all_slides_have_overlays_resolve_independently() {
        let dir = tempdir().expect("tempdir");
        let overlays: Vec<SlideOverlay> = (0..10)
            .map(|i| SlideOverlay {
                logo_path: None,
                footer_text: Some(Arc::from(format!("Slide {i} Footer").as_str())),
                confidentiality: None,
                span: SourceSpan::default(),
            })
            .collect();

        // Each call to resolve_overlay is independent — no shared mutable state.
        for (i, overlay) in overlays.iter().enumerate() {
            let result = resolve_overlay(overlay, dir.path());
            let brand_overlay = result
                .unwrap_or_else(|e| panic!("slide {i} overlay must not error: {e}"))
                .unwrap_or_else(|| panic!("slide {i} must return Some overlay"));
            assert_eq!(
                brand_overlay.footer_text.as_deref(),
                Some(format!("Slide {i} Footer").as_str()),
                "slide {i} footer text must match"
            );
        }
    }

    // ── BC-2.02.002 structural: BrandOverlay has no master field ─────────────

    /// BC-2.02.002 invariant 2: No API, flag, or field in `BrandOverlay` allows
    /// a second slide master. This test validates the struct is exhaustively
    /// constructible with exactly 3 fields (logo, `footer_text`, confidentiality).
    #[test]
    fn test_bc_2_02_002_invariant_no_second_master_api() {
        // Exhaustive struct construction.
        // If template_path, master_path, or layout_idx were added, this would fail.
        let overlay = BrandOverlay {
            logo: None,
            footer_text: None,
            confidentiality: None,
        };
        // Success = proof of structural absence of master-switch API.
        assert!(overlay.footer_text.is_none());
    }

    // ── infer_media_type ─────────────────────────────────────────────────────

    /// `infer_media_type` returns `"image/png"` for `.png` extension.
    #[test]
    fn test_bc_2_02_001_infer_media_type_png() {
        let path = std::path::Path::new("logo.png");
        let result = infer_media_type(path);
        assert_eq!(result.as_ref(), "image/png");
    }

    /// `infer_media_type` returns `"image/jpeg"` for `.jpg` extension.
    #[test]
    fn test_bc_2_02_001_infer_media_type_jpg() {
        let path = std::path::Path::new("logo.jpg");
        let result = infer_media_type(path);
        assert_eq!(result.as_ref(), "image/jpeg");
    }
}
