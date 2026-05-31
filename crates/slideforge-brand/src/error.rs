//! Error types for the `slideforge-brand` crate.
//!
//! Each variant maps to an error code from the slideforge error taxonomy:
//!
//! | Code | Variant | Severity |
//! |------|---------|---------|
//! | `E-BRD-001` | [`BrandError::FileNotFound`] | broken (exit 4) |
//! | `E-BRD-002` | [`BrandError::ParseError`] | broken (exit 4) |
//! | `E-BRD-003` | [`BrandError::MissingColorSlot`] | cosmetic (exit 0) |
//! | `E-BRD-004` | [`BrandError::FontUnavailable`] | cosmetic (exit 0) |
//! | `E-BRD-005` | [`BrandError::InvalidHexColor`] | cosmetic (exit 0) |
//! | `E-BRD-006` | [`BrandError::OutputExists`] | broken (exit 4) |

use std::sync::Arc;

use slideforge_types::SourceSpan;
use thiserror::Error;

// ─── Error code constants ────────────────────────────────────────────────────

/// `E-BRD-001`: brand file not found at the resolved path.
///
/// Also emitted when a synthesized brand is missing the required `[logo]` path.
pub const E_BRD_001: &str = "E-BRD-001";

/// `E-BRD-002`: the brand template file cannot be parsed (corrupt or not OOXML).
pub const E_BRD_002: &str = "E-BRD-002";

/// `E-BRD-003`: a required OOXML color slot is absent in the source template;
/// the slot is inferred and the build continues.
pub const E_BRD_003: &str = "E-BRD-003";

/// `E-BRD-004`: a declared font is not installed on the build host; the build
/// continues with a fallback font for metrics only.
pub const E_BRD_004: &str = "E-BRD-004";

/// `E-BRD-005`: a hex color value in `brand.toml` is not valid 6-digit uppercase
/// RGB hex (e.g. `#3B82F6`). Cosmetic warning — the invalid slot is treated as
/// absent and inference continues with the remaining slots.
pub const E_BRD_005: &str = "E-BRD-005";

/// `E-BRD-006`: the output `brand.toml` already exists and `--force` was not
/// passed to the extraction command.
pub const E_BRD_006: &str = "E-BRD-006";

/// `E-BRD-007`: the logo path in `brand.toml` escapes the directory containing
/// `brand.toml`. This is a path-traversal security violation — the logo must
/// reside inside (or underneath) the `brand.toml` directory.
pub const E_BRD_007: &str = "E-BRD-007";

// ─── Error enum ──────────────────────────────────────────────────────────────

