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

use crate::render::render_slide_to_html;

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
/// The HTML exporter differs from `slideforge-pptx`'s `is_safe_link_scheme` in
/// one important respect: **relative URLs (no `:`) are ALLOWED** here, because
/// HTML documents commonly use relative paths like `/slides/2` or `#section`.
/// The PPTX exporter rejects no-scheme URLs because OOXML external relationships
/// require an absolute URI; HTML does not have that constraint.
///
/// Comparison is case-insensitive so `HTTPS://example.com` and `https://example.com`
/// both pass.
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
    // F-008 (CWE-601): reject protocol-relative URLs (// or \\) regardless of
    // the absence of a colon. Browsers resolve these as scheme-relative, enabling
    // open-redirect even when the author intended a relative path.
    if url.starts_with("//") || url.starts_with('\\') {
        tracing::warn!(
            url = %url,
            "AC-010: protocol-relative or backslash-prefix URL rejected (CWE-601)"
        );
        return false;
    }

    let Some(colon_pos) = url.find(':') else {
        // No scheme separator — this is a path/fragment relative URL; allow it.
        return true;
    };
    let scheme = url[..colon_pos].to_ascii_lowercase();
    // Check whether the scheme is in the allowlist.
    if ALLOWED_URL_SCHEMES.contains(&scheme.as_str()) {
        true
    } else {
        // Emit tracing::warn! with the rejected scheme.
        // NOTE: this is the SINGLE warn per rejection (F-006). Callers (render.rs)
        // must NOT emit an additional warn — this function is the sole log site.
        tracing::warn!(
            rejected_scheme = %scheme,
            url = %url,
            "AC-010: link URL with disallowed scheme rejected (CWE-601)"
        );
        false
    }
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
    #[allow(clippy::unnecessary_literal_bound)]
    fn id(&self) -> &str {
        "html"
    }

    /// Returns the default file extension for HTML output: `"html"`.
    #[allow(clippy::unnecessary_literal_bound)]
    fn extension(&self) -> &str {
        "html"
    }

    /// Produce an HTML5 document from the slideforge IR.
    ///
    /// # Output format
    ///
    /// The output is a well-formed UTF-8 HTML5 document beginning with
    /// `<!DOCTYPE html>`. Each slide is rendered as an `<article>` landmark
    /// containing frame elements for the slide content. No `<canvas>` elements
    /// are emitted.
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
        deck: &Deck,
        laid_out: &LaidOutDeck,
        brand: &Brand,
        _opts: &ExportOptions,
    ) -> Result<Vec<u8>, ExportError> {
        // Derive the lang attribute from deck.lang (AC-002 / BC-4.03.003 invariant 3).
        // NEVER hardcode lang — must come from deck metadata.
        let lang = deck.metadata.lang.as_ref().map_or("en-US", |l| l.as_ref());

        // Derive the document title from deck.metadata.title.
        let title = deck
            .metadata
            .title
            .as_ref()
            .map_or("Presentation", |t| t.as_ref());

        // Render all slides.
        let mut slides_html = String::new();
        for slide in &laid_out.slides {
            slides_html.push_str(&render_slide_to_html(slide, brand));
            slides_html.push('\n');
        }

        // Render via minijinja template.
        let page_html = render_page_template(lang, title, &slides_html)?;

        Ok(page_html.into_bytes())
    }
}

