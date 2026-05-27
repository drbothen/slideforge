//! TOML parser for slideforge data sources.
//!
//! Parses a TOML string into a [`slideforge_types::Value`].
//! TOML datetimes are represented as [`slideforge_types::Value::Str`] using
//! ISO 8601 formatting (no native datetime variant in `Value`).

use std::sync::Arc;

use ordered_float::OrderedFloat;
use slideforge_types::{OrderedMap, Value};

use crate::DataError;

/// Convert a [`toml::Value`] into a [`slideforge_types::Value`].
fn toml_value_to_sf(v: toml::Value) -> Value {
    match v {
        toml::Value::String(s) => Value::Str(Arc::from(s.as_str())),
        toml::Value::Integer(i) => Value::Int(i),
        toml::Value::Float(f) => Value::Float(OrderedFloat(f)),
        toml::Value::Boolean(b) => Value::Bool(b),
        toml::Value::Array(arr) => {
            let items = arr.into_iter().map(toml_value_to_sf).collect();
            Value::List(items)
        }
        toml::Value::Table(table) => {
            let mut map = OrderedMap::new();
            for (k, v) in table {
                map.insert(Arc::from(k.as_str()), toml_value_to_sf(v));
            }
            Value::Map(map)
        }
        toml::Value::Datetime(dt) => {
            // TOML has a native datetime type; serialize to ISO 8601 string.
            Value::Str(Arc::from(dt.to_string().as_str()))
        }
    }
}

/// Parse a TOML string and return the root [`Value`].
///
/// # Errors
///
/// Returns [`DataError::ParseError`] with error code `E-DAT-003` if the
/// input is not valid TOML.
pub fn parse_toml(source: &str, path: &str) -> Result<Value, DataError> {
    let raw: toml::Value = toml::from_str(source)
        .map_err(|e| DataError::parse_error(path, e.to_string()))?;
    Ok(toml_value_to_sf(raw))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use slideforge_types::Value;

    use super::*;

    /// test_BC_5_03_006_parse_toml_happy_path — section with int, string, bool, float.
    #[test]
    fn test_bc_5_03_006_parse_toml_happy_path() {
        let src = r#"
[meta]
count = 10
label = "revenue"
active = true
score = 2.5
"#;
        let value = parse_toml(src, "test.toml").expect("valid TOML must parse");
        let root = value.as_map().expect("root must be Value::Map");
        let meta = root
            .get("meta")
            .expect("'meta' section must be present")
            .as_map()
            .expect("'meta' must be Value::Map");

        assert_eq!(
            meta.get("count"),
            Some(&Value::Int(10)),
            "count must be Value::Int(10)"
        );
        assert_eq!(
            meta.get("label"),
            Some(&Value::Str(Arc::from("revenue"))),
            "label must be Value::Str"
        );
        assert_eq!(
            meta.get("active"),
            Some(&Value::Bool(true)),
            "active must be Value::Bool(true)"
        );
        let score = meta.get("score").expect("score must be present");
        let f = score.as_float().expect("score must be Value::Float");
        assert!(
            (f - 2.5_f64).abs() < 1e-10,
            "score must be approximately 2.5"
        );
    }

    /// test_BC_5_03_006_parse_toml_datetime — TOML datetime → Value::Str with ISO 8601.
    ///
    /// TOML has a native datetime type; since `Value` has no datetime variant,
    /// datetimes must be serialized to ISO 8601 strings.
    #[test]
    fn test_bc_5_03_006_parse_toml_datetime() {
        let src = "created = 1979-05-27T07:32:00Z\n";
        let value = parse_toml(src, "test.toml").expect("must parse");
        let root = value.as_map().expect("must be map");
        let created = root.get("created").expect("'created' must be present");
        // Must be a string, not panic
        let s = created
            .as_str()
            .expect("TOML datetime must become Value::Str");
        // ISO 8601: must contain at least the date portion
        assert!(
            s.contains("1979"),
            "ISO 8601 datetime string must contain '1979', got: {s}"
        );
    }

    /// test_BC_5_03_006_parse_toml_malformed — invalid TOML → DataError with E-DAT-003.
    #[test]
    fn test_bc_5_03_006_parse_toml_malformed() {
        let src = "this is not = [valid toml";
        let result = parse_toml(src, "bad.toml");
        let err = result.expect_err("malformed TOML must return Err");
        assert_eq!(err.code(), "E-DAT-003");
    }

    /// test_BC_5_03_006_parse_toml_flat_table — flat key-value pairs without sections.
    #[test]
    fn test_bc_5_03_006_parse_toml_flat_table() {
        let src = "name = \"Alice\"\nage = 30\n";
        let value = parse_toml(src, "test.toml").expect("must parse");
        let root = value.as_map().expect("must be map");
        assert_eq!(
            root.get("name"),
            Some(&Value::Str(Arc::from("Alice")))
        );
        assert_eq!(root.get("age"), Some(&Value::Int(30)));
    }
}
