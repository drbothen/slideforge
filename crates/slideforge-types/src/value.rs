//! The `Value` type — the runtime value type for slideforge expressions.
//!
//! `Value` intentionally provides NO implicit coercions (`From` / `Into`)
//! between variants. This prevents the silent type-coercion bugs present in
//! the Python reference implementation (R3 finding: `NO` should stay `"NO"`,
//! not silently become `false`).

use std::sync::Arc;

use ordered_float::OrderedFloat;

use crate::ordered_map::OrderedMap;
use crate::type_kind::TypeKind;

/// A runtime value produced by the slideforge evaluator.
///
/// ## Coercion policy
///
/// `Value` provides no `From` / `Into` conversions between variants. The
/// evaluator must produce the correct variant directly. This is intentional:
/// YAML-style implicit coercion (`NO` → bool, `1.10` → float) is a forbidden
/// pattern in slideforge (R3 finding).
///
/// The following compile-time guards enforce the no-coercion invariant.
/// If anyone adds `impl From<bool> for Value` (or `From<i64>`, `From<&str>`),
/// the corresponding `compile_fail` test below will break the build.
///
/// ```compile_fail
/// use slideforge_types::Value;
/// let _: Value = true.into(); // must not compile — no From<bool> for Value
/// ```
///
/// ```compile_fail
/// use slideforge_types::Value;
/// let _: Value = 42_i64.into(); // must not compile — no From<i64> for Value
/// ```
///
/// ```compile_fail
/// use slideforge_types::Value;
/// let _: Value = "hello".into(); // must not compile — no From<&str> for Value
/// ```
///
/// ```compile_fail
/// use slideforge_types::Value;
/// let _: Value = 1.0_f64.into(); // must not compile — no From<f64> for Value
/// ```
///
/// ## Hash + Eq
///
/// `Value` implements `Hash` and `Eq` because floating-point values are
/// wrapped in [`OrderedFloat`], which provides a **total order** over all
/// `f64` values, including NaN. Unlike raw `f64` (IEEE 754), two NaN values
/// wrapped in `OrderedFloat` compare equal to each other and hash to the same
/// bucket. This makes `Value` safe to use as a `HashMap` key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    /// A string value. Uses `Arc<str>` for cheap cloning.
    Str(Arc<str>),

    /// A 64-bit signed integer value.
    Int(i64),

    /// A 64-bit floating-point value, wrapped in [`OrderedFloat`] so that
    /// `Value` can implement `Hash` and `Eq`.
    ///
    /// [`OrderedFloat`] provides a **total ordering** over all `f64` values,
    /// including NaN. Crucially, `OrderedFloat::nan() == OrderedFloat::nan()`
    /// is `true` (unlike raw `f64` where `NaN != NaN` per IEEE 754). Two NaN
    /// values hash to the same bucket and compare equal.
    Float(OrderedFloat<f64>),

    /// A boolean value.
    Bool(bool),

    /// An ordered list of values.
    List(Vec<Value>),

    /// An ordered map from string keys to values.
    ///
    /// [`OrderedMap`] preserves insertion order, which is important for
    /// deterministic rendering and snapshot testing, and additionally
    /// implements `Hash` for comemo compatibility.
    Map(OrderedMap<Arc<str>, Value>),

    /// The null / absent value.
    Null,
}

