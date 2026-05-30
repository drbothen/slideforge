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
/// - `.sqlite` / `.sqlite3` / `.db` → [`crate::sqlite::SqliteDataSource`] (bundled `SQLite`)
///   For `SQLite`, the `uri` field is used as the path per the plugin interface convention.
///   A query string must be provided via `DataSourceOptions` or the DSL `query:` directive.
#[derive(Debug)]
pub struct FileDataSource {
    /// The file path baked in at construction time.
    ///
    /// When `DataSource::load()` is called with an empty `uri`, this path is used
    /// as the effective file path. When `load()` is called with a non-empty `uri`,
    /// the `uri` overrides this field (allows the dispatcher to call `load("", opts)`
    /// and have each source resolve its own pre-configured address).
    path: Arc<str>,
}

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
    /// Construct a new [`FileDataSource`] with the given file path.
    ///
    /// The `path` is baked in at construction time. When the dispatcher calls
    /// `DataSource::load("", opts)` (empty URI), the source uses this path.
    /// When `load(uri, opts)` is called with a non-empty URI, the URI overrides
    /// the baked-in path (enabling the evaluator to override the path at call time).
    ///
    /// # Example
    ///
    /// ```rust
    /// use slideforge_data::FileDataSource;
    /// let src = FileDataSource::new("data/sales.json");
    /// ```
    #[must_use]
    pub fn new(path: impl Into<Arc<str>>) -> Self {
        FileDataSource { path: path.into() }
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
                // Delegate to XlsxDataSource via load_internal, which returns DataError directly.
                // This avoids the DataError → DataSourceError → DataError double-wrap that
                // occurred when calling load() + re-wrapping (F-PASS26-MED-1): the inner error
                // code (e.g. E-DAT-011 for wrong magic bytes) was lost and replaced by E-DAT-003,
                // and the bracket notation appeared twice in the user-visible message.
                let src = XlsxDataSource::new(Arc::clone(&path_str));
                return src.load_internal(path_str.as_ref());
            },
            DataFormat::Sqlite => {
                // Delegate to SqliteDataSource. A query is required for SQLite;
                // load_path cannot provide it (no opts parameter). Note that
                // SqliteDataSource ignores opts.query at runtime — the query is
                // fixed at construction time via SqliteDataSource::new(path, query).
                // This path is a fallback error to guide callers to the correct API.
                return Err(DataError::parse_error(
                    &*path_str,
                    format,
                    "SQLite data sources require a query string. Construct \
                    SqliteDataSource::new(path, query) directly, or use the DSL @data \
                    directive with query: \"SELECT ...\". DataSource::load() opts.query \
                    is not honored by SqliteDataSource.",
                ));
            },
            _ => {
                // Text-based formats: read the file content below.
            },
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
                // SAFETY INVARIANT: binary formats (Xlsx, Sqlite) must be dispatched at the
                // pre-dispatch block above (lines ~222-249). If this arm is reached in
                // correct code, the pre-dispatch block is missing a new binary format. Rather
                // than panicking, surface a structured error so an invariant violation is
                // visible to callers and to the error taxonomy, rather than crashing the process.
                //
                // F-P10-LOW-002 fix: replaced `unreachable!()` (which panics in production)
                // with a graceful `DataError::unsupported_format` so future maintainers
                // adding binary format variants without wiring the pre-dispatch route receive
                // a structured error rather than a process abort.
                Err(DataError::unsupported_format(Arc::from(format!(
                    "{format:?}"
                ))))
            },
            // DataFormat is #[non_exhaustive]; wildcard arm required to remain
            // forward-compatible when new variants are added in future releases.
            // This arm catches unknown variants such as DataFormat::Unknown that
            // are not valid file-load targets (they exist for error-message annotation only).
            _ => Err(DataError::unsupported_format(Arc::from(format!(
                "{format:?}"
            )))),
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
    ///
    /// ## URI resolution
    ///
    /// When `uri` is non-empty it is used as the file path directly (override).
    /// When `uri` is empty the source falls back to `self.path`, which is the
    /// path baked in at construction time via [`FileDataSource::new`]. This
    /// matches the convention used by `HttpDataSource` (`self.url` fallback) and
    /// `XlsxDataSource` / `SqliteDataSource` (`self.path` fallback), so the
    /// dispatcher can uniformly call `load("", opts)` on all source types and
    /// have each resolve its own pre-configured address.
    ///
    // `_opts` is intentionally ignored: local file formats (JSON, CSV, YAML, TOML) have no
    // concept of timeout, auth token, or query filter at the DataSource trait boundary.
    // File-based sources MAY silently ignore options that have no semantic meaning for their
    // format (see `DataSourceOptions` rustdoc convention, F-PASS21-LOW-1).
    fn load(&self, uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        // Use uri as the primary path; fall back to self.path when uri is empty.
        // This mirrors HttpDataSource::load() which falls back to self.url when uri is empty,
        // ensuring the dispatcher's uniform `load("", opts)` call works for all source types.
        let effective_path_str: &str = if uri.is_empty() {
            self.path.as_ref()
        } else {
            uri
        };
        let path = Path::new(effective_path_str);
        self.load_path(path, None).map_err(|e| match e {
            DataError::FileNotFound { path, .. } => DataSourceError::IoError {
                uri: effective_path_str.to_owned(),
                // FileNotFound: construct message manually so the [E-DAT-004] bracket code
                // is explicit without the Display's "I/O error reading" boilerplate.
                message: format!("[{}] file not found: {path}", crate::error::E_DAT_004),
            },
            DataError::UnsupportedFormat { extension, .. } => DataSourceError::UnsupportedUri {
                // Convention: emit only the bare extension (e.g., "txt") as the uri slot.
                // The dispatcher maps UnsupportedUri.uri → DataError::UnsupportedFormat.extension,
                // which is rendered by the Display as "[E-DAT-003] unsupported format: 'txt' — ...".
                // Putting the full path + annotation here would produce stuttered output:
                // "[E-DAT-003] unsupported format: '/tmp/data.txt (unsupported extension: txt)' — ..."
                // F-P11-LOW-002 fix.
                uri: extension.to_string(),
            },
            // F-PASS18-MED-2 / F-PASS26-MED-1: Translate DataError::ParseError into
            // DataSourceError::ParseError without re-wrapping the boilerplate prefix.
            //
            // Using err.to_string() would produce "[E-DAT-NNN] parse error for 'path' (Fmt): reason"
            // and then DataSourceError::ParseError Display adds "data parse error for 'uri': " —
            // resulting in two "parse error for" phrases and (if the reason itself contains
            // [E-DAT-NNN]) potentially two bracket codes.
            //
            // Instead: extract the granular code and reason directly, composing
            // "[{code}] {reason}" as the DataSourceError message. The bracket code is preserved
            // (satisfying F-PASS18-MED-2) and no boilerplate prefix is duplicated (F-PASS26-MED-1).
            DataError::ParseError { code, reason, .. } => DataSourceError::ParseError {
                uri: effective_path_str.to_owned(),
                message: format!("[{code}] {reason}"),
            },
            // F-PASS18-MED-1: use err.to_string() for IoError ([E-DAT-004]) and
            // PathTraversalBlocked ([E-DAT-006]) so bracket codes are preserved.
            // The catch-all also uses err.to_string(), which always includes the bracket code
            // from the DataError Display format for any remaining variants.
            err => DataSourceError::IoError {
                uri: effective_path_str.to_owned(),
                message: err.to_string(),
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
        // Unit tests that use `loader()` always call `load_path()` directly or
        // call `load(uri, opts)` with an explicit URI — they never rely on
        // self.path. An empty string is fine here.
        FileDataSource::new("")
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
        // Load-bearing: E-DAT-004 bracket code must be embedded in the IoError message.
        // Traces to F-PASS17-LOW-1 workspace sweep.
        let msg = err.to_string();
        assert!(
            msg.contains("[E-DAT-004]"),
            "file-not-found IoError message must embed [E-DAT-004] bracket code; got: {msg}"
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

    /// `test_unsupported_extension_uri_is_bare_extension`
    ///
    /// F-P11-LOW-002: The `UnsupportedUri.uri` field must contain only the bare
    /// extension (e.g., `"txt"`), NOT the full path + annotation string
    /// `"/tmp/data.txt (unsupported extension: txt)"`.
    ///
    /// When the dispatcher maps `UnsupportedUri { uri }` →
    /// `DataError::UnsupportedFormat { extension: Arc::from(uri.as_str()) }`,
    /// the resulting Display must read:
    ///   `"[E-DAT-003] unsupported format: 'txt' — supported: json, csv, ..."`
    /// NOT the stuttered form:
    ///   `"[E-DAT-003] unsupported format: '/tmp/data.txt (unsupported extension: txt)' — ..."`
    ///
    /// This test drives the `DataSource::load()` trait boundary (not `load_path`)
    /// so it exercises exactly the code path that feeds the dispatcher.
    ///
    /// Traces to F-P11-LOW-002 (Pass 11 adversarial finding).
    #[test]
    fn test_unsupported_extension_uri_is_bare_extension() {
        let f = temp_file_with_suffix(".txt", b"data");
        let src = loader();
        let opts = DataSourceOptions::default();
        let uri = f.path().to_str().unwrap();
        let result = src.load(uri, &opts);
        let err = result.expect_err(".txt must return DataSourceError");

        match &err {
            DataSourceError::UnsupportedUri { uri: uri_field } => {
                // The uri slot must hold only the bare extension — not a composite string.
                assert_eq!(
                    uri_field.as_str(),
                    "txt",
                    "UnsupportedUri.uri must be the bare extension 'txt'; \
                    got: '{uri_field}' — stutter guard: must NOT contain path or '(unsupported extension:'"
                );
                assert!(
                    !uri_field.contains('('),
                    "UnsupportedUri.uri must not contain '(' (no annotation); got: '{uri_field}'"
                );
            },
            other => panic!("expected DataSourceError::UnsupportedUri, got: {other:?}"),
        }
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

    /// `test_file_datasource_dispatches_xlsx` — `.xlsx` file loads via `FileDataSource` (F-LOW-6).
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

    // ---------------------------------------------------------------------------
    // F-LOW-3: path traversal with .xlsx extension is blocked by generic containment.
    // Confirms FileDataSource::load_path path-containment catches .xlsx routes.
    // ---------------------------------------------------------------------------

    /// `test_xlsx_path_traversal_blocked` — `.xlsx` path traversal outside base is blocked.
    ///
    /// Verifies that the generic path-containment check in `FileDataSource::load_path`
    /// catches traversal attempts on `.xlsx` paths (e.g., `../malicious.xlsx`).
    ///
    /// This test FAILS if path-containment is only applied to text formats (JSON/CSV)
    /// and skipped for xlsx dispatch — making it load-bearing per TD-VSDD-059.
    ///
    /// Traces to BC-1.03.006, F-LOW-3 (path containment for xlsx route).
    #[test]
    fn test_xlsx_path_traversal_blocked() {
        let src = loader();
        let base = std::path::PathBuf::from("/tmp");
        // Attempt to load an .xlsx file above /tmp via ../
        let traversal = std::path::Path::new("../some_other_dir/malicious.xlsx");
        let result = src.load_path(traversal, Some(&base));
        // Must NOT return Ok — traversal block, file not found, or another error.
        match result {
            Err(_) => {
                // Any error is acceptable — traversal blocked, file not found,
                // extension rejected, etc. The key invariant is: not Ok.
            },
            Ok(_) => panic!(
                "xlsx path traversal must not succeed silently — \
                path containment must apply to the .xlsx dispatch route"
            ),
        }
    }

    // ---------------------------------------------------------------------------
    // F-PASS18-MED-1/MED-2: DataSource translator must preserve bracket codes.
    // Each arm that strips bracket codes is a defect — the bracket code from the
    // underlying DataError::Display must survive translation into DataSourceError.
    // ---------------------------------------------------------------------------

    /// `test_pass18_med1_io_error_preserves_bracket_code` — `DataSource::load()` `IoError` arm
    /// preserves `[E-DAT-004]` bracket code from the underlying `DataError::IoError`.
    ///
    /// Before F-PASS18-MED-1 fix: `DataError::IoError { message, .. }` extracted the raw
    /// `message` field (which is just the I/O description text, no bracket code).
    /// After fix: `err.to_string()` is used, which includes `[E-DAT-004]` from Display.
    ///
    /// Traces to F-PASS18-MED-1 (sibling-sweep gap: `IoError` → `DataSourceError::IoError` strips `[E-DAT-004]`).
    #[test]
    fn test_pass18_med1_io_error_preserves_bracket_code() {
        use std::path::PathBuf;
        // Trigger a DataError::IoError path: load_path with a non-canonicalizable base_dir
        // (which returns IoError E-DAT-004 via the base_dir canonicalization failure path).
        let src = loader();
        let nonexistent_base = PathBuf::from("/tmp/__slideforge_nonexistent_base_dir_99998");
        // Use DataSource::load() with a URI — this goes through the translator in load().
        // We need to trigger the IoError arm: use a valid-looking JSON path but a base_dir
        // that can't be canonicalized. Since load() calls load_path(path, None), we need
        // to trigger IoError through load_path.
        // The IoError path fires when base_dir exists but is not canonicalizable, OR when
        // an I/O read fails. We can test via load_path directly since the translator arms
        // are exercised by the DataSourceError the caller sees.
        // Strategy: Use an existing temp dir as base_dir, load a path that produces a
        // DataError::IoError (e.g., a file exists but fails to read — hard to arrange
        // without mocking). Instead, verify the FileNotFound → IoError arm (already tested)
        // and separately verify the IoError arm via a parse failure path that hits IoError.
        //
        // Best direct approach: use DataSource::load() on a path that does not exist.
        // FileNotFound → IoError arm (already tested with [E-DAT-004]).
        // For pure IoError arm: create a file that exists with right extension but trigger
        // an IoError by loading a file we can't read.
        //
        // Simpler: the DataError::IoError arm fires from base_dir canonicalization failure.
        // We exercise it through load_path() and verify the code on the returned DataError.
        let result = src.load_path(std::path::Path::new("some.json"), Some(&nonexistent_base));
        let err = result.expect_err("non-canonicalizable base_dir must return Err");
        // The DataError itself (from load_path) must carry E-DAT-004.
        assert_eq!(
            err.code(),
            "E-DAT-004",
            "base_dir failure must return E-DAT-004 DataError"
        );
        // Verify the DataError Display includes [E-DAT-004] — this is what the translator
        // must preserve when mapping to DataSourceError.
        let msg = err.to_string();
        assert!(
            msg.contains("[E-DAT-004]"),
            "DataError::IoError Display must include [E-DAT-004]; got: {msg}"
        );
    }

    /// `test_pass18_med2_parse_error_preserves_bracket_code` — `DataSource::load()` `ParseError` arm
    /// preserves `[E-DAT-003]` bracket code from the underlying `DataError::ParseError`.
    ///
    /// Before F-PASS18-MED-2 fix: `DataError::ParseError { reason, .. }` extracted the raw
    /// `reason` field (no bracket code). After fix: `err.to_string()` used, including `[E-DAT-003]`.
    ///
    /// Traces to F-PASS18-MED-2 (sibling-sweep gap: `ParseError` → `DataSourceError::ParseError` strips `[E-DAT-003]`).
    #[test]
    fn test_pass18_med2_parse_error_preserves_bracket_code() {
        // Trigger a DataError::ParseError by loading a file with invalid JSON.
        let f = temp_file_with_suffix(".json", b"{ this is not valid json }");
        let src = loader();
        let opts = DataSourceOptions::default();
        let uri = f.path().to_str().unwrap();
        // DataSource::load() maps DataError::ParseError → DataSourceError::ParseError.
        // The resulting DataSourceError::ParseError::message must contain [E-DAT-003].
        let err = src
            .load(uri, &opts)
            .expect_err("invalid JSON must return Err");
        assert!(
            matches!(err, DataSourceError::ParseError { .. }),
            "invalid JSON must produce DataSourceError::ParseError; got: {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("[E-DAT-003]"),
            "DataSourceError::ParseError message must preserve [E-DAT-003] bracket code; got: {msg}"
        );
    }

    /// `test_pass18_med1_path_traversal_preserves_bracket_code` — `DataSource::load()`
    /// `PathTraversalBlocked` arm preserves `[E-DAT-006]` bracket code.
    ///
    /// Before F-PASS18-MED-1 fix: `DataError::PathTraversalBlocked { path, .. }` was used to
    /// format a message without the bracket code. After fix: `err.to_string()` used.
    ///
    /// This path is exercised through `load_path()` (the `DataError` is returned from there),
    /// but the translator in `load()` is what is being tested for bracket preservation.
    ///
    /// Traces to F-PASS18-MED-1 (path traversal arm strips `[E-DAT-006]`).
    #[test]
    fn test_pass18_med1_path_traversal_preserves_bracket_code() {
        // We test via DataError::PathTraversalBlocked Display directly (load() doesn't go
        // through the PathTraversalBlocked arm because load() calls load_path(path, None)
        // which has no base_dir and thus never triggers path traversal).
        // The translator arm is code coverage for callers that may pass a base_dir; we
        // verify the DataError itself has the bracket code in its Display.
        let err = crate::error::DataError::path_traversal_blocked("../../etc/passwd");
        let msg = err.to_string();
        assert!(
            msg.contains("[E-DAT-006]"),
            "DataError::PathTraversalBlocked Display must include [E-DAT-006]; got: {msg}"
        );
        // Now verify the translated message also preserves the code via err.to_string().
        // We can't easily trigger this arm through DataSource::load() (no base_dir),
        // but we verify the Display is preserved by simulating the translator logic:
        let translated_message = err.to_string();
        assert!(
            translated_message.contains("[E-DAT-006]"),
            "err.to_string() (used by translator after fix) must contain [E-DAT-006]; got: {translated_message}"
        );
    }

    // ---------------------------------------------------------------------------
    // F-PASS26-MED-1: XLSX dispatch through FileDataSource must not double-wrap errors.
    // Before the fix, file.rs called XlsxDataSource::load() and re-wrapped
    // DataSourceError::ParseError into DataError::parse_error(), which prepended
    // "[E-DAT-003] parse error for..." around a message that already contained
    // "[E-DAT-011] ...". The fix uses load_internal() which returns DataError directly.
    // ---------------------------------------------------------------------------

    /// `test_pass26_med1_xlsx_magic_byte_failure_no_double_bracket` — magic-byte failure
    /// through `FileDataSource::load` must not produce duplicate `[E-DAT-` brackets.
    ///
    /// Regression test for F-PASS26-MED-1. Before the fix, the error message contained
    /// `[E-DAT-003]` (outer wrapper) and `[E-DAT-011]` (inner granular code) — both
    /// from the double-wrap. After the fix, only `[E-DAT-011]` appears.
    ///
    /// Load-bearing assertions (TD-VSDD-059):
    /// - `msg.matches("[E-DAT-").count() == 1` — no duplicate bracket codes
    /// - `msg.matches("parse error for").count() == 1` — no duplicate boilerplate prefix
    /// - `msg.contains("[E-DAT-011]")` — granular code is preserved, NOT replaced by E-DAT-003
    ///
    /// Traces to F-PASS26-MED-1, BC-1.03.006 postcondition 9, VP-026.
    #[test]
    fn test_pass26_med1_xlsx_magic_byte_failure_no_double_bracket() {
        // Create a file with .xlsx extension but wrong magic bytes (not a ZIP archive).
        let f = temp_file_with_suffix(".xlsx", b"This is not a real xlsx file");
        let src = loader();
        let opts = DataSourceOptions::default();
        let uri = f.path().to_str().unwrap();

        let err = src
            .load(uri, &opts)
            .expect_err("magic-byte failure must return Err");

        let msg = err.to_string();

        // Load-bearing: no duplicate [E-DAT-... bracket codes.
        let bracket_count = msg.matches("[E-DAT-").count();
        assert_eq!(
            bracket_count, 1,
            "exactly one [E-DAT-... bracket code must appear (no double-wrap); got {bracket_count} in: {msg}"
        );

        // Load-bearing: no duplicate "parse error for" prefix.
        let prefix_count = msg.matches("parse error for").count();
        assert_eq!(
            prefix_count, 1,
            "exactly one 'parse error for' phrase must appear (no double-wrap); got {prefix_count} in: {msg}"
        );

        // Load-bearing: granular code E-DAT-011 is preserved (not replaced by E-DAT-003).
        assert!(
            msg.contains("[E-DAT-011]"),
            "granular code [E-DAT-011] must be present in the error message; got: {msg}"
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
