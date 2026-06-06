//! Error types for the `slideforge-pdf` crate.
//!
//! [`PdfExportError`] is the crate-level error enum. All error paths use
//! structured variants via [`thiserror`]; no `.unwrap()` is permitted in
//! non-test code.

use thiserror::Error;

/// Error produced by [`crate::PdfExporter`] when PDF generation fails.
///
/// `#[non_exhaustive]` allows adding new variants in minor releases without
/// breaking downstream consumers.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum PdfExportError {
    /// krilla `Document::finish()` returned a serialization error.
    ///
    /// Maps from `krilla::KrillaError` via the `?` operator.
    #[error("PDF serialization error: {message}")]
    Serialize {
        /// Human-readable description of the serialization failure.
        message: String,
    },

    /// The PDF document failed PDF/UA-1 (or another validator) validation.
    ///
    /// This variant is produced when `krilla::Document::finish()` returns
    /// `KrillaError::Validation(errors)` — meaning the document violates
    /// one or more constraints of the active [`krilla::configure::Validator`].
    ///
    /// Under `Validator::UA1`, common causes include:
    /// - Missing document title (`NoDocumentTitle`)
    /// - Missing document language (`NoDocumentLanguage`)
    /// - Missing heading title on Hn elements (`MissingHeadingTitle`)
    /// - Missing document outline (`MissingDocumentOutline`)
    ///
    /// BC-4.03.001 invariant 5: a `KrillaError::Validation` MUST be propagated
    /// as this variant (not silently swallowed or merged into `Serialize`).
    #[error("PDF validation error: {message}")]
    ValidationFailed {
        /// Human-readable summary of the validation failure(s).
        ///
        /// For multi-error cases, violations are joined with `"; "`.
        message: String,
    },

    /// `Configuration::new_with(Validator, PdfVersion)` returned `None`.
    ///
    /// This means the supplied validator / PDF version combination is invalid.
    #[error("PDF configuration error: invalid validator + version combination ({detail})")]
    InvalidConfiguration {
        /// Description of the invalid combination.
        detail: String,
    },

    /// A required font file could not be loaded.
    #[error("PDF font load error: {message}")]
    FontLoad {
        /// Description of the font load failure.
        message: String,
    },

    /// SVG embedding failed (usvg parse or krilla Surface draw error).
    #[error("PDF SVG embed error: {message}")]
    SvgEmbed {
        /// Description of the SVG embed failure.
        message: String,
    },

    /// An I/O error occurred (e.g., reading a font file from disk).
    #[error("PDF I/O error: {message}")]
    Io {
        /// Description of the I/O failure.
        message: String,
    },

    /// The deck title contains an XML-1.0-illegal control character and cannot
    /// be safely embedded in XMP metadata.
    ///
    /// XMP metadata is an XML-1.0 document. The legal character set excludes
    /// U+0000–U+0008, U+000B, U+000C, U+000E–U+001F, U+FFFE, and U+FFFF.
    /// Embedding such characters would produce a malformed XMP stream
    /// (CWE-116 / SEC-050-001).
    ///
    /// Callers must sanitize the deck title before passing it to the exporter,
    /// or strip/replace illegal characters upstream (e.g., in the DSL parser).
    #[error(
        "PDF XMP metadata error: deck title contains XML-1.0-illegal \
         control character U+{code_point:04X} — title: {title:?}"
    )]
    InvalidXmpTitle {
        /// The raw title string that triggered the rejection.
        title: String,
        /// The Unicode code point of the first illegal character found.
        code_point: u32,
    },
}