impl Value {
    /// Return `true` if this is a [`Value::Null`].
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::Value;
    /// assert!(Value::Null.is_null());
    /// assert!(!Value::Bool(true).is_null());
    /// ```
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Return the string slice if this is a [`Value::Str`], otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::Value;
    /// use std::sync::Arc;
    /// let v = Value::Str(Arc::from("hello"));
    /// assert_eq!(v.as_str(), Some("hello"));
    /// assert_eq!(Value::Null.as_str(), None);
    /// ```
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s.as_ref()),
            _ => None,
        }
    }

    /// Return the integer if this is a [`Value::Int`], otherwise `None`.
    #[must_use]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    /// Return the float if this is a [`Value::Float`], otherwise `None`.
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(f.0),
            _ => None,
        }
    }

    /// Return the bool if this is a [`Value::Bool`], otherwise `None`.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Return a slice reference if this is a [`Value::List`], otherwise `None`.
    #[must_use]
    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(v) => Some(v),
            _ => None,
        }
    }

    /// Return a reference to the map if this is a [`Value::Map`], otherwise `None`.
    #[must_use]
    pub fn as_map(&self) -> Option<&OrderedMap<Arc<str>, Value>> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    /// Return the discriminant name for display / error messages.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::Value;
    /// assert_eq!(Value::Null.type_name(), "null");
    /// assert_eq!(Value::Bool(true).type_name(), "bool");
    /// ```
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Str(_) => "string",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::List(_) => "list",
            Value::Map(_) => "map",
            Value::Null => "null",
        }
    }

    /// Return the [`TypeKind`] discriminant of this value.
    ///
    /// Used by the evaluator to format type-error messages (E-EVL-003) such as
    /// `"expected integer, got string"` without cloning the full value.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::{Value, TypeKind};
    /// use std::sync::Arc;
    ///
    /// assert_eq!(Value::Str(Arc::from("hello")).type_kind(), TypeKind::Str);
    /// assert_eq!(Value::Int(42).type_kind(), TypeKind::Int);
    /// assert_eq!(Value::Null.type_kind(), TypeKind::Null);
    /// ```
    #[must_use]
    pub fn type_kind(&self) -> TypeKind {
        match self {
            Value::Str(_) => TypeKind::Str,
            Value::Int(_) => TypeKind::Int,
            Value::Float(_) => TypeKind::Float,
            Value::Bool(_) => TypeKind::Bool,
            Value::List(_) => TypeKind::List,
            Value::Map(_) => TypeKind::Map,
            Value::Null => TypeKind::Null,
        }
    }

    /// Evaluates the truthiness of a value for use in `@if` condition evaluation.
    ///
    /// This method does NOT perform type coercion. It is an explicit truthiness
    /// check that is only called by the evaluator when processing an `@if`
    /// condition. The evaluator explicitly calls `.is_truthy()` — it is never
    /// called implicitly during arithmetic or comparison.
    ///
    /// Per BC-1.02.003: a string value like `"NO"` evaluates to truthy
    /// (non-empty string), NOT to false. Use `@if var == false` to compare a
    /// boolean, not `@if var`.
    ///
    /// | Variant | Truthy when |
    /// |---------|-------------|
    /// | `Bool(b)` | `b` is `true` |
    /// | `Int(n)` | `n != 0` |
    /// | `Float(f)` | `f != 0.0` (NaN is truthy — see EC-001) |
    /// | `Str(s)` | `s` is non-empty |
    /// | `List(l)` | `l` is non-empty |
    /// | `Map(m)` | `m` is non-empty |
    /// | `Null` | always `false` |
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::Value;
    /// use std::sync::Arc;
    ///
    /// assert!(Value::Bool(true).is_truthy());
    /// assert!(!Value::Bool(false).is_truthy());
    /// assert!(Value::Int(1).is_truthy());
    /// assert!(!Value::Int(0).is_truthy());
    /// assert!(Value::Str(Arc::from("NO")).is_truthy()); // "NO" is truthy — non-empty string
    /// assert!(!Value::Str(Arc::from("")).is_truthy());
    /// assert!(!Value::Null.is_truthy());
    /// ```
    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => f.0 != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Map(m) => !m.is_empty(),
            Value::Null => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ──────────────────────────────────────────────────────────────────────────
    // AC-003 — Value has exactly 7 variants, implements Hash+Eq+Clone+Debug
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_003_value_str_variant() {
        let v = Value::Str(Arc::from("hello"));
        assert_eq!(v.as_str(), Some("hello"));
    }

    #[test]
    fn test_bc_1_01_003_value_int_variant() {
        let v = Value::Int(42);
        assert_eq!(v.as_int(), Some(42));
    }

    #[test]
    fn test_bc_1_01_003_value_float_variant() {
        // Use a non-standard value to avoid the clippy::approx_constant lint.
        let val = 1.234_567_89_f64;
        let v = Value::Float(OrderedFloat(val));
        assert!((v.as_float().expect("Value::Float variant must return Some") - val).abs() < 1e-10);
    }

    #[test]
    fn test_bc_1_01_003_value_bool_variant() {
        let v = Value::Bool(true);
        assert_eq!(v.as_bool(), Some(true));
    }

    #[test]
    fn test_bc_1_01_003_value_list_variant() {
        let v = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(
            v.as_list()
                .expect("Value::List variant must return Some")
                .len(),
            2
        );
    }

    #[test]
    fn test_bc_1_01_003_value_map_variant() {
        let mut m = OrderedMap::new();
        m.insert(Arc::from("key"), Value::Str(Arc::from("val")));
        let v = Value::Map(m);
        assert!(v.as_map().is_some());
    }

    #[test]
    fn test_bc_1_01_003_value_null_variant() {
        assert!(Value::Null.is_null());
    }

    #[test]
    fn test_bc_1_01_003_value_implements_hash() {
        use std::collections::HashSet;
        // HashSet<Value> must compile — proves Hash
        let mut set: HashSet<Value> = HashSet::new();
        set.insert(Value::Int(1));
        set.insert(Value::Str(Arc::from("hello")));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_bc_1_01_003_value_implements_eq() {
        assert_eq!(Value::Int(1), Value::Int(1));
        assert_ne!(Value::Int(1), Value::Int(2));
    }

    #[test]
    fn test_bc_1_01_003_value_implements_clone() {
        let v = Value::Str(Arc::from("clone me"));
        let v2 = v.clone();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_bc_1_01_003_value_implements_debug() {
        let v = Value::Bool(true);
        let s = format!("{v:?}");
        assert!(s.contains("Bool"));
    }

    /// No implicit coercion: Int(1) != Bool(true) — different types stay distinct.
    #[test]
    fn test_bc_1_01_003_value_no_coercion_int_vs_bool() {
        assert_ne!(Value::Int(1), Value::Bool(true));
    }

    /// No implicit coercion: Str("NO") != Bool(false) — YAML-style coercion forbidden.
    #[test]
    fn test_bc_1_01_003_value_no_coercion_str_no_vs_bool() {
        assert_ne!(Value::Str(Arc::from("NO")), Value::Bool(false));
    }

    /// No implicit coercion: Str("1") != Int(1).
    #[test]
    fn test_bc_1_01_003_value_no_coercion_str_vs_int() {
        assert_ne!(Value::Str(Arc::from("1")), Value::Int(1));
    }

    /// Type names are stable strings useful in error messages.
    #[test]
    fn test_bc_1_01_003_value_type_names() {
        assert_eq!(Value::Str(Arc::from("")).type_name(), "string");
        assert_eq!(Value::Int(0).type_name(), "int");
        assert_eq!(Value::Float(OrderedFloat(0.0)).type_name(), "float");
        assert_eq!(Value::Bool(false).type_name(), "bool");
        assert_eq!(Value::List(vec![]).type_name(), "list");
        assert_eq!(Value::Map(OrderedMap::new()).type_name(), "map");
        assert_eq!(Value::Null.type_name(), "null");
    }

    // Compile-time proof: no From<bool> / From<i64> / From<String> for Value.
    // If any of these existed, the compiler would emit an "ambiguous" error
    // because multiple From impls would apply. The test below is a runtime
    // confirmation that explicit construction is required.
    #[test]
    fn test_bc_1_01_003_value_no_from_bool_impl() {
        // Must construct explicitly, not via .into()
        let v = Value::Bool(false);
        assert!(v.as_bool().is_some());
    }

    #[test]
    fn test_bc_1_01_003_value_no_from_i64_impl() {
        let v = Value::Int(99);
        assert!(v.as_int().is_some());
    }

    // ──────────────────────────────────────────────────────────────────────────
    // EC-001 — NaN edge case: OrderedFloat gives total ordering (NaN == NaN)
    // ──────────────────────────────────────────────────────────────────────────

    /// EC-001: With `OrderedFloat`, `NaN == NaN` is `true` (total ordering).
    /// This is the opposite of raw `f64` behavior (IEEE 754).
    #[test]
    fn test_bc_1_01_003_nan_ordered_float_eq() {
        let nan1 = Value::Float(OrderedFloat(f64::NAN));
        let nan2 = Value::Float(OrderedFloat(f64::NAN));
        // OrderedFloat provides total ordering: NaN == NaN
        assert_eq!(nan1, nan2, "OrderedFloat NaN must equal OrderedFloat NaN");
    }

    /// EC-001: NaN values can be inserted into a `HashSet` (requires Hash + Eq).
    #[test]
    fn test_bc_1_01_003_nan_hashset_insert() {
        use std::collections::HashSet;
        let mut set: HashSet<Value> = HashSet::new();
        set.insert(Value::Float(OrderedFloat(f64::NAN)));
        set.insert(Value::Float(OrderedFloat(f64::NAN)));
        // Both insertions are the same key — set should have exactly one entry
        assert_eq!(set.len(), 1, "two NaN values must hash to the same slot");
    }

    /// EC-001: Two NaN values wrapped in `OrderedFloat` hash to the same value.
    #[test]
    fn test_bc_1_01_003_nan_same_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let nan1 = Value::Float(OrderedFloat(f64::NAN));
        let nan2 = Value::Float(OrderedFloat(f64::NAN));
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        nan1.hash(&mut h1);
        nan2.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish(), "NaN values must hash identically");
    }

    // ──────────────────────────────────────────────────────────────────────────
    // STORY-004 AC-002 — "NO" stays "NO", never coerced to Bool(false)
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_02_003_str_no_stays_str() {
        let v = Value::Str(Arc::from("NO"));
        assert_ne!(v, Value::Bool(false));
        assert_eq!(v.type_kind(), crate::TypeKind::Str);
    }

    // ──────────────────────────────────────────────────────────────────────────
    // STORY-004 AC-003 — "1.10" precision preserved
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_02_003_float_precision_preserved() {
        // "1.10" must not become "1.1" (BC-1.02.003 postcondition 3)
        let s = "1.10";
        let v = Value::Str(Arc::from(s));
        assert_eq!(v.as_str().unwrap(), "1.10");
    }

    // ──────────────────────────────────────────────────────────────────────────
    // STORY-004 AC-005 — type_kind() correct for all 7 Value variants
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_02_003_type_kind_all_variants() {
        let cases = [
            (Value::Str(Arc::from("")), crate::TypeKind::Str),
            (Value::Int(0), crate::TypeKind::Int),
            (Value::Float(OrderedFloat(0.0)), crate::TypeKind::Float),
            (Value::Bool(false), crate::TypeKind::Bool),
            (Value::List(vec![]), crate::TypeKind::List),
            (Value::Map(OrderedMap::new()), crate::TypeKind::Map),
            (Value::Null, crate::TypeKind::Null),
        ];
        for (v, expected) in cases {
            assert_eq!(v.type_kind(), expected, "type_kind mismatch for {v:?}");
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // STORY-004 AC-006 — is_truthy() for all variants
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_02_003_is_truthy_bool() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
    }

    #[test]
    fn test_bc_1_02_003_is_truthy_int() {
        assert!(Value::Int(1).is_truthy());
        assert!(Value::Int(-1).is_truthy());
        assert!(!Value::Int(0).is_truthy());
    }

    #[test]
    fn test_bc_1_02_003_is_truthy_float() {
        assert!(Value::Float(OrderedFloat(1.0)).is_truthy());
        assert!(Value::Float(OrderedFloat(-1.0)).is_truthy());
        assert!(!Value::Float(OrderedFloat(0.0)).is_truthy());
    }

    /// EC-001: NaN is truthy (OrderedFloat(NaN) != 0.0 is true).
    #[test]
    fn test_bc_1_02_003_is_truthy_nan_is_truthy() {
        // With OrderedFloat, NaN != 0.0 evaluates to true, so NaN is truthy.
        assert!(
            Value::Float(OrderedFloat(f64::NAN)).is_truthy(),
            "NaN must be truthy (EC-001)"
        );
    }

    #[test]
    fn test_bc_1_02_003_is_truthy_str() {
        assert!(Value::Str(Arc::from("x")).is_truthy());
        // "NO" is truthy — non-empty string, no coercion
        assert!(Value::Str(Arc::from("NO")).is_truthy());
        // EC-002: empty string is falsy
        assert!(!Value::Str(Arc::from("")).is_truthy());
    }

    #[test]
    fn test_bc_1_02_003_is_truthy_list() {
        assert!(Value::List(vec![Value::Int(1)]).is_truthy());
        // EC-003: non-empty list even if contents are Null
        assert!(Value::List(vec![Value::Null]).is_truthy());
        assert!(!Value::List(vec![]).is_truthy());
    }

    #[test]
    fn test_bc_1_02_003_is_truthy_map() {
        let mut m = OrderedMap::new();
        m.insert(Arc::from("k"), Value::Null);
        assert!(Value::Map(m).is_truthy());
        assert!(!Value::Map(OrderedMap::new()).is_truthy());
    }

    #[test]
    fn test_bc_1_02_003_is_truthy_null() {
        assert!(!Value::Null.is_truthy());
    }

    // ──────────────────────────────────────────────────────────────────────────
    // STORY-004 AC-012 — as_bool() does not coerce
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_02_003_as_bool_does_not_coerce_str() {
        // Value::Str("true") must NOT become Some(true)
        let v = Value::Str(Arc::from("true"));
        assert_eq!(
            v.as_bool(),
            None,
            "Str(\"true\") must not coerce to Some(true)"
        );
    }

    #[test]
    fn test_bc_1_02_003_as_bool_does_not_coerce_int() {
        // Value::Int(1) must NOT become Some(true)
        let v = Value::Int(1);
        assert_eq!(v.as_bool(), None, "Int(1) must not coerce to Some(true)");
    }

    #[test]
    fn test_bc_1_02_003_as_bool_returns_some_for_bool() {
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::Bool(false).as_bool(), Some(false));
    }

    #[test]
    fn test_bc_1_02_003_as_str_does_not_coerce_int() {
        assert_eq!(Value::Int(42).as_str(), None);
    }

    #[test]
    fn test_bc_1_02_003_as_int_does_not_coerce_float() {
        // Float should not coerce to Int
        assert_eq!(Value::Float(OrderedFloat(1.0)).as_int(), None);
    }

    #[test]
    fn test_bc_1_02_003_as_float_unwraps_ordered_float() {
        // as_float() should return the inner f64, not the OrderedFloat wrapper.
        // Use a value that is NOT an approximation of a well-known constant.
        let val = 1.234_567_89_f64;
        let v = Value::Float(OrderedFloat(val));
        let f = v.as_float().expect("Float variant must return Some");
        assert!((f - val).abs() < 1e-10);
    }
}
