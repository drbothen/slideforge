//! Per-slide brand overlay — thin metadata type for the `Slide` IR.
//!
//! A [`SlideOverlay`] is stored on [`crate::slide::Slide`] when the DSL
//! declares a `brand_overlay:` block on that slide. It carries only path and
//! text fields (no loaded bytes), keeping `slideforge-types` as a pure leaf
//! crate with no I/O dependencies.
//!
//! The `slideforge-brand` crate interprets a `SlideOverlay` into a fully-loaded
//! `BrandOverlay` (with logo bytes) at export time via `resolve_overlay()`.
//!
//! ## Invariants (BC-2.02.001 + BC-2.02.002)
//!
//! - `SlideOverlay` has NO `master_path`, `template_path`, or `layout_idx` field.
//!   This is the compile-time enforcement of the single-master invariant (DI-016).
//!   It is structurally impossible to represent a master switch via `SlideOverlay`.
//!
//! - `footer_text: None` means "no change to footer" (deck-level brand used).
//!   `footer_text: Some("")` means "clear the footer text content" (placeholder
//!   structure remains; only the text content is cleared). This distinction is
//!   critical — see BC-2.02.001 invariant 3 and AC-003.
//!
//! - `logo_path`, `footer_text`, and `confidentiality` are all `Option`. An
//!   all-`None` `SlideOverlay` is a no-op (produced when the parser emits a
//!   warning for an empty `brand_overlay:` block — BC-2.02.001 edge case EC-004).

use std::sync::Arc;

use crate::span::SourceSpan;

/// Per-slide brand overlay metadata, stored on [`crate::slide::Slide`].
///
/// Produced by the DSL parser (STORY-008/009) when it encounters a
/// `brand_overlay:` block. Consumed by `slideforge-brand::overlay::resolve_overlay`
/// at export time to load logo bytes and construct a `BrandOverlay`.
///
/// ## No Master Switch
///
/// This struct deliberately contains NO field for switching the slide master or
/// layout template. The absence of such a field is the compile-time guarantee of
/// the single-master invariant (BC-2.02.002, DI-016). Any DSL syntax attempting
/// `brand_overlay: template "other.pptx"` is rejected by the parser before it
/// can produce a `SlideOverlay`.
///
/// ## `Some("")` vs `None` for `footer_text`
///
/// - `None` — footer field absent; deck-level brand footer text is used unchanged.
/// - `Some("")` — footer field explicitly set to empty string; clears the footer
///   placeholder text content (placeholder XML structure is preserved).
/// - `Some("CONFIDENTIAL")` — replaces footer placeholder text on this slide only.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlideOverlay {
    /// Filesystem path to the override logo image file.
    ///
    /// - `None` — no logo override; deck-level brand logo is used.
    /// - `Some(path)` — path to logo file. Resolved relative to the `.sf` source
    ///   file directory. `slideforge-brand` validates existence at brand-apply time
    ///   (BC-2.02.001 edge case EC-001 → `BrandError::FileNotFound`).
    pub logo_path: Option<Arc<str>>,

    /// Override text for the footer placeholder on this slide.
    ///
    /// - `None` — no change; deck-level brand footer text is used.
    /// - `Some("")` — explicitly clears the footer text content (placeholder
    ///   structure preserved; see BC-2.02.001 invariant 3).
    /// - `Some(text)` — replaces footer placeholder text on this slide only.
    pub footer_text: Option<Arc<str>>,

    /// Confidentiality label text for this slide.
    ///
    /// - `None` — no confidentiality label shape added to this slide.
    /// - `Some(text)` — adds or updates a confidentiality label shape at the
    ///   bottom-right of the slide (position applied by PPTX exporter in STORY-037).
    pub confidentiality: Option<Arc<str>>,

    /// Source location of the `brand_overlay:` block in the `.sf` file.
    ///
    /// Used for error messages (e.g., `BrandError::FileNotFound` with span).
    pub span: SourceSpan,
}

