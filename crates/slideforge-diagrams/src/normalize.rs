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
//! ## Implementation note
//!
//! The implementation is a `todo!()` stub. The full implementation will:
//! 1. Call `usvg::Tree::from_str(raw_svg, &usvg::Options::default())` to parse.
//! 2. Call `usvg::TreeWriting::to_string(&tree)` to re-serialize.
//! 3. Wrap the result in [`NormalizedDiagramSvg`].
//! 4. Run post-normalization debug assertions (AC-002 through AC-006).

use tracing::instrument;

use crate::types::{DiagramError, NormalizedDiagramSvg, RawDiagramSvg};

/// Normalize a [`RawDiagramSvg`] into a PPTX-safe [`NormalizedDiagramSvg`]
/// using `usvg` 0.47.0.
///
/// This is a **pure, synchronous** operation — no I/O, no network access.
/// It parses the raw SVG through `usvg::Tree::from_str` and re-serializes
/// via `usvg::TreeWriting::to_string`. usvg guarantees the following
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
    // STORY-034: Full implementation.
    // Steps:
    // 1. Call usvg::Tree::from_str(raw.as_str(), &usvg::Options::default())
    // 2. Call usvg::TreeWriting::to_string(&tree)
    // 3. Wrap in NormalizedDiagramSvg(Arc::from(normalized_str))
    // 4. Call debug_assert_post_normalization(&normalized_str) in debug builds
    //
    // Error path: if usvg::Tree::from_str returns Err, return
    // DiagramError::SvgNormalizationFailed { source_id, cause, span }.
    //
    // The `raw` parameter and `source_id` are consumed in the real
    // implementation. They are referenced here to suppress dead-code warnings
    // on the parameter and to ensure the field is captured by `instrument`.
    let _ = (raw, source_id);
    todo!(
        "STORY-034: implement usvg_normalize — \
         parse raw SVG via usvg::Tree::from_str, re-serialize with \
         usvg::TreeWriting::to_string, wrap in NormalizedDiagramSvg, \
         and run post-normalization debug assertions (AC-002 through AC-006)."
    )
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
#[allow(dead_code)] // Called from usvg_normalize in debug builds once implemented.
fn debug_assert_post_normalization(_normalized_svg: &str) {
    todo!(
        "STORY-034: implement debug_assert_post_normalization — \
         check that normalized_svg does not contain <foreignObject>, \
         <script>, @keyframes, percentage width/height, or <use> elements. \
         Each violation is a debug_assert!(false, ...) panic. \
         This function is only called in debug builds."
    )
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
        let raw = RawDiagramSvg("<svg/>".to_owned());
        let result = std::panic::catch_unwind(|| usvg_normalize(&raw, "test-diagram"));
        assert!(
            result.is_err(),
            "stub must panic with todo!() — Red Gate not satisfied"
        );
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
        assert!(
            !normalized.is_empty(),
            "normalized SVG must not be empty"
        );
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
                panic!(
                    "expected SvgNormalizationFailed, got: {other:?}"
                );
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
        let normalized =
            usvg_normalize(&raw, "viewbox-check").expect("simple SVG must normalize");
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
    /// `height="...%"` anywhere (a proxy for the root element having
    /// percentage dimensions). Used by `test_bc_1_12_003_normalize_resolves_percentage_dimensions`.
    ///
    /// The check is conservative: it scans the entire string rather than
    /// only the root element, which is safe because usvg-emitted SVGs do
    /// not use percentage dimensions on inner elements.
    fn contains_percentage_dimension(svg: &str) -> bool {
        // Match patterns like: width="100%", width='50%', height="100%"
        let check = |attr: &str| -> bool {
            let mut search = svg;
            let needle = attr;
            while let Some(pos) = search.find(needle) {
                let after = &search[pos + needle.len()..];
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
}
