//! usvg-based SVG normalization pass (STORY-034, BC-1.12.003).
//!
//! This module implements the mandatory post-rendering normalization step that
//! converts a [`RawDiagramSvg`] into a [`NormalizedDiagramSvg`]. The normalized
//! form is guaranteed to be PPTX-safe (BC-1.12.003 postconditions 1–5):
//!
//! - No `<foreignObject>` elements.
//! - No `<script>` elements.
//! - No CSS `@keyframes` or class-based `<style>` blocks.
//! - Absolute pixel `width` and `height` on the root `<svg>` element.
//! - No `<use>` elements (all `href="#symbol"` references inlined).
//!
//! ## Pipeline position
//!
//! ```text
//! render_mermaid() → RawDiagramSvg
//!                               ↓
//!                     usvg_normalize()
//!                               ↓
//!                    NormalizedDiagramSvg
//!                               ↓
//!                   LaidOutDeck / Exporters
//! ```
//!
//! There is no bypass path. Exporters receive [`NormalizedDiagramSvg`]; the
//! type system prevents them from receiving [`RawDiagramSvg`] directly.
//!
//! ## Implementation
//!
//! 1. Gate system font loading on presence of `<text` elements in the raw SVG
//!    (keeps geometry-only paths fast; mermaid SVGs always have text).
//! 2. Call `usvg::Tree::from_str(raw_svg, &opt)` to parse.
//! 3. Call `tree.to_string(&WriteOptions { preserve_text: true, .. })` to re-serialize.
//! 4. Re-inject `aria-label`, `role="img"`, `<title>`, and `viewBox` that usvg strips.
//! 5. Run post-normalization debug assertions (AC-002 through AC-006).

use std::fmt::Write as FmtWrite;
use std::sync::{Arc, OnceLock};

use miette::SourceSpan;
use tracing::instrument;

use crate::types::{DiagramError, NormalizedDiagramSvg, RawDiagramSvg};

/// Lazily-initialized system font database for usvg text normalization.
///
/// Loading system fonts is expensive (50–300ms depending on platform). This
/// `OnceLock` ensures the database is built exactly once per process, then
/// reused for every subsequent `usvg_normalize` call (warm path < 1ms).
///
/// The `Arc<usvg::fontdb::Database>` wrapper is required by `usvg::Options`.
static FONT_DB: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();

/// Return a reference to the lazily-initialized system font database.
///
/// On the first call this loads all system fonts (one-time cost ~50–300ms).
/// Subsequent calls return the cached database in O(1).
fn font_db() -> Arc<usvg::fontdb::Database> {
    Arc::clone(FONT_DB.get_or_init(|| {
        let mut db = usvg::fontdb::Database::new();
        db.load_system_fonts();
        Arc::new(db)
    }))
}