impl SlideOverlay {
    /// Returns `true` if all overlay fields are `None` (this is a no-op overlay).
    ///
    /// An all-`None` overlay is produced when the parser encounters an empty
    /// `brand_overlay:` block and emits a parse warning (BC-2.02.001 EC-004).
    /// `resolve_overlay()` in `slideforge-brand` returns `Ok(None)` for these.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.logo_path.is_none() && self.footer_text.is_none() && self.confidentiality.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn make_empty_overlay() -> SlideOverlay {
        SlideOverlay {
            logo_path: None,
            footer_text: None,
            confidentiality: None,
            span: SourceSpan::default(),
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // AC-001 / BC-2.02.001 precondition 2: SlideOverlay struct fields
    // ──────────────────────────────────────────────────────────────────────────

    /// AC-001: `SlideOverlay` has the required fields with correct types.
    #[test]
    fn test_bc_2_02_001_slide_overlay_fields_present() {
        let overlay = SlideOverlay {
            logo_path: Some(Arc::from("client-logo.png")),
            footer_text: Some(Arc::from("CONFIDENTIAL")),
            confidentiality: Some(Arc::from("DO NOT DISTRIBUTE")),
            span: SourceSpan::default(),
        };
        assert_eq!(overlay.logo_path.as_deref(), Some("client-logo.png"));
        assert_eq!(overlay.footer_text.as_deref(), Some("CONFIDENTIAL"));
        assert_eq!(overlay.confidentiality.as_deref(), Some("DO NOT DISTRIBUTE"));
    }

    /// AC-008 / BC-2.02.002 invariant: `SlideOverlay` has NO master-switching field.
    ///
    /// This test is a compile-time proof: if a `master_path` or `template_path`
    /// field existed on `SlideOverlay`, this test would not compile (struct
    /// initialization would fail for unknown fields). The fact that it compiles
    /// with only the four declared fields proves the absence of master-switch
    /// capability at the type level.
    #[test]
    fn test_bc_2_02_002_invariant_no_master_path_field() {
        // Exhaustive struct construction — if any undeclared field existed, this
        // would fail to compile with "unknown field" error.
        let overlay = SlideOverlay {
            logo_path: None,
            footer_text: None,
            confidentiality: None,
            span: SourceSpan::default(),
        };
        // The struct compiles with exactly 4 fields (no template_path, no master_path,
        // no layout_idx). This is the structural single-master invariant enforcement.
        assert!(overlay.is_empty());
    }

    /// AC-006 / BC-2.02.001 EC-004: empty overlay `is_empty()` returns true.
    #[test]
    fn test_bc_2_02_001_empty_overlay_is_empty() {
        let overlay = make_empty_overlay();
        assert!(overlay.is_empty(), "all-None overlay must be empty");
    }

    /// BC-2.02.001 invariant 3: `footer_text: Some("")` vs `None` distinction.
    ///
    /// `Some("")` means "clear footer text" — distinct from `None` which means
    /// "no change". This test verifies the type system preserves the distinction.
    #[test]
    fn test_bc_2_02_001_invariant_footer_some_empty_distinct_from_none() {
        let with_none = SlideOverlay {
            logo_path: None,
            footer_text: None,
            confidentiality: None,
            span: SourceSpan::default(),
        };
        let with_empty = SlideOverlay {
            logo_path: None,
            footer_text: Some(Arc::from("")),
            confidentiality: None,
            span: SourceSpan::default(),
        };
        // None and Some("") must be distinct values — different behavior at export time.
        assert_ne!(with_none.footer_text, with_empty.footer_text);
        // Some("") is NOT empty — it has an explicit intent (clear the footer).
        assert!(!with_empty.is_empty());
        // None IS empty for footer specifically.
        assert!(with_none.is_empty());
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Hash + Eq + Clone + Debug (comemo compatibility)
    // ──────────────────────────────────────────────────────────────────────────

    /// AC-011: `SlideOverlay` implements `Hash + Clone` correctly.
    #[test]
    fn test_bc_2_02_001_slide_overlay_hash_clone() {
        use std::collections::HashMap;
        let overlay = SlideOverlay {
            logo_path: Some(Arc::from("logo.png")),
            footer_text: Some(Arc::from("Footer")),
            confidentiality: None,
            span: SourceSpan::default(),
        };
        let overlay2 = overlay.clone();
        assert_eq!(overlay, overlay2);
        let mut map: HashMap<SlideOverlay, u32> = HashMap::new();
        map.insert(overlay.clone(), 1);
        assert_eq!(map[&overlay2], 1);
    }

    /// AC-011: `SlideOverlay` implements `Debug`.
    #[test]
    fn test_bc_2_02_001_slide_overlay_debug() {
        let overlay = make_empty_overlay();
        let s = format!("{overlay:?}");
        assert!(s.contains("SlideOverlay"));
    }

    /// AC-011: Two different `SlideOverlay` instances with different fields are not equal.
    #[test]
    fn test_bc_2_02_001_slide_overlay_eq_ne() {
        let a = SlideOverlay {
            logo_path: Some(Arc::from("logo-a.png")),
            footer_text: None,
            confidentiality: None,
            span: SourceSpan::default(),
        };
        let b = SlideOverlay {
            logo_path: Some(Arc::from("logo-b.png")),
            footer_text: None,
            confidentiality: None,
            span: SourceSpan::default(),
        };
        assert_ne!(a, b);
    }
}
