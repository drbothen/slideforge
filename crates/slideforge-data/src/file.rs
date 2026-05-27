//! File-based [`DataSource`] implementation.
//!
//! [`FileDataSource`] implements the [`slideforge_plugin_api::DataSource`] trait
//! for local file paths. It detects the file format from the extension and
//! dispatches to the appropriate parser in [`crate::parse`].

use std::path::Path;

use slideforge_plugin_api::{DataSource, DataSourceError, DataSourceOptions};
use slideforge_types::Value;

#[allow(unused_imports)]
use crate::format::DataFormat;
#[allow(unused_imports)]
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
/// are resolved against the working directory of the slideforge process.
///
/// ## Error handling
///
/// - Missing file → [`DataSourceError::IoError`] wrapping [`DataError::FileNotFound`]
/// - Unsupported extension → [`DataSourceError::UnsupportedUri`]
/// - Parse failure → [`DataSourceError::ParseError`]
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
    /// This is the internal method exercised by unit tests; the public
    /// [`DataSource::load`] delegates to this.
    ///
    /// # Errors
    ///
    /// Returns [`DataError`] on file-not-found, unsupported extension, or parse failure.
    pub fn load_path(&self, path: &Path) -> Result<Value, DataError> {
        // Determine format from extension before reading the file.
        let format = DataFormat::from_path(path).ok_or_else(|| {
            DataError::unsupported_format(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_owned(),
            )
        })?;

        // Read the file contents.
        let path_str = path.to_string_lossy().into_owned();
        let contents = std::fs::read_to_string(path).map_err(|_| {
            DataError::file_not_found(path_str.clone())
        })?;

        // Dispatch to the appropriate parser.
        match format {
            DataFormat::Json => json::parse_json(&contents, &path_str),
            DataFormat::Csv => csv::parse_csv(&contents, &path_str),
            DataFormat::Yaml => yaml::parse_yaml(&contents, &path_str),
            DataFormat::Toml => toml::parse_toml(&contents, &path_str),
            DataFormat::Xlsx | DataFormat::Sqlite => Err(DataError::unsupported_format(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_owned(),
            )),
        }
    }
}

impl DataSource for FileDataSource {
    #[allow(clippy::unnecessary_literal_bound)]
    fn id(&self) -> &str {
        "file"
    }

    fn load(&self, uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        let path = Path::new(uri);
        self.load_path(path).map_err(|e| match e {
            DataError::FileNotFound { path, .. } => DataSourceError::IoError {
                uri: uri.to_owned(),
                message: format!("file not found: {path}"),
            },
            DataError::UnsupportedFormat { extension, .. } => DataSourceError::UnsupportedUri {
                uri: format!("{uri} (unsupported extension: {extension})"),
            },
            DataError::ParseError { message, .. } => DataSourceError::ParseError {
                uri: uri.to_owned(),
                message,
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
    #[allow(unused_imports)]
    use tempfile::Builder;

    use super::*;

    fn loader() -> FileDataSource {
        FileDataSource::new()
    }

    /// test_BC_5_03_007_file_not_found — non-existent path → DataError::FileNotFound.
    #[test]
    fn test_bc_5_03_007_file_not_found() {
        let src = loader();
        let path = Path::new("/tmp/__slideforge_nonexistent_12345.json");
        let result = src.load_path(path);
        let err = result.expect_err("missing file must return Err");
        assert_eq!(
            err.code(),
            "E-DAT-001",
            "file-not-found must carry E-DAT-001"
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
        let value = src.load_path(f.path()).expect(".json file must parse");
        let map = value.as_map().expect("must be map");
        assert!(map.get("key").is_some(), "JSON key must be present");
    }

    /// test_BC_5_03_007_file_format_dispatch_csv — .csv file dispatches to CSV parser.
    #[test]
    fn test_bc_5_03_007_file_format_dispatch_csv() {
        let f = temp_file_with_suffix(".csv", b"col\nval");

        let src = loader();
        let value = src.load_path(f.path()).expect(".csv file must parse");
        let list = value.as_list().expect("CSV must produce list");
        assert_eq!(list.len(), 1);
    }

    /// test_BC_5_03_007_file_format_dispatch_yaml — .yaml file dispatches to YAML parser.
    #[test]
    fn test_bc_5_03_007_file_format_dispatch_yaml() {
        let f = temp_file_with_suffix(".yaml", b"x: 1\n");

        let src = loader();
        let value = src.load_path(f.path()).expect(".yaml file must parse");
        assert!(value.as_map().is_some(), "YAML must produce map");
    }

    /// test_BC_5_03_007_file_format_dispatch_yml — .yml file dispatches to YAML parser (EC-008).
    #[test]
    fn test_bc_5_03_007_file_format_dispatch_yml() {
        let f = temp_file_with_suffix(".yml", b"x: 1\n");

        let src = loader();
        let value = src.load_path(f.path()).expect(".yml file must parse");
        assert!(value.as_map().is_some(), ".yml must be treated as YAML");
    }

    /// test_BC_5_03_007_file_format_dispatch_toml — .toml file dispatches to TOML parser.
    #[test]
    fn test_bc_5_03_007_file_format_dispatch_toml() {
        let f = temp_file_with_suffix(".toml", b"x = 1\n");

        let src = loader();
        let value = src.load_path(f.path()).expect(".toml file must parse");
        assert!(value.as_map().is_some(), "TOML must produce map");
    }

    /// test_BC_5_03_007_unsupported_extension — .txt file → DataError::UnsupportedFormat.
    #[test]
    fn test_bc_5_03_007_unsupported_extension() {
        let f = temp_file_with_suffix(".txt", b"hello");

        let src = loader();
        let result = src.load_path(f.path());
        let err = result.expect_err(".txt must return Err");
        assert_eq!(
            err.code(),
            "E-DAT-005",
            "unsupported extension must carry E-DAT-005"
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
}