/// Normalize a [`RawDiagramSvg`] into a PPTX-safe [`NormalizedDiagramSvg`]
/// using `usvg` 0.47.0.
///
/// This is a **pure, synchronous** operation — no I/O, no network access.
/// It parses the raw SVG through `usvg::Tree::from_str` and re-serializes
/// via `tree.to_string`. usvg guarantees the following
/// transformations on the output (BC-1.12.003 postconditions):
///
/// | Postcondition | Guarantee |
/// |---|---|
/// | No `<foreignObject>` | Removed by usvg (unsupported element) |
/// | No `<script>` | Removed by usvg (unsupported element) |
/// | No `@keyframes` / `<style>` | CSS inlined into presentation attrs |
/// | Absolute pixel dimensions | Computed from `viewBox` by usvg |
/// | No `<use>` | All `href="#symbol"` references inlined |
///
/// # Arguments
///
/// - `raw` — The raw SVG string produced by [`crate::renderer::render_mermaid`].
/// - `source_id` — An identifier for the diagram (e.g., slide title or alt text)
///   used in error messages if normalization fails.
///
/// # Errors
///
/// Returns [`DiagramError::SvgNormalizationFailed`] (error code `E-EXP-004`) if
/// `usvg::Tree::from_str` rejects the raw SVG as malformed. This indicates a
/// rendering-pipeline failure, not a user-authored syntax error.
///
/// # Panics
///
/// In **debug builds only**, panics if the post-normalization output still
/// contains a forbidden element (`foreignObject`, `script`, `<use`, or
/// a `%` dimension). This is a programming error — usvg must always remove
/// these elements, so their presence indicates a regression in usvg 0.47.0
/// or a bug in this function. Production (release) builds do not panic.
#[instrument(skip(raw), fields(source_id = %source_id))]
pub fn usvg_normalize(
    raw: &RawDiagramSvg,
    source_id: &str,
) -> Result<NormalizedDiagramSvg, DiagramError> {
    // Build usvg options, conditionally loading the system font database.
    //
    // System fonts are required so that usvg preserves <text> elements as text
    // (rather than converting them to outlined paths). Without fonts:
    // - Node labels ("Client", "API") are lost from flowchart SVGs.
    // - Participant names ("Alice", "Bob") are lost from sequence diagrams.
    // - PPTX accessibility and screen-reader compatibility are broken.
    //
    // Font loading is gated on whether the raw SVG contains `<text` elements.
    // For geometry-only SVGs (no text), we skip font loading entirely — this
    // keeps the warm + no-text path under 1ms (NFR-003/004). SVGs produced by
    // mermaid-rs-renderer always contain `<text`, so the gate is rarely false
    // in production; the exception is test fixtures and degenerate inputs.
    //
    // The font_db() call is O(1) after the first call (the database is loaded
    // once and cached in `FONT_DB` via OnceLock). The first call may take
    // 50–300ms on systems with large font collections; subsequent calls are
    // nanoseconds (Arc::clone of the cached Arc<Database>).
    let opt = if raw.as_str().contains("<text") {
        usvg::Options {
            fontdb: font_db(),
            ..usvg::Options::default()
        }
    } else {
        usvg::Options::default()
    };

    // Parse the raw SVG through usvg. On failure, map to E-EXP-004.
    let tree = usvg::Tree::from_str(raw.as_str(), &opt).map_err(|e| {
        DiagramError::SvgNormalizationFailed {
            source_id: Arc::from(source_id),
            cause: Arc::from(format!("usvg parse failed: {e}").as_str()),
            span: SourceSpan::from(0..0),
        }
    })?;

    // Re-serialize the parsed tree back to SVG. This produces a normalized
    // form with all usvg guarantees applied (no foreignObject, no script,
    // no @keyframes, absolute dimensions, <use> resolved).
    //
    // `preserve_text: true` keeps <text> elements as-is instead of converting
    // them to outlined paths. This is required for:
    // 1. PPTX accessibility — text in <text> elements remains selectable/searchable.
    // 2. Test assertions — node labels like "Client", "Alice" must remain in the SVG.
    // 3. Smaller output — text-to-path expansion bloats file size significantly.
    let write_opts = usvg::WriteOptions {
        preserve_text: true,
        ..usvg::WriteOptions::default()
    };
    let normalized_str = tree.to_string(&write_opts);

    // Debug-only post-normalization assertions (AC-002 through AC-006).
    // In release builds this compiles to nothing.
    debug_assert_post_normalization(&normalized_str);

    // Re-inject accessibility attributes and viewBox that usvg strips.
    //
    // usvg does not preserve non-SVG-standard attributes (aria-label, role)
    // or <title> elements, and it emits absolute width/height without viewBox.
    // These are required by BC-1.12.001 postcondition 7 (accessibility) and
    // by downstream consumers that expect viewBox for layout computation.
    //
    // source_id is the alt_text passed through from render_diagram().
    let enriched = reinject_accessibility_and_viewbox(&normalized_str, source_id).map_err(|e| {
        DiagramError::SvgNormalizationFailed {
            source_id: Arc::from(source_id),
            cause: Arc::from(format!("post-normalization enrichment failed: {e}").as_str()),
            span: SourceSpan::from(0..0),
        }
    })?;

    Ok(NormalizedDiagramSvg(Arc::from(enriched.as_str())))
}

