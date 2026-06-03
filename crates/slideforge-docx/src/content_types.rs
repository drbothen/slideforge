//! DOCX `[Content_Types].xml` generator.
//!
//! Produces the `[Content_Types].xml` part required by the OOXML packaging
//! spec. Registers every mandatory DOCX part type so that Word and LibreOffice
//! can locate each part within the ZIP.
//!
//! Required content types (STORY-041 Task 3):
//! - `word/document.xml` → `application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml`
//! - `word/styles.xml` → `application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml`
//! - `word/numbering.xml` → `application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml`
//! - `word/settings.xml` → `application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml`

use crate::error::ExportError;

/// Builds the `[Content_Types].xml` byte buffer for a `.docx` archive.
///
/// # Errors
///
/// Returns [`ExportError::OoxmlError`] if the XML cannot be constructed.
pub fn build_content_types() -> Result<Vec<u8>, ExportError> {
    todo!()
}
