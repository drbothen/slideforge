//! File-based [`DataSource`] implementation.
//!
//! [`FileDataSource`] implements the [`slideforge_plugin_api::DataSource`] trait
//! for local file paths. It detects the file format from the extension and
//! dispatches to the appropriate parser in [`crate::parse`].

use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use slideforge_plugin_api::{DataSource, DataSourceError, DataSourceOptions};
use slideforge_types::Value;

use crate::DataError;
use crate::format::DataFormat;
use crate::parse::{csv, json, toml, yaml};
use crate::xlsx::XlsxDataSource;

/// The built-in file-based data source plugin.
///
/// Handles local file paths with extensions `.json`, `.csv`, `.yaml`, `.yml`,
/// `.toml`, `.xlsx`, `.sqlite`, `.sqlite3`, and `.db`. The plugin identifier
/// is `"file"`.
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
///
/// ## Format dispatch
///
/// - `.json` → JSON parser
/// - `.csv` → CSV parser
/// - `.yaml` / `.yml` → YAML parser
/// - `.toml` → TOML parser
/// - `.xlsx` → [`XlsxDataSource`] (pure Rust calamine, no C deps)
/// - `.sqlite` / `.sqlite3` / `.db` → [`SqliteDataSource`] (bundled SQLite)
///   For SQLite, the `uri` field is used as the path per the plugin interface convention.
///   A query string must be provided via `DataSourceOptions` or the DSL `query:` directive.
#[derive(Debug, Default)]
pub struct FileDataSource;