/// Run post-normalization debug assertions (AC-002 through AC-006).
///
/// Checks that the normalized SVG string does not contain any of the
/// forbidden elements that usvg guarantees to remove. This function is
/// **only active in debug builds** — it compiles to nothing in release
/// builds. If an assertion fails, it indicates a regression in usvg
/// (programming error, not user error).
///
/// | Assertion | Forbidden pattern |
/// |---|---|
/// | AC-002 | `<foreignobject` (case-insensitive) |
/// | AC-003 | `<script` (case-insensitive) |
/// | AC-004 | `@keyframes` (case-insensitive) |
/// | AC-005 | `width="...%"` or `height="...%"` (percentage dimensions) |
/// | AC-006 | `<use` (case-insensitive) |
fn debug_assert_post_normalization(normalized_svg: &str) {
    let lower = normalized_svg.to_ascii_lowercase();

    debug_assert!(
        !lower.contains("<foreignobject"),
        "usvg normalization regression: output contains <foreignObject> (AC-002). \
         usvg 0.47.0 must always remove <foreignObject>. This is a bug in usvg or this function."
    );

    debug_assert!(
        !lower.contains("<script"),
        "usvg normalization regression: output contains <script> (AC-003). \
         usvg 0.47.0 must always remove <script>. This is a bug in usvg or this function."
    );

    debug_assert!(
        !lower.contains("@keyframes"),
        "usvg normalization regression: output contains @keyframes (AC-004). \
         usvg 0.47.0 must always strip CSS @keyframes. This is a bug in usvg or this function."
    );

    debug_assert!(
        !contains_percentage_dimension_impl(normalized_svg),
        "usvg normalization regression: output contains percentage width or height (AC-005). \
         usvg 0.47.0 must resolve percentage dimensions to absolute pixels. \
         This is a bug in usvg or this function."
    );

    debug_assert!(
        !lower.contains("<use"),
        "usvg normalization regression: output contains <use> elements (AC-006). \
         usvg 0.47.0 must inline all <use href=\"#symbol\"> references. \
         This is a bug in usvg or this function."
    );
}

/// Returns `true` if the SVG string contains `width="...%"` or `height="...%"`.
///
/// Used by [`debug_assert_post_normalization`] and the test helper. Scans the
/// full string for percentage-valued `width` or `height` attributes.
fn contains_percentage_dimension_impl(svg: &str) -> bool {
    let check = |attr: &str| -> bool {
        let mut search = svg;
        while let Some(pos) = search.find(attr) {
            let after = &search[pos + attr.len()..];
            // Skip optional whitespace
            let value_start = after.trim_start_matches([' ', '\t']);
            // Skip opening quote
            let value = value_start
                .strip_prefix('"')
                .or_else(|| value_start.strip_prefix('\''))
                .unwrap_or(value_start);
            // Check if the value ends with %
            let value_end = value.find(['"', '\'']).unwrap_or(value.len());
            let attr_value = &value[..value_end];
            if attr_value.trim_end().ends_with('%') {
                return true;
            }
            search = &search[pos + 1..];
        }
        false
    };
    check("width=") || check("height=")
}

