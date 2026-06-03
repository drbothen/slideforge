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

use std::io::{Cursor, Write as _};

use zip::write::SimpleFileOptions;

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
    /// Parts are sorted alphabetically before writing to guarantee byte-identical
    /// output for the same input (AC-007).
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::Zip`] if the underlying `zip` crate fails to
    /// write or finalise the archive.
    pub fn assemble(mut parts: Vec<ZipPart>) -> Result<Vec<u8>, PptxError> {
        // Deterministic ordering: sort parts alphabetically by path.
        parts.sort_by(|a, b| a.path.cmp(&b.path));

        let buf = Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(buf);

        // Use epoch timestamp (1980-01-01 00:00:00) for all entries.
        // `zip::DateTime::default()` is 1980-01-01 00:00:00 in zip 4.x.
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .last_modified_time(zip::DateTime::default());

        for part in parts {
            tracing::debug!(path = %part.path, bytes = part.bytes.len(), "writing ZIP part");
            zip.start_file(&part.path, options)?;
            zip.write_all(&part.bytes).map_err(PptxError::Io)?;
        }

        let cursor = zip.finish()?;
        Ok(cursor.into_inner())
    }
}
