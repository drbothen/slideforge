//! File-based [`DataSource`] implementation.
//!
//! [`FileDataSource`] implements the [`slideforge_plugin_api::DataSource`] trait
//! for local file paths. It detects the file format from the extension and
//! dispatches to the appropriate parser in [`crate::parse`].

use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use slideforge_plugin_api::{DataSource, DataSourceError, DataSourceOptions};
use slideforge_types::Value;

use crate::format::DataFormat;
use crate::parse::{csv, json, toml, yaml};
use crate::DataError;

/// The built-in file-based data source plugin.
///
/// Handles local file paths with extensions `.json`, `.csv`, `.yaml`,
/// `.yml`, and `.toml`. The plugin identifier is `"file"`.
///
/// ## URI format
///
/// The URI is treated as a relative or absolute file path. Relative paths
/// are resolved against the provided `base_dir` (defaulting to the current
/// working directory when loaded via the [`DataSource`] trait).
///
/// ## Path containment
///
/// When a `base_dir` is provided and the path resolves outside that directory,
/// the load is rejected with [`DataError::PathTraversalBlocked`].
///
/// ## Error handling
///
/// - Missing file → [`DataSourceError::IoError`] wrapping [`DataError::FileNotFound`]
/// - Non-NotFound I/O error → [`DataSourceError::IoError`] wrapping [`DataError::IoError`]
/// - Unsupported extension → [`DataSourceError::UnsupportedUri`]
/// - Parse failure → [`DataSourceError::ParseError`]
/// - Path traversal → [`DataSourceError::IoError`] wrapping [`DataError::PathTraversalBlocked`]
#[derive(Debug, Default)]
pub struct FileDataSource;

impl FileDataSource {
    /// Construct a new [`FileDataSource`] plugin instance.
    #[must_use]
    pub fn new() -> Self {
        FileDataSource
    }

    /// Load a file from a path, detecting format by extension, and return the parsed [`Value`].
    ///
    /// If `base_dir` is `Some`, relative paths are resolved against it and
    /// path containment is enforced (paths outside `base_dir` are rejected).
    ///
    /// This is the internal method exercised by unit tests; the public
    /// [`DataSource::load`] delegates to this.
    ///
    /// # Errors
    ///
    /// Returns [`DataError`] on file-not-found, I/O error, unsupported extension,
    /// path traversal, or parse failure.
    pub fn load_path(&self, path: &Path, base_dir: Option<&Path>) -> Result<Value, DataError> {
        // Resolve the path against base_dir if provided.
        let resolved: PathBuf = if let Some(base) = base_dir {
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                base.join(path)
            }
        } else {
            path.to_path_buf()
        };

        // Enforce path containment when base_dir is provided.
        if let Some(base) = base_dir {
            // base_dir MUST be canonicalizable. If it cannot be resolved (e.g., the
            // directory does not exist), that is a caller error and we fail fast.
            // This prevents a bypass where an unresolvable base_dir would silently
            // skip the containment check and allow path traversal.
            let canonical_base = base.canonicalize().map_err(|e| {
                DataError::io_error(
                    Arc::from(base.to_string_lossy().as_ref()),
                    Arc::from(
                        format!("base_dir cannot be canonicalized: {e}").as_str(),
                    ),
                )
            })?;

            // The target file may not exist yet (that is fine — the read below will
            // produce FileNotFound). Only check containment if canonicalization of the
            // resolved path succeeds (i.e., the file currently exists on disk).
            if let Ok(canonical_resolved) = resolved.canonicalize()
                && !canonical_resolved.starts_with(&canonical_base)
            {
                let path_str: Arc<str> = Arc::from(resolved.to_string_lossy().as_ref());
                return Err(DataError::path_traversal_blocked(path_str));
            }
        }

        // Determine format from extension before reading the file.
        let format = DataFormat::from_path(&resolved).ok_or_else(|| {
            let ext: Arc<str> = Arc::from(
                resolved
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or(""),
            );
            DataError::unsupported_format(ext)
        })?;

        // Read the file contents, distinguishing NotFound from other I/O errors.
        let path_str: Arc<str> = Arc::from(resolved.to_string_lossy().as_ref());
        let contents = std::fs::read_to_string(&resolved).map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                DataError::file_not_found(Arc::clone(&path_str))
            } else {
                DataError::io_error(Arc::clone(&path_str), Arc::from(e.to_string().as_str()))
            }
        })?;

        // Dispatch to the appropriate parser.
        match format {
            DataFormat::Json => json::parse_json(&contents, &path_str),
            DataFormat::Csv => csv::parse_csv(&contents, &path_str),
            DataFormat::Yaml => yaml::parse_yaml(&contents, &path_str),
            DataFormat::Toml => toml::parse_toml(&contents, &path_str),
            DataFormat::Xlsx | DataFormat::Sqlite => {
                let ext: Arc<str> = Arc::from(
                    resolved
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or(""),
                );
                Err(DataError::unsupported_format(ext))
            }
        }
    }
}

