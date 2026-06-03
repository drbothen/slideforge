//! DOCX `word/styles.xml` generator.
//!
//! Produces the `word/styles.xml` part that defines the paragraph and
//! character styles used in `word/document.xml`. Required styles:
//!
//! - `Heading1` — slide title level (paragraph style)
//! - `Heading2` — section title level (paragraph style)
//! - `Normal` — body paragraph (paragraph style)
//! - `Hyperlink` — hyperlink character style (character style)
//! - `CodeText` — inline code (character style; Courier New font)
//!
//! Style definitions are sourced from the brand when available (fonts, colors).
//! Minimal valid stubs are emitted when no brand override is present.

use slideforge_types::Brand;

use crate::error::ExportError;

/// Builds the `word/styles.xml` byte buffer.
///
/// When `brand` is `Some`, heading and body styles are configured using the
/// brand's font names and palette colors. When `brand` is `None`, minimal
/// valid stubs are emitted (sufficient for Word to open the document).
///
/// # Errors
///
/// Returns [`ExportError::OoxmlError`] if the XML cannot be constructed.
pub fn build_styles(brand: Option<&Brand>) -> Result<Vec<u8>, ExportError> {
    todo!()
}
