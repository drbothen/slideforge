//! `TypeKind` — the discriminant of a [`crate::Value`] variant, used in type-error messages.
//!
//! When the evaluator encounters a type mismatch, it formats the error using
//! `TypeKind`'s [`std::fmt::Display`] impl to produce human-readable names like
//! `"string"`, `"integer"`, etc.  (E-EVL-003).

use std::fmt;

/// Identifies the kind of a [`crate::Value`] without carrying the value itself.
///
/// Returned by [`crate::Value::type_kind`] and used in type-error diagnostics
/// so that the evaluator can produce messages like
/// `"expected integer, got string"` without cloning or inspecting the full value.
///
/// ## Display format
///
/// Each variant renders as a lowercase English noun:
///
/// | Variant | Display |
/// |---------|---------|
/// | `Str`   | `"string"` |
/// | `Int`   | `"integer"` |
/// | `Float` | `"float"` |
/// | `Bool`  | `"boolean"` |
/// | `List`  | `"list"` |
/// | `Map`   | `"map"` |
/// | `Null`  | `"null"` |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeKind {
    /// A string value (`Arc<str>`).
    Str,
    /// A 64-bit signed integer.
    Int,
    /// A 64-bit floating-point value (wrapped in `OrderedFloat`).
    Float,
    /// A boolean value.
    Bool,
    /// An ordered list of values.
    List,
    /// An ordered map from string keys to values.
    Map,
    /// The null / absent value.
    Null,
}

impl fmt::Display for TypeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeKind::Str => write!(f, "string"),
            TypeKind::Int => write!(f, "integer"),
            TypeKind::Float => write!(f, "float"),
            TypeKind::Bool => write!(f, "boolean"),
            TypeKind::List => write!(f, "list"),
            TypeKind::Map => write!(f, "map"),
            TypeKind::Null => write!(f, "null"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ──────────────────────────────────────────────────────────────────────────
    // AC-004 — TypeKind Display for all 7 variants
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_02_003_type_kind_display_str() {
        assert_eq!(TypeKind::Str.to_string(), "string");
    }

    #[test]
    fn test_bc_1_02_003_type_kind_display_int() {
        assert_eq!(TypeKind::Int.to_string(), "integer");
    }

    #[test]
    fn test_bc_1_02_003_type_kind_display_float() {
        assert_eq!(TypeKind::Float.to_string(), "float");
    }

    #[test]
    fn test_bc_1_02_003_type_kind_display_bool() {
        assert_eq!(TypeKind::Bool.to_string(), "boolean");
    }

    #[test]
    fn test_bc_1_02_003_type_kind_display_list() {
        assert_eq!(TypeKind::List.to_string(), "list");
    }

    #[test]
    fn test_bc_1_02_003_type_kind_display_map() {
        assert_eq!(TypeKind::Map.to_string(), "map");
    }

    #[test]
    fn test_bc_1_02_003_type_kind_display_null() {
        assert_eq!(TypeKind::Null.to_string(), "null");
    }

    /// `TypeKind` implements Hash + Eq — required for use as a map key in error
    /// reporting tables (e.g., expected-type lookup by `TypeKind`).
    #[test]
    fn test_bc_1_02_003_type_kind_hash_eq() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(TypeKind::Str);
        set.insert(TypeKind::Int);
        set.insert(TypeKind::Str); // duplicate — should not increase count
        assert_eq!(set.len(), 2);
    }

    /// `TypeKind` implements Clone + Copy.
    #[test]
    fn test_bc_1_02_003_type_kind_copy() {
        let k = TypeKind::Bool;
        let k2 = k; // Copy
        let k3 = k; // still valid
        assert_eq!(k2, TypeKind::Bool);
        assert_eq!(k3, TypeKind::Bool);
    }

    /// `TypeKind` implements Debug.
    #[test]
    fn test_bc_1_02_003_type_kind_debug() {
        let s = format!("{:?}", TypeKind::Float);
        assert!(s.contains("Float"));
    }
}
