//! JSON parser for slideforge data sources.
//!
//! Parses a JSON string into a [`slideforge_types::Value`].
//! All JSON types map to `Value` variants; `null` → [`Value::Null`],
//! numbers without fractional parts → [`Value::Int`], fractional → [`Value::Float`].

use std::sync::Arc;

use ordered_float::OrderedFloat;
use slideforge_types::{OrderedMap, Value};

use crate::format::DataFormat;
use crate::DataError;

/// Convert a [`serde_json::Value`] into a [`slideforge_types::Value`].
fn json_value_to_sf(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(OrderedFloat(f))
            } else {
                // Fallback: render as string (should not occur with standard JSON)
                Value::Str(Arc::from(n.to_string().as_str()))
            }
        }
        serde_json::Value::String(s) => Value::Str(Arc::from(s.as_str())),
        serde_json::Value::Array(arr) => {
            let items = arr.into_iter().map(json_value_to_sf).collect();
            Value::List(items)
        }
        serde_json::Value::Object(obj) => {
            let mut map = OrderedMap::new();
            for (k, v) in obj {
                map.insert(Arc::from(k.as_str()), json_value_to_sf(v));
            }
            Value::Map(map)
        }
    }
}

/// Parse a JSON string and return the root [`Value`].
///
/// # Errors
///
/// Returns [`DataError::ParseError`] with error code `E-DAT-003` if the
/// input is not valid JSON.
pub fn parse_json(source: &str, path: &str) -> Result<Value, DataError> {
    let raw: serde_json::Value = serde_json::from_str(source)
        .map_err(|e| DataError::parse_error(path, DataFormat::Json, e.to_string()))?;
    Ok(json_value_to_sf(raw))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use insta::assert_debug_snapshot;
    #[allow(unused_imports)]
    use slideforge_types::OrderedMap;
    use std::sync::Arc;

    use super::*;

    /// test_BC_5_03_003_snapshot_json_fixture — snapshot test for JSON parsing output (FINDING-008).
    ///
    /// Verifies that the JSON parser produces a stable, deterministic debug representation
    /// for a known fixture. Snapshot is checked via `cargo insta test`.
    #[test]
    fn test_bc_5_03_003_snapshot_json_fixture() {
        let src = r#"{"name": "Alice", "score": 42, "active": true, "tags": ["rust", "data"]}"#;
        let value = parse_json(src, "fixture.json").expect("fixture must parse");
        assert_debug_snapshot!("json_fixture", value);
    }

    /// test_BC_5_03_003_parse_json_happy_path — complete JSON object with all supported types.
    ///
    /// Input: `{"revenue": 42, "label": "Q1", "active": true, "score": 3.14, "items": [1, 2], "meta": null}`
    /// Expected: Map with correct Value variants per field.
    #[test]
    fn test_bc_5_03_003_parse_json_happy_path() {
        let src =
            r#"{"revenue": 42, "label": "Q1", "active": true, "score": 3.14, "items": [1, 2], "meta": null}"#;
        let result = parse_json(src, "test.json");
        let value = result.expect("happy-path JSON must parse without error");

        let map = value
            .as_map()
            .expect("root JSON object must produce Value::Map");

        // revenue: 42 → Int(42)
        assert_eq!(
            map.get("revenue"),
            Some(&Value::Int(42)),
            "revenue must be Value::Int(42)"
        );

        // label: "Q1" → Str("Q1")
        assert_eq!(
            map.get("label"),
            Some(&Value::Str(Arc::from("Q1"))),
            "label must be Value::Str(\"Q1\")"
        );

        // active: true → Bool(true)
        assert_eq!(
            map.get("active"),
            Some(&Value::Bool(true)),
            "active must be Value::Bool(true)"
        );

        // score: 3.14 → Float(3.14)
        let score = map.get("score").expect("score field must be present");
        let f = score
            .as_float()
            .expect("score must be Value::Float");
        assert!(
            (f - 3.14_f64).abs() < 1e-10,
            "score float must be approximately 3.14"
        );

        // items: [1, 2] → List([Int(1), Int(2)])
        let items = map.get("items").expect("items field must be present");
        let list = items.as_list().expect("items must be Value::List");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], Value::Int(1));
        assert_eq!(list[1], Value::Int(2));

        // meta: null → Null
        assert_eq!(
            map.get("meta"),
            Some(&Value::Null),
            "meta must be Value::Null"
        );
    }

    /// test_BC_5_03_003_parse_json_null_preservation — explicit null stays Value::Null, not empty string.
    #[test]
    fn test_bc_5_03_003_parse_json_null_preservation() {
        let src = r#"{"x": null}"#;
        let value = parse_json(src, "test.json").expect("must parse");
        let map = value.as_map().expect("must be map");
        assert_eq!(
            map.get("x"),
            Some(&Value::Null),
            "JSON null must become Value::Null — not empty string"
        );
    }

    /// test_BC_5_03_003_parse_json_malformed — malformed JSON returns DataError::ParseError with E-DAT-003.
    #[test]
    fn test_bc_5_03_003_parse_json_malformed() {
        let src = "{broken";
        let result = parse_json(src, "bad.json");
        let err = result.expect_err("malformed JSON must return Err");
        assert_eq!(
            err.code(),
            "E-DAT-003",
            "malformed JSON must carry E-DAT-003 error code"
        );
        assert!(
            err.to_string().contains("E-DAT-003"),
            "error Display must include the error code"
        );
    }

    /// test_BC_5_03_003_parse_json_empty_object — `{}` → Value::Map(empty).
    #[test]
    fn test_bc_5_03_003_parse_json_empty_object() {
        let src = "{}";
        let value = parse_json(src, "test.json").expect("empty object must parse");
        let map = value.as_map().expect("empty object must be Value::Map");
        assert!(map.is_empty(), "empty JSON object must produce empty map");
    }

    /// test_BC_5_03_003_parse_json_empty_array — `[]` → Value::List(empty).
    #[test]
    fn test_bc_5_03_003_parse_json_empty_array() {
        let src = "[]";
        let value = parse_json(src, "test.json").expect("empty array must parse");
        let list = value.as_list().expect("empty array must be Value::List");
        assert!(list.is_empty(), "empty JSON array must produce empty list");
    }

    /// test_BC_5_03_003_parse_json_integer_vs_float — integer JSON numbers become Value::Int,
    /// fractional numbers become Value::Float.
    #[test]
    fn test_bc_5_03_003_parse_json_integer_vs_float() {
        let src = r#"{"n": 7, "f": 7.5}"#;
        let value = parse_json(src, "test.json").expect("must parse");
        let map = value.as_map().expect("must be map");
        assert!(
            matches!(map.get("n"), Some(Value::Int(7))),
            "integer 7 must be Value::Int"
        );
        assert!(
            matches!(map.get("f"), Some(Value::Float(_))),
            "fractional 7.5 must be Value::Float"
        );
    }
}