/// Errors produced by the `slideforge-brand` crate.
///
/// **Fatal variants** (broken, exit 4): [`BrandError::FileNotFound`], [`BrandError::ParseError`].
///
/// **Cosmetic variants** (exit 0, warning only): [`BrandError::MissingColorSlot`], [`BrandError::FontUnavailable`].
// #[non_exhaustive] for v1.0 SemVer hygiene; new variants may be added in minor releases
// per CLAUDE.md Quality Bar Supply chain row.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BrandError {
    /// `E-BRD-001` — the brand template file was not found at the resolved path.
    ///
    /// This is a fatal error. The build exits with code 4.
    #[error(
        "E-BRD-001: Brand file not found: '{path}'. \
         Check the template path in slideforge.toml or --template flag."
    )]
    FileNotFound {
        /// The path that was not found.
        path: Arc<str>,
        /// Source location from the `.sf` file that referenced this path.
        span: SourceSpan,
    },

    /// `E-BRD-002` — the brand template file exists but cannot be parsed as an
    /// OOXML package.
    ///
    /// This is a fatal error. The build exits with code 4.
    #[error(
        "E-BRD-002: Cannot parse brand template '{path}': {reason}. \
         File may be corrupted or not a valid PPTX/DOCX."
    )]
    ParseError {
        /// The path of the file that failed to parse.
        path: Arc<str>,
        /// Human-readable description of why parsing failed.
        reason: Arc<str>,
        /// Source location of the `brand "..."` declaration in the `.sf` file.
        span: SourceSpan,
    },

    /// `E-BRD-003` — a required OOXML color slot is absent in the source
    /// `theme1.xml`. The build continues with an inferred value.
    ///
    /// This is a cosmetic warning. The build exits with code 0.
    #[error(
        "E-BRD-003: Brand color slot '{slot_name}' inferred as {inferred_hex} \
         (derived from {derivation}). Review in brand.toml to confirm."
    )]
    MissingColorSlot {
        /// The OOXML slot name (e.g., `"dk1"`, `"acc3"`).
        slot_name: Arc<str>,
        /// The inferred hex color value (e.g., `"#808080"`).
        inferred_hex: Arc<str>,
        /// Description of where the inferred value was derived from (e.g., `"default inference"`).
        derivation: Arc<str>,
    },

    /// `E-BRD-004` — a font declared in the brand template is not installed on
    /// the build host. The build continues; OOXML output still writes the
    /// declared font name.
    ///
    /// This is a cosmetic warning. The build exits with code 0.
    ///
    /// ## PANOSE deviation note (FINDING-005)
    ///
    /// The error taxonomy's aspirational format for E-BRD-004 includes a
    /// `(panose: [<class>])` suffix.  PANOSE data cannot be obtained from a
    /// directory scan — it requires parsing the font binary (e.g., via
    /// `font-kit` or the OS font API).  The current implementation performs a
    /// name-based scan only and therefore omits the PANOSE field.
    /// A full `font-kit`-based implementation is deferred to a future story.
    #[error(
        "E-BRD-004: Font '{font_name}' not available on this build host. \
         Using '{fallback}' for build-time metrics. Text metrics may differ."
    )]
    FontUnavailable {
        /// The font name that is unavailable on the build host.
        font_name: Arc<str>,
        /// The fallback font name being used for metric calculations.
        fallback: Arc<str>,
    },

    // ─── STORY-023 synthesis variants ─────────────────────────────────────────
    /// `E-BRD-001` (synthesis) — the `[logo]` section is absent in a
    /// synthesized brand's `brand.toml`.
    ///
    /// This is a fatal error (exit 4) for synthesized brands. A synthesized
    /// brand cannot embed a logo if no path is declared.
    ///
    /// Traces to BC-2.01.002 edge case EC-005 (AC-004).
    #[error(
        "E-BRD-001: Synthesized brand requires a logo path. \
         Add a [logo] section with 'path = \"...\"' to brand.toml."
    )]
    LogoRequired {
        /// Source location of the `brand "..."` or `brand.toml` declaration.
        ///
        /// May be `SourceSpan::default()` when invoked from a non-DSL context
        /// (e.g., direct API call without a source file).
        span: SourceSpan,
    },

    /// `E-BRD-001` (synthesis) — the `brand.toml` file could not be read.
    ///
    /// Fatal error (exit 4). `reason` carries the underlying I/O error message.
    #[error("E-BRD-001: Cannot read brand.toml at '{path}': {reason}.")]
    TomlReadError {
        /// Path of the file that could not be read.
        path: Arc<str>,
        /// Human-readable I/O error description.
        reason: Arc<str>,
    },

    /// `E-BRD-002` (synthesis) — the `brand.toml` file is not valid TOML.
    ///
    /// Fatal error (exit 4). `reason` carries the TOML parser error message.
    #[error(
        "E-BRD-002: Cannot parse brand.toml at '{path}': {reason}. \
         File may contain invalid TOML syntax."
    )]
    TomlParseError {
        /// Path of the file that failed to parse.
        path: Arc<str>,
        /// Human-readable parse error description.
        reason: Arc<str>,
    },

    /// `E-BRD-005` — a declared hex color value in `brand.toml` is not valid.
    ///
    /// Cosmetic warning (exit 0). The invalid slot is treated as absent and
    /// inference continues with the remaining slots (see `inference.rs`). The
    /// build does NOT abort on an invalid hex — callers receive the error via
    /// the warnings `Vec` returned alongside the synthesized `BrandTemplate`.
    ///
    /// Traces to AC-006 (BC-2.01.004 invariant 3).
    ///
    /// ## Spec note (E-BRD-005 revised semantic)
    ///
    /// The original E-BRD-005 semantic ("missing required color slot — fatal") was
    /// retired when the brand synthesis algorithm was designed to always infer missing
    /// slots (BC-2.01.004). The error code was subsequently reused for invalid hex
    /// color validation (introduced in Pass-11 fix burst). The error-taxonomy.md spec
    /// has been updated to reflect this revised semantic.
    #[error(
        "E-BRD-005: Invalid hex color value '{value}' in brand.toml slot '{slot_name}'. \
         Use 6-digit uppercase hex RGB (e.g. #3B82F6; case insensitive — uppercase or lowercase accepted)."
    )]
    InvalidHexColor {
        /// The OOXML color slot name (e.g., `"acc1"`).
        slot_name: Arc<str>,
        /// The invalid value that was declared.
        value: Arc<str>,
    },

    /// `E-BRD-007` — the logo path in `brand.toml` escapes the directory containing
    /// `brand.toml`.
    ///
    /// Fatal error (exit 4). A logo path must resolve to a file inside (or
    /// beneath) the directory that contains `brand.toml`. Paths that escape
    /// via `../` sequences or symlinks pointing outside that directory are
    /// rejected to prevent path-traversal attacks.
    #[error(
        "E-BRD-007: Logo path '{logo_path}' escapes the brand.toml directory '{brand_dir}'. \
         The logo file must be inside (or beneath) the brand.toml directory."
    )]
    LogoOutsideBrandDir {
        /// The resolved canonical path of the logo file that escaped the brand dir.
        logo_path: String,
        /// The canonical path of the `brand.toml` parent directory.
        brand_dir: String,
    },

    /// `E-BRD-006` — the output `brand.toml` already exists and `--force` was
    /// not passed to the extraction command.
    ///
    /// This is a fatal error. The command exits with code 4.
    ///
    /// Traces to BC-2.01.003 edge case EC-001.
    #[error("E-BRD-006: {path}: brand.toml already exists. Use --force to overwrite.")]
    OutputExists {
        /// The path of the existing `brand.toml` file.
        path: Arc<str>,
    },

    /// `E-BRD-004` (synthesis) — a font name declared in `brand.toml`
    /// is not installed on the build host.
    ///
    /// Cosmetic warning (exit 0). The OOXML output still writes the declared
    /// font name. The build continues.
    ///
    /// Traces to BC-2.01.002 edge case EC-004 (AC-015, NFR-021).
    ///
    /// ## EC-004 font-availability check — deferral note
    ///
    /// The font availability check for synthesized brands (EC-004) is deferred
    /// to STORY-024 (Brand Extraction CLI). The reason: `BrandSynthesizer::synthesize`
    /// is a **pure function** (Architecture Compliance Rule 2 — no side effects, no I/O).
    /// Querying the OS font registry requires I/O and therefore belongs in the
    /// effectful extraction/validation stage, not in the pure synthesizer.
    ///
    /// This variant is kept in the enum so that STORY-024 can emit it without a
    /// breaking API change. It is not currently constructed by any production code
    /// path; STORY-024 will wire up the construction.
    #[error(
        "E-BRD-004: Font '{font_name}' declared in brand.toml is not available \
         on this build host. Build continues; output will use '{font_name}'."
    )]
    DeclaredFontUnavailable {
        /// The font name declared in `brand.toml` that is not installed.
        font_name: Arc<str>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BC-2.01.001 / BC-2.01.006 — error code constants are correct.
    #[test]
    fn test_bc_2_01_001_error_codes() {
        assert_eq!(E_BRD_001, "E-BRD-001");
        assert_eq!(E_BRD_002, "E-BRD-002");
        assert_eq!(E_BRD_003, "E-BRD-003");
        assert_eq!(E_BRD_004, "E-BRD-004");
        assert_eq!(E_BRD_005, "E-BRD-005");
        assert_eq!(E_BRD_007, "E-BRD-007");
    }

    /// BC-2.01.001 EC-001 — `FileNotFound` error message contains the path.
    #[test]
    fn test_bc_2_01_001_file_not_found_message() {
        let err = BrandError::FileNotFound {
            path: Arc::from("missing.pptx"),
            span: SourceSpan::default(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("missing.pptx"),
            "error message must contain path, got: {msg}"
        );
        assert!(
            msg.contains("E-BRD-001"),
            "error message must contain error code, got: {msg}"
        );
    }

    /// BC-2.01.001 EC-002 — `ParseError` message contains path, reason, and span.
    #[test]
    fn test_bc_2_01_001_parse_error_message() {
        let err = BrandError::ParseError {
            path: Arc::from("corrupt.pptx"),
            reason: Arc::from("not a valid ZIP archive"),
            span: SourceSpan::default(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("corrupt.pptx"),
            "error message must contain path, got: {msg}"
        );
        assert!(
            msg.contains("not a valid ZIP archive"),
            "error message must contain reason, got: {msg}"
        );
        assert!(
            msg.contains("E-BRD-002"),
            "error message must contain error code, got: {msg}"
        );
    }

    /// FINDING-001 — `ParseError` message says "PPTX/DOCX", not "PPTX/TOML".
    ///
    /// The error message previously contained "PPTX/TOML" which was a copy-paste
    /// error. The correct file types supported are PPTX and DOCX.
    #[test]
    fn test_finding_001_parse_error_says_docx_not_toml() {
        let err = BrandError::ParseError {
            path: Arc::from("brand.pptx"),
            reason: Arc::from("not a ZIP archive"),
            span: SourceSpan::default(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("DOCX"),
            "ParseError message must contain 'DOCX', got: {msg}"
        );
        assert!(
            !msg.contains("TOML"),
            "ParseError message must NOT contain 'TOML', got: {msg}"
        );
    }

    /// BC-2.01.001 EC-003 / BC-2.01.004 — `MissingColorSlot` message contains slot name,
    /// inferred hex, and derivation source.
    #[test]
    fn test_bc_2_01_001_missing_color_slot_message() {
        let err = BrandError::MissingColorSlot {
            slot_name: Arc::from("acc3"),
            inferred_hex: Arc::from("#808080"),
            derivation: Arc::from("default inference"),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("acc3"),
            "error message must contain slot name, got: {msg}"
        );
        assert!(
            msg.contains("#808080"),
            "error message must contain inferred hex, got: {msg}"
        );
        assert!(
            msg.contains("E-BRD-003"),
            "error message must contain error code, got: {msg}"
        );
    }

    /// BC-2.01.006 — `FontUnavailable` message contains font name and fallback.
    #[test]
    fn test_bc_2_01_006_font_unavailable_message() {
        let err = BrandError::FontUnavailable {
            font_name: Arc::from("Aptos Display"),
            fallback: Arc::from("Calibri"),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("Aptos Display"),
            "error message must contain font name, got: {msg}"
        );
        assert!(
            msg.contains("Calibri"),
            "error message must contain fallback, got: {msg}"
        );
        assert!(
            msg.contains("E-BRD-004"),
            "error message must contain error code, got: {msg}"
        );
    }

    /// E-BRD-007 — `LogoOutsideBrandDir` message contains logo path, brand dir, and error code.
    #[test]
    fn test_e_brd_007_logo_outside_brand_dir_message() {
        let err = BrandError::LogoOutsideBrandDir {
            logo_path: "/tmp/etc/passwd".to_owned(),
            brand_dir: "/tmp/brand".to_owned(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("E-BRD-007"),
            "error message must contain E-BRD-007, got: {msg}"
        );
        assert!(
            msg.contains("/tmp/etc/passwd"),
            "error message must contain logo path, got: {msg}"
        );
        assert!(
            msg.contains("/tmp/brand"),
            "error message must contain brand dir, got: {msg}"
        );
    }

    /// F-PASS16-MED-1 — E-BRD-005 message matches error-taxonomy.md row 115.
    ///
    /// Verifies the word "value" is present and the case-insensitivity note is
    /// included, mirroring the `validate_hex` behaviour introduced in Pass-11.
    #[test]
    fn test_f_pass16_med_1_e_brd_005_message_matches_taxonomy() {
        let err = BrandError::InvalidHexColor {
            slot_name: Arc::from("acc1"),
            value: Arc::from("zzzzzz"),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("E-BRD-005"),
            "must contain error code, got: {msg}"
        );
        assert!(
            msg.contains("value"),
            "must contain the word 'value', got: {msg}"
        );
        assert!(
            msg.contains("case insensitive"),
            "must contain case-insensitivity note, got: {msg}"
        );
        assert!(msg.contains("acc1"), "must contain slot name, got: {msg}");
        assert!(
            msg.contains("zzzzzz"),
            "must contain the invalid value, got: {msg}"
        );
    }

    /// FINDING-005 — E-BRD-004 message does NOT include PANOSE data.
    ///
    /// The error taxonomy's aspirational format includes `(panose: [<class>])`,
    /// but PANOSE data is not available from a directory scan.  This test
    /// verifies the message format is stable and does NOT include a PANOSE
    /// field that we cannot populate.
    #[test]
    fn test_finding_005_font_unavailable_does_not_include_panose() {
        let err = BrandError::FontUnavailable {
            font_name: Arc::from("Calibri Light"),
            fallback: Arc::from("Arial"),
        };
        let msg = err.to_string();
        // Must NOT include "panose" — we cannot provide it without font binary parsing.
        assert!(
            !msg.contains("panose"),
            "E-BRD-004 message must not include panose (not available from name-scan), \
             got: {msg}"
        );
        // Must still include the required fields.
        assert!(msg.contains("E-BRD-004"), "must have error code");
        assert!(msg.contains("Calibri Light"), "must have font name");
        assert!(msg.contains("Arial"), "must have fallback");
    }
}
