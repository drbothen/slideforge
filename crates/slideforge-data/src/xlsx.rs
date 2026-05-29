//! Excel XLSX [`DataSource`] implementation.
//!
//! [`XlsxDataSource`] implements the [`slideforge_plugin_api::DataSource`] trait
//! for `.xlsx` workbook files using the [`calamine`] crate (pure Rust, no C
//! dependencies).
//!
//! ## Sheet selection
//!
//! If `sheet` is `None`, the first sheet in the workbook is used. If `sheet`
//! is `Some(name)`, the named sheet is selected; a missing sheet produces
//! [`crate::DataError::ParseError`] with the list of available sheet names.
//!
//! ## Row mapping
//!
//! The first row of the selected sheet is the header row (column names). Each
//! subsequent row becomes a [`slideforge_types::Value::Map`] keyed by header
//! values. Empty cells become [`slideforge_types::Value::Null`].
//!
//! ## Format restrictions
//!
//! Only `.xlsx` (Office Open XML) format is supported. `.xls` (legacy binary)
//! files are rejected with [`crate::DataError::UnsupportedFormat`].
//!
//! ## Error codes
//!
//! | Condition | Error |
//! |-----------|-------|
//! | File does not exist | `E-DAT-004` ([`crate::DataError::FileNotFound`]) |
//! | `.xls` extension | `E-DAT-003` ([`crate::DataError::UnsupportedFormat`]) |
//! | Named sheet not found | `E-DAT-003` ([`crate::DataError::ParseError`]) |
//! | Empty header row | `E-DAT-003` ([`crate::DataError::ParseError`]) |
//! | Merged cells in header | `E-DAT-003` ([`crate::DataError::ParseError`]) |
//! | `calamine` open error | `E-DAT-003` ([`crate::DataError::ParseError`]) |
//!
//! Traces to BC-1.03.006.

use std::sync::Arc;

use calamine::{Data, Xlsx};
use slideforge_plugin_api::{DataSource, DataSourceError, DataSourceOptions};
use slideforge_types::Value;

use crate::DataError;

/// The built-in Excel XLSX data source plugin.
///
/// Reads `.xlsx` workbooks and maps each data row to a
/// [`slideforge_types::Value::Map`] keyed by the header row values.
/// The plugin identifier is `"xlsx"`.
///
/// ## Usage in the DSL
///
/// ```text
/// @data sales from "data/sales.xlsx"
/// @data regional from "data/report.xlsx" sheet: "West"
/// ```
///
/// Traces to BC-1.03.006.
#[derive(Debug, Clone)]
pub struct XlsxDataSource {
    /// The path to the `.xlsx` workbook file.
    pub path: Arc<str>,

    /// The sheet to read. `None` means the first sheet in the workbook.
    pub sheet: Option<Arc<str>>,
}

impl XlsxDataSource {
    /// Construct a new [`XlsxDataSource`] for the given file path.
    ///
    /// Uses the first sheet in the workbook.
    #[must_use]
    pub fn new(path: impl Into<Arc<str>>) -> Self {
        XlsxDataSource {
            path: path.into(),
            sheet: None,
        }
    }

    /// Construct a new [`XlsxDataSource`] targeting a specific named sheet.
    #[must_use]
    pub fn with_sheet(mut self, sheet: impl Into<Arc<str>>) -> Self {
        self.sheet = Some(sheet.into());
        self
    }
}

impl DataSource for XlsxDataSource {
    fn id(&self) -> &'static str {
        "xlsx"
    }

    /// Load the XLSX workbook and return `Value::List(rows)`.
    ///
    /// # Steps
    ///
    /// 1. Reject `.xls` extension with `UnsupportedFormat`.
    /// 2. Open the workbook via `calamine::open_workbook`.
    /// 3. Select the target sheet (first or named).
    /// 4. Validate the header row (non-empty, no merged cells).
    /// 5. Convert each data row to `Value::Map` keyed by header values.
    /// 6. Return `Value::List(rows)`.
    ///
    /// # Errors
    ///
    /// Returns [`DataSourceError`] on file-not-found, unsupported format,
    /// missing sheet, empty header, merged header cells, or parse failure.
    ///
    /// Traces to BC-1.03.006 postconditions 1-7.
    fn load(&self, uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        // Resolve the effective path: prefer uri if non-empty, fall back to self.path.
        let path_str: &str = if uri.is_empty() {
            self.path.as_ref()
        } else {
            uri
        };
        todo!(
            "BC-1.03.006: open workbook, validate header row, \
            convert rows to Value::List(Vec<Value::Map>); \
            path={path_str:?}, sheet={:?}",
            self.sheet
        )
    }
}

