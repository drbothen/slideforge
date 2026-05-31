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
    /// Returns [`PdfExportError::Io`] if the file cannot be read.
    pub fn load(path: &std::path::Path) -> Result<Self, PdfExportError> {
        let bytes = std::fs::read(path).map_err(|e| PdfExportError::Io {
            message: format!("failed to read font file '{}': {e}", path.display()),
        })?;
        Ok(Self(bytes))
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

/// Load raw font file bytes from `path`.
///
/// This is the AC-004 font loading function. It reads the font file bytes into
/// memory so they can be passed to krilla's font API. The loaded bytes are
/// consumed by krilla's internal subsetting mechanism when drawing text via
/// `Surface`.
///
/// # Errors
///
/// Returns [`PdfExportError::Io`] if the file cannot be read (e.g., it does
/// not exist, or the process lacks read permission). The error message includes
/// the path for diagnostic context.
pub fn load_font_data(path: &std::path::Path) -> Result<Vec<u8>, PdfExportError> {
    std::fs::read(path).map_err(|e| PdfExportError::Io {
        message: format!("failed to read font file '{}': {e}", path.display()),
    })
}

/// Best-effort lookup of a system font file by family name.
///
/// Scans the standard per-platform font directories for a `.ttf`, `.otf`, or
/// `.ttc` file whose file stem matches `family` after normalizing both sides
/// (lowercased, spaces and hyphens stripped). Returns the path of the first
/// match, or `None` if no match is found.
///
/// ## Platform font directories searched
///
/// - **macOS:** `/System/Library/Fonts`, `/Library/Fonts`,
///   `~/Library/Fonts`
/// - **Linux:** `/usr/share/fonts`, `/usr/local/share/fonts`, `~/.fonts`
/// - **Windows:** `%WINDIR%\Fonts` (falls back to `C:\Windows\Fonts` if the
///   env var is absent)
///
/// ## Important caveats
///
/// This is a **best-effort, filename-based** fallback. It does NOT do full
/// font family/subfamily matching, does NOT read font metadata, and does NOT
/// handle collection files (`.ttc`) that bundle multiple families in one file.
/// Real font resolution (with family name lookup via font metadata) happens in
/// STORY-044's draw pass. Use this only as a convenience fallback when a
/// specific font file path is not already known.
///
/// Returns `None` (without panicking) if:
/// - The font directory cannot be read.
/// - No file with a matching stem is found.
/// - The home directory cannot be determined.
#[must_use]
pub fn system_font_fallback(family: &str) -> Option<std::path::PathBuf> {
    let needle = normalize_font_name(family);
    if needle.is_empty() {
        return None;
    }

    let dirs = system_font_dirs();

    for dir in dirs {
        if let Some(found) = search_font_dir(&dir, &needle) {
            return Some(found);
        }
    }

    None
}

/// Normalize a font family name for comparison: lowercase, strip spaces and hyphens.
fn normalize_font_name(name: &str) -> String {
    name.chars()
        .filter(|c| *c != ' ' && *c != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

/// Returns the platform-specific list of directories to search for font files.
fn system_font_dirs() -> Vec<std::path::PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let mut dirs = vec![
            std::path::PathBuf::from("/System/Library/Fonts"),
            std::path::PathBuf::from("/Library/Fonts"),
        ];
        if let Some(home) = home_dir() {
            dirs.push(home.join("Library/Fonts"));
        }
        dirs
    }

    #[cfg(target_os = "windows")]
    {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| String::from("C:\\Windows"));
        vec![std::path::PathBuf::from(windir).join("Fonts")]
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // Linux and other Unix-like systems.
        let mut dirs = vec![
            std::path::PathBuf::from("/usr/share/fonts"),
            std::path::PathBuf::from("/usr/local/share/fonts"),
        ];
        if let Some(home) = home_dir() {
            dirs.push(home.join(".fonts"));
        }
        dirs
    }
}

/// Attempt to determine the current user's home directory.
///
/// Uses the `HOME` env var (Unix/macOS) or `USERPROFILE` (Windows).
/// Returns `None` if neither is set.
fn home_dir() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE")
            .ok()
            .map(std::path::PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").ok().map(std::path::PathBuf::from)
    }
}