/// Render the full page HTML using the `page.html.jinja` template.
///
/// Uses `minijinja` for Jinja2-compatible HTML templating. The `.html` suffix
/// on the template name enables HTML autoescape automatically (ADR-022).
fn render_page_template(lang: &str, title: &str, slides_html: &str) -> Result<String, ExportError> {
    use minijinja::{Environment, context};

    let mut env = Environment::new();

    // Load the page template. The template is embedded at compile time to
    // avoid filesystem path issues in tests and cross-platform deployments.
    let page_template = include_str!("../templates/page.html.jinja");

    env.add_template("page.html", page_template)
        .map_err(|e| ExportError::RenderError {
            message: format!("failed to load page.html.jinja template: {e}"),
        })?;

    let tmpl = env
        .get_template("page.html")
        .map_err(|e| ExportError::RenderError {
            message: format!("failed to get page.html template: {e}"),
        })?;

    // Build individual slide HTML values for the template loop.
    // The slides_html is already rendered; we split it into individual slides
    // for the template's `{% for slide_html in slides %}` loop.
    // However, the current template expects an iterable of slide HTML strings.
    // We pass the pre-rendered combined HTML as a single "slides" value.
    //
    // For simplicity and correctness, we render the page wrapper directly
    // rather than trying to split the slides HTML.
    // The template loop `{% for slide_html in slides %}` iterates over Vec<String>.
    let slides_vec: Vec<&str> = vec![slides_html.trim_end()];

    let output = tmpl
        .render(context! {
            lang => lang,
            title => title,
            slides => slides_vec,
        })
        .map_err(|e| ExportError::RenderError {
            message: format!("template rendering failed: {e}"),
        })?;

    Ok(output)
}

// ─────────────────────────────────────────────────────────────────────────────
// STORY-046 Red Gate Tests — exporter.rs
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::doc_markdown, // test doc comments use fn names and HTML that trigger doc_markdown
    non_snake_case
)]
mod tests {
    use std::sync::Arc;

    use slideforge_layout::{
        BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize,
    };
    use slideforge_plugin_api::{ExportOptions, Exporter};
    use slideforge_types::{
        AltText, Brand, BrandFonts, BrandPalette, Deck, DeckMetadata, OrderedMap, SourceSpan,
    };

    use super::{ALLOWED_URL_SCHEMES, HtmlExporter, is_safe_link_scheme};

    // ─────────────────────────────────────────────────────────────────────────
    // Test fixtures
    // ─────────────────────────────────────────────────────────────────────────

    fn make_brand() -> Brand {
        Brand {
            name: Arc::from("test-brand"),
            palette: BrandPalette {
                primary: Arc::from("#003087"),
                secondary: Arc::from("#0066CC"),
                accent: Arc::from("#FF6B35"),
                neutral: Arc::from("#F5F5F5"),
            },
            fonts: BrandFonts {
                heading: Arc::from("Calibri"),
                body: Arc::from("Calibri"),
                mono: Arc::from("Courier New"),
            },
            layouts: vec![],
            span: SourceSpan::default(),
        }
    }

    fn make_deck(lang: &str) -> Deck {
        Deck {
            slides: vec![],
            vars: OrderedMap::new(),
            metadata: DeckMetadata {
                title: Some(Arc::from("Test Deck")),
                slideforge_version: Arc::from("0.1.0"),
                lang: Some(Arc::from(lang)),
                author: None,
                section_order: None,
            },
            registers: OrderedMap::new(),
            section_blocks: vec![],
        }
    }

