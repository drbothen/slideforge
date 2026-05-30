//! Format detection for slideforge data sources.
//!
//! [`DataFormat`] is an enum of all supported file formats. The
//! [`DataFormat::from_extension`] method maps a file extension (with or without
//! leading dot) to the appropriate format variant.

use std::path::Path;

/// The set of data formats that the built-in file loader can handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataFormat {
    /// JSON (`.json`)
    Json,
    /// CSV (`.csv`)
    Csv,
    /// YAML (`.yaml` or `.yml`)
    Yaml,
    /// TOML (`.toml`)
    Toml,
    /// Excel XLSX (`.xlsx`) — implemented in STORY-020 (see [`crate::XlsxDataSource`]).
    Xlsx,
    /// `SQLite` (`.sqlite`, `.db`, `.sqlite3`) — implemented in STORY-020 (see [`crate::SqliteDataSource`]).
    Sqlite,
}

impl DataFormat {
    /// Determine the [`DataFormat`] from a file path's extension.
    ///
    /// Returns `None` if the extension is absent or not recognized.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use slideforge_data::DataFormat;
    /// use std::path::Path;
    /// assert_eq!(DataFormat::from_path(Path::new("data.json")), Some(DataFormat::Json));
    /// assert_eq!(DataFormat::from_path(Path::new("data.yml")), Some(DataFormat::Yaml));
    /// assert_eq!(DataFormat::from_path(Path::new("data.txt")), None);
    /// ```
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        Self::from_extension(ext)
    }

    /// Determine the [`DataFormat`] from a bare extension string (no leading dot).
    ///
    /// The comparison is case-insensitive.
    ///
    /// Returns `None` if the extension is not recognized.
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            "yaml" | "yml" => Some(Self::Yaml),
            "toml" => Some(Self::Toml),
            "xlsx" => Some(Self::Xlsx),
            "sqlite" | "sqlite3" | "db" => Some(Self::Sqlite),
            _ => None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// `test_BC_5_03_002_from_extension` — known extensions map to correct variants.
    #[test]
    fn test_bc_5_03_002_from_extension() {
        assert_eq!(DataFormat::from_extension("json"), Some(DataFormat::Json));
        assert_eq!(DataFormat::from_extension("csv"), Some(DataFormat::Csv));
        assert_eq!(DataFormat::from_extension("yaml"), Some(DataFormat::Yaml));
        assert_eq!(DataFormat::from_extension("toml"), Some(DataFormat::Toml));
    }

    /// `test_BC_5_03_002_from_extension_yml` — `.yml` maps to Yaml (EC-008).
    #[test]
    fn test_bc_5_03_002_from_extension_yml() {
        assert_eq!(DataFormat::from_extension("yml"), Some(DataFormat::Yaml));
    }

    /// `test_BC_5_03_002_from_extension_unknown` — unrecognized extension returns None.
    #[test]
    fn test_bc_5_03_002_from_extension_unknown() {
        assert_eq!(DataFormat::from_extension("txt"), None);
        assert_eq!(DataFormat::from_extension("exe"), None);
        assert_eq!(DataFormat::from_extension(""), None);
    }

    /// `test_BC_5_03_002_from_path` — `Path::from_path` delegates to `from_extension`.
    #[test]
    fn test_bc_5_03_002_from_path() {
        assert_eq!(
            DataFormat::from_path(Path::new("data.json")),
            Some(DataFormat::Json)
        );
        assert_eq!(
            DataFormat::from_path(Path::new("data.yaml")),
            Some(DataFormat::Yaml)
        );
        assert_eq!(
            DataFormat::from_path(Path::new("data.yml")),
            Some(DataFormat::Yaml)
        );
        assert_eq!(
            DataFormat::from_path(Path::new("data.csv")),
            Some(DataFormat::Csv)
        );
        assert_eq!(
            DataFormat::from_path(Path::new("data.toml")),
            Some(DataFormat::Toml)
        );
        assert_eq!(DataFormat::from_path(Path::new("data.txt")), None);
    }

    /// `test_BC_5_03_002_case_insensitive` — extension matching is case-insensitive.
    #[test]
    fn test_bc_5_03_002_case_insensitive() {
        assert_eq!(DataFormat::from_extension("JSON"), Some(DataFormat::Json));
        assert_eq!(DataFormat::from_extension("CSV"), Some(DataFormat::Csv));
        assert_eq!(DataFormat::from_extension("YAML"), Some(DataFormat::Yaml));
        assert_eq!(DataFormat::from_extension("TOML"), Some(DataFormat::Toml));
    }

    /// `test_BC_1_03_006_xlsx_extension` — `.xlsx` maps to `DataFormat::Xlsx` (STORY-020 AC-001).
    #[test]
    fn test_bc_1_03_006_xlsx_extension() {
        assert_eq!(DataFormat::from_extension("xlsx"), Some(DataFormat::Xlsx));
        assert_eq!(DataFormat::from_extension("XLSX"), Some(DataFormat::Xlsx));
        assert_eq!(
            DataFormat::from_path(Path::new("data.xlsx")),
            Some(DataFormat::Xlsx)
        );
    }

    /// `test_BC_1_03_006_xls_unrecognized` — `.xls` (legacy) is NOT recognized (STORY-020 AC-005).
    ///
    /// `.xls` must not map to `DataFormat::Xlsx`. The `XlsxDataSource` produces
    /// `DataError::UnsupportedFormat` if the file has an `.xls` extension.
    #[test]
    fn test_bc_1_03_006_xls_unrecognized() {
        assert_eq!(
            DataFormat::from_extension("xls"),
            None,
            ".xls must not map to any DataFormat — only .xlsx is supported"
        );
    }

    /// `test_BC_1_03_007_sqlite_extensions` — `.sqlite`, `.sqlite3`, `.db` map to `DataFormat::Sqlite`.
    #[test]
    fn test_bc_1_03_007_sqlite_extensions() {
        assert_eq!(
            DataFormat::from_extension("sqlite"),
            Some(DataFormat::Sqlite)
        );
        assert_eq!(
            DataFormat::from_extension("sqlite3"),
            Some(DataFormat::Sqlite)
        );
        assert_eq!(DataFormat::from_extension("db"), Some(DataFormat::Sqlite));
        assert_eq!(
            DataFormat::from_path(Path::new("app.sqlite3")),
            Some(DataFormat::Sqlite)
        );
    }
}