/// Convert a single [`calamine::Data`] cell value to a [`slideforge_types::Value`].
///
/// Mapping (BC-1.03.006 postconditions 4, 5, 6):
/// - `Empty` => `Value::Null`
/// - `Int` => `Value::Int(i64)`
/// - `Float` => `Value::Float(OrderedFloat(f64))`
/// - `Bool` => `Value::Bool`
/// - `String` => `Value::Str(Arc<str>)`
/// - `DateTime` => `Value::Str` in ISO 8601 format
/// - `Error(_)` => `Value::Null` (formula errors are nulled, not propagated)
/// - `DateTimeIso` / `DurationIso` => `Value::Str` as-is
///
/// Formula cells: `calamine::Data::Formula` is not present in calamine 0.26+;
/// formula cached values are represented by one of the above variants directly.
///
/// Traces to BC-1.03.006 postconditions 4-7 (cell type mapping).
#[allow(dead_code)] // Implementer will call this from DataSource::load
fn convert_calamine_cell(cell: &Data) -> Value {
    todo!(
        "BC-1.03.006: map calamine::Data variant {:?} to the correct Value variant \
        (Int=>Int, Float=>Float, Bool=>Bool, String=>Str, Empty=>Null, DateTime=>Str ISO 8601, \
        Error=>Null)",
        cell
    )
}

/// Convert a [`DataError`] from the XLSX layer to a [`DataSourceError`].
///
/// Used at the `DataSource::load` boundary to convert rich internal errors to
/// the plugin-api error type.
#[allow(dead_code)] // Implementer will call this from DataSource::load
fn data_error_to_source_error(path: &str, err: &DataError) -> DataSourceError {
    match err {
        DataError::FileNotFound { .. } => DataSourceError::IoError {
            uri: path.to_owned(),
            message: err.to_string(),
        },
        DataError::UnsupportedFormat { .. } => DataSourceError::UnsupportedUri {
            uri: path.to_owned(),
        },
        _ => DataSourceError::ParseError {
            uri: path.to_owned(),
            message: err.to_string(),
        },
    }
}

/// Validate that the `.xlsx` extension is used (not `.xls`).
///
/// Returns `Err(DataError::UnsupportedFormat)` with the actionable hint
/// `"Only .xlsx format is supported in v1.0. Convert to .xlsx before use."` if
/// the path ends in `.xls`.
///
/// Traces to BC-1.03.006 invariant 3 and edge case EC-002 (AC-005).
#[allow(dead_code)] // Implementer will call this from DataSource::load
fn reject_xls_extension(path: &str) -> Result<(), DataError> {
    todo!(
        "BC-1.03.006 AC-005: if path ends in '.xls' (case-insensitive), \
        return DataError::UnsupportedFormat; path={path:?}"
    )
}