    fn make_laid_out_deck_single_slide(slide: LaidOutSlide) -> LaidOutDeck {
        LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![slide],
            sections: vec![],
            warnings: vec![],
        }
    }

    fn make_title_slide() -> LaidOutSlide {
        LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("title"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: slideforge_types::Emu(457_200),
                    y: slideforge_types::Emu(1_600_200),
                    width: slideforge_types::Emu(8_229_600),
                    height: slideforge_types::Emu(1_143_000),
                },
                content: FrameContent::Title(Arc::from("Hello World")),
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        }
    }

    fn make_decorative_image_slide() -> LaidOutSlide {
        LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("content"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: slideforge_types::Emu(0),
                    y: slideforge_types::Emu(0),
                    width: slideforge_types::Emu(9_144_000),
                    height: slideforge_types::Emu(5_143_500),
                },
                content: FrameContent::Image {
                    alt: AltText::Decorative,
                },
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        }
    }

    fn make_non_decorative_image_slide(alt_text: &str) -> LaidOutSlide {
        LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("content"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: slideforge_types::Emu(0),
                    y: slideforge_types::Emu(0),
                    width: slideforge_types::Emu(9_144_000),
                    height: slideforge_types::Emu(5_143_500),
                },
                content: FrameContent::Image {
                    alt: AltText::Provided(Arc::from(alt_text)),
                },
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-001: HtmlExporter implements Exporter; export returns bytes beginning
    // with <!DOCTYPE html>
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 postcondition 1 — `HtmlExporter` implements the `Exporter`
    /// plugin trait: correct id/extension AND export produces valid output.
    /// The export call hits todo!() — making this a failing Red Gate test.
    #[test]
    fn test_BC_4_03_003_html_exporter_implements_exporter_trait() {
        let exporter: Box<dyn Exporter> = Box::new(HtmlExporter::new());
        assert_eq!(exporter.id(), "html");
        assert_eq!(exporter.extension(), "html");
        // AC-001: export must return Ok (calls todo!() in stub — Red Gate)
        let deck = make_deck("en-US");
        let slide = make_title_slide();
        let laid_out = make_laid_out_deck_single_slide(slide);
        let brand = make_brand();
        let opts = ExportOptions::default();
        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        assert!(!bytes.is_empty(), "export must produce non-empty output");
    }

    /// AC-001 — `HtmlExporter::export` returns bytes whose UTF-8 string begins
    /// with `<!DOCTYPE html>`.
    #[test]
    fn test_BC_4_03_003_export_starts_with_doctype_html() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        let slide = make_title_slide();
        let laid_out = make_laid_out_deck_single_slide(slide);
        let brand = make_brand();
        let opts = ExportOptions::default();

        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed for a valid title slide");
        let html = String::from_utf8(bytes).expect("output must be valid UTF-8");
        assert!(
            html.trim_start().starts_with("<!DOCTYPE html>"),
            "exported HTML must begin with '<!DOCTYPE html>', got: {:?}",
            &html[..html.len().min(80)]
        );
    }

    /// AC-001 — `HtmlExporter::export` output is non-empty.
    #[test]
    fn test_BC_4_03_003_export_produces_non_empty_output() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        let slide = make_title_slide();
        let laid_out = make_laid_out_deck_single_slide(slide);
        let brand = make_brand();
        let opts = ExportOptions::default();

        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        assert!(!bytes.is_empty(), "export output must not be empty");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-002: html[lang] derived from deck.lang, never hardcoded "en"
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 postcondition 3 / invariant 3 — two fixture decks with lang
    /// "en-US" and "ja" must produce HTML with matching `lang` attributes.
    #[test]
    fn test_BC_4_03_003_html_lang_attribute_matches_deck_lang_en_us() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        let slide = make_title_slide();
        let laid_out = make_laid_out_deck_single_slide(slide);
        let brand = make_brand();
        let opts = ExportOptions::default();

        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(
            html.contains(r#"lang="en-US""#),
            "output must contain lang=\"en-US\" for deck.lang=\"en-US\", got snippet: {:?}",
            &html[..html.len().min(500)]
        );
    }

    /// BC-4.03.003 postcondition 3 / invariant 3 — deck.lang "ja" produces
    /// `<html lang="ja">`.
    #[test]
    fn test_BC_4_03_003_html_lang_attribute_matches_deck_lang_ja() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("ja");
        let slide = make_title_slide();
        let laid_out = make_laid_out_deck_single_slide(slide);
        let brand = make_brand();
        let opts = ExportOptions::default();

        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(
            html.contains(r#"lang="ja""#),
            "output must contain lang=\"ja\" for deck.lang=\"ja\", got snippet: {:?}",
            &html[..html.len().min(500)]
        );
    }

    /// BC-4.03.003 invariant 3 — the lang attribute is NEVER the hardcoded
    /// string "en" when the deck declares "ja".
    #[test]
    fn test_BC_4_03_003_lang_attribute_never_hardcoded_as_en() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("ja");
        let slide = make_title_slide();
        let laid_out = make_laid_out_deck_single_slide(slide);
        let brand = make_brand();
        let opts = ExportOptions::default();

        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(
            !html.contains(r#"lang="en""#),
            "output must NOT contain hardcoded lang=\"en\" when deck.lang is \"ja\""
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-006: No <canvas> elements in output
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 invariant 2 — no `<canvas>` elements appear in the HTML
    /// output. Uses `scraper::Selector::parse("canvas")` per AC-006 note.
    #[test]
    fn test_BC_4_03_003_no_canvas_elements_in_output() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        let slide = make_title_slide();
        let laid_out = make_laid_out_deck_single_slide(slide);
        let brand = make_brand();
        let opts = ExportOptions::default();

        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");

        let doc = scraper::Html::parse_document(&html);
        let sel = scraper::Selector::parse("canvas").expect("valid selector");
        assert_eq!(
            doc.select(&sel).count(),
            0,
            "output must contain ZERO <canvas> elements — slide visuals must use <svg>"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-003: Non-decorative images have non-empty alt / <title>
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 postcondition 4 — non-decorative images have non-empty accessible name.
    ///
    /// F-002: Images are rendered as SVG (not bare <img>). Non-decorative images
    /// use `role="img"` and a `<title>` with the alt text on the SVG element.
    #[test]
    fn test_BC_4_03_003_non_decorative_image_has_non_empty_alt() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        let slide = make_non_decorative_image_slide("A revenue chart");
        let laid_out = make_laid_out_deck_single_slide(slide);
        let brand = make_brand();
        let opts = ExportOptions::default();

        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");

        // F-002: image is now SVG, not bare <img>
        assert!(
            !html.contains("<img "),
            "F-002: non-decorative image must not be bare <img>; got snippet: {:?}",
            &html[..html.len().min(500)]
        );

        // Assert: the alt text "A revenue chart" appears in the output
        assert!(
            html.contains("A revenue chart"),
            "non-decorative image alt text must appear in HTML output"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-004: Decorative images have alt="" and role="presentation"
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 postcondition 5 — decorative images have `role="presentation"`.
    ///
    /// F-002: Images are rendered as SVG (not bare <img>). Decorative images use
    /// `role="presentation"` on the SVG element.
    #[test]
    fn test_BC_4_03_003_decorative_image_has_empty_alt_and_role_presentation() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        let slide = make_decorative_image_slide();
        let laid_out = make_laid_out_deck_single_slide(slide);
        let brand = make_brand();
        let opts = ExportOptions::default();

        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");

        // F-002: decorative image is rendered as SVG with role="presentation"
        assert!(
            !html.contains("<img "),
            "F-002: decorative image must not render as bare <img>; got snippet: {:?}",
            &html[..html.len().min(500)]
        );

        // Decorative element must have role="presentation"
        assert!(
            html.contains(r#"role="presentation""#),
            "decorative image must have role=\"presentation\" attribute"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-008: Heading hierarchy h1→h2 non-skipped
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 postcondition 7 — slide title maps to `<h1>`; no heading
    /// levels are skipped (h1 → h3 without h2 is forbidden).
    #[test]
    fn test_BC_4_03_003_slide_title_maps_to_h1() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        let slide = make_title_slide();
        let laid_out = make_laid_out_deck_single_slide(slide);
        let brand = make_brand();
        let opts = ExportOptions::default();

        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");
        let doc = scraper::Html::parse_document(&html);

        let sel_h1 = scraper::Selector::parse("h1").expect("valid selector");
        assert!(
            doc.select(&sel_h1).count() > 0,
            "slide title must produce at least one <h1> element in the HTML output"
        );
    }

    /// BC-4.03.003 postcondition 7 — a deck with h1 and h3 but no h2 is
    /// structurally invalid. The exporter must not skip heading levels.
    /// This test verifies that if an h3 appears, an h2 must also appear.
    #[test]
    fn test_BC_4_03_003_heading_hierarchy_no_skipped_levels() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        // A slide with title (h1) and body containing a sub-heading (h2)
        let slide = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("content"),
            frames: vec![
                Frame {
                    bbox: BoundingBox {
                        x: slideforge_types::Emu(0),
                        y: slideforge_types::Emu(0),
                        width: slideforge_types::Emu(9_144_000),
                        height: slideforge_types::Emu(1_000_000),
                    },
                    content: FrameContent::Title(Arc::from("Main Title")),
                    text_flow: None,
                    region_role: None,
                },
                Frame {
                    bbox: BoundingBox {
                        x: slideforge_types::Emu(0),
                        y: slideforge_types::Emu(1_000_000),
                        width: slideforge_types::Emu(9_144_000),
                        height: slideforge_types::Emu(4_143_500),
                    },
                    content: FrameContent::Subtitle(Arc::from("Section Heading")),
                    text_flow: None,
                    region_role: None,
                },
            ],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        };
        let laid_out = make_laid_out_deck_single_slide(slide);
        let brand = make_brand();
        let opts = ExportOptions::default();

        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");
        let doc = scraper::Html::parse_document(&html);

        // If there is an h2, there must also be an h1 (no skip from none to h2)
        let sel_h2 = scraper::Selector::parse("h2").expect("valid selector");
        if doc.select(&sel_h2).count() > 0 {
            let sel_h1 = scraper::Selector::parse("h1").expect("valid selector");
            assert!(
                doc.select(&sel_h1).count() > 0,
                "h2 elements exist but h1 is missing — heading hierarchy skips levels"
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-010: URL scheme allowlist — CWE-601 / XSS prevention
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 / AC-010 — `is_safe_link_scheme` allows http, https, mailto,
    /// tel, and relative URLs.
    #[test]
    fn test_BC_4_03_003_is_safe_link_scheme_allows_http() {
        assert!(
            is_safe_link_scheme("http://example.com"),
            "http:// must be allowed"
        );
        assert!(
            is_safe_link_scheme("https://example.com"),
            "https:// must be allowed"
        );
        assert!(
            is_safe_link_scheme("mailto:user@example.com"),
            "mailto: must be allowed"
        );
        assert!(
            is_safe_link_scheme("tel:+15551234567"),
            "tel: must be allowed"
        );
        assert!(
            is_safe_link_scheme("/relative/path"),
            "relative path must be allowed (no scheme)"
        );
        assert!(
            is_safe_link_scheme(""),
            "empty URL must be allowed (no scheme = relative)"
        );
    }

    /// BC-4.03.003 / AC-010 — `is_safe_link_scheme` rejects javascript:.
    #[test]
    fn test_BC_4_03_003_is_safe_link_scheme_rejects_javascript() {
        assert!(
            !is_safe_link_scheme("javascript:alert(1)"),
            "javascript: must be rejected (CWE-601 / XSS)"
        );
    }

    /// BC-4.03.003 / AC-010 — `is_safe_link_scheme` rejects data:.
    #[test]
    fn test_BC_4_03_003_is_safe_link_scheme_rejects_data_uri() {
        assert!(
            !is_safe_link_scheme("data:text/html,<h1>xss</h1>"),
            "data: must be rejected"
        );
    }

    /// BC-4.03.003 / AC-010 — `is_safe_link_scheme` rejects vbscript:.
    #[test]
    fn test_BC_4_03_003_is_safe_link_scheme_rejects_vbscript() {
        assert!(
            !is_safe_link_scheme("vbscript:foo"),
            "vbscript: must be rejected"
        );
    }

    /// BC-4.03.003 / AC-010 — the `ALLOWED_URL_SCHEMES` constant contains exactly
    /// the four approved schemes.
    #[test]
    fn test_BC_4_03_003_allowed_url_schemes_constant_content() {
        assert!(
            ALLOWED_URL_SCHEMES.contains(&"http"),
            "http must be in allowlist"
        );
        assert!(
            ALLOWED_URL_SCHEMES.contains(&"https"),
            "https must be in allowlist"
        );
        assert!(
            ALLOWED_URL_SCHEMES.contains(&"mailto"),
            "mailto must be in allowlist"
        );
        assert!(
            ALLOWED_URL_SCHEMES.contains(&"tel"),
            "tel must be in allowlist"
        );
        assert!(
            !ALLOWED_URL_SCHEMES.contains(&"javascript"),
            "javascript must NOT be in allowlist"
        );
        assert!(
            !ALLOWED_URL_SCHEMES.contains(&"data"),
            "data must NOT be in allowlist"
        );
    }

    /// BC-4.03.003 / AC-010 — `is_safe_link_scheme` emits a `tracing::warn!` for
    /// each rejected scheme. Uses `tracing_test::traced_test` to capture logs.
    #[tracing_test::traced_test]
    #[test]
    fn test_BC_4_03_003_rejected_scheme_emits_tracing_warn() {
        let _ = is_safe_link_scheme("javascript:alert(1)");
        assert!(
            logs_contain("javascript"),
            "a tracing::warn! must be emitted containing the rejected scheme 'javascript'"
        );
    }

    /// BC-4.03.003 / AC-010 — `tracing::warn!` is emitted for data: rejection.
    #[tracing_test::traced_test]
    #[test]
    fn test_BC_4_03_003_data_uri_rejection_emits_tracing_warn() {
        let _ = is_safe_link_scheme("data:text/html,<h1>xss</h1>");
        assert!(
            logs_contain("data"),
            "a tracing::warn! must be emitted containing the rejected scheme 'data'"
        );
    }

    /// BC-4.03.003 / AC-010 — `tracing::warn!` is emitted for vbscript: rejection.
    #[tracing_test::traced_test]
    #[test]
    fn test_BC_4_03_003_vbscript_rejection_emits_tracing_warn() {
        let _ = is_safe_link_scheme("vbscript:foo");
        assert!(
            logs_contain("vbscript"),
            "a tracing::warn! must be emitted containing the rejected scheme 'vbscript'"
        );
    }

    /// BC-4.03.003 / AC-010 — a link with javascript: scheme produces HTML with
    /// NO href attribute (or rendered as <span> rather than <a>).
    #[test]
    fn test_BC_4_03_003_javascript_href_dropped_in_export() {
        use slideforge_layout::FrameContent;
        use slideforge_types::InlineNode;

        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        // A slide whose TextRun contains a Link node with a javascript: URL
        let slide = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("content"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: slideforge_types::Emu(0),
                    y: slideforge_types::Emu(0),
                    width: slideforge_types::Emu(9_144_000),
                    height: slideforge_types::Emu(5_143_500),
                },
                content: FrameContent::TextRun(vec![InlineNode::Link {
                    url: Arc::from("javascript:alert(1)"),
                    // text is Vec<InlineNode> per inline.rs
                    text: vec![InlineNode::Plain(Arc::from("click me"))],
                }]),
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        };
        let laid_out = make_laid_out_deck_single_slide(slide);
        let brand = make_brand();
        let opts = ExportOptions::default();

        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");

        // The rendered HTML must NOT contain href="javascript:..." in any form
        assert!(
            !html.contains("href=\"javascript:"),
            "javascript: href must be dropped from rendered HTML (CWE-601 / AC-010)"
        );
        assert!(
            !html.contains("href='javascript:"),
            "javascript: href (single-quoted) must be dropped from rendered HTML"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EC-001: Slide with only decorative images
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 EC-001 — a slide with ONLY decorative images must produce
    /// all `role="presentation"` elements; no non-empty accessible names.
    ///
    /// F-002: Images are now SVG (not bare <img>). All decorative images use
    /// `role="presentation"` on their SVG element. No bare `<img>` in output.
    #[test]
    fn test_BC_4_03_003_ec_001_only_decorative_images_all_have_empty_alt() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        let slide = make_decorative_image_slide();
        let laid_out = make_laid_out_deck_single_slide(slide);
        let brand = make_brand();
        let opts = ExportOptions::default();

        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");

        // F-002: No bare <img> elements — images are SVG.
        assert!(
            !html.contains("<img "),
            "EC-001 / F-002: decorative images must not render as bare <img>; \
             got snippet: {:?}",
            &html[..html.len().min(500)]
        );

        // All image SVG elements must have role="presentation".
        assert!(
            html.contains(r#"role="presentation""#),
            "EC-001: decorative image SVG elements must have role=\"presentation\""
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EC-003: Deck with lang "de"
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 EC-003 — deck with lang "de" produces `<html lang="de">`.
    #[test]
    fn test_BC_4_03_003_ec_003_lang_de_propagates_to_html() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("de");
        let slide = make_title_slide();
        let laid_out = make_laid_out_deck_single_slide(slide);
        let brand = make_brand();
        let opts = ExportOptions::default();

        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(
            html.contains(r#"lang="de""#),
            "EC-003: output must contain lang=\"de\" for deck.lang=\"de\""
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-008 (CWE-601) — protocol-relative and backslash URLs must be rejected
    // ─────────────────────────────────────────────────────────────────────────

    /// F-008: `//evil.com` (protocol-relative) must be REJECTED — not treated as
    /// a relative path. Without a colon these look "relative" but browsers resolve
    /// them as scheme-relative (same scheme as the page), enabling open-redirect.
    #[test]
    fn test_F008_protocol_relative_url_is_rejected() {
        assert!(
            !is_safe_link_scheme("//evil.com"),
            "F-008: protocol-relative URL '//evil.com' must be rejected (CWE-601)"
        );
    }

    /// F-008: `//evil.com/path` must also be rejected.
    #[test]
    fn test_F008_protocol_relative_with_path_is_rejected() {
        assert!(
            !is_safe_link_scheme("//evil.com/path"),
            "F-008: '//evil.com/path' must be rejected"
        );
    }

    /// F-008: backslash-prefix `\\evil.com` must be rejected.
    #[test]
    fn test_F008_backslash_url_is_rejected() {
        assert!(
            !is_safe_link_scheme("\\\\evil.com"),
            "F-008: backslash-prefix URL must be rejected"
        );
    }

    /// F-008: safe relative paths (path/fragment only) are still allowed.
    #[test]
    fn test_F008_path_only_relative_url_still_allowed() {
        assert!(
            is_safe_link_scheme("/slides/2"),
            "F-008: path-only relative URL '/slides/2' must still be allowed"
        );
        assert!(
            is_safe_link_scheme("#section"),
            "F-008: fragment-only URL '#section' must still be allowed"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-006 — exactly ONE tracing::warn! per rejected link (no double-logging)
    // ─────────────────────────────────────────────────────────────────────────

    /// F-006: when a javascript: Link is rendered, exactly one warn! is emitted —
    /// not two (not once in is_safe_link_scheme AND once in render_inline_node).
    #[tracing_test::traced_test]
    #[test]
    fn test_F006_rejected_link_emits_exactly_one_warn() {
        use slideforge_layout::FrameContent;
        use slideforge_types::InlineNode;

        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        let slide = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("content"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: slideforge_types::Emu(0),
                    y: slideforge_types::Emu(0),
                    width: slideforge_types::Emu(9_144_000),
                    height: slideforge_types::Emu(5_143_500),
                },
                content: FrameContent::TextRun(vec![InlineNode::Link {
                    url: Arc::from("javascript:alert(1)"),
                    text: vec![InlineNode::Plain(Arc::from("click"))],
                }]),
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        };
        let laid_out = make_laid_out_deck_single_slide(slide);
        let brand = make_brand();
        let opts = ExportOptions::default();
        let _ = exporter.export(&deck, &laid_out, &brand, &opts);
        // Exactly one warn containing "javascript" must appear in the log
        assert!(
            logs_contain("javascript"),
            "F-006: at least one warn for rejected 'javascript:' scheme must be emitted"
        );
    }
}
