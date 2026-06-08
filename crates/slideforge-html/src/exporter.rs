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

use slideforge_layout::{FrameContent, LaidOutDeck, LaidOutSlide};
use slideforge_plugin_api::{ExportError, ExportOptions, Exporter};
use slideforge_types::{Brand, Deck};

use crate::render::{HeadingLevel, render_slide_to_html};

/// Determine the [`HeadingLevel`] for each slide in the deck.
///
/// Implements the exporter-level pre-pass (CRITICAL-B1 / BC-4.03.003 postcondition 6):
///
/// 1. Find the first slide with `slide_type_keyword == "title"` that also has a
///    `FrameContent::Title` frame → that slide's Title frame emits `<h1>`.
/// 2. If no title-type slide: find the first slide with any `FrameContent::Title`
///    frame → that frame gets `<h1>`.
/// 3. If no Title frame anywhere (body-only deck): promote the first slide that
///    has a `FrameContent::Body` frame → that frame gets `<h1>`. Emits
///    `tracing::warn!` because a page-has-heading-one invariant requires an h1
///    but the deck has no semantic Title frames.
///
/// All other slides receive `HeadingLevel::H2`.
///
/// Returns a `Vec<HeadingLevel>` parallel to `slides`.
#[allow(clippy::match_same_arms)] // branches are semantically distinct even if code is similar
fn compute_heading_levels(slides: &[LaidOutSlide]) -> Vec<HeadingLevel> {
    let len = slides.len();
    let mut levels = vec![HeadingLevel::H2; len];

    // Pass 1: first slide with slide_type == "title" AND a Title frame.
    let h1_idx = slides.iter().position(|s| {
        s.slide_type_keyword.as_ref() == "title"
            && s.frames
                .iter()
                .any(|f| matches!(f.content, FrameContent::Title(_)))
    });

    if let Some(idx) = h1_idx {
        levels[idx] = HeadingLevel::H1;
        return levels;
    }

    // Pass 2: first slide with any Title frame (regardless of slide_type).
    let h1_idx = slides.iter().position(|s| {
        s.frames
            .iter()
            .any(|f| matches!(f.content, FrameContent::Title(_)))
    });

    if let Some(idx) = h1_idx {
        levels[idx] = HeadingLevel::H1;
        return levels;
    }

    // Pass 3: no Title frames anywhere — body-only deck.
    // Promote the first slide's first body frame to h1 by giving that slide H1 level.
    // The per-slide render logic will emit the first body block as h1 when level is H1
    // BUT render_text_frame only uses heading_level for FrameContent::Title.
    // For a body-only deck we need special handling: mark level H1 on first slide,
    // and the body frame will be wrapped in a special h1 wrapper.
    // Since render_text_frame Body renders as <div class="sf-body">, we must use
    // the BodyH1 sentinel. We encode this as a special variant that is handled in
    // render_slide_to_html: when heading_level==H1 and there's no Title frame,
    // the first Body frame is wrapped in <h1>.
    // For simplicity in this implementation, we use HeadingLevel::H1 on the first slide
    // and handle the body-h1 promotion inside render_slide_to_html.
    tracing::warn!(
        "HtmlExporter: no Title frame found in any slide — \
         promoting first body frame to <h1> to satisfy page-has-heading-one (axe-core). \
         Add a slide with slide_type=\"title\" or a Title frame to fix this."
    );
    if !slides.is_empty() {
        levels[0] = HeadingLevel::H1;
    }
    levels
}

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

        // CRITICAL-B1: exporter-level pre-pass to assign exactly one <h1> per document.
        // compute_heading_levels returns a Vec<HeadingLevel> parallel to slides.
        let heading_levels = compute_heading_levels(&laid_out.slides);

        // Render all slides (MED-3: thread page_size; B1: thread pre-computed heading_level).
        let page_size = &laid_out.page_size;
        let mut slides_html = String::new();
        for (slide, &heading_level) in laid_out.slides.iter().zip(heading_levels.iter()) {
            slides_html.push_str(&render_slide_to_html(
                slide,
                brand,
                heading_level,
                page_size,
            ));
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
    // AC-006: No <canvas>, no <foreignObject>, <article> container present
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

    /// BC-4.03.003 invariant 2 (P4) — no `<foreignObject>` elements in output.
    /// usvg drops foreignObject silently; its presence is a rendering regression.
    #[test]
    fn test_BC_4_03_003_no_foreign_object_in_output() {
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
        let sel = scraper::Selector::parse("foreignObject").expect("valid selector");
        assert_eq!(
            doc.select(&sel).count(),
            0,
            "P4: output must contain ZERO <foreignObject> elements"
        );
    }

    /// BC-4.03.003 invariant 2 (P4) — at least one `<article>` slide container.
    #[test]
    fn test_BC_4_03_003_article_container_present() {
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
        let sel = scraper::Selector::parse("article").expect("valid selector");
        assert!(
            doc.select(&sel).count() > 0,
            "P4: output must contain at least one <article> slide container"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-003: Non-decorative images have non-empty alt / <title> (P4 SVG layer)
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 postcondition 4 — non-decorative images have non-empty accessible name.
    ///
    /// P4: Images go to the SVG graphics layer. Non-decorative images use
    /// `<g role="img" aria-labelledby><title>alt text</title>...</g>`.
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

        // P4: image is in SVG layer, not bare <img>
        assert!(
            !html.contains("<img "),
            "P4: non-decorative image must not be bare <img>; got snippet: {:?}",
            &html[..html.len().min(500)]
        );

        // Assert: the alt text "A revenue chart" appears in the output (in <title>)
        assert!(
            html.contains("A revenue chart"),
            "non-decorative image alt text must appear in HTML output (in <title>)"
        );
        // Must have role="img" on the <g> wrapper in the SVG layer.
        assert!(
            html.contains(r#"role="img""#),
            "non-decorative image must have role=\"img\" on the <g> wrapper in SVG layer"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-004: Decorative images have aria-hidden="true" in SVG layer (P4)
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 postcondition 5 — decorative images are hidden from AT.
    ///
    /// P4: Decorative images go to the SVG graphics layer with
    /// `<g aria-hidden="true">` — NOT `role="presentation"` on an `<img>`.
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

        // P4: decorative image is hidden via aria-hidden="true" on the <g> group
        assert!(
            !html.contains("<img "),
            "P4: decorative image must not render as bare <img>; got snippet: {:?}",
            &html[..html.len().min(500)]
        );

        // Decorative element must have aria-hidden="true" (P4 SVG layer pattern)
        assert!(
            html.contains(r#"aria-hidden="true""#),
            "decorative image must have aria-hidden=\"true\" in the SVG layer (P4)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-008: Heading hierarchy — slide_type drives heading level (P4)
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-4.03.003 postcondition 7 — title-slide at index 0 maps title frame to `<h1>`.
    /// (AC-008 P4: heading level from slide_type_keyword, not content heuristics.)
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
            "title-slide at index 0 must produce at least one <h1> element in the HTML output"
        );
    }

    /// BC-4.03.003 postcondition 7 (P4) — in a 2-slide deck where slide 0 is the
    /// title-slide (h1) and slide 1 is a content slide with Title + Subtitle,
    /// the content slide produces h2 → h3 (no skipped levels).
    /// Heading order: h2 before h3.
    #[test]
    fn test_BC_4_03_003_heading_hierarchy_no_skipped_levels() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");

        // Slide 0: title-type → gets H1 from pre-pass
        let title_slide = make_title_slide(); // title-type slide

        // Slide 1: content slide with Title (→ h2) and Subtitle (→ h3)
        let content_slide = LaidOutSlide {
            source_index: 1,
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

        let laid_out = LaidOutDeck {
            page_size: slideforge_layout::PageSize::default(),
            slides: vec![title_slide, content_slide],
            sections: vec![],
            warnings: vec![],
        };
        let brand = make_brand();
        let opts = ExportOptions::default();

        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");
        let doc = scraper::Html::parse_document(&html);

        // Content-type slide (non-h1): Title → h2, Subtitle → h3.
        // No heading levels are skipped: h2 before h3 is valid.
        let sel_h2 = scraper::Selector::parse("h2").expect("valid selector");
        let sel_h3 = scraper::Selector::parse("h3").expect("valid selector");
        assert!(
            doc.select(&sel_h2).count() > 0,
            "content slide Title must produce h2; P4 heading hierarchy rule"
        );
        assert!(
            doc.select(&sel_h3).count() > 0,
            "content slide Subtitle must produce h3 (below h2, no level skip)"
        );
        // h2 must appear before h3 in document order (no skip).
        let h2_pos = html.find("<h2").expect("h2 must be present");
        let h3_pos = html.find("<h3").expect("h3 must be present");
        assert!(
            h2_pos < h3_pos,
            "h2 must appear before h3 in document order (no level skip); got: {html}"
        );
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
    /// aria-hidden elements; no accessible names exposed to AT.
    ///
    /// P4: Decorative images in the SVG layer use `<g aria-hidden="true">`.
    /// No bare `<img>` in output.
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

        // P4: No bare <img> elements — images are in SVG graphics layer.
        assert!(
            !html.contains("<img "),
            "EC-001 / P4: decorative images must not render as bare <img>; \
             got snippet: {:?}",
            &html[..html.len().min(500)]
        );

        // P4: Decorative image groups must have aria-hidden="true".
        assert!(
            html.contains(r#"aria-hidden="true""#),
            "EC-001 / P4: decorative image group must have aria-hidden=\"true\""
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

    /// F-006: when a javascript: Link is rendered, EXACTLY ONE warn! is emitted —
    /// not two (not once in is_safe_link_scheme AND once in render_inline_node).
    /// LOW-1: count == 1, not just >= 1.
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

        // LOW-1: EXACTLY one warn containing "javascript" must appear in the log
        // (not two — one from is_safe_link_scheme, NOT a second from render_inline_node).
        logs_assert(|lines| {
            let matching: Vec<&&str> = lines
                .iter()
                .filter(|line| line.contains("javascript"))
                .collect();
            if matching.len() == 1 {
                Ok(())
            } else {
                Err(format!(
                    "F-006 / LOW-1: expected exactly 1 warn containing 'javascript', \
                     got {}. Lines: {:?}",
                    matching.len(),
                    matching
                ))
            }
        });
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CRITICAL-B1 + MED-B4 — exporter-level single <h1> invariant
    // ─────────────────────────────────────────────────────────────────────────

    fn make_content_slide_with_title(title: &str) -> LaidOutSlide {
        LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("content"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: slideforge_types::Emu(0),
                    y: slideforge_types::Emu(0),
                    width: slideforge_types::Emu(9_144_000),
                    height: slideforge_types::Emu(1_000_000),
                },
                content: FrameContent::Title(Arc::from(title)),
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        }
    }

    fn make_body_only_slide(body_text: &str) -> LaidOutSlide {
        use slideforge_types::{ContentBlock, InlineNode, SourceSpan, TextBlock, TextTag};
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
                content: FrameContent::Body(vec![ContentBlock::Text(TextBlock {
                    inlines: vec![InlineNode::Plain(Arc::from(body_text))],
                    tag: TextTag::Body,
                    span: SourceSpan::default(),
                })]),
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        }
    }

    /// B1/B4: 3 content-type slides (no title-slide) → exactly one <h1>.
    /// The pre-pass must pick the first slide with a Title frame and promote it.
    #[test]
    fn test_B1_three_content_slides_no_title_type_produces_exactly_one_h1() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        let laid_out = LaidOutDeck {
            page_size: slideforge_layout::PageSize::default(),
            slides: vec![
                make_content_slide_with_title("Slide One"),
                make_content_slide_with_title("Slide Two"),
                make_content_slide_with_title("Slide Three"),
            ],
            sections: vec![],
            warnings: vec![],
        };
        let brand = make_brand();
        let opts = ExportOptions::default();
        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");
        let doc = scraper::Html::parse_document(&html);
        let sel_h1 = scraper::Selector::parse("h1").expect("valid selector");
        assert_eq!(
            doc.select(&sel_h1).count(),
            1,
            "B1: 3 content-type slides (no title-slide) must produce exactly one <h1>; got: {html}"
        );
    }

    /// B1/B4: title-slide at index 2 (not 0) → exactly one <h1> on that slide.
    #[test]
    fn test_B1_title_slide_at_index_2_produces_exactly_one_h1() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        // First two slides are content-type, third is title-type
        let laid_out = LaidOutDeck {
            page_size: slideforge_layout::PageSize::default(),
            slides: vec![
                make_content_slide_with_title("Intro"),
                make_content_slide_with_title("Overview"),
                make_title_slide(), // title-type at index 2
            ],
            sections: vec![],
            warnings: vec![],
        };
        let brand = make_brand();
        let opts = ExportOptions::default();
        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");
        let doc = scraper::Html::parse_document(&html);
        let sel_h1 = scraper::Selector::parse("h1").expect("valid selector");
        assert_eq!(
            doc.select(&sel_h1).count(),
            1,
            "B1: title-slide at index 2 must produce exactly one <h1> in the document; got: {html}"
        );
        // The h1 must be the title-slide (third article).
        // Verify: the slide at index 0 (content) produces h2, not h1.
        let sel_h2 = scraper::Selector::parse("h2").expect("valid selector");
        assert!(
            doc.select(&sel_h2).count() >= 2,
            "B1: first two content-type slides must produce h2 headings; got: {html}"
        );
    }

    /// B1/B4 (c): body-only deck (no Title frames anywhere) → exactly one <h1>
    /// on the first body frame (promoted) AND a tracing::warn! emitted.
    #[tracing_test::traced_test]
    #[test]
    fn test_B1_body_only_deck_promotes_first_body_frame_to_h1_and_warns() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        let laid_out = LaidOutDeck {
            page_size: slideforge_layout::PageSize::default(),
            slides: vec![
                make_body_only_slide("First body"),
                make_body_only_slide("Second body"),
            ],
            sections: vec![],
            warnings: vec![],
        };
        let brand = make_brand();
        let opts = ExportOptions::default();
        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");
        let doc = scraper::Html::parse_document(&html);
        let sel_h1 = scraper::Selector::parse("h1").expect("valid selector");
        assert_eq!(
            doc.select(&sel_h1).count(),
            1,
            "B1 (c): body-only deck must produce exactly one <h1> (promoted from first body frame); got: {html}"
        );
        assert!(
            logs_contain("no Title frame"),
            "B1 (c): body-only deck must emit tracing::warn! about missing Title frame; got logs"
        );
    }

    /// B1/B4 (d): multi title-slide deck → still exactly one <h1>.
    #[test]
    fn test_B1_multi_title_slide_deck_still_exactly_one_h1() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        let laid_out = LaidOutDeck {
            page_size: slideforge_layout::PageSize::default(),
            slides: vec![make_title_slide(), make_title_slide(), make_title_slide()],
            sections: vec![],
            warnings: vec![],
        };
        let brand = make_brand();
        let opts = ExportOptions::default();
        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");
        let doc = scraper::Html::parse_document(&html);
        let sel_h1 = scraper::Selector::parse("h1").expect("valid selector");
        assert_eq!(
            doc.select(&sel_h1).count(),
            1,
            "B1 (d): multi title-slide deck must produce exactly one <h1>; got: {html}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HIGH-B2 — aria-hidden suppression bug
    // ─────────────────────────────────────────────────────────────────────────

    /// B2: The outer graphics <svg> must carry role="presentation", NOT aria-hidden="true".
    /// Non-decorative <g role="img"> must have NO aria-hidden="true" ancestor.
    #[test]
    fn test_B2_outer_svg_has_role_presentation_not_aria_hidden() {
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
                content: FrameContent::Chart {
                    alt: AltText::Provided(Arc::from("Revenue chart")),
                },
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        };
        let laid_out = LaidOutDeck {
            page_size: slideforge_layout::PageSize::default(),
            slides: vec![slide],
            sections: vec![],
            warnings: vec![],
        };
        let brand = make_brand();
        let opts = ExportOptions::default();
        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");

        // The outer SVG graphics layer must have role="presentation"
        assert!(
            html.contains(r#"role="presentation""#),
            "B2: outer graphics <svg> must carry role=\"presentation\"; got: {html}"
        );

        // The outer SVG must NOT carry aria-hidden="true" at the svg element level
        // (aria-hidden on ancestor hides whole subtree including <g role="img"> children).
        // Find the outer svg tag and assert it does NOT have aria-hidden="true".
        let svg_start = html.find("<svg").expect("must contain svg");
        let svg_tag_end = html[svg_start..].find('>').expect("svg must close");
        let outer_svg_tag = &html[svg_start..=(svg_start + svg_tag_end)];
        assert!(
            !outer_svg_tag.contains(r#"aria-hidden="true""#),
            "B2: outer graphics <svg> must NOT have aria-hidden=\"true\" \
             (hides non-decorative <g role=\"img\"> from AT); got outer svg tag: {outer_svg_tag}"
        );
    }

    /// B2: Decorative <g> carries aria-hidden="true" individually.
    #[test]
    fn test_B2_decorative_frame_g_carries_aria_hidden() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        let slide = make_decorative_image_slide();
        let laid_out = LaidOutDeck {
            page_size: slideforge_layout::PageSize::default(),
            slides: vec![slide],
            sections: vec![],
            warnings: vec![],
        };
        let brand = make_brand();
        let opts = ExportOptions::default();
        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");

        // The decorative <g> must carry aria-hidden="true"
        assert!(
            html.contains(r#"<g aria-hidden="true""#),
            "B2: decorative frame <g> must carry aria-hidden=\"true\"; got: {html}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MED-B3 — canonical id format sf-{slide_id}-{frame_index}
    // ─────────────────────────────────────────────────────────────────────────

    /// B3: chart frame id must match sf-slide-{n}-{idx} (no doubled slide_id).
    #[test]
    fn test_B3_chart_frame_id_canonical_format_no_doubling() {
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
                content: FrameContent::Chart {
                    alt: AltText::Provided(Arc::from("Revenue")),
                },
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        };
        let laid_out = LaidOutDeck {
            page_size: slideforge_layout::PageSize::default(),
            slides: vec![slide],
            sections: vec![],
            warnings: vec![],
        };
        let brand = make_brand();
        let opts = ExportOptions::default();
        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");

        // Must contain sf-slide-1-0 (0-based frame index)
        assert!(
            html.contains("sf-slide-1-0"),
            "B3: chart frame id must be 'sf-slide-1-0' (0-based frame index); got: {html}"
        );
        // Must NOT contain doubled slide_id like sf-slide-1-slide-1-...
        assert!(
            !html.contains("sf-slide-1-slide-1"),
            "B3: doubled slide_id pattern 'sf-slide-1-slide-1' must not appear; got: {html}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MED-B5 — negative/degenerate geometry
    // ─────────────────────────────────────────────────────────────────────────

    /// B5: negative width in text frame must be skipped (no emit width="-N").
    #[test]
    fn test_B5_negative_width_text_frame_is_skipped() {
        use crate::render::render_text_frame;
        use slideforge_layout::BoundingBox;
        use slideforge_types::Emu;

        let frame = Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(-500_000),
                height: Emu(1_000_000),
            },
            content: FrameContent::Title(Arc::from("Negative width")),
            text_flow: None,
            region_role: None,
        };
        let result = render_text_frame(&frame, crate::render::HeadingLevel::H2);
        assert!(
            result.is_none(),
            "B5: negative-width frame must return None from render_text_frame; got: {result:?}"
        );
    }

    /// B5: negative height in text frame must be skipped.
    #[test]
    fn test_B5_negative_height_text_frame_is_skipped() {
        use crate::render::render_text_frame;
        use slideforge_layout::BoundingBox;
        use slideforge_types::Emu;

        let frame = Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(9_144_000),
                height: Emu(-500_000),
            },
            content: FrameContent::Title(Arc::from("Negative height")),
            text_flow: None,
            region_role: None,
        };
        let result = render_text_frame(&frame, crate::render::HeadingLevel::H2);
        assert!(
            result.is_none(),
            "B5: negative-height frame must return None from render_text_frame; got: {result:?}"
        );
    }

    /// B5: negative width in graphics layer frame is skipped (no crash, no negative CSS).
    #[test]
    fn test_B5_negative_width_graphics_frame_is_skipped() {
        use crate::render::render_graphics_layer;
        use slideforge_layout::BoundingBox;
        use slideforge_types::Emu;

        let frames = vec![Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(-9_144_000),
                height: Emu(5_143_500),
            },
            content: FrameContent::Chart {
                alt: AltText::Provided(Arc::from("Broken chart")),
            },
            text_flow: None,
            region_role: None,
        }];
        let page_size = slideforge_layout::PageSize::default();
        let result = render_graphics_layer(&frames, "slide-1", &page_size);
        // Skipped frame → empty graphics layer
        assert!(
            result.is_empty(),
            "B5: negative-width graphics frame must be skipped; got: {result}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // LOW-B7 — empty <nav> must not appear in page output
    // ─────────────────────────────────────────────────────────────────────────

    /// B7: the page template must NOT emit an empty <nav aria-label="Slide navigation">.
    #[test]
    fn test_B7_no_empty_nav_in_page_output() {
        let exporter = HtmlExporter::new();
        let deck = make_deck("en-US");
        let slide = make_title_slide();
        let laid_out = LaidOutDeck {
            page_size: slideforge_layout::PageSize::default(),
            slides: vec![slide],
            sections: vec![],
            warnings: vec![],
        };
        let brand = make_brand();
        let opts = ExportOptions::default();
        let bytes = exporter
            .export(&deck, &laid_out, &brand, &opts)
            .expect("export must succeed");
        let html = String::from_utf8(bytes).expect("valid UTF-8");

        // The nav element with empty body must not appear.
        let doc = scraper::Html::parse_document(&html);
        let sel = scraper::Selector::parse(r#"nav[aria-label="Slide navigation"]"#)
            .expect("valid selector");
        assert_eq!(
            doc.select(&sel).count(),
            0,
            "B7: empty <nav aria-label=\"Slide navigation\"> must be removed (deferred to STORY-047); got: {html}"
        );
    }
}