/// Select the target sheet by name or index, returning the sheet range data.
///
/// If `sheet` is `None`, selects the first sheet. If `sheet` is `Some(name)`,
/// searches by name. Returns `Err` with the list of available sheet names if
/// the named sheet is not found.
///
/// Traces to BC-1.03.006 edge case EC-003 (AC-004).
#[allow(dead_code)] // Implementer will call this from DataSource::load
fn select_sheet(
    _workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>,
    sheet: Option<&str>,
    path: &str,
) -> Result<calamine::Range<Data>, DataError> {
    todo!(
        "BC-1.03.006 AC-004: select sheet by name or first; \
        if named sheet missing, return DataError::ParseError with available sheets list; \
        path={path:?}, sheet={sheet:?}"
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    /// `test_BC_1_03_006_xlsx_source_id` -- plugin ID is `"xlsx"`.
    ///
    /// Traces to BC-1.03.006 AC-001.
    #[test]
    fn test_bc_1_03_006_xlsx_source_id() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_006_xlsx_happy_path` -- basic workbook with headers+rows loads correctly.
    ///
    /// Create xlsx with headers `["name", "score"]`, rows `[["Alice", 95], ["Bob", 87]]`.
    /// Expect `Value::List` of 2 maps with correct keys and `Value::Int` scores.
    ///
    /// Traces to BC-1.03.006 AC-001, AC-002, AC-003.
    #[test]
    fn test_bc_1_03_006_xlsx_happy_path() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_006_xlsx_empty_cell_is_null` -- empty cell becomes `Value::Null`.
    ///
    /// Traces to BC-1.03.006 AC-003.
    #[test]
    fn test_bc_1_03_006_xlsx_empty_cell_is_null() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_006_xlsx_date_cell_iso` -- date cell emits ISO 8601 string.
    ///
    /// Traces to BC-1.03.006 AC-003 (`DateTime` => `Value::Str` ISO 8601).
    #[test]
    fn test_bc_1_03_006_xlsx_date_cell_iso() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_006_xlsx_missing_sheet` -- named sheet not found returns `DataError::ParseError`.
    ///
    /// Error message must contain "not found" and "Available sheets:".
    ///
    /// Traces to BC-1.03.006 AC-004, edge case EC-003.
    #[test]
    fn test_bc_1_03_006_xlsx_missing_sheet() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_006_xls_rejected` -- `.xls` extension returns `DataError::UnsupportedFormat`.
    ///
    /// Error message must contain ".xlsx" suggestion.
    ///
    /// Traces to BC-1.03.006 AC-005, invariant 3, edge case EC-002.
    #[test]
    fn test_bc_1_03_006_xls_rejected() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_006_xlsx_formula_uses_cached` -- formula cell uses cached value, not formula text.
    ///
    /// Traces to BC-1.03.006 AC-007, edge case EC-006.
    #[test]
    fn test_bc_1_03_006_xlsx_formula_uses_cached() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_006_xlsx_empty_header_row` -- empty header row returns `DataError::ParseError`.
    ///
    /// Error message must contain "empty sheet" (BC-1.03.006 AC-002, EC-004).
    #[test]
    fn test_bc_1_03_006_xlsx_empty_header_row() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_006_xlsx_merged_header_cells` -- merged cells in header returns `DataError::ParseError`.
    ///
    /// Error message must contain "merged cells in header row" (BC-1.03.006 AC-006, EC-005).
    #[test]
    fn test_bc_1_03_006_xlsx_merged_header_cells() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_006_xlsx_file_not_found` -- missing file returns `DataError::FileNotFound`.
    ///
    /// Traces to BC-1.03.006 edge case EC-001.
    #[test]
    fn test_bc_1_03_006_xlsx_file_not_found() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_006_convert_calamine_cell_int` -- `Data::Int` maps to `Value::Int`.
    ///
    /// Traces to BC-1.03.006 postcondition 4.
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_int() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_006_convert_calamine_cell_float` -- `Data::Float` maps to `Value::Float`.
    ///
    /// Traces to BC-1.03.006 postcondition 4.
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_float() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_006_convert_calamine_cell_bool` -- `Data::Bool` maps to `Value::Bool`.
    ///
    /// Traces to BC-1.03.006 postcondition 5.
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_bool() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_006_convert_calamine_cell_string` -- `Data::String` maps to `Value::Str`.
    ///
    /// Traces to BC-1.03.006 postcondition 4.
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_string() {
        panic!("not yet implemented (Red Gate)")
    }

    /// `test_BC_1_03_006_convert_calamine_cell_empty` -- `Data::Empty` maps to `Value::Null`.
    ///
    /// Traces to BC-1.03.006 postcondition 6.
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_empty() {
        panic!("not yet implemented (Red Gate)")
    }
}
