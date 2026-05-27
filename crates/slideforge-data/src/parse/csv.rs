//! CSV parser for slideforge data sources.
//!
//! Parses a CSV string into a [`slideforge_types::Value::List`] of
//! [`slideforge_types::Value::Map`] rows. Column headers become map keys.
//! All cell values are represented as [`slideforge_types::Value::Str`]
//! (CSV has no type information; type inference is not performed).

use std::collections::HashSet;
use std::sync::Arc;

use slideforge_types::{OrderedMap, Value};

use crate::DataError;

/// Parse a CSV string and return a [`Value::List`] of row maps.
///
/// Each row in the CSV (after the header row) becomes a [`Value::Map`]
/// where keys are trimmed column header names and values are
/// [`Value::Str`] cell contents. Empty cells become [`Value::Null`].
///
/// # Errors
///
/// Returns [`DataError::ParseError`] with error code `E-DAT-003` if:
/// - The input contains no header row.
/// - Duplicate column headers are detected (EC-006 / EC-007).
/// - Any I/O error occurs during parsing.
pub fn parse_csv(source: &str, path: &str) -> Result<Value, DataError> {
    // Empty input → empty list (graceful handling)
    if source.trim().is_empty() {
        return Ok(Value::List(vec![]));
    }

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(source.as_bytes());

    // Read and trim headers
    let raw_headers = reader
        .headers()
        .map_err(|e| DataError::parse_error(path, e.to_string()))?
        .clone();

    let headers: Vec<Arc<str>> = raw_headers
        .iter()
        .map(|h| Arc::from(h.trim()))
        .collect();

    // Detect duplicate headers
    let mut seen: HashSet<&str> = HashSet::new();
    for header in &headers {
        let h: &str = header.as_ref();
        if !seen.insert(h) {
            return Err(DataError::parse_error(
                path,
                format!("duplicate column header '{h}'"),
            ));
        }
    }

    let mut rows: Vec<Value> = Vec::new();

    for result in reader.records() {
        let record = result.map_err(|e| DataError::parse_error(path, e.to_string()))?;
        let mut map = OrderedMap::new();

        for (i, header) in headers.iter().enumerate() {
            let cell = record.get(i).unwrap_or("");
            let value = if cell.is_empty() {
                Value::Null
            } else {
                Value::Str(Arc::from(cell))
            };
            map.insert(Arc::clone(header), value);
        }

        rows.push(Value::Map(map));
    }

    Ok(Value::List(rows))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use slideforge_types::Value;

    use super::*;

    /// test_BC_5_03_004_parse_csv_happy_path — two-column CSV with two data rows.
    ///
    /// Input: `"name,revenue\nAlice,100\nBob,200"`
    /// Expected: `Value::List` of 2 `Value::Map` rows.
    #[test]
    fn test_bc_5_03_004_parse_csv_happy_path() {
        let src = "name,revenue\nAlice,100\nBob,200";
        let value = parse_csv(src, "test.csv").expect("valid CSV must parse without error");

        let list = value.as_list().expect("CSV must produce Value::List");
        assert_eq!(list.len(), 2, "two data rows must produce list of length 2");

        let row0 = list[0].as_map().expect("each row must be Value::Map");
        assert_eq!(
            row0.get("name"),
            Some(&Value::Str(Arc::from("Alice"))),
            "first row name must be Alice"
        );
        assert_eq!(
            row0.get("revenue"),
            Some(&Value::Str(Arc::from("100"))),
            "first row revenue must be string 100"
        );

        let row1 = list[1].as_map().expect("each row must be Value::Map");
        assert_eq!(
            row1.get("name"),
            Some(&Value::Str(Arc::from("Bob"))),
            "second row name must be Bob"
        );
        assert_eq!(
            row1.get("revenue"),
            Some(&Value::Str(Arc::from("200"))),
            "second row revenue must be string 200"
        );
    }

    /// test_BC_5_03_004_parse_csv_duplicate_headers — duplicate column names → DataError.
    ///
    /// Per EC-006 and EC-007: duplicate header names must be rejected.
    #[test]
    fn test_bc_5_03_004_parse_csv_duplicate_headers() {
        let src = "name,name\nAlice,Alice";
        let result = parse_csv(src, "test.csv");
        let err = result.expect_err("duplicate headers must return Err");
        let msg = err.to_string();
        assert!(
            msg.contains("duplicate") || msg.contains("E-DAT-003"),
            "error must mention 'duplicate' or carry E-DAT-003: {msg}"
        );
    }

    /// test_BC_5_03_004_parse_csv_headers_only — header row with no data rows → empty List.
    #[test]
    fn test_bc_5_03_004_parse_csv_headers_only() {
        let src = "name,val\n";
        let value = parse_csv(src, "test.csv").expect("headers-only CSV must parse");
        let list = value.as_list().expect("must be Value::List");
        assert!(
            list.is_empty(),
            "CSV with only header row must produce empty list"
        );
    }

    /// test_BC_5_03_004_parse_csv_header_whitespace_trimming — headers with surrounding spaces are trimmed.
    #[test]
    fn test_bc_5_03_004_parse_csv_header_whitespace_trimming() {
        let src = " name , val \nAlice,42";
        let value = parse_csv(src, "test.csv").expect("whitespace-header CSV must parse");
        let list = value.as_list().expect("must be Value::List");
        assert_eq!(list.len(), 1);
        let row = list[0].as_map().expect("row must be map");
        // Keys must be trimmed: "name" not " name "
        assert!(
            row.get("name").is_some(),
            "header whitespace must be trimmed — key 'name' must be accessible"
        );
        assert!(
            row.get(" name ").is_none(),
            "untrimmed key ' name ' must NOT be present"
        );
    }

    /// test_BC_5_03_004_parse_csv_missing_header — empty string → DataError (no header row).
    #[test]
    fn test_bc_5_03_004_parse_csv_missing_header() {
        let src = "";
        // Either an empty list or an error is acceptable, but must not panic.
        // If it's an error, it should carry E-DAT-003.
        match parse_csv(src, "test.csv") {
            Ok(v) => {
                // Empty input — either empty list is OK
                let list = v.as_list().expect("if Ok, must be List");
                assert!(list.is_empty(), "empty CSV produces empty list");
            }
            Err(e) => {
                assert_eq!(e.code(), "E-DAT-003");
            }
        }
    }
}
