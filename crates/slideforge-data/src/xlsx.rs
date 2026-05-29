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

use calamine::{Data, Reader, Xlsx, open_workbook};
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
    /// Traces to BC-1.03.006 postconditions 1-9.
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
        reject_xls_extension(path_str).map_err(|e| data_error_to_source_error(path_str, &e))?;

        // Check file existence before trying to open as workbook (better error).
        if !std::path::Path::new(path_str).exists() {
            return Err(DataSourceError::IoError {
                uri: path_str.to_owned(),
                message: format!("file not found: {path_str}"),
            });
        }

        // BC-1.03.006 postcondition 9 / invariant 3 (second phase):
        // Validate XLSX magic bytes (ZIP local file header PK\x03\x04).
        // Extension check already passed; this catches files that have .xlsx extension
        // but are not actually XLSX/ZIP archives.
        // Traces to VP-026, E-DAT-011.
        validate_xlsx_magic(path_str).map_err(|e| data_error_to_source_error(path_str, &e))?;

        // Open workbook via calamine.
        let mut workbook: Xlsx<_> =
            open_workbook(path_str).map_err(|e| DataSourceError::ParseError {
                uri: path_str.to_owned(),
                message: format!("failed to open xlsx workbook '{path_str}': {e}"),
            })?;

        // Select the target sheet (returns (sheet_name, range)).
        let (sheet_name, range) = select_sheet(&mut workbook, self.sheet.as_deref(), path_str)
            .map_err(|e| data_error_to_source_error(path_str, &e))?;

        // AC-006: Check for merged cells in the header row (row 0) using the
        // workbook-level merge metadata.
        // Traces to BC-1.03.006 edge case EC-005.
        if let Some(Ok(merge_dims)) = workbook.worksheet_merge_cells(&sheet_name) {
            for dim in &merge_dims {
                let (start_row, start_col) = dim.start;
                let (end_row, end_col) = dim.end;
                // Horizontal merge: a merge spanning >1 column within the header row.
                // Traces to BC-1.03.006 edge case EC-005.
                if start_row == 0 && end_row == 0 && end_col > start_col {
                    return Err(DataSourceError::ParseError {
                        uri: path_str.to_owned(),
                        message: format!(
                            "merged cells in header row are not supported at {path_str}:1:{}",
                            start_col + 1
                        ),
                    });
                }
                // Vertical merge: a merge starting in the header row and spanning
                // into one or more data rows. This creates a phantom header cell
                // across multiple rows, making row-to-header mapping ambiguous.
                // Traces to BC-1.03.006 edge case EC-005, F-LOW-9.
                if start_row == 0 && end_row > 0 {
                    return Err(DataSourceError::ParseError {
                        uri: path_str.to_owned(),
                        message: format!(
                            "vertically merged cell in header row at {path_str}:1:{} spans \
                            into data rows — this makes column mapping ambiguous. \
                            Split the merge before loading.",
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
                // cast_possible_truncation: usize→u32 is safe within Excel row/col limits.
                #[allow(clippy::cast_possible_truncation)]
                let (row_u32, col_u32) = (row_idx as u32, col_idx as u32);
                let cell = range.get_value((row_u32, col_u32));
                let value = match cell {
                    None | Some(Data::Empty) => Value::Null,
                    Some(c) => convert_calamine_cell(c, col_u32, row_u32, path_str)
                        .map_err(|e| data_error_to_source_error(path_str, &e))?,
                };
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
/// - `Float(f)` where `f.fract() == 0.0 && f.is_finite()` and in i64 range => `Value::Int(f as i64)`
/// - `Float(f)` where `!f.is_finite()` => `Err(DataError::ParseError(E-DAT-010))`
/// - `Float(f)` otherwise => `Value::Float(OrderedFloat(f64))`
/// - `Bool` => `Value::Bool`
/// - `String` => `Value::Str(Arc<str>)`
/// - `DurationIso` => `Value::Str` as-is
/// - `DateTimeIso(s)` => validated against chrono ISO 8601 parse; `Err` if invalid (E-DAT-009)
/// - `DateTime` => `Value::Str` in ISO 8601 format
/// - `Error(_)` => `Value::Null` (formula errors are nulled, not propagated)
///
/// Formula cells: `calamine::Data::Formula` is not present in calamine 0.26+;
/// formula cached values are represented by one of the above variants directly.
///
/// # Errors
///
/// Returns `Err(DataError)` for:
/// - Non-finite float cells (E-DAT-010)
/// - `DateTimeIso` cells with invalid ISO 8601 values (E-DAT-009)
///
/// Traces to BC-1.03.006 postconditions 4-7 (cell type mapping).
fn convert_calamine_cell(cell: &Data, col: u32, row: u32, path: &str) -> Result<Value, DataError> {
    match cell {
        Data::Int(n) => Ok(Value::Int(*n)),
        Data::Float(f) => {
            // BC-1.03.006 postcondition 5: whole-number Float promotion.
            // Kani-amenable (VP-021, VP-022, VP-023).
            if !f.is_finite() {
                // E-DAT-010: NaN or Infinity is not representable.
                return Err(DataError::parse_error(
                    path,
                    DataFormat::Xlsx,
                    format!(
                        "XLSX numeric cell at col {col} row {row} in '{path}' has non-finite value \
                        ({}). Non-finite floats are not representable in slideforge values.",
                        if f.is_nan() { "NaN" } else { "Infinity" }
                    ),
                ));
            }
            // Safe: f.is_finite() guaranteed above.
            // F-MED-1: Use round-trip check for exact representability instead of
            // `*f <= i64::MAX as f64`. The latter allows 2^63 (which f64 CAN represent
            // exactly as i64::MAX + 1) to pass the bounds check, after which
            // `*f as i64` silently saturates to i64::MAX (corruption).
            // Round-trip: `(*f as i64) as f64 == *f` confirms the value is exactly
            // representable as i64 without loss. Values in range but not exactly
            // representable (e.g. 2^63) stay as Value::Float.
            // Traces to VP-021 boundary, F-MED-1.
            // float_cmp: round-trip equality check is intentional — comparing exact
            // bit representation to detect lossless i64 round-trip (not approximate equality).
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::float_cmp
            )]
            if f.fract() == 0.0 && (*f as i64) as f64 == *f {
                Ok(Value::Int(*f as i64))
            } else {
                Ok(Value::Float(OrderedFloat(*f)))
            }
        },
        Data::String(s) | Data::DurationIso(s) => Ok(Value::Str(Arc::from(s.as_str()))),
        Data::DateTimeIso(s) => {
            // BC-1.03.006 postcondition 6 / invariant 9: validate ISO 8601.
            // Traces to VP-024 (valid) and VP-025 (invalid).
            // Try RFC 3339 first (e.g. "2024-01-15T09:00:00+00:00"), then
            // NaiveDateTime ("%Y-%m-%dT%H:%M:%S"), then NaiveDate ("%Y-%m-%d").
            let is_valid = chrono::DateTime::parse_from_rfc3339(s).is_ok()
                || chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").is_ok()
                || chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok();
            if is_valid {
                Ok(Value::Str(Arc::from(s.as_str())))
            } else {
                Err(DataError::parse_error(
                    path,
                    DataFormat::Xlsx,
                    format!(
                        "XLSX datetime cell at col {col} row {row} in '{path}' has invalid \
                        ISO 8601 value '{s}'. Expected format: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS\u{00b1}HH:MM"
                    ),
                ))
            }
        },
        Data::Bool(b) => Ok(Value::Bool(*b)),
        Data::DateTime(dt) => {
            // Convert calamine's ExcelDateTime to an ISO 8601 string.
            // as_datetime() requires the calamine "dates" feature (chrono integration).
            // We use chrono::Timelike to access nanoseconds for sub-second precision.
            if let Some(naive_dt) = dt.as_datetime() {
                use chrono::Timelike as _;
                // Use format string to preserve sub-second precision if present.
                // F-LOW-3: include sub-second precision when non-zero.
                let formatted = if naive_dt.nanosecond() > 0 {
                    naive_dt.format("%Y-%m-%dT%H:%M:%S%.f").to_string()
                } else {
                    naive_dt.format("%Y-%m-%dT%H:%M:%S").to_string()
                };
                // If time is 00:00:00, emit date-only format.
                let dt_str = if formatted.ends_with("T00:00:00") {
                    formatted[..10].to_owned()
                } else {
                    formatted
                };
                Ok(Value::Str(Arc::from(dt_str.as_str())))
            } else {
                // Fallback: raw float serial number as string (should not occur
                // in practice with well-formed xlsx files).
                Ok(Value::Str(Arc::from(format!("{dt}").as_str())))
            }
        },
        Data::Error(_) | Data::Empty => Ok(Value::Null),
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
        DataError::UnsupportedFormat { extension, .. } => DataSourceError::UnsupportedUri {
            // F-MED-2: include the offending extension and the supported format.
            // TD-VSDD-060: generic message covers all non-xlsx extensions, not just .xls.
            uri: format!("{path} (only .xlsx extension supported; got '.{extension}')"),
        },
        _ => DataSourceError::ParseError {
            uri: path.to_owned(),
            message: err.to_string(),
        },
    }
}

/// Validate that a file with `.xlsx` extension is actually a ZIP/XLSX archive.
///
/// XLSX files are ZIP archives — the local file header is `PK\x03\x04` (4 bytes).
/// A file with the correct extension but wrong magic bytes is rejected with
/// `DataError::ParseError` (E-DAT-011).
///
/// This is the second-phase check after extension validation. Combined with
/// extension check, it ensures: extension=.xlsx AND magic=ZIP.
///
/// ## TOCTOU note
///
/// We open the file here to read 4 bytes, then close it. `calamine::open_workbook`
/// then opens the file again. There is a residual TOCTOU window between these two
/// opens in which a file could be swapped — however, the trust model here is
/// local-trusted-file (the orchestrator enforces path containment before reaching
/// this point), so the risk is accepted and documented. A zero-cost TOCTOU
/// mitigation (single open via a shared file handle) would require calamine VFS
/// integration, which is out of scope for v1.0.
///
/// Traces to BC-1.03.006 postcondition 9, invariant 3 (VP-026), F-LOW-1.
fn validate_xlsx_magic(path: &str) -> Result<(), DataError> {
    use std::io::Read as _;

    use crate::error::E_DAT_011;

    // ZIP local file header magic: PK\x03\x04
    const ZIP_MAGIC: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];

    let mut buf = [0u8; 4];
    let mut file = std::fs::File::open(path)
        .map_err(|e| DataError::io_error(Arc::from(path), Arc::from(e.to_string().as_str())))?;
    let n = file
        .read(&mut buf)
        .map_err(|e| DataError::io_error(Arc::from(path), Arc::from(e.to_string().as_str())))?;

    if n < 4 || buf != ZIP_MAGIC {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("xlsx");
        return Err(DataError::ParseError {
            code: E_DAT_011,
            path: Arc::from(path),
            format: DataFormat::Xlsx,
            reason: Arc::from(
                format!(
                    "'{path}' has .{ext} extension but is not a valid XLSX archive \
                    (ZIP magic bytes not found). File may be corrupted or misnamed."
                )
                .as_str(),
            ),
            span: slideforge_types::SourceSpan::default(),
        });
    }
    Ok(())
}

/// Validate that the `.xlsx` extension is used; reject all other extensions.
///
/// Per BC-1.03.006 invariant 3 and AC-BC-005, this check fires BEFORE magic-byte
/// validation ("Do NOT read bytes" — extension check is the first gate).
///
/// - `.xlsx` (case-insensitive) → `Ok(())`
/// - `.xls` → `Err(UnsupportedFormat)` with actionable hint
/// - Any other extension → `Err(UnsupportedFormat)` with the offending extension named
///
/// Note: this function is named `reject_xls_extension` for historical reasons;
/// it now validates the full extension contract (only `.xlsx` is accepted).
///
/// Traces to BC-1.03.006 invariant 3, AC-BC-005, edge case EC-002 (AC-005), F-MED-2.
fn reject_xls_extension(path: &str) -> Result<(), DataError> {
    let p = std::path::Path::new(path);
    let ext_lower = p
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();

    if ext_lower == "xlsx" {
        return Ok(());
    }

    // All non-xlsx extensions are rejected with an informative message.
    // .xls gets a special "convert to .xlsx" hint; others get a generic note.
    let extension_arc = if ext_lower.is_empty() {
        Arc::from("(no extension)")
    } else {
        Arc::from(ext_lower.as_str())
    };
    Err(DataError::UnsupportedFormat {
        code: crate::error::E_DAT_003,
        extension: extension_arc,
        span: slideforge_types::SourceSpan::default(),
    })
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
            sheet_names.first().cloned().ok_or_else(|| {
                DataError::parse_error(path, DataFormat::Xlsx, "workbook has no sheets")
            })?
        },
        Some(name) => {
            // AC-004: named sheet must exist.
            if sheet_names.iter().any(|s| s == name) {
                name.to_owned()
            } else {
                let available = sheet_names.join(", ");
                return Err(DataError::parse_error(
                    path,
                    DataFormat::Xlsx,
                    format!(
                        "sheet '{name}' not found in '{path}'. Available sheets: [{available}]"
                    ),
                ));
            }
        },
    };

    let range = workbook.worksheet_range(&target_name).map_err(|e| {
        DataError::parse_error(
            path,
            DataFormat::Xlsx,
            format!("failed to read sheet '{target_name}': {e}"),
        )
    })?;

    Ok((target_name, range))
}

/// Extract and validate the header row from the range.
///
/// Returns a `Vec<Arc<str>>` of column names from row 0.
///
/// Errors:
/// - Empty sheet (no rows or no non-empty cells in row 0) → `ParseError` "empty sheet" (EC-007)
/// - Partial-empty header row (any cell `None`/`Empty` while others populated) → E-DAT-007 (AC-002)
/// - Non-string header cell (Int, Float, Bool, `DateTime`) → E-DAT-008 (F-HIGH-1)
///
/// Note: Merged cell detection in the header row is handled at the `load()` level
/// via `workbook.worksheet_merge_cells()` before this function is called.
///
/// Traces to BC-1.03.006 postconditions 2-3, invariants 5-6, AC-002.
fn extract_headers(range: &calamine::Range<Data>, path: &str) -> Result<Vec<Arc<str>>, DataError> {
    use crate::error::{E_DAT_007, E_DAT_008};

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

    // First pass: collect all cells and classify them to distinguish
    // "entirely empty" (EC-004) from "partially empty" (E-DAT-007).
    let cells: Vec<Option<&Data>> = (0..col_count)
        .map(|col_idx| {
            #[allow(clippy::cast_possible_truncation)]
            range.get_value((0, col_idx as u32))
        })
        .collect();

    let all_empty = cells.iter().all(|c| matches!(c, None | Some(Data::Empty)));

    // EC-004: entirely empty header row — "empty sheet" error.
    if all_empty {
        return Err(DataError::parse_error(
            path,
            DataFormat::Xlsx,
            "cannot load data from empty sheet",
        ));
    }

    // Second pass: validate each cell. Non-empty sheet with any empty header
    // cell is E-DAT-007 (partial-empty header). Non-string header is E-DAT-008.
    let mut headers: Vec<Arc<str>> = Vec::with_capacity(col_count);

    for (col_idx, cell) in cells.iter().enumerate() {
        match cell {
            None | Some(Data::Empty) => {
                // E-DAT-007: partial-empty header — reject with actionable message.
                // Traces to BC-1.03.006 invariant 5, postcondition 2.
                return Err(DataError::ParseError {
                    code: E_DAT_007,
                    path: Arc::from(path),
                    format: DataFormat::Xlsx,
                    reason: Arc::from(
                        format!(
                            "XLSX header row at '{path}' has empty cell at column {col_idx} (0-indexed). \
                            All header cells must be non-empty strings. Do not use blank column headers; \
                            remove unused columns or name all headers."
                        )
                        .as_str(),
                    ),
                    span: slideforge_types::SourceSpan::default(),
                });
            },
            Some(Data::String(s)) => {
                // F-LOW-1 adjudication (pass 3): headers are stored EXACTLY as authored.
                // Per CLAUDE.md "Forbidden Patterns" (DI-004, "strings are strings") and
                // the canonical "no implicit type coercion" rule, silently mutating header
                // column names with trim() is implicit transformation.
                //
                // A whitespace-only cell (e.g. "   ") is a non-empty string — it passes
                // the empty-cell check (None/Data::Empty above) and is stored as-is.
                // Users who want stripped headers must clean the XLSX before loading.
                //
                // This removes the previous trim() + whitespace-only error branch (F-LOW-2).
                // Traces to BC-1.03.006 invariant 5 (AC-BC-001).
                headers.push(Arc::from(s.as_str()));
            },
            Some(non_string_cell) => {
                // E-DAT-008: non-string header cell — Int, Float, Bool, DateTime etc.
                // Traces to BC-1.03.006 invariant 6, postcondition 2.
                let type_name = match non_string_cell {
                    Data::Int(_) => "Integer",
                    Data::Float(_) => "Float",
                    Data::Bool(_) => "Boolean",
                    Data::DateTime(_) => "DateTime",
                    Data::DateTimeIso(_) => "DateTimeIso",
                    Data::DurationIso(_) => "DurationIso",
                    Data::Error(_) => "Error",
                    Data::Empty | Data::String(_) => unreachable!(),
                };
                let repr = format!("{non_string_cell}");
                return Err(DataError::ParseError {
                    code: E_DAT_008,
                    path: Arc::from(path),
                    format: DataFormat::Xlsx,
                    reason: Arc::from(
                        format!(
                            "XLSX header cell at column {col_idx} in '{path}' has type {type_name} \
                            (value: {repr}). Header cells must be String-typed. Use a string label \
                            as the column header."
                        )
                        .as_str(),
                    ),
                    span: slideforge_types::SourceSpan::default(),
                });
            },
        }
    }

    Ok(headers)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use calamine::Data;
    use rust_xlsxwriter::{Formula, Workbook};
    use slideforge_plugin_api::{DataSource, DataSourceOptions};
    use slideforge_types::Value;

    use super::{XlsxDataSource, convert_calamine_cell, extract_headers};

    // ---------------------------------------------------------------------------
    // Helper: write an xlsx bytes buffer to a tempfile and return the path.
    // ---------------------------------------------------------------------------

    fn write_xlsx_to_tempfile(
        buf: Vec<u8>,
        suffix: &str,
    ) -> (tempfile::TempDir, std::path::PathBuf) {
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
        // After F-HIGH-4 (VP-021): whole-number floats are promoted to Int.
        // calamine reads 95_f64 as Data::Float(95.0) which must become Value::Int(95).
        let score0 = row0_map.get("score").unwrap();
        assert_eq!(
            score0,
            &Value::Int(95),
            "row 0 'score' must be Int(95) after whole-number Float promotion (VP-021); got {score0:?}"
        );

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
        assert_eq!(
            score1,
            &Value::Int(87),
            "row 1 'score' must be Int(87) after whole-number Float promotion (VP-021); got {score1:?}"
        );
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
        ws.write_datetime_with_format(1, 1, &excel_date, &date_fmt)
            .unwrap();

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
                    "float cell must be approximately 1.5, got {}",
                    f.0
                );
            },
            Value::Int(_) => {
                panic!("1.5 must NOT become an Int; must be Value::Float")
            },
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
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
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
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::UnsupportedUri { .. }
            ),
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
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::UnsupportedUri { .. }
            ),
            "`.XLS` (uppercase) must also be rejected; got: {err:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // F-LOW-1 adjudication: headers stored EXACTLY as authored (no trim).
    // BC-1.03.006 invariant 5 (AC-BC-001). Per DI-004 / "strings are strings".
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_whitespace_only_header_stored_as_is` -- whitespace-only
    /// headers are stored exactly as authored (not rejected, not trimmed).
    ///
    /// F-LOW-1 adjudication (pass 3): the previous trim()-based implementation was
    /// classified as implicit type coercion, forbidden by CLAUDE.md DI-004 and the
    /// canonical "strings are strings" rule. Headers must be stored byte-for-byte.
    ///
    /// A whitespace-only cell `"   "` is a non-empty string. It passes the empty-cell
    /// check (`None`/`Data::Empty`) and is stored as `"   "` (three spaces). Users who
    /// want stripped headers must clean the XLSX before loading.
    ///
    /// This test FAILS if `trim()` is reintroduced (the column key would be `""` after
    /// trimming, causing key mismatch) — load-bearing per TD-VSDD-059.
    ///
    /// Traces to BC-1.03.006 invariant 5 (AC-BC-001), F-LOW-1.
    #[test]
    fn test_bc_1_03_006_xlsx_whitespace_only_header_stored_as_is() {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();

        // Header row: "name", "   " (whitespace-only), "score".
        ws.write_string(0, 0, "name").unwrap();
        ws.write_string(0, 1, "   ").unwrap(); // whitespace-only — must be stored as "   "
        ws.write_string(0, 2, "score").unwrap();
        ws.write_string(1, 0, "Alice").unwrap();
        ws.write_string(1, 1, "placeholder").unwrap(); // value for the whitespace-key column
        ws.write_string(1, 2, "95").unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let result = src
            .load("", &default_opts())
            .expect("whitespace-only header must be accepted and stored as-is (F-LOW-1)");

        let rows = match result {
            slideforge_types::Value::List(r) => r,
            other => panic!("load must return Value::List, got: {other:?}"),
        };
        assert_eq!(rows.len(), 1, "must load 1 data row");

        // The key must be the raw string "   " (three spaces), NOT "" or "score".
        let row = match &rows[0] {
            slideforge_types::Value::Map(m) => m,
            other => panic!("row must be Value::Map, got: {other:?}"),
        };
        assert!(
            row.contains_key("   "),
            "row must contain key '   ' (whitespace-only, exactly as authored); \
            keys present: {:?}",
            row.keys().collect::<Vec<_>>()
        );
        // Bonus: confirm "name" and "score" keys are also stored exactly.
        assert!(row.contains_key("name"), "row must contain key 'name'");
        assert!(row.contains_key("score"), "row must contain key 'score'");
    }

    /// `test_bc_1_03_006_xlsx_header_with_surrounding_spaces_stored_as_is` -- a header
    /// like `" Score "` (spaces around content) is stored byte-for-byte, not stripped.
    ///
    /// Complements `test_bc_1_03_006_xlsx_whitespace_only_header_stored_as_is`.
    /// Confirms that F-LOW-1 removal of `trim()` applies to mixed-whitespace headers
    /// as well as whitespace-only cells.
    ///
    /// This test FAILS if `trim()` is reintroduced — the column key would be `"Score"`
    /// instead of `" Score "`, causing the `row.contains_key(" Score ")` assertion to fail.
    ///
    /// Traces to BC-1.03.006 invariant 5 (AC-BC-001), F-LOW-1.
    #[test]
    fn test_bc_1_03_006_xlsx_header_with_surrounding_spaces_stored_as_is() {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();

        ws.write_string(0, 0, "id").unwrap();
        ws.write_string(0, 1, " Score ").unwrap(); // leading + trailing spaces
        ws.write_string(1, 0, "1").unwrap();
        ws.write_string(1, 1, "42").unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let result = src
            .load("", &default_opts())
            .expect("header with surrounding spaces must be accepted and stored as-is");

        let rows = match result {
            slideforge_types::Value::List(r) => r,
            other => panic!("load must return Value::List, got: {other:?}"),
        };
        assert_eq!(rows.len(), 1, "must load 1 data row");

        let row = match &rows[0] {
            slideforge_types::Value::Map(m) => m,
            other => panic!("row must be Value::Map, got: {other:?}"),
        };
        // Key must be " Score " (with spaces), not "Score".
        assert!(
            row.contains_key(" Score "),
            "row must contain key ' Score ' (spaces preserved); keys: {:?}",
            row.keys().collect::<Vec<_>>()
        );
        assert!(
            !row.contains_key("Score"),
            "trimmed key 'Score' must NOT appear — trim() was removed (F-LOW-1)"
        );
    }

    // ---------------------------------------------------------------------------
    // F-MED-2 / AC-BC-005: non-xlsx extension rejected (e.g. .txt, .foo, .csv).
    // BC-1.03.006 invariant 3: extension check fires BEFORE magic-byte check.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_unsupported_extension_rejected` -- non-`.xlsx` extensions are rejected.
    ///
    /// Per BC-1.03.006 AC-BC-005 ("Do NOT read bytes"), the extension check fires
    /// BEFORE magic-byte validation. Files with `.txt`, `.foo`, or other non-xlsx
    /// extensions must produce `UnsupportedUri` with the offending extension named.
    ///
    /// This test FAILS if the extension check is reverted to "only reject .xls" —
    /// making it load-bearing per TD-VSDD-059.
    ///
    /// Traces to BC-1.03.006 invariant 3, AC-BC-005, F-MED-2.
    #[test]
    fn test_bc_1_03_006_unsupported_extension_rejected() {
        // .txt extension: no file needs to exist (extension check fires first).
        let src_txt = XlsxDataSource::new("/tmp/data.txt");
        let err_txt = src_txt.load("", &default_opts()).unwrap_err();
        assert!(
            matches!(
                err_txt,
                slideforge_plugin_api::DataSourceError::UnsupportedUri { .. }
            ),
            "`.txt` must produce UnsupportedUri; got: {err_txt:?}"
        );
        let msg_txt = err_txt.to_string();
        assert!(
            msg_txt.contains("txt") || msg_txt.to_lowercase().contains("only .xlsx"),
            "error must name the offending extension '.txt'; got: {msg_txt}"
        );

        // .foo extension:
        let src_foo = XlsxDataSource::new("/tmp/data.foo");
        let err_foo = src_foo.load("", &default_opts()).unwrap_err();
        assert!(
            matches!(
                err_foo,
                slideforge_plugin_api::DataSourceError::UnsupportedUri { .. }
            ),
            "`.foo` must produce UnsupportedUri; got: {err_foo:?}"
        );
        let msg_foo = err_foo.to_string();
        assert!(
            msg_foo.contains("foo") || msg_foo.to_lowercase().contains("only .xlsx"),
            "error must name the offending extension '.foo'; got: {msg_foo}"
        );

        // .csv extension (common mistake):
        let src_csv = XlsxDataSource::new("/tmp/data.csv");
        let err_csv = src_csv.load("", &default_opts()).unwrap_err();
        assert!(
            matches!(
                err_csv,
                slideforge_plugin_api::DataSourceError::UnsupportedUri { .. }
            ),
            "`.csv` must produce UnsupportedUri; got: {err_csv:?}"
        );
        let msg_csv = err_csv.to_string();
        assert!(
            msg_csv.contains("csv") || msg_csv.to_lowercase().contains("only .xlsx"),
            "error must name the offending extension '.csv'; got: {msg_csv}"
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
        ws.merge_range(0, 0, 0, 1, "MergedHeader", &merge_fmt)
            .unwrap();
        // Data row.
        ws.write_string(1, 0, "val1").unwrap();
        ws.write_string(1, 1, "val2").unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let err = src.load("", &default_opts()).unwrap_err();

        let msg = err.to_string();
        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
            "merged header cells must produce DataSourceError::ParseError, got: {err:?}"
        );
        assert!(
            msg.to_lowercase().contains("merged"),
            "error message must mention 'merged'; got: {msg}"
        );
    }

    /// `test_bc_1_03_006_xlsx_vertical_merge_in_header` -- vertical merge spanning header into data rows returns error.
    ///
    /// A cell that spans from row 0 (header) into row 1+ creates an ambiguous
    /// column mapping. Must produce `DataSourceError::ParseError` (F-LOW-9, EC-005).
    ///
    /// Traces to BC-1.03.006 AC-006, edge case EC-005, F-LOW-9.
    #[test]
    fn test_bc_1_03_006_xlsx_vertical_merge_in_header() {
        use rust_xlsxwriter::Format;

        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();

        // Vertical merge: A1:A2 spans header row (row 0) into first data row (row 1).
        // This creates a merged cell that spans from the header into a data row.
        let merge_fmt = Format::new();
        ws.merge_range(0, 0, 1, 0, "VertMergedHeader", &merge_fmt)
            .unwrap();
        // Second column with normal header.
        ws.write_string(0, 1, "value").unwrap();
        // Second data row to ensure the sheet has some data.
        ws.write_string(2, 0, "data_a").unwrap();
        ws.write_string(2, 1, "data_b").unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let err = src.load("", &default_opts()).unwrap_err();

        let msg = err.to_string();
        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
            "vertical merge in header must produce DataSourceError::ParseError, got: {err:?}"
        );
        assert!(
            msg.to_lowercase().contains("vertical") || msg.to_lowercase().contains("merged"),
            "error message must mention 'vertical' or 'merged'; got: {msg}"
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
        ws.write_formula(1, 0, Formula::new("=2+3").set_result("5"))
            .unwrap();

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
    // Two distinct "empty" sub-paths in extract_headers (F-LOW-4):
    //   Path A — row_count == 0: worksheet has zero rows (struct is empty).
    //   Path B — all_empty: worksheet has rows but every header cell is Data::Empty.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_empty_header_row` -- zero-row sheet (path A) returns `ParseError`.
    ///
    /// Exercises the `row_count == 0` branch in `extract_headers`. Writing nothing
    /// to a worksheet causes calamine to return a range with zero rows.
    ///
    /// Error message must contain "empty" (BC-1.03.006 AC-002, EC-004, F-LOW-4 path A).
    #[test]
    fn test_bc_1_03_006_xlsx_empty_header_row() {
        let mut wb = Workbook::new();
        // Add a worksheet but write nothing — completely empty (row_count == 0).
        let _ws = wb.add_worksheet();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let err = src.load("", &default_opts()).unwrap_err();

        let msg = err.to_string();
        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
            "zero-row sheet must produce DataSourceError::ParseError, got: {err:?}"
        );
        assert!(
            msg.to_lowercase().contains("empty"),
            "error message must mention 'empty'; got: {msg}"
        );
    }

    /// `test_bc_1_03_006_xlsx_all_empty_header_cells` -- all-empty header cells (path B) returns `ParseError`.
    ///
    /// Exercises the `all_empty` branch in `extract_headers`: a range with non-zero
    /// dimensions where every header cell is `Data::Empty`. This is distinct from the
    /// zero-row path above — the sheet has structure but every header cell is empty.
    ///
    /// Error message must contain "empty" (BC-1.03.006 AC-002, EC-004, F-LOW-4 path B).
    #[test]
    fn test_bc_1_03_006_xlsx_all_empty_header_cells() {
        use calamine::Range;
        // Construct a Range with 2 rows × 2 cols, all cells defaulting to Data::Empty.
        // Range::new creates a range with the given dimensions but no data — every
        // call to get_value returns None (calamine's representation of an empty cell).
        let empty_range: Range<Data> = Range::new((0, 0), (1, 1));
        let err = extract_headers(&empty_range, "test_all_empty.xlsx")
            .expect_err("all-empty header cells must produce DataError");
        let msg = err.to_string();
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
        assert_eq!(
            rows.len(),
            3,
            "canonical test vector requires exactly 3 data rows"
        );

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
        let result = convert_calamine_cell(&cell, 0, 1, "test.xlsx").unwrap();
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
        let result = convert_calamine_cell(&cell, 0, 1, "test.xlsx").unwrap();
        assert_eq!(
            result,
            Value::Int(-99),
            "Data::Int(-99) must map to Value::Int(-99)"
        );
    }

    /// `test_bc_1_03_006_convert_calamine_cell_float` -- `Data::Float(1.5)` maps to `Value::Float`.
    ///
    /// VP-022: fractional floats stay as Float.
    ///
    /// Traces to BC-1.03.006 postcondition 5.
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_float() {
        let cell = Data::Float(1.5);
        let result = convert_calamine_cell(&cell, 0, 1, "test.xlsx").unwrap();
        match result {
            Value::Float(f) => {
                assert!(
                    (f.0 - 1.5_f64).abs() < 1e-10,
                    "Data::Float(1.5) must map to Value::Float(1.5)"
                );
            },
            other => panic!("Data::Float(1.5) must produce Value::Float, got {other:?}"),
        }
    }

    /// `test_bc_1_03_006_convert_calamine_cell_bool` -- `Data::Bool` maps to `Value::Bool`.
    ///
    /// Traces to BC-1.03.006 postcondition 5 (DI-004 no-coercion: bool stays bool).
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_bool() {
        assert_eq!(
            convert_calamine_cell(&Data::Bool(true), 0, 1, "test.xlsx").unwrap(),
            Value::Bool(true),
            "Data::Bool(true) must map to Value::Bool(true)"
        );
        assert_eq!(
            convert_calamine_cell(&Data::Bool(false), 0, 1, "test.xlsx").unwrap(),
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
        let result = convert_calamine_cell(&cell, 0, 1, "test.xlsx").unwrap();
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
        let result = convert_calamine_cell(&cell, 0, 1, "test.xlsx").unwrap();
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
        let result = convert_calamine_cell(&cell, 0, 1, "test.xlsx").unwrap();
        assert_eq!(
            result,
            Value::Null,
            "Data::Error (formula errors like #DIV/0!) must produce Value::Null"
        );
    }

    /// `test_bc_1_03_006_convert_calamine_cell_datetime_iso_str` -- valid `Data::DateTimeIso` maps to `Value::Str`.
    ///
    /// VP-024: valid `DateTimeIso` strings pass through as `Value::Str`.
    ///
    /// Traces to BC-1.03.006 postcondition 6.
    #[test]
    fn test_bc_1_03_006_convert_calamine_cell_datetime_iso_str() {
        let cell = Data::DateTimeIso("2024-01-15T09:00:00".to_owned());
        let result = convert_calamine_cell(&cell, 0, 1, "test.xlsx").unwrap();
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
        let result = convert_calamine_cell(&cell, 0, 1, "test.xlsx").unwrap();
        assert_eq!(
            result,
            Value::Str(Arc::from("PT2H30M")),
            "Data::DurationIso must produce Value::Str"
        );
    }

    // ---------------------------------------------------------------------------
    // VP-021: Float(95.0) → Int(95) — whole-number Float promotion.
    // BC-1.03.006 postcondition 5, invariant 7 (Kani-amenable).
    // ---------------------------------------------------------------------------

    /// `test_vp_021_float_whole_number_promotes_to_int` -- VP-021: `Data::Float(95.0)` → `Value::Int(95)`.
    ///
    /// Whole-number floats that fit in i64 are promoted to Int (not Float).
    /// This is the canonical VP-021 test vector (Kani-amenable).
    ///
    /// Traces to BC-1.03.006 postcondition 5, invariant 7.
    #[test]
    fn test_vp_021_float_whole_number_promotes_to_int() {
        let cell = Data::Float(95.0);
        let result = convert_calamine_cell(&cell, 1, 1, "test.xlsx").unwrap();
        assert_eq!(
            result,
            Value::Int(95),
            "Data::Float(95.0) must promote to Value::Int(95) (VP-021)"
        );

        // Negative whole number
        let cell_neg = Data::Float(-42.0);
        let result_neg = convert_calamine_cell(&cell_neg, 0, 1, "test.xlsx").unwrap();
        assert_eq!(
            result_neg,
            Value::Int(-42),
            "Float(-42.0) must promote to Int(-42)"
        );

        // Zero
        let cell_zero = Data::Float(0.0);
        let result_zero = convert_calamine_cell(&cell_zero, 0, 1, "test.xlsx").unwrap();
        assert_eq!(
            result_zero,
            Value::Int(0),
            "Float(0.0) must promote to Int(0)"
        );
    }

    // ---------------------------------------------------------------------------
    // VP-021 boundary: Float(2^63) must NOT promote to Int (out-of-range whole).
    // F-MED-1 regression test.
    // BC-1.03.006 postcondition 5.
    // ---------------------------------------------------------------------------

    /// `test_vp_021_boundary_round_trip` -- VP-021 boundary: round-trip check rejects non-exact promotions.
    ///
    /// The round-trip check `(*f as i64) as f64 == *f` guarantees that only f64 values
    /// that round-trip exactly through i64 are promoted. This test exercises the boundary:
    ///
    /// - `i64::MAX as f64` (= 2^63 in f64 due to rounding): round-trips to itself →
    ///   promoted to `Value::Int(i64::MAX)`. This is the correct behavior because the f64
    ///   value IS `i64::MAX` when interpreted as an integer (f64 has insufficient precision
    ///   to distinguish `i64::MAX` from `2^63`; they share the same bit pattern).
    /// - Large fractional-part float values that do NOT round-trip: stay as `Value::Float`.
    /// - The key invariant: `Value::Int(n)` is only produced when `n as f64 == *f` exactly.
    ///
    /// This test FAILS if the round-trip check is removed — making it load-bearing
    /// per TD-VSDD-059.
    ///
    /// Traces to BC-1.03.006 postcondition 5, F-MED-1.
    // float_cmp: intentional exact bitwise equality checks for round-trip validation.
    // cast_precision_loss + cast_possible_truncation: intentional — testing the
    // same cast used in production code.
    #[allow(
        clippy::float_cmp,
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation
    )]
    #[test]
    fn test_vp_021_boundary_round_trip() {
        // Case 1: f64 value that has fract() == 0.0 but does NOT round-trip exactly.
        // 1.5e20 is a whole-number float > i64::MAX — its round-trip through i64 saturates
        // and does NOT reproduce the original float, so it stays as Value::Float.
        let large_whole: f64 = 1.5e20_f64;
        assert_eq!(
            large_whole.fract(),
            0.0,
            "precondition: 1.5e20 has fract() == 0.0"
        );
        assert!(large_whole.is_finite(), "precondition: 1.5e20 is finite");
        // Confirm round-trip FAILS:
        assert_ne!(
            (large_whole as i64) as f64,
            large_whole,
            "1.5e20 must not round-trip through i64"
        );
        let cell_large = calamine::Data::Float(large_whole);
        let result_large = convert_calamine_cell(&cell_large, 0, 1, "test.xlsx").unwrap();
        match result_large {
            Value::Float(_) => {},
            Value::Int(n) => panic!(
                "Data::Float(1.5e20) must stay as Value::Float (too large for i64); \
                got Value::Int({n})"
            ),
            other => panic!("unexpected: {other:?}"),
        }

        // Case 2: f64 value that round-trips exactly through i64 — promoted.
        // 1_000_000.0 round-trips: (1_000_000 as i64) as f64 = 1_000_000.0.
        let cell_ok = calamine::Data::Float(1_000_000.0_f64);
        let result_ok = convert_calamine_cell(&cell_ok, 0, 1, "test.xlsx").unwrap();
        assert_eq!(
            result_ok,
            Value::Int(1_000_000),
            "Data::Float(1_000_000.0) must promote to Value::Int(1_000_000)"
        );

        // Case 3: i64::MIN boundary — promoted (i64::MIN is exactly representable).
        let min_f64: f64 = i64::MIN as f64;
        assert_eq!(
            (min_f64 as i64) as f64,
            min_f64,
            "i64::MIN round-trips exactly"
        );
        let cell_min = calamine::Data::Float(min_f64);
        let result_min = convert_calamine_cell(&cell_min, 0, 1, "test.xlsx").unwrap();
        assert_eq!(
            result_min,
            Value::Int(i64::MIN),
            "Data::Float(i64::MIN as f64) must promote to Value::Int(i64::MIN)"
        );
    }

    // ---------------------------------------------------------------------------
    // VP-022: Float(1.5) → Float (fractional, stays Float).
    // Uses 1.5 (exact in IEEE 754) rather than 3.14 (approx of π) to avoid
    // clippy::approx_constant false positive.
    // BC-1.03.006 postcondition 5.
    // ---------------------------------------------------------------------------

    /// `test_vp_022_float_fractional_stays_float` -- VP-022: `Data::Float(1.5)` stays `Value::Float`.
    ///
    /// Fractional floats must NOT be promoted to Int.
    /// 1.5 is used because it is exactly representable in IEEE 754 and is not
    /// an approximation of any mathematical constant (avoiding `clippy::approx_constant`).
    ///
    /// Traces to BC-1.03.006 postcondition 5.
    #[test]
    fn test_vp_022_float_fractional_stays_float() {
        let cell = Data::Float(1.5);
        let result = convert_calamine_cell(&cell, 0, 1, "test.xlsx").unwrap();
        match result {
            Value::Float(f) => {
                assert!(
                    (f.0 - 1.5).abs() < 1e-10,
                    "Float(1.5) must be Value::Float(1.5)"
                );
            },
            other => panic!("Float(1.5) must NOT be promoted to Int; got {other:?}"),
        }
    }

    // ---------------------------------------------------------------------------
    // VP-023: NaN/Infinity → E-DAT-010 (Kani-amenable).
    // BC-1.03.006 postcondition 5.
    // ---------------------------------------------------------------------------

    /// `test_vp_023_nan_produces_parse_error` -- VP-023: `Data::Float(NaN)` produces E-DAT-010.
    ///
    /// NaN and Infinity are not representable in slideforge Values and must error.
    /// Kani-amenable (pure function, finite/infinite distinction).
    ///
    /// Traces to BC-1.03.006 postcondition 5, E-DAT-010.
    #[test]
    fn test_vp_023_nan_produces_parse_error() {
        let cell = Data::Float(f64::NAN);
        let result = convert_calamine_cell(&cell, 2, 3, "test.xlsx");
        assert!(
            result.is_err(),
            "Data::Float(NaN) must produce Err (VP-023)"
        );
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("NaN") || msg.contains("non-finite"),
            "NaN error must mention NaN or non-finite; got: {msg}"
        );
    }

    /// `test_vp_023_infinity_produces_parse_error` -- VP-023: `Data::Float(INFINITY)` produces E-DAT-010.
    ///
    /// Traces to BC-1.03.006 postcondition 5, E-DAT-010.
    #[test]
    fn test_vp_023_infinity_produces_parse_error() {
        let cell = Data::Float(f64::INFINITY);
        let result = convert_calamine_cell(&cell, 0, 1, "test.xlsx");
        assert!(
            result.is_err(),
            "Data::Float(Infinity) must produce Err (VP-023)"
        );
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Infinity") || msg.contains("non-finite"),
            "Infinity error must mention Infinity or non-finite; got: {msg}"
        );
    }

    // ---------------------------------------------------------------------------
    // VP-024: valid `DateTimeIso` → `Value::Str`.
    // BC-1.03.006 postcondition 6.
    // ---------------------------------------------------------------------------

    /// `test_vp_024_valid_datetime_iso_passes_through` -- VP-024: valid `DateTimeIso` → `Value::Str`.
    ///
    /// Tests multiple valid ISO 8601 formats.
    ///
    /// Traces to BC-1.03.006 postcondition 6.
    #[test]
    fn test_vp_024_valid_datetime_iso_passes_through() {
        // RFC 3339 format
        let cell = Data::DateTimeIso("2024-01-15T09:00:00+00:00".to_owned());
        let result = convert_calamine_cell(&cell, 0, 1, "test.xlsx").unwrap();
        assert!(
            matches!(result, Value::Str(_)),
            "RFC 3339 DateTimeIso must produce Str"
        );

        // Date-only format
        let cell2 = Data::DateTimeIso("2024-01-15".to_owned());
        let result2 = convert_calamine_cell(&cell2, 0, 1, "test.xlsx").unwrap();
        assert!(
            matches!(result2, Value::Str(_)),
            "Date-only DateTimeIso must produce Str"
        );
    }

    // ---------------------------------------------------------------------------
    // VP-025: invalid `DateTimeIso` → E-DAT-009.
    // BC-1.03.006 postcondition 6, F-HIGH-5.
    // ---------------------------------------------------------------------------

    /// `test_vp_025_invalid_datetime_iso_produces_error` -- VP-025: invalid `DateTimeIso` → E-DAT-009.
    ///
    /// Calamine emits `Data::DateTimeIso("not-iso-string")` — must produce `ParseError`.
    ///
    /// Traces to BC-1.03.006 postcondition 6, F-HIGH-5.
    #[test]
    fn test_vp_025_invalid_datetime_iso_produces_error() {
        let cell = Data::DateTimeIso("not-iso-string".to_owned());
        let result = convert_calamine_cell(&cell, 3, 7, "data.xlsx");
        assert!(
            result.is_err(),
            "Data::DateTimeIso(\"not-iso-string\") must produce Err (VP-025)"
        );
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("not-iso-string") || msg.contains("invalid") || msg.contains("ISO 8601"),
            "invalid DateTimeIso error must describe the bad value; got: {msg}"
        );
    }

    // ---------------------------------------------------------------------------
    // VP-019: partial-empty header → E-DAT-007.
    // BC-1.03.006 invariant 5, F-HIGH-2.
    // ---------------------------------------------------------------------------

    /// `test_vp_019_partial_empty_header_produces_e_dat_007` -- VP-019: partial-empty header row is rejected.
    ///
    /// A header row where some cells are String and at least one is empty must return E-DAT-007.
    /// NO phantom "__empty_<idx>" names may be invented.
    ///
    /// Traces to BC-1.03.006 invariant 5, postcondition 2.
    #[test]
    fn test_vp_019_partial_empty_header_produces_e_dat_007() {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();

        // "name", <empty>, "score" — partial-empty header
        ws.write_string(0, 0, "name").unwrap();
        // col 1: intentionally left blank (empty header)
        ws.write_string(0, 2, "score").unwrap();
        ws.write_string(1, 0, "Alice").unwrap();
        ws.write_string(1, 2, "95").unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let err = src.load("", &default_opts()).unwrap_err();

        let msg = err.to_string();
        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
            "partial-empty header must produce DataSourceError::ParseError, got: {err:?}"
        );
        assert!(
            msg.contains("E-DAT-007") || msg.contains("empty cell") || msg.contains("empty"),
            "partial-empty header error must mention E-DAT-007 or empty cell; got: {msg}"
        );
        // Must NOT contain __empty phantom names (confirmed by absence of "phantom" or "__empty")
        assert!(
            !msg.contains("__empty"),
            "error must NOT mention __empty phantom column names; got: {msg}"
        );
    }

    // ---------------------------------------------------------------------------
    // VP-020: non-string header cell → E-DAT-008.
    // BC-1.03.006 invariant 6, F-HIGH-1.
    // ---------------------------------------------------------------------------

    /// `test_vp_020_non_string_header_produces_e_dat_008` -- VP-020: non-string header cell is rejected.
    ///
    /// An integer header cell must produce E-DAT-008 naming the column index and type.
    ///
    /// Traces to BC-1.03.006 invariant 6, postcondition 2.
    #[test]
    fn test_vp_020_non_string_header_produces_e_dat_008() {
        // We test this via the extract_headers function directly with a calamine Range
        // that has a non-string header cell. We build an XLSX with a numeric header.
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();

        // Write an integer as the header of col 0 (non-string header)
        ws.write_number(0, 0, 42.0).unwrap();
        ws.write_string(0, 1, "score").unwrap();
        ws.write_string(1, 0, "Alice").unwrap();
        ws.write_number(1, 1, 95.0).unwrap();

        let buf = wb.save_to_buffer().unwrap();
        let (_dir, path) = write_xlsx_to_tempfile(buf, ".xlsx");

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let err = src.load("", &default_opts()).unwrap_err();

        let msg = err.to_string();
        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
            "non-string header must produce DataSourceError::ParseError, got: {err:?}"
        );
        assert!(
            msg.contains("E-DAT-008") || msg.contains("type") || msg.contains("header"),
            "non-string header error must mention E-DAT-008 or type; got: {msg}"
        );
    }

    // ---------------------------------------------------------------------------
    // VP-026: wrong magic bytes → ParseError E-DAT-011.
    // BC-1.03.006 postcondition 9, invariant 3.
    // ---------------------------------------------------------------------------

    /// `test_vp_026_wrong_magic_bytes_produces_e_dat_011` -- VP-026: `.xlsx` file with non-ZIP magic is rejected.
    ///
    /// A file with `.xlsx` extension but non-ZIP content must produce `ParseError` (E-DAT-011).
    ///
    /// Traces to BC-1.03.006 postcondition 9, invariant 3.
    #[test]
    fn test_vp_026_wrong_magic_bytes_produces_e_dat_011() {
        // Write a file with .xlsx extension but non-ZIP content.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fake.xlsx");
        // Not a ZIP: first 4 bytes are not PK\x03\x04.
        std::fs::write(&path, b"not a zip file at all").unwrap();

        let src = XlsxDataSource::new(path.to_str().unwrap());
        let err = src.load("", &default_opts()).unwrap_err();

        let msg = err.to_string();
        assert!(
            matches!(
                err,
                slideforge_plugin_api::DataSourceError::ParseError { .. }
            ),
            "wrong magic bytes must produce DataSourceError::ParseError, got: {err:?}"
        );
        // F-MED-P5-3: strict bracket form "[E-DAT-011]" required (TD-VSDD-059 load-bearing).
        // The DataError::ParseError Display format is "[{code}] parse error for ...".
        // Passing via substring alone (e.g., "magic" or "ZIP") does NOT catch a regression
        // where the code embedding is removed.  This assertion fails if `code: E_DAT_011` is
        // stripped from the ParseError variant, catching the sibling-drift at source.
        assert!(
            msg.contains("[E-DAT-011]"),
            "wrong magic bytes error must embed '[E-DAT-011]' bracket code in message; got: {msg}"
        );
    }

    // ---------------------------------------------------------------------------
    // NFR-036: large-sheet load performance — loading a sheet with many rows
    // must complete within the time gate specified by NFR-036 (< 2,000ms for
    // 10,000 rows × 50 cols). This smoke test exercises the bulk-loading path
    // with 1,000 rows to verify correctness; the full perf gate runs via
    // `cargo bench -p slideforge-data -- xlsx_10k_rows` in CI.
    // ---------------------------------------------------------------------------

    /// `test_bc_1_03_006_xlsx_large_sheet_loads` -- a sheet with many rows loads without panic.
    ///
    /// Writes 1000 data rows to verify the load path handles volume correctly.
    /// This is a correctness smoke test for the bulk-loading path (NFR-036).
    ///
    /// Traces to BC-1.03.006, NFR-036.
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
            },
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
            },
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
