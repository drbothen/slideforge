//! [`HtmlExporter`] — implements the [`Exporter`] plugin trait for static HTML
//! output (BC-4.03.003, STORY-046).
//!
//! The exporter converts a [`LaidOutDeck`] + [`Deck`] + [`Brand`] triple into a
//! single UTF-8 HTML5 document (or a directory of per-slide HTML files).
//! Output passes WCAG AA validation via `@axe-core/playwright` in CI.
//!
//! ## Security note (AC-010 / CWE-601)
//!
//! All `Link` and `Xref` inline nodes are validated against the URL scheme
//! allowlist before rendering to `<a href="...">`. Disallowed schemes are
//! dropped; a `tracing::warn!` is emitted for each rejection.

use slideforge_layout::LaidOutDeck;
use slideforge_plugin_api::{ExportError, ExportOptions, Exporter};
use slideforge_types::{Brand, Deck};

/// Allowlist of URL schemes permitted in rendered `<a href="...">` attributes.
///
/// All other schemes (e.g., `javascript:`, `data:`, `vbscript:`, `blob:`)
/// are rejected and the `href` attribute is omitted. A `tracing::warn!` is
/// emitted for each rejected scheme. (AC-010 / CWE-601)
pub const ALLOWED_URL_SCHEMES: &[&str] = &["http", "https", "mailto", "tel"];

/// Returns `true` if the given URL has an allowed scheme per AC-010.
///
/// URLs with no scheme (relative references) are permitted by default.
/// URLs with disallowed schemes are rejected.
///
/// # Examples
///
/// ```rust
/// use slideforge_html::exporter::is_safe_link_scheme;
///
/// assert!(is_safe_link_scheme("https://example.com"));
/// assert!(is_safe_link_scheme("mailto:user@example.com"));
/// assert!(is_safe_link_scheme("/relative/path"));
/// assert!(!is_safe_link_scheme("javascript:alert(1)"));
/// assert!(!is_safe_link_scheme("data:text/html,<h1>xss</h1>"));
/// assert!(!is_safe_link_scheme("vbscript:foo"));
/// ```
#[must_use]
pub fn is_safe_link_scheme(url: &str) -> bool {
    todo!("AC-010: validate URL scheme against ALLOWED_URL_SCHEMES allowlist; return true for relative URLs (no scheme), false + tracing::warn! for disallowed schemes")
}

/// Static HTML exporter — implements the [`Exporter`] plugin trait.
///
/// `HtmlExporter` converts a [`LaidOutDeck`] into an HTML5 document that passes
/// WCAG AA validation via `@axe-core/playwright` in CI. SVG-based canvas
/// rendering (not `<canvas>`), correct ARIA landmarks, and non-hardcoded
/// `<html lang>` are enforced by this exporter (BC-4.03.003).
///
/// Register via `PluginRegistry::register_exporter(Box::new(HtmlExporter::new()))`.
#[derive(Debug, Default)]
pub struct HtmlExporter;

impl HtmlExporter {
    /// Creates a new `HtmlExporter` instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Exporter for HtmlExporter {
    /// Returns the unique plugin identifier: `"html"`.
    fn id(&self) -> &str {
        "html"
    }

    /// Returns the default file extension for HTML output: `"html"`.
    fn extension(&self) -> &str {
        "html"
    }

    /// Produce an HTML5 document from the slideforge IR.
    ///
    /// # Output format
    ///
    /// The output is a well-formed UTF-8 HTML5 document beginning with
    /// `<!DOCTYPE html>`. Each slide is rendered as an `<article>` landmark
    /// containing an `<svg>` canvas element. No `<canvas>` elements are emitted.
    ///
    /// # Accessibility invariants (BC-4.03.003)
    ///
    /// - `<html lang="...">` is always set from `deck.lang` (never hardcoded).
    /// - Non-decorative SVG elements have `role="img"` + `<title>alt text</title>`.
    /// - Decorative images have `alt="" role="presentation"`.
    /// - Heading hierarchy is correct and non-skipped (h1 → h2 → ...).
    ///
    /// # Security (AC-010 / CWE-601)
    ///
    /// All `Link`/`Xref` URLs are validated via [`is_safe_link_scheme`] before
    /// being emitted as `href` attributes.
    ///
    /// # Errors
    ///
    /// Returns [`ExportError`] when the HTML document cannot be produced.
    fn export(
        &self,
        _deck: &Deck,
        _laid_out: &LaidOutDeck,
        _brand: &Brand,
        _opts: &ExportOptions,
    ) -> Result<Vec<u8>, ExportError> {
        todo!("AC-001: implement HtmlExporter::export — render deck to HTML5 via minijinja templates; return UTF-8 bytes")
    }
}
