//! Atomic output file writing for compiled slideforge artifacts.
//!
//! The [`OutputWriter`] writes output to a temporary path inside the target
//! directory and only renames to the final path on success.  On failure, the
//! temporary file is deleted so no partial output is left on disk.
//!
//! # Atomicity Contract (STORY-055 architecture rule 4)
//!
//! Write to `<output_dir>/<filename>.tmp` then `fs::rename()` to final path.
//! Never write partial output.  On any error the `.tmp` file is removed.
//!
//! # Traceability
//!
//! - BC-1.15.003 invariant 3: "No partial output in strict mode"
//! - BC-1.15.003 invariant 4: error-slide placeholders written at position in
//!   `--warn-only` mode (handled by the exporter; `OutputWriter` is agnostic)

use std::path::{Path, PathBuf};

/// Atomic output writer for a single compiled artifact.
///
/// # Usage
///
/// ```rust,no_run
/// use slideforge_cli::output::OutputWriter;
/// use std::path::Path;
///
/// let writer = OutputWriter::new(Path::new("dist"), "deck", "pptx");
/// writer.write_atomic(b"fake pptx bytes").unwrap();
/// ```
pub struct OutputWriter {
    /// The directory to write into.
    output_dir: PathBuf,
    /// The base filename (without extension) for the output artifact.
    stem: String,
    /// The file extension for the output artifact (without leading dot).
    extension: String,
}

impl OutputWriter {
    /// Construct a new `OutputWriter`.
    ///
    /// `output_dir` — the directory to write into (created if absent).
    /// `stem` — the base filename without extension (e.g., `"deck"`).
    /// `extension` — the file extension without the leading dot (e.g., `"pptx"`).
    #[must_use]
    pub fn new(output_dir: &Path, stem: &str, extension: &str) -> Self {
        Self {
            output_dir: output_dir.to_owned(),
            stem: stem.to_owned(),
            extension: extension.to_owned(),
        }
    }

    /// Return the final output path: `<output_dir>/<stem>.<extension>`.
    #[must_use]
    pub fn final_path(&self) -> PathBuf {
        self.output_dir
            .join(format!("{}.{}", self.stem, self.extension))
    }

    /// Return the temporary path used during writing:
    /// `<output_dir>/<stem>.<extension>.tmp`.
    #[must_use]
    pub fn tmp_path(&self) -> PathBuf {
        self.output_dir
            .join(format!("{}.{}.tmp", self.stem, self.extension))
    }

    /// Write `bytes` to a temporary file, then rename to the final path.
    ///
    /// If any step fails (directory creation, write, rename), the temporary
    /// file is removed and the error is returned.  The final path is never
    /// written to directly, ensuring no partial output.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if directory creation, file write, or rename
    /// fails.
    pub fn write_atomic(&self, bytes: &[u8]) -> std::io::Result<()> {
        use std::io::Write as _;

        // Ensure the output directory exists.
        std::fs::create_dir_all(&self.output_dir)?;

        let tmp = self.tmp_path();
        let final_path = self.final_path();

        // Write to the tmp file.  On any error, attempt cleanup and propagate.
        let write_result = std::fs::File::create(&tmp)
            .and_then(|mut f| f.write_all(bytes));

        if let Err(e) = write_result {
            // Best-effort removal of the partial tmp file.
            let _ = std::fs::remove_file(&tmp);
            return Err(e);
        }

        // Atomic rename: on success the final path appears atomically.
        if let Err(e) = std::fs::rename(&tmp, &final_path) {
            let _ = std::fs::remove_file(&tmp);
            return Err(e);
        }

        Ok(())
    }
}