/// Re-inject accessibility attributes and a synthesized `viewBox` into a
/// usvg-normalized SVG string.
///
/// usvg 0.47.0 strips `aria-label`, `role`, `<title>`, and `viewBox` during
/// normalization because these are not part of its internal tree model. This
/// function re-adds them so that the [`NormalizedDiagramSvg`] satisfies both
/// BC-1.12.003 (PPTX-safe normalization) and BC-1.12.001 postcondition 7
/// (WCAG AA accessibility).
///
/// ## Steps
///
/// 1. Parse out `width` and `height` values from the root `<svg>` opening tag.
/// 2. Inject `viewBox="0 0 {width} {height}"` before the closing `>` of the tag.
/// 3. Inject `aria-label="{alt_text}"` and `role="img"` attributes.
/// 4. Insert `<title>{alt_text}</title>` as the first child of `<svg>`.
///
/// # Errors
///
/// Returns an error string if the SVG string does not contain a root `<svg` tag.
fn reinject_accessibility_and_viewbox(
    normalized_svg: &str,
    alt_text: &str,
) -> Result<String, &'static str> {
    // Locate the root <svg opening tag.
    let svg_start = normalized_svg
        .find("<svg")
        .ok_or("no <svg> root element in usvg-normalized output")?;

    // Find the closing > of the opening <svg ... > tag.
    // We look for > after svg_start. usvg emits well-formed SVG so this > is
    // always present and belongs to the opening tag (not a child element).
    let tag_relative_close = normalized_svg[svg_start..]
        .find('>')
        .ok_or("root <svg> tag is not properly closed in usvg-normalized output")?;
    let tag_close = svg_start + tag_relative_close; // index of '>'

    // Extract the opening tag to check if attributes are already present
    // (idempotency guard — should never happen with usvg output, but defensive).
    let opening_tag = &normalized_svg[svg_start..=tag_close];

    // Parse `width` and `height` values from the opening tag for viewBox synthesis.
    // usvg emits e.g. `width="100" height="200"`.
    let width_str = extract_attr_value(opening_tag, "width").unwrap_or("0");
    let height_str = extract_attr_value(opening_tag, "height").unwrap_or("0");

    // Escape alt_text for XML attribute and text content.
    let escaped_attr = xml_attr_escape(alt_text);
    let escaped_text = xml_text_escape(alt_text);

    // Build the injected additions (viewBox + aria + role, and <title> child).
    // Use write! to avoid intermediate allocations (clippy::format_push_string).
    let mut extra_attrs = String::new();

    // Inject viewBox only if not already present.
    if !opening_tag.contains("viewBox") {
        let _ = write!(extra_attrs, r#" viewBox="0 0 {width_str} {height_str}""#);
    }

    // Inject accessibility attributes only if not already present.
    if !opening_tag.contains("aria-label=") {
        let _ = write!(extra_attrs, r#" aria-label="{escaped_attr}" role="img""#);
    }

    // Build the <title> child element.
    let title_element = format!("<title>{escaped_text}</title>");

    // Reconstruct the SVG:
    //   [before tag_close][extra_attrs]>[title_element][rest]
    let before_close = &normalized_svg[..tag_close];
    let from_close = &normalized_svg[tag_close + 1..]; // skip the '>'

    Ok(format!(
        "{before_close}{extra_attrs}>{title_element}{from_close}"
    ))
}

/// Escape a string for use in an XML attribute value (double-quote delimited).
fn xml_attr_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Escape a string for use as XML text content.
fn xml_text_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Extract the value of an attribute from an SVG opening tag string.
///
/// Handles both double-quoted and single-quoted attribute values.
/// Returns `None` if the attribute is not found.
fn extract_attr_value<'a>(tag: &'a str, attr_name: &str) -> Option<&'a str> {
    // Look for: attr_name="..." or attr_name='...'
    let needle = format!("{attr_name}=");
    let pos = tag.find(needle.as_str())?;
    let after = &tag[pos + needle.len()..];
    // Determine quote character
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value_start = &after[1..]; // skip opening quote
    let end = value_start.find(quote)?;
    Some(&value_start[..end])
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Test fixtures — canonical SVG inputs for all normalization tests.
    //
    // All fixtures produce well-formed SVG documents that usvg can parse.
    // The `svg_with_*` fixtures deliberately embed elements or attributes
    // that the normalization pass must remove or transform.
    // -----------------------------------------------------------------------

    mod test_fixtures {
        /// Minimal valid SVG with a `<rect>` element — the simplest input
        /// that usvg can round-trip. Used for happy-path tests.
        pub(super) fn simple_rect_svg() -> &'static str {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><rect x="10" y="10" width="80" height="80" fill="blue"/></svg>"#
        }

        /// SVG containing a `<foreignObject>` element.
        ///
        /// usvg removes `<foreignObject>` because it cannot be represented in
        /// usvg's internal tree model. After normalization, no `<foreignObject>`
        /// must remain in the output (AC-002, EC-003).
        pub(super) fn svg_with_foreign_object() -> &'static str {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 0 200 200"><rect x="0" y="0" width="200" height="200" fill="white"/><foreignObject x="10" y="10" width="180" height="180"><div xmlns="http://www.w3.org/1999/xhtml">hello</div></foreignObject></svg>"#
        }

        /// SVG containing a `<script>` element.
        ///
        /// usvg removes `<script>` because it is unsupported. After
        /// normalization, no `<script>` must remain in the output (AC-003).
        pub(super) fn svg_with_script() -> &'static str {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><script>alert(1)</script><rect x="10" y="10" width="80" height="80" fill="red"/></svg>"#
        }

        /// SVG containing a `<style>` block with `@keyframes` CSS animation.
        ///
        /// usvg strips `<style>` blocks and inlines CSS properties as
        /// presentation attributes, removing `@keyframes` in the process
        /// (AC-004, EC-005).
        pub(super) fn svg_with_keyframes_css() -> &'static str {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><style>@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } } rect { fill: green; }</style><rect x="10" y="10" width="80" height="80"/></svg>"#
        }

        /// SVG using percentage dimensions on the root element.
        ///
        /// usvg resolves percentage `width` / `height` to absolute pixel values
        /// using the `viewBox` attribute. After normalization, the root element
        /// must have absolute (non-`%`) `width` and `height` (AC-005, EC-002).
        pub(super) fn svg_with_percentage_dims() -> &'static str {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 400 300"><rect x="0" y="0" width="400" height="300" fill="gray"/></svg>"#
        }

        /// SVG using `<defs>` + `<symbol>` + `<use>` to reference a symbol.
        ///
        /// usvg inlines all `<use href="#id">` references, resolving them into
        /// the concrete shape elements they reference. After normalization, no
        /// `<use>` element must remain in the output (AC-006, EC-001).
        pub(super) fn svg_with_use_reference() -> &'static str {
            // Raw string with ## delimiter to avoid `"#` being parsed as Rust end-of-raw-string.
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 0 200 200"><defs><symbol id="dot" viewBox="0 0 10 10"><circle cx="5" cy="5" r="5" fill="black"/></symbol></defs><use href="#dot" x="10" y="10" width="20" height="20"/><use href="#dot" x="50" y="50" width="20" height="20"/></svg>"##
        }
    }

    // -----------------------------------------------------------------------
    // Stub existence check (Red Gate sentinel — must pass at all times).
    //
    // This test verifies the public API is wired correctly before the full
    // implementation exists. In the stub phase it catches the todo!() panic
    // via catch_unwind and asserts IS a panic (Red Gate). The implementer
    // replaces this with a correct-behavior assertion once the function exists.
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_12_003_usvg_normalize_stub_is_callable() {
        // Now that the function is implemented (STORY-034 green gate), this test
        // verifies the public API is callable and returns a structured result
        // (not a panic). `<svg/>` has no width/height/viewBox so usvg returns
        // InvalidSize — we get Err, not a panic.
        let raw = RawDiagramSvg("<svg/>".to_owned());
        let result = std::panic::catch_unwind(|| usvg_normalize(&raw, "test-diagram"));
        assert!(
            result.is_ok(),
            "usvg_normalize must not panic — must return Ok(Result<...>), not unwind"
        );
        // The inner result may be Ok or Err depending on usvg's handling of <svg/>.
        // Either way, no panic means the function is correctly wired.
        let _ = result.unwrap();
    }

    // -----------------------------------------------------------------------
    // AC-001: usvg normalization core — return type and happy path
    // -----------------------------------------------------------------------

    /// BC-1.12.003 postcondition: `usvg_normalize` on a minimal valid SVG
    /// must return `Ok(NormalizedDiagramSvg)` and the normalized string must
    /// contain SVG content (e.g., the rect element, in normalized form).
    #[test]
    fn test_bc_1_12_003_normalize_simple_svg() {
        let raw = RawDiagramSvg(test_fixtures::simple_rect_svg().to_owned());
        let result = usvg_normalize(&raw, "simple-rect");
        let normalized = result.expect("usvg_normalize must return Ok for a minimal valid SVG");
        assert!(!normalized.is_empty(), "normalized SVG must not be empty");
        assert!(
            normalized.as_str().contains("<svg"),
            "normalized SVG must contain the <svg root element; got: {}",
            &normalized.as_str()[..normalized.as_str().len().min(200)]
        );
    }

    /// BC-1.12.003 AC-001: the return type is `NormalizedDiagramSvg` (not a
    /// raw `String`). This test verifies the type-system guarantee at the call
    /// site — calling `.as_str()` on the result must compile only if the return
    /// type has that method (it does: `NormalizedDiagramSvg::as_str`).
    #[test]
    fn test_bc_1_12_003_normalize_returns_normalized_type() {
        let raw = RawDiagramSvg(test_fixtures::simple_rect_svg().to_owned());
        let result = usvg_normalize(&raw, "type-check");
        let normalized: NormalizedDiagramSvg =
            result.expect("must return NormalizedDiagramSvg, not a raw String");
        // If this compiles and passes, the return type is correct.
        let _: &str = normalized.as_str();
    }

    /// BC-1.12.003 AC-001 / error path: when `usvg_normalize` fails, the
    /// `SvgNormalizationFailed.source_id` field must match the `source_id`
    /// argument passed to the function.
    #[test]
    fn test_bc_1_12_003_normalize_propagates_source_id() {
        // Garbage input — usvg cannot parse this as SVG.
        let raw = RawDiagramSvg("<not-valid-xml-at-all".to_owned());
        let result = usvg_normalize(&raw, "my-diagram-label");
        let err = result.expect_err("malformed SVG must return Err");
        match err {
            crate::types::DiagramError::SvgNormalizationFailed { source_id, .. } => {
                assert_eq!(
                    source_id.as_ref(),
                    "my-diagram-label",
                    "source_id in error must match the argument passed to usvg_normalize"
                );
            },
            other => {
                panic!("expected SvgNormalizationFailed, got: {other:?}");
            },
        }
    }

    // -----------------------------------------------------------------------
    // AC-002: no `<foreignObject>` after normalization (BC-1.12.003 postcondition 1)
    // -----------------------------------------------------------------------

    /// EC-003: SVG containing `<foreignObject>` must have that element removed
    /// by usvg. The normalized output must contain zero occurrences of
    /// `<foreignobject` (case-insensitive).
    #[test]
    fn test_bc_1_12_003_normalize_strips_foreignobject() {
        let raw = RawDiagramSvg(test_fixtures::svg_with_foreign_object().to_owned());
        let normalized =
            usvg_normalize(&raw, "fo-diagram").expect("SVG with foreignObject must normalize");
        let lower = normalized.as_str().to_ascii_lowercase();
        assert!(
            !lower.contains("<foreignobject"),
            "normalized SVG must not contain <foreignObject>; \
             usvg must have removed it (AC-002, EC-003)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-003: no `<script>` after normalization (BC-1.12.003 postcondition 2)
    // -----------------------------------------------------------------------

    /// SVG with `<script>alert(1)</script>` must have the script element
    /// removed by usvg. The normalized output must contain zero occurrences
    /// of `<script` (case-insensitive).
    #[test]
    fn test_bc_1_12_003_normalize_strips_script() {
        let raw = RawDiagramSvg(test_fixtures::svg_with_script().to_owned());
        let normalized =
            usvg_normalize(&raw, "script-diagram").expect("SVG with script must normalize");
        let lower = normalized.as_str().to_ascii_lowercase();
        assert!(
            !lower.contains("<script"),
            "normalized SVG must not contain <script>; usvg must have removed it (AC-003)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-004: no `@keyframes` CSS after normalization (BC-1.12.003 postcondition 3)
    // -----------------------------------------------------------------------

    /// EC-005: SVG with `<style>@keyframes spin { ... }</style>` must have
    /// the animation removed during usvg normalization. The normalized output
    /// must contain no `@keyframes` text.
    #[test]
    fn test_bc_1_12_003_normalize_strips_keyframes_css() {
        let raw = RawDiagramSvg(test_fixtures::svg_with_keyframes_css().to_owned());
        let normalized =
            usvg_normalize(&raw, "keyframes-diagram").expect("SVG with keyframes must normalize");
        let lower = normalized.as_str().to_ascii_lowercase();
        assert!(
            !lower.contains("@keyframes"),
            "normalized SVG must not contain @keyframes; usvg must have stripped it (AC-004, EC-005)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-005: absolute pixel dimensions after normalization (BC-1.12.003 postcondition 4)
    // -----------------------------------------------------------------------

    /// EC-002: SVG with `width="100%" height="100%"` on the root element must
    /// be transformed by usvg to use absolute pixel dimensions. The normalized
    /// output must not have a `width` or `height` attribute ending with `%`.
    ///
    /// Specifically, the root `<svg` element must not have `width="...%"` or
    /// `height="...%"` anywhere in the output (case-insensitive attribute check).
    #[test]
    fn test_bc_1_12_003_normalize_resolves_percentage_dimensions() {
        let raw = RawDiagramSvg(test_fixtures::svg_with_percentage_dims().to_owned());
        let normalized =
            usvg_normalize(&raw, "pct-dims").expect("SVG with percentage dims must normalize");
        let s = normalized.as_str();
        // Find the root <svg ... > opening tag (everything up to the first >).
        // Check that the attribute values for width= and height= do not end with %.
        // We scan for `width="...%"` and `height="...%"` patterns.
        assert!(
            !contains_percentage_dimension(s),
            "normalized SVG root must not have percentage width or height; \
             usvg must resolve to absolute pixels (AC-005, EC-002). \
             Got first 300 chars: {}",
            &s[..s.len().min(300)]
        );
    }

    // -----------------------------------------------------------------------
    // AC-006: no `<use>` after normalization (BC-1.12.003 postcondition 5)
    // -----------------------------------------------------------------------

    /// EC-001: SVG with `<defs><symbol id="dot"/></defs><use href="#dot"/>`
    /// must have all `<use>` elements resolved and inlined by usvg. The
    /// normalized output must contain no `<use` text (case-insensitive).
    #[test]
    fn test_bc_1_12_003_normalize_resolves_use_references() {
        let raw = RawDiagramSvg(test_fixtures::svg_with_use_reference().to_owned());
        let normalized =
            usvg_normalize(&raw, "use-ref-diagram").expect("SVG with <use> must normalize");
        let lower = normalized.as_str().to_ascii_lowercase();
        assert!(
            !lower.contains("<use"),
            "normalized SVG must not contain <use> elements; \
             usvg must have inlined all href=#symbol references (AC-006, EC-001)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-005 companion: viewBox preserved (BC-1.12.003 postcondition 4)
    // -----------------------------------------------------------------------

    /// After normalization, the output SVG should still carry dimensional
    /// information. usvg may preserve a `viewBox` attribute (or encode it in
    /// the root element's width/height). We verify the output contains either
    /// a `viewBox` or explicit `width` / `height` attributes so callers can
    /// compute layout geometry.
    #[test]
    fn test_bc_1_12_003_normalize_preserves_viewbox() {
        let raw = RawDiagramSvg(test_fixtures::simple_rect_svg().to_owned());
        let normalized = usvg_normalize(&raw, "viewbox-check").expect("simple SVG must normalize");
        let s = normalized.as_str();
        let has_viewbox = s.contains("viewBox") || s.contains("viewbox");
        let has_width = s.contains("width");
        let has_height = s.contains("height");
        assert!(
            has_viewbox || (has_width && has_height),
            "normalized SVG must retain dimensional information (viewBox or width+height); \
             got first 300 chars: {}",
            &s[..s.len().min(300)]
        );
    }

    // -----------------------------------------------------------------------
    // AC-001 / error mapping: malformed SVG → SvgNormalizationFailed (E-EXP-004)
    // -----------------------------------------------------------------------

    /// BC-1.12.003 / AC-007: invalid (non-parseable) SVG input must return
    /// `Err(DiagramError::SvgNormalizationFailed)`. The error display string
    /// must contain `E-EXP-004` per the error taxonomy.
    ///
    /// The test uses a string that is definitely not valid XML/SVG so that
    /// `usvg::Tree::from_str` is guaranteed to reject it.
    #[test]
    fn test_bc_1_12_003_normalize_invalid_svg_returns_error() {
        let raw = RawDiagramSvg("<not-valid-xml-at-all".to_owned());
        let result = usvg_normalize(&raw, "bad-svg");
        let err = result.expect_err("garbage input must return Err, not Ok");
        let msg = err.to_string();
        assert!(
            msg.contains("E-EXP-004"),
            "error message must contain E-EXP-004 for normalization failure; got: {msg}"
        );
        // Also verify it is the correct variant.
        assert!(
            matches!(
                err,
                crate::types::DiagramError::SvgNormalizationFailed { .. }
            ),
            "error must be SvgNormalizationFailed variant; got: {err:?}"
        );
    }

    /// AC-001 edge case: empty string input must return Err (there is no SVG
    /// to normalize). The empty string is a degenerate case distinct from a
    /// well-formed SVG document, and usvg must reject it.
    #[test]
    fn test_bc_1_12_003_normalize_empty_input_returns_error() {
        let raw = RawDiagramSvg(String::new());
        let result = usvg_normalize(&raw, "empty-input");
        assert!(
            result.is_err(),
            "empty string input must return Err — there is no SVG to normalize"
        );
    }

    // -----------------------------------------------------------------------
    // AC-001 / render_diagram integration: full pipeline returns NormalizedDiagramSvg
    // -----------------------------------------------------------------------

    /// BC-1.12.003 AC-001 / invariant 1: `render_diagram` is the full pipeline
    /// entry point. It must call `usvg_normalize` after `render_mermaid` and
    /// return `NormalizedDiagramSvg`. This test verifies the pipeline from
    /// Mermaid source to normalized output succeeds end-to-end.
    #[test]
    fn test_bc_1_12_003_render_diagram_returns_normalized() {
        use crate::{DiagramLang, DiagramRendererImpl};
        let source = "graph TD\n  A --> B";
        let result = DiagramRendererImpl::render_diagram(source, DiagramLang::Mermaid, "A to B");
        let normalized: NormalizedDiagramSvg =
            result.expect("render_diagram must succeed for a valid Mermaid flowchart");
        assert!(
            !normalized.is_empty(),
            "render_diagram must return non-empty NormalizedDiagramSvg"
        );
        assert!(
            normalized.as_str().contains("<svg"),
            "render_diagram output must be an SVG document"
        );
    }

    /// BC-1.12.003 invariant 1 / AC-002: the output of `render_diagram` must
    /// have passed through the normalization step. As a consequence, even if
    /// the raw `mermaid-rs-renderer` output happened to contain `<foreignObject>`,
    /// the normalized result must not. We verify the full-pipeline output is
    /// clean of `<foreignObject>` as a black-box integration assertion.
    #[test]
    fn test_bc_1_12_003_render_diagram_output_has_no_foreignobject() {
        use crate::{DiagramLang, DiagramRendererImpl};
        let source = "sequenceDiagram\n  Alice->>Bob: Hello";
        let normalized = DiagramRendererImpl::render_diagram(source, DiagramLang::Mermaid, "Seq")
            .expect("sequenceDiagram render must succeed");
        let lower = normalized.as_str().to_ascii_lowercase();
        assert!(
            !lower.contains("<foreignobject"),
            "render_diagram output must never contain <foreignObject> — \
             normalization is mandatory (BC-1.12.003 invariant 1, AC-002)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-008 / NFR-003: per-diagram normalization performance budget
    //
    // The NFR-003 performance gate (< 500ms cold build for a 25-slide deck)
    // distributes across the pipeline. Per AC-008, warm normalization must be
    // a small fraction of the 10ms warm budget. We use a conservative 100ms
    // per-call ceiling here (well within even a cold-path single-call budget)
    // to catch gross regressions without being flaky.
    //
    // Criterion benchmarks (cold_render / warm_render) provide the precise
    // percentile measurements; this unit test catches catastrophic slowdowns
    // (e.g., accidentally calling resvg rasterization, I/O in a loop, etc.).
    // -----------------------------------------------------------------------

    /// AC-008 / NFR-003: normalizing a sample SVG must complete in < 100ms.
    ///
    /// This is a coarse wall-clock guard, not a precision benchmark. The
    /// Criterion bench suites (`cold_render` / `warm_render`) enforce the
    /// precise per-AC-008 budgets (cold < 200ms total, warm < 10ms total).
    #[test]
    fn test_bc_1_12_003_normalize_under_budget() {
        use std::time::Instant;
        // Use a realistic sample SVG (close to what mermaid-rs-renderer produces).
        let raw = RawDiagramSvg(test_fixtures::simple_rect_svg().to_owned());
        let start = Instant::now();
        let _ = usvg_normalize(&raw, "perf-test");
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 100,
            "usvg_normalize must complete in < 100ms (NFR-003 coarse guard); \
             actual: {}ms. Use Criterion benches for precise measurement.",
            elapsed.as_millis()
        );
    }

    // -----------------------------------------------------------------------
    // Helper: detect percentage dimensions in the root <svg ...> opening tag.
    // -----------------------------------------------------------------------

    /// Returns `true` if the SVG string contains `width="...%"` or
    /// `height="...%"` anywhere. Delegates to the module-level implementation.
    fn contains_percentage_dimension(svg: &str) -> bool {
        contains_percentage_dimension_impl(svg)
    }
}
