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
//! ## `resolve_overlay` stub (STORY-025 Red Gate)
//!
//! The `resolve_overlay` function is bodied as `todo!()`. Tests that call it MUST
//! panic. The implementer (STORY-025 implementation phase) will replace the body.

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

// ─── resolve_overlay (STUB — Red Gate) ───────────────────────────────────────

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
///
/// # Red Gate (STORY-025)
///
/// This function body is `todo!()`. Every test that calls it will panic with
/// `"BC-2.02.001/002 — STORY-025 — Red Gate: brand overlay not yet implemented"`.
/// The implementer must replace this body with production logic.
pub fn resolve_overlay(
    raw: &SlideOverlay,
    root_dir: &Path,
) -> Result<Option<BrandOverlay>, BrandError> {
    let _ = (raw, root_dir);
    todo!("BC-2.02.001/002 — STORY-025 — Red Gate: brand overlay not yet implemented")
}

// ─── infer_media_type (STUB — Red Gate) ──────────────────────────────────────

/// Infer the MIME type of a logo image from its file extension.
///
/// Supports `.png` → `"image/png"`, `.jpg`/`.jpeg` → `"image/jpeg"`,
/// `.gif` → `"image/gif"`, `.svg` → `"image/svg+xml"`. Unknown extensions
/// return `"application/octet-stream"`.
///
/// # Red Gate (STORY-025)
///
/// This function body is `todo!()`. The implementer must replace this body.
#[must_use]
pub fn infer_media_type(path: &Path) -> Arc<str> {
    let _ = path;
    todo!("BC-2.02.001/002 — STORY-025 — Red Gate: media type inference not yet implemented")
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

    /// AC-001: `BrandOverlay` has the correct fields (logo, footer_text, confidentiality).
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
        assert_eq!(overlay.confidentiality.as_deref(), Some("DO NOT DISTRIBUTE"));
    }

    /// AC-001: `LogoOverride` has path, bytes, and media_type fields.
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
        let _overlay = BrandOverlay {
            logo: None,
            footer_text: None,
            confidentiality: None,
        };
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
    ///
    /// RED GATE: This test will panic with todo!() until the stub is implemented.
    #[test]
    #[should_panic(expected = "Red Gate")]
    fn test_bc_2_02_001_resolve_overlay_empty_returns_none() {
        let overlay = empty_overlay();
        let dir = tempdir().expect("tempdir");
        let result = resolve_overlay(&overlay, dir.path());
        // After implementation this assert replaces the panic check:
        // assert!(result.is_ok());
        // assert!(result.unwrap().is_none());
        let _ = result;
    }

    // ── AC-005 / EC-001: missing logo → BrandError::FileNotFound ─────────────

    /// AC-005 / BC-2.02.001 EC-001: a `brand_overlay:` with a nonexistent logo
    /// path produces `BrandError::FileNotFound` (E-BRD-001, fatal, exit 4).
    ///
    /// RED GATE: This test will panic with todo!() until the stub is implemented.
    #[test]
    #[should_panic(expected = "Red Gate")]
    fn test_bc_2_02_001_resolve_overlay_missing_logo_file_not_found() {
        let overlay = SlideOverlay {
            logo_path: Some(Arc::from("absolutely-does-not-exist-7f3a.png")),
            footer_text: None,
            confidentiality: None,
            span: SourceSpan::default(),
        };
        let dir = tempdir().expect("tempdir");
        let result = resolve_overlay(&overlay, dir.path());
        // After implementation this replaces the panic check:
        // match result {
        //     Err(BrandError::FileNotFound { path, .. }) => {
        //         assert!(path.contains("absolutely-does-not-exist-7f3a.png"));
        //     }
        //     _ => panic!("expected FileNotFound"),
        // }
        let _ = result;
    }

    // ── AC-002 / BC-2.02.001 postcondition 1: logo bytes loaded from path ─────

    /// AC-002 / BC-2.02.001 postcondition 1: `resolve_overlay` with a valid logo
    /// path loads the bytes and returns `BrandOverlay.logo = Some(LogoOverride { bytes })`.
    ///
    /// RED GATE: This test will panic with todo!() until the stub is implemented.
    #[test]
    #[should_panic(expected = "Red Gate")]
    fn test_bc_2_02_001_resolve_overlay_logo_path_loads_bytes() {
        let dir = tempdir().expect("tempdir");
        let logo_path = dir.path().join("test-logo.png");
        let png_bytes = b"\x89PNG\r\n\x1a\n"; // minimal PNG header
        std::fs::write(&logo_path, png_bytes).expect("write png");

        let overlay = SlideOverlay {
            logo_path: Some(Arc::from(
                logo_path.file_name().unwrap().to_str().unwrap(),
            )),
            footer_text: None,
            confidentiality: None,
            span: SourceSpan::default(),
        };
        let result = resolve_overlay(&overlay, dir.path());
        // After implementation:
        // let brand_overlay = result.expect("no error").expect("Some overlay");
        // let logo = brand_overlay.logo.expect("logo present");
        // assert_eq!(logo.bytes, png_bytes);
        // assert_eq!(logo.media_type.as_ref(), "image/png");
        let _ = result;
    }

    // ── AC-003 / BC-2.02.001 postcondition 2: footer_text semantics ───────────

    /// AC-003 / BC-2.02.001 postcondition 2: `footer_text: Some("")` in
    /// `SlideOverlay` produces `BrandOverlay.footer_text = Some("")` — NOT `None`.
    ///
    /// RED GATE: This test will panic with todo!() until the stub is implemented.
    #[test]
    #[should_panic(expected = "Red Gate")]
    fn test_bc_2_02_001_resolve_overlay_footer_text_some_empty_distinct_from_none() {
        let overlay = SlideOverlay {
            logo_path: None,
            footer_text: Some(Arc::from("")), // explicitly empty — clears footer text
            confidentiality: None,
            span: SourceSpan::default(),
        };
        let dir = tempdir().expect("tempdir");
        let result = resolve_overlay(&overlay, dir.path());
        // After implementation:
        // let brand_overlay = result.expect("no error").expect("Some overlay — Some('') is not empty");
        // assert_eq!(brand_overlay.footer_text, Some(Arc::from("")));
        let _ = result;
    }

    /// AC-003 / BC-2.02.001 postcondition 2: `footer_text: None` in `SlideOverlay`
    /// produces `BrandOverlay.footer_text = None` — no change to footer.
    ///
    /// RED GATE: This test will panic with todo!() until the stub is implemented.
    #[test]
    #[should_panic(expected = "Red Gate")]
    fn test_bc_2_02_001_resolve_overlay_footer_text_none() {
        let overlay = SlideOverlay {
            logo_path: None,
            footer_text: None,
            confidentiality: None,
            span: SourceSpan::default(),
        };
        let dir = tempdir().expect("tempdir");
        let result = resolve_overlay(&overlay, dir.path());
        // After implementation: result = Ok(None) since all fields are None.
        let _ = result;
    }

    // ── AC-004 / BC-2.02.001 postcondition 3: confidentiality ────────────────

    /// AC-004 / BC-2.02.001 postcondition 3: confidentiality text is passed
    /// through to `BrandOverlay.confidentiality`.
    ///
    /// RED GATE: This test will panic with todo!() until the stub is implemented.
    #[test]
    #[should_panic(expected = "Red Gate")]
    fn test_bc_2_02_001_resolve_overlay_confidentiality_passthrough() {
        let overlay = SlideOverlay {
            logo_path: None,
            footer_text: None,
            confidentiality: Some(Arc::from("CONFIDENTIAL — DO NOT DISTRIBUTE")),
            span: SourceSpan::default(),
        };
        let dir = tempdir().expect("tempdir");
        let result = resolve_overlay(&overlay, dir.path());
        // After implementation:
        // let brand_overlay = result.expect("no error").expect("Some overlay");
        // assert_eq!(brand_overlay.confidentiality.as_deref(), Some("CONFIDENTIAL — DO NOT DISTRIBUTE"));
        let _ = result;
    }

    // ── AC-010 / BC-2.02.002 EC-002: all slides have overlays ────────────────

    /// AC-010 / BC-2.02.002 EC-002: `resolve_overlay` is called N times for an
    /// N-slide deck (e.g., 10 slides all with overlays). The function is pure
    /// per-slide — no shared state. Each call returns independently.
    ///
    /// RED GATE: This test will panic with todo!() until the stub is implemented.
    #[test]
    #[should_panic(expected = "Red Gate")]
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
        // After implementation: all should return Ok(Some(...)) with per-slide footer.
        for overlay in &overlays {
            let result = resolve_overlay(overlay, dir.path());
            let _ = result;
        }
    }

    // ── BC-2.02.002 structural: BrandOverlay has no master field ─────────────

    /// BC-2.02.002 invariant 2: No API, flag, or field in `BrandOverlay` allows
    /// a second slide master. This test validates the struct is exhaustively
    /// constructible with exactly 3 fields (logo, footer_text, confidentiality).
    #[test]
    fn test_bc_2_02_002_invariant_no_second_master_api() {
        // Exhaustive struct construction.
        // If template_path, master_path, or layout_idx were added, this would fail.
        let _overlay = BrandOverlay {
            logo: None,
            footer_text: None,
            confidentiality: None,
        };
        // Success = proof of structural absence of master-switch API.
    }

    // ── infer_media_type stub ─────────────────────────────────────────────────

    /// `infer_media_type` returns `"image/png"` for `.png` extension.
    ///
    /// RED GATE: This test will panic with todo!() until the stub is implemented.
    #[test]
    #[should_panic(expected = "Red Gate")]
    fn test_bc_2_02_001_infer_media_type_png() {
        let path = std::path::Path::new("logo.png");
        let result = infer_media_type(path);
        // After implementation: assert_eq!(result.as_ref(), "image/png");
        let _ = result;
    }

    /// `infer_media_type` returns `"image/jpeg"` for `.jpg` extension.
    ///
    /// RED GATE: This test will panic with todo!() until the stub is implemented.
    #[test]
    #[should_panic(expected = "Red Gate")]
    fn test_bc_2_02_001_infer_media_type_jpg() {
        let path = std::path::Path::new("logo.jpg");
        let result = infer_media_type(path);
        // After implementation: assert_eq!(result.as_ref(), "image/jpeg");
        let _ = result;
    }
}
