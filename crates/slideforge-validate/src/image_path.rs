//! Image path traversal validator (SEC-001 / BC-1.16.001 EC-012 / CWE-22).
//!
//! [`ImagePathValidator`] rejects [`slideforge_types::specs::ImageSpec`] path
//! values that escape the source root directory. Three classes of traversal are
//! detected:
//!
//! 1. **Path-traversal segment** — path contains `..` as a segment.
//! 2. **Absolute path** — path starts with `/` or `\`.
//! 3. **Windows drive letter** — path starts with a drive letter prefix (e.g. `C:\`).
//!
//! Any violation produces exactly one [`E_VAL_012`] diagnostic with
//! [`slideforge_plugin_api::DiagnosticSeverity::Error`] severity, carrying the
//! `ImageSpec`'s `source_span` as the diagnostic span.
//!
//! ## Error taxonomy
//!
//! | Code | Severity | Exit | CWE |
//! |------|----------|------|-----|
//! | `E-VAL-012` | Error | 2 | CWE-22 (Path Traversal) |
//!
//! ## Traceability
//!
//! Traces to BC-1.16.001 EC-012, SEC-001, CWE-22.

use std::sync::Arc;

use slideforge_plugin_api::{Diagnostic, DiagnosticSeverity, Validator, ValidatorOptions};
use slideforge_types::{ContentBlock, Deck, SourceSpan};

/// Error code for an image path that escapes the source root.
///
/// Severity: [`DiagnosticSeverity::Error`] (blocking, exit 2 in strict mode).
/// Traces to BC-1.16.001 EC-012, SEC-001, CWE-22.
pub const E_VAL_012: &str = "E-VAL-012";

/// Validates that all [`slideforge_types::specs::ImageSpec`] paths are safely
/// contained within the source root directory.
///
/// Register with [`slideforge_plugin_api::PluginRegistry::register_validator`].
pub struct ImagePathValidator;

impl Validator for ImagePathValidator {
    fn id(&self) -> &'static str {
        "image-path"
    }

    /// Pre-layout pass: iterates `deck.slides[*].blocks` for `ContentBlock::Image`
    /// entries and rejects any `ImageSpec.path` that escapes the source root.
    ///
    /// Three classes of violation are detected and accumulated (no short-circuit):
    /// 1. **Path-traversal segment** — any path segment equal to exactly `".."`.
    /// 2. **Absolute path** — path starts with `'/'` or `'\\'`.
    /// 3. **Windows drive letter** — path starts with a drive letter (`[A-Za-z]:`).
    ///
    /// Each violating image produces exactly one [`E_VAL_012`] diagnostic carrying
    /// the `ImageSpec`'s own `span` (not the enclosing `Block`'s span).
    fn validate(&self, deck: &Deck, _opts: &ValidatorOptions) -> Vec<Diagnostic> {
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        for slide in &deck.slides {
            for block in &slide.blocks {
                if let ContentBlock::Image(spec) = &block.content {
                    let path = spec.path.as_ref();
                    if let Some(diag) = check_image_path(path, &spec.span) {
                        diagnostics.push(diag);
                    }
                }
            }
        }

        diagnostics
    }
}

// ─── Path safety helpers ──────────────────────────────────────────────────────