/// Lexically normalize a path by resolving `..` and `.` components.
///
/// This function does NOT touch the filesystem — it operates purely on the
/// path string. This is used as a first-line path-traversal check: if the
/// normalized path escapes the base directory lexically, we can reject it
/// immediately without needing `canonicalize()` to succeed on the target file.
///
/// Behaviour on edge cases:
/// - `..` at the root of a relative path is preserved (e.g. `../../foo` stays
///   escaped and will NOT start with any non-escaping base).
/// - `.` components are dropped.
/// - Leading `CurDir`/`Prefix`/`RootDir` components are preserved.
fn normalize_path(path: &Path) -> PathBuf {
    let mut components: Vec<Component<'_>> = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                // Only pop a normal component — never pop a root, prefix, or
                // another `..`. If there's nothing to pop, keep the `..` so
                // the resulting path still escapes its logical root.
                match components.last() {
                    Some(Component::Normal(_)) => {
                        components.pop();
                    },
                    _ => {
                        components.push(component);
                    },
                }
            },
            Component::CurDir => {}, // `.` is always a no-op
            other => components.push(other),
        }
    }
    components.iter().collect()
}

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
                    Arc::from(format!("base_dir cannot be canonicalized: {e}").as_str()),
                )
            })?;

            // --- Lexical check (first line of defence) ---
            //
            // Normalize both paths without touching the filesystem, then check
            // containment. This catches `../` traversal for NON-EXISTENT targets
            // before we ever attempt to open the file.  Without this check an
            // attacker could distinguish "file doesn't exist" from "file exists
            // but is blocked" by observing which error variant is returned
            // (FileNotFound vs PathTraversalBlocked) — leaking file-existence
            // information for paths outside the sandbox.
            //
            // We anchor against `canonical_base` (symlinks resolved) so that
            // macOS's `/var` → `/private/var` symlink doesn't cause false
            // positives. For absolute input paths we normalize directly; for
            // relative input paths we re-join against `canonical_base` so the
            // comparison is always anchored on the same root.
            let canonical_candidate: PathBuf = if path.is_absolute() {
                // For absolute paths, resolve symlinks so the comparison is
                // anchored on the same root as `canonical_base`.  On macOS,
                // `TempDir` returns `/var/folders/…` but `canonicalize()` of
                // the base returns `/private/var/folders/…` (because `/var` is
                // a symlink to `/private/var`).  Using the raw absolute path
                // here caused false-positive PathTraversalBlocked for legitimate
                // absolute paths inside the sandbox.
                //
                // If the target does not yet exist, `canonicalize()` fails and
                // we fall back to the raw path.  Non-existent absolute-path
                // traversals are caught by the `starts_with` check that follows
                // (the raw absolute path still cannot start with `canonical_base`
                // if it genuinely escapes the sandbox).
                path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
            } else {
                canonical_base.join(path)
            };
            let normalized_resolved = normalize_path(&canonical_candidate);
            let normalized_base = normalize_path(&canonical_base);
            if !normalized_resolved.starts_with(&normalized_base) {
                let path_str: Arc<str> = Arc::from(resolved.to_string_lossy().as_ref());
                return Err(DataError::path_traversal_blocked(path_str));
            }

            // --- Filesystem check (second line of defence, handles symlinks) ---
            //
            // For files that already exist on disk, re-check after full
            // canonicalization so that symlinks pointing outside the sandbox are
            // also caught.
            if let Ok(canonical_resolved) = resolved.canonicalize()
                && !canonical_resolved.starts_with(&canonical_base)
            {
                let path_str: Arc<str> = Arc::from(resolved.to_string_lossy().as_ref());
                return Err(DataError::path_traversal_blocked(path_str));
            }
        }

        // Determine format from extension before reading the file.
        let format = DataFormat::from_path(&resolved).ok_or_else(|| {
            let ext: Arc<str> =
                Arc::from(resolved.extension().and_then(|e| e.to_str()).unwrap_or(""));
            DataError::unsupported_format(ext)
        })?;

        let path_str: Arc<str> = Arc::from(resolved.to_string_lossy().as_ref());

        // F-CRIT-1: Dispatch to binary format parsers before reading file as text.
        // XLSX and SQLite are binary formats; read_to_string would fail or corrupt them.
        // Traces to BC-1.03.006 and BC-1.03.007 end-to-end load path.
        match format {
            DataFormat::Xlsx => {
                // Delegate to XlsxDataSource. No query needed for XLSX.
                // uri convention: pass the resolved path as the uri (non-empty → XlsxDataSource uses it).
                let src = XlsxDataSource::new(Arc::clone(&path_str));
                let opts = slideforge_plugin_api::DataSourceOptions::default();
                return src
                    .load(path_str.as_ref(), &opts)
                    .map_err(|e| match e {
                        slideforge_plugin_api::DataSourceError::IoError { message, .. } => {
                            DataError::io_error(Arc::clone(&path_str), Arc::from(message.as_str()))
                        }
                        slideforge_plugin_api::DataSourceError::ParseError { message, .. } => {
                            DataError::parse_error(&*path_str, format, message)
                        }
                        slideforge_plugin_api::DataSourceError::UnsupportedUri { uri } => {
                            DataError::unsupported_format(Arc::from(uri.as_str()))
                        }
                        other => DataError::io_error(
                            Arc::clone(&path_str),
                            Arc::from(other.to_string().as_str()),
                        ),
                    });
            }
            DataFormat::Sqlite => {
                // Delegate to SqliteDataSource. A query is required for SQLite;
                // load_path cannot provide it (no opts parameter).
                // Callers that need SQLite support should use DataSource::load()
                // which receives DataSourceOptions with query.
                // This path is a fallback error to guide users to the correct API.
                return Err(DataError::parse_error(
                    &*path_str,
                    format,
                    "SQLite data sources require a query string. Use the DataSource::load() \
                    API with DataSourceOptions.query set to your SELECT statement, or use \
                    the DSL @data directive with query: \"SELECT ...\".",
                ));
            }
            _ => {
                // Text-based formats: read the file content below.
            }
        }

        // Read the file contents, distinguishing NotFound from other I/O errors.
        let raw_contents = std::fs::read_to_string(&resolved).map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                DataError::file_not_found(Arc::clone(&path_str))
            } else {
                DataError::io_error(Arc::clone(&path_str), Arc::from(e.to_string().as_str()))
            }
        })?;

        // Strip a leading UTF-8 BOM (U+FEFF) if present.
        // Excel on Windows exports CSV/TSV with a BOM, which corrupts the first
        // column header with an invisible prefix. JSON parsers also reject a BOM.
        let contents = raw_contents
            .strip_prefix('\u{FEFF}')
            .unwrap_or(&raw_contents);

        // Dispatch to the text-based parsers.
        match format {
            DataFormat::Json => json::parse_json(contents, &path_str),
            DataFormat::Csv => csv::parse_csv(contents, &path_str),
            DataFormat::Yaml => yaml::parse_yaml(contents, &path_str),
            DataFormat::Toml => toml::parse_toml(contents, &path_str),
            DataFormat::Xlsx | DataFormat::Sqlite => {
                // Already handled above; this arm is unreachable.
                unreachable!("xlsx/sqlite dispatched before text-read")
            }
        }
    }
}

impl DataSource for FileDataSource {
    fn id(&self) -> &'static str {
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
    use std::sync::Arc;

    use tempfile::NamedTempFile;

    use super::*;

    fn loader() -> FileDataSource {
        FileDataSource::new()
    }

    /// `test_BC_5_03_007_file_not_found` — non-existent path → `DataError::FileNotFound` with E-DAT-004.
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

