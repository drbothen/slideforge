//! Shared helpers for STORY-050 end-to-end integration tests.
//!
//! ## Purpose
//!
//! Every E2E test that calls `slideforge::build()` must supply a
//! `BrandSource::TomlFile` pointing to a real `brand.toml` on disk.
//! The `make_brand_tmpdir()` helper creates a self-contained temporary
//! directory with:
//!
//! - `brand.toml` — minimal slideforge brand configuration
//! - `logo.png` — minimal valid PNG header (8 bytes)
//!
//! The absolute path to `brand.toml` is returned for use in `BuildOptions`.
//!
//! ## Why we cannot use the fixture brand.toml directly
//!
//! `BrandSynthesizer::load()` resolves `[logo].path` relative to the brand TOML
//! file's parent directory. The `tests/fixtures/brand.toml` has `path = "logo.png"`
//! which would resolve to `tests/fixtures/logo.png` — a file that may not exist
//! in all CI environments. To avoid this coupling, E2E tests always create a
//! self-contained tmpdir with both files.
//!
//! ## Traceability
//!
//! - STORY-050 AC-001 through AC-009, EC-001 through EC-005
//! - Mirrors the tmpdir setup pattern from `test_crit2_end_to_end_build_ok_with_toml_brand`
//!   in `crates/slideforge/src/lib.rs`.

#![allow(clippy::unwrap_used)] // test helpers — panics are intentional

use std::path::PathBuf;
use std::sync::Arc;

use slideforge::BuildOptions;
use slideforge_plugin_api::BrandSource;

/// Return the absolute path to a fixture file in `tests/fixtures/`.
///
/// Panics if the file cannot be read (fixture is missing or corrupt).
#[must_use]
pub fn fixture_source(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("STORY-050: cannot read fixture '{name}': {e}"))
}

/// A self-contained temporary directory containing `brand.toml` + `logo.png`.
///
/// Drop value is `keep` — the directory is cleaned up when this struct is dropped
/// via `Drop`. Tests hold on to this until the test body completes.
pub struct BrandTmpDir {
    /// The directory on disk (cleaned up on drop).
    dir: PathBuf,
    /// Absolute path to `brand.toml` inside `dir`.
    pub brand_toml_path: PathBuf,
}

impl BrandTmpDir {
    /// Create a new tmpdir with `brand.toml` + `logo.png`.
    ///
    /// Uses a unique subdirectory name derived from `label` so that
    /// concurrent test threads do not collide.
    #[must_use]
    pub fn new(label: &str) -> Self {
        use std::io::Write as _;
        use std::time::SystemTime;

        let ts = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "slideforge_e2e_{label}_{pid}_{ts}",
            pid = std::process::id(),
        ));
        std::fs::create_dir_all(&dir)
            .unwrap_or_else(|e| panic!("STORY-050: cannot create brand tmpdir: {e}"));

        // Minimal 8-byte PNG header — satisfies BrandSynthesizer logo.exists() check.
        let logo_bytes: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let logo_path = dir.join("logo.png");
        {
            let mut f = std::fs::File::create(&logo_path)
                .unwrap_or_else(|e| panic!("STORY-050: cannot create logo.png: {e}"));
            f.write_all(logo_bytes)
                .unwrap_or_else(|e| panic!("STORY-050: cannot write logo.png: {e}"));
        }

        // Minimal brand.toml with logo path pointing to logo.png in same directory.
        // BrandSynthesizer resolves relative paths against the brand.toml's parent dir.
        let brand_toml_content = concat!(
            "[colors]\n",
            "dk1 = \"#1F2937\"\n",
            "acc1 = \"#3B82F6\"\n",
            "\n",
            "[fonts]\n",
            "heading = \"Arial\"\n",
            "body = \"Arial\"\n",
            "\n",
            "[logo]\n",
            "path = \"logo.png\"\n",
        );
        let brand_toml_path = dir.join("brand.toml");
        {
            let mut f = std::fs::File::create(&brand_toml_path)
                .unwrap_or_else(|e| panic!("STORY-050: cannot create brand.toml: {e}"));
            f.write_all(brand_toml_content.as_bytes())
                .unwrap_or_else(|e| panic!("STORY-050: cannot write brand.toml: {e}"));
        }

        Self {
            dir,
            brand_toml_path,
        }
    }

    /// Construct `BuildOptions` with the brand source pointing to `brand.toml`.
    ///
    /// `format` is the output format string (e.g., `"pptx"`, `"docx"`, `"pdf"`).
    /// `strict` controls strict validation mode.
    #[must_use]
    pub fn build_options(&self, format: &str, strict: bool) -> BuildOptions {
        BuildOptions {
            brand_source: Some(BrandSource::TomlFile(Arc::from(
                self.brand_toml_path.to_string_lossy().as_ref(),
            ))),
            format: Some(format.to_owned()),
            strict,
        }
    }
}

impl Drop for BrandTmpDir {
    fn drop(&mut self) {
        // Best-effort cleanup — ignore errors (tmpdir may already be cleaned by OS).
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Read a `zip::ZipArchive` from raw bytes.
///
/// Returns `Ok(archive)` or panics with a descriptive message.
#[must_use]
pub fn open_zip<'a>(bytes: &'a [u8], label: &str) -> zip::ZipArchive<std::io::Cursor<&'a [u8]>> {
    zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .unwrap_or_else(|e| panic!("STORY-050 {label}: output bytes are not a valid ZIP: {e}"))
}

/// Assert that `archive` contains a file named `entry_name`.
pub fn assert_zip_contains(
    archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
    entry_name: &str,
    label: &str,
) {
    let names: Vec<String> = archive.file_names().map(str::to_owned).collect();
    assert!(
        names.iter().any(|n| n == entry_name),
        "STORY-050 {label}: ZIP does not contain '{entry_name}'; entries: {names:?}"
    );
}