/// Check a single image path for traversal, absolute, or drive-letter violations.
///
/// Returns `Some(Diagnostic)` if the path escapes the source root, `None` if safe.
///
/// ## Detection order (priority: absolute/drive-letter first, then traversal)
///
/// 1. **Windows drive letter** — `^[A-Za-z]:` prefix (e.g. `C:\`, `c:/`).
/// 2. **Absolute path** — starts with `'/'` or `'\\'` (after drive-letter check so
///    `C:\` doesn't also trigger the absolute check).
/// 3. **Traversal segment** — any path segment split on `'/'` or `'\\'` that equals
///    exactly `".."`. A segment like `"..images"` is NOT a traversal segment.
fn check_image_path(path: &str, span: &SourceSpan) -> Option<Diagnostic> {
    // Check 1: Windows drive letter prefix `[A-Za-z]:`.
    if is_drive_letter_path(path) {
        return Some(make_image_path_diagnostic(
            path,
            span,
            &format!(
                "Image path '{path}' escapes the source root — uses a Windows drive letter prefix. \
                 Image paths must be relative and contained within the source file's directory tree. \
                 (at {span})"
            ),
        ));
    }

    // Check 2: Absolute path starting with `/` or `\`.
    let first = path.chars().next();
    if first == Some('/') || first == Some('\\') {
        return Some(make_image_path_diagnostic(
            path,
            span,
            &format!(
                "Image path '{path}' escapes the source root — path is absolute \
                 (starts with '/' or '\\'). \
                 Image paths must be relative and contained within the source file's directory tree. \
                 (at {span})"
            ),
        ));
    }

    // Check 3: Path-traversal segment — any segment exactly equal to `".."`.
    // Split on both `/` and `\` separators.
    let has_traversal = path.split(['/', '\\']).any(|seg| seg == "..");
    if has_traversal {
        return Some(make_image_path_diagnostic(
            path,
            span,
            &format!(
                "Image path '{path}' escapes the source root — contains a path-traversal \
                 segment ('..'). \
                 Image paths must be relative and contained within the source file's directory tree. \
                 (at {span})"
            ),
        ));
    }

    None
}

/// Returns `true` if the path begins with a Windows drive letter prefix (`[A-Za-z]:`).
fn is_drive_letter_path(path: &str) -> bool {
    let mut chars = path.chars();
    match (chars.next(), chars.next()) {
        (Some(c), Some(':')) => c.is_ascii_alphabetic(),
        _ => false,
    }
}

/// Construct an `E-VAL-012` error diagnostic for an image path that escapes the
/// source root.
fn make_image_path_diagnostic(_path: &str, span: &SourceSpan, message: &str) -> Diagnostic {
    Diagnostic {
        severity: DiagnosticSeverity::Error,
        code: Arc::from(E_VAL_012),
        message: Arc::from(message),
        span: span.clone(),
        hint: Some(Arc::from(
            "Image paths must be relative and contained within the source file's directory tree. \
             Use paths like 'images/logo.png' or 'assets/photo.png'.",
        )),
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::doc_markdown)]
mod tests {
    use std::sync::Arc;

    use slideforge_plugin_api::{DiagnosticSeverity, Validator, ValidatorOptions};
    use slideforge_types::{
        Block, ContentBlock, Deck, DeckMetadata, OrderedMap, Slide, SourceSpan,
        specs::{AltText, ImageSpec},
    };

    use super::{E_VAL_012, ImagePathValidator};

    // ── Construction helpers ─────────────────────────────────────────────────

