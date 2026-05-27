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
    /// Excel XLSX (`.xlsx`) — reserved for future implementation.
    Xlsx,
    /// `SQLite` (`.sqlite`, `.db`) — reserved for future implementation.
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
            "sqlite" | "db" => Some(Self::Sqlite),
            _ => None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// test_BC_5_03_002_from_extension — known extensions map to correct variants.
    #[test]
    fn test_bc_5_03_002_from_extension() {
        assert_eq!(DataFormat::from_extension("json"), Some(DataFormat::Json));
        assert_eq!(DataFormat::from_extension("csv"), Some(DataFormat::Csv));
        assert_eq!(DataFormat::from_extension("yaml"), Some(DataFormat::Yaml));
        assert_eq!(DataFormat::from_extension("toml"), Some(DataFormat::Toml));
    }

    /// test_BC_5_03_002_from_extension_yml — `.yml` maps to Yaml (EC-008).
    #[test]
    fn test_bc_5_03_002_from_extension_yml() {
        assert_eq!(DataFormat::from_extension("yml"), Some(DataFormat::Yaml));
    }

    /// test_BC_5_03_002_from_extension_unknown — unrecognized extension returns None.
    #[test]
    fn test_bc_5_03_002_from_extension_unknown() {
        assert_eq!(DataFormat::from_extension("txt"), None);
        assert_eq!(DataFormat::from_extension("exe"), None);
        assert_eq!(DataFormat::from_extension(""), None);
    }

    /// test_BC_5_03_002_from_path — Path::from_path delegates to from_extension.
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

    /// test_BC_5_03_002_case_insensitive — extension matching is case-insensitive.
    #[test]
    fn test_bc_5_03_002_case_insensitive() {
        assert_eq!(DataFormat::from_extension("JSON"), Some(DataFormat::Json));
        assert_eq!(DataFormat::from_extension("CSV"), Some(DataFormat::Csv));
        assert_eq!(DataFormat::from_extension("YAML"), Some(DataFormat::Yaml));
        assert_eq!(DataFormat::from_extension("TOML"), Some(DataFormat::Toml));
    }
}
