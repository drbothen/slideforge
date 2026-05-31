//! Type-level errors produced during evaluation.
//!
//! [`TypeError`] is the error type for type-checking and type-coercion failures
//! in the slideforge evaluator. It uses `thiserror` for structured error variants
//! and carries source spans for `miette` rendering.

use std::sync::Arc;

use crate::span::SourceSpan;

/// A type error produced during slideforge expression evaluation.
///
/// Every variant carries a [`SourceSpan`] so that `miette` can render a
/// colored source pointer to the exact location in the `.sf` file.
///
/// # Extensibility
///
/// `#[non_exhaustive]` ensures that adding new type-error variants in minor releases
/// does not force downstream callers to update exhaustive match arms (OBS-1 rule,
/// conventions.md §non_exhaustive-policy).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TypeError {
    /// A value was of the wrong type for the operation.
    #[error("type error at {span}: expected {expected}, got {actual}")]
    WrongType {
        /// The expected type name.
        expected: Arc<str>,
        /// The actual type name.
        actual: Arc<str>,
        /// Source location.
        span: SourceSpan,
    },

    /// An undefined variable was referenced.
    #[error("undefined variable '{name}' at {span}")]
    UndefinedVariable {
        /// The variable name.
        name: Arc<str>,
        /// Source location.
        span: SourceSpan,
    },

    /// A map key was not found.
    #[error("key '{key}' not found in map at {span}")]
    KeyNotFound {
        /// The missing key.
        key: Arc<str>,
        /// Source location.
        span: SourceSpan,
    },

    /// An index was out of bounds.
    #[error("index {index} out of bounds (length {length}) at {span}")]
    IndexOutOfBounds {
        /// The attempted index.
        index: i64,
        /// The actual list length.
        length: usize,
        /// Source location.
        span: SourceSpan,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_bc_1_01_type_error_wrong_type() {
        let err = TypeError::WrongType {
            expected: Arc::from("string"),
            actual: Arc::from("int"),
            span: SourceSpan::default(),
        };
        let msg = err.to_string();
        assert!(msg.contains("expected"));
        assert!(msg.contains("string"));
        assert!(msg.contains("int"));
    }

    #[test]
    fn test_bc_1_01_type_error_undefined_variable() {
        let err = TypeError::UndefinedVariable {
            name: Arc::from("my_var"),
            span: SourceSpan::default(),
        };
        let msg = err.to_string();
        assert!(msg.contains("my_var"));
    }

    #[test]
    fn test_bc_1_01_type_error_key_not_found() {
        let err = TypeError::KeyNotFound {
            key: Arc::from("title"),
            span: SourceSpan::default(),
        };
        let msg = err.to_string();
        assert!(msg.contains("title"));
    }

    #[test]
    fn test_bc_1_01_type_error_index_out_of_bounds() {
        let err = TypeError::IndexOutOfBounds {
            index: 5,
            length: 3,
            span: SourceSpan::default(),
        };
        let msg = err.to_string();
        assert!(msg.contains('5'));
        assert!(msg.contains('3'));
    }

    #[test]
    fn test_bc_1_01_type_error_clone() {
        let err = TypeError::UndefinedVariable {
            name: Arc::from("x"),
            span: SourceSpan::default(),
        };
        let err2 = err.clone();
        assert_eq!(err, err2);
    }

    #[test]
    fn test_bc_1_01_type_error_debug() {
        let err = TypeError::KeyNotFound {
            key: Arc::from("k"),
            span: SourceSpan::default(),
        };
        let s = format!("{err:?}");
        assert!(s.contains("KeyNotFound"));
    }
}