    fn make_metadata() -> DeckMetadata {
        DeckMetadata {
            title: Some(Arc::from("Test Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        }
    }

    fn make_deck(slides: Vec<Slide>) -> Deck {
        Deck {
            slides,
            vars: OrderedMap::new(),
            metadata: make_metadata(),
            registers: OrderedMap::new(),
            section_blocks: vec![],
            slide_sections: vec![],
        }
    }

    fn make_slide(blocks: Vec<Block>) -> Slide {
        Slide {
            slide_type: Arc::from("content"),
            fields: OrderedMap::new(),
            blocks,
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
            field_spans: OrderedMap::new(),
        }
    }

    /// Build an image block with an explicit `SourceSpan` so the test can
    /// assert `diagnostic.span == spec.span`.
    fn make_image_block_with_span(path: &str, span: SourceSpan) -> Block {
        Block {
            content: ContentBlock::Image(ImageSpec {
                path: Arc::from(path),
                alt: Some(AltText::Provided(Arc::from("alt text"))),
                decorative: false,
                span,
            }),
            label: None,
            span: SourceSpan::default(),
        }
    }

    fn default_opts() -> ValidatorOptions {
        ValidatorOptions::default()
    }

    /// A non-default SourceSpan suitable for asserting span identity in diagnostics.
    fn sentinel_span() -> SourceSpan {
        SourceSpan {
            file: Arc::from("test.sf"),
            line: 7,
            col: 3,
            byte_offset: 99,
        }
    }

    // ── Validator ID ─────────────────────────────────────────────────────────

    /// Validator ID must be "image-path".
    #[test]
    fn test_BC_1_16_001_validator_id() {
        assert_eq!(ImagePathValidator.id(), "image-path");
    }

    // ── Rejection case 1: path-traversal segment ─────────────────────────────

    /// BC-1.16.001 EC-012 — path containing `..` as a segment must produce
    /// exactly one E-VAL-012 Error diagnostic.
    ///
    /// Red Gate: stub returns empty Vec → assertion on `diags.len() == 1` FAILS.
    #[test]
    fn test_BC_1_16_001_rejects_dotdot_traversal_segment() {
        let span = sentinel_span();
        let deck = make_deck(vec![make_slide(vec![make_image_block_with_span(
            "../outside/secret.png",
            span.clone(),
        )])]);
        let diags = ImagePathValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "path '../outside/secret.png' contains '..': must produce exactly 1 E-VAL-012; \
             got {} diagnostics: {diags:?}",
            diags.len()
        );
        assert_eq!(
            diags[0].code.as_ref(),
            E_VAL_012,
            "diagnostic code must be E-VAL-012; got '{}'",
            diags[0].code
        );
        assert_eq!(
            diags[0].severity,
            DiagnosticSeverity::Error,
            "E-VAL-012 must be Error severity; got {:?}",
            diags[0].severity
        );
        assert_eq!(
            diags[0].span, span,
            "diagnostic span must equal the ImageSpec's source_span; \
             got {:?}",
            diags[0].span
        );
        assert!(
            diags[0].message.contains(".."),
            "message must mention the '..'; got: {}",
            diags[0].message
        );
        assert!(
            diags[0].message.contains("path-traversal"),
            "message must mention 'path-traversal'; got: {}",
            diags[0].message
        );
    }

    /// Embedded `..` in a multi-segment path must also be rejected.
    ///
    /// Red Gate: stub returns empty Vec → assertion FAILS.
    #[test]
    fn test_BC_1_16_001_rejects_embedded_dotdot_traversal() {
        let span = sentinel_span();
        let deck = make_deck(vec![make_slide(vec![make_image_block_with_span(
            "images/../../../etc/passwd.png",
            span.clone(),
        )])]);
        let diags = ImagePathValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "path with embedded '..': must produce exactly 1 E-VAL-012; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_VAL_012);
        assert_eq!(diags[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diags[0].span, span);
    }

    /// Message format for traversal case must match spec:
    /// `Image path '<path>' escapes the source root — contains a path-traversal
    /// segment ('..'). Image paths must be relative and contained within the
    /// source file's directory tree. (at <file>:<line>:<col>)`
    ///
    /// Red Gate: stub returns empty Vec → assertion FAILS.
    #[test]
    fn test_BC_1_16_001_traversal_message_format() {
        let span = sentinel_span();
        let path = "../secret.png";
        let deck = make_deck(vec![make_slide(vec![make_image_block_with_span(
            path,
            span.clone(),
        )])]);
        let diags = ImagePathValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "must produce exactly 1 diagnostic; got {diags:?}"
        );
        let msg = diags[0].message.as_ref();
        assert!(
            msg.contains(path),
            "message must contain the path '{path}'; got: {msg}"
        );
        assert!(
            msg.contains("escapes the source root"),
            "message must contain 'escapes the source root'; got: {msg}"
        );
        assert!(
            msg.contains("path-traversal"),
            "message must contain 'path-traversal'; got: {msg}"
        );
        assert!(
            msg.contains("('..')"),
            "message must contain \"('..')\"; got: {msg}"
        );
        // Must end with span location in (at <file>:<line>:<col>) form.
        assert!(
            msg.contains("at "),
            "message must include span location starting with 'at '; got: {msg}"
        );
    }

    // ── Rejection case 2: absolute path (Unix) ───────────────────────────────

    /// BC-1.16.001 EC-012 — path starting with `/` must produce one E-VAL-012.
    ///
    /// Red Gate: stub returns empty Vec → assertion FAILS.
    #[test]
    fn test_BC_1_16_001_rejects_absolute_unix_path() {
        let span = sentinel_span();
        let deck = make_deck(vec![make_slide(vec![make_image_block_with_span(
            "/etc/passwd.png",
            span.clone(),
        )])]);
        let diags = ImagePathValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "path '/etc/passwd.png' is absolute (starts with '/'): must produce 1 E-VAL-012; \
             got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_VAL_012);
        assert_eq!(diags[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diags[0].span, span);
        assert!(
            diags[0].message.contains("absolute"),
            "message must mention 'absolute'; got: {}",
            diags[0].message
        );
    }

    /// Message format for absolute-path case must match spec:
    /// `Image path '<path>' escapes the source root — path is absolute
    /// (starts with '/' or '\'). ...`
    ///
    /// Red Gate: stub returns empty Vec → assertion FAILS.
    #[test]
    fn test_BC_1_16_001_absolute_message_format() {
        let span = sentinel_span();
        let path = "/home/user/images/logo.png";
        let deck = make_deck(vec![make_slide(vec![make_image_block_with_span(
            path,
            span.clone(),
        )])]);
        let diags = ImagePathValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "must produce exactly 1 diagnostic; got {diags:?}"
        );
        let msg = diags[0].message.as_ref();
        assert!(
            msg.contains(path),
            "message must contain the path; got: {msg}"
        );
        assert!(
            msg.contains("escapes the source root"),
            "must say 'escapes the source root'; got: {msg}"
        );
        assert!(msg.contains("absolute"), "must say 'absolute'; got: {msg}");
        assert!(msg.contains("'/'"), "must mention '/'; got: {msg}");
        assert!(
            msg.contains("at "),
            "must include span location; got: {msg}"
        );
    }

    /// Absolute path starting with `\` (backslash) must also be rejected.
    ///
    /// Red Gate: stub returns empty Vec → assertion FAILS.
    #[test]
    fn test_BC_1_16_001_rejects_absolute_backslash_path() {
        let span = sentinel_span();
        let deck = make_deck(vec![make_slide(vec![make_image_block_with_span(
            "\\server\\share\\image.png",
            span.clone(),
        )])]);
        let diags = ImagePathValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "path starting with '\\' is absolute: must produce 1 E-VAL-012; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_VAL_012);
        assert_eq!(diags[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diags[0].span, span);
    }

    // ── Rejection case 3: Windows drive letter ────────────────────────────────

    /// BC-1.16.001 EC-012 — path with a Windows drive letter prefix (e.g. `C:\`)
    /// must produce one E-VAL-012.
    ///
    /// Red Gate: stub returns empty Vec → assertion FAILS.
    #[test]
    fn test_BC_1_16_001_rejects_windows_drive_letter() {
        let span = sentinel_span();
        let deck = make_deck(vec![make_slide(vec![make_image_block_with_span(
            r"C:\Users\user\image.png",
            span.clone(),
        )])]);
        let diags = ImagePathValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            r"path 'C:\Users\user\image.png' has drive letter: must produce 1 E-VAL-012; \
             got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_VAL_012);
        assert_eq!(diags[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diags[0].span, span);
        assert!(
            diags[0].message.contains("drive letter"),
            "message must mention 'drive letter'; got: {}",
            diags[0].message
        );
    }

    /// Lowercase drive letter (e.g. `c:\`) must also be rejected.
    ///
    /// Red Gate: stub returns empty Vec → assertion FAILS.
    #[test]
    fn test_BC_1_16_001_rejects_lowercase_windows_drive_letter() {
        let span = sentinel_span();
        let deck = make_deck(vec![make_slide(vec![make_image_block_with_span(
            r"c:\images\logo.png",
            span.clone(),
        )])]);
        let diags = ImagePathValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            r"lowercase drive letter 'c:\' must also be rejected; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_VAL_012);
        assert_eq!(diags[0].severity, DiagnosticSeverity::Error);
    }

    /// Message format for Windows drive letter case must match spec:
    /// `Image path '<path>' escapes the source root — uses a Windows drive letter prefix.`
    ///
    /// Red Gate: stub returns empty Vec → assertion FAILS.
    #[test]
    fn test_BC_1_16_001_drive_letter_message_format() {
        let span = sentinel_span();
        let path = r"D:\data\chart.png";
        let deck = make_deck(vec![make_slide(vec![make_image_block_with_span(
            path,
            span.clone(),
        )])]);
        let diags = ImagePathValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "must produce exactly 1 diagnostic; got {diags:?}"
        );
        let msg = diags[0].message.as_ref();
        assert!(
            msg.contains(path),
            "message must contain the path; got: {msg}"
        );
        assert!(
            msg.contains("escapes the source root"),
            "must say 'escapes the source root'; got: {msg}"
        );
        assert!(
            msg.contains("drive letter"),
            "must say 'drive letter'; got: {msg}"
        );
        assert!(
            msg.contains("at "),
            "must include span location; got: {msg}"
        );
    }

    // ── Span identity ─────────────────────────────────────────────────────────

    /// For all rejection cases, the diagnostic span MUST equal the ImageSpec's
    /// own `span` (not the Block's span, not SourceSpan::default()).
    ///
    /// Red Gate: stub returns empty Vec → assertion FAILS for each case.
    #[test]
    fn test_BC_1_16_001_diagnostic_span_equals_imagespec_span() {
        let span = SourceSpan {
            file: Arc::from("deck/slides.sf"),
            line: 42,
            col: 8,
            byte_offset: 1234,
        };

        // Case 1: traversal
        let deck1 = make_deck(vec![make_slide(vec![make_image_block_with_span(
            "../escape.png",
            span.clone(),
        )])]);
        let diags1 = ImagePathValidator.validate(&deck1, &default_opts());
        assert_eq!(
            diags1.len(),
            1,
            "traversal: must produce 1 diagnostic; got {diags1:?}"
        );
        assert_eq!(
            diags1[0].span, span,
            "traversal: span must be ImageSpec.span; got {:?}",
            diags1[0].span
        );

        // Case 2: absolute
        let deck2 = make_deck(vec![make_slide(vec![make_image_block_with_span(
            "/absolute.png",
            span.clone(),
        )])]);
        let diags2 = ImagePathValidator.validate(&deck2, &default_opts());
        assert_eq!(
            diags2.len(),
            1,
            "absolute: must produce 1 diagnostic; got {diags2:?}"
        );
        assert_eq!(
            diags2[0].span, span,
            "absolute: span must be ImageSpec.span; got {:?}",
            diags2[0].span
        );

        // Case 3: drive letter
        let deck3 = make_deck(vec![make_slide(vec![make_image_block_with_span(
            r"Z:\drive.png",
            span.clone(),
        )])]);
        let diags3 = ImagePathValidator.validate(&deck3, &default_opts());
        assert_eq!(
            diags3.len(),
            1,
            "drive letter: must produce 1 diagnostic; got {diags3:?}"
        );
        assert_eq!(
            diags3[0].span, span,
            "drive letter: span must be ImageSpec.span; got {:?}",
            diags3[0].span
        );
    }

    // ── Multiple violations accumulate ────────────────────────────────────────

    /// Multiple images with escaping paths in the same deck must each produce
    /// one E-VAL-012 (error accumulation, no short-circuit).
    ///
    /// Red Gate: stub returns empty Vec → assertion FAILS.
    #[test]
    fn test_BC_1_16_001_multiple_violations_accumulate() {
        let s = SourceSpan::default();
        let deck = make_deck(vec![make_slide(vec![
            make_image_block_with_span("../a.png", s.clone()),
            make_image_block_with_span("/absolute.png", s.clone()),
            make_image_block_with_span(r"C:\win.png", s.clone()),
        ])]);
        let diags = ImagePathValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            3,
            "each of 3 escaping paths must produce its own E-VAL-012 (accumulation); \
             got {} diagnostics: {diags:?}",
            diags.len()
        );
        for d in &diags {
            assert_eq!(
                d.code.as_ref(),
                E_VAL_012,
                "all diagnostics must be E-VAL-012"
            );
            assert_eq!(d.severity, DiagnosticSeverity::Error);
        }
    }

    // ── Accept cases: safe paths produce zero diagnostics ────────────────────
    // These tests PASS against the stub (stub returns empty Vec).
    // They serve as regression guards after implementation.

    /// Safe relative path "images/logo.png" → 0 diagnostics.
    #[test]
    fn test_BC_1_16_001_accepts_relative_flat_path() {
        let deck = make_deck(vec![make_slide(vec![make_image_block_with_span(
            "logo.png",
            SourceSpan::default(),
        )])]);
        let diags = ImagePathValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "safe relative path 'logo.png' must produce no diagnostics; got {diags:?}"
        );
    }

    /// Safe relative path in a subdirectory → 0 diagnostics.
    #[test]
    fn test_BC_1_16_001_accepts_relative_subdirectory_path() {
        let deck = make_deck(vec![make_slide(vec![make_image_block_with_span(
            "images/logo.png",
            SourceSpan::default(),
        )])]);
        let diags = ImagePathValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "safe path 'images/logo.png' must produce no diagnostics; got {diags:?}"
        );
    }

