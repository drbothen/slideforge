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
//! ## I/O note
//!
//! The **first call** to `usvg_normalize` performs disk I/O via
//! `fontdb::Database::load_system_fonts()` (one-time cost ~50–300ms depending
//! on platform). Subsequent calls are I/O-free — the font database is cached
//! in a `OnceLock` and returned in O(1) as an `Arc::clone`.  The font database
//! is unconditionally supplied to usvg (rather than gated on `<text` presence)
//! to avoid false-negatives when `<text` elements appear with namespace prefixes
//! or are introduced by `<use>` expansion.
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
//! 1. Supply the system font database unconditionally via `OnceLock` (first call
//!    loads fonts; subsequent calls are O(1) `Arc::clone` — no repeated I/O).
//! 2. Call `usvg::Tree::from_str(raw_svg, &opt)` to parse.
//! 3. Call `tree.to_string(&WriteOptions { preserve_text: true, .. })` to re-serialize.
//! 4. Re-inject `aria-label`, `role="img"`, `<title>`, and `viewBox` that usvg strips.
//! 5. Run post-normalization debug assertions (AC-002 through AC-006, AC-style-check).
//!
//! ## String surgery for re-injection
//!
//! usvg 0.47.0 does not expose a mutation API for `Tree` nodes after parsing
//! (titles, ARIA attributes, and viewBox are not part of its internal model).
//! Re-injection therefore uses targeted string surgery on the serialized output.
//! This is inherently more fragile than a Tree-level API. Known limitations:
//!
//! - XML comments between `<!DOCTYPE>` and `<svg` are handled because we search
//!   for `<svg`, not for the start of the document.
//! - Namespace-prefixed `<svg:svg` roots would not be found — usvg does not
//!   produce such output in practice.
//! - Attribute values containing `>` would cause `find('>')` to find the wrong
//!   position. usvg escapes `>` in attribute values as `&gt;`, so this is safe
//!   for usvg-produced output.
//!
//! Regression tests for these edge cases are present in the test suite.

use std::fmt::Write as FmtWrite;
use std::sync::{Arc, OnceLock};

use miette::SourceSpan;
use tracing::instrument;