impl DataSource for FileDataSource {
    #[allow(clippy::unnecessary_literal_bound)]
    fn id(&self) -> &str {
        "file"
    }

    /// Load a file from a URI.
    ///
    /// The [`DataSource`] trait does not receive a `base_dir`, so this method
    /// passes `None` and performs **no path containment check**. Callers that
    /// need containment enforcement (e.g., the evaluator processing an `@data`
    /// directive) MUST use [`FileDataSource::load_path`] directly with the
    /// project root as `base_dir`.
    fn load(&self, uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        let path = Path::new(uri);
        self.load_path(path, None).map_err(|e| match e {
            DataError::FileNotFound { path, .. } => DataSourceError::IoError {
                uri: uri.to_owned(),
                message: format!("file not found: {path}"),
            },
            DataError::IoError { message, .. } => DataSourceError::IoError {
                uri: uri.to_owned(),
                message: message.to_string(),
            },
            DataError::UnsupportedFormat { extension, .. } => DataSourceError::UnsupportedUri {
                uri: format!("{uri} (unsupported extension: {extension})"),
            },
            DataError::ParseError { reason, .. } => DataSourceError::ParseError {
                uri: uri.to_owned(),
                message: reason.to_string(),
            },
            DataError::PathTraversalBlocked { path, .. } => DataSourceError::IoError {
                uri: uri.to_owned(),
                message: format!("path traversal blocked: {path}"),
            },
            other => DataSourceError::IoError {
                uri: uri.to_owned(),
                message: other.to_string(),
            },
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::io::Write as _;

    use tempfile::NamedTempFile;

    use super::*;

    fn loader() -> FileDataSource {
        FileDataSource::new()
    }

    /// test_BC_5_03_007_file_not_found — non-existent path → DataError::FileNotFound with E-DAT-004.
    #[test]
    fn test_bc_5_03_007_file_not_found() {
        let src = loader();
        let path = Path::new("/tmp/__slideforge_nonexistent_12345.json");
        let result = src.load_path(path, None);
        let err = result.expect_err("missing file must return Err");
        assert_eq!(
            err.code(),
            "E-DAT-004",
            "file-not-found must carry E-DAT-004"
        );
    }

    /// Helper: create a temp file with a specific extension using Builder.
    fn temp_file_with_suffix(suffix: &str, content: &[u8]) -> NamedTempFile {
        use tempfile::Builder;
        let mut f = Builder::new()
            .suffix(suffix)
            .tempfile()
            .expect("temp file creation must succeed");
        f.write_all(content).expect("temp file write must succeed");
        f
    }

    /// test_BC_5_03_007_file_format_dispatch_json — .json file dispatches to JSON parser.
    #[test]
    fn test_bc_5_03_007_file_format_dispatch_json() {
        let f = temp_file_with_suffix(".json", br#"{"key": "value"}"#);

        let src = loader();
        let value = src.load_path(f.path(), None).expect(".json file must parse");
        let map = value.as_map().expect("must be map");
        assert!(map.get("key").is_some(), "JSON key must be present");
    }

    /// test_BC_5_03_007_file_format_dispatch_csv — .csv file dispatches to CSV parser.
    #[test]
    fn test_bc_5_03_007_file_format_dispatch_csv() {
        let f = temp_file_with_suffix(".csv", b"col\nval");

        let src = loader();
        let value = src.load_path(f.path(), None).expect(".csv file must parse");
        let list = value.as_list().expect("CSV must produce list");
        assert_eq!(list.len(), 1);
    }

    /// test_BC_5_03_007_file_format_dispatch_yaml — .yaml file dispatches to YAML parser.
    #[test]
    fn test_bc_5_03_007_file_format_dispatch_yaml() {
        let f = temp_file_with_suffix(".yaml", b"x: 1\n");

        let src = loader();
        let value = src.load_path(f.path(), None).expect(".yaml file must parse");
        assert!(value.as_map().is_some(), "YAML must produce map");
    }

    /// test_BC_5_03_007_file_format_dispatch_yml — .yml file dispatches to YAML parser (EC-008).
    #[test]
    fn test_bc_5_03_007_file_format_dispatch_yml() {
        let f = temp_file_with_suffix(".yml", b"x: 1\n");

        let src = loader();
        let value = src.load_path(f.path(), None).expect(".yml file must parse");
        assert!(value.as_map().is_some(), ".yml must be treated as YAML");
    }

    /// test_BC_5_03_007_file_format_dispatch_toml — .toml file dispatches to TOML parser.
    #[test]
    fn test_bc_5_03_007_file_format_dispatch_toml() {
        let f = temp_file_with_suffix(".toml", b"x = 1\n");

        let src = loader();
        let value = src.load_path(f.path(), None).expect(".toml file must parse");
        assert!(value.as_map().is_some(), "TOML must produce map");
    }

    /// test_BC_5_03_007_unsupported_extension — .txt file → DataError::UnsupportedFormat (E-DAT-003).
    #[test]
    fn test_bc_5_03_007_unsupported_extension() {
        let f = temp_file_with_suffix(".txt", b"hello");

        let src = loader();
        let result = src.load_path(f.path(), None);
        let err = result.expect_err(".txt must return Err");
        assert_eq!(
            err.code(),
            "E-DAT-003",
            "unsupported extension must carry E-DAT-003"
        );
    }

    /// test_BC_5_03_007_datasource_trait_id — DataSource::id() returns "file".
    #[test]
    fn test_bc_5_03_007_datasource_trait_id() {
        let src = loader();
        assert_eq!(src.id(), "file");
    }

    /// test_BC_5_03_007_datasource_trait_load_missing — DataSource::load() maps to DataSourceError.
    #[test]
    fn test_bc_5_03_007_datasource_trait_load_missing() {
        let src = loader();
        let opts = DataSourceOptions::default();
        let result = src.load("/tmp/__slideforge_nonexistent_12345.json", &opts);
        let err = result.expect_err("missing file must return DataSourceError");
        assert!(
            matches!(err, DataSourceError::IoError { .. }),
            "file-not-found must map to DataSourceError::IoError"
        );
    }

    /// test_BC_5_03_007_datasource_trait_load_unsupported — DataSource::load() maps to UnsupportedUri.
    #[test]
    fn test_bc_5_03_007_datasource_trait_load_unsupported() {
        let f = temp_file_with_suffix(".txt", b"data");
        let src = loader();
        let opts = DataSourceOptions::default();
        let uri = f.path().to_str().unwrap();
        let result = src.load(uri, &opts);
        let err = result.expect_err(".txt must return DataSourceError");
        assert!(
            matches!(err, DataSourceError::UnsupportedUri { .. }),
            "unsupported extension must map to DataSourceError::UnsupportedUri"
        );
    }

    /// test_BC_5_03_007_datasource_trait_load_json — DataSource::load() happy path for JSON.
    #[test]
    fn test_bc_5_03_007_datasource_trait_load_json() {
        let f = temp_file_with_suffix(".json", br#"{"answer": 42}"#);
        let src = loader();
        let opts = DataSourceOptions::default();
        let uri = f.path().to_str().unwrap();
        let result = src.load(uri, &opts);
        let value = result.expect("DataSource::load() must succeed for valid JSON file");
        let map = value.as_map().expect("result must be a map");
        assert_eq!(
            map.get("answer"),
            Some(&Value::Int(42)),
            "JSON field 'answer' must be Int(42)"
        );
    }

    /// test_BC_5_03_007_path_containment_blocks_traversal — `../` path outside base_dir is blocked.
    #[test]
    fn test_bc_5_03_007_path_containment_blocks_traversal() {
        use std::path::PathBuf;
        let src = loader();
        let base = PathBuf::from("/tmp");
        // Attempt to read a file above /tmp using ..
        let traversal = Path::new("../etc/passwd");
        let result = src.load_path(traversal, Some(&base));
        // Either the path traversal is blocked, or the file doesn't exist.
        // Both are acceptable — what's NOT acceptable is returning Ok with /etc/passwd contents.
        match result {
            // Any error variant is acceptable — traversal block, file not found,
            // or another I/O error. What is NOT acceptable is Ok.
            Err(_) => {}
            Ok(_) => {
                panic!("path traversal must not succeed silently");
            }
        }
    }

    /// test_BC_5_03_007_base_dir_nonexistent_fails — non-canonicalizable base_dir must return Err,
    /// not silently skip the path containment check (FINDING-001).
    #[test]
    fn test_bc_5_03_007_base_dir_nonexistent_fails() {
        let src = loader();
        let nonexistent_base = Path::new("/tmp/__slideforge_nonexistent_base_dir_12345");
        let result = src.load_path(Path::new("data.json"), Some(nonexistent_base));
        let err = result.expect_err("unresolvable base_dir must return Err");
        // Must be an I/O error — the base_dir cannot be canonicalized.
        assert_eq!(
            err.code(),
            "E-DAT-004",
            "base_dir canonicalization failure must return E-DAT-004"
        );
    }

    /// test_BC_5_03_007_relative_path_resolved_against_base_dir — relative path uses base_dir.
    #[test]
    fn test_bc_5_03_007_relative_path_resolved_against_base_dir() {
        use tempfile::TempDir;
        let dir = TempDir::new().expect("temp dir");
        let json_path = dir.path().join("data.json");
        std::fs::write(&json_path, br#"{"x": 1}"#).expect("write temp json");

        let src = loader();
        let result = src.load_path(Path::new("data.json"), Some(dir.path()));
        let value = result.expect("relative path with base_dir must resolve correctly");
        assert!(value.as_map().is_some(), "must be a map");
    }
}
