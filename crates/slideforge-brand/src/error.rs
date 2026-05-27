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

use std::sync::Arc;

use slideforge_types::SourceSpan;
use thiserror::Error;

// ─── Error code constants ────────────────────────────────────────────────────

/// `E-BRD-001`: brand file not found at the resolved path.
pub const E_BRD_001: &str = "E-BRD-001";

/// `E-BRD-002`: the brand template file cannot be parsed (corrupt or not OOXML).
pub const E_BRD_002: &str = "E-BRD-002";

/// `E-BRD-003`: a required OOXML color slot is absent in the source template;
/// the slot is inferred and the build continues.
pub const E_BRD_003: &str = "E-BRD-003";

/// `E-BRD-004`: a declared font is not installed on the build host; the build
/// continues with a fallback font for metrics only.
pub const E_BRD_004: &str = "E-BRD-004";

// ─── Error enum ──────────────────────────────────────────────────────────────

/// Errors produced by the `slideforge-brand` crate.
///
/// **Fatal variants** (broken, exit 4): [`BrandError::FileNotFound`], [`BrandError::ParseError`].
///
/// **Cosmetic variants** (exit 0, warning only): [`BrandError::MissingColorSlot`], [`BrandError::FontUnavailable`].
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
         File may be corrupted or not a valid PPTX/TOML."
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
    }

    /// BC-2.01.001 EC-001 — FileNotFound error message contains the path.
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

    /// BC-2.01.001 EC-002 — ParseError message contains path, reason, and span.
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

    /// BC-2.01.001 EC-003 / BC-2.01.004 — MissingColorSlot message contains slot name,
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

    /// BC-2.01.006 — FontUnavailable message contains font name and fallback.
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
}
