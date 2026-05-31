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
        std::fs::read(path).map(Self).map_err(|e| {
            // SEC-005: log the full path at debug level for diagnostics,
            // but keep the user-facing error generic (no internal path disclosure).
            tracing::debug!(path = %path.display(), error = %e, "failed to read font file");
            PdfExportError::Io {
                message: format!("failed to read font file: {e}"),
            }
        })
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
/// not exist, or the process lacks read permission). The full path is emitted
/// via `tracing::debug!` for diagnostics; the user-facing error message is
/// generic and does not expose internal paths (SEC-005).
pub fn load_font_data(path: &std::path::Path) -> Result<Vec<u8>, PdfExportError> {
    std::fs::read(path).map_err(|e| {
        // SEC-005: log the full path at debug level for diagnostics,
        // but keep the user-facing error generic (no internal path disclosure).
        tracing::debug!(path = %path.display(), error = %e, "failed to read font file");
        PdfExportError::Io {
            message: format!("failed to read font file: {e}"),
        }
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
///
/// ## Platform font directories searched
///
/// - **macOS:** system fonts + user `~/Library/Fonts`
/// - **Windows:** system `%WINDIR%\Fonts` + per-user
///   `%USERPROFILE%\AppData\Local\Microsoft\Windows\Fonts`
/// - **Linux / other:** `/usr/share/fonts`, `/usr/local/share/fonts`, `~/.fonts`
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
        let mut dirs = vec![std::path::PathBuf::from(windir).join("Fonts")];
        // Per-user font directory (Windows 10+): %USERPROFILE%\AppData\Local\Microsoft\Windows\Fonts
        if let Some(home) = home_dir() {
            dirs.push(
                home.join("AppData")
                    .join("Local")
                    .join("Microsoft")
                    .join("Windows")
                    .join("Fonts"),
            );
        }
        dirs
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

/// Maximum recursion depth for [`search_font_dir`].
///
/// System font directories rarely exceed 3-4 levels deep. 32 is a generous
/// ceiling that prevents stack exhaustion from a symlink loop while allowing
/// any legitimate font tree layout.
///
/// **SEC-004 (CWE-61):** Symlinks to directories are skipped outright (not
/// followed); this depth cap is belt-and-suspenders for any case the symlink
/// check misses.
const MAX_FONT_DIR_DEPTH: usize = 32;

/// Search a single directory (recursively) for a font file matching `needle`.
///
/// Returns the **lexicographically-first** match found (sorted by path), so
/// results are deterministic regardless of the OS `read_dir` iteration order.
/// Returns `None` if the directory cannot be read or no matching file exists.
///
/// **SEC-004 (CWE-61) — symlink-loop safety:**
/// - Directory entries that are symlinks are **skipped** (not followed into).
///   `path.symlink_metadata().file_type().is_dir()` is used rather than
///   `path.is_dir()` (which follows symlinks) to detect real directories.
/// - A [`MAX_FONT_DIR_DEPTH`] cap prevents stack exhaustion in edge cases.
fn search_font_dir(dir: &std::path::Path, needle: &str) -> Option<std::path::PathBuf> {
    search_font_dir_inner(dir, needle, 0)
}

/// Inner recursive implementation with explicit `depth` counter.
fn search_font_dir_inner(
    dir: &std::path::Path,
    needle: &str,
    depth: usize,
) -> Option<std::path::PathBuf> {
    if depth > MAX_FONT_DIR_DEPTH {
        tracing::warn!(
            depth,
            max = MAX_FONT_DIR_DEPTH,
            dir = %dir.display(),
            "font dir recursion depth cap exceeded — stopping traversal"
        );
        return None;
    }

    // Collect all readable entries and sort them so iteration is deterministic.
    // Non-readable entries are silently skipped (graceful handling for
    // permission-restricted system font directories).
    let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();

    for path in entries {
        // SEC-004: use symlink_metadata() so we inspect the symlink itself,
        // not its target. If the entry IS a symlink, skip it (do not follow
        // into symlinked directories — prevents infinite loops).
        let Ok(meta) = path.symlink_metadata() else {
            continue;
        };
        let file_type = meta.file_type();

        if file_type.is_symlink() {
            // Skip symlinks entirely — do not follow into symlinked directories.
            tracing::debug!(path = %path.display(), "skipping symlink during font dir scan (SEC-004)");
            continue;
        }

        if file_type.is_dir() {
            // Real directory (not a symlink) — recurse with incremented depth.
            if let Some(found) = search_font_dir_inner(&path, needle, depth + 1) {
                return Some(found);
            }
        } else if file_type.is_file() {
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
    ///
    /// SEC-005: The user-facing error message must NOT contain the internal
    /// file path (no path disclosure). The path is emitted at `tracing::debug!`
    /// level for diagnostics but is not exposed in the `PdfExportError::Io`
    /// message field.
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
                // SEC-005: message must NOT contain the internal path.
                assert!(
                    !message.contains("__nonexistent_font_for_slideforge_test__"),
                    "Io error message must NOT expose the internal file path (SEC-005): {message}"
                );
                // The message must still describe the failure generically.
                assert!(
                    message.contains("failed to read font file"),
                    "Io error message must describe the failure: {message}"
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

    /// SEC-004: `search_font_dir` skips symlinks and applies a depth cap so
    /// a self-referential symlink loop in a font directory does not cause
    /// infinite recursion or a stack overflow.
    ///
    /// This test creates a temp directory with a self-referential symlink
    /// (`link -> .`) and verifies that `search_font_dir` returns without
    /// hanging. It is `#[cfg(unix)]` because symlink creation requires Unix
    /// semantics.
    #[cfg(unix)]
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_sec_004_symlink_loop_does_not_recurse_infinitely() {
        use std::fs;
        use std::os::unix::fs as unix_fs;

        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let dir_path = dir.path();

        // Create a self-referential symlink: link -> . (points back to parent).
        let link_path = dir_path.join("loop_link");
        unix_fs::symlink(dir_path, &link_path).expect("failed to create symlink");

        // Place a real font file so there's something to find.
        let font_path = dir_path.join("TestFont.ttf");
        fs::write(&font_path, b"FAKE_FONT").expect("write font file");

        // The symlink loop must NOT cause infinite recursion or panic.
        // The real font file must still be found (symlink is skipped, not the real dir).
        let result = search_font_dir(dir_path, "testfont");
        assert_eq!(
            result,
            Some(font_path),
            "search_font_dir must find the real font file even when a symlink loop exists"
        );
    }

    /// `search_font_dir` returns the **lexicographically-first** match when
    /// multiple files share the same normalized stem (determinism invariant).
    ///
    /// Two temp `.ttf` files whose stems both normalize to "arial" are placed in
    /// a controlled temp directory. The function must always return the
    /// lexicographically-first path, regardless of the OS-level `read_dir`
    /// order. We verify this by running the search multiple times and confirming
    /// the result is identical and equal to the expected first path.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_search_font_dir_is_deterministic_on_matching_stem() {
        use std::fs;

        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let dir_path = dir.path();

        // Create two files with the same normalized stem ("arial"):
        //   "Arial-Bold.ttf"  → normalize_font_name → "arialbold"   (no match)
        //   "Arial.ttf"       → normalize_font_name → "arial"        (match)
        //   "Arial_v2.ttf"    → normalize_font_name → "arial_v2"     (no match)
        // We want two DISTINCT files that BOTH normalize to "arial" so we can
        // test which one wins. Use a subdirectory trick: place "AArial.ttf" and
        // "BArial.ttf" — both normalize to "aarial" / "barial" — they don't
        // share a stem. Instead, use TWO files with identical stems but
        // different filenames is not possible with the current normalizer since
        // it strips only spaces and hyphens. So we test the simpler invariant:
        // given one match and one non-match, the match is returned and repeated
        // calls produce the identical path.
        let font_a = dir_path.join("AArial.ttf"); // normalizes to "aarial"
        let font_b = dir_path.join("BArial.ttf"); // normalizes to "barial"
        fs::write(&font_a, b"FAKE").expect("write font_a");
        fs::write(&font_b, b"FAKE").expect("write font_b");

        // Search for "aarial" — only font_a matches.
        let result1 = search_font_dir(dir_path, "aarial");
        let result2 = search_font_dir(dir_path, "aarial");
        assert_eq!(
            result1,
            Some(font_a.clone()),
            "search_font_dir must return the matching file"
        );
        assert_eq!(
            result1, result2,
            "search_font_dir must return the same result on repeated calls (determinism)"
        );

        // Now test that when two files share the same normalized stem, the
        // lexicographically-first path wins. We achieve identical stems by
        // writing two files with the same name in two subdirectories.
        let sub_a = dir_path.join("a_subdir");
        let sub_b = dir_path.join("b_subdir");
        fs::create_dir(&sub_a).expect("create sub_a");
        fs::create_dir(&sub_b).expect("create sub_b");

        let match_in_a = sub_a.join("CommonFont.ttf"); // normalizes to "commonfont"
        let match_in_b = sub_b.join("CommonFont.ttf"); // normalizes to "commonfont"
        fs::write(&match_in_a, b"FONT_A").expect("write match_in_a");
        fs::write(&match_in_b, b"FONT_B").expect("write match_in_b");

        // sub_a sorts before sub_b lexicographically, so match_in_a must win.
        let result = search_font_dir(dir_path, "commonfont");
        assert_eq!(
            result,
            Some(match_in_a),
            "search_font_dir must return the lexicographically-first path \
             when multiple files share the same normalized stem"
        );
    }
}
