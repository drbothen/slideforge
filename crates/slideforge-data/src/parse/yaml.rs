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

use crate::format::DataFormat;
use crate::DataError;

/// Convert a [`serde_yaml_ng::Value`] into a [`slideforge_types::Value`].
fn yaml_value_to_sf(v: serde_yaml_ng::Value) -> Value {
    match v {
        serde_yaml_ng::Value::Null => Value::Null,
        serde_yaml_ng::Value::Bool(b) => Value::Bool(b),
        serde_yaml_ng::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if n.as_u64().is_some() {
                // u64 values that overflow i64 (e.g. 9999999999999999999) must be
                // preserved as strings — not silently rounded via f64 (FINDING-001).
                Value::Str(Arc::from(n.to_string().as_str()))
            } else if let Some(f) = n.as_f64() {
                Value::Float(OrderedFloat(f))
            } else {
                // Fallback: render as string (e.g. special float representations)
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
                // Keys are expected to be strings; convert others to their natural
                // string representation without YAML document markers.
                let key = match k {
                    serde_yaml_ng::Value::String(s) => Arc::from(s.as_str()),
                    serde_yaml_ng::Value::Number(n) => Arc::from(n.to_string().as_str()),
                    serde_yaml_ng::Value::Bool(b) => Arc::from(if b { "true" } else { "false" }),
                    serde_yaml_ng::Value::Null => Arc::from("null"),
                    other => {
                        // For sequences, mappings, or tagged values: serialize and
                        // strip the YAML document-start marker (`---\n`) if present.
                        let raw = serde_yaml_ng::to_string(&other).unwrap_or_default();
                        let stripped = raw
                            .strip_prefix("---\n")
                            .unwrap_or(raw.as_str())
                            .trim()
                            .to_owned();
                        Arc::from(stripped.as_str())
                    }
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
        .map_err(|e| DataError::parse_error(path, DataFormat::Yaml, e.to_string()))?;
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

    /// test_BC_5_03_005_parse_yaml_yes_upper_stays_string — `YES` (uppercase) must remain a string.
    ///
    /// YAML 1.1 treats `YES` as `true`; YAML 1.2 does not. slideforge uses YAML 1.2 semantics.
    #[test]
    fn test_bc_5_03_005_parse_yaml_yes_upper_stays_string() {
        let src = "flag: YES";
        let value = parse_yaml(src, "test.yaml").expect("must parse");
        let map = value.as_map().expect("must be map");
        let flag = map.get("flag").expect("flag must be present");
        assert_eq!(flag, &Value::Str(Arc::from("YES")), "'YES' must be Str");
    }

    /// test_BC_5_03_005_parse_yaml_on_upper_stays_string — `ON` must remain a string.
    #[test]
    fn test_bc_5_03_005_parse_yaml_on_upper_stays_string() {
        let src = "flag: ON";
        let value = parse_yaml(src, "test.yaml").expect("must parse");
        let map = value.as_map().expect("must be map");
        let flag = map.get("flag").expect("flag must be present");
        assert_eq!(flag, &Value::Str(Arc::from("ON")), "'ON' must be Str");
    }

    /// test_BC_5_03_005_parse_yaml_no_upper_stays_string — `NO` must remain a string.
    #[test]
    fn test_bc_5_03_005_parse_yaml_no_upper_stays_string() {
        let src = "flag: NO";
        let value = parse_yaml(src, "test.yaml").expect("must parse");
        let map = value.as_map().expect("must be map");
        let flag = map.get("flag").expect("flag must be present");
        assert_eq!(flag, &Value::Str(Arc::from("NO")), "'NO' must be Str");
    }

    /// test_BC_5_03_005_parse_yaml_off_lower_stays_string — `off` must remain a string.
    #[test]
    fn test_bc_5_03_005_parse_yaml_off_lower_stays_string() {
        let src = "flag: off";
        let value = parse_yaml(src, "test.yaml").expect("must parse");
        let map = value.as_map().expect("must be map");
        let flag = map.get("flag").expect("flag must be present");
        assert_eq!(flag, &Value::Str(Arc::from("off")), "'off' must be Str");
    }

    /// test_BC_5_03_005_parse_yaml_off_upper_stays_string — `OFF` must remain a string.
    #[test]
    fn test_bc_5_03_005_parse_yaml_off_upper_stays_string() {
        let src = "flag: OFF";
        let value = parse_yaml(src, "test.yaml").expect("must parse");
        let map = value.as_map().expect("must be map");
        let flag = map.get("flag").expect("flag must be present");
        assert_eq!(flag, &Value::Str(Arc::from("OFF")), "'OFF' must be Str");
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

    /// test_bc_5_03_005_parse_yaml_large_u64_stays_str — u64 value outside i64 range must become
    /// Value::Str, not a lossy Value::Float (FINDING-001).
    ///
    /// A value like `9999999999999999999` cannot fit in i64 (max ~9.2e18) and must not be
    /// silently rounded to a float. The JSON parser has the same guard; YAML must match.
    #[test]
    fn test_bc_5_03_005_parse_yaml_large_u64_stays_str() {
        // 9999999999999999999 > i64::MAX (9223372036854775807) but fits in u64
        let src = "big: 9999999999999999999";
        let value = parse_yaml(src, "test.yaml").expect("must parse");
        let map = value.as_map().expect("must be map");
        let big = map.get("big").expect("big field must be present");
        assert_eq!(
            big,
            &Value::Str(Arc::from("9999999999999999999")),
            "u64-range integer outside i64 range must become Value::Str to preserve precision (FINDING-001)"
        );
        // Must NOT be a Float (which would lose precision)
        assert!(
            big.as_float().is_none(),
            "large u64 must NOT become Value::Float (precision loss)"
        );
    }

    /// test_bc_5_03_005_parse_yaml_snapshot — representative YAML fixture snapshot test.
    ///
    /// Covers map, list, int, float, bool, string, and null in one fixture to catch
    /// any regression in the `yaml_value_to_sf` conversion.
    #[test]
    fn test_bc_5_03_005_parse_yaml_snapshot() {
        let src = r#"
title: "Q1 Report"
slides: 12
active: true
ratio: 1.78
tags:
  - revenue
  - growth
meta: null
"#;
        let value = parse_yaml(src, "fixture.yaml").expect("must parse");
        insta::assert_debug_snapshot!(value);
    }

    /// test_BC_5_03_005_parse_yaml_non_string_keys — integer and bool keys are converted to
    /// string without a `---\n` prefix (FINDING-013).
    #[test]
    fn test_bc_5_03_005_parse_yaml_non_string_keys() {
        // YAML allows integer and boolean mapping keys
        let src = "? 1\n: one\n? true\n: yes_value";
        let value = parse_yaml(src, "test.yaml").expect("must parse");
        let map = value.as_map().expect("must be map");

        // Integer key 1 → "1" (no "---\n" prefix)
        let key_1 = map.get("1");
        assert!(key_1.is_some(), "integer key 1 must be accessible as \"1\"");
        assert_eq!(
            key_1,
            Some(&Value::Str(Arc::from("one"))),
            "value under integer key 1 must be \"one\""
        );

        // Bool key true → "true"
        let key_true = map.get("true");
        assert!(key_true.is_some(), "bool key true must be accessible as \"true\"");
        assert_eq!(
            key_true,
            Some(&Value::Str(Arc::from("yes_value"))),
            "value under bool key true must be \"yes_value\""
        );
    }
}