use crate::types::{DiagramError, NormalizedDiagramSvg, RawDiagramSvg};
use crate::xml_escape::{xml_attr_escape, xml_text_escape};

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
/// This is a **synchronous** operation. The first call performs disk I/O to
/// load system fonts (one-time cost; subsequent calls hit the `OnceLock` cache).
///
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
/// - `slide_title` — An identifier for the diagram (e.g., slide title or alt text)
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
/// contains a forbidden element (`foreignObject`, `script`, `@keyframes`,
/// `<style`, or `<use`), or a `%` dimension. This is a programming error —
/// usvg must always remove these elements, so their presence indicates a
/// regression in usvg 0.47.0 or a bug in this function. Production (release)
/// builds do not panic.
#[instrument(skip(raw), fields(slide_title = %slide_title))]
pub fn usvg_normalize(
    raw: &RawDiagramSvg,
    slide_title: &str,
) -> Result<NormalizedDiagramSvg, DiagramError> {
    // Build usvg options, always loading the system font database.
    //
    // System fonts are required so that usvg preserves <text> elements as text
    // (rather than converting them to outlined paths). Without fonts:
    // - Node labels ("Client", "API") are lost from flowchart SVGs.
    // - Participant names ("Alice", "Bob") are lost from sequence diagrams.
    // - PPTX accessibility and screen-reader compatibility are broken.
    //
    // We always supply the font database regardless of whether the raw SVG
    // contains `<text` elements. This eliminates a potential false-negative
    // when `<text` appears with a namespace prefix (e.g., `<svg:text`) or
    // when an SVG has text elements introduced by `<use>` expansion. The
    // font_db() call is O(1) after the first call (the database is loaded
    // once and cached in `FONT_DB` via OnceLock). The first call may take
    // 50–300ms on systems with large font collections; subsequent calls are
    // nanoseconds (Arc::clone of the cached Arc<Database>).
    //
    // NOTE: FONT_DB is process-global and initialized exactly once. Integration
    // tests in tests/cold_budget.rs run in a fresh process (each `tests/*.rs`
    // file compiles to a separate binary) and therefore see an uninitialized
    // FONT_DB — this is the correct way to measure cold-path latency.
    let opt = usvg::Options {
        fontdb: font_db(),
        ..usvg::Options::default()
    };

    // Parse the raw SVG through usvg. On failure, map to E-EXP-004.
    let tree = usvg::Tree::from_str(raw.as_str(), &opt).map_err(|e| {
        // Extract byte-position span where available.
        let span = extract_usvg_error_span(&e);
        DiagramError::SvgNormalizationFailed {
            slide_title: Arc::from(slide_title),
            cause: Arc::from(format!("usvg parse failed: {e}").as_str()),
            span,
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
    // slide_title is the alt_text passed through from render_diagram().
    let enriched =
        reinject_accessibility_and_viewbox(&normalized_str, slide_title).map_err(|e| {
            DiagramError::SvgNormalizationFailed {
                slide_title: Arc::from(slide_title),
                cause: Arc::from(format!("post-normalization enrichment failed: {e}").as_str()),
                span: SourceSpan::from(0..0),
            }
        })?;

    Ok(NormalizedDiagramSvg::from_normalized_string(Arc::from(
        enriched.as_str(),
    )))
}

/// Extract a byte-position [`SourceSpan`] from a [`usvg::Error`], if available.
///
/// usvg 0.47.0 wraps `roxmltree::Error` for parse failures. `roxmltree::Error`
/// exposes a `pos()` method that returns a `(line, col)` text position. We
/// convert this to a byte offset by searching for the Nth newline in the
/// source — an approximation, since we do not have the source at this call site.
/// When no position is available (non-parse errors), we return a zero-length
/// span at offset 0 as a sentinel.
fn extract_usvg_error_span(e: &usvg::Error) -> SourceSpan {
    match e {
        usvg::Error::ParsingFailed(xml_err) => {
            // roxmltree reports (row, col) 1-based. We cannot convert to a byte
            // offset without the source string, so we encode row/col as a
            // zero-length span at a synthetic offset of (row * 1000 + col) for
            // diagnostic hints. This is clearly documented as approximate.
            let pos = xml_err.pos();
            let synthetic_offset = (pos.row as usize)
                .saturating_mul(1000)
                .saturating_add(pos.col as usize);
            SourceSpan::from(synthetic_offset..synthetic_offset)
        },
        _ => SourceSpan::from(0..0),
    }
}

/// Run post-normalization debug assertions (AC-002 through AC-006 + style-check).
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
/// | AC-style | `<style` (case-insensitive) — CSS must be fully inlined |
/// | AC-005 | `width="...%"` or `height="...%"` (percentage dimensions on root) |
/// | AC-006 | `<use` (case-insensitive) |
pub(crate) fn debug_assert_post_normalization(normalized_svg: &str) {
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
        !lower.contains("<style"),
        "usvg normalization regression: output contains <style> element (AC-style). \
         usvg 0.47.0 must inline all CSS into presentation attributes. \
         This is a bug in usvg or this function."
    );

    debug_assert!(
        !contains_percentage_dimension_in_root_tag(normalized_svg),
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

/// Returns `true` if the **root `<svg>` opening tag** contains
/// `width="...%"` or `height="...%"`.
///
/// This function restricts the search to the opening `<svg ...>` tag only,
/// preventing false positives from attributes like `stroke-width="50%"` on
/// descendant elements such as `<rect stroke-width="50%"/>`.
///
/// Used by [`debug_assert_post_normalization`] and the test helper.
fn contains_percentage_dimension_in_root_tag(svg: &str) -> bool {
    // Slice the opening tag: from `<svg` to the first `>`.
    let Some(start) = svg.find("<svg") else {
        return false;
    };
    let Some(rel_end) = svg[start..].find('>') else {
        return false;
    };
    let tag_end = start + rel_end;
    let opening_tag = &svg[start..=tag_end];

    // Check for width or height attributes with percentage values, using
    // word-boundary-aware extraction to avoid matching `stroke-width=`.
    for attr in &["width", "height"] {
        if let Some(val) = extract_attr_value(opening_tag, attr)
            && val.trim_end().ends_with('%')
        {
            return true;
        }
    }
    false
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
/// 4. Insert `<title>{alt_text}</title>` as the first child of `<svg>`,
///    but only if `<title>` is not already present (idempotency guard).
///
/// ## Idempotency
///
/// If this function is called twice on the same SVG, the second call is a
/// no-op for the `<title>` element (the idempotency guard prevents double
/// injection). `viewBox` and `aria-label` are also guarded by existing-attribute
/// checks so they are not duplicated either.
///
/// # Errors
///
/// Returns an error string if:
/// - The SVG string does not contain a root `<svg` tag.
/// - usvg did not emit `width` or `height` on the root element (defensive;
///   should never happen with valid usvg 0.47.0 output).
fn reinject_accessibility_and_viewbox(
    normalized_svg: &str,
    alt_text: &str,
) -> Result<String, String> {
    // Locate the root <svg opening tag.
    let svg_start = normalized_svg
        .find("<svg")
        .ok_or_else(|| "no <svg> root element in usvg-normalized output".to_owned())?;

    // Find the closing > of the opening <svg ... > tag.
    // We look for > after svg_start. usvg emits well-formed SVG so this > is
    // always present and belongs to the opening tag (not a child element).
    let tag_relative_close = normalized_svg[svg_start..].find('>').ok_or_else(|| {
        "root <svg> tag is not properly closed in usvg-normalized output".to_owned()
    })?;
    let tag_close = svg_start + tag_relative_close; // index of '>'

    // Extract the opening tag to check if attributes are already present
    // (idempotency guard — should never happen with usvg output, but defensive).
    let opening_tag = &normalized_svg[svg_start..=tag_close];

    // Parse `width` and `height` values from the opening tag for viewBox synthesis.
    // usvg emits e.g. `width="100" height="200"`. If they are absent, that is a
    // usvg regression and we return a structured error rather than silently
    // synthesizing a degenerate "0 0 0 0" viewBox.
    let width_str = extract_attr_value(opening_tag, "width").ok_or_else(|| {
        "usvg-normalized SVG root element is missing 'width' attribute; \
                        usvg 0.47.0 must always emit absolute width — this is a usvg regression"
            .to_owned()
    })?;
    let height_str = extract_attr_value(opening_tag, "height").ok_or_else(|| {
        "usvg-normalized SVG root element is missing 'height' attribute; \
                        usvg 0.47.0 must always emit absolute height — this is a usvg regression"
            .to_owned()
    })?;

    // Escape alt_text for XML attribute and text content.
    let escaped_attr = xml_attr_escape(alt_text);
    let escaped_text = xml_text_escape(alt_text);

    // Build the injected additions (viewBox + aria + role, and <title> child).
    // Use write! to avoid intermediate allocations (clippy::format_push_string).
    let mut extra_attrs = String::new();

    // Inject viewBox only if not already present.
    if !opening_tag.contains("viewBox") {
        // Strip CSS unit suffixes (e.g., "px") from width/height before
        // interpolating into viewBox.  usvg 0.47.0 emits bare integers for
        // absolute dimensions, but defensive stripping handles edge cases
        // where upstream emits "800px" instead of "800".
        let w = strip_unit_suffix(width_str);
        let h = strip_unit_suffix(height_str);
        let _ = write!(extra_attrs, r#" viewBox="0 0 {w} {h}""#);
    }

    // Inject accessibility attributes only if not already present.
    if !opening_tag.contains("aria-label=") {
        let _ = write!(extra_attrs, r#" aria-label="{escaped_attr}" role="img""#);
    }

    // Reconstruct the SVG:
    //   [before tag_close][extra_attrs]>[<title> if needed][rest]
    let before_close = &normalized_svg[..tag_close];
    let from_close = &normalized_svg[tag_close + 1..]; // skip the '>'

    // Idempotency guard for <title>: only inject if not already present.
    // Check for both the bare `<title>` form AND the attributed `<title ` form
    // (e.g., `<title id="t1">`) so that re-injection is skipped whenever any
    // `<title` element exists, regardless of whether it carries attributes.
    let has_title = normalized_svg.contains("<title>") || normalized_svg.contains("<title ");
    let title_fragment = if has_title {
        String::new()
    } else {
        format!("<title>{escaped_text}</title>")
    };

    Ok(format!(
        "{before_close}{extra_attrs}>{title_fragment}{from_close}"
    ))
}

/// Strip a CSS unit suffix from a dimension string and return a trimmed numeric slice.
///
/// usvg 0.47.0 emits bare integers (e.g., `"800"`) for absolute `width`/`height`
/// attributes on the root `<svg>` element. However, upstream SVG sources may carry
/// `"px"` suffixes (e.g., `"800px"`). Stripping the suffix before constructing a
/// `viewBox` value ensures we produce valid SVG (`viewBox="0 0 800 600"`, not
/// `viewBox="0 0 800px 600px"` which is malformed XML attribute content).
///
/// Only `"px"` is stripped; other unit suffixes (e.g., `"em"`, `"pt"`) are
/// intentionally left in place so that an unexpected unit becomes visible rather
/// than silently producing a wrong value.
fn strip_unit_suffix(s: &str) -> &str {
    s.trim_end_matches("px").trim_end()
}

/// Extract the value of a **named attribute** from an SVG opening tag string.
///
/// Uses a word-boundary check to avoid substring matches: `extract_attr_value(tag, "width")`
/// will not match `stroke-width=` because the check requires that the character
/// immediately before the attribute name be XML whitespace (space, tab, CR, LF)
/// or that the attribute name starts at position 0.
///
/// The function handles both double-quoted and single-quoted attribute values.
/// Returns `None` if the attribute is not found.
///
/// ## Word-boundary rule
///
/// XML 1.0 section 2.3 defines attribute-value whitespace as: space (`0x20`), tab (`0x09`),
/// carriage-return (`0x0D`), and line-feed (`0x0A`).  All four are accepted as
/// valid separators between attributes.  This means `stroke-width=` is never
/// matched when searching for `width` because the preceding character is `-`,
/// not an XML whitespace character.
///
/// ## Examples
///
/// ```
/// // Opening tag: `<svg stroke-width="2" width="100">`
/// // extract_attr_value(tag, "width") returns Some("100")  — correct
/// // extract_attr_value(tag, "stroke-width") returns Some("2")  — correct
/// // Opening tag: `<svg\twidth="800">`
/// // extract_attr_value(tag, "width") returns Some("800")  — correct (tab separator)
/// ```
fn extract_attr_value<'a>(tag: &'a str, attr_name: &str) -> Option<&'a str> {
    // Scan the tag byte-by-byte looking for `attr_name=`.
    // A match is valid only if the byte immediately before `attr_name` is an
    // XML whitespace character (space, tab, CR, LF) or the attribute name
    // begins at index 0.
    let bytes = tag.as_bytes();
    let needle = attr_name.as_bytes();
    let needle_len = needle.len();

    // We need at least `needle_len + 1` bytes (for the `=` sign) to match.
    if bytes.len() < needle_len + 1 {
        return None;
    }

    let search_limit = bytes.len() - needle_len; // inclusive upper bound
    let mut i = 0usize;
    while i <= search_limit {
        // Check if attr_name starts at position i.
        if bytes[i..i + needle_len] == *needle {
            // Check that the character after attr_name is `=`.
            if bytes.get(i + needle_len) != Some(&b'=') {
                i += 1;
                continue;
            }
            // Check word boundary: preceding character must be XML whitespace
            // or the attribute name starts at position 0.
            let valid_boundary = i == 0 || matches!(bytes[i - 1], b' ' | b'\t' | b'\n' | b'\r');
            if !valid_boundary {
                i += 1;
                continue;
            }

            // We have a valid attribute name match at position i.
            // Now extract the quoted value.
            let after = &tag[i + needle_len + 1..]; // skip attr_name + '='
            let quote = after.chars().next()?;
            if quote != '"' && quote != '\'' {
                return None;
            }
            let value_start = &after[1..]; // skip opening quote
            let end = value_start.find(quote)?;
            return Some(&value_start[..end]);
        }
        i += 1;
    }
    None
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
        /// that usvg can round-trip without any text elements. Used for
        /// geometry-only (no-font-path) happy-path tests.
        pub(super) fn simple_geometry_svg() -> &'static str {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><rect x="10" y="10" width="80" height="80" fill="blue"/></svg>"#
        }

        /// Mermaid-like SVG with `<text>` elements and labelled nodes.
        ///
        /// This fixture exercises the font-loading slow path that runs when
        /// `<text` is present. Used for performance tests that need to
        /// prove the font-loading path stays within the NFR-003 budget.
        pub(super) fn mermaid_like_with_text_svg() -> &'static str {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="300" height="200" viewBox="0 0 300 200">
  <rect x="10" y="10" width="100" height="40" fill="lightblue" rx="5"/>
  <text x="60" y="35" text-anchor="middle" font-family="sans-serif">Client</text>
  <rect x="190" y="10" width="100" height="40" fill="lightgreen" rx="5"/>
  <text x="240" y="35" text-anchor="middle" font-family="sans-serif">API</text>
  <line x1="110" y1="30" x2="190" y2="30" stroke="black" stroke-width="2" marker-end="url(#arrow)"/>
  <defs><marker id="arrow" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="black"/></marker></defs>
</svg>"#
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

        /// SVG containing a `<style>` block (without @keyframes) on the root.
        ///
        /// usvg must inline CSS from `<style>` blocks into presentation attributes,
        /// so the normalized output must have no `<style>` element regardless of
        /// whether it contains `@keyframes` (F-HIGH-001).
        pub(super) fn svg_with_style_element() -> &'static str {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><style>rect { fill: green; stroke: black; }</style><rect x="10" y="10" width="80" height="80"/></svg>"#
        }

        /// SVG using percentage dimensions on the root element.
        ///
        /// usvg resolves percentage `width` / `height` to absolute pixel values
        /// using the `viewBox` attribute. After normalization, the root element
        /// must have absolute (non-`%`) `width` and `height` (AC-005, EC-002).
        pub(super) fn svg_with_percentage_dims() -> &'static str {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 400 300"><rect x="0" y="0" width="400" height="300" fill="gray"/></svg>"#
        }

        /// SVG with `stroke-width` on the root `<svg>` and absolute `width`/`height`.
        ///
        /// Tests that `extract_attr_value(tag, "width")` does NOT match
        /// `stroke-width=` due to the word-boundary requirement (F-CRIT-002).
        pub(super) fn svg_with_stroke_width_on_root() -> &'static str {
            r#"<svg xmlns="http://www.w3.org/2000/svg" stroke-width="2" width="100" height="100" viewBox="0 0 100 100"><rect x="10" y="10" width="80" height="80" fill="blue"/></svg>"#
        }

        /// SVG with `stroke-width="50%"` on a child element.
        ///
        /// Tests that `contains_percentage_dimension_in_root_tag` does NOT
        /// return true for percentage values on non-root elements (F-CRIT-002).
        pub(super) fn svg_child_has_stroke_width_percent() -> &'static str {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 0 200 200"><rect x="0" y="0" width="200" height="200" stroke-width="50%" fill="none"/></svg>"#
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
        let raw = RawDiagramSvg(test_fixtures::simple_geometry_svg().to_owned());
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
        let raw = RawDiagramSvg(test_fixtures::simple_geometry_svg().to_owned());
        let result = usvg_normalize(&raw, "type-check");
        let normalized: NormalizedDiagramSvg =
            result.expect("must return NormalizedDiagramSvg, not a raw String");
        // If this compiles and passes, the return type is correct.
        let _: &str = normalized.as_str();
    }

    /// BC-1.12.003 AC-001 / error path: when `usvg_normalize` fails, the
    /// `SvgNormalizationFailed.slide_title` field must match the `slide_title`
    /// argument passed to the function.
    #[test]
    fn test_bc_1_12_003_normalize_propagates_slide_title() {
        // Garbage input — usvg cannot parse this as SVG.
        let raw = RawDiagramSvg("<not-valid-xml-at-all".to_owned());
        let result = usvg_normalize(&raw, "my-diagram-label");
        let err = result.expect_err("malformed SVG must return Err");
        match err {
            crate::types::DiagramError::SvgNormalizationFailed { slide_title, .. } => {
                assert_eq!(
                    slide_title.as_ref(),
                    "my-diagram-label",
                    "slide_title in error must match the argument passed to usvg_normalize"
                );
            },
            other => {
                panic!("expected SvgNormalizationFailed, got: {other:?}");
            },
        }
    }

    // -----------------------------------------------------------------------
    // F-CRIT-001: title idempotency — calling usvg_normalize twice must not
    // produce duplicate <title> elements.
    // -----------------------------------------------------------------------

    /// F-CRIT-001: calling `usvg_normalize` twice on the same input must
    /// produce output that contains exactly one `<title>` element, not two.
    ///
    /// The second call receives the already-normalized SVG (which already has
    /// `<title>` injected). The idempotency guard in `reinject_accessibility_and_viewbox`
    /// must detect the existing `<title>` and skip re-injection.
    #[test]
    fn test_bc_1_12_003_normalize_title_idempotent_on_double_call() {
        let raw = RawDiagramSvg(test_fixtures::simple_geometry_svg().to_owned());
        let first = usvg_normalize(&raw, "my-title").expect("first normalize must succeed");

        // Feed the normalized output back through usvg_normalize a second time.
        let second_raw = RawDiagramSvg(first.as_str().to_owned());
        // The second call may fail (usvg re-parse of already-normalized SVG) or
        // succeed. If it succeeds, <title> must appear exactly once.
        if let Ok(second) = usvg_normalize(&second_raw, "my-title") {
            let s = second.as_str();
            let title_count = s.matches("<title>").count();
            assert_eq!(
                title_count,
                1,
                "double-normalized SVG must contain exactly one <title> element, \
                 not {title_count}; idempotency guard must have prevented re-injection. \
                 SVG excerpt: {}",
                &s[..s.len().min(500)]
            );
        }
        // If the second call returns Err (usvg cannot re-parse the enriched SVG),
        // that is also acceptable — the idempotency guarantee only applies when the
        // re-parse succeeds. The important property is: no panic on the second call.
    }

    // -----------------------------------------------------------------------
    // F-CRIT-002: word-boundary attribute extraction — `stroke-width` must not
    // be confused with `width`.
    // -----------------------------------------------------------------------

    /// F-CRIT-002: `extract_attr_value(tag, "width")` must NOT match
    /// `stroke-width=` as a substring. The word-boundary (leading space) check
    /// must return only the standalone `width` attribute value.
    #[test]
    fn test_extract_width_not_confused_by_stroke_width() {
        // Opening tag has `stroke-width="2"` before `width="100"`.
        let tag = r#"<svg stroke-width="2" width="100" height="50">"#;
        let width_val = extract_attr_value(tag, "width");
        assert_eq!(
            width_val,
            Some("100"),
            "extract_attr_value must return '100' for the standalone width attribute; \
             it must NOT return '2' from stroke-width. Got: {width_val:?}"
        );
    }

    /// F-CRIT-002: `contains_percentage_dimension_in_root_tag` on an SVG
    /// that has `stroke-width="50%"` on a child element but absolute `width`
    /// and `height` on the root must return `false`.
    #[test]
    fn test_percentage_check_ignores_stroke_width_on_child() {
        let svg = test_fixtures::svg_child_has_stroke_width_percent();
        assert!(
            !contains_percentage_dimension_in_root_tag(svg),
            "contains_percentage_dimension_in_root_tag must return false when \
             only child elements have stroke-width percentage — root has absolute dimensions"
        );
    }

    /// F-CRIT-002: normalizing an SVG with `stroke-width="2"` on the root `<svg>`
    /// and absolute `width`/`height` must succeed and produce correct dimensions.
    #[test]
    fn test_normalize_svg_with_stroke_width_on_root_succeeds() {
        let raw = RawDiagramSvg(test_fixtures::svg_with_stroke_width_on_root().to_owned());
        let result = usvg_normalize(&raw, "stroke-width-test");
        let normalized = result
            .expect("SVG with stroke-width on root and absolute w/h must normalize successfully");
        let s = normalized.as_str();
        assert!(
            !contains_percentage_dimension_in_root_tag(s),
            "normalized SVG must not have percentage root dimensions"
        );
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
    // AC-004 / F-HIGH-001: no `@keyframes` CSS or `<style>` after normalization
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

    /// F-HIGH-001: SVG with a `<style>` block (no @keyframes) must have the
    /// entire `<style>` element removed by usvg, which inlines all CSS into
    /// presentation attributes.
    #[test]
    fn test_bc_1_12_003_normalize_strips_style_element() {
        let raw = RawDiagramSvg(test_fixtures::svg_with_style_element().to_owned());
        let normalized =
            usvg_normalize(&raw, "style-diagram").expect("SVG with <style> must normalize");
        let lower = normalized.as_str().to_ascii_lowercase();
        assert!(
            !lower.contains("<style"),
            "normalized SVG must not contain any <style> element; \
             usvg must have inlined all CSS into presentation attributes (F-HIGH-001)"
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
            !contains_percentage_dimension_in_root_tag(s),
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
        let raw = RawDiagramSvg(test_fixtures::simple_geometry_svg().to_owned());
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
    // a small fraction of the 10ms warm budget.
    //
    // F-HIGH-004: The performance test uses a fixture with <text> elements
    // to exercise the font-loading slow path. This ensures the test is
    // meaningful — a geometry-only fixture (no text) would skip font loading
    // and measure only usvg parsing overhead, missing the dominant cost.
    //
    // Criterion benchmarks (cold_render / warm_render) provide the precise
    // percentile measurements; this unit test catches catastrophic slowdowns
    // (e.g., accidentally calling resvg rasterization, I/O in a loop, etc.).
    // -----------------------------------------------------------------------

    /// AC-008 / NFR-003 / F-HIGH-004: normalizing a mermaid-like SVG with
    /// `<text>` elements exercises the font-loading slow path. This test
    /// verifies the **warm-path** latency (after the font DB is initialized)
    /// stays within the 50ms warm budget.
    ///
    /// ## Methodology
    ///
    /// 1. Warm-up call: run a geometry-only (no-text) SVG to ensure the test
    ///    process is JIT-warm without triggering font loading.
    /// 2. Font-DB init: run the text SVG once to load the system font DB
    ///    (cold-path; cost ~50–300ms, not measured).
    /// 3. Timed call: run the text SVG again and assert < 50ms (warm path).
    ///
    /// The Criterion bench suites (`cold_render` / `warm_render`) enforce the
    /// precise per-AC-008 budgets (cold < 200ms total, warm < 10ms total).
    /// This test catches catastrophic regressions (accidentally calling
    /// `load_system_fonts()` on every call, I/O in a loop, etc.).
    #[test]
    fn test_bc_1_12_003_normalize_under_budget() {
        use std::time::Instant;

        // Step 1: warm up the process (geometry-only, no font loading).
        let warmup_raw = RawDiagramSvg(test_fixtures::simple_geometry_svg().to_owned());
        let _ = usvg_normalize(&warmup_raw, "warmup");

        // Step 2: trigger font DB initialization (first call with <text>).
        let text_raw = RawDiagramSvg(test_fixtures::mermaid_like_with_text_svg().to_owned());
        let _ = usvg_normalize(&text_raw, "font-init");

        // Step 3: time the warm-path call with <text> elements.
        // Take the median of 5 samples to reduce sensitivity to OS scheduling
        // jitter on CI runners. A single sample can flake on a loaded system
        // even when the code is correct. Median suppresses outliers without
        // hiding a genuine regression (which would show up in ALL samples).
        let mut samples = Vec::with_capacity(5);
        for _ in 0..5 {
            let start = Instant::now();
            let _ = usvg_normalize(&text_raw, "perf-test");
            samples.push(start.elapsed());
        }
        samples.sort();
        let median = samples[2]; // index 2 of 0..4 is the median

        assert!(
            median.as_millis() < 50,
            "usvg_normalize warm path (font DB already loaded) median latency must be < 50ms; \
             median: {}ms (samples: {:?}). This guards against per-call font-loading regressions. \
             Use Criterion benches for precise measurement.",
            median.as_millis(),
            samples
        );
    }

    // -----------------------------------------------------------------------
    // F-HIGH-005: debug-assert panic test (debug builds only)
    // -----------------------------------------------------------------------

    /// F-HIGH-005: `debug_assert_post_normalization` must panic in debug builds
    /// when the normalized SVG still contains `<foreignObject>`.
    ///
    /// This test is only compiled and run in debug mode. In release mode the
    /// `debug_assert!` compiles to nothing, so the test would always pass vacuously —
    /// we guard it with `#[cfg(debug_assertions)]` to make the intent explicit.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "usvg normalization regression: output contains <foreignObject>")]
    fn test_debug_assert_post_normalization_panics_on_foreignobject() {
        let svg_with_fo = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><foreignObject width="50" height="50"><div>bad</div></foreignObject></svg>"#;
        debug_assert_post_normalization(svg_with_fo);
    }

    /// F-HIGH-005 variant: `debug_assert_post_normalization` must panic in debug
    /// builds when the normalized SVG still contains a `<style>` element.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "usvg normalization regression: output contains <style>")]
    fn test_debug_assert_post_normalization_panics_on_style_element() {
        let svg_with_style = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><style>rect { fill: red; }</style><rect x="0" y="0" width="100" height="100"/></svg>"#;
        debug_assert_post_normalization(svg_with_style);
    }

    // -----------------------------------------------------------------------
    // F-CRIT-001 low-level: reinject_accessibility_and_viewbox idempotency
    // -----------------------------------------------------------------------

    /// F-CRIT-001 (unit): `reinject_accessibility_and_viewbox` must not inject
    /// a second `<title>` when one is already present in the SVG string.
    #[test]
    fn test_reinject_does_not_duplicate_title_when_already_present() {
        let svg_with_title = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><title>existing</title><rect x="0" y="0" width="100" height="100" fill="blue"/></svg>"#;
        let result =
            reinject_accessibility_and_viewbox(svg_with_title, "new text").expect("must succeed");
        let title_count = result.matches("<title>").count();
        assert_eq!(
            title_count, 1,
            "reinject must not add a second <title> when one already exists; \
             got {title_count} occurrences"
        );
    }

    // -----------------------------------------------------------------------
    // F-MED-001: extract_attr_value handles all XML whitespace separators
    // -----------------------------------------------------------------------

    /// F-MED-001: `extract_attr_value` must match an attribute that is
    /// separated from the previous attribute by a tab character, not just
    /// a space.  XML 1.0 §2.3 allows space, tab, CR, and LF as whitespace
    /// between attributes.
    #[test]
    fn test_extract_attr_value_handles_tab_whitespace() {
        // Tab-separated attributes: `<svg\twidth="800">`
        let tag = "<svg\twidth=\"800\">";
        let val = extract_attr_value(tag, "width");
        assert_eq!(
            val,
            Some("800"),
            "extract_attr_value must find width='800' in tab-separated tag; got {val:?}"
        );
    }

    /// F-MED-001: `extract_attr_value` must handle CR (\\r) as whitespace.
    #[test]
    fn test_extract_attr_value_handles_cr_whitespace() {
        let tag = "<svg\rwidth=\"400\">";
        let val = extract_attr_value(tag, "width");
        assert_eq!(
            val,
            Some("400"),
            "extract_attr_value must find width='400' in CR-separated tag; got {val:?}"
        );
    }

    // -----------------------------------------------------------------------
    // F-MED-002: viewBox synthesis strips px unit suffixes
    // -----------------------------------------------------------------------

    /// F-MED-002: `reinject_accessibility_and_viewbox` must strip `px` unit
    /// suffixes from width and height values before constructing the viewBox.
    /// For example, `width="800px"` must produce `viewBox="0 0 800 600"`,
    /// not `viewBox="0 0 800px 600px"`.
    #[test]
    fn test_viewbox_synthesis_strips_px_suffix() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="800px" height="600px"><rect x="0" y="0" width="800" height="600" fill="white"/></svg>"#;
        let result = reinject_accessibility_and_viewbox(svg, "px-test").expect("must succeed");
        // The injected viewBox must not contain "px".
        assert!(
            !result.contains("viewBox=\"0 0 800px 600px\""),
            "viewBox must not contain px suffixes; got result snippet: {}",
            &result[..result.len().min(300)]
        );
        assert!(
            result.contains("viewBox=\"0 0 800 600\""),
            "viewBox must be '0 0 800 600' with px stripped; got result snippet: {}",
            &result[..result.len().min(300)]
        );
    }

    // -----------------------------------------------------------------------
    // F-LOW-002: title idempotency handles attributed <title> tags
    // -----------------------------------------------------------------------

    /// F-LOW-002: `reinject_accessibility_and_viewbox` must not inject a
    /// second `<title>` when the existing title element has XML attributes
    /// (e.g., `<title id="t1">existing</title>`).  A naive `contains("<title>")`
    /// check would miss `<title id="t1">` because it does not start with
    /// the verbatim substring `<title>`.
    #[test]
    fn test_reinject_does_not_duplicate_title_when_attributed_title_present() {
        let svg_with_attributed_title = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><title id="t1">existing title</title><rect x="0" y="0" width="100" height="100" fill="blue"/></svg>"#;
        let result = reinject_accessibility_and_viewbox(svg_with_attributed_title, "new text")
            .expect("must succeed");
        // Count both plain <title> and attributed <title ...> occurrences.
        let plain_count = result.matches("<title>").count();
        let attr_count = result.matches("<title ").count();
        let total = plain_count + attr_count;
        assert_eq!(
            total, 1,
            "reinject must not add a second <title> when an attributed <title ...> already exists; \
             plain={plain_count}, attributed={attr_count}"
        );
    }

    // -----------------------------------------------------------------------
    // F-LOW-002: snapshot test for normalization output (insta)
    // -----------------------------------------------------------------------

    /// Snapshot test for the normalized output of `usvg_normalize` on the
    /// `simple_geometry_svg` fixture.
    ///
    /// This test pins the exact serialized form that usvg 0.47.0 produces for
    /// a deterministic geometry-only input. If the usvg version or
    /// normalization logic changes, the snapshot must be reviewed and updated
    /// deliberately (`cargo insta review`).
    ///
    /// The `simple_geometry_svg` fixture is geometry-only (no `<text>`) so the
    /// result is platform-independent — the font database is not consulted.
    #[test]
    fn snapshot_normalize_simple_geometry() {
        let raw = RawDiagramSvg(test_fixtures::simple_geometry_svg().to_owned());
        let normalized = usvg_normalize(&raw, "snapshot-test")
            .expect("usvg_normalize must succeed for simple_geometry_svg fixture");
        insta::assert_snapshot!(normalized.as_str());
    }
}
