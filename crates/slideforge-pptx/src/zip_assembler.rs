//! ZIP archive assembler for PPTX output.
//!
//! [`ZipAssembler`] writes all PPTX parts to an in-memory
//! `zip::ZipWriter<std::io::Cursor<Vec<u8>>>` and finalises the archive into a
//! `Vec<u8>`. Entries are written in deterministic alphabetical order and use
//! the epoch timestamp `1980-01-01 00:00:00` so that identical inputs produce
//! byte-identical output (BC-4.01.001 invariant 4).
//!
//! This module does NOT construct any OOXML content — it is purely a ZIP
//! container. Content bytes are supplied by the caller.

use crate::error::PptxError;

/// A part to be written into the PPTX ZIP archive.
///
/// Each `ZipPart` is a path + raw bytes pair. The assembler writes them in
/// sorted order to guarantee deterministic archive layout.
pub struct ZipPart {
    /// Path inside the ZIP (e.g., `"ppt/slides/slide1.xml"`).
    pub path: String,
    /// Raw bytes for this part (UTF-8 XML or binary media).
    pub bytes: Vec<u8>,
}

/// Assembles a collection of [`ZipPart`] values into a valid PPTX ZIP archive.
///
/// ## Determinism guarantee (AC-007)
///
/// - Parts are written in ascending lexicographic order of `ZipPart::path`.
/// - All ZIP local-file headers use `zip::DateTime::default()` (epoch
///   `1980-01-01 00:00:00`) so timestamps never vary between runs.
/// - No system clocks, random seeds, or HashMap iteration order may influence
///   the output byte sequence.
pub struct ZipAssembler;

impl ZipAssembler {
    /// Assemble `parts` into a PPTX ZIP and return the raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::Zip`] if the underlying `zip` crate fails to
    /// write or finalise the archive.
    pub fn assemble(parts: Vec<ZipPart>) -> Result<Vec<u8>, PptxError> {
        todo!("ZipAssembler::assemble — sort parts, write to ZipWriter with epoch timestamps, finalise")
    }
}
