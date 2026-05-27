//! YAML parser for slideforge data sources.
//!
//! Parses a YAML string into a [`slideforge_types::Value`].
//!
//! ## No implicit coercion (R3 finding)
//!
//! `serde_yaml_ng` 0.10+ preserves YAML 1.2 semantics where `YES`, `NO`,
//! `on`, `off` are NOT automatically coerced to booleans. Only the canonical
//! YAML 1.2 boolean literals (`true` / `false`) become [`Value::Bool`].
//!
//! This is enforced by the test suite:
//! - `flag: YES` → [`Value::Str`]`("YES")` — NOT [`Value::Bool`]`(false)`
//! - `flag: on` → [`Value::Str`]`("on")`
//! - `flag: no` → [`Value::Str`]`("no")`
//! - `flag: true` → [`Value::Bool`]`(true)` ✓

use std::sync::Arc;

use ordered_float::OrderedFloat;
use slideforge_types::{OrderedMap, Value};

use crate::DataError;

/// Convert a [`serde_yaml_ng::Value`] into a [`slideforge_types::Value`].
fn yaml_value_to_sf(v: serde_yaml_ng::Value) -> Value {
    match v {
        serde_yaml_ng::Value::Null => Value::Null,
        serde_yaml_ng::Value::Bool(b) => Value::Bool(b),
        serde_yaml_ng::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(OrderedFloat(f))
            } else {
                // Fallback: render as string (e.g. u64 overflow)
                Value::Str(Arc::from(n.to_string().as_str()))
            }
        }
        serde_yaml_ng::Value::String(s) => Value::Str(Arc::from(s.as_str())),
        serde_yaml_ng::Value::Sequence(seq) => {
            let items = seq.into_iter().map(yaml_value_to_sf).collect();
            Value::List(items)
        }
        serde_yaml_ng::Value::Mapping(mapping) => {
            let mut map = OrderedMap::new();
            for (k, v) in mapping {
                // Keys are expected to be strings; convert others to string
                let key = match k {
                    serde_yaml_ng::Value::String(s) => Arc::from(s.as_str()),
                    other => Arc::from(
                        serde_yaml_ng::to_string(&other)
                            .unwrap_or_default()
                            .trim()
                            .to_string()
                            .as_str(),
                    ),
                };
                map.insert(key, yaml_value_to_sf(v));
            }
            Value::Map(map)
        }
        serde_yaml_ng::Value::Tagged(tagged) => {
            // Treat tagged values as their inner value
            yaml_value_to_sf(tagged.value)
        }
    }
}

/// Parse a YAML string and return the root [`Value`].
///
/// # Errors
///
/// Returns [`DataError::ParseError`] with error code `E-DAT-003` if the
/// input is not valid YAML.
pub fn parse_yaml(source: &str, path: &str) -> Result<Value, DataError> {
    let raw: serde_yaml_ng::Value = serde_yaml_ng::from_str(source)
        .map_err(|e| DataError::parse_error(path, e.to_string()))?;
    Ok(yaml_value_to_sf(raw))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use slideforge_types::Value;

    use super::*;

    /// test_BC_5_03_005_parse_yaml_yes_stays_string — `YES` must remain a string.
    ///
    /// R3 finding: YAML-style implicit coercion is forbidden in slideforge.
    /// `YES` is NOT a boolean in YAML 1.2.
    #[test]
    fn test_bc_5_03_005_parse_yaml_yes_stays_string() {
        let src = "flag: YES";
        let value = parse_yaml(src, "test.yaml").expect("must parse");
        let map = value.as_map().expect("must be map");
        let flag = map.get("flag").expect("flag field must be present");
        assert_eq!(
            flag,
            &Value::Str(Arc::from("YES")),
            "'YES' must stay Value::Str(\"YES\") — no implicit bool coercion (R3)"
        );
        assert_ne!(
            flag,
            &Value::Bool(false),
            "'YES' must NOT become Value::Bool(false)"
        );
    }

    /// test_BC_5_03_005_parse_yaml_on_stays_string — `on` must remain a string.
    #[test]
    fn test_bc_5_03_005_parse_yaml_on_stays_string() {
        let src = "flag: on";
        let value = parse_yaml(src, "test.yaml").expect("must parse");
        let map = value.as_map().expect("must be map");
        let flag = map.get("flag").expect("flag field must be present");
        assert_eq!(
            flag,
            &Value::Str(Arc::from("on")),
            "'on' must stay Value::Str — no YAML 1.1 bool coercion (R3)"
        );
    }

    /// test_BC_5_03_005_parse_yaml_no_stays_string — `no` must remain a string.
    #[test]
    fn test_bc_5_03_005_parse_yaml_no_stays_string() {
        let src = "flag: no";
        let value = parse_yaml(src, "test.yaml").expect("must parse");
        let map = value.as_map().expect("must be map");
        let flag = map.get("flag").expect("flag field must be present");
        assert_eq!(
            flag,
            &Value::Str(Arc::from("no")),
            "'no' must stay Value::Str — no YAML 1.1 bool coercion (R3)"
        );
    }

    /// test_BC_5_03_005_parse_yaml_true_is_bool — canonical `true` must become Value::Bool(true).
    #[test]
    fn test_bc_5_03_005_parse_yaml_true_is_bool() {
        let src = "flag: true";
        let value = parse_yaml(src, "test.yaml").expect("must parse");
        let map = value.as_map().expect("must be map");
        let flag = map.get("flag").expect("flag field must be present");
        assert_eq!(
            flag,
            &Value::Bool(true),
            "canonical 'true' must become Value::Bool(true)"
        );
    }

    /// test_BC_5_03_005_parse_yaml_false_is_bool — canonical `false` must become Value::Bool(false).
    #[test]
    fn test_bc_5_03_005_parse_yaml_false_is_bool() {
        let src = "flag: false";
        let value = parse_yaml(src, "test.yaml").expect("must parse");
        let map = value.as_map().expect("must be map");
        let flag = map.get("flag").expect("flag field must be present");
        assert_eq!(
            flag,
            &Value::Bool(false),
            "canonical 'false' must become Value::Bool(false)"
        );
    }

    /// test_BC_5_03_005_parse_yaml_happy_path — int and string fields parse correctly.
    #[test]
    fn test_bc_5_03_005_parse_yaml_happy_path() {
        let src = "count: 5\nlabel: hello";
        let value = parse_yaml(src, "test.yaml").expect("must parse");
        let map = value.as_map().expect("must be map");

        assert_eq!(
            map.get("count"),
            Some(&Value::Int(5)),
            "integer field must become Value::Int"
        );
        assert_eq!(
            map.get("label"),
            Some(&Value::Str(Arc::from("hello"))),
            "string field must become Value::Str"
        );
    }

    /// test_BC_5_03_005_parse_yaml_null_preserved — YAML `null` → Value::Null.
    #[test]
    fn test_bc_5_03_005_parse_yaml_null_preserved() {
        let src = "x: null";
        let value = parse_yaml(src, "test.yaml").expect("must parse");
        let map = value.as_map().expect("must be map");
        assert_eq!(
            map.get("x"),
            Some(&Value::Null),
            "YAML null must become Value::Null"
        );
    }

    /// test_BC_5_03_005_parse_yaml_malformed — invalid YAML → DataError with E-DAT-003.
    #[test]
    fn test_bc_5_03_005_parse_yaml_malformed() {
        let src = ":\t:invalid";
        let result = parse_yaml(src, "bad.yaml");
        let err = result.expect_err("malformed YAML must return Err");
        assert_eq!(err.code(), "E-DAT-003");
    }
}