    /// Deep nested relative path → 0 diagnostics.
    #[test]
    fn test_BC_1_16_001_accepts_deep_relative_path() {
        let deck = make_deck(vec![make_slide(vec![make_image_block_with_span(
            "sub/dir/img.png",
            SourceSpan::default(),
        )])]);
        let diags = ImagePathValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "safe path 'sub/dir/img.png' must produce no diagnostics; got {diags:?}"
        );
    }

    /// A directory segment named "..something" (not exactly "..") → 0 diagnostics.
    /// This is a safe path: the segment is not a traversal segment.
    #[test]
    fn test_BC_1_16_001_accepts_dotdot_prefix_segment_that_is_not_traversal() {
        // "..images/logo.png" — the component "..images" is not ".."
        let deck = make_deck(vec![make_slide(vec![make_image_block_with_span(
            "..images/logo.png",
            SourceSpan::default(),
        )])]);
        let diags = ImagePathValidator.validate(&deck, &default_opts());
        // This path is debatable — "..images" starts with ".." but is not a traversal.
        // The spec says: "contains a .. traversal *segment*". A segment is defined by
        // path component boundaries (/ or \). "..images" is a single component that
        // happens to start with ".." but is not equal to "..".
        // Accept case: we assert 0 diagnostics for well-formed relative paths
        // whose individual components are not exactly "..".
        assert!(
            diags.is_empty(),
            "'..images/logo.png' — '..images' is not a traversal segment; \
             should produce 0 diagnostics; got {diags:?}"
        );
    }

    /// Empty deck → 0 diagnostics.
    #[test]
    fn test_BC_1_16_001_accepts_empty_deck() {
        let deck = make_deck(vec![]);
        let diags = ImagePathValidator.validate(&deck, &default_opts());
        assert!(diags.is_empty(), "empty deck must produce no diagnostics");
    }
}
