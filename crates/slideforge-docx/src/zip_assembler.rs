//! DOCX ZIP assembler.
//!
//! [`DocxZipAssembler`] collects all OOXML parts (strings of XML bytes keyed
//! by their ZIP entry path) and writes them into a `.docx`-compatible ZIP
//! archive. Entry ordering is deterministic (alphabetical on path) and
//! timestamps are fixed to the MS-DOS epoch for reproducible builds.

use std::collections::BTreeMap;
use std::io::Write;

use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

use crate::error::ExportError;

/// Assembles a collection of named XML parts into a `.docx` ZIP archive.
///
/// # Determinism guarantee
///
/// All ZIP entries are written in lexicographic path order. All entry
/// timestamps are set to the MS-DOS epoch (1980-01-01 00:00:00). This
/// guarantees that identical inputs produce byte-identical output, enabling
/// SHA-256-based determinism tests (STORY-041 Test Strategy).
pub struct DocxZipAssembler {
    /// Accumulated parts keyed by ZIP path. `BTreeMap` provides deterministic
    /// lexicographic ordering at iteration time, satisfying the invariant that
    /// ZIP entries are written in alphabetical path order (BC-4.02.001
    /// invariant 4 / STORY-041 Test Strategy).
    parts: BTreeMap<String, Vec<u8>>,
}

impl DocxZipAssembler {
    /// Create a new, empty assembler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            parts: BTreeMap::new(),
        }
    }

    /// Add a named part.
    ///
    /// `path` is the ZIP entry path (e.g., `"word/document.xml"`).
    /// `content` is the raw UTF-8 XML bytes for that part.
    ///
    /// If a part with the same path is added twice, the later content wins.
    pub fn add_part(&mut self, path: impl Into<String>, content: Vec<u8>) {
        self.parts.insert(path.into(), content);
    }

    /// Finalize all parts into a `.docx` ZIP byte buffer.
    ///
    /// Parts are written in lexicographic path order. All entries use
    /// `DateTime::default()` (MS-DOS epoch 1980-01-01 00:00:00) for
    /// deterministic output.
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::ZipError`] if the ZIP writer encounters an I/O
    /// failure.
    pub fn finish(self) -> Result<Vec<u8>, ExportError> {
        let buf = Vec::new();
        let cursor = std::io::Cursor::new(buf);
        let mut writer = zip::ZipWriter::new(cursor);

        // MS-DOS epoch timestamp for deterministic output.
        let epoch = zip::DateTime::default();
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .last_modified_time(epoch);

        // BTreeMap iterates in lexicographic key order — deterministic entry order.
        for (path, content) in &self.parts {
            writer
                .start_file(path.as_str(), options)
                .map_err(|e| ExportError::ZipError {
                    message: format!("failed to start ZIP entry {path:?}: {e}"),
                })?;
            writer
                .write_all(content)
                .map_err(|e| ExportError::ZipError {
                    message: format!("failed to write ZIP entry {path:?}: {e}"),
                })?;
        }

        let cursor = writer.finish().map_err(|e| ExportError::ZipError {
            message: format!("failed to finalize ZIP archive: {e}"),
        })?;

        Ok(cursor.into_inner())
    }
}

impl Default for DocxZipAssembler {
    fn default() -> Self {
        Self::new()
    }
}
