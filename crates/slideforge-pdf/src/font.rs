//! Font file loading helpers for `slideforge-pdf`.
//!
//! Font subsetting for text drawn via krilla's `Surface`/text API is handled
//! **internally** by krilla — no `subsetter::subset(...)` call is needed for
//! the normal text-rendering path (BC-4.03.002 AC-004, tech-validation RISK-2).
//!
//! This module provides helpers to load raw font bytes from disk so they can be
//! passed to krilla's font API. The loaded bytes are consumed by krilla's
//! internal subsetting mechanism; only the glyphs actually used by the deck are
//! embedded in the output PDF.
//!
//! ## Direct `subsetter` usage
//!
//! If a concrete gap in krilla's text API is discovered during the TDD green
//! phase that requires manual glyph embedding outside krilla's Surface (unlikely
//! for this story), `subsetter::{subset, GlyphRemapper}` (transitive dep of
//! krilla, no direct Cargo dep needed) may be used. See tech-validation RISK-2
//! before adding that path.

use crate::error::PdfExportError;

/// Raw font bytes ready for use with krilla's font API.
///
/// This newtype wraps `Vec<u8>` so that font bytes are distinguishable from
/// arbitrary byte buffers at the type level. krilla's subsetting operates on
/// this raw data internally when drawing text via `Surface`.
#[derive(Debug, Clone)]
pub struct FontBytes(pub Vec<u8>);

impl FontBytes {
    /// Load a font file from `path` into memory.
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError::FontLoad`] if the file cannot be read.
    pub fn load(_path: &std::path::Path) -> Result<Self, PdfExportError> {
        todo!("STORY-043 Red Gate stub: FontBytes::load not yet implemented")
    }

    /// Construct `FontBytes` from a raw byte buffer.
    ///
    /// Used in tests to inject synthetic font data without hitting the
    /// filesystem.
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Return the raw font bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// BC-4.03.002 AC-004: `FontBytes::load` reads a font file from disk.
    ///
    /// RED GATE: This test MUST FAIL because `FontBytes::load` is a `todo!()`.
    #[test]
    fn test_bc_4_03_002_font_bytes_load_reads_file() {
        // Create a temp file with fake font bytes to avoid filesystem dependency.
        let mut tmp = NamedTempFile::new().expect("failed to create temp file");
        tmp.write_all(b"FAKE_FONT_DATA_FOR_TEST")
            .expect("failed to write temp font data");
        let path = tmp.path();

        // This panics at todo!() — confirms Red Gate is active.
        let result = FontBytes::load(path);
        assert!(result.is_ok(), "FontBytes::load must succeed for a valid file");
        let bytes = result.unwrap();
        assert_eq!(
            bytes.as_bytes(),
            b"FAKE_FONT_DATA_FOR_TEST",
            "FontBytes must contain exact file contents"
        );
    }

    /// `FontBytes::from_bytes` constructs from raw bytes without touching disk.
    ///
    /// This test PASSES immediately — it exercises the non-stub path.
    #[test]
    fn test_font_bytes_from_bytes_round_trips() {
        let data = b"TTF_FONT_DATA".to_vec();
        let fb = FontBytes::from_bytes(data.clone());
        assert_eq!(fb.as_bytes(), data.as_slice());
    }
}