/// Search a single directory (recursively) for a font file matching `needle`.
///
/// Returns the path of the first match found, or `None`.
fn search_font_dir(dir: &std::path::Path, needle: &str) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Recurse into subdirectories (common on Linux: /usr/share/fonts/truetype/…).
            if let Some(found) = search_font_dir(&path, needle) {
                return Some(found);
            }
        } else {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(str::to_lowercase);
            if let Some("ttf" | "otf" | "ttc") = ext.as_deref() {
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(normalize_font_name)
                    .unwrap_or_default();
                if stem == needle {
                    return Some(path);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// BC-4.03.002 AC-004: `FontBytes::load` reads a font file from disk.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_bc_4_03_002_font_bytes_load_reads_file() {
        // Create a temp file with fake font bytes to avoid filesystem dependency.
        let mut tmp = NamedTempFile::new().expect("failed to create temp file");
        tmp.write_all(b"FAKE_FONT_DATA_FOR_TEST")
            .expect("failed to write temp font data");
        let path = tmp.path();

        let result = FontBytes::load(path);
        assert!(
            result.is_ok(),
            "FontBytes::load must succeed for a valid file"
        );
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

    /// BC-4.03.002 AC-004: `load_font_data` reads a real font file's bytes.
    ///
    /// Uses a `tempfile` to avoid filesystem coupling; verifies the exact bytes
    /// round-trip correctly.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_bc_4_03_002_load_font_data_reads_file() {
        let mut tmp = NamedTempFile::new().expect("failed to create temp file");
        tmp.write_all(b"FAKE_FONT_BYTES_FOR_LOAD_FONT_DATA")
            .expect("failed to write temp font data");
        let path = tmp.path();

        let result = load_font_data(path);
        assert!(
            result.is_ok(),
            "load_font_data must succeed for a valid path"
        );
        assert_eq!(
            result.unwrap(),
            b"FAKE_FONT_BYTES_FOR_LOAD_FONT_DATA",
            "load_font_data must return exact file contents"
        );
    }

    /// BC-4.03.002 AC-004: `load_font_data` returns `Err(PdfExportError::Io)`
    /// for a path that does not exist.
    #[test]
    fn test_bc_4_03_002_load_font_data_errors_on_missing_path() {
        let missing = std::path::Path::new("/tmp/__nonexistent_font_for_slideforge_test__.ttf");
        let result = load_font_data(missing);
        assert!(
            result.is_err(),
            "load_font_data must return Err for a non-existent path"
        );
        // Verify the error is an Io variant (not a panic).
        match result {
            Err(PdfExportError::Io { message }) => {
                assert!(
                    message.contains("__nonexistent_font_for_slideforge_test__"),
                    "Io error message must include the path: {message}"
                );
            },
            Err(other) => panic!("expected PdfExportError::Io, got {other:?}"),
            Ok(_) => panic!("expected Err, got Ok"),
        }
    }

    /// BC-4.03.002 AC-004: `system_font_fallback` returns `None` for a
    /// guaranteed-absent family name without panicking.
    ///
    /// This is the CI-safe assertion: `NoSuchFontFamily_ZZZ_Slideforge_Test`
    /// will never exist in any system font directory.
    #[test]
    fn test_bc_4_03_002_system_font_fallback_none_for_absent_family() {
        let result = system_font_fallback("NoSuchFontFamily_ZZZ_Slideforge_Test");
        assert!(
            result.is_none(),
            "system_font_fallback must return None for a non-existent font family"
        );
    }

    /// `normalize_font_name` strips spaces and hyphens and lowercases.
    #[test]
    fn test_normalize_font_name() {
        assert_eq!(normalize_font_name("Helvetica Neue"), "helveticaneue");
        assert_eq!(normalize_font_name("Open-Sans"), "opensans");
        assert_eq!(normalize_font_name("Arial"), "arial");
        assert_eq!(normalize_font_name(""), "");
    }
}
