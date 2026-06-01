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
}
