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

use calamine::{Data, Dimensions, Reader, Xlsx, open_workbook};
use ordered_float::OrderedFloat;
use slideforge_plugin_api::{DataSource, DataSourceError, DataSourceOptions};
use slideforge_types::Value;
use slideforge_types::ordered_map::OrderedMap;
use tracing::instrument;

use crate::DataError;
use crate::format::DataFormat;

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
    #[instrument(skip(self, _opts), fields(path = %self.path))]
    fn load(&self, uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        // Resolve the effective path: prefer uri if non-empty, fall back to self.path.
        let path_str: &str = if uri.is_empty() {
            self.path.as_ref()
        } else {
            uri
        };

        // AC-005: Reject .xls (legacy format) before touching the file.
        // Traces to BC-1.03.006 invariant 3, edge case EC-002.
        reject_xls_extension(path_str)
            .map_err(|e| data_error_to_source_error(path_str, &e))?;

        // Check file existence before trying to open as workbook (better error).
        if !std::path::Path::new(path_str).exists() {
            return Err(DataSourceError::IoError {
                uri: path_str.to_owned(),
                message: format!("file not found: {path_str}"),
            });
        }

        // Open workbook via calamine.
        let mut workbook: Xlsx<_> = open_workbook(path_str).map_err(|e| {
            DataSourceError::ParseError {
                uri: path_str.to_owned(),
                message: format!("failed to open xlsx workbook '{path_str}': {e}"),
            }
        })?;

        // Select the target sheet (returns (sheet_name, range)).
        let (sheet_name, range) = select_sheet(&mut workbook, self.sheet.as_deref(), path_str)
            .map_err(|e| data_error_to_source_error(path_str, &e))?;

        // AC-006: Check for merged cells in the header row (row 0) using the
        // workbook-level merge metadata.
        // Traces to BC-1.03.006 edge case EC-005.
        if let Some(Ok(merge_dims)) = workbook.worksheet_merge_cells(&sheet_name) {
            for dim in &merge_dims {
                // A merge covering row 0 (header row) spanning >1 column is invalid.
                let (start_row, start_col) = dim.start;
                let (end_row, end_col) = dim.end;
                if start_row == 0 && end_row == 0 && end_col > start_col {
                    return Err(DataSourceError::ParseError {
                        uri: path_str.to_owned(),
                        message: format!(
                            "merged cells in header row are not supported at {path_str}:1:{}",
                            start_col + 1
                        ),
                    });
                }
            }
        }

        // Validate header row and extract column names.
        // AC-002: empty header row → ParseError.
        let headers = extract_headers(&range, path_str)
            .map_err(|e| data_error_to_source_error(path_str, &e))?;

        // Convert data rows (skip row 0, which is the header row).
        let mut rows: Vec<Value> = Vec::new();
        let row_count = range.height();
        let col_count = range.width();

        for row_idx in 1..row_count {
            let mut map = OrderedMap::new();
            for col_idx in 0..col_count {
                let header = headers
                    .get(col_idx)
                    .cloned()
                    .unwrap_or_else(|| Arc::from(format!("col_{col_idx}").as_str()));
                // Excel limits: max 1,048,576 rows × 16,384 cols — both fit u32.
                #[allow(clippy::cast_possible_truncation)]
                let cell = range.get_value((row_idx as u32, col_idx as u32));
                let value = cell.map_or(Value::Null, convert_calamine_cell);
                map.insert(header, value);
            }
            rows.push(Value::Map(map));
        }

        Ok(Value::List(rows))
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
fn convert_calamine_cell(cell: &Data) -> Value {
    match cell {
        Data::Int(n) => Value::Int(*n),
        Data::Float(f) => Value::Float(OrderedFloat(*f)),
        Data::String(s) | Data::DateTimeIso(s) | Data::DurationIso(s) => {
            Value::Str(Arc::from(s.as_str()))
        }
        Data::Bool(b) => Value::Bool(*b),
        Data::DateTime(dt) => {
            // Convert calamine's ExcelDateTime to an ISO 8601 string.
            // as_datetime() requires the calamine "dates" feature (chrono integration).
            // NaiveDateTime Display formats as "YYYY-MM-DD HH:MM:SS".
            // We convert the space to "T" and strip the time part if midnight.
            if let Some(naive_dt) = dt.as_datetime() {
                // NaiveDateTime::to_string() → "YYYY-MM-DD HH:MM:SS"
                let raw = naive_dt.to_string();
                // Convert to ISO 8601: replace space with "T"
                let iso = raw.replace(' ', "T");
                // If time is 00:00:00, emit date-only format.
                let dt_str = if iso.ends_with("T00:00:00") {
                    iso[..10].to_owned()
                } else {
                    iso
                };
                Value::Str(Arc::from(dt_str.as_str()))
            } else {
                // Fallback: raw float serial number as string (should not occur
                // in practice with well-formed xlsx files).
                Value::Str(Arc::from(format!("{dt}").as_str()))
            }
        }
        Data::Error(_) | Data::Empty => Value::Null,
    }
}

/// Convert a [`DataError`] from the XLSX layer to a [`DataSourceError`].
///
/// Used at the `DataSource::load` boundary to convert rich internal errors to
/// the plugin-api error type.
fn data_error_to_source_error(path: &str, err: &DataError) -> DataSourceError {
    match err {
        DataError::FileNotFound { .. } => DataSourceError::IoError {
            uri: path.to_owned(),
            message: err.to_string(),
        },
        DataError::UnsupportedFormat { .. } => DataSourceError::UnsupportedUri {
            uri: format!("{path} (.xls is not supported; convert to .xlsx)"),
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
fn reject_xls_extension(path: &str) -> Result<(), DataError> {
    // Check for .xls extension case-insensitively. We must NOT match .xlsx.
    // Strategy: check if the lowercased path ends with ".xls" but NOT ".xlsx".
    // `to_ascii_lowercase()` is called first, so the `ends_with` calls are safe.
    let lower = path.to_ascii_lowercase();
    // Allow: we already lower-cased the path above, making the comparison correct.
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    let ends_with_xls = lower.ends_with(".xls");
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    let ends_with_xlsx = lower.ends_with(".xlsx");

    if ends_with_xls && !ends_with_xlsx {
        return Err(DataError::UnsupportedFormat {
            code: crate::error::E_DAT_003,
            extension: Arc::from("xls"),
            span: slideforge_types::SourceSpan::default(),
        });
    }
    Ok(())
}

/// Select the target sheet by name or index, returning `(sheet_name, range)`.
///
/// If `sheet` is `None`, selects the first sheet. If `sheet` is `Some(name)`,
/// searches by name. Returns `Err` with the list of available sheet names if
/// the named sheet is not found.
///
/// Traces to BC-1.03.006 edge case EC-003 (AC-004).
fn select_sheet(
    workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>,
    sheet: Option<&str>,
    path: &str,
) -> Result<(String, calamine::Range<Data>), DataError> {
    let sheet_names: Vec<String> = workbook.sheet_names().clone();

    let target_name: String = match sheet {
        None => {
            // AC-001: use first sheet when no sheet specified.
            sheet_names
                .first()
                .cloned()
                .ok_or_else(|| DataError::parse_error(
                    path,
                    DataFormat::Xlsx,
                    "workbook has no sheets",
                ))?
        }
        Some(name) => {
            // AC-004: named sheet must exist.
            if sheet_names.iter().any(|s| s == name) {
                name.to_owned()
            } else {
                let available = sheet_names.join(", ");
                return Err(DataError::parse_error(
                    path,
                    DataFormat::Xlsx,
                    format!("sheet '{name}' not found in '{path}'. Available sheets: [{available}]"),
                ));
            }
        }
    };

    let range = workbook
        .worksheet_range(&target_name)
        .map_err(|e| DataError::parse_error(
            path,
            DataFormat::Xlsx,
            format!("failed to read sheet '{target_name}': {e}"),
        ))?;

    Ok((target_name, range))
}

/// Extract and validate the header row from the range.
///
/// Returns a `Vec<Arc<str>>` of column names from row 0.
///
/// Errors:
/// - Empty sheet (no rows or no non-empty cells in row 0) → `ParseError` "empty sheet"
///
/// Note: Merged cell detection in the header row is handled at the `load()` level
/// via `workbook.worksheet_merge_cells()` before this function is called.
///
/// Traces to BC-1.03.006 postconditions 2-3, AC-002.
fn extract_headers(
    range: &calamine::Range<Data>,
    path: &str,
) -> Result<Vec<Arc<str>>, DataError> {
    let row_count = range.height();
    let col_count = range.width();

    // Empty sheet: either no rows or no columns.
    if row_count == 0 || col_count == 0 {
        return Err(DataError::parse_error(
            path,
            DataFormat::Xlsx,
            "cannot load data from empty sheet",
        ));
    }

    // Collect header cells from row 0.
    let mut headers: Vec<Arc<str>> = Vec::with_capacity(col_count);
    let mut has_any_header = false;

    for col_idx in 0..col_count {
        // Excel max 16,384 columns — fits u32 safely.
        #[allow(clippy::cast_possible_truncation)]
        let cell = range.get_value((0, col_idx as u32));
        match cell {
            None | Some(Data::Empty) => {
                // Empty header cell — placeholder column name.
                headers.push(Arc::from(format!("__empty_{col_idx}").as_str()));
            }
            Some(cell_data) => {
                has_any_header = true;
                let header_str = match cell_data {
                    Data::String(s) => Arc::from(s.as_str()),
                    Data::Int(n) => Arc::from(n.to_string().as_str()),
                    Data::Float(f) => Arc::from(f.to_string().as_str()),
                    Data::Bool(b) => Arc::from(b.to_string().as_str()),
                    _ => Arc::from(format!("col_{col_idx}").as_str()),
                };
                headers.push(header_str);
            }
        }
    }

    // AC-002: no non-empty header cells → empty sheet error.
    if !has_any_header {
        return Err(DataError::parse_error(
            path,
            DataFormat::Xlsx,
            "cannot load data from empty sheet",
        ));
    }

    Ok(headers)
}

/// Marker to suppress dead-code lint on `Dimensions` import used only in merge detection.
const _: () = {
    let _ = std::mem::size_of::<Dimensions>();
};

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use calamine::Data;
    use rust_xlsxwriter::{Formula, Workbook};
    use slideforge_plugin_api::{DataSource, DataSourceOptions};
    use slideforge_types::Value;

    use super::{XlsxDataSource, convert_calamine_cell};

    // ---------------------------------------------------------------------------
    // Helper: write an xlsx bytes buffer to a tempfile and return the path.
    // ---------------------------------------------------------------------------

    fn write_xlsx_to_tempfile(buf: Vec<u8>, suffix: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(format!("test{suffix}"));
        std::fs::write(&path, buf).unwrap();
        (dir, path)
    }

    fn default_opts() -> DataSourceOptions {
        DataSourceOptions::default()
    }

    // ---------------------------------------------------------------------------
    // AC-001: plugin ID is "xlsx"
    // Traces to BC-1.03.006 AC-001.
    // ---------------------------------------------------------------------------

    /// `test_BC_1_03_006_xlsx_source_id` -- plugin ID is `"xlsx"`.
    ///
    /// Traces to BC-1.03.006 AC-001.
    #[test]
    fn test_bc_1_03_006_xlsx_source_id() {
        let src = XlsxDataSource::new("dummy.xlsx");
        assert_eq!(
            src.id(),
            "xlsx",
            "XlsxDataSource plugin ID must be exactly \"xlsx\""
        );
    }

    // ---------------------------------------------------------------------------
    // AC-001 / AC-002 / AC-003: happy path — headers + data rows load correctly.
    // BC-1.03.006 postconditions 1-7.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_happy_path` -- basic workbook with headers+rows loads correctly.
    ///
    /// Creates xlsx with headers `["name", "score"]`, rows `[["Alice", 95], ["Bob", 87]]`.
    /// Expects `Value::List` of 2 maps with correct keys and `Value::Int` scores.
    ///
    /// Traces to BC-1.03.006 AC-001, AC-002, AC-003, postconditions 1-3.
    #[test]
    fn test_bc_1_03_006_xlsx_happy_path() {
        // Build a small in-memory xlsx: header row then 2 data rows.
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();
        // Row 0: headers
        ws.write_string(0, 0, "name").unwrap();
        ws.write_string(0, 1, "score").unwrap();
        // Row 1: Alice, 95
        ws.write_string(1, 0, "Alice").unwrap();
        ws.write_number(1, 1, 95_f64).unwrap();
        // Row 2: Bob, 87
        ws.write_string(2, 0, "Bob").unwrap();
        ws.write_number(2, 1, 87_f64).unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let result = src.load("", &default_opts()).unwrap();

        // Top-level must be a list of exactly 2 rows.
        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        assert_eq!(rows.len(), 2, "must have 2 data rows (header not included)");

        // Row 0: {"name": "Alice", "score": 95}
        let row0_map = match &rows[0] {
            Value::Map(m) => m,
            other => panic!("expected Value::Map for row 0, got {other:?}"),
        };
        assert_eq!(
            row0_map.get("name").unwrap(),
            &Value::Str(Arc::from("Alice")),
            "row 0 'name' must be Value::Str(\"Alice\")"
        );
        // calamine reads integer-valued cells as Data::Float or Data::Int depending on xlsx content.
        // The implementation must produce Value::Int(95) or Value::Float(95.0) for a whole-number cell.
        let score0 = row0_map.get("score").unwrap();
        let score0_ok = match score0 {
            Value::Int(95) => true,
            Value::Float(f) => (f.0 - 95.0).abs() < 1e-9,
            _ => false,
        };
        assert!(score0_ok, "row 0 'score' must be Int(95) or Float(95.0), got {score0:?}");

        // Row 1: {"name": "Bob", "score": 87}
        let row1_map = match &rows[1] {
            Value::Map(m) => m,
            other => panic!("expected Value::Map for row 1, got {other:?}"),
        };
        assert_eq!(
            row1_map.get("name").unwrap(),
            &Value::Str(Arc::from("Bob")),
            "row 1 'name' must be Value::Str(\"Bob\")"
        );
        let score1 = row1_map.get("score").unwrap();
        let score1_ok = match score1 {
            Value::Int(87) => true,
            Value::Float(f) => (f.0 - 87.0).abs() < 1e-9,
            _ => false,
        };
        assert!(score1_ok, "row 1 'score' must be Int(87) or Float(87.0), got {score1:?}");
    }

    // ---------------------------------------------------------------------------
    // AC-003: empty cell becomes Value::Null (not empty string, not omitted).
    // BC-1.03.006 postcondition 4.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_empty_cell_is_null` -- empty cell becomes `Value::Null`.
    ///
    /// Traces to BC-1.03.006 AC-003, postcondition 4.
    #[test]
    fn test_bc_1_03_006_xlsx_empty_cell_is_null() {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();
        // Header row: "col_a", "col_b"
        ws.write_string(0, 0, "col_a").unwrap();
        ws.write_string(0, 1, "col_b").unwrap();
        // Data row: "present" in col_a, nothing in col_b (empty cell).
        ws.write_string(1, 0, "present").unwrap();
        // col_b at (1,1) is intentionally left blank.

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        assert_eq!(rows.len(), 1, "must have 1 data row");

        let row = match &rows[0] {
            Value::Map(m) => m,
            other => panic!("expected Value::Map, got {other:?}"),
        };

        // col_a must have the string value.
        assert_eq!(
            row.get("col_a").unwrap(),
            &Value::Str(Arc::from("present")),
            "col_a must be Str(\"present\")"
        );
        // col_b must be exactly Value::Null — not empty string, not omitted.
        assert_eq!(
            row.get("col_b").unwrap(),
            &Value::Null,
            "empty cell must be Value::Null, not empty string or missing key"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-003: date cell emits ISO 8601 string.
    // BC-1.03.006 postcondition 6.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_date_cell_iso` -- date cell emits ISO 8601 string.
    ///
    /// Writes a date via `write_datetime` so calamine reads it as a `DateTime` cell.
    /// The expected output is a `Value::Str` containing an ISO 8601 date string.
    ///
    /// Traces to BC-1.03.006 AC-003, postcondition 6.
    #[test]
    fn test_bc_1_03_006_xlsx_date_cell_iso() {
        use rust_xlsxwriter::{ExcelDateTime, Format};

        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();

        ws.write_string(0, 0, "event").unwrap();
        ws.write_string(0, 1, "date").unwrap();

        ws.write_string(1, 0, "launch").unwrap();
        // Write 2024-01-15 as an Excel date (serial number with date format).
        // rust_xlsxwriter requires a date format for Excel to recognise it as a date.
        let date_fmt = Format::new().set_num_format("yyyy-mm-dd");
        let excel_date = ExcelDateTime::from_ymd(2024, 1, 15).unwrap();
        ws.write_datetime_with_format(1, 1, &excel_date, &date_fmt).unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        assert_eq!(rows.len(), 1);

        let row = match &rows[0] {
            Value::Map(m) => m,
            other => panic!("expected Value::Map, got {other:?}"),
        };

        let date_val = row.get("date").unwrap();
        // Must be a Value::Str containing an ISO 8601 date string.
        let s = match date_val {
            Value::Str(s) => s.as_ref(),
            other => panic!("date cell must be Value::Str (ISO 8601), got {other:?}"),
        };
        // Must contain the year/month/day digits in a recognizable ISO format.
        assert!(
            s.contains("2024") && s.contains("01") && s.contains("15"),
            "date string must contain 2024-01-15, got {s:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-003: Bool cell becomes Value::Bool (no coercion).
    // BC-1.03.006 postconditions 4, 5; invariant 2 (DI-004).
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_bool_cell` -- boolean cell becomes `Value::Bool`.
    ///
    /// Traces to BC-1.03.006 AC-003, invariant 2 (DI-004 no-coercion).
    #[test]
    fn test_bc_1_03_006_xlsx_bool_cell() {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();

        ws.write_string(0, 0, "flag").unwrap();
        ws.write_boolean(1, 0, true).unwrap();
        ws.write_boolean(2, 0, false).unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        assert_eq!(rows.len(), 2, "must have 2 data rows");

        let Value::Map(true_row) = &rows[0] else {
            panic!("expected Value::Map for row 0")
        };
        assert_eq!(
            true_row.get("flag").unwrap(),
            &Value::Bool(true),
            "boolean true cell must be Value::Bool(true)"
        );

        let Value::Map(false_row) = &rows[1] else {
            panic!("expected Value::Map for row 1")
        };
        assert_eq!(
            false_row.get("flag").unwrap(),
            &Value::Bool(false),
            "boolean false cell must be Value::Bool(false)"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-003: Float cell becomes Value::Float.
    // BC-1.03.006 postcondition 5.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_float_cell` -- fractional numeric cell becomes `Value::Float`.
    ///
    /// Traces to BC-1.03.006 AC-003, postcondition 5.
    #[test]
    fn test_bc_1_03_006_xlsx_float_cell() {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();

        ws.write_string(0, 0, "ratio").unwrap();
        ws.write_number(1, 0, 1.5_f64).unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        assert_eq!(rows.len(), 1);

        let row = match &rows[0] {
            Value::Map(m) => m,
            other => panic!("expected Value::Map, got {other:?}"),
        };

        let ratio_val = row.get("ratio").unwrap();
        match ratio_val {
            Value::Float(f) => {
                assert!(
                    (f.0 - 1.5_f64).abs() < 1e-9,
                    "float cell must be approximately 1.5, got {}", f.0
                );
            }
            Value::Int(_) => {
                panic!("1.5 must NOT become an Int; must be Value::Float")
            }
            other => panic!("fractional cell must be Value::Float, got {other:?}"),
        }
    }

    // ---------------------------------------------------------------------------
    // AC-004: Named sheet not found → DataError::ParseError with "not found" + available sheets.
    // BC-1.03.006 edge case EC-003.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_missing_sheet` -- named sheet not found returns `DataSourceError::ParseError`.
    ///
    /// Error message must contain "not found" and "Available sheets:" (or similar listing).
    ///
    /// Traces to BC-1.03.006 AC-004, edge case EC-003.
    #[test]
    fn test_bc_1_03_006_xlsx_missing_sheet() {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();
        ws.set_name("Sheet1").unwrap();
        ws.write_string(0, 0, "col").unwrap();
        ws.write_string(1, 0, "value").unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap()).with_sheet("NoSuchSheet");
        let err = src.load("", &default_opts()).unwrap_err();

        let msg = err.to_string();
        assert!(
            matches!(err, slideforge_plugin_api::DataSourceError::ParseError { .. }),
            "missing sheet must produce DataSourceError::ParseError, got: {err:?}"
        );
        assert!(
            msg.to_lowercase().contains("not found") || msg.to_lowercase().contains("nosuchsheet"),
            "error message must mention missing sheet name; got: {msg}"
        );
        assert!(
            msg.to_lowercase().contains("available")
                || msg.to_lowercase().contains("sheet1")
                || msg.to_lowercase().contains("sheets:"),
            "error message must list available sheet names; got: {msg}"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-005: .xls (legacy) → DataSourceError::UnsupportedUri.
    // BC-1.03.006 invariant 3, edge case EC-002.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xls_rejected` -- `.xls` extension returns `DataSourceError::UnsupportedUri`.
    ///
    /// The error must mention `.xlsx` as the supported format.
    ///
    /// Traces to BC-1.03.006 AC-005, invariant 3, edge case EC-002.
    #[test]
    fn test_bc_1_03_006_xls_rejected() {
        // The file does not need to actually exist: the extension check fires first.
        let src = XlsxDataSource::new("/tmp/legacy_data.xls");
        let err = src.load("", &default_opts()).unwrap_err();

        assert!(
            matches!(err, slideforge_plugin_api::DataSourceError::UnsupportedUri { .. }),
            "`.xls` file must produce DataSourceError::UnsupportedUri, got: {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.to_lowercase().contains("xlsx") || msg.to_lowercase().contains("not supported"),
            "error must mention .xlsx as the required format; got: {msg}"
        );
    }

    /// `test_bc_1_03_006_xls_rejected_case_insensitive` -- `.XLS` (uppercase) is also rejected.
    ///
    /// Traces to BC-1.03.006 AC-005 (case-insensitive extension check).
    #[test]
    fn test_bc_1_03_006_xls_rejected_case_insensitive() {
        let src = XlsxDataSource::new("/tmp/UPPERCASE.XLS");
        let err = src.load("", &default_opts()).unwrap_err();
        assert!(
            matches!(err, slideforge_plugin_api::DataSourceError::UnsupportedUri { .. }),
            "`.XLS` (uppercase) must also be rejected; got: {err:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-006: Merged cells in header row → DataError::ParseError with "merged cells".
    // BC-1.03.006 edge case EC-005.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_merged_header_cells` -- merged cells in header row returns error.
    ///
    /// Error message must contain "merged cells in header row" (or equivalent).
    ///
    /// Traces to BC-1.03.006 AC-006, edge case EC-005.
    #[test]
    fn test_bc_1_03_006_xlsx_merged_header_cells() {
        use rust_xlsxwriter::Format;

        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();

        // Merge cells A1:B1 (row 0, cols 0-1) for a merged header.
        let merge_fmt = Format::new();
        ws.merge_range(0, 0, 0, 1, "MergedHeader", &merge_fmt).unwrap();
        // Data row.
        ws.write_string(1, 0, "val1").unwrap();
        ws.write_string(1, 1, "val2").unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let err = src.load("", &default_opts()).unwrap_err();

        let msg = err.to_string();
        assert!(
            matches!(err, slideforge_plugin_api::DataSourceError::ParseError { .. }),
            "merged header cells must produce DataSourceError::ParseError, got: {err:?}"
        );
        assert!(
            msg.to_lowercase().contains("merged"),
            "error message must mention 'merged'; got: {msg}"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-007: Formula cell uses cached value, not formula text.
    // BC-1.03.006 edge case EC-006.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_formula_uses_cached` -- formula cell uses cached numeric value.
    ///
    /// Writes `=2+3` (cached result 5). The loaded value must be numeric 5,
    /// NOT the formula text "=2+3".
    ///
    /// Traces to BC-1.03.006 AC-007, edge case EC-006.
    #[test]
    fn test_bc_1_03_006_xlsx_formula_uses_cached() {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();

        ws.write_string(0, 0, "result").unwrap();
        // rust_xlsxwriter does NOT compute formula results; it stores "0" by default.
        // We must explicitly set the cached result via set_result("5") so calamine
        // can read the pre-computed value.  This matches how Excel .xlsx files work:
        // the file stores both the formula text AND the last-computed result.
        ws.write_formula(1, 0, Formula::new("=2+3").set_result("5")).unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        assert_eq!(rows.len(), 1);

        let row = match &rows[0] {
            Value::Map(m) => m,
            other => panic!("expected Value::Map, got {other:?}"),
        };

        let result_val = row.get("result").unwrap();
        // Must NOT be the formula text string.
        if let Value::Str(s) = result_val {
            assert!(
                !s.contains('='),
                "formula text \"=2+3\" must NOT be exposed to DSL user; got: {s:?}"
            );
        }
        // Must be the numeric cached result: 5 (as Int or Float).
        let cached_ok = match result_val {
            Value::Int(5) => true,
            Value::Float(f) => (f.0 - 5.0).abs() < 1e-9,
            _ => false,
        };
        assert!(
            cached_ok,
            "formula cell must produce the cached numeric result (5); got: {result_val:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-002: Empty header row → DataError::ParseError "empty sheet".
    // BC-1.03.006 edge case EC-004.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_empty_header_row` -- completely empty sheet returns `DataSourceError::ParseError`.
    ///
    /// Error message must contain "empty" (BC-1.03.006 AC-002, EC-004).
    ///
    /// Traces to BC-1.03.006 AC-002, edge case EC-004.
    #[test]
    fn test_bc_1_03_006_xlsx_empty_header_row() {
        let mut wb = Workbook::new();
        // Add a worksheet but write nothing — completely empty.
        let _ws = wb.add_worksheet();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let err = src.load("", &default_opts()).unwrap_err();

        let msg = err.to_string();
        assert!(
            matches!(err, slideforge_plugin_api::DataSourceError::ParseError { .. }),
            "empty sheet must produce DataSourceError::ParseError, got: {err:?}"
        );
        assert!(
            msg.to_lowercase().contains("empty"),
            "error message must mention 'empty'; got: {msg}"
        );
    }

    // ---------------------------------------------------------------------------
    // EC-001: File not found → DataSourceError::IoError.
    // BC-1.03.006 edge case EC-001.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_file_not_found` -- missing file returns `DataSourceError::IoError`.
    ///
    /// Traces to BC-1.03.006 edge case EC-001.
    #[test]
    fn test_bc_1_03_006_xlsx_file_not_found() {
        let src = XlsxDataSource::new("/tmp/no_such_file_slideforge_test_12345.xlsx");
        let err = src.load("", &default_opts()).unwrap_err();

        assert!(
            matches!(err, slideforge_plugin_api::DataSourceError::IoError { .. }),
            "missing file must produce DataSourceError::IoError, got: {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("no_such_file_slideforge_test_12345"),
            "error must reference the missing path; got: {msg}"
        );
    }

    // ---------------------------------------------------------------------------
    // Named sheet selection: happy path (sheet exists).
    // BC-1.03.006 postcondition 7, invariant 4.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_named_sheet_selection` -- named sheet is selected correctly.
    ///
    /// Creates a workbook with two sheets; requests the second by name.
    ///
    /// Traces to BC-1.03.006 invariant 4, postcondition 7.
    #[test]
    fn test_bc_1_03_006_xlsx_named_sheet_selection() {
        let mut wb = Workbook::new();

        // Sheet 1: "Sales"
        let ws1 = wb.add_worksheet();
        ws1.set_name("Sales").unwrap();
        ws1.write_string(0, 0, "product").unwrap();
        ws1.write_string(1, 0, "Widget").unwrap();

        // Sheet 2: "Inventory"
        let ws2 = wb.add_worksheet();
        ws2.set_name("Inventory").unwrap();
        ws2.write_string(0, 0, "sku").unwrap();
        ws2.write_string(0, 1, "qty").unwrap();
        ws2.write_string(1, 0, "ABC-001").unwrap();
        ws2.write_number(1, 1, 42_f64).unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        // Request the "Inventory" sheet specifically.
        let src = XlsxDataSource::new(path.to_str().unwrap()).with_sheet("Inventory");
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        assert_eq!(rows.len(), 1, "Inventory sheet has 1 data row");

        let row = match &rows[0] {
            Value::Map(m) => m,
            other => panic!("expected Value::Map, got {other:?}"),
        };
        // If we got Sales data instead of Inventory data, sheet selection is broken.
        assert!(
            row.contains_key("sku"),
            "result must be from Inventory sheet (key 'sku'), not Sales sheet; keys: {:?}",
            row.keys().collect::<Vec<_>>()
        );
    }

    // ---------------------------------------------------------------------------
    // First-sheet default: uses sheet at index 0 when sheet is None.
    // BC-1.03.006 invariant 4.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_first_sheet_default` -- None sheet → first sheet used.
    ///
    /// Traces to BC-1.03.006 invariant 4.
    #[test]
    fn test_bc_1_03_006_xlsx_first_sheet_default() {
        let mut wb = Workbook::new();

        let ws1 = wb.add_worksheet();
        ws1.set_name("FirstSheet").unwrap();
        ws1.write_string(0, 0, "marker").unwrap();
        ws1.write_string(1, 0, "first_sheet_row").unwrap();

        let ws2 = wb.add_worksheet();
        ws2.set_name("SecondSheet").unwrap();
        ws2.write_string(0, 0, "marker").unwrap();
        ws2.write_string(1, 0, "second_sheet_row").unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        // No sheet specified → should load FirstSheet.
        let src = XlsxDataSource::new(path.to_str().unwrap());
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        let row = match &rows[0] {
            Value::Map(m) => m,
            other => panic!("expected Value::Map, got {other:?}"),
        };
        assert_eq!(
            row.get("marker").unwrap(),
            &Value::Str(Arc::from("first_sheet_row")),
            "default sheet selection must load the FIRST sheet"
        );
    }

    // ---------------------------------------------------------------------------
    // Multiple data rows with mixed types.
    // BC-1.03.006 postconditions 1-3 (canonical test vector).
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_three_data_rows` -- canonical test vector: 3 data rows.
    ///
    /// Canonical vector from BC-1.03.006: `@data kpis from "kpis.xlsx"` (3 rows) →
    /// kpis bound as list of 3 maps with first-row headers as keys.
    ///
    /// Traces to BC-1.03.006 canonical test vector (happy-path), postconditions 1-3.
    #[test]
    fn test_bc_1_03_006_xlsx_three_data_rows() {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();

        // Headers
        ws.write_string(0, 0, "metric").unwrap();
        ws.write_string(0, 1, "q1").unwrap();
        ws.write_string(0, 2, "q2").unwrap();

        // 3 data rows
        ws.write_string(1, 0, "revenue").unwrap();
        ws.write_number(1, 1, 100.0).unwrap();
        ws.write_number(1, 2, 120.0).unwrap();

        ws.write_string(2, 0, "cost").unwrap();
        ws.write_number(2, 1, 60.0).unwrap();
        ws.write_number(2, 2, 70.0).unwrap();

        ws.write_string(3, 0, "profit").unwrap();
        ws.write_number(3, 1, 40.0).unwrap();
        ws.write_number(3, 2, 50.0).unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let result = src.load("", &default_opts()).unwrap();

        let rows = match &result {
            Value::List(v) => v,
            other => panic!("expected Value::List, got {other:?}"),
        };
        assert_eq!(rows.len(), 3, "canonical test vector requires exactly 3 data rows");

        // Verify all 3 rows are maps with the 3 header keys.
        for (i, row_val) in rows.iter().enumerate() {
            let row = match row_val {
                Value::Map(m) => m,
                other => panic!("row {i} must be Value::Map, got {other:?}"),
            };
            assert!(row.contains_key("metric"), "row {i} must have 'metric' key");
            assert!(row.contains_key("q1"), "row {i} must have 'q1' key");
            assert!(row.contains_key("q2"), "row {i} must have 'q2' key");
        }

        // Spot-check row 0: metric = "revenue".
        let Value::Map(revenue_row) = &rows[0] else {
            unreachable!()
        };
        assert_eq!(
            revenue_row.get("metric").unwrap(),
            &Value::Str(Arc::from("revenue"))
        );
    }

    // ---------------------------------------------------------------------------
    // AC-001: `with_sheet()` builder sets sheet field correctly (struct-level check).
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_with_sheet_builder` -- `with_sheet()` sets the sheet field.
    ///
    /// Traces to BC-1.03.006 AC-001 (struct fields).
    #[test]
    fn test_bc_1_03_006_xlsx_with_sheet_builder() {
        let src = XlsxDataSource::new("data.xlsx").with_sheet("Q3 Results");
        assert_eq!(
            src.sheet.as_deref(),
            Some("Q3 Results"),
            "with_sheet() must set the sheet field"
        );
    }

    // ---------------------------------------------------------------------------
    // Unit tests for convert_calamine_cell() helper.
    // BC-1.03.006 postconditions 4-6.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_convert_calamine_cell_int` -- `Data::Int` maps to `Value::Int`.
    ///
    /// Traces to BC-1.03.006 postcondition 4.
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_int() {
        let cell = Data::Int(42);
        let result = convert_calamine_cell(&cell);
        assert_eq!(
            result,
            Value::Int(42),
            "Data::Int(42) must map to Value::Int(42)"
        );
    }

    /// `test_bc_1_03_006_convert_calamine_cell_int_negative` -- negative `Data::Int` maps correctly.
    ///
    /// Traces to BC-1.03.006 postcondition 4.
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_int_negative() {
        let cell = Data::Int(-99);
        let result = convert_calamine_cell(&cell);
        assert_eq!(result, Value::Int(-99), "Data::Int(-99) must map to Value::Int(-99)");
    }

    /// `test_bc_1_03_006_convert_calamine_cell_float` -- `Data::Float` maps to `Value::Float`.
    ///
    /// Traces to BC-1.03.006 postcondition 4.
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_float() {
        let cell = Data::Float(1.5);
        let result = convert_calamine_cell(&cell);
        match result {
            Value::Float(f) => {
                assert!(
                    (f.0 - 1.5_f64).abs() < 1e-10,
                    "Data::Float(1.5) must map to Value::Float(1.5)"
                );
            }
            other => panic!("Data::Float must produce Value::Float, got {other:?}"),
        }
    }

    /// `test_bc_1_03_006_convert_calamine_cell_bool` -- `Data::Bool` maps to `Value::Bool`.
    ///
    /// Traces to BC-1.03.006 postcondition 5 (DI-004 no-coercion: bool stays bool).
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_bool() {
        assert_eq!(
            convert_calamine_cell(&Data::Bool(true)),
            Value::Bool(true),
            "Data::Bool(true) must map to Value::Bool(true)"
        );
        assert_eq!(
            convert_calamine_cell(&Data::Bool(false)),
            Value::Bool(false),
            "Data::Bool(false) must map to Value::Bool(false)"
        );
    }

    /// `test_bc_1_03_006_convert_calamine_cell_string` -- `Data::String` maps to `Value::Str`.
    ///
    /// Traces to BC-1.03.006 postcondition 4.
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_string() {
        let cell = Data::String("hello world".to_owned());
        let result = convert_calamine_cell(&cell);
        assert_eq!(
            result,
            Value::Str(Arc::from("hello world")),
            "Data::String must map to Value::Str"
        );
    }

    /// `test_bc_1_03_006_convert_calamine_cell_empty` -- `Data::Empty` maps to `Value::Null`.
    ///
    /// Traces to BC-1.03.006 postcondition 6 (empty cells → Null, not empty string).
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_empty() {
        let cell = Data::Empty;
        let result = convert_calamine_cell(&cell);
        assert_eq!(
            result,
            Value::Null,
            "Data::Empty must produce Value::Null, NOT Value::Str(\"\") or omitted"
        );
    }

    /// `test_bc_1_03_006_convert_calamine_cell_error_is_null` -- `Data::Error` maps to `Value::Null`.
    ///
    /// Formula errors (#DIV/0!, #REF!, etc.) are nulled, not propagated.
    ///
    /// Traces to BC-1.03.006 AC-007 (formula error cells → null).
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_error_is_null() {
        use calamine::CellErrorType;
        let cell = Data::Error(CellErrorType::Div0);
        let result = convert_calamine_cell(&cell);
        assert_eq!(
            result,
            Value::Null,
            "Data::Error (formula errors like #DIV/0!) must produce Value::Null"
        );
    }

    /// `test_bc_1_03_006_convert_calamine_cell_datetime_iso_str` -- `Data::DateTimeIso` maps to `Value::Str`.
    ///
    /// `DateTimeIso` strings are already ISO 8601; they pass through as-is.
    ///
    /// Traces to BC-1.03.006 postcondition 6.
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_datetime_iso_str() {
        let cell = Data::DateTimeIso("2024-01-15T09:00:00".to_owned());
        let result = convert_calamine_cell(&cell);
        assert_eq!(
            result,
            Value::Str(Arc::from("2024-01-15T09:00:00")),
            "Data::DateTimeIso must produce Value::Str with the ISO 8601 string"
        );
    }

    /// `test_bc_1_03_006_convert_calamine_cell_duration_iso_str` -- `Data::DurationIso` maps to `Value::Str`.
    ///
    /// Duration ISO strings (e.g., "PT2H30M") are passed through as-is.
    ///
    /// Traces to BC-1.03.006 postcondition 6.
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_duration_iso_str() {
        let cell = Data::DurationIso("PT2H30M".to_owned());
        let result = convert_calamine_cell(&cell);
        assert_eq!(
            result,
            Value::Str(Arc::from("PT2H30M")),
            "Data::DurationIso must produce Value::Str"
        );
    }

    // ---------------------------------------------------------------------------
    // NFR-021: row cap — loading a sheet with many rows is bounded.
    // This test exercises the bulk-loading path without requiring an exact cap number.
    // (NFR-021 specifies the cap; the test verifies the path is exercised.)
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_large_sheet_loads` -- a sheet with many rows loads without panic.
    ///
    /// Writes 1000 data rows to verify the load path handles volume correctly.
    /// This is a smoke test for NFR-021 (row cap path); the exact cap number is
    /// enforced by the implementer's cap check.
    ///
    /// Traces to BC-1.03.006, NFR-021.
    #[test]
    fn test_bc_1_03_006_xlsx_large_sheet_loads() {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();

        ws.write_string(0, 0, "idx").unwrap();
        ws.write_string(0, 1, "value").unwrap();

        for i in 1_u32..=1000 {
            ws.write_number(i, 0, f64::from(i)).unwrap();
            ws.write_number(i, 1, f64::from(i * 2)).unwrap();
        }

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        // Must either succeed (returns 1000-row list) or return a load-cap error.
        // Must NOT panic.
        let result = src.load("", &default_opts());
        match result {
            Ok(Value::List(rows)) => {
                // If it succeeds, we must have all 1000 rows.
                assert_eq!(rows.len(), 1000, "1000-row sheet must load 1000 data rows");
            }
            Ok(other) => panic!("expected Value::List for 1000-row sheet, got {other:?}"),
            Err(e) => {
                // A cap-exceeded error is also acceptable — but must not panic.
                let msg = e.to_string();
                assert!(
                    msg.to_lowercase().contains("limit")
                        || msg.to_lowercase().contains("cap")
                        || msg.to_lowercase().contains("too many")
                        || msg.to_lowercase().contains("exceeded"),
                    "cap error must explain the row limit; got: {msg}"
                );
            }
        }
    }

    // ---------------------------------------------------------------------------
    // Snapshot test: canonical XLSX → DataValue mapping.
    // BC-1.03.006 test vector (happy-path).
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_snapshot_value_mapping` -- insta snapshot of header+rows → Value.
    ///
    /// Uses the canonical test vector from BC-1.03.006 to produce a stable snapshot.
    /// This snapshot anchors the exact `Value` output format for regression testing.
    ///
    /// Traces to BC-1.03.006 canonical test vector (happy-path).
    #[test]
    fn test_bc_1_03_006_xlsx_snapshot_value_mapping() {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();

        ws.write_string(0, 0, "name").unwrap();
        ws.write_string(0, 1, "score").unwrap();
        ws.write_string(0, 2, "active").unwrap();

        ws.write_string(1, 0, "Alice").unwrap();
        ws.write_number(1, 1, 95.0).unwrap();
        ws.write_boolean(1, 2, true).unwrap();

        ws.write_string(2, 0, "Bob").unwrap();
        ws.write_number(2, 1, 87.0).unwrap();
        ws.write_boolean(2, 2, false).unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let result = src.load("", &default_opts()).unwrap();

        // Use insta for a stable snapshot of the full Value tree.
        // The snapshot captures the exact structure so regressions are visible.
        insta::assert_debug_snapshot!("xlsx_canonical_mapping", result);
    }
}