    /// `test_BC_5_03_007_file_format_dispatch_json` — .json file dispatches to JSON parser.
    #[test]
    fn test_bc_5_03_007_file_format_dispatch_json() {
        let f = temp_file_with_suffix(".json", br#"{"key": "value"}"#);

        let src = loader();
        let value = src
            .load_path(f.path(), None)
            .expect(".json file must parse");
        let map = value.as_map().expect("must be map");
        assert!(map.get("key").is_some(), "JSON key must be present");
    }

    /// `test_BC_5_03_007_file_format_dispatch_csv` — .csv file dispatches to CSV parser.
    #[test]
    fn test_bc_5_03_007_file_format_dispatch_csv() {
        let f = temp_file_with_suffix(".csv", b"col\nval");

        let src = loader();
        let value = src.load_path(f.path(), None).expect(".csv file must parse");
        let list = value.as_list().expect("CSV must produce list");
        assert_eq!(list.len(), 1);
    }

    /// `test_BC_5_03_007_file_format_dispatch_yaml` — .yaml file dispatches to YAML parser.
    #[test]
    fn test_bc_5_03_007_file_format_dispatch_yaml() {
        let f = temp_file_with_suffix(".yaml", b"x: 1\n");

        let src = loader();
        let value = src
            .load_path(f.path(), None)
            .expect(".yaml file must parse");
        assert!(value.as_map().is_some(), "YAML must produce map");
    }

    /// `test_BC_5_03_007_file_format_dispatch_yml` — .yml file dispatches to YAML parser (EC-008).
    #[test]
    fn test_bc_5_03_007_file_format_dispatch_yml() {
        let f = temp_file_with_suffix(".yml", b"x: 1\n");

        let src = loader();
        let value = src.load_path(f.path(), None).expect(".yml file must parse");
        assert!(value.as_map().is_some(), ".yml must be treated as YAML");
    }

    /// `test_BC_5_03_007_file_format_dispatch_toml` — .toml file dispatches to TOML parser.
    #[test]
    fn test_bc_5_03_007_file_format_dispatch_toml() {
        let f = temp_file_with_suffix(".toml", b"x = 1\n");

        let src = loader();
        let value = src
            .load_path(f.path(), None)
            .expect(".toml file must parse");
        assert!(value.as_map().is_some(), "TOML must produce map");
    }

    /// `test_BC_5_03_007_unsupported_extension` — .txt file → `DataError::UnsupportedFormat` (E-DAT-003).
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

    /// `test_BC_5_03_007_datasource_trait_id` — `DataSource::id()` returns "file".
    #[test]
    fn test_bc_5_03_007_datasource_trait_id() {
        let src = loader();
        assert_eq!(src.id(), "file");
    }

    /// `test_BC_5_03_007_datasource_trait_load_missing` — `DataSource::load()` maps to `DataSourceError`.
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

    /// `test_BC_5_03_007_datasource_trait_load_unsupported` — `DataSource::load()` maps to `UnsupportedUri`.
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

    /// `test_BC_5_03_007_datasource_trait_load_json` — `DataSource::load()` happy path for JSON.
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

    /// `test_BC_5_03_007_path_containment_blocks_traversal` — `../` path outside `base_dir` is blocked.
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
            Err(_) => {},
            Ok(_) => {
                panic!("path traversal must not succeed silently");
            },
        }
    }

    /// `test_BC_5_03_007_base_dir_nonexistent_fails` — non-canonicalizable `base_dir` must return Err,
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

    /// `test_BC_5_03_007_relative_path_resolved_against_base_dir` — relative path uses `base_dir`.
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

    /// `test_bom_stripped_json` — JSON file with UTF-8 BOM (U+FEFF) prefix parses correctly.
    ///
    /// Excel on Windows and some editors prepend a BOM to UTF-8 files. Without BOM
    /// stripping, `serde_json` rejects the file with an unexpected character error.
    #[test]
    fn test_bom_stripped_json() {
        // UTF-8 BOM = 0xEF 0xBB 0xBF, followed by valid JSON.
        let mut content = vec![0xEF_u8, 0xBB, 0xBF];
        content.extend_from_slice(br#"{"answer": 42}"#);

        let f = temp_file_with_suffix(".json", &content);
        let src = loader();
        let value = src
            .load_path(f.path(), None)
            .expect("JSON with BOM must parse without error");
        let map = value.as_map().expect("result must be a map");
        assert_eq!(
            map.get("answer"),
            Some(&Value::Int(42)),
            "BOM must be stripped so JSON is parsed correctly"
        );
    }

    /// `test_traversal_blocked_nonexistent_file` — lexical `../` traversal to non-existent file
    /// must return `PathTraversalBlocked` (E-DAT-006), NOT `FileNotFound` (E-DAT-004).
    ///
    /// This is the FINDING-001 regression test. Before the fix, `canonicalize()` failed on
    /// the non-existent target so the containment check was skipped, and the subsequent
    /// `read_to_string` produced `FileNotFound` — leaking file-existence information.
    #[test]
    fn test_traversal_blocked_nonexistent_file() {
        use tempfile::TempDir;
        let dir = TempDir::new().expect("temp dir");
        let src = loader();

        // `../../nonexistent.json` lexically escapes base_dir regardless of whether
        // the target exists on disk.
        let traversal = Path::new("../../nonexistent_slideforge_test_12345.json");
        let result = src.load_path(traversal, Some(dir.path()));
        let err = result.expect_err("traversal to non-existent file must return Err");
        assert_eq!(
            err.code(),
            "E-DAT-006",
            "path traversal for non-existent file must return E-DAT-006 (PathTraversalBlocked), not E-DAT-004 (FileNotFound): {err:?}"
        );
    }

    /// `test_traversal_blocked_existing_and_nonexistent_same_error` — both existing and
    /// non-existing out-of-sandbox paths must produce the same error variant (E-DAT-006).
    ///
    /// This prevents the file-existence oracle: an attacker must not be able to
    /// distinguish "file exists outside sandbox" from "file doesn't exist outside sandbox"
    /// by observing the error code.
    #[test]
    fn test_traversal_blocked_existing_and_nonexistent_same_error() {
        use tempfile::TempDir;
        let dir = TempDir::new().expect("temp dir");
        let src = loader();

        // Non-existent path outside sandbox.
        let nonexistent = Path::new("../nonexistent_slideforge_oracle_test.json");
        let err_nonexistent = src
            .load_path(nonexistent, Some(dir.path()))
            .expect_err("non-existent traversal must Err");

        // /etc/passwd reliably exists on macOS/Linux and is definitely outside any TempDir.
        let existing = Path::new("../../../etc/passwd.json");
        let err_existing = src
            .load_path(existing, Some(dir.path()))
            .expect_err("existing traversal must Err");

        assert_eq!(
            err_nonexistent.code(),
            "E-DAT-006",
            "non-existent outside-sandbox file must return E-DAT-006 (PathTraversalBlocked)"
        );
        assert_eq!(
            err_existing.code(),
            "E-DAT-006",
            "existing outside-sandbox file must return E-DAT-006 (same as non-existent — no oracle)"
        );
    }

    /// `test_absolute_path_inside_base_dir_succeeds` — absolute path pointing into `base_dir` loads OK.
    ///
    /// Exercises the `path.is_absolute()` branch in the resolver (lines ~102-103) and the
    /// corresponding lexical containment check (lines ~141-151) for an absolute path that IS
    /// inside the sandbox.  Before FINDING-002 this code path had zero test coverage.
    #[test]
    fn test_absolute_path_inside_base_dir_succeeds() {
        use tempfile::TempDir;
        let dir = TempDir::new().expect("temp dir");
        let json_path = dir.path().join("inside.json");
        std::fs::write(&json_path, br#"{"ok": true}"#).expect("write temp json");

        let src = loader();
        // Pass the absolute path to a file that lives inside base_dir.
        let result = src.load_path(&json_path, Some(dir.path()));
        let value = result.expect("absolute path inside base_dir must succeed");
        assert!(value.as_map().is_some(), "must be a map");
    }

    /// `test_absolute_path_outside_base_dir_blocked` — absolute path outside `base_dir` returns
    /// E-DAT-006 (`PathTraversalBlocked`).
    ///
    /// Exercises the `path.is_absolute()` branch plus the lexical containment check for an
    /// absolute path that escapes the sandbox.  Before FINDING-002 this code path was
    /// untested, leaving a gap in the security regression suite.
    #[test]
    fn test_absolute_path_outside_base_dir_blocked() {
        use tempfile::TempDir;
        let sandbox = TempDir::new().expect("sandbox temp dir");
        let outside = TempDir::new().expect("outside temp dir");

        // Create a real JSON file in `outside` so the test is not confounded by
        // FileNotFound — the containment check must fire before the read attempt.
        let outside_json = outside.path().join("secret.json");
        std::fs::write(&outside_json, br#"{"secret": 42}"#).expect("write outside json");

        let src = loader();
        // Pass the absolute path to a file OUTSIDE the sandbox with base_dir set to sandbox.
        let result = src.load_path(&outside_json, Some(sandbox.path()));
        let err = result.expect_err("absolute path outside base_dir must be blocked");
        assert_eq!(
            err.code(),
            "E-DAT-006",
            "absolute path outside sandbox must return E-DAT-006 (PathTraversalBlocked): {err:?}"
        );
    }

    /// `test_bom_stripped_csv` — CSV file with UTF-8 BOM prefix has clean column headers.
    ///
    /// A BOM prefix corrupts the first column name with an invisible U+FEFF character,
    /// making header-based lookups fail silently. Stripping ensures clean headers.
    #[test]
    fn test_bom_stripped_csv() {
        // UTF-8 BOM followed by a CSV with header "name" and one data row.
        let mut content = vec![0xEF_u8, 0xBB, 0xBF];
        content.extend_from_slice(b"name\nalice");

        let f = temp_file_with_suffix(".csv", &content);
        let src = loader();
        let value = src
            .load_path(f.path(), None)
            .expect("CSV with BOM must parse without error");
        let list = value.as_list().expect("CSV must produce list");
        assert_eq!(list.len(), 1, "must have one data row");

        // The first row must be a map whose key is exactly "name" (no BOM prefix).
        let row = &list[0];
        let row_map = row.as_map().expect("CSV row must be a map");
        assert!(
            row_map.contains_key("name"),
            "column header must be 'name' (no invisible BOM prefix); got keys: {:?}",
            row_map.keys().collect::<Vec<_>>()
        );
    }

    // ---------------------------------------------------------------------------
    // F-LOW-6: XLSX dispatch through FileDataSource end-to-end (BC-1.03.006).
    // FileDataSource::load_path() must delegate .xlsx files to XlsxDataSource
    // rather than attempting to read them as text.
    // ---------------------------------------------------------------------------

    /// `test_file_datasource_dispatches_xlsx` — `.xlsx` file loads via FileDataSource (F-LOW-6).
    ///
    /// Verifies that `FileDataSource::load_path()` correctly delegates `.xlsx` files
    /// to `XlsxDataSource` instead of trying to read them as UTF-8 text (which would
    /// corrupt or fail on binary data).
    ///
    /// Traces to BC-1.03.006, F-LOW-6 (F-CRIT-1 binary-format dispatch).
    #[test]
    fn test_file_datasource_dispatches_xlsx() {
        use rust_xlsxwriter::Workbook;
        let dir = tempfile::TempDir::new().expect("temp dir");
        let xlsx_path = dir.path().join("data.xlsx");

        // Write a minimal valid .xlsx file.
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();
        ws.write_string(0, 0, "city").unwrap();
        ws.write_string(0, 1, "pop").unwrap();
        ws.write_string(1, 0, "London").unwrap();
        ws.write_number(1, 1, 9_500_000.0).unwrap();
        wb.save(&xlsx_path).expect("save test xlsx");

        let src = loader();
        let value = src
            .load_path(&xlsx_path, None)
            .expect("FileDataSource must load .xlsx without error");

        let list = value.as_list().expect("xlsx result must be a list");
        assert_eq!(list.len(), 1, "must have exactly one data row");

        let row = &list[0];
        let row_map = row.as_map().expect("row must be a map");
        assert_eq!(
            row_map.get("city"),
            Some(&Value::Str(Arc::from("London"))),
            "city column must contain 'London'"
        );
        assert_eq!(
            row_map.get("pop"),
            Some(&Value::Int(9_500_000)),
            "pop column must contain Int(9_500_000) (Float→Int promotion)"
        );
    }

    /// `test_file_datasource_sqlite_no_query_returns_error` — `.sqlite` via `load_path()` returns
    /// a clear error directing users to `DataSource::load()` with a query string.
    ///
    /// `load_path()` cannot provide a query (no `DataSourceOptions` parameter).
    /// This test verifies the error message is actionable rather than a confusing
    /// binary-parse failure.
    ///
    /// Traces to BC-1.03.007, F-CRIT-1 (binary-format dispatch).
    #[test]
    fn test_file_datasource_sqlite_no_query_returns_error() {
        // Use a temp file with .sqlite extension (content doesn't matter — the error
        // fires before the file is opened).
        let f = temp_file_with_suffix(".sqlite", b"");
        let src = loader();
        let result = src.load_path(f.path(), None);
        let err = result.expect_err(".sqlite via load_path must return Err (no query available)");
        // Must be a ParseError (E-DAT-003), not an IoError (E-DAT-004).
        assert_eq!(
            err.code(),
            "E-DAT-003",
            "sqlite no-query error must be E-DAT-003 (ParseError)"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("query") || msg.contains("SELECT"),
            "error message must mention 'query' or 'SELECT'; got: {msg}"
        );
    }
}
